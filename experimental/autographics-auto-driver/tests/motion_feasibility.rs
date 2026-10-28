include!("support/motion.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
#[test]
fn motion_128_directional_cost_matrix() {
    let mut failures = Vec::new();
    for dim in [2, 3] {
        for mode in ["diagonal", "late-miss", "decreasing"] {
            for sign in [1., -1.] {
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
                    rows[i].2 = [1., x - 1., x - 1., x - 1., x + 1., x + 1., x + 1.];
                    if mode == "late-miss" {
                        let a = if dim == 3 { 3 } else { 2 };
                        rows[i].2[a] = 3000.;
                        rows[i].2[a + 3] = 3002.;
                    }
                }
                let z = if dim == 3 {
                    format!("ag_set_local(126,2,{:?});", -1000. * sign)
                } else {
                    String::new()
                };
                let dz = if dim == 3 { 2000. * sign } else { 0. };
                let update=format!("ag_set_local(0,0,2.0);ag_set_local(126,0,{:?});ag_set_local(126,1,{:?});{z}let t=ag_move_and_slide(126,0,{:?},{:?},{dz:?},0.25);let s=ag_motion_status(t);let c=ag_motion_contacts(t);let x=ag_motion_delta(t,0);let y=ag_motion_delta(t,1);let z=ag_motion_delta(t,2);let nx=ag_motion_normal(t,0);let ny=ag_motion_normal(t,1);let nz=ag_motion_normal(t,2);ag_set_local(0,0,4.0);if s==4 {{observed=99}}",-1000.*sign,-1000.*sign,2000.*sign,2000.*sign);
                let mut vm = scene_custom(&update, &rows, false, |s| s);
                let before = vm.yielded;
                let tick_clock = std::time::Instant::now();
                let result = vm.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit);
                eprintln!("MOTION_TICK compile_us={} full_callback_wall_us={} scope=VM_call_not_GPU_timestamp",vm.compile_us,tick_clock.elapsed().as_micros());
                eprintln!(
                    "matrix dim={dim} mode={mode} sign={sign} slices={} result={result:?}",
                    vm.yielded - before
                );
                if let Err(error) = result {
                    failures.push(format!("{dim}/{mode}/{sign}: {error}"));
                    continue;
                };
                assert_eq!(vm.int("_ag_status", &[]).unwrap(), 0);
                assert_eq!(vm.int("probe_observed", &[]).unwrap(), 0);
                assert_eq!(vm.int("probe_credit", &[]).unwrap(), 32);
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn first_scan_cost_diagnostic() {
    for mode in ["diagonal", "late-miss", "decreasing"] {
        let mut rows = vec![(2, 3, BOX); 128];
        rows[0] = (5, 3, [0.; 7]);
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
        }
        let mut vm = scene_custom("", &rows, false, |mut s| {
            s.push_str("\nfn probe_first() int {_ag_motion_checked=0;return _ag_motion_three(3,126,0,-1000.0,-1000.0,-1000.0,2000.0,2000.0,2000.0)}\n");
            s
        });
        let before = vm.yielded;
        let clock = std::time::Instant::now();
        let result = vm.int("probe_first", &[]);
        eprintln!("MOTION_COST compile_us={} first_query_scan_wall_us={} scope=private_scan_not_full_tick_not_GPU_timestamp",vm.compile_us,clock.elapsed().as_micros());
        eprintln!("first {mode}: {result:?} slices={}", vm.yielded - before);
        assert!(result.is_ok());
    }
}

#[test]
fn mixed_sign_unequal_three_axes_128_decreasing_candidates() {
    let mut rows = vec![(2, 3, BOX); 128];
    rows[0] = (5, 3, [0.; 7]);
    for i in 1..128 {
        if i == 126 {
            continue;
        };
        let x = 500. - i as f32;
        let y = x * 0.75;
        let z = -x * 0.6;
        rows[i].2 = [1., x - 1., y - 1., z - 1., x + 1., y + 1., z + 1.];
    }
    let script="ag_set_local(0,0,2.0);ag_set_local(126,0,-1000.0);ag_set_local(126,1,-750.0);ag_set_local(126,2,600.0);let q=ag_move_and_slide(126,0,2000.0,1500.0,-1200.0,0.25);let s=ag_motion_status(q);let c=ag_motion_contacts(q);let x=ag_motion_delta(q,0);let y=ag_motion_delta(q,1);let z=ag_motion_delta(q,2);let nx=ag_motion_normal(q,0);let ny=ag_motion_normal(q,1);let nz=ag_motion_normal(q,2);ag_set_local(0,1,4.0);if s!=1 {observed=1};if c!=1 {observed=2};if nx!= -1.0 {observed=3};if x<1370.749 || x>1370.751 {observed=4};if y<1499.999 || y>1500.001 {observed=5}";
    let mut vm = scene_custom(script, &rows, false, |s| s);
    let before = vm.yielded;
    let tick_clock = std::time::Instant::now();
    let result = vm.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit);
    eprintln!(
        "MOTION_TICK compile_us={} full_callback_wall_us={} scope=VM_call_not_GPU_timestamp",
        vm.compile_us,
        tick_clock.elapsed().as_micros()
    );
    eprintln!(
        "unequal mixed128 slices={} result={result:?}",
        vm.yielded - before
    );
    result.unwrap();
    assert_eq!(vm.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(vm.int("probe_observed", &[]).unwrap(), 0);
    assert!((vm.float("probe_direct_x", &[Int(126)]).unwrap() - 372.75).abs() < 0.001);
    assert!((vm.float("probe_direct_y", &[Int(126)]).unwrap() - 754.).abs() < 0.001);
}
