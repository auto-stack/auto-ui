//! auto-ui PLAN-016 node2d_flow: the lowered minimal Node2D world (one
//! Player, two same-definition Hazards with independent state, one
//! destroyable Coin) executes on the pinned AG path with per-instance
//! Start-once semantics, an independent f32 trajectory oracle, real
//! input->sweep motion, one real trigger event -> Auto-side Collect ->
//! condition-delta real despawn, idempotent repeats, the shared 32-credit
//! pool with a 33rd-charged-call fault, and old-generation handle
//! isolation. The consumed world.at/scene.agscene are frozen generator
//! products (experimental/autographics-node2d-pilot/generated, byte-identity
//! enforced by tools/verify_plan016.py). Run single-threaded:
//! cargo test --locked --offline --test node2d_flow -- --test-threads=1
use autographics_auto_driver::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::Path;

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
