//! PLAN-012 authoring path: accepted resources → real Auto → one frame/PCM accept.
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object, SpriteBinding},
    host::Host,
    play::{Action, State},
    public_audio::{self, CueRef},
    storage::{Fault, Saved, Storage},
};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_owned()
}
fn scene(api: Api) -> Host {
    let mut registry = Registry::new();
    let mut doc = Document::empty("audio-host".into());
    doc.schema = "autographics.scene/7".into();
    doc.cues = Some(vec![]);
    let mut actor = Object::new(
        1,
        if api == Api::Audio3V1 {
            Kind::Model
        } else {
            Kind::Quad
        },
    );
    if api == Api::Audio3V1 {
        let asset = assets::import_animated(
            &root().join("samples/textured_models/models/character.glb"),
            &registry,
        )
        .unwrap();
        let (lo, hi) =
            autographics_auto_driver::authoring::interaction::bounds(&asset.model).unwrap();
        let extent = hi - lo;
        actor.scale = [0.8 / extent.x, 1.0 / extent.y, 0.8 / extent.z];
        actor.resource = Some(asset.identity.key());
        doc.assets.push(asset.identity.clone());
        assets::insert(&mut registry, asset).unwrap();
    } else {
        let mut pixels = vec![];
        for _ in 0..8 {
            for x in 0..8 {
                pixels.extend_from_slice(if x < 4 {
                    &[255, 0, 0, 255]
                } else {
                    &[0, 0, 255, 255]
                });
            }
        }
        let bytes = agp::assets::encode_png_rgba(8, 8, &pixels).unwrap();
        let asset = assets::decode_png_in(bytes, Some("two-colors.png".into()), &registry).unwrap();
        actor.scale = [0.8, 1.0, 1.0];
        actor.sprite = Some(SpriteBinding {
            asset: asset.identity.key(),
            frame: [0, 0, 4, 8],
            flip: false,
        });
        doc.assets.push(asset.identity.clone());
        assets::insert(&mut registry, asset).unwrap();
    }
    doc.objects.push(actor);
    let mut template = Object::new(
        3,
        if api == Api::Audio3V1 {
            Kind::Cylinder
        } else {
            Kind::Circle
        },
    );
    template.translation = [20.0, 0.0, 0.0];
    template.scale = [0.3, 0.5, if api == Api::Audio3V1 { 0.3 } else { 1.0 }];
    doc.objects.push(template);
    for (i, name) in ["jump.ogg", "coin.ogg"].into_iter().enumerate() {
        let asset = public_audio::import(
            &root().join("samples/public_audio/sounds").join(name),
            &registry,
        )
        .unwrap();
        doc.cues.as_mut().unwrap().push(CueRef {
            name: format!("sound{i}"),
            asset: asset.identity.key(),
            start_frame: 0,
            end_frame: asset.identity.audio.as_ref().unwrap().frames,
            gain: 1.0,
        });
        doc.assets.push(asset.identity.clone());
        assets::insert(&mut registry, asset).unwrap();
    }
    Host::with_assets(doc, registry).unwrap()
}
fn controller(api: Api, bad: &str) -> String {
    let (start, motion, present) = if api == Api::Audio3V1 {
        ("ok=ag_char3_bind(1);ok=ag_camera_orbit_3d(1,8.0,45.0,0,0.0)","let jump=(pressed/16)%2;let right=(held/2)%2;let fx=ag_cam3_right_x();let fz=ag_cam3_right_z();let token=ag_char3_move(right*dt*fx,jump*dt,right*dt*fz,0.002);let grounded=ag_char3_grounded(token)","ag_present3d(0,1,2,step*0.1,1,0.5,2,step*2 - 1,choice,0.25,step*2,1 - choice,0.5,0,0,0.0,0,0,0.0)")
    } else {
        ("ok=ag_char_bind(1);ok=ag_camera_follow_2d(1,-10.0,-10.0,10.0,10.0,4.0,4.0)","let jump=(pressed/4)%2;let right=(held/2)%2;let token=ag_char_move(right*dt,jump*dt,0.002);let grounded=ag_char_grounded(token)","ag_present2d(0,1,(step%2)*4,0,4,8,0,2,step*2 - 1,choice,0.25,step*2,1 - choice,0.5,0,0,0.0,0,0,0.0)")
    };
    format!("var ok=0\nvar step=0\nvar observed=0\nfn app_start(){{{start};let obj=ag_object_spawn(0,3,0.0,0.0,0.0);ok=ag_trigger_bind(0,1,1,obj,0,0.0)}}\nfn app_update(dt float){{step=step+1;let packed=ag_actions_snapshot();let held=packed%64;let pressed=packed/64;{motion};let event=ag_event_next();let pair=ag_event_pair();let phase=ag_event_phase();observed=event*100+pair*10+phase;var choice=0;if observed>=100 {{choice=1}};ok={present};{bad}}}\n")
}
fn pump(h: &mut Host, success: bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < deadline, "pump deadline");
        h.frame(None).unwrap();
    }
    if success {
        assert_ne!(
            h.play.as_ref().unwrap().state,
            State::Faulted,
            "{:?}",
            h.play.as_ref().unwrap().error
        )
    }
}
fn start(h: &mut Host, api: Api, user: &str) {
    let source = SourceSnapshot::from_bytes_for(
        Path::new("public-audio-controller.at"),
        user.as_bytes().to_vec(),
        api,
    )
    .unwrap();
    h.start_user_snapshot(source).unwrap();
    h.pause_play("");
    pump(h, true);
}
fn tick(h: &mut Host, neutral: bool, success: bool) {
    let p = h.play.as_mut().unwrap();
    if neutral {
        p.single_step().unwrap();
    } else {
        p.resume().unwrap();
        p.char_key(1, true, false);
        p.dispatch(Action::default()).unwrap();
    }
    pump(h, success);
    if success && !neutral {
        h.pause_play("");
    }
}
#[test]
fn authoring_host_two_dimensions_accept_real_cues_with_same_frame_visual_and_native_manager() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let mut h = scene(api);
        let user = controller(api, "");
        start(&mut h, api, &user);
        let initial = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
        assert_eq!(initial.audio.as_ref().unwrap().requests.len(), 0);
        let decodes = assets::decode_attempts();
        let uploads = h.resource_snapshot()["mesh_uploads"].clone();
        tick(&mut h, false, true);
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
        assert_eq!(frame.tick, 1);
        assert_eq!(frame.audio.as_ref().unwrap().tick, 1);
        assert_eq!(frame.audio.as_ref().unwrap().requests.len(), 2);
        assert_ne!(frame.nodes[&1].translation, initial.nodes[&1].translation);
        if api == Api::Audio3V1 {
            assert!(
                (frame
                    .animations
                    .as_ref()
                    .unwrap()
                    .visual_for(1)
                    .unwrap()
                    .phase
                    - 0.1)
                    .abs()
                    < 1e-6
            )
        } else {
            assert_eq!(
                frame.sprites.as_ref().unwrap().targets[0].frame,
                [4, 0, 4, 8]
            );
        }
        assert_eq!(assets::decode_attempts(), decodes);
        assert_eq!(h.resource_snapshot()["mesh_uploads"], uploads);
        tick(&mut h, false, true);
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
        assert_eq!(frame.tick, 2);
        assert_eq!(
            frame.audio.as_ref().unwrap().requests[0].cue,
            1,
            "real event fields select another cue"
        );
        assert_eq!(frame.audio.as_ref().unwrap().requests[1].cue, 0);
        assert_eq!(assets::decode_attempts(), decodes);
        assert_eq!(h.resource_snapshot()["mesh_uploads"], uploads);
        let token = h.ui().play_token;
        h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"play_token":token,"action":"play-resume"}).to_string()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while h
            .audio
            .as_ref()
            .unwrap()
            .observation
            .lock()
            .unwrap()
            .nonzero
            == 0
        {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        h.pause_play("");
        let output = h.audio_snapshot();
        assert!(output["nonzero"].as_u64().unwrap() > 0);
        assert!(output["peak"].as_f64().unwrap() > 0.01);
        println!(
            "{api:?} native={output} assets={}",
            h.resource_snapshot()["asset_union"]
        );
        h.stop_play();
        assert!(h.audio.is_some(), "Host manager survives Stop");
        assert_eq!(h.audio_snapshot()["voices"], 0);
        assert_eq!(h.audio_snapshot()["run"], 0);
    }
}
#[test]
fn versioned_audio_closure_saves_relocates_and_reopens_without_import_directory() {
    let mut h = scene(Api::Audio2V1);
    let directory = std::env::temp_dir().join(format!(
        "ui012-closure-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(directory.join("a")).unwrap();
    let entry = directory.join("a/scene.agscene");
    assert_eq!(
        h.storage.save(&mut h.editor, &entry, Fault::None).unwrap(),
        Saved::Confirmed
    );
    let bytes = std::fs::read(&entry).unwrap();
    assert_eq!(Document::decode(&bytes).unwrap(), h.editor.document);
    std::fs::rename(directory.join("a"), directory.join("b")).unwrap();
    drop(h);
    let (doc, registry, _) =
        Storage::load_assets(&directory.join("b/scene.agscene"), &Registry::new()).unwrap();
    assert_eq!(doc.cues.as_ref().unwrap().len(), 2);
    let cues = public_audio::resolve_cues(&doc, &registry).unwrap();
    assert_eq!(cues.len(), 2);
    let mut h = Host::with_assets(doc, registry).unwrap();
    start(&mut h, Api::Audio2V1, &controller(Api::Audio2V1, ""));
    tick(&mut h, false, true);
    h.stop_play();
}

#[test]
fn failed_audio_or_gpu_candidate_preserves_last_frame_and_stops_run_without_fault_rewrite() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        for gpu in [false, true] {
            let mut h = scene(api);
            let user = controller(
                api,
                if gpu {
                    ""
                } else {
                    "if step==2 {let excess=ag_actions_snapshot();}"
                },
            );
            start(&mut h, api, &user);
            tick(&mut h, false, true);
            let good = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
            h.frame(None).unwrap();
            let surface = if api == Api::Audio2V1 { h.two } else { h.three };
            let image = h.rt.capture(surface, "audio-last-good").unwrap().0;
            h.fail_resource = gpu;
            tick(&mut h, false, false);
            let play = h.play.as_ref().unwrap();
            assert_eq!(play.state, State::Faulted);
            let error = play.error.clone().unwrap();
            assert!(
                error.contains(if gpu { "resource preparation" } else { "10" }),
                "{error}"
            );
            assert_eq!(play.last.as_ref().unwrap().tick, good.tick);
            assert_eq!(play.last.as_ref().unwrap().nodes, good.nodes);
            assert_eq!(play.last.as_ref().unwrap().audio, good.audio);
            assert_eq!(h.audio_snapshot()["voices"], 0);
            h.fail_resource = false;
            for _ in 0..3 {
                h.frame(None).unwrap();
            }
            assert_eq!(
                h.play.as_ref().unwrap().error.as_deref(),
                Some(error.as_str()),
                "later pumps preserve causal fault"
            );
            assert_eq!(
                h.rt.capture(surface, "audio-fault-keeps-visual").unwrap().0,
                image
            );
            h.stop_play();
        }
    }
}
#[test]
fn neutral_step_consumes_audio_ordinals_without_voice_or_resume_backlog() {
    use autographics_auto_driver::authoring::public_audio::mixer::Accepted;
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let mut h = scene(api);
        start(&mut h, api, &controller(api, ""));
        for _ in 0..2 {
            tick(&mut h, true, true);
            let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
            assert_eq!(frame.audio_disposition, Some(Accepted::SuppressedByStep));
            assert_eq!(h.audio_snapshot()["voices"], 0);
            assert_eq!(h.audio_snapshot()["nonzero"], 0);
        }
        let step_tick = h.play.as_ref().unwrap().last.as_ref().unwrap().tick;
        assert_eq!(step_tick, 2);
        tick(&mut h, false, true);
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
        assert_eq!(frame.audio.as_ref().unwrap().requests[0].ordinal, 5);
        assert_eq!(frame.audio_disposition, Some(Accepted::Played));
        assert!(
            h.audio_snapshot()["voices"].as_u64().unwrap() <= 2,
            "suppressed steps did not accumulate voices"
        );
        h.stop_play();
        let service = h.audio.as_ref().unwrap();
        let before = service.observation.lock().unwrap().callbacks;
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let s = service.observation.lock().unwrap();
            if s.callbacks > before {
                assert_eq!(s.last_run, 0);
                assert_eq!(s.last_nonzero, 0);
                break;
            }
            drop(s);
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
    }
}

#[test]
fn repeated_render_capture_and_multiple_portals_do_not_resubmit_accepted_sound() {
    use agp::core::composition::{MountKind, Rect};
    use autographics_auto_driver::authoring::public_audio::mixer::Accepted;
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let mut h = scene(api);
        start(&mut h, api, &controller(api, ""));
        tick(&mut h, false, true);
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
        let cues = h.play.as_ref().unwrap().audio_cues.clone().unwrap();
        let mixer = h.audio.as_ref().unwrap().mixer.clone();
        let observe = h.audio.as_ref().unwrap().observation.clone();
        let voices = mixer.lock().unwrap().voices();
        let nonzero = observe.lock().unwrap().nonzero;
        let surface = if api == Api::Audio2V1 { h.two } else { h.three };
        h.frame(None).unwrap();
        let before = h.rt.capture(surface, "single-mount-audio").unwrap().0;
        let view = h.rt.graph.surface(h.top).unwrap().views[0];
        let mut portals = vec![];
        for i in 0..2 {
            portals.push(
                h.rt.add_portal(
                    view,
                    surface,
                    MountKind::Flat2d {
                        offset: [20.0 + i as f32 * 100.0, 20.0],
                        scale: [0.25, 0.25],
                        clip: Rect::at(0.0, 0.0, 600.0, 600.0),
                    },
                    false,
                )
                .unwrap(),
            );
        }
        for _ in 0..5 {
            h.frame(None).unwrap();
            assert_eq!(
                h.rt.capture(surface, "multiple-mount-audio").unwrap().0,
                before
            );
            assert_eq!(
                h.play.as_ref().unwrap().last.as_ref().unwrap().tick,
                frame.tick
            );
        }
        assert_eq!(mixer.lock().unwrap().voices(), voices);
        assert_eq!(
            observe.lock().unwrap().nonzero,
            nonzero,
            "paused presentation/capture cannot create PCM"
        );
        assert_eq!(
            mixer
                .lock()
                .unwrap()
                .accept(frame.audio.unwrap(), &cues, false, || panic!(
                    "accepted transport replay must not recommit visual"
                ))
                .unwrap(),
            Accepted::Replay
        );
        assert_eq!(mixer.lock().unwrap().voices(), voices);
        for portal in portals {
            h.rt.remove_portal(portal).unwrap();
        }
        h.stop_play();
    }
}
