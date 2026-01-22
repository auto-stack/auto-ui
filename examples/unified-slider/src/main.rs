// Slider example
//
// Demonstrates slider widgets and value handling
//
// Run with:
//   cargo run --package slider-example --features iced

use auto_ui::{Component, View, App};

#[derive(Debug, Default)]
struct SliderExample {
    value: f32,
    volume: f32,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ValueChanged(f32),
    VolumeChanged(f32),
}

impl Component for SliderExample {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ValueChanged(value) => self.value = value,
            Message::VolumeChanged(volume) => self.volume = volume,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(30)
            .padding(40)
            .child(View::text("Slider Controls"))
            .child(
                // Value Slider Section
                View::col()
                    .spacing(10)
                    .padding(20)
                    .child(View::text("Value:"))
                    .child(View::text(format!("{:.2}", self.value)))
                    .child(View::slider(0.0..=100.0, self.value, Message::ValueChanged).build())
                    .build()
            )
            .child(
                // Volume Slider Section
                View::col()
                    .spacing(10)
                    .padding(20)
                    .child(View::text("Volume:"))
                    .child(View::text(format!("{:.1}%", self.volume * 100.0)))
                    .child(
                        View::slider(0.0..=1.0, self.volume, Message::VolumeChanged)
                            .step(0.01)
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
        println!("🎨 Running slider example with Iced backend");
        return auto_ui_iced::run_app::<SliderExample>();
    }

    #[cfg(feature = "gpui")]
    {
        println!("🎨 Running slider example with GPUI backend");
        return auto_ui_gpui::run_app::<SliderExample>("Slider - AutoUI");
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
