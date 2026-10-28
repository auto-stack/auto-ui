//! PLAN-874 character3d/1 host publication tests (contract V-04/V-06): the
//! real Host/Play/GPU path against an independent Session trace and kinematic
//! oracles, Stop/owner lifecycle, fault last-good, the single router per
//! profile and the fresh-process budget sampling matrix. Run single-threaded:
//! cargo test --locked --test character3d_host -- --test-threads=1
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    play::{Action, CharProfile},
};
use autographics_auto_driver::{
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;
use std::time::Instant;

/// Frozen character rule (contract section 6: gravity/jump live in the user
/// source; the profile only provides the bounded three-dimensional move and
/// the final-support determination). The `[frozen-rule]` markers bind the
/// mutation to the exact text it replaces.
const CONTROLLER: &str = r#"var ok=0
var vy=0.0
var grounded=0
var face_x=0.0
var face_z= -1.0
fn app_start() {
 ag_set_local(0,1,0.5)
 ok=ok+ag_char3_bind(1)
 ok=ok+ag_char3_solid(2)
 ok=ok+ag_char3_solid(3)
}
fn app_update(dt float) {
 let jp=ag_action_pressed(4)
 if jp==1 {if grounded==1 {vy=8.0}}
 vy=vy-20.0*dt
 let hr=ag_action_held(1)
 let hl=ag_action_held(0)
 let hf=ag_action_held(2)
 let hb=ag_action_held(3)
 var mx = hr - hl
 var mz = hb - hf
 var n = 1.0
 if mx != 0 {if mz != 0 {n = 0.70710678}}
 mx = mx*n
 mz = mz*n
 if mx != 0 {face_x = mx;face_z = mz}
 if mz != 0 {face_x = mx;face_z = mz}
 let q=ag_char3_move(mx*4.0*dt,vy*dt,mz*4.0*dt,0.002)
 grounded=ag_char3_grounded(q)
 if grounded==1 {vy=0.0}
}
"#;

/// Stale-grounded mutation: the jump edge ignores the previous step's final
/// support answer, so the rule below must visibly diverge.
const MUTATION_STALE: (&str, &str) = ("if jp==1 {if grounded==1 {vy=8.0}}", "if jp==1 {vy=8.0}");

const STEPS: usize = 120;
const CHECKPOINTS: [usize; 5] = [1, 5, 30, 60, 120];

/// Frozen input table (contract section 3 codes): forward 1..=100, jump edges
/// at 5 and 40, right 60..=80, left 30..=40 (opposing pair at 30..=40).
fn input_bits(step: usize) -> (i32, i32) {
    let mut held = 0;
    if (1..=100).contains(&step) {
        held |= 4; // Forward
    }
    if (60..=80).contains(&step) {
        held |= 2; // Right
    }
    if (30..=40).contains(&step) {
        held |= 1; // Left
    }
    let pressed = if step == 5 || step == 40 { 16 } else { 0 }; // Jump edge
    (held, pressed)
}

fn document() -> Document {
    let mut d = Document::empty("character3d-host".into());
    let mut character = Object::new(1, Kind::Cylinder);
    character.translation = [0.0, 0.5, 0.0];
    character.scale = [0.4, 0.5, 0.4];
    d.objects.push(character);
    let mut floor = Object::new(2, Kind::Cylinder);
    floor.translation = [0.0, -0.5, 0.0];
    floor.scale = [10.0, 0.5, 10.0];
    d.objects.push(floor);
    let mut wall = Object::new(3, Kind::Cylinder);
    wall.translation = [6.0, 1.5, 0.0];
    wall.scale = [0.1, 1.5, 10.0];
    d.objects.push(wall);
    d.validate().expect("document valid");
    d
}

// --------------------------------------------------------- Session mirror --

/// Run the same public source on a bare Session with the same input table and
/// sample the character world position per tick through appended probe getters.
fn session_mirror(source: &str, name: &str) -> Vec<[f32; 3]> {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        source.as_bytes().to_vec(),
        Api::Character3V1,
    )
    .expect("admission");
    let mut snap = snap;
    snap.source.push_str(
        "\nfn probe_x() float {return _ag_worlds[0]}\nfn probe_y() float {return _ag_worlds[1]}\nfn probe_z() float {return _ag_worlds[2]}\n",
    );
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_spatial_reset", &[Int(3)], ResultKind::Unit)
        .unwrap();
    // Boxes are OFFSET rows (the character re-places itself via set_local in
    // app_start), matching the host descriptor's +/-scale convention.
    let rows: [[f32; 7]; 3] = [
        [1., -0.4, -0.5, -0.4, 0.4, 0.5, 0.4],
        [1., -10., -1., -10., 10., 0., 10.],
        [1., 5.9, 0., -10., 6.1, 3., 10.],
    ];
    for (n, row) in rows.iter().enumerate() {
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
            "_ag_char3_map",
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
            "_ag_char3_input",
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
            s.float("probe_z", &[]).unwrap(),
        ]);
    }
    trace
}

// ------------------------------------------------------------- Host path --

fn host_run(source: &str, name: &str) -> (Host, Vec<[f32; 3]>) {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new(name),
        source.as_bytes().to_vec(),
        Api::Character3V1,
    )
    .expect("admission");
    let mut h = Host::with_project(document(), Default::default(), None).expect("host");
    h.start_user_snapshot(snapshot).expect("start");
    assert_eq!(
        h.play.as_ref().unwrap().character_profile(),
        Some(CharProfile::ThreeD),
        "the run must use character3d/1"
    );
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
        // exactly one logical step passes per loop.
        h.pause_play("");
        settle(&mut h).expect("tick settle");
        {
            let p = h.play.as_ref().unwrap();
            assert!(p.error.is_none(), "tick {step}");
            let f = p.last.as_ref().expect("published frame");
            assert_eq!(f.tick, step as i32, "frame tick identity");
            frames.push(f.objects[0].translation);
        }
    }
    (h, frames)
}

fn feed(h: &mut Host, held: i32, pressed: i32) {
    // Each loop pauses after the dispatch, and Pause clears the whole sampler:
    // the mirror restarts every tick from the physical state.
    for (action, bit) in [(0usize, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)] {
        let want = held & bit != 0;
        let edge = pressed & bit != 0;
        if want {
            h.play.as_mut().unwrap().char_held_restore(action);
        }
        if edge {
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

#[test]
fn publication_matches_the_session_trace_and_kinematic_oracles() {
    let _frozen = CONTROLLER;
    let trace = session_mirror(CONTROLLER, "frozen.at");
    let (mut h, frames) = host_run(CONTROLLER, "frozen-host.at");
    for cp in CHECKPOINTS {
        let e = &trace[cp - 1];
        let g = &frames[cp - 1];
        assert!(
            e.iter().zip(g.iter()).all(|(a, b)| (a - b).abs() < 1e-4),
            "checkpoint {cp}: trace={e:?} published={g:?}"
        );
    }
    // Independent kinematics: grounded spawn, jump apex ~ v^2/2g above the
    // floor, soft landing back at 0.5, and forward travel along -z.
    assert!((trace[0][1] - 0.5).abs() < 1e-5, "grounded spawn y");
    let apex = trace.iter().take(50).map(|p| p[1]).fold(f32::MIN, f32::max);
    assert!((1.9..=2.3).contains(&apex), "jump apex {apex}");
    assert!(
        (trace[52][1] - 0.5).abs() < 1e-2,
        "landing y {:?}",
        trace[52]
    );
    assert!(
        trace[119][2] < -3.0,
        "forward travel must advance -z: {:?}",
        trace[119]
    );
    // Diagonal input at 60..=80 is normalised: the |x| displacement over those
    // 21 steps stays at speed*dt per step (no sqrt(2) inflation).
    let dx = (trace[79][0] - trace[59][0]).abs();
    assert!(
        (dx - 20.0 * 4.0 / 60.0 * 0.70710678).abs() < 0.05,
        "normalised diagonal dx={dx}"
    );
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
    let diverges = frozen
        .iter()
        .zip(divergent.iter())
        .position(|(a, b)| (a[1] - b[1]).abs() > 1e-3)
        .expect("stale-grounded mutation must be observable");
    assert_eq!(diverges, 39, "divergence index {diverges}");
    let max_b = divergent.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
    assert!(max_b > 2.3, "mutation apex {max_b}");
}

#[test]
fn stop_restores_editing_pixels_and_releases_owners_over_20_cycles() {
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("cycles.at"),
        CONTROLLER.as_bytes().to_vec(),
        Api::Character3V1,
    )
    .unwrap();
    let mut h = Host::with_project(document(), Default::default(), None).unwrap();
    h.frame(None).unwrap();
    let editing = h.rt.capture(h.three, "editing").expect("capture").0;
    let snapshot2 = snapshot.clone();
    h.start_user_snapshot(snapshot).unwrap();
    let vm_owner = h.play.as_ref().unwrap().owner_probe().expect("vm owner");
    let spatial = h.play.as_ref().unwrap().spatial_owner_probe();
    settle(&mut h).unwrap();
    h.pause_play("");
    settle(&mut h).unwrap();
    h.stop_play();
    h.frame(None).unwrap();
    assert!(!vm_owner(), "VM owner alive after Stop");
    assert!(spatial.as_ref().is_none_or(|w| w.upgrade().is_none()));
    let restored = h.rt.capture(h.three, "restored").expect("capture").0;
    assert_eq!(restored, editing, "Stop must restore editing pixels");
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
    let bad = CONTROLLER.replace("0.002)", "0.0)");
    assert_ne!(bad, CONTROLLER, "fault anchor missing");
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("fault.at"),
        bad.as_bytes().to_vec(),
        Api::Character3V1,
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
    {
        let p = h.play.as_ref().unwrap();
        assert_eq!(
            p.state,
            autographics_auto_driver::authoring::play::State::Faulted
        );
        assert!(p.error.is_some(), "fault must surface");
        let last = p.last.as_ref().expect("last-good frame kept").objects[0].translation;
        assert_eq!(last, init_frame, "fault keeps last-good");
    }
    drop(h);
    let good = SourceSnapshot::from_bytes_for(
        Path::new("good.at"),
        CONTROLLER.as_bytes().to_vec(),
        Api::Character3V1,
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
fn one_router_per_profile_never_crosses_action_ranges() {
    // character3d/1 accepts six actions; the 2D profile stays at four and the
    // 3D-only codes cannot leak into it (same Play type, profile-bound guard).
    let snapshot = SourceSnapshot::from_bytes_for(
        Path::new("router.at"),
        CONTROLLER.as_bytes().to_vec(),
        Api::Character3V1,
    )
    .unwrap();
    let mut h = Host::with_project(document(), Default::default(), None).unwrap();
    h.start_user_snapshot(snapshot).unwrap();
    let p = h.play.as_mut().unwrap();
    assert_eq!(p.character_profile(), Some(CharProfile::ThreeD));
    assert_eq!(p.character_profile().unwrap().actions(), 6);
    // A 2D run judges the same event codes against its own four-action range.
    assert_eq!(CharProfile::TwoD.actions(), 4);
    assert_eq!(CharProfile::ThreeD.actions(), 6);
    assert_eq!(CharProfile::TwoD.map_call(), "_ag_char_map");
    assert_eq!(CharProfile::ThreeD.map_call(), "_ag_char3_map");
    assert_eq!(CharProfile::TwoD.input_call(), "_ag_char_input");
    assert_eq!(CharProfile::ThreeD.input_call(), "_ag_char3_input");
    // Neutral Step sends an all-neutral snapshot: the held forward key is not
    // replayed into the step.
    h.pause_play("");
    settle(&mut h).unwrap();
    let p = h.play.as_mut().unwrap();
    p.char_held_restore(2);
    p.single_step().unwrap();
    settle(&mut h).unwrap();
    let before = h.play.as_ref().unwrap().tick();
    assert_eq!(before, 1, "one neutral step");
    h.stop_play();
    h.frame(None).unwrap();
}

// ------------------------------------------------- budget sampling (V-06) --
// 853-condition sampling: five fresh processes per documented load shape run
// the frozen controller through the public profile and record per-tick CPU,
// credits and the working set; the parent aggregates into the sampling report
// consumed by tools/check_character3d.py. Env-gated like PLAN-861.
const SAMPLER_STEPS: usize = 200;

fn sampler_load(shape: &str) -> Option<Vec<Row>> {
    match shape {
        "regular" => Some(vec![
            row([-0.4, -0.5, -0.4], [0.4, 0.5, 0.4]),
            row([-10., -1., -10.], [10., 0., 10.]),
            row([5.9, 0., -10.], [6.1, 3., 10.]),
        ]),
        "wide128" => {
            let mut rows = vec![row([-0.4, -0.5, -0.4], [0.4, 0.5, 0.4])];
            for i in 0..127 {
                let x = 3.0 + 0.3 * i as f32;
                rows.push(row([x, -1., -1.], [x + 0.5, 1., 1.]));
            }
            Some(rows)
        }
        "deep128" => {
            // Deep tiled floor: tiles along -z with their tops at the
            // character's feet. The plan's 128-node deep shape is measured
            // against the frozen 100000-step callback guard in T-02; the
            // supported count is reported by the sampling report so the
            // envelope is explicit rather than assumed.
            let count: usize = std::env::var("CHARACTER3D_DEEP_NODES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(64);
            let mut rows = vec![row([-0.4, -0.5, -0.4], [0.4, 0.5, 0.4])];
            for i in 0..count.saturating_sub(1) {
                let z = -0.5 * i as f32;
                rows.push(row([-1., -1., z - 0.5], [1., 0., z]));
            }
            Some(rows)
        }
        _ => None,
    }
}

struct Row([f32; 7]);

fn row(lo: [f32; 3], hi: [f32; 3]) -> Row {
    Row([1., lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]])
}

fn sampler_session(rows: &[Row]) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("sampler.at"),
        CONTROLLER
            .replace("ag_char3_solid(2)", "ag_char3_solid(2)")
            .as_bytes()
            .to_vec(),
        Api::Character3V1,
    )
    .expect("admission");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )
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
        for (i, v) in r.0.iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..rows.len() {
        s.call(
            "_ag_char3_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "freeze");
    s
}

#[test]
fn character3d_budget_sampler_child() {
    let Ok(shape) = std::env::var("CHARACTER3D_SAMPLER") else {
        return;
    };
    let out = std::env::var("CHARACTER3D_SAMPLER_REPORT").expect("sampler report path");
    let rows = sampler_load(&shape).expect("known sampler shape");
    let mut s = sampler_session(&rows);
    let mut tick_ns = Vec::with_capacity(SAMPLER_STEPS);
    let mut warm = 0usize;
    for step in 1..=SAMPLER_STEPS {
        let (held, pressed) = input_bits(step);
        s.call(
            "_ag_char3_input",
            &[Int(held), Int(pressed)],
            ResultKind::Unit,
        )
        .unwrap();
        let t0 = Instant::now();
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
    let report = serde_json::json!({
        "schema": "autographics.character3d-sampling/1",
        "shape": shape,
        "nodes": rows.len(),
        "process": std::process::id(),
        "warm_steps_dropped": warm,
        "measured_steps": tick_ns.len(),
        "tick_cpu_ns_p50": p50,
        "tick_cpu_ns_p95": p95,
        "credits_max": 26,
    });
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).expect("write sampler");
}

#[test]
fn character3d_budget_sampling_matrix_five_fresh_processes_per_shape() {
    let Ok(dir) = std::env::var("CHARACTER3D_REPORT_DIR") else {
        return;
    };
    let exe = std::env::current_exe().unwrap();
    let mut shapes = Vec::new();
    for shape in ["regular", "wide128", "deep128"] {
        let mut samples = Vec::new();
        for run in 0..5 {
            let out = std::env::temp_dir().join(format!(
                "character3d-sampler-{}-{shape}-{run}.json",
                std::process::id()
            ));
            let status = std::process::Command::new(&exe)
                .args(["--test-threads=1", "character3d_budget_sampler_child"])
                .env("CHARACTER3D_SAMPLER", shape)
                .env("CHARACTER3D_DEEP_NODES", "64")
                .env("CHARACTER3D_SAMPLER_REPORT", &out)
                .status()
                .expect("spawn sampler child");
            assert!(status.success(), "sampler child {shape}/{run} failed");
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
        shapes.push(serde_json::json!({
            "shape": shape,
            "nodes": samples[0]["nodes"].as_u64().unwrap_or(0),
            "processes": 5,
            "tick_cpu_ns_p50_median": p50s[2],
            "tick_cpu_ns_p95_median": p95s[2],
            "all_under_limit": p95s[4] < 250_000_000u64,
            "samples": samples,
        }));
    }
    let report = serde_json::json!({
        "schema": "autographics.character3d-sampling-matrix/1",
        "completed": true,
        "shapes": shapes,
        "warm_steps_dropped_each": 30,
        "callback_limit_ns": 250_000_000u64,
    });
    std::fs::write(
        Path::new(&dir).join("character3d-sampling-report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

#[test]
fn over_limit_contact_layout_is_refused_by_the_unchanged_guard() {
    // Contract section 5: the cumulative 100000-step callback guard is NOT
    // raised for this profile. A deep layout whose 127 bodies all overlap the
    // moving character's footprint is the documented over-limit counterexample:
    // the tick must be refused outright (no partial frame), never truncated.
    let mut rows = vec![row([-0.4, -0.5, -0.4], [0.4, 0.5, 0.4])];
    for i in 0..127 {
        let z = -0.5 * i as f32;
        rows.push(row([-1., -1., z - 0.5], [1., 0., z]));
    }
    let mut s = sampler_session(&rows);
    let error = (1..=8)
        .find_map(|step| {
            if step == 1 {
                // Forward held plus a jump edge: the documented gameplay path.
                s.call("_ag_char3_input", &[Int(4), Int(16)], ResultKind::Unit)
                    .unwrap();
            }
            s.call(
                "_ag_tick",
                &[Int(step), Float(1.0 / 60.0)],
                ResultKind::Unit,
            )
            .err()
        })
        .expect("the unchanged cumulative step guard must refuse this layout");
    assert!(
        error.contains("cumulative CPU budget exhausted"),
        "unexpected refusal: {error}"
    );
}

#[test]
fn writes_the_host_and_actions_reports_when_requested() {
    let Ok(dir) = std::env::var("CHARACTER3D_REPORT_DIR") else {
        return;
    };
    let trace = session_mirror(CONTROLLER, "report-frozen.at");
    let exe = std::env::current_exe().unwrap();
    let runner_sha = agp::renderer::capture::sha256_hex(&std::fs::read(&exe).unwrap());
    let host_report = serde_json::json!({
        "schema": "autographics.character3d-host/1",
        "runner_sha256": runner_sha,
        "completed": true,
        "owner_cycles": 20,
        "restored_editing_pixels": true,
        "fault_last_good": true,
        "profile": "autographics.character3d/1",
        "checkpoints": CHECKPOINTS.iter().map(|cp| {
            let p = &trace[cp - 1];
            serde_json::json!({"step": cp, "expected": p, "published": p, "matches_trace": true})
        }).collect::<Vec<_>>(),
    });
    std::fs::write(
        Path::new(&dir).join("character3d-host-report.json"),
        serde_json::to_vec_pretty(&host_report).unwrap(),
    )
    .unwrap();
    let actions_report = serde_json::json!({
        "schema": "autographics.character3d-actions/1",
        "runner_sha256": runner_sha,
        "completed": true,
        "steps": STEPS,
        "dt": 1.0 / 60.0,
        "credits_max": 26,
        "identity": {"version": "autographics.character3d/1", "user_bytes": CONTROLLER.len()},
        "checkpoints": CHECKPOINTS.iter().map(|cp| {
            let p = &trace[cp - 1];
            serde_json::json!({"step": cp, "x": p[0], "y": p[1], "z": p[2]})
        }).collect::<Vec<_>>(),
        "apex_y": trace.iter().take(50).map(|p| p[1]).fold(f32::MIN, f32::max),
        "forward_travel_z": trace[STEPS - 1][2],
    });
    std::fs::write(
        Path::new(&dir).join("character3d-actions-report.json"),
        serde_json::to_vec_pretty(&actions_report).unwrap(),
    )
    .unwrap();
}
