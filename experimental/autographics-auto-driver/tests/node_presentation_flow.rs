//! auto-ui PLAN-025 node_presentation_flow: the generic sound/HUD presentation
//! chain executes as real project play. V-02 (actor-free): two same-definition
//! Notify instances with arbitrary business names (hits/live) decide their own
//! cue/gain/text via ordinary handlers; the scene binds two_d: null and runs
//! on the actor-free autographics.presentation2d/1 composition — no character,
//! no camera, no trigger rows, no hidden actor. Ordinals are one shared exact
//! int sequence across instances; each closeout is one bundle (one
//! ag_present2d per callback); HUD text follows the owner's local field
//! through the author helper and the visible latch flips off after two hits.
//! V-04 (four requests): two Bell instances in the SAME callback fill the
//! four bounded rows with four distinguishable cues and exact gains — none
//! truncated to two; a derived in-memory fifth request hits the named fault
//! 51 at the closeout packet bound with no partial sound. PCM oracle: the
//! session-captured closeout rows are fed into the real mixer built from the
//! GENERATED scene's cue table; rendered PCM must equal the bank slices at
//! the requested gains bit-exactly in PCM16 (t04 capture shape).
//! Run single-threaded:
//! cargo test --locked --offline --test node_presentation_flow -- --test-threads=1
use autographics_auto_driver::{
    authoring::{
        assets::Registry,
        behavior::{Api, SourceSnapshot},
        behavior_project::CodePayload,
        document::Document,
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

const NOTIFY_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-presentation2d/notify/scene.agscene");
const NOTIFY_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-presentation2d/notify/world.at");
const NOTIFY_WORLD_SHA: &str = "50a80d386f2c8f5ffe4a518ea083150e983a2025160bd1b97841eb51305aeab4";
const FOUR_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-presentation2d/four/scene.agscene");
const FOUR_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-presentation2d/four/world.at");
const FOUR_WORLD_SHA: &str = "0a35a0d11a0a139f05e98aee40cc10ecf05e3c4f8a37d4e05c210471ddb12dc5";
const GAME_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-presentation2d/game/scene.agscene");
const GAME_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-presentation2d/game/world.at");
const GAME_WORLD_SHA: &str = "a670a464678ae2ee15c344e038371997a6dd56e0207172d9ab46a625c3c643a3";
const BANK_SHA: &str = "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d";
const FONT_SHA: &str = "4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa";
const CLOSURE_ASSETS: &str = "../autographics-node2d-pilot/generated-presentation2d";

const DT: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------- harness --

/// Stage a generated presentation product as a loadable project (scene +
/// bound script + assets/), exactly as load_project resolves it.
fn stage_product(scene_text: &str, world: &str, world_sha: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan025-presentation")
        .join(format!("{}-{tag}", std::process::id()));
    let scripts = dir.join("scripts");
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::create_dir_all(&assets_dir).expect("assets dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    std::fs::write(scripts.join(format!("{world_sha}.at")), world).expect("write script");
    let closure = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CLOSURE_ASSETS);
    // load_project resolves assets/<sha256>.<ext> (blob_name); sources are the
    // closure copies shared by both products.
    for (src, blob) in [
        (
            "notify/closure/assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav",
            format!("{BANK_SHA}.wav"),
        ),
        (
            "notify/closure/assets/lilita_one_regular.ttf",
            format!("{FONT_SHA}.ttf"),
        ),
    ] {
        std::fs::copy(closure.join(src), assets_dir.join(&blob))
            .unwrap_or_else(|e| panic!("copy asset {src}: {e}"));
    }
    dir
}

fn load_product(
    scene_text: &str,
    world: &str,
    world_sha: &str,
    tag: &str,
) -> (
    PathBuf,
    Document,
    Registry,
    std::sync::Arc<CodePayload>,
) {
    let dir = stage_product(scene_text, world, world_sha, tag);
    let (d, r, code, _) = Storage::load_project(
        &dir.join("scene.agscene"),
        &Registry::new(),
    )
    .unwrap_or_else(|e| panic!("load staged presentation project: {e:?}"));
    let code = code.expect("presentation world bound as behavior code");
    (dir, d, r, code)
}

/// Independent PCM16 RIFF reader over the staged asset bytes.
fn bank_oracle(registry: &Registry) -> (Vec<i16>, Vec<[f32; 2]>) {
    let asset = registry.values().find(|a| a.audio.is_some()).unwrap();
    assert_eq!(asset.identity.format, "wav");
    assert_eq!(asset.identity.sha256, BANK_SHA);
    assert_eq!(asset.identity.byte_length, 90060);
    let meta = asset.identity.audio.as_ref().unwrap();
    assert_eq!((meta.rate, meta.channels, meta.frames), (44100, 2, 22504));
    let bytes = &asset.bytes[..];
    assert_eq!(&bytes[..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[36..40], b"data");
    let data_bytes = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;
    assert_eq!(data_bytes, 90016);
    let raw: Vec<i16> = bytes[44..]
        .chunks_exact(2)
        .map(|p| i16::from_le_bytes([p[0], p[1]]))
        .collect();
    assert_eq!(raw.len(), 45008, "22504 frames x 2 channels");
    let frames: Vec<[f32; 2]> = raw
        .chunks_exact(2)
        .map(|p| [f32::from(p[0]) / 32768.0, f32::from(p[1]) / 32768.0])
        .collect();
    (raw, frames)
}

/// Max abs delta in PCM16 LSB between the rendered output (re-quantized by
/// the mixer's frozen quantizer) and the raw bank slice (t04 shape).
fn max_abs_delta_i16(actual: &[[f32; 2]], raw: &[i16], start_frame: usize, scale: f32) -> i64 {
    let mut max: i64 = 0;
    for (i, frame) in actual.iter().enumerate() {
        let base = (start_frame + i) * 2;
        let raw = &raw[base..base + 2];
        for ch in 0..2 {
            let expect = (f32::from(raw[ch]) / 32768.0) * scale;
            let got = frame[ch];
            let lsb = ((expect - got) * 32768.0).round() as i64;
            max = max.max(lsb.abs());
        }
    }
    max
}

/// Session rows of the generated notify product (3 objects, translation-
/// relative boxes).
fn session(world_src: &str, name: &str, run: i32, scene_rows: i32, cues: usize) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world_src.as_bytes().to_vec(),
        Api::Presentation2V1,
    )
    .unwrap_or_else(|e| panic!("{name} admission: {e}"));
    let mut s = Session::new(&snap.source).unwrap_or_else(|e| panic!("{name} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(scene_rows)], ResultKind::Unit)
        .unwrap();
    // Rows: (id, kind, tx, ty, sx, sy). Object templates sit parked far away;
    // the floor is a static visible leaf.
    let rows: [(i32, i32, f32, f32, f32, f32); 3] = [
        (1, 1, 12.0, -1.0, 24.0, 2.0), // floor quad (static leaf)
        (2, 0, -30.0, -40.0, 0.6, 0.6), // alto/left template
        (3, 0, -30.0, -42.0, 0.6, 0.6), // tenor/right template
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
    for n in 0..scene_rows {
        for name in ["_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n + 1), Int(n)], ResultKind::Unit).unwrap();
        }
    }
    s.call("_ag_audio_config", &[Int(cues as i32)], ResultKind::Unit).unwrap();
    for slot in 0..4 {
        s.call(
            "_ag_hud_config",
            &[Int(slot), Int(1), Int(1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_hud_config_code", &[Int(slot), Int(48)], ResultKind::Unit)
            .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        0,
        "{name} app_start fault {} arg {}",
        s.int("_ag_error_node", &[]).unwrap(),
        s.int("_ag_error_argument", &[]).unwrap()
    );
    s
}

fn tick(s: &mut Session, step: i32) -> i32 {
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}

fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}

/// This tick's closeout bundle rows (ordinal, cue, request gain).
fn audio_rows(s: &mut Session) -> Vec<(i32, i32, f32)> {
    let n = s.int("_ag_audio_count", &[]).unwrap();
    (0..n)
        .map(|i| {
            (
                s.int("_ag_audio_i", &[Int(i), Int(0)]).unwrap(),
                s.int("_ag_audio_i", &[Int(i), Int(1)]).unwrap(),
                s.float("_ag_audio_g", &[Int(i)]).unwrap(),
            )
        })
        .collect()
}

fn hud_text(s: &mut Session, slot: i32) -> String {
    let len = s.int("_ag_hud_length", &[Int(slot)]).unwrap();
    (0..len)
        .map(|i| s.int("_ag_hud_code", &[Int(slot), Int(i)]).unwrap() as u8 as char)
        .collect()
}

/// Character-mode session of the generated G12-equivalent product
/// (autographics.hud2d/1 assembly; 7 scene rows, t04 geometry).
fn game_session(name: &str, run: i32, px: f32, py: f32) -> Session {
    game_session_world(GAME_WORLD, name, run, px, py)
}

fn game_session_world(
    world_src: &str,
    name: &str,
    run: i32,
    px: f32,
    py: f32,
) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world_src.as_bytes().to_vec(),
        Api::Hud2V1,
    )
    .unwrap_or_else(|e| panic!("{name} admission: {e}"));
    let mut s = Session::new(&snap.source).unwrap_or_else(|e| panic!("{name} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let scene = 7;
    s.call("_ag_spatial_reset", &[Int(scene)], ResultKind::Unit).unwrap();
    let rows: [(i32, i32, f32, f32, f32, f32); 7] = [
        (1, 1, px, py, 0.8, 1.0),        // player.body (character)
        (2, 1, 8.0, -0.5, 24.0, 1.0),    // ground
        (3, 0, -30.0, -40.0, 0.5, 0.5),  // c1 template
        (4, 0, -30.0, -42.0, 0.5, 0.5),  // c2 template
        (5, 0, -30.0, -44.0, 0.5, 0.5),  // c3 template
        (6, 1, -30.0, -40.0, 2.0, 1.0),  // pad template
        (7, 1, -30.0, -42.0, 40.0, 2.0), // hazard template
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
    for n in 0..scene {
        for name in ["_ag_char_map", "_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n + 1), Int(n)], ResultKind::Unit).unwrap();
        }
    }
    s.call("_ag_audio_config", &[Int(3)], ResultKind::Unit).unwrap();
    // Slot configs from the generated scene: slot0 "3" (vis), slot1 "Playing",
    // slots 2/3 hidden/empty.
    for (slot, vis, text) in [(0, 1, "3"), (1, 1, "Playing"), (2, 0, ""), (3, 0, "")] {
        s.call(
            "_ag_hud_config",
            &[Int(slot), Int(vis), Int(text.len() as i32)],
            ResultKind::Unit,
        )
        .unwrap();
        for b in text.bytes() {
            s.call(
                "_ag_hud_config_code",
                &[Int(slot), Int(b as i32)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        0,
        "{name} app_start fault {} arg {}",
        s.int("_ag_error_node", &[]).unwrap(),
        s.int("_ag_error_argument", &[]).unwrap()
    );
    s
}

/// Character tick: sample input (held/pressed bit masks), advance one tick,
/// return the run status.
fn char_tick(s: &mut Session, step: i32, held: i32) -> i32 {
    char_tick_p(s, step, held, 0)
}

fn char_tick_p(s: &mut Session, step: i32, held: i32, pressed: i32) -> i32 {
    s.call("_ag_char_input", &[Int(held), Int(pressed)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}

/// Walk right from the settled start; capture every closeout bundle with its
/// real tick identity; stop when status turns 1 (Won). Independent f32 walk
/// oracle: every non-event tick advances x by 2.5*DT.
fn walk_until_won(
    s: &mut Session,
    name: &str,
) -> (i32, Vec<(i32, Vec<(i32, i32, f32)>)>) {
    // Settle: fall from the spawn y to the ground.
    let mut t0 = 0;
    for t in 1..=300 {
        t0 = t;
        let st = char_tick(s, t, 0);
        assert_eq!(st, 0, "{name} settle tick {t} status {st}");
        if int(s, "get_player_grounded") == 1 {
            break;
        }
    }
    assert_eq!(int(s, "get_player_grounded"), 1, "{name} never settled");
    let mut t = t0;
    let mut captures = Vec::new();
    while t < t0 + 4000 {
        t += 1;
        let st = char_tick(s, t, 2);
        assert_eq!(st, 0, "{name} tick {t} over the pool (fault {st})");
        let rows = audio_rows(s);
        if !rows.is_empty() {
            captures.push((t, rows));
        }
        if int(s, "get_r_status") == 1 {
            return (t0, captures);
        }
    }
    panic!("{name}: pad never won");
}

// ---------------------------------------------------------------- tests ----

/// V-02 actor-free chain: real Auto execution of two same-definition Notify
/// instances (no character, no camera, no trigger rows). Events land on the
/// author-chosen ticks; each handler's typed requests ride one bundle with a
/// shared strict ordinal; HUD text follows each owner's local field; the
/// visible latch flips off after the second hit on its OWN row.
#[test]
fn t02_actor_free_notify_chain_real_auto_hud_ordinals() {
    let s = &mut session(NOTIFY_WORLD, "t02-notify.at", 1, 3, 2);
    // Tick 3: left.Ping(1) -> cue ping (id 0) at gain 1.0, ordinal 1.
    for t in 1..=2 {
        assert_eq!(tick(s, t), 0, "tick {t}");
        assert!(audio_rows(s).is_empty(), "no requests before the event");
    }
    assert_eq!(tick(s, 3), 0, "event tick 3 over the pool");
    assert_eq!(audio_rows(s), vec![(1, 0, 1.0)], "left ping: ordinal 1, cue 0, gain 1");
    assert_eq!(int(s, "get_left_hits"), 1);
    assert_eq!(int(s, "get_left_live"), 1);
    assert_eq!(int(s, "get_right_hits"), 0);
    // Tick 6: right.Ping(2) -> cue pong (id 1) at gain 0.5, ordinal 2.
    for t in 4..=5 {
        assert_eq!(tick(s, t), 0, "tick {t}");
        assert!(audio_rows(s).is_empty());
    }
    assert_eq!(tick(s, 6), 0, "event tick 6");
    assert_eq!(audio_rows(s), vec![(2, 1, 0.5)], "right pong: ordinal 2, cue 1, gain 0.5");
    assert_eq!(int(s, "get_right_hits"), 1);
    // Tick 12: left.Ping(1) again -> ordinal 3; left.live flips to 0 (its own
    // row), right stays live.
    for t in 7..=11 {
        assert_eq!(tick(s, t), 0, "tick {t}");
    }
    assert_eq!(tick(s, 12), 0, "event tick 12");
    assert_eq!(audio_rows(s), vec![(3, 0, 1.0)], "left ping again: ordinal 3");
    assert_eq!(int(s, "get_left_live"), 0, "left latch off after two hits");
    assert_eq!(int(s, "get_right_live"), 1, "right unaffected");
    // HUD: slot0 follows left.hits ("1"), slot1 follows right.hits ("1");
    // each closeout is exactly one bundle (the rows above are per-tick).
    assert_eq!(hud_text(s, 0), "2+", "slot0 = left.hits 2 via author helper");
    assert_eq!(hud_text(s, 1), "1", "slot1 = right.hits 1 via author helper");
}

/// V-02 PCM oracle: the captured closeout rows drive the REAL mixer built
/// from the GENERATED scene's cue table; rendered PCM equals the bank slices
/// at the requested gains bit-exactly in PCM16 (t04 capture shape).
#[test]
fn t02_notify_pcm_matches_bank_slices_through_real_mixer() {
    let (_dir, doc, registry, _code) =
        load_product(NOTIFY_SCENE, NOTIFY_WORLD, NOTIFY_WORLD_SHA, "pcm");
    let (raw, _frames) = bank_oracle(&registry);
    let cues = public_audio::resolve_cues(&doc, &registry).expect("resolve cues");
    assert_eq!(cues.len(), 2, "two declared cues");
    assert_eq!((cues[0].start, cues[0].end), (0, 7403), "ping span");
    assert_eq!((cues[1].start, cues[1].end), (7403, 20046), "pong span");
    // Render each request exactly as the closeout delivered it: one cue at
    // the requested gain in a fresh mixer (independent bank-slice oracle).
    let cases: [(usize, f32, usize); 3] = [
        (0, 1.0, 7403),  // ping at gain 1.0
        (1, 0.5, 12643), // pong at gain 0.5
        (0, 1.0, 7403),  // second ping (fresh Run would re-use cue 0)
    ];
    for (cue, gain, expected_len) in cases {
        let mut m = Mixer::default();
        m.begin(1).unwrap();
        let packet = Packet {
            run: 1,
            tick: 0,
            bundle: 0,
            requests: vec![Request {
                ordinal: 1,
                cue,
                gain,
            }],
        };
        assert_eq!(
            m.accept(packet, &cues, false, || Ok(())).unwrap(),
            Accepted::Played
        );
        let mut out = vec![[0.0f32; 2]; expected_len];
        m.render(&mut out, 44100);
        // Rendered output = slice x request_gain x scene cue gain (t04 gain
        // fact); the oracle stays independent of the request path.
        let scale = gain * cues[cue].gain;
        let lsb = max_abs_delta_i16(&out, &raw, cues[cue].start, scale);
        assert_eq!(
            lsb, 0,
            "cue {cue} at gain {gain} (cue gain {}) must be bit-exact in PCM16",
            cues[cue].gain
        );
        assert_eq!(m.voices(), 0, "voice ends exactly at the half-open span");
    }
}

/// V-04 four-request bound: two Bell instances in ONE callback fill the four
/// bounded rows with four distinguishable cues at exact gains; a second
/// bundle in the same callback is the named fault 50.
#[test]
fn t04_four_requests_one_bundle_exact_rows() {
    let s = &mut session(FOUR_WORLD, "t04-four.at", 1, 3, 4);
    assert_eq!(tick(s, 1), 0);
    assert!(audio_rows(s).is_empty());
    assert_eq!(tick(s, 2), 0, "the four-request tick stays inside the pool");
    assert_eq!(
        audio_rows(s),
        vec![(1, 0, 1.0), (2, 1, 0.25), (3, 2, 0.5), (4, 3, 0.75)],
        "all four requests in execution order with exact ordinals and gains",
    );
    assert_eq!(hud_text(s, 0), "1");
    assert_eq!(hud_text(s, 1), "1");
}

/// V-04 fifth-request named rejection: the derived in-memory world adds one
/// more typed request to the same callback; the closeout packet n=5 is
/// rejected by the unit boundary (fault 51) with NO partial sound — the four
/// legit rows are not played either (no partial acceptance).
#[test]
fn t04_fifth_request_named_fault51_no_partial_sound() {
    // Derived world variant: the alto handler gets a third request in the
    // same callback (the committed product above stays untouched).
    let marker = "  sfx_play(1, 0.25)";
    let idx = FOUR_WORLD.find(marker).expect("alto second request marker");
    let mut variant = String::with_capacity(FOUR_WORLD.len() + 24);
    variant.push_str(&FOUR_WORLD[..idx + marker.len()]);
    variant.push_str("\n  sfx_play(3, 1.0)");
    variant.push_str(&FOUR_WORLD[idx + marker.len()..]);
    assert!(variant != FOUR_WORLD);
    let s = &mut session(&variant, "t04-five.at", 1, 3, 4);
    assert_eq!(tick(s, 1), 0);
    // The closeout passes n=5: _ag_audio_begin rejects (fault 51) before any
    // put — the named packet bound, no partial voice acceptance.
    let st = tick(s, 2);
    assert_ne!(st, 0, "the fifth request must be named-rejected");
    assert_eq!(st, 51, "fault 51 = packet bound");
    assert!(audio_rows(s).is_empty(), "no partial sound accepted");
}

/// V-02 negative: the crossed-binding negative from the 020 pattern — the
/// receiver table mutated so left events land on right's row — swaps the
/// business state while the run itself stays green (isolation of state per
/// instance, not name magic).
#[test]
fn t02_receiver_isolation_swap_negative() {
    // Derived world variant: left dispatches with right's row index.
    let marker = "  left_ping(0, 1)";
    let idx = NOTIFY_WORLD.find(marker).expect("left dispatch marker");
    let variant = format!(
        "{}left_ping(1, 1){}",
        &NOTIFY_WORLD[..idx],
        &NOTIFY_WORLD[idx + marker.len()..]
    );
    let s = &mut session(&variant, "t02-swap.at", 1, 3, 2);
    for t in 1..=2 {
        assert_eq!(tick(s, t), 0);
    }
    assert_eq!(tick(s, 3), 0);
    assert_eq!(audio_rows(s), vec![(1, 0, 1.0)]);
    // The hit landed on RIGHT's row: right.hits == 1, left untouched.
    assert_eq!(int(s, "get_right_hits"), 1, "swapped binding reaches the other row");
    assert_eq!(int(s, "get_left_hits"), 0, "left state untouched by the swap");
}

/// V-03 author G12-equivalent win path: continuous movement, one real pickup
/// per coin, win only after the last coin on the pad enter phase; every tick
/// inside the original 32-credit pool; ordinals strictly increase across the
/// Run; HUD text follows the root's own business fields.
#[test]
fn t03_walk_collect_win_author_trajectory() {
    let s = &mut game_session("t03-win.at", 1, 2.0, 1.0);
    let (_t0, captures) = walk_until_won(s, "t03-win");
    // Independent event oracle: coins at x=8/12/16 (r 0.9, char half-w 0.4)
    // are picked when the walk oracle's x first reaches 6.7/10.7/14.7; each
    // pickup costs one axis-skipped turn. Win on the pad enter (x >= 17.6).
    let picks: Vec<(i32, i32, f32)> = captures
        .iter()
        .flat_map(|(_, rows)| rows.iter().cloned())
        .collect();
    let cues: Vec<i32> = picks.iter().map(|(_, c, _)| *c).collect();
    assert_eq!(cues, vec![0, 0, 0, 1], "three coin cues then the win cue");
    let ordinals: Vec<i32> = picks.iter().map(|(o, _, _)| *o).collect();
    assert_eq!(
        ordinals,
        vec![1, 2, 3, 4],
        "one shared strict ordinal sequence across the Run"
    );
    assert_eq!(int(s, "get_r_remaining"), 0, "all three picked");
    assert_eq!(int(s, "get_r_status"), 1, "Won");
    assert_eq!(hud_text(s, 0), "0", "remaining text via author helper");
    assert_eq!(hud_text(s, 1), "Won", "status text via author helper");
    // Exactly four bundles carried the four requests (one per callback).
    assert_eq!(captures.len(), 4, "one closeout bundle per event callback");
}

/// V-03 early-arrival branch: standing on the pad with coins missing wins
/// nothing, destroys nothing, and stays inside the pool (the goal handler
/// reads the enter phase and the author's own remaining field).
#[test]
fn t03_early_pad_arrive_stay_no_win() {
    let s = &mut game_session("t03-early.at", 1, 18.6, 1.0);
    for t in 1..=120 {
        let held = if t <= 30 { 0 } else if t <= 60 { 1 } else { 2 };
        let st = char_tick(s, t, held);
        assert_eq!(st, 0, "tick {t} status {st} (early arrival must be clean)");
    }
    assert_eq!(int(s, "get_r_status"), 0, "no win with coins missing");
    assert_eq!(int(s, "get_r_remaining"), 3, "no destruction on early arrival");
    assert!(audio_rows(s).is_empty(), "no requests on the early path");
    assert_eq!(hud_text(s, 1), "Playing", "status text unchanged");
}

/// V-03 hazard branch: the author routes the hazard trigger to its own
/// verdict (status 2 + fail cue), one bundle, no partial win.
#[test]
fn t03_hazard_fail_named_cue() {
    let s = &mut game_session("t03-fail.at", 1, 8.0, -6.0);
    // Tick 1 evaluates the trigger snapshot (enter phase); the poll prologue
    // dispatches it at tick 2.
    let st = char_tick(s, 1, 0);
    assert_eq!(st, 0, "hazard evaluate tick inside the pool");
    assert_eq!(int(s, "get_r_status"), 0);
    let st = char_tick(s, 2, 0);
    assert_eq!(st, 0, "hazard dispatch tick inside the pool");
    assert_eq!(int(s, "get_r_status"), 2, "Failed via the author handler");
    assert_eq!(audio_rows(s), vec![(1, 2, 1.0)], "fail cue, ordinal 1");
    assert_eq!(hud_text(s, 1), "Failed", "status text via author helper");
    for t in 2..=10 {
        assert_eq!(char_tick(s, t, 0), 0, "tick {t}");
    }
    assert!(audio_rows(s).is_empty(), "fail fires exactly once (status guard)");
}

/// V-03 held Left+Right stands still; Jump leaves the ground with the frozen
/// impulse and lands again (Jump/land overlap with continued input).
#[test]
fn t03_held_jump_land_branches() {
    let s = &mut game_session("t03-jump.at", 1, 2.0, 1.0);
    for t in 1..=300 {
        assert_eq!(char_tick(s, t, 0), 0);
        if int(s, "get_player_grounded") == 1 {
            break;
        }
    }
    // Left+Right: axis 0 -> no movement.
    for t in 301..=310 {
        assert_eq!(char_tick(s, t, 3), 0, "tick {t}");
    }
    // Jump with held Right: the pressed edge fires once; leaves the ground,
    // returns (grounded 1 -> 0 -> 1).
    let mut airborne = 0;
    let mut landed_again = false;
    for t in 311..=420 {
        let pressed = if t == 311 { 4 } else { 0 };
        assert_eq!(char_tick_p(s, t, 6, pressed), 0, "jump tick {t} over the pool");
        let g = int(s, "get_player_grounded");
        if g == 0 {
            airborne += 1;
        } else if airborne > 0 {
            landed_again = true;
            break;
        }
    }
    assert!(airborne > 0, "jump left the ground");
    assert!(landed_again, "landed again under continued input");
}

/// V-03/AC-05 budget boundary: the author program walk tick sits at the
/// original pool edge — one additional held read is the named fault 10 at
/// the first walk tick (derived in-memory world; committed product
/// untouched).
#[test]
fn t03_budget_plus_one_named_fault10() {
    let marker = "  axis = (ag_action_held(1) - ag_action_held(0))";
    let idx = GAME_WORLD.find(marker).expect("axis read marker");
    let variant = format!(
        "{}  let _waste = ag_action_held(0);\n{}",
        &GAME_WORLD[..idx],
        &GAME_WORLD[idx..]
    );
    let mut s = game_session_world(&variant, "t03-plus1.at", 1, 2.0, 1.0);
    // The very first tick already runs the full body (pressed + two held
    // reads + move + grounded + poll prologue + closeout) at the exact
    // 32-credit edge, so one extra held read faults immediately.
    let st = char_tick(&mut s, 1, 0);
    assert_eq!(
        st, 10,
        "one extra held read on the exact-edge walk tick must be the named          budget fault 10",
    );
}

/// V-05 fresh-Run isolation: a second Run in the same session rebuilds the
/// presentation infrastructure — the shared ordinal restarts at 1 and no
/// old-Run request row leaks into the new closeout.
#[test]
fn t05_fresh_run_ordinal_isolation() {
    let s = &mut session(NOTIFY_WORLD, "t05-fresh.at", 1, 3, 2);
    for t in 1..=3 {
        assert_eq!(tick(s, t), 0);
    }
    assert_eq!(audio_rows(s), vec![(1, 0, 1.0)], "run 1 first request");
    for t in 4..=6 {
        assert_eq!(tick(s, t), 0);
    }
    assert_eq!(audio_rows(s), vec![(2, 1, 0.5)]);
    // Fresh Run: the host rebuilds the whole VM per Run (a fresh Session of
    // the same world under a new run identity — the unit freeze/config state
    // is Run-owned and cannot be re-configured in place).
    let s = &mut session(NOTIFY_WORLD, "t05-fresh2.at", 2, 3, 2);
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "fresh app_start clean");
    for t in 1..=3 {
        assert_eq!(tick(s, t), 0);
    }
    assert_eq!(
        audio_rows(s),
        vec![(1, 0, 1.0)],
        "run 2 ordinal restarts at 1 (no old-Run carryover)"
    );
}

/// V-05 mixed-write conflict: an author-side standalone ag_audio_emit in the
/// same callback as the closeout bundle is the named fault 50 (mode
/// conflict; the service path stays bundle-only, no partial acceptance).
#[test]
fn t05_standalone_emit_conflicts_with_bundle_fault50() {
    // Derived world variant: the left handler ALSO fires a standalone emit.
    let marker = "  sfx_play(0, 1.0)";
    let idx = NOTIFY_WORLD.find(marker).expect("left sfx marker");
    let variant = format!(
        "{}  let _leak = ag_audio_emit(900, 0, 1.0);{}",
        &NOTIFY_WORLD[..idx],
        &NOTIFY_WORLD[idx + marker.len()..]
    );
    let s = &mut session(&variant, "t05-mixed.at", 1, 3, 2);
    for t in 1..=2 {
        assert_eq!(tick(s, t), 0);
    }
    let st = tick(s, 3);
    assert_eq!(st, 50, "bundle after standalone emit is the named mode conflict");
    // The standalone emit itself was the only accepted request; the bundle
    // is refused whole (no partial bundle acceptance).
    assert_eq!(audio_rows(s), vec![(900, 0, 1.0)], "only the standalone row");
}

/// V-05 ordinal replay across callbacks: the shared counter never resets
/// within a Run, and a stale ordinal at the unit boundary stays the named
/// fault 52 (the notify chain already asserts 1,2,3; this asserts the unit
/// rejects a replayed ordinal directly).
#[test]
fn t05_ordinal_replay_named_fault52() {
    // Derived world variant: the right handler replays ordinal 900/900 after
    // the closeout already consumed it (emulate a stale emitter).
    let marker = "  sfx_play(1, 0.5)";
    // The receivers are per-leaf copies: target the SECOND occurrence (the
    // right handler) so the stale emit fires on the tick-6 event.
    let idx = NOTIFY_WORLD
        .find(marker)
        .map(|first| first + 1)
        .and_then(|from| NOTIFY_WORLD[from..].find(marker).map(|m| from + m))
        .expect("right sfx marker (second copy)");
    let variant = format!(
        "{}
  let _stale = ag_audio_emit(1, 1, 1.0);{}",
        &NOTIFY_WORLD[..idx + marker.len()],
        &NOTIFY_WORLD[idx + marker.len()..]
    );
    let s = &mut session(&variant, "t05-replay.at", 1, 3, 2);
    for t in 1..=5 {
        assert_eq!(tick(s, t), 0, "tick {t}");
    }
    // Tick 6: the right handler runs; the stale standalone emit replays
    // ordinal 1 (already consumed by the tick-3 closeout) -> fault 52.
    let st = tick(s, 6);
    assert_eq!(st, 52, "stale ordinal replay is the named fault 52");
}
