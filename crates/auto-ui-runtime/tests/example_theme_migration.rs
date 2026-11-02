//! PLAN-031 T-06（work）——第二批示例接入官方主题包（003/010）的 VM 侧切面。
//!
//! 真实仓库源码（examples/ui/003-converter、010-contact-form）→
//! `build_dynamic_component`（含 pac `dep themes` 包注册链）→ design 绑定发布
//! → 语义色按 scope 解析 → 业务状态读回。
//!
//! 本文件锁定的是"源码迁移后仍然成立"的合同：
//! - 两 App 独立默认 Auto/light（010 保留其显式 dark）；
//! - 换族/换模式经 design 绑定 + 同一 scope 快照生效；
//! - 003 两向换算字段、010 无校验/Submit→submitted/FAQ 业务不被主题改写；
//! - 010 回执走包内 `success` 语义槽，不再硬编码 emerald 调色板值。
//!
//! 真实双端交互（MCP 输入/点击、Vue Playwright）归 T-06 的运行腿与 T-07。

use std::path::PathBuf;

use auto_ui_compiler::style::class::StyleClass;
use auto_ui_compiler::style::{theme, Color};
use auto_ui_runtime::ui::dynamic::DynamicComponent;
use auto_ui_runtime::ui::interpreter::DynamicMessage;
use auto_ui_runtime::ui::view::View;
use auto_val::Value;

fn example_app(rel: &str) -> String {
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
    let path = example_app(rel);
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

fn collect_text<'a>(view: &'a View<DynamicMessage>, out: &mut Vec<&'a str>) {
    match view {
        View::Text { content, .. } => out.push(content.as_str()),
        View::Column {
            children: kids, ..
        }
        | View::Row {
            children: kids, ..
        } => {
            for k in kids {
                collect_text(k, out);
            }
        }
        View::Button {
            content: Some(c), ..
        } => collect_text(c, out),
        View::Container { child, .. } | View::Scrollable { child, .. } => collect_text(child, out),
        _ => {}
    }
}

fn text_of(comp: &DynamicComponent) -> String {
    let (view, _id, _probe) = comp.view_with_debug();
    let mut parts = Vec::new();
    collect_text(&view, &mut parts);
    parts.join("\n")
}

fn input_values(comp: &DynamicComponent) -> Vec<String> {
    fn walk<'a>(view: &'a View<DynamicMessage>, out: &mut Vec<String>) {
        match view {
            View::Input { value, .. } => out.push(value.clone()),
            View::Column {
                children: kids, ..
            }
            | View::Row {
                children: kids, ..
            } => {
                for k in kids {
                    walk(k, out);
                }
            }
            View::Button {
                content: Some(c), ..
            } => walk(c, out),
            View::Container { child, .. } | View::Scrollable { child, .. } => walk(child, out),
            _ => {}
        }
    }
    let (view, _id, _probe) = comp.view_with_debug();
    let mut out = Vec::new();
    walk(&view, &mut out);
    out
}

fn collect_classes<'a>(view: &'a View<DynamicMessage>, out: &mut Vec<&'a StyleClass>) {
    fn styles_of<'a>(
        view: &'a View<DynamicMessage>,
        out: &mut Vec<&'a auto_ui_compiler::style::Style>,
    ) {
        match view {
            View::Text { style, .. } => {
                if let Some(s) = style {
                    out.push(s);
                }
            }
            View::Column {
                style,
                children: kids,
                ..
            }
            | View::Row {
                style,
                children: kids,
                ..
            } => {
                if let Some(s) = style {
                    out.push(s);
                }
                for k in kids {
                    styles_of(k, out);
                }
            }
            View::Button {
                style,
                content,
                ..
            } => {
                if let Some(s) = style {
                    out.push(s);
                }
                if let Some(c) = content {
                    styles_of(c, out);
                }
            }
            View::Container { style, child, .. } | View::Scrollable { style, child, .. } => {
                if let Some(s) = style {
                    out.push(s);
                }
                styles_of(child, out);
            }
            _ => {}
        }
    }
    let mut styles = Vec::new();
    styles_of(view, &mut styles);
    for s in styles {
        for c in &s.classes {
            out.push(c);
        }
    }
}

/// 003：官方包 + design 默认 Auto/light；两向换算字段与输入值随状态；
/// 换族/换模式只改解析值，不改业务字段。
#[test]
fn converter_uses_official_package_without_touching_business() {
    let mut comp = build("003-converter");
    assert_eq!(comp.widget_name(), "App");
    // design 绑定在首帧求值（与真实启动路径一致）
    comp.view_with_debug();
    assert_eq!(family(&comp), "Auto", "003 独立默认 Auto");
    assert!(!is_dark(&comp), "003 独立默认 light");

    // 官方 Auto light 值（包内单源，不是 App 内复制色表）
    assert_eq!(rgb(&comp, &Color::Background), Some((255, 255, 255)));
    assert_eq!(rgb(&comp, &Color::Surface), Some((255, 255, 255)));
    assert_eq!(rgb(&comp, &Color::Border), Some((228, 228, 231)));
    assert_eq!(rgb(&comp, &Color::OnBackground), Some((9, 9, 11)));

    // 业务字段仍在（两向换算的输入输出状态）
    let as_f64 = |v: Value| match v {
        Value::Double(d) => d,
        Value::Float(f) => f as f64,
        Value::Int(i) => i as f64,
        other => panic!("非数值：{other:?}"),
    };
    let c = as_f64(comp.read_state("celsius").expect("celsius 字段"));
    let f = as_f64(comp.read_state("fahrenheit").expect("fahrenheit 字段"));
    assert_eq!(c, 0.0);
    assert_eq!(f, 32.0);

    // 输入值经状态绑定渲染（C=100 的换算结果由 handler 产出；此处锁定绑定面）
    comp.write_state("celsius", Value::Double(100.0)).unwrap();
    comp.write_state("fahrenheit", Value::Double(212.0)).unwrap();
    let values = input_values(&comp);
    assert!(
        values.iter().any(|v| v.starts_with("100")),
        "摄氏输入应显示 100，实际 {values:?}"
    );
    assert!(
        values.iter().any(|v| v.starts_with("212")),
        "华氏输入应显示 212，实际 {values:?}"
    );

    // 切 dark：同一 scope 重新解析（Auto dark background #09090b）
    comp.write_state("dark_mode", Value::Bool(true)).unwrap();
    comp.view_with_debug();
    assert!(is_dark(&comp));
    assert_eq!(rgb(&comp, &Color::Background), Some((9, 9, 11)));
    assert_eq!(rgb(&comp, &Color::Surface), Some((15, 15, 18)));
    // accent 驱动（同一模式内）换族不改 primary
    let primary_dark = rgb(&comp, &Color::Primary);

    // 换族 Solarized dark：基色切换，业务字段不受影响
    comp.write_state("theme_name", Value::Str("Solarized".into()))
        .unwrap();
    comp.view_with_debug();
    assert_eq!(family(&comp), "Solarized");
    assert_eq!(rgb(&comp, &Color::Background), Some((0, 43, 54)));
    assert_eq!(
        rgb(&comp, &Color::Primary),
        primary_dark,
        "同一模式内换族不改 accent 驱动的 primary 语义"
    );
    assert_eq!(
        as_f64(comp.read_state("celsius").unwrap()),
        100.0,
        "换族不得改业务字段"
    );
}

/// 010：显式 dark 默认保留；空提交仍置 submitted 并显示回执；回执走
/// `success` 语义槽；FAQ 与无校验业务不变。
#[test]
fn contact_form_keeps_business_and_uses_success_semantics() {
    let mut comp = build("010-contact-form");
    assert_eq!(comp.widget_name(), "App");
    comp.view_with_debug();
    assert_eq!(family(&comp), "Auto");
    assert!(is_dark(&comp), "010 的显式 pac dark 默认保留");
    assert_eq!(rgb(&comp, &Color::Background), Some((9, 9, 11)));

    // 初始无回执
    assert!(
        !text_of(&comp).contains("Thank you"),
        "初始不应显示提交回执"
    );

    // 空提交：submitted 翻转即显示回执（不新增必填/邮箱校验）
    comp.write_state("submitted", Value::Bool(true)).unwrap();
    let text = text_of(&comp);
    assert!(
        text.contains("Thank you! Your message has been sent."),
        "空提交也必须显示回执，实际文本：{text}"
    );

    // 回执表面使用 success 语义（不是硬编码 emerald 调色板值）
    let (view, _id, _probe) = comp.view_with_debug();
    let mut classes = Vec::new();
    collect_classes(&view, &mut classes);
    assert!(
        classes
            .iter()
            .any(|c| matches!(c, StyleClass::TextColor(Color::Success))),
        "回执文字必须走 success 语义，实际类：{:?}",
        classes
            .iter()
            .filter(|c| matches!(c, StyleClass::TextColor(_)))
            .collect::<Vec<_>>()
    );
    assert!(
        !classes.iter().any(|c| matches!(
            c,
            StyleClass::BackgroundColor(Color::Rgba {
                r: 16,
                g: 185,
                b: 129,
                ..
            })
        )),
        "不得残留硬编码 emerald-500 底色"
    );

    // 填写三字段：值进入输入面（业务值不因换肤丢失）
    comp.write_state("name", Value::Str("Alice Smith".into()))
        .unwrap();
    comp.write_state("email", Value::Str("alice@example.com".into()))
        .unwrap();
    comp.write_state(
        "message",
        Value::Str("enterprise support options".into()),
    )
    .unwrap();
    let values = input_values(&comp);
    assert!(values.iter().any(|v| v == "Alice Smith"), "{values:?}");
    assert!(values.iter().any(|v| v == "alice@example.com"), "{values:?}");

    // 换模式/换族：回执与业务值保持，仅解析值变化
    comp.write_state("dark_mode", Value::Bool(false)).unwrap();
    comp.view_with_debug();
    assert_eq!(rgb(&comp, &Color::Background), Some((255, 255, 255)));
    let light_text = text_of(&comp);
    assert!(light_text.contains("Thank you"), "换模式不得丢回执");

    comp.write_state("theme_name", Value::Str("Solarized".into()))
        .unwrap();
    comp.view_with_debug();
    assert_eq!(rgb(&comp, &Color::Background), Some((253, 246, 227)));
    let values = input_values(&comp);
    assert!(
        values.iter().any(|v| v == "Alice Smith"),
        "换族后业务值保持：{values:?}"
    );
    // FAQ 仍为 4 条
    assert_eq!(comp.read_state_as_vec("faqs").map(|v| v.len()), Ok(4));
}
