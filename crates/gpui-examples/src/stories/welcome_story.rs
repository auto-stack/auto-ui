// welcome_story.rs: Welcome Story for Gallery homepage

use gpui::*;
use gpui_component::*;

/// Welcome Story - Gallery 的欢迎页面
pub struct WelcomeStory {
    focus_handle: FocusHandle,
}

impl WelcomeStory {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for WelcomeStory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for WelcomeStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .px_12()
            .py_8()
            .child(
                v_flex()
                    .gap_6()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(32.0))
                            .font_weight(FontWeight::BOLD)
                            .child("Welcome to AutoUI")
                    )
                    .child(
                        div()
                            .text_size(px(18.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("Unified UI framework for GPUI and Iced")
                    )
                    .child(
                        div()
                            .max_w(px(600.0))
                            .text_size(px(14.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("Select a component from the sidebar to view its story")
                    )
            )
    }
}
