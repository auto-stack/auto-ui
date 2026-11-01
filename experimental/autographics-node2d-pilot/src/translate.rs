//! 语句/表达式 → 单文件 AG 用户 Auto 源的结构化翻译（合同 S3）。
//!
//! 有界研究面：只接受 fixture 实际使用的形态；任何未列形态都是硬错误
//! （需要修订合同），不静默改写。成员 `.x` 在解析层就是
//! `Dot(Ident("self"), x)`；服务映射冻结：
//! - `input.axis(Left, Right)` → `(ag_action_held(1) - ag_action_held(0))`
//! - `refs.body.move_and_slide((dx, dy))` → `ag_char_move(dx, dy, 0.002)`
//! - `refs.body.grounded(q)` / `refs.body.support(q)` → `ag_char_grounded/support(q)`
//!   （PLAN-019 character-2d/camera-2d 服务拼写）
//! - `refs.body.move(x, y)` → `ag_object_move(<实例 handle>, x, y, 0.0)`
//! - `emit .Ev(args)` → 绑定表投递（lower 层注入目标调用文本）

use auto_parser_api::ast::{Arg, Expr, Op, Stmt};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum LowerError {
    Unsupported { what: String, line: usize },
}

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LowerError::Unsupported { what, line } => {
                write!(f, "unsupported research construct at line {line}: {what}")
            }
        }
    }
}

/// 成员存储上下文：`.field` 到生成存储的映射。
#[derive(Debug, Clone)]
pub enum Target {
    /// 根 node：标量存储 `r_<field>`。
    Root,
    /// 非 root 实例：SoA 数组 `<prefix>_<field>[<index>]`，
    /// handle 为 `<prefix>_handle[<index>]`。
    Instance { prefix: String, index: String },
}

pub struct Ctx<'a> {
    pub target: Target,
    /// `emit .Ev` → 生成调用文本（绑定表投递）。
    pub emit_hook: Box<dyn Fn(&str) -> Result<String, LowerError> + 'a>,
    /// 枚举路径求值：`Key.Amber` → `KEY_AMBER`。
    pub enum_path: Box<dyn Fn(&str, &str) -> Option<String> + 'a>,
    /// PLAN-017（g11-2d profile）：完整差值轴。false 保持 016 冻结映射
    /// `ag_action_held(1)`（legacy 字节兼容）。
    pub axis_differential: bool,
    /// PLAN-017：参数成员覆盖（表达式绑定参数在 owner 上下文内联求值，
    /// 如 `enabled`/actor 端口）。None 时回落 SoA 数组映射。
    pub member_hook: Option<Box<dyn Fn(&str) -> Option<String> + 'a>>,
    /// PLAN-017：移动标记全局变量名（`refs.<elem>.move_and_slide` 前置
    /// 置位，事件段据此选择移动 1 条/静止 3 条读取）。legacy 为 None。
    pub moved_flag: Option<String>,
    /// PLAN-017：refs.<elem> 接受任意本定义 view 元素名（legacy 冻结
    /// 仅 `body`）。
    pub refs_any_elem: bool,
    /// PLAN-020（components-2d）：裸纯函数调用的模块作用域解析。
    /// Some(限定名) 按返回文本降层；None/未命中保持逐字（旧档字节不变）。
    pub call_scope: Option<Box<dyn Fn(&str) -> Option<String> + 'a>>,
    /// PLAN-020：子实例事件投递 `emit <child>.Ev(args)` → 调用头部文本
    /// （含接收下标实参，未闭合；模板续接 `, args)`）。None 时该形态保持
    /// unsupported（旧档不变）。
    pub child_emit: Option<Box<dyn Fn(&str, &str, usize) -> Result<String, LowerError> + 'a>>,
    /// PLAN-021（queries-2d）：typed 查询服务环境。None 保持旧档行为
    /// （query2d 构造在 plan 层具名拒绝，旧字节不变）。
    pub query: Option<QueryEnv<'a>>,
    /// PLAN-022（sprites-2d）：typed 渲染服务环境。None 保持旧档行为
    /// （render2d/input.held 保持 unsupported，旧字节不变）。
    pub sprites: Option<SpriteEnv<'a>>,
    /// PLAN-024（components-3d）：v1 空间服务环境（refs/self 行句柄）。
    /// None 保持旧档行为（place/shift/show 等保持 unsupported，旧字节不变）。
    pub space3: Option<SpaceEnv<'a>>,
    /// PLAN-025（presentation-2d）：typed 声音请求环境（cue 名 → scene
    /// cue id；降层映射，作者不写数字 ID）。None 保持旧档行为（sfx.play
    /// 保持 unsupported，旧字节不变）。
    pub sfx: Option<SfxEnv<'a>>,
}

/// PLAN-025：typed 声音请求环境。cue 表 = presentation2d 块声明序。
pub struct SfxEnv<'a> {
    pub cues: &'a BTreeMap<String, usize>,
}

/// PLAN-024：view-3d 元素行句柄环境。refs：本定义 view-3d 元素名 →
/// 行表达式（root = 1-based 序数字面量；实例 = `<prefix>_<elem>_rows[i]`）。
/// 行句柄 = driver `hierarchy::ordered()` 先序 + 1（v1 节点 handle 基 1）。
pub struct SpaceEnv<'a> {
    pub refs: &'a BTreeMap<String, String>,
    pub self_row: Option<String>,
}

/// PLAN-022：图像帧表（生成侧全局数组 `{image}_fx/fy/fw/fh`；序 = 帧资源
/// 声明序，ordinal 即数组下标）。
#[derive(Clone, Debug)]
pub struct FrameTable {
    pub image: String,
    pub len: usize,
}

/// PLAN-022：render2d 服务解析面（下降期常量；plan 层已验证目标存在）。
pub struct SpriteEnv<'a> {
    /// `render2d.spawn(<elem>, x, y)`：根视图元素名 → (slot, 模板行 id)。
    pub spawns: &'a BTreeMap<String, (i64, i64)>,
    /// 本工程引用到的图像帧表（生成数组前缀 = 图像资源名）。
    pub frames: &'a BTreeMap<String, FrameTable>,
}

/// PLAN-021：typed 查询结果种类（同一世界共享 latest slot，q 即 token）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QueryKind {
    Cast,
    Sweep,
}

/// PLAN-021：具名行引用目标。writable=false = character 行（位置由角色
/// 服务拥有：shift/place 具名拒绝；show/world/local 仍合法）。
#[derive(Clone, Copy, Debug)]
pub struct RefRow {
    pub row: i64,
    pub writable: bool,
}

/// PLAN-021：queries-2d handler 翻译环境。binds 是本 handler 内
/// `let q = query2d.…` 的有序绑定（作用域由 handler 边界的
/// [`translate_handler`] 清理）。
pub struct QueryEnv<'a> {
    pub refs: &'a BTreeMap<String, RefRow>,
    pub self_row: Option<String>,
    pub self_writable: bool,
    pub binds: &'a std::cell::RefCell<Vec<(String, QueryKind)>>,
}

/// handler 边界包装：清空 typed 结果绑定后翻译整个 handler 体。
pub fn translate_handler(
    stmts: &[Stmt],
    ctx: &Ctx,
    out: &mut String,
    indent: usize,
) -> Result<(), LowerError> {
    if let Some(q) = &ctx.query {
        q.binds.borrow_mut().clear();
    }
    translate_stmts(stmts, ctx, out, indent)
}

/// 语法识别：query2d.segment/sweep 调用（用于 let 绑定分类）。
pub fn query_call_kind(e: &Expr) -> Option<QueryKind> {
    if let Expr::Call(c) = e {
        if let Expr::Dot(inner, m) = &*c.name {
            if let Expr::Ident(base) = &**inner {
                if base == "query2d" {
                    return match m.as_str() {
                        "segment" => Some(QueryKind::Cast),
                        "sweep" => Some(QueryKind::Sweep),
                        _ => None,
                    };
                }
            }
        }
    }
    None
}

fn line_of(e: &Expr) -> usize {
    match e {
        Expr::Call(c) => c.pos.as_ref().map(|p| p.line).unwrap_or(0),
        _ => 0,
    }
}

fn unsupported(what: impl Into<String>, e: &Expr) -> LowerError {
    LowerError::Unsupported {
        what: what.into(),
        line: line_of(e),
    }
}

fn op_text(op: &Op) -> Option<&'static str> {
    use Op::*;
    Some(match op {
        Asn => "=",
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Eq => "==",
        Neq => "!=",
        Gt => ">",
        Lt => "<",
        Ge => ">=",
        Le => "<=",
        _ => return None,
    })
}

fn member_ref(field: &str, ctx: &Ctx) -> Result<String, LowerError> {
    if let Some(hook) = &ctx.member_hook {
        if let Some(text) = hook(field) {
            return Ok(text);
        }
    }
    Ok(match &ctx.target {
        Target::Root => format!("r_{field}"),
        Target::Instance { prefix, index } => format!("{prefix}_{field}[{index}]"),
    })
}

pub fn translate_stmts(
    stmts: &[Stmt],
    ctx: &Ctx,
    out: &mut String,
    indent: usize,
) -> Result<(), LowerError> {
    for s in stmts {
        translate_stmt(s, ctx, out, indent)?;
    }
    Ok(())
}

fn pad(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn translate_stmt(s: &Stmt, ctx: &Ctx, out: &mut String, indent: usize) -> Result<(), LowerError> {
    match s {
        Stmt::Expr(e) => {
            // PLAN-024：v1 空间写语句（place/shift/show；refs.<elem>/self）。
            // 值位置使用具名拒绝（place 展开三条 ag_set_local）。
            if let Some(texts) = space3_stmt(e, ctx)? {
                for t in texts {
                    pad(out, indent);
                    out.push_str(&t);
                    out.push('\n');
                }
                return Ok(());
            }
            // PLAN-021：`place(x, y)` 是语句形态（一次作者语句 → 两条
            // ag_set_local）；值位置使用具名拒绝。
            if let Some(texts) = row_place_pair(e, ctx)? {
                for t in texts {
                    pad(out, indent);
                    out.push_str(&t);
                    out.push('\n');
                }
                return Ok(());
            }
            // PLAN-022：`render2d.frame(tok, img, ord, flip)` 是语句形态
            // （帧表 if-chain + 具名 fault 42 兜底；值位置具名拒绝）。
            if let Some(text) = render_frame_stmt(e, ctx)? {
                for line in text {
                    pad(out, indent);
                    out.push_str(&line);
                    out.push('\n');
                }
                return Ok(());
            }
            pad(out, indent);
            translate_expr(e, ctx, out)?;
            out.push('\n');
            Ok(())
        }
        Stmt::If(if_) => {
            let last = if_.branches.len().saturating_sub(1);
            for (i, branch) in if_.branches.iter().enumerate() {
                pad(out, indent);
                if i == 0 {
                    out.push_str("if ");
                } else {
                    out.push_str("} else if ");
                }
                translate_expr(&branch.cond, ctx, out)?;
                out.push_str(" {\n");
                translate_stmts(&branch.body.stmts, ctx, out, indent + 1)?;
                if i == last && if_.else_.is_none() {
                    pad(out, indent);
                    out.push_str("}\n");
                }
            }
            if let Some(else_) = &if_.else_ {
                pad(out, indent);
                out.push_str("} else {\n");
                translate_stmts(&else_.stmts, ctx, out, indent + 1)?;
                pad(out, indent);
                out.push_str("}\n");
            }
            Ok(())
        }
        Stmt::Store(v) => {
            pad(out, indent);
            let kw = match v.kind {
                auto_parser_api::ast::StoreKind::Var => "var",
                _ => "let",
            };
            // PLAN-021：typed 查询结果只允许 `let q = query2d.…` 绑定；
            // var 形态具名拒绝（结果不是可复制持有的普通 int）。
            if let Some(kind) = query_call_kind(&v.expr) {
                if matches!(v.kind, auto_parser_api::ast::StoreKind::Var) {
                    return Err(unsupported(
                        "query results bind with 'let' only ('var' would claim mutable ownership of a shared latest-slot token)",
                        &v.expr,
                    ));
                }
                out.push_str(kw);
                out.push(' ');
                out.push_str(&v.name.to_string());
                out.push_str(" = ");
                translate_expr(&v.expr, ctx, out)?;
                out.push('\n');
                if let Some(q) = &ctx.query {
                    q.binds.borrow_mut().push((v.name.to_string(), kind));
                }
                return Ok(());
            }
            out.push_str(kw);
            out.push(' ');
            out.push_str(&v.name.to_string());
            out.push_str(" = ");
            translate_expr(&v.expr, ctx, out)?;
            out.push('\n');
            Ok(())
        }
        Stmt::Return(e) => {
            pad(out, indent);
            out.push_str("return ");
            translate_expr(e, ctx, out)?;
            out.push('\n');
            Ok(())
        }
        Stmt::EmptyLine(_) | Stmt::Comment(_) => Ok(()),
        other => Err(LowerError::Unsupported {
            what: format!("statement {other:?}"),
            line: 0,
        }),
    }
}

fn translate_expr(e: &Expr, ctx: &Ctx, out: &mut String) -> Result<(), LowerError> {
    match e {
        Expr::Int(v) => {
            out.push_str(&v.to_string());
            Ok(())
        }
        Expr::Float(_, raw) => {
            out.push_str(raw);
            Ok(())
        }
        Expr::Bool(b) => {
            out.push_str(if *b { "1" } else { "0" });
            Ok(())
        }
        Expr::Ident(n) => {
            out.push_str(n);
            Ok(())
        }
        Expr::Dot(obj, name) => {
            // `.field`（self 成员）、枚举路径、refs 服务、emit 标记。
            match &**obj {
                Expr::Ident(base) => match base.as_str() {
                    "self" => {
                        out.push_str(&member_ref(name, ctx)?);
                        Ok(())
                    }
                    "emit" => {
                        let call = (ctx.emit_hook)(name)?;
                        out.push_str(&call);
                        Ok(())
                    }
                    _ => {
                        // PLAN-021：裸 `refs.<inst>` 值 = 具名行 handle
                        // （行 ordinal 字面量；plan 层已验证名字与行存在）。
                        if base == "refs" {
                            if let Some(q) = &ctx.query {
                                if let Some(row) = q.refs.get(name.as_str()) {
                                    out.push_str(&row.row.to_string());
                                    return Ok(());
                                }
                            }
                        }
                        if let Some(text) = (ctx.enum_path)(base, name) {
                            out.push_str(&text);
                            Ok(())
                        } else {
                            Err(unsupported(format!("path {base}.{name}"), e))
                        }
                    }
                },
                _ => Err(unsupported(format!("member path .{name} of {obj:?}"), e)),
            }
        }
        Expr::Call(call) => {
            // input.axis(Left, Right) → 差值轴（每方向一次 held 计费）。
            if let Expr::Dot(inner, m) = &*call.name {
                if let Expr::Ident(base) = &**inner {
                    if base == "input" && m == "axis" {
                        let sides: Vec<String> = call
                            .args
                            .args
                            .iter()
                            .map(|a| match a {
                                Arg::Pos(Expr::Ident(n)) => Ok(n.to_string()),
                                _ => Err(unsupported("input.axis args", e)),
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        if sides.len() != 2 || sides[0] != "Left" || sides[1] != "Right" {
                            return Err(unsupported(format!("input.axis({sides:?})"), e));
                        }
                        if ctx.axis_differential {
                            // PLAN-017 冻结映射：Left=-1、Right=+1、两者/都无=0；
                            // 两次计费 held 读取（预算算式 held2）。
                            out.push_str("(ag_action_held(1) - ag_action_held(0))");
                        } else {
                            // 016 冻结研究输入模型（合同 S3）：axis(Left, Right)
                            // 映射为正向 held 位单次计费；legacy 字节不变。
                            out.push_str("ag_action_held(1)");
                        }
                        return Ok(());
                    }
                    // PLAN-022（sprites-2d）：input.held(Action) → 受信输入
                    // 服务的单次 held 读（计费 1；动作号沿受信冻结表
                    // 0=Left/1=Right/2=Jump/3=Enter）。
                    if base == "input" && m == "held" {
                        let names: Vec<String> = call
                            .args
                            .args
                            .iter()
                            .map(|a| match a {
                                Arg::Pos(Expr::Ident(n)) => Ok(n.to_string()),
                                _ => Err(unsupported("input.held args", e)),
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        if names.len() != 1 {
                            return Err(unsupported(format!("input.held({names:?})"), e));
                        }
                        let action = match names[0].as_str() {
                            "Left" => 0,
                            "Right" => 1,
                            "Jump" => 2,
                            "Enter" => 3,
                            other => {
                                return Err(unsupported(
                                    format!(
                                    "input.held({other}) (research surface: Left/Right/Jump/Enter)"
                                ),
                                    e,
                                ))
                            }
                        };
                        out.push_str(&format!("ag_action_held({action})"));
                        return Ok(());
                    }
                    // PLAN-017：input.pressed(Enter) → pressed 边沿（消耗性读取，
                    // 计费 1）。PLAN-019 扩展 Jump（G06 走跳沿），动作号沿
                    // 受信 character 服务冻结表 0=Left/1=Right/2=Jump/3=Enter。
                    if base == "input" && m == "pressed" {
                        let names: Vec<String> = call
                            .args
                            .args
                            .iter()
                            .map(|a| match a {
                                Arg::Pos(Expr::Ident(n)) => Ok(n.to_string()),
                                _ => Err(unsupported("input.pressed args", e)),
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        if names.len() != 1 {
                            return Err(unsupported(format!("input.pressed({names:?})"), e));
                        }
                        let action = match names[0].as_str() {
                            "Enter" => 3,
                            "Jump" => 2,
                            other => {
                                return Err(unsupported(
                                    format!(
                                        "input.pressed({other}) (research surface: Enter/Jump)"
                                    ),
                                    e,
                                ))
                            }
                        };
                        out.push_str(&format!("ag_action_pressed({action})"));
                        return Ok(());
                    }
                    // PLAN-017：round.request() → ag_round_request()（计费 1，
                    // accepted 候选由宿主消费）。
                    if base == "round" && m == "request" {
                        if !call.args.args.is_empty() {
                            return Err(unsupported("round.request arity", e));
                        }
                        out.push_str("ag_round_request()");
                        return Ok(());
                    }
                    // PLAN-025（presentation-2d）：typed 声音请求
                    // sfx.play(<cue>, <gain>) → 生成收集器 sfx_play(<cue_id>,
                    // <gain>)（ordinal/缓冲为生成基础设施，0 计费；cue 域/
                    // 增益/容量界由原 API 运行时具名强制）。
                    if base == "sfx" && m == "play" {
                        let Some(env) = &ctx.sfx else {
                            return Err(unsupported(
                                "sfx.play is only available in the presentation-2d profile",
                                e,
                            ));
                        };
                        if call.args.args.len() != 2 {
                            return Err(unsupported("sfx.play(<cue>, <gain>) arity", e));
                        }
                        // cue 名为字符串字面量（typed 引用：编译器映射为
                        // scene cue id，作者不写数字 ID；跨模块可解析）。
                        let cue = match &call.args.args[0] {
                            Arg::Pos(Expr::Str(s)) => s.to_string(),
                            other => {
                                return Err(unsupported(
                                    format!("sfx.play cue must be a string literal naming a presentation2d cue, got {other:?}"),
                                    e,
                                ))
                            }
                        };
                        let Some(id) = env.cues.get(cue.as_str()).copied() else {
                            return Err(unsupported(
                                format!(
                                    "unknown sfx cue '{cue}' (cue id = presentation2d cue declaration order; declared: {})",
                                    env.cues.keys().cloned().collect::<Vec<_>>().join(", ")
                                ),
                                e,
                            ));
                        };
                        let mut gain = String::new();
                        match &call.args.args[1] {
                            Arg::Pos(x) => translate_expr(x, ctx, &mut gain)?,
                            other => {
                                return Err(unsupported(format!("sfx.play gain {other:?}"), e))
                            }
                        }
                        out.push_str(&format!("sfx_play({id}, {gain})"));
                        return Ok(());
                    }
                    // PLAN-021（queries-2d）：空间输入沿 transport 全局
                    // （`_ag_action(space,x,y,toggle)` 喂入；space==2 = 2D
                    // 游戏面）。不占用 character action 位。
                    if base == "input" && matches!(m.as_str(), "space" | "x" | "y") {
                        if ctx.query.is_none() {
                            return Err(unsupported(
                                format!("input.{m}() is only available in the queries-2d profile"),
                                e,
                            ));
                        }
                        if !call.args.args.is_empty() {
                            return Err(unsupported(format!("input.{m} arity"), e));
                        }
                        out.push_str(match m.as_str() {
                            "space" => "ag_input_space()",
                            "x" => "ag_input_x()",
                            _ => "ag_input_y()",
                        });
                        return Ok(());
                    }
                    // PLAN-021（queries-2d）：self 行寻址（本实例 primary
                    // 静态行）。root 无实例行；character 行只读（位置由
                    // 角色服务拥有）。
                    if base == "self"
                        && matches!(
                            m.as_str(),
                            "row" | "world" | "local" | "place" | "shift" | "show"
                        )
                    {
                        return translate_self_row_op(m, e, call, ctx, out);
                    }
                    // PLAN-021（queries-2d）：typed 结果 getter（仅结果
                    // 变量；kind 错配 plan 层已具名拒绝，这里按绑定种类
                    // 机械映射原 API）。
                    if let Some(q) = &ctx.query {
                        if let Some(kind) = q
                            .binds
                            .borrow()
                            .iter()
                            .rev()
                            .find(|(n, _)| n == base.as_str())
                            .map(|(_, k)| *k)
                        {
                            return translate_query_getter(base, m, e, call, ctx, out, kind);
                        }
                    }
                }
            }
            // PLAN-021（queries-2d）：query2d.segment/sweep 查询调用。
            if let Expr::Dot(inner, m) = &*call.name {
                if let Expr::Ident(base) = &**inner {
                    if base == "query2d" {
                        return translate_query_call(m, e, call, ctx, out);
                    }
                }
            }
            // PLAN-022（sprites-2d）：render2d.spawn/despawn（frame 是语句
            // 形态，在 translate_stmt 展开；值位置具名拒绝）。
            if let Expr::Dot(inner, m) = &*call.name {
                if let Expr::Ident(base) = &**inner {
                    if base == "render2d" && matches!(m.as_str(), "spawn" | "despawn") {
                        return translate_render_call(m, e, call, ctx, out);
                    }
                }
            }
            // refs.<elem> 服务调用（move_and_slide 等）。
            if let Expr::Dot(inner, op) = &*call.name {
                if let Expr::Dot(boxed_root, mid) = &**inner {
                    if matches!(&**boxed_root, Expr::Ident(r) if r == "refs")
                        && (ctx.refs_any_elem || mid == "body")
                    {
                        // PLAN-024：v1 空间服务优先（place/shift/show 值位置
                        // 具名拒绝；local/world/visible/row 读取）。
                        if ctx.space3.is_some() {
                            return translate_space3_call(op, mid, e, ctx, out);
                        }
                        return translate_refs_body(op, mid, e, ctx, out);
                    }
                }
            }
            // PLAN-024：self 空间读取（self.local/self.world/self.visible/
            // self.row；写形态已在语句层展开）。self 平移写属于 021 合同，
            // 3D 档 self.shift 已在语句层展开，此处仅剩余读取形态。
            if let Expr::Dot(inner, op) = &*call.name {
                if let Expr::Ident(base) = &**inner {
                    if base == "self"
                        && ctx.space3.is_some()
                        && matches!(op.as_str(), "local" | "world" | "visible" | "row")
                    {
                        return translate_space3_self(op, e, ctx, out);
                    }
                }
            }
            // PLAN-020：子实例事件投递（选择器形态 `<child>.Ev(args)`，含
            // `emit .Ev(args)` 前导点形态 child=="emit"）。在 input/refs 之后
            // 判定， hook 缺席时保持旧 unsupported 文本。
            if let Expr::Dot(inner, ev) = &*call.name {
                if let Expr::Ident(child) = &**inner {
                    if let Some(hook) = &ctx.child_emit {
                        let mut args = Vec::new();
                        for a in &call.args.args {
                            match a {
                                Arg::Pos(x) => {
                                    let mut buf = String::new();
                                    translate_expr(x, ctx, &mut buf)?;
                                    args.push(buf);
                                }
                                other => return Err(unsupported(format!("emit arg {other:?}"), e)),
                            }
                        }
                        // 钩子返回调用头部（接收下标实参与分隔符在内，
                        // 未闭合；PLAN-024 修复 root 直呼形态的括号缺失——
                        // 旧实例投递字节不变）。模板续接实参并闭合。
                        let call_text = (hook)(child, ev, args.len())?;
                        out.push_str(&call_text);
                        out.push_str(&args.join(", "));
                        out.push(')');
                        return Ok(());
                    }
                }
            }
            // 普通调用：池化纯函数（patrol_y）逐字参数翻译；PLAN-020 档先经
            // 模块作用域解析 hook（未命中或 hook 缺席保持逐字）。
            match &*call.name {
                Expr::Ident(fname) => {
                    let scoped = match &ctx.call_scope {
                        Some(hook) => hook(fname).unwrap_or_else(|| fname.to_string()),
                        None => fname.to_string(),
                    };
                    out.push_str(&scoped);
                    out.push('(');
                    for (i, a) in call.args.args.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        match a {
                            Arg::Pos(x) => translate_expr(x, ctx, out)?,
                            other => return Err(unsupported(format!("arg {other:?}"), e)),
                        }
                    }
                    out.push(')');
                    Ok(())
                }
                other => Err(unsupported(format!("call target {other:?}"), e)),
            }
        }
        Expr::Unary(op, inner) => {
            if matches!(op, Op::Sub) {
                out.push_str("(-");
                translate_expr(inner, ctx, out)?;
                out.push(')');
                Ok(())
            } else {
                Err(unsupported(format!("unary {op:?}"), e))
            }
        }
        Expr::Bina(l, op, r) => {
            let text = op_text(op).ok_or_else(|| unsupported(format!("operator {op:?}"), e))?;
            // PLAN-021：typed 结果变量不可再赋值（共享 latest slot 的
            // token 不是可写状态）。
            if *op == Op::Asn {
                if let Expr::Ident(name) = &**l {
                    if let Some(q) = &ctx.query {
                        if q.binds.borrow().iter().any(|(n, _)| n == name) {
                            return Err(unsupported(
                                format!("query result '{name}' cannot be reassigned (read the values out instead)"),
                                e,
                            ));
                        }
                    }
                }
            }
            let paren = *op != Op::Asn; // 赋值是语句形态，不加括号。
            if paren {
                out.push('(');
            }
            translate_expr(l, ctx, out)?;
            out.push(' ');
            out.push_str(text);
            out.push(' ');
            translate_expr(r, ctx, out)?;
            if paren {
                out.push(')');
            }
            Ok(())
        }
        Expr::Tuple(items) => {
            // 处理器体内的裸元组只允许作为已解包的服务参数（不会到这里）；
            // 保险起见按括号组直译。
            out.push('(');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                translate_expr(item, ctx, out)?;
            }
            out.push(')');
            Ok(())
        }
        other => Err(unsupported(format!("expression {other:?}"), e)),
    }
}

/// `refs.<elem>.<op>(args)`：`move_and_slide((dx,dy))` → char 扫掠；
/// `move_to(x,y)` → 实例对象移动。其余拒绝。
fn translate_refs_body(
    op: &str,
    elem: &str,
    e: &Expr,
    ctx: &Ctx,
    out: &mut String,
) -> Result<(), LowerError> {
    let _ = elem;
    let call = match e {
        Expr::Call(c) => c,
        _ => return Err(unsupported("refs.<elem> non-call", e)),
    };
    match op {
        "move_and_slide" => {
            if call.args.args.len() != 1 {
                return Err(unsupported("move_and_slide arity", e));
            }
            let pair = match &call.args.args[0] {
                Arg::Pos(Expr::Tuple(items)) if items.len() == 2 => items,
                _ => return Err(unsupported("move_and_slide pair arg", e)),
            };
            // PLAN-017：本 tick 真实扫掠标记（事件段读取条数选择的依据，
            // 与原 001 的 moved 语义一致）。前置置位后紧邻扫掠调用。
            if let Some(flag) = &ctx.moved_flag {
                // 当前语句缩进 = translate_stmt 已写入 out 末尾的空格。
                let spaces = out.len() - out.trim_end_matches(' ').len();
                let indent: String = " ".repeat(spaces);
                out.push_str(&format!("{flag} = 1\n{indent}"));
            }
            out.push_str("ag_char_move(");
            translate_expr(&pair[0], ctx, out)?;
            out.push_str(", ");
            translate_expr(&pair[1], ctx, out)?;
            out.push_str(", 0.002)");
            Ok(())
        }
        "move_to" => {
            if call.args.args.len() != 2 {
                return Err(unsupported("move arity", e));
            }
            let handle = match &ctx.target {
                Target::Instance { prefix, index } => format!("{prefix}_handle[{index}]"),
                Target::Root => return Err(unsupported("refs.body.move on root", e)),
            };
            out.push_str("ag_object_move(");
            out.push_str(&handle);
            for a in &call.args.args {
                out.push_str(", ");
                match a {
                    Arg::Pos(x) => translate_expr(x, ctx, out)?,
                    other => return Err(unsupported(format!("move arg {other:?}"), e)),
                }
            }
            out.push_str(", 0.0)");
            Ok(())
        }
        // PLAN-019 character-2d/camera-2d：move_and_slide 结果的支撑判定。
        // `refs.<char>.grounded(q)` → `ag_char_grounded(q)`（计费 1，
        // token==0 读冻结初值、旧 token fault 13 全部由受信服务承担）。
        "grounded" | "support" => {
            if call.args.args.len() != 1 {
                return Err(unsupported(format!("{op} arity"), e));
            }
            let arg = match &call.args.args[0] {
                Arg::Pos(x) => x,
                other => return Err(unsupported(format!("{op} arg {other:?}"), e)),
            };
            out.push_str(if op == "grounded" {
                "ag_char_grounded("
            } else {
                "ag_char_support("
            });
            translate_expr(arg, ctx, out)?;
            out.push_str(")");
            Ok(())
        }
        // PLAN-021（queries-2d）：具名行 op（语句形态除 world/local 外
        // 均为 bare unit 调用；character 行位置写具名拒绝）。
        "place" => Err(unsupported(
            "place is statement-only (it writes both row axes)",
            e,
        )),
        "shift" => {
            let row = query_row_of(elem, ctx, e)?;
            if call.args.args.len() != 2 {
                return Err(unsupported("shift arity", e));
            }
            if !row.writable {
                return Err(unsupported(
                    format!("row '{elem}' is the character row; its position is owned by the character service"),
                    e,
                ));
            }
            out.push_str(&format!("ag_move_local({}, ", row.row));
            for (i, a) in call.args.args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                match a {
                    Arg::Pos(x) => translate_expr(x, ctx, out)?,
                    other => return Err(unsupported(format!("shift arg {other:?}"), e)),
                }
            }
            out.push_str(", 0.0)");
            Ok(())
        }
        "show" => {
            let row = query_row_of(elem, ctx, e)?;
            if call.args.args.len() != 1 {
                return Err(unsupported("show arity", e));
            }
            out.push_str(&format!("ag_set_visible({}, ", row.row));
            match &call.args.args[0] {
                Arg::Pos(x) => translate_expr(x, ctx, out)?,
                other => return Err(unsupported(format!("show arg {other:?}"), e)),
            }
            out.push(')');
            Ok(())
        }
        "world" => {
            let row = query_row_of(elem, ctx, e)?;
            if call.args.args.len() != 1 {
                return Err(unsupported("world arity", e));
            }
            out.push_str(&format!("ag_world({}, ", row.row));
            match &call.args.args[0] {
                Arg::Pos(x) => translate_expr(x, ctx, out)?,
                other => return Err(unsupported(format!("world arg {other:?}"), e)),
            }
            out.push(')');
            Ok(())
        }
        "local" => {
            let row = query_row_of(elem, ctx, e)?;
            if call.args.args.len() != 1 {
                return Err(unsupported("local arity", e));
            }
            out.push_str(&format!("ag_local({}, ", row.row));
            match &call.args.args[0] {
                Arg::Pos(x) => translate_expr(x, ctx, out)?,
                other => return Err(unsupported(format!("local arg {other:?}"), e)),
            }
            out.push(')');
            Ok(())
        }
        other => Err(unsupported(format!("refs.body.{other}"), e)),
    }
}

/// PLAN-021：refs 具名行解析（queries-2d 环境；未知引用具名拒绝）。
fn query_row_of(elem: &str, ctx: &Ctx, e: &Expr) -> Result<RefRow, LowerError> {
    let Some(q) = &ctx.query else {
        return Err(unsupported(
            "named row ops are only available in the queries-2d profile",
            e,
        ));
    };
    q.refs
        .get(elem)
        .copied()
        .ok_or_else(|| unsupported(format!("unknown query row reference 'refs.{elem}'"), e))
}

/// PLAN-021：解析具名行实参（refs.<inst> | self.row() | -1 字面量）。
fn query_handle_arg(
    a: &Arg,
    ctx: &Ctx,
    allow_minus_one: bool,
    e: &Expr,
) -> Result<String, LowerError> {
    match a {
        Arg::Pos(Expr::Int(v)) => {
            if allow_minus_one && *v == -1 {
                return Ok("-1".to_string());
            }
            Err(unsupported(
                "query handle argument must be a named row (refs.<inst> / self.row()) or -1",
                e,
            ))
        }
        Arg::Pos(Expr::Unary(Op::Sub, inner)) => {
            if allow_minus_one {
                if let Expr::Int(v) = &**inner {
                    if *v == 1 {
                        return Ok("-1".to_string());
                    }
                }
            }
            Err(unsupported(
                "query handle argument must be a named row (refs.<inst> / self.row()) or -1",
                e,
            ))
        }
        Arg::Pos(Expr::Dot(inner, name)) => {
            if let Expr::Ident(base) = &**inner {
                if base == "refs" {
                    if let Some(q) = &ctx.query {
                        if let Some(row) = q.refs.get(name.as_str()) {
                            return Ok(row.row.to_string());
                        }
                    }
                }
            }
            Err(unsupported(
                format!("unknown query row reference '{a:?}'"),
                e,
            ))
        }
        Arg::Pos(Expr::Call(c)) => {
            if let Expr::Dot(inner, m) = &*c.name {
                if let Expr::Ident(base) = &**inner {
                    if base == "self" && m == "row" && c.args.args.is_empty() {
                        if let Some(q) = &ctx.query {
                            if let Some(row) = &q.self_row {
                                return Ok(row.clone());
                            }
                        }
                        return Err(unsupported("self.row() has no instance row at the root", e));
                    }
                }
            }
            Err(unsupported("query handle argument form", e))
        }
        _ => Err(unsupported("query handle argument form", e)),
    }
}

/// PLAN-021：语句位置识别 `place(x, y)`（refs.<inst> 或 self 形态）→
/// 两条 `ag_set_local(row, axis, …)`。非 place 调用返回 None。
fn row_place_pair(e: &Expr, ctx: &Ctx) -> Result<Option<Vec<String>>, LowerError> {
    let Expr::Call(c) = e else { return Ok(None) };
    let Expr::Dot(inner, m) = &*c.name else {
        return Ok(None);
    };
    if m != "place" {
        return Ok(None);
    }
    let (row, writable, what): (String, bool, &str) = match &**inner {
        Expr::Dot(boxed_root, name) => {
            if !matches!(&**boxed_root, Expr::Ident(r) if r == "refs") {
                return Ok(None);
            }
            let Some(q) = &ctx.query else {
                return Err(unsupported(
                    "row placement is only available in the queries-2d profile",
                    e,
                ));
            };
            let Some(rr) = q.refs.get(name.as_str()) else {
                return Err(unsupported(
                    format!("unknown query row reference 'refs.{name}'"),
                    e,
                ));
            };
            (rr.row.to_string(), rr.writable, "refs")
        }
        Expr::Ident(base) if base == "self" => {
            let Some(q) = &ctx.query else {
                return Err(unsupported(
                    "row placement is only available in the queries-2d profile",
                    e,
                ));
            };
            let Some(row) = q.self_row.clone() else {
                return Err(unsupported("self.place has no instance row at the root", e));
            };
            (row, q.self_writable, "self")
        }
        _ => return Ok(None),
    };
    if !writable {
        return Err(unsupported(
            "this row is the character row; its position is owned by the character service",
            e,
        ));
    }
    if c.args.args.len() != 2 {
        return Err(unsupported(
            format!("{what}.place arity {}", c.args.args.len()),
            e,
        ));
    }
    let mut parts = Vec::new();
    for a in &c.args.args {
        match a {
            Arg::Pos(x) => {
                let mut t = String::new();
                translate_expr(x, ctx, &mut t)?;
                parts.push(t);
            }
            _ => return Err(unsupported("place arg", e)),
        }
    }
    Ok(Some(vec![
        format!("ag_set_local({row}, 0, {})", parts[0]),
        format!("ag_set_local({row}, 1, {})", parts[1]),
    ]))
}

/// PLAN-024：v1 空间行句柄解析（refs.<elem> 或 self；space3 环境必需）。
fn space3_row_of(inner: &Expr, e: &Expr, ctx: &Ctx) -> Result<String, LowerError> {
    let Some(env) = &ctx.space3 else {
        return Err(unsupported("view-3d row service outside components-3d", e));
    };
    match inner {
        Expr::Dot(boxed_root, name) => {
            if !matches!(&**boxed_root, Expr::Ident(r) if r == "refs") {
                return Err(unsupported("view-3d row reference form", e));
            }
            env.refs.get(name.as_str()).cloned().ok_or_else(|| {
                unsupported(
                    format!("unknown view-3d element reference 'refs.{name}'"),
                    e,
                )
            })
        }
        Expr::Ident(base) if base == "self" => env.self_row.clone().ok_or_else(|| {
            unsupported(
                "self has no view-3d row (definition has no view-3d element)",
                e,
            )
        }),
        _ => Err(unsupported("view-3d row reference form", e)),
    }
}

/// PLAN-024：v1 空间写语句（place/shift/show）。place 展开三条
/// `ag_set_local(row, axis, …)`；shift → `ag_move_local`；show →
/// `ag_set_visible`。非目标形态返回 None（语句层继续分派）。
fn space3_stmt(e: &Expr, ctx: &Ctx) -> Result<Option<Vec<String>>, LowerError> {
    let Expr::Call(c) = e else {
        return Ok(None);
    };
    let Expr::Dot(inner, m) = &*c.name else {
        return Ok(None);
    };
    if !matches!(m.as_str(), "place" | "shift" | "show") {
        return Ok(None);
    }
    // 仅 refs.<elem>/self 形态归本服务；其它基础（如 `refs.<q>.place` 在
    // 021 档）由 ctx.space3 缺席回退旧路径。
    let is_refs = matches!(&**inner, Expr::Dot(outer, _) if matches!(&**outer, Expr::Ident(r) if r == "refs"));
    let is_self = matches!(&**inner, Expr::Ident(b) if b == "self");
    if !is_refs && !is_self {
        return Ok(None);
    }
    if ctx.space3.is_none() {
        return Ok(None);
    }
    let row = space3_row_of(inner, e, ctx)?;
    let mut parts = Vec::new();
    for a in &c.args.args {
        match a {
            Arg::Pos(x) => {
                let mut t = String::new();
                translate_expr(x, ctx, &mut t)?;
                parts.push(t);
            }
            _ => return Err(unsupported(format!("{m} arg"), e)),
        }
    }
    match m.as_str() {
        "place" => {
            if parts.len() != 3 {
                return Err(unsupported(format!("place arity {}", parts.len()), e));
            }
            Ok(Some(vec![
                format!("ag_set_local({row}, 0, {})", parts[0]),
                format!("ag_set_local({row}, 1, {})", parts[1]),
                format!("ag_set_local({row}, 2, {})", parts[2]),
            ]))
        }
        "shift" => {
            if parts.len() != 3 {
                return Err(unsupported(format!("shift arity {}", parts.len()), e));
            }
            Ok(Some(vec![format!(
                "ag_move_local({row}, {}, {}, {})",
                parts[0], parts[1], parts[2]
            )]))
        }
        "show" => {
            if parts.len() != 1 {
                return Err(unsupported(format!("show arity {}", parts.len()), e));
            }
            Ok(Some(vec![format!("ag_set_visible({row}, {})", parts[0])]))
        }
        _ => unreachable!("space3 op dispatch"),
    }
}

/// PLAN-024：v1 空间读取（值位置）：`refs.<elem>.local(axis)/world(axis)/
/// visible()/row()`。写形态（place/shift/show）在语句层展开，这里具名拒绝。
fn translate_space3_call(
    op: &str,
    elem: &str,
    e: &Expr,
    ctx: &Ctx,
    out: &mut String,
) -> Result<(), LowerError> {
    let env = ctx
        .space3
        .as_ref()
        .ok_or_else(|| unsupported("view-3d service outside components-3d", e))?;
    let row = env.refs.get(elem).cloned().ok_or_else(|| {
        unsupported(
            format!("unknown view-3d element reference 'refs.{elem}'"),
            e,
        )
    })?;
    let call = match e {
        Expr::Call(c) => c,
        _ => return Err(unsupported("refs.<elem> non-call", e)),
    };
    let axis_arg = |call: &auto_parser_api::ast::Call| -> Result<String, LowerError> {
        if call.args.args.len() != 1 {
            return Err(unsupported(
                format!("{op} arity {}", call.args.args.len()),
                e,
            ));
        }
        match &call.args.args[0] {
            Arg::Pos(x) => {
                let mut t = String::new();
                translate_expr(x, ctx, &mut t)?;
                Ok(t)
            }
            _ => Err(unsupported(format!("{op} axis arg"), e)),
        }
    };
    match op {
        "place" | "shift" | "show" => Err(unsupported(
            format!("'{op}' is statement-only (it writes the row)"),
            e,
        )),
        "local" => {
            let axis = axis_arg(call)?;
            out.push_str(&format!("ag_local({row}, {axis})"));
            Ok(())
        }
        "world" => {
            let axis = axis_arg(call)?;
            out.push_str(&format!("ag_world({row}, {axis})"));
            Ok(())
        }
        "visible" => {
            if !call.args.args.is_empty() {
                return Err(unsupported("visible arity", e));
            }
            out.push_str(&format!("ag_visible({row})"));
            Ok(())
        }
        "row" => {
            if !call.args.args.is_empty() {
                return Err(unsupported("row arity", e));
            }
            out.push_str(&row);
            Ok(())
        }
        _ => Err(unsupported(
            format!(
                "unknown view-3d service 'refs.{elem}.{op}' (place/shift/show/local/world/visible/row)"
            ),
            e,
        )),
    }
}

/// PLAN-024：self 空间读取（值位置）：`self.local/self.world/self.visible/
/// self.row`。
fn translate_space3_self(
    op: &str,
    e: &Expr,
    ctx: &Ctx,
    out: &mut String,
) -> Result<(), LowerError> {
    let env = ctx
        .space3
        .as_ref()
        .ok_or_else(|| unsupported("view-3d service outside components-3d", e))?;
    let row = env.self_row.clone().ok_or_else(|| {
        unsupported(
            "self has no view-3d row (definition has no view-3d element)",
            e,
        )
    })?;
    let call = match e {
        Expr::Call(c) => c,
        _ => return Err(unsupported("self non-call", e)),
    };
    match op {
        "local" | "world" => {
            if call.args.args.len() != 1 {
                return Err(unsupported(
                    format!("self.{op} arity {}", call.args.args.len()),
                    e,
                ));
            }
            let mut t = String::new();
            match &call.args.args[0] {
                Arg::Pos(x) => translate_expr(x, ctx, &mut t)?,
                _ => return Err(unsupported(format!("self.{op} axis arg"), e)),
            }
            let f = if op == "local" {
                "ag_local"
            } else {
                "ag_world"
            };
            out.push_str(&format!("{f}({row}, {t})"));
            Ok(())
        }
        "visible" => {
            if !call.args.args.is_empty() {
                return Err(unsupported("self.visible arity", e));
            }
            out.push_str(&format!("ag_visible({row})"));
            Ok(())
        }
        "row" => {
            if !call.args.args.is_empty() {
                return Err(unsupported("self.row arity", e));
            }
            out.push_str(&row);
            Ok(())
        }
        _ => unreachable!("space3 self op dispatch"),
    }
}

/// PLAN-021：`query2d.segment(ignore, visible_only, ax, ay, bx, by)` /
/// `query2d.sweep(mover, visible_only, dx, dy)` → 原 `ag_cast_segment(2,…)` /
/// `ag_sweep_aabb(…, dx, dy, 0.0)`。2D 固定；z=0 隐含。plan 层已验证
/// 元数/实参形态/引用存在（模块/行/符号诊断），此处为机械映射。
fn translate_query_call(
    m: &str,
    e: &Expr,
    call: &auto_parser_api::ast::Call,
    ctx: &Ctx,
    out: &mut String,
) -> Result<(), LowerError> {
    if ctx.query.is_none() {
        return Err(unsupported(
            "query2d is only available in the queries-2d profile",
            e,
        ));
    }
    match m {
        "segment" => {
            if call.args.args.len() != 6 {
                return Err(unsupported(
                    format!(
                        "query2d.segment arity {} (want ignore, visible_only, ax, ay, bx, by)",
                        call.args.args.len()
                    ),
                    e,
                ));
            }
            let handle = query_handle_arg(&call.args.args[0], ctx, true, e)?;
            let nums = query_numeric_args(&call.args.args[2..6], ctx, e)?;
            out.push_str(&format!(
                "ag_cast_segment(2, {handle}, {}, {}, {}, 0.0, {}, {}, 0.0)",
                visible_only_text(&call.args.args[1], e)?,
                nums[0],
                nums[1],
                nums[2],
                nums[3]
            ));
            Ok(())
        }
        "sweep" => {
            if call.args.args.len() != 4 {
                return Err(unsupported(
                    format!(
                        "query2d.sweep arity {} (want mover, visible_only, dx, dy)",
                        call.args.args.len()
                    ),
                    e,
                ));
            }
            let handle = query_handle_arg(&call.args.args[0], ctx, false, e)?;
            let nums = query_numeric_args(&call.args.args[2..4], ctx, e)?;
            out.push_str(&format!(
                "ag_sweep_aabb({handle}, {}, {}, {}, 0.0)",
                visible_only_text(&call.args.args[1], e)?,
                nums[0],
                nums[1]
            ));
            Ok(())
        }
        other => Err(unsupported(
            format!("unknown query service 'query2d.{other}' (segment/sweep)"),
            e,
        )),
    }
}

fn visible_only_text(a: &Arg, e: &Expr) -> Result<String, LowerError> {
    match a {
        Arg::Pos(Expr::Int(v)) if *v == 0 || *v == 1 => Ok(v.to_string()),
        Arg::Pos(Expr::Bool(b)) => Ok(if *b { "1".to_string() } else { "0".to_string() }),
        _ => Err(unsupported(
            "visible_only must be the literal 0/1 (or true/false)",
            e,
        )),
    }
}

/// 位置数值实参 → 生成文本（机械映射；plan 层已验证元数）。
fn query_numeric_args(args: &[Arg], ctx: &Ctx, e: &Expr) -> Result<Vec<String>, LowerError> {
    let mut parts = Vec::new();
    for a in args {
        match a {
            Arg::Pos(x) => {
                let mut t = String::new();
                translate_expr(x, ctx, &mut t)?;
                parts.push(t);
            }
            _ => return Err(unsupported("query numeric arg", e)),
        }
    }
    Ok(parts)
}

/// PLAN-021：typed 结果 getter（cast5/sweep6，计费 1 进原 API）。
fn translate_query_getter(
    base: &str,
    m: &str,
    e: &Expr,
    call: &auto_parser_api::ast::Call,
    ctx: &Ctx,
    out: &mut String,
    kind: QueryKind,
) -> Result<(), LowerError> {
    let axis_arg = |call: &auto_parser_api::ast::Call| -> Result<Option<String>, LowerError> {
        match call.args.args.len() {
            0 => Ok(None),
            1 => match &call.args.args[0] {
                Arg::Pos(x) => {
                    let mut t = String::new();
                    translate_expr(x, ctx, &mut t)?;
                    Ok(Some(t))
                }
                _ => Err(unsupported("getter axis arg", e)),
            },
            n => Err(unsupported(format!("getter arity {n}"), e)),
        }
    };
    let (fn_name, axis) = match (kind, m) {
        (QueryKind::Cast, "hit") => ("ag_cast_hit", None),
        (QueryKind::Cast, "node") => ("ag_cast_node", None),
        (QueryKind::Cast, "t") => ("ag_cast_t", None),
        (QueryKind::Cast, "point") => ("ag_cast_point", Some(axis_arg(call)?)),
        (QueryKind::Cast, "normal") => ("ag_cast_normal", Some(axis_arg(call)?)),
        (QueryKind::Sweep, "hit") => ("ag_sweep_hit", None),
        (QueryKind::Sweep, "node") => ("ag_sweep_node", None),
        (QueryKind::Sweep, "t") => ("ag_sweep_t", None),
        (QueryKind::Sweep, "origin") => ("ag_sweep_origin", Some(axis_arg(call)?)),
        (QueryKind::Sweep, "normal") => ("ag_sweep_normal", Some(axis_arg(call)?)),
        (QueryKind::Sweep, "start_overlap") => ("ag_sweep_start_overlap", None),
        (QueryKind::Cast, other) => {
            return Err(unsupported(
                format!("'.{other}()' is a sweep result getter; '{base}' is a cast result"),
                e,
            ));
        }
        (QueryKind::Sweep, other) => {
            return Err(unsupported(
                format!("'.{other}()' is a cast result getter; '{base}' is a sweep result"),
                e,
            ));
        }
    };
    match axis {
        None => out.push_str(&format!("{fn_name}({base})")),
        Some(Some(axis)) => out.push_str(&format!("{fn_name}({base}, {axis})")),
        Some(None) => {
            return Err(unsupported(format!("{fn_name} needs an axis argument"), e));
        }
    }
    Ok(())
}

/// PLAN-021：self 行 op（本实例 primary 静态行）。root 无实例行；
/// character 行 shift/place 具名拒绝。
fn translate_self_row_op(
    m: &str,
    e: &Expr,
    call: &auto_parser_api::ast::Call,
    ctx: &Ctx,
    out: &mut String,
) -> Result<(), LowerError> {
    let Some(q) = &ctx.query else {
        return Err(unsupported(
            "self row ops are only available in the queries-2d profile",
            e,
        ));
    };
    let row = q
        .self_row
        .clone()
        .ok_or_else(|| unsupported("self.row() has no instance row at the root", e))?;
    let args = |n: usize| -> Result<Vec<String>, LowerError> {
        if call.args.args.len() != n {
            return Err(unsupported(
                format!("self.{m} arity {}", call.args.args.len()),
                e,
            ));
        }
        let mut t = String::new();
        let mut parts = Vec::new();
        for a in &call.args.args {
            match a {
                Arg::Pos(x) => {
                    translate_expr(x, ctx, &mut t)?;
                    parts.push(std::mem::take(&mut t));
                }
                _ => return Err(unsupported(format!("self.{m} arg"), e)),
            }
        }
        Ok(parts)
    };
    match m {
        "row" => {
            if !call.args.args.is_empty() {
                return Err(unsupported("self.row arity", e));
            }
            out.push_str(&row.to_string());
            Ok(())
        }
        "world" => {
            let a = args(1)?;
            out.push_str(&format!("ag_world({row}, {})", a[0]));
            Ok(())
        }
        "local" => {
            let a = args(1)?;
            out.push_str(&format!("ag_local({row}, {})", a[0]));
            Ok(())
        }
        "place" => Err(unsupported(
            "place is statement-only (it writes both row axes)",
            e,
        )),
        "shift" => {
            if !q.self_writable {
                return Err(unsupported(
                    "self row is the character row; its position is owned by the character service",
                    e,
                ));
            }
            let a = args(2)?;
            out.push_str(&format!("ag_move_local({row}, {}, {}, 0.0)", a[0], a[1]));
            Ok(())
        }
        "show" => {
            let a = args(1)?;
            out.push_str(&format!("ag_set_visible({row}, {})", a[0]));
            Ok(())
        }
        _ => unreachable!("self row op dispatch"),
    }
}

/// PLAN-022：`render2d.spawn(<elem>, x, y)` → `ag_object_spawn(slot, row, x, y, 0.0)`
/// （elem/ slot/行 id 为下降期常量，plan 层已验证目标与槽预算）；
/// `render2d.despawn(t)` → `ag_object_despawn(t)`。
fn translate_render_call(
    m: &str,
    e: &Expr,
    call: &auto_parser_api::ast::Call,
    ctx: &Ctx,
    out: &mut String,
) -> Result<(), LowerError> {
    let Some(env) = &ctx.sprites else {
        return Err(unsupported(
            "render2d is only available in the sprites-2d profile",
            e,
        ));
    };
    match m {
        "spawn" => {
            if call.args.args.len() != 3 {
                return Err(unsupported(
                    format!(
                        "render2d.spawn arity {} (want <elem>, x, y)",
                        call.args.args.len()
                    ),
                    e,
                ));
            }
            let elem = match &call.args.args[0] {
                Arg::Pos(Expr::Str(n)) => n.to_string(),
                _ => {
                    return Err(unsupported(
                        "render2d.spawn target must be a string literal element name",
                        e,
                    ))
                }
            };
            let (slot, row) = env.spawns.get(&elem).copied().ok_or_else(|| {
                unsupported(
                    format!("unknown render2d.spawn target '{elem}' (no bound sprite leaf)"),
                    e,
                )
            })?;
            out.push_str(&format!("ag_object_spawn({slot}, {row}, "));
            for a in &call.args.args[1..3] {
                match a {
                    Arg::Pos(x) => translate_expr(x, ctx, out)?,
                    other => return Err(unsupported(format!("spawn arg {other:?}"), e)),
                }
                out.push_str(", ");
            }
            out.push_str("0.0)");
            Ok(())
        }
        "despawn" => {
            if call.args.args.len() != 1 {
                return Err(unsupported(
                    format!("render2d.despawn arity {}", call.args.args.len()),
                    e,
                ));
            }
            out.push_str("ag_object_despawn(");
            match &call.args.args[0] {
                Arg::Pos(x) => translate_expr(x, ctx, out)?,
                other => return Err(unsupported(format!("despawn arg {other:?}"), e)),
            }
            out.push(')');
            Ok(())
        }
        other => Err(unsupported(format!("unknown render2d.{other}"), e)),
    }
}

/// PLAN-022：`render2d.frame(tok, img, ord, flip)` 语句形态 → 帧表 if-chain
/// （每分支恰一次 `ag_sprite_slot` 写；序数越界走具名 `_ag_fail(42, 0, ord)`，
/// 不取模不截断）。rect 一律来自命名图像的帧表；与槽模板图像不一致由旧受
/// 检边界（资产序数/矩形域）具名拒绝。非 frame 调用返回 None。
fn render_frame_stmt(e: &Expr, ctx: &Ctx) -> Result<Option<Vec<String>>, LowerError> {
    let Expr::Call(c) = e else {
        return Ok(None);
    };
    let Expr::Dot(inner, m) = &*c.name else {
        return Ok(None);
    };
    if m != "frame" {
        return Ok(None);
    }
    if !matches!(&**inner, Expr::Ident(b) if b == "render2d") {
        return Ok(None);
    }
    let Some(env) = &ctx.sprites else {
        return Err(unsupported(
            "render2d is only available in the sprites-2d profile",
            e,
        ));
    };
    if c.args.args.len() != 4 {
        return Err(unsupported(
            format!(
                "render2d.frame arity {} (want <token>, <image>, <ordinal>, <flip>)",
                c.args.args.len()
            ),
            e,
        ));
    }
    let image = match &c.args.args[1] {
        Arg::Pos(Expr::Str(n)) => n.to_string(),
        _ => {
            return Err(unsupported(
                "render2d.frame image must be a string literal image resource name",
                e,
            ))
        }
    };
    let table = env.frames.get(&image).ok_or_else(|| {
        unsupported(
            format!("unknown render2d.frame image 'resources.{image}'"),
            e,
        )
    })?;
    let mut tok = String::new();
    match &c.args.args[0] {
        Arg::Pos(x) => translate_expr(x, ctx, &mut tok)?,
        _ => return Err(unsupported("render2d.frame token arg", e)),
    }
    let mut ord = String::new();
    match &c.args.args[2] {
        Arg::Pos(x) => translate_expr(x, ctx, &mut ord)?,
        _ => return Err(unsupported("render2d.frame ordinal arg", e)),
    }
    let mut flip = String::new();
    match &c.args.args[3] {
        Arg::Pos(x) => translate_expr(x, ctx, &mut flip)?,
        _ => return Err(unsupported("render2d.frame flip arg", e)),
    }
    let img = &table.image;
    let mut line = String::new();
    for i in 0..table.len {
        if i == 0 {
            line.push_str(&format!(
                "if {ord} == {i} {{ ag_sprite_slot({tok}, {img}_fx[{i}], {img}_fy[{i}], {img}_fw[{i}], {img}_fh[{i}], {flip}) }}"
            ));
        } else {
            line.push_str(&format!(
                " else if {ord} == {i} {{ ag_sprite_slot({tok}, {img}_fx[{i}], {img}_fy[{i}], {img}_fw[{i}], {img}_fh[{i}], {flip}) }}"
            ));
        }
    }
    // 越界序数：退化矩形写，由受信单元自身边界具名拒绝（fault 42）。
    line.push_str(&format!(" else {{ ag_sprite_slot({tok}, 0, 0, 0, 0, 0) }}"));
    Ok(Some(vec![line]))
}
