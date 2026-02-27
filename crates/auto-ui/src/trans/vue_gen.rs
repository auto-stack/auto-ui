//! Vue.js Code Generator
//!
//! Generates Vue 3 Single File Components (SFC) from Auto language AST.
//!
//! Output format:
//! - `<script setup lang="ts">` with Composition API
//! - `<template>` with HTML + Tailwind CSS classes
//! - `<style scoped>` for component styles
//!
//! # Deprecation Notice (Plan 096 Phase 5)
//!
//! This module is deprecated. Use `auto_lang::ui_gen::VueGenerator` instead.
//!
//! ## Migration Guide
//!
//! Old (deprecated):
//! ```ignore
//! use auto_ui::trans::VueCodeGenerator;
//! let mut gen = VueCodeGenerator::new();
//! let code = gen.generate_widget(&type_decl)?;
//! ```
//!
//! New (recommended):
//! ```ignore
//! use auto_lang::ui_gen::VueGenerator;
//! use auto_lang::ui_gen::BackendGenerator;
//! use auto_lang::aura::extract_widget_from_decl;
//!
//! let aura_widget = extract_widget_from_decl(&widget_decl)?;
//! let mut gen = VueGenerator::new();
//! let code = gen.generate(&aura_widget)?;
//! ```

#![deprecated(
    since = "0.2.0",
    note = "Use auto_lang::ui_gen::VueGenerator instead. See module docs for migration guide."
)]

use auto_lang::ast::{Arg, Body, Expr, Fn, IsBranch, Member, Node, Stmt, Type, TypeDecl};
use auto_val::Op;
use std::collections::HashSet;

/// Vue code generator for widgets
///
/// # Deprecated
/// Use `auto_lang::ui_gen::VueGenerator` instead.
#[deprecated(since = "0.2.0", note = "Use auto_lang::ui_gen::VueGenerator instead")]
pub struct VueCodeGenerator {
    current_widget: Option<String>,
    imports: HashSet<String>,
    component_refs: Vec<String>,
    fields: Vec<(String, String, Option<String>)>,
    has_emit: bool,
    emit_events: Vec<String>,
    handlers: Vec<(String, String)>,
    template_nodes: Vec<String>,
    tailwind_classes: String,
}

impl VueCodeGenerator {
    pub fn new() -> Self {
        Self {
            current_widget: None,
            imports: HashSet::new(),
            component_refs: Vec::new(),
            fields: Vec::new(),
            has_emit: false,
            emit_events: Vec::new(),
            handlers: Vec::new(),
            template_nodes: Vec::new(),
            tailwind_classes: String::new(),
        }
    }

    pub fn generate_widget(&mut self, type_decl: &TypeDecl) -> Result<String, String> {
        self.current_widget = Some(type_decl.name.to_string());
        self.reset();

        self.analyze_widget(type_decl)?;

        let script = self.generate_script();
        let template = self.generate_template();
        let style = self.generate_style();

        Ok(format!(
            "<!-- {} component - Auto-generated from Auto language -->\n<script setup lang=\"ts\">\n{}</script>\n\n<template>\n{}</template>\n\n<style scoped>\n{}</style>\n",
            type_decl.name, script, template, style
        ))
    }

    fn reset(&mut self) {
        self.imports.clear();
        self.component_refs.clear();
        self.fields.clear();
        self.has_emit = false;
        self.emit_events.clear();
        self.handlers.clear();
        self.template_nodes.clear();
        self.tailwind_classes.clear();
    }

    fn analyze_widget(&mut self, type_decl: &TypeDecl) -> Result<(), String> {
        for member in &type_decl.members {
            let name = member.name.to_string();
            let ty = self.auto_type_to_typescript(&member.ty);
            let default = member.value.as_ref().map(|e| self.expr_to_ts_value(e));
            self.fields.push((name, ty, default));
        }

        for method in &type_decl.methods {
            match method.name.as_str() {
                "view" => self.analyze_view_method(&method.body)?,
                "on" => self.analyze_on_method(&method.body)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn analyze_view_method(&mut self, body: &Body) -> Result<(), String> {
        for stmt in &body.stmts {
            if let Stmt::Expr(expr) = stmt {
                self.analyze_view_expr(expr)?;
            }
        }
        Ok(())
    }

    fn analyze_view_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Node(node) => {
                let html = self.node_to_html(node)?;
                self.template_nodes.push(html);
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_on_method(&mut self, body: &Body) -> Result<(), String> {
        for stmt in &body.stmts {
            if let Stmt::Is(is_stmt) = stmt {
                for branch in &is_stmt.branches {
                    if let IsBranch::EqBranch(pattern, handler_body) = branch {
                        self.process_pattern_branch(pattern, handler_body);
                    }
                }
            }
        }
        Ok(())
    }

    fn process_pattern_branch(&mut self, pattern: &Expr, handler_body: &Body) {
        let (event_name, handler_name) = self.extract_event_info(pattern);

        if let Some(name) = event_name {
            self.emit_events.push(name.to_lowercase());
            self.has_emit = true;
        }

        let handler_body = self.body_to_handler_code(handler_body);
        self.handlers.push((handler_name, handler_body));
    }

    fn extract_event_info(&self, pattern: &Expr) -> (Option<String>, String) {
        match pattern {
            Expr::Bina(lhs, op, rhs) => {
                if *op == Op::Dot {
                    if let (Expr::Ident(_type_name), Expr::Ident(variant)) =
                        (lhs.as_ref(), rhs.as_ref())
                    {
                        return (Some(variant.to_string()), format!("on{}", variant));
                    }
                }
                (None, "onUnknown".to_string())
            }
            Expr::Ident(name) => (Some(name.to_string()), format!("on{}", name)),
            _ => (None, "onUnknown".to_string()),
        }
    }

    fn body_to_handler_code(&self, body: &Body) -> String {
        let mut code = String::new();
        for stmt in &body.stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    code.push_str(&self.expr_to_ts_statement(expr));
                }
                Stmt::Store(store) => {
                    code.push_str(&self.store_to_ts(store.name.as_str(), &store.expr));
                }
                _ => {}
            }
        }
        code.trim_end().to_string()
    }

    fn store_to_ts(&self, name: &str, expr: &Expr) -> String {
        match expr {
            Expr::Int(n) => format!("{}.value = {}", name, n),
            Expr::Bina(lhs, op, rhs) => {
                let field = self.expr_to_ts_field(lhs);
                let value = self.expr_to_ts_value(rhs);
                let op_str = self.op_to_assignment(op);
                format!("{}.value {} {}", field, op_str, value)
            }
            _ => String::new(),
        }
    }

    fn expr_to_ts_field(&self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(name) => name.to_string(),
            Expr::Bina(obj, op, field) => {
                if *op == Op::Dot {
                    format!("{}.{}", self.expr_to_ts_field(obj), field)
                } else {
                    "unknown".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }

    fn node_to_html(&mut self, node: &Node) -> Result<String, String> {
        let tag = self.node_name_to_tag(&node.name);
        let classes = self.extract_classes_from_node(node);
        let attrs = self.extract_attrs_from_node(node);
        let children = self.generate_children(&node.body)?;

        let all_attrs = if classes.is_empty() {
            attrs
        } else if attrs.is_empty() {
            classes
        } else {
            format!("{} {}", classes, attrs)
        };

        if children.is_empty() {
            Ok(format!("<{}{} />", tag, all_attrs))
        } else {
            Ok(format!("<{}{}>\n{}\n</{}>", tag, all_attrs, children, tag))
        }
    }

    fn node_name_to_tag(&mut self, name: &str) -> String {
        match name {
            "col" | "column" => "div".to_string(),
            "row" => "div".to_string(),
            "center" => "div".to_string(),
            "button" => "button".to_string(),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => name.to_string(),
            "text" | "label" | "span" => "span".to_string(),
            "input" => "input".to_string(),
            "div" => "div".to_string(),
            _ => {
                if name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    self.component_refs.push(name.to_string());
                    name.to_string()
                } else {
                    "div".to_string()
                }
            }
        }
    }

    fn extract_classes_from_node(&self, node: &Node) -> String {
        let mut classes = Vec::new();

        match node.name.as_str() {
            "col" | "column" => classes.push("flex flex-col".to_string()),
            "row" => classes.push("flex flex-row".to_string()),
            "center" => classes.push("flex items-center justify-center".to_string()),
            _ => {}
        }

        for arg in &node.args.args {
            if let Arg::Pair(key, value) = arg {
                if key == "class" {
                    if let Expr::Str(s) = value {
                        classes.push(s.to_string());
                    }
                }
            }
        }

        if classes.is_empty() {
            String::new()
        } else {
            format!(" class=\"{}\"", classes.join(" "))
        }
    }

    fn extract_attrs_from_node(&self, node: &Node) -> String {
        let mut attrs = Vec::new();

        for arg in &node.args.args {
            if let Arg::Pair(key, value) = arg {
                match key.as_str() {
                    "onclick" => {
                        let handler = self.extract_handler_name(value);
                        attrs.push(format!("@click=\"{}\"", handler));
                    }
                    "bind" => {
                        let field = self.expr_to_ts_field(value);
                        attrs.push(format!("v-model=\"{}\"", field));
                    }
                    _ => {}
                }
            }
        }

        if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        }
    }

    fn extract_handler_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::Bina(lhs, op, rhs) => {
                if *op == Op::Dot {
                    if let Expr::Ident(variant) = rhs.as_ref() {
                        return format!("on{}", variant);
                    }
                }
                "onUnknown".to_string()
            }
            Expr::Ident(name) => format!("on{}", name),
            _ => "onUnknown".to_string(),
        }
    }

    fn generate_children(&mut self, body: &Body) -> Result<String, String> {
        let mut children = Vec::new();
        for stmt in &body.stmts {
            if let Stmt::Expr(expr) = stmt {
                match expr {
                    Expr::Node(child_node) => {
                        let html = self.node_to_html(child_node)?;
                        children.push(format!("  {}", html));
                    }
                    Expr::Str(s) => {
                        children.push(format!("  {}", s));
                    }
                    Expr::Ident(name) => {
                        children.push(format!("  {{{{ {} }}}}", name));
                    }
                    Expr::Bina(lhs, op, rhs) if *op == Op::Dot => {
                        let field = self.expr_to_ts_field(expr);
                        children.push(format!("  {{{{ {} }}}}", field));
                    }
                    _ => {}
                }
            }
        }
        Ok(children.join("\n"))
    }

    fn generate_script(&mut self) -> String {
        let mut script = String::new();

        if !self.fields.is_empty() {
            script.push_str("import { ref } from 'vue'\n\n");
        }

        for (name, ty, default) in &self.fields {
            let default_val = default.clone().unwrap_or_else(|| self.default_for_type(ty));
            script.push_str(&format!("const {} = ref<{}>({})\n", name, ty, default_val));
        }

        if !self.fields.is_empty() {
            script.push('\n');
        }

        if self.has_emit {
            script.push_str("const emit = defineEmits<{\n");
            for event in &self.emit_events {
                script.push_str(&format!("  {}: []\n", event));
            }
            script.push_str("}>()\n\n");
        }

        for (handler_name, handler_body) in &self.handlers {
            if handler_body.is_empty() {
                script.push_str(&format!(
                    "function {}() {{\n  // TODO\n}}\n\n",
                    handler_name
                ));
            } else {
                script.push_str(&format!(
                    "function {}() {{\n  {}\n}}\n\n",
                    handler_name, handler_body
                ));
            }
        }

        script
    }

    fn generate_template(&self) -> String {
        let mut template = String::new();

        let wrapper_classes = if self.tailwind_classes.is_empty() {
            "flex flex-col".to_string()
        } else {
            format!("flex flex-col {}", self.tailwind_classes)
        };

        template.push_str(&format!("  <div class=\"{}\">\n", wrapper_classes));

        for node in &self.template_nodes {
            for line in node.lines() {
                template.push_str(&format!("    {}\n", line));
            }
        }

        template.push_str("  </div>\n");
        template
    }

    fn generate_style(&self) -> String {
        String::from("/* Component styles */\n")
    }

    fn auto_type_to_typescript(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "number".to_string(),
            Type::Float => "number".to_string(),
            Type::Double => "number".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::Str(_) => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Unknown => "any".to_string(),
            _ => "any".to_string(),
        }
    }

    fn default_for_type(&self, ty: &str) -> String {
        match ty {
            "number" => "0".to_string(),
            "boolean" => "false".to_string(),
            "string" => "''".to_string(),
            _ => "null".to_string(),
        }
    }

    fn expr_to_ts_value(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::Float(f, _) => f.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Str(s) => format!("'{}'", s),
            Expr::Ident(name) => name.to_string(),
            _ => "null".to_string(),
        }
    }

    fn expr_to_ts_statement(&self, expr: &Expr) -> String {
        match expr {
            Expr::Bina(lhs, op, rhs) => {
                let field = self.expr_to_ts_field(lhs);
                let value = self.expr_to_ts_value(rhs);
                let op_str = self.op_to_assignment(op);
                format!("{}.value {} {}", field, op_str, value)
            }
            _ => String::new(),
        }
    }

    fn op_to_assignment(&self, op: &Op) -> &'static str {
        match op {
            Op::Add | Op::AddEq => "+=",
            Op::Sub | Op::SubEq => "-=",
            Op::Mul | Op::MulEq => "*=",
            Op::Div | Op::DivEq => "/=",
            Op::Asn | Op::Eq => "=",
            _ => "=",
        }
    }
}

impl Default for VueCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_generator() {
        let gen = VueCodeGenerator::new();
        assert!(gen.current_widget.is_none());
    }

    #[test]
    fn test_type_conversion() {
        let gen = VueCodeGenerator::new();

        assert_eq!(gen.auto_type_to_typescript(&Type::Int), "number");
        assert_eq!(gen.auto_type_to_typescript(&Type::Str(0)), "string");
        assert_eq!(gen.auto_type_to_typescript(&Type::Bool), "boolean");
    }

    #[test]
    fn test_op_conversion() {
        let gen = VueCodeGenerator::new();

        assert_eq!(gen.op_to_assignment(&Op::Add), "+=");
        assert_eq!(gen.op_to_assignment(&Op::Sub), "-=");
        assert_eq!(gen.op_to_assignment(&Op::Asn), "=");
    }
}
