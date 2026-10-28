use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::*,
    Session,
};
use std::path::Path;
fn scene(update: &str, rows: &[(i32, i32, [f32; 7])]) -> Session {
    scene_guard(update, rows, false)
}
fn scene_guard(update: &str, rows: &[(i32, i32, [f32; 7])], alternate: bool) -> Session {
    let src = format!("fn app_start(){{}}\nfn app_update(dt float){{{update}}}\n");
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("spatial-api.at"),
        src.into_bytes(),
        Api::SpatialV2,
    )
    .unwrap();
    let source = if alternate {
        snap.source.replace("v - v != 0.0", "v != v")
    } else {
        snap.source.clone()
    };
    let folder = Path::new("target/spatial-feasibility-sources");
    std::fs::create_dir_all(folder).unwrap();
    let hash = agp::renderer::capture::sha256_hex(source.as_bytes());
    std::fs::write(folder.join(format!("{hash}.at")), &source).unwrap();
    let mut s = Session::new(&source).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, (kind, dim, row)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(0)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in row.iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}
const BOX: [f32; 7] = [1., -1., -1., 0., 1., 1., 0.];
#[test]
fn closed_bounds_ulp_hidden_self_and_dimensions() {
    let mut s = scene(
        "",
        &[
            (0, 2, BOX),
            (1, 2, BOX),
            (2, 3, [1., 0., 0., 0., 0., 0., 0.]),
        ],
    );
    assert_eq!(s.float("ag_aabb_min", &[Int(0), Int(0)]).unwrap(), -1.);
    assert_eq!(s.float("ag_aabb_max", &[Int(0), Int(2)]).unwrap(), 0.);
    assert_eq!(
        s.int(
            "ag_aabb_contains",
            &[Int(0), Float(1.), Float(1.), Float(0.)]
        )
        .unwrap(),
        1
    );
    assert_eq!(
        s.int(
            "ag_aabb_contains",
            &[
                Int(0),
                Float(f32::from_bits(1f32.to_bits() + 1)),
                Float(1.),
                Float(0.)
            ]
        )
        .unwrap(),
        0
    );
    assert_eq!(
        s.int(
            "ag_aabb_contains",
            &[
                Int(0),
                Float(f32::from_bits(1f32.to_bits() - 1)),
                Float(1.),
                Float(0.)
            ]
        )
        .unwrap(),
        1
    );
    assert_eq!(s.int("ag_aabb_overlaps", &[Int(0), Int(0)]).unwrap(), 1);
    s.call(
        "ag_set_local",
        &[Int(1), Int(0), Float(2.)],
        ResultKind::Unit,
    )
    .unwrap();
    assert_eq!(s.int("ag_aabb_overlaps", &[Int(0), Int(1)]).unwrap(), 1);
    s.call(
        "ag_set_local",
        &[Int(1), Int(0), Float(f32::from_bits(2f32.to_bits() + 1))],
        ResultKind::Unit,
    )
    .unwrap();
    assert_eq!(s.int("ag_aabb_overlaps", &[Int(0), Int(1)]).unwrap(), 0);
    assert_eq!(s.int("ag_aabb_overlaps", &[Int(2), Int(2)]).unwrap(), 1);
    assert_eq!(s.int("ag_aabb_overlaps", &[Int(0), Int(2)]).unwrap(), 0);
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 8);
}
#[test]
fn invalid_geometry_arguments_sticky_and_budget_boundary() {
    for (update,expected) in [
 ("let v=ag_aabb_min(-1,0)",1),
 ("let v=ag_aabb_min(0,3)",2),
 ("let v=ag_aabb_contains(0,0.0,0.0,1.0)",7),
 ("let v=ag_aabb_contains(0,(-1.0).sqrt(),0.0,0.0)",7),
 ("let v=ag_aabb_contains(0,99999999999999999999999999999999999999.0*99999999999999999999999999999999999999.0,0.0,0.0)",7),
 ("let bad=(-1.0).sqrt(); let v=ag_aabb_contains(0,bad,0.0,0.0)",7),
 ("let values=[(-1.0).sqrt()]; let v=ag_aabb_contains(0,values[0],0.0,0.0)",7),
 ("fn bad() float {return (-1.0).sqrt()}; let v=ag_aabb_contains(0,bad(),0.0,0.0)",7),
 ("let large=99999999999999999999999999999999999999.0; let v=ag_aabb_contains(0,0.0 - large*large,0.0,0.0)",7),
 ("let v=ag_aabb_contains(0,(-10.0).pow(9999.0),0.0,0.0)",7),
 ("let v=ag_aabb_contains(0,9999.0.exp(),0.0,0.0)",7),
 ("let v=ag_aabb_min(1,0)",6),
 ("let v=ag_aabb_min(2,0)",9),
 ("for i in 0..33 {let v=ag_aabb_min(0,0)}",10),
 ]{
 let update=format!("{update}; let good=ag_aabb_min(0,0)");
 let mut s=scene(&update,&[(0,2,BOX),(5,2,[0.;7]),(2,3,[0.;7])]);
 s.call("_ag_tick",&[Int(1),Float(1./60.)],ResultKind::Unit).unwrap();
 assert_eq!(s.int("_ag_status",&[]).unwrap(),expected,"{update}");
 }
    let mut s = scene("for i in 0..32 {let v=ag_aabb_min(0,0)}", &[(0, 2, BOX)]);
    for n in 1..3 {
        s.call("_ag_tick", &[Int(n), Float(1. / 60.)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    }
}

#[test]
fn distinguish_nonfinite_producer_and_actual_query_parameter() {
    let mut s=Session::new("fn main(){}\nfn nan_value() float {return (-1.0).sqrt()}\nfn inf_value() float {return 99999999999999999999999999999999999999.0*99999999999999999999999999999999999999.0}\n").unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    let error = s.float("nan_value", &[]).unwrap_err();
    assert!(
        error.contains("nonfinite") || error.contains("return type mismatch: 7ff8"),
        "{error}"
    );
    let mut s=Session::new("fn main(){}\nfn inf_value() float {return 99999999999999999999999999999999999999.0*99999999999999999999999999999999999999.0}\n").unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    assert!(s
        .float("inf_value", &[])
        .unwrap_err()
        .contains("nonfinite return"));
}

#[test]
fn nan_guard_distinguishability_gate_original_and_alternate() {
    let big = "99999999999999999999999999999999999999.0";
    let nan = format!("({big}*{big}) - ({big}*{big})");
    let mut gaps = vec![];
    for alternate in [false, true] {
        for (name, expr, status, hit) in [
            ("literal-zero", "0.0".to_string(), 0, true),
            ("native-zero", "0.0.sqrt()".to_string(), 0, true),
            ("native-finite-outside", "4.0.sqrt()".to_string(), 0, false),
            ("f32-nan", nan.clone(), 7, false),
            ("native-nan", "(-1.0).sqrt()".to_string(), 7, false),
        ] {
            let update =
                format!("if ag_aabb_contains(0,{expr},0.0,0.0)==1 {{ag_set_local(0,1,10.0)}}");
            let mut s = scene_guard(&update, &[(0, 2, BOX)], alternate);
            let tick = s.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit);
            let actual_status = s.int("_ag_status", &[]).unwrap();
            let actual_hit = s.float("_ag_local", &[Int(0), Int(1)]).unwrap() == 10.;
            eprintln!("alternate={alternate} case={name} tick={tick:?} expected_status={status} actual_status={actual_status} expected_hit={hit} actual_hit={actual_hit}");
            if actual_status != status || actual_hit != hit {
                gaps.push(format!("alternate={alternate} {name}"));
            }
        }
    }
    assert!(
        gaps.is_empty(),
        "T02 numeric contract is not feasible with these two managed guards: {gaps:?}"
    );
}
