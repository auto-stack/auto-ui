use iced::Element;
use iced::widget::{column, text};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Slider Example").size(24),
        text(""),
        text("This demonstrates the slider widget."),
        text(""),
        text("Interactive slider coming soon..."),
    )
    .spacing(10)
    .padding(20)
    .into()
}
