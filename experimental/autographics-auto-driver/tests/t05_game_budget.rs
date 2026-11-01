//! auto-ui PLAN-023 T-05 V-05 budget battery on the GENERATED
//! components-2d-game world (world sha ed98b0b5... — the T-03 product, never
//! modified). The freeze F-7 credit table is RE-PINNED here with real
//! named-fault controls, using the T-02/T-04 harness choice: CPU-session
//! credit instrumentation via deliberately-overcharging world VARIANTS
//! derived in memory from the committed product text (the budget-plus trick:
//! one extra charged ag_event_next on selected ticks). No generated product
//! byte is touched; every variant asserts it differs from the pinned world.
//!   a. walk-only 30-pin: green at 30, +2 green (at the 32 pool), +3 -> named
//!      fault 10 at the first tick;
//!   b. full playthrough every-tick green (all ticks <= 32), the run
//!      completes the win (settle 13, collects 85/146/207, win 251 — the
//!      T-04 V-03 latch table), with the VM envelope printed;
//!   c. plus-one (one extra charged call on every cue tick) -> fault 10
//!      exactly at the first collection tick (the 32 boundary is real);
//!   d. naive-ungated control (idle-turn gate removed, held reads ungated,
//!      collection tick 34) -> fault 10 exactly at the first collection
//!      tick — the gate is load-bearing;
//!   e. goal/hazard/pad-stay at their frozen values: goal latch tick = 32
//!      exactly (+1 -> fault at the win tick), hazard fail tick = 31 exactly
//!      (+1 green at the fail tick, +2 -> fault there), pad-stay (count<3)
//!      HONESTLY re-measured at 33 -> fault 10 at the first pad-overlap tick
//!      on a derived coin-bind-stripped variant (the freeze F-7 row said 32,
//!      but its own F-6 composition 30+next+pair+bundle sums to 33; the
//!      shape is UNREACHABLE in the shipped level — coin 3 forces count==3
//!      at tick 207, before the pad AABB is even reachable at tick 248 — so
//!      the shipped game never executes it; the reachable pad shapes are
//!      green: the goal latch 32 and post-win stand 31).
//! Run single-threaded:
//! cargo test --locked --offline --test t05_game_budget -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d-game/game/world.at");
/// T-03 pin: the generated world under test is the committed product.
const WORLD_SHA: &str = "ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0";

const DT: f32 = 1.0 / 60.0;

fn sha256_hex(bytes: &[u8]) -> String {
    agp::renderer::capture::sha256_hex(bytes)
}

// ------------------------------------------------------------ variants ----
// All variants are derived IN MEMORY from the committed world text; none is
// written to the product tree. Each constructor asserts the exact edit site
// exists exactly once, so the derivation tracks the frozen product.

/// walk-only: strip the event poll call and the closeout call (the components
/// walk shape remains: pressed 1 + held 2 + move 26 + grounded 1 = 30).
fn walkonly() -> String {
    let w = WORLD
        .replacen(" game_events()\n", "", 1)
        .replacen(" game_closeout()\n", "", 1);
    assert_ne!(w, WORLD, "walk-only derivation");
    assert!(w.contains("fn game_events()"), "fns stay, only calls stripped");
    assert!(!w.contains(" game_events()\n") && !w.contains(" game_closeout()\n"));
    w
}

/// +n extra charged calls on every tick of an arbitrary derived world.
fn plus_every_tick_on(base: &str, n: usize) -> String {
    let mut w = base.to_string();
    let marker = " player_lifetick(0,dt)\n";
    assert_eq!(w.matches(marker).count(), 1, "plus marker");
    let mut extra = String::new();
    for i in 0..n {
        extra.push_str(&format!(" let xp{i} = ag_event_next()\n"));
    }
    let at = w.find(marker).unwrap();
    w.replace_range(at..at + marker.len(), &format!("{marker}{extra}"));
    w
}

/// plus-one: exactly one extra charged call on every cue-carrying tick (the
/// T-02 budget-plus trick; on the generated world only latch ticks carry
/// cues, so the +1 lands exactly on the collection/goal/fail ticks).
fn plus_one_on_cue_ticks() -> String {
    let marker = "fn game_closeout() {\n";
    assert_eq!(WORLD.matches(marker).count(), 1, "closeout marker");
    let w = WORLD.replacen(
        marker,
        &format!("{marker} if r_np > 0 {{\n  let xq = ag_event_next()\n }}\n"),
        1,
    );
    assert_ne!(w, WORLD, "plus-one derivation");
    w
}

/// naive: the idle-turn gate removed (the two held reads run ungated on
/// event ticks; collection tick = 30 + next + pair + despawn + bundle = 34).
fn naive_ungated() -> String {
    let gate = " var axis = 0\n if (player_halt[i] == 1) {\n  player_halt[i] = 0\n } else {\n  axis = (ag_action_held(1) - ag_action_held(0))\n }\n";
    assert_eq!(WORLD.matches(gate).count(), 1, "gate marker");
    let w = WORLD.replace(
        gate,
        " let axis = (ag_action_held(1) - ag_action_held(0))\n",
    );
    assert_ne!(w, WORLD, "naive derivation");
    w
}

/// pad-stay reachability variant: strip the three coin trigger binds so the
/// character can walk into the pad AABB with count 0 (stay events fire every
/// tick inside). Same physics, same spawns, same pad/hazard binds.
fn padstay_no_coin_binds() -> String {
    let mut w = String::new();
    for line in WORLD.lines() {
        let is_coin_bind = line.trim_start().starts_with("ok=ok+ag_trigger_bind(")
            && (line.contains("coin_handle[0]")
                || line.contains("coin_handle[1]")
                || line.contains("coin_handle[2]"));
        if !is_coin_bind {
            w.push_str(line);
            w.push('\n');
        }
    }
    assert!(w.len() < WORLD.len(), "coin binds stripped");
    assert_eq!(w.matches("ag_trigger_bind(").count(), 2, "pad+hazard binds only");
    w
}

/// +n extra charged calls only in the hazard (fail) latch branch.
fn hazard_plus(n: usize) -> String {
    let marker = " if p == 3 {\n  hazard_enter(1, 0)\n";
    assert_eq!(WORLD.matches(marker).count(), 1, "hazard marker");
    let mut extra = String::new();
    for i in 0..n {
        extra.push_str(&format!("  let xh{i} = ag_event_next()\n"));
    }
    let w = WORLD.replacen(marker, &format!("{marker}{extra}"), 1);
    assert_ne!(w, WORLD, "hazard+{n} derivation");
    w
}

/// +n extra charged calls only in the goal (win) latch branch.
fn goal_plus(n: usize) -> String {
    let marker = "    pad_enter(0, 0)\n";
    assert_eq!(WORLD.matches(marker).count(), 1, "goal marker");
    let mut extra = String::new();
    for i in 0..n {
        extra.push_str(&format!("    let xg{i} = ag_event_next()\n"));
    }
    let w = WORLD.replacen(marker, &format!("{marker}{extra}"), 1);
    assert_ne!(w, WORLD, "goal+{n} derivation");
    w
}

// ------------------------------------------------------------- session ----

/// CPU session over a game world (Hud2V1 admission, the T-02/T-04 harness
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
    // Scene rows of the generated game (7 rows, translation-relative boxes).
    let rows: [(i32, i32, f32, f32, f32, f32); 7] = [
        (1, 1, 2.0, 1.0, 0.8, 1.0),      // player.body
        (2, 1, 8.0, -0.5, 24.0, 1.0),    // ground
        (3, 0, -30.0, -40.0, 0.5, 0.5),  // c1 template
        (4, 0, -30.0, -42.0, 0.5, 0.5),  // c2 template
        (5, 0, -30.0, -44.0, 0.5, 0.5),  // c3 template
        (6, 1, -30.0, -40.0, 2.0, 1.0),  // pad template
        (7, 1, -30.0, -42.0, 40.0, 2.0), // hazard template
    ];
    s.call("_ag_spatial_reset", &[Int(7)], ResultKind::Unit).unwrap();
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
    for n in 0..7 {
        for name in ["_ag_char_map", "_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n + 1), Int(n)], ResultKind::Unit).unwrap();
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
fn tick(s: &mut Session, step: i32, held: i32) -> i32 {
    s.call("_ag_char_input", &[Int(held), Int(0)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}

fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}

/// Settle the character onto the ground (state-based) and return the tick.
fn settle(s: &mut Session, name: &str) -> i32 {
    for t in 1..=300 {
        let st = tick(s, t, 0);
        assert_eq!(st, 0, "{name} settle tick {t} status {st}");
        if int(s, "get_player_grounded") == 1 {
            return t;
        }
    }
    panic!("{name}: never settled");
}

/// Latch marks of a right-walk from settle on the real world: the first tick
/// where count reaches 1/2/3 and where state latches Won. Every tick is
/// asserted green. Matches the T-04 V-03 table (13, 85, 146, 207, 251).
fn win_marks(s: &mut Session, name: &str) -> (i32, i32, i32, i32, i32) {
    let t0 = settle(s, name);
    let mut t = t0;
    let (mut t1, mut t2, mut t3, mut twin) = (0, 0, 0, 0);
    while t < t0 + 1200 {
        t += 1;
        let st = tick(s, t, 2);
        assert_eq!(st, 0, "{name} tick {t} over the pool");
        let c = int(s, "get_r_count");
        if c == 1 && t1 == 0 {
            t1 = t;
        }
        if c == 2 && t2 == 0 {
            t2 = t;
        }
        if c == 3 && t3 == 0 {
            t3 = t;
        }
        if int(s, "get_r_state") == 1 {
            twin = t;
            break;
        }
    }
    assert!(twin > 0, "{name}: pad never won");
    (t0, t1, t2, t3, twin)
}

/// The fail-latch tick of a left-walk from settle (every tick green).
fn fail_mark(s: &mut Session, name: &str) -> i32 {
    let t0 = settle(s, name);
    let mut t = t0;
    while t < t0 + 500 {
        t += 1;
        let st = tick(s, t, 1);
        assert_eq!(st, 0, "{name} fail-path tick {t} over the pool");
        if int(s, "get_r_state") == 2 {
            return t;
        }
    }
    panic!("{name}: never failed");
}

/// First non-green tick of a right-walk from settle (0 = green throughout).
fn first_fault(s: &mut Session, name: &str, bound: i32, held: i32) -> i32 {
    let t0 = settle(s, name);
    let mut t = t0;
    while t < t0 + bound {
        t += 1;
        let st = tick(s, t, held);
        if st != 0 {
            return t;
        }
    }
    0
}

// ---------------------------------------------------------------- V-05 ----

/// V-05 (a): the components walk tick is exactly 30 on the generated world.
/// walk-only (poll and closeout stripped) runs 120 ticks green at 30; +2
/// charged calls land exactly AT the 32 pool (green); +3 = 33 raises the
/// named runtime fault 10 at the very first tick, and it latches.
#[test]
fn v05_a_walk_tick_pins_exactly_30() {
    assert_eq!(sha256_hex(WORLD.as_bytes()), WORLD_SHA, "generated world pinned");
    let mut s = session(&walkonly(), "t05a-walkonly.at", 1);
    for t in 1..=120 {
        assert_eq!(tick(&mut s, t, 2), 0, "walk-only tick {t}");
    }
    assert_eq!(int(&mut s, "get_player_grounded"), 1, "settled");
    let mut s2 = session(&plus_every_tick_on(&walkonly(), 2), "t05a-walkplus2.at", 1);
    assert_eq!(tick(&mut s2, 1, 2), 0, "walk+2 = 32 sits exactly at the pool");
    let mut s3 = session(&plus_every_tick_on(&walkonly(), 3), "t05a-walkplus3.at", 1);
    assert_eq!(tick(&mut s3, 1, 2), 10, "walk+3 = 33 faults named 10 at tick 1");
    assert_eq!(tick(&mut s3, 2, 2), 10, "fault 10 latches");
    assert_eq!(int(&mut s3, "get_player_grounded"), 0, "world froze at the fault");
    println!("V05_A walk tick = 30 exactly on the generated world (120 green; +2 at-pool green; +3 fault 10 at tick 1)");
}

/// V-05 (b): the full playthrough is every-tick green (all ticks <= 32) and
/// completes the win. The latch ticks re-pin the T-04 V-03 table on this
/// build: settle 13, coin1 85, coin2 146, coin3 207, win 251; ordinals 1..5;
/// post-win walk ticks stay green, latched and silent. VM envelope printed
/// over the whole run (report, not a gate).
#[test]
fn v05_b_playthrough_every_tick_green_completes_win() {
    let mut s = session(WORLD, "t05b.at", 1);
    let t0 = settle(&mut s, "t05b");
    let mut t = t0;
    let (mut t1, mut t2, mut t3, mut twin) = (0, 0, 0, 0);
    while t < t0 + 1200 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2), 0, "playthrough tick {t} over the pool");
        let c = int(&mut s, "get_r_count");
        if c == 1 && t1 == 0 {
            t1 = t;
        }
        if c == 2 && t2 == 0 {
            t2 = t;
        }
        if c == 3 && t3 == 0 {
            t3 = t;
        }
        if int(&mut s, "get_r_state") == 1 {
            twin = t;
            break;
        }
    }
    assert!(t1 < t2 && t2 < t3 && t3 < twin, "latch order: {t1} {t2} {t3} {twin}");
    assert_eq!(int(&mut s, "get_r_ord"), 5, "ordinals 1..5 spent");
    assert_eq!(int(&mut s, "get_c1_alive") + int(&mut s, "get_c2_alive") + int(&mut s, "get_c3_alive"), 0);
    // post-win: green, latched, no re-poll (state != 0 early-returns).
    for _ in 0..10 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2), 0, "post-win tick {t}");
        assert_eq!(int(&mut s, "get_r_ord"), 5);
    }
    assert!(s.max_call_steps < 100_000, "VM step guard headroom");
    assert!(s.max_call_slices <= 120, "slice guard");
    println!(
        "V05_B every-tick green: settle={t0} t1={t1} t2={t2} t3={t3} twin={twin} end={t} | VM envelope: max_call_steps={} max_call_slices={} calls={} (report, not a gate)",
        s.max_call_steps,
        s.max_call_slices,
        s.calls
    );
}

/// V-05 (c): the collection tick is exactly 32 — a derived variant adding
/// exactly ONE charged call on every cue-carrying tick runs green up to the
/// first collection and raises the named fault 10 EXACTLY there (85 on this
/// build), with the fault latching and the world frozen at the fault tick.
#[test]
fn v05_c_plus_one_faults_10_at_first_collection_tick() {
    let mut sb = session(WORLD, "t05c-base.at", 1);
    let (_, t1, _, _, _) = win_marks(&mut sb, "t05c-base");
    drop(sb);
    let mut sp = session(&plus_one_on_cue_ticks(), "t05c-plus.at", 1);
    let fault = first_fault(&mut sp, "t05c-plus", 1200, 2);
    assert_eq!(fault, t1, "plus-one must fault exactly at the first collection tick");
    assert_eq!(tick(&mut sp, fault + 1, 2), 10, "fault 10 latches");
    assert_eq!(int(&mut sp, "get_r_count"), 1, "world froze at the fault tick");
    assert_eq!(int(&mut sp, "get_r_state"), 0, "no further game progress");
    println!("V05_C collection tick = 32 exactly: +1 on cue ticks faults 10 exactly at t={fault} (baseline t1={t1}), all earlier ticks green");
}

/// V-05 (d): the idle-turn gate is load-bearing. The naive variant (only the
/// halt-gate branch removed, so the two held reads run ungated on event
/// ticks; collection tick = 34) faults the named 10 exactly at the first
/// collection tick — while the identical input script on the real world is
/// green through the same tick (asserted by the baseline run here).
#[test]
fn v05_d_naive_ungated_faults_10_gate_is_load_bearing() {
    let mut sb = session(WORLD, "t05d-base.at", 1);
    let (_, t1, _, _, twin) = win_marks(&mut sb, "t05d-base");
    drop(sb);
    let mut sn = session(&naive_ungated(), "t05d-naive.at", 1);
    let fault = first_fault(&mut sn, "t05d-naive", 1200, 2);
    assert_eq!(fault, t1, "naive (34 on the collection tick) faults exactly there");
    assert_eq!(tick(&mut sn, fault + 1, 2), 10, "fault 10 latches; run dead");
    assert_eq!(int(&mut sn, "get_r_count"), 1, "frozen at the first collect");
    println!("V05_D naive control: ungated held reads kill the game at t={fault} (baseline green t1={t1} twin={twin}) — the idle-turn gate closes 34 -> 32");
}

/// V-05 (e): goal/hazard/pad-stay at their frozen values, measured:
///   goal latch tick = 32 EXACTLY (real world green at twin; +1 in the goal
///     branch -> named fault 10 exactly at the win tick);
///   hazard fail tick = 31 EXACTLY (+1 in the fail branch stays green and
///     still latches Failed; +2 = 33 -> fault exactly at the fail tick);
///   pad-stay (count<3) = 33 HONEST RE-MEASUREMENT: on a derived variant
///     with the coin binds stripped the character walks into the pad AABB
///     with count 0 and the FIRST pad-stay tick faults the named 10 at t=248
///     (33 = 30 + next + pair + always-charged closeout; the closeout-stripped
///     control is green through and +1 faults at the same tick). The freeze
///     F-7 row said 32 while its own F-6 composition sums to 33 — this test
///     records the measured value. The shape is UNREACHABLE in the shipped
///     level: coin 3 forces count==3 at tick 207, before the pad AABB is even
///     reachable (first overlap at tick 248 on the identical physics), so the
///     shipped game never executes a pad-stay tick; its reachable pad shapes
///     (goal latch 32, post-win stand 31 = the playthrough's green post-win
///     ticks) are asserted green in (b) and here.
#[test]
fn v05_e_goal_hazard_padstay_at_frozen_values() {
    // baseline marks on the real world
    let twin = win_marks(&mut session(WORLD, "t05e-win.at", 1), "t05e-win").4;
    let tfail = fail_mark(&mut session(WORLD, "t05e-fail.at", 1), "t05e-fail");
    // goal = 32 exactly: +1 in the goal branch faults exactly at the win tick.
    let mut sg = session(&goal_plus(1), "t05e-goalplus.at", 1);
    let gfault = first_fault(&mut sg, "t05e-goalplus", 1200, 2);
    assert_eq!(gfault, twin, "goal+1 faults exactly at the win tick");
    assert_eq!(tick(&mut sg, gfault + 1, 2), 10, "fault latches");
    // hazard = 31 exactly: +1 green (still latches), +2 faults at the fail tick.
    for (n, want_fault) in [(1usize, false), (2, true)] {
        let mut sh = session(&hazard_plus(n), &format!("t05e-hplus{n}.at"), 1);
        let h0 = settle(&mut sh, &format!("t05e-hplus{n}"));
        let mut t = h0;
        let mut at = 0;
        let mut faulted = false;
        while t < h0 + 500 {
            t += 1;
            let st = tick(&mut sh, t, 1);
            if st != 0 {
                at = t;
                faulted = true;
                break;
            }
            if int(&mut sh, "get_r_state") == 2 {
                at = t;
                break;
            }
        }
        assert_eq!(at, tfail, "hazard+{n} moves exactly at the fail tick");
        assert_eq!(faulted, want_fault, "hazard+{n} fault expectation");
        if !want_fault {
            assert_eq!(int(&mut sh, "get_r_state"), 2, "fail still latches at 32");
            assert_eq!(int(&mut sh, "get_r_ord"), 1, "fail buzz emitted");
        }
    }
    // pad-stay honest re-measurement on the reachability variant.
    let mut sp = session(&padstay_no_coin_binds(), "t05e-padstay.at", 1);
    let pfault = first_fault(&mut sp, "t05e-padstay", 700, 2);
    assert!(pfault > 0, "pad-stay (count<3) measures 33 -> named fault 10");
    assert_eq!(tick(&mut sp, pfault + 1, 2), 10, "fault latches");
    assert_eq!(int(&mut sp, "get_r_count"), 0, "no coins on this variant");
    assert_eq!(int(&mut sp, "get_r_state"), 0, "the pad did not latch (count<3)");
    // closeout-stripped control: the event shape alone is 32 (green through),
    // and +1 faults at exactly the same tick -> the 33rd credit is the closeout.
    let mut base = padstay_no_coin_binds();
    let marker = " game_closeout()\n";
    assert_eq!(base.matches(marker).count(), 1);
    base.replace_range(base.find(marker).unwrap()..base.find(marker).unwrap() + marker.len(), "");
    let mut snc = session(&base, "t05e-psnc.at", 1);
    let ncfault = first_fault(&mut snc, "t05e-psnc", 700, 2);
    assert_eq!(ncfault, 0, "pad-stay event shape = 32, green without the closeout");
    let mut snc1 = session(&plus_every_tick_on(&base, 1), "t05e-psnc1.at", 1);
    let nc1fault = first_fault(&mut snc1, "t05e-psnc1", 700, 2);
    assert_eq!(nc1fault, pfault, "+1 faults exactly at the first pad-stay tick");
    // In-level unreachability: the first pad overlap tick (pfault, identical
    // physics) is strictly after the real world's coin-3 latch (207): count
    // is always 3 before any pad-stay tick could exist.
    let (_, _, _, t3, _) = win_marks(&mut session(WORLD, "t05e-order.at", 1), "t05e-order");
    assert!(
        t3 < pfault,
        "count==3 latches (t={t3}) before the pad AABB is reachable (t={pfault})"
    );
    println!(
        "V05_E goal=32 exact (+1 faults at win tick {gfault}), hazard=31 exact (+1 green, +2 faults at {tfail}), pad-stay(count<3)=33 measured (fault 10 at {pfault}; shape unreachable in-level: coin3 t={t3} < pad entry t={pfault})"
    );
}
