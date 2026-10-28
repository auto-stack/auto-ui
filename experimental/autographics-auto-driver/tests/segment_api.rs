include!("support/segment.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
fn tick(s: &mut Session, n: i32) -> i32 {
    s.call("_ag_tick", &[Int(n), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}
#[test]
fn explicit_version_old_helpers_and_managed_identity() {
    use autographics_auto_driver::authoring::behavior::{API, SPATIAL_API};
    let raw = b"fn app_start(){}\nfn app_update(dt float){}\n";
    let mut hashes = vec![];
    for api in [Api::V1, Api::SpatialV2, Api::SegmentV3] {
        let snap = SourceSnapshot::from_bytes_for(Path::new("same.at"), raw.to_vec(), api).unwrap();
        if api == Api::V1 {
            assert_eq!(
                snap.api_hash,
                agp::renderer::capture::sha256_hex(API.as_bytes())
            );
        }
        if api == Api::SpatialV2 {
            assert_eq!(
                snap.api_hash,
                agp::renderer::capture::sha256_hex(format!("{API}\n{SPATIAL_API}").as_bytes())
            );
        }
        assert_eq!(snap.user_hash, agp::renderer::capture::sha256_hex(raw));
        let mut vm = Session::new(&snap.source).unwrap();
        vm.call("main", &[], ResultKind::Unit).unwrap();
        assert_eq!(vm.int("_ag_profile", &[]).unwrap(), api.profile());
        assert_eq!(vm.int("_ag_version", &[]).unwrap(), api.number());
        // Inspect the actual managed declaration rather than calling a renamed root.
        assert!(snap.source.contains(&format!("return {}", api.profile())));
        hashes.push(snap.combined_hash);
        let helper =
            b"fn ag_cast_hit(n int) int {return 17}\nfn app_start(){}\nfn app_update(dt float){}\n";
        assert_eq!(
            SourceSnapshot::from_bytes_for(Path::new("helper.at"), helper.to_vec(), api).is_ok(),
            api != Api::SegmentV3
        );
    }
    assert_ne!(hashes[0], hashes[1]);
    assert_ne!(hashes[1], hashes[2]);
    assert_eq!(
        Api::parse("autographics.behavior/5").unwrap(),
        Api::MotionV5
    );
    assert!(Api::parse("autographics.behavior/6").is_err());
}
#[test]
fn closed_slab_parallel_inside_endpoint_normal_and_miss() {
    for (cast, hit, t, px, nx) in [
        ("-2.0,0.0,0.0,2.0,0.0,0.0", 1, "0.25", "-1.0", "-1.0"),
        ("2.0,0.0,0.0,-2.0,0.0,0.0", 1, "0.25", "1.0", "1.0"),
        ("0.0,0.0,0.0,2.0,0.0,0.0", 1, "0.0", "0.0", "0.0"),
        ("-1.0,0.0,0.0,-1.0,0.0,0.0", 1, "0.0", "-1.0", "0.0"),
        ("-2.0,0.0,0.0,-1.0,0.0,0.0", 1, "1.0", "-1.0", "-1.0"),
        ("-2.0,2.0,0.0,2.0,2.0,0.0", 0, "1.0", "2.0", "0.0"),
        ("2.0,0.0,0.0,2.0,0.0,0.0", 0, "1.0", "2.0", "0.0"),
        ("-2.0,-2.0,-2.0,2.0,2.0,2.0", 1, "0.25", "-1.0", "-1.0"),
    ] {
        let update=format!("let q=ag_cast_segment(3,-1,0,{cast}); if ag_cast_hit(q)!={hit} || ag_cast_t(q)!={t} || ag_cast_point(q,0)!={px} || ag_cast_normal(q,0)!={nx} {{observed=1}}");
        let mut s = scene(&update, &[(2, 3, BOX)]);
        assert_eq!(tick(&mut s, 1), 0, "{cast}");
        assert_eq!(s.int("probe_observed", &[]).unwrap(), 0, "{cast}");
    }
    // A representable one-ULP gap stays a miss; no epsilon expands the box.
    let mut s=scene("let q=ag_cast_segment(3,-1,0,1.0000001192092896,0.0,0.0,2.0,0.0,0.0); observed=ag_cast_hit(q)",&[(2,3,BOX)]);
    assert_eq!(tick(&mut s, 1), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}
#[test]
fn nearest_tie_filters_and_later_bad_geometry_before_pruning() {
    let cast = "ag_cast_segment(3,-1,0,-4.0,0.0,0.0,4.0,0.0,0.0)";
    for (second, expected) in [(BOX, 0), ([1., -2., -1., -1., 0., 1., 1.], 1)] {
        let mut s = scene(
            &format!("let q={cast}; observed=ag_cast_node(q)"),
            &[(2, 3, BOX), (2, 3, second)],
        );
        assert_eq!(tick(&mut s, 1), 0);
        assert_eq!(s.int("probe_observed", &[]).unwrap(), expected);
    }
    let mut s = scene(&format!("let q={cast}"), &[(2, 3, BOX), (2, 3, [0.; 7])]);
    assert_eq!(tick(&mut s, 1), 9);
    for call in [
        "ag_cast_segment(3,1,0,-4.0,0.0,0.0,4.0,0.0,0.0)",
        "ag_cast_segment(2,-1,0,-4.0,0.0,0.0,4.0,0.0,0.0)",
    ] {
        let mut s = scene(&format!("let q={call}"), &[(2, 3, BOX), (2, 3, [0.; 7])]);
        assert_eq!(tick(&mut s, 1), 0);
    }
    let mut s=scene("ag_set_visible(0,0); let q=ag_cast_segment(3,-1,1,-4.0,0.0,0.0,4.0,0.0,0.0); observed=ag_cast_node(q)",&[(2,3,BOX),(2,3,BOX)]);
    assert_eq!(tick(&mut s, 1), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 1);
}
#[test]
fn token_expiry_sticky_failure_and_exact_credit_boundary() {
    let cast = "ag_cast_segment(3,-1,0,-2.0,0.0,0.0,2.0,0.0,0.0)";
    for (update,error) in [
  (format!("let a={cast}; let b={cast}; let h=ag_cast_hit(a)"),13),
  ("let h=ag_cast_hit(0)".into(),13),
  (format!("let a={cast}; let h=ag_cast_point(a,3)"),2),
  (format!("for i in 0..4 {{let a={cast}}}; let h=ag_cast_hit(4)"),10),
  (format!("let a={cast}; let b=ag_cast_segment(9,-1,0,0.0,0.0,0.0,1.0,0.0,0.0); let h=ag_cast_hit(a)"),7),
 ] {
  let mut s=scene(&update,&[(2,3,BOX)]);assert_eq!(tick(&mut s,1),error,"{update}");assert_eq!(s.int("probe_token",&[]).unwrap(),0);
 }
    let mut s = scene(
        &format!("if ag_tick()==1 {{observed={cast}}} else {{let h=ag_cast_hit(observed)}}"),
        &[(2, 3, BOX)],
    );
    assert_eq!(tick(&mut s, 1), 0);
    assert_eq!(tick(&mut s, 2), 13);
    let mut s = scene(&format!("for i in 0..4 {{let a={cast}}}"), &[(2, 3, BOX)]);
    assert_eq!(tick(&mut s, 1), 0);
    assert_eq!(tick(&mut s, 2), 0);
    let mut s = scene(&format!("let a={cast}"), &[(2, 3, BOX)]);
    s.call("probe_exhaust", &[], ResultKind::Unit).unwrap();
    assert_eq!(tick(&mut s, 1), 14);
}
#[test]
fn native_nonfinite_values_cross_parameter_local_return_and_container() {
    for value in [
        "(-1.0).sqrt()",
        "9999.0.exp()",
        "0.0 - 9999.0.exp()",
        "[(-1.0).sqrt()][0]",
    ] {
        let mut s=scene(&format!("let v={value}; let q=ag_cast_segment(3,-1,0,v,0.0,0.0,1.0,0.0,0.0); let good=ag_cast_segment(3,-1,0,0.0,0.0,0.0,1.0,0.0,0.0)"),&[(2,3,BOX)]);
        assert_eq!(tick(&mut s, 1), 12, "{value}");
    }
    let mut s=scene("fn invalid() float {return (-1.0).sqrt()}; let q=ag_cast_segment(3,-1,0,invalid(),0.0,0.0,1.0,0.0,0.0)",&[(2,3,BOX)]);
    assert_eq!(tick(&mut s, 1), 12);
}
#[test]
fn nearest_and_parent_cache_mutations_are_detected_by_real_consumer() {
    let update="ag_set_local(0,0,2.0); let a=ag_cast_segment(3,-1,0,0.0,0.0,0.0,8.0,0.0,0.0); if ag_cast_t(a)!=0.125 {observed=1}; ag_set_local(0,0,4.0); let b=ag_cast_segment(3,-1,0,0.0,0.0,0.0,8.0,0.0,0.0); if ag_cast_t(b)!=0.375 {observed=2}";
    let rows = [(5, 3, [0.; 7]), (2, 3, BOX)];
    let mut accepted = scene(update, &rows);
    assert_eq!(tick(&mut accepted, 1), 0);
    assert_eq!(accepted.int("probe_observed", &[]).unwrap(), 0);
    let mut wrong = scene_custom(update, &rows, false, |s| {
        assert!(s.contains("parent_value=_ag_worlds[_ag_parent*3+axis]"));
        s.replace(
            "parent_value=_ag_worlds[_ag_parent*3+axis]",
            "parent_value=0.0",
        )
    });
    assert_eq!(tick(&mut wrong, 1), 0);
    assert_ne!(wrong.int("probe_observed", &[]).unwrap(), 0);
    let update = "let q=ag_cast_segment(3,-1,0,-4.0,0.0,0.0,4.0,0.0,0.0); observed=ag_cast_node(q)";
    let rows = [(2, 3, BOX), (2, 3, [1., -2., -1., -1., 0., 1., 1.])];
    let mut accepted = scene(update, &rows);
    assert_eq!(tick(&mut accepted, 1), 0);
    assert_eq!(accepted.int("probe_observed", &[]).unwrap(), 1);
    let mut wrong = scene_custom(update, &rows, false, |s| {
        assert!(s.contains("target=node; fraction=enter"));
        s.replace(
            "target=node; fraction=enter",
            "if target<0 {target=node; fraction=enter}",
        )
    });
    assert_eq!(tick(&mut wrong, 1), 0);
    assert_ne!(wrong.int("probe_observed", &[]).unwrap(), 1);
}

#[test]
fn actual_two_dimensional_mixed_results_control() {
    let raw="ag_set_local(0,0,2.0); let a=ag_cast_segment(2,-1,0,-50.0,0.0,0.0,50.0,0.0,0.0); let h=ag_cast_hit(a); let n=ag_cast_node(a); let t=ag_cast_t(a); let p=ag_cast_point(a,0); let f=ag_cast_normal(a,0); ag_set_local(0,0,4.0); let b=ag_cast_segment(2,-1,0,-50.0,0.0,0.0,50.0,0.0,0.0); let h2=ag_cast_hit(b); let n2=ag_cast_node(b); let t2=ag_cast_t(b); let p2=ag_cast_point(b,0); let f2=ag_cast_normal(b,0); for i in 0..6 {let q=ag_aabb_min(7,0)}; if h!=1 {observed=1}; if h2!=1 {observed=2}; if p < -23.001 || p > -22.999 {observed=3}; if p2 < -21.001 || p2 > -20.999 {observed=4}; if f!= -1.0 {observed=5}; if f2!= -1.0 {observed=6}";
    let mut rows = vec![(0, 2, [1., -25., -25., 0., 25., 25., 0.]); 8];
    rows[0] = (5, 2, [0.; 7]);
    let mut s = scene(raw, &rows);
    assert_eq!(tick(&mut s, 1), 0);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
}

#[test]
fn every_entry_axis_and_sign_has_the_expected_point_and_normal() {
    for axis in 0..3 {
        for sign in [-1, 1] {
            let mut a = [0.; 3];
            let mut b = [0.; 3];
            a[axis] = -2. * sign as f32;
            b[axis] = 2. * sign as f32;
            let mut expected_point = [0.; 3];
            let mut expected_normal = [0.; 3];
            expected_point[axis] = -sign as f32;
            expected_normal[axis] = -sign as f32;
            let mut update=format!("let q=ag_cast_segment(3,-1,0,{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}); if ag_cast_t(q)!=0.25 {{observed=1}};",a[0],a[1],a[2],b[0],b[1],b[2]);
            for k in 0..3 {
                update.push_str(&format!("if ag_cast_point(q,{k})!={:.1} || ag_cast_normal(q,{k})!={:.1} {{observed=2}};",expected_point[k],expected_normal[k]));
            }
            let mut s = scene(&update, &[(2, 3, BOX)]);
            assert_eq!(tick(&mut s, 1), 0);
            assert_eq!(
                s.int("probe_observed", &[]).unwrap(),
                0,
                "axis={axis} sign={sign}"
            );
        }
    }
}
