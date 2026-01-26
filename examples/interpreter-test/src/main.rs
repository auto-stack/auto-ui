// 动态解释器测试示例
//
// 这个示例演示了如何使用 auto-ui 的动态解释器来：
// 1. 加载 .at 文件
// 2. 使用 auto-lang::Interpreter 解析
// 3. 转换为 View<DynamicMessage>
// 4. 渲染到 GPUI

use auto_ui::interpreter::InterpreterBridge;
use auto_ui::node_converter;
use auto_ui::view::View;
use gpui::*;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(
            WindowOptions::default(),
            |cx: &mut WindowContext| {
                // 创建简单的测试组件
                cx.new_view(|cx| TestApp::new(cx))
            },
        )
    });
}

struct TestApp {
    bridge: InterpreterBridge,
    current_view: Option<View<String>>,
}

impl TestApp {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        println!("🚀 动态解释器测试示例启动");

        // 创建解释器桥梁
        let mut bridge = InterpreterBridge::new();

        // 尝试加载 counter.at 文件
        println!("📄 加载 counter.at 文件...");
        match bridge.load_file("counter.at") {
            Ok(_) => println!("✅ 文件加载成功"),
            Err(e) => println!("❌ 文件加载失败: {}", e),
        }

        // 获取主视图
        println!("🎨 获取主视图...");
        let current_view = match bridge.get_main_view() {
            Ok(node) => {
                println!("✅ 获取 Node 成功");

                // 转换 Node → View<String>
                println!("🔄 转换 Node 到 View...");
                match node_converter::convert_node(&node) {
                    Ok(view) => {
                        println!("✅ 转换成功");
                        println!("📊 View 类型: {:?}", std::mem::discriminant(&view));
                        Some(view)
                    }
                    Err(e) => {
                        println!("❌ 转换失败: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                println!("❌ 获取视图失败: {}", e);
                None
            }
        }

        Self {
            bridge,
            current_view,
        }
    }
}

impl Render for TestApp {
    fn render(&mut self, _window: &mut Window, cx: &mut ViewContext<Self>) -> impl IntoElement {
        if let Some(view) = &self.current_view {
            // 渲染视图
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .child(self.render_view(view.clone(), cx))
        } else {
            // 显示错误信息
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .p_4()
                        .bg(gpui::red())
                        .text_xl()
                        .text_color(gpui::white())
                        .child("❌ 无法加载 counter.at 文件")
                )
        }
    }
}

impl TestApp {
    fn render_view(&mut self, view: View<String>, cx: &mut ViewContext<Self>) -> AnyElement {
        match view {
            View::Text { content, .. } => {
                div()
                    .px_4()
                    .py_2()
                    .bg(gpui::gray())
                    .rounded_md()
                    .child(content)
                    .into_any()
            }
            View::Button { label, .. } => {
                div()
                    .px_4()
                    .py_2()
                    .bg(gpui::blue())
                    .text_color(gpui::white())
                    .rounded_md()
                    .cursor_pointer()
                    .child(label)
                    .into_any()
            }
            View::Col { spacing, children, .. } => {
                let mut col = div().flex().flex_col().gap(spacing);
                for child in children {
                    col = col.child(self.render_view(child, cx));
                }
                col.into_any()
            }
            View::Row { spacing, children, .. } => {
                let mut row = div().flex().flex_row().gap(spacing);
                for child in children {
                    row = row.child(self.render_view(child, cx));
                }
                row.into_any()
            }
            View::Container { child, .. } => {
                self.render_view(*child, cx).into_any()
            }
            View::Empty => {
                div().child("(空)").into_any()
            }
            _ => {
                div()
                    .text_color(gpui::yellow())
                    .child(format!("🔧 组件类型暂未实现: {:?}", std::mem::discriminant(&view)))
                    .into_any()
            }
        }
    }
}
