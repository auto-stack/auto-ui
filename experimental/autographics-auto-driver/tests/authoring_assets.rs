use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    document::{Document, Kind, Object},
    preview,
};
use serde_json::{json, Value};
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
const GLB: &[u8] = include_bytes!("../research/asset-probe/two-parts.glb");

#[test]
fn reserved_save_targets_use_resolved_windows_identity_before_any_confirmation() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp();
    let (doc, registry) = scene();
    let asset = registry.values().next().unwrap().clone();
    let entry = dir.join("original.agscene");
    let mut original = Editor::with_assets(doc.clone(), registry).unwrap();
    Storage::default()
        .save(&mut original, &entry, Fault::None)
        .unwrap();
    let original_bytes = std::fs::read(&entry).unwrap();
    let image = render_saved(&entry);
    let blob = dir
        .join("assets")
        .join(format!("{}.glb", asset.identity.sha256));
    std::fs::create_dir(dir.join("assets/nested")).unwrap();
    std::fs::create_dir(dir.join("ordinary")).unwrap();
    let replacement_entry = dir.join("replacement.agscene");
    let mut replacement = Editor::new(Document::empty("replacement".into())).unwrap();
    let mut storage = Storage::default();
    storage
        .save(&mut replacement, &replacement_entry, Fault::None)
        .unwrap();
    replacement
        .begin(
            replacement.stamp,
            Edit::Create(Object::new(1, Kind::Circle)),
        )
        .unwrap();
    while !replacement.pump().unwrap() {}
    replacement.commit().unwrap();
    assert!(replacement.dirty());
    let old_path = storage.path.clone();
    let history = replacement.history();
    let stamp = replacement.stamp;
    let aliases = [
        blob.clone(),
        blob.with_extension("GLB"),
        dir.join("ASSETS").join(blob.file_name().unwrap()),
        dir.join("assets/nested/..").join(blob.file_name().unwrap()),
        dir.join("ordinary/../assets")
            .join(blob.file_name().unwrap()),
        dir.join("assets")
            .join(format!("{}.GLB", asset.identity.sha256.to_uppercase())),
        dir.join("assets")
            .join(format!("{}.glb.", asset.identity.sha256)),
    ];
    for alias in aliases {
        let error = storage
            .save(&mut replacement, &alias, Fault::None)
            .unwrap_err();
        assert!(error.contains("reserved"), "{}: {error}", alias.display());
        assert_eq!(std::fs::read(&blob).unwrap(), asset.bytes.as_ref());
        assert_eq!(std::fs::read(&entry).unwrap(), original_bytes);
        let (reopened, _, _) = Storage::load_assets(&entry, &Registry::new()).unwrap();
        assert_eq!(reopened, doc);
        assert!(replacement.dirty());
        assert_eq!(storage.path, old_path);
        assert_eq!(replacement.history(), history);
        assert_eq!(replacement.stamp, stamp);
        assert!(!storage.unverified());
    }
    assert_eq!(render_saved(&entry), image);
    // Nonexistent names and the project-root lock have the same reservation.
    for alias in [
        dir.join("ASSETS").join(format!("{}.GLB", "A".repeat(64))),
        dir.join(".AUTOGRAPHICS-ASSETS-LOCK"),
        dir.join("ordinary/../.autographics-assets-lock"),
    ] {
        assert!(storage
            .save(&mut replacement, &alias, Fault::None)
            .unwrap_err()
            .contains("reserved"));
        assert!(!alias.exists());
        assert!(replacement.dirty());
        assert_eq!(storage.path, old_path);
    }
    // Reject before verifying an older receipt: an invalid Save As must not
    // incidentally confirm/clean a pending save or change its path/ownership.
    let pending_entry = dir.join("pending.agscene");
    assert_eq!(
        storage
            .save(&mut replacement, &pending_entry, Fault::AfterReplaceRead)
            .unwrap(),
        Saved::VerificationPending
    );
    assert!(storage
        .save(&mut replacement, &blob.with_extension("GLB"), Fault::None)
        .is_err());
    assert!(replacement.dirty() && storage.unverified());
    assert_eq!(storage.path, old_path);
    assert!(storage.verify(&mut replacement).unwrap());
    // Ordinary case/.. aliases remain supported and share conflict identity.
    let good = dir.join("ordinary/../NEW.agscene");
    assert_eq!(
        storage.save(&mut replacement, &good, Fault::None).unwrap(),
        Saved::Confirmed
    );
    assert!(!replacement.dirty());
    std::fs::write(dir.join("NEW.agscene"), b"external edit").unwrap();
    assert!(storage
        .save(&mut replacement, &dir.join("new.agscene"), Fault::None)
        .unwrap_err()
        .contains("external file modification"));
    assert_eq!(
        std::fs::read(dir.join("NEW.agscene")).unwrap(),
        b"external edit"
    );
    drop(storage);
    remove_repair_test_dir(&dir);
}

fn admission_holdouts() -> Vec<(String, Vec<u8>)> {
    let mut cases = vec![];
    for (semantic, index) in [("POSITION", 0), ("NORMAL", 1)] {
        for component in [5120, 5121, 5122, 5123, 5125] {
            cases.push((
                format!("{semantic}-{component}"),
                mutation(|j| j["accessors"][index]["componentType"] = json!(component)),
            ));
        }
        for dimension in ["SCALAR", "VEC2", "VEC4", "MAT2", "MAT3", "MAT4"] {
            cases.push((
                format!("{semantic}-{dimension}"),
                mutation(|j| j["accessors"][index]["type"] = json!(dimension)),
            ));
        }
        cases.push((
            format!("{semantic}-normalized"),
            mutation(|j| j["accessors"][index]["normalized"] = json!(true)),
        ));
        let (mut j, b) = chunks(GLB);
        j["accessors"][index]
            .as_object_mut()
            .unwrap()
            .remove("bufferView");
        j["accessors"][index]["sparse"] = json!({"count":3,"indices":{"bufferView":2,"componentType":5123},"values":{"bufferView":index}});
        j["accessors"][index]["componentType"] = json!(5125);
        cases.push((format!("{semantic}-sparse-wrong-type"), pack(&j, &b)));
    }
    for field in ["componentType", "type", "normalized"] {
        let value = match field {
            "componentType" => json!(5126),
            "type" => json!("VEC3"),
            _ => json!(true),
        };
        cases.push((
            format!("indices-{field}"),
            mutation(|j| j["accessors"][2][field] = value),
        ));
    }
    // All declarations matter, even when not reachable from the scene.
    cases.push((
        "unreferenced-mesh-normal".into(),
        mutation(|j| {
            let mesh = j["meshes"][0].clone();
            let mut accessor = j["accessors"][1].clone();
            accessor["componentType"] = json!(5121);
            j["accessors"].as_array_mut().unwrap().push(accessor);
            j["meshes"].as_array_mut().unwrap().push(mesh);
            j["meshes"][2]["primitives"][0]["attributes"]["NORMAL"] = json!(6);
        }),
    ));
    for value in [json!([1.0]), json!([]), Value::Null] {
        for unused in [false, true] {
            let value = value.clone();
            cases.push((
                format!("mesh-weights-{value}-unused{unused}"),
                mutation(|j| {
                    if unused {
                        let m = j["meshes"][0].clone();
                        j["meshes"].as_array_mut().unwrap().push(m);
                    }
                    j["meshes"][if unused { 2 } else { 0 }]["weights"] = value;
                }),
            ));
        }
    }
    cases.push((
        "node-weights-empty".into(),
        mutation(|j| j["nodes"][0]["weights"] = json!([])),
    ));
    cases.push((
        "primitive-targets-empty".into(),
        mutation(|j| j["meshes"][0]["primitives"][0]["targets"] = json!([])),
    ));
    for (field, value) in [
        ("translation", json!([20, 0, 0])),
        ("rotation", json!([0, 0, 0, 1])),
        ("scale", json!([1, 1, 1])),
    ] {
        for unused in [false, true] {
            let value = value.clone();
            cases.push((
                format!("matrix-{field}-unused{unused}"),
                mutation(|j| {
                    let n = if unused {
                        j["nodes"].as_array_mut().unwrap().push(json!({}));
                        2
                    } else {
                        0
                    };
                    j["nodes"][n] = json!({"matrix":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"mesh":0});
                    j["nodes"][n][field] = value;
                }),
            ));
        }
    }
    cases
}

#[test]
fn typed_accessors_and_declared_morph_transforms_reject_before_readers_and_host_publication() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp();
    let path = dir.join("input.glb");
    std::fs::write(&path, GLB).unwrap();
    let mut host = Host::new(Document::empty("strict-admission".into())).unwrap();
    cmd(&mut host, "import", json!(path.to_str())).unwrap();
    host.frame(None).unwrap();
    pixels(&mut host, 0.0, true);
    let doc = host.editor.document.clone();
    let history = host.editor.history();
    let stamp = host.editor.stamp;
    let key = doc.assets[0].key();
    let owner = host.editor.assets[&key].clone();
    let image = host.rt.capture(host.three, "strict-old-content").unwrap().0;
    let writes = host.resource_snapshot()["mesh_uploads"].clone();
    let mut cases = admission_holdouts();
    for (component, normalized, dim) in [
        (5120, false, "VEC2"),
        (5125, false, "VEC2"),
        (5126, true, "VEC2"),
        (5121, false, "VEC2"),
        (5123, false, "VEC2"),
        (5126, false, "VEC3"),
    ] {
        let (mut j, b) = chunks(GLB);
        j["accessors"].as_array_mut().unwrap().push(json!({"bufferView":0,"componentType":component,"count":3,"type":dim,"normalized":normalized}));
        j["meshes"][0]["primitives"][0]["attributes"]["TEXCOORD_0"] = json!(6);
        cases.push((format!("uv-{component}-{normalized}-{dim}"), pack(&j, &b)));
    }
    for (name, bytes) in &cases {
        let calls = assets::decode_attempts();
        let error = assets::decode(bytes.clone(), None).unwrap_err();
        assert!(
            error.contains("invalid:") || error.contains("unsupported:"),
            "{name}: {error}"
        );
        assert_eq!(assets::decode_attempts(), calls, "{name}");
        std::fs::write(&path, bytes).unwrap();
        assert!(
            cmd(&mut host, "import", json!(path.to_str())).is_err(),
            "{name}"
        );
        assert_eq!(assets::decode_attempts(), calls, "{name}");
        assert_eq!(host.editor.document, doc, "{name}");
        assert_eq!(host.editor.history(), history, "{name}");
        assert_eq!(host.editor.stamp, stamp, "{name}");
        assert!(Arc::ptr_eq(&owner, &host.editor.assets[&key]), "{name}");
        assert_eq!(host.resource_snapshot()["mesh_uploads"], writes, "{name}");
    }
    host.frame(None).unwrap();
    assert_eq!(
        host.rt.capture(host.three, "strict-old-content").unwrap().0,
        image
    );
    pixels(&mut host, 0.0, true);
    std::fs::write(&path, GLB).unwrap();
    cmd(&mut host, "import", json!(path.to_str())).unwrap();
    assert_eq!(host.editor.document.objects.len(), 2);
    assert_eq!(host.editor.document.assets.len(), 1);
    assert_eq!(host.resource_snapshot()["mesh_uploads"], writes);
    println!("{} invalid declared inputs rejected directly and through Host without panic/decode/upload/publication; valid retry passed", cases.len());
    drop(host);
    remove_repair_test_dir(&dir);
}

#[test]
fn interleaved_sparse_uv_and_single_transform_controls_preserve_original_geometry_and_bytes() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let baseline = assets::decode(GLB.to_vec(), None).unwrap();
    let mut valid = vec![];
    let (mut j, mut b) = chunks(GLB);
    let at = b.len();
    for i in 0..3 {
        b.extend_from_slice(&glb_bin_position(i));
        b.extend_from_slice(
            &[0f32, 0., 1.]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
    }
    let view = j["bufferViews"].as_array().unwrap().len();
    j["bufferViews"]
        .as_array_mut()
        .unwrap()
        .push(json!({"buffer":0,"byteOffset":at,"byteLength":72,"byteStride":24}));
    j["accessors"][0]["bufferView"] = json!(view);
    j["accessors"][0]["byteOffset"] = json!(0);
    j["accessors"][1]["bufferView"] = json!(view);
    j["accessors"][1]["byteOffset"] = json!(12);
    j["buffers"][0]["byteLength"] = json!(b.len());
    valid.push(("interleaved", pack(&j, &b)));
    for index in [0, 1, 2] {
        let (mut j, b) = chunks(GLB);
        j["accessors"][index]
            .as_object_mut()
            .unwrap()
            .remove("bufferView");
        j["accessors"][index]["sparse"] = json!({"count":3,"indices":{"bufferView":2,"componentType":5123},"values":{"bufferView":index}});
        valid.push(("sparse", pack(&j, &b)));
    }
    for (component, normalized, size) in [(5126, false, 4), (5121, true, 1), (5123, true, 2)] {
        let (mut j, mut b) = chunks(GLB);
        let at = b.len();
        b.resize(at + 6 * size, 0);
        while b.len() % 4 != 0 {
            b.push(0);
        }
        let view = j["bufferViews"].as_array().unwrap().len();
        j["bufferViews"]
            .as_array_mut()
            .unwrap()
            .push(json!({"buffer":0,"byteOffset":at,"byteLength":6*size}));
        j["accessors"].as_array_mut().unwrap().push(json!({"bufferView":view,"componentType":component,"count":3,"type":"VEC2","normalized":normalized}));
        j["meshes"][0]["primitives"][0]["attributes"]["TEXCOORD_0"] = json!(6);
        j["buffers"][0]["byteLength"] = json!(b.len());
        valid.push(("uv", pack(&j, &b)));
    }
    // Equivalent matrix-only and TRS-only transforms must produce all original
    // baked positions and normals; admission does not rewrite persisted bytes.
    valid.push((
        "matrix-only",
        mutation(|j| {
            j["nodes"][0] = json!({"mesh":0,"matrix":[1,0,0,0,0,1,0,0,0,0,1,0,-1.5,0,0,1]});
        }),
    ));
    valid.push((
        "TRS-only",
        mutation(|j| {
            j["nodes"][0]["rotation"] = json!([0, 0, 0, 1]);
            j["nodes"][0]["scale"] = json!([1, 1, 1]);
        }),
    ));
    for (name, bytes) in valid {
        let a = assets::decode(bytes.clone(), None).unwrap();
        assert_eq!(a.bytes.as_ref(), bytes.as_slice(), "{name}");
        assert_eq!(
            a.identity.sha256,
            agp::renderer::capture::sha256_hex(&bytes),
            "{name}"
        );
        for (actual, expected) in a.model.meshes.iter().zip(&baseline.model.meshes) {
            assert_eq!(actual.positions, expected.positions, "{name}");
            assert_eq!(actual.normals, expected.normals, "{name}");
            assert_eq!(actual.indices, expected.indices, "{name}");
        }
    }
}

fn glb_bin_position(index: usize) -> Vec<u8> {
    let (_, b) = chunks(GLB);
    b[index * 12..index * 12 + 12].to_vec()
}

fn remove_repair_test_dir(dir: &std::path::Path) {
    let resolved = dir.canonicalize().unwrap();
    let root = std::env::temp_dir().canonicalize().unwrap();
    assert!(resolved.starts_with(&root) && resolved != root);
    std::fs::remove_dir_all(resolved).unwrap();
}
#[test]
fn unverified_save_snapshot_is_in_active_budget_after_redo_branch_is_discarded() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp();
    let source = dir.join("input.glb");
    std::fs::write(&source, GLB).unwrap();
    let mut h = Host::new(Document::empty("snapshot-owner".into())).unwrap();
    cmd(&mut h, "import", json!(source.to_str())).unwrap();
    let identity = h.editor.document.assets[0].clone();
    let weak = Arc::downgrade(&h.editor.assets[&identity.key()]);
    let gpu = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => {
            Arc::downgrade(&c.store.pin(&identity.sha256, "primitive-0").unwrap())
        }
        _ => unreachable!(),
    };
    let path = dir.join("scene.agscene");
    assert_eq!(
        h.storage
            .save(&mut h.editor, &path, Fault::AfterReplaceRead)
            .unwrap(),
        Saved::VerificationPending
    );
    cmd(&mut h, "undo", Value::Null).unwrap();
    cmd(&mut h, "create", json!("circle")).unwrap();
    assert!(h.editor.assets.is_empty());
    assert!(weak.upgrade().is_some() && gpu.upgrade().is_some());
    assert_eq!(h.resource_snapshot()["asset_union"]["count"], 1);
    for i in 0..7 {
        std::fs::write(
            &source,
            mutation(|j| j["asset"]["generator"] = json!(format!("budget-{i}"))),
        )
        .unwrap();
        cmd(&mut h, "import", json!(source.to_str())).unwrap();
    }
    assert_eq!(h.editor.assets.len(), 7);
    assert_eq!(h.resource_snapshot()["asset_union"]["count"], 8);
    let before = h.editor.document.clone();
    std::fs::write(
        &source,
        mutation(|j| j["asset"]["generator"] = json!("ninth")),
    )
    .unwrap();
    assert!(cmd(&mut h, "import", json!(source.to_str()))
        .unwrap_err()
        .contains("resource limit"));
    assert_eq!(h.editor.document, before);
    assert!(h.storage.verify(&mut h.editor).unwrap());
    h.frame(None).unwrap();
    assert!(weak.upgrade().is_none() && gpu.upgrade().is_none());
    assert!(h.editor.dirty());
    assert_eq!(h.resource_snapshot()["asset_union"]["count"], 7);
    cmd(&mut h, "import", json!(source.to_str())).unwrap();
    assert_eq!(h.editor.assets.len(), 8);
    let blob = dir.join("assets").join(format!("{}.glb", identity.sha256));
    let old = std::fs::read(&blob).unwrap();
    assert!(h
        .storage
        .save(&mut h.editor, &blob, Fault::None)
        .unwrap_err()
        .contains("reserved"));
    assert_eq!(std::fs::read(&blob).unwrap(), old);
    drop(h);
    std::fs::remove_file(blob).unwrap();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(source).unwrap();
    std::fs::remove_dir(dir.join("assets")).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
#[test]
fn sparse_and_u8_controls_validate_actual_storage_before_decoder() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut j, bin) = chunks(GLB);
    j["accessors"][0]
        .as_object_mut()
        .unwrap()
        .remove("bufferView");
    j["accessors"][0]["sparse"] = json!({"count":3,"indices":{"bufferView":2,"componentType":5123},"values":{"bufferView":0}});
    let good = assets::decode(pack(&j, &bin), None).unwrap();
    assert_eq!(
        good.model.meshes[0].positions,
        assets::decode(GLB.to_vec(), None).unwrap().model.meshes[0].positions
    );
    let mut bad = bin.clone();
    let at = j["bufferViews"][2]["byteOffset"].as_u64().unwrap() as usize;
    bad[at + 4..at + 6].copy_from_slice(&0u16.to_le_bytes());
    let calls = assets::decode_attempts();
    assert!(assets::decode(pack(&j, &bad), None)
        .unwrap_err()
        .contains("sparse indices"));
    assert_eq!(assets::decode_attempts(), calls);
    let (mut j, mut bin) = chunks(GLB);
    j["accessors"][2]["componentType"] = json!(5121);
    j["bufferViews"][2]["byteLength"] = json!(3);
    let at = j["bufferViews"][2]["byteOffset"].as_u64().unwrap() as usize;
    bin[at..at + 3].copy_from_slice(&[0, 1, 2]);
    assert_eq!(
        assets::decode(pack(&j, &bin), None).unwrap().model.meshes[0].indices,
        [0, 1, 2]
    );
}
fn chunks(b: &[u8]) -> (Value, Vec<u8>) {
    let n = u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize;
    (
        serde_json::from_slice(&b[20..20 + n]).unwrap(),
        b[28 + n..].to_vec(),
    )
}
fn pack(j: &Value, bin: &[u8]) -> Vec<u8> {
    let mut j = serde_json::to_vec(j).unwrap();
    while j.len() % 4 != 0 {
        j.push(b' ');
    }
    let mut b = vec![];
    for v in [
        0x46546c67,
        2,
        (28 + j.len() + bin.len()) as u32,
        j.len() as u32,
        0x4e4f534a,
    ] {
        b.extend(v.to_le_bytes());
    }
    b.extend(j);
    b.extend((bin.len() as u32).to_le_bytes());
    b.extend(0x004e4942u32.to_le_bytes());
    b.extend(bin);
    b
}
fn mutation(f: impl FnOnce(&mut Value)) -> Vec<u8> {
    let (mut j, b) = chunks(GLB);
    f(&mut j);
    pack(&j, &b)
}
fn scene() -> (Document, Registry) {
    let a = assets::decode(GLB.to_vec(), Some("two parts".into())).unwrap();
    let mut r = Registry::new();
    assets::insert(&mut r, a.clone()).unwrap();
    let mut d = Document::empty("asset-scene".into());
    d.assets.push(a.identity.clone());
    let mut o = Object::new(1, Kind::Model);
    o.resource = Some(a.identity.key());
    d.objects.push(o);
    (d, r)
}
#[test]
fn supported_glb_all_parts_ordinals_defaults_and_bounded_identity() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let implicit = mutation(|j| {
        j.as_object_mut().unwrap().remove("scene");
    });
    let implicit_sha = agp::renderer::capture::sha256_hex(&implicit);
    let imported = assets::decode(implicit.clone(), None).unwrap();
    assert_eq!(imported.identity.sha256, implicit_sha);
    assert_eq!(imported.model.source_sha256, implicit_sha);
    assert_eq!(imported.bytes.as_ref(), implicit);
    assert_eq!(
        imported.model.meshes,
        assets::decode(GLB.to_vec(), None).unwrap().model.meshes
    );
    let mut registry = Registry::new();
    assets::insert(&mut registry, imported.clone()).unwrap();
    let mut doc = Document::empty("implicit-scene".into());
    doc.assets.push(imported.identity.clone());
    let mut leaf = Object::new(1, Kind::Model);
    leaf.resource = Some(imported.identity.key());
    doc.objects.push(leaf);
    let mut editor = Editor::with_assets(doc.clone(), registry).unwrap();
    let dir = temp();
    let entry = dir.join("scene.agscene");
    Storage::default()
        .save(&mut editor, &entry, Fault::None)
        .unwrap();
    let (reopened, loaded, _) = Storage::load_assets(&entry, &Registry::new()).unwrap();
    assert_eq!(reopened, doc);
    assert_eq!(loaded[&imported.identity.key()].bytes.as_ref(), implicit);
    assert_eq!(
        loaded[&imported.identity.key()].model.meshes,
        imported.model.meshes
    );
    std::fs::remove_file(dir.join("assets").join(format!("{}.glb", implicit_sha))).unwrap();
    std::fs::remove_file(entry).unwrap();
    std::fs::remove_dir(dir.join("assets")).unwrap();
    std::fs::remove_dir(dir).unwrap();
    let a = assets::decode(GLB.to_vec(), None).unwrap();
    assert_eq!(a.model.meshes.len(), 2);
    assert_eq!(a.model.meshes[0].name, "primitive-0");
    assert_eq!(a.model.meshes[1].name, "primitive-1");
    assert!(a.model.meshes[0].positions[0][0] < -2.0);
    assert!(a.model.meshes[1].positions[0][0] > 0.0);
    assert_ne!(
        a.model.meshes[0].material.base_color,
        a.model.meshes[1].material.base_color
    );
    let b = mutation(|j| {
        j.as_object_mut().unwrap().remove("materials");
        for m in j["meshes"].as_array_mut().unwrap() {
            m["primitives"][0]
                .as_object_mut()
                .unwrap()
                .remove("material");
        }
    });
    let a = assets::decode(b, None).unwrap();
    assert!(a
        .model
        .meshes
        .iter()
        .all(|m| m.material.base_color == [1.0; 4]
            && m.material.metallic == 0.0
            && m.material.roughness == 1.0));
    let repeated = mutation(|j| {
        j["nodes"][1]["mesh"] = json!(0);
    });
    let a = assets::decode(repeated, None).unwrap();
    assert_eq!(a.model.meshes.len(), 2);
    assert_ne!(a.model.meshes[0].name, a.model.meshes[1].name);
    assert_ne!(a.model.meshes[0].positions, a.model.meshes[1].positions);
}
#[test]
fn unsupported_rejects_before_io_decoder_and_recursion() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let cases: Vec<(&str, Box<dyn Fn(&mut Value)>)> = vec![
        (
            "URI",
            Box::new(|j| j["buffers"][0]["uri"] = json!("file:///must-not-read")),
        ),
        (
            "images",
            Box::new(|j| j["images"] = json!([{"uri":"data:image/png;base64,INVALID"}])),
        ),
        ("textures", Box::new(|j| j["textures"] = json!([{}]))),
        ("animations", Box::new(|j| j["animations"] = json!([{}]))),
        ("skins", Box::new(|j| j["skins"] = json!([{}]))),
        (
            "extensions",
            Box::new(|j| j["extensionsUsed"] = json!(["KHR_draco_mesh_compression"])),
        ),
        (
            "material",
            Box::new(|j| j["materials"][0]["doubleSided"] = json!(true)),
        ),
        (
            "material",
            Box::new(|j| j["materials"][0]["alphaMode"] = json!("BLEND")),
        ),
        (
            "material",
            Box::new(|j| j["materials"][0]["pbrMetallicRoughness"]["metallicFactor"] = json!(0.5)),
        ),
        (
            "material",
            Box::new(|j| j["materials"][0]["pbrMetallicRoughness"]["roughnessFactor"] = json!(0.5)),
        ),
        (
            "material",
            Box::new(|j| j["materials"][0]["emissiveFactor"] = json!([1, 0, 0])),
        ),
        (
            "material",
            Box::new(|j| {
                j["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"] = json!([1, 0, 0, 0.5])
            }),
        ),
        (
            "semantic",
            Box::new(|j| j["meshes"][0]["primitives"][0]["attributes"]["COLOR_0"] = json!(0)),
        ),
        (
            "triangles",
            Box::new(|j| j["meshes"][0]["primitives"][0]["mode"] = json!(1)),
        ),
        (
            "transform",
            Box::new(|j| j["nodes"][0]["scale"] = json!([-1, 1, 1])),
        ),
        (
            "transform",
            Box::new(|j| j["nodes"][0]["scale"] = json!([0, 1, 1])),
        ),
        (
            "cycle",
            Box::new(|j| {
                j["nodes"][0]["children"] = json!([1]);
                j["nodes"][1]["children"] = json!([0]);
            }),
        ),
    ];
    for (label, f) in cases {
        let b = mutation(f);
        let e = assets::decode(b, None).unwrap_err();
        assert!(e.contains(label), "{label}: {e}");
        assert!(!e.contains("decoder"), "must fail before import_slice: {e}");
    }
    let b = mutation(|j| j["bufferViews"][0]["byteLength"] = json!(1000000));
    assert!(assets::decode(b, None).unwrap_err().contains("span"));
    let mut b = GLB.to_vec();
    b.extend([0; 4]);
    assert!(assets::decode(b, None)
        .unwrap_err()
        .contains("header/length"));
}
#[test]
fn geometry_limits_sparse_bad_indices_and_raw_snapshot_budgets() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (j, mut b) = chunks(GLB);
    let at = j["bufferViews"][2]["byteOffset"].as_u64().unwrap() as usize;
    b[at..at + 2].copy_from_slice(&999u16.to_le_bytes());
    assert!(assets::decode(pack(&j, &b), None)
        .unwrap_err()
        .contains("index range"));
    let mut big = vec![0u8; assets::MAX_RAW + 1];
    big[..GLB.len()].copy_from_slice(GLB);
    assert!(assets::decode(big, None).unwrap_err().contains("8MiB"));
    let b = mutation(|j| j["nodes"] = Value::Array((0..129).map(|_| json!({})).collect()));
    assert!(assets::decode(b, None).unwrap_err().contains("nodes 128"));
    let b = mutation(|j| {
        j["nodes"] = Value::Array((0..17).map(|_| json!({"mesh":0})).collect());
        j["scenes"][0]["nodes"] = Value::Array((0..17).map(|i| json!(i)).collect());
    });
    assert!(assets::decode(b, None)
        .unwrap_err()
        .contains("expanded geometry"));
    let mut r = Registry::new();
    for i in 0..8 {
        let b = mutation(|j| j["asset"]["generator"] = json!(format!("own-{i}")));
        assets::insert(&mut r, assets::decode(b, None).unwrap()).unwrap();
    }
    let a = assets::decode(GLB.to_vec(), None).unwrap();
    assert!(assets::insert(&mut r, a)
        .unwrap_err()
        .contains("active asset union"));
    assert_eq!(r.len(), 8);
}
#[test]
fn schema3_strict_legacy_mapping_resource_identity_and_actual_auto() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (d, _) = scene();
    assert_eq!(Document::decode(&d.encode().unwrap()).unwrap(), d);
    let p = preview::build(&d).unwrap();
    assert_eq!(p.objects[0].resource, d.objects[0].resource);
    assert_eq!(p.objects[0].kind, Kind::Model);
    for key in ["assets", "resource", "sha256", "format", "byte_length"] {
        let mut v = serde_json::to_value(&d).unwrap();
        match key {
            "assets" => v["assets"] = json!([]),
            "resource" => v["objects"][0]["resource"] = json!("sha256:bad"),
            _ => v["assets"][0][key] = Value::Null,
        };
        assert!(
            Document::decode(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{key}"
        );
    }
    let mut v = serde_json::to_value(&d).unwrap();
    let duplicate = v["assets"][0].clone();
    v["assets"].as_array_mut().unwrap().push(duplicate);
    assert!(Document::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    for bytes in [
        include_bytes!("../samples/authoring/example.agscene").as_slice(),
        include_bytes!("../samples/authoring_hierarchy/example.agscene").as_slice(),
    ] {
        let old: Value = serde_json::from_slice(bytes).unwrap();
        let d = Document::decode(bytes).unwrap();
        assert_eq!(d.schema, "autographics.scene/5");
        assert!(d.assets.is_empty());
        assert_eq!(old["objects"].as_array().unwrap().len(), d.objects.len());
        assert!(d.objects.iter().all(|o| o.resource.is_none()));
    }
    let source = preview::ASSET_SOURCE.replace("return resources[i]", "return 0");
    let mut build = preview::Building::new(&d, &source).unwrap();
    while !build.pump().unwrap() {}
    assert!(build.finish().err().unwrap().contains("wrong-resource"));
}
fn large_primitive(vertices: usize, last: u32, nodes: usize) -> Vec<u8> {
    let mut bin = vec![];
    for i in 0..vertices {
        for f in [i as f32, 0.0, 0.0] {
            bin.extend(f.to_le_bytes());
        }
    }
    for _ in 0..vertices {
        for f in [0.0f32, 0.0, 1.0] {
            bin.extend(f.to_le_bytes());
        }
    }
    for i in [0u32, 1, last] {
        bin.extend(i.to_le_bytes());
    }
    let j = json!({"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":(0..nodes).collect::<Vec<_>>()}],"nodes":(0..nodes).map(|i|json!({"mesh":0,"translation":[0,i,0]})).collect::<Vec<_>>(),"meshes":[{"primitives":[{"attributes":{"POSITION":0,"NORMAL":1},"indices":2}]}],"buffers":[{"byteLength":bin.len()}],"bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":vertices*12},{"buffer":0,"byteOffset":vertices*12,"byteLength":vertices*12},{"buffer":0,"byteOffset":vertices*24,"byteLength":12}],"accessors":[{"bufferView":0,"componentType":5126,"count":vertices,"type":"VEC3","min":[0,0,0],"max":[vertices,0,0]},{"bufferView":1,"componentType":5126,"count":vertices,"type":"VEC3"},{"bufferView":2,"componentType":5125,"count":3,"type":"SCALAR"}]});
    pack(&j, &bin)
}
#[test]
fn exact_u16_vertex_expansion_primitive_and_node_boundaries() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let b = large_primitive(65536, 65535, 4);
    let a = assets::decode(b, None).unwrap();
    assert_eq!(a.model.meshes.len(), 4);
    assert_eq!(
        a.model
            .meshes
            .iter()
            .map(|m| m.positions.len())
            .sum::<usize>(),
        262144
    );
    assert_eq!(a.model.meshes[0].indices[2], 65535);
    assert!(assets::decode(large_primitive(65536, 65536, 1), None)
        .unwrap_err()
        .contains("index range"));
    assert!(assets::decode(large_primitive(65537, 65536, 1), None)
        .unwrap_err()
        .contains("vertices"));
    assert!(assets::decode(large_primitive(65536, 65535, 5), None)
        .unwrap_err()
        .contains("expanded"));
    let b = mutation(|j| {
        j["nodes"] = Value::Array((0..16).map(|_| json!({"mesh":0})).collect());
        j["scenes"][0]["nodes"] = json!((0..16).collect::<Vec<_>>());
    });
    assert_eq!(assets::decode(b, None).unwrap().model.meshes.len(), 16);
    let b = mutation(|j| {
        j["nodes"]
            .as_array_mut()
            .unwrap()
            .extend((2..128).map(|_| json!({})));
    });
    assets::decode(b, None).unwrap();
    let b = mutation(|j| {
        for m in j["meshes"].as_array_mut().unwrap() {
            m["primitives"][0]
                .as_object_mut()
                .unwrap()
                .remove("indices");
        }
    });
    assert!(assets::decode(b, None)
        .unwrap()
        .model
        .meshes
        .iter()
        .all(|m| m.indices == [0, 1, 2]));
}
use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::authoring::{
    host::Host,
    interaction,
    session::{Edit, Editor},
    storage::{Fault, Saved, Storage},
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
static NEXT: AtomicUsize = AtomicUsize::new(1);
fn temp() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ag838-test-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&p).unwrap();
    p
}
fn cmd(h: &mut Host, action: &str, payload: Value) -> Result<(), String> {
    h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":action,"payload":payload,"selection":h.editor.selection,"asset_generation":h.ui().asset_token}).to_string())?;
    h.settle()?;
    if !h.ui().error.is_empty() {
        return Err(h.ui().error.clone());
    }
    Ok(())
}
fn render_saved(path: &std::path::Path) -> String {
    let (d, r, _) = Storage::load_assets(path, &Registry::new()).unwrap();
    let mut h = Host::with_assets(d, r).unwrap();
    h.frame(None).unwrap();
    h.rt.capture(h.three, "saved-runnable-control").unwrap().0
}
fn pixels(h: &mut Host, x: f32, shown: bool) {
    for (i, v) in [-1.5, 1.5].into_iter().enumerate() {
        let p = interaction::project(h.camera(), glam::Vec3::new(v + x, 0.5, 0.0)).unwrap();
        let c = h.rt.probe_pixel(h.top, [p.x, p.y + 400.0]).unwrap();
        if shown {
            assert!(c[i] > 80 && c[1 - i] < 20 && c[2] < 20, "part {i} {c:?}");
        } else {
            assert!(c[0] > 50 && c[1] > 50 && c[2] > 50, "background {c:?}");
        }
    }
}
#[test]
fn actual_host_import_history_staged_rejection_reuse_and_full_model_pick() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp();
    let path = dir.join("own.glb");
    std::fs::write(&path, GLB).unwrap();
    let mut h = Host::new(Document::empty("import".into())).unwrap();
    cmd(&mut h, "import", json!(path.to_str().unwrap())).unwrap();
    let first = h.editor.document.clone();
    let key = first.assets[0].key();
    let asset = h.editor.assets[&key].clone();
    let id = first.objects[0].id;
    assert_eq!(h.editor.history().0, 1);
    assert!(h.editor.dirty());
    pixels(&mut h, 0.0, true);
    let p = interaction::project(h.camera(), glam::Vec3::new(1.5, 0.5, 0.0)).unwrap();
    h.pointer_down([p.x, p.y + 400.0]);
    h.pointer_up([p.x, p.y + 400.0]);
    h.settle().unwrap();
    assert_eq!(h.editor.selection, Some(id));
    let (pin, uploads) = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => (
            c.store.pin(&asset.identity.sha256, "primitive-0").unwrap(),
            c.store.uploads.snapshot(),
        ),
        _ => unreachable!(),
    };
    let alias = dir.join("same-bytes-other-path.glb");
    std::fs::write(&alias, GLB).unwrap();
    cmd(&mut h, "import", json!(alias.to_str().unwrap())).unwrap();
    assert!(Arc::ptr_eq(&asset, &h.editor.assets[&key]));
    assert_eq!(h.editor.document.assets.len(), 1);
    assert_eq!(h.editor.document.objects.len(), 2);
    match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => {
            assert!(Arc::ptr_eq(
                &pin,
                &c.store.pin(&asset.identity.sha256, "primitive-0").unwrap()
            ));
            assert_eq!(
                c.store.uploads.snapshot().buffer_rewrites,
                uploads.buffer_rewrites
            );
        }
        _ => unreachable!(),
    }
    let second = mutation(|j| {
        j["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"] = json!([0, 0, 1, 1])
    });
    std::fs::write(&path, &second).unwrap();
    let before = h.editor.document.clone();
    let history = h.editor.history();
    h.fail_after_asset_upload = true;
    h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"import","selection":h.editor.selection,"asset_generation":h.ui().asset_token,"payload":path.to_str()}).to_string()).unwrap();
    let candidate = h
        .editor
        .candidate_assets()
        .values()
        .find(|a| a.identity.sha256 != asset.identity.sha256)
        .unwrap();
    let rejected = Arc::downgrade(candidate);
    h.settle().unwrap();
    assert_eq!(h.editor.document, before);
    assert_eq!(h.editor.history(), history);
    assert!(rejected.upgrade().is_none());
    match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => {
            assert_eq!(c.store.uploaded_count(), 9);
            assert_eq!(
                c.store.uploads.snapshot().buffer_rewrites,
                uploads.buffer_rewrites + 4
            );
            assert!(Arc::ptr_eq(
                &pin,
                &c.store.pin(&asset.identity.sha256, "primitive-0").unwrap()
            ));
        }
        _ => unreachable!(),
    }
    pixels(&mut h, 0.0, true);
    h.fail_after_asset_upload = false;
    cmd(&mut h, "import", json!(path.to_str().unwrap())).unwrap();
    assert_eq!(h.editor.document.assets.len(), 2);
    assert_ne!(
        h.editor.document.assets[0].sha256,
        h.editor.document.assets[1].sha256
    );
    let accepted = h.editor.document.clone();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(alias).unwrap();
    cmd(&mut h, "undo", Value::Null).unwrap();
    assert_eq!(h.editor.document, before);
    cmd(&mut h, "redo", Value::Null).unwrap();
    assert_eq!(h.editor.document, accepted);
    cmd(&mut h, "select", json!(id)).unwrap();
    let stale=json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"selection":h.editor.selection,"action":"asset-replace","payload":accepted.assets[1].key()}).to_string();
    cmd(&mut h, "select", json!(accepted.objects[1].id)).unwrap();
    assert!(h
        .dispatch(&stale)
        .unwrap_err()
        .contains("stale asset target"));
    assert_eq!(h.editor.document, accepted);
    cmd(&mut h, "select", json!(id)).unwrap();
    cmd(&mut h, "asset-replace", json!(accepted.assets[1].key())).unwrap();
    assert_eq!(h.editor.document.objects[0].id, id);
    assert_eq!(
        h.editor.document.objects[0].translation,
        first.objects[0].translation
    );
    assert_eq!(h.editor.document.objects[0].parent, first.objects[0].parent);
    std::fs::remove_dir(dir).unwrap();
}
#[test]
fn potential512_and_hidden513_reject_before_upload_or_publication() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let b = mutation(|j| {
        j["nodes"] = Value::Array(
            (0..16)
                .map(|i| json!({"mesh":0,"translation":[if i==15{0.0}else{1000.0},0,0]}))
                .collect(),
        );
        j["scenes"][0]["nodes"] = json!((0..16).collect::<Vec<_>>());
    });
    let a = assets::decode(b, None).unwrap();
    let mut r = Registry::new();
    assets::insert(&mut r, a.clone()).unwrap();
    let mut d = Document::empty("capacity".into());
    d.assets.push(a.identity.clone());
    for id in 1..=32 {
        let mut o = Object::new(id, Kind::Model);
        o.resource = Some(a.identity.key());
        if id != 32 {
            o.translation[0] = 1000.0;
        }
        d.objects.push(o);
    }
    let mut h = Host::with_assets(d.clone(), r).unwrap();
    h.frame(None).unwrap();
    let writes = h.resource_snapshot()["mesh_uploads"].clone();
    match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => assert_eq!(c.instances.len(), 512),
        _ => unreachable!(),
    }
    let p = interaction::project(h.camera(), glam::Vec3::new(0.0, 0.5, 0.0)).unwrap();
    let pixel = h.rt.probe_pixel(h.three, p.to_array()).unwrap();
    assert!(
        pixel[0] > 80 && pixel[1] < 20 && pixel[2] < 20,
        "last admitted primitive must render on actual GPU: {pixel:?}"
    );
    let mut o = Object::new(33, Kind::Cylinder);
    o.visible = false;
    h.editor.begin(h.editor.stamp, Edit::Create(o)).unwrap();
    h.settle().unwrap();
    assert!(h.ui().error.contains("512"));
    assert_eq!(h.editor.document, d);
    assert_eq!(h.resource_snapshot()["mesh_uploads"], writes);
    assert_eq!(h.rt.probe_pixel(h.three, p.to_array()), Some(pixel));
}
#[test]
fn assets_before_entry_faults_conflicts_unverified_recovery_and_relocated_fresh_process() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut d, r) = scene();
    let mut e = Editor::with_assets(d.clone(), r.clone()).unwrap();
    e.select(Some(1)).unwrap();
    let mut moved = d.objects[0].clone();
    moved.translation[0] = 1.0;
    e.begin(e.stamp, Edit::Replace(moved)).unwrap();
    while !e.pump().unwrap() {}
    e.commit().unwrap();
    d = e.document.clone();
    assert!(e.dirty());
    let dir = temp();
    let path = dir.join("scene.agscene");
    let old_asset = assets::decode(
        mutation(|j| {
            j["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"] = json!([0, 0, 1, 1])
        }),
        None,
    )
    .unwrap();
    let mut old_doc = d.clone();
    old_doc.document_id = "old-runnable-scene".into();
    old_doc.assets = vec![old_asset.identity.clone()];
    old_doc.objects[0].resource = Some(old_asset.identity.key());
    old_doc.objects[0].translation = [0.0; 3];
    let mut old_registry = Registry::new();
    assets::insert(&mut old_registry, old_asset.clone()).unwrap();
    let mut old_editor = Editor::with_assets(old_doc.clone(), old_registry).unwrap();
    Storage::default()
        .save(&mut old_editor, &path, Fault::None)
        .unwrap();
    let old_entry = std::fs::read(&path).unwrap();
    let old_blob = dir
        .join("assets")
        .join(format!("{}.glb", old_asset.identity.sha256));
    let old_image = render_saved(&path);
    let mut s = Storage::default();
    assert!(s.save(&mut e, &path, Fault::AssetAfterPrepare).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), old_entry);
    assert_eq!(std::fs::read(&old_blob).unwrap(), old_asset.bytes.as_ref());
    assert_eq!(render_saved(&path), old_image);
    assert_eq!(std::fs::read_dir(dir.join("assets")).unwrap().count(), 1);
    assert!(s
        .save(&mut e, &path, Fault::AssetAfterInstall)
        .unwrap_err()
        .contains("owned residuals"));
    assert_eq!(std::fs::read(&path).unwrap(), old_entry);
    assert_eq!(std::fs::read(&old_blob).unwrap(), old_asset.bytes.as_ref());
    assert_eq!(render_saved(&path), old_image);
    assert!(e.dirty());
    assert_eq!(s.residuals.len(), 1);
    let blob = s.residuals[0].clone();
    assert_eq!(std::fs::read(&blob).unwrap(), GLB);
    assert_eq!(
        s.save(&mut e, &path, Fault::ClosureAfterReplace).unwrap(),
        Saved::VerificationPending
    );
    assert!(s.unverified());
    assert!(s.path.is_none());
    assert!(e.dirty());
    std::fs::write(&blob, b"tampered").unwrap();
    assert!(s.verify(&mut e).is_err());
    assert!(s.unverified());
    assert!(s.path.is_none());
    assert!(e.dirty());
    std::fs::write(&blob, GLB).unwrap();
    assert!(s.verify(&mut e).unwrap());
    assert!(!e.dirty());
    assert!(s.residuals.is_empty());
    let original = std::fs::read(&path).unwrap();
    std::fs::write(&blob, b"foreign bad").unwrap();
    assert!(s.save(&mut e, &path, Fault::None).is_err());
    assert_eq!(std::fs::read(&blob).unwrap(), b"foreign bad");
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(Storage::load_assets(&path, &Registry::new()).is_err());
    std::fs::write(&blob, GLB).unwrap();
    let copy = temp();
    let target = copy.join("relocated.agscene");
    let mut changed = e.document.objects[0].clone();
    changed.translation[0] += 1.0;
    e.begin(e.stamp, Edit::Replace(changed)).unwrap();
    while !e.pump().unwrap() {}
    e.commit().unwrap();
    d = e.document.clone();
    assert!(e.dirty());
    assert_eq!(
        s.save(&mut e, &target, Fault::AfterReplaceRead).unwrap(),
        Saved::VerificationPending
    );
    let old_entry = std::fs::read(&path).unwrap();
    let new_root_lock = copy.join(".autographics-assets-lock");
    std::fs::write(&new_root_lock, b"foreign new-root lock").unwrap();
    assert!(s
        .save(&mut e, &path, Fault::None)
        .unwrap_err()
        .contains("occupied"));
    assert!(s.unverified() && e.dirty());
    assert_eq!(s.path.as_ref(), Some(&path.canonicalize().unwrap()));
    assert_eq!(std::fs::read(&path).unwrap(), old_entry);
    assert_eq!(
        std::fs::read(&new_root_lock).unwrap(),
        b"foreign new-root lock"
    );
    std::fs::remove_file(new_root_lock).unwrap();
    assert_eq!(
        s.save(&mut e, &target, Fault::None).unwrap(),
        Saved::Confirmed
    );
    assert_eq!(s.path.as_ref(), Some(&target.canonicalize().unwrap()));
    std::fs::remove_file(&blob).unwrap();
    std::fs::remove_file(old_blob).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(dir.join("assets")).unwrap();
    std::fs::remove_dir(dir).unwrap();
    let child = std::process::Command::new(env!("CARGO_BIN_EXE_autographics-authoring"))
        .arg("--inspect")
        .arg(&target)
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
    let report: Value = serde_json::from_slice(&child.stdout).unwrap();
    assert_eq!(report["document"], serde_json::to_value(&d).unwrap());
    assert_eq!(report["history"], json!([0, 0, 0]));
    assert_eq!(report["dirty"], false);
    let b = copy
        .join("assets")
        .join(format!("{}.glb", d.assets[0].sha256));
    std::fs::remove_file(&b).unwrap();
    assert!(Storage::load(&target).is_err());
    std::fs::write(&b, GLB).unwrap();
    Storage::load(&target).unwrap();
    std::fs::remove_file(b).unwrap();
    std::fs::remove_file(target).unwrap();
    std::fs::remove_dir(copy.join("assets")).unwrap();
    std::fs::remove_dir(copy).unwrap();
}
#[test]
fn twenty_real_cycles_vm_asset_mesh_last_owner_cancel_and_external_pins() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp();
    let path = dir.join("own.glb");
    let other = dir.join("other.glb");
    let failing = dir.join("failing.glb");
    std::fs::write(&path, GLB).unwrap();
    std::fs::write(
        &other,
        mutation(|j| {
            j["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"] = json!([0, 0, 1, 1])
        }),
    )
    .unwrap();
    std::fs::write(
        &failing,
        mutation(|j| j["asset"]["generator"] = json!("rejected candidate")),
    )
    .unwrap();
    let mut retained = vec![];
    for cycle in 0..20 {
        let mut h = Host::new(Document::empty(format!("cycles-{cycle}"))).unwrap();
        let targets = [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq);
        let baseline = h.editor.document.clone();
        let history = h.editor.history();
        let old = h.editor.preview.vm.owner_probe();
        let message=json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"import","selection":h.editor.selection,"asset_generation":h.ui().asset_token,"payload":path.to_str()}).to_string();
        h.dispatch(&message).unwrap();
        let cancelled = Arc::downgrade(h.editor.candidate_assets().values().next().unwrap());
        let cancelled_vm = h.editor.candidate_owner_probe().unwrap();
        assert!(cancelled_vm());
        h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"cancel-import","payload":h.ui().asset_token}).to_string()).unwrap();
        assert!(cancelled.upgrade().is_none() && !cancelled_vm());
        assert_eq!(h.editor.document, baseline);
        assert_eq!(h.editor.history(), history);
        assert!(old());
        assert!(h.dispatch(&message).unwrap_err().contains("generation"));
        let prior = h.ui().asset_token;
        let current=json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"selection":h.editor.selection,"asset_generation":prior,"action":"import","payload":path.to_str()}).to_string();
        h.dispatch(&current).unwrap();
        let pending = h.editor.candidate_owner_probe().unwrap();
        let stale_cancel=json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"cancel-import","payload":prior}).to_string();
        assert!(h
            .dispatch(&stale_cancel)
            .unwrap_err()
            .contains("stale import token"));
        assert!(pending());
        h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"cancel-import","payload":h.ui().asset_token}).to_string()).unwrap();
        assert!(!pending());
        cmd(&mut h, "import", json!(path.to_str())).unwrap();
        assert!(!old());
        cmd(&mut h, "import", json!(other.to_str())).unwrap();
        let key = h.editor.document.assets[0].key();
        let asset = h.editor.assets[&key].clone();
        let weak = Arc::downgrade(&asset);
        let raw = Arc::downgrade(&asset.bytes);
        cmd(&mut h, "select", json!(2)).unwrap();
        cmd(&mut h, "asset-replace", json!(key)).unwrap();
        let before = h.editor.document.clone();
        let history = h.editor.history();
        let attempts = h.resource_snapshot()["mesh_uploads"]["buffer_rewrites"]
            .as_u64()
            .unwrap();
        h.fail_after_asset_upload = true;
        h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"import","selection":h.editor.selection,"asset_generation":h.ui().asset_token,"payload":failing.to_str()}).to_string()).unwrap();
        let rejected_vm = h.editor.candidate_owner_probe().unwrap();
        h.settle().unwrap();
        assert!(!rejected_vm());
        assert_eq!(h.editor.document, before);
        assert_eq!(h.editor.history(), history);
        assert_eq!(
            h.resource_snapshot()["mesh_uploads"]["buffer_rewrites"],
            attempts + 4
        );
        h.fail_after_asset_upload = false;
        let decodes = assets::decode_attempts();
        let stats = h.resource_snapshot()["mesh_uploads"].clone();
        let pin = match h.rt.content(h.three).unwrap() {
            SurfaceContent::Scene3d(c) => {
                c.store.pin(&asset.identity.sha256, "primitive-0").unwrap()
            }
            _ => unreachable!(),
        };
        let gpu = Arc::downgrade(&pin);
        if std::env::var_os("AG_ASSET_RETAIN_OWNER").is_some() {
            retained.push(asset.clone());
        }
        if std::env::var_os("AG_ASSET_FRESH_UPLOAD").is_some() {
            if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(h.three) {
                c.store.retain_models(|sha| sha != asset.identity.sha256);
            }
        }
        cmd(&mut h, "import", json!(path.to_str())).unwrap();
        assert!(Arc::ptr_eq(&asset, &h.editor.assets[&key]));
        assert_eq!(assets::decode_attempts(), decodes);
        assert_eq!(
            h.resource_snapshot()["mesh_uploads"],
            stats,
            "warm existing resource must not upload again"
        );
        cmd(&mut h, "undo", Value::Null).unwrap();
        cmd(&mut h, "redo", Value::Null).unwrap();
        assert_eq!(
            h.resource_snapshot()["mesh_uploads"],
            stats,
            "warm existing resource must not upload again"
        );
        let project = dir.join(format!("project-{cycle}"));
        std::fs::create_dir(&project).unwrap();
        let entry = project.join("scene.agscene");
        h.storage.save(&mut h.editor, &entry, Fault::None).unwrap();
        let saved = h.editor.document.clone();
        let previous = h.editor.preview.vm.owner_probe();
        h.open(&entry).unwrap();
        h.settle().unwrap();
        assert!(!previous());
        assert_eq!(h.editor.document, saved);
        assert_eq!(h.editor.history(), (0, 0, 0));
        assert!(Arc::ptr_eq(&asset, &h.editor.assets[&key]));
        assert_eq!(
            h.resource_snapshot()["mesh_uploads"],
            stats,
            "warm existing resource must not upload again"
        );
        println!("asset-cycle={cycle} resources={}", h.resource_snapshot());
        assert!(h.dispatch(&message).unwrap_err().contains("stale"));
        assert_eq!(
            [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq),
            targets
        );
        let vm = h.editor.preview.vm.owner_probe();
        h.request_close();
        assert!(h.closed && !vm());
        assert!(h.editor.assets.is_empty());
        assert_eq!(h.resource_snapshot()["mesh_count"], 7);
        assert!(
            weak.upgrade().is_some() && gpu.upgrade().is_some(),
            "legitimate external pins retain resources"
        );
        drop(asset);
        drop(pin);
        assert!(
            weak.upgrade().is_none() && gpu.upgrade().is_none() && raw.upgrade().is_none(),
            "last actual asset/raw/GPU owner must release"
        );
        drop(h);
        for a in saved.assets {
            std::fs::remove_file(project.join("assets").join(format!("{}.glb", a.sha256))).unwrap();
        }
        std::fs::remove_file(entry).unwrap();
        std::fs::remove_dir(project.join("assets")).unwrap();
        std::fs::remove_dir(project).unwrap();
    }
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(other).unwrap();
    std::fs::remove_file(failing).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
#[test]
fn project_lock_disk_residual_budget_foreign_temp_and_old_snapshot() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (d, r) = scene();
    let mut e = Editor::with_assets(d, r).unwrap();
    let dir = temp();
    let path = dir.join("entry.agscene");
    let mut storage = Storage::default();
    let lock = dir.join(".autographics-assets-lock");
    std::fs::write(&lock, b"foreign lock").unwrap();
    assert!(storage
        .save(&mut e, &path, Fault::None)
        .unwrap_err()
        .contains("occupied"));
    assert_eq!(std::fs::read(&lock).unwrap(), b"foreign lock");
    std::fs::remove_file(lock).unwrap();
    let assetsdir = dir.join("assets");
    std::fs::create_dir(&assetsdir).unwrap();
    let foreign = assetsdir.join("foreign.tmp");
    std::fs::write(&foreign, b"leave untouched").unwrap();
    assert!(storage
        .save(&mut e, &path, Fault::None)
        .unwrap_err()
        .contains("foreign"));
    assert_eq!(std::fs::read(&foreign).unwrap(), b"leave untouched");
    std::fs::remove_file(foreign).unwrap();
    let mut owned = vec![];
    for i in 0..8 {
        let p = assetsdir.join(format!("{i:064x}.glb"));
        std::fs::write(&p, GLB).unwrap();
        owned.push(p);
    }
    assert!(storage
        .save(&mut e, &path, Fault::None)
        .unwrap_err()
        .contains("8 files"));
    assert!(!path.exists());
    std::fs::remove_file(owned.pop().unwrap()).unwrap();
    storage.save(&mut e, &path, Fault::None).unwrap();
    assert_eq!(std::fs::read_dir(&assetsdir).unwrap().count(), 8);
    let snapshot = autographics_auto_driver::authoring::storage::Snapshot::capture(&e).unwrap();
    e.begin_open(Document::empty("later".into())).unwrap();
    while !e.pump().unwrap() {}
    e.commit().unwrap();
    let before = std::fs::read(&path).unwrap();
    assert!(storage
        .save_snapshot(&mut e, &path, snapshot, Fault::None)
        .unwrap_err()
        .contains("old epoch"));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    for p in owned {
        std::fs::remove_file(p).unwrap();
    }
    for p in std::fs::read_dir(&assetsdir).unwrap() {
        std::fs::remove_file(p.unwrap().path()).unwrap();
    }
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(assetsdir).unwrap();
    std::fs::remove_dir(dir).unwrap();
    disk_byte_boundary();
}
fn disk_byte_boundary() {
    let dir = temp();
    let assetsdir = dir.join("assets");
    std::fs::create_dir(&assetsdir).unwrap();
    let path = dir.join("scene.agscene");
    let old = Document::sample().encode().unwrap();
    std::fs::write(&path, &old).unwrap();
    let mut owned = vec![];
    for i in 0..6 {
        let b = padded(5 * 1024 * 1024, 100 + i);
        let sha = agp::renderer::capture::sha256_hex(&b);
        let p = assetsdir.join(format!("{sha}.glb"));
        std::fs::write(&p, b).unwrap();
        owned.push(p);
    }
    let editor = |bytes: Vec<u8>| {
        let a = assets::decode(bytes, None).unwrap();
        let mut r = Registry::new();
        assets::insert(&mut r, a.clone()).unwrap();
        let mut d = Document::empty("disk-boundary".into());
        d.assets.push(a.identity.clone());
        Editor::with_assets(d, r).unwrap()
    };
    let mut valid = editor(padded(2 * 1024 * 1024, 200));
    assert!(Storage::default()
        .save(&mut valid, &path, Fault::AssetAfterPrepare)
        .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), old);
    assert_eq!(std::fs::read_dir(&assetsdir).unwrap().count(), 6);
    Storage::load(&path).unwrap();
    let mut over = editor(padded(2 * 1024 * 1024 + 4, 201));
    assert!(Storage::default()
        .save(&mut over, &path, Fault::None)
        .unwrap_err()
        .contains("32MiB"));
    assert_eq!(std::fs::read(&path).unwrap(), old);
    assert_eq!(std::fs::read_dir(&assetsdir).unwrap().count(), 6);
    Storage::default()
        .save(&mut valid, &path, Fault::None)
        .unwrap();
    assert_eq!(
        std::fs::read_dir(&assetsdir)
            .unwrap()
            .map(|e| e.unwrap().metadata().unwrap().len())
            .sum::<u64>(),
        32 * 1024 * 1024
    );
    Storage::load(&path).unwrap();
    for a in valid.document.assets {
        owned.push(assetsdir.join(format!("{}.glb", a.sha256)));
    }
    for p in owned {
        std::fs::remove_file(p).unwrap();
    }
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(assetsdir).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
fn padded(target: usize, nonce: usize) -> Vec<u8> {
    let (mut j, mut bin) = chunks(GLB);
    j["asset"]["generator"] = json!(format!("own-bounded-{nonce}"));
    for _ in 0..4 {
        j["buffers"][0]["byteLength"] = json!(bin.len());
        let n = pack(&j, &bin).len();
        if n == target {
            return pack(&j, &bin);
        }
        if n < target {
            bin.resize(bin.len() + target - n, 0);
        } else {
            bin.truncate(bin.len() - (n - target));
        }
    }
    panic!("fixture padding failed")
}
#[test]
fn exact_raw_json_active_union_and_predicted_gpu_budgets_before_decode() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let maximum = padded(assets::MAX_RAW, 0);
    assert_eq!(maximum.len(), assets::MAX_RAW);
    assets::decode(maximum, None).unwrap();
    let (mut j, bin) = chunks(GLB);
    j["extras"] = json!("");
    for _ in 0..4 {
        let b = pack(&j, &bin);
        let n = u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize;
        if n == assets::MAX_JSON {
            break;
        }
        let current = j["extras"].as_str().unwrap().len();
        j["extras"] = json!("x".repeat(current + assets::MAX_JSON - n));
    }
    let b = pack(&j, &bin);
    assert_eq!(
        u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize,
        assets::MAX_JSON
    );
    assets::decode(b, None).unwrap();
    let text = j["extras"].as_str().unwrap().to_owned() + "xxxx";
    j["extras"] = json!(text);
    assert!(assets::decode(pack(&j, &bin), None)
        .unwrap_err()
        .contains("JSON 1MiB"));
    let mut r = Registry::new();
    for i in 0..8 {
        let a = assets::decode_in(padded(4 * 1024 * 1024, i), None, &r).unwrap();
        assets::insert(&mut r, a).unwrap();
    }
    assert_eq!(
        r.values().map(|a| a.bytes.len()).sum::<usize>(),
        assets::MAX_RAW_TOTAL
    );
    let dir = temp();
    let path = dir.join("duplicate-at-full-budget.glb");
    std::fs::write(&path, padded(4 * 1024 * 1024, 0)).unwrap();
    let calls = assets::decode_attempts();
    let reused = assets::import(&path, &r).unwrap();
    assert!(Arc::ptr_eq(&reused, &r[&reused.identity.key()]));
    assert_eq!(assets::decode_attempts(), calls);
    std::fs::write(&path, padded(4 * 1024 * 1024, 99)).unwrap();
    assert!(assets::import(&path, &r)
        .unwrap_err()
        .contains("before snapshot"));
    assert_eq!(assets::decode_attempts(), calls);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
    let mut seven = r.clone();
    seven.pop_last();
    let count = assets::decode_attempts();
    assert!(
        assets::decode_in(padded(4 * 1024 * 1024 + 4, 99), None, &seven)
            .unwrap_err()
            .contains("before decode")
    );
    assert_eq!(assets::decode_attempts(), count);
    drop(r);
    drop(seven);
    let mut r = Registry::new();
    for i in 0..6 {
        let b = large_primitive(65536, 65535, 4);
        let (mut j, bin) = chunks(&b);
        j["asset"]["generator"] = json!(format!("{i}"));
        let a = assets::decode_in(pack(&j, &bin), None, &r).unwrap();
        assert_eq!(
            a.cpu_bytes,
            a.model
                .meshes
                .iter()
                .map(|m| m.positions.len() * 24 + m.indices.len() * 4)
                .sum::<usize>()
        );
        assets::insert(&mut r, a).unwrap();
    }
    let b = large_primitive(65536, 65535, 4);
    let count = assets::decode_attempts();
    assert!(assets::decode_in(b, None, &r)
        .unwrap_err()
        .contains("before decode"));
    assert_eq!(assets::decode_attempts(), count);
}
#[cfg(windows)]
#[test]
fn outside_worktree_reparse_path_refuses_without_touching_target() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp();
    let target = dir.join("real");
    std::fs::create_dir(&target).unwrap();
    let source = target.join("model.glb");
    std::fs::write(&source, GLB).unwrap();
    let link = dir.join("junction");
    let status = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&link)
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(assets::import(&link.join("model.glb"), &Registry::new())
        .unwrap_err()
        .contains("reparse"));
    let mut e = Editor::new(Document::sample()).unwrap();
    assert!(Storage::default()
        .save(&mut e, &link.join("entry.agscene"), Fault::None)
        .unwrap_err()
        .contains("reparse"));
    assert_eq!(std::fs::read(&source).unwrap(), GLB);
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
    // RemoveDirectory on this literal junction removes the link, never recurses.
    std::fs::remove_dir(&link).unwrap();
    std::fs::remove_file(source).unwrap();
    std::fs::remove_dir(target).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
