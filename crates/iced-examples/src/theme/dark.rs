use iced::application;
use iced::color;

/// 深色主题
pub struct Dark;

impl Dark {
    pub fn default_appearance() -> application::Appearance {
        application::Appearance {
            background_color: color!(0x1E1E1E),
            text_color: color!(0xFFFFFF),
        }
    }
}
