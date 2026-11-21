use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ace_core::source::age_component::{ComponentEntryValue, ComponentNode, ComponentViewItem};
use ace_core::source::ast::{Expr, ExprKind};
use ace_core::source::packages::PackageModuleInput;

use super::ace_node_desktop::NodeDesktopBridge;
use super::ace_route::{RouteAction, RouteModulePaths, RouteNavigation, RouteTable};
use super::ace_widget_packages::append_widget_packages;

pub(super) struct DesktopRoutes {
    table: RouteTable,
    units: Vec<PackageModuleInput>,
    paths: RouteModulePaths,
    history: Vec<String>,
    current: Option<ActivePage>,
    transitions: Vec<serde_json::Value>,
}

struct ActivePage {
    path: String,
    content: Result<Box<NodeDesktopBridge>, String>,
}

impl DesktopRoutes {
    pub fn new(table: RouteTable, units: Vec<PackageModuleInput>, paths: RouteModulePaths) -> Self {
        Self {
            table,
            units,
            paths,
            history: Vec::new(),
            current: None,
            transitions: Vec::new(),
        }
    }

    pub fn navigate(&mut self, action: RouteNavigation, instances: usize) -> Result<(), String> {
        let path = match action {
            RouteNavigation::Push(path) => {
                if let Some(current) = &self.current {
                    if current.path == path {
                        return Ok(());
                    }
                    self.history.push(current.path.clone());
                }
                path
            }
            RouteNavigation::Back => match self.history.pop() {
                Some(path) => path,
                None => return Ok(()),
            },
        };
        self.mount(path, instances)
    }

    fn mount(&mut self, path: String, instances: usize) -> Result<(), String> {
        self.close_page()?;
        let content = self.load_page(&path, instances).map(Box::new);
        self.transitions.push(serde_json::json!({
            "path": path,
            "session": content.as_ref().ok().and_then(|page| page.session),
            "error": content.as_ref().err(),
        }));
        self.current = Some(ActivePage { path, content });
        Ok(())
    }

    fn load_page(&self, path: &str, instances: usize) -> Result<NodeDesktopBridge, String> {
        let (route, params) = self.table.resolve(path)?;
        let entry = route.page.as_ref().map_err(Clone::clone)?;
        let result = (|| {
            let mut units = self.table.entry_units(&self.units, entry);
            let unit = units
                .iter_mut()
                .find(|unit| format!("{}::{}", unit.package, unit.id) == *entry)
                .ok_or_else(|| format!("resolved module {entry:?} disappeared"))?;
            let prepared = super::ace_route::prepare_page(&unit.source, Some(&params))?;
            unit.source = prepared.source;
            let nodes = super::ace_route::package_candidates(&unit.source)?;
            let root = nodes.first().ok_or("prepared page widget is missing")?;
            let links = route_links(root)?;
            if let Some(source_path) = self.paths.get(entry) {
                append_widget_packages(source_path, &nodes, &mut units)?;
            }
            let mut bridge = NodeDesktopBridge::load_package_leaf(units, entry, instances)?;
            bridge.set_route_controls(prepared.commands, links);
            bridge.open()?;
            if prepared.has_init && !bridge.has_checked_package_view() {
                if let Err(err) = bridge.initialize_route_page() {
                    let _ = bridge.close_for_shutdown();
                    return Err(err);
                }
            }
            Ok(bridge)
        })();
        result.map_err(|reason: String| {
            format!(
                "ace.route.page: path {path:?} module {:?} ({entry}): {reason}",
                route.module
            )
        })
    }

    fn close_page(&mut self) -> Result<(), String> {
        if let Some(ActivePage {
            content: Ok(mut page),
            ..
        }) = self.current.take()
        {
            page.close_for_shutdown()?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.history.clear();
        self.close_page()
    }

    pub fn page_mut(&mut self) -> Option<&mut NodeDesktopBridge> {
        self.current.as_mut()?.content.as_deref_mut().ok()
    }

    pub fn outlet_after(&self) -> usize {
        self.table.outlet_after()
    }

    pub fn page_session(&self) -> Option<i32> {
        self.current.as_ref()?.content.as_ref().ok()?.session
    }

    pub fn error(&self) -> Option<&str> {
        self.current
            .as_ref()?
            .content
            .as_ref()
            .err()
            .map(String::as_str)
    }

    pub fn evidence(&self) -> serde_json::Value {
        serde_json::json!({
            "current_path": self.current.as_ref().map(|page| &page.path),
            "history": self.history,
            "transitions": self.transitions,
            "page": self.current.as_ref().and_then(|page| page.content.as_ref().ok()).map(|page| page.evidence()),
            "error": self.error(),
        })
    }
}

pub(super) fn front_route_modules(app: &Path) -> Result<Vec<PathBuf>, String> {
    let front = app.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let pages = front.join("pages");
    let mut pending = vec![(front.to_path_buf(), false)];
    if pages.is_dir() {
        pending.push((pages, true));
    }
    let mut modules = Vec::new();
    while let Some((dir, recursive)) = pending.pop() {
        let error = |err| format!("ace.route.module: {}: {err}", dir.display());
        for entry in std::fs::read_dir(&dir).map_err(error)? {
            let entry = entry.map_err(error)?;
            let path = entry.path();
            if recursive && entry.file_type().map_err(error)?.is_dir() {
                pending.push((path, true));
            } else if path != app
                && path.extension().is_some_and(|ext| ext == "at")
                && path.file_name().is_some_and(|name| name != "pac.at" && (recursive || name != "app.at"))
            {
                modules.push(path);
            }
        }
    }
    modules.sort();
    Ok(modules)
}

pub(super) fn route_links(node: &ComponentNode) -> Result<Vec<(String, String)>, String> {
    let mut links = Vec::new();
    let mut items: Vec<&ComponentViewItem> = node.view.tree.iter().rev().collect();
    while let Some(item) = items.pop() {
        match item {
            ComponentViewItem::Element(element) => {
                if matches!(element.tag.as_str(), "link" | "sidebar_menu_button") {
                    if let Some(to) = element.items.iter().find_map(|item| match item {
                        ComponentViewItem::Entry(entry) if entry.key == "to" => Some(&entry.value),
                        _ => None,
                    }) {
                        let ComponentEntryValue::Expr(Expr {
                            kind: ExprKind::StrLit(path),
                            ..
                        }) = to
                        else {
                            return Err(
                                "ace.route.link: desktop link target must be a string literal"
                                    .into(),
                            );
                        };
                        let label = element
                            .label
                            .as_ref()
                            .or_else(|| {
                                element.items.iter().find_map(|item| match item {
                                    ComponentViewItem::Text(leaf) => Some(&leaf.expr),
                                    ComponentViewItem::Element(child) if child.tag == "text" => {
                                        child.label.as_ref()
                                    }
                                    _ => None,
                                })
                            })
                            .and_then(|expr| match &expr.kind {
                                ExprKind::StrLit(text) => Some(text.as_str()),
                                _ => None,
                            })
                            .unwrap_or(path);
                        links.push((label.to_string(), path.clone()));
                    }
                }
                items.extend(element.items.iter().rev());
            }
            ComponentViewItem::Cond(condition) => {
                items.extend(condition.else_items.iter().rev());
                items.extend(condition.then_items.iter().rev());
            }
            ComponentViewItem::For(_)
            | ComponentViewItem::Text(_)
            | ComponentViewItem::Entry(_) => {}
        }
    }
    Ok(links)
}

pub(super) type RouteHandlers = BTreeMap<String, RouteAction>;
