//! Case `shared-runtime` (Plan V-02): drives the G-base runtime through the
//! probe with the canonical replay. Evidence produced: two independent
//! replays agree (snapshot hash + exact event equality), change-log replay
//! reproduces live state, reset+replay reproduces state, stale ids are
//! rejected, and the bounded identity-keyed cache on the real device shows
//! zero rebuilds of untouched resources. A real window renders the demo sim
//! live; headless fallback records `window_created=false`.

use crate::core::demo::{
    canonical_action, run_canonical, DemoAction, DemoDriver, DemoEvent, DEMO_DT, DEMO_SEED,
};
use crate::core::runtime::Runtime;
use crate::core::scene::{Color, SceneChange};
use crate::host::report::{write_report, SharedRuntimeReport};
use crate::renderer::capture::sha256_hex;
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};
use crate::renderer::resources::{CacheKey, ResourceCache};
use crate::renderer::target::{OffscreenTarget, WindowTarget};
use crate::renderer::GpuDevice;
use std::path::Path;
use std::sync::Arc;

const TICKS: u64 = 180;

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    match mode {
        "headless" => run_offscreen(replay, report_dir, false),
        "window" => match run_windowed(replay, report_dir) {
            Ok(code) => code,
            Err(note) => {
                eprintln!("window unavailable ({note}); use --mode headless for offscreen-only");
                3
            }
        },
        "auto" => match run_windowed(replay, report_dir) {
            Ok(code) => code,
            Err(note) => {
                eprintln!("window unavailable ({note}); falling back to offscreen");
                run_offscreen(replay, report_dir, false)
            }
        },
        other => {
            eprintln!("unknown mode: {other} (auto|window|headless)");
            2
        }
    }
}

// ---------------------------------------------------------------------------
// Verification core (shared by windowed and offscreen paths)
// ---------------------------------------------------------------------------

struct Verification {
    report: SharedRuntimeReport,
    pass: bool,
}

fn verify(gpu: &GpuDevice, window_created: bool, window_mode: &str) -> Verification {
    let mut notes = Vec::new();

    // 1. Two independent canonical replays.
    let (rt_a, events_a) = run_canonical(TICKS);
    let (rt_b, events_b) = run_canonical(TICKS);
    let snap_a = rt_a.scene().snapshot();
    let snap_b = rt_b.scene().snapshot();
    let hash_a = sha256_hex(snap_a.as_bytes());
    let hash_b = sha256_hex(snap_b.as_bytes());
    let snapshots_agree = snap_a == snap_b;
    let events_equal = events_a == events_b;
    assert!(!events_a.is_empty());

    // 2. Change-log replay reproduces live final state.
    let replayed = rt_a.log().replay();
    let changelog_replay_matches = replayed.snapshot() == snap_a;

    // 3. Reset + same seed/stream reproduces state.
    let mut rt_c = Runtime::new(DEMO_DT, DEMO_SEED);
    let mut driver_c = DemoDriver::default();
    let mut events_c = Vec::new();
    for tick in 0..TICKS {
        let step = rt_c.step(&mut driver_c, vec![canonical_action(tick)]);
        events_c.extend(step.into_iter().map(|e| e.payload));
    }
    // (rt_c IS a fresh replay; now reset it and run again — the post-reset
    // run must match the pre-reset run.)
    let snap_before = rt_c.scene().snapshot();
    let events_before = events_c.clone();
    rt_c.reset(DEMO_SEED);
    let mut driver_c = DemoDriver::default();
    let mut events_c2 = Vec::new();
    for tick in 0..TICKS {
        let step = rt_c.step(&mut driver_c, vec![canonical_action(tick)]);
        events_c2.extend(step.into_iter().map(|e| e.payload));
    }
    let reset_replay_matches = rt_c.scene().snapshot() == snap_before && events_c2 == events_before;

    // 4. Stale-id rejection (destroyed decoy id, late mutation).
    let mut stale_id_rejections = 0u32;
    {
        let mut rt: Runtime<DemoAction, DemoEvent> = Runtime::new(DEMO_DT, DEMO_SEED);
        let mut driver = DemoDriver::default();
        let mut decoy = None;
        for tick in 0..12u64 {
            let _ = rt.step(&mut driver, vec![canonical_action(tick)]);
            if tick == 6 {
                decoy = driver.decoy;
            }
        }
        let stale = decoy.expect("decoy alive at tick 6");
        let before = rt.scene().snapshot();
        let errs = rt.submit(&[SceneChange::SetColor(stale, Color::rgb(0.0, 0.0, 0.0))]);
        if errs.len() == 1 && before == rt.scene().snapshot() {
            stale_id_rejections += 1;
        }
        // Double-destroy also rejected.
        let errs = rt.submit(&[SceneChange::Destroy(stale)]);
        if errs.len() == 1 {
            stale_id_rejections += 1;
        }
    }

    // 5. Late events from a previous session rejected.
    let late_event_rejected = {
        let mut rt: Runtime<DemoAction, DemoEvent> = Runtime::new(DEMO_DT, DEMO_SEED);
        let mut driver = DemoDriver::default();
        let step = rt.step(&mut driver, vec![DemoAction::None]);
        let stale_event = step[0].clone();
        rt.reset(DEMO_SEED);
        rt.accept_external_event(&stale_event).is_err()
    };
    if !late_event_rejected {
        notes.push("late-event rejection missing".into());
    }

    // 6. Cache composition on the real device: identity-keyed, no rebuilds.
    let mut cache = ResourceCache::new(8);
    let owners: Vec<_> = (0..4).map(|_| cache.allocator().allocate()).collect();
    for (i, owner) in owners.iter().enumerate() {
        let key = CacheKey {
            owner: owner.token(),
            role: format!("runtime-tex-{i}"),
        };
        let _ = cache.texture(gpu, key, *owner, 32, 32);
    }
    let created_before = cache.stats().created;
    // Second full lookup pass: all hits, zero new creations.
    for (i, owner) in owners.iter().enumerate() {
        let key = CacheKey {
            owner: owner.token(),
            role: format!("runtime-tex-{i}"),
        };
        let _ = cache.texture(gpu, key, *owner, 32, 32);
    }
    // Stale owner into the cache must be rejected and counted.
    let stale_owner = {
        let o = cache.allocator().allocate();
        let snap = o;
        let _ = cache.allocator().free(o);
        let _ = cache.allocator().allocate();
        snap
    };
    let stale_key = CacheKey {
        owner: stale_owner.token(),
        role: "runtime-stale".into(),
    };
    let cache_stale_rejected = cache.texture(gpu, stale_key, stale_owner, 16, 16).is_err();
    let stats = cache.stats().clone();
    let untouched_resource_rebuilds = stats.created != created_before;
    if cache_stale_rejected {
        stale_id_rejections += 1;
    }

    // 7. RNG reproducibility across separately-built runtimes.
    let jitter = |ev: &[DemoEvent]| -> Vec<f32> {
        ev.iter()
            .filter_map(|e| match e {
                DemoEvent::Spawned { jitter, .. } => Some(*jitter),
                _ => None,
            })
            .collect()
    };
    let rng_reproducible = jitter(&events_a) == jitter(&events_b) && !jitter(&events_a).is_empty();

    let pass = snapshots_agree
        && events_equal
        && changelog_replay_matches
        && reset_replay_matches
        && stale_id_rejections >= 3
        && late_event_rejected
        && !untouched_resource_rebuilds
        && rng_reproducible;

    let report = SharedRuntimeReport {
        adapter: gpu.adapter_info().clone(),
        device_token: gpu.token().to_report(),
        window_created,
        window_mode: window_mode.into(),
        ticks: TICKS,
        snapshot_hash_a: hash_a,
        snapshot_hash_b: hash_b,
        snapshots_agree,
        events_equal,
        event_count: events_a.len(),
        changelog_replay_matches,
        changelog_replay_hash: sha256_hex(replayed.snapshot().as_bytes()),
        reset_replay_matches,
        stale_id_rejections,
        late_event_rejected,
        cache_created: stats.created,
        cache_hits: stats.hits,
        cache_misses: stats.misses,
        cache_stale_rejected: stats.stale_rejected,
        untouched_resource_rebuilds,
        rng_reproducible,
        gpu_timing_available: false,
        notes,
    };
    Verification { report, pass }
}

fn finish(report_dir: &Path, replay: &str, v: Verification) -> i32 {
    let exit = if v.pass { "pass" } else { "fail" };
    match write_report(report_dir, "shared-runtime", replay, exit, v.report) {
        Ok(()) => {
            if v.pass {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("shared-runtime: report write failed: {e}");
            3
        }
    }
}

// ---------------------------------------------------------------------------
// Offscreen path (no window; used as fallback and by --mode headless)
// ---------------------------------------------------------------------------

fn run_offscreen(replay: &str, report_dir: &Path, _unused: bool) -> i32 {
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("shared-runtime(offscreen): device failed: {e}");
            return 3;
        }
    };
    // Exercise one offscreen render so the GPU path is genuinely involved.
    let target = match OffscreenTarget::new(&gpu, 320, 240) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("shared-runtime(offscreen): target failed: {e}");
            return 3;
        }
    };
    let pipe = match QuadPipeline::new(&gpu, OffscreenTarget::FORMAT) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("shared-runtime(offscreen): pipeline failed: {e}");
            return 3;
        }
    };
    pipe.set_ortho_camera(&gpu, 0.0, 800.0, 600.0, 0.0);
    pipe.render(
        &gpu,
        target.view(),
        OffscreenTarget::FORMAT,
        CLEAR_OFFSCREEN,
        &[],
    );

    let v = verify(&gpu, false, "offscreen");
    finish(report_dir, replay, v)
}

const CLEAR_OFFSCREEN: Color = Color {
    r: 0.07,
    g: 0.07,
    b: 0.10,
    a: 1.0,
};

// ---------------------------------------------------------------------------
// Windowed path: renders the demo sim live, then runs verification
// ---------------------------------------------------------------------------

struct SharedApp {
    gpu: Option<GpuDevice>,
    window: Option<Arc<winit::window::Window>>,
    target: Option<WindowTarget>,
    pipeline: Option<QuadPipeline>,
    rt: Runtime<DemoAction, DemoEvent>,
    driver: DemoDriver,
    report_dir: std::path::PathBuf,
    replay: String,
    exit_code: i32,
    verified: bool,
}

impl SharedApp {
    fn draw_cmds(rt: &Runtime<DemoAction, DemoEvent>) -> Vec<DrawCmd> {
        let mut cmds = Vec::new();
        for obj in rt.scene().objects() {
            // Arena x∈[-1,1] → px∈[100,700]; y placement per object index.
            let px = 100.0 + (obj.transform.translation.x + 1.0) * 0.5 * 600.0;
            let py = 300.0 + obj.transform.translation.y * 100.0;
            cmds.push(DrawCmd {
                center: [px, py],
                size: [
                    40.0 * obj.transform.scale.x.max(0.2),
                    40.0 * obj.transform.scale.y.max(0.2),
                ],
                z: 0.0,
                color: obj.color,
                shape: Shape::Rect,
            });
        }
        cmds
    }
}

impl winit::application::ApplicationHandler for SharedApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let gpu = match GpuDevice::new() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("shared-runtime(window): device failed: {e}");
                self.exit_code = 3;
                event_loop.exit();
                return;
            }
        };
        let attrs = winit::window::Window::default_attributes()
            .with_title("AGP graphics-probe: shared-runtime")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("shared-runtime(window): create_window failed: {e}");
                self.exit_code = 4; // fallback-eligible
                event_loop.exit();
                return;
            }
        };
        let size = window.inner_size();
        let target = match WindowTarget::new(&gpu, window.clone(), size.width, size.height) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("shared-runtime(window): surface failed: {e}");
                self.exit_code = 3;
                event_loop.exit();
                return;
            }
        };
        let fmt = target.format();
        let pipe = match QuadPipeline::new(&gpu, fmt) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("shared-runtime(window): pipeline failed: {e}");
                self.exit_code = 3;
                event_loop.exit();
                return;
            }
        };
        pipe.set_ortho_camera(&gpu, 0.0, 800.0, 600.0, 0.0);
        self.pipeline = Some(pipe);
        self.target = Some(target);
        self.window = Some(window);
        self.gpu = Some(gpu);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::Resized(size) => {
                if let (Some(gpu), Some(t)) = (&self.gpu, &mut self.target) {
                    let _ = t.handle_resize(gpu, size.width, size.height);
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.on_frame(event_loop);
            }
            winit::event::WindowEvent::CloseRequested => {
                self.exit_code = 3;
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.gpu.is_some() && !self.verified {
            self.on_frame(event_loop);
            if !self.verified {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
        }
    }
}

impl SharedApp {
    fn on_frame(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.verified || self.gpu.is_none() {
            return;
        }
        let tick = self.rt.tick();
        if tick < TICKS {
            let action = canonical_action(tick);
            let _ = self.rt.step(&mut self.driver, vec![action]);
        }
        // Render current state.
        let cmds = Self::draw_cmds(&self.rt);
        let (gpu, target, pipe) = (
            self.gpu.as_ref().unwrap(),
            self.target.as_mut().unwrap(),
            self.pipeline.as_ref().unwrap(),
        );
        let fmt = target.format();
        match target.present_frame(gpu, |view| {
            pipe.render(gpu, view, fmt, CLEAR_OFFSCREEN, &cmds);
        }) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("shared-runtime(window): present failed: {e}");
                self.exit_code = 3;
                event_loop.exit();
                return;
            }
        }
        if tick >= TICKS {
            self.verified = true;
            let v = verify(gpu, true, "window");
            self.exit_code = finish(&self.report_dir, &self.replay, v);
            event_loop.exit();
        }
    }
}

fn run_windowed(replay: &str, report_dir: &Path) -> Result<i32, String> {
    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::new().map_err(|e| format!("event loop: {e}"))?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = SharedApp {
        gpu: None,
        window: None,
        target: None,
        pipeline: None,
        rt: Runtime::new(DEMO_DT, DEMO_SEED),
        driver: DemoDriver::default(),
        report_dir: report_dir.to_path_buf(),
        replay: replay.to_string(),
        exit_code: 3,
        verified: false,
    };
    let run_result = event_loop.run_app(&mut app);
    if app.exit_code == 4 {
        return Err("window creation refused".into());
    }
    match run_result {
        Ok(()) => Ok(app.exit_code),
        Err(e) => Err(format!("event loop: {e}")),
    }
}
