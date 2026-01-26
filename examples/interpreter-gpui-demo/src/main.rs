// GPUI 动态解释器可视化演示
//
// 这个示例展示了完整的动态解释器功能：
// 1. 加载 .at 文件
// 2. 实时渲染到 GPUI
// 3. 交互式按钮点击
// 4. 热重载支持（按 'R' 键重新加载）

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
                    title: Some("AutoUI 动态解释器演示".into()),
                    appears_transient: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |cx| {
                cx.new_view(|cx| DemoApp::new(cx))
            },
        )
    });
}

struct DemoApp {
    interpreter: DynamicInterpreterComponent,
    file_path: PathBuf,
    reload_count: usize,
}

impl DemoApp {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        // 使用相对路径查找 counter.at 文件
        let mut path = std::env::current_dir().unwrap();
        path.push("counter.at");

        // 如果文件不存在，尝试在 examples 目录中查找
        if !path.exists() {
            path = std::env::current_dir().unwrap();
            path.push("examples");
            path.push("interpreter-gpui-demo");
            path.push("counter.at");
        }

        println!("📄 加载文件: {:?}", path);

        Self {
            interpreter: DynamicInterpreterComponent::from_file(&path, cx),
            file_path: path,
            reload_count: 0,
        }
    }

    fn reload(&mut self, cx: &mut ViewContext<Self>) {
        self.reload_count += 1;
        println!("🔄 重新加载文件 (#{})...", self.reload_count);
        self.interpreter.reload(cx);
        cx.notify();
    }
}

impl Render for DemoApp {
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
                    .justify_between()
                    .px_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .child("AutoUI 动态解释器")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(format!("已加载: {:?}", self.file_path.file_name().unwrap()))
                            )
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x9ca3af))
                            .child("按 'R' 键重新加载")
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

// 注册键盘快捷键
impl EventEmitter for DemoApp {}

// 注册全局快捷键处理
impl DemoApp {
    fn handle_key_event(&mut self, event: &KeyEvent, _window: &mut Window, cx: &mut ViewContext<Self>) {
        if event.keystroke.key == 'r' {
            self.reload(cx);
        }
    }
}
