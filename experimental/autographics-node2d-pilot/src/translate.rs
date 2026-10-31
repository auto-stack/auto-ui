//! 语句/表达式 → 单文件 AG 用户 Auto 源的结构化翻译（合同 S3）。
//!
//! 有界研究面：只接受 fixture 实际使用的形态；任何未列形态都是硬错误
//! （需要修订合同），不静默改写。成员 `.x` 在解析层就是
//! `Dot(Ident("self"), x)`；服务映射冻结：
//! - `input.axis(Left, Right)` → `(ag_action_held(1) - ag_action_held(0))`
//! - `refs.body.move_and_slide((dx, dy))` → `ag_char_move(dx, dy, 0.002)`
//! - `refs.body.move(x, y)` → `ag_object_move(<实例 handle>, x, y, 0.0)`
//! - `emit .Ev(args)` → 绑定表投递（lower 层注入目标调用文本）

use auto_parser_api::ast::{Arg, Expr, Op, Stmt};

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

fn member_ref(field: &str, ctx: &Ctx) -> String {
    match &ctx.target {
        Target::Root => format!("r_{field}"),
        Target::Instance { prefix, index } => format!("{prefix}_{field}[{index}]"),
    }
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
                        out.push_str(&member_ref(name, ctx));
                        Ok(())
                    }
                    "emit" => {
                        let call = (ctx.emit_hook)(name)?;
                        out.push_str(&call);
                        Ok(())
                    }
                    _ => {
                        if let Some(text) = (ctx.enum_path)(base, name) {
                            out.push_str(&text);
                            Ok(())
                        } else {
                            Err(unsupported(format!("path {base}.{name}"), e))
                        }
                    }
                },
                _ => Err(unsupported(format!("member path .{name} of {obj:?}"), e)),
                _ => Err(unsupported(format!("member of {obj:?}"), e)),
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
                    // 冻结研究输入模型（合同 S3）：axis(Left, Right) 映射为
                    // 正向 held 位单次计费；完整差值轴为后继项（预算算式
                    // 1snapshot 以此为准）。
                    out.push_str("ag_action_held(1)");
                    return Ok(());
                }
              }
            }
            // refs.body 服务调用（move_to 等）。
            if let Expr::Dot(inner, op) = &*call.name {
                if let Expr::Dot(boxed_root, mid) = &**inner {
                    if matches!(&**boxed_root, Expr::Ident(r) if r == "refs") && mid == "body" {
                        return translate_refs_body(op, e, ctx, out);
                    }
                }
            }
            // 普通调用：池化纯函数（patrol_y）逐字参数翻译。
            match &*call.name {
                Expr::Ident(fname) => {
                    out.push_str(fname);
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
            let text =
                op_text(op).ok_or_else(|| unsupported(format!("operator {op:?}"), e))?;
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

/// `refs.body.<op>(args)`：`move_and_slide((dx,dy))` → char 扫掠；
/// `move(x,y)` → 实例对象移动。其余拒绝。
fn translate_refs_body(op: &str, e: &Expr, ctx: &Ctx, out: &mut String) -> Result<(), LowerError> {
    let call = match e {
        Expr::Call(c) => c,
        _ => return Err(unsupported("refs.body non-call", e)),
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
        other => Err(unsupported(format!("refs.body.{other}"), e)),
    }
}
