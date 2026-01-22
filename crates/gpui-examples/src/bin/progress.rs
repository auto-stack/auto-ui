// Progress bar example for gpui-component
use gpui::*;
use gpui_component::{progress::*, slider::*, Root, *};

pub struct ProgressExample {
    progress_state: Entity<SliderState>,
}

impl ProgressExample {
    fn new(cx: &mut Context<Self>) -> Self {
        let progress_state = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.01)
                .default_value(0.5)
        });

        Self { progress_state }
    }
}

impl Render for ProgressExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let progress = self.progress_state.read(cx).value().start();
        let percentage = (progress * 100.0) as i32;

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
                    .child("Progress Bar Example")
            )
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::MEDIUM)
                    .child(format!("{}%", percentage))
            )
            .child(
                div()
                    .h(px(24.0))
                    .w(px(300.0))
                    .bg(rgb(0x222222))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(0x444444))
                    .child(
                        div()
                            .h(px(24.0))
                            .w(px(progress * 300.0))
                            .bg(rgb(0x3b82f6))
                            .rounded_lg()
                    )
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .items_center()
                    .child(div().text_sm().child("0%"))
                    .child(div().flex_1().child(Slider::new(&self.progress_state).horizontal()))
                    .child(div().text_sm().child("100%"))
            )
            .child(
                div()
                    .text_sm()
                    .child("Use the slider to adjust the progress")
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
                        title: Some("Progress Bar - GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| ProgressExample::new(cx));
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
