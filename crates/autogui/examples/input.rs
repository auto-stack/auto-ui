use gpui::*;
use autogui::widget::input::TextInput;
use autogui::app::SimpleApp;
use autogui::app::Viewable;
use autogui::widget::button::Button;
struct InputView {
    label: SharedString,
    text_input: View<TextInput>,
}

impl Viewable for InputView {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            label: "world".into(),
            text_input: cx.new_view(|cx| TextInput::new(cx)),
        }
    }
}

impl Render for InputView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child(format!("Hello, {}!", self.label))
            .gap_1()
            .child(self.text_input.clone())
            .gap_1()
            .child(Button::button("Click me").on_click_mut(cx, |this, _ev, cx| {
                this.label = this.text_input.read(cx).text().clone();
                cx.notify();
            }))
    }
}

fn main() {
    SimpleApp::new().title("Input Example").run_simple::<InputView>();
}

