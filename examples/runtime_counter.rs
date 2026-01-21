// Example: Runtime interpretation of AutoUI
//
// This example demonstrates how to use AutoUI's runtime interpretation mode
// where you build UI nodes programmatically without transpilation.

use auto_ui::Component;
use auto_ui::View;
use auto_val::Node;

// Simple counter component using runtime interpretation
struct Counter {
    count: i32,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    Inc,
    Dec,
}

impl Component for Counter {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::Inc => self.count += 1,
            Msg::Dec => self.count -= 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        // Build UI using the View builder API
        // This is what the transpiler would generate from counter.at
        View::col()
            .spacing(16)
            .padding(8)
            .child(
                View::button("+")
                    .on_click(Msg::Inc)
                    .build()
            )
            .child(
                View::text(&format!("Count: {}", self.count))
                    .build()
            )
            .child(
                View::button("-")
                    .on_click(Msg::Dec)
                    .build()
            )
            .build()
    }
}

fn main() {
    println!("=== AutoUI Runtime Interpretation Example ===\n");

    let mut counter = Counter { count: 0 };

    println!("📊 Initial state: count = {}", counter.count);

    // Simulate some user interactions
    counter.on(Msg::Inc);
    println!("➕ After Inc: count = {}", counter.count);

    counter.on(Msg::Inc);
    println!("➕ After Inc: count = {}", counter.count);

    counter.on(Msg::Dec);
    println!("➖ After Dec: count = {}", counter.count);

    // Build the view
    let view = counter.view();
    println!("\n🎨 View built successfully!");
    println!("   (In a real app, this would be rendered by GPUI or Iced)");

    // You could also create nodes programmatically:
    let text_node = Node::new("text")
        .with_arg(auto_val::Value::Str("Hello from Auto!".into()));

    if let Ok(view) = auto_ui::node_converter::convert_node(&text_node) {
        println!("\n✅ Runtime node conversion works!");
    }

    println!("\n💡 This is runtime mode - no transpilation needed!");
    println!("   For .at file support, use the transpiler:");
    println!("   cargo run --example transpile_counter --features transpiler");
}

// Helper to create nodes with arguments
trait NodeHelper {
    fn with_arg(self, arg: auto_val::Value) -> Self;
}

impl NodeHelper for Node {
    fn with_arg(mut self, arg: auto_val::Value) -> Self {
        self.add_pos_arg_unified(arg);
        self
    }
}
