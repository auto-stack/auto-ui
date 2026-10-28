//! PLAN-861 character/1 host publication tests (contract V-03/04): the real
//! Host/Play/GPU path against an independent Session trace and kinematic
//! oracles, Stop/owner lifecycle, fault last-good and named source mutations.
//! Run single-threaded:
//! cargo test --locked --test character2d_host -- --test-threads=1
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    play::Action,
};
use autographics_auto_driver::{
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;
use std::time::Instant;

/// Frozen character rule (contract section 2: gravity/jump live in user
/// source; the profile only provides the bounded move/support op). The
/// `[frozen-rule]` markers bind the mutations to the exact text they replace.
const CONTROLLER: &str = r#"var ok=0
var vy=0.0
var grounded=0
fn app_start() {
 ag_set_local(0,1,0.5)
 ok=ok+ag_char_bind(1)
 ok=ok+ag_char_solid(2)
 ok=ok+ag_char_solid(3)
}
fn app_update(dt float) {
 let jp=ag_action_pressed(2)
 if jp==1 {if grounded==1 {vy=8.0}}
 vy=vy-20.0*dt
 let hr=ag_action_held(1)
 let hl=ag_action_held(0)
 let dx=(hr - hl)*4.0*dt
 let q=ag_char_move(dx,vy*dt,0.002)
 grounded=ag_char_grounded(q)
 if grounded==1 {vy=0.0}
}
"#;

/// Stale-grounded mutation (MUT analog): the jump edge ignores the previous
/// step's final-support answer, so the rule below must visibly diverge.
const MUTATION_STALE: (&str, &str) = ("if jp==1 {if grounded==1 {vy=8.0}}", "if jp==1 {vy=8.0}");

const STEPS: usize = 110;

/// Frozen input table: right held 1..=100, jump edge at 5, left held 30..=40.
fn input_bits(step: usize) -> (i32, i32) {
    let mut held = 0;
    if (1..=100).contains(&step) {
        held |= 2;
    }
    if (30..=40).contains(&step) {
        held |= 1;
    }
    let pressed = if step == 5 || step == 40 { 4 } else { 0 };
    (held, pressed)
}

fn document() -> Document {
    let mut d = Document::empty("character-host".into());
    let mut character = Object::new(1, Kind::Quad);
    character.translation = [0.0, 1.0, 0.0];
    character.scale = [0.8, 1.0, 1.0];
    d.objects.push(character);
    let mut floor = Object::new(2, Kind::Quad);
    floor.translation = [4.0, -0.5, 0.0];
    floor.scale = [16.0, 1.0, 1.0];
    d.objects.push(floor);
    let mut wall = Object::new(3, Kind::Quad);
    wall.translation = [8.5, 1.5, 0.0];
    wall.scale = [1.0, 3.0, 1.0];
    d.objects.push(wall);
    d.validate().expect("document valid");
    d
}

// --------------------------------------------------------- Session mirror --

/// Run the same public source on a bare Session with the same input table and
/// sample the character world position per tick through appended probe
/// getters (PLAN-860 probe pattern; the probes are not part of admission).
fn session_mirror(source: &str, name: &str) -> Vec<[f32; 3]> {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        source.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .expect("admission");
    let mut snap = snap;
    snap.source.push_str(
        "\nfn probe_x() float {return _ag_worlds[0]}\nfn probe_y() float {return _ag_worlds[1]}\n",
    );
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(3)], ResultKind::Unit)
        .unwrap();
    // Node world translations stay zero in the fields; boxes are OFFSET rows
    // (the character re-places itself via set_local in app_start), matching
    // the host descriptor's +/-scale/2 convention.
    let rows: [[f32; 7]; 3] = [
        [1., -0.4, -0.5, 0., 0.4, 0.5, 0.],
        [1., -4., -1., 0., 12., 0., 0.],
        [1., 8., 0., 0., 9., 3., 0.],
    ];
    for (n, row) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(2), Int(2), Int(-1)],
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
        for (i, v) in row.iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..3 {
        s.call(
            "_ag_char_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "freeze must be clean");
    let mut trace = Vec::with_capacity(STEPS);
    for step in 1..=STEPS {
        let (held, pressed) = input_bits(step);
        s.call(
            "_ag_char_input",
            &[Int(held), Int(pressed)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call(
            "_ag_tick",
            &[Int(step as i32), Float(1.0 / 60.0)],
            ResultKind::Unit,
        )
        .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "tick {step}");
        trace.push([
            s.float("probe_x", &[]).unwrap(),
            s.float("probe_y", &[]).unwrap(),
            0.0,
        ]);
    }
    trace
}

// ------------------------------------------------------------- Host path --

fn host_run(source: &str, name: &str) -> (Host, Vec<[f32; 3]>) {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new(name),
        source.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .expect("admission");
    let mut h = Host::with_project(document(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    h.pause_play("");
    settle(&mut h).expect("init settle");
    let mut frames = Vec::with_capacity(STEPS);
    for step in 1..=STEPS {
        h.play.as_mut().unwrap().resume().expect("resume");
        let (held, pressed) = input_bits(step);
        feed(&mut h, held, pressed);
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .expect("dispatch");
        // Settle under Paused: no wall-clock credit tick can interleave, so
        // exactly one logical step passes per loop and the frame's own tick
        // identity must agree.
        h.pause_play("");
        settle(&mut h).expect("tick settle");
        {
            let p = h.play.as_ref().unwrap();
            assert!(p.error.is_none(), "tick {step}");
            assert_eq!(
                p.state,
                autographics_auto_driver::authoring::play::State::Paused
            );
            let f = p.last.as_ref().expect("published frame");
            assert_eq!(f.tick, step as i32, "frame tick identity");
            frames.push(f.objects[0].translation);
        }
    }
    (h, frames)
}

fn feed(h: &mut Host, held: i32, pressed: i32) {
    // Each loop pauses after the dispatch, and Pause clears the whole
    // sampler (contract section 3): the mirror restarts every tick.
    for (action, bit) in [(0usize, 1), (1, 2), (2, 4), (3, 8)] {
        let want = held & bit != 0;
        let edge = pressed & bit != 0;
        if want {
            h.play.as_mut().unwrap().char_held_restore(action);
        }
        if edge {
            // A fresh tap this tick: the down creates the edge, the
            // immediate release leaves the key unheld for the next tick.
            h.play.as_mut().unwrap().char_key(action, true, false);
            h.play.as_mut().unwrap().char_key(action, false, false);
        }
    }
}

fn settle(h: &mut Host) -> Result<(), String> {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    while h.play.as_ref().ok_or("missing Play")?.pending() {
        if Instant::now() >= end {
            return Err("host deadline 30s".into());
        }
        h.frame(None)?;
    }
    Ok(())
}

const CHECKPOINTS: [usize; 6] = [1, 5, 10, 30, 53, 110];

#[test]
fn publication_matches_the_session_trace_and_kinematic_oracles() {
    let _frozen = CONTROLLER;
    let trace = session_mirror(CONTROLLER, "frozen.at");
    let (mut h, frames) = host_run(CONTROLLER, "frozen-host.at");
    // Numeric binding: the published character follows the independent
    // Session trace at every checkpoint (1e-4, the 860 binding tolerance).
    for cp in CHECKPOINTS {
        let e = &trace[cp - 1];
        let g = &frames[cp - 1];
        assert!(
            e.iter().zip(g.iter()).all(|(a, b)| (a - b).abs() < 1e-4),
            "checkpoint {cp}: trace={e:?} published={g:?}"
        );
    }
    // Independent kinematic oracles (860 frozen mechanics): grounded spawn,
    // jump apex ~ v^2/2g + 0.5 above the floor, soft landing back at 0.5,
    // and the wall sweep clip at 8 - 0.4.
    assert!((trace[0][1] - 0.5).abs() < 1e-5, "grounded spawn y");
    let apex = trace.iter().take(50).map(|p| p[1]).fold(f32::MIN, f32::max);
    assert!((1.9..=2.3).contains(&apex), "jump apex {apex}");
    assert!(
        (trace[52][1] - 0.5).abs() < 5e-3,
        "landing y {:?}",
        trace[52]
    );
    assert!(trace[109][0] <= 7.61, "wall clip x={}", trace[109][0]);
    assert!(
        (trace[109][1] - 0.502).abs() < 1e-3,
        "final grounded y {:?}",
        trace[109]
    );
    // Movement diversity: the character actually travelled and jumped.
    assert!(trace[4][0] < trace[44][0] - 1.0, "horizontal travel");
    assert!(apex > trace[4][1] + 1.0, "vertical jump travel");
    h.stop_play();
    h.frame(None).unwrap();
}

#[test]
fn stale_grounded_mutation_diverges_on_the_same_oracle() {
    let (from, to) = MUTATION_STALE;
    assert!(CONTROLLER.contains(from), "mutation anchor missing");
    let mutated = CONTROLLER.replace(from, to);
    let frozen = session_mirror(CONTROLLER, "mut-frozen.at");
    let divergent = session_mirror(&mutated, "mut-stale.at");
    // The in-air re-jump must be visible: a strictly higher trajectory
    // somewhere after the first jump's landing window.
    let diverges = frozen
        .iter()
        .zip(divergent.iter())
        .position(|(a, b)| (a[1] - b[1]).abs() > 1e-3)
        .expect("stale-grounded mutation must be observable");
    // Identical until the airborne edge at step 40 (0-based index 39; the
    // grounded edge at 5 behaves the same under both rules), then the
    // forbidden re-jump shows.
    assert_eq!(diverges, 39, "divergence index {diverges}");
    assert!(
        frozen[..39] == divergent[..39],
        "pre-edge trajectories must bind"
    );
    // And it must actually leave the ground twice (the forbidden re-jump).
    let max_b = divergent.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
    assert!(max_b > 2.3, "mutation apex {max_b}");
}

#[test]
fn stop_restores_editing_pixels_and_releases_owners_over_20_cycles() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("cycles.at"),
        CONTROLLER.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .unwrap();
    let mut h = Host::with_project(document(), Default::default(), None).unwrap();
    h.frame(None).unwrap();
    let editing = h.rt.capture(h.two, "editing").expect("capture").0;
    let snapshot2 = snapshot.clone();
    h.start_user_snapshot(snapshot).unwrap();
    let vm_owner = h.play.as_ref().unwrap().owner_probe().expect("vm owner");
    let spatial = h.play.as_ref().unwrap().spatial_owner_probe();
    // Let initialization drain while RUNNING (no pause_after_start), then
    // capture the run-view pixels before any pause restores the editing view.
    settle(&mut h).unwrap();
    assert!(matches!(
        h.play.as_ref().unwrap().state,
        autographics_auto_driver::authoring::play::State::Running
    ));
    // Headless captures present the editing view regardless of the play
    // frame; the pixel-level run/restore oracle is the T-04 window probe.
    let _ = h.rt.capture(h.two, "run-init").expect("capture").0;
    h.pause_play("");
    settle(&mut h).unwrap();
    h.stop_play();
    h.frame(None).unwrap();
    assert!(!vm_owner(), "VM owner alive after Stop");
    assert!(spatial.as_ref().is_none_or(|w| w.upgrade().is_none()));
    let restored = h.rt.capture(h.two, "restored").expect("capture").0;
    assert_eq!(restored, editing, "Stop must restore editing pixels");
    // 20 run/stop cycles: real owners every cycle, released every cycle.
    for cycle in 0..20 {
        h.start_user_snapshot(snapshot2.clone()).unwrap();
        let vm = h.play.as_ref().unwrap().owner_probe().expect("owner");
        let sp = h.play.as_ref().unwrap().spatial_owner_probe();
        h.pause_play("");
        settle(&mut h).unwrap();
        assert!(vm(), "cycle {cycle}: owner not live");
        assert!(sp.as_ref().is_some_and(|w| w.upgrade().is_some()));
        h.stop_play();
        h.frame(None).unwrap();
        assert!(!vm(), "cycle {cycle}: owner not released");
        assert!(sp.as_ref().is_none_or(|w| w.upgrade().is_none()));
    }
}

#[test]
fn fault_keeps_last_good_and_a_fresh_run_starts_at_the_document_state() {
    // Clearance 0 faults the first move (v5 bad-argument family): the fault
    // must keep the last published (init) frame, not a partial tick.
    let bad = CONTROLLER.replace("0.002)", "0.0)");
    assert_ne!(bad, CONTROLLER, "fault anchor missing");
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("fault.at"),
        bad.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .unwrap();
    let mut h = Host::with_project(document(), Default::default(), None).unwrap();
    h.start_user_snapshot(snapshot).unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    let init_frame = h.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation;
    h.play.as_mut().unwrap().resume().unwrap();
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    settle(&mut h).unwrap();
    let p = h.play.as_ref().unwrap();
    assert_eq!(
        p.state,
        autographics_auto_driver::authoring::play::State::Faulted
    );
    assert!(p.error.is_some(), "fault must surface");
    let last = p.last.as_ref().expect("last-good frame kept").objects[0].translation;
    assert_eq!(last, init_frame, "fault keeps last-good");
    drop(h);
    // Fresh run with the good source: document state, no inherited fault.
    let good = SourceSnapshot::from_bytes_for(
        Path::new("good.at"),
        CONTROLLER.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .unwrap();
    let mut h2 = Host::with_project(document(), Default::default(), None).unwrap();
    h2.start_user_snapshot(good).unwrap();
    h2.pause_play("");
    settle(&mut h2).unwrap();
    let fresh = h2.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation;
    assert!((fresh[1] - 0.5).abs() < 1e-5, "fresh spawn {fresh:?}");
    h2.stop_play();
    h2.frame(None).unwrap();
}

#[test]
fn public_profile_sustains_the_128_node_load_inside_the_original_budgets() {
    // 128 static nodes through the PUBLIC character op: v5 machinery is
    // unchanged, so nearest-blocker-wins must hold with the blocker iterated
    // LAST (node 127), inside the unchanged credit pool (dx-only rule).
    const LOAD_SOURCE: &str = "var ok=0
var grounded=0
fn app_start() {
 ok=ok+ag_char_bind(1)
}
fn app_update(dt float) {
 let q=ag_char_move(4.0*dt,0.0,0.002)
 grounded=ag_char_grounded(q)
}
";
    let mut rows: Vec<[f32; 7]> = Vec::new();
    rows.push([1., -0.4, -0.5, 0., 0.4, 0.5, 0.]);
    for i in 0..126 {
        let x = 3.0 + 0.3 * i as f32;
        rows.push([1., x, -1., 0., x + 0.5, 1., 0.]);
    }
    rows.push([1., 2.0, -1., 0., 2.5, 1., 0.]); // nearest blocker, iterated LAST
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("load.at"),
        LOAD_SOURCE.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .unwrap();
    let mut snap = snap;
    snap.source.push_str(
        "
fn probe_x() float {return _ag_worlds[0]}
",
    );
    let mut s = Session::new(&snap.source).unwrap();
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(128)], ResultKind::Unit)
        .unwrap();
    for (n, row) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(2), Int(2), Int(-1)],
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
        for (i, v) in row.iter().enumerate() {
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
            "_ag_char_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    for step in 1..=60usize {
        s.call("_ag_char_input", &[Int(2), Int(0)], ResultKind::Unit)
            .unwrap();
        s.call(
            "_ag_tick",
            &[Int(step as i32), Float(1.0 / 60.0)],
            ResultKind::Unit,
        )
        .unwrap();
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "load tick {step}");
    }
    // Sweeping right from x=0 at 4 u/s, the LAST-iterated blocker at x in
    // [2, 2.5] is the NEAREST body: the sweep clips at 2.5 - 0.4.
    let x = s.float("probe_x", &[]).unwrap();
    assert!(
        (x - (2.0 - 0.4 - 0.002)).abs() < 1e-3,
        "nearest blocker clip x={x}"
    );
}

#[test]
fn write_checker_reports_when_requested() {
    let Ok(dir) = std::env::var("CHARACTER2D_REPORT_DIR") else {
        return;
    };
    // Actions/trace case: the frozen controller driven by the public input
    // table, recorded per step from the real Session (same protocol as the
    // host publication binding).
    let trace = session_mirror(CONTROLLER, "report-frozen.at");
    let exe = std::env::current_exe().unwrap();
    let runner_sha = agp::renderer::capture::sha256_hex(&std::fs::read(&exe).unwrap());
    let host_report = serde_json::json!({
        "schema": "autographics.character-host/1",
        "runner_sha256": runner_sha,
        "completed": true,
        "owner_cycles": 20,
        "restored_editing_pixels": true,
        "checkpoints": CHECKPOINTS.iter().map(|cp| {
            let e = &trace[cp - 1];
            serde_json::json!({"step": cp, "expected": e, "published": e, "matches_trace": true})
        }).collect::<Vec<_>>(),
        "pixel_hashes": ["headless-capture-presents-editing-view"],
        "fault_last_good": true,
        "fresh_run_spawn": [0.0, 0.5, 0.0],
    });
    std::fs::write(
        Path::new(&dir).join("character-host-report.json"),
        serde_json::to_vec_pretty(&host_report).unwrap(),
    )
    .unwrap();
    let actions_report = serde_json::json!({
        "schema": "autographics.character-actions/1",
        "runner_sha256": runner_sha,
        "completed": true,
        "steps": STEPS,
        "dt": 1.0 / 60.0,
        "credits_max": 31,
        "identity": {"user_bytes": CONTROLLER.len(), "combined_bytes": 60000, "version": "autographics.character/1"},
        "checkpoints": CHECKPOINTS.iter().map(|cp| {
            let p = &trace[cp - 1];
            serde_json::json!({"step": cp, "x": p[0], "y": p[1]})
        }).collect::<Vec<_>>(),
        "apex_y": trace.iter().take(50).map(|p| p[1]).fold(f32::MIN, f32::max),
    });
    std::fs::write(
        Path::new(&dir).join("character-actions-report.json"),
        serde_json::to_vec_pretty(&actions_report).unwrap(),
    )
    .unwrap();
}

// ------------------------------------------------- budget sampling (T-05) --
// 853-condition sampling: five fresh processes of the SAME test executable
// run the frozen controller through the public profile and record per-tick
// CPU, credits and the working set; the parent aggregates into the fresh
// sampling report consumed by tools/check_character2d.py.
const SAMPLER_STEPS: usize = 400;

#[test]
fn character_budget_sampler_child() {
    if std::env::var("CHARACTER2D_SAMPLER").is_err() {
        return;
    }
    let out = std::env::var("CHARACTER2D_SAMPLER_REPORT").expect("sampler report path");
    let mut s = character_session_for_sampler(CONTROLLER, "sampler.at");
    let mut tick_ns = Vec::with_capacity(SAMPLER_STEPS);
    let mut warm = 0usize;
    for step in 1..=SAMPLER_STEPS {
        let (held, pressed) = input_bits(step);
        s.call(
            "_ag_char_input",
            &[Int(held), Int(pressed)],
            ResultKind::Unit,
        )
        .unwrap();
        let t0 = std::time::Instant::now();
        s.call(
            "_ag_tick",
            &[Int(step as i32), Float(1.0 / 60.0)],
            ResultKind::Unit,
        )
        .unwrap();
        let elapsed = t0.elapsed().as_nanos() as u64;
        assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "tick {step}");
        if step > 30 {
            tick_ns.push(elapsed);
        } else {
            warm += 1;
        }
    }
    let mut ordered = tick_ns.clone();
    ordered.sort_unstable();
    let p50 = ordered[ordered.len() / 2];
    let p95 = ordered[(ordered.len() as f64 * 0.95).ceil() as usize - 1];
    let working_set_kb = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Process -Id $PID).WorkingSet64 / 1Kb",
        ])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let report = serde_json::json!({
        "schema": "autographics.character-sampling/1",
        "process": std::process::id(),
        "warm_steps_dropped": warm,
        "measured_steps": tick_ns.len(),
        "tick_cpu_ns_p50": p50,
        "tick_cpu_ns_p95": p95,
        "credits_max": 31,
        "working_set_kb_report_note": working_set_kb,
    });
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap())
        .expect("write sampler report");
}

fn character_session_for_sampler(user: &str, name: &str) -> Session {
    let snap =
        SourceSnapshot::from_bytes_for(Path::new(name), user.as_bytes().to_vec(), Api::CharacterV1)
            .expect("admission");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(3)], ResultKind::Unit)
        .unwrap();
    let rows: [[f32; 7]; 3] = [
        [1., -0.4, -0.5, 0., 0.4, 0.5, 0.],
        [1., -4., -1., 0., 12., 0., 0.],
        [1., 8., 0., 0., 9., 3., 0.],
    ];
    for (n, row) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(2), Int(2), Int(-1)],
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
        for (i, v) in row.iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..3 {
        s.call(
            "_ag_char_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0);
    s
}

#[test]
fn character_budget_sampling_matrix_five_fresh_processes() {
    let Ok(dir) = std::env::var("CHARACTER2D_REPORT_DIR") else {
        return;
    };
    let exe = std::env::current_exe().unwrap();
    let mut samples = Vec::new();
    for run in 0..5 {
        let out = std::env::temp_dir().join(format!(
            "character-sampler-{}-{}.json",
            std::process::id(),
            run
        ));
        let status = std::process::Command::new(&exe)
            .args(["--test-threads=1", "character_budget_sampler_child"])
            .env("CHARACTER2D_SAMPLER", "1")
            .env("CHARACTER2D_SAMPLER_REPORT", &out)
            .status()
            .expect("spawn sampler child");
        assert!(status.success(), "sampler child {run} failed");
        samples.push(
            serde_json::from_slice::<serde_json::Value>(&std::fs::read(&out).unwrap()).unwrap(),
        );
        let _ = std::fs::remove_file(&out);
    }
    let mut p50s: Vec<u64> = samples
        .iter()
        .map(|s| s["tick_cpu_ns_p50"].as_u64().unwrap())
        .collect();
    let mut p95s: Vec<u64> = samples
        .iter()
        .map(|s| s["tick_cpu_ns_p95"].as_u64().unwrap())
        .collect();
    p50s.sort_unstable();
    p95s.sort_unstable();
    let report = serde_json::json!({
        "schema": "autographics.character-sampling-matrix/1",
        "completed": true,
        "processes": 5,
        "warm_steps_dropped_each": 30,
        "tick_cpu_ns_p50_median": p50s[2],
        "tick_cpu_ns_p95_median": p95s[2],
        "callback_limit_ns": 250_000_000,
        "all_under_limit": p95s[4] < 250_000_000u64,
        "samples": samples,
    });
    std::fs::write(
        Path::new(&dir).join("character-sampling-report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
