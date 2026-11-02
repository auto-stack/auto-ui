//! PLAN-031 T-03（work）——V-04 局部 design 作用域：父→子继承、子覆盖、
//! 兄弟/实例隔离、literal 不随槽变、非法槽不破坏父级有效子树。
//!
//! 走真实源码 → `build_dynamic_component` → `view_with_debug`（tracked 转换
//! 路径）→ View 结构类读回（RoundedT/BackgroundColor）。

use auto_ui_compiler::style::class::{RoundedSize, StyleClass};
use auto_ui_compiler::style::Color;
use auto_ui_runtime::ui::dynamic::DynamicComponent;
use auto_ui_runtime::ui::interpreter::DynamicMessage;
use auto_ui_runtime::ui::style::Style;
use auto_ui_runtime::ui::view::View;
use auto_val::Value;

const APP: &str = r##"
pub theme IsoR = {
    tokens: {
        colors: {
            background: "#101010"
            foreground: "#eeeeee"
            primary: "#8888ff"
            card: "#112233"
        }
        radius: { control: "8dp" }
    }
    modes: {
        light: {
            tokens: { colors: { background: "#101010" } }
        }
        dark: {
            tokens: { colors: { background: "#010101" } }
        }
    }
}

pub theme IsoR2 = {
    tokens: {
        colors: {
            background: "#202020"
            foreground: "#dddddd"
            primary: "#8888ff"
            card: "#445566"
        }
        radius: { control: "12dp" }
    }
    modes: {
        light: {
            tokens: { colors: { background: "#202020" } }
        }
        dark: {
            tokens: { colors: { background: "#020202" } }
        }
    }
}

widget App {
    model {
        var theme_name str = "IsoR"
        var dark_mode bool = false
    }

    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }

    view {
        col {
            container {
                design: { radius: tokens.radius.control }
                col {
                    container {
                        design: { background: tokens.colors.card }
                        text "b" {}
                    }
                    container { text "c" {} }
                    container {
                        design: { nope: "x" }
                        text "f" {}
                    }
                }
            }
            container {
                design: { radius: "4dp" }
                text "d" {}
            }
            container {
                design: { radius: tokens.radius.control, background: tokens.colors.card }
                text "e" {}
            }
            container {
                design: { background: "#123456" }
                text "g" {}
            }
        }
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
            content: Some(content),
            ..
        } => {
            if let Some(style) = style {
                out.push(style);
            }
            collect_styles(content, out);
        }
        View::Button { style, .. } => {
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

fn radius_px(style: &Style) -> Option<f32> {
    style.classes.iter().find_map(|class| match class {
        StyleClass::RoundedT(Some(RoundedSize::Px(v))) => Some(*v),
        _ => None,
    })
}

fn bg_rgb(style: &Style) -> Option<(u8, u8, u8)> {
    style.classes.iter().find_map(|class| match class {
        StyleClass::BackgroundColor(Color::Rgba { r, g, b, .. }) => Some((*r, *g, *b)),
        _ => None,
    })
}

struct Observed {
    radii: Vec<f32>,
    backgrounds: Vec<(u8, u8, u8)>,
}

fn observe(comp: &DynamicComponent) -> Observed {
    let (view, _id_map, _probe) = comp.view_with_debug();
    let mut styles = Vec::new();
    collect_styles(&view, &mut styles);
    let mut radii: Vec<f32> = styles.iter().filter_map(|s| radius_px(s)).collect();
    radii.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut backgrounds: Vec<(u8, u8, u8)> =
        styles.iter().filter_map(|s| bg_rgb(s)).collect();
    backgrounds.sort();
    Observed { radii, backgrounds }
}

/// V-04 主腿：A 的 radius 由子 B/F 继承（未覆盖），C 无 design 不被绘制，
/// D 的 literal 固定 4px，E 覆盖两槽；主题切换后槽引用联动、字面量不动。
#[test]
fn design_scope_inherits_overrides_and_detaches_literals() {
    let dir = tempfile::tempdir().unwrap();
    let mut comp = build_component(dir.path());

    let first = observe(&comp);
    assert_eq!(
        first.radii,
        vec![4.0, 8.0, 8.0, 8.0, 8.0],
        "A(8)/B(8 继承)/F(8 继承)/D(4 字面)/E(8)；C 无 design 不产生局部半径"
    );
    assert_eq!(
        first.backgrounds,
        vec![
            (17, 34, 51),
            (17, 34, 51),
            (18, 52, 86)
        ],
        "B/E 用主题 card #112233；G 的 literal #123456 固定"
    );

    // 主题切换：槽引用（radius.control/card）随 scope 主题重解析；
    // D 的 "4dp" 与 G 的 "#123456" 是字面量，不随主题变。
    comp.write_state("theme_name", Value::Str("IsoR2".into()))
        .unwrap();
    let second = observe(&comp);
    assert_eq!(
        second.radii,
        vec![4.0, 12.0, 12.0, 12.0, 12.0],
        "A/B/F/E 的 token 引用跟随新主题 12dp；D 的 literal 保持 4dp"
    );
    assert_eq!(
        second.backgrounds,
        vec![(18, 52, 86), (68, 85, 102), (68, 85, 102)],
        "B/E 的 card 引用跟随 #445566；G literal 保持 #123456"
    );
}
