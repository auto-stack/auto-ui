mod app;

use app::*;
use gpui::*;


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
        .child(format!("Hello, {}!", self.label))
    }
}


fn main() {
    let app = SimpleApp::new();
    app.run(|cx| {
        cx.new_view(HelloWorldView::new)
    });
}