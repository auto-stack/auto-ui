//! The same entrypoint is compiled/run in the independent exported consumer.
use autographics_auto_driver::authoring::{
    assets::Registry,
    behavior::{Api, SourceSnapshot},
    document::{Kind, Object},
    host::Host,
    play::{Action, State},
    storage::{Fault, Saved, Storage},
};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
fn entry() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/node_animation/scene.agscene")
}
fn pump(h: &mut Host) {
    let end = Instant::now() + Duration::from_secs(60);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < end);
        h.frame(None).unwrap();
    }
}
fn input(h: &mut Host, bits: u8) {
    let p = h.play.as_mut().unwrap();
    p.resume().unwrap();
    for k in 0..6 {
        p.char_key(k, bits & (1 << k) != 0, false);
    }
    p.cam_drag(0.5, 0.);
    p.dispatch(Action::default()).unwrap();
    h.pause_play("");
    pump(h);
}
fn host(path: &Path) -> Host {
    let (d, r, c, _) = Storage::load_project(path, &Registry::new()).unwrap();
    assert!(c.is_some());
    assert!(d.assets[0].closure.as_ref().unwrap().node_animation);
    Host::with_project(d, r, c).unwrap()
}
fn start(h: &mut Host) {
    h.prepare_measurement.enable(true);
    h.start_selected_play().unwrap();
    h.pause_play("");
    pump(h);
    assert!(
        h.play.as_ref().unwrap().error.is_none(),
        "{:?}",
        h.play.as_ref().unwrap().error
    );
}

#[test]
fn bound_original_project_cold_save_move_reopen_and_input_sensitive_user_source() {
    let mut h = host(&entry());
    start(&mut h);
    let first = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
    let initial = first.nodes[&1].translation;
    let upload = h.resource_snapshot()["mesh_uploads"].clone();
    let texture_upload = h.resource_snapshot()["model_texture_uploads"].clone();
    for tick in 1..=60 {
        input(&mut h, if tick == 5 { 4 | 16 } else { 4 });
        assert!(
            h.play.as_ref().unwrap().error.is_none(),
            "{:?}",
            h.play.as_ref().unwrap().error
        );
    }
    let p = h.play.as_ref().unwrap();
    let f = p.last.as_ref().unwrap();
    assert_ne!(f.nodes[&1].translation, initial);
    assert_eq!(f.tick, 60);
    assert!((f.animations.as_ref().unwrap().visual_for(1).unwrap().phase - 1.).abs() < 1e-5);
    assert!(f.camera.is_some() && f.rounds.is_some() && f.instances.is_some());
    assert!(!f.rounds.as_ref().unwrap().events.is_empty());
    let steps = p.callback_high_water().unwrap();
    assert!(steps.0 > 0 && steps.0 <= 100000 && steps.1 <= 121);
    assert_eq!(h.resource_snapshot()["mesh_uploads"], upload);
    assert_eq!(
        h.resource_snapshot()["model_texture_uploads"],
        texture_upload
    );
    h.stop_play();
    let root = std::env::temp_dir().join(format!(
        "ui006-consumer-project-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!root.exists());
    std::fs::create_dir_all(root.join("a")).unwrap();
    assert_eq!(
        h.storage
            .save(&mut h.editor, &root.join("a/scene.agscene"), Fault::None)
            .unwrap(),
        Saved::Confirmed
    );
    let code = h.editor.document.behavior.as_ref().unwrap().source.clone();
    drop(h);
    std::fs::rename(root.join("a"), root.join("b")).unwrap();
    let mut h = host(&root.join("b/scene.agscene"));
    start(&mut h);
    assert_eq!(h.editor.document.behavior.as_ref().unwrap().source, code);
    assert_eq!(
        h.editor
            .assets
            .values()
            .next()
            .unwrap()
            .animation
            .as_ref()
            .unwrap()
            .clips
            .len(),
        4
    );
    assert_eq!(
        h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&1].translation,
        initial
    );
    assert_eq!(
        h.play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .unwrap()
            .animations
            .as_ref()
            .unwrap()
            .visual_for(1)
            .unwrap()
            .phase,
        0.
    );
    input(&mut h, 2);
    let normal = h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&1].translation;
    h.stop_play();
    // A real user-source mutation changes the trajectory via the public API;
    // the Host has no movement/clip choice hidden in its implementation.
    let raw = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/node_animation/main.at"),
    )
    .unwrap();
    let changed = raw.replace("var speed=4.0", "var speed=1.0");
    assert_ne!(raw, changed);
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("edited-controller.at"),
        changed.into_bytes(),
        Api::NodeAnimation3V1,
    )
    .unwrap();
    h.start_user_snapshot(snapshot).unwrap();
    h.pause_play("");
    pump(&mut h);
    input(&mut h, 2);
    let edited = h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&1].translation;
    assert!((normal[0] - initial[0]).abs() > (edited[0] - initial[0]).abs() * 3.9);
    h.stop_play();
    let source = SourceSnapshot::read_for(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/node_animation/main.at"),
        Api::NodeAnimation3V1,
    )
    .unwrap();
    let managed = source.source.len() - source.bytes.len();
    assert!(managed <= 96 * 1024);
    let value = json!({"manifest_dir":env!("CARGO_MANIFEST_DIR"),"cold_reopen":true,"moved_project":true,"clips":4,"ticks":60,"user_mutation_sensitive":true,"callback_steps":steps.0,"callback_slices":steps.1,"managed_bytes":managed,"warm_mesh_uploads":0,"warm_texture_uploads":0});
    println!("UI006 consumer {value}");
    report("consumer", &value);
}
fn report(name: &str, value: &serde_json::Value) {
    if let Some(root) = std::env::var_os("UI006_REPORT_DIR") {
        let root = PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(format!("{name}.json")),
            serde_json::to_vec_pretty(value).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn inherited_regular_sparse_and_deep_cpu_envelope_uses_unchanged_guards() {
    let mut values = Vec::new();
    for (shape, count, accepted) in [
        ("regular", 3, true),
        ("wide", 120, true),
        ("deep", 64, true),
        ("deep", 120, false),
    ] {
        let (mut d, r, c, _) = Storage::load_project(&entry(), &Registry::new()).unwrap();
        for id in 3..=count {
            let mut o = Object::new(id, Kind::Cylinder);
            o.visible = false;
            if shape == "deep" {
                o.translation = [0., -0.5, 0.];
                o.scale = [10., 0.5, 10.];
            } else {
                o.translation = [10. + id as f32, 0., 0.];
                o.scale = [0.1, 0.5, 0.1];
            }
            d.objects.push(o);
        }
        let mut h = Host::with_project(d, r, c).unwrap();
        start(&mut h);
        let old = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
        let mut durations = Vec::new();
        for tick in 1..=if accepted { 12 } else { 1 } {
            input(&mut h, if tick == 3 { 4 | 16 } else { 4 });
            if accepted {
                assert!(
                    h.play.as_ref().unwrap().error.is_none(),
                    "{shape}{count}: {:?}",
                    h.play.as_ref().unwrap().error
                );
                durations.push(h.play.as_ref().unwrap().reports.back().unwrap().elapsed_us as u64);
            }
        }
        if accepted {
            let p = h.play.as_ref().unwrap();
            let (steps, slices) = p.callback_high_water().unwrap();
            assert!(steps <= 100000 && slices <= 121);
            durations.sort();
            let resources = h.resource_snapshot();
            let union = &resources["asset_union"];
            assert!(union["cpu_total_with_pose_bytes"].as_u64().unwrap() <= 64 * 1024 * 1024);
            assert!(union["gpu_expected_bytes"].as_u64().unwrap() <= 64 * 1024 * 1024);
            values.push(json!({"shape":shape,"nodes":count,"accepted":true,"steps":steps,"slices":slices,"tick_median_us":durations[6],"tick_max_us":durations[11],"resources":union}));
        } else {
            let p = h.play.as_ref().unwrap();
            assert_eq!(p.state, State::Faulted);
            assert!(
                p.error.as_ref().unwrap().contains("budget"),
                "{:?}",
                p.error
            );
            assert_eq!(p.tick(), old.tick);
            assert_eq!(p.last.as_ref().unwrap().nodes, old.nodes);
            values.push(json!({"shape":shape,"nodes":count,"accepted":false,"last_good_preserved":true,"error":p.error}));
        }
        h.stop_play();
    }
    let value = json!({"layouts":values,"guards":{"steps":100000,"resumptions":120,"callback_ms":250,"tick_ms":2000},"wide_base_plus_slots":128});
    println!("UI006 performance {value}");
    report("performance", &value);
}
