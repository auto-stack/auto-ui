use super::{native_style, Renderer, WidgetView};
use crate::ui::ace_node_desktop::eval_view_cond;
use crate::ui::ace_svg;
use crate::ui::View;
use ace_core::source::age_component::{ComponentViewElement, ComponentViewItem};
use ace_core::source::ast::Expr;
use std::fmt::Write;

const DIAGNOSTIC: &str = "ace.desktop.svg";

impl Renderer<'_> {
    pub(super) fn svg(&mut self, element: &ComponentViewElement) -> WidgetView {
        let snapshot = self.texts.clone();
        let mut document = String::new();
        match self.svg_element(element, &mut document) {
            Ok(classes) => View::Image {
                src: format!("svgdoc:{document}"),
                style: Some(native_style(&classes)),
            },
            Err(error) => {
                self.texts = snapshot;
                self.skip_svg_texts(&element.items);
                View::text(error)
            }
        }
    }

    fn svg_value(&self, expr: &Expr) -> Result<String, String> {
        self.display(expr)
            .or_else(|| ace_svg::scalar_literal(expr))
            .ok_or_else(|| {
                format!(
                    "{DIAGNOSTIC}: unsupported SVG value at byte {}",
                    expr.span.start
                )
            })
    }

    fn svg_element(
        &mut self,
        element: &ComponentViewElement,
        document: &mut String,
    ) -> Result<String, String> {
        let tag = &element.tag;
        let mut attributes = ace_svg::attributes(element, DIAGNOSTIC)?;
        let label = attributes.remove("text");
        write!(document, "<{tag}").unwrap();
        if tag == "svg" {
            write!(document, " xmlns=\"{}\"", ace_svg::NAMESPACE).unwrap();
        }
        let mut classes = Vec::new();
        for (key, expr) in attributes {
            match key {
                "style" | "class" => classes.push(self.svg_value(expr)?),
                "key" => {}
                _ => {
                    let value = ace_svg::escape_xml(&self.svg_value(expr)?);
                    write!(document, " {key}=\"{value}\"").unwrap();
                }
            }
        }
        let mut classes = classes.join(" ");
        self.append_conditional_classes(element, &mut classes)?;
        if !classes.trim().is_empty() {
            write!(document, " class=\"{}\"", ace_svg::escape_xml(classes.trim())).unwrap();
        }
        document.push('>');
        let label = element.label.as_ref().or(label);
        let observed = if tag == "text" {
            self.texts.next().cloned()
        } else {
            None
        };
        let label = match observed {
            Some(value) => Some(value),
            None => label.map(|expr| self.svg_value(expr)).transpose()?,
        };
        if let Some(label) = label {
            document.push_str(&ace_svg::escape_xml(&label));
        }
        self.svg_children(&element.items, document)?;
        write!(document, "</{tag}>").unwrap();
        Ok(classes)
    }

    fn svg_children(
        &mut self,
        items: &[ComponentViewItem],
        document: &mut String,
    ) -> Result<(), String> {
        for child in items {
            match child {
                ComponentViewItem::Element(child) => {
                    self.svg_element(child, document)?;
                }
                ComponentViewItem::Text(leaf) => {
                    let label = match self.texts.next() {
                        Some(label) => label.clone(),
                        None => self.svg_value(&leaf.expr)?,
                    };
                    write!(document, "<text>{}</text>", ace_svg::escape_xml(&label)).unwrap();
                }
                ComponentViewItem::Entry(_) => {}
                ComponentViewItem::Cond(condition) => {
                    if eval_view_cond(&condition.cond, self.values)? {
                        self.svg_children(&condition.then_items, document)?;
                        self.skip_svg_texts(&condition.else_items);
                    } else {
                        self.skip_svg_texts(&condition.then_items);
                        self.svg_children(&condition.else_items, document)?;
                    }
                }
                ComponentViewItem::For(_) => {
                    return Err(format!(
                        "{DIAGNOSTIC}: SVG child needs evaluated control flow"
                    ));
                }
            }
        }
        Ok(())
    }

    fn skip_svg_texts(&mut self, items: &[ComponentViewItem]) {
        if let Some(index) = text_count(items).checked_sub(1) {
            self.texts.nth(index);
        }
    }
}

fn text_count(items: &[ComponentViewItem]) -> usize {
    items
        .iter()
        .map(|item| match item {
            ComponentViewItem::Text(_) => 1,
            ComponentViewItem::Element(element) if element.tag == "text" => 1,
            ComponentViewItem::Element(element)
                if matches!(element.tag.as_str(), "button" | "input") =>
            {
                0
            }
            ComponentViewItem::Element(element) => text_count(&element.items),
            ComponentViewItem::Cond(cond) => {
                text_count(&cond.then_items) + text_count(&cond.else_items)
            }
            ComponentViewItem::For(each) => text_count(&each.items),
            ComponentViewItem::Entry(_) => 0,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::ace_node_desktop::widgets::{view, State};
    use crate::ui::ace_package_bridge::PackageStateEngine;
    use crate::ui::ace_route::package_candidates;
    use ace_core::source::age_h0::LegacyWidgetAlias;
    use ace_core::source::packages::PackageModuleInput;
    use std::collections::BTreeMap;

    fn svg_image(view: &WidgetView) -> Option<&str> {
        match view {
            View::Image { src, .. } => src.strip_prefix("svgdoc:"),
            View::Column { children, .. } => children.iter().find_map(svg_image),
            View::Container { child, .. } => svg_image(child),
            _ => None,
        }
    }

    #[test]
    fn checked_svg_draws_updated_geometry_from_the_av_model() {
        let source = r##"
node Diagram {
    msg { Extend }
    model { var extent int = 4 }
    view {
        preview-card {
            svg (viewBox: "0 0 20 20") {
                if .extent == 4 && true {
                    line (x1: 1, y1: 4, x2: .extent, y2: 4, stroke: "#ff0000", stroke-width: 2) {}
                    text "Short" { x: 2, y: 10 }
                } else {
                    line (x1: 1, y1: 4, x2: .extent, y2: 4, stroke: "#0000ff", stroke-width: 2) {}
                    text "Long" { x: 2, y: 10 }
                }
                text "A < B & C" { x: 2, y: 14 }
            }
        }
    }
    on { .Extend -> { .extent = 18 } }
}
"##;
        let node = &package_candidates(source).expect("project checked SVG")[0];
        let input = PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: source.into(),
            provider: None,
        };
        let mut engine =
            PackageStateEngine::new(vec![input], "app::main", LegacyWidgetAlias::Reject)
                .expect("construct AV package");
        engine
            .mount("instance", Vec::new())
            .expect("mount AV model");
        let raster = |engine: &PackageStateEngine, active: &str, hidden: &str| {
            let states = engine.read_all("instance").expect("read AV model");
            let texts = engine.texts("instance").expect("observe SVG label");
            let rendered = view(
                1,
                0,
                &node.view.tree,
                &texts,
                &states,
                &State::new(BTreeMap::new()),
            );
            let document = svg_image(&rendered).expect("Iced receives inline SVG");
            assert!(document.contains("A &lt; B &amp; C"), "{document}");
            assert!(document.contains(active), "{document}");
            assert!(!document.contains(hidden), "{document}");
            let tree = resvg::usvg::Tree::from_str(document, &resvg::usvg::Options::default())
                .expect("renderable SVG document");
            let mut pixels = tiny_skia::Pixmap::new(20, 20).unwrap();
            resvg::render(
                &tree,
                tiny_skia::Transform::identity(),
                &mut pixels.as_mut(),
            );
            pixels
        };
        let before = raster(&engine, "Short", "Long");
        assert_eq!(before.pixel(12, 4).unwrap().alpha(), 0);
        engine
            .dispatch("instance", "Extend")
            .expect("dispatch checked handler in AV");
        let after = raster(&engine, "Long", "Short");
        let pixel = after.pixel(12, 4).unwrap();
        assert_eq!(
            (pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()),
            (0, 0, 255, 255)
        );
    }
}
