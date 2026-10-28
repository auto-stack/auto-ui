//! PLAN-874 character3d/1 public API tests (contract V-01..V-03): assembly
//! identity, bind/solid admission, freeze validation, three-axis read-only
//! enforcement, six-action snapshot, token lifecycle, the frozen credit path
//! and the real 3D contact/support geometry. Real Auto Session execution; run
//! single-threaded:
//! cargo test --locked --test character3d_api -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

/// Frozen by contract section 2: v5 clearance; support skin lives in the unit.
const CLEARANCE: f32 = 0.002;
const DT: f32 = 1.0 / 60.0;

/// Static scene row: (kind, dimension, [valid, lo0, lo1, lo2, hi0, hi1, hi2]).
struct Row(i32, i32, [f32; 7]);

fn row(kind: i32, dim: i32, lo: [f32; 3], hi: [f32; 3]) -> Row {
    Row(kind, dim, [1., lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]])
}

/// Character (id 1), floor (2), x-wall (3), z-wall (4), 2D decoy (5), group (6).
fn scene_rows() -> Vec<Row> {
    vec![
        row(2, 3, [-0.4, -0.5, -0.4], [0.4, 0.5, 0.4]),
        row(2, 3, [-8., -1., -8.], [8., 0., 8.]),
        row(2, 3, [3., 0., -1.], [3.2, 3., 1.]),
        row(2, 3, [-1., 0., 3.], [1., 3., 3.2]),
        row(2, 2, [9., 0., 0.], [10., 1., 0.]),
        row(5, 2, [30., -1., 0.], [31., 0., 1.]),
    ]
}

const PRELUDE: &str = "var ok=0\nvar stepn=0\nvar held=0\nvar pressed=0\nvar g=0\nvar sup=0\nvar st=0\nvar x=0.0\nvar y=0.0\nvar z=0.0\n";
const GETTERS: &str = "fn held_v() int {return held}\nfn pressed_v() int {return pressed}\nfn g_v() int {return g}\nfn sup_v() int {return sup}\nfn st_v() int {return st}\nfn x_v() float {return x}\nfn y_v() float {return y}\nfn z_v() float {return z}\n";

fn source(body: &str) -> String {
    format!("{PRELUDE}{body}{GETTERS}")
}

/// Run the frozen three-dimensional configuration protocol: transport reset,
/// spatial rows, stable-id map, then `_ag_start` (app_start + the freeze).
fn session(user: &str, name: &str, rows: &[Row]) -> Result<Session, String> {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        user.as_bytes().to_vec(),
        Api::Character3V1,
    )?;
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
            "_ag_char3_map",
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

fn tick(s: &mut Session, step: i32) {
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

fn int(s: &mut Session, name: &str, v: i32) -> i32 {
    s.int(name, &[Int(v)]).unwrap()
}

fn upd(start: &str, update: &str) -> String {
    source(&format!(
        "fn app_start() {{\n{start}}}\nfn app_update(dt float) {{\nstepn=stepn+1\n{update}}}\n"
    ))
}

fn idle() -> &'static str {
    "let q=ag_char3_move(0.0,0.0,0.0,0.002)\ng=ag_char3_grounded(q)\n"
}

const BIND: &str = " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n";

// ------------------------------------------------- assembly/identity (V-01) --

#[test]
fn character3d_assembly_binds_profile_version_and_entry_hooks() {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("identity.at"),
        upd(
            " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
            idle(),
        )
        .into_bytes(),
        Api::Character3V1,
    )
    .unwrap();
    assert_eq!(snap.version, "autographics.character3d/1");
    assert!(snap.source.contains("fn _ag_profile() int {return 13}"));
    assert!(snap.source.contains("fn _ag_version() int {return 102}"));
    assert!(snap.source.contains("_ag_char3_freeze()"));
    assert!(snap
        .source
        .contains("_ag_char3_tick_begin(); app_update(dt); _ag_cast_token=0"));
    assert!(
        !snap.source.contains("fn ag_char_move("),
        "the 3D profile must not assemble the 2D unit"
    );
    assert!(snap.regions.iter().any(|r| r.file == "<character3d_v1.at>"));
    // behavior/6 stays unclaimed; the old profiles keep parsing.
    assert!(Api::parse("autographics.behavior/6").is_err());
    for v in [
        "autographics.behavior/1",
        "autographics.behavior/2",
        "autographics.behavior/3",
        "autographics.behavior/4",
        "autographics.behavior/5",
        "autographics.character/1",
        "autographics.character3d/1",
    ] {
        assert!(Api::parse(v).is_ok(), "{v}");
    }
    let mut s = session(
        &upd(" ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n", idle()),
        "identity-run.at",
        &scene_rows(),
    )
    .unwrap();
    assert_eq!(s.int("_ag_profile", &[]).unwrap(), 13);
    assert_eq!(s.int("_ag_version", &[]).unwrap(), 102);
    assert_eq!(status(&mut s), 0);
}

#[test]
fn freeze_measures_initial_support_in_three_dimensions() {
    // Feet exactly on the floor top (y=0): measured support, never defaulted.
    let grounded = upd(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
        idle(),
    );
    let mut s = session(&grounded, "support.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0);
    assert_eq!(int(&mut s, "ag_char3_grounded", 0), 1);
    assert_eq!(int(&mut s, "ag_char3_support", 0), 1);
    // Same scene, character lifted: no support, and the answer is not cached.
    let air = upd(
        " ag_set_local(0,1,3.0)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
        idle(),
    );
    let mut s2 = session(&air, "air.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s2), 0);
    assert_eq!(int(&mut s2, "ag_char3_grounded", 0), 0);
    assert_eq!(int(&mut s2, "ag_char3_support", 0), -1);
    // An empty solid set is allowed; nothing supports the character.
    let empty = upd(" ag_set_local(0,1,3.0)\n ok=ok+ag_char3_bind(1)\n", idle());
    let mut s3 = session(&empty, "empty.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s3), 0);
    assert_eq!(int(&mut s3, "ag_char3_grounded", 0), 0);
}

// ------------------------------------------------------ admission faults ----

#[test]
fn bind_rejects_unknown_group_2d_and_unsupported_shape() {
    for (call, expected) in [
        ("ag_char3_bind(9)", 23),  // unknown stable id
        ("ag_char3_bind(6)", 24),  // group
        ("ag_char3_bind(5)", 24),  // 2D decoy
        ("ag_char3_solid(6)", 24), // group solid
        ("ag_char3_solid(5)", 24), // 2D solid
    ] {
        let user = upd(&format!(" ok=ok+{call}\n"), idle());
        let mut s = session(&user, "bad.at", &scene_rows()).unwrap();
        assert_eq!(status(&mut s), expected, "{call}");
    }
    for (calls, expected) in [
        (" ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_bind(1)\n", 23),
        (" ok=ok+ag_char3_solid(2)\n ok=ok+ag_char3_solid(2)\n", 23),
    ] {
        let user = upd(calls, idle());
        let mut s = session(&user, "dup.at", &scene_rows()).unwrap();
        assert_eq!(status(&mut s), expected, "{calls}");
    }
    // Shape is pinned to the 858 half extents .4/.5/.4 with depth.
    let mut rows = scene_rows();
    rows[0] = row(2, 3, [-0.4, -0.5, -0.2], [0.4, 0.5, 0.2]);
    let mut s = session(&upd(BIND, idle()), "shape-depth.at", &rows).unwrap();
    assert_eq!(status(&mut s), 24);
    let mut rows2 = scene_rows();
    rows2[0] = row(2, 3, [-1., -2., -1.], [1., 2., 1.]);
    let mut s2 = session(&upd(BIND, idle()), "shape.at", &rows2).unwrap();
    assert_eq!(status(&mut s2), 24);
}

#[test]
fn cross_membership_and_missing_character_reject_at_freeze() {
    // Solid declared first, then bound as the character: rejected at freeze.
    let cross = upd(
        " ok=ok+ag_char3_solid(1)\n ok=ok+ag_char3_bind(1)\n",
        idle(),
    );
    let mut s = session(&cross, "cross.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 23);
    // No character at freeze.
    let none = upd(" ok=ok+ag_char3_solid(2)\n", idle());
    let mut s2 = session(&none, "none.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s2), 25);
}

#[test]
fn config_after_freeze_and_move_before_freeze_fault() {
    let mut s = session(&upd(BIND, idle()), "late.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0);
    s.call("ag_char3_bind", &[Int(2)], ResultKind::Int).unwrap();
    assert_eq!(status(&mut s), 22);
    // move before any freeze: configured scene without _ag_start.
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("early.at"),
        upd(BIND, idle()).into_bytes(),
        Api::Character3V1,
    )
    .unwrap();
    let mut s2 = Session::new(&snap.source).unwrap();
    s2.call("main", &[], ResultKind::Unit).unwrap();
    s2.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s2.call("_ag_spatial_reset", &[Int(6)], ResultKind::Unit)
        .unwrap();
    for (n, Row(kind, dim, box7)) in scene_rows().iter().enumerate() {
        s2.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s2.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
            .unwrap();
        s2.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s2.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in box7.iter().enumerate() {
            s2.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    s2.call(
        "ag_char3_move",
        &[Float(0.), Float(0.), Float(0.), Float(CLEARANCE)],
        ResultKind::Int,
    )
    .unwrap();
    assert_eq!(status(&mut s2), 26);
}

#[test]
fn externally_moved_solid_and_character_are_rejected_on_every_axis() {
    // Third-axis (z) solid bypass: the 2D unit only checked x/y.
    let mover = upd(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
        "if stepn==2 {ag_set_local(1,2,1.5)}\nlet q=ag_char3_move(0.0,0.0,0.0,0.002)\ng=ag_char3_grounded(q)\n",
    );
    let mut s = session(&mover, "solidmove.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    tick(&mut s, 2);
    assert_eq!(status(&mut s), 0);
    // The bypass is rejected at the next tick boundary (run index 1 = floor).
    tick(&mut s, 3);
    assert_eq!(status(&mut s), 20);
    assert_eq!(fault_node(&mut s), 1);
    // Character moved around the controller (a raw v5 move, not ag_char3_move):
    // the controller's published translation is stale at the next boundary.
    let bypass = upd(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
        "if stepn==2 {let t=ag_move_and_slide(0,0,2.0,0.0,0.0,0.002)}\nif stepn!=2 {let q=ag_char3_move(0.0,0.0,0.0,0.002)\ng=ag_char3_grounded(q)}\n",
    );
    let mut s2 = session(&bypass, "charmove.at", &scene_rows()).unwrap();
    tick(&mut s2, 1);
    tick(&mut s2, 2);
    assert_eq!(status(&mut s2), 0);
    tick(&mut s2, 3);
    assert_eq!(status(&mut s2), 21);
}

// --------------------------------------------------- actions and tokens -----

#[test]
fn held_reads_the_six_action_snapshot_and_pressed_consumes_once_per_tick() {
    // Budget: 26 (move) + 6 held reads = 32 exactly, so held and edge reads
    // live in separate sources; one callback cannot exceed the pool.
    let held_user = upd(
        BIND,
        "held=ag_action_held(0)+ag_action_held(1)*2+ag_action_held(2)*4+ag_action_held(3)*8+ag_action_held(4)*16+ag_action_held(5)*32\nlet q=ag_char3_move(0.0,0.0,0.0,0.002)\n",
    );
    let mut s = session(&held_user, "held6.at", &scene_rows()).unwrap();
    s.call("_ag_char3_input", &[Int(63), Int(16)], ResultKind::Unit)
        .unwrap();
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0);
    assert_eq!(s.int("held_v", &[]).unwrap(), 63);
    // Edges: one dispatch consumes each action edge at most once.
    let edge_user = upd(
        BIND,
        "pressed=ag_action_pressed(4)*10+ag_action_pressed(4)+ag_action_pressed(5)*100\nlet q=ag_char3_move(0.0,0.0,0.0,0.002)\ng=ag_char3_grounded(q)\n",
    );
    let mut s2 = session(&edge_user, "edges.at", &scene_rows()).unwrap();
    s2.call("_ag_char3_input", &[Int(0), Int(16 | 32)], ResultKind::Unit)
        .unwrap();
    tick(&mut s2, 1);
    assert_eq!(status(&mut s2), 0);
    // Jump read twice -> 10 then 0; restart edge still pending -> 100.
    assert_eq!(s2.int("pressed_v", &[]).unwrap(), 10 + 100);
    // A second read of the same dispatch consumes the pending restart too.
    s2.call("_ag_char3_input", &[Int(0), Int(0)], ResultKind::Unit)
        .unwrap();
    tick(&mut s2, 2);
    assert_eq!(s2.int("pressed_v", &[]).unwrap(), 0);
}

#[test]
fn action_range_and_the_frozen_charge_path_are_enforced() {
    let user = upd(BIND, idle());
    let mut s = session(&user, "range.at", &scene_rows()).unwrap();
    s.call("ag_action_held", &[Int(6)], ResultKind::Int)
        .unwrap();
    assert_eq!(status(&mut s), 7);
    // 26 (24 move + 1 reserve + 1 fused support) + 6 held reads = 32 exactly.
    let budget = upd(
        BIND,
        "let q=ag_char3_move(0.0,0.0,0.0,0.002)\nheld=ag_action_held(0)+ag_action_held(1)+ag_action_held(2)+ag_action_held(3)+ag_action_held(4)+ag_action_held(5)\n",
    );
    let mut s2 = session(&budget, "budget.at", &scene_rows()).unwrap();
    tick(&mut s2, 1);
    assert_eq!(status(&mut s2), 0, "move + six reads must fit 32 credits");
    // One more credit in the same callback crosses the unchanged pool.
    let over = upd(
        BIND,
        "let q=ag_char3_move(0.0,0.0,0.0,0.002)\nheld=ag_action_held(0)+ag_action_held(1)+ag_action_held(2)+ag_action_held(3)+ag_action_held(4)+ag_action_held(5)\nsup=ag_char3_support(q)\n",
    );
    let mut s3 = session(&over, "over.at", &scene_rows()).unwrap();
    tick(&mut s3, 1);
    assert_eq!(status(&mut s3), 10, "the pool is unchanged at 32");
}

#[test]
fn stale_tokens_reject_and_the_move_result_slot_is_not_overwritten() {
    // Step 2 moves down onto the floor; the scan must not disturb the v5
    // result slot, so the move token still reads its own contact data.
    let user = format!(
        "{PRELUDE}var q1=0\nvar q2=0\nvar n2=-2\nvar g2=0\nfn app_start() {{\n ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)}}\nfn app_update(dt float) {{\nstepn=stepn+1\nif stepn==1 {{q1=ag_char3_move(0.0,0.0,0.0,0.002)}}\nif stepn==2 {{q2=ag_char3_move(0.0, -0.001,0.0,0.002)\nn2=ag_motion_node(q2)\ng2=ag_char3_grounded(q2)\nst=ag_motion_status(q1)}}}}\nfn q2_v() int {{return q2}}\nfn n2_v() int {{return n2}}\nfn g2_v() int {{return g2}}\n{GETTERS}"
    );
    let mut s = session(&user, "tokens.at", &scene_rows()).unwrap();
    tick(&mut s, 1);
    tick(&mut s, 2);
    assert_eq!(s.int("g2_v", &[]).unwrap(), 1, "current token grounded");
    // Contact node is the floor (run index 1), not the support re-publication.
    assert_eq!(s.int("n2_v", &[]).unwrap(), 1);
    // q1 is one step old: the read rejects it with fault 13 and returns 0.
    assert_eq!(s.int("st_v", &[]).unwrap(), 0);
    assert_eq!(status(&mut s), 13);
}

// ------------------------------------------------------ 3D contact (V-03) ---

#[test]
fn clips_on_all_three_axes_and_keeps_final_support() {
    // +x into the x-wall: clip at 3.0 - 0.4 - clearance.
    let mut s = session(
        &upd(
            " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_char3_solid(3)\n",
            "let q=ag_char3_move(4.0*dt,0.0,0.0,0.002)\ng=ag_char3_grounded(q)\nsup=ag_char3_support(q)\nx=ag_motion_origin(q,0)\n",
        ),
        "xclip.at",
        &scene_rows(),
    )
    .unwrap();
    for step in 1..=200 {
        tick(&mut s, step);
        assert_eq!(status(&mut s), 0, "step {step}");
    }
    let x = s.float("x_v", &[]).unwrap();
    assert!((x - (3.0 - 0.4 - CLEARANCE)).abs() < 1e-3, "x clip {x}");
    assert_eq!(s.int("g_v", &[]).unwrap(), 1);
    assert_eq!(s.int("sup_v", &[]).unwrap(), 1);
    // -z into the z-wall: clip at 3.0 - 0.4 - clearance on the third axis.
    let mut s2 = session(
        &upd(
            " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_char3_solid(4)\n",
            "let q=ag_char3_move(0.0,0.0, 4.0*dt,0.002)\ng=ag_char3_grounded(q)\nsup=ag_char3_support(q)\nz=ag_motion_origin(q,2)\n",
        ),
        "zclip.at",
        &scene_rows(),
    )
    .unwrap();
    for step in 1..=200 {
        tick(&mut s2, step);
        assert_eq!(status(&mut s2), 0, "step {step}");
    }
    let z = s2.float("z_v", &[]).unwrap();
    assert!((z - (3.0 - 0.4 - CLEARANCE)).abs() < 1e-3, "z clip {z}");
}

#[test]
fn ceiling_stops_upward_motion_and_grounded_clears_in_the_air() {
    let rows = vec![
        row(2, 3, [-0.4, -0.5, -0.4], [0.4, 0.5, 0.4]),
        row(2, 3, [-8., -1., -8.], [8., 0., 8.]),
        row(2, 3, [-1., 2., -1.], [1., 2.2, 1.]), // ceiling
    ];
    let user = upd(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_char3_solid(3)\n",
        "let q=ag_char3_move(0.0,0.3,0.0,0.002)\ng=ag_char3_grounded(q)\ny=ag_motion_origin(q,1)\n",
    );
    let mut s = session(&user, "ceiling.at", &rows).unwrap();
    // 0.3 per step from y=0.5: the head (y+0.5) reaches the ceiling bottom
    // (2.0) at y=1.5, then upward motion stops.
    tick(&mut s, 1);
    let y1 = s.float("y_v", &[]).unwrap();
    assert!((y1 - 0.8).abs() < 1e-4, "free rise {y1}");
    for step in 2..=6 {
        tick(&mut s, step);
        assert_eq!(status(&mut s), 0, "step {step}");
    }
    let y = s.float("y_v", &[]).unwrap();
    assert!(y <= 1.5 + 1e-3, "ceiling clip y={y}");
    assert_eq!(s.int("g_v", &[]).unwrap(), 0, "airborne is not grounded");
}

#[test]
fn edge_loss_wall_top_landing_and_initial_overlap_report_real_state() {
    // Platform edge: walking past x = 4.0 + 0.4 loses final support.
    let rows = vec![
        row(2, 3, [-0.4, -0.5, -0.4], [0.4, 0.5, 0.4]),
        row(2, 3, [-1., 0., -1.], [4., 0.5, 1.]), // platform, top 0.5
    ];
    let walk = upd(
        " ag_set_local(0,1,1.0)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
        "let q=ag_char3_move(4.0*dt,0.0,0.0,0.002)\ng=ag_char3_grounded(q)\nsup=ag_char3_support(q)\nx=ag_motion_origin(q,0)\n",
    );
    let mut s = session(&walk, "edge.at", &rows).unwrap();
    assert_eq!(int(&mut s, "ag_char3_grounded", 0), 1);
    for step in 1..=20 {
        tick(&mut s, step);
        assert_eq!(status(&mut s), 0, "step {step}");
    }
    assert_eq!(s.int("g_v", &[]).unwrap(), 1, "still on the platform");
    for step in 21..=120 {
        tick(&mut s, step);
        assert_eq!(status(&mut s), 0, "step {step}");
    }
    assert!(s.float("x_v", &[]).unwrap() > 4.4, "walked off the edge");
    assert_eq!(s.int("g_v", &[]).unwrap(), 0, "edge loses final support");
    assert_eq!(s.int("sup_v", &[]).unwrap(), -1);

    // Wall-top landing: one sweeps down onto the platform top (0.5) and the
    // final support is the platform, measured after the clip.
    let drop = upd(
        " ag_set_local(0,1,3.0)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
        "var dy=0.0\nif stepn==1 {dy= -2.05}\nlet q=ag_char3_move(0.0,dy,0.0,0.002)\ng=ag_char3_grounded(q)\nsup=ag_char3_support(q)\ny=ag_motion_origin(q,1)\n",
    );
    let mut s2 = session(&drop, "land.at", &rows).unwrap();
    for step in 1..=3 {
        tick(&mut s2, step);
        assert_eq!(status(&mut s2), 0, "step {step}");
    }
    let y = s2.float("y_v", &[]).unwrap();
    assert!(
        (y - 1.002).abs() < 1e-2,
        "landed on the platform top (feet 0.5 + clearance) y={y}"
    );
    assert_eq!(s2.int("g_v", &[]).unwrap(), 1);
    assert_eq!(s2.int("sup_v", &[]).unwrap(), 1);

    // Initial overlap: status 3 is reported and never counted as grounded.
    let overlap = upd(
        " ag_set_local(0,1,0.2)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_char3_solid(3)\n",
        "let q=ag_char3_move(0.0,0.0,0.0,0.002)\ng=ag_char3_grounded(q)\nst=ag_motion_status(q)\n",
    );
    let mut s3 = session(&overlap, "overlap.at", &scene_rows()).expect("overlap session");
    tick(&mut s3, 1);
    assert_eq!(s3.int("st_v", &[]).unwrap(), 3, "initial overlap status");
    assert_eq!(s3.int("g_v", &[]).unwrap(), 0, "status 3 is not grounded");
}

#[test]
fn nearest_blocker_last_wins_on_the_128_node_load() {
    // 128 static nodes through the public profile: v5 nearest-blocker-wins
    // must hold with the blocker iterated LAST (node 127).
    const LOAD: &str = "var ok=0
var g=0
var x=0.0
fn app_start() {
 ag_set_local(0,1,0.5)
 ok=ok+ag_char3_bind(1)
}
fn app_update(dt float) {
 let q=ag_char3_move(4.0*dt,0.0,0.0,0.002)
 g=ag_char3_grounded(q)
 x=ag_motion_origin(q,0)
}
fn x_v() float {return x}
fn g_v2() int {return g}
";
    let mut rows: Vec<[f32; 7]> = Vec::new();
    rows.push([1., -0.4, -0.5, -0.4, 0.4, 0.5, 0.4]);
    for i in 0..126 {
        let x = 3.0 + 0.3 * i as f32;
        rows.push([1., x, -1., -1., x + 0.5, 1., 1.]);
    }
    rows.push([1., 2.0, -1., -1., 2.5, 1., 1.]); // nearest blocker, last
    assert_eq!(rows.len(), 128);
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("load.at"),
        LOAD.as_bytes().to_vec(),
        Api::Character3V1,
    )
    .unwrap();
    let mut s = Session::new(&snap.source).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(128)], ResultKind::Unit)
        .unwrap();
    for (n, r) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(2), Int(3), Int(-1)],
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
        for (i, v) in r.iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..128 {
        s.call(
            "_ag_char3_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    for step in 1..=60 {
        s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
            .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "load tick {step}");
    }
    let x = s.float("x_v", &[]).unwrap();
    assert!(
        (x - (2.0 - 0.4 - CLEARANCE)).abs() < 1e-3,
        "nearest blocker clip x={x}"
    );
}

#[test]
fn rotated_horizontal_basis_rotates_the_published_world_motion() {
    // Default basis F=(0,0,-1), R=(1,0,0) lives in the user source; a 90 degree
    // basis swap must rotate the world motion with no Rust change (camera seam).
    let rotated = upd(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n",
        "let hf=ag_action_held(2)\nlet q=ag_char3_move((hf)*3.0*dt,0.0,0.0,0.002)\ng=ag_char3_grounded(q)\nx=ag_motion_origin(q,0)\nz=ag_motion_origin(q,2)\n",
    );
    let mut s = session(&rotated, "rotated.at", &scene_rows()).unwrap();
    s.call("_ag_char3_input", &[Int(4), Int(0)], ResultKind::Unit)
        .unwrap();
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0);
    let x = s.float("x_v", &[]).unwrap();
    let z = s.float("z_v", &[]).unwrap();
    assert!((x - 3.0 * DT).abs() < 1e-4, "rotated forward x={x}");
    assert!(z.abs() < 1e-6, "rotated basis must not move z: {z}");
}

// F-874-R1-01: the two contract section 4 rules that had no assertion.

#[test]
fn hidden_solid_still_collides_and_supports() {
    // Visibility never changes collision: a hidden floor still carries the
    // character, and a hidden wall still clips horizontal motion.
    let rows = vec![
        row(2, 3, [-0.4, -0.5, -0.4], [0.4, 0.5, 0.4]),
        row(2, 3, [-8., -1., -8.], [8., 0., 8.]),
        row(2, 3, [3., 0., -1.], [3.2, 3., 1.]),
    ];
    let user = upd(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_char3_solid(3)\n",
        "let q=ag_char3_move(4.0*dt,0.0,0.0,0.002)\ng=ag_char3_grounded(q)\nsup=ag_char3_support(q)\nx=ag_motion_origin(q,0)\n",
    );
    let mut s = session(&user, "hidden.at", &rows).unwrap();
    // Hide the floor (run index 1) and the wall (run index 2) before the freeze.
    for i in [1, 2] {
        s.call("_ag_visibility", &[Int(i)], ResultKind::Unit)
            .unwrap();
    }
    assert_eq!(status(&mut s), 0);
    assert_eq!(
        int(&mut s, "ag_char3_grounded", 0),
        1,
        "hidden floor supports"
    );
    assert_eq!(int(&mut s, "ag_char3_support", 0), 1);
    for step in 1..=200 {
        tick(&mut s, step);
        assert_eq!(status(&mut s), 0, "step {step}");
    }
    let x = s.float("x_v", &[]).unwrap();
    assert!(
        (x - (3.0 - 0.4 - CLEARANCE)).abs() < 1e-3,
        "hidden wall must still clip: x={x}"
    );
}

#[test]
fn ancestor_group_translation_is_refused_at_the_tick_boundary() {
    // A group translation inside app_update changes the world of every node
    // under it; the next tick boundary must refuse the run rather than publish
    // the bypassed translation (contract sections 4/5.2). Self-contained scene
    // because the shared helper only builds parentless rows.
    //   run 0 / stable id 1: group (3D)
    //   run 1 / stable id 2: character, parent 0
    //   run 2 / stable id 3: floor, parent 0 (so the group carries it too)
    let rows: [(i32, i32, i32, [f32; 7]); 3] = [
        (5, 3, -1, [0.; 7]),
        (2, 3, 0, [1., -0.4, -0.5, -0.4, 0.4, 0.5, 0.4]),
        (2, 3, 0, [1., -8., -1., -8., 8., 0., 8.]),
    ];
    let user = upd(
        " ag_set_local(1,1,0.5)\n ok=ok+ag_char3_bind(2)\n ok=ok+ag_char3_solid(3)\n",
        "if stepn==2 {ag_set_local(0,2,1.5)}\nif stepn!=2 {let q=ag_char3_move(0.0,0.0,0.0,0.002)\ng=ag_char3_grounded(q)}\n",
    );
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("ancestor.at"),
        user.as_bytes().to_vec(),
        Api::Character3V1,
    )
    .unwrap();
    let mut s = Session::new(&snap.source).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(3)], ResultKind::Unit)
        .unwrap();
    for (n, (kind, dim, parent, box7)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(*parent)],
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
                &[Int((n * 7 + i as usize) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..3 {
        s.call(
            "_ag_char3_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(status(&mut s), 0, "configured scene");
    assert_eq!(
        int(&mut s, "ag_char3_grounded", 0),
        1,
        "group-carried support"
    );
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0, "step 1 clean");
    tick(&mut s, 2);
    assert_eq!(
        status(&mut s),
        0,
        "the bypass tick itself is not yet refused"
    );
    tick(&mut s, 3);
    assert_eq!(status(&mut s), 20, "solid world drift through its group");
    assert_eq!(
        fault_node(&mut s),
        2,
        "fault node is the group-carried solid run index"
    );
}
