use autogui::app::SimpleApp;
use autogui::app::Viewable;
use gpui::*;

#[derive(Clone)]
struct HelloWorldView {
    label: String,
}

impl Viewable for HelloWorldView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        HelloWorldView {
            label: "world".into(),
        }
    }
}

impl Render for HelloWorldView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(format!("Hello, {}!", self.label))
    }
}

fn main() {
    SimpleApp::new().title("Hello World Example").run_simple::<HelloWorldView>();
}
