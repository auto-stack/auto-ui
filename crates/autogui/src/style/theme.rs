use super::color::{Colors, Colorize};
use super::color;
use std::ops::Deref;

use gpui::{
    hsla, AppContext, Global, Hsla, ModelContext, SharedString, ViewContext, WindowAppearance,
    WindowContext,
};

pub fn init_theme(cx: &mut AppContext) {
    Theme::sync_system_appearance(cx)
    // Theme::change(ThemeMode::Dark, cx);
}

pub trait ActiveTheme {
    fn active_theme(&self) -> &Theme;
}

impl ActiveTheme for AppContext {
    fn active_theme(&self) -> &Theme {
        Theme::get_global(self)
    }
}

impl<'a, V> ActiveTheme for ViewContext<'a, V> {
    fn active_theme(&self) -> &Theme {
        self.deref().active_theme()
    }
}

impl<'a, V> ActiveTheme for ModelContext<'a, V> {
    fn active_theme(&self) -> &Theme {
        self.deref().active_theme()
    }
}

impl<'a> ActiveTheme for WindowContext<'a> {
    fn active_theme(&self) -> &Theme {
        self.deref().active_theme()
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub transparent: Hsla,
    pub title_bar_background: Hsla,
    /// Basic font size
    pub font_size: f32,
    pub font_family: SharedString,
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
    /// Set to true to enable shadow for Button, Input, Dropdown, DatePicker ...
    pub shadow: bool,
    pub selection: Hsla,
    pub scrollbar: Hsla,
    pub scrollbar_thumb: Hsla,
    pub panel: Hsla,
    pub drag_border: Hsla,
    pub drop_target: Hsla,
    pub radius: f32,
    pub tab_bar: Hsla,
    pub tab: Hsla,
    pub tab_active: Hsla,
    pub tab_foreground: Hsla,
    pub tab_active_foreground: Hsla,
    pub progress_bar: Hsla,
    pub slider_bar: Hsla,
    pub slider_thumb: Hsla,
    pub list: Hsla,
    pub list_even: Hsla,
    pub list_head: Hsla,
    pub list_active: Hsla,
    pub list_hover: Hsla,
    pub table: Hsla,
    pub table_even: Hsla,
    pub table_head: Hsla,
    pub table_active: Hsla,
    pub table_hover: Hsla,
    pub link: Hsla,
    pub link_hover: Hsla,
    pub link_active: Hsla,
    pub skeleton: Hsla,
}

impl Global for Theme {}

impl Theme {
    pub fn get_global(cx: &AppContext) -> &Self {
        cx.global::<Self>()
    }
}

impl From<Colors> for Theme {
    fn from(colors: Colors) -> Self {
        Theme {
            mode: ThemeMode::default(),
            transparent: Hsla::transparent_black(),
            font_size: 14.0,
            font_family: if cfg!(target_os = "macos") {
                ".SystemUIFont".into()
            } else if cfg!(target_os = "windows") {
                "Segoe UI".into()
            } else {
                "FreeMono".into()
            },
            radius: 4.0,
            shadow: true,
            title_bar_background: colors.title_bar_background,
            background: colors.background,
            foreground: colors.foreground,
            card: colors.card,
            card_foreground: colors.card_foreground,
            popover: colors.popover,
            popover_foreground: colors.popover_foreground,
            primary: colors.primary,
            primary_hover: colors.primary_hover,
            primary_active: colors.primary_active,
            primary_foreground: colors.primary_foreground,
            secondary: colors.secondary,
            secondary_hover: colors.secondary_hover,
            secondary_active: colors.secondary_active,
            secondary_foreground: colors.secondary_foreground,
            destructive: colors.destructive,
            destructive_hover: colors.destructive_hover,
            destructive_active: colors.destructive_active,
            destructive_foreground: colors.destructive_foreground,
            muted: colors.muted,
            muted_foreground: colors.muted_foreground,
            accent: colors.accent,
            accent_foreground: colors.accent_foreground,
            border: colors.border,
            input: colors.input,
            ring: colors.ring,
            scrollbar: colors.scrollbar,
            scrollbar_thumb: colors.scrollbar_thumb,
            panel: colors.panel,
            selection: colors.selection,
            drag_border: color::blue_500(),
            drop_target: colors.drop_target,
            tab_bar: colors.tab_bar,
            tab: gpui::transparent_black(),
            tab_active: colors.background,
            tab_foreground: colors.foreground,
            tab_active_foreground: colors.foreground,
            progress_bar: colors.primary,
            slider_bar: colors.primary,
            slider_thumb: colors.background,
            list: colors.list,
            list_even: colors.list_even,
            list_head: colors.list_head,
            list_active: colors.list_active,
            list_hover: colors.list_active.opacity(0.6),
            table_head: colors.list_active.opacity(0.6),
            table: colors.list,
            table_even: colors.list_even,
            table_active: colors.list_active,
            table_hover: colors.list_active.opacity(0.8),
            link: colors.link,
            link_hover: colors.link.lighten(0.2),
            link_active: colors.link.darken(0.2),
            skeleton: hsla(colors.primary.h, colors.primary.s, colors.primary.l, 0.1),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

impl ThemeMode {
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }
}

impl Theme {
    /// Sync the theme with the system appearance
    pub fn sync_system_appearance(cx: &mut AppContext) {
        match cx.window_appearance() {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => {
                Self::change(ThemeMode::Dark, cx)
            }
            WindowAppearance::Light | WindowAppearance::VibrantLight => {
                Self::change(ThemeMode::Light, cx)
            }
        }
    }

    pub fn toggle(cx: &mut AppContext) {
        let mode = if cx.active_theme().mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Self::change(mode, cx);
    }

    pub fn change(mode: ThemeMode, cx: &mut AppContext) {
        let colors = match mode {
            ThemeMode::Light => Colors::light(),
            ThemeMode::Dark => Colors::dark(),
        };

        let mut theme = Theme::from(colors);
        theme.mode = mode;

        cx.set_global(theme);
        cx.refresh();
    }
}
