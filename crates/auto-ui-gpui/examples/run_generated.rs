// 完整的 GPUI 应用示例：运行生成的 AutoUI 组件
//
// 这个示例展示了如何使用 scratch/text_simple.at 生成的组件
//
// 运行：
//   cargo run --package auto-ui-gpui --example run_generated

use auto_ui::{Component, View};

// ============================================================
// 组件：从 scratch/text_simple.at 生成
// ============================================================

#[derive(Debug)]
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

// 为 Hello 实现 Default，提供默认消息
impl Default for Hello {
    fn default() -> Self {
        Self {
            msg: "Hello from Auto Language!".to_string(),
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
// 主函数
// ============================================================

fn main() -> auto_ui::AppResult<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        AutoUI Generated Component - GPUI Example           ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("📝 组件来源: scratch/text_simple.at");
    println!("🔄 组件类型: Hello");
    println!("💬 默认消息: \"Hello from Auto Language!\"");
    println!();
    println!("正在启动 GPUI 应用...");

    // 使用 auto_ui_gpui 的 run_app 函数
    // 这会自动设置 GPUI 应用并运行组件
    // 组件必须实现 Default trait（我们在上面提供了自定义实现）
    auto_ui_gpui::run_app::<Hello>("AutoUI Generated - Hello")
}
