//! PLAN-105（U03）：真语料 app 的 **SFC 生成现状**（浏览器腿）——按 app 钉住当前
//! 首个阻断。S1（包级 `store` 语义）落地后这些断言会翻红，届时改为结构断言。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_sfc::generate_package_sfc;
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
            e.contains(needle),
            "{app} 的首个阻断应为 `{needle}`，实际: {e}"
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
        "package-level store semantics are not wired yet (S1)",
    );
    // 016：同一条 S1 闸（CalendarStore）。
    expect_gap(
        "016-calendar",
        &[
            ("main", "src/front/app.at"),
            ("calendar_store", "src/front/calendar_store.at"),
            ("calendar_util", "src/front/calendar_util.at"),
        ],
        "package-level store semantics are not wired yet (S1)",
    );
    // 015：`.store` 引用在**同源组件面**缺 store 声明（prep 未内联）。
    expect_gap(
        "015-notes",
        &[("main", "src/front/app.at")],
        "model has no field `store`",
    );
}
