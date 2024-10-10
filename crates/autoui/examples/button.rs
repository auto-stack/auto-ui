use gpui::*;
use autoui::widget::Button;
use autoui::app::SimpleApp;
use autoui::app::Viewable;

struct ButtonView {
    label: String,
}

impl Viewable for ButtonView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            label: "world".into(),
        }
    }
}

impl Render for ButtonView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child(format!("Hello, {}!", self.label))
            .gap_1()
            .child(Button::new("Click Me".into()).on_click_mut(cx, |view, _ev, cx| {
                view.label = "Button".into();
                cx.notify();
            }))
    }
}

fn main() {
    SimpleApp::new().run_simple::<ButtonView>();
}

