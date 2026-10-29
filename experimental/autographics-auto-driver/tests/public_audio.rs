//! PLAN-012 T-02: real Auto producer feasibility. Host/PCM closure is separate.
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot, MAX_BRIDGE},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

fn session(api: Api, user: &str) -> Session {
    let user=user.replace("fn app_start(){",if api==Api::Audio3V1 {
        "fn app_start(){let bound=ag_char3_bind(1);let camera=ag_camera_orbit_3d(1,8.0,45.0,0,0.0);"
    } else {"fn app_start(){let bound=ag_char_bind(1);let camera=ag_camera_follow_2d(1,-10.0,-10.0,10.0,10.0,4.0,4.0);"});
    let snap =
        SourceSnapshot::from_bytes_for(Path::new("audio-probe.at"), user.as_bytes().to_vec(), api)
            .unwrap();
    let managed = snap.source.len() - user.len();
    assert!(managed <= MAX_BRIDGE);
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
                Int(if api == Api::Audio3V1 { 4 } else { 1 }),
                Int(if api == Api::Audio3V1 { 3 } else { 2 }),
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
            if api == Api::Audio3V1 {
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
    if api == Api::Audio3V1 {
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
    if api == Api::Audio3V1 {
        format!("ag_present3d(0,1,2,dt,1,0.5,2,{ordinal},0,0.25,{ordinal}+1,1,0.5,0,0,0.0,0,0,0.0)")
    } else {
        format!("ag_present2d(0,1,0,0,4,4,0,2,{ordinal},0,0.25,{ordinal}+1,1,0.5,0,0,0.0,0,0,0.0)")
    }
}
#[test]
fn two_dimensional_profiles_produce_finite_typed_cues_and_visual_in_one_credit() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let user = format!(
            "var result=0\nfn app_start(){{}}\nfn app_update(dt float){{result={}}}\n",
            bundle(api, "1")
        );
        let mut s = session(api, &user);
        tick(&mut s, 1);
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(s.int("_ag_audio_count", &[]).unwrap(), 2);
        assert_eq!(s.int("_ag_audio_i", &[Int(1), Int(0)]).unwrap(), 2);
        assert_eq!(s.float("_ag_audio_g", &[Int(0)]).unwrap(), 0.25);
        tick(&mut s, 2);
        assert_eq!(
            s.int("_ag_status", &[]).unwrap(),
            52,
            "ordinal reuse across ticks"
        );
    }
}
#[test]
fn bundle_conflicts_invalid_gains_cues_and_duplicate_ordinals_reject_candidates() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let b = bundle(api, "1");
        for (body, expected) in [
            (format!("r={b};r={b}"), 50),
            (format!("r=ag_audio_emit(1,0,1.0);r={b}"), 50),
            (format!("r={b};r=ag_audio_emit(3,0,1.0)"), 50),
            ("r=ag_audio_emit(1,2,1.0)".into(), 53),
            ("r=ag_audio_emit(1,0,1.01)".into(), 54),
            (
                "r=ag_audio_emit(1,0,0.5);r=ag_audio_emit(1,0,0.5)".into(),
                52,
            ),
        ] {
            let mut s = session(
                api,
                &format!("var r=0\nfn app_start(){{}}\nfn app_update(dt float){{{body}}}\n"),
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

fn packet(
    s: &mut Session,
    tick: i32,
) -> autographics_auto_driver::authoring::public_audio::mixer::Packet {
    use autographics_auto_driver::authoring::public_audio::mixer::Packet;
    let count = s.int("_ag_audio_count", &[]).unwrap();
    let mut data = vec![
        Int(s.int("_ag_audio_run", &[]).unwrap()),
        Int(s.int("_ag_audio_tick", &[]).unwrap()),
        Int(s.int("_ag_audio_bundle_id", &[]).unwrap()),
        Int(count),
    ];
    for i in 0..count {
        data.push(Int(s.int("_ag_audio_i", &[Int(i), Int(0)]).unwrap()));
        data.push(Int(s.int("_ag_audio_i", &[Int(i), Int(1)]).unwrap()));
        data.push(Float(s.float("_ag_audio_g", &[Int(i)]).unwrap()));
    }
    Packet::from_wire(&data, 1, tick).unwrap()
}
fn cues() -> Vec<autographics_auto_driver::authoring::public_audio::mixer::Cue> {
    use autographics_auto_driver::authoring::public_audio::mixer::{Cue, Pcm};
    use std::sync::Arc;
    // Two deliberately distinguishable stereo channels; shared immutable bank.
    let mut frames = vec![[0.4, 0.0]; 4410];
    frames.extend(vec![[0.0, -0.8]; 4410]);
    let pcm = Arc::new(Pcm {
        rate: 44100,
        frames: frames.into(),
    });
    vec![
        Cue {
            pcm: pcm.clone(),
            start: 0,
            end: 4410,
            gain: 1.0,
        },
        Cue {
            pcm,
            start: 4410,
            end: 8820,
            gain: 1.0,
        },
    ]
}
#[test]
fn real_auto_packet_common_accept_and_pcm_oracle_detect_replay_gain_and_failure() {
    use autographics_auto_driver::authoring::public_audio::mixer::{Accepted, Mixer};
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let mut s = session(
            api,
            &format!(
                "var r=0\nfn app_start(){{}}\nfn app_update(dt float){{r={}}}\n",
                bundle(api, "1")
            ),
        );
        tick(&mut s, 1);
        let p = packet(&mut s, 1);
        let cues = cues();
        let mut mixer = Mixer::default();
        mixer.begin(1).unwrap();
        let mut output = [[0.0; 2]; 32];
        assert!(mixer
            .accept(p.clone(), &cues, false, || Err(
                "visual prepare failure".into()
            ))
            .is_err());
        mixer.render(&mut output, 44100);
        assert_eq!(output, [[0.0; 2]; 32]);
        assert_eq!(mixer.voices(), 0);
        assert_eq!(
            mixer.accept(p.clone(), &cues, false, || Ok(())).unwrap(),
            Accepted::Played
        );
        mixer.render(&mut output, 44100);
        assert!(output
            .iter()
            .all(|v| (v[0] - 0.1).abs() < 1e-7 && (v[1] + 0.4).abs() < 1e-7));
        assert_eq!(
            mixer
                .accept(p, &cues, false, || panic!("replay cannot reaccept"))
                .unwrap(),
            Accepted::Replay
        );
        assert_eq!(mixer.voices(), 2);
        mixer.pause(1, true).unwrap();
        mixer.render(&mut output, 44100);
        assert_eq!(output, [[0.0; 2]; 32]);
        mixer.pause(1, false).unwrap();
        mixer.render(&mut output, 44100);
        assert!(output.iter().all(|v| v[0] > 0.09 && v[1] < -0.39));
        mixer.stop();
        mixer.render(&mut output, 44100);
        assert_eq!(output, [[0.0; 2]; 32]);
        assert_eq!(mixer.voices(), 0);
        assert!(
            mixer.begin(1).is_err(),
            "a stopped old Run cannot reactivate the retained manager"
        );
        mixer.begin(2).unwrap();
        assert!(
            mixer.pause(1, false).is_err(),
            "old control loses authority after new Run"
        );
        assert!(mixer
            .accept(packet(&mut s, 1), &cues, false, || panic!(
                "old Run cannot accept"
            ))
            .is_err());
    }
}
#[test]
fn real_auto_cues_reach_this_service_native_output_stream() {
    use autographics_auto_driver::authoring::public_audio::native::Native;
    let mut s = session(
        Api::Audio3V1,
        &format!(
            "var r=0\nfn app_start(){{}}\nfn app_update(dt float){{r={}}}\n",
            bundle(Api::Audio3V1, "1")
        ),
    );
    tick(&mut s, 1);
    let packet = packet(&mut s, 1);
    let native = Native::open().expect("native audio unavailable is a failure, never mock success");
    {
        let mut m = native.mixer.lock().unwrap();
        m.begin(1).unwrap();
        m.accept(packet, &cues(), false, || Ok(())).unwrap();
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if native.observation.lock().unwrap().nonzero >= 100 {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "native expected PCM timeout"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let observation = native.observation.lock().unwrap();
    assert!(observation
        .ring
        .iter()
        .any(|v| (v[0] - 0.1).abs() < 1e-7 && (v[1] + 0.4).abs() < 1e-7));
    println!("native_device={} rate={} callbacks={} nonzero={} peak={} max_us={} deadline_misses={} quantum_frames={}",native.device,native.rate,observation.callbacks,observation.nonzero,observation.peak,observation.max_us,observation.deadline_misses,observation.last_quantum);
    drop(observation);
    native.mixer.lock().unwrap().stop();
}

#[test]
fn full_input_motion_event_visual_audio_and_thirty_third_credit_control() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        for extra in [false, true] {
            let motion = if api == Api::Audio3V1 {
                "let left=held%2;let right=(held/2)%2;let forward=(held/4)%2;let back=(held/8)%2;let jump=(pressed/16)%2;let fx=ag_cam3_forward_x();let fz=ag_cam3_forward_z();let rx=ag_cam3_right_x();let rz=ag_cam3_right_z();let q=ag_char3_move(((right - left)*1.0*rx+(forward - back)*1.0*fx)*dt,jump*dt,((right - left)*1.0*rz+(forward - back)*1.0*fz)*dt,0.002);let grounded=ag_char3_grounded(q);"
            } else {
                "let left=held%2;let right=(held/2)%2;let jump=(pressed/4)%2;let q=ag_char_move((right - left)*dt,jump*dt,0.002);let grounded=ag_char_grounded(q);"
            };
            let user=format!("var r=0\nvar step=0\nvar observed=0\nfn app_start(){{let obj=ag_object_spawn(0,3,0.0,0.0,0.0);r=ag_trigger_bind(0,1,1,obj,0,0.0)}}\nfn event_observed() int{{return observed}}\nfn app_update(dt float){{step=step+1;let packet=ag_actions_snapshot();let held=packet%64;let pressed=packet/64;{motion}let e=ag_event_next();let pair=ag_event_pair();let phase=ag_event_phase();observed=e*100+pair*10+phase;r={};{}}}\n",bundle(api,"step*2 - 1"),if extra {"if step==2 {let excess=ag_actions_snapshot();}"} else {""});
            let mut s = session(api, &user);
            for t in 1..=2 {
                s.call(
                    if api == Api::Audio3V1 {
                        "_ag_char3_input"
                    } else {
                        "_ag_char_input"
                    },
                    &[
                        Int(2),
                        Int(if t == 2 {
                            if api == Api::Audio3V1 {
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
                    "full {api:?} tick={t} extra={extra} status={status} event={}",
                    s.int("event_observed", &[]).unwrap()
                );
                assert_eq!(status, if extra && t == 2 { 10 } else { 0 });
                if extra && t == 2 {
                    assert!(s.int("event_observed", &[]).unwrap() >= 100);
                    break;
                }
                assert_eq!(s.int("_ag_audio_count", &[]).unwrap(), 2);
                if t == 2 {
                    assert!(
                        s.int("event_observed", &[]).unwrap() >= 100,
                        "event branch must actually exist"
                    );
                }
            }
        }
    }
}

#[test]
fn packed_2d_actions_preserve_original_bit_order_and_consumption() {
    let user="var p=0\nvar again=0\nvar legacy=0\nfn app_start(){}\nfn result() int{return p}\nfn repeated() int{return again}\nfn old_pressed() int{return legacy}\nfn app_update(dt float){p=ag_actions_snapshot();again=ag_actions_snapshot();legacy=ag_action_pressed(2)}\n";
    let mut s = session(Api::Audio2V1, user);
    for (i, bits) in [0, 1, 2, 4, 8, 15].into_iter().enumerate() {
        s.call("_ag_char_input", &[Int(bits), Int(bits)], ResultKind::Unit)
            .unwrap();
        tick(&mut s, i as i32 + 1);
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(s.int("result", &[]).unwrap(), bits * 65);
        assert_eq!(s.int("repeated", &[]).unwrap(), bits);
        assert_eq!(s.int("old_pressed", &[]).unwrap(), 0);
    }
}
