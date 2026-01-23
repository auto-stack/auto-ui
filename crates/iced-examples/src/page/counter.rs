use iced::Element;
use iced::widget::{column, text, button};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Counter Example").size(24),
        text(""),
        text("This demonstrates state management with buttons."),
        text(""),
        button(text("Increment")),
        text("Counter: 0").size(40),
        button(text("Decrement")),
    )
    .spacing(10)
    .padding(20)
    .into()
}