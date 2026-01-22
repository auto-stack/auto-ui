// 示例：验证从 Auto 语言生成的组件代码
// 使用方法：将生成的 Component 代码复制到这里，然后运行：
//   cargo run --package auto-ui --example verify_generated

use auto_ui::{Component, View};

// === 以下是生成的代码（从 scratch/text_simple.rs 复制） ===

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

impl Component for Hello {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        View::text(&self.msg)
    }
}

// === 测试代码 ===

fn main() {
    println!("=== AutoUI Generated Component Verification ===\n");

    // 测试 1: 创建组件
    let hello = Hello::new("Hello from Auto!".to_string());
    println!("✅ Component created: {:?}", hello);

    // 测试 2: 访问字段
    println!("✅ Message field: {}", hello.msg);

    // 测试 3: 生成 View
    let view = hello.view();
    println!("\n✅ View generated successfully!");
    println!("View structure: {:#?}", view);

    // 测试 4: 创建不同消息的组件
    let hello2 = Hello::new("Different message".to_string());
    let view2 = hello2.view();
    println!("\n✅ Second component view: {:#?}", view2);

    println!("\n=== All Tests Passed! ===");
    println!("\n📝 Note: This is a logical verification.");
    println!("To render actual UI, you need to:");
    println!("  1. Use auto-ui-gpui backend for GPUI applications");
    println!("  2. Use auto-ui-iced backend for Iced applications");
    println!("  3. Or integrate with your own UI backend");
}
