// Unified App example
//
// This demonstrates the new unified App abstraction that allows
// running the same Component code with different backends via feature flags.
//
// Run with:
//   cargo run --package unified-app-example  # uses default backend from features
//   cargo run --package unified-app-example --features iced
//   cargo run --package unified-app-example --features gpui

use auto_ui::{Component, View, App};

#[derive(Debug, Default)]
struct Counter {
    count: i64,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Increment,
    Decrement,
}

impl Component for Counter {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(16)
            .padding(20)
            .child(View::button("Increment", Message::Increment))
            .child(View::text(format!("Count: {}", self.count)))
            .child(View::button("Decrement", Message::Decrement))
            .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    // The same code works with both backends!
    // Just change the feature flag in Cargo.toml or CLI:
    //   --features iced   → Iced backend
    //   --features gpui   → GPUI backend (will show error, needs manual impl)

    App::run::<Counter>()
}
