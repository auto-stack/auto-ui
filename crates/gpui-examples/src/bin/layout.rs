// Layout example for gpui-component
// Demonstrates various layout patterns: rows, columns, spacing
use gpui::*;
use gpui_component::{Root, *};

pub struct LayoutExample;

impl Render for LayoutExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .gap_6()
            .p_6()
            .size_full()
            .items_center()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Layout Examples")
            )
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::MEDIUM)
                    .child("Column Layout")
            )
            .child(column_layout_example())
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::MEDIUM)
                    .child("Row Layout")
            )
            .child(row_layout_example())
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::MEDIUM)
                    .child("Nested Layout")
            )
            .child(nested_layout_example())
    }
}

fn column_layout_example() -> Div {
    div()
        .v_flex()
        .gap_2()
        .p_4()
        .border_1()
        .border_color(rgb(0x333333))
        .rounded_lg()
        .child(div().text_sm().child("Item 1"))
        .child(div().text_sm().child("Item 2"))
        .child(div().text_sm().child("Item 3"))
}

fn row_layout_example() -> Div {
    div()
        .h_flex()
        .gap_4()
        .p_4()
        .border_1()
        .border_color(rgb(0x333333))
        .rounded_lg()
        .child(div().text_sm().child("Item 1"))
        .child(div().text_sm().child("Item 2"))
        .child(div().text_sm().child("Item 3"))
}

fn nested_layout_example() -> Div {
    div()
        .v_flex()
        .gap_2()
        .p_4()
        .border_1()
        .border_color(rgb(0x333333))
        .rounded_lg()
        .child(div().text_sm().child("Column 1"))
        .child(
            div()
                .h_flex()
                .gap_2()
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(div().text_xs().child("Nested A1"))
                        .child(div().text_xs().child("Nested A2"))
                )
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(div().text_xs().child("Nested B1"))
                        .child(div().text_xs().child("Nested B2"))
                )
        )
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
                        title: Some("Layout - GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| LayoutExample);
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
