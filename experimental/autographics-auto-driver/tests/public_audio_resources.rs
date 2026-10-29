//! PLAN-012 T-03: accepted audio identity, shared quotas and durable repair.
use autographics_auto_driver::authoring::{
    assets::{self, Asset, Registry},
    document::Document,
    public_audio::{self, CueRef},
    session::{Edit, Editor},
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
fn temp() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "ui012-resource-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn wav(seed: i16, frames: usize) -> Vec<u8> {
    let mut b = Vec::with_capacity(44 + frames * 2);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + frames as u32 * 2).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&22050u32.to_le_bytes());
    b.extend_from_slice(&44100u32.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(frames as u32 * 2).to_le_bytes());
    for _ in 0..frames {
        b.extend_from_slice(&seed.to_le_bytes());
    }
    b
}
fn audio(seed: i16, registry: &Registry) -> Arc<Asset> {
    public_audio::import_raw(
        wav(seed, 32).into(),
        registry,
        Some(format!("sound{seed}.wav")),
    )
    .unwrap()
}
fn scene(asset: &Arc<Asset>) -> Document {
    let mut doc = Document::empty("audio-resources".into());
    doc.schema = "autographics.scene/7".into();
    doc.assets.push(asset.identity.clone());
    doc.cues = Some(vec![
        CueRef {
            name: "first".into(),
            asset: asset.identity.key(),
            start_frame: 1,
            end_frame: 16,
            gain: 0.5,
        },
        CueRef {
            name: "second".into(),
            asset: asset.identity.key(),
            start_frame: 16,
            end_frame: 32,
            gain: 1.0,
        },
    ]);
    doc
}
fn registry(asset: Arc<Asset>) -> Registry {
    let mut r = Registry::new();
    assets::insert(&mut r, asset).unwrap();
    r
}
fn accept(editor: &mut Editor) {
    while !editor.pump().unwrap() {}
    editor.commit().unwrap();
}

#[test]
fn versioned_cues_reject_shape_span_gain_kind_and_integer_drift() {
    let asset = audio(4000, &Registry::new());
    let good = scene(&asset);
    let r = registry(asset.clone());
    let bytes = good.encode().unwrap();
    assert_eq!(Document::decode(&bytes).unwrap(), good);
    let resolved = public_audio::resolve_cues(&good, &r).unwrap();
    assert!(Arc::ptr_eq(&resolved[0].pcm, &resolved[1].pcm));
    let input: Value = serde_json::from_slice(&bytes).unwrap();
    let mut cases = vec![];
    for (field, value) in [
        ("start_frame", json!(-1)),
        ("start_frame", json!(1.5)),
        ("start_frame", json!(16)),
        ("end_frame", json!(33)),
        ("end_frame", json!(0)),
        ("gain", json!(-0.01)),
        ("gain", json!(1.01)),
        ("name", json!("")),
        ("name", json!("x".repeat(129))),
        ("asset", json!("sha256:missing")),
        ("unknown", json!(1)),
    ] {
        let mut v = input.clone();
        v["cues"][0][field] = value;
        cases.push(v);
    }
    let mut duplicate = input.clone();
    duplicate["cues"][1]["name"] = duplicate["cues"][0]["name"].clone();
    cases.push(duplicate);
    let mut too_many = input.clone();
    too_many["cues"] = json!((0..9)
        .map(|i| {
            let mut c = input["cues"][0].clone();
            c["name"] = json!(format!("cue{i}"));
            c
        })
        .collect::<Vec<_>>());
    cases.push(too_many);
    for schema in [
        "autographics.scene/1",
        "autographics.scene/4",
        "autographics.scene/5",
        "autographics.scene/6",
    ] {
        let mut old = input.clone();
        old["schema"] = json!(schema);
        cases.push(old);
        let mut old = serde_json::to_value(Document::empty("old".into())).unwrap();
        old["schema"] = json!(schema);
        old["cues"] = json!([]);
        cases.push(old);
    }
    for field in ["rate", "channels", "frames"] {
        let mut v = input.clone();
        v["assets"][0]["audio"][field] = json!(1.5);
        cases.push(v);
    }
    for (field, value) in [
        ("rate", json!(96000)),
        ("channels", json!(3)),
        ("frames", json!(220501)),
    ] {
        let mut v = input.clone();
        v["assets"][0]["audio"][field] = value;
        cases.push(v);
    }
    let mut missing = input.clone();
    missing.as_object_mut().unwrap().remove("cues");
    cases.push(missing);
    let mut null = input.clone();
    null["cues"] = Value::Null;
    cases.push(null);
    let mut wrong_kind = input.clone();
    wrong_kind["assets"][0]
        .as_object_mut()
        .unwrap()
        .remove("audio");
    wrong_kind["assets"][0]["format"] = json!("glb");
    cases.push(wrong_kind);
    for (i, bad) in cases.iter().enumerate() {
        assert!(
            Document::decode(&serde_json::to_vec(bad).unwrap()).is_err(),
            "case{i}: {bad}"
        );
    }
    let mut nan = good.clone();
    nan.cues.as_mut().unwrap()[0].gain = f32::NAN;
    assert!(nan.validate().is_err());
    let mut infinity = good.clone();
    infinity.cues.as_mut().unwrap()[0].gain = f32::INFINITY;
    assert!(infinity.validate().is_err());
    let mut eight = good.clone();
    eight.cues = Some(
        (0..8)
            .map(|i| {
                let mut c = good.cues.as_ref().unwrap()[0].clone();
                c.name = format!("c{i}");
                c
            })
            .collect(),
    );
    eight.validate().unwrap();
    let shared = public_audio::resolve_cues(&eight, &r).unwrap();
    assert_eq!(shared.len(), 8);
    assert!(shared
        .iter()
        .all(|c| Arc::ptr_eq(&c.pcm, asset.audio.as_ref().unwrap())));
    println!(
        "schema/cue negative controls={} plus NaN/Inf;8accepted cues share1PCM",
        cases.len()
    );
}

#[test]
fn declared_audio_metadata_and_raw_hash_are_verified_even_with_warm_cache() {
    let root = temp();
    let asset = audio(4000, &Registry::new());
    let doc = scene(&asset);
    let r = registry(asset.clone());
    let mut editor = Editor::with_assets(doc, r.clone()).unwrap();
    let mut storage = Storage::default();
    let entry = root.join("scene.agscene");
    storage.save(&mut editor, &entry, Fault::None).unwrap();
    let original = std::fs::read(&entry).unwrap();
    let v: Value = serde_json::from_slice(&original).unwrap();
    for (field, value) in [
        ("rate", json!(44100)),
        ("channels", json!(2)),
        ("frames", json!(33)),
    ] {
        let mut changed = v.clone();
        changed["assets"][0]["audio"][field] = value;
        std::fs::write(&entry, serde_json::to_vec(&changed).unwrap()).unwrap();
        for cache in [&Registry::new(), &r] {
            assert!(
                Storage::load_assets(&entry, cache)
                    .err()
                    .unwrap()
                    .contains("identity"),
                "{field}"
            );
        }
    }
    std::fs::write(&entry, &original).unwrap();
    let blob = root.join(format!("assets/{}.wav", asset.identity.sha256));
    let accepted = std::fs::read(&blob).unwrap();
    for mode in 0..3 {
        if mode == 0 {
            std::fs::remove_file(&blob).unwrap();
        } else {
            let mut bad = accepted.clone();
            if mode == 1 {
                bad.pop();
            } else {
                bad[44] ^= 1;
            }
            std::fs::write(&blob, bad).unwrap();
        }
        assert!(
            Storage::load_assets(&entry, &r).is_err(),
            "warm cache must read/verify disk blob mode{mode}"
        );
        std::fs::write(&blob, &accepted).unwrap();
        assert!(Storage::load_assets(&entry, &r).is_ok());
    }
    let (_, loaded, _) = Storage::load_assets(&entry, &Registry::new()).unwrap();
    assert_eq!(
        loaded[&asset.identity.key()]
            .audio
            .as_ref()
            .unwrap()
            .frames
            .as_ref(),
        asset.audio.as_ref().unwrap().frames.as_ref()
    );
    assert!(Arc::ptr_eq(
        &Storage::load_assets(&entry, &r).unwrap().1[&asset.identity.key()],
        &asset
    ));
}

#[test]
fn save_faults_retry_and_pending_verification_heal_before_cold_reopen() {
    for fault in [
        Fault::AssetAfterPrepare,
        Fault::AssetAfterInstall,
        Fault::BeforeReplace,
        Fault::AfterReplaceRead,
        Fault::ClosureAfterReplace,
    ] {
        let root = temp();
        let asset = audio(7000, &Registry::new());
        let r = registry(asset.clone());
        let mut editor = Editor::with_assets(scene(&asset), r.clone()).unwrap();
        let entry = root.join("scene.agscene");
        let mut storage = Storage::default();
        let result = storage.save(&mut editor, &entry, fault);
        if [Fault::AfterReplaceRead, Fault::ClosureAfterReplace].contains(&fault) {
            assert_eq!(result.unwrap(), Saved::VerificationPending);
            assert!(storage.unverified());
            assert!(storage.path.is_none());
            let blob = root.join(format!("assets/{}.wav", asset.identity.sha256));
            let good = std::fs::read(&blob).unwrap();
            std::fs::write(&blob, b"broken").unwrap();
            assert!(storage.verify(&mut editor).is_err());
            assert!(storage.unverified());
            assert!(storage.path.is_none());
            std::fs::write(&blob, &good).unwrap();
            assert!(storage.verify(&mut editor).unwrap());
            assert!(!storage.unverified());
        } else {
            assert!(result.is_err());
            assert!(!entry.exists());
            assert_eq!(
                storage.save(&mut editor, &entry, Fault::None).unwrap(),
                Saved::Confirmed
            );
        }
        let (doc, loaded, _) = Storage::load_assets(&entry, &Registry::new()).unwrap();
        assert_eq!(doc, editor.document);
        assert_eq!(public_audio::resolve_cues(&doc, &loaded).unwrap().len(), 2);
        // Saving to another root uses accepted bytes, never the original path.
        let next = root.join("moved");
        std::fs::create_dir(&next).unwrap();
        storage
            .save(&mut editor, &next.join("scene.agscene"), Fault::None)
            .unwrap();
        assert!(Storage::load_assets(&next.join("scene.agscene"), &Registry::new()).is_ok());
    }
}

#[test]
fn history_pending_and_current_share_eight_raw_slots_and_duplicate_import_is_free() {
    let mut editor = Editor::new(Document::empty("history".into())).unwrap();
    let root = temp();
    let mut first = None;
    for i in 0..8 {
        let asset = audio(i + 1, &editor.assets);
        if first.is_none() {
            first = Some(asset.clone());
        }
        editor
            .begin(
                editor.stamp,
                Edit::Audio {
                    asset: Some(asset),
                    cues: vec![],
                },
            )
            .unwrap();
        assert_eq!(editor.document.assets.len(), i as usize);
        assert_eq!(editor.candidate_assets().len(), i as usize + 1);
        accept(&mut editor);
    }
    assert_eq!(textured_models::usage(&editor.assets).0, 8);
    let decodes = assets::decode_attempts();
    let path = root.join("same.wav");
    std::fs::write(&path, first.as_ref().unwrap().bytes.as_ref()).unwrap();
    let cached = public_audio::import(&path, &editor.assets).unwrap();
    assert!(Arc::ptr_eq(&cached, first.as_ref().unwrap()));
    assert_eq!(assets::decode_attempts(), decodes);
    let ninth = root.join("ninth.wav");
    std::fs::write(&ninth, wav(99, 32)).unwrap();
    assert!(public_audio::import(&ninth, &editor.assets)
        .unwrap_err()
        .contains("resource limit"));
    assert_eq!(assets::decode_attempts(), decodes);
    editor.begin(editor.stamp, Edit::Undo).unwrap();
    accept(&mut editor);
    assert_eq!(editor.document.assets.len(), 7);
    assert_eq!(editor.assets.len(), 8, "redo/history owns eighth payload");
    assert!(public_audio::import(&ninth, &editor.assets).is_err());
    assert_eq!(assets::decode_attempts(), decodes);
    editor.begin(editor.stamp, Edit::Redo).unwrap();
    accept(&mut editor);
    assert_eq!(editor.document.assets.len(), 8);
    editor.close();
    assert!(editor.assets.is_empty());
}

#[test]
fn raw_total_thirty_two_mib_boundary_is_shared_and_duplicate_needs_no_new_snapshot() {
    let root = temp();
    let mut r = Registry::new();
    let mut first = None;
    for i in 0..4u8 {
        let mut bytes = wav(1 + i as i16, 1);
        bytes.extend_from_slice(b"JUNK");
        let n = 8 * 1024 * 1024 - bytes.len() - 4;
        bytes.extend_from_slice(&(n as u32).to_le_bytes());
        bytes.resize(8 * 1024 * 1024, i);
        let length = bytes.len();
        bytes[4..8].copy_from_slice(&(length as u32 - 8).to_le_bytes());
        let path = root.join(format!("large{i}.wav"));
        std::fs::write(&path, &bytes).unwrap();
        let asset = public_audio::import(&path, &r).unwrap();
        if first.is_none() {
            first = Some((path, asset.clone()));
        }
        assets::insert(&mut r, asset).unwrap();
    }
    assert_eq!(textured_models::usage(&r).1, 32 * 1024 * 1024);
    let decodes = assets::decode_attempts();
    let (path, asset) = first.unwrap();
    assert!(Arc::ptr_eq(
        &public_audio::import(&path, &r).unwrap(),
        &asset
    ));
    let extra = root.join("extra.wav");
    std::fs::write(&extra, wav(99, 1)).unwrap();
    assert!(public_audio::import(&extra, &r).is_err());
    assert_eq!(assets::decode_attempts(), decodes);
}

#[test]
fn real_resident_image_union_rejects_audio_peak_before_decode_and_recovers_when_space_freed() {
    let mut r = Registry::new();
    for color in 0..3u8 {
        let pixels = [color * 40, 0, 255, 255].repeat(2048 * 2048);
        let png = agp::assets::encode_png_rgba(2048, 2048, &pixels).unwrap();
        let asset = assets::decode_png_in(png, None, &r).unwrap();
        assets::insert(&mut r, asset).unwrap();
    }
    assert_eq!(textured_models::usage(&r).2, 48 * 1024 * 1024);
    let sound = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/public_audio/sounds/jump.ogg");
    let before = assets::decode_attempts();
    assert!(public_audio::import(&sound, &r)
        .unwrap_err()
        .contains("audio_resource_limit"));
    assert_eq!(assets::decode_attempts(), before);
    let key = r.keys().next().unwrap().clone();
    r.remove(&key);
    let accepted = public_audio::import(&sound, &r).unwrap();
    assert_eq!(assets::decode_attempts(), before + 1);
    assets::insert(&mut r, accepted).unwrap();
    assert!(textured_models::usage(&r).2 < 64 * 1024 * 1024);
}

#[test]
fn disk_residual_quota_foreign_paths_and_reserved_audio_entries_preserve_existing_bytes() {
    let root = temp();
    let asset = audio(8000, &Registry::new());
    let mut r = registry(asset.clone());
    let mut editor = Editor::with_assets(scene(&asset), r.clone()).unwrap();
    let entry = root.join("scene.agscene");
    let mut storage = Storage::default();
    storage.save(&mut editor, &entry, Fault::None).unwrap();
    let original = std::fs::read(&entry).unwrap();
    let blob = root.join(format!("assets/{}.wav", asset.identity.sha256));
    let blob_bytes = std::fs::read(&blob).unwrap();
    for path in [
        blob.clone(),
        root.join(format!("assets/{}.WAV", asset.identity.sha256)),
        root.join(format!("assets/../assets/{}.wav", asset.identity.sha256)),
    ] {
        assert!(storage
            .save(&mut editor, &path, Fault::None)
            .unwrap_err()
            .contains("reserved"));
        assert_eq!(std::fs::read(&entry).unwrap(), original);
        assert_eq!(std::fs::read(&blob).unwrap(), blob_bytes);
    }
    let foreign = root.join("assets/foreign.tmp");
    std::fs::write(&foreign, b"foreign bytes").unwrap();
    assert!(storage
        .save(&mut editor, &entry, Fault::None)
        .unwrap_err()
        .contains("foreign"));
    assert_eq!(std::fs::read(&foreign).unwrap(), b"foreign bytes");
    std::fs::remove_file(&foreign).unwrap();
    let mut unused = vec![];
    for i in 0..7 {
        let bytes = wav(100 + i, 1);
        let sha = agp::renderer::capture::sha256_hex(&bytes);
        let path = root.join(format!("assets/{sha}.wav"));
        std::fs::write(&path, &bytes).unwrap();
        unused.push(path);
    }
    let second = audio(9000, &r);
    assets::insert(&mut r, second.clone()).unwrap();
    editor
        .begin(
            editor.stamp,
            Edit::Audio {
                asset: Some(second),
                cues: vec![],
            },
        )
        .unwrap();
    accept(&mut editor);
    assert!(storage
        .save(&mut editor, &entry, Fault::None)
        .unwrap_err()
        .contains("resource limit"));
    assert_eq!(
        std::fs::read(&entry).unwrap(),
        original,
        "entry is last commit after full closure preparation"
    );
    assert_eq!(std::fs::read_dir(root.join("assets")).unwrap().count(), 8);
    std::fs::remove_file(&unused[0]).unwrap();
    assert_eq!(
        storage.save(&mut editor, &entry, Fault::None).unwrap(),
        Saved::Confirmed
    );
    let (reopened, reloaded, _) = Storage::load_assets(&entry, &Registry::new()).unwrap();
    assert_eq!(reopened.assets.len(), 2);
    assert_eq!(reloaded.len(), 2);
    // Physical residual bytes share32MiB, independent of current document refs.
    let disk = temp();
    std::fs::create_dir(disk.join("assets")).unwrap();
    let mut paths = vec![];
    for i in 1..=4 {
        let p = disk.join(format!("assets/{i:064x}.wav"));
        let f = std::fs::File::create(&p).unwrap();
        f.set_len(8 * 1024 * 1024).unwrap();
        paths.push(p);
    }
    let mut storage = Storage::default();
    assert!(storage
        .save(&mut editor, &disk.join("scene.agscene"), Fault::None)
        .unwrap_err()
        .contains("resource limit"));
    assert!(!disk.join("scene.agscene").exists());
    for p in &paths {
        assert_eq!(std::fs::metadata(p).unwrap().len(), 8 * 1024 * 1024);
    }
    std::fs::remove_file(&paths[0]).unwrap();
    assert_eq!(
        storage
            .save(&mut editor, &disk.join("scene.agscene"), Fault::None)
            .unwrap(),
        Saved::Confirmed
    );
    assert!(Storage::load_assets(&disk.join("scene.agscene"), &Registry::new()).is_ok());
}

#[test]
fn pending_save_owner_joins_host_candidate_union_and_releases_only_after_confirmation() {
    use autographics_auto_driver::authoring::host::Host;
    let root = temp();
    let old = audio(1, &Registry::new());
    let mut h = Host::with_assets(scene(&old), registry(old.clone())).unwrap();
    let entry = root.join("scene.agscene");
    assert_eq!(
        h.storage
            .save(&mut h.editor, &entry, Fault::AfterReplaceRead)
            .unwrap(),
        Saved::VerificationPending
    );
    // Close the accepted editor payload owner, leaving the actual pending-save owner.
    h.editor
        .begin_open(Document::empty("replacement".into()))
        .unwrap();
    h.settle().unwrap();
    assert!(h.editor.assets.is_empty());
    assert!(h
        .storage
        .retained_assets()
        .unwrap()
        .contains_key(&old.identity.key()));
    let mut next = Document::empty("eight-new".into());
    next.schema = "autographics.scene/7".into();
    next.cues = Some(vec![]);
    let mut r = Registry::new();
    for i in 20..28 {
        let a = audio(i, &r);
        next.assets.push(a.identity.clone());
        assets::insert(&mut r, a).unwrap();
    }
    h.editor.begin_open_assets(next.clone(), r.clone()).unwrap();
    h.settle().unwrap();
    assert!(
        !h.ui().error.is_empty(),
        "ninth union identity from pending save prevents candidate publication"
    );
    assert_eq!(h.editor.document.document_id, "replacement");
    assert!(h.editor.assets.is_empty());
    assert!(h.storage.unverified());
    // Confirm the pending receipt in its original root. It must not silently
    // mark another editor epoch saved; after that explicit conflict the owner
    // remains until the original epoch is restored or Storage is replaced.
    assert!(h
        .storage
        .verify(&mut h.editor)
        .unwrap_err()
        .contains("old epoch"));
    assert!(h.storage.unverified());
    h.storage = Storage::default();
    h.editor.begin_open_assets(next, r).unwrap();
    h.settle().unwrap();
    assert!(
        h.ui().error.is_empty(),
        "same legitimate8-resource candidate succeeds after owner release"
    );
    assert_eq!(h.editor.assets.len(), 8);
}

#[cfg(windows)]
#[test]
fn ancestor_junction_rejects_audio_import_and_save_before_io_side_effects() {
    use std::os::windows::process::CommandExt;
    let root = temp();
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir(&real).unwrap();
    let bytes = wav(1000, 32);
    std::fs::write(real.join("sound.wav"), &bytes).unwrap();
    let quote = |p: &Path| p.to_string_lossy().replace('\'', "''");
    let result = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "New-Item -ItemType Junction -Path '{}' -Value '{}' -ErrorAction Stop | Out-Null",
                quote(&link),
                quote(&real)
            ),
        ])
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let before = assets::decode_attempts();
    assert!(
        public_audio::import(&link.join("sound.wav"), &Registry::new())
            .unwrap_err()
            .contains("reparse")
    );
    assert_eq!(assets::decode_attempts(), before);
    let a = audio(2000, &Registry::new());
    let mut editor = Editor::with_assets(scene(&a), registry(a)).unwrap();
    assert!(Storage::default()
        .save(&mut editor, &link.join("scene.agscene"), Fault::None)
        .unwrap_err()
        .contains("reparse"));
    assert!(!real.join("scene.agscene").exists());
    assert!(!real.join(".autographics-assets-lock").exists());
    std::fs::remove_dir(&link).unwrap();
    assert!(real.is_dir());
    assert_eq!(std::fs::read(real.join("sound.wav")).unwrap(), bytes);
}
