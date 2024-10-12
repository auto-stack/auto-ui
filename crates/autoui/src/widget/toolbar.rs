use gpui::*;
use crate::style::color::Colorize;
use crate::style::theme::{ActiveTheme, ThemeMode};
use crate::widget::theme_toggle::ThemeToggle;

#[derive(IntoElement)]
pub struct WindowControlIcon {
    name: String,
}

impl WindowControlIcon {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

impl RenderOnce for WindowControlIcon {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.active_theme();
        let hover_color = match theme.mode {
            ThemeMode::Dark => {
                theme.background.lighten(0.05)
            }
            ThemeMode::Light => {
                theme.background.darken(0.05)
            }
        };

        div()
            .flex()
            .flex_row()
            .h_full()
            .w(px(36.0))
            .items_center()
            .content_center()
            .justify_center()
            .hover(|style| {
                style.bg(hover_color)
            })
            .child(self.name)
    }
}

pub struct Toolbar {
}

impl Render for Toolbar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .flex()
            .flex_row()
            .id("toolbar")
            .w_full()
            .h(px(40.0))
            .pl_2()
            .border_b_1()
            .border_color(theme.border)
            .pt_0()
            .pb_0()
            .bg(theme.title_bar_background)
            .content_stretch()
            .content_center()
            .items_center()
            // Title
            .child(
                div()
                    .items_center()
                    .child("Toolbar"))
            // Middle
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .justify_center()
                    .items_center()
                    .child("Middle")
            )
            // Theme Toggle
            .child(ThemeToggle::new())
            // Window Controls
            .child(
                div()
                    .flex()
                    .flex_row()
                    .h_full()
                    .content_stretch()
                    .child(WindowControlIcon::new("🗕"))
                    .child(WindowControlIcon::new("🗖"))
                    .child(WindowControlIcon::new("🗙"))
            )
    }
}

