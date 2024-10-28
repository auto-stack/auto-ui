use autogui::app::SimpleApp;
use autogui::app::Viewable;
use autogui::widget::icon::SysIcon;
use gpui::*;

struct IconView {}

impl Viewable for IconView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {}
    }
}

impl Render for IconView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child("Icon Demo:")
            .gap_1()
            .child(SysIcon::Check.icon())
            .child(SysIcon::Check.icon().color(rgb(0xFF0000)))
            .child(SysIcon::Check.icon().color(rgb(0x00FF00)))
            .child(SysIcon::Sun.icon().size(Rems(4.)).color(red()))
            .child(SysIcon::Moon.icon().size(Rems(4.)).color(blue()))
    }
}

fn main() {
    SimpleApp::new()
        .title("Icon Example")
        .run_simple::<IconView>();
}
