// Checkbox example for gpui-component
use gpui::*;
use gpui_component::{checkbox::*, Root, *};

pub struct CheckboxExample {
    good: bool,
}

impl CheckboxExample {
    fn new() -> Self {
        Self { good: false }
    }
}

impl Render for CheckboxExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let msg = if self.good {
            "I'm feeling GOOD!"
        } else {
            "I'm feeling bad ..."
        };

        div()
            .v_flex()
            .gap_6()
            .p_6()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Checkbox Example")
            )
            .child(
                Checkbox::new("check1")
                    .label("Feeling Good?")
                    .selected(self.good)
                    .on_click(cx.listener(|view, _, _, _cx| {
                        view.good = !view.good;
                    }))
            )
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child(msg)
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
                        size: gpui::Size { width: px(600.0), height: px(400.0) },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Checkbox - GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| CheckboxExample::new());
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
