use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    play::{Action, State},
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn fixture() -> Host {
    let mut d = Document::empty("spatial-host".into());
    d.objects.push(Object::new(1, Kind::Circle));
    Host::new(d).unwrap()
}
fn settle(h: &mut Host) {
    let deadline = Instant::now() + Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        h.frame(None).unwrap();
        assert!(Instant::now() < deadline)
    }
}
#[test]
fn callback_fault_keeps_last_good_closes_vm_stop_and_new_run_recover() {
    let mut h = fixture();
    h.frame(None).unwrap();
    let original = h.editor.isolation_identity();
    let pix = h.rt.capture(h.two, "initial").unwrap().0;
    let raw=b"fn app_start(){}\nfn app_update(dt float){ag_move_local(0,1.0,0.0,0.0); if ag_tick()==2 {let x=ag_cast_hit(1)} else {let x=ag_aabb_min(0,0)}}\n";
    h.start_user_snapshot(
        SourceSnapshot::from_bytes_for(Path::new("fault.at"), raw.to_vec(), Api::SegmentV3)
            .unwrap(),
    )
    .unwrap();
    h.pause_play("");
    let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
    let spatial = h.play.as_ref().unwrap().spatial_owner_probe().unwrap();
    settle(&mut h);
    for tick in 1..3 {
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .unwrap();
        settle(&mut h);
        if tick == 1 {
            assert_eq!(h.play.as_ref().unwrap().tick(), 1);
        }
    }
    let p = h.play.as_ref().unwrap();
    assert_eq!(p.state, State::Faulted);
    assert_eq!(p.tick(), 1);
    assert_eq!(
        p.last.as_ref().unwrap().document.objects[0].translation[0],
        1.
    );
    assert!(p.error.as_ref().unwrap().contains("InvalidResult"));
    assert!(!owner());
    assert!(h.select_behavior_api(Api::V1).is_err());
    h.stop_play();
    h.frame(None).unwrap();
    assert!(spatial.upgrade().is_none());
    assert_eq!(h.editor.isolation_identity(), original);
    assert_eq!(h.rt.capture(h.two, "restored").unwrap().0, pix);
    let raw = b"fn app_start(){let x=ag_aabb_min(0,0)}\nfn app_update(dt float){}\n";
    h.start_user_snapshot(
        SourceSnapshot::from_bytes_for(Path::new("fixed.at"), raw.to_vec(), Api::SegmentV3)
            .unwrap(),
    )
    .unwrap();
    h.pause_play("cancel starting");
    settle(&mut h);
    assert_eq!(h.play.as_ref().unwrap().state, State::Paused);
    assert_eq!(h.play.as_ref().unwrap().tick(), 0);
    h.stop_play();
}

#[test]
fn explicit_file_version_and_forged_snapshot_do_not_fallback() {
    let mut h = fixture();
    let file = std::env::temp_dir().join(format!("ag848-version-{}.at", std::process::id()));
    let raw = b"fn app_start(){let v=ag_aabb_min(0,0)}\nfn app_update(dt float){}\n";
    std::fs::write(&file, raw).unwrap();
    h.select_behavior(Some(file.to_string_lossy().into()))
        .unwrap();
    h.select_behavior_api(Api::SegmentV3).unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("");
    settle(&mut h);
    assert_eq!(
        h.play
            .as_ref()
            .unwrap()
            .user_source
            .as_ref()
            .unwrap()
            .version,
        Api::SegmentV3.version()
    );
    h.stop_play();
    h.select_behavior_api(Api::V1).unwrap();
    assert!(h.start_selected_play().is_err());
    assert!(h.play.is_none());
    for field in ["version", "source", "base"] {
        let mut s = SourceSnapshot::from_bytes_for(&file, raw.to_vec(), Api::SegmentV3).unwrap();
        match field {
            "version" => s.version = Api::V1.version().into(),
            "source" => s.source = s.source.replace("return 9", "return 7"),
            _ => s.base_scene = Some(serde_json::json!({"spatial_hash":"foreign"})),
        }
        assert!(h.start_user_snapshot(s).is_err(), "{field}");
        assert!(h.play.is_none());
    }
}

#[test]
fn actual_host_128_wide_deep_original_initialization_and_mixed_query_budget() {
    use autographics_auto_driver::authoring::document::{Dimension, Group};
    for deep in [false, true] {
        let mut d = Document::empty(format!("128-{deep}"));
        let groups = if deep { 126 } else { 1 };
        for i in 0..groups {
            let mut g = Group::new(i + 1, Dimension::Two);
            if i > 0 {
                g.parent = Some(i)
            };
            d.groups.push(g);
        }
        for i in groups..128 {
            let mut o = Object::new(i + 1, Kind::Circle);
            o.parent = Some(groups);
            d.objects.push(o);
        }
        let mut h = Host::new(d).unwrap();
        let raw=b"fn app_start(){}\nfn app_update(dt float){ag_set_local(0,0,2.0); let a=ag_cast_segment(2,-1,0,-50.0,0.0,0.0,50.0,0.0,0.0); let h=ag_cast_hit(a); let n=ag_cast_node(a); let t=ag_cast_t(a); let p=ag_cast_point(a,0); let f=ag_cast_normal(a,0); ag_set_local(0,0,4.0); let b=ag_cast_segment(2,-1,0,-50.0,0.0,0.0,50.0,0.0,0.0); let h2=ag_cast_hit(b); let n2=ag_cast_node(b); let t2=ag_cast_t(b); let p2=ag_cast_point(b,0); let f2=ag_cast_normal(b,0); for i in 0..6 {let q=ag_aabb_min(127,0)}; if h!=1 || h2!=1 || p < -23.001 || p > -22.999 || p2 < -21.001 || p2 > -20.999 || f!= -1.0 || f2!= -1.0 {ag_set_visible(127,0)}}\n";
        let start = Instant::now();
        h.start_user_snapshot(
            SourceSnapshot::from_bytes_for(Path::new("128.at"), raw.to_vec(), Api::SegmentV3)
                .unwrap(),
        )
        .unwrap();
        h.pause_play("");
        settle(&mut h);
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.state, State::Paused, "{:?}", p.error);
        assert_eq!(p.spatial_configuration_calls, 897);
        assert!(start.elapsed() < Duration::from_secs(30));
        eprintln!(
            "128 deep={deep} init_us={} spatial_calls={} config_us={}",
            start.elapsed().as_micros(),
            p.spatial_configuration_calls,
            p.spatial_configuration_us
        );
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .unwrap();
        settle(&mut h);
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.state, State::Paused, "{:?}", p.error);
        assert_eq!(p.tick(), 1);
        assert!(p.last.as_ref().unwrap().objects.iter().all(|o| o.visible));
        assert_eq!(p.last.as_ref().unwrap().objects[0].translation[0], 4.);
        h.stop_play();
    }
}

#[test]
fn stop_during_starting_releases_v3_vm_and_descriptor_without_publication() {
    let mut h = fixture();
    h.frame(None).unwrap();
    let before = h.editor.isolation_identity();
    h.start_user_snapshot(SourceSnapshot::from_bytes_for(Path::new("cancel-v3.at"),b"fn app_start(){let q=ag_cast_segment(2,-1,0,-50.0,0.0,0.0,50.0,0.0,0.0)}\nfn app_update(dt float){}\n".to_vec(),Api::SegmentV3).unwrap()).unwrap();
    assert_eq!(h.play.as_ref().unwrap().state, State::Starting);
    let vm = h.play.as_ref().unwrap().owner_probe().unwrap();
    let descriptor = h.play.as_ref().unwrap().spatial_owner_probe().unwrap();
    assert!(h.play.as_ref().unwrap().last.is_none());
    h.stop_play();
    h.frame(None).unwrap();
    assert!(h.play.is_none());
    assert!(!vm());
    assert!(descriptor.upgrade().is_none());
    assert_eq!(h.editor.isolation_identity(), before);
}
