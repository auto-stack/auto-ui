//! PLAN-005 public import/storage/render closure probes. Required inputs are
//! original fixed-commit assets, not stripped or recoloured derivatives.
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    document::Document,
    host::Host,
    storage::{Fault, Saved, Storage},
};
use serde_json::{json, Value};
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
