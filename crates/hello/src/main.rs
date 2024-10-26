use autoui::app::SimpleApp;
use autoui::baseui::*;
use autoui::app::Viewable;
use autoui::widget::button::Button;

#[derive(Clone)]
struct HelloView {
    label: String,
}

impl Viewable for HelloView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self { label: "world".into() }
    }
}

impl Render for HelloView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child(format!("Hello now!, {}!", self.label))
            .child(Button::new().label("Click me!".into()).on_click(|_cx, _ev| {
                println!("Button clicked!");
            }))
    }
}

fn main() {
    println!("Hello, world!");
    
    SimpleApp::new().title("Hello World Example").run_simple::<HelloView>();
}


