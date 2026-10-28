use autographics_auto_driver::authoring::behavior::{Api, SourceSnapshot};
use std::path::Path;
#[test]
fn explicit_motion_version_identity() {
    assert_eq!(
        Api::parse("autographics.behavior/5").unwrap(),
        Api::MotionV5
    );
    assert_eq!(Api::MotionV5.number(), 5);
    assert_eq!(Api::MotionV5.profile(), 11);
    assert!(Api::parse("autographics.behavior/6").is_err());
    let raw = b"fn app_start(){}\nfn app_update(dt float){}\n".to_vec();
    let old =
        SourceSnapshot::from_bytes_for(Path::new("user.at"), raw.clone(), Api::SweepV4).unwrap();
    let new =
        SourceSnapshot::from_bytes_for(Path::new("user.at"), raw.clone(), Api::MotionV5).unwrap();
    assert_eq!(new.bytes, raw);
    assert_eq!(old.user_hash, new.user_hash);
    assert_ne!(old.combined_hash, new.combined_hash);
}

#[test]
fn motion_runtime_header_and_legacy_identity() {
    use autographics_auto_driver::{ResultKind, Session};
    let raw = b"fn app_start(){}\nfn app_update(dt float){}\n".to_vec();
    for api in [
        Api::V1,
        Api::SpatialV2,
        Api::SegmentV3,
        Api::SweepV4,
        Api::MotionV5,
    ] {
        let snap =
            SourceSnapshot::from_bytes_for(Path::new("header.at"), raw.clone(), api).unwrap();
        let mut vm = Session::new(&snap.source).unwrap();
        vm.call("main", &[], ResultKind::Unit).unwrap();
        assert_eq!(vm.int("_ag_profile", &[]).unwrap(), api.profile());
        assert_eq!(vm.int("_ag_version", &[]).unwrap(), api.number());
    }
}

#[test]
fn new_names_only_reserved_in_explicit_v5() {
    let raw=b"fn ag_move_and_slide(n int) int {return n}\nfn ag_motion_status(n int) int {return n}\nfn app_start(){}\nfn app_update(dt float){}\n";
    for api in [
        Api::V1,
        Api::SpatialV2,
        Api::SegmentV3,
        Api::SweepV4,
        Api::MotionV5,
    ] {
        assert_eq!(
            SourceSnapshot::from_bytes_for(Path::new("old-helper.at"), raw.to_vec(), api).is_ok(),
            api != Api::MotionV5
        );
    }
}
#[test]
fn forged_motion_snapshot_rejected_before_runtime() {
    use autographics_auto_driver::authoring::{document::Document, host::Host};
    let mut h = Host::new(Document::sample()).unwrap();
    let raw = b"fn app_start(){}\nfn app_update(dt float){}\n";
    for field in ["source", "api", "bridge", "version", "regions"] {
        let mut snap =
            SourceSnapshot::from_bytes_for(Path::new("raw.at"), raw.to_vec(), Api::MotionV5)
                .unwrap();
        match field {
            "source" => snap.source.push_str("\nfn surprise(){}"),
            "api" => snap.api_hash = "0".repeat(64),
            "bridge" => snap.bridge_hash = "0".repeat(64),
            "version" => snap.version = Api::SweepV4.version().into(),
            _ => snap.regions.clear(),
        };
        assert!(h.start_user_snapshot(snap).is_err(), "{field}");
        assert!(h.play.is_none());
    }
}
