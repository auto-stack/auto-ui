use autographics_auto_driver::authoring::{
    behavior::VERSION, behavior_project::*, document::*, session::*,
};
use std::{path::Path, sync::Arc};
fn code(n: usize) -> Code {
    CodePayload::from_bytes(
        Path::new("<memory>.at"),
        format!("//{n}\nfn app_start(){{while true {{}}}}\nfn app_update(dt float){{}}\n")
            .into_bytes(),
    )
    .unwrap()
}
fn bind(c: &Code) -> Edit {
    Edit::Behavior {
        binding: Some(Binding {
            api: VERSION.into(),
            source: c.identity().clone(),
            actors: Actors {
                two_d: Some(1),
                three_d: Some(3),
            },
        }),
        code: Some(c.clone()),
    }
}
fn settle(e: &mut Editor) {
    while !e.pump().unwrap() {}
    e.commit().unwrap();
}
#[test]
fn payload_compile_no_callback_history_stale_cancel_and_quota() {
    let a = code(0);
    let weak = Arc::downgrade(&a);
    let mut e = Editor::new(Document::sample()).unwrap();
    let before = e.isolation_identity();
    let stamp = e.stamp;
    e.begin(stamp, bind(&a)).unwrap();
    assert_eq!(e.isolation_identity(), before);
    e.reject();
    assert!(e.code.is_none());
    e.begin(stamp, bind(&a)).unwrap();
    settle(&mut e);
    assert!(e.dirty());
    e.confirm_saved(e.stamp, e.document.hash().unwrap());
    assert!(!e.begin(e.stamp, bind(&a)).unwrap());
    assert_eq!(e.history().0, 1);
    assert!(e.begin(stamp, Edit::Delete(1)).is_err());
    assert!(e.begin(e.stamp, Edit::Delete(1)).is_err());
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    assert!(e.code.is_none());
    e.begin(e.stamp, Edit::Redo).unwrap();
    settle(&mut e);
    assert!(Arc::ptr_eq(e.code.as_ref().unwrap(), &a));
    assert!(!e.dirty());
    drop(a);
    for i in 1..8 {
        let c = code(i);
        e.begin(e.stamp, bind(&c)).unwrap();
        settle(&mut e);
    }
    let snapshot = e.isolation_identity();
    let ninth = code(8);
    assert!(e
        .begin(e.stamp, bind(&ninth))
        .unwrap_err()
        .contains("budget"));
    assert_eq!(e.isolation_identity(), snapshot);
    e.close();
    assert!(weak.upgrade().is_none());
}
#[test]
fn raw_identity_rejects_bad_sources_and_releases() {
    let valid = b"fn app_start(){}\nfn app_update(dt float){}\n";
    let lf = CodePayload::from_bytes(Path::new("lf.at"), valid.to_vec()).unwrap();
    let crlf = CodePayload::from_bytes(
        Path::new("crlf.at"),
        String::from_utf8(valid.to_vec())
            .unwrap()
            .replace("\n", "\r\n")
            .into_bytes(),
    )
    .unwrap();
    assert_ne!(lf.identity(), crlf.identity());
    assert_eq!(lf.bytes(), valid);
    for bytes in [
        vec![255],
        [vec![0xef, 0xbb, 0xbf], valid.to_vec()].concat(),
        vec![b' '; 262145],
        b"fn app_start(){}".to_vec(),
        b"fn app_start(){undefined_function()}\nfn app_update(dt float){}".to_vec(),
    ] {
        assert!(CodePayload::from_bytes(Path::new("bad.at"), bytes).is_err());
    }
}

#[test]
fn boundary_bytes_history_drop_redo_and_new_epoch() {
    let mut raw = b"fn app_start(){}\nfn app_update(dt float){}\n//".to_vec();
    raw.resize(262144, b'x');
    let exact = CodePayload::from_bytes(Path::new("max.at"), raw.clone()).unwrap();
    assert_eq!(exact.bytes().len(), 262144);
    raw.push(b'x');
    assert!(CodePayload::from_bytes(Path::new("too-big.at"), raw).is_err());
    let mut e = Editor::new(Document::sample()).unwrap();
    let a = code(11);
    let wa = Arc::downgrade(&a);
    e.begin(e.stamp, bind(&a)).unwrap();
    settle(&mut e);
    drop(a);
    let b = code(12);
    let wb = Arc::downgrade(&b);
    e.begin(e.stamp, bind(&b)).unwrap();
    settle(&mut e);
    drop(b);
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    let c = code(13);
    e.begin(e.stamp, bind(&c)).unwrap();
    settle(&mut e);
    assert!(wb.upgrade().is_none(), "new branch releases redo source");
    drop(c);
    let old = e.stamp;
    e.begin_open(Document::sample()).unwrap();
    settle(&mut e);
    assert!(wa.upgrade().is_none());
    assert!(e.begin(old, Edit::Delete(1)).is_err());
    e.history_limit = 1;
    let c = code(14);
    let wc = Arc::downgrade(&c);
    assert!(e.begin(e.stamp, bind(&c)).is_err());
    drop(c);
    assert!(wc.upgrade().is_none());
    assert!(!e.pending());
}

#[test]
fn bound_ancestor_rejection_and_history_eviction_release() {
    let mut d = Document::sample();
    d.groups.push(Group::new(20, Dimension::Two));
    d.objects[0].parent = Some(20);
    let mut e = Editor::new(d).unwrap();
    let a = code(31);
    let weak = Arc::downgrade(&a);
    e.begin(e.stamp, bind(&a)).unwrap();
    settle(&mut e);
    drop(a);
    let before = e.isolation_identity();
    assert!(e
        .begin(e.stamp, Edit::Delete(20))
        .unwrap_err()
        .contains("behavior actor"));
    assert_eq!(e.isolation_identity(), before);
    e.history_limit = e.document.encode().unwrap().len() * 2;
    for i in 32..35 {
        let c = code(i);
        e.begin(e.stamp, bind(&c)).unwrap();
        settle(&mut e);
    }
    assert!(
        weak.upgrade().is_none(),
        "history eviction releases old source"
    );
    assert!(e.history().0 <= 1);
}
