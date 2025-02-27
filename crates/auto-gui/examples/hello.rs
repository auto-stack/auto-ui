use auto_gui::*;
use gpui_widgets::*;
use gpui::*;

fn main() {
    AutoGuiApp::new().run(|cx| PaneView::Center(vec![
        KidView::View(cx.new(|_| TextView::new("Hello, World!")).into()),
        KidView::View(cx.new(|_| TextView::new("Hello, Universe!")).into()),
    ]));
}
