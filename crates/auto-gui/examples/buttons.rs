use auto_gui::*;
use auto_widgets::*;
use gpui_widgets::*;


fn main() {
    AutoGuiApp::new().run(|_cx| PaneView::new(Pane::Center(vec![
        Kid::Widget(Widget::Text(Text::new("Hello, World!"))),
        Kid::Widget(Widget::Button(Button::new("Click me"))),
    ])));
}
