// Temperature Converter example using auto-ui abstraction layer with GPUI adapter
//
// This demonstrates bidirectional data flow and computed values

use auto_ui::{Component, View};
use auto_ui_gpui::ComponentGpui;
use gpui::*;
use gpui_component::Root;
use std::fmt::Debug;

#[derive(Debug, Default, Clone)]
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
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::col()
                    .spacing(8)
                    .padding(16)
                    .child(View::text(format!(
                        "Fahrenheit: {:.1}°F",
                        self.fahrenheit
                    )))
                    .child(
                        View::row()
                            .spacing(8)
                            .padding(0)
                            .child(View::button("-1°F", Message::DecrementFahrenheit))
                            .child(View::button("+1°F", Message::IncrementFahrenheit))
                            .build(),
                    )
                    .build(),
            )
            .child(View::button("Reset", Message::Reset))
            .build()
    }
}

// GPUI Renderer for TempConverter
#[derive(Clone)]
struct TempConverterRenderer {
    converter: TempConverter,
}

impl TempConverterRenderer {
    fn new() -> Self {
        Self {
            converter: TempConverter::default(),
        }
    }
}

impl Render for TempConverterRenderer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.converter.view_gpui_static()
    }
}

fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point { x: px(100.0), y: px(100.0) },
                        size: gpui::Size {
                            width: px(800.0),
                            height: px(600.0),
                        },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Temperature Converter - AutoUI GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| TempConverterRenderer::new());
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
