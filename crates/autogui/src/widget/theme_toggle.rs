use gpui::*;
use gpui::prelude::*;
use crate::widget::button::{Button, ButtonStyles};
use crate::widget::icon::SysIcon;
use crate::style::theme::{ThemeMode, Theme, ActiveTheme};

#[derive(IntoElement)]
pub struct ThemeToggle {
}

impl ThemeToggle {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderOnce for ThemeToggle {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.active_theme();
        let mode = theme.mode;
        let color = theme.secondary_foreground;
        let sun = SysIcon::Sun.icon().color(color);
        let moon = SysIcon::Moon.icon().color(color);
        div()
            .child(
                Button::new().map(|this| {
                    if mode == ThemeMode::Light {
                        this.icon(moon)
                    } else {
                        this.icon(sun)
                    }
                }).style(ButtonStyles::Bare)
                .on_click(move |_ev, cx| {
                    println!("clicked theme toggle");
                    if mode == ThemeMode::Light {
                        Theme::change(ThemeMode::Dark, cx);
                    } else {
                        Theme::change(ThemeMode::Light, cx);
                    }
                })
            )
    }
}
