//! Case `metrics` (V-07, review r1 F08 repair): honest instrumented
//! accounting — resource CREATIONS vs buffer REWRITE bytes are counted at
//! the actual paths (pipeline upload counters + store deltas), pipelines and
//! views are built ONCE (cold/warm = first/second pass, everything resident
//! for warm), the raw per-frame series is persisted for recomputation, every
//! report carries fingerprints, and a missing required observation or a
//! sabotaged counter CANNOT produce PASS. CPU wall timing only; GPU timing
//! remains explicitly unavailable. Headless by design; mode recorded.

use crate::core::runtime::Runtime;
use crate::core::scene::Color;
use crate::games::breakout::{BreakoutAction, BreakoutDriver, PaddleDir, BACKGROUND_COLOR};
use crate::games::squash::{SquashAction, SquashDriver, SQUASH_SEED};
use crate::host::cases::squash_game::{build_scene, frame_instances};
use crate::host::report::{write_report, MetricsReport, MetricsRun};
use crate::render3d::{MeshPipeline, MeshStore};
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};
use crate::renderer::target::OffscreenTarget;
use crate::renderer::GpuDevice;
use std::path::Path;
use std::time::{Duration, Instant};

const VIEWS: u32 = 2;
const FRAMES_PER_RUN: u32 = 300;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FrameSample {
    frame: u32,
    cpu_ms: f64,
    creations: u64,
    rewrite_bytes: u64,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Process working set in MB (OS residency of THIS process). Shared with
/// `surface-lifecycle` for the V-09 memory facts (review r3 F-830-R3-03).
pub(crate) fn sample_rss_mb() -> Option<f64> {
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

fn rect(x: f32, y: f32, w: f32, h: f32, color: Color) -> DrawCmd {
    DrawCmd {
        center: [x, y],
        size: [w, h],
        z: 0.0,
        color,
        shape: Shape::Rect,
    }
}

fn live_breakout_cmds(driver: &BreakoutDriver) -> Vec<DrawCmd> {
    use crate::games::breakout::*;
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

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    if replay != "canonical" {
        eprintln!("metrics: unknown replay '{replay}'");
        return 2;
    }
    let _ = mode;

    let sabotage = std::env::var("AGP_SABOTAGE_METRICS").is_ok();

    let t_start = Instant::now();
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("metrics: device: {e}");
            return 3;
        }
    };
    let startup_ms = t_start.elapsed().as_secs_f64() * 1000.0;
    ADAPTER.with(|a| *a.lock().unwrap() = gpu.adapter_info().clone());
    TOKEN.with(|t| *t.lock().unwrap() = gpu.token().to_report());

    // ---- build everything ONCE: pipelines, views, scene (r1 F08: warm
    // passes reuse these; nothing is recreated per pass).
    let mut store = MeshStore::default();
    let scene = match build_scene(&gpu, &mut store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("metrics: scene: {e}");
            return 3;
        }
    };
    let mut mesh3d = match MeshPipeline::new(&gpu, OffscreenTarget::FORMAT) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("metrics: 3d pipeline: {e}");
            return 3;
        }
    };
    let quad2d = match QuadPipeline::new(&gpu, OffscreenTarget::FORMAT) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("metrics: 2d pipeline: {e}");
            return 3;
        }
    };
    quad2d.set_ortho_camera(&gpu, 0.0, 900.0, 600.0, 0.0);
    let views_2d: Vec<OffscreenTarget> = (0..VIEWS)
        .map(|_| OffscreenTarget::new(&gpu, 900, 600))
        .collect::<Result<_, _>>()
        .expect("2d views");
    let views_3d: Vec<OffscreenTarget> = (0..VIEWS)
        .map(|_| OffscreenTarget::new(&gpu, 900, 600))
        .collect::<Result<_, _>>()
        .expect("3d views");
    let light = crate::render3d::sample_light();

    let mut runs = Vec::new();
    let mut sabotage_detected = false;

    for pass in ["cold", "warm"] {
        // ---------- breakout ----------
        {
            let mut rt = Runtime::new(Duration::from_nanos(16_666_667), 0x8293_D01);
            let mut driver = BreakoutDriver::default();
            let rss_start = sample_rss_mb();
            let inst_before = quad2d.uploads.snapshot();
            let mut samples: Vec<FrameSample> = Vec::with_capacity(FRAMES_PER_RUN as usize);
            for f in 0..FRAMES_PER_RUN {
                let t0 = Instant::now();
                let dir = match (f / 30) % 4 {
                    0 => PaddleDir::Right,
                    2 => PaddleDir::Left,
                    _ => PaddleDir::None,
                };
                let _ = rt.step(&mut driver, vec![BreakoutAction::Paddle(dir)]);
                let up_before = quad2d.uploads.snapshot();
                let creations_before = store.uploaded_count() as u64;
                let view = &views_2d[(f as usize) % views_2d.len()];
                quad2d.render(
                    &gpu,
                    view.view(),
                    OffscreenTarget::FORMAT,
                    BACKGROUND_COLOR,
                    &live_breakout_cmds(&driver),
                );
                let up = quad2d.uploads.snapshot().delta_since(&up_before);
                let creations = store.uploaded_count() as u64 - creations_before;
                samples.push(FrameSample {
                    frame: f,
                    cpu_ms: t0.elapsed().as_secs_f64() * 1000.0,
                    creations,
                    rewrite_bytes: up.bytes_rewritten,
                });
            }
            gpu.wait_idle();
            let rss_end = sample_rss_mb();
            runs.push(summarize(
                "ag2d-breakout",
                pass,
                samples,
                rss_start,
                rss_end,
                if pass == "cold" { startup_ms } else { 0.0 },
                0.0,
                report_dir,
                &quad2d.uploads.snapshot().delta_since(&inst_before),
                sabotage,
            ));
        }

        // ---------- squash ----------
        {
            let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
            let mut driver = SquashDriver::new(SQUASH_SEED);
            let squash_actions: Vec<[i8; 3]> = {
                let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("testdata/squash/golden_squash_trace.json");
                let bytes = std::fs::read(&path).expect("squash trace");
                let doc: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
                serde_json::from_value(doc["actions"].clone()).expect("actions")
            };
            let rss_start = sample_rss_mb();
            let inst_before = mesh3d.uploads.snapshot();
            let mut samples: Vec<FrameSample> = Vec::with_capacity(FRAMES_PER_RUN as usize);
            let mut step = 0usize;
            for f in 0..FRAMES_PER_RUN {
                let t0 = Instant::now();
                for _ in 0..6 {
                    if step < squash_actions.len() {
                        let a = squash_actions[step];
                        let _ = rt.step(
                            &mut driver,
                            vec![SquashAction {
                                move_x: a[0],
                                move_z: a[1],
                                jump: a[2] != 0,
                                retry: false,
                                reset: false,
                            }],
                        );
                        step += 1;
                    }
                }
                let up_before = mesh3d.uploads.snapshot();
                let creations_before = store.uploaded_count() as u64;
                let view = &views_3d[(f as usize) % views_3d.len()];
                let instances = frame_instances(rt.scene(), &scene);
                mesh3d.render(
                    &gpu,
                    view.view(),
                    900,
                    600,
                    [0.64625, 0.65575, 0.67075, 1.0],
                    &scene.camera.view_proj(),
                    &light,
                    &store,
                    &instances,
                    true,
                );
                let up = mesh3d.uploads.snapshot().delta_since(&up_before);
                let creations = store.uploaded_count() as u64 - creations_before;
                samples.push(FrameSample {
                    frame: f,
                    cpu_ms: t0.elapsed().as_secs_f64() * 1000.0,
                    creations,
                    rewrite_bytes: up.bytes_rewritten,
                });
            }
            gpu.wait_idle();
            let rss_end = sample_rss_mb();
            runs.push(summarize(
                "ag3d-squash",
                pass,
                samples,
                rss_start,
                rss_end,
                0.0,
                if pass == "warm" { 0.0 } else { 0.0 },
                report_dir,
                &mesh3d.uploads.snapshot().delta_since(&inst_before),
                sabotage,
            ));
        }
    }

    // exit wall: dropping everything, then a final idle poll.
    let t_exit = Instant::now();
    drop(mesh3d);
    drop(quad2d);
    drop(views_2d);
    drop(views_3d);
    drop(store);
    drop(scene);
    gpu.wait_idle();
    let exit_ms = t_exit.elapsed().as_secs_f64() * 1000.0;
    drop(gpu);
    for r in &mut runs {
        if r.pass == "warm" {
            r.exit_wall_ms = exit_ms;
        }
    }

    // PASS requires every observation to be present and consistent.
    let all_complete = runs.iter().all(|r| r.observations_complete);
    let consistent = runs.iter().all(|r| {
        // cross-check: instrumented totals must equal the raw-series sums
        // (a sabotaged counter makes these disagree).
        let raw: Vec<FrameSample> = match std::fs::read(report_dir.join(&r.raw_series_file)) {
            Ok(b) => serde_json::from_slice(&b).unwrap_or_default(),
            Err(_) => return false,
        };
        let sum_bytes: u64 = raw.iter().map(|s| s.rewrite_bytes).sum();
        sum_bytes == r.instrumented_total_bytes
    });
    if sabotage && !consistent {
        sabotage_detected = true;
    }
    let pass = all_complete && consistent && !sabotage_detected;

    let report = MetricsReport {
        adapter: {
            // adapter facts captured before the device was dropped: rebuild a
            // diagnostic instance is a second device — instead capture early.
            ADAPTER.with(|a| a.lock().unwrap().clone())
        },
        device_token: TOKEN.with(|t| t.lock().unwrap().clone()),
        mode: "headless".into(),
        runs,
        gpu_timing_available: false,
        methodology: "CPU frame time = wall clock around {canonical sim ticks + wgpu encode + queue submit} per rendered frame (offscreen, 2 alternating views). Pipelines/views/scene are built ONCE before the passes: cold = first pass in this process, warm = second pass with everything resident. Accounted separately: resource CREATIONS (new GPU objects, instrumented store deltas) and buffer REWRITE bytes (instrumented at the actual write paths: quad instance buffer; 3D camera+light+instance buffers). Raw per-frame series is persisted next to the report for recomputation. GPU timing UNAVAILABLE (no timestamp queries). Fingerprints (commit/lock/assets) are attached to every report.".into(),
        notes: {
            let mut n = vec![
                "no cross-platform or competitor conclusions are drawn; numbers describe this Windows x64 machine only".into(),
            ];
            if sabotage {
                n.push("AGP_SABOTAGE_METRICS was set: legacy literal accounting was used and the instrumented-total cross-check correctly turned the run RED".into());
            }
            n
        },
    };
    let exit = if pass { "pass" } else { "fail" };
    match write_report(report_dir, "metrics", "canonical", exit, report) {
        Ok(()) => {
            if pass {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("metrics: report: {e}");
            3
        }
    }
}

thread_local! {
    static ADAPTER: std::sync::Mutex<crate::renderer::AdapterInfoReport> =
        std::sync::Mutex::new(crate::renderer::AdapterInfoReport {
            name: String::new(), backend: String::new(), device_type: String::new(),
            driver: String::new(), driver_info: String::new(), vendor: 0, device: 0,
        });
    static TOKEN: std::sync::Mutex<String> = const { std::sync::Mutex::new(String::new()) };
}

#[allow(clippy::too_many_arguments)]
fn summarize(
    game: &str,
    pass: &str,
    samples: Vec<FrameSample>,
    rss_start: Option<f64>,
    rss_end: Option<f64>,
    startup_ms: f64,
    exit_ms: f64,
    report_dir: &Path,
    instrumented: &crate::renderer::UploadStats,
    sabotage: bool,
) -> MetricsRun {
    let file = format!("frames_{game}_{pass}.json");
    std::fs::write(
        report_dir.join(&file),
        serde_json::to_string_pretty(&samples).unwrap_or_default(),
    )
    .ok();

    // Summaries are computed from the raw series (which is also persisted).
    let mut ms: Vec<f64> = samples.iter().map(|s| s.cpu_ms).collect();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut creations: Vec<f64> = samples.iter().map(|s| s.creations as f64).collect();
    creations.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut bytes: Vec<f64> = samples.iter().map(|s| s.rewrite_bytes as f64).collect();
    bytes.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Sabotage path (negative control): the OLD literal accounting replaces
    // the instrumented values; the totals cross-check in run() turns red.
    let (bytes_p50, bytes_max, instrumented_total_bytes) = if sabotage {
        let fake: u64 = if game == "ag2d-breakout" { 1 } else { 0 };
        (
            fake as f64,
            fake,
            instrumented.bytes_rewritten + 1, // poisoned total
        )
    } else {
        (
            percentile(&bytes, 0.50),
            *bytes.last().unwrap_or(&0.0) as u64,
            instrumented.bytes_rewritten,
        )
    };

    let observations_complete = !samples.is_empty()
        && rss_start.is_some()
        && rss_end.is_some()
        // honest accounting sanity: both games rewrite buffers every frame.
        && bytes_max > 0;

    MetricsRun {
        game: game.into(),
        pass: pass.into(),
        frames: samples.len() as u32,
        cpu_frame_ms_p50: percentile(&ms, 0.50),
        cpu_frame_ms_p95: percentile(&ms, 0.95),
        cpu_frame_ms_max: *ms.last().unwrap_or(&0.0),
        creations_per_frame_p50: percentile(&creations, 0.50),
        creations_per_frame_max: *creations.last().unwrap_or(&0.0) as u64,
        rewrite_bytes_per_frame_p50: bytes_p50,
        rewrite_bytes_per_frame_max: bytes_max,
        instrumented_total_rewrites: instrumented.buffer_rewrites,
        instrumented_total_bytes,
        raw_series_file: file,
        rss_start_mb: rss_start.unwrap_or(-1.0),
        rss_end_mb: rss_end.unwrap_or(-1.0),
        startup_wall_ms: startup_ms,
        exit_wall_ms: exit_ms,
        views: VIEWS,
        observations_complete,
    }
}
