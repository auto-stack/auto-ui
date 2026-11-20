//! PLAN-105（U03）：真语料 app 的 **SFC 生成现状**（浏览器腿）——按 app 钉住当前
//! 首个阻断。S1（包级 `store` 语义）落地后这些断言会翻红，届时改为结构断言。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_sfc::generate_package_sfc;
use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;
use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui")
}

fn gen(app: &str, units: &[(&str, &str)], entry: &str) -> Result<String, String> {
    let base = root().join(app);
    let mut inputs = Vec::new();
    for (id, rel) in units {
        let src = std::fs::read_to_string(base.join(rel)).unwrap_or_default();
        let prepared = prepare_component_source(&base, &src).map(|p| p.source).unwrap_or(src);
        inputs.push(PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: (*id).into(),
            source: prepared,
            provider: None,
        });
    }
    generate_package_sfc(&inputs, entry)
}

fn expect_gap(app: &str, units: &[(&str, &str)], needle: &str) {
    match gen(app, units, "app::main") {
        Ok(sfc) => panic!(
            "{app} 现在已能生成 SFC（len={}）——把本用例改成结构断言",
            sfc.len()
        ),
        Err(e) => assert!(
            needle.split("|").any(|n| e.contains(n)),
            "{app} 的首个阻断应属于 `{needle}`，实际: {e}"
        ),
    }
}

#[test]
fn p105_u03_apps_sfc_status() {
    std::env::set_var("P105_MEM_BACKEND", "1");
    // 013：包级 store（S1）。
    expect_gap(
        "013-todo",
        &[
            ("main", "src/front/app.at"),
            ("todo_store", "src/front/todo_store.at"),
            ("todo_list", "src/front/todo_list.at"),
        ],
        "store semantics are not wired yet (S1)|cross-unit store loading is not wired yet (S1)|source.syntax|model has no field `store`",
    );
    // 016：同一条 S1 闸（CalendarStore）。
    expect_gap(
        "016-calendar",
        &[
            ("main", "src/front/app.at"),
            ("calendar_store", "src/front/calendar_store.at"),
            ("calendar_util", "src/front/calendar_util.at"),
        ],
        "store semantics are not wired yet (S1)|cross-unit store loading is not wired yet (S1)|source.syntax|model has no field `store`",
    );
    // 015：`.store` 引用在**同源组件面**缺 store 声明（prep 未内联）。
    expect_gap(
        "015-notes",
        &[("main", "src/front/app.at")],
        "model has no field `store`|source.syntax",
    );
}

/// A2Vue lane（单 unit）对 U03 各 app 的**可生成性**与模板要素。
/// 这是浏览器腿的实际出口：不依赖包片段 lane / S1 / MIR。
fn lane_sfc(app: &str, rel: &str) -> String {
    let base = root().join(app);
    let src = std::fs::read_to_string(base.join(rel)).unwrap_or_default();
    let prepped = prepare_component_source(&base, &src).map(|p| p.source).unwrap_or(src);
    generate_sfc_with_lane(&prepped, rel)
        .unwrap_or_else(|e| panic!("{app} lane SFC: {e}"))
        .0
}

#[test]
fn p105_u03_apps_lane_sfc_generate() {
    std::env::set_var("P105_MEM_BACKEND", "1");
    for app in ["013-todo", "015-notes", "016-calendar", "022-kanban", "012-clock"] {
        let sfc = lane_sfc(app, "src/front/app.at");
        assert!(sfc.contains("<template>"), "{app}: 模板");
        assert!(sfc.contains("@click"), "{app}: 事件绑定");
    }
    // 列表渲染：013 的子组件、016/012 的主视图已进模板。
    assert!(lane_sfc("016-calendar", "src/front/app.at").contains("v-for"), "016 v-for");
    assert!(lane_sfc("012-clock", "src/front/app.at").contains("v-for"), "012 v-for");
    assert!(
        lane_sfc("013-todo", "src/front/todo_list.at").contains("v-for"),
        "013 todo_list v-for"
    );
}