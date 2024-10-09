use autoui::app::SimpleApp;
use autoui_theme::theme::ActiveTheme;
use autoui::app::Viewable;
use gpui::*;

#[derive(Clone)]
struct HelloWorldView {
    label: String,
}

impl HelloWorldView {
    fn new() -> Self {
        Self {
            label: "world".into(),
        }
    }
}

impl Viewable for HelloWorldView {
    fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| HelloWorldView::new())
    }
}

impl Render for HelloWorldView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child(format!("Hello, {}!", self.label))
    }
}

fn main() {
    // SimpleApp::new().run(|cx| cx.new_view(HelloWorldView::new));
    SimpleApp::new().run_simple::<HelloWorldView>();
}
