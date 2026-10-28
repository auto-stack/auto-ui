use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::authoring::{
    assets,
    document::{Dimension, Document, Group, Kind, Object},
    host::Host,
    play::{Action, State, SOURCE},
    session::Edit,
};
use serde_json::{json, Value};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
fn fixture() -> Host {
    let asset = assets::decode(
        include_bytes!("../research/asset-probe/two-parts.glb").to_vec(),
        None,
    )
    .unwrap();
    let mut d = Document::empty("play-host".into());
    let mut group = Group::new(10, Dimension::Three);
    group.translation = [1.0, 0.0, 0.0];
    d.groups.push(group);
    let mut model = Object::new(11, Kind::Model);
    model.parent = Some(10);
    model.translation = [-1.0, 0.0, 0.0];
    model.resource = Some(asset.identity.key());
    d.objects.push(model);
    let mut two = Object::new(12, Kind::Quad);
    two.translation = [-100.0, 0.0, 0.0];
    d.objects.push(two);
    d.assets.push(asset.identity.clone());
    Host::with_assets(d, assets::Registry::from([(asset.identity.key(), asset)])).unwrap()
}
fn command(h: &mut Host, action: &str) -> Result<(), String> {
    let m = message(h, action);
    h.dispatch(&m)
}
fn message(h: &Host, action: &str) -> String {
    json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"play_token":h.ui().play_token,"action":action,"payload":null}).to_string()
}
fn settle(h: &mut Host) {
    let limit = Instant::now() + Duration::from_secs(31);
    loop {
        h.frame(None).unwrap();
        if !h.play.as_ref().unwrap().pending() {
            break;
        }
        assert!(Instant::now() < limit, "host transaction timeout");
    }
}
fn capture(h: &mut Host) -> String {
    h.frame(None).unwrap();
    format!(
        "{}:{}",
        h.rt.capture(h.two, "play2").unwrap().0,
        h.rt.capture(h.three, "play3").unwrap().0
    )
}
#[test]
fn accepted_dirty_redo_unsaved_asset_isolation_stop_restore_files_and_reopen() {
    let mut h = fixture();
    h.editor.select(Some(12)).unwrap();
    let mut o = h.editor.document.objects[1].clone();
    o.translation[0] -= 10.0;
    h.editor
        .begin(h.editor.stamp, Edit::Replace(o.clone()))
        .unwrap();
    h.settle().unwrap();
    o.translation[0] -= 10.0;
    h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
    h.settle().unwrap();
    h.editor.begin(h.editor.stamp, Edit::Undo).unwrap();
    h.settle().unwrap();
    assert!(h.editor.dirty());
    assert_eq!(h.editor.history().1, 1);
    let baseline = h.editor.isolation_identity();
    let image = capture(&mut h);
    let dir = std::env::temp_dir().join(format!("play839-files-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("scene.agscene");
    std::fs::write(&path, b"protected sentinel").unwrap();
    command(&mut h, "play-run").unwrap();
    settle(&mut h);
    command(&mut h, "play-pause").unwrap();
    settle(&mut h);
    for phase in 0..3 {
        if phase == 1 {
            command(&mut h, "play-resume").unwrap();
        }
        if phase == 2 {
            h.play.as_mut().unwrap().fail("probe fault".into());
        }
        for action in [
            "save",
            "saveas",
            "open",
            "new",
            "import",
            "create",
            "delete-request",
            "apply",
            "reparent",
            "undo",
            "redo",
            "play-actor2",
        ] {
            let m=json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":action,"payload":path,"play_token":h.ui().play_token}).to_string();
            assert!(h.dispatch(&m).is_err(), "{action} phase{phase}");
        }
        assert_eq!(h.editor.isolation_identity(), baseline);
        assert_eq!(std::fs::read(&path).unwrap(), b"protected sentinel");
        assert!(std::fs::read_dir(&dir).unwrap().count() == 1);
    }
    command(&mut h, "play-stop").unwrap();
    assert_eq!(capture(&mut h), image);
    assert_eq!(h.editor.isolation_identity(), baseline);
    h.request_close();
    assert!(h.ui().dialog);
    assert!(h.play.is_none());
    let m = message(&h, "leave-cancel");
    h.dispatch(&m).unwrap();
    assert!(!h.closed);
    assert_eq!(h.editor.isolation_identity(), baseline);
    // Save only after Stop, then use the real new-process inspector and compare editable locals.
    let path = dir.join("saved.agscene");
    h.storage
        .save(
            &mut h.editor,
            &path,
            autographics_auto_driver::authoring::storage::Fault::None,
        )
        .unwrap();
    let child = std::process::Command::new(env!("CARGO_BIN_EXE_autographics-authoring"))
        .args(["--inspect", path.to_str().unwrap()])
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
    let restored: Value = serde_json::from_slice(&child.stdout).unwrap();
    assert_eq!(restored["document"], json!(h.editor.document));
    assert_eq!(restored["history"], json!([0, 0, 0]));
    assert_eq!(restored["dirty"], false);
}
#[test]
fn preparing_failure_retains_last_good_and_stop_needs_no_prepare_or_compile() {
    let mut h = fixture();
    let original = capture(&mut h);
    let compile = h.editor.preview.compile_us;
    command(&mut h, "play-run").unwrap();
    settle(&mut h);
    h.pause_play("auto-Pause: diagnostic control");
    settle(&mut h);
    assert!(h.ui().error.contains("diagnostic control"));
    command(&mut h, "play-resume").unwrap();
    assert!(
        h.ui().error.is_empty(),
        "successful Resume must clear stale pause reason"
    );
    let p = h.play.as_mut().unwrap();
    p.pause("");
    p.dispatch(Action {
        space: 3,
        x: 1,
        ..Default::default()
    })
    .unwrap();
    settle(&mut h);
    let good = capture(&mut h);
    assert_ne!(good, original);
    let accepted = h.play.as_ref().unwrap().tick();
    let alive = h.play.as_ref().unwrap().owner_probe().unwrap();
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action {
            space: 3,
            x: 1,
            ..Default::default()
        })
        .unwrap();
    h.fail_resource = true;
    settle(&mut h);
    assert_eq!(h.play.as_ref().unwrap().state, State::Faulted);
    assert_eq!(h.play.as_ref().unwrap().tick(), accepted);
    assert!(!alive());
    assert_eq!(capture(&mut h), good);
    command(&mut h, "play-stop").unwrap();
    assert_eq!(capture(&mut h), original);
    assert_eq!(h.editor.preview.compile_us, compile);
    h.fail_resource = false;
    command(&mut h, "play-run").unwrap();
    settle(&mut h);
    assert_eq!(h.play.as_ref().unwrap().tick(), 0);
    command(&mut h, "play-stop").unwrap();
    let bad = SOURCE.replace("return 6", "return 5");
    h.start_play(&bad).unwrap();
    let bad_owner = h.play.as_ref().unwrap().owner_probe().unwrap();
    settle(&mut h);
    assert!(
        !bad_owner(),
        "failed initialization must close the actual runtime VM"
    );
    assert_eq!(h.play.as_ref().unwrap().state, State::Faulted);
    assert_eq!(capture(&mut h), original);
    command(&mut h, "play-stop").unwrap();
    command(&mut h, "play-run").unwrap();
    settle(&mut h);
    assert_eq!(h.play.as_ref().unwrap().state, State::Running);
    assert_eq!(h.play.as_ref().unwrap().tick(), 0);
    command(&mut h, "play-stop").unwrap();
}
#[test]
fn twenty_real_vm_resource_owner_cycles_and_last_external_pin() {
    let mut h = fixture();
    let asset = h.editor.assets.values().next().unwrap().clone();
    let source_weak = Arc::downgrade(&asset);
    let model_sha = asset.identity.sha256.clone();
    drop(asset);
    let (mesh_pin, initial_uploads) = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => (
            c.store.pin(&model_sha, "primitive-0").unwrap(),
            c.store.uploads.snapshot(),
        ),
        _ => unreachable!(),
    };
    let mesh_weak = Arc::downgrade(&mesh_pin);
    let targets = h.rt.owned_target_bytes();
    let target_ids = [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq);
    let pipeline = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => &c.mesh as *const _ as usize,
        _ => unreachable!(),
    };
    let baseline = h.editor.isolation_identity();
    let mut retained = vec![];
    for i in 0..20 {
        let begin = Instant::now();
        command(&mut h, "play-run").unwrap();
        settle(&mut h);
        let alive = h.play.as_ref().unwrap().owner_probe().unwrap();
        assert!(alive());
        command(&mut h, "play-pause").unwrap();
        settle(&mut h);
        command(&mut h, "play-step").unwrap();
        settle(&mut h);
        if i % 2 == 0 {
            h.play.as_mut().unwrap().fail("owner cycle fault".into());
            assert!(!alive());
        }
        if std::env::var_os("AG_PLAY_RETAIN_VM").is_some() {
            retained.push(h.play.take().unwrap());
            h.stop_play();
        } else {
            command(&mut h, "play-stop").unwrap();
        }
        assert!(!alive(), "runtime VM must release at Stop");
        assert!(source_weak.upgrade().is_some());
        assert!(mesh_weak.upgrade().is_some());
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
        h.frame(None).unwrap();
        assert_eq!(h.editor.isolation_identity(), baseline);
        if let SurfaceContent::Scene3d(c) = h.rt.content(h.three).unwrap() {
            assert!(
                Arc::ptr_eq(&mesh_pin, &c.store.pin(&model_sha, "primitive-0").unwrap()),
                "warm model must retain actual GPU Arc"
            );
            assert_eq!(
                c.store.uploads.snapshot().buffer_rewrites,
                initial_uploads.buffer_rewrites
            );
            assert_eq!(
                c.store.uploads.snapshot().bytes_rewritten,
                initial_uploads.bytes_rewritten
            );
        }
        assert_eq!(h.rt.owned_target_bytes(), targets);
        assert_eq!(
            [h.two, h.three, h.panel, h.top].map(|id| h.rt.target(id).unwrap().build_seq),
            target_ids
        );
        if let SurfaceContent::Scene3d(c) = h.rt.content(h.three).unwrap() {
            assert_eq!(&c.mesh as *const _ as usize, pipeline);
        }
        println!(
            "owner cycle {i}: {}us {:?}",
            begin.elapsed().as_micros(),
            h.resource_snapshot()
        );
    }
    h.request_close();
    assert!(h.closed);
    assert!(source_weak.upgrade().is_none());
    assert!(
        mesh_weak.upgrade().is_some(),
        "legitimate external pin retains mesh after editor close"
    );
    drop(mesh_pin);
    assert!(mesh_weak.upgrade().is_none());
}
#[test]
fn draft_pending_actor_and_stale_operations_reject_before_side_effects() {
    let mut h = fixture();
    h.editor.select(Some(12)).unwrap();
    h.settle().unwrap();
    h.ui_mut().draft = vec!["unapplied".into()];
    assert!(command(&mut h, "play-run").unwrap_err().contains("draft"));
    let m = message(&h, "cancel");
    h.dispatch(&m).unwrap();
    command(&mut h, "play-run").unwrap();
    let stale = message(&h, "play-pause");
    settle(&mut h);
    command(&mut h, "play-pause").unwrap();
    assert!(h.dispatch(&stale).is_err());
    settle(&mut h);
    let step = message(&h, "play-step");
    h.dispatch(&step).unwrap();
    assert!(h.dispatch(&step).is_err());
    let stop = message(&h, "play-stop");
    h.dispatch(&stop).unwrap();
    assert!(h.dispatch(&stop).is_err());
    command(&mut h, "play-run").unwrap();
    assert!(h.dispatch(&step).is_err());
    command(&mut h, "play-stop").unwrap();
    let mut o = h.editor.document.objects[1].clone();
    o.translation[0] += 1.0;
    h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
    assert!(command(&mut h, "play-run").is_err());
}

#[test]
fn changed_auto_speed_and_visibility_are_consumed_by_actual_gpu() {
    let mut h = fixture();
    let source = SOURCE.replace("speed=60.0", "speed=120.0");
    assert_ne!(source, SOURCE);
    h.start_play(&source).unwrap();
    settle(&mut h);
    h.play.as_mut().unwrap().pause("");
    for _ in 0..60 {
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action {
                space: 2,
                x: 1,
                ..Default::default()
            })
            .unwrap();
        settle(&mut h);
    }
    // Source-controlled 120 units, not the host's documented default60.
    assert_eq!(
        h.play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .unwrap()
            .document
            .objects[1]
            .translation[0],
        20.0
    );
    h.frame(None).unwrap();
    let endpoint = h.rt.probe_pixel(h.top, [420.0, 190.0]).unwrap();
    let origin = h.rt.probe_pixel(h.top, [300.0, 190.0]).unwrap();
    assert!(
        endpoint[0] > 150 && endpoint[1] > 100 && origin[0] < 100,
        "actual Auto changed-speed endpoint/origin {endpoint:?}/{origin:?}"
    );
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action {
            space: 2,
            toggle: true,
            ..Default::default()
        })
        .unwrap();
    settle(&mut h);
    let hidden = h.rt.probe_pixel(h.top, [420.0, 190.0]).unwrap();
    assert!(
        hidden[0] < 100 && hidden[1] < 100,
        "actual Auto visibility must reach GPU: {hidden:?}"
    );
    h.stop_play();
    assert_eq!(h.editor.document.objects[1].translation[0], -100.0);
}

fn starting_cancel(h: &mut Host, edge: &str) {
    match edge {
        "focus lost" => {
            h.input.window_focus_lost();
            h.blur();
        }
        "resize/DPI" => h.resize(1200, 800),
        "cursor left" => {
            h.pause_play("auto-Pause: cursor left");
            h.input.pointer_left_window();
        }
        "window occluded" => h.pause_play("auto-Pause: window occluded"),
        _ => unreachable!(),
    }
}

#[test]
fn starting_window_cancellation_waits_for_initial_frame_and_explicit_resume() {
    use autographics_auto_driver::authoring::play::Key;
    for edge in ["focus lost", "resize/DPI", "cursor left", "window occluded"] {
        let mut h = fixture();
        let baseline = h.editor.isolation_identity();
        let original = capture(&mut h);
        command(&mut h, "play-run").unwrap();
        let alive = h.play.as_ref().unwrap().owner_probe().unwrap();
        let p = h.play.as_mut().unwrap();
        assert_eq!(p.state, State::Starting);
        p.input.focus(2);
        p.input.key(Key::Right, true, false);
        p.input.key(Key::Toggle, true, false);
        starting_cancel(&mut h, edge);
        assert_eq!(h.play.as_ref().unwrap().state, State::Starting);
        assert!(command(&mut h, "play-resume").is_err());
        assert!(command(&mut h, "play-step").is_err());
        settle(&mut h);
        let p = h.play.as_mut().unwrap();
        assert_eq!(p.state, State::Paused, "{edge}");
        assert!(p
            .error
            .as_deref()
            .is_some_and(|reason| reason.contains(edge)));
        assert_eq!(p.tick(), 0);
        assert_eq!(p.last.as_ref().unwrap().document, h.editor.document);
        let input = p.input.sample();
        assert_eq!((input.x, input.y, input.toggle), (0, 0, false));
        assert!(alive());
        for _ in 0..3 {
            std::thread::sleep(Duration::from_millis(20));
            h.frame(None).unwrap();
        }
        assert_eq!(h.play.as_ref().unwrap().tick(), 0, "{edge}");
        assert_eq!(capture(&mut h), original);
        assert_eq!(h.editor.isolation_identity(), baseline);

        command(&mut h, "play-step").unwrap();
        settle(&mut h);
        assert_eq!(h.play.as_ref().unwrap().state, State::Paused);
        assert_eq!(h.play.as_ref().unwrap().tick(), 1);
        assert_eq!(capture(&mut h), original);
        command(&mut h, "play-resume").unwrap();
        let now = Instant::now();
        let p = h.play.as_mut().unwrap();
        assert!(p.error.is_none());
        p.input.focus(2);
        p.input.key(Key::Right, true, true);
        p.clock(now);
        p.clock(now + Duration::from_millis(17));
        assert!(p.dispatch_credit().unwrap());
        settle(&mut h);
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.tick(), 2);
        assert_eq!(p.last.as_ref().unwrap().document, h.editor.document);
        assert_eq!(
            p.reports.back().unwrap().x,
            0,
            "repeat cannot revive held key"
        );
        command(&mut h, "play-pause").unwrap();
        command(&mut h, "play-resume").unwrap();
        let p = h.play.as_mut().unwrap();
        p.input.key(Key::Right, true, false);
        p.clock(now);
        p.clock(now + Duration::from_millis(17));
        assert!(p.dispatch_credit().unwrap());
        settle(&mut h);
        assert_eq!(h.play.as_ref().unwrap().tick(), 3);
        assert_eq!(
            h.play
                .as_ref()
                .unwrap()
                .last
                .as_ref()
                .unwrap()
                .document
                .objects[1]
                .translation[0],
            -99.0
        );
        command(&mut h, "play-stop").unwrap();
        assert!(!alive());
        assert_eq!(capture(&mut h), original);
        assert_eq!(h.editor.isolation_identity(), baseline);

        // A latched startup cancellation can still Stop without accepting any runtime frame.
        command(&mut h, "play-run").unwrap();
        let pending_owner = h.play.as_ref().unwrap().owner_probe().unwrap();
        starting_cancel(&mut h, edge);
        command(&mut h, "play-stop").unwrap();
        assert!(!pending_owner());
        assert!(h.play.is_none());
        assert_eq!(capture(&mut h), original);
        assert_eq!(h.editor.isolation_identity(), baseline);
    }
}

#[test]
fn canceled_startup_failure_releases_vm_and_next_run_initializes_normally() {
    for resource_fault in [false, true] {
        let mut h = fixture();
        let original = capture(&mut h);
        let baseline = h.editor.isolation_identity();
        let wrong_profile = SOURCE.replace("return 6", "return 5");
        h.start_play(if resource_fault {
            SOURCE
        } else {
            &wrong_profile
        })
        .unwrap();
        let run = h.play.as_ref().unwrap().run_id;
        let alive = h.play.as_ref().unwrap().owner_probe().unwrap();
        h.fail_resource = resource_fault;
        starting_cancel(&mut h, "focus lost");
        settle(&mut h);
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.state, State::Faulted);
        assert!(p.last.is_none());
        assert!(p.error.as_ref().unwrap().contains(if resource_fault {
            "preparation"
        } else {
            "profile/run/tick/count"
        }));
        assert!(!alive());
        assert_eq!(capture(&mut h), original);
        command(&mut h, "play-stop").unwrap();
        assert_eq!(capture(&mut h), original);
        h.fail_resource = false;
        command(&mut h, "play-run").unwrap();
        settle(&mut h);
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.state, State::Running);
        assert_ne!(p.run_id, run);
        assert_eq!(p.tick(), 0);
        assert_eq!(p.last.as_ref().unwrap().document, h.editor.document);
        command(&mut h, "play-stop").unwrap();
        assert_eq!(h.editor.isolation_identity(), baseline);
    }
}
