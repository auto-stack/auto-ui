// Iced Adapter - Convert StyleClass IR to Iced style objects
//
// This adapter translates the unified StyleClass IR into Iced-specific
// style objects for styling components.

use crate::style::{Style, StyleClass, SizeValue, Color};

/// Iced style representation
///
/// Iced has a more traditional style system with separate Style, Theme, and layout objects.
/// This adapter converts StyleClass IR into Iced-compatible structures.
pub struct IcedStyle {
    pub padding: Option<f32>,
    pub gap: Option<f32>,
    pub background_color: Option<iced::Color>,
    pub text_color: Option<iced::Color>,
    pub width: Option<IcedSize>,
    pub height: Option<IcedSize>,
    pub rounded: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedSize {
    Full,
    Fixed(f32),
}

impl IcedStyle {
    /// Convert a Style to IcedStyle
    pub fn from_style(style: &Style) -> Self {
        let mut iced_style = IcedStyle {
            padding: None,
            gap: None,
            background_color: None,
            text_color: None,
            width: None,
            height: None,
            rounded: false,
        };

        for class in &style.classes {
            iced_style.apply_class(class);
        }

        iced_style
    }

    /// Apply a single StyleClass to this IcedStyle
    fn apply_class(&mut self, class: &StyleClass) {
        match class {
            StyleClass::Padding(size) => {
                self.padding = Some(size.to_pixels() as f32);
            }
            StyleClass::Gap(size) => {
                self.gap = Some(size.to_pixels() as f32);
            }
            StyleClass::BackgroundColor(color) => {
                self.background_color = Some(convert_color(color));
            }
            StyleClass::TextColor(color) => {
                self.text_color = Some(convert_color(color));
            }
            StyleClass::Width(size) => {
                self.width = Some(convert_size(size));
            }
            StyleClass::Height(size) => {
                self.height = Some(convert_size(size));
            }
            StyleClass::Rounded => {
                self.rounded = true;
            }
            // Layout-related styles (flex, items-center, etc.) are handled differently in Iced
            // They're applied through layout methods rather than style objects
            _ => {
                // Ignore layout classes for now - they're handled separately
            }
        }
    }

    /// Convert to iced::Style (for containers, buttons, etc.)
    pub fn to_container_style(&self) -> iced::style::Container {
        iced::style::Container::default()
    }
}

/// Convert a SizeValue to IcedSize
fn convert_size(size: &SizeValue) -> IcedSize {
    match size {
        SizeValue::Full => IcedSize::Full,
        SizeValue::Fixed(units) => IcedSize::Fixed(units.to_pixels() as f32),
        _ => IcedSize::Full, // Default to full for other variants
    }
}

/// Convert a Color to iced::Color
fn convert_color(color: &Color) -> iced::Color {
    match color {
        Color::Rgb { r, g, b } => {
            iced::Color::from_rgb(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0)
        }
        Color::Rgba { r, g, b, a } => {
            iced::Color::from_rgba(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, *a as f32 / 255.0)
        }
        Color::White => iced::Color::WHITE,
        Color::Black => iced::Color::BLACK,
        Color::Slate(shade) | Color::Gray(shade) | Color::Zinc(shade) | Color::Neutral(shade) => {
            // Grayscale colors
            let value = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(value, value, value)
        }
        Color::Red(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(intensity, 0.0, 0.0)
        }
        Color::Blue(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(0.0, 0.0, intensity)
        }
        Color::Green(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(0.0, intensity, 0.0)
        }
        Color::Yellow(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(intensity, intensity, 0.0)
        }
        _ => iced::Color::from_rgb(0.5, 0.5, 0.5), // Default gray (semantic colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_style() {
        let style = Style::parse("p-4 bg-white").unwrap();
        let iced_style = IcedStyle::from_style(&style);

        assert_eq!(iced_style.padding, Some(16.0));
    }

    #[test]
    fn test_convert_color() {
        let white = convert_color(&Color::White);
        assert_eq!(white.r, 1.0);
        assert_eq!(white.g, 1.0);
        assert_eq!(white.b, 1.0);
    }
}
