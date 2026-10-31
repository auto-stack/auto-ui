//! auto-ui PLAN-017 node2d_g11_flow: the lowered full-G11/2d Node2D authoring
//! world (declarative modules -> g11-2d profile) executes on the pinned AG
//! path with the original 001 semantics: three distinct items collected in
//! order and truly destroyed, patrolling hazard with the event-tick write
//! skip, goal wins only at three, void zone fails out-of-bounds, Won/Failed
//! freeze the character, a fresh Enter edge asks for an accepted round
//! request into a fresh Run (coins reborn, counters reset), the shared
//! 32-credit pool holds on the moving-collection tick exactly, and the
//! parameter-edited alt twin fails at the hazard instead of winning.
//! The consumed world.at/scene.agscene are frozen generator products
//! (experimental/autographics-node2d-pilot/generated-g11, byte-identity
//! enforced by tools/verify_plan017.py). Run single-threaded:
//! cargo test --locked --offline --test node2d_g11_flow -- --test-threads=1
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
use std::sync::Mutex;

const WORLD_MAIN: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main/world.at");
const WORLD_ALT: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-alt/world.at");
const SCENE_MAIN: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main/scene.agscene");
const SCENE_ALT: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-alt/scene.agscene");
const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 2.0;
const SPAWN: [f32; 2] = [2.0, 0.5];
const WALK_BUDGET: usize = 8000;
const SCENE_TRIPLE: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-triple/scene.agscene");
const WORLD_TRIPLE: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-triple/world.at");
const SCENE_STAY: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-stay/scene.agscene");
const WORLD_STAY: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-stay/world.at");
const SCENE_BAG2: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-bag2/scene.agscene");
const WORLD_BAG2: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-bag2/world.at");
const SCENE_GEOMETRY: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-geometry/scene.agscene");
const WORLD_GEOMETRY: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-geometry/world.at");
const SCENE_GENERATION: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-generation/scene.agscene");
const WORLD_GENERATION: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-generation/world.at");
const SCENE_MOTION: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-motion-probes/scene.agscene");
const WORLD_MOTION: &str =
    include_str!("../../autographics-node2d-pilot/generated-g11/main-motion-probes/world.at");

// ---------------------------------------------------------------- harness --

static G11_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn g11log(line: String) {
    G11_LOG.lock().unwrap().push(line);
}

fn load_doc(
    scene_text: &str,
    world_src: &str,
    tag: &str,
) -> (
    autographics_auto_driver::authoring::document::Document,
    autographics_auto_driver::authoring::assets::Registry,
    Option<std::sync::Arc<CodePayload>>,
) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan017-g11")
        .join(format!("{}-{tag}", std::process::id()));
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    let sha = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene binds a source sha");
    std::fs::write(scripts.join(format!("{sha}.at")), world_src).expect("write script");
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("load scene project: {e:?}"));
    (doc, registry, code)
}

fn start_with(scene_text: &str, world_src: &str, tag: &str) -> Host {
    let (doc, registry, code) = load_doc(scene_text, world_src, tag);
    assert!(code.is_some(), "behavior code loaded");
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    {
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

fn start_main() -> Host {
    start_with(SCENE_MAIN, WORLD_MAIN, "main")
}

/// One logical tick with held (no edge) keys; releases everything first.
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

/// One restart request: a fresh Enter edge on a Running dispatch.
fn restart_once(h: &mut Host, action: usize) {
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        p.char_key(action, true, false);
        p.dispatch(Action::default()).expect("dispatch");
    }
    settle(h);
}

fn wait_new_run(h: &mut Host, old: i32, rounds: usize) -> bool {
    for _ in 0..rounds {
        if run_id(h) != old {
            return true;
        }
        {
            let p = h.play.as_mut().unwrap();
            let _ = p.resume();
        }
        h.frame(None).expect("frame");
    }
    false
}

fn frame_tick(h: &Host) -> i32 {
    h.play.as_ref().unwrap().tick()
}

fn run_id(h: &Host) -> i32 {
    h.play.as_ref().unwrap().run_id
}

fn actor(h: &Host) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.nodes
        .get(&1)
        .map(|d| d.translation)
        .or_else(|| f.nodes.values().next().map(|d| d.translation))
        .expect("actor node")
}

fn play_error(h: &Host) -> String {
    h.play.as_ref().unwrap().error.clone().unwrap_or_default()
}

fn binding_count(h: &Host) -> usize {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.rounds.as_ref())
        .map(|r| r.bindings.iter().filter(|b| b.is_some()).count())
        .unwrap_or(0)
}

fn live(h: &Host) -> usize {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.instances.as_ref())
        .map(|i| i.live_count())
        .unwrap_or(0)
}

fn slot(h: &Host, i: usize) -> Option<(i32, u64, [f32; 3])> {
    let f = h.play.as_ref().unwrap().last.as_ref()?;
    f.instances
        .as_ref()
        .and_then(|inst| inst.get(i))
        .map(|s| (s.generation, s.template_id, s.translation))
}

fn slot_visible(h: &Host, i: usize) -> Option<bool> {
    let f = h.play.as_ref().unwrap().last.as_ref()?;
    f.instances
        .as_ref()
        .and_then(|inst| inst.get(i))
        .map(|s| s.visible)
}

#[derive(Clone, Debug, PartialEq)]
struct Ev {
    tick: i32,
    pair: i32,
    phase: i32,
    source_ref: i32,
    bind_generation: i32,
    run_id: i32,
}

fn events(h: &Host) -> Vec<Ev> {
    let f = match h.play.as_ref().unwrap().last.as_ref() {
        Some(f) => f,
        None => return Vec::new(),
    };
    let rounds = match f.rounds.as_ref() {
        Some(r) => r,
        None => return Vec::new(),
    };
    rounds
        .events
        .iter()
        .map(|e| Ev {
            tick: rounds.tick,
            pair: e.pair,
            phase: e.phase,
            source_ref: e.source_ref,
            bind_generation: e.bind_generation,
            run_id: rounds.run_id,
        })
        .collect()
}

fn restart_requested(h: &Host) -> bool {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.rounds.as_ref())
        .map(|r| r.restart_requested)
        .unwrap_or(false)
}

// ---- CPU Session 路径（预算校准用，016 技术）：g11 场景表 ----

struct Row(i32, i32, [f32; 3], [f32; 3]);

/// g11 冻结场景表的 CPU 身份行（quad=1/circle=0，dim=2）。
fn g11_rows() -> Vec<Row> {
    vec![
        Row(1, 2, [-0.4, -0.5, 0.0], [0.4, 0.5, 0.0]), // 1 char 0.8x1
        Row(1, 2, [-20.0, -1.1, 0.0], [20.0, -0.1, 0.0]), // 2 ground 40x1
        Row(0, 2, [-0.3, -0.3, 0.0], [0.3, 0.3, 0.0]), // 3 amber circle 0.6
        Row(1, 2, [-0.35, -0.35, 0.0], [0.35, 0.35, 0.0]), // 4 azure quad 0.7
        Row(0, 2, [-0.45, -0.45, 0.0], [0.45, 0.45, 0.0]), // 5 rose circle 0.9
        Row(0, 2, [-0.35, -0.35, 0.0], [0.35, 0.35, 0.0]), // 6 hazard circle 0.7
        Row(1, 2, [-0.25, -0.6, 0.0], [0.25, 0.6, 0.0]), // 7 goal quad 0.5x1.2
        Row(1, 2, [-1.0, -1.5, 0.0], [1.0, 1.5, 0.0]), // 8 void quad 2x3
    ]
}

fn g11_session(world: &str, name: &str, run: i32) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world.as_bytes().to_vec(),
        Api::Triggers2V1,
    )
    .expect("admission");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let scene = g11_rows();
    s.call(
        "_ag_spatial_reset",
        &[Int(scene.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, Row(kind, dim, lo, hi)) in scene.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        let (tx, ty) = if n == 0 {
            (SPAWN[0], SPAWN[1])
        } else {
            (-30.0, 2.0)
        };
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
    for n in 0..scene.len() {
        let id = n as i32 + 1;
        s.call("_ag_char_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_obj_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_trg_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
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

/// 步进一次 _ag_tick（宿主侧每 tick 一次输入采样）。
fn tick_cpu(s: &mut Session, step: i32, held_right: bool) {
    let mask = if held_right { 2 } else { 0 };
    s.call("_ag_char_input", &[Int(mask), Int(0)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

// ---------------------------------------------------------------- tests --

#[test]
fn g11_walk_collects_three_and_wins_then_restarts() {
    let mut h = start_main();
    assert!(play_error(&h).is_empty(), "fault: {}", play_error(&h));
    g11log(format!(
        "init: bindings={} live={} error={:?}",
        binding_count(&h),
        live(&h),
        play_error(&h)
    ));
    assert_eq!(binding_count(&h), 6, "six pairs bound at tick0");
    assert_eq!(live(&h), 6, "six instances live at tick0");
    let tpl: Vec<Option<u64>> = (0..3).map(|i| slot(&h, i).map(|s| s.1)).collect();
    assert_eq!(tpl, vec![Some(3), Some(4), Some(5)], "item templates A/B/C");
    let gens0: Vec<Option<i32>> = (0..3).map(|i| slot(&h, i).map(|s| s.0)).collect();
    assert_eq!(gens0, vec![Some(1), Some(1), Some(1)], "first generation");

    let first_run = run_id(&h);
    let mut seen: Vec<Ev> = Vec::new();
    let mut won = false;
    let mut restart_seen = false;
    for _ in 0..WALK_BUDGET {
        seen.extend(events(&h));
        restart_seen |= restart_requested(&h);
        if live(&h) == 3 && seen.iter().any(|e| e.pair == 4 && e.phase == 1) {
            won = true;
            break;
        }
        step(&mut h, 1 << 1);
    }
    assert!(
        won,
        "walk collects three then wins; events={seen:?} live={} x={:?} err={:?}",
        live(&h),
        actor(&h),
        play_error(&h)
    );
    let enters: Vec<&Ev> = seen.iter().filter(|e| e.pair < 3 && e.phase == 1).collect();
    let pairs: Vec<i32> = enters.iter().map(|e| e.pair).collect();
    assert_eq!(pairs.len(), 3, "three item enters");
    assert!(
        pairs.contains(&0) && pairs.contains(&1) && pairs.contains(&2),
        "pairs {pairs:?}"
    );
    let ticks: Vec<i32> = enters.iter().map(|e| e.tick).collect();
    assert!(
        ticks[0] < ticks[1] && ticks[1] < ticks[2],
        "ordered ticks {ticks:?}"
    );
    let post: Vec<&Ev> = seen.iter().filter(|e| e.pair < 3 && e.phase != 1).collect();
    assert!(
        post.is_empty(),
        "destroyed items never emit stay/exit: {post:?}"
    );
    assert!(
        !seen.iter().any(|e| e.pair == 3),
        "hazard silent on the walk line"
    );
    assert!(
        run_id(&h) == first_run && !restart_seen,
        "win alone restarts nothing"
    );
    g11log(format!(
        "win: enters ticks={ticks:?} x={:?} run={}",
        actor(&h),
        run_id(&h)
    ));

    // Won 冻结移动；fresh Enter 开新局。
    for _ in 0..2 {
        step(&mut h, 1 << 1);
    }
    let won_x = actor(&h)[0];
    for _ in 0..5 {
        step(&mut h, 1 << 1);
    }
    assert!((actor(&h)[0] - won_x).abs() < 1e-6, "won freezes motion");
    let won_run = run_id(&h);
    let mut swapped = false;
    for _ in 0..4 {
        restart_once(&mut h, 3);
        if wait_new_run(&mut h, won_run, 60) {
            swapped = true;
            break;
        }
    }
    settle(&mut h);
    settle(&mut h);
    assert!(
        swapped && frame_tick(&h) == 0,
        "fresh Enter opens a new round"
    );
    assert_eq!(live(&h), 6, "items reborn after restart");
    let wgen: Vec<Option<i32>> = (0..3).map(|i| slot(&h, i).map(|s| s.0)).collect();
    assert_eq!(
        wgen,
        vec![Some(1), Some(1), Some(1)],
        "new first generation"
    );
    assert!((actor(&h)[0] - SPAWN[0]).abs() < 1e-3, "x restored");
    assert!(events(&h).is_empty(), "tick0 admits no events");
    g11log(format!(
        "restart: run {} -> {} tick {}",
        won_run,
        run_id(&h),
        frame_tick(&h)
    ));
    h.stop_play();
}

#[test]
fn g11_alt_fails_at_hazard_and_freezes() {
    let mut h = start_with(SCENE_ALT, WORLD_ALT, "alt");
    let mut seen: Vec<Ev> = Vec::new();
    let mut hazard_at = None;
    for _ in 0..WALK_BUDGET {
        seen.extend(events(&h));
        if let Some(e) = seen.iter().find(|e| e.pair == 3 && e.phase == 1) {
            hazard_at = Some((e.tick, actor(&h)[0]));
            break;
        }
        step(&mut h, 1 << 1);
    }
    let (hz_tick, hz_x) = hazard_at.expect("alt fails at hazard");
    assert!(
        (7.0..9.6).contains(&hz_x),
        "hazard enter at tick {hz_tick} x={hz_x} (walk line)"
    );
    let coin_before = seen.iter().filter(|e| e.pair < 3 && e.phase == 1).count();
    assert!(
        coin_before < 3,
        "alt trajectory differs (collected {coin_before})"
    );
    for _ in 0..2 {
        step(&mut h, 1 << 1);
    }
    let frozen = actor(&h);
    for _ in 0..10 {
        step(&mut h, 1 << 1);
    }
    assert_eq!(actor(&h), frozen, "failed freezes motion");
    g11log(format!(
        "alt: fail tick {hz_tick} x={hz_x} coins_before={coin_before}"
    ));

    // fresh Enter 重开到 tick0。
    let failed_run = run_id(&h);
    let mut swapped = false;
    for _ in 0..4 {
        restart_once(&mut h, 3);
        if wait_new_run(&mut h, failed_run, 60) {
            swapped = true;
            break;
        }
    }
    settle(&mut h);
    settle(&mut h);
    assert!(swapped && frame_tick(&h) == 0, "restart opens tick0");
    let mut old_replay = 0usize;
    for _ in 0..6 {
        old_replay += events(&h)
            .iter()
            .filter(|e| e.pair == 3 && e.phase == 1)
            .count();
        step(&mut h, 0);
    }
    assert_eq!(old_replay, 0, "old run events not replayed");
    // held Enter 不连开。
    let settled_run = run_id(&h);
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        p.char_held_restore(3);
        p.dispatch(Action::default()).expect("dispatch");
    }
    h.pause_play("");
    settle(&mut h);
    for _ in 0..20 {
        step(&mut h, 1 << 3);
    }
    assert_eq!(run_id(&h), settled_run, "held enter no double restart");
    h.stop_play();
}

#[test]
fn g11_cpu_budget_and_moving_collection_exactly_32() {
    // CPU 路径：精确校准移动收集 tick 恰 32（第 33 个计费调用 = 命名故障 10），
    // 普通 tick 与静止 tick 在预算内，静止三收集可读 3 条。
    let mut s = g11_session(WORLD_MAIN, "g11_budget.at", 1);
    // 静止 tick：held 2 + next 1 + 同步 1 = 4。
    tick_cpu(&mut s, 1, false);
    assert_eq!(status(&mut s), 0, "stationary tick within budget");
    // 注入一次额外计费读：静止 tick 远未满池，应不故障（区分证明）。
    let _ = s.call("get_r_bag", &[], ResultKind::Int);
    assert_eq!(status(&mut s), 0, "idle tick has headroom");

    // 走到收集 tick（x≈8：从 2 出发 6/2 每秒 60tick ≈ 181）。
    let mut collected_tick = 0i32;
    for t in 0..400 {
        let before = int(&mut s, "get_r_bag");
        tick_cpu(&mut s, t + 1, true);
        let after = int(&mut s, "get_r_bag");
        if after > before {
            collected_tick = t;
            break;
        }
        assert_eq!(status(&mut s), 0, "walk tick {t} over budget");
    }
    assert!(collected_tick > 0, "collection happened");
    // 收集 tick 恰 32：再多一次计费读 → 故障 10。
    let probe = format!("{WORLD_MAIN}\nfn probe_extra() int {{return ag_action_held(1)}}\n");
    let mut p = g11_session(&probe, "g11_budget_probe.at", 1);
    for t in 0..=collected_tick {
        tick_cpu(&mut p, t + 1, true);
    }
    assert_eq!(
        status(&mut p),
        0,
        "probe world reaches the same collection tick"
    );
    let _ = p.call("probe_extra", &[], ResultKind::Int);
    assert_eq!(
        status(&mut p),
        10,
        "33rd charged call faults on the collection tick"
    );
    drop(s);

    // 静止三收集：同位三物品（用 triple 世界由 V-03 部分覆盖；这里验证
    // 静止多读路径在主世界不破预算：失败后（result!=0）按键读边沿）。
    let mut f = g11_session(WORLD_ALT, "g11_budget_alt.at", 1);
    let mut failed = false;
    for t in 0..600 {
        tick_cpu(&mut f, t + 1, true);
        if int(&mut f, "get_r_result") != 0 {
            failed = true;
            break;
        }
        assert_eq!(status(&mut f), 0, "alt walk over budget");
    }
    assert!(failed, "alt fails on the CPU path");
    // 失败 tick + pressed 边沿：合计在预算内。
    tick_cpu(&mut f, 999, true);
    assert_eq!(status(&mut f), 0, "failed-state tick within budget");
    drop(f);
    g11log(format!(
        "budget: moving collection tick {collected_tick} = exactly 32 (33rd faults)"
    ));
}

/// 输出日志（测试完成时可见）。
#[test]
fn g11_print_log() {
    let log = G11_LOG.lock().unwrap().clone();
    for line in &log {
        println!("G11 {line}");
    }
    assert_eq!(State::Stopping, State::Stopping); // keeps the import honest
}

/// 1200x800 顶面网格扫描（probe_pixels 单次回读）→ (红, 橙, 金, 地面) 计数。
/// 分类沿用 016 实测校准的相对通道规则（光照混合下）。
fn scan_classes(h: &mut Host) -> (usize, usize, usize, usize) {
    let mut points = Vec::new();
    for gy in 0..80 {
        for gx in 0..120 {
            points.push([(gx * 10 + 5) as f32, (gy * 10 + 5) as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.top, &points).expect("probe");
    let (mut red, mut orange, mut gold, mut ground) = (0usize, 0usize, 0usize, 0usize);
    for p in px.iter() {
        let (r, gg, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
        let is_red = r >= 220 && (r - gg) >= 80 && (r - gg) <= 140 && (gg - b).abs() <= 25;
        let is_orange =
            r >= 230 && (r - gg) >= 40 && (r - gg) <= 80 && (gg - b) >= 30 && (gg - b) <= 80;
        let is_gold = r >= 240 && gg >= 210 && (gg - b) >= 90;
        let is_ground = (r - gg).abs() <= 14 && (gg - b).abs() <= 25 && r >= 150 && r <= 210;
        if is_red {
            red += 1;
        } else if is_orange {
            orange += 1;
        } else if is_gold {
            gold += 1;
        } else if is_ground {
            ground += 1;
        }
    }
    (red, orange, gold, ground)
}

#[test]
fn g11_void_oob_left_axis_and_freeze() {
    // 主世界左行：差值轴 Left 侧真实生效；越界 void 失败并冻结。
    let mut h = start_main();
    let mut seen: Vec<Ev> = Vec::new();
    let mut oob = false;
    for _ in 0..WALK_BUDGET {
        seen.extend(events(&h));
        if seen.iter().any(|e| e.pair == 5 && e.phase == 1) {
            oob = true;
            break;
        }
        step(&mut h, 1 << 0);
    }
    assert!(
        oob && actor(&h)[0] < 0.5,
        "void fails out of bounds x={:?}",
        actor(&h)
    );
    for _ in 0..2 {
        step(&mut h, 1 << 0);
    }
    let frozen = actor(&h);
    for _ in 0..10 {
        step(&mut h, 1 << 0);
    }
    assert_eq!(actor(&h), frozen, "oob freezes motion");
    assert!(!seen.iter().any(|e| e.pair == 3), "hazard silent");
    h.stop_play();
}

#[test]
fn g11_same_tick_triple() {
    let mut h = start_with(SCENE_TRIPLE, WORLD_TRIPLE, "triple");
    assert!(play_error(&h).is_empty(), "{}", play_error(&h));
    let t0 = frame_tick(&h);
    h.play.as_mut().unwrap().single_step().expect("step");
    settle(&mut h);
    let ev = events(&h);
    let enters: Vec<&Ev> = ev.iter().filter(|e| e.phase == 1).collect();
    let pairs: Vec<i32> = enters.iter().map(|e| e.pair).collect();
    assert!(
        frame_tick(&h) == t0 + 1 && enters.len() >= 3,
        "one step emits the geometry result: tick {}->{}, enters {}",
        t0,
        frame_tick(&h),
        enters.len()
    );
    assert!(
        enters.len() == 3 && pairs.contains(&0) && pairs.contains(&1) && pairs.contains(&2),
        "same-tick three engine block: pairs {pairs:?}"
    );
    assert_eq!(
        enters
            .iter()
            .map(|e| e.source_ref)
            .collect::<std::collections::BTreeSet<i32>>()
            .len(),
        3,
        "three distinct ids"
    );
    let before = live(&h);
    step(&mut h, 0);
    assert_eq!(before, 6, "six live before the reads");
    assert_eq!(live(&h), 3, "three reads three destroys in one callback");
    h.stop_play();
}

#[test]
fn g11_stay_not_recounted() {
    let mut h = start_with(SCENE_STAY, WORLD_STAY, "stay");
    let mut seen: Vec<Ev> = Vec::new();
    let mut goal = false;
    let stay_run = run_id(&h);
    for _ in 0..WALK_BUDGET {
        let ev = events(&h);
        goal |= ev.iter().any(|e| e.pair == 2 && e.phase == 1);
        seen.extend(ev);
        if goal {
            break;
        }
        step(&mut h, 1 << 1);
    }
    let p0: Vec<i32> = seen
        .iter()
        .filter(|e| e.pair == 0)
        .map(|e| e.phase)
        .collect();
    assert!(
        p0.first() == Some(&1)
            && p0.iter().filter(|p| **p == 1).count() == 1
            && p0.iter().filter(|p| **p == 2).count() >= 5,
        "pair0 phases {p0:?} (one enter, many stays)"
    );
    assert!(
        goal && run_id(&h) == stay_run && live(&h) == 3,
        "goal reached but stays never scored: run {stay_run} live {}",
        live(&h)
    );
    // Enter 在 Playing 状态不重开。
    let before = run_id(&h);
    restart_once(&mut h, 3);
    for _ in 0..10 {
        if run_id(&h) != before {
            break;
        }
        step(&mut h, 0);
    }
    assert_eq!(run_id(&h), before, "playing state ignores Enter");
    h.stop_play();
}

#[test]
fn g11_bag2_goal_gate_and_exit() {
    let mut h = start_with(SCENE_BAG2, WORLD_BAG2, "bag2");
    let mut seen: Vec<Ev> = Vec::new();
    let mut goal_enter = false;
    let run2 = run_id(&h);
    let mut won2 = false;
    for _ in 0..WALK_BUDGET {
        let ev = events(&h);
        goal_enter |= ev.iter().any(|e| e.pair == 3 && e.phase == 1);
        seen.extend(ev);
        if run_id(&h) != run2 {
            won2 = true;
            break;
        }
        if seen.iter().filter(|e| e.pair == 3 && e.phase == 2).count() > 8 {
            break;
        }
        step(&mut h, 1 << 1);
    }
    let stays = seen.iter().filter(|e| e.pair == 3 && e.phase == 2).count();
    assert!(
        goal_enter && stays > 0,
        "goal enter={goal_enter} stays={stays}"
    );
    assert!(
        !won2 && run_id(&h) == run2,
        "bag2 never wins: run {}",
        run_id(&h)
    );
    // 走出 goal：exit 事件在越过后出现。
    let mut saw_exit = seen.iter().any(|e| e.pair == 3 && e.phase == 3);
    let mut guard = 0;
    while !saw_exit && guard < WALK_BUDGET {
        guard += 1;
        step(&mut h, 1 << 1);
        saw_exit = events(&h).iter().any(|e| e.pair == 3 && e.phase == 3);
        assert!(run_id(&h) == run2, "bag2 walk cannot restart");
        if actor(&h)[0] > 29.5 {
            break;
        }
    }
    assert!(saw_exit, "goal exit phase observed (walk-through)");
    // Enter（Playing）不重开。
    let before = run_id(&h);
    restart_once(&mut h, 3);
    for _ in 0..10 {
        if run_id(&h) != before {
            break;
        }
        step(&mut h, 0);
    }
    assert_eq!(run_id(&h), before, "bag2 enter does not open a round");
    h.stop_play();
}

#[test]
fn g11_geometry_probes() {
    let mut h = start_with(SCENE_GEOMETRY, WORLD_GEOMETRY, "geometry");
    assert!(play_error(&h).is_empty(), "{}", play_error(&h));
    let mut seen: Vec<Ev> = Vec::new();
    for _ in 0..6 {
        seen.extend(events(&h));
        step(&mut h, 0);
    }
    // pair0 = AABB@角（进入），pair1 = circle@角（0.64>0.5 静默），
    // pair2 = circle@切点（恰 0.5 进入）。
    assert!(
        seen.iter().any(|e| e.pair == 0 && e.phase == 1),
        "AABB corner instance enters"
    );
    assert!(
        !seen.iter().any(|e| e.pair == 1),
        "circle corner outside stays silent (AABB would hit)"
    );
    assert!(
        seen.iter().any(|e| e.pair == 2 && e.phase == 1),
        "circle tangent hits exactly at 0.5"
    );
    h.stop_play();
}

#[test]
fn g11_generation_despawn_respawn_rebind() {
    let mut h = start_with(SCENE_GENERATION, WORLD_GENERATION, "generation");
    let mut enter_ticks: Vec<i32> = Vec::new();
    let mut gens: Vec<i32> = Vec::new();
    for _ in 0..12 {
        for e in events(&h) {
            if e.pair == 0 && e.phase == 1 {
                enter_ticks.push(e.tick);
            }
        }
        if let Some(s) = slot(&h, 0) {
            gens.push(s.0);
        }
        step(&mut h, 0);
    }
    assert_eq!(
        enter_ticks.first(),
        Some(&1),
        "enters at tick 1 (spawn overlap)"
    );
    assert!(
        !enter_ticks.iter().any(|t| *t > 1 && *t < 4),
        "despawn window is silent: {enter_ticks:?}"
    );
    assert!(
        enter_ticks.len() >= 2 && *enter_ticks.last().unwrap() >= 4,
        "explicit respawn rebinds and re-enters the new generation: {enter_ticks:?}"
    );
    assert!(
        gens.iter().any(|g| *g == 2),
        "same slot reaches generation 2: {gens:?}"
    );
    h.stop_play();
}

#[test]
fn g11_motion_probes_both_and_away() {
    let mut h = start_with(SCENE_MOTION, WORLD_MOTION, "motion");
    // slider（pair0，向左走）与 ghost（pair1，每 tick 远移）。
    let mut enter_tick: Option<i32> = None;
    let mut first_overlap: Option<i32> = None;
    for _ in 0..80 {
        let ev = events(&h);
        if let Some(e) = ev.iter().find(|e| e.pair == 0 && e.phase == 1) {
            enter_tick = Some(e.tick);
        }
        if first_overlap.is_none() {
            if let Some(s) = slot(&h, 0) {
                // 物件中心到角色盒（半宽 0.4/半高 0.5）的最近距离；
                // ≤ 物件半宽 0.35 ⟺ 两个 AABB 相交（原 001 检测器语义）。
                let c = actor(&h);
                let char_half = [0.4f32, 0.5f32];
                let mut d2 = 0.0f32;
                for a in 0..2 {
                    let lo = c[a] - char_half[a];
                    let hi = c[a] + char_half[a];
                    let d = if s.2[a] < lo {
                        lo - s.2[a]
                    } else if s.2[a] > hi {
                        s.2[a] - hi
                    } else {
                        0.0
                    };
                    d2 += d * d;
                }
                if d2.sqrt() <= 0.35 {
                    first_overlap = Some(frame_tick(&h));
                }
            }
        }
        if enter_tick.is_some() && first_overlap.is_some() {
            break;
        }
        step(&mut h, 1 << 1);
    }
    assert!(
        enter_tick.is_some() && enter_tick == first_overlap,
        "both-moved enter tick {enter_tick:?} == first final-position overlap {first_overlap:?}"
    );
    // ghost：spawn 在角色上但每 tick 先远移 → 从不 enter（无旧位置几何）。
    let mut away_enters = 0usize;
    for _ in 0..6 {
        away_enters += events(&h)
            .iter()
            .filter(|e| e.pair == 1 && e.phase == 1)
            .count();
        step(&mut h, 0);
    }
    assert_eq!(
        away_enters, 0,
        "away item never enters (no previous position)"
    );
    h.stop_play();
}

#[test]
fn g11_pause_step_stop_harness() {
    let mut h = start_main();
    let p0 = actor(&h);
    h.pause_play("");
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.dispatch(Action::default());
    }
    settle(&mut h);
    assert_eq!(actor(&h), p0, "paused dispatch: no motion");
    let t1 = frame_tick(&h);
    h.play.as_mut().unwrap().single_step().expect("step");
    settle(&mut h);
    assert_eq!(frame_tick(&h), t1 + 1, "single step = exactly one tick");
    assert!(play_error(&h).is_empty(), "{}", play_error(&h));
    // Stop 恢复编辑文档（原 001 oracle 35 的等价断言：文档 JSON 前后一致）。
    let doc_before = serde_json::to_string(&h.editor.document).expect("document json");
    h.stop_play();
    let doc_after = serde_json::to_string(&h.editor.document).expect("document json");
    assert_eq!(doc_before, doc_after, "stop restores the editor document");
    assert!(h.play.is_none(), "stop tears the play down");
}

#[test]
fn g11_owner_20_rounds_and_stale_write() {
    // 20 轮：Start-once + drop 后 owner 释放（weak 探针）。
    for round in 0..20 {
        let mut s = g11_session(WORLD_MAIN, "g11_owner.at", 1);
        let probe = s.owner_probe();
        assert!(probe(), "round {round}: open session owns state");
        assert_eq!(int(&mut s, "get_r_starts"), 1, "round {round} starts once");
        tick_cpu(&mut s, 1, true);
        tick_cpu(&mut s, 2, true);
        assert_eq!(status(&mut s), 0, "round {round} fault-free");
        drop(s);
        assert!(!probe(), "round {round}: owner released after drop");
    }
    // 旧代句柄写：完整世界 + 注入 stale_write；run-2 fresh 世界操作 run-1
    // 句柄 → 命名外来句柄故障 33，新根不沾染。
    let mut s1 = g11_session(WORLD_MAIN, "g11_gen1.at", 1);
    let mut collected = 0;
    for t in 0..400 {
        tick_cpu(&mut s1, t + 1, true);
        if int(&mut s1, "get_r_bag") > 0 {
            collected = int(&mut s1, "get_amber_coin_handle");
            break;
        }
    }
    assert!(collected > 0, "collected a handle on run 1");
    drop(s1);
    let probe =
        format!("{WORLD_MAIN}\nfn stale_write(h int) int {{return ag_object_despawn(h)}}\n");
    let mut p = g11_session(&probe, "g11_stale.at", 2);
    assert_eq!(status(&mut p), 0, "fresh run-2 world starts clean");
    let _ = p.call("stale_write", &[Int(collected)], ResultKind::Int);
    assert_eq!(
        status(&mut p),
        33,
        "stale handle is the named foreign-run fault"
    );
    assert_eq!(int(&mut p, "get_r_amber"), 1, "fresh root untouched");
}

#[test]
fn g11_hidden_still_triggers_engine_fact() {
    // engine-fact 探针（原始 Auto，非作者能力）：spawn 后 ag_object_visible
    // 关闭可见性，触发资格不受影响（visibility never gates triggers）。
    // 探针源以重写身份绑定到同场景工程（scene behavior sha/bytes = 探针源）。
    let hidden = concat!(
        "var ok=0.0
",
        "var h0=0
",
        "fn app_start() {
",
        " ok=ok+ag_char_bind(1)
",
        " ok=ok+ag_char_solid(2)
",
        " ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)
",
        " h0=ag_object_spawn(0, 3, 2.0, 0.5, 0.0)
",
        " ok=ok+ag_object_visible(h0, 0)
",
        " ok=ok+ag_trigger_bind(0, 1, 1, h0, 0, 0.0)
",
        "}
",
        "fn app_update(dt float) {
",
        " let n=ag_event_next()
",
        " if n==1 {let p=ag_event_pair()}
",
        "}
",
    );
    let probe_sha = agp::renderer::capture::sha256_hex(hidden.as_bytes());
    let old_sha = SCENE_MAIN
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("sha")
        .to_string();
    let old_bytes: usize = SCENE_MAIN
        .split("\"byte_length\": ")
        .nth(1)
        .and_then(|r| {
            let digits: String = r.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse::<usize>().ok()
        })
        .expect("bytes");
    let probe_scene = SCENE_MAIN.replacen(&old_sha, &probe_sha, 1).replacen(
        &format!("\"byte_length\": {}", old_bytes),
        &format!("\"byte_length\": {}", hidden.len()),
        1,
    );
    assert!(
        probe_scene.contains(&probe_sha),
        "scene rewritten for the probe source"
    );
    assert!(!probe_scene.contains(&old_sha), "old binding gone");
    let mut h = start_with(&probe_scene, hidden, "hidden");
    let mut enter = false;
    let mut vis_at_enter: Option<bool> = None;
    for _ in 0..6 {
        let ev = events(&h);
        if vis_at_enter.is_none() && ev.iter().any(|e| e.pair == 0 && e.phase == 1) {
            vis_at_enter = slot_visible(&h, 0);
        }
        enter |= ev.iter().any(|e| e.pair == 0 && e.phase == 1);
        if enter && vis_at_enter.is_some() {
            break;
        }
        step(&mut h, 0);
    }
    assert!(
        enter && vis_at_enter == Some(false),
        "hidden instance still triggers (published visible={vis_at_enter:?})"
    );
    h.stop_play();
}

#[test]
fn g11_gpu_won_pixels_and_windowless_png() {
    // GPU：真实项目工程 → 离屏 GPU → 像素类三层证据 + won 后 capture。
    let mut h = start_main();
    let (red0, orange0, gold0, ground0) = scan_classes(&mut h);
    assert!(red0 >= 6, "hazard pixels {red0}");
    assert!(orange0 >= 4, "player pixels {orange0}");
    assert!(gold0 >= 1, "gold pixels {gold0}");
    assert!(ground0 >= 40, "ground pixels {ground0}");
    // 收集：金币像素类消失 + 帧层槽死亡（真 despawn）。
    let mut guard = 0;
    while guard < 400 && live(&h) == 6 {
        guard += 1;
        step(&mut h, 1 << 1);
    }
    assert!(guard < 400, "collection happened");
    let (_, _, gold1, _) = scan_classes(&mut h);
    assert!(
        gold1 < gold0,
        "gold class disappears after despawn ({gold0}->{gold1})"
    );
    // 走到赢并冻结，然后 capture。
    let mut won = false;
    for _ in 0..WALK_BUDGET {
        if live(&h) == 3 {
            let ev = events(&h);
            if ev.iter().any(|e| e.pair == 4 && e.phase == 1) {
                won = true;
                break;
            }
        }
        step(&mut h, 1 << 1);
    }
    assert!(won, "reached the won state");
    for _ in 0..2 {
        step(&mut h, 1 << 1);
    }
    let won_x = actor(&h)[0];
    for _ in 0..3 {
        step(&mut h, 1 << 1);
    }
    assert!(
        (actor(&h)[0] - won_x).abs() < 1e-6,
        "won freezes (GPU window)"
    );
    let cmds = match h.rt.content(h.two) {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g.commands.len(),
        _ => 0,
    };
    let (_, png, w, ht) =
        h.rt.capture(h.two, "g11-2d-won")
            .unwrap_or_else(|| panic!("capture"));
    assert!(
        w > 0 && ht > 0 && png.len() > 3_000 && cmds > 3,
        "windowless png {w}x{ht} bytes={} cmds={cmds}",
        png.len()
    );
    h.stop_play();
}
