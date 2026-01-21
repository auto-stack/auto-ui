// Unit tests for Node → View conversion (Phase 2)
//
// Tests the runtime interpretation of AutoLang nodes

use auto_ui::node_converter::{convert_node, ConversionError};
use auto_val::Node;

#[test]
fn test_convert_text_node() {
    let mut node = Node::new("text");
    node.add_pos_arg_unified(auto_val::Value::Str("Hello".into()));

    let result = convert_node(&node);
    assert!(result.is_ok(), "Should successfully convert text node");

    let view = result.unwrap();
    // Verify the view was created (we can't inspect internals easily)
    // Just check it doesn't panic
}

#[test]
fn test_convert_col_node() {
    let mut node = Node::new("col");
    node.add_pos_arg_unified(auto_val::Value::Node({
        let mut child = Node::new("text");
        child.add_pos_arg_unified(auto_val::Value::Str("Hello".into()));
        child
    }));

    let result = convert_node(&node);
    assert!(result.is_ok(), "Should successfully convert col node");
}

#[test]
fn test_convert_button_node() {
    let mut node = Node::new("button");
    node.add_pos_arg_unified(auto_val::Value::Str("Click me".into()));
    node.add_arg_unified("onclick", auto_val::Value::Str("ButtonClick".into()));

    let result = convert_node(&node);
    assert!(result.is_ok(), "Should successfully convert button node");
}

#[test]
fn test_convert_unknown_node() {
    let node = Node::new("unknown_widget");
    let result = convert_node(&node);

    assert!(result.is_err(), "Should fail for unknown node type");
    match result {
        Err(ConversionError::UnknownKind { kind }) => {
            assert_eq!(kind, "unknown_widget");
        }
        _ => panic!("Expected UnknownKind error"),
    }
}

#[test]
fn test_convert_nested_layout() {
    // col { row { text("Hello") } }
    let mut inner_row = Node::new("row");
    inner_row.add_pos_arg_unified(auto_val::Value::Node({
        let mut text = Node::new("text");
        text.add_pos_arg_unified(auto_val::Value::Str("Hello".into()));
        text
    }));

    let mut outer_col = Node::new("col");
    outer_col.add_pos_arg_unified(auto_val::Value::Node(inner_row));

    let result = convert_node(&outer_col);
    assert!(result.is_ok(), "Should successfully convert nested layout");
}

#[test]
fn test_convert_with_style() {
    let mut node = Node::new("container");
    node.add_pos_arg_unified(auto_val::Value::Node({
        let mut text = Node::new("text");
        text.add_pos_arg_unified(auto_val::Value::Str("Styled".into()));
        text
    }));
    node.add_arg_unified("style", auto_val::Value::Str("padding: 16px".into()));

    let result = convert_node(&node);
    assert!(result.is_ok(), "Should successfully convert node with style");
}

#[test]
fn test_convert_all_node_types() {
    let node_types = vec![
        "col", "row", "center", "container", "scrollable",
        "text", "button", "input", "checkbox", "radio", "select", "list", "table",
    ];

    for node_type in node_types {
        let node = Node::new(node_type);
        let result = convert_node(&node);

        // All known node types should at least not crash
        // They may fail for missing required args, but should recognize the type
        match result {
            Err(ConversionError::UnknownKind { .. }) => {
                panic!("Node type '{}' should be recognized", node_type);
            }
            _ => {
                // Ok or other error is fine
            }
        }
    }
}
