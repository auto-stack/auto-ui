use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::style::theme::ActiveTheme;
use crate::widget::util::*;

pub struct DropList {
    focus_handle: FocusHandle,
    items: Vec<SharedString>,
}

impl DropList {
    pub fn new(cx: &mut WindowContext, items: Vec<SharedString>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            focus_handle,
            items,
        }
    }
}

impl Render for DropList {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child(format!("List: {:?}", self.items))
    }
}

pub struct Dropdown {
    id: ElementId,
    focus_handle: FocusHandle,
    list: View<DropList>,
    bounds: Bounds<Pixels>,
    is_open: bool,
}

actions!(dropdown, [Up, Down, Enter, Escape]);

impl Dropdown {
    pub fn new(id: impl Into<ElementId>, items: Vec<SharedString>, cx: &mut WindowContext) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            id: id.into(),
            focus_handle,
            list: cx.new_view(|cx| DropList::new(cx, items)),
            bounds: Bounds::default(),
            is_open: false,
        }
    }
}

impl Render for Dropdown {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();

        div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .size_full()
            .relative()
            .child(
                div()
                    .id("dropdown-head")
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(theme.background)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(px(theme.radius))
                    .overflow_hidden()
                    .w_full()
                    .on_click(cx.listener(self.toggle))
                    .child(
                        "This is a dropdown"
                    )
            )
            .when(self.is_open, |this| {
                this.child(deferred(
                    anchored().snap_to_window_with_margin(px(8.)).child(
                        div().id("dropdown-list")
                            .occlude()
                            .w(self.bounds.size.width)
                            .child(
                                col()
                                    .occlude()
                                    .mt_1p5()
                                    .bg(theme.background)
                                    .border_1()
                                    .border_color(theme.border)
                                    .rounded(px(theme.radius))
                                    .shadow_md()
                                    .on_mouse_down_out(|_, cx| {
                                        cx.dispatch_action(Box::new(Escape));
                                    })
                                    .child(self.list.clone())
                            )
                    )
                ))
            })
    }
}

pub trait DropItem{
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