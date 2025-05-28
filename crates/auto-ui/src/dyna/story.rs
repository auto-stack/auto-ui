use super::{Dyna, Snip};
use crate::story::Story;
use crate::{center, col, row};

use gpui::{
    App, AppContext, Application, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    SharedString, Window,
};

pub struct DynaStory {
    focus_handle: gpui::FocusHandle,
    dyna: Entity<Dyna>,
}

impl Story for DynaStory {
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

impl DynaStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        let snip = Snip::new("Hello, World!");
        Self {
            focus_handle: cx.focus_handle(),
            dyna: cx.new(|cx| Dyna::new(snip, cx)),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for DynaStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DynaStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        center().child(col().child(self.dyna.clone()))
    }
}
