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

use autographics_node2d_pilot::{load_project, lower_project_components, lower_project_node_ui};

const MAIN: &str = include_str!("../fixtures/node-ui/main.at");
const COUNTER: &str = include_str!("../fixtures/node-ui/counter.at");
const PANEL: &str = include_str!("../fixtures/node-ui/panel.at");
const NEG_VIEW2D: &str = include_str!("../fixtures/node-ui/neg-view-2d/main.at");
const NEG_FLOAT: &str = include_str!("../fixtures/node-ui/neg-float-model/main.at");
const NEG_PAYLOAD: &str = include_str!("../fixtures/node-ui/neg-payload/main.at");
const NEG_UNKNOWN_ONCLICK: &str = include_str!("../fixtures/node-ui/neg-unknown-onclick/main.at");
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
const CMP_FROZEN_WORLD: &str = include_str!("../generated-components2d/probe/world.at");

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
    write(
        &dir,
        &[("main", MAIN), ("counter", COUNTER), ("panel", PANEL)],
    );
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
    assert!(out
        .ui_json
        .contains("\"schema\": \"autographics.node-ui/1\""));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ui_regeneration_is_byte_identical() {
    let dir = scratch("regen");
    write(
        &dir,
        &[("main", MAIN), ("counter", COUNTER), ("panel", PANEL)],
    );
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
    assert_ui_reject("neg-payload", &[("main", NEG_PAYLOAD)], "zero-arg events");
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

// ---------------------------------------------------------- PLAN-065 --
// 少文字槽（≤2 槽）def 半缓冲剪裁生成（D-06 关闭面）：1/2 槽 def 的
// digits/getter 高槽分支剪裁——不再引用未声明 `_ui_text_b_{k}`（AG021
// 具名反例：固定 SDK `Undefined variable: {p}_ui_text_b_0`，V-01 先红
// 后绿复现在案）；≥3 槽 def 生成段逐字不变（V-04 冻结面）；0 槽合法
// 应用保持；文字预算 63 字节合法 / 64 字节具名拒绝（字节计数）。

const FEW_A_MAIN: &str = include_str!("../fixtures/node-ui-few/a/main.at");
const FEW_A_FEW0: &str = include_str!("../fixtures/node-ui-few/a/few0.at");
const FEW_A_FEW1: &str = include_str!("../fixtures/node-ui-few/a/few1.at");
const FEW_A_FEW2: &str = include_str!("../fixtures/node-ui-few/a/few2.at");
const FEW_B_MAIN: &str = include_str!("../fixtures/node-ui-few/b/main.at");
const FEW_B_FEW3: &str = include_str!("../fixtures/node-ui-few/b/few3.at");
const FEW_B_FEW4: &str = include_str!("../fixtures/node-ui-few/b/few4.at");
const FEW_EMPTY_MAIN: &str = include_str!("../fixtures/node-ui-few/empty/main.at");

const TEXT_63_ASCII: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
const TEXT_64_ASCII: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
// 21 个汉字 × 3 字节 = 63 字节（合法上界）；再拼 1 个 ASCII = 64（拒绝）。
const TEXT_64_CJK: &str = "二十一字二十一字二十一字二十一字二十一字甲A";

fn zeros(n: usize) -> String {
    vec!["0"; n].join(",")
}

fn lower_few(tag: &str, files: &[(&str, &str)]) -> autographics_node2d_pilot::LowerOutputNodeUi {
    let dir = scratch(tag);
    write(&dir, files);
    let out = lower_ui(&dir).unwrap_or_else(|e| panic!("{tag}: {e}"));
    let _ = fs::remove_dir_all(&dir);
    out
}

#[test]
fn ui_few_slot_defs_prune_undeclared_half_buffer() {
    // 家族 A：root 2 槽 + Few1 1 槽（tick 驱动）+ Few2 2 槽（int+按钮）
    // + Few0 0 槽。
    let a = lower_few(
        "few-a",
        &[
            ("main", FEW_A_MAIN),
            ("few0", FEW_A_FEW0),
            ("few1", FEW_A_FEW1),
            ("few2", FEW_A_FEW2),
        ],
    );
    let src = &a.user_source;
    // 1 槽 def：仅缓冲 A（64 字节）＋len；无 b 引用；getter 单分支。
    assert!(
        src.contains(&format!("var few1_ui_text_0=[{}]", zeros(64))),
        "few1 64-byte buffer A"
    );
    assert!(
        src.contains("var few1_ui_text_len_0=[0]"),
        "few1 len array is single-slot"
    );
    assert!(
        !src.contains("few1_ui_text_b"),
        "few1 must not reference text_b"
    );
    assert!(
        src.contains("return few1_ui_text_0[((e*64)+j)]"),
        "few1 getter single-branch"
    );
    assert!(
        !src.contains("if e<2 {return few1_ui_text_0"),
        "few1 getter pruned"
    );
    // 2 槽 def：缓冲 A 128 字节；digits 单分支（无 buf 分派、mag 无 buf 参）。
    assert!(
        src.contains(&format!("var few2_ui_text_0=[{}]", zeros(128))),
        "few2 128-byte buffer A (2 slots, one half)"
    );
    assert!(
        !src.contains("few2_ui_text_b"),
        "few2 must not reference text_b"
    );
    let di = src.find("fn few2_ui_digits_0").expect("few2 digits fn");
    let seg = &src[di..di + 200];
    assert!(
        seg.contains(" let at=(e*64)"),
        "few2 digits pruned at: {seg}"
    );
    assert!(
        !seg.contains(" let buf="),
        "few2 digits has no buf dispatch"
    );
    assert!(
        src.contains("fn few2_ui_digits_mag_0(at int, n int) int {"),
        "few2 mag fn without buf param"
    );
    // root（2 槽）：同样剪裁。
    assert!(
        src.contains("return main_ui_text_0[((e*64)+j)]"),
        "root getter single-branch"
    );
    assert!(
        !src.contains("main_ui_text_b"),
        "root (2 texts) must not reference text_b"
    );
    // 0 槽 def：无任何文字段/刷新段。
    assert!(
        !src.contains("few0_ui_text"),
        "Few0 (0 texts) has no text segments"
    );
    assert!(
        !src.contains("few0_ui_refresh"),
        "Few0 has no refresh dispatcher"
    );

    // 家族 B：root 4 槽 + Few3 3 槽（控制形状：双分支冻结）+ Few4 4 槽
    // （含 63 字节多字节字面量）。
    let b = lower_few(
        "few-b",
        &[
            ("main", FEW_B_MAIN),
            ("few3", FEW_B_FEW3),
            ("few4", FEW_B_FEW4),
        ],
    );
    let src = &b.user_source;
    // 3 槽控制：双半缓冲都声明，getter/digits 保持冻结双分支形状。
    assert!(
        src.contains(&format!("var few3_ui_text_0=[{}]", zeros(128))),
        "few3 buffer A (2 slots)"
    );
    assert!(
        src.contains(&format!("var few3_ui_text_b_0=[{}]", zeros(64))),
        "few3 buffer B (1 slot) declared at hi>0"
    );
    assert!(
        src.contains(
            "if e<2 {return few3_ui_text_0[((e*64)+j)]}else{return few3_ui_text_b_0[(((e-2)*64)+j)]}"
        ),
        "few3 getter keeps the frozen dual-branch shape"
    );
    assert!(
        src.contains(" let buf=if e<2{1}else{0}"),
        "few3 digits keeps buf dispatch"
    );
    // 4 槽 def：128+128；63 字节内容按字节写入 len。
    assert!(
        src.contains(&format!("var few4_ui_text_b_0=[{}]", zeros(128))),
        "few4 buffer B (2 slots)"
    );
    assert!(
        src.contains("few4_ui_text_len_0[0]=63"),
        "few4 63-byte multi-byte literal writes len 63 (byte-counted)"
    );
    // root 4 槽：128+128 双半缓冲。
    assert!(
        src.contains(&format!("var main_ui_text_b_0=[{}]", zeros(128))),
        "root (4 texts) declares buffer B"
    );
}

#[test]
fn ui_few_slot_regeneration_is_byte_identical() {
    let files_a = &[
        ("main", FEW_A_MAIN),
        ("few0", FEW_A_FEW0),
        ("few1", FEW_A_FEW1),
        ("few2", FEW_A_FEW2),
    ];
    let a1 = lower_few("few-regen-a1", files_a);
    let a2 = lower_few("few-regen-a2", files_a);
    assert_eq!(
        a1.user_source, a2.user_source,
        "family a world byte-identical"
    );
    assert_eq!(a1.ui_json, a2.ui_json, "family a ui.json byte-identical");
    let files_b = &[
        ("main", FEW_B_MAIN),
        ("few3", FEW_B_FEW3),
        ("few4", FEW_B_FEW4),
    ];
    let b1 = lower_few("few-regen-b1", files_b);
    let b2 = lower_few("few-regen-b2", files_b);
    assert_eq!(
        b1.user_source, b2.user_source,
        "family b world byte-identical"
    );
    assert_eq!(b1.ui_json, b2.ui_json, "family b ui.json byte-identical");
}

#[test]
fn ui_zero_text_app_is_legal_and_minimal() {
    // 0 槽应用合法（整应用无文字仍成立；不是靠删控件或注水占位槽换绿）。
    let out = lower_few(
        "few-empty",
        &[("main", FEW_EMPTY_MAIN), ("few0", FEW_A_FEW0)],
    );
    let src = &out.user_source;
    assert!(
        src.contains("fn app_start()"),
        "empty app still has app_start"
    );
    assert!(
        src.contains("fn app_update(dt float)"),
        "empty app still has app_update"
    );
    assert!(!src.contains("_ui_text"), "no text buffers anywhere");
    assert!(!src.contains("_ui_refresh"), "no refresh dispatchers");
    assert!(!src.contains("_ui_click"), "no click dispatchers");
}

#[test]
fn ui_text_budget_63_byte_legal_64_named_rejected() {
    // 63 字节 ASCII 字面量合法，len 按字节写入 63。
    let main63 = format!(
        "node Main {{\n    view {{\n        text t {{ text: \"{TEXT_63_ASCII}\" }}\n    }}\n}}\n"
    );
    let out = lower_few("few-63-ascii", &[("main", main63.as_str())]);
    if !out.user_source.contains("main_ui_text_len_0[0]=63") {
        panic!(
            "63-byte len write missing; source:
{}",
            out.user_source
        );
    }
    // 64 字节 ASCII 字面量具名拒绝。
    let main64 = format!(
        "node Main {{\n    view {{\n        text t {{ text: \"{TEXT_64_ASCII}\" }}\n    }}\n}}\n"
    );
    let dir = scratch("few-64-ascii");
    write(&dir, &[("main", main64.as_str())]);
    let err = lower_ui(&dir).expect_err("64-byte literal");
    assert!(
        err.contains("63-byte budget") && err.contains("(64 bytes)"),
        "64-byte ASCII named rejection, got: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
    // 64 字节多字节（63 字节汉字 + 1 ASCII）同样拒绝（字节而非字符计数）。
    let main64c = format!(
        "node Main {{\n    view {{\n        text t {{ text: \"{TEXT_64_CJK}\" }}\n    }}\n}}\n"
    );
    let dir = scratch("few-64-cjk");
    write(&dir, &[("main", main64c.as_str())]);
    let err = lower_ui(&dir).expect_err("64-byte CJK literal");
    assert!(
        err.contains("63-byte budget") && err.contains("(64 bytes)"),
        "64-byte CJK named rejection, got: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
    // 常量文字字段：63 字节合法、64 字节拒绝。
    let field63 = format!(
        "node Main {{\n    model {{\n        var label = \"{TEXT_63_ASCII}\"\n    }}\n    view {{\n        text t {{ text: .label }}\n    }}\n}}\n"
    );
    let out = lower_few("few-63-field", &[("main", field63.as_str())]);
    assert!(
        out.user_source.contains("main_ui_text_len_0[0]=63"),
        "63-byte constant field inlines legally"
    );
    let field64 = format!(
        "node Main {{\n    model {{\n        var label = \"{TEXT_64_ASCII}\"\n    }}\n    view {{\n        text t {{ text: .label }}\n    }}\n}}\n"
    );
    let dir = scratch("few-64-field");
    write(&dir, &[("main", field64.as_str())]);
    let err = lower_ui(&dir).expect_err("64-byte constant field");
    assert!(
        err.contains("text field 'label'") && err.contains("63-byte budget"),
        "64-byte constant field named rejection, got: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
    // 组件侧第 5 槽同样具名拒绝（root 侧由 neg-five-texts 覆盖）。
    let comp5 = "node Five {\n    view {\n        column box {\n            text a { text: \"1\" }\n            text b { text: \"2\" }\n            text c { text: \"3\" }\n            text d { text: \"4\" }\n            text e { text: \"5\" }\n        }\n    }\n}\n";
    let main5 = "use five: Five\n\nnode Main {\n    view {\n        column root {\n            Five x { }\n        }\n    }\n}\n";
    let dir = scratch("few-comp-five");
    write(&dir, &[("main", main5), ("five", comp5)]);
    let err = lower_ui(&dir).expect_err("5-text component");
    assert!(
        err.contains("4-element budget"),
        "component-side 5th slot named rejection, got: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ui_no_instance_def_generates_no_text_segments() {
    // 无实例 def（有文字但 0 实例）不生成任何文字段——「无实例 def 行为」保持。
    let main = "use few1: Few1\n\nnode Main {\n    view {\n        column root {\n            text t { text: \"solo\" }\n        }\n    }\n}\n";
    // 有文字但 0 实例的 def 在 plan 期即具名拒绝（唯一 root 规则把未实例
    // 化 def 当 root 候选）——「无实例 def」因此到不了生成面；n==0 代码
    // 路径（getters/refresh/digits 早退）保持不变。该具名拒绝即「无实例
    // def 行为」的现有合同面。
    let dir = scratch("few-no-inst");
    write(&dir, &[("main", main), ("few1", FEW_A_FEW1)]);
    let err = lower_ui(&dir).expect_err("unused import is named-rejected");
    assert!(
        err.contains("exactly one root node expected"),
        "unused-import named rejection, got: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
}
