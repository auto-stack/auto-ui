include!("support/sweep.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
#[test]
fn thickness_and_origin() {
    let mut s=scene("let t=ag_sweep_aabb(0,0,10.0,0.0,0.0); if ag_sweep_hit(t)!=1 || ag_sweep_node(t)!=1 || ag_sweep_t(t)!=0.3 || ag_sweep_origin(t,0)!=3.0 || ag_sweep_normal(t,0)!= -1.0 || ag_sweep_start_overlap(t)!=0 {observed=99};let c=ag_cast_segment(3,0,0,0.0,0.0,0.0,10.0,0.0,0.0);if ag_cast_t(c)!=0.4 {observed=98};ag_set_local(0,0,10.0);if ag_aabb_overlaps(0,1)!=0 {observed=97}",&[(2,3,BOX),(2,3,[1.,4.,-1.,-1.,4.2,1.,1.])]);
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}
#[test]
fn sweep_128_wide_deep_exact_budget() {
    for dim in [2, 3] {
        for deep in [false, true] {
            let mut rows = vec![(2, dim, BOX); 128];
            for i in 0..if deep { 126 } else { 4 } {
                rows[i] = (5, dim, [0.; 7]);
            }
            let update="ag_set_local(0,0,2.0); let t=ag_sweep_aabb(126,0,8.0,0.0,0.0); let h=ag_sweep_hit(t); let n=ag_sweep_node(t); let f=ag_sweep_t(t); let x=ag_sweep_origin(t,0); let y=ag_sweep_origin(t,1); let nx=ag_sweep_normal(t,0); let ny=ag_sweep_normal(t,1); let o=ag_sweep_start_overlap(t); ag_set_local(0,0,4.0); for i in 0..8 {let a=ag_aabb_min(127,0)}; if h!=1 || f!=0.0 || o!=1 {observed=98}";
            let mut s = scene_guard(update, &rows, deep);
            let before = s.yielded;
            let result = s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit);
            eprintln!(
                "SWEEP dim={dim} deep={deep} result={result:?} slices={}",
                s.yielded - before
            );
            result.unwrap();
            assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
            assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
            assert_eq!(s.int("probe_credit", &[]).unwrap(), 32);
            assert_eq!(s.float("probe_direct_x", &[Int(127)]).unwrap(), 4.);
        }
    }
}

#[test]
fn sweep_128_diagonal_and_late_axis_controls() {
    for dim in [2, 3] {
        for mode in ["diagonal", "late-miss", "decreasing"] {
            let mut rows = vec![(2, dim, BOX); 128];
            rows[0] = (5, dim, [0.; 7]);
            for i in 1..128 {
                if i == 126 {
                    continue;
                };
                let x = if mode == "decreasing" {
                    500. - i as f32
                } else {
                    0.
                };
                rows[i].2 = [
                    1.,
                    x - 1.,
                    x - 1.,
                    if mode == "late-miss" { 3000. } else { x - 1. },
                    x + 1.,
                    x + 1.,
                    if mode == "late-miss" { 3002. } else { x + 1. },
                ];
                if dim == 2 && mode == "late-miss" {
                    rows[i].2[2] = 3000.;
                    rows[i].2[5] = 3002.;
                }
            }
            let z = if dim == 3 {
                "ag_set_local(126,2,-1000.0);"
            } else {
                ""
            };
            let dz = if dim == 3 { "2000.0" } else { "0.0" };
            let update=format!("ag_set_local(0,0,2.0);ag_set_local(126,0,-1000.0);ag_set_local(126,1,-1000.0);{z}let t=ag_sweep_aabb(126,0,2000.0,2000.0,{dz});let h=ag_sweep_hit(t);let n=ag_sweep_node(t);let f=ag_sweep_t(t);let x=ag_sweep_origin(t,0);let y=ag_sweep_origin(t,1);let nx=ag_sweep_normal(t,0);let ny=ag_sweep_normal(t,1);let o=ag_sweep_start_overlap(t);ag_set_local(0,0,4.0);for i in 0..8 {{let a=ag_aabb_min(127,0)}};if h!={} {{observed=99}}",if mode=="late-miss" {0}else{1});
            let mut s = scene(&update, &rows);
            let before = s.yielded;
            let result = s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit);
            eprintln!(
                "STRESS dim={dim} mode={mode} result={result:?} slices={}",
                s.yielded - before
            );
            result.unwrap();
            assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
            assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
        }
    }
}

#[test]
fn bounded_query_compile_cpu_observation() {
    let mut s = scene_custom("", &vec![(2, 3, BOX); 128], false, |mut source| {
        source.push_str("\nfn probe_query() int {return ag_sweep_aabb(0,0,10.0,0.0,0.0)}\n");
        source
    });
    let start = std::time::Instant::now();
    let token = s.int("probe_query", &[]).unwrap();
    let query_us = start.elapsed().as_micros();
    assert!(token > 0);
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    eprintln!(
        "SWEEP_CPU compile_us={} query_128_us={query_us} boundary=wall_CPU_call_not_GPU_timestamp",
        s.compile_us
    );
}

#[test]
fn switching_parent_world_cache_and_final_publication_128() {
    let mut rows = vec![(2, 3, BOX); 128];
    rows[0] = (5, 3, [0.; 7]);
    rows[1] = (5, 3, [0.; 7]);
    let update="ag_set_local(0,0,2.0);ag_set_local(1,0,-3.0);let q=ag_sweep_aabb(126,0,8.0,0.0,0.0);let h=ag_sweep_hit(q);let n=ag_sweep_node(q);let t=ag_sweep_t(q);let x=ag_sweep_origin(q,0);let y=ag_sweep_origin(q,1);let nx=ag_sweep_normal(q,0);let ny=ag_sweep_normal(q,1);let o=ag_sweep_start_overlap(q);for i in 0..8 {let b=ag_aabb_min(127,0)};ag_set_local(1,0,7.0);if h!=1 || x!=2.0 || o!=1 {observed=1}";
    let mut s = scene_custom(update, &rows, false, |mut src| {
        src.push_str("\nfn probe_parents(){_ag_parents[1]= -1;for i in 2..128 {_ag_parents[i]=i%2};_ag_dirty=1}\n");
        src
    });
    s.call("probe_parents", &[], ResultKind::Unit).unwrap();
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
    assert_eq!(s.float("probe_direct_x", &[Int(126)]).unwrap(), 2.);
    assert_eq!(s.float("probe_direct_x", &[Int(127)]).unwrap(), 7.);
    assert_eq!(s.int("probe_credit", &[]).unwrap(), 32);
}
