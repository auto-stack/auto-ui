//! Resolve authored component packages before passing their source modules to AGE.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ace_core::source::{
    age_component::{ComponentNode, ComponentScenario, ComponentViewItem},
    age_package::widget_tag_matches,
    ast::ExprKind,
    config_document::{parse_config_document, ConfigLine, ConfigPairValue},
    cst::FrontBudget,
    packages::{parse_package_source_age, PackageModuleInput, PACKAGES_CAPABILITY},
};
use auto_lang_core::{pac_declares_dep, pac_workspace_member_dir, resolve_module_path};
use auto_ui_compiler::{
    aura::AuraNode,
    ui_gen::widget::{ComponentRegistry, LoadedPackage},
};

pub(super) fn append_widget_packages(
    source_path: &Path,
    candidates: &[ComponentNode],
    units: &mut Vec<PackageModuleInput>,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let parent = source_path.parent().unwrap_or_else(|| Path::new("."));
    let mut registry = ComponentRegistry::new();
    let mut source_paths = BTreeMap::new();
    for node in candidates {
        if node.package_imports.is_empty() {
            continue;
        }
        let mut tags = BTreeSet::new();
        collect_tags(&node.view.tree, &mut tags);
        for (_, items) in &node.view.named_trees {
            collect_tags(items, &mut tags);
        }
        for import in &node.package_imports {
            let name = &import.package.name;
            let package = registry
                .load_package(Path::new(&import.directory), parent)
                .map_err(|error| format!("ace.widget.package: {name}: {error}"))?;
            if package.manifest.name != *name {
                return Err(format!(
                    "ace.widget.package-identity: requested `{name}` from {:?}, found `{}`",
                    package.dir, package.manifest.name
                ));
            }
            append_package_sources(package, &tags, units, &mut source_paths)?;
        }
    }
    if !source_paths.is_empty() {
        append_project_dependencies(parent, units)?;
    }
    Ok(source_paths)
}

fn append_project_dependencies(root: &Path, units: &mut Vec<PackageModuleInput>) -> Result<(), String> {
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario { instances: 0, ops: Vec::new() };
    let mut known: BTreeSet<_> = units
        .iter()
        .map(|unit| (unit.package.clone(), unit.id.clone()))
        .collect();
    let mut roots = vec![root.to_path_buf(); units.len()];
    let mut index = 0;
    while index < units.len() {
        let base = roots[index].clone();
        let syntax = parse_package_source_age(
            &units[index].source, &caps, FrontBudget::WORK_OWNED_64MIB, &scenario, 0,
        );
        index += 1;
        let Ok(syntax) = syntax else {
            // The package compiler owns any source diagnostics, including
            // deferred failures in an otherwise usable style module.
            continue;
        };
        for import in &syntax.imports {
            if !import.reexport && !syntax.referenced.contains(import.bound_name()) {
                continue;
            }
            let [package, module, _] = import.path.as_slice() else {
                continue;
            };
            let key = (package.clone(), module.clone());
            if known.contains(&key) {
                continue;
            }
            let declared = base.ancestors().any(|parent| {
                std::fs::read_to_string(parent.join("pac.at")).is_ok_and(|manifest| {
                    pac_declares_dep(&manifest, package)
                        || pac_workspace_member_dir(&manifest, package).is_some()
                })
            });
            if !declared {
                continue;
            }
            let Some(path) = resolve_module_path(&base, &format!("{package}.{module}")) else {
                continue;
            };
            let source = std::fs::read_to_string(&path)
                .map_err(|error| format!("ace.widget.project-dependency: {}: {error}", path.display()))?;
            let dir = path.parent().expect("resolved module has a directory");
            let version = project_package_version(dir).ok_or_else(|| {
                format!("ace.widget.project-dependency: no version in the manifest for `{package}::{module}`")
            })?;
            known.insert(key);
            roots.push(dir.to_path_buf());
            units.push(PackageModuleInput {
                package: package.clone(),
                version,
                id: module.clone(),
                source,
                provider: None,
            });
        }
    }
    Ok(())
}

fn project_package_version(dir: &Path) -> Option<String> {
    let manifest = dir
        .ancestors()
        .find_map(|path| std::fs::read_to_string(path.join("pac.at")).ok())?;
    parse_config_document(&manifest).ok()?.lines.into_iter().find_map(|line| {
        let ConfigLine::Pair(pair) = line else { return None };
        if pair.key.name != "version" { return None; }
        let ConfigPairValue::Expr(expr) = pair.value else { return None };
        let ExprKind::StrLit(version) = &expr.kind else { return None };
        Some(version.clone())
    })
}

fn collect_tags(items: &[ComponentViewItem], tags: &mut BTreeSet<String>) {
    let mut pending: Vec<_> = items.iter().collect();
    while let Some(item) = pending.pop() {
        match item {
            ComponentViewItem::Element(element) => {
                tags.insert(element.tag.clone());
                pending.extend(&element.items);
            }
            ComponentViewItem::For(iteration) => pending.extend(&iteration.items),
            ComponentViewItem::Cond(cond) => {
                pending.extend(&cond.then_items);
                pending.extend(&cond.else_items);
            }
            ComponentViewItem::Text(_) | ComponentViewItem::Entry(_) => {}
        }
    }
}

fn collect_aura_tags(view: &AuraNode, tags: &mut BTreeSet<String>) {
    let mut pending = vec![view];
    while let Some(item) = pending.pop() {
        match item {
            AuraNode::Element { tag, children, .. }
            | AuraNode::Component {
                name: tag,
                children,
                ..
            } => {
                tags.insert(tag.clone());
                pending.extend(children);
            }
            AuraNode::ForLoop { body, .. } | AuraNode::MemoBlock { body, .. } => {
                pending.extend(body)
            }
            AuraNode::Conditional {
                then_body,
                else_body,
                ..
            } => {
                pending.extend(then_body);
                pending.extend(else_body.iter().flatten());
            }
            AuraNode::Link { children, .. } => pending.extend(children),
            AuraNode::Text(_) | AuraNode::Outlet { .. } => {}
        }
    }
}

fn append_package_sources(
    package: &LoadedPackage,
    tags: &BTreeSet<String>,
    units: &mut Vec<PackageModuleInput>,
    source_paths: &mut BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    let mut pending = BTreeSet::new();
    enqueue_widgets(package, tags, &mut pending)?;
    let mut resolved = BTreeSet::new();
    let mut source_units = Vec::new();
    while let Some(path) = pending.pop_first() {
        if !resolved.insert(path.clone()) {
            continue;
        }
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("ace.widget.module: {}: {error}", path.display()))?;
        if canonical.parent() != Some(package.dir.as_path()) {
            return Err(format!(
                "ace.widget.module: {} leaves the component package",
                path.display()
            ));
        }
        let id = path
            .file_stem()
            .and_then(|id| id.to_str())
            .ok_or_else(|| format!("ace.widget.module: invalid module path {}", path.display()))?;
        let source = std::fs::read_to_string(&canonical)
            .map_err(|error| format!("ace.widget.module: {}: {error}", path.display()))?;
        source_paths.entry(format!("{}::{id}", package.manifest.name)).or_insert(canonical);
        let mut child_tags = BTreeSet::new();
        for (_, widget) in &package.full_widgets {
            if package.widget_sources[&widget.name].contains(&path) {
                collect_aura_tags(&widget.view_tree, &mut child_tags);
                for (_, view) in &widget.named_views {
                    collect_aura_tags(view, &mut child_tags);
                }
            }
        }
        enqueue_widgets(package, &child_tags, &mut pending)?;
        for import in auto_lang_core::use_scanner::scan_use_statements(&source) {
            // The gallery uses both `use tree_util` and `use components.tree_util`.
            // Keep local helpers by source identity. External imports belong to
            // the project's ordinary dependency resolver.
            let qualifier = package
                .dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let (prefix, module) = import
                .module
                .split_once('.')
                .unwrap_or(("", &import.module));
            if (prefix.is_empty() || prefix == qualifier || prefix == package.manifest.name)
                && module
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                let dependency = package.dir.join(format!("{module}.at"));
                if dependency.is_file() {
                    pending.insert(dependency);
                }
            }
        }
        source_units.push(PackageModuleInput {
            package: package.manifest.name.clone(),
            version: package.manifest.version.clone(),
            id: id.to_string(),
            source,
            provider: None,
        });
    }
    for unit in source_units {
        if let Some(existing) = units
            .iter()
            .find(|existing| existing.package == unit.package && existing.id == unit.id)
        {
            if existing.version != unit.version || existing.source != unit.source {
                return Err(format!(
                    "ace.widget.module-identity: conflicting source for `{}::{}`",
                    unit.package, unit.id
                ));
            }
        } else {
            units.push(unit);
        }
    }
    Ok(())
}

fn enqueue_widgets(
    package: &LoadedPackage,
    tags: &BTreeSet<String>,
    pending: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    for tag in tags {
        for (name, sources) in package.widget_sources.iter()
            .filter(|(name, _)| widget_tag_matches(tag, name))
        {
            if sources.len() != 1 {
                return Err(format!(
                    "ace.widget.ambiguous: `{name}` has {} declarations in `{}`",
                    sources.len(),
                    package.manifest.name
                ));
            }
            pending.insert(sources[0].clone());
        }
    }
    Ok(())
}
