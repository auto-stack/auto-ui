use iced::Element;
use iced::widget::{column, text};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Todos Example").size(24),
        text(""),
        text("This demonstrates a todo list application."),
        text(""),
        text("• Task 1"),
        text("• Task 2"),
        text("• Task 3"),
    )
    .spacing(10)
    .padding(20)
    .into()
}
