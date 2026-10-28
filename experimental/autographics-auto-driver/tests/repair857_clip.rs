use autographics_auto_driver::authoring::{
    document::{Dimension, Document, Group},
    host::Host,
    interaction,
    view3d::Mode,
};
use glam::Vec3;
#[test]
fn grouped_descendants_respect_near_and_far_clipping() {
    let mut results = Vec::new();
    for (name, perspective, position) in [
        ("orthographic visible", false, [0., 0., 0.]),
        ("orthographic far", false, [0., -500., -500.]),
        ("orthographic near", false, [0., 100., 100.]),
        ("perspective visible", true, [0., 0., 0.]),
        ("perspective far", true, [0., 0., -20000.]),
    ] {
        let mut doc = Document::sample();
        doc.objects.retain(|o| o.id == 3);
        doc.groups.push(Group::new(10, Dimension::Three));
        doc.objects[0].parent = Some(10);
        doc.objects[0].translation = position;
        let mut h = Host::new(doc).unwrap();
        if perspective {
            let mut view = h.view3d();
            view.sample_default = false;
            view.mode = Mode::Perspective;
            view.target = Vec3::ZERO;
            view.yaw = 0.;
            view.pitch = 0.;
            h.set_view3d(view).unwrap();
        }
        let select = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"select","payload":10});
        h.dispatch(&select.to_string()).unwrap();
        h.frame(None).unwrap();
        let vp = h.camera();
        let projected = interaction::project(vp, Vec3::from(position));
        let reason = h.ui().group_handle_reason.clone();
        let origin = h.editor.document.local_target(10).unwrap();
        let (_, a, d) = interaction::axes(&origin, vp)
            .into_iter()
            .find(|(i, _, _)| *i == 0)
            .unwrap();
        let p = a + d.normalize() * 40.;
        h.pointer_down([p.x, p.y + 400.]);
        println!(
            "case={name} projected={projected:?} handle_reason={reason:?} gesture_active={}",
            h.gesture_active()
        );
        if name.contains("far") || name.contains("near") {
            assert!(
                reason.is_some(),
                "clipped group should expose an explanation"
            );
        }
        results.push((name, projected.is_some(), h.gesture_active()));
    }
    assert!(
        results[0].1 && results[0].2 && results[3].1 && results[3].2,
        "visible positive controls"
    );
    assert!(
        results
            .iter()
            .filter(|r| r.0.contains("far") || r.0.contains("near"))
            .all(|r| !r.1 && !r.2),
        "entirely clipped descendants must not expose a group handle"
    );
}

#[test]
fn boxes_crossing_depth_planes_are_not_discarded() {
    use autographics_auto_driver::authoring::interaction::clipped_box_points;
    use glam::{Mat4, Vec3};
    for (lo, hi) in [
        (Vec3::new(-1., -1., -0.5), Vec3::new(1., 1., 0.5)),
        (Vec3::new(-1., -1., 0.5), Vec3::new(1., 1., 1.5)),
        (Vec3::new(-1., -1., -1.), Vec3::new(1., 1., 2.)),
    ] {
        assert!(
            !clipped_box_points(Mat4::IDENTITY, Mat4::IDENTITY, lo, hi).is_empty(),
            "box crossing the depth slab"
        );
    }
    for (lo, hi) in [
        (Vec3::new(-1., -1., -2.), Vec3::new(1., 1., -1.)),
        (Vec3::new(-1., -1., 2.), Vec3::new(1., 1., 3.)),
    ] {
        assert!(
            clipped_box_points(Mat4::IDENTITY, Mat4::IDENTITY, lo, hi).is_empty(),
            "box outside depth slab"
        );
    }
}

#[test]
fn group_with_a_child_crossing_either_perspective_clip_plane_keeps_handle() {
    use autographics_auto_driver::authoring::{
        document::{Dimension, Document, Group},
        host::Host,
        interaction,
        view3d::Mode,
    };
    use glam::Vec3;
    for (label, z, scale) in [("near", 18.95, 5.0), ("far", -16365.0, 500.0)] {
        let mut doc = Document::sample();
        doc.objects.retain(|o| o.id == 3);
        doc.groups.push(Group::new(10, Dimension::Three));
        doc.objects[0].parent = Some(10);
        doc.objects[0].translation = [0., 0., z];
        doc.objects[0].scale = [scale, scale, scale];
        let mut h = Host::new(doc).unwrap();
        let mut v = h.view3d();
        v.sample_default = false;
        v.mode = Mode::Perspective;
        v.target = Vec3::ZERO;
        v.yaw = 0.;
        v.pitch = 0.;
        h.set_view3d(v).unwrap();
        let select = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"select","payload":10});
        h.dispatch(&select.to_string()).unwrap();
        h.frame(None).unwrap();
        assert!(
            h.ui().group_handle_reason.is_none(),
            "{label} crossing must stay available: {:?}",
            h.ui().group_handle_reason
        );
        let origin = h.editor.document.local_target(10).unwrap();
        let (_, a, d) = interaction::axes(&origin, h.camera())
            .into_iter()
            .find(|(i, _, _)| *i == 0)
            .unwrap();
        let p = a + d.normalize() * 40.;
        h.pointer_down([p.x, p.y + 400.]);
        assert!(
            h.gesture_active(),
            "{label} crossing must permit group gesture"
        );
        h.cancel_gesture();
    }
}
