use ace_av::age_package::{WidgetElement, WidgetViewItem};

use crate::ui::ace_svg;
use crate::ui::aura_view_builder::parse_coords_extent;
use crate::ui::view::PointerMoveHandler;
use crate::ui::{Style, View};

use super::{NodeDesktopMsg, PackageFrame};

pub(super) fn view(
    session: i32,
    instance: usize,
    frame: &PackageFrame,
    outlet: Option<&View<NodeDesktopMsg>>,
    routed: bool,
) -> View<NodeDesktopMsg> {
    let context = FrameView {
        session,
        instance,
        frame,
        routed,
    };
    View::col()
        .children(context.items(&frame.view, &mut Vec::new(), outlet))
        .spacing(4)
        .build()
}

struct FrameView<'a> {
    session: i32,
    instance: usize,
    frame: &'a PackageFrame,
    routed: bool,
}

impl FrameView<'_> {
    fn items(
        &self,
        items: &[WidgetViewItem],
        path: &mut Vec<usize>,
        outlet: Option<&View<NodeDesktopMsg>>,
    ) -> Vec<View<NodeDesktopMsg>> {
        items
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| match item {
                WidgetViewItem::Text(text) => Some(View::text(text.clone())),
                WidgetViewItem::Element(element) => {
                    path.push(slot);
                    let rendered = self.element(element, path, outlet);
                    path.pop();
                    Some(rendered)
                }
                WidgetViewItem::Timer(_) => None,
            })
            .collect()
    }

    fn element(
        &self,
        element: &WidgetElement,
        path: &mut Vec<usize>,
        outlet: Option<&View<NodeDesktopMsg>>,
    ) -> View<NodeDesktopMsg> {
        if self.frame.is_widget(path) {
            return View::col().children(self.items(&element.children, path, None)).build();
        }
        let style = style(element);
        if style.as_ref().is_some_and(Style::is_hidden) {
            return View::empty();
        }
        if element.tag == "outlet" {
            return outlet.cloned().unwrap_or_else(View::empty);
        }
        if element.tag == "svg" {
            return match ace_svg::checked_document(element) {
                Ok(document) => View::Image {
                    src: format!("svgdoc:{document}"),
                    style,
                },
                Err(error) => View::text(error),
            };
        }
        if element.tag == "drawer" && prop(element, "open") != Some("true") {
            return View::empty();
        }
        let mut children = self.items(&element.children, path, outlet);
        let label = element.label.as_deref().or_else(|| prop(element, "text"));
        let link = self.routed.then(|| self.frame.link(path)).flatten();
        let (session, instance, revision) = (self.session, self.instance, self.frame.revision);
        let action = |target| NodeDesktopMsg::PackageClick {
                session,
                instance,
                revision,
                target,
            };
        let click = link.or_else(|| self.frame.click(path)).map(action);
        if element.tag == "checkbox" {
            return View::Checkbox {
                is_checked: prop(element, "checked") == Some("true"),
                label: label.unwrap_or_default().to_owned(),
                on_toggle: click,
                style,
            };
        }
        if element.tag == "button" || link.is_some() {
            if let Some(onclick) = click {
                let icon = prop(element, "icon");
                let label = label.or_else(|| first_text(&element.children))
                    .filter(|label| !label.is_empty()).or(icon)
                    .or_else(|| prop(element, "aria-label")).unwrap_or_default();
                if let Some(icon) = icon {
                    children.insert(0, View::Image {
                        src: format!("lucide:{icon}"),
                        style: Style::parse("w-4 h-4").ok(),
                    });
                }
                return View::Button {
                    label: label.to_owned(),
                    onclick,
                    style,
                    on_right_click: None,
                    content: (!children.is_empty())
                        .then(|| Box::new(View::row().spacing(6).children(children).build())),
                    disabled: false,
                    keyboard: None,
                };
            }
        }
        if element.tag == "drawer" {
            if let Some(onclick) = click {
                children.push(View::button_styled("Close drawer", onclick, "px-2 py-1"));
            }
            return View::col().children(children).build();
        }
        if let Some(pointer) = self.frame.pointer(path) {
            let logical_extent = prop(element, "coords").and_then(parse_coords_extent)
                .filter(|(width, height)| width.is_finite() && height.is_finite());
            if pointer.motion.is_some() && logical_extent.is_none() && prop(element, "coords").is_some() {
                return View::text("ace.desktop.pointer: coords must have a finite WxH extent");
            }
            return View::MouseArea {
                content: Box::new(View::col().children(children).build()),
                on_click: click,
                on_enter: pointer.enter.map(action),
                on_exit: pointer.leave.map(action),
                on_double_click: None,
                on_context_menu: None,
                on_release: None,
                on_move: pointer.motion.map(|target| {
                    PointerMoveHandler::new(move |x, y| NodeDesktopMsg::PackagePointer {
                        session, instance, revision, target, x, y,
                    })
                }),
                logical_extent,
                style,
            };
        }
        let content = match element.tag.as_str() {
            "icon" => View::Image {
                src: format!("lucide:{}", prop(element, "name").unwrap_or_default()),
                style,
            },
            "text" | "span" | "h1" | "h2" | "h3" | "th" | "td" | "codeblock"
                if children.is_empty() =>
            {
                View::Text {
                    content: label
                        .or_else(|| prop(element, "code"))
                        .unwrap_or_default()
                        .to_owned(),
                    style,
                    selectable: element.tag == "codeblock",
                }
            }
            "sidebar_content" => {
                let content = View::col().children(children).build();
                let mut viewport = View::scrollable(content);
                if let Some(style) = style {
                    viewport = viewport.with_style(style);
                }
                viewport.build()
            }
            tag => {
                if let Some(label) = label {
                    children.insert(0, View::text(label.to_owned()));
                }
                let mut builder = if matches!(tag, "row" | "tr" | "mouse-area") {
                    View::row()
                } else {
                    View::col()
                };
                if let Some(style) = style {
                    builder = builder.with_style(style);
                }
                let content = builder.children(children).build();
                if tag == "scroll" {
                    View::scrollable(content).build()
                } else {
                    content
                }
            }
        };
        if let Some(on_click) = click {
            View::MouseArea {
                content: Box::new(content),
                on_click: Some(on_click),
                on_enter: None,
                on_exit: None,
                on_double_click: None,
                on_context_menu: None,
                on_release: None,
                on_move: None,
                logical_extent: None,
                style: None,
            }
        } else {
            content
        }
    }
}

fn first_text(items: &[WidgetViewItem]) -> Option<&str> {
    items.iter().find_map(|item| match item {
        WidgetViewItem::Text(text) => Some(text.as_str()),
        WidgetViewItem::Element(element) => element.label.as_deref()
            .or_else(|| prop(element, "text"))
            .or_else(|| first_text(&element.children)),
        WidgetViewItem::Timer(_) => None,
    })
}

fn prop<'a>(element: &'a WidgetElement, key: &str) -> Option<&'a str> {
    element
        .properties
        .iter()
        .rev()
        .find_map(|(name, value)| (name == key).then_some(value.as_str()))
}

fn style(element: &WidgetElement) -> Option<Style> {
    let mut properties = ["style", "class"]
        .into_iter()
        .filter_map(|key| prop(element, key));
    let first = properties.next();
    let second = properties.next();
    match (first, second) {
        (Some(first), Some(second)) => Style::parse(&format!("{first} {second}")).ok(),
        (Some(one), None) => Style::parse(one).ok(),
        _ => {
            let default = match element.tag.as_str() {
                "h1" => "text-2xl font-semibold",
                "h2" => "text-lg font-semibold",
                "preview-card" => "rounded-md border p-4",
                "codeblock" => "font-mono bg-muted p-2",
                _ => return None,
            };
            Style::parse(default).ok()
        }
    }
}
