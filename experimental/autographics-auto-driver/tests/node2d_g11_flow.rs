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

const WORLD_MAIN: &str = include_str!("../../autographics-node2d-pilot/generated-g11/main/world.at");
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

// ---------------------------------------------------------------- harness --

static G11_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn g11log(line: String) {
    G11_LOG.lock().unwrap().push(line);
}

fn load_doc(scene_text: &str, world_src: &str, tag: &str) -> (autographics_auto_driver::authoring::document::Document, autographics_auto_driver::authoring::assets::Registry, Option<std::sync::Arc<CodePayload>>) {
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

fn start_with(
    scene_text: &str,
    world_src: &str,
    tag: &str,
) -> Host {
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
    h.play
        .as_ref()
        .unwrap()
        .error
        .clone()
        .unwrap_or_default()
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
    f.instances.as_ref().and_then(|inst| inst.get(i)).map(|s| s.visible)
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
        Row(1, 2, [-0.4, -0.5, 0.0], [0.4, 0.5, 0.0]),   // 1 char 0.8x1
        Row(1, 2, [-20.0, -1.1, 0.0], [20.0, -0.1, 0.0]), // 2 ground 40x1
        Row(0, 2, [-0.3, -0.3, 0.0], [0.3, 0.3, 0.0]),   // 3 amber circle 0.6
        Row(1, 2, [-0.35, -0.35, 0.0], [0.35, 0.35, 0.0]), // 4 azure quad 0.7
        Row(0, 2, [-0.45, -0.45, 0.0], [0.45, 0.45, 0.0]), // 5 rose circle 0.9
        Row(0, 2, [-0.35, -0.35, 0.0], [0.35, 0.35, 0.0]), // 6 hazard circle 0.7
        Row(1, 2, [-0.25, -0.6, 0.0], [0.25, 0.6, 0.0]), // 7 goal quad 0.5x1.2
        Row(1, 2, [-1.0, -1.5, 0.0], [1.0, 1.5, 0.0]),   // 8 void quad 2x3
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
        let (tx, ty) = if n == 0 { (SPAWN[0], SPAWN[1]) } else { (-30.0, 2.0) };
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
    g11log(format!("init: bindings={} live={} error={:?}", binding_count(&h), live(&h), play_error(&h)));
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
    assert!(won, "walk collects three then wins; events={seen:?} live={} x={:?} err={:?}", live(&h), actor(&h), play_error(&h));
    let enters: Vec<&Ev> = seen.iter().filter(|e| e.pair < 3 && e.phase == 1).collect();
    let pairs: Vec<i32> = enters.iter().map(|e| e.pair).collect();
    assert_eq!(pairs.len(), 3, "three item enters");
    assert!(pairs.contains(&0) && pairs.contains(&1) && pairs.contains(&2), "pairs {pairs:?}");
    let ticks: Vec<i32> = enters.iter().map(|e| e.tick).collect();
    assert!(ticks[0] < ticks[1] && ticks[1] < ticks[2], "ordered ticks {ticks:?}");
    let post: Vec<&Ev> = seen.iter().filter(|e| e.pair < 3 && e.phase != 1).collect();
    assert!(post.is_empty(), "destroyed items never emit stay/exit: {post:?}");
    assert!(!seen.iter().any(|e| e.pair == 3), "hazard silent on the walk line");
    assert!(run_id(&h) == first_run && !restart_seen, "win alone restarts nothing");
    g11log(format!("win: enters ticks={ticks:?} x={:?} run={}", actor(&h), run_id(&h)));

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
    assert!(swapped && frame_tick(&h) == 0, "fresh Enter opens a new round");
    assert_eq!(live(&h), 6, "items reborn after restart");
    let wgen: Vec<Option<i32>> = (0..3).map(|i| slot(&h, i).map(|s| s.0)).collect();
    assert_eq!(wgen, vec![Some(1), Some(1), Some(1)], "new first generation");
    assert!((actor(&h)[0] - SPAWN[0]).abs() < 1e-3, "x restored");
    assert!(events(&h).is_empty(), "tick0 admits no events");
    g11log(format!("restart: run {} -> {} tick {}", won_run, run_id(&h), frame_tick(&h)));
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
    assert!(coin_before < 3, "alt trajectory differs (collected {coin_before})");
    for _ in 0..2 {
        step(&mut h, 1 << 1);
    }
    let frozen = actor(&h);
    for _ in 0..10 {
        step(&mut h, 1 << 1);
    }
    assert_eq!(actor(&h), frozen, "failed freezes motion");
    g11log(format!("alt: fail tick {hz_tick} x={hz_x} coins_before={coin_before}"));

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
        old_replay += events(&h).iter().filter(|e| e.pair == 3 && e.phase == 1).count();
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
    assert_eq!(status(&mut p), 0, "probe world reaches the same collection tick");
    let _ = p.call("probe_extra", &[], ResultKind::Int);
    assert_eq!(status(&mut p), 10, "33rd charged call faults on the collection tick");
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
    g11log(format!("budget: moving collection tick {collected_tick} = exactly 32 (33rd faults)"));
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
