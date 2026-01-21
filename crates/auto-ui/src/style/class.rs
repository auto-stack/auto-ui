// StyleClass - Intermediate Representation (IR) for style classes
//
// This enum represents the parsed form of Tailwind-style utility classes.
// It is backend-agnostic and can be translated to GPUI, Iced, or other backends.

use crate::style::Color;

/// Size value (used for width, height, spacing, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeValue {
    Full,        // 100%
    Half,        // 50%
    Third,       // 33.333%
    TwoThirds,   // 66.666%
    Quarter,     // 25%
    ThreeQuarters, // 75%
    Auto,
    Fixed(u16),  // Pixels (Tailwind uses 4px base unit, so 1 = 4px, 2 = 8px, etc.)
}

impl SizeValue {
    /// Convert Tailwind spacing unit to pixels (1 unit = 4px)
    pub fn to_pixels(&self) -> u16 {
        match self {
            SizeValue::Fixed(units) => units * 4,
            _ => 0, // Full, Auto, etc. are handled differently by backends
        }
    }
}

/// Style class IR - represents a single parsed style property
///
/// This enum contains only L1 Core features for the MVP prototype:
/// - Spacing: p-*, gap-*
/// - Colors: bg-*, text-*
/// - Layout: flex, flex-row/col, items-center
/// - Sizing: w-full, w-*, h-full, h-*
/// - Border Radius: rounded
#[derive(Debug, Clone, PartialEq)]
pub enum StyleClass {
    // ========== Spacing (L1 Core) ==========
    /// Padding: p-{0-12} (p-0, p-1, ..., p-12)
    Padding(SizeValue),

    /// Gap: gap-{0-12} (gap-0, gap-1, ..., gap-12)
    Gap(SizeValue),

    // ========== Colors (L1 Core) ==========
    /// Background color: bg-{color}
    BackgroundColor(Color),

    /// Text color: text-{color}
    TextColor(Color),

    // ========== Layout (L1 Core) ==========
    /// Flex container
    Flex,

    /// Flex direction: row (default)
    FlexRow,

    /// Flex direction: column
    FlexCol,

    /// Items center alignment
    ItemsCenter,

    /// Justify center
    JustifyCenter,

    /// Justify between
    JustifyBetween,

    // ========== Sizing (L1 Core) ==========
    /// Width: w-{size}
    Width(SizeValue),

    /// Height: h-{size}
    Height(SizeValue),

    // ========== Border Radius (L1 Core) ==========
    /// Border radius: rounded
    Rounded,
}

impl StyleClass {
    /// Parse a single style class string into a StyleClass
    pub fn parse_single(class: &str) -> Result<Self, String> {
        let class = class.trim();

        // Skip empty strings
        if class.is_empty() {
            return Err("Empty style class".to_string());
        }

        // Parse spacing: p-{0-12}
        if let Some(rest) = class.strip_prefix("p-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Padding(size));
        }

        // Parse gap: gap-{0-12}
        if let Some(rest) = class.strip_prefix("gap-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Gap(size));
        }

        // Parse background: bg-{color}
        if let Some(color_name) = class.strip_prefix("bg-") {
            let color = Color::from_tailwind(color_name)
                .or_else(|_| Color::from_hex(color_name))?;
            return Ok(StyleClass::BackgroundColor(color));
        }

        // Parse text color: text-{color}
        if let Some(color_name) = class.strip_prefix("text-") {
            let color = Color::from_tailwind(color_name)
                .or_else(|_| Color::from_hex(color_name))?;
            return Ok(StyleClass::TextColor(color));
        }

        // Parse flex
        if class == "flex" {
            return Ok(StyleClass::Flex);
        }

        // Parse flex-row
        if class == "flex-row" {
            return Ok(StyleClass::FlexRow);
        }

        // Parse flex-col
        if class == "flex-col" {
            return Ok(StyleClass::FlexCol);
        }

        // Parse items-center
        if class == "items-center" {
            return Ok(StyleClass::ItemsCenter);
        }

        // Parse justify-center
        if class == "justify-center" {
            return Ok(StyleClass::JustifyCenter);
        }

        // Parse justify-between
        if class == "justify-between" {
            return Ok(StyleClass::JustifyBetween);
        }

        // Parse width: w-{size}
        if let Some(rest) = class.strip_prefix("w-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Width(size));
        }

        // Parse height: h-{size}
        if let Some(rest) = class.strip_prefix("h-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Height(size));
        }

        // Parse rounded
        if class == "rounded" {
            return Ok(StyleClass::Rounded);
        }

        Err(format!("Unknown style class: {}", class))
    }
}

/// Helper function to parse size values
fn parse_size_value(input: &str) -> Result<SizeValue, String> {
    match input {
        "full" => Ok(SizeValue::Full),
        "auto" => Ok(SizeValue::Auto),
        "1/2" => Ok(SizeValue::Half),
        "1/3" => Ok(SizeValue::Third),
        "2/3" => Ok(SizeValue::TwoThirds),
        "1/4" => Ok(SizeValue::Quarter),
        "3/4" => Ok(SizeValue::ThreeQuarters),
        _ => {
            // Try to parse as a number
            let value: u16 = input.parse()
                .map_err(|_| format!("Invalid size value: {}", input))?;
            Ok(SizeValue::Fixed(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_padding() {
        assert_eq!(StyleClass::parse_single("p-4"), Ok(StyleClass::Padding(SizeValue::Fixed(4))));
        assert_eq!(StyleClass::parse_single("p-0"), Ok(StyleClass::Padding(SizeValue::Fixed(0))));
    }

    #[test]
    fn test_parse_gap() {
        assert_eq!(StyleClass::parse_single("gap-2"), Ok(StyleClass::Gap(SizeValue::Fixed(2))));
    }

    #[test]
    fn test_parse_colors() {
        assert!(matches!(StyleClass::parse_single("bg-white"), Ok(StyleClass::BackgroundColor(_))));
        assert!(matches!(StyleClass::parse_single("text-slate-500"), Ok(StyleClass::TextColor(_))));
    }

    #[test]
    fn test_parse_layout() {
        assert_eq!(StyleClass::parse_single("flex"), Ok(StyleClass::Flex));
        assert_eq!(StyleClass::parse_single("flex-row"), Ok(StyleClass::FlexRow));
        assert_eq!(StyleClass::parse_single("flex-col"), Ok(StyleClass::FlexCol));
        assert_eq!(StyleClass::parse_single("items-center"), Ok(StyleClass::ItemsCenter));
    }

    #[test]
    fn test_parse_sizing() {
        assert_eq!(StyleClass::parse_single("w-full"), Ok(StyleClass::Width(SizeValue::Full)));
        assert_eq!(StyleClass::parse_single("h-12"), Ok(StyleClass::Height(SizeValue::Fixed(12))));
    }

    #[test]
    fn test_parse_border_radius() {
        assert_eq!(StyleClass::parse_single("rounded"), Ok(StyleClass::Rounded));
    }

    #[test]
    fn test_size_to_pixels() {
        assert_eq!(SizeValue::Fixed(4).to_pixels(), 16); // 4 * 4px = 16px
    }
}
