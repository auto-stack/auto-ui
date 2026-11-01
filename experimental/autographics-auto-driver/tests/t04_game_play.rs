//! auto-ui PLAN-023 T-04 V-03 CPU-session playthrough oracles on the
//! GENERATED components-2d-game world
//! (experimental/autographics-node2d-pilot/generated-components2d-game/game,
//! world sha ed98b0b5... — the T-03 generator product, not the T-02
//! hand-extended probe world). Freeze F-9 V-03: six blocks, each with
//! exact-value assertions (counts, states, ordinals, audio rows, HUD text):
//!   a. settle/walk/collect x3/win playthrough,
//!   b. fail branch (hazard) latches Failed at the right tick,
//!   c. once-only (dead-spot stand + re-crossings: no repeat cue),
//!   d. two cues in ONE callback bundle, strictly increasing ordinals,
//!   e. ordinal sequence integrity across a full run (1..5, no reuse),
//!   f. restart-shape smoke (fresh Run rebuilds the model, ordinals
//!      restart per Run, no old-Run leakage).
//! Plus the T-04 behavior-equivalence trace: the SAME input script driven
//! through the generated world and through the T-02 hand-proven reference
//! world (committed byte-identical at tests/fixtures/t04/
//! t02-reference-world.at, LF-normalized sha 52f60972...) must produce the
//! identical observable sequence at the identical tick indices.
//! Idle-turn gate observability (F-6): `get_r_gate` exists and reads 0
//! after every tick — the poll sets it on a latch tick and the same turn's
//! root dispatch consumes it (main_lifetick: game_events -> Idle(1) ->
//! r_gate=0), so the post-tick 0 IS the gate's observable; the positional
//! halt effect and the 32-credit latch ticks are the GPU/V-05 side.
//! CPU-session convention: spatial boxes are fed as translation-RELATIVE
//! half-extents (host Descriptor rows, spatial.rs — F-8.3).
//! Run single-threaded:
//! cargo test --locked --offline --test t04_game_play -- --test-threads=1
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
/// The T-02 hand-proven reference world (scratch t02/gameplay/world-src/
/// world-game.at, copied byte-identical; the probe itself stays untracked).
const REF_WORLD: &str = include_str!("fixtures/t04/t02-reference-world.at");

/// T-03 pin: the generated world under test is the committed product.
const WORLD_SHA: &str = "ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0";
const REF_WORLD_SHA: &str = "52f60972ddb0220093326347ecd5d0b84e8c87566cbe6a0a34afe4e4f91b70e6";

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

/// CPU session over a game world (Hud2V1 admission, the T-02/T-03 harness
/// shape; spatial boxes translation-relative, F-8.3).
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
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        0,
        "{name} app_start fault {} arg {}",
        s.int("_ag_error_node", &[]).unwrap(),
        s.int("_ag_error_argument", &[]).unwrap()
    );
    s
}

/// One input tick: character input + `_ag_tick`; returns the status.
fn tick(s: &mut Session, step: i32, held: i32, pressed: i32) -> i32 {
    s.call("_ag_char_input", &[Int(held), Int(pressed)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}

fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}

/// HUD slot text through the session glyph-code getters.
fn hud_text(s: &mut Session, slot: i32) -> String {
    let len = s.int("_ag_hud_length", &[Int(slot)]).unwrap();
    (0..len)
        .map(|i| s.int("_ag_hud_code", &[Int(slot), Int(i)]).unwrap() as u8 as char)
        .collect()
}

/// This tick's pending audio bundle rows (ordinal, cue) as the closeout's
/// ag_present2d delivered them (cleared at the next tick begin).
fn audio(s: &mut Session) -> Vec<(i32, i32)> {
    let n = s.int("_ag_audio_count", &[]).unwrap();
    (0..n)
        .map(|i| {
            (
                s.int("_ag_audio_i", &[Int(i), Int(0)]).unwrap(),
                s.int("_ag_audio_i", &[Int(i), Int(1)]).unwrap(),
            )
        })
        .collect()
}

/// Settle the character onto the ground (state-based) and return the tick.
fn settle(s: &mut Session, name: &str) -> i32 {
    for t in 1..=300 {
        let st = tick(s, t, 0, 0);
        assert_eq!(st, 0, "{name} settle tick {t} status {st}");
        if int(s, "get_player_grounded") == 1 {
            return t;
        }
    }
    panic!("{name}: never settled");
}

/// One observed event of a playthrough trace.
#[derive(Clone, PartialEq, Debug)]
struct Ev {
    tick: i32,
    count: i32,
    state: i32,
    ord: i32,
    np: i32,
    rows: Vec<(i32, i32)>,
    hud0: String,
    hud1: String,
}

#[derive(Clone, PartialEq, Debug)]
struct Trace {
    settle: i32,
    events: Vec<Ev>,
    win: i32,
    end: i32,
}

/// Drive the adopted input script on one world source: settle (held=0),
/// then walk right (held=2) every tick until the state latches Won, then
/// 10 more walk ticks. Every tick where the closeout carried audio rows or
/// count/state moved is recorded with its full observable row.
fn drive_win_trace(world_src: &str, name: &str) -> Trace {
    let mut s = session(world_src, name, 1);
    let t0 = settle(&mut s, name);
    let mut events = Vec::new();
    let mut t = t0;
    let mut win = 0;
    let mut last_c = 0;
    let mut last_st = 0;
    while t < t0 + 1200 {
        t += 1;
        let st = tick(&mut s, t, 2, 0);
        assert_eq!(st, 0, "{name} tick {t} over the pool");
        let (c, state, ord) = (
            int(&mut s, "get_r_count"),
            int(&mut s, "get_r_state"),
            int(&mut s, "get_r_ord"),
        );
        let rows = audio(&mut s);
        if state == 1 && win == 0 {
            win = t;
        }
        if !rows.is_empty() || c != last_c || state != last_st || state == 1 {
            events.push(Ev {
                tick: t,
                count: c,
                state,
                ord,
                np: int(&mut s, "get_r_np"),
                rows: rows.clone(),
                hud0: hud_text(&mut s, 0),
                hud1: hud_text(&mut s, 1),
            });
            last_c = c;
            last_st = state;
        }
        if win > 0 {
            break;
        }
    }
    assert!(win > 0, "{name}: pad never won");
    // 10 post-win walk ticks: green, latched, no further cues.
    let mut end = win;
    for _ in 0..10 {
        end += 1;
        let st = tick(&mut s, end, 2, 0);
        assert_eq!(st, 0, "{name} post-win tick {end}");
        assert!(audio(&mut s).is_empty(), "{name} post-win cue at {end}");
        assert_eq!(int(&mut s, "get_r_state"), 1);
    }
    Trace {
        settle: t0,
        events,
        win,
        end,
    }
}

// --------------------------------------------------------------- V-03 ----

/// V-03 (a): the full playthrough on the generated world — settle, walk
/// right, collect 1/2/3, win on the pad — with exact-value assertions at
/// every step: counts 0->1->2->3, state 0->1 only at the pad, ordinals
/// 1..5, the exact audio rows per the fixture mapping (cue 0 = coin,
/// cue 1 = win), HUD slot texts, once-state of every coin.
#[test]
fn v03_a_playthrough_settle_walk_collect3_win() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "generated world pinned");
    let mut s = session(WORLD, "t04-a.at", 1);
    let t0 = settle(&mut s, "t04-a");
    // HUD at rest: count 0, Playing; scalars at their initial values.
    assert_eq!(hud_text(&mut s, 0), "0");
    assert_eq!(hud_text(&mut s, 1), "Playing");
    assert_eq!(int(&mut s, "get_r_count"), 0);
    assert_eq!(int(&mut s, "get_r_state"), 0);
    assert_eq!(int(&mut s, "get_r_ord"), 0);
    assert_eq!(int(&mut s, "get_r_np"), 0);

    // Walk right until each coin, asserting the exact latch row.
    let mut t = t0;
    let mut marks = Vec::new();
    for (want_count, want_rows) in [(1, vec![(1, 0)]), (2, vec![(2, 0)]), (3, vec![(3, 0), (4, 1)])]
    {
        let mut hit = 0;
        while t < t0 + 400 {
            t += 1;
            let st = tick(&mut s, t, 2, 0);
            assert_eq!(st, 0, "tick {t} over the pool");
            if int(&mut s, "get_r_count") == want_count {
                hit = t;
                assert_eq!(audio(&mut s), want_rows, "count {want_count} cue rows");
                let want_ord = if want_count == 3 { 4 } else { want_count };
                assert_eq!(int(&mut s, "get_r_ord"), want_ord);
                assert_eq!(int(&mut s, "get_r_np"), 0, "closeout consumed the pendings");
                assert_eq!(int(&mut s, "get_r_state"), 0, "not won before the pad");
                assert_eq!(hud_text(&mut s, 0), want_count.to_string());
                assert_eq!(hud_text(&mut s, 1), "Playing");
                break;
            }
        }
        assert!(hit > 0, "coin {want_count} never collected");
        marks.push(hit);
    }
    let (t1, t2, t3) = (marks[0], marks[1], marks[2]);
    assert!(t1 < t2 && t2 < t3, "collect order along the walk: {marks:?}");
    assert_eq!(int(&mut s, "get_c1_alive"), 0, "coin 1 once-state");
    assert_eq!(int(&mut s, "get_c2_alive"), 0, "coin 2 once-state");
    assert_eq!(int(&mut s, "get_c3_alive"), 0, "coin 3 once-state");

    // Walk to the goal pad: the goal branch reads phase (count==3) and
    // latches Won with one fanfare cue; ord reaches 5.
    let mut twin = 0;
    while t < t0 + 1200 {
        t += 1;
        let st = tick(&mut s, t, 2, 0);
        assert_eq!(st, 0, "tick {t} over the pool");
        if int(&mut s, "get_r_state") == 1 {
            twin = t;
            break;
        }
    }
    assert!(twin > t3, "pad wins after the third coin");
    assert_eq!(audio(&mut s), vec![(5, 1)], "win fanfare ordinal 5 cue 1");
    assert_eq!(int(&mut s, "get_r_ord"), 5, "five cues total");
    assert_eq!(int(&mut s, "get_r_np"), 0);
    assert_eq!(hud_text(&mut s, 0), "3");
    assert_eq!(hud_text(&mut s, 1), "Won");
    // Post-win: green, latched, silent.
    for _ in 0..10 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "post-win tick {t}");
    }
    assert_eq!(int(&mut s, "get_r_state"), 1);
    assert_eq!(int(&mut s, "get_r_ord"), 5);
    println!(
        "V03_A generated playthrough: settle={t0} t1={t1} t2={t2} t3={t3} twin={twin} end={t}"
    );
}

/// V-03 (b): fail branch — walk left off the ground edge into the hazard
/// AABB: state latches Failed with the fail buzz (cue 2) as ordinal 1,
/// count stays 0, HUD reads "0"/"Failed", the latch happens once, and the
/// generated receiver indexing is observable (the hazard zone dies, the
/// pad stays alive — the T-02 hand world killed the wrong SoA index).
#[test]
fn v03_b_fail_branch_hazard_latches_failed() {
    let mut s = session(WORLD, "t04-b.at", 1);
    let t0 = settle(&mut s, "t04-b");
    let mut t = t0;
    let mut tfail = 0;
    while t < t0 + 500 {
        t += 1;
        let st = tick(&mut s, t, 1, 0);
        assert_eq!(st, 0, "tick {t} over the pool");
        if int(&mut s, "get_r_state") == 2 {
            tfail = t;
            break;
        }
    }
    assert!(tfail > 0, "never failed");
    assert!(tfail > t0 + 60, "failed only after leaving the ground edge");
    assert_eq!(int(&mut s, "get_r_count"), 0, "no coins on the fail path");
    assert_eq!(audio(&mut s), vec![(1, 2)], "fail buzz ordinal 1 cue 2");
    assert_eq!(int(&mut s, "get_r_ord"), 1);
    assert_eq!(int(&mut s, "get_r_np"), 0);
    assert_eq!(hud_text(&mut s, 0), "0");
    assert_eq!(hud_text(&mut s, 1), "Failed");
    assert_eq!(int(&mut s, "get_hazard_alive"), 0, "hazard consumed (index 1)");
    assert_eq!(int(&mut s, "get_pad_alive"), 1, "pad untouched by the fail");
    // Latches once: post-fail ticks stay green and silent, state stays 2.
    for _ in 0..20 {
        t += 1;
        assert_eq!(tick(&mut s, t, 1, 0), 0, "post-fail tick {t}");
        assert!(audio(&mut s).is_empty(), "post-fail cue at {t}");
    }
    assert_eq!(int(&mut s, "get_r_state"), 2, "fail latches once");
    assert_eq!(int(&mut s, "get_r_ord"), 1, "no further ordinals");
    println!("V03_B fail branch: settle={t0} tfail={tfail}");
}

/// V-03 (c): once-only — after collecting coin 1 the character STANDS
/// inside the dead trigger circle for 10 ticks, crosses it to the right and
/// walks back through it: no repeat event, no second cue, count/ordinals
/// unchanged; then coin 2 fires fresh.
#[test]
fn v03_c_once_only_dead_spot_and_recross() {
    let mut s = session(WORLD, "t04-c.at", 1);
    let t0 = settle(&mut s, "t04-c");
    let mut t = t0;
    let mut t1 = 0;
    while t < t0 + 400 {
        t += 1;
        let st = tick(&mut s, t, 2, 0);
        assert_eq!(st, 0, "tick {t} over the pool");
        if int(&mut s, "get_r_count") == 1 {
            t1 = t;
            break;
        }
    }
    assert!(t1 > 0, "coin 1 never collected");
    assert_eq!(audio(&mut s), vec![(1, 0)]);
    assert_eq!(int(&mut s, "get_c1_alive"), 0);
    // Stand INSIDE the dead circle (a live source would emit stay events
    // every tick): silence, count pinned, gate consumed.
    for _ in 0..10 {
        t += 1;
        assert_eq!(tick(&mut s, t, 0, 0), 0, "stand tick {t}");
        assert!(audio(&mut s).is_empty(), "repeat cue while standing at {t}");
        assert_eq!(int(&mut s, "get_r_np"), 0, "no pending event while standing");
        assert_eq!(int(&mut s, "get_r_count"), 1);
        assert_eq!(int(&mut s, "get_r_gate"), 0, "gate consumed every turn");
    }
    // Cross the dead spot right (10 ticks), then walk back through it
    // (25 ticks): the despawned coin never re-fires.
    for _ in 0..10 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "cross-right tick {t}");
        assert_eq!(int(&mut s, "get_r_count"), 1, "no double collect");
        assert!(audio(&mut s).is_empty(), "repeat cue crossing at {t}");
    }
    for _ in 0..25 {
        t += 1;
        assert_eq!(tick(&mut s, t, 1, 0), 0, "walk-back tick {t}");
        assert_eq!(int(&mut s, "get_r_np"), 0, "no events from the dead source");
        assert!(audio(&mut s).is_empty(), "repeat cue walking back at {t}");
    }
    assert_eq!(int(&mut s, "get_r_ord"), 1, "ordinal 1 not reused");
    // Walk right again: coin 2 fires fresh with ordinal 2.
    let mut t2 = 0;
    while t < t0 + 800 {
        t += 1;
        let st = tick(&mut s, t, 2, 0);
        assert_eq!(st, 0, "tick {t} over the pool");
        if int(&mut s, "get_r_count") == 2 {
            t2 = t;
            break;
        }
    }
    assert!(t2 > t1, "coin 2 after the recross");
    assert_eq!(audio(&mut s), vec![(2, 0)], "fresh cue ordinal 2 for coin 2");
    assert_eq!(int(&mut s, "get_c2_alive"), 0);
    println!("V03_C once-only: settle={t0} t1={t1} recross-clean t2={t2}");
}

/// V-03 (d): two cues in ONE callback — the third collect carries the coin
/// cue AND the set-complete win cue in a single bundle: rows (3,0),(4,1)
/// with strictly increasing ordinals 3 < 4, the pending bookkeeping kept
/// p0=0/p1=1, and the closeout consumed the pendings (np back to 0).
#[test]
fn v03_d_two_cues_one_bundle_strict_ordinals() {
    let mut s = session(WORLD, "t04-d.at", 1);
    let t0 = settle(&mut s, "t04-d");
    let mut t = t0;
    let mut t3 = 0;
    while t < t0 + 900 {
        t += 1;
        let st = tick(&mut s, t, 2, 0);
        assert_eq!(st, 0, "tick {t} over the pool");
        if int(&mut s, "get_r_count") == 3 {
            t3 = t;
            break;
        }
    }
    assert!(t3 > 0, "third coin never collected");
    assert_eq!(
        audio(&mut s),
        vec![(3, 0), (4, 1)],
        "one bundle, two cues, ordinals strictly increasing"
    );
    assert_eq!(int(&mut s, "get_r_np"), 0, "closeout consumed the pendings");
    assert_eq!(int(&mut s, "get_r_p0"), 0, "pending slot 0 kept the coin cue");
    assert_eq!(int(&mut s, "get_r_p1"), 1, "pending slot 1 kept the win cue");
    assert_eq!(int(&mut s, "get_r_ord"), 4, "two ordinals spent in the bundle");
    assert_eq!(int(&mut s, "get_r_state"), 0, "set-complete cue does not win");
    assert_eq!(hud_text(&mut s, 0), "3");
    assert_eq!(hud_text(&mut s, 1), "Playing");
    // The pad fanfare is the NEXT bundle: one row, ordinal 5.
    let mut twin = 0;
    while t < t0 + 1200 {
        t += 1;
        let st = tick(&mut s, t, 2, 0);
        assert_eq!(st, 0, "tick {t} over the pool");
        if int(&mut s, "get_r_state") == 1 {
            twin = t;
            break;
        }
    }
    assert!(twin > t3);
    assert_eq!(audio(&mut s), vec![(5, 1)], "fanfare is its own bundle");
    println!("V03_D two-cue bundle: settle={t0} t3={t3} twin={twin}");
}

/// V-03 (e): ordinal sequence integrity across a full run — recording every
/// emitted (tick, ordinal, cue) from settle to win yields exactly
/// (t1,1,coin) (t2,2,coin) (t3,3,coin) (t3,4,win) (twin,5,win): ordinals
/// 1..5 strictly increasing, no reuse, nothing after the latch.
#[test]
fn v03_e_ordinal_sequence_integrity_full_run() {
    let trace = drive_win_trace(WORLD, "t04-e");
    let seq: Vec<(i32, i32)> = trace
        .events
        .iter()
        .flat_map(|e| e.rows.iter().map(|(o, c)| (*o, *c)))
        .collect();
    assert_eq!(
        seq,
        vec![(1, 0), (2, 0), (3, 0), (4, 1), (5, 1)],
        "full-run ordinal/cue sequence"
    );
    // Every ordinal strictly greater than the previous (no reuse).
    for w in seq.windows(2) {
        assert!(w[1].0 > w[0].0, "ordinals strictly increasing: {seq:?}");
    }
    // The recorded event ticks: three distinct collect ticks, the two-cue
    // bundle on the third, the win after it.
    let ticks: Vec<i32> = trace.events.iter().map(|e| e.tick).collect();
    assert_eq!(ticks.len(), 4, "four latch ticks: {ticks:?}");
    assert_eq!(trace.events[0].count, 1);
    assert_eq!(trace.events[1].count, 2);
    assert_eq!(trace.events[2].count, 3);
    assert_eq!(trace.events[2].rows, vec![(3, 0), (4, 1)]);
    assert_eq!(trace.events[3].state, 1);
    assert!(ticks[2] < ticks[3]);
    // Gate observable: consumed (0) on every latch turn post-tick.
    for e in &trace.events {
        assert_eq!(e.np, 0, "closeout consumed every bundle");
    }
    println!(
        "V03_E ordinals 1..5 no reuse: settle={} ticks={ticks:?} win={}",
        trace.settle, trace.win
    );
}

/// V-03 (f): restart-shape smoke — Run 1 walks to count 2 (ordinals 1..2
/// spent); a fresh Run session over the SAME world source rebuilds the
/// model: count 0, state 0, ord 0, np 0, every coin alive again, HUD back
/// to "0"/"Playing", and the first collect of Run 2 carries ordinal 1 (per
/// Run counter, no old-Run leakage). Full stop/reset semantics stay T-05.
#[test]
fn v03_f_restart_shape_smoke_fresh_run_rebuilds() {
    // Run 1: drive to count 2.
    let mut s1 = session(WORLD, "t04-f-run1.at", 1);
    let t0 = settle(&mut s1, "t04-f-run1");
    let mut t = t0;
    let mut t2 = 0;
    while t < t0 + 600 {
        t += 1;
        let st = tick(&mut s1, t, 2, 0);
        assert_eq!(st, 0, "run1 tick {t}");
        if int(&mut s1, "get_r_count") == 2 {
            t2 = t;
            break;
        }
    }
    assert!(t2 > 0, "run1 never reached count 2");
    assert_eq!(int(&mut s1, "get_r_ord"), 2, "run1 spent ordinals 1..2");
    assert_eq!(int(&mut s1, "get_c1_alive"), 0);
    assert_eq!(int(&mut s1, "get_c2_alive"), 0);
    assert_eq!(hud_text(&mut s1, 0), "2");

    // Fresh start: a new Run session on the same world source (run id 2).
    let mut s2 = session(WORLD, "t04-f-run2.at", 2);
    let r0 = settle(&mut s2, "t04-f-run2");
    assert_eq!(int(&mut s2, "get_r_count"), 0, "count rebuilt");
    assert_eq!(int(&mut s2, "get_r_state"), 0, "state rebuilt");
    assert_eq!(int(&mut s2, "get_r_ord"), 0, "ordinal counter rebuilt");
    assert_eq!(int(&mut s2, "get_r_np"), 0);
    assert_eq!(int(&mut s2, "get_c1_alive"), 1, "coin 1 alive again");
    assert_eq!(int(&mut s2, "get_c2_alive"), 1, "coin 2 alive again");
    assert_eq!(int(&mut s2, "get_c3_alive"), 1, "coin 3 alive again");
    assert_eq!(hud_text(&mut s2, 0), "0", "HUD rebuilt for the new Run");
    assert_eq!(hud_text(&mut s2, 1), "Playing");
    // Run 2 walks green and its first cue restarts the ordinals at 1.
    let mut t = r0;
    let mut first = 0;
    while t < r0 + 400 {
        t += 1;
        let st = tick(&mut s2, t, 2, 0);
        assert_eq!(st, 0, "run2 tick {t}");
        if int(&mut s2, "get_r_count") == 1 {
            first = t;
            break;
        }
    }
    assert!(first > 0, "run2 first collect missing");
    assert_eq!(audio(&mut s2), vec![(1, 0)], "run 2 ordinals restart at 1");
    assert_eq!(int(&mut s2, "get_r_ord"), 1, "no old-Run leakage");
    println!(
        "V03_F restart shape: run1 settle={t0} t2={t2}; run2 settle={r0} first={first} ord=1"
    );
}

// ---------------------------------------------------- behavior equality ----

/// T-04 behavior equivalence: the SAME input script (settle -> walk right
/// until Won -> 10 post ticks) through the GENERATED world and through the
/// T-02 hand-proven reference world must produce the identical observable
/// sequence at the identical tick indices: settle tick, every collect/win
/// latch tick, count/state/ord/np per tick, audio rows, HUD texts.
/// Recorded (and acceptable) static deltas, none trace-visible: the
/// reference carried a dead `r_state == 9` forced-dispatch site (generator
/// removed it) and indexed the fail path at zone_alive[0] (generator fixes
/// it to [1]; only get_hazard_alive differs, not the game trace).
#[test]
fn v03_equivalence_generated_matches_t02_reference_trace() {
    assert_eq!(sha256_hex(REF_WORLD.as_bytes()), REF_WORLD_SHA, "reference pinned");
    assert!(
        REF_WORLD.contains("T-02 probe wiring (hand extension"),
        "the fixture is the hand-extended T-02 reference world"
    );
    assert!(
        REF_WORLD.contains("r_state == 9") && !WORLD.contains("r_state == 9"),
        "reference keeps the forced dispatch site, generated dropped it"
    );
    let gen = drive_win_trace(WORLD, "t04-eq-gen.at");
    let refw = drive_win_trace(REF_WORLD, "t04-eq-ref.at");
    assert_eq!(gen.settle, refw.settle, "settle tick must match");
    assert_eq!(gen.events, refw.events, "event trace must match tick for tick");
    assert_eq!(gen.win, refw.win, "win latch tick must match");
    assert_eq!(gen.end, refw.end, "post-win walk must match");
    let ticks: Vec<i32> = gen.events.iter().map(|e| e.tick).collect();
    let seq: Vec<(i32, i32)> = gen
        .events
        .iter()
        .flat_map(|e| e.rows.iter().map(|(o, c)| (*o, *c)))
        .collect();
    assert_eq!(seq, vec![(1, 0), (2, 0), (3, 0), (4, 1), (5, 1)]);
    println!(
        "EQUIV settle={} latch_ticks={ticks:?} win={} ordinals={seq:?} (generated == t02 reference)",
        gen.settle, gen.win
    );
}
