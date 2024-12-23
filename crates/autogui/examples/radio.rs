use gpui::*;
use autogui::widget::radio::Radio;
use autogui::widget::radio::RadioGroup;
use autogui::app::SimpleApp;
use autogui::app::Viewable;

struct RadioView {
    title: SharedString,
    index: usize,
}

impl Viewable for RadioView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            title: "Select a color".into(),
            index: 0,
        }
    }
}

impl Render for RadioView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child(self.title.clone())
            .gap_1()
            .child(
                RadioGroup::new("radio_group")
                    .add(Radio::new("radio_red")
                        .label("Red")
                    )
                    .add(Radio::new("radio_blue")
                        .label("Blue")
                    )
                    .add(Radio::new("radio_green")
                        .label("Green")
                    )
                    .select(self.index)
                    .on_click(cx.listener(|this, v: &usize, _cx| {
                        this.index = *v;
                    }))
            )
    }
}

fn main() {
    SimpleApp::new().run_simple::<RadioView>();
}

