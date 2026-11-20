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
use super::ace_route::{package_candidates, prepare_page, RouteSource, RouteTable};

use crate::ui::ace_package_lane::PackageFragmentLane;

/// 生成一份可直接挂载的 Vue SFC（template + script setup）。
pub fn generate_package_sfc(units: &[PackageModuleInput], entry: &str) -> Result<String, String> {
    if let Some(product) = generate_routed_package_sfcs(units, entry)? {
        return Ok(product.app);
    }
    generate_package_sfc_leaf(units, entry, None, false)
}

/// Publish the app and page SFCs together. The project supplies the Vue Router
/// instance with the same module aliases as the declared routes.
pub struct RoutedPackageSfc {
    pub app: String,
    pub app_name: String,
    pub pages: std::collections::BTreeMap<String, String>,
    pub paths: Vec<(String, String)>,
}

/// Resolve a routed app from the standard front/app.at and front/pages layout.
pub fn generate_routed_front_sfc(
    app_at: &std::path::Path,
    source: &str,
    modules: &[std::path::PathBuf],
) -> Result<Option<RoutedPackageSfc>, String> {
    let unit = |id: String, source: String| PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id,
        source,
        provider: None,
    };
    let mut units = vec![unit("app".into(), source.into())];
    for module in modules {
        if let Some(id) = module.file_stem().and_then(|name| name.to_str()) {
            let source = std::fs::read_to_string(module)
                .map_err(|error| format!("ace.route.module: {}: {error}", module.display()))?;
            units.push(unit(id.into(), source));
        }
    }
    generate_routed_package_sfcs(&units, "app::app")
        .map_err(|error| format!("{}: {error}", app_at.display()))
}

pub fn generate_routed_package_sfcs(
    units: &[PackageModuleInput],
    entry: &str,
) -> Result<Option<RoutedPackageSfc>, String> {
    let shell = units.iter().find(|unit| format!("{}::{}", unit.package, unit.id) == entry)
        .ok_or_else(|| format!("entry module `{entry}` is not in the package units"))?;
    let candidates = package_candidates(&shell.source)?;
    let root = candidates.first().ok_or_else(|| "no AGE node candidate in the entry module".to_string())?;
    let Some(table) = RouteTable::from_root(root, units, shell)? else {
        return Ok(None);
    };

    let app = generate_route_file(&table, units, entry, true)?;
    let mut pages = std::collections::BTreeMap::new();
    let mut paths = Vec::new();
    for route in table.routes() {
        // A failed page component is per declaration so its diagnostic keeps
        // the originating pattern when multiple routes use the same alias.
        let result = route.page.as_ref().map_err(Clone::clone).and_then(|page| {
            generate_route_file(&table, units, page, false).map_err(|error| {
                format!("ace.route.page: pattern {:?} module {:?}: {error}", route.pattern, route.module)
            })
        });
        let name = match result {
            Ok(sfc) => {
                pages.insert(route.module.clone(), sfc);
                route.module.clone()
            }
            Err(diagnostic) => {
                let name = format!("ace-route-error-{}", paths.len());
                pages.insert(name.clone(), format!("<template><pre>{}</pre></template>\n", escape_html(&diagnostic)));
                name
            }
        };
        paths.push((route.pattern.clone(), name));
    }
    Ok(Some(RoutedPackageSfc { app, app_name: root.name.name.clone(), pages, paths }))
}

fn generate_route_file(
    table: &RouteTable,
    units: &[PackageModuleInput],
    entry: &str,
    shell: bool,
) -> Result<String, String> {
    let mut scoped = table.entry_units(units, entry);
    let unit = scoped.first_mut().ok_or_else(|| format!("route entry `{entry}` is missing"))?;
    let context = prepare_page(&unit.source, None)?;
    unit.source = context.source.clone();
    generate_package_sfc_leaf(&scoped, entry, Some(context), shell)
}

fn generate_package_sfc_leaf(
    units: &[PackageModuleInput],
    entry: &str,
    route: Option<RouteSource>,
    shell: bool,
) -> Result<String, String> {
    let entry_src = units
        .iter()
        .find(|u| format!("{}::{}", u.package, u.id) == entry)
        .map(|u| u.source.clone())
        .ok_or_else(|| format!("entry module `{entry}` is not in the package units"))?;
    let node = package_candidates(&entry_src)?
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

    // F-R21-2（PLAN-981 r21 复审指派，owner 980/UI）：`handler_stmts` 契约
    // 的消费门——本车道只复放平铺赋值（`handler_assigns`）；体含 if/let
    //（age-package 作者面文法可达）时**具名拒**，不双支裸喷/部分复放
    //（lang 侧 `age_component.rs` `handler_stmts` 契约：部分复放=错状态
    // 值，禁止）。
    fn control_flow_kind(
        stmts: &[ace_core::source::age_component::ComponentStmt],
    ) -> Option<&'static str> {
        for st in stmts {
            match st {
                ace_core::source::age_component::ComponentStmt::Assign(_) => {}
                ace_core::source::age_component::ComponentStmt::Let { .. } => {
                    return Some("let");
                }
                // If 本身即控制流（then/els 内嵌套同拒）。
                ace_core::source::age_component::ComponentStmt::If { .. } => {
                    return Some("if");
                }
            }
        }
        None
    }
    for (msg, stmts) in &component.handler_stmts {
        if let Some(kind) = control_flow_kind(stmts) {
            return Err(format!(
                "source.age.sfc-handler-stmt: handler `{msg}` contains a `{kind}` statement — \
                 this lane replays flat assigns only; partial replay is forbidden"
            ));
        }
    }

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
    if let Some(route) = &route {
        script.push_str("import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'\n");
        script.push_str("const aceRoute = useRoute()\nconst aceRouter = useRouter()\n");
        if !route.commands.is_empty() {
            script.push_str("import { onBeforeUnmount } from 'vue'\nlet aceRouteActive = true\nonBeforeUnmount(() => { aceRouteActive = false })\n");
        }
        if route.has_init {
            script.push_str("import { onMounted } from 'vue'\n");
        }
        if !route.web_params.is_empty() {
            script.push_str("function aceParam(key, kind) {\n  const value = aceRoute.params[key];\n  if (typeof value !== 'string') throw new Error('ace.route.param: missing ' + key);\n  if (kind === 'str') return value;\n  const number = Number(value);\n  if (!/^[+-]?\\d+$/.test(value) || !Number.isInteger(number) || number < -2147483648 || number > 2147483647) throw new Error('ace.route.param: invalid int for ' + key);\n  return number;\n}\n");
        }
    }
    for (name, js) in &inits {
        if let Some((key, kind)) = route.as_ref().and_then(|route| route.web_params.get(name)) {
            script.push_str(&format!("const {name} = ref(aceParam({key:?}, {kind:?}))\n"));
        } else {
            script.push_str(&format!("const {name} = ref({js})\n"));
        }
    }
    for (msg, assigns) in &component.handler_assigns {
        if route.as_ref().is_some_and(|route| route.commands.contains_key(msg)) {
            continue;
        }
        let fn_name = handler_fn_name(msg);
        script.push_str(&format!("function {fn_name}() {{\n"));
        for (field, rhs) in assigns {
            let state_fn = format!("__age_state_0_{field}");
            let span = rhs.span.start..rhs.span.end;
            let frag = lane.emit_expr_at_span(&state_fn, span).map_err(|e| {
                format!("source.age.sfc-expr: handler `{msg}` field `{field}`: {e}")
            })?;
            if frag.text.is_empty() {
                // Keep the existing scalar representation for inline constants and fields.
                script.push_str(&format!("  {field}.value = {};\n", expr_to_js(rhs)?));
            } else {
                // A fragment may emit several temporaries. Use the value returned by
                // the lane, and give its temporaries their own assignment scope.
                script.push_str("  {\n");
                for line in &frag.text {
                    script.push_str("    ");
                    script.push_str(&rewrite_lane_locals(line));
                    script.push('\n');
                }
                helper_fns.extend(frag.helpers.iter().cloned());
                script.push_str(&format!("    {field}.value = {};\n  }}\n", rewrite_lane_locals(&frag.expr)));
            }
        }
        script.push_str("}\n");
    }
    if let Some(route) = &route {
        for (message, action) in &route.commands {
            let handler = handler_fn_name(message);
            let operation = action.vue();
            script.push_str(&format!("function {handler}() {{ if (aceRouteActive) {operation}; }}\n"));
        }
        if route.has_init {
            script.push_str("onMounted(onInit)\n");
        }
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
        .filter(|(name, _)| !route.as_ref().is_some_and(|route| route.web_params.contains_key(name)))
        .map(|(name, _)| format!("{name}: {name}.value"))
        .collect();
    script.push_str(&format!(
        "defineExpose({{ __state: () => ({{ {} }}) }})\n",
        state_entries.join(", ")
    ));

    // ---- template ----
    let mut template = String::new();
    let mut ctx = TmplCtx { out: &mut template, routed: route.is_some(), shell };
    ctx.push_str("<div class=\"age-sfc\">");
    render_items(&node.view.tree, &mut ctx)?;
    ctx.push_str("</div>");

    Ok(format!(
        "<template>\n{template}\n</template>\n\n<script setup>\n{script}</script>\n"
    ))
}

struct TmplCtx<'a> {
    out: &'a mut String,
    routed: bool,
    shell: bool,
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

/// lane 发射文本的局部名改写：`l<i>___age_s_<inst>_<field>` → `<field>.value`。
/// 边界修正（静态核对暴露的预存缺陷）：原实现硬编码 `l0___age_s_0_` 前缀
/// ——**第二个及以后的状态字段（l1/l2/…）漏改**（#3 单字段计数器从未暴露；
/// 双字段源实测 `a2x_add(l1___age_s_0_n, 1)` 裸名进产物=运行时
/// ReferenceError）。泛化为任意局部索引与实例段（字段名=剥局部段与实例段
/// 后的尾段——`___age_s_` 后是 `<inst>_<field>`）。
fn rewrite_lane_locals(line: &str) -> String {
    const MARK: &str = "___age_s_";
    let mut out = String::new();
    let bytes = line.as_bytes();
    let mut rest_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        // 局部名锚：`l<digits>` 后随 `___age_s_`。
        if bytes[i] == b'l' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && line[j..].starts_with(MARK) {
                let field_start = j + MARK.len();
                // 字段名=`___age_s_` 后剥掉实例数字段（`<inst>_`）。
                let mut k = field_start;
                while k < bytes.len() && bytes[k].is_ascii_digit() {
                    k += 1;
                }
                if k > field_start && k < bytes.len() && bytes[k] == b'_' {
                    let field = &line[k + 1..];
                    let end = field
                        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .unwrap_or(field.len());
                    if end > 0 {
                        out.push_str(&line[rest_start..i]);
                        out.push_str(&format!("{}.value", &field[..end]));
                        rest_start = k + 1 + end;
                        i = rest_start;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out.push_str(&line[rest_start..]);
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

fn render_numeric_attributes(
    ctx: &mut TmplCtx<'_>,
    items: &[ace_core::source::age_component::ComponentViewItem],
    defaults: &[(&str, &str)],
) -> Result<(), String> {
    use ace_core::source::age_component::{ComponentEntryValue, ComponentViewItem};
    use ace_core::source::ast::ExprKind;

    for &(key, default) in defaults {
        let entry = items.iter().rev().find_map(|item| match item {
            ComponentViewItem::Entry(entry) if entry.key == key => Some(&entry.value),
            _ => None,
        });
        let (prefix, value) = match entry {
            Some(ComponentEntryValue::Expr(expr)) => match &expr.kind {
                ExprKind::StrLit(value) => ("", value.clone()),
                ExprKind::IntLit(value) => ("", value.to_string()),
                _ => (":", expr_to_template_js(expr)?),
            },
            None => ("", default.to_owned()),
            _ => return Err(format!("source.age.sfc-number: unsupported `{key}` binding")),
        };
        ctx.push_str(&format!(" {prefix}{key}=\"{}\"", escape_attr(&value)));
    }
    Ok(())
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
            // PLAN-1072 刀B：循环项（`for <i>, <v> in <view-data>`）——
            // **组件面形**（视图数据字段经旧管线方言 AST 承载 v-for；包面
            // 脚本无视图数据发射面）——包面模板腿具名拒，不静默丢项。
            ComponentViewItem::For(f) => {
                return Err(format!(
                    "source.age.sfc-view-for: view 循环项 @{}..{} 本面未承载（循环+视图数据是组件面形；包面保持标量模板边界）",
                    f.span.start, f.span.end
                ));
            }
            // PLAN-980 view-if 刀：条件项 → `<template v-if/v-else>`（WEB
            // 模板腿的真语义；else-if 链=嵌套 Cond→嵌套 template v-if）。
            ComponentViewItem::Cond(c) => {
                // 边界（静态核对补）：①条件经 escape_attr（含双引号的比较
                // 表达式不得裸插双引号属性）；②有 else 时保留前置 v-if 锚
                // （空 then 发空 v-if 模板作锚，避免孤立 v-else）；双空臂
                // （无 else 且 then 空）整体跳过。
                let cond_js = escape_attr(&expr_to_template_js(&c.cond)?);
                if !c.then_items.is_empty() || !c.else_items.is_empty() {
                    ctx.push_str(&format!("<template v-if=\"{cond_js}\">"));
                    render_items(&c.then_items, ctx)?;
                    ctx.push_str("</template>");
                }
                if !c.else_items.is_empty() {
                    ctx.push_str("<template v-else>");
                    render_items(&c.else_items, ctx)?;
                    ctx.push_str("</template>");
                }
            }
            ComponentViewItem::Element(el) => {
                if ctx.shell && el.tag == "outlet" {
                    ctx.push_str(r#"<RouterView v-slot="{ Component, route }"><component v-if="Component" :is="Component" :key="route.path" /><pre v-else>ace.route.no-match: no route for {{ route.path }}</pre></RouterView>"#);
                    continue;
                }
                if ctx.routed && matches!(el.tag.as_str(), "link" | "sidebar_menu_button") {
                    let target = el.items.iter().find_map(|item| match item {
                        ComponentViewItem::Entry(entry) if entry.key == "to" => match &entry.value {
                            ComponentEntryValue::Expr(expr) => Some(expr),
                            _ => None,
                        },
                        _ => None,
                    });
                    if let Some(target) = target {
                        ctx.push_str(&format!("<RouterLink :to=\"{}\">", escape_attr(&expr_to_template_js(target)?)));
                        if let Some(label) = &el.label {
                            ctx.push_str(&format!("{{{{ {} }}}}", expr_to_template_js(label)?));
                        }
                        render_items(&el.items, ctx)?;
                        ctx.push_str("</RouterLink>");
                        continue;
                    }
                }
                let tag = match el.tag.as_str() {
                    "col" => "div",
                    "row" => "div",
                    "slider" => "input",
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
                match el.tag.as_str() {
                    "progress" => {
                        render_numeric_attributes(ctx, &el.items, &[("value", "0"), ("max", "100")])?;
                    }
                    "slider" => {
                        ctx.push_str(" type=\"range\"");
                        render_numeric_attributes(
                            ctx,
                            &el.items,
                            &[("min", "0"), ("max", "100"), ("step", "1"), ("value", "0")],
                        )?;
                        ctx.push_str(" />");
                        continue;
                    }
                    _ => {}
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

#[cfg(test)]
mod p980_cond_render_tests {
    //! PLAN-980 view-if 边界（静态核对两缺陷）的单元级判据：转义（含双引号
    //! 比较条件→&quot;）与空 then 的 v-if 锚（非孤立 v-else）。集成档（真
    //! Vue 编译+分支切换）见 tests/plan980_package_sfc.rs——包面 checker 的
    //! 条件值域（str 字面量比较/props 读）是另一具名边界，此处手造 Expr
    //! 直测渲染机械。
    use super::*;
    use ace_core::atom_text::Span;
    use ace_core::source::age_component::{
        ComponentViewCond, ComponentViewElement, ComponentViewItem,
    };
    use ace_core::source::ast::{BinOp, Expr, ExprKind, Ident};

    fn self_field(name: &str) -> Expr {
        let sp = Span { start: 0, end: 0 };
        Expr {
            span: sp,
            kind: ExprKind::Field {
                base: Box::new(Expr {
                    span: sp,
                    kind: ExprKind::Var(Ident {
                        name: "self".to_string(),
                        span: sp,
                    }),
                }),
                name: Ident {
                    name: name.to_string(),
                    span: sp,
                },
            },
        }
    }

    fn str_lit(v: &str) -> Expr {
        Expr {
            span: self_field("x").span,
            kind: ExprKind::StrLit(v.to_string()),
        }
    }

    fn cond_item(
        cond: Expr,
        then_items: Vec<ComponentViewItem>,
        else_items: Vec<ComponentViewItem>,
    ) -> ComponentViewItem {
        ComponentViewItem::Cond(ComponentViewCond {
            cond,
            then_items,
            else_items,
            span: self_field("x").span,
        })
    }

    fn render(items: &[ComponentViewItem]) -> String {
        let mut out = String::new();
        let mut ctx = TmplCtx { out: &mut out, routed: false, shell: false };
        render_items(items, &mut ctx).expect("render ok");
        out
    }

    /// 含双引号的比较条件 → 属性转义（非裸插）。
    #[test]
    fn p980_cond_attr_escapes_double_quotes() {
        let sp = self_field("x").span;
        let cmp = Expr {
            span: sp,
            kind: ExprKind::Binary {
                op: BinOp::Eq,
                lhs: Box::new(self_field("mode")),
                rhs: Box::new(str_lit("dark")),
            },
        };
        let out = render(&[cond_item(
            cmp,
            vec![ComponentViewItem::Text(leaf("d"))],
            vec![],
        )]);
        assert!(
            out.contains("v-if=\"(mode === &quot;dark&quot;)\">"),
            "quoted cond is attr-escaped: {out}"
        );
        assert!(
            !out.contains("\"dark\""),
            "no raw double quotes inside the attr: {out}"
        );
    }

    /// 空 then + else → 空 v-if 锚紧邻 v-else（非孤立 v-else）。
    #[test]
    fn p980_cond_empty_then_keeps_vif_anchor() {
        let out = render(&[cond_item(
            self_field("flag"),
            vec![],
            vec![ComponentViewItem::Text(leaf("f"))],
        )]);
        assert_eq!(
            out,
            "<template v-if=\"flag\"></template><template v-else>f</template>"
        );
    }

    /// 双空臂（无 else 且 then 空）整体跳过。
    #[test]
    fn p980_cond_both_empty_skips() {
        let out = render(&[cond_item(self_field("flag"), vec![], vec![])]);
        assert_eq!(out, "", "no output for a fully empty conditional");
    }

    /// 局部名改写泛化（静态核对暴露的预存缺陷）：`l1/l2…___age_s_<i>_` 与
    /// `l0` 同改（原硬编码 l0——第二字段起漏改=运行时 ReferenceError）。
    #[test]
    fn p980_rewrite_lane_locals_maps_every_local_index() {
        assert_eq!(
            rewrite_lane_locals("const t0 = a2x_add(l1___age_s_0_n, 1);"),
            "const t0 = a2x_add(n.value, 1);"
        );
        assert_eq!(
            rewrite_lane_locals("l0___age_s_0_count = l2___age_s_1_other;"),
            "count.value = other.value;"
        );
        assert_eq!(rewrite_lane_locals("plain_ident"), "plain_ident");
    }

    fn leaf(label: &str) -> ace_core::source::age_h0::AgeTextLeaf {
        let sp = self_field("x").span;
        ace_core::source::age_h0::AgeTextLeaf {
            role: "view".to_string(),
            spelling: "text".to_string(),
            component_span: sp,
            expr: str_lit(label),
            span: sp,
        }
    }
}
