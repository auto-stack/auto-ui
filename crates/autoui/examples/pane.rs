use gpui::*;
use autoui::widget::pane::*;
use autoui::app::SimpleApp;
use autoui::app::Viewable;

struct PaneView {
    pane: View<Pane>,
}

struct ContentView {
    content: String,
}

impl Render for ContentView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(self.content.clone())
    }
}

impl Viewable for PaneView {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        let content = ContentView { content: "Content".to_string() };
        let pane = Pane::new(PaneSide::Left, Pixels(200.0)).child(cx.new_view(|_cx| content));
        Self {
            pane: cx.new_view(|_cx| pane),
        }
    }
}

impl Render for PaneView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(self.pane.clone())
    }
}

fn main() {
    SimpleApp::new("Pane Example").run_simple::<PaneView>();
}
