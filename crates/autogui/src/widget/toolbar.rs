use gpui::*;
use crate::style::color::Colorize;
use crate::style::theme::{ActiveTheme, ThemeMode};
use crate::widget::theme_toggle::ThemeToggle;
use crate::widget::button::Button;
use crate::widget::icon::SysIcon;
use crate::widget::button::ButtonStyles;
use crate::event::ReloadEvent;

use gpui::UpdateGlobal;
use crate::app::{GlobalDataStoreSave, GlobalState, ReloadState};

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

pub const TOOLBAR_HEIGHT: f32 = 40.0;

impl Render for Toolbar {


    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .flex()
            .flex_row()
            .id("toolbar")
            .w_full()
            .h(px(TOOLBAR_HEIGHT))
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
                    .child("Toolbar")
            )
            // Middle
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_grow()
                    .justify_center()
                    .items_center()
                    .child("Middle")
            )
            // Reload
            .child(
                Button::new().icon(SysIcon::Reload.icon()).style(ButtonStyles::Bare)
                    .on_click(cx.listener(move |_this, _ev, cx| {
                        println!("Reload Clicked! Event");
                        cx.emit(ReloadEvent);

                        GlobalState::update_global(cx, |g, _| {
                            g.count += 1;
                        });

                        ReloadState::update_global(cx, |_g, _| {
                        });

                        cx.refresh();
                    }))
            )
            .child(
                Button::new().icon(SysIcon::Download.icon()).style(ButtonStyles::Bare)
                    .on_click(cx.listener(move |_this, _ev, cx| {
                        println!("Download Clicked! Event");

                        GlobalDataStoreSave::update_global(cx, |a, b| {});

                        cx.refresh();
                    }))
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

impl EventEmitter<ReloadEvent> for Toolbar {}