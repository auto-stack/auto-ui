//! PLAN-031 T-03（work）——App/窗口主题作用域隔离与生命周期（V-03/V-10
//! 的 VM 侧切面）。
//!
//! 同一进程内多个 App 各自持有 `ThemeScope`：family/mode/目录/design/revision
//! 互不覆盖；同一 App 的多窗口可共享一个 scope；关闭一个 App 不残留监听或
//! 选择状态到另一个；失败候选不跨 App 泄漏。

use std::path::Path;

use auto_ui_compiler::style::theme;
use auto_ui_compiler::style::Color;
use auto_ui_runtime::ui::dynamic::DynamicComponent;
use auto_val::Value;

const THEMES: &str = r##"
pub theme IsoA = {
    tokens: {
        colors: {
            background: "#0a0a0a"
            foreground: "#eeeeee"
            primary: "#8888ff"
        }
    }
    modes: {
        light: {
            tokens: { colors: { background: "#0a0a0a" } }
        }
        dark: {
            tokens: { colors: { background: "#050505" } }
        }
    }
}

pub theme IsoB = {
    tokens: {
        colors: {
            background: "#0b0b0b"
            foreground: "#dddddd"
            primary: "#ff8888"
        }
    }
    modes: {
        light: {
            tokens: { colors: { background: "#0b0b0b" } }
        }
        dark: {
            tokens: { colors: { background: "#060606" } }
        }
    }
}
"##;

fn app_source(default_theme: &str, default_dark: bool) -> String {
    format!(
        "{THEMES}\nwidget App {{\n    model {{\n        var theme_name str = \"{default_theme}\"\n        var dark_mode bool = {default_dark}\n    }}\n\n    design {{\n        theme: .theme_name\n        mode: if .dark_mode {{ \"dark\" }} else {{ \"light\" }}\n    }}\n\n    view {{\n        col {{ text \"probe\" {{}} }}\n    }}\n}}\n"
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

fn write(comp: &mut DynamicComponent, field: &str, value: Value) {
    comp.write_state(field, value)
        .unwrap_or_else(|e| panic!("write_state({field}) 失败: {e}"));
}

fn bg(comp: &DynamicComponent) -> Option<(u8, u8, u8)> {
    comp.with_theme_scope(|| theme::resolve_semantic_rgb(&Color::Background))
}

fn name(comp: &DynamicComponent) -> String {
    comp.with_theme_scope(theme::theme_name)
}

fn dark(comp: &DynamicComponent) -> bool {
    comp.with_theme_scope(theme::dark_mode)
}

fn revision(comp: &DynamicComponent) -> u32 {
    comp.with_theme_scope(theme::theme_epoch)
}

/// V-03（VM 侧）：同进程两 App 的 family/mode/revision 各自独立，交替渲染
/// 不串；改一个 App 的 mode 不影响另一个。
#[test]
fn two_apps_keep_independent_family_mode_and_revision() {
    let dir = tempfile::tempdir().unwrap();
    let mut app_a = build_app(dir.path(), "a", &app_source("IsoA", false));
    let mut app_b = build_app(dir.path(), "b", &app_source("IsoB", true));

    app_a.view_with_debug();
    assert_eq!(name(&app_a), "IsoA");
    assert!(!dark(&app_a));
    assert_eq!(bg(&app_a), Some((10, 10, 10)));
    assert_eq!(revision(&app_a), 1);

    app_b.view_with_debug();
    assert_eq!(name(&app_b), "IsoB");
    assert!(dark(&app_b));
    assert_eq!(bg(&app_b), Some((6, 6, 6)));
    assert_eq!(revision(&app_b), 1);

    // A 再渲染：仍 A/light（B 的发布没有改写 A 的 scope）。
    app_a.view_with_debug();
    assert_eq!(name(&app_a), "IsoA");
    assert!(!dark(&app_a));
    assert_eq!(bg(&app_a), Some((10, 10, 10)));
    assert_eq!(revision(&app_a), 1, "无变化不增 revision");

    // 只切 A 的 mode：A → 2，B 保持 1 与自身值。
    write(&mut app_a, "dark_mode", Value::Bool(true));
    app_a.view_with_debug();
    assert!(dark(&app_a));
    assert_eq!(bg(&app_a), Some((5, 5, 5)));
    assert_eq!(revision(&app_a), 2);

    app_b.view_with_debug();
    assert_eq!(name(&app_b), "IsoB");
    assert!(dark(&app_b));
    assert_eq!(bg(&app_b), Some((6, 6, 6)));
    assert_eq!(revision(&app_b), 1, "B 的 revision 不受 A 影响");
}

/// 同一 App 多窗口：两个组件附加同一个 scope → 选择共享（同选择不重复失效，
/// 选择变化对两端可见）；未附加的第三个 App 仍独立。
///
/// 边界：本切片的共享 scope 只共享「选择/目录/revision」。两窗口若各自持有
/// 分歧的 selector（不同 `.theme_name`），后渲染者会按自己的设计值发布——
/// 显式 preview 子 scope 与 session 级选择属 T-03 后续切片/T-07 宿主合同；
/// 真实同 App 窗口共享业务 store，不产生该分歧。
#[test]
fn shared_scope_links_windows_of_one_app() {
    let dir = tempfile::tempdir().unwrap();
    let mut win1 = build_app(dir.path(), "w1", &app_source("IsoA", false));
    let mut win2 = build_app(dir.path(), "w2", &app_source("IsoA", false));

    let shared = win1.theme_scope().expect("有 design 绑定的组件必须有 scope");
    win2.attach_theme_scope(shared);

    win1.view_with_debug();
    assert_eq!(name(&win1), "IsoA");
    assert_eq!(revision(&win1), 1);

    // 第二窗口同一 scope：同选择不重复发布。
    win2.view_with_debug();
    assert_eq!(name(&win2), "IsoA");
    assert_eq!(revision(&win2), 1, "共享 scope 同一选择不重复失效");

    // App 会话改变主题（两窗口的状态同源；此处显式同步），选择对两端可见。
    write(&mut win1, "theme_name", Value::Str("IsoB".into()));
    write(&mut win2, "theme_name", Value::Str("IsoB".into()));
    win2.view_with_debug();
    assert_eq!(name(&win2), "IsoB");
    assert_eq!(revision(&win2), 2);

    win1.view_with_debug();
    assert_eq!(name(&win1), "IsoB", "同 App 窗口共享选择");
    assert_eq!(bg(&win1), Some((11, 11, 11)));
    assert_eq!(revision(&win1), 2, "同一选择不重复失效");

    // 另一 App 独立 scope 不受影响。
    let mut other = build_app(dir.path(), "other", &app_source("IsoA", false));
    other.view_with_debug();
    assert_eq!(name(&other), "IsoA");
    assert_eq!(revision(&other), 1);
}

/// V-10（VM 侧）：关闭一个 App 不残留状态；其余 App 继续正确工作；重开
/// 得到全新 scope（revision 从 0 起、按自身选择发布）。
#[test]
fn closing_one_app_leaves_the_other_intact_and_reopen_is_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let src = app_source("IsoA", false);
    let mut app_a = build_app(dir.path(), "a", &src);
    let mut app_b = build_app(dir.path(), "b", &app_source("IsoB", true));

    app_a.view_with_debug();
    app_b.view_with_debug();
    assert_eq!(bg(&app_a), Some((10, 10, 10)));
    assert_eq!(bg(&app_b), Some((6, 6, 6)));

    drop(app_b);

    // A 继续工作，且其 scope 值未被关闭动作影响。
    app_a.view_with_debug();
    assert_eq!(name(&app_a), "IsoA");
    assert_eq!(bg(&app_a), Some((10, 10, 10)));
    write(&mut app_a, "dark_mode", Value::Bool(true));
    app_a.view_with_debug();
    assert_eq!(bg(&app_a), Some((5, 5, 5)));
    assert_eq!(revision(&app_a), 2);

    // 重开同一 App：新 scope（fresh revision），首帧按自身 design 发布。
    let mut reopened = build_app(dir.path(), "a2", &src);
    assert_eq!(revision(&reopened), 0, "重开组件初始无发布");
    reopened.view_with_debug();
    assert_eq!(name(&reopened), "IsoA");
    assert!(!dark(&reopened));
    assert_eq!(bg(&reopened), Some((10, 10, 10)));
    assert_eq!(revision(&reopened), 1);
}

/// 失败候选不跨 App 泄漏：B 的非法选择被自身 scope 拒绝，A 已发布的选择
/// 与 revision 完全不变。
#[test]
fn failed_candidate_does_not_cross_apps() {
    let dir = tempfile::tempdir().unwrap();
    let mut app_a = build_app(dir.path(), "a", &app_source("IsoA", false));
    app_a.view_with_debug();
    let a_rev = revision(&app_a);
    assert_eq!(name(&app_a), "IsoA");
    assert_eq!(bg(&app_a), Some((10, 10, 10)));

    // B：design 引用未注册族并请求 dark。
    let bad_src = format!(
        "{THEMES}\nwidget App {{\n    model {{\n        var theme_name str = \"IsoMissing\"\n        var dark_mode bool = true\n    }}\n\n    design {{\n        theme: .theme_name\n        mode: if .dark_mode {{ \"dark\" }} else {{ \"light\" }}\n    }}\n\n    view {{\n        col {{ text \"probe\" {{}} }}\n    }}\n}}\n"
    );
    let mut app_b = build_app(dir.path(), "b", &bad_src);
    app_b.view_with_debug();
    assert!(
        name(&app_b) != "IsoA",
        "失败候选不得落到 A 的 family（B 保留自己的 capture 快照）"
    );
    assert_eq!(revision(&app_b), 0, "失败候选零 revision");

    // A 不受影响。
    app_a.view_with_debug();
    assert_eq!(name(&app_a), "IsoA");
    assert!(!dark(&app_a));
    assert_eq!(bg(&app_a), Some((10, 10, 10)));
    assert_eq!(revision(&app_a), a_rev);
}
