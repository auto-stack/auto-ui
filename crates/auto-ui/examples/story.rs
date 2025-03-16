use auto_ui::*;
use gpui::prelude::FluentBuilder;
use auto_ui::layout::row;
use gpui_component::ActiveTheme;

use gpui::{
    div, Application, Styled, App, AppContext, Context, Entity, Focusable, ClickEvent, 
    InteractiveElement, IntoElement, ParentElement, Render, Window,
    SharedString,
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

        hello: SharedString,
}

impl Story for LoginStory {
    fn title() -> &'static str {
        "Hello"
    }

    fn description() -> &'static str {
        "Hello Example"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl LoginStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),

                hello: SharedString::new("Hello World!"),
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
        center().child(Label::new(self.hello.clone()))
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Hello", StoryView::view::<LoginStory>, cx, 800, 600);
    });
}
