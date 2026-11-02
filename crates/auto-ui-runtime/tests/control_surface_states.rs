//! PLAN-031 T-05（work）——基础控件表面（V-05 的投影/状态腿）：
//!
//! - `panel`/`card` 是结构容器（§5.4 Panel/文本标签）：class/style 与
//!   `design:` 局部 frame 都有消费面，父→子继承、子覆盖、兄弟不串；
//! - 控件状态经**同一 scope 快照**表达：button 的 disabled 不 dispatch、
//!   hover 变体保留；input/textarea 的 border/ring/radius 与错误态
//!   `border-destructive` 解析为语义色。
//!
//! 走真实源码 → `build_dynamic_component` → `view_with_debug`（tracked 转换
//! 路径）→ View 结构类读回。与 `design_scope_inheritance.rs` 同一观测口径；
//! 像素级双端渲染（Vue/VM 实跑）归 T-06 的 003/010 真实腿。

use auto_ui_compiler::style::class::{RoundedSize, StyleClass};
use auto_ui_compiler::style::color::Color;
use auto_ui_compiler::style::{Style, Variant};
use auto_ui_runtime::ui::dynamic::DynamicComponent;
use auto_ui_runtime::ui::interpreter::DynamicMessage;
use auto_ui_runtime::ui::view::View;
use auto_val::Value;

const APP: &str = r##"
pub theme Surf = {
    tokens: {
        colors: {
            background: "#101010"
            foreground: "#eeeeee"
            primary: "#8888ff"
            primary-foreground: "#ffffff"
            card: "#112233"
            border: "#334455"
            ring: "#ff00ff"
            destructive: "#cc0000"
            input: "#223344"
        }
        radius: { control: "10dp" }
    }
    modes: {
        light: { tokens: { colors: { background: "#101010" } } }
        dark: { tokens: { colors: { background: "#010101" } } }
    }
}

pub theme Surf2 = {
    tokens: {
        colors: {
            background: "#202020"
            foreground: "#dddddd"
            primary: "#8888ff"
            card: "#445566"
            border: "#667788"
            ring: "#00ff00"
            destructive: "#cc0000"
            input: "#223344"
        }
        radius: { control: "12dp" }
    }
    modes: {
        light: { tokens: { colors: { background: "#202020" } } }
        dark: { tokens: { colors: { background: "#020202" } } }
    }
}

widget App {
    msg { Hit }

    model {
        var theme_name str = "Surf"
        var dark_mode bool = false
        var name str = ""
        var note str = ""
        var blocked bool = true
    }

    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }

    view {
        col {
            // A：局部 design 四类槽（radius 槽引用 + 四种颜色）。
            panel {
                design: {
                    radius: tokens.radius.control
                    background: tokens.colors.card
                    foreground: tokens.colors.foreground
                    border: tokens.colors.border
                    ring: tokens.colors.ring
                }
                text "A" { style: "text-sm" }
            }

            // B：无 design 的兄弟——不继承 A 的任何局部值。
            panel {
                text "B" { style: "text-sm" }
            }

            // C 的父面板给 radius；C 自己只给 background（继承 + 覆盖）。
            panel {
                design: { radius: tokens.radius.control }
                panel {
                    design: { background: tokens.colors.card }
                    text "C" { style: "text-sm" }
                }
                text "D" { style: "text-sm" }
            }

            // 字面量：不随主题槽变。
            panel {
                design: { radius: "6dp", background: "#123456" }
                text "E" { style: "text-sm" }
            }

            // 控件表面：disabled 由模型字段驱动，hover 变体经 class 声明。
            button "Send" {
                onclick: .Hit
                disabled: .blocked
                style: "rounded-lg bg-primary text-primary-foreground border border-border hover:bg-primary/90"
            }

            // 错误态用语义色（005 同款 text-destructive 家族的 border 版）。
            // PLAN-031 T-05：placeholder 槽由本元素 design 声明（§5.3 冻结面）。
            input (value: .name) {
                placeholder: "name"
                disabled: .blocked
                design: { placeholder: "#ff0000" }
                style: "rounded-md border border-destructive bg-background text-foreground"
            }

            textarea (value: .note) {
                placeholder: "note"
                disabled: .blocked
                oninput: () => { }
                onkeydown.enter: () => { }
                design: { placeholder: "#00ff00" }
                style: "rounded-md border border-border bg-input text-foreground"
            }
        }
    }

    on {
        .Hit -> { }
    }
}
"##;

fn build_component(dir: &std::path::Path) -> DynamicComponent {
    let front = dir.join("app/src/front");
    std::fs::create_dir_all(&front).unwrap();
    let path = front.join("app.at");
    std::fs::write(&path, APP).unwrap();
    auto_ui_runtime::build_dynamic_component(APP, Some(path.to_str().unwrap())).expect("build app")
}

fn collect_styles<'a>(view: &'a View<DynamicMessage>, out: &mut Vec<&'a Style>) {
    match view {
        View::Column {
            style, children, ..
        }
        | View::Row {
            style, children, ..
        }
        | View::Grid {
            style,
            cells: children,
            ..
        } => {
            if let Some(style) = style {
                out.push(style);
            }
            for child in children {
                collect_styles(child, out);
            }
        }
        View::Container { style, child, .. } => {
            if let Some(style) = style {
                out.push(style);
            }
            collect_styles(child, out);
        }
        View::Scrollable { style, child, .. } => {
            if let Some(style) = style {
                out.push(style);
            }
            collect_styles(child, out);
        }
        View::Button {
            style,
            content,
            ..
        } => {
            if let Some(style) = style {
                out.push(style);
            }
            if let Some(content) = content {
                collect_styles(content, out);
            }
        }
        View::Input { style, .. } | View::Textarea { style, .. } => {
            if let Some(style) = style {
                out.push(style);
            }
        }
        View::Text { style, .. } => {
            if let Some(style) = style {
                out.push(style);
            }
        }
        _ => {}
    }
}

/// 面板可观测形态：四角半径、背景色、前景色、边色、环色。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Surface {
    radius: Option<f32>,
    bg: Option<(u8, u8, u8)>,
    fg: Option<(u8, u8, u8)>,
    border: Option<(u8, u8, u8)>,
    ring: Option<(u8, u8, u8)>,
}

fn rgb(c: &Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Rgba { r, g, b, .. } => Some((*r, *g, *b)),
        _ => None,
    }
}

fn surface_of(style: &Style) -> Surface {
    let mut s = Surface::default();
    for class in &style.classes {
        match class {
            StyleClass::RoundedT(Some(RoundedSize::Px(v))) => s.radius = Some(*v),
            StyleClass::BackgroundColor(c) => s.bg = rgb(c),
            StyleClass::TextColor(c) => s.fg = rgb(c),
            StyleClass::BorderColor(c) => s.border = rgb(c),
            StyleClass::RingColor(c) => s.ring = rgb(c),
            _ => {}
        }
    }
    s
}

fn surfaces(comp: &DynamicComponent) -> Vec<Surface> {
    let (view, _id_map, _probe) = comp.view_with_debug();
    let mut styles = Vec::new();
    collect_styles(&view, &mut styles);
    styles.iter().map(|s| surface_of(s)).collect()
}

fn find_surface(comp: &DynamicComponent, want: Surface) -> usize {
    let all = surfaces(comp);
    all.iter()
        .position(|s| *s == want)
        .unwrap_or_else(|| panic!("surface {want:?} not found in {all:?}"))
}

fn all_views<'a>(view: &'a View<DynamicMessage>, out: &mut Vec<&'a View<DynamicMessage>>) {
    out.push(view);
    match view {
        View::Column {
            children: kids, ..
        }
        | View::Row {
            children: kids, ..
        } => {
            for k in kids {
                all_views(k, out);
            }
        }
        View::Button {
            content: Some(c), ..
        } => all_views(c, out),
        View::Container { child, .. } | View::Scrollable { child, .. } => all_views(child, out),
        _ => {}
    }
}

/// V-05 / AC-04：panel 是被绘制的结构容器——class/style 不丢、design 局部值
/// 生效且父→子继承、子覆盖、兄弟与字面量互不串。
#[test]
fn panel_surface_carries_local_design_and_inherits() {
    let dir = tempfile::tempdir().unwrap();
    let comp = build_component(dir.path());

    // A：四类槽全部来自本元素 design（radius 槽引用 10dp；颜色槽解析为
    // 主题值——scope 快照决定，不读进程级旧槽）。
    let a = Surface {
        radius: Some(10.0),
        bg: Some((17, 34, 51)),
        fg: Some((238, 238, 238)),
        border: Some((51, 68, 85)),
        ring: Some((255, 0, 255)),
    };
    find_surface(&comp, a);

    // C：父面板 radius 继承 + 自己只覆盖 background。
    let c = Surface {
        radius: Some(10.0),
        bg: Some((17, 34, 51)),
        ..Default::default()
    };
    find_surface(&comp, c);

    // E：字面量固定 6px/#123456（不随槽）。
    find_surface(
        &comp,
        Surface {
            radius: Some(6.0),
            bg: Some((18, 52, 86)),
            ..Default::default()
        },
    );

    // B：无 design 的兄弟面板不得携带 A 的局部四类槽——A 的完整表面只出现
    // 一次（它自己）；兄弟只在共享同一 token 时才有继承（B 没有 design）。
    let all = surfaces(&comp);
    let count_a = all.iter().filter(|s| **s == a).count();
    assert_eq!(
        count_a, 1,
        "带四类槽的面板只应来自 A 自己的 design，兄弟不得继承：{all:?}"
    );
    assert!(
        !all.iter().any(|s| s.radius == Some(10.0)
            && s.bg.is_none()
            && s.fg.is_some()),
        "A 的 foreground/border/ring 不得泄漏到只声明 radius 的父面板：{all:?}"
    );
}

/// V-05 / AC-04/05：Input/Textarea 的边/环/radius 来自同一 scope 快照；
/// 错误态用语义 destructive（不是硬编码红）。
#[test]
fn input_and_textarea_states_use_scope_semantics() {
    let dir = tempfile::tempdir().unwrap();
    let comp = build_component(dir.path());
    let (view, _id_map, _probe) = comp.view_with_debug();
    let mut views = Vec::new();
    all_views(&view, &mut views);

    let mut input_checked = false;
    let mut textarea_checked = false;
    for v in &views {
        match v {
            View::Input {
                style,
                disabled,
                on_change,
                on_submit,
                ..
            } => {
                // PLAN-031 F-02（二次修复）：`disabled: .blocked`（true）必须进
                // View，且**构造期**就不挂 on_change/on_submit——判据不是渲染层
                // 某个臂是否额外加了门（VM 帧渲染曾漏掉），而是 View 里没有
                // handler 可派发。
                assert!(
                    *disabled,
                    "input 的 disabled 必须由模型驱动进入 View（F-02）"
                );
                assert!(
                    on_change.is_none() && on_submit.is_none(),
                    "禁用 input 不得携带 on_change/on_submit（F-02 不派发）"
                );
                let s = style.as_ref().expect("input carries style");
                assert!(
                    s.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::BorderColor(Color::Error))),
                    "input 错误态必须是语义 destructive → Color::Error，实际 {:?}",
                    s.classes
                );
                assert!(
                    s.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::RoundedMd)),
                    "input 的 rounded-md 必须保留，实际 {:?}",
                    s.classes
                );
                // PLAN-031 T-05：本元素 design 的 placeholder 槽 → 样式类，
                // 并经 IcedStyle 落到 text_input 的 placeholder 通道。
                assert!(
                    s.classes.iter().any(|c| matches!(
                        c,
                        StyleClass::PlaceholderColor(Color::Rgba {
                            r: 255,
                            g: 0,
                            b: 0,
                            ..
                        })
                    )),
                    "input 的 design.placeholder 必须产出 PlaceholderColor，实际 {:?}",
                    s.classes
                );
                let iced =
                    auto_ui_runtime::ui::style::IcedStyle::from_style(s);
                assert!(
                    iced.placeholder_color.is_some(),
                    "IcedStyle 必须消费 placeholder 颜色"
                );
                input_checked = true;
            }
            View::Textarea {
                style,
                disabled,
                on_change,
                on_submit,
                keydown,
                ..
            } => {
                let s = style.as_ref().expect("textarea carries style");
                // PLAN-031 F-02（二次修复）：textarea 同 input——禁用即无 handler。
                assert!(*disabled, "textarea 的 disabled 必须进入 View（F-02）");
                assert!(
                    on_change.is_none() && on_submit.is_none(),
                    "禁用 textarea 不得携带 on_change/on_submit（F-02 不派发）"
                );
                // N-12/N-23：onkeydown.* 绑定在禁用态也不得上车（Binding::Custom
                // 会在无 on_change 时照样派发）。
                assert!(
                    keydown.is_empty(),
                    "禁用 textarea 不得携带 onkeydown 绑定，实际 {:?}",
                    keydown.keys().collect::<Vec<_>>()
                );
                assert!(
                    s.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::BorderColor(Color::Border))),
                    "textarea 边色走主题 border 语义，实际 {:?}",
                    s.classes
                );
                assert!(
                    s.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::RoundedMd)),
                    "textarea 的 rounded-md 必须保留，实际 {:?}",
                    s.classes
                );
                textarea_checked = true;
            }
            _ => {}
        }
    }
    assert!(input_checked, "input 未在构建树中");
    assert!(textarea_checked, "textarea 未在构建树中");
}

/// V-05：button 的 disabled 由模型驱动（置 disabled 即不派发），hover 变体
/// 与语义底色同时保留——禁用不靠删 handler 实现。
#[test]
fn button_disabled_state_and_hover_variant() {
    let dir = tempfile::tempdir().unwrap();
    let mut comp = build_component(dir.path());

    let observe_button = |comp: &DynamicComponent| -> (bool, bool, bool) {
        let (view, _id_map, _probe) = comp.view_with_debug();
        let mut views = Vec::new();
        all_views(&view, &mut views);
        let btn = views
            .iter()
            .find_map(|v| match v {
                View::Button {
                    disabled,
                    style,
                    onclick,
                    ..
                } => Some((*disabled, style.clone(), onclick.clone())),
                _ => None,
            })
            .expect("button present");
        let (disabled, style, _onclick) = btn;
        let style = style.expect("button carries style");
        let has_hover = style.has_variant(Variant::Hover);
        let has_primary = style
            .classes
            .iter()
            .any(|c| matches!(c, StyleClass::BackgroundColor(Color::Primary)));
        (disabled, has_hover, has_primary)
    };

    // blocked = true（初始）→ disabled，但 hover/语义底色与 handler 仍在。
    let (disabled, hover, primary) = observe_button(&comp);
    assert!(disabled, "disabled-if 为真时按钮必须 disabled");
    assert!(hover, "hover 变体不得因 disabled 丢失");
    assert!(primary, "语义底色 bg-primary 不得因 disabled 丢失");

    // blocked = false → 可点，状态切换不改变表面类。
    comp.write_state("blocked", Value::Bool(false)).unwrap();
    let (disabled, hover, primary) = observe_button(&comp);
    assert!(!disabled, "disabled-if 为假时按钮必须可点");
    assert!(hover && primary, "状态翻转不改变 hover/底色类");
}

/// V-05：主题切换后同一 panel 设计槽重解析到新值（快照驱动，无旧值叠加）。
#[test]
fn design_slots_follow_theme_switch() {
    let dir = tempfile::tempdir().unwrap();
    let mut comp = build_component(dir.path());
    comp.view_with_debug();
    // F-07(b)：必须**真正切换**主题再断言槽值变化（此前只是同源码建两次，
    // 只证明确定性，不能证槽引用随快照更新）。
    let before = surfaces(&comp);
    assert!(
        before.iter().any(|s| s.radius == Some(10.0) && s.bg == Some((17, 34, 51))),
        "初始应为 Surf：radius 10dp / card #112233，实际 {before:?}"
    );

    comp.write_state("theme_name", Value::Str("Surf2".into()))
        .unwrap();
    comp.view_with_debug();
    let after = surfaces(&comp);
    assert!(
        after.iter().any(|s| s.radius == Some(12.0)),
        "切到 Surf2 后 radius 槽应变成 12dp，实际 {after:?}"
    );
    assert!(
        after.iter().any(|s| s.bg == Some((68, 85, 102))),
        "切到 Surf2 后 card 槽应变成 #445566，实际 {after:?}"
    );
    assert!(
        !after.iter().any(|s| s.bg == Some((17, 34, 51))),
        "旧族 card 值不得残留，实际 {after:?}"
    );
}
