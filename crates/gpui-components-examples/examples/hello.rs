use gpui::*;
use gpui_component::label::Label;
use gpui_story::*;
use auto_ui::*;
use auto_ui::assets::Assets;

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_story::init(cx);
        cx.activate(true);
        gpui_story::create_new_window("Hello Example", StoryView::view::<HelloStory>, cx);
    });
}

pub struct HelloStory {
    focus_handle: gpui::FocusHandle,
    msg: SharedString,
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
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        center().child(
            col().child(Label::new(self.msg.clone()).text_size(px(41.)))
        )
    }
}

