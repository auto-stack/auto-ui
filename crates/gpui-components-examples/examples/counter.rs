use gpui::*;
use gpui_component::{ button::Button, label::Label};
use auto_ui::*;
use gpui_story::*;

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);
        gpui_story::create_new_window("Counter Example", StoryView::view::<CounterStory>, cx);
    });
}

pub struct CounterStory {
    focus_handle: gpui::FocusHandle,
    count: i32,
}

impl Story for CounterStory {
    fn title() -> &'static str {
        "Counter"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl CounterStory {
    pub(crate) fn new(_w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            count: 0
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn on(&mut self, ev: SharedString) {
        if ev == "button-inc" {
            self.count = self.count + 1
        } else if ev == "button-dec" {
            self.count = self.count - 1
        }
    }
    
}

impl Focusable for CounterStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CounterStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        center()
            .child(
                col()
                    .child(
                        Button::new("+")
                            .label("+")
                            .on_click(
                                cx
                                    .listener(|v, _, _, cx| {
                                        v.on("button-inc".into());
                                        cx.notify();
                                    }),
                            ),
                    )
                    .child(Label::new(self.count.to_string()))
                    .child(
                        Button::new("-")
                            .label("-")
                            .on_click(
                                cx
                                    .listener(|v, _, _, cx| {
                                        v.on("button-dec".into());
                                        cx.notify();
                                    }),
                            ),
                    ),
            )
    }
    
}
