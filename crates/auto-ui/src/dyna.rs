mod snip;
pub use snip::*;
mod story;
pub use story::*;
mod button;
pub use button::*;
mod label;
pub use label::*;

use auto_val::AutoStr;
use auto_val::Shared;
use gpui::*;

pub enum DynaWidget {
    Button(DynaButton),
    Label(DynaLabel),
}

/// Class for a Dynamic Widget
pub struct Dyna {
    /// Code Snippet for how to draw this widget
    pub snip: Snip,
    // pub interp: Shared<Interpreter>,
    pub kids: Vec<DynaWidget>,
}

impl Dyna {
    pub fn new(mut snip: Snip, cx: &mut Context<'_, Self>) -> Self {
        let kids = Self::parse(&mut snip);
        Self { snip, kids }
    }

    pub fn parse(snip: &mut Snip) -> Vec<DynaWidget> {
        let kids = vec![
            DynaWidget::Button(DynaButton::new(snip.clone())),
            DynaWidget::Label(DynaLabel::new(snip.clone())),
        ];
        kids
    }
}

impl Render for Dyna {
    fn render(
        &mut self,
        w: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) -> impl gpui::IntoElement {
        let mut div = div().child(format!("Hello, {}!", self.snip.code));
        for k in &mut self.kids {
            match k {
                DynaWidget::Button(button) => {
                    div = button.draw(div, w, cx);
                }
                DynaWidget::Label(label) => {
                    div = label.draw(div, w, cx);
                }
            }
        }
        div
    }
}
