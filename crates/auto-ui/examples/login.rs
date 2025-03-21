use auto_ui::*;
use gpui::prelude::FluentBuilder;
use auto_ui::row;
use gpui_component::ActiveTheme;

use gpui::{
    Application, App, AppContext, Context, Entity, Focusable, ClickEvent, 
    Render, Window, SharedString, IntoElement, ParentElement,
};

use gpui_component::{
    h_flex,
    input::TextInput,
    button::Button,
    label::Label,
    form::{v_form, form_field}
};

pub struct LoginStory {
    focus_handle: gpui::FocusHandle,
    username: SharedString,
    password: SharedString,
    status: SharedString,
    input_username: Entity<TextInput>,
    input_password: Entity<TextInput>,
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
            username: SharedString::new("nil"),
            password: SharedString::new("nil"),
            status: SharedString::new(""),
            input_username: cx.new(|cx| TextInput::new(w, cx)),
            input_password: cx.new(|cx| TextInput::new(w, cx)),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn on(&mut self, ev: SharedString) {
        self.status = format!("Login {} ...", self.username).into();
    }
    
}

impl Focusable for LoginStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LoginStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.username = self.input_username.read(cx).text();
        self.password = self.input_password.read(cx).text();
        center()
            .child(
                col()
                    .child(Label::new("Username"))
                    .child(self.input_username.clone())
                    .child(Label::new("Password"))
                    .child(self.input_password.clone())
                    .child(
                        Button::new("Login")
                            .label("Login")
                            .on_click(
                                cx
                                    .listener(|v, _, _, cx| {
                                        v.on("button-login".into());
                                        cx.notify();
                                    }),
                            ),
                    )
                    .child(Label::new(self.username.clone()))
                    .child(Label::new(self.status.clone())),
            )
    }
    
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Login Example", StoryView::view::<LoginStory>, cx, 800, 600);
    });
}
