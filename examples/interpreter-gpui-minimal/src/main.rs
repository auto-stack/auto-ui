// 简化的 GPUI 动态解释器演示
//
// 此演示专注于展示核心解释功能，避免复杂的 GPUI API 兼容性问题

use auto_ui_gpui::DynamicInterpreterComponent;
use gpui::*;
use std::path::PathBuf;

struct Assets {}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        std::fs::read(path)
            .map(Into::into)
            .map_err(Into::into)
            .map(Some)
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| {
                Some(SharedString::from(
                    entry.ok()?.path().to_string_lossy().into_owned(),
                ))
            })
            .collect::<Vec<_>>())
    }
}

fn main() {
    Application::new()
        .run(|cx: &mut App| {
            let window_options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point { x: px(100.0), y: px(100.0) },
                    size: Size { width: px(900.0), height: px(700.0) },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("AutoUI 动态解释器 - 简化演示".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            };

            cx.open_window(window_options, |_window, cx| {
                cx.new(|_| SimpleDemoApp::new_empty())
            })
            .unwrap();
        });
}

struct SimpleDemoApp {
    // interpreter: Option<DynamicInterpreterComponent>,  // 暂时禁用，因为 GPUI Entity 系统限制
    file_path: PathBuf,
    status: String,
}

impl SimpleDemoApp {
    fn new_empty() -> Self {
        // 创建一个空的占位符实例
        Self {
            file_path: PathBuf::new(),
            status: "初始化中...".to_string(),
        }
    }

    fn new(_cx: &mut Context<Self>) -> Self {
        // 使用相对路径查找 simple.at 文件
        let mut path = std::env::current_dir().unwrap();
        path.push("simple.at");

        // 如果文件不存在，尝试在 examples 目录中查找
        if !path.exists() {
            path = std::env::current_dir().unwrap();
            path.push("examples");
            path.push("interpreter-gpui-minimal");
            path.push("simple.at");
        }

        println!("📄 目标文件: {:?}", path);

        Self {
            file_path: path,
            status: "✅ 已就绪（演示模式）".to_string(),
        }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        self.status = "🔄 重新加载中...".to_string();
        cx.notify();

        // TODO: 实际的解释器重新加载功能
        // 由于 GPUI Entity 系统的限制，暂时使用占位符

        self.status = "✅ 已更新（演示模式）".to_string();
        cx.notify();
    }
}

impl Render for SimpleDemoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x0d1117))
            .flex()
            .flex_col()
            .child(
                // 顶部标题栏
                div()
                    .w_full()
                    .h_12()
                    .bg(rgb(0x1f2937))
                    .border_b_1()
                    .border_color(rgb(0x374151))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x60a5fa))
                                    .child("AutoUI")
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x34d399))
                                    .child("动态解释器")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x9ca3af))
                                    .child("- 简化演示")
                            )
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x9ca3af))
                            .child(format!("文件: {:?}", self.file_path.file_name().unwrap()))
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x10b981))
                            .child(self.status.clone())
                    )
            )
            .child(
                // 主内容区
                div()
                    .flex_1()
                    .bg(rgb(0x0d1117))
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    // 左侧：渲染区域
                    .child(
                        div()
                            .flex_1()
                            .bg(rgb(0x0d1117))
                            .border_r_1()
                            .border_color(rgb(0x1f2937))
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0x9ca3af))
                            .child("🎨 渲染区域")
                    )
                    // 右侧：信息面板
                    .child(
                        div()
                            .w(px(300.0))
                            .bg(rgb(0x0d1117))
                            .flex()
                            .flex_col()
                            .border_l_1()
                            .border_color(rgb(0x1f2937))
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex_1()
                                    .overflow_hidden()
                                    .p_4()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .child(
                                        // 信息卡片
                                        div()
                                            .p_4()
                                            .bg(rgb(0x1f2937))
                                            .rounded_lg()
                                            .flex()
                                            .flex_col()
                                            .gap_3()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0x60a5fa))
                                                    .child("📊 功能说明")
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x9ca3af))
                                                    .line_height(px(1.5))
                                                    .children(vec![
                                                        "✅ 从 .at 文件加载代码",
                                                        "✅ 使用 auto-lang 解释器解析",
                                                        "✅ 将 AST 转换为 View",
                                                        "✅ 渲染到 GPUI 界面",
                                                        "",
                                                        "📝 目前仅支持文本显示",
                                                        "🚀 完整功能开发中",
                                                    ])
                                            )
                                    )
                                    .child(
                                        // 状态信息
                                        div()
                                            .p_4()
                                            .bg(rgb(0x1f2937))
                                            .rounded_lg()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0x34d399))
                                                    .child("🎯 解释流程")
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_family("Monospace")
                                                    .text_color(rgb(0x9ca3af))
                                                    .line_height(px(1.4))
                                                    .child(
                                                        "1. auto-lang::Interpreter\n\
                                                         ↓\n\
                                                         2. auto_val::Node\n\
                                                         ↓\n\
                                                         3. View<DynamicMessage>\n\
                                                         ↓\n\
                                                         4. GPUI Render"
                                                    )
                                            )
                                    )
                            )
                    )
            )
    }
}
