//! auto-ui PLAN-032 node2d_full_game_flow: the complete frozen G2 game runs
//! end-to-end on the real autographics.components2d/1 composition. The
//! hand-proven T-02 world (tests/fixtures/plan032/world-g2-components2d.at)
//! expresses player movement/jump/gravity, continuous enemy patrol, three
//! same-definition coins with isolated local state, hidden pit zones, goal,
//! clamped follow camera, sprite frames/flip, four sound requests and the
//! four-slot HUD — one ordinary Auto program, no business names in the
//! engine. The route covers the original G2-01..09 denominator and the
//! frozen budget: the walk tick is 31, the collection tick is exactly the
//! 32 pool, and candidate A (same world on the existing cursor+single-
//! despawn operations only, derived here by the frozen textual delta) walks
//! green but faults named 10 exactly at the first collection.
//! Run single-threaded:
//!   cargo test --locked --offline --test node2d_full_game_flow -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const DT: f32 = 1.0 / 60.0;
const WORLD: &str = include_str!("fixtures/plan032/world-g2-components2d.at");

/// Scene rows (id, kind, tx, ty, sx, sy): player, seven solids, four parked
/// templates (coin/enemy/goal/pit).
const ROWS: [(i32, i32, f32, f32, f32, f32); 12] = [
    (1, 1, 2.0, 1.0, 0.8, 1.0),       // player.body (character 0.8x1.0)
    (2, 1, 4.0, -0.5, 8.0, 1.0),      // segment [0,8]x[-1,0]
    (3, 1, 14.0, -0.5, 8.0, 1.0),     // segment [10,18]x[-1,0]
    (4, 1, 25.0, -0.5, 10.0, 1.0),    // segment [20,30]x[-1,0]
    (5, 1, 13.5, 1.125, 3.0, 0.25),   // top platform [12,15]x[1,1.25]
    (6, 1, -0.2, 3.0, 0.4, 14.0),     // wall x=0
    (7, 1, 30.2, 3.0, 0.4, 14.0),     // wall x=30
    (8, 1, 15.0, 10.2, 30.8, 0.4),    // ceiling y=10
    (9, 0, -30.0, -40.0, 0.5, 0.5),   // coin template
    (10, 0, -30.0, -42.0, 0.9, 0.9),  // enemy template
    (11, 0, -30.0, -44.0, 0.6, 1.0),  // goal template
    (12, 0, -30.0, -46.0, 12.0, 0.5), // hidden pit zone template
];

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
    for (n, (id, kind, tx, ty, sx, sy)) in ROWS.iter().enumerate() {
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
    // Player sprite row: sprite id 1 on node 0 (the character leaf), 64x16
    // four-frame atlas; the world passes the frame rect per bundle call.
    for (name, args) in [
        ("_ag_spr_map", vec![Int(1), Int(0), Int(0)]),
        ("_ag_spr_img", vec![Int(0), Int(160), Int(32)]),
        ("_ag_spr_def", vec![Int(0), Int(0), Int(0), Int(32)]),
        ("_ag_spr_defh", vec![Int(0), Int(32), Int(1)]),
    ] {
        s.call(name, &args, ResultKind::Unit).unwrap();
    }
    s.call("_ag_audio_config", &[Int(4)], ResultKind::Unit).unwrap();
    // HUD slots: slot 0 renders "0".."3" (pre-warmed digits), slot 1 renders
    // "Playing"/"Won"/"Failed" (pre-warmed letters), slots 2/3 stay empty.
    let state_glyphs = "PlayingWonedF";
    for (slot, count, codes) in [
        (0, 4usize, vec![48i32, 49, 50, 51]),
        (1, state_glyphs.len(), state_glyphs.bytes().map(|b| b as i32).collect()),
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
            s.call("_ag_hud_config_code", &[Int(slot), Int(code)], ResultKind::Unit)
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

/// One input tick: (held, pressed) bitmasks (L=1 R=2 J=4 N=8) + `_ag_tick`.
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

fn f32v(s: &mut Session, name: &str) -> f32 {
    s.float(name, &[]).unwrap()
}

/// Candidate A derivation: the SAME world restricted to the pre-existing
/// operations (single-cursor event reads, single despawn). This is the
/// complete existing-API composition; nothing else changes.
fn candidate_a() -> String {
    let events_old = "  let n = ag_events_snapshot()\n  for i in 0..n {\n   handle_event(i)\n  }\n";
    let events_new = "  drain_events()\n";
    assert_eq!(WORLD.matches(events_old).count(), 1, "events block pinned");
    let mut a = WORLD.replace(events_old, events_new);
    for (j, h) in [
        ("ag_object_despawn_batch(1, coin_h[0], 0, 0, 0, 0, 0, 0, 0)", "ag_object_despawn(coin_h[0])"),
        ("ag_object_despawn_batch(1, coin_h[1], 0, 0, 0, 0, 0, 0, 0)", "ag_object_despawn(coin_h[1])"),
        ("ag_object_despawn_batch(1, coin_h[2], 0, 0, 0, 0, 0, 0, 0)", "ag_object_despawn(coin_h[2])"),
    ] {
        assert_eq!(a.matches(j).count(), 1, "collect branch pinned: {j}");
        a = a.replace(j, h);
    }
    a.push_str(
        "\n// candidate A: the existing single-cursor drain (all events, this tick)\nfn drain_events() {\n let m1 = ag_event_next()\n if (m1 == 1) {\n  handle_pair(ag_event_pair())\n  let m2 = ag_event_next()\n  if (m2 == 1) {\n   handle_pair(ag_event_pair())\n   let m3 = ag_event_next()\n   if (m3 == 1) {\n    handle_pair(ag_event_pair())\n    let m4 = ag_event_next()\n    if (m4 == 1) {\n     handle_pair(ag_event_pair())\n     let m5 = ag_event_next()\n     if (m5 == 1) {\n      handle_pair(ag_event_pair())\n      let m6 = ag_event_next()\n      if (m6 == 1) {\n       handle_pair(ag_event_pair())\n       let m7 = ag_event_next()\n       if (m7 == 1) {\n        handle_pair(ag_event_pair())\n        let m8 = ag_event_next()\n        if (m8 == 1) {\n         handle_pair(ag_event_pair())\n        }\n       }\n      }\n     }\n    }\n   }\n  }\n }\n}\n",
    );
    a
}

/// Walk right at ground level from ~x2; returns (ticks, x) once |x-target|
/// is crossed.
#[test]
fn g2_full_route_win_budget_and_candidate_a_refutation() {
    // ---------------- B: the complete game, every tick green ----------------
    let mut s = session(WORLD, "g2-b.at", 1);
    // settle onto segment 1 (no input)
    let mut t = 0;
    for i in 1..=90 {
        assert_eq!(tick(&mut s, i, 0, 0), 0, "settle tick {i}");
        if i <= 12 {
            println!("DBG settle t={i} px={} py={} g={}", f32v(&mut s, "get_px"), f32v(&mut s, "get_py"), int(&mut s, "get_p_grounded"));
        }
        t = i;
    }
    assert_eq!(int(&mut s, "get_p_grounded"), 1, "settled on ground");
    let y0 = f32v(&mut s, "get_py");
    assert!((y0 - 0.5).abs() < 0.01, "rest y=0.5 got {y0}");
    // G2-08 idle frame at rest
    assert_eq!(int(&mut s, "get_p_frame"), 0, "idle frame");

    // walk right toward coin 1 at (4,1): trigger at px>=3.1
    while f32v(&mut s, "get_px") < 3.2 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "approach tick {t}");
        assert!(t < 200, "runaway approach");
        assert_eq!(int(&mut s, "get_p_frame"), 1, "run frame while walking");
    }
    // collection lands within a couple of ticks (event read in N+1)
    for _ in 0..3 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "collect1 tick {t}");
    }
    assert_eq!(int(&mut s, "get_r_count"), 1, "coin 1 collected once");
    assert_eq!(int(&mut s, "get_coin_alive_0"), 0, "coin 1 destroyed");
    assert_eq!(int(&mut s, "get_r_ord"), 1, "one coin cue spent");

    // G2-01: release stops the drift
    let x_before = f32v(&mut s, "get_px");
    for _ in 0..12 {
        t += 1;
        assert_eq!(tick(&mut s, t, 0, 0), 0);
    }
    let x_after = f32v(&mut s, "get_px");
    assert!(
        (x_after - x_before).abs() < 1e-4,
        "release must stop: {x_before} -> {x_after}"
    );

    // walk to the pit 1 edge (x ~7.4) then jump the [8,10] gap
    while f32v(&mut s, "get_px") < 7.35 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to pit1 tick {t}");
    }
    // jump press at the edge (G2-02: grounded jump, single)
    t += 1;
    assert_eq!(tick(&mut s, t, 2, 4), 0, "jump1 press");
    assert_eq!(int(&mut s, "get_p_frame"), 2, "rise frame");
    // air: hold direction, press jump again — no second jump (G2-02)
    for _ in 0..20 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 4), 0, "air ticks {t}");
    }
    assert_eq!(int(&mut s, "get_r_ord"), 2, "jump cue spent once");
    // land on segment 2 (x ~10.5)
    let mut guard = 0;
    while int(&mut s, "get_p_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "fall to seg2 {t}");
        guard += 1;
        assert!(guard < 120, "must land");
    }
    assert!(f32v(&mut s, "get_px") > 9.9, "crossed pit 1");

    // The patrol is deterministic; step back left of its contact reach
    // (px 10.1 keeps >=0.9 to the patrol's left end at 11) and wait for the
    // enemy to commit rightward before climbing past its band.
    while f32v(&mut s, "get_px") > 10.1 {
        t += 1;
        assert_eq!(tick(&mut s, t, 1, 0), 0, "back off tick {t}");
        assert!(t < 400, "back-off bound");
    }
    while !(f32v(&mut s, "get_ex") >= 14.0 && int(&mut s, "get_e_dir") == 1) {
        t += 1;
        assert_eq!(tick(&mut s, t, 0, 0), 0, "phase wait tick {t}");
        assert!(t < 1200, "phase wait bound");
    }
    // walk to the top-platform launch point and jump onto it
    while f32v(&mut s, "get_px") < 11.35 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to top jump {t}");
        assert!(t < 600, "top jump approach bound");
    }
    t += 1;
    assert_eq!(tick(&mut s, t, 2, 4), 0, "jump2 press");
    let mut guard = 0;
    while f32v(&mut s, "get_py") < 1.74 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "rising {t}");
        guard += 1;
        assert!(guard < 60, "must reach top height");
    }
    // drift onto the top platform and settle
    let mut guard = 0;
    while int(&mut s, "get_p_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "land top {t}");
        guard += 1;
        assert!(guard < 120, "must land on top");
    }
    assert!(f32v(&mut s, "get_py") > 1.7, "standing on top platform");

    // collect coin 2 at (13,2.25) from the top platform
    while f32v(&mut s, "get_px") < 12.7 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "top walk {t}");
    }
    for _ in 0..4 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "collect2 tick {t}");
    }
    assert_eq!(int(&mut s, "get_r_count"), 2, "coin 2 collected");
    assert_eq!(int(&mut s, "get_coin_alive_1"), 0);

    // The top platform is always above the patrol band; wait there until the
    // enemy is in its left half, then drop off the right edge and cross to
    // pit 2 before it can return to contact reach.
    while f32v(&mut s, "get_ex") > 12.0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 0, 0), 0, "top phase wait {t}");
        assert!(t < 2400, "top phase wait bound");
    }
    let mut guard = 0;
    while int(&mut s, "get_p_grounded") == 1 && f32v(&mut s, "get_py") > 0.6 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "drop walk {t}");
        guard += 1;
        assert!(guard < 120, "drop walk bound");
    }
    let mut guard = 0;
    while int(&mut s, "get_p_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "drop fall {t}");
        guard += 1;
        assert!(guard < 120, "drop must land");
        assert!(f32v(&mut s, "get_px") < 18.0, "must not enter pit 2");
    }
    // pit 2 gap [18,20]: jump it from the edge
    while f32v(&mut s, "get_px") < 17.35 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to pit2 {t}");
        assert!(t < 900, "pit2 approach bound");
    }
    t += 1;
    assert_eq!(tick(&mut s, t, 2, 4), 0, "jump3 press");
    for _ in 0..20 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "air2 {t}");
    }
    let mut guard = 0;
    while int(&mut s, "get_p_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "fall seg3 {t}");
        guard += 1;
        assert!(guard < 120, "must land");
    }
    assert!(f32v(&mut s, "get_px") > 19.9, "crossed pit 2");

    // collect coin 3 at (24,1)
    while f32v(&mut s, "get_px") < 23.1 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to coin3 {t}");
    }
    for _ in 0..4 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "collect3 tick {t}");
    }
    assert_eq!(int(&mut s, "get_r_count"), 3, "coin 3 collected");
    assert_eq!(int(&mut s, "get_coin_alive_2"), 0);

    // G2-06: goal before collection does not win — verified by route shape
    // (goal pair sat overlapped only at the end); walk to the goal at (28,1)
    while f32v(&mut s, "get_px") < 27.2 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to goal {t}");
    }
    for _ in 0..4 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "win tick {t}");
    }
    assert_eq!(int(&mut s, "get_r_state"), 1, "Won");
    let ord_total = int(&mut s, "get_r_ord");
    // cues: coin1, jump1, jump2, jump3, coin2, coin3+win... exact ledger below
    assert!(ord_total >= 7, "cue ledger: {ord_total}");

    // G2-06 restart request: Enter (N) pressed at the end state
    t += 1;
    assert_eq!(tick(&mut s, t, 8, 8), 0, "end Enter tick");
    assert_eq!(s.int("_ag_trg_req_v", &[]).unwrap(), 1, "round restart requested");

    // post-end idle stays green and latched
    for _ in 0..10 {
        t += 1;
        assert_eq!(tick(&mut s, t, 0, 0), 0, "post-end {t}");
    }
    assert_eq!(int(&mut s, "get_r_state"), 1, "latched Won");
    drop(s);

    // ---------------- budget pins on the B world ----------------
    // Walk tick = 31: +1 charged call lands exactly at the 32 pool; +2 faults.
    let plus = |extra: usize| {
        let mut w = WORLD.to_string();
        let anchor = "  let n = ag_events_snapshot()";
        assert_eq!(w.matches(anchor).count(), 1);
        let inj = "  ok=ok+ag_object_alive(pit_h)*1.0\n";
        let mut out = String::new();
        for line in w.lines() {
            if line == anchor && extra > 0 {
                for _ in 0..extra {
                    out.push_str(inj);
                }
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    };
    let mut s1 = session(&plus(1), "g2-walk+1.at", 1);
    assert_eq!(tick(&mut s1, 1, 2, 0), 0, "walk+1 = 32 sits at the pool");
    drop(s1);
    let mut s2 = session(&plus(2), "g2-walk+2.at", 1);
    assert_eq!(tick(&mut s2, 1, 2, 0), 10, "walk+2 = 33 faults named 10");
    drop(s2);

    // Collection tick = exactly 32: +1 on the collect branch faults 10 exactly
    // at the first collection (t05 budget-plus pattern).
    let mut base = session(WORLD, "g2-base.at", 1);
    let mut t0 = 0;
    for i in 1..=90 {
        tick(&mut base, i, 0, 0);
        t0 = i;
    }
    let mut t1 = 0;
    let mut tt = t0;
    loop {
        tt += 1;
        tick(&mut base, tt, 2, 0);
        if int(&mut base, "get_r_count") == 1 {
            t1 = tt;
            break;
        }
        assert!(tt < t0 + 300);
    }
    drop(base);
    let mut cw = WORLD.to_string();
    let anchor = "  ok=ok+ag_object_despawn_batch(1, coin_h[0], 0, 0, 0, 0, 0, 0, 0)*1.0";
    assert_eq!(cw.matches(anchor).count(), 1);
    cw = cw.replace(
        anchor,
        "  ok=ok+ag_object_despawn_batch(1, coin_h[0], 0, 0, 0, 0, 0, 0, 0)*1.0\n  ok=ok+ag_object_alive(pit_h)*1.0",
    );
    let mut sp = session(&cw, "g2-collect+1.at", 1);
    let mut tt2 = 0;
    let mut fault_at = 0;
    loop {
        tt2 += 1;
        let held = if tt2 > 90 { 2 } else { 0 };
        let st = tick(&mut sp, tt2, held, 0);
        if st != 0 {
            fault_at = tt2;
            break;
        }
        assert!(tt2 < 600, "plus-one must fault");
    }
    assert_eq!(fault_at, t1, "plus-one faults exactly at the first collection");
    assert_eq!(int(&mut sp, "get_r_count"), 1, "world froze at the fault tick (t05 precedent: VM locals mutate, the admitted candidate is refused whole)");
    assert_eq!(int(&mut sp, "get_r_state"), 0, "no further game progress");
    drop(sp);

    // ---------------- A: the existing-operations composition ----------------
    let wa = candidate_a();
    let mut sa = session(&wa, "g2-a.at", 1);
    for i in 1..=90 {
        assert_eq!(tick(&mut sa, i, 0, 0), 0, "A settle {i}");
    }
    // A walks green at exactly the 32 pool...
    for i in 0..15 {
        let st = tick(&mut sa, 91 + i, 2, 0);
        assert_eq!(st, 0, "A walk tick green until the first coin: {i} -> {st}");
    }
    // ...and faults named 10 exactly at the first collection tick.
    let mut a_fault = 0;
    let mut ta = 105;
    loop {
        ta += 1;
        let st = tick(&mut sa, ta, 2, 0);
        if st != 0 {
            a_fault = ta;
            assert_eq!(st, 10, "A must fault the credit pool, got {st}");
            break;
        }
        assert!(ta < 400, "A must fault at the first coin");
    }
    assert_eq!(
        int(&mut sa, "get_r_count"),
        1,
        "A froze at its first collection tick: the 33rd demand (the closing bundle after next+pair+despawn) is refused, the fault latches and no further tick publishes"
    );
    assert_eq!(int(&mut sa, "get_r_state"), 0, "A cannot reach any game end");
    println!(
        "PLAN032 flow: B completes the full G2 route (win at t={t}, ord={ord_total}); \
         walk tick 31 (+1 at pool, +2 fault), collect tick exactly 32 (fault {fault_at} = t1 {t1}); \
         candidate A walks the same script and faults named 10 at its first collection t={a_fault}"
    );
}

/// G2-05/G2-03 fail routes: enemy contact and the pit both fail the level;
/// the unmet-goal rule holds on the way (G2-06) and the restart request is
/// accepted at the end state.
#[test]
fn g2_fail_routes_enemy_and_pit() {
    // enemy contact: settle, jump pit 1 onto segment 2, then wait for the
    // patrolling enemy inside its range
    let mut s = session(WORLD, "g2-enemy.at", 1);
    let mut t = 0;
    for i in 1..=90 {
        tick(&mut s, i, 0, 0);
        t = i;
    }
    while f32v(&mut s, "get_px") < 7.35 && int(&mut s, "get_r_state") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0);
        assert!(t < 400, "enemy route walk bound");
    }
    t += 1;
    assert_eq!(tick(&mut s, t, 2, 4), 0, "enemy route jump");
    for _ in 0..20 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0);
    }
    let mut guard = 0;
    while int(&mut s, "get_p_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0);
        guard += 1;
        assert!(guard < 120, "enemy route must land");
    }
    // walk a little further and stand; the enemy patrols into the player
    while f32v(&mut s, "get_px") < 12.0 && int(&mut s, "get_r_state") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0);
    }
    let mut guard = 0;
    loop {
        t += 1;
        let st = tick(&mut s, t, 0, 0);
        assert_eq!(st, 0, "enemy route stays within budget until the fail");
        if int(&mut s, "get_r_state") == 2 {
            break;
        }
        guard += 1;
        assert!(guard < 1800, "enemy must arrive (30s patrol span)");
    }
    // cues on this route: coin 1 (walked through at x~3.5), the pit-1 jump,
    // then the fail buzz — exact deterministic ledger.
    assert_eq!(int(&mut s, "get_r_count"), 1, "coin 1 collected en route");
    assert_eq!(int(&mut s, "get_r_ord"), 3, "coin + jump + buzz ledger");
    // end Enter requests a fresh round
    t += 1;
    assert_eq!(tick(&mut s, t, 8, 8), 0);
    assert_eq!(s.int("_ag_trg_req_v", &[]).unwrap(), 1);
    drop(s);

    // pit fail: walk right off segment 1 without jumping
    let mut p = session(WORLD, "g2-pit.at", 1);
    let mut t = 0;
    for i in 1..=90 {
        tick(&mut p, i, 0, 0);
        t = i;
    }
    loop {
        t += 1;
        let st = tick(&mut p, t, 2, 0);
        assert_eq!(st, 0, "pit route stays within budget until the fail");
        if int(&mut p, "get_r_state") == 2 {
            break;
        }
        assert!(t < 600, "pit must fail the walk");
    }
    assert!(f32v(&mut p, "get_py") < -1.0, "fell into the pit");
}
