// Scroll example demonstrating scrollable containers
//
// This shows how to use scrollable containers for content overflow

use auto_ui::{Component, View};
use auto_ui_gpui::ComponentGpui;
use gpui::*;
use gpui_component::Root;
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
#[derive(Clone)]
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.app.view_gpui_static()
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
