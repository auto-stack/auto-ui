use autogui::app::SimpleApp;
use autogui::widget::button::Button;
use autogui::widget::label::Label;
use gpui::*;

fn main() {
    SimpleApp::new().run_elems(|div| {
        div.child(Label::new("Hello, world 1!"))
            .child(Label::new("Hello, world 2!"))
            .child(Button::button("Click me!").on_click(|_ev, _cx| {
                println!("Button clicked!");
            }))
    });
}
