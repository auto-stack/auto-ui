include!("support/segment.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
fn run(s: &mut Session) -> i32 {
    s.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}
#[test]
fn geometry_overflow_is_checked_before_nearest_pruning_and_filtered_nodes_skip_it() {
    let big = "99999999999999999999999999999999999999.0";
    let rows = [
        (2, 3, BOX),
        (2, 3, [1., f32::MAX, -1., -1., f32::MAX, 1., 1.]),
    ];
    for (ignore, expected) in [(-1, 9), (1, 0)] {
        let mut s=scene(&format!("ag_set_local(1,0,{big}); let q=ag_cast_segment(3,{ignore},0,-2.0,0.0,0.0,2.0,0.0,0.0)"),&rows);
        assert_eq!(run(&mut s), expected);
    }
    let mut s=scene("let q=ag_cast_segment(3,-1,0,0.0,0.0,0.0,0.000000000000000000000000000000000000000000001,0.0,0.0)",&[(2,3,[1.,1.,-1.,-1.,2.,1.,1.])]);
    assert_eq!(run(&mut s), 12);
    let mut s = scene(
        "let q=ag_cast_segment(3,-1,0,0.0,0.0,0.0,0.0,0.0,0.0); observed=ag_cast_hit(q)",
        &[(2, 3, [1., 1., -1., -1., 2., 1., 1.])],
    );
    assert_eq!(run(&mut s), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}
#[test]
fn group_cache_switches_parent_and_final_writes_publish_latest_world() {
    let update="ag_set_local(0,0,2.0); ag_set_local(1,0,-3.0); if ag_world(2,0)!=2.0 {observed=1}; if ag_world(3,0)!= -3.0 {observed=2}; ag_set_local(3,0,5.0)";
    let rows = [(5, 3, [0.; 7]), (5, 3, [0.; 7]), (2, 3, BOX), (2, 3, BOX)];
    let mut s = scene_custom(update, &rows, false, |mut s| {
        s.push_str("\nfn probe_reparent(){_ag_parents[1]= -1; _ag_parents[2]=0; _ag_parents[3]=1; _ag_dirty=1}\n");
        s
    });
    s.call("probe_reparent", &[], ResultKind::Unit).unwrap();
    assert_eq!(run(&mut s), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
    // Read the same non-deriving field consumed by Play; ag_world could repair
    // a missing callback-final derive and hide a stale published frame.
    assert_eq!(s.float("_ag_field", &[Int(3), Int(0)]).unwrap(), 2.);
    let mut missing_final = scene_custom(update, &rows, false, |s| {
        let begin = s.find("fn _ag_tick(").unwrap();
        let old = "; if _ag_dirty==1 {_ag_derive()}; _ag_dirty=0";
        assert!(s[begin..].contains(old));
        let mut changed = format!(
            "{}{}",
            &s[..begin],
            s[begin..].replace(old, "; _ag_dirty=0")
        );
        changed.push_str("\nfn probe_reparent(){_ag_parents[1]= -1; _ag_parents[2]=0; _ag_parents[3]=1; _ag_dirty=1}\n");
        changed
    });
    missing_final
        .call("probe_reparent", &[], ResultKind::Unit)
        .unwrap();
    assert_eq!(run(&mut missing_final), 0);
    assert_ne!(
        missing_final.float("_ag_field", &[Int(3), Int(0)]).unwrap(),
        2.
    );
}
#[test]
fn snapshot_then_new_query_changes_result_without_changing_old_snapshot() {
    let update="let a=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); let before=ag_cast_point(a,0); ag_set_local(0,0,10.0); if ag_cast_point(a,0)!=before {observed=1}; let b=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); if ag_cast_hit(b)!=0 {observed=2}; if ag_cast_point(b,0)!=2.0 {observed=3}";
    let mut s = scene(update, &[(2, 3, BOX)]);
    assert_eq!(run(&mut s), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}
#[test]
fn empty_filtered_zero_signed_parallel_and_degenerate_point() {
    for (row, cast, expected) in [
        ([1., 0., 0., 0., 0., 0., 0.], "0.0,-0.0,0.0,0.0,0.0,0.0", 1),
        ([1., 0., 0., 0., 0., 0., 0.], "1.0,-0.0,0.0,1.0,0.0,0.0", 0),
        (BOX, "-2.0,1.0,0.0,2.0,1.0,0.0", 1),
    ] {
        let mut s = scene(
            &format!("let q=ag_cast_segment(3,-1,0,{cast}); observed=ag_cast_hit(q)"),
            &[(2, 3, row)],
        );
        assert_eq!(run(&mut s), 0);
        assert_eq!(s.int("probe_observed", &[]).unwrap(), expected);
    }
    let mut s = scene(
        "let q=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); observed=ag_cast_hit(q)",
        &[],
    );
    assert_eq!(run(&mut s), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}

#[test]
fn repeated_queries_keep_one_fixed_result_slot_without_heap_growth() {
    let mut s = scene(
        "let q=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); observed=ag_cast_hit(q)",
        &[(2, 3, BOX)],
    );
    let initial = s.heap_objects();
    for n in 1..65 {
        s.call("_ag_tick", &[Int(n), Float(1. / 60.)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(s.int("probe_observed", &[]).unwrap(), 1);
        assert_eq!(s.heap_objects(), initial);
    }
}

#[test]
fn outside_entry_keeps_its_face_when_positive_fraction_underflows_to_zero() {
    let tiny = "-0.000000000000000000000000000000000000000000001";
    let big = "100000000000000000000000000000000000000.0";
    for space in [2, 3] {
        for axis in 0..space as usize {
            let mut row = BOX;
            row[axis + 1] = 0.;
            if space == 2 {
                row[3] = 0.;
                row[6] = 0.;
            }
            let mut a = ["0.0"; 3];
            let mut b = ["0.0"; 3];
            a[axis] = tiny;
            b[axis] = big;
            let update=format!("let q=ag_cast_segment({space},-1,0,{},{},{},{},{},{}); if ag_cast_hit(q)!=1 || ag_cast_t(q)!=0.0 || ag_cast_normal(q,{axis})!= -1.0 {{observed=1}}",a[0],a[1],a[2],b[0],b[1],b[2]);
            let mut s = scene(&update, &[(if space == 2 { 0 } else { 2 }, space, row)]);
            assert_eq!(run(&mut s), 0);
            assert_eq!(
                s.int("probe_observed", &[]).unwrap(),
                0,
                "outside axis={axis}"
            );
            a[axis] = "0.0";
            let update=format!("let q=ag_cast_segment({space},-1,0,{},{},{},{},{},{}); observed=ag_cast_hit(q); if ag_cast_normal(q,{axis})!=0.0 {{observed=2}}",a[0],a[1],a[2],b[0],b[1],b[2]);
            let mut s = scene(&update, &[(if space == 2 { 0 } else { 2 }, space, row)]);
            assert_eq!(run(&mut s), 0);
            assert_eq!(
                s.int("probe_observed", &[]).unwrap(),
                1,
                "boundary axis={axis}"
            );
        }
    }
}
