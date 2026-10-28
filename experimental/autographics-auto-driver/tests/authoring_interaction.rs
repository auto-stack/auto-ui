use agp::assets::{MaterialDef, MeshData, ModelData};
use autographics_auto_driver::authoring::{document::*, host::Host, interaction::*};
use glam::{Mat4, Vec3};
fn proxy() -> ModelData {
    let mesh = |name: &str, positions: Vec<[f32; 3]>| MeshData {
        name: name.into(),
        positions,
        normals: vec![],
        indices: vec![],
        material: MaterialDef {
            name: "test".into(),
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
        },
    };
    ModelData {
        source_sha256: "two-parts".into(),
        meshes: vec![
            mesh("first", vec![[-0.2, -0.2, -0.05], [0.0, 0.2, 0.05]]),
            mesh("second", vec![[0.0, -0.2, -0.05], [0.2, 0.2, 0.05]]),
        ],
    }
}
#[test]
fn exact_2d_shapes_order_and_invalid_inputs() {
    let mut bottom = Object::new(100, Kind::Quad);
    bottom.scale = [100.0, 100.0, 1.0];
    let mut circle = Object::new(2, Kind::Circle);
    circle.scale = [80.0, 40.0, 1.0];
    assert_eq!(
        pick2(&[bottom.clone(), circle.clone()], [400.0, 190.0]),
        Some(2)
    );
    assert_eq!(
        pick2(&[bottom.clone(), circle.clone()], [435.0, 207.0]),
        Some(100),
        "ellipse corner must miss"
    );
    circle.visible = false;
    assert_eq!(
        pick2(&[bottom.clone(), circle.clone()], [400.0, 190.0]),
        Some(100)
    );
    circle.visible = true;
    circle.rgba[3] = 0.0;
    assert_eq!(pick2(&[bottom.clone(), circle], [400.0, 190.0]), Some(100));
    for p in [
        [f32::NAN, 190.0],
        [-1.0, 190.0],
        [800.0, 190.0],
        [400.0, 380.0],
    ] {
        assert_eq!(pick2(&[bottom.clone()], p), None);
    }
}
#[test]
fn full_mesh_near_far_depth_ties_and_obb() {
    let models = vec![proxy(); 3];
    let mut a = Object::new(7, Kind::Cylinder);
    a.translation = [0.0, 0.0, 0.4];
    assert_eq!(
        bounds(&models[0]),
        Some((Vec3::new(-0.2, -0.2, -0.05), Vec3::new(0.2, 0.2, 0.05)))
    );
    assert_eq!(
        pick3(&[a.clone()], &models, Mat4::IDENTITY, [460.0, 200.0]),
        Some(7),
        "right-hand mesh, absent in first part"
    );
    let mut b = a.clone();
    b.id = 3;
    b.translation[2] = 0.7;
    assert_eq!(
        pick3(
            &[b.clone(), a.clone()],
            &models,
            Mat4::IDENTITY,
            [400.0, 200.0]
        ),
        Some(7)
    );
    b.translation[2] = 0.4;
    assert_eq!(
        pick3(
            &[a.clone(), b.clone()],
            &models,
            Mat4::IDENTITY,
            [400.0, 200.0]
        ),
        Some(3)
    );
    for z in [-0.5, 1.5] {
        a.translation[2] = z;
        assert_eq!(
            pick3(&[a.clone()], &models, Mat4::IDENTITY, [400.0, 200.0]),
            None,
            "near/far {z}"
        );
    }
    a.translation[2] = 0.4;
    a.scale = [2.0, 0.5, 3.0];
    a.yaw = 0.6;
    a.pitch = 0.2;
    // Independent projected point from a known inside local coordinate must hit transformed proxy.
    let local = Vec3::new(0.12, 0.04, 0.0);
    let world = transform(&a).transform_point3(local);
    let p = project(Mat4::IDENTITY, world).unwrap();
    assert_eq!(
        pick3(&[a.clone()], &models, Mat4::IDENTITY, p.to_array()),
        Some(7)
    );
    assert_eq!(
        pick3(&[a.clone()], &models, Mat4::ZERO, [400.0, 200.0]),
        None
    );
    a.scale[0] = 0.0;
    assert_eq!(pick3(&[a], &models, Mat4::IDENTITY, [400.0, 200.0]), None);
}
#[test]
fn host_routing_dpi_selection_empty_clip_and_no_content_history() {
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let doc = h.editor.document.clone();
    let hash = doc.hash().unwrap();
    let stamp = h.editor.stamp;
    h.resize(2400, 1600);
    h.pointer_down([690.0, 380.0]);
    h.pointer_up([690.0, 380.0]);
    assert_eq!(h.editor.selection, Some(1));
    assert_eq!(h.editor.document.hash().unwrap(), hash);
    assert_eq!(h.editor.stamp, stamp);
    assert_eq!(h.editor.history(), (0, 0, 0));
    assert!(!h.editor.dirty());
    h.pointer_down([20.0, 20.0]);
    h.pointer_up([20.0, 20.0]);
    assert_eq!(h.editor.selection, None);
    h.pointer_down([900.0, 780.0]);
    assert_eq!(h.editor.selection, None, "viewport gap");
    h.resize(0, 0);
    h.pointer_down([1.0, 1.0]);
    assert!(!h.gesture_active());
    h.resize(1200, 800);
    h.pointer_down([f32::NAN, 0.0]);
    assert!(!h.gesture_active());
    assert_eq!(h.editor.document, doc);
}

fn point(h: &Host) -> [f32; 2] {
    let o = &h.editor.document.objects[0];
    [400.0 + o.translation[0], 190.0 - o.translation[1]]
}
fn command(h: &mut Host, action: &str) {
    let m = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":action,"payload":null});
    h.dispatch(&m.to_string()).unwrap();
    h.settle().unwrap();
}
#[test]
fn merged_samples_noop_duplicate_release_auto_failure_and_recovery() {
    use autographics_auto_driver::authoring::preview::SOURCE;
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let accepted = h.editor.document.clone();
    let start = point(&h);
    h.pointer_down(start);
    for i in 1..=1000 {
        h.pointer_move([start[0] + i as f32 / 100.0, start[1]]);
    }
    let state = h.interaction_snapshot();
    assert_eq!(state["latest"], 1000);
    assert!(state["inflight"].is_null());
    h.frame(None).unwrap();
    for _ in 0..20 {
        h.frame(None).unwrap();
    }
    assert_eq!(h.editor.document, accepted);
    assert_eq!(h.editor.history(), (0, 0, 0));
    h.pointer_up([start[0] + 20.0, start[1] - 10.0]);
    h.pointer_up([start[0] + 99.0, start[1]]);
    h.settle().unwrap();
    assert_eq!(h.editor.stamp.revision, 1);
    assert_eq!(h.editor.history().0, 1);
    assert_eq!(h.editor.document.objects[0].translation, [-35.0, 10.0, 0.0]);
    command(&mut h, "undo");
    assert_eq!(h.editor.document, accepted);
    let stamp = h.editor.stamp;
    h.pointer_down(start);
    h.pointer_move([start[0] + 20.0, start[1]]);
    for _ in 0..20 {
        h.frame(None).unwrap();
    }
    h.pointer_up(start);
    h.settle().unwrap();
    assert_eq!(h.editor.stamp, stamp);
    for source in [
        SOURCE.replace("fields.push(value)", "fields.push(0.0)"),
        SOURCE.replace("return identities[i]", "return identities[i] + 1"),
    ] {
        h.pointer_down(start);
        h.pointer_move([start[0] + 10.0, start[1]]);
        for _ in 0..20 {
            h.frame(None).unwrap();
        }
        let original = h.editor.document.clone();
        let history = h.editor.history();
        h.gesture_source_for_probe(Some(source));
        h.pointer_up([start[0] + 30.0, start[1]]);
        h.settle().unwrap();
        assert_eq!(h.editor.document, original);
        assert_eq!(h.editor.history(), history);
        assert_eq!(h.editor.stamp, stamp);
        assert!(!h.ui().error.is_empty());
        h.gesture_source_for_probe(None);
    }
    h.pointer_down(start);
    h.pointer_up([start[0] + 10.0, start[1]]);
    h.settle().unwrap();
    assert_eq!(h.editor.stamp.revision, stamp.revision + 1);
    assert!(!h.gesture_compile_us.is_empty());
    assert!(!h.gesture_pump_us.is_empty());
    println!(
        "actual compile_us={:?}, pump_us={:?}",
        h.gesture_compile_us, h.gesture_pump_us
    );
}
#[test]
fn twenty_gestures_actual_vm_release_and_mesh_reuse() {
    use agp::host::surface_runtime::SurfaceContent;
    use std::sync::Arc;
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let (pin, writes, bytes) = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => {
            let i = &c.instances[0];
            (
                c.store.pin(&i.mesh.0, &i.mesh.1).unwrap(),
                c.store.uploads.snapshot().buffer_rewrites,
                c.store.owned_buffer_bytes(),
            )
        }
        _ => panic!(),
    };
    let weak = Arc::downgrade(&pin);
    let targets = h.rt.stats.target_creations;
    for round in 0..20 {
        let start = point(&h);
        h.pointer_down(start);
        h.pointer_move([start[0] + 5.0, start[1]]);
        for _ in 0..20 {
            h.frame(None).unwrap();
        }
        let owners = h.gesture_owner_probes();
        assert!((2..=3).contains(&owners.len()));
        assert!(owners.iter().all(|p| p()));
        let accepted = h.editor.document.clone();
        let history = h.editor.history();
        let stamp = h.editor.stamp;
        match round % 3 {
            0 => {
                h.pointer_up([start[0] + 6.0, start[1]]);
                h.settle().unwrap();
                assert!(owners.iter().all(|p| !p()));
            }
            1 => {
                h.cancel_gesture();
                assert!(owners[0]());
                assert!(owners.iter().skip(1).all(|p| !p()));
                assert_eq!(h.editor.document, accepted);
            }
            _ => {
                h.fail_resource = true;
                h.pointer_up([start[0] + 6.0, start[1]]);
                h.settle().unwrap();
                h.fail_resource = false;
                assert_eq!(h.editor.document, accepted);
                assert_eq!(h.editor.history(), history);
                assert_eq!(h.editor.stamp, stamp);
                assert!(owners[0]());
                assert!(owners.iter().skip(1).all(|p| !p()));
            }
        }
        if let SurfaceContent::Scene3d(c) = h.rt.content(h.three).unwrap() {
            assert_eq!(c.store.uploads.snapshot().buffer_rewrites, writes);
            assert_eq!(c.store.owned_buffer_bytes(), bytes);
            assert_eq!(c.store.uploaded_count(), 7);
            let first = &c.instances[0];
            let current = c.store.pin(&first.mesh.0, &first.mesh.1).unwrap();
            assert!(
                Arc::ptr_eq(&pin, &current),
                "warm model GPU identity must be reused"
            );
        }
        assert_eq!(h.rt.stats.target_creations, targets);
    }
    let accepted = h.gesture_owner_probes();
    drop(h);
    assert!(accepted.iter().all(|p| !p()));
    assert!(
        weak.upgrade().is_some(),
        "external mesh pin legitimate control"
    );
    drop(pin);
    assert!(
        weak.upgrade().is_none(),
        "last mesh owner must release buffers"
    );
}

#[test]
fn capture_busy_cancel_stale_and_large_overlay_boundaries() {
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let start = point(&h);
    let original = h.editor.document.clone();
    h.pointer_down(start);
    h.pointer_move([850.0, 600.0]);
    h.frame(None).unwrap();
    assert!(
        h.gesture_active(),
        "capture must stay on origin surface across panel and other viewport"
    );
    assert_eq!(h.editor.selection, Some(1));
    for action in [
        "select", "apply", "create", "delete", "undo", "redo", "save", "open", "new",
    ] {
        let m = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":action,"payload":2});
        assert!(h.dispatch(&m.to_string()).is_err(), "busy {action}");
    }
    h.blur();
    h.pointer_up([850.0, 600.0]);
    h.frame(None).unwrap();
    assert_eq!(h.editor.document, original);
    assert!(!h.gesture_active());
    for reason in 0..4 {
        h.pointer_down(start);
        h.pointer_move([start[0] + 20.0, start[1]]);
        h.frame(None).unwrap();
        match reason {
            0 => h.resize(1201, 800),
            1 => h.pointer_move([-1.0, 190.0]),
            2 => h.pointer_move([f32::NAN, 190.0]),
            _ => h.cancel_gesture(),
        }
        h.pointer_up([start[0] + 25.0, start[1]]);
        h.frame(None).unwrap();
        assert_eq!(h.editor.document, original);
        assert_eq!(h.editor.stamp.revision, 0);
        h.resize(1200, 800);
    }
    h.pointer_down(start);
    h.request_close();
    assert!(h.closed, "clean accepted state closes after cancellation");
    assert_eq!(h.editor.document, original);
    // A legal huge model must not overrun the existing renderer's 4096 overlay quads.
    let mut doc = Document::empty("large".into());
    let mut o = Object::new(1, Kind::Player);
    o.scale = [1000.0; 3];
    doc.objects.push(o);
    let mut large = Host::new(doc).unwrap();
    large.editor.select(Some(1)).unwrap();
    large.dispatch(&serde_json::json!({"epoch":large.editor.stamp.epoch,"revision":0,"action":"select","payload":1}).to_string()).unwrap();
    large.frame(None).unwrap();
}

#[test]
fn storage_busy_dirty_close_and_unpresented_camera() {
    use agp::host::surface_runtime::SurfaceContent;
    use autographics_auto_driver::authoring::session::Edit;
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let start = point(&h);
    let path = std::env::temp_dir().join(format!("p836-busy-{}.agscene", std::process::id()));
    std::fs::write(&path, b"original unrelated bytes").unwrap();
    h.pointer_down(start);
    h.pointer_move([start[0] + 10.0, start[1]]);
    let message = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"saveas","payload":path});
    assert!(h.dispatch(&message.to_string()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"original unrelated bytes");
    std::fs::remove_file(&path).unwrap();
    h.cancel_gesture();
    if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(h.three) {
        c.view_proj = Mat4::IDENTITY;
    }
    h.pointer_down(start);
    assert!(!h.gesture_active(), "camera changed before presentation");
    h.frame(None).unwrap();
    h.editor.select(Some(1)).unwrap();
    let mut o = h.editor.document.objects[0].clone();
    o.name = "dirty accepted".into();
    h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
    h.settle().unwrap();
    let accepted = h.editor.document.clone();
    let history = h.editor.history();
    h.pointer_down(start);
    h.pointer_move([start[0] + 10.0, start[1]]);
    h.request_close();
    assert!(!h.closed);
    assert!(h.ui().dialog);
    assert!(!h.gesture_active());
    assert_eq!(h.editor.document, accepted);
    assert_eq!(h.editor.history(), history);
}
