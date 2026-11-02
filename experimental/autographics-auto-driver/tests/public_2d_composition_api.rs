//! auto-ui PLAN-032 public_2d_composition_api: the autographics.components2d/1
//! trusted composition unit — whole-tick event snapshot and whole-batch
//! despawn — admitted against the real assembled profile. T-02 candidate
//! proofs live here first; T-03 hardens the full admission battery.
//! Run single-threaded:
//!   cargo test --locked --offline --test public_2d_composition_api -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

const DT: f32 = 1.0 / 60.0;

fn session(world_src: &str, name: &str, run: i32, scene_rows: i32) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world_src.as_bytes().to_vec(),
        Api::Components2V1,
    )
    .unwrap_or_else(|e| panic!("{name} admission: {e}"));
    let mut s = Session::new(&snap.source).unwrap_or_else(|e| panic!("{name} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(scene_rows)], ResultKind::Unit)
        .unwrap();
    // The smoke probe only touches the trigger snapshot rows; allocate them
    // directly (the full flow tests start real units via _ag_start).
    s.call("_ag_trg_freeze", &[], ResultKind::Unit).unwrap();
    s
}

fn tick(s: &mut Session, step: i32) -> i32 {
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
    s.int("_ag_status", &[]).unwrap()
}

/// The components2d assembly admits, exposes the snapshot + batch to user
/// source, and a snapshot on a tick with no events legally returns 0 for 1
/// credit; the profile/version numbers are the frozen 27/116.
#[test]
fn smoke_admit_and_empty_snapshot() {
    let world = r#"
fn app_start() {
}
fn app_update(dt float) {
}
fn probe() int {
 return ag_events_snapshot()
}
fn probe_run() int {return ag_events_run()}
fn probe_tick() int {return ag_events_tick()}
fn probe_count() int {return ag_events_count()}
"#;
    let mut s = session(world, "cmp-smoke.at", 1, 0);
    assert_eq!(s.int("_ag_profile", &[]).unwrap(), 27, "components2d profile id");
    assert_eq!(s.int("_ag_version", &[]).unwrap(), 116, "components2d api number");
    assert_eq!(tick(&mut s, 1), 0);
    assert_eq!(s.int("probe", &[]).unwrap(), 0, "empty tick snapshot is 0");
    assert_eq!(s.int("probe_count", &[]).unwrap(), 0);
    assert_eq!(s.int("probe_run", &[]).unwrap(), 1);
    assert_eq!(s.int("probe_tick", &[]).unwrap(), 1);
}

// ------------------------------------------------------ T-03 battery -----

struct Row(i32, i32, [f32; 3], [f32; 3]);

fn row2(kind: i32) -> Row {
    Row(kind, 2, [-0.4, -0.5, 0.0], [0.4, 0.5, 0.0])
}

/// Components2d session: character quad (1), floor quad (2), instance
/// template circle (3, parked far away at spawn time).
fn cmp_triggers_session(user: &str, name: &str) -> Session {
    let rows = [row2(1), row2(1), row2(0)];
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        user.as_bytes().to_vec(),
        Api::Components2V1,
    )
    .unwrap_or_else(|e| panic!("{name} admission: {e}"));
    let mut s = Session::new(&snap.source).unwrap_or_else(|e| panic!("{name} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(rows.len() as i32)], ResultKind::Unit)
        .unwrap();
    for (n, Row(kind, dim, lo, hi)) in rows.iter().enumerate() {
        // the character stays at the origin; every other row parks far away so
        // nothing overlaps the free character sweep
        let (tx, ty) = if n == 0 { (0.0f32, 0.0f32) } else { (60.0 - n as f32, -30.0) };
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit).unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [tx, ty, 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in [1.0, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
            .iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..rows.len() {
        for name in ["_ag_char_map", "_ag_obj_map", "_ag_trg_map"] {
            s.call(name, &[Int(n as i32 + 1), Int(n as i32)], ResultKind::Unit)
                .unwrap();
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        0,
        "{name} app_start fault {} arg {}",
        s.int("_ag_error_node", &[]).unwrap(),
        s.int("_ag_error_argument", &[]).unwrap()
    );
    s
}

/// Probe world: character at cx with a spawnable/bound instance; every
/// battery operation is a zero-arg or one-arg user wrapper (the session call
/// arity cap), mirroring production user code.
fn probe_world(extra: &str) -> String {
    format!(
        "var ok=0\nvar h1=0\nvar cx=0.5\nfn sp(v float) int {{h1=ag_object_spawn(0, 3, v, 0.5, 0.0); return h1}}\nfn movetox(v float) {{ok=ok+ag_char_move(v - cx, 0.0, 0.002); cx=v}}\nfn app_start() {{\n ok=ok+ag_char_bind(1)*1.0\n ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)*1.0\n h1=ag_object_spawn(0, 3, 40.0, 0.5, 0.0)\n ok=ok+ag_trigger_bind(0, 1, 1, h1, 0, 0.0)*1.0\n}}\nfn app_update(dt float) {{}}\nfn respawn() int {{h1=ag_object_spawn(0, 3, 40.0, 0.5, 0.0); return h1}}\nfn alive() int {{return ag_object_alive(h1)}}
fn dbg_cx() float {{return cx}}
fn dbg_px() float {{return ag_world(0, 0)}}
fn dbg_h1() int {{return h1}}
{extra}"
    )
}

fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}

/// One event: the character moves onto the bound instance; the event is
/// readable in the NEXT callback (evaluate rides the tick end).
fn make_event(s: &mut Session) {
    // the bound instance sits at x=40; walking the character into its box
    // makes the evaluate at this tick's end emit one enter event, readable
    // in the next callback
    s.call("movetox", &[Float(40.0)], ResultKind::Unit).unwrap();
    s.call("_ag_tick", &[Int(1), Float(DT)], ResultKind::Unit).unwrap();
    assert_eq!(status(s), 0, "event tick");
}

#[test]
fn snapshot_domain_faults_are_named() {
    let world = probe_world(
        "fn p_snap() int {return ag_events_snapshot()}\nfn p_next() int {return ag_event_next()}\nfn p_g(i int) int {return ag_event_snap_pair(i)}\n",
    );
    // second snapshot in the same callback
    let mut s = cmp_triggers_session(&world, "cmp-snap2.at");
    assert_eq!(s.int("p_snap", &[]).unwrap(), 0, "empty snapshot first");
    s.int("p_snap", &[]).unwrap();
    assert_eq!(status(&mut s), 58, "second snapshot named 58");
    drop(s);

    // cursor read after a snapshot
    let mut s = cmp_triggers_session(&world, "cmp-snap-then-next.at");
    assert_eq!(s.int("p_snap", &[]).unwrap(), 0);
    s.int("p_next", &[]).unwrap();
    assert_eq!(status(&mut s), 58, "next after snapshot named 58");
    drop(s);

    // snapshot after a cursor read
    let mut s = cmp_triggers_session(&world, "cmp-next-then-snap.at");
    s.int("p_next", &[]).unwrap();
    assert_eq!(status(&mut s), 0, "cursor read alone is legal");
    s.int("p_snap", &[]).unwrap();
    assert_eq!(status(&mut s), 58, "snapshot after cursor named 58");
    drop(s);

    // getters out of range
    let mut s = cmp_triggers_session(&world, "cmp-getter-oob.at");
    s.int("p_g", &[Int(-1)]).unwrap();
    assert_eq!(status(&mut s), 58, "negative index named 58");
    drop(s);
    let mut s = cmp_triggers_session(&world, "cmp-getter-oob2.at");
    s.int("p_g", &[Int(0)]).unwrap();
    assert_eq!(status(&mut s), 58, "index 0 with empty snapshot named 58");
    drop(s);
}

#[test]
fn batch_domain_faults_are_named() {
    let world = probe_world(
        "fn p_b(n int) int {return ag_object_despawn_batch(n, h1, 0, 0, 0, 0, 0, 0, 0)}\nfn p_dup() int {return ag_object_despawn_batch(2, h1, h1, 0, 0, 0, 0, 0, 0)}\n",
    );
    let mut s = cmp_triggers_session(&world, "cmp-b9.at");
    s.int("p_b", &[Int(9)]).unwrap();
    assert_eq!(status(&mut s), 59, "n=9 named 59");
    drop(s);
    let mut s = cmp_triggers_session(&world, "cmp-bneg.at");
    s.int("p_b", &[Int(-1)]).unwrap();
    assert_eq!(status(&mut s), 59, "n=-1 named 59");
    drop(s);
    // duplicates: named 59, no partial mutation (live-slot count unchanged;
    // the sticky error zeroes ag_object_alive, so count the slots instead)
    let mut s = cmp_triggers_session(&world, "cmp-bdup.at");
    assert_eq!(s.int("_ag_obj_count", &[]).unwrap(), 1, "live before");
    s.int("p_dup", &[]).unwrap();
    assert_eq!(status(&mut s), 59, "duplicate named 59");
    assert_eq!(
        s.int("_ag_obj_count", &[]).unwrap(),
        1,
        "still live after the refused batch"
    );
    drop(s);
}

#[test]
fn batch_stale_and_foreign_handles_use_object_identity_errors() {
    let world = probe_world(
        "fn p_b1(h int) int {return ag_object_despawn_batch(1, h, 0, 0, 0, 0, 0, 0, 0)}\n",
    );
    // stale generation: app_start spawned gen 1 on slot 0; destroy it and
    // respawn — the old handle is then stale for every consumer
    let mut s = cmp_triggers_session(&world, "cmp-bstale.at");
    let h1v = s.int("dbg_h1", &[]).unwrap();
    assert_eq!(h1v, 1 * 65536 * 8 + 1, "first generation handle");
    assert_eq!(s.int("p_b1", &[Int(h1v)]).unwrap(), 1, "destroy gen 1");
    s.int("sp", &[Float(40.0)]).unwrap();
    let h2 = s.int("dbg_h1", &[]).unwrap();
    assert_eq!(h2, 1 * 65536 * 8 + 2, "second generation handle");
    s.int("p_b1", &[Int(h2 - 1)]).unwrap();
    assert_eq!(status(&mut s), 33, "stale generation named 33");
    drop(s);
    // wrong-Run handle: run-2 base while the session runs 1
    let mut s2 = cmp_triggers_session(&world, "cmp-brun.at");
    let foreign = Int(2 * 65536 * 8 + 1);
    s2.int("p_b1", &[foreign]).unwrap();
    assert_eq!(status(&mut s2), 33, "foreign-run handle named 33");
    drop(s2);
}

#[test]
fn batch_destroys_all_eight_and_empty_batch_is_legal() {
    let world = "var ok=0\nvar hs=[0,0,0,0,0,0,0,0]\nfn app_start() {\n ok=ok+ag_char_bind(1)*1.0\n ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)*1.0\n for i in 0..8 {\n  hs[i]=ag_object_spawn(i, 3, 40.0 + i, 0.5, 0.0)\n }\n}\nfn app_update(dt float) {}\nfn p_b8() int {return ag_object_despawn_batch(8, hs[0], hs[1], hs[2], hs[3], hs[4], hs[5], hs[6], hs[7])}\nfn p_alive(i int) int {return ag_object_alive(hs[i])}\nfn p_batch0() int {return ag_object_despawn_batch(0, 0, 0, 0, 0, 0, 0, 0, 0)}\n";
    let mut s = cmp_triggers_session(world, "cmp-b8.at");
    assert_eq!(s.int("p_b8", &[]).unwrap(), 8, "batch returns n");
    for i in 0..8 {
        assert_eq!(s.int("p_alive", &[Int(i)]).unwrap(), 0, "slot {i} destroyed");
    }
    let mut s2 = cmp_triggers_session(world, "cmp-b0.at");
    assert_eq!(s2.int("p_batch0", &[]).unwrap(), 0, "empty batch legal");
    assert_eq!(status(&mut s2), 0);
}

#[test]
fn snapshot_fields_match_the_frozen_event_shape() {
    let world = probe_world(
        "fn p_snap() int {return ag_events_snapshot()}\nfn p_pair(i int) int {return ag_event_snap_pair(i)}\nfn p_phase(i int) int {return ag_event_snap_phase(i)}\nfn p_src(i int) int {return ag_event_snap_source(i)}\nfn p_gen(i int) int {return ag_event_snap_source_generation(i)}\nfn p_run() int {return ag_events_run()}\nfn p_b1(h int) int {return ag_object_despawn_batch(1, h, 0, 0, 0, 0, 0, 0, 0)}\n",
    );
    let mut s = cmp_triggers_session(&world, "cmp-equiv.at");
    make_event(&mut s);
    println!(
        "DBG cx={} px={} trg_count={} obj_count={}",
        s.float("dbg_cx", &[]).unwrap(),
        s.float("dbg_px", &[]).unwrap(),
        s.int("_ag_trg_count_v", &[]).unwrap(),
        s.int("_ag_obj_count", &[]).unwrap()
    );
    let n = s.int("p_snap", &[]).unwrap();
    assert_eq!(n, 1, "one enter event materialized");
    assert_eq!(s.int("p_pair", &[Int(0)]).unwrap(), 0, "pair 0");
    assert_eq!(s.int("p_phase", &[Int(0)]).unwrap(), 1, "enter phase");
    let h = s.int("p_src", &[Int(0)]).unwrap();
    assert_eq!(h, 1 * 65536 * 8 + 1, "source handle identity (slot 0, gen 1)");
    assert_eq!(s.int("p_gen", &[Int(0)]).unwrap(), 1, "binding generation");
    assert_eq!(s.int("p_run", &[]).unwrap(), 1, "run provenance");
    // The snapshot's handle still goes through the unchanged alive/generation
    // validation: a second destroy of the same handle is the objects unit's
    // own named 33 (an older value authorizes no new operation).
    assert_eq!(s.int("p_b1", &[Int(h)]).unwrap(), 1, "batch destroys via the recorded handle");
    s.int("p_b1", &[Int(h)]).unwrap();
    assert_eq!(status(&mut s), 33, "stale reuse named 33");
}

#[test]
fn earlier_profiles_keep_their_exact_composition() {
    // The cursor renames exist ONLY in the components2d bytes: every earlier
    // profile still defines the public cursor functions exactly once, and
    // none of them contain the composition unit.
    for api in [
        Api::Triggers2V1,
        Api::Sprites2V1,
        Api::Audio2V1,
        Api::Hud2V1,
        Api::Presentation2V1,
    ] {
        let (api_units, entry) = autographics_auto_driver::authoring::behavior::assembled_units_for(api).expect("assemble");
        assert_eq!(
            api_units.matches("fn ag_event_next(").count(),
            1,
            "{api:?}: public cursor untouched"
        );
        assert!(!api_units.contains("ag_events_snapshot"), "{api:?}: no composition unit");
        assert!(!api_units.contains("_ag_cmp_event_next"), "{api:?}: no renamed cursor");
        let _ = entry;
    }
    let (api_units, entry) = autographics_auto_driver::authoring::behavior::assembled_units_for(Api::Components2V1).expect("assemble");
    assert_eq!(api_units.matches("fn ag_event_next(").count(), 1, "components2d: one public wrapper");
    assert_eq!(
        api_units.matches("fn _ag_cmp_event_next(").count(),
        1,
        "components2d: renamed internal"
    );
    assert!(api_units.contains("fn ag_events_snapshot("));
    assert!(api_units.contains("fn ag_object_despawn_batch("));
    assert!(entry.contains("_ag_cmp_tick_begin();"));
}
