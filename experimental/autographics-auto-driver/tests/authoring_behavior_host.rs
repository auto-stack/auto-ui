use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::authoring::{
    assets,
    document::{Dimension, Document, Group, Kind, Object},
    host::Host,
    play::{Action, State},
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
        .join(name);
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
fn actual_source_gpu_twenty_runs_stop_fault_and_source_selection_isolation() {
    let mut h = fixture();
    h.frame(None).unwrap();
    let original = (
        h.rt.capture(h.two, "initial2").unwrap().0,
        h.rt.capture(h.three, "initial3").unwrap().0,
    );
    let before = h.editor.isolation_identity();
    let resources = h.resource_snapshot();
    let model_sha = h
        .editor
        .assets
        .values()
        .next()
        .unwrap()
        .identity
        .sha256
        .clone();
    let mesh_pin = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(c) => c.store.pin(&model_sha, "primitive-0").unwrap(),
        _ => unreachable!(),
    };
    let mut retained = vec![];
    let editor_owner = h.editor.preview.vm.owner_probe();
    let path = dir("behavior-host-source").join("main.at");
    for i in 0..20 {
        let world = if i % 2 == 0 { 3.0 } else { 6.0 };
        let source=format!("var initial={world}\nfn app_start(){{let n=ag_actor(3); ag_set_local(ag_parent(n),0,initial+1.0)}}\nfn app_update(dt float){{}}\n");
        std::fs::write(&path, source).unwrap();
        h.select_behavior(Some(path.display().to_string())).unwrap();
        h.start_selected_play().unwrap();
        h.pause_play("init pause");
        settle(&mut h);
        assert_eq!(h.play.as_ref().unwrap().state, State::Paused);
        let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
        assert!(h.select_behavior(None).is_err());
        assert!(command(&mut h, "play-source-path", json!("other.at")).is_err());
        for (part, dx) in [-1.5, 1.5].into_iter().enumerate() {
            let p = autographics_auto_driver::authoring::interaction::project(
                h.camera(),
                glam::Vec3::new(world + dx, 0.5, 0.0),
            )
            .unwrap();
            let pixel = h.rt.probe_pixel(h.top, [p.x, p.y + 400.0]).unwrap();
            assert!(
                pixel[part] > 80 && pixel[1 - part] < 20,
                "user all-part GPU {pixel:?}"
            );
        }
        assert_eq!(h.editor.isolation_identity(), before);
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
        h.frame(None).unwrap();
        assert!(!owner(), "runtime VM must release at Stop");
        if let SurfaceContent::Scene3d(c) = h.rt.content(h.three).unwrap() {
            assert!(
                Arc::ptr_eq(&mesh_pin, &c.store.pin(&model_sha, "primitive-0").unwrap()),
                "warm model must retain actual GPU Arc"
            );
        }
        assert!(editor_owner());
        assert_eq!(
            (
                h.rt.capture(h.two, "restored2").unwrap().0,
                h.rt.capture(h.three, "restored3").unwrap().0
            ),
            original
        );
        assert_eq!(
            h.resource_snapshot()["mesh_uploads"],
            resources["mesh_uploads"]
        );
    }
    std::fs::write(
        &path,
        "fn app_start(){}\nfn app_update(dt float){ag_set_visible(-1,1)}",
    )
    .unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("");
    settle(&mut h);
    let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
    h.play.as_mut().unwrap().single_step().unwrap();
    let limit = Instant::now() + Duration::from_secs(5);
    while h.play.as_ref().unwrap().state != State::Faulted {
        let _ = h.frame(None);
        assert!(Instant::now() < limit)
    }
    assert!(!owner());
    h.stop_play();
    std::fs::write(&path, "fn app_start(){}\nfn app_update(dt float){}").unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("");
    settle(&mut h);
    h.stop_play();
    assert_eq!(h.editor.isolation_identity(), before);
}
#[test]
fn busy_before_read_relative_root_and_save_as_no_fallback() {
    let mut h = fixture();
    let root = dir("behavior-roots");
    let old = root.join("old");
    let new = root.join("new");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::create_dir_all(&new).unwrap();
    h.ui_mut().path = old.join("scene.agscene").display().to_string();
    command(&mut h, "saveas", json!(old.join("scene.agscene"))).unwrap();
    h.settle().unwrap();
    std::fs::write(
        old.join("main.at"),
        "fn app_start(){}\nfn app_update(dt float){}",
    )
    .unwrap();
    h.select_behavior(Some("main.at".into())).unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("");
    settle(&mut h);
    assert_eq!(
        h.play
            .as_ref()
            .unwrap()
            .user_source
            .as_ref()
            .unwrap()
            .path
            .canonicalize()
            .unwrap(),
        old.join("main.at").canonicalize().unwrap()
    );
    h.stop_play();
    h.ui_mut().path = new.join("scene.agscene").display().to_string();
    command(&mut h, "saveas", json!(new.join("scene.agscene"))).unwrap();
    h.settle().unwrap();
    let before = h.editor.isolation_identity();
    assert!(h.ui().behavior_resolved.contains("new"));
    let error = h.start_selected_play().unwrap_err();
    assert!(error.contains("new"), "{error}");
    assert!(h.play.is_none());
    assert_eq!(h.editor.isolation_identity(), before);
    assert!(!new.join("main.at").exists());
    command(&mut h, "select", json!(12)).unwrap();
    h.frame(None).unwrap();
    h.ui_mut().draft[1] = "123".into();
    let error = h.start_selected_play().unwrap_err();
    assert!(
        error.contains("draft"),
        "must reject before trying missing file: {error}"
    );
}
#[test]
fn profile7_600_tick_trace_against_independent_reference() {
    let mut h = fixture();
    let path = dir("behavior-trace").join("main.at");
    std::fs::write(&path,"fn app_start(){}\nfn app_update(dt float){let n=ag_actor(ag_input_space());if ag_valid(n)==1 {ag_move_local(n,ag_input_x()*3.0*dt,0.0,0.0)}}").unwrap();
    h.select_behavior(Some(path.display().to_string())).unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("");
    settle(&mut h);
    let mut expected = -1.0f32;
    for i in 1..=600 {
        let x = if i % 3 == 0 { -1 } else { 1 };
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action {
                space: 3,
                x,
                ..Action::default()
            })
            .unwrap();
        settle(&mut h);
        expected += x as f32 * 3.0 * (1.0 / 60.0);
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.tick(), i);
        assert!(
            (p.last.as_ref().unwrap().document.objects[0].translation[0] - expected).abs() < 1e-3
        )
    }
    h.stop_play();
}

#[test]
fn rejected_nested_return_then_constant_source_recovers_through_actual_host_gpu() {
    let mut h = fixture();
    h.frame(None).unwrap();
    let original = (
        h.rt.capture(h.two, "before2").unwrap().0,
        h.rt.capture(h.three, "before3").unwrap().0,
    );
    let before = h.editor.isolation_identity();
    let path = dir("behavior-repair-source").join("main.at");
    std::fs::write(
        &path,
        "fn app_start(){let x=if true {return 1} else {return 2}}\nfn app_update(dt float){}",
    )
    .unwrap();
    h.select_behavior(Some(path.display().to_string())).unwrap();
    assert!(h
        .start_selected_play()
        .unwrap_err()
        .contains("callback must return Unit"));
    assert!(h.play.is_none());
    assert_eq!(h.editor.isolation_identity(), before);
    std::fs::write(&path, "const BASE=1.0\nconst SHIFT=BASE+2.0\nvar text=f\"literal\"\nvar raw=`safe\nuse nonexistent\n`\nfn app_start(){if text.len()==7 {ag_set_local(0,0,SHIFT+1.0)}}\nfn app_update(dt float){}").unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("verify repaired source");
    settle(&mut h);
    let owner = h.play.as_ref().unwrap().owner_probe().unwrap();
    assert_eq!(
        h.play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .unwrap()
            .document
            .groups[0]
            .translation[0],
        4.0
    );
    for (part, dx) in [-1.5, 1.5].into_iter().enumerate() {
        let point = autographics_auto_driver::authoring::interaction::project(
            h.camera(),
            glam::Vec3::new(3.0 + dx, 0.5, 0.0),
        )
        .unwrap();
        let pixel = h.rt.probe_pixel(h.top, [point.x, point.y + 400.0]).unwrap();
        assert!(
            pixel[part] > 80 && pixel[1 - part] < 20,
            "repaired source all-part GPU {pixel:?}"
        );
    }
    h.stop_play();
    h.frame(None).unwrap();
    assert!(!owner());
    assert_eq!(h.editor.isolation_identity(), before);
    assert_eq!(
        (
            h.rt.capture(h.two, "after2").unwrap().0,
            h.rt.capture(h.three, "after3").unwrap().0
        ),
        original
    );
}
