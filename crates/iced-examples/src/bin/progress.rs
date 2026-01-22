// Progress bar example for iced
// Demonstrates progress bars
use iced::widget::{center, column, progress_bar, slider, text};
use iced::Element;

fn main() -> iced::Result {
    iced::run(ProgressExample::update, ProgressExample::view)
}

#[derive(Default)]
struct ProgressExample {
    progress: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ProgressChanged(f32),
}

impl ProgressExample {
    fn update(&mut self, message: Message) {
        match message {
            Message::ProgressChanged(value) => self.progress = value,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        center(
            column![
                text("Progress Bar Example").size(32),
                text(format!("{:.1}%", self.progress * 100.0)).size(24),
                progress_bar(0.0..=1.0, self.progress),
                slider(0.0..=1.0, self.progress, Message::ProgressChanged),
                text("Use the slider to adjust the progress").size(14),
            ]
            .spacing(20)
            .padding(40),
        )
        .into()
    }
}
