//! PLAN-031 T-03（work）——Vue 容器作用域主题投影（生成器合同）。
//!
//! 真实管线（源码 → Parser → extract → VueGenerator）生成 SFC，断言：
//! - 带主题/强调色运行时的 widget，模板根挂 `ref="__autoThemeRoot"`，
//!   `applyTheme`/`applyAccent` 接收该容器，CSS vars 只写容器子树；
//! - 运行时的缺省仍是 `document.documentElement`（legacy 单应用兼容）；
//! - 无主题/强调色的 widget 零注入（零变化边界）；
//! - store composable 路径无组件根 ref，保持 document bootstrap（具名边界，
//!   待 T-06 真实 003/010 Vue 运行侧复核）。

use auto_lang_core::parser::Parser;
use auto_parser_api::session::CompilerSession;
use auto_ui_compiler::aura::extract_widget_from_decl;
use auto_ui_compiler::ui_gen::{BackendGenerator, VueGenerator};
use auto_ui_dialect::UiStmt as Stmt;

fn gen_sfc(src: &str) -> String {
    let session = CompilerSession::ui();
    let mut parser = Parser::from(src).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .expect("widget source must parse");
    let decl = ast
        .domain_stmts()
        .find_map(|s| match s {
            Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = extract_widget_from_decl(decl).expect("extract widget");
    VueGenerator::new().generate(&widget).expect("generate SFC")
}

fn gen_store(src: &str) -> String {
    let session = CompilerSession::ui();
    let mut parser = Parser::from(src).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .expect("store source must parse");
    let decl = ast
        .domain_stmts()
        .find_map(|s| match s {
            Stmt::StoreDecl(d) => Some(d),
            _ => None,
        })
        .expect("store decl");
    let store = auto_ui_compiler::aura::extract_store_from_decl(decl).expect("extract store");
    VueGenerator::generate_store_composable(&store)
}

/// 主题 widget：根容器 ref + 容器化 applyTheme，document 仍作缺省回退。
#[test]
fn theme_widget_projects_vars_to_container_root() {
    let sfc = gen_sfc(
        r#"
widget App {
    model {
        var theme_name str = "Auto"
        var dark_mode bool = false
    }
    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }
    view {
        col { text "hi" {} }
    }
}
"#,
    );

    assert!(
        sfc.contains("ref=\"__autoThemeRoot\""),
        "template root must carry the container ref:\n{sfc}"
    );
    assert!(
        sfc.contains("const __autoThemeRoot = ref<HTMLElement | null>(null)"),
        "script must declare the container ref:\n{sfc}"
    );
    let vue_import = sfc
        .lines()
        .find(|l| l.contains("from 'vue'"))
        .expect("vue import line");
    assert!(
        vue_import.contains("ref"),
        "container ref needs the `ref` import: {vue_import}"
    );
    // onMounted: 容器优先、document 缺省回退。
    assert!(
        sfc.contains("const __rootEl = __autoThemeRoot.value"),
        "onMounted must read the container ref:\n{sfc}"
    );
    assert!(
        sfc.contains("if (__th !== 'scaffold') applyTheme(__th, __isDark, getSavedAccent(), __rootEl)"),
        "theme restore must target the container:\n{sfc}"
    );
    // 模式翻转 watch 同步容器（PLAN-031 F-1：自动应用点统一经 `__thName()`，
    // 声明族优先；容器的 dark 面由 applyTheme 内的 rootEl 分支写入）。
    assert!(
        sfc.contains("applyTheme(__thName(), v, getSavedAccent(), __autoThemeRoot.value)"),
        "mode-flip watch must re-apply to the container:\n{sfc}"
    );
    assert!(
        sfc.contains("const __thName = (): string => theme_name.value || getActiveTheme()"),
        "declared family must win on automatic application points:\n{sfc}"
    );
    // theme_name watch 同步容器。
    assert!(
        sfc.contains("applyTheme(n, (__autoThemeRoot.value ?? document.documentElement)"),
        "theme_name watch must re-apply to the container:\n{sfc}"
    );
    // 运行时本身：容器优先、document 回退。
    assert!(
        sfc.contains("const root = rootEl ?? document.documentElement"),
        "applyTheme must fall back to documentElement without a container:\n{sfc}"
    );
    assert!(
        sfc.contains("const scope: ParentNode = rootEl ?? document"),
        "enclosing-scope queries must not cross the container boundary:\n{sfc}"
    );
    assert!(
        sfc.contains("if (accentName) applyAccent(accentName, isDark, rootEl)"),
        "accent overlay must follow the same container:\n{sfc}"
    );
}

/// 仅强调色的 widget 同样容器化（accent 是主题变量的同一投影面）。
#[test]
fn accent_widget_projects_vars_to_container_root() {
    let sfc = gen_sfc(
        r#"
widget Acc {
    model { var accent_color str = "indigo" }
    view { col { text "a" {} } }
}
"#,
    );
    assert!(sfc.contains("ref=\"__autoThemeRoot\""), "{sfc}");
    assert!(
        sfc.contains("applyAccent(saved, __isDark, __rootEl)"),
        "accent restore must target the container:\n{sfc}"
    );
    assert!(
        sfc.contains("const root = rootEl ?? document.documentElement"),
        "{sfc}"
    );
}

/// 无主题/强调色/widget dark 的普通 widget：零注入、根无 ref（零变化）。
#[test]
fn plain_widget_keeps_legacy_output_without_container_ref() {
    let sfc = gen_sfc(
        r#"
widget Plain {
    model { var count int = 0 }
    view { col { button "inc" onclick: .inc } }
    on { .inc -> { count += 1 } }
}
"#,
    );
    assert!(!sfc.contains("__autoThemeRoot"), "{sfc}");
    assert!(!sfc.contains("THEME_PALETTES"), "{sfc}");
    assert!(!sfc.contains("function applyTheme"), "{sfc}");
}

/// store composable 路径无组件根 ref：保持 document bootstrap（具名边界）。
#[test]
fn store_composable_keeps_document_bootstrap_boundary() {
    let code = gen_store(
        r#"
store Shell {
    model {
        var dark_mode bool = false
        var accent_color str = "indigo"
    }
    msg Msg { ToggleDark }
    on { .ToggleDark -> { dark_mode = !dark_mode } }
}
"#,
    );
    assert!(
        code.contains("applyTheme(__th, document.documentElement.classList.contains('dark'))"),
        "store bootstrap keeps document-level vars (no component root):\n{code}"
    );
    assert!(!code.contains("__autoThemeRoot"), "{code}");
}
