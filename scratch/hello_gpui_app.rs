// 完整的 GPUI 应用示例：运行从 Auto 语言生成的组件
//
// 这个示例展示了如何将 transpiler 生成的组件集成到 GPUI 应用中
//
// 运行方式（两种方法）：
//
// 方法 1：作为独立 binary 运行
//   rustc --edition 2021 \
//     -L target/debug/deps \
//     --extern auto_ui=target/debug/libauto_ui.rlib \
//     --extern auto_ui_gpui=target/debug/libauto_ui_gpui.rlib \
//     --extern gpui=target/debug/deps/libgpui-*.rlib \
//     --extern gpui_component=target/debug/deps/libgpui_component-*.rlib \
//     scratch/hello_gpui_app.rs -o scratch/hello_gpui_app.exe
//   scratch/hello_gpui_app.exe
//
// 方法 2：作为 example 运行（推荐）
//   将此文件复制到 crates/auto-ui-gpui/examples/ 目录
//   cargo run --package auto-ui-gpui --example hello_gpui_app

use auto_ui::{Component, View};
use auto_ui_gpui::GpuiComponentState;
use gpui::{App, AppContext, Context, Window, WindowOptions};

// ============================================================
// 从 Auto 语言生成的组件
// 源文件：scratch/text_simple.at
// ============================================================

#[derive(Debug, Clone)]
pub struct Hello {
    pub msg: String,
}

impl Hello {
    pub fn new(msg: String) -> Self {
        Self {
            msg,
        }
    }
}

impl Component for Hello {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        View::text(&self.msg)
    }
}

// ============================================================
// GPUI 应用状态
// ============================================================

struct HelloAppState {
    hello_state: GpuiComponentState<Hello>,
}

impl HelloAppState {
    fn new(initial_message: String) -> Self {
        let hello = Hello::new(initial_message);
        Self {
            hello_state: GpuiComponentState::new(hello),
        }
    }
}

// ============================================================
// GPUI 渲染实现
// ============================================================

impl gpui::Render for HelloAppState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        // 从组件获取 view
        let view = self.hello_state.component().view();

        // 使用 auto-ui-gpui 的转换功能渲染
        // 注意：需要实现 IntoGpuiElement trait
        view.into_gpui_static()
    }
}

// ============================================================
// 主函数：启动 GPUI 应用
// ============================================================

fn main() {
    println!("╔════════════════════════════════════════════════╗");
    println!("║   AutoUI + GPUI 应用示例                       ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();
    println!("📝 组件来源: Auto 语言 (scratch/text_simple.at)");
    println!("🔄 转译器: auto-ui-transpiler");
    println!("🎨 渲染引擎: GPUI");
    println!();
    println!("正在启动应用...");

    // 初始化消息
    let message = "Hello from Auto Language! 🚀".to_string();

    // 创建 GPUI 应用
    App::new().run(move |cx: &mut AppContext| {
        // 配置窗口
        let window_options = WindowOptions {
            window_bounds: Some(gpui::Bounds {
                origin: gpui::Point { x: 100.0, y: 100.0 },
                size: gpui::Size { width: 800.0, height: 600.0 },
            }),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("AutoUI Hello - GPUI".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        // 打开窗口
        match cx.open_window(window_options, |cx| {
            // 创建应用状态
            let state = HelloAppState::new(message);
            cx.new_view(|_cx| state)
        }) {
            Ok(_) => println!("✅ 窗口创建成功！"),
            Err(e) => eprintln!("❌ 窗口创建失败: {:?}", e),
        }
    });

    println!();
    println!("🎉 应用已启动！");
    println!("💡 提示：按 Ctrl+C 或关闭窗口退出应用");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("工作流程说明：");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("1. 编写 Auto 语言代码 (.at 文件)");
    println!("2. 运行 transpiler 生成 Rust 组件");
    println!("3. 集成到 GPUI 应用中");
    println!("4. 享受声明式 UI 开发体验！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

// ============================================================
// 扩展示例：带 Col 布局的组件
// ============================================================

#[cfg(feature = "demo_col")]
fn run_col_example() {
    use auto_ui::View;

    // 源文件：scratch/col_test.at
    #[derive(Debug, Clone)]
    struct ColHello {
        msg: String,
    }

    impl ColHello {
        fn new(msg: String) -> Self {
            Self { msg }
        }
    }

    impl Component for ColHello {
        type Msg = ();

        fn on(&mut self, _msg: Self::Msg) {}

        fn view(&self) -> View<Self::Msg> {
            // 从 col_test.at 生成的代码
            View::col()
                .spacing(0)
                .padding(0)
                .child(View::text(&self.msg))
                .child(View::text(&"World".to_string()))
                .build()
        }
    }

    println!("运行 Col 布局示例...");

    App::new().run(move |cx: &mut AppContext| {
        let window_options = WindowOptions {
            window_bounds: Some(gpui::Bounds {
                origin: gpui::Point { x: 150.0, y: 150.0 },
                size: gpui::Size { width: 400.0, height: 300.0 },
            }),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Col Layout Example".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(window_options, |cx| {
            let col_hello = ColHello::new("Hello".to_string());
            let state = GpuiComponentState::new(col_hello);
            cx.new_view(|_cx| state)
        })
        .unwrap();
    });
}
