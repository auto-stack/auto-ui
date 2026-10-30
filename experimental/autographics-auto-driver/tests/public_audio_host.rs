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

// Finish the one candidate dispatched before Pause while the native cursor is
// frozen; wall-clock speed cannot turn a capacity test into a voice-expiry test.
fn frozen_tick(h: &mut Host, success: bool) {
    let p = h.play.as_mut().unwrap();
    p.resume().unwrap();
    p.char_key(1, true, false);
    p.dispatch(Action::default()).unwrap();
    h.pause_play("");
    pump(h, success);
}
fn next_silent_quantum(h: &Host) {
    let audio = h.audio.as_ref().unwrap();
    let before = audio.observation.lock().unwrap().callbacks;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let s = audio.observation.lock().unwrap();
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
#[test]
fn ninth_voice_refuses_whole_candidate_without_stealing_or_partial_visual() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let mut h = scene(api);
        let user = controller(api, "")
            .replace("0.5,2,step*2 - 1", "0.5,voices,step*2 - 1")
            .replace("0,2,step*2 - 1", "0,voices,step*2 - 1")
            .replace(
                "let packed=",
                "var voices=2;if step==5 {voices=1};let packed=",
            );
        // Unused wire rows must remain canonical when the ninth request is alone.
        let user = user.replace("ok=ag_present", "if step==5 {choice=0};ok=ag_present");
        // Present API canonicalizes slots beyond its count; only the first is used.
        start(&mut h, api, &user);
        for i in 1..=4 {
            frozen_tick(&mut h, true);
            assert_eq!(h.audio_snapshot()["voices"], i * 2);
        }
        let good = h.play.as_ref().unwrap().last.clone().unwrap();
        let before = h.audio_snapshot()["nonzero"].clone();
        h.frame(None).unwrap();
        let surface = if api == Api::Audio2V1 { h.two } else { h.three };
        let image = h.rt.capture(surface, "eight-voices").unwrap().0;
        frozen_tick(&mut h, false);
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.state, State::Faulted);
        assert!(
            p.error.as_ref().unwrap().contains("eight voices"),
            "{:?}",
            p.error
        );
        assert_eq!(p.last.as_ref().unwrap().tick, good.tick);
        assert_eq!(p.last.as_ref().unwrap().nodes, good.nodes);
        assert_eq!(
            h.audio_snapshot()["voices"],
            0,
            "Fault cancels existing voices"
        );
        assert_eq!(
            h.audio_snapshot()["nonzero"],
            before,
            "rejected voice never submitted"
        );
        assert_eq!(h.rt.capture(surface, "ninth-refused").unwrap().0, image);
        next_silent_quantum(&h);
        h.stop_play();
    }
}
#[test]
fn postaccept_native_error_preserves_visual_cancels_pcm_and_fresh_run_reopens_stream() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let mut h = scene(api);
        let source = controller(api, "");
        start(&mut h, api, &source);
        frozen_tick(&mut h, true);
        let oldrun = h.play.as_ref().unwrap().run_id();
        let oldowner = h.play.as_ref().unwrap().owner_probe().unwrap();
        let oldmanager = std::sync::Arc::downgrade(&h.audio.as_ref().unwrap().mixer);
        h.audio
            .as_ref()
            .unwrap()
            .mixer
            .lock()
            .unwrap()
            .pause(oldrun, false)
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while h.audio_snapshot()["nonzero"].as_u64().unwrap() == 0 {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        h.pause_play("");
        h.frame(None).unwrap();
        let frame = h.play.as_ref().unwrap().last.clone().unwrap();
        let surface = if api == Api::Audio2V1 { h.two } else { h.three };
        let image = h.rt.capture(surface, "postaccept").unwrap().0;
        // This is the exact terminal method used by CPAL's output-error callback.
        h.audio
            .as_ref()
            .unwrap()
            .mixer
            .lock()
            .unwrap()
            .output_fault("injected device disconnect".into());
        let emitted = h.audio_snapshot()["nonzero"].clone();
        for _ in 0..3 {
            h.frame(None).unwrap();
        }
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.state, State::Faulted);
        assert!(
            p.error
                .as_ref()
                .unwrap()
                .ends_with("audio_fault: injected device disconnect"),
            "{:?}",
            p.error
        );
        assert_eq!(p.last.as_ref().unwrap().tick, frame.tick);
        assert_eq!(p.last.as_ref().unwrap().audio, frame.audio);
        assert_eq!(
            h.rt.capture(surface, "fault-keeps-accepted").unwrap().0,
            image
        );
        assert!(!oldowner(), "Fault releases actual VM owner");
        assert!(h.play.as_mut().unwrap().resume().is_err());
        next_silent_quantum(&h);
        assert_eq!(
            h.audio_snapshot()["nonzero"],
            emitted,
            "no automatic retry or rollback"
        );
        h.stop_play();
        let decodes = assets::decode_attempts();
        start(&mut h, api, &source);
        assert!(
            oldmanager.upgrade().is_none(),
            "faulted stream replaced, not reset"
        );
        assert!(h.play.as_ref().unwrap().run_id() > oldrun);
        frozen_tick(&mut h, true);
        assert_eq!(
            h.play
                .as_ref()
                .unwrap()
                .last
                .as_ref()
                .unwrap()
                .audio
                .as_ref()
                .unwrap()
                .requests[0]
                .ordinal,
            1
        );
        assert_eq!(assets::decode_attempts(), decodes);
        h.stop_play();
    }
}
#[test]
fn twenty_runs_release_real_voice_vm_manager_and_pcm_owners_with_legitimate_external_pin() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let mut h = scene(api);
        let source = controller(api, "");
        let cues = public_audio::resolve_cues(&h.editor.document, &h.editor.assets).unwrap();
        let external = cues[0].pcm.clone();
        let pcm0 = std::sync::Arc::downgrade(&cues[0].pcm);
        let pcm1 = std::sync::Arc::downgrade(&cues[1].pcm);
        drop(cues);
        let decodes = assets::decode_attempts();
        let mut manager = None;
        let mut previous = 0;
        let uploads = h.resource_snapshot()["mesh_uploads"].clone();
        for cycle in 0..20 {
            start(&mut h, api, &source);
            let p = h.play.as_ref().unwrap();
            let run = p.run_id();
            assert!(run > previous);
            let owner = p.owner_probe().unwrap();
            let audio = h.audio.as_ref().unwrap();
            if let Some(weak) = &manager {
                assert!(std::sync::Weak::ptr_eq(
                    weak,
                    &std::sync::Arc::downgrade(&audio.mixer)
                ));
            } else {
                manager = Some(std::sync::Arc::downgrade(&audio.mixer));
            }
            assert!(audio.mixer.lock().unwrap().pause(previous, false).is_err());
            frozen_tick(&mut h, true);
            let p = h.play.as_ref().unwrap();
            assert_eq!(
                p.last.as_ref().unwrap().audio.as_ref().unwrap().requests[0].ordinal,
                1
            );
            assert_eq!(h.audio_snapshot()["voices"], 2);
            h.stop_play();
            assert!(!owner(), "{api:?} cycle{cycle}: VM actually released");
            assert_eq!(h.audio_snapshot()["voices"], 0);
            assert_eq!(h.audio_snapshot()["run"], 0);
            assert!(
                manager.as_ref().unwrap().upgrade().is_some(),
                "Host retains healthy stream across Stop"
            );
            next_silent_quantum(&h);
            assert_eq!(assets::decode_attempts(), decodes);
            assert_eq!(h.resource_snapshot()["mesh_uploads"], uploads);
            previous = run;
        }
        drop(h);
        assert!(
            manager.unwrap().upgrade().is_none(),
            "final Host close releases real manager"
        );
        assert!(pcm1.upgrade().is_none(), "unpinned static PCM released");
        assert!(
            pcm0.upgrade().is_some(),
            "legitimate external PCM pin retained"
        );
        drop(external);
        assert!(pcm0.upgrade().is_none(), "last legitimate owner released");
        println!("{api:?} 20Run cycles, zero warm decode/upload, actual VM/manager/PCM weak owners released");
    }
}

fn editor_command(h: &mut Host, action: &str, payload: serde_json::Value) -> Result<(), String> {
    h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,
        "asset_generation":h.ui().asset_token,"selection":h.editor.selection,"action":action,"payload":payload}).to_string())
}
fn import_budget_image(h: &mut Host, index: usize) -> Result<(), String> {
    editor_command(
        h,
        "import-sprite",
        json!(root().join(format!(
            "tests/fixtures/public_audio_budget/budget{index}.png"
        ))),
    )?;
    h.settle()
}
fn fresh_directory() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ui012-owner-budget-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn thin_image(path: &Path, height: usize) {
    let rgba = vec![97; 2048 * height * 4];
    std::fs::write(
        path,
        agp::assets::encode_png_rgba(2048, height as u32, &rgba).unwrap(),
    )
    .unwrap();
}
#[test]
fn retained_manager_budget_survives_new_open_and_refuses_boundary_image_before_decode() {
    use autographics_auto_driver::authoring::{session::Editor, textured_models};
    for (api, open) in [(Api::Audio2V1, false), (Api::Audio3V1, true)] {
        let dir = fresh_directory();
        let mut h = scene(api);
        start(&mut h, api, &controller(api, ""));
        h.stop_play();
        let manager = std::sync::Arc::downgrade(&h.audio.as_ref().unwrap().mixer);
        if open {
            let mut editor = Editor::new(Document::empty("empty-open".into())).unwrap();
            let path = dir.join("empty.agscene");
            Storage::default()
                .save(&mut editor, &path, Fault::None)
                .unwrap();
            editor_command(&mut h, "open", json!(path)).unwrap();
        } else {
            editor_command(&mut h, "new", json!(null)).unwrap();
        }
        h.settle().unwrap();
        let reserve = h.audio_snapshot()["reserved_cpu_bytes"].as_u64().unwrap();
        assert_eq!(
            h.resource_snapshot()["asset_union"]["cpu_total_with_pose_bytes"],
            reserve
        );
        assert_eq!(h.resource_snapshot()["asset_union"]["count"], 0);
        for i in 0..3 {
            import_budget_image(&mut h, i).unwrap();
        }
        let before = h.editor.document.clone();
        let decoded = assets::decode_attempts();
        let error = import_budget_image(&mut h, 3).unwrap_err();
        assert!(error.contains("resource limit"), "{error}");
        assert_eq!(
            assets::decode_attempts(),
            decoded,
            "fourth image refused before allocation/decode"
        );
        assert_eq!(h.editor.document, before);
        assert!(!h.editor.pending());
        assert_eq!(
            h.resource_snapshot()["asset_union"]["cpu_total_with_pose_bytes"],
            3 * 16 * 1024 * 1024 + reserve
        );
        // Four fewer rows save32768B, insufficient for this33528B service.
        let thin = dir.join("too-close.png");
        thin_image(&thin, 2044);
        assert!(editor_command(&mut h, "import-sprite", json!(thin))
            .unwrap_err()
            .contains("resource limit"));
        assert_eq!(assets::decode_attempts(), decoded);
        // Five fewer rows save40960B. This real fourth identity is legal.
        let fits = dir.join("fits.png");
        thin_image(&fits, 2043);
        editor_command(&mut h, "import-sprite", json!(fits)).unwrap();
        h.settle().unwrap();
        let usage = h.resource_snapshot();
        assert_eq!(
            usage["asset_union"]["cpu_total_with_pose_bytes"],
            64 * 1024 * 1024 - 40960 + reserve
        );
        assert!(
            manager.upgrade().is_some(),
            "New/Open must retain healthy Host manager"
        );
        let decoded = assets::decode_attempts();
        import_budget_image(&mut h, 0).unwrap();
        assert_eq!(
            assets::decode_attempts(),
            decoded,
            "duplicate remains admissible at boundary"
        );
        // First tiny audio import replaces this manager-only reservation with
        // the audio Asset's existing conservative reserve, rather than doubling.
        let sound = dir.join("tiny.wav");
        let mut wav = b"RIFF".to_vec();
        wav.extend_from_slice(&100u32.to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&22050u32.to_le_bytes());
        wav.extend_from_slice(&44100u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&64u32.to_le_bytes());
        for _ in 0..32 {
            wav.extend_from_slice(&300i16.to_le_bytes());
        }
        std::fs::write(&sound, wav).unwrap();
        editor_command(&mut h, "import-audio", json!(sound)).unwrap();
        h.settle().unwrap();
        assert_eq!(
            h.resource_snapshot()["asset_union"]["audio_manager_extra_cpu_bytes"],
            0
        );
        assert!(textured_models::usage(&h.editor.assets).2 <= 64 * 1024 * 1024);
        let audio = h
            .editor
            .assets
            .values()
            .find(|a| a.audio.is_some())
            .unwrap();
        assert_eq!(
            h.resource_snapshot()["asset_union"]["cpu_total_with_pose_bytes"]
                .as_u64()
                .unwrap(),
            (64 * 1024 * 1024 - 40960 + audio.cpu_bytes) as u64
        );
    }
}
#[test]
fn empty_cue_run_preflights_new_service_and_legacy_full_image_budget_stays_accepted() {
    use autographics_auto_driver::authoring::document::Object;
    let mut doc = Document::empty("legacy-full-images".into());
    let mut actor = Object::new(1, Kind::Quad);
    actor.scale = [0.8, 1.0, 1.0];
    doc.objects.push(actor);
    let mut h = Host::with_assets(doc, Registry::new()).unwrap();
    for i in 0..4 {
        import_budget_image(&mut h, i).unwrap();
    }
    assert_eq!(h.audio_snapshot()["manager"], false);
    assert_eq!(
        h.resource_snapshot()["asset_union"]["cpu_total_with_pose_bytes"],
        64 * 1024 * 1024
    );
    // The very same pixels remain accepted in a scene7 with zero audio assets.
    let mut doc = h.editor.document.clone();
    doc.schema = "autographics.scene/7".into();
    doc.cues = Some(vec![]);
    let mut h = Host::with_assets(doc, h.editor.assets.clone()).unwrap();
    let source=SourceSnapshot::from_bytes_for(Path::new("empty-cues.at"),b"var ok=0\nfn app_start(){ok=ag_char_bind(1);ok=ag_camera_follow_2d(1,-10.0,-10.0,10.0,10.0,4.0,4.0)}\nfn app_update(dt float){}\n".to_vec(),Api::Audio2V1).unwrap();
    let before = h.editor.document.clone();
    let camera = h.camera();
    let error = h.start_user_snapshot(source).unwrap_err();
    assert!(error.contains("resource limit"), "{error}");
    assert!(
        h.audio.is_none(),
        "native allocation must follow service-budget admission"
    );
    assert!(h.play.is_none());
    assert_eq!(h.editor.document, before);
    assert_eq!(h.camera(), camera);
}

#[test]
fn empty_audio_service_and_animation_pose_share_one_cpu_ceiling() {
    use autographics_auto_driver::authoring::public_audio::native::Native;
    use autographics_auto_driver::authoring::{hierarchy, runtime_node_animation, textured_models};
    let mut r = Registry::new();
    let model = assets::import_animated(
        &root().join("samples/textured_models/models/character.glb"),
        &r,
    )
    .unwrap();
    let mut doc = Document::empty("pose-service-boundary".into());
    doc.schema = "autographics.scene/7".into();
    doc.cues = Some(vec![]);
    let mut actor = Object::new(1, Kind::Model);
    actor.resource = Some(model.identity.key());
    let (lo, hi) = autographics_auto_driver::authoring::interaction::bounds(&model.model).unwrap();
    let extent = hi - lo;
    actor.scale = [0.8 / extent.x, 1.0 / extent.y, 0.8 / extent.z];
    doc.objects.push(actor);
    assets::insert(&mut r, model).unwrap();
    for i in 0..3 {
        let image = assets::import_png(
            &root().join(format!("tests/fixtures/public_audio_budget/budget{i}.png")),
            &r,
        )
        .unwrap();
        assets::insert(&mut r, image).unwrap();
    }
    doc.assets = r.values().map(|a| a.identity.clone()).collect();
    let targets =
        runtime_node_animation::targets(&doc, &hierarchy::ordered(&doc).unwrap(), &r).unwrap();
    let pose = runtime_node_animation::pose_reservation(&targets);
    let manager = Native::reserved_bytes();
    let cap = assets::MAX_GEOMETRY;
    let margin = (pose + manager + pose.max(manager)) / 2;
    let height = (cap - textured_models::usage(&r).2 - margin) / (2048 * 4);
    let pixels = vec![113; 2048 * height * 4];
    let image = assets::decode_png_in(
        agp::assets::encode_png_rgba(2048, height as u32, &pixels).unwrap(),
        None,
        &r,
    )
    .unwrap();
    assets::insert(&mut r, image).unwrap();
    doc.assets = r.values().map(|a| a.identity.clone()).collect();
    let cpu = textured_models::usage(&r).2;
    assert!(
        cpu + pose <= cap && cpu + manager <= cap && cpu + pose + manager > cap,
        "distinguish combined owners, not either independent check"
    );
    let mut h = Host::with_assets(doc.clone(), r.clone()).unwrap();
    let raw=b"var ok=0\nfn app_start(){ok=ag_char3_bind(1);ok=ag_camera_orbit_3d(1,8.0,45.0,0,0.0)}\nfn app_update(dt float){}\n";
    let source =
        SourceSnapshot::from_bytes_for(Path::new("pose-service.at"), raw.to_vec(), Api::Audio3V1)
            .unwrap();
    let error = h.start_user_snapshot(source).unwrap_err();
    assert!(error.contains("asset/service/pose CPU union"), "{error}");
    assert!(h.audio.is_none() && h.play.is_none());
    drop(h);
    // Identical resources/pose remain legal for the delivered legacy profile
    // when it does not require or retain a native manager.
    doc.schema = "autographics.scene/6".into();
    doc.cues = None;
    let mut h = Host::with_assets(doc, r).unwrap();
    let source = SourceSnapshot::from_bytes_for(
        Path::new("legacy-pose.at"),
        raw.to_vec(),
        Api::NodeAnimation3V1,
    )
    .unwrap();
    h.start_user_snapshot(source).unwrap();
    h.pause_play("");
    pump(&mut h, true);
    assert_eq!(h.audio_snapshot()["manager"], false);
    h.stop_play();
}

#[test]
fn retained_service_is_in_every_model_closure_predecode_forecast() {
    use autographics_auto_driver::authoring::{session::Editor, textured_models};
    for (model_name, animated, opening) in [
        ("platform.glb", false, false),
        ("character.glb", true, false),
        ("platform.glb", false, true),
    ] {
        let model_path = root()
            .join("samples/textured_models/models")
            .join(model_name);
        let mut r = Registry::new();
        let model = if animated {
            assets::import_animated(&model_path, &r)
        } else {
            assets::import(&model_path, &r)
        }
        .unwrap();
        let identity = model.identity.clone();
        assets::insert(&mut r, model).unwrap();
        let model_cpu = textured_models::usage(&r).2;
        let dir = fresh_directory();
        let project = dir.join("scene.agscene");
        if opening {
            let mut doc = Document::empty("closure-load-control".into());
            doc.schema = "autographics.scene/6".into();
            doc.assets = r.values().map(|a| a.identity.clone()).collect();
            let mut editor = Editor::with_assets(doc, r.clone()).unwrap();
            Storage::default()
                .save(&mut editor, &project, Fault::None)
                .unwrap();
            let hash = agp::renderer::capture::sha256_hex(&std::fs::read(&project).unwrap());
            assert_eq!(
                hash, "2fbf5eb26981188f62c8aea7af096e9ac37862345398567109acf7bf8d3cb0aa",
                "exact R2declared closure input"
            );
        }
        drop(r);
        for manager in [false, true] {
            let mut h = if manager {
                let (d, r, c, _) = Storage::load_project(
                    &root().join("samples/public_audio/project2d/scene.agscene"),
                    &Registry::new(),
                )
                .unwrap();
                let mut h = Host::with_project(d, r, c).unwrap();
                h.start_selected_play().unwrap();
                h.pause_play("");
                pump(&mut h, true);
                h.stop_play();
                editor_command(&mut h, "new", json!(null)).unwrap();
                h.settle().unwrap();
                h
            } else {
                Host::with_assets(Document::empty("legacy-control".into()), Registry::new())
                    .unwrap()
            };
            for i in 0..3 {
                import_budget_image(&mut h, i).unwrap();
            }
            let boundary = root()
                .join("tests/fixtures/public_audio_budget")
                .join(if animated {
                    "animated-boundary.png"
                } else {
                    "textured-boundary.png"
                });
            editor_command(&mut h, "import-sprite", json!(boundary)).unwrap();
            h.settle().unwrap();
            let images = textured_models::usage(&h.editor.assets).2;
            let service =
                autographics_auto_driver::authoring::public_audio::native::Native::reserved_bytes();
            assert!(
                images + model_cpu <= assets::MAX_GEOMETRY
                    && images + model_cpu + service > assets::MAX_GEOMETRY
            );
            let before = h.editor.document.clone();
            let decoded = assets::decode_attempts();
            let result = if opening {
                editor_command(&mut h, "open", json!(&project)).unwrap();
                editor_command(&mut h, "leave-discard", json!(null))
            } else {
                editor_command(
                    &mut h,
                    if animated {
                        "import-animated"
                    } else {
                        "import"
                    },
                    json!(&model_path),
                )
            };
            let delta = assets::decode_attempts() - decoded;
            if manager {
                assert!(result.unwrap_err().contains("resource limit"));
                assert_eq!(
                    delta, 0,
                    "{model_name} opening={opening}: full closure refused BEFORE allocation/decode"
                );
                assert_eq!(h.editor.document, before);
                assert!(!h.editor.pending());
            } else {
                result.unwrap();
                h.settle().unwrap();
                assert_eq!(delta, 2, "actual positive image+geometry decode");
                assert!(h.editor.assets.contains_key(&identity.key()));
                assert_eq!(h.audio_snapshot()["manager"], false);
            }
            println!("closure forecast {model_name} opening={opening} manager={manager} decode_delta={delta}");
        }
    }
}
