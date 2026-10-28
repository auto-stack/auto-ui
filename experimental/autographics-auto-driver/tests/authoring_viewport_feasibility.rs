use autographics_auto_driver::authoring::{document::*, host::Host, interaction, session::Edit};
use serde_json::json;
fn settle(h: &mut Host) {
    h.settle().unwrap();
    assert!(!h.gesture_active());
}
#[test]
fn whole_path_auto_temporary_release_cancel_and_final_failure() {
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let original = h.editor.document.clone();
    let hash = original.hash().unwrap();
    let p = [
        400.0 + original.objects[0].translation[0],
        190.0 - original.objects[0].translation[1],
    ];
    h.pointer_down(p);
    assert_eq!(h.editor.selection, Some(1));
    assert!(h.gesture_active());
    h.frame(None).unwrap();
    let accepted_pixels = h.rt.capture(h.two, "accepted").unwrap().0;
    h.pointer_move([p[0] + 20.0, p[1] + 10.0]);
    for _ in 0..200 {
        h.frame(None).unwrap();
    }
    assert_eq!(h.editor.document, original);
    assert_eq!(h.editor.history().0, 0);
    assert!(!h.editor.dirty());
    assert_ne!(h.rt.capture(h.two, "temporary").unwrap().0, accepted_pixels);
    h.pointer_up([p[0] + 30.0, p[1] + 15.0]);
    settle(&mut h);
    assert_eq!(h.editor.history().0, 1);
    assert_eq!(h.editor.stamp.revision, 1);
    assert_eq!(
        h.editor.document.objects[0].translation,
        [
            original.objects[0].translation[0] + 30.0,
            original.objects[0].translation[1] - 15.0,
            0.0
        ]
    );
    assert_eq!(h.editor.document.objects[1..], original.objects[1..]);
    h.editor.begin(h.editor.stamp, Edit::Undo).unwrap();
    settle(&mut h);
    assert_eq!(h.editor.document.hash().unwrap(), hash);
    h.pointer_down(p);
    h.pointer_move([p[0] + 40.0, p[1]]);
    for _ in 0..100 {
        h.frame(None).unwrap();
    }
    h.cancel_gesture();
    h.frame(None).unwrap();
    assert_eq!(h.editor.document, original);
    assert_eq!(h.rt.capture(h.two, "cancelled").unwrap().0, accepted_pixels);
    let history = h.editor.history();
    let stamp = h.editor.stamp;
    h.pointer_down(p);
    h.pointer_move([p[0] + 20.0, p[1]]);
    for _ in 0..100 {
        h.frame(None).unwrap();
    }
    h.fail_resource = true;
    h.pointer_up([p[0] + 50.0, p[1]]);
    settle(&mut h);
    assert_eq!(h.editor.document, original);
    assert_eq!(h.editor.history(), history);
    assert_eq!(h.editor.stamp, stamp);
    assert_eq!(
        h.rt.capture(h.two, "final-failed").unwrap().0,
        accepted_pixels
    );
    h.fail_resource = false;
    h.pointer_down(p);
    h.pointer_up([p[0] + 10.0, p[1]]);
    settle(&mut h);
    assert_eq!(h.editor.stamp.revision, stamp.revision + 1);
    // Original GLB model, each projected world axis, exactly one history transaction.
    for axis in 0..3 {
        let o = h
            .editor
            .document
            .objects
            .iter()
            .find(|o| o.kind == Kind::Player)
            .unwrap()
            .clone();
        h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"select","payload":o.id}).to_string()).unwrap();
        h.frame(None).unwrap();
        let (_, a, d) = interaction::axes(&o, h.camera())
            .into_iter()
            .find(|x| x.0 == axis)
            .unwrap();
        let start = a + d.normalize() * 40.0;
        let end = start + d * 0.4;
        let before = h.editor.document.clone();
        let revision = h.editor.stamp.revision;
        h.pointer_down([start.x, start.y + 400.0]);
        assert!(h.gesture_active(), "axis {axis}");
        h.pointer_move([end.x, end.y + 400.0]);
        for _ in 0..100 {
            h.frame(None).unwrap();
        }
        assert_eq!(h.editor.document, before);
        h.pointer_up([end.x, end.y + 400.0]);
        settle(&mut h);
        let target = h
            .editor
            .document
            .objects
            .iter()
            .find(|x| x.id == o.id)
            .unwrap();
        for i in 0..3 {
            assert!(
                (target.translation[i] - o.translation[i] - if i == axis { 0.4 } else { 0.0 })
                    .abs()
                    < 1e-3,
                "axis {axis}/{i}"
            );
        }
        let mut expected = before.clone();
        expected
            .objects
            .iter_mut()
            .find(|x| x.id == o.id)
            .unwrap()
            .translation[axis] += 0.4;
        for (actual, want) in h.editor.document.objects.iter().zip(&expected.objects) {
            let mut adjusted = actual.clone();
            adjusted.translation = want.translation;
            assert_eq!(&adjusted, want, "all non-translation fields and siblings");
        }
        assert_eq!(h.editor.stamp.revision, revision + 1);
    }
}
