use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::authoring::{
    assets,
    document::{Dimension, Document, Group, Kind, Object},
    host::Host,
    play::State,
};
use serde_json::json;
use std::sync::Arc;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
fn fixture() -> Host {
    let asset = assets::decode(
        include_bytes!("../research/asset-probe/two-parts.glb").to_vec(),
        None,
    )
    .unwrap();
    let mut d = Document::empty("behavior-host".into());
    let mut g = Group::new(10, Dimension::Three);
    g.translation = [1.0, 0.0, 0.0];
    d.groups.push(g);
    let mut o = Object::new(11, Kind::Model);
    o.parent = Some(10);
    o.translation = [-1.0, 0.0, 0.0];
    o.resource = Some(asset.identity.key());
    d.objects.push(o);
    let mut q = Object::new(12, Kind::Quad);
    q.translation = [-100.0, 0.0, 0.0];
    d.objects.push(q);
    d.assets.push(asset.identity.clone());
    Host::with_assets(d, assets::Registry::from([(asset.identity.key(), asset)])).unwrap()
}
fn dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!(
            "{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&d).unwrap();
    d
}
fn command(h: &mut Host, action: &str, payload: serde_json::Value) -> Result<(), String> {
    h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"play_token":h.ui().play_token,"action":action,"payload":payload}).to_string())
}
fn settle(h: &mut Host) {
    let limit = Instant::now() + Duration::from_secs(31);
    loop {
        h.frame(None).unwrap();
        if !h.play.as_ref().unwrap().pending() {
            break;
        }
        assert!(Instant::now() < limit)
    }
}
#[test]
fn bound_project_whole_path_gpu_history_and_twenty_owner_cycles() {
    use autographics_auto_driver::authoring::{
        behavior_project::*,
        session::Edit,
        storage::{Fault, Saved, Storage},
    };
    let mut h = fixture();
    h.frame(None).unwrap();
    let root = dir("project-845");
    let source = root.join("main.at");
    let entry = root.join("scene.agscene");
    h.storage.save(&mut h.editor, &entry, Fault::None).unwrap();
    let model_sha = h.editor.document.assets[0].sha256.clone();
    let mesh_pin = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => c.store.pin(&model_sha, "primitive-0").unwrap(),
        _ => unreachable!(),
    };
    let mut retained = vec![];
    let mut retained_sources = vec![];
    let mut source_probes = vec![];
    for i in 0..20 {
        let world = if i % 2 == 0 { 3.0 } else { 6.0 };
        std::fs::write(&source,format!("fn app_start(){{ag_set_local(ag_parent(ag_actor(3)),0,{world}+1.0)}}\nfn app_update(dt float){{}}\n")).unwrap();
        command(&mut h, "behavior-bind", json!(source)).unwrap();
        h.settle().unwrap();
        let code = h.editor.code.as_ref().unwrap();
        let weak = Arc::downgrade(code);
        let expected = code.identity().clone();
        source_probes.push(weak.clone());
        if std::env::var_os("AG_PROJECT_RETAIN_CODE").is_some() {
            retained_sources.push(code.clone());
        }
        std::fs::remove_file(&source).unwrap();
        if i == 0 {
            command(&mut h, "undo", json!(null)).unwrap();
            h.settle().unwrap();
            assert!(h.editor.code.is_none());
            command(&mut h, "redo", json!(null)).unwrap();
            h.settle().unwrap();
        }
        let before = h.editor.isolation_identity();
        let resources_before = h.resource_snapshot();
        let targets_before =
            [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq);
        h.start_selected_play().unwrap();
        h.pause_play("control");
        settle(&mut h);
        assert_eq!(h.play.as_ref().unwrap().state, State::Paused);
        assert_eq!(
            h.play
                .as_ref()
                .unwrap()
                .user_source
                .as_ref()
                .unwrap()
                .user_hash,
            expected.sha256
        );
        let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
        for (part, dx) in [-1.5, 1.5].into_iter().enumerate() {
            let p = autographics_auto_driver::authoring::interaction::project(
                h.camera(),
                glam::Vec3::new(world + dx, 0.5, 0.0),
            )
            .unwrap();
            let pixel = h.rt.probe_pixel(h.top, [p.x, p.y + 400.0]).unwrap();
            assert!(
                pixel[part] > 80 && pixel[1 - part] < 20,
                "bound project GPU part {part}: {pixel:?}"
            );
        }
        assert!(command(&mut h, "behavior-detach", json!(null)).is_err());
        if std::env::var_os("AG_PLAY_RETAIN_VM").is_some() {
            retained.push(h.play.take().unwrap());
        }
        h.stop_play();
        if std::env::var_os("AG_PLAY_FRESH_UPLOAD").is_some() {
            let asset = h.editor.assets.values().next().unwrap();
            let mut stage = match h.rt.content(h.three).unwrap() {
                SurfaceContent::Scene3d(c) => c.store.staged(),
                _ => unreachable!(),
            };
            stage.retain_models(|sha| sha != model_sha);
            for part in 0..asset.model.meshes.len() {
                stage.get_or_upload(h.rt.gpu(), &asset.model, part).unwrap();
            }
            if let SurfaceContent::Scene3d(c) = h.rt.content_mut(h.three).unwrap() {
                c.store = stage;
            }
        }
        assert!(!owner(), "runtime VM must release at Stop");
        assert_eq!(h.editor.isolation_identity(), before);
        let pin = match h.rt.content(h.three).unwrap() {
            SurfaceContent::Scene3d(c) => c.store.pin(&model_sha, "primitive-0").unwrap(),
            _ => unreachable!(),
        };
        assert!(
            Arc::ptr_eq(&pin, &mesh_pin),
            "warm model must retain actual GPU Arc"
        );
        let resources_after = h.resource_snapshot();
        for key in ["target_bytes", "target_count", "mesh_uploads"] {
            assert_eq!(
                resources_after[key], resources_before[key],
                "stable bound resource {key}"
            );
        }
        assert_eq!(
            [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq),
            targets_before
        );
        assert!(weak.upgrade().is_some());
    }
    assert_eq!(
        h.storage.save(&mut h.editor, &entry, Fault::None).unwrap(),
        Saved::Confirmed
    );
    let moved = root.join("moved");
    std::fs::create_dir(&moved).unwrap();
    let target = moved.join("entry.agscene");
    std::fs::remove_dir_all(root.join("assets")).unwrap();
    std::fs::remove_dir_all(root.join("scripts")).unwrap();
    h.storage.save(&mut h.editor, &target, Fault::None).unwrap();
    let (d, a, c, _) = Storage::load_project(&target, &Default::default()).unwrap();
    let mut fresh = Host::with_project(d, a, c).unwrap();
    fresh.start_selected_play().unwrap();
    fresh.pause_play("open");
    settle(&mut fresh);
    fresh.stop_play();
    assert!(!fresh.editor.dirty());
    assert_eq!(fresh.editor.history(), (0, 0, 0));
    let prior = fresh.editor.isolation_identity();
    let blob = target.parent().unwrap().join(
        fresh
            .editor
            .code
            .as_ref()
            .unwrap()
            .identity()
            .relative_path(),
    );
    std::fs::write(&blob, b"bad source").unwrap();
    assert!(fresh.open(&target).is_err());
    assert_eq!(fresh.editor.isolation_identity(), prior);
    // Runtime Fault is still statically compilable and does not run during binding/Open.
    let c = CodePayload::from_bytes(
        std::path::Path::new("fault.at"),
        b"fn app_start(){while true {}}\nfn app_update(dt float){}\n".to_vec(),
    )
    .unwrap();
    let b = Binding {
        api: autographics_auto_driver::authoring::behavior::VERSION.into(),
        source: c.identity().clone(),
        actors: Actors {
            two_d: None,
            three_d: None,
        },
    };
    fresh
        .editor
        .begin(
            fresh.editor.stamp,
            Edit::Behavior {
                binding: Some(b),
                code: Some(c),
            },
        )
        .unwrap();
    fresh.settle().unwrap();
    let weak = Arc::downgrade(fresh.editor.code.as_ref().unwrap());
    let external_pin = fresh.editor.code.as_ref().unwrap().pin_bytes();
    fresh.editor.close();
    assert!(weak.upgrade().is_none());
    assert!(!external_pin.is_empty());
    h.editor.close();
    assert!(
        source_probes.iter().all(|w| w.upgrade().is_none()),
        "project source must release after last internal owner"
    );
    assert!(retained_sources.is_empty());
}

#[test]
fn binding_cancel_retarget_temporary_and_failed_prepare_are_atomic() {
    let mut h = fixture();
    h.frame(None).unwrap();
    let root = dir("project-845-isolation");
    let source = root.join("main.at");
    std::fs::write(&source, "fn app_start(){}\nfn app_update(dt float){}\n").unwrap();
    let before = h.editor.isolation_identity();
    let old = h.editor.stamp;
    let generation = h.ui().play_token;
    command(&mut h, "behavior-bind", json!(source)).unwrap();
    command(&mut h, "cancel-binding", json!(null)).unwrap();
    assert_eq!(h.editor.isolation_identity(), before);
    assert!(h.dispatch(&json!({"epoch":old.epoch,"revision":old.revision,"play_token":generation,"action":"behavior-bind","payload":source}).to_string()).is_err());
    h.fail_resource = true;
    command(&mut h, "behavior-bind", json!(source)).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.isolation_identity(), before);
    h.fail_resource = false;
    command(&mut h, "behavior-bind", json!(source)).unwrap();
    h.settle().unwrap();
    let bound = h.editor.document.behavior.clone();
    command(
        &mut h,
        "behavior-actors",
        json!({"two_d":null,"three_d":null}),
    )
    .unwrap();
    h.settle().unwrap();
    assert_eq!(
        h.editor.document.behavior.as_ref().unwrap().actors.ids(),
        [None, None]
    );
    command(&mut h, "undo", json!(null)).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document.behavior, bound);
    let baseline = h.editor.isolation_identity();
    h.select_behavior(Some(source.display().to_string()))
        .unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("control");
    settle(&mut h);
    h.stop_play();
    assert_eq!(h.editor.isolation_identity(), baseline);
    h.select_behavior(None).unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("control");
    settle(&mut h);
    assert!(h.play.as_ref().unwrap().user_source.is_none());
    h.stop_play();
    assert_eq!(h.editor.isolation_identity(), baseline);
    command(&mut h, "play-project-default", json!(null)).unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("control");
    settle(&mut h);
    assert!(h.play.as_ref().unwrap().user_source.is_some());
    h.stop_play();
}

// Provenance is separate from document dirtiness and uses accepted bytes at Run.
fn assert_project_origin(h: &mut Host, expected: &std::path::Path, runtime_error: bool) {
    h.start_selected_play().unwrap();
    let source = h.play.as_ref().unwrap().user_source.as_ref().unwrap();
    assert_eq!(source.path, expected);
    let hash = source.user_hash.clone();
    let origin = source.path.display().to_string();
    assert!(h.ui().last_behavior.contains(&origin));
    assert!(h.ui().last_behavior.contains(&hash));
    if runtime_error {
        let deadline = Instant::now() + Duration::from_secs(31);
        while h.play.as_ref().unwrap().error.is_none() {
            h.frame(None).unwrap();
            assert!(Instant::now() < deadline);
        }
        let error = h.play.as_ref().unwrap().error.as_ref().unwrap();
        assert!(error.contains(&origin), "{error}");
        assert!(
            error.contains("999"),
            "must reach the actual Auto callback: {error}"
        );
    }
    h.stop_play();
}
fn memory_origin(h: &Host) -> PathBuf {
    PathBuf::from(format!(
        "<memory:{}>.at",
        h.editor.code.as_ref().unwrap().identity().sha256
    ))
}
fn bind_origin_source(h: &mut Host, path: &std::path::Path, body: &str) {
    std::fs::write(
        path,
        format!("fn app_start(){{{body}}}\nfn app_update(dt float){{}}\n"),
    )
    .unwrap();
    command(h, "behavior-bind", json!(path)).unwrap();
    h.settle().unwrap();
    std::fs::remove_file(path).unwrap();
}
#[test]
fn bound_source_origin_tracks_verified_identity_through_history_and_open() {
    use autographics_auto_driver::authoring::{session::Edit, storage::Fault};
    let mut h = Host::new(Document::sample()).unwrap();
    let root = dir("source-origin-history").canonicalize().unwrap();
    let input = root.join("input.at");
    bind_origin_source(&mut h, &input, "");
    let memory_a = memory_origin(&h);
    assert_project_origin(&mut h, &memory_a, false);
    let entry = root.join("scene.agscene");
    h.storage.save(&mut h.editor, &entry, Fault::None).unwrap();
    let saved_a = root.join(h.editor.code.as_ref().unwrap().identity().relative_path());
    assert_project_origin(&mut h, &saved_a, false);
    let mut object = h.editor.document.objects[0].clone();
    object.translation[0] += 1.0;
    h.editor.selection = Some(object.id);
    h.editor
        .begin(h.editor.stamp, Edit::Replace(object))
        .unwrap();
    h.settle().unwrap();
    assert!(h.editor.dirty());
    assert_project_origin(&mut h, &saved_a, false);
    bind_origin_source(&mut h, &input, "ag_set_visible(999,1)");
    let memory_b = memory_origin(&h);
    let saved_b = root.join(h.editor.code.as_ref().unwrap().identity().relative_path());
    assert!(!saved_b.exists());
    assert_project_origin(&mut h, &memory_b, true);
    command(&mut h, "undo", json!(null)).unwrap();
    h.settle().unwrap();
    assert_project_origin(&mut h, &saved_a, false);
    command(&mut h, "redo", json!(null)).unwrap();
    h.settle().unwrap();
    assert_project_origin(&mut h, &memory_b, true);
    h.storage.save(&mut h.editor, &entry, Fault::None).unwrap();
    assert_project_origin(&mut h, &saved_b, true);
    command(&mut h, "undo", json!(null)).unwrap();
    h.settle().unwrap();
    assert_project_origin(&mut h, &saved_a, false);
    command(&mut h, "redo", json!(null)).unwrap();
    h.settle().unwrap();
    assert_project_origin(&mut h, &saved_b, true);
    // No Run reread even if the original import and confirmed blob disappear.
    let bytes = std::fs::read(&saved_b).unwrap();
    std::fs::remove_file(&saved_b).unwrap();
    assert_project_origin(&mut h, &saved_b, true);
    std::fs::write(&saved_b, bytes).unwrap();
    // Save As exports only the selected source. Old-root proof for B must not
    // claim that B exists beside A in the new project after Redo.
    command(&mut h, "undo", json!(null)).unwrap();
    h.settle().unwrap();
    let exported = dir("source-origin-new-root").canonicalize().unwrap();
    h.storage
        .save(&mut h.editor, &exported.join("copy.agscene"), Fault::None)
        .unwrap();
    let exported_a = exported.join(h.editor.code.as_ref().unwrap().identity().relative_path());
    assert_project_origin(&mut h, &exported_a, false);
    command(&mut h, "redo", json!(null)).unwrap();
    h.settle().unwrap();
    assert!(!exported
        .join(h.editor.code.as_ref().unwrap().identity().relative_path())
        .exists());
    assert_project_origin(&mut h, &memory_b, true);
    let mut reopened = Host::new(Document::sample()).unwrap();
    reopened.open(&entry).unwrap();
    reopened.settle().unwrap();
    assert_project_origin(&mut reopened, &saved_b, true);
}
#[test]
fn bound_source_origin_waits_for_verified_save_as_and_old_revision_receipts() {
    use autographics_auto_driver::authoring::storage::{Fault, Saved, Snapshot};
    let mut h = Host::new(Document::sample()).unwrap();
    let root = dir("source-origin-receipt").canonicalize().unwrap();
    let input = root.join("input.at");
    let entry = root.join("scene.agscene");
    h.storage.save(&mut h.editor, &entry, Fault::None).unwrap();
    bind_origin_source(&mut h, &input, "");
    let memory_a = memory_origin(&h);
    assert_project_origin(&mut h, &memory_a, false); // first binding on saved empty scene
    assert_eq!(
        h.storage
            .save(&mut h.editor, &entry, Fault::CodeClosureAfterReplace)
            .unwrap(),
        Saved::VerificationPending
    );
    assert_eq!(
        h.storage
            .source_origin(h.editor.code.as_ref().unwrap().identity()),
        memory_a
    );
    assert!(h
        .start_selected_play()
        .unwrap_err()
        .contains("idle accepted editor")); // pending Run stays blocked
    h.storage.verify(&mut h.editor).unwrap();
    let saved_a = root.join(h.editor.code.as_ref().unwrap().identity().relative_path());
    assert_project_origin(&mut h, &saved_a, false);
    let old = Snapshot::capture(&h.editor).unwrap();
    bind_origin_source(&mut h, &input, "ag_set_visible(999,1)");
    let memory_b = memory_origin(&h);
    let other = dir("source-origin-save-as").canonicalize().unwrap();
    let other_entry = other.join("copy.agscene");
    assert!(h
        .storage
        .save(&mut h.editor, &other_entry, Fault::BeforeReplace)
        .is_err());
    assert_project_origin(&mut h, &memory_b, true);
    assert_eq!(
        h.storage
            .save_snapshot(&mut h.editor, &other_entry, old, Fault::AfterReplaceRead)
            .unwrap(),
        Saved::VerificationPending
    );
    assert_eq!(
        h.storage
            .source_origin(h.editor.code.as_ref().unwrap().identity()),
        memory_b
    );
    assert!(h
        .start_selected_play()
        .unwrap_err()
        .contains("idle accepted editor"));
    h.storage.verify(&mut h.editor).unwrap(); // confirms A at a different root, not current B
    assert!(h.editor.dirty());
    assert_project_origin(&mut h, &memory_b, true);
    command(&mut h, "undo", json!(null)).unwrap();
    h.settle().unwrap();
    let other_a = other.join(h.editor.code.as_ref().unwrap().identity().relative_path());
    assert_project_origin(&mut h, &other_a, false);
    command(&mut h, "redo", json!(null)).unwrap();
    h.settle().unwrap();
    assert_project_origin(&mut h, &memory_b, true);
    h.storage
        .save(&mut h.editor, &other_entry, Fault::None)
        .unwrap();
    let other_b = other.join(h.editor.code.as_ref().unwrap().identity().relative_path());
    assert_project_origin(&mut h, &other_b, true);
}
