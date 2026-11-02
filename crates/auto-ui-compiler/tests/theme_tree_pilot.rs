//! PLAN-007 T-01 bounded probe (seed of the T-04 pilot suite).
//!
//! Checks which pieces of the theme-tree pilot surface already exist in the
//! real pipeline: top-level `pub let` data decls, object literals, dotted
//! module member access, and what statement families they lower to.

use auto_ui_compiler::ui_ast::parse_ui;

#[test]
fn probe_top_level_pub_let_object() {
    let src = r##"
pub let Auto = {
    name: "Auto"
    tokens: {
        colors: { background: "#ffffff" }
        radius: { control: "8dp" }
    }
    modes: {
        light: { tokens: { colors: { background: "#ffffff" } } }
        dark: { tokens: { colors: { background: "#09090b" } } }
    }
}

widget App {
    view { col { text "hi" {} } }
}
"##;
    match parse_ui(src) {
        Ok(ui) => {
            let count = ui.domain_stmts().count();
            println!("probe_top_level_pub_let_object: parsed, domain stmts = {count}");
        }
        Err(e) => {
            // T-01 finding (r4): top-level `pub let` data decls are NOT part of
            // the current surface — the pilot will introduce the UI-domain
            // `pub theme` decl in T-04. Pin the current error shape so the
            // gap is visible and its closure is a deliberate flip.
            let msg = format!("{e}");
            assert!(
                msg.contains("Let") || msg.contains("let"),
                "expected the `let` rejection diagnostic, got: {msg}"
            );
            println!("probe_top_level_pub_let_object: currently rejected as expected — {msg}");
        }
    }
}

#[test]
fn probe_core_object_let_and_field_access() {
    // Pure Core: object literal in let + field access, through the real parse.
    let src = r#"
fn main() {
    let t = { name: "Auto", radius: { control: "8dp" } }
    print(t.radius.control)
}
"#;
    use auto_lang_core::parser::Parser;
    use auto_lang_core::session::CompilerSession;
    let mut parser = Parser::from(src).with_session(CompilerSession::ui());
    match parser.parse() {
        Ok(code) => println!("probe_core_object: parsed, stmts = {}", code.stmts.len()),
        Err(e) => panic!("core object literal + dot access must parse: {e}"),
    }
}
