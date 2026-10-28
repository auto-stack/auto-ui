//! PLAN-861 character/1 public API tests (contract V-01): assembly identity,
//! bind/solid admission, freeze validation, read-only enforcement, actions,
//! token lifecycle and unchanged credit/fault semantics. Real Auto Session
//! execution; run single-threaded:
//! cargo test --locked --test character2d_api -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

/// Frozen per contract: v5 clearance; support skin lives in the unit (0.01).
const CLEARANCE: f32 = 0.002;
const DT: f32 = 1.0 / 60.0;

/// Static scene row: (kind, dimension, [valid, lo0, lo1, lo2, hi0, hi1, hi2]).
struct Row(i32, i32, [f32; 7]);

fn row(kind: i32, dim: i32, lo: [f32; 3], hi: [f32; 3]) -> Row {
    Row(kind, dim, [1., lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]])
}

/// Character + floor + wall + a 3D decoy + a group root. Stable ids 1..=5 in
/// document order; run indices are 0-based array positions.
fn scene_rows() -> Vec<Row> {
    vec![
        row(2, 2, [-0.4, -0.5, 0.], [0.4, 0.5, 0.]), // id 1: character
        row(2, 2, [-4., -1., 0.], [12., 0., 0.]),    // id 2: floor
        row(2, 2, [8., 0., 0.], [9., 3., 0.]),       // id 3: wall
        row(2, 3, [20., -1., 20.], [21., 0., 21.]),  // id 4: 3D decoy
        row(5, 2, [30., -1., 0.], [31., 0., 1.]),    // id 5: group
    ]
}

/// User sources share one prelude (all observed globals) and one getter tail
/// so tests read state through real Auto functions.
const PRELUDE: &str = "var ok=0\nvar stepn=0\nvar held=0\nvar pressed=0\nvar g=0\nvar sup=0\nvar st=0\nvar contacts=0\nvar uy=0.0\nvar x=0.0\n";
const GETTERS: &str = "fn held_v() int {return held}\nfn pressed_v() int {return pressed}\nfn g_v() int {return g}\nfn sup_v() int {return sup}\nfn st_v() int {return st}\nfn contacts_v() int {return contacts}\nfn uy_v() float {return uy}\nfn x_v() float {return x}\n";

fn source(body: &str) -> String {
    format!("{PRELUDE}{body}{GETTERS}")
}

/// Build the combined character/1 source for `user` and run the frozen scene
/// configuration protocol: transport reset, spatial rows, stable-id map, then
/// `_ag_start` (which runs app_start and the freeze).
fn character_session(user: &str, name: &str, rows: &[Row]) -> Result<Session, String> {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        user.as_bytes().to_vec(),
        Api::CharacterV1,
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
    for (n, _) in rows.iter().enumerate() {
        s.call(
            "_ag_char_map",
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

fn source_with_app_start(start_body: &str, update_body: &str) -> String {
    source(&format!(
        "fn app_start() {{\n{start_body}}}\nfn app_update(dt float) {{\nstepn=stepn+1\n{update_body}}}\n"
    ))
}

fn idle_update() -> String {
    "let q=ag_char_move(0.0,0.0,0.002)\ng=ag_char_grounded(q)\n".into()
}

const BIND3: &str = " ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_char_solid(3)\n";
const BIND2: &str = " ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n";

// ------------------------------------------------- assembly/identity (V-01) --

#[test]
fn character_assembly_binds_profile_version_and_entry_hooks() {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("identity.at"),
        source_with_app_start(BIND3, &idle_update()).into_bytes(),
        Api::CharacterV1,
    )
    .unwrap();
    assert_eq!(snap.version, "autographics.character/1");
    // The shared entry symbols are transport-namespaced in the combined source.
    assert!(snap.source.contains("fn _ag_profile() int {return 12}"));
    assert!(snap.source.contains("fn _ag_version() int {return 101}"));
    assert!(snap.source.contains("_ag_char_freeze()"));
    assert!(snap
        .source
        .contains("_ag_char_tick_begin(); app_update(dt); _ag_cast_token=0"));
    assert_eq!(
        snap.regions
            .iter()
            .find(|r| r.file == "<character2d_v1.at>")
            .map(|r| r.file.clone())
            .unwrap(),
        "<character2d_v1.at>"
    );
    // behavior/6 stays unclaimed and the old five versions keep parsing.
    assert!(Api::parse("autographics.behavior/6").is_err());
    for v in [
        "autographics.behavior/1",
        "autographics.behavior/2",
        "autographics.behavior/3",
        "autographics.behavior/4",
        "autographics.behavior/5",
    ] {
        assert!(Api::parse(v).is_ok());
    }
    let mut s = character_session(
        &source_with_app_start(BIND3, &idle_update()),
        "identity.at",
        &scene_rows(),
    )
    .unwrap();
    assert_eq!(s.int("_ag_profile", &[]).unwrap(), 12);
    assert_eq!(s.int("_ag_version", &[]).unwrap(), 101);
    assert_eq!(status(&mut s), 0);
}

#[test]
fn bind_freeze_and_initial_support_follow_the_actual_scene() {
    // Spawn with the box bottom exactly on the floor top (y=0.5): the
    // freeze-time neutral down-scan must observe real support, never default.
    let grounded_spawn = source_with_app_start(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_char_solid(3)\n",
        &idle_update(),
    );
    let mut s = character_session(&grounded_spawn, "support.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0);
    assert_eq!(int(&mut s, "ag_char_grounded", 0), 1);
    assert_eq!(int(&mut s, "ag_char_support", 0), 1);
    // In-air spawn: same scene, character lifted above the floor.
    let air = source_with_app_start(
        " ag_set_local(0,1,3.0)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_char_solid(3)\n",
        &idle_update(),
    );
    let mut s2 = character_session(&air, "air.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s2), 0);
    assert_eq!(int(&mut s2, "ag_char_grounded", 0), 0);
    assert_eq!(int(&mut s2, "ag_char_support", 0), -1);
}

// ------------------------------------------------------ admission faults ----

#[test]
fn bind_rejects_unknown_group_3d_and_duplicate_targets() {
    for (call, expected) in [
        ("ag_char_bind(9)", 23),  // unknown stable id
        ("ag_char_bind(5)", 24),  // group
        ("ag_char_bind(4)", 24),  // 3D
        ("ag_char_solid(5)", 24), // group solid
        ("ag_char_solid(4)", 24), // 3D solid
    ] {
        let user = source_with_app_start(&format!(" ok=ok+{call}\n"), &idle_update());
        let mut s = character_session(&user, "bad.at", &scene_rows()).unwrap();
        assert_eq!(status(&mut s), expected, "{call}");
    }
    // Duplicate bind and duplicate solid declaration.
    for (calls, expected) in [
        (" ok=ok+ag_char_bind(1)\n ok=ok+ag_char_bind(1)\n", 23),
        (" ok=ok+ag_char_solid(2)\n ok=ok+ag_char_solid(2)\n", 23),
    ] {
        let user = source_with_app_start(calls, &idle_update());
        let mut s = character_session(&user, "dup.at", &scene_rows()).unwrap();
        assert_eq!(status(&mut s), expected, "{calls}");
    }
    // Missing character at freeze.
    let user = source_with_app_start(" ok=ok+ag_char_solid(2)\n", &idle_update());
    let mut s = character_session(&user, "nochar.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 25);
    // Unsupported character shape: half extents are pinned to 0.4/0.5.
    let mut rows = scene_rows();
    rows[0] = row(2, 2, [-1., -2., 0.], [1., 2., 0.]);
    let user = source_with_app_start(BIND3, &idle_update());
    let mut s2 = character_session(&user, "shape.at", &rows).unwrap();
    assert_eq!(status(&mut s2), 24);
}

#[test]
fn config_after_freeze_and_move_before_freeze_fault() {
    // Runtime rejection: a bind call issued after the freeze.
    let mut s = character_session(
        &source_with_app_start(BIND3, &idle_update()),
        "late.at",
        &scene_rows(),
    )
    .unwrap();
    assert_eq!(status(&mut s), 0);
    s.call("ag_char_bind", &[Int(2)], ResultKind::Int).unwrap();
    assert_eq!(status(&mut s), 22);
    // char_move before any freeze: configured scene without _ag_start.
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("early.at"),
        source_with_app_start(BIND3, &idle_update()).into_bytes(),
        Api::CharacterV1,
    )
    .unwrap();
    let mut s2 = Session::new(&snap.source).unwrap();
    s2.call("main", &[], ResultKind::Unit).unwrap();
    s2.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s2.call("_ag_spatial_reset", &[Int(5)], ResultKind::Unit)
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
                &[Int(((n * 7) + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    s2.call(
        "ag_char_move",
        &[Float(0.), Float(0.), Float(CLEARANCE)],
        ResultKind::Int,
    )
    .unwrap();
    assert_eq!(status(&mut s2), 26);
}

#[test]
fn externally_moved_solid_and_character_are_rejected_at_tick_begin() {
    // Solid moved by a user setter inside app_update: tick 2 tick_begin.
    let mover = source_with_app_start(
        BIND2,
        "if stepn==2 {ag_set_local(1,0,1.5)}\nlet q=ag_char_move(0.0,0.0,0.002)\ng=ag_char_grounded(q)\n",
    );
    let mut s = character_session(&mover, "solidmove.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 2);
    assert_eq!(status(&mut s), 0);
    // The bypass is rejected at the next tick boundary, per contract section 5.
    tick(&mut s, 3);
    assert_eq!(status(&mut s), 20);
    assert_eq!(fault_node(&mut s), 1);
    // Character moved by a bypass setter instead of the controller.
    let bypass = source_with_app_start(
        BIND2,
        "if stepn==2 {ag_set_local(0,0,5.0)}\nlet q=ag_char_move(0.0,0.0,0.002)\ng=ag_char_grounded(q)\n",
    );
    let mut s2 = character_session(&bypass, "charmove.at", &scene_rows()).unwrap();
    tick(&mut s2, 1);
    tick(&mut s2, 2);
    assert_eq!(status(&mut s2), 0);
    tick(&mut s2, 3);
    assert_eq!(status(&mut s2), 21);
}

// --------------------------------------------------- actions and tokens -----

#[test]
fn held_reads_snapshot_and_pressed_consumes_once_per_tick() {
    let user = source_with_app_start(
        BIND3,
        "held=ag_action_held(0)+ag_action_held(1)*2+ag_action_held(2)*4+ag_action_held(3)*8\npressed=ag_action_pressed(2)*10+ag_action_pressed(2)*1\nlet q=ag_char_move(0.0,0.0,0.002)\ng=ag_char_grounded(q)\n",
    );
    let mut s = character_session(&user, "actions.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0);
    // held bits: left+right+restart -> 1+2+8 = 11; jump edge delivered (bit 4).
    s.call("_ag_char_input", &[Int(11), Int(4)], ResultKind::Unit)
        .unwrap();
    tick(&mut s, 1);
    assert_eq!(s.int("held_v", &[]).unwrap(), 1 + 2 + 8);
    // First pressed read in the tick returns 10, the second read is 0.
    assert_eq!(s.int("pressed_v", &[]).unwrap(), 10);
    // A new tick without edges: pressed stale -> 0, held snapshot replaced.
    s.call("_ag_char_input", &[Int(0), Int(0)], ResultKind::Unit)
        .unwrap();
    tick(&mut s, 2);
    assert_eq!(s.int("pressed_v", &[]).unwrap(), 0);
    assert_eq!(s.int("held_v", &[]).unwrap(), 0);
}

#[test]
fn action_reads_reject_unknown_codes_and_char_move_twice_self_faults() {
    let user = source_with_app_start(BIND3, &idle_update());
    let mut s = character_session(&user, "actions2.at", &scene_rows()).unwrap();
    s.call("ag_action_held", &[Int(4)], ResultKind::Int)
        .unwrap();
    assert_eq!(status(&mut s), 7);
    let mut s2 = character_session(&user, "actions3.at", &scene_rows()).unwrap();
    s2.call(
        "ag_char_move",
        &[Float(0.), Float(0.), Float(CLEARANCE)],
        ResultKind::Int,
    )
    .unwrap();
    assert_eq!(status(&mut s2), 0);
    // 25 + 25 credits exceed the unchanged 32 pool inside one callback.
    s2.call(
        "ag_char_move",
        &[Float(0.), Float(0.), Float(CLEARANCE)],
        ResultKind::Int,
    )
    .unwrap();
    assert_eq!(status(&mut s2), 10);
}

#[test]
fn stale_tokens_reject_and_v5_getters_remain_public() {
    // Step 2 moves down onto the floor and reads the CURRENT token through
    // both getter families first; the step-1 token is read last because its
    // fault 13 is terminal for the callback (later reads stay put).
    let user = format!(
        "{PRELUDE}var q1=0\nvar q2=0\nvar g2=0\nfn app_start() {{\n ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)}}\nfn app_update(dt float) {{\nstepn=stepn+1\nif stepn==1 {{q1=ag_char_move(0.0,0.0,0.002)}}\nif stepn==2 {{q2=ag_char_move(0.0, -0.001,0.002)}}\nif stepn==2 {{contacts=ag_motion_contacts(q2)\nuy=ag_motion_unapplied(q2,1)\ng2=ag_char_grounded(q2)\nst=ag_motion_status(q1)}}}}\nfn q2_v() int {{return q2}}\nfn g2_v() int {{return g2}}\n{GETTERS}"
    );
    let mut s = character_session(&user, "tokens.at", &scene_rows()).unwrap();
    tick(&mut s, 1);
    tick(&mut s, 2);
    // The current step-2 token reports the real move through v5 getters.
    assert_eq!(s.int("contacts_v", &[]).unwrap(), 1);
    assert!(s.float("uy_v", &[]).unwrap() < 0.0);
    // The character getter agrees on the same current token.
    assert_eq!(s.int("g2_v", &[]).unwrap(), 1);
    // q1 is one step old: the read rejects it with fault 13 and returns 0.
    assert_eq!(s.int("st_v", &[]).unwrap(), 0);
    assert_eq!(status(&mut s), 13);
}

#[test]
fn move_on_floor_lands_and_reports_support_geometry() {
    let walker = source_with_app_start(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_char_solid(3)\n",
        "let q=ag_char_move(4.0*dt,0.0,0.002)\ng=ag_char_grounded(q)\nsup=ag_char_support(q)\nx=ag_motion_origin(q,0)\n",
    );
    let mut s = character_session(&walker, "walk.at", &scene_rows()).unwrap();
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(s.int("g_v", &[]).unwrap(), 1);
    assert_eq!(s.int("sup_v", &[]).unwrap(), 1);
    let x1 = s.float("x_v", &[]).unwrap();
    assert!((x1 - 4.0 * DT).abs() < 1e-4, "x={x1}");
    // Walk into the wall (x in [8,9]): the sweep clips, support stays the floor.
    for step in 2..200 {
        tick(&mut s, step);
        assert_eq!(status(&mut s), 0, "step {step}");
    }
    let x_end = s.float("x_v", &[]).unwrap();
    assert!(x_end <= 7.6 + 1e-3, "clipped x={x_end}");
    assert_eq!(s.int("g_v", &[]).unwrap(), 1);
    assert_eq!(s.int("sup_v", &[]).unwrap(), 1);
}

#[test]
fn credit_step_budget_stays_inside_the_unchanged_pool() {
    let poller = source_with_app_start(
        " ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n",
        "held=ag_action_held(0)+ag_action_held(1)+ag_action_held(2)+ag_action_held(3)\nlet q=ag_char_move(0.0,0.0,0.002)\ng=ag_char_grounded(q)\nsup=ag_char_support(q)\n",
    );
    let mut s = character_session(&poller, "credits.at", &scene_rows()).unwrap();
    // 25 (move incl. fused support) + grounded + support + 4 held reads = 31.
    s.call("_ag_char_input", &[Int(1), Int(0)], ResultKind::Unit)
        .unwrap();
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0);
    assert_eq!(s.int("g_v", &[]).unwrap(), 1);
}
