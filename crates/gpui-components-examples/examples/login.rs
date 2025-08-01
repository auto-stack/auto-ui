use gpui::*;
use gpui::prelude::FluentBuilder;
use auto_ui::*;
use gpui_story::*;
use gpui_component::{
    ActiveTheme,
    h_flex,
    input::{TextInput, InputState},
    button::Button,
    form::{v_form, form_field}
};

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);
        create_new_window("Login Example", StoryView::view::<LoginStory>, cx);
    });
}

pub struct LoginStory {
    focus_handle: gpui::FocusHandle,
    name_input: Entity<InputState>,
    password_input: Entity<InputState>,
    status: SharedString,
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
            name_input: cx.new(|cx| InputState::new(w, cx)),
            password_input: cx.new(|cx| InputState::new(w, cx)),
            status: SharedString::default(),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        row().center().child(
            col()
            .id("login-story")
            .border_1()
            .border_color(cx.theme().border)
            .p_4()
            .rounded_lg()
            .gap_6()
            .w_2_5()
            .child(
                row().w_begin().child(
                    v_form()
                        .child(
                            form_field()
                                .label("Name: ")
                                .child(TextInput::new(&self.name_input)),
                        )
                        .child(
                            form_field()
                                .label("Password: ")
                                .child(TextInput::new(&self.password_input)),
                        )
                    ))
            .child(
                h_flex()
                    .w_full()
                    .gap_5()
                    .child(Button::new("login").label("Login").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.status = SharedString::from("Logging in...");
                            println!("Username: {}", this.name_input.read(cx).value());
                        })
                    ))
                    .child(div().flex_grow())
                    .child(Button::new("cancel").label("Cancel").on_click(Self::on_cancel))
            )
            .when(!self.status.is_empty(), |e| e.child(self.status.clone())))
    }
}
