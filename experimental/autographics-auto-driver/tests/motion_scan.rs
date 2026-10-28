include!("support/motion.rs");
#[test]
fn private_scan_has_no_token_side_effect() {
    let rows = [
        (2, 2, [1., -1., -1., 0., 1., 1., 0.]),
        (2, 2, [1., 4., -10., 0., 4.2, 10., 0.]),
    ];
    let mut vm = scene_custom("", &rows, false, |mut s| {
        s.push_str("\nfn probe_motion() int {return _ag_motion_two(2,0,0,0.0,0.0,0.0,10.0,4.0,0.0)}\nfn probe_fraction() float {return _ag_cast_fraction}\n");
        s
    });
    assert_eq!(vm.int("probe_motion", &[]).unwrap(), 1);
    assert_eq!(vm.float("probe_fraction", &[]).unwrap(), 0.3);
    assert_eq!(vm.int("probe_token", &[]).unwrap(), 0);
    assert_eq!(vm.int("probe_credit", &[]).unwrap(), 0);
}

#[test]
fn directional_boundary_and_negative_scan() {
    for dim in [2, 3] {
        let rows = [
            (2, dim, [1., -1., -1., -1., 1., 1., 1.]),
            (2, dim, [1., 4., -10., -10., 4.2, 10., 10.]),
        ];
        for (origin, delta, found, normal) in [
            (3., 1., 1, -1.),
            (3., 0., 0, 0.),
            (3., -1., 0, 0.),
            (6., -4., 1, 1.),
            (5.2, -1., 1, 1.),
            (5.2, 1., 0, 0.),
        ] {
            let entry = if dim == 2 {
                "_ag_motion_two"
            } else {
                "_ag_motion_three"
            };
            let mut vm = scene_custom("", &rows, false, |mut s| {
                s.push_str(&format!("\nfn probe_motion() int {{return {entry}({dim},0,0,{origin:?},0.0,0.0,{delta:?},0.0,0.0)}}\nfn probe_found() int {{return _ag_cast_found}}\nfn probe_normal() float {{return _ag_cast_nx}}\n"));
                s
            });
            assert_eq!(vm.int("probe_motion", &[]).unwrap(), 1);
            assert_eq!(
                vm.int("probe_found", &[]).unwrap(),
                found,
                "dim={dim} origin={origin} delta={delta}"
            );
            assert_eq!(
                vm.float("probe_normal", &[]).unwrap(),
                normal,
                "dim={dim} origin={origin} delta={delta}"
            );
        }
    }
}

#[test]
fn hand_computed_wall_slide() {
    let rows = [
        (2, 2, [1., -1., -1., 0., 1., 1., 0.]),
        (2, 2, [1., 4., -10., 0., 4.2, 10., 0.]),
    ];
    let mut vm=scene_custom("let t=ag_move_and_slide(0,0,10.0,4.0,0.0,0.25);if ag_motion_status(t)!=1 || ag_motion_contacts(t)!=1 {observed=99}",&rows,false,|s|s);
    vm.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(vm.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(vm.int("probe_observed", &[]).unwrap(), 0);
    assert!((vm.float("probe_direct_x", &[Int(0)]).unwrap() - 2.75).abs() < 0.00001);
    assert!((vm.float("probe_direct_y", &[Int(0)]).unwrap() - 4.).abs() < 0.00001);
}

#[test]
fn bounded_128_two_contacts() {
    for dim in [2, 3] {
        let mut rows = vec![(2, dim, [1., 4., -100., -100., 4.2, 100., 100.]); 128];
        rows[0] = (2, dim, [1., -1., -1., -1., 1., 1., 1.]);
        rows[127] = (2, dim, [1., -100., 6., -100., 100., 6.2, 100.]);
        let mut vm=scene_custom("let t=ag_move_and_slide(0,0,10.0,10.0,0.0,0.25);let s=ag_motion_status(t);let c=ag_motion_contacts(t);let x=ag_motion_delta(t,0);let y=ag_motion_delta(t,1);let z=ag_motion_delta(t,2);let nx=ag_motion_normal(t,0);let ny=ag_motion_normal(t,1);let nz=ag_motion_normal(t,2);if s!=2 || c!=2 {observed=99}",&rows,false,|s|s);
        let before = vm.yielded;
        let result = vm.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit);
        eprintln!(
            "motion dim={dim} slices={} result={result:?}",
            vm.yielded - before
        );
        result.unwrap();
        assert_eq!(vm.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(vm.int("probe_observed", &[]).unwrap(), 0);
        assert_eq!(vm.int("probe_credit", &[]).unwrap(), 32);
        assert!((vm.float("probe_direct_x", &[Int(0)]).unwrap() - 2.75).abs() < 0.00001);
        assert!((vm.float("probe_direct_y", &[Int(0)]).unwrap() - 4.75).abs() < 0.00001);
    }
}

#[test]
fn second_scan_oracle_rejects_stop_only_mutation() {
    let rows = [
        (2, 2, [1., -1., -1., 0., 1., 1., 0.]),
        (2, 2, [1., 4., -10., 0., 4.2, 10., 0.]),
    ];
    for mutate in [false, true, false] {
        let mut vm = scene_custom(
            "let q=ag_move_and_slide(0,0,10.0,4.0,0.0,0.25)",
            &rows,
            false,
            |s| {
                if mutate {
                    assert!(s.contains("for stage in 0..2"));
                    s.replace("for stage in 0..2", "for stage in 0..1")
                } else {
                    s
                }
            },
        );
        vm.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
            .unwrap();
        let y = vm.float("probe_direct_y", &[Int(0)]).unwrap();
        assert_eq!((y - 4.).abs() < 0.00001, !mutate);
        if mutate {
            assert!((y - 1.1).abs() < 0.00001)
        }
    }
}
