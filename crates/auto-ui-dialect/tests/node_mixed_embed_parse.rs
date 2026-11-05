//! PLAN-049 T-03（mixed-ui-2d-3d 解析面）：`Node2DMixedDialect` 放行基础
//! UI `view` 元素树内的 `view-2d`/`view-3d` **嵌入区域**（挂载关系，非
//! node 级双出口）；默认 dialect 的 048 互斥拒绝字节不变；node 级
//! view+view-2d/view-3d 共存在任何 dialect 下具名拒绝（039 有限 marker
//! 档身份不扩大）。
//! Run: cargo test -p auto-ui-dialect --test node_mixed_embed_parse
use auto_lang_core::parser::Parser;
use auto_parser_api::{CompilerSession, DomainDialect, DomainStmt, Scenario};
use auto_ui_dialect::node2d::{Node2DDialect, Node2DMixedDialect, Node2DStmt};

fn parse_with<D: DomainDialect<Output = Node2DStmt>>(
    src: &str,
    dialect: &D,
) -> Result<Vec<Node2DStmt>, String> {
    let mut parser = Parser::from(src).with_session(CompilerSession {
        scenario: Scenario::UI,
        ..Default::default()
    });
    match parser.parse_domain(dialect) {
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

fn parse_mixed(src: &str) -> Result<Vec<Node2DStmt>, String> {
    parse_with(src, &Node2DMixedDialect)
}

fn parse_default(src: &str) -> Result<Vec<Node2DStmt>, String> {
    parse_with(src, &Node2DDialect)
}

const MIXED_OK: &str = r#"
node Main {
    model {
        var count = 0
        var reg2x = 0
        var reg3z = 0
        var clicks2 = 0
        var clicks3 = 0
    }
    view {
        column root_col {
            text label { text: "count:" }
            button inc { text: "inc", onclick: .Inc }
            row regions {
                view-2d reg2 {
                    width: 160
                    height: 90
                    onclick: .Region2
                    shape2d mover { kind: quad, x: .reg2x, y: 20, w: 24, h: 24 }
                }
                view-3d reg3 {
                    width: 160
                    height: 90
                    onclick: .Region3
                    cylinder3d spin { translation: (0, 0, 0) }
                }
            }
        }
    }
    on {
        .Inc -> {
            .count = .count + 1
            .reg2x = .reg2x + 8
            .reg3z = .reg3z + 2
        }
        .Region2 -> {
            .clicks2 = .clicks2 + 1
        }
        .Region3 -> {
            .clicks3 = .clicks3 + 1
        }
    }
}
fn main() {}
"#;

#[test]
fn mixed_dialect_parses_ui_embed_regions_with_props_and_scene_elements() {
    let nodes = parse_mixed(MIXED_OK).expect("mixed dialect parses embed regions");
    assert_eq!(nodes.len(), 1);
    let auto_ui_dialect::node2d::Node2DStmt::Node(main) = &nodes[0];
    let view = main.view.as_ref().expect("basic UI view present");
    assert_eq!(view.elements.len(), 1, "single column root");
    let auto_ui_dialect::node2d::UiElement::Container { name, children, .. } = &view.elements[0]
    else {
        panic!("root is a column container");
    };
    assert_eq!(name, "root_col");
    assert_eq!(children.len(), 3, "text + button + regions row");

    let auto_ui_dialect::node2d::UiElement::Container {
        children: regions, ..
    } = &children[2]
    else {
        panic!("third child is the regions row");
    };
    assert_eq!(regions.len(), 2, "one embed region per dimension");

    let auto_ui_dialect::node2d::UiElement::Embed2d {
        name,
        props,
        elements,
        ..
    } = &regions[0]
    else {
        panic!("first region is view-2d");
    };
    assert_eq!(name, "reg2");
    let prop_names: Vec<_> = props.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(prop_names, vec!["width", "height", "onclick"]);
    assert_eq!(elements.len(), 1, "one scene element");
    assert!(matches!(
        &elements[0],
        auto_ui_dialect::node2d::View2dElement::Shape { name, .. } if name == "mover"
    ));

    let auto_ui_dialect::node2d::UiElement::Embed3d {
        name,
        props,
        elements,
        ..
    } = &regions[1]
    else {
        panic!("second region is view-3d");
    };
    assert_eq!(name, "reg3");
    assert_eq!(props.len(), 3, "width/height/onclick");
    assert!(matches!(
        &elements[0],
        auto_ui_dialect::node2d::View3dElement::Primitive { name, .. } if name == "spin"
    ));
}

#[test]
fn default_dialect_still_rejects_embed_regions_in_view() {
    // 默认（node-ui）dialect 下 view 内出现 view-2d/view-3d 嵌入头不获
    // 识别（is_ui_element_head 默认不认），与 048 基线同样落在具名语法
    // 错误（`view` 头后随 Sub 而非 Colon）；node 级 view+view-2d 共存由
    // 具名互斥守卫拒绝（见 mixed_dialect_still_rejects_node_level_*）。
    let err = parse_default(MIXED_OK).expect_err("default dialect must reject embed regions");
    assert!(
        err.contains("Colon") || err.contains("view") || err.contains("Unexpected"),
        "default dialect rejects with its 048-baseline parse error: {err}"
    );
}

#[test]
fn mixed_dialect_still_rejects_node_level_view_with_view2d() {
    // 嵌入是 view 元素树内的挂载关系；node 级 `view`+`view-2d` 共存守卫
    // 在混合 dialect 下同样具名拒绝（不打开 039 的 node 级双出口档）。
    let src = r#"
node Main {
    model {
        var count = 0
    }
    view {
        text t { text: "x" }
    }
    view-2d {
        shape2d s { kind: quad, x: 0, y: 0, w: 8, h: 8 }
    }
}
fn main() {}
"#;
    let err = parse_mixed(src).expect_err("node-level view+view-2d must stay rejected");
    assert!(
        err.contains("together") || err.contains("view"),
        "rejection names the coexistence rule: {err}"
    );
}

#[test]
fn mixed_dialect_still_rejects_node_level_dual_view2d_view3d() {
    // allow_dual_view 保持 false：node 级 view-2d+view-3d 双出口仅 039
    // 显式查询 profile 放行，混合 profile 不扩大。
    let src = r#"
node Main {
    model {
        var count = 0
    }
    view-2d {
        shape2d s { kind: quad, x: 0, y: 0, w: 8, h: 8 }
    }
    view-3d {
        cylinder3d c { translation: (0, 0, 0) }
    }
}
fn main() {}
"#;
    let err = parse_mixed(src).expect_err("node-level dual 2D+3D exits stay rejected");
    assert!(
        err.contains("view") || err.contains("dual") || err.contains("both"),
        "rejection names the dual-view rule: {err}"
    );
}
