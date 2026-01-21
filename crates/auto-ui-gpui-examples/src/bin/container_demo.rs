// Container example demonstrating styling and layout options
//
// This shows how to use containers for padding, centering, and sizing

use auto_ui::{Component, View};
use gpui::*;
use gpui_component::{button::Button, *};
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct ContainerApp {
    selected_example: Example,
}

impl Default for ContainerApp {
    fn default() -> Self {
        Self {
            selected_example: Example::Padding,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Example {
    Padding,
    Sizing,
    Centering,
    Nested,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ShowExample(Example),
}

impl Component for ContainerApp {
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
            .child(View::text("Container Examples".to_string()))
            .child(self.view_navigation())
            .child(self.view_current_example())
            .build()
    }
}

impl ContainerApp {
    fn view_navigation(&self) -> View<Message> {
        View::row()
            .spacing(8)
            .padding(0)
            .child(View::button("Padding", Message::ShowExample(Example::Padding)))
            .child(View::button("Sizing", Message::ShowExample(Example::Sizing)))
            .child(View::button("Centering", Message::ShowExample(Example::Centering)))
            .child(View::button("Nested", Message::ShowExample(Example::Nested)))
            .build()
    }

    fn view_current_example(&self) -> View<Message> {
        match self.selected_example {
            Example::Padding => self.view_padding_example(),
            Example::Sizing => self.view_sizing_example(),
            Example::Centering => self.view_centering_example(),
            Example::Nested => self.view_nested_example(),
        }
    }

    fn view_padding_example(&self) -> View<Message> {
        View::col()
            .spacing(16)
            .child(View::container(View::text("No Padding")).padding(0).build())
            .child(
                View::container(View::text("Padding 20"))
                    .padding(20)
                    .build(),
            )
            .child(
                View::container(View::text("Padding 40"))
                    .padding(40)
                    .build(),
            )
            .build()
    }

    fn view_sizing_example(&self) -> View<Message> {
        View::col()
            .spacing(16)
            .child(
                View::container(View::text("Fixed Width: 200"))
                    .width(200)
                    .padding(10)
                    .build(),
            )
            .child(
                View::container(View::text("Fixed Size: 200x100"))
                    .width(200)
                    .height(100)
                    .padding(10)
                    .build(),
            )
            .build()
    }

    fn view_centering_example(&self) -> View<Message> {
        View::col()
            .spacing(16)
            .child(
                View::container(View::text("Centered X"))
                    .width(300)
                    .padding(10)
                    .center_x()
                    .build(),
            )
            .child(
                View::container(View::text("Centered Y"))
                    .width(300)
                    .height(100)
                    .padding(10)
                    .center_y()
                    .build(),
            )
            .child(
                View::container(View::text("Centered Both"))
                    .width(300)
                    .height(100)
                    .padding(10)
                    .center()
                    .build(),
            )
            .build()
    }

    fn view_nested_example(&self) -> View<Message> {
        View::container(
            View::col()
                .spacing(10)
                .child(View::text("Outer Container".to_string()))
                .child(
                    View::container(
                        View::row()
                            .spacing(8)
                            .child(View::text("Nested".to_string()))
                            .child(View::text("Layout".to_string()))
                            .build(),
                    )
                    .padding(20)
                    .center_x()
                    .build(),
                )
                .build(),
        )
        .padding(30)
        .width(400)
        .build()
    }
}

// GPUI Renderer for ContainerApp
struct ContainerRenderer {
    app: ContainerApp,
}

impl ContainerRenderer {
    fn new() -> Self {
        Self {
            app: ContainerApp::default(),
        }
    }
}

impl Render for ContainerRenderer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_example = self.app.selected_example;

        div()
            .v_flex()
            .gap_4()
            .p_4()
            .size_full()
            .child(div().text_xl().child("Container Examples"))
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .child(
                        Button::new("padding")
                            .label("Padding")
                            .selected(selected_example == Example::Padding)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::ShowExample(Example::Padding));
                            })),
                    )
                    .child(
                        Button::new("sizing")
                            .label("Sizing")
                            .selected(selected_example == Example::Sizing)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::ShowExample(Example::Sizing));
                            })),
                    )
                    .child(
                        Button::new("centering")
                            .label("Centering")
                            .selected(selected_example == Example::Centering)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::ShowExample(Example::Centering));
                            })),
                    )
                    .child(
                        Button::new("nested")
                            .label("Nested")
                            .selected(selected_example == Example::Nested)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::ShowExample(Example::Nested));
                            })),
                    ),
            )
            .child(match selected_example {
                Example::Padding => div()
                    .v_flex()
                    .gap_4()
                    .child(div().p_0().bg(gpui::rgb(0x333333)).child("No Padding"))
                    .child(div().p_4().bg(gpui::rgb(0x333333)).child("Padding 20"))
                    .child(div().p_10().bg(gpui::rgb(0x333333)).child("Padding 40")),
                Example::Sizing => div()
                    .v_flex()
                    .gap_4()
                    .child(
                        div()
                            .w(px(200.0))
                            .p_2()
                            .bg(gpui::rgb(0x333333))
                            .child("Fixed Width: 200"),
                    )
                    .child(
                        div()
                            .w(px(200.0))
                            .h(px(100.0))
                            .p_2()
                            .bg(gpui::rgb(0x333333))
                            .child("Fixed Size: 200x100"),
                    ),
                Example::Centering => div()
                    .v_flex()
                    .gap_4()
                    .child(
                        div()
                            .w(px(300.0))
                            .p_2()
                            .bg(gpui::rgb(0x333333))
                            .items_center()
                            .justify_center()
                            .child("Centered X"),
                    )
                    .child(
                        div()
                            .w(px(300.0))
                            .h(px(100.0))
                            .p_2()
                            .bg(gpui::rgb(0x333333))
                            .items_center()
                            .justify_center()
                            .child("Centered Y"),
                    )
                    .child(
                        div()
                            .w(px(300.0))
                            .h(px(100.0))
                            .p_2()
                            .bg(gpui::rgb(0x333333))
                            .items_center()
                            .justify_center()
                            .child("Centered Both"),
                    ),
                Example::Nested => div()
                    .w(px(400.0))
                    .p_8()
                    .bg(gpui::rgb(0x333333))
                    .v_flex()
                    .gap_2()
                    .child("Outer Container")
                    .child(
                        div()
                            .p_5()
                            .bg(gpui::rgb(0x444444))
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .justify_center()
                            .child("Nested")
                            .child("Layout"),
                    ),
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
                        title: Some("Container Demo - AutoUI GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| ContainerRenderer::new());
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
