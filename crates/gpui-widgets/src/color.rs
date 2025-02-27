use std::collections::HashMap;

use gpui::{hsla, point, BoxShadow, Hsla, Pixels};
use serde::{de::Error, Deserialize, Deserializer};

use anyhow::Result;

/// Make a [gpui::Hsla] color.
///
/// - h: 0..360.0
/// - s: 0.0..100.0
/// - l: 0.0..100.0
pub fn hsl(h: f32, s: f32, l: f32) -> Hsla {
    hsla(h / 360., s / 100.0, l / 100.0, 1.0)
}

/// Make a BoxShadow like CSS
///
/// e.g:
///
/// If CSS is `box-shadow: 0 0 10px 0 rgba(0, 0, 0, 0.1);`
///
/// Then the equivalent in Rust is `box_shadow(0., 0., 10., 0., hsla(0., 0., 0., 0.1))`
pub fn box_shadow(
    x: impl Into<Pixels>,
    y: impl Into<Pixels>,
    blur: impl Into<Pixels>,
    spread: impl Into<Pixels>,
    color: Hsla,
) -> BoxShadow {
    BoxShadow {
        offset: point(x.into(), y.into()),
        blur_radius: blur.into(),
        spread_radius: spread.into(),
        color,
    }
}
pub trait Colorize {
    fn opacity(&self, opacity: f32) -> Hsla;
    fn divide(&self, divisor: f32) -> Hsla;
    fn invert(&self) -> Hsla;
    fn invert_l(&self) -> Hsla;
    fn lighten(&self, amount: f32) -> Hsla;
    fn darken(&self, amount: f32) -> Hsla;
}

impl Colorize for Hsla {
    /// Returns a new color with the given opacity.
    ///
    /// The opacity is a value between 0.0 and 1.0, where 0.0 is fully transparent and 1.0 is fully opaque.
    fn opacity(&self, factor: f32) -> Hsla {
        Hsla {
            a: self.a * factor.clamp(0.0, 1.0),
            ..*self
        }
    }

    /// Returns a new color with each channel divided by the given divisor.
    ///
    /// The divisor in range of 0.0 .. 1.0
    fn divide(&self, divisor: f32) -> Hsla {
        Hsla {
            a: divisor,
            ..*self
        }
    }

    /// Return inverted color
    fn invert(&self) -> Hsla {
        Hsla {
            h: (self.h + 1.8) % 3.6,
            s: 1.0 - self.s,
            l: 1.0 - self.l,
            a: self.a,
        }
    }

    /// Return inverted lightness
    fn invert_l(&self) -> Hsla {
        Hsla {
            l: 1.0 - self.l,
            ..*self
        }
    }

    /// Return a new color with the lightness increased by the given factor.
    fn lighten(&self, factor: f32) -> Hsla {
        let l = (self.l * 1.0 + factor.clamp(0.0, 1.0)).min(1.0);

        Hsla { l, ..*self }
    }

    /// Return a new color with the darkness increased by the given factor.
    fn darken(&self, factor: f32) -> Hsla {
        let l = (self.l * 1.0 - factor.clamp(0.0, 1.0)).max(0.0);

        Hsla { l, ..*self }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub title_bar_background: Hsla,
    pub background: Hsla,
    pub foreground: Hsla,
    pub card: Hsla,
    pub card_foreground: Hsla,
    pub popover: Hsla,
    pub popover_foreground: Hsla,
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_active: Hsla,
    pub primary_foreground: Hsla,
    pub secondary: Hsla,
    pub secondary_hover: Hsla,
    pub secondary_active: Hsla,
    pub secondary_foreground: Hsla,
    pub destructive: Hsla,
    pub destructive_hover: Hsla,
    pub destructive_active: Hsla,
    pub destructive_foreground: Hsla,
    pub muted: Hsla,
    pub muted_foreground: Hsla,
    pub accent: Hsla,
    pub accent_foreground: Hsla,
    pub border: Hsla,
    pub input: Hsla,
    pub ring: Hsla,
    pub selection: Hsla,
    pub scrollbar: Hsla,
    pub scrollbar_thumb: Hsla,
    pub panel: Hsla,
    pub tab_bar: Hsla,
    pub list: Hsla,
    pub list_even: Hsla,
    pub list_active: Hsla,
    pub list_head: Hsla,
    pub link: Hsla,
    pub drop_target: Hsla,
}

impl Colors {
    pub fn light() -> Colors {
        Colors {
            title_bar_background: hsl(0.0, 0.0, 100.),
            background: hsl(0.0, 0.0, 100.),
            foreground: hsl(240.0, 10., 3.9),
            card: hsl(0.0, 0.0, 100.0),
            card_foreground: hsl(240.0, 10.0, 3.9),
            popover: hsl(0.0, 0.0, 100.0),
            popover_foreground: hsl(240.0, 10.0, 3.9),
            primary: hsl(223.0, 5.9, 10.0),
            primary_hover: hsl(223.0, 5.9, 15.0),
            primary_active: hsl(223.0, 1.9, 25.0),
            primary_foreground: hsl(223.0, 0.0, 98.0),
            secondary: hsl(240.0, 4.8, 95.9),
            secondary_hover: hsl(240.0, 5.8, 10.).opacity(0.05),
            secondary_active: hsl(240.0, 5.9, 10.).opacity(0.1),
            secondary_foreground: hsl(240.0, 59.0, 10.),
            destructive: hsl(0.0, 84.2, 60.2),
            destructive_hover: hsl(0.0, 84.2, 65.0),
            destructive_active: hsl(0.0, 84.2, 47.0),
            destructive_foreground: hsl(0.0, 0.0, 98.0),
            muted: hsl(240.0, 4.8, 95.9),
            muted_foreground: hsl(240.0, 3.8, 46.1),
            accent: hsl(240.0, 5.0, 96.0),
            accent_foreground: hsl(240.0, 5.9, 10.0),
            border: hsl(240.0, 5.9, 90.0),
            input: hsl(240.0, 5.9, 90.0),
            ring: hsl(240.0, 5.9, 65.0),
            selection: hsl(211.0, 97.0, 85.0).opacity(0.25),
            scrollbar: hsl(0., 0., 97.).opacity(0.7),
            scrollbar_thumb: hsl(0., 0., 69.),
            panel: hsl(0.0, 0.0, 100.0),
            tab_bar: hsl(240.0, 4.8, 95.9),
            list: hsl(0.0, 0.0, 100.),
            list_even: hsl(240.0, 5.0, 96.0),
            list_active: hsl(240.0, 7., 88.0).opacity(0.75),
            list_head: hsl(0.0, 0.0, 100.),
            link: hsl(221.0, 83.0, 53.0),
            drop_target: hsl(235.0, 30., 44.0).opacity(0.25),
        }
    }

    pub fn dark() -> Colors {
        Colors {
            title_bar_background: hsl(0., 0., 12.),
            background: hsl(0.0, 0.0, 6.0),
            foreground: hsl(0., 0., 98.),
            card: hsl(299.0, 2., 9.),
            card_foreground: hsl(0.0, 0.0, 98.0),
            popover: hsl(240.0, 10.0, 3.9),
            popover_foreground: hsl(0.0, 0.0, 98.0),
            primary: hsl(223.0, 0.0, 98.0),
            primary_hover: hsl(223.0, 0.0, 90.0),
            primary_active: hsl(223.0, 0.0, 80.0),
            primary_foreground: hsl(223.0, 5.9, 10.0),
            secondary: hsl(240.0, 3.7, 15.9),
            secondary_hover: hsl(240.0, 3.7, 20.9).opacity(0.5),
            secondary_active: hsl(240.0, 3.7, 20.9).opacity(0.8),
            secondary_foreground: hsl(0.0, 0.0, 98.0),
            destructive: hsl(0.0, 62.8, 30.6),
            destructive_hover: hsl(0.0, 62.8, 35.6),
            destructive_active: hsl(0.0, 62.8, 20.6),
            destructive_foreground: hsl(0.0, 0.0, 98.0),
            muted: hsl(240.0, 3.7, 15.9),
            muted_foreground: hsl(240.0, 5.0, 64.9),
            accent: hsl(240.0, 3.7, 15.9),
            accent_foreground: hsl(0.0, 0.0, 98.0),
            border: hsl(240.0, 3.7, 15.9),
            input: hsl(240.0, 3.7, 15.9),
            ring: hsl(240.0, 4.9, 83.9),
            selection: hsl(211.0, 97.0, 22.0),
            scrollbar: hsl(240., 1., 15.).opacity(0.7),
            scrollbar_thumb: hsl(0., 0., 58.),
            panel: hsl(299.0, 2., 9.),
            tab_bar: hsl(299.0, 2., 9.),
            list: hsl(0.0, 0.0, 6.0),
            list_even: hsl(240.0, 3.7, 8.0),
            list_active: hsl(240.0, 3.7, 15.0),
            list_head: hsl(0.0, 0.0, 6.0),
            link: hsl(221.0, 83.0, 53.0),
            drop_target: hsl(235.0, 30., 44.0).opacity(0.1),
        }
    }
}

pub(crate) trait ColorExt {
    fn to_hex_string(&self) -> String;
    fn parse_hex_string(hex: &str) -> Result<Hsla>;
}

impl ColorExt for Hsla {
    fn to_hex_string(&self) -> String {
        let rgb = self.to_rgb();

        if rgb.a < 1. {
            return format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                ((rgb.r * 255.) as u32),
                ((rgb.g * 255.) as u32),
                ((rgb.b * 255.) as u32),
                ((self.a * 255.) as u32)
            );
        }

        format!(
            "#{:02X}{:02X}{:02X}",
            ((rgb.r * 255.) as u32),
            ((rgb.g * 255.) as u32),
            ((rgb.b * 255.) as u32)
        )
    }

    fn parse_hex_string(hex: &str) -> Result<Hsla> {
        let hex = hex.trim_start_matches('#');
        let len = hex.len();
        if len != 6 && len != 8 {
            return Err(anyhow::anyhow!("invalid hex color"));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)? as f32 / 255.;
        let g = u8::from_str_radix(&hex[2..4], 16)? as f32 / 255.;
        let b = u8::from_str_radix(&hex[4..6], 16)? as f32 / 255.;
        let a = if len == 8 {
            u8::from_str_radix(&hex[6..8], 16)? as f32 / 255.
        } else {
            1.
        };

        let v = gpui::Rgba { r, g, b, a };
        let color: Hsla = v.into();
        Ok(color)
    }
}

pub(crate) static DEFAULT_COLOR: once_cell::sync::Lazy<ShadcnColors> =
    once_cell::sync::Lazy::new(|| {
        serde_json::from_str(include_str!("../assets/default-colors.json"))
            .expect("failed to parse default-json")
    });

type ColorScales = HashMap<usize, ShadcnColor>;

mod color_scales {
    use std::collections::HashMap;

    use super::{ColorScales, ShadcnColor};

    use serde::de::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ColorScales, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = HashMap::new();
        for color in Vec::<ShadcnColor>::deserialize(deserializer)? {
            map.insert(color.scale, color);
        }
        Ok(map)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct ShadcnColors {
    pub(crate) black: ShadcnColor,
    pub(crate) white: ShadcnColor,
    #[serde(with = "color_scales")]
    pub(crate) slate: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) gray: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) zinc: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) neutral: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) stone: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) red: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) orange: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) amber: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) yellow: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) lime: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) green: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) emerald: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) teal: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) cyan: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) sky: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) blue: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) indigo: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) violet: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) purple: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) fuchsia: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) pink: ColorScales,
    #[serde(with = "color_scales")]
    pub(crate) rose: ColorScales,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
pub(crate) struct ShadcnColor {
    #[serde(default)]
    pub(crate) scale: usize,
    #[serde(deserialize_with = "from_hsa_channel", alias = "hslChannel")]
    pub(crate) hsla: Hsla,
}

/// Deserialize Hsla from a string in the format "210 40% 98%"
fn from_hsa_channel<'de, D>(deserializer: D) -> Result<Hsla, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer).unwrap();

    let mut parts = s.split_whitespace();
    if parts.clone().count() != 3 {
        return Err(D::Error::custom(
            "expected hslChannel has 3 parts, e.g: '210 40% 98%'",
        ));
    }

    fn parse_number(s: &str) -> f32 {
        s.trim_end_matches('%')
            .parse()
            .expect("failed to parse number")
    }

    let (h, s, l) = (
        parse_number(parts.next().unwrap()),
        parse_number(parts.next().unwrap()),
        parse_number(parts.next().unwrap()),
    );

    Ok(hsl(h, s, l))
}

macro_rules! color_method {
    ($color:tt, $scale:tt) => {
        paste::paste! {
            #[allow(unused)]
            pub fn [<$color _ $scale>]() -> Hsla {
                if let Some(color) = DEFAULT_COLOR.$color.get(&($scale as usize)) {
                    return color.hsla;
                }

                black()
            }
        }
    };
}

macro_rules! color_methods {
    ($color:tt) => {
        color_method!($color, 50);
        color_method!($color, 100);
        color_method!($color, 200);
        color_method!($color, 300);
        color_method!($color, 400);
        color_method!($color, 500);
        color_method!($color, 600);
        color_method!($color, 700);
        color_method!($color, 800);
        color_method!($color, 900);
        color_method!($color, 950);
    };
}

pub fn black() -> Hsla {
    DEFAULT_COLOR.black.hsla
}

pub fn white() -> Hsla {
    DEFAULT_COLOR.white.hsla
}

color_methods!(slate);
color_methods!(gray);
color_methods!(zinc);
color_methods!(neutral);
color_methods!(stone);
color_methods!(red);
color_methods!(orange);
color_methods!(amber);
color_methods!(yellow);
color_methods!(lime);
color_methods!(green);
color_methods!(emerald);
color_methods!(teal);
color_methods!(cyan);
color_methods!(sky);
color_methods!(blue);
color_methods!(indigo);
color_methods!(violet);
color_methods!(purple);
color_methods!(fuchsia);
color_methods!(pink);
color_methods!(rose);

#[cfg(test)]
mod tests {
    use gpui::{rgb, rgba};

    use super::*;

    #[test]
    fn test_default_colors() {
        assert_eq!(white(), hsl(0.0, 0.0, 100.0));
        assert_eq!(black(), hsl(0.0, 0.0, 0.0));

        assert_eq!(slate_50(), hsl(210.0, 40.0, 98.0));
        assert_eq!(slate_100(), hsl(210.0, 40.0, 96.1));
        assert_eq!(slate_900(), hsl(222.2, 47.4, 11.2));

        assert_eq!(red_50(), hsl(0.0, 85.7, 97.3));
        assert_eq!(yellow_100(), hsl(54.9, 96.7, 88.0));
        assert_eq!(green_200(), hsl(141.0, 78.9, 85.1));
        assert_eq!(cyan_300(), hsl(187.0, 92.4, 69.0));
        assert_eq!(blue_400(), hsl(213.1, 93.9, 67.8));
        assert_eq!(indigo_500(), hsl(238.7, 83.5, 66.7));
    }

    #[test]
    fn test_to_hex_string() {
        let color: Hsla = rgb(0xf8fafc).into();
        assert_eq!(color.to_hex_string(), "#F8FAFC");

        let color: Hsla = rgb(0xfef2f2).into();
        assert_eq!(color.to_hex_string(), "#FEF2F2");

        let color: Hsla = rgba(0x0413fcaa).into();
        assert_eq!(color.to_hex_string(), "#0413FCAA");
    }

    #[test]
    fn test_from_hex_string() {
        let color: Hsla = Hsla::parse_hex_string("#F8FAFC").unwrap();
        assert_eq!(color, rgb(0xf8fafc).into());

        let color: Hsla = Hsla::parse_hex_string("#FEF2F2").unwrap();
        assert_eq!(color, rgb(0xfef2f2).into());

        let color: Hsla = Hsla::parse_hex_string("#0413FCAA").unwrap();
        assert_eq!(color, rgba(0x0413fcaa).into());
    }
}
