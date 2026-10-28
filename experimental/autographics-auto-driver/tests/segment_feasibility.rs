include!("support/segment.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
#[test]
fn segment_candidate_closed_nearest_snapshot_and_weighted_budget() {
    let update="let t=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); ag_set_local(0,0,20.0); if ag_cast_hit(t)!=1 {observed=90}; if ag_cast_node(t)!=0 {observed=91}; if ag_cast_t(t)!=0.25 {observed=92}; if ag_cast_point(t,0)!= -1.0 {observed=93}; if ag_cast_normal(t,0)!= -1.0 {observed=94}";
    let mut s = scene(update, &[(2, 3, BOX), (2, 3, BOX)]);
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}
#[test]
fn segment_candidate_128_wide_two_cast_ten_get_six_aabb() {
    let update="let a=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); let h=ag_cast_hit(a); let n=ag_cast_node(a); let t=ag_cast_t(a); let p=ag_cast_point(a,0); let f=ag_cast_normal(a,0); ag_set_local(0,0,0.5); let b=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); let h2=ag_cast_hit(b); let n2=ag_cast_node(b); let t2=ag_cast_t(b); let p2=ag_cast_point(b,0); let f2=ag_cast_normal(b,0); for i in 0..6 {let q=ag_aabb_min(0,0)}";
    let mut s = scene(update, &vec![(2, 3, BOX); 128]);
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}
#[test]
fn segment_candidate_16_wide_accepted_control() {
    let update="let a=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); let h=ag_cast_hit(a); let n=ag_cast_node(a); let t=ag_cast_t(a); let p=ag_cast_point(a,0); let f=ag_cast_normal(a,0); ag_set_local(0,0,0.5); let b=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); let h2=ag_cast_hit(b); let n2=ag_cast_node(b); let t2=ag_cast_t(b); let p2=ag_cast_point(b,0); let f2=ag_cast_normal(b,0); for i in 0..6 {let q=ag_aabb_min(0,0)}";
    let mut s = scene(update, &vec![(2, 3, BOX); 16]);
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}

#[test]
fn segment_operation_diagnosis() {
    for (name,update) in [
 ("empty", ""),
 ("single", "let a=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0)"),
 ("double", "let a=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0); let b=ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0)"),
 ("derive", "ag_set_local(0,0,0.5)") ] {
 let mut s=scene(update,&vec![(2,3,BOX);128]); let before=s.yielded;
 let r=s.call("_ag_tick",&[Int(1),Float(0.016)],ResultKind::Unit);
 eprintln!("DIAG {name} result={r:?} slices={}",s.yielded-before);
 }
}

#[test]
fn segment_128_wide_and_deep_two_group_writes_exact_credits() {
    let update="ag_set_local(0,0,2.0); let a=ag_cast_segment(3,-1,0,0.0,0.0,0.0,8.0,0.0,0.0); let h=ag_cast_hit(a); let n=ag_cast_node(a); let t=ag_cast_t(a); let p=ag_cast_point(a,0); let f=ag_cast_normal(a,0); ag_set_local(0,0,4.0); let b=ag_cast_segment(3,-1,0,0.0,0.0,0.0,8.0,0.0,0.0); let h2=ag_cast_hit(b); let n2=ag_cast_node(b); let t2=ag_cast_t(b); let p2=ag_cast_point(b,0); let f2=ag_cast_normal(b,0); for i in 0..6 {let q=ag_aabb_min(127,0)}; if h!=1 || h2!=1 || t!=0.125 || t2!=0.375 || p!=1.0 || p2!=3.0 {observed=99}";
    for deep in [false, true] {
        let mut rows = vec![(2, 3, BOX); 128];
        for i in 0..if deep { 126 } else { 1 } {
            rows[i] = (5, 3, [0.0; 7]);
        }
        let mut s = scene_guard(update, &rows, deep);
        let before = s.yielded;
        let r = s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit);
        eprintln!(
            "GROUP deep={deep} result={r:?} slices={}",
            s.yielded - before
        );
        r.unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
        assert_eq!(s.int("probe_credit", &[]).unwrap(), 32);
        assert_eq!(s.int("probe_token", &[]).unwrap(), 0);
    }
}

#[test]
fn segment_group_cost_diagnosis() {
    let mut rows = vec![(2, 3, BOX); 128];
    rows[0] = (5, 3, [0.0; 7]);
    for (name, update) in [
        ("derive", "ag_set_local(0,0,2.0)"),
        (
            "move_cast",
            "ag_set_local(0,0,2.0); let a=ag_cast_segment(3,-1,0,0.0,0.0,0.0,8.0,0.0,0.0)",
        ),
        (
            "cast",
            "let a=ag_cast_segment(3,-1,0,0.0,0.0,0.0,8.0,0.0,0.0)",
        ),
    ] {
        let mut s = scene_guard(update, &rows, false);
        let before = s.yielded;
        let r = s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit);
        eprintln!("GROUP COST {name} {r:?} slices={}", s.yielded - before);
    }
}
