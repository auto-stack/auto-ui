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
    Camera3dBlock, Character3dBlock, EventPattern, Handler, LightSpan, NameListBlock, Node2DStmt,
    NodeDecl, Object3dBlock, OnBlock, Prop, PropValue, UiContainerKind, UiElement, View2dBlock,
    View2dElement, View3dBlock, View3dElement, ViewUiBlock,
};
use crate::declarations::{parse_model_block, parse_msg_decl};

/// Node2D 研究 dialect。经由 `Parser::parse_domain(&Node2DDialect)`
/// 显式组合；生产 UI 管线不引用此类型。
pub struct Node2DDialect;

impl Node2DDialect {
    fn try_parse_stmt_impl(
        &self,
        cx: &mut dyn DomainParseContext,
        keyword: &str,
        allow_dual_view: bool,
        allow_embed: bool,
    ) -> AutoResult<Option<DomainStmt<Node2DStmt>>> {
        if cx.in_on_body() || cx.in_fn_body() {
            return Ok(None);
        }
        if keyword != "node" {
            return Ok(None);
        }
        let decl = if allow_dual_view {
            // PLAN-039：仅显式查询 profile 的组合 dialect 在解析期放行
            // 同 node 双 view；有限组合仍由该 profile 降层准入裁决。
            parse_node_decl_opts(cx, true, allow_embed)?
        } else {
            parse_node_decl_opts(cx, false, allow_embed)?
        };
        Ok(Some(DomainStmt::Domain(Node2DStmt::Node(decl))))
    }
}

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
        self.try_parse_stmt_impl(cx, keyword, false, false)
    }
}

/// PLAN-039 queries-3d 组合 dialect：与 [`Node2DDialect`] 完全同一文法，
/// 唯一差异是解析期放行同 node `view-2d`＋`view-3d` 双出口（单 node
/// 旧拒绝在默认 [`Node2DDialect`] 路径字节不变）；放行后的有限组合
/// （仅根、共用 model/msg/on、子 Node 单维）由 pilot 的 queries-3d
/// profile 降层准入具名裁决，不放宽任何旧 profile 行为。
pub struct Node2DQueriesDialect;

impl DomainDialect for Node2DQueriesDialect {
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
        Node2DDialect.try_parse_stmt_impl(cx, keyword, true, false)
    }
}

/// PLAN-049 mixed-ui-2d-3d 组合 dialect：与 [`Node2DDialect`] 完全同一
/// 文法，唯一差异是解析期放行基础 UI `view` 元素树内的 `view-2d`/
/// `view-3d` **嵌入区域**（挂载关系，非 node 级双出口——node 级
/// view+view-2d/view-3d 互斥守卫在任何 dialect 下字节不变；`allow_dual_view`
/// 保持 false，039 查询组合的有限 marker 档身份不扩大）。区域内为现有
/// view-2d/view-3d 图元元素。放行后的组合由 pilot 的 mixed-ui-2d-3d
/// profile 降层准入具名裁决，不放宽任何旧 profile 行为。
pub struct Node2DMixedDialect;

impl DomainDialect for Node2DMixedDialect {
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
        Node2DDialect.try_parse_stmt_impl(cx, keyword, false, true)
    }
}

/// `node Name(param: Type, ...) { model? msg? view-2d? on? }`
pub fn parse_node_decl(cx: &mut dyn DomainParseContext) -> AutoResult<NodeDecl> {
    parse_node_decl_opts(cx, false, false)
}

/// PLAN-039：`allow_dual_view=true` 时不做单 view 出口解析拒绝；
/// 有限组合（仅根双出口等）由调用方 profile 降层准入具名裁决。
/// PLAN-049：`allow_embed=true` 时 view 元素树内放行 view-2d/view-3d
/// 嵌入区域（Node2DMixedDialect 使用）。
pub fn parse_node_decl_opts(
    cx: &mut dyn DomainParseContext,
    allow_dual_view: bool,
    allow_embed: bool,
) -> AutoResult<NodeDecl> {
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
    // PLAN-027：3D 运行六动作符号（Left/Right/Forward/Back/Jump/Restart
    // =0..5，值由 components-3d-runtime 降层映射）。旧档夹具不引用这些
    // 名字，解析作用域扩展不改变旧字节。
    for action in ["Forward", "Back", "Restart"] {
        cx.bind_var(action, Type::Int);
    }
    // PLAN-019：shape2d `kind` 的冻结几何符号（circle/quad）。与动作符号
    // 同机制：解析作用域内的名义符号，值由 lowering 按名字映射。
    // PLAN-023：`box`（game2d 触发行的 AABB 形状名，同一名义符号族）。
    for shape_kind in ["circle", "quad", "box"] {
        cx.bind_var(shape_kind, Type::Int);
    }
    // PLAN-027：camera3d `projection` 的名义投影符号（perspective/ortho，
    // 值由 components-3d-runtime 降层映射 0/1）。
    for projection in ["perspective", "ortho"] {
        cx.bind_var(projection, Type::Int);
    }
    // PLAN-028：components-3d-events `bind` 形状名义符号（aabb/sphere，
    // 值由 components-3d-events 降层映射 0/1）。旧档夹具不引用这些名字，
    // 解析作用域扩展不改变旧字节。
    for trigger_shape in ["aabb", "sphere"] {
        cx.bind_var(trigger_shape, Type::Int);
    }
    // PLAN-030：components-3d-bindings 名义引用根符号（点根形态：
    // `objects3d.<槽>`、`triggers.<pair>`、`templates.<模板定义>`、
    // `refs.<实例路径>`；数值/行身份由 bindings 档降层按名字解析）。与
    // aabb/sphere 同机制：解析作用域内的名义符号；旧档夹具不引用这些
    // 名字（旧值面全为字面量/resources./字符串字面量），字节不变。
    for binding_root in ["objects3d", "triggers", "templates", "refs"] {
        cx.bind_var(binding_root, Type::Int);
    }

    cx.expect(TokenKind::LBrace)?;
    let mut model = None;
    let mut msg = None;
    let mut view = None;
    let mut view2d = None;
    let mut view3d = None;
    let mut on = None;
    let mut game2d = None;
    let mut presentation2d = None;
    let mut components2d = None;
    let mut presentation3d = None;
    let mut character3d = None;
    let mut camera3d = None;
    let mut object3d = None;
    let mut objects3d = None;
    let mut triggers = None;
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        // view 域标记分派（PLAN-024）：View + Sub + 数字 2/3。'2' 走冻结
        // view-2d 文法（旧路径与旧诊断逐字不变），'3' 走 view-3d；其它
        // 标记保持 view-2d 的既有具名拒绝。投机窥探用 checkpoint/restore
        // 回滚游标。PLAN-048：裸 `view`（无 Sub 标记）= 基础 UI 出口，
        // 在域标记窥探前拦截，view-2d/view-3d 的旧路径与旧诊断逐字不变。
        if cx.is_kind(TokenKind::View) {
            let marker = {
                let cp = cx.checkpoint();
                cx.advance();
                let m = match cx.expect(TokenKind::Sub) {
                    Ok(()) => cx.cur_text().to_string(),
                    Err(_) => String::new(),
                };
                cx.restore(cp)?;
                m
            };
            if marker.is_empty() {
                if view.is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate view block in node".to_string(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
                view = Some(parse_view_ui_block(cx, allow_embed)?);
                continue;
            }
            if marker.starts_with('3') {
                if view3d.is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate view-3d block in node".to_string(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
                view3d = Some(parse_view3d_block(cx)?);
            } else {
                if view2d.is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate view-2d block in node".to_string(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
                view2d = Some(parse_view2d_block(cx)?);
            }
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
                // PLAN-023：components-2d-game 作者入口块（触发表/cue 表/
                // HUD 槽/音频资源）。任何 node 都可携带；root-only 与
                // profile 归属由降层具名校验（解析面保持宽容）。
                "game2d" => {
                    if game2d.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate game2d block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    game2d = Some(super::game::parse_game2d_block(cx)?);
                    continue;
                }
                // PLAN-025：通用声音/HUD 接线入口块（typed 资源/槽 owner/
                // generic 触发行；无业务枚举）。任何 node 都可携带；root-only
                // 与 profile 归属由降层具名校验（解析面保持宽容）。
                "components2d" => {
                    if components2d.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate components2d block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    components2d = Some(super::components2d::parse_components2d_block(cx)?);
                    continue;
                }
                "presentation2d" => {
                    if presentation2d.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate presentation2d block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    presentation2d = Some(super::presentation::parse_presentation2d_block(cx)?);
                    continue;
                }
                // PLAN-035：`presentation3d { ... }` —— 三维通用声音/HUD
                // 接线块。任何 node 都可携带；root-only 与 profile 归属由
                // 降层具名校验（解析面保持宽容）。
                "presentation3d" => {
                    if presentation3d.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate presentation3d block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    presentation3d = Some(super::presentation::parse_presentation3d_block(cx)?);
                    continue;
                }
                // PLAN-027：`character3d { }` —— 角色声明块。任何 node 都可
                // 携带；恰一/profile 归属由降层具名校验（解析面保持宽容）。
                "character3d" => {
                    if character3d.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate character3d block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    character3d = Some(parse_character3d_block(cx)?);
                    continue;
                }
                // PLAN-027：`camera3d { ... }` —— 运行相机配置声明。任何
                // node 都可携带；root-only/恰一/target 归属由降层具名校验。
                "camera3d" => {
                    if camera3d.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate camera3d block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    camera3d = Some(parse_camera3d_block(cx)?);
                    continue;
                }
                // PLAN-028：`object3d { }` —— spawn 模板定义标记。任何 node
                // 都可携带；档归属/模板形状由降层具名校验（解析面宽容）。
                "object3d" => {
                    if object3d.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate object3d block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    object3d = Some(parse_object3d_block(cx)?);
                    continue;
                }
                // PLAN-028：`objects3d { a, b, ... }` —— 具名对象槽池声明。
                // 任何 node 都可携带；root-only/池包络由降层具名校验。
                "objects3d" => {
                    if objects3d.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate objects3d block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    objects3d = Some(parse_name_list_block(cx)?);
                    continue;
                }
                // PLAN-028：`triggers { a, b, ... }` —— 具名触发 pair 池
                // 声明。任何 node 都可携带；root-only/池包络由降层具名校验。
                "triggers" => {
                    if triggers.is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate triggers block in node".to_string(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    triggers = Some(parse_name_list_block(cx)?);
                    continue;
                }
                _ => {
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "unsupported node block '{keyword}' (research surface: model/msg/view/view-2d/on/game2d/presentation2d/character3d/camera3d/object3d/objects3d/triggers)"
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

    // PLAN-024：单一 view 出口——同一 node 同时携带 view-2d 与 view-3d
    // 属多呈现混合，不在本片（具名拒绝）。PLAN-039：显式查询组合 dialect
    // 经 `parse_node_decl_opts(_, true)` 放行解析，有限组合由其 profile
    // 降层准入裁决；本默认路径的拒绝消息与行为字节不变。
    if view2d.is_some() && view3d.is_some() && !allow_dual_view {
        return Err(SyntaxError::Generic {
            message: format!(
                "node '{name}' declares both view-2d and view-3d (single view exit per node)"
            ),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }

    // PLAN-048：基础 UI `view` 与图形出口互斥（混合嵌入是 PLAN-049；
    // 双维 profile 的准入不变，本守卫在任何 dialect 下都成立）。
    if view.is_some() && (view2d.is_some() || view3d.is_some()) {
        return Err(SyntaxError::Generic {
            message: format!(
                "node '{name}' declares view together with view-2d/view-3d (basic UI and graphics embedding are separate exits; composition is PLAN-049)"
            ),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }

    Ok(NodeDecl {
        name,
        params,
        model,
        msg,
        view,
        view2d,
        view3d,
        on,
        game2d,
        presentation2d,
        components2d,
        presentation3d,
        character3d,
        camera3d,
        object3d,
        objects3d,
        triggers,
        span: super::ast::span_of(start),
    })
}

/// `view-2d { element* }`；元素：solid2d/trigger2d/camera2d/shape2d/group2d/实例化/if。
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

/// `view-3d { element* }`（PLAN-024）：元素：cylinder3d/model3d/group3d/
/// UpperCamel 实例化。复用共享 Core 表达式/prop 文法，不新增解释器。
fn parse_view3d_block(cx: &mut dyn DomainParseContext) -> AutoResult<View3dBlock> {
    let start = cx.cur_pos();
    cx.advance(); // View
    cx.expect(TokenKind::Sub)?;
    let marker = cx.cur_text();
    if !marker.starts_with('3') {
        return Err(SyntaxError::Generic {
            message: format!("view domain marker must be view-3d, got 'view-{marker}'"),
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
        elements.push(parse_view3d_element(cx)?);
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(View3dBlock {
        elements,
        span: super::ast::span_of(start),
    })
}

fn parse_view3d_element(cx: &mut dyn DomainParseContext) -> AutoResult<View3dElement> {
    let head = cx.cur_text();
    if head.is_empty() {
        return Err(SyntaxError::Generic {
            message: "view-3d element expected".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    let elem_name = cx.cur_text();
    if elem_name.is_empty() {
        return Err(SyntaxError::Generic {
            message: "view-3d element name expected".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    // `group3d name { props/children 混合块 }`：显式同维父组，先于通用
    // props 分派（子元素递归走同一文法）。
    if head == "group3d" {
        let (props, children) = parse_group3d_block(cx)?;
        return Ok(View3dElement::Group {
            name: elem_name,
            props,
            children,
        });
    }
    let props = parse_prop_block(cx)?;
    Ok(match head.as_str() {
        "cylinder3d" => View3dElement::Primitive {
            name: elem_name,
            props,
        },
        "model3d" => View3dElement::Model {
            name: elem_name,
            props,
        },
        // PLAN-027：具名内置视觉叶（原 Kind::Player/Mob；mesh/材质沿
        // 原 owner 暖上传映射，声明只透传 kind/静态变换）。
        "player3d" => View3dElement::Player {
            name: elem_name,
            props,
        },
        "mob3d" => View3dElement::Mob {
            name: elem_name,
            props,
        },
        other => {
            // 约定：UpperCamelCase 头部 = 同定义实例化（组件实例语义）。
            let first = other.chars().next().unwrap();
            if !first.is_ascii_uppercase() {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "unknown view-3d element '{other}' (expected cylinder3d/model3d/player3d/mob3d/group3d or a Node instance)"
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            View3dElement::Instance {
                node: other.to_string(),
                name: elem_name,
                props,
            }
        }
    })
}

/// PLAN-027：`character3d { }` —— v1 无 props（形状/支撑/价格由受信
/// 服务与元素静态 scale 决定）；props 出现即具名拒绝文本（降层校验），
/// 这里只接受空块。
fn parse_character3d_block(cx: &mut dyn DomainParseContext) -> AutoResult<Character3dBlock> {
    let start = cx.cur_pos();
    cx.advance(); // character3d
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();
    if !cx.is_kind(TokenKind::RBrace) {
        return Err(SyntaxError::Generic {
            message: "character3d block takes no properties (v1 research surface)".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(Character3dBlock {
        span: super::ast::span_of(start),
    })
}

/// PLAN-028：`object3d { }` —— spawn 模板定义标记（v1 无 props；模板几何
/// = 本定义 view-3d 唯一叶）。
fn parse_object3d_block(cx: &mut dyn DomainParseContext) -> AutoResult<Object3dBlock> {
    let start = cx.cur_pos();
    cx.advance(); // object3d
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();
    if !cx.is_kind(TokenKind::RBrace) {
        return Err(SyntaxError::Generic {
            message: "object3d block takes no properties (v1 research surface)".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(Object3dBlock {
        span: super::ast::span_of(start),
    })
}

/// PLAN-028：`objects3d { a, b, ... }` / `triggers { a, b, ... }` —— 具名
/// 池声明。裸标识符名字表，逗号分隔可选（换行也可）；池包络/唯一性/
/// root-only 由降层具名校验。
fn parse_name_list_block(cx: &mut dyn DomainParseContext) -> AutoResult<NameListBlock> {
    let start = cx.cur_pos();
    let keyword = cx.cur_text().to_string();
    cx.advance(); // objects3d / triggers
    cx.expect(TokenKind::LBrace)?;
    let mut names = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        if !(cx.cur_is_soft_ident() || cx.is_kind(TokenKind::Ident)) {
            return Err(SyntaxError::Generic {
                message: format!(
                    "expected a name in {keyword} block (bare identifiers separated by commas)"
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        names.push(cx.cur_text().to_string());
        cx.advance();
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(NameListBlock {
        names,
        span: super::ast::span_of(start),
    })
}

/// PLAN-027：`camera3d { distance: .. pitch: .. projection: .. ortho_size: .. }`
/// —— 复用共享 prop 文法；值域/组合（ortho 必带 ortho_size）由降层具名校验。
fn parse_camera3d_block(cx: &mut dyn DomainParseContext) -> AutoResult<Camera3dBlock> {
    let start = cx.cur_pos();
    cx.advance(); // camera3d
    let props = parse_prop_block(cx)?;
    Ok(Camera3dBlock {
        props,
        span: super::ast::span_of(start),
    })
}

/// group3d 混合块：props（小写名行）与子元素按行交错；先 prop 后子元素
/// 为推荐书写序，不强制。子元素头 = cylinder3d/model3d/group3d/UpperCamel。
fn parse_group3d_block(
    cx: &mut dyn DomainParseContext,
) -> AutoResult<(Vec<Prop>, Vec<View3dElement>)> {
    cx.expect(TokenKind::LBrace)?;
    let mut props = Vec::new();
    let mut children = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        if matches!(
            head.as_str(),
            "group3d" | "cylinder3d" | "model3d" | "player3d" | "mob3d"
        ) || head.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        {
            children.push(parse_view3d_element(cx)?);
            continue;
        }
        let start = cx.cur_pos();
        let pname = cx.cur_text();
        if pname.is_empty() {
            return Err(SyntaxError::Generic {
                message: "group3d property name expected".to_string(),
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
    Ok((props, children))
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
    // PLAN-019 基础二维：`group2d name { props/children 混合块 }`（同维
    // 父组，仅平移/可见性继承）；子元素递归走同一文法。group2d 有自己的
    // 块文法，先于通用 props 分派。
    if head == "group2d" {
        let (props, children) = parse_group_block(cx)?;
        return Ok(View2dElement::Group {
            name: elem_name,
            props,
            children,
        });
    }
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
        "character2d" => View2dElement::Character {
            name: elem_name,
            props,
        },
        "camera2d" => View2dElement::Camera {
            name: elem_name,
            props,
        },
        "shape2d" => View2dElement::Shape {
            name: elem_name,
            props,
        },
        other => {
            // 约定：UpperCamelCase 头部 = 同定义实例化（组件实例语义）。
            let first = other.chars().next().unwrap();
            if !first.is_ascii_uppercase() {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "unknown view-2d element '{other}' (expected solid2d/trigger2d/camera2d/shape2d/group2d or a Node instance)"
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

/// PLAN-048 停止集判定：基础 UI 元素头（props 块在此让位给子元素/块界）。
/// 小写关键字与属性名可能同形（`text` 既是元素头又是主属性名）——后随
/// `:` 的按属性名处理（投机窥探 checkpoint/restore 回滚游标）。PLAN-049：
/// `allow_embed` 时 `view-2d`/`view-3d`（View+Sub+2/3）也是元素头
/// （嵌入区域；默认路径不识别，保持 048 拒绝字节）。
fn is_ui_element_head(cx: &mut dyn DomainParseContext, head: &str, allow_embed: bool) -> bool {
    if head.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return true;
    }
    if allow_embed && head == "view" {
        let cp = cx.checkpoint();
        cx.advance();
        let is_embed = cx.is_kind(TokenKind::Sub);
        cx.restore(cp).is_ok() && is_embed
    } else if !matches!(head, "row" | "column" | "text" | "button" | "if") {
        return false;
    } else {
        let cp = cx.checkpoint();
        cx.advance();
        let is_prop = cx.is_kind(TokenKind::Colon);
        cx.restore(cp).is_ok() && !is_prop
    }
}

/// `view { element* }`（PLAN-048）：基础 UI 出口。元素：row/column/text/
/// button/同定义实例化/if。复用共享 Core 表达式/prop 文法；属性白名单与
/// 事件拼写（onclick: `.Ev` / `inst.Ev`）由降层具名校验（解析面与
/// view-2d 同宽容）。
fn parse_view_ui_block(
    cx: &mut dyn DomainParseContext,
    allow_embed: bool,
) -> AutoResult<ViewUiBlock> {
    let start = cx.cur_pos();
    cx.advance(); // View
                  // parse_view_ui_children 消费块界 RBrace（与 view-2d 块循环同责）。
    let elements = parse_view_ui_children(cx, allow_embed)?;
    Ok(ViewUiBlock {
        elements,
        span: super::ast::span_of(start),
    })
}

/// `{ ... }` UI 元素子列表（块体/if 分支体）。
fn parse_view_ui_children(
    cx: &mut dyn DomainParseContext,
    allow_embed: bool,
) -> AutoResult<Vec<UiElement>> {
    cx.expect(TokenKind::LBrace)?;
    let mut elements = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        elements.push(parse_view_ui_element(cx, allow_embed)?);
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(elements)
}

fn parse_view_ui_element(
    cx: &mut dyn DomainParseContext,
    allow_embed: bool,
) -> AutoResult<UiElement> {
    if cx.is_kind(TokenKind::If) {
        cx.advance();
        let cond = cx.parse_expr()?;
        let then = parse_view_ui_children(cx, allow_embed)?;
        return Ok(UiElement::If { cond, then });
    }
    // PLAN-049：`view-2d`/`view-3d` 嵌入区域头（View+Sub+2/3；仅混合
    // profile 放行）。在元素位置识别后转入现有 2D/3D 块解析（区域名
    // 在标记之后）。
    if allow_embed && cx.is_kind(TokenKind::View) {
        let marker = {
            let cp = cx.checkpoint();
            cx.advance();
            let m = match cx.expect(TokenKind::Sub) {
                Ok(()) => cx.cur_text().to_string(),
                Err(_) => String::new(),
            };
            cx.restore(cp)?;
            m
        };
        if marker.starts_with('2') || marker.starts_with('3') {
            return parse_view_embed_element(cx, &marker);
        }
        // 裸 view 头在元素位置：落入下方通用头处理（具名拒绝）。
    }
    let head = cx.cur_text();
    if head.is_empty() {
        return Err(SyntaxError::Generic {
            message: "view element expected".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    let elem_name = cx.cur_text();
    if elem_name.is_empty() {
        return Err(SyntaxError::Generic {
            message: "view element name expected".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    match head.as_str() {
        // 容器：`row/column name { props+children 混合块 }`（group2d 同文）。
        "row" | "column" => {
            let kind = if head == "row" {
                UiContainerKind::Row
            } else {
                UiContainerKind::Column
            };
            let (props, children) = parse_ui_mixed_block(cx, allow_embed)?;
            Ok(UiElement::Container {
                kind,
                name: elem_name,
                props,
                children,
            })
        }
        "text" | "button" => {
            // 叶子元素无子元素：块内出现元素头/实例化时具名拒绝，不静默
            // 丢弃（与 view-2d 叶子同语义）。
            let props = parse_ui_prop_block(
                cx,
                &format!("'{head}' element (name '{elem_name}')"),
                allow_embed,
            )?;
            if head == "text" {
                Ok(UiElement::Text {
                    name: elem_name,
                    props,
                })
            } else {
                Ok(UiElement::Button {
                    name: elem_name,
                    props,
                })
            }
        }
        other => {
            // 约定：UpperCamelCase 头部 = 同定义实例化（组件实例语义）。
            let first = other.chars().next().unwrap_or('\0');
            if !first.is_ascii_uppercase() {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "unknown view element '{other}' (expected row/column/text/button or a Node instance)"
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            let props = parse_ui_prop_block(
                cx,
                &format!("instance '{elem_name}' of '{other}'"),
                allow_embed,
            )?;
            Ok(UiElement::Instance {
                node: other.to_string(),
                name: elem_name,
                props,
            })
        }
    }
}

/// PLAN-049：`view-2d name { props+elements }` / `view-3d name { props+elements }`
/// 嵌入区域。调用方已窥探 View+Sub+marker（未消费）；本函数消费
/// View/Sub/marker、区域名与 `{ props+elements }`（属性行与 2D/3D 场景
/// 元素按行交错；`head:` 为属性行，否则按对应维度元素解析——区域内禁止
/// 跨维场景 parent，挂载关系由降层裁决）。
fn parse_view_embed_element(
    cx: &mut dyn DomainParseContext,
    marker: &str,
) -> AutoResult<UiElement> {
    let start = cx.cur_pos();
    cx.advance(); // View
    cx.expect(TokenKind::Sub)?;
    let m = marker.to_string();
    cx.advance();
    let elem_name = cx.cur_text();
    if elem_name.is_empty() {
        return Err(SyntaxError::Generic {
            message: "view element name expected".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    cx.advance();
    let (props, elements) = if m.starts_with('2') {
        let (props, elements, _span) = parse_embed_body_2d(cx)?;
        (props, EmbedBody::D2(elements))
    } else {
        let (props, elements, _span) = parse_embed_body_3d(cx)?;
        (props, EmbedBody::D3(elements))
    };
    let span = super::ast::span_of(start);
    match elements {
        EmbedBody::D2(elements) => Ok(UiElement::Embed2d {
            name: elem_name,
            props,
            elements,
            span,
        }),
        EmbedBody::D3(elements) => Ok(UiElement::Embed3d {
            name: elem_name,
            props,
            elements,
            span,
        }),
    }
}

enum EmbedBody {
    D2(Vec<View2dElement>),
    D3(Vec<View3dElement>),
}

/// 区域体内的一行是属性还是元素：`head:`（后随冒号）= 属性。
fn embed_head_is_prop(cx: &mut dyn DomainParseContext) -> bool {
    let cp = cx.checkpoint();
    cx.advance();
    let is_prop = cx.is_kind(TokenKind::Colon);
    let ok = cx.restore(cp).is_ok();
    ok && is_prop
}

/// `{ view-2d props+elements }`：属性行复用 UI 属性行文法，元素复用
/// `parse_view2d_element`。
fn parse_embed_body_2d(
    cx: &mut dyn DomainParseContext,
) -> AutoResult<(Vec<Prop>, Vec<View2dElement>, LightSpan)> {
    let start = cx.cur_pos();
    cx.expect(TokenKind::LBrace)?;
    let mut props = Vec::new();
    let mut elements = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        if embed_head_is_prop(cx) {
            props.push(parse_embed_prop_row(cx)?);
            continue;
        }
        elements.push(parse_view2d_element(cx)?);
    }
    cx.expect(TokenKind::RBrace)?;
    Ok((props, elements, super::ast::span_of(start)))
}

/// `{ view-3d props+elements }`：同 2D，元素为 `parse_view3d_element`。
fn parse_embed_body_3d(
    cx: &mut dyn DomainParseContext,
) -> AutoResult<(Vec<Prop>, Vec<View3dElement>, LightSpan)> {
    let start = cx.cur_pos();
    cx.expect(TokenKind::LBrace)?;
    let mut props = Vec::new();
    let mut elements = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        if embed_head_is_prop(cx) {
            props.push(parse_embed_prop_row(cx)?);
            continue;
        }
        elements.push(parse_view3d_element(cx)?);
    }
    cx.expect(TokenKind::RBrace)?;
    Ok((props, elements, super::ast::span_of(start)))
}

/// 区域内一行属性（`name: value`；空行/逗号宽容——与 UI 属性行同文）。
fn parse_embed_prop_row(cx: &mut dyn DomainParseContext) -> AutoResult<Prop> {
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
    let prop = Prop {
        name: pname,
        value,
        span: super::ast::span_of(start),
    };
    if cx.is_kind(TokenKind::Comma) {
        cx.advance();
    }
    Ok(prop)
}

/// `{ name: value ... }` UI 属性行块；空行/逗号宽容。叶子/实例属性块
/// 遇到 UI 元素头/UpperCamel 实例化 = 子元素出现，具名拒绝（不静默丢
/// 弃）；容器混合块由 parse_ui_mixed_block 接管（该处元素头让位子元素）。
fn parse_ui_prop_block(
    cx: &mut dyn DomainParseContext,
    owner: &str,
    allow_embed: bool,
) -> AutoResult<Vec<Prop>> {
    cx.expect(TokenKind::LBrace)?;
    let mut props = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        if is_ui_element_head(cx, &head, allow_embed) {
            return Err(SyntaxError::Generic {
                message: format!("{owner} takes no children"),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
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

/// row/column 混合块：props（小写名行）与子元素（UI 元素头/UpperCamel）
/// 按行交错；先 prop 后子元素为推荐书写序，不强制（group2d 同文）。
fn parse_ui_mixed_block(
    cx: &mut dyn DomainParseContext,
    allow_embed: bool,
) -> AutoResult<(Vec<Prop>, Vec<UiElement>)> {
    cx.expect(TokenKind::LBrace)?;
    let mut props = Vec::new();
    let mut children = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        if is_ui_element_head(cx, &head, allow_embed) {
            children.push(parse_view_ui_element(cx, allow_embed)?);
            continue;
        }
        let start = cx.cur_pos();
        let pname = cx.cur_text();
        if pname.is_empty() {
            return Err(SyntaxError::Generic {
                message: "container property name expected".to_string(),
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
    Ok((props, children))
}

/// group2d 混合块：props（小写名行）与子元素（元素头/UpperCamel）按行
/// 交错；先 prop 后子元素为推荐书写序，不强制。
fn parse_group_block(
    cx: &mut dyn DomainParseContext,
) -> AutoResult<(Vec<Prop>, Vec<View2dElement>)> {
    cx.expect(TokenKind::LBrace)?;
    let mut props = Vec::new();
    let mut children = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        if matches!(
            head.as_str(),
            "group2d" | "shape2d" | "solid2d" | "trigger2d" | "character2d" | "camera2d" | "if"
        ) || head.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        {
            children.push(parse_view2d_element(cx)?);
            continue;
        }
        let start = cx.cur_pos();
        let pname = cx.cur_text();
        if pname.is_empty() {
            return Err(SyntaxError::Generic {
                message: "group2d property name expected".to_string(),
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
    Ok((props, children))
}

/// `{ name: value ... }` 属性行块；空行/逗号宽容。遇到下一个 view 元素头
/// （group2d/shape2d 等冻结关键字或 UpperCamelCase 实例化）时停止 ——
/// 仅 group2d 的 props 后会跟子元素；属性名一律小写，无误伤。
pub(super) fn parse_prop_block(cx: &mut dyn DomainParseContext) -> AutoResult<Vec<Prop>> {
    cx.expect(TokenKind::LBrace)?;
    let mut props = Vec::new();
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) || cx.is_kind(TokenKind::EOF) {
            break;
        }
        let head = cx.cur_text();
        if matches!(
            head.as_str(),
            "group2d" | "shape2d" | "solid2d" | "trigger2d" | "character2d" | "camera2d" | "if"
        ) || head.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        {
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
pub(super) fn parse_prop_value(cx: &mut dyn DomainParseContext) -> AutoResult<PropValue> {
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
    // PLAN-030：handler 参数进解析作用域（此前 `Life.Tick(dt)` 的 dt 仅在
    // 二元链内经裸 Ident 豁免路径通过；裸实参形态（如 `hero.Move(dt)`）
    // 撞 E0201）。旧夹具不依赖参数未定义行为，作用域收紧不改变旧字节。
    let handler_params = match &pattern {
        EventPattern::Life { params, .. } => params.clone(),
        EventPattern::Member { params, .. } => params.clone(),
    };
    cx.enter_scope();
    for p in &handler_params {
        cx.bind_var(p, Type::Unknown);
    }
    let body = cx.parse_body()?;
    cx.exit_scope();
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
