use super::PackageSfc;
use ace_core::source::{
    age_component::{ComponentEntryValue, ComponentNode, ComponentViewItem},
    ast::{Expr as CheckedExpr, ExprKind},
};
use auto_lang_core::ast::Expr;
use auto_ui_compiler::{
    aura::{AuraNode, AuraWidget},
    ui_gen::vue::VueGenerator,
};

/// Instantiate registered Vue controls from an admitted, literal page view.
/// Their interactive state belongs to the Vue controls and their mount.
pub(super) fn generate(node: &ComponentNode) -> Result<Option<PackageSfc>, String> {
    if node.public
        || !node.params.is_empty()
        || !node.package_imports.is_empty()
        || !node.members.is_empty()
        || !node.design.is_empty()
        || !node.stores.is_empty()
        || !node.view.named_trees.is_empty()
        || !node.view.node_refs.is_empty()
        || node.messages.iter().any(|message| message.outbound)
        || node.fields.iter().any(|field| literal(&field.init).is_none())
        || node.handlers.iter().any(|handler| {
            handler.body_foreign || !handler.stmts.is_empty() || !handler.channel_calls.is_empty()
        })
    {
        return Ok(None);
    }

    let [root] = node.view.tree.as_slice() else { return Ok(None) };
    let mut uses_composite = false;
    let Some(view_tree) = view(root, &mut uses_composite).filter(|_| uses_composite) else {
        return Ok(None);
    };
    let widget = AuraWidget {
        name: node.name.name.clone(),
        view_tree,
        state_vars: Vec::new(),
        computed: Vec::new(),
        messages: Vec::new(),
        named_views: Vec::new(),
        design: Vec::new(),
        handlers: Default::default(),
        handler_params: Default::default(),
        props: Vec::new(),
        routes: None,
        lifecycle: Vec::new(),
        tick_interval: None,
        timers: Vec::new(),
        span_map: Default::default(),
        key_bindings: Default::default(),
        api_imports: Vec::new(),
        style_css: None,
        ext_imports: Vec::new(),
        watchers: Vec::new(),
        exposes: Vec::new(),
        setup: None,
        actions: None,
    };
    let source = VueGenerator::new_shadcn()
        .generate_sfc(&widget)
        .map_err(|error| format!("source.age.sfc-builtin: {error}"))?;
    Ok(Some(PackageSfc {
        source,
        uses_runtime: false,
        embedded_widget_modules: Default::default(),
    }))
}

fn literal(expr: &CheckedExpr) -> Option<Expr> {
    match &expr.kind {
        ExprKind::StrLit(value) => Some(Expr::Str(value.clone().into())),
        ExprKind::IntLit(value) => Some(Expr::Int(*value)),
        ExprKind::BoolLit(value) => Some(Expr::Bool(*value)),
        _ => None,
    }
}

fn view(item: &ComponentViewItem, uses_composite: &mut bool) -> Option<AuraNode> {
    use crate::ui::ace_widget_props::literal as prop;

    match item {
        ComponentViewItem::Text(leaf) => {
            let ExprKind::StrLit(value) = &leaf.expr.kind else { return None };
            Some(AuraNode::text(value))
        }
        ComponentViewItem::Element(element) => {
            let tag = element.tag.as_str();
            *uses_composite |= matches!(
                tag,
                "alert-dialog" | "alert_dialog" | "drawer" | "dropdown-menu"
                    | "hover-card" | "hover_card" | "popover" | "select" | "sheet"
            );
            let local_href = tag == "link"
                && prop(element, "href").is_some_and(|value| {
                    value.starts_with('/') && !value.starts_with("//")
                });
            let internal_link = local_href || (tag == "link" && prop(element, "to").is_some());
            let mut node = AuraNode::element(if internal_link { "nav_link" } else { tag });
            if let Some(label) = &element.label {
                let ExprKind::StrLit(value) = &label.kind else { return None };
                node = node.with_child(AuraNode::text(value));
            }
            for item in &element.items {
                if let ComponentViewItem::Entry(entry) = item {
                    let ComponentEntryValue::Expr(expr) = &entry.value else { return None };
                    if entry.is_event {
                        return None;
                    }
                    let key = if local_href && entry.key == "href" { "to" } else { &entry.key };
                    node = node.with_prop(key, literal(expr)?);
                } else {
                    node = node.with_child(view(item, uses_composite)?);
                }
            }
            Some(node)
        }
        _ => None,
    }
}
