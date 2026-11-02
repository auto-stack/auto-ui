//! `theme` 顶层主题数据声明（PLAN-007 T-04，C-01/03）。
//!
//! ```auto
//! pub theme Auto = { tokens: { ... } modes: { light: { ... } dark: { ... } } }
//! theme Internal = extend(themes.Auto, app_defaults, { ... })
//! ```
//!
//! 形态镜像 `style` recipe 声明（名字 + `=` + 表达式）；树体是普通 Auto
//! 对象字面量或 `extend(...)` 组合调用，由编译器侧 theme_tree 模块做
//! Schema 校验/合成/模式选择。此处不做树形语义校验——dialect 只负责
//! 把声明接住为 UI 域语句（`UiStmt::ThemeDataDecl`）。

use crate::ast::ThemeDataDecl;
use crate::UiStmt;
use auto_parser_api::error::AutoResult;
use auto_parser_api::token::TokenKind;
use auto_parser_api::DomainParseContext;

/// 解析 `theme <Name> = <expr>`（`pub` 前缀已由 dialect pub 臂消费）。
/// 非 `theme Name =` 形态回退 `Ok(None)`（调用方恢复/走 Core 解析）。
pub fn parse_theme_decl_stmt(cx: &mut dyn DomainParseContext) -> AutoResult<Option<UiStmt>> {
    let cp = cx.raw_cursor_save();

    cx.advance(); // consume "theme"
    if !cx.is_kind(TokenKind::Ident) {
        cx.raw_cursor_restore(cp)?;
        return Ok(None);
    }
    let name = cx.cur_text();
    cx.advance(); // consume theme name

    if !cx.is_kind(TokenKind::Asn) {
        cx.raw_cursor_restore(cp)?;
        return Ok(None);
    }
    cx.raw_cursor_release(cp);

    let doc = cx.take_docs();
    cx.advance(); // consume '='
    cx.skip_empty_lines();
    let tree = cx.parse_expr()?;

    Ok(Some(UiStmt::ThemeDataDecl(ThemeDataDecl {
        name: name.into(),
        is_pub: false,
        tree,
        doc: doc.map(Into::into),
    })))
}
