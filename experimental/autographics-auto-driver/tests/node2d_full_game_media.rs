//! auto-ui PLAN-032 node2d_full_game_media: the complete G2 world under the
//! REAL Host — offscreen GPU play with the admitted sprite/HUD/objects blocks
//! and the real audio mixer. Evidence on the hand-proven T-02 world:
//!   frames/flip — the admitted sprite block carries idle[0,0]/run[32]/
//!     rise[64]/fall[96] 32x32 rects and the flip latch exactly per G2-08;
//!   HUD — slot texts follow the root state ("0".."3", "Playing"/"Won") and
//!     render as viewport-anchored glyph pixels (t04 shape);
//!   enemy — the patrolling instance renders and its admitted position moves;
//!   camera — the character projection sweeps across the viewport;
//!   PCM — every captured closeout packet re-rendered through the real mixer
//!     equals its bank slice x request gain bit-exactly in PCM16;
//!   restart — the end-state Enter request rebuilds a fresh Run (count 0,
//!     coins restored, character at the spawn, new tick base).
//! Run (single instance, GPU + audio):
//!   CARGO_TERM_COLOR=never RUST_MIN_STACK=16777216 cargo test --locked \
//!     --offline --test node2d_full_game_media -- --test-threads=1
use autographics_auto_driver::{
    authoring::{
        assets,
        behavior::{Api, SourceSnapshot},
        host::Host,
        play::Action,
        public_audio::{
            self,
            mixer::{Accepted, Mixer, Packet, Request},
        },
        storage::Storage,
    },
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const SCENE: &str = include_str!("fixtures/plan032/scene-g2-components2d.agscene");
const WORLD: &str = include_str!("fixtures/plan032/world-g2-components2d.at");
const WORLD_SHA: &str = "dbb65232b51922624776beaeb29564e507a4aeadf48edf7ab4ccc7930eaa8cce";
const ATLAS_SHA: &str = "8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df";
const BANK_SHA: &str = "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d";
const FONT_SHA: &str = "4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa";
const PILOT: &str = "../autographics-node2d-pilot";
const DT: f32 = 1.0 / 60.0;

// ------------------------------------------------------------- staging ----

fn stage_game(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan032-media")
        .join(format!("{}-{tag}", std::process::id()))
        .join("game");
    let scripts = dir.join("scripts");
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::create_dir_all(&assets_dir).expect("assets dir");
    std::fs::write(dir.join("scene.agscene"), SCENE).expect("write scene");
    std::fs::write(scripts.join(format!("{WORLD_SHA}.at")), WORLD).expect("write script");
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PILOT);
    for (src, blob) in [
        (
            base.join(format!("fixtures/sprites2d/assets/{ATLAS_SHA}.png")),
            format!("{ATLAS_SHA}.png"),
        ),
        (
            base.join(format!("generated-components2d-game/game/closure/assets/{BANK_SHA}.wav")),
            format!("{BANK_SHA}.wav"),
        ),
        (
            base.join("generated-components2d-game/game/closure/assets/lilita_one_regular.ttf"),
            format!("{FONT_SHA}.ttf"),
        ),
    ] {
        std::fs::copy(&src, assets_dir.join(&blob))
            .unwrap_or_else(|e| panic!("copy asset {}: {e}", src.display()));
    }
    dir
}

fn load_project(
    tag: &str,
) -> (
    PathBuf,
    autographics_auto_driver::authoring::document::Document,
    assets::Registry,
    std::sync::Arc<autographics_auto_driver::authoring::behavior_project::CodePayload>,
) {
    let dir = stage_game(tag);
    let (d, r, code, _) = Storage::load_project(&dir.join("scene.agscene"), &assets::Registry::new())
        .unwrap_or_else(|e| panic!("load staged g2 project: {e:?}"));
    let code = code.expect("g2 world bound as behavior code");
    (dir, d, r, code)
}

// --------------------------------------------------------- VM capture -----

fn session(world_src: &str, name: &str, run: i32) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world_src.as_bytes().to_vec(),
        Api::Components2V1,
    )
    .unwrap_or_else(|e| panic!("{name} admission: {e}"));
    let mut s = Session::new(&snap.source).unwrap_or_else(|e| panic!("{name} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(12)], ResultKind::Unit)
        .unwrap();
    let rows: [(i32, i32, f32, f32, f32, f32); 12] = [
        (1, 1, 2.0, 1.0, 0.8, 1.0),
        (2, 1, 4.0, -0.5, 8.0, 1.0),
        (3, 1, 14.0, -0.5, 8.0, 1.0),
        (4, 1, 25.0, -0.5, 10.0, 1.0),
        (5, 1, 13.5, 1.125, 3.0, 0.25),
        (6, 1, -0.2, 3.0, 0.4, 14.0),
        (7, 1, 30.2, 3.0, 0.4, 14.0),
        (8, 1, 15.0, 10.2, 30.8, 0.4),
        (9, 0, -30.0, -40.0, 0.5, 0.5),
        (10, 0, -30.0, -42.0, 0.9, 0.9),
        (11, 0, -30.0, -44.0, 0.6, 1.0),
        (12, 0, -30.0, -46.0, 12.0, 0.5),
    ];
    for (n, (id, kind, tx, ty, sx, sy)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(*id), Int(*kind), Int(2), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit).unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [*tx, *ty, 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in [1.0, -sx / 2., -sy / 2., 0., sx / 2., sy / 2., 0.]
            .iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..12 {
        for name in ["_ag_char_map", "_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n + 1), Int(n)], ResultKind::Unit).unwrap();
        }
    }
    for (name, args) in [
        ("_ag_spr_map", vec![Int(1), Int(0), Int(0)]),
        ("_ag_spr_img", vec![Int(0), Int(160), Int(32)]),
        ("_ag_spr_def", vec![Int(0), Int(0), Int(0), Int(32)]),
        ("_ag_spr_defh", vec![Int(0), Int(32), Int(1)]),
    ] {
        s.call(name, &args, ResultKind::Unit).unwrap();
    }
    s.call("_ag_audio_config", &[Int(4)], ResultKind::Unit).unwrap();
    let state_glyphs = "PlayingWonedF";
    for (slot, count, codes) in [
        (0, 4usize, vec![48i32, 49, 50, 51]),
        (
            1,
            state_glyphs.len(),
            state_glyphs.bytes().map(|b| b as i32).collect(),
        ),
        (2, 0, vec![]),
        (3, 0, vec![]),
    ] {
        s.call(
            "_ag_hud_config",
            &[Int(slot), Int(1), Int(count as i32)],
            ResultKind::Unit,
        )
        .unwrap();
        for code in codes {
            s.call(
                "_ag_hud_config_code",
                &[Int(slot), Int(code)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "{name} app_start");
    s
}

fn vtick(s: &mut Session, t: i32, held: i32, pressed: i32) -> i32 {
    s.call("_ag_char_input", &[Int(held), Int(pressed)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(t), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}
fn vpx(s: &mut Session) -> f32 {
    s.float("get_px", &[]).unwrap()
}
fn vpy(s: &mut Session) -> f32 {
    s.float("get_py", &[]).unwrap()
}
fn vgr(s: &mut Session) -> i32 {
    s.int("get_p_grounded", &[]).unwrap()
}
fn vst(s: &mut Session) -> i32 {
    s.int("get_r_state", &[]).unwrap()
}
/// One input tick that also records this tick's closeout packet, if any.
fn ctick(
    s: &mut Session,
    caps: &mut Vec<(i32, Vec<(i32, i32, f32)>)>,
    t: i32,
    held: i32,
    pressed: i32,
) -> i32 {
    let status = vtick(s, t, held, pressed);
    let n = s.int("_ag_audio_count", &[]).unwrap();
    if n > 0 {
        let rows = (0..n)
            .map(|i| {
                (
                    s.int("_ag_audio_i", &[Int(i), Int(0)]).unwrap(),
                    s.int("_ag_audio_i", &[Int(i), Int(1)]).unwrap(),
                    s.float("_ag_audio_g", &[Int(i)]).unwrap(),
                )
            })
            .collect();
        caps.push((t, rows));
    }
    status
}

/// Drive the complete win route on the VM session and capture every
/// closeout packet with its real tick identity (the mixer oracle input).
fn capture_route(s: &mut Session) -> Vec<(i32, Vec<(i32, i32, f32)>)> {
    use vgr as gr;
    use vpx as px;
    use vpy as py;
    use vst as st;
    let mut t = 0;
    let mut caps: Vec<(i32, Vec<(i32, i32, f32)>)> = Vec::new();
    fn phase(
        s: &mut Session,
        caps: &mut Vec<(i32, Vec<(i32, i32, f32)>)>,
        t: i32,
        cond: &dyn Fn(&mut Session) -> bool,
        held: i32,
        bound: i32,
        name: &str,
    ) -> i32 {
        let mut t = t;
        let mut g = 0;
        while !cond(s) {
            t += 1;
            assert_eq!(ctick(s, caps, t, held, 0), 0, "{name} tick {t}");
            g += 1;
            assert!(g < bound, "{name} bound");
        }
        t
    }
    // settle
    for i in 1..=90 {
        assert_eq!(ctick(s, &mut caps, i, 0, 0), 0);
        t = i;
    }
    // coin 1
    t = phase(s, &mut caps, t, &|s| px(s) >= 3.2, 2, 200, "coin1 approach");
    for _ in 0..3 {
        t += 1;
        assert_eq!(ctick(s, &mut caps, t, 2, 0), 0);
    }
    // pit 1
    t = phase(s, &mut caps, t, &|s| px(s) >= 7.35, 2, 200, "pit1 edge");
    t += 1;
    assert_eq!(ctick(s, &mut caps, t, 2, 4), 0, "jump1");
    for _ in 0..20 {
        t += 1;
        assert_eq!(ctick(s, &mut caps, t, 2, 4), 0);
    }
    t = phase(s, &mut caps, t, &|s| gr(s) == 1, 2, 120, "land seg2");
    // wait out the patrol, climb the top platform
    t = phase(s, &mut caps, t, &|s| px(s) <= 10.1, 1, 200, "back off");
    t = phase(
        s,
        &mut caps,
        t,
        &|s| s.float("get_ex", &[]).unwrap() >= 14.0 && s.int("get_e_dir", &[]).unwrap() == 1,
        0,
        1200,
        "phase wait",
    );
    t = phase(s, &mut caps, t, &|s| px(s) >= 11.35, 2, 200, "launch walk");
    t += 1;
    assert_eq!(ctick(s, &mut caps, t, 2, 4), 0, "jump2");
    t = phase(s, &mut caps, t, &|s| py(s) >= 1.74, 2, 60, "rise");
    t = phase(s, &mut caps, t, &|s| gr(s) == 1, 2, 120, "land top");
    // coin 2
    t = phase(s, &mut caps, t, &|s| px(s) >= 12.7, 2, 200, "top walk");
    for _ in 0..4 {
        t += 1;
        assert_eq!(ctick(s, &mut caps, t, 2, 0), 0);
    }
    // wait, drop, cross to pit 2
    t = phase(s, &mut caps, t, &|s| s.float("get_ex", &[]).unwrap() <= 12.0, 0, 2400, "top phase");
    t = phase(s, &mut caps, t, &|s| gr(s) == 0, 2, 120, "drop walk");
    t = phase(s, &mut caps, t, &|s| gr(s) == 1, 2, 120, "drop land");
    t = phase(s, &mut caps, t, &|s| px(s) >= 17.35, 2, 200, "pit2 edge");
    t += 1;
    assert_eq!(ctick(s, &mut caps, t, 2, 4), 0, "jump3");
    for _ in 0..20 {
        t += 1;
        assert_eq!(ctick(s, &mut caps, t, 2, 0), 0);
    }
    t = phase(s, &mut caps, t, &|s| gr(s) == 1, 2, 120, "land seg3");
    // coin 3 + goal
    t = phase(s, &mut caps, t, &|s| px(s) >= 23.1, 2, 200, "coin3 walk");
    for _ in 0..4 {
        t += 1;
        assert_eq!(ctick(s, &mut caps, t, 2, 0), 0);
    }
    t = phase(s, &mut caps, t, &|s| px(s) >= 27.2, 2, 200, "goal walk");
    for _ in 0..4 {
        t += 1;
        let status = ctick(s, &mut caps, t, 2, 0);
        assert_eq!(status, 0, "win tick {t}");
    }
    assert_eq!(st(s), 1, "Won at the end of the captured route");
    caps
}

// ------------------------------------------------------- bank oracle ------

fn bank_oracle(registry: &assets::Registry) -> (Vec<i16>, Vec<[f32; 2]>) {
    let asset = registry.values().find(|a| a.audio.is_some()).unwrap();
    assert_eq!(asset.identity.sha256, BANK_SHA);
    let bytes = &asset.bytes[..];
    assert_eq!(&bytes[..4], b"RIFF");
    let raw: Vec<i16> = bytes[44..]
        .chunks_exact(2)
        .map(|p| i16::from_le_bytes([p[0], p[1]]))
        .collect();
    let frames: Vec<[f32; 2]> = raw
        .chunks_exact(2)
        .map(|p| [f32::from(p[0]) / 32768.0, f32::from(p[1]) / 32768.0])
        .collect();
    (raw, frames)
}

fn max_abs_delta(actual: &[[f32; 2]], expected: &[[f32; 2]]) -> f32 {
    actual
        .iter()
        .zip(expected)
        .map(|(a, e)| (a[0] - e[0]).abs().max((a[1] - e[1]).abs()))
        .fold(0.0f32, f32::max)
}

fn max_abs_delta_i16(actual: &[[f32; 2]], raw: &[i16], start_frame: usize, scale: f32) -> i64 {
    actual
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let base = (start_frame + i) * 2;
            let l = i64::from((a[0] * 32768.0).round() as i32)
                - i64::from((f32::from(raw[base]) * scale).round() as i32);
            let r = i64::from((a[1] * 32768.0).round() as i32)
                - i64::from((f32::from(raw[base + 1]) * scale).round() as i32);
            l.abs().max(r.abs())
        })
        .max()
        .unwrap()
}

fn render_one(cues: &[public_audio::mixer::Cue], index: usize, gain: f32) -> Vec<[f32; 2]> {
    let len = cues[index].end - cues[index].start;
    let mut m = Mixer::default();
    m.begin(1).unwrap();
    let packet = Packet {
        run: 1,
        tick: 0,
        bundle: 0,
        requests: vec![Request {
            ordinal: 1,
            cue: index,
            gain,
        }],
    };
    assert_eq!(m.accept(packet, cues, false, || Ok(())).unwrap(), Accepted::Played);
    let mut out = vec![[0.0; 2]; len];
    m.render(&mut out, 44100);
    out
}

// -------------------------------------------------------- host helpers ----

type Px = ([f32; 2], [u8; 4]);

fn pixel_classes(h: &mut Host) -> Vec<Px> {
    let mut points = Vec::new();
    for gy in 0..95 {
        for gx in 0..200 {
            points.push([(gx * 4 + 2) as f32, (gy * 4 + 2) as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.top, &points).expect("probe");
    points.iter().zip(px.iter()).map(|(p, c)| (*p, *c)).collect()
}

fn rgb(c: [u8; 4]) -> (f32, f32, f32) {
    (c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0)
}

fn is_white(c: [u8; 4]) -> bool {
    let (r, g, b) = rgb(c);
    r > 0.85 && g > 0.85 && b > 0.85
}

/// Top platform quad (declared [0.6,0.6,0.62] -> gamma-lifted ~0.79-0.80).
fn is_top_platform(c: [u8; 4]) -> bool {
    let (r, g, b) = rgb(c);
    (r - g).abs() < 0.04 && r > 0.74 && r < 0.86 && b >= g
}

fn is_enemy_red(c: [u8; 4]) -> bool {
    // declared [0.8,0.2,0.2] -> gamma-lifted ~ (0.905, 0.483, 0.483)
    let (r, g, b) = rgb(c);
    r > 0.8 && g < 0.58 && b < 0.58 && (r - g) > 0.25
}

fn in_box(px: &[Px], x0: f32, x1: f32, y0: f32, y1: f32, pred: fn([u8; 4]) -> bool) -> Vec<Px> {
    px.iter()
        .filter(|(p, c)| p[0] >= x0 && p[0] <= x1 && p[1] >= y0 && p[1] <= y1 && pred(*c))
        .copied()
        .collect()
}

fn centroid(px: &[Px]) -> (f32, f32) {
    let (mut sx, mut sy) = (0.0f32, 0.0f32);
    for (p, _) in px {
        sx += p[0];
        sy += p[1];
    }
    (sx / px.len() as f32, sy / px.len() as f32)
}

fn actor(h: &Host, id: u64) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.nodes
        .get(&id)
        .map(|d| d.translation)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

/// Grounded probe from the admitted character state (host-side).
fn is_grounded_host(h: &Host) -> bool {
    let a = actor(h, 1);
    (a[1] - 0.5).abs() < 1e-2 || (a[1] - 1.75).abs() < 1e-2
}

/// The admitted Run id of the last accepted frame.
fn run_id(h: &Host) -> i32 {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .run_id
}

/// The admitted 2D camera world rect (min_x, min_y, max_x, max_y).
fn camera_rect(h: &Host) -> [f32; 4] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.camera.as_ref().expect("camera block").rect
}

/// The admitted live instance translation in `slot`.
fn instance_xy(h: &Host, slot: usize) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.instances
        .as_ref()
        .expect("instances block")
        .get(slot)
        .unwrap_or_else(|| panic!("slot {slot} live"))
        .translation
}

fn sprite_visual(h: &Host, id: u64) -> ([u32; 4], bool) {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    let sprites = f.sprites.as_ref().expect("sprite block");
    let v = sprites.visual_for(id).unwrap_or_else(|| panic!("sprite target {id}"));
    (v.frame, v.flip)
}

fn slot_text(h: &Host, i: usize) -> String {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.hud.as_ref().expect("hud frame").slots[i].text.clone()
}

fn settle_host(h: &mut Host) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or(false);
        if !pending {
            break;
        }
        assert!(std::time::Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

/// One live host tick; returns the tick's closeout rows + mixer disposition.
fn step_play(h: &mut Host, held: u32, pressed: u32) -> (Vec<(i32, usize, f32)>, Option<Accepted>) {
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        for action in 0..4usize {
            if pressed & (1 << action) != 0 {
                p.char_key(action, true, false);
            } else if held & (1 << action) != 0 {
                p.char_held_restore(action);
            } else {
                p.char_key(action, false, false);
            }
        }
        if p.dispatch(Action::default()).is_err() {
            return (vec![], None);
        }
    }
    h.pause_play("");
    settle_host(h);
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    let rows = f
        .audio
        .as_ref()
        .map(|p| {
            p.requests
                .iter()
                .map(|r| (r.ordinal, r.cue, r.gain))
                .collect()
        })
        .unwrap_or_default();
    (rows, f.audio_disposition)
}

// -------------------------------------------------------------- tests -----

/// PCM oracle: every closeout packet captured on the real route re-rendered
/// through the real mixer matches its bank slice at the requested gain.
#[test]
fn g2_pcm_route_through_real_mixer() {
    let mut s = session(WORLD, "g2-pcm.at", 1);
    let caps = capture_route(&mut s);
    let total: usize = caps.iter().map(|(_, rows)| rows.len()).sum();
    assert!(total >= 8, "route must carry jump x3 + coin x3 + win: {total}");
    drop(s);
    let (_dir, doc, registry, _code) = load_project("pcm");
    let (raw, _frames) = bank_oracle(&registry);
    let (_, frames) = bank_oracle(&registry);
    let cues = public_audio::resolve_cues(&doc, &registry).unwrap();
    assert_eq!(cues.len(), 4, "four declared cues");
    for (t, rows) in &caps {
        for (ordinal, cue, gain) in rows {
            let c = &cues[*cue as usize];
            let expected: Vec<[f32; 2]> = frames[c.start..c.end]
                .iter()
                .map(|f| [f[0] * gain, f[1] * gain])
                .collect();
            let rendered = render_one(&cues, *cue as usize, *gain);
            let delta = max_abs_delta(&rendered, &expected);
            let lsb = max_abs_delta_i16(&rendered, &raw, c.start, *gain);
            assert!(
                delta <= 2.0e-5,
                "tick {t} ordinal {ordinal} cue {cue}: delta {delta}"
            );
            assert_eq!(lsb, 0, "tick {t} cue {cue} bit-exact in PCM16");
        }
    }
    println!("G2_PCM: {total} rows across {} packets, all bit-exact vs bank x gain", caps.len());
}

/// GPU evidence: admitted sprite frames/flip per G2-08, HUD text per the root
/// state, enemy patrol visible, camera sweep, and the end-state Enter
/// rebuilds a fresh Run (count 0, coins restored, new tick base).
#[test]
fn g2_gpu_frames_hud_enemy_camera_and_fresh_run() {
    let (_dir, doc, registry, code) = load_project("gpu");
    let mut h = Host::with_project(doc, registry, Some(code)).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("start run: {e:?}"));
    h.pause_play("");
    settle_host(&mut h);
    assert!(h
        .play
        .as_ref()
        .unwrap()
        .error
        .clone()
        .unwrap_or_default()
        .is_empty());

    // Settle onto the ground from the spawn (idle ticks), then the idle frame.
    for _ in 0..45 {
        let _ = step_play(&mut h, 0, 0);
    }
    let (f0, fl0) = sprite_visual(&h, 1);
    assert_eq!(f0, [0, 0, 32, 32], "idle frame rect: {f0:?}");
    assert!(!fl0, "idle faces right");
    assert_eq!(slot_text(&h, 0), "0");
    assert_eq!(slot_text(&h, 1), "Playing");
    let a0 = actor(&h, 1);
    assert!((a0[0] - 2.0).abs() < 1e-2, "rest x: {a0:?}");
    assert!((a0[1] - 0.5).abs() < 1e-2, "landed at rest: {a0:?}");

    // Run frame while walking right.
    for _ in 0..6 {
        let _ = step_play(&mut h, 1 << 1, 0);
    }
    let (f1, fl1) = sprite_visual(&h, 1);
    assert_eq!(f1, [32, 0, 32, 32], "run frame rect: {f1:?}");
    assert!(!fl1, "run right unflipped");

    // Walk left: flip latches on (G2-08 direction).
    for _ in 0..6 {
        let _ = step_play(&mut h, 1 << 0, 0);
    }
    let (_fx, fl2) = sprite_visual(&h, 1);
    assert!(fl2, "left walk flips the sprite");

    // Jump: rise frame [64,0,32,32] on the launch tick; past the apex
    // (24 ticks of rise at vy=8/g=20) the frame is fall [96,0,32,32].
    let (_, disp) = step_play(&mut h, 1 << 2, 1 << 2);
    let (fr, _) = sprite_visual(&h, 1);
    assert_eq!(fr, [64, 0, 32, 32], "rise frame rect: {fr:?}");
    let _ = disp;
    for _ in 0..30 {
        let _ = step_play(&mut h, 0, 0);
    }
    let (ff, _) = sprite_visual(&h, 1);
    assert_eq!(ff, [96, 0, 32, 32], "fall frame rect: {ff:?}");

    // Enemy patrol: the admitted instance (slot 3) stays inside its frozen
    // patrol range, moves across ticks, and renders as the red template quad.
    let e0 = instance_xy(&h, 3);
    assert!(e0[0] >= 11.0 && e0[0] <= 17.0, "enemy in patrol range: {e0:?}");
    // Bring the enemy into the settled camera's view (x<12.2 with the player
    // at 2 keeps it inside [0,12]) before the pixel scan.
    let mut guard = 0;
    while instance_xy(&h, 3)[0] > 12.2 {
        let _ = step_play(&mut h, 0, 0);
        guard += 1;
        assert!(guard < 1500, "enemy approaches the visible band");
    }
    let cap0 = pixel_classes(&mut h);
    let red0 = in_box(&cap0, 0.0, 800.0, 60.0, 440.0, is_enemy_red);
    assert!(red0.len() >= 3, "enemy renders red: {}", red0.len());
    let mut moved = false;
    for _ in 0..40 {
        let _ = step_play(&mut h, 0, 0);
        let e = instance_xy(&h, 3);
        if (e[0] - e0[0]).abs() > 0.1 {
            moved = true;
            break;
        }
    }
    assert!(moved, "enemy patrols (admitted x moves)");

    // Camera follow + clamp (G2-07): with the character near the left clamp
    // the view is pinned to [0,12] (the top platform at x=12..15 is outside);
    // after walking right past the clamp the panned view reveals it.
    let cam0 = camera_rect(&h);
    assert_eq!(cam0, [0.0, -4.0, 30.0, 12.0], "declared clamp rect rides the frame");
    let cap_at = pixel_classes(&mut h);
    let top_at = in_box(&cap_at, 0.0, 800.0, 0.0, 440.0, is_top_platform);
    let a_before = actor(&h, 1)[0];
    assert!(top_at.len() <= 8, "top platform only a boundary sliver at the clamped start view: {}", top_at.len());
    // Walk right and jump pit 1 so the follow camera pans past the clamp.
    let mut guard = 0;
    while actor(&h, 1)[0] < 7.3 {
        let _ = step_play(&mut h, 1 << 1, 0);
        guard += 1;
        assert!(guard < 200, "pan walk bound");
    }
    let _ = step_play(&mut h, 1 << 1, 1 << 2);
    for _ in 0..34 {
        let _ = step_play(&mut h, 1 << 1, 0);
    }
    let mut guard = 0;
    while !is_grounded_host(&h) {
        let _ = step_play(&mut h, 1 << 1, 0);
        guard += 1;
        assert!(guard < 120, "pan jump must land");
    }
    for _ in 0..40 {
        let _ = step_play(&mut h, 1 << 1, 0);
    }
    let a_after = actor(&h, 1)[0];
    assert!(a_after - a_before > 2.0, "character world x advanced: {a_before} -> {a_after}");
    let cap_after = pixel_classes(&mut h);
    let top_after = in_box(&cap_after, 0.0, 800.0, 0.0, 440.0, is_top_platform);
    assert!(top_after.len() >= 60, "panned view reveals the top platform: {}", top_after.len());
    println!("G2_GPU frames/flip/HUD/enemy/camera all observed");
}

/// The complete win route on the real Host, then the end-state Enter
/// rebuilding a fresh Run: HUD 3/Won with rendered glyphs, restart restores
/// count/state/spawn with a new tick base.
#[test]
fn g2_gpu_win_route_hud_and_fresh_run() {
    let (_dir, doc, registry, code) = load_project("win");
    let mut h = Host::with_project(doc, registry, Some(code)).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("start run: {e:?}"));
    h.pause_play("");
    settle_host(&mut h);
    for _ in 0..45 {
        let _ = step_play(&mut h, 0, 0);
    }
    let walk = |h: &mut Host, target: f32, max_steps: usize| {
        let mut g = 0;
        while actor(h, 1)[0] < target {
            let _ = step_play(h, 1 << 1, 0);
            g += 1;
            assert!(g < max_steps, "walk to {target} bound");
        }
    };
    let idle = |h: &mut Host, n: usize| {
        for _ in 0..n {
            let _ = step_play(h, 0, 0);
        }
    };
    // coin 1
    walk(&mut h, 3.2, 200);
    idle(&mut h, 4);
    assert_eq!(slot_text(&h, 0), "1", "coin 1 on the HUD");
    // release-stop check (G2-01) at the block level
    let xa = actor(&h, 1)[0];
    idle(&mut h, 12);
    assert!((actor(&h, 1)[0] - xa).abs() < 1e-3, "release stops the drift");
    // pit 1
    walk(&mut h, 7.3, 200);
    let _ = step_play(&mut h, 1 << 1, 1 << 2);
    for _ in 0..20 {
        let _ = step_play(&mut h, 1 << 1, 0);
    }
    let mut g = 0;
    while !is_grounded_host(&h) {
        let _ = step_play(&mut h, 1 << 1, 0);
        g += 1;
        assert!(g < 120, "pit1 landing");
    }
    // patrol phase, climb the top platform
    {
        let mut g = 0;
        while actor(&h, 1)[0] > 10.1 {
            let _ = step_play(&mut h, 1 << 0, 0);
            g += 1;
            assert!(g < 200, "back off");
        }
    }
    {
        let mut g = 0;
        while instance_xy(&h, 3)[0] < 14.0 {
            let _ = step_play(&mut h, 0, 0);
            g += 1;
            assert!(g < 1500, "phase wait");
        }
    }
    walk(&mut h, 11.35, 200);
    let _ = step_play(&mut h, 1 << 1, 1 << 2);
    {
        let mut g = 0;
        while actor(&h, 1)[1] < 1.74 {
            let _ = step_play(&mut h, 1 << 1, 0);
            g += 1;
            assert!(g < 60, "rise");
        }
    }
    {
        let mut g = 0;
        while !is_grounded_host(&h) {
            let _ = step_play(&mut h, 1 << 1, 0);
            g += 1;
            assert!(g < 120, "land top");
        }
    }
    // coin 2
    walk(&mut h, 12.7, 200);
    idle(&mut h, 4);
    assert_eq!(slot_text(&h, 0), "2", "coin 2 on the HUD");
    // top phase wait, drop, cross to pit 2
    {
        let mut g = 0;
        while instance_xy(&h, 3)[0] > 12.0 {
            let _ = step_play(&mut h, 0, 0);
            g += 1;
            assert!(g < 2400, "top phase");
        }
    }
    {
        let mut g = 0;
        while is_grounded_host(&h) && actor(&h, 1)[1] > 0.6 {
            let _ = step_play(&mut h, 1 << 1, 0);
            g += 1;
            assert!(g < 120, "drop walk");
        }
    }
    {
        let mut g = 0;
        while !is_grounded_host(&h) {
            let _ = step_play(&mut h, 1 << 1, 0);
            g += 1;
            assert!(g < 120, "drop land");
            assert!(actor(&h, 1)[0] < 18.0, "no pit 2 entry");
        }
    }
    walk(&mut h, 17.3, 200);
    let _ = step_play(&mut h, 1 << 1, 1 << 2);
    for _ in 0..20 {
        let _ = step_play(&mut h, 1 << 1, 0);
    }
    {
        let mut g = 0;
        while !is_grounded_host(&h) {
            let _ = step_play(&mut h, 1 << 1, 0);
            g += 1;
            assert!(g < 120, "pit2 landing");
        }
    }
    // coin 3 + goal
    walk(&mut h, 23.1, 200);
    idle(&mut h, 4);
    assert_eq!(slot_text(&h, 0), "3", "coin 3 on the HUD");
    walk(&mut h, 27.2, 200);
    idle(&mut h, 4);
    assert_eq!(slot_text(&h, 1), "Won", "state HUD at win");
    let capw = pixel_classes(&mut h);
    let whitew = in_box(&capw, 0.0, 800.0, 2.0, 58.0, is_white);
    assert!(whitew.len() >= 6, "HUD glyphs render at win: {}", whitew.len());

    // Fresh Run: the end-state Enter edge on a Running dispatch rebuilds the
    // play from the same snapshot (round restart, t06/g11 shape).
    let tick_before = h
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .tick;
    let old_run = run_id(&h);
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        p.char_key(3, true, false);
        p.dispatch(Action::default()).expect("dispatch");
    }
    settle_host(&mut h);
    let mut swapped = run_id(&h) != old_run;
    for _ in 0..4 {
        if swapped {
            break;
        }
        {
            let p = h.play.as_mut().unwrap();
            let _ = p.resume();
        }
        h.frame(None).expect("frame");
        swapped = run_id(&h) != old_run;
    }
    settle_host(&mut h);
    for _ in 0..30 {
        let _ = step_play(&mut h, 0, 0);
    }
    assert!(swapped, "fresh run after Enter: run {old_run}");
    assert_eq!(slot_text(&h, 0), "0", "fresh run count restored");
    assert_eq!(slot_text(&h, 1), "Playing", "fresh run state restored");
    let a1 = actor(&h, 1);
    assert!((a1[0] - 2.0).abs() < 1e-1, "fresh run at spawn: {a1:?}");
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame after restart");
    assert!(f.tick < tick_before, "fresh run tick base restarted: {} -> {}", tick_before, f.tick);
    let inst = f.instances.as_ref().expect("instances after restart");
    assert_eq!(inst.slots.iter().filter(|s| s.live()).count(), 6, "coins restored in the fresh run");
    println!("G2_GPU win route + fresh run observed (tick {tick_before} -> {})", f.tick);
}
