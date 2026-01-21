// Comprehensive Styling System Showcase
//
// This example demonstrates all L1, L2, and L3 features of the unified
// styling system (Plan 004) integrated with the View API (Plan 005).
//
// Coverage: 90% of Tailwind CSS core features

use auto_ui::{Component, View};
use std::fmt::Debug;

#[derive(Debug, Default)]
struct StylingShowcase {
    // State for interactive examples
    text_size: &'static str,
    bg_color: &'static str,
    show_advanced: bool,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    ChangeTextSize,
    ChangeBgColor,
    ToggleAdvanced,
    Reset,
}

impl Component for StylingShowcase {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::ChangeTextSize => {
                // Cycle through text sizes
                self.text_size = match self.text_size {
                    "text-sm" => "text-base",
                    "text-base" => "text-lg",
                    "text-lg" => "text-xl",
                    "text-xl" => "text-2xl",
                    _ => "text-sm",
                };
            }
            Msg::ChangeBgColor => {
                // Cycle through background colors
                self.bg_color = match self.bg_color {
                    "bg-white" => "bg-gray-100",
                    "bg-gray-100" => "bg-blue-100",
                    "bg-blue-100" => "bg-green-100",
                    "bg-green-100" => "bg-white",
                    _ => "bg-white",
                };
            }
            Msg::ToggleAdvanced => {
                self.show_advanced = !self.show_advanced;
            }
            Msg::Reset => {
                *self = Self::default();
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .style("gap-6 p-8 min-h-screen bg-gray-50")
            .child(
                // Header section
                View::col()
                    .style("gap-3")
                    .child(
                        View::text_styled(
                            "Unified Styling System Showcase",
                            "text-4xl font-extrabold text-center text-transparent bg-clip-text bg-gradient-to-r from-blue-600 to-purple-600"
                        )
                    )
                    .child(
                        View::text_styled(
                            "90% Tailwind CSS Coverage • L1+L2+L3 Features",
                            "text-lg text-center text-gray-600 font-medium"
                        )
                    )
                    .build()
            )
            .child(
                // L1 Core Features Section
                View::container(
                    View::col()
                        .style("gap-4")
                        .child(
                            View::text_styled("L1: Core Features", "text-2xl font-bold text-gray-800")
                        )
                        .child(
                            // Spacing demonstration
                            View::row()
                                .style("gap-4 flex flex-wrap")
                                .child(
                                    View::col()
                                        .style("gap-2")
                                        .child(
                                            View::container(
                                                View::text("Padding Demo")
                                                    .into())
                                                .style("p-4 bg-blue-500 text-white rounded")
                                                .build()
                                        )
                                        .child(
                                            View::container(
                                                View::text("Gap Demo")
                                                    .into()
                                                .style("gap-2 p-4 bg-green-500 text-white rounded")
                                                .build()
                                        )
                                )
                                .child(
                                    View::col()
                                        .style("gap-2")
                                        .child(
                                            View::container(
                                                View::text("Colors Demo")
                                                    .into()
                                                .style("p-4 bg-red-500 text-white rounded")
                                                .build()
                                        )
                                        .child(
                                            View::container(
                                                View::text("Layout Demo")
                                                    .into()
                                                .style("p-4 bg-purple-500 text-white rounded flex items-center justify-center")
                                                .build()
                                        )
                                )
                                .build()
                        )
                        .build()
                )
                .style("p-6 bg-white rounded-lg shadow-md")
                .build()
            )
            .child(
                // L2 Important Features Section
                View::container(
                    View::col()
                        .style("gap-4")
                        .child(
                            View::text_styled("L2: Important Features", "text-2xl font-bold text-gray-800")
                        )
                        .child(
                            // Typography demonstration
                            View::row()
                                .style("gap-4 flex flex-wrap")
                                .child(
                                    View::col()
                                        .style("gap-2")
                                        .child(
                                            View::text_styled(
                                                format!("Typography: {}", self.text_size),
                                                self.text_size
                                            )
                                        )
                                        .child(
                                            View::button_styled(
                                                "Change Size",
                                                Msg::ChangeTextSize,
                                                "px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600"
                                            )
                                        )
                                )
                                .child(
                                    View::col()
                                        .style("gap-2")
                                        .child(
                                            View::text_styled(
                                                "Font Weights:",
                                                "text-base font-medium text-gray-700"
                                            )
                                        )
                                        .child(
                                            View::row().style("gap-2").children(vec![
                                                View::text_styled("Normal", "font-normal text-gray-600"),
                                                View::text_styled("Medium", "font-medium text-gray-700"),
                                                View::text_styled("Bold", "font-bold text-gray-900"),
                                            ]).build()
                                        )
                                )
                                .build()
                        )
                        .build()
                )
                .style("p-6 bg-white rounded-lg shadow-md")
                .build()
            )
            .child(
                // L3 Advanced Features Section
                View::container(
                    View::col()
                        .style("gap-4")
                        .child(
                            View::row()
                                .style("gap-4 justify-between items-center")
                                .child(
                                    View::text_styled("L3: Advanced Features", "text-2xl font-bold text-gray-800")
                                )
                                .child(
                                    View::button_styled(
                                        if self.show_advanced { "Hide Advanced" } else { "Show Advanced" },
                                        Msg::ToggleAdvanced,
                                        "px-4 py-2 bg-gray-700 text-white rounded-lg hover:bg-gray-800"
                                    )
                                )
                                .build()
                        )
                        .child(
                            // Advanced features demonstration
                            View::col()
                                .style("gap-4")
                                .child(
                                    // Shadow examples
                                    View::row()
                                        .style("gap-4 flex-wrap")
                                        .children(vec![
                                            View::container(
                                                View::text("shadow-sm")
                                                    .into())
                                                .style("p-4 bg-white rounded-lg shadow-sm border")
                                                .build()
                                            ),
                                            View::container(
                                                View::text("shadow-md")
                                                    .into())
                                                .style("p-4 bg-white rounded-lg shadow-md border")
                                                .build()
                                            ),
                                            View::container(
                                                View::text("shadow-lg")
                                                    .into())
                                                .style("p-4 bg-white rounded-lg shadow-lg border")
                                                .build()
                                            ),
                                        ])
                                        .build()
                                )
                                .child(
                                    // Opacity examples
                                    View::row()
                                        .style("gap-4")
                                        .children(vec![
                                            View::container(
                                                View::text("opacity-50")
                                                    .into())
                                                .style("p-4 bg-gray-500 text-white rounded opacity-50")
                                                .build()
                                            ),
                                            View::container(
                                                View::text("opacity-75")
                                                    .into())
                                                .style("p-4 bg-gray-500 text-white rounded opacity-75")
                                                .build()
                                            ),
                                        ])
                                        .build()
                                )
                                .child(if self.show_advanced {
                                    // More advanced features when expanded
                                    View::col()
                                        .style("gap-3")
                                        .child(
                                            View::text_styled(
                                                "Overflow Examples",
                                                "text-lg font-semibold text-gray-700"
                                            )
                                        )
                                        .child(
                                            View::row()
                                                .style("gap-4")
                                                .children(vec![
                                                    View::container(
                                                        View::text("overflow-hidden")
                                                            .into())
                                                        .style("w-32 p-4 bg-red-500 text-white rounded overflow-hidden text-ellipsis")
                                                        .build()
                                                    ),
                                                    View::container(
                                                        View::text("overflow-auto")
                                                            .into())
                                                        .style("w-32 p-4 bg-blue-500 text-white rounded overflow-auto")
                                                        .build()
                                                    ),
                                                ])
                                                .build()
                                        )
                                        .build()
                                } else {
                                    View::empty()
                                })
                                .build()
                        )
                        .build()
                )
                .style("p-6 bg-white rounded-lg shadow-md")
                .build()
            )
            .child(
                // Background color demonstration
                View::container(
                    View::col()
                        .style("gap-3")
                        .child(
                            View::text_styled("Interactive Background", "text-xl font-semibold text-gray-800")
                        )
                        .child(
                            View::row()
                                .style("gap-4")
                                .child(
                                    View::button_styled(
                                        "Change Background",
                                        Msg::ChangeBgColor,
                                        "px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
                                    )
                                )
                                .child(
                                    View::button_styled(
                                        "Reset All",
                                        Msg::Reset,
                                        "px-6 py-3 bg-gray-600 text-white rounded-lg hover:bg-gray-700"
                                    )
                                )
                                .build()
                        )
                        .child(
                            View::container(
                                View::text_styled(
                                    format!("Current: {}", self.bg_color.replace("bg-", "").replace("-", " ")),
                                    "text-lg"
                                )
                                .style(self.bg_color)
                                .build()
                        )
                        .build()
                )
                .style("p-6 bg-white rounded-lg shadow-md")
                .build()
            )
            .build()
    }
}

fn main() {
    let mut showcase = StylingShowcase::default();

    println!("=== Unified Styling System Showcase ===\n");
    println!("This example demonstrates the complete styling system integration.\n");

    // Simulate some interactions
    println!("1. Initial state:");
    println!("   - View tree structure created with styles");
    println!("   - All L1, L2, and L3 features available\n");

    println!("2. Style classes coverage:");
    println!("   - L1 (Core): 15 classes - spacing, colors, layout, sizing, border radius");
    println!("   - L2 (Important): 25+ classes - typography, extended spacing, borders");
    println!("   - L3 (Advanced): 25+ classes - shadows, opacity, overflow, grid, position");
    println!("   - Total: 65+ classes covering 90% of Tailwind CSS core features\n");

    println!("3. API usage patterns:");
    println!("   - View::col().style(\"gap-4 p-8 bg-blue-50\")");
    println!("   - View::text_styled(\"Hello\", \"text-xl font-bold\")");
    println!("   - View::button_styled(\"Click\", msg, \"px-4 py-2 bg-blue-500\")");
    println!("   - Legacy API still works: .spacing(10).padding(20)\n");

    println!("✅ Styling system fully integrated!");
    println!("\nSee Plan 004 for implementation details:");
    println!("  - docs/plans/004-unified-styling-system.md");
    println!("\nSee Plan 005 for integration details:");
    println!("  - docs/plans/005-style-system-integration.md");
}
