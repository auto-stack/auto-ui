mod templates;

pub use templates::*;

use auto_lang::trans::Trans;
use auto_lang::ast::{Code, Stmt, Fn, TypeDecl, Expr};
use std::io::Write;
use auto_lang::AutoResult;
use auto_atom::Atom;
use auto_gen::{AutoGen, Mold};
use auto_val::{AutoStr, StrExt, AutoPath, Type, Value, Op};
use auto_lang::ast;
use auto_lang::eval;
use auto_val::Shared;
use auto_lang::Universe;
use auto_lang::scope::Meta;
use auto_lang::ast::StoreKind;
use std::rc::Rc;

#[derive(Debug, Default, Clone)]
pub struct WidgetInfo {
    pub name: AutoStr,
    pub model: WidgetModel,
    pub view: AutoStr,
    pub methods: Vec<AutoStr>,
}

#[derive(Debug, Default, Clone)]
pub struct WidgetModel {
    pub fields: Vec<WidgetField>,
}

#[derive(Debug, Default, Clone)]
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
                _ => format!("{}: {}::new(r#\"{}\"#),", field.name.clone(), kind.clone(), field.value.to_astr()),
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
    // currently only support one widget in each side
    pub left: AutoStr,
    pub right: AutoStr,
    pub middle: AutoStr,
    pub bottom: AutoStr,
    pub top: AutoStr,
}

impl AppInfo {
    pub fn new(title: impl Into<AutoStr>) -> Self {
        Self { title: title.into(), left: AutoStr::default(), right: AutoStr::default(), middle: AutoStr::default(), bottom: AutoStr::default(), top: AutoStr::default() }
    }

    pub fn to_node(&self) -> auto_val::Node {
        let mut root = auto_val::Node::new("app");
        root.set_prop("title", self.title.clone());
        root.set_prop("left", self.left.clone());
        root.set_prop("right", self.right.clone());
        root.set_prop("middle", self.middle.clone());
        root.set_prop("bottom", self.bottom.clone());
        root.set_prop("top", self.top.clone());
        root
    }
}

pub struct GpuiTrans {
    pub name: AutoStr,
    pub widget: WidgetInfo,
    pub widgets: Vec<WidgetInfo>,
    pub app: Option<AppInfo>,
    pub universe: Shared<Universe>,
}

impl Trans for GpuiTrans {
    fn trans(&mut self, ast: Code, out: &mut impl Write) -> AutoResult<()> {
        for stmt in ast.stmts.into_iter() {
            match stmt {
                Stmt::Store(store) => {
                    self.do_store(&store)?;
                }
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
    pub fn new(name: impl Into<AutoStr>, universe: Shared<Universe>) -> Self {
        Self {
            name: name.into(),
            widget: WidgetInfo::default(),
            widgets: vec![],
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
                self.do_app(node)?;
            }
        }
        Ok(())
    }

    fn do_app(&mut self, node: &ast::Node) -> AutoResult<()> {
        let title = node.args.args[0].repr();
        let mut app = AppInfo::new(title);
        // check sides
        for kid in &node.body.stmts {
            match kid {
                ast::Stmt::Node(node) => {
                    let name = AutoStr::from(node.name.text.clone());
                    println!("app node: {}", name);
                    match name.as_str() {
                        "left" => {
                            app.left = self.get_first_kid_name(node)?;
                        }
                        "right" => {
                            app.right = self.get_first_kid_name(node)?;
                        }
                        "middle" => {
                            app.middle = self.get_first_kid_name(node)?;
                        }
                        "bottom" => {
                            app.bottom = self.get_first_kid_name(node)?;
                        }
                        "top" => {
                            app.top = self.get_first_kid_name(node)?;
                        }
                        _ => {
                            app.middle = name.to_camel();
                        }
                    }
                }
                _ => {}
            }
        }
        self.app = Some(app);
        Ok(())
    }

    fn do_store(&mut self, store: &ast::Store) -> AutoResult<()> {
        println!("do_store");
        let var = store.name.text.clone();
        let value = self.do_rhs_expr(&store.expr)?;
        self.universe.borrow_mut().set_local_val(var.as_str(), value);
        Ok(())
    }

    fn get_first_kid_name(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        let stmt = &node.body.stmts[0];
        match stmt {
            ast::Stmt::Node(node) => {
                let name = AutoStr::from(node.name.text.clone()).to_camel();
                Ok(name)
            }
            _ => Err("first kid is not a node".into()),
        }
    }

    fn do_fn(&mut self, _fn: &Fn) -> AutoResult<()> {
        println!("do_fn");
        Ok(())
    }

    fn do_type_decl(&mut self, type_decl: &TypeDecl) -> AutoResult<()> {
        println!("do_type_decl");
        if has_view(type_decl) { // View types
            self.do_widget(type_decl)?;
            
        } else { // Normal types
            println!("no view");
        }
        Ok(())
    }

    fn do_widget(&mut self, type_decl: &TypeDecl) -> AutoResult<()> {
        let mut widget_info = WidgetInfo::new();
        widget_info.name = type_decl.name.clone().into();
        widget_info.model = self.do_model(&type_decl.members)?;
        self.widget = widget_info;
        for method in type_decl.methods.iter() {
            self.do_method(method)?;
            println!("widget view now: {}", self.widget.view.clone());
        }
        println!("widget info: {}", self.widget.view.clone());
        // widget completed, add to widgets
        self.widgets.push(self.widget.clone());
        self.widget = WidgetInfo::default();
        Ok(())
    }

    fn do_model(&mut self, members: &Vec<ast::Member>) -> AutoResult<WidgetModel> {
        let mut model = WidgetModel::new();
        for member in members {
            model.fields.push(self.do_field(member)?);  
        }
        Ok(model)
    }

    fn do_field(&mut self, member: &ast::Member) -> AutoResult<WidgetField> {
        let mut field = WidgetField::default();
        field.name = member.name.clone().into();
        field.ty = member.ty.clone().into();
        field.value = match &member.value {
            Some(value) => self.do_rhs_expr(value)?,
            None => Value::Nil,
        };
        Ok(field)
    }

    fn do_rhs_expr(&mut self, expr: &ast::Expr) -> AutoResult<Value> {
        match expr {
            ast::Expr::Ident(name) => {
                let val = self.universe.borrow().lookup_val(name.text.as_str());
                let Some(val) = val else {
                    return Err("value not found".into());
                };
                Ok(val)
            }
            ast::Expr::Node(node) => {
                if node.name.text == "markdown" {
                    if node.body.stmts.len() != 1 {
                        return Err(format!("markdown node should have exactly one statement, but got {}", node.body.stmts.len()).into());
                    }
                    let stmt = &node.body.stmts[0];
                    match stmt {
                        ast::Stmt::Expr(ast::Expr::Str(s)) => {
                            return Ok(Value::Str(s.clone().into()));
                        }
                        _ => {
                            return Err(format!("markdown node should have exactly one statement, but got {}", node.body.stmts.len()).into());
                        }
                    }
                } else {
                    return Err(format!("unsupported node {}", node.name.text).into());
                }
            }
            _ => {
                Ok(eval::eval_basic_expr(expr))
            }
        }
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
                self.do_store(store)?;
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
            "center" | "row" | "col" | "form" | "field" => {
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
        let mut is_wrap = false;
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
            "form" => {
                code.push_str("row().w_begin().child(v_form()");
                is_wrap = true;
            }
            "field" => {
                // get main_arg
                if node.args.len() != 0 {
                    let main_arg = &node.args.args[0].repr();
                    code.push_str(&format!("form_field().label(\"{}\")", main_arg));
                } else {
                    code.push_str("form_field()");
                }
            }
            _ => {}
        }
        // do kids
        for kid in &node.body.stmts {
            match kid {
                ast::Stmt::Node(kid_node) => {
                    if kid_node.name.text == "style" {
                        let style_code = self.do_style(kid_node)?;
                        code.push_str(&style_code);
                    } else {
                        code.push_str(".child(");
                        let kid_code = self.do_node(kid_node)?;
                        code.push_str(&kid_code);
                        code.push_str(")");
                    }
                }
                ast::Stmt::Expr(expr) => {
                    match expr {
                        ast::Expr::Pair(pair) => {
                            let prop_code = self.do_prop(pair)?;
                            code.push_str(&prop_code);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        if is_wrap {
            code.push_str(")");
        }
        Ok(code.into())
    }

    fn do_style(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        // TODO: import real theme to scope
        // self.universe.borrow_mut().define("theme", std::rc::Rc::new(Meta::Ref(ast::Name::new("theme"))));
        println!("do_style");
        let mut code = String::new();
        // Style node should contain these forms:
        // 1. key: value
        // 2. key
        println!("node: {:?}", node);
        for prop in &node.body.stmts {
            if let ast::Stmt::Expr(expr) = prop {
                match expr {
                    ast::Expr::Ident(ident) => {
                        code.push_str(&format!(".{}()", ident.text));
                    }
                    ast::Expr::Str(s) => {
                        code.push_str(&format!(".{}()", s));
                    }
                    ast::Expr::Pair(pair) => {
                        let key = pair.key.name().unwrap();
                        code.push_str(&format!(".{}(", key));
                        match &*pair.value {
                            ast::Expr::Str(s) => {
                                code.push_str(&format!(".{}()", s));
                            }
                            ast::Expr::Bina(lhs, op, rhs) => {
                                let lhs_code = lhs.to_code();
                                let rhs_code = rhs.to_code();
                                if let Op::Dot = op {
                                    if lhs_code == "theme" {
                                        code.push_str(&format!("cx.theme().{}", rhs_code));
                                    }
                                } 
                            }
                            _ => {}
                        }
                        code.push_str(")");
                    }
                    _ => {}
                }
            }
        }
        Ok(code.into())
    }

    fn do_prop(&mut self, prop: &ast::Pair) -> AutoResult<AutoStr> {
        println!("do_prop");
        let mut code = String::new();
        match prop.key.name() {
            Some("id") => {
                let id = prop.value.to_code();
                code.push_str(&format!(".id({})", id));
            }
            _ => {}
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
            "markdown" => {
                let markdown_code = self.do_markdown(node)?;
                code.push_str(&markdown_code);
            }
            _ => {}
        }
        Ok(code.into())
    }

    fn do_markdown(&mut self, node: &ast::Node) -> AutoResult<AutoStr> {
        let mut code = String::new();
        code.push_str("Label::new(");
        let args = self.do_args(node)?;
        code.push_str(&args);
        code.push_str(")");
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
        let mut app_node = auto_val::Node::new("app");
        let Some(app) = &self.app else {
            return Ok(());
        };

        for widget in &self.widgets {
            println!("widget view: {}", widget.view.clone());
            app_node.add_kid(widget.to_node());
        }

        app_node.add_kid(app.to_node());
        app_node.set_prop("name", self.name.clone());
        let atom = Atom::node(app_node);
        println!("{}", auto_val::pretty(&atom.to_string().as_str(), 2));

        // 3. feed atom to generator and generate code
        let app_mold = Mold::new("app.at.rs", Templates::app().unwrap());
        let outpath = AutoPath::crate_root().join("examples/");
        println!("outpath: {}", outpath.to_astr());
        let gen = AutoGen::new()
            .molds(vec![app_mold])
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