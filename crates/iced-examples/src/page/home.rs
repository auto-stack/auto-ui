use iced::{Element};
use iced::widget::{container, text};

/// 主页视图
pub fn view() -> Element<'static, crate::Message> {
    container(
        text("欢迎来到 Iced Gallery")
            .size(40)
    )
    .padding(20)
    .into()
}
