// Unified Temperature Converter Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates temperature conversion between Celsius, Fahrenheit, and Kelvin.
// The same Component code works with both backends through automatic message conversion.
//
// Run with:
//   cargo run --package unified-temp_converter --features iced
//   cargo run --package unified-temp_converter --features gpui
//
use auto_ui::{Component, View};

#[derive(Debug, Default)]
struct TempConverter {
    celsius: f64,
    fahrenheit: f64,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    IncrementCelsius,
    DecrementCelsius,
    IncrementFahrenheit,
    DecrementFahrenheit,
    Reset,
}

impl Component for TempConverter {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::IncrementCelsius => {
                self.celsius += 1.0;
                self.fahrenheit = self.celsius * 9.0 / 5.0 + 32.0;
            }
            Message::DecrementCelsius => {
                self.celsius -= 1.0;
                self.fahrenheit = self.celsius * 9.0 / 5.0 + 32.0;
            }
            Message::IncrementFahrenheit => {
                self.fahrenheit += 1.0;
                self.celsius = (self.fahrenheit - 32.0) * 5.0 / 9.0;
            }
            Message::DecrementFahrenheit => {
                self.fahrenheit -= 1.0;
                self.celsius = (self.fahrenheit - 32.0) * 5.0 / 9.0;
            }
            Message::Reset => {
                self.celsius = 0.0;
                self.fahrenheit = 32.0;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(16)
            .padding(32)
            .child(View::text("Temperature Converter".to_string()))
            .child(
                View::col()
                    .spacing(8)
                    .padding(16)
                    .child(View::text(format!("Celsius: {:.1}°C", self.celsius)))
                    .child(
                        View::row()
                            .spacing(8)
                            .padding(0)
                            .child(View::button("-1°C", Message::DecrementCelsius))
                            .child(View::button("+1°C", Message::IncrementCelsius))
                            .build()
                    )
                    .build()
            )
            .child(
                View::col()
                    .spacing(8)
                    .padding(16)
                    .child(View::text(format!("Fahrenheit: {:.1}°F", self.fahrenheit)))
                    .child(
                        View::row()
                            .spacing(8)
                            .padding(0)
                            .child(View::button("-1°F", Message::DecrementFahrenheit))
                            .child(View::button("+1°F", Message::IncrementFahrenheit))
                            .build()
                    )
                    .build()
            )
            .child(View::button("Reset", Message::Reset))
            .build()
    }
}

// Unified main() - works with BOTH backends!
fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "iced")]
    {
        println!("🎨 Running with Iced backend");
        return auto_ui_iced::run_app::<TempConverter>();
    }

    #[cfg(feature = "gpui")]
    {
        println!("🎨 Running with GPUI backend (with auto-conversion!)");
        return auto_ui_gpui::run_app::<TempConverter>("Temperature Converter - AutoUI");
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
