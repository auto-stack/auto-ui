use iced::Element;
use iced::widget::{column, text, text_input};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Select / Dropdown Example").size(24),
        text(""),
        text("This demonstrates dropdown selection."),
        text(""),
        text("Dropdown widget coming soon..."),
    )
    .spacing(10)
    .padding(20)
    .into()
}
