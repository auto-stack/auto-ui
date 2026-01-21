// Example demonstrating the improved Component trait and View API
//
// This shows how the new abstraction layer aligns with Auto language syntax:
// - Component trait with `on()` method (Auto: fn on)
// - View enum with direct message storage (Auto: onclick: Msg.Inc)
// - ViewBuilder for fluent layout construction (Auto: col { ... })
//
// Phase 3 Update: Now demonstrates both legacy and unified styling APIs

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
        // ✅ NEW: Using unified styling system (Plan 004 + Plan 005)
        // Demonstrates L1 (Core) and L2 (Important) style classes
        View::col()
            .style("gap-4 p-8 bg-gray-100 flex items-center justify-center")
            .child(
                // Styled counter display (L2 typography)
                View::text_styled(
                    format!("Counter: {}", self.count),
                    "text-3xl font-bold text-center text-blue-600"
                )
            )
            .child(
                View::row()
                    .style("gap-4 flex justify-center")
                    .child(
                        // Styled increment button (L1+L2 classes)
                        View::button_styled(
                            "➕ Increment",
                            Msg::Inc,
                            "px-6 py-3 bg-green-500 text-white rounded-lg font-bold"
                        )
                    )
                    .child(
                        // Styled decrement button
                        View::button_styled(
                            "➖ Decrement",
                            Msg::Dec,
                            "px-6 py-3 bg-red-500 text-white rounded-lg font-bold"
                        )
                    )
                    .build()
            )
            .build()
    }
}

// Example showing legacy API for comparison
impl Counter {
    fn view_legacy_example() -> View<Msg> {
        // Legacy API (still supported for backward compatibility)
        View::col()
            .spacing(10)
            .padding(20)
            .child(View::button("+", Msg::Inc))
            .child(View::text("0"))
            .child(View::button("-", Msg::Dec))
            .build()
    }
}

// Example usage (not runnable without backend adapter)
fn main() {
    let mut counter = Counter::default();

    println!("=== Counter Component Demo ===\n");

    // Simulate message handling
    println!("1. Incrementing counter:");
    counter.on(Msg::Inc);
    counter.on(Msg::Inc);
    counter.on(Msg::Inc);
    println!("   Count: {}", counter.count);

    println!("\n2. Decrementing counter:");
    counter.on(Msg::Dec);
    println!("   Count: {}", counter.count);

    // Inspect view structure
    println!("\n3. View structure with unified styling:");
    let view = counter.view();
    println!("{:#?}\n", view);

    println!("✅ Component trait and View API working correctly!");
    println!("\nUnified Styling Features:");
    println!("  - L1 (Core): gap, p-8, bg-*, rounded, flex, items-center, justify-center");
    println!("  - L2 (Important): px-*, py-*, text-3xl, font-bold, text-center");
    println!("\nNote: This example demonstrates the abstraction layer.");
    println!("To see actual UI rendering, run styled_counter example.");
}
