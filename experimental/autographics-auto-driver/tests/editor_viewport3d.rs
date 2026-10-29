use agp::render3d::Camera3d;
use autographics_auto_driver::authoring::view3d::{Mode, View3d};
use glam::{Vec2, Vec3};

#[test]
fn reset_is_exact_sample_and_navigation_is_finite() {
    let v = View3d::default();
    assert_eq!(
        v.camera().unwrap().view_proj(),
        Camera3d::sample_camera(2.0).view_proj()
    );
    let rotated = v.orbit(Vec2::new(250.0, 50.0)).unwrap();
    assert!(rotated.valid() && rotated.camera().unwrap().view_proj().is_finite());
    let panned = rotated.pan(Vec2::new(40.0, -20.0)).unwrap();
    assert_ne!(panned.target, rotated.target);
    assert!(panned.wheel(f32::NAN).is_err());
    assert!(panned.orbit(Vec2::splat(f32::INFINITY)).is_err());
    let edge = View3d {
        target: Vec3::splat(65536.0),
        ..panned
    };
    assert!(
        edge.pan(Vec2::splat(1e9))
            .unwrap()
            .target
            .abs()
            .max_element()
            <= 65536.0
    );
}
#[test]
fn projection_switch_preserves_target_height_until_saturation() {
    let v = View3d::default();
    let (p, limited) = v.toggle_projection().unwrap();
    assert_eq!(p.mode, Mode::Perspective);
    assert!(!limited);
    assert!((p.visible_height() - v.ortho_height).abs() < 1e-4);
    let (o, limited) = p.toggle_projection().unwrap();
    assert!(!limited);
    assert!((o.ortho_height - v.ortho_height).abs() < 1e-4);
    assert_eq!(o.target, v.target);
    let big = View3d {
        ortho_height: 8192.0,
        sample_default: false,
        ..v
    };
    let (p, limited) = big.toggle_projection().unwrap();
    assert!(limited);
    assert_eq!(p.distance, 4096.0);
}
#[test]
fn grid_is_world_aligned_bounded_and_snap_is_away_from_zero() {
    let mut v = View3d {
        grid: true,
        snap: true,
        step: 10.0,
        ..View3d::default()
    };
    for height in [0.125, 1.0, 24.0, 8192.0] {
        v.ortho_height = height;
        v.sample_default = false;
        let (spacing, lines) = v.grid_vertices().unwrap();
        assert!(spacing > 0.0 && lines.len() <= 256 && lines.len() % 2 == 0);
        assert!(lines.iter().all(|p| p.position[1] == 0.0
            && p.position
                .iter()
                .chain(p.color.iter())
                .all(|x| x.is_finite())));
    }
    assert_eq!(v.snapped_axis(5.0), 10.0);
    assert_eq!(v.snapped_axis(-5.0), -10.0);
    v.grid = false;
    assert!(v.grid_vertices().unwrap().1.is_empty());
}

#[test]
fn focus_frames_world_bounds_under_both_projections_and_rejects_impossible() {
    let base = View3d::default().orbit(Vec2::new(100.0, 20.0)).unwrap();
    let lo = Vec3::new(-2.0, -1.0, -3.0);
    let hi = Vec3::new(4.0, 3.0, 2.0);
    for view in [base, base.toggle_projection().unwrap().0] {
        let focused = view.focus(lo, hi).unwrap();
        let vp = focused.camera().unwrap().view_proj();
        for i in 0..8 {
            let p = Vec3::new(
                if i & 1 == 0 { lo.x } else { hi.x },
                if i & 2 == 0 { lo.y } else { hi.y },
                if i & 4 == 0 { lo.z } else { hi.z },
            );
            let c = vp * p.extend(1.0);
            let n = c.truncate() / c.w;
            assert!(
                c.w > 0.0 && n.x.abs() <= 0.801 && n.y.abs() <= 0.801 && (0.0..=1.0).contains(&n.z),
                "corner {i}: {n:?}"
            );
        }
    }
    assert!(base
        .focus(Vec3::splat(-10000.0), Vec3::splat(10000.0))
        .is_err());
}

#[test]
fn host_camera_grid_navigation_and_play_restore_use_one_snapshot() {
    use autographics_auto_driver::authoring::{document::Document, host::Host};
    let mut h = Host::new(Document::sample()).unwrap();
    let document = h.editor.document.clone();
    let history = h.editor.history();
    let mut v = h.view3d().orbit(Vec2::new(100.0, 40.0)).unwrap();
    v.grid = true;
    h.set_view3d(v).unwrap();
    h.frame(None).unwrap();
    assert_eq!(h.presented_view3d(), h.view3d());
    let vp = h.view3d().camera().unwrap().view_proj();
    assert_eq!(h.camera(), vp);
    assert_eq!(h.input.mapper.parents.get(&h.three).unwrap().0, vp);
    let before = h.view3d();
    h.navigation3d_down([400.0, 600.0], true);
    h.pointer_move([440.0, 610.0]);
    assert_ne!(h.view3d(), before);
    assert!(h.set_view3d(before).is_err());
    h.cancel_navigation();
    let restored = h.view3d();
    assert_eq!(restored.target, before.target);
    assert_eq!(restored.yaw, before.yaw);
    assert_eq!(restored.pitch, before.pitch);
    h.frame(None).unwrap();
    assert_eq!(h.editor.document, document);
    assert_eq!(h.editor.history(), history);
    h.start_play(autographics_auto_driver::authoring::play::SOURCE)
        .unwrap();
    assert_eq!(h.camera(), Camera3d::sample_camera(2.0).view_proj());
    h.stop_play();
    assert_eq!(h.camera(), h.view3d().camera().unwrap().view_proj());
    assert_eq!(h.editor.document, document);
    assert_eq!(h.editor.history(), history);
}

#[test]
fn actual_host_perspective_axis_drag_commits_one_auto_edit() {
    use autographics_auto_driver::authoring::{
        document::Document,
        host::Host,
        interaction::{self, AxisDrag},
    };
    let mut h = Host::new(Document::sample()).unwrap();
    let mut view = h
        .view3d()
        .orbit(Vec2::new(110.0, 30.0))
        .unwrap()
        .toggle_projection()
        .unwrap()
        .0;
    view.snap = false;
    h.set_view3d(view).unwrap();
    h.editor.select(Some(3)).unwrap();
    h.frame(None).unwrap();
    let before = h.editor.document.clone();
    let original = before.objects.iter().find(|o| o.id == 3).unwrap().clone();
    let vp = h.camera();
    let (axis, a, d) = interaction::axes(&original, vp)
        .into_iter()
        .find(|(axis, _, _)| *axis == 0)
        .unwrap();
    let start = a + d.normalize() * 32.0;
    assert_eq!(
        interaction::handle(&original, vp, start.to_array())
            .unwrap()
            .0,
        axis
    );
    let origin = Vec3::from(original.translation);
    let drag = AxisDrag::new(vp, origin, axis, start).unwrap();
    let mut end = start;
    for dx in 1..150 {
        let candidate = start + Vec2::new(dx as f32, 0.0);
        if drag.delta(candidate).is_some_and(|d| d.abs() > 1.0) {
            end = candidate;
            break;
        }
    }
    let expected = drag.delta(end).unwrap();
    assert!(expected.abs() > 1.0);
    let window_start = [start.x, start.y + 400.0];
    let window_end = [end.x, end.y + 400.0];
    h.pointer_down(window_start);
    assert!(h.gesture_active(), "X-axis capture must start");
    h.pointer_move(window_end);
    h.pointer_up(window_end);
    h.settle().unwrap();
    let after = h
        .editor
        .document
        .objects
        .iter()
        .find(|o| o.id == 3)
        .unwrap();
    assert!(
        (after.translation[0] - original.translation[0] - expected).abs() < 0.002,
        "actual {:?}, expected {expected}",
        after.translation
    );
    assert_eq!(after.translation[1], original.translation[1]);
    assert_eq!(after.translation[2], original.translation[2]);
    assert_eq!(h.editor.history().0, 1);
    assert!(!h.gesture_compile_us.is_empty());
    let m = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"undo","payload":null});
    h.dispatch(&m.to_string()).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document, before);
}

#[test]
fn axis_nearly_parallel_to_camera_is_rejected_without_jump() {
    use autographics_auto_driver::authoring::interaction::AxisDrag;
    for z in [0.0, 0.0001] {
        let camera = Camera3d {
            eye: Vec3::new(10.0, 0.0, z),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: agp::render3d::Projection::Perspective {
                fov_y_deg: 60.0,
                aspect: 2.0,
                near: 0.05,
                far: 16384.0,
            },
        };
        assert!(
            AxisDrag::new(camera.view_proj(), Vec3::ZERO, 0, Vec2::new(400.0, 200.0)).is_none()
        );
    }
}

#[test]
fn failed_3d_navigation_preserves_admitted_view_and_draft() {
    use autographics_auto_driver::authoring::{document::Document, host::Host};
    let mut h = Host::new(Document::sample()).unwrap();
    h.editor.select(Some(3)).unwrap();
    h.frame(None).unwrap();
    let mut v = h.view3d().orbit(Vec2::new(45.0, 15.0)).unwrap();
    v.grid = true;
    h.set_view3d(v).unwrap();
    h.frame(None).unwrap();
    h.ui_mut().draft[0] = "unsubmitted".into();
    let initial = h.view3d();
    let original = h.editor.document.clone();
    let history = h.editor.history();
    h.navigation3d_down([400.0, 600.0], true);
    h.fail_resource = true;
    h.pointer_move([420.0, 610.0]);
    h.fail_resource = false;
    assert_eq!(h.view3d().yaw, initial.yaw);
    assert_eq!(h.view3d().pitch, initial.pitch);
    assert_eq!(h.camera(), initial.camera().unwrap().view_proj());
    assert_eq!(h.ui().draft[0], "unsubmitted");
    assert!(h.ui().error.contains("resource preparation rejected"));
    h.pointer_move([460.0, 630.0]);
    h.navigation3d_up([460.0, 630.0]);
    assert_eq!(h.view3d().yaw, initial.yaw);
    assert_eq!(h.editor.document, original);
    assert_eq!(h.editor.history(), history);
}

#[test]
fn parented_3d_axes_snap_in_world_and_preserve_other_fields() {
    use autographics_auto_driver::authoring::{
        document::{Dimension, Document, Group},
        host::Host,
        interaction::{self, AxisDrag},
    };
    let mut doc = Document::sample();
    let mut group = Group::new(10, Dimension::Three);
    group.translation = [13.0, -7.0, 5.0];
    doc.groups.push(group);
    doc.objects[2].parent = Some(10);
    let mut h = Host::new(doc.clone()).unwrap();
    let mut view = h
        .view3d()
        .orbit(Vec2::new(105.0, 10.0))
        .unwrap()
        .toggle_projection()
        .unwrap()
        .0;
    view.target = Vec3::new(13.0, -7.0, 5.0);
    view.snap = true;
    view.step = 1.0;
    h.set_view3d(view).unwrap();
    h.frame(None).unwrap();
    for axis in 0..3 {
        h.editor.select(Some(3)).unwrap();
        h.frame(None).unwrap();
        let world = h
            .editor
            .preview
            .objects
            .iter()
            .find(|o| o.id == 3)
            .unwrap()
            .clone();
        let local = h
            .editor
            .document
            .objects
            .iter()
            .find(|o| o.id == 3)
            .unwrap()
            .clone();
        let vp = h.camera();
        let (_, a, d) = interaction::axes(&world, vp)
            .into_iter()
            .find(|(n, _, _)| *n == axis)
            .expect("axis visible");
        let start = a + d.normalize() * 40.0;
        assert_eq!(
            interaction::handle(&world, vp, start.to_array()).unwrap().0,
            axis
        );
        let drag = AxisDrag::new(vp, Vec3::from(world.translation), axis, start).unwrap();
        let mut endpoint = None;
        for n in 12..100 {
            let p = start + d.normalize() * n as f32;
            if let Some(v) = drag.delta(p) {
                if v.abs() > 1.0 && p.x >= 0.0 && p.x < 800.0 && p.y >= 0.0 && p.y < 400.0 {
                    endpoint = Some((p, v));
                    break;
                }
            }
        }
        let (end, delta) = endpoint.expect("finite axis movement");
        let expected = (world.translation[axis] + delta).round() - world.translation[axis];
        h.pointer_down([start.x, start.y + 400.0]);
        assert!(h.gesture_active());
        h.pointer_up([end.x, end.y + 400.0]);
        h.settle().unwrap();
        let after = h
            .editor
            .document
            .objects
            .iter()
            .find(|o| o.id == 3)
            .unwrap();
        for j in 0..3 {
            if j == axis {
                assert!(
                    (after.translation[j] - local.translation[j] - expected).abs() < 0.002,
                    "axis {axis}"
                );
            } else {
                assert_eq!(after.translation[j], local.translation[j]);
            }
        }
        assert_eq!(after.parent, Some(10));
        assert_eq!(h.editor.document.groups, doc.groups);
        let m = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"undo","payload":null});
        h.dispatch(&m.to_string()).unwrap();
        h.settle().unwrap();
        assert_eq!(h.editor.document, doc);
    }
}

#[test]
fn saved_scene_omits_camera_failed_open_keeps_view_successful_open_resets() {
    use autographics_auto_driver::authoring::{
        document::Document,
        host::Host,
        storage::{Fault, Storage},
    };
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let mut view = h.view3d().orbit(Vec2::new(80.0, 20.0)).unwrap();
    view.grid = true;
    view.snap = true;
    h.set_view3d(view).unwrap();
    h.frame(None).unwrap();
    let stored = h.view3d();
    let doc = h.editor.document.clone();
    let dir = std::env::temp_dir().join(format!("plan857-view-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("saved.agscene");
    h.storage.save(&mut h.editor, &path, Fault::None).unwrap();
    assert_eq!(h.view3d(), stored);
    let (loaded, assets, code, _) = Storage::load_project(&path, &Default::default()).unwrap();
    let fresh = Host::with_project(loaded, assets, code).unwrap();
    assert_eq!(fresh.view3d(), View3d::default());
    assert_eq!(fresh.editor.document, doc);
    let cmd = |h: &mut Host, path: std::path::PathBuf| {
        let m = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"open","payload":path});
        h.dispatch(&m.to_string()).unwrap();
    };
    cmd(&mut h, dir.join("absent.agscene"));
    assert!(!h.ui().error.is_empty());
    assert_eq!(h.view3d(), stored);
    cmd(&mut h, path);
    h.settle().unwrap();
    let reset = h.view3d();
    assert!(reset.sample_default && !reset.grid && !reset.snap);
    assert_eq!(h.camera(), Camera3d::sample_camera(2.0).view_proj());
}

#[test]
fn physical_dpi_navigation_and_stale_commands_are_bounded() {
    use autographics_auto_driver::authoring::{document::Document, host::Host};
    let mut h = Host::new(Document::sample()).unwrap();
    for scale in [1.0f32, 1.5, 2.0] {
        h.resize((1200.0 * scale) as u32, (800.0 * scale) as u32);
        h.frame(None).unwrap();
        let start = h.view3d();
        h.navigation3d_down([400.0 * scale, 600.0 * scale], true);
        h.pointer_move([440.0 * scale, 610.0 * scale]);
        h.navigation3d_up([440.0 * scale, 610.0 * scale]);
        assert!(
            (h.view3d().yaw - (start.yaw - 40.0 / 800.0 * std::f32::consts::TAU)).abs() < 0.001
        );
        assert!(
            (h.view3d().pitch - (start.pitch + 10.0 / 400.0 * std::f32::consts::PI)).abs() < 0.001
        );
    }
    let view = h.view3d();
    let cmd = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"view3d_revision":view.revision-1,"action":"view3d-grid","payload":null});
    assert!(h.dispatch(&cmd.to_string()).is_err());
    assert_eq!(h.view3d(), view);
    h.resize(0, 0);
    h.navigation3d_down([1.0, 1.0], true);
    h.pointer_move([4.0, 4.0]);
    assert_eq!(h.view3d(), view);
}

#[test]
fn composed_ground_axis_is_hidden_under_sample_model() {
    use agp::host::surface_runtime::SurfaceContent;
    use autographics_auto_driver::authoring::{document::Document, host::Host, interaction};
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let vp = h.camera();
    let mut points = Vec::new();
    for i in -12..=12 {
        let x = -1.5 + i as f32 * 0.1;
        let p = interaction::project(vp, Vec3::new(x, 0.0, 0.0)).unwrap();
        for dy in -1..=1 {
            points.push([p.x, p.y + 400.0 + dy as f32]);
        }
    }
    let off = h.rt.probe_pixels(h.top, &points).unwrap();
    let mut v = h.view3d();
    v.grid = true;
    h.set_view3d(v).unwrap();
    h.frame(None).unwrap();
    let on = h.rt.probe_pixels(h.top, &points).unwrap();
    let saved = match h.rt.content_mut(h.three).unwrap() {
        SurfaceContent::Scene3d(scene) => std::mem::take(&mut scene.instances),
        _ => panic!("3d"),
    };
    h.rt.graph.mark_content_dirty(h.three);
    h.rt.frame(None, &Default::default()).unwrap();
    let removed_on = h.rt.probe_pixels(h.top, &points).unwrap();
    h.rt.set_editor_grid(h.three, None).unwrap();
    h.rt.frame(None, &Default::default()).unwrap();
    let removed_off = h.rt.probe_pixels(h.top, &points).unwrap();
    match h.rt.content_mut(h.three).unwrap() {
        SurfaceContent::Scene3d(scene) => scene.instances = saved,
        _ => panic!("3d"),
    };
    let (_, grid) = v.grid_vertices().unwrap();
    h.rt.set_editor_grid(h.three, Some(&grid)).unwrap();
    h.rt.graph.mark_content_dirty(h.three);
    h.rt.frame(None, &Default::default()).unwrap();
    assert!(
        (0..points.len()).any(|i| off[i] == on[i] && removed_on[i] != removed_off[i]),
        "grid must be hidden with foreground, then visible at the same pixel without it"
    );
    assert!(
        (0..points.len()).any(|i| off[i] != on[i]),
        "grid must also be visible outside foreground"
    );
}

#[test]
fn zero_navigation_preserves_exact_default_camera_and_history() {
    use autographics_auto_driver::authoring::{document::Document, host::Host};
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let v = h.view3d();
    let camera = h.camera();
    let history = h.editor.history();
    assert_eq!(v.orbit(Vec2::ZERO).unwrap(), v);
    assert_eq!(v.pan(Vec2::ZERO).unwrap(), v);
    assert_eq!(v.wheel(0.0).unwrap(), v);
    for orbit in [false, true] {
        h.navigation3d_down([400.0, 600.0], orbit);
        if orbit {
            h.navigation3d_up([400.0, 600.0]);
        } else {
            h.navigation_up([400.0, 600.0]);
        }
        assert!(
            h.set_view3d(v).is_ok(),
            "the owning release must finish capture"
        );
        assert_eq!(h.view3d(), v);
        assert_eq!(h.camera(), camera);
        assert_eq!(h.editor.history(), history);
    }
}

#[test]
fn repeated_navigation_and_resize_keep_auxiliary_allocations_bounded() {
    use autographics_auto_driver::authoring::{document::Document, host::Host};
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let initial = h.resource_snapshot();
    assert_eq!(initial["view3d"]["grid_gpu_bytes"], 0);
    let original_mesh = initial["mesh_uploads"].clone();
    let targets = initial["target_count"].clone();
    let mut view = h.view3d();
    view.grid = true;
    h.set_view3d(view).unwrap();
    h.frame(None).unwrap();
    for i in 0..20 {
        h.resize(
            if i % 2 == 0 { 1200 } else { 1800 },
            if i % 2 == 0 { 800 } else { 1200 },
        );
        let next = h.view3d().orbit(Vec2::new(5.0, 1.0)).unwrap();
        h.set_view3d(next).unwrap();
        h.frame(None).unwrap();
        let stats = h.resource_snapshot();
        assert_eq!(stats["mesh_uploads"], original_mesh);
        assert_eq!(stats["target_count"], targets);
        assert!(stats["view3d"]["grid_vertices"].as_u64().unwrap() <= 256);
        assert_eq!(stats["view3d"]["grid_gpu_bytes"].as_u64().unwrap(), 7232);
        assert!(stats["view3d"]["grid_cpu_capacity_bytes"].as_u64().unwrap() <= 65536);
    }
    let mut off = h.view3d();
    off.grid = false;
    h.set_view3d(off).unwrap();
    h.frame(None).unwrap();
    let owner = match h.rt.content(h.three).unwrap() {
        agp::host::surface_runtime::SurfaceContent::Scene3d(scene) => {
            scene.mesh.editor_grid_owner_probe().unwrap()
        }
        _ => panic!("3d"),
    };
    assert!(owner.upgrade().is_some());
    let disabled = h.resource_snapshot();
    h.frame(None).unwrap();
    let stable = h.resource_snapshot();
    assert_eq!(
        disabled["view3d"]["grid_uploads"],
        stable["view3d"]["grid_uploads"]
    );
    assert_eq!(stable["mesh_uploads"], original_mesh);
    println!("resource measurement: {}", stable);
    drop(h);
    assert!(
        owner.upgrade().is_none(),
        "grid owner must be released when scene closes"
    );
}

#[test]
fn perspective_multimesh_obb_pick_respects_nearest_and_clip() {
    use agp::assets::{MaterialDef, MeshData, ModelData};
    use autographics_auto_driver::authoring::{
        document::{Kind, Object},
        interaction,
    };
    let mesh = |name: &str, positions| MeshData {
        visual: None,
        name: name.into(),
        positions,
        normals: vec![],
        indices: vec![],
        material: MaterialDef {
            name: "m".into(),
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 0.0,
        },
    };
    let model = ModelData {
        source_sha256: "two-parts".into(),
        meshes: vec![
            mesh("left", vec![[-0.5, -0.5, -0.5], [0.0, 0.5, 0.5]]),
            mesh("right", vec![[0.0, -0.5, -0.5], [0.5, 0.5, 0.5]]),
        ],
    };
    let view = View3d::default()
        .orbit(Vec2::new(85.0, 30.0))
        .unwrap()
        .toggle_projection()
        .unwrap()
        .0;
    let camera = view.camera().unwrap();
    let vp = camera.view_proj();
    let backward = (camera.eye - camera.target).normalize();
    let mut far = Object::new(7, Kind::Cylinder);
    far.translation = camera.target.to_array();
    far.scale = [2.0, 0.7, 1.5];
    far.yaw = 0.6;
    far.pitch = -0.2;
    let p = interaction::project(
        vp,
        interaction::transform(&far).transform_point3(Vec3::new(0.3, 0.0, 0.0)),
    )
    .unwrap();
    assert_eq!(
        interaction::pick3_with(&[far.clone()], vp, p.to_array(), |_| Some(&model)),
        Some(7)
    );
    let mut near = far.clone();
    near.id = 3;
    near.translation = (camera.target + backward * 3.0).to_array();
    assert_eq!(
        interaction::pick3_with(&[far.clone(), near.clone()], vp, [400.0, 200.0], |_| Some(
            &model
        )),
        Some(3)
    );
    near.translation = (camera.eye + backward * 2.0).to_array();
    assert_eq!(
        interaction::pick3_with(&[near], vp, [400.0, 200.0], |_| Some(&model)),
        None
    );
}

#[test]
fn view_navigation_stays_out_of_other_panes_and_cancel_restores_camera() {
    use autographics_auto_driver::authoring::{document::Document, host::Host};
    use iced_wgpu::core::mouse::ScrollDelta;
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let original = h.view3d();
    let doc = h.editor.document.clone();
    let history = h.editor.history();
    for point in [[300.0, 200.0], [900.0, 300.0]] {
        h.navigation3d_down(point, true);
        h.pointer_move([point[0] + 20.0, point[1] + 10.0]);
        h.navigation3d_up(point);
        h.scroll(point, ScrollDelta::Lines { x: 0.0, y: 1.0 });
        assert_eq!(h.view3d(), original);
    }
    h.frame(None).unwrap();
    h.navigation3d_down([400.0, 600.0], true);
    h.pointer_move([440.0, 610.0]);
    assert_ne!(h.camera(), original.camera().unwrap().view_proj());
    h.blur();
    assert_eq!(h.camera(), original.camera().unwrap().view_proj());
    h.navigation3d_up([440.0, 610.0]);
    assert_eq!(h.camera(), original.camera().unwrap().view_proj());
    assert_eq!(h.editor.document, doc);
    assert_eq!(h.editor.history(), history);
}

#[test]
fn perspective_group_axis_drag_snaps_once_and_undo_restores_children() {
    use autographics_auto_driver::authoring::{
        document::{Dimension, Document, Group},
        host::Host,
        interaction::{self, AxisDrag},
    };
    let mut doc = Document::sample();
    let mut group = Group::new(10, Dimension::Three);
    group.translation = [13.3, -7.0, 5.0];
    doc.groups.push(group);
    doc.objects[2].parent = Some(10);
    let mut h = Host::new(doc.clone()).unwrap();
    let mut view = h
        .view3d()
        .orbit(Vec2::new(105.0, 10.0))
        .unwrap()
        .toggle_projection()
        .unwrap()
        .0;
    view.target = Vec3::new(13.3, -7.0, 5.0);
    view.snap = true;
    view.step = 1.0;
    h.set_view3d(view).unwrap();
    h.editor.select(Some(10)).unwrap();
    h.frame(None).unwrap();
    let origin = h.editor.document.local_target(10).unwrap();
    let vp = h.camera();
    let (_, a, d) = interaction::axes(&origin, vp)
        .into_iter()
        .find(|(n, _, _)| *n == 0)
        .unwrap();
    let start = a + d.normalize() * 40.0;
    assert_eq!(
        interaction::handle(&origin, vp, start.to_array())
            .unwrap()
            .0,
        0
    );
    let drag = AxisDrag::new(vp, Vec3::from(origin.translation), 0, start).unwrap();
    let (end, delta) = (12..100)
        .filter_map(|n| {
            let p = start + d.normalize() * n as f32;
            drag.delta(p)
                .filter(|v| v.abs() > 1.0 && p.x >= 0.0 && p.x < 800.0 && p.y >= 0.0 && p.y < 400.0)
                .map(|v| (p, v))
        })
        .next()
        .unwrap();
    let expected = (origin.translation[0] + delta).round();
    h.pointer_down([start.x, start.y + 400.0]);
    assert!(h.gesture_active());
    h.pointer_up([end.x, end.y + 400.0]);
    h.settle().unwrap();
    let moved = h
        .editor
        .document
        .groups
        .iter()
        .find(|g| g.id == 10)
        .unwrap();
    assert!((moved.translation[0] - expected).abs() < 0.002);
    assert_eq!(&moved.translation[1..], &origin.translation[1..]);
    assert_eq!(h.editor.document.objects, doc.objects);
    assert_eq!(h.editor.history().0, 1);
    let undo = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"undo","payload":null});
    h.dispatch(&undo.to_string()).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document, doc);
}

#[test]
fn report_default_and_grid_frame_costs_without_claiming_gpu_time() {
    use autographics_auto_driver::authoring::{document::Document, host::Host};
    use std::time::Instant;
    let mut h = Host::new(Document::sample()).unwrap();
    fn measure(h: &mut Host) -> (u128, u128) {
        for _ in 0..8 {
            h.frame(None).unwrap();
        }
        let mut samples = Vec::new();
        for _ in 0..50 {
            let start = Instant::now();
            h.frame(None).unwrap();
            samples.push(start.elapsed().as_micros());
        }
        samples.sort_unstable();
        (samples[25], samples[47])
    }
    let off = measure(&mut h);
    let off_resources = h.resource_snapshot();
    let mut v = h.view3d();
    v.grid = true;
    h.set_view3d(v).unwrap();
    let on = measure(&mut h);
    let on_resources = h.resource_snapshot();
    println!("frame cpu us p50/p95: default={off:?}, grid={on:?}; default resources={off_resources}; grid resources={on_resources}");
    assert_eq!(off_resources["view3d"]["grid_gpu_bytes"], 0);
    assert_eq!(on_resources["view3d"]["grid_gpu_bytes"], 7232);
}

#[test]
fn invalid_in_bounds_perspective_sample_retains_last_valid_auto_edit() {
    use autographics_auto_driver::authoring::{
        document::Document,
        host::Host,
        interaction::{self, AxisDrag},
    };
    let mut h = Host::new(Document::sample()).unwrap();
    let original = h.editor.document.clone();
    let object = h
        .editor
        .preview
        .objects
        .iter()
        .find(|o| o.id == 3)
        .unwrap()
        .clone();
    let mut view = h.view3d().toggle_projection().unwrap().0;
    view.sample_default = false;
    view.yaw = 65f32.to_radians();
    view.pitch = 0.0;
    view.distance = 10.0;
    view.target = Vec3::from(object.translation);
    h.set_view3d(view).unwrap();
    h.editor.select(Some(3)).unwrap();
    h.frame(None).unwrap();
    let vp = h.camera();
    let (_, a, d) = interaction::axes(&object, vp)
        .into_iter()
        .find(|(n, _, _)| *n == 0)
        .unwrap();
    let start = a + d.normalize() * 40.0;
    assert_eq!(
        interaction::handle(&object, vp, start.to_array())
            .unwrap()
            .0,
        0
    );
    let drag = AxisDrag::new(vp, Vec3::from(object.translation), 0, start).unwrap();
    let valid = start + d.normalize() * 20.0;
    let delta = drag.delta(valid).unwrap();
    let invalid = Vec2::new(1.0, 1.0);
    assert!(
        drag.delta(invalid).is_none(),
        "counterexample must reject the in-bounds sample"
    );
    h.pointer_down([start.x, start.y + 400.0]);
    assert!(h.gesture_active());
    h.pointer_move([valid.x, valid.y + 400.0]);
    h.pointer_up([invalid.x, invalid.y + 400.0]);
    h.settle().unwrap();
    let edited = h
        .editor
        .document
        .objects
        .iter()
        .find(|o| o.id == 3)
        .unwrap();
    assert!((edited.translation[0] - (object.translation[0] + delta)).abs() < 0.002);
    assert_eq!(h.editor.history().0, 1);
    h.pointer_up([invalid.x, invalid.y + 400.0]);
    h.frame(None).unwrap();
    assert_eq!(h.editor.history().0, 1);
    let undo = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"undo","payload":null});
    h.dispatch(&undo.to_string()).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document, original);
}

#[test]
fn group_focus_aggregates_only_visible_world_descendants() {
    use autographics_auto_driver::authoring::{
        document::{Dimension, Document, Group},
        host::Host,
    };
    let mut doc = Document::sample();
    let mut group = Group::new(10, Dimension::Three);
    group.translation = [13.0, -7.0, 5.0];
    doc.groups.push(group);
    doc.objects[2].parent = Some(10);
    doc.objects[2].translation = [-5.0, 0.0, 0.0];
    let mut right = doc.objects[2].clone();
    right.id = 6;
    right.translation = [5.0, 0.0, 0.0];
    let mut hidden = right.clone();
    hidden.id = 7;
    hidden.translation = [500.0, 0.0, 0.0];
    hidden.visible = false;
    doc.objects.push(right);
    doc.objects.push(hidden);
    let mut h = Host::new(doc.clone()).unwrap();
    h.frame(None).unwrap();
    fn focus(h: &mut Host, id: u64) -> Vec3 {
        h.editor.select(Some(id)).unwrap();
        h.frame(None).unwrap();
        let cmd = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"view3d_revision":h.view3d().revision,"action":"view3d-focus","payload":null});
        h.dispatch(&cmd.to_string()).unwrap();
        h.frame(None).unwrap();
        h.view3d().target
    }
    let left = focus(&mut h, 3);
    let right = focus(&mut h, 6);
    let aggregate = focus(&mut h, 10);
    assert!(
        (aggregate - (left + right) * 0.5).length() < 0.001,
        "hidden descendant must not expand group focus"
    );
    assert_eq!(h.editor.document, doc);
    assert_eq!(h.editor.history().0, 0);
}
