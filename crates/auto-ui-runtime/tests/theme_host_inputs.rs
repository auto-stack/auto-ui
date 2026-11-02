//! PLAN-031 T-03（work）——宿主/受控主题输入优先级与 preview 子 scope
//! （§5.2：App override → 声明 follow 的宿主选择 → App design 默认；非法
//! 高优先来源整帧拒绝、不回落）。
//!
//! 走真实源码 → build_dynamic_component（组件 scope）→ 宿主端口
//! `set_host_theme_inputs` → view_with_debug（tracked 转换）→ scope 读回。

use std::path::Path;

use auto_ui_compiler::style::theme;
use auto_ui_compiler::style::Color;
use auto_ui_compiler::style::theme::{HostThemeInputs, ModeInput, ThemeInput};
use auto_ui_runtime::ui::dynamic::DynamicComponent;
use auto_val::Value;

const APP: &str = r##"
pub theme IsoH1 = {
    tokens: {
        colors: {
            background: "#111111"
            foreground: "#eeeeee"
            primary: "#8888ff"
        }
    }
    modes: {
        light: {
            tokens: { colors: { background: "#111111" } }
        }
        dark: {
            tokens: { colors: { background: "#010101" } }
        }
    }
}

pub theme IsoH2 = {
    tokens: {
        colors: {
            background: "#222222"
            foreground: "#dddddd"
            primary: "#ff8888"
        }
    }
    modes: {
        light: {
            tokens: { colors: { background: "#222222" } }
        }
        dark: {
            tokens: { colors: { background: "#020202" } }
        }
    }
}

widget App {
    model {
        var theme_name str = "IsoH1"
        var dark_mode bool = false
    }

    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }

    view {
        col { text "probe" {} }
    }
}
"##;

fn build_app(dir: &Path, id: &str) -> DynamicComponent {
    let front = dir.join(id).join("src/front");
    std::fs::create_dir_all(&front).unwrap();
    let path = front.join("app.at");
    std::fs::write(&path, APP).unwrap();
    auto_ui_runtime::build_dynamic_component(APP, Some(path.to_str().unwrap()))
        .unwrap_or_else(|e| panic!("build {id} 失败: {e}"))
}

fn write(comp: &mut DynamicComponent, field: &str, value: Value) {
    comp.write_state(field, value)
        .unwrap_or_else(|e| panic!("write_state({field}) 失败: {e}"));
}

fn set_host(comp: &mut DynamicComponent, inputs: HostThemeInputs) {
    comp.set_host_theme_inputs(inputs);
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

fn bg(comp: &DynamicComponent) -> Option<(u8, u8, u8)> {
    comp.with_theme_scope(|| theme::resolve_semantic_rgb(&Color::Background))
}

/// App override 优先于 App design 默认；清空输入后回落 design。
#[test]
fn app_override_beats_design_binding() {
    let dir = tempfile::tempdir().unwrap();
    let mut comp = build_app(dir.path(), "a");

    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH1");
    assert_eq!(bg(&comp), Some((17, 17, 17)));

    set_host(
        &mut comp,
        HostThemeInputs {
            app_override: ThemeInput::Value("IsoH2".into()),
            ..Default::default()
        },
    );
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH2");
    assert_eq!(bg(&comp), Some((34, 34, 34)));

    set_host(&mut comp, HostThemeInputs::default());
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH1", "Unset 后回落 App design 默认");
}

/// follow_host=false 时宿主 family 不生效；声明 follow 后生效。
#[test]
fn follow_host_gate() {
    let dir = tempfile::tempdir().unwrap();
    let mut comp = build_app(dir.path(), "b");
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH1");

    set_host(
        &mut comp,
        HostThemeInputs {
            follow_host: false,
            host_family: ThemeInput::Value("IsoH2".into()),
            ..Default::default()
        },
    );
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH1", "未声明 follow 的 App 忽略宿主 family");

    set_host(
        &mut comp,
        HostThemeInputs {
            follow_host: true,
            host_family: ThemeInput::Value("IsoH2".into()),
            ..Default::default()
        },
    );
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH2");
}

/// 非法高优先来源：整帧拒绝、保留旧 pair、不得回落到合法的 design 值。
#[test]
fn invalid_high_priority_source_does_not_fall_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut comp = build_app(dir.path(), "c");
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH1");
    let rev = revision(&comp);

    // design 已指向合法 IsoH2——若错误回落会立即发布。
    write(&mut comp, "theme_name", Value::Str("IsoH2".into()));
    set_host(
        &mut comp,
        HostThemeInputs {
            app_override: ThemeInput::Invalid("host bus version mismatch".into()),
            ..Default::default()
        },
    );
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH1", "非法 override 保留旧 family");
    assert_eq!(revision(&comp), rev, "拒绝帧零 revision");

    // 清空非法输入后 design 恢复生效（证明仍可恢复）。
    set_host(&mut comp, HostThemeInputs::default());
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH2");
}

/// 宿主同时给 family+mode：完全接管（即使 App design 求值会失败）。
#[test]
fn full_host_candidate_overrides_broken_design() {
    let dir = tempfile::tempdir().unwrap();
    let mut comp = build_app(dir.path(), "d");
    comp.view_with_debug();

    write(&mut comp, "theme_name", Value::Str("IsoMissing".into()));
    set_host(
        &mut comp,
        HostThemeInputs {
            app_override: ThemeInput::Value("IsoH2".into()),
            app_override_mode: ModeInput::Dark,
            ..Default::default()
        },
    );
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH2");
    assert!(dark(&comp), "宿主 mode 一并接管");
    assert_eq!(bg(&comp), Some((2, 2, 2)));

    // 清空宿主输入后，非法 design 保留旧有效 pair（不半发布）。
    set_host(&mut comp, HostThemeInputs::default());
    comp.view_with_debug();
    assert_eq!(name(&comp), "IsoH2");
    assert!(dark(&comp));
}

/// 宿主 mode 独立于 family 生效（follow 门控内）。
#[test]
fn host_mode_priority() {
    let dir = tempfile::tempdir().unwrap();
    let mut comp = build_app(dir.path(), "e");
    comp.view_with_debug();
    assert!(!dark(&comp));

    set_host(
        &mut comp,
        HostThemeInputs {
            follow_host: true,
            host_mode: ModeInput::Dark,
            ..Default::default()
        },
    );
    comp.view_with_debug();
    assert!(dark(&comp), "宿主 mode 覆盖 App design light");
    assert_eq!(bg(&comp), Some((1, 1, 1)), "IsoH1 dark 差异生效");
    assert_eq!(name(&comp), "IsoH1", "family 仍由 App 默认提供");
}

/// preview 子 scope：继承父选择、独立 revision/身份，发布不回流父；
/// 关闭 preview 不影响父（V-10）。
#[test]
fn preview_scope_isolated_from_parent() {
    let dir = tempfile::tempdir().unwrap();
    let mut parent = build_app(dir.path(), "p");
    parent.view_with_debug();
    assert_eq!(name(&parent), "IsoH1");
    let parent_rev = revision(&parent);

    let preview_scope = parent.preview_theme_scope().expect("parent scope");
    let mut preview = build_app(dir.path(), "p-preview");
    preview.attach_theme_scope(preview_scope);
    write(&mut preview, "theme_name", Value::Str("IsoH2".into()));
    preview.view_with_debug();
    assert_eq!(name(&preview), "IsoH2");
    assert_eq!(bg(&preview), Some((34, 34, 34)));

    // 父保持原选择与 revision（preview 发布不回流）。
    parent.view_with_debug();
    assert_eq!(name(&parent), "IsoH1");
    assert_eq!(revision(&parent), parent_rev);

    // 关闭 preview 后父仍正确。
    drop(preview);
    parent.view_with_debug();
    assert_eq!(name(&parent), "IsoH1");
    assert_eq!(bg(&parent), Some((17, 17, 17)));
}
