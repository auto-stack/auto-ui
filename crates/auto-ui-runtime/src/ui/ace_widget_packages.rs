//! Resolve authored component packages before passing their source modules to AGE.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
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
use auto_lang_core::use_scanner::{scan_use_statements, UseStatement};
use auto_ui_dialect::ast::ExtImportKind;
use auto_ui_compiler::{
    aura::AuraNode,
    ui_gen::widget::{ComponentRegistry, LoadedPackage},
};

/// Prepare legacy widget modules for the checked AGE package compiler.
pub(super) fn normalize_package_module_placeholders(src: &str) -> String {
    let normalized = normalize_legacy_json_import(normalize_event_placeholders(src, true));
    // PLAN-118：依赖包模块的 canvas 场景合成名与 str/Str 注解同样归一
    // （app 走 prep 同一 pass；088 包的模块 fn 以 `str` 声明返回/局部）。
    let normalized = crate::ui::ace_source_prep::normalize_package_text_surface(&normalized);
    crate::ui::ace_source_prep::normalize_package_scene_bindings(&normalized)
}

/// PLAN-118：VM 轨组件包的模块 `fn` 不写 `pub`（VM 包面不区分可见性），而
/// AGE 包面要求跨模块导入符号必须 public（`source.module-private`）。装配依赖
/// 包模块时做**可见性提升**：只改顶层 `fn` 声明为 `pub fn`，不动嵌套 fn、
/// 不动 `pub fn`、不动函数体——语义零变化，只是让既有跨模块导入可绑定。
/// 仅对依赖包模块源使用；app 源不经过本函数。
pub(super) fn promote_module_fns_to_public(src: &str) -> String {
    if !src.lines().any(|line| line.starts_with("fn ")) {
        return src.to_string();
    }
    let mut out = String::with_capacity(src.len() + 64);
    for line in src.split_inclusive('\n') {
        if line.starts_with("fn ") {
            out.push_str("pub ");
        }
        out.push_str(line);
    }
    out
}

pub(super) fn normalize_legacy_json_import(mut source: String) -> String {
    if !source.contains("auto.json") {
        return source;
    }
    let Ok(syntax) = super::ace_route::package_syntax(&source) else {
        return source;
    };
    // AGE resolves the VM's bare JSON namespace through its checked value
    // primitives. An import of std.json would shadow those with text helpers.
    for import in syntax.imports {
        if import.path == ["auto", "json"] && import.alias.is_none() && !import.reexport {
            let span = import.span.start..import.span.end;
            let blank: String = source[span.clone()]
                .bytes()
                .map(|byte| match byte {
                    b'\r' => '\r',
                    b'\n' => '\n',
                    _ => ' ',
                })
                .collect();
            source.replace_range(span, &blank);
        }
    }
    source
}

pub(super) fn normalize_event_placeholders(src: &str, preserve_child_event: bool) -> String {
    let mut out = String::with_capacity(src.len());
    let mut cursor = 0;
    for (start, marker) in src.match_indices("$event") {
        let end = start + marker.len();
        out.push_str(&src[cursor..start]);
        if preserve_child_event && src[end..].trim_start().starts_with(')') {
            out.push_str(marker);
        } else {
            out.push_str("nevent");
        }
        cursor = end;
    }
    out.push_str(&src[cursor..]);
    let chars: Vec<char> = out.chars().collect();
    let mut result = String::with_capacity(out.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            result.push('n');
            result.push(chars[i + 1]);
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

pub(super) fn append_widget_packages(
    source_path: &Path,
    source: &str,
    candidates: &[ComponentNode],
    units: &mut Vec<PackageModuleInput>,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let parent = source_path.parent().unwrap_or_else(|| Path::new("."));
    let imports = active_uses(source)?;
    let mut registry = ComponentRegistry::new();
    let mut source_paths = BTreeMap::new();
    let mut pending = VecDeque::new();
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
            pending.push_back(PackageRequest {
                name: import.package.name.clone(),
                directory: parent.join(&import.directory),
                tags: tags.clone(),
                imports: imports.clone(),
            });
        }
    }
    while let Some(request) = pending.pop_front() {
        let name = &request.name;
        let package = registry
            .load_package(&request.directory, Path::new("."))
            .map_err(|error| format!("ace.widget.package: {name}: {error}"))?;
        if package.manifest.name != *name && package.manifest.name.replace('-', "_") != *name {
            return Err(format!(
                "ace.widget.package-identity: requested `{name}` from {:?}, found `{}`",
                package.dir, package.manifest.name
            ));
        }
        append_package_sources(package, &request, units, &mut source_paths, &mut pending)?;
    }
    if !source_paths.is_empty() {
        append_project_dependencies(parent, units)?;
    }
    Ok(source_paths)
}

struct PackageRequest {
    name: String,
    directory: PathBuf,
    tags: BTreeSet<String>,
    imports: Vec<UseStatement>,
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
            let source = promote_module_fns_to_public(&normalize_package_module_placeholders(&source));
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
    request: &PackageRequest,
    units: &mut Vec<PackageModuleInput>,
    source_paths: &mut BTreeMap<String, PathBuf>,
    nested: &mut VecDeque<PackageRequest>,
) -> Result<(), String> {
    let import_name = &request.name;
    let mut pending = BTreeSet::new();
    enqueue_widgets(package, &request.tags, &mut pending)?;
    enqueue_modules(package, import_name, &request.imports, false, &mut pending);
    if pending.is_empty() && !units.iter().any(|unit| unit.package == *import_name) {
        // The manifest has been resolved and checked even when the selected
        // view has no active references to this package's source modules.
        units.push(PackageModuleInput::component_manifest(
            import_name.clone(), package.manifest.version.clone(),
        ));
    }
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
        let key = format!("{import_name}::{id}");
        if let Some(previous) = source_paths.get(&key) {
            if previous != &canonical {
                return Err(format!("ace.widget.module-identity: conflicting paths for `{key}`"));
            }
            continue;
        }
        let source = std::fs::read_to_string(&canonical)
            .map_err(|error| format!("ace.widget.module: {}: {error}", path.display()))?;
        let source = promote_module_fns_to_public(&normalize_package_module_placeholders(&source));
        source_paths.insert(key, canonical);
        let imports = active_uses(&source)?;
        for (decl, widget) in &package.full_widgets {
            if package.widget_sources[&widget.name].contains(&path) {
                let mut child_tags = BTreeSet::new();
                collect_aura_tags(&widget.view_tree, &mut child_tags);
                for (_, view) in &widget.named_views {
                    collect_aura_tags(view, &mut child_tags);
                }
                enqueue_widgets(package, &child_tags, &mut pending)?;
                for import in decl.ext_imports.iter().filter(|import| matches!(import.kind, ExtImportKind::Package)) {
                    for name in &import.symbols {
                        nested.push_back(PackageRequest {
                            name: name.as_str().to_owned(),
                            directory: package.dir.join(import.path.as_str()),
                            tags: child_tags.clone(),
                            imports: imports.clone(),
                        });
                    }
                }
            }
        }
        enqueue_modules(package, import_name, &imports, true, &mut pending);
        source_units.push(PackageModuleInput {
            package: import_name.clone(),
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

fn active_uses(source: &str) -> Result<Vec<UseStatement>, String> {
    let mut imports = scan_use_statements(source);
    if source.contains("Env") && source.contains("track") {
        let syntax = super::ace_route::package_syntax(source)?;
        // Use the checked compiler's reachability after host guards have been
        // specialized. Leave namespace and wildcard imports to bind normally.
        imports.retain(|import| {
            import.is_pub || import.is_wildcard || import.items.is_empty()
                || import.items.iter().any(|item| {
                    !item.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                        || syntax.referenced.contains(item)
                })
        });
    }
    Ok(imports)
}

fn enqueue_modules(
    package: &LoadedPackage,
    import_name: &str,
    imports: &[UseStatement],
    from_inside_package: bool,
    pending: &mut BTreeSet<PathBuf>,
) {
    let directory_name = package.dir.file_name().and_then(|name| name.to_str());
    for import in imports {
        let module = match import.module.split_once('.') {
            Some((prefix, module)) if prefix == import_name
                || prefix == package.manifest.name
                || Some(prefix) == directory_name
                || (from_inside_package && prefix.is_empty()) => module,
            None if from_inside_package => &import.module,
            _ => continue,
        };
        if module.is_empty() || !module.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
            continue;
        }
        let dependency = package.dir.join(format!("{module}.at"));
        if dependency.is_file() {
            pending.insert(dependency);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_widget_modules_use_age_json_values() {
        use ace_core::source::{
            age_component::ScenarioOp, age_h0::LegacyWidgetAlias,
            age_package::compile_age_package_with_scenario,
        };

        let authored = r#"use auto.json
widget App {
    msg { Load }
    model { var ready bool = false }
    on {
        .Load -> {
            let parsed = json.parse("{\"ok\":true}")
            .ready = parsed.ok ?? false
        }
    }
    view { text "use auto.json" {} }
}"#;
        let source = normalize_package_module_placeholders(authored);
        assert_eq!(source.len(), authored.len());
        assert_eq!(source.split_once('\n').unwrap().1, authored.split_once('\n').unwrap().1);
        let unit = PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source,
            provider: None,
        };
        let scenario = ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send { instance: 0, msg: "Load".into() }],
        };
        compile_age_package_with_scenario(
            &[unit], "app::main", LegacyWidgetAlias::Accept,
            &[PACKAGES_CAPABILITY.to_string()], &scenario,
        ).expect("legacy JSON value calls should resolve in a checked package");

        let stdlib = authored.replace("use auto.json\n", "use std.json\n");
        assert_eq!(normalize_package_module_placeholders(&stdlib), stdlib);
    }

    #[test]
    fn tree_view_checked_helpers_preserve_unicode_ids_and_navigation() {
        use ace_core::source::{
            age_component::ComponentScenario,
            age_h0::LegacyWidgetAlias,
            age_package::compile_age_package_with_scenario,
        };

        let app = r#"
use tree_view.tree_view_util: tv_duplicate_id, tv_count_rows, tv_derive_rows, tv_focus_id, tv_visible_parent, tv_struct_parent, tv_repair_focus
fn verdict() Str {
    var nodes List = []
    nodes.push({id: "田|🌅!", children: []})
    nodes.push({id: "田!1🌅!0", children: []})
    nodes.push({id: "🌅", children: []})
    if tv_duplicate_id(nodes) != "" { return "distinct IDs were rejected" }
    nodes.push({id: "田|🌅!", children: []})
    return tv_duplicate_id(nodes)
}
fn navigation() Str {
    var children List = []
    children.push({id: "照片|🌅", label: "照片.png", children: [], kind: "file", icon: "", is_leaf: true, badge: ""})
    var nodes List = []
    nodes.push({id: "root", label: "Root", children: children, kind: "dir", icon: "", is_leaf: false, badge: ""})
    var expanded List = []
    expanded.push("root")
    var empty List = []
    if tv_count_rows(nodes, expanded, empty) != 2 { return "incorrect reachable count" }
    var rows List = tv_derive_rows(nodes, expanded, "照片|🌅", empty, empty, empty, 1, true, false)
    if rows.len() != 2 { return "incorrect row count" }
    let parent_empty bool = rows[0].empty
    let child_empty bool = rows[1].empty
    let child_depth int = rows[1].depth
    if parent_empty || child_empty || child_depth != 1 { return "incorrect ordinary row shape" }
    let empty_rows List = tv_derive_rows(empty, empty, "", empty, empty, empty, -1, true, false)
    if empty_rows.len() != 1 { return "missing empty row" }
    let sentinel_empty bool = empty_rows[0].empty
    let sentinel_depth int = empty_rows[0].depth
    if !sentinel_empty || sentinel_depth != 0 { return "incorrect empty row shape" }
    if tv_repair_focus(rows, nodes, expanded, "照片|🌅", 0) != 1 { return "incorrect focus" }
    return tv_focus_id(rows, 1) + "/" + tv_visible_parent(nodes, expanded, "照片|🌅") + "/" + tv_struct_parent(nodes, "照片|🌅")
}
widget App {
    view {
        text verdict() {}
        text navigation() {}
    }
}
"#;
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/tree-view/tree_view_util.at");
        let units = [
            PackageModuleInput {
                package: "app".into(), version: "1.0.0".into(), id: "app".into(),
                source: app.into(), provider: None,
            },
            PackageModuleInput {
                package: "tree_view".into(), version: "0.1.0".into(),
                id: "tree_view_util".into(), provider: None,
                source: std::fs::read_to_string(path).expect("authored TreeView utility"),
            },
        ];
        let checked = compile_age_package_with_scenario(
            &units, "app::app", LegacyWidgetAlias::Accept,
            &[PACKAGES_CAPABILITY.to_string()],
            &ComponentScenario { instances: 1, ops: Vec::new() },
        ).expect("bind the authored TreeView helpers");
        let observed = ace_av::age_package::observe_age_package(&checked, 1)
            .expect("execute TreeView duplicate detection through AV");
        assert_eq!(observed.components[0].instance_texts, [vec!["田|🌅!", "照片|🌅/root/root"]]);
    }

    #[test]
    fn asset_browser_checked_record_validation_preserves_exact_ids() {
        use ace_core::source::{
            age_component::ComponentScenario,
            age_h0::LegacyWidgetAlias,
            age_package::compile_age_package_with_scenario,
        };

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages");
        let app = r#"
use asset_browser.asset_browser_util: ab_check_records, ab_find_index
fn verdict() Str {
    var records List = []
    records.push({ id: "a|b!日", revision: "r1" })
    records.push({ id: "a!1b!0日", revision: "r1" })
    if ab_check_records(records) != "" { return "distinct IDs were rejected" }
    if ab_find_index(records, "a!1b!0日") != 1 { return "incorrect index" }
    if ab_find_index(records, "missing") != -1 { return "unexpected index" }
    records.push({ id: "a|b!日", revision: "r2" })
    return ab_check_records(records).to(Str)
}
widget App { view { text verdict() {} } }
"#;
        let units = [
            PackageModuleInput {
                package: "app".into(), version: "1.0.0".into(), id: "app".into(),
                source: app.into(), provider: None,
            },
            PackageModuleInput {
                package: "asset_browser".into(), version: "0.1.0".into(),
                id: "asset_browser_util".into(),
                source: std::fs::read_to_string(root.join("asset-browser/asset_browser_util.at"))
                    .expect("authored AssetBrowser utility"),
                provider: None,
            },
            PackageModuleInput {
                package: "collection_view".into(), version: "0.1.0".into(),
                id: "collection_schema".into(),
                source: std::fs::read_to_string(root.join("collection-view/collection_schema.at"))
                    .expect("authored CollectionView schema"),
                provider: None,
            },
        ];
        let checked = compile_age_package_with_scenario(
            &units, "app::app", LegacyWidgetAlias::Accept,
            &[PACKAGES_CAPABILITY.to_string()],
            &ComponentScenario { instances: 1, ops: Vec::new() },
        ).expect("check the original package modules and record validation fixture");
        let observed = ace_av::age_package::observe_age_package(&checked, 1)
            .expect("execute the package utility through AV");
        assert_eq!(observed.components[0].instance_texts, [vec!["a|b!日"]]);
    }

    #[test]
    fn photo_gallery_loads_asset_browser_dependencies() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/029-photo-gallery");
        let path = root.join("src/front/app.at");
        let source = std::fs::read_to_string(&path).expect("photo gallery source");
        let prepared = crate::ui::ace_source_prep::prepare_component_source(&root, &source)
            .expect("photo gallery source preparation");
        let source = normalize_package_module_placeholders(&prepared.source);
        let candidates = super::super::ace_route::package_candidates(&source)
            .expect("prepared package syntax");
        let mut units = vec![PackageModuleInput {
            package: "app".into(),
            id: "app".into(),
            version: "1.0.0".into(),
            source: source.clone(),
            provider: None,
        }];
        append_widget_packages(&path, &source, &candidates, &mut units)
            .expect("widget package discovery");

        for (package, module) in [
            ("asset_browser", "asset_browser"),
            ("tree_view", "tree_view"),
            ("pagination", "pagination_control"),
            ("collection_view", "collection_view"),
            ("image_viewer", "image_viewport"),
        ] {
            assert_eq!(
                units.iter().filter(|unit| unit.package == package && unit.id == module).count(),
                1,
                "expected one `{package}::{module}` source unit"
            );
        }
    }

    /// PLAN-118：VM 轨组件包的模块 `fn` 装配期提升为 `pub fn`（跨模块导入
    /// 可绑定）；只动顶层声明，嵌套/已 pub 的声明不变。
    #[test]
    fn plan118_promotes_only_top_level_module_fns() {
        let source = "fn helper() int { return 1 }\npub fn shown() int { return 2 }\nwidget W {\n    fn inner() int { return 3 }\n    view { text \"x\" {} }\n}\n";
        let promoted = promote_module_fns_to_public(source);
        assert_eq!(promoted.matches("pub fn helper(").count(), 1);
        assert_eq!(promoted.matches("pub fn shown(").count(), 1);
        assert!(!promoted.contains("pub pub fn"));
        assert!(promoted.contains("    fn inner() int"));
        // 无顶层 fn 的源逐字节不变。
        let plain = "use auto.json\nwidget W { view { text \"x\" {} } }\n";
        assert_eq!(promote_module_fns_to_public(plain), plain);
    }
}