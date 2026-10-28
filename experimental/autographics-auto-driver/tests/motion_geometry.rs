include!("support/motion.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
fn run(script: &str, rows: &[(i32, i32, [f32; 7])], error: i32) -> Session {
    let mut s = scene_custom(script, rows, false, |s| s);
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), error, "{script}");
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0, "{script}");
    s
}
#[test]
fn statuses_clearance_direction_and_zero_displacement() {
    for dim in [2, 3] {
        let rows = [
            (2, dim, BOX),
            (2, dim, [1., 4., -100., -100., 4.2, 100., 100.]),
            (2, dim, [1., -100., 6., -100., 100., 6.2, 100.]),
        ];
        for (delta, status, contacts, x, y) in [
            ("10.0,10.0,0.0", 2, 2, 2.75, 4.75),
            ("10.0,4.0,0.0", 1, 1, 2.75, 4.),
            ("0.0,0.0,0.0", 0, 0, 0., 0.),
            ("-4.0,2.0,0.0", 0, 0, -4., 2.),
        ] {
            let script=format!("let q=ag_move_and_slide(0,0,{delta},0.25);if ag_motion_status(q)!={status} || ag_motion_contacts(q)!={contacts} || ag_motion_node(q)< -1 {{observed=1}}");
            let mut s = run(&script, &rows, 0);
            assert!((s.float("probe_direct_x", &[Int(0)]).unwrap() - x).abs() < 0.00001);
            assert!((s.float("probe_direct_y", &[Int(0)]).unwrap() - y).abs() < 0.00001);
        }
        run("let q=ag_move_and_slide(0,0,10.0,0.0,0.0,4.0);if ag_motion_status(q)!=1 || ag_motion_delta(q,0)!=0.0 {observed=1}",&rows,0);
        run("ag_set_local(0,0,3.0);let q=ag_move_and_slide(0,0,0.0,2.0,0.0,0.25);if ag_motion_status(q)!=0 || ag_motion_delta(q,1)!=2.0 {observed=1}",&rows,0);
        run("ag_set_local(0,0,4.0);let q=ag_move_and_slide(0,0,-10.0,0.0,0.0,0.25);if ag_motion_status(q)!=3 || ag_motion_node(q)!=1 || ag_motion_contacts(q)!=0 || ag_motion_delta(q,0)!=0.0 {observed=1}",&rows,0);
    }
}
#[test]
fn tokens_snapshot_budget_faults_and_shape() {
    let rows = [(2, 3, BOX)];
    for (code,error) in [("let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25);let h=ag_sweep_hit(q)",13),("let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25);let h=ag_motion_status(0)",13),("let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25);let b=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25)",10),("let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.0)",7),("let q=ag_move_and_slide(-1,0,1.0,0.0,0.0,0.25)",1),("let q=ag_move_and_slide(0,2,1.0,0.0,0.0,0.25)",7),("let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25);ag_set_local(0,0,7.0);if ag_motion_origin(q,0)!=1.0 || ag_motion_delta(q,0)!=1.0 || ag_motion_unapplied(q,0)!=0.0 {observed=1}",0)] {run(code,&rows,error);}
    run(
        "let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25)",
        &[(2, 3, [1., 0., -1., -1., 0., 1., 1.])],
        15,
    );
    run(
        "let q=ag_move_and_slide(0,0,10.0,0.0,0.0,0.25)",
        &[
            (2, 3, BOX),
            (2, 3, [1., 4., -100., -100., 4.2, 100., 100.]),
            (2, 3, [0.; 7]),
        ],
        9,
    );
    run("ag_set_local(0,0,100000000.0);let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25);if ag_motion_status(q)!=4 || ag_motion_delta(q,0)!=0.0 {observed=1}",&[(2,3,[1.,-16.,-16.,-16.,16.,16.,16.])],0);
}
#[test]
fn clean_leaf_axis_group_interleaving_keeps_direct_world_coherent() {
    let rows = [(5, 3, [0.; 7]), (5, 3, [0.; 7]), (2, 3, BOX), (2, 3, BOX)];
    for update in [
        "ag_set_local(0,0,2.0);ag_set_local(2,1,3.0);ag_set_local(1,1,5.0);ag_set_local(2,1,4.0)",
        "ag_set_local(2,1,3.0);ag_set_local(0,0,2.0);ag_set_local(1,1,5.0);ag_set_local(2,1,4.0)",
    ] {
        let mut s = run(update, &rows, 0);
        assert_eq!(s.float("probe_direct_x", &[Int(2)]).unwrap(), 2.);
        assert_eq!(s.float("probe_direct_y", &[Int(2)]).unwrap(), 4.);
        assert_eq!(s.float("probe_direct_y", &[Int(3)]).unwrap(), 0.);
    }
}

#[test]
fn sixty_four_motion_callbacks_keep_objects_and_release_vm() {
    let mut s = scene_custom(
        "let q=ag_move_and_slide(0,0,0.0,0.0,0.0,0.25);let h=ag_motion_status(q)",
        &[(2, 3, BOX)],
        false,
        |s| s,
    );
    let alive = s.owner_probe();
    let before = s.heap_objects();
    for i in 1..65 {
        s.call("_ag_tick", &[Int(i), Float(0.016)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.heap_objects(), before);
    }
    s.close();
    assert!(!alive());
}

#[test]
fn native_nan_inf_local_return_container_and_collapse() {
    for expression in [
        "(-1.0).sqrt()",
        "0.0.ln()",
        "0.0 - 0.0.ln()",
        "[(-1.0).sqrt()][0]",
    ] {
        run(
            &format!("let v={expression};let q=ag_move_and_slide(0,0,v,0.0,0.0,0.25)"),
            &[(2, 3, BOX)],
            12,
        );
    }
    run(
        "fn bad() float {return (-1.0).sqrt()};let q=ag_move_and_slide(0,0,bad(),0.0,0.0,0.25)",
        &[(2, 3, BOX)],
        12,
    );
    run(
        "ag_set_local(0,0,(-1.0).sqrt());let q=ag_move_and_slide(0,0,0.0,0.0,0.0,0.25)",
        &[(2, 3, BOX)],
        9,
    );
    run(
        "ag_set_local(0,0,100000000.0);let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25)",
        &[(2, 3, BOX)],
        15,
    );
}

#[test]
fn outside_tangent_closed_arrival_is_not_initial_boundary_leave() {
    for dim in [2, 3] {
        run("ag_set_local(0,0,3.0);ag_set_local(0,1,-20.0);let q=ag_move_and_slide(0,0,0.0,40.0,0.0,0.25);if ag_motion_status(q)!=1 || ag_motion_normal(q,1)!= -1.0 || ag_motion_origin(q,1)!= -11.25 {observed=1}",&[(2,dim,BOX),(2,dim,[1.,4.,-10.,-10.,4.2,10.,10.])],0);
    }
}

#[test]
fn six_axis_slide_normals_ties_and_three_plane_limit() {
    for a in 0..3 {
        for sign in [1., -1.] {
            let mut wall = [1., -100., -100., -100., 100., 100., 100.];
            wall[1 + a] = if sign > 0. { 4. } else { -4.2 };
            wall[4 + a] = if sign > 0. { 4.2 } else { -4. };
            let mut delta = [4.; 3];
            delta[a] = 10. * sign;
            let script=format!("let q=ag_move_and_slide(0,0,{:?},{:?},{:?},0.25);if ag_motion_status(q)!=1 || ag_motion_contacts(q)!=1 || ag_motion_normal(q,{a})!={:?} {{observed=1}}",delta[0],delta[1],delta[2],-sign);
            let mut vm = run(&script, &[(2, 3, BOX), (2, 3, wall)], 0);
            for axis in 0..3 {
                let getter = ["probe_direct_x", "probe_direct_y", "probe_direct_z"][axis];
                let expected = if axis == a { sign * 2.75 } else { 4. };
                assert!((vm.float(getter, &[Int(0)]).unwrap() - expected).abs() < 0.00001);
            }
        }
    }
    let rows = [
        (2, 3, BOX),
        (2, 3, [1., 4., -100., -100., 4.2, 100., 100.]),
        (2, 3, [1., -100., 6., -100., 100., 6.2, 100.]),
        (2, 3, [1., -100., -100., 8., 100., 100., 8.2]),
    ];
    run("let q=ag_move_and_slide(0,0,10.0,10.0,10.0,0.25);if ag_motion_status(q)!=2 || ag_motion_contacts(q)!=2 || ag_motion_origin(q,2)!=4.75 || ag_motion_normal(q,1)!= -1.0 {observed=1}",&rows,0);
    run("let q=ag_move_and_slide(0,0,10.0,10.0,10.0,0.25);if ag_motion_node(q)!=1 || ag_motion_normal(q,0)!= -1.0 {observed=1}",&[(2,3,BOX),(2,3,[1.,4.,4.,4.,4.2,4.2,4.2]),(2,3,[1.,4.,4.,4.,4.2,4.2,4.2])],0);
}
#[test]
fn exhausted_and_cross_callback_result_ids() {
    let mut vm = scene_custom(
        "let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25)",
        &[(2, 3, BOX)],
        false,
        |s| s,
    );
    vm.call("probe_exhaust", &[], ResultKind::Unit).unwrap();
    vm.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(vm.int("_ag_status", &[]).unwrap(), 14);
    assert_eq!(vm.float("probe_direct_x", &[Int(0)]).unwrap(), 0.);
    let mut vm=scene_custom("if ag_tick()==1 {observed=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25)} else {let h=ag_motion_status(observed)}",&[(2,3,BOX)],false,|s|s);
    for i in 1..3 {
        vm.call("_ag_tick", &[Int(i), Float(0.016)], ResultKind::Unit)
            .unwrap();
    }
    assert_eq!(vm.int("_ag_status", &[]).unwrap(), 13);
}

#[test]
fn visibility_filter_axis_validation_and_skin_quantization_controls() {
    run("ag_set_visible(0,0);ag_set_visible(1,0);let q=ag_move_and_slide(0,1,10.0,0.0,0.0,0.25);if ag_motion_status(q)!=0 || ag_motion_origin(q,0)!=10.0 {observed=1}",&[(2,3,BOX),(2,3,[1.,4.,-10.,-10.,4.2,10.,10.])],0);
    run(
        "let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25);let v=ag_motion_delta(q,3)",
        &[(2, 3, BOX)],
        2,
    );
    run(
        "let q=ag_move_and_slide(0,0,1.0,0.0,1.0,0.25)",
        &[(2, 2, BOX)],
        7,
    );
    run(
        "let q=ag_move_and_slide(0,0,1.0,0.0,0.0,0.25)",
        &[(5, 3, [0.; 7]), (2, 3, BOX)],
        6,
    );
    run("ag_set_local(0,0,-5.14);ag_set_local(0,1,64.86);let q=ag_move_and_slide(0,0,4.0,1.6,0.0,0.04);if ag_motion_status(q)!=4 || ag_motion_contacts(q)!=0 || ag_motion_node(q)!= -1 {observed=1}",&[(2,2,[1.,-5.,-5.,0.,5.,5.,0.]),(2,2,[1.,-0.1,-100.,0.,0.1,100.,0.]),(2,2,[1.,-70.,69.9,0.,30.,70.1,0.])],0);
}

#[test]
fn actual_parent_rounding_final_admission_and_removed_guard_counterexample() {
    let rows = [
        (5, 2, [0.; 7]),
        (2, 2, [1., -1., -1., 0., 1., 1., 0.]),
        (2, 2, [1., 7., -100., 0., 7.2, 100., 0.]),
    ];
    for mutate in [false, true, false] {
        let mut vm=scene_custom("ag_set_local(0,0,100000000.0);ag_set_local(1,0,-100000000.0);let q=ag_move_and_slide(1,0,10.0,0.0,0.0,0.25);observed=ag_motion_status(q)",&rows,false,|mut s|{if mutate {let rule="if status!=3 && inside==1 {status=4}";assert!(s.contains(rule));s=s.replace(rule,"if status!=3 && inside==1 {status=status}");}s.push_str("\nfn probe_root(){_ag_parents[2]= -1;_ag_segment_dx=1;_ag_dirty=1}\n");s});
        vm.call("probe_root", &[], ResultKind::Unit).unwrap();
        vm.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
            .unwrap();
        assert_eq!(vm.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(
            vm.int("probe_observed", &[]).unwrap(),
            if mutate { 1 } else { 4 }
        );
        assert_eq!(
            vm.float("probe_direct_x", &[Int(1)]).unwrap(),
            if mutate { 8. } else { 0. }
        );
    }
}
