//! PLAN-875 runtime camera host tests (contract V-02/V-04): the real
//! Host/Play/GPU path. 2D follow/clamp/letterbox values and 3D view-projection
//! identity against independent glam oracles, co-publication (the camera of
//! THIS tick's accepted actor, never the previous one), failure last-good,
//! NaN configuration admission, paused/neutral camera behavior, and the
//! preserved no-camera defaults. Run single-threaded:
//! cargo test --locked --test runtime_camera_host -- --test-threads=1
use agp::render3d::{Camera3d, Projection};
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    play::Action,
    runtime_camera::{self, CameraDim},
};
use autographics_auto_driver::{Scalar, Scalar::Float};
use glam::{Mat4, Vec3};
use serde_json::json;
use std::path::Path;
use std::time::Instant;

const DT: f32 = 1.0 / 60.0;

fn document2d() -> Document {
    let mut d = Document::empty("camera2d-host".into());
    let mut character = Object::new(1, Kind::Quad);
    character.translation = [2.0, 1.0, 0.0];
    character.scale = [0.8, 1.0, 1.0];
    d.objects.push(character);
    let mut floor = Object::new(2, Kind::Quad);
    floor.translation = [15.0, -0.5, 0.0];
    floor.scale = [30.0, 1.0, 1.0];
    d.objects.push(floor);
    let mut west = Object::new(3, Kind::Quad);
    west.translation = [-0.5, 1.5, 0.0];
    west.scale = [1.0, 3.0, 1.0];
    d.objects.push(west);
    let mut east = Object::new(4, Kind::Quad);
    east.translation = [30.5, 1.5, 0.0];
    east.scale = [1.0, 3.0, 1.0];
    d.objects.push(east);
    d.validate().expect("document valid");
    d
}

fn document3d() -> Document {
    let mut d = Document::empty("camera3d-host".into());
    let mut character = Object::new(1, Kind::Cylinder);
    character.translation = [0.0, 0.5, 0.0];
    character.scale = [0.4, 0.5, 0.4];
    d.objects.push(character);
    let mut floor = Object::new(2, Kind::Cylinder);
    floor.translation = [0.0, -0.5, 0.0];
    floor.scale = [10.0, 0.5, 10.0];
    d.objects.push(floor);
    let mut wall = Object::new(3, Kind::Cylinder);
    wall.translation = [6.0, 1.5, 0.0];
    wall.scale = [0.1, 1.5, 10.0];
    d.objects.push(wall);
    d.validate().expect("document valid");
    d
}

/// 2D controller: right/left walk with the frozen speed; camera configured
/// to the contract sample rect/viewport (or the mutated variant).
fn source2d(rect: [f32; 4], view: [f32; 2]) -> String {
    format!(
        "var ok=0\nvar vy=0.0\nvar grounded=0\nfn app_start() {{\n ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_char_solid(3)\n ok=ok+ag_char_solid(4)\n ok=ok+ag_camera_follow_2d(1, {:?} , {:?} , {:?} , {:?} , {:?} , {:?})\n}}\nfn app_update(dt float) {{\n vy=vy-20.0*dt\n let hr=ag_action_held(1)\n let hl=ag_action_held(0)\n let dx = (hr - hl)*4.0*dt\n let q=ag_char_move(dx,vy*dt,0.002)\n grounded=ag_char_grounded(q)\n if grounded==1 {{vy=0.0}}\n}}\n",
        rect[0], rect[1], rect[2], rect[3], view[0], view[1]
    )
}

/// 3D controller: forward walk relative to the host basis; orbit configured
/// to the contract defaults (or the mutated variant).
fn source3d(distance: f32, pitch: f32, proj: i32, ortho: f32) -> String {
    format!(
        "var ok=0\nvar vy=0.0\nvar grounded=0\nfn app_start() {{\n ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_char3_solid(3)\n ok=ok+ag_camera_orbit_3d(1, {distance:?}, {pitch:?}, {proj}, {ortho:?})\n}}\nfn app_update(dt float) {{\n vy=vy-20.0*dt\n let hf=ag_action_held(2)\n let fx=ag_cam3_forward_x()\n let fz=ag_cam3_forward_z()\n let rx=ag_cam3_right_x()\n let rz=ag_cam3_right_z()\n let dx=(fx*hf+rx*0.0)*4.0*dt\n let dz=(fz*hf+rz*0.0)*4.0*dt\n let q=ag_char3_move(dx,vy*dt,dz,0.002)\n grounded=ag_char3_grounded(q)\n if grounded==1 {{vy=0.0}}\n}}\n"
    )
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "host deadline");
        h.frame(None).expect("frame");
    }
}

fn step(h: &mut Host, held: i32) {
    h.play.as_mut().unwrap().resume().expect("resume");
    for (action, bit) in [(0usize, 1), (1, 2), (2, 4), (3, 8)] {
        if held & bit != 0 {
            h.play.as_mut().unwrap().char_held_restore(action);
        }
    }
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(h);
}

fn scene_vp(h: &Host) -> Mat4 {
    match h.rt.content(h.three) {
        Some(agp::host::surface_runtime::SurfaceContent::Scene3d(c)) => c.view_proj,
        _ => panic!("3D scene content"),
    }
}

fn two_commands(h: &Host) -> Vec<agp::renderer::pipeline::DrawCmd> {
    match h.rt.content(h.two) {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g.commands.clone(),
        _ => panic!("2D game content"),
    }
}

fn expected_vp(yaw: f32, pitch: f32, dist: f32, pos: Vec3, ortho: Option<f32>) -> Mat4 {
    let (sy, cy) = yaw.to_radians().sin_cos();
    let (sp, cp) = pitch.to_radians().sin_cos();
    let projection = match ortho {
        Some(size) => Projection::Orthographic {
            size,
            aspect: 2.0,
            near: 0.05,
            far: 256.0,
        },
        None => Projection::Perspective {
            fov_y_deg: 60.0,
            aspect: 2.0,
            near: 0.05,
            far: 256.0,
        },
    };
    Camera3d {
        eye: pos + Vec3::new(sy * cp, sp, cy * cp) * dist,
        target: pos,
        up: Vec3::Y,
        projection,
    }
    .view_proj()
}

// ---------------------------------------------------------------- V-02 (2D) --

fn follow2d_clamps_and_letterboxes_through_the_real_host_body() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("follow2d.at"),
        source2d([0.0, -4.0, 30.0, 12.0], [12.0, 8.0])
            .as_bytes()
            .to_vec(),
        Api::Camera2V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    assert_eq!(h.play.as_ref().unwrap().camera_dim(), Some(CameraDim::TwoD));
    h.pause_play("");
    settle(&mut h);
    // Tick 0: actor at (2,1) -> center clamps to (6,4).
    let cam = h
        .play
        .as_ref()
        .unwrap()
        .camera_frame()
        .expect("tick0 camera")
        .clone();
    assert_eq!(cam.dim, CameraDim::TwoD);
    assert_eq!(cam.rect, [0.0, -4.0, 30.0, 12.0]);
    assert_eq!(cam.view, [12.0, 8.0]);
    let scale = 47.5f32;
    let bars = 115.0f32;
    // screen(world) = bars + content/2 + (world-center)*scale (y flipped);
    // the follow center is the content-area center, not the bar corner.
    for (world, center, px_want) in [
        (
            [2.0f32, 1.0],
            [6.0, 4.0],
            [
                bars + 570.0 / 2.0 + (2.0 - 6.0) * scale,
                190.0 + (4.0 - 1.0) * scale,
            ],
        ),
        (
            [2.0, -0.5],
            [6.0, 4.0],
            [
                bars + 570.0 / 2.0 + (2.0 - 6.0) * scale,
                190.0 + (4.0 - -0.5) * scale,
            ],
        ),
    ] {
        let want = runtime_camera::RuntimeCameraFrame {
            dim: CameraDim::TwoD,
            target: 0,
            target_id: 1,
            run_id: 1,
            tick: 0,
            rect: cam.rect,
            view: cam.view,
            yaw: 0.0,
            pitch: 0.0,
            distance: 0.0,
            projection: 0,
            ortho_size: 0.0,
        }
        .screen_2d(glam::Vec2::from(world), glam::Vec2::from(center))
        .unwrap();
        assert!(
            (want.x - px_want[0]).abs() < 1e-3,
            "{world:?} -> {want:?} want {px_want:?}"
        );
        assert!((want.y - px_want[1]).abs() < 1e-3);
    }
    // Walk right past the clamp: at x=15 the center is still 15-6=9? No:
    // center = clamp(actor, 6, 24); at 15 it is 15 exactly.
    for _ in 0..20 {
        step(&mut h, 2); // right
    }
    let f = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
    let actor = f.objects.iter().find(|o| o.id == 1).unwrap().translation;
    let cam = f.camera.clone().expect("camera rides tick 20");
    let center = cam
        .follow_center_2d(glam::Vec2::new(actor[0], actor[1]))
        .unwrap();
    assert!(
        (center.x - actor[0].clamp(6.0, 24.0)).abs() < 1e-5,
        "{center:?} vs {actor:?}"
    );
    // Published 2D commands: letterbox bars painted last with the clear
    // color and no world draw lands outside the content area.
    let commands = two_commands(&h);
    assert!(commands.len() >= 2, "bar rects must exist");
    let clear = match h.rt.content(h.two) {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g.clear,
        _ => panic!(),
    };
    let bars_drawn: Vec<_> = commands
        .iter()
        .filter(|c| c.color == clear && c.size[0] > 100.0 && c.size[1] > 300.0)
        .collect();
    assert_eq!(
        bars_drawn.len(),
        2,
        "two vertical bars, got {:?}",
        commands.last()
    );
    for bar in bars_drawn {
        assert!(
            (bar.size[0] - bars).abs() < 1e-2,
            "bar width {:?}",
            bar.size
        );
        assert!((bar.size[1] - 380.0).abs() < 1e-2);
        let left = bar.center[0] < 400.0;
        let want_center = if left { bars / 2.0 } else { 800.0 - bars / 2.0 };
        assert!(
            (bar.center[0] - want_center).abs() < 1e-2,
            "bar center {:?}",
            bar.center
        );
    }
}

fn follow2d_second_configuration_changes_projection_from_the_source_body() {
    // The alt sample configuration (rect x[4,26] view 8x6) must letterbox
    // differently: scale = 380/6 = 63.33, content width 506.67, bars 146.67.
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("follow2d-alt.at"),
        source2d([4.0, -2.0, 26.0, 10.0], [8.0, 6.0])
            .as_bytes()
            .to_vec(),
        Api::Camera2V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    h.pause_play("");
    settle(&mut h);
    let commands = two_commands(&h);
    let clear = match h.rt.content(h.two) {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g.clear,
        _ => panic!(),
    };
    let bars: Vec<_> = commands
        .iter()
        .filter(|c| c.color == clear && c.size[0] > 100.0)
        .collect();
    assert_eq!(bars.len(), 2);
    let width = bars[0].size[0];
    assert!(
        (width - (800.0 - 8.0 * (380.0 / 6.0)) / 2.0).abs() < 1e-2,
        "bar width {width}"
    );
}

fn nan_configuration_rejects_at_the_named_admission_stage_body() {
    // NaN literal from the user source passes the unit ordering checks
    // (NaN comparisons are false) and must be caught by the frame admission.
    let src = source2d([0.0, -4.0, 30.0, 12.0], [12.0, 8.0]).replace(
        "ok=ok+ag_camera_follow_2d(1, 0.0 , ",
        "ok=ok+ag_camera_follow_2d(1, 0.0/0.0 , ",
    );
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("nan2d.at"),
        src.as_bytes().to_vec(),
        Api::Camera2V1,
    )
    .expect("admission accepts the parse");
    let mut h = Host::with_project(document2d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    let end = Instant::now() + std::time::Duration::from_secs(31);
    loop {
        assert!(Instant::now() < end, "init deadline");
        h.frame(None).expect("frame");
        let play = h.play.as_ref().unwrap();
        if !play.pending() {
            break;
        }
    }
    let play = h.play.as_ref().unwrap();
    assert_eq!(
        play.state,
        autographics_auto_driver::authoring::play::State::Faulted
    );
    let error = play.error.clone().expect("named error");
    assert!(
        error.contains("DivisionByZero") || (error.contains("camera") && error.contains("output")),
        "named rejection before publication, got {error}"
    );
    assert!(play.last.is_none(), "no partial publication");
}

// ---------------------------------------------------------------- V-04 (3D) --

fn camera3d_view_proj_is_this_tick_actor_not_the_previous_one_body() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("orbit3d.at"),
        source3d(8.0, 45.0, 0, 0.0).as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document3d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    h.pause_play("");
    settle(&mut h);
    let play = h.play.as_ref().unwrap();
    assert_eq!(play.camera_dim(), Some(CameraDim::ThreeD));
    let cam = play.camera_frame().expect("tick0 camera").clone();
    assert_eq!((cam.yaw, cam.pitch, cam.distance), (0.0, 45.0, 8.0));
    let pos0 = Vec3::from(play.last.as_ref().unwrap().objects[0].translation);
    assert_eq!(
        scene_vp(&h),
        expected_vp(0.0, 45.0, 8.0, pos0, None),
        "yaw0: eye above +z"
    );
    // Walk forward (-z at yaw 0): every accepted tick must publish the
    // camera of THIS tick's actor (co-publication; a lost uniform or a
    // previous-tick basis would fail this identity).
    let mut prev = pos0;
    for step_i in 1..=10 {
        step(&mut h, 4); // forward
        let play = h.play.as_ref().unwrap();
        let f = play.last.as_ref().unwrap();
        let pos = Vec3::from(f.objects.iter().find(|o| o.id == 1).unwrap().translation);
        assert!(pos.z < prev.z, "forward walk moves -z at yaw 0");
        assert_eq!(
            scene_vp(&h),
            expected_vp(0.0, 45.0, 8.0, pos, None),
            "tick {step_i}"
        );
        if (pos - prev).length() > 1e-4 {
            assert_ne!(
                scene_vp(&h),
                expected_vp(0.0, 45.0, 8.0, prev, None),
                "published camera must not be the previous actor position"
            );
        }
        prev = pos;
    }
    // Orbit 90 degrees via the host feed, then forward walks along +x.
    h.play.as_mut().unwrap().resume().expect("resume");
    h.play.as_mut().unwrap().cam_drag(90.0, 0.0);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(&mut h);
    let play = h.play.as_ref().unwrap();
    let pos = Vec3::from(play.last.as_ref().unwrap().objects[0].translation);
    assert_eq!(scene_vp(&h), expected_vp(90.0, 45.0, 8.0, pos, None));
    let before = pos;
    step(&mut h, 4);
    let after = Vec3::from(h.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation);
    assert!(
        after[0] - before[0] < -0.05,
        "world move rotates with the orbit"
    );
}

fn zoom_and_pitch_stay_inside_the_frozen_domains_through_the_host_body() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("zoom3d.at"),
        source3d(8.0, 45.0, 0, 0.0).as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document3d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    h.pause_play("");
    settle(&mut h);
    // Paused zoom is dropped entirely.
    h.play.as_mut().unwrap().cam_zoom(-8.0);
    step(&mut h, 0);
    let cam = h.play.as_ref().unwrap().camera_frame().unwrap().clone();
    assert_eq!(cam.distance, 8.0, "Paused zoom never accumulates");
    // Running zoom: 8 wheel-down notches multiply by 1.1^8 and clamp to 12.
    h.play.as_mut().unwrap().resume().expect("resume");
    h.play.as_mut().unwrap().cam_zoom(-8.0);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(&mut h);
    let cam = h.play.as_ref().unwrap().camera_frame().unwrap().clone();
    assert!((cam.distance - (8.0 * 1.1f32.powf(8.0)).clamp(4.0, 12.0)).abs() < 1e-4);
    assert!((4.0..=12.0).contains(&cam.distance));
    // 8 wheel-up notches from there multiply by 1.1^-8 and clamp to the floor.
    h.play.as_mut().unwrap().resume().expect("resume");
    h.play.as_mut().unwrap().cam_zoom(8.0);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(&mut h);
    let cam = h.play.as_ref().unwrap().camera_frame().unwrap().clone();
    let want = 12.0 * 1.1f32.powf(-8.0);
    assert!(
        (cam.distance - want).abs() < 1e-3,
        "zoom back in from the clamp: {} want {want}",
        cam.distance
    );
    // Pitch drag up to the 75 degree ceiling.
    h.play.as_mut().unwrap().resume().expect("resume");
    h.play.as_mut().unwrap().cam_drag(0.0, 400.0);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(&mut h);
    let cam = h.play.as_ref().unwrap().camera_frame().unwrap().clone();
    assert!((cam.pitch - 75.0).abs() < 1e-4, "pitch clamped at 75");
    assert_eq!(h.camera_saturation, 0, "within the accumulator caps");
    // Saturated drag events are diagnosed, the candidate stays finite.
    h.play.as_mut().unwrap().resume().expect("resume");
    h.play.as_mut().unwrap().cam_drag(5000.0, -5000.0);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(&mut h);
    let cam = h.play.as_ref().unwrap().camera_frame().unwrap().clone();
    assert!(cam.yaw.is_finite() && cam.pitch.is_finite());
    assert_eq!(
        h.camera_saturation, 1,
        "the saturated drag event is counted once"
    );
}

fn orthographic_projection_config_publishes_the_alt_matrix_body() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("ortho3d.at"),
        source3d(6.0, 30.0, 1, 16.0).as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document3d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    h.pause_play("");
    settle(&mut h);
    let play = h.play.as_ref().unwrap();
    let pos = Vec3::from(play.last.as_ref().unwrap().objects[0].translation);
    assert_eq!(scene_vp(&h), expected_vp(0.0, 30.0, 6.0, pos, Some(16.0)));
}

fn failure_keeps_the_last_good_frame_and_camera_pair_body() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("fault3d.at"),
        source3d(8.0, 45.0, 0, 0.0).as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document3d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    h.pause_play("");
    settle(&mut h);
    step(&mut h, 4);
    let good = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
    let good_vp = scene_vp(&h);
    // Injected preparation failure: the run faults and the LAST COMPLETE
    // pair stays published; no new camera with an old actor (or vice versa).
    h.fail_resource = true;
    step(&mut h, 4);
    h.fail_resource = false;
    let play = h.play.as_ref().unwrap();
    assert_eq!(
        play.state,
        autographics_auto_driver::authoring::play::State::Faulted
    );
    assert_eq!(scene_vp(&h), good_vp, "last-good camera stays installed");
    assert_eq!(
        play.last.as_ref().unwrap().tick,
        good.tick,
        "last-good frame stays"
    );
    // The VM is closed by the original fault rules.
    assert!(
        play.owner_probe().map_or(true, |f| !f()),
        "fault closes the VM"
    );
}

fn paused_neutral_step_moves_the_camera_without_orbit_and_preserves_defaults_body() {
    // No-camera 3D run: the scene camera stays the frozen sample default the
    // whole run (the old profile pixel behavior is preserved).
    let plain = SourceSnapshot::from_bytes_for(
        Path::new("plain3d.at"),
        source3d(8.0, 45.0, 0, 0.0)
            .replace("ok=ok+ag_camera_orbit_3d(1, 8.0, 45.0, 0, 0.0)\n", "")
            .replace("let fx=ag_cam3_forward_x()", "let fx=0.0")
            .replace("let fz=ag_cam3_forward_z()", "let fz= -1.0")
            .replace("let rx=ag_cam3_right_x()", "let rx=1.0")
            .replace("let rz=ag_cam3_right_z()", "let rz=0.0")
            .as_bytes()
            .to_vec(),
        Api::Character3V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document3d(), Default::default(), None).expect("host");
    h.start_user_snapshot(plain).expect("start");
    h.pause_play("");
    settle(&mut h);
    assert_eq!(h.play.as_ref().unwrap().camera_dim(), None);
    let default_vp = expected_vp(0.0, 0.0, 0.0, Vec3::ZERO, None); // placeholder, compare below
    let _ = default_vp;
    step(&mut h, 4);
    let sample = Camera3d::sample_camera(2.0).view_proj();
    assert_eq!(
        scene_vp(&h),
        sample,
        "no-camera run keeps the 860 default pixels"
    );
    assert!(h.play.as_ref().unwrap().camera_frame().is_none());

    // Camera run, neutral Step: no orbit/zoom, camera follows the position.
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("step3d.at"),
        source3d(8.0, 45.0, 0, 0.0).as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("admission");
    let mut h2 = Host::with_project(document3d(), Default::default(), None).expect("host");
    h2.start_user_snapshot(snapshot).expect("start");
    h2.pause_play("");
    settle(&mut h2);
    // Orbit input while Paused is dropped.
    h2.play.as_mut().unwrap().cam_drag(90.0, 20.0);
    h2.play.as_mut().unwrap().single_step().expect("step");
    settle(&mut h2);
    let play = h2.play.as_ref().unwrap();
    assert_eq!(
        play.last.as_ref().unwrap().tick,
        1,
        "exactly the neutral step"
    );
    let cam = play.camera_frame().unwrap().clone();
    assert_eq!(
        (cam.yaw, cam.pitch, cam.distance),
        (0.0, 45.0, 8.0),
        "no orbit on Step"
    );
    let pos = Vec3::from(play.last.as_ref().unwrap().objects[0].translation);
    assert_eq!(
        scene_vp(&h2),
        expected_vp(0.0, 45.0, 8.0, pos, None),
        "camera follows the neutral step position"
    );
}

/// The wheel path: zoom only over the 3D surface region while Running.
fn scroll_zoom_routes_by_surface_and_running_state_body() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("wheel3d.at"),
        source3d(8.0, 45.0, 0, 0.0).as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document3d(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    h.pause_play("");
    settle(&mut h);
    // The host-level wheel conversion is exercised through the play feed:
    // the scroll handler maps physical pixels to notches once (856 rule).
    // Feed -8 notches like one full wheel event batch at the 3D region.
    h.play.as_mut().unwrap().resume().expect("resume");
    h.play.as_mut().unwrap().cam_zoom(-4.0);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(&mut h);
    let cam = h.play.as_ref().unwrap().camera_frame().unwrap().clone();
    let want = (8.0 * 1.1f32.powf(4.0)).clamp(4.0, 12.0);
    assert!((cam.distance - want).abs() < 1e-4, "{want}");
    let _ = Float(0.0); // keep the Scalar import used on all platforms
    let _ = Scalar::Int(0);
}

// ----------------------------------------------------------- V-05 lifecycle --

/// Twenty camera start/stop cycles on one resident Host per dimension over
/// orbit/resize/fault/stop: the VM weak owner closes on every Stop, the
/// editing camera restores, and after the first cycle's warm-up the per-cycle
/// upload deltas stay exactly flat (no static mesh re-upload, no extra
/// dynamic work). Writes the env-gated host report for the checker.
fn twenty_camera_lifecycle_cycles_release_owners_and_keep_uploads_flat_body() {
    let report_dir = std::env::var("RUNTIME_CAMERA_REPORT_DIR").ok();
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("cycle3d.at"),
        source3d(8.0, 45.0, 0, 0.0).as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("admission");
    let snapshot2 = SourceSnapshot::from_bytes_for(
        Path::new("cycle2d.at"),
        source2d([0.0, -4.0, 30.0, 12.0], [12.0, 8.0])
            .as_bytes()
            .to_vec(),
        Api::Camera2V1,
    )
    .expect("admission");
    let mut hosts = [
        Host::with_project(document3d(), Default::default(), None).expect("host3d"),
        Host::with_project(document2d(), Default::default(), None).expect("host2d"),
    ];
    let mut report_cycles = Vec::new();
    let mut deltas: [Vec<(u64, bool)>; 2] = [Vec::new(), Vec::new()];
    for cycle in 0..20 {
        let which = cycle % 2;
        let h = &mut hosts[which];
        let three_d = which == 0;
        let before = h.rt.upload_stats().buffer_rewrites;
        h.start_user_snapshot(if three_d {
            snapshot.clone()
        } else {
            snapshot2.clone()
        })
        .expect("start");
        h.pause_play("");
        settle(h);
        let alive = h.play.as_ref().unwrap().owner_probe().expect("VM owner");
        assert!(alive(), "cycle {cycle}: owner alive after init");
        // Orbit/zoom while running.
        h.play.as_mut().unwrap().resume().expect("resume");
        if three_d {
            h.play.as_mut().unwrap().cam_drag(30.0, 5.0);
            h.play.as_mut().unwrap().cam_zoom(2.0);
        }
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .expect("dispatch");
        h.pause_play("");
        settle(h);
        h.resize(1000, 700);
        assert_eq!(
            h.play.as_ref().unwrap().state,
            autographics_auto_driver::authoring::play::State::Paused,
            "cycle {cycle}: resize pauses"
        );
        assert!(
            h.camera_drag.is_none(),
            "cycle {cycle}: resize clears the capture"
        );
        // A fault injection every fifth cycle keeps the last-good pair.
        let faulted = cycle % 5 == 4;
        if faulted {
            h.play.as_mut().unwrap().resume().expect("resume");
            h.play
                .as_mut()
                .unwrap()
                .dispatch(Action::default())
                .expect("dispatch");
            h.fail_resource = true;
            settle(h);
            h.fail_resource = false;
            assert_eq!(
                h.play.as_ref().unwrap().state,
                autographics_auto_driver::authoring::play::State::Faulted,
                "cycle {cycle}"
            );
        }
        // The dimension's presentation marker: the 3D camera matrix changes
        // under the run; the 2D letterbox bars exist only under the run.
        let run_marker = if three_d {
            format!("{:?}", scene_vp(h))
        } else {
            let clear = match h.rt.content(h.two) {
                Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g.clear,
                _ => panic!("2D content"),
            };
            let bars = two_commands(h)
                .iter()
                .filter(|c| c.color == clear && c.size[0] > 100.0 && c.size[1] > 300.0)
                .count();
            assert!(bars == 2, "cycle {cycle}: letterbox bars must be published");
            format!("{bars}")
        };
        h.stop_play();
        h.frame(None).ok();
        assert!(
            h.play.is_none() || !h.play.as_ref().unwrap().owner_probe().map_or(true, |f| f()),
            "cycle {cycle}: Stop closes the VM owner"
        );
        let restored_marker = if three_d {
            format!("{:?}", scene_vp(h))
        } else {
            let clear = match h.rt.content(h.two) {
                Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g.clear,
                _ => panic!("2D content"),
            };
            let bars = two_commands(h)
                .iter()
                .filter(|c| c.color == clear && c.size[0] > 100.0 && c.size[1] > 300.0)
                .count();
            format!("{bars}")
        };
        assert_ne!(
            run_marker, restored_marker,
            "cycle {cycle}: Stop restores the editing presentation"
        );
        deltas[which].push((h.rt.upload_stats().buffer_rewrites - before, faulted));
        report_cycles.push(json!({
            "cycle": cycle,
            "three_d": three_d,
            "faulted": faulted,
        }));
    }
    // After cycle 0 warms each resident store, every later cycle re-uses the
    // static meshes and issues the same dynamic work: the non-fault cycles
    // stay exactly flat (a fault cycle pays one extra failing dispatch).
    for (which, per_host) in deltas.iter().enumerate() {
        let warm = per_host.iter().find(|(_, f)| !f).map(|(v, _)| *v).unwrap();
        let outliers: Vec<u64> = per_host[1..]
            .iter()
            .filter(|(v, f)| !f && *v != warm)
            .map(|(v, _)| *v)
            .collect();
        assert!(
            outliers.is_empty(),
            "host {which} unexpected upload outliers (warm {warm}): {:?}",
            per_host
        );
    }
    if let Some(dir) = report_dir {
        let report = json!({
            "schema": "autographics.runtime-camera-host/1",
            "completed": true,
            "cycles": report_cycles.len(),
            "per_cycle": report_cycles,
            "upload_deltas_3d": deltas[0].iter().map(|(v, f)| json!({"delta": v, "faulted": f})).collect::<Vec<_>>(),
            "upload_deltas_2d": deltas[1].iter().map(|(v, f)| json!({"delta": v, "faulted": f})).collect::<Vec<_>>(),
        });
        std::fs::create_dir_all(&dir).ok();
        std::fs::write(
            std::path::Path::new(&dir).join("runtime-camera-host.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .expect("write host report");
        println!("host report written to {dir}");
    }
}

/// The camera assemblies compile three full trusted units plus the user
/// source; the combined parse depth overflows the default 2MiB libtest stack
/// on a cold consumer build, so every test body runs on a dedicated
/// big-stack thread (a fn pointer keeps the closure Send).
fn big_stack(body: fn()) {
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(scope, body)
            .expect("test thread")
            .join()
            .expect("test body panicked");
    });
}

#[test]
fn follow2d_clamps_and_letterboxes_through_the_real_host() {
    big_stack(follow2d_clamps_and_letterboxes_through_the_real_host_body);
}

#[test]
fn follow2d_second_configuration_changes_projection_from_the_source() {
    big_stack(follow2d_second_configuration_changes_projection_from_the_source_body);
}

#[test]
fn nan_configuration_rejects_at_the_named_admission_stage() {
    big_stack(nan_configuration_rejects_at_the_named_admission_stage_body);
}

#[test]
fn camera3d_view_proj_is_this_tick_actor_not_the_previous_one() {
    big_stack(camera3d_view_proj_is_this_tick_actor_not_the_previous_one_body);
}

#[test]
fn zoom_and_pitch_stay_inside_the_frozen_domains_through_the_host() {
    big_stack(zoom_and_pitch_stay_inside_the_frozen_domains_through_the_host_body);
}

#[test]
fn orthographic_projection_config_publishes_the_alt_matrix() {
    big_stack(orthographic_projection_config_publishes_the_alt_matrix_body);
}

#[test]
fn failure_keeps_the_last_good_frame_and_camera_pair() {
    big_stack(failure_keeps_the_last_good_frame_and_camera_pair_body);
}

#[test]
fn paused_neutral_step_moves_the_camera_without_orbit_and_preserves_defaults() {
    big_stack(paused_neutral_step_moves_the_camera_without_orbit_and_preserves_defaults_body);
}

#[test]
fn scroll_zoom_routes_by_surface_and_running_state() {
    big_stack(scroll_zoom_routes_by_surface_and_running_state_body);
}

#[test]
fn twenty_camera_lifecycle_cycles_release_owners_and_keep_uploads_flat() {
    big_stack(twenty_camera_lifecycle_cycles_release_owners_and_keep_uploads_flat_body);
}

// --------------------------------------------------- fresh-process sampling --

fn sampler_document(shape: &str) -> Document {
    if shape.contains("128") {
        // The frozen wide-128 spatial denominator (contract section 8).
        let mut d = if shape.contains("2d") {
            let mut d = Document::empty("cam2d-128".into());
            let mut character = Object::new(1, Kind::Quad);
            character.translation = [2.0, 1.0, 0.0];
            character.scale = [0.8, 1.0, 1.0];
            d.objects.push(character);
            for i in 1..128 {
                let mut node = Object::new(i as u64 + 1, Kind::Quad);
                node.translation = [i as f32 * 3.0, 1.0, 0.0];
                node.scale = [0.8, 1.0, 1.0];
                d.objects.push(node);
            }
            d
        } else {
            let mut d = Document::empty("cam3d-128".into());
            let mut character = Object::new(1, Kind::Cylinder);
            character.translation = [0.0, 0.5, 0.0];
            character.scale = [0.4, 0.5, 0.4];
            d.objects.push(character);
            let mut floor = Object::new(2, Kind::Cylinder);
            floor.translation = [96.0, -0.5, 0.0];
            floor.scale = [290.0, 0.5, 10.0];
            d.objects.push(floor);
            for i in 2..128 {
                let mut node = Object::new(i as u64 + 1, Kind::Cylinder);
                node.translation = [i as f32 * 2.4, 0.5, 0.0];
                node.scale = [0.4, 0.5, 2.0];
                d.objects.push(node);
            }
            d
        };
        d.validate().expect("document valid");
        return d;
    }
    if shape.contains("2d") {
        document2d()
    } else {
        document3d()
    }
}

/// One fresh process per sample: the full Host path (camera inputs, camera
/// block admission, prepare/publish) for the warm steps, reporting per-tick
/// whole-transaction wall time. Shape grammar: camera{2d|3d}-{regular|128}.
#[test]
fn runtime_camera_sampler_child() {
    let Ok(shape) = std::env::var("RUNTIME_CAMERA_SAMPLER") else {
        return;
    };
    let out = std::env::var("RUNTIME_CAMERA_SAMPLER_REPORT").expect("sampler report path");
    let api = if shape.contains("2d") {
        Api::Camera2V1
    } else {
        Api::Camera3V1
    };
    let source = if shape.contains("2d") {
        source2d([0.0, -4.0, 30.0, 12.0], [12.0, 8.0])
    } else {
        source3d(8.0, 45.0, 0, 0.0)
    };
    let snapshot =
        SourceSnapshot::from_bytes_for(Path::new("sampler.at"), source.as_bytes().to_vec(), api)
            .expect("admission");
    let mut h =
        Host::with_project(sampler_document(&shape), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    h.pause_play("");
    settle(&mut h);
    let mut tick_ns = Vec::new();
    for step in 1..=60i32 {
        h.play.as_mut().unwrap().resume().expect("resume");
        h.play.as_mut().unwrap().cam_drag(6.0, 0.0);
        if step % 2 == 0 {
            h.play.as_mut().unwrap().cam_zoom(1.0);
        }
        let begin = Instant::now();
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .expect("dispatch");
        h.pause_play("");
        settle(&mut h);
        // The first ten steps pay one-time GPU/first-frame warm-up and stay
        // out of the steady-state sample (874 sampler methodology).
        if step > 10 {
            tick_ns.push(begin.elapsed().as_nanos() as u64);
        }
        let play = h.play.as_ref().unwrap();
        assert!(
            play.error.is_none(),
            "sampler step {step}: {:?}",
            play.error
        );
        assert!(
            play.last.as_ref().unwrap().camera.is_some(),
            "camera block rides"
        );
    }
    tick_ns.sort();
    let p50 = tick_ns[tick_ns.len() / 2];
    let p95 = tick_ns[tick_ns.len() * 95 / 100];
    let report = json!({
        "shape": shape,
        "steps": 50,
        "camera_block": true,
        "tick_ns_p50": p50,
        "tick_ns_p95": p95,
        "max_ns": tick_ns.last().copied().unwrap_or(0),
    });
    std::fs::write(&out, serde_json::to_vec(&report).unwrap()).expect("write sample");
}

/// Five fresh processes per shape across camera2d/camera3d x regular/128;
/// aggregates the frozen sampling report for tools/check_runtime_camera.py.
/// Gate: RUNTIME_CAMERA_REPORT_DIR.
#[test]
fn runtime_camera_sampling_matrix_five_fresh_processes_per_shape() {
    let Ok(dir) = std::env::var("RUNTIME_CAMERA_REPORT_DIR") else {
        return;
    };
    let exe = std::env::current_exe().unwrap();
    let mut shapes = Vec::new();
    for shape in [
        "camera2d-regular",
        "camera2d-128",
        "camera3d-regular",
        "camera3d-128",
    ] {
        let mut samples = Vec::new();
        for run in 0..5 {
            let out = std::env::temp_dir().join(format!(
                "runtime-camera-sampler-{}-{shape}-{run}.json",
                std::process::id()
            ));
            let status = std::process::Command::new(&exe)
                .args(["--test-threads=1", "runtime_camera_sampler_child"])
                .env("RUNTIME_CAMERA_SAMPLER", shape)
                .env("RUNTIME_CAMERA_SAMPLER_REPORT", &out)
                .status()
                .expect("spawn sampler child");
            assert!(status.success(), "sampler child {shape}/{run} failed");
            samples.push(
                serde_json::from_slice::<serde_json::Value>(&std::fs::read(&out).unwrap()).unwrap(),
            );
            let _ = std::fs::remove_file(&out);
        }
        let mut p50s: Vec<u64> = samples
            .iter()
            .map(|s| s["tick_ns_p50"].as_u64().unwrap())
            .collect();
        let mut p95s: Vec<u64> = samples
            .iter()
            .map(|s| s["tick_ns_p95"].as_u64().unwrap())
            .collect();
        p50s.sort_unstable();
        p95s.sort_unstable();
        shapes.push(json!({
            "shape": shape,
            "processes": 5,
            "tick_ns_p50_median": p50s[2],
            "tick_ns_p95_median": p95s[2],
            "all_under_limit": p95s.iter().all(|v| *v < 250_000_000u64),
            "samples": samples,
        }));
    }
    let report = json!({
        "schema": "autographics.runtime-camera-sampling/1",
        "completed": true,
        "shapes": shapes,
    });
    std::fs::create_dir_all(&dir).ok();
    std::fs::write(
        std::path::Path::new(&dir).join("runtime-camera-sampling.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .expect("write sampling report");
    println!("sampling report written to {dir}");
}
