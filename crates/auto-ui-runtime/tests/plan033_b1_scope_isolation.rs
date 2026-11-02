//! PLAN-033 T-05（work）——B1 示例官方主题接入的 VM 侧隔离/优先级切面
//! （V-05 的进程内真实源腿）。
//!
//! 真实仓库源码（examples/ui/001-helloworld、006-hero-section，含 pac
//! `dep themes` 包注册链）→ `build_dynamic_component` → design 绑定发布 →
//! 同进程双 App 相反主题互不串（色/族/revision），宿主 override/follow
//! 交替与关闭重开按 §5.2 优先级；业务状态不被主题改写。

use std::path::PathBuf;

use auto_ui_compiler::style::theme::{self, HostThemeInputs, ModeInput, ThemeInput};
use auto_ui_compiler::style::Color;
use auto_ui_runtime::ui::dynamic::DynamicComponent;

fn example_path(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui")
        .join(rel)
        .join("src/front/app.at");
    p.canonicalize()
        .unwrap_or_else(|e| panic!("例程源码解析失败 {}: {e}", p.display()))
        .to_str()
        .unwrap()
        .to_string()
}

fn build(rel: &str) -> DynamicComponent {
    let path = example_path(rel);
    let src = std::fs::read_to_string(&path).expect("例程源码在场");
    auto_ui_runtime::build_dynamic_component(&src, Some(&path))
        .unwrap_or_else(|e| panic!("{rel} 构建失败: {e}"))
}

fn family(comp: &DynamicComponent) -> String {
    comp.with_theme_scope(theme::theme_name)
}

fn is_dark(comp: &DynamicComponent) -> bool {
    comp.with_theme_scope(theme::dark_mode)
}

fn rgb(comp: &DynamicComponent, color: &Color) -> Option<(u8, u8, u8)> {
    comp.with_theme_scope(|| theme::resolve_semantic_rgb(color))
}

/// design 绑定模型同帧变更 → 视图构建期发布（C-06/07）。
fn drive(comp: &mut DynamicComponent, family_name: &str, dark: bool) {
    comp.write_state("theme_name", auto_val::Value::str(family_name))
        .expect("theme_name 状态写在场");
    comp.write_state("dark_mode", auto_val::Value::Bool(dark))
        .expect("dark_mode 状态写在场");
    let _ = comp.view_with_debug();
}

#[test]
fn plan033_two_examples_opposite_themes_isolated_in_process() {
    let mut hello = build("001-helloworld");
    let mut hero = build("006-hero-section");
    // 首帧（真实 boot 语义）：design 绑定默认经视图构建期发布到各自 scope。
    // 注：eval 读的是 build 期登记的 app_design_entries 全局槽（PLAN-031
    // 形态）；六例 design 形状一致（theme/mode 绑定）故本测试无形状冲突。
    let _ = hello.view_with_debug();
    let _ = hero.view_with_debug();

    // 默认：001 = Auto/light（声明默认）；006 = Auto/dark（模型显式默认，
    // pac theme:"dark" 的运行链另由运行腿覆盖）。
    assert_eq!(family(&hello), "Auto");
    assert!(!is_dark(&hello));
    assert_eq!(family(&hero), "Auto");
    assert!(is_dark(&hero));

    // 相反主题：001 → Solarized/dark；006 保持 Auto/dark。两 scope 互不覆盖。
    drive(&mut hello, "Solarized", true);
    assert_eq!(family(&hello), "Solarized");
    assert!(is_dark(&hello));
    assert_eq!(family(&hero), "Auto", "006 family 不得被 001 的切换覆盖");
    assert!(is_dark(&hero));

    // 有效值隔离：Background 按各自 scope 解析——
    // 001 = Solarized dark #002b36；006 = Auto dark #09090b（非串色）。
    let hello_bg = rgb(&hello, &Color::Background).expect("001 Background 可解析");
    let hero_bg = rgb(&hero, &Color::Background).expect("006 Background 可解析");
    assert_eq!(hello_bg, (0x00, 0x2b, 0x36), "001 Solarized dark 包值");
    assert_eq!(hero_bg, (0x09, 0x09, 0x0b), "006 Auto dark 包值");

    // 006 反向换族，001 不回传。
    drive(&mut hero, "Solarized", true);
    assert_eq!(family(&hero), "Solarized");
    assert_eq!(family(&hello), "Solarized");
    drive(&mut hero, "Auto", true);
    assert_eq!(family(&hero), "Auto");
    assert_eq!(family(&hello), "Solarized", "001 选择不受 006 换族影响");
}

#[test]
fn plan033_host_override_priority_and_close_reopen() {
    let mut comp = build("001-helloworld");

    // App override（最高优先）：宿主 family+mode 压过 design 绑定默认。
    comp.set_host_theme_inputs(HostThemeInputs {
        app_override: ThemeInput::Value("Solarized".to_string()),
        app_override_mode: ModeInput::Dark,
        ..Default::default()
    });
    let _ = comp.view_with_debug();
    assert_eq!(family(&comp), "Solarized", "宿主 override 生效");
    assert!(is_dark(&comp));

    // Override 交替回退到 design 绑定默认（follow/override 交替选择）。
    comp.set_host_theme_inputs(HostThemeInputs::default());
    let _ = comp.view_with_debug();
    assert_eq!(family(&comp), "Auto", "override 释放后回 design 默认");
    assert!(!is_dark(&comp));

    // 非法高优先来源 = 整帧拒绝并保旧（C-02/C-03；不静默回落）。
    comp.set_host_theme_inputs(HostThemeInputs {
        app_override: ThemeInput::Value("NoSuchFamily".to_string()),
        app_override_mode: ModeInput::Light,
        ..Default::default()
    });
    let _ = comp.view_with_debug();
    assert_eq!(family(&comp), "Auto", "非法 override 整帧拒绝保旧");
    assert!(!is_dark(&comp));
    comp.set_host_theme_inputs(HostThemeInputs::default());
    let _ = comp.view_with_debug();

    // 关闭重开 = 独立新 scope，不继承前一次临时选择（C-06）。
    drop(comp);
    let mut reopened = build("001-helloworld");
    let _ = reopened.view_with_debug();
    assert_eq!(family(&reopened), "Auto", "重开得独立新 scope");
    assert!(!is_dark(&reopened));

    // 非法族经 design 绑定路径同样整候选拒绝保旧（model 写非法名；先 boot
    // 帧发布合法默认，再非法变更——真实运行序列）。
    let mut comp = build("001-helloworld");
    let _ = comp.view_with_debug();
    comp.write_state("theme_name", auto_val::Value::str("NoSuchFamily"))
        .expect("theme_name 写入");
    let _ = comp.view_with_debug();
    assert_eq!(family(&comp), "Auto", "design 绑定非法族保旧（未注册拒绝）");
}
