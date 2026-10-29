//! T-02 producer probe. Host/GPU transaction coverage is added in T-02..05;
//! this file currently proves real Auto source assembly, output and budget.
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot, MAX_BRIDGE},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

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
