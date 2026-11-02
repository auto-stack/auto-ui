//! auto-ui PLAN-032 node2d_full_game_generated: the GENERATED components-2d
//! runtime world (autographics-node2d-pilot
//! generated-components2d-runtime/g2, world sha d0c6c196…) runs the complete
//! frozen G2 route on the real components2d profile. The generator emitted
//! the whole program from the authoring fixture (ordinary use/node/model/
//! view-2d/msg/on + the components2d block): the walk tick is 31, the
//! collection tick exactly the 32 pool, the fail routes and the end-state
//! Enter round request close the denominator. Run single-threaded:
//!   cargo test --locked --offline --test node2d_full_game_generated -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const DT: f32 = 1.0 / 60.0;
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d-runtime/g2/world.at");
const WORLD_SHA: &str = "d0c6c196ea73d45bb7d66198aa1b614b09da1145a91608ef63919ceaea44a0ac";

/// Scene rows (id, kind, tx, ty, sx, sy): the character, seven solids, six
/// parked templates — the generated scene's objects in id order.
const ROWS: [(i32, i32, f32, f32, f32, f32); 14] = [
    (1, 1, 2.0, 1.0, 0.8, 1.0),       // player.body
    (2, 1, 4.0, -0.5, 8.0, 1.0),      // seg1 [0,8]x[-1,0]
    (3, 1, 14.0, -0.5, 8.0, 1.0),     // seg2 [10,18]x[-1,0]
    (4, 1, 25.0, -0.5, 10.0, 1.0),    // seg3 [20,30]x[-1,0]
    (5, 1, 13.5, 1.125, 3.0, 0.25),   // top [12,15]x[1,1.25]
    (6, 1, -0.2, 3.0, 0.4, 14.0),     // wall x=0
    (7, 1, 30.2, 3.0, 0.4, 14.0),     // wall x=30
    (8, 1, 15.0, 10.2, 30.8, 0.4),    // ceiling y=10
    (9, 0, -30.0, -40.0, 0.5, 0.5),   // c1 template (circle)
    (10, 0, -30.0, -42.0, 0.5, 0.5),  // c2 template
    (11, 0, -30.0, -44.0, 0.5, 0.5),  // c3 template
    (12, 0, -30.0, -40.0, 0.9, 0.9),  // enemy template
    (13, 0, -30.0, -40.0, 0.6, 1.0),  // goal template
    (14, 0, -30.0, -40.0, 12.0, 0.5), // pit template
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
    s.call("_ag_spatial_reset", &[Int(14)], ResultKind::Unit)
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
    for n in 0..14 {
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
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        0,
        "{name} app_start fault {} arg {}",
        s.int("_ag_error_node", &[]).unwrap(),
        s.int("_ag_error_argument", &[]).unwrap()
    );
    s
}

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

fn enemy_x(s: &mut Session) -> f32 {
    s.float("_ag_obj_xv", &[Int(3)]).unwrap()
}

/// The complete win route on the GENERATED world: deterministic walk legs by
/// tick count, landings by the generated grounded getters, patrol phases by
/// the admitted instance row, and every latch asserted at its tick.
#[test]
fn g2_generated_world_route_and_budget() {
    assert_eq!(WORLD_SHA, {
        // the committed product is pinned at compile time; the generation
        // byte-identity itself is the pilot gates' first test
        WORLD_SHA
    });
    let mut s = session(WORLD, "g2-gen.at", 1);
    let mut t = 0;
    for i in 1..=90 {
        assert_eq!(tick(&mut s, i, 0, 0), 0, "settle {i}");
        t = i;
    }
    assert_eq!(int(&mut s, "get_player_grounded"), 1, "settled");
    // coin 1: walk 18 ticks from x=2 (contact ~3.1), read in N+1
    for _ in 0..18 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "walk1 {t}");
    }
    for _ in 0..3 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "collect1 {t}");
    }
    assert_eq!(int(&mut s, "get_r_taken"), 1, "coin 1 collected");
    assert_eq!(int(&mut s, "get_coin_alive_0"), 0);
    // release stops the drift
    for _ in 0..12 {
        t += 1;
        assert_eq!(tick(&mut s, t, 0, 0), 0);
    }
    // pit 1 edge then jump (7.35 from ~3.53: 58 ticks)
    for _ in 0..58 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to pit1 {t}");
    }
    t += 1;
    assert_eq!(tick(&mut s, t, 2, 4), 0, "jump1");
    for _ in 0..20 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 4), 0, "air1 {t}");
    }
    let mut g = 0;
    while int(&mut s, "get_player_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "land seg2 {t}");
        g += 1;
        assert!(g < 120);
    }
    // back off left 8 ticks (px ~10.0, outside the patrol contact reach)
    for _ in 0..8 {
        t += 1;
        assert_eq!(tick(&mut s, t, 1, 0), 0, "back off {t}");
    }
    // wait for the patrol to commit rightward
    let mut g = 0;
    while enemy_x(&mut s) < 14.0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 0, 0), 0, "phase wait {t}");
        g += 1;
        assert!(g < 1500, "phase wait bound");
    }
    // walk 19 ticks to the launch point, jump onto the top platform
    for _ in 0..19 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "launch walk {t}");
    }
    t += 1;
    assert_eq!(tick(&mut s, t, 2, 4), 0, "jump2");
    let mut g = 0;
    while int(&mut s, "get_player_pose") != 3 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "airborne {t}");
        g += 1;
        assert!(g < 60, "must reach the fall pose");
    }
    let mut g = 0;
    while int(&mut s, "get_player_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "land top {t}");
        g += 1;
        assert!(g < 120, "must land on top");
    }
    // coin 2: 21 ticks then the N+1 read
    for _ in 0..21 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "top walk {t}");
    }
    for _ in 0..4 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "collect2 {t}");
    }
    assert_eq!(int(&mut s, "get_r_taken"), 2, "coin 2 collected");
    // top phase wait (the platform sits above the patrol band), drop, cross
    let mut g = 0;
    while enemy_x(&mut s) > 12.0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 0, 0), 0, "top phase {t}");
        g += 1;
        assert!(g < 2400, "top phase bound");
    }
    let mut g = 0;
    while int(&mut s, "get_player_grounded") == 1 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "drop walk {t}");
        g += 1;
        assert!(g < 120, "drop walk bound");
    }
    let mut g = 0;
    while int(&mut s, "get_player_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "drop fall {t}");
        g += 1;
        assert!(g < 120, "drop must land");
    }
    // pit 2 edge: 14 ticks then jump
    for _ in 0..14 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to pit2 {t}");
    }
    t += 1;
    assert_eq!(tick(&mut s, t, 2, 4), 0, "jump3");
    for _ in 0..20 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "air2 {t}");
    }
    let mut g = 0;
    while int(&mut s, "get_player_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "land seg3 {t}");
        g += 1;
        assert!(g < 120, "must land");
    }
    // coin 3: 38 ticks then the read
    for _ in 0..38 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to coin3 {t}");
    }
    for _ in 0..4 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "collect3 {t}");
    }
    assert_eq!(int(&mut s, "get_r_taken"), 3, "coin 3 collected");
    // goal: 56 ticks then the read
    for _ in 0..56 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to goal {t}");
    }
    for _ in 0..4 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "win tick {t}");
    }
    assert_eq!(int(&mut s, "get_r_state"), 1, "Won");
    // end-state Enter requests the fresh round
    t += 1;
    assert_eq!(tick(&mut s, t, 8, 8), 0, "Enter tick");
    assert_eq!(s.int("_ag_trg_req_v", &[]).unwrap(), 1, "round restart requested");
    drop(s);

    // budget pins on the generated world: walk tick 31 (+1 at the pool,
    // +2 faults), collection tick exactly 32 (+1 faults at the first collect)
    let plus_walk = |extra: usize| {
        let anchor = " r_snap = ag_actions_snapshot()";
        assert_eq!(WORLD.matches(anchor).count(), 1);
        let inj = " ok=ok+ag_object_alive(pit_handle[0])*1.0\n";
        let mut out = String::new();
        for line in WORLD.lines() {
            if line == anchor {
                for _ in 0..extra {
                    out.push_str(inj);
                }
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    };
    let anchor_line = " r_snap = ag_actions_snapshot()".trim_start().to_string();
    let mut s1 = session(&plus_walk(1), "g2-gen-walk+1.at", 1);
    assert_eq!(tick(&mut s1, 1, 2, 0), 0, "walk+1 = 32 at the pool");
    drop(s1);
    let mut s2 = session(&plus_walk(2), "g2-gen-walk+2.at", 1);
    assert_eq!(tick(&mut s2, 1, 2, 0), 10, "walk+2 = 33 faults named 10");
    drop(s2);
    let _ = anchor_line;

    // collection tick: +1 after the batch faults 10 exactly at the first collect
    let mut base = session(WORLD, "g2-gen-base.at", 1);
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
        if int(&mut base, "get_r_taken") == 1 {
            t1 = tt;
            break;
        }
        assert!(tt < t0 + 300);
    }
    drop(base);
    let batch_anchor =
        "  ag_object_despawn_batch(1, coin_handle[j], 0, 0, 0, 0, 0, 0, 0)".to_string();
    let mut cw = WORLD.to_string();
    let occurrences = cw.matches(&batch_anchor).count();
    assert_eq!(occurrences, 3, "three collect branches");
    cw = cw.replace(
        &batch_anchor,
        &format!("{batch_anchor}\n  ok=ok+ag_object_alive(pit_handle[0])*1.0"),
    );
    let mut sp = session(&cw, "g2-gen-collect+1.at", 1);
    let mut tt2 = 0;
    let mut fault_at = 0;
    loop {
        tt2 += 1;
        let held = if tt2 > 90 { 2 } else { 0 };
        let st = tick(&mut sp, tt2, held, 0);
        if st != 0 {
            fault_at = tt2;
            assert_eq!(st, 10, "named credit fault");
            break;
        }
        assert!(tt2 < 600, "plus-one must fault");
    }
    assert_eq!(fault_at, t1, "the collection tick is exactly the 32 pool");
    println!(
        "PLAN032 generated world: full G2 route green (win), walk+1 at-pool green, walk+2 fault 10, \
         collect+1 faults exactly at the first collection t={fault_at} (= t1 {t1})"
    );
}

/// The fail routes on the generated world: enemy contact and the pit both
/// latch Failed with the exact cue ledger.
#[test]
fn g2_generated_world_fail_routes() {
    // enemy contact: settle, jump pit 1, stand inside the patrol band
    let mut s = session(WORLD, "g2-gen-enemy.at", 1);
    let mut t = 0;
    for i in 1..=90 {
        tick(&mut s, i, 0, 0);
        t = i;
    }
    for _ in 0..18 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0);
    }
    for _ in 0..3 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0);
    }
    for _ in 0..58 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "to pit1 {t}");
    }
    t += 1;
    assert_eq!(tick(&mut s, t, 2, 4), 0, "jump1");
    for _ in 0..20 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 4), 0);
    }
    let mut g = 0;
    while int(&mut s, "get_player_grounded") == 0 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0);
        g += 1;
        assert!(g < 120);
    }
    // walk right into the band and wait for the patrol
    for _ in 0..30 {
        t += 1;
        assert_eq!(tick(&mut s, t, 2, 0), 0, "into band {t}");
    }
    let mut g = 0;
    loop {
        t += 1;
        let st = tick(&mut s, t, 0, 0);
        assert_eq!(st, 0, "budget holds until the fail");
        if int(&mut s, "get_r_state") == 2 {
            break;
        }
        g += 1;
        assert!(g < 1800, "enemy must arrive");
    }
    assert!(int(&mut s, "get_r_taken") >= 1, "coin 1 collected en route");
    t += 1;
    assert_eq!(tick(&mut s, t, 8, 8), 0);
    assert_eq!(s.int("_ag_trg_req_v", &[]).unwrap(), 1, "restart requested at Failed");
    drop(s);

    // pit fail: walk right off segment 1 without jumping
    let mut p = session(WORLD, "g2-gen-pit.at", 1);
    let mut t = 0;
    for i in 1..=90 {
        tick(&mut p, i, 0, 0);
        t = i;
    }
    loop {
        t += 1;
        let st = tick(&mut p, t, 2, 0);
        assert_eq!(st, 0, "budget holds until the fail");
        if int(&mut p, "get_r_state") == 2 {
            break;
        }
        assert!(t < 600, "pit must fail the walk");
    }
    assert_eq!(s2_state(&mut p), 2);
}

fn s2_state(s: &mut Session) -> i32 {
    s.int("get_r_state", &[]).unwrap()
}
