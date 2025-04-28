use super::Snip;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};

pub struct DynaButton {
    /// contains code like: button(self.button_label) { onclick: "click-event" }
    snip: Snip,
}

impl DynaButton {
    pub fn new(snip: Snip) -> Self {
        Self { snip }
    }

    pub fn draw<T>(&mut self, mut div: Div, w: &mut Window, cx: &mut Context<T>) -> Div {
        div.child(Button::new("button").label("button").primary())
    }
}
