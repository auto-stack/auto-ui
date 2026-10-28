//! PLAN-875 runtime camera public API tests (contract V-01): the two new
//! dimension-specific identities, legal configuration admission, wrong
//! dimension/target/NaN/degenerate/duplicate rejections with named fault
//! codes, the host mirror reads, and the frozen free-reading credit rule.
//! Real Auto Session execution; run single-threaded:
//! cargo test --locked --test runtime_camera_api -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const CLEARANCE: f32 = 0.002;
const DT: f32 = 1.0 / 60.0;

struct Row(i32, i32, [f32; 7]);

fn row(kind: i32, dim: i32, lo: [f32; 3], hi: [f32; 3]) -> Row {
    Row(kind, dim, [1., lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]])
}

fn scene_rows() -> Vec<Row> {
    vec![
        row(2, 3, [-0.4, -0.5, -0.4], [0.4, 0.5, 0.4]), // id 1: character
        row(2, 3, [-8., -1., -8.], [8., 0., 8.]),       // id 2: floor
        row(2, 3, [3., 0., -1.], [3.2, 3., 1.]),        // id 3: wall
        row(2, 2, [9., 0., 0.], [10., 1., 0.]),         // id 4: 2D decoy
    ]
}

fn scene_rows_2d() -> Vec<Row> {
    vec![
        row(2, 2, [-0.4, -0.5, 0.], [0.4, 0.5, 0.]), // id 1: character
        row(2, 2, [-4., -1., 0.], [34., 0., 0.]),    // id 2: floor
        row(2, 2, [-1., 0., 0.], [0., 3., 0.]),      // id 3: west wall
        row(2, 3, [20., -1., 20.], [21., 0., 21.]),  // id 4: 3D decoy
    ]
}

const PRELUDE: &str = "var ok=0\nvar stepn=0\nvar fx=0.0\nvar fz= -1.0\nvar rx=1.0\nvar rz=0.0\nvar yw=0.0\nvar pt=0.0\nvar ds=0.0\n";
const GETTERS: &str = "fn fx_v() float {return fx}\nfn fz_v() float {return fz}\nfn rx_v() float {return rx}\nfn rz_v() float {return rz}\nfn yw_v() float {return yw}\nfn pt_v() float {return pt}\nfn ds_v() float {return ds}\n";

fn source(body: &str) -> String {
    format!("{PRELUDE}{body}{GETTERS}")
}

fn upd(start: &str, update: &str) -> String {
    source(&format!(
        "fn app_start() {{\n{start}}}\nfn app_update(dt float) {{\nstepn=stepn+1\n{update}}}\n"
    ))
}

fn map_call(api: Api) -> &'static str {
    match api {
        Api::Camera2V1 => "_ag_char_map",
        _ => "_ag_char3_map",
    }
}

/// One camera-run session: transport reset, spatial rows, stable-id map,
/// then `_ag_start` (app_start + character freeze + camera freeze).
fn camera_session(api: Api, user: &str, name: &str, rows: &[Row]) -> Result<Session, String> {
    let snap = SourceSnapshot::from_bytes_for(Path::new(name), user.as_bytes().to_vec(), api)?;
    let mut s = Session::new(&snap.source)?;
    s.call("main", &[], ResultKind::Unit)?;
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)?;
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )?;
    for (n, Row(kind, dim, box7)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
            ResultKind::Unit,
        )?;
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)?;
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)?;
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)?;
        }
        for (i, v) in box7.iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )?;
        }
    }
    for n in 0..rows.len() {
        s.call(
            map_call(api),
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )?;
    }
    s.call("_ag_start", &[], ResultKind::Unit)?;
    Ok(s)
}

fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}

fn fault_node(s: &mut Session) -> i32 {
    s.int("_ag_error_node", &[]).unwrap()
}

fn fault_arg(s: &mut Session) -> i32 {
    s.int("_ag_error_argument", &[]).unwrap()
}

fn tick(s: &mut Session, step: i32) {
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

const BIND3: &str = " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_char3_solid(3)\n";
const BIND2: &str = " ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_char_solid(3)\n";
const CAM3: &str = " ok=ok+ag_camera_orbit_3d(1,8.0,45.0,0,0.0)\n";
const CAM2: &str = " ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n";
const IDLE3: &str = "let q=ag_char3_move(0.0,0.0,0.0,0.002)\nok=ok+ag_char3_grounded(q)\n";
const IDLE2: &str = "let q=ag_char_move(0.0,0.0,0.002)\nok=ok+ag_char_grounded(q)\n";

// ------------------------------------------------- assembly/identity (V-01) --

fn camera_identities_parse_with_frozen_numbers_and_profiles_body() {
    for (api, version, number, profile) in [
        (Api::Camera2V1, "autographics.runtime_camera2d/1", 103, 14),
        (Api::Camera3V1, "autographics.runtime_camera3d/1", 104, 15),
    ] {
        assert_eq!(Api::parse(version).unwrap(), api);
        assert_eq!(api.number(), number, "{version}");
        assert_eq!(api.profile(), profile, "{version}");
        assert_eq!(api.version(), version);
    }
    // Old identities stay exactly where 874 froze them.
    for (api, number, profile) in [
        (Api::V1, 1, 7),
        (Api::SpatialV2, 2, 8),
        (Api::SegmentV3, 3, 9),
        (Api::SweepV4, 4, 10),
        (Api::MotionV5, 5, 11),
        (Api::CharacterV1, 101, 12),
        (Api::Character3V1, 102, 13),
    ] {
        assert_eq!(api.number(), number);
        assert_eq!(api.profile(), profile);
    }
    // Camera freeze hooks ride the entry after the character hooks.
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("identity.at"),
        source("fn app_start() {}\nfn app_update(dt float) {}\n")
            .as_bytes()
            .to_vec(),
        Api::Camera3V1,
    )
    .unwrap();
    assert!(
        snap.source
            .contains("app_start(); _ag_cast_token=0; _ag_char3_freeze(); _ag_cam3_freeze()"),
        "3D entry must chain the camera freeze"
    );
    assert!(
        snap.source
            .contains("_ag_char3_tick_begin(); app_update(dt); _ag_cast_token=0"),
        "tick begin chain must stay"
    );
    let regions: Vec<_> = snap.regions.iter().map(|r| r.file.as_str()).collect();
    assert!(regions.contains(&"<runtime_camera3d_v1.at>"), "{regions:?}");
    let snap2 = SourceSnapshot::from_bytes_for(
        Path::new("identity2.at"),
        source("fn app_start() {}\nfn app_update(dt float) {}\n")
            .as_bytes()
            .to_vec(),
        Api::Camera2V1,
    )
    .unwrap();
    let regions: Vec<_> = snap2.regions.iter().map(|r| r.file.as_str()).collect();
    assert!(regions.contains(&"<runtime_camera2d_v1.at>"), "{regions:?}");
}

fn sample_sources_admit_on_their_own_dimension_body() {
    let two = std::fs::read_to_string(Path::new("samples/runtime-camera/two/main.at")).unwrap();
    SourceSnapshot::from_bytes_for(Path::new("two.at"), two.as_bytes().to_vec(), Api::Camera2V1)
        .expect("2D sample admits");
    let three = std::fs::read_to_string(Path::new("samples/runtime-camera/three/main.at")).unwrap();
    SourceSnapshot::from_bytes_for(
        Path::new("three.at"),
        three.as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("3D sample admits");
    let three_alt =
        std::fs::read_to_string(Path::new("samples/runtime-camera/three/main-alt.at")).unwrap();
    SourceSnapshot::from_bytes_for(
        Path::new("three-alt.at"),
        three_alt.as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("3D alt sample admits");
    let two_alt =
        std::fs::read_to_string(Path::new("samples/runtime-camera/two/main-alt.at")).unwrap();
    SourceSnapshot::from_bytes_for(
        Path::new("two-alt.at"),
        two_alt.as_bytes().to_vec(),
        Api::Camera2V1,
    )
    .expect("2D alt sample admits");
}

fn camera_functions_of_the_other_dimension_are_not_admitted_body() {
    // The 3D camera call in the 2D assembly: no such root symbol exists, so
    // the compile of the admitted combined source must fail.
    let wrong = upd(&format!("{BIND2}{CAM3}"), IDLE2);
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("wrong2.at"),
        wrong.as_bytes().to_vec(),
        Api::Camera2V1,
    )
    .expect("admission is name-agnostic; the compiler catches it");
    let e = Session::new(&snap.source).err().expect("compile must fail");
    assert!(e.contains("Undefined") || e.contains("undefined"), "{e}");
    // And the 2D camera call in the 3D assembly.
    let wrong = upd(&format!("{BIND3}{CAM2}"), IDLE3);
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("wrong3.at"),
        wrong.as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .expect("admission is name-agnostic; the compiler catches it");
    let e = Session::new(&snap.source).err().expect("compile must fail");
    assert!(e.contains("Undefined") || e.contains("undefined"), "{e}");
    // The private transport calls stay unwritable by user sources.
    let hostile = upd(
        &format!("{BIND3}{CAM3}"),
        "let q=1\n_ag_cam3_orbit(0.0,45.0,8.0)\n",
    );
    let e = SourceSnapshot::from_bytes_for(
        Path::new("hostile.at"),
        hostile.as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .unwrap_err();
    assert!(
        e.contains("reserved") || e.contains("parse") || e.contains("Undefined"),
        "{e}"
    );
}

// ---------------------------------------------- configuration admission (V-01) --

fn orbit_config_domains_reject_with_frozen_fault_codes_body() {
    for (bad, arg_want) in [
        ("ok=ok+ag_camera_orbit_3d(1,3.9,45.0,0,0.0)\n", 0),
        ("ok=ok+ag_camera_orbit_3d(1,12.1,45.0,0,0.0)\n", 0),
        ("ok=ok+ag_camera_orbit_3d(1,8.0,14.9,0,0.0)\n", 0),
        ("ok=ok+ag_camera_orbit_3d(1,8.0,75.1,0,0.0)\n", 0),
        ("ok=ok+ag_camera_orbit_3d(1,8.0,45.0,2,0.0)\n", 2),
        ("ok=ok+ag_camera_orbit_3d(1,8.0,45.0,1, -1.0)\n", 0),
        ("ok=ok+ag_camera_orbit_3d(1,8.0,45.0,1,0.5)\n", 0),
        ("ok=ok+ag_camera_orbit_3d(1,8.0,45.0,1,64.1)\n", 0),
    ] {
        let user = upd(&format!("{BIND3}{bad}"), IDLE3);
        let mut s = camera_session(Api::Camera3V1, &user, "dom.at", &scene_rows())
            .expect("session builds (faults surface at start)");
        assert_eq!(status(&mut s), 27, "config {bad}");
        assert_eq!(fault_arg(&mut s), arg_want, "arg for {bad}");
    }
}

fn follow2d_config_domains_reject_with_frozen_fault_codes_body() {
    for bad in [
        "ok=ok+ag_camera_follow_2d(1, 4.0, -4.0, 2.0, 12.0, 12.0, 8.0)\n", // min>=max
        "ok=ok+ag_camera_follow_2d(1, 0.0, 12.0, 30.0, 2.0, 12.0, 8.0)\n", // min>=max y
        "ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 0.0, 8.0)\n", // view<=0
        "ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 0.0)\n",
        "ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 30.1, 8.0)\n", // wider than rect
        "ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 16.1)\n", // taller than rect
    ] {
        let user = upd(&format!("{BIND2}{bad}"), IDLE2);
        let mut s = camera_session(Api::Camera2V1, &user, "dom2.at", &scene_rows_2d())
            .expect("session builds");
        assert_eq!(status(&mut s), 27, "config {bad}");
    }
}

fn camera_target_must_be_this_run_bound_character_body() {
    // Unmapped stable id.
    let user = upd(
        &format!("{BIND3}ok=ok+ag_camera_orbit_3d(99,8.0,45.0,0,0.0)\n"),
        IDLE3,
    );
    let mut s = camera_session(Api::Camera3V1, &user, "t99.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 30);
    assert_eq!(fault_arg(&mut s), 99);
    // A mapped but non-character node (the floor, run index 1).
    let user = upd(
        &format!("{BIND3}ok=ok+ag_camera_orbit_3d(2,8.0,45.0,0,0.0)\n"),
        IDLE3,
    );
    let mut s = camera_session(Api::Camera3V1, &user, "t2.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 30);
    assert_eq!(fault_node(&mut s), 1, "the attempted target node");
    // 2D: unknown and non-character targets.
    let user = upd(
        &format!("{BIND2}ok=ok+ag_camera_follow_2d(99, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n"),
        IDLE2,
    );
    let mut s = camera_session(Api::Camera2V1, &user, "t99b.at", &scene_rows_2d()).unwrap();
    assert_eq!(status(&mut s), 30);
    assert_eq!(fault_arg(&mut s), 99);
}

fn duplicate_and_post_freeze_configuration_reject_body() {
    // Two configs in app_start: the second refuses with 29.
    let user = upd(&format!("{BIND3}{CAM3}{CAM3}"), IDLE3);
    let mut s = camera_session(Api::Camera3V1, &user, "dup.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 29);
    // Config inside app_update is after the freeze: 29.
    let user = upd(
        &format!("{BIND3}{CAM3}"),
        "let q=ag_char3_move(0.0,0.0,0.0,0.002)\nlet c=ag_camera_orbit_3d(1,8.0,45.0,0,0.0)\n",
    );
    let mut s = camera_session(Api::Camera3V1, &user, "post.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0, "start is clean");
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 29, "post-freeze configuration refuses");
}

fn missing_configuration_rejects_at_the_start_boundary_body() {
    let user = upd(BIND3, IDLE3);
    let mut s = camera_session(Api::Camera3V1, &user, "missing.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 27, "camera configuration missing");
    let user = upd(BIND2, IDLE2);
    let mut s = camera_session(Api::Camera2V1, &user, "missing2.at", &scene_rows_2d()).unwrap();
    assert_eq!(status(&mut s), 27, "2D camera configuration missing");
}

fn camera_public_calls_never_bypass_the_host_transport_body() {
    // The camera config functions take five arguments, beyond the host
    // Session::start profile cap of four: they are only reachable through
    // compiled user sources, never as direct transport calls. Nonfinite
    // literals in user sources are refused by the frame admission instead
    // (runtime_camera_host).
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("nan.at"),
        upd(&format!("{BIND3}{CAM3}"), IDLE3).as_bytes().to_vec(),
        Api::Camera3V1,
    )
    .unwrap();
    let mut s = Session::new(&snap.source).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    let e = s
        .call(
            "ag_camera_orbit_3d",
            &[Int(1), Float(8.0), Float(45.0), Int(0), Float(0.0)],
            ResultKind::Int,
        )
        .unwrap_err();
    assert!(e.contains("argument count exceeds profile"), "{e}");
}

// ----------------------------------------------- mirror reads and queries (V-01) --

fn host_sent_basis_and_orbit_mirror_exactly_and_persist_body() {
    // The public reads are captured into user globals by app_update on every
    // tick, so the getters observe the mirror through the real call path.
    let capture = "fx=ag_cam3_forward_x()\nfz=ag_cam3_forward_z()\nrx=ag_cam3_right_x()\nrz=ag_cam3_right_z()\nyw=ag_cam3_yaw()\npt=ag_cam3_pitch()\nds=ag_cam3_distance()\nlet q=ag_char3_move(0.0,0.0,0.0,0.002)\nok=ok+ag_char3_grounded(q)\n";
    let user = upd(&format!("{BIND3}{CAM3}"), capture);
    let mut s = camera_session(Api::Camera3V1, &user, "mirror.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0);
    // Yaw 0 mirrors the frozen default axes (captured at tick 1).
    s.call("_ag_char3_input", &[Int(0), Int(0)], ResultKind::Unit)
        .unwrap();
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0);
    assert_eq!(s.float("fx_v", &[]).unwrap(), 0.0);
    assert_eq!(s.float("fz_v", &[]).unwrap(), -1.0);
    assert_eq!(s.float("rx_v", &[]).unwrap(), 1.0);
    assert_eq!(s.float("rz_v", &[]).unwrap(), 0.0);
    assert_eq!(s.float("yw_v", &[]).unwrap(), 0.0);
    assert_eq!(s.float("pt_v", &[]).unwrap(), 45.0);
    assert_eq!(s.float("ds_v", &[]).unwrap(), 8.0);
    // The host candidate: yaw 90, pitch 30, distance 6.
    s.call(
        "_ag_cam3_basis",
        &[Float(-1.0), Float(0.0), Float(0.0), Float(-1.0)],
        ResultKind::Unit,
    )
    .unwrap();
    s.call(
        "_ag_cam3_orbit",
        &[Float(90.0), Float(30.0), Float(6.0)],
        ResultKind::Unit,
    )
    .unwrap();
    s.call("_ag_char3_input", &[Int(0), Int(0)], ResultKind::Unit)
        .unwrap();
    tick(&mut s, 2);
    assert_eq!(status(&mut s), 0);
    assert_eq!(s.float("fx_v", &[]).unwrap(), -1.0);
    assert_eq!(s.float("fz_v", &[]).unwrap(), 0.0);
    assert_eq!(s.float("rx_v", &[]).unwrap(), 0.0);
    assert_eq!(s.float("rz_v", &[]).unwrap(), -1.0);
    assert_eq!(s.float("yw_v", &[]).unwrap(), 90.0);
    assert_eq!(s.float("pt_v", &[]).unwrap(), 30.0);
    assert_eq!(s.float("ds_v", &[]).unwrap(), 6.0);
    // The next tick with no new input keeps the last candidate (mirrors).
    s.call("_ag_char3_input", &[Int(0), Int(0)], ResultKind::Unit)
        .unwrap();
    tick(&mut s, 3);
    assert_eq!(s.float("yw_v", &[]).unwrap(), 90.0);
    assert_eq!(s.float("ds_v", &[]).unwrap(), 6.0);
    // Camera block queries report the run/tick identity of this Run.
    assert_eq!(s.int("_ag_cam3_run", &[]).unwrap(), 1);
    assert_eq!(s.int("_ag_cam3_tick", &[]).unwrap(), 3);
    assert_eq!(s.int("_ag_cam3_status", &[]).unwrap(), 1);
    assert_eq!(s.int("_ag_cam3_proj_v", &[]).unwrap(), 0);
}

fn follow2d_block_queries_echo_the_frozen_configuration_body() {
    let user = upd(&format!("{BIND2}{CAM2}"), IDLE2);
    let mut s = camera_session(Api::Camera2V1, &user, "echo2.at", &scene_rows_2d()).unwrap();
    assert_eq!(status(&mut s), 0);
    assert_eq!(s.int("_ag_cam2_status", &[]).unwrap(), 1);
    assert_eq!(s.int("_ag_cam2_target", &[]).unwrap(), 0);
    assert_eq!(s.int("_ag_cam2_run", &[]).unwrap(), 1);
    assert_eq!(s.float("_ag_cam2_min_x_v", &[]).unwrap(), 0.0);
    assert_eq!(s.float("_ag_cam2_min_y_v", &[]).unwrap(), -4.0);
    assert_eq!(s.float("_ag_cam2_max_x_v", &[]).unwrap(), 30.0);
    assert_eq!(s.float("_ag_cam2_max_y_v", &[]).unwrap(), 12.0);
    assert_eq!(s.float("_ag_cam2_view_w_v", &[]).unwrap(), 12.0);
    assert_eq!(s.float("_ag_cam2_view_h_v", &[]).unwrap(), 8.0);
    tick(&mut s, 1);
    assert_eq!(s.int("_ag_cam2_tick", &[]).unwrap(), 1);
    assert_eq!(
        s.float("_ag_cam2_view_w_v", &[]).unwrap(),
        12.0,
        "config is frozen"
    );
}

/// The frozen free-reading rule (contract section 8): the full controller
/// path sits exactly at 32 credits with the camera reads included, identical
/// to the no-camera controller.
fn camera_reads_stay_outside_the_frozen_credit_budget_body() {
    let rows = scene_rows();
    let with_camera = {
        let snap = SourceSnapshot::from_bytes_for(
            Path::new("credits-cam.at"),
            upd(&format!("{BIND3}{CAM3}"), "let fx1=ag_cam3_forward_x()\nlet fz1=ag_cam3_forward_z()\nlet rx1=ag_cam3_right_x()\nlet rz1=ag_cam3_right_z()\nlet y1=ag_cam3_yaw()\nlet p1=ag_cam3_pitch()\nlet d1=ag_cam3_distance()\nlet q=ag_char3_move(4.0*dt,0.0,0.0,0.002)\nok=ok+ag_char3_grounded(q)\n").as_bytes().to_vec(),
            Api::Camera3V1,
        )
        .unwrap();
        let mut snap = snap;
        snap.source
            .push_str("\nfn probe_calls() int {return _ag_spatial_calls}\n");
        let mut s = Session::new(&snap.source).unwrap();
        s.call("main", &[], ResultKind::Unit).unwrap();
        s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_spatial_reset", &[Int(4)], ResultKind::Unit)
            .unwrap();
        for (n, Row(kind, dim, box7)) in rows.iter().enumerate() {
            s.call(
                "_ag_identity",
                &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
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
            for (i, v) in box7.iter().enumerate() {
                s.call(
                    "_ag_spatial_config",
                    &[Int((n * 7 + i) as i32), Float(*v)],
                    ResultKind::Unit,
                )
                .unwrap();
            }
        }
        for n in 0..4 {
            s.call(
                "_ag_char3_map",
                &[Int(n as i32 + 1), Int(n as i32)],
                ResultKind::Unit,
            )
            .unwrap();
        }
        s.call("_ag_start", &[], ResultKind::Unit).unwrap();
        s.call("_ag_char3_input", &[Int(4), Int(4)], ResultKind::Unit)
            .unwrap();
        tick(&mut s, 1);
        assert_eq!(status(&mut s), 0);
        s.int("probe_calls", &[]).unwrap()
    };
    let without_camera = {
        let snap = SourceSnapshot::from_bytes_for(
            Path::new("credits-base.at"),
            upd(
                BIND3,
                "let q=ag_char3_move(4.0*dt,0.0,0.0,0.002)\nok=ok+ag_char3_grounded(q)\n",
            )
            .as_bytes()
            .to_vec(),
            Api::Character3V1,
        )
        .unwrap();
        let mut snap = snap;
        snap.source
            .push_str("\nfn probe_calls() int {return _ag_spatial_calls}\n");
        let mut s = Session::new(&snap.source).unwrap();
        s.call("main", &[], ResultKind::Unit).unwrap();
        s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_spatial_reset", &[Int(4)], ResultKind::Unit)
            .unwrap();
        for (n, Row(kind, dim, box7)) in rows.iter().enumerate() {
            s.call(
                "_ag_identity",
                &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
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
            for (i, v) in box7.iter().enumerate() {
                s.call(
                    "_ag_spatial_config",
                    &[Int((n * 7 + i) as i32), Float(*v)],
                    ResultKind::Unit,
                )
                .unwrap();
            }
        }
        for n in 0..4 {
            s.call(
                "_ag_char3_map",
                &[Int(n as i32 + 1), Int(n as i32)],
                ResultKind::Unit,
            )
            .unwrap();
        }
        s.call("_ag_start", &[], ResultKind::Unit).unwrap();
        s.call("_ag_char3_input", &[Int(4), Int(4)], ResultKind::Unit)
            .unwrap();
        tick(&mut s, 1);
        assert_eq!(status(&mut s), 0);
        s.int("probe_calls", &[]).unwrap()
    };
    assert_eq!(with_camera, without_camera, "camera reads must not charge");
    assert!(
        with_camera <= 32,
        "credits {with_camera} over the frozen cap"
    );
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
fn camera_identities_parse_with_frozen_numbers_and_profiles() {
    big_stack(camera_identities_parse_with_frozen_numbers_and_profiles_body);
}

#[test]
fn sample_sources_admit_on_their_own_dimension() {
    big_stack(sample_sources_admit_on_their_own_dimension_body);
}

#[test]
fn camera_functions_of_the_other_dimension_are_not_admitted() {
    big_stack(camera_functions_of_the_other_dimension_are_not_admitted_body);
}

#[test]
fn orbit_config_domains_reject_with_frozen_fault_codes() {
    big_stack(orbit_config_domains_reject_with_frozen_fault_codes_body);
}

#[test]
fn follow2d_config_domains_reject_with_frozen_fault_codes() {
    big_stack(follow2d_config_domains_reject_with_frozen_fault_codes_body);
}

#[test]
fn camera_target_must_be_this_run_bound_character() {
    big_stack(camera_target_must_be_this_run_bound_character_body);
}

#[test]
fn duplicate_and_post_freeze_configuration_reject() {
    big_stack(duplicate_and_post_freeze_configuration_reject_body);
}

#[test]
fn missing_configuration_rejects_at_the_start_boundary() {
    big_stack(missing_configuration_rejects_at_the_start_boundary_body);
}

#[test]
fn camera_public_calls_never_bypass_the_host_transport() {
    big_stack(camera_public_calls_never_bypass_the_host_transport_body);
}

#[test]
fn host_sent_basis_and_orbit_mirror_exactly_and_persist() {
    big_stack(host_sent_basis_and_orbit_mirror_exactly_and_persist_body);
}

#[test]
fn follow2d_block_queries_echo_the_frozen_configuration() {
    big_stack(follow2d_block_queries_echo_the_frozen_configuration_body);
}

#[test]
fn camera_reads_stay_outside_the_frozen_credit_budget() {
    big_stack(camera_reads_stay_outside_the_frozen_credit_budget_body);
}
