use iced::application;
use iced::color;

use super::Light;
use super::Dark;

/// 主题枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl application::StyleSheet for Theme {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        match style {
            Theme::Light => Light::default_appearance(),
            Theme::Dark => Dark::default_appearance(),
        }
    }
}
