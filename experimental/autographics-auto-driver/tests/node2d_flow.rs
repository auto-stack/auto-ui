//! auto-ui PLAN-016 node2d_flow: the lowered minimal Node2D world (one
//! Player, two same-definition Hazards with independent state, one
//! destroyable Coin) executes on the pinned AG path with per-instance
//! Start-once semantics, an independent f32 trajectory oracle, real
//! input->sweep motion, one real trigger event -> Auto-side Collect ->
//! condition-delta real despawn, idempotent repeats, the shared 32-credit
//! pool with a 33rd-charged-call fault, and old-generation handle
//! isolation. The consumed world.at/scene.agscene are frozen generator
//! products (experimental/autographics-node2d-pilot/generated, byte-identity
//! enforced by tools/verify_plan016.py). Scene schema: autographics.scene/7 (no HUD). Run single-threaded:
//! cargo test --locked --offline --test node2d_flow -- --test-threads=1
use autographics_auto_driver::{
    authoring::{
        behavior::{Api, SourceSnapshot},
        host::Host,
        play::{Action, State},
        storage::Storage,
    },
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use autographics_auto_driver::authoring::assets::Registry;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const WORLD: &str = include_str!("../../autographics-node2d-pilot/generated/world.at");
const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 2.0;
const SPAWN_X: f32 = 2.0;
const SPAWN_Y: f32 = 1.0;

/// Frozen scene table (scene.agscene): char row 0 (player, id 1),
/// ground row 1 (id 2), coin template row 2 (id 3, parked),
/// hazard template row 3 (id 4, parked). Coin radius 0.3, char box
/// +-0.4/+-0.5, hazard box +-0.5/+-0.3.
struct Row(i32, i32, [f32; 3], [f32; 3]);

fn rows() -> Vec<Row> {
    vec![
        Row(1, 2, [-0.4, -0.5, 0.0], [0.4, 0.5, 0.0]),
        Row(1, 2, [-5.0, -1.0, 0.0], [35.0, 0.0, 0.0]),
        Row(0, 2, [-0.3, -0.3, 0.0], [0.3, 0.3, 0.0]),
        Row(1, 2, [-0.5, -0.3, 0.0], [0.5, 0.3, 0.0]),
    ]
}

fn session(world: &str, name: &str, run: i32) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world.as_bytes().to_vec(),
        Api::Triggers2V1,
    )
    .expect("admission of the lowered world");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let scene = rows();
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
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit).unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        // 行 0 = player 模板，在位 (2,1)；其余模板停靠玩法区外。
        let (tx, ty) = if n == 0 {
            (SPAWN_X, SPAWN_Y)
        } else {
            (-100.0, -100.0)
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

fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}
fn int(s: &mut Session, name: &str) -> i32 {
    s.int(name, &[]).unwrap()
}
fn float(s: &mut Session, name: &str) -> f32 {
    s.float(name, &[]).unwrap()
}
fn tick(s: &mut Session, step: i32) {
    // 宿主侧输入注入：每 tick 采样一次，按住 Right（held 位 1）。
    s.call("_ag_char_input", &[Int(2), Int(0)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

/// 巡逻相位独立 oracle（与 fixtures patrol.at 逐字同形）。
fn wave_phase(phase0: f32, speed: f32, ticks: i32) -> f32 {
    (phase0 + speed * DT * ticks as f32) % 0.8
}

/// 驱动一个世界直到收集发生（r_coin_present=0 的差量 tick）。
fn drive_to_collect(s: &mut Session) -> i32 {
    for t in 1..=120 {
        tick(s, t);
        assert_eq!(status(s), 0, "tick {t} fault-free");
        if int(s, "get_r_coin_present") == 0 {
            return t;
        }
    }
    panic!("coin not collected within 120 ticks");
}

#[test]
fn flow_minimal_world_instances_events() {
    let mut s = session(WORLD, "node2d_flow.at", 1);
    assert_eq!(status(&mut s), 0, "start fault-free");

    // Start-once per instance（V-02/V-04）。
    assert_eq!(int(&mut s, "get_r_starts"), 1);
    assert_eq!(int(&mut s, "get_player_starts"), 1);
    assert_eq!(int(&mut s, "get_h1_starts"), 1);
    assert_eq!(int(&mut s, "get_h2_starts"), 1);

    let collect_tick = drive_to_collect(&mut s);
    assert!(
        collect_tick >= 38 && collect_tick <= 60,
        "collect at {collect_tick}"
    );

    // 实例隔离（V-02）：同定义双 Hazard 相位/计数独立匹配独立 oracle；
    // 根同名 phase 哨兵不动。
    assert_eq!(int(&mut s, "get_h1_ticks"), collect_tick);
    assert_eq!(int(&mut s, "get_h2_ticks"), collect_tick);
    let h1 = float(&mut s, "get_h1_phase");
    let h2 = float(&mut s, "get_h2_phase");
    assert!(
        (h1 - wave_phase(0.1, 0.5, collect_tick)).abs() < 1e-4,
        "h1 phase {h1}"
    );
    assert!(
        (h2 - wave_phase(0.45, 0.9, collect_tick)).abs() < 1e-4,
        "h2 phase {h2}"
    );
    assert!((float(&mut s, "get_r_phase") - 0.5).abs() < 1e-6, "sentinel");
    // 真实运动（V-02）：char 节点位置精确随扫掠推进。
    let px = float(&mut s, "get_player_node_x");
    assert!(
        (px - (SPAWN_X + SPEED * DT * collect_tick as f32)).abs() < 1e-4,
        "player x {px} at tick {collect_tick}"
    );

    // 真实销毁（V-03）：条件差量 despawn；重复收集幂等；玩家继续走完窗口。
    assert_eq!(int(&mut s, "get_coin_alive"), 0);
    for t in collect_tick + 1..=120 {
        tick(&mut s, t);
    }
    assert_eq!(status(&mut s), 0, "window stays fault-free");
    assert_eq!(int(&mut s, "get_r_coin_present"), 0, "stays collected");
    assert_eq!(int(&mut s, "get_coin_alive"), 0, "stays destroyed");
    let px_end = float(&mut s, "get_player_node_x");
    assert!(
        (px_end - (SPAWN_X + SPEED * DT * 120.0)).abs() < 1e-3,
        "player walked the whole window: {px_end}"
    );

    // fresh run（V-04）：初态中性、Start 恰一次。
    let mut s2 = session(WORLD, "node2d_flow_fresh.at", 2);
    assert_eq!(status(&mut s2), 0);
    assert_eq!(int(&mut s2, "get_r_starts"), 1, "fresh run starts once");
    assert_eq!(int(&mut s2, "get_r_coin_present"), 1, "fresh run neutral");
    assert_eq!(int(&mut s2, "get_coin_alive"), 1, "fresh coin alive");
}

#[test]
fn flow_budget_full_pool_and_stale_generation() {
    // 预算（V-05 形状）：共享池收集 tick 恰满 32；同一回调窗口内再多一个
    // 计费调用即命名故障 10（直接计费读加在上一 tick 用量上）。
    {
        let probe = format!("{WORLD}\nfn probe_extra() int {{return ag_action_held(1)}}\n");
        let mut sf = session(&probe, "node2d_flow_budget.at", 4);
        let ctf = drive_to_collect(&mut sf);
        assert!(ctf > 0);
        sf.call("probe_extra", &[], ResultKind::Int).unwrap();
        assert_eq!(status(&mut sf), 10, "33rd charged call faults");
    }

    // 旧代写控制（V-03/V-04）：完整世界源 + 注入的旧代写包装（顶层 fn，
    // admission 合法），run-2 fresh 会话里对 run-1 句柄执行 despawn：
    // 命名外来句柄故障 33，且不污染新代根状态。
    let mut s1 = session(WORLD, "node2d_flow_gen1.at", 1);
    let _ct = drive_to_collect(&mut s1);
    let handle = int(&mut s1, "get_coin_handle");
    let probe = format!("{WORLD}\nfn stale_write(h int) int {{return ag_object_despawn(h)}}\n");
    let mut p = session(&probe, "node2d_flow_stale.at", 2);
    assert_eq!(status(&mut p), 0, "fresh run-2 world starts clean");
    p.call("stale_write", &[Int(handle)], ResultKind::Int).unwrap();
    assert_eq!(
        status(&mut p),
        33,
        "stale handle is the named foreign-run fault"
    );
    assert_eq!(int(&mut p, "get_r_coin_present"), 1, "fresh root untouched");
}

// ---- GPU 层（V-02/V-03/V-05）：真 Host/项目 → 离屏 GPU → 像素类证据 ----
// 渲染管线对材质色做光照/混合（实测校准：红(216,51,51)→(237,124,124)、
// 橙(229,127,63)→(243,188,137)、金(242,204,51)→(249,231,124)、地面灰
// (114,114,127)→(179,179,188)），分类用相对通道规则而非绝对 RGB。

const SCENE: &str = include_str!("../../autographics-node2d-pilot/generated/scene.agscene");

/// 1200x800 顶面网格扫描（probe_pixels 单次回读）。
/// 返回 (红, 橙, 金, 地面) 计数与橙/金质心 x。
fn scan(
    h: &mut Host,
) -> (
    usize,
    usize,
    usize,
    usize,
    (f32, usize),
    (f32, usize),
) {
    let mut points = Vec::new();
    for gy in 0..80 {
        for gx in 0..120 {
            points.push([(gx * 10 + 5) as f32, (gy * 10 + 5) as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.top, &points).expect("probe");
    let (mut red, mut orange, mut gold, mut ground) = (0usize, 0usize, 0usize, 0usize);
    let mut o_sum = 0f32;
    let mut o_n = 0usize;
    let mut g_sum = 0f32;
    let mut g_n = 0usize;
    for (i, p) in px.iter().enumerate() {
        let (r, gg, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
        let x = (i % 120 * 10 + 5) as f32;
        let is_red = r >= 220 && (r - gg) >= 80 && (r - gg) <= 140 && (gg - b).abs() <= 25;
        let is_orange =
            r >= 230 && (r - gg) >= 40 && (r - gg) <= 80 && (gg - b) >= 30 && (gg - b) <= 80;
        let is_gold = r >= 240 && gg >= 210 && (gg - b) >= 90;
        let is_ground =
            (r - gg).abs() <= 14 && (gg - b).abs() <= 25 && r >= 150 && r <= 210;
        if is_red {
            red += 1;
        } else if is_orange {
            orange += 1;
            o_sum += x;
            o_n += 1;
        } else if is_gold {
            gold += 1;
            g_sum += x;
            g_n += 1;
        } else if is_ground {
            ground += 1;
        }
    }
    (
        red,
        orange,
        gold,
        ground,
        (if o_n > 0 { o_sum / o_n as f32 } else { 0.0 }, o_n),
        (if g_n > 0 { g_sum / g_n as f32 } else { 0.0 }, g_n),
    )
}

fn host_tick(h: &mut Host) {
    let end = Instant::now() + Duration::from_secs(20);
    let p = h.play.as_mut().unwrap();
    p.resume().unwrap();
    p.char_key(1, true, false);
    p.dispatch(Action::default()).unwrap();
    h.pause_play("");
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "pump deadline");
        h.frame(None).unwrap();
    }
    assert_ne!(
        h.play.as_ref().unwrap().state,
        State::Faulted,
        "{:?}",
        h.play.as_ref().unwrap().error
    );
}

fn admitted_instances(h: &mut Host) -> autographics_auto_driver::authoring::runtime_objects::RuntimeObjectsFrame {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("admitted frame")
        .instances
        .clone()
        .expect("objects frame")
}

#[test]
fn flow_gpu_project_pixels_and_real_despawn() {
    // 物化生成工程：scene.agscene + scripts/<sha>.at（身份绑定）。
    let scene_json: serde_json::Value = serde_json::from_str(SCENE).expect("scene json");
    let sha = scene_json["behavior"]["source"]["sha256"]
        .as_str()
        .expect("sha")
        .to_string();
    let bytes = scene_json["behavior"]["source"]["byte_length"]
        .as_u64()
        .expect("bytes") as usize;
    assert_eq!(bytes, WORLD.len(), "scene binds world bytes");
    // target/ 为 gitignored 构建输出（项目目录按运行重建）。
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan016-gpu")
        .join(format!("{}", std::process::id()));
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), SCENE).expect("scene");
    std::fs::write(scripts.join(format!("{sha}.at")), WORLD).expect("script");

    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .expect("load pilot project");
    assert!(code.is_some(), "behavior code loaded");
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play().expect("start");
    h.pause_play("");
    let end = Instant::now() + Duration::from_secs(30);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "start pump deadline");
        h.frame(None).unwrap();
    }
    assert_ne!(
        h.play.as_ref().unwrap().state,
        State::Faulted,
        "{:?}",
        h.play.as_ref().unwrap().error
    );

    // 帧状态层：币 slot 2 存活，双 hazard slot 0/1 存活。
    let inst = admitted_instances(&mut h);
    assert!(inst.slots[2].live(), "coin slot live pre-collect");
    assert!(inst.slots[0].live() && inst.slots[1].live(), "hazards live");
    let hazard_y0 = inst.slots[0].translation[1];

    // 像素证据 A（tick 0）：四类全在。
    let (red_a, orange_a, gold_a, ground_a, player_a, coin_a) = scan(&mut h);
    assert!(red_a >= 6, "hazard pixels {red_a}");
    assert!(orange_a >= 4, "player pixels {orange_a}");
    assert!(gold_a >= 1, "coin pixels {gold_a}"); // 小圆 24px 在 10px 网格下 1-2 个采样
    assert!(ground_a >= 40, "ground pixels {ground_a}");

    // 驱动 20 tick：玩家质心右移（相机 clamp 段内投影恒定 40px/单位）。
    for _ in 0..20 {
        host_tick(&mut h);
    }
    let (red_b, orange_b, gold_b, _ground_b, player_b, coin_b) = scan(&mut h);
    assert!(red_b >= 6 && orange_b >= 4 && gold_b >= 1, "classes persist");
    let expected_dx = 40.0 * SPEED * DT * 20.0;
    assert!(
        (player_b.0 - player_a.0 - expected_dx).abs() < 12.0,
        "player centroid moved {} (expected ~{expected_dx})",
        player_b.0 - player_a.0
    );
    assert!((coin_a.0 - coin_b.0).abs() < 1.0, "coin static pre-collect");

    // 驱动到收集：帧状态层币槽死亡（真 despawn 非 hidden：hidden 保持
    // generation>0 且 visible=false；dead = 槽 free）。
    let mut guard = 0;
    while guard < 120 {
        guard += 1;
        host_tick(&mut h);
        if !admitted_instances(&mut h).slots[2].live() {
            break;
        }
    }
    assert!(guard < 120, "coin despawned within window");
    let inst = admitted_instances(&mut h);
    assert!(!inst.slots[2].live(), "coin slot free post-collect");
    assert!(
        inst.slots[0].live() && inst.slots[1].live(),
        "hazards unaffected"
    );
    // hazard 巡逻真的动了（独立运动层证据）。
    let hazard_y1 = inst.slots[0].translation[1];
    assert!(
        (hazard_y1 - hazard_y0).abs() > 0.01,
        "hazard patrols: {hazard_y0} -> {hazard_y1}"
    );

    // 像素证据 B：金色类消失，其余类保持（跨层交叉）。
    let (_r, _o, gold_c, ground_c, _pb, _cb) = scan(&mut h);
    assert_eq!(gold_c, 0, "coin gold gone from GPU");
    assert!(ground_c >= 40, "ground still rendered");
}

// ---- V-03 补充夹具：同定义双父实例，事件投递 receiver 隔离 ----

const WORLD_SUP: &str =
    include_str!("../../autographics-node2d-pilot/generated-supplement/world.at");

#[test]
fn flow_supplement_two_parent_receiver_isolation() {
    // 冻结场景表（补充）：char 行 0（player, id 1）、ground 行 1（id 2）、
    // coin 模板行 2（id 3, 停靠）。两个 coin 实例 = pair 0/1，slot 0/1。
    let mut s = session(WORLD_SUP, "node2d_flow_sup.at", 1);
    // 行集与 minimal 不同（无 hazard 模板行）：重建场景行（3 行）。
    drop(s);
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("node2d_flow_sup.at"),
        WORLD_SUP.as_bytes().to_vec(),
        Api::Triggers2V1,
    )
    .expect("supplement admission");
    let mut sup = Session::new(&snap.source).expect("supplement session");
    sup.call("main", &[], ResultKind::Unit).unwrap();
    sup.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    sup.call("_ag_spatial_reset", &[Int(3)], ResultKind::Unit)
        .unwrap();
    let scene: [(i32, [f32; 3], [f32; 3]); 3] = [
        (1, [-0.4, -0.5, 0.0], [0.4, 0.5, 0.0]),   // char
        (1, [-5.0, -1.0, 0.0], [35.0, 0.0, 0.0]),  // ground
        (0, [-0.3, -0.3, 0.0], [0.3, 0.3, 0.0]),   // coin template
    ];
    for (n, (kind, lo, hi)) in scene.iter().enumerate() {
        sup.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(2), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        sup.call("_ag_visibility", &[Int(1)], ResultKind::Unit)
            .unwrap();
        sup.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        let (tx, ty) = if n == 0 { (SPAWN_X, SPAWN_Y) } else { (-100.0, -100.0) };
        for v in [tx, ty, 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            sup.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in [1.0, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
            .iter()
            .enumerate()
        {
            sup.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..3 {
        let id = n as i32 + 1;
        sup.call("_ag_char_map", &[Int(id), Int(n)], ResultKind::Unit)
            .unwrap();
        sup.call("_ag_obj_map", &[Int(id), Int(n)], ResultKind::Unit)
            .unwrap();
        sup.call("_ag_trg_map", &[Int(id), Int(n)], ResultKind::Unit)
            .unwrap();
    }
    sup.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(status(&mut sup), 0, "supplement start fault-free");

    // 双父 Start 恰一次；币位置来自父成员引用（运行时初始化数据流）。
    assert_eq!(int(&mut sup, "get_a_starts"), 1);
    assert_eq!(int(&mut sup, "get_b_starts"), 1);
    assert_eq!(float(&mut sup, "get_coin0_position_x"), 6.0);
    assert_eq!(float(&mut sup, "get_coin1_position_x"), 12.0);

    // 玩家穿过币 0（x=6，接触 x>=5.3，t≈100，事件 t≈101 消费）：
    // 只有父 a 被投递（receiver 隔离的区分证据）；b 的币未动。
    let mut grabbed_tick = 0;
    for t in 1..=120 {
        tick(&mut sup, t);
        assert_eq!(status(&mut sup), 0, "supplement tick {t}");
        if int(&mut sup, "get_a_grabbed") == 1 {
            grabbed_tick = t;
            break;
        }
    }
    assert!(
        grabbed_tick >= 95 && grabbed_tick <= 115,
        "a grabbed at {grabbed_tick}"
    );
    assert_eq!(int(&mut sup, "get_a_grabbed"), 1, "parent a received");
    assert_eq!(int(&mut sup, "get_b_grabbed"), 0, "parent b NOT received");
    assert_eq!(
        float(&mut sup, "get_coin1_position_x"),
        12.0,
        "coin b untouched"
    );
    // 继续走：币 1（x=12）超出 120-tick 窗口，b 保持未投递。
    for t in grabbed_tick + 1..=120 {
        tick(&mut sup, t);
    }
    assert_eq!(status(&mut sup), 0);
    assert_eq!(int(&mut sup, "get_b_grabbed"), 0, "b still not received");
}

// ---- V-04/AC-05：Pause/Step 中性语义 + 20 轮 Stop/fresh 生命周期 ----

fn start_paused(h: &mut Host) {
    h.start_selected_play().expect("start");
    h.pause_play("");
    let end = Instant::now() + Duration::from_secs(30);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "start pump deadline");
        h.frame(None).unwrap();
    }
    assert_ne!(
        h.play.as_ref().unwrap().state,
        State::Faulted,
        "{:?}",
        h.play.as_ref().unwrap().error
    );
}

fn host_frame_tick(h: &mut Host) -> i32 {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .tick
}

fn pump_idle(h: &mut Host) {
    let end = Instant::now() + Duration::from_secs(20);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "pump deadline");
        h.frame(None).unwrap();
    }
}

#[test]
fn flow_host_pause_step_neutral_and_20_rounds() {
    let scene_json: serde_json::Value = serde_json::from_str(SCENE).expect("scene json");
    let sha = scene_json["behavior"]["source"]["sha256"]
        .as_str()
        .expect("sha")
        .to_string();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan016-gpu-cycle")
        .join(format!("{}", std::process::id()));
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), SCENE).expect("scene");
    std::fs::write(scripts.join(format!("{sha}.at")), WORLD).expect("script");
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .expect("load");
    let mut h = Host::with_project(doc, registry, code).expect("host");

    // Start → Paused：tick 0，中性输入。
    start_paused(&mut h);
    assert_eq!(host_frame_tick(&mut h), 0, "paused start tick 0");

    // Step 恰一次中性 tick ×3：无输入推进（玩家不移动）。
    for _ in 0..3 {
        h.play.as_mut().unwrap().single_step().unwrap();
        pump_idle(&mut h);
    }
    assert_eq!(host_frame_tick(&mut h), 3, "three neutral steps");

    // 真实 tick ×2（Resume + 输入）：tick 计数恰 +2。
    for _ in 0..2 {
        let p = h.play.as_mut().unwrap();
        p.resume().unwrap();
        p.char_key(1, true, false);
        p.dispatch(Action::default()).unwrap();
        h.pause_play("");
        pump_idle(&mut h);
    }
    assert_eq!(host_frame_tick(&mut h), 5, "two real ticks after steps");

    // 币未收集（3+2 tick 时玩家 x≈2.23，未到币）。
    assert!(
        admitted_instances(&mut h).slots[2].live(),
        "coin still live"
    );

    // 20 轮 Stop/fresh：每轮新 Run（tick 归零），Stop 后回编辑态可再启动。
    for round in 0..20 {
        h.stop_play();
        start_paused(&mut h);
        assert_eq!(host_frame_tick(&mut h), 0, "round {round} fresh tick 0");
        assert!(
            admitted_instances(&mut h).slots[2].live(),
            "round {round} coin fresh alive"
        );
        h.play.as_mut().unwrap().single_step().unwrap();
        pump_idle(&mut h);
        assert_eq!(host_frame_tick(&mut h), 1, "round {round} one step");
        h.stop_play();
    }

    // 20 轮后 fresh 仍健康。
    start_paused(&mut h);
    assert_eq!(host_frame_tick(&mut h), 0);
    assert!(admitted_instances(&mut h).slots[2].live());
    h.stop_play();
}

#[test]
fn flow_cpu_20_round_session_owner_release() {
    // AC-05：20 轮正常生命周期后 VM/堆 owner 按约释放（weak 探针），
    // 且每轮 Start-once/计数中性一致。
    for round in 0..20 {
        let mut s = session(WORLD, "node2d_flow_owner.at", 1);
        let probe = s.owner_probe();
        assert!(probe(), "round {round}: open session owns state");
        assert_eq!(int(&mut s, "get_r_starts"), 1, "round {round} starts once");
        tick(&mut s, 1);
        tick(&mut s, 2);
        assert_eq!(status(&mut s), 0, "round {round} fault-free");
        assert_eq!(int(&mut s, "get_r_ticks"), 2, "round {round} ticks");
        assert_eq!(int(&mut s, "get_h1_starts"), 1);
        drop(s);
        assert!(!probe(), "round {round}: owner released after drop");
    }
}
