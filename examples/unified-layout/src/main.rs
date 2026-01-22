// Layout examples
//
// Demonstrates various layout patterns: rows, columns, nesting
//
// Run with:
//   cargo run --package layout-example --features iced

use auto_ui::{Component, View, App};

#[derive(Debug, Default)]
struct LayoutExample;

#[derive(Clone, Copy, Debug)]
enum Message {
    NoOp,
}

impl Component for LayoutExample {
    type Msg = Message;

    fn on(&mut self, _msg: Self::Msg) {
        // No state changes in this example
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(30)
            .padding(40)
            .child(View::text("Layout Examples"))
            .child(
                // Column Layout Section
                View::col()
                    .spacing(10)
                    .padding(20)
                    .child(View::text("Column Layout"))
                    .child(View::text("Item 1"))
                    .child(View::text("Item 2"))
                    .child(View::text("Item 3"))
                    .build()
            )
            .child(
                // Row Layout Section
                View::col()
                    .spacing(10)
                    .padding(20)
                    .child(View::text("Row Layout"))
                    .child(
                        View::row()
                            .spacing(20)
                            .padding(20)
                            .child(View::text("Item 1"))
                            .child(View::text("Item 2"))
                            .child(View::text("Item 3"))
                            .build()
                    )
                    .build()
            )
            .child(
                // Nested Layout Section
                View::col()
                    .spacing(10)
                    .padding(20)
                    .child(View::text("Nested Layout"))
                    .child(
                        View::col()
                            .spacing(5)
                            .child(View::text("Column 1"))
                            .child(
                                View::row()
                                    .spacing(10)
                                    .child(
                                        View::col()
                                            .spacing(5)
                                            .child(View::text("Nested A1"))
                                            .child(View::text("Nested A2"))
                                            .build()
                                    )
                                    .child(
                                        View::col()
                                            .spacing(5)
                                            .child(View::text("Nested B1"))
                                            .child(View::text("Nested B2"))
                                            .build()
                                    )
                                    .build()
                            )
                            .build()
                    )
                    .build()
            )
            .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "iced")]
    {
        println!("🎨 Running layout example with Iced backend");
        return auto_ui_iced::run_app::<LayoutExample>();
    }

    #[cfg(feature = "gpui")]
    {
        println!("🎨 Running layout example with GPUI backend");
        return auto_ui_gpui::run_app::<LayoutExample>("Layout - AutoUI");
    }

    #[cfg(not(any(feature = "iced", feature = "gpui")))]
    {
        Err(
            "❌ No backend enabled!\n\n\
             Please run with a backend feature:\n\
             • cargo run --features iced\n\
             • cargo run --features gpui"
                .into(),
        )
    }
}
