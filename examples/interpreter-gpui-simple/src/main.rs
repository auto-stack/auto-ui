// 简化版 GPUI 动态解释器演示
//
// 展示基本的 VTree 渲染功能

use gpui::*;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point { x: px(100.0), y: px(100.0) },
                    size: Size { width: px(800.0), height: px(600.0) },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("AutoUI VNode 渲染演示".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                cx.new_view(|cx| SimpleDemoApp::new(window, cx))
            },
        )
    });
}

struct SimpleDemoApp {
    title: String,
}

impl SimpleDemoApp {
    fn new(_window: &mut Window, cx: &mut ViewContext<Self>) -> Self {
        println!("✅ VNode 渲染演示启动");
        Self {
            title: "AutoUI VNode 渲染演示".to_string(),
        }
    }
}

impl Render for SimpleDemoApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x1a1a1a))
            .flex()
            .flex_col()
            .child(
                // 顶部工具栏
                div()
                    .w_full()
                    .h_8()
                    .bg(rgb(0x2a2a2a))
                    .border_b_1()
                    .border_color(rgb(0x3a3a3a))
                    .flex()
                    .items_center()
                    .px_4()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .child(&self.title)
                    )
            )
            .child(
                // 主内容区 - 演示各种控件
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_6()
                    .child(
                        // 标题
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .child("🎨 VNode 架构演示")
                    )
                    .child(
                        // 副标题
                        div()
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .child("展示 AutoUI 的 VTree 渲染能力")
                    )
                    .child(
                        // 演示控件
                        div()
                            .flex()
                            .flex_row()
                            .gap_4()
                            .child(
                                // 按钮
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(0x3b82f6))
                                    .rounded_md()
                                    .cursor_pointer()
                                    .child("按钮")
                            )
                            .child(
                                // 输入框
                                div()
                                    .px_3()
                                    .py_2()
                                    .bg(rgb(0x2a2a2a))
                                    .border_1()
                                    .border_color(rgb(0x4a4a4a))
                                    .rounded_md()
                                    .text_sm()
                                    .child("输入框 (不可交互)")
                            )
                            .child(
                                // 复选框
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w_4()
                                            .h_4()
                                            .border_1()
                                            .border_color(rgb(0x6c6c6c))
                                            .bg(rgb(0x2a2a2a))
                                            .rounded_sm()
                                    )
                                    .child("复选框")
                            )
                    )
                    .child(
                        // 布局示例
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .w(px(400.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .child("布局示例：")
                            )
                            .child(
                                // 列布局
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .p_2()
                                    .bg(rgb(0x222222))
                                    .rounded_md()
                                    .child(
                                        div()
                                            .text_xs()
                                            .child("项目 1")
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .child("项目 2")
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .child("项目 3")
                                    )
                            )
                            .child(
                                // 行布局
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .p_2()
                                    .bg(rgb(0x222222))
                                    .rounded_md()
                                    .child(
                                        div()
                                            .text_xs()
                                            .child("左")
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .child("中")
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .child("右")
                                    )
                            )
                    )
                    .child(
                        // 说明文字
                        div()
                            .max_w(px(500.0))
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .child("此示例展示了 VNode 架构的渲染能力。完整的解释器功能需要编译 auto-lang，当前示例使用硬编码的 UI 演示。")
                    )
            )
    }
}
