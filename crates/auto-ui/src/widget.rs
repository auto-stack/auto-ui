use std::fmt::Write;

use auto_val::{AutoStr, Type, Value};
use auto_lang::ast;
use auto_lang::ast::Node;
use auto_lang::ast::Name;
use auto_lang::eval;
#[derive(Debug)]
pub struct WidgetInfo {
    pub name: AutoStr,
    pub model: WidgetModel,
    pub view: WidgetView,
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

#[derive(Debug)]
pub struct WidgetView {
    pub root: Node,
}

impl From<&ast::TypeDecl> for WidgetInfo {
    fn from(decl: &ast::TypeDecl) -> Self {
        let name = decl.name.clone().into();
        let model = WidgetModel::from(&decl.members);
        let view = match decl.methods.iter().find(|m| m.name.text == "view") {
            Some(view_fn) => {
                let body = view_fn.body.clone();
                // parse body into node
                let last_stmt = body.stmts.last();
                if let Some(stmt) = last_stmt {
                    if let ast::Stmt::Node(node) = stmt {
                        node.clone()
                    } else {
                        Node::new(Name::new("view"))
                    }
                } else {
                    Node::new(Name::new("view"))
                }

            }
            None => Node::new(Name::new("view")),
        };
        WidgetInfo { name, model, view: WidgetView {root: view} }
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

impl From<WidgetInfo> for Node {
    fn from(info: WidgetInfo) -> Self {
        info.view.root
    }
}

impl WidgetInfo {
    pub fn to_node(&self, name: impl Into<AutoStr>) -> auto_val::Node {
        let mut root = auto_val::Node::new(name);
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
        let mut code: String = "".to_string();
        let view_node = &self.view.root;
        if view_node.name.text == "center" {
            code.write_str("center().child(");
            for kid in &view_node.body.stmts {
                if let ast::Stmt::Node(kid_node) = kid {
                    if kid_node.name.text == "label" {
                        code.write_str("Label::new(");
                        kid_node.args.args.iter().for_each(|arg| {
                            match arg {
                                auto_lang::ast::Arg::Pos(arg) => {
                                    let mut arg = arg.repr();
                                    if arg == "self.msg" {
                                        arg = "self.msg.clone()".to_string();
                                    }
                                    code.write_str(&arg);

                                }
                                _ => {}
                            }
                        });
                        code.write_str(")");
                    }
                }
            }
            code.write_str(")");
        }
        println!("got code: {}", code);
        root.set_prop("code", Value::Str(code.into()));
        root
    }
}