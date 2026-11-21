use ace_core::source::age_component::{
    ComponentClassPart, ComponentEntryValue, ComponentViewElement, ComponentViewItem,
};
use ace_core::source::ast::{Expr, ExprKind};

pub(super) fn expression<'a>(element: &'a ComponentViewElement, name: &str) -> Option<&'a Expr> {
    element.items.iter().rev().find_map(|item| match item {
        ComponentViewItem::Entry(entry) if entry.key == name => match &entry.value {
            ComponentEntryValue::Expr(expr) => Some(expr),
            _ => None,
        },
        _ => None,
    })
}

pub(super) fn literal<'a>(element: &'a ComponentViewElement, name: &str) -> Option<&'a str> {
    match &expression(element, name)?.kind {
        ExprKind::StrLit(value) => Some(value),
        _ => None,
    }
}

pub(super) fn flag(element: &ComponentViewElement, name: &str) -> bool {
    match expression(element, name).map(|expr| &expr.kind) {
        Some(ExprKind::BoolLit(value)) => *value,
        Some(ExprKind::StrLit(value)) => value == "true",
        _ => false,
    }
}

pub(super) fn self_field(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Field { base, name } if matches!(&base.kind, ExprKind::Var(id) if id.name == "self") => {
            Some(&name.name)
        }
        _ => None,
    }
}

pub(super) fn element_children(
    element: &ComponentViewElement,
) -> impl Iterator<Item = &ComponentViewElement> {
    element.items.iter().filter_map(|item| match item {
        ComponentViewItem::Element(child) => Some(child),
        _ => None,
    })
}

pub(super) fn style(element: &ComponentViewElement) -> String {
    [literal(element, "style"), literal(element, "class")]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn conditional_classes(
    element: &ComponentViewElement,
) -> impl Iterator<Item = &ComponentClassPart> {
    element.items.iter().filter_map(|item| {
        if let ComponentViewItem::Entry(entry) = item {
            if let ComponentEntryValue::ClassList(parts) = &entry.value {
                return Some(parts.as_slice());
            }
        }
        None
    }).flatten()
}

pub(super) fn square_style(element: &ComponentViewElement) -> String {
    let measure = |name| match expression(element, name).map(|expr| &expr.kind) {
        Some(ExprKind::IntLit(number)) if *number >= 0 => Some(number.to_string()),
        Some(ExprKind::StrLit(word)) if word == "full" || word.parse::<u16>().is_ok() => {
            Some(word.clone())
        }
        _ => None,
    };
    let size = measure("size").unwrap_or_else(|| "12".into());
    let height = measure("h").unwrap_or_else(|| size.clone());
    let width = measure("w").unwrap_or(size);
    let color = literal(element, "color").unwrap_or("blue");
    let extra = style(element);
    format!("h-{height} w-{width} rounded-md border bg-{color}-500/40 border-{color}-500 text-{color}-600 font-medium text-sm {extra}")
}
