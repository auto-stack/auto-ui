use gpui::*;
use autoui::widget::icon::{Icon, SysIcon};
use autoui::app::SimpleApp;
use autoui::app::Viewable;
use autoui::theme::ActiveTheme;

struct IconView {
}

impl Viewable for IconView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
        }
    }
}

impl Render for IconView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let icon = Icon::from(SysIcon::Check);
        div()
            .child("Icon Demo:")
            .gap_1()
            .child(Icon::from(SysIcon::Check))
            .child(Icon::from(SysIcon::Check).color(rgb(0xFF0000)))
            .child(Icon::from(SysIcon::Check).color(rgb(0x00FF00)))
            .child(Icon::from(SysIcon::Check).color(rgb(0x0000FF)))
            .child(Icon::from(SysIcon::Sun).size(Rems(4.)).color(red()))
            .child(Icon::from(SysIcon::Moon).size(Rems(4.)).color(blue()))
    }
}

fn main() {
    SimpleApp::new().title("Icon Example").run_simple::<IconView>();
}

