use auto_ui::*;

use gpui::{
    App, AppContext, Application, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    SharedString, Window,
};

use gpui_component::{button::Button, label::Label};

pub struct HelloStory {
    focus_handle: gpui::FocusHandle,
    msg: SharedString,
    button_label: SharedString,
}

impl Story for HelloStory {
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

impl HelloStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            msg: SharedString::new("Hello World"),
            button_label: SharedString::new("Click"),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn on(&mut self, ev: SharedString) {
        self.msg = ev
    }
}

impl Focusable for HelloStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for HelloStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        center().child(
            col().child(Label::new(self.msg.clone())).child(
                Button::new(self.button_label.clone())
                    .label(self.button_label.clone())
                    .on_click(cx.listener(|v, _, _, cx| {
                        v.on("button-clicked".into());
                        cx.notify();
                    })),
            ),
        )
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Hello Example", StoryView::view::<HelloStory>, cx, 800, 600);
    });
}
