//! PLAN-105 T-02（U03 013）：**013-todo 浏览器腿（A2Vue lane）**的真实源回归。
//!
//! 两个层次：
//! 1. `lane_sfc_*`：`prepare_component_source` → `generate_sfc_with_lane`——app
//!    单元出模板与 `@click`，todo_list 单元出 `v-for`（列表渲染）。这是
//!    **不依赖 S1/MIR** 的浏览器腿证据。
//! 2. `package_lane_reports_store_gap`：包片段 lane（`generate_package_sfc`）
//!    仍在 S1（包级 store 语义）处具名拒绝——S1 落地后翻转为结构断言。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_sfc::generate_package_sfc;
use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
use auto_ui_runtime::ui::ace_vue_script::{generate_sfc_with_lane, generate_sfcs_with_lane_product};
use std::path::{Path, PathBuf};

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/013-todo")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(project_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn prepared(rel: &str) -> String {
    let root = project_root();
    prepare_component_source(&root, &read(rel))
        .unwrap_or_else(|e| panic!("prep {rel}: {e}"))
        .source
}

#[test]
fn p105_p013_lane_sfc_app_has_template_and_events() {
    std::env::set_var("P105_MEM_BACKEND", "1");
    let (sfc, _bodies, _absent) =
        generate_sfc_with_lane(&prepared("src/front/app.at"), "src/front/app.at")
            .expect("013 app 单元生成 SFC");
    assert!(sfc.contains("<template>"), "模板存在");
    assert!(sfc.contains("@click"), "事件绑定（add/filter/toggle）进模板");
}

#[test]
fn p105_p013_lane_sfc_list_has_v_for() {
    std::env::set_var("P105_MEM_BACKEND", "1");
    let (sfc, _bodies, _absent) =
        generate_sfc_with_lane(&prepared("src/front/todo_list.at"), "src/front/todo_list.at")
            .expect("013 todo_list 单元生成 SFC");
    assert!(sfc.contains("<template>"), "模板存在");
    assert!(sfc.contains("v-for"), "列表渲染进模板");
    assert!(sfc.contains("@click"), "行内事件进模板");
}

#[test]
fn p105_p013_package_lane_reports_store_gap() {
    std::env::set_var("P105_MEM_BACKEND", "1");
    let units: Vec<PackageModuleInput> = [
        ("main", "src/front/app.at"),
        ("todo_list", "src/front/todo_list.at"),
    ]
    .iter()
    .map(|(id, rel)| PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: (*id).into(),
        source: prepared(rel),
        provider: None,
    })
    .collect();
    match generate_package_sfc(&units, "app::main") {
        Ok(sfc) => {
            // S1 落地后的期望形状。
            assert!(sfc.contains("<template>"), "template block: {sfc}");
            assert!(sfc.contains("@click") || sfc.contains("v-for"), "events/list: {sfc}");
        }
        Err(e) => assert!(
            e.contains("package-level store semantics are not wired yet (S1)")
                || e.contains("source.syntax")
                // W104：S1 首片（包面 store 内联）落地后，墙前移到 store 状态
                // 函数的符号面——`__age_state_0_st_*` 是 S1 家族的具名拒绝。
                || e.contains("__age_state_0_st_"),
            "包片段 lane 当前应停在 S1/语法面（已知缺口），实际: {e}"
        ),
    }
}

/// 013 浏览器腿的**单页装配**：app + todo_list 合源走多 widget lane，产出
/// 根 SFC（App）与子组件 SFC（TodoList），并带子组件 import/引用。
#[test]
fn p105_p013_multi_widget_page_assembles() {
    std::env::set_var("P105_MEM_BACKEND", "1");
    let merged = format!(
        "{}\n{}\n",
        prepared("src/front/app.at"),
        prepared("src/front/todo_list.at")
    );
    let multi = generate_sfcs_with_lane_product(&merged, "src/front/app.at", "./a2x_rt.mjs", "./a2x_rt.mjs")
        .expect("013 双 widget 源生成 SFC 产品");
    assert_eq!(multi.widgets.len(), 2, "App + TodoList 两件产物");
    let app = multi.widgets.iter().find(|w| w.name == "App").expect("App 产物");
    let list = multi.widgets.iter().find(|w| w.name == "TodoList").expect("TodoList 产物");
    assert!(app.sfc.contains("import TodoList from"), "根 SFC 注册子组件: {}", app.sfc.lines().take(6).collect::<Vec<_>>().join(" | "));
    assert!(app.sfc.contains("<TodoList"), "根 SFC 渲染子组件");
    assert!(app.sfc.contains("@click"), "根 SFC 承载事件");
    assert!(list.sfc.contains("v-for"), "子组件 SFC 承载列表渲染");
    assert!(list.sfc.contains("@click"), "子组件 SFC 承载行内事件");
}