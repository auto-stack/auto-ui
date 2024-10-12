use gpui::*;
use autoui::widget::slider::Slider;
use autoui::widget::slider::SliderEvent;
use autoui::app::SimpleApp;
use autoui::app::Viewable;

struct SliderView {
    value: f32,
    slider: View<Slider>,
}

impl Viewable for SliderView {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        let slider = Slider::new("slider").min(0.).max(100.).initial_value(0.0);
        let slider_view = cx.new_view(|_cx| slider);
        cx.subscribe(&slider_view, |this, _, ev, cx| match ev {
            SliderEvent::Move(value) => {
                this.value = *value;
                cx.notify();
            }
        }).detach();
        Self {
            value: 0.0,
            slider: slider_view,
        }
    }
}

impl Render for SliderView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child("Slider Demo:")
            .gap_1()
            .child(self.slider.clone())
            .child(div().child(self.value.to_string()))
    }
}

fn main() {
    SimpleApp::new().title("Slider Example").run_simple::<SliderView>();
}

