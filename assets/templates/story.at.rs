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

pub struct ${story.name}Story {
    focus_handle: gpui::FocusHandle,
$ for f in story.fields {
    ${f.name}: ${f.kind},
$ }
}

impl Story for ${story.name}Story {
    fn title() -> &'static str {
        "${story.name}"
    }

    fn description() -> &'static str {
        "${story.name} Example"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl ${story.name}Story {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        $ for f in story.fields {
            ${f.name}: ${f.kind}::new("${f.value}"),
        $ }
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

}

impl Focusable for ${story.name}Story {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ${story.name}Story {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        ${story.code}
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("${app.title}", StoryView::view::<${story.name}Story>, cx, 800, 600);
    });
}
