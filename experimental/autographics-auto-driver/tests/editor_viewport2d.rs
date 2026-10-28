use autographics_auto_driver::authoring::{document::*, host::Host, interaction, view2d::*};
use glam::Vec2;
fn command(h: &mut Host, action: &str, payload: serde_json::Value) -> Result<(), String> {
    h.dispatch(&serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":action,"payload":payload}).to_string())
}
#[test]
fn mapping_anchor_bounds_and_grid() {
    assert_eq!(
        View2d::default().screen(Vec2::new(12.0, 7.0)),
        Some(Vec2::new(412.0, 183.0))
    );
    for zoom in [0.125, 1.0, 8.0] {
        for center in [Vec2::ZERO, Vec2::new(-123.0, 321.0), Vec2::splat(65536.0)] {
            let v = View2d {
                center,
                zoom,
                grid: true,
                ..Default::default()
            };
            for p in [Vec2::ZERO, Vec2::new(799.0, 379.0), Vec2::new(400.0, 190.0)] {
                let w = v.world(p).unwrap();
                assert!((v.screen(w).unwrap() - p).abs().max_element() <= 1.0);
                let next = v.wheel(p, 2.0).unwrap();
                // Saturated center is allowed to move the anchor at the supported boundary.
                if next.center.abs().max_element() < 65536.0 {
                    assert!((next.world(p).unwrap() - w).length() < 0.02);
                }
            }
            let (spacing, lines) = v.grid_lines();
            assert!((16.0..=64.0).contains(&(spacing * zoom)));
            assert!(lines.len() <= 128);
            assert!(lines.capacity() * std::mem::size_of::<GridLine>() <= 16384);
            for l in lines {
                assert!(
                    l.center[0] >= 0.0
                        && l.center[0] <= 800.0
                        && l.center[1] >= 0.0
                        && l.center[1] <= 380.0
                );
            }
        }
    }
    assert!(View2d::default()
        .navigate(Vec2::splat(f32::NAN), 1.0)
        .is_err());
    assert!(View2d::default().navigate(Vec2::ZERO, 0.0).is_err());
    assert!(View2d::default()
        .focus(Vec2::splat(-1e6), Vec2::splat(1e6))
        .is_err());
    let v = View2d {
        snap: true,
        ..Default::default()
    };
    assert_eq!(v.snapped(Vec2::new(-5.0, 5.0)), Vec2::new(-10.0, 10.0));
}
#[test]
fn transformed_exact_pick_keeps_reverse_order_and_ellipse() {
    let mut a = Object::new(1, Kind::Quad);
    a.scale = [100.0, 100.0, 1.0];
    let mut b = Object::new(2, Kind::Circle);
    b.scale = [80.0, 40.0, 1.0];
    let v = View2d {
        center: Vec2::new(40.0, -30.0),
        zoom: 2.0,
        ..Default::default()
    };
    assert_eq!(
        interaction::pick2_view(
            &[a.clone(), b.clone()],
            v.screen(Vec2::ZERO).unwrap().to_array(),
            v
        ),
        Some(2)
    );
    assert_eq!(
        interaction::pick2_view(
            &[a, b],
            v.screen(Vec2::new(35.0, -17.0)).unwrap().to_array(),
            v
        ),
        Some(1)
    );
}
#[test]
fn real_auto_drag_view_snap_history_and_busy() {
    let mut h = Host::new(Document::sample()).unwrap();
    let original = h.editor.document.clone();
    let v = View2d {
        center: Vec2::new(-20.0, 10.0),
        zoom: 2.0,
        snap: true,
        grid: true,
        ..Default::default()
    };
    h.set_view2d(v).unwrap();
    h.frame(None).unwrap();
    let p = h.view2d().screen(Vec2::new(-55.0, 0.0)).unwrap().to_array();
    h.pointer_down(p);
    assert_eq!(h.editor.selection, Some(1));
    assert!(h.set_view2d(View2d::default()).is_err());
    h.pointer_move([p[0] + 40.0, p[1] - 10.0]);
    h.pointer_up([p[0] + 40.0, p[1] - 10.0]);
    h.settle().unwrap();
    assert_eq!(h.editor.document.objects[0].translation, [-40.0, 10.0, 0.0]);
    assert_eq!(h.editor.history().0, 1);
    let view = h.view2d();
    command(&mut h, "undo", serde_json::Value::Null).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document, original);
    assert_eq!(h.view2d(), view);
    command(&mut h, "redo", serde_json::Value::Null).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document.objects[0].translation, [-40.0, 10.0, 0.0]);
}
#[test]
fn capture_cancel_unpresented_and_dpi() {
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let hash = h.editor.document.hash().unwrap();
    let start = h.view2d();
    h.navigation_down([300.0, 190.0]);
    h.pointer_move([340.0, 210.0]);
    assert_eq!(h.view2d().center, Vec2::new(-40.0, 20.0));
    h.pointer_down([345.0, 190.0]);
    assert!(!h.gesture_active());
    h.cancel_navigation();
    assert_eq!(h.view2d().center, start.center);
    h.set_view2d(View2d { zoom: 2.0, ..start }).unwrap();
    h.pointer_down([290.0, 190.0]);
    assert!(!h.gesture_active(), "unpresented view rejected");
    h.resize(2400, 1600);
    h.frame(None).unwrap();
    h.pointer_down([580.0, 380.0]);
    assert_eq!(h.editor.selection, Some(1));
    h.pointer_up([580.0, 380.0]);
    assert_eq!(h.editor.document.hash().unwrap(), hash);
    assert_eq!(h.editor.history().0, 0);
}

#[test]
fn parent_group_snap_is_world_absolute_and_children_unchanged() {
    let mut d = Document::sample();
    let mut g = Group::new(10, Dimension::Two);
    g.translation = [13.0, -7.0, 0.0];
    d.groups.push(g);
    d.objects[0].parent = Some(10);
    let mut h = Host::new(d.clone()).unwrap();
    h.set_view2d(View2d {
        zoom: 2.0,
        snap: true,
        center: Vec2::new(20.0, 0.0),
        ..Default::default()
    })
    .unwrap();
    h.frame(None).unwrap();
    let p = h
        .view2d()
        .screen(Vec2::new(-42.0, -7.0))
        .unwrap()
        .to_array();
    h.pointer_down(p);
    h.pointer_up([p[0] + 20.0, p[1]]);
    h.settle().unwrap();
    assert_eq!(h.editor.document.objects[0].translation, [-43.0, -3.0, 0.0]);
    assert_eq!(h.editor.preview.objects[0].translation, [-30.0, -10.0, 0.0]);
    command(&mut h, "undo", serde_json::Value::Null).unwrap();
    h.settle().unwrap();
    h.editor.select(Some(10)).unwrap();
    h.frame(None).unwrap();
    let p = h.view2d().screen(Vec2::new(13.0, -7.0)).unwrap().to_array();
    h.pointer_down(p);
    assert!(h.gesture_active());
    h.pointer_up([p[0] + 20.0, p[1]]);
    h.settle().unwrap();
    assert_eq!(h.editor.document.groups[0].translation, [20.0, -10.0, 0.0]);
    assert_eq!(
        h.editor.document.objects, d.objects,
        "group moves only its local"
    );
}
#[test]
fn view_lifecycle_save_open_failure_play_and_resource_reuse() {
    use autographics_auto_driver::authoring::{
        play,
        storage::{Fault, Storage},
    };
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let base = h.resource_snapshot();
    let doc = h.editor.document.clone();
    for i in 0..20 {
        h.set_view2d(View2d {
            center: Vec2::new(i as f32, -10.0),
            zoom: 2.0,
            grid: i % 2 == 0,
            ..Default::default()
        })
        .unwrap();
        h.frame(None).unwrap();
    }
    let v = h.view2d();
    let before = h.rt.capture(h.two, "view").unwrap().0;
    let now = h.resource_snapshot();
    for key in [
        "mesh_uploads",
        "mesh_bytes",
        "mesh_count",
        "two_buffers",
        "target_bytes",
        "target_count",
    ] {
        assert_eq!(base[key], now[key], "resource {key}");
    }
    assert_eq!(h.editor.document, doc);
    assert!(!h.editor.dirty());
    let dir = std::env::temp_dir().join(format!("plan856-view-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("saved.agscene");
    h.storage.save(&mut h.editor, &path, Fault::None).unwrap();
    assert_eq!(h.view2d(), v);
    let (loaded, assets, code, _) = Storage::load_project(&path, &Default::default()).unwrap();
    let fresh = Host::with_project(loaded, assets, code).unwrap();
    assert_eq!(fresh.view2d(), View2d::default());
    assert_eq!(fresh.editor.document, doc);
    command(
        &mut h,
        "open",
        serde_json::json!(dir.join("absent.agscene")),
    )
    .unwrap();
    assert!(!h.ui().error.is_empty());
    assert_eq!(h.view2d(), v);
    h.start_play(play::SOURCE).unwrap();
    h.frame(None).unwrap();
    assert_eq!(h.presented_view2d(), View2d::default());
    assert!(h.set_view2d(View2d::default()).is_err());
    h.stop_play();
    h.frame(None).unwrap();
    assert_eq!(h.view2d(), v);
    assert_eq!(h.rt.capture(h.two, "restored").unwrap().0, before);
    command(&mut h, "open", serde_json::json!(path)).unwrap();
    h.settle().unwrap();
    assert_eq!(h.view2d().center, Vec2::ZERO);
    assert_eq!(h.view2d().zoom, 1.0);
    assert!(!h.view2d().grid);
}

#[test]
fn view_rejection_focus_drafts_and_noop() {
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    h.editor.select(Some(1)).unwrap();
    command(&mut h, "view-focus", serde_json::Value::Null).unwrap();
    assert_eq!(h.view2d().center, Vec2::new(-55.0, 0.0));
    assert!((h.view2d().zoom - 6.08).abs() < 0.01);
    h.frame(None).unwrap();
    let old = h.view2d();
    let pixels = h.rt.capture(h.two, "before-reject").unwrap().0;
    h.fail_resource = true;
    assert!(h.set_view2d(View2d::default()).is_err());
    h.fail_resource = false;
    assert_eq!(h.view2d(), old);
    h.frame(None).unwrap();
    assert_eq!(pixels, h.rt.capture(h.two, "after-reject").unwrap().0);
    h.ui_mut().draft[0] = "unsubmitted".into();
    h.scroll(
        [300.0, 190.0],
        iced_wgpu::core::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
    );
    assert_eq!(h.ui().draft[0], "unsubmitted");
    h.frame(None).unwrap();
    let navigation_start = h.view2d();
    h.navigation_down([300.0, 190.0]);
    h.pointer_move([310.0, 200.0]);
    assert!(h.set_view2d(View2d::default()).is_err());
    h.blur();
    assert_eq!(h.view2d().center, navigation_start.center);
    let v = h.view2d();
    h.scroll(
        [300.0, 190.0],
        iced_wgpu::core::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
    );
    let wheel = h.view2d();
    h.blur();
    assert_eq!(wheel, h.view2d());
    assert_ne!(v.zoom, wheel.zoom);
    let mut d = Document::sample();
    d.objects[0].translation = [-40.0, 10.0, 0.0];
    let mut h = Host::new(d.clone()).unwrap();
    h.set_view2d(View2d {
        snap: true,
        ..Default::default()
    })
    .unwrap();
    h.frame(None).unwrap();
    h.pointer_down([360.0, 180.0]);
    h.pointer_move([370.0, 180.0]);
    h.pointer_up([360.0, 180.0]);
    h.settle().unwrap();
    assert_eq!(h.editor.document, d);
    assert_eq!(h.editor.history().0, 0);
    assert_eq!(
        h.input.keyboard_target(),
        Some(h.two),
        "normal release preserves actual focus"
    );
    for scale in [1.0, 1.5, 2.0] {
        h.resize((1200.0 * scale) as u32, (800.0 * scale) as u32);
        h.frame(None).unwrap();
        h.pointer_down([360.0 * scale, 180.0 * scale]);
        assert_eq!(h.editor.selection, Some(1));
        h.pointer_up([360.0 * scale, 180.0 * scale]);
    }
    h.editor.select(Some(3)).unwrap();
    let v = h.view2d();
    assert!(command(&mut h, "view-focus", serde_json::Value::Null).is_err());
    assert_eq!(h.view2d(), v);
}

#[test]
fn zero_window_and_nonfinite_navigation_input_reject_before_capture_or_view_change() {
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let original = h.view2d();
    for size in [(0, 0), (0, 800), (1200, 0)] {
        h.resize(size.0, size.1);
        h.scroll(
            [0.0, 0.0],
            iced_wgpu::core::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        );
        h.navigation_down([0.0, 0.0]);
        h.pointer_move([0.1, 0.1]);
        h.navigation_up([0.1, 0.1]);
        assert_eq!(h.view2d(), original);
        assert!(!h.gesture_active());
    }
    h.resize(1200, 800);
    h.frame(None).unwrap();
    for point in [[f32::NAN, 0.0], [0.0, f32::INFINITY]] {
        h.scroll(
            point,
            iced_wgpu::core::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        );
        h.navigation_down(point);
        assert_eq!(h.view2d(), original);
    }
    h.scroll(
        [300.0, 190.0],
        iced_wgpu::core::mouse::ScrollDelta::Lines {
            x: 0.0,
            y: f32::NAN,
        },
    );
    assert_eq!(h.view2d(), original);
}

#[test]
fn pixel_scroll_uses_logical_child_units_and_preserves_anchor() {
    use iced_wgpu::core::mouse::ScrollDelta;
    for (scale, physical) in [(1.0, 40.0), (1.5, 60.0), (2.0, 80.0)] {
        for (delta, expected) in [
            (physical, 1.1f32),
            (-physical, 1.0 / 1.1),
            (physical * 1000.0, 1.1f32.powi(8)),
            (-physical * 1000.0, 1.1f32.powi(-8)),
        ] {
            let mut h = Host::new(Document::sample()).unwrap();
            h.resize((1200.0 * scale) as u32, (800.0 * scale) as u32);
            h.frame(None).unwrap();
            let before = h.editor.document.clone();
            let anchor = h.view2d().world(Vec2::new(300.0, 130.0)).unwrap();
            h.scroll(
                [300.0 * scale, 130.0 * scale],
                ScrollDelta::Pixels { x: 0.0, y: delta },
            );
            assert!(
                (h.view2d().zoom - expected).abs() < 1e-5,
                "scale={scale} delta={delta}"
            );
            assert!((h.view2d().world(Vec2::new(300.0, 130.0)).unwrap() - anchor).length() < 1e-3);
            assert_eq!(h.editor.document, before);
            assert_eq!(h.editor.history().0, 0);
        }
        let mut h = Host::new(Document::sample()).unwrap();
        h.resize((1200.0 * scale) as u32, (800.0 * scale) as u32);
        h.frame(None).unwrap();
        h.scroll(
            [300.0 * scale, 190.0 * scale],
            ScrollDelta::Lines { x: 0.0, y: 1.0 },
        );
        assert!((h.view2d().zoom - 1.1).abs() < 1e-6);
        let v = h.view2d();
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            h.scroll(
                [300.0 * scale, 190.0 * scale],
                ScrollDelta::Pixels { x: 0.0, y: bad },
            );
            assert_eq!(h.view2d(), v);
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            h.scroll(
                [300.0 * scale, 190.0 * scale],
                ScrollDelta::Pixels {
                    x: bad,
                    y: physical,
                },
            );
            assert_eq!(h.view2d(), v);
            h.scroll(
                [300.0 * scale, 190.0 * scale],
                ScrollDelta::Lines { x: bad, y: 1.0 },
            );
            assert_eq!(h.view2d(), v);
        }
        h.scroll(
            [1000.0 * scale, 500.0 * scale],
            ScrollDelta::Pixels {
                x: 0.0,
                y: physical,
            },
        );
        assert_eq!(
            h.view2d(),
            v,
            "panel consumes wheel without view navigation"
        );
    }
}

#[test]
fn cancellation_restores_admitted_frame_under_preparation_failures() {
    for (edge, fault, after_upload) in [
        ("cancel", false, false),
        ("cancel", true, false),
        ("blur", true, false),
        ("resize", true, false),
        ("cancel", false, true),
        ("blur", false, true),
        ("resize", false, true),
        ("failed-move", true, false),
        ("failed-move", false, true),
    ] {
        let mut h = Host::new(Document::sample()).unwrap();
        h.set_view2d(View2d {
            center: Vec2::new(17.0, -9.0),
            grid: true,
            ..Default::default()
        })
        .unwrap();
        h.frame(None).unwrap();
        let start = h.view2d();
        let doc = h.editor.document.clone();
        let history = h.editor.history();
        let original = h.rt.capture(h.two, "original").unwrap().0;
        let resources = h.resource_snapshot();
        h.navigation_down([300.0, 190.0]);
        h.pointer_move([340.0, 210.0]);
        h.frame(None).unwrap();
        assert_ne!(h.rt.capture(h.two, "moved").unwrap().0, original);
        h.fail_resource = fault;
        h.fail_after_asset_upload = after_upload;
        match edge {
            "blur" => h.blur(),
            "resize" => h.resize(2400, 1600),
            "failed-move" => h.pointer_move([350.0, 220.0]),
            _ => h.cancel_navigation(),
        }
        assert_eq!(h.fail_resource, fault);
        assert_eq!(h.fail_after_asset_upload, after_upload);
        if edge == "failed-move" {
            assert!(h.ui().error.contains("preparation"));
        }
        h.pointer_move([390.0, 240.0]);
        h.navigation_up([390.0, 240.0]);
        assert_eq!(
            h.view2d().center,
            start.center,
            "{edge} {fault} {after_upload}"
        );
        assert_eq!(h.view2d().zoom, start.zoom);
        h.frame(None).unwrap();
        assert_eq!(
            h.rt.capture(h.two, "restored").unwrap().0,
            original,
            "{edge} {fault} {after_upload}"
        );
        assert_eq!(h.editor.document, doc);
        assert_eq!(h.editor.history(), history);
        let now = h.resource_snapshot();
        for key in [
            "mesh_uploads",
            "mesh_bytes",
            "mesh_count",
            "target_count",
            "two_buffers",
        ] {
            assert_eq!(now[key], resources[key], "{edge} {key}");
        }
    }
}

#[test]
fn rejected_wheel_preserves_last_frame_and_close_cancels_capture() {
    use iced_wgpu::core::mouse::ScrollDelta;
    for after_upload in [false, true] {
        let mut h = Host::new(Document::sample()).unwrap();
        h.frame(None).unwrap();
        h.scroll([300.0, 190.0], ScrollDelta::Lines { x: 0.0, y: 1.0 });
        h.frame(None).unwrap();
        let view = h.view2d();
        let pixels = h.rt.capture(h.two, "last-good").unwrap().0;
        h.fail_resource = !after_upload;
        h.fail_after_asset_upload = after_upload;
        h.scroll([300.0, 190.0], ScrollDelta::Pixels { x: 0.0, y: 40.0 });
        assert_eq!(h.view2d(), view);
        assert!(h.ui().error.contains("preparation"));
        h.frame(None).unwrap();
        assert_eq!(h.rt.capture(h.two, "still-good").unwrap().0, pixels);
        assert_eq!(h.editor.history().0, 0);

        h.fail_resource = false;
        h.fail_after_asset_upload = false;
        h.navigation_down([300.0, 190.0]);
        h.pointer_move([340.0, 210.0]);
        h.frame(None).unwrap();
        h.fail_resource = !after_upload;
        h.fail_after_asset_upload = after_upload;
        h.request_close();
        assert!(h.closed);
        assert_eq!(h.view2d().center, view.center);
        assert_eq!(h.fail_resource, !after_upload);
        assert_eq!(h.fail_after_asset_upload, after_upload);
        h.pointer_move([390.0, 240.0]);
        h.navigation_up([390.0, 240.0]);
        assert_eq!(h.view2d().center, view.center);
    }
}
