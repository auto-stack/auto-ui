// Hello World example for iced
use iced::widget::{center, text};

fn main() -> iced::Result {
    iced::run(Hello::update, Hello::view)
}

#[derive(Default)]
struct Hello;

impl Hello {
    fn update(&mut self, _message: ()) {}

    fn view(&self) -> iced::Element<'_, ()> {
        center(text("Hello, World!").size(30)).into()
    }
}
