use auto_gui::*;
use gpui_widgets::*;
use auto_widgets::*;

fn main() {
    AutoGuiApp::new().run(|_cx| PaneView::new(Pane::Center(vec![
        Kid::Widget(Widget::Text(Text::new("Hello, World!"))),
        Kid::Widget(Widget::Text(Text::new("Hello, Universe!"))),
    ])));
}
