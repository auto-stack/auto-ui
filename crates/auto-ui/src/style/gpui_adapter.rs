// GPUI Adapter - Convert StyleClass IR to GPUI builder methods
//
// This adapter translates the unified StyleClass IR into GPUI-specific
// builder method calls for styling components.

use crate::style::{Style, StyleClass, SizeValue, Color};

/// Apply a Style to a GPUI component builder
///
/// This is a trait that can be implemented for different GPUI builder types.
/// For the MVP prototype, we'll provide helper functions that return
/// style parameters that can be applied to GPUI builders.

pub struct GpuiStyle {
    // For GPUI, styles are typically applied via builder methods
    // This struct collects the style parameters
    pub padding: Option<GpuiPadding>,
    pub gap: Option<f32>,
    pub flex: Option<bool>,
    pub flex_direction: Option<GpuiFlexDirection>,
    pub items_align: Option<GpuiAlignment>,
    pub justify_align: Option<GpuiAlignment>,
    pub width: Option<GpuiSize>,
    pub height: Option<GpuiSize>,
    pub background_color: Option<gpui::Rgba>,
    pub text_color: Option<gpui::Rgba>,
    pub rounded: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiPadding {
    Uniform(f32), // p-4 = 16px
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiFlexDirection {
    Row,
    Col,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiAlignment {
    Center,
    Between,
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiSize {
    Full,
    Fixed(f32),
}

impl GpuiStyle {
    /// Convert a Style to GpuiStyle
    pub fn from_style(style: &Style) -> Self {
        let mut gpui_style = GpuiStyle {
            padding: None,
            gap: None,
            flex: None,
            flex_direction: None,
            items_align: None,
            justify_align: None,
            width: None,
            height: None,
            background_color: None,
            text_color: None,
            rounded: false,
        };

        for class in &style.classes {
            gpui_style.apply_class(class);
        }

        gpui_style
    }

    /// Apply a single StyleClass to this GpuiStyle
    fn apply_class(&mut self, class: &StyleClass) {
        match class {
            StyleClass::Padding(size) => {
                self.padding = Some(GpuiPadding::Uniform(size.to_pixels() as f32));
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
            StyleClass::Flex => {
                self.flex = Some(true);
            }
            StyleClass::FlexRow => {
                self.flex = Some(true);
                self.flex_direction = Some(GpuiFlexDirection::Row);
            }
            StyleClass::FlexCol => {
                self.flex = Some(true);
                self.flex_direction = Some(GpuiFlexDirection::Col);
            }
            StyleClass::ItemsCenter => {
                self.items_align = Some(GpuiAlignment::Center);
            }
            StyleClass::JustifyCenter => {
                self.justify_align = Some(GpuiAlignment::Center);
            }
            StyleClass::JustifyBetween => {
                self.justify_align = Some(GpuiAlignment::Between);
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
        }
    }
}

/// Convert a SizeValue to GpuiSize
fn convert_size(size: &SizeValue) -> GpuiSize {
    match size {
        SizeValue::Full => GpuiSize::Full,
        SizeValue::Fixed(units) => GpuiSize::Fixed(units.to_pixels() as f32),
        _ => GpuiSize::Full, // Default to full for other variants
    }
}

/// Convert a Color to gpui::Rgba
fn convert_color(color: &Color) -> gpui::Rgba {
    match color {
        Color::Rgb { r, g, b } => {
            gpui::Rgba { r: *r as f32 / 255.0, g: *g as f32 / 255.0, b: *b as f32 / 255.0, a: 1.0 }
        }
        Color::Rgba { r, g, b, a } => {
            gpui::Rgba { r: *r as f32 / 255.0, g: *g as f32 / 255.0, b: *b as f32 / 255.0, a: *a as f32 / 255.0 }
        }
        Color::White => gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
        Color::Black => gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
        Color::Slate(shade) | Color::Gray(shade) | Color::Zinc(shade) | Color::Neutral(shade) => {
            // Grayscale colors
            let value = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: value, g: value, b: value, a: 1.0 }
        }
        Color::Red(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: intensity, g: 0.0, b: 0.0, a: 1.0 }
        }
        Color::Blue(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: 0.0, g: 0.0, b: intensity, a: 1.0 }
        }
        Color::Green(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: 0.0, g: intensity, b: 0.0, a: 1.0 }
        }
        Color::Yellow(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: intensity, g: intensity, b: 0.0, a: 1.0 }
        }
        _ => gpui::Rgba { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }, // Default gray (semantic colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_style() {
        let style = Style::parse("p-4 bg-white flex").unwrap();
        let gpui_style = GpuiStyle::from_style(&style);

        assert!(gpui_style.flex.unwrap());
        assert_eq!(gpui_style.padding, Some(GpuiPadding::Uniform(16.0)));
    }

    #[test]
    fn test_convert_color() {
        let white = convert_color(&Color::White);
        assert_eq!(white.r, 1.0);
        assert_eq!(white.g, 1.0);
        assert_eq!(white.b, 1.0);
    }
}
