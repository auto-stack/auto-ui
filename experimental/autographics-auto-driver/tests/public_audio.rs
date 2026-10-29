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

#[test]
fn exact_integer_ordinals_four_request_bound_and_optional_bundle_conflicts() {
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let user="var r=0\nvar step=0\nfn app_start(){}\nfn app_update(dt float){step=step+1;var o=16777217;if step==2 {o=2147483646};if step==3 {o=2147483647};r=ag_audio_emit(o,0,0.5)}\n";
        let mut s = session(api, user);
        for (t, ordinal) in [(1, 16777217), (2, 2147483646)] {
            tick(&mut s, t);
            assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
            assert_eq!(s.int("_ag_audio_i", &[Int(0), Int(0)]).unwrap(), ordinal);
        }
        tick(&mut s, 3);
        assert_eq!(
            s.int("_ag_status", &[]).unwrap(),
            52,
            "ordinal exhaustion never wraps"
        );
        for n in [4, 5] {
            let mut s=session(api,&format!("var r=0\nfn app_start(){{}}\nfn app_update(dt float){{for i in 0..{n} {{r=ag_audio_emit(i+1,i%2,0.5)}}}}\n"));
            tick(&mut s, 1);
            assert_eq!(
                s.int("_ag_status", &[]).unwrap(),
                if n == 4 { 0 } else { 51 }
            );
            if n == 4 {
                assert_eq!(s.int("_ag_audio_count", &[]).unwrap(), 4);
            }
        }
        let visual = if api == Api::Audio3V1 {
            "ag_visual_pose(0,1,2,0.1,1,0.0)"
        } else {
            "ag_sprite_frame(1,0,0,4,4,0)"
        };
        let b = bundle(api, "1");
        for body in [
            format!("r={visual};r={visual}"),
            format!("r={visual};r={b}"),
            format!("r={b};r={visual}"),
        ] {
            let mut s = session(
                api,
                &format!("var r=0\nfn app_start(){{}}\nfn app_update(dt float){{{body}}}\n"),
            );
            tick(&mut s, 1);
            assert_eq!(s.int("_ag_status", &[]).unwrap(), 50, "{body}");
        }
        let mut control=session(api,&format!("var r=0\nfn app_start(){{}}\nfn app_update(dt float){{r={visual};r=ag_audio_emit(1,0,0.5)}}\n"));
        tick(&mut control, 1);
        assert_eq!(
            control.int("_ag_status", &[]).unwrap(),
            0,
            "standalone visual plus charged standalone audio is legitimate"
        );
        for n in [0, 4, 5] {
            let present = if api == Api::Audio3V1 {
                format!("ag_present3d(-1,0,-1,0.0,0,0.0,{n},1,0,0.1,2,1,0.2,3,0,0.3,4,1,0.4)")
            } else {
                format!("ag_present2d(-1,0,0,0,1,1,0,{n},1,0,0.1,2,1,0.2,3,0,0.3,4,1,0.4)")
            };
            let mut s = session(
                api,
                &format!("var r=0\nfn app_start(){{}}\nfn app_update(dt float){{r={present}}}\n"),
            );
            tick(&mut s, 1);
            assert_eq!(
                s.int("_ag_status", &[]).unwrap(),
                if n <= 4 { 0 } else { 51 }
            );
            if n <= 4 {
                assert_eq!(s.int("_ag_audio_count", &[]).unwrap(), n);
            }
        }
    }
}

fn decoded_wave_cues() -> (
    Vec<autographics_auto_driver::authoring::public_audio::mixer::Cue>,
    std::sync::Arc<autographics_auto_driver::authoring::assets::Asset>,
) {
    use autographics_auto_driver::authoring::{
        assets,
        document::Document,
        public_audio::{self, CueRef},
    };
    let mut raw = b"RIFF".to_vec();
    raw.extend_from_slice(&116u32.to_le_bytes());
    raw.extend_from_slice(b"WAVEfmt ");
    raw.extend_from_slice(&16u32.to_le_bytes());
    raw.extend_from_slice(&1u16.to_le_bytes());
    raw.extend_from_slice(&2u16.to_le_bytes());
    raw.extend_from_slice(&44100u32.to_le_bytes());
    raw.extend_from_slice(&176400u32.to_le_bytes());
    raw.extend_from_slice(&4u16.to_le_bytes());
    raw.extend_from_slice(&16u16.to_le_bytes());
    raw.extend_from_slice(b"data");
    raw.extend_from_slice(&80u32.to_le_bytes());
    for i in 1..=20i16 {
        raw.extend_from_slice(&(i * 1000).to_le_bytes());
        raw.extend_from_slice(&(-i * 500).to_le_bytes());
    }
    let mut registry = assets::Registry::new();
    let asset = public_audio::import_raw(raw.into(), &registry, None).unwrap();
    assets::insert(&mut registry, asset.clone()).unwrap();
    let mut doc = Document::empty("wave-oracle".into());
    doc.schema = "autographics.scene/7".into();
    doc.assets.push(asset.identity.clone());
    doc.cues = Some(vec![
        CueRef {
            name: "short".into(),
            asset: asset.identity.key(),
            start_frame: 4,
            end_frame: 8,
            gain: 0.5,
        },
        CueRef {
            name: "long".into(),
            asset: asset.identity.key(),
            start_frame: 12,
            end_frame: 18,
            gain: 1.0,
        },
    ]);
    (public_audio::resolve_cues(&doc, &registry).unwrap(), asset)
}
fn waveform(rate: u32) -> Vec<[f32; 2]> {
    let n = (6.0 * rate as f64 / 44100.0).ceil() as usize + 2;
    (0..n)
        .map(|i| {
            let t = i as f64 * 44100.0 / rate as f64;
            let first = if t < 4.0 {
                (5.0 + t.min(3.0)) * 1000.0 / 32768.0 * 0.125
            } else {
                0.0
            };
            let second = if t < 6.0 {
                (13.0 + t.min(5.0)) * 1000.0 / 32768.0 * 0.5
            } else {
                0.0
            };
            [(first + second) as f32, (-(first + second) * 0.5) as f32]
        })
        .collect()
}
#[test]
fn decoded_cue_ranges_gains_tails_and_transport_replays_match_independent_waveform() {
    use autographics_auto_driver::authoring::public_audio::mixer::{Accepted, Mixer};
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let (cues, _asset) = decoded_wave_cues();
        for rate in [22050, 44100, 48000] {
            let mut s = session(
                api,
                &format!(
                    "var r=0\nfn app_start(){{}}\nfn app_update(dt float){{r={}}}\n",
                    bundle(api, "1")
                ),
            );
            tick(&mut s, 1);
            let packet = packet(&mut s, 1);
            let mut mixer = Mixer::default();
            mixer.begin(1).unwrap();
            mixer
                .accept(packet.clone(), &cues, false, || Ok(()))
                .unwrap();
            let expected = waveform(rate);
            let mut actual = vec![[0.0; 2]; expected.len()];
            mixer.render(&mut actual, rate);
            for (i, (a, e)) in actual.iter().zip(&expected).enumerate() {
                for ch in 0..2 {
                    assert!(
                        (a[ch] - e[ch]).abs() < 0.000001,
                        "{api:?} rate{rate} frame{i} ch{ch}: {a:?} != {e:?}"
                    );
                }
            }
            assert_eq!(mixer.voices(), 0, "voice ends at accepted half-open span");
            assert_eq!(
                mixer
                    .accept(packet.clone(), &cues, false, || panic!(
                        "same logical packet cannot accept twice"
                    ))
                    .unwrap(),
                Accepted::Replay
            );
            mixer.render(&mut actual, rate);
            assert!(
                actual.iter().all(|f| *f == [0.0; 2]),
                "finished voice cannot be replayed by transport"
            );
            let mut replaced = packet;
            replaced.requests[0].gain = 0.5;
            assert!(mixer
                .accept(replaced, &cues, false, || panic!(
                    "changed packet at accepted tick"
                ))
                .is_err());
        }
    }
}
#[test]
fn native_stream_contains_actual_decoded_span_waveform_once_after_common_accept() {
    use autographics_auto_driver::authoring::public_audio::native::{Native, TRACE_FRAMES};
    let (cues, _asset) = decoded_wave_cues();
    let mut s = session(
        Api::Audio3V1,
        &format!(
            "var r=0\nfn app_start(){{}}\nfn app_update(dt float){{r={}}}\n",
            bundle(Api::Audio3V1, "1")
        ),
    );
    tick(&mut s, 1);
    let packet = packet(&mut s, 1);
    let native = Native::open().unwrap();
    {
        let mut mixer = native.mixer.lock().unwrap();
        mixer.begin(1).unwrap();
        mixer
            .accept(packet.clone(), &cues, false, || Ok(()))
            .unwrap();
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if native.observation.lock().unwrap().nonzero > 0 {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "native span PCM missing"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let mut expected = waveform(native.rate);
    if native.channels == 1 {
        for p in &mut expected {
            let mono = (p[0] + p[1]) * 0.5;
            *p = [mono; 2];
        }
    }
    let tolerance = if native.sample_format == "F32" {
        0.000001
    } else {
        2.0 / 32768.0
    };
    {
        let observation = native.observation.lock().unwrap();
        assert!(
            (0..TRACE_FRAMES).any(|start| expected.iter().enumerate().all(|(i, e)| (0..2).all(
                |ch| (observation.ring[(start + i) % TRACE_FRAMES][ch] - e[ch]).abs() <= tolerance
            ))),
            "actual submitted PCM must contain complete expected positions/gains/tails"
        );
        println!("native span waveform device={} rate={} channels={} format={} submitted_frames={} max_us={} deadline_misses={}",native.device,native.rate,native.channels,native.sample_format,expected.len(),observation.max_us,observation.deadline_misses);
    }
    let before = native.observation.lock().unwrap().nonzero;
    {
        let mut m = native.mixer.lock().unwrap();
        assert_eq!(
            m.accept(packet, &cues, false, || panic!(
                "native replay cannot recommit"
            ))
            .unwrap(),
            autographics_auto_driver::authoring::public_audio::mixer::Accepted::Replay
        );
    }
    std::thread::sleep(std::time::Duration::from_millis(30));
    assert_eq!(
        native.observation.lock().unwrap().nonzero,
        before,
        "transport replay must not submit another waveform"
    );
}

#[test]
fn pause_freezes_sample_cursor_and_step_consumes_ordinals_without_backlog() {
    use autographics_auto_driver::authoring::public_audio::mixer::{Accepted, Mixer};
    let (cues, _asset) = decoded_wave_cues();
    let mut s = session(
        Api::Audio2V1,
        &format!(
            "var r=0\nfn app_start(){{}}\nfn app_update(dt float){{r={}}}\n",
            bundle(Api::Audio2V1, "1")
        ),
    );
    tick(&mut s, 1);
    let first = packet(&mut s, 1);
    let mut m = Mixer::default();
    m.begin(1).unwrap();
    m.accept(first.clone(), &cues, false, || Ok(())).unwrap();
    let mut head = [[0.0; 2]; 2];
    m.render(&mut head, 44100);
    assert_eq!(head.as_slice(), &waveform(44100)[..2]);
    m.pause(1, true).unwrap();
    let mut silence = [[1.0; 2]; 4096];
    m.render(&mut silence, 44100);
    assert!(silence.iter().all(|f| *f == [0.0; 2]));
    let mut step = first.clone();
    step.tick = 2;
    for r in &mut step.requests {
        r.ordinal += 2;
    }
    assert_eq!(
        m.accept(step.clone(), &cues, true, || Ok(())).unwrap(),
        Accepted::SuppressedByStep
    );
    m.pause(1, false).unwrap();
    let mut tail = [[0.0; 2]; 6];
    m.render(&mut tail, 44100);
    assert_eq!(tail.as_slice(), &waveform(44100)[2..]);
    assert_eq!(m.voices(), 0);
    step.tick = 3;
    assert!(m
        .accept(step, &cues, false, || panic!(
            "step ordinal cannot be reused"
        ))
        .is_err());
    m.render(&mut tail, 44100);
    assert!(tail.iter().all(|f| *f == [0.0; 2]));
}
