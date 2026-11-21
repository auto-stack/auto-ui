//! PLAN-980 ④-A #3：**AGE `view` 树 → Vue `<template>`** 的完整 SFC 生成。
//!
//! 输入=包闭包（单元集合 + 入口）。bin 面：
//! * 模板：入口 unit 的组件面候选（`ComponentViewItem`）逐项渲染为 Vue 模板
//!   （`col/row→div`、`button/text/input` 保拼写；`style:`→`class`；
//!   `onclick/oninput`→`@click/@input` 指向生成的事件函数；文本叶=字面量或 `{{ expr }}`）。
//! * 脚本：`PackageFragmentLane` 发射 handler 的赋值 RHS 与被调 helper 函数体
//!   （同一份 976 片段文本；lane 局部名改写为 Vue ref），model 字段= `ref(初值)`。
//!
//! 平铺空载荷处理器沿赋值车道复放。结构化处理器使用完整受检语句。
//! 鼠标绑定在浏览器换算为 f32 后按声明顺序传参。
use ace_core::source::age_package::AgePackageComponent;
use ace_core::source::age_component::{ComponentClassPart, ComponentViewElement};
use ace_core::source::packages::PackageModuleInput;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use super::ace_route::{front_package_units, package_candidates, prepare_page, RouteSource, RouteTable};
use super::ace_svg;
use super::ace_widget_packages::append_widget_packages;

use crate::ui::ace_package_lane::PackageFragmentLane;
use crate::ui::ace_widget_props::conditional_classes;

mod checked_widgets;
mod widgets;

/// 生成一份可直接挂载的 Vue SFC（template + script setup）。
pub fn generate_package_sfc(units: &[PackageModuleInput], entry: &str) -> Result<String, String> {
    // PLAN-124/125：routes 声明优先走 routed lane；非路由包回落 PLAN-105
    // 的 prep/desugar 管线（leaf 兜底）。
    if let Some(product) = generate_routed_package_sfcs(units, entry)? {
        return Ok(product.app);
    }
    Ok(generate_package_sfc_leaf(units, entry, None, false)?.source)
}

/// Publish the app and page SFCs together. The project supplies the Vue Router
/// instance with the same module aliases as the declared routes.
pub struct RoutedPackageSfc {
    pub app: String,
    pub app_name: String,
    pub pages: std::collections::BTreeMap<String, String>,
    pub paths: Vec<(String, String)>,
    pub runtime_mjs: Option<String>,
    /// Canonical files whose widgets were all emitted as part of a checked route.
    pub embedded_widget_sources: BTreeSet<PathBuf>,
}

struct PackageSfc {
    source: String,
    uses_runtime: bool,
    embedded_widget_modules: BTreeSet<String>,
}

/// Resolve a routed app from the standard front/app.at and front/pages layout.
pub fn generate_routed_front_sfc(
    app_at: &std::path::Path,
    source: &str,
    modules: &[std::path::PathBuf],
) -> Result<Option<RoutedPackageSfc>, String> {
    let (units, paths) = front_package_units(app_at, source, modules)?;
    generate_routed_package_sfcs_with_paths(&units, "app::app", &paths)
        .map_err(|error| format!("{}: {error}", app_at.display()))
}

/// Generate an app with an authored component package and no route outlet.
pub fn generate_linked_front_sfc(
    app_at: &Path,
    source: &str,
    modules: &[PathBuf],
) -> Result<Option<RoutedPackageSfc>, String> {
    let candidates = package_candidates(source)?;
    let [root] = candidates.as_slice() else { return Ok(None) };
    if root.package_imports.is_empty() {
        return Ok(None);
    }
    let (mut units, _) = front_package_units(app_at, source, modules)?;
    let widget_paths = append_widget_packages(app_at, &candidates, &mut units)?;
    if widget_paths.is_empty() {
        return Ok(None);
    }
    let sfc = generate_package_sfc_leaf(&units, "app::app", None, false)
        .map_err(|error| format!("{}: {error}", app_at.display()))?;
    let embedded_widget_sources = widget_paths.into_iter()
        .filter_map(|(module, path)| sfc.embedded_widget_modules.contains(&module).then_some(path))
        .collect();
    Ok(Some(RoutedPackageSfc {
        app: sfc.source,
        app_name: root.name.name.clone(),
        pages: BTreeMap::new(),
        paths: Vec::new(),
        runtime_mjs: sfc.uses_runtime.then(|| {
            ace_a2x_ts::script_runtime_body(ace_a2x_ts::ScriptLang::JavaScript, &[])
        }),
        embedded_widget_sources,
    }))
}

pub fn generate_routed_package_sfcs(
    units: &[PackageModuleInput],
    entry: &str,
) -> Result<Option<RoutedPackageSfc>, String> {
    generate_routed_package_sfcs_with_paths(units, entry, &BTreeMap::new())
}

pub const ROUTE_DIAGNOSTIC_SFC_PREFIX: &str = "<template><pre v-pre data-ace-route-error>";

fn generate_routed_package_sfcs_with_paths(
    units: &[PackageModuleInput],
    entry: &str,
    paths_by_module: &BTreeMap<String, PathBuf>,
) -> Result<Option<RoutedPackageSfc>, String> {
    let shell = units.iter().find(|unit| format!("{}::{}", unit.package, unit.id) == entry)
        .ok_or_else(|| format!("entry module `{entry}` is not in the package units"))?;
    let candidates = package_candidates(&shell.source)?;
    let root = candidates.first().ok_or_else(|| "no AGE node candidate in the entry module".to_string())?;
    let Some(table) = RouteTable::from_root(root, units, shell, true)? else {
        return Ok(None);
    };

    let (app, mut embedded_widget_sources) = generate_route_file(
        &table, units, entry, true, paths_by_module.get(entry).map(PathBuf::as_path),
    )?;
    let mut uses_runtime = app.uses_runtime;
    let mut pages = std::collections::BTreeMap::new();
    let mut paths = Vec::new();
    for route in table.routes() {
        // A failed page component is per declaration so its diagnostic keeps
        // the originating pattern when multiple routes use the same alias.
        let result = route.page.as_ref().map_err(Clone::clone).and_then(|page| {
            generate_route_file(&table, units, page, false, paths_by_module.get(page).map(PathBuf::as_path)).map_err(|error| {
                format!("ace.route.page: pattern {:?} module {:?}: {error}", route.pattern, route.module)
            })
        });
        let name = match result {
            Ok((sfc, sources)) => {
                uses_runtime |= sfc.uses_runtime;
                embedded_widget_sources.extend(sources);
                pages.insert(route.module.clone(), sfc.source);
                route.module.clone()
            }
            Err(diagnostic) => {
                let name = format!("ace-route-error-{}", paths.len());
                pages.insert(name.clone(), format!("{ROUTE_DIAGNOSTIC_SFC_PREFIX}{}</pre></template>\n", escape_html(&diagnostic)));
                name
            }
        };
        paths.push((route.pattern.clone(), name));
    }
    Ok(Some(RoutedPackageSfc {
        app: app.source,
        app_name: root.name.name.clone(),
        pages,
        paths,
        runtime_mjs: uses_runtime.then(|| {
            ace_a2x_ts::script_runtime_body(ace_a2x_ts::ScriptLang::JavaScript, &[])
        }),
        embedded_widget_sources,
    }))
}

fn generate_route_file(
    table: &RouteTable,
    units: &[PackageModuleInput],
    entry: &str,
    shell: bool,
    source_path: Option<&Path>,
) -> Result<(PackageSfc, BTreeSet<PathBuf>), String> {
    let mut scoped = table.entry_units(units, entry);
    let unit = scoped.first_mut().ok_or_else(|| format!("route entry `{entry}` is missing"))?;
    let context = prepare_page(&unit.source, None)?;
    unit.source = context.source.clone();
    let widget_paths = if let Some(path) = source_path {
        let candidates = package_candidates(&context.source)?;
        append_widget_packages(path, &candidates, &mut scoped)?
    } else {
        BTreeMap::new()
    };
    let sfc = generate_package_sfc_leaf(&scoped, entry, Some(context), shell)?;
    let sources = widget_paths.into_iter()
        .filter_map(|(module, path)| sfc.embedded_widget_modules.contains(&module).then_some(path))
        .collect();
    Ok((sfc, sources))
}

fn generate_package_sfc_leaf(
    units: &[PackageModuleInput],
    entry: &str,
    route: Option<RouteSource>,
    shell: bool,
) -> Result<PackageSfc, String> {
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
    let node = compiled.entry_node()
        .ok_or_else(|| "no AGE node candidate in the entry module".to_string())?;
    if compiled.first_widget_ref().is_some() {
        if route.as_ref().is_some_and(|page| {
            !page.commands.is_empty() || !page.web_params.is_empty()
        }) {
            return Err("source.age.sfc-widget-route: checked widget views do not support route commands or parameters".into());
        }
        return checked_widgets::generate(units, entry, &caps, route.is_some(), shell);
    }
    if node.members.iter().any(|member| member.role == "computed") {
        use ace_core::{
            hir::ExprKind,
            source::{age_component::ComponentViewItem, age_package::AGE_PACKAGE_HIDDEN_PREFIX},
        };
        let offset = compiled.description.modules.iter()
            .find(|module| module.key == entry)
            .ok_or_else(|| format!("compiled package has no entry module `{entry}`"))?
            .flat_start;
        let view_spans: Vec<_> = node.view.tree.iter().map(|item| {
            let span = match item {
                ComponentViewItem::Text(item) => item.span,
                ComponentViewItem::Element(item) => item.span,
                ComponentViewItem::Entry(item) => item.span,
                ComponentViewItem::Cond(item) => item.span,
                ComponentViewItem::For(item) => item.span,
            };
            offset + span.start..offset + span.end
        }).collect();
        let prefix = format!("{AGE_PACKAGE_HIDDEN_PREFIX}computed_");
        for module in &compiled.checked.bundle().modules {
            let computed_in_view = module.bodies.iter().flat_map(|body| &body.exprs).any(|expr| {
                if let ExprKind::Call { callee, .. } = expr.kind {
                    module.defs[callee.0 as usize].name_for_diagnostic().contains(&prefix)
                        && view_spans.iter().any(|span| span.contains(&expr.span.start))
                } else {
                    false
                }
            });
            if computed_in_view {
                return Err("source.age.sfc-computed: the view requires a reactive computed value".into());
            }
        }
    }
    let component = compiled
        .description
        .components
        .iter()
        .find(|component| component.module == entry && component.name == node.name.name)
        .ok_or_else(|| "package declares no component".to_string())?;

    // Flat replay is valid only for assignments. A structured handler needs
    // its declaration-check function before the consumer can run it.
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
                ace_core::source::age_component::ComponentStmt::AssignLocal { .. } => {
                    return Some("local assignment");
                }
                ace_core::source::age_component::ComponentStmt::For { .. } => {
                    return Some("for");
                }
                ace_core::source::age_component::ComponentStmt::While { .. } => {
                    return Some("while");
                }
                ace_core::source::age_component::ComponentStmt::Effect { .. } => {
                    return Some("effect");
                }
                ace_core::source::age_component::ComponentStmt::Send { .. } => {
                    return Some("message call");
                }
                ace_core::source::age_component::ComponentStmt::Call { .. } => {}
            }
        }
        None
    }
    let has_payload = |msg: &str| {
        component
            .messages
            .iter()
            .any(|variant| variant.name == msg && !variant.payload.is_empty())
    };
    let use_checked_body = |msg: &str| {
        has_payload(msg) || component.checked_handler_symbols.contains_key(msg)
    };
    for (msg, stmts) in &component.handler_stmts {
        if use_checked_body(msg) {
            continue;
        }
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
    let msgs_ref: Vec<&str> = msgs
        .iter()
        .filter(|msg| !use_checked_body(msg))
        .map(|msg| msg.as_str())
        .collect();
    let lane = PackageFragmentLane::compile(units, entry, &msgs_ref)?;
    let list_fields: BTreeSet<_> = component
        .fields
        .iter()
        .filter(|(_, ty)| ty == "List")
        .map(|(name, _)| name.as_str())
        .collect();
    let has_lists = !list_fields.is_empty();

    // ---- script：helper 定义 + refs + handlers ----
    let mut helper_fns: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut runtime_imports = std::collections::BTreeSet::new();
    let mut inits: Vec<(String, String)> = Vec::new();
    for (name, _ty) in &component.fields {
        let (js, helpers) = field_init_js(component, name, &lane, list_fields.contains(name.as_str()))?;
        helper_fns.extend(helpers);
        inits.push((name.clone(), js));
    }
    let mut script = String::new();
    let handles_route_commands = route.as_ref().is_some_and(|route| !route.commands.is_empty());
    if has_lists || handles_route_commands {
        script.push_str("import { onBeforeUnmount, ref } from 'vue'\n");
    } else {
        script.push_str("import { ref } from 'vue'\n");
    }
    if let Some(route) = &route {
        script.push_str("import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'\n");
        script.push_str("const aceRoute = useRoute()\nconst aceRouter = useRouter()\n");
        if handles_route_commands {
            script.push_str("let aceRouteActive = true\nonBeforeUnmount(() => { aceRouteActive = false })\n");
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
    if has_lists {
        script.push_str(TEXT_LIST_MODEL_HELPERS);
        script.push_str("let __ageListModelActive = true;\nonBeforeUnmount(() => {\n  __ageListModelActive = false;\n");
        for field in &list_fields {
            script.push_str(&format!("  __ageReleaseTextList({field}.value);\n"));
        }
        script.push_str("})\n");
    }
    for (msg, assigns) in &component.handler_assigns {
        if route.as_ref().is_some_and(|route| route.commands.contains_key(msg)) {
            continue;
        }
        let fn_name = handler_fn_name(msg);
        if use_checked_body(msg) {
            let diagnostic = if has_payload(msg) { "source.age.sfc-payload" } else { "source.age.sfc-handler" };
            let symbol = component.checked_handler_symbols.get(msg)
                .ok_or_else(|| format!("{diagnostic}: no checked model body for `{msg}`"))?;
            let body = lane.emit_checked_handler_body(symbol, &component.fields)
                .map_err(|err| format!("{diagnostic}: `{msg}`: {err}"))?;
            for helper in body.helpers {
                if helper.starts_with("a2x_fn_") {
                    helper_fns.insert(helper);
                    continue;
                }
                if !ace_a2x_ts::runtime_export_names().contains(&helper.as_str()) {
                    return Err(format!("{diagnostic}: `{msg}` needs unsupported script helper `{helper}`"));
                }
                runtime_imports.insert(helper);
            }
            script.push_str(&format!("function {fn_name}({}) {{\n", body.params.join(", ")));
            for line in body.lines {
                script.push_str(&format!("  {line}\n"));
            }
            script.push_str("}\n");
            continue;
        }
        script.push_str(&format!("function {fn_name}() {{\n"));
        if has_lists {
            script.push_str("  if (!__ageListModelActive) return;\n");
        }
        for (field, rhs) in assigns {
            let is_list = list_fields.contains(field.as_str());
            if is_list {
                if let Some(value) = text_list_literal_js(rhs)? {
                    script.push_str(&format!("  __ageReplaceTextList({field}, {value});\n"));
                    continue;
                }
            }
            let state_fn = format!("__age_state_0_{field}");
            let span = rhs.span.start..rhs.span.end;
            let frag = lane.emit_expr_at_span(&state_fn, span).map_err(|e| {
                format!("source.age.sfc-expr: handler `{msg}` field `{field}`: {e}")
            })?;
            if frag.text.is_empty() && !is_list {
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
                let value = rewrite_lane_locals(&frag.expr);
                if is_list {
                    script.push_str(&format!("    __ageReplaceTextList({field}, {value});\n  }}\n"));
                } else {
                    script.push_str(&format!("    {field}.value = {value};\n  }}\n"));
                }
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
    let mut emitted = std::collections::BTreeSet::new();
    while let Some(helper) = helper_fns.pop_first() {
        if helper.starts_with("a2x_fn_") {
            if emitted.insert(helper.clone()) {
                helper_fns.extend(emit_helper(&lane, &helper, &mut script)?);
            }
        } else if !matches!(helper.as_str(), "a2x_add" | "a2x_sub" | "a2x_mul")
            && ace_a2x_ts::runtime_export_names().contains(&helper.as_str())
        {
            runtime_imports.insert(helper);
        }
    }
    if !has_lists {
        if !runtime_imports.is_empty() {
            let names = runtime_imports.iter().map(String::as_str).collect::<Vec<_>>().join(", ");
            script.push_str(&format!("import {{ {names} }} from '@/a2x_rt.mjs'\n"));
        }
        script.push_str(&runtime_helpers(&runtime_imports));
    }
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
    let mut ctx = TmplCtx::new(&mut template, route.is_some(), shell);
    ctx.component = Some(component);
    ctx.push_str("<div class=\"age-sfc\">");
    render_items(&node.view.tree, &mut ctx)?;
    ctx.push_str("</div>");
    script.push_str(&ctx.widget_state);
    if ctx.pointer_move_used {
        script.push_str(POINTER_SAMPLE);
        script.push_str(POINTER_MOVE);
    }


    let runtime = if has_lists {
        format!("<script>\n{}</script>\n\n",
            ace_a2x_ts::script_runtime_body(ace_a2x_ts::ScriptLang::JavaScript, &[]))
    } else {
        String::new()
    };
    let mut source = format!("<template>\n{template}\n</template>\n\n{runtime}<script setup>\n{script}</script>\n");
    for css in component.members.iter().filter(|member| member.role == "scoped-style").filter_map(|member| member.text.as_deref()) {
        if css.to_ascii_lowercase().contains("</style") {
            return Err("source.age.sfc-style: style block contains a closing style tag".into());
        }
        source.push_str("\n<style scoped>");
        source.push_str(css);
        source.push_str("</style>\n");
    }
    Ok(PackageSfc {
        source,
        uses_runtime: !has_lists && !runtime_imports.is_empty(),
        embedded_widget_modules: BTreeSet::new(),
    })
}
/// PLAN-105 T-02（U03 真语料）：带工程根的 SFC 生成——entry 源先过组件面源准备
/// （`use` 行空白化 + 依赖样式内联 + 类型化列表数据字段改写），再走组件投影；
/// 其余单元已由 `desugar_units` 统一 f-string 脱糖。
pub fn generate_package_sfc_with_root(
    units: &[PackageModuleInput],
    entry: &str,
    project_root: Option<&std::path::Path>,
    route: Option<RouteSource>,
    shell: bool,
) -> Result<String, String> {
    // PLAN-105 T-02：单元统一 f-string 脱糖（与组件面/包桥同一入口）。
    let units = &crate::ui::ace_package_units::desugar_units(units.to_vec())?[..];
    let entry_src = units
        .iter()
        .find(|u| format!("{}::{}", u.package, u.id) == entry)
        .map(|u| u.source.clone())
        .ok_or_else(|| format!("entry module `{entry}` is not in the package units"))?;
    let prepared_entry = match project_root {
        Some(root) => crate::ui::ace_source_prep::prepare_component_source(root, &entry_src)
            .map(|p| p.source)
            .unwrap_or_else(|_| entry_src.clone()),
        None => entry_src.clone(),
    };
    let node = package_candidates(&prepared_entry)?
        .into_iter()
        .next()
        .ok_or_else(|| "no AGE node candidate in the entry module".to_string())?;
    // entry 单元用**源准备后的源**参与包编译（`use` 已空白化、依赖样式已内联），
    // 否则包面仍会解析原始 `use`（导入样式 = 具名待接面）。
    let prepared_units: Vec<PackageModuleInput> = units
        .iter()
        .map(|u| {
            if format!("{}::{}", u.package, u.id) == entry {
                PackageModuleInput {
                    source: prepared_entry.clone(),
                    ..u.clone()
                }
            } else {
                u.clone()
            }
        })
        .collect();
    Ok(generate_package_sfc_leaf(&prepared_units, entry, route, shell)?.source)
}

struct TmplCtx<'a> {
    out: &'a mut String,
    routed: bool,
    shell: bool,
    widget_state: String,
    widget_count: usize,
    scroll_depth: usize,
    svg_depth: usize,
    pointer_move_used: bool,
    component: Option<&'a AgePackageComponent>,
}

impl<'a> TmplCtx<'a> {
    fn new(out: &'a mut String, routed: bool, shell: bool) -> Self {
        Self {
            out,
            routed,
            shell,
            widget_state: String::new(),
            widget_count: 0,
            scroll_depth: 0,
            svg_depth: 0,
            pointer_move_used: false,
            component: None,
        }
    }

    fn push_str(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn event_handler(
        &mut self,
        element: &ace_core::source::age_component::ComponentViewElement,
        event: &str,
        message: &str,
    ) -> Result<String, String> {
        let handler = handler_fn_name(message);
        let payload = self
            .component
            .and_then(|component| component.messages.iter().find(|variant| variant.name == message))
            .map_or(&[][..], |variant| variant.payload.as_slice());
        use crate::ui::ace_widget_props::{expression, literal};
        let mouse_area = element.tag == "mouse-area";
        if mouse_area && event == "mousemove" {
            if payload.len() != 2 || !payload.iter().all(|ty| matches!(ty.as_str(), "float" | "f32")) {
                return Err(format!("source.age.sfc-pointer: `{message}` needs two f32 coordinates"));
            }
            if !self.component.is_some_and(|component| component.checked_handler_symbols.contains_key(message)) {
                return Err(format!("source.age.sfc-pointer: `{message}` needs a checked handler"));
            }
            let extent = if expression(element, "coords").is_some() {
                let (width, height) = literal(element, "coords")
                    .and_then(crate::ui::aura_view_builder::parse_coords_extent)
                    .filter(|(width, height)| width.is_finite() && height.is_finite())
                    .ok_or_else(|| "source.age.sfc-pointer: coords must have a finite WxH extent".to_owned())?;
                format!(", {width}, {height}")
            } else {
                String::new()
            };
            let leave = expression(element, "onmouseleave")
                .and_then(event_message)
                .map(handler_fn_name)
                .unwrap_or_else(|| "null".into());
            self.pointer_move_used = true;
            return Ok(format!("acePointerMove($event, {handler}, {leave}{extent})"));
        }
        if !payload.is_empty() {
            return Err(format!("source.age.sfc-payload: `{event}` does not supply the payload for `{message}`"));
        }
        if mouse_area && event == "mouseleave" && expression(element, "onmousemove").is_some() {
            return Ok("acePointerLeave($event.currentTarget)".into());
        }
        Ok(handler)
    }
}

const POINTER_MOVE: &str = r#"import { onMounted as acePointerMounted, onBeforeUnmount as acePointerUnmounted } from 'vue'
const acePointerSamples = new WeakMap()
const acePointerAreas = new Map()
function acePointerLeave(element) {
  const leave = acePointerAreas.get(element)
  acePointerAreas.delete(element)
  acePointerSamples.delete(element)
  if (element.isConnected && leave) leave()
}
// Replacing a hovered SVG child can discard its browser mouseleave path.
function aceCheckPointerAreas(event) {
  for (const element of acePointerAreas.keys()) {
    if (!element.contains(event.target)) acePointerLeave(element)
  }
}
acePointerMounted(() => document.addEventListener('mousemove', aceCheckPointerAreas, true))
acePointerUnmounted(() => {
  document.removeEventListener('mousemove', aceCheckPointerAreas, true)
  acePointerAreas.clear()
})
function acePointerMove(event, send, leave, width, height) {
  const element = event.currentTarget
  if (leave) acePointerAreas.set(element, leave)
  const sample = acePointerSample(event, acePointerSamples, width, height)
  if (sample !== undefined) send(...sample)
}
"#;

const POINTER_SAMPLE: &str = r#"function acePointerSample(event, samples, width, height) {
  const element = event.currentTarget
  const bounds = element.getBoundingClientRect()
  if (bounds.width <= 0 || bounds.height <= 0) return
  const x = Math.fround((event.clientX - bounds.left) * (width === undefined ? 1 : width / bounds.width))
  const y = Math.fround((event.clientY - bounds.top) * (height === undefined ? 1 : height / bounds.height))
  if (!Number.isFinite(x) || !Number.isFinite(y)) return
  const qx = Math.round(x * 2)
  const qy = Math.round(y * 2)
  const previous = samples.get(element)
  const now = performance.now()
  if (previous && (now - previous.now < 33 || (previous.qx === qx && previous.qy === qy))) return
  samples.set(element, { qx, qy, now })
  return [x, y]
}
"#;

fn event_message(expr: &ace_core::source::ast::Expr) -> Option<&str> {
    use ace_core::source::ast::ExprKind;
    match &expr.kind {
        ExprKind::Var(name) => Some(&name.name),
        _ => crate::ui::ace_widget_props::self_field(expr),
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
    is_list: bool,
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
    if is_list {
        if let Some(value) = text_list_literal_js(expr)? {
            return Ok((value, Vec::new()));
        }
    }
    if let ace_core::source::ast::ExprKind::StrLit(value) = &expr.kind {
        return Ok((js_string(value), Vec::new()));
    }
    // 受检初值表达式（含 helper 调用）经同一片段 lane 发射；局部名=ref 名。
    let state_fn = format!("__age_state_0_{field}");
    let span = expr.span.start..expr.span.end;
    match lane.emit_expr_at_span(&state_fn, span) {
        Ok(frag) if is_list => {
            let lines = rewrite_lane_locals(&frag.text.join(" "));
            let value = rewrite_lane_locals(&frag.expr);
            Ok((format!("(() => {{ {lines} return {value}; }})()"), frag.helpers))
        }
        Ok(frag) if !frag.text.is_empty() => {
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
        _ => Ok((expr_to_js(expr)?, Vec::new())),
    }
}

const TEXT_LIST_MODEL_HELPERS: &str = r#"
function __ageCreateTextList(strings) {
  const list = a2x_list_new("str");
  try {
    for (const value of strings) a2x_list_push(list, a2x_str_to_owned(a2x_str_lit(value)));
  } catch (error) {
    __ageReleaseTextList(list);
    throw error;
  }
  return list;
}

function __ageReleaseTextList(list) {
  for (let i = a2x_list_len(list); i > 0; i--) {
    a2x_str_free(a2x_list_remove(list, i - 1));
  }
}

function __ageReplaceTextList(field, next) {
  const previous = field.value;
  if (previous === next) return;
  field.value = next;
  __ageReleaseTextList(previous);
}
"#;

fn text_list_literal_js(expr: &ace_core::source::ast::Expr) -> Result<Option<String>, String> {
    use ace_core::source::ast::ExprKind;
    let ExprKind::ArrayLit(items) = &expr.kind else {
        return Ok(None);
    };
    let values = items
        .iter()
        .map(|item| match &item.kind {
            ExprKind::StrLit(value) => Ok(js_string(value)),
            _ => Err("source.age.sfc-expr: expected a checked text-list literal"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(format!("__ageCreateTextList([{}])", values.join(", "))))
}

/// 表达式 → JS（域内标量；调用/文本面不在本刀）。
fn expr_to_js(expr: &ace_core::source::ast::Expr) -> Result<String, String> {
    use ace_core::source::ast::{Arg, ExprKind, UnaryOp};
    match &expr.kind {
        ExprKind::IntLit(i) => Ok(i.to_string()),
        ExprKind::BoolLit(b) => Ok(b.to_string()),
        ExprKind::StrLit(s) => Ok(js_string(s)),
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
    expr_to_template_js_with_strings(expr, |text| format!("{text:?}"))
}

fn expr_to_template_js_with_strings(
    expr: &ace_core::source::ast::Expr,
    string_literal: fn(&str) -> String,
) -> Result<String, String> {
    use ace_core::source::ast::{Arg, ExprKind, UnaryOp};
    match &expr.kind {
        ExprKind::IntLit(i) => Ok(i.to_string()),
        ExprKind::BoolLit(b) => Ok(b.to_string()),
        ExprKind::StrLit(s) => Ok(string_literal(s)),
        ExprKind::Var(id) => Ok(id.name.clone()),
        ExprKind::Field { base, name } => {
            if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") {
                Ok(name.name.clone())
            } else {
                Err("source.age.sfc-expr: nested field access outside this slice".into())
            }
        }
        ExprKind::Unary { op, operand } => {
            let inner = expr_to_template_js_with_strings(operand, string_literal)?;
            match op {
                UnaryOp::Neg => Ok(format!("(-{inner})")),
                UnaryOp::Not => Ok(format!("(!{inner})")),
                _ => Err("source.age.sfc-expr: unary op outside this slice".into()),
            }
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let l = expr_to_template_js_with_strings(lhs, string_literal)?;
            let r = expr_to_template_js_with_strings(rhs, string_literal)?;
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
                ace_core::source::ast::BinOp::And => "&&",
                ace_core::source::ast::BinOp::Or => "||",
                _ => return Err("source.age.sfc-expr: binary op outside this slice".into()),
            };
            Ok(format!("({l} {js_op} {r})"))
        }
        ExprKind::Call { args, .. } if is_format_call(expr) => {
            let mut parts = vec![string_literal("")];
            for arg in args {
                let Arg::Pos(part) = arg else {
                    return Err("source.age.sfc-expr: invalid format string argument".into());
                };
                parts.push(expr_to_template_js_with_strings(part, string_literal)?);
            }
            Ok(format!("({})", parts.join(" + ")))
        }
        _ => Err("source.age.sfc-expr: template expression outside this slice".into()),
    }
}

fn is_format_call(expr: &ace_core::source::ast::Expr) -> bool {
    matches!(&expr.kind, ace_core::source::ast::ExprKind::Call { callee, .. }
        if callee.name == "__fstr")
}

fn js_string(value: &str) -> String {
    serde_json::Value::String(value.to_string())
        .to_string()
        .replace('<', "\\u003c")
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
) -> Result<Vec<String>, String> {
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
    Ok(body.helpers)
}

/// Retain the flat-assignment lane's self-contained scalar helpers when a
/// typed handler in the same file has not already imported their definitions.
fn runtime_helpers(imports: &std::collections::BTreeSet<String>) -> String {
    let mut out = String::new();
    for (name, operator) in [("a2x_add", "+"), ("a2x_sub", "-"), ("a2x_mul", "*")] {
        if !imports.contains(name) {
            out.push_str(&format!(
                "function {name}(a, b) {{\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a {operator} b)) throw new RangeError('{name} overflow');\n  return a {operator} b;\n}}\n"
            ));
        }
    }
    out
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

// Vue's type checker also reads the directive as source text. Use JS escapes
// for characters significant to the surrounding double-quoted attribute.
fn class_binding_string(value: &str) -> String {
    let mut literal = String::from("'");
    for character in value.chars() {
        match character {
            '"' | '\'' | '\\' | '&' | '<' | '>' | '\u{2028}' | '\u{2029}' => {
                literal.extend(character.escape_unicode());
            }
            character if character.is_control() => literal.extend(character.escape_unicode()),
            character => literal.push(character),
        }
    }
    literal.push('\'');
    literal
}

fn class_binding_expr(expr: &ace_core::source::ast::Expr) -> Result<String, String> {
    expr_to_template_js_with_strings(expr, class_binding_string)
}

fn append_conditional_classes(
    element: &ComponentViewElement,
    literals: &mut Vec<String>,
    bindings: &mut Vec<String>,
) -> Result<(), String> {
    for part in conditional_classes(element) {
        match part {
            ComponentClassPart::Value(value) => {
                let ace_core::source::ast::ExprKind::StrLit(value) = &value.kind else {
                    return Err("source.age.sfc-style: class list item is not text".into());
                };
                literals.push(value.clone());
            }
            ComponentClassPart::If { cond, then_value, else_value } => {
                let cond = class_binding_expr(cond)?;
                let then_value = class_binding_expr(then_value)?;
                let else_value = class_binding_expr(else_value)?;
                bindings.push(format!("({cond} ? {then_value} : {else_value})"));
            }
        }
    }
    Ok(())
}

fn render_svg_element(
    element: &ace_core::source::age_component::ComponentViewElement,
    ctx: &mut TmplCtx<'_>,
) -> Result<(), String> {
    let tag = &element.tag;
    let mut attributes = ace_svg::attributes(element, "source.age.sfc-svg")?;
    let text = attributes.remove("text");
    ctx.push_str(&format!("<{tag}"));
    if tag == "svg" {
        ctx.push_str(&format!(" xmlns=\"{}\"", ace_svg::NAMESPACE));
    }
    let mut literal_classes = Vec::new();
    let mut bound_classes = Vec::new();
    for (key, expr) in attributes {
        let literal = ace_svg::scalar_literal(expr);
        if matches!(key, "style" | "class") {
            match literal {
                Some(class) => literal_classes.push(class),
                None => bound_classes.push(class_binding_expr(expr)?),
            }
            continue;
        }
        match literal {
            Some(value) => ctx.push_str(&format!(" {key}=\"{}\"", escape_attr(&value))),
            None => ctx.push_str(&format!(
                " :{key}=\"{}\"",
                escape_attr(&expr_to_template_js(expr)?)
            )),
        }
    }
    append_conditional_classes(element, &mut literal_classes, &mut bound_classes)?;
    if !literal_classes.is_empty() {
        ctx.push_str(&format!(" class=\"{}\"", escape_attr(&literal_classes.join(" "))));
    }
    if !bound_classes.is_empty() {
        ctx.push_str(&format!(" :class=\"[{}]\"", bound_classes.join(", ")));
    }
    ctx.push_str(">");
    if let Some(label) = element.label.as_ref().or(text) {
        match ace_svg::scalar_literal(label) {
            Some(value) => ctx.push_str(&escape_html(&value)),
            None => ctx.push_str(&format!("{{{{ {} }}}}", expr_to_template_js(label)?)),
        }
    }
    ctx.svg_depth += 1;
    let result = render_items(&element.items, ctx);
    ctx.svg_depth -= 1;
    result?;
    ctx.push_str(&format!("</{tag}>"));
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
                if ctx.svg_depth == 0 && is_format_call(&leaf.expr) {
                    let value = class_binding_expr(&leaf.expr)?;
                    ctx.push_str(&format!("<span v-text=\"{value}\"></span>"));
                    continue;
                }
                if ctx.svg_depth > 0 {
                    ctx.push_str("<text>");
                } else if ctx.scroll_depth > 0 {
                    ctx.push_str("<span>");
                }
                if let ace_core::source::ast::ExprKind::StrLit(s) = &leaf.expr.kind {
                    ctx.push_str(&escape_html(s));
                } else {
                    let js = expr_to_template_js(&leaf.expr)?;
                    ctx.push_str(&format!("{{{{ {js} }}}}"));
                }
                if ctx.svg_depth > 0 {
                    ctx.push_str("</text>");
                } else if ctx.scroll_depth > 0 {
                    ctx.push_str("</span>");
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
                if ctx.svg_depth > 0 || el.tag == "svg" {
                    render_svg_element(el, ctx)?;
                    continue;
                }
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
                if widgets::render(el, ctx)? {
                    continue;
                }
                let tag = match el.tag.as_str() {
                    "col" | "row" | "scroll" | "mouse-area" => "div",
                    "text" if ctx.scroll_depth > 0 => "span",
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
                if el.tag == "scroll" {
                    classes.push("overflow-y-auto".into());
                }
                let mut events: Vec<(String, String)> = Vec::new();
                let mut bound_classes = Vec::new();
                let mut accessibility = std::collections::BTreeMap::new();
                for sub in &el.items {
                    if let ComponentViewItem::Entry(en) = sub {
                        match en.key.as_str() {
                            "style" | "class" => {
                                if let ComponentEntryValue::Expr(e) = &en.value {
                                    if let ace_core::source::ast::ExprKind::StrLit(s) = &e.kind {
                                        classes.push(s.clone());
                                    } else if en.key == "class" && is_format_call(e) {
                                        bound_classes.push(class_binding_expr(e)?);
                                    }
                                }
                            }
                            k if k == "role" || k.strip_prefix("aria-").is_some_and(|suffix| {
                                !suffix.is_empty() && suffix.bytes().all(|byte| {
                                    byte.is_ascii_alphanumeric() || byte == b'-'
                                })
                            }) => {
                                accessibility.insert(k, &en.value);
                            }
                            k if k.starts_with("on") => {
                                if let Some((message, args)) = en.message_call() {
                                    if !args.is_empty() {
                                        return Err(format!(
                                            "source.age.sfc-event-payload: typed view event `{}` requires an emitted handler signature",
                                            message.name,
                                        ));
                                    }
                                    events.push((
                                        k.trim_start_matches("on").to_string(),
                                        handler_fn_name(&message.name),
                                    ));
                                    continue;
                                }
                                if let ComponentEntryValue::Expr(e) = &en.value {
                                    if let Some(msg) = event_message(e) {
                                        let event = k.trim_start_matches("on");
                                        events.push((
                                            event.to_string(),
                                            ctx.event_handler(el, event, msg)?,
                                        ));
                                    } else if el.tag == "mouse-area" {
                                        return Err(format!("source.age.sfc-pointer: `{k}` needs a direct message binding"));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                append_conditional_classes(el, &mut classes, &mut bound_classes)?;
                ctx.push_str(&format!("<{tag}"));
                let scroll = el.tag == "scroll"
                    || classes.iter().any(|class| {
                        class.split_whitespace().any(|name| {
                            matches!(name, "overflow-auto" | "overflow-x-auto" | "overflow-y-auto")
                        })
                    });
                if scroll {
                    ctx.push_str(" tabindex=\"0\"");
                }
                if !classes.is_empty() {
                    ctx.push_str(&format!(" class=\"{}\"", escape_attr(&classes.join(" "))));
                }
                if !bound_classes.is_empty() {
                    ctx.push_str(&format!(" :class=\"[{}]\"", bound_classes.join(", ")));
                }
                for (key, binding) in accessibility {
                    let ComponentEntryValue::Expr(expr) = binding else {
                        return Err(format!("source.age.sfc-accessibility: unsupported `{key}` binding"));
                    };
                    let (prefix, value) = match &expr.kind {
                        ace_core::source::ast::ExprKind::StrLit(value) => ("", value.clone()),
                        _ => (":", expr_to_template_js(expr)?),
                    };
                    ctx.push_str(&format!(" {prefix}{key}=\"{}\"", escape_attr(&value)));
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
                    } else if is_format_call(label) {
                        let value = class_binding_expr(label)?;
                        ctx.push_str(&format!("<span v-text=\"{value}\"></span>"));
                    }
                } else if matches!(el.tag.as_str(), "span" | "td") {
                    if let Some(label) = crate::ui::ace_widget_props::literal(el, "text") {
                        ctx.push_str(&escape_html(label));
                    }
                }
                ctx.scroll_depth += usize::from(scroll);
                let children = render_items(&el.items, ctx);
                ctx.scroll_depth -= usize::from(scroll);
                children?;
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
        let mut ctx = TmplCtx::new(&mut out, false, false);
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
