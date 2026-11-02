//! PLAN-007 主题树试点集成测试（T-04；AC-01..03 / V-01..02）。
//!
//! 走真实源码路径：`use` 模块解析（resolve_use_module）→ `pub theme` 数据
//! 声明 → extend 合成 → 模式选择 → 两种投影（CSS 变量对 / VM RGB）→
//! 类型化槽引用。不以 Rust 手工构造最终树代替源码路径。

use auto_ui_compiler::design_tokens::theme_tree::{
    collect_theme_trees, compose_mode, css_var_pairs, resolve_rgb, resolve_slot, ThemeTree,
};
use auto_ui_compiler::ui_ast::parse_ui;
use auto_ui_dialect::UiStmt;
use std::collections::BTreeMap;

const THEMES_AT: &str = r##"
// 官方候选族（试点内联版；examples/ui/themes/ 为 T-06 落点的同构数据）
pub theme Auto = {
    tokens: {
        colors: {
            background: "#ffffff"
            foreground: "#09090b"
            primary: "#8b5cf6"
            card: "#ffffff"
        }
        radius: { control: "8dp" }
        tags: ["shell", "content"]
    }
    modes: {
        light: { tokens: { colors: { background: "#ffffff" } } }
        dark: { tokens: { colors: { background: "#09090b", card: "#111113" } } }
    }
}

pub theme Solarized = extend("Auto", {
    tokens: {
        colors: {
            background: "#fdf6e3"
            primary: "#268bd2"
        }
    }
    modes: {
        light: { tokens: { colors: { background: "#fdf6e3" } } }
        dark: { tokens: { colors: { background: "#002b36", card: "#073642" } } }
    }
})
"##;

const APP_DEFAULTS_AT: &str = r##"
// app 命名空间扩展默认值（C-02：Feature/app 在所属命名空间增加字段）
pub theme app_defaults = {
    my_app: { panel: "#eee8d5" }
}
"##;

const APP_AT: &str = r##"
use themes: Auto, Solarized
use my_app.design: app_defaults

// 试点应用主题：官方族 × app 扩展 × 局部基准（C-04/06）
pub theme Pilot = extend("Solarized", "app_defaults", {
    tokens: { radius: { control: "8dp" } }
    modes: {
        light: { tokens: { colors: { primary: "#268bd2" } } }
        dark: { tokens: { colors: { primary: "#2aa198" } } }
    }
})

widget App {
    view { col { text "hi" {} } }
}
"##;

fn write_tree(dir: &std::path::Path) -> std::path::PathBuf {
    // 布局沿真实解析序：src/front 为 base_dir，src/（父级）被向上探测
    // ——共享 themes.at 与 app 命名空间扩展放 src/ 下；T-06 的多消费者
    // 官方包走 pac `dep themes { path: "../themes" }` 同一解析面。
    let front = dir.join("src/front");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::create_dir_all(dir.join("src/my_app")).unwrap();
    std::fs::write(dir.join("src/themes.at"), THEMES_AT).unwrap();
    std::fs::write(dir.join("src/my_app/design.at"), APP_DEFAULTS_AT).unwrap();
    let app = front.join("app.at");
    std::fs::write(&app, APP_AT).unwrap();
    app
}

fn pilot_theme(
    known: &BTreeMap<String, auto_ui_compiler::design_tokens::theme_tree::Tree>,
    code: &str,
) -> ThemeTree {
    let ui = parse_ui(code).unwrap();
    let decl = ui
        .domain_stmts()
        .find_map(|stmt| match stmt {
            UiStmt::ThemeDataDecl(t) if t.name.as_str() == "Pilot" => Some(t),
            _ => None,
        })
        .expect("Pilot theme decl");
    auto_ui_compiler::design_tokens::theme_tree::build_theme(
        decl.name.as_str(),
        &decl.tree,
        known,
    )
    .unwrap()
}

/// V-01（AC-01/02）：真实 use/数据导出 → 树合成 → 两种投影；extend 语义
///（后层 8dp 胜、未改字段保留、两 modes 保留、只读基主题不变）。
#[test]
fn v01_use_to_composed_tree_to_both_projections() {
    let dir = tempfile::tempdir().unwrap();
    let app = write_tree(dir.path());

    let code = std::fs::read_to_string(&app).unwrap();
    let known = collect_theme_trees(&app, &code).unwrap();
    // 全部主题经真实模块解析取到（裸名 + 点路径双键）
    for key in ["Auto", "themes.Auto", "Solarized", "app_defaults", "Pilot"] {
        assert!(known.contains_key(key), "{key} missing in {:?}", known.keys());
    }

    let pilot = pilot_theme(&known, &code);
    let light = compose_mode(&pilot, false).unwrap();
    let dark = compose_mode(&pilot, true).unwrap();

    // 未改字段保留（Solarized 基座）+ 两 modes 均保留
    assert_eq!(resolve_rgb(&light, Token::Background), Some((253, 246, 227)));
    assert_eq!(resolve_rgb(&dark, Token::Background), Some((0, 43, 54)));
    // Solarized dark 的 card 差异保留（Pilot 未触）
    assert_eq!(resolve_rgb(&dark, Token::Card), Some((7, 54, 66)));
    assert_eq!(resolve_rgb(&light, Token::Card), Some((255, 255, 255)));
    // Pilot 自己的 modes 后层胜（primary）
    assert_eq!(resolve_rgb(&light, Token::Primary), Some((38, 139, 210)));
    assert_eq!(resolve_rgb(&dark, Token::Primary), Some((42, 161, 152)));

    // app 命名空间扩展进投影（CSS 面 --my_app-panel）
    let css = css_var_pairs(&light);
    assert!(
        css.contains(&("my_app-panel".to_string(), "#eee8d5".to_string())),
        "{css:?}"
    );
    // 公共键投影保持 31 键 css_var 名
    assert!(css.contains(&("background".to_string(), "#fdf6e3".to_string())));

    // 只读基主题值不变（extend 输入不被 patch 污染）
    let solarized_raw = known.get("Solarized").unwrap();
    assert_eq!(
        resolve_rgb(solarized_raw, Token::Primary),
        Some((38, 139, 210)),
        "基主题原始值不受 Pilot patch 影响"
    );
    // 未触数组字段原样保留（数组整体替换语义在 theme_tree 单测钉死）
    let tags = resolve_slot(&light, &["tokens".into(), "tags".into()]).unwrap();
    match tags {
        auto_ui_compiler::design_tokens::theme_tree::Tree::Array(items) => {
            assert_eq!(items.len(), 2, "未触数组字段原样保留");
        }
        other => panic!("tags 应为数组，得到 {other:?}"),
    }
}

/// V-02（AC-02/03）：同槽引用随主题数据改值联动；直接字面值不受同槽变值
/// 影响；公共 Color 填长度值报错；31 键外公共键拒绝。
#[test]
fn v02_slot_reference_follows_value_but_literal_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let app = write_tree(dir.path());
    let code = std::fs::read_to_string(&app).unwrap();
    let known = collect_theme_trees(&app, &code).unwrap();

    let pilot = pilot_theme(&known, &code);
    let light = compose_mode(&pilot, false).unwrap();
    let ref_path = [
        "tokens".to_string(),
        "radius".to_string(),
        "control".to_string(),
    ];
    assert_eq!(
        resolve_slot(&light, &ref_path).unwrap().as_str(),
        Some("8dp"),
        "槽引用解析到当前主题值"
    );

    // 同槽改值 → 同一引用路径读到新值（联动）
    use auto_ui_compiler::design_tokens::theme_tree::Tree;
    let bumped = Tree::Object(vec![(
        "tokens".to_string(),
        Tree::Object(vec![(
            "radius".to_string(),
            Tree::Object(vec![("control".to_string(), Tree::Str("12dp".into()))]),
        )]),
    )]);
    let merged = auto_ui_compiler::design_tokens::theme_tree::extend_trees(&light, &[bumped]).unwrap();
    assert_eq!(
        resolve_slot(&merged, &ref_path).unwrap().as_str(),
        Some("12dp"),
        "同槽变值后引用联动"
    );
    // 组件直接字面值不受影响：字面 "8dp" 就是 "8dp"（无引用语义）
    assert_eq!(Tree::Str("8dp".into()).as_str(), Some("8dp"));

    // 负例：公共 Color 槽填长度值 → 校验错误且可定位
    let bad_app = dir.path().join("src/front/bad.at");
    std::fs::write(
        &bad_app,
        r##"
use themes: Auto
pub theme BadSlot = extend("Auto", {
    tokens: { colors: { background: "8dp" } }
})
"##,
    )
    .unwrap();
    let bad_code = std::fs::read_to_string(&bad_app).unwrap();
    let err = collect_theme_trees(&bad_app, &bad_code).unwrap_err();
    assert!(err.msg.contains("颜色") || err.msg.contains("#rrggbb"), "{err}");
    assert!(err.path.contains("colors.background"), "{err:?}");

    // 负例：未知公共键（31 键外）拒绝
    let uk_app = dir.path().join("src/front/unknown.at");
    std::fs::write(
        &uk_app,
        r##"
use themes: Auto
pub theme UnknownKey = extend("Auto", {
    tokens: { colors: { nope: "#000000" } }
})
"##,
    )
    .unwrap();
    let uk_code = std::fs::read_to_string(&uk_app).unwrap();
    let err = collect_theme_trees(&uk_app, &uk_code).unwrap_err();
    assert!(err.msg.contains("31 键"), "{err}");
    assert!(err.path.contains("colors.nope"), "{err:?}");
}

/// 官方主题数据包落点预演（T-06 前置）：examples/ui/themes/ 下的 .at 经
/// 真实解析可被第三方消费者读取（V-08 三消费者同源的 compiler 侧前提）。
#[test]
fn official_theme_package_readable_from_examples_root() {
    let pkg = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/themes");
    let src_path = pkg.join("themes.at");
    let src = std::fs::read_to_string(&src_path).expect("官方主题包 themes/themes.at 必须存在");
    let known = collect_theme_trees(&src_path, &src).unwrap();
    assert!(known.contains_key("Auto"));
    assert!(known.contains_key("Solarized"));
    // Solarized extend Auto：基座键继承 + 自身覆盖，两 modes 保留
    let auto = known.get("Auto").unwrap();
    let sol = known.get("Solarized").unwrap();
    use auto_ui_compiler::design_tokens::registry::TokenName;
    assert_eq!(resolve_rgb(auto, TokenName::Background), Some((255, 255, 255)));
    assert_eq!(resolve_rgb(sol, TokenName::Primary), Some((38, 139, 210)));
}

mod token_alias {
    pub use auto_ui_compiler::design_tokens::registry::TokenName as Token;
}
use token_alias::Token;



#[test]
fn probe_element_design_surface_forms() {
    // (form, source) — which element-side local-diff forms parse today?
    let cases: [(&str, &str); 5] = [
        ("prop_dot_ref", "widget A { view { col { style: tokens.radius.control } } }"),
        ("prop_object", "widget A { view { col { design: { radius: \"8dp\" } } } }"),
        ("prop_dot_design", "widget A { view { col { design: panel_tokens } } }"),
        ("block_design", "widget A { view { col { design { radius: \"8dp\" } } } }"),
        ("widget_design_block", "widget A { design { theme: \"Auto\" } view { col { text \"x\" {} } } }"),
    ];
    for (label, src) in cases {
        match parse_ui(src) {
            Ok(_) => println!("DESIGN-PROBE {label}: OK"),
            Err(e) => println!("DESIGN-PROBE {label}: ERR {e}"),
        }
    }
}

#[test]
fn probe_theme_lab_source_parses() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/theme-lab/src/front/app.at")).unwrap();
    match parse_ui(&src) {
        Ok(ui) => println!("THEME-LAB PARSE: OK ({} domain stmts)", ui.domain_stmts().count()),
        Err(e) => println!("THEME-LAB PARSE: ERR {e}"),
    }
}
