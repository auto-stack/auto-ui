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
    $ for w in widgets {
        ${w.name}: Entity<${w.kind}>,
    $ }
    $ for d in datas {
        ${d.name}: ${d.kind},
    $ }
}

impl Story for LoginStory {
    fn title() -> &'static str {
        "${title}"
    }

    fn description() -> &'static str {
        "${description}"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl LoginStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            $ for w in widgets {
                ${w.name}: cx.new(|cx| ${w.kind}::new(w, cx)),
            $ }
            $ for d in datas {
                ${d.name}: ${d.kind}::new("${d.value}"),
            $ }
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
        ${code}
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("${title}", StoryView::view::<LoginStory>, cx, 800, 600);
    });
}
