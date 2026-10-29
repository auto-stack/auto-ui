//! PLAN-006 V-01/S-01 production admission and persistent content closure.
use agp::assets::node_animation::{channel_allocation_count, Animation};
use autographics_auto_driver::authoring::{
    assets::{self, Asset, Registry},
    document::{Document, Kind, Object},
    session::Editor,
    storage::{Fault, Saved, Storage},
    textured_models,
};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/textured_models/models")
}
fn temp() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let p = std::env::temp_dir().join(format!(
        "ui006-resources-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(p.join("Textures")).unwrap();
    std::fs::copy(
        samples().join("Textures/colormap.png"),
        p.join("Textures/colormap.png"),
    )
    .unwrap();
    p
}
fn chunks() -> (Value, Vec<u8>) {
    let b = std::fs::read(samples().join("character.glb")).unwrap();
    let n = u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize;
    (
        serde_json::from_slice(&b[20..20 + n]).unwrap(),
        b[28 + n..].to_vec(),
    )
}
fn pack(j: &Value, bin: &[u8]) -> Vec<u8> {
    let mut text = serde_json::to_vec(j).unwrap();
    while text.len() % 4 != 0 {
        text.push(b' ')
    }
    let mut b = b"glTF".to_vec();
    b.extend(2u32.to_le_bytes());
    b.extend(((28 + text.len() + bin.len()) as u32).to_le_bytes());
    b.extend((text.len() as u32).to_le_bytes());
    b.extend(b"JSON");
    b.extend(text);
    b.extend((bin.len() as u32).to_le_bytes());
    b.extend(b"BIN\0");
    b.extend(bin);
    b
}
fn put(root: &Path, j: &Value, bin: &[u8]) -> PathBuf {
    let p = root.join("character.glb");
    std::fs::write(&p, pack(j, bin)).unwrap();
    p
}
fn location(j: &Value, accessor: usize) -> usize {
    let a = &j["accessors"][accessor];
    j["bufferViews"][a["bufferView"].as_u64().unwrap() as usize]["byteOffset"]
        .as_u64()
        .unwrap_or(0) as usize
        + a["byteOffset"].as_u64().unwrap_or(0) as usize
}
fn curve(j: &mut Value, bin: &mut Vec<u8>, count: usize) {
    let views = j["bufferViews"].as_array_mut().unwrap();
    let v = views.len();
    let offset = bin.len();
    for i in 0..count {
        bin.extend((i as f32).to_le_bytes());
    }
    views.push(json!({"buffer":0,"byteOffset":offset,"byteLength":count*4}));
    let offset = bin.len();
    for _ in 0..count {
        for value in [0f32, 0., 0.] {
            bin.extend(value.to_le_bytes());
        }
    }
    views.push(json!({"buffer":0,"byteOffset":offset,"byteLength":count*12}));
    let accessors = j["accessors"].as_array_mut().unwrap();
    let a = accessors.len();
    accessors.push(json!({"bufferView":v,"componentType":5126,"count":count,"type":"SCALAR","min":[0.],"max":[count-1]}));
    accessors.push(json!({"bufferView":v+1,"componentType":5126,"count":count,"type":"VEC3"}));
    j["buffers"][0]["byteLength"] = json!(bin.len());
    j["animations"] = json!([{"name":"finite","samplers":[{"input":a,"output":a+1}],"channels":[{"sampler":0,"target":{"node":1,"path":"translation"}}]}]);
}
fn scene(a: Arc<Asset>) -> (Document, Registry) {
    let mut d = Document::empty("ui006-admission".into());
    d.schema = "autographics.scene/6".into();
    d.assets.push(a.identity.clone());
    let mut o = Object::new(1, Kind::Model);
    o.resource = Some(a.identity.key());
    d.objects.push(o);
    let mut r = Registry::new();
    assets::insert(&mut r, a).unwrap();
    (d, r)
}

#[test]
fn production_rejects_malformed_and_unsupported_tracks_before_payload_or_decode() {
    let (j, bin) = chunks();
    let root = temp();
    let input = j["animations"][0]["samplers"][0]["input"].as_u64().unwrap() as usize;
    let output = j["animations"][0]["samplers"][0]["output"]
        .as_u64()
        .unwrap() as usize;
    let cases: Vec<(&str, Box<dyn Fn(&mut Value)>, &str)> = vec![
        (
            "cubic",
            Box::new(|j| j["animations"][0]["samplers"][0]["interpolation"] = json!("CUBICSPLINE")),
            "unsupported:",
        ),
        (
            "unused cubic",
            Box::new(|j| {
                let mut s = j["animations"][0]["samplers"][0].clone();
                s["interpolation"] = json!("CUBICSPLINE");
                j["animations"][0]["samplers"]
                    .as_array_mut()
                    .unwrap()
                    .push(s);
            }),
            "unsupported:",
        ),
        (
            "weights",
            Box::new(|j| j["animations"][0]["channels"][0]["target"]["path"] = json!("weights")),
            "unsupported:",
        ),
        (
            "skin",
            Box::new(|j| j["skins"] = json!([{}])),
            "unsupported:",
        ),
        (
            "node skin",
            Box::new(|j| j["nodes"][0]["skin"] = json!(0)),
            "unsupported:",
        ),
        (
            "node weights",
            Box::new(|j| j["nodes"][0]["weights"] = json!([0])),
            "unsupported:",
        ),
        (
            "mesh weights",
            Box::new(|j| j["meshes"][0]["weights"] = json!([])),
            "unsupported:",
        ),
        (
            "morph targets",
            Box::new(|j| j["meshes"][0]["primitives"][0]["targets"] = json!([{"POSITION":0}])),
            "unsupported:",
        ),
        (
            "duplicate target",
            Box::new(|j| {
                let c = j["animations"][0]["channels"][0].clone();
                j["animations"][0]["channels"]
                    .as_array_mut()
                    .unwrap()
                    .push(c);
            }),
            "invalid:",
        ),
        (
            "wrong output count",
            Box::new(move |j| j["accessors"][output]["count"] = json!(1)),
            "invalid:",
        ),
        (
            "wrong input type",
            Box::new(move |j| j["accessors"][input]["type"] = json!("VEC2")),
            "invalid:",
        ),
        (
            "normalized output",
            Box::new(move |j| j["accessors"][output]["normalized"] = json!(true)),
            "invalid:",
        ),
        (
            "bad stride",
            Box::new(move |j| {
                let v = j["accessors"][input]["bufferView"].as_u64().unwrap() as usize;
                j["bufferViews"][v]["byteStride"] = json!(2);
            }),
            "invalid:",
        ),
        (
            "bad span",
            Box::new(move |j| j["accessors"][input]["byteOffset"] = json!(u64::MAX)),
            "invalid:",
        ),
        (
            "target node",
            Box::new(|j| j["animations"][0]["channels"][0]["target"]["node"] = json!(128)),
            "invalid:",
        ),
        (
            "cycle",
            Box::new(|j| j["nodes"][7]["children"] = json!([0])),
            "invalid:",
        ),
        (
            "multiple parents",
            Box::new(|j| j["nodes"][0]["children"] = json!([1, 7])),
            "invalid:",
        ),
        (
            "zero scale",
            Box::new(|j| j["nodes"][0]["scale"] = json!([0, 1, 1])),
            "invalid:",
        ),
        (
            "negative scale",
            Box::new(|j| j["nodes"][0]["scale"] = json!([-1, 1, 1])),
            "invalid:",
        ),
        (
            "invalid interpolation type",
            Box::new(|j| j["animations"][0]["samplers"][0]["interpolation"] = json!(1)),
            "invalid:",
        ),
        (
            "nodes129",
            Box::new(|j| j["nodes"] = json!(vec![json!({}); 129])),
            "resource limit:",
        ),
        (
            "clips17",
            Box::new(|j| {
                let c = j["animations"][0].clone();
                j["animations"] = json!(vec![c; 17]);
            }),
            "resource limit:",
        ),
        (
            "channels129",
            Box::new(|j| {
                let c = j["animations"][0]["channels"][0].clone();
                j["animations"][0]["channels"] = json!(vec![c; 129]);
            }),
            "resource limit:",
        ),
    ];
    for (name, mutate, prefix) in &cases {
        let mut j = j.clone();
        mutate(&mut j);
        let path = put(&root, &j, &bin);
        let before = (channel_allocation_count(), assets::decode_attempts());
        let error = assets::import_animated(&path, &Registry::new()).unwrap_err();
        assert!(error.starts_with(prefix), "{name}: {error}");
        assert_eq!(
            before,
            (channel_allocation_count(), assets::decode_attempts()),
            "{name} must fail before retained payload/decode"
        );
    }
    let rotation = j["animations"][0]["samplers"][1]["output"]
        .as_u64()
        .unwrap() as usize;
    let scale = j["animations"][0]["samplers"][2]["output"]
        .as_u64()
        .unwrap() as usize;
    for (name, offset, value) in [
        ("duplicate time", location(&j, input) + 4, 0f32),
        ("negative time", location(&j, input), -1.),
        ("infinite time", location(&j, input), f32::INFINITY),
        ("nan output", location(&j, output), f32::NAN),
        ("quaternion", location(&j, rotation) + 12, 0.),
        ("negative scale output", location(&j, scale), -1.),
    ] {
        let mut bin = bin.clone();
        bin[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        let path = put(&root, &j, &bin);
        let before = (channel_allocation_count(), assets::decode_attempts());
        let error = assets::import_animated(&path, &Registry::new()).unwrap_err();
        assert!(error.starts_with("invalid:"), "{name}: {error}");
        assert_eq!(
            before,
            (channel_allocation_count(), assets::decode_attempts()),
            "{name}"
        );
    }
    let path = put(&root, &j, &bin);
    let before = channel_allocation_count();
    let accepted = assets::import_animated(&path, &Registry::new()).unwrap();
    assert_eq!(channel_allocation_count() - before, 44);
    assert_eq!(accepted.animation.as_ref().unwrap().clips.len(), 4);
    println!("production admission {} structural + 6 binary negatives, all zero allocations/decodes; original positive 44 actual channel allocations",cases.len());
}

#[test]
fn production_accepts_exact_limits_step_and_untextured_hierarchy() {
    let root = temp();
    let (mut j, mut bin) = chunks();
    curve(&mut j, &mut bin, 8192);
    let path = put(&root, &j, &bin);
    let a = assets::import_animated(&path, &Registry::new()).unwrap();
    assert_eq!(
        a.animation.as_ref().unwrap().clips[0].channels[0]
            .times
            .len(),
        8192
    );
    assert_eq!(
        Animation::preflight(&j, &bin).unwrap(),
        a.animation.as_ref().unwrap().cpu_bytes
    );
    let (mut j, mut bin) = chunks();
    curve(&mut j, &mut bin, 8193);
    let path = put(&root, &j, &bin);
    let before = channel_allocation_count();
    assert!(assets::import_animated(&path, &Registry::new())
        .unwrap_err()
        .starts_with("resource limit:"));
    assert_eq!(before, channel_allocation_count());
    let (mut j, mut bin) = chunks();
    curve(&mut j, &mut bin, 2);
    let c = j["animations"][0].clone();
    j["animations"] = json!(vec![c; 16]);
    assert_eq!(
        assets::import_animated(&put(&root, &j, &bin), &Registry::new())
            .unwrap()
            .animation
            .as_ref()
            .unwrap()
            .clips
            .len(),
        16
    );
    let (mut j, mut bin) = chunks();
    curve(&mut j, &mut bin, 2);
    for _ in 8..128 {
        j["nodes"].as_array_mut().unwrap().push(json!({}));
    }
    for i in 8..128 {
        j["scenes"][0]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!(i));
    }
    j["animations"][0]["channels"] = json!((0..128)
        .map(|node| json!({"sampler":0,"target":{"node":node,"path":"translation"}}))
        .collect::<Vec<_>>());
    let a = assets::import_animated(&put(&root, &j, &bin), &Registry::new()).unwrap();
    assert_eq!(a.animation.as_ref().unwrap().nodes.len(), 128);
    assert_eq!(a.animation.as_ref().unwrap().clips[0].channels.len(), 128);
    // An ordinary two-layer untextured TRS fixture uses the same public entry,
    // not a loader or transformed substitute that silently strips animations.
    let (mut j, bin) = chunks();
    j["animations"][0]["samplers"][0]["interpolation"] = json!("STEP");
    for name in [
        "images",
        "textures",
        "samplers",
        "extensionsUsed",
        "extensionsRequired",
    ] {
        j.as_object_mut().unwrap().remove(name);
    }
    for m in j["materials"].as_array_mut().unwrap() {
        m["pbrMetallicRoughness"]
            .as_object_mut()
            .unwrap()
            .remove("baseColorTexture");
    }
    let a = assets::import_animated(&put(&root, &j, &bin), &Registry::new()).unwrap();
    assert!(a.model_images.is_empty());
    assert!(a.identity.closure.as_ref().unwrap().images.is_empty());
    let anim = a.animation.as_ref().unwrap();
    assert_eq!(anim.nodes[7].parent, Some(4));
    assert_eq!(
        anim.clips[0].channels[0].interpolation,
        agp::assets::node_animation::Interpolation::Step
    );
    let (d, r) = scene(a);
    let mut e = Editor::with_assets(d, r).unwrap();
    let mut s = Storage::default();
    let entry = root.join("untextured.agscene");
    assert_eq!(
        s.save(&mut e, &entry, Fault::None).unwrap(),
        Saved::Confirmed
    );
    assert!(Storage::load_assets(&entry, &Registry::new()).is_ok());
    println!("production controls:8192/8193 keys,16/17 clips,128 nodes/channels,STEP,untextured hierarchy save/reopen");
}

#[test]
fn full_resident_union_rejects_before_channel_allocation_and_cached_import_shares_payload() {
    let root = temp();
    let mut registry = Registry::new();
    for color in 0..4u8 {
        let rgba = [color * 40, 0, 255, 255].repeat(2048 * 2048);
        let png = agp::assets::encode_png_rgba(2048, 2048, &rgba).unwrap();
        let path = root.join(format!("resident{color}.png"));
        std::fs::write(&path, png).unwrap();
        let a = assets::import_png(&path, &registry).unwrap();
        assets::insert(&mut registry, a).unwrap();
    }
    assert_eq!(textured_models::usage(&registry).2, assets::MAX_GEOMETRY);
    let before = (channel_allocation_count(), assets::decode_attempts());
    let error = assets::import_animated(&samples().join("character.glb"), &registry).unwrap_err();
    assert!(error.starts_with("resource limit:"), "{error}");
    assert_eq!(
        before,
        (channel_allocation_count(), assets::decode_attempts()),
        "full closure gate precedes any channel or image/geometry decode"
    );
    let key = registry.keys().next().unwrap().clone();
    registry.remove(&key);
    let before = channel_allocation_count();
    let a = assets::import_animated(&samples().join("character.glb"), &registry).unwrap();
    assert_eq!(channel_allocation_count() - before, 44);
    assets::insert(&mut registry, a.clone()).unwrap();
    let before = (channel_allocation_count(), assets::decode_attempts());
    let cached = assets::import_animated(&samples().join("character.glb"), &registry).unwrap();
    assert!(Arc::ptr_eq(&a, &cached));
    assert!(Arc::ptr_eq(
        a.animation.as_ref().unwrap(),
        cached.animation.as_ref().unwrap()
    ));
    assert_eq!(
        before,
        (channel_allocation_count(), assets::decode_attempts())
    );
    println!("resident64MiB rejection zero channel allocations;48MiB accepted;cached44-channel immutable Arc shared");
}

#[test]
fn animated_closure_save_faults_cached_dependencies_and_profile_forgery_recover() {
    let a = assets::import_animated(&samples().join("character.glb"), &Registry::new()).unwrap();
    for fault in [
        Fault::AssetAfterPrepare,
        Fault::AssetAfterInstall,
        Fault::BeforeReplace,
        Fault::AfterReplaceRead,
        Fault::ClosureAfterReplace,
    ] {
        let root = temp();
        let entry = root.join("scene.agscene");
        let (d, r) = scene(a.clone());
        let mut e = Editor::with_assets(d, r.clone()).unwrap();
        let mut s = Storage::default();
        let result = s.save(&mut e, &entry, fault);
        if [Fault::AfterReplaceRead, Fault::ClosureAfterReplace].contains(&fault) {
            assert_eq!(result.unwrap(), Saved::VerificationPending);
            assert!(s.unverified());
            assert!(s.path.is_none());
            let image = &a.identity.closure.as_ref().unwrap().images[0];
            let blob = root.join(format!("assets/{}.png", image.sha256));
            let good = std::fs::read(&blob).unwrap();
            std::fs::write(&blob, b"damaged").unwrap();
            assert!(s.verify(&mut e).is_err());
            assert!(s.unverified());
            std::fs::write(&blob, good).unwrap();
            assert!(s.verify(&mut e).unwrap());
        } else {
            assert!(result.is_err());
            assert!(!entry.exists());
            assert_eq!(
                s.save(&mut e, &entry, Fault::None).unwrap(),
                Saved::Confirmed
            );
        }
        let (_, loaded, _) = Storage::load_assets(&entry, &r).unwrap();
        assert!(Arc::ptr_eq(&loaded[&a.identity.key()], &a));
        let image = &a.identity.closure.as_ref().unwrap().images[0];
        let blob = root.join(format!("assets/{}.png", image.sha256));
        let good = std::fs::read(&blob).unwrap();
        std::fs::remove_file(&blob).unwrap();
        assert!(Storage::load_assets(&entry, &r).is_err());
        std::fs::write(&blob, b"wrong").unwrap();
        assert!(Storage::load_assets(&entry, &r).is_err());
        std::fs::write(&blob, good).unwrap();
        assert!(Storage::load_assets(&entry, &r).is_ok());
        if fault == Fault::AssetAfterPrepare {
            let root_blob = root.join(format!("assets/{}.glb", a.identity.sha256));
            let good = std::fs::read(&root_blob).unwrap();
            let mut damaged = good.clone();
            let last = damaged.len() - 1;
            damaged[last] ^= 1;
            std::fs::write(&root_blob, damaged).unwrap();
            assert!(
                Storage::load_assets(&entry, &r)
                    .err()
                    .unwrap()
                    .contains("root identity"),
                "warm registry cannot bypass original GLB identity"
            );
            std::fs::write(&root_blob, good).unwrap();
            assert!(Storage::load_assets(&entry, &r).is_ok());
        }
        let good = std::fs::read(&entry).unwrap();
        let mut forged: Value = serde_json::from_slice(&good).unwrap();
        forged["assets"][0]["closure"]["node_animation"] = json!(false);
        assert!(
            Document::decode(&serde_json::to_vec(&forged).unwrap()).is_err(),
            "profile switch cannot reuse animation content id"
        );
        // Rehash the forged static domain as well: the actual loader must still
        // reject the animated raw bytes, even with a warm original payload.
        let images: &Value = &forged["assets"][0]["closure"]["images"];
        let id = agp::renderer::capture::sha256_hex(
            &serde_json::to_vec(&(
                "autographics.textured/1",
                &a.identity.sha256,
                a.bytes.len(),
                images,
            ))
            .unwrap(),
        );
        forged["assets"][0]["closure"]["id"] = json!(id);
        forged["objects"][0]["resource"] = json!(format!("closure:{id}"));
        std::fs::write(&entry, serde_json::to_vec(&forged).unwrap()).unwrap();
        assert!(Storage::load_assets(&entry, &r)
            .err()
            .unwrap()
            .contains("unsupported: animations"));
        std::fs::write(&entry, good).unwrap();
        let (_, restored, _) = Storage::load_assets(&entry, &Registry::new()).unwrap();
        assert_eq!(
            restored[&a.identity.key()]
                .animation
                .as_ref()
                .unwrap()
                .clips
                .len(),
            4
        );
    }
    println!("5 save fault/retry/pending verify flows;warm missing/damaged/repaired PNG;forged flag+rehash static domain rejected;fresh restore4clips");
}

#[test]
fn pose_metadata_wire_and_hidden_potential_slots_share_the_existing_cpu_union() {
    use autographics_auto_driver::authoring::{hierarchy, runtime_node_animation as visual};
    let a = assets::import_animated(&samples().join("character.glb"), &Registry::new()).unwrap();
    let (mut d, mut r) = scene(a);
    let order = hierarchy::ordered(&d).unwrap();
    let t = visual::targets(&d, &order, &r).unwrap();
    let reserve = visual::pose_reservation(&t);
    let root = temp();
    let add = |r: &mut Registry, d: &mut Document, width: usize, height: usize, color: u8| {
        let rgba = [color, 0, 255, 255].repeat(width * height);
        let path = root.join(format!("pose-{width}-{height}-{color}.png"));
        std::fs::write(
            &path,
            agp::assets::encode_png_rgba(width as u32, height as u32, &rgba).unwrap(),
        )
        .unwrap();
        let image = assets::import_png(&path, r).unwrap();
        let key = image.identity.key();
        d.assets.push(image.identity.clone());
        assets::insert(r, image).unwrap();
        key
    };
    for color in 0..3 {
        add(&mut r, &mut d, 2048, 2048, color * 30);
    }
    let remaining = assets::MAX_GEOMETRY - textured_models::usage(&r).2;
    let old_reserve = (8 * (48 + 64 * 4)) * 9;
    let wanted = (old_reserve + reserve) / 2;
    let height = (remaining - wanted) / (2048 * 4);
    let key = add(&mut r, &mut d, 2048, height, 120);
    let extra = (assets::MAX_GEOMETRY - textured_models::usage(&r).2 - wanted) / 4;
    if extra > 0 {
        add(&mut r, &mut d, 1, extra, 122);
    }
    let gap = assets::MAX_GEOMETRY - textured_models::usage(&r).2;
    assert!(
        gap < reserve && gap > old_reserve,
        "fixture must fit assets but fail actual pose reservation"
    );
    d.validate().unwrap();
    assets::check_registry(&r).unwrap();
    assert!(visual::targets(&d, &order, &r)
        .err()
        .is_some_and(|e| e.contains("pose CPU union")));
    d.objects[0].visible = false;
    assert!(
        visual::targets(&d, &order, &r).is_err(),
        "hiding the template cannot erase eight potential slot poses"
    );
    r.remove(&key);
    d.assets.retain(|a| a.key() != key);
    add(&mut r, &mut d, 2048, height - 8, 121);
    assert!(visual::targets(&d, &order, &r).is_ok());
    println!("CPU pose reservation={reserve}B; asset-only headroom={gap}B rejects including hidden; +65536B headroom accepts");
}
