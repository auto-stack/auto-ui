//! PLAN-031 T-02（work）——R-01「同名新内容未发布」真实调用族回归。
//!
//! 链路与生产一致：`pub theme` 源码 → `build_theme` / `register_theme_trees`
//! → `lookup_theme_tree`（builder 发布前置）→ `set_theme_tree`（`set_active`）
//! → VM RGB 投影 / CSS 变量投影 / `THEME_EPOCH` 失效代数。
//!
//! 修复前红点（PLAN-031 §4.2 R-01）：`set_active` 只比较 `name()`，同名但
//! token 内容不同的树被短路为「无变化」——树不替换、epoch 不自增，消费端
//! 继续读旧值。本文件同时钉住合法控制：同名同内容不发布、不同来源不算同一
//! 身份、内容变化必须恰好 +1 revision。

use std::collections::BTreeMap;

use auto_ui_compiler::design_tokens::theme_tree::{
    build_theme, compose_mode, css_var_pairs, ThemeTree,
};
use auto_ui_compiler::style::theme;
use auto_ui_compiler::style::Color;
use auto_ui_compiler::ui_ast::parse_ui;
use auto_ui_dialect::UiStmt;

/// 真实源码声明 → build_theme（与 `build_dynamic_component` 的注册链同源）。
fn tree(name: &str, background: &str) -> ThemeTree {
    let src = format!(
        "pub theme {name} = {{\n  tokens: {{\n    colors: {{\n      background: \"{background}\"\n      foreground: \"#101010\"\n    }}\n  }}\n  modes: {{\n    light: {{\n      tokens: {{ colors: {{ background: \"{background}\" }} }}\n    }}\n    dark: {{\n      tokens: {{ colors: {{ background: \"{background}\" }} }}\n    }}\n  }}\n}}\n"
    );
    let ui = parse_ui(&src).expect("主题源码可解析");
    let decl = ui
        .domain_stmts()
        .find_map(|stmt| match stmt {
            UiStmt::ThemeDataDecl(d) if d.name.as_str() == name => Some(d),
            _ => None,
        })
        .expect("ThemeDataDecl");
    build_theme(name, &decl.tree, &BTreeMap::new()).expect("主题树 build 通过 schema 校验")
}

fn vm_background() -> Option<(u8, u8, u8)> {
    theme::resolve_semantic_rgb(&Color::Background)
}

/// R-01 主反例：同名内容变化必须实际发布并把 revision/epoch 推进恰好一次；
/// 同名同内容（合法控制）不发布也不增 epoch。
#[test]
fn r01_same_name_content_change_publishes_and_bumps_epoch_once() {
    theme::set_dark_mode(false);
    let _ = theme::set_theme("scaffold");
    let e0 = theme::theme_epoch();

    let a = tree("Plan031Same", "#010101");
    let b = tree("Plan031Same", "#020202");
    assert_eq!(a.name, b.name, "前置：两树同名");
    assert_ne!(a.raw, b.raw, "前置：两树内容不同");

    // 首帧/首次发布：新名 → 替换（与旧实现无关的合法控制）。
    assert!(theme::set_theme_tree(a.clone()));
    assert_eq!(theme::theme_epoch(), e0.wrapping_add(1));
    assert_eq!(vm_background(), Some((1, 1, 1)), "VM 投影读 A 内容");

    // R-01 差额：同名不同内容必须真实替换并失效重绘。
    assert!(
        theme::set_theme_tree(b.clone()),
        "同名不同内容必须发布（旧实现按 name() 短路返回 false）"
    );
    assert_eq!(
        theme::theme_epoch(),
        e0.wrapping_add(2),
        "内容变化必须推进一次 revision（epoch +1）"
    );
    assert_eq!(vm_background(), Some((2, 2, 2)), "VM 投影必须读新内容");

    // 合法控制：同名同内容重复发布不增 revision（无变化不错误替换）。
    assert!(!theme::set_theme_tree(b.clone()));
    assert_eq!(theme::theme_epoch(), e0.wrapping_add(2));
    assert_eq!(vm_background(), Some((2, 2, 2)));

    // A→B→A 往返：同名交替不滞留旧内容。
    assert!(theme::set_theme_tree(a.clone()));
    assert_eq!(vm_background(), Some((1, 1, 1)));
    assert_eq!(theme::theme_epoch(), e0.wrapping_add(3));
}

/// R-01 双投影一致性：注册表取到的新内容与 CSS 变量投影、VM RGB 投影同源；
/// 页面/组件消费的 active spec 必须跟上同名更新。
#[test]
fn r01_registry_and_css_projection_follow_same_name_update() {
    theme::set_dark_mode(false);

    let a = tree("Plan031Css", "#0a0a0a");
    let b = tree("Plan031Css", "#0b0b0b");

    let mut first = BTreeMap::new();
    first.insert("Plan031Css".to_string(), a);
    theme::register_theme_trees(first);
    let looked = theme::lookup_theme_tree("Plan031Css").expect("A 已注册");
    assert!(theme::set_theme_tree(looked));
    assert_eq!(vm_background(), Some((10, 10, 10)));

    let mut second = BTreeMap::new();
    second.insert("Plan031Css".to_string(), b);
    theme::register_theme_trees(second);
    let looked2 = theme::lookup_theme_tree("Plan031Css").expect("B 覆盖注册");
    let css = css_var_pairs(&compose_mode(&looked2, false).expect("light 合成"));
    assert!(
        css.contains(&("background".to_string(), "#0b0b0b".to_string())),
        "CSS 投影必须读注册后的新内容: {css:?}"
    );
    assert!(theme::set_theme_tree(looked2), "同名新内容必须发布");
    assert_eq!(
        vm_background(),
        Some((11, 11, 11)),
        "VM 消费族与 CSS 投影同源（不能一边新一边旧）"
    );
}

/// 身份不是「名字」：同名但来源形态不同（树 vs 内置族）也必须发布。
#[test]
fn r01_same_name_different_source_kind_is_not_identity() {
    theme::set_dark_mode(false);
    let t = tree("zinc", "#0c0c0c");
    assert!(theme::set_theme_tree(t));
    assert_eq!(vm_background(), Some((12, 12, 12)));

    // 内置 zinc 与树同名但不同来源：必须切换（否则旧树把内置族挡在门外）。
    assert!(theme::set_theme("zinc"), "同名不同来源必须发布");
    assert_ne!(
        vm_background(),
        Some((12, 12, 12)),
        "切到内置族后不得继续读旧树内容"
    );
}
