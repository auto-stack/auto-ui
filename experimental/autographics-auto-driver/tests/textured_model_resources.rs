//! PLAN-005 public import/storage/render closure probes. Required inputs are
//! original fixed-commit assets, not stripped or recoloured derivatives.
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    document::Document,
    host::Host,
    storage::{Fault, Saved, Storage},
};
use autographics_auto_driver::authoring::{
    document::{Kind, Object},
    session::Editor,
    textured_models,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/textured_models")
}
fn temp() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ui005-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    root
}
fn cmd(h: &mut Host, action: &str, payload: Value) -> Result<(), String> {
    h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":action,"payload":payload,"selection":h.editor.selection,"asset_generation":h.ui().asset_token}).to_string())?;
    h.settle()?;
    if !h.ui().error.is_empty() {
        return Err(h.ui().error.clone());
    }
    Ok(())
}
#[test]
fn fixed_sources_public_import_save_move_reopen_and_actual_gpu() {
    let expected = [
        (
            "platform",
            "78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c",
        ),
        (
            "coin",
            "a3fb8f779a6af1cd75e5a02da77a87546ca8a59f6fe52f79f25179c4a68fd0e5",
        ),
        (
            "flag",
            "0ba84ff43f0fea0a9f0b080cfa1be54ebda36c8c35174d1ac2553ae6da9f65ae",
        ),
    ];
    for (name, hash) in expected {
        let path = samples().join(format!("models/{name}.glb"));
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(agp::renderer::capture::sha256_hex(&bytes), hash);
        assert!(
            assets::decode(bytes.clone(), None)
                .unwrap_err()
                .contains("unsupported"),
            "old static profile remains restrictive"
        );
        let mut h = Host::new(Document::empty(format!("texture-{name}"))).unwrap();
        cmd(&mut h, "import", json!(path.to_str().unwrap())).unwrap();
        let identity = h.editor.document.assets[0].clone();
        assert_eq!(identity.sha256, hash);
        assert!(identity.closure.is_some());
        assert_eq!(h.editor.document.schema, "autographics.scene/6");
        let a = h.editor.assets[&identity.key()].clone();
        assert_eq!(a.bytes.as_ref(), bytes);
        assert_eq!(a.dependencies.len(), 1);
        for m in &a.model.meshes {
            let v = m.visual.as_ref().unwrap();
            assert!(v.double_sided);
            assert!(v.image.is_some());
            assert_eq!(v.uv.len(), m.positions.len());
            assert_eq!(v.node_index, 0);
        }
        h.frame(None).unwrap();
        let before = h.rt.capture(h.three, "ui005-original").unwrap().0;
        let root = temp();
        let a_path = root.join("a");
        std::fs::create_dir(&a_path).unwrap();
        let entry = a_path.join("scene.agscene");
        let saved = h.storage.save(&mut h.editor, &entry, Fault::None).unwrap();
        assert_eq!(saved, Saved::Confirmed);
        let moved = root.join("b");
        std::fs::rename(&a_path, &moved).unwrap();
        let (doc, registry, _) =
            Storage::load_assets(&moved.join("scene.agscene"), &Registry::new()).unwrap();
        assert_eq!(registry[&identity.key()].identity, identity);
        let mut reopened = Host::with_assets(doc, registry).unwrap();
        reopened.frame(None).unwrap();
        let after = reopened
            .rt
            .capture(reopened.three, "ui005-reopened")
            .unwrap()
            .0;
        assert_eq!(before, after, "{name} moved render content");
        // Same original GLB, different accepted external image: this must
        // change real pixels, not just a metadata key or resource counter.
        let other = root.join("other");
        std::fs::create_dir_all(other.join("Textures")).unwrap();
        std::fs::write(other.join("model.glb"), &bytes).unwrap();
        let png = agp::assets::encode_png_rgba(2, 2, &[255, 0, 255, 255].repeat(4)).unwrap();
        std::fs::write(other.join("Textures/colormap.png"), png).unwrap();
        let mut changed = Host::new(Document::empty(format!("changed-{name}"))).unwrap();
        cmd(
            &mut changed,
            "import",
            json!(other.join("model.glb").to_str().unwrap()),
        )
        .unwrap();
        changed.frame(None).unwrap();
        assert_eq!(changed.editor.document.assets[0].sha256, hash);
        assert_ne!(changed.editor.document.assets[0].key(), identity.key());
        assert_ne!(
            changed
                .rt
                .capture(changed.three, "ui005-external-image-control")
                .unwrap()
                .0,
            before,
            "{name}: image mutation must alter GPU output"
        );
        println!(
            "UI005 fixed source {name} root={hash} closure={} rendered={before}",
            identity.closure.unwrap().id
        );
    }
    let error =
        assets::import(&samples().join("models/character.glb"), &Registry::new()).unwrap_err();
    assert!(error.contains("animations"), "{error}");
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
    let mut out = b"glTF".to_vec();
    out.extend(2u32.to_le_bytes());
    out.extend(((28 + j.len() + bin.len()) as u32).to_le_bytes());
    out.extend((j.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(j);
    out.extend((bin.len() as u32).to_le_bytes());
    out.extend([b'B', b'I', b'N', 0]);
    out.extend(bin);
    out
}
#[test]
fn unsafe_declarations_are_rejected_and_source_control_remains_accepted() {
    let root = temp();
    std::fs::create_dir(root.join("Textures")).unwrap();
    let original = std::fs::read(samples().join("models/platform.glb")).unwrap();
    std::fs::copy(
        samples().join("models/Textures/colormap.png"),
        root.join("Textures/colormap.png"),
    )
    .unwrap();
    for value in [
        "../escape.png",
        "/absolute.png",
        "https://a/image.png",
        "data:image/png;base64,AAAA",
        "Textures/%2e/image.png",
        "Textures/colormap.png?x",
        "Textures\\colormap.png",
        "CON.png",
        "Textures/LPT1.png",
        "Textures/colormap.png.",
    ] {
        let (mut j, b) = chunks(&original);
        j["images"][0]["uri"] = json!(value);
        std::fs::write(root.join("model.glb"), pack(&j, &b)).unwrap();
        let e = assets::import(&root.join("model.glb"), &Registry::new()).unwrap_err();
        assert!(e.contains("URI"), "{value}: {e}");
    }
    let cases: Vec<(&str, Box<dyn Fn(&mut Value)>)> = vec![
        (
            "extension",
            Box::new(|j| j["extensionsRequired"] = json!(["KHR_draco_mesh_compression"])),
        ),
        (
            "extension",
            Box::new(|j| j["nodes"][0]["extensions"] = json!({"KHR_texture_transform":{}})),
        ),
        (
            "UV0",
            Box::new(|j| {
                j["materials"][0]["pbrMetallicRoughness"]["baseColorTexture"]["texCoord"] = json!(1)
            }),
        ),
        (
            "sampler",
            Box::new(|j| j["textures"][0]["sampler"] = json!(99)),
        ),
        (
            "minFilter",
            Box::new(|j| j["samplers"][0]["minFilter"] = json!(123)),
        ),
        (
            "material",
            Box::new(|j| j["materials"][0]["alphaMode"] = json!("BLEND")),
        ),
        (
            "accessor",
            Box::new(|j| j["accessors"][0]["componentType"] = json!(5121)),
        ),
        (
            "span",
            Box::new(|j| j["bufferViews"][0]["byteLength"] = json!(999999)),
        ),
        (
            "image",
            Box::new(|j| j["images"][0]["bufferView"] = json!(0)),
        ),
        (
            "UV",
            Box::new(|j| {
                j["materials"][0]["pbrMetallicRoughness"]["baseColorTexture"]["extensions"]
                    ["KHR_texture_transform"]["scale"] = json!([1e300, 1])
            }),
        ),
    ];
    for (label, change) in cases {
        let (mut j, b) = chunks(&original);
        change(&mut j);
        std::fs::write(root.join("model.glb"), pack(&j, &b)).unwrap();
        let e = assets::import(&root.join("model.glb"), &Registry::new()).unwrap_err();
        assert!(e.contains(label), "{label}: {e}");
    }
    std::fs::write(root.join("model.glb"), original).unwrap();
    assert!(assets::import(&root.join("model.glb"), &Registry::new()).is_ok());
}

#[test]
fn nonfinite_uv_and_tangents_and_image_bombs_are_rejected_before_decode() {
    let root = temp();
    let path = input_copy(&root);
    let original = std::fs::read(&path).unwrap();
    for semantic in ["TEXCOORD_0", "TANGENT"] {
        let (j, mut bin) = chunks(&original);
        let index = j["meshes"][0]["primitives"][0]["attributes"][semantic]
            .as_u64()
            .unwrap() as usize;
        let a = &j["accessors"][index];
        let view = &j["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
        let at = view.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize
            + a.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize;
        bin[at..at + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        std::fs::write(&path, pack(&j, &bin)).unwrap();
        let before = assets::decode_attempts();
        let e = assets::import(&path, &Registry::new()).unwrap_err();
        assert!(e.contains("invalid:"), "{semantic}: {e}");
        assert_eq!(assets::decode_attempts(), before);
    }
    // Both operands fit f32, but their product does not. This must still be
    // rejected before a PNG decoder or external dependency read is attempted.
    let (mut j, mut bin) = chunks(&original);
    let index = j["meshes"][0]["primitives"][0]["attributes"]["TEXCOORD_0"]
        .as_u64()
        .unwrap() as usize;
    let a = &j["accessors"][index];
    let view = &j["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
    let at = view.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize
        + a.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize;
    bin[at..at + 4].copy_from_slice(&1e38f32.to_le_bytes());
    j["materials"][0]["pbrMetallicRoughness"]["baseColorTexture"]["extensions"]
        ["KHR_texture_transform"]["scale"] = json!([1e38, 1]);
    std::fs::write(&path, pack(&j, &bin)).unwrap();
    let before = assets::decode_attempts();
    assert!(assets::import(&path, &Registry::new())
        .unwrap_err()
        .contains("transformed UV"));
    assert_eq!(assets::decode_attempts(), before);
    let (mut j, bin) = chunks(&original);
    j["meshes"][0]["primitives"][0]["attributes"]
        .as_object_mut()
        .unwrap()
        .remove("TEXCOORD_0");
    std::fs::write(&path, pack(&j, &bin)).unwrap();
    assert!(assets::import(&path, &Registry::new())
        .unwrap_err()
        .contains("requires UV0"));
    assert_eq!(assets::decode_attempts(), before);
    std::fs::write(&path, &original).unwrap();
    let image = root.join("Textures/colormap.png");
    let original_image = std::fs::read(&image).unwrap();
    let mut bomb = original_image.clone();
    bomb[16..20].copy_from_slice(&2049u32.to_be_bytes());
    std::fs::write(&image, bomb).unwrap();
    let before = assets::decode_attempts();
    let e = assets::import(&path, &Registry::new()).unwrap_err();
    assert!(e.contains("resource limit"), "{e}");
    assert_eq!(assets::decode_attempts(), before);
    std::fs::write(
        &image,
        agp::assets::encode_png_rgba(1, 1, &[1, 2, 3, 254]).unwrap(),
    )
    .unwrap();
    assert!(assets::import(&path, &Registry::new())
        .unwrap_err()
        .contains("opaque"));
    std::fs::write(image, original_image).unwrap();
    assert!(assets::import(&path, &Registry::new()).is_ok());
}

#[cfg(windows)]
#[test]
fn external_png_reparse_dependency_is_refused_without_touching_target() {
    let root = temp();
    let input = root.join("input");
    let path = input_copy(&input);
    let target = root.join("outside");
    std::fs::create_dir(&target).unwrap();
    let image = target.join("colormap.png");
    std::fs::copy(samples().join("models/Textures/colormap.png"), &image).unwrap();
    let original = std::fs::read(&image).unwrap();
    let link = input.join("linked");
    let result = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&link)
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let (mut j, bin) = chunks(&std::fs::read(&path).unwrap());
    j["images"][0]["uri"] = json!("linked/colormap.png");
    std::fs::write(&path, pack(&j, &bin)).unwrap();
    let before = assets::decode_attempts();
    let error = assets::import(&path, &Registry::new()).unwrap_err();
    assert!(error.contains("reparse"), "{error}");
    assert_eq!(assets::decode_attempts(), before);
    assert_eq!(std::fs::read(image).unwrap(), original);
    // Remove only the literal test junction, without recursing into its target.
    std::fs::remove_dir(link).unwrap();
}

fn scene_for(a: Arc<assets::Asset>) -> (Document, Registry) {
    let mut doc = Document::empty("ui005-storage".into());
    doc.schema = "autographics.scene/6".into();
    doc.assets.push(a.identity.clone());
    let mut o = Object::new(1, Kind::Model);
    o.resource = Some(a.identity.key());
    doc.objects.push(o);
    let mut r = Registry::new();
    assets::insert(&mut r, a).unwrap();
    (doc, r)
}
fn input_copy(root: &Path) -> PathBuf {
    std::fs::create_dir_all(root.join("Textures")).unwrap();
    std::fs::copy(
        samples().join("models/platform.glb"),
        root.join("model.glb"),
    )
    .unwrap();
    std::fs::copy(
        samples().join("models/Textures/colormap.png"),
        root.join("Textures/colormap.png"),
    )
    .unwrap();
    root.join("model.glb")
}
#[test]
fn same_root_different_images_coexist_and_shared_image_owners_are_real() {
    let root = temp();
    let path = input_copy(&root);
    let a = assets::import(&path, &Registry::new()).unwrap();
    let (_, mut r) = scene_for(a.clone());
    let b = assets::import(&samples().join("models/coin.glb"), &r).unwrap();
    assets::insert(&mut r, b.clone()).unwrap();
    assert!(Arc::ptr_eq(&a.model_images[0], &b.model_images[0]));
    let image_id = &a.identity.closure.as_ref().unwrap().images[0].sha256;
    assert!(Arc::ptr_eq(
        &a.dependencies[image_id],
        &b.dependencies[image_id]
    ));
    assert_eq!(
        textured_models::usage(&r).0,
        3,
        "two roots + one shared PNG"
    );
    std::fs::write(
        root.join("Textures/colormap.png"),
        agp::assets::encode_png_rgba(1, 1, &[0, 255, 255, 255]).unwrap(),
    )
    .unwrap();
    let c = assets::import(&path, &r).unwrap();
    assert_eq!(a.identity.sha256, c.identity.sha256);
    assert_ne!(a.identity.key(), c.identity.key());
    assert!(Arc::ptr_eq(&a.bytes, &c.bytes));
    assets::insert(&mut r, c.clone()).unwrap();
    assert_eq!(
        textured_models::usage(&r).0,
        4,
        "root identity deduplication"
    );
    assert_ne!(a.model_images[0].rgba, c.model_images[0].rgba);
    assert_eq!(a.model_images[0].rgba[3], 255);
    let sprite = assets::import_png(&root.join("Textures/colormap.png"), &r).unwrap();
    assert!(Arc::ptr_eq(
        sprite.image.as_ref().unwrap(),
        &c.model_images[0]
    ));
    assets::insert(&mut r, sprite).unwrap();
    assert_eq!(textured_models::usage(&r).0, 4);
    // Persist both environments of the same root in one project.
    let mut doc = Document::empty("two-closures".into());
    doc.schema = "autographics.scene/6".into();
    for (i, asset) in [a, c].into_iter().enumerate() {
        doc.assets.push(asset.identity.clone());
        let mut o = Object::new(i as u64 + 1, Kind::Model);
        o.resource = Some(asset.identity.key());
        doc.objects.push(o);
    }
    let selected: Registry = doc
        .assets
        .iter()
        .map(|a| (a.key(), r[&a.key()].clone()))
        .collect();
    let mut e = Editor::with_assets(doc.clone(), selected).unwrap();
    let entry = root.join("saved.agscene");
    Storage::default()
        .save(&mut e, &entry, Fault::None)
        .unwrap();
    let (loaded, registry, _) = Storage::load_assets(&entry, &Registry::new()).unwrap();
    assert_eq!(loaded, doc);
    assert_eq!(registry.len(), 2);
    assert_eq!(textured_models::usage(&registry).0, 3);
}

#[test]
fn legacy_schema_controls_reject_closure_fields_even_null() {
    let a = assets::import(&samples().join("models/platform.glb"), &Registry::new()).unwrap();
    for version in 1..=5 {
        let mut valid: Value =
            serde_json::from_slice(&Document::empty("legacy-control".into()).encode().unwrap())
                .unwrap();
        valid["schema"] = json!(format!("autographics.scene/{version}"));
        if version <= 3 {
            valid.as_object_mut().unwrap().remove("behavior");
        }
        if version <= 2 {
            valid.as_object_mut().unwrap().remove("assets");
        }
        if version == 1 {
            valid.as_object_mut().unwrap().remove("groups");
        }
        Document::decode(&serde_json::to_vec(&valid).unwrap())
            .unwrap_or_else(|e| panic!("valid legacy {version}: {e}"));
        for closure in [
            serde_json::to_value(&a.identity.closure).unwrap(),
            Value::Null,
        ] {
            let mut invalid = valid.clone();
            let mut asset = serde_json::to_value(&a.identity).unwrap();
            asset["closure"] = closure;
            invalid["assets"] = json!([asset]);
            let error = Document::decode(&serde_json::to_vec(&invalid).unwrap()).unwrap_err();
            assert!(
                error.contains("model closure field"),
                "legacy {version}: {error}"
            );
        }
    }
}

#[test]
fn dependencies_are_checked_even_with_cached_models_and_recovery_reads_again() {
    let root = temp();
    let path = input_copy(&root.join("input"));
    let a = assets::import(&path, &Registry::new()).unwrap();
    let original = a.dependencies.values().next().unwrap().clone();
    let (doc, registry) = scene_for(a.clone());
    let mut e = Editor::with_assets(doc.clone(), registry.clone()).unwrap();
    // An accepted snapshot must not be contaminated by a changed import directory.
    std::fs::write(
        path.parent().unwrap().join("Textures/colormap.png"),
        b"changed after import",
    )
    .unwrap();
    let entry = root.join("scene.agscene");
    let mut storage = Storage::default();
    assert_eq!(
        storage.save(&mut e, &entry, Fault::None).unwrap(),
        Saved::Confirmed
    );
    let image = &a.identity.closure.as_ref().unwrap().images[0];
    let blob = root.join(format!("assets/{}.png", image.sha256));
    assert_eq!(std::fs::read(&blob).unwrap(), original.as_ref());
    std::fs::remove_file(&blob).unwrap();
    assert!(Storage::load_assets(&entry, &registry).is_err());
    std::fs::write(&blob, b"bad").unwrap();
    assert!(Storage::load_assets(&entry, &registry).is_err());
    std::fs::write(&blob, &original).unwrap();
    let (_, healed, _) = Storage::load_assets(&entry, &registry).unwrap();
    assert!(Arc::ptr_eq(&healed[&a.identity.key()], &a));
    let saved = std::fs::read(&entry).unwrap();
    let mut wire: Value = serde_json::from_slice(&saved).unwrap();
    // Rehash the forged edge to exercise the binding-to-root check, not only a stale hash.
    wire["assets"][0]["closure"]["images"][0]["source"] = json!("uri:wrong.png");
    let refs: Vec<textured_models::ImageRef> =
        serde_json::from_value(wire["assets"][0]["closure"]["images"].clone()).unwrap();
    let forged = agp::renderer::capture::sha256_hex(
        &serde_json::to_vec(&(
            "autographics.textured/1",
            &a.identity.sha256,
            a.bytes.len(),
            refs,
        ))
        .unwrap(),
    );
    wire["assets"][0]["closure"]["id"] = json!(forged);
    wire["objects"][0]["resource"] = json!(format!("closure:{forged}"));
    std::fs::write(&entry, serde_json::to_vec(&wire).unwrap()).unwrap();
    let error = Storage::load_assets(&entry, &registry).err().unwrap();
    assert!(error.contains("edge"), "{error}");
    std::fs::write(&entry, &saved).unwrap();
    assert!(Storage::load_assets(&entry, &registry).is_ok());
    for schema in [
        "autographics.scene/1",
        "autographics.scene/2",
        "autographics.scene/3",
        "autographics.scene/4",
        "autographics.scene/5",
    ] {
        let mut old: Value = serde_json::from_slice(&saved).unwrap();
        old["schema"] = json!(schema);
        assert!(
            Document::decode(&serde_json::to_vec(&old).unwrap()).is_err(),
            "{schema} cannot claim closure semantics"
        );
    }
}

#[test]
fn save_faults_keep_entry_last_and_never_confirm_a_broken_dependency() {
    let a = assets::import(&samples().join("models/platform.glb"), &Registry::new()).unwrap();
    for fault in [
        Fault::AssetAfterPrepare,
        Fault::AssetAfterInstall,
        Fault::BeforeReplace,
        Fault::AfterReplaceRead,
        Fault::ClosureAfterReplace,
    ] {
        let root = temp();
        let entry = root.join("scene.agscene");
        let (doc, r) = scene_for(a.clone());
        let mut e = Editor::with_assets(doc, r).unwrap();
        let mut s = Storage::default();
        let result = s.save(&mut e, &entry, fault);
        if [Fault::AfterReplaceRead, Fault::ClosureAfterReplace].contains(&fault) {
            assert_eq!(result.unwrap(), Saved::VerificationPending);
            assert!(s.path.is_none());
            assert!(s.unverified());
            let blob = root.join(format!(
                "assets/{}.png",
                a.identity.closure.as_ref().unwrap().images[0].sha256
            ));
            let b = std::fs::read(&blob).unwrap();
            std::fs::remove_file(&blob).unwrap();
            assert!(s.verify(&mut e).is_err());
            assert!(s.unverified());
            assert!(s.path.is_none());
            std::fs::write(blob, b).unwrap();
            assert!(s.verify(&mut e).unwrap());
            assert!(!s.unverified());
            assert!(s.residuals.is_empty());
        } else {
            assert!(result.is_err());
            assert!(!entry.exists());
            assert!(s.path.is_none());
            assert_eq!(
                s.save(&mut e, &entry, Fault::None).unwrap(),
                Saved::Confirmed
            );
        }
        assert!(Storage::load_assets(&entry, &Registry::new()).is_ok());
    }
    let root = temp();
    let entry = root.join("scene.agscene");
    std::fs::create_dir(root.join("assets")).unwrap();
    let blob = root.join(format!(
        "assets/{}.png",
        a.identity.closure.as_ref().unwrap().images[0].sha256
    ));
    std::fs::write(&blob, b"foreign").unwrap();
    let (doc, r) = scene_for(a);
    let mut e = Editor::with_assets(doc, r).unwrap();
    assert!(Storage::default()
        .save(&mut e, &entry, Fault::None)
        .is_err());
    assert_eq!(std::fs::read(blob).unwrap(), b"foreign");
    assert!(!entry.exists());
}

#[test]
fn embedded_image_uses_no_external_io_and_every_unused_image_is_checked() {
    let original = std::fs::read(samples().join("models/platform.glb")).unwrap();
    let png = std::fs::read(samples().join("models/Textures/colormap.png")).unwrap();
    let (mut j, mut bin) = chunks(&original);
    let offset = bin.len();
    bin.extend(&png);
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    let view = j["bufferViews"].as_array().unwrap().len();
    j["bufferViews"]
        .as_array_mut()
        .unwrap()
        .push(json!({"buffer":0,"byteOffset":offset,"byteLength":png.len()}));
    j["buffers"][0]["byteLength"] = json!(bin.len());
    j["images"] = json!([{"bufferView":view,"mimeType":"image/png"}]);
    let root = temp();
    let path = root.join("embedded.glb");
    std::fs::write(&path, pack(&j, &bin)).unwrap();
    let a = assets::import(&path, &Registry::new()).unwrap();
    assert!(a.dependencies.is_empty());
    assert_eq!(
        a.model_images[0].rgba,
        assets::import(&samples().join("models/platform.glb"), &Registry::new())
            .unwrap()
            .model_images[0]
            .rgba
    );
    let (doc, r) = scene_for(a);
    let mut e = Editor::with_assets(doc, r).unwrap();
    let entry = root.join("scene.agscene");
    Storage::default()
        .save(&mut e, &entry, Fault::None)
        .unwrap();
    assert_eq!(std::fs::read_dir(root.join("assets")).unwrap().count(), 1);
    assert!(Storage::load_assets(&entry, &Registry::new()).is_ok());
    input_copy(&root);
    let (mut j, bin) = chunks(&original);
    j["images"]
        .as_array_mut()
        .unwrap()
        .push(json!({"uri":"missing.png"}));
    std::fs::write(&path, pack(&j, &bin)).unwrap();
    let before = assets::decode_attempts();
    assert!(assets::import(&path, &Registry::new()).is_err());
    assert_eq!(assets::decode_attempts(), before);
}

#[test]
fn raw_identity_union_at_eight_accepts_duplicates_and_refuses_the_ninth_before_decode() {
    let root = temp();
    let original = std::fs::read(samples().join("models/platform.glb")).unwrap();
    let (mut j, bin) = chunks(&original);
    let images: Vec<_> = (0..7)
        .map(|i| json!({"uri":format!("image{i}.png")}))
        .collect();
    j["images"] = json!(images);
    for i in 0..7 {
        std::fs::write(
            root.join(format!("image{i}.png")),
            agp::assets::encode_png_rgba(1, 1, &[i * 30, 0, 255, 255]).unwrap(),
        )
        .unwrap();
    }
    let path = root.join("model.glb");
    std::fs::write(&path, pack(&j, &bin)).unwrap();
    let a = assets::import(&path, &Registry::new()).unwrap();
    let (_, mut r) = scene_for(a.clone());
    assert_eq!(textured_models::usage(&r).0, 8);
    let before = assets::decode_attempts();
    assert!(Arc::ptr_eq(&a, &assets::import(&path, &r).unwrap()));
    let sprite = assets::import_png(&root.join("image0.png"), &r).unwrap();
    assert!(Arc::ptr_eq(
        sprite.image.as_ref().unwrap(),
        &a.model_images[0]
    ));
    assets::insert(&mut r, sprite).unwrap();
    assert_eq!(textured_models::usage(&r).0, 8);
    assert_eq!(
        assets::decode_attempts(),
        before,
        "full-union duplicates do not decode again"
    );
    let other = root.join("ninth.png");
    std::fs::write(
        &other,
        agp::assets::encode_png_rgba(1, 1, &[1, 2, 3, 255]).unwrap(),
    )
    .unwrap();
    assert!(assets::import_png(&other, &r)
        .unwrap_err()
        .contains("resource limit"));
    assert_eq!(assets::decode_attempts(), before);
}

#[test]
fn decoded_union_includes_unused_model_images_and_legacy_sprite_imports() {
    let root = temp();
    let original = std::fs::read(samples().join("models/platform.glb")).unwrap();
    let (mut j, bin) = chunks(&original);
    for i in 0..4 {
        let pixels = [i * 50, 0, 255, 255].repeat(2048 * 2048);
        std::fs::write(
            root.join(format!("image{i}.png")),
            agp::assets::encode_png_rgba(2048, 2048, &pixels).unwrap(),
        )
        .unwrap();
    }
    j["images"] = json!((0..4)
        .map(|i| json!({"uri":format!("image{i}.png")}))
        .collect::<Vec<_>>());
    let path = root.join("model.glb");
    std::fs::write(&path, pack(&j, &bin)).unwrap();
    let before = assets::decode_attempts();
    let error = assets::import(&path, &Registry::new()).unwrap_err();
    assert!(error.contains("resource limit"), "{error}");
    assert_eq!(
        assets::decode_attempts(),
        before,
        "all declarations predicted before pixels or mesh decode"
    );
    j["images"].as_array_mut().unwrap().pop();
    std::fs::write(&path, pack(&j, &bin)).unwrap();
    let a = assets::import(&path, &Registry::new()).unwrap();
    let (_, r) = scene_for(a);
    let usage = textured_models::usage(&r);
    assert_eq!(usage.0, 4);
    assert!(usage.2 > 48 * 1024 * 1024 && usage.2 < 64 * 1024 * 1024);
    assert!(usage.3 > 48 * 1024 * 1024 && usage.3 < 64 * 1024 * 1024);
    let before = assets::decode_attempts();
    assert!(assets::import_png(&root.join("image3.png"), &r)
        .unwrap_err()
        .contains("before decode"));
    assert_eq!(
        assets::decode_attempts(),
        before,
        "sprite shares the model image budget"
    );
}

fn padded_png(colour: u8) -> Vec<u8> {
    let b = agp::assets::encode_png_rgba(1, 1, &[colour, 0, 0, 255]).unwrap();
    let data = vec![0; assets::MAX_RAW - b.len() - 12];
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xedb88320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *t = c;
    }
    let mut crc = !0u32;
    for x in b"ruSt".iter().chain(data.iter()) {
        crc = table[((crc ^ *x as u32) & 255) as usize] ^ (crc >> 8);
    }
    let mut out = b[..b.len() - 12].to_vec();
    out.extend((data.len() as u32).to_be_bytes());
    out.extend(b"ruSt");
    out.extend(data);
    out.extend((!crc).to_be_bytes());
    out.extend(&b[b.len() - 12..]);
    assert_eq!(out.len(), assets::MAX_RAW);
    out
}
#[test]
fn raw_byte_union_refuses_over_32_mib_before_decoder_and_reuses_exact_large_duplicates() {
    let a = assets::import(&samples().join("models/platform.glb"), &Registry::new()).unwrap();
    let (_, mut r) = scene_for(a);
    let root = temp();
    for i in 0..3 {
        let path = root.join(format!("large{i}.png"));
        std::fs::write(&path, padded_png(i * 50)).unwrap();
        let image = assets::import_png(&path, &r).unwrap();
        assets::insert(&mut r, image).unwrap();
    }
    let before = assets::decode_attempts();
    let image = assets::import_png(&root.join("large0.png"), &r).unwrap();
    assert!(Arc::ptr_eq(&image, &r[&image.identity.key()]));
    assert_eq!(assets::decode_attempts(), before);
    let path = root.join("large3.png");
    std::fs::write(&path, padded_png(200)).unwrap();
    let e = assets::import_png(&path, &r).unwrap_err();
    assert!(e.contains("resource limit"), "{e}");
    assert_eq!(assets::decode_attempts(), before);
    assert!(textured_models::usage(&r).1 < assets::MAX_RAW_TOTAL);
}

fn textured_quad_fixture(transform: bool, min_filter: u32) -> (Vec<u8>, Vec<u8>) {
    let mut bin = vec![];
    for p in [
        [-1f32, -1., 0.5],
        [1., -1., 0.5],
        [1., 1., 0.5],
        [-1., 1., 0.5],
    ] {
        for v in p {
            bin.extend(v.to_le_bytes());
        }
    }
    for _ in 0..4 {
        for v in [0f32, 0., 1.] {
            bin.extend(v.to_le_bytes());
        }
    }
    for uv in [[0f32, 1.], [1., 1.], [1., 0.], [0., 0.]] {
        for v in uv {
            bin.extend(v.to_le_bytes());
        }
    }
    for i in [0u16, 1, 2, 0, 2, 3] {
        bin.extend(i.to_le_bytes());
    }
    let mut j = json!({"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"translation":[0.2,-0.1,0],"children":[1]},{"mesh":0}],
        "buffers":[{"byteLength":bin.len()}],"bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":48},{"buffer":0,"byteOffset":48,"byteLength":48},{"buffer":0,"byteOffset":96,"byteLength":32},{"buffer":0,"byteOffset":128,"byteLength":12}],
        "accessors":[{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","min":[-1,-1,0.5],"max":[1,1,0.5]},{"bufferView":1,"componentType":5126,"count":4,"type":"VEC3"},{"bufferView":2,"componentType":5126,"count":4,"type":"VEC2"},{"bufferView":3,"componentType":5123,"count":6,"type":"SCALAR"}],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0,"NORMAL":1,"TEXCOORD_0":2},"indices":3,"material":0}]}],
        "materials":[{"pbrMetallicRoughness":{"baseColorFactor":[0.35,0.65,0.8,1],"metallicFactor":0,"roughnessFactor":1,"baseColorTexture":{"index":0}}}],
        "images":[{"uri":"colours.png"}],"textures":[{"source":0,"sampler":0}],"samplers":[{"magFilter":9728,"minFilter":min_filter,"wrapS":33071,"wrapT":33071}]});
    if transform {
        j["extensionsUsed"] = json!(["KHR_texture_transform"]);
        j["materials"][0]["pbrMetallicRoughness"]["baseColorTexture"]["extensions"] = json!({"KHR_texture_transform":{"offset":[0.8,0.1],"rotation":std::f32::consts::FRAC_PI_2,"scale":[0.7,0.6],"texCoord":0}});
    }
    let mut rgba = vec![];
    for y in 0..4 {
        for x in 0..4 {
            rgba.extend([
                20 + x * 40 + y * 10,
                15 + x * 15 + y * 45,
                240 - x * 30 - y * 20,
                255,
            ]);
        }
    }
    (
        pack(&j, &bin),
        agp::assets::encode_png_rgba(4, 4, &rgba).unwrap(),
    )
}
fn linear_srgb(v: u8) -> f32 {
    let x = v as f32 / 255.;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}
fn encode_srgb(x: f32) -> u8 {
    let x = x.clamp(0., 1.);
    (255.
        * if x <= 0.0031308 {
            x * 12.92
        } else {
            1.055 * x.powf(1. / 2.4) - 0.055
        })
    .round() as u8
}
#[test]
fn gltf_uv_transform_changes_pixels_but_preserves_collision_bounds_and_node_associations() {
    use agp::{
        render3d::{LightDef, MeshInstance, MeshPipeline, MeshStore},
        renderer::{capture::capture_png, target::OffscreenTarget, GpuDevice},
    };
    use glam::{Mat4, Vec3};
    let root = temp();
    let (raw, png) = textured_quad_fixture(true, 9728);
    let (plain, _) = textured_quad_fixture(false, 9728);
    let path = root.join("transformed.glb");
    std::fs::write(&path, &raw).unwrap();
    std::fs::write(root.join("colours.png"), png).unwrap();
    let a = assets::import(&path, &Registry::new()).unwrap();
    std::fs::write(root.join("plain.glb"), plain).unwrap();
    let b = assets::import(&root.join("plain.glb"), &Registry::new()).unwrap();
    let (doc, _) = scene_for(a.clone());
    let (plain_doc, _) = scene_for(b.clone());
    let before =
        autographics_auto_driver::authoring::spatial::Descriptor::build(&doc, |_| Some(&a.model))
            .unwrap();
    let after = autographics_auto_driver::authoring::spatial::Descriptor::build(&plain_doc, |_| {
        Some(&b.model)
    })
    .unwrap();
    assert_eq!(
        before.rows(),
        after.rows(),
        "visual transform cannot change collision AABB"
    );
    assert_eq!(a.bytes.as_ref(), raw);
    let visual = a.model.meshes[0].visual.as_ref().unwrap();
    assert_eq!(
        (visual.node_index, visual.mesh_index, visual.primitive_index),
        (1, 0, 0)
    );
    let local = Mat4::from_cols_array_2d(&visual.world)
        .inverse()
        .transform_point3(Vec3::from(a.model.meshes[0].positions[0]));
    assert!(
        (local - Vec3::new(-1., -1., 0.5)).length() < 1e-6,
        "local geometry remains recoverable for 006"
    );
    let gpu = GpuDevice::new().unwrap();
    let target = OffscreenTarget::new(&gpu, 64, 64).unwrap();
    let mut pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap();
    let mut store = MeshStore::default();
    let light = LightDef {
        dir_to_light: Vec3::Z,
        color: [1.; 3],
        energy: 0.5,
        shadow_vp: Mat4::IDENTITY,
        shadow_bias: 0.0005,
    };
    let vp = Mat4::from_translation(Vec3::new(-0.2, 0.1, 0.));
    let draw =
        |model: &agp::assets::ModelData, pipeline: &mut MeshPipeline, store: &mut MeshStore| {
            store.get_or_upload(&gpu, model, 0).unwrap();
            pipeline.render(
                &gpu,
                target.view(),
                64,
                64,
                [0., 0., 0., 1.],
                &vp,
                &light,
                store,
                &[MeshInstance {
                    mesh: (model.source_sha256.clone(), model.meshes[0].name.clone()),
                    model: Mat4::IDENTITY,
                }],
                false,
            );
            capture_png(&gpu, &target).unwrap().1
        };
    let expected = |x: u32, y: u32| {
        // Independently specialize a 90-degree KHR transform, rather than
        // reusing the producer's sin/cos or already transformed vertex UVs.
        let u = 0.8 - 0.6 * (y as f32 + 0.5) / 64.;
        let v = 0.1 + 0.7 * (x as f32 + 0.5) / 64.;
        let ix = (u * 4.).floor().clamp(0., 3.) as usize;
        let iy = (v * 4.).floor().clamp(0., 3.) as usize;
        // Reference colours come from the fixture definition, not the
        // producer's decoded image or the GPU-bound material data.
        let p = [
            (20 + ix * 40 + iy * 10) as u8,
            (15 + ix * 15 + iy * 45) as u8,
            (240 - ix * 30 - iy * 20) as u8,
        ];
        let factor = [0.35, 0.65, 0.8];
        std::array::from_fn::<u8, 3, _>(|i| encode_srgb(linear_srgb(p[i]) * factor[i] * 0.75))
    };
    let probes = [(5, 9), (17, 25), (42, 51), (60, 2)];
    let pixels = draw(&a.model, &mut pipeline, &mut store);
    let mut maximum = 0;
    for (x, y) in probes {
        let got = pixels.get_pixel(x, y).0;
        let expect = expected(x, y);
        let error = (0..3).map(|i| got[i].abs_diff(expect[i])).max().unwrap();
        maximum = maximum.max(error);
        assert!(error <= 2, "{x},{y}: GPU {got:?} CPU {expect:?}");
    }
    let mut frozen = a.model.clone();
    frozen.source_sha256 = "frozen-uv-control".into();
    frozen.meshes[0]
        .visual
        .as_mut()
        .unwrap()
        .uv
        .fill([0.125; 2]);
    let mut rejected = 0;
    for wrong in [&b.model, &frozen] {
        let pixels = draw(wrong, &mut pipeline, &mut store);
        let mismatches = probes
            .into_iter()
            .filter(|&(x, y)| {
                let p = pixels.get_pixel(x, y).0;
                let e = expected(x, y);
                (0..3).any(|i| p[i].abs_diff(e[i]) > 10)
            })
            .count();
        assert!(
            mismatches >= 3,
            "missing KHR transform / frozen UV must fail the pixel oracle"
        );
        rejected += 1;
    }
    let report = json!({"probes":probes.len(),"max_error":maximum,"tolerance":2,"mutation_rejections":rejected,"collision_bounds_unchanged":true,"node_association_recoverable":true});
    println!("UI005 gltf-transform: {report}");
    if let Some(root) = std::env::var_os("UI005_REPORT_DIR") {
        let root = PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("gltf-transform.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn gltf_mipmap_filters_have_explicit_base_filter_normalization() {
    let root = temp();
    let path = root.join("sampler.glb");
    for (input, base) in [
        (9728, 9728),
        (9729, 9729),
        (9984, 9728),
        (9985, 9729),
        (9986, 9728),
        (9987, 9729),
    ] {
        let (bytes, png) = textured_quad_fixture(false, input);
        std::fs::write(&path, bytes).unwrap();
        std::fs::write(root.join("colours.png"), png).unwrap();
        let a = assets::import(&path, &Registry::new()).unwrap();
        assert_eq!(
            a.model.meshes[0].visual.as_ref().unwrap().sampling.min,
            base,
            "{input}"
        );
    }
}
