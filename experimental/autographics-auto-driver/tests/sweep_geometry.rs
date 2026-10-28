include!("support/sweep.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
// Independent scalar reference, fixed public f32 operation order. No managed helper calls.
fn reference(
    o: [f32; 3],
    d: [f32; 3],
    m: [f32; 7],
    targets: &[[f32; 7]],
    dim: usize,
) -> (i32, f32, [f32; 3], [f32; 3], i32) {
    let mut best = (-1, 1., [o[0] + d[0], o[1] + d[1], o[2] + d[2]], [0.; 3], 0);
    for (i, b) in targets.iter().enumerate() {
        let mut enter = 0f32;
        let mut leave = 1f32;
        let mut inside = true;
        let mut axis_entries = [None; 3];
        let mut normal = [0.; 3];
        let mut good = true;
        for k in 0..dim {
            let lo = b[1 + k] - m[4 + k];
            let hi = b[4 + k] - m[1 + k];
            inside &= o[k] >= lo && o[k] <= hi;
            if d[k] == 0. {
                if o[k] < lo || o[k] > hi {
                    good = false;
                    break;
                }
            } else {
                let (n, f) = if d[k] > 0. {
                    ((lo - o[k]) / d[k], (hi - o[k]) / d[k])
                } else {
                    ((hi - o[k]) / d[k], (lo - o[k]) / d[k])
                };
                axis_entries[k] = Some(n);
                enter = enter.max(n);
                leave = leave.min(f);
                if enter > leave {
                    good = false;
                    break;
                }
            }
        }
        if !good || best.0 >= 0 && enter >= best.1 {
            continue;
        }
        if !inside {
            for k in 0..dim {
                if axis_entries[k] == Some(enter) {
                    normal[k] = if d[k] > 0. { -1. } else { 1. };
                    break;
                }
            }
        }
        best = (
            i as i32 + 1,
            enter,
            std::array::from_fn(|k| o[k] + enter * d[k]),
            normal,
            i32::from(inside),
        );
    }
    best
}
fn lit(v: f32) -> String {
    format!("{v:.20}")
}
fn run(o: [f32; 3], d: [f32; 3], m: [f32; 7], targets: &[[f32; 7]], dim: usize) {
    let (node, t, p, n, inside) = reference(o, d, m, targets, dim);
    let mut script = String::new();
    for k in 0..dim {
        script += &format!("ag_set_local(0,{k},{});", lit(o[k]));
    }
    script+=&format!("let q=ag_sweep_aabb(0,0,{},{},{});if ag_sweep_hit(q)!={} || ag_sweep_node(q)!={node} || ag_sweep_start_overlap(q)!={inside} {{observed=1}};let t=ag_sweep_t(q);if t<{} || t>{} {{observed=2}};",lit(d[0]),lit(d[1]),lit(d[2]),i32::from(node>=0),lit(t-1e-6),lit(t+1e-6));
    for k in 0..3 {
        script+=&format!("let p{k}=ag_sweep_origin(q,{k});if p{k}<{} || p{k}>{} || ag_sweep_normal(q,{k})!={} {{observed=3}};",lit(p[k]-1e-3),lit(p[k]+1e-3),lit(n[k]));
    }
    let mut rows = vec![(2, dim as i32, m)];
    rows.extend(targets.iter().map(|b| (2, dim as i32, *b)));
    let mut s = scene(&script, &rows);
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "{o:?} {d:?}");
    assert_eq!(
        s.int("probe_observed", &[]).unwrap(),
        0,
        "{o:?} {d:?} expected {node}/{t}/{n:?}"
    );
}
#[test]
fn independent_f32_volume_slab_states_and_ties() {
    for dim in [2, 3] {
        for (o, d) in [
            ([-4., 0., 0.], [8., 0., 0.]),
            ([4., 0., 0.], [-8., -0., 0.]),
            ([0.; 3], [0.; 3]),
            ([-2., 0., 0.], [-8., 0., 0.]),
            ([-4., 0., 0.], [2., 0., 0.]),
            ([-4., 3., 0.], [8., 0., 0.]),
            ([-4., -4., 0.], [8., 8., 0.]),
            ([-4., -4., -4.], [8., 8., 8.]),
        ] {
            if dim == 2 && o[2] != 0. {
                continue;
            }
            run(o, d, BOX, &[BOX, BOX], dim);
        }
    }
    run(
        [0.; 3],
        [10., 0., 0.],
        BOX,
        &[[1., 4., -1., -1., 4.2, 1., 1.]],
        3,
    );
    run(
        [0.; 3],
        [10., 0., 0.],
        [1., 2., -1., -1., 5., 1., 1.],
        &[[1., 8., -1., -1., 8.2, 1., 1.]],
        3,
    );
    run(
        [-4., 0., 0.],
        [8., 0., 0.],
        BOX,
        &[[1., 2., -1., -1., 3., 1., 1.], BOX],
        3,
    );
    run([0.; 3], [0.; 3], BOX, &[], 3);
    run([-2.000000238418579, 0., 0.], [0.; 3], BOX, &[BOX], 3);
    run(
        [-3., 0., 0.],
        [1., 0., 0.],
        BOX,
        &[[1., 0., 0., 0., 0., 0., 0.]],
        3,
    );
}
#[test]
fn bad_matching_geometry_is_not_hidden_and_filters_exclude_it() {
    for (prefix, query, error) in [
        ("", "ag_sweep_aabb(0,0,0.0,0.0,0.0)", 9),
        ("ag_set_visible(2,0);", "ag_sweep_aabb(0,1,0.0,0.0,0.0)", 0),
        ("", "ag_sweep_aabb(-1,0,1.0,0.0,0.0)", 1),
        ("", "ag_sweep_aabb(0,2,1.0,0.0,0.0)", 7),
        ("", "ag_sweep_aabb(0,0,(-1.0).sqrt(),0.0,0.0)", 12),
    ] {
        let mut s = scene(
            &format!("{prefix}let q={query}"),
            &[(2, 3, BOX), (2, 3, BOX), (2, 3, [0.; 7])],
        );
        s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), error, "{query}");
    }
}
#[test]
fn thickness_oracle_rejects_point_proxy_mutation() {
    let update = "let q=ag_sweep_aabb(0,0,10.0,0.0,0.0);if ag_sweep_t(q)!=0.3 {observed=99}";
    let rows = [(2, 3, BOX), (2, 3, [1., 4., -1., -1., 4.2, 1., 1.])];
    for bad in [false, true] {
        let mut s = scene_custom(update, &rows, false, |src| {
            if bad {
                src.replace(" - cb[ignore*7+f0]", " - 0.0")
                    .replace(" - cb[ignore*7+n0]", " - 0.0")
            } else {
                src
            }
        });
        s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(
            s.int("probe_observed", &[]).unwrap(),
            if bad { 99 } else { 0 }
        );
    }
}

#[test]
fn positive_entry_underflow_has_face_but_not_initial_overlap() {
    for space in [2, 3] {
        for axis in 0..space as usize {
            let mut row = BOX;
            row[axis + 1] = 0.;
            let mover = [1., 0., 0., 0., 0., 0., 0.];
            let mut d = ["0.0"; 3];
            d[axis] = "100000000000000000000000000000000000000.0";
            for (start, overlap, n) in [
                (
                    "-0.000000000000000000000000000000000000000000001",
                    0,
                    "-1.0",
                ),
                ("0.0", 1, "0.0"),
            ] {
                let update=format!("ag_set_local(0,{axis},{start});let q=ag_sweep_aabb(0,0,{},{},{});if ag_sweep_hit(q)!=1 || ag_sweep_t(q)!=0.0 || ag_sweep_start_overlap(q)!={overlap} || ag_sweep_normal(q,{axis})!={n} {{observed=1}}",d[0],d[1],d[2]);
                let mut s = scene(&update, &[(2, space, mover), (2, space, row)]);
                s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
                    .unwrap();
                assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
                assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
            }
        }
    }
}
#[test]
fn native_nonfinite_flow_and_arithmetic_vs_geometry_classification() {
    for expression in [
        "(-1.0).sqrt()",
        "0.0.ln()",
        "0.0 - 0.0.ln()",
        "[(-1.0).sqrt()][0]",
    ] {
        let mut s = scene(
            &format!("let value={expression};let q=ag_sweep_aabb(0,0,value,0.0,0.0)"),
            &[(2, 3, BOX)],
        );
        s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 12);
    }
    let mut s = scene(
        "fn bad() float {return (-1.0).sqrt()};let q=ag_sweep_aabb(0,0,bad(),0.0,0.0)",
        &[(2, 3, BOX)],
    );
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 12);
    let big = "99999999999999999999999999999999999999.0";
    let mut s = scene(
        &format!("ag_set_local(2,0,{big});let q=ag_sweep_aabb(0,0,0.0,0.0,0.0)"),
        &[
            (2, 3, BOX),
            (2, 3, BOX),
            (2, 3, [1., f32::MAX, -1., -1., f32::MAX, 1., 1.]),
        ],
    );
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 9);
    let mut s = scene(
        "let q=ag_sweep_aabb(0,0,1.0,0.0,0.0)",
        &[
            (2, 3, [1., -f32::MAX, 0., 0., -f32::MAX, 0., 0.]),
            (2, 3, [1., f32::MAX, 0., 0., f32::MAX, 0., 0.]),
        ],
    );
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 12);
}
#[test]
fn fixed_slot_repetition_and_real_last_owner_release() {
    let update="let a=ag_cast_segment(3,-1,0,-4.0,0.0,0.0,4.0,0.0,0.0);let h=ag_cast_hit(a);let b=ag_sweep_aabb(0,0,1.0,0.0,0.0);observed=ag_sweep_hit(b)";
    let mut s = scene(update, &[(2, 3, BOX), (2, 3, BOX)]);
    let heap = s.heap_objects();
    let alive = s.owner_probe();
    for n in 1..65 {
        s.call("_ag_tick", &[Int(n), Float(0.016)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(s.int("probe_observed", &[]).unwrap(), 1);
        assert_eq!(s.heap_objects(), heap);
    }
    s.close();
    assert!(!alive());
}

#[test]
fn actor_handle_group_dimension_visibility_and_local_nonfinite() {
    for (rows, body, error, observed) in [
        (
            vec![(5, 3, [0.; 7]), (2, 3, BOX)],
            "let q=ag_sweep_aabb(0,0,1.0,0.0,0.0)",
            6,
            0,
        ),
        (
            vec![(2, 2, BOX)],
            "let q=ag_sweep_aabb(0,0,1.0,0.0,1.0)",
            7,
            0,
        ),
        (
            vec![(2, 2, BOX), (2, 3, [0.; 7])],
            "ag_set_visible(0,0);let q=ag_sweep_aabb(0,1,1.0,0.0,0.0);observed=ag_sweep_hit(q)",
            0,
            0,
        ),
        (
            vec![(2, 3, BOX), (2, 3, BOX)],
            "ag_set_visible(0,0);let q=ag_sweep_aabb(0,1,1.0,0.0,0.0);observed=ag_sweep_hit(q)",
            0,
            1,
        ),
        (
            vec![(2, 3, BOX)],
            "ag_set_local(0,0,(-1.0).sqrt());let q=ag_sweep_aabb(0,0,1.0,0.0,0.0)",
            9,
            0,
        ),
        (
            vec![(2, 3, BOX), (2, 3, BOX), (2, 3, BOX)],
            "ag_set_local(2,0,(-1.0).sqrt());let q=ag_sweep_aabb(0,0,0.0,0.0,0.0)",
            9,
            0,
        ),
    ] {
        let mut s = scene(body, &rows);
        s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
            .unwrap();
        eprintln!(
            "INPUT body={body} status={} node={} arg={}",
            s.int("_ag_status", &[]).unwrap(),
            s.int("_ag_error_node", &[]).unwrap(),
            s.int("_ag_error_argument", &[]).unwrap()
        );
        assert_eq!(s.int("_ag_status", &[]).unwrap(), error, "{body}");
        assert_eq!(s.int("probe_observed", &[]).unwrap(), observed);
    }
}
