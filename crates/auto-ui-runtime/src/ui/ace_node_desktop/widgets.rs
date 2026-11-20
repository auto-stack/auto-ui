use super::NodeDesktopMsg;
use crate::ui::ace_widget_props::{
    element_children, expression, flag, literal, self_field, square_style, style,
};
use crate::ui::scroll::ScrollAxes;
use crate::ui::view::{AccordionItem, ScrollMetrics};
use crate::ui::{Style, View};
use ace_core::source::age_component::{ComponentField, ComponentViewElement, ComponentViewItem};
use ace_core::source::ast::{Expr, ExprKind};
use std::collections::{BTreeMap, BTreeSet};

type WidgetView = View<NodeDesktopMsg>;

#[derive(Clone, Debug)]
pub enum Action {
    Focus(usize),
    Accordion {
        key: usize,
        item: usize,
        open: bool,
        multiple: bool,
        collapsible: bool,
    },
    Collapsible {
        key: usize,
        field: Option<String>,
        open: bool,
    },
    Page {
        key: usize,
        page: u32,
    },
    Slide {
        key: usize,
        value: f32,
    },
    Scroll {
        key: usize,
        metrics: ScrollMetrics,
    },
}

#[derive(Clone, Debug)]
pub(super) struct State {
    focus: Option<usize>,
    expanded: BTreeMap<usize, BTreeSet<usize>>,
    disclosure: BTreeMap<usize, bool>,
    bindings: BTreeMap<String, bool>,
    pages: BTreeMap<usize, u32>,
    slider: BTreeMap<usize, f32>,
    scroll: BTreeMap<usize, ScrollMetrics>,
}

impl State {
    pub fn new(bindings: BTreeMap<String, bool>) -> Self {
        Self {
            focus: None,
            expanded: BTreeMap::new(),
            disclosure: BTreeMap::new(),
            bindings,
            pages: BTreeMap::new(),
            slider: BTreeMap::new(),
            scroll: BTreeMap::new(),
        }
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Focus(key) => {
                self.focus = Some(key);
            }
            Action::Accordion {
                key,
                item,
                open,
                multiple,
                collapsible,
            } => {
                let selected = self.expanded.entry(key).or_default();
                if open {
                    if !multiple {
                        selected.clear();
                    }
                    selected.insert(item);
                } else if multiple || collapsible {
                    selected.remove(&item);
                }
            }
            Action::Collapsible { key, field, open } => {
                if let Some(field) = field {
                    self.bindings.insert(field, open);
                } else {
                    self.disclosure.insert(key, open);
                }
            }
            Action::Page { key, page } => {
                self.pages.insert(key, page);
            }
            Action::Slide { key, value } => {
                self.slider.insert(key, value);
            }
            Action::Scroll { key, metrics } => {
                self.scroll.insert(key, metrics);
            }
        }
    }

    pub fn evidence(&self) -> serde_json::Value {
        serde_json::json!({
            "focus": self.focus,
            "expanded": self.expanded,
            "disclosure": self.disclosure,
            "bindings": self.bindings,
            "pages": self.pages,
            "slider": self.slider,
            "scroll": self.scroll.iter().map(|(key, m)| serde_json::json!({
                "key": key, "offset": [m.offset_x, m.offset_y],
                "viewport": [m.viewport_w, m.viewport_h], "content": [m.content_w, m.content_h],
            })).collect::<Vec<_>>(),
        })
    }
}

pub(super) fn initial_bools(fields: &[ComponentField]) -> BTreeMap<String, bool> {
    fields
        .iter()
        .filter_map(|field| match field.init.kind {
            ExprKind::BoolLit(value) => Some((field.name.clone(), value)),
            _ => None,
        })
        .collect()
}

pub(super) fn contains_preview(items: &[ComponentViewItem]) -> bool {
    items.iter().any(|item| match item {
        ComponentViewItem::Element(element) if element.tag == "preview-card" => {
            contains_control(&element.items)
        }
        ComponentViewItem::Element(element) => contains_preview(&element.items),
        _ => false,
    })
}

fn contains_control(items: &[ComponentViewItem]) -> bool {
    items.iter().any(|item| match item {
        ComponentViewItem::Element(element) => {
            matches!(
                element.tag.as_str(),
                "accordion" | "collapsible" | "pagination" | "progress" | "slider"
            ) || scroll_axes(element, &style(element)).is_some()
                || contains_control(&element.items)
        }
        _ => false,
    })
}

pub(super) fn view(
    session: i32,
    instance: usize,
    tree: &[ComponentViewItem],
    texts: &[String],
    values: &[(String, String)],
    state: &State,
) -> WidgetView {
    let mut renderer = Renderer {
        session,
        instance,
        texts: texts.iter(),
        values,
        state,
        targets: Vec::new(),
    };
    View::col()
        .spacing(10)
        .children(renderer.items(tree))
        .build()
}

pub(super) const KEYS: &[&str] = &[
    "Tab",
    "Shift+Tab",
    "Enter",
    " ",
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    "ArrowDown",
    "Home",
    "End",
];

enum KeyboardTarget {
    Activate(Action),
    Slider {
        min: f32,
        max: f32,
        step: f32,
        value: f32,
    },
}

pub(super) fn keyboard_action(
    tree: &[ComponentViewItem],
    texts: &[String],
    values: &[(String, String)],
    state: &State,
    key: &str,
) -> Option<Action> {
    let mut renderer = Renderer {
        session: 0,
        instance: 0,
        texts: texts.iter(),
        values,
        state,
        targets: Vec::new(),
    };
    renderer.items(tree);
    let targets = renderer.targets;
    let focus = state
        .focus
        .and_then(|focus| targets.iter().position(|(id, _)| *id == focus));
    if matches!(key, "Tab" | "Shift+Tab") {
        let n = targets.len();
        if n == 0 {
            return None;
        }
        let next = if key == "Shift+Tab" {
            focus.map_or(n - 1, |i| (i + n - 1) % n)
        } else {
            focus.map_or(0, |i| (i + 1) % n)
        };
        return Some(Action::Focus(targets[next].0));
    }
    let (id, target) = &targets[focus?];
    match target {
        KeyboardTarget::Activate(action) if matches!(key, "Enter" | " ") => Some(action.clone()),
        KeyboardTarget::Slider {
            min,
            max,
            step,
            value,
        } => {
            let value = match key {
                "Home" => *min,
                "End" => *max,
                "ArrowLeft" | "ArrowDown" => value - step,
                "ArrowRight" | "ArrowUp" => value + step,
                _ => return None,
            }
            .clamp(*min, *max);
            Some(Action::Slide { key: *id, value })
        }
        _ => None,
    }
}

struct Renderer<'a> {
    session: i32,
    instance: usize,
    texts: std::slice::Iter<'a, String>,
    values: &'a [(String, String)],
    state: &'a State,
    targets: Vec<(usize, KeyboardTarget)>,
}

impl Renderer<'_> {
    fn action(&self, action: Action) -> NodeDesktopMsg {
        NodeDesktopMsg::Widget {
            session: self.session,
            instance: self.instance,
            action,
        }
    }

    fn button(
        &mut self,
        key: usize,
        label: String,
        classes: &str,
        action: Option<Action>,
    ) -> WidgetView {
        if let Some(action) = &action {
            self.targets
                .push((key, KeyboardTarget::Activate(action.clone())));
        }
        let classes = if self.state.focus == Some(key) {
            format!("{classes} border-2 border-blue-700")
        } else {
            classes.to_owned()
        };
        View::Button {
            disabled: action.is_none(),
            label,
            onclick: action
                .map(|action| self.action(action))
                .unwrap_or(NodeDesktopMsg::BridgeTick),
            on_right_click: None,
            style: Some(Style::parse(&classes).unwrap_or_default()),
            content: None,
        }
    }

    fn display(&self, expr: &Expr) -> Option<String> {
        match &expr.kind {
            ExprKind::StrLit(value) => Some(value.clone()),
            ExprKind::IntLit(value) => Some(value.to_string()),
            ExprKind::BoolLit(value) => Some(value.to_string()),
            _ => self_field(expr)
                .and_then(|field| self.values.iter().find(|(name, _)| name == field))
                .map(|(_, value)| value.clone()),
        }
    }

    fn number(&self, el: &ComponentViewElement, prop: &str, default: f32) -> f32 {
        expression(el, prop)
            .and_then(|expr| self.display(expr))
            .and_then(|value| value.parse::<f32>().ok())
            .filter(|value| value.is_finite())
            .unwrap_or(default)
    }

    fn items(&mut self, items: &[ComponentViewItem]) -> Vec<WidgetView> {
        items
            .iter()
            .filter_map(|item| match item {
                ComponentViewItem::Text(leaf) => {
                    let text = self
                        .texts
                        .next()
                        .cloned()
                        .or_else(|| self.display(&leaf.expr))
                        .unwrap_or_default();
                    Some(View::text(text))
                }
                ComponentViewItem::Element(el) => Some(self.element(el)),
                _ => None,
            })
            .collect()
    }

    fn element(&mut self, el: &ComponentViewElement) -> WidgetView {
        let key = el.span.start;
        let classes = style(el);
        let parsed = native_style(&classes);
        match el.tag.as_str() {
            "accordion" => self.accordion(el, key),
            "collapsible" => self.collapsible(el, key),
            "pagination" => self.pagination(el, key),
            "progress" => {
                let max = self.number(el, "max", 100.0).max(f32::EPSILON);
                let value = self.number(el, "value", 0.0).clamp(0.0, max);
                View::col()
                    .spacing(5)
                    .child(View::ProgressBar {
                        progress: value / max,
                        style: Some(parsed),
                        on_seek: None,
                    })
                    .child(View::text(format!("{value} / {max}")))
                    .build()
            }
            "slider" => {
                let min = self.number(el, "min", 0.0);
                let max = self.number(el, "max", 100.0).max(min);
                let step = self.number(el, "step", 1.0).max(f32::EPSILON);
                let value = self
                    .state
                    .slider
                    .get(&key)
                    .copied()
                    .unwrap_or_else(|| self.number(el, "value", min))
                    .clamp(min, max);
                self.targets.push((
                    key,
                    KeyboardTarget::Slider {
                        min,
                        max,
                        step,
                        value,
                    },
                ));
                let (session, instance) = (self.session, self.instance);
                let slider = View::slider(min..=max, value)
                    .step(step)
                    .with_style(parsed)
                    .on_change(move |value| NodeDesktopMsg::Widget {
                        session,
                        instance,
                        action: Action::Slide { key, value },
                    })
                    .build();
                let slider = if self.state.focus == Some(key) {
                    View::container(slider)
                        .style("border-2 border-blue-700")
                        .build()
                } else {
                    slider
                };
                View::col()
                    .spacing(5)
                    .child(slider)
                    .child(View::text(format!("{value}")))
                    .build()
            }
            "text" => {
                let value = self
                    .texts
                    .next()
                    .cloned()
                    .or_else(|| el.label.as_ref().and_then(|label| self.display(label)))
                    .unwrap_or_default();
                View::Text {
                    content: value,
                    style: Some(parsed),
                    selectable: false,
                }
            }
            "square" => {
                let mut children = self.items(&el.items);
                if children.is_empty() {
                    if let Some(text) = literal(el, "text") {
                        children.push(View::text(text));
                    }
                }
                View::container(View::col().children(children).build())
                    .center_x()
                    .center_y()
                    .with_style(Style::parse(&square_style(el)).unwrap_or_default())
                    .build()
            }
            "codeblock" => {
                View::text_styled(literal(el, "code").unwrap_or_default(), "text-sm font-mono")
            }
            "separator" => View::container(View::empty())
                .style("w-full h-1 bg-gray-200")
                .build(),
            _ => {
                let mut children = self.items(&el.items);
                if let Some(label) = el.label.as_ref().and_then(|expr| self.display(expr)) {
                    let heading = match el.tag.as_str() {
                        "h1" => "text-2xl font-bold text-slate-900",
                        "h2" => "text-lg font-semibold text-slate-900",
                        _ => "text-slate-900",
                    };
                    children.insert(0, View::text_styled(label, heading));
                }
                if let Some(axes) = scroll_axes(el, &classes) {
                    let inner = if axes == ScrollAxes::X {
                        View::row().spacing(8)
                    } else {
                        View::col().spacing(8)
                    };
                    let (session, instance) = (self.session, self.instance);
                    return View::scrollable(inner.children(children).build())
                        .axes(axes)
                        .with_style(parsed)
                        .on_scroll(move |metrics| NodeDesktopMsg::Widget {
                            session,
                            instance,
                            action: Action::Scroll { key, metrics },
                        })
                        .build();
                }
                if let Some(cols) = classes.split_whitespace().find_map(|name| {
                    name.strip_prefix("grid-cols-")
                        .and_then(|n| n.parse::<usize>().ok())
                }) {
                    return View::grid()
                        .cols(cols)
                        .spacing(8)
                        .with_style(parsed)
                        .children(children)
                        .build();
                }
                if matches!(el.tag.as_str(), "row" | "tr") {
                    View::row()
                        .spacing(8)
                        .with_style(parsed)
                        .children(children)
                        .build()
                } else if el.tag == "preview-card" {
                    View::container(View::col().spacing(8).children(children).build())
                        .padding(12)
                        .with_style(parsed)
                        .build()
                } else {
                    View::col()
                        .spacing(8)
                        .with_style(parsed)
                        .children(children)
                        .build()
                }
            }
        }
    }

    fn label(&mut self, element: &ComponentViewElement) -> String {
        let mut parts = Vec::new();
        if let Some(value) = element.label.as_ref().and_then(|expr| self.display(expr)) {
            parts.push(value);
        }
        for item in &element.items {
            match item {
                ComponentViewItem::Text(leaf) => {
                    if let Some(value) = self
                        .texts
                        .next()
                        .cloned()
                        .or_else(|| self.display(&leaf.expr))
                    {
                        parts.push(value);
                    }
                }
                ComponentViewItem::Element(text) if text.tag == "text" => {
                    if let Some(value) = self
                        .texts
                        .next()
                        .cloned()
                        .or_else(|| text.label.as_ref().and_then(|label| self.display(label)))
                    {
                        parts.push(value);
                    }
                }
                _ => {}
            }
        }
        parts.join(" ")
    }

    fn accordion(&mut self, el: &ComponentViewElement, key: usize) -> WidgetView {
        let multiple = literal(el, "type") == Some("multiple");
        let collapsible = flag(el, "collapsible");
        let selected = self.state.expanded.get(&key);
        let items = element_children(el)
            .filter(|item| item.tag == "accordion-item")
            .enumerate()
            .map(|(index, item)| {
                let mut title = String::new();
                let mut panels = Vec::new();
                for child in element_children(item) {
                    match child.tag.as_str() {
                        "accordion-trigger" => title = self.label(child),
                        "accordion-content" => panels.extend(self.items(&child.items)),
                        _ => {}
                    }
                }
                let open = selected.is_some_and(|items| items.contains(&index));
                let key_for_header = item.span.start;
                self.targets.push((
                    key_for_header,
                    KeyboardTarget::Activate(Action::Accordion {
                        key,
                        item: index,
                        open: !open,
                        multiple,
                        collapsible,
                    }),
                ));
                if self.state.focus == Some(key_for_header) {
                    title.insert_str(0, "› ");
                }
                AccordionItem::new(title)
                    .with_icon(if open { '▾' } else { '▸' })
                    .with_children(panels)
                    .with_expanded(open)
            })
            .collect();
        let (session, instance) = (self.session, self.instance);
        View::accordion()
            .allow_multiple(multiple)
            .items(items)
            .on_toggle(move |item, open| NodeDesktopMsg::Widget {
                session,
                instance,
                action: Action::Accordion {
                    key,
                    item,
                    open,
                    multiple,
                    collapsible,
                },
            })
            .build()
    }

    fn collapsible(&mut self, el: &ComponentViewElement, key: usize) -> WidgetView {
        let field = expression(el, "open").and_then(self_field);
        let open = field
            .and_then(|name| self.state.bindings.get(name))
            .or_else(|| self.state.disclosure.get(&key))
            .copied()
            .unwrap_or_else(|| flag(el, "open"));
        let mut view = View::col().spacing(8);
        for child in element_children(el) {
            match child.tag.as_str() {
                "collapsible-trigger" => {
                    let label = self.label(child);
                    let action = (!flag(el, "disabled")).then(|| Action::Collapsible {
                        key,
                        field: field.map(str::to_owned),
                        open: !open,
                    });
                    view = view.child(self.button(
                        child.span.start,
                        label,
                        &format!("{} bg-slate-100 text-slate-900 rounded-md", style(child)),
                        action,
                    ));
                }
                "collapsible-content" => {
                    let panel = self.items(&child.items);
                    if open {
                        view = view.children(panel);
                    }
                }
                _ => {}
            }
        }
        view.build()
    }

    fn pagination(&mut self, el: &ComponentViewElement, key: usize) -> WidgetView {
        let total = self.number(el, "total", 0.0).max(0.0);
        let per_page = self.number(el, "per_page", 10.0).max(1.0);
        let last = (total / per_page).ceil().max(1.0) as u32;
        let page = self
            .state
            .pages
            .get(&key)
            .copied()
            .unwrap_or_else(|| self.number(el, "page", 1.0) as u32)
            .clamp(1, last);
        let mut row = View::row().spacing(8);
        for list in element_children(el).filter(|child| child.tag == "pagination-list") {
            for child in element_children(list) {
                let (label, target, selected) = match child.tag.as_str() {
                    "pagination-previous" => {
                        ("Previous".to_owned(), (page > 1).then(|| page - 1), false)
                    }
                    "pagination-next" => {
                        ("Next".to_owned(), (page < last).then(|| page + 1), false)
                    }
                    "pagination-item" => {
                        let value = (self.number(child, "value", 0.0) as u32).clamp(1, last);
                        (self.label(child), Some(value), page == value)
                    }
                    "pagination-ellipsis" => {
                        row = row.child(View::text("…"));
                        continue;
                    }
                    _ => continue,
                };
                let classes = if selected {
                    "font-bold bg-blue-600 text-white rounded-md"
                } else {
                    "bg-slate-100 text-slate-900 rounded-md"
                };
                row = row.child(self.button(
                    child.span.start,
                    label,
                    classes,
                    target.map(|page| Action::Page { key, page }),
                ));
            }
        }
        View::col()
            .spacing(5)
            .child(row.build())
            .child(View::text(format!("Page {page} / {last}")))
            .build()
    }
}

fn native_style(classes: &str) -> Style {
    let normalized = classes
        .split_whitespace()
        .map(|class| {
            let fixed_rem_width = class
                .strip_prefix("w-[")
                .and_then(|width| width.strip_suffix("rem]"))
                .and_then(|width| width.parse::<f32>().ok())
                .filter(|width| width.is_finite() && *width > 0.0);
            match fixed_rem_width {
                Some(width) => format!("w-[{}px]", width * 16.0),
                None => class.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Style::parse(&normalized).unwrap_or_default()
}

fn scroll_axes(element: &ComponentViewElement, classes: &str) -> Option<ScrollAxes> {
    let mut axes = None;
    for class in classes.split_whitespace() {
        match class {
            "overflow-auto" => return Some(ScrollAxes::BOTH),
            "overflow-x-auto" => {
                axes = Some(if axes == Some(ScrollAxes::Y) {
                    ScrollAxes::BOTH
                } else {
                    ScrollAxes::X
                })
            }
            "overflow-y-auto" => {
                axes = Some(if axes == Some(ScrollAxes::X) {
                    ScrollAxes::BOTH
                } else {
                    ScrollAxes::Y
                })
            }
            _ => {}
        }
    }
    axes.or_else(|| (element.tag == "scroll").then_some(ScrollAxes::Y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::source::age_component::project_component_candidates;

    #[test]
    fn native_widget_keyboard_targets_the_bound_disclosure_and_bounded_slider() {
        let source = r#"
widget Preview {
    model { var isOpen bool = false }
    view {
        preview-card (id: "example") {
            collapsible (open: .isOpen) {
                collapsible-trigger { text "Toggle" {} }
                collapsible-content { text "Open" {} }
            }
            slider (min: 2, max: 20, step: 3, value: 8) {}
        }
    }
}
"#;
        let original = project_component_candidates(source).expect("widget source");
        let tree = &original[0].view.tree;
        let mut state = State::new(initial_bools(&original[0].fields));
        assert!(contains_preview(tree));
        assert!(keyboard_action(tree, &[], &[], &state, "Enter").is_none());
        let mut press = |key| {
            let action = keyboard_action(tree, &[], &[], &state, key).expect(key);
            state.apply(action);
            state.evidence()
        };
        press("Tab");
        assert_eq!(press("Enter")["bindings"]["isOpen"], true);
        assert_eq!(press(" ")["bindings"]["isOpen"], false);
        press("Tab");
        for (key, value) in [
            ("ArrowRight", 11.0),
            ("End", 20.0),
            ("ArrowRight", 20.0),
            ("Home", 2.0),
            ("ArrowLeft", 2.0),
        ] {
            let result = press(key);
            assert_eq!(
                result["slider"]
                    .as_object()
                    .unwrap()
                    .values()
                    .next()
                    .unwrap()
                    .as_f64(),
                Some(value)
            );
        }
        press("Shift+Tab");
        assert_eq!(press("Enter")["bindings"]["isOpen"], true);
    }
}
