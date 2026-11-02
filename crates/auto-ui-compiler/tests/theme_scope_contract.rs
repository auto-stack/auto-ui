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

/// T-03：两个 App scope 的目录/选择/revision 互不覆盖；scoped 发布不写
/// legacy 全局槽（单应用旧宿主保持原语义）；同一 scope 内同名内容变化
/// 仍然按内容身份发布。
#[test]
fn scope_catalog_selection_and_revision_are_isolated() {
    theme::set_dark_mode(false);
    let _ = theme::set_theme("scaffold");
    let legacy_name = theme::theme_name();
    assert_eq!(vm_background(), Some((255, 255, 255)), "legacy scaffold light");

    let a = theme::ThemeScope::capture();
    let b = theme::ThemeScope::capture();

    let mut ma = BTreeMap::new();
    ma.insert("Plan031Scope".to_string(), tree("Plan031Scope", "#111111"));
    a.register_trees(ma);
    let mut mb = BTreeMap::new();
    mb.insert("Plan031Scope".to_string(), tree("Plan031Scope", "#222222"));
    b.register_trees(mb);

    {
        let _guard = theme::install_theme_scope(a.clone());
        let looked = theme::lookup_theme_tree("Plan031Scope").expect("A 目录内有树");
        assert!(theme::set_theme_tree(looked));
        assert_eq!(theme::theme_name(), "Plan031Scope");
        assert_eq!(vm_background(), Some((17, 17, 17)));
        assert_eq!(theme::theme_epoch(), 1, "A 自己的 revision");
    }
    {
        let _guard = theme::install_theme_scope(b.clone());
        let looked = theme::lookup_theme_tree("Plan031Scope").expect("B 目录内有同名的另一棵树");
        assert!(theme::set_theme_tree(looked));
        assert_eq!(vm_background(), Some((34, 34, 34)));
        assert_eq!(theme::theme_epoch(), 1, "B 的 revision 从自身起算");
    }
    // A 仍读自己的选择。
    {
        let _guard = theme::install_theme_scope(a.clone());
        assert_eq!(theme::theme_name(), "Plan031Scope");
        assert_eq!(vm_background(), Some((17, 17, 17)));
        assert_eq!(theme::theme_epoch(), 1);
    }
    // 同名内容变化：同一 scope 内按内容身份发布并推进 revision。
    {
        let mut ma2 = BTreeMap::new();
        ma2.insert("Plan031Scope".to_string(), tree("Plan031Scope", "#333333"));
        a.register_trees(ma2);
        let _guard = theme::install_theme_scope(a.clone());
        let looked = theme::lookup_theme_tree("Plan031Scope").unwrap();
        assert!(theme::set_theme_tree(looked));
        assert_eq!(vm_background(), Some((51, 51, 51)));
        assert_eq!(theme::theme_epoch(), 2);
    }
    // legacy 全局槽完全未被 scoped 发布改写。
    assert_eq!(theme::theme_name(), legacy_name);
    assert_eq!(vm_background(), Some((255, 255, 255)));
}

/// T-03：scope 安装严格入栈/退出恢复（嵌套构建不串，drop 后回上一层，
/// 全部退出后回 legacy）。
#[test]
fn scope_install_is_nested_and_restores_on_drop() {
    theme::set_dark_mode(false);
    let _ = theme::set_theme("scaffold");

    let a = theme::ThemeScope::capture();
    let b = theme::ThemeScope::capture();
    let mut ma = BTreeMap::new();
    ma.insert("Plan031Nest".to_string(), tree("Plan031Nest", "#101010"));
    a.register_trees(ma);
    let mut mb = BTreeMap::new();
    mb.insert("Plan031Nest".to_string(), tree("Plan031Nest", "#202020"));
    b.register_trees(mb);

    {
        let _outer = theme::install_theme_scope(a.clone());
        let looked = theme::lookup_theme_tree("Plan031Nest").unwrap();
        assert!(theme::set_theme_tree(looked));
        assert_eq!(vm_background(), Some((16, 16, 16)));

        {
            let _inner = theme::install_theme_scope(b.clone());
            assert_eq!(theme::theme_name(), "scaffold", "B 尚未发布自己的选择");
            let looked_b = theme::lookup_theme_tree("Plan031Nest").unwrap();
            assert!(theme::set_theme_tree(looked_b));
            assert_eq!(vm_background(), Some((32, 32, 32)), "内层 scope 生效");
        }

        // 内层 guard drop → 外层 A 的快照恢复（嵌套构建不串）。
        assert_eq!(theme::theme_name(), "Plan031Nest");
        assert_eq!(vm_background(), Some((16, 16, 16)));
    }

    // 全部退出 → legacy 槽位（scaffold/light）。
    assert_eq!(theme::theme_name(), "scaffold");
    assert_eq!(vm_background(), Some((255, 255, 255)));
}

/// T-03：宿主/受控输入优先级（App override → follow 宿主 → App 默认）、
/// Unset/Invalid 分离、非法高优先来源不回落；preview 子 scope 独立身份与
/// 发布隔离。
#[test]
fn host_input_priority_and_preview_scope_isolation() {
    use theme::{HostResolution, HostThemeInputs, ModeInput, ThemeInput};
    theme::set_dark_mode(false);
    let a = theme::ThemeScope::capture();

    assert_eq!(a.resolve_host_inputs(), HostResolution::None, "未设置 = 无高优先输入");

    a.set_host_inputs(HostThemeInputs {
        follow_host: false,
        host_family: ThemeInput::Value("Ignored".into()),
        ..Default::default()
    });
    assert_eq!(
        a.resolve_host_inputs(),
        HostResolution::None,
        "未声明 follow 的 App 不吃宿主 family"
    );

    a.set_host_inputs(HostThemeInputs {
        follow_host: true,
        host_family: ThemeInput::Value("Followed".into()),
        host_mode: ModeInput::Dark,
        ..Default::default()
    });
    assert_eq!(
        a.resolve_host_inputs(),
        HostResolution::Candidate {
            family: Some("Followed".into()),
            mode: Some(ModeInput::Dark)
        }
    );

    a.set_host_inputs(HostThemeInputs {
        app_override: ThemeInput::Value("Override".into()),
        follow_host: true,
        host_family: ThemeInput::Value("Followed".into()),
        ..Default::default()
    });
    assert_eq!(
        a.resolve_host_inputs(),
        HostResolution::Candidate {
            family: Some("Override".into()),
            mode: None
        },
        "App override 优先于宿主 follow"
    );

    a.set_host_inputs(HostThemeInputs {
        app_override: ThemeInput::Invalid("bad source".into()),
        follow_host: true,
        host_family: ThemeInput::Value("Followed".into()),
        ..Default::default()
    });
    match a.resolve_host_inputs() {
        HostResolution::Blocked(msg) => assert!(msg.contains("bad source"), "{msg}"),
        other => panic!("非法高优先来源必须 Blocked（不回落），得到 {other:?}"),
    }

    // preview 子 scope：独立身份、继承父选择、发布隔离。
    let b = theme::ThemeScope::preview_child(&a);
    assert_ne!(a.id(), b.id(), "preview 必须有独立身份");
    assert_eq!(b.family_name(), a.family_name(), "preview 继承父选择");
    let mut ma = BTreeMap::new();
    ma.insert("Plan031Preview".to_string(), tree("Plan031Preview", "#010101"));
    b.register_trees(ma);
    {
        let _guard = theme::install_theme_scope(b.clone());
        let looked = theme::lookup_theme_tree("Plan031Preview").unwrap();
        assert!(theme::set_theme_tree(looked));
        assert_eq!(theme::theme_name(), "Plan031Preview");
    }
    assert_eq!(a.family_name(), "scaffold", "子发布不回流父");
    assert_eq!(a.revision(), 0, "父 revision 不变");
    assert_eq!(b.revision(), 1, "preview 自己 +1");
}
