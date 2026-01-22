// Slider example for iced
// Demonstrates slider widgets and value handling
use iced::widget::{center, column, slider, text};
use iced::Element;

fn main() -> iced::Result {
    iced::run(SliderExample::update, SliderExample::view)
}

#[derive(Default)]
struct SliderExample {
    value: f32,
    volume: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ValueChanged(f32),
    VolumeChanged(f32),
}

impl SliderExample {
    fn update(&mut self, message: Message) {
        match message {
            Message::ValueChanged(value) => self.value = value,
            Message::VolumeChanged(volume) => self.volume = volume,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        center(
            column![
                text("Slider Controls").size(30),
                column![
                    text("Value:").size(20),
                    text(format!("{:.2}", self.value)).size(18),
                    slider(0.0..=100.0, self.value, Message::ValueChanged),
                ]
                .spacing(10)
                .padding(20),
                column![
                    text("Volume:").size(20),
                    text(format!("{:.1}%", self.volume * 100.0)).size(18),
                    slider(0.0..=1.0, self.volume, Message::VolumeChanged),
                ]
                .spacing(10)
                .padding(20),
            ]
            .spacing(30)
            .padding(40),
        )
        .into()
    }
}
