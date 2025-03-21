use autogui::app::SimpleApp;
use autogui::app::SimpleRootView;
use autogui::app::Viewable;
use autogui::widget::button::*;
use autoui::dyna::state::*;
use gpui::*;


struct CounterView {
    state: State,
    builder: Option<Box<dyn Fn(Div, &mut State, &mut ViewContext<Self>) -> Div + 'static>>,
}

impl Viewable for CounterView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        let mut state = State::new();
        state.set_int("count", 0);
        Self {
            state,
            builder: None,
        }
    }
}

impl CounterView {
    pub fn contents(
        mut self,
        builder: impl Fn(Div, &mut State, &mut ViewContext<Self>) -> Div + 'static,
    ) -> Self {
        self.builder = Some(Box::new(builder));
        self
    }
}

impl Render for CounterView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let div = div().flex().flex_col();
        if let Some(builder) = self.builder.as_ref() {
            builder(div, &mut self.state, cx)
        } else {
            div
        }
    }
}

fn main() {
    SimpleApp::new().run(true, |cx| {
        cx.new_view(|cx| SimpleRootView::new(cx.new_view(|cx| {
            CounterView::new(cx)
                .contents(|div, state, cx| {
                    div.child(format!("Count: {}", state.get_int("count")))
                        .child(Button::primary("+").on_click_mut::<CounterView>(cx, |this, _ev, _cx| {
                            this.state.inc_int("count");
                        }))
                })
        })))
    });
}
