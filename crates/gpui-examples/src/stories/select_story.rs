// select_story.rs: Select component showcase (Plan 007)

use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::select::*;
use gpui_component::*;

/// Simple select item for demonstration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fruit {
    Apple,
    Banana,
    Cherry,
    Durian,
}

impl Fruit {
    fn emoji(&self) -> &str {
        match self {
            Fruit::Apple => "🍎",
            Fruit::Banana => "🍌",
            Fruit::Cherry => "🍒",
            Fruit::Durian => "🥘",
        }
    }

    fn name(&self) -> &str {
        match self {
            Fruit::Apple => "Apple",
            Fruit::Banana => "Banana",
            Fruit::Cherry => "Cherry",
            Fruit::Durian => "Durian",
        }
    }
}

impl SelectItem for Fruit {
    type Value = &'static str;

    fn title(&self) -> SharedString {
        format!("{} {}", self.emoji(), self.name()).into()
    }

    fn value(&self) -> &Self::Value {
        match self {
            Fruit::Apple => &"Apple",
            Fruit::Banana => &"Banana",
            Fruit::Cherry => &"Cherry",
            Fruit::Durian => &"Durian",
        }
    }
}

/// Select Story - 展示 Select 组件功能
pub struct SelectStory {
    fruit_select: Entity<SelectState<Vec<Fruit>>>,
    selected_fruit: Option<Fruit>,
    focus_handle: FocusHandle,
}

impl SelectStory {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let fruits = vec![Fruit::Apple, Fruit::Banana, Fruit::Cherry, Fruit::Durian];
        let fruit_select = cx.new(|cx| {
            SelectState::new(
                fruits,
                Some(IndexPath::default().row(0)),
                window,
                cx,
            )
        });

        // Subscribe to selection changes
        cx.subscribe_in(&fruit_select, window, Self::on_fruit_select)
            .detach();

        Self {
            fruit_select,
            selected_fruit: Some(Fruit::Apple),
            focus_handle: cx.focus_handle(),
        }
    }

    fn on_fruit_select(
        &mut self,
        _: &Entity<SelectState<Vec<Fruit>>>,
        event: &SelectEvent<Vec<Fruit>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(value) => {
                if let Some(fruit_value) = value {
                    self.selected_fruit = match *fruit_value {
                        "Apple" => Some(Fruit::Apple),
                        "Banana" => Some(Fruit::Banana),
                        "Cherry" => Some(Fruit::Cherry),
                        "Durian" => Some(Fruit::Durian),
                        _ => None,
                    };
                }
            }
        }
    }
}

impl Focusable for SelectStory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SelectStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_text = self.selected_fruit.as_ref()
            .map(|f| format!("{} Selected: {}", f.emoji(), f.name()))
            .unwrap_or_else(|| "No selection".to_string());

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
                            .child("Select Component")
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(cx.theme().muted_foreground)
                            .child("下拉选择组件，支持键盘导航和筛选")
                    )
                    // Basic Select
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Basic Select")
                            )
                            .child(
                                div()
                                    .w(px(300.0))
                                    .child(Select::new(&self.fruit_select).placeholder("Select a fruit"))
                            )
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(selected_text)
                            )
                    )
                    // Features List
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
                                            .child("• 键盘导航：使用 ↑↓ 方向键选择")
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .child("• 快速筛选：输入字符快速定位")
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .child("• 确认选择：按 Enter 确认，Esc 取消")
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .child("• 原生实现：使用原生控件，性能优异")
                                    )
                            )
                    )
                    // Usage Example
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Implementation (Plan 007)")
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("此组件使用原生 OS 控件实现，通过 pre-initialization 技术确保")
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(cx.theme().muted_foreground)
                                    .child("在首次渲染前完成控件创建，避免 GPUI stack overflow 问题。")
                            )
                    )
            )
    }
}
