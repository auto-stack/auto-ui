// GPUI 应用示例：运行带 Col 布局的组件
//
// 这个示例展示了如何使用 scratch/col_test.at 生成的组件
//
// 运行：
//   cargo run --package auto-ui-gpui --example run_col --features gpui

use auto_ui::{Component, View};

// ============================================================
// 组件：从 scratch/col_test.at 生成
// ============================================================

#[derive(Debug)]
pub struct ColHello {
    pub msg: String,
}

impl ColHello {
    pub fn new(msg: String) -> Self {
        Self {
            msg,
        }
    }
}

// 为 ColHello 实现 Default
impl Default for ColHello {
    fn default() -> Self {
        Self {
            msg: "Hello".to_string(),
        }
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

// ============================================================
// 主函数
// ============================================================

fn main() -> auto_ui::AppResult<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║      AutoUI Col Layout - GPUI Example                      ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("📝 组件来源: scratch/col_test.at");
    println!("🔄 组件类型: ColHello");
    println!("📦 布局类型: Col (垂直布局)");
    println!("📄 子元素: 2 个 text 节点");
    println!();
    println!("正在启动 GPUI 应用...");

    auto_ui_gpui::run_app::<ColHello>("AutoUI Col Layout")
}
