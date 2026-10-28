use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::authoring::{
    document::Document,
    host::Host,
    interaction::{self, stable_inverse},
    view3d::Mode,
};
use glam::Vec3;
#[test]
fn actual_visible_3d_geometry_and_selection_at_two_distances() {
    for distance in [1024., 4096.] {
        let mut doc = Document::sample();
        doc.objects.retain(|o| o.id == 3);
        doc.objects[0].translation = [0., 0., 0.];
        doc.objects[0].scale = [500., 500., 500.];
        let mut h = Host::new(doc).unwrap();
        let mut v = h.view3d();
        v.sample_default = false;
        v.mode = Mode::Perspective;
        v.target = Vec3::ZERO;
        v.yaw = 1.2;
        v.pitch = 0.;
        v.distance = distance;
        assert!(v.valid());
        h.set_view3d(v).unwrap();
        h.frame(None).unwrap();
        let vp = h.camera();
        let c = interaction::project(vp, Vec3::ZERO).unwrap();
        let e = interaction::project(vp, Vec3::new(0., 250., 0.)).unwrap();
        let points: Vec<_> = (-2..=2)
            .flat_map(|x| (-2..=2).map(move |y| [c.x + x as f32, c.y + y as f32]))
            .collect();
        let with = h.rt.probe_pixels(h.three, &points).unwrap();
        let saved = match h.rt.content_mut(h.three).unwrap() {
            SurfaceContent::Scene3d(scene) => std::mem::take(&mut scene.instances),
            _ => panic!("3d"),
        };
        h.rt.graph.mark_content_dirty(h.three);
        h.rt.frame(None, &Default::default()).unwrap();
        let without = h.rt.probe_pixels(h.three, &points).unwrap();
        match h.rt.content_mut(h.three).unwrap() {
            SurfaceContent::Scene3d(scene) => scene.instances = saved,
            _ => panic!("3d"),
        };
        h.rt.graph.mark_content_dirty(h.three);
        h.frame(None).unwrap();
        let changed = with.iter().zip(&without).filter(|(a, b)| a != b).count();
        h.pointer_down([c.x, c.y + 400.]);
        h.pointer_up([c.x, c.y + 400.]);
        println!("distance={distance} half_height_px={} matrix_inverse_admitted={} visible_changed_pixels={changed} selection={:?}",(e-c).length(),stable_inverse(vp).is_some(),h.editor.selection);
        assert!(changed > 0, "real model must change rendered pixels");
        assert_eq!(
            h.editor.selection,
            Some(3),
            "a visibly rendered large object at center must be selectable"
        );
    }
}

#[test]
fn visible_large_object_retains_usable_world_axes_at_far_view() {
    use autographics_auto_driver::authoring::{
        document::Document, host::Host, interaction, view3d::Mode,
    };
    use glam::Vec3;
    let mut doc = Document::sample();
    doc.objects.retain(|o| o.id == 3);
    doc.objects[0].translation = [0., 0., 0.];
    doc.objects[0].scale = [500., 500., 500.];
    let mut h = Host::new(doc).unwrap();
    let mut v = h.view3d();
    v.sample_default = false;
    v.mode = Mode::Perspective;
    v.target = Vec3::ZERO;
    v.yaw = 1.2;
    v.pitch = 0.;
    v.distance = 4096.;
    h.set_view3d(v).unwrap();
    h.editor.select(Some(3)).unwrap();
    h.frame(None).unwrap();
    let object = h.editor.preview.objects.iter().find(|o| o.id == 3).unwrap();
    let vp = h.camera();
    let axes = interaction::axes(object, vp);
    assert!(
        axes.iter().any(|(i, _, _)| *i == 1),
        "a visibly large selected object needs the nondegenerate Y handle"
    );
    let (_, a, d) = axes.iter().find(|(i, _, _)| *i == 1).unwrap();
    let start = *a + d.normalize() * 40.;
    let drag = interaction::AxisDrag::new(vp, Vec3::ZERO, 1, start).unwrap();
    let end = start + d.normalize() * 20.;
    let delta = drag.delta(end).unwrap();
    assert!(delta.abs() > 1.);
    h.pointer_down([start.x, start.y + 400.]);
    assert!(h.gesture_active());
    h.pointer_up([end.x, end.y + 400.]);
    h.settle().unwrap();
    let actual = h
        .editor
        .document
        .objects
        .iter()
        .find(|o| o.id == 3)
        .unwrap()
        .translation[1];
    assert!(
        (actual - delta).abs() < 0.1,
        "actual={actual}, independent ray delta={delta}"
    );
}

#[test]
fn large_world_target_is_pickable_in_both_projections() {
    use autographics_auto_driver::authoring::{
        document::Document, host::Host, interaction, view3d::Mode,
    };
    use glam::Vec3;
    for mode in [Mode::Orthographic, Mode::Perspective] {
        let target = Vec3::new(60000., -60000., 60000.);
        let mut doc = Document::sample();
        doc.objects.retain(|o| o.id == 3);
        doc.objects[0].translation = target.to_array();
        doc.objects[0].scale = [500., 500., 500.];
        let mut h = Host::new(doc).unwrap();
        let mut v = h.view3d();
        v.sample_default = false;
        v.mode = mode;
        v.target = target;
        v.yaw = 1.2;
        v.pitch = 0.2;
        v.distance = 4096.;
        v.ortho_height = 8192.;
        h.set_view3d(v).unwrap();
        h.frame(None).unwrap();
        let vp = h.camera();
        assert!(interaction::stable_inverse(vp).is_some());
        let p = interaction::project(vp, target).unwrap();
        h.pointer_down([p.x, p.y + 400.]);
        h.pointer_up([p.x, p.y + 400.]);
        assert_eq!(h.editor.selection, Some(3), "{mode:?}");
        h.frame(None).unwrap();
        let object = h.editor.preview.objects.iter().find(|o| o.id == 3).unwrap();
        let (_, a, d) = interaction::axes(object, vp)
            .into_iter()
            .find(|(i, _, _)| *i == 1)
            .unwrap();
        let start = a + d.normalize() * 40.;
        let drag = interaction::AxisDrag::new(vp, target, 1, start).unwrap();
        let end = start + d.normalize() * 10.;
        assert!(drag.delta(end).unwrap().abs() > 1.);
        assert!(interaction::stable_inverse(glam::Mat4::ZERO).is_none());
        assert!(
            interaction::stable_inverse(glam::Mat4::from_cols_array(&[f32::NAN; 16])).is_none()
        );
    }
}
