// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct Counter {
    pub count: i32,
    pub style: String,
}

impl Counter {
    pub fn new(count: i32, style: String) -> Self {
        Self {
            count,
            style,
        }
    }
}

impl Component for Counter {
    type Msg = /* unknown type */;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            "inc" => {
            }
            "dec" => {
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
    .child(View::text(&""))
    .build()
    }
}


