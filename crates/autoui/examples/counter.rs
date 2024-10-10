use gpui::*;
use autoui::widget::Button;
use autoui::app::SimpleApp;
use autoui::app::Viewable;

struct CounterView {
    count: i32,
}

impl Viewable for CounterView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            count: 0,
        }
    }
}

impl Render for CounterView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .child(Button::new("+".into()).on_click_mut(cx, |this, _ev, cx| {
                this.count += 1;
                cx.notify();
            }))
            .gap_1()
            .child(format!("Count: {}", self.count))
            .gap_1()
            .child(Button::new("-".into()).on_click_mut(cx, |this, _ev, cx| {
                this.count -= 1;
                cx.notify();
            }))
    }
}

fn main() {
    SimpleApp::new().run_simple::<CounterView>();
}

