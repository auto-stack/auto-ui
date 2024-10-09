use gpui::*;
use autoui_widgets::Button;
use autoui_theme::theme;

struct CounterView {
    count: u32,
}

impl CounterView {
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
            .bg(rgb(0xeeeeee))
            .size_full()
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0xcccccc))
            .text_xl()
            .text_color(rgb(0x333333))
            .child(format!("Count: {}", self.count))
            .gap_1()
            .child(Button::new("Increment".into()).on_click(
                cx.listener(|view, _ev, cx| {
                    view.count += 1;
                    cx.notify();
                })
            ))
    }
}

fn main() {
    App::new().run(|cx| {
        theme::init(cx);
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(CounterView::new)
        })
        .unwrap();
    });
}

