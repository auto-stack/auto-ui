use iced::Element;
use iced::widget::{column, text, button};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Button Example").size(24),
        text(""),
        text("This demonstrates the button widget."),
        text(""),
        button(text("Primary Button")),
        text(""),
        text("Click the button to see interactions"),
    )
    .spacing(10)
    .padding(20)
    .into()
}
