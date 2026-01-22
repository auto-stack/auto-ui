// 完整的 GPUI 应用示例：运行从 Auto 语言生成的组件
//
// 这个示例展示了如何：
// 1. 使用 auto-ui-transpiler 从 Auto 语言生成 Rust 组件
// 2. 集成生成的组件到 GPUI 应用中
// 3. 处理消息和更新 UI
//
// 运行方式：
//   cargo run --package auto-ui-gpui --example hello_auto

use auto_ui::{Component, View};
use auto_ui_gpui::{GpuiComponentState, IntoGpuiElement};
use gpui::{App, AppContext, Context, Window, WindowOptions};
use std::sync::Arc;

// ============================================================
// 从 Auto 语言生成的组件（代码来自 scratch/text_simple.rs）
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

struct AppState {
    hello_state: GpuiComponentState<Hello>,
}

impl AppState {
    fn new() -> Self {
        let hello = Hello::new("Hello from Auto Language!".to_string());
        Self {
            hello_state: GpuiComponentState::new(hello),
        }
    }
}

// ============================================================
// GPUI 渲染实现
// ============================================================

impl gpui::Render for AppState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        // 获取组件的 view 并转换为 GPUI 元素
        let view = self.hello_state.component().view();

        // 将抽象 View 转换为 GPUI 元素
        view.into_gpui_static()
    }
}

// ============================================================
// 主函数：启动 GPUI 应用
// ============================================================

fn main() {
    println!("=== AutoUI + GPUI 应用示例 ===\n");
    println!("正在启动 GPUI 应用...");
    println!("组件: Hello (从 Auto 语言生成)");
    println!("消息: \"Hello from Auto Language!\"\n");

    // 创建 GPUI 应用
    App::new().run(move |cx: &mut AppContext| {
        // 配置窗口
        let window_options = WindowOptions {
            window_bounds: Some(gpui::Bounds {
                origin: gpui::Point { x: 100.0, y: 100.0 },
                size: gpui::Size { width: 800.0, height: 600.0 },
            }),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("AutoUI Hello Example".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        // 打开窗口
        cx.open_window(window_options, |cx| {
            // 创建应用状态
            let state = AppState::new();

            // 显示窗口并返回状态
            cx.new_view(|_cx| state)
        })
        .unwrap();
    });

    println!("\n✅ 应用已启动！");
    println!("提示：按 Ctrl+C 退出应用");
}

// ============================================================
// 额外示例：带交互的计数器组件
// ============================================================

#[cfg(feature = "demo_counter")]
mod counter_example {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct Counter {
        count: i32,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum CounterMsg {
        Inc,
        Dec,
    }

    impl Counter {
        pub fn new() -> Self {
            Self { count: 0 }
        }
    }

    impl Component for Counter {
        type Msg = CounterMsg;

        fn on(&mut self, msg: Self::Msg) {
            match msg {
                CounterMsg::Inc => self.count += 1,
                CounterMsg::Dec => self.count -= 1,
            }
        }

        fn view(&self) -> View<Self::Msg> {
            // 注意：这需要 button 节点生成功能
            // 当前 rust_gen.rs 还没有实现 button，这里是示例代码
            View::col()
                .spacing(10)
                .padding(20)
                .child(View::text(&format!("Count: {}", self.count)))
                // 当 button 节点实现后，可以这样使用：
                // .child(View::button("+", CounterMsg::Inc))
                // .child(View::button("-", CounterMsg::Dec))
                .build()
        }
    }

    pub fn run_counter() {
        App::new().run(move |cx: &mut AppContext| {
            let window_options = WindowOptions {
                window_bounds: Some(gpui::Bounds {
                    origin: gpui::Point { x: 100.0, y: 100.0 },
                    size: gpui::Size { width: 400.0, height: 300.0 },
                }),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Counter Example".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            };

            cx.open_window(window_options, |cx| {
                let counter = Counter::new();
                let state = GpuiComponentState::new(counter);
                cx.new_view(|_cx| state)
            })
            .unwrap();
        });
    }
}
