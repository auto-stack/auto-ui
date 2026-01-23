// gallery.rs: Main Gallery application for Story showcase

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::*;
use gpui::InteractiveElement;
use crate::stories::*;

/// Story 类型枚举
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoryType {
    Welcome,
    Button,
    Select,
    Checkbox,
}

impl StoryType {
    fn name(&self) -> &str {
        match self {
            StoryType::Welcome => "Welcome",
            StoryType::Button => "Button",
            StoryType::Select => "Select",
            StoryType::Checkbox => "Checkbox",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            StoryType::Welcome,
            StoryType::Button,
            StoryType::Select,
            StoryType::Checkbox,
        ]
    }
}

/// Gallery - 主应用
pub struct Gallery {
    story_type: StoryType,
    focus_handle: FocusHandle,
}

impl Gallery {
    /// 创建新的 Gallery 实例
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            story_type: StoryType::Welcome,
            focus_handle: cx.focus_handle(),
        }
    }

    /// 创建 Gallery 视图
    pub fn view(_init_story: Option<&str>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for Gallery {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let story_type = self.story_type;

        div()
            .size_full()
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .size_full()
                    .child(
                        // 侧边栏
                        div()
                            .h_full()
                            .w(px(250.0))
                            .border_r_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().sidebar)
                            .child(
                                v_flex()
                                    .size_full()
                                    .child(
                                        // Gallery 标题
                                        div()
                                            .p_4()
                                            .border_b_1()
                                            .border_color(cx.theme().border)
                                            .child(
                                                div()
                                                    .text_size(px(18.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .child("AutoUI Gallery")
                                            )
                                    )
                                    .child(
                                        // Stories 列表
                                        v_flex()
                                            .flex_1()
                                            .overflow_y_scrollbar()
                                            .children(
                                                StoryType::all()
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(idx, s)| {
                                                        let is_selected = self.story_type == s;
                                                        let name = s.name().to_string();

                                                        div()
                                                            .w_full()
                                                            .px_4()
                                                            .py_2()
                                                            .cursor_pointer()
                                                            .when(is_selected, |div| {
                                                                div.bg(rgb(0x3c3c3c))
                                                            })
                                                            .hover(|div| {
                                                                div.bg(rgb(0x2a2a2a))
                                                            })
                                                            .child(
                                                                div()
                                                                    .text_size(px(14.0))
                                                                    .child(name)
                                                            )
                                                            .on_mouse_down(MouseButton::Left, cx.listener(move |gallery, _, _, _cx| {
                                                                gallery.story_type = StoryType::all()[idx];
                                                                _cx.notify();
                                                            }))
                                                    })
                                                    .collect::<Vec<_>>()
                                            )
                                    )
                            )
                    )
                    .child(
                        // 主内容区 - 根据 story_type 动态渲染
                        div()
                            .flex_1()
                            .size_full()
                            .child(match story_type {
                                StoryType::Welcome => {
                                    let story = cx.new(|cx| WelcomeStory::new(_window, cx));
                                    div().child(story)
                                }
                                StoryType::Button => {
                                    let story = cx.new(|cx| ButtonStory::new(_window, cx));
                                    div().child(story)
                                }
                                StoryType::Select => {
                                    let story = cx.new(|cx| SelectStory::new(_window, cx));
                                    div().child(story)
                                }
                                StoryType::Checkbox => {
                                    let story = cx.new(|cx| CheckboxStory::new(_window, cx));
                                    div().child(story)
                                }
                            }.into_any())
                    )
            )
    }
}
