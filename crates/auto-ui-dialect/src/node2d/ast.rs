//! Node2D 研究 AST（auto-ui PLAN-016 T-03）。
//!
//! 显式研究入口的领域表示：`node` 定义（params/model/msg/view-2d/on）、
//! `view-2d` 元素（solid2d/trigger2d/camera2d/实例/if 条件）、`Life.*`
//! 系统事件与前导点业务事件的混合 `on`。只服务 pilot 研究 lowering，
//! 不进入 `UiCode`、不改生产 widget/store 文法；复用共享
//! ModelBlock/MsgDecl/Core Expr/Body，不复制表达式或词法。

use auto_parser_api::ast::{Body, Expr, Type};
use auto_parser_api::error::pos_to_span;
use auto_parser_api::token::Pos;

use crate::{ModelBlock, MsgDecl};

/// 研究 dialect 的领域语句。当前仅 `node`。
#[derive(Debug, Clone)]
pub enum Node2DStmt {
    Node(NodeDecl),
}

/// 一个 node 定义：同作用域的参数、model 实例状态、匿名 msg 事件、
/// view-2d 呈现与 Life/业务事件处理。
#[derive(Debug, Clone)]
pub struct NodeDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub model: Option<ModelBlock>,
    pub msg: Option<MsgDecl>,
    pub view2d: Option<View2dBlock>,
    pub on: Option<OnBlock>,
    pub span: LightSpan,
}

/// `view-2d { ... }`：本 node 实例的直属二维呈现（Node 直属呈现，
/// 不是嵌入区域；多呈现选择器不在 PLAN-016 范围）。
#[derive(Debug, Clone)]
pub struct View2dBlock {
    pub elements: Vec<View2dElement>,
    pub span: LightSpan,
}

#[derive(Debug, Clone)]
pub enum View2dElement {
    /// `solid2d name { props }` —— 静态实体。
    Solid { name: String, props: Vec<Prop> },
    /// `trigger2d name { props }` —— 触发体。
    Trigger { name: String, props: Vec<Prop> },
    /// `character2d name { props }` —— 角色宿主（PLAN-017 研究端口：
    /// 声明式角色声明，映射 AG character 服务；按类型而非名字识别）。
    Character { name: String, props: Vec<Prop> },
    /// `camera2d name { props }` —— 相机。
    Camera { name: String, props: Vec<Prop> },
    /// `NodeName instance_name { props }` —— 同定义多实例的实例化点。
    Instance {
        node: String,
        name: String,
        props: Vec<Prop>,
    },
    /// `if cond { elements }` —— 条件分支；分支消失对应真实销毁。
    If {
        cond: Expr,
        then: Vec<View2dElement>,
    },
}

/// 元素属性 `name: value`；值是共享 Core 表达式或字面二维/多元组对。
#[derive(Debug, Clone)]
pub struct Prop {
    pub name: String,
    pub value: PropValue,
    pub span: LightSpan,
}

#[derive(Debug, Clone)]
pub enum PropValue {
    /// 单个 Core 表达式（字面量、路径、前导点成员、调用等）。
    One(Expr),
    /// `(e0, e1, ...)` 字面量对/组（position、bounds、view_size）。
    Pair(Vec<Expr>),
}

impl PropValue {
    /// 按序取组元；单表达式形式返回长度 1。
    pub fn exprs(&self) -> Vec<&Expr> {
        match self {
            PropValue::One(e) => vec![e],
            PropValue::Pair(es) => es.iter().collect(),
        }
    }
}

/// `on { ... }`：Life 系统事件与前导点业务事件的混合处理块。
#[derive(Debug, Clone)]
pub struct OnBlock {
    pub handlers: Vec<Handler>,
    pub span: LightSpan,
}

#[derive(Debug, Clone)]
pub enum EventPattern {
    /// `Life.Start` / `Life.Tick(dt)` —— 系统事件，不在 msg 声明。
    Life { event: String, params: Vec<String> },
    /// `.Collect(key)` —— 本 node 业务事件（匿名 msg 变体），前导点即
    /// 本实例成员引用。
    Member { name: String, params: Vec<String> },
}

impl EventPattern {
    /// 诊断/降层用的事件名（`"Life.Tick"` / `"Collect"`）。
    pub fn display_name(&self) -> String {
        match self {
            EventPattern::Life { event, .. } => format!("Life.{event}"),
            EventPattern::Member { name, .. } => name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Handler {
    pub pattern: EventPattern,
    pub body: Body,
    pub span: LightSpan,
}

/// 供诊断与降层使用的 span 构造。
/// span 的轻量形态：(offset, len)。研究 AST 不直接绑定 miette
/// SourceSpan 类型，诊断构造时经 `pos_to_span` 还原。
pub type LightSpan = (usize, usize);

pub(crate) fn span_of(pos: Pos) -> LightSpan {
    let s = pos_to_span(pos);
    (s.offset(), s.len())
}
