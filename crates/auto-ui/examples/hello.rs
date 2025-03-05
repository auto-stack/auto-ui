use auto_ui::story::*;
use gpui::*;

use gpui::{
    div, App, AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Window,
};

use gpui_component::{h_flex, label::Label, v_flex};

pub struct HelloStory {
    focus_handle: gpui::FocusHandle,
}

impl Story for HelloStory {
    fn title() -> &'static str {
        "Hello"
    }

    fn description() -> &'static str {
        "Hello Examples"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl HelloStory {
    pub(crate) fn new(_: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    #[allow(unused)]
    fn on_click(checked: &bool, window: &mut Window, cx: &mut App) {
        println!("Check value changed: {}", checked);
    }
}

impl Focusable for HelloStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for HelloStory {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().size_full().items_center().justify_center().child(
            h_flex()
                .w_full()
                .justify_center()
                .items_center()
                .child(Label::new("Hello World!")),
        )
    }
}

pub struct Hello {
    root: Entity<HelloStory>,
}

impl Hello {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root = HelloStory::view(window, cx);

        Self { root }
    }

    fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for Hello {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().p_4().size_full().child(self.root.clone())
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Hello Example", Hello::view, cx, 800, 600);
    });
}
