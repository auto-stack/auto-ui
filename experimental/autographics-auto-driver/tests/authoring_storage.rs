use autographics_auto_driver::authoring::{document::*, session::*, storage::*};
fn settle(e: &mut Editor) {
    while !e.pump().unwrap() {}
    e.commit().unwrap()
}
#[test]
fn save_failures_conflicts_pending_and_recovery() {
    let dir = std::env::temp_dir().join(format!("ag-save-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("scene.agscene");
    let foreign = dir.join(".foreign-stage");
    std::fs::write(&foreign, b"foreign").unwrap();
    let mut e = Editor::new(Document::sample()).unwrap();
    let mut s = Storage::default();
    s.save(&mut e, &path, Fault::None).unwrap();
    let old = std::fs::read(&path).unwrap();
    e.select(Some(1)).unwrap();
    let mut a = e.document.objects[0].clone();
    a.name = "modified".into();
    e.begin(e.stamp, Edit::Replace(a)).unwrap();
    settle(&mut e);
    for f in [
        Fault::Prepare,
        Fault::Write,
        Fault::Flush,
        Fault::BeforeReplace,
    ] {
        assert!(s.save(&mut e, &path, f).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), old);
        assert!(e.dirty());
        assert!(!s.unverified());
    }
    assert_eq!(
        s.save(&mut e, &path, Fault::AfterReplaceRead).unwrap(),
        Saved::VerificationPending
    );
    assert!(e.dirty());
    assert!(s.unverified());
    assert_ne!(std::fs::read(&path).unwrap(), old);
    assert!(s.verify(&mut e).unwrap());
    assert!(!e.dirty());
    s.save(&mut e, &path, Fault::None).unwrap();
    let snapshot = Snapshot::capture(&e).unwrap();
    let mut a = e.document.objects[0].clone();
    a.name = "newer".into();
    e.begin(e.stamp, Edit::Replace(a)).unwrap();
    settle(&mut e);
    s.save_snapshot(&mut e, &path, snapshot, Fault::None)
        .unwrap();
    assert!(e.dirty());
    let loaded = Storage::load(&path).unwrap().0;
    assert_eq!(loaded.objects[0].name, "modified");
    s.save(&mut e, &path, Fault::None).unwrap();
    assert!(!e.dirty());
    std::fs::write(&path, b"external").unwrap();
    assert!(s
        .save(&mut e, &path, Fault::None)
        .unwrap_err()
        .contains("external"));
    assert_eq!(std::fs::read(&path).unwrap(), b"external");
    let before = e.document.clone();
    assert!(Storage::load(&path).is_err());
    assert_eq!(e.document, before);
    let lock = dir.join("scene.agscene.authoring-lock");
    std::fs::write(&lock, b"foreign lock").unwrap();
    assert!(s
        .save(&mut e, &path, Fault::None)
        .unwrap_err()
        .contains("occupied"));
    assert_eq!(std::fs::read(&lock).unwrap(), b"foreign lock");
    assert_eq!(std::fs::read(&foreign).unwrap(), b"foreign");
    for p in [path, foreign, lock] {
        std::fs::remove_file(p).unwrap();
    }
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    std::fs::remove_dir(dir).unwrap();
}
#[test]
fn old_epoch_cannot_save_or_confirm() {
    let mut e = Editor::new(Document::sample()).unwrap();
    let s = Snapshot::capture(&e).unwrap();
    e.begin_open(Document::sample()).unwrap();
    settle(&mut e);
    assert!(!e.confirm_saved(s.stamp, s.hash.clone()));
    assert!(Storage::default()
        .save_snapshot(
            &mut e,
            std::path::Path::new("never-created.agscene"),
            s,
            Fault::None
        )
        .is_err());
}
#[cfg(windows)]
#[test]
fn occupied_os_file_preserves_bytes_then_recovers() {
    use std::os::windows::fs::OpenOptionsExt;
    let dir = std::env::temp_dir().join(format!("ag-occupied-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("scene.agscene");
    let mut e = Editor::new(Document::sample()).unwrap();
    let mut s = Storage::default();
    s.save(&mut e, &path, Fault::None).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();
    assert!(s.save(&mut e, &path, Fault::None).is_err());
    drop(held);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    s.save(&mut e, &path, Fault::None).unwrap();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
