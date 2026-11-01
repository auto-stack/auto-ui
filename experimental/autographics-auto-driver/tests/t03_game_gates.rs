//! auto-ui PLAN-023 T-03 game gates: the components-2d-game generated world
//! (experimental/autographics-node2d-pilot/generated-components2d-game/game,
//! the T-02 hand-proven collect-the-coins game now GENERATOR-produced from
//! the game2d authoring surface) admits as autographics.hud2d/1 and holds
//! the session contract.
//! F-9 oracles in this file: V-01 positive (the committed generated world is
//! byte-pinned: world sha ed98b0b5..., scene document pinned) and V-02
//! (admission green with the 5 trigger binds of the authoring table; the
//! 8-slot envelopes at the game's natural occupancy — 5/8 object spawn
//! slots, spawned stateful instances inside the 8-instance envelope; and
//! the beyond-pool spawn control raising the named fault 31).
//! NOTE on F-9 wording: the freeze text says "33rd-object-slot fault 31";
//! the frozen objects pool is slots 0..7 (api/runtime_objects_v1.at lines
//! 76-77 reject slot>7 with fault 31), so the realizable control is the
//! 9TH slot — recorded here and in the T-03 notes as the F-9 delta.
//! V-01's three lowering-rule negatives live pilot-side
//! (experimental/autographics-node2d-pilot/tests/node_components_gates.rs).
//! CPU-session convention: spatial boxes are fed as translation-RELATIVE
//! half-extents (host Descriptor rows, spatial.rs) — see the lowering
//! module docs (F-8.3).
//! Run single-threaded:
//! cargo test --locked --offline --test t03_game_gates -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d-game/game/scene.agscene");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d-game/game/world.at");
const IDENTITY: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d-game/game/identity.json");

/// V-01 pin: the generator output committed with T-03 (regeneration is
/// proven byte-identical pilot-side; here the frozen product is pinned).
const WORLD_SHA: &str = "ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0";

const DT: f32 = 1.0 / 60.0;

fn sha256_hex(bytes: &[u8]) -> String {
    agp::renderer::capture::sha256_hex(bytes)
}

/// Scene rows parsed from the committed scene document:
/// (id, kind(circle=0/quad=1), tx, ty, sx, sy).
fn scene_rows() -> Vec<(i32, i32, f32, f32, f32, f32)> {
    let objects = SCENE
        .split("\"objects\": [")
        .nth(1)
        .expect("objects list")
        .split("\n ],")
        .next()
        .expect("objects end");
    let mut rows = Vec::new();
    for chunk in objects.split("\"id\": ").skip(1) {
        let num = |key: &str| -> f64 {
            let ki = chunk.find(key).expect(key);
            let rest = &chunk[ki + key.len()..];
            let rest = rest.trim_start_matches(|c: char| c == '[' || c == ' ');
            let end = rest
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest.len());
            rest[..end].parse().expect("number")
        };
        let id: i32 = chunk[..chunk.find(',').expect("id comma")]
            .trim()
            .parse()
            .expect("id");
        let kind = if chunk.contains("\"kind\": \"quad\"") {
            1
        } else if chunk.contains("\"kind\": \"circle\"") {
            0
        } else {
            panic!("unexpected scene kind: {}", &chunk[..60.min(chunk.len())]);
        };
        let tx = num("\"translation\": [") as f32;
        let ty = {
            let ki = chunk.find("\"translation\": [").expect("t");
            let rest = &chunk[ki + "\"translation\": [".len()..];
            let comma = rest.find(',').expect("comma");
            let rest2 = rest[comma + 1..].trim_start();
            let end = rest2
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest2.len());
            rest2[..end].parse::<f64>().expect("ty") as f32
        };
        let sx = num("\"scale\": [") as f32;
        let sy = {
            let ki = chunk.find("\"scale\": [").expect("s");
            let rest = &chunk[ki + "\"scale\": [".len()..];
            let comma = rest.find(',').expect("comma");
            let rest2 = rest[comma + 1..].trim_start();
            let end = rest2
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest2.len());
            rest2[..end].parse::<f64>().expect("sy") as f32
        };
        rows.push((id, kind, tx, ty, sx, sy));
    }
    rows
}

/// CPU session over the generated game world (Hud2V1 admission, the T-02
/// probe harness shape; spatial boxes translation-relative, F-8.3).
fn session(world_src: &str, name: &str, run: i32) -> Session {
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
    let rows = scene_rows();
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
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
    for n in 0..rows.len() {
        for name in ["_ag_char_map", "_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n as i32 + 1), Int(n as i32)], ResultKind::Unit)
                .unwrap();
        }
    }
    s.call("_ag_audio_config", &[Int(3)], ResultKind::Unit).unwrap();
    for slot in 0..4 {
        s.call(
            "_ag_hud_config",
            &[Int(slot), Int(1), Int(0)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}

fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}

fn tick(s: &mut Session, step: i32, held: i32, pressed: i32) -> i32 {
    s.call("_ag_char_input", &[Int(held), Int(pressed)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}

// ------------------------------------------------------------- V-01 ----

/// V-01 positive: the committed game product is byte-pinned — world sha,
/// identity api, scene schema/assets/cues/slots — and the world text
/// carries the frozen closeout skeleton (generator, not hand wiring).
#[test]
fn v01_game_product_is_byte_pinned_and_hud2d() {
    assert_eq!(
        sha256_hex(WORLD.as_bytes()),
        WORLD_SHA,
        "committed world.at must equal the pinned T-03 generation"
    );
    assert!(
        IDENTITY.contains("\"api\": \"autographics.hud2d/1\""),
        "identity api is hud2d/1"
    );
    assert!(
        IDENTITY.contains("\"world_sha256\": \"ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0\""),
        "identity provenance pins the same world sha"
    );
    assert!(SCENE.contains("\"schema\": \"autographics.scene/8\""));
    assert!(SCENE.contains("\"api\": \"autographics.hud2d/1\""));
    // scene assets: bank (frozen G12 sha) + font (frozen Lilita sha).
    assert!(SCENE.contains("b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d"));
    assert!(SCENE.contains("4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa"));
    // cue table (cue id = array index, order coin/win/fail).
    let cues = SCENE.split("\"cues\": [").nth(1).expect("cues");
    let coin = cues.find("\"name\": \"coin\"").expect("coin cue");
    let win = cues.find("\"name\": \"win\"").expect("win cue");
    let fail = cues.find("\"name\": \"fail\"").expect("fail cue");
    assert!(coin < win && win < fail, "cue ids follow declaration order");
    // four complete hud slots.
    assert_eq!(SCENE.matches("\"slot\": ").count(), 4, "four hud slots");
    // world carries the generated closeout skeleton and F-8.2 receivers
    // without the T-02 forced dispatch site.
    for marker in [
        "fn game_cue(c int) {",
        "fn game_events() {",
        "fn hud_count_text(n int) str {",
        "fn hud_state_text(n int) str {",
        "fn game_closeout() {",
        " game_closeout()",
        " game_events()",
        "var zone_visq=[1,0]",
        "var zone_vis=[1,0]",
    ] {
        assert!(WORLD.contains(marker), "world must contain `{marker}`");
    }
    assert!(
        !WORLD.contains("r_state == 9"),
        "the T-02 forced dispatch site must be gone (F-8.2 receiver completion)"
    );
}

// ------------------------------------------------------------- V-02 ----

/// V-02 case 1: Hud2V1 admission green; app_start carries exactly the 5
/// authoring-table trigger binds (pairs 0..4, subject = character id 1,
/// sources = instance handles); the first walk ticks stay inside the pool
/// (the session contract holds on the generated world).
#[test]
fn v02_admission_green_with_five_binds_and_walk() {
    // static: 5 binds, pair = table order, subject id 1.
    let binds: Vec<&str> = WORLD
        .lines()
        .filter(|l| l.contains("ag_trigger_bind("))
        .collect();
    assert_eq!(binds.len(), 5, "5 authoring trigger binds: {binds:?}");
    for (pair, line) in binds.iter().enumerate() {
        let marker = format!("ag_trigger_bind({},1,1,", pair);
        assert!(
            line.contains(&marker),
            "bind {pair} keeps table order + character subject: {line}"
        );
    }
    // dynamic: admission + app_start green; settle + walk ticks green.
    let mut s = session(WORLD, "t03-admission.at", 1);
    assert_eq!(status(&mut s), 0, "app_start green (5 binds admitted)");
    let mut grounded = false;
    for t in 1..=300 {
        assert_eq!(tick(&mut s, t, 0, 0), 0, "settle tick {t} within pool");
        if s.int("get_player_grounded", &[]).unwrap() == 1 {
            grounded = true;
            break;
        }
    }
    assert!(grounded, "character settles");
    for t in 300..306 {
        assert_eq!(tick(&mut s, t, 2, 0), 0, "walk tick {t} within pool");
    }
    assert_eq!(s.int("get_r_count", &[]).unwrap(), 0, "no coin yet");
    assert_eq!(s.int("get_r_state", &[]).unwrap(), 0, "playing");
}

/// V-02 case 2: the envelopes hold at the game's natural occupancy — 5/8
/// object spawn slots (3 coins + 2 zones; slot args 0..=4), 5 spawned
/// stateful instances + player + root inside the 8-instance envelope, and
/// the scene stays within the 128-node bound (7 rows).
#[test]
fn v02_envelope_5_of_8_object_slots_and_stateful() {
    let spawns: Vec<&str> = WORLD
        .lines()
        .filter(|l| l.contains("ag_object_spawn("))
        .collect();
    assert_eq!(spawns.len(), 5, "five spawned regions/coins: {spawns:?}");
    for (slot, line) in spawns.iter().enumerate() {
        let marker = format!("ag_object_spawn({slot},");
        assert!(line.contains(&marker), "spawn slot {slot} in order: {line}");
    }
    assert!(
        !WORLD.contains("ag_object_spawn(5,"),
        "no sixth object slot (5/8 occupancy)"
    );
    // stateful envelope: 6 instances + root = 7 <= 8 (the header line lists
    // the declaration pre-order instances).
    let head = WORLD
        .lines()
        .find(|l| l.starts_with("// Root:"))
        .expect("instances header");
    let inst = head.split("): ").nth(1).expect("instance list");
    let n = inst.split(',').filter(|s| !s.trim().is_empty()).count();
    assert_eq!(n, 6, "six component instances (c1,c2,c3,player,pad,hazard)");
    assert!(n + 1 <= 8, "stateful envelope 7/8");
    // scene row bound.
    assert_eq!(scene_rows().len(), 7, "char + ground + 5 templates");
}

/// V-02 case 3: beyond-pool spawn control — a 9th object slot (slot arg 8;
/// the frozen pool is 0..7 per api/runtime_objects_v1.at, so this is the
/// realizable form of the F-9 "33rd" control) raises the named fault 31 at
/// app_start, while the unmodified world starts green in the same session.
#[test]
fn v02_beyond_pool_spawn_faults_31() {
    let bad = WORLD.replace(
        "coin_handle[2]=ag_object_spawn(2,5,coin_cx[2],coin_cy[2],0.0)",
        "coin_handle[2]=ag_object_spawn(8,5,coin_cx[2],coin_cy[2],0.0)",
    );
    assert_ne!(bad, WORLD, "control variant differs");
    let mut s = session(&bad, "t03-slot8.at", 1);
    assert_eq!(status(&mut s), 31, "slot 8 spawn faults named 31 at start");
    assert_eq!(s.int("_ag_error_node", &[]).unwrap(), -1);
    assert_eq!(s.int("_ag_error_argument", &[]).unwrap(), 8, "fault arg = slot");
    // control: the frozen world starts green in the same harness.
    let mut ok = session(WORLD, "t03-slotok.at", 1);
    assert_eq!(status(&mut ok), 0, "frozen world starts green");
}
