//! Real input/transaction/lifetime path; no private VM input injection.
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    interaction,
    play::{Action, Frame, State},
};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn pump(h: &mut Host, success: bool) {
    let end = Instant::now() + Duration::from_secs(60);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < end, "worker deadline");
        h.frame(None).unwrap();
    }
    if success {
        assert!(
            h.play.as_ref().unwrap().error.is_none(),
            "{:?}",
            h.play.as_ref().unwrap().error
        );
    }
}
fn fixture() -> (Document, Registry) {
    let a = assets::import_animated(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/textured_models/models/character.glb"),
        &Registry::new(),
    )
    .unwrap();
    let mut d = Document::empty("ui006-input-lifetime".into());
    d.schema = "autographics.scene/6".into();
    d.assets.push(a.identity.clone());
    let (lo, hi) = interaction::bounds(&a.model).unwrap();
    let size = hi - lo;
    let mut actor = Object::new(1, Kind::Model);
    actor.resource = Some(a.identity.key());
    actor.scale = [0.8 / size.x, 1. / size.y, 0.8 / size.z];
    actor.translation[1] = -lo.y * actor.scale[1] + 0.002;
    d.objects.push(actor.clone());
    actor.id = 2;
    actor.visible = false;
    actor.translation[0] = 30.;
    d.objects.push(actor);
    let mut marker = Object::new(3, Kind::Quad);
    marker.visible = false;
    d.objects.push(marker);
    let mut floor = Object::new(4, Kind::Cylinder);
    floor.translation = [0., -0.5, 0.];
    floor.scale = [10., 0.5, 10.];
    d.objects.push(floor);
    let mut wall = Object::new(5, Kind::Cylinder);
    wall.translation = [2., 1., 0.];
    wall.scale = [0.1, 1., 5.];
    d.objects.push(wall);
    (d, Registry::from([(a.identity.key(), a)]))
}
fn scene() -> Host {
    let (d, r) = fixture();
    Host::with_assets(d, r).unwrap()
}
const CONTROLLER: &str = r#"
var ok=0
var step=0
var phase=0.0
var yaw=0.0
fn app_start(){
 ok=ag_char3_bind(1);ok=ag_char3_solid(4);ok=ag_char3_solid(5)
 ok=ag_camera_orbit_3d(1,4.0,30.0,0,0.0)
 ok=ag_trigger_bind(0,1,0,2,1,40.0)
 ok=ag_visual_pose(0,1,1,0.0,1,0.0)
}
fn app_update(dt float){
 step=step+1
 let packet=ag_actions_snapshot();let held=packet%64;let pressed=packet/64
 let left=held%2;let right=(held/2)%2;let forward=(held/4)%2;let back=(held/8)%2;let jump=(pressed/16)%2
 let fx=ag_cam3_forward_x();let fz=ag_cam3_forward_z();let rx=ag_cam3_right_x();let rz=ag_cam3_right_z()
 let dx=((right - left)*1.0*rx+(forward - back)*1.0*fx)*dt*4.0
 let dz=((right - left)*1.0*rz+(forward - back)*1.0*fz)*dt*4.0
 let q=ag_char3_move(dx,jump*dt*6.0-0.01,dz,0.002)
 let grounded=ag_char3_grounded(q)
 let e=ag_event_next();var pair=0;var ep=0;if e==1 {pair=ag_event_pair();ep=ag_event_phase()}
 ag_set_local(2,0,packet*1.0);ag_set_local(2,1,(e*100+pair*10+ep)*1.0)
 var clip=1
 if held%16!=0 {
  clip=2;yaw=ag_cam3_yaw()*0.01745329252
  if forward==1 {yaw=yaw+3.14159265359}
  if right==1 {yaw=yaw+1.57079632679}
  if left==1 {yaw=yaw-1.57079632679}
 }
 if grounded==0 {clip=3}
 phase=phase+dt
 ok=ag_visual_pose(0,1,clip,phase,1,yaw)
 // extra-control
}
"#;
fn start(h: &mut Host, user: &str) {
    let s = SourceSnapshot::from_bytes_for(
        Path::new("input-controller.at"),
        user.as_bytes().to_vec(),
        Api::NodeAnimation3V1,
    )
    .unwrap();
    h.start_user_snapshot(s).unwrap();
    h.pause_play("");
    pump(h, true);
}
fn input(h: &mut Host, held: u8, camera: f32, success: bool) {
    let p = h.play.as_mut().unwrap();
    p.resume().unwrap();
    for i in 0..6 {
        p.char_key(i, held & (1 << i) != 0, false);
    }
    p.cam_drag(camera, 0.);
    p.dispatch(Action::default()).unwrap();
    h.pause_play("");
    pump(h, success);
}
fn last(h: &Host) -> &Frame {
    h.play.as_ref().unwrap().last.as_ref().unwrap()
}
fn visual(h: &Host) -> &autographics_auto_driver::authoring::runtime_node_animation::Visual {
    last(h).animations.as_ref().unwrap().visual_for(1).unwrap()
}

#[test]
fn full_input_event_camera_visual_budget_and_neutral_step_are_real_host_transactions() {
    let mut h = scene();
    start(&mut h, CONTROLLER);
    let uploads = h.resource_snapshot()["mesh_uploads"].clone();
    let decodes = assets::decode_attempts();
    let resources = h.resource_snapshot();
    let union = &resources["asset_union"];
    let pose = union["pose_reserved_cpu_bytes"].as_u64().unwrap();
    assert!(pose > 0);
    assert_eq!(
        union["cpu_total_with_pose_bytes"].as_u64().unwrap(),
        union["cpu_geometry_bytes"].as_u64().unwrap() + pose
    );
    assert!(union["cpu_total_with_pose_bytes"].as_u64().unwrap() <= 64 * 1024 * 1024);
    println!("CPU geometry+animation+pose ledger: {union}");
    let first = last(&h).clone();
    input(&mut h, 2, 30., true);
    let moved = last(&h).clone();
    assert_eq!(moved.nodes[&3].translation[0], 130.); // held Right + pressed Right
    assert_eq!(moved.nodes[&3].translation[1], 0.); // evaluation publishes after callback; tick2 consumes enter
    assert_eq!(visual(&h).clip, 2);
    assert_ne!(moved.nodes[&1].translation, first.nodes[&1].translation);
    assert!((moved.camera.as_ref().unwrap().yaw - 30.).abs() < 1e-5);
    assert!((visual(&h).yaw - std::f32::consts::PI * 2. / 3.).abs() < 1e-5);
    input(&mut h, 16, 0., true);
    assert_eq!(visual(&h).clip, 3);
    assert_eq!(last(&h).nodes[&3].translation[0], 1040.);
    assert_eq!(last(&h).nodes[&3].translation[1], 101.);
    let before = last(&h).clone();
    for _ in 0..3 {
        h.frame(None).unwrap();
    }
    assert_eq!(last(&h).tick, before.tick);
    h.play.as_mut().unwrap().char_key(4, true, false); // paused input must not leak into Step
    h.play.as_mut().unwrap().cam_drag(70., 0.);
    h.play.as_mut().unwrap().single_step().unwrap();
    pump(&mut h, true);
    assert_eq!(last(&h).tick, before.tick + 1);
    assert_eq!(last(&h).nodes[&3].translation[0], 0.);
    assert_eq!(visual(&h).clip, 3);
    assert!((visual(&h).phase - 3. / 60.).abs() < 1e-6);
    assert_eq!(
        last(&h).camera.as_ref().unwrap().yaw,
        before.camera.as_ref().unwrap().yaw
    );
    assert_eq!(h.resource_snapshot()["mesh_uploads"], uploads);
    assert_eq!(assets::decode_attempts(), decodes);
    println!(
        "Host32: packet130/1040 -> neutral0; enter101/stay102; camera30; tick_us={}",
        h.play.as_ref().unwrap().reports.back().unwrap().elapsed_us
    );
    h.stop_play();
    start(&mut h, CONTROLLER);
    assert_eq!(visual(&h).phase, 0.);
    assert_eq!(visual(&h).yaw, 0.);
    assert_eq!(visual(&h).clip, 1);
    assert_ne!(last(&h).run_id, first.run_id);
    h.stop_play();

    // The extra credit is spent only on tick2, after a real last-good frame.
    let over = CONTROLLER.replace(
        "// extra-control",
        "if step==2 {let over=ag_actions_snapshot()}",
    );
    start(&mut h, &over);
    input(&mut h, 2, 0., true);
    let good = last(&h).clone();
    h.frame(None).unwrap();
    let image = h.rt.capture(h.three, "before-credit-fault").unwrap().0;
    input(&mut h, 16, 50., false);
    assert!(h
        .play
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap()
        .contains("10"));
    assert_eq!(last(&h).tick, good.tick);
    assert_eq!(last(&h).nodes, good.nodes);
    assert_eq!(last(&h).rounds, good.rounds);
    assert_eq!(
        visual(&h).worlds,
        good.animations
            .as_ref()
            .unwrap()
            .visual_for(1)
            .unwrap()
            .worlds
    );
    h.frame(None).unwrap();
    assert_eq!(
        h.rt.capture(h.three, "after-credit-fault").unwrap().0,
        image
    );
    assert_eq!(h.play.as_ref().unwrap().state, State::Faulted);
    h.stop_play();
    start(&mut h, CONTROLLER);
    input(&mut h, 1, 0., true);
    h.stop_play();
}

fn assert_preserved(h: &mut Host, good: &Frame, image: &str) {
    assert_eq!(last(h).tick, good.tick);
    assert_eq!(last(h).run_id, good.run_id);
    assert_eq!(last(h).nodes, good.nodes);
    assert_eq!(last(h).rounds, good.rounds);
    let old = good.animations.as_ref().unwrap().visual_for(1).unwrap();
    assert_eq!(visual(h).worlds, old.worlds);
    assert_eq!(visual(h).phase, old.phase);
    assert_eq!(last(h).camera, good.camera);
    h.frame(None).unwrap();
    assert_eq!(h.rt.capture(h.three, "preserved-frame").unwrap().0, image);
}

#[test]
fn invalid_visual_and_prepare_failure_retain_whole_last_good_and_stop_recovers() {
    let mut h = scene();
    for expression in [
        "ag_visual_pose(0,1,4,phase,1,0.0)",
        "ag_visual_pose(0,1,1,-1.0,1,0.0)",
        // A live slot is destroyed/reused; its previous full token is stale.
        "ag_visual_pose(1,stale,1,phase,1,0.0)",
        "prepare-failure",
    ] {
        let mut source = CONTROLLER.to_string();
        if expression.contains("stale") {
            source = source.replace("var ok=0", "var ok=0\nvar stale=0\nvar fresh=0").replace(
                "ok=ag_visual_pose(0,1,1,0.0,1,0.0)",
                "stale=ag_object_spawn(0,2,30.0,0.0,0.0);ok=ag_object_despawn(stale);fresh=ag_object_spawn(0,2,31.0,0.0,0.0)");
        }
        // Replace the normal setter rather than add a charged operation:
        // the failure must diagnose this visual value, not credit exhaustion.
        if expression != "prepare-failure" {
            source = source.replace("ok=ag_visual_pose(0,1,clip,phase,1,yaw)", &format!("if step==2 {{ok={expression}}} else {{ok=ag_visual_pose(0,1,clip,phase,1,yaw)}}"));
        }
        start(&mut h, &source);
        input(&mut h, 2, 0., true);
        let good = last(&h).clone();
        h.frame(None).unwrap();
        let image = h.rt.capture(h.three, "before-invalid-visual").unwrap().0;
        h.fail_resource = expression == "prepare-failure";
        input(&mut h, 16, 45., false);
        assert!(h.play.as_ref().unwrap().error.is_some(), "{expression}");
        if expression != "prepare-failure" {
            let code = if expression.contains("stale") {
                33
            } else if expression.contains("-1.0") {
                47
            } else {
                46
            };
            assert!(
                h.play
                    .as_ref()
                    .unwrap()
                    .error
                    .as_ref()
                    .unwrap()
                    .contains(&format!("{code}")),
                "{:?}",
                h.play.as_ref().unwrap().error
            );
        }
        h.fail_resource = false;
        assert_preserved(&mut h, &good, &image);
        h.stop_play();
        assert!(h.play.is_none());
    }
    start(&mut h, CONTROLLER);
    input(&mut h, 4, 0., true);
    h.stop_play();
}

#[test]
fn twenty_play_restart_stop_cycles_release_vm_pose_and_final_payload_mesh_texture_owners() {
    use agp::host::surface_runtime::SurfaceContent;
    use std::sync::Arc;
    let mut h = scene();
    let asset = h.editor.assets.values().next().unwrap();
    let asset_weak = Arc::downgrade(asset);
    let animation_weak = Arc::downgrade(asset.animation.as_ref().unwrap());
    let mesh_pin = match h.rt.content(h.three).unwrap() {
        SurfaceContent::Scene3d(s) => s
            .store
            .pin(&asset.model.source_sha256, &asset.model.meshes[0].name)
            .unwrap(),
        _ => panic!(),
    };
    let mesh_weak = Arc::downgrade(&mesh_pin);
    let texture_pin = mesh_pin.texture.as_ref().unwrap().clone();
    let texture_weak = Arc::downgrade(&texture_pin);
    let source = CONTROLLER.replace(
        "let packet=ag_actions_snapshot();",
        "let packet=ag_actions_snapshot();if packet/64>=32 {ok=ag_round_request();return};",
    );
    let decodes = assets::decode_attempts();
    let baseline = h.resource_snapshot()["mesh_uploads"].clone();
    let texture_baseline = h.resource_snapshot()["model_texture_uploads"].clone();
    for cycle in 0..20 {
        start(&mut h, &source);
        let vm = h.play.as_ref().unwrap().owner_probe().unwrap();
        let descriptor = h.play.as_ref().unwrap().spatial_owner_probe().unwrap();
        input(&mut h, 2, 0., true);
        let pose = Arc::downgrade(&visual(&h).worlds);
        let old_run = last(&h).run_id;
        let p = h.play.as_mut().unwrap();
        p.resume().unwrap();
        p.char_key(5, true, false);
        p.dispatch(Action::default()).unwrap();
        let end = Instant::now() + Duration::from_secs(60);
        while h.play.as_ref().unwrap().run_id == old_run {
            assert!(Instant::now() < end);
            h.frame(None).unwrap();
            assert!(
                h.play.as_ref().unwrap().error.is_none(),
                "{:?}",
                h.play.as_ref().unwrap().error
            );
        }
        h.pause_play("");
        pump(&mut h, true);
        assert!(!vm(), "old VM cycle{cycle}");
        assert!(
            descriptor.upgrade().is_none(),
            "old descriptor cycle{cycle}"
        );
        assert!(pose.upgrade().is_none(), "old sampled pose cycle{cycle}");
        assert_eq!(visual(&h).phase, 0.);
        assert_eq!(visual(&h).clip, 1);
        assert_eq!(visual(&h).yaw, 0.);
        assert_eq!(last(&h).nodes[&3].translation, [0.; 3]);
        let fresh_vm = h.play.as_ref().unwrap().owner_probe().unwrap();
        let fresh_pose = Arc::downgrade(&visual(&h).worlds);
        h.stop_play();
        assert!(!fresh_vm());
        assert!(fresh_pose.upgrade().is_none());
        assert_eq!(h.resource_snapshot()["mesh_uploads"], baseline);
        assert_eq!(
            h.resource_snapshot()["model_texture_uploads"],
            texture_baseline
        );
        assert_eq!(assets::decode_attempts(), decodes);
    }
    h.editor.close();
    h.frame(None).unwrap();
    assert!(asset_weak.upgrade().is_none());
    assert!(animation_weak.upgrade().is_none());
    assert!(mesh_weak.upgrade().is_some(), "legitimate external pin");
    drop(mesh_pin);
    assert!(mesh_weak.upgrade().is_none());
    assert!(texture_weak.upgrade().is_some());
    drop(texture_pin);
    assert!(texture_weak.upgrade().is_none());
    println!("20 cycles: Play/restart/Stop; 40 VMs and poses dropped; warm uploads/decode delta0; final payload/mesh/texture drop verified");
}

#[test]
fn visual_animation_and_yaw_leave_floor_wall_platform_support_and_triggers_identical() {
    let mut observations = Vec::new();
    for platform in [false, true] {
        let mut control = None;
        for animate in [false, true] {
            let (mut d, r) = fixture();
            if platform {
                let mut p = Object::new(6, Kind::Cylinder);
                p.translation = [-3., 0.5, 0.];
                p.scale = [1., 0.5, 1.];
                d.objects.push(p);
                d.objects[0].translation[0] = -3.;
                d.objects[0].translation[1] += 1.;
            }
            let mut h = Host::with_assets(d, r).unwrap();
            // One existing event getter is exchanged for final support. No
            // change to any operation's cost or to the 32-credit envelope.
            let mut source = CONTROLLER.replace("pair=ag_event_pair();", "").replace(
                "ag_set_local(2,0,packet*1.0)",
                "ag_set_local(2,0,(grounded+ag_char3_support(q)*2)*1.0)",
            );
            if !animate {
                source = source.replace(
                    "ag_visual_pose(0,1,clip,phase,1,yaw)",
                    "ag_visual_pose(0,1,-1,0.0,0,0.0)",
                );
            }
            start(&mut h, &source);
            let initial_x = last(&h).nodes[&1].translation[0];
            let mut trace = Vec::new();
            for tick in 1..=60 {
                // Floor case walks into the wall; platform case starts on a
                // raised support, jumps and lands again without walking off.
                let held = if platform {
                    if tick == 3 {
                        16
                    } else {
                        0
                    }
                } else {
                    2 | if tick == 3 { 16 } else { 0 }
                };
                input(&mut h, held, 0., true);
                let f = last(&h);
                trace.push((
                    f.nodes[&1].translation,
                    f.nodes[&3].translation,
                    f.rounds.as_ref().unwrap().events.clone(),
                ));
            }
            let ground_values: Vec<_> = trace.iter().map(|t| t.1[0] as i32).collect();
            assert!(
                ground_values.iter().any(|x| *x % 2 == 0),
                "jump leaves support"
            );
            assert_eq!(ground_values[59] % 2, 1, "landed by tick60");
            assert!(
                trace.iter().any(|t| !t.2.is_empty()),
                "real trigger evaluation"
            );
            if platform {
                assert_eq!(ground_values[59], 11, "platform support run index5");
            } else {
                let end = trace[59].0[0];
                assert!(end - initial_x > 1.);
                assert!(end < 1.501, "wall stops character at {end}");
                assert_eq!(ground_values[59], 7, "floor support run index3");
            }
            if let Some(ref previous) = control {
                assert_eq!(
                    previous, &trace,
                    "pose/yaw cannot change physical or trigger output"
                );
            } else {
                control = Some(trace);
            }
            observations.push((platform, animate, ground_values[59]));
            h.stop_play();
        }
    }
    println!("240 real input ticks: animation/yaw on/off equal positions, support/grounded and trigger events; floor/wall/platform={observations:?}");
}

#[test]
fn four_direction_motion_matches_independent_camera_basis_and_stopping_retains_yaw() {
    let mut h = scene();
    let (sin, cos) = 30f32.to_radians().sin_cos();
    for (key, dx, dz) in [
        (1, -cos, sin),
        (2, cos, -sin),
        (4, -sin, -cos),
        (8, sin, cos),
    ] {
        start(&mut h, CONTROLLER);
        let origin = last(&h).nodes[&1].translation;
        input(&mut h, key, 30., true);
        let moved = last(&h).nodes[&1].translation;
        assert!((moved[0] - origin[0] - dx * 4. / 60.).abs() < 1e-6);
        assert!((moved[2] - origin[2] - dz * 4. / 60.).abs() < 1e-6);
        let yaw = visual(&h).yaw;
        assert!((yaw.sin() - dx).abs() < 1e-5);
        assert!((yaw.cos() - dz).abs() < 1e-5);
        assert_eq!(visual(&h).clip, 2);
        input(&mut h, 0, 0., true);
        assert_eq!(visual(&h).clip, 1);
        assert_eq!(last(&h).nodes[&1].translation, moved);
        assert_eq!(visual(&h).yaw, yaw);
        h.stop_play();
    }
    println!("four real camera-relative motion directions: displacement and +Z facing agree with independent sin/cos, stop retains position/yaw and selects idle");
}
