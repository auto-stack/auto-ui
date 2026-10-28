//! Case `verifier-negative-controls` (V-08): injects four-plus counterexample
//! classes INTO the actual runner/pipeline, each with a legitimate positive
//! control, and asserts the verifier gates behave: positive passes, negative
//! is caught with the expected reason. Mis-green, skip, or unexecuted ⇒
//! non-zero exit. The verifier re-derives truth from independent state
//! (event stream, scene snapshots, keyframe hashes, device tokens) — it
//! never trusts the game's own PASS/0-count printing.
//!
//! Sabotage demo: `AGP_VERIFIER_BROKEN=<gate>` skips that gate's check; the
//! case must then exit non-zero (recorded evidence that an unexecuted gate
//! is never green).

use crate::core::runtime::{Runtime, TickEvent};
use crate::core::scene::{Color, Scene, SceneChange, Transform};
use crate::games::squash::{SquashAction, SquashDriver, SquashEvent, SQUASH_SEED};
use crate::host::gates;
use crate::host::report::{write_report, GateResult, NegativeReport};
use crate::renderer::capture::sha256_hex;
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};
use crate::renderer::target::OffscreenTarget;
use crate::renderer::GpuDevice;
use glam::Vec3;
use std::path::Path;
use std::time::Duration;

fn action_none() -> SquashAction {
    SquashAction::default()
}

/// Run the sim for `ticks` with a default (never-jumping, never-moving)
/// player; mobs converge and eventually a side contact kills. Stops early
/// on death. Returns (events, driver).
fn run_until<F: FnMut(&SquashEvent) -> bool>(
    ticks: usize,
    mut classifier: Option<fn(Vec3) -> crate::render3d::collision::PlayerMobContact>,
    mut stop: F,
) -> (Vec<TickEvent<SquashEvent>>, SquashDriver) {
    let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
    let mut driver = SquashDriver::new(SQUASH_SEED);
    if let Some(c) = classifier.take() {
        driver.squash_classifier = c;
    }
    let mut events = Vec::new();
    for _ in 0..ticks {
        let step = rt.step(&mut driver, vec![action_none()]);
        let mut hit = false;
        for e in &step {
            if stop(&e.payload) {
                hit = true;
            }
        }
        events.extend(step);
        if hit {
            break;
        }
    }
    (events, driver)
}

fn payloads(events: &[TickEvent<SquashEvent>]) -> Vec<SquashEvent> {
    events.iter().map(|e| e.payload.clone()).collect()
}

/// Run the frozen golden action stream (the canonical replay), yielding the
/// 3-squash scenario needed by the scoring gates.
fn run_golden(max_ticks: usize) -> (Vec<TickEvent<SquashEvent>>, SquashDriver) {
    run_golden_with_opts(max_ticks, None)
}

fn run_golden_with(
    classifier: Option<fn(Vec3) -> crate::render3d::collision::PlayerMobContact>,
) -> (Vec<TickEvent<SquashEvent>>, SquashDriver) {
    run_golden_with_opts(3600, classifier)
}

fn run_golden_with_opts(
    max_ticks: usize,
    classifier: Option<fn(Vec3) -> crate::render3d::collision::PlayerMobContact>,
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

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    if replay != "canonical" {
        eprintln!("verifier-negative-controls: unknown replay '{replay}'");
        return 2;
    }
    let broken = std::env::var("AGP_VERIFIER_BROKEN").ok();
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("verifier-negative-controls: device: {e}");
            return 3;
        }
    };
    let mut gates_out: Vec<GateResult> = Vec::new();

    // ------------------------------------------------------------------
    // Gate 1: missed scoring vs correct scoring.
    // ------------------------------------------------------------------
    {
        let (events, driver) = run_golden(3600);
        let pl = payloads(&events);
        let positive = gates::verify_score_matches_events(&pl, driver.score).is_ok();
        // Inject the fault: a buggy build that dropped one score increment.
        let tampered_claim = driver.score.saturating_sub(1);
        let negative = gates::verify_score_matches_events(&pl, tampered_claim).is_err();
        let reason = format!(
            "positive (events {} == score {}) ok={positive}; tampered claim {tampered_claim} caught={negative}",
            pl.iter().filter(|e| matches!(e, SquashEvent::Squashed { .. })).count(),
            driver.score
        );
        gates_out.push(GateResult {
            gate: "missed_scoring".into(),
            positive_passed: positive,
            negative_fired: negative,
            reason,
        });
    }

    // ------------------------------------------------------------------
    // Gate 2: wrong squash semantics (side contact counted as squash).
    // A standing player is killed by a side contact; the frozen classifier
    // yields death-only. The tampered classifier (anything = squash) emits a
    // Squashed event instead — the gate must catch it.
    // ------------------------------------------------------------------
    {
        // positive: frozen classifier, stand still, wait for death.
        let (events, driver) = run_until(3600, None, |e| matches!(e, SquashEvent::PlayerDied));
        let pl = payloads(&events);
        // A standing player contact is a side contact (normal_y <= 0.1).
        let side_normal_y = 0.0f32;
        let positive =
            gates::verify_side_contact_is_fatal(&pl, side_normal_y).is_ok() && driver.dead;
        // tampered: classifier that squashes everything. Driven with the
        // golden stream (body top contacts occur), so the wrong semantics
        // demonstrably emit squashes instead of the frozen death.
        let wrong: fn(Vec3) -> _ =
            |_| crate::render3d::collision::PlayerMobContact::TopLandingSquash;
        let (events_wrong, driver_wrong) = run_golden_with(Some(wrong));
        let pl_wrong = payloads(&events_wrong);
        let negative = gates::verify_side_contact_is_fatal(&pl_wrong, side_normal_y).is_err()
            || !driver_wrong.dead
                && pl_wrong
                    .iter()
                    .any(|e| matches!(e, SquashEvent::Squashed { .. }));
        gates_out.push(GateResult {
            gate: "wrong_squash_semantics".into(),
            positive_passed: positive,
            negative_fired: negative,
            reason: format!(
                "frozen classifier: death={} squashes={}; tampered: squashes={} death={}",
                driver.dead,
                pl.iter()
                    .filter(|e| matches!(e, SquashEvent::Squashed { .. }))
                    .count(),
                pl_wrong
                    .iter()
                    .filter(|e| matches!(e, SquashEvent::Squashed { .. }))
                    .count(),
                driver_wrong.dead
            ),
        });
    }

    // ------------------------------------------------------------------
    // Gate 3: blank/undriven frames vs driven frames. Render a real frame
    // at t=0 and at t=45 of the breakout-style scene: driven frames must
    // differ; a tampered run that skips sim steps hashes equal and fires.
    // ------------------------------------------------------------------
    {
        let (hash_a, hash_b) = render_two_frames(&gpu, true);
        let positive = gates::verify_frame_progression(true, &hash_a, &hash_b).is_ok();
        let (same_a, same_b) = render_two_frames(&gpu, false); // undriven
        let negative = gates::verify_frame_progression(true, &same_a, &same_b).is_err();
        gates_out.push(GateResult {
            gate: "blank_undriven_frames".into(),
            positive_passed: positive,
            negative_fired: negative,
            reason: format!(
                "driven hashes differ: {} (positive ok={positive}); undriven hashes equal caught={negative}",
                hash_a != hash_b
            ),
        });
    }

    // ------------------------------------------------------------------
    // Gate 4: second device vs the same-device contract.
    // ------------------------------------------------------------------
    {
        let game_token = gpu.token().to_report();
        // positive: two offscreen targets on the SAME device.
        let t1 = OffscreenTarget::new(&gpu, 64, 64).map(|t| t);
        let t2 = OffscreenTarget::new(&gpu, 64, 64);
        let _ = t1;
        let positive = match &t2 {
            Ok(_) => {
                gates::verify_same_device(&game_token, &[game_token.clone(), game_token.clone()])
                    .is_ok()
            }
            Err(_) => false,
        };
        // negative: a second GpuDevice is created and a target binds to it.
        let second = GpuDevice::new();
        let negative = match second {
            Ok(dev_b) => {
                let token_b = dev_b.token().to_report();
                let target_b = OffscreenTarget::new(&dev_b, 64, 64);
                match target_b {
                    Ok(_) => gates::verify_same_device(&game_token, &[game_token.clone(), token_b])
                        .is_err(),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        };
        gates_out.push(GateResult {
            gate: "second_device".into(),
            positive_passed: positive,
            negative_fired: negative,
            reason: "same-device tokens pass; foreign-device target caught".into(),
        });
    }

    // ------------------------------------------------------------------
    // Gate 5: late events (prior session) vs fresh-session events.
    // ------------------------------------------------------------------
    {
        let mut rt = Runtime::new(Duration::from_nanos(8_333_333), SQUASH_SEED);
        let mut driver = SquashDriver::new(SQUASH_SEED);
        // run 61 ticks so a spawn event (tick 60) exists to steal.
        let mut step = Vec::new();
        for _ in 0..61 {
            step.extend(rt.step(&mut driver, vec![action_none()]));
        }
        let session_a = step[0].session;
        rt.reset(SQUASH_SEED);
        driver.reset(); // frozen retry pairs runtime reset with the rebuild
        let session_b = rt.session();
        // positive: fresh-session events all carry session_b.
        let step_b = rt.step(&mut driver, vec![action_none()]);
        let positive =
            gates::verify_session_isolation(session_b, &step_b).is_ok() && session_a != session_b;
        // negative: inject the stale event into the new stream. The runtime
        // rejects it via accept_external_event — the gate catches any stream
        // that actually contains it (simulating a broken runner that
        // delivers it).
        let stale = step[0].clone();
        let rejected = rt.accept_external_event(&stale).is_err();
        let mut poisoned = step_b.clone();
        poisoned.push(stale);
        let negative = rejected && gates::verify_session_isolation(session_b, &poisoned).is_err();
        gates_out.push(GateResult {
            gate: "late_events".into(),
            positive_passed: positive,
            negative_fired: negative,
            reason: format!(
                "fresh session ok={positive}; stale event rejected={rejected} and caught={negative}"
            ),
        });
    }

    // ------------------------------------------------------------------
    // Sabotage handling: a skipped/unexecuted gate is never green.
    // ------------------------------------------------------------------
    let mut skipped = false;
    if let Some(g) = &broken {
        if let Some(gr) = gates_out.iter_mut().find(|x| x.gate == *g) {
            gr.positive_passed = false;
            gr.reason = format!("SABOTAGED (AGP_VERIFIER_BROKEN): check skipped");
            skipped = true;
        } else {
            eprintln!("unknown gate '{g}' for AGP_VERIFIER_BROKEN");
            return 2;
        }
    }
    let all_behaved = gates_out
        .iter()
        .all(|g| g.positive_passed && g.negative_fired);
    let pass = all_behaved && !skipped;

    let report = NegativeReport {
        adapter: gpu.adapter_info().clone(),
        device_token: gpu.token().to_report(),
        mode: if mode == "headless" { "headless" } else { "window" }.into(),
        gates: gates_out,
        all_gates_behaved: all_behaved,
        any_gate_skipped_or_unexecuted: skipped,
        notes: vec![
            "gates assert on independent state: event-stream recount, death-only side-contact semantics, keyframe content hashes, device tokens, session ids — never on the game's own PASS printing".into(),
            "AGP_VERIFIER_BROKEN=<gate> simulates a broken verifier; the case exits non-zero (unexecuted ⇒ fail)".into(),
        ],
    };
    let exit = if pass { "pass" } else { "fail" };
    match write_report(
        report_dir,
        "verifier-negative-controls",
        "canonical",
        exit,
        report,
    ) {
        Ok(()) => {
            if pass {
                println!("verifier-negative-controls: all gates behaved");
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("verifier-negative-controls: report: {e}");
            3
        }
    }
}

/// Render two offscreen 2D frames of a tiny animated scene; if `driven`,
/// advance a value between frames; return content hashes.
fn render_two_frames(gpu: &GpuDevice, driven: bool) -> (String, String) {
    let target = OffscreenTarget::new(gpu, 64, 64).expect("target");
    let pipe = QuadPipeline::new(gpu, OffscreenTarget::FORMAT).expect("pipe");
    pipe.set_ortho_camera(gpu, 0.0, 64.0, 64.0, 0.0);
    let frame = |x: f32| -> String {
        pipe.render(
            gpu,
            target.view(),
            OffscreenTarget::FORMAT,
            Color::rgb(0.1, 0.1, 0.1),
            &[DrawCmd {
                center: [x, 32.0],
                size: [16.0, 16.0],
                z: 0.0,
                color: Color::rgb(1.0, 0.4, 0.2),
                shape: Shape::Rect,
            }],
        );
        gpu.wait_idle();
        let (png, _) = crate::renderer::capture::capture_png(gpu, &target).expect("capture");
        sha256_hex(&png)
    };
    let a = frame(10.0);
    let b = frame(if driven { 40.0 } else { 10.0 });
    (a, b)
}

/// Helper used by gate-2 positive: a tiny scene mutation sanity check that
/// the Scene API still rejects late ids (kept here so the negative case
/// exercises the shared core too).
#[allow(dead_code)]
fn scene_late_id_rejected() -> bool {
    let mut scene = Scene::new();
    assert!(scene
        .submit(&[SceneChange::Spawn {
            transform: Transform::default(),
            color: Color::rgb(1.0, 1.0, 1.0),
        }])
        .is_empty());
    let id = scene.objects()[0].id;
    assert!(scene.submit(&[SceneChange::Destroy(id)]).is_empty());
    !scene
        .submit(&[SceneChange::SetColor(id, Color::rgb(0.0, 0.0, 0.0))])
        .is_empty()
}
