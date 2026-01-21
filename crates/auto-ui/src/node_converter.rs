// Node → View Converter for Auto Language Integration
//
// This module converts AutoLang's Value::Node (runtime AST) into AutoUI's View<M>
// to enable runtime interpretation mode without transpilation.

use auto_val::{Value, Node};
use crate::view::View;
use crate::style::Style;

/// Errors that can occur during node conversion
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Unknown node kind (e.g., "unknown_widget")
    UnknownKind { kind: String },
    /// Missing required property
    MissingProp { kind: String, prop: String },
    /// Invalid property value type
    InvalidPropType {
        kind: String,
        prop: String,
        expected: String,
        got: String,
    },
    /// Message type is required for this node (e.g., Button needs onclick)
    MessageRequired { kind: String },
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnknownKind { kind } => {
                write!(f, "Unknown UI node kind: '{}'", kind)
            }
            ConversionError::MissingProp { kind, prop } => {
                write!(f, "Missing required property '{}' for node kind '{}'", prop, kind)
            }
            ConversionError::InvalidPropType { kind, prop, expected, got } => {
                write!(f, "Invalid type for property '{}' on node kind '{}': expected {}, got {}", prop, kind, expected, got)
            }
            ConversionError::MessageRequired { kind } => {
                write!(f, "Message type required for node kind '{}' (runtime interpretation not yet supported)", kind)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

/// Result type for conversion operations
pub type ConversionResult<T> = Result<T, ConversionError>;

/// Convert a Value::Node to View<String> for runtime interpretation
///
/// # Message Type
///
/// Uses `View<String>` where messages are stored as string identifiers
/// (e.g., "button-clicked"). This is a temporary limitation - future versions
/// will support typed message conversion via metadata.
///
/// # Limitations
///
/// - Event handlers (onclick, on_change, etc.) stored as strings, not typed messages
/// - Style strings parsed at runtime (Plan 005 integration)
/// - Type safety limited compared to transpiled Rust code
///
/// # Example
///
/// ```ignore
/// // Auto code: `text("Hello") {}`
/// let node = Node::new("text").with_arg("Hello");
/// let view = convert_node(&node)?;
/// assert!(matches!(view, View::Text { content: "Hello", .. }));
/// ```
pub fn convert_node(node: &Node) -> ConversionResult<View<String>> {
    let kind = node.name.as_str();

    match kind {
        // Layout Components
        "center" => convert_center(node),
        "col" | "column" => convert_column(node),
        "row" => convert_row(node),
        "container" => convert_container(node),
        "scrollable" => convert_scrollable(node),

        // Element Components
        "text" | "label" => convert_text(node),
        "button" => convert_button(node),
        "input" => convert_input(node),
        "checkbox" => convert_checkbox(node),
        "radio" => convert_radio(node),
        "select" => convert_select(node),
        "list" => convert_list(node),
        "table" => convert_table(node),

        // Unknown kind
        _ => Err(ConversionError::UnknownKind {
            kind: kind.to_string(),
        }),
    }
}

// ============================================================================
// Layout Component Converters
// ============================================================================

/// Convert Center node: `center { child }`
fn convert_center(node: &Node) -> ConversionResult<View<String>> {
    // Center is a container with center_x and center_y
    let kids = extract_child_nodes(node)?;

    if kids.is_empty() {
        // Empty center - default to empty view
        return Ok(View::container(View::empty())
            .center()
            .center_x()
            .center_y()
            .build());
    }

    // Take first child as the centered content
    let child = convert_node(&kids[0])?;

    Ok(View::container(child)
        .center()
        .center_x()
        .center_y()
        .build())
}

/// Convert Column node: `col { spacing: 10, padding: 20, kids [...] }`
fn convert_column(node: &Node) -> ConversionResult<View<String>> {
    let spacing = extract_prop_u16(node, "spacing").unwrap_or(0);
    let padding = extract_prop_u16(node, "padding").unwrap_or(0);
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;
    let children: Result<Vec<_>, _> = kids.iter().map(convert_node).collect();
    let children = children?;

    let mut builder = View::col().spacing(spacing).padding(padding);

    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    for child in children {
        builder = builder.child(child);
    }

    Ok(builder.build())
}

/// Convert Row node: `row { spacing: 10, padding: 20, kids [...] }`
fn convert_row(node: &Node) -> ConversionResult<View<String>> {
    let spacing = extract_prop_u16(node, "spacing").unwrap_or(0);
    let padding = extract_prop_u16(node, "padding").unwrap_or(0);
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;
    let children: Result<Vec<_>, _> = kids.iter().map(convert_node).collect();
    let children = children?;

    let mut builder = View::row().spacing(spacing).padding(padding);

    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    for child in children {
        builder = builder.child(child);
    }

    Ok(builder.build())
}

/// Convert Container node: `container { width: 100, height: 200, child }`
fn convert_container(node: &Node) -> ConversionResult<View<String>> {
    let padding = extract_prop_u16(node, "padding").unwrap_or(0);
    let width = extract_prop_opt_u16(node, "width");
    let height = extract_prop_opt_u16(node, "height");
    let center_x = extract_prop_bool(node, "center_x").unwrap_or(false);
    let center_y = extract_prop_bool(node, "center_y").unwrap_or(false);
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;

    if kids.is_empty() {
        // Empty container
        let mut builder = View::container(View::empty()).padding(padding);
        if center_x { builder = builder.center_x(); }
        if center_y { builder = builder.center_y(); }
        if let Some(style) = style {
            builder = builder.with_style(style);
        }
        return Ok(builder.build());
    }

    let child = convert_node(&kids[0])?;

    let mut builder = View::container(child).padding(padding);
    if let Some(w) = width { builder = builder.width(w); }
    if let Some(h) = height { builder = builder.height(h); }
    if center_x { builder = builder.center_x(); }
    if center_y { builder = builder.center_y(); }
    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

/// Convert Scrollable node: `scrollable { width: 100, height: 200, child }`
fn convert_scrollable(node: &Node) -> ConversionResult<View<String>> {
    let width = extract_prop_opt_u16(node, "width");
    let height = extract_prop_opt_u16(node, "height");
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;

    if kids.is_empty() {
        // Empty scrollable
        let mut builder = View::scrollable(View::empty());
        if let Some(w) = width { builder = builder.width(w); }
        if let Some(h) = height { builder = builder.height(h); }
        if let Some(style) = style {
            builder = builder.with_style(style);
        }
        return Ok(builder.build());
    }

    let child = convert_node(&kids[0])?;

    let mut builder = View::scrollable(child);
    if let Some(w) = width { builder = builder.width(w); }
    if let Some(h) = height { builder = builder.height(h); }
    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

// ============================================================================
// Element Component Converters
// ============================================================================

/// Convert Text node: `text("Hello") { style: "text-lg" }`
fn convert_text(node: &Node) -> ConversionResult<View<String>> {
    // Get main argument as text content
    let content = extract_main_arg_str(node).unwrap_or_default();

    let style = extract_style(node)?;

    if let Some(style) = style {
        Ok(View::Text {
            content,
            style: Some(style),
        })
    } else {
        Ok(View::text(content))
    }
}

/// Convert Button node: `button("Click") { onclick: "clicked", style: "px-4" }`
fn convert_button(node: &Node) -> ConversionResult<View<String>> {
    let label = extract_main_arg_str(node).unwrap_or_default();

    // onclick is required for buttons
    let onclick = extract_prop_str(node, "onclick")
        .ok_or_else(|| ConversionError::MissingProp {
            kind: "button".to_string(),
            prop: "onclick".to_string(),
        })?;

    let style = extract_style(node)?;

    if let Some(style) = style {
        Ok(View::Button {
            label,
            onclick, // Store as string message ID
            style: Some(style),
        })
    } else {
        Ok(View::button(label, onclick))
    }
}

/// Convert Input node: `input("Email") { value: "", on_change: "changed" }`
fn convert_input(node: &Node) -> ConversionResult<View<String>> {
    let placeholder = extract_main_arg_str(node).unwrap_or_default();
    let value = extract_prop_str(node, "value").unwrap_or_default();
    let on_change = extract_prop_opt_str(node, "on_change");
    let width = extract_prop_opt_u16(node, "width");
    let password = extract_prop_bool(node, "password").unwrap_or(false);
    let style = extract_style(node)?;

    let mut builder = View::input(placeholder).value(value);

    if password {
        builder = builder.password();
    }
    if let Some(msg) = on_change {
        builder = builder.on_change(msg);
    }
    if let Some(w) = width {
        builder = builder.width(w);
    }
    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

/// Convert Checkbox node: `checkbox("Remember") { is_checked: true, on_toggle: "toggle" }`
fn convert_checkbox(node: &Node) -> ConversionResult<View<String>> {
    let label = extract_main_arg_str(node).unwrap_or_default();
    let is_checked = extract_prop_bool(node, "is_checked").unwrap_or(false);
    let on_toggle = extract_prop_opt_str(node, "on_toggle");
    let style = extract_style(node)?;

    let mut view = View::checkbox(is_checked, label);

    if let Some(msg) = on_toggle {
        view = view.on_toggle(msg);
    }
    if let Some(style) = style {
        if let View::Checkbox { style: s, .. } = &mut view {
            *s = Some(style);
        }
    }

    Ok(view)
}

/// Convert Radio node: `radio("Option") { is_selected: false, on_select: "select" }`
fn convert_radio(node: &Node) -> ConversionResult<View<String>> {
    let label = extract_main_arg_str(node).unwrap_or_default();
    let is_selected = extract_prop_bool(node, "is_selected").unwrap_or(false);
    let on_select = extract_prop_opt_str(node, "on_select");
    let style = extract_style(node)?;

    let mut view = View::radio(is_selected, label);

    if let Some(msg) = on_select {
        view = view.on_select(msg);
    }
    if let Some(style) = style {
        if let View::Radio { style: s, .. } = &mut view {
            *s = Some(style);
        }
    }

    Ok(view)
}

/// Convert Select node: `select { options: ["A", "B"], selected_index: 0, on_select: "change" }`
fn convert_select(node: &Node) -> ConversionResult<View<String>> {
    let options = extract_prop_str_array(node, "options").unwrap_or_default();
    let selected_index = extract_prop_opt_usize(node, "selected_index");
    let on_select = extract_prop_opt_str(node, "on_select");
    let style = extract_style(node)?;

    let mut view = View::select(options);

    if let Some(idx) = selected_index {
        view = view.selected(idx);
    }
    if let Some(msg) = on_select {
        view = view.on_choose(msg);
    }
    if let Some(style) = style {
        if let View::Select { style: s, .. } = &mut view {
            *s = Some(style);
        }
    }

    Ok(view)
}

/// Convert List node: `list { spacing: 10, items: [...] }`
fn convert_list(node: &Node) -> ConversionResult<View<String>> {
    let spacing = extract_prop_u16(node, "spacing").unwrap_or(0);
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;
    let items: Result<Vec<_>, _> = kids.iter().map(convert_node).collect();
    let items = items?;

    let mut builder = View::list(items).spacing(spacing);

    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

/// Convert Table node: `table { headers: [...], rows: [[...]], spacing: 5 }`
fn convert_table(node: &Node) -> ConversionResult<View<String>> {
    let spacing = extract_prop_u16(node, "spacing").unwrap_or(0);
    let col_spacing = extract_prop_u16(node, "col_spacing").unwrap_or(0);
    let style = extract_style(node)?;

    // Extract headers from child nodes
    let header_kids = extract_child_nodes_by_name(node, "header");
    let headers: Result<Vec<_>, _> = header_kids.iter().map(convert_node).collect();
    let headers = headers?;

    // Extract rows from child nodes
    let row_kids = extract_child_nodes_by_name(node, "row");
    let mut rows = Vec::new();

    for row_node in &row_kids {
        let cell_kids = extract_child_nodes(row_node)?;
        let row: Result<Vec<_>, _> = cell_kids.iter().map(convert_node).collect();
        rows.push(row?);
    }

    let mut builder = View::table(headers, rows).spacing(spacing).col_spacing(col_spacing);

    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

// ============================================================================
// Property Extraction Helpers
// ============================================================================

/// Extract main argument as string
fn extract_main_arg_str(node: &Node) -> Option<String> {
    let arg = node.main_arg();
    match arg {
        Value::Str(s) => Some(s.to_string()),
        Value::OwnedStr(s) => Some(s.as_str().to_string()),
        _ => None,
    }
}

/// Extract property as string
fn extract_prop_str(node: &Node, key: &str) -> Option<String> {
    let value = node.get_prop(key);
    match value {
        Value::Str(s) => Some(s.to_string()),
        Value::OwnedStr(s) => Some(s.as_str().to_string()),
        _ => None,
    }
}

/// Extract optional property as string
fn extract_prop_opt_str(node: &Node, key: &str) -> Option<String> {
    extract_prop_str(node, key)
}

/// Extract property as u16
fn extract_prop_u16(node: &Node, key: &str) -> Option<u16> {
    let value = node.get_prop(key);
    match value {
        Value::Int(i) if i >= 0 && i <= u16::MAX as i32 => Some(i as u16),
        Value::Uint(u) if u <= u16::MAX as u32 => Some(u as u16),
        Value::USize(u) if u <= u16::MAX as usize => Some(u as u16),
        _ => None,
    }
}

/// Extract optional property as u16
fn extract_prop_opt_u16(node: &Node, key: &str) -> Option<u16> {
    extract_prop_u16(node, key)
}

/// Extract optional property as usize
fn extract_prop_opt_usize(node: &Node, key: &str) -> Option<usize> {
    let value = node.get_prop(key);
    match value {
        Value::Int(i) if i >= 0 => Some(i as usize),
        Value::Uint(u) => Some(u as usize),
        Value::USize(u) => Some(u),
        _ => None,
    }
}

/// Extract property as bool
fn extract_prop_bool(node: &Node, key: &str) -> Option<bool> {
    let value = node.get_prop(key);
    match value {
        Value::Bool(b) => Some(b),
        _ => None,
    }
}

/// Extract property as string array
fn extract_prop_str_array(node: &Node, key: &str) -> Option<Vec<String>> {
    let value = node.get_prop(key);
    match value {
        Value::Array(arr) => {
            let strs: Vec<_> = arr.iter().filter_map(|v| match v {
                Value::Str(s) => Some(s.to_string()),
                Value::OwnedStr(s) => Some(s.as_str().to_string()),
                _ => None,
            }).collect();
            if strs.is_empty() { None } else { Some(strs) }
        }
        _ => None,
    }
}

/// Extract child nodes (all nodes in kids)
fn extract_child_nodes(node: &Node) -> ConversionResult<Vec<Node>> {
    let kids = node.kids_iter()
        .filter_map(|(_, kid)| {
            if let auto_val::Kid::Node(n) = kid {
                Some(n.clone())
            } else {
                None
            }
        })
        .collect();

    Ok(kids)
}

/// Extract child nodes by name
fn extract_child_nodes_by_name(node: &Node, name: &str) -> Vec<Node> {
    node.kids_iter()
        .filter_map(|(_, kid)| {
            if let auto_val::Kid::Node(n) = kid {
                if n.name.as_str() == name {
                    Some(n.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Extract style string and parse it (Plan 005 integration)
fn extract_style(node: &Node) -> ConversionResult<Option<Style>> {
    let style_str = extract_prop_str(node, "style");

    if let Some(s) = style_str {
        // Parse style string using Plan 005's Style::parse
        Style::parse(&s)
            .map(Some)
            .map_err(|e| ConversionError::InvalidPropType {
                kind: node.name.to_string(),
                prop: "style".to_string(),
                expected: "valid Tailwind CSS class string".to_string(),
                got: format!("invalid: {}", e),
            })
    } else {
        Ok(None)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Node;

    #[test]
    fn test_convert_text() {
        let node = Node::new("text").with_arg("Hello World");
        let view = convert_node(&node).unwrap();

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Hello World");
            }
            _ => panic!("Expected View::Text"),
        }
    }

    #[test]
    fn test_convert_button() {
        let node = Node::new("button")
            .with_arg("Click Me")
            .with_prop("onclick", "button-clicked");
        let view = convert_node(&node).unwrap();

        match view {
            View::Button { label, onclick, .. } => {
                assert_eq!(label, "Click Me");
                assert_eq!(onclick, "button-clicked");
            }
            _ => panic!("Expected View::Button"),
        }
    }

    #[test]
    fn test_convert_column() {
        let node = Node::new("col")
            .with_prop("spacing", 10u32)
            .with_child(Node::new("text").with_arg("Item 1"))
            .with_child(Node::new("text").with_arg("Item 2"));

        let view = convert_node(&node).unwrap();

        match view {
            View::Column { spacing, children, .. } => {
                assert_eq!(spacing, 10);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_convert_row() {
        let node = Node::new("row")
            .with_prop("spacing", 5u32)
            .with_child(Node::new("text").with_arg("A"))
            .with_child(Node::new("text").with_arg("B"));

        let view = convert_node(&node).unwrap();

        match view {
            View::Row { spacing, children, .. } => {
                assert_eq!(spacing, 5);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Row"),
        }
    }

    #[test]
    fn test_convert_container() {
        let node = Node::new("container")
            .with_prop("width", 100u32)
            .with_prop("height", 200u32)
            .with_child(Node::new("text").with_arg("Content"));

        let view = convert_node(&node).unwrap();

        match view {
            View::Container { width, height, .. } => {
                assert_eq!(width, Some(100));
                assert_eq!(height, Some(200));
            }
            _ => panic!("Expected View::Container"),
        }
    }

    #[test]
    fn test_convert_checkbox() {
        let node = Node::new("checkbox")
            .with_arg("Remember me")
            .with_prop("is_checked", true)
            .with_prop("on_toggle", "toggle-checkbox");

        let view = convert_node(&node).unwrap();

        match view {
            View::Checkbox { is_checked, label, on_toggle, .. } => {
                assert_eq!(is_checked, true);
                assert_eq!(label, "Remember me");
                assert_eq!(on_toggle, Some("toggle-checkbox".to_string()));
            }
            _ => panic!("Expected View::Checkbox"),
        }
    }

    #[test]
    fn test_unknown_kind() {
        let node = Node::new("unknown_widget");
        let result = convert_node(&node);

        match result {
            Err(ConversionError::UnknownKind { kind }) => {
                assert_eq!(kind, "unknown_widget");
            }
            _ => panic!("Expected UnknownKind error"),
        }
    }

    #[test]
    fn test_button_missing_onclick() {
        let node = Node::new("button").with_arg("Click");
        let result = convert_node(&node);

        match result {
            Err(ConversionError::MissingProp { kind, prop }) => {
                assert_eq!(kind, "button");
                assert_eq!(prop, "onclick");
            }
            _ => panic!("Expected MissingProp error"),
        }
    }

    #[test]
    fn test_nested_layout() {
        let node = Node::new("col")
            .with_prop("spacing", 10u32)
            .with_child(
                Node::new("row")
                    .with_prop("spacing", 5u32)
                    .with_child(Node::new("text").with_arg("A"))
                    .with_child(Node::new("text").with_arg("B"))
            )
            .with_child(Node::new("text").with_arg("C"));

        let view = convert_node(&node).unwrap();

        match view {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 2);

                // First child should be a Row
                match &children[0] {
                    View::Row { children: row_children, .. } => {
                        assert_eq!(row_children.len(), 2);
                    }
                    _ => panic!("Expected first child to be View::Row"),
                }
            }
            _ => panic!("Expected View::Column"),
        }
    }
}
