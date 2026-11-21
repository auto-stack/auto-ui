//! Shared route admission for the ACE package consumers.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ace_av::age_component::project_component_candidates;
use ace_core::source::age_component::ComponentNode;
use ace_core::source::age_component::ComponentScenario;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    parse_package_source_age, PackageModuleInput, PACKAGES_CAPABILITY,
};

pub(super) type RouteModulePaths = BTreeMap<String, PathBuf>;

pub(super) fn front_package_units(
    app: &Path,
    source: &str,
    modules: &[PathBuf],
) -> Result<(Vec<PackageModuleInput>, RouteModulePaths), String> {
    let unit = |id: String, source: String| PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id,
        source,
        provider: None,
    };
    let mut paths = BTreeMap::from([("app::app".to_string(), app.to_path_buf())]);
    let mut units = vec![unit("app".into(), source.into())];
    for module in modules {
        if let Some(id) = module.file_stem().and_then(|name| name.to_str()) {
            let source = std::fs::read_to_string(module)
                .map_err(|error| format!("ace.route.module: {}: {error}", module.display()))?;
            units.push(unit(id.into(), source));
            paths.insert(format!("app::{id}"), module.clone());
        }
    }
    Ok((units, paths))
}

pub(super) fn package_candidates(source: &str) -> Result<Vec<ComponentNode>, String> {
    parse_package_source_age(
        source,
        &[PACKAGES_CAPABILITY.to_string()],
        FrontBudget::WORK_OWNED_64MIB,
        &ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
        0,
    )
    .map(|syntax| {
        syntax
            .age_components
            .into_iter()
            .map(|(node, _, _)| node)
            .collect()
    })
    .map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join(", ")
    })
}

#[derive(Debug)]
pub(crate) struct RouteTable {
    root: String,
    outlet_after: usize,
    routes: Vec<Route>,
}

#[derive(Debug)]
pub(crate) struct Route {
    pub pattern: String,
    pub module: String,
    pub page: Result<String, String>,
    segments: Vec<Segment>,
}

#[derive(Debug, PartialEq, Eq)]
enum Segment {
    Literal(String),
    Param(String),
}

impl RouteTable {
    pub fn from_root(
        root: &ComponentNode,
        units: &[PackageModuleInput],
        entry: &PackageModuleInput,
        validate_pages: bool,
    ) -> Result<Option<Self>, String> {
        let mut patterns = BTreeSet::new();
        let routes = root
            .members
            .iter()
            .filter(|row| row.role == "route")
            .map(|row| {
                let context = format!(
                    "{}::{} @{}..{}",
                    entry.package, entry.id, row.span.start, row.span.end
                );
                let pattern: String = serde_json::from_str(&row.name).map_err(|_| {
                    format!("ace.route.pattern: {context}: invalid path {}", row.name)
                })?;
                let segments = parse_pattern(&pattern).ok_or_else(|| {
                    format!("ace.route.pattern: {context}: invalid path {pattern:?}")
                })?;
                if !patterns.insert(pattern.clone()) {
                    return Err(format!(
                        "ace.route.pattern: {context}: duplicate path {pattern:?}"
                    ));
                }

                let module = row.text.clone().ok_or_else(|| {
                    format!("ace.route.module: {context}: path {pattern:?} has no module")
                })?;
                let page = resolve_page(units, entry, &module, validate_pages).map_err(|reason| {
                    format!("ace.route.page: path {pattern:?} module {module:?}: {reason}")
                });
                Ok(Route {
                    pattern,
                    module,
                    page,
                    segments,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        if routes.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            root: format!("{}::{}", entry.package, entry.id),
            outlet_after: outlet_position(root)?,
            routes,
        }))
    }

    pub fn outlet_after(&self) -> usize {
        self.outlet_after
    }

    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    pub fn entry_units(
        &self,
        units: &[PackageModuleInput],
        entry: &str,
    ) -> Vec<PackageModuleInput> {
        let package = entry.split_once("::").map(|(package, _)| package);
        let (selected, dependencies): (Vec<_>, Vec<_>) = units
            .iter()
            .filter(|unit| {
                let key = format!("{}::{}", unit.package, unit.id);
                key == entry
                    || (key != self.root
                        && (Some(unit.package.as_str()) != package
                            || !self.routes.iter().any(|route| route.module == unit.id)))
            })
            .partition(|unit| format!("{}::{}", unit.package, unit.id) == entry);
        selected.into_iter().chain(dependencies).cloned().collect()
    }

    pub fn resolve(&self, path: &str) -> Result<(&Route, BTreeMap<String, String>), String> {
        let parts = path_segments(path);
        if let Some(parts) = parts {
            for route in &self.routes {
                if route.segments.len() != parts.len() {
                    continue;
                }
                let mut params = BTreeMap::new();
                if route
                    .segments
                    .iter()
                    .zip(&parts)
                    .all(|(segment, value)| match segment {
                        Segment::Literal(expected) => expected == *value,
                        Segment::Param(name) => {
                            params.insert(name.clone(), (*value).to_string());
                            true
                        }
                    })
                {
                    return Ok((route, params));
                }
            }
        }
        Err(format!("ace.route.no-match: no route for {path:?}"))
    }
}

fn outlet_position(root: &ComponentNode) -> Result<usize, String> {
    use ace_core::source::age_component::ComponentViewItem;
    let mut items: Vec<_> = root.view.tree.iter().rev().collect();
    let mut seen = 0;
    let mut outlet = None;
    while let Some(item) = items.pop() {
        match item {
            ComponentViewItem::Text(_) => seen += 1,
            ComponentViewItem::Element(element) => {
                if element.tag == "text" {
                    seen += 1;
                }
                if element.tag == "outlet" && outlet.replace(seen).is_some() {
                    return Err(format!(
                        "ace.route.outlet: {} declares multiple outlets",
                        root.name.name
                    ));
                }
                items.extend(element.items.iter().rev());
            }
            ComponentViewItem::Cond(condition) => {
                items.extend(condition.else_items.iter().rev());
                items.extend(condition.then_items.iter().rev());
            }
            ComponentViewItem::For(_) | ComponentViewItem::Entry(_) => {}
        }
    }
    outlet.ok_or_else(|| format!("ace.route.outlet: {} requires an outlet", root.name.name))
}

fn path_segments(path: &str) -> Option<Vec<&str>> {
    let tail = path.strip_prefix('/')?;
    if tail.is_empty() {
        return Some(Vec::new());
    }
    let parts: Vec<_> = tail.split('/').collect();
    parts.iter().all(|part| !part.is_empty()).then_some(parts)
}

fn parse_pattern(pattern: &str) -> Option<Vec<Segment>> {
    let mut params = BTreeSet::new();
    path_segments(pattern)?
        .into_iter()
        .map(|part| {
            if let Some(name) = part.strip_prefix(':') {
                let mut chars = name.chars();
                if !chars
                    .next()
                    .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
                    || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                    || !params.insert(name)
                {
                    return None;
                }
                Some(Segment::Param(name.to_string()))
            } else {
                Some(Segment::Literal(part.to_string()))
            }
        })
        .collect()
}

fn resolve_page(
    units: &[PackageModuleInput],
    entry: &PackageModuleInput,
    alias: &str,
    validate_page: bool,
) -> Result<String, String> {
    let id = format!("{}::{alias}", entry.package);
    let mut targets = units
        .iter()
        .filter(|unit| unit.package == entry.package && unit.id == alias);
    let target = targets
        .next()
        .ok_or_else(|| format!("{id}: module is missing"))?;
    if targets.next().is_some() {
        return Err(format!("{id}: module is ambiguous"));
    }
    if target.id == entry.id {
        return Err(format!("{id}: page refers to the app entry"));
    }
    if validate_page {
        let components = package_candidates(&target.source)
            .map_err(|reason| format!("{id}: projection failed: {reason}"))?;
        if page_widget(&components).is_none() {
            return Err(format!("{id}: module has no page widget with a view"));
        }
    }
    Ok(id)
}

fn page_widget(candidates: &[ComponentNode]) -> Option<&ComponentNode> {
    candidates.first().filter(|node| node.view.exits.iter().any(|exit| exit.kind == "view"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const APP: &str = include_str!("../../tests/fixtures/plan124/app.at");
    const KANBAN: &str = include_str!("../../tests/fixtures/plan124/kanban.at");
    const BOARD: &str = include_str!("../../tests/fixtures/plan124/board.at");

    fn unit(id: &str, source: &str) -> PackageModuleInput {
        PackageModuleInput {
            package: "fixture".into(),
            version: "1.0.0".into(),
            id: id.into(),
            source: source.into(),
            provider: None,
        }
    }

    fn routes(units: &[PackageModuleInput]) -> RouteTable {
        let shell = &units[0];
        let candidates = package_candidates(&shell.source).unwrap();
        RouteTable::from_root(&candidates[0], units, shell, true)
            .unwrap()
            .expect("route rows")
    }

    #[test]
    fn project_reader_aliases_parameters_and_independent_errors() {
        let units = [
            unit("app", APP),
            unit(
                "bookshelf",
                include_str!("../../tests/fixtures/plan124/bookshelf.at"),
            ),
            unit(
                "book_detail",
                include_str!("../../tests/fixtures/plan124/book_detail.at"),
            ),
            unit(
                "settings",
                include_str!("../../tests/fixtures/plan124/settings.at"),
            ),
            unit(
                "reading",
                include_str!("../../tests/fixtures/plan124/reading.at"),
            ),
            unit(
                "chapter_math",
                include_str!("../../tests/fixtures/plan124/chapter_math.at"),
            ),
            unit(
                "broken",
                include_str!("../../tests/fixtures/plan124/broken.at"),
            ),
        ];
        let table = routes(&units);
        assert_eq!(table.routes().len(), 6);
        let (first, parameters) = table.resolve("/").unwrap();
        assert_eq!(first.page.as_deref(), Ok("fixture::bookshelf"));
        assert!(parameters.is_empty());

        let (detail, parameters) = table.resolve("/book/17").unwrap();
        assert_eq!(detail.page.as_deref(), Ok("fixture::book_detail"));
        assert_eq!(parameters.get("id").map(String::as_str), Some("17"));

        for (path, diagnostic) in [
            ("/missing", "module is missing"),
            ("/broken", "has no page widget"),
        ] {
            let (route, _) = table.resolve(path).unwrap();
            let error = route.page.as_ref().unwrap_err();
            assert!(
                error.contains(path) && error.contains(diagnostic),
                "{error}"
            );
        }
        assert!(table
            .resolve("/book")
            .unwrap_err()
            .contains("ace.route.no-match"));
        assert_eq!(
            table.resolve("/settings").unwrap().0.page.as_deref(),
            Ok("fixture::settings")
        );
    }

    #[test]
    fn project_kanban_shell_and_legacy_single_page() {
        let units = [unit("app", KANBAN), unit("board", BOARD)];
        assert_eq!(
            routes(&units).resolve("/").unwrap().0.page.as_deref(),
            Ok("fixture::board")
        );

        let page = &units[1];
        let candidates = package_candidates(&page.source).unwrap();
        assert!(RouteTable::from_root(&candidates[0], &units, page, true)
            .unwrap()
            .is_none());
    }
}

#[derive(Clone, Debug)]
pub(super) enum RouteAction {
    Push(Vec<RoutePart>),
    Back,
}

#[derive(Clone, Debug)]
pub(super) enum RoutePart {
    Literal(String),
    IntField(String),
}

pub(super) enum RouteNavigation {
    Push(String),
    Back,
}

impl RouteAction {
    pub fn literal(path: String) -> Self {
        Self::Push(vec![RoutePart::Literal(path)])
    }

    pub fn native(
        &self,
        state: &[(String, super::ace_state_bridge::AceStateValue)],
    ) -> Result<RouteNavigation, String> {
        use super::ace_state_bridge::AceStateValue;
        match self {
            Self::Back => Ok(RouteNavigation::Back),
            Self::Push(parts) => {
                let mut path = String::new();
                for part in parts {
                    match part {
                        RoutePart::Literal(text) => path.push_str(text),
                        RoutePart::IntField(field) => {
                            let Some((_, AceStateValue::Int(value))) =
                                state.iter().find(|(name, _)| name == field)
                            else {
                                return Err(format!(
                                    "ace.route.push: {field:?} has no live int state"
                                ));
                            };
                            path.push_str(&value.to_string());
                        }
                    }
                }
                Ok(RouteNavigation::Push(path))
            }
        }
    }

    pub fn vue(&self) -> String {
        match self {
            Self::Back => "aceRouter.back()".into(),
            Self::Push(parts) => {
                let path = parts
                    .iter()
                    .map(|part| match part {
                        RoutePart::Literal(text) => {
                            serde_json::to_string(text).expect("serialize path segment")
                        }
                        RoutePart::IntField(field) => format!("String({field}.value)"),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                format!("aceRouter.push({path})")
            }
        }
    }
}

pub(super) struct RouteSource {
    pub source: String,
    pub commands: BTreeMap<String, RouteAction>,
    pub has_init: bool,
    pub web_params: BTreeMap<String, (String, String)>,
}

pub(super) fn prepare_page(
    source: &str,
    native_params: Option<&BTreeMap<String, String>>,
) -> Result<RouteSource, String> {
    use ace_core::source::ast::{Arg, ExprKind};

    let candidates = package_candidates(source)?;
    let widget = page_widget(&candidates)
        .ok_or_else(|| "module has no page widget with a view".to_string())?;
    let mut replacements = Vec::new();
    // PLAN-105 W111：文本定位游标——resolver 对重复 `router.param` 调用的
    // expr.span 可能重复（实测两处同 span），按“上次命中之后”递进定位。
    let mut param_search_from = 0usize;
    let mut commands = BTreeMap::new();
    let mut web_params = BTreeMap::new();
    let mut initialize = false;
    for handler in &widget.handlers {
        initialize |= handler.msg == "Init";
        // PLAN-105 W111（落地适配）：AGE 投影的 span 属解析期坐标（脱糖后与
        // 原源**必然可能偏移**）——handler 体一律按 `.msg ->` 文本定位
        // （花括号配平），不信任解析 span。
        let mut span = handler.span.start as usize..handler.span.end as usize;
        {
            let needle = format!(".{} ->", handler.msg);
            if let Some(pos) = source.find(&needle) {
                if let Some(brace_rel) = source[pos..].find('{') {
                    let open = pos + brace_rel;
                    let bytes = source.as_bytes();
                    let mut depth = 0i32;
                    let mut k = open;
                    while k < bytes.len() {
                        match bytes[k] {
                            b'{' => depth += 1,
                            b'}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        k += 1;
                    }
                    // 从 `.msg` 起（含 `->`）——下游 split_once("->") 依赖它。
                    span = pos..(k + 1).min(source.len());
                }
            }
        }
        if handler.body_foreign {
            let body = source[span.clone()]
                .split_once("->")
                .map(|(_, tail)| tail.trim())
                .and_then(|tail| tail.strip_prefix('{')?.strip_suffix('}'))
                .map(str::trim)
                .unwrap_or("");
            if let Some(command) = route_command(body, widget)? {
                commands.insert(handler.msg.clone(), command);
                replacements.push((span, format!(".{} -> {{}}", handler.msg)));
            } else {
                return Err(format!("ace.route.handler: handler {:?} contains a call outside the checked route commands (body: {body:?})", handler.msg));
            }
        }
        // PLAN-105 W111（落地适配）：router 交互调用在落地后的解析器里是
        // ComponentStmt::Call（不再是 body_foreign）——语句 span 属解析期
        // 坐标（脱糖后与原源可能有偏移），故按**结构渲染回原文**再定位替换，
        // 语义与 124/125 期的 body_foreign 剥离一致。
        for stmt in &handler.stmts {
            let ace_core::source::age_component::ComponentStmt::Call { call, .. } = stmt
            else {
                continue;
            };
            let ExprKind::MethodCall { recv, name, args, .. } = &call.kind else {
                continue;
            };
            let ExprKind::Var(owner) = &recv.kind else {
                continue;
            };
            if owner.name != "router" {
                continue;
            }
            let rendered_args = args
                .iter()
                .map(|a| match a {
                    Arg::Pos(e) | Arg::MovePos(e) => render_router_arg(e),
                    Arg::Named { value, .. } => render_router_arg(value),
                })
                .collect::<Vec<_>>()
                .join(", ");
            let text = format!("router.{}({})", name.name, rendered_args);
            if let Some(command) = route_command(&text, widget)? {
                commands.insert(handler.msg.clone(), command);
                if let Some(pos) = source.find(&text) {
                    replacements.push((pos..pos + text.len(), String::new()));
                }
            } else {
                return Err(format!(
                    "ace.route.handler: handler {:?} contains a call outside the checked route commands (call: {text:?})",
                    handler.msg
                ));
            }
        }
        for assign in &handler.assigns {
            let field_type = widget
                .fields
                .iter()
                .find(|field| field.name == assign.field)
                .map(|field| field.ty.as_str());
            let mut expressions = vec![&assign.rhs];
            while let Some(expr) = expressions.pop() {
                match &expr.kind {
                    ExprKind::Method { recv, name, args }
                    | ExprKind::MethodCall { recv, name, args, .. }
                        if matches!(&recv.kind, ExprKind::Var(owner) if owner.name == "router")
                            && name.name == "param" =>
                    {
                        let [Arg::Pos(arg)] = args.as_slice() else {
                            return Err(
                                "ace.route.param: expected router.param with one literal key"
                                    .into(),
                            );
                        };
                        let ExprKind::StrLit(key) = &arg.kind else {
                            return Err("ace.route.param: expected a literal parameter key".into());
                        };
                        let kind = match field_type {
                            Some("int" | "i32") => "int",
                            Some("str") => "str",
                            _ => {
                                return Err(format!(
                                    "ace.route.param: invalid destination field {:?}",
                                    assign.field
                                ))
                            }
                        };
                        let literal = if let Some(params) = native_params {
                            let value = params.get(key).ok_or_else(|| {
                                format!("ace.route.param: path does not bind {key:?}")
                            })?;
                            if kind == "int" {
                                value.parse::<i32>().map(|number| number.to_string()).map_err(|_| {
                                    format!("ace.route.param: value {value:?} for {key:?} is outside int range")
                                })?
                            } else {
                                serde_json::to_string(value).map_err(|e| e.to_string())?
                            }
                        } else {
                            let name = format!("ace_route_parameter_{}", web_params.len());
                            if widget.fields.iter().any(|field| field.name == name) {
                                return Err(format!(
                                    "ace.route.param: generated field {name:?} is reserved"
                                ));
                            }
                            web_params.insert(name.clone(), (key.clone(), kind.to_string()));
                            format!(".{name}")
                        };
                        // PLAN-105 W111：解析期坐标（脱糖后）与原源可能偏移——
                        // 按 `router.param(<key>)` 文本定位替换（找不到才回退 span）。
                        let param_text = format!("router.param({key:?})");
                        let range = match source[param_search_from..].find(&param_text) {
                            Some(rel) => {
                                let pos = param_search_from + rel;
                                param_search_from = pos + param_text.len();
                                pos..pos + param_text.len()
                            }
                            None => {
                                let a = expr.span.start as usize;
                                let b = expr.span.end as usize;
                                if b <= source.len() { a..b } else { a..a }
                            }
                        };
                        replacements.push((range, literal));
                    }
                    ExprKind::Binary { lhs, rhs, .. } => {
                        expressions.extend([lhs.as_ref(), rhs.as_ref()])
                    }
                    ExprKind::Unary { operand, .. } => expressions.push(operand),
                    ExprKind::Call { args, .. } => {
                        expressions.extend(args.iter().map(|arg| match arg {
                            Arg::Pos(value) | Arg::MovePos(value) | Arg::Named { value, .. } => {
                                value
                            }
                        }));
                    }
                    _ => {}
                }
            }
        }
    }
    if !web_params.is_empty() {
        let first_field = widget
            .fields
            .first()
            .ok_or_else(|| "ace.route.param: missing model".to_string())?;
        // PLAN-105 W111：解析期坐标可能越原源界——插到 model 块文本处（原源）。
        let mut start = first_field.span.start as usize;
        if start > source.len() {
            start = source.find("var ").unwrap_or(source.len());
        }
        let mut declarations = String::new();
        for (field, (_, kind)) in &web_params {
            let initial = if kind == "int" { "0" } else { "\"\"" };
            declarations.push_str(&format!("var {field} {kind} = {initial}\n"));
        }
        replacements.push((start..start, declarations));
    }
    let mut rewritten = source.to_string();
    // PLAN-105 W111（落地适配）：解析期坐标与原源可能偏移（AGE 脱糖）——
    // 越界替换直接丢弃（文本定位路径已在产替换前锚定原源坐标）。
    replacements.retain(|(range, _)| range.end <= rewritten.len());
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    // 重叠防护：按 start 降序应用，新 range 的 end 不得超过上一个已应用
    // range 的 start（Call-strip 与 body_foreign 剥离同指一个 handler 时，
    // 内层已替换、外层整段跳过）。
    let mut applied_floor = rewritten.len();
    for (range, replacement) in replacements {
        if range.end > applied_floor {
            continue;
        }
        rewritten.replace_range(range.clone(), &replacement);
        applied_floor = applied_floor.min(range.start);
    }
    Ok(RouteSource {
        source: rewritten,
        commands,
        has_init: initialize,
        web_params,
    })
}


/// PLAN-105 W111：router 交互实参的结构渲染（喂回 `route_command` 的文本
/// 解析器：字符串字面量 / `+` 拼接 / `self.<int 字段>.to_string()`）。
fn render_router_arg(expr: &ace_core::source::ast::Expr) -> String {
    match &expr.kind {
        ace_core::source::ast::ExprKind::StrLit(v) => format!("{v:?}"),
        ace_core::source::ast::ExprKind::Binary {
            op: ace_core::source::ast::BinOp::Add,
            lhs,
            rhs,
        } => format!("{} + {}", render_router_arg(lhs), render_router_arg(rhs)),
        ace_core::source::ast::ExprKind::MethodCall {
            recv,
            name,
            args,
            ..
        } if name.name == "to_string" && args.is_empty() =>
        {
            if let ace_core::source::ast::ExprKind::Field { base, name } = &recv.kind {
                // AutoLang 源形态是 `.field`（点前缀字段引用）。
                let _ = base;
                return format!(".{}.to_string()", name.name);
            }
            format!("{}.to_string()", render_router_arg(recv))
        }
        _ => String::new(),
    }
}

fn route_command(body: &str, widget: &ComponentNode) -> Result<Option<RouteAction>, String> {
    let Some(call) = body.strip_prefix("router.") else {
        return Ok(None);
    };
    if call == "back()" {
        return Ok(Some(RouteAction::Back));
    }
    let Some(argument) = call
        .strip_prefix("push(")
        .and_then(|tail| tail.strip_suffix(')'))
    else {
        return Ok(None);
    };
    if let Ok(path) = serde_json::from_str::<String>(argument.trim()) {
        return Ok(Some(RouteAction::literal(path)));
    }
    let probe = format!(
        "widget RouteValue {{\nmsg {{ Check }}\nmodel {{ var path str = \"\" }}\nview {{ text \"route\" }}\non {{ .Check -> {{ .path = {argument} }} }}\n}}"
    );
    let nodes = project_component_candidates(&probe)
        .map_err(|_| "ace.route.push: could not project the destination expression".to_string())?;
    let [node] = nodes.as_slice() else {
        return Err("ace.route.push: expected one destination expression".into());
    };
    let [handler] = node.handlers.as_slice() else {
        return Err("ace.route.push: expected one destination expression".into());
    };
    let [assign] = handler.assigns.as_slice() else {
        return Err("ace.route.push: expected one destination expression".into());
    };
    if handler.body_foreign {
        return Err("ace.route.push: expected one destination expression".into());
    }
    Ok(Some(RouteAction::Push(push_parts(&assign.rhs, widget)?)))
}

fn push_parts(
    expr: &ace_core::source::ast::Expr,
    widget: &ComponentNode,
) -> Result<Vec<RoutePart>, String> {
    use ace_core::source::ast::{BinOp, ExprKind};
    let mut items = vec![expr];
    let mut output = Vec::new();
    while let Some(expr) = items.pop() {
        match &expr.kind {
            ExprKind::StrLit(value) => output.push(RoutePart::Literal(value.clone())),
            ExprKind::Binary {
                op: BinOp::Add,
                lhs,
                rhs,
            } => {
                items.extend([rhs.as_ref(), lhs.as_ref()]);
            }
            ExprKind::Method { recv, name, args } | ExprKind::MethodCall { recv, name, args, .. }
                if name.name == "to_string" && args.is_empty() =>
            {
                let ExprKind::Field { base, name } = &recv.kind else {
                    return Err(
                        "ace.route.push: expected an int model field before .to_string()".into(),
                    );
                };
                let is_model = matches!(&base.kind, ExprKind::Var(owner) if owner.name == "self");
                if !is_model
                    || !widget.fields.iter().any(|field| {
                        field.name == name.name && matches!(field.ty.as_str(), "int" | "i32")
                    })
                {
                    return Err(format!(
                        "ace.route.push: expected int model field {:?}",
                        name.name
                    ));
                }
                output.push(RoutePart::IntField(name.name.clone()));
            }
            _ => {
                return Err("ace.route.push: expected string segments or int model fields converted with .to_string()".into());
            }
        }
    }
    Ok(output)
}
