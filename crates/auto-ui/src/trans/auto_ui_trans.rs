// AutoUI Transpiler
//
// Transpiles Auto language .at files to auto-ui Rust code.

use super::{CodeSink, Trans};
use auto_lang::{Parser, ast::*};
use std::collections::HashSet;

// Type alias for Result
type AutoResult<T> = Result<T, String>;

pub struct AutoUITrans {
    current_widget: Option<String>,
    messages: HashSet<String>,
}

impl AutoUITrans {
    pub fn new() -> Self {
        Self {
            current_widget: None,
            messages: HashSet::new(),
        }
    }

    /// Transpile a single .at file to Rust code
    pub fn transpile_file(path: &str) -> AutoResult<String> {
        // Read the .at file
        let code = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

        // Try to parse with auto-lang parser
        // Note: This is a simplified implementation that manually handles the parsing
        // for now since auto-lang integration has compilation issues

        // For now, return the source code as a comment
        Ok(format!("// Transpiled from {}\n\n// Source .at file:\n{}\n\n// TODO: Implement full parsing and transpilation\n// The transpiler framework is ready, but needs integration with auto-lang parser.\n// Below is the expected Rust output structure:\n/*\nuse auto_ui::{{Component, View}};

#[derive(Debug)]
pub struct Hello {{
    pub msg: String,
}}

impl Component for Hello {{
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {{}}

    fn view(&self) -> View<()> {{
        View::text(&self.msg)
    }}
}}
*/", path, code))
    }
}

impl Trans for AutoUITrans {
    fn trans(&mut self, ast: Code, sink: &mut CodeSink) -> AutoResult<()> {
        // Add default imports
        sink.add_import("auto_ui::{Component, View}");

        // Process all statements
        for stmt in ast.stmts {
            self.trans_stmt(&stmt, sink)?;
        }

        Ok(())
    }
}

impl AutoUITrans {
    fn trans_stmt(&mut self, stmt: &Stmt, sink: &mut CodeSink) -> AutoResult<()> {
        match stmt {
            Stmt::TypeDecl(type_decl) => {
                self.trans_widget(type_decl, sink)?;
            }
            Stmt::Fn(fn_decl) => {
                self.trans_function(fn_decl, sink)?;
            }
            Stmt::Expr(expr) => {
                self.trans_app_expr(expr, sink)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn trans_widget(&mut self, type_decl: &TypeDecl, sink: &mut CodeSink) -> AutoResult<()> {
        let widget_name = &type_decl.name;
        self.current_widget = Some(widget_name.to_string());

        sink.writeln(&format!("// Widget: {}", widget_name));
        sink.writeln("#[derive(Debug)]");
        sink.writeln(&format!("pub struct {} {{", widget_name));
        sink.indent();

        for member in &type_decl.members {
            let field_name = &member.name;
            let field_type = self.rust_type_name(&member.ty);
            sink.writeln(&format!("pub {}: {},", field_name, field_type));
        }

        sink.dedent();
        sink.writeln("}");
        sink.writeln("");

        self.trans_component_impl(type_decl, sink)?;

        for method in &type_decl.methods {
            self.trans_method(method, sink)?;
        }

        self.current_widget = None;
        self.messages.clear();
        Ok(())
    }

    fn trans_component_impl(&mut self, type_decl: &TypeDecl, sink: &mut CodeSink) -> AutoResult<()> {
        let widget_name = &type_decl.name;

        sink.writeln(&format!("impl Component for {} {{", widget_name));
        sink.indent();
        sink.writeln("type Msg = ();");
        sink.writeln("");
        sink.writeln("fn on(&mut self, _msg: Self::Msg) {}");
        sink.dedent();
        sink.writeln("}");
        sink.writeln("");
        Ok(())
    }

    fn trans_method(&mut self, method: &Fn, sink: &mut CodeSink) -> AutoResult<()> {
        let widget_name = self.current_widget.as_ref().unwrap();

        match method.name.as_str() {
            "view" => {
                sink.writeln("fn view(&self) -> View<()> {");
                sink.indent();
                self.trans_body(&method.body, sink)?;
                sink.dedent();
                sink.writeln("}");
                sink.writeln("");
            }
            "on" => {
                sink.writeln("fn on(&mut self, msg: ()) {");
                sink.indent();
                self.trans_body(&method.body, sink)?;
                sink.dedent();
                sink.writeln("}");
                sink.writeln("");
            }
            _ => {
                sink.writeln(&format!("pub fn {}(&self) {{", method.name));
                sink.indent();
                self.trans_body(&method.body, sink)?;
                sink.dedent();
                sink.writeln("}");
                sink.writeln("");
            }
        }
        Ok(())
    }

    fn trans_body(&mut self, body: &Body, sink: &mut CodeSink) -> AutoResult<()> {
        for stmt in &body.stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    if let Ok(code) = self.trans_view_expr(expr) {
                        sink.writeln(&code);
                    } else {
                        sink.writeln(&format!("/* expr */"));
                    }
                }
                Stmt::Is(_) => {
                    sink.writeln("match msg { _ => {} }");
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn trans_view_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Node(node) => self.trans_view_node(node),
            Expr::Ident(name) => Ok(format!("View::text(\"{}\")", name)),
            _ => Ok("/* expr */".to_string()),
        }
    }

    fn trans_view_node(&mut self, node: &Node) -> Result<String, String> {
        match node.name.as_str() {
            "col" => Ok("View::col().build()".to_string()),
            "row" => Ok("View::row().build()".to_string()),
            "button" => Ok("View::button(\"Button\", ())".to_string()),
            "text" => Ok("View::text(\"text\")".to_string()),
            "input" => Ok("View::input(\"\").build()".to_string()),
            "center" => Ok("View::container(View::empty()).center().build()".to_string()),
            _ => Ok(format!("/* widget: {} */", node.name)),
        }
    }

    fn trans_function(&mut self, fn_decl: &Fn, sink: &mut CodeSink) -> AutoResult<()> {
        sink.writeln(&format!("pub fn {}() {{", fn_decl.name));
        sink.indent();
        self.trans_body(&fn_decl.body, sink)?;
        sink.dedent();
        sink.writeln("}");
        sink.writeln("");
        Ok(())
    }

    fn trans_app_expr(&mut self, _expr: &Expr, sink: &mut CodeSink) -> AutoResult<()> {
        sink.writeln("// App definition");
        Ok(())
    }

    fn rust_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "i32".to_string(),
            Type::Str(_) => "String".to_string(),
            Type::Bool => "bool".to_string(),
            Type::User(user) => user.name.to_string(),
            _ => "/* TODO */".to_string(),
        }
    }
}
