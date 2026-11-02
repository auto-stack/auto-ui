//! PLAN-030 components-3d-bindings：具名引用参数面（声明/校验）。
//!
//! 四类只读名义端口（准确拼写 T-02 冻结）：`SlotRef3D`/`TemplateRef3D`/
//! `PairRef3D`/`ActorRef3D`，外加 017 血统的 `msg` 事件端口。端口值由根
//! 具名池、模板定义与静态实例路径在下降期解析为原数值 ABI（ParamVal 载体），
//! 作者不写 slot/pair/scene ID。本模块只承载类型判定与解析后的绑定表；
//! 档装配与生成在 lower_components3d_bindings。

use auto_parser_api::ast::Type;

/// 名义引用参数类别（含 017 血统 msg 事件端口）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefKind {
    /// `SlotRef3D`：根 objects3d 池槽位符号。
    Slot,
    /// `TemplateRef3D`：object3d 模板定义符号。
    Template,
    /// `PairRef3D`：根 triggers 池 pair 符号。
    Pair,
    /// `ActorRef3D`：静态同 Run 三维 Node 实例（leaf 行持有者）。
    Actor,
    /// `msg`：绑定 `.Ev(args)` 根成员事件的只读事件端口。
    MsgEvent,
}

/// 参数声明类型 → 名义引用类别；非引用类型返回 None（其余类型沿旧
/// ptype_of_3d 面）。旧档不调用本函数，拼写面零变化。
pub(crate) fn ref_kind_of(ty: &Type) -> Option<RefKind> {
    match ty {
        Type::User(d) => match d.name.as_str() {
            "SlotRef3D" => Some(RefKind::Slot),
            "TemplateRef3D" => Some(RefKind::Template),
            "PairRef3D" => Some(RefKind::Pair),
            "ActorRef3D" => Some(RefKind::Actor),
            _ => None,
        },
        // 017 血统：`msg` 参数解析为 Type::Unknown（node2d/parse.rs 976）。
        Type::Unknown => Some(RefKind::MsgEvent),
        _ => None,
    }
}

impl RefKind {
    /// 诊断用端口拼写。
    pub(crate) fn spell(self) -> &'static str {
        match self {
            RefKind::Slot => "SlotRef3D",
            RefKind::Template => "TemplateRef3D",
            RefKind::Pair => "PairRef3D",
            RefKind::Actor => "ActorRef3D",
            RefKind::MsgEvent => "msg",
        }
    }
}

/// 每实例解析后的引用绑定（下降期；数值即原 ABI 身份）。
#[derive(Debug, Clone)]
pub(crate) enum RefBinding {
    /// 池/模板符号 → 数值身份（slot 0..8 / pair 0..8 / 模板行 1-based）。
    Num(i64),
    /// actor 实例路径 → (目标定义名，目标 def_idx，目标定义前缀)。
    Actor {
        def: String,
        def_idx: usize,
        prefix: String,
    },
    /// msg 事件端口 → (接收方 recv fn 名，预绑实参已翻译文本；空 = 无参)。
    MsgEvent { call: String },
}
