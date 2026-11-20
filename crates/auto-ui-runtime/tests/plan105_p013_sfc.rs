//! PLAN-105 T-02（U03 013）：**真实 013-todo 源 → Vue SFC**（浏览器腿）的现状回归。
//!
//! 现状（W89）：包片段 lane 已能消费组件面 prep 产物，但包内 `store` 的
//! 共享实例语义仍未接线（S1）——本用例把该**具名拒绝**钉住：S1 落地后它会
//! 翻红，届时改成结构断言（模板/`v-for`/`@click`/refs）。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_sfc::generate_package_sfc;
use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
use std::path::{Path, PathBuf};

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/013-todo")
}

fn prepared(rel: &str) -> String {
    let root = project_root();
    let src = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    match prepare_component_source(&root, &src) {
        Ok(p) => p.source,
        Err(e) => panic!("prep {rel}: {e}"),
    }
}

fn unit(id: &str, rel: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: prepared(rel),
        provider: None,
    }
}

fn generate() -> Result<String, String> {
    std::env::set_var("P105_MEM_BACKEND", "1");
    generate_package_sfc(
        &[
            unit("main", "src/front/app.at"),
            unit("todo_store", "src/front/todo_store.at"),
            unit("todo_list", "src/front/todo_list.at"),
            unit("api", "src/back/api.at"),
            unit("db", "src/back/db.at"),
        ],
        "app::main",
    )
}

#[test]
fn p105_p013_sfc_reports_the_package_store_gap() {
    match generate() {
        Ok(sfc) => {
            // S1 落地后的期望形状（届时本断言生效）：模板 + 列表渲染 + 事件。
            assert!(sfc.contains("<template>"), "template block: {sfc}");
            assert!(sfc.contains("v-for"), "list rendering reaches the template: {sfc}");
            assert!(sfc.contains("@click"), "event bindings reach the template: {sfc}");
        }
        Err(e) => {
            assert!(
                e.contains("package-level store semantics are not wired yet (S1)"),
                "013 的浏览器腿当前应停在 S1 具名拒绝，实际: {e}"
            );
        }
    }
}
