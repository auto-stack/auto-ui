// 简化版 GPUI 动态解释器演示
//
// 展示基本的渲染功能，暂时不包含交互处理

use auto_ui_gpui::DynamicInterpreterComponent;
use gpui::*;
use std::path::PathBuf;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point { x: Pixel(100.0), y: Pixel(100.0) },
                    size: Size { width: Pixel(800.0), height: Pixel(600.0) },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("AutoUI 动态解释器 - 简化演示".into()),
                    appears_transient: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |cx| {
                cx.new_view(|cx| SimpleDemoApp::new(cx))
            },
        )
    });
}

struct SimpleDemoApp {
    interpreter: DynamicInterpreterComponent,
    file_path: PathBuf,
}

impl SimpleDemoApp {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        // 使用相对路径查找 simple.at 文件
        let mut path = std::env::current_dir().unwrap();
        path.push("simple.at");

        // 如果文件不存在，尝试在 examples 目录中查找
        if !path.exists() {
            path = std::env::current_dir().unwrap();
            path.push("examples");
            path.push("interpreter-gpui-demo");
            path.push("simple.at");
        }

        println!("📄 加载文件: {:?}", path);

        Self {
            interpreter: DynamicInterpreterComponent::from_file(&path, cx),
            file_path: path,
        }
    }
}

impl Render for SimpleDemoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut ViewContext<Self>) -> impl IntoElement {
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
                            .child("AutoUI 动态解释器 - 简化演示")
                    )
            )
            .child(
                // 主内容区 - 嵌入解释器组件
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(self.interpreter.clone())
            )
    }
}
