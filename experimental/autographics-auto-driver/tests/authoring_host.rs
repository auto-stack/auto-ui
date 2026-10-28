use autographics_auto_driver::authoring::{
    document::*,
    host::Host,
    session::Edit,
    storage::{Fault, Saved},
};
use serde_json::{json, Value};
fn msg(h: &Host, a: &str, p: Value) -> String {
    json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":a,"payload":p})
        .to_string()
}
fn command(h: &mut Host, a: &str, p: Value) {
    let m = msg(h, a, p);
    h.dispatch(&m).unwrap();
    h.settle().unwrap();
}
#[test]
fn pixels_transaction_failures_and_real_widgets() {
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    let initial = h.rt.capture(h.two, "initial").unwrap().0;
    let initial3 = h.rt.capture(h.three, "initial3").unwrap().0;
    command(&mut h, "select", json!(1));
    let old = h.editor.document.clone();
    let mut a = old.objects[0].clone();
    a.translation[0] += 80.0;
    h.editor.begin(h.editor.stamp, Edit::Replace(a)).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document.objects[1], old.objects[1]);
    assert_ne!(h.rt.capture(h.two, "moved").unwrap().0, initial);
    assert_eq!(h.rt.capture(h.three, "same3").unwrap().0, initial3);
    command(&mut h, "select", json!(3));
    let three = h.rt.capture(h.three, "selected3").unwrap().0;
    let mut a = h.editor.document.objects[2].clone();
    a.translation[0] += 2.0;
    h.editor.begin(h.editor.stamp, Edit::Replace(a)).unwrap();
    h.settle().unwrap();
    assert_ne!(h.rt.capture(h.three, "moved3").unwrap().0, three);
    let before = h.editor.document.clone();
    let history = h.editor.history();
    let pixels = h.rt.capture(h.three, "lastgood").unwrap().0;
    let mut a = before.objects[2].clone();
    a.translation[0] += 1.0;
    h.fail_resource = true;
    h.editor
        .begin(h.editor.stamp, Edit::Replace(a.clone()))
        .unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document, before);
    assert_eq!(h.editor.history(), history);
    assert_eq!(h.rt.capture(h.three, "failed").unwrap().0, pixels);
    h.fail_resource = false;
    h.editor.begin(h.editor.stamp, Edit::Replace(a)).unwrap();
    h.settle().unwrap();
    assert_ne!(h.rt.capture(h.three, "recovered").unwrap().0, pixels);
    // Locate a real list button through actual Iced clicks; observe the consumed stable ID.
    let b = h.widget_bounds("node-1").unwrap();
    h.click([810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5]);
    h.frame(None).unwrap();
    assert_eq!(h.editor.selection, Some(1), "real object-list widget click");
    let mut bounds = h.widget_bounds("authoring-name").unwrap();
    for _ in 0..20 {
        if bounds.y >= 0.0 && bounds.y + bounds.height < 800.0 {
            break;
        }
        h.scroll(
            [1000.0, 500.0],
            iced_wgpu::core::mouse::ScrollDelta::Lines {
                x: 0.0,
                y: if bounds.y > 750.0 { -3.0 } else { 3.0 },
            },
        );
        h.frame(None).unwrap();
        bounds = h.widget_bounds("authoring-name").unwrap();
    }
    assert!(
        bounds.y >= 0.0 && bounds.y + bounds.height < 800.0,
        "name input must be scrolled into the real viewport"
    );
    assert!(bounds.width > 0.0);
    h.click([810.0 + bounds.x + 20.0, bounds.y + 10.0]);
    h.frame(None).unwrap();
    assert!(h
        .rt
        .iced_text_input_focused(h.panel, iced_wgpu::core::widget::Id::new("authoring-name")));
    h.settle().unwrap();
    let mut edited = h.editor.document.objects[0].clone();
    edited.name = "close-guard".into();
    h.editor.select(Some(1)).unwrap();
    h.editor
        .begin(h.editor.stamp, Edit::Replace(edited))
        .unwrap();
    h.settle().unwrap();
    let doc = h.editor.document.clone();
    h.blur();
    assert_eq!(h.editor.document, doc);
    assert_eq!(h.ui().draft[0], doc.objects[0].name);
    // Dirty close cancel preserves content, history and rendering.
    h.request_close();
    assert!(h.ui().dialog);
    command(&mut h, "leave-cancel", Value::Null);
    assert!(!h.closed);
    assert_eq!(h.editor.document, doc);
    let dir = std::env::temp_dir().join(format!("ag-reopen-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("saved.agscene");
    assert_eq!(
        h.storage.save(&mut h.editor, &path, Fault::None).unwrap(),
        Saved::Confirmed
    );
    command(&mut h, "select", json!(1));
    h.editor.select(None).unwrap(); // Save does not persist selection/highlight.
    let mut fresh = Host::new(Document::sample()).unwrap();
    fresh.open(&path).unwrap();
    fresh.settle().unwrap();
    assert_eq!(fresh.editor.document, doc);
    assert_eq!(fresh.editor.history(), (0, 0, 0));
    assert!(!fresh.editor.dirty());
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&dir).unwrap();
}

#[test]
fn twenty_open_edit_close_actual_owners_and_warm_resources() {
    use agp::host::surface_runtime::SurfaceContent;
    use std::sync::Arc;
    for cycle in 0..20 {
        let mut h = Host::new(Document::sample()).unwrap();
        h.frame(None).unwrap();
        let targets = h.rt.stats.target_creations;
        let sequences =
            [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq);
        for _ in 0..3 {
            h.frame(None).unwrap();
        }
        assert_eq!(h.rt.stats.target_creations, targets);
        assert_eq!(
            [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq),
            sequences
        );
        h.editor.select(Some(1)).unwrap();
        let old_owner = h.editor.preview.vm.owner_probe();
        let mut o = h.editor.document.objects[0].clone();
        o.translation[0] += cycle as f32 + 1.0;
        h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
        h.settle().unwrap();
        assert!(!old_owner());
        let owner = h.editor.preview.vm.owner_probe();
        h.editor.begin_open(Document::sample()).unwrap();
        h.settle().unwrap();
        assert!(!owner());
        let owner = h.editor.preview.vm.owner_probe();
        let (pin, bytes) = if let Some(SurfaceContent::Scene3d(c)) = h.rt.content(h.three) {
            let first = &c.instances[0];
            (
                c.store.pin(&first.mesh.0, &first.mesh.1).unwrap(),
                c.store.owned_buffer_bytes(),
            )
        } else {
            panic!("scene")
        };
        assert!(bytes > 0);
        let weakmesh = Arc::downgrade(&pin);
        let rt = Arc::new(h.rt);
        let weakrt = Arc::downgrade(&rt);
        let held = rt.clone();
        h.editor.close();
        assert!(!owner());
        drop(rt);
        assert!(weakrt.upgrade().is_some(), "legitimate runtime pin");
        drop(held);
        assert!(
            weakrt.upgrade().is_none(),
            "runtime targets last owner drops"
        );
        assert!(weakmesh.upgrade().is_some(), "legitimate mesh pin");
        drop(pin);
        assert!(
            weakmesh.upgrade().is_none(),
            "actual GPU buffers last owner drops"
        );
    }
    println!("20 actual VM/runtime target/mesh-owner cycles; warm target identity stable");
}
fn click_visible_widget(h: &mut Host, id: &str) {
    for _ in 0..30 {
        let b = h.widget_bounds(id).unwrap();
        if b.y >= 0.0 && b.y + b.height < 800.0 {
            h.click([810.0 + b.x + 10.0, b.y + b.height * 0.5]);
            return;
        }
        h.scroll(
            [1000.0, 500.0],
            iced_wgpu::core::mouse::ScrollDelta::Lines {
                x: 0.0,
                y: if b.y > 750.0 { -3.0 } else { 3.0 },
            },
        );
        h.frame(None).unwrap();
    }
    panic!("widget {id} must be visible before actual Iced click");
}
#[test]
fn actual_create_delete_widgets_and_bad_open() {
    let mut h = Host::new(Document::sample()).unwrap();
    h.frame(None).unwrap();
    click_visible_widget(&mut h, "action-create");
    h.settle().unwrap();
    assert_eq!(
        h.editor.document.objects.len(),
        5,
        "create button must publish through Auto"
    );
    click_visible_widget(&mut h, "node-5");
    h.frame(None).unwrap();
    assert_eq!(h.editor.selection, Some(5));
    click_visible_widget(&mut h, "action-delete-request");
    h.settle().unwrap();
    assert_eq!(
        h.editor.document.objects.len(),
        4,
        "delete button must publish through Auto"
    );
    assert_eq!(h.editor.selection, None);
    let d = h.editor.document.clone();
    let history = h.editor.history();
    let pixels = h.rt.capture(h.two, "before-bad-open").unwrap().0;
    assert!(h
        .open(std::path::Path::new("nonexistent-authoring-file.agscene"))
        .is_err());
    assert_eq!(h.editor.document, d);
    assert_eq!(h.editor.history(), history);
    h.frame(None).unwrap();
    assert_eq!(h.rt.capture(h.two, "after-bad-open").unwrap().0, pixels);
}
#[test]
fn stale_draft_apply_and_navigation_guards() {
    use agp::host::iced_surface::SurfaceUi;
    let mut h = Host::new(Document::sample()).unwrap();
    command(&mut h, "select", json!(1));
    let old = h.editor.stamp;
    let draft = format!("draft:{}:{}:1:0:late A", old.epoch, old.revision);
    let stale_apply = msg(&h, "apply", json!({"target":1,"draft":h.ui().draft}));
    command(&mut h, "select", json!(2));
    h.ui_mut().update(draft);
    assert_eq!(h.ui().draft[0], "Object");
    assert!(h.dispatch(&stale_apply).is_err());
    let mut o = h.editor.document.objects[1].clone();
    o.name = "dirty B".into();
    h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
    h.settle().unwrap();
    let doc = h.editor.document.clone();
    let history = h.editor.history();
    command(&mut h, "new", Value::Null);
    assert!(h.ui().dialog);
    command(&mut h, "leave-cancel", Value::Null);
    assert_eq!(h.editor.document, doc);
    assert_eq!(h.editor.history(), history);
    command(&mut h, "new", Value::Null);
    command(&mut h, "leave-discard", Value::Null);
    assert!(h.editor.document.objects.is_empty());
    assert_eq!(h.editor.history(), (0, 0, 0));
    h.ui_mut()
        .update(format!("draft:{}:{}:2:0:late B", old.epoch, old.revision));
    assert!(h.ui().draft.is_empty());
    assert!(h.dispatch(&stale_apply).is_err());
}
#[test]
fn shared_mount_once_and_upload_accounting_control() {
    use agp::core::composition::{MountKind, Rect};
    let mut h = Host::new(Document::sample()).unwrap();
    let view = h.rt.graph.views()[0].id;
    h.rt.add_portal(
        view,
        h.two,
        MountKind::Flat2d {
            offset: [0.0, 0.0],
            scale: [0.1, 0.1],
            clip: Rect::at(0.0, 0.0, 80.0, 38.0),
        },
        false,
    )
    .unwrap();
    h.editor.select(Some(1)).unwrap();
    let mut o = h.editor.document.objects[0].clone();
    o.translation[0] += 20.0;
    h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
    h.settle().unwrap();
    assert_eq!(h.accepted, 1);
    assert!(h.rt.stats.last_frame_renders.values().all(|n| *n == 1));
    let oracle = |h: &Host| {
        let mut sum = h.rt.composition_upload_stats();
        sum.merge(&h.rt.content_uploads());
        let actual = h.rt.upload_stats();
        sum.buffer_rewrites == actual.buffer_rewrites
            && sum.bytes_rewritten == actual.bytes_rewritten
    };
    assert!(oracle(&h));
    h.rt.set_content_upload_aggregation(false);
    assert!(!oracle(&h), "omitted content uploads must fail same oracle");
    h.rt.set_content_upload_aggregation(true);
    assert!(oracle(&h));
    let report = h.resource_snapshot();
    assert_eq!(report["target_count"], 4);
    assert_eq!(report["target_bytes"], 7584000);
    // PLAN-836 warms all three fixed resources: cylinder(1), player(3), mob(3).
    assert_eq!(report["mesh_count"], 7);
    println!("resource_snapshot={report}");
}
#[test]
fn selection_and_failed_discard_open_recover_controls() {
    let mut h = Host::new(Document::sample()).unwrap();
    command(&mut h, "select", json!(1));
    h.fail_resource = true;
    let m = msg(&h, "select", json!(2));
    assert!(h.dispatch(&m).is_err());
    assert_eq!(h.editor.selection, Some(1));
    h.fail_resource = false;
    let mut o = h.editor.document.objects[0].clone();
    o.name = "dirty".into();
    h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
    h.settle().unwrap();
    command(&mut h, "open", json!("does-not-exist.agscene"));
    assert!(h.ui().dialog);
    let m = msg(&h, "leave-discard", Value::Null);
    h.ui_mut().queue.push(m);
    h.frame(None).unwrap();
    assert!(!h.ui().dialog, "bad Open must restore enabled controls");
    assert!(h.editor.dirty());
    command(&mut h, "select", json!(2));
    assert_eq!(h.editor.selection, Some(2));
}
#[test]
fn maximum_persistent_id_opens_edits_saves_and_reopens() {
    let mut d = Document::empty("upper-id".into());
    d.objects.push(Object::new(u64::MAX, Kind::Circle));
    d.encode().unwrap();
    let mut h = Host::new(d.clone()).unwrap();
    h.frame(None).unwrap();
    command(&mut h, "select", json!(u64::MAX));
    let m = msg(&h, "create", json!("circle"));
    assert!(h.dispatch(&m).unwrap_err().contains("identity exhausted"));
    assert_eq!(h.editor.document, d);
    let mut o = d.objects[0].clone();
    o.translation[0] = 45.0;
    h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
    h.settle().unwrap();
    let dir = std::env::temp_dir().join(format!("ag-maxid-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("scene.agscene");
    h.storage.save(&mut h.editor, &path, Fault::None).unwrap();
    let mut opened = Host::new(Document::sample()).unwrap();
    opened.open(&path).unwrap();
    opened.settle().unwrap();
    assert_eq!(opened.editor.document, h.editor.document);
    assert!(!opened.editor.dirty());
    command(&mut opened, "select", json!(u64::MAX));
    let mut o = opened.editor.document.objects[0].clone();
    o.name = "still editable".into();
    opened
        .editor
        .begin(opened.editor.stamp, Edit::Replace(o))
        .unwrap();
    opened.settle().unwrap();
    opened
        .storage
        .save(&mut opened.editor, &path, Fault::None)
        .unwrap();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
