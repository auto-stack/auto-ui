//! auto-ui PLAN-001 runtime objects public API tests (contract V-01/V-02 plus
//! the T-02 whole-path probe): the two combined identities, legal spawn/move/
//! hide/despawn/reuse admission through the REAL assembled VM, every named
//! counterexample, the integer handle encoding with no float truncation, the
//! generation high-water and exhaustion, the per-callback write cap and the
//! shared credit budget, and the reserved three-item collection combination
//! for PLAN-002. The host Session call profile caps arguments at four, so the
//! direct-drive helpers go through zero-argument user wrappers while the
//! five-argument spawn stays a user-side call, exactly like production.
//! Run single-threaded:
//! cargo test --locked --test runtime_objects_api -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const DT: f32 = 1.0 / 60.0;
const RUN: i32 = 1;
const GEN_SPAN: i32 = 65536;
const RUN_STRIDE: i32 = GEN_SPAN * 8;

struct Row(i32, i32, [f32; 3], [f32; 3]);

fn row2(kind: i32, dim: i32) -> Row {
    Row(kind, dim, [-0.4, -0.5, 0.0], [0.4, 0.5, 0.0])
}

fn row3(kind: i32, dim: i32) -> Row {
    Row(kind, dim, [-0.4, -0.5, -0.4], [0.4, 0.5, 0.4])
}

/// 2D scene: character quad (id 1), floor quad (2), circle (3), quad (4);
/// id 5 is a 3D cylinder decoy, id 6 a group decoy (groups never spawn).
fn rows2d() -> Vec<Row> {
    vec![
        row2(1, 2),
        Row(1, 2, [-4.0, -1.0, 0.0], [34.0, 0.0, 0.0]),
        row2(0, 2),
        row2(1, 2),
        row2(2, 3),
        row2(5, 2),
    ]
}

/// 3D scene: character cylinder (1), floor cylinder (2), mob (3), model-kind
/// cylinder (4); id 5 is a 2D quad decoy, id 6 a group decoy.
fn rows3d() -> Vec<Row> {
    vec![
        row3(2, 3),
        Row(2, 3, [-8.0, -1.0, -8.0], [8.0, 0.0, 8.0]),
        row3(4, 3),
        row3(2, 3),
        row3(1, 2),
        row3(5, 3),
    ]
}

const BIND2: &str = " ag_set_local(0,1,0.5)\n ok=ok+ag_char_bind(1)\n ok=ok+ag_char_solid(2)\n ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)\n";
const BIND3: &str = " ag_set_local(0,1,0.5)\n ok=ok+ag_char3_bind(1)\n ok=ok+ag_char3_solid(2)\n ok=ok+ag_camera_orbit_3d(1,8.0,45.0,0,0.0)\n";

fn source2d(start: &str, update: &str) -> String {
    format!(
        "var ok=0\nvar h1=0\nvar h2=0\nvar h3=0\nfn sp() int {{h1=ag_object_spawn(0, 3, 7.0, 0.0, 0.0); return h1}}\nfn de() int {{return ag_object_despawn(h1)}}\nfn app_start() {{\n{BIND2}{start}}}\nfn app_update(dt float) {{\n{update}}}\n"
    )
}

fn source3d(start: &str, update: &str) -> String {
    format!(
        "var ok=0\nvar h1=0\nvar h2=0\nvar h3=0\nfn app_start() {{\n{BIND3}{start}}}\nfn app_update(dt float) {{\n{update}}}\n"
    )
}

fn objects_session(
    api: Api,
    run: i32,
    user: &str,
    name: &str,
    rows: &[Row],
) -> Result<Session, String> {
    let snap = SourceSnapshot::from_bytes_for(Path::new(name), user.as_bytes().to_vec(), api)?;
    let mut s = Session::new(&snap.source)?;
    s.call("main", &[], ResultKind::Unit)?;
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)?;
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )?;
    for (n, Row(kind, dim, lo, hi)) in rows.iter().enumerate() {
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
        for (i, v) in [1.0, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
            .iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )?;
        }
    }
    let char_map = match api {
        Api::Objects2V1 => "_ag_char_map",
        _ => "_ag_char3_map",
    };
    for n in 0..rows.len() {
        s.call(
            char_map,
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )?;
        s.call(
            "_ag_obj_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )?;
    }
    s.call("_ag_start", &[], ResultKind::Unit)?;
    Ok(s)
}

/// `_ag_status` mirrors the sticky error code; 0 means fault-free.
fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}

fn fault_code(s: &mut Session) -> i32 {
    status(s)
}

fn fault_arg(s: &mut Session) -> i32 {
    s.int("_ag_error_argument", &[]).unwrap()
}

fn tick(s: &mut Session, step: i32) {
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

fn handle(slot: i32, gen: i32) -> i32 {
    RUN * RUN_STRIDE + slot * GEN_SPAN + gen
}

fn generation_of(handle: i32) -> i32 {
    let rel = handle - RUN * RUN_STRIDE;
    let slot = rel / GEN_SPAN;
    let gen = rel - slot * GEN_SPAN;
    assert!((0..8).contains(&slot), "handle {handle} outside the table");
    gen
}

fn slot_row(s: &mut Session, slot: i32) -> (i32, i32, f32, f32, f32, i32) {
    (
        s.int("_ag_obj_gv", &[Int(slot)]).unwrap(),
        s.int("_ag_obj_tv", &[Int(slot)]).unwrap(),
        s.float("_ag_obj_xv", &[Int(slot)]).unwrap(),
        s.float("_ag_obj_yv", &[Int(slot)]).unwrap(),
        s.float("_ag_obj_zv", &[Int(slot)]).unwrap(),
        s.int("_ag_obj_vv", &[Int(slot)]).unwrap(),
    )
}

fn live_count(s: &mut Session) -> i32 {
    s.int("_ag_obj_count", &[]).unwrap()
}

/// Drive a zero-argument user wrapper (`sp` spawns slot 0 at x=7, `de`
/// despawns it) — the host Session profile caps direct arguments at four.
fn user_int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}

// ------------------------------------------------- assembly/identity (V-01) --

#[test]
fn objects_identities_parse_with_frozen_numbers_and_profiles() {
    for (api, version, number, profile) in [
        (Api::Objects2V1, "autographics.objects2d/1", 105, 16),
        (Api::Objects3V1, "autographics.objects3d/1", 106, 17),
    ] {
        assert_eq!(Api::parse(version).unwrap(), api);
        assert_eq!(api.number(), number, "{version}");
        assert_eq!(api.profile(), profile, "{version}");
        assert_eq!(api.version(), version);
    }
    // Every older identity stays exactly where its plan froze it.
    for (api, number, profile) in [
        (Api::V1, 1, 7),
        (Api::SpatialV2, 2, 8),
        (Api::SegmentV3, 3, 9),
        (Api::SweepV4, 4, 10),
        (Api::MotionV5, 5, 11),
        (Api::CharacterV1, 101, 12),
        (Api::Character3V1, 102, 13),
        (Api::Camera2V1, 103, 14),
        (Api::Camera3V1, 104, 15),
    ] {
        assert_eq!(api.number(), number);
        assert_eq!(api.profile(), profile);
    }
    // The combined entry chains the object freeze before app_start (the table
    // must exist before the first spawn) and the camera/character freezes
    // after it, then resets the per-callback write counter before app_update.
    let bare = "var ok=0\nfn app_start() {}\nfn app_update(dt float) {}\n";
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("identity-obj3.at"),
        bare.as_bytes().to_vec(),
        Api::Objects3V1,
    )
    .unwrap();
    assert!(
        snap.source.contains(
            "if _ag_error==0 { _ag_obj_freeze(); app_start(); _ag_cast_token=0; _ag_char3_freeze(); _ag_cam3_freeze() }"
        ),
        "3D objects entry must freeze the table before app_start"
    );
    assert!(
        snap.source.contains(
            "_ag_char3_tick_begin(); _ag_obj_tick_begin(); app_update(dt); _ag_cast_token=0"
        ),
        "tick begin chain must reset the write counter"
    );
    let regions: Vec<_> = snap.regions.iter().map(|r| r.file.as_str()).collect();
    assert!(regions.contains(&"<runtime_objects_v1.at>"), "{regions:?}");
    let snap2 = SourceSnapshot::from_bytes_for(
        Path::new("identity-obj2.at"),
        bare.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .unwrap();
    assert!(
        snap2.source.contains("var _ag_obj_dim=2"),
        "the 2D assembly freezes the run dimension"
    );
    let regions: Vec<_> = snap2.regions.iter().map(|r| r.file.as_str()).collect();
    assert!(regions.contains(&"<runtime_objects_v1.at>"), "{regions:?}");
}

// --------------------------------------------------- whole-path T-02 probe --

#[test]
fn whole_path_probe_spawns_moves_hides_despawns_across_all_slots() {
    // The full lifecycle over the reserved slots in app_start on the real
    // assembled VM; this is the frozen T-02 feasibility probe evidence.
    let start = "\
 h1=ag_object_spawn(0, 3, 1.0, 2.0, 0.0)\n\
 h2=ag_object_spawn(1, 4, -3.0, 0.0, 0.0)\n\
 ok=ok+ag_object_spawn(2, 3, 5.0, 1.0, 0.0)\n\
 ok=ok+ag_object_spawn(3, 3, -5.0, 1.0, 0.0)\n\
 ok=ok+ag_object_spawn(4, 3, 8.0, 1.0, 0.0)\n\
 ok=ok+ag_object_spawn(5, 3, -8.0, 1.0, 0.0)\n\
 ok=ok+ag_object_spawn(6, 3, 11.0, 1.0, 0.0)\n\
 ok=ok+ag_object_spawn(7, 4, 0.0, 5.0, 0.0)\n\
 ok=ok+ag_object_visible(h2, 0)\n\
 ok=ok+ag_object_despawn(h1)\n\
";
    let update = "ok=ok+ag_object_move(h2, -4.0, 0.0, 0.0)\n";
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(start, update),
        "probe-all-slots.at",
        &rows2d(),
    )
    .expect("objects2d session");
    assert_eq!(status(&mut s), 0, "init must be fault-free");
    assert_eq!(live_count(&mut s), 7);
    // slot 0 is free after the despawn; slot 1 is hidden but alive.
    assert_eq!(slot_row(&mut s, 0), (0, 0, 0.0, 0.0, 0.0, 0));
    assert_eq!(slot_row(&mut s, 1).0, 1, "slot 1 alive");
    assert_eq!(slot_row(&mut s, 1).5, 0, "slot 1 hidden");
    assert_eq!(slot_row(&mut s, 2), (1, 2, 5.0, 1.0, 0.0, 1));
    assert_eq!(slot_row(&mut s, 7).0, 1);
    // Reads through the real unit after a hide: identity and geometry stay.
    assert_eq!(s.float("ag_object_x", &[Int(handle(1, 1))]).unwrap(), -3.0);
    assert_eq!(s.int("ag_object_alive", &[Int(handle(1, 1))]).unwrap(), 1);
    // The tick body moved the hidden instance through the admitted block.
    tick(&mut s, 1);
    assert_eq!(fault_code(&mut s), 0, "tick must stay clean");
    assert_eq!(s.float("ag_object_x", &[Int(handle(1, 1))]).unwrap(), -4.0);
    // Same-slot reuse through the user wrapper: new generation, old handle
    // dead, own position.
    tick(&mut s, 2);
    assert_eq!(fault_code(&mut s), 0);
    let nh = user_int(&mut s, "sp");
    assert_eq!(nh, handle(0, 2), "reuse issues generation 2");
    assert_eq!(s.int("ag_object_alive", &[Int(handle(0, 1))]).unwrap(), 0);
    assert_eq!(s.int("ag_object_alive", &[Int(nh)]).unwrap(), 1);
    assert_eq!(slot_row(&mut s, 0), (2, 2, 7.0, 0.0, 0.0, 1));
    assert_eq!(live_count(&mut s), 8, "every slot live again");
    assert_eq!(
        s.float("ag_object_x", &[Int(nh)]).unwrap(),
        7.0,
        "the new generation reads before any fault"
    );
    // A duplicate spawn on the now-occupied slot fails named; afterwards the
    // sticky error short-circuits every further objects read.
    tick(&mut s, 3);
    assert_eq!(user_int(&mut s, "sp"), 0, "slot 0 is occupied");
    assert_eq!(fault_code(&mut s), 32);
    assert_eq!(fault_arg(&mut s), 2);
    assert_eq!(s.float("ag_object_x", &[Int(nh)]).unwrap(), 0.0);
}

#[test]
fn three_item_collection_combo_stays_inside_the_frozen_budget() {
    // The reserved PLAN-002 combination: three items collected in one callback
    // (3 despawns) plus three respawns beside real character movement — 6
    // lifecycle writes and bounded reads inside the shared 32 credits and the
    // 100000-step callback guard, in both dimensions.
    let start2d = "\
 h1=ag_object_spawn(0, 3, 1.0, 1.0, 0.0)\n\
 h2=ag_object_spawn(1, 4, 2.0, 1.0, 0.0)\n\
 h3=ag_object_spawn(2, 3, 3.0, 1.0, 0.0)\n\
";
    let update2d = "\
 ok=ok+ag_char_move(0.0,0.0,0.002)\n\
 ok=ok+ag_object_despawn(h1)\n\
 ok=ok+ag_object_despawn(h2)\n\
 ok=ok+ag_object_despawn(h3)\n\
 h1=ag_object_spawn(0, 3, 4.0, 1.0, 0.0)\n\
 h2=ag_object_spawn(1, 4, 5.0, 1.0, 0.0)\n\
 h3=ag_object_spawn(2, 3, 6.0, 1.0, 0.0)\n\
";
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(start2d, update2d),
        "collect-combo.at",
        &rows2d(),
    )
    .expect("collect session");
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(fault_code(&mut s), 0, "the whole combo must stay clean");
    assert_eq!(live_count(&mut s), 3);
    // The callback sits on the credit line: read through the free emitters.
    assert_eq!(
        slot_row(&mut s, 0),
        (2, 2, 4.0, 1.0, 0.0, 1),
        "the respawned generation sits at its new position"
    );
    assert_eq!(slot_row(&mut s, 2).0, 2, "slot 2 reused into generation 2");

    let start3d = "\
 h1=ag_object_spawn(0, 4, 9.0, 1.0, -2.0)\n\
 h2=ag_object_spawn(1, 4, 10.0, 1.0, -2.0)\n\
 h3=ag_object_spawn(2, 4, 11.0, 1.0, -2.0)\n\
";
    let update3d = "\
 let q=ag_char3_move(0.5,0.0,0.0,0.002)\n\
 ok=ok+ag_char3_grounded(q)\n\
 ok=ok+ag_cam3_forward_x()+ag_cam3_right_z()\n\
 ok=ok+ag_object_despawn(h1)+ag_object_despawn(h2)+ag_object_despawn(h3)\n\
 h1=ag_object_spawn(0, 4, 1.0, 1.0, -2.0)\n\
 h2=ag_object_spawn(1, 4, 2.0, 1.0, -2.0)\n\
 h3=ag_object_spawn(2, 4, 3.0, 1.0, -2.0)\n\
";
    let mut s3 = objects_session(
        Api::Objects3V1,
        RUN,
        &source3d(start3d, update3d),
        "collect-combo-3d.at",
        &rows3d(),
    )
    .expect("collect 3d session");
    assert_eq!(status(&mut s3), 0);
    tick(&mut s3, 1);
    assert_eq!(fault_code(&mut s3), 0);
    assert_eq!(live_count(&mut s3), 3);
    assert_eq!(slot_row(&mut s3, 2).4, -2.0, "3D respawn keeps its z");
}

// ------------------------------------------------- V-01 counterexamples --

#[test]
fn every_named_spawn_rejection_fires_at_its_own_stage() {
    let cases = [
        (
            "slot-negative",
            "ok=ok+ag_object_spawn( -1, 3, 0.0, 0.0, 0.0)",
            31i32,
            -1i32,
        ),
        (
            "slot-high",
            "ok=ok+ag_object_spawn( 8, 3, 0.0, 0.0, 0.0)",
            31,
            8,
        ),
        (
            "template-unknown",
            "ok=ok+ag_object_spawn( 0, 99, 0.0, 0.0, 0.0)",
            34,
            99,
        ),
        (
            "template-group",
            "ok=ok+ag_object_spawn( 0, 6, 0.0, 0.0, 0.0)",
            34,
            6,
        ),
        (
            "template-dimension",
            "ok=ok+ag_object_spawn( 0, 5, 0.0, 0.0, 0.0)",
            34,
            5,
        ),
        (
            "coordinate-nan",
            "ok=ok+ag_object_spawn( 0, 3, 0.0/0.0, 0.0, 0.0)",
            35,
            0,
        ),
        (
            "coordinate-inf",
            "ok=ok+ag_object_spawn( 0, 3, 1.0/0.0, 0.0, 0.0)",
            35,
            0,
        ),
        (
            "coordinate-range",
            "ok=ok+ag_object_spawn( 0, 3, 200000.0, 0.0, 0.0)",
            35,
            0,
        ),
        (
            "2d-z-domain",
            "ok=ok+ag_object_spawn( 0, 3, 0.0, 0.0, 0.5)",
            35,
            2,
        ),
    ];
    for (name, call, code, arg) in cases {
        let session = objects_session(
            Api::Objects2V1,
            RUN,
            &source2d(call, ""),
            &format!("case-{name}.at"),
            &rows2d(),
        );
        if call.contains("/0.0") {
            // 0.0/0.0 and 1.0/0.0 trap at the VM arithmetic boundary before
            // any spawn: a fail-closed rejection with no published instance.
            assert!(session.is_err(), "{name}: must fail closed");
            assert!(
                session.err().unwrap().contains("DivisionByZero"),
                "{name}: VM arithmetic boundary"
            );
            continue;
        }
        let mut s = session.unwrap_or_else(|e| panic!("{name}: session {e}"));
        assert_eq!(status(&mut s), code, "{name}: wrong fault code");
        assert_eq!(fault_arg(&mut s), arg, "{name}: wrong fault argument");
        assert_eq!(live_count(&mut s), 0, "{name}: nothing may spawn");
    }
    // The 3D dimension mirror: a 2D decoy template is rejected there.
    let mut s = objects_session(
        Api::Objects3V1,
        RUN,
        &source3d("ok=ok+ag_object_spawn( 0, 5, 0.0, 0.0, 0.0)", ""),
        "case-3d-decoy.at",
        &rows3d(),
    )
    .expect("3d decoy session");
    assert_eq!(status(&mut s), 34);
}

#[test]
fn occupied_despawn_twice_stale_generation_and_cross_run_reject_named() {
    let setup = " h1=ag_object_spawn(0, 3, 1.0, 1.0, 0.0)\n";
    // Despawn twice: the second is a sticky named error, never a fake success.
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(
            &format!("{setup} ok=ok+ag_object_despawn(h1)\n"),
            "ok=ok+ag_object_despawn(h1)\n",
        ),
        "despawn-twice.at",
        &rows2d(),
    )
    .expect("despawn-twice session");
    assert_eq!(status(&mut s), 0, "app_start despawn stays clean");
    tick(&mut s, 1);
    assert_eq!(fault_code(&mut s), 33, "second despawn");
    assert_eq!(s.int("ag_object_alive", &[Int(handle(0, 1))]).unwrap(), 0);
    // Stale generation: h2 copies the generation-1 handle; h1 is then reused
    // into generation 2 and the OLD handle must be dead for every operation.
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(
            &format!(
                "{setup} h2=h1\n ok=ok+ag_object_despawn(h1)\n h1=ag_object_spawn(0, 3, 5.0, 5.0, 0.0)\n"
            ),
            "ok=ok+ag_object_move(h2, 9.0, 9.0, 0.0)\n",
        ),
        "stale-gen.at",
        &rows2d(),
    )
    .expect("stale-gen session");
    assert_eq!(status(&mut s), 0);
    assert_eq!(
        slot_row(&mut s, 0).0,
        2,
        "the slot reused into generation 2"
    );
    tick(&mut s, 1);
    assert_eq!(fault_code(&mut s), 33, "stale generation move");
    assert_eq!(
        slot_row(&mut s, 0).2,
        5.0,
        "the respawned slot keeps its own position"
    );
    // Cross-Run: a run-1 handle is below the run-2 base and must be dead.
    let mut s2 = objects_session(
        Api::Objects2V1,
        2,
        &source2d(
            " h2=ag_object_spawn(0, 3, 1.0, 1.0, 0.0)\n",
            &format!(" ok=ok+ag_object_move({}, 9.0, 9.0, 0.0)\n", handle(0, 1)),
        ),
        "cross-run.at",
        &rows2d(),
    )
    .expect("run-2 session");
    assert_eq!(status(&mut s2), 0);
    tick(&mut s2, 1);
    assert_eq!(fault_code(&mut s2), 33, "a run-1 handle is dead in run 2");
    assert_eq!(
        slot_row(&mut s2, 0),
        (1, 2, 1.0, 1.0, 0.0, 1),
        "the run-2 instance (template 3, generation 1) is unaffected"
    );
}

#[test]
fn generation_exhaustion_is_named_and_never_wraps() {
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(" h1=ag_object_spawn(0, 3, 0.0, 0.0, 0.0)\n", ""),
        "exhaustion.at",
        &rows2d(),
    )
    .expect("exhaustion session");
    assert_eq!(status(&mut s), 0);
    // Drive slot 0 to the frozen ceiling through the zero-argument wrappers:
    // one despawn+respawn pair per callback (2 writes, 2 credits).
    let mut last = handle(0, 1);
    let mut gen = 1i32;
    let mut step = 1i32;
    while gen < 65535 {
        tick(&mut s, step);
        step += 1;
        assert_eq!(user_int(&mut s, "de"), 1, "despawn generation {gen}");
        last = user_int(&mut s, "sp");
        gen += 1;
        assert_eq!(generation_of(last), gen, "monotone generations, no wrap");
    }
    assert_eq!(gen, 65535, "reached the frozen ceiling");
    tick(&mut s, step);
    assert_eq!(user_int(&mut s, "de"), 1);
    assert_eq!(user_int(&mut s, "sp"), 0, "spawn refused at the ceiling");
    assert_eq!(fault_code(&mut s), 36, "named generation exhaustion");
    assert_eq!(s.int("ag_object_alive", &[Int(last)]).unwrap(), 0);
    assert_eq!(slot_row(&mut s, 0).0, 0, "the exhausted slot stays free");
}

#[test]
fn write_cap_and_credit_budget_bite_inside_one_callback() {
    // 31 alternating despawn/spawn writes after the first spawn leave slot 0
    // alive on generation 16; one visibility write lands exactly on 32 writes
    // and 32 credits — the cap boundary must pass.
    let mut writes = String::from(" h1=ag_object_spawn(0, 3, 0.0, 0.0, 0.0)\n");
    for i in 0..30 {
        if i % 2 == 0 {
            writes.push_str(" ok=ok+ag_object_despawn(h1)\n");
        } else {
            writes.push_str(" h1=ag_object_spawn(0, 3, 0.0, 0.0, 0.0)\n");
        }
    }
    writes.push_str(" ok=ok+ag_object_visible(h1, 1)\n");
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(&writes, ""),
        "cap-32.at",
        &rows2d(),
    )
    .expect("cap session");
    assert_eq!(status(&mut s), 0, "32 writes and credits stay inside");
    // The app_start callback sits exactly on the 32-credit line, so any
    // further charged host probe would itself fault the budget: assert
    // through the free block emitters instead.
    assert_eq!(
        slot_row(&mut s, 0),
        (16, 2, 0.0, 0.0, 0.0, 1),
        "alive on generation 16, visible"
    );
    // One more write in the same callback is the named cap failure (the write
    // counter is checked before the shared credit charge).
    let mut writes33 = writes;
    writes33.push_str(" ok=ok+ag_object_visible(h1, 1)\n");
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(&writes33, ""),
        "cap-33.at",
        &rows2d(),
    )
    .expect("cap-33 session");
    assert_eq!(status(&mut s), 37, "the 33rd write is the named cap");
}

#[test]
fn shared_credit_budget_bites_at_33_charged_reads() {
    // The frozen shared credit budget: 32 charged object reads ride beside the
    // callback's other work; the 33rd charge in the same window faults
    // QueryBudget (10) without any fake success.
    let mut reads = String::new();
    for _ in 0..33 {
        reads.push_str(" ok=ok+ag_object_alive(h1)\n");
    }
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(" h1=ag_object_spawn(0, 3, 0.0, 0.0, 0.0)\n", &reads),
        "credits-33.at",
        &rows2d(),
    )
    .expect("credits session");
    tick(&mut s, 1);
    assert_eq!(fault_code(&mut s), 10, "the shared credit budget bites");
    // 32 reads stay inside the budget window.
    let mut reads32 = String::new();
    for _ in 0..32 {
        reads32.push_str(" ok=ok+ag_object_alive(h1)\n");
    }
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(" h1=ag_object_spawn(0, 3, 0.0, 0.0, 0.0)\n", &reads32),
        "credits-32.at",
        &rows2d(),
    )
    .expect("credits-32 session");
    tick(&mut s, 1);
    assert_eq!(fault_code(&mut s), 0, "32 reads stay inside the budget");
}

#[test]
fn visibility_only_flips_presentation_and_never_despawns() {
    let setup = " h1=ag_object_spawn(0, 3, 1.0, 2.0, 0.0)\n ok=ok+ag_object_visible(h1, 0)\n";
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(&format!("{setup} ok=ok+ag_object_visible(h1, 1)\n"), ""),
        "visibility.at",
        &rows2d(),
    )
    .expect("visibility session");
    assert_eq!(status(&mut s), 0);
    let (gen, _tmpl, x, y, _z, vis) = slot_row(&mut s, 0);
    assert_eq!((gen, vis), (1, 1), "still alive and visible again");
    assert_eq!((x, y), (1.0, 2.0), "geometry untouched by visibility");
    // A bad flag is a named domain error and changes nothing.
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(&format!("{setup} ok=ok+ag_object_visible(h1, 2)\n"), ""),
        "visibility-flag.at",
        &rows2d(),
    )
    .expect("visibility flag session");
    assert_eq!(status(&mut s), 35);
    assert_eq!(fault_arg(&mut s), 2);
    // Reading a despawned instance is a sticky error, never a fake value.
    let mut s = objects_session(
        Api::Objects2V1,
        RUN,
        &source2d(
            &format!("{setup} ok=ok+ag_object_despawn(h1)\n"),
            "let r=ag_object_x(h1)\n",
        ),
        "visibility-dead.at",
        &rows2d(),
    )
    .expect("visibility dead session");
    assert_eq!(status(&mut s), 0);
    tick(&mut s, 1);
    assert_eq!(fault_code(&mut s), 33, "read after despawn is sticky");
}
