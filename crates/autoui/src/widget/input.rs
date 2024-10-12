use gpui::*;
use crate::style::size::Size;

pub struct Input {
    focus_handle: FocusHandle,
    text: SharedString,
    size: Size,
}

impl FocusableView for Input {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Input {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
    }
}
