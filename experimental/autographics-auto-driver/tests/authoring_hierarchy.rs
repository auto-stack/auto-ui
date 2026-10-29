use autographics_auto_driver::authoring::{
    document::*,
    hierarchy,
    host::Host,
    interaction,
    preview::{self, Building, SOURCE},
    session::{Edit, Editor},
    storage::{Fault, Saved, Storage},
};
use serde_json::{json, Value};
fn scene() -> Document {
    let mut d = Document::sample();
    let mut a = Group::new(10, Dimension::Two);
    a.translation = [20.0, 10.0, 0.0];
    let mut b = Group::new(11, Dimension::Two);
    b.parent = Some(10);
    b.translation = [10.0, -5.0, 0.0];
    let mut c = Group::new(12, Dimension::Three);
    c.translation = [0.5, 0.0, 0.0];
    let mut e = Group::new(13, Dimension::Three);
    e.parent = Some(12);
    e.translation = [0.5, 1.0, 0.0];
    d.groups = vec![a, b, c, e];
    d.objects[0].parent = Some(11);
    d.objects[1].parent = Some(10);
    d.objects[2].parent = Some(13);
    d
}
fn settle(e: &mut Editor) {
    while !e.pump().unwrap() {}
    e.commit().unwrap();
}
fn cmd(h: &mut Host, a: &str, p: Value) -> Result<(), String> {
    h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":a,"payload":p}).to_string())?;
    h.settle()
}
fn group_move(e: &mut Editor, id: u64, x: f32) {
    e.select(Some(id)).unwrap();
    let mut g = e
        .document
        .groups
        .iter()
        .find(|g| g.id == id)
        .unwrap()
        .clone();
    g.translation[0] = x;
    e.begin(e.stamp, Edit::ReplaceGroup(g)).unwrap();
    settle(e);
}
#[test]
fn strict_v2_forest_and_explicit_v1_migration() {
    let d = scene();
    let b = d.encode().unwrap();
    assert_eq!(Document::decode(&b).unwrap(), d);
    let mut bad = vec![];
    let mut v = serde_json::to_value(&d).unwrap();
    v["groups"][0]["id"] = json!(1);
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["objects"][0]["parent"] = json!(999);
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["groups"][0]["parent"] = json!(10);
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["groups"][0]["parent"] = json!(11);
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["objects"][0]["parent"] = json!(2);
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["objects"][0]["parent"] = json!(12);
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["groups"][0]["rotation"] = json!([0, 0, 0]);
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["objects"][0].as_object_mut().unwrap().remove("parent");
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["groups"][0].as_object_mut().unwrap().remove("parent");
    bad.push(v);
    let mut v = serde_json::to_value(&d).unwrap();
    v["groups"][0]["translation"][2] = json!(1);
    bad.push(v);
    for v in bad {
        assert!(
            Document::decode(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{v}"
        );
    }
    let s = String::from_utf8(b).unwrap();
    assert!(Document::decode(
        s.replacen("\"parent\":null", "\"parent\":null,\"parent\":null", 1)
            .as_bytes()
    )
    .is_err());
    let legacy = include_bytes!("../samples/authoring/example.agscene");
    let imported = Document::decode(legacy).unwrap();
    assert_eq!(imported.schema, "autographics.scene/5");
    assert!(imported.groups.is_empty());
    assert!(imported.objects.iter().all(|o| o.parent.is_none()));
    let mut v: Value = serde_json::from_slice(legacy).unwrap();
    v["objects"][0]["parent"] = Value::Null;
    assert!(Document::decode(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn shared_capacity_deep_chain_hidden_world_and_u64_max() {
    let mut d = Document::empty("deep".into());
    for id in 1..=127 {
        let mut g = Group::new(id, Dimension::Two);
        g.parent = (id > 1).then_some(id - 1);
        g.translation = [1.0, 0.0, 0.0];
        d.groups.push(g);
    }
    let mut o = Object::new(u64::MAX, Kind::Circle);
    o.parent = Some(127);
    d.objects.push(o);
    d.encode().unwrap();
    assert_eq!(
        hierarchy::derive(&d).unwrap()[&u64::MAX].translation,
        [127.0, 0.0, 0.0]
    );
    assert_eq!(preview::build(&d).unwrap().objects[0].id, u64::MAX);
    d.groups.push(Group::new(129, Dimension::Two));
    assert!(d.encode().unwrap_err().contains("128"));
    d.groups.pop();
    d.groups[0].parent = Some(127);
    assert!(d.encode().unwrap_err().contains("cycle"));
    d.groups[0].parent = None;
    d.groups[0].visible = false;
    d.groups[0].translation[0] = 100000.0;
    assert!(d.encode().unwrap_err().contains("world"));
    let mut d = Document::empty("old128".into());
    for id in 1..=128 {
        d.objects.push(Object::new(id, Kind::Circle));
    }
    let mut v = serde_json::to_value(&d).unwrap();
    v["schema"] = json!("autographics.scene/1");
    v.as_object_mut().unwrap().remove("behavior");
    v.as_object_mut().unwrap().remove("groups");
    v.as_object_mut().unwrap().remove("assets");
    for o in v["objects"].as_array_mut().unwrap() {
        o.as_object_mut().unwrap().remove("parent");
        o.as_object_mut().unwrap().remove("resource");
        // scene/1 predates the PNG visual binding (PLAN-004): a real v1 file
        // never carries it and the reader rejects it instead of upgrading.
        o.as_object_mut().unwrap().remove("sprite");
    }
    let imported = Document::decode(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(imported.objects.len(), 128);
    let mut e = Editor::new(imported.clone()).unwrap();
    assert!(e
        .begin(e.stamp, Edit::CreateGroup(Group::new(129, Dimension::Two)))
        .is_err());
    assert_eq!(e.document, imported);
}
#[test]
fn actual_auto_world_output_and_mutation_controls() {
    let mut d = scene();
    d.groups[0].visible = false;
    d.objects[2].yaw = 0.4;
    d.objects[2].scale = [1.0, 2.0, 0.5];
    let p = preview::build(&d).unwrap();
    assert_eq!(p.objects[0].translation, [-25.0, 5.0, 0.0]);
    assert!(!p.objects[0].visible);
    assert_eq!(p.objects[2].translation, [-0.5, 1.0, 0.0]);
    assert_eq!(p.objects[2].scale, [1.0, 2.0, 0.5]);
    assert!(p.matches(&d));
    for source in [
        SOURCE.replace(
            "world = world + worlds[parent * 3 + axis]",
            "world = world - worlds[parent * 3 + axis]",
        ),
        SOURCE.replace(
            "if effective[parent] == 0 { shown = 0 }",
            "if effective[parent] == 1 { shown = 0 }",
        ),
        SOURCE.replace("return parents[i]", "return -1"),
        SOURCE.replace("return identities[i]", "return identities[i] + 1"),
        SOURCE.replace("return 4", "return 3"),
    ] {
        assert_ne!(source, SOURCE);
        let mut b = Building::new(&d, &source).unwrap();
        let owner = b.owner_probe();
        while !b.pump().unwrap() {}
        assert!(b.finish().is_err());
        assert!(!owner());
    }
    assert_eq!(
        preview::build(&d).unwrap().objects,
        p.objects,
        "accepted control recovers"
    );
}
#[test]
fn reparent_keep_world_visibility_precision_and_atomic_rejections() {
    let d = scene();
    let mut e = Editor::new(d.clone()).unwrap();
    e.select(Some(11)).unwrap();
    let before = e.preview.nodes.clone();
    e.begin(
        e.stamp,
        Edit::Reparent {
            target: 11,
            expected_parent: Some(10),
            parent: None,
        },
    )
    .unwrap();
    settle(&mut e);
    assert_eq!(e.document.groups[1].translation, [30.0, 5.0, 0.0]);
    assert_eq!(e.document.objects, d.objects);
    assert_eq!(e.preview.nodes, before);
    assert_eq!(e.history().0, 1);
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    assert_eq!(e.document, d);
    assert!(!e.dirty());
    e.select(Some(10)).unwrap();
    for parent in [Some(11), Some(12), Some(1), Some(999)] {
        let stamp = e.stamp;
        let history = e.history();
        assert!(e
            .begin(
                stamp,
                Edit::Reparent {
                    target: 10,
                    expected_parent: None,
                    parent
                }
            )
            .is_err());
        assert_eq!(e.document, d);
        assert_eq!(e.stamp, stamp);
        assert_eq!(e.history(), history);
        assert_eq!(e.selection, Some(10));
    }
    assert!(e
        .begin(
            e.stamp,
            Edit::Reparent {
                target: 10,
                expected_parent: Some(11),
                parent: None
            }
        )
        .unwrap_err()
        .contains("stale parent"));
    let mut x = Document::empty("precision".into());
    let mut g = Group::new(1, Dimension::Two);
    g.translation[0] = 100000.0;
    x.groups.push(g);
    let mut o = Object::new(2, Kind::Circle);
    o.translation[0] = 0.002;
    x.objects.push(o);
    assert!(hierarchy::reparent(&x, 2, None, Some(1))
        .unwrap_err()
        .contains("precision"));
    x.objects[0].translation[0] = 10.0;
    let re = hierarchy::reparent(&x, 2, None, Some(1)).unwrap();
    assert_eq!(
        hierarchy::derive(&re).unwrap()[&2].translation,
        [10.0, 0.0, 0.0]
    );
    x.groups[0].visible = false;
    let re = hierarchy::reparent(&x, 2, None, Some(1)).unwrap();
    assert!(re.objects[0].visible);
    assert!(!preview::build(&re).unwrap().objects[0].visible);
}
#[test]
fn subtree_history_order_branch_and_empty_groups() {
    let d = scene();
    let mut e = Editor::new(d.clone()).unwrap();
    e.select(Some(10)).unwrap();
    e.begin(e.stamp, Edit::Delete(10)).unwrap();
    settle(&mut e);
    assert!(!e.document.contains(1));
    assert!(!e.document.contains(2));
    assert!(!e.document.contains(11));
    assert_eq!(e.document.objects, d.objects[2..]);
    assert_eq!(e.selection, None);
    assert_eq!(e.history().0, 1);
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    assert_eq!(e.document, d);
    e.begin(e.stamp, Edit::Redo).unwrap();
    settle(&mut e);
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    e.begin(e.stamp, Edit::CreateGroup(Group::new(99, Dimension::Two)))
        .unwrap();
    settle(&mut e);
    assert_eq!(e.history().1, 0);
    group_move(&mut e, 99, 40.0);
    assert_eq!(e.preview.nodes[&99].translation, [40.0, 0.0, 0.0]);
    for i in 0..66 {
        group_move(&mut e, 99, i as f32);
    }
    assert_eq!(e.history().0, 64);
    e.history_limit = 1;
    assert!(e.begin(e.stamp, Edit::Delete(99)).is_err());
}
#[test]
fn host_world_local_gestures_reparent_failure_and_recovery() {
    let d = scene();
    let mut h = Host::new(d.clone()).unwrap();
    h.frame(None).unwrap();
    assert_eq!(
        interaction::pick2(&h.editor.preview.objects, [375.0, 185.0]),
        Some(1)
    );
    h.pointer_down([375.0, 185.0]);
    h.pointer_up([385.0, 175.0]);
    h.settle().unwrap();
    assert_eq!(h.editor.document.objects[0].translation, [-45.0, 10.0, 0.0]);
    assert_eq!(h.editor.preview.objects[0].translation, [-15.0, 15.0, 0.0]);
    cmd(&mut h, "undo", Value::Null).unwrap();
    cmd(&mut h, "select", json!(10)).unwrap();
    let before = h.editor.document.clone();
    let resources = h.resource_snapshot();
    h.pointer_down([420.0, 180.0]);
    assert!(h.gesture_active());
    h.pointer_move([450.0, 160.0]);
    h.pointer_up([450.0, 160.0]);
    h.pointer_up([490.0, 160.0]);
    h.settle().unwrap();
    assert_eq!(h.editor.document.groups[0].translation, [50.0, 30.0, 0.0]);
    assert_eq!(h.editor.document.objects, before.objects);
    assert_eq!(h.editor.preview.objects[0].translation, [5.0, 25.0, 0.0]);
    assert_eq!(h.editor.preview.objects[3], before.objects[3]);
    assert_eq!(h.editor.history().0, 1);
    assert_eq!(
        h.resource_snapshot()["mesh_uploads"],
        resources["mesh_uploads"]
    );
    assert_eq!(
        h.resource_snapshot()["target_bytes"],
        resources["target_bytes"]
    );
    let accepted = h.editor.document.clone();
    let history = h.editor.history();
    let stamp = h.editor.stamp;
    h.pointer_down([450.0, 160.0]);
    h.pointer_move([460.0, 160.0]);
    h.frame(None).unwrap();
    assert!(h.dispatch(&json!({"epoch":stamp.epoch,"revision":stamp.revision,"action":"reparent","payload":{"target":10,"expected_parent":null,"parent":11}}).to_string()).is_err());
    h.cancel_gesture();
    assert_eq!(h.editor.document, accepted);
    assert_eq!(h.editor.history(), history);
    h.frame(None).unwrap();
    let pixels = h.rt.capture(h.two, "before").unwrap().0;
    h.fail_resource = true;
    let mut g = h.editor.document.groups[0].clone();
    g.translation[0] += 10.0;
    h.editor
        .begin(stamp, Edit::ReplaceGroup(g.clone()))
        .unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document, accepted);
    assert_eq!(h.rt.capture(h.two, "failed").unwrap().0, pixels);
    h.fail_resource = false;
    h.editor.begin(stamp, Edit::ReplaceGroup(g)).unwrap();
    h.settle().unwrap();
    assert_ne!(h.rt.capture(h.two, "recovered").unwrap().0, pixels);
}
#[test]
fn host_real_tree_confirm_cancel_and_high_water() {
    let mut h = Host::new(scene()).unwrap();
    h.frame(None).unwrap();
    let b = h.widget_bounds("node-10").unwrap();
    h.click([810.0 + b.x + 20.0, b.y + b.height * 0.5]);
    h.frame(None).unwrap();
    assert_eq!(h.editor.selection, Some(10));
    let before = h.editor.document.clone();
    cmd(&mut h, "delete-request", json!(10)).unwrap();
    assert_eq!(h.ui().delete_confirmation, Some(10));
    cmd(&mut h, "delete-cancel", Value::Null).unwrap();
    assert_eq!(h.editor.document, before);
    assert!(cmd(&mut h, "delete-confirm", json!(10)).is_err());
    cmd(&mut h, "delete-request", json!(10)).unwrap();
    cmd(&mut h, "delete-confirm", json!(10)).unwrap();
    assert!(!h.editor.document.contains(1));
    cmd(&mut h, "undo", Value::Null).unwrap();
    assert_eq!(h.editor.document, before);
    cmd(&mut h, "create-group2", json!("2d")).unwrap();
    assert!(h.editor.document.contains(14));
    cmd(&mut h, "undo", Value::Null).unwrap();
    cmd(&mut h, "create-group2", json!("2d")).unwrap();
    assert!(h.editor.document.contains(15));
    let mut max = Document::empty("max".into());
    max.groups.push(Group::new(u64::MAX, Dimension::Three));
    let mut h = Host::new(max).unwrap();
    cmd(&mut h, "select", json!(u64::MAX)).unwrap();
    assert!(cmd(&mut h, "create-group3", json!("3d"))
        .unwrap_err()
        .contains("exhausted"));
    let mut g = h.editor.document.groups[0].clone();
    g.name = "still editable".into();
    h.editor
        .begin(h.editor.stamp, Edit::ReplaceGroup(g))
        .unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document.groups[0].id, u64::MAX);
}
#[test]
fn v1_raw_bytes_conflict_v2_storage_and_new_process_reopen() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("research").join(format!(
        "hierarchy-roundtrip-{}.agscene",
        std::process::id()
    ));
    let old = include_bytes!("../samples/authoring/example.agscene");
    std::fs::write(&path, old).unwrap();
    let (doc, mut storage) = Storage::load(&path).unwrap();
    let mut e = Editor::new(doc).unwrap();
    assert!(!e.dirty());
    assert_eq!(std::fs::read(&path).unwrap(), old);
    std::fs::write(&path, [old.as_slice(), b" "].concat()).unwrap();
    assert!(storage
        .save(&mut e, &path, Fault::None)
        .unwrap_err()
        .contains("external file modification"));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        [old.as_slice(), b" "].concat()
    );
    std::fs::write(&path, old).unwrap();
    assert_eq!(
        storage.save(&mut e, &path, Fault::None).unwrap(),
        Saved::Confirmed
    );
    assert!(String::from_utf8(std::fs::read(&path).unwrap())
        .unwrap()
        .contains("autographics.scene/5"));
    let mut h = Host::new(scene()).unwrap();
    h.frame(None).unwrap();
    h.storage.save(&mut h.editor, &path, Fault::None).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_autographics-authoring"))
        .args(["--inspect", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["hash"], h.editor.document.hash().unwrap());
    assert_eq!(
        v["document"],
        serde_json::to_value(&h.editor.document).unwrap()
    );
    assert_eq!(
        v["world"],
        serde_json::to_value(&h.editor.preview.objects).unwrap()
    );
    assert_eq!(v["dirty"], false);
    assert_eq!(v["history"], json!([0, 0, 0]));
    assert!(v["selection"].is_null());
    assert_eq!(v["two"], h.rt.capture(h.two, "same").unwrap().0);
    assert_eq!(v["three"], h.rt.capture(h.three, "same").unwrap().0);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn three_dimensional_group_axes_and_hidden_empty_clipped_groups() {
    use glam::{Mat4, Vec3};
    let mut doc = scene();
    doc.objects[2].yaw = 0.35;
    doc.objects[2].scale = [1.0, 2.0, 0.5];
    let mut h = Host::new(doc).unwrap();
    h.frame(None).unwrap();
    cmd(&mut h, "select", json!(12)).unwrap();
    for axis in 0..3 {
        let before = h.editor.document.clone();
        let world = h.editor.preview.nodes[&12].translation;
        let mut origin = Object::new(12, Kind::Cylinder);
        origin.translation = world;
        let (_, a, d) = interaction::axes(&origin, h.camera())
            .into_iter()
            .find(|x| x.0 == axis)
            .unwrap();
        let start = a + d.normalize() * 40.0;
        let end = start + d * 0.4;
        h.pointer_down([start.x, start.y + 400.0]);
        assert!(h.gesture_active());
        h.pointer_up([end.x, end.y + 400.0]);
        h.settle().unwrap();
        let changed = h
            .editor
            .document
            .groups
            .iter()
            .find(|g| g.id == 12)
            .unwrap();
        assert!(
            (changed.translation[axis] - before.groups[2].translation[axis] - 0.4).abs() < 1e-3
        );
        assert_eq!(h.editor.document.objects, before.objects);
        assert_eq!(h.editor.document.groups[3], before.groups[3]);
        let w = h.editor.preview.objects[2].translation;
        let old = hierarchy::derive(&before).unwrap()[&3].translation;
        for i in 0..3 {
            assert!((w[i] - old[i] - if i == axis { 0.4 } else { 0.0 }).abs() < 1e-3);
        }
        let projected = interaction::project(h.camera(), Vec3::from(w)).unwrap();
        assert!(projected.is_finite());
        let expected = Mat4::from_translation(Vec3::from(w))
            * Mat4::from_rotation_y(0.35)
            * Mat4::from_scale(Vec3::new(1.0, 2.0, 0.5));
        let Some(agp::host::surface_runtime::SurfaceContent::Scene3d(rendered)) =
            h.rt.content(h.three)
        else {
            panic!("actual scene consumer");
        };
        assert!(
            rendered.instances[..3].iter().all(|i| i.model == expected),
            "all original player meshes consume Auto world with local rotation/scale"
        );
        cmd(&mut h, "undo", Value::Null).unwrap();
        assert_eq!(h.editor.document, before);
    }
    cmd(&mut h, "create-group3", json!("3d")).unwrap();
    let empty = h.editor.document.groups.last().unwrap().id;
    cmd(&mut h, "select", json!(empty)).unwrap();
    assert!(h
        .ui()
        .group_handle_reason
        .as_ref()
        .unwrap()
        .contains("No visible descendants"));
    let origin = interaction::project(
        h.camera(),
        Vec3::from(h.editor.preview.nodes[&empty].translation),
    )
    .unwrap();
    h.pointer_down([origin.x, origin.y + 400.0]);
    assert!(!h.gesture_active());
    h.pointer_up([origin.x, origin.y + 400.0]);
    cmd(&mut h, "select", json!(12)).unwrap();
    cmd(&mut h, "toggle-visible", json!(12)).unwrap();
    assert!(!h.editor.preview.objects[2].visible);
    let mut g = h.editor.document.groups[2].clone();
    g.visible = true;
    g.translation = [1000.0, 0.0, 0.0];
    h.editor
        .begin(h.editor.stamp, Edit::ReplaceGroup(g))
        .unwrap();
    h.settle().unwrap();
    assert!(h
        .ui()
        .group_handle_reason
        .as_ref()
        .unwrap()
        .contains("clipped"));
    // Offscreen origin is a property-only group, not a guessed 2D screen proxy.
    h.pointer_down([10.0, 410.0]);
    assert!(!h.gesture_active());
    h.pointer_up([10.0, 410.0]);
}

#[test]
fn twenty_hierarchy_rounds_bound_vm_history_mesh_and_last_owner() {
    use agp::host::surface_runtime::SurfaceContent;
    use std::sync::Arc;
    let mut h = Host::new(scene()).unwrap();
    h.frame(None).unwrap();
    let (pin, uploads) = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => {
            let i = &c.instances[0];
            (
                c.store.pin(&i.mesh.0, &i.mesh.1).unwrap(),
                c.store.uploads.snapshot(),
            )
        }
        _ => unreachable!(),
    };
    let targets = [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq);
    for cycle in 0..20 {
        cmd(&mut h, "select", json!(10)).unwrap();
        let world = h.editor.preview.nodes[&10].translation;
        let p = [400.0 + world[0], 190.0 - world[1]];
        let accepted = h.editor.document.clone();
        let owner = h.editor.preview.vm.owner_probe();
        h.pointer_down(p);
        h.pointer_move([p[0] + 10.0, p[1]]);
        for _ in 0..4 {
            h.frame(None).unwrap();
        }
        let owners = h.gesture_owner_probes();
        assert!(owners.len() <= 3);
        assert!(owners.iter().all(|p| p()));
        h.cancel_gesture();
        h.frame(None).unwrap();
        assert_eq!(h.editor.document, accepted);
        assert!(owner());
        assert!(
            owners.iter().skip(1).all(|p| !p()),
            "temporary owners released"
        );
        h.pointer_down(p);
        h.pointer_up([p[0] + 2.0, p[1] - 4.0]);
        h.settle().unwrap();
        assert!(!owner());
        let current = h.editor.document.clone();
        let history = h.editor.history();
        h.fail_resource = true;
        let mut g = current.groups[0].clone();
        g.translation[0] += 1.0;
        h.editor
            .begin(h.editor.stamp, Edit::ReplaceGroup(g.clone()))
            .unwrap();
        h.settle().unwrap();
        assert_eq!(h.editor.document, current);
        assert_eq!(h.editor.history(), history);
        h.fail_resource = false;
        h.editor
            .begin(h.editor.stamp, Edit::ReplaceGroup(g))
            .unwrap();
        h.settle().unwrap();
        cmd(&mut h, "select", json!(11)).unwrap();
        let old = h.editor.document.parent(11).unwrap();
        cmd(
            &mut h,
            "reparent",
            json!({"target":11,"expected_parent":old,"parent":if cycle%2==0{None}else{Some(10)}}),
        )
        .unwrap();
        let before = h.editor.document.clone();
        cmd(&mut h, "delete-request", json!(11)).unwrap();
        cmd(&mut h, "delete-confirm", json!(11)).unwrap();
        cmd(&mut h, "undo", Value::Null).unwrap();
        assert_eq!(h.editor.document, before);
        cmd(&mut h, "create-group2", json!("2d")).unwrap();
        let id = h.editor.document.groups.last().unwrap().id;
        cmd(&mut h, "select", json!(id)).unwrap();
        cmd(&mut h, "delete-request", json!(id)).unwrap();
        cmd(&mut h, "delete-confirm", json!(id)).unwrap();
        let scene = match h.rt.content(h.three).unwrap() {
            SurfaceContent::Scene3d(c) => c,
            _ => unreachable!(),
        };
        let i = &scene.instances[0];
        let current = scene.store.pin(&i.mesh.0, &i.mesh.1).unwrap();
        assert!(Arc::ptr_eq(&pin, &current));
        let now = scene.store.uploads.snapshot();
        assert_eq!(
            (now.buffer_rewrites, now.bytes_rewritten),
            (uploads.buffer_rewrites, uploads.bytes_rewritten)
        );
        assert_eq!(
            [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq),
            targets
        );
        assert!(h.editor.history().0 <= 64);
        assert!(h.editor.history().2 <= 16 * 1024 * 1024);
    }
    let owner = h.editor.preview.vm.owner_probe();
    let mesh = Arc::downgrade(&pin);
    let runtime = Arc::new(h.rt);
    let weakrt = Arc::downgrade(&runtime);
    let held = runtime.clone();
    h.editor.close();
    assert!(!owner());
    drop(runtime);
    assert!(
        weakrt.upgrade().is_some(),
        "retained runtime negative/accepted external control"
    );
    drop(held);
    assert!(weakrt.upgrade().is_none());
    assert!(
        mesh.upgrade().is_some(),
        "retained mesh control distinguishes last owner"
    );
    drop(pin);
    assert!(mesh.upgrade().is_none());
}
