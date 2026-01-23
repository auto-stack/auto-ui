use iced::Element;
use iced::widget::{column, text};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Layout Example").size(24),
        text(""),
        text("This demonstrates various layout options:"),
        text(""),
        text("Column layout:"),
        text("• Item 1"),
        text("• Item 2"),
        text("• Item 3"),
    )
    .spacing(10)
    .padding(20)
    .into()
}
