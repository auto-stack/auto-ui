fn main() {
    let name = std::env::args().nth(1).unwrap_or("player".into());
    let src = std::fs::read_to_string(format!("fixtures/minimal/{name}.at")).expect("fixture");
    let mut parser = auto_lang_core::parser::Parser::from(&src).with_session(auto_parser_api::CompilerSession {
        scenario: auto_parser_api::Scenario::UI,
        ..Default::default()
    });
    let code = parser.parse_domain(&auto_ui_dialect::node2d::Node2DDialect).expect("parse");
    for item in code.items {
        if let auto_parser_api::DomainStmt::Domain(auto_ui_dialect::node2d::Node2DStmt::Node(node)) = item {
            let on = node.on.expect("on");
            let want = std::env::args().nth(2);
            for h in &on.handlers {
                let dn = h.pattern.display_name();
                if want.as_deref().map(|w| dn.contains(w)).unwrap_or(true) {
                    println!("=== {dn} ===");
                    println!("{:#?}", h.body.stmts);
                }
            }
        }
    }
}
