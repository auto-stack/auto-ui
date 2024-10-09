use gpui::*;
use autoui_widgets::Button;

struct HelloWorldView {
    label: String,
}

impl HelloWorldView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            label: "world".into(),
        }
    }
}

impl Render for HelloWorldView {
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
            .child(format!("Hello, {}!", self.label))
            .gap_1()
            .child(Button::new("Click Me".into()).on_click(
                cx.listener(|view, _ev, cx| {
                    view.label = "Button".into();
                    cx.notify();
                })
            ))
    }
}

fn main() {
    App::new().run(|cx| {
        autoui_theme::theme::init(cx);
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(HelloWorldView::new)
        })
        .unwrap();
    });
}
