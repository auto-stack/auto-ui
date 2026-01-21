// Scroll example demonstrating scrollable containers
//
// This shows how to use scrollable containers for content overflow

use auto_ui::{Component, View};
use gpui::*;
use gpui_component::{button::Button, *};
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct ScrollApp {
    selected_example: Example,
}

impl Default for ScrollApp {
    fn default() -> Self {
        Self {
            selected_example: Example::Basic,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Example {
    Basic,
    LongContent,
    NestedScrollable,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ShowExample(Example),
}

impl Component for ScrollApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ShowExample(example) => {
                self.selected_example = example;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Scrollable Examples".to_string()))
            .child(self.view_navigation())
            .child(self.view_current_example())
            .build()
    }
}

impl ScrollApp {
    fn view_navigation(&self) -> View<Message> {
        View::row()
            .spacing(8)
            .padding(0)
            .child(View::button("Basic", Message::ShowExample(Example::Basic)))
            .child(View::button(
                "Long Content",
                Message::ShowExample(Example::LongContent),
            ))
            .child(View::button("Nested", Message::ShowExample(Example::NestedScrollable)))
            .build()
    }

    fn view_current_example(&self) -> View<Message> {
        match self.selected_example {
            Example::Basic => self.view_basic_example(),
            Example::LongContent => self.view_long_content_example(),
            Example::NestedScrollable => self.view_nested_example(),
        }
    }

    fn view_basic_example(&self) -> View<Message> {
        // Simple scrollable with fixed size
        View::scrollable(
            View::col()
                .spacing(10)
                .child(View::text("Item 1".to_string()))
                .child(View::text("Item 2".to_string()))
                .child(View::text("Item 3".to_string()))
                .child(View::text("Item 4".to_string()))
                .child(View::text("Item 5".to_string()))
                .child(View::text("Item 6".to_string()))
                .child(View::text("Item 7".to_string()))
                .child(View::text("Item 8".to_string()))
                .child(View::text("Item 9".to_string()))
                .child(View::text("Item 10".to_string()))
                .build(),
        )
        .width(300)
        .height(200)
        .build()
    }

    fn view_long_content_example(&self) -> View<Message> {
        // Generate many items to demonstrate scrolling
        let mut items = Vec::new();
        for i in 1..=50 {
            items.push(View::text(format!(
                "Item {}: Long content that scrolls",
                i
            )));
        }

        View::col()
            .spacing(16)
            .child(View::text("Scrollable with 50 items:".to_string()))
            .child(
                View::scrollable(View::col().spacing(8).children(items).build())
                    .height(300)
                    .build(),
            )
            .build()
    }

    fn view_nested_example(&self) -> View<Message> {
        // Nested scrollable containers
        View::col()
            .spacing(16)
            .child(View::text("Nested Scrollable Containers".to_string()))
            .child(
                View::container(
                    View::col()
                        .spacing(10)
                        .child(View::text("Outer container - fixed size".to_string()))
                        .child(
                            View::scrollable(
                                View::col()
                                    .spacing(8)
                                    .children(
                                        (1..=20)
                                            .map(|i| View::text(format!("Nested item {}", i)))
                                            .collect::<Vec<_>>(),
                                    )
                                    .build(),
                            )
                            .height(150)
                            .build(),
                        )
                        .child(View::text("More content outside scrollable".to_string()))
                        .build(),
                )
                .padding(10)
                .build(),
            )
            .build()
    }
}

// GPUI Renderer for ScrollApp
struct ScrollRenderer {
    app: ScrollApp,
}

impl ScrollRenderer {
    fn new() -> Self {
        Self {
            app: ScrollApp::default(),
        }
    }
}

impl Render for ScrollRenderer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_example = self.app.selected_example;

        div()
            .v_flex()
            .gap_4()
            .p_4()
            .size_full()
            .child(div().text_xl().child("Scrollable Examples"))
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .child(
                        Button::new("basic")
                            .label("Basic")
                            .selected(selected_example == Example::Basic)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::ShowExample(Example::Basic));
                            })),
                    )
                    .child(
                        Button::new("long-content")
                            .label("Long Content")
                            .selected(selected_example == Example::LongContent)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::ShowExample(Example::LongContent));
                            })),
                    )
                    .child(
                        Button::new("nested")
                            .label("Nested")
                            .selected(selected_example == Example::NestedScrollable)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::ShowExample(Example::NestedScrollable));
                            })),
                    ),
            )
            .child(match selected_example {
                Example::Basic => {
                    let items: Vec<_> = (1..=10).map(|i| div().child(format!("Item {}", i))).collect();
                    div()
                        .w(px(300.0))
                        .h(px(200.0))
                        .bg(gpui::rgb(0x333333))
                        .v_flex()
                        .gap_2()
                        .overflow_y_hidden()
                        .children(items)
                }
                Example::LongContent => {
                    let items: Vec<_> = (1..=50)
                        .map(|i| div().child(format!("Item {}: Long content that scrolls", i)))
                        .collect();
                    div()
                        .v_flex()
                        .gap_4()
                        .child("Scrollable with 50 items:")
                        .child(
                            div()
                                .h(px(300.0))
                                .bg(gpui::rgb(0x333333))
                                .v_flex()
                                .gap_2()
                                .overflow_y_hidden()
                                .children(items),
                        )
                }
                Example::NestedScrollable => {
                    let nested_items: Vec<_> =
                        (1..=20).map(|i| div().child(format!("Nested item {}", i))).collect();
                    div()
                        .v_flex()
                        .gap_4()
                        .child("Nested Scrollable Containers")
                        .child(
                            div()
                                .p_2()
                                .bg(gpui::rgb(0x333333))
                                .v_flex()
                                .gap_2()
                                .child("Outer container - fixed size")
                                .child(
                                    div()
                                        .h(px(150.0))
                                        .bg(gpui::rgb(0x444444))
                                        .v_flex()
                                        .gap_2()
                                        .overflow_y_hidden()
                                        .children(nested_items),
                                )
                                .child("More content outside scrollable"),
                        )
                }
            })
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
                        title: Some("Scroll Demo - AutoUI GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| ScrollRenderer::new());
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
