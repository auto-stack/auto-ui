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
