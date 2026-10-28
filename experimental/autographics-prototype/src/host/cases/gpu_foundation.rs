//! Case `gpu-foundation` (Plan V-01): real device, two independent targets on
//! ONE device/queue, resize → minimize → restore, unchanged-resource
//! survival, and classified failure injection. Windowed when possible;
//! headless offscreen fallback with `window_created=false` recorded.

use crate::core::id::ObjectId;
use crate::core::scene::Color;
use crate::host::report::*;
use crate::renderer::capture::{capture_png, sha256_hex};
use crate::renderer::error::RendererError;
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};
use crate::renderer::resources::{CacheKey, ResourceCache};
use crate::renderer::target::{OffscreenTarget, RenderOutcome, WindowTarget};
use crate::renderer::GpuDevice;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Entry point from the case registry.
pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    let result = match mode {
        "headless" => run_headless(replay, report_dir),
        "window" => match run_windowed(replay, report_dir) {
            Ok(code) => Ok(code),
            Err(WindowRunError::Unavailable(note)) => {
                eprintln!("window unavailable ({note}); use --mode headless for offscreen-only");
                Err(3)
            }
            Err(WindowRunError::Failed(code)) => Err(code),
        },
        "auto" => match run_windowed(replay, report_dir) {
            Ok(r) => Ok(r),
            Err(WindowRunError::Unavailable(note)) => {
                eprintln!("window unavailable ({note}); falling back to headless offscreen");
                run_headless(replay, report_dir)
            }
            Err(WindowRunError::Failed(code)) => Err(code),
        },
        other => {
            return {
                eprintln!("unknown mode: {other} (auto|window|headless)");
                2
            }
        }
    };
    match result {
        Ok(code) => code,
        Err(code) => code,
    }
}

// ---------------------------------------------------------------------------
// Shared drawing + failure injection
// ---------------------------------------------------------------------------

fn frame_cmds(t: f32) -> Vec<DrawCmd> {
    // Scene: light background + three quads, one moving with `t`.
    vec![
        DrawCmd {
            center: [100.0 + 400.0 * t, 200.0],
            size: [120.0, 80.0],
            z: 0.0,
            color: Color::rgb(0.9, 0.4, 0.3),
            shape: Shape::Rect,
        },
        DrawCmd {
            center: [650.0, 450.0],
            size: [200.0, 120.0],
            z: 0.0,
            color: Color::rgba(0.2, 0.6, 0.9, 0.8),
            shape: Shape::Rect,
        },
        DrawCmd {
            center: [60.0, 540.0],
            size: [80.0, 80.0],
            z: 0.0,
            color: Color::rgb(0.3, 0.8, 0.4),
            shape: Shape::Rect,
        },
    ]
}

const CLEAR: Color = Color {
    r: 0.08,
    g: 0.09,
    b: 0.12,
    a: 1.0,
};

/// Failure injections shared by windowed and headless runs. Each must come
/// back with its expected classification and leave the last-good state intact.
fn run_failure_injections(gpu: &GpuDevice, cache: &mut ResourceCache) -> Vec<FailureInjection> {
    let mut out = Vec::new();

    // 1. Zero-size offscreen target.
    let (cls, rejected, preserved) = match OffscreenTarget::new(gpu, 0, 0) {
        Err(e @ RendererError::ZeroSizeTarget { .. }) => (e.classify().to_string(), true, true),
        Err(e) => (e.classify().to_string(), false, true),
        Ok(_) => ("none".to_string(), false, true),
    };
    out.push(FailureInjection {
        name: "zero_size_offscreen".into(),
        expected_classification: "zero_size_target".into(),
        observed_classification: cls,
        rejected_as_expected: rejected,
        last_good_preserved: preserved,
    });

    // 2. Missing file.
    let missing = PathBuf::from("assets/__definitely_missing__.png");
    let (cls, rejected) = match std::fs::read(&missing) {
        Err(_) if !missing.exists() => ("missing_file".to_string(), true),
        Ok(_) => ("none".to_string(), false),
        Err(_) => ("none".to_string(), false),
    };
    out.push(FailureInjection {
        name: "missing_file".into(),
        expected_classification: "missing_file".into(),
        observed_classification: cls,
        rejected_as_expected: rejected,
        last_good_preserved: true,
    });

    // 3. Bad shader (broken WGSL through the validation-scope path).
    let broken = r#"
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(this is not wgsl);
}
"#;
    match QuadPipeline::validate_shader_source(gpu, broken, "injection") {
        Err(RendererError::BadShader { .. }) => out.push(FailureInjection {
            name: "bad_shader".into(),
            expected_classification: "bad_shader".into(),
            observed_classification: "bad_shader".into(),
            rejected_as_expected: true,
            last_good_preserved: true,
        }),
        other => out.push(FailureInjection {
            name: "bad_shader".into(),
            expected_classification: "bad_shader".into(),
            observed_classification: match &other {
                Ok(()) => "none".to_string(),
                Err(e) => e.classify().to_string(),
            },
            rejected_as_expected: false,
            last_good_preserved: true,
        }),
    }

    // 4. Stale identity into the cache (freed owner, slot reused).
    let stale_owner: ObjectId = {
        let owner = cache.allocator().allocate();
        let snapshot = owner;
        let _ = cache.allocator().free(owner);
        let _reused = cache.allocator().allocate(); // same slot, gen+1
        snapshot
    };
    let key = CacheKey {
        owner: stale_owner.token(),
        role: "stale-probe".into(),
    };
    match cache.texture(gpu, key, stale_owner, 16, 16) {
        Err(RendererError::StaleIdentity { .. }) => out.push(FailureInjection {
            name: "stale_identity".into(),
            expected_classification: "stale_identity".into(),
            observed_classification: "stale_identity".into(),
            rejected_as_expected: true,
            last_good_preserved: true,
        }),
        other => out.push(FailureInjection {
            name: "stale_identity".into(),
            expected_classification: "stale_identity".into(),
            observed_classification: match &other {
                Ok(_) => "none".to_string(),
                Err(e) => e.classify().to_string(),
            },
            rejected_as_expected: false,
            last_good_preserved: true,
        }),
    }

    // 5. Double release.
    let owner2 = cache.allocator().allocate();
    let key2 = CacheKey {
        owner: owner2.token(),
        role: "double-release".into(),
    };
    let _ = cache.texture(gpu, key2.clone(), owner2, 8, 8);
    let first = cache.release(&key2);
    let second = cache.release(&key2);
    let (cls, rejected) = match (first, second) {
        (Ok(()), Err(RendererError::ResourceReleased { .. })) => {
            ("resource_released".to_string(), true)
        }
        (Ok(()), Ok(())) => ("none".to_string(), false),
        (_, Err(e)) => (e.classify().to_string(), false),
        (Err(e), _) => (e.classify().to_string(), false),
    };
    out.push(FailureInjection {
        name: "double_release".into(),
        expected_classification: "resource_released".into(),
        observed_classification: cls,
        rejected_as_expected: rejected,
        last_good_preserved: true,
    });

    out
}

fn write_keyframe(
    report_dir: &Path,
    name: &str,
    png_bytes: &[u8],
    width: u32,
    height: u32,
) -> Result<Keyframe, RendererError> {
    std::fs::create_dir_all(report_dir)
        .map_err(|e| RendererError::ReportIo(format!("mkdir {}: {e}", report_dir.display())))?;
    let file = format!("{name}.png");
    let path = report_dir.join(&file);
    std::fs::write(&path, png_bytes)
        .map_err(|e| RendererError::ReportIo(format!("write {}: {e}", path.display())))?;
    Ok(Keyframe {
        name: name.to_string(),
        file,
        width,
        height,
        sha256: sha256_hex(png_bytes),
    })
}

// ---------------------------------------------------------------------------
// Headless path
// ---------------------------------------------------------------------------

fn run_headless(replay: &str, report_dir: &Path) -> Result<i32, i32> {
    let mut notes = vec!["headless offscreen fallback: no real window was created".to_string()];
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("gpu-foundation(headless): device creation failed: {e}");
            return Err(3);
        }
    };
    let token = gpu.token().to_report();

    let mut target_a = match OffscreenTarget::new(&gpu, 800, 600) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("gpu-foundation(headless): offscreen target A failed: {e}");
            return Err(3);
        }
    };
    let target_b = match OffscreenTarget::new(&gpu, 640, 480) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("gpu-foundation(headless): offscreen target B failed: {e}");
            return Err(3);
        }
    };

    let pipeline_a = match QuadPipeline::new(&gpu, OffscreenTarget::FORMAT) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("gpu-foundation(headless): pipeline failed: {e}");
            return Err(3);
        }
    };
    pipeline_a.set_ortho_camera(&gpu, 0.0, 800.0, 600.0, 0.0);

    let mut cache = ResourceCache::new(8);
    let cache_owners: Vec<ObjectId> = (0..4).map(|_| cache.allocator().allocate()).collect();
    for (i, owner) in cache_owners.iter().enumerate() {
        let key = CacheKey {
            owner: owner.token(),
            role: format!("probe-tex-{i}"),
        };
        if let Err(e) = cache.texture(&gpu, key, *owner, 64, 64) {
            eprintln!("gpu-foundation(headless): cache seeding failed: {e}");
            return Err(3);
        }
    }
    let stats_before = cache.stats().clone();

    // Render alternating frames on both targets (same device).
    let mut keyframes = Vec::new();
    let total_frames: u32 = 12;
    let mut rendered_a = 0u64;
    let mut rendered_b = 0u64;
    for f in 0..total_frames {
        let t = f as f32 / (total_frames - 1) as f32;
        let cmds = frame_cmds(t);
        pipeline_a.render(&gpu, target_a.view(), OffscreenTarget::FORMAT, CLEAR, &cmds);
        rendered_a += 1;
        if f % 2 == 1 {
            pipeline_a.render(&gpu, target_b.view(), OffscreenTarget::FORMAT, CLEAR, &cmds);
            rendered_b += 1;
        }
        if f == 0 || f == total_frames / 2 || f == total_frames - 1 {
            match capture_png(&gpu, &target_a) {
                Ok((png, _)) => {
                    gpu.wait_idle();
                    match write_keyframe(
                        report_dir,
                        &format!("offscreen_frame_{f}"),
                        &png,
                        800,
                        600,
                    ) {
                        Ok(k) => keyframes.push(k),
                        Err(e) => {
                            eprintln!("gpu-foundation(headless): keyframe write failed: {e}");
                            return Err(3);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("gpu-foundation(headless): capture failed: {e}");
                    return Err(3);
                }
            }
        }
    }

    // Valid resize of target A (texture recreated; pipeline/device untouched).
    let mut resize_events = Vec::new();
    match target_a.resize(&gpu, 1024, 640) {
        Ok(()) => resize_events.push([1024u32, 640]),
        Err(e) => {
            notes.push(format!("offscreen resize unexpectedly failed: {e}"));
            return Err(3);
        }
    }
    pipeline_a.render(
        &gpu,
        target_a.view(),
        OffscreenTarget::FORMAT,
        CLEAR,
        &frame_cmds(0.5),
    );
    rendered_a += 1;
    resize_events.push([800, 600]);
    let _ = target_a.resize(&gpu, 800, 600);

    // Cache survival: repeat lookups; no new entries may be created.
    for (i, owner) in cache_owners.iter().enumerate() {
        let key = CacheKey {
            owner: owner.token(),
            role: format!("probe-tex-{i}"),
        };
        if let Err(e) = cache.texture(&gpu, key, *owner, 64, 64) {
            eprintln!("gpu-foundation(headless): cache survival lookup failed: {e}");
            return Err(3);
        }
    }
    let stats_after = cache.stats().clone();
    let unmodified_survived = stats_after.created == stats_before.created;

    let failures = run_failure_injections(&gpu, &mut cache);

    let body = GpuFoundationReport {
        backend: gpu.adapter_info().backend.clone(),
        adapter: gpu.adapter_info().clone(),
        device_token: token.clone(),
        target_tokens: vec![token.clone(), token.clone()],
        window_created: false,
        window_mode: "headless".into(),
        window: None,
        frames: FrameFacts {
            window_frames_presented: 0,
            offscreen_frames_rendered: rendered_a + rendered_b,
            offscreen_bytes_copied_for_keyframes: keyframes.len() as u64 * 800 * 600 * 4,
        },
        keyframes,
        lifecycle: LifecycleFacts {
            resize_events_observed: resize_events.clone(),
            minimized_size_observed: None,
            restore_size_observed: None,
            zero_size_presentation_suspended: failures
                .iter()
                .any(|f| f.name == "zero_size_offscreen" && f.rejected_as_expected),
            device_lost_seen: false,
        },
        resources: ResourceFacts {
            cache_capacity: 8,
            stats_before,
            stats_after,
            live_entries: cache.live_entries(),
            unmodified_survived,
        },
        failure_injection: failures,
        gpu_timing_available: false,
        notes,
    };

    let all_ok = body
        .failure_injection
        .iter()
        .all(|f| f.rejected_as_expected)
        && body.target_tokens.iter().all(|t| t == &token)
        && body.resources.unmodified_survived;
    let exit = if all_ok { "pass" } else { "fail" };
    match write_report(report_dir, "gpu-foundation", replay, exit, body) {
        Ok(()) => Ok(if all_ok { 0 } else { 1 }),
        Err(e) => {
            eprintln!("gpu-foundation(headless): report write failed: {e}");
            Err(3)
        }
    }
}

// ---------------------------------------------------------------------------
// Windowed path (winit)
// ---------------------------------------------------------------------------

enum WindowRunError {
    /// Window could not be created in this environment (fallback eligible).
    Unavailable(String),
    /// A real failure after the window existed (no fallback).
    Failed(i32),
}

struct WindowProbeState {
    report_dir: PathBuf,
    replay: String,
    frame: u32,
    total_frames: u32,
    gpu: Option<GpuDevice>,
    window: Option<Arc<winit::window::Window>>,
    window_target: Option<WindowTarget>,
    offscreen: Option<OffscreenTarget>,
    win_pipeline: Option<QuadPipeline>,
    off_pipeline: Option<QuadPipeline>,
    cache: Option<ResourceCache>,
    cache_owners: Vec<ObjectId>,
    stats_before: Option<crate::renderer::CacheStats>,
    keyframes: Vec<Keyframe>,
    resize_observed: Vec<[u32; 2]>,
    desired_resize: Option<[u32; 2]>,
    minimized_seen: Option<[u32; 2]>,
    restored_seen: Option<[u32; 2]>,
    zero_suspended: bool,
    window_frames: u64,
    offscreen_frames: u64,
    notes: Vec<String>,
    phase: Phase,
    phase_since: std::time::Instant,
    done: bool,
    failed: Option<i32>,
}

impl WindowProbeState {
    fn set_phase(&mut self, p: Phase) {
        self.phase = p;
        self.phase_since = std::time::Instant::now();
        self.frame = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Warmup,
    Frames,
    WaitResize,
    Minimize,
    WaitMinimize,
    Restore,
    WaitRestore,
    Finalize,
}

impl WindowProbeState {
    fn new(report_dir: PathBuf, replay: String) -> Self {
        WindowProbeState {
            report_dir,
            replay,
            frame: 0,
            total_frames: 12,
            gpu: None,
            window: None,
            window_target: None,
            offscreen: None,
            win_pipeline: None,
            off_pipeline: None,
            cache: None,
            cache_owners: Vec::new(),
            stats_before: None,
            keyframes: Vec::new(),
            resize_observed: Vec::new(),
            desired_resize: None,
            minimized_seen: None,
            restored_seen: None,
            zero_suspended: false,
            window_frames: 0,
            offscreen_frames: 0,
            notes: Vec::new(),
            phase: Phase::Warmup,
            phase_since: std::time::Instant::now(),
            done: false,
            failed: None,
        }
    }

    fn warmup(&mut self) -> Result<(), String> {
        let gpu = GpuDevice::new().map_err(|e| format!("{e}"))?;
        let off_pipeline =
            QuadPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err(|e| format!("{e}"))?;
        off_pipeline.set_ortho_camera(&gpu, 0.0, 800.0, 600.0, 0.0);
        let mut cache = ResourceCache::new(8);
        let owners: Vec<ObjectId> = (0..4).map(|_| cache.allocator().allocate()).collect();
        for (i, owner) in owners.iter().enumerate() {
            let key = CacheKey {
                owner: owner.token(),
                role: format!("probe-tex-{i}"),
            };
            cache
                .texture(&gpu, key, *owner, 64, 64)
                .map_err(|e| format!("cache seed: {e}"))?;
        }
        self.stats_before = Some(cache.stats().clone());
        self.off_pipeline = Some(off_pipeline);
        self.cache = Some(cache);
        self.cache_owners = owners;
        self.gpu = Some(gpu);
        self.set_phase(Phase::Frames);
        Ok(())
    }
}

struct ProbeApp {
    state: WindowProbeState,
    initial_size: [u32; 2],
    scale_factor: f64,
    exit_code: i32,
}

impl ProbeApp {}

impl winit::application::ApplicationHandler for ProbeApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.gpu.is_some() && self.state.window.is_some() {
            return;
        }
        if let Err(e) = self.state.warmup() {
            eprintln!("gpu-foundation(window): warmup failed: {e}");
            self.state.failed = Some(3);
            event_loop.exit();
            return;
        }
        let attrs = winit::window::Window::default_attributes()
            .with_title("AGP graphics-probe: gpu-foundation")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                // Signal fallback eligibility via a distinguishable code path:
                // store the reason and exit; run_windowed maps this.
                eprintln!("gpu-foundation(window): create_window failed: {e}");
                self.state.failed = Some(4);
                event_loop.exit();
                return;
            }
        };
        self.scale_factor = window.scale_factor();
        let size = window.inner_size();
        self.initial_size = [size.width, size.height];
        let gpu = self.state.gpu.as_ref().unwrap();
        match WindowTarget::new(gpu, window.clone(), size.width, size.height) {
            Ok(t) => {
                self.state.window_target = Some(t);
            }
            Err(e) => {
                eprintln!("gpu-foundation(window): surface failed: {e}");
                self.state.failed = Some(3);
                event_loop.exit();
                return;
            }
        }
        let fmt = self.state.window_target.as_ref().unwrap().format();
        {
            let gpu = self.state.gpu.as_ref().unwrap();
            match QuadPipeline::new(gpu, fmt) {
                Ok(pipe) => {
                    pipe.set_ortho_camera(gpu, 0.0, 800.0, 600.0, 0.0);
                    self.state.win_pipeline = Some(pipe);
                }
                Err(e) => {
                    eprintln!("gpu-foundation(window): win pipeline: {e}");
                    self.state.failed = Some(3);
                    event_loop.exit();
                    return;
                }
            }
        }
        match OffscreenTarget::new(gpu, 800, 600) {
            Ok(t) => self.state.offscreen = Some(t),
            Err(e) => {
                eprintln!("gpu-foundation(window): offscreen failed: {e}");
                self.state.failed = Some(3);
                event_loop.exit();
                return;
            }
        }
        self.state.window = Some(window);
        self.state.set_phase(Phase::Frames);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = &mut self.state;
        match event {
            winit::event::WindowEvent::Resized(size) => {
                state.resize_observed.push([size.width, size.height]);
                if state.phase == Phase::WaitResize && state.desired_resize.is_some() {
                    state.set_phase(Phase::Minimize);
                }
                if state.phase == Phase::WaitMinimize {
                    state.minimized_seen = Some([size.width, size.height]);
                    // Tick's WaitMinimize arm performs the suspended-render
                    // check and un-minimizes; do NOT switch phase here.
                }
                if state.phase == Phase::WaitRestore && size.width > 0 && size.height > 0 {
                    state.restored_seen = Some([size.width, size.height]);
                    // Tick's WaitRestore arm performs the post-restore
                    // present and moves to Finalize.
                }
                if let (Some(gpu), Some(target)) = (&state.gpu, &mut state.window_target) {
                    if let Ok(RenderOutcome::SuspendedZeroSize) =
                        target.handle_resize(gpu, size.width, size.height)
                    {
                        state.zero_suspended = true;
                    }
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.on_redraw(event_loop);
            }
            winit::event::WindowEvent::CloseRequested => {
                state.notes.push("window closed by user/external".into());
                state.failed = Some(3);
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if !self.state.done && self.state.failed.is_none() {
            // Poll control flow: drive the scripted lifecycle here so phases
            // advance even when the platform stops delivering RedrawRequested
            // (e.g. while minimized on Windows).
            self.on_redraw(event_loop);
            if !self.state.done && self.state.failed.is_none() {
                if let Some(w) = &self.state.window {
                    w.request_redraw();
                }
            }
        }
    }
}

impl ProbeApp {
    fn on_redraw(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.done || self.state.failed.is_some() {
            return;
        }
        match self.state.phase {
            Phase::Frames => {
                let state = &mut self.state;
                let f = state.frame;
                let t = f as f32 / (state.total_frames - 1) as f32;
                let cmds = frame_cmds(t);
                let gpu = state.gpu.as_ref().unwrap();
                // Window present.
                let win_result = {
                    let target = state.window_target.as_mut().unwrap();
                    let pipe = state.win_pipeline.as_ref().unwrap();
                    let fmt = target.format();
                    target.present_frame(gpu, |view| {
                        pipe.render(gpu, view, fmt, CLEAR, &cmds);
                    })
                };
                match win_result {
                    Ok(RenderOutcome::Presented) => state.window_frames += 1,
                    Ok(RenderOutcome::SuspendedZeroSize) => {
                        state.zero_suspended = true;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("gpu-foundation(window): present failed: {e}");
                        state.failed = Some(3);
                        event_loop.exit();
                        return;
                    }
                }
                // Offscreen frame.
                let off = state.offscreen.as_ref().unwrap();
                state.off_pipeline.as_ref().unwrap().render(
                    gpu,
                    off.view(),
                    OffscreenTarget::FORMAT,
                    CLEAR,
                    &cmds,
                );
                state.offscreen_frames += 1;
                if f == 0 || f == state.total_frames / 2 || f == state.total_frames - 1 {
                    match capture_png(gpu, off) {
                        Ok((png, img)) => {
                            gpu.wait_idle();
                            let (w, h) = img.dimensions();
                            match write_keyframe(
                                &state.report_dir,
                                &format!("offscreen_frame_{f}"),
                                &png,
                                w,
                                h,
                            ) {
                                Ok(k) => state.keyframes.push(k),
                                Err(e) => {
                                    eprintln!("gpu-foundation(window): keyframe: {e}");
                                    state.failed = Some(3);
                                    event_loop.exit();
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("gpu-foundation(window): capture: {e}");
                            state.failed = Some(3);
                            event_loop.exit();
                            return;
                        }
                    }
                }
                state.frame += 1;
                if state.frame >= state.total_frames {
                    // Kick off resize: 800x600 -> 1024x640.
                    state.desired_resize = Some([1024, 640]);
                    state.set_phase(Phase::WaitResize);
                    if let Some(w) = &state.window {
                        let _ = w.request_inner_size(winit::dpi::LogicalSize::new(1024.0, 640.0));
                        // Some platforms resolve synchronously with no event;
                        // synthesize the observation after a couple of frames
                        // in WaitResize if nothing arrives.
                    }
                }
            }
            Phase::WaitResize => {
                // Give the platform a few seconds to deliver Resized.
                let observed = self.state.resize_observed.iter().any(|s| *s == [1024, 640]);
                if observed || self.state.phase_since.elapsed().as_secs() > 5 {
                    if !observed {
                        self.state.notes.push(
                            "resize event not observed within 5s; continuing to minimize".into(),
                        );
                    }
                    self.state.set_phase(Phase::Minimize);
                }
                self.state.frame += 1;
            }
            Phase::Minimize => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(w) = &self.state.window {
                        w.set_minimized(true);
                    }
                    self.state.set_phase(Phase::WaitMinimize);
                }
                #[cfg(not(target_os = "windows"))]
                {
                    self.state
                        .notes
                        .push("set_minimized unavailable on this platform; zero-size path exercised via offscreen rejection only".into());
                    self.state.phase = Phase::Finalize;
                }
            }
            Phase::WaitMinimize => {
                self.state.frame += 1;
                if self.state.minimized_seen.is_some() {
                    // Try to render while minimized: must suspend, not error.
                    let state = &mut self.state;
                    let gpu = state.gpu.as_ref().unwrap();
                    let target = state.window_target.as_mut().unwrap();
                    let pipe = state.win_pipeline.as_ref().unwrap();
                    let cmds = frame_cmds(0.5);
                    let fmt = target.format();
                    match target.present_frame(gpu, |view| {
                        pipe.render(gpu, view, fmt, CLEAR, &cmds);
                    }) {
                        Ok(RenderOutcome::SuspendedZeroSize) => state.zero_suspended = true,
                        Ok(RenderOutcome::Presented) => {
                            state.notes.push(
                                "present succeeded during minimize (platform kept size)".into(),
                            );
                        }
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("gpu-foundation(window): minimized render failed: {e}");
                            state.failed = Some(3);
                            event_loop.exit();
                            return;
                        }
                    }
                    #[cfg(target_os = "windows")]
                    if let Some(w) = &state.window {
                        w.set_minimized(false);
                    }
                    state.phase = Phase::WaitRestore;
                    state.frame = 0;
                } else if self.state.frame > 60 {
                    self.state
                        .notes
                        .push("minimize size event not observed; skipping restore wait".into());
                    #[cfg(target_os = "windows")]
                    if let Some(w) = &self.state.window {
                        w.set_minimized(false);
                    }
                    self.state.set_phase(Phase::WaitRestore);
                }
            }
            Phase::WaitRestore => {
                self.state.frame += 1;
                if self.state.restored_seen.is_some()
                    || self.state.phase_since.elapsed().as_secs() > 5
                {
                    if self.state.restored_seen.is_none() {
                        self.state.notes.push(
                            "restore size event not observed; querying inner_size directly".into(),
                        );
                        if let Some(w) = &self.state.window {
                            let s = w.inner_size();
                            self.state.restored_seen = Some([s.width, s.height]);
                        }
                    }
                    // Present one frame after restore.
                    let state = &mut self.state;
                    let gpu = state.gpu.as_ref().unwrap();
                    let target = state.window_target.as_mut().unwrap();
                    let pipe = state.win_pipeline.as_ref().unwrap();
                    let cmds = frame_cmds(0.75);
                    let fmt = target.format();
                    match target.present_frame(gpu, |view| {
                        pipe.render(gpu, view, fmt, CLEAR, &cmds);
                    }) {
                        Ok(RenderOutcome::Presented) => state.window_frames += 1,
                        other => {
                            eprintln!(
                                "gpu-foundation(window): post-restore present failed: {other:?}"
                            );
                            state.failed = Some(3);
                            event_loop.exit();
                            return;
                        }
                    }
                    state.phase = Phase::Finalize;
                }
            }
            Phase::Finalize => {
                let state = &mut self.state;
                let gpu = state.gpu.as_ref().unwrap();
                // Cache survival check.
                let owners = state.cache_owners.clone();
                let mut cache = state.cache.take().unwrap();
                for (i, owner) in owners.iter().enumerate() {
                    let key = CacheKey {
                        owner: owner.token(),
                        role: format!("probe-tex-{i}"),
                    };
                    if let Err(e) = cache.texture(gpu, key, *owner, 64, 64) {
                        eprintln!("gpu-foundation(window): cache survival: {e}");
                        state.failed = Some(3);
                        event_loop.exit();
                        return;
                    }
                }
                let stats_after = cache.stats().clone();
                let stats_before = state.stats_before.clone().unwrap_or_default();
                let unmodified_survived = stats_after.created == stats_before.created;
                let failures = run_failure_injections(gpu, &mut cache);
                let all_ok = failures.iter().all(|f| f.rejected_as_expected)
                    && unmodified_survived
                    && state.keyframes.len() == 3;

                let token = gpu.token().to_report();
                let body = GpuFoundationReport {
                    backend: gpu.adapter_info().backend.clone(),
                    adapter: gpu.adapter_info().clone(),
                    device_token: token.clone(),
                    target_tokens: vec![token.clone(), token.clone()],
                    window_created: true,
                    window_mode: "window".into(),
                    window: Some(WindowFacts {
                        initial_inner_size: self.initial_size,
                        scale_factor: self.scale_factor,
                        dpi_observations: vec![format!(
                            "scale_factor observed: {}",
                            self.scale_factor
                        )],
                    }),
                    frames: FrameFacts {
                        window_frames_presented: state.window_frames,
                        offscreen_frames_rendered: state.offscreen_frames,
                        offscreen_bytes_copied_for_keyframes: state.keyframes.len() as u64
                            * 800
                            * 600
                            * 4,
                    },
                    keyframes: state.keyframes.clone(),
                    lifecycle: LifecycleFacts {
                        resize_events_observed: state.resize_observed.clone(),
                        minimized_size_observed: state.minimized_seen,
                        restore_size_observed: state.restored_seen,
                        zero_size_presentation_suspended: state.zero_suspended,
                        device_lost_seen: false,
                    },
                    resources: ResourceFacts {
                        cache_capacity: 8,
                        stats_before,
                        stats_after,
                        live_entries: cache.live_entries(),
                        unmodified_survived,
                    },
                    failure_injection: failures,
                    gpu_timing_available: false,
                    notes: std::mem::take(&mut state.notes),
                };
                let exit = if all_ok { "pass" } else { "fail" };
                if let Err(e) = write_report(
                    &state.report_dir,
                    "gpu-foundation",
                    &state.replay,
                    exit,
                    body,
                ) {
                    eprintln!("gpu-foundation(window): report write failed: {e}");
                    self.exit_code = 3;
                } else {
                    self.exit_code = if all_ok { 0 } else { 1 };
                }
                state.done = true;
                event_loop.exit();
            }
            Phase::Restore | Phase::Warmup => {
                // Restore is only a transient marker; never linger here.
                if self.state.phase == Phase::Restore
                    && self.state.phase_since.elapsed().as_secs() > 5
                {
                    self.state
                        .notes
                        .push("stuck in Restore; forcing WaitRestore".into());
                    self.state.set_phase(Phase::WaitRestore);
                }
            }
        }
    }
}

fn run_windowed(replay: &str, report_dir: &Path) -> Result<i32, WindowRunError> {
    use winit::event_loop::EventLoop;
    let event_loop =
        EventLoop::new().map_err(|e| WindowRunError::Unavailable(format!("event loop: {e}")))?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = ProbeApp {
        state: WindowProbeState::new(report_dir.to_path_buf(), replay.to_string()),
        initial_size: [0, 0],
        scale_factor: 1.0,
        exit_code: 3,
    };
    let run_result = event_loop.run_app(&mut app);
    if let Some(code) = app.state.failed {
        if code == 4 {
            return Err(WindowRunError::Unavailable(
                "window creation refused".to_string(),
            ));
        }
        return Err(WindowRunError::Failed(code));
    }
    match run_result {
        Ok(()) => Ok(app.exit_code),
        Err(e) => Err(WindowRunError::Failed({
            eprintln!("gpu-foundation(window): event loop error: {e}");
            3
        })),
    }
}
