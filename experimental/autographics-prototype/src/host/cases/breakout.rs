//! Case `ag2d-breakout` (Plan V-03 / AC-03): the frozen-sample Breakout
//! prototype. `--replay canonical` drives the FROZEN action stream from the
//! golden trace and verifies against it (positions ≤1e-3, score/contacts/
//! sounds exact, bounce-side sequence exact), real kira audio playback
//! (trigger count == contact count), HUD text keyframes, and 3 host-reset
//! rounds with G-base isolation. `--replay live` is the interactive mode
//! (ArrowLeft/ArrowRight; R = host-extension reset).

use crate::core::runtime::{Runtime, TickEvent};
use crate::games::breakout::*;
use crate::host::report::{write_report, BreakoutReport, BreakoutResetEvidence, Keyframe};
use crate::render2d::{GlyphQuad, TextRenderer};
use crate::renderer::capture::{capture_png, sha256_hex};
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};
use crate::renderer::resources::{CacheKey, ResourceCache};
use crate::renderer::target::{OffscreenTarget, WindowTarget};
use crate::renderer::GpuDevice;
use kira::manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings};
use kira::sound::static_sound::StaticSoundData;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

const SEED: u64 = 0x8293_D01;
const HUD_CHARS: &str = "Score: 0123456789";
const TICKS_PER_FRAME: usize = 120;
const RESET_PREFIX_TICKS: usize = 600;

fn agp_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

// ---------------------------------------------------------------------------
// Golden trace loading
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TraceTick {
    t: u64,
    bx: f32,
    by: f32,
    px: f32,
    score: u32,
    contacts: u32,
    sounds: u64,
}

#[derive(Debug, Deserialize)]
struct TraceContact {
    t: u64,
    kind: String,
    side: String,
    brick: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct GoldenTrace {
    ticks: Vec<TraceTick>,
    actions: Vec<i8>,
    contacts_detail: Vec<TraceContact>,
}

fn load_trace() -> Result<GoldenTrace, String> {
    let path = agp_file("testdata/breakout/golden_trace.json");
    let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("parse trace: {e}"))
}

fn kind_str(k: ColliderKind) -> &'static str {
    match k {
        ColliderKind::WallLeft => "wall_left",
        ColliderKind::WallRight => "wall_right",
        ColliderKind::WallBottom => "wall_bottom",
        ColliderKind::WallTop => "wall_top",
        ColliderKind::Paddle => "paddle",
        ColliderKind::Brick(_) => "brick",
    }
}

fn side_str(s: Side) -> &'static str {
    match s {
        Side::Left => "L",
        Side::Right => "R",
        Side::Top => "T",
        Side::Bottom => "B",
    }
}

fn action_of(a: i8) -> BreakoutAction {
    BreakoutAction::Paddle(match a {
        -1 => PaddleDir::Left,
        1 => PaddleDir::Right,
        _ => PaddleDir::None,
    })
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

/// World (frozen arena) → window pixels (900×600 logical, y down).
fn to_screen(wx: f32, wy: f32) -> [f32; 2] {
    [wx + 450.0, 300.0 - wy]
}

fn draw_cmds(driver: &BreakoutDriver) -> Vec<DrawCmd> {
    let mut cmds = Vec::with_capacity(6 + BRICK_COUNT);
    for (center, half) in walls() {
        let [sx, sy] = to_screen(center[0], center[1]);
        cmds.push(DrawCmd {
            center: [sx, sy],
            size: [half[0] * 2.0, half[1] * 2.0],
            z: 0.0,
            color: WALL_COLOR,
            shape: Shape::Rect,
        });
    }
    let [sx, sy] = to_screen(driver.paddle_x, paddle_y());
    cmds.push(DrawCmd {
        center: [sx, sy],
        size: PADDLE_SIZE,
        z: 0.0,
        color: PADDLE_COLOR,
        shape: Shape::Rect,
    });
    let [sx, sy] = to_screen(driver.ball_pos[0], driver.ball_pos[1]);
    cmds.push(DrawCmd {
        center: [sx, sy],
        size: [BALL_DIAMETER, BALL_DIAMETER],
        z: 0.0,
        color: BALL_COLOR,
        shape: Shape::Circle,
    });
    for (i, center) in brick_grid().iter().enumerate() {
        if driver.bricks_alive[i] {
            let [sx, sy] = to_screen(center[0], center[1]);
            cmds.push(DrawCmd {
                center: [sx, sy],
                size: BRICK_SIZE,
                z: 0.0,
                color: BRICK_COLOR,
                shape: Shape::Rect,
            });
        }
    }
    cmds
}

/// Frozen HUD: "Score: " in TEXT_COLOR + number in SCORE_COLOR, 33 px,
/// top-left padding 5 px (manifest §1.2).
fn hud_quads(text: &TextRenderer, score: u32) -> Vec<GlyphQuad> {
    let pad = SCOREBOARD_TEXT_PADDING;
    let mut quads = text.layout("Score: ", pad, pad, TEXT_COLOR);
    let number_x = pad + text.measure("Score: ");
    quads.extend(text.layout(&score.to_string(), number_x, pad, SCORE_COLOR));
    quads
}

// ---------------------------------------------------------------------------
// Audio (kira real playback path)
// ---------------------------------------------------------------------------

struct Audio {
    manager: AudioManager<DefaultBackend>,
    data: StaticSoundData,
}

impl Audio {
    fn new(ogg_bytes: &[u8]) -> Result<Self, String> {
        // The frozen semantics allow unbounded overlapping one-shots (one per
        // contact, no throttling); size the sound pool for the canonical
        // fast-forward burst (210 contacts within seconds).
        let settings = AudioManagerSettings {
            capacities: kira::manager::Capacities {
                sound_capacity: 1024,
                ..Default::default()
            },
            ..Default::default()
        };
        let manager = AudioManager::<DefaultBackend>::new(settings)
            .map_err(|e| format!("audio manager (cpal output): {e:?}"))?;
        let data = StaticSoundData::from_cursor(std::io::Cursor::new(ogg_bytes.to_vec()))
            .map_err(|e| format!("decode collision ogg: {e:?}"))?;
        Ok(Audio { manager, data })
    }

    fn play(&mut self) -> bool {
        self.manager.play(self.data.clone()).is_ok()
    }
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    match replay {
        "canonical" => match mode {
            "headless" => canonical_impl(report_dir, false),
            "window" | "auto" => canonical_impl(report_dir, true),
            other => {
                eprintln!("unknown mode: {other} (auto|window|headless)");
                2
            }
        },
        "live" => run_live(),
        other => {
            eprintln!("ag2d-breakout: unknown replay '{other}' (canonical|live)");
            2
        }
    }
}

/// Full-run verification results (pure sim + audio + GPU facts filled later).
struct CanonicalVerify {
    adapter: crate::renderer::AdapterInfoReport,
    device_token: String,
    window_created: bool,
    window_mode: String,
    score_final: u32,
    bricks_destroyed: u32,
    trace_match: bool,
    max_pos_diff: f32,
    pos_mismatch_ticks: u64,
    score_mismatch_ticks: u64,
    contact_mismatch_ticks: u64,
    contact_sequence_match: bool,
    contacts_total: u64,
    sounds_triggered: u64,
    kira_plays_ok: u64,
    kira_play_errors: u64,
    sound_count_equals_contacts: bool,
    hud_text_instances: usize,
    keyframes: Vec<Keyframe>,
    /// F07 content-oracle results per frozen golden frame.
    golden_frame_checks: Vec<(String, f64, f64, bool)>,
    notes: Vec<String>,
}

impl CanonicalVerify {
    fn keyframes_are_distinct(&self) -> bool {
        let mut hashes: Vec<&str> = self.keyframes.iter().map(|k| k.sha256.as_str()).collect();
        hashes.sort_unstable();
        hashes.dedup();
        hashes.len() == self.keyframes.len() && !hashes.is_empty()
    }
}

fn canonical_impl(report_dir: &Path, windowed: bool) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    let trace = match load_trace() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ag2d-breakout: {e}");
            return 3;
        }
    };
    let ogg_path = agp_file("assets/breakout/sounds/breakout_collision.ogg");
    let ogg_bytes = match std::fs::read(&ogg_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ag2d-breakout: read {}: {e}", ogg_path.display());
            return 3;
        }
    };
    let font_bytes = match std::fs::read(agp_file("assets/breakout/fonts/FiraMono-Medium.ttf")) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ag2d-breakout: read font: {e}");
            return 3;
        }
    };
    let mut audio = match Audio::new(&ogg_bytes) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("ag2d-breakout: audio init failed: {e}");
            return 3;
        }
    };

    // Pure verification: full frozen stream vs golden trace + kira playback.
    let mut v = verify_canonical(&trace, &mut audio);

    // GPU keyframes (windowed preferred; offscreen fallback).
    let mut window_ok = windowed;
    if windowed {
        match render_window(&trace, &font_bytes, &mut v, report_dir) {
            Ok(()) => {}
            Err(WindowOutcome::Fallback) => {
                v.notes
                    .push("window unavailable; keyframes rendered offscreen".into());
                window_ok = false;
            }
            Err(WindowOutcome::Failed(code)) => return code,
        }
    }
    if !window_ok {
        if let Err(code) = render_headless_keyframes(&trace, &font_bytes, &mut v, report_dir) {
            return code;
        }
    }

    // Cache composition on a real device: one atlas creation per session,
    // hits within a session (no rebuild of untouched resources).
    let cache_stats;
    let mut created_per_session = Vec::new();
    match GpuDevice::new() {
        Ok(gpu) => {
            let mut cache = ResourceCache::new(8);
            for _round in 0..4 {
                let owner = cache.allocator().allocate();
                let key = CacheKey {
                    owner: owner.token(),
                    role: "hud-atlas".into(),
                };
                let _ = cache.texture(&gpu, key.clone(), owner, 1024, 1024);
                let _ = cache.texture(&gpu, key, owner, 1024, 1024);
                created_per_session.push(cache.stats().created);
            }
            v.notes.push(format!(
                "cache created counter per fresh session: {created_per_session:?} (first lookup creates, second hits)"
            ));
            cache_stats = cache.stats().clone();
        }
        Err(e) => {
            v.notes
                .push(format!("cache check skipped (no device): {e}"));
            cache_stats = Default::default();
        }
    }

    // Reset rounds (host extension): fresh runtime+driver each round, replay
    // the 600-tick prefix, demand identical snapshot/objects/trace and a new
    // session with no stale events.
    let baseline = run_prefix(&trace, RESET_PREFIX_TICKS);
    let mut resets = Vec::new();
    for i in 0..3u32 {
        let r = run_prefix(&trace, RESET_PREFIX_TICKS);
        resets.push(BreakoutResetEvidence {
            index: i,
            ticks: RESET_PREFIX_TICKS as u64,
            snapshot_hash: r.snapshot_hash.clone(),
            objects: r.objects,
            trace_matches_prefix: r.snapshot_hash == baseline.snapshot_hash
                && r.objects == baseline.objects
                && r.mismatches == 0,
            events_all_new_session: r.events_all_new_session,
            cache_created: *created_per_session.first().unwrap_or(&1),
        });
    }
    v.notes.push(format!(
        "reset baseline snapshot {} ({} objects, {} trace mismatches)",
        baseline.snapshot_hash, baseline.objects, baseline.mismatches
    ));

    let pass = v.trace_match
        && v.contact_sequence_match
        && v.sound_count_equals_contacts
        && v.kira_plays_ok == v.contacts_total
        && v.kira_play_errors == 0
        && v.keyframes_are_distinct()
        && !v.keyframes.is_empty()
        && !v.golden_frame_checks.is_empty()
        && v.golden_frame_checks.iter().all(|c| c.3)
        && resets
            .iter()
            .all(|r| r.trace_matches_prefix && r.events_all_new_session);

    let report = BreakoutReport {
        adapter: v.adapter.clone(),
        device_token: v.device_token.clone(),
        window_created: v.window_created,
        window_mode: v.window_mode.clone(),
        ticks: trace.ticks.len() as u64,
        score_final: v.score_final,
        bricks_destroyed: v.bricks_destroyed,
        trace_match: v.trace_match,
        max_pos_diff: v.max_pos_diff,
        pos_mismatch_ticks: v.pos_mismatch_ticks,
        score_mismatch_ticks: v.score_mismatch_ticks,
        contact_mismatch_ticks: v.contact_mismatch_ticks,
        contact_sequence_match: v.contact_sequence_match,
        contacts_total: v.contacts_total,
        sounds_triggered: v.sounds_triggered,
        kira_plays_ok: v.kira_plays_ok,
        kira_play_errors: v.kira_play_errors,
        sound_count_equals_contacts: v.sound_count_equals_contacts,
        audio_backend: "kira 0.9 / cpal default output (real playback path)".into(),
        audio_asset: "assets/breakout/sounds/breakout_collision.ogg".into(),
        audio_asset_sha256: sha256_hex(&ogg_bytes),
        hud_text_instances: v.hud_text_instances,
        hud_font: "FiraMono-Medium.ttf (Bevy default-font family; evidence note E11)".into(),
        hud_font_size: SCOREBOARD_FONT_SIZE,
        keyframes: v.keyframes.clone(),
        resets,
        cache_stats,
        live_mode_available: true,
        gpu_timing_available: false,
        notes: std::mem::take(&mut v.notes),
    };
    let exit = if pass { "pass" } else { "fail" };
    match write_report(report_dir, "ag2d-breakout", "canonical", exit, report) {
        Ok(()) => {
            if pass {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("ag2d-breakout: report write failed: {e}");
            3
        }
    }
}

struct PrefixResult {
    snapshot_hash: String,
    objects: usize,
    mismatches: u64,
    events_all_new_session: bool,
}

/// Fresh runtime+driver, replay the first `n` frozen ticks.
fn run_prefix(trace: &GoldenTrace, n: usize) -> PrefixResult {
    let mut rt = Runtime::new(Duration::from_nanos(16_666_667), SEED);
    let mut driver = BreakoutDriver::default();
    let mut mismatches = 0u64;
    let session = rt.session();
    for (t, a) in trace.actions.iter().take(n).enumerate() {
        let events = rt.step(&mut driver, vec![action_of(*a)]);
        let want = &trace.ticks[t];
        if (driver.ball_pos[0] - want.bx).abs() > 1e-3
            || (driver.ball_pos[1] - want.by).abs() > 1e-3
            || (driver.paddle_x - want.px).abs() > 1e-3
            || driver.score != want.score
        {
            mismatches += 1;
        }
        if events.iter().any(|e| e.session != session) {
            mismatches += 1;
        }
    }
    PrefixResult {
        snapshot_hash: sha256_hex(rt.scene().snapshot().as_bytes()),
        objects: rt.scene().live_count(),
        mismatches,
        events_all_new_session: mismatches == 0,
    }
}

fn verify_canonical(trace: &GoldenTrace, audio: &mut Audio) -> CanonicalVerify {
    let mut rt = Runtime::new(Duration::from_nanos(16_666_667), SEED);
    let mut driver = BreakoutDriver::default();
    let mut max_pos_diff = 0.0f32;
    let mut pos_mm = 0u64;
    let mut score_mm = 0u64;
    let mut contact_mm = 0u64;
    let mut contacts: Vec<(u64, String, String, Option<u8>)> = Vec::new();
    let mut kira_ok = 0u64;
    let mut kira_err = 0u64;
    let mut cumulative_sounds = 0u64;
    let mut sound_mm = 0u64;
    for (t, a) in trace.actions.iter().enumerate() {
        let events: Vec<TickEvent<BreakoutEvent>> = rt.step(&mut driver, vec![action_of(*a)]);
        let want = &trace.ticks[t];
        debug_assert_eq!(want.t, t as u64, "golden trace tick ids must align");
        let dx = (driver.ball_pos[0] - want.bx).abs();
        let dy = (driver.ball_pos[1] - want.by).abs();
        let dp = (driver.paddle_x - want.px).abs();
        max_pos_diff = max_pos_diff.max(dx).max(dy).max(dp);
        if dx > 1e-3 || dy > 1e-3 || dp > 1e-3 {
            pos_mm += 1;
        }
        if driver.score != want.score {
            score_mm += 1;
        }
        let mut tick_contacts = 0u32;
        for e in &events {
            match e.payload {
                BreakoutEvent::Contact { kind, side } => {
                    tick_contacts += 1;
                    contacts.push((
                        t as u64,
                        kind_str(kind).to_string(),
                        side_str(side).to_string(),
                        match kind {
                            ColliderKind::Brick(i) => Some(i),
                            _ => None,
                        },
                    ));
                }
                BreakoutEvent::Sound { .. } => {
                    cumulative_sounds += 1;
                    if audio.play() {
                        kira_ok += 1;
                    } else {
                        kira_err += 1;
                    }
                }
                BreakoutEvent::BrickGone { .. } => {}
            }
        }
        if tick_contacts != want.contacts {
            contact_mm += 1;
        }
        if cumulative_sounds != want.sounds {
            sound_mm += 1;
        }
    }
    let contact_sequence_match = contacts.len() == trace.contacts_detail.len()
        && contacts
            .iter()
            .zip(trace.contacts_detail.iter())
            .all(|((t, k, s, b), w)| *t == w.t && *k == w.kind && *s == w.side && *b == w.brick);
    let sounds = driver.sounds_total;
    let contacts_total = driver.contacts_total;
    CanonicalVerify {
        adapter: crate::renderer::AdapterInfoReport {
            name: String::new(),
            backend: String::new(),
            device_type: String::new(),
            driver: String::new(),
            driver_info: String::new(),
            vendor: 0,
            device: 0,
        },
        device_token: String::new(),
        window_created: false,
        window_mode: String::new(),
        score_final: driver.score,
        bricks_destroyed: driver.bricks_alive.iter().filter(|b| !**b).count() as u32,
        trace_match: pos_mm == 0 && score_mm == 0 && contact_mm == 0 && sound_mm == 0,
        max_pos_diff,
        pos_mismatch_ticks: pos_mm,
        score_mismatch_ticks: score_mm,
        contact_mismatch_ticks: contact_mm,
        contact_sequence_match,
        contacts_total,
        sounds_triggered: sounds,
        kira_plays_ok: kira_ok,
        kira_play_errors: kira_err,
        sound_count_equals_contacts: sounds == contacts_total,
        hud_text_instances: 0,
        keyframes: Vec::new(),
        golden_frame_checks: Vec::new(),
        notes: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Keyframes
// ---------------------------------------------------------------------------

struct KeyframePlan {
    tick: u64,
    name: String,
}

fn keyframe_ticks(trace: &GoldenTrace) -> Vec<KeyframePlan> {
    let first = |pred: &dyn Fn(&TraceContact) -> bool| -> Option<u64> {
        trace.contacts_detail.iter().find(|c| pred(c)).map(|c| c.t)
    };
    let mut plans = Vec::new();
    if let Some(t) = first(&|c| c.kind == "paddle") {
        plans.push(KeyframePlan {
            tick: t,
            name: "first_paddle_contact".into(),
        });
    }
    if let Some(t) = first(&|c| c.kind == "brick") {
        plans.push(KeyframePlan {
            tick: t,
            name: "first_brick_contact".into(),
        });
    }
    if let Some(t) = first(&|c| c.kind == "wall_bottom") {
        plans.push(KeyframePlan {
            tick: t,
            name: "first_bottom_wall_bounce".into(),
        });
    }
    plans.push(KeyframePlan {
        tick: trace.ticks.len() as u64 - 1,
        name: "final_frame".into(),
    });
    plans
}

/// Run the sim to each keyframe tick, render the frame offscreen (arena
/// geometry + HUD text), capture PNGs.
fn capture_keyframes_offscreen(
    trace: &GoldenTrace,
    font_bytes: &[u8],
    gpu: &GpuDevice,
    report_dir: &Path,
    v: &mut CanonicalVerify,
) -> Result<(), String> {
    let mut rt = Runtime::new(Duration::from_nanos(16_666_667), SEED);
    let mut driver = BreakoutDriver::default();
    let target = OffscreenTarget::new(gpu, 900, 600).map_err(|e| format!("offscreen: {e}"))?;
    let pipe = QuadPipeline::new(gpu, OffscreenTarget::FORMAT).map_err(|e| format!("pipe: {e}"))?;
    pipe.set_ortho_camera(gpu, 0.0, 900.0, 600.0, 0.0);
    let text = TextRenderer::new(
        gpu,
        OffscreenTarget::FORMAT,
        font_bytes,
        SCOREBOARD_FONT_SIZE,
        HUD_CHARS,
    )
    .map_err(|e| format!("text: {e}"))?;
    text.set_ortho_camera(gpu, 0.0, 900.0, 600.0, 0.0);

    std::fs::create_dir_all(report_dir).map_err(|e| format!("mkdir: {e}"))?;
    let mut plans = keyframe_ticks(trace);
    plans.sort_by_key(|p| p.tick);
    for (t, a) in trace.actions.iter().enumerate() {
        let _ = rt.step(&mut driver, vec![action_of(*a)]);
        if plans.first().map(|p| p.tick == t as u64).unwrap_or(false) {
            let plan = plans.remove(0);
            let cmds = draw_cmds(&driver);
            pipe.render(
                gpu,
                target.view(),
                OffscreenTarget::FORMAT,
                BACKGROUND_COLOR,
                &cmds,
            );
            let quads = hud_quads(&text, driver.score);
            if plan.name == "final_frame" {
                v.hud_text_instances = quads.len();
            }
            text.render(gpu, target.view(), &quads);
            gpu.wait_idle();
            let (png, _) = capture_png(gpu, &target).map_err(|e| format!("capture: {e}"))?;
            // F07 content oracle: per-pixel comparison against the frozen
            // golden frame — same wiring as the windowed capture path
            // (review r2 F829-R2-01: headless/offscreen frames are
            // bit-exact vs the goldens, so the gate must run here too).
            if !super::frame_oracle::has_golden("ag2d-breakout", &plan.name) {
                // no frozen golden declared for this frame
            } else if let Some((mean, p99, ok)) =
                super::frame_oracle::compare_golden_frame("ag2d-breakout", &plan.name, &png)
            {
                v.golden_frame_checks
                    .push((plan.name.clone(), mean, p99, ok));
                if !ok {
                    eprintln!(
                        "ag2d-breakout: golden frame mismatch {} (mean {mean}, p99 {p99})",
                        plan.name
                    );
                }
            } else {
                v.golden_frame_checks
                    .push((plan.name.clone(), -1.0, -1.0, false));
            }
            let file = format!("{}.png", plan.name);
            let path = report_dir.join(&file);
            std::fs::write(&path, &png).map_err(|e| format!("keyframe write: {e}"))?;
            v.keyframes.push(Keyframe {
                name: plan.name,
                file,
                width: 900,
                height: 600,
                sha256: sha256_hex(&png),
            });
        }
        if plans.is_empty() {
            break;
        }
    }
    Ok(())
}

fn render_headless_keyframes(
    trace: &GoldenTrace,
    font_bytes: &[u8],
    v: &mut CanonicalVerify,
    report_dir: &Path,
) -> Result<(), i32> {
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ag2d-breakout(headless): device failed: {e}");
            return Err(3);
        }
    };
    v.adapter = gpu.adapter_info().clone();
    v.device_token = gpu.token().to_report();
    v.window_created = false;
    v.window_mode = "headless".into();
    capture_keyframes_offscreen(trace, font_bytes, &gpu, report_dir, v).map_err(|e| {
        eprintln!("ag2d-breakout(headless): {e}");
        3
    })
}

// ---------------------------------------------------------------------------
// Windowed canonical run
// ---------------------------------------------------------------------------

enum WindowOutcome {
    /// Window could not be created; offscreen fallback used instead.
    Fallback,
    Failed(i32),
}

struct GpuKit {
    gpu: GpuDevice,
    window: Arc<winit::window::Window>,
    target: WindowTarget,
    pipe: QuadPipeline,
    text: TextRenderer,
}

struct BreakoutApp<'a> {
    trace: &'a GoldenTrace,
    font_bytes: &'a [u8],
    verify: &'a mut CanonicalVerify,
    report_dir: PathBuf,
    kit: Option<GpuKit>,
    offscreen: Option<OffscreenTarget>,
    off_pipe: Option<QuadPipeline>,
    off_text: Option<TextRenderer>,
    rt: Option<Runtime<BreakoutAction, BreakoutEvent>>,
    driver: Option<BreakoutDriver>,
    plans: Vec<KeyframePlan>,
    tick: usize,
    outcome: Option<WindowOutcome>,
}

fn render_window(
    trace: &GoldenTrace,
    font_bytes: &[u8],
    v: &mut CanonicalVerify,
    report_dir: &Path,
) -> Result<(), WindowOutcome> {
    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::new().map_err(|e| {
        WindowOutcome::Failed({
            eprintln!("ag2d-breakout(window): event loop: {e}");
            3
        })
    })?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = BreakoutApp {
        trace,
        font_bytes,
        verify: v,
        report_dir: report_dir.to_path_buf(),
        kit: None,
        offscreen: None,
        off_pipe: None,
        off_text: None,
        rt: None,
        driver: None,
        plans: Vec::new(),
        tick: 0,
        outcome: None,
    };
    match event_loop.run_app(&mut app) {
        Ok(()) => match app.outcome {
            None => Ok(()),
            Some(WindowOutcome::Fallback) => Err(WindowOutcome::Fallback),
            Some(WindowOutcome::Failed(c)) => Err(WindowOutcome::Failed(c)),
        },
        Err(e) => {
            eprintln!("ag2d-breakout(window): {e}");
            Err(WindowOutcome::Failed(3))
        }
    }
}

impl BreakoutApp<'_> {
    /// Render the CURRENT sim state offscreen at the exact keyframe tick and
    /// store the capture.
    fn capture_keyframe(
        &mut self,
        driver: &BreakoutDriver,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<(), i32> {
        // Lazily create the offscreen kit.
        if self.off_pipe.is_none() {
            let gpu = &self.kit.as_ref().unwrap().gpu;
            match (
                OffscreenTarget::new(gpu, 900, 600),
                QuadPipeline::new(gpu, OffscreenTarget::FORMAT),
                TextRenderer::new(
                    gpu,
                    OffscreenTarget::FORMAT,
                    self.font_bytes,
                    SCOREBOARD_FONT_SIZE,
                    HUD_CHARS,
                ),
            ) {
                (Ok(target), Ok(pipe), Ok(text)) => {
                    pipe.set_ortho_camera(gpu, 0.0, 900.0, 600.0, 0.0);
                    text.set_ortho_camera(gpu, 0.0, 900.0, 600.0, 0.0);
                    self.offscreen = Some(target);
                    self.off_pipe = Some(pipe);
                    self.off_text = Some(text);
                }
                (a, b, c) => {
                    eprintln!(
                        "ag2d-breakout(window): offscreen init failed: target_err={} pipe_err={} text_err={}",
                        a.err().map(|e| e.classify()).unwrap_or("none"),
                        b.err().map(|e| e.classify()).unwrap_or("none"),
                        c.err().map(|e| e.classify()).unwrap_or("none"),
                    );
                    event_loop.exit();
                    return Err(3);
                }
            }
        }
        let plan = self.plans.remove(0);
        let cmds = draw_cmds(driver);
        let (gpu, target, pipe, text) = {
            let g = self.kit.as_ref().unwrap();
            (
                &g.gpu,
                self.offscreen.as_ref().unwrap(),
                self.off_pipe.as_ref().unwrap(),
                self.off_text.as_ref().unwrap(),
            )
        };
        pipe.render(
            gpu,
            target.view(),
            OffscreenTarget::FORMAT,
            BACKGROUND_COLOR,
            &cmds,
        );
        let quads = hud_quads(text, driver.score);
        text.render(gpu, target.view(), &quads);
        gpu.wait_idle();
        match capture_png(gpu, target) {
            Ok((png, _)) => {
                // F07 content oracle: per-pixel comparison against the
                // frozen golden frame (not just hash uniqueness).
                if !super::frame_oracle::has_golden("ag2d-breakout", &plan.name) {
                    // no frozen golden declared for this frame
                } else if let Some((mean, p99, ok)) =
                    super::frame_oracle::compare_golden_frame("ag2d-breakout", &plan.name, &png)
                {
                    self.verify
                        .golden_frame_checks
                        .push((plan.name.clone(), mean, p99, ok));
                    if !ok {
                        eprintln!(
                            "ag2d-breakout: golden frame mismatch {} (mean {mean}, p99 {p99})",
                            plan.name
                        );
                    }
                } else {
                    self.verify
                        .golden_frame_checks
                        .push((plan.name.clone(), -1.0, -1.0, false));
                }
                let file = format!("{}.png", plan.name);
                let path = self.report_dir.join(&file);
                if let Err(e) = std::fs::write(&path, &png) {
                    eprintln!("ag2d-breakout: keyframe write: {e}");
                    event_loop.exit();
                    return Err(3);
                }
                self.verify.keyframes.push(Keyframe {
                    name: plan.name,
                    file,
                    width: 900,
                    height: 600,
                    sha256: sha256_hex(&png),
                });
                Ok(())
            }
            Err(e) => {
                eprintln!("ag2d-breakout: capture: {e}");
                event_loop.exit();
                Err(3)
            }
        }
    }

    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) -> Result<(), i32> {
        let gpu = GpuDevice::new().map_err(|e| {
            eprintln!("ag2d-breakout(window): device: {e}");
            3
        })?;
        let attrs = winit::window::Window::default_attributes()
            .with_title("AGP ag2d-breakout (canonical replay)")
            .with_inner_size(winit::dpi::LogicalSize::new(900.0, 600.0));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("ag2d-breakout(window): create_window: {e}; using offscreen fallback");
                self.outcome = Some(WindowOutcome::Fallback);
                return Ok(());
            }
        };
        let size = window.inner_size();
        let target =
            WindowTarget::new(&gpu, window.clone(), size.width, size.height).map_err(|e| {
                eprintln!("ag2d-breakout(window): surface: {e}");
                3
            })?;
        let fmt = target.format();
        let pipe = QuadPipeline::new(&gpu, fmt).map_err(|e| {
            eprintln!("ag2d-breakout(window): pipeline: {e}");
            3
        })?;
        pipe.set_ortho_camera(&gpu, 0.0, 900.0, 600.0, 0.0);
        let text = TextRenderer::new(&gpu, fmt, self.font_bytes, SCOREBOARD_FONT_SIZE, HUD_CHARS)
            .map_err(|e| {
            eprintln!("ag2d-breakout(window): text: {e}");
            3
        })?;
        text.set_ortho_camera(&gpu, 0.0, 900.0, 600.0, 0.0);

        self.verify.adapter = gpu.adapter_info().clone();
        self.verify.device_token = gpu.token().to_report();
        self.verify.window_created = true;
        self.verify.window_mode = format!(
            "window {}x{} scale {}",
            size.width,
            size.height,
            window.scale_factor()
        );

        self.kit = Some(GpuKit {
            gpu,
            window,
            target,
            pipe,
            text,
        });
        self.rt = Some(Runtime::new(Duration::from_nanos(16_666_667), SEED));
        self.driver = Some(BreakoutDriver::default());
        self.plans = keyframe_ticks(self.trace);
        self.plans.sort_by_key(|p| p.tick);
        Ok(())
    }

    fn step_frame(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let total = self.trace.actions.len();
        let tick = self.tick;
        let end = (tick + TICKS_PER_FRAME).min(total);
        // Step tick-by-tick: keyframes must capture the EXACT contact tick,
        // not the end-of-frame state.
        let mut rt = self.rt.take();
        let mut driver = self.driver.take();
        let mut failed: Option<i32> = None;
        if let (Some(rt), Some(driver)) = (&mut rt, &mut driver) {
            for t in tick..end {
                let _ = rt.step(driver, vec![action_of(self.trace.actions[t])]);
                if self
                    .plans
                    .first()
                    .map(|p| p.tick == t as u64)
                    .unwrap_or(false)
                {
                    if let Err(code) = self.capture_keyframe(driver, event_loop) {
                        self.outcome = Some(WindowOutcome::Failed(code));
                        failed = Some(code);
                        break;
                    }
                }
            }
        }
        self.rt = rt;
        self.driver = driver;
        if failed.is_some() {
            return;
        }
        self.tick = end;
        let driver = self.driver.as_ref().unwrap();
        let quads = self
            .kit
            .as_ref()
            .map(|k| hud_quads(&k.text, driver.score))
            .unwrap_or_default();
        self.verify.hud_text_instances = quads.len();
        let cmds = draw_cmds(driver);

        if let Some(kit) = &mut self.kit {
            let outcome = {
                let fmt = kit.target.format();
                kit.target.present_frame(&kit.gpu, |view| {
                    kit.pipe
                        .render(&kit.gpu, view, fmt, BACKGROUND_COLOR, &cmds);
                    kit.text.render(&kit.gpu, view, &quads);
                })
            };
            if let Err(e) = outcome {
                eprintln!("ag2d-breakout(window): present: {e}");
                self.outcome = Some(WindowOutcome::Failed(3));
                event_loop.exit();
                return;
            }
        }

        if self.tick >= total {
            event_loop.exit();
        } else if let Some(kit) = &self.kit {
            kit.window.request_redraw();
        }
    }
}

impl winit::application::ApplicationHandler for BreakoutApp<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.kit.is_some() || self.outcome.is_some() {
            return;
        }
        match self.init(event_loop) {
            Ok(()) => {
                if self.outcome.is_none() {
                    if let Some(kit) = &self.kit {
                        kit.window.request_redraw();
                    }
                } else {
                    event_loop.exit();
                }
            }
            Err(code) => {
                self.outcome = Some(WindowOutcome::Failed(code));
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => self.step_frame(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(kit) = &self.kit {
            if self.tick < self.trace.actions.len() {
                kit.window.request_redraw();
                return;
            }
        }
        if self.kit.is_none() && self.outcome.is_none() {
            event_loop.exit();
        }
    }
}

// ---------------------------------------------------------------------------
// Live interactive mode
// ---------------------------------------------------------------------------

struct LiveState {
    rt: Runtime<BreakoutAction, BreakoutEvent>,
    driver: BreakoutDriver,
    kit: GpuKit,
    audio: Audio,
}

struct LiveApp {
    state: Option<LiveState>,
    left: bool,
    right: bool,
    /// F02: fixed 1/60 ticks are driven by elapsed wall time, not redraw
    /// count. Catch-up policy: at most 4 ticks (~66 ms debt) per redraw;
    /// sustained overload drops the debt instead of spiraling.
    clock: crate::core::TickClock,
    last_frame: std::time::Instant,
}

const LIVE_MAX_CATCHUP_TICKS: u32 = 4;

fn run_live() -> i32 {
    use winit::event_loop::EventLoop;
    let event_loop = match EventLoop::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ag2d-breakout(live): event loop: {e}");
            return 3;
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = LiveApp {
        state: None,
        left: false,
        right: false,
        clock: crate::core::TickClock::new(Duration::from_secs_f32(DT)),
        last_frame: std::time::Instant::now(),
    };
    match event_loop.run_app(&mut app) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ag2d-breakout(live): {e}");
            3
        }
    }
}

impl winit::application::ApplicationHandler for LiveApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let ogg = std::fs::read(agp_file("assets/breakout/sounds/breakout_collision.ogg"))
            .expect("collision ogg");
        let font =
            std::fs::read(agp_file("assets/breakout/fonts/FiraMono-Medium.ttf")).expect("hud font");
        let audio = Audio::new(&ogg).expect("audio");
        let gpu = GpuDevice::new().expect("gpu");
        let attrs = winit::window::Window::default_attributes()
            .with_title("AGP Breakout — LIVE (arrows move, R = reset)")
            .with_inner_size(winit::dpi::LogicalSize::new(900.0, 600.0));
        let window = Arc::new(event_loop.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let target =
            WindowTarget::new(&gpu, window.clone(), size.width, size.height).expect("surface");
        let fmt = target.format();
        let pipe = QuadPipeline::new(&gpu, fmt).expect("pipeline");
        pipe.set_ortho_camera(&gpu, 0.0, 900.0, 600.0, 0.0);
        let text =
            TextRenderer::new(&gpu, fmt, &font, SCOREBOARD_FONT_SIZE, HUD_CHARS).expect("text");
        text.set_ortho_camera(&gpu, 0.0, 900.0, 600.0, 0.0);
        println!("LIVE MODE: ArrowLeft/ArrowRight = paddle, R = reset (host extension). Close window to exit.");
        self.state = Some(LiveState {
            rt: Runtime::new(Duration::from_nanos(16_666_667), SEED),
            driver: BreakoutDriver::default(),
            kit: GpuKit {
                gpu,
                window,
                target,
                pipe,
                text,
            },
            audio,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::KeyboardInput { event: key, .. } => {
                use winit::keyboard::{KeyCode, PhysicalKey};
                let pressed = key.state == winit::event::ElementState::Pressed;
                match key.physical_key {
                    PhysicalKey::Code(KeyCode::ArrowLeft) => self.left = pressed,
                    PhysicalKey::Code(KeyCode::ArrowRight) => self.right = pressed,
                    PhysicalKey::Code(KeyCode::KeyR) if pressed => {
                        if let Some(s) = &mut self.state {
                            s.rt.reset(SEED);
                            s.driver.reset();
                            println!("reset (host extension): new session {:?}", s.rt.session());
                        }
                    }
                    _ => {}
                }
            }
            winit::event::WindowEvent::Resized(size) => {
                // keep the swapchain in sync; zero-size suspends, restore
                // reconfigures (same policy as the canonical window host).
                if let Some(s) = &mut self.state {
                    let _ = s
                        .kit
                        .target
                        .handle_resize(&s.kit.gpu, size.width, size.height);
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let elapsed = now.saturating_duration_since(self.last_frame);
                self.last_frame = now;
                let ticks = self.clock.feed_capped(elapsed, LIVE_MAX_CATCHUP_TICKS);
                if let Some(s) = &mut self.state {
                    let dir = match (self.left, self.right) {
                        (true, false) => PaddleDir::Left,
                        (false, true) => PaddleDir::Right,
                        _ => PaddleDir::None,
                    };
                    for _ in 0..ticks {
                        let events = s.rt.step(&mut s.driver, vec![BreakoutAction::Paddle(dir)]);
                        for e in &events {
                            if matches!(e.payload, BreakoutEvent::Sound { .. }) && !s.audio.play() {
                                eprintln!("ag2d-breakout(live): audio play failed");
                            }
                        }
                    }
                    let cmds = draw_cmds(&s.driver);
                    let quads = hud_quads(&s.kit.text, s.driver.score);
                    let fmt = s.kit.target.format();
                    let outcome = s.kit.target.present_frame(&s.kit.gpu, |view| {
                        s.kit
                            .pipe
                            .render(&s.kit.gpu, view, fmt, BACKGROUND_COLOR, &cmds);
                        s.kit.text.render(&s.kit.gpu, view, &quads);
                    });
                    if let Err(e) = outcome {
                        eprintln!("ag2d-breakout(live): present: {e}");
                    }
                    s.kit.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(s) = &self.state {
            s.kit.window.request_redraw();
            std::thread::sleep(Duration::from_millis(4)); // redraw pacing only;
                                                          // sim rate is decoupled via the elapsed-time accumulator above.
        }
    }
}
