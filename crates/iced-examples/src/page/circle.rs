use iced::Element;
use iced::widget::{column, text, container};
use iced::Length;

pub fn view() -> Element<'static, crate::Message> {
    column!(
        text("Circle Example").size(24),
        text(""),
        text("This demonstrates shapes and styling."),
        text(""),
        container(
            text("⭕")
                .size(100)
        )
        .width(Length::Fixed(120.0))
        .height(Length::Fixed(120.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    )
    .spacing(10)
    .padding(20)
    .into()
}
