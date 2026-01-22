// Auto-generated from Auto language (with main function for testing)
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

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

fn main() {
    let hello = Hello::new("Hello from Auto!".to_string());

    println!("=== AutoUI Component Demo ===\n");
    println!("Component: {:?}", hello);
    println!("Message: {}", hello.msg);

    // Inspect view structure
    let view = hello.view();
    println!("\nView structure:");
    println!("{:#?}", view);

    println!("\n✅ Component created successfully!");
    println!("Note: This is a logical test. For actual UI rendering,");
    println!("you need to integrate with GPUI or Iced backend.");
}
