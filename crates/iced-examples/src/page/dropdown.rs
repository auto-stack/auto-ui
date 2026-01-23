use iced::Element;
use iced::widget::{column, text};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Dropdown Example").size(24),
        text(""),
        text("This demonstrates the dropdown widget."),
        text(""),
        text("Dropdown widget coming soon..."),
    )
    .spacing(10)
    .padding(20)
    .into()
}
