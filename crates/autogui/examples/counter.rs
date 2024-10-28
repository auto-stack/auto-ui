use gpui::*;
use autogui::widget::button::*;
use autogui::app::SimpleApp;
use autogui::app::Viewable;

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
            .child(Button::primary("+".into()).on_click_mut(cx, |this, _ev, cx| {
                this.count += 1;
                cx.notify();
            }))
            .gap_1()
            .child(format!("Count: {}", self.count))
            .gap_1()
            .child(Button::primary("-".into()).on_click_mut(cx, |this, _ev, cx| {
                this.count -= 1;
                cx.notify();
            }))
    }
}

fn main() {
    SimpleApp::new().title("Counter Example").run_simple::<CounterView>();
}

