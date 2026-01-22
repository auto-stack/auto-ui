// Progress bar example
//
// Demonstrates progress bars with slider control
//
// Run with:
//   cargo run --package progress-example --features iced

use auto_ui::{Component, View, App};

#[derive(Debug, Default)]
struct ProgressExample {
    progress: f32,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ProgressChanged(f32),
}

impl Component for ProgressExample {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ProgressChanged(value) => self.progress = value,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(40)
            .child(View::text("Progress Bar Example"))
            .child(View::text(format!("{:.1}%", self.progress * 100.0)))
            .child(View::progress_bar(self.progress))
            .child(View::slider(0.0..=1.0, self.progress, Message::ProgressChanged).build())
            .child(View::text("Use the slider to adjust the progress"))
            .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "iced")]
    {
        println!("🎨 Running progress bar example with Iced backend");
        return auto_ui_iced::run_app::<ProgressExample>();
    }

    #[cfg(feature = "gpui")]
    {
        println!("🎨 Running progress bar example with GPUI backend");
        return auto_ui_gpui::run_app::<ProgressExample>("Progress - AutoUI");
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
