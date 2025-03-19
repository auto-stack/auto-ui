use auto_lang::trans::Trans;
use auto_lang::ast::{Code, Stmt, Fn, TypeDecl};
use std::io::Write;
use auto_lang::AutoResult;
use auto_atom::Atom;
use auto_gen::{AutoGen, Mold};
use auto_val::{AutoStr, AutoPath, Type, Value};
use auto_lang::ast;
use auto_lang::ast::Name;
use auto_lang::eval;

#[derive(Debug)]
pub struct WidgetInfo {
    pub name: AutoStr,
    pub model: WidgetModel,
    pub view: AutoStr,
}

#[derive(Debug)]
pub struct WidgetModel {
    pub fields: Vec<WidgetField>,
}

#[derive(Debug)]
pub struct WidgetField {
    pub name: AutoStr,
    pub ty: Type,
    pub value: Value,
}

impl WidgetInfo {
    fn new() -> Self {
        Self { name: AutoStr::new(), model: WidgetModel::new(), view: AutoStr::new() }
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
    pub widget: Option<WidgetInfo>,
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
                    self.do_type(&type_decl)?;
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
            widget: None,
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

    fn do_type(&mut self, type_decl: &TypeDecl) -> AutoResult<()> {
        println!("do_type");
        if has_view(type_decl) { // View types
            println!("has view");
            let mut widget_info = WidgetInfo::new();
            widget_info.name = type_decl.name.clone().into();
            widget_info.model = WidgetModel::from(&type_decl.members);
            for method in type_decl.methods.iter() {
                self.do_method(method, &mut widget_info)?;
            }
            self.widget = Some(widget_info);
        } else { // Normal types
            println!("no view");
        }
        Ok(())
    }

    fn do_method(&mut self, method: &ast::Fn, widget_info: &mut WidgetInfo) -> AutoResult<()> {
        println!("do_method");
        if method.name.text != "view" {
            return Ok(());
        }
        let mut code: String = "".to_string();
        let view_node = &method.body.stmts.last().unwrap();
        if let auto_lang::ast::Stmt::Node(node) = view_node {
            if node.name.text == "center" {
                code.push_str("center().child(");
                for kid in &node.body.stmts {
                    if let ast::Stmt::Node(kid_node) = kid {
                        if kid_node.name.text == "label" {
                            code.push_str("Label::new(");
                            kid_node.args.args.iter().for_each(|arg| {
                                match arg {
                                    auto_lang::ast::Arg::Pos(arg) => {
                                        let mut arg = arg.repr();
                                        if arg == "self.msg" {
                                            arg = "self.msg.clone()".to_string();
                                        }
                                        code.push_str(&arg);
    
                                    }
                                    _ => {}
                                }
                            });
                            code.push_str(")");
                        }
                    }
                }
                code.push_str(")");
            }
            println!("got code: {}", code);
            widget_info.view = code.into();
        }
            
        Ok(())
    }

    fn gen(&mut self) -> AutoResult<()> {
        println!("gen");
        let mut story_node = auto_val::Node::new("story");
        if let Some(widget) = &self.widget {
            if let Some(app) = &self.app {
                story_node.add_kid(widget.to_node());
                story_node.add_kid(app.to_node());
                story_node.set_prop("name", widget.name.clone());
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
            label(self.msg) {}
        }
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
}