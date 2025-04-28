use super::Snip;
use gpui::*;
use gpui_component::label::Label;

pub struct DynaLabel {
    /// contains code like: button(self.button_label) { onclick: "click-event" }
    snip: Snip,
}

impl DynaLabel {
    pub fn new(snip: Snip) -> Self {
        Self { snip }
    }

    pub fn draw<T>(&mut self, mut div: Div, w: &mut Window, cx: &mut Context<T>) -> Div {
        div.child(Label::new("label"))
    }
}
