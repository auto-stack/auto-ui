// Input example demonstrating text input fields
//
// This shows how to use input fields for data entry with various configurations

use auto_ui::{Component, View};
use gpui::*;
use gpui_component::{button::Button, button::ButtonVariants, *};
use std::fmt::Debug;

#[derive(Debug, Default, Clone)]
struct InputApp {
    username: String,
    email: String,
    password: String,
    bio: String,
}

#[derive(Clone, Debug)]
enum Message {
    UsernameChanged(String),
    EmailChanged(String),
    PasswordChanged(String),
    BioChanged(String),
    Clear,
}

impl Component for InputApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::UsernameChanged(value) => {
                self.username = value;
            }
            Message::EmailChanged(value) => {
                self.email = value;
            }
            Message::PasswordChanged(value) => {
                self.password = value;
            }
            Message::BioChanged(value) => {
                self.bio = value;
            }
            Message::Clear => {
                self.username.clear();
                self.email.clear();
                self.password.clear();
                self.bio.clear();
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("User Information Form".to_string()))
            .child(self.view_form())
            .child(self.view_summary())
            .build()
    }
}

impl InputApp {
    fn view_form(&self) -> View<Message> {
        View::col()
            .spacing(16)
            .child(
                View::col()
                    .spacing(4)
                    .child(View::text("Username:".to_string()))
                    .child(
                        View::input("Enter your username")
                            .value(&self.username)
                            .width(300)
                            .on_change(Message::UsernameChanged(self.username.clone()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::col()
                    .spacing(4)
                    .child(View::text("Email:".to_string()))
                    .child(
                        View::input("user@example.com")
                            .value(&self.email)
                            .width(300)
                            .on_change(Message::EmailChanged(self.email.clone()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::col()
                    .spacing(4)
                    .child(View::text("Password:".to_string()))
                    .child(
                        View::input("Enter password")
                            .value(&self.password)
                            .width(300)
                            .password()
                            .on_change(Message::PasswordChanged(self.password.clone()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::col()
                    .spacing(4)
                    .child(View::text("Bio:".to_string()))
                    .child(
                        View::input("Tell us about yourself")
                            .value(&self.bio)
                            .width(400)
                            .on_change(Message::BioChanged(self.bio.clone()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::row()
                    .spacing(8)
                    .child(View::button("Clear Form", Message::Clear))
                    .build(),
            )
            .build()
    }

    fn view_summary(&self) -> View<Message> {
        View::container(
            View::col()
                .spacing(8)
                .child(View::text("Form Summary:".to_string()))
                .child(View::text(format!("Username: {}", self.username)))
                .child(View::text(format!("Email: {}", self.email)))
                .child(View::text(format!(
                    "Password: {}",
                    if self.password.is_empty() {
                        "(empty)"
                    } else {
                        "••••••••"
                    }
                )))
                .child(View::text(format!("Bio: {}", self.bio)))
                .build(),
        )
        .padding(20)
        .width(400)
        .build()
    }
}

// GPUI Renderer for InputApp
struct InputRenderer {
    app: InputApp,
}

impl InputRenderer {
    fn new() -> Self {
        Self {
            app: InputApp::default(),
        }
    }
}

impl Render for InputRenderer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let username = self.app.username.clone();
        let email = self.app.email.clone();
        let password = self.app.password.clone();
        let bio = self.app.bio.clone();

        div()
            .v_flex()
            .gap_4()
            .p_4()
            .size_full()
            .child(div().text_xl().child("User Information Form"))
            .child(
                div()
                    .v_flex()
                    .gap_4()
                    .child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(div().child("Username:"))
                            .child(
                                div()
                                    .child(format!(
                                        "{}",
                                        if username.is_empty() {
                                            "Enter your username"
                                        } else {
                                            &username
                                        }
                                    ))
                                    .w(px(300.0))
                                    .p_2()
                                    .bg(gpui::rgb(0x333333)),
                            ),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(div().child("Email:"))
                            .child(
                                div()
                                    .child(format!(
                                        "{}",
                                        if email.is_empty() {
                                            "user@example.com"
                                        } else {
                                            &email
                                        }
                                    ))
                                    .w(px(300.0))
                                    .p_2()
                                    .bg(gpui::rgb(0x333333)),
                            ),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(div().child("Password:"))
                            .child(
                                div()
                                    .child(format!(
                                        "{}",
                                        if password.is_empty() {
                                            "Enter password"
                                        } else {
                                            "••••••••"
                                        }
                                    ))
                                    .w(px(300.0))
                                    .p_2()
                                    .bg(gpui::rgb(0x333333)),
                            ),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(div().child("Bio:"))
                            .child(
                                div()
                                    .child(format!(
                                        "{}",
                                        if bio.is_empty() {
                                            "Tell us about yourself"
                                        } else {
                                            &bio
                                        }
                                    ))
                                    .w(px(400.0))
                                    .p_2()
                                    .bg(gpui::rgb(0x333333)),
                            ),
                    )
                    .child(
                        Button::new("clear")
                            .label("Clear Form")
                            .small()
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::Clear);
                            })),
                    ),
            )
            .child(
                div()
                    .p_5()
                    .w(px(400.0))
                    .bg(gpui::rgb(0x2a2a2a))
                    .v_flex()
                    .gap_2()
                    .child("Form Summary:")
                    .child(div().child(format!("Username: {}", username)))
                    .child(div().child(format!("Email: {}", email)))
                    .child(div().child(format!(
                        "Password: {}",
                        if password.is_empty() {
                            "(empty)"
                        } else {
                            "••••••••"
                        }
                    )))
                    .child(div().child(format!("Bio: {}", bio))),
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
                        title: Some("Input Demo - AutoUI GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| InputRenderer::new());
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
