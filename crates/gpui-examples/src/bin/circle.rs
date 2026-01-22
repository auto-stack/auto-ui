// Circle example for gpui-component - custom widget
use gpui::*;
use gpui_component::{slider::*, Root, *};

pub struct CircleExample {
    radius_state: Entity<SliderState>,
}

impl CircleExample {
    fn new(cx: &mut Context<Self>) -> Self {
        let radius_state = cx.new(|_| {
            SliderState::new()
                .min(1.0)
                .max(100.0)
                .step(1.0)
                .default_value(50.0)
        });

        Self { radius_state }
    }
}

impl Render for CircleExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let radius = self.radius_state.read(cx).value().start();
        let diameter = radius * 2.0;

        div()
            .v_flex()
            .gap_6()
            .p_6()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Circle Widget")
            )
            .child(
                // Custom circle widget
                div()
                    .w(px(diameter))
                    .h(px(diameter))
                    .rounded_full()
                    .bg(rgb(0xffdd00)) // Yellow circle
                    .border_2()
                    .border_color(rgb(0x000000))
            )
            .child(
                div()
                    .text_lg()
                    .child(format!("Radius: {:.2}", radius))
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .items_center()
                    .child(div().text_sm().child("1"))
                    .child(div().flex_1().child(Slider::new(&self.radius_state).horizontal()))
                    .child(div().text_sm().child("100"))
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
                        size: gpui::Size { width: px(600.0), height: px(500.0) },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Circle - GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| CircleExample::new(cx));
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
