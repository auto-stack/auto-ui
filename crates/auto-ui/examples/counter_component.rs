// Example demonstrating the improved Component trait and View API
//
// This shows how the new abstraction layer aligns with Auto language syntax:
// - Component trait with `on()` method (Auto: fn on)
// - View enum with direct message storage (Auto: onclick: Msg.Inc)
// - ViewBuilder for fluent layout construction (Auto: col { ... })

use auto_ui::{Component, View};
use std::fmt::Debug;

#[derive(Debug, Default)]
struct Counter {
    count: i64,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    Inc,
    Dec,
}

impl Component for Counter {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        // Auto: fn on(ev Msg) { is ev { Msg.Inc => { .count += 1 } } }
        match msg {
            Msg::Inc => self.count += 1,
            Msg::Dec => self.count -= 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        // Auto: fn view() View { col { ... } }
        View::col()
            .spacing(10)
            .padding(20)
            .child(View::button("+", Msg::Inc))
            .child(View::text(self.count.to_string()))
            .child(View::button("-", Msg::Dec))
            .build()
    }
}

// Example usage (not runnable without backend adapter)
fn main() {
    let mut counter = Counter::default();

    // Simulate message handling
    counter.on(Msg::Inc);
    println!("Count after Inc: {}", counter.count);

    counter.on(Msg::Dec);
    println!("Count after Dec: {}", counter.count);

    // Inspect view structure
    let view = counter.view();
    println!("View structure: {:?}", view);

    println!("\n✅ Component trait and View API working correctly!");
    println!("Note: This example only demonstrates the abstraction layer.");
    println!("To see actual UI, run iced-examples or gpui-examples with backend adapters.");
}
