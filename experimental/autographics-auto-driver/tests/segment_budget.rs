include!("support/segment.rs");

// Independent f32 interval oracle; never calls or reads managed cast output.
fn reference(
    rows: &[(i32, i32, [f32; 7])],
    space: i32,
    end: [f32; 3],
    axis: usize,
    shift: f32,
) -> (i32, f32, f32, f32) {
    let mut best = (-1, 1., end[0], 0.);
    for (node, (kind, dim, row)) in rows.iter().enumerate() {
        if *kind == 5 || *dim != space {
            continue;
        }
        let mut enter = 0.0f32;
        let mut leave = 1.0f32;
        let mut normal = 0.;
        for k in 0..space as usize {
            let lo = row[1 + k] + if k == axis { shift } else { 0. };
            let hi = row[4 + k] + if k == axis { shift } else { 0. };
            if end[k] == 0. {
                if 0. < lo || 0. > hi {
                    leave = -1.;
                    break;
                }
            } else {
                let n = (lo / end[k]).min(hi / end[k]);
                let f = (lo / end[k]).max(hi / end[k]);
                if n > enter {
                    enter = n;
                    normal = if k == 0 {
                        if end[k] > 0. {
                            -1.
                        } else {
                            1.
                        }
                    } else {
                        0.
                    }
                };
                leave = leave.min(f)
            }
        }
        if enter <= leave && (best.0 < 0 || enter < best.1) {
            best = (node as i32, enter, enter * end[0], normal)
        }
    }
    best
}
fn script(
    rows: &[(i32, i32, [f32; 7])],
    space: i32,
    end: [f32; 3],
    axis: usize,
    host: bool,
) -> String {
    let cast = format!(
        "ag_cast_segment({space},-1,0,0.0,0.0,0.0,{:.9},{:.9},{:.9})",
        end[0], end[1], end[2]
    );
    let mut out = String::new();
    for (q, shift) in [("a", 2.), ("b", 4.)] {
        out+=&format!("ag_set_local(0,{axis},{shift:.1});let {q}={cast};let h{q}=ag_cast_hit({q});let n{q}=ag_cast_node({q});let t{q}=ag_cast_t({q});let p{q}=ag_cast_point({q},0);let f{q}=ag_cast_normal({q},0);");
    }
    out += "for i in 0..6 {let unused=ag_aabb_min(127,0)};";
    let mut assertions = Vec::new();
    for (q, shift) in [("a", 2.), ("b", 4.)] {
        let (node, t, p, n) = reference(rows, space, end, axis, shift);
        assertions.push(format!("h{q}!={} || n{q}!={node} || t{q}<{:.9} || t{q}>{:.9} || p{q}<{:.9} || p{q}>{:.9} || f{q}!={n:.1}",i32::from(node>=0),t-1e-6,t+1e-6,p-1e-3,p+1e-3));
    }
    out += &format!(
        "if {} {{{}}}",
        assertions.join(" || "),
        if host {
            "ag_set_visible(127,0)"
        } else {
            "observed=1"
        }
    );
    out
}
fn rows(mode: &str, space: i32, deep: bool) -> (Vec<(i32, i32, [f32; 7])>, [f32; 3]) {
    let diagonal = mode.contains("diagonal");
    let sign = if mode == "negative-diagonal" { -1. } else { 1. };
    let scale = if mode == "unequal-diagonal" {
        [1., 0.75, 0.5]
    } else {
        [1.; 3]
    };
    let mut end = [
        800. * sign,
        if diagonal { 800. * sign * scale[1] } else { 0. },
        if diagonal && space == 3 {
            800. * sign * scale[2]
        } else {
            0.
        },
    ];
    if space == 2 {
        end[2] = 0.
    }
    let mut rows = vec![
        (
            if space == 2 { 0 } else { 2 },
            space,
            [1., -1., -1., -1., 1., 1., 1.]
        );
        128
    ];
    let groups = if deep { 126 } else { 1 };
    for i in 0..groups {
        rows[i] = (5, space, [0.; 7])
    }
    for i in groups..128 {
        let x = 500. - i as f32;
        if mode == "decreasing-t" {
            rows[i].2 = [1., x, -1., -1., x + 10., 1., 1.]
        }
        if diagonal {
            for k in 0..space as usize {
                let a = x * sign * scale[k];
                let b = (x + 10.) * sign * scale[k];
                rows[i].2[k + 1] = a.min(b);
                rows[i].2[k + 4] = a.max(b)
            }
        }
        if mode == "late-z-miss" {
            let k = (space - 1) as usize;
            rows[i].2[k + 1] = 10.;
            rows[i].2[k + 4] = 11.
        }
        if space == 2 {
            rows[i].2[3] = 0.;
            rows[i].2[6] = 0.
        }
    }
    (rows, end)
}
#[test]
fn original_counterexamples_and_diagonal_controls_128_exact_credits() {
    for space in [2, 3] {
        for deep in [false, true] {
            for mode in [
                "equal",
                "late-z-miss",
                "decreasing-t",
                "diagonal-decreasing",
                "negative-diagonal",
                "unequal-diagonal",
            ] {
                for axis in 0..space as usize {
                    let (rows, end) = rows(mode, space, deep);
                    let update = script(&rows, space, end, axis, false);
                    let mut vm = if deep {
                        scene_guard(&update, &rows, true)
                    } else {
                        scene(&update, &rows)
                    };
                    let before = vm.yielded;
                    vm.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
                        .unwrap_or_else(|e| {
                            panic!("space={space} deep={deep} mode={mode} axis={axis}: {e}")
                        });
                    assert_eq!(vm.int("_ag_status", &[]).unwrap(), 0);
                    assert_eq!(
                        vm.int("probe_observed", &[]).unwrap(),
                        0,
                        "space={space} deep={deep} mode={mode} axis={axis}"
                    );
                    assert_eq!(vm.int("probe_credit", &[]).unwrap(), 32);
                    assert_eq!(vm.int("probe_token", &[]).unwrap(), 0);
                    eprintln!(
                        "BUDGET space={space} deep={deep} mode={mode} axis={axis} slices={}",
                        vm.yielded - before
                    );
                }
            }
        }
    }
}

#[test]
fn redundant_source_lines_reproduce_the_diagonal_budget_failure() {
    let (rows, end) = rows("unequal-diagonal", 3, false);
    let update = script(&rows, 3, end, 0, false);
    let mut accepted = scene(&update, &rows);
    accepted
        .call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
    assert_eq!(accepted.int("probe_observed", &[]).unwrap(), 0);
    let mut expanded = scene_custom(&update, &rows, false, |s| {
        let user = s.find("var observed=0").unwrap();
        format!("{}{}", s[..user].replace(';', ";\n"), &s[user..])
    });
    let error = expanded
        .call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
        .unwrap_err();
    assert!(error.contains("cumulative CPU budget exhausted"), "{error}");
    assert!(error.contains("100000 steps"), "{error}");
}

#[test]
fn actual_host_counterexamples_and_diagonal_controls() {
    use autographics_auto_driver::authoring::{
        document::{Dimension, Document, Group, Kind, Object},
        host::Host,
        play::{Action, State},
    };
    use std::time::{Duration, Instant};
    fn settle(h: &mut Host) {
        let end = Instant::now() + Duration::from_secs(31);
        while h.play.as_ref().unwrap().pending() {
            h.frame(None).unwrap();
            assert!(Instant::now() < end)
        }
    }
    for mode in [
        "equal",
        "late-z-miss",
        "decreasing-t",
        "diagonal-decreasing",
        "negative-diagonal",
        "unequal-diagonal",
    ] {
        let (wanted, end) = rows(mode, 3, false);
        let mut d = Document::empty(format!("budget-{mode}"));
        d.groups.push(Group::new(1, Dimension::Three));
        for i in 1..128 {
            let mut o = Object::new(i as u64 + 1, Kind::Cylinder);
            o.parent = Some(1);
            // Cylinder local bounds are obtained from actual resources below;
            // world centers arrange the same adversarial ordinal distributions.
            let r = wanted[i].2;
            o.translation = [
                (r[1] + r[4]) * 0.5,
                (r[2] + r[5]) * 0.5 - 10.,
                (r[3] + r[6]) * 0.5,
            ];
            o.scale = [20.; 3];
            d.objects.push(o);
        }
        let mut h = Host::new(d).unwrap();
        let mesh = agp::render3d::cylinder_mesh();
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for vertex in &mesh.positions {
            for k in 0..3 {
                lo[k] = lo[k].min(vertex[k] * 20.);
                hi[k] = hi[k].max(vertex[k] * 20.)
            }
        }
        let mut actual = Vec::from([(5, 3, [0.; 7])]);
        for o in &h.editor.document.objects {
            let mut r = [1., 0., 0., 0., 0., 0., 0.];
            for k in 0..3 {
                r[k + 1] = o.translation[k] + lo[k];
                r[k + 4] = o.translation[k] + hi[k]
            }
            actual.push((2, 3, r))
        }
        let raw = format!(
            "fn app_start(){{}}\nfn app_update(dt float){{{}}}\n",
            script(&actual, 3, end, 0, true)
        );
        h.start_user_snapshot(
            SourceSnapshot::from_bytes_for(
                Path::new("budget.at"),
                raw.into_bytes(),
                Api::SegmentV3,
            )
            .unwrap(),
        )
        .unwrap();
        h.pause_play("");
        settle(&mut h);
        assert_eq!(h.play.as_ref().unwrap().state, State::Paused, "init {mode}");
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .unwrap();
        settle(&mut h);
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.state, State::Paused, "mode={mode} {:?}", p.error);
        assert_eq!(p.tick(), 1);
        assert!(
            p.last.as_ref().unwrap().objects.iter().all(|o| o.visible),
            "wrong cast result: {mode}"
        );
        assert_eq!(
            p.last.as_ref().unwrap().document.groups[0].translation[0],
            4.
        );
        eprintln!(
            "HOST BUDGET {mode} Paused/tick1 accepted; oracle includes hit/node/t/point.x/normal.x"
        );
        h.stop_play();
    }
}
