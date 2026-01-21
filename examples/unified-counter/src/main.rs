// Unified Counter Example
//
// This example demonstrates how to write a single Component
// that works with both Iced and GPUI backends.
//
// Run with:
//   cargo run --package unified-counter --features iced
//   cargo run --package unified-counter --features gpui

use auto_ui::{Component, View};

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
            .child(View::button("Increment (+)", Message::Increment))
            .child(View::text(format!("Count: {}", self.count)))
            .child(View::button("Decrement (-)", Message::Decrement))
            .build()
    }
}

// Unified main() function - selects backend via feature flags
fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "iced")]
    {
        println!("Running with Iced backend");
        return auto_ui_iced::run_app::<Counter>();
    }

    #[cfg(feature = "gpui")]
    {
        println!("Running with GPUI backend");
        // Note: GPUI requires manual Render trait implementation
        // This is a limitation of GPUI's architecture
        return auto_ui_gpui::run_app::<Counter>();
    }

    #[cfg(not(any(feature = "iced", feature = "gpui")))]
    {
        Err(
            "No backend enabled!\n\n\
             Please run with a backend feature:\n\
             - cargo run --features iced\n\
             - cargo run --features gpui"
                .into(),
        )
    }
}
