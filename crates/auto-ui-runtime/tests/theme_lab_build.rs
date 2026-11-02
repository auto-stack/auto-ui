//! PLAN-007 T-05：Theme Lab 源码端到端构建测试（VM 轨，无窗口）。
//!
//! 真实源码 → build_dynamic_component（use/pac dep 主题注册链）→ 树状
//! 主题按名 lookup → 模式重建 → 语义色解析。V-01/02 的运行时面投影。

use auto_ui_compiler::design_tokens::registry::TokenName;

const THEME_LAB: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/theme-lab/src/front/app.at"
);

#[test]
fn theme_lab_builds_and_registers_official_trees() {
    let src = std::fs::read_to_string(THEME_LAB).expect("theme-lab app.at 在场");
    let path = THEME_LAB.to_string();

    let comp = auto_ui_runtime::build_dynamic_component(&src, Some(&path))
        .expect("theme-lab 源码构建成功（含主题注册链）");
    assert_eq!(comp.widget_name(), "App");

    // 官方两族已注册（dep themes 链）
    let auto_tree = auto_ui_compiler::style::theme::lookup_theme_tree("Auto")
        .expect("Auto 主题已注册");
    let sol_tree = auto_ui_compiler::style::theme::lookup_theme_tree("Solarized")
        .expect("Solarized 主题已注册");

    // Solarized light：基座继承 + 覆盖（背景 #fdf6e3）
    let sol_light =
        auto_ui_compiler::design_tokens::theme_tree::compose_mode(&sol_tree, false).unwrap();
    assert_eq!(
        auto_ui_compiler::design_tokens::theme_tree::resolve_rgb(&sol_light, TokenName::Background),
        Some((253, 246, 227))
    );
    // Auto dark：暗色差异生效
    let auto_dark =
        auto_ui_compiler::design_tokens::theme_tree::compose_mode(&auto_tree, true).unwrap();
    assert_eq!(
        auto_ui_compiler::design_tokens::theme_tree::resolve_rgb(&auto_dark, TokenName::Background),
        Some((9, 9, 11))
    );

    // 模式可用性（C-07）：两族均声明 light+dark
    assert!(auto_ui_compiler::design_tokens::theme_tree::mode_available(&sol_tree, true));
    assert!(auto_ui_compiler::design_tokens::theme_tree::mode_available(&sol_tree, false));

    // design 块条目已登记（theme/mode 数据绑定）
    let entries = auto_ui_compiler::style::theme::app_design_entries();
    let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
    assert!(keys.contains(&"theme"), "{keys:?}");
    assert!(keys.contains(&"mode"), "{keys:?}");
}
