//! PLAN-048 node-ui 降层门（V-S04/AC-01/AC-05；T-03/T-05）。
//! 正面：A/B 夹具（root 两同定义 Counter + Panel 嵌套 meter 子事件 +
//! bool 格式化 + 条件呈现）降层 Ok；实例表声明先序（a=0、b=1、
//! side/meter=2）；SoA/绑定表/getter/文字缓冲齐备；两次再生字节一致；
//! 旧 components-2d 档产物逐字节无漂移（dialect 改动不触旧路径）。
//! 负面：view+view-2d 混出、float 模型字段、payload 事件、未知/缺位
//! onclick、无 view 实例化、超长文字、超 4 文字槽、第 9 实例、未知元素、
//! 未知属性、缺 text —— 全部执行前具名拒绝（module/line 定位），不
//! panic。运行：cargo test --offline --test node_ui_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::{
    load_project, lower_project_components, lower_project_node_ui,
};

const MAIN: &str = include_str!("../fixtures/node-ui/main.at");
const COUNTER: &str = include_str!("../fixtures/node-ui/counter.at");
const PANEL: &str = include_str!("../fixtures/node-ui/panel.at");
const NEG_VIEW2D: &str = include_str!("../fixtures/node-ui/neg-view-2d/main.at");
const NEG_FLOAT: &str = include_str!("../fixtures/node-ui/neg-float-model/main.at");
const NEG_PAYLOAD: &str = include_str!("../fixtures/node-ui/neg-payload/main.at");
const NEG_UNKNOWN_ONCLICK: &str =
    include_str!("../fixtures/node-ui/neg-unknown-onclick/main.at");
const NEG_UNKNOWN_INST: &str = include_str!("../fixtures/node-ui/neg-unknown-inst/main.at");
const NEG_NO_VIEW_MAIN: &str = include_str!("../fixtures/node-ui/neg-no-view-def/main.at");
const NEG_NO_VIEW_LOGIC: &str = include_str!("../fixtures/node-ui/neg-no-view-def/logic.at");
const NEG_TEXT_LONG: &str = include_str!("../fixtures/node-ui/neg-text-long/main.at");
const NEG_FIVE_TEXTS: &str = include_str!("../fixtures/node-ui/neg-five-texts/main.at");
const NEG_NINE_MAIN: &str = include_str!("../fixtures/node-ui/neg-nine/main.at");
const NEG_NINE_ITEM: &str = include_str!("../fixtures/node-ui/neg-nine/item.at");
const NEG_UNKNOWN_ELEM: &str = include_str!("../fixtures/node-ui/neg-unknown-elem/main.at");
const NEG_UNKNOWN_PROP: &str = include_str!("../fixtures/node-ui/neg-unknown-prop/main.at");
const NEG_MISSING_TEXT: &str = include_str!("../fixtures/node-ui/neg-missing-text/main.at");
// 旧档对照（PLAN-020 probe 夹具 + 交付产物）。
const CMP_MAIN: &str = include_str!("../fixtures/components2d/main.at");
const CMP_PLAYER: &str = include_str!("../fixtures/components2d/player.at");
const CMP_PANEL: &str = include_str!("../fixtures/components2d/panel.at");
const CMP_GAUGE: &str = include_str!("../fixtures/components2d/gauge.at");
const CMP_RES: &str = include_str!("../fixtures/components2d/resources.at");
const CMP_FROZEN_WORLD: &str =
    include_str!("../generated-components2d/probe/world.at");

fn write(dir: &path::Path, files: &[(&str, &str)]) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower_ui(dir: &path::Path) -> Result<autographics_node2d_pilot::LowerOutputNodeUi, String> {
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_node_ui(&project).map_err(|e| e.to_string())
}

fn scratch(tag: &str) -> path::PathBuf {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("plan048-gates-{tag}-{}", std::process::id()));
    dir
}

#[test]
fn ui_ab_fixture_lowers_with_full_instance_table() {
    let dir = scratch("ab");
    write(&dir, &[("main", MAIN), ("counter", COUNTER), ("panel", PANEL)]);
    let out = lower_ui(&dir).expect("A/B fixture lowers");
    // 实例表：声明先序 + 每 def 下标（a=0、b=1、side=panel:0、meter=2）。
    assert!(out.ui_json.contains("\"path\": \"Main/a\""));
    assert!(out.ui_json.contains("\"def_idx\": 0"));
    assert!(out.ui_json.contains("\"path\": \"Main/b\""));
    assert!(out.ui_json.contains("\"path\": \"Main/side/meter\""));
    assert!(out.ui_json.contains("\"def_idx\": 2"));
    // SoA/绑定表/getter/文字缓冲。
    let src = &out.user_source;
    for needle in [
        "var counter_count=[0,10,100]",
        "var counter_step=[1,2,5]",
        "var panel_bind_meter_INC=[2]",
        "fn counter_inc(i int)",
        "fn counter_ui_click_inc(i int)",
        "fn main_ui_click_flip()",
        "fn panel_ui_click_reset(i int)",
        "fn get_Main_a_count() int",
        "fn get_Main_side_meter_count() int",
        "fn get_r_flips() int",
        "fn counter_ui_text_len(i int, e int) int",
        "fn app_start()",
        "fn app_update(dt float)",
    ] {
        assert!(src.contains(needle), "generated source missing `{needle}`");
    }
    // schema 身份。
    assert!(out.ui_json.contains("\"schema\": \"autographics.node-ui/1\""));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ui_regeneration_is_byte_identical() {
    let dir = scratch("regen");
    write(&dir, &[("main", MAIN), ("counter", COUNTER), ("panel", PANEL)]);
    let a = lower_ui(&dir).expect("first");
    let b = lower_ui(&dir).expect("second");
    assert_eq!(a.user_source, b.user_source, "world.at byte-identical");
    assert_eq!(a.ui_json, b.ui_json, "ui.json byte-identical");
    assert_eq!(a.source_sha256, b.source_sha256);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn old_components2d_products_do_not_drift() {
    // PLAN-020 probe 夹具经本次 dialect 改动后再降层，与已交付冻结产物
    // 逐字节一致（旧档行为不变的直接证据）。
    let dir = scratch("cmp");
    write(
        &dir,
        &[
            ("main", CMP_MAIN),
            ("player", CMP_PLAYER),
            ("panel", CMP_PANEL),
            ("gauge", CMP_GAUGE),
            ("resources", CMP_RES),
        ],
    );
    let project = load_project(&dir.join("main.at")).expect("load");
    let out = lower_project_components(&project)
        .map_err(|e| e.to_string())
        .expect("components lowers");
    assert_eq!(
        out.user_source.expect("user source"),
        CMP_FROZEN_WORLD,
        "components-2d probe world.at drifted"
    );
    let _ = fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------- negatives --

fn assert_ui_reject(tag: &str, files: &[(&str, &str)], needle: &str) {
    let dir = scratch(tag);
    write(&dir, files);
    let err = lower_ui(&dir).expect_err(tag);
    assert!(
        err.contains(needle),
        "{tag}: expected rejection containing `{needle}`, got: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ui_negatives_are_named_rejections() {
    // 解析面：view 与 view-2d 混出（PLAN-049 域）。
    assert_ui_reject(
        "neg-view-2d",
        &[("main", NEG_VIEW2D)],
        "declares view together with view-2d",
    );
    // 模型面：float 字段冻结拒绝。
    assert_ui_reject(
        "neg-float-model",
        &[("main", NEG_FLOAT)],
        "basic-UI v1 model is int/bool/constant-text",
    );
    // 事件面：payload 事件零参冻结。
    assert_ui_reject(
        "neg-payload",
        &[("main", NEG_PAYLOAD)],
        "zero-arg events",
    );
    // onclick 未知事件（module 定位）。
    assert_ui_reject(
        "neg-unknown-onclick",
        &[("main", NEG_UNKNOWN_ONCLICK)],
        "no handler",
    );
    // onclick 未知实例。
    assert_ui_reject(
        "neg-unknown-inst",
        &[("main", NEG_UNKNOWN_INST)],
        "unknown instance 'ghost'",
    );
    // 实例化无 view 定义（049 域）。
    assert_ui_reject(
        "neg-no-view-def",
        &[("main", NEG_NO_VIEW_MAIN), ("logic", NEG_NO_VIEW_LOGIC)],
        "declares no view",
    );
    // 超长文字。
    assert_ui_reject("neg-text-long", &[("main", NEG_TEXT_LONG)], "byte budget");
    // 超 4 文字槽（256 字面量上限）。
    assert_ui_reject(
        "neg-five-texts",
        &[("main", NEG_FIVE_TEXTS)],
        "4-element budget",
    );
    // 第 9 实例（root 计入 9）。
    assert_ui_reject(
        "neg-nine",
        &[("main", NEG_NINE_MAIN), ("item", NEG_NINE_ITEM)],
        "8-instance envelope",
    );
    // 未知元素种类。
    assert_ui_reject(
        "neg-unknown-elem",
        &[("main", NEG_UNKNOWN_ELEM)],
        "unknown view element 'grid'",
    );
    // 未知属性。
    assert_ui_reject(
        "neg-unknown-prop",
        &[("main", NEG_UNKNOWN_PROP)],
        "unknown prop 'colour' on button",
    );
    // 缺 text。
    assert_ui_reject(
        "neg-missing-text",
        &[("main", NEG_MISSING_TEXT)],
        "requires a 'text' prop",
    );
}
