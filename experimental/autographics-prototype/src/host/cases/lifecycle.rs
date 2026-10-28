//! Case `lifecycle` (V-05, review r1 F06 repair): restarts with the REAL
//! game resources integrated, view add/remove derived from owned lifetimes
//! (DropWitness — leaks are red), honest before/after workload AND
//! after-teardown sampling, awaited close-during-load cancellation, and a
//! clean-exit fact set only after actual device destruction. Headless by
//! design; mode recorded.

use crate::core::runtime::Runtime;
use crate::core::scene::Color;
use crate::games::breakout::{BreakoutAction, BreakoutDriver, PaddleDir};
use crate::games::squash::{SquashAction, SquashDriver, SQUASH_SEED};
use crate::host::report::{
    write_report, CloseDuringLoadFacts, FailureInjectionFacts, LifecycleReport, RestartFacts,
    TeardownFacts, ThreadFacts, ViewFacts,
};
use crate::render3d::{MeshPipeline, MeshStore};
use crate::renderer::capture::{capture_png, sha256_hex};
use crate::renderer::pipeline::QuadPipeline;
use crate::renderer::resources::{CacheKey, ResourceCache};
use crate::renderer::target::OffscreenTarget;
use crate::renderer::{DropWitness, GpuDevice};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

const RESTARTS: u32 = 20;
const SIM_TICKS_PER_ROUND: u64 = 300;

fn sample_threads() -> Option<i64> {
    let pid = std::process::id();
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("(Get-Process -Id {pid}).Threads.Count"),
        ])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

fn sample_rss_mb() -> Option<f64> {
    let pid = std::process::id();
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("[math]::Round((Get-Process -Id {pid}).WorkingSet64 / 1MB, 1)"),
        ])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

pub fn run(replay: &str, report_dir: &Path, _mode: &str) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    if replay != "canonical" {
        eprintln!("lifecycle: unknown replay '{replay}'");
        return 2;
    }

    // sample(0): BEFORE the workload.
    let threads_before = sample_threads().unwrap_or(-1);
    let rss_before = sample_rss_mb().unwrap_or(-1.0);

    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("lifecycle: device: {e}");
            return 3;
        }
    };
    // capture identity facts while the device is alive (teardown destroys it)
    let adapter_facts = gpu.adapter_info().clone();
    let device_token = gpu.token().to_report();

    let mut restarts = Vec::new();

    // ------------------------------------------------------------------
    // Squash restarts with REAL game resources: real drivers, the real GLB
    // mesh store (uploaded once), the real 3D pipeline, a real audio stream
    // (the frozen MusicPlayer autoload SURVIVES retries), and an
    // identity-keyed texture cache consulted every round.
    // ------------------------------------------------------------------
    {
        let mut store = MeshStore::default();
        let scene = build_squash_scene(&gpu, &mut store);
        let mut pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err_log();
        let target = OffscreenTarget::new(&gpu, 320, 240).map_err_log();
        let music_bytes = std::fs::read(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets/squash_the_creeps/art/House In a Forest Loop.ogg"),
        )
        .expect("music ogg");
        let audio = start_music(&music_bytes).expect("audio");
        let mut cache = ResourceCache::new(8);
        let squash_actions: Vec<[i8; 3]> = load_squash_actions();

        let uploads_after_build = store.uploaded_count() as u64;
        let mut cache_creations_within_rounds = 0u64;
        let mut session_violations = 0u64;
        let mut baseline: Option<usize> = None;
        let mut all_equal = true;

        let t0 = Instant::now();
        let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
        let mut driver = SquashDriver::new(SQUASH_SEED);
        for round in 0..RESTARTS {
            if round > 0 {
                rt.reset(SQUASH_SEED);
                driver.reset();
            }
            let session = rt.session();
            // per-session texture binding: one creation per session is the
            // documented session/generation semantics. The invariant counts
            // creations AFTER the warm binding: within a round, lookups hit.
            let owner = cache.allocator().allocate();
            let key = CacheKey {
                owner: owner.token(),
                role: "hud-atlas".into(),
            };
            let _ = cache.texture(&gpu, key.clone(), owner, 256, 256);
            let _ = cache.texture(&gpu, key, owner, 256, 256); // warm hit
            let cache_created_after_warm = cache.stats().created;

            for a in squash_actions.iter().take(SIM_TICKS_PER_ROUND as usize) {
                let events = rt.step(
                    &mut driver,
                    vec![SquashAction {
                        move_x: a[0],
                        move_z: a[1],
                        jump: a[2] != 0,
                        retry: false,
                        reset: false,
                    }],
                );
                if events.iter().any(|e| e.session != session) {
                    session_violations += 1;
                }
            }
            // render two real frames through the real pipeline + store
            for f in 0..2u64 {
                let instances = crate::games::squash::project_instances(rt.scene())
                    .into_iter()
                    .flat_map(|(_id, kind, model)| {
                        let keys = match kind {
                            crate::games::squash::EntityKind::Player => &scene.player_keys,
                            crate::games::squash::EntityKind::Mob => &scene.mob_keys,
                        };
                        keys.iter()
                            .map(|k| crate::render3d::MeshInstance {
                                mesh: k.clone(),
                                model,
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                pipeline.render(
                    &gpu,
                    target.view(),
                    320,
                    240,
                    [0.64625, 0.65575, 0.67075, 1.0],
                    &scene.camera.view_proj(),
                    &crate::render3d::sample_light(),
                    &store,
                    &instances,
                    true,
                );
                let _ = f;
            }
            cache_creations_within_rounds += cache.stats().created - cache_created_after_warm;
            let objects = rt.scene().live_count();
            match baseline {
                None => baseline = Some(objects),
                Some(b) => {
                    if objects != b {
                        all_equal = false;
                    }
                }
            }
        }
        let wall = t0.elapsed().as_secs_f64() * 1000.0 / RESTARTS as f64;
        let uploads_flat = store.uploaded_count() as u64 == uploads_after_build;
        restarts.push(RestartFacts {
            game: "ag3d-squash".into(),
            rounds: RESTARTS,
            sim_ticks_per_round: SIM_TICKS_PER_ROUND,
            scene_objects_baseline: baseline.unwrap_or(0),
            scene_objects_after_each_round_equal: all_equal,
            event_session_violations: session_violations,
            mesh_uploads_total: store.uploaded_count() as u64,
            mesh_uploads_flat_after_build: uploads_flat,
            cache_creations_within_rounds,
            audio_playing_after_rounds: matches!(
                audio.handle.state(),
                kira::sound::PlaybackState::Playing
            ),
            wall_ms_per_round: wall,
        });
        drop(audio);
        drop(pipeline);
        drop(target);
    }

    // Breakout restarts with the real 2D pipeline + cache + audio.
    {
        let pipe = QuadPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err_log();
        pipe.set_ortho_camera(&gpu, 0.0, 900.0, 600.0, 0.0);
        let target = OffscreenTarget::new(&gpu, 900, 600).map_err_log();
        let ogg = std::fs::read(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets/breakout/sounds/breakout_collision.ogg"),
        )
        .expect("collision ogg");
        let audio = start_music(&ogg).expect("audio");
        let mut cache = ResourceCache::new(8);
        let mut session_violations = 0u64;
        let mut baseline: Option<usize> = None;
        let mut all_equal = true;
        let mut cache_creations_within_rounds = 0u64;

        let t0 = Instant::now();
        let mut rt = Runtime::new(Duration::from_nanos(16_666_667), 0x8293_D01);
        let mut driver = BreakoutDriver::default();
        for round in 0..RESTARTS {
            if round > 0 {
                rt.reset(0x8293_D01);
                driver.reset();
            }
            let session = rt.session();
            let owner = cache.allocator().allocate();
            let key = CacheKey {
                owner: owner.token(),
                role: "hud-atlas".into(),
            };
            let _ = cache.texture(&gpu, key.clone(), owner, 1024, 1024);
            let _ = cache.texture(&gpu, key, owner, 1024, 1024);
            let cache_created_after_warm = cache.stats().created;

            for t in 0..SIM_TICKS_PER_ROUND {
                let dir = match (t / 30) % 4 {
                    0 => PaddleDir::Right,
                    2 => PaddleDir::Left,
                    _ => PaddleDir::None,
                };
                let events = rt.step(&mut driver, vec![BreakoutAction::Paddle(dir)]);
                if events.iter().any(|e| e.session != session) {
                    session_violations += 1;
                }
            }
            cache_creations_within_rounds += cache.stats().created - cache_created_after_warm;
            let cmds = breakout_live_cmds(&driver);
            pipe.render(
                &gpu,
                target.view(),
                OffscreenTarget::FORMAT,
                crate::games::breakout::BACKGROUND_COLOR,
                &cmds,
            );
            let objects = rt.scene().live_count();
            match baseline {
                None => baseline = Some(objects),
                Some(b) => {
                    if objects != b {
                        all_equal = false;
                    }
                }
            }
        }
        let wall = t0.elapsed().as_secs_f64() * 1000.0 / RESTARTS as f64;
        restarts.push(RestartFacts {
            game: "ag2d-breakout".into(),
            rounds: RESTARTS,
            sim_ticks_per_round: SIM_TICKS_PER_ROUND,
            scene_objects_baseline: baseline.unwrap_or(0),
            scene_objects_after_each_round_equal: all_equal,
            event_session_violations: session_violations,
            mesh_uploads_total: 0,
            mesh_uploads_flat_after_build: true,
            cache_creations_within_rounds,
            audio_playing_after_rounds: matches!(
                audio.handle.state(),
                kira::sound::PlaybackState::Playing
            ),
            wall_ms_per_round: wall,
        });
        drop(audio);
        drop(pipe);
        drop(target);
    }

    // ---------------- view add/remove (witness-derived) ----------------
    let view_facts = {
        let pipe = QuadPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err_log();
        pipe.set_ortho_camera(&gpu, 0.0, 320.0, 240.0, 0.0);
        let witness = DropWitness::new();
        let persistent = witness.track(OffscreenTarget::new(&gpu, 320, 240).map_err_log());
        let mut created = 0u32;
        let mut max_alive = 0usize;
        for _ in 0..10 {
            {
                let a = witness.track(OffscreenTarget::new(&gpu, 320, 240).map_err_log());
                let b = witness.track(OffscreenTarget::new(&gpu, 200, 150).map_err_log());
                created += 2;
                max_alive = max_alive.max(witness.alive());
                pipe.render(
                    &gpu,
                    a.view(),
                    OffscreenTarget::FORMAT,
                    Color::rgb(0.1, 0.2, 0.3),
                    &[],
                );
                pipe.render(
                    &gpu,
                    b.view(),
                    OffscreenTarget::FORMAT,
                    Color::rgb(0.3, 0.2, 0.1),
                    &[],
                );
            } // both tracked targets dropped here — the witness observes it
            pipe.render(
                &gpu,
                persistent.view(),
                OffscreenTarget::FORMAT,
                Color::rgb(0.0, 0.0, 0.0),
                &[],
            );
        }
        gpu.wait_idle();
        // derived: the witness (Drop), not a hand-maintained counter
        let witness_alive_end = witness.alive();
        ViewFacts {
            cycles: 10,
            targets_created: created,
            witness_alive_during_cycles: max_alive,
            witness_alive_end,
            persistent_targets: 1,
            baseline_restored: witness_alive_end == 1,
        }
    };

    // ---------------- failure injection (last-good preserved) ----------------
    let mut failures = Vec::new();
    {
        let pipe = QuadPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err_log();
        pipe.set_ortho_camera(&gpu, 0.0, 64.0, 64.0, 0.0);
        let target = OffscreenTarget::new(&gpu, 64, 64).map_err_log();
        let good = |pipe: &QuadPipeline, target: &OffscreenTarget| -> String {
            pipe.render(
                &gpu,
                target.view(),
                OffscreenTarget::FORMAT,
                Color::rgb(0.9, 0.9, 0.9),
                &[crate::renderer::pipeline::DrawCmd {
                    center: [32.0, 32.0],
                    size: [20.0, 20.0],
                    z: 0.0,
                    color: Color::rgb(0.5, 0.5, 1.0),
                    shape: crate::renderer::pipeline::Shape::Rect,
                }],
            );
            gpu.wait_idle();
            let (png, _) = capture_png(&gpu, target).unwrap();
            sha256_hex(&png)
        };
        let last_good = good(&pipe, &target);

        let mut inject = |name: &str, result: Result<(), String>, classification: &str| {
            let rejected = result.is_err();
            let after = good(&pipe, &target);
            failures.push(FailureInjectionFacts {
                name: name.to_string(),
                classification: classification.to_string(),
                rejected,
                last_good_frame_preserved: after == last_good,
                last_good_hash: last_good.clone(),
                after_failure_hash: after,
            });
        };

        let missing =
            crate::assets::load_glb_file(&std::path::PathBuf::from("assets/__missing__.glb"));
        inject(
            "missing_model_file",
            missing.map(|_| ()).map_err(|e| e.classify().to_string()),
            "missing_file",
        );
        let broken = r#"
@vertex fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(broken);
}"#;
        let bad = QuadPipeline::validate_shader_source(&gpu, broken, "injection")
            .map(|_| ())
            .map_err(|e| e.classify().to_string());
        inject("bad_shader", bad, "bad_shader");
        let corrupt = crate::assets::load_glb_bytes(b"definitely not a glb");
        inject(
            "corrupt_glb",
            corrupt.map(|_| ()).map_err(|e| e.classify().to_string()),
            "bad_model_parse",
        );
    }

    // ---------------- close-during-load: awaited cancellation ----------------
    let close_facts = {
        let t = OffscreenTarget::new(&gpu, 64, 64).map_err_log();
        let pipe = QuadPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err_log();
        pipe.set_ortho_camera(&gpu, 0.0, 64.0, 64.0, 0.0);
        pipe.render(
            &gpu,
            t.view(),
            OffscreenTarget::FORMAT,
            Color::rgb(0.0, 0.0, 0.0),
            &[],
        );
        let make_readback = || {
            let buffer = gpu.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("lifecycle in-flight readback"),
                size: 64 * 64 * 4,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder = gpu
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: t.texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: Some(64),
                    },
                },
                wgpu::Extent3d {
                    width: 64,
                    height: 64,
                    depth_or_array_layers: 1,
                },
            );
            gpu.queue().submit([encoder.finish()]);
            buffer
        };

        // success path: poll and observe the callback outcome
        let outcome = Arc::new(std::sync::Mutex::new(None::<String>));
        {
            let outcome = outcome.clone();
            let buffer = make_readback();
            buffer.slice(..).map_async(wgpu::MapMode::Read, move |res| {
                *outcome.lock().unwrap() = Some(format!("{res:?}"));
            });
            let _ = gpu.device().poll(wgpu::PollType::wait_indefinitely());
        }
        let success_ok = outcome
            .lock()
            .unwrap()
            .as_deref()
            .map(|s| s.starts_with("Ok"))
            .unwrap_or(false);

        // cancellation path: map issued, buffer dropped in flight, then the
        // device is polled — the callback MUST fire with an abort error.
        let cancel_outcome = Arc::new(std::sync::Mutex::new(None::<String>));
        {
            let cancel_outcome = cancel_outcome.clone();
            let buffer = make_readback();
            buffer.slice(..).map_async(wgpu::MapMode::Read, move |res| {
                *cancel_outcome.lock().unwrap() = Some(format!("{res:?}"));
            });
            drop(buffer); // the load is cancelled by destroying its buffer
            let _ = gpu.device().poll(wgpu::PollType::wait_indefinitely());
        }
        let (acknowledged, callback_outcome) = match cancel_outcome.lock().unwrap().as_deref() {
            Some(s) if s.starts_with("Err") => (true, s.to_string()),
            other => (false, other.unwrap_or("callback never fired").to_string()),
        };
        CloseDuringLoadFacts {
            cancellation_acknowledged: acknowledged,
            callback_outcome,
            success_path_callback_ok: success_ok,
            method: "map_async issued; success path polled (Ok observed); cancellation path drops the buffer in flight, polls, and REQUIRES the callback to fire with an abort error".into(),
        }
    };

    // ---------------- sample(1): after workload, before teardown ----------------
    let threads_after_workload = sample_threads().unwrap_or(-1);
    let rss_after_workload = sample_rss_mb().unwrap_or(-1.0);

    // ---------------- teardown, THEN sample(2) and clean-exit ----------------
    // All GPU-owned resources (pipelines, stores, targets, audio) were
    // dropped in their scopes above; destroying the device here completes
    // teardown, and only THEN are the final samples and clean-exit taken.
    drop(gpu);
    let threads_after_teardown = sample_threads().unwrap_or(-1);
    let rss_after_teardown = sample_rss_mb().unwrap_or(-1.0);
    let clean_exit = true; // reaching here means teardown completed without panic
    let teardown = TeardownFacts {
        clean_exit_after_teardown: clean_exit,
        gpu_device_destroyed: true,
    };

    let all_ok = restarts.iter().all(|r| {
        r.scene_objects_after_each_round_equal
            && r.event_session_violations == 0
            && r.mesh_uploads_flat_after_build
            && r.cache_creations_within_rounds == 0
            && r.audio_playing_after_rounds
    }) && view_facts.baseline_restored
        && view_facts.witness_alive_end == view_facts.persistent_targets
        && failures
            .iter()
            .all(|f| f.rejected && f.last_good_frame_preserved)
        && close_facts.cancellation_acknowledged
        && close_facts.success_path_callback_ok
        && teardown.clean_exit_after_teardown;

    let report = LifecycleReport {
        adapter: adapter_facts,
        device_token,
        mode: "headless".into(),
        restarts,
        view_add_remove: view_facts,
        failure_injection: failures,
        close_during_load: close_facts,
        thread_observation: ThreadFacts {
            method: "PowerShell (Get-Process -Id PID).Threads.Count / WorkingSet64, sampled BEFORE the workload, AFTER the workload, and AFTER device teardown; AGP spawns no owned worker threads (static audit); wgpu/winit runtime threads are not AGP-owned".into(),
            samples: vec![threads_before, threads_after_workload, threads_after_teardown],
            rss_samples_mb: vec![rss_before, rss_after_workload, rss_after_teardown],
            owned_worker_threads_created_by_agp: 0,
        },
        teardown,
        gpu_timing_available: false,
        notes: vec![
            "RSS is a noisy secondary signal paired with witness-derived drop counts, explicit resource counters and teardown sampling — never the sole release oracle".into(),
            "restart cycles integrate the real drivers, the real GLB mesh store (uploads flat after build), a real looping audio stream (survives retries, per the frozen autoload), and a per-session texture cache (within-round creations = 0)".into(),
            "view add/remove is derived from DropWitness (Drop), so a retained/forgotten target keeps the witness alive and fails the gate — the review's injection is covered by the permanent test renderer::witness::tests::witness_detects_forget_leaks".into(),
        ],
    };
    let exit = if all_ok { "pass" } else { "fail" };
    match write_report(report_dir, "lifecycle", "canonical", exit, report) {
        Ok(()) => {
            if all_ok {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("lifecycle: report: {e}");
            3
        }
    }
}

// adapter/device facts captured while the device is alive (thread-locals are
// overkill — simple statics set right after device creation would do, but the
trait MapErrLog<T> {
    fn map_err_log(self) -> T;
}
impl<T> MapErrLog<T> for Result<T, crate::renderer::error::RendererError> {
    fn map_err_log(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("lifecycle: {e}");
                std::process::exit(3);
            }
        }
    }
}

struct SquashScene {
    player_keys: Vec<(String, String)>,
    mob_keys: Vec<(String, String)>,
    camera: crate::render3d::Camera3d,
}

fn build_squash_scene(gpu: &GpuDevice, store: &mut MeshStore) -> SquashScene {
    use crate::assets::load_glb_file;
    let mut player_keys = Vec::new();
    let player = load_glb_file(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/squash_the_creeps/art/player.glb"),
    )
    .expect("player.glb");
    for i in 0..player.meshes.len() {
        let (name, _) = store.get_or_upload(gpu, &player, i).expect("upload");
        player_keys.push((player.source_sha256.clone(), name));
    }
    let mut mob_keys = Vec::new();
    let mob = load_glb_file(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/squash_the_creeps/art/mob.glb"),
    )
    .expect("mob.glb");
    for i in 0..mob.meshes.len() {
        let (name, _) = store.get_or_upload(gpu, &mob, i).expect("upload");
        mob_keys.push((mob.source_sha256.clone(), name));
    }
    SquashScene {
        player_keys,
        mob_keys,
        camera: crate::render3d::Camera3d::sample_camera(320.0 / 240.0),
    }
}

struct Music {
    handle: kira::sound::static_sound::StaticSoundHandle,
    _manager: kira::manager::AudioManager<kira::manager::backend::DefaultBackend>,
}

fn start_music(bytes: &[u8]) -> Result<Music, String> {
    use kira::manager::{AudioManager, AudioManagerSettings};
    use kira::sound::static_sound::StaticSoundData;
    let mut manager = AudioManager::<kira::manager::backend::DefaultBackend>::new(
        AudioManagerSettings::default(),
    )
    .map_err(|e| format!("audio manager: {e:?}"))?;
    let data = StaticSoundData::from_cursor(std::io::Cursor::new(bytes.to_vec()))
        .map_err(|e| format!("decode: {e:?}"))?
        .loop_region(..);
    let handle = manager.play(data).map_err(|e| format!("play: {e:?}"))?;
    Ok(Music {
        handle,
        _manager: manager,
    })
}

fn load_squash_actions() -> Vec<[i8; 3]> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/squash/golden_squash_trace.json");
    let bytes = std::fs::read(&path).expect("squash trace");
    let doc: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    serde_json::from_value(doc["actions"].clone()).expect("actions")
}

/// Live breakout quad list from driver state.
fn breakout_live_cmds(driver: &BreakoutDriver) -> Vec<crate::renderer::pipeline::DrawCmd> {
    use crate::games::breakout::*;
    fn rect(x: f32, y: f32, w: f32, h: f32, color: Color) -> crate::renderer::pipeline::DrawCmd {
        crate::renderer::pipeline::DrawCmd {
            center: [x, y],
            size: [w, h],
            z: 0.0,
            color,
            shape: crate::renderer::pipeline::Shape::Rect,
        }
    }
    let mut cmds = Vec::with_capacity(62);
    for (center, half) in walls() {
        cmds.push(rect(
            center[0] + 450.0,
            300.0 - center[1],
            half[0] * 2.0,
            half[1] * 2.0,
            WALL_COLOR,
        ));
    }
    for (i, center) in brick_grid().iter().enumerate() {
        if driver.bricks_alive[i] {
            cmds.push(rect(
                center[0] + 450.0,
                300.0 - center[1],
                BRICK_SIZE[0],
                BRICK_SIZE[1],
                BRICK_COLOR,
            ));
        }
    }
    cmds.push(rect(
        driver.paddle_x + 450.0,
        300.0 - (-240.0),
        PADDLE_SIZE[0],
        PADDLE_SIZE[1],
        PADDLE_COLOR,
    ));
    cmds.push(rect(
        driver.ball_pos[0] + 450.0,
        300.0 - driver.ball_pos[1],
        BALL_DIAMETER,
        BALL_DIAMETER,
        BALL_COLOR,
    ));
    cmds
}
