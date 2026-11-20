//! Shared route admission for the ACE package consumers.

use std::collections::{BTreeMap, BTreeSet};

use ace_av::age_component::project_component_candidates;
use ace_core::source::age_component::ComponentNode;
use ace_core::source::packages::PackageModuleInput;

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
                let page = resolve_page(units, entry, &module).map_err(|reason| {
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
    let components = project_component_candidates(&target.source).map_err(|diagnostics| {
        let message = diagnostics
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{id}: projection failed: {message}")
    })?;
    if !components
        .first()
        .is_some_and(|component| component.view.exits.iter().any(|exit| exit.kind == "view"))
    {
        return Err(format!("{id}: module has no page widget with a view"));
    }
    Ok(id)
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
        let candidates = project_component_candidates(&shell.source).unwrap();
        RouteTable::from_root(&candidates[0], units, shell)
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
                "broken",
                include_str!("../../tests/fixtures/plan124/broken.at"),
            ),
        ];
        let table = routes(&units);
        assert_eq!(table.routes().len(), 5);
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
        let candidates = project_component_candidates(&page.source).unwrap();
        assert!(RouteTable::from_root(&candidates[0], &units, page)
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

    let candidates = project_component_candidates(source).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join(", ")
    })?;
    let widget = candidates
        .first()
        .ok_or_else(|| "page widget is missing".to_string())?;
    let mut replacements = Vec::new();
    let mut commands = BTreeMap::new();
    let mut web_params = BTreeMap::new();
    let mut initialize = false;
    for handler in &widget.handlers {
        initialize |= handler.msg == "Init";
        let span = handler.span.start as usize..handler.span.end as usize;
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
                    | ExprKind::MethodCall { recv, name, args }
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
                        replacements
                            .push((expr.span.start as usize..expr.span.end as usize, literal));
                    }
                    ExprKind::Binary { lhs, rhs, .. } => {
                        expressions.extend([lhs.as_ref(), rhs.as_ref()])
                    }
                    ExprKind::Unary { operand, .. } => expressions.push(operand),
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
        let start = first_field.span.start as usize;
        let mut declarations = String::new();
        for (field, (_, kind)) in &web_params {
            let initial = if kind == "int" { "0" } else { "\"\"" };
            declarations.push_str(&format!("var {field} {kind} = {initial}\n"));
        }
        replacements.push((start..start, declarations));
    }
    let mut rewritten = source.to_string();
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    for (range, replacement) in replacements {
        rewritten.replace_range(range, &replacement);
    }
    Ok(RouteSource {
        source: rewritten,
        commands,
        has_init: initialize,
        web_params,
    })
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
            ExprKind::Method { recv, name, args } | ExprKind::MethodCall { recv, name, args }
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
