use crate::Dyna;
use crate::Snip;
use gpui::*;

pub struct Pen {
    // ui code for this pen to draw
    pub snip: Snip,
}

impl Pen {
    pub fn new(snip: Snip) -> Self {
        Self { snip }
    }
}

impl Pen {
    pub fn draw(
        &self,
        w: &mut gpui::Window,
        cx: &mut gpui::Context<Dyna>,
    ) -> impl gpui::IntoElement {
        div().child(format!("Hello, {}!", self.snip.code))
    }
}
