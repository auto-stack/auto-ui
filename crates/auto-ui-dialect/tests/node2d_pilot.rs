//! PLAN-016 T-03 (V-01): the explicit Node2D research dialect parses the
//! real minimal pilot project (multi-file node/view-2d/Life/anonymous
//! msg/leading-dot surface) and rejects malformed research sources with
//! positioned diagnostics. The production UiDialect path stays opt-out:
//! `node` never parses through it.
//! Run: cargo test -p auto-ui-dialect --test node2d_pilot
use auto_lang_core::parser::Parser;
use auto_parser_api::{CompilerSession, DomainStmt, Scenario};
use auto_ui_dialect::node2d::View2dElement as Node2DElement;
use auto_ui_dialect::node2d::{Node2DDialect, Node2DStmt};
use auto_ui_dialect::UiDialect;

fn fixture_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experimental/autographics-node2d-pilot/fixtures/minimal")
}

fn parse_node2d(src: &str) -> Result<Vec<Node2DStmt>, String> {
    let mut parser = Parser::from(src).with_session(CompilerSession {
        scenario: Scenario::UI,
        ..Default::default()
    });
    match parser.parse_domain(&Node2DDialect) {
        Ok(code) => {
            let mut nodes = Vec::new();
            for item in code.items {
                if let DomainStmt::Domain(stmt) = item {
                    nodes.push(stmt);
                }
            }
            Ok(nodes)
        }
        Err(e) => Err(format!("{e:?}")),
    }
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_dir().join(name)).expect("fixture readable")
}

#[test]
fn minimal_project_modules_parse_with_expected_surface() {
    // The player/hazard/coin/main modules carry every frozen surface piece:
    // node params, model state, anonymous msg, view-2d with solid/trigger/
    // camera/instances/if, Life.Start/Life.Tick, .Member handlers, emit,
    // refs/paths and leading-dot members.
    let mut total_nodes = 0;
    for name in ["player.at", "hazard.at", "coin.at", "main.at"] {
        let src = read_fixture(name);
        let nodes = parse_node2d(&src)
            .unwrap_or_else(|e| panic!("{name} must parse through the research entry: {e}"));
        assert_eq!(nodes.len(), 1, "{name}: exactly one node");
        total_nodes += 1;
    }
    assert_eq!(total_nodes, 4);

    // Pure Core modules (types/patrol/resources) parse with zero domain
    // statements through the same entry.
    for name in ["types.at", "patrol.at", "resources.at"] {
        let src = read_fixture(name);
        let nodes = parse_node2d(&src).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(nodes.is_empty(), "{name}: no node declarations expected");
    }
}

#[test]
fn main_node_structure_is_complete() {
    let src = read_fixture("main.at");
    let nodes = parse_node2d(&src).expect("main parses");
    let Node2DStmt::Node(main) = &nodes[0] else {
        panic!("node expected")
    };
    assert_eq!(main.name, "Main");
    let model = main.model.as_ref().expect("model block");
    let names: Vec<_> = model.fields.iter().map(|f| f.name.to_string()).collect();
    assert_eq!(names, vec!["coin_present", "starts", "ticks", "phase"]);
    let msg = main.msg.as_ref().expect("anonymous msg");
    assert_eq!(msg.variants.len(), 1);
    assert_eq!(msg.variants[0].name.to_string(), "Collect");

    let view = main.view2d.as_ref().expect("view-2d block");
    // ground + player + h1 + h2 + if(coin) + camera = 6 elements.
    assert_eq!(view.elements.len(), 6, "main view-2d elements");
    assert!(matches!(
        &view.elements[0],
        Node2DElement::Solid { name, .. } if name == "ground"
    ));
    assert!(matches!(
        &view.elements[1],
        Node2DElement::Instance { node, name, .. } if node == "Player" && name == "player"
    ));
    assert!(matches!(
        &view.elements[2],
        Node2DElement::Instance { node, name, .. } if node == "Hazard" && name == "h1"
    ));
    assert!(matches!(
        &view.elements[3],
        Node2DElement::Instance { node, name, .. } if node == "Hazard" && name == "h2"
    ));
    assert!(matches!(&view.elements[4], Node2DElement::If { .. }));
    assert!(matches!(&view.elements[5], Node2DElement::Camera { name, .. } if name == "camera"));

    let on = main.on.as_ref().expect("on block");
    let patterns: Vec<String> = on
        .handlers
        .iter()
        .map(|h| h.pattern.display_name())
        .collect();
    assert_eq!(patterns, vec!["Life.Start", "Life.Tick", "Collect"]);
}

#[test]
fn coin_handler_and_instance_bindings_parse() {
    let src = read_fixture("coin.at");
    let nodes = parse_node2d(&src).expect("coin parses");
    let Node2DStmt::Node(coin) = &nodes[0] else {
        panic!("node expected")
    };
    // Constructor params: position floats, typed actor ref, msg port.
    let params: Vec<_> = coin.params.iter().map(|(n, _)| n.clone()).collect();
    assert_eq!(
        params,
        vec!["position_x", "position_y", "target", "on_collect"]
    );
    // Anonymous business msg with a typed payload.
    let msg = coin.msg.as_ref().expect("msg block");
    assert_eq!(msg.variants[0].name.to_string(), "Touched");
    assert_eq!(msg.variants[0].payload.len(), 1);

    let on = coin.on.as_ref().expect("on block");
    assert_eq!(on.handlers.len(), 1);
    match &on.handlers[0].pattern {
        auto_ui_dialect::node2d::EventPattern::Member { name, params } => {
            assert_eq!(name, "Touched");
            assert_eq!(params.len(), 1);
        }
        other => panic!("member pattern expected, got {other:?}"),
    }
    // The handler body carries the guard chain and the emit statement.
    assert!(
        !on.handlers[0].body.stmts.is_empty(),
        "Touched handler body must not be empty"
    );
}

#[test]
fn hazard_view2d_uses_leading_dot_members_and_call() {
    let src = read_fixture("hazard.at");
    let nodes = parse_node2d(&src).expect("hazard parses");
    let Node2DStmt::Node(hazard) = &nodes[0] else {
        panic!("node expected")
    };
    let view = hazard.view2d.as_ref().expect("view-2d");
    let Node2DElement::Trigger { props, .. } = &view.elements[0] else {
        panic!("trigger2d expected");
    };
    let position = props
        .iter()
        .find(|p| p.name == "position")
        .expect("position");
    // position: (.x, patrol_y(.base_y, .phase, 0.4)) — a literal pair whose
    // second member is a real call of the imported pure function.
    let exprs = position.value.exprs();
    assert_eq!(exprs.len(), 2);
    assert!(matches!(exprs[1], auto_parser_api::ast::Expr::Call(_)));

    let on = hazard.on.as_ref().expect("on block");
    assert_eq!(on.handlers.len(), 2);
    assert_eq!(on.handlers[0].pattern.display_name(), "Life.Start");
    match &on.handlers[1].pattern {
        auto_ui_dialect::node2d::EventPattern::Life { event, params } => {
            assert_eq!(event, "Tick");
            assert_eq!(params.len(), 1);
        }
        other => panic!("Life.Tick expected, got {other:?}"),
    }
}

#[test]
fn malformed_research_sources_reject_with_positioned_errors() {
    // Unknown node block keyword.
    let err = parse_node2d("node Bad {\n model { var a = 1 }\n widget W {}\n}\n")
        .expect_err("unknown block must reject");
    assert!(err.contains("unsupported node block 'widget'"), "{err}");

    // Duplicate model block.
    let err = parse_node2d("node Bad {\n model { var a = 1 }\n model { var b = 2 }\n}\n")
        .expect_err("duplicate model must reject");
    assert!(err.contains("duplicate model block"), "{err}");

    // on pattern neither Life.* nor leading-dot.
    let err = parse_node2d("node Bad {\n on { Clicked -> {} }\n}\n")
        .expect_err("bare pattern must reject");
    assert!(err.contains("Life.X' or '.X'"), "{err}");

    // Node name must be UpperCamelCase.
    let err = parse_node2d("node bad {\n}\n").expect_err("lowercase node name must reject");
    assert!(err.contains("UpperCamelCase"), "{err}");

    // Unknown view-2d element kind.
    let err = parse_node2d("node Bad {\n view-2d { blob b { size: 1.0 } }\n}\n")
        .expect_err("unknown element must reject");
    assert!(err.contains("unknown view-2d element 'blob'"), "{err}");

    // The conditional instance element is legal inside view-2d.
    parse_node2d(
        "node Ok {\n view-2d { if .alive == 1 { solid2d s { position: (0.0, 0.0) } } }\n}\n",
    )
    .expect("conditional element must parse");
}

#[test]
fn production_ui_dialect_stays_opt_out_for_research_surface() {
    // Core has pre-existing native `node` parsing (Expr::Node); the isolation
    // guarantee is that the RESEARCH surface is not reachable through the
    // production dialect: `node` is not a UiDialect dispatch keyword, and the
    // view-2d domain marker has no production arm.
    use auto_parser_api::DomainDialect;
    assert!(!UiDialect.keywords().contains(&"node"));
    let mut parser = Parser::from("widget T {\n view-2d { }\n}\n").with_session(CompilerSession {
        scenario: Scenario::UI,
        ..Default::default()
    });
    let result = parser.parse_domain(&UiDialect);
    assert!(
        result.is_err(),
        "view-2d must not parse through the production UiDialect"
    );
}
