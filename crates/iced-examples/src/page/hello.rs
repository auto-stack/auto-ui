use iced::Element;
use iced::widget::{column, text};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Hello World Example").size(24),
        text(""),
        text("Hello, World!").size(40),
    )
    .spacing(10)
    .padding(20)
    .into()
}