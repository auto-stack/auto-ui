//! T-02 producer probe. Host/GPU transaction coverage is added in T-02..05;
//! this file currently proves real Auto source assembly, output and budget.
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    document::Document,
    host::Host,
    play::{Action, State},
};
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot, MAX_BRIDGE},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use serde_json::json;
use std::path::Path;
use std::time::{Duration, Instant};

fn pump(h: &mut Host) {
    let end = Instant::now() + Duration::from_secs(60);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < end, "host deadline");
        h.frame(None).unwrap();
    }
    assert!(
        h.play.as_ref().unwrap().error.is_none(),
        "{:?}",
        h.play.as_ref().unwrap().error
    );
}
fn step_host(h: &mut Host) {
    pump(h);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .unwrap();
    pump(h);
}

#[test]
fn original_textured_character_auto_host_same_frame_pose_and_gpu_pixels() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/textured_models/models/character.glb");
    assert!(assets::import(&path, &Registry::new())
        .unwrap_err()
        .contains("animations"));
    let mut h = Host::new(Document::empty("node-probe".into())).unwrap();
    h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":"import-animated","payload":path.to_str().unwrap(),"selection":h.editor.selection,"asset_generation":h.ui().asset_token}).to_string()).unwrap();
    h.settle().unwrap();
    assert!(h.ui().error.is_empty(), "{}", h.ui().error);
    let asset = h.editor.assets.values().next().unwrap().clone();
    assert_eq!(asset.animation.as_ref().unwrap().clips.len(), 4);
    assert!(asset.identity.closure.as_ref().unwrap().node_animation);
    // The pre-existing character contract requires a frozen .8 x 1 x .8 AABB.
    // Fixture scaling configures that collision shape without editing the GLB.
    let (lo, hi) = autographics_auto_driver::authoring::interaction::bounds(&asset.model).unwrap();
    let extent = hi - lo;
    let mut doc = h.editor.document.clone();
    doc.objects[0].scale = [0.8 / extent.x, 1.0 / extent.y, 0.8 / extent.z];
    let registry = h.editor.assets.clone();
    drop(h);
    let mut h = Host::with_assets(doc, registry).unwrap();
    // The explicit profile is part of the persisted content identity. Cold
    // relocation must rebuild the animation from the same raw GLB and PNG.
    let root = std::env::temp_dir().join(format!(
        "ui006-storage-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("a")).unwrap();
    let entry = root.join("a/scene.agscene");
    assert_eq!(
        h.storage
            .save(
                &mut h.editor,
                &entry,
                autographics_auto_driver::authoring::storage::Fault::None
            )
            .unwrap(),
        autographics_auto_driver::authoring::storage::Saved::Confirmed
    );
    std::fs::rename(root.join("a"), root.join("b")).unwrap();
    let (doc, registry, _) = autographics_auto_driver::authoring::storage::Storage::load_assets(
        &root.join("b/scene.agscene"),
        &Registry::new(),
    )
    .unwrap();
    let reloaded = &registry[&asset.identity.key()];
    assert_eq!(reloaded.identity, asset.identity);
    assert_eq!(reloaded.animation.as_ref().unwrap().clips.len(), 4);
    drop(h);
    let mut h = Host::with_assets(doc, registry).unwrap();

    let user="var ok=0\nvar phase=0.0\nvar obj=0\nvar mode=0\nfn app_start(){ok=ag_char3_bind(1);ok=ag_camera_orbit_3d(1,4.0,30.0,0,0.0);obj=ag_object_spawn(0,1,1.0,0.0,0.0);ok=ag_trigger_bind(0,1,1,obj,0,0.0);ok=ag_visual_pose(0,1,2,0.0,1,0.0)}\nfn app_update(dt float){phase=phase+dt*6.0;let fx=ag_cam3_forward_x();let fz=ag_cam3_forward_z();let q=ag_char3_move(fx*dt,0.01,fz*dt,0.002);let g=ag_char3_grounded(q);let e=ag_event_next();if e==1 {let p=ag_event_pair();ok=p};var clip=2;if g==0 {clip=3};ok=ag_visual_pose(0,1,clip,phase,1,0.5);ok=ag_visual_pose(1,obj,1,phase,1,0.0)}\n";
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("original-character.at"),
        user.as_bytes().to_vec(),
        Api::NodeAnimation3V1,
    )
    .unwrap();
    let legacy = SourceSnapshot::from_bytes_for(
        Path::new("legacy.at"),
        b"fn app_start(){}\nfn app_update(dt float){}\n".to_vec(),
        Api::Triggers3V1,
    )
    .unwrap();
    assert!(
        h.start_user_snapshot(legacy)
            .unwrap_err()
            .contains("animated payload requires"),
        "old profiles cannot silently drop admitted clips"
    );
    h.prepare_measurement.enable(true);
    h.start_user_snapshot(snapshot).unwrap();
    h.pause_play("");
    pump(&mut h);
    assert_eq!(h.play.as_ref().unwrap().state, State::Paused);
    h.frame(None).unwrap();
    let initial = h.rt.capture(h.three, "ui006-start").unwrap().0;
    let first = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
    assert_eq!(first.animations.as_ref().unwrap().targets[0].1.phase, 0.);
    let warm = h.resource_snapshot()["mesh_uploads"].clone();
    let decodes = assets::decode_attempts();
    // dispatch is a fixed-tick producer call. Sampling is observed on the same
    // accepted tick as the changed position and the echoed runtime camera.
    step_host(&mut h);
    let f = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
    let v = f.animations.as_ref().unwrap().visual_for(1).unwrap();
    assert!((v.phase - 0.1).abs() < 1e-6);
    assert_eq!(v.clip, 3);
    assert!((v.yaw - 0.5).abs() < 1e-6);
    assert_eq!(f.animations.as_ref().unwrap().tick, f.tick);
    assert_eq!(f.camera.as_ref().unwrap().tick, f.tick);
    assert_ne!(f.nodes[&1].translation, first.nodes[&1].translation);
    assert!(f.animations.as_ref().unwrap().slots[0].is_some());
    assert_eq!(
        h.resource_snapshot()["mesh_uploads"],
        warm,
        "pose must reuse mesh/UV uploads"
    );
    assert_eq!(
        assets::decode_attempts(),
        decodes,
        "pose must not decode resources again"
    );
    let mesh = &asset.model.meshes[0];
    assert!(
        autographics_auto_driver::authoring::runtime_node_animation::mesh_matrix(
            glam::Mat4::from_scale(glam::Vec3::splat(1e20)),
            Some(v),
            mesh
        )
        .is_err(),
        "final composition overflow cannot reach either GPU pass"
    );
    println!(
        "resource_usage={:?}",
        autographics_auto_driver::authoring::textured_models::usage(&h.editor.assets)
    );
    // Rebuild the exact wire from the accepted real producer, then mutate one
    // field at a time through the production independent Host validator.
    use autographics_auto_driver::authoring::{hierarchy, runtime_node_animation as visual};
    let targets = visual::targets(
        &f.document,
        &hierarchy::ordered(&f.document).unwrap(),
        &h.editor.assets,
    )
    .unwrap();
    let mut wire = vec![
        Int(1),
        Int(targets.len() as i32),
        Int(f.run_id),
        Int(f.tick),
    ];
    for t in &targets {
        let v = f
            .animations
            .as_ref()
            .unwrap()
            .visual_for(t.node_id)
            .unwrap();
        wire.extend([
            Int(t.node_index),
            Int(t.asset_index),
            Int(v.clip),
            Int(v.looping as i32),
            Float(v.phase),
            Float(v.yaw),
        ]);
    }
    for slot in &f.animations.as_ref().unwrap().slots {
        if let Some(slot) = slot {
            wire.extend([
                Int(slot.generation),
                Int(0),
                Int(slot.visual.clip),
                Int(slot.visual.looping as i32),
                Float(slot.visual.phase),
                Float(slot.visual.yaw),
            ]);
        } else {
            wire.extend([Int(0), Int(0), Int(0), Int(0), Float(0.), Float(0.)]);
        }
    }
    let admit = |wire: &[autographics_auto_driver::Scalar]| {
        visual::admit_block(wire, f.run_id, f.tick, &targets, f.instances.as_ref())
    };
    assert_eq!(
        admit(&wire).unwrap().visual_for(1).unwrap().worlds,
        v.worlds
    );
    let mutants = [
        (0, Int(0)),
        (1, Int(0)),
        (2, Int(f.run_id + 1)),
        (3, Int(f.tick + 1)),
        (4, Int(1)),
        (5, Int(1)),
        (6, Int(4)),
        (6, Int(-2)),
        (7, Int(2)),
        (8, Float(-1.)),
        (8, Float(f32::NAN)),
        (8, Int(0)),
        (9, Float(f32::INFINITY)),
        (9, Float(7000.)),
        (10, Int(2)),
        (11, Int(1)),
        (16, Int(1)),
        (20, Float(1.)),
    ];
    for (at, value) in mutants {
        let mut mutated = wire.clone();
        mutated[at] = value;
        assert!(admit(&mutated).is_err(), "wire field{at}");
    }
    assert!(admit(&wire[..wire.len() - 1]).is_err());
    println!(
        "Host_wire_controls={} prepare_samples={} vm_samples={} tick_us={}",
        mutants.len() + 1,
        h.prepare_measurement.samples_ns.len(),
        h.play
            .as_ref()
            .unwrap()
            .vm_output_measurement
            .samples_ns
            .len(),
        h.play.as_ref().unwrap().reports.back().unwrap().elapsed_us
    );
    h.frame(None).unwrap();
    let changed = h.rt.capture(h.three, "ui006-pose").unwrap().0;
    assert_ne!(
        initial, changed,
        "actual GPU image must respond to Auto pose"
    );
    // Hold positions, camera, yaw, texture and actual meshes fixed; erase ONLY
    // each source-node track delta in the real rendered instance matrices.
    // This control cannot pass merely because the camera or actor moved.
    if let Some(agp::host::surface_runtime::SurfaceContent::Scene3d(scene)) =
        h.rt.content_mut(h.three)
    {
        for (i, instance) in scene.instances.iter_mut().enumerate() {
            let mesh = asset
                .model
                .meshes
                .iter()
                .find(|m| m.name == instance.mesh.1)
                .unwrap();
            let meta = mesh.visual.as_ref().unwrap();
            let visual = if i < asset.model.meshes.len() {
                v
            } else {
                &f.animations.as_ref().unwrap().slots[0]
                    .as_ref()
                    .unwrap()
                    .visual
            };
            let delta = visual.worlds[meta.node_index]
                * glam::Mat4::from_cols_array_2d(&meta.world).inverse();
            instance.model *= delta.inverse();
        }
    } else {
        panic!("3D surface")
    }
    h.rt.graph.mark_content_dirty(h.three);
    h.frame(None).unwrap();
    let frozen =
        h.rt.capture(h.three, "ui006-tracks-removed-control")
            .unwrap()
            .0;
    assert_ne!(
        changed, frozen,
        "removing only node tracks must change actual GPU pixels"
    );
    println!(
        "same_frame_tick={} meshes={} cpu_bytes={} original_sha={} pixels={} -> {}",
        f.tick,
        asset.model.meshes.len(),
        asset.cpu_bytes,
        asset.identity.sha256,
        initial,
        changed
    );
    h.stop_play();
}

fn session(user: &str) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("node-animation-probe.at"),
        user.as_bytes().to_vec(),
        Api::NodeAnimation3V1,
    )
    .unwrap();
    let managed = snap.source.len() - user.len();
    assert!(managed <= MAX_BRIDGE);
    println!("managed_bytes={managed}, user_bytes={}", user.len());
    let mut s = Session::new(&snap.source).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(3)], ResultKind::Unit)
        .unwrap();
    for n in 0..3 {
        s.call(
            "_ag_identity",
            &[Int(n + 1), Int(4), Int(3), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
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
        for name in ["_ag_char3_map", "_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n + 1), Int(n)], ResultKind::Unit)
                .unwrap();
        }
    }
    s.call(
        "_ag_vis_map",
        &[Int(1), Int(0), Int(0), Int(4)],
        ResultKind::Unit,
    )
    .unwrap();
    s.call(
        "_ag_vis_map",
        &[Int(3), Int(2), Int(0), Int(4)],
        ResultKind::Unit,
    )
    .unwrap();
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "start");
    s
}
const START:&str="var ok=0\nvar obj=0\nvar phase=0.0\nvar clip=1\nvar step=0\nfn app_start(){ok=ag_char3_bind(1);ok=ag_camera_orbit_3d(1,8.0,45.0,0,0.0);obj=ag_object_spawn(0,3,0.0,0.0,0.0);ok=ag_trigger_bind(0,1,1,obj,0,0.0)}\n";

#[test]
fn public_profile_and_real_auto_combination_fit_unchanged_envelopes() {
    assert_eq!(
        Api::parse_cli("node-animation3d"),
        Some(Api::NodeAnimation3V1)
    );
    assert_eq!(Api::NodeAnimation3V1.number(), 110);
    let user=format!("{START}fn app_update(dt float){{\nstep=step+1;phase=phase+dt;\nlet fx=ag_cam3_forward_x();let fz=ag_cam3_forward_z();\nlet token=ag_char3_move(fx*dt,0.01,fz*dt,0.002);\nlet ground=ag_char3_grounded(token);\nclip=2;if ground==0 {{clip=3}};\nlet e=ag_event_next();\nif e==1 {{let pair=ag_event_pair();ok=pair}};\nok=ag_visual_pose(0,1,clip,phase,1,0.5);\n}}\n");
    let mut s = session(&user);
    for tick in 1..=3 {
        s.call("_ag_tick", &[Int(tick), Float(1. / 60.)], ResultKind::Unit)
            .unwrap();
        assert_eq!(
            s.int("_ag_status", &[]).unwrap(),
            0,
            "move/jump+grounded+camera+trigger read+visual at tick{tick}"
        );
        assert_eq!(s.int("_ag_vis_pi", &[Int(0), Int(2)]).unwrap(), 3);
        assert!(
            (s.float("_ag_vis_pf", &[Int(0), Int(0)]).unwrap() - tick as f32 / 60.).abs() < 1e-7
        );
        assert_eq!(s.int("_ag_vis_tick", &[]).unwrap(), tick);
    }
}

#[test]
fn visual_writes_really_share_the_spatial_credit_window() {
    // Nine visual writes alone pass their own 32-operation cap. With the
    // character's sweep they must hit shared spatial error10, so a mistakenly
    // free visual setter cannot satisfy this distinguishing negative control.
    let mut s=session(&format!("{START}fn app_update(dt float){{let q=ag_char3_move(0.001,0.01,0.0,0.002);for i in 0..9 {{ok=ag_visual_pose(0,1,2,0.1,1,0.0)}}}}\n"));
    s.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 10);
}

#[test]
fn full_input_movement_trigger_and_visual_combination_exposes_scalar_read_budget_limit() {
    // Permanent baseline counterexample for the T-02 design revision: all four
    // direction keys + jump edge are five charged reads before the character
    // sweep, grounded, one event read and the required atomic visual write.
    let user=format!("{START}fn app_update(dt float){{let l=ag_action_held(0);let r=ag_action_held(1);let f=ag_action_held(2);let b=ag_action_held(3);let jump=ag_action_pressed(4);let q=ag_char3_move(0.01,0.01,0.01,0.002);let g=ag_char3_grounded(q);let e=ag_event_next();ok=ag_visual_pose(0,1,3,dt,1,0.0)}}\n");
    let mut s = session(&user);
    s.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        10,
        "the frozen scalar input path really exceeds 32, rather than a hypothetical sum"
    );
}

#[test]
fn packed_actions_encode_all_six_bits_and_share_pressed_consumption_with_legacy_getters() {
    let mut s=session(&format!("{START}var packet=0\nvar repeated=0\nvar legacy=0\nvar oldjump=0\nvar premix=0\nfn setmix(v int){{premix=v}}\nfn getpacket() int{{return packet}}\nfn getrepeat() int{{return repeated}}\nfn getlegacy() int{{return legacy}}\nfn getoldjump() int{{return oldjump}}\nfn app_update(dt float){{if premix==1 {{oldjump=ag_action_pressed(4)}};packet=ag_actions_snapshot();repeated=ag_actions_snapshot();legacy=0;for i in 0..6 {{legacy=legacy+ag_action_pressed(i)}}}}\n"));
    for (i, (held, pressed)) in [
        (0, 0),
        (1, 1),
        (2, 2),
        (4, 4),
        (8, 8),
        (16, 16),
        (32, 32),
        (63, 63),
        (21, 42),
    ]
    .into_iter()
    .enumerate()
    {
        s.call(
            "_ag_char3_input",
            &[Int(held), Int(pressed)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call(
            "_ag_tick",
            &[Int(i as i32 + 1), Float(1. / 60.)],
            ResultKind::Unit,
        )
        .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
        assert_eq!(s.int("getpacket", &[]).unwrap(), held + 64 * pressed);
        assert_eq!(s.int("getrepeat", &[]).unwrap(), held);
        assert_eq!(s.int("getlegacy", &[]).unwrap(), 0);
    }
    s.call("setmix", &[Int(1)], ResultKind::Unit).unwrap();
    s.call("_ag_char3_input", &[Int(18), Int(18)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(10), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("getoldjump", &[]).unwrap(), 1);
    assert_eq!(s.int("getpacket", &[]).unwrap(), 18 + 64 * 2);
    assert_eq!(s.int("getrepeat", &[]).unwrap(), 18);
    assert_eq!(s.int("getlegacy", &[]).unwrap(), 0);
}

#[test]
fn packed_input_full_movement_camera_event_fields_and_visual_use_exactly_thirty_two_credits() {
    let update="let packet=ag_actions_snapshot();let held=packet%64;let pressed=packet/64;let left=held%2;let right=(held/2)%2;let forward=(held/4)%2;let back=(held/8)%2;let jump=(pressed/16)%2;let fx=ag_cam3_forward_x();let fz=ag_cam3_forward_z();let rx=ag_cam3_right_x();let rz=ag_cam3_right_z();let q=ag_char3_move(((right - left)*1.0*rx+(forward - back)*1.0*fx)*dt,jump*dt,((right - left)*1.0*rz+(forward - back)*1.0*fz)*dt,0.002);let grounded=ag_char3_grounded(q);let e=ag_event_next();let pair=ag_event_pair();let ep=ag_event_phase();ok=pair+ep+e;clip=1;if held%16!=0 {clip=2};if jump==1 {clip=3};phase=phase+dt;ok=ag_visual_pose(0,1,clip,phase,1,0.5);";
    for extra in [false, true] {
        let mut s = session(&format!(
            "{}fn getevent() int{{return ok}}\nfn app_update(dt float){{{update}{}}}\n",
            START
                .replace(
                    "ok=ag_char3_bind(1)",
                    "ag_set_local(1,0,10.0);ag_set_local(2,0,20.0);ok=ag_char3_bind(1)"
                )
                .replace(
                    "0.0,0.0,0.0);ok=ag_trigger_bind",
                    "20.0,0.0,0.0);ok=ag_trigger_bind"
                ),
            if extra {
                "let over=ag_actions_snapshot();"
            } else {
                ""
            }
        ));
        for tick in 1..=2 {
            s.call(
                "_ag_char3_input",
                &[Int(2), Int(if tick == 2 { 16 } else { 0 })],
                ResultKind::Unit,
            )
            .unwrap();
            s.call("_ag_tick", &[Int(tick), Float(1. / 60.)], ResultKind::Unit)
                .unwrap();
            assert_eq!(
                s.int("_ag_status", &[]).unwrap(),
                if extra { 10 } else { 0 }
            );
            if extra {
                break;
            }
            assert_eq!(
                s.int("_ag_vis_pi", &[Int(0), Int(2)]).unwrap(),
                if tick == 2 { 3 } else { 2 }
            );
        }
    }
    println!("S06 full input+move26+grounded+event_next/pair/phase+visual=32 fault0; additional snapshot causes error10");
}

#[test]
fn public_setter_rejects_stale_instance_and_invalid_clip_or_phase() {
    for expression in [
        "ag_visual_pose(0,1,4,0.0,1,0.0)",
        "ag_visual_pose(0,1,1,-1.0,1,0.0)",
        "ag_visual_pose(0,1,1,0.0,2,0.0)",
        "ag_visual_pose(0,1,1,0.0,1,7000.0)",
        "ag_visual_pose(1,obj+1,1,0.0,1,0.0)",
        "ag_visual_pose(2,1,1,0.0,1,0.0)",
    ] {
        let mut s = session(&format!(
            "{START}fn app_update(dt float){{ok={expression}}}\n"
        ));
        s.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
            .unwrap();
        assert_ne!(s.int("_ag_status", &[]).unwrap(), 0, "{expression}");
    }
    let mut s = session(&format!(
        "{START}fn app_update(dt float){{ok=ag_visual_pose(1,obj,2,0.25,0,1.0)}}\n"
    ));
    s.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(s.int("_ag_vis_qi", &[Int(0), Int(2)]).unwrap(), 2);
}

#[test]
fn reused_slot_restores_template_initial_visual_instead_of_previous_generation() {
    let start = START.replace("ok=ag_trigger_bind(0,1,1,obj,0,0.0)",
        "ok=ag_visual_pose(1,obj,3,0.4,1,1.2);ok=ag_object_despawn(obj);obj=ag_object_spawn(0,3,2.0,0.0,0.0);ok=ag_trigger_bind(0,1,1,obj,0,0.0)");
    let mut s = session(&format!("{start}fn app_update(dt float){{}}\n"));
    assert_eq!(s.int("_ag_vis_qi", &[Int(0), Int(0)]).unwrap(), 2);
    assert_eq!(s.int("_ag_vis_qi", &[Int(0), Int(2)]).unwrap(), -1);
    assert_eq!(s.int("_ag_vis_qi", &[Int(0), Int(3)]).unwrap(), 0);
    for field in 0..2 {
        assert_eq!(s.float("_ag_vis_qf", &[Int(0), Int(field)]).unwrap(), 0.);
    }
}
