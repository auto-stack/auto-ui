// Slider example for gpui-component
//
// Demonstrates the use of gpui-component's built-in slider widget
// with draggable thumb support.
use gpui::*;
use gpui_component::{slider::*, Root, *};

pub struct SliderExample {
    value_state: Entity<SliderState>,
    volume_state: Entity<SliderState>,
}

impl SliderExample {
    fn new(cx: &mut Context<Self>) -> Self {
        // Create slider states
        let value_state = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .default_value(50.0)
        });

        let volume_state = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.01)
                .default_value(0.5)
        });

        Self {
            value_state,
            volume_state,
        }
    }
}

impl Render for SliderExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get current values
        let value = self.value_state.read(cx).value();
        let volume = self.volume_state.read(cx).value();

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
                    .child("Slider Controls")
            )
            .child(
                // Value Slider Section
                div()
                    .v_flex()
                    .gap_3()
                    .p_4()
                    .border_1()
                    .border_color(rgb(0x333333))
                    .rounded_lg()
                    .w(px(400.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Value")
                    )
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .child(format!("{:.2}", value.start()))
                    )
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().child("0.0"))
                            .child(div().flex_1().child(Slider::new(&self.value_state).horizontal()))
                            .child(div().text_sm().child("100.0"))
                    )
            )
            .child(
                // Volume Slider Section
                div()
                    .v_flex()
                    .gap_3()
                    .p_4()
                    .border_1()
                    .border_color(rgb(0x333333))
                    .rounded_lg()
                    .w(px(400.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Volume")
                    )
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .child(format!("{:.1}%", volume.start() * 100.0))
                    )
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().child("0%"))
                            .child(div().flex_1().child(Slider::new(&self.volume_state).horizontal()))
                            .child(div().text_sm().child("100%"))
                    )
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
                        title: Some("Slider - GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| SliderExample::new(cx));
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
