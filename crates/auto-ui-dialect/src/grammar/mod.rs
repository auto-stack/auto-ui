pub use auto_parser_api::DomainParseContext as UiParseContext;

pub mod actions;
pub mod helpers;
pub mod store;
pub mod use_web;
pub mod view;
pub mod widget;

use crate::UiStmt;
use auto_parser_api::ast::Stmt;
use auto_parser_api::error::AutoResult;
use auto_parser_api::token::TokenKind;
use auto_parser_api::{CompilerSession, DomainDialect, DomainStmt, Scenario};

pub struct UiDialect;

impl DomainDialect for UiDialect {
    type Output = UiStmt;

    fn matches(&self, session: &CompilerSession) -> bool {
        session.scenario == Scenario::UI
    }

    fn keywords(&self) -> &'static [&'static str] {
        &[
            "widget",
            "msg",
            "model",
            "component",
            "actions",
            "style",
            "store",
        ]
    }

    fn try_parse_stmt(
        &self,
        cx: &mut dyn UiParseContext,
        keyword: &str,
    ) -> AutoResult<Option<DomainStmt<UiStmt>>> {
        if cx.in_on_body() || cx.in_fn_body() {
            return Ok(None);
        }
        let item = match keyword {
            "widget" => DomainStmt::Domain(widget::parse_widget_decl(cx)?),
            "msg" => DomainStmt::Domain(UiStmt::MsgDecl(crate::declarations::parse_msg_decl(cx)?)),
            "model" => DomainStmt::Domain(UiStmt::ModelBlock(
                crate::declarations::parse_model_block(cx)?,
            )),
            "component" => DomainStmt::Domain(view::parse_component_fn_decl(cx)?),
            "actions" => match actions::parse_actions_decl_stmt(cx)? {
                Some(stmt) => DomainStmt::Domain(stmt),
                None => return Ok(None),
            },
            "style" => match actions::parse_style_recipe_decl_stmt(cx)? {
                Some(stmt) => DomainStmt::Domain(stmt),
                None => return Ok(None),
            },
            "store" => DomainStmt::Domain(store::parse_store_decl(cx)?),
            _ => return Ok(None),
        };
        Ok(Some(item))
    }

    fn try_parse_token_stmt(
        &self,
        cx: &mut dyn UiParseContext,
        kind: TokenKind,
    ) -> AutoResult<Option<DomainStmt<UiStmt>>> {
        match kind {
            TokenKind::View => Ok(Some(DomainStmt::Domain(
                view::parse_view_block_or_fragment(cx)?,
            ))),
            TokenKind::On => Ok(Some(DomainStmt::Core(Stmt::OnEvents(
                cx.parse_on_events()?,
            )))),
            TokenKind::Use if use_web::looks_like_use_web(cx)? => {
                Ok(Some(DomainStmt::Domain(use_web::parse_use_web(cx)?)))
            }
            _ => Ok(None),
        }
    }

    fn try_parse_pub_stmt(
        &self,
        cx: &mut dyn UiParseContext,
    ) -> AutoResult<Option<DomainStmt<UiStmt>>> {
        cx.advance();
        if cx.cur_text() != "style" {
            return Ok(None);
        }
        let Some(mut stmt) = actions::parse_style_recipe_decl_stmt(cx)? else {
            return Ok(None);
        };
        let UiStmt::StyleRecipeDecl(recipe) = &mut stmt else {
            unreachable!()
        };
        recipe.is_pub = true;
        Ok(Some(DomainStmt::Domain(stmt)))
    }
}
