use auto_lang::trans::Trans;
use auto_lang::ast::{Code, Stmt, Fn, TypeDecl, Expr};
use std::io::Write;
use auto_lang::AutoResult;
use auto_atom::Atom;
use auto_gen::{AutoGen, Mold};
use auto_val::{AutoStr, AutoPath, Type, Value};
use auto_lang::ast;
use auto_lang::eval;

#[derive(Debug, Default)]
pub struct WidgetInfo {
    pub name: AutoStr,
    pub model: WidgetModel,
    pub view: AutoStr,
    pub methods: Vec<AutoStr>,
}

#[derive(Debug, Default)]
pub struct WidgetModel {
    pub fields: Vec<WidgetField>,
}

#[derive(Debug, Default)]
pub struct WidgetField {
    pub name: AutoStr,
    pub ty: Type,
    pub value: Value,
}

impl WidgetInfo {
    fn new() -> Self {
        Self { name: AutoStr::new(), model: WidgetModel::new(), view: AutoStr::new(), methods: vec![] }
    }
}

impl WidgetModel {
    fn new() -> Self {
        Self { fields: vec![] }
    }
}

impl From<&Vec<ast::Member>> for WidgetModel {
    fn from(members: &Vec<ast::Member>) -> Self {
        let fields = members.iter().map(|member| WidgetField::from(member)).collect();
        WidgetModel { fields }
    }
}

impl From<&ast::Member> for WidgetField {
    fn from(member: &ast::Member) -> Self {
        let name = member.name.clone().into();
        let ty = member.ty.clone().into();
        let value = match &member.value {
            Some(value) => eval::eval_basic_expr(&value),
            None => Value::Nil,
        };
        WidgetField { name, ty, value }
    }
}

impl WidgetInfo {
    pub fn to_node(&self) -> auto_val::Node {
        let mut root = auto_val::Node::new("story");
        root.set_prop("name", self.name.clone());
        for field in &self.model.fields {
            let mut field_node = auto_val::Node::new("field");
            field_node.add_arg(auto_val::Arg::Pos(field.name.clone().into()));
            field_node.set_prop("name", field.name.clone());
            let kind = match field.ty {
                Type::Str => "SharedString".to_string(),
                _ => field.ty.to_string(),
            };
            field_node.set_prop("kind", kind);
            field_node.set_prop("value", field.value.clone());
            root.add_kid(field_node);
        }
        root.set_prop("methods", self.methods.clone());
        // code: "center().child(Label::new(self.msg.clone()))"
        // evaluate fn view() code into Rust code
        // let view_node = &self.view.root;
        // if view_node.name.text == "center" {
        //     code.push_str("center().child(");
        //     for kid in &view_node.body.stmts {
        //         if let ast::Stmt::Node(kid_node) = kid {
        //             if kid_node.name.text == "label" {
        //                 code.push_str("Label::new(");
        //                 kid_node.args.args.iter().for_each(|arg| {
        //                     match arg {
        //                         auto_lang::ast::Arg::Pos(arg) => {
        //                             let mut arg = arg.repr();
        //                             if arg == "self.msg" {
        //                                 arg = "self.msg.clone()".to_string();
        //                             }
        //                             code.push_str(&arg);

        //                         }
        //                         _ => {}
        //                     }
        //                 });
        //                 code.push_str(")");
        //             }
        //         }
        //     }
        //     code.push_str(")");
        // }
        // println!("got code: {}", code);
        root.set_prop("code", Value::Str(self.view.clone()));
        root
    }
}

pub struct AppInfo {
    pub title: AutoStr,
}

impl AppInfo {
    pub fn to_node(&self) -> auto_val::Node {
        let mut root = auto_val::Node::new("app");
        root.set_prop("title", self.title.clone());
        root
    }
}

pub struct UITrans {
    pub widget: WidgetInfo,
    pub app: Option<AppInfo>,
}

impl Trans for UITrans {
    fn trans(&mut self, ast: Code, out: &mut impl Write) -> AutoResult<()> {
        for stmt in ast.stmts.into_iter() {
            match stmt {
                Stmt::Fn(fn_stmt) => {
                    if fn_stmt.name.text == "main" {
                        self.do_main(&fn_stmt)?;
                    } else {
                        self.do_fn(&fn_stmt)?;
                    }
                }
                Stmt::TypeDecl(type_decl) => {
                    self.do_type_decl(&type_decl)?;
                }
                _ => {}
            }
        }

        // Trans to rust code
        self.gen()?;

        Ok(())
    }
}

fn has_view(type_decl: &TypeDecl) -> bool {
    type_decl.methods.iter().any(|m| m.name.text == "view")
}

impl UITrans {
    fn new() -> Self {
        Self {
            widget: WidgetInfo::default(),
            app: None,
        }
    }

    fn do_main(&mut self, fn_stmt: &Fn) -> AutoResult<()> {
        let body = &fn_stmt.body;
            let last_stmt = body.stmts.last().unwrap();
            if let auto_lang::ast::Stmt::Node(node) = last_stmt {
                // convert app code to rust code
                if node.name.text == "app" {
                    let title = node.args.args[0].repr();
                    self.app = Some(AppInfo { title: title.into() });
            }
        }
        Ok(())
    }

    fn do_fn(&mut self, _fn: &Fn) -> AutoResult<()> {
        println!("do_fn");
        Ok(())
    }

    fn do_type_decl(&mut self, type_decl: &TypeDecl) -> AutoResult<()> {
        println!("do_type_decl");
        if has_view(type_decl) { // View types
            println!("has view");
            let mut widget_info = WidgetInfo::new();
            widget_info.name = type_decl.name.clone().into();
            widget_info.model = WidgetModel::from(&type_decl.members);
            self.widget = widget_info;
            for method in type_decl.methods.iter() {
                self.do_method(method)?;
            }
        } else { // Normal types
            println!("no view");
        }
        Ok(())
    }

    fn do_method(&mut self, method: &ast::Fn) -> AutoResult<()> {
        println!("do_method");
        match method.name.text.as_str() {
            "view" => {
                let code = self.do_view(method)?;
                self.widget.view = code;
            }
            "on" => {
                let code = self.do_on(method)?;
                self.widget.methods.push(code);
            }
            _ => {}
        };
        Ok(())
    }



    fn do_view(&mut self, method: &ast::Fn) -> AutoResult<AutoStr> {
        let mut code: String = "".to_string();
        let view_node = &method.body.stmts.last();
        if let Some(view_node) = view_node {
            if let auto_lang::ast::Stmt::Node(node) = view_node {
                let view_code = self.do_node(node)?;
                code.push_str(&view_code);
            }
        }
        Ok(code.into())
    }

    fn do_on(&mut self, method: &ast::Fn) -> AutoResult<AutoStr> {
        println!("do_on");
        let sig = self.do_sig(method)?;
        let body = self.do_method_body(method)?;
        let code = format!("{} {}", sig, body);
        // self.methods.push(code.into());
        Ok(code.into())
    }

    fn do_method_body(&mut self, method: &ast::Fn) -> AutoResult<AutoStr> {
        println!("do_method_body");
        let mut body = String::new();
        body.push_str("{");
        for stmt in &method.body.stmts {
            let code = self.do_stmt(stmt)?;
            body.push_str(&code);
        }
        body.push_str("}");
        Ok(body.into())
    }

    fn do_stmt(&mut self, stmt: &ast::Stmt) -> AutoResult<AutoStr> {
        println!("do_stmt");
        let mut code = String::new();
        match stmt {
            ast::Stmt::Node(node) => {
                code.push_str(&self.do_node(node)?);
            }
            ast::Stmt::Store(store) => {
                code.push_str(&self.do_store(store)?);
            }
            ast::Stmt::Expr(expr) => {
                code.push_str(&expr.to_code());
            }
            _ => {
                println!("unknown stmt: {:?}", stmt);
            }
        }
        Ok(code.into())
    }

    fn do_store(&mut self, store: &ast::Store) -> AutoResult<AutoStr> {
        println!("do_store");
        let mut code = String::new();
        let var = store.name.text.clone();
        let value = store.expr.to_code();
        code.push_str(&format!("{} = {}", var, value));
        Ok(code.into())
    }

    fn do_sig(&mut self, method: &ast::Fn) -> AutoResult<AutoStr> {
        println!("do_sig");
        let mut sig = String::new();

        let fname = method.name.text.clone();

        sig.push_str(format!("pub fn {}", fname).as_str());
        sig.push_str("(&mut self, ");

        for (i, param) in method.params.iter().enumerate() {
            sig.push_str(&param.name.text);
            sig.push_str(": ");
            sig.push_str(self.do_type(&param.ty)?);
            if i < method.params.len() - 1 {
                sig.push_str(", ");
            }
        }

        sig.push_str(")"); // TODO: return type?
        
        Ok(sig.into())
    }

    fn do_type(&mut self, ty: &ast::Type) -> AutoResult<&str> {
        let typ = match ty {
            ast::Type::Str => "SharedString",
            ast::Type::Int => "u32",
            ast::Type::Bool => "bool",
            ast::Type::Char => "u8",
            ast::Type::Float => "f32",
            _ => "()",
        };
        Ok(typ)
    }

    fn do_node(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        let mut code = String::new();
        let name = node.name.text.clone();
        match name.as_str() {
            "center" | "row" | "col" => {
                let layout_code = self.do_layout(node)?;
                code.push_str(&layout_code);
            }
            _ => {
                let element_code = self.do_element(node)?;
                code.push_str(&element_code);
            }
        }
        Ok(code.into())
    }

    fn do_layout(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        println!("do_layout");
        let mut code = String::new();
        match node.name.text.as_str() {
            "center" => {
                code.push_str("center()");
            }
            "row" => {
                code.push_str("row()");
            }
            "col" => {
                code.push_str("col()");
            }
            _ => {}
        }
        for kid in &node.body.stmts {
            if let ast::Stmt::Node(kid_node) = kid {
                code.push_str(".child(");
                let kid_code = self.do_node(kid_node)?;
                code.push_str(&kid_code);
                code.push_str(")");
            }
        }
        Ok(code.into())
    }

    fn do_element(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        println!("do_element");
        let mut code = String::new();
        match node.name.text.as_str() {
            "label" => {
                code.push_str("Label::new(");
                let args = self.do_args(node)?;
                code.push_str(&args);
                code.push_str(")");
            }
            "button" => {
                let button_code = self.do_button(node)?;
                code.push_str(&button_code);
            }
            _ => {}
        }
        Ok(code.into())
    }

    fn do_button(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        let mut code = String::new();
        code.push_str("Button::new(");
        let args = self.do_args(node)?;
        code.push_str(&args);
        code.push_str(").label(");
        code.push_str(&args);
        code.push_str(")");

        // deal with onclick
        for stmt in &node.body.stmts {
            if let Stmt::Expr(Expr::Pair(pair)) = stmt {
                if let Some(name) = pair.key.name() {
                    if name == "onclick" {
                        let event = pair.value.to_code();
                        code.push_str(&format!(r#"
                        .on_click(cx.listener(|v, _, _, cx| {{
                            v.on({}.into());cx.notify();
                        }}))"#, event));
                    }
                }
            }
        }
        Ok(code.into())
    }

    fn do_args(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        let mut args = vec![];
        for arg in node.args.args.iter() {
            let mut arg = arg.to_code();
            if arg == "self.msg" {
                arg = "self.msg.clone()".to_string();
            }
            args.push(arg);
        }
        let code = args.join(", ");
        Ok(code.into())
    }

    fn gen(&mut self) -> AutoResult<()> {
        println!("gen");
        let mut story_node = auto_val::Node::new("story");
        if let Some(app) = &self.app {
            story_node.add_kid(self.widget.to_node());
            story_node.add_kid(app.to_node());
            story_node.set_prop("name", self.widget.name.clone());
            let atom = Atom::node(story_node);
            println!("{}", atom);

            // 3. feed atom to generator and generate code
            let gen = AutoGen::new()
                .molds(vec![Mold::from_file(AutoPath::new("../../assets/templates/story.at.rs"))])
                .data(atom)
                .out(AutoPath::new("../../crates/auto-ui/examples/"));
            let result = gen.gen();
            println!("{}", result);
        }
    
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_lang::parse;

    #[test]
    fn test_ui_trans() {
        let code = r#"
type Hello as View {
    msg str = "Hello World"

    fn view() {
        center {
            col {
                label(self.msg) {}
                button("Click me") {
                    onclick: "button-clicked"
                }
            }
        }
    }

    fn on(ev str) {
        self.msg = ev
    }
}

fn main() {
    app("Hello Example") {
        hello() {}
    }
}
        "#;
        let mut trans = UITrans::new();
        let mut out = Vec::new();
        let ast = parse(code).unwrap();
        trans.trans(ast, &mut out).unwrap();
        println!("{}", String::from_utf8(out).unwrap());
    }


    #[test]
    fn test_do_method() {
        let code = r#"
        fn on(msg str) {
            self.msg = "Clicked"
        }
        "#;
        let mut trans = UITrans::new();
        let ast = parse(code).unwrap();
        trans.do_method(&ast.stmts[0].as_fn().unwrap()).unwrap();
        println!("{}", trans.widget.methods[0]);
    }
}