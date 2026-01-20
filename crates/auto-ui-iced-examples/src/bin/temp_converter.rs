// Temperature Converter example using auto-ui abstraction layer with Iced adapter
//
// This demonstrates bidirectional data flow and computed values

use auto_ui::{Component, View};
use auto_ui_iced::ComponentIced;

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

fn main() -> iced::Result {
    iced::run(TempConverter::update, view)
}

fn view(converter: &TempConverter) -> iced::Element<'_, Message> {
    converter.view_iced()
}
