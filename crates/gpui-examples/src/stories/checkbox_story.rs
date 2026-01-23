// checkbox_story.rs: Checkbox component showcase

use gpui::*;
use gpui_component::checkbox::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::*;

/// Checkbox Story - 展示复选框组件功能
pub struct CheckboxStory {
    enabled_checkboxes: Vec<bool>,
    mood_happy: bool,
    mood_productive: bool,
    mood_creative: bool,
    focus_handle: FocusHandle,
}

impl CheckboxStory {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            enabled_checkboxes: vec![false, false, false],
            mood_happy: false,
            mood_productive: false,
            mood_creative: false,
            focus_handle: cx.focus_handle(),
        }
    }

    fn toggle_enabled(&mut self, index: usize) {
        if index < self.enabled_checkboxes.len() {
            self.enabled_checkboxes[index] = !self.enabled_checkboxes[index];
        }
    }

    fn toggle_mood_happy(&mut self) {
        self.mood_happy = !self.mood_happy;
    }

    fn toggle_mood_productive(&mut self) {
        self.mood_productive = !self.mood_productive;
    }

    fn toggle_mood_creative(&mut self) {
        self.mood_creative = !self.mood_creative;
    }
}

impl Focusable for CheckboxStory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CheckboxStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mood_count = [self.mood_happy, self.mood_productive, self.mood_creative]
            .iter()
            .filter(|&&x| x)
            .count();

        let mood_text = match mood_count {
            0 => "Select your moods below",
            1 => "Feeling something specific",
            2 => "Mixed emotions today",
            _ => "Feeling great!",
        };

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
                            .child("Checkbox Component")
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("用于选择一个或多个选项的复选框控件")
                    )
                    // Basic Checkboxes
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Basic Checkboxes")
                            )
                            .child(
                                Checkbox::new("cb-1")
                                    .label("Checkbox 1")
                                    .selected(self.enabled_checkboxes[0])
                                    .on_click(cx.listener(|view, _, _, _cx| {
                                        view.toggle_enabled(0);
                                    }))
                            )
                            .child(
                                Checkbox::new("cb-2")
                                    .label("Checkbox 2")
                                    .selected(self.enabled_checkboxes[1])
                                    .on_click(cx.listener(|view, _, _, _cx| {
                                        view.toggle_enabled(1);
                                    }))
                            )
                            .child(
                                Checkbox::new("cb-3")
                                    .label("Checkbox 3")
                                    .selected(self.enabled_checkboxes[2])
                                    .on_click(cx.listener(|view, _, _, _cx| {
                                        view.toggle_enabled(2);
                                    }))
                            )
                    )
                    // Mood Tracker Example
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Mood Tracker (Example)")
                            )
                            .child(
                                div()
                                    .text_size(px(18.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(mood_text)
                            )
                            .child(
                                Checkbox::new("mood-happy")
                                    .label("😊 Happy")
                                    .selected(self.mood_happy)
                                    .on_click(cx.listener(|view, _, _, _cx| {
                                        view.toggle_mood_happy();
                                    }))
                            )
                            .child(
                                Checkbox::new("mood-productive")
                                    .label("💪 Productive")
                                    .selected(self.mood_productive)
                                    .on_click(cx.listener(|view, _, _, _cx| {
                                        view.toggle_mood_productive();
                                    }))
                            )
                            .child(
                                Checkbox::new("mood-creative")
                                    .label("🎨 Creative")
                                    .selected(self.mood_creative)
                                    .on_click(cx.listener(|view, _, _, _cx| {
                                        view.toggle_mood_creative();
                                    }))
                            )
                    )
                    // Features
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Features")
                            )
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .child("• 简单易用：通过 .selected() 控制选中状态")
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .child("• 事件处理：使用 .on_click() 处理点击事件")
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .child("• 灵活布局：支持水平和垂直布局")
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .child("• 自定义标签：支持文本和图标")
                                    )
                            )
                    )
            )
    }
}
