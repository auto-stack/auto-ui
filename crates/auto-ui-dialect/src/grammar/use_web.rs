use crate::ast::{ExtImport, ExtImportKind};
use crate::UiStmt;
use auto_parser_api::ast::{Expr, Name, Type};
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;
use auto_parser_api::DomainParseContext;
use auto_val::AutoStr;

pub fn looks_like_use_web(cx: &mut dyn DomainParseContext) -> AutoResult<bool> {
    let cp = cx.raw_cursor_save();
    cx.advance();
    let matches = cx.is_kind(TokenKind::Dot) && {
        cx.advance();
        cx.cur_text() == "web"
    };
    cx.raw_cursor_restore(cp)?;
    Ok(matches)
}

pub fn parse_use_web(cx: &mut dyn DomainParseContext) -> AutoResult<UiStmt> {
    cx.advance();
    cx.expect(TokenKind::Dot)?;
    cx.expect_ident("web")?;

    let kind = if cx.is_kind(TokenKind::Dot) {
        cx.advance();
        let modifier = cx.cur_text();
        cx.advance();
        match modifier.as_str() {
            "fn" => ExtImportKind::ExplicitFn,
            "component" => ExtImportKind::Component,
            "composable" => ExtImportKind::Composable,
            other => {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Unknown use.web.<kind> '{}' — expected fn / component / composable",
                        other
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into())
            }
        }
    } else {
        match cx.cur_text().as_str() {
            "component" => {
                cx.advance();
                ExtImportKind::Component
            }
            "composable" => {
                cx.advance();
                ExtImportKind::Composable
            }
            _ => ExtImportKind::Fn,
        }
    };

    let mut symbols: Vec<Name> = Vec::new();
    loop {
        if !cx.is_kind(TokenKind::Ident) {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Expected symbol name in use.web statement, got '{}'",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        symbols.push(cx.cur_text().into());
        cx.advance();
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        } else {
            break;
        }
    }

    let mut call_args: Vec<Expr> = Vec::new();
    if cx.is_kind(TokenKind::LParen) {
        cx.advance();
        while !cx.is_kind(TokenKind::RParen) {
            call_args.push(cx.parse_expr()?);
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
            }
        }
        cx.expect(TokenKind::RParen)?;
    }

    let mut ref_fields: Vec<Name> = Vec::new();
    if cx.cur_text() == "refs" {
        cx.advance();
        cx.expect(TokenKind::Colon)?;
        cx.expect(TokenKind::LSquare)?;
        while !cx.is_kind(TokenKind::RSquare) {
            ref_fields.push(cx.cur_text().into());
            cx.advance();
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
            }
        }
        cx.expect(TokenKind::RSquare)?;
    }

    let from = cx.cur_text();
    if from != "from" {
        return Err(SyntaxError::Generic {
            message: format!("Expected 'from' in use.web statement, got '{}'", from),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    if !cx.is_kind(TokenKind::Str) {
        return Err(SyntaxError::Generic {
            message: "Expected module path string after 'from' in use.web statement".into(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    let path: AutoStr = cx.cur_text().into();
    cx.advance();
    for symbol in &symbols {
        cx.bind_var(symbol.as_str(), Type::Unknown);
    }

    Ok(UiStmt::UseWeb(vec![ExtImport {
        kind,
        symbols,
        path,
        call_args,
        ref_fields,
    }]))
}
