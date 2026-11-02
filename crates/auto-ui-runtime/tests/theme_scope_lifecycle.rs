//! PLAN-031 T-02/T-03（work）——R-02「mode 先变 / 失败半发布」、R-01
//! 「同名新内容未发布」与 App 级 scope 观察面回归。
//!
//! 走生产链：真实 `.at` 源码 → `build_dynamic_component`（主题注册 + design
//! 块登记 + 组件 scope 建立）→ `VmBridge` 状态 → `AuraViewBuilder::eval_app_design`
//! 每帧求值 → 组件 scope 内原子发布 → VM 语义色投影 + scope revision。
//!
//! 观察面：T-03 起主题选择由组件 scope owner，读回经
//! `DynamicComponent::with_theme_scope`（宿主端口的最小形态），不再读进程级
//! 全局槽——后者只服务未绑定主题的 legacy 组件。

use std::path::Path;

use auto_ui_compiler::style::theme;
use auto_ui_compiler::style::Color;
use auto_ui_runtime::ui::dynamic::DynamicComponent;
use auto_val::Value;

/// 基线族：Base 声明 light+dark；LightOnly 只声明 light（合法单模式包）。
const THEMES: &str = r##"
pub theme Plan031Base = {
    tokens: {
        colors: {
            background: "#0f0f0f"
            foreground: "#eeeeee"
            primary: "#8888ff"
        }
    }
    modes: {
        light: {
            tokens: { colors: { background: "#0f0f0f" } }
        }
        dark: {
            tokens: { colors: { background: "#010101" } }
        }
    }
}

pub theme Plan031LightOnly = {
    tokens: {
        colors: {
            background: "#f0f0f0"
            foreground: "#111111"
            primary: "#8888ff"
        }
    }
    modes: {
        light: {
            tokens: { colors: { background: "#f0f0f0" } }
        }
    }
}
"##;

fn same_theme(name: &str, background: &str) -> String {
    format!(
        "pub theme {name} = {{\n  tokens: {{\n    colors: {{\n      background: \"{background}\"\n      foreground: \"#202020\"\n    }}\n  }}\n  modes: {{\n    light: {{\n      tokens: {{ colors: {{ background: \"{background}\" }} }}\n    }}\n    dark: {{\n      tokens: {{ colors: {{ background: \"#000000\" }} }}\n    }}\n  }}\n}}\n"
    )
}

fn app_source(themes: &str, theme_expr: &str, default_theme: &str, default_dark: bool) -> String {
    format!(
        "{themes}\nwidget App {{\n    model {{\n        var theme_name str = \"{default_theme}\"\n        var dark_mode bool = {default_dark}\n    }}\n\n    design {{\n        theme: {theme_expr}\n        mode: if .dark_mode {{ \"dark\" }} else {{ \"light\" }}\n    }}\n\n    view {{\n        col {{ text \"probe\" {{}} }}\n    }}\n}}\n"
    )
}

fn build_app(dir: &Path, id: &str, src: &str) -> DynamicComponent {
    let front = dir.join(id).join("src/front");
    std::fs::create_dir_all(&front).unwrap();
    let path = front.join("app.at");
    std::fs::write(&path, src).unwrap();
    auto_ui_runtime::build_dynamic_component(src, Some(path.to_str().unwrap()))
        .unwrap_or_else(|e| panic!("build {id} 失败: {e}"))
}

fn vm_background() -> Option<(u8, u8, u8)> {
    theme::resolve_semantic_rgb(&Color::Background)
}

fn write(comp: &mut DynamicComponent, field: &str, value: Value) {
    comp.write_state(field, value)
        .unwrap_or_else(|e| panic!("write_state({field}) 失败: {e}"));
}

/// 应用 scope 读回（宿主端口形态）。
fn observed_name(comp: &DynamicComponent) -> String {
    comp.with_theme_scope(theme::theme_name)
}

fn observed_dark(comp: &DynamicComponent) -> bool {
    comp.with_theme_scope(theme::dark_mode)
}

fn observed_bg(comp: &DynamicComponent) -> Option<(u8, u8, u8)> {
    comp.with_theme_scope(vm_background)
}

fn observed_revision(comp: &DynamicComponent) -> u32 {
    comp.with_theme_scope(theme::theme_epoch)
}

/// R-02a：未知主题 + 合法新模式 —— 完整旧 pair（family+mode+revision）保留。
#[test]
fn r02_unknown_theme_with_mode_change_keeps_old_pair() {
    let dir = tempfile::tempdir().unwrap();
    let src = app_source(THEMES, ".theme_name", "Plan031Base", false);
    let mut comp = build_app(dir.path(), "a", &src);

    comp.view_with_debug();
    assert_eq!(observed_name(&comp), "Plan031Base");
    assert!(!observed_dark(&comp));
    assert_eq!(observed_bg(&comp), Some((15, 15, 15)));
    let e0 = observed_revision(&comp);

    write(&mut comp, "theme_name", Value::Str("Plan031Missing".into()));
    write(&mut comp, "dark_mode", Value::Bool(true));
    comp.view_with_debug();

    assert_eq!(
        observed_name(&comp),
        "Plan031Base",
        "未知主题整候选拒绝：旧 family 保留"
    );
    assert!(
        !observed_dark(&comp),
        "失败候选不得先写 mode（R-02 半发布：旧 mode 必须保留）"
    );
    assert_eq!(observed_revision(&comp), e0, "拒绝候选零 revision");
    assert_eq!(observed_bg(&comp), Some((15, 15, 15)), "画面仍是旧 light 树");
}

/// R-02b：已知族缺少请求模式 —— 同样整候选拒绝，不得先翻 mode。
#[test]
fn r02_undeclared_mode_keeps_old_pair() {
    let dir = tempfile::tempdir().unwrap();
    let src = app_source(THEMES, ".theme_name", "Plan031Base", false);
    let mut comp = build_app(dir.path(), "b", &src);

    comp.view_with_debug();
    assert_eq!(observed_name(&comp), "Plan031Base");
    assert!(!observed_dark(&comp));
    let e0 = observed_revision(&comp);

    write(
        &mut comp,
        "theme_name",
        Value::Str("Plan031LightOnly".into()),
    );
    write(&mut comp, "dark_mode", Value::Bool(true));
    comp.view_with_debug();

    assert_eq!(observed_name(&comp), "Plan031Base", "缺 dark 的族拒绝换肤");
    assert!(!observed_dark(&comp), "拒绝发生在 mode 写入之前");
    assert_eq!(observed_revision(&comp), e0);
    assert_eq!(observed_bg(&comp), Some((15, 15, 15)));
}

/// R-02c：selector 求值失败（theme 绑定字段缺失）—— 整个候选拒绝，另一
/// selector 的合法 mode 也不能单独发布。
#[test]
fn r02_selector_eval_failure_rejects_whole_candidate() {
    let dir = tempfile::tempdir().unwrap();

    // 先以合法组件建立旧有效 pair（Base/light）。
    let valid = app_source(THEMES, ".theme_name", "Plan031Base", false);
    let mut base = build_app(dir.path(), "base", &valid);
    base.view_with_debug();
    assert_eq!(observed_name(&base), "Plan031Base");
    assert!(!observed_dark(&base));
    let e0 = observed_revision(&base);

    // 失败组件：theme selector 引用不存在的 state 字段；mode selector 仍合法
    // 且求值到 "dark"。
    let failing = app_source(
        &same_theme("Plan031Broken", "#777777"),
        ".missing_theme_field",
        "Plan031Broken",
        true,
    );
    let comp = build_app(dir.path(), "failing", &failing);
    comp.view_with_debug();

    assert_eq!(
        observed_name(&base),
        "Plan031Base",
        "selector 求值失败保留旧 family（C-11）"
    );
    assert!(
        !observed_dark(&base),
        "另半个 selector 的成功结果不得单独发布（R-02 continue 差额）"
    );
    assert_eq!(observed_revision(&base), e0, "失败帧零 revision");
}

/// R-02d：合法完整 pair 一次发布（恰好 +1 revision）；单独 theme 变化同样
/// 只 +1；合法后继可从失败恢复。
#[test]
fn r02_legal_pair_publishes_once_and_recovers_after_failure() {
    let dir = tempfile::tempdir().unwrap();
    let src = app_source(THEMES, ".theme_name", "Plan031Base", false);
    let mut comp = build_app(dir.path(), "d", &src);

    comp.view_with_debug();
    assert_eq!(observed_name(&comp), "Plan031Base");
    assert!(!observed_dark(&comp));
    assert_eq!(observed_bg(&comp), Some((15, 15, 15)));

    // 仅 theme 变化（family Base → LightOnly，light 可用）：+1。
    let e0 = observed_revision(&comp);
    write(
        &mut comp,
        "theme_name",
        Value::Str("Plan031LightOnly".into()),
    );
    comp.view_with_debug();
    assert_eq!(observed_name(&comp), "Plan031LightOnly");
    assert_eq!(observed_revision(&comp), e0.wrapping_add(1));
    assert_eq!(observed_bg(&comp), Some((240, 240, 240)));

    // 失败候选（LightOnly 无 dark）后仍可恢复：改回 Base + dark，一次发布。
    write(&mut comp, "dark_mode", Value::Bool(true));
    comp.view_with_debug();
    assert_eq!(
        observed_name(&comp),
        "Plan031LightOnly",
        "缺 dark 候选整体拒绝"
    );
    assert!(!observed_dark(&comp));

    let e1 = observed_revision(&comp);
    write(&mut comp, "theme_name", Value::Str("Plan031Base".into()));
    comp.view_with_debug();
    assert_eq!(observed_name(&comp), "Plan031Base");
    assert!(observed_dark(&comp));
    assert_eq!(
        observed_revision(&comp),
        e1.wrapping_add(1),
        "theme+mode 同帧完整候选必须一次发布（旧实现 +2）"
    );
    assert_eq!(observed_bg(&comp), Some((1, 1, 1)), "dark 差异生效");
}

/// R-01（consumer 面）：同进程先后两个 App 注册同名主题、内容不同；各自
/// scope 内渲染看到自己的内容，交替渲染不串（旧实现滞留第一个 App 的树）。
#[test]
fn r01_same_name_update_reaches_real_builder_projection() {
    let dir = tempfile::tempdir().unwrap();

    let src_a = app_source(
        &same_theme("Plan031Builder", "#010101"),
        ".theme_name",
        "Plan031Builder",
        false,
    );
    let mut comp_a = build_app(dir.path(), "app-a", &src_a);
    comp_a.view_with_debug();
    assert_eq!(observed_name(&comp_a), "Plan031Builder");
    assert_eq!(observed_bg(&comp_a), Some((1, 1, 1)));
    assert_eq!(observed_revision(&comp_a), 1, "首帧发布恰好一个 revision");

    let src_b = app_source(
        &same_theme("Plan031Builder", "#020202"),
        ".theme_name",
        "Plan031Builder",
        false,
    );
    let mut comp_b = build_app(dir.path(), "app-b", &src_b);
    comp_b.view_with_debug();
    assert_eq!(
        observed_bg(&comp_b),
        Some((2, 2, 2)),
        "同名新内容必须在 B 自己的 scope 生效"
    );
    assert_eq!(observed_revision(&comp_b), 1);

    // A 再次渲染仍是 A 的内容（scope 隔离；无变化不增 revision）。
    comp_a.view_with_debug();
    assert_eq!(observed_bg(&comp_a), Some((1, 1, 1)));
    assert_eq!(observed_revision(&comp_a), 1, "同名同内容不重复失效");

    // B 再次渲染仍是 B 的内容。
    comp_b.view_with_debug();
    assert_eq!(observed_bg(&comp_b), Some((2, 2, 2)));
    assert_eq!(observed_revision(&comp_b), 1);
}
