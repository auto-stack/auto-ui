// Counter example using auto-ui abstraction layer with GPUI adapter
//
// This demonstrates how to use the Component trait and View abstraction
// together with the auto-ui-gpui adapter to render with GPUI framework.

use auto_ui::{Component, View};
use gpui::*;
use gpui_component::{button::Button, button::ButtonVariants, *};
use std::fmt::Debug;

#[derive(Debug, Clone)]
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
            .child(View::button("+", Message::Increment))
            .child(View::text(self.count.to_string()))
            .child(View::button("-", Message::Decrement))
            .build()
    }
}

// GPUI Renderer for Counter
struct CounterRenderer {
    counter: Counter,
}

impl CounterRenderer {
    fn new() -> Self {
        Self {
            counter: Counter { count: 0 },
        }
    }
}

impl Render for CounterRenderer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.counter.count;

        div()
            .v_flex()
            .gap_3()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                Button::new("inc")
                    .primary()
                    .label("+")
                    .on_click(cx.listener(|view, _, _, _cx| {
                        view.counter.on(Message::Increment);
                    })),
            )
            .child(div().text_size(px(50.0)).child(count.to_string()))
            .child(
                Button::new("dec")
                    .primary()
                    .label("-")
                    .on_click(cx.listener(|view, _, _, _cx| {
                        view.counter.on(Message::Decrement);
                    })),
            )
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
                        title: Some("Counter - AutoUI GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| CounterRenderer::new());
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
