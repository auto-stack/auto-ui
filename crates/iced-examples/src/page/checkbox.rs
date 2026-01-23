use iced::Element;
use iced::widget::{column, text, checkbox};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Checkbox Example").size(24),
        text(""),
        text("This demonstrates the checkbox widget."),
        text(""),
        checkbox(false).label("Unchecked checkbox"),
        checkbox(true).label("Checked checkbox"),
    )
    .spacing(10)
    .padding(20)
    .into()
}