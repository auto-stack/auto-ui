include!("support/sweep.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
#[test]
fn explicit_v4_managed_identity_legacy_helpers_and_runtime_header() {
    let raw = b"fn app_start(){}\nfn app_update(dt float){}\n";
    let raw = String::from_utf8(raw.to_vec())
        .unwrap()
        .replace("\\n", "\n")
        .into_bytes();
    for api in [Api::V1, Api::SpatialV2, Api::SegmentV3, Api::SweepV4] {
        let snap = SourceSnapshot::from_bytes_for(Path::new("raw.at"), raw.clone(), api).unwrap();
        let mut vm = Session::new(&snap.source).unwrap();
        vm.call("main", &[], ResultKind::Unit).unwrap();
        assert_eq!(vm.int("_ag_profile", &[]).unwrap(), api.profile());
        assert_eq!(vm.int("_ag_version", &[]).unwrap(), api.number());
        let helper=b"fn ag_sweep_hit(n int) int {return 17}\nfn app_start(){}\nfn app_update(dt float){}\n";
        assert_eq!(
            SourceSnapshot::from_bytes_for(Path::new("h.at"), helper.to_vec(), api).is_ok(),
            api != Api::SweepV4
        );
        eprintln!(
            "MANAGED {} bytes={}",
            api.version(),
            snap.source.len() - raw.len()
        );
    }
}
#[test]
fn shared_tokens_kinds_expiry_exhaustion_and_credit_boundaries() {
    let sweep = "ag_sweep_aabb(0,0,1.0,0.0,0.0)";
    let cast = "ag_cast_segment(3,-1,0,-4.0,0.0,0.0,4.0,0.0,0.0)";
    for (script,error) in [(format!("let a={cast};let b={sweep};if b<=a {{observed=1}};let h=ag_cast_hit(b)"),13),(format!("let a={sweep};let b={cast};if b<=a {{observed=1}};let h=ag_sweep_hit(b)"),13),(format!("let a={sweep};let b={cast};let h=ag_sweep_hit(a)"),13),(format!("let a={sweep};for i in 0..16 {{let h=ag_sweep_hit(a)}};let h=ag_sweep_hit(a)"),10),(format!("let a={sweep};let h=ag_sweep_origin(a,3)"),2),(format!("let a={cast};let h=ag_cast_hit(a);let n=ag_cast_node(a);let b={sweep};for i in 0..6 {{let h=ag_sweep_hit(b)}}"),0),(format!("let a={sweep};ag_set_local(0,0,4.0);if ag_sweep_origin(a,0)!=0.0 {{observed=1}};let z=ag_aabb_min(0,0);if ag_sweep_origin(a,0)!=0.0 {{observed=2}}"),0)]{
 let mut s=scene(&script,&[(2,3,BOX),(2,3,BOX)]);s.call("_ag_tick",&[Int(1),Float(0.016)],ResultKind::Unit).unwrap();assert_eq!(s.int("_ag_status",&[]).unwrap(),error,"{script}");assert_eq!(s.int("probe_observed",&[]).unwrap(),0);}
    let mut s = scene(&format!("let q={sweep}"), &[(2, 3, BOX)]);
    s.call("probe_exhaust", &[], ResultKind::Unit).unwrap();
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 14);
}

#[test]
fn callback_expiry_and_failed_attempt_invalidate_shared_slot() {
    let update="if ag_tick()==1 {observed=ag_sweep_aabb(0,0,1.0,0.0,0.0)} else {let h=ag_sweep_hit(observed)}";
    let mut s = scene(update, &[(2, 3, BOX)]);
    s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("probe_token", &[]).unwrap(), 0);
    s.call("_ag_tick", &[Int(2), Float(0.016)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 13);
    for query in [
        "ag_sweep_aabb(-1,0,1.0,0.0,0.0)",
        "ag_cast_segment(9,-1,0,0.0,0.0,0.0,1.0,0.0,0.0)",
    ] {
        let mut s = scene(
            &format!("let a=ag_sweep_aabb(0,0,1.0,0.0,0.0);let b={query};let h=ag_sweep_hit(a)"),
            &[(2, 3, BOX)],
        );
        s.call("_ag_tick", &[Int(1), Float(0.016)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.int("probe_token", &[]).unwrap(), 0);
        assert_ne!(s.int("_ag_status", &[]).unwrap(), 0);
    }
}
