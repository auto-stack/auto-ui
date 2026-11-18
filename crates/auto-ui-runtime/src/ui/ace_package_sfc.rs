//! PLAN-980 ④-A #3：**AGE `view` 树 → Vue `<template>`** 的完整 SFC 生成。
//!
//! 输入=包闭包（单元集合 + 入口）。bin 面：
//! * 模板：入口 unit 的组件面候选（`ComponentViewItem`）逐项渲染为 Vue 模板
//!   （`col/row→div`、`button/text/input` 保拼写；`style:`→`class`；
//!   `onclick/oninput`→`@click/@input` 指向生成的事件函数；文本叶=字面量或 `{{ expr }}`）。
//! * 脚本：`PackageFragmentLane` 发射 handler 的赋值 RHS 与被调 helper 函数体
//!   （同一份 976 片段文本；lane 局部名改写为 Vue ref），model 字段= `ref(初值)`。
//!
//! 边界（具名）：只覆盖标量 model（int/bool/str）、单层 on-block 赋值、字面量/
//! 字段/算术/比较/helper 调用表达式；f-string/backtick 文本叶与列表/存储/
//! 组件引用不在本刀（`source.age.sfc-*` 具名拒）。
use ace_core::source::packages::PackageModuleInput;

use crate::ui::ace_package_lane::PackageFragmentLane;

/// 生成一份可直接挂载的 Vue SFC（template + script setup）。
pub fn generate_package_sfc(units: &[PackageModuleInput], entry: &str) -> Result<String, String> {
    let entry_src = units
        .iter()
        .find(|u| format!("{}::{}", u.package, u.id) == entry)
        .map(|u| u.source.clone())
        .ok_or_else(|| format!("entry module `{entry}` is not in the package units"))?;
    let node = age_component_candidates(&entry_src)?
        .into_iter()
        .next()
        .ok_or_else(|| "no AGE node candidate in the entry module".to_string())?;
    // 受检描述：字段/初值/处理器赋值（同名一次编译）。
    let caps = [ace_core::source::packages::PACKAGES_CAPABILITY.to_string()];
    let compiled = ace_core::source::age_package::compile_age_package_with_scenario(
        units,
        entry,
        ace_av::age_component::LegacyWidgetAlias::Accept,
        &caps,
        &ace_av::age_component::ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let component = compiled
        .description
        .components
        .first()
        .ok_or_else(|| "package declares no component".to_string())?;

    // handler 消息（on-block 声明序）——lane 用它重放状态函数。
    let msgs: Vec<String> = component
        .handler_assigns
        .iter()
        .map(|(m, _)| m.clone())
        .collect();
    let msgs_ref: Vec<&str> = msgs.iter().map(|m| m.as_str()).collect();
    let lane = PackageFragmentLane::compile(units, entry, &msgs_ref)?;

    // ---- script：helper 定义 + refs + handlers ----
    let mut helper_fns: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut inits: Vec<(String, String)> = Vec::new();
    for (name, _ty) in &component.fields {
        let (js, helpers) = field_init_js(component, name, &lane)?;
        helper_fns.extend(helpers);
        inits.push((name.clone(), js));
    }
    let mut script = String::new();
    script.push_str("import { ref } from 'vue'\n");
    for (name, js) in &inits {
        script.push_str(&format!("const {name} = ref({js})\n"));
    }
    for (msg, assigns) in &component.handler_assigns {
        let fn_name = handler_fn_name(msg);
        script.push_str(&format!("function {fn_name}() {{\n"));
        for (field, rhs) in assigns {
            let state_fn = format!("__age_state_0_{field}");
            let span = rhs.span.start..rhs.span.end;
            let frag = lane.emit_expr_at_span(&state_fn, span).map_err(|e| {
                format!("source.age.sfc-expr: handler `{msg}` field `{field}`: {e}")
            })?;
            let defines_t0 = frag.text.iter().any(|l| l.contains("const t0"));
            for line in &frag.text {
                script.push_str("  ");
                script.push_str(&rewrite_lane_locals(line));
                script.push('\n');
            }
            helper_fns.extend(frag.helpers.iter().cloned());
            if defines_t0 {
                script.push_str(&format!("  {field}.value = t0;\n"));
            } else {
                // 常量/简单表达式：lane 不铸临时绑定——用同一 AST 打印（域内标量）。
                script.push_str(&format!("  {field}.value = {};\n", expr_to_js(rhs)?));
            }
        }
        script.push_str("}\n");
    }
    // helper 定义在 handler 之后统一发射（函数声明提升；遍历 helper_fns 已是全集）。
    for helper in &helper_fns {
        // a2x_fn_* = 包内函数（lane 发射定义）；其余 a2x_* 是 runtime helper，
        // 由 runtime_helpers 内联，不能当包函数发射。
        if !helper.starts_with("a2x_fn_") {
            continue;
        }
        emit_helper(&lane, helper, &mut script)?;
    }
    script.push_str(&runtime_helpers(&helper_fns)?);
    // 组件公开状态（defineExpose：宿主/测试经组件实例可读；不是测试专用钩子）。
    let state_entries: Vec<String> = component
        .fields
        .iter()
        .map(|(name, _)| format!("{name}: {name}.value"))
        .collect();
    script.push_str(&format!(
        "defineExpose({{ __state: () => ({{ {} }}) }})\n",
        state_entries.join(", ")
    ));

    // ---- template ----
    let mut template = String::new();
    let mut ctx = TmplCtx { out: &mut template };
    ctx.push_str("<div class=\"age-sfc\">");
    render_items(&node.view.tree, &mut ctx)?;
    ctx.push_str("</div>");

    Ok(format!(
        "<template>\n{template}\n</template>\n\n<script setup>\n{script}</script>\n"
    ))
}

struct TmplCtx<'a> {
    out: &'a mut String,
}

impl TmplCtx<'_> {
    fn push_str(&mut self, s: &str) {
        self.out.push_str(s);
    }
}

fn handler_fn_name(msg: &str) -> String {
    let mut out = String::from("on");
    for part in msg.split(|c: char| !c.is_alphanumeric()) {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    out
}

fn field_init_js(
    component: &ace_core::source::age_package::AgePackageComponent,
    field: &str,
    lane: &PackageFragmentLane,
) -> Result<(String, Vec<String>), String> {
    use ace_core::source::age_package::AgeSceneValue;
    if let Some((_, v)) = component.field_init_cpu.iter().find(|(n, _)| n == field) {
        if let AgeSceneValue::Int(i) = v {
            return Ok((i.to_string(), Vec::new()));
        }
        if let AgeSceneValue::Bool(b) = v {
            return Ok((b.to_string(), Vec::new()));
        }
    }
    let (_, expr) = component
        .field_inits
        .iter()
        .find(|(n, _)| n == field)
        .ok_or_else(|| format!("source.age.sfc-init: no initial for `{field}`"))?;
    // 受检初值表达式（含 helper 调用）经同一片段 lane 发射；局部名=ref 名。
    let state_fn = format!("__age_state_0_{field}");
    let span = expr.span.start..expr.span.end;
    match lane.emit_expr_at_span(&state_fn, span) {
        Ok(frag) => {
            let joined = frag.text.join(" ");
            if let Some(eq) = joined.find('=') {
                let rhs = joined[eq + 1..]
                    .trim()
                    .trim_end_matches(';')
                    .trim()
                    .to_string();
                Ok((rewrite_lane_locals(&rhs), frag.helpers))
            } else {
                Ok((rewrite_lane_locals(joined.trim()), frag.helpers))
            }
        }
        Err(_) => Ok((expr_to_js(expr)?, Vec::new())),
    }
}

/// 表达式 → JS（域内标量；调用/文本面不在本刀）。
fn expr_to_js(expr: &ace_core::source::ast::Expr) -> Result<String, String> {
    use ace_core::source::ast::{ExprKind, UnaryOp};
    match &expr.kind {
        ExprKind::IntLit(i) => Ok(i.to_string()),
        ExprKind::BoolLit(b) => Ok(b.to_string()),
        ExprKind::StrLit(s) => Ok(format!("{s:?}")),
        ExprKind::CharLit(_) => Err("source.age.sfc-expr: char literal outside this slice".into()),
        ExprKind::Var(id) => Ok(format!("{}.value", id.name)),
        ExprKind::Field { base, name } => {
            if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") {
                Ok(format!("{}.value", name.name))
            } else {
                Err("source.age.sfc-expr: nested field access outside this slice".into())
            }
        }
        ExprKind::Unary { op, operand } => {
            let inner = expr_to_js(operand)?;
            match op {
                UnaryOp::Neg => Ok(format!("(-{inner})")),
                UnaryOp::Not => Ok(format!("(!{inner})")),
                _ => Err("source.age.sfc-expr: unary op outside this slice".into()),
            }
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let l = expr_to_js(lhs)?;
            let r = expr_to_js(rhs)?;
            let js_op = match op {
                ace_core::source::ast::BinOp::Add => "+",
                ace_core::source::ast::BinOp::Sub => "-",
                ace_core::source::ast::BinOp::Mul => "*",
                ace_core::source::ast::BinOp::Div => "/",
                ace_core::source::ast::BinOp::Rem => "%",
                ace_core::source::ast::BinOp::Eq => "===",
                ace_core::source::ast::BinOp::Ne => "!==",
                ace_core::source::ast::BinOp::Lt => "<",
                ace_core::source::ast::BinOp::Le => "<=",
                ace_core::source::ast::BinOp::Gt => ">",
                ace_core::source::ast::BinOp::Ge => ">=",
                _ => return Err("source.age.sfc-expr: binary op outside this slice".into()),
            };
            Ok(format!("({l} {js_op} {r})"))
        }
        ExprKind::Call { callee, .. } => Err(format!(
            "source.age.sfc-expr: helper call `{}` in a model initial is outside this slice",
            callee.name
        )),
        _ => Err("source.age.sfc-expr: expression shape outside this slice".into()),
    }
}

/// 模板侧表达式：Vue 模板对 setup 里的 ref **自动解包**，故字段/变量不加 `.value`。
fn expr_to_template_js(expr: &ace_core::source::ast::Expr) -> Result<String, String> {
    use ace_core::source::ast::{ExprKind, UnaryOp};
    match &expr.kind {
        ExprKind::IntLit(i) => Ok(i.to_string()),
        ExprKind::BoolLit(b) => Ok(b.to_string()),
        ExprKind::StrLit(s) => Ok(format!("{s:?}")),
        ExprKind::Var(id) => Ok(id.name.clone()),
        ExprKind::Field { base, name } => {
            if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") {
                Ok(name.name.clone())
            } else {
                Err("source.age.sfc-expr: nested field access outside this slice".into())
            }
        }
        ExprKind::Unary { op, operand } => {
            let inner = expr_to_template_js(operand)?;
            match op {
                UnaryOp::Neg => Ok(format!("(-{inner})")),
                UnaryOp::Not => Ok(format!("(!{inner})")),
                _ => Err("source.age.sfc-expr: unary op outside this slice".into()),
            }
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let l = expr_to_template_js(lhs)?;
            let r = expr_to_template_js(rhs)?;
            let js_op = match op {
                ace_core::source::ast::BinOp::Add => "+",
                ace_core::source::ast::BinOp::Sub => "-",
                ace_core::source::ast::BinOp::Mul => "*",
                ace_core::source::ast::BinOp::Div => "/",
                ace_core::source::ast::BinOp::Rem => "%",
                ace_core::source::ast::BinOp::Eq => "===",
                ace_core::source::ast::BinOp::Ne => "!==",
                ace_core::source::ast::BinOp::Lt => "<",
                ace_core::source::ast::BinOp::Le => "<=",
                ace_core::source::ast::BinOp::Gt => ">",
                ace_core::source::ast::BinOp::Ge => ">=",
                _ => return Err("source.age.sfc-expr: binary op outside this slice".into()),
            };
            Ok(format!("({l} {js_op} {r})"))
        }
        _ => Err("source.age.sfc-expr: template expression outside this slice".into()),
    }
}

/// lane 发射文本的局部名改写：`l0___age_s_0_<field>` → `<field>.value`。
fn rewrite_lane_locals(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(pos) = rest.find("l0___age_s_0_") {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos + "l0___age_s_0_".len()..];
        let end = tail
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(tail.len());
        out.push_str(&format!("{}.value", &tail[..end]));
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

fn emit_helper(
    lane: &PackageFragmentLane,
    mapped: &str,
    script: &mut String,
) -> Result<(), String> {
    // mapped 形如 `a2x_fn_<mangled>`；lane 的 source_name 是 `<mangled>`。
    let suffix = mapped.strip_prefix("a2x_fn_").unwrap_or(mapped);
    let body = lane
        .emit_function_body(suffix)
        .map_err(|e| format!("source.age.sfc-helper: `{mapped}`: {e}"))?;
    let params: Vec<String> = body
        .locals
        .iter()
        .filter(|l| l.role == "param")
        .map(|l| l.emitted_name.clone())
        .collect();
    script.push_str(&format!("function {mapped}({}) {{\n", params.join(", ")));
    for line in &body.text {
        script.push_str("  ");
        script.push_str(line);
        script.push('\n');
    }
    script.push_str("}\n");
    Ok(())
}

/// 需要内联的 a2x_rt helper（int 语义；与 a2x-ts runtime 同式的最小集合）。
fn runtime_helpers(_helpers: &std::collections::BTreeSet<String>) -> Result<String, String> {
    // 本刀只内联 int 标量的最小 a2x_rt 集合（与 a2x-ts runtime/body.js 同式；
    // 函数声明提升，顺序不影响 ref 初值里的调用）。
    Ok("function a2x_add(a, b) {\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a + b)) throw new RangeError('a2x_add overflow');\n  return a + b;\n}\nfunction a2x_sub(a, b) {\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a - b)) throw new RangeError('a2x_sub overflow');\n  return a - b;\n}\nfunction a2x_mul(a, b) {\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a * b)) throw new RangeError('a2x_mul overflow');\n  return a * b;\n}\n".to_string())
}

/// 模板渲染：逐 view 项。
fn render_items(
    items: &[ace_core::source::age_component::ComponentViewItem],
    ctx: &mut TmplCtx<'_>,
) -> Result<(), String> {
    use ace_core::source::age_component::{ComponentEntryValue, ComponentViewItem};
    for it in items {
        match it {
            ComponentViewItem::Text(leaf) => {
                if let ace_core::source::ast::ExprKind::StrLit(s) = &leaf.expr.kind {
                    ctx.push_str(&escape_html(s));
                } else {
                    let js = expr_to_template_js(&leaf.expr)?;
                    ctx.push_str(&format!("{{{{ {js} }}}}"));
                }
            }
            ComponentViewItem::Element(el) => {
                let tag = match el.tag.as_str() {
                    "col" => "div",
                    "row" => "div",
                    other => other,
                };
                let mut classes: Vec<String> = Vec::new();
                if el.tag == "col" {
                    classes.push("flex flex-col".into());
                }
                if el.tag == "row" {
                    classes.push("flex flex-row".into());
                }
                let mut events: Vec<(String, String)> = Vec::new();
                for sub in &el.items {
                    if let ComponentViewItem::Entry(en) = sub {
                        match en.key.as_str() {
                            "style" => {
                                if let ComponentEntryValue::Expr(e) = &en.value {
                                    if let ace_core::source::ast::ExprKind::StrLit(s) = &e.kind {
                                        classes.push(s.clone());
                                    }
                                }
                            }
                            k if k.starts_with("on") => {
                                if let ComponentEntryValue::Expr(e) = &en.value {
                                    let msg = match &e.kind {
                                        ace_core::source::ast::ExprKind::Var(id) => {
                                            Some(id.name.clone())
                                        }
                                        ace_core::source::ast::ExprKind::Field { base, name } => {
                                            if matches!(&base.kind, ace_core::source::ast::ExprKind::Var(v) if v.name == "self")
                                            {
                                                Some(name.name.clone())
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    };
                                    if let Some(msg) = msg {
                                        events.push((
                                            k.trim_start_matches("on").to_string(),
                                            handler_fn_name(&msg),
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                ctx.push_str(&format!("<{tag}"));
                if !classes.is_empty() {
                    ctx.push_str(&format!(" class=\"{}\"", escape_attr(&classes.join(" "))));
                }
                for (ev, handler) in &events {
                    ctx.push_str(&format!(" @{ev}=\"{handler}\""));
                }
                ctx.push_str(">");
                if let Some(label) = &el.label {
                    if let ace_core::source::ast::ExprKind::StrLit(s) = &label.kind {
                        ctx.push_str(&escape_html(s));
                    }
                }
                render_items(&el.items, ctx)?;
                ctx.push_str(&format!("</{tag}>"));
            }
            ComponentViewItem::Entry(_) => {}
        }
    }
    Ok(())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_html(s).replace('"', "&quot;")
}

/// 组件面候选（view 树 + 文本叶表达式）——与包前门同一次解析的公开投影。
fn age_component_candidates(
    src: &str,
) -> Result<Vec<ace_core::source::age_component::ComponentNode>, String> {
    ace_av::age_component::project_component_candidates(src).map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })
}
