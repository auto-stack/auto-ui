//! Node2D 研究 dialect 解析（PLAN-016 T-03）。
//!
//! 独立 `DomainDialect`：仅显式 `parse_domain(&Node2DDialect)` 研究
//! 入口可达；关键字 `node` 不在 `UiDialect` 关键字表内，旧 widget/store
//! 解析路径零变化。`view-2d` 域标记按 token 组装识别（`View` + `Sub`
//! + 数字 `2`，共享词法零改动，普通表达式减号语义不变）；`on` 块头
//! （`Life.*` 系统事件 / 前导点业务事件）为研究域自有解析，handler
//! 体复用共享 `parse_body`（`emit`/前导点语句由 Core 体文法接受）。

use auto_parser_api::ast::Type;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;
use auto_parser_api::{CompilerSession, DomainDialect, DomainParseContext, DomainStmt, Scenario};

use super::ast::{
    EventPattern, Handler, Node2DStmt, NodeDecl, OnBlock, Prop, PropValue, View2dBlock,
    View2dElement,
};
use crate::declarations::{parse_model_block, parse_msg_decl};

/// Node2D 研究 dialect。经由 `Parser::parse_domain(&Node2DDialect)`
/// 显式组合；生产 UI 管线不引用此类型。
pub struct Node2DDialect;

impl DomainDialect for Node2DDialect {
    type Output = Node2DStmt;

    fn matches(&self, session: &CompilerSession) -> bool {
        session.scenario == Scenario::UI
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["node"]
    }

    fn try_parse_stmt(
        &self,
        cx: &mut dyn DomainParseContext,
        keyword: &str,
    ) -> AutoResult<Option<DomainStmt<Node2DStmt>>> {
        if cx.in_on_body() || cx.in_fn_body() {
            return Ok(None);
        }
        if keyword != "node" {
            return Ok(None);
        }
        let decl = parse_node_decl(cx)?;
        Ok(Some(DomainStmt::Domain(Node2DStmt::Node(decl))))
    }
}

/// `node Name(param: Type, ...) { model? msg? view-2d? on? }`
pub fn parse_node_decl(cx: &mut dyn DomainParseContext) -> AutoResult<NodeDecl> {
    cx.expect_keyword("node")?;
    let start = cx.cur_pos();
    let name = cx.cur_text();
    if name.is_empty() || !name.chars().next().unwrap().is_ascii_uppercase() {
        return Err(SyntaxError::Generic {
            message: format!("node name must be UpperCamelCase, got '{name}'"),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();

    let mut params = Vec::new();
    if cx.is_kind(TokenKind::LParen) {
        cx.advance();
        cx.skip_empty_lines();
        while !cx.is_kind(TokenKind::RParen) {
            let pname = cx.cur_text();
            if pname.is_empty() {
                return Err(SyntaxError::Generic {
                    message: "node parameter name expected".to_string(),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            cx.advance();
            cx.expect(TokenKind::Colon)?;
            let ty = parse_param_type(cx)?;
            params.push((pname, ty));
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
            }
            cx.skip_empty_lines();
        }
        cx.expect(TokenKind::RParen)?;
    }

    // 构造参数与冻结动作符号进入解析作用域：view/handler 体把参数视作
    // 本实例成员，`input.axis(Left, Right)` 的动作名是冻结研究服务符号
    // （Left=0/Right=1/Jump=2/Enter=3，值由 lowering 映射）。
    cx.enter_scope();
    for (pname, ty) in &params {
        cx.bind_var(pname, ty.clone());
    }
    for action in ["Left", "Right", "Jump", "Enter"] {
        cx.bind_var(action, Type::Int);
    }

    cx.expect(TokenKind::LBrace)?;
    let mut model = None;
    let mut msg = None;
    let mut view2d = None;
    let mut on = None;
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        // view-2d 域标记：View + Sub + 数字 2（词法面见模块文档）。
        if cx.is_kind(TokenKind::View) {
            if view2d.is_some() {
                return Err(SyntaxError::Generic {
                    message: "duplicate view-2d block in node".to_string(),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            view2d = Some(parse_view2d_block(cx)?);
            continue;
        }
        if cx.is_kind(TokenKind::On) {
            if on.is_some() {
                return Err(SyntaxError::Generic {
                    message: "duplicate on block in node".to_string(),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            on = Some(parse_on_block(cx)?);
            continue;
        }
        if cx.cur_is_soft_ident() || cx.is_kind(TokenKind::Ident) {
            let keyword = cx.cur_text();
            match keyword.as_str() {
                "model" => {
                    if model.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate model block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    model = Some(parse_model_block(cx)?);
                    continue;
                }
                "msg" => {
                    if msg.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate msg block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    msg = Some(parse_msg_decl(cx)?);
                    continue;
                }
                _ => {
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "unsupported node block '{keyword}' (research surface: model/msg/view-2d/on)"
                        ),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
        }
        return Err(SyntaxError::Generic {
            message: "unsupported token in node body".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.expect(TokenKind::RBrace)?;
    cx.exit_scope();

    Ok(NodeDecl {
        name,
        params,
        model,
        msg,
        view2d,
        on,
        span: super::ast::span_of(start),
    })
}

/// `view-2d { element* }`；元素：solid2d/trigger2d/camera2d/实例化/if。
fn parse_view2d_block(cx: &mut dyn DomainParseContext) -> AutoResult<View2dBlock> {
    let start = cx.cur_pos();
    cx.advance(); // View
    cx.expect(TokenKind::Sub)?;
    let marker = cx.cur_text();
    if !marker.starts_with('2') {
        return Err(SyntaxError::Generic {
            message: format!("view domain marker must be view-2d, got 'view-{marker}'"),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    cx.expect(TokenKind::LBrace)?;
    let mut elements = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        elements.push(parse_view2d_element(cx)?);
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(View2dBlock {
        elements,
        span: super::ast::span_of(start),
    })
}

fn parse_view2d_element(cx: &mut dyn DomainParseContext) -> AutoResult<View2dElement> {
    if cx.is_kind(TokenKind::If) {
        cx.advance();
        let cond = cx.parse_expr()?;
        let then = parse_element_children(cx)?;
        return Ok(View2dElement::If { cond, then });
    }
    let head = cx.cur_text();
    if head.is_empty() {
        return Err(SyntaxError::Generic {
            message: "view-2d element expected".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    let elem_name = cx.cur_text();
    if elem_name.is_empty() {
        return Err(SyntaxError::Generic {
            message: "view-2d element name expected".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    let props = parse_prop_block(cx)?;
    Ok(match head.as_str() {
        "solid2d" => View2dElement::Solid {
            name: elem_name,
            props,
        },
        "trigger2d" => View2dElement::Trigger {
            name: elem_name,
            props,
        },
        "camera2d" => View2dElement::Camera {
            name: elem_name,
            props,
        },
        other => {
            // 约定：UpperCamelCase 头部 = 同定义实例化（组件实例语义）。
            let first = other.chars().next().unwrap();
            if !first.is_ascii_uppercase() {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "unknown view-2d element '{other}' (expected solid2d/trigger2d/camera2d or a Node instance)"
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            View2dElement::Instance {
                node: other.to_string(),
                name: elem_name,
                props,
            }
        }
    })
}

/// `{ ... }` 元素子列表（if 分支体）。
fn parse_element_children(cx: &mut dyn DomainParseContext) -> AutoResult<Vec<View2dElement>> {
    cx.expect(TokenKind::LBrace)?;
    let mut elements = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        elements.push(parse_view2d_element(cx)?);
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(elements)
}

/// `{ name: value ... }` 属性行块；空行/逗号宽容。
fn parse_prop_block(cx: &mut dyn DomainParseContext) -> AutoResult<Vec<Prop>> {
    cx.expect(TokenKind::LBrace)?;
    let mut props = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let start = cx.cur_pos();
        let pname = cx.cur_text();
        if pname.is_empty() {
            return Err(SyntaxError::Generic {
                message: "property name expected".to_string(),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        cx.advance();
        cx.expect(TokenKind::Colon)?;
        let value = parse_prop_value(cx)?;
        props.push(Prop {
            name: pname,
            value,
            span: super::ast::span_of(start),
        });
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(props)
}

/// 属性值：`(a, b)` 字面量组（position/bounds/view_size）或单个共享
/// Core 表达式（路径、前导点成员、调用、字面量）。
fn parse_prop_value(cx: &mut dyn DomainParseContext) -> AutoResult<PropValue> {
    if cx.is_kind(TokenKind::LParen) {
        // 投机尝试字面量组 `(a, b)`；restore 回滚游标并截断投机诊断。
        let cp = cx.checkpoint();
        let mark = cx.diagnostics_mark();
        cx.advance();
        cx.skip_empty_lines();
        let mut exprs = Vec::new();
        loop {
            match cx.parse_expr() {
                Ok(e) => exprs.push(e),
                Err(_) => {
                    cx.restore(cp)?;
                    cx.diagnostics_truncate(mark);
                    break;
                }
            }
            cx.skip_empty_lines();
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
                cx.skip_empty_lines();
                continue;
            }
            if cx.is_kind(TokenKind::RParen) {
                cx.advance();
                return Ok(PropValue::Pair(exprs));
            }
            cx.restore(cp)?;
            cx.diagnostics_truncate(mark);
            break;
        }
    }
    Ok(PropValue::One(cx.parse_expr()?))
}

/// `on { handler* }`；模式：`Life.X` / `Life.X(p)`（系统事件）或
/// `.X` / `.X(p)`（本 node 业务事件）；`->` 后为共享体文法。
fn parse_on_block(cx: &mut dyn DomainParseContext) -> AutoResult<OnBlock> {
    let start = cx.cur_pos();
    cx.expect(TokenKind::On)?;
    cx.expect(TokenKind::LBrace)?;
    let mut handlers = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        handlers.push(parse_handler(cx)?);
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(OnBlock {
        handlers,
        span: super::ast::span_of(start),
    })
}

fn parse_handler(cx: &mut dyn DomainParseContext) -> AutoResult<Handler> {
    let start = cx.cur_pos();
    let pattern = if cx.cur_text() == "Life" && cx.is_kind(TokenKind::Ident) {
        cx.advance();
        cx.expect(TokenKind::Dot)?;
        let event = cx.cur_text();
        if event.is_empty() {
            return Err(SyntaxError::Generic {
                message: "Life event name expected (Life.Start/Life.Tick/Life.Frame)".to_string(),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        cx.advance();
        // Life.Frame 是呈现帧节奏后继事件，不纳入 PLAN-016 实现/夹具/
        // 验收（计划 §5.2）——显式越界拒绝，不用空处理器冒充支持。
        if event == "Frame" {
            return Err(SyntaxError::Generic {
                message: "Life.Frame is out of PLAN-016 scope (presentation-frame successor event)"
                    .to_string(),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        if event != "Start" && event != "Tick" {
            return Err(SyntaxError::Generic {
                message: format!(
                    "unknown Life event '{event}' (research surface: Life.Start/Life.Tick)"
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        EventPattern::Life {
            event,
            params: parse_handler_params(cx)?,
        }
    } else if cx.is_kind(TokenKind::Dot) {
        cx.advance();
        let name = cx.cur_text();
        if name.is_empty() {
            return Err(SyntaxError::Generic {
                message: "business event name expected after '.'".to_string(),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        cx.advance();
        EventPattern::Member {
            name,
            params: parse_handler_params(cx)?,
        }
    } else {
        return Err(SyntaxError::Generic {
            message: "research on pattern must be 'Life.X' or '.X'".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    };
    if cx.is_kind(TokenKind::Arrow) {
        cx.advance();
    } else if cx.is_kind(TokenKind::DoubleArrow) {
        cx.advance();
    } else {
        return Err(SyntaxError::Generic {
            message: "expected '->' after on pattern".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    let body = cx.parse_body()?;
    Ok(Handler {
        pattern,
        body,
        span: super::ast::span_of(start),
    })
}

/// 可选 `(p0, p1)` 处理器参数（仅标识符，不取类型）。
fn parse_handler_params(cx: &mut dyn DomainParseContext) -> AutoResult<Vec<String>> {
    let mut params = Vec::new();
    if cx.is_kind(TokenKind::LParen) {
        cx.advance();
        while !cx.is_kind(TokenKind::RParen) {
            let p = cx.cur_text();
            if p.is_empty() {
                return Err(SyntaxError::Generic {
                    message: "handler parameter name expected".to_string(),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            params.push(p);
            cx.advance();
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
            }
        }
        cx.expect(TokenKind::RParen)?;
    }
    Ok(params)
}

/// 参数类型：普通 Core 类型，或事件端口关键字 `msg`（Msg token，无
/// 载荷形状 —— 投递走绑定表，不在参数位展开）。
fn parse_param_type(cx: &mut dyn DomainParseContext) -> AutoResult<Type> {
    // `msg` 不是词法关键字（UiDialect 在派发层认领），此处按事件端口
    // 端类型识别为无载荷形状的 Unknown。
    if cx.is_kind(TokenKind::Ident) && cx.cur_text() == "msg" {
        cx.advance();
        return Ok(Type::Unknown);
    }
    cx.parse_type()
}
