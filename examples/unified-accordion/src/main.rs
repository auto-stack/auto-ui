// Unified Accordion Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates the Accordion (collapsible sections) component.
//
// Run with:
//   cargo run --package unified-accordion --features iced
//   cargo run --package unified-accordion --features gpui

use auto_ui::{Component, View, AccordionItem};

#[derive(Debug)]
struct AccordionApp {
    // Track which sections are expanded
    expanded_sections: Vec<bool>,
}

impl Default for AccordionApp {
    fn default() -> Self {
        Self {
            expanded_sections: vec![true, false, false, false], // First section expanded by default
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ToggleSection(usize, bool),
}

impl Component for AccordionApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ToggleSection(index, expanded) => {
                if index < self.expanded_sections.len() {
                    self.expanded_sections[index] = expanded;
                }
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Accordion Example".to_string()))
            .child(View::text("Click on section headers to expand/collapse".to_string()))
            .child(
                View::accordion()
                    .items(vec![
                        AccordionItem::new("Getting Started")
                            .with_icon('🏠')
                            .with_children(vec![
                                View::text("• Home Page".to_string()),
                                View::text("• Hello World".to_string()),
                            ])
                            .with_expanded(self.expanded_sections[0]),
                        AccordionItem::new("Basic Widgets")
                            .with_icon('📦')
                            .with_children(vec![
                                View::text("• Button".to_string()),
                                View::text("• Checkbox".to_string()),
                                View::text("• Slider".to_string()),
                            ])
                            .with_expanded(self.expanded_sections[1]),
                        AccordionItem::new("Forms & Input")
                            .with_icon('📝')
                            .with_children(vec![
                                View::text("• Text Input".to_string()),
                                View::text("• Select".to_string()),
                                View::text("• Todos".to_string()),
                            ])
                            .with_expanded(self.expanded_sections[2]),
                        AccordionItem::new("Layout & Style")
                            .with_icon('🎨')
                            .with_children(vec![
                                View::text("• Layout".to_string()),
                                View::text("• Container".to_string()),
                            ])
                            .with_expanded(self.expanded_sections[3]),
                    ])
                    .allow_multiple(true)
                    .on_toggle(|idx, expanded| Message::ToggleSection(idx, expanded))
                    .build()
            )
            .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    // The same code works with both backends!
    // Just change the feature flag in Cargo.toml or CLI:
    //   --features iced   → Iced backend
    //   --features gpui   → GPUI backend

    #[cfg(feature = "iced")]
    {
        return auto_ui_iced::run_app::<AccordionApp>();
    }

    #[cfg(feature = "gpui")]
    {
        return auto_ui_gpui::run_app::<AccordionApp>("Accordion Example");
    }

    #[cfg(not(any(feature = "iced", feature = "gpui")))]
    {
        Err("No backend enabled. Please enable either 'iced' or 'gpui' feature in Cargo.toml.".into())
    }
}
