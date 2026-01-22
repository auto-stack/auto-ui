// Counter example for gpui-component
use gpui::*;
use gpui_component::{button::*, Root, *};

pub struct Counter {
    count: i64,
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.count;

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
                        view.count += 1;
                    })),
            )
            .child(div().text_size(px(50.0)).child(count.to_string()))
            .child(
                Button::new("dec")
                    .primary()
                    .label("-")
                    .on_click(cx.listener(|view, _, _, _cx| {
                        view.count -= 1;
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
                        size: gpui::Size { width: px(800.0), height: px(600.0) },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Counter - GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| Counter { count: 0 });
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
