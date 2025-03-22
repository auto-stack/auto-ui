mod templates;

pub use templates::*;

use auto_lang::trans::Trans;
use auto_lang::ast::{Code, Stmt, Fn, TypeDecl, Expr};
use std::io::Write;
use auto_lang::AutoResult;
use auto_atom::Atom;
use auto_gen::{AutoGen, Mold};
use auto_val::{AutoStr, AutoPath, Type, Value, Op};
use auto_lang::ast;
use auto_lang::eval;
use auto_val::Shared;
use auto_lang::Universe;
use auto_lang::scope::Meta;
use auto_lang::ast::StoreKind;

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
            let kind = match &field.ty {
                Type::Str => "SharedString".to_string(),
                Type::User(typ) => {
                    if typ == "input" {
                        "TextInput".to_string()
                    } else {
                        typ.clone()
                    }
                }
                _ => field.ty.to_string(),
            };
            field_node.set_prop("value", if field.value.is_nil() { Value::str("") } else { field.value.clone() });
            let init_code = match &field.ty {
                Type::User(typ) => {
                    if typ == "input" {
                        format!("{}: cx.new(|cx| {}::new(w, cx)),", field.name.clone(), kind.clone())
                    } else {
                        format!("{}: {}::new(\"{}\"),", field.name.clone(), kind.clone(), field.value.to_astr())
                    }
                }
                _ => format!("{}: {}::new(\"{}\"),", field.name.clone(), kind.clone(), field.value.to_astr()),
            };
            field_node.set_prop("init_code", init_code);
            if kind == "TextInput" {
                field_node.set_prop("kind", "Entity<TextInput>".to_string());
            } else {
                field_node.set_prop("kind", kind.clone());
            }
            root.add_kid(field_node);
        }
        root.set_prop("methods", self.methods.clone());
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

pub struct GpuiTrans {
    pub widget: WidgetInfo,
    pub app: Option<AppInfo>,
    pub universe: Shared<Universe>,
}

impl Trans for GpuiTrans {
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

pub trait Indent {
    fn indent(&self) -> String;
}

impl Indent for String {
    fn indent(&self) -> String {
        self.replace("\n", "\n    ")
    }
}

impl GpuiTrans {
    pub fn new(universe: Shared<Universe>) -> Self {
        Self {
            widget: WidgetInfo::default(),
            app: None,
            universe,
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

        let mut code_header: String = "".to_string();
        for field in &self.widget.model.fields {
            if field.ty == Type::User("input".to_string()) {
                let name = field.name.clone();
                let text_name = name.split("_").last().unwrap();
                code_header.push_str(&format!("self.{} = self.{}.read(cx).text();", text_name, name));
            }
        }

        let code = format!("fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {{ {} {} }}", code_header, code);
        println!("code: {}", code);
        let pretty = self.pretty(&code);
        Ok(pretty)
    }

    fn do_on(&mut self, method: &ast::Fn) -> AutoResult<AutoStr> {
        println!("do_on");
        let sig = self.do_sig(method)?;
        let body = self.do_method_body(method)?;
        let code = format!("{} {}", sig, body);
        let pretty = self.pretty(&code);
        Ok(pretty)
    }

    fn do_method_body(&mut self, method: &ast::Fn) -> AutoResult<AutoStr> {
        println!("do_method_body");
        let mut body = String::new();
        body.push_str("{");
        for (i, stmt) in method.body.stmts.iter().enumerate() {
            let code = self.do_stmt(stmt)?;
            body.push_str(&code);
            if i < method.body.stmts.len() - 1 {
                body.push_str(";");
            }
        }
        if let ast::Type::Unknown = method.ret {
            body.push_str(";");
        } 
        body.push_str("}");
        println!("Got body: {}", body);
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
                code.push_str(&self.do_expr(expr)?);
            }
            _ => {
                println!("unknown stmt: {:?}", stmt);
            }
        }
        println!("got code: {}", code);
        Ok(code.into())
    }

    fn do_expr(&mut self, expr: &ast::Expr) -> AutoResult<AutoStr> {
        println!("do_expr");
        println!("expr: {:?}", expr);
        let mut code = String::new();
        let value = match expr {
            ast::Expr::Bina(lhs, op, rhs) => {
                let lhs_code = self.do_expr(lhs)?;
                let rhs_code = self.do_expr(rhs)?;
                format!("{} {} {}", lhs_code, op.op(), rhs_code)
            }
            ast::Expr::Ident(name) => {
                println!("name: {}", name.text.clone());
                // TODO: get meta from universe, and check if the name is a field or a local store
                let meta = self.universe.borrow().lookup_meta(name.text.as_str());
                let Some(meta) = meta else {
                    return Err("meta not found".into());
                };
                let Meta::Store(store) = meta.as_ref() else {
                    return Err("meta is not a store".into());
                };
                let out_name = match store.kind {
                    StoreKind::Field => {
                        format!("self.{}", store.name.text)
                    }
                    _ => {
                        format!("{}", store.name.text)
                    }
                };
                out_name.to_string()
            }
            ast::Expr::FStr(fstr) => {
                self.do_fstr(fstr)?.to_string()
            }
            ast::Expr::Str(s) => {
                format!("\"{}\".into()", s)
            }
            _ => {expr.to_code()}
        };
        code.push_str(&value);
        println!("got expr code: {}", code);
        Ok(code.into())
    }

    fn do_fstr(&mut self, fstr: &ast::FStr) -> AutoResult<AutoStr> {
        println!("do_fstr");
        let mut code = String::new();
        let mut vars = vec![];
        code.push_str("format!(\"");
        for part in fstr.parts.iter() {
            match part {
                ast::Expr::Str(s) => {
                    code.push_str(s);
                }
                ast::Expr::Ident(_) => {
                    let name = self.do_expr(part)?;
                    vars.push(name);
                    code.push_str("{}");
                }
                _ => {}
            }
        }
        code.push_str("\"");
        for var in vars {
            code.push_str(&format!(", {}", var));
        }
        code.push_str(").into()");
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
            "input" => {
                let input_code = self.do_input(node)?;
                code.push_str(&input_code);
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
            if let ast::Arg::Pos(pos) = arg {
                if let ast::Expr::Bina(lhs, op, rhs) = pos {
                    if let Op::Dot = op {
                        let arg = lhs.to_code();
                        let field = rhs.to_code();
                        args.push(format!("{}.{}.clone()", arg, field));
                        continue;
                    }
                }
            }
            // else
            args.push(arg.to_code());
        }
        let code = args.join(", ");
        Ok(code.into())
    }

    fn do_input(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        let mut code = String::new();
        if node.args.len() != 1 {
            return Err(format!("input expects 1 argument, got {}", node.args.len()).into());
        }
        let arg = &node.args.args[0];
        let ast::Arg::Pos(pos) = arg else {
            return Err("input expects 1 positional argument with identifier".into());
        };
        let name = format!("input_{}", pos.to_code());
        code.push_str(format!("self.{}.clone()", name).as_str());
        self.widget.model.fields.push(WidgetField {
            name: name.into(),
            ty: Type::User("input".to_string()),
            value: Value::str(""),
        });
        Ok(code.into())
    }

    fn gen(&mut self) -> AutoResult<()> {
        println!("gen");
        let mut story_node = auto_val::Node::new("story");
        let Some(app) = &self.app else {
            return Ok(());
        };

        story_node.add_kid(self.widget.to_node());
        story_node.add_kid(app.to_node());
        story_node.set_prop("name", self.widget.name.clone());
        let atom = Atom::node(story_node);
        println!("{}", atom);

        // 3. feed atom to generator and generate code
        let story_mold = Mold::new("story.at.rs", Templates::story().unwrap());
        let outpath = AutoPath::crate_root().join("examples/");
        println!("outpath: {}", outpath.to_astr());
        let gen = AutoGen::new()
            .molds(vec![story_mold])
            .data(atom)
            .out(outpath);
        let result = gen.gen();
        println!("{}", result);
    
        Ok(())
    }

    fn pretty(&mut self, code: impl Into<AutoStr>) -> AutoStr {
        let parsed = syn::parse_str(code.into().as_str()).unwrap();
        let pretty = prettyplease::unparse(&parsed).indent();
        pretty.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_lang::{parse, parse_with_scope};
    use auto_val::shared;
    use auto_lang::Universe;
    use std::rc::Rc;
    use auto_lang::scope::Meta;
    use auto_lang::ast::{Type, TypeDecl};
    #[test]
    fn test_ui_hello() {
        let code = r#"
type Hello as View {
    msg str = "Hello World"
    button_label str = "Click"

    fn view() {
        center {
            col {
                label(self.msg) {}
                button(self.button_label) {
                    onclick: "button-clicked"
                }
            }
        }
    }

    fn on(ev str) {
        msg = `Hello Button clicked`
    }
}

fn main() {
    app("Hello Example") {
        hello() {}
    }
}
        "#;
        let universe = shared(Universe::new());
        let mut trans = GpuiTrans::new(universe.clone());
        let mut out = Vec::new();
        let ast = parse_with_scope(code, universe.clone()).unwrap();
        trans.trans(ast, &mut out).unwrap();
        println!("{}", String::from_utf8(out).unwrap());
    }

    #[test]
    fn test_ui_login() {
        let code = r#"
type input {
    text str = ""
}

type Login as View {
    username str
    password str
    status str = ""

    fn view() {
        center {
            col {
                label("Username") {}
                input(username) {}
                label("Password") {}
                input(password) {}
                button("Login") {
                    onclick: "button-login"
                }
                label(self.username) {}
                label(self.status) {}
            }
        }
    }

    fn on(ev str) {
        status = `Login ${username}`
    }
}

fn main() {
    app("Login Example") {
        login() {}
    }
}
        "#;
        let universe = shared(Universe::new());
        let mut trans = GpuiTrans::new(universe.clone());
        let mut out = Vec::new();
        let ast = parse_with_scope(code, universe).unwrap();
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
        let universe = shared(Universe::new());
        let mut trans = GpuiTrans::new(universe.clone());
        let ast = parse_with_scope(code, universe).unwrap();
        trans.do_method(&ast.stmts[0].as_fn().unwrap()).unwrap();
        println!("{}", trans.widget.methods[0]);
    }
}