use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use ace_a2x_core::{RecordFacts, SemanticView, Ty};
use ace_a2x_ts::seed::{decode_coll_seed, encode_coll_seed, render_coll_seed_helpers, SeedCollTy};
use ace_a2x_ts::{
    emit_function_body_fragment, script_widget_runtime_body, ts_fn_name, FragmentProgramContext,
    ScriptLang,
};
use ace_core::source::age_component::ComponentViewItem;
use ace_core::source::age_package::{
    compile_age_package_widget_runtime, AgePackageCompilation, AgeWidgetProperty, AgeWidgetWatchSource,
    WIDGET_FOCUS_FIELD, WIDGET_OUTBOUND_FIELD,
};
use ace_core::source::packages::PackageModuleInput;

use crate::ui::ace_vue_script::channel_pascal;
use crate::ui::ace_widget_props::literal;
use crate::ui::ace_package_bridge::file_directory::FileDirectoryBinding;
use crate::ui::ace_package_bridge::file_mutation::FileMutationBinding;
use crate::ui::ace_package_bridge::file_open::{FileOpenBinding, UNAVAILABLE};
use ace_core::hir::ExprKind;
use crate::ui::ace_svg;
use super::PackageSfc;

pub(super) fn generate(
    units: &[PackageModuleInput],
    entry: &str,
    caps: &[String],
    routed: bool,
    shell: bool,
) -> Result<PackageSfc, String> {
    let compiled = compile_age_package_widget_runtime(
        units,
        entry,
        ace_av::age_component::LegacyWidgetAlias::Accept,
        caps,
    )
    .map_err(|errors| {
        errors
            .into_iter()
            .map(|err| format!("{}: {}", err.code, err.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let view = SemanticView::from_checked_age_package(&compiled.checked)
        .map_err(|err| format!("source.age.sfc-widget-view: {err:?}"))?;
    let root = compiled
        .widget_runtime
        .first()
        .ok_or("missing checked widget root")?;
    let root_index = compiled
        .widget_runtime
        .iter()
        .find(|runtime| compiled.description.components[runtime.node].module == entry)
        .map_or(root.node, |runtime| runtime.node);
    let file_directory = directory_binding(&compiled, &view, root_index)?;
    let file_mutation = FileMutationBinding::from_component(&compiled.description.components[root_index])?;
    let file_open = FileOpenBinding::from_component(&compiled.description.components[root_index])?;
    let preferences = crate::ui::ace_package_bridge::preferences::checked_bindings(
        &compiled.description.components[root_index],
    )?;

    let icons = icon_imports(&compiled);
    let mut script = String::from("import { Fragment, h, nextTick, onBeforeUnmount, onMounted, shallowRef } from 'vue'\n");
    if file_directory.is_some() || file_mutation.is_some() {
        let mut imports = Vec::new();
        if file_directory.is_some() { imports.push("readFileDirectory"); }
        if file_mutation.is_some() { imports.push("applyFileMutation"); }
        imports.push("fileBrowserSignal");
        writeln!(script, "import {{ {} }} from '@/lib/file-browser'", imports.join(", ")).unwrap();
    }
    if shell {
        script.push_str("import { RouterLink, RouterView } from 'vue-router'\n");
    } else if routed {
        script.push_str("import { RouterLink } from 'vue-router'\n");
    }
    if !icons.is_empty() {
        writeln!(
            script,
            "import {{ {} }} from 'lucide-vue-next'",
            icons.values().cloned().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>().join(", ")
        )
        .unwrap();
    }
    script.push_str("const AcePackageView = (() => {\n");
    let routing = if shell {
        "{ link: RouterLink, outlet: RouterView }"
    } else if routed {
        "{ link: RouterLink, outlet: null }"
    } else {
        "null"
    };
    writeln!(script, "const aceRouting = {routing};").unwrap();
    let svg_tags: Vec<_> = ace_svg::tags().collect();
    writeln!(
        script,
        "const aceSvgTags = new Set({});\nconst aceSvgProperties = new Set({});\nconst aceSvgNamespace = {};",
        serde_json::to_string(&svg_tags).unwrap(),
        serde_json::to_string(ace_svg::PROPERTIES).unwrap(),
        quoted(ace_svg::NAMESPACE),
    )
    .unwrap();
    script.push_str(&script_widget_runtime_body());
    script.push('\n');
    let seed = encode_coll_seed(&view)
        .and_then(|seed| decode_coll_seed(&seed))
        .map_err(|err| format!("source.age.sfc-widget-types: {err:?}"))?;
    for line in render_coll_seed_helpers(&seed, false).lines() {
        writeln!(script, "{}", line.strip_prefix("export ").unwrap_or(line)).unwrap();
    }
    let mut roots = write_components(&compiled, &mut script)?;
    roots.extend(view.global_init_fns().into_iter().map(str::to_owned));
    let reachable = widget_functions(&view, &roots)?;
    let context = FragmentProgramContext::from_view(&view);
    for function in view.functions().iter().filter(|function| reachable.contains(&function.def_index)) {
        let code = emit_function_body_fragment(ScriptLang::JavaScript, &view, function, &context)
            .map_err(|err| {
            format!(
                "source.age.sfc-widget-body: {}: {err:?}",
                function.source_name
            )
        })?;
        let params = code
            .locals
            .iter()
            .filter(|local| local.role == "param")
            .map(|local| local.emitted_name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            script,
            "function {}({params}) {{\n{}\n}}",
            ts_fn_name(&function.source_name),
            code.text.join("\n")
        )
        .unwrap();
    }
    for initializer in view.global_init_fns() {
        writeln!(script, "{}();", ts_fn_name(initializer)).unwrap();
    }
    script.push_str("const aceIcons = {\n");
    for (name, symbol) in icons {
        writeln!(script, "  {}: {symbol},", quoted(&name)).unwrap();
    }
    script.push_str("};\n");
    script.push_str(super::POINTER_SAMPLE);
    script.push_str(include_str!("checked_widget_view.js"));
    let file_directory = file_directory.map_or_else(|| "null".into(), |(reply, entry_tag, include_extension)| format!(
        "{{ read: readFileDirectory, lease: fileBrowserSignal, reply: {}, entryTag: {}, includeExtension: {include_extension} }}",
        quoted(&reply), quoted(&entry_tag),
    ));
    let file_mutation = file_mutation.map_or_else(|| "null".into(), |binding| format!(
        "{{ apply: applyFileMutation, lease: fileBrowserSignal, reply: {} }}",
        quoted(binding.reply_name()),
    ));
    let file_open = file_open.map_or_else(|| "null".into(), |binding| format!(
        "{{ reply: {}, unavailable: {UNAVAILABLE} }}", quoted(binding.reply_name()),
    ));
    let preferences = preferences.iter().map(|entry| (&entry.key, &entry.field, &entry.restore))
        .collect::<Vec<_>>();
    let preferences = serde_json::to_string(&preferences).map_err(|error| error.to_string())?;
    writeln!(
        script,
        "\nreturn createCheckedWidgetView({root_index}, {file_directory}, {file_mutation}, {preferences}, {file_open});\n}})();"
    )
    .unwrap();
    let source = format!(
        "<template><AcePackageView /></template>\n<script setup>\n{}</script>\n",
        escape_script(&script)
    );
    let active: BTreeSet<_> = compiled.widget_runtime.iter().map(|runtime| runtime.node).collect();
    let other_modules: BTreeSet<_> = compiled.description.components.iter().enumerate()
        .filter_map(|(node, component)| (!active.contains(&node)).then_some(&component.module))
        .collect();
    let embedded_widget_modules = compiled.widget_runtime.iter()
        .map(|runtime| &compiled.description.components[runtime.node].module)
        .filter(|module| !other_modules.contains(module))
        .cloned().collect();
    Ok(PackageSfc { source, uses_runtime: false, embedded_widget_modules })
}

fn directory_binding(
    compiled: &AgePackageCompilation,
    view: &SemanticView<'_>,
    root: usize,
) -> Result<Option<(String, String, bool)>, String> {
    let component = &compiled.description.components[root];
    let Some(binding) = FileDirectoryBinding::from_package(compiled, component)? else { return Ok(None) };
    let reply = binding.reply_name();
    let bad_shape = || "source.age.sfc-file-binding: invalid DirListing reply record".to_string();
    let handler = compiled.widget_runtime.iter().find(|runtime| runtime.node == root)
        .and_then(|runtime| runtime.handlers.iter().find(|handler| handler.message == reply))
        .and_then(|handler| compiled.symbol_of_in(&component.module, &handler.function))
        .and_then(|symbol| view.functions().iter().find(|function| function.source_name == symbol))
        .ok_or_else(bad_shape)?;
    let Some(Ty::Record(listing)) = handler.params.last().map(|param| param.ty) else {
        return Err(bad_shape());
    };
    let listing = view.record(listing).ok_or_else(bad_shape)?;
    let field_type = |record: &RecordFacts, name: &str| {
        record.fields.iter().find(|field| field.name == name).map(|field| field.ty)
    };
    let Some(Ty::List(entries)) = field_type(listing, "entries") else { return Err(bad_shape()) };
    let Ok(Ty::Record(entry)) = view.list_elem(entries) else { return Err(bad_shape()) };
    let entry = view.record(entry).ok_or_else(bad_shape)?;
    if listing.fields.len() != 3 || field_type(listing, "ok") != Some(Ty::Bool)
        || field_type(listing, "path") != Some(Ty::StrOwned)
        || entry.fields.len() != 5 + usize::from(binding.include_extension)
        || (binding.include_extension && field_type(entry, "extension") != Some(Ty::StrOwned))
        || [
            ("name", Ty::StrOwned), ("path", Ty::StrOwned), ("is_dir", Ty::Bool),
            ("size", Ty::U64), ("mtime", Ty::I64),
        ].into_iter().any(|(name, ty)| field_type(entry, name) != Some(ty))
    {
        return Err(bad_shape());
    }
    Ok(Some((reply.to_owned(), SeedCollTy::Record(entry.name.clone()).elem_tag(), binding.include_extension)))
}

fn widget_functions(view: &SemanticView<'_>, roots: &BTreeSet<String>) -> Result<BTreeSet<usize>, String> {
    let by_name: BTreeMap<_, _> = view.functions().iter().map(|f| (f.source_name, f.def_index)).collect();
    let by_index: BTreeMap<_, _> = view.functions().iter().map(|f| (f.def_index, f)).collect();
    let mut pending = roots.iter().map(|name| {
        by_name.get(name.as_str()).copied()
            .ok_or_else(|| format!("source.age.sfc-widget-symbol: no checked body for `{name}`"))
    }).collect::<Result<Vec<_>, _>>()?;
    let mut reachable = BTreeSet::new();
    while let Some(index) = pending.pop() {
        let Some(function) = by_index.get(&index) else { continue };
        if !reachable.insert(index) {
            continue;
        }
        pending.extend(function.body.exprs.iter().filter_map(|expr| match expr.kind {
            ExprKind::Call { callee, .. } | ExprKind::CreateClosure { callee, .. } => Some(callee.0 as usize),
            ExprKind::CallbackRef { def } => Some(def.0 as usize),
            _ => None,
        }));
    }
    Ok(reachable)
}

fn write_components(
    compiled: &AgePackageCompilation,
    script: &mut String,
) -> Result<BTreeSet<String>, String> {
    let mut roots = BTreeSet::new();
    let mut symbol = |module: &str, name: &str| -> Result<String, String> {
        let symbol = compiled
            .symbol_of_in(module, name)
            .ok_or_else(|| format!("source.age.sfc-widget-symbol: {module}::{name}"))?;
        roots.insert(symbol.clone());
        Ok(ts_fn_name(&symbol))
    };
    writeln!(script, "const checkedOutboundField = {};", quoted(WIDGET_OUTBOUND_FIELD)).unwrap();
    writeln!(script, "const checkedFocusField = {};", quoted(WIDGET_FOCUS_FIELD)).unwrap();
    script.push_str("const aceComponents = {\n");
    for runtime in &compiled.widget_runtime {
        let owner = &compiled.description.components[runtime.node];
        let key = (owner.module.clone(), owner.name.clone());
        let init = runtime
            .initialize
            .as_deref()
            .map(|init| symbol(&owner.module, init))
            .transpose()?;
        writeln!(
            script,
            "{}: {{ name: {}, init: {}, view: {}, focus: {}, contextHandler: {}, handlers: {{",
            runtime.node,
            quoted(&owner.name),
            init.as_deref().unwrap_or("null"),
            symbol(&owner.module, &runtime.view)?,
            runtime.focus,
            runtime.context_handler.as_deref().map_or_else(|| "null".into(), quoted),
        )
        .unwrap();
        for handler in &runtime.handlers {
            let payload = owner
                .messages
                .iter()
                .find(|item| item.name == handler.message)
                .map_or(&[][..], |item| item.payload.as_slice());
            writeln!(
                script,
                "[{}]: {{ run: {}, types: {} }},",
                quoted(&handler.message),
                symbol(&owner.module, &handler.function)?,
                serde_json::to_string(payload).unwrap()
            )
            .unwrap();
        }
        script.push_str("}, outbound: {\n");
        if runtime.outbound {
            for message in &owner.messages {
                if !owner.handler_msgs.contains(&message.name) {
                    writeln!(script, "[{}]: {},", quoted(&message.name), serde_json::to_string(&message.payload).unwrap()).unwrap();
                }
            }
        }
        script.push_str("}, watches: [\n");
        for watch in &runtime.watches {
            let sources = watch.sources.iter().map(|source| match source {
                AgeWidgetWatchSource::Property(index) => serde_json::json!(index),
                AgeWidgetWatchSource::Model(index) => serde_json::json!(owner.fields[*index].0),
            }).collect::<Vec<_>>();
            writeln!(
                script,
                "{{ sources: {}, immediate: {}, run: {} }},",
                serde_json::to_string(&sources).unwrap(),
                watch.immediate,
                symbol(&owner.module, &watch.function)?,
            ).unwrap();
        }
        script.push_str("], timers: [\n");
        for timer in &runtime.timers {
            let condition = timer.condition.as_deref()
                .map(|name| symbol(&owner.module, name))
                .transpose()?;
            writeln!(
                script,
                "{{ message: {}, interval: {}, condition: {} }},",
                quoted(&timer.message),
                timer.interval_ms,
                condition.as_deref().unwrap_or("null"),
            )
            .unwrap();
        }
        script.push_str("], sites: {\n");
        for site in compiled.widget_refs.get(&key).into_iter().flatten() {
            let child = compiled
                .widget_runtime
                .iter()
                .find(|candidate| {
                    let node = &compiled.description.components[candidate.node];
                    node.module == site.module && node.name == site.node
                })
                .ok_or("checked widget site has no runtime component")?;
            writeln!(
                script,
                "{}: {{ node: {}, tag: {}, props: [",
                site.span.start, child.node, quoted(&site.authored_node)
            )
            .unwrap();
            let declaration = &compiled.description.components[child.node];
            for prop in &site.properties {
                match prop {
                    AgeWidgetProperty::Value {
                        module,
                        evaluators,
                        authored_default,
                        runtime_evaluator,
                        ..
                    } => {
                        let getter = runtime_evaluator
                            .as_ref()
                            .or_else(|| evaluators.first())
                            .ok_or("checked widget property has no evaluator")?;
                        writeln!(
                            script,
                            "{{ get: {}, defaulted: {authored_default} }},",
                            symbol(module, getter)?
                        )
                        .unwrap();
                    }
                    AgeWidgetProperty::ScopedValue { name, .. } => {
                        let ty = declaration
                            .params
                            .iter()
                            .find(|(param, _)| param == name)
                            .map(|(_, ty)| ty)
                            .ok_or("checked widget property has no type")?;
                        writeln!(
                            script,
                            "{{ name: {}, type: {} }},",
                            quoted(name),
                            quoted(ty)
                        )
                        .unwrap();
                    }
                    AgeWidgetProperty::Message { .. } => {}
                }
            }
            script.push_str("], channels: {\n");
            let mut channels = BTreeMap::new();
            for prop in &site.properties {
                if let AgeWidgetProperty::Message { name, target, .. } = prop {
                    channels.insert(channel_pascal(name), target);
                }
            }
            for (message, channel) in &declaration.handler_forwards {
                if let Some(target) = site.properties.iter().find_map(|prop| match prop {
                    AgeWidgetProperty::Message { name, target, .. } if name == channel => Some(target),
                    _ => None,
                }) {
                    channels.insert(message.clone(), target);
                }
            }
            for event in &site.events {
                channels.insert(event.name.clone(), &event.target);
            }
            for (message, target) in channels {
                writeln!(script, "[{}]: {},", quoted(&message), quoted(target)).unwrap();
            }
            script.push_str("} },\n");
        }
        script.push_str("} },\n");
    }
    script.push_str("};\n");
    Ok(roots)
}

fn icon_imports(compiled: &AgePackageCompilation) -> BTreeMap<String, String> {
    fn collect(items: &[ComponentViewItem], names: &mut BTreeSet<String>) {
        for item in items {
            match item {
                ComponentViewItem::Element(element) => {
                    let icon = match element.tag.as_str() {
                        "icon" => literal(element, "name"),
                        "button" => literal(element, "icon"),
                        _ => None,
                    };
                    if let Some(name) = icon {
                            if name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                                && name.bytes().all(|byte| {
                                    byte.is_ascii_lowercase()
                                        || byte.is_ascii_digit()
                                        || byte == b'-'
                                })
                            {
                                names.insert(name.into());
                            }
                    }
                    collect(&element.items, names);
                }
                ComponentViewItem::Cond(branch) => {
                    collect(&branch.then_items, names);
                    collect(&branch.else_items, names);
                }
                ComponentViewItem::For(iteration) => collect(&iteration.items, names),
                _ => {}
            }
        }
    }
    let mut names = BTreeSet::new();
    for runtime in &compiled.widget_runtime {
        let component = &compiled.description.components[runtime.node];
        if let Some(unit) = compiled.bound.units.get(&component.module) {
            if let Some((node, _, _)) = unit
                .age_components
                .iter()
                .find(|(node, _, _)| node.name.name == component.name)
            {
                collect(&node.view.tree, &mut names);
            }
        }
    }
    names
        .into_iter()
        .map(|name| {
            let symbol = match name.as_str() {
                // The checked generator pins lucide-vue-next 0.312, where
                // the same icon is exported under its former name.
                "loader-circle" => "Loader2".to_owned(),
                _ => channel_pascal(&name.replace('-', "_")),
            };
            (name, symbol)
        })
        .collect()
}

fn quoted(text: &str) -> String {
    serde_json::to_string(text).expect("serialize a JavaScript string")
}

fn escape_script(script: &str) -> String {
    let mut out = String::with_capacity(script.len());
    let mut start = 0;
    for (index, _) in script.to_ascii_lowercase().match_indices("</script") {
        out.push_str(&script[start..index + 1]);
        out.push('\\');
        start = index + 1;
    }
    out.push_str(&script[start..]);
    out
}
