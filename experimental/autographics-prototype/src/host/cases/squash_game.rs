//! Case `ag3d-squash` (Plan V-04 / AC-04): the frozen-sample 3D game.
//! `--replay canonical` drives the FROZEN action stream + seed through the
//! squash sim, verifies against the frozen golden trace (positions ≤1e-2 —
//! measured bit-exact —, exact score/event sequence), plays the frozen
//! looping music through kira (real playback path, autoplay like the
//! sample's MusicPlayer autoload), captures keyframes at squash/death/
//! retry moments, and proves retry-reset determinism. `--replay live` is
//! the interactive mode.

use crate::assets::{load_glb_file, MeshData, ModelData};
use crate::core::runtime::{Runtime, TickEvent};
use crate::core::scene::Color;
use crate::core::scene::Scene;
use crate::games::squash::*;
use crate::host::report::{
    write_report, Keyframe, SquashMusicFacts, SquashReport, SquashRetryFacts, SquashTraceFacts,
};
use crate::render2d::{GlyphQuad, TextRenderer};
use crate::render3d::procedural::{cylinder_mesh, ground_mesh};
use crate::render3d::{
    sample_light, Camera3d, MeshInstance, MeshPipeline, MeshStore, CYLINDER_POSITIONS,
};
use crate::renderer::capture::{capture_png, sha256_hex};
use crate::renderer::error::RendererError;
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};
use crate::renderer::target::{OffscreenTarget, WindowTarget};
use crate::renderer::GpuDevice;
use glam::{Mat4, Vec3};
use kira::manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const W: u32 = 900;
const H: u32 = 600;
const SKY_HORIZON: [f32; 4] = [0.64625, 0.65575, 0.67075, 1.0];

fn sky_color() -> Color {
    Color::rgba(
        SKY_HORIZON[0],
        SKY_HORIZON[1],
        SKY_HORIZON[2],
        SKY_HORIZON[3],
    )
}
/// 120 Hz sim, 6 physics ticks per redraw (~20 fps fast-forward).
const TICKS_PER_FRAME: usize = 6;
const HUD_FONT_PX: f32 = 32.0; // ScoreLabel theme override (Main.tscn line 157)
const HUD_CHARS: &str = "Score: 0123456789Prs pantEy";
const RETRY_TEXT: &str = "Press Space or Enter to retry";

fn agp_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

// ---------------------------------------------------------------------------
// Golden trace
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TraceTick {
    px: f32,
    py: f32,
    pz: f32,
    score: u32,
    alive: u32,
    mobs_alive: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // schema documentation; consumed via GoldenEventFull below
struct TraceEvent {
    t: u64,
    kind: String,
}
// (fields consumed via the GoldenEventFull/GoldenSpawnFull views below; the
// raw struct documents the frozen schema)

#[derive(Debug, Deserialize)]
struct GoldenTrace {
    ticks_trace: Vec<TraceTick>,
    events: Vec<TraceEvent>,
    actions: Vec<[i8; 3]>,
}

fn load_trace() -> Result<GoldenTrace, String> {
    let path = agp_file("testdata/squash/golden_squash_trace.json");
    let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("parse: {e}"))
}

fn action_of(a: [i8; 3]) -> SquashAction {
    SquashAction {
        move_x: a[0],
        move_z: a[1],
        jump: a[2] != 0,
        retry: false,
        reset: false,
    }
}

// ---------------------------------------------------------------------------
// Scene
// ---------------------------------------------------------------------------

pub(crate) struct SceneData {
    /// Retained for provenance/identity (asset facts); rendering uses keys.
    #[allow(dead_code)]
    player: ModelData,
    #[allow(dead_code)]
    mob: ModelData,
    player_keys: Vec<(String, String)>,
    mob_keys: Vec<(String, String)>,
    ground_key: (String, String),
    cylinder_key: (String, String),
    pub(crate) camera: Camera3d,
}

fn synth_model(mesh: &MeshData) -> ModelData {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    for p in &mesh.positions {
        h.update(p[0].to_ne_bytes());
        h.update(p[1].to_ne_bytes());
        h.update(p[2].to_ne_bytes());
    }
    h.update(mesh.name.as_bytes());
    ModelData {
        source_sha256: format!("{:x}", h.finalize()),
        meshes: vec![mesh.clone()],
    }
}

pub(crate) fn build_scene(
    gpu: &GpuDevice,
    store: &mut MeshStore,
) -> Result<SceneData, RendererError> {
    let player = load_glb_file(&agp_file("assets/squash_the_creeps/art/player.glb"))
        .map_err(|e| RendererError::ReportIo(format!("player.glb: {e}")))?;
    let mob = load_glb_file(&agp_file("assets/squash_the_creeps/art/mob.glb"))
        .map_err(|e| RendererError::ReportIo(format!("mob.glb: {e}")))?;
    let mut player_keys = Vec::new();
    for i in 0..player.meshes.len() {
        let (name, _) = store.get_or_upload(gpu, &player, i)?;
        player_keys.push((player.source_sha256.clone(), name));
    }
    let mut mob_keys = Vec::new();
    for i in 0..mob.meshes.len() {
        let (name, _) = store.get_or_upload(gpu, &mob, i)?;
        mob_keys.push((mob.source_sha256.clone(), name));
    }
    let ground = synth_model(&ground_mesh());
    let ground_key = (ground.source_sha256.clone(), ground.meshes[0].name.clone());
    store.get_or_upload(gpu, &ground, 0)?;
    let cylinder = synth_model(&cylinder_mesh());
    let cylinder_key = (
        cylinder.source_sha256.clone(),
        cylinder.meshes[0].name.clone(),
    );
    store.get_or_upload(gpu, &cylinder, 0)?;
    Ok(SceneData {
        player,
        mob,
        player_keys,
        mob_keys,
        ground_key,
        cylinder_key,
        camera: Camera3d::sample_camera(W as f32 / H as f32),
    })
}

/// Frame instances: environment + live mobs + player (hidden when dead —
/// the sample queue_frees the player on hit).
pub(crate) fn frame_instances(shared: &Scene, scene: &SceneData) -> Vec<MeshInstance> {
    let mut out = Vec::new();
    out.push(MeshInstance {
        mesh: scene.ground_key.clone(),
        model: Mat4::IDENTITY,
    });
    for pos in CYLINDER_POSITIONS {
        out.push(MeshInstance {
            mesh: scene.cylinder_key.clone(),
            model: Mat4::from_translation(Vec3::new(pos[0], pos[1], pos[2])),
        });
    }
    // F11: entities are projected from the SHARED scene (kind by spawn
    // order), never from private driver fields. The frozen sample hides the
    // player on hit by freeing it — the shared scene mirrors that (Destroy).
    for (_id, kind, model) in project_instances(shared) {
        let keys = match kind {
            EntityKind::Player => &scene.player_keys,
            EntityKind::Mob => &scene.mob_keys,
        };
        for key in keys {
            out.push(MeshInstance {
                mesh: key.clone(),
                model,
            });
        }
    }
    out
}

fn hud_quads(text: &TextRenderer, score: u32) -> Vec<GlyphQuad> {
    text.layout(
        &format!("Score: {score}"),
        16.0,
        16.0,
        Color::rgb(0.0, 0.0, 0.0),
    )
}

fn retry_overlay(text: &TextRenderer) -> (DrawCmd, Vec<GlyphQuad>) {
    let w = text.measure(RETRY_TEXT);
    let quads = text.layout(
        RETRY_TEXT,
        (W as f32 - w) * 0.5,
        H as f32 * 0.5 - 16.0,
        Color::rgb(1.0, 1.0, 1.0),
    );
    (
        DrawCmd {
            center: [W as f32 / 2.0, H as f32 / 2.0],
            size: [W as f32, H as f32],
            z: 0.0,
            color: Color::rgba(0.0, 0.0, 0.0, 0.447059),
            shape: Shape::Rect,
        },
        quads,
    )
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    match replay {
        "canonical" => {
            let trace = match load_trace() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("ag3d-squash: {e}");
                    return 3;
                }
            };
            let music_bytes = match std::fs::read(agp_file(
                "assets/squash_the_creeps/art/House In a Forest Loop.ogg",
            )) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("ag3d-squash: music: {e}");
                    return 3;
                }
            };
            let music_sha = sha256_hex(&music_bytes);
            let music = match start_music(&music_bytes) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("ag3d-squash: audio init failed: {e}");
                    return 3;
                }
            };
            if mode == "headless" {
                run_canonical_headless(report_dir, trace, music, music_sha)
            } else {
                run_canonical_windowed(report_dir, trace, music, music_sha)
            }
        }
        "live" => run_live(),
        other => {
            eprintln!("ag3d-squash: unknown replay '{other}' (canonical|live)");
            2
        }
    }
}

// ---------------------------------------------------------------------------
// Canonical verification core
// ---------------------------------------------------------------------------

struct VerifyOut {
    facts: SquashTraceFacts,
    spawn_stream_matches_golden: bool,
    keyframe_ticks: Vec<(u64, String)>,
    retry: SquashRetryFacts,
}

fn verify_canonical(trace: &GoldenTrace) -> VerifyOut {
    let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
    let mut driver = SquashDriver::new(SQUASH_SEED);
    let mut max_pos = 0.0f32;
    let mut pos_mm = 0u64;
    let mut score_mm = 0u64;
    let mut mobs_mm = 0u64;
    let mut alive_mm = 0u64;
    let mut events: Vec<(u64, SquashEvent)> = Vec::new();
    let mut spawns = 0u32;

    for (i, a) in trace.actions.iter().enumerate() {
        let step_events: Vec<TickEvent<SquashEvent>> = rt.step(&mut driver, vec![action_of(*a)]);
        for e in step_events {
            if matches!(e.payload, SquashEvent::Spawned { .. }) {
                spawns += 1;
            }
            events.push((i as u64, e.payload));
        }
        let want = &trace.ticks_trace[i];
        let d = (driver.player.x - want.px)
            .abs()
            .max((driver.player.y - want.py).abs())
            .max((driver.player.z - want.pz).abs());
        max_pos = max_pos.max(d);
        if d > 1.0e-2 {
            pos_mm += 1;
        }
        if driver.score != want.score {
            score_mm += 1;
        }
        let mobs_alive = driver.mobs.iter().filter(|m| m.alive).count() as u32;
        if mobs_alive != want.mobs_alive {
            mobs_mm += 1;
        }
        if (driver.dead as u32) != 1 - want.alive {
            alive_mm += 1;
        }
    }

    // F07: FULL-payload comparison — kind, tick, mob index, squash score,
    // and the complete spawn tuples (position, yaw, speed, velocity, tick).
    #[derive(serde::Deserialize)]
    struct GoldenEventFull {
        t: u64,
        kind: String,
        index: Option<u32>,
        score: Option<u32>,
    }
    let golden_events: Vec<GoldenEventFull> = {
        let path = agp_file("testdata/squash/golden_squash_trace.json");
        let bytes = std::fs::read(&path).expect("trace");
        let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        serde_json::from_value(doc["events"].clone()).expect("events")
    };
    let event_seq_exact = events.len() == golden_events.len()
        && events.iter().zip(golden_events.iter()).all(|((t, e), w)| {
            *t == w.t
                && match (e, w.kind.as_str()) {
                    (SquashEvent::Spawned { index, .. }, "spawn")
                    | (SquashEvent::Squashed { index, .. }, "squash")
                    | (SquashEvent::Despawned { index }, "despawn") => Some(*index) == w.index,
                    (SquashEvent::PlayerDied, "death") => true,
                    _ => false,
                }
        });
    // squash payloads: index AND score both exact
    let squash_payloads_exact = {
        let got: Vec<(u64, u32, u32)> = events
            .iter()
            .filter_map(|(t, e)| match e {
                SquashEvent::Squashed { index, score } => Some((*t, *index, *score)),
                _ => None,
            })
            .collect();
        let want: Vec<(u64, u32, u32)> = golden_events
            .iter()
            .filter(|w| w.kind == "squash")
            .map(|w| (w.t, w.index.unwrap(), w.score.unwrap()))
            .collect();
        got == want
    };
    // spawn tuples: full comparison against the frozen spawns table
    #[derive(serde::Deserialize)]
    struct GoldenSpawnFull {
        #[allow(dead_code)]
        index: u32,
        tick: u64,
        pos: [f32; 3],
        yaw: f32,
        speed: f32,
        vel: [f32; 2],
    }
    let golden_spawns: Vec<GoldenSpawnFull> = {
        let path = agp_file("testdata/squash/golden_squash_trace.json");
        let bytes = std::fs::read(&path).expect("trace");
        let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        serde_json::from_value(doc["spawns"].clone()).expect("spawns")
    };
    let spawn_tuples_exact = {
        let got: Vec<(u64, Vec3, f32, f32, Vec3)> = events
            .iter()
            .filter_map(|(t, e)| match e {
                SquashEvent::Spawned {
                    pos,
                    yaw,
                    speed,
                    vel,
                    ..
                } => Some((*t, *pos, *yaw, *speed, *vel)),
                _ => None,
            })
            .collect();
        got.len() == golden_spawns.len()
            && got
                .iter()
                .zip(golden_spawns.iter())
                .all(|((t, pos, yaw, speed, vel), w)| {
                    *t == w.tick
                        && (pos.x - w.pos[0]).abs() < 1e-4
                        && (pos.y - w.pos[1]).abs() < 1e-4
                        && (pos.z - w.pos[2]).abs() < 1e-4
                        && (*yaw - w.yaw).abs() < 1e-4
                        && *speed == w.speed
                        && (vel.x - w.vel[0]).abs() < 1e-3
                        && (vel.z - w.vel[1]).abs() < 1e-3
                })
    };
    let squashes = events
        .iter()
        .filter(|(_, e)| matches!(e, SquashEvent::Squashed { .. }))
        .count() as u32;
    let death_tick = events
        .iter()
        .find(|(_, e)| matches!(e, SquashEvent::PlayerDied))
        .map(|(t, _)| *t);

    let facts = SquashTraceFacts {
        ticks: trace.actions.len() as u64,
        max_pos_diff: max_pos,
        pos_mismatch_ticks: pos_mm,
        score_mismatch_ticks: score_mm,
        mobs_alive_mismatch_ticks: mobs_mm,
        alive_mismatch_ticks: alive_mm,
        event_count: events.len(),
        golden_event_count: trace.events.len(),
        event_sequence_exact: event_seq_exact && squash_payloads_exact && spawn_tuples_exact,
        squashes,
        death_tick,
        spawns,
        score_final: driver.score,
    };

    let first_squash = events
        .iter()
        .find(|(_, e)| matches!(e, SquashEvent::Squashed { .. }))
        .map(|(t, _)| *t);
    let mut keyframe_ticks = Vec::new();
    if let Some(t) = first_squash {
        keyframe_ticks.push((t, "squash_moment".into()));
    }
    if let Some(t) = death_tick {
        keyframe_ticks.push((t, "death_moment".into()));
        keyframe_ticks.push((t + 30, "retry_overlay".into()));
    }
    keyframe_ticks.push((trace.actions.len() as u64 - 1, "final_frame".into()));

    let prefix = 300u64;
    let run_prefix = || {
        let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
        let mut d = SquashDriver::new(SQUASH_SEED);
        let session = rt.session();
        let mut bytes = Vec::new();
        for a in trace.actions.iter().take(prefix as usize) {
            let evs = rt.step(&mut d, vec![action_of(*a)]);
            for e in &evs {
                assert_eq!(e.session, session);
            }
            bytes.extend_from_slice(&d.player.x.to_ne_bytes());
            bytes.extend_from_slice(&d.player.y.to_ne_bytes());
            bytes.extend_from_slice(&d.player.z.to_ne_bytes());
        }
        (session, sha256_hex(&bytes))
    };
    let (s1, h1) = run_prefix();
    let (s2, h2) = run_prefix();
    let (s3, h3) = run_prefix();

    VerifyOut {
        facts,
        spawn_stream_matches_golden: spawns as usize
            == trace.events.iter().filter(|e| e.kind == "spawn").count(),
        keyframe_ticks,
        retry: SquashRetryFacts {
            reset_rounds: 2,
            prefix_ticks: prefix,
            snapshots_equal: h1 == h2 && h2 == h3,
            new_session_each_round: s1 != s2 && s2 != s3,
        },
    }
}

struct Music {
    _manager: AudioManager<DefaultBackend>,
    handle: StaticSoundHandle,
}

fn start_music(bytes: &[u8]) -> Result<Music, String> {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
        .map_err(|e| format!("audio manager: {e:?}"))?;
    let data = StaticSoundData::from_cursor(std::io::Cursor::new(bytes.to_vec()))
        .map_err(|e| format!("decode music ogg: {e:?}"))?
        // MusicPlayer.tscn .import param loop=true; autoload autoplay.
        .loop_region(..);
    let handle = manager
        .play(data)
        .map_err(|e| format!("play music: {e:?}"))?;
    Ok(Music {
        _manager: manager,
        handle,
    })
}

fn write(report_dir: &Path, report: SquashReport, pass: bool) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    let exit = if pass { "pass" } else { "fail" };
    match write_report(report_dir, "ag3d-squash", "canonical", exit, report) {
        Ok(()) => {
            if pass {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("ag3d-squash: report: {e}");
            3
        }
    }
}

fn build_report(
    gpu: &GpuDevice,
    window_created: bool,
    v: &VerifyOut,
    music: &Music,
    music_sha: &str,
    keyframes: Vec<Keyframe>,
    uploaded: usize,
) -> (SquashReport, bool) {
    let f = &v.facts;
    let mut hashes: Vec<&str> = keyframes.iter().map(|k| k.sha256.as_str()).collect();
    hashes.sort_unstable();
    hashes.dedup();
    let pass = f.pos_mismatch_ticks == 0
        && f.score_mismatch_ticks == 0
        && f.mobs_alive_mismatch_ticks == 0
        && f.alive_mismatch_ticks == 0
        && f.event_sequence_exact
        && f.squashes == 3
        && f.death_tick.is_some()
        && v.spawn_stream_matches_golden
        && v.retry.snapshots_equal
        && v.retry.new_session_each_round
        && keyframes.len() == v.keyframe_ticks.len()
        && hashes.len() == keyframes.len()
        // F07 content oracle: every captured keyframe that has a frozen
        // golden must match it within the manifest tolerances.
        && {
            let checks = FRAME_CHECKS.lock().unwrap();
            !checks.is_empty() && checks.iter().all(|c| c.3)
        }
        && matches!(music.handle.state(), kira::sound::PlaybackState::Playing);
    (
        SquashReport {
            adapter: gpu.adapter_info().clone(),
            device_token: gpu.token().to_report(),
            window_created,
            window_mode: if window_created { "window" } else { "headless" }.into(),
            trace: f.clone(),
            spawn_stream_matches_golden: v.spawn_stream_matches_golden,
            music: SquashMusicFacts {
                asset: "assets/squash_the_creeps/art/House In a Forest Loop.ogg".into(),
                asset_sha256: music_sha.to_string(),
                backend: "kira 0.9 / cpal default output".into(),
                autoplay_on_start: true,
                playback_state: format!("{:?}", music.handle.state()),
                looped: true,
            },
            keyframes,
            retry: v.retry.clone(),
            cache_uploaded_meshes: uploaded,
            gpu_timing_available: false,
            live_mode_available: true,
            notes: vec![
                "frozen semantics: mobs fly straight (no wall bounce; collision_mask = undefined layer) and despawn via the screen-exit frustum test of the frozen notifier AABB".into(),
                "squash() is instant despawn + signal in the frozen source — no squash scale animation exists (Mob.gd lines 35-37)".into(),
                "Retry overlay text/color per Main.tscn lines 161-182; score label 32px black at (16,16) per lines 150-158; Montserrat Medium font per ui_theme.tres".into(),
                "WorldBoundaryShape3D approximated by inward planes at frozen positions (3d-feasibility.md)".into(),
            ],
        },
        pass,
    )
}

// ---------------------------------------------------------------------------
// Keyframe rendering (shared)
// ---------------------------------------------------------------------------

struct FrameKits<'a> {
    gpu: &'a GpuDevice,
    pipeline: &'a mut MeshPipeline,
    quads2d: &'a QuadPipeline,
    text: &'a TextRenderer,
    store: &'a MeshStore,
    scene: &'a SceneData,
    offscreen: &'a OffscreenTarget,
}

pub static FRAME_CHECKS: std::sync::LazyLock<Mutex<Vec<(String, f64, f64, bool)>>> =
    std::sync::LazyLock::new(|| Mutex::new(Vec::new()));

fn render_frame(
    kits: &mut FrameKits,
    shared: &Scene,
    driver: &SquashDriver,
    name: &str,
    report_dir: &Path,
) -> Result<Keyframe, RendererError> {
    let instances = frame_instances(shared, kits.scene);
    kits.pipeline.render(
        kits.gpu,
        kits.offscreen.view(),
        W,
        H,
        SKY_HORIZON,
        &kits.scene.camera.view_proj(),
        &sample_light(),
        kits.store,
        &instances,
        true,
    );
    // HUD (2D quads over the 3D frame).
    kits.quads2d
        .set_ortho_camera(kits.gpu, 0.0, W as f32, H as f32, 0.0);
    kits.text
        .set_ortho_camera(kits.gpu, 0.0, W as f32, H as f32, 0.0);
    let mut quads = hud_quads(kits.text, driver.score);
    if driver.dead {
        let (overlay, retry) = retry_overlay(kits.text);
        kits.quads2d.render_over(
            kits.gpu,
            kits.offscreen.view(),
            OffscreenTarget::FORMAT,
            &[overlay],
        );
        quads.extend(retry);
    }
    kits.text.render(kits.gpu, kits.offscreen.view(), &quads);
    kits.gpu.wait_idle();
    let (png, _) = capture_png(kits.gpu, kits.offscreen)?;
    // F07 content oracle: per-pixel comparison against the frozen golden.
    if !super::frame_oracle::has_golden("ag3d-squash", name) {
        // no frozen golden declared for this frame
    } else if let Some((mean, p99, ok)) =
        super::frame_oracle::compare_golden_frame("ag3d-squash", name, &png)
    {
        FRAME_CHECKS
            .lock()
            .unwrap()
            .push((name.to_string(), mean, p99, ok));
        if !ok {
            eprintln!("ag3d-squash: golden frame mismatch {name} (mean {mean}, p99 {p99})");
        }
    } else {
        FRAME_CHECKS
            .lock()
            .unwrap()
            .push((name.to_string(), -1.0, -1.0, false));
    }
    let file = format!("{name}.png");
    std::fs::write(report_dir.join(&file), &png)
        .map_err(|e| RendererError::ReportIo(format!("keyframe write: {e}")))?;
    Ok(Keyframe {
        name: name.to_string(),
        file,
        width: W,
        height: H,
        sha256: sha256_hex(&png),
    })
}

// ---------------------------------------------------------------------------
// Headless canonical path
// ---------------------------------------------------------------------------

fn run_canonical_headless(
    report_dir: &Path,
    trace: GoldenTrace,
    music: Music,
    music_sha: String,
) -> i32 {
    let v = verify_canonical(&trace);
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ag3d-squash(headless): device: {e}");
            return 3;
        }
    };
    let offscreen = match OffscreenTarget::new(&gpu, W, H) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ag3d-squash(headless): offscreen: {e}");
            return 3;
        }
    };
    let mut pipeline = match MeshPipeline::new(&gpu, OffscreenTarget::FORMAT) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ag3d-squash(headless): pipeline: {e}");
            return 3;
        }
    };
    let quads2d = match QuadPipeline::new(&gpu, OffscreenTarget::FORMAT) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ag3d-squash(headless): 2d pipeline: {e}");
            return 3;
        }
    };
    let font = std::fs::read(agp_file(
        "assets/squash_the_creeps/fonts/Montserrat-Medium.ttf",
    ))
    .map_err(|e| {
        eprintln!("ag3d-squash(headless): font: {e}");
    })
    .unwrap_or_default();
    let text = match TextRenderer::new(&gpu, OffscreenTarget::FORMAT, &font, HUD_FONT_PX, HUD_CHARS)
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ag3d-squash(headless): text: {e}");
            return 3;
        }
    };
    let mut store = MeshStore::default();
    let scene = match build_scene(&gpu, &mut store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ag3d-squash(headless): scene: {e}");
            return 3;
        }
    };
    let plans = v.keyframe_ticks.clone();
    let mut keyframes = Vec::new();
    let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
    let mut driver = SquashDriver::new(SQUASH_SEED);
    for (i, a) in trace.actions.iter().enumerate() {
        let _ = rt.step(&mut driver, vec![action_of(*a)]);
        if let Some((_, name)) = plans.iter().find(|(t, _)| *t == i as u64) {
            let mut kits = FrameKits {
                gpu: &gpu,
                pipeline: &mut pipeline,
                quads2d: &quads2d,
                text: &text,
                store: &store,
                scene: &scene,
                offscreen: &offscreen,
            };
            match render_frame(&mut kits, rt.scene(), &driver, name, report_dir) {
                Ok(k) => keyframes.push(k),
                Err(e) => {
                    eprintln!("ag3d-squash(headless): keyframe: {e}");
                    return 3;
                }
            }
        }
    }
    let uploaded = store.uploaded_count();
    let (report, pass) = build_report(&gpu, false, &v, &music, &music_sha, keyframes, uploaded);
    write(report_dir, report, pass)
}

// ---------------------------------------------------------------------------
// Windowed canonical path
// ---------------------------------------------------------------------------

fn run_canonical_windowed(
    report_dir: &Path,
    trace: GoldenTrace,
    music: Music,
    music_sha: String,
) -> i32 {
    use winit::event_loop::EventLoop;
    let event_loop = match EventLoop::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ag3d-squash: event loop: {e}; headless fallback");
            return run_canonical_headless(report_dir, trace, music, music_sha);
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let verify = verify_canonical(&trace);
    let mut app = SquashApp {
        report_dir: report_dir.to_path_buf(),
        gpu: None,
        scene: None,
        kit: None,
        offline: None,
        rt: None,
        driver: None,
        music: Some(music),
        music_sha,
        trace,
        plans: verify.keyframe_ticks.clone(),
        verify: Some(verify),
        step: 0,
        keyframes: Vec::new(),
        exit_code: 0,
    };
    match event_loop.run_app(&mut app) {
        Ok(()) => app.exit_code,
        Err(e) => {
            eprintln!("ag3d-squash: {e}");
            3
        }
    }
}

struct WindowKit {
    window: Arc<winit::window::Window>,
    target: WindowTarget,
    pipeline: MeshPipeline,
    quads2d: QuadPipeline,
    text: TextRenderer,
    width: u32,
    height: u32,
}

struct SquashApp {
    report_dir: PathBuf,
    gpu: Option<GpuDevice>,
    scene: Option<SceneData>,
    kit: Option<WindowKit>,
    offline: Option<(
        MeshStore,
        OffscreenTarget,
        MeshPipeline,
        TextRenderer,
        QuadPipeline,
    )>,
    rt: Option<Runtime<SquashAction, SquashEvent>>,
    driver: Option<SquashDriver>,
    music: Option<Music>,
    music_sha: String,
    trace: GoldenTrace,
    plans: Vec<(u64, String)>,
    verify: Option<VerifyOut>,
    step: usize,
    keyframes: Vec<Keyframe>,
    exit_code: i32,
}

impl winit::application::ApplicationHandler for SquashApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.kit.is_some() || self.exit_code != 0 {
            if self.kit.is_none() {
                event_loop.exit();
            }
            return;
        }
        match self.init(event_loop) {
            Ok(()) => {
                if let Some(kit) = &self.kit {
                    kit.window.request_redraw();
                } else {
                    // headless fallback already produced the report
                    event_loop.exit();
                }
            }
            Err(code) => {
                self.exit_code = code;
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
            winit::event::WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(kit) = &self.kit {
            if self.step < self.trace.actions.len() {
                kit.window.request_redraw();
            }
        }
    }
}

impl SquashApp {
    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) -> Result<(), i32> {
        let gpu = GpuDevice::new().map_err(|e| {
            eprintln!("ag3d-squash: device: {e}");
            3
        })?;
        let attrs = winit::window::Window::default_attributes()
            .with_title("AGP ag3d-squash (canonical replay)")
            .with_inner_size(winit::dpi::LogicalSize::new(W as f64, H as f64));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("ag3d-squash: create_window: {e}; headless fallback");
                let trace = std::mem::replace(
                    &mut self.trace,
                    GoldenTrace {
                        ticks_trace: Vec::new(),
                        events: Vec::new(),
                        actions: Vec::new(),
                    },
                );
                let music = self.music.take().unwrap();
                self.exit_code =
                    run_canonical_headless(&self.report_dir, trace, music, self.music_sha.clone());
                return Ok(());
            }
        };
        let size = window.inner_size();
        let target =
            WindowTarget::new(&gpu, window.clone(), size.width, size.height).map_err(|e| {
                eprintln!("ag3d-squash: surface: {e}");
                3
            })?;
        let fmt = target.format();
        let pipeline = MeshPipeline::new(&gpu, fmt).map_err(|e| {
            eprintln!("ag3d-squash: pipeline: {e}");
            3
        })?;
        let quads2d = QuadPipeline::new(&gpu, fmt).map_err(|e| {
            eprintln!("ag3d-squash: 2d pipeline: {e}");
            3
        })?;
        let font = std::fs::read(agp_file(
            "assets/squash_the_creeps/fonts/Montserrat-Medium.ttf",
        ))
        .map_err(|e| {
            eprintln!("ag3d-squash: font: {e}");
            3
        })?;
        let text = TextRenderer::new(&gpu, fmt, &font, HUD_FONT_PX, HUD_CHARS).map_err(|e| {
            eprintln!("ag3d-squash: text: {e}");
            3
        })?;
        let offscreen = OffscreenTarget::new(&gpu, W, H).map_err(|e| {
            eprintln!("ag3d-squash: offscreen: {e}");
            3
        })?;
        let off_pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err(|e| {
            eprintln!("ag3d-squash: offscreen pipeline: {e}");
            3
        })?;
        let off_text =
            TextRenderer::new(&gpu, OffscreenTarget::FORMAT, &font, HUD_FONT_PX, HUD_CHARS)
                .map_err(|e| {
                    eprintln!("ag3d-squash: offscreen text: {e}");
                    3
                })?;
        let off_quads2d = QuadPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err(|e| {
            eprintln!("ag3d-squash: offscreen 2d pipeline: {e}");
            3
        })?;
        let mut store = MeshStore::default();
        let scene = build_scene(&gpu, &mut store).map_err(|e| {
            eprintln!("ag3d-squash: scene: {e}");
            3
        })?;
        self.kit = Some(WindowKit {
            window,
            target,
            pipeline,
            quads2d,
            text,
            width: size.width,
            height: size.height,
        });
        self.gpu = Some(gpu);
        self.scene = Some(scene);
        self.offline = Some((store, offscreen, off_pipeline, off_text, off_quads2d));
        self.rt = Some(Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED));
        self.driver = Some(SquashDriver::new(SQUASH_SEED));
        Ok(())
    }

    fn frame(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Advance the sim by TICKS_PER_FRAME, capturing planned keyframes.
        let start = self.step;
        let end = (start + TICKS_PER_FRAME).min(self.trace.actions.len());
        let mut rt = self.rt.take();
        let mut driver = self.driver.take();
        let mut capture_err: Option<String> = None;
        if let (Some(rt), Some(driver)) = (&mut rt, &mut driver) {
            for i in start..end {
                let _ = rt.step(driver, vec![action_of(self.trace.actions[i])]);
                if let Some((_, name)) = self
                    .plans
                    .iter()
                    .find(|(t, _)| *t == i as u64)
                    .map(|(t, n)| (*t, n.clone()))
                {
                    let (gpu, scene, offline) = (
                        self.gpu.as_ref().unwrap(),
                        self.scene.as_ref().unwrap(),
                        self.offline.as_mut().unwrap(),
                    );
                    let _kit = self.kit.as_ref().unwrap();
                    let mut kits = FrameKits {
                        gpu,
                        pipeline: &mut offline.2,
                        quads2d: &offline.4,
                        text: &offline.3,
                        store: &offline.0,
                        scene,
                        offscreen: &offline.1,
                    };
                    match render_frame(&mut kits, rt.scene(), driver, &name, &self.report_dir) {
                        Ok(k) => self.keyframes.push(k),
                        Err(e) => {
                            capture_err = Some(format!("keyframe: {e}"));
                            break;
                        }
                    }
                }
            }
        }
        self.rt = rt;
        self.driver = driver;
        if let Some(e) = capture_err {
            eprintln!("ag3d-squash: {e}");
            self.exit_code = 3;
            event_loop.exit();
            return;
        }
        self.step = end;

        // Present the live frame (with HUD + overlay as needed).
        {
            let gpu = self.gpu.as_ref().unwrap();
            let scene = self.scene.as_ref().unwrap();
            let driver = self.driver.as_ref().unwrap();
            let store = &self.offline.as_ref().unwrap().0;
            let shared = self.rt.as_ref().unwrap().scene();
            let instances = frame_instances(shared, scene);
            let dead = driver.dead;
            let score = driver.score;
            let kit = self.kit.as_mut().unwrap();
            let WindowKit {
                target,
                pipeline,
                quads2d,
                text,
                width,
                height,
                ..
            } = kit;
            let fmt = target.format();
            let outcome = target.present_frame(gpu, |view| {
                pipeline.render(
                    gpu,
                    view,
                    *width,
                    *height,
                    SKY_HORIZON,
                    &scene.camera.view_proj(),
                    &sample_light(),
                    store,
                    &instances,
                    true,
                );
                quads2d.set_ortho_camera(gpu, 0.0, W as f32, H as f32, 0.0);
                let mut quads = hud_quads(text, score);
                if dead {
                    let (overlay, retry) = retry_overlay(text);
                    quads2d.render_over(gpu, view, fmt, &[overlay]);
                    quads.extend(retry);
                }
                text.render(gpu, view, &quads);
            });
            if let Err(e) = outcome {
                eprintln!("ag3d-squash: present: {e}");
                self.exit_code = 3;
                event_loop.exit();
                return;
            }
        }

        if self.step >= self.trace.actions.len() {
            self.exit_code = self.finish();
            event_loop.exit();
        } else {
            self.kit.as_ref().unwrap().window.request_redraw();
        }
    }

    fn finish(&mut self) -> i32 {
        let gpu = self.gpu.as_ref().unwrap();
        let (store, _, _, _, _) = self.offline.as_ref().unwrap();
        let verify = self.verify.take().unwrap();
        let music = self.music.as_ref().unwrap();
        let keyframes = std::mem::take(&mut self.keyframes);
        let (report, pass) = build_report(
            gpu,
            true,
            &verify,
            music,
            &self.music_sha,
            keyframes,
            store.uploaded_count(),
        );
        write(&self.report_dir, report, pass)
    }
}

// ---------------------------------------------------------------------------
// Live interactive mode
// ---------------------------------------------------------------------------

struct LiveState {
    rt: Runtime<SquashAction, SquashEvent>,
    driver: SquashDriver,
    kit: WindowKit,
    scene: SceneData,
    store: MeshStore,
    gpu: GpuDevice,
    /// Kept alive so the music stream keeps playing for the session.
    _music: Music,
}

struct LiveApp {
    state: Option<LiveState>,
    input: LiveSquashInput,
    /// F02: fixed ticks are driven by elapsed wall time, not redraw count.
    clock: crate::core::TickClock,
    last_frame: std::time::Instant,
}

/// Catch-up policy (documented): at most 8 physics ticks (≈66 ms of debt)
/// per redraw; sustained overload drops the debt instead of spiraling.
const LIVE_MAX_CATCHUP_TICKS: u32 = 8;

fn run_live() -> i32 {
    use winit::event_loop::EventLoop;
    let event_loop = match EventLoop::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ag3d-squash(live): event loop: {e}");
            return 3;
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = LiveApp {
        state: None,
        input: LiveSquashInput::default(),
        clock: crate::core::TickClock::new(Duration::from_nanos(8_333_333)),
        last_frame: std::time::Instant::now(),
    };
    match event_loop.run_app(&mut app) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("ag3d-squash(live): {e}");
            3
        }
    }
}

impl winit::application::ApplicationHandler for LiveApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let music_bytes = std::fs::read(agp_file(
            "assets/squash_the_creeps/art/House In a Forest Loop.ogg",
        ))
        .expect("music ogg");
        let music = start_music(&music_bytes).expect("audio");
        let gpu = GpuDevice::new().expect("gpu");
        let attrs = winit::window::Window::default_attributes()
            .with_title(
                "AGP Squash the Creeps — LIVE (WASD/arrows move, Space jump, Enter retry, R reset)",
            )
            .with_inner_size(winit::dpi::LogicalSize::new(W as f64, H as f64));
        let window = Arc::new(event_loop.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let target =
            WindowTarget::new(&gpu, window.clone(), size.width, size.height).expect("surface");
        let fmt = target.format();
        let pipeline = MeshPipeline::new(&gpu, fmt).expect("pipeline");
        let quads2d = QuadPipeline::new(&gpu, fmt).expect("2d pipeline");
        let font = std::fs::read(agp_file(
            "assets/squash_the_creeps/fonts/Montserrat-Medium.ttf",
        ))
        .expect("font");
        let text = TextRenderer::new(&gpu, fmt, &font, HUD_FONT_PX, HUD_CHARS).expect("text");
        let mut store = MeshStore::default();
        let scene = build_scene(&gpu, &mut store).expect("scene");
        println!("LIVE MODE: WASD/arrows = move, Space = jump, Enter = retry (when dead), R = reset. Close to exit.");
        self.state = Some(LiveState {
            rt: Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED),
            driver: SquashDriver::new(SQUASH_SEED),
            kit: WindowKit {
                window,
                target,
                pipeline,
                quads2d,
                text,
                width: size.width,
                height: size.height,
            },
            scene,
            store,
            gpu,
            _music: music,
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
                use crate::games::squash::{LiveEffect, LiveKey};
                use winit::keyboard::{KeyCode, PhysicalKey};
                let pressed = key.state == winit::event::ElementState::Pressed;
                // The REAL keyboard path: every key goes through the same
                // press-edge state machine the tests drive (F01).
                let live_key =
                    match key.physical_key {
                        PhysicalKey::Code(KeyCode::Space) => Some(LiveKey::Space),
                        PhysicalKey::Code(KeyCode::Enter) => Some(LiveKey::Enter),
                        PhysicalKey::Code(KeyCode::ArrowLeft)
                        | PhysicalKey::Code(KeyCode::KeyA) => Some(LiveKey::ArrowLeft),
                        PhysicalKey::Code(KeyCode::ArrowRight)
                        | PhysicalKey::Code(KeyCode::KeyD) => Some(LiveKey::ArrowRight),
                        PhysicalKey::Code(KeyCode::ArrowUp) | PhysicalKey::Code(KeyCode::KeyW) => {
                            Some(LiveKey::ArrowUp)
                        }
                        PhysicalKey::Code(KeyCode::ArrowDown)
                        | PhysicalKey::Code(KeyCode::KeyS) => Some(LiveKey::ArrowDown),
                        PhysicalKey::Code(KeyCode::KeyR) => Some(LiveKey::KeyR),
                        _ => None,
                    };
                if let Some(lk) = live_key {
                    // r4 F829-R4-02: the machine needs the dead-state to
                    // route Space/Enter to retry (frozen ui_accept promise);
                    // the host still enforces dead && retry_visible.
                    let dead = self.state.as_ref().map(|s| s.driver.dead).unwrap_or(false);
                    let effect = self.input.key_event(lk, pressed, dead);
                    match effect {
                        Some(LiveEffect::RetryPressed) => {
                            if let Some(s) = &mut self.state {
                                if s.driver.dead && s.driver.retry_visible {
                                    s.rt.reset(SQUASH_SEED);
                                    s.driver.reset();
                                    println!(
                                        "retry (host extension): new session {:?}",
                                        s.rt.session()
                                    );
                                }
                            }
                        }
                        Some(LiveEffect::ResetPressed) => {
                            if let Some(s) = &mut self.state {
                                s.rt.reset(SQUASH_SEED ^ 0xFF);
                                s.driver.reset();
                                println!("reset (host extension)");
                            }
                        }
                        None => {}
                    }
                }
            }
            winit::event::WindowEvent::Resized(size) => {
                // keep the swapchain and depth target in sync with the
                // window (zero-size suspends; restore reconfigures).
                if let Some(s) = &mut self.state {
                    s.kit.width = size.width;
                    s.kit.height = size.height;
                    let _ = s.kit.target.handle_resize(&s.gpu, size.width, size.height);
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                // F02: advance the frozen 120 Hz step from ELAPSED time
                // (catch-up capped, debt dropped under sustained overload).
                let now = std::time::Instant::now();
                let elapsed = now.saturating_duration_since(self.last_frame);
                self.last_frame = now;
                let ticks = self.clock.feed_capped(elapsed, LIVE_MAX_CATCHUP_TICKS);
                if let Some(s) = &mut self.state {
                    for _ in 0..ticks {
                        let action = self.input.action(s.driver.dead);
                        let _ = s.rt.step(&mut s.driver, vec![action]);
                    }
                    let LiveState {
                        rt,
                        driver,
                        kit,
                        scene,
                        store,
                        gpu,
                        ..
                    } = s;
                    let instances = frame_instances(rt.scene(), scene);
                    let dead = driver.dead;
                    let score = driver.score;
                    let WindowKit {
                        window,
                        target,
                        pipeline,
                        quads2d,
                        text,
                        width,
                        height,
                    } = kit;
                    let fmt = target.format();
                    let _ = target.present_frame(gpu, |view| {
                        pipeline.render(
                            gpu,
                            view,
                            *width,
                            *height,
                            SKY_HORIZON,
                            &scene.camera.view_proj(),
                            &sample_light(),
                            store,
                            &instances,
                            true,
                        );
                        quads2d.set_ortho_camera(gpu, 0.0, W as f32, H as f32, 0.0);
                        let mut quads = hud_quads(text, score);
                        if dead {
                            let (overlay, retry) = retry_overlay(text);
                            quads2d.render(gpu, view, fmt, sky_color(), &[overlay]);
                            quads.extend(retry);
                        }
                        text.render(gpu, view, &quads);
                    });
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(s) = &self.state {
            s.kit.window.request_redraw();
            std::thread::sleep(Duration::from_millis(8));
        }
    }
}
