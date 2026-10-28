//! T-07 contract tests: verifier gate logic (pure) and lifecycle invariants.

use agp::core::runtime::{Runtime, TickEvent};
use agp::games::squash::{SquashAction, SquashDriver, SquashEvent, SQUASH_SEED};
use agp::host::gates;
use agp::render3d::collision::PlayerMobContact;
use glam::Vec3;
use std::time::Duration;

fn run_quiet(
    ticks: usize,
    classifier: Option<fn(Vec3) -> PlayerMobContact>,
) -> (Vec<TickEvent<SquashEvent>>, SquashDriver) {
    let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
    let mut driver = SquashDriver::new(SQUASH_SEED);
    if let Some(c) = classifier {
        driver.squash_classifier = c;
    }
    let mut events = Vec::new();
    for _ in 0..ticks {
        events.extend(rt.step(&mut driver, vec![SquashAction::default()]));
    }
    (events, driver)
}

fn run_golden_with(
    classifier: Option<fn(Vec3) -> PlayerMobContact>,
) -> (Vec<TickEvent<SquashEvent>>, SquashDriver) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/squash/golden_squash_trace.json");
    let bytes = std::fs::read(&path).expect("squash trace");
    let doc: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    let actions: Vec<[i8; 3]> = serde_json::from_value(doc["actions"].clone()).expect("actions");
    let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
    let mut driver = SquashDriver::new(SQUASH_SEED);
    if let Some(c) = classifier {
        driver.squash_classifier = c;
    }
    let mut events = Vec::new();
    for a in actions.iter() {
        events.extend(rt.step(
            &mut driver,
            vec![SquashAction {
                move_x: a[0],
                move_z: a[1],
                jump: a[2] != 0,
                retry: false,
                reset: false,
            }],
        ));
        if driver.dead {
            break;
        }
    }
    (events, driver)
}

fn run_golden(max_ticks: usize) -> (Vec<TickEvent<SquashEvent>>, SquashDriver) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/squash/golden_squash_trace.json");
    let bytes = std::fs::read(&path).expect("squash trace");
    let doc: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    let actions: Vec<[i8; 3]> = serde_json::from_value(doc["actions"].clone()).expect("actions");
    let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
    let mut driver = SquashDriver::new(SQUASH_SEED);
    let mut events = Vec::new();
    for a in actions.iter().take(max_ticks) {
        events.extend(rt.step(
            &mut driver,
            vec![SquashAction {
                move_x: a[0],
                move_z: a[1],
                jump: a[2] != 0,
                retry: false,
                reset: false,
            }],
        ));
        if driver.dead {
            break;
        }
    }
    (events, driver)
}

#[test]
fn gate_missed_scoring_detects_tamper() {
    let (events, driver) = run_golden(3600);
    assert!(
        driver.score >= 3,
        "golden scenario scores 3, got {}",
        driver.score
    );
    let pl: Vec<SquashEvent> = events.iter().map(|e| e.payload.clone()).collect();
    // positive: honest driver passes.
    assert!(gates::verify_score_matches_events(&pl, driver.score).is_ok());
    // negative: a buggy build that missed one increment is caught.
    assert!(gates::verify_score_matches_events(&pl, driver.score - 1).is_err());
    // phantom scoring is also caught.
    assert!(gates::verify_score_matches_events(&pl, driver.score + 1).is_err());
}

#[test]
fn gate_wrong_squash_semantics_detects_tamper() {
    // Frozen classifier on a standing player: death, no squash.
    let (events, driver) = run_quiet(3600, None);
    let pl: Vec<SquashEvent> = events.iter().map(|e| e.payload.clone()).collect();
    assert!(driver.dead, "standing player must die");
    assert!(gates::verify_side_contact_is_fatal(&pl, 0.0).is_ok());

    // Tampered classifier (everything is a squash) driven on the golden
    // stream: body top contacts are mis-counted as squashes.
    let wrong: fn(Vec3) -> PlayerMobContact = |_| PlayerMobContact::TopLandingSquash;
    let (events_w, driver_w) = run_golden_with(Some(wrong));
    let pl_w: Vec<SquashEvent> = events_w.iter().map(|e| e.payload.clone()).collect();
    let squashes = pl_w
        .iter()
        .filter(|e| matches!(e, SquashEvent::Squashed { .. }))
        .count();
    assert!(squashes > 0, "tampered build squashes on contact");
    let _ = driver_w.dead; // the geometric detector channel may still kill;
                           // the gate fires because squashes exist on a side-contact scenario.
    assert!(gates::verify_side_contact_is_fatal(&pl_w, 0.0).is_err());
}

#[test]
fn gate_blank_frames_detects_undriven() {
    // honest progression check: hashes that should differ must differ.
    assert!(gates::verify_frame_progression(true, "aaa", "bbb").is_ok());
    assert!(gates::verify_frame_progression(true, "same", "same").is_err());
    // positive control of the control: identical initial frames are fine.
    assert!(gates::verify_frame_progression(false, "same", "same").is_ok());
    assert!(gates::verify_frame_progression(false, "a", "b").is_err());
}

#[test]
fn gate_second_device_detects_foreign_token() {
    assert!(
        gates::verify_same_device("dev0:x", &["dev0:x".to_string(), "dev0:x".to_string()]).is_ok()
    );
    assert!(
        gates::verify_same_device("dev0:x", &["dev0:x".to_string(), "dev1:y".to_string()]).is_err()
    );
    assert!(gates::verify_same_device("dev0:x", &[]).is_ok());
}

#[test]
fn gate_late_events_detects_prior_session() {
    let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
    let mut driver = SquashDriver::new(SQUASH_SEED);
    let mut step = Vec::new();
    for _ in 0..61 {
        step.extend(rt.step(&mut driver, vec![SquashAction::default()]));
    }
    let session_a = step[0].session;
    rt.reset(SQUASH_SEED);
    driver.reset(); // frozen retry pairs runtime reset with the driver rebuild
    let session_b = rt.session();
    assert_ne!(session_a, session_b);
    let step_b = rt.step(&mut driver, vec![SquashAction::default()]);
    assert!(gates::verify_session_isolation(session_b, &step_b).is_ok());
    // poisoned stream containing a stale event: caught.
    let stale = step[0].clone();
    assert!(
        rt.accept_external_event(&stale).is_err(),
        "runtime rejects stale delivery"
    );
    let mut poisoned = step_b.clone();
    poisoned.push(stale);
    assert!(gates::verify_session_isolation(session_b, &poisoned).is_err());
}

#[test]
fn lifecycle_reset_baseline_twenty_rounds_pure() {
    // 20 restart rounds: scene returns to the same object count each round,
    // zero session violations (pure half of V-05).
    let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
    let mut driver = SquashDriver::new(SQUASH_SEED);
    let mut baseline: Option<usize> = None;
    for round in 0..20 {
        if round > 0 {
            rt.reset(SQUASH_SEED);
            driver.reset();
        }
        let session = rt.session();
        for _ in 0..300 {
            let events = rt.step(&mut driver, vec![SquashAction::default()]);
            assert!(events.iter().all(|e| e.session == session));
        }
        let objects = rt.scene().live_count();
        match baseline {
            None => baseline = Some(objects),
            Some(b) => assert_eq!(objects, b, "round {round} baseline drift"),
        }
    }
}

#[test]
fn lifecycle_gate_matrix_never_trusts_self_report() {
    // The verifier must derive truth independently: a driver that LIES about
    // its score (self-report) is caught by the event-stream recount.
    let (events, driver) = run_golden(3600);
    let pl: Vec<SquashEvent> = events.iter().map(|e| e.payload.clone()).collect();
    let honest = driver.score;
    assert!(honest >= 3);
    // A "self-reporting" buggy build claims honest-1 while events say honest.
    let claimed = honest - 1;
    let verdict = gates::verify_score_matches_events(&pl, claimed);
    assert!(
        verdict.is_err(),
        "verifier must not trust the claimed score"
    );
}
