use autographics_auto_driver::authoring::{
    behavior::VERSION,
    behavior_project::*,
    document::*,
    play::{Play, State},
    session::*,
    storage::*,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
fn settle(e: &mut Editor) {
    while !e.pump().unwrap() {}
    e.commit().unwrap();
}
fn fixture() -> (Editor, Storage, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "ag845-{}-{}",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    let root = root.join(format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    (
        Editor::new(Document::sample()).unwrap(),
        Storage::default(),
        root,
    )
}
fn attach(e: &mut Editor, n: i32) {
    let c = CodePayload::from_bytes(
        Path::new("deleted-original.at"),
        format!(
            "fn app_start(){{ag_set_local(ag_actor(2),0,{n}.0)}}\nfn app_update(dt float){{}}\n"
        )
        .into_bytes(),
    )
    .unwrap();
    let b = Binding {
        api: VERSION.into(),
        source: c.identity().clone(),
        actors: Actors {
            two_d: Some(1),
            three_d: None,
        },
    };
    e.begin(
        e.stamp,
        Edit::Behavior {
            binding: Some(b),
            code: Some(c),
        },
    )
    .unwrap();
    settle(e);
}
#[test]
fn whole_path_move_open_run_and_before_after_commit_failure() {
    let (mut e, mut s, root) = fixture();
    let entry = root.join("scene.agscene");
    s.save(&mut e, &entry, Fault::None).unwrap();
    let original = fs::read(&entry).unwrap();
    attach(&mut e, 7);
    for fault in [
        Fault::CodeAfterPrepare,
        Fault::CodeAfterInstall,
        Fault::BeforeReplace,
    ] {
        assert!(s.save(&mut e, &entry, fault).is_err());
        assert_eq!(fs::read(&entry).unwrap(), original);
        assert!(e.dirty());
    }
    assert_eq!(
        s.save(&mut e, &entry, Fault::CodeClosureAfterReplace)
            .unwrap(),
        Saved::VerificationPending
    );
    assert!(s.unverified());
    assert!(e.dirty());
    let blob = root.join(e.code.as_ref().unwrap().identity().relative_path());
    let raw = fs::read(&blob).unwrap();
    fs::write(&blob, b"broken").unwrap();
    assert!(s.verify(&mut e).is_err());
    assert!(s.unverified());
    fs::write(&blob, &raw).unwrap();
    assert!(s.verify(&mut e).unwrap());
    assert!(!e.dirty());
    let other = root.join("moved");
    fs::create_dir(&other).unwrap();
    let target = other.join("entry.agscene");
    fs::remove_dir_all(root.join("scripts")).unwrap();
    s.save(&mut e, &target, Fault::None).unwrap();
    let (d, a, c, _) = Storage::load_project(&target, &Default::default()).unwrap();
    let opened = Editor::with_project(d, a, c).unwrap();
    assert!(!opened.dirty());
    assert_eq!(opened.history(), (0, 0, 0));
    let c = opened.code.as_ref().unwrap();
    let mut play = Play::new_user(
        &opened,
        opened.document.behavior.as_ref().unwrap().actors.ids(),
        c.snapshot(&target.parent().unwrap().join(c.identity().relative_path()))
            .unwrap(),
    )
    .unwrap();
    play.pause("control");
    let limit = Instant::now() + Duration::from_secs(30);
    while play.state == State::Starting {
        assert!(Instant::now() < limit);
        play.pump_until(Instant::now() + Duration::from_millis(8))
            .unwrap();
        if play.candidate().is_some() {
            play.accept().unwrap();
        }
    }
    assert_eq!(
        play.last.as_ref().unwrap().document.objects[0].translation[0],
        7.0
    );
    play.stop();
}
#[test]
fn reserved_targets_conflicts_and_old_revision_receipt() {
    let (mut e, mut s, root) = fixture();
    attach(&mut e, 3);
    let target = root.join("entry.agscene");
    let old = Snapshot::capture(&e).unwrap();
    attach(&mut e, 4);
    assert_eq!(
        s.save_snapshot(&mut e, &target, old, Fault::None).unwrap(),
        Saved::Confirmed
    );
    assert!(e.dirty());
    for name in [
        "scripts",
        "assets",
        ".autographics-assets-lock",
        "entry.agscene.authoring-lock",
        ".entry.authoring-1-2",
        ".ag-code-1-2.tmp",
    ] {
        assert!(s.save(&mut e, &root.join(name), Fault::None).is_err());
    }
    s.save(
        &mut e,
        &root.join("my.authoring-draft.agscene"),
        Fault::None,
    )
    .unwrap();
    s.save(&mut e, &root.join(".ag-project.agscene"), Fault::None)
        .unwrap();
    let blob = root.join(e.code.as_ref().unwrap().identity().relative_path());
    fs::write(&blob, b"wrong").unwrap();
    let before = fs::read(&target).unwrap();
    assert!(s.save(&mut e, &target, Fault::None).is_err());
    assert_eq!(fs::read(&target).unwrap(), before);
    assert_eq!(fs::read(blob).unwrap(), b"wrong");
}

#[test]
fn disk_quota_unknown_lock_and_receipt_owners() {
    let (mut e, mut s, root) = fixture();
    attach(&mut e, 1);
    let target = root.join("entry.agscene");
    assert_eq!(
        s.save(&mut e, &target, Fault::CodeClosureAfterReplace)
            .unwrap(),
        Saved::VerificationPending
    );
    let weak = std::sync::Arc::downgrade(e.code.as_ref().unwrap());
    e.begin_open(Document::sample()).unwrap();
    settle(&mut e);
    assert!(weak.upgrade().is_some(), "receipt must own its code");
    assert!(s.verify(&mut e).unwrap_err().contains("old epoch"));
    drop(s);
    assert!(weak.upgrade().is_none());
    let (mut e, mut s, root) = fixture();
    attach(&mut e, 2);
    let target = root.join("entry.agscene");
    let scripts = root.join("scripts");
    fs::create_dir(&scripts).unwrap();
    fs::write(scripts.join("foreign.tmp"), b"keep").unwrap();
    assert!(s.save(&mut e, &target, Fault::None).is_err());
    assert_eq!(fs::read(scripts.join("foreign.tmp")).unwrap(), b"keep");
    fs::remove_file(scripts.join("foreign.tmp")).unwrap();
    for i in 0..8 {
        let raw = format!("residual {i}").into_bytes();
        let sha = agp::renderer::capture::sha256_hex(&raw);
        fs::write(scripts.join(format!("{sha}.at")), raw).unwrap();
    }
    assert!(s
        .save(&mut e, &target, Fault::None)
        .unwrap_err()
        .contains("budget"));
    assert!(!target.exists());
    let (mut e, mut s, root) = fixture();
    attach(&mut e, 3);
    let target = root.join("entry.agscene");
    fs::write(root.join(".autographics-assets-lock"), b"owner").unwrap();
    assert!(s.save(&mut e, &target, Fault::None).is_err());
    assert_eq!(
        fs::read(root.join(".autographics-assets-lock")).unwrap(),
        b"owner"
    );
}
#[test]
fn mixed_glb_code_failure_and_complete_readback() {
    use autographics_auto_driver::authoring::assets;
    let (mut e, mut s, root) = fixture();
    let target = root.join("entry.agscene");
    s.save(&mut e, &target, Fault::None).unwrap();
    let old = fs::read(&target).unwrap();
    let asset = assets::decode(
        include_bytes!("../research/asset-probe/two-parts.glb").to_vec(),
        None,
    )
    .unwrap();
    let mut o = Object::new(5, Kind::Model);
    o.resource = Some(asset.identity.key());
    e.begin(
        e.stamp,
        Edit::AssetObject {
            asset: Some(asset.clone()),
            object: o,
            create: true,
        },
    )
    .unwrap();
    settle(&mut e);
    attach(&mut e, 6);
    assert!(s.save(&mut e, &target, Fault::CodeAfterPrepare).is_err());
    assert_eq!(fs::read(&target).unwrap(), old);
    let glb = root
        .join("assets")
        .join(format!("{}.glb", asset.identity.sha256));
    assert!(glb.exists());
    assert!(s.residuals.contains(&glb.canonicalize().unwrap()));
    s.save(&mut e, &target, Fault::CodeClosureAfterReplace)
        .unwrap();
    let bytes = fs::read(&glb).unwrap();
    fs::write(&glb, b"bad").unwrap();
    assert!(s.verify(&mut e).is_err());
    assert!(e.dirty());
    fs::write(&glb, &bytes).unwrap();
    assert!(s.verify(&mut e).unwrap());
    assert!(!e.dirty());
    assert!(s.residuals.is_empty());
}

#[test]
fn raw_crlf_and_runtime_fault_roundtrip_without_callbacks() {
    use autographics_auto_driver::authoring::behavior::VERSION;
    let (mut e, mut s, root) = fixture();
    let raw = b"fn app_start(){while true {}}\r\nfn app_update(dt float){}\r\n".to_vec();
    let c = CodePayload::from_bytes(Path::new("removed.at"), raw.clone()).unwrap();
    let b = Binding {
        api: VERSION.into(),
        source: c.identity().clone(),
        actors: Actors {
            two_d: None,
            three_d: None,
        },
    };
    e.begin(
        e.stamp,
        Edit::Behavior {
            binding: Some(b),
            code: Some(c),
        },
    )
    .unwrap();
    settle(&mut e);
    let target = root.join("entry.agscene");
    s.save(&mut e, &target, Fault::None).unwrap();
    let (d, a, c, _) = Storage::load_project(&target, &Default::default()).unwrap();
    assert_eq!(c.as_ref().unwrap().bytes(), raw);
    let opened = Editor::with_project(d, a, c).unwrap();
    assert!(!opened.dirty());
    assert_eq!(opened.history(), (0, 0, 0));
}
