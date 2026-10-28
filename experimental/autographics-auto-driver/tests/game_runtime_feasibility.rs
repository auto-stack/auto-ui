//! PLAN-860 game-runtime feasibility tests (contract V-01/02/03 work scope).
//! Real Auto Session execution on the frozen G2/G3 scenes; no Rust gameplay
//! replacement. Run single-threaded: cargo test --locked --test
//! game_runtime_feasibility -- --test-threads=1
use autographics_auto_driver::{
    research::game_runtime::{self as gr, Candidate, Route},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};

fn assert_open(s: &Session) {
    assert!(!s.closed(), "session closed unexpectedly");
}

fn place(s: &mut Session, x: f32, y: f32) {
    s.call(
        "ag_set_local",
        &[Int(0), Int(0), Float(x)],
        ResultKind::Unit,
    )
    .unwrap();
    s.call(
        "ag_set_local",
        &[Int(0), Int(1), Float(y)],
        ResultKind::Unit,
    )
    .unwrap();
}

fn sample(s: &mut Session) -> (f32, f32, f32, i32) {
    let x = s.float("probe_x", &[]).unwrap();
    let y = s.float("probe_y", &[]).unwrap();
    let z = s.float("probe_z", &[]).unwrap();
    let g = s.int("probe_grounded", &[]).unwrap();
    (x, y, z, g)
}

fn tick(s: &mut Session, step: usize) {
    s.call(
        "_ag_tick",
        &[Int(step as i32), Float(gr::DT)],
        ResultKind::Unit,
    )
    .unwrap();
}

// ---------------------------------------------------------------- routes ---

#[test]
fn route_files_bind_the_frozen_grounded_rule() {
    for source in [gr::G2_ROUTE_A, gr::G3_ROUTE_A] {
        assert!(source.contains(gr::GROUNDED_RULE_A), "frozen rule drifted");
    }
    assert!(gr::G2_ROUTE_B.contains("agp_move(0,dx,dy,0.0,0.002)"));
    assert!(gr::G3_ROUTE_B.contains("agp_move(0,dx,dy,dz,0.002)"));
    assert!(gr::G2_ROUTE_B.contains("grounded=agp_grounded(q)"));
    assert!(gr::PROBE_UNIT.contains("_ag_motion_two(2,node,0,ox,oy,oz,0.0, -0.01,0.0)"));
}

fn run_route(route: Route, candidate: Candidate) -> gr::RouteReport {
    let user = gr::route_user(route, candidate);
    let name = gr::route_name(route, candidate);
    let rows = if route == Route::G2 {
        gr::g2_rows()
    } else {
        gr::g3_rows()
    };
    let mut s = gr::session_for(user, name, candidate, &rows, false).unwrap();
    let reads: Vec<&str> = if candidate == Candidate::B {
        vec!["scans", "support"]
    } else {
        vec![]
    };
    let steps = if route == Route::G2 {
        gr::G2_STEPS
    } else {
        gr::G3_STEPS
    };
    let samples = gr::run_steps(&mut s, steps, &reads).unwrap();
    assert!(
        samples.iter().all(|s| s.credits <= 32),
        "{candidate:?} credits over cap"
    );
    if candidate == Candidate::B {
        assert!(
            samples.iter().all(|s| s.scans >= 1 && s.scans <= 2),
            "{candidate:?} scan count contract drift"
        );
    }
    assert_open(&s);
    gr::route_report(route, &samples)
}

#[test]
fn route_g2_both_candidates_reach_all_triggers_without_violations() {
    for candidate in [Candidate::A, Candidate::B] {
        let rep = run_route(Route::G2, candidate);
        assert!(
            rep.coins_reached.iter().all(|c| *c),
            "{candidate:?} coins {:?}",
            rep.coin_steps
        );
        assert!(rep.goal_reached, "{candidate:?} goal missed");
        assert_eq!(rep.enemy_contact_step, None, "{candidate:?} enemy contact");
        assert_eq!(rep.pit_step, None, "{candidate:?} pit fall");
        assert!(rep.hazard_contact_step.is_none());
    }
}

#[test]
fn route_g3_both_candidates_reach_all_triggers_without_violations() {
    for candidate in [Candidate::A, Candidate::B] {
        let rep = run_route(Route::G3, candidate);
        assert!(
            rep.coins_reached.iter().all(|c| *c),
            "{candidate:?} coins {:?}",
            rep.coin_steps
        );
        assert!(rep.goal_reached, "{candidate:?} goal missed");
        assert_eq!(rep.hazard_contact_step, None, "{candidate:?} hazard touch");
        assert_eq!(rep.pit_step, None, "{candidate:?} pit fall");
    }
}

#[test]
fn route_trace_stays_within_frozen_bounds() {
    let mut s = gr::session_for(
        gr::route_user(Route::G2, Candidate::A),
        gr::route_name(Route::G2, Candidate::A),
        Candidate::A,
        &gr::g2_rows(),
        false,
    )
    .unwrap();
    let samples = gr::run_steps(&mut s, gr::G2_STEPS, &[]).unwrap();
    assert!(samples.len() <= 1800);
    let serialized = serde_json::to_vec(&samples).unwrap().len();
    assert!(
        serialized <= 5 * 1024 * 1024,
        "trace {serialized}B over 5MiB"
    );
}

// ------------------------------------------------------------ semantics ----

/// Semantic-case controller. Scenario inputs are the frozen constants
/// (j1/j2 jump edges, [run0,run1] right-input window). Candidate A carries the
/// exact frozen grounded rule; candidate B the geometric support getter.
fn sc_source(candidate: Candidate, j1: usize, j2: usize, run0: usize, run1: usize) -> String {
    let move_call = match candidate {
        Candidate::A => " let q=ag_move_and_slide(0,0,dx,dy,0.0,0.002)",
        Candidate::B => " let q=agp_move(0,dx,dy,0.0,0.002)",
    };
    let rule = match candidate {
        Candidate::A => {
            "
 let st=ag_motion_status(q)
 let cc=ag_motion_contacts(q)
 let uy=ag_motion_unapplied(q,1)
 let ny=ag_motion_normal(q,1)
 //grounded-rule
grounded=0
 if st==1 {grounded=1}
 if st==2 {grounded=1}
 if cc<1 {grounded=0}
 if dy>=0.0 {grounded=0}
 if uy>=0.0 {grounded=0}"
        }
        Candidate::B => {
            "
 grounded=agp_grounded(q)"
        }
    };
    format!(
        "var vy=0.0\nvar grounded=0\nvar jprev=0\nvar stepn=0\nfn app_start(){{}}\nfn app_update(dt float){{\n stepn=stepn+1\n let s=stepn\n var jp=0\n if s=={j1} {{jp=1}}\n if s=={j2} {{jp=1}}\n if jp==1 {{if jprev==0 {{if grounded==1 {{vy=8.0}}}}}}\n jprev=jp\n vy=vy-20.0*dt\n var vx=0.0\n if s>={run0} {{if s<={run1} {{vx=4.0}}}}\n let dx=vx*dt\n let dy=vy*dt\n{move_call}\n{rule}\n if grounded==1 {{vy=0.0}}\n}}\n"
    )
}

fn sc_session(
    candidate: Candidate,
    rows: &[gr::Row],
    j1: usize,
    j2: usize,
    run0: usize,
    run1: usize,
) -> Session {
    let src = sc_source(candidate, j1, j2, run0, run1);
    gr::session_for(&src, "sc.at", candidate, rows, false).unwrap()
}

fn char2() -> gr::Row {
    gr::Row(2, 2, [1., -0.4, -0.5, 0., 0.4, 0.5, 0.])
}

fn sc_ground() -> Vec<gr::Row> {
    vec![char2(), gr::Row(2, 2, [1., -4., -1., 0., 12., 0., 0.])]
}

#[test]
fn sc01_stand_and_walk_keeps_ground_support() {
    for candidate in [Candidate::A, Candidate::B] {
        let mut s = sc_session(candidate, &sc_ground(), 0, 0, 1, 60);
        place(&mut s, 0.0, 0.5);
        for step in 1..=60 {
            tick(&mut s, step);
            let (x, y, _, g) = sample(&mut s);
            assert_eq!(g, 1, "{candidate:?} step {step} lost ground");
            assert!((y - 0.5).abs() < 0.01, "{candidate:?} step {step} y={y}");
            assert!((x - (step as f32) * 4.0 * gr::DT).abs() < 0.01);
        }
        assert_open(&s);
    }
}

fn sc_corner_rows() -> Vec<gr::Row> {
    vec![
        char2(),
        gr::Row(2, 2, [1., -2., -1., 0., 8., 0., 0.]),
        gr::Row(2, 2, [1., 3., 0., 0., 5., 4., 0.]),
    ]
}

#[test]
fn sc02_floor_then_wall_corner_stays_grounded_and_naive_normal_fails() {
    for candidate in [Candidate::A, Candidate::B] {
        let mut s = sc_session(candidate, &sc_corner_rows(), 0, 0, 1, 120);
        place(&mut s, 2.0, 0.5);
        let mut jam_step = 0;
        let mut jam_grounded = 0;
        let mut last_x = 0.0;
        for step in 1..=120 {
            tick(&mut s, step);
            let (x, _, _, g) = sample(&mut s);
            if x == last_x && jam_step == 0 && step > 1 {
                jam_step = step;
                jam_grounded = g;
            }
            last_x = x;
        }
        assert!(jam_step > 0, "{candidate:?} never reached the wall");
        assert_eq!(
            jam_grounded, 1,
            "{candidate:?} corner lost grounding at {jam_step}"
        );
        assert_open(&s);
    }
    // MUT-2: the naive last-normal-only rule must RED on the same gate.
    let mutated = sc_source(Candidate::A, 0, 0, 1, 120).replace(
        " if cc<1 {grounded=0}\n if dy>=0.0 {grounded=0}\n if uy>=0.0 {grounded=0}",
        " if cc<1 {grounded=0}\n if ny!=1.0 {grounded=0}",
    );
    assert_ne!(
        mutated,
        sc_source(Candidate::A, 0, 0, 1, 120),
        "mutation did not apply"
    );
    let mut s = gr::session_for(
        &mutated,
        "sc-mut2.at",
        Candidate::A,
        &sc_corner_rows(),
        false,
    )
    .unwrap();
    place(&mut s, 2.0, 0.5);
    let mut jam_grounded = 1;
    let mut last_x = 0.0;
    for step in 1..=120 {
        tick(&mut s, step);
        let (x, _, _, g) = sample(&mut s);
        if x == last_x && step > 1 {
            jam_grounded = g;
        }
        last_x = x;
    }
    assert_eq!(
        jam_grounded, 0,
        "MUT-2 corner counterexample missed (false green)"
    );
}

fn sc_edge_rows() -> Vec<gr::Row> {
    vec![char2(), gr::Row(2, 2, [1., 0., -1., 0., 8., 0., 0.])]
}

#[test]
fn sc04_sc05_edge_leave_and_air_jump_denied_with_stale_mutation_red() {
    for candidate in [Candidate::A, Candidate::B] {
        let mut s = sc_session(candidate, &sc_edge_rows(), 130, 0, 1, 160);
        place(&mut s, 2.0, 0.5);
        let mut leave_step = 0;
        let mut y130 = 0.0;
        let mut y131 = 0.0;
        for step in 1..=160 {
            tick(&mut s, step);
            let (_, y, _, g) = sample(&mut s);
            if g == 0 && leave_step == 0 {
                leave_step = step;
            }
            if step == 130 {
                y130 = y;
            }
            if step == 131 {
                y131 = y;
            }
        }
        assert!(
            (70..110).contains(&leave_step),
            "{candidate:?} left at {leave_step}, support window drifted"
        );
        assert!(
            y131 < y130,
            "{candidate:?} air jump was applied at the mid-air edge ({y130}->{y131})"
        );
        assert_open(&s);
    }
    // MUT-1: sticky grounded must allow the mid-air jump (named red); the
    // unmutated loop above is the green side of the same oracle.
    let mutated = sc_source(Candidate::A, 130, 0, 1, 160).replace(
        "grounded=0\n if st==1 {grounded=1}\n if st==2 {grounded=1}\n if cc<1 {grounded=0}\n if dy>=0.0 {grounded=0}\n if uy>=0.0 {grounded=0}",
        " if st==1 {grounded=1}\n if st==2 {grounded=1}",
    );
    assert_ne!(
        mutated,
        sc_source(Candidate::A, 130, 0, 1, 160),
        "mutation did not apply"
    );
    let mut s =
        gr::session_for(&mutated, "sc-mut1.at", Candidate::A, &sc_edge_rows(), false).unwrap();
    place(&mut s, 2.0, 0.5);
    let mut y129 = 0.0;
    let mut y130 = 0.0;
    let mut g131 = 0;
    for step in 1..=160 {
        tick(&mut s, step);
        let (_, y, _, g) = sample(&mut s);
        if step == 129 {
            y129 = y;
        }
        if step == 130 {
            y130 = y;
        }
        if step == 131 {
            g131 = g;
        }
    }
    assert_eq!(g131, 1, "MUT-1 did not keep stale grounding");
    // The stale jump lifts exactly on the edge step (the sticky clamp then
    // zeroes vy again), so the counterexample is the step-130 rise itself.
    assert!(
        y130 > y129,
        "MUT-1 air jump counterexample missed (false green)"
    );
}

#[test]
fn sc03_ceiling_contact_is_not_ground() {
    let rows = vec![
        char2(),
        gr::Row(2, 2, [1., -4., -1., 0., 12., 0., 0.]),
        gr::Row(2, 2, [1., -4., 1.5, 0., 12., 2.5, 0.]),
    ];
    for candidate in [Candidate::A, Candidate::B] {
        let mut s = sc_session(candidate, &rows, 10, 0, 0, 0);
        place(&mut s, 0.0, 0.5);
        let mut hit_ceiling = false;
        for step in 1..=60 {
            tick(&mut s, step);
            let (_, y, _, g) = sample(&mut s);
            if y > 0.95 {
                hit_ceiling = true;
            }
            if y > 0.9 {
                assert_eq!(g, 0, "{candidate:?} step {step} ceiling counted as ground");
            }
            assert!(y <= 1.0, "{candidate:?} penetrated ceiling: y={y}");
        }
        assert!(hit_ceiling, "{candidate:?} never reached the ceiling");
        assert_open(&s);
    }
}

#[test]
fn sc06_initial_overlap_reports_status3_without_motion() {
    let rows = vec![char2(), gr::Row(2, 2, [1., -1., -1., 0., 1., 1., 0.])];
    for candidate in [Candidate::A, Candidate::B] {
        let mut s = sc_session(candidate, &rows, 0, 0, 0, 0);
        place(&mut s, 0.0, 0.0);
        for step in 1..=5 {
            tick(&mut s, step);
            let (x, y, _, g) = sample(&mut s);
            assert_eq!(g, 0, "{candidate:?} overlap counted as grounded");
            assert_eq!(x, 0.0);
            assert_eq!(y, 0.0);
        }
        let status = s.int("_ag_status", &[]).unwrap();
        assert_eq!(status, 0, "{candidate:?} overlap must not be a Fault");
        assert_open(&s);
    }
}

#[test]
fn sc07_precision_stop_reverts_and_support_stays_resolvable() {
    // Zero-progress PrecisionStop repro: at x=1e8 the f32 ulp is 8, so the
    // requested 1-unit advance rounds to zero (motion_geometry precedent).
    let rows = vec![gr::Row(2, 3, [1., -0.4, -0.5, -0.4, 0.4, 0.5, 0.4])];
    let mut s = sc_session(Candidate::A, &rows, 0, 0, 1, 1);
    place(&mut s, 0.0, 1000000.0);
    // x=3e6: f32 ulp is 0.25 there, so the 0.067 per-step advance rounds to
    // zero while the +/-0.4 box stays non-degenerate (1e8 would fault shape 15).
    s.call(
        "ag_set_local",
        &[Int(0), Int(0), Float(3000000.0)],
        ResultKind::Unit,
    )
    .unwrap();
    tick(&mut s, 1);
    let ms = s.int("probe_ms", &[]).unwrap();
    assert_eq!(ms, 4, "expected PrecisionStop");
    let (x, y, _, g_a) = sample(&mut s);
    assert_eq!(
        x, 3000000.0,
        "position must revert to the action-start world"
    );
    assert_eq!(y, 1000000.0);
    assert_eq!(
        g_a, 0,
        "no support contact exists within this step (measured)"
    );

    let mut sb = sc_session(Candidate::B, &rows, 0, 0, 1, 1);
    place(&mut sb, 0.0, 1000000.0);
    sb.call(
        "ag_set_local",
        &[Int(0), Int(0), Float(3000000.0)],
        ResultKind::Unit,
    )
    .unwrap();
    tick(&mut sb, 1);
    assert_eq!(sb.int("probe_ms", &[]).unwrap(), 4);
    let support = sb.int("probe_support", &[]).unwrap();
    assert_eq!(support, -1, "no floor exists under the restored position");
    let scans = sb.int("probe_scans", &[]).unwrap();
    assert_eq!(
        scans, 2,
        "B support mechanism must still run on revert steps"
    );
    let (_, _, _, g_b) = sample(&mut sb);
    assert_eq!(
        g_b, 0,
        "correct no-support determination at the restored position"
    );
}

#[test]
fn sc08_bad_parameters_classify_as_faults_and_keep_last_good() {
    let rows = sc_ground();
    for (script, code) in [
        (
            "var grounded=0\nvar stepn=0\nfn app_start(){}\nfn app_update(dt float){let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.0)}\n",
            7,
        ),
        (
            "var grounded=0\nvar stepn=0\nfn app_start(){}\nfn app_update(dt float){let q=ag_move_and_slide(0,0,1.0,0.0,1.0,0.002)}\n",
            7,
        ),
        (
            "var grounded=0\nvar stepn=0\nfn app_start(){}\nfn app_update(dt float){let q=ag_move_and_slide(0,0,(0.0).ln(),0.0,0.0,0.002)}\n",
            12,
        ),
    ] {
        let mut s = gr::session_for(script, "sc-bad.at", Candidate::A, &rows, false).unwrap();
        place(&mut s, 0.0, 0.5);
        let before = sample(&mut s);
        tick(&mut s, 1);
        let status = s.int("_ag_status", &[]).unwrap();
        assert_eq!(status, code, "fault classification for {code}");
        let after = sample(&mut s);
        assert_eq!(before, after, "last-good position changed after fault");
        tick(&mut s, 2);
        let status2 = s.int("_ag_status", &[]).unwrap();
        assert_eq!(status2, code, "sticky fault must not re-run app_update");
    }
}

#[test]
fn sc09_budget_and_credit_cap_classify_without_partial_frames() {
    let rows = sc_ground();
    let burner =
        "var grounded=0\nvar stepn=0\nfn app_start(){}\nfn app_update(dt float){for i in 0..400000 {ag_set_local(0,0,1.0)}}\n";
    let mut s = gr::session_for(burner, "sc-budget.at", Candidate::A, &rows, false).unwrap();
    let err = s
        .call("_ag_tick", &[Int(1), Float(gr::DT)], ResultKind::Unit)
        .unwrap_err();
    assert!(
        err.contains("budget") || err.contains("steps"),
        "unexpected budget error: {err}"
    );
    assert!(s.closed(), "budget exhaustion must close the VM");

    let spender = "var grounded=0\nvar stepn=0\nfn app_start(){}\nfn app_update(dt float){for i in 0..40 {let q=ag_move_and_slide(0,0,0.0,0.0,0.0,0.002)}}\n";
    let mut s = gr::session_for(spender, "sc-credit.at", Candidate::A, &rows, false).unwrap();
    tick(&mut s, 1);
    let status = s.int("_ag_status", &[]).unwrap();
    assert_eq!(status, 10, "credit overflow classification");
}

#[test]
fn sc10_jump_lands_on_wall_top_and_support_classifies() {
    let rows = vec![
        char2(),
        gr::Row(2, 2, [1., -4., -1., 0., 12., 0., 0.]),
        gr::Row(2, 2, [1., 6., 0., 0., 8., 1.25, 0.]),
    ];
    for candidate in [Candidate::A, Candidate::B] {
        let mut s = sc_session(candidate, &rows, 39, 0, 1, 90);
        place(&mut s, 2.0, 0.5);
        let mut landed_top = false;
        for step in 1..=120 {
            tick(&mut s, step);
            let (x, y, _, g) = sample(&mut s);
            if x > 6.4 && (y - 1.75).abs() < 0.05 && g == 1 {
                landed_top = true;
            }
        }
        assert!(landed_top, "{candidate:?} never landed on the wall top");
        assert_open(&s);
    }
}

// ------------------------------------------------------------- 128 load ----

#[test]
fn load_128_wide_and_deep_blocker_last_both_candidates() {
    for deep in [false, true] {
        for candidate in [Candidate::A, Candidate::B] {
            let rows = gr::load_rows(deep);
            let user = gr::load_source(candidate);
            let name = gr::load_name(candidate);
            let mut s = gr::session_for(user, name, candidate, &rows, deep).unwrap();
            let reads: Vec<&str> = if candidate == Candidate::B {
                vec!["scans", "support"]
            } else {
                vec![]
            };
            let all = gr::run_steps(&mut s, 35, &reads).unwrap();
            let samples: Vec<_> = all.into_iter().skip(5).collect();
            assert_eq!(samples.len(), 30);
            let visible = s.int("ag_visible", &[Int(127)]).unwrap();
            assert_eq!(visible, 1, "{candidate:?} deep={deep} load oracle failed");
            if candidate == Candidate::B {
                assert!(samples.iter().all(|s| s.scans == 2), "B scan count drifted");
                assert!(samples.iter().all(|s| s.support == -1));
            }
            let p50 = gr::percentile(samples.iter().map(|s| s.tick_ns).collect(), 0.5);
            let p95 = gr::percentile(samples.iter().map(|s| s.tick_ns).collect(), 0.95);
            assert!(p50 > 0 && p95 >= p50);
            eprintln!(
                "LOAD128 {} deep={} p50={}ns p95={}ns credits_max={} heap={}",
                format!("{candidate:?}").to_lowercase(),
                deep,
                p50,
                p95,
                samples.iter().map(|s| s.credits).max().unwrap(),
                s.heap_objects()
            );
        }
    }
}

#[test]
fn load_128_blocker_identity_is_the_last_iterated_node() {
    for deep in [false, true] {
        let rows = gr::load_rows(deep);
        let mut s = gr::session_for(
            gr::load_source(Candidate::A),
            gr::load_name(Candidate::A),
            Candidate::A,
            &rows,
            deep,
        )
        .unwrap();
        tick(&mut s, 1);
        // The tick source records ag_motion_node(q)==127 into stepn (a host
        // call cannot pass the 6-argument move under the profile cap).
        let node = s.int("probe_step", &[]).unwrap();
        assert_eq!(
            node, 127,
            "nearest blocker must win over iteration order (deep={deep})"
        );
    }
}

// ------------------------------------------------- state, owner, budget ----

#[test]
fn route_state_owner_and_release_across_20_cycles() {
    let mut s = gr::session_for(
        gr::route_user(Route::G2, Candidate::B),
        gr::route_name(Route::G2, Candidate::B),
        Candidate::B,
        &gr::g2_rows(),
        false,
    )
    .unwrap();
    let reads = vec!["scans", "support"];
    gr::run_steps(&mut s, 60, &reads).unwrap();
    let heap = s.heap_objects();
    for cycle in 61..=80 {
        gr::run_steps(&mut s, 1, &reads).unwrap();
        assert_eq!(s.heap_objects(), heap, "cycle {cycle} grew heap objects");
    }
    let alive = s.owner_probe();
    s.close();
    assert!(!alive(), "probe unit must release the VM on close");
}

#[test]
fn candidate_b_state_stays_within_declared_scalars() {
    let mut s = gr::session_for(
        gr::route_user(Route::G3, Candidate::B),
        gr::route_name(Route::G3, Candidate::B),
        Candidate::B,
        &gr::g3_rows(),
        false,
    )
    .unwrap();
    gr::run_steps(&mut s, 30, &["scans", "support"]).unwrap();
    let scalars = s.globals_with_prefix("_agp_");
    assert!(
        scalars <= 6,
        "probe-unit scalar globals {scalars} exceed the declared contract cap"
    );
    s.close();
}

// ------------------------------------------------- host presentation (V-04) --

#[test]
fn ht_g2_host_publication_binds_trace_pixels_and_owners() {
    let report = gr::host_case::run(Route::G2, None).unwrap();
    assert_eq!(report["completed"], serde_json::json!(true));
    assert_eq!(report["owner_cycles"], serde_json::json!(20));
    eprintln!("HT-G2 {}", report);
}

#[test]
fn ht_g3_host_publication_binds_trace_pixels_and_owners() {
    let report = gr::host_case::run(Route::G3, None).unwrap();
    assert_eq!(report["completed"], serde_json::json!(true));
    eprintln!("HT-G3 {}", report);
}

#[test]
fn ht_real_mutations_are_rejected_by_the_same_oracles() {
    // MUT-3 (dropped integration output): the numeric oracle must go red.
    let err = gr::host_case::run(Route::G2, Some(("vy=vy-20.0*dt", "vy=vy-0.0*dt"))).unwrap_err();
    assert!(err.contains("numeric oracle"), "{err}");
    // MUT-4 (stale/static frames): freezing the input diverges from the
    // frozen expectation, so the same frozen-oracle family goes red (the
    // numeric checkpoint fires before the movement check).
    let err = gr::host_case::run(Route::G2, Some(("if s<=383 {vx=4.0}", "if s<=0 {vx=4.0}")))
        .unwrap_err();
    assert!(err.contains("oracle:"), "{err}");
}
