//! auto-ui PLAN-020 node_components_flow: the stateful-components chain
//! (components-2d profile) executes on the pinned AG path as real project
//! play. Chain: Player-local walk/jump (discrete f32 oracle: land 0.502,
//! bounded apex, no double jump), two same-def Gauges under different parent
//! instances moving/hiding per their OWN model on real GPU (exact even-tick
//! formulas, blue hidden after hide_at 61 while green persists, slot
//! identity kept), root vy sentinel untouched, exact typed-event delivery
//! (12/8 at T=120) observed through generated getters on the CPU session,
//! and the crossed-binding negative (receiver table mutated -> swapped
//! counts, run itself stays green). The consumed scene.agscene/world.at are
//! frozen generator products
//! (experimental/autographics-node2d-pilot/generated-components2d/probe).
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
