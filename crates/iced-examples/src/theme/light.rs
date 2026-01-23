use iced::application;
use iced::color;

/// 浅色主题
pub struct Light;

impl Light {
    pub fn default_appearance() -> application::Appearance {
        application::Appearance {
            background_color: color!(0xF3F3F3),
            text_color: color!(0x000000),
        }
    }
}
