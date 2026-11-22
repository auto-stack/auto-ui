use super::{element_children, literal, Action, NodeDesktopMsg, Renderer, WidgetView};
use crate::ui::view::{PopoverAnchor, PopoverPlacement, SelectCallback};
use crate::ui::{Style, View};
use ace_core::source::age_component::{ComponentViewElement, ComponentViewItem};

fn family(tag: &str) -> Option<&'static str> {
    match tag {
        "alert-dialog" | "alert_dialog" => Some("alert-dialog"),
        "drawer" => Some("drawer"),
        "dropdown-menu" | "dropdown_menu" => Some("dropdown-menu"),
        "hover-card" | "hover_card" => Some("hover-card"),
        "popover" => Some("popover"),
        "sheet" => Some("sheet"),
        _ => None,
    }
}

pub(super) fn contains_panel(items: &[ComponentViewItem]) -> bool {
    items.iter().any(|item| match item {
        ComponentViewItem::Element(element) => {
            (family(&element.tag).is_some() && literal_panel(element))
                || contains_panel(&element.items)
        }
        _ => false,
    })
}

fn literal_panel(element: &ComponentViewElement) -> bool {
    use ace_core::source::{age_component::ComponentEntryValue, ast::ExprKind};
    element.items.iter().all(|item| match item {
        ComponentViewItem::Text(text) => matches!(text.expr.kind, ExprKind::StrLit(_)),
        ComponentViewItem::Element(child) => literal_panel(child),
        ComponentViewItem::Entry(entry) => {
            !entry.is_event
                && matches!(
                    &entry.value,
                    ComponentEntryValue::Expr(expr) if matches!(
                        expr.kind, ExprKind::StrLit(_) | ExprKind::BoolLit(_) | ExprKind::IntLit(_)
                    )
                )
        }
        ComponentViewItem::Cond(_) | ComponentViewItem::For(_) => false,
    })
}

impl Renderer<'_> {
    pub(super) fn render_panel(&mut self, el: &ComponentViewElement) -> Option<WidgetView> {
        let kind = family(&el.tag)?;
        if !literal_panel(el) {
            return None;
        }
        let key = el.span.start;
        let trigger_tag = format!("{kind}-trigger");
        let content_tag = format!("{kind}-content");
        let children = element_children(el).collect::<Vec<_>>();
        let trigger = children
            .iter()
            .copied()
            .find(|child| child.tag.replace('_', "-") == trigger_tag);
        let body = children
            .iter()
            .copied()
            .find(|child| child.tag.replace('_', "-") == content_tag);
        let open = self.state.disclosure.get(&key).copied().unwrap_or(false);
        let open_action = Action::Collapsible {
            key,
            field: None,
            open: true,
        };
        let close_action = Action::Collapsible {
            key,
            field: None,
            open: false,
        };

        let anchor = trigger.map_or_else(View::empty, |trigger| {
            let button = element_children(trigger)
                .find(|child| child.tag == "button")
                .unwrap_or(trigger);
            let label = literal(button, "text")
                .map(str::to_owned)
                .unwrap_or_else(|| self.label(button));
            let classes = [
                super::style(button),
                "px-3 py-2 rounded-md border border-border".into(),
            ]
            .join(" ");
            self.button(
                key,
                label,
                &classes,
                Some(if open {
                    close_action.clone()
                } else {
                    open_action.clone()
                }),
                Some(open),
                None,
            )
        });
        let placement = match (kind, literal(el, "direction"), literal(el, "placement")) {
            ("alert-dialog", _, _) => PopoverPlacement::Modal,
            ("drawer" | "sheet", Some("left"), _) => PopoverPlacement::EdgeLeft,
            ("drawer" | "sheet", Some("right"), _) => PopoverPlacement::EdgeRight,
            ("drawer", Some("top"), _) => PopoverPlacement::EdgeTop,
            ("drawer", _, _) => PopoverPlacement::EdgeBottom,
            ("sheet", _, _) => PopoverPlacement::EdgeRight,
            ("dropdown-menu", _, _) | (_, _, Some("bottom-start")) => PopoverPlacement::BottomStart,
            (_, _, Some("top")) => PopoverPlacement::Top,
            (_, _, Some("left")) => PopoverPlacement::Left,
            (_, _, Some("right")) => PopoverPlacement::Right,
            _ => PopoverPlacement::Bottom,
        };

        // Advance the checked observation cursor through both the closed and
        // open shapes; closed controls contribute no focus targets.
        let first_target = self.targets.len();
        let panel = self.panel_close.replace(key);
        let contents = body.map(|body| self.items(&body.items)).unwrap_or_default();
        self.panel_close = panel;
        if !open {
            self.targets.truncate(first_target);
        }
        let chrome = if placement.is_modal_chrome() {
            "w-96 gap-4 p-6 rounded-lg border bg-background"
        } else {
            "w-52 gap-2 p-3 rounded-md border bg-popover"
        };
        let panel = View::col().style(chrome).children(contents).build();
        let anchor = if kind == "hover-card" {
            View::MouseArea {
                content: Box::new(anchor),
                on_enter: Some(self.action(open_action)),
                on_exit: Some(self.action(close_action.clone())),
                on_click: None,
                on_double_click: None,
                on_context_menu: None,
                on_release: None,
                on_move: None,
                logical_extent: None,
                style: None,
            }
        } else {
            anchor
        };
        Some(View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(anchor)),
            content: Box::new(panel),
            placement,
            open,
            on_dismiss: (kind != "alert-dialog").then(|| self.action(close_action)),
        })
    }

    pub(super) fn panel_close_button(
        &mut self,
        element: &ComponentViewElement,
        classes: &str,
    ) -> Option<WidgetView> {
        let key = self.panel_close?;
        if !matches!(
            element.tag.as_str(),
            "alert-dialog-action"
                | "alert_dialog_action"
                | "alert-dialog-cancel"
                | "alert_dialog_cancel"
                | "dropdown-menu-item"
                | "dropdown_menu_item"
        ) {
            return None;
        }
        let label = self.label(element);
        let classes = format!("px-3 py-2 rounded-md border border-border {classes}");
        Some(self.button(
            element.span.start,
            label,
            &classes,
            Some(Action::Collapsible {
                key,
                field: None,
                open: false,
            }),
            None,
            None,
        ))
    }

    pub(super) fn native_select(&self, el: &ComponentViewElement, style: Style) -> WidgetView {
        let key = el.span.start;
        let placeholder = element_children(el)
            .find(|child| matches!(child.tag.as_str(), "select-trigger" | "select_trigger"))
            .into_iter()
            .flat_map(element_children)
            .find(|child| matches!(child.tag.as_str(), "select-value" | "select_value"))
            .and_then(|child| literal(child, "placeholder"))
            .or_else(|| literal(el, "placeholder"))
            .map(str::to_owned);
        let options = element_children(el)
            .find(|child| matches!(child.tag.as_str(), "select-content" | "select_content"))
            .into_iter()
            .flat_map(element_children)
            .filter_map(|child| literal(child, "text"))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let enabled = !options.is_empty()
            && !super::flag(el, "disabled")
            && literal(el, "disabled") != Some("true");
        let (session, instance) = (self.session, self.instance);
        View::Select {
            options,
            selected_index: self.state.selected.get(&key).copied(),
            placeholder,
            on_select: enabled.then(|| {
                SelectCallback::new(move |index, _| NodeDesktopMsg::Widget {
                    session,
                    instance,
                    action: Action::Select { key, index },
                })
            }),
            style: Some(style),
        }
    }

    pub(super) fn native_textarea(
        &self,
        element: &ComponentViewElement,
        mut style: Style,
    ) -> WidgetView {
        let key = element.span.start;
        let value = self
            .state
            .inputs
            .get(&key)
            .map(String::as_str)
            .or_else(|| literal(element, "value"))
            .unwrap_or_default();
        if style.classes.is_empty() {
            style = Style::parse("w-48").unwrap_or_default();
        }
        let rows = self.number(element, "rows", 3.0).max(1.0) as u16;
        View::textarea(literal(element, "placeholder").unwrap_or_default())
            .value(value)
            .height(rows.saturating_mul(24).saturating_add(16))
            .with_style(style)
            .disabled(super::flag(element, "disabled"))
            .on_change(self.action(Action::Textarea { key }))
            .build()
    }

    pub(super) fn native_input(&self, el: &ComponentViewElement, mut parsed: Style) -> WidgetView {
        let key = el.span.start;
        let (session, instance) = (self.session, self.instance);
        let value = self
            .state
            .inputs
            .get(&key)
            .map(String::as_str)
            .or_else(|| literal(el, "value"))
            .unwrap_or_default();
        if parsed.classes.is_empty() {
            parsed = Style::parse("w-40").unwrap_or_default();
        }
        let mut input = View::input(literal(el, "placeholder").unwrap_or_default())
            .value(value)
            .disabled(super::flag(el, "disabled"))
            .with_style(parsed)
            .on_text(move |text| NodeDesktopMsg::Widget {
                session,
                instance,
                action: Action::Input { key, text },
            });
        if literal(el, "type") == Some("password") {
            input = input.password();
        }
        input.build()
    }
}
