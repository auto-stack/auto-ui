use crate::ast::{
    ActionsBlock, ExtImport, ModelBlock, MsgDecl, StoreDecl, StyleRecipeDecl, ViewBlock,
    ViewFragmentDecl, WidgetDecl,
};
use auto_ast::ast::{AtomWriter, ToAtom, ToAtomStr, ToNode};
use auto_parser_api::{DomainCode, DomainStmt};
use auto_val::{AutoStr, Node as AutoNode, Value};
use std::fmt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UiStmt {
    WidgetDecl(WidgetDecl),
    StoreDecl(StoreDecl),
    ActionsDecl(ActionsBlock),
    ViewFragmentDecl(ViewFragmentDecl),
    StyleRecipeDecl(StyleRecipeDecl),
    MsgDecl(MsgDecl),
    ModelBlock(ModelBlock),
    ViewBlock(ViewBlock),
    UseWeb(Vec<ExtImport>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UiCode(pub DomainCode<UiStmt>);

impl UiCode {
    pub fn new(items: Vec<DomainStmt<UiStmt>>) -> Self {
        Self(DomainCode {
            items,
            source_lines: Vec::new(),
            file_attrs: Vec::new(),
        })
    }

    pub fn from_domain(stmts: Vec<UiStmt>) -> Self {
        Self::new(stmts.into_iter().map(DomainStmt::Domain).collect())
    }

    pub fn from_core(stmts: Vec<auto_ast::ast::Stmt>) -> Self {
        Self::new(stmts.into_iter().map(DomainStmt::Core).collect())
    }

    pub fn items(&self) -> &[DomainStmt<UiStmt>] {
        &self.0.items
    }

    pub fn core_stmts(&self) -> impl Iterator<Item = &auto_ast::ast::Stmt> {
        self.0.items.iter().filter_map(|item| match item {
            DomainStmt::Core(stmt) => Some(stmt),
            DomainStmt::Domain(_) => None,
        })
    }

    pub fn domain_stmts(&self) -> impl Iterator<Item = &UiStmt> {
        self.0.items.iter().filter_map(|item| match item {
            DomainStmt::Domain(stmt) => Some(stmt),
            DomainStmt::Core(_) => None,
        })
    }
}

impl fmt::Display for UiStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WidgetDecl(widget) => write!(f, "(widget {})", widget.name),
            Self::StoreDecl(store) => write!(f, "(store {})", store.name),
            Self::ActionsDecl(_) => write!(f, "(actions)"),
            Self::ViewFragmentDecl(fragment) => write!(f, "(view fn {})", fragment.name),
            Self::StyleRecipeDecl(recipe) => write!(f, "(style-recipe {})", recipe.name),
            Self::MsgDecl(msg) => write!(f, "(msg {} variants)", msg.variants.len()),
            Self::ModelBlock(model) => write!(f, "(model {} fields)", model.fields.len()),
            Self::ViewBlock(_) => write!(f, "(view)"),
            Self::UseWeb(entries) => write!(
                f,
                "use.web ({} web import{})",
                entries.len(),
                if entries.len() == 1 { "" } else { "s" }
            ),
        }
    }
}

impl ToNode for UiStmt {
    fn to_node(&self) -> AutoNode {
        let node = match self {
            Self::WidgetDecl(widget) => {
                let mut node = AutoNode::new("widget");
                node.add_arg(auto_val::Arg::Pos(Value::str(widget.name.as_str())));
                node
            }
            Self::StoreDecl(store) => {
                let mut node = AutoNode::new("store");
                node.add_arg(auto_val::Arg::Pos(Value::str(store.name.as_str())));
                node
            }
            Self::ActionsDecl(_) => AutoNode::new("actions"),
            Self::ViewFragmentDecl(fragment) => {
                let mut node = AutoNode::new("view-fn");
                node.add_arg(auto_val::Arg::Pos(Value::str(fragment.name.as_str())));
                node
            }
            Self::StyleRecipeDecl(recipe) => {
                let mut node = AutoNode::new("style-recipe");
                node.add_arg(auto_val::Arg::Pos(Value::str(recipe.name.as_str())));
                node
            }
            Self::MsgDecl(msg) => {
                let mut node = AutoNode::new("msg");
                node.set_prop("variants", Value::Int(msg.variants.len() as i32));
                node
            }
            Self::ModelBlock(model) => {
                let mut node = AutoNode::new("model");
                node.set_prop("fields", Value::Int(model.fields.len() as i32));
                node
            }
            Self::ViewBlock(_) => AutoNode::new("view"),
            Self::UseWeb(_) => AutoNode::new("use.web"),
        };
        node
    }
}

impl ToAtom for UiStmt {
    fn to_atom(&self) -> AutoStr {
        match self {
            Self::UseWeb(_) => AutoStr::from("use.web"),
            _ => self.to_string().into(),
        }
    }
}

impl AtomWriter for UiStmt {
    fn write_atom(&self, f: &mut impl std::io::Write) -> auto_val::AutoResult<()> {
        write!(f, "{}", self.to_atom_str())?;
        Ok(())
    }
}
