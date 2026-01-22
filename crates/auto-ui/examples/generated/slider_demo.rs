// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct SliderDemo {
    pub value: /* unknown type */,
    pub volume: /* unknown type */,
    pub progress: /* unknown type */,
    pub style: String,
}

impl SliderDemo {
    pub fn new(value: /* unknown type */, volume: /* unknown type */, progress: /* unknown type */, style: String) -> Self {
        Self {
            value,
            volume,
            progress,
            style,
        }
    }
}

impl Component for SliderDemo {
    type Msg = /* unknown type */;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            "value_changed" => {
            }
            "volume_changed" => {
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
    .child(View::text(&"Slider and Progress Demo"))
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&""))
    .child(/* unknown widget: slider */)
    .build())
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&""))
    .child(/* unknown widget: slider */)
    .build())
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&""))
    .child(/* unknown call: progress_bar */)
    .build())
    .build()
    }
}


