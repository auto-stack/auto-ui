use autographics_auto_driver::authoring::{document::*, preview::SOURCE, session::*};
fn settle(e: &mut Editor) {
    while !e.pump().unwrap() {}
    e.commit().unwrap()
}
#[test]
fn transaction_history_failure_recovery() {
    let mut e = Editor::new(Document::sample()).unwrap();
    e.select(Some(1)).unwrap();
    let saved = e.document.hash().unwrap();
    let old = e.stamp;
    let before = e.document.clone();
    let mut a = e.document.objects[0].clone();
    a.translation[0] += 40.0;
    e.begin(e.stamp, Edit::Replace(a.clone())).unwrap();
    assert!(e.pending());
    assert!(e.begin(e.stamp, Edit::Delete(2)).is_err());
    assert_eq!(e.document, before);
    settle(&mut e);
    assert_eq!(e.document.objects[1], before.objects[1]);
    assert!(e.dirty());
    e.confirm_saved(e.stamp, e.document.hash().unwrap());
    assert!(!e.dirty());
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    assert!(e.dirty());
    assert_eq!(e.document.hash().unwrap(), saved);
    e.begin(e.stamp, Edit::Redo).unwrap();
    settle(&mut e);
    assert!(!e.dirty());
    assert!(e.begin(old, Edit::Delete(1)).is_err());
    let snapshot = e.document.clone();
    let history = e.history();
    let wrong = SOURCE.replace(
        "return fields[i * 12 + f]",
        "return fields[i * 12 + f] + 1.0",
    );
    a.translation[0] += 1.0;
    e.begin_source(e.stamp, Edit::Replace(a.clone()), &wrong)
        .unwrap();
    loop {
        match e.pump() {
            Ok(false) => {}
            Ok(true) => panic!("bad Auto output admitted"),
            Err(_) => break,
        }
    }
    assert_eq!(e.document, snapshot);
    assert_eq!(e.history(), history);
    e.begin(e.stamp, Edit::Replace(a)).unwrap();
    while !e.pump().unwrap() {}
    e.reject();
    assert_eq!(e.document, snapshot);
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    let mut a = e.document.objects[0].clone();
    a.name = "branch".into();
    e.begin(e.stamp, Edit::Replace(a)).unwrap();
    settle(&mut e);
    assert_eq!(e.history().1, 0);
    assert!(e.begin(e.stamp, Edit::Redo).is_err());
    let s = e.stamp;
    e.begin(s, Edit::Delete(1)).unwrap();
    settle(&mut e);
    assert_eq!(e.selection, None);
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    assert_eq!(e.document.objects[0].id, 1);
    let owner = e.preview.vm.owner_probe();
    assert!(owner());
    e.close();
    assert!(!owner());
    assert!(!e.confirm_saved(s, "old".into()));
}
#[test]
fn budgets_noop_stale_selection_and_open() {
    let mut e = Editor::new(Document::sample()).unwrap();
    e.select(Some(1)).unwrap();
    assert!(!e
        .begin(e.stamp, Edit::Replace(e.document.objects[0].clone()))
        .unwrap());
    assert_eq!(e.history().0, 0);
    let stale = e.document.objects[0].clone();
    e.select(Some(2)).unwrap();
    assert!(e.begin(e.stamp, Edit::Replace(stale)).is_err());
    e.select(Some(1)).unwrap();
    for i in 0..66 {
        let mut o = e.document.objects[0].clone();
        o.name = format!("step{i}");
        e.begin(e.stamp, Edit::Replace(o)).unwrap();
        settle(&mut e);
    }
    assert_eq!(e.history().0, 64);
    let before = e.document.clone();
    e.history_limit = 1;
    assert!(e.begin(e.stamp, Edit::Delete(1)).is_err());
    assert_eq!(e.document, before);
    let stamp = e.stamp;
    e.begin_open(Document::sample()).unwrap();
    settle(&mut e);
    assert_ne!(stamp.epoch, e.stamp.epoch);
    assert_eq!(e.history(), (0, 0, 0));
    assert!(!e.dirty());
    assert!(e.begin(stamp, Edit::Delete(1)).is_err());
}
#[test]
fn full_128_object_config_is_accepted() {
    let mut d = Document::empty("capacity".into());
    for i in 1..=128 {
        d.objects.push(Object::new(
            i,
            if i % 2 == 0 { Kind::Mob } else { Kind::Circle },
        ));
    }
    let mut e = Editor::new(d.clone()).unwrap();
    assert_eq!(e.preview.objects, d.objects);
    e.select(Some(128)).unwrap();
    let mut o = e.document.objects[127].clone();
    o.translation[0] = 100000.0;
    e.begin(e.stamp, Edit::Replace(o)).unwrap();
    settle(&mut e);
    assert_eq!(e.preview.objects[127].translation[0], 100000.0);
    assert_eq!(e.document.objects.len(), 128);
}
#[test]
fn dropped_configuration_and_wrong_target_controls_preserve_last_good() {
    for source in [
        SOURCE.replace("fields.push(value)", "fields.push(0.0)"),
        SOURCE.replace("return identities[i]", "return identities[i] + 1"),
    ] {
        let mut e = Editor::new(Document::sample()).unwrap();
        e.select(Some(1)).unwrap();
        let before = e.document.clone();
        let mut o = e.document.objects[0].clone();
        o.translation[0] += 5.0;
        e.begin_source(e.stamp, Edit::Replace(o.clone()), &source)
            .unwrap();
        loop {
            match e.pump() {
                Ok(false) => {}
                Ok(true) => panic!("configuration/target mutation admitted"),
                Err(_) => break,
            }
        }
        assert_eq!(e.document, before);
        assert_eq!(e.history(), (0, 0, 0));
        assert_eq!(e.selection, Some(1));
        e.begin(e.stamp, Edit::Replace(o)).unwrap();
        settle(&mut e);
        assert_eq!(
            e.preview.objects[0].translation[0],
            before.objects[0].translation[0] + 5.0
        );
    }
}
