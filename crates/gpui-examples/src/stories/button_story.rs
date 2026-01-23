// button_story.rs: Button component showcase

use gpui::*;
use gpui_component::button::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::*;

/// Button Story - 展示各种按钮样式和状态
pub struct ButtonStory {
    click_count: u32,
    focus_handle: FocusHandle,
}

impl ButtonStory {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            click_count: 0,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for ButtonStory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ButtonStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let click_count = self.click_count;

        div()
            .w_full()
            .h_full()
            .px_8()
            .py_6()
            .child(
                v_flex()
                    .gap_6()
                    .size_full()
                    .overflow_y_scrollbar()
                    .child(
                        div()
                            .text_size(px(24.0))
                            .font_weight(FontWeight::BOLD)
                            .child("Button Component")
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("各种按钮样式和交互状态的展示")
                    )
                    // Primary Buttons
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Primary Buttons")
                            )
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        Button::new("primary-1")
                                            .primary()
                                            .label("Primary Button")
                                    )
                                    .child(
                                        Button::new("primary-2")
                                            .primary()
                                            .label(format!("Clicked: {}", click_count))
                                            .on_click(cx.listener(|view, _, _, _cx| {
                                                view.click_count += 1;
                                            }))
                                    )
                                    .child(
                                        Button::new("primary-disabled")
                                            .primary()
                                            .label("Disabled")
                                            .disabled(true)
                                    )
                            )
                    )
                    // Ghost Buttons
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Ghost Buttons")
                            )
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        Button::new("ghost-1")
                                            .ghost()
                                            .label("Ghost Button")
                                    )
                                    .child(
                                        Button::new("ghost-2")
                                            .ghost()
                                            .label("Disabled")
                                            .disabled(true)
                                    )
                            )
                    )
                    // Danger Buttons
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Danger Buttons")
                            )
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        Button::new("danger-1")
                                            .danger()
                                            .label("Delete")
                                    )
                                    .child(
                                        Button::new("danger-2")
                                            .danger()
                                            .label("Disabled")
                                            .disabled(true)
                                    )
                            )
                    )
                    // Icon Buttons
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Icon Buttons")
                            )
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        Button::new("icon-add")
                                            .ghost()
                                            .xsmall()
                                            .label("+")
                                    )
                                    .child(
                                        Button::new("icon-edit")
                                            .ghost()
                                            .small()
                                            .label("✏")
                                    )
                                    .child(
                                        Button::new("icon-delete")
                                            .ghost()
                                            .small()
                                            .label("🗑")
                                    )
                            )
                    )
                    // Button Sizes
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Button Sizes")
                            )
                            .child(
                                v_flex()
                                    .gap_2()
                                    .items_start()
                                    .child(
                                        Button::new("size-large")
                                            .primary()
                                            .large()
                                            .label("Large Button")
                                    )
                                    .child(
                                        Button::new("size-default")
                                            .primary()
                                            .label("Default Size")
                                    )
                                    .child(
                                        Button::new("size-small")
                                            .primary()
                                            .small()
                                            .label("Small Button")
                                    )
                                    .child(
                                        Button::new("size-xsmall")
                                            .primary()
                                            .xsmall()
                                            .label("XSmall")
                                    )
                            )
                    )
            )
    }
}
