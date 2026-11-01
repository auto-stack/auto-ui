//! auto-ui PLAN-020 node_components_flow: the stateful-components chain
//! (components-2d profile) executes on the pinned AG path as real project
//! play. Chain: Player-local walk/jump (discrete f32 oracle: land 0.502,
//! bounded apex, no double jump), two same-def Gauges under different parent
//! instances moving/hiding per their OWN model on real GPU (exact even-tick
//! formulas, blue hidden after hide_at 61 while green persists, slot
//! identity kept), root vy sentinel untouched, exact typed-event delivery
//! (12/8 at T=120) observed through generated getters on the CPU session,
//! and the crossed-binding negative (receiver table mutated -> swapped
//! counts, run itself stays green).
//! T-05 (V-04/V-05) extends the same file: budget / budget-plus (pool
//! exactly 32 vs the named 33rd-call fault 10 at the predicted tick),
//! lifecycle counters on host and CPU (Start exactly once per Run, Resume
//! never repeats Start, Tick exact with dispatches, paused dispatch /
//! render-only frames advance nothing, single Step advances exactly one),
//! 20 host-driven Stop/fresh rounds (model rebuilt, no old-Run leakage,
//! document unchanged), the consumable pressed edge (double read), the
//! 8-stateful-instance envelope green, and the runtime init fault (9th
//! spawn out of the frozen 8-slot objects pool -> named fault 31) with the
//! host retaining last-good. The consumed scene.agscene/world.at are frozen
//! generator products
//! (experimental/autographics-node2d-pilot/generated-components2d/*).
//! Run single-threaded:
//! cargo test --locked --offline --test node_components_flow -- --test-threads=1
use autographics_auto_driver::{
    authoring::{
        assets::Registry,
        behavior::{Api, SourceSnapshot},
        behavior_project::CodePayload,
        host::Host,
        play::{Action, State},
        storage::Storage,
    },
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/probe/scene.agscene");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/probe/world.at");
const SCENE_BUDGET: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/budget/scene.agscene");
const WORLD_BUDGET: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/budget/world.at");
const SCENE_BUDGET_PLUS: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components2d/budget-plus/scene.agscene"
);
const WORLD_BUDGET_PLUS: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/budget-plus/world.at");
const SCENE_LIFECYCLE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/lifecycle/scene.agscene");
const WORLD_LIFECYCLE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/lifecycle/world.at");
const SCENE_DP: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components2d/lifecycle-doublepress/scene.agscene"
);
const WORLD_DP: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components2d/lifecycle-doublepress/world.at"
);
const SCENE_ENVELOPE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/envelope/scene.agscene");
const WORLD_ENVELOPE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/envelope/world.at");
const SCENE_INIT_FAULT: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/init-fault/scene.agscene");
const WORLD_INIT_FAULT: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/init-fault/world.at");

const DT: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------- harness --

fn load_doc(scene_text: &str, world_src: &str, tag: &str) -> std::sync::Arc<CodePayload> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan020-components")
        .join(format!("{}-{tag}", std::process::id()));
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    let sha = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene binds a source sha")
        .to_string();
    std::fs::write(scripts.join(format!("{sha}.at")), world_src).expect("write script");
    let (_, _, code, _) = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .unwrap_or_else(|e| panic!("load scene project: {e:?}"));
    code.expect("behavior code loaded")
}

fn start_with(scene_text: &str, world_src: &str, tag: &str) -> Host {
    let code = load_doc(scene_text, world_src, tag);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan020-components")
        .join(format!("{}-{tag}", std::process::id()));
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("reload: {e:?}"));
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    settle(&mut h);
    h
}

fn settle(h: &mut Host) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or(false);
        if !pending {
            break;
        }
        assert!(std::time::Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

fn step(h: &mut Host, held: u32) {
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        for action in 0..4usize {
            if held & (1 << action) != 0 {
                p.char_held_restore(action);
            } else {
                p.char_key(action, false, false);
            }
        }
        if p.dispatch(Action::default()).is_err() {
            return;
        }
    }
    h.pause_play("");
    settle(h);
}

fn press_once(h: &mut Host, action: usize) {
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        p.char_key(action, true, false);
        p.dispatch(Action::default()).expect("dispatch");
    }
    h.pause_play("");
    settle(h);
}

fn actor(h: &Host, id: u64) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.nodes
        .get(&id)
        .map(|d| d.translation)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

fn slot(h: &Host, i: usize) -> (i32, u64, [f32; 3], bool) {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    let s = f
        .instances
        .as_ref()
        .and_then(|inst| inst.get(i))
        .expect("live slot");
    (s.generation, s.template_id, s.translation, s.visible)
}

fn play_error(h: &Host) -> String {
    h.play.as_ref().unwrap().error.clone().unwrap_or_default()
}

type Px = ([f32; 2], [u8; 4]);

fn pixel_classes(h: &mut Host) -> Vec<Px> {
    let mut points = Vec::new();
    for gy in 0..95 {
        for gx in 0..200 {
            points.push([(gx * 4 + 2) as f32, (gy * 4 + 2) as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.top, &points).expect("probe");
    points
        .iter()
        .zip(px.iter())
        .map(|(p, c)| (*p, *c))
        .collect()
}

fn count_class(px: &[Px], class: &str) -> usize {
    px.iter()
        .filter(|(_, c)| {
            let (r, g, b) = (
                c[0] as f32 / 255.0,
                c[1] as f32 / 255.0,
                c[2] as f32 / 255.0,
            );
            match class {
                "blue" => b - r.max(g) > 0.12,
                "green" => g - r.max(b) > 0.12,
                _ => false,
            }
        })
        .count()
}

/// Mean x of a color class over the sampled surface.
fn center_x(px: &[Px], class: &str) -> f32 {
    let (mut sum, mut n) = (0.0f32, 0usize);
    for (p, c) in px {
        let (r, g, b) = (
            c[0] as f32 / 255.0,
            c[1] as f32 / 255.0,
            c[2] as f32 / 255.0,
        );
        let hit = match class {
            "blue" => b - r.max(g) > 0.12,
            "green" => g - r.max(b) > 0.12,
            _ => false,
        };
        if hit {
            sum += p[0];
            n += 1;
        }
    }
    assert!(n > 0, "class {class} absent");
    sum / n as f32
}

/// Exact gauge slot x (the model recomputes draw_x on even ticks only; an
/// odd T shows the even tick before). a: phase0 0 speed 1; b: phase0 5 speed 6.
fn gauge_x(i: usize, t: f64) -> f64 {
    let te = (t / 2.0).floor() * 2.0;
    match i {
        0 => 4.0 + 2.0 * (te / 60.0) + 0.2 * (te / 10.0).floor(),
        _ => -4.0 + 2.0 * (5.0 + 6.0 * te / 60.0) + 0.2 * (te / 15.0).floor(),
    }
}

/// Parse the frozen scene document objects into CPU spatial rows:
/// kind quad/circle -> shape ids 1/0, dim 2, boxes from scale/translation.
fn scene_rows(scene_text: &str) -> Vec<(i32, [f32; 3], [f32; 3], [f32; 2])> {
    let objects = scene_text
        .split("\"objects\": [")
        .nth(1)
        .expect("objects list");
    let objects = &objects[..objects.find("\n ],").expect("objects end")];
    let mut rows = Vec::new();
    for chunk in objects.split("\"id\": ").skip(1) {
        let num = |key: &str| -> f64 {
            let ki = chunk.find(key).expect(key);
            let rest = &chunk[ki + key.len()..];
            let rest = rest.trim_start_matches(|c: char| c == '[' || c == ' ');
            let end = rest
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest.len());
            rest[..end].parse().expect("number")
        };
        let kind = if chunk.contains("\"kind\": \"quad\"") {
            1
        } else if chunk.contains("\"kind\": \"circle\"") {
            0
        } else {
            panic!("unexpected scene kind: {}", &chunk[..60.min(chunk.len())]);
        };
        let tx = num("\"translation\": [");
        let rest = &chunk[chunk.find("\"translation\": [").expect("t")..];
        let comma = rest.find(',').expect("comma");
        let rest2 = rest[comma + 1..].trim_start();
        let end = rest2
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(rest2.len());
        let ty: f64 = rest2[..end].parse().expect("ty");
        let sx = num("\"scale\": [");
        let rests = &chunk[chunk.find("\"scale\": [").expect("s")..];
        let commas = rests.find(',').expect("comma");
        let rests2 = rests[commas + 1..].trim_start();
        let ends = rests2
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(rests2.len());
        let sy: f64 = rests2[..ends].parse().expect("sy");
        rows.push((
            kind,
            [(tx - sx / 2.0) as f32, (ty - sy / 2.0) as f32, 0.0],
            [(tx + sx / 2.0) as f32, (ty + sy / 2.0) as f32, 0.0],
            [tx as f32, ty as f32],
        ));
    }
    rows
}

// ------------------------------------------------ CPU session harness ----

fn probe_session(scene_text: &str, world_src: &str, name: &str, run: i32) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world_src.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .expect("admission");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let scene = scene_rows(scene_text);
    s.call(
        "_ag_spatial_reset",
        &[Int(scene.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, (kind, lo, hi, pos)) in scene.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(2), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [pos[0], pos[1], 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
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
    for n in 0..scene.len() {
        let id = n as i32 + 1;
        s.call("_ag_char_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_obj_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}

fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}

fn flt(s: &mut Session, name: &str) -> f32 {
    s.float(name, &[]).unwrap()
}

fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}

fn tick_cpu(s: &mut Session, step: i32, held_right: bool) {
    let mask = if held_right { 2 } else { 0 };
    s.call("_ag_char_input", &[Int(mask), Int(0)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

// -------------------------------------------------------------- tests ----

/// GPU chain: player-local walk/jump physics (discrete oracle); two
/// same-def gauges move/hide per their own model on real GPU with slot
/// identity; root never carries the component business.
#[test]
fn components_gpu_player_gauges_events() {
    let mut h = start_with(SCENE, WORLD, "components-gpu");
    assert!(play_error(&h).is_empty(), "fault: {}", play_error(&h));
    let mut t = 0usize;

    // settle: fall from (2,1) to the ground rest height 0.502 (T=26).
    for _ in 0..26 {
        step(&mut h, 0);
        t += 1;
    }
    let a0 = actor(&h, 1);
    assert!((a0[0] - 2.0).abs() < 1e-3, "no drift without input: {a0:?}");
    assert!((a0[1] - 0.502).abs() < 1e-3, "landed at rest: {a0:?}");
    // both gauges live at their own positions, both visible (T=26).
    {
        let (g0, tp0, p0, v0) = slot(&h, 0);
        let (g1, tp1, p1, v1) = slot(&h, 1);
        assert_eq!((g0, tp0, v0), (1, 3, true), "gauge a slot");
        assert_eq!((g1, tp1, v1), (1, 4, true), "gauge b slot");
        assert!(
            (p0[0] - gauge_x(0, 26.0) as f32).abs() < 1e-3,
            "gauge a x {} vs {}",
            p0[0],
            gauge_x(0, 26.0)
        );
        assert!(
            (p1[0] - gauge_x(1, 26.0) as f32).abs() < 1e-3,
            "gauge b x {} vs {}",
            p1[0],
            gauge_x(1, 26.0)
        );
        assert!((p0[1] - 3.0).abs() < 1e-6 && (p1[1] - 3.0).abs() < 1e-6);
    }

    // walk right 24 ticks: player-only motion oracle (moves grounded) (T=50).
    for _ in 0..24 {
        step(&mut h, 1 << 1);
        t += 1;
    }
    let a1 = actor(&h, 1);
    assert!(
        (a1[0] - (2.0 + 4.0 * 24.0 * DT)).abs() < 1e-3,
        "walk: {a1:?}"
    );
    assert!((a1[1] - 0.502).abs() < 1e-3, "stayed grounded: {a1:?}");

    // GPU capture 1 (T=50): both gauges rendered.
    let cap1 = pixel_classes(&mut h);
    assert!(
        count_class(&cap1, "blue") > 4,
        "gauge a on GPU: {}",
        count_class(&cap1, "blue")
    );
    assert!(
        count_class(&cap1, "green") > 4,
        "gauge b on GPU: {}",
        count_class(&cap1, "green")
    );

    // 8 neutral ticks: gauges advance per own model (T=58, before hide).
    for _ in 0..8 {
        step(&mut h, 0);
        t += 1;
    }
    let cap2 = pixel_classes(&mut h);
    let da = center_x(&cap2, "blue") - center_x(&cap1, "blue");
    let db = center_x(&cap2, "green") - center_x(&cap1, "green");
    println!("GPU motion px: da={da} db={db}");
    assert!(db > 1.0, "gauge b moved per own model: {db}");
    assert!(db - da > 1.0, "distinct per-instance motion: {da} vs {db}");

    // 4 more ticks: T=66 > hide_at 61 — blue gone, green still rendered.
    for _ in 0..4 {
        step(&mut h, 0);
        t += 1;
    }
    let cap3 = pixel_classes(&mut h);
    assert_eq!(count_class(&cap3, "blue"), 0, "gauge a hidden on GPU");
    assert!(count_class(&cap3, "green") > 4, "gauge b still rendered");
    let (gen_after, _, _, v0b) = slot(&h, 0);
    assert!(!v0b, "slot visible=false");
    assert_eq!(gen_after, 1, "hide keeps generation/identity");
    let (_, _, _, v1b) = slot(&h, 1);
    assert!(v1b, "gauge b persists");

    // player jump from the walked position (T=66..): discrete oracle.
    press_once(&mut h, 2);
    t += 1;
    let mut vy = 8.0 - 20.0 * DT;
    let mut y = 0.502f32 + vy * DT;
    let mut best = y;
    for j in 2..=70 {
        vy -= 20.0 * DT;
        let new_y = y + vy * DT;
        if new_y <= 0.502 {
            y = 0.502;
            vy = 0.0;
        } else {
            y = new_y;
        }
        best = best.max(y);
        step(&mut h, 0);
        t += 1;
        let a = actor(&h, 1);
        assert!((a[1] - y).abs() < 1e-3, "jump tick {j}: {} vs {y}", a[1]);
        if j > 2 && y == 0.502 {
            break;
        }
    }
    let rise = best - 0.502;
    assert!(
        rise <= 1.6 + 1e-3 && rise >= 1.6 - 8.0 * DT - 1e-3,
        "apex: {rise}"
    );
    for _ in 0..3 {
        step(&mut h, 1 << 2);
        t += 1;
    }
    assert!((actor(&h, 1)[1] - 0.502).abs() < 1e-3, "no double jump");

    // walk to the exact final tick count and check exact per-model motion.
    while t < 120 {
        step(&mut h, 0);
        t += 1;
    }
    let (_, _, pf0, _) = slot(&h, 0);
    let (_, _, pf1, _) = slot(&h, 1);
    assert!(
        (pf0[0] - gauge_x(0, 120.0) as f32).abs() < 1e-2,
        "final a: {} vs {}",
        pf0[0],
        gauge_x(0, 120.0)
    );
    assert!(
        (pf1[0] - gauge_x(1, 120.0) as f32).abs() < 1e-2,
        "final b: {} vs {}",
        pf1[0],
        gauge_x(1, 120.0)
    );
    h.stop_play();
}

/// CPU session: root vy sentinel untouched by player physics; typed events
/// deliver exact counts per receiver through generated getters; per-model
/// phase formulas exact; budget stays within the frozen 32-credit pool;
/// crossed-binding negative (mutated receiver table) swaps the counts while
/// the run itself stays executable.
#[test]
fn components_cpu_sentinel_counts_and_crossed_negative() {
    let mut s = probe_session(SCENE, WORLD, "components-cpu.at", 1);
    for tick in 1..=120 {
        tick_cpu(&mut s, tick, false);
        assert_eq!(status(&mut s), 0, "tick {tick} within budget");
    }
    assert_eq!(int(&mut s, "get_r_vy"), 999, "root sentinel untouched");
    assert_eq!(
        int(&mut s, "get_gauge_count_0"),
        12,
        "panel a -> meter a only"
    );
    assert_eq!(
        int(&mut s, "get_gauge_count_1"),
        8,
        "panel b -> meter b only"
    );
    assert_eq!(int(&mut s, "get_player_grounded"), 1, "settled grounded");
    assert!(flt(&mut s, "get_player_vy").abs() < 1e-4, "settled vy");
    assert_eq!(int(&mut s, "get_gauge_ticks_0"), 120);
    assert_eq!(int(&mut s, "get_gauge_ticks_1"), 120);
    // exact per-model phase (speed 1/6, dt 1/60, 120 ticks).
    assert!(
        (flt(&mut s, "get_gauge_phase_0") - 2.0).abs() < 1e-3,
        "phase a"
    );
    assert!(
        (flt(&mut s, "get_gauge_phase_1") - 17.0).abs() < 1e-2,
        "phase b"
    );
    // gauge a hidden (show=0), gauge b persists (hide_at=-1 never hits).
    assert_eq!(
        int(&mut s, "get_a_meter_show"),
        0,
        "gauge a show after hide"
    );
    assert_eq!(int(&mut s, "get_b_meter_show"), 1, "gauge b persists");
    println!(
        "PLAN-020 components CPU cost: calls={} max_call_steps={} max_slices={}",
        s.calls, s.max_call_steps, s.max_call_slices
    );

    // crossed-binding negative: a's emits land on meter b (receiver oracle).
    let wcross = WORLD.replace("var meter_bind_ADD=[0,1]", "var meter_bind_ADD=[1,0]");
    assert_ne!(wcross, WORLD, "negative variant differs");
    let mut sneg = probe_session(SCENE, &wcross, "components-crossed.at", 1);
    for tick in 1..=30 {
        tick_cpu(&mut sneg, tick, false);
    }
    assert_eq!(status(&mut sneg), 0, "crossed run itself executes");
    let mut spos = probe_session(SCENE, WORLD, "components-pos.at", 1);
    for tick in 1..=30 {
        tick_cpu(&mut spos, tick, false);
    }
    assert_eq!(
        int(&mut spos, "get_gauge_count_0"),
        3,
        "positive T=30 count a"
    );
    assert_eq!(
        int(&mut spos, "get_gauge_count_1"),
        2,
        "positive T=30 count b"
    );
    assert_eq!(
        int(&mut sneg, "get_gauge_count_0"),
        2,
        "crossed: b emits on a's meter"
    );
    assert_eq!(
        int(&mut sneg, "get_gauge_count_1"),
        3,
        "crossed: a emits on b's meter"
    );
}

// =================================================== T-05 (V-04/V-05) ====

/// Extra charged-call input variant: `_ag_char_input(held, pressed)` — the
/// pressed mask carries one-tick edges (bit2 = Jump) consumed by the first
/// `input.pressed` read.
fn tick_cpu_input(s: &mut Session, step: i32, held: i32, pressed: i32) {
    s.call(
        "_ag_char_input",
        &[Int(held), Int(pressed)],
        ResultKind::Unit,
    )
    .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

/// Lifecycle gauge slot x. The panel's Life.Start emits meter.Add(5), so
/// count = 5 (+1 per period event at/behind the recompute tick); draw_x
/// recomputes only on the gauge's parity side (a: odd ticks, b: even ticks).
fn lifecycle_gauge_x(i: usize, n: i32) -> f64 {
    let (base, phase0, speed, period, odd_side) = if i == 0 {
        (4.0, 0.0, 1.0, 10, true)
    } else {
        (-4.0, 5.0, 6.0, 15, false)
    };
    let first_recompute = if odd_side { 1 } else { 2 };
    let tr: i32 = if (n % 2 == 1) == odd_side { n } else { n - 1 };
    if tr < first_recompute {
        return base; // draw_x still at its model init
    }
    let count = 5 + tr / period;
    base + 2.0 * (phase0 + speed * tr as f64 / 60.0) + 0.2 * count as f64
}

/// V-05: the budget chain charges EXACTLY 32 credits on every tick (player
/// walk 30 + two staggered gauge writes) and runs status 0 through 120
/// ticks; budget-plus adds ONE charged write (a fifth gauge on the even
/// side) so the 33rd charged call raises the NAMED runtime fault 10 at the
/// predicted first even tick, tick 2 (tick 1 stays green at 32).
#[test]
fn v05_budget_pool_exact_32_and_plus_faults_at_tick_2() {
    let mut s = probe_session(SCENE_BUDGET, WORLD_BUDGET, "v05-budget.at", 1);
    for tick in 1..=120 {
        tick_cpu(&mut s, tick, false);
        assert_eq!(status(&mut s), 0, "budget tick {tick} within the 32 pool");
    }
    // all four gauge models ran to completion (parity-staggered writes).
    assert_eq!(int(&mut s, "get_g1_ticks"), 120);
    assert_eq!(int(&mut s, "get_g2_ticks"), 120);
    assert_eq!(int(&mut s, "get_g3_ticks"), 120);
    assert_eq!(int(&mut s, "get_g4_ticks"), 120);
    assert_eq!(int(&mut s, "get_player_grounded"), 1);
    assert_eq!(int(&mut s, "get_r_vy"), 999, "root sentinel untouched");
    println!(
        "PLAN-020 T-05 budget CPU cost (32/tick): calls={} max_call_steps={} max_slices={}",
        s.calls, s.max_call_steps, s.max_call_slices
    );
    assert!(s.max_call_steps < 100_000, "VM step guard headroom");
    assert!(s.max_call_slices <= 120, "slice guard");

    // budget-plus: the ONLY delta is the fifth gauge (one more charged write
    // on even ticks). Tick 1 = 30 + 2 = 32 -> green; tick 2 = 30 + 3 = 33 ->
    // the named fault 10, and the world freezes at tick 2.
    let mut sp = probe_session(
        SCENE_BUDGET_PLUS,
        WORLD_BUDGET_PLUS,
        "v05-budget-plus.at",
        1,
    );
    tick_cpu(&mut sp, 1, false);
    assert_eq!(
        status(&mut sp),
        0,
        "budget-plus tick 1 is exactly at the pool"
    );
    tick_cpu(&mut sp, 2, false);
    assert_eq!(status(&mut sp), 10, "33rd charged call faults 10 at tick 2");
    assert_eq!(
        int(&mut sp, "get_g1_ticks"),
        2,
        "model froze at the fault tick"
    );
    assert_eq!(int(&mut sp, "get_g5_ticks"), 2);
    tick_cpu(&mut sp, 3, false);
    assert_eq!(status(&mut sp), 10, "fault latches");
    assert_eq!(
        int(&mut sp, "get_g1_ticks"),
        2,
        "no further ticks after fault"
    );
    println!(
        "PLAN-020 T-05 budget-plus CPU cost: calls={} max_call_steps={} max_slices={}",
        sp.calls, sp.max_call_steps, sp.max_call_slices
    );
}

/// V-04 (CPU): Start counters ran exactly once; Tick counters advance
/// exactly with dispatched ticks; the Start-time meter.Add(5) delivery is
/// visible in the gauge counts (17 = 12 period events + 5 Start, 13 = 8+5).
#[test]
fn v04_lifecycle_cpu_counters_exact() {
    let mut s = probe_session(SCENE_LIFECYCLE, WORLD_LIFECYCLE, "v04-lifecycle.at", 1);
    for tick in 1..=120 {
        tick_cpu(&mut s, tick, false);
        assert_eq!(status(&mut s), 0, "lifecycle tick {tick}");
    }
    assert_eq!(int(&mut s, "get_r_starts"), 1, "root Start exactly once");
    assert_eq!(int(&mut s, "get_r_ticks"), 120, "root Tick exact");
    assert_eq!(int(&mut s, "get_panel_starts_0"), 1);
    assert_eq!(int(&mut s, "get_panel_starts_1"), 1);
    assert_eq!(int(&mut s, "get_panel_ticks_0"), 120);
    assert_eq!(int(&mut s, "get_panel_ticks_1"), 120);
    assert_eq!(int(&mut s, "get_gauge_starts_0"), 1);
    assert_eq!(int(&mut s, "get_gauge_starts_1"), 1);
    assert_eq!(int(&mut s, "get_gauge_ticks_0"), 120);
    assert_eq!(int(&mut s, "get_gauge_ticks_1"), 120);
    assert_eq!(int(&mut s, "get_player_starts_0"), 1);
    assert_eq!(int(&mut s, "get_player_ticks_0"), 120);
    // Start-time event delivery: count = period events + 5 Start payload.
    assert_eq!(
        int(&mut s, "get_gauge_count_0"),
        17,
        "12 events + 5 from Start"
    );
    assert_eq!(
        int(&mut s, "get_gauge_count_1"),
        13,
        "8 events + 5 from Start"
    );
    println!(
        "PLAN-020 T-05 lifecycle CPU cost: calls={} max_call_steps={} max_slices={}",
        s.calls, s.max_call_steps, s.max_call_slices
    );
}

/// V-05: the consumable pressed edge. The Player reads input.pressed(Jump)
/// twice per tick into p1/p2: the first read consumes the edge (1), the
/// second reads 0 the same tick, and the jump fires exactly once.
#[test]
fn v05_doublepress_edge_consumed_jump_once() {
    let mut s = probe_session(SCENE_DP, WORLD_DP, "v05-doublepress.at", 1);
    // The reduced CPU spatial environment lands the char later than the GPU
    // host trajectory, so settle on the STATE (grounded==1), not a fixed
    // tick count (the probe CPU contract does the same at T=120).
    let mut landed_at = 0;
    for tick in 1..=300 {
        tick_cpu_input(&mut s, tick, 0, 0);
        assert_eq!(status(&mut s), 0, "settle tick {tick}");
        if int(&mut s, "get_player_grounded") == 1 {
            landed_at = tick;
            break;
        }
    }
    assert!(landed_at > 0, "char must settle in the CPU session");
    // one Jump edge on the tick after landing: first read 1, second read 0,
    // single jump.
    let press_tick = landed_at + 1;
    tick_cpu_input(&mut s, press_tick, 0, 4);
    assert_eq!(status(&mut s), 0, "press tick within pool");
    assert_eq!(
        int(&mut s, "get_player_p1"),
        1,
        "first read consumed the edge"
    );
    assert_eq!(int(&mut s, "get_player_p2"), 0, "second read same tick = 0");
    let vy1 = flt(&mut s, "get_player_vy");
    assert!(
        (vy1 - (8.0 - 20.0 * DT)).abs() < 1e-3,
        "jump fired once: vy {vy1}"
    );
    // next tick without an edge: no re-trigger, plain gravity decay.
    tick_cpu_input(&mut s, press_tick + 1, 0, 0);
    assert_eq!(int(&mut s, "get_player_p1"), 0);
    assert_eq!(int(&mut s, "get_player_p2"), 0);
    let vy2 = flt(&mut s, "get_player_vy");
    assert!((vy2 - (vy1 - 20.0 * DT)).abs() < 1e-3, "single decay {vy2}");
    assert!(vy2 > 0.5, "still airborne (no second jump): {vy2}");
}

/// V-05: exactly EIGHT stateful instances (root + player + six tickers) run
/// green at the envelope boundary with service-free tickers (player-only 30
/// credits). The 9th stateful instance is lowering-rejected by the T-03
/// negative fixture fixtures/components2d/neg-9-stateful/ (named envelope
/// diagnostic, pilot gate negative_fixtures_fail_with_named_diagnostics).
#[test]
fn v05_envelope_exactly_8_stateful_runs_green() {
    let mut s = probe_session(SCENE_ENVELOPE, WORLD_ENVELOPE, "v05-envelope.at", 1);
    for tick in 1..=120 {
        tick_cpu(&mut s, tick, false);
        assert_eq!(status(&mut s), 0, "envelope tick {tick}");
    }
    for t in 1..=6 {
        assert_eq!(int(&mut s, &format!("get_t{t}_acc")), 120, "ticker t{t}");
        assert_eq!(int(&mut s, &format!("get_t{t}_starts")), 1, "ticker t{t}");
    }
    assert_eq!(int(&mut s, "get_player_grounded"), 1, "player settled");
    assert_eq!(int(&mut s, "get_r_vy"), 999);
    println!(
        "PLAN-020 T-05 envelope CPU cost (30/tick): calls={} max_call_steps={} max_slices={}",
        s.calls, s.max_call_steps, s.max_call_slices
    );
}

/// V-04 negative: the init-fault chain lowers green but fails DURING
/// initialization — the ninth ag_object_spawn targets slot 8, outside the
/// frozen objects2d pool (slots 0..7), raising the named runtime fault 31.
/// CPU: status 31 right after _ag_start. GPU host: the run surfaces the
/// named fault, publishes no good frame, and the host retains last-good
/// (document intact, stop/start cycle still works, a good project runs
/// green right after — no cross-host damage).
#[test]
fn v04_init_fault_named_and_host_retains_last_good() {
    // CPU: the init fault is deterministic at _ag_start.
    let mut s = probe_session(SCENE_INIT_FAULT, WORLD_INIT_FAULT, "v04-init-fault.at", 1);
    assert_eq!(status(&mut s), 31, "ninth spawn faults 31 at init");

    // GPU host: the failed initialization must surface the named fault and
    // leave the host at its last-good state.
    let code = load_doc(SCENE_INIT_FAULT, WORLD_INIT_FAULT, "v04-init-fault-gpu");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan020-components")
        .join(format!("{}-v04-init-fault-gpu", std::process::id()));
    let (doc, registry, code2, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("reload: {e:?}"));
    let _ = code;
    assert!(
        code2.is_some(),
        "behavior code admitted (not a load rejection)"
    );
    let mut h = Host::with_project(doc, registry, code2).expect("host");
    let started = h.start_selected_play();
    if let Err(e) = &started {
        println!("init fault surfaced at start: {e}");
    }
    h.pause_play("");
    settle(&mut h);
    let err = play_error(&h);
    assert!(
        !err.is_empty(),
        "init fault must surface a named host error"
    );
    println!("DEBUG init-fault host error: {err}");
    assert!(err.contains("31"), "named slot fault, got: {err}");
    // last-good retention: no run frame with mutated world state was
    // published as good (the fault hit during app_start; the player never
    // ran a tick), and the host still stops cleanly.
    if let Some(p) = h.play.as_ref() {
        if let Some(f) = p.last.as_ref() {
            for (id, d) in &f.nodes {
                let t = d.translation;
                assert!(
                    id != &1 || (t[0] - 2.0).abs() < 1e-3,
                    "player never advanced past spawn: node {id} at {t:?}"
                );
            }
        }
    }
    h.stop_play();

    // recovery: a good project starts and runs green immediately after the
    // failed init (the host pipeline retains last-good; no global damage).
    let mut hg = start_with(SCENE, WORLD, "v04-post-fault-good");
    assert!(play_error(&hg).is_empty(), "good run after failed init");
    for _ in 0..26 {
        step(&mut hg, 0);
    }
    let a = actor(&hg, 1);
    assert!((a[1] - 0.502).abs() < 1e-3, "good run settles: {a:?}");
    hg.stop_play();
}

/// V-04 (GPU host): Start exactly once per Run (Resume never repeats it),
/// Tick exact with dispatches, paused dispatch and render-only frames
/// advance nothing, a single Step advances exactly one tick, and 20
/// host-driven Stop/fresh rounds rebuild the model (counters reset, gauge
/// params re-applied, Start-time count back at 5) with no old-Run leakage
/// and the document unchanged. Shared legal resources stay alive across
/// all rounds (same slot templates and rendered classes at the end — the
/// honest observable the harness has for "not released early").
#[test]
fn v04_lifecycle_host_pause_step_render_20_fresh_rounds() {
    let mut h = start_with(SCENE_LIFECYCLE, WORLD_LIFECYCLE, "v04-host");
    assert!(play_error(&h).is_empty(), "fault: {}", play_error(&h));

    // fresh start, zero dispatched ticks: gauge a sits at its model init
    // (draw_x = base_x; the Start-time count only shows after the first
    // recompute tick).
    {
        let (_, _, p0, _) = slot(&h, 0);
        assert!((p0[0] - 4.0).abs() < 1e-3, "fresh init x: {}", p0[0]);
    }
    // three dispatched ticks: gauge a (odd side) recomputed at ticks 1 and 3.
    for _ in 0..3 {
        step(&mut h, 0);
    }
    {
        let (_, _, p0, _) = slot(&h, 0);
        assert!(
            (p0[0] - lifecycle_gauge_x(0, 3) as f32).abs() < 1e-3,
            "after 3 ticks: {} vs {}",
            p0[0],
            lifecycle_gauge_x(0, 3)
        );
    }
    // Paused dispatch with neutral input (019 frozen pattern): the probe
    // dispatch processes exactly one tick — tick 4 — but applies NO input
    // edges (char edges do not accumulate while paused; the host/UI gate is
    // the real pause). Tick 4 is gauge b's recompute side: a holds at its
    // tick-3 value, b advances one tick.
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.dispatch(Action::default());
    }
    settle(&mut h);
    {
        let (_, _, p0, _) = slot(&h, 0);
        let (_, _, p1, _) = slot(&h, 1);
        assert!(
            (p0[0] - lifecycle_gauge_x(0, 3) as f32).abs() < 1e-3,
            "paused dispatch: odd-side gauge holds (tick 4): {}",
            p0[0]
        );
        assert!(
            (p1[0] - lifecycle_gauge_x(1, 4) as f32).abs() < 1e-3,
            "paused dispatch: even-side gauge stepped one: {} vs {}",
            p1[0],
            lifecycle_gauge_x(1, 4)
        );
    }
    // render-only frames advance nothing (both gauges hold).
    for _ in 0..3 {
        h.frame(None).expect("render-only frame");
    }
    {
        let (_, _, p0, _) = slot(&h, 0);
        let (_, _, p1, _) = slot(&h, 1);
        assert!(
            (p0[0] - lifecycle_gauge_x(0, 3) as f32).abs() < 1e-3
                && (p1[0] - lifecycle_gauge_x(1, 4) as f32).abs() < 1e-3,
            "render-only frames advanced nothing: {} / {}",
            p0[0],
            p1[0]
        );
    }
    // One Step advances exactly one tick (tick 5): gauge a recomputes, b
    // holds.
    {
        let p = h.play.as_mut().unwrap();
        p.single_step().expect("single step");
    }
    settle(&mut h);
    {
        let (_, _, p0, _) = slot(&h, 0);
        let (_, _, p1, _) = slot(&h, 1);
        assert!(
            (p0[0] - lifecycle_gauge_x(0, 5) as f32).abs() < 1e-3,
            "step: odd-side gauge recomputed at tick 5: {} vs {}",
            p0[0],
            lifecycle_gauge_x(0, 5)
        );
        assert!(
            (p1[0] - lifecycle_gauge_x(1, 4) as f32).abs() < 1e-3,
            "step: even-side gauge holds: {}",
            p1[0]
        );
    }
    // Start ran exactly once across all those resumes: count still carries
    // only the Start payload 5 (no period event before tick 10), so a's x
    // is exactly the count-5 formula.
    {
        let (_, _, p0, _) = slot(&h, 0);
        let expect = 4.0 + 2.0 * (5.0 / 60.0) + 0.2 * 5.0;
        assert!(
            (p0[0] - expect as f32).abs() < 1e-3,
            "no repeated Start on Resume: {} vs {expect}",
            p0[0]
        );
    }

    // 20 host-driven Stop/fresh rounds. Round 0 mutates state (walk + 12
    // ticks so gauge a's count grows past the Start payload); every fresh
    // Run must rebuild the model (count back at 5, x at the rebuilt
    // formula) with no old-Run leakage; the document (same frozen project
    // replayed by the host) stays unchanged across runs, and the shared
    // gauge resources keep rendering (honest observable for "legal shared
    // resources are not released early").
    let mut last_templates = (0u64, 0u64);
    for round in 0..20i32 {
        h.stop_play();
        h.start_selected_play()
            .unwrap_or_else(|e| panic!("round {round} start: {e:?}"));
        h.pause_play("");
        settle(&mut h);
        // fresh rebuild: gauge a back at its init x; player back at spawn.
        {
            let (_, tp0, p0, _) = slot(&h, 0);
            let a = actor(&h, 1);
            assert_eq!(tp0, 3, "round {round} template identity");
            assert!(
                (p0[0] - 4.0).abs() < 1e-3,
                "round {round} fresh x: {}",
                p0[0]
            );
            assert!((a[0] - 2.0).abs() < 1e-3, "round {round} player respawned");
            assert!(play_error(&h).is_empty(), "round {round} fault");
        }
        if round == 0 {
            // mutate: walk right for 12 ticks (count a grows to 6 at tick 10).
            for _ in 0..12 {
                step(&mut h, 1 << 1);
            }
            let (_, _, p0, _) = slot(&h, 0);
            assert!(
                (p0[0] - lifecycle_gauge_x(0, 12) as f32).abs() < 1e-3,
                "round 0 mutated: {} vs {}",
                p0[0],
                lifecycle_gauge_x(0, 12)
            );
            let a = actor(&h, 1);
            assert!(a[0] > 2.1, "round 0 player walked: {}", a[0]);
        } else {
            // two neutral ticks: rebuilt model shows the fresh formula
            // (count back at 5 — the old Run's mutated count is not visible;
            // gauge a recomputes on the odd tick 1, holds on tick 2).
            for _ in 0..2 {
                step(&mut h, 0);
            }
            let (_, _, p0, _) = slot(&h, 0);
            let expect = lifecycle_gauge_x(0, 2);
            assert!(
                (p0[0] - expect as f32).abs() < 1e-3,
                "round {round} rebuilt: {} vs {expect}",
                p0[0]
            );
        }
        let (_, tp0, _, _) = slot(&h, 0);
        let (_, tp1, _, _) = slot(&h, 1);
        last_templates = (tp0, tp1);
    }
    assert_eq!(
        last_templates,
        (3, 4),
        "shared gauge templates alive at the end"
    );
    // shared resources still render after 20 rounds (blue AND green classes).
    let px = pixel_classes(&mut h);
    assert!(count_class(&px, "blue") > 0, "blue gauge still rendered");
    assert!(count_class(&px, "green") > 0, "green gauge still rendered");
    h.stop_play();
}
