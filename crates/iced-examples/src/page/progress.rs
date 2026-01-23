use iced::Element;
use iced::widget::{column, text, progress_bar};

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Progress Bar Example").size(24),
        text(""),
        text("This demonstrates the progress bar widget:"),
        text("0%"),
        progress_bar(0.0..=1.0, 0.0),
        text(""),
        text("50%"),
        progress_bar(0.0..=1.0, 0.5),
        text(""),
        text("100%"),
        progress_bar(0.0..=1.0, 1.0),
    )
    .spacing(10)
    .padding(20)
    .into()
}