use gpui::*;
use autoui::widget::checkbox::Checkbox;
use autoui::app::SimpleApp;
use autoui::app::Viewable;

struct CheckboxView {
    label: String,
    checked: bool,
}

impl Viewable for CheckboxView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            label: "world".into(),
            checked: false,
        }
    }
}

impl Render for CheckboxView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child(format!("Hello, {}!", self.label))
            .gap_1()
            .child(
                Checkbox::new("cb1")
                    .checked(self.checked)
                    .on_click_mut(cx, |view, new_checked, cx| {
                        view.checked = *new_checked;
                        cx.notify();
                    })
            )
    }
}

fn main() {
    SimpleApp::new().title("Checkbox Example").run_simple::<CheckboxView>();
}

