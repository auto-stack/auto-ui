use auto_lang_core::parser::Parser;
use auto_parser_api::CompilerSession;
use auto_ui_dialect::UiDialect;

fn parse(source: &str) -> Result<String, String> {
    let mut parser = Parser::from(source).with_session(CompilerSession::ui());
    parser
        .parse_domain(&UiDialect)
        .map(|ast| format!("{ast:?}"))
        .map_err(|err| format!("{err:?}"))
}

#[test]
fn nested_views_and_else_if_chains_fit_default_worker_stack() {
    std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
        let nested = format!("widget App {{ view {{ {}text \"deep-leaf\"{} }} }}", "col { ".repeat(64), " }".repeat(64));
        assert!(parse(&nested).unwrap().contains("deep-leaf"));
        let mut chain = "widget App { view { if true { text \"first\" }".to_string();
        for _ in 0..128 { chain.push_str(" else if false { text \"middle\" }"); }
        chain.push_str(" else { text \"last\" } } }");
        let tree = parse(&chain).unwrap();
        assert!(tree.contains("first") && tree.contains("last"));
        assert_eq!(tree.matches("Conditional {").count(), 129);
        let mixed = "widget App { model { var items []int = [] } view { col { for item in .items { if true { memo (deps: item) { link (to: \"/ok\") { col { text item } } } } else { text \"empty\" } } } } }";
        let tree = parse(mixed).unwrap();
        for kind in ["ForLoop", "Conditional", "MemoBlock", "Link"] { assert!(tree.contains(kind), "missing {kind}"); }
    }).unwrap().join().unwrap();
}

#[test]
fn iterative_view_parser_preserves_props_events_and_rejects_missing_braces() {
    let tree = parse("widget App { view { col { style: \"a\" } { button \"OK\" { onclick: .Click } text \"tail\" } } on { .Click -> {} } }").unwrap();
    for value in ["onclick", "Click", "tail", "style"] {
        assert!(tree.contains(value), "{tree}");
    }
    assert!(parse("widget App { view { col { text \"unfinished\"").is_err());
    assert!(parse("widget App { view { if true { text \"x\" } else if false {").is_err());
}
