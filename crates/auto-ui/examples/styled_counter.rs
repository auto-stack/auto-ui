// Example demonstrating the unified styling system integration with View API
//
// This shows how the new styling API (Plan 005) works alongside the legacy API.
//
// Phase 1 Implementation: All View variants now support optional `style` field

use auto_ui::{Component, View};
use std::fmt::Debug;

#[derive(Debug, Default)]
struct StyledCounter {
    count: i64,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    Inc,
    Dec,
    Reset,
}

impl Component for StyledCounter {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::Inc => self.count += 1,
            Msg::Dec => self.count -= 1,
            Msg::Reset => self.count = 0,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        // ✅ NEW: Using unified styling system
        // Note: Only using L1+L2+L3 supported classes (90% Tailwind coverage)
        View::col()
            .style("gap-4 p-8 bg-gray-100 flex items-center")  // L1+L2 classes
            .child(
                // Styled text (L2 typography)
                View::text_styled(
                    format!("Counter: {}", self.count),
                    "text-3xl font-bold text-center text-blue-600"  // L2: text-3xl, font-bold
                )
            )
            .child(
                View::row()
                    .style("gap-4 flex justify-center")  // L1: gap, flex, justify
                    .child(
                        // Styled button (L1+L2 classes)
                        View::button_styled(
                            "Increment",
                            Msg::Inc,
                            "px-4 py-2 bg-green-500 text-white rounded-lg font-bold"  // px, py: L2 | bg, text, rounded: L1 | font-bold: L2
                        )
                    )
                    .child(
                        View::button_styled(
                            "Decrement",
                            Msg::Dec,
                            "px-4 py-2 bg-red-500 text-white rounded-lg font-bold"
                        )
                    )
                    .child(
                        View::button_styled(
                            "Reset",
                            Msg::Reset,
                            "px-4 py-2 bg-gray-500 text-white rounded-lg font-medium"  // font-medium: L2
                        )
                    )
                    .build()
            )
            .child(
                // Styled container card (L3 shadow)
                View::container(
                    View::text("Instructions: Use the buttons above to change the counter value.")
                )
                .style("p-4 bg-white rounded-lg shadow-md")  // L1+L2+L3: shadow-md is L3
                .build()
            )
            .build()
    }
}

fn main() {
    let mut counter = StyledCounter::default();

    println!("=== Unified Styling System Demo ===\n");

    // Simulate interactions
    println!("1. Incrementing counter:");
    counter.on(Msg::Inc);
    counter.on(Msg::Inc);
    counter.on(Msg::Inc);

    println!("\n2. View structure with styles:");
    let view = counter.view();
    println!("{:#?}\n", view);

    println!("✅ Unified styling system integration working!");
    println!("\nFeatures demonstrated:");
    println!("  - Text with style: text_styled()");
    println!("  - Buttons with style: button_styled()");
    println!("  - Layout with style: .style() on builders");
    println!("  - Container with style: .style() on builders");
    println!("\nStyle classes used:");
    println!("  - L1 (Core): p-8, gap-4, bg-*, text-*, rounded, flex, justify-center");
    println!("  - L2 (Important): px-*, py-*, text-3xl, font-bold/medium, text-center, items-center");
    println!("  - L3 (Advanced): shadow-md");
    println!("\nCoverage: 90% of Tailwind CSS core features supported!");
    println!("See Plan 004 for details on supported style classes.");
}
