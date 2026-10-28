//! G-base contract tests (PLAN-829 T-03 / V-02).
//!
//! Run scoped: `cargo test --locked --test prototype_contract <filter>`.
//! All tests are pure-state (no GPU). The frozen oracle
//! (`research/sample-manifest.md`) defines the only tolerances used here:
//! position agreement 1e-3, exact event-sequence equality.

use agp::core::demo::{
    canonical_action, run_canonical, DemoAction, DemoDriver, DemoEvent, ObjRef, DEMO_SEED,
};
use agp::core::id::ObjectId;
use agp::core::runtime::{Runtime, TickEvent};
use agp::core::scene::{Color, Scene, SceneChange};
use agp::core::QueueError;
use agp::renderer::capture::sha256_hex;
use glam::Vec3;

/// Oracle tolerance from research/sample-manifest.md §1.5 (position compare).
const POS_TOL: f32 = 1.0e-3;

// ---------------------------------------------------------------------------
// shared_runtime — deterministic replay (V-02 headline case)
// ---------------------------------------------------------------------------

#[test]
fn shared_runtime_replay_twice_agrees() {
    let ticks = 240;
    let (rt_a, events_a) = run_canonical(ticks);
    // Second runtime built separately (fresh session, same seed/stream).
    let (rt_b, events_b) = run_canonical(ticks);

    // Scene snapshots: session-independent canonical strings must be equal.
    let snap_a = rt_a.scene().snapshot();
    let snap_b = rt_b.scene().snapshot();
    assert_eq!(snap_a, snap_b, "scene snapshots diverged across replays");

    // Event payload sequences: EXACTLY equal (manifest §1.5).
    let pay_a: Vec<&DemoEvent> = events_a.iter().collect();
    let pay_b: Vec<&DemoEvent> = events_b.iter().collect();
    assert_eq!(pay_a, pay_b, "event sequences diverged across replays");
    assert!(!events_a.is_empty());

    // Mover positions agree within the oracle tolerance.
    let x = |events: &[DemoEvent]| {
        events
            .iter()
            .filter_map(|e| match e {
                DemoEvent::Moved { x } => Some(*x),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    for (xa, xb) in x(&events_a).iter().zip(x(&events_b).iter()) {
        assert!(
            (xa - xb).abs() <= POS_TOL,
            "position diverged: {xa} vs {xb}"
        );
    }

    // Session isolation: the two runtimes are different sessions, yet
    // replay-equivalence holds — sessions never enter the equality contract.
    assert_ne!(rt_a.session(), rt_b.session());
}

#[test]
fn shared_runtime_changelog_replay_matches_live_state() {
    let ticks = 120;
    let (rt, _events) = run_canonical(ticks);
    let live = rt.scene().snapshot();

    // Rebuild purely from the recorded change log — no driver, no actions.
    let replayed = rt.log().replay();
    assert_eq!(
        live,
        replayed.snapshot(),
        "change-log replay does not reproduce the live final scene"
    );
    assert!(rt.log().len() > 0);
}

#[test]
fn shared_runtime_reset_and_replay_matches() {
    let ticks = 60;
    let (mut rt, events_first) = run_canonical(ticks);
    let snap_first = rt.scene().snapshot();

    rt.reset(DEMO_SEED);
    assert_eq!(rt.tick(), 0, "reset must rewind the clock");
    assert_eq!(rt.scene().live_count(), 0, "reset must clear the scene");
    assert!(rt.log().is_empty(), "reset must clear the change log");

    let mut driver = DemoDriver::default();
    let mut events_second = Vec::new();
    for tick in 0..ticks {
        let step = rt.step(&mut driver, vec![canonical_action(tick)]);
        events_second.extend(step.into_iter().map(|e| e.payload));
    }
    assert_eq!(
        snap_first,
        rt.scene().snapshot(),
        "same seed + same stream after reset must reproduce the same state"
    );
    assert_eq!(events_first, events_second);
}

#[test]
fn shared_runtime_rng_draws_are_reproducible() {
    // The decoy's color carries an rng jitter; Spawned events carry it.
    let (_, events) = run_canonical(20);
    let j1: Vec<f32> = events
        .iter()
        .filter_map(|e| match e {
            DemoEvent::Spawned { jitter, .. } => Some(*jitter),
            _ => None,
        })
        .collect();
    let (_, events2) = run_canonical(20);
    let j2: Vec<f32> = events2
        .iter()
        .filter_map(|e| match e {
            DemoEvent::Spawned { jitter, .. } => Some(*jitter),
            _ => None,
        })
        .collect();
    assert_eq!(j1, j2, "rng draws diverged for same seed");
    assert_eq!(j1.len(), 1);
}

// ---------------------------------------------------------------------------
// Identity / generation cases
// ---------------------------------------------------------------------------

#[test]
fn identity_stale_id_mutation_rejected_without_altering_scene() {
    let (mut rt, _events) = run_canonical(20); // decoy spawned @5, destroyed @8
    let mut driver = DemoDriver::default();
    // Re-run to get a decoy handle (destroyed during the run).
    let mut rt2 = Runtime::new(agp::core::demo::DEMO_DT, DEMO_SEED);
    let mut decoy: Option<ObjectId> = None;
    for tick in 0..12u64 {
        let _ = rt2.step(&mut driver, vec![canonical_action(tick)]);
        if tick == 6 {
            decoy = driver.decoy;
        }
    }
    let stale = decoy.expect("decoy must exist at tick 6");
    assert!(
        rt2.scene().validate_id(stale).is_err(),
        "destroyed id must be stale"
    );

    let before = rt2.scene().snapshot();
    let errs = rt2.submit(&[SceneChange::SetColor(stale, Color::rgb(0.0, 0.0, 0.0))]);
    assert_eq!(errs.len(), 1, "late mutation must be rejected");
    assert_eq!(
        before,
        rt2.scene().snapshot(),
        "rejected change must not alter the scene"
    );
    let _ = &mut rt;
}

#[test]
fn identity_double_destroy_rejected() {
    let mut scene = Scene::new();
    assert!(scene
        .submit(&[SceneChange::Spawn {
            transform: Default::default(),
            color: Color::rgb(1.0, 1.0, 1.0),
        }])
        .is_empty());
    let id = scene.objects()[0].id;
    assert!(scene.submit(&[SceneChange::Destroy(id)]).is_empty());
    let errs = scene.submit(&[SceneChange::Destroy(id)]);
    assert_eq!(errs.len(), 1, "double destroy must be rejected");
}

#[test]
fn identity_reuse_across_reset_gets_new_generation_and_old_fails() {
    let (mut rt, _) = run_canonical(30);
    let old_id = rt.scene().objects()[0].id;
    rt.reset(DEMO_SEED);
    // Old reference must fail validation against the new session's scene.
    assert!(rt.scene().validate_id(old_id).is_err());

    // Re-run: same index is minted again but under a new session, and the
    // runtime-level contract (snapshot equality) still holds.
    let mut driver = DemoDriver::default();
    for tick in 0..30u64 {
        let _ = rt.step(&mut driver, vec![canonical_action(tick)]);
    }
    let new_id = rt.scene().objects()[0].id;
    assert_eq!(old_id.index(), new_id.index(), "allocator reuses slots");
    assert_ne!(
        old_id.session(),
        new_id.session(),
        "reset must mint identities in a new session"
    );
    assert!(rt.scene().validate_id(old_id).is_err());
}

// ---------------------------------------------------------------------------
// Late / expired event isolation across reset
// ---------------------------------------------------------------------------

#[test]
fn events_late_events_from_previous_session_rejected() {
    let mut rt: Runtime<DemoAction, DemoEvent> = Runtime::new(agp::core::demo::DEMO_DT, DEMO_SEED);
    let mut driver = DemoDriver::default();
    let step = rt.step(&mut driver, vec![DemoAction::None]);
    let stale_event: TickEvent<DemoEvent> = step[0].clone();
    let stale_session = stale_event.session;
    assert_eq!(stale_session, rt.session());

    rt.reset(DEMO_SEED);
    assert_ne!(stale_session, rt.session());

    // Delivering a pre-reset event into the new session must be rejected.
    let result = rt.accept_external_event(&stale_event);
    assert_eq!(result.unwrap_err(), QueueError::StaleSession);

    // And the new run's events carry only the new session.
    let step_events = rt.step(&mut driver, vec![DemoAction::None]);
    assert!(step_events.iter().all(|e| e.session == rt.session()));
}

#[test]
fn events_carry_tick_ids_monotonic() {
    let mut rt: Runtime<DemoAction, DemoEvent> = Runtime::new(agp::core::demo::DEMO_DT, DEMO_SEED);
    let mut driver = DemoDriver::default();
    let mut stamped: Vec<TickEvent<DemoEvent>> = Vec::new();
    for tick in 0..30u64 {
        let step = rt.step(&mut driver, vec![canonical_action(tick)]);
        stamped.extend(step);
    }
    assert!(stamped.windows(2).all(|w| w[0].tick <= w[1].tick));
    assert!(stamped.windows(2).all(|w| w[0].seq < w[1].seq));
    assert!(stamped.iter().all(|e| e.session == rt.session()));
}

// ---------------------------------------------------------------------------
// Local-edit cases: untouched identities keep bindings, no rebuild signal
// ---------------------------------------------------------------------------

#[test]
fn local_edit_untouched_object_keeps_resource_binding_and_revision() {
    // Build a scene with two objects, both with resource bindings; edit only
    // object A; B's binding, revision and touch-set membership are unchanged.
    let mut rt: Runtime<DemoAction, DemoEvent> = Runtime::new(agp::core::demo::DEMO_DT, DEMO_SEED);
    let mut driver = DemoDriver::default();
    let _ = rt.step(&mut driver, vec![DemoAction::None]); // spawns mover
    let _ = rt.step(&mut driver, vec![DemoAction::None]);
    let _ = rt.step(&mut driver, vec![DemoAction::None]);
    let _ = rt.step(&mut driver, vec![DemoAction::None]);
    let _ = rt.step(&mut driver, vec![DemoAction::None]); // decoy alive @ tick 5

    let mover = rt.scene().objects()[0].id;
    let decoy = rt.scene().objects().last().unwrap().id;
    let res_a = mover; // resource identity: the object's own id as cache owner
    let res_b = decoy;
    assert!(rt
        .submit(&[SceneChange::SetResource(mover, Some(res_a))])
        .is_empty());
    assert!(rt
        .submit(&[SceneChange::SetResource(decoy, Some(res_b))])
        .is_empty());
    let rev_b_before = rt
        .scene()
        .objects()
        .iter()
        .find(|o| o.id == decoy)
        .unwrap()
        .resource_revision;
    let log_len_before = rt.log().len();
    rt.clear_touch_log(); // new observation window for the edit below

    // Local edit: mover transform only.
    assert!(rt
        .submit(&[SceneChange::SetTransform(
            mover,
            agp::core::scene::Transform {
                translation: Vec3::new(7.0, 0.0, 0.0),
                ..Default::default()
            },
        )])
        .is_empty());

    let decoy_state = rt.scene().objects().iter().find(|o| o.id == decoy).unwrap();
    assert_eq!(
        decoy_state.resource,
        Some(res_b),
        "untouched object must keep its binding"
    );
    assert_eq!(
        decoy_state.resource_revision, rev_b_before,
        "untouched object must not show a rebuild signal"
    );
    assert!(
        !rt.last_touched().contains(&decoy),
        "untouched object must not appear in the touch set"
    );
    assert!(rt.last_touched().contains(&mover));
    assert_eq!(
        rt.log().len(),
        log_len_before + 1,
        "only the one edit is logged"
    );
}

// ---------------------------------------------------------------------------
// Counterexample guards: prove the detectors actually fire
// ---------------------------------------------------------------------------

#[test]
fn counterexample_corrupted_changelog_replay_detected() {
    let ticks = 60;
    let (rt, _events) = run_canonical(ticks);
    let live = rt.scene().snapshot();

    // Corrupt the log: flip the LAST SetTransform translation (earlier ones
    // are overwritten by later ticks — only the final value survives).
    let mut corrupted = rt.log().clone();
    let target = corrupted
        .entries
        .iter_mut()
        .rev()
        .find(|e| matches!(e.change, SceneChange::SetTransform(_, _)))
        .expect("demo must contain transform edits");
    target.change = SceneChange::SetTransform(
        match target.change {
            SceneChange::SetTransform(id, _) => id,
            _ => unreachable!(),
        },
        agp::core::scene::Transform {
            translation: Vec3::new(999.0, 999.0, 999.0),
            ..Default::default()
        },
    );

    let replayed = corrupted.replay();
    assert_ne!(
        live,
        replayed.snapshot(),
        "corrupted replay must NOT match live state — detector is vacuous otherwise"
    );
}

#[test]
fn counterexample_divergent_action_stream_detected() {
    let ticks = 120;
    let (rt_a, events_a) = run_canonical(ticks);
    // Same runtime/seed, but with one MoveRight swapped to MoveLeft at a
    // boundary tick: positions must diverge beyond the oracle tolerance.
    let mut rt_b: Runtime<DemoAction, DemoEvent> =
        Runtime::new(agp::core::demo::DEMO_DT, DEMO_SEED);
    let mut driver = DemoDriver::default();
    let mut events_b = Vec::new();
    for tick in 0..ticks {
        // Fully inverted stream: clamped phases end at the OPPOSITE wall.
        let action = match canonical_action(tick) {
            DemoAction::MoveLeft => DemoAction::MoveRight,
            DemoAction::MoveRight => DemoAction::MoveLeft,
            a => a,
        };
        let step = rt_b.step(&mut driver, vec![action]);
        events_b.extend(step.into_iter().map(|e| e.payload));
    }
    assert_ne!(
        rt_a.scene().snapshot(),
        rt_b.scene().snapshot(),
        "divergent action stream must change the final state"
    );
    // At least one Moved position differs beyond tolerance.
    let xs = |ev: &[DemoEvent]| -> Vec<f32> {
        ev.iter()
            .filter_map(|e| match e {
                DemoEvent::Moved { x } => Some(*x),
                _ => None,
            })
            .collect()
    };
    let (xa, xb) = (xs(&events_a), xs(&events_b));
    let diverged = xa
        .iter()
        .zip(xb.iter())
        .any(|(a, b)| (a - b).abs() > POS_TOL);
    assert!(diverged, "position detector must fire for divergent input");
}

#[test]
fn counterexample_wrong_seed_detected() {
    let (_, events_a) = run_canonical(20);
    let mut rt_b: Runtime<DemoAction, DemoEvent> =
        Runtime::new(agp::core::demo::DEMO_DT, DEMO_SEED ^ 0xFFFF);
    let mut driver = DemoDriver::default();
    let mut events_b = Vec::new();
    for tick in 0..20u64 {
        let step = rt_b.step(&mut driver, vec![canonical_action(tick)]);
        events_b.extend(step.into_iter().map(|e| e.payload));
    }
    // The decoy's Spawned event carries the rng jitter; a different seed must
    // produce a different jitter (the decoy itself is gone from the final
    // scene, so the EVENT stream is the correct detector surface).
    let jitter = |ev: &[DemoEvent]| -> Vec<f32> {
        ev.iter()
            .filter_map(|e| match e {
                DemoEvent::Spawned { jitter, .. } => Some(*jitter),
                _ => None,
            })
            .collect()
    };
    assert_eq!(jitter(&events_a).len(), 1);
    assert_ne!(jitter(&events_a), jitter(&events_b));
}

// ---------------------------------------------------------------------------
// Reset semantics
// ---------------------------------------------------------------------------

#[test]
fn reset_clears_pending_actions_and_emitted_events() {
    let mut rt: Runtime<DemoAction, DemoEvent> = Runtime::new(agp::core::demo::DEMO_DT, DEMO_SEED);
    let mut driver = DemoDriver::default();
    let step_events = rt.step(&mut driver, vec![DemoAction::MoveRight]);
    assert!(!step_events.is_empty());
    assert!(rt.tick() > 0);

    rt.reset(DEMO_SEED);
    assert_eq!(rt.tick(), 0);
    assert_eq!(rt.scene().live_count(), 0);
    assert!(rt.log().is_empty());
    // Next step runs fresh: tick 1 re-spawns the mover (tick ids restart).
    let step_events = rt.step(&mut driver, vec![DemoAction::None]);
    assert!(step_events
        .iter()
        .all(|e| e.tick == 1 && e.session == rt.session()));
}

// ---------------------------------------------------------------------------
// Snapshot hash stability (probe-side cross-check helper)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_hash_is_stable_across_runs() {
    let (rt_a, _) = run_canonical(90);
    let (rt_b, _) = run_canonical(90);
    assert_eq!(
        sha256_hex(rt_a.scene().snapshot().as_bytes()),
        sha256_hex(rt_b.scene().snapshot().as_bytes()),
        "snapshot hashes must be reproducible for the canonical run"
    );
}

// Guard against accidentally-empty event streams (vacuous equality).
#[test]
fn demo_event_stream_nonempty_and_typed() {
    let (_, events) = run_canonical(60);
    assert!(events.len() >= 60, "one Moved event per tick expected");
    assert!(events
        .iter()
        .any(|e| matches!(e, DemoEvent::Spawned { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, DemoEvent::Destroyed { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, DemoEvent::Bounced { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, DemoEvent::ResourceBound { .. })));
    let _ = ObjRef {
        index: 0,
        generation: 0,
    };
}
