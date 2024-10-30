use crate::style::theme::ActiveTheme;
use super::list::List;
use super::list::ListEvent;
use crate::widget::util::*;
use crate::widget::icon::{SysIcon, Icon};
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct Dropdown {
    id: ElementId,
    focus_handle: FocusHandle,
    list: View<List>,
    bounds: Bounds<Pixels>,
    is_open: bool,
}

actions!(dropdown, [Up, Down, Enter, Escape]);

impl Dropdown {
    pub fn new(id: impl Into<ElementId>, items: Vec<SharedString>, selected: Option<usize>, cx: &mut ViewContext<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let list = cx.new_view(|cx| List::new(cx, items).select(selected.unwrap_or(0))); 

        cx.subscribe(&list, Self::on_list_event).detach();
        
        cx.on_blur(&list.focus_handle(cx), Self::on_blur).detach();
        cx.on_blur(&focus_handle, Self::on_blur).detach();
        Self {
            id: id.into(),
            focus_handle,
            list,
            bounds: Bounds::default(),
            is_open: false,
        }
    }

    pub fn toggle(&mut self, _ev: &ClickEvent, _cx: &mut ViewContext<Self>) {
        println!("toggle on dropdown!");
        self.is_open = !self.is_open;
    }

    fn on_blur(&mut self, cx: &mut ViewContext<Self>) {

        // When the dropdown and dropdown menu are both not focused, close the dropdown menu.
        if self.list.focus_handle(cx).is_focused(cx) || self.focus_handle.is_focused(cx) {
            return;
        }

        self.is_open = false;
        cx.notify();

    }

    fn on_list_event(&mut self, _list: View<List>, ev: &ListEvent, cx: &mut ViewContext<Self>) {
        match ev {
            ListEvent::Selected(i) => {
                self.list.update(cx, |list, _cx| list.update_selected(*i));
                self.is_open = false;
                cx.notify();
            }
        }
    }
}

impl Render for Dropdown {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();

        div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .relative()
            .min_w(px(100.0))
            .child(
                div()
                    .id("dropdown-head")
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .bg(theme.background)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(px(theme.radius))
                    .overflow_hidden()
                    .cursor_pointer()
                    .w_full()
                    .on_click(cx.listener(Self::toggle))
                    .px_4()
                    .child(self.list.read(cx).selected_text())
                    .map(|this| {
                        let icon = if self.is_open {
                            SysIcon::ArrowUp.icon()
                        } else {
                            SysIcon::ArrowDown.icon()
                        };
                        this.child(icon)
                    })
            )
            .when(self.is_open, |this| {
                this.child(deferred(
                    anchored().snap_to_window_with_margin(px(8.)).child(
                        div().id("dropdown-list").occlude().w(px(200.0)).child(
                            col()
                                .occlude()
                                .mt_1p5()
                                .bg(theme.background)
                                .border_1()
                                .border_color(theme.border)
                                .rounded(px(theme.radius))
                                .shadow_md()
                                .on_mouse_down_out(cx.listener(move |this, _ev, cx| {
                                    this.is_open = false;
                                    cx.notify();
                                }))
                                .child(self.list.clone()),
                        ),
                    ),
                ))
            })
    }
}

pub trait DropItem {
    type Value: Clone;
    fn text(&self) -> SharedString;
    fn value(&self) -> &Self::Value;
}

impl DropItem for String {
    type Value = Self;

    fn text(&self) -> SharedString {
        SharedString::from(self.to_string())
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}

impl DropItem for SharedString {
    type Value = Self;

    fn text(&self) -> SharedString {
        self.clone()
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}
