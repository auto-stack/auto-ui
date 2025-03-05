use auto_ui::story::*;
// use gpui::prelude::FluentBuilder as _;

use gpui::{
    div, Application, Styled, App, AppContext, Context, Entity, Focusable, ClickEvent, InteractiveElement, IntoElement, ParentElement, Render, Window,
};

use gpui_component::{
    h_flex,
    gray_800,
    input::TextInput,
    button::Button,
    form::{v_form, form_field}
};

pub struct LoginStory {
    focus_handle: gpui::FocusHandle,
    name_input: Entity<TextInput>,
    password_input: Entity<TextInput>,
}

impl Story for LoginStory {
    fn title() -> &'static str {
        "Login"
    }

    fn description() -> &'static str {
        "Login Example"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl LoginStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            name_input: cx.new(|cx| TextInput::new(w, cx)),
            password_input: cx.new(|cx| TextInput::new(w, cx)),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn on_login(_ev: &ClickEvent, _w: &mut Window, _cx: &mut App) {
        println!("Login");
    }

    fn on_cancel(_ev: &ClickEvent, _w: &mut Window, _cx: &mut App) {
        println!("Cancel");
    }
}

impl Focusable for LoginStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LoginStory {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .id("login-story")
            .flex_col()
            .items_center()
            .justify_center()
            .border_1()
            .p_4()
            .rounded_lg()
            .gap_6()
            .w_2_5()
            .child(
                v_form()
                    .child(
                        form_field()
                            .label("Name: ")
                            .child(self.name_input.clone()),
                    )
                    .child(
                        form_field()
                            .label("Password: ")
                            .child(self.password_input.clone()),
                    )
                )
            .child(
                h_flex()
                    .w_full()
                    .gap_5()
                    .child(Button::new("login").label("Login").on_click(Self::on_login))
                    .child(div().flex_grow())
                    .child(Button::new("cancel").label("Cancel").on_click(Self::on_cancel))
            )
    }
}

pub struct Login {
    root: Entity<LoginStory>,
}

impl Login {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root = LoginStory::view(window, cx);

        Self { root }
    }

    fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for Login {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().p_4().size_full().flex().flex_col().items_center().justify_center().child(self.root.clone())
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Login Example", Login::view, cx, 800, 600);
    });
}
