use super::aura_view_builder::{is_svg_shape_tag, SVG_SHAPE_TAGS};
use ace_core::source::age_component::{
    ComponentEntryValue, ComponentViewElement, ComponentViewItem,
};
use ace_core::source::ast::{Expr, ExprKind, TypeWord};
use ace_av::age_package::{WidgetElement, WidgetViewItem};
use std::collections::BTreeMap;
use std::fmt::Write;

pub(super) const NAMESPACE: &str = "http://www.w3.org/2000/svg";
const ANNOTATION_TAGS: &[&str] = &["title", "desc"];
pub(super) const PROPERTIES: &[&str] = &[
    "viewBox", "width", "height", "preserveAspectRatio", "x", "y", "x1", "y1", "x2", "y2",
    "cx", "cy", "r", "rx", "ry", "d", "points", "pathLength", "fill", "fill-opacity",
    "fill-rule", "stroke", "stroke-width", "stroke-opacity", "stroke-linejoin",
    "stroke-linecap", "stroke-dasharray", "stroke-dashoffset", "opacity", "transform",
    "font-size", "font-family", "font-weight", "text-anchor", "dominant-baseline", "dx", "dy",
    "id", "clip-path", "clip-rule", "clipPathUnits", "mask", "maskUnits", "offset",
    "stop-color", "stop-opacity", "gradientUnits", "gradientTransform", "fx", "fy", "class",
    "style", "key", "text", "role", "aria-label", "aria-labelledby", "aria-describedby",
    "aria-hidden",
];

pub(super) fn tags() -> impl Iterator<Item = &'static str> {
    SVG_SHAPE_TAGS.iter().chain(ANNOTATION_TAGS).copied()
}

fn supports_tag(tag: &str) -> bool {
    is_svg_shape_tag(tag) || ANNOTATION_TAGS.contains(&tag)
}

pub(super) fn attributes<'a>(
    element: &'a ComponentViewElement,
    lane: &str,
) -> Result<BTreeMap<&'a str, &'a Expr>, String> {
    if !supports_tag(&element.tag) {
        return Err(format!("{lane}: unsupported SVG element `{}`", element.tag));
    }
    let mut attrs = BTreeMap::new();
    for item in &element.items {
        let ComponentViewItem::Entry(entry) = item else {
            continue;
        };
        let key = entry.key.as_str();
        if key == "xmlns"
            && matches!(
                &entry.value,
                ComponentEntryValue::Expr(expr)
                    if matches!(&expr.kind, ExprKind::StrLit(value) if value == NAMESPACE)
            )
        {
            continue;
        }
        match &entry.value {
            ComponentEntryValue::Expr(expr) if PROPERTIES.contains(&key) && !entry.is_event => {
                attrs.insert(key, expr);
            }
            ComponentEntryValue::ClassList(_) if matches!(key, "style" | "class") => {}
            _ => {
                return Err(format!(
                    "{lane}: unsupported `{key}` on SVG `{}`",
                    element.tag
                ))
            }
        }
    }
    Ok(attrs)
}

pub(super) fn checked_document(element: &WidgetElement) -> Result<String, String> {
    fn write_element(element: &WidgetElement, output: &mut String) -> Result<(), String> {
        let tag = element.tag.as_str();
        if !supports_tag(tag) {
            return Err(format!("ace.desktop.svg: unsupported SVG element `{tag}`"));
        }
        if !element.events.is_empty() {
            return Err(format!("ace.desktop.svg: unsupported event on SVG `{tag}`"));
        }
        write!(output, "<{tag}").unwrap();
        if tag == "svg" {
            write!(output, " xmlns=\"{NAMESPACE}\"").unwrap();
        }
        let mut classes = Vec::new();
        let mut label = element.label.as_deref();
        // The checked tape is ordered. Repeated geometry properties take
        // their last value, as they do in the browser's property map.
        for (key, value) in element.properties.iter().map(|(key, value)| (key.as_str(), value.as_str()))
            .collect::<BTreeMap<_, _>>()
        {
            match key {
                "xmlns" if value == NAMESPACE => {}
                "style" | "class" => classes.push(value),
                "key" => {}
                "text" => label = label.or(Some(value)),
                _ if PROPERTIES.contains(&key) => {
                    write!(output, " {key}=\"{}\"", escape_xml(value)).unwrap();
                }
                _ => return Err(format!("ace.desktop.svg: unsupported `{key}` on SVG `{tag}`")),
            }
        }
        if !classes.is_empty() {
            write!(output, " class=\"{}\"", escape_xml(&classes.join(" "))).unwrap();
        }
        output.push('>');
        if let Some(label) = label {
            output.push_str(&escape_xml(label));
        }
        for child in &element.children {
            match child {
                WidgetViewItem::Element(child) => write_element(child, output)?,
                WidgetViewItem::Text(value) if matches!(tag, "text" | "title" | "desc") => {
                    output.push_str(&escape_xml(value));
                }
                WidgetViewItem::Text(value) => {
                    write!(output, "<text>{}</text>", escape_xml(value)).unwrap();
                }
                WidgetViewItem::Timer(_) => return Err("source.age.sfc-svg: timer inside SVG".into()),
            }
        }
        write!(output, "</{tag}>").unwrap();
        Ok(())
    }
    let mut document = String::new();
    write_element(element, &mut document)?;
    Ok(document)
}

pub(super) fn scalar_literal(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::StrLit(value) => Some(value.clone()),
        ExprKind::IntLit(value) => Some(value.to_string()),
        ExprKind::BoolLit(value) => Some(value.to_string()),
        ExprKind::TypedLit {
            ty: TypeWord::Float,
            bits,
        } => {
            let value = f32::from_bits(*bits as u32);
            value.is_finite().then(|| value.to_string())
        }
        ExprKind::TypedLit {
            ty: TypeWord::F64,
            bits,
        } => {
            let value = f64::from_bits(*bits);
            value.is_finite().then(|| value.to_string())
        }
        _ => None,
    }
}

pub(super) fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
