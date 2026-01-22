// Rust Code Generator for AutoUI Transpiler
//
// This module generates idiomatic Rust code from Auto language widget definitions.

use crate::style::Style;
use auto_lang::ast::*;
use std::collections::HashSet;

/// Rust code generator for widgets
pub struct RustCodeGenerator {
    /// Current widget being processed
    current_widget: Option<String>,
    /// Message variants discovered during parsing
    messages: HashSet<MessageVariant>,
    /// Message type (extracted from on() method parameter)
    message_type: Option<String>,
    /// Imports needed for generated code
    imports: HashSet<String>,
    /// Current indent level
    indent: usize,
}

/// Message variant definition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageVariant {
    pub name: String,
    pub has_fields: bool,
}

impl RustCodeGenerator {
    pub fn new() -> Self {
        Self {
            current_widget: None,
            messages: HashSet::new(),
            message_type: None,
            imports: HashSet::new(),
            indent: 0,
        }
    }

    /// Generate complete Rust code from widget type declaration
    pub fn generate_widget(&mut self, type_decl: &TypeDecl) -> Result<String, String> {
        self.current_widget = Some(type_decl.name.to_string());
        self.messages.clear();
        self.message_type = None;
        self.imports.clear();

        let mut code = String::new();

        // Add file header
        code.push_str("// Auto-generated from Auto language\n");
        code.push_str("// DO NOT EDIT - changes will be overwritten\n\n");

        // Collect imports and analyze methods
        self.analyze_widget(type_decl)?;

        // Generate imports
        code.push_str(&self.generate_imports());
        code.push('\n');

        // Generate message enum (if has messages)
        if !self.messages.is_empty() {
            code.push_str(&self.generate_message_enum());
            code.push('\n');
        }

        // Generate struct definition
        code.push_str(&self.generate_struct(type_decl));
        code.push('\n');

        // Generate constructor
        code.push_str(&self.generate_constructor(type_decl));
        code.push('\n');

        // Generate Component impl
        code.push_str(&self.generate_component_impl(type_decl));
        code.push('\n');

        self.current_widget = None;
        Ok(code)
    }

    /// Analyze widget to extract messages and imports
    fn analyze_widget(&mut self, type_decl: &TypeDecl) -> Result<(), String> {
        // Add default imports
        self.imports.insert("auto_ui::Component".to_string());
        self.imports.insert("auto_ui::View".to_string());

        // Analyze methods
        for method in &type_decl.methods {
            if method.name == "view" {
                self.analyze_view_method(&method.body)?;
            } else if method.name == "on" {
                self.analyze_on_method(&method.params, &method.body)?;
            }
        }

        Ok(())
    }

    /// Analyze view() method to collect UI component usage
    fn analyze_view_method(&mut self, body: &Body) -> Result<(), String> {
        for stmt in &body.stmts {
            if let Stmt::Expr(expr) = stmt {
                self.analyze_expr(expr)?;
            }
        }
        Ok(())
    }

    /// Analyze expression for UI components
    fn analyze_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Node(node) => {
                // Analyze node type and add necessary imports
                match node.name.as_str() {
                    "col" | "column" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "row" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "center" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "container" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "scrollable" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "button" => {
                        // Button needs message type
                    }
                    "input" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "checkbox" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "radio" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "select" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "list" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "table" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    "text" | "label" => {
                        self.imports.insert("auto_ui::View".to_string());
                    }
                    _ => {}
                }

                // Analyze body for nested nodes
                for stmt in &node.body.stmts {
                    if let Stmt::Expr(expr) = stmt {
                        self.analyze_expr(expr)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Analyze on() method to extract message type and variants
    fn analyze_on_method(&mut self, params: &[Param], body: &Body) -> Result<(), String> {
        // Extract message type from first parameter
        if let Some(first_param) = params.first() {
            let msg_type = self.rust_type_name(&first_param.ty);
            self.message_type = Some(msg_type);
        }

        // Extract message variants from is statements
        for stmt in &body.stmts {
            if let Stmt::Is(is_stmt) = stmt {
                // Match statement on message - extract patterns from is statement
                for branch in &is_stmt.branches {
                    if let auto_lang::ast::IsBranch::EqBranch(pattern, _body) = branch {
                        // Pattern is an expression - check if it's an identifier with Msg prefix
                        if let Expr::Ident(name) = pattern {
                            // Extract message variant name
                            if let Some(msg_name) = name.strip_prefix("Msg.") {
                                self.messages.insert(MessageVariant {
                                    name: msg_name.to_string(),
                                    has_fields: false, // TODO: detect field patterns
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Generate import statements
    fn generate_imports(&self) -> String {
        let mut imports: Vec<_> = self.imports.iter().cloned().collect();
        imports.sort();
        imports.dedup();

        let mut code = String::new();
        for import in imports {
            code.push_str(&format!("use {};\n", import));
        }
        code
    }

    /// Generate message enum definition
    fn generate_message_enum(&self) -> String {
        let mut messages: Vec<_> = self.messages.iter().collect();
        messages.sort_by_key(|m| &m.name);

        let mut code = String::new();
        code.push_str("#[derive(Clone, Copy, Debug, PartialEq)]\n");
        code.push_str("pub enum Msg {\n");

        for msg in messages {
            code.push_str(&format!("    {},\n", msg.name));
        }

        code.push_str("}\n");
        code
    }

    /// Generate struct definition
    fn generate_struct(&self, type_decl: &TypeDecl) -> String {
        let mut code = String::new();

        code.push_str("#[derive(Debug)]\n");
        code.push_str(&format!("pub struct {} {{\n", type_decl.name));

        for member in &type_decl.members {
            let field_name = &member.name;
            let field_type = self.rust_type_name(&member.ty);
            code.push_str(&format!("    pub {}: {},\n", field_name, field_type));
        }

        code.push_str("}\n");

        code
    }

    /// Generate constructor method
    fn generate_constructor(&self, type_decl: &TypeDecl) -> String {
        let widget_name = &type_decl.name;
        let mut code = String::new();

        code.push_str(&format!("impl {} {{\n", widget_name));

        // Parameter list
        let params: Vec<String> = type_decl.members.iter()
            .map(|m| format!("{}: {}", m.name, self.rust_type_name(&m.ty)))
            .collect();

        code.push_str(&format!("    pub fn new({}) -> Self {{\n", params.join(", ")));

        code.push_str("        Self {\n");
        for member in &type_decl.members {
            code.push_str(&format!("            {},\n", member.name));
        }
        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("}\n");

        code
    }

    /// Generate Component trait implementation
    fn generate_component_impl(&self, type_decl: &TypeDecl) -> String {
        let widget_name = &type_decl.name;
        let mut code = String::new();

        code.push_str(&format!("impl Component for {} {{\n", widget_name));

        // Message type - use extracted message type or default to ()
        let msg_type = self.message_type.as_ref().map(|s| s.as_str()).unwrap_or("()");
        code.push_str(&format!("    type Msg = {};\n", msg_type));
        code.push('\n');

        // Find on() method
        let on_method = type_decl.methods.iter().find(|m| m.name == "on");
        if let Some(method) = on_method {
            code.push_str(&self.generate_on_method_impl(method));
        } else {
            // Default empty on() implementation
            code.push_str("    fn on(&mut self, _msg: Self::Msg) {}\n");
        }
        code.push('\n');

        // Find view() method
        let view_method = type_decl.methods.iter().find(|m| m.name == "view");
        if let Some(method) = view_method {
            code.push_str(&self.generate_view_method_impl(method));
        } else {
            code.push_str("    fn view(&self) -> View<Self::Msg> {\n");
            code.push_str("        View::empty()\n");
            code.push_str("    }\n");
        }

        code.push_str("}\n");

        code
    }

    /// Generate on() method implementation
    fn generate_on_method_impl(&self, method: &Fn) -> String {
        let mut code = String::new();

        code.push_str("    fn on(&mut self, msg: Self::Msg) {\n");

        // Generate match statement
        code.push_str("        match msg {\n");

        // Process match arms from body
        for stmt in &method.body.stmts {
            if let Stmt::Is(is_stmt) = stmt {
                for branch in &is_stmt.branches {
                    if let auto_lang::ast::IsBranch::EqBranch(pattern, body) = branch {
                        // Generate pattern match for any expression type
                        let pattern_code = self.generate_expr_pattern(pattern);
                        code.push_str(&format!("            {} => {{\n", pattern_code));
                        code.push_str(&self.generate_body_stmts(body));
                        code.push_str("            }\n");
                    }
                }
            }
        }

        // Add wildcard arm if not complete
        code.push_str("            _ => {}\n");
        code.push_str("        }\n");
        code.push_str("    }\n");

        code
    }

    /// Generate view() method implementation
    fn generate_view_method_impl(&self, method: &Fn) -> String {
        let mut code = String::new();

        code.push_str("    fn view(&self) -> View<Self::Msg> {\n");

        // Collect all view expressions
        let mut view_exprs: Vec<String> = Vec::new();
        for stmt in &method.body.stmts {
            // Handle both Stmt::Expr(Expr::Node(...)) and Stmt::Node(...)
            let result = match stmt {
                Stmt::Expr(expr) => self.generate_view_expr(expr),
                Stmt::Node(node) => self.generate_view_node(node),
                _ => continue,
            };

            match result {
                Ok(view_code) => {
                    view_exprs.push(view_code);
                }
                Err(e) => {
                    // Log error but continue
                    eprintln!("[WARN] Failed to generate view expr: {}", e);
                }
            }
        }

        // Generate code based on number of expressions
        if view_exprs.is_empty() {
            code.push_str("        View::empty()\n");
        } else if view_exprs.len() == 1 {
            // Single expression - return it directly
            code.push_str(&format!("        {}\n", view_exprs[0]));
        } else {
            // Multiple expressions - wrap in a col
            code.push_str("        View::col().spacing(0).padding(0)\n");
            for expr in &view_exprs {
                code.push_str(&format!("        .child({})\n", expr));
            }
            code.push_str("        .build()\n");
        }

        code.push_str("    }\n");

        code
    }

    /// Generate view expression code
    fn generate_view_expr(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Node(node) => self.generate_view_node(node),
            Expr::Call(call) => {
                // Handle function call expressions like text(msg), button(label) {...}
                self.generate_call_expr(call)
            }
            Expr::Ident(name) => {
                // Reference to field
                Ok(format!("View::text(&self.{})", name))
            }
            _ => Ok("View::empty()".to_string()),
        }
    }

    /// Generate call expression code (e.g., text(msg), button(label))
    /// Note: This only handles simple function calls without body
    /// Calls with body {} are handled as Node
    fn generate_call_expr(&self, call: &Call) -> Result<String, String> {
        // Get the function name from the expression
        let name = match call.name.as_ref() {
            Expr::Ident(n) => n.to_string(),
            _ => return Ok("/* unknown call */".to_string()),
        };

        match name.as_str() {
            "text" | "label" => {
                // text(msg) -> View::text(&self.msg) or View::text(&"string")
                let content = if call.args.len() > 0 {
                    if let Some(arg) = call.args.get(0) {
                        let expr = arg.get_expr();
                        match &expr {
                            Expr::Ident(name) => {
                                // For field references, use .to_string() to handle int types
                                format!("&self.{}.to_string()", name)
                            }
                            Expr::Str(s) => format!("&\"{}\"", s),
                            _ => "&\"\"".to_string(),
                        }
                    } else {
                        "&\"\"".to_string()
                    }
                } else {
                    "&\"\"".to_string()
                };
                Ok(format!("View::text({})", content))
            }
            "button" => {
                // button(label, onclick: value)
                let label = if call.args.len() > 0 {
                    if let Some(arg) = call.args.get(0) {
                        let expr = arg.get_expr();
                        match &expr {
                            Expr::Ident(name) => format!("&self.{}", name),
                            Expr::Str(s) => format!("\"{}\"", s),
                            _ => "\"\"".to_string(),
                        }
                    } else {
                        "\"\"".to_string()
                    }
                } else {
                    "\"\"".to_string()
                };

                // Get onclick property from named args
                let onclick = call.args.args.iter()
                    .find(|arg| matches!(arg, Arg::Pair(name, _) if name.as_str() == "onclick"))
                    .and_then(|arg| {
                        if let Arg::Pair(_, expr) = arg {
                            Some(expr)
                        } else {
                            None
                        }
                    });

                let onclick_value = match onclick {
                    Some(Expr::Int(n)) => n.to_string(),
                    Some(Expr::Str(s)) => format!("\"{}\"", s),
                    Some(Expr::Ident(name)) => name.to_string(),
                    _ => "0".to_string(),
                };

                Ok(format!("View::button({}, {})", label, onclick_value))
            }
            "input" => {
                // input(placeholder, value: ..., style: ...)
                let placeholder = if call.args.len() > 0 {
                    if let Some(arg) = call.args.get(0) {
                        let expr = arg.get_expr();
                        match &expr {
                            Expr::Str(s) => format!("\"{}\"", s),
                            _ => "\"\"".to_string(),
                        }
                    } else {
                        "\"\"".to_string()
                    }
                } else {
                    "\"\"".to_string()
                };

                Ok(format!("View::input({}).build()", placeholder))
            }
            "checkbox" => {
                // checkbox(label, is_checked: ...)
                let label = if call.args.len() > 0 {
                    if let Some(arg) = call.args.get(0) {
                        let expr = arg.get_expr();
                        match &expr {
                            Expr::Str(s) => format!("\"{}\"", s),
                            _ => "\"\"".to_string(),
                        }
                    } else {
                        "\"\"".to_string()
                    }
                } else {
                    "\"\"".to_string()
                };

                // Get is_checked property
                let is_checked = call.args.args.iter()
                    .find(|arg| matches!(arg, Arg::Pair(name, _) if name.as_str() == "is_checked"))
                    .and_then(|arg| {
                        if let Arg::Pair(_, expr) = arg {
                            Some(expr)
                        } else {
                            None
                        }
                    });

                let checked_value = match is_checked {
                    Some(Expr::Bool(b)) => b.to_string(),
                    _ => "false".to_string(),
                };

                Ok(format!("View::checkbox({}, {})", checked_value, label))
            }
            "radio" => {
                // radio(label, is_selected: ...)
                let label = if call.args.len() > 0 {
                    if let Some(arg) = call.args.get(0) {
                        let expr = arg.get_expr();
                        match &expr {
                            Expr::Str(s) => format!("\"{}\"", s),
                            _ => "\"\"".to_string(),
                        }
                    } else {
                        "\"\"".to_string()
                    }
                } else {
                    "\"\"".to_string()
                };

                // Get is_selected property
                let is_selected = call.args.args.iter()
                    .find(|arg| matches!(arg, Arg::Pair(name, _) if name.as_str() == "is_selected"))
                    .and_then(|arg| {
                        if let Arg::Pair(_, expr) = arg {
                            Some(expr)
                        } else {
                            None
                        }
                    });

                let selected_value = match is_selected {
                    Some(Expr::Bool(b)) => b.to_string(),
                    _ => "false".to_string(),
                };

                Ok(format!("View::radio({}, {})", selected_value, label))
            }
            "select" => {
                // select(options) or select("default_value")
                let default = if call.args.len() > 0 {
                    if let Some(arg) = call.args.get(0) {
                        let expr = arg.get_expr();
                        match &expr {
                            Expr::Str(s) => format!("\"{}\"", s),
                            _ => "\"\"".to_string(),
                        }
                    } else {
                        "\"\"".to_string()
                    }
                } else {
                    "\"\"".to_string()
                };

                Ok(format!("View::select(vec![{}])", default))
            }
            _ => Ok(format!("/* unknown call: {} */", name)),
        }
    }

    /// Generate view node code
    fn generate_view_node(&self, node: &Node) -> Result<String, String> {
        match node.name.as_str() {
            "col" | "column" => self.generate_col_node(node),
            "row" => self.generate_row_node(node),
            "center" => self.generate_center_node(node),
            "container" => self.generate_container_node(node),
            "scrollable" => self.generate_scrollable_node(node),
            "text" | "label" => self.generate_text_node(node),
            "button" => self.generate_button_node(node),
            "input" => self.generate_input_node(node),
            "checkbox" => self.generate_checkbox_node(node),
            "radio" => self.generate_radio_node(node),
            "select" => self.generate_select_node(node),
            "list" => self.generate_list_node(node),
            "table" => self.generate_table_node(node),
            _ => Ok(format!("/* unknown widget: {} */", node.name)),
        }
    }

    fn generate_col_node(&self, node: &Node) -> Result<String, String> {
        let spacing = self.get_prop_u16(node, "spacing").unwrap_or(0);
        let padding = self.get_prop_u16(node, "padding").unwrap_or(0);
        let style = self.get_prop_string(node, "style");

        let mut code = format!("View::col().spacing({}).padding({})", spacing, padding);

        if let Some(s) = style {
            code.push_str(&format!(".style(\"{}\")", s));
        }

        // Process children from body statements
        for stmt in &node.body.stmts {
            // Handle both Stmt::Expr and Stmt::Node
            let child_result = match stmt {
                Stmt::Expr(expr) => self.generate_view_expr(expr),
                Stmt::Node(child_node) => self.generate_view_node(child_node),
                _ => continue,
            };

            if let Ok(child_code) = child_result {
                code.push_str(&format!("\n    .child({})", child_code));
            }
        }

        code.push_str("\n    .build()");
        Ok(code)
    }

    fn generate_row_node(&self, node: &Node) -> Result<String, String> {
        let spacing = self.get_prop_u16(node, "spacing").unwrap_or(0);
        let padding = self.get_prop_u16(node, "padding").unwrap_or(0);
        let style = self.get_prop_string(node, "style");

        let mut code = format!("View::row().spacing({}).padding({})", spacing, padding);

        if let Some(s) = style {
            code.push_str(&format!(".style(\"{}\")", s));
        }

        // Process children from body statements
        for stmt in &node.body.stmts {
            // Handle both Stmt::Expr and Stmt::Node
            let child_result = match stmt {
                Stmt::Expr(expr) => self.generate_view_expr(expr),
                Stmt::Node(child_node) => self.generate_view_node(child_node),
                _ => continue,
            };

            if let Ok(child_code) = child_result {
                code.push_str(&format!("\n    .child({})", child_code));
            }
        }

        code.push_str("\n    .build()");
        Ok(code)
    }

    fn generate_center_node(&self, node: &Node) -> Result<String, String> {
        // Get first child from body
        for stmt in &node.body.stmts {
            if let Stmt::Expr(expr) = stmt {
                if let Ok(child_code) = self.generate_view_expr(expr) {
                    return Ok(format!("View::container({})\n    .center()\n    .build()", child_code));
                }
            }
        }
        Ok("View::empty()".to_string())
    }

    fn generate_text_node(&self, node: &Node) -> Result<String, String> {
        // Try to get the content argument
        // In Auto: text(msg) where msg is the first positional argument
        let content = if node.args.len() > 0 {
            // Try to get first argument
            if let Some(arg) = node.args.get(0) {
                let expr = arg.get_expr();
                // Format the expression for Rust code
                // For text node, we need to pass a reference: &self.msg or &"string"
                match &expr {
                    Expr::Ident(name) => format!("&self.{}", name),
                    Expr::Str(s) => format!("&\"{}\"", s),
                    _ => "&\"\"".to_string(),
                }
            } else {
                "&\"\"".to_string()
            }
        } else {
            "&\"\"".to_string()
        };

        // Try to get style if present
        let has_style = node.args.args.iter().any(|arg| {
            matches!(arg, Arg::Pair(name, _) if name.as_str() == "style")
        });

        if has_style {
            Ok(format!("View::text_styled({}, /* style */)", content))
        } else {
            Ok(format!("View::text({})", content))
        }
    }

    fn generate_button_node(&self, node: &Node) -> Result<String, String> {
        let label = self.get_main_arg(node).unwrap_or("\"\"".to_string());
        let onclick = self.get_prop_string(node, "onclick")
            .ok_or_else(|| "Button missing 'onclick' property".to_string())?;
        let style = self.get_prop_string(node, "style");

        if let Some(s) = style {
            Ok(format!("View::button_styled({}, {}, \"{}\")", label, onclick, s))
        } else {
            Ok(format!("View::button({}, {})", label, onclick))
        }
    }

    fn generate_input_node(&self, node: &Node) -> Result<String, String> {
        let placeholder = self.get_main_arg(node).unwrap_or("\"\"".to_string());
        let value = self.get_prop_string(node, "value").unwrap_or("\"\"".to_string());
        let style = self.get_prop_string(node, "style");

        let mut code = format!("View::input({}).value({})", placeholder, value);

        if let Some(s) = style {
            code.push_str(&format!(".style(\"{}\")", s));
        }

        code.push_str("\n    .build()");
        Ok(code)
    }

    fn generate_container_node(&self, node: &Node) -> Result<String, String> {
        let padding = self.get_prop_u16(node, "padding").unwrap_or(0);
        let width = self.get_prop_opt_u16(node, "width");
        let height = self.get_prop_opt_u16(node, "height");
        let center_x = self.get_prop_bool(node, "center_x").unwrap_or(false);
        let center_y = self.get_prop_bool(node, "center_y").unwrap_or(false);
        let style = self.get_prop_string(node, "style");

        // Get first child from body
        for stmt in &node.body.stmts {
            if let Stmt::Expr(expr) = stmt {
                if let Ok(child_code) = self.generate_view_expr(expr) {
                    let mut code = format!("View::container({}).padding({})", child_code, padding);

                    if let Some(w) = width {
                        code.push_str(&format!(".width({})", w));
                    }
                    if let Some(h) = height {
                        code.push_str(&format!(".height({})", h));
                    }
                    if center_x {
                        code.push_str(".center_x()");
                    }
                    if center_y {
                        code.push_str(".center_y()");
                    }
                    if let Some(s) = style {
                        code.push_str(&format!(".style(\"{}\")", s));
                    }

                    code.push_str("\n    .build()");
                    return Ok(code);
                }
            }
        }

        Ok("View::empty()".to_string())
    }

    fn generate_scrollable_node(&self, node: &Node) -> Result<String, String> {
        let width = self.get_prop_opt_u16(node, "width");
        let height = self.get_prop_opt_u16(node, "height");
        let style = self.get_prop_string(node, "style");

        // Get first child from body
        for stmt in &node.body.stmts {
            if let Stmt::Expr(expr) = stmt {
                if let Ok(child_code) = self.generate_view_expr(expr) {
                    let mut code = format!("View::scrollable({})", child_code);

                    if let Some(w) = width {
                        code.push_str(&format!(".width({})", w));
                    }
                    if let Some(h) = height {
                        code.push_str(&format!(".height({})", h));
                    }
                    if let Some(s) = style {
                        code.push_str(&format!(".style(\"{}\")", s));
                    }

                    code.push_str("\n    .build()");
                    return Ok(code);
                }
            }
        }

        Ok("View::empty()".to_string())
    }

    fn generate_checkbox_node(&self, node: &Node) -> Result<String, String> {
        let label = self.get_main_arg(node).unwrap_or("\"\"".to_string());
        let is_checked = self.get_prop_bool(node, "is_checked").unwrap_or(false);
        let style = self.get_prop_string(node, "style");

        let mut code = format!("View::checkbox({}, {})", is_checked, label);

        if let Some(s) = style {
            if let Ok(parsed) = Style::parse(&s) {
                // Has style but we can't apply it to checkbox easily yet
                // For now just ignore
            }
        }

        Ok(code)
    }

    fn generate_radio_node(&self, node: &Node) -> Result<String, String> {
        let label = self.get_main_arg(node).unwrap_or("\"\"".to_string());
        let is_selected = self.get_prop_bool(node, "is_selected").unwrap_or(false);

        Ok(format!("View::radio({}, {})", is_selected, label))
    }

    fn generate_select_node(&self, node: &Node) -> Result<String, String> {
        let options = self.get_prop_string(node, "options").unwrap_or("\"[]\"".to_string());

        Ok(format!("View::select(vec![{}])", options))
    }

    fn generate_list_node(&self, node: &Node) -> Result<String, String> {
        let spacing = self.get_prop_u16(node, "spacing").unwrap_or(0);

        Ok(format!("View::list(vec![]).spacing({}).build()", spacing))
    }

    fn generate_table_node(&self, node: &Node) -> Result<String, String> {
        let spacing = self.get_prop_u16(node, "spacing").unwrap_or(0);
        let col_spacing = self.get_prop_u16(node, "col_spacing").unwrap_or(0);

        Ok(format!("View::table(vec![], vec![]).spacing({}).col_spacing({}).build()", spacing, col_spacing))
    }

    // Helper methods

    fn get_main_arg(&self, node: &Node) -> Option<String> {
        // Get first positional argument using args.get()
        if let Some(arg) = node.args.get(0) {
            // Use arg.get_expr() to get the expression
            let expr = arg.get_expr();
            Some(format!("\"{}\"", self.expr_to_string(&expr)))
        } else {
            None
        }
    }

    fn get_prop_string(&self, node: &Node, key: &str) -> Option<String> {
        // Use args.lookup() to find named argument
        if let Some(arg) = node.args.lookup(key) {
            let expr = arg.get_expr();
            if let Expr::Str(s) = &expr {
                return Some(format!("\"{}\"", s));
            }
        }
        None
    }

    fn get_prop_u16(&self, node: &Node, key: &str) -> Option<u16> {
        if let Some(arg) = node.args.lookup(key) {
            let expr = arg.get_expr();
            if let Expr::Int(n) = &expr {
                return Some(*n as u16);
            }
        }
        None
    }

    fn get_prop_opt_u16(&self, node: &Node, key: &str) -> Option<u16> {
        self.get_prop_u16(node, key)
    }

    fn get_prop_bool(&self, node: &Node, key: &str) -> Option<bool> {
        if let Some(arg) = node.args.lookup(key) {
            let expr = arg.get_expr();
            if let Expr::Bool(b) = &expr {
                return Some(*b);
            }
        }
        None
    }

    fn get_prop_u16_from_args(&self, args: &Args, key: &str) -> Option<u16> {
        if let Some(arg) = args.lookup(key) {
            let expr = arg.get_expr();
            if let Expr::Int(n) = &expr {
                return Some(*n as u16);
            }
        }
        None
    }

    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::Str(s) => s.to_string(),
            Expr::Int(n) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Ident(name) => name.to_string(),
            _ => "\"\"".to_string(),
        }
    }

    fn expr_to_rust(&self, expr: &Expr) -> String {
        match expr {
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::Int(n) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Ident(name) => format!("self.{}", name),
            Expr::Bina(lhs, op, rhs) => {
                // Handle expressions like Msg.Inc
                if let Expr::Ident(lhs_name) = lhs.as_ref() {
                    if let Expr::Ident(rhs_name) = rhs.as_ref() {
                        return format!("{}::{}", lhs_name, rhs_name);
                    }
                }
                "\"\"".to_string()
            }
            _ => "\"\"".to_string(),
        }
    }

    /// Generate pattern match code for match expressions
    fn generate_expr_pattern(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::Bool(b) => b.to_string(),
            Expr::Ident(name) => {
                // Handle Msg.EnumVariant format
                if let Some(msg_name) = name.strip_prefix("Msg.") {
                    msg_name.to_string()
                } else {
                    name.to_string()
                }
            }
            _ => "_".to_string(),
        }
    }

    fn generate_body_stmts(&self, body: &Body) -> String {
        let mut code = String::new();
        for stmt in &body.stmts {
            match stmt {
                Stmt::Store(store) => {
                    // Store uses 'name' (of type Name) and 'expr' fields
                    let name_str = store.name.to_string();
                    if let Expr::Int(n) = &store.expr {
                        code.push_str(&format!("                self.{} = {};\n", name_str, n));
                    } else if let Expr::Bina(lhs, op, rhs) = &store.expr {
                        // Handle binary operations like += and -=
                        if let Expr::Ident(field_name) = lhs.as_ref() {
                            if let Expr::Int(1) = rhs.as_ref() {
                                // Check operator by string representation
                                if op.to_string().contains("+") || op.to_string().contains("+=") {
                                    code.push_str(&format!("                self.{} += 1;\n", field_name));
                                } else if op.to_string().contains("-") || op.to_string().contains("-=") {
                                    code.push_str(&format!("                self.{} -= 1;\n", field_name));
                                }
                            }
                        }
                    }
                }
                Stmt::Expr(expr) => {
                    // Handle expression statements like count += 1
                    if let Expr::Bina(lhs, op, rhs) = expr {
                        if let Expr::Ident(field_name) = lhs.as_ref() {
                            let op_str = op.to_string();

                            // Generate code based on operator (use semicolon for statement termination)
                            if op_str.contains("+=") {
                                if let Expr::Int(n) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} += {};\n", field_name, n));
                                } else if let Expr::Ident(rhs_name) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} += self.{};\n", field_name, rhs_name));
                                }
                            } else if op_str.contains("-=") {
                                if let Expr::Int(n) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} -= {};\n", field_name, n));
                                } else if let Expr::Ident(rhs_name) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} -= self.{};\n", field_name, rhs_name));
                                }
                            } else if op_str.contains("*=") {
                                if let Expr::Int(n) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} *= {};\n", field_name, n));
                                } else if let Expr::Ident(rhs_name) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} *= self.{};\n", field_name, rhs_name));
                                }
                            } else if op_str.contains("/=") {
                                if let Expr::Int(n) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} /= {};\n", field_name, n));
                                } else if let Expr::Ident(rhs_name) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} /= self.{};\n", field_name, rhs_name));
                                }
                            } else if op_str.contains("=") && !op_str.contains("+=") && !op_str.contains("-=") && !op_str.contains("*=") && !op_str.contains("/=") {
                                // Simple assignment (not +=, -=, *=, /=)
                                if let Expr::Int(n) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} = {};\n", field_name, n));
                                } else if let Expr::Ident(rhs_name) = rhs.as_ref() {
                                    code.push_str(&format!("                self.{} = self.{};\n", field_name, rhs_name));
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Ignore unknown statement types for now
                }
            }
        }
        code
    }

    fn rust_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "i32".to_string(),
            Type::Str(_) => "String".to_string(),
            Type::Bool => "bool".to_string(),
            Type::User(user) => user.name.to_string(),
            _ => "/* unknown type */".to_string(),
        }
    }
}
