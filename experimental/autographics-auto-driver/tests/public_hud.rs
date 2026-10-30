use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot, MAX_BRIDGE},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

fn three(api: Api) -> bool {
    api == Api::Hud3V1
}

fn session(api: Api, user: &str) -> Session {
    let user = user.replace(
        "fn app_start(){",
        if three(api) {
            "fn app_start(){let bound=ag_char3_bind(1);let camera=ag_camera_orbit_3d(1,8.0,45.0,0,0.0);"
        } else {
            "fn app_start(){let bound=ag_char_bind(1);let camera=ag_camera_follow_2d(1,-10.0,-10.0,10.0,10.0,4.0,4.0);"
        },
    );
    let snap =
        SourceSnapshot::from_bytes_for(Path::new("hud-probe.at"), user.as_bytes().to_vec(), api)
            .unwrap();
    let managed = snap.source.len() - user.len();
    assert!(managed <= MAX_BRIDGE, "managed={managed}");
    println!("{api:?} managed={managed} user={}", user.len());
    let mut s = Session::new(&snap.source).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(3)], ResultKind::Unit)
        .unwrap();
    for n in 0..3 {
        s.call(
            "_ag_identity",
            &[
                Int(n + 1),
                Int(if three(api) { 4 } else { 1 }),
                Int(if three(api) { 3 } else { 2 }),
                Int(-1),
            ],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [n as f32 * 10., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in [1., -0.4, -0.5, -0.4, 0.4, 0.5, 0.4]
            .into_iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int(n * 7 + i as i32), Float(v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..3 {
        for name in [
            if three(api) {
                "_ag_char3_map"
            } else {
                "_ag_char_map"
            },
            "_ag_obj_map",
            "_ag_trg_map",
        ] {
            s.call(name, &[Int(n + 1), Int(n)], ResultKind::Unit)
                .unwrap();
        }
    }
    if three(api) {
        s.call(
            "_ag_vis_map",
            &[Int(1), Int(0), Int(0), Int(4)],
            ResultKind::Unit,
        )
        .unwrap();
    } else {
        for (name, args) in [
            ("_ag_spr_map", vec![Int(1), Int(0), Int(0)]),
            ("_ag_spr_img", vec![Int(0), Int(8), Int(8)]),
            ("_ag_spr_def", vec![Int(0), Int(0), Int(0), Int(8)]),
            ("_ag_spr_defh", vec![Int(0), Int(8), Int(0)]),
        ] {
            s.call(name, &args, ResultKind::Unit).unwrap();
        }
    }
    s.call("_ag_audio_config", &[Int(2)], ResultKind::Unit)
        .unwrap();
    for slot in 0..4 {
        s.call(
            "_ag_hud_config",
            &[Int(slot), Int(1), Int(0)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        0,
        "app_start node={} arg={}",
        s.int("_ag_error_node", &[]).unwrap(),
        s.int("_ag_error_argument", &[]).unwrap()
    );
    s
}

fn tick(s: &mut Session, t: i32) {
    s.call("_ag_tick", &[Int(t), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
}

fn bundle(api: Api, ordinal: &str) -> String {
    let hud = "1,f\"Count: $step\",1,\"Playing\",0,\"\",1,\"Won\"";
    if three(api) {
        format!("ag_present3d(0,1,2,dt,1,0.5,2,{ordinal},0,0.25,{ordinal}+1,1,0.5,0,0,0.0,0,0,0.0,{hud})")
    } else {
        format!("ag_present2d(0,1,0,0,4,4,0,2,{ordinal},0,0.25,{ordinal}+1,1,0.5,0,0,0.0,0,0,0.0,{hud})")
    }
}

fn text(s: &mut Session, slot: i32) -> String {
    let len = s.int("_ag_hud_length", &[Int(slot)]).unwrap();
    (0..len)
        .map(|i| s.int("_ag_hud_code", &[Int(slot), Int(i)]).unwrap() as u8 as char)
        .collect()
}

#[test]
fn real_auto_bundle_emits_complete_dynamic_hud_visual_and_audio_state() {
    for api in [Api::Hud2V1, Api::Hud3V1] {
        let user = format!(
            "var step=0\nvar result=0\nfn app_start(){{}}\nfn app_update(dt float){{step=step+1;result={}}}\n",
            bundle(api, "step*2 - 1")
        );
        let mut s = session(api, &user);
        tick(&mut s, 1);
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(text(&mut s, 0), "Count: 1");
        assert_eq!(text(&mut s, 1), "Playing");
        assert_eq!(text(&mut s, 2), "");
        assert_eq!(text(&mut s, 3), "Won");
        assert_eq!(s.int("_ag_hud_visible", &[Int(2)]).unwrap(), 0);
        assert_eq!(s.int("_ag_audio_count", &[]).unwrap(), 2);
        tick(&mut s, 2);
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(text(&mut s, 0), "Count: 2");
    }
}

#[test]
fn hud_domain_and_standalone_bundle_conflicts_are_named_rejections() {
    for api in [Api::Hud2V1, Api::Hud3V1] {
        let b = bundle(api, "1");
        for (body, expected) in [
            ("r=ag_hud_set(4,1,\"bad\")".to_string(), 55),
            ("r=ag_hud_set(0,2,\"bad\")".to_string(), 55),
            (
                "r=ag_hud_set(0,1,\"123456789012345678901234567890123\")".to_string(),
                56,
            ),
            ("r=ag_hud_set(0,1,\"é\")".to_string(), 56),
            ("r=ag_hud_set(0,1,\"a\\nb\")".to_string(), 56),
            (format!("r=ag_hud_set(0,1,\"old\");r={b}"), 57),
            (format!("r={b};r=ag_hud_set(0,1,\"new\")"), 57),
        ] {
            let mut s = session(
                api,
                &format!(
                    "var step=1\nvar r=0\nfn app_start(){{}}\nfn app_update(dt float){{{body}}}\n"
                ),
            );
            tick(&mut s, 1);
            assert_eq!(
                s.int("_ag_status", &[]).unwrap(),
                expected,
                "{api:?}: {body}"
            );
        }
    }
}

#[test]
fn full_input_motion_event_visual_audio_hud_and_thirty_third_credit_control() {
    for api in [Api::Hud2V1, Api::Hud3V1] {
        for extra in [false, true] {
            let motion = if three(api) {
                "let left=held%2;let right=(held/2)%2;let forward=(held/4)%2;let back=(held/8)%2;let jump=(pressed/16)%2;let fx=ag_cam3_forward_x();let fz=ag_cam3_forward_z();let rx=ag_cam3_right_x();let rz=ag_cam3_right_z();let q=ag_char3_move(((right - left)*1.0*rx+(forward - back)*1.0*fx)*dt,jump*dt,((right - left)*1.0*rz+(forward - back)*1.0*fz)*dt,0.002);let grounded=ag_char3_grounded(q);"
            } else {
                "let left=held%2;let right=(held/2)%2;let jump=(pressed/4)%2;let q=ag_char_move((right - left)*dt,jump*dt,0.002);let grounded=ag_char_grounded(q);"
            };
            let user=format!("var r=0\nvar step=0\nvar observed=0\nfn app_start(){{let obj=ag_object_spawn(0,3,0.0,0.0,0.0);r=ag_trigger_bind(0,1,1,obj,0,0.0)}}\nfn event_observed() int{{return observed}}\nfn app_update(dt float){{step=step+1;let packet=ag_actions_snapshot();let held=packet%64;let pressed=packet/64;{motion}let e=ag_event_next();let pair=ag_event_pair();let phase=ag_event_phase();observed=e*100+pair*10+phase;r={};{}}}\n",bundle(api,"step*2 - 1"),if extra {"if step==2 {let excess=ag_actions_snapshot();}"} else {""});
            let mut s = session(api, &user);
            for t in 1..=2 {
                s.call(
                    if three(api) {
                        "_ag_char3_input"
                    } else {
                        "_ag_char_input"
                    },
                    &[
                        Int(2),
                        Int(if t == 2 {
                            if three(api) {
                                16
                            } else {
                                4
                            }
                        } else {
                            0
                        }),
                    ],
                    ResultKind::Unit,
                )
                .unwrap();
                tick(&mut s, t);
                let status = s.int("_ag_status", &[]).unwrap();
                println!(
                    "full {api:?} tick={t} extra={extra} status={status} event={} hud={}",
                    s.int("event_observed", &[]).unwrap(),
                    text(&mut s, 0)
                );
                assert_eq!(status, if extra && t == 2 { 10 } else { 0 });
                if extra && t == 2 {
                    assert!(s.int("event_observed", &[]).unwrap() >= 100);
                    break;
                }
                assert_eq!(s.int("_ag_audio_count", &[]).unwrap(), 2);
                assert_eq!(text(&mut s, 0), format!("Count: {t}"));
                if t == 2 {
                    assert!(s.int("event_observed", &[]).unwrap() >= 100);
                }
            }
        }
    }
}

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_owned()
}

fn host_scene(api: Api) -> autographics_auto_driver::authoring::host::Host {
    use autographics_auto_driver::authoring::{
        assets::{self, Registry},
        document::{Document, Kind, Object, SpriteBinding},
        host::Host,
        public_audio::{self, CueRef},
        public_hud::{DocumentHud, Slot},
    };
    let mut registry = Registry::new();
    let mut doc = Document::empty("public-hud".into());
    doc.schema = "autographics.scene/8".into();
    doc.cues = Some(vec![]);
    let mut actor = Object::new(1, if three(api) { Kind::Model } else { Kind::Quad });
    if three(api) {
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
        let asset = assets::decode_png_in(
            agp::assets::encode_png_rgba(8, 8, &pixels).unwrap(),
            Some("two-colors.png".into()),
            &registry,
        )
        .unwrap();
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
        if three(api) {
            Kind::Cylinder
        } else {
            Kind::Circle
        },
    );
    template.translation = [20.0, 0.0, 0.0];
    template.scale = [0.3, 0.5, if three(api) { 0.3 } else { 1.0 }];
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
    let font = autographics_auto_driver::authoring::public_hud::import(
        &root().join("samples/public_hud/fonts/lilita_one_regular.ttf"),
        &registry,
    )
    .unwrap();
    doc.hud = Some(DocumentHud {
        font: font.identity.key(),
        license: autographics_auto_driver::authoring::public_hud::LICENSE_TEXT.into(),
        slots: vec![
            Slot {
                slot: 0,
                visible: true,
                text: "0".into(),
                anchor: 0,
                offset: [8.0, 8.0],
                size: 32.0,
                rgba: [1.0, 1.0, 1.0, 1.0],
            },
            Slot {
                slot: 1,
                visible: true,
                text: "Playing".into(),
                anchor: 1,
                offset: [8.0, 8.0],
                size: 32.0,
                rgba: [0.0, 1.0, 0.0, 1.0],
            },
            Slot {
                slot: 2,
                visible: false,
                text: "".into(),
                anchor: 2,
                offset: [8.0, 8.0],
                size: 32.0,
                rgba: [1.0, 1.0, 0.0, 1.0],
            },
            Slot {
                slot: 3,
                visible: true,
                text: "Won".into(),
                anchor: 3,
                offset: [8.0, 8.0],
                size: 32.0,
                rgba: [1.0, 0.0, 1.0, 1.0],
            },
        ],
    });
    doc.assets.push(font.identity.clone());
    assets::insert(&mut registry, font).unwrap();
    Host::with_assets(doc, registry).unwrap()
}

fn host_controller(api: Api, bad: &str) -> String {
    let (start, motion, present) = if three(api) {
        ("ok=ag_char3_bind(1);ok=ag_camera_orbit_3d(1,8.0,45.0,0,0.0)","let jump=(pressed/16)%2;let right=(held/2)%2;let fx=ag_cam3_right_x();let fz=ag_cam3_right_z();let token=ag_char3_move(right*dt*fx,jump*dt,right*dt*fz,0.002);let grounded=ag_char3_grounded(token)","ag_present3d(0,1,2,step*0.1,1,0.5,2,step*2 - 1,choice,0.25,step*2,1 - choice,0.5,0,0,0.0,0,0,0.0")
    } else {
        ("ok=ag_char_bind(1);ok=ag_camera_follow_2d(1,-10.0,-10.0,10.0,10.0,4.0,4.0)","let jump=(pressed/4)%2;let right=(held/2)%2;let token=ag_char_move(right*dt,jump*dt,0.002);let grounded=ag_char_grounded(token)","ag_present2d(0,1,(step%2)*4,0,4,8,0,2,step*2 - 1,choice,0.25,step*2,1 - choice,0.5,0,0,0.0,0,0,0.0")
    };
    format!("var ok=0\nvar step=0\nvar observed=0\nfn app_start(){{{start};let obj=ag_object_spawn(0,3,0.0,0.0,0.0);ok=ag_trigger_bind(0,1,1,obj,0,0.0)}}\nfn app_update(dt float){{step=step+1;let packed=ag_actions_snapshot();let held=packed%64;let pressed=packed/64;{motion};let event=ag_event_next();let pair=ag_event_pair();let phase=ag_event_phase();observed=event*100+pair*10+phase;var choice=0;if observed>=100 {{choice=1}};var state=\"Playing\";var result=\"Won\";if step==2 {{state=\"Won\"}};if step>=3 {{state=\"Failed\";result=\"Failed\"}};ok={present},1,f\"$step\",1,state,0,\"\",1,result);{bad}}}\n")
}

fn host_start(h: &mut autographics_auto_driver::authoring::host::Host, api: Api, user: &str) {
    let source = SourceSnapshot::from_bytes_for(
        std::path::Path::new("public-hud-controller.at"),
        user.as_bytes().to_vec(),
        api,
    )
    .unwrap();
    h.start_user_snapshot(source).unwrap();
    h.pause_play("");
    host_pump(h, true);
}

fn host_pump(h: &mut autographics_auto_driver::authoring::host::Host, success: bool) {
    use autographics_auto_driver::authoring::play::State;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(std::time::Instant::now() < deadline, "pump deadline");
        h.frame(None).unwrap();
    }
    if success {
        assert_ne!(
            h.play.as_ref().unwrap().state,
            State::Faulted,
            "{:?}",
            h.play.as_ref().unwrap().error
        );
    }
}

fn host_tick(h: &mut autographics_auto_driver::authoring::host::Host, success: bool) {
    use autographics_auto_driver::authoring::play::Action;
    let p = h.play.as_mut().unwrap();
    p.resume().unwrap();
    p.char_key(1, true, false);
    p.dispatch(Action::default()).unwrap();
    host_pump(h, success);
    if success {
        h.pause_play("");
    }
}

#[test]
fn host_real_gpu_hud_pcm_warm_atlas_letterbox_and_last_good() {
    use autographics_auto_driver::authoring::public_hud::atlas_builds;
    for api in [Api::Hud2V1, Api::Hud3V1] {
        let mut h = host_scene(api);
        host_start(&mut h, api, &host_controller(api, ""));
        let surface = if three(api) { h.three } else { h.two };
        let initial = h.rt.capture(surface, "hud-initial").unwrap();
        let mut previous = initial.1;
        let builds = atlas_builds();
        assert_eq!(
            h.play
                .as_ref()
                .unwrap()
                .last
                .as_ref()
                .unwrap()
                .hud
                .as_ref()
                .unwrap()
                .slots[0]
                .text,
            "0"
        );
        for (step, state) in [(1, "Playing"), (2, "Won"), (3, "Failed")] {
            host_tick(&mut h, true);
            let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
            let hud = frame.hud.as_ref().unwrap();
            assert_eq!(hud.slots[0].text, step.to_string());
            assert_eq!(hud.slots[1].text, state);
            assert!(!hud.slots[2].visible);
            assert_eq!(hud.slots[3].text, if step >= 3 { "Failed" } else { "Won" });
            let capture = h.rt.capture(surface, "hud-tick").unwrap();
            let (mean, _) = agp::renderer::capture::compare_png_region(
                &previous,
                &capture.1,
                [0, 0, capture.2, 64],
            )
            .unwrap();
            assert!(
                mean > 0.0,
                "fresh HUD glyph ROI must change: {api:?} {step}"
            );
            if !three(api) {
                let (bars, _) = agp::renderer::capture::compare_png_region(
                    &previous,
                    &capture.1,
                    [0, 0, 200, capture.3],
                )
                .unwrap();
                assert_eq!(
                    bars, 0.0,
                    "letterbox pixels cannot receive HUD/world changes"
                );
            }
            previous = capture.1;
        }
        assert_eq!(
            atlas_builds(),
            builds,
            "warm text changes cannot rebuild an atlas"
        );
        let observation = h.audio.as_ref().unwrap().observation.lock().unwrap();
        assert!(
            observation.nonzero > 0,
            "accepted cues must reach the native service PCM"
        );
        drop(observation);
        h.stop_play();
        assert!(h.hud_atlases.len() <= 4);
    }
}

#[test]
fn invalid_auto_text_and_gpu_prepare_failure_preserve_complete_last_good_and_insert_no_new_voice() {
    for api in [Api::Hud2V1, Api::Hud3V1] {
        let mut h = host_scene(api);
        host_start(&mut h, api, &host_controller(api, ""));
        host_tick(&mut h, true);
        let surface = if three(api) { h.three } else { h.two };
        let before = h.rt.capture(surface, "last-good").unwrap().0;
        h.fail_resource = true;
        host_tick(&mut h, false);
        assert!(h.play.as_ref().unwrap().error.is_some());
        assert_eq!(h.rt.capture(surface, "failed").unwrap().0, before);
        assert_eq!(h.audio.as_ref().unwrap().mixer.lock().unwrap().voices(), 0);
    }
    for api in [Api::Hud2V1, Api::Hud3V1] {
        let mut h = host_scene(api);
        host_start(
            &mut h,
            api,
            &host_controller(api, "ok=ag_hud_set(0,1,\"é\")"),
        );
        host_tick(&mut h, false);
        assert!(h.play.as_ref().unwrap().error.is_some());
        assert_eq!(h.audio.as_ref().unwrap().mixer.lock().unwrap().voices(), 0);
    }
}

#[test]
fn scene8_font_closure_saves_relocates_rejects_corruption_and_recovers() {
    use autographics_auto_driver::authoring::{
        assets::Registry,
        document::Document,
        host::Host,
        public_hud::FONT_SHA,
        storage::{Fault, Saved, Storage},
    };
    for api in [Api::Hud2V1, Api::Hud3V1] {
        let mut h = host_scene(api);
        let directory = std::env::temp_dir().join(format!(
            "ui013-closure-{}-{}",
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
        assert_eq!(
            Document::decode(&std::fs::read(&entry).unwrap()).unwrap(),
            h.editor.document
        );
        let blob = entry
            .parent()
            .unwrap()
            .join("assets")
            .join(format!("{FONT_SHA}.ttf"));
        let original = std::fs::read(&blob).unwrap();
        std::fs::write(&blob, b"damaged font").unwrap();
        assert!(Storage::load_assets(&entry, &Registry::new()).is_err());
        std::fs::write(&blob, original).unwrap();
        std::fs::rename(directory.join("a"), directory.join("b")).unwrap();
        drop(h);
        let (doc, registry, _) =
            Storage::load_assets(&directory.join("b/scene.agscene"), &Registry::new()).unwrap();
        assert!(doc.hud.is_some());
        let mut h = Host::with_assets(doc, registry).unwrap();
        host_start(&mut h, api, &host_controller(api, ""));
        host_tick(&mut h, true);
        assert_eq!(
            h.play
                .as_ref()
                .unwrap()
                .last
                .as_ref()
                .unwrap()
                .hud
                .as_ref()
                .unwrap()
                .slots[0]
                .text,
            "1"
        );
        h.stop_play();
    }
}

#[test]
fn scene8_document_and_fixed_font_negative_controls_reject_before_effects() {
    let h = host_scene(Api::Hud2V1);
    let valid = h.editor.document.clone();
    for (case, mutate) in [
        (
            "missing-slot",
            (|d: &mut autographics_auto_driver::authoring::document::Document| {
                d.hud.as_mut().unwrap().slots.pop();
            }) as fn(&mut _),
        ),
        ("duplicate-slot", |d| {
            d.hud.as_mut().unwrap().slots[1].slot = 0
        }),
        ("long-text", |d| {
            d.hud.as_mut().unwrap().slots[0].text = "123456789012345678901234567890123".into()
        }),
        ("control-text", |d| {
            d.hud.as_mut().unwrap().slots[0].text = "a\nb".into()
        }),
        ("unicode-text", |d| {
            d.hud.as_mut().unwrap().slots[0].text = "é".into()
        }),
        ("anchor", |d| d.hud.as_mut().unwrap().slots[0].anchor = 4),
        ("size-nan", |d| {
            d.hud.as_mut().unwrap().slots[0].size = f32::NAN
        }),
        ("size-low", |d| d.hud.as_mut().unwrap().slots[0].size = 15.0),
        ("offset-infinite", |d| {
            d.hud.as_mut().unwrap().slots[0].offset[0] = f32::INFINITY
        }),
        ("rgba-nan", |d| {
            d.hud.as_mut().unwrap().slots[0].rgba[0] = f32::NAN
        }),
        ("rgba-range", |d| {
            d.hud.as_mut().unwrap().slots[0].rgba[0] = 1.1
        }),
        ("license", |d| d.hud.as_mut().unwrap().license.truncate(10)),
        ("font-ref", |d| {
            d.hud.as_mut().unwrap().font = "missing".into()
        }),
        ("font-sha", |d| {
            d.assets.iter_mut().find(|a| a.is_font()).unwrap().sha256 = "0".repeat(64)
        }),
        ("cues-required", |d| d.cues = None),
        ("hud-required", |d| d.hud = None),
        ("legacy-hud", |d| d.schema = "autographics.scene/7".into()),
    ] {
        let mut changed = valid.clone();
        mutate(&mut changed);
        assert!(changed.validate().is_err(), "accepted invalid case {case}");
    }
    let mut legacy = serde_json::to_value(&valid).unwrap();
    legacy["schema"] = serde_json::Value::String("autographics.scene/7".into());
    assert!(
        autographics_auto_driver::authoring::document::Document::decode(
            &serde_json::to_vec(&legacy).unwrap()
        )
        .is_err()
    );
    let mut unknown = serde_json::to_value(&valid).unwrap();
    unknown["hud"]["unknown"] = serde_json::Value::Bool(true);
    assert!(
        autographics_auto_driver::authoring::document::Document::decode(
            &serde_json::to_vec(&unknown).unwrap()
        )
        .is_err()
    );
    let builds = autographics_auto_driver::authoring::public_hud::atlas_builds();
    assert!(autographics_auto_driver::authoring::public_hud::import_raw(
        b"not the audited font".as_slice().into(),
        &Default::default(),
        None,
    )
    .is_err());
    assert_eq!(
        autographics_auto_driver::authoring::public_hud::atlas_builds(),
        builds
    );
    let duplicate = autographics_auto_driver::authoring::public_hud::import(
        &root().join("samples/public_hud/fonts/lilita_one_regular.ttf"),
        &h.editor.assets,
    )
    .unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &duplicate,
        h.editor
            .assets
            .values()
            .find(|a| a.identity.is_font())
            .unwrap()
    ));
}

#[test]
fn atlas_sharing_capacity_and_strict_glyph_controls_use_real_gpu() {
    let h = host_scene(Api::Hud2V1);
    assert_eq!(h.hud_atlases.len(), 1, "all equal sizes share one atlas");
    let mut document = h.editor.document.clone();
    for (slot, size) in document
        .hud
        .as_mut()
        .unwrap()
        .slots
        .iter_mut()
        .zip([16.0, 24.0, 32.0, 48.0])
    {
        slot.size = size;
        slot.visible = true;
        slot.text = format!("S{size}");
    }
    let (overlay, four) = autographics_auto_driver::authoring::public_hud::prepare_overlay(
        h.rt.gpu(),
        &h.hud_atlases,
        &h.editor.assets,
        &document,
        None,
        None,
        false,
    )
    .unwrap();
    assert_eq!(four.len(), 4);
    assert_eq!(overlay.unwrap().layers.len(), 4);
    document.hud.as_mut().unwrap().slots[0].size = 20.0;
    assert!(
        autographics_auto_driver::authoring::public_hud::prepare_overlay(
            h.rt.gpu(),
            &four,
            &h.editor.assets,
            &document,
            None,
            None,
            false,
        )
        .is_err()
    );
    let bytes = include_bytes!("../samples/public_hud/fonts/lilita_one_regular.ttf");
    assert!(agp::render2d::TextRenderer::new_checked(
        h.rt.gpu(),
        agp::renderer::target::OffscreenTarget::FORMAT,
        bytes,
        32.0,
        "😀",
    )
    .is_err());
    let ascii: String = (32u8..=126).map(char::from).collect();
    agp::render2d::TextRenderer::new_checked(
        h.rt.gpu(),
        agp::renderer::target::OffscreenTarget::FORMAT,
        bytes,
        32.0,
        &ascii,
    )
    .unwrap();
}

#[test]
fn seven_identity_resource_union_and_public_import_command_close_scene8() {
    use autographics_auto_driver::authoring::{
        assets::{self, Registry},
        document::Document,
        host::Host,
        public_hud,
        storage::Storage,
    };
    let entry = root().join("samples/public_audio/project3d/scene.agscene");
    let (mut document, mut registry, _, _) =
        Storage::load_project(&entry, &Registry::new()).unwrap();
    let font = public_hud::import(
        &root().join("samples/public_hud/fonts/lilita_one_regular.ttf"),
        &registry,
    )
    .unwrap();
    document.schema = "autographics.scene/8".into();
    document.hud = Some(public_hud::default_document(font.identity.key()));
    document.assets.push(font.identity.clone());
    assets::insert(&mut registry, font).unwrap();
    document.validate().unwrap();
    let (raw, bytes, cpu, gpu) =
        autographics_auto_driver::authoring::textured_models::usage(&registry);
    assert_eq!((raw, bytes), (7, 263372));
    assert!(cpu <= assets::MAX_GEOMETRY && gpu <= assets::MAX_GEOMETRY);

    let mut h = Host::new(Document::sample()).unwrap();
    let message = serde_json::json!({
        "epoch": h.editor.stamp.epoch,
        "revision": h.editor.stamp.revision,
        "action": "import-font",
        "payload": root().join("samples/public_hud/fonts/lilita_one_regular.ttf").to_string_lossy(),
        "asset_generation": 0,
    });
    h.dispatch(&message.to_string()).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.document.schema, "autographics.scene/8");
    assert_eq!(
        h.editor.document.hud.as_ref().unwrap().license,
        public_hud::LICENSE_TEXT
    );
    assert_eq!(h.editor.document.cues.as_ref().unwrap().len(), 0);
}

#[test]
fn complete_hud_wire_mutations_reject_and_legitimate_control_restores() {
    let mut wire = vec![];
    for slot in 0..4 {
        wire.push(Int(i32::from(slot != 2)));
        wire.push(Int(3));
        for index in 0..32 {
            wire.push(Int(if index < 3 { 65 + slot } else { 0 }));
        }
    }
    let accepted = autographics_auto_driver::authoring::public_hud::admit_block(&wire).unwrap();
    assert_eq!(accepted.slots[0].text, "AAA");
    assert!(!accepted.slots[2].visible);
    for slot in 0..4 {
        let base = slot * 34;
        for (index, value) in [
            (base, Int(-1)),
            (base, Int(2)),
            (base, Float(1.0)),
            (base + 1, Int(-1)),
            (base + 1, Int(33)),
            (base + 1, Float(3.0)),
            (base + 2, Int(31)),
            (base + 2, Int(127)),
            (base + 2, Float(65.0)),
            (base + 5, Int(1)),
            (base + 5, Float(0.0)),
        ] {
            let original = wire[index].clone();
            wire[index] = value;
            assert!(
                autographics_auto_driver::authoring::public_hud::admit_block(&wire).is_err(),
                "slot {slot} index {index} accepted mutation"
            );
            wire[index] = original;
            autographics_auto_driver::authoring::public_hud::admit_block(&wire).unwrap();
        }
    }
    for changed in [wire[..wire.len() - 1].to_vec(), {
        let mut changed = wire.clone();
        changed.push(Int(0));
        changed
    }] {
        assert!(autographics_auto_driver::authoring::public_hud::admit_block(&changed).is_err());
    }
}

#[test]
fn repeated_render_and_capture_do_not_replay_audio_or_rebuild_atlas() {
    for api in [Api::Hud2V1, Api::Hud3V1] {
        let mut h = host_scene(api);
        host_start(&mut h, api, &host_controller(api, ""));
        host_tick(&mut h, true);
        let surface = if three(api) { h.three } else { h.two };
        let voices = h.audio.as_ref().unwrap().mixer.lock().unwrap().voices();
        assert_eq!(voices, 2);
        let builds = autographics_auto_driver::authoring::public_hud::atlas_builds();
        let image = h.rt.capture(surface, "first").unwrap().0;
        for _ in 0..3 {
            h.frame(None).unwrap();
            assert_eq!(h.rt.capture(surface, "replay").unwrap().0, image);
            assert_eq!(
                h.audio.as_ref().unwrap().mixer.lock().unwrap().voices(),
                voices
            );
        }
        assert_eq!(
            autographics_auto_driver::authoring::public_hud::atlas_builds(),
            builds
        );
        h.stop_play();
    }
}
