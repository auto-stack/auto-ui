mod pen;
mod snip;

use auto_val::AutoStr;
use gpui::Render;
pub use pen::*;
pub use snip::*;

/// Class for Dyna Widget
pub struct Dyna {
    /// Pen for how to draw this widget
    pub pen: Pen,
}

impl Dyna {
    pub fn new() -> Self {
        let snip = Snip {
            code: "MYSNIP".into(),
        };
        let pen = Pen { snip };
        Self { pen }
    }
}

impl Render for Dyna {
    fn render(
        &mut self,
        w: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) -> impl gpui::IntoElement {
        self.pen.draw(w, cx)
    }
}
