//! PLAN-020 components-2d 降层门（V-01/AC-02；T-03/T-04）。
//! 正面：T-02 探针链（Player 局部走跳 + 双同定义 Gauge + Panel→meter 事件 +
//! 根 vy 哨兵）以作者源降层 Ok；每定义 SoA / 按实例索引 handler / 绑定表
//! [0,1] / getters 齐备；嵌套同定义子实例路径/下标正确（a/meter=0、
//! b/meter=1）；两模块同名 helper 池化为模块前缀 fn 且按模块作用域解析；
//! 两次再生字节一致。负面：缺 use / 错元数 / 未知端口 / 实例环 / 第 9
//! 有状态实例 / 缺相机 —— 全部执行前具名拒绝（module/line/symbol），不
//! panic。运行：cargo test --offline --test node_components_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::{load_project, lower_project_components, LowerOutputBasic2D};

const MAIN: &str = include_str!("../fixtures/components2d/main.at");
const PLAYER: &str = include_str!("../fixtures/components2d/player.at");
const PANEL: &str = include_str!("../fixtures/components2d/panel.at");
const GAUGE: &str = include_str!("../fixtures/components2d/gauge.at");
const RES: &str = include_str!("../fixtures/components2d/resources.at");
const NEG_MISSING_USE: &str = include_str!("../fixtures/components2d/neg-missing-use/main.at");
const NEG_WRONG_ARITY: &str = include_str!("../fixtures/components2d/neg-wrong-arity/panel.at");
const NEG_UNKNOWN_PORT: &str = include_str!("../fixtures/components2d/neg-unknown-port/panel.at");
const NEG_CYCLE_MAIN: &str = include_str!("../fixtures/components2d/neg-cycle/main.at");
const NEG_CYCLE_BOXA: &str = include_str!("../fixtures/components2d/neg-cycle/boxa.at");
const NEG_CYCLE_BOXB: &str = include_str!("../fixtures/components2d/neg-cycle/boxb.at");
const NEG9_MAIN: &str = include_str!("../fixtures/components2d/neg-9-stateful/main.at");
const NEG9_ITEM: &str = include_str!("../fixtures/components2d/neg-9-stateful/item.at");
const NEG_NO_CAMERA: &str = include_str!("../fixtures/components2d/neg-no-camera/main.at");
const BUDGET_MAIN: &str = include_str!("../fixtures/components2d/budget/main.at");
const BUDGET_PLUS_MAIN: &str = include_str!("../fixtures/components2d/budget-plus/main.at");
const BUDGET_GAUGE: &str = include_str!("../fixtures/components2d/budget/gauge.at");
const ENVELOPE_MAIN: &str = include_str!("../fixtures/components2d/envelope/main.at");
const ENVELOPE_TICKER: &str = include_str!("../fixtures/components2d/envelope/ticker.at");
const INIT_FAULT_MAIN: &str = include_str!("../fixtures/components2d/init-fault/main.at");
const INIT_FAULT_CLUSTER: &str = include_str!("../fixtures/components2d/init-fault/cluster.at");
const NEG_PARAM_WRITE: &str = include_str!("../fixtures/components2d/neg-param-write/gauge.at");
const HELP_MAIN: &str = include_str!("../fixtures/components2d/helpers/main.at");
const HELP_A: &str = include_str!("../fixtures/components2d/helpers/helpera.at");
const HELP_B: &str = include_str!("../fixtures/components2d/helpers/helperb.at");

fn write(dir: &path::Path, files: &[(&str, &str)]) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower(dir: &path::Path) -> Result<LowerOutputBasic2D, String> {
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_components(&project).map_err(|e| e.to_string())
}

fn components(files: &[(&str, &str)]) -> Result<LowerOutputBasic2D, String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir =
        path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("lc-{}-{n}", std::process::id()));
    write(&dir, files);
    lower(&dir)
}

fn probe_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("main", MAIN),
        ("player", PLAYER),
        ("panel", PANEL),
        ("gauge", GAUGE),
        ("resources", RES),
    ]
}

// ------------------------------------------------- 正面：探针链形状/再生 --

#[test]
fn probe_chain_lowers_ok_with_frozen_shape_and_is_byte_stable() {
    let a = components(&probe_files()).expect("probe chain lowers");
    let b = components(&probe_files()).expect("probe chain lowers again");
    let wa = a.user_source.as_ref().expect("world.at");
    let wb = b.user_source.as_ref().expect("world.at");
    assert_eq!(wa, wb, "world regeneration byte-identical");
    assert_eq!(
        a.scene_json, b.scene_json,
        "scene regeneration byte-identical"
    );

    // 每定义 SoA（model 初值按实例内联 + 运行期参数数组）。
    for line in [
        "var r_vy=999",
        "var panel_acc=[0,0]",
        "var panel_period=[10,15]",
        "var panel_payload=[1,1]",
        "var gauge_phase=[0.0,5.0]",
        "var gauge_ticks=[0,0]",
        "var gauge_count=[0,0]",
        "var gauge_speed=[1.0,6.0]",
        "var gauge_base_x=[4.0,-4.0]",
        "var gauge_base_y=[3.0,3.0]",
        "var gauge_hide_at=[61,-1]",
        "var player_vy=[0.0]",
        "var player_grounded=[0]",
    ] {
        assert!(wa.contains(line), "SoA line missing: {line}");
    }
    // 按实例索引 handler（<inst>_lifetick(i,dt)）+ 接收 fn + present。
    for line in [
        "fn main_lifestart() {",
        "fn main_lifetick(dt float) {",
        "fn panel_lifetick(i int, dt float) {",
        "fn gauge_lifetick(i int, dt float) {",
        "fn player_lifetick(i int, dt float) {",
        "fn meter_add(j int, n int) {",
        "fn gauge_present(i int) {",
    ] {
        assert!(wa.contains(line), "handler fn missing: {line}");
    }
    // 绑定表（meter 表 [0,1]）与投递调用（同步直呼）。
    assert!(wa.contains("var meter_bind_ADD=[0,1]"), "binding table");
    assert!(
        wa.contains("meter_add(meter_bind_ADD[i], panel_payload[i])"),
        "synchronous child dispatch call"
    );
    // getters（root r_ + 每实例下标与路径别名）。
    for line in [
        "fn get_r_vy() int {return r_vy}",
        "fn get_gauge_count_0() int {return gauge_count[0]}",
        "fn get_gauge_count_1() int {return gauge_count[1]}",
        "fn get_a_meter_count() int {return gauge_count[0]}",
        "fn get_b_meter_count() int {return gauge_count[1]}",
        "fn get_player_vy_0() float {return player_vy[0]}",
    ] {
        assert!(wa.contains(line), "getter missing: {line}");
    }
    // app_update 冻结序：根 → 声明先序 tick → present。
    let upd_pos = |needle: &str| wa.find(needle).expect(needle);
    let o = [
        upd_pos("fn app_update"),
        upd_pos(" main_lifetick(dt)"),
        upd_pos(" panel_lifetick(0,dt)"),
        upd_pos(" gauge_lifetick(0,dt)"),
        upd_pos(" gauge_present(0)"),
        upd_pos(" panel_lifetick(1,dt)"),
        upd_pos(" gauge_lifetick(1,dt)"),
        upd_pos(" gauge_present(1)"),
        upd_pos(" player_lifetick(0,dt)"),
    ];
    assert!(o.windows(2).all(|w| w[0] < w[1]), "app_update order: {o:?}");
    // app_start：角色/solid/相机 + spawn（模板 id 3/4、槽 0/1）。
    assert!(wa.contains(" ok=ok+ag_char_bind(1)*1.0"), "char bind");
    assert!(wa.contains(" ok=ok+ag_char_solid(2)*1.0"), "char solid");
    assert!(
        wa.contains(" ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 40.0, 12.0, 40.0, 12.0)*1.0"),
        "camera follow"
    );
    assert!(
        wa.contains("gauge_handle[0]=ag_object_spawn(0,3,gauge_draw_x[0],gauge_base_y[0],0.0)"),
        "gauge a spawn"
    );
    assert!(
        wa.contains("gauge_handle[1]=ag_object_spawn(1,4,gauge_draw_x[1],gauge_base_y[1],0.0)"),
        "gauge b spawn"
    );
    // 场景：角色行 + 静态 solid + 每实例动态模板（停靠隐藏，保身份）。
    assert!(
        a.scene_json.contains("\"name\": \"player.body\""),
        "char row"
    );
    assert!(a.scene_json.contains("\"name\": \"ground\""), "solid row");
    let tmpl_a = segment_after(&a.scene_json, "\"name\": \"a_meter_template\"");
    let tmpl_b = segment_after(&a.scene_json, "\"name\": \"b_meter_template\"");
    assert!(tmpl_a.contains("\"kind\": \"circle\"") && tmpl_a.contains("\"visible\": false"));
    assert!(tmpl_b.contains("\"kind\": \"quad\"") && tmpl_b.contains("\"visible\": false"));
    assert!(a
        .scene_json
        .contains("\"api\": \"autographics.objects2d/1\""));
}

#[test]
fn nested_same_def_children_carry_correct_paths_and_indexes() {
    let out = components(&probe_files()).expect("probe chain");
    let w = out.user_source.as_ref().expect("world.at");
    // 定义 SoA 注释按完整实例路径与定义下标点名。
    assert!(
        w.contains("// Panel SoA (instances a=0, b=1)"),
        "panel paths"
    );
    assert!(
        w.contains("// Gauge SoA (instances a/meter=0, b/meter=1)"),
        "gauge nested paths"
    );
    // 嵌套同定义子实例：a/meter 是 gauge[0]、b/meter 是 gauge[1] —— 由
    // 绑定表（发射实例 i=0/1 → 各自 meter 下标 0/1）与别名 getters 共证。
    assert!(w.contains("var meter_bind_ADD=[0,1]"));
    assert!(w.contains("fn get_a_meter_ticks() int {return gauge_ticks[0]}"));
    assert!(w.contains("fn get_b_meter_ticks() int {return gauge_ticks[1]}"));
    // 模板停靠位置按定义实例下标错开（-40-2*idx）。
    let ta = segment_after(&out.scene_json, "\"name\": \"a_meter_template\"");
    let tb = segment_after(&out.scene_json, "\"name\": \"b_meter_template\"");
    assert!(ta.contains("[-30.0, -40.0, 0.0]"), "park a: {ta}");
    assert!(tb.contains("[-30.0, -42.0, 0.0]"), "park b: {tb}");
    // 根标量独立状态（019 约定 r_<field>；与组件字段不合并）。
    assert!(w.contains("var r_vy=999") && w.contains("fn get_r_vy() int {return r_vy}"));
    assert!(
        !w.contains("gauge_vy") && !w.contains("panel_vy"),
        "no cross-def flatten"
    );
}

// ------------------------------------------------- 正面：同名 helper 作用域 --

#[test]
fn same_name_helpers_across_modules_pool_prefixed_and_resolve_per_module() {
    let out = components(&[
        ("main", HELP_MAIN),
        ("helpera", HELP_A),
        ("helperb", HELP_B),
        ("resources", RES),
    ])
    .expect("helpers fixture lowers");
    let w = out.user_source.as_ref().expect("world.at");
    // 池化 fn 带模块前缀且两体都在（同名不同体）。
    assert!(
        w.contains("fn helpera_bump(v float) float {"),
        "module A fn"
    );
    assert!(w.contains("return v + 1.0"), "module A body");
    assert!(
        w.contains("fn helperb_bump(v float) float {"),
        "module B fn"
    );
    assert!(w.contains("return v + 2.0"), "module B body");
    // 调用点按定义所在模块解析：Alpha→helpera_bump、Beta→helperb_bump。
    assert!(w.contains("alpha_acc[i] = helpera_bump((alpha_acc[i]"));
    assert!(w.contains("beta_acc[i] = helperb_bump((beta_acc[i]"));
    // 同名字段各定义独立 SoA（alpha_acc / beta_acc 不共享）。
    assert!(w.contains("var alpha_acc=[0.0]") && w.contains("var beta_acc=[0.0]"));
}

// --------------------------------------------------------- 负面：具名拒绝 --

#[test]
fn negative_fixtures_fail_with_named_diagnostics_and_no_panic() {
    // 缺 use：实例化的 node 不在模块闭包内（加载期定位拒绝）。
    let err = components(&[
        ("main", NEG_MISSING_USE),
        ("player", PLAYER),
        ("resources", RES),
    ])
    .err()
    .expect("missing use must fail");
    assert!(err.contains("Panel") || err.contains("panel"), "{err}");

    let base = |panel: &'static str| {
        vec![
            ("main", MAIN),
            ("player", PLAYER),
            ("panel", panel),
            ("gauge", GAUGE),
            ("resources", RES),
        ]
    };
    // 错元数：payload/handler 1 参，emit 0 参。
    let err = components(&base(NEG_WRONG_ARITY))
        .err()
        .expect("wrong arity must fail");
    assert!(err.contains("arity") || err.contains("arg"), "{err}");
    assert!(
        err.contains("module panel") || err.contains("module gauge"),
        "{err}"
    );
    assert!(err.contains("line ") && err.contains("'Add'"), "{err}");

    // 未知端口：Gauge 只声明 Add。
    let err = components(&base(NEG_UNKNOWN_PORT))
        .err()
        .expect("unknown port must fail");
    assert!(err.contains("unknown event port"), "{err}");
    assert!(err.contains("'Bogus'"), "{err}");
    assert!(err.contains("line "), "{err}");

    // 实例环：BoxA -> BoxB -> BoxA（按定义路径集具名拒绝）。
    let err = components(&[
        ("main", NEG_CYCLE_MAIN),
        ("boxa", NEG_CYCLE_BOXA),
        ("boxb", NEG_CYCLE_BOXB),
    ])
    .err()
    .expect("cycle must fail");
    assert!(
        err.contains("instance cycle: BoxA -> BoxB -> BoxA"),
        "{err}"
    );
    assert!(
        err.contains("module boxb") && err.contains("line ") && err.contains("'inner'"),
        "{err}"
    );

    // 第 9 个有状态实例（含根）：8 包络具名拒绝。
    let err = components(&[("main", NEG9_MAIN), ("item", NEG9_ITEM), ("resources", RES)])
        .err()
        .expect("9 stateful instances must fail");
    assert!(err.contains("8-instance envelope"), "{err}");
    assert!(err.contains("'i8'") && err.contains("line "), "{err}");

    // 缺相机：components 档要求恰一 camera2d（Objects2V1 入口冻结相机态）。
    let err = components(&[
        ("main", NEG_NO_CAMERA),
        ("player", PLAYER),
        ("panel", PANEL),
        ("gauge", GAUGE),
        ("resources", RES),
    ])
    .err()
    .expect("missing camera must fail");
    assert!(err.contains("camera2d"), "{err}");
    assert!(err.contains("requires"), "{err}");

    // 参数只读：`.speed` 是声明参数，作为赋值目标具名拒绝；model 字段赋值
    // 不受影响（全部正面链的 .ticks/.acc/.phase 写入即证）。
    let err = components(&[
        ("main", BUDGET_MAIN),
        ("gauge", NEG_PARAM_WRITE),
        ("player", PLAYER),
        ("resources", RES),
    ])
    .err()
    .expect("param write must fail");
    assert!(err.contains("read-only"), "{err}");
    assert!(err.contains("'speed'"), "{err}");
    assert!(
        err.contains("module gauge") && err.contains("line "),
        "{err}"
    );
}

// --------------------------------------- T-05：预算/包络/初始化故障降层面 --

#[test]
fn t05_budget_envelope_and_init_fault_lowering_shape() {
    // budget: 4 staggered gauges -> exactly two spawn slots moving per tick
    // side; the charged writes come only from the parity-gated presents.
    let out = components(&[
        ("main", BUDGET_MAIN),
        ("gauge", BUDGET_GAUGE),
        ("player", PLAYER),
        ("resources", RES),
    ])
    .expect("budget lowers");
    let w = out.user_source.as_ref().expect("world.at");
    assert!(w.contains("var gauge_parity=[0,1,0,1]"), "staggered sides");
    for slot in 0..4 {
        assert!(
            w.contains(&format!(
                "gauge_handle[{slot}]=ag_object_spawn({slot},{}",
                slot + 3
            )),
            "spawn slot {slot}"
        );
    }
    assert_eq!(
        w.matches("ag_object_move(gauge_handle[").count(),
        1,
        "one move site"
    );
    // budget-plus: the ONLY delta is the fifth gauge (fifth spawn, 8 env).
    let plus = components(&[
        ("main", BUDGET_PLUS_MAIN),
        ("gauge", BUDGET_GAUGE),
        ("player", PLAYER),
        ("resources", RES),
    ])
    .expect("budget-plus lowers");
    let wp = plus.user_source.as_ref().expect("world.at");
    assert!(wp.contains("gauge_handle[4]=ag_object_spawn(4,7,"));
    assert!(
        wp.contains("var gauge_parity=[0,1,0,1,1]"),
        "3 even-side writes"
    );

    // envelope: root + player + 6 tickers = exactly 8 stateful instances.
    let env = components(&[
        ("main", ENVELOPE_MAIN),
        ("ticker", ENVELOPE_TICKER),
        ("player", PLAYER),
        ("resources", RES),
    ])
    .expect("envelope lowers");
    let we = env.user_source.as_ref().expect("world.at");
    assert!(we.contains("// Ticker SoA (instances t1=0, t2=1, t3=2, t4=3, t5=4, t6=5)"));
    assert!(!we.contains("ag_object_spawn"), "tickers are service-free");
    assert!(we.contains("// Player SoA (instance player=0)"));
    // stateful count: root r_vy + player + 6 tickers = 8 (no 9th anywhere);
    // lifetick FNS are per definition (SoA), the SoA comments carry the
    // per-instance indexes.
    assert_eq!(
        we.matches("_lifetick(i int, dt float)").count(),
        2,
        "ticker + player lifetick defs"
    );
    assert!(we.contains("fn main_lifetick(dt float)"), "root lifetick");

    // init-fault: NINE dynamic shapes lower GREEN (the failure is the
    // runtime slot contract, not a lowering rejection): 9 spawn sites with
    // slot indices 0..=8, scene rows still <= 128.
    let fault = components(&[
        ("main", INIT_FAULT_MAIN),
        ("cluster", INIT_FAULT_CLUSTER),
        ("player", PLAYER),
        ("resources", RES),
    ])
    .expect("init-fault must LOWER (fault is runtime)");
    let wf = fault.user_source.as_ref().expect("world.at");
    assert!(wf.contains("ag_object_spawn(8,"), "ninth spawn present");
    assert_eq!(wf.matches("ag_object_spawn(").count(), 9);
    assert!(fault.scene_json.contains("\"name\": \"c_d8_template\""));
}

// ---------------------------------------------------------------- 工具 --

/// 取 name 标记之后到本对象行结束前的片段。
fn segment_after<'a>(scene: &'a str, marker: &str) -> &'a str {
    let pos = scene.find(marker).expect(marker);
    let rest = &scene[pos..];
    let end = rest.find("\n  }").unwrap_or(rest.len());
    &rest[..end]
}
