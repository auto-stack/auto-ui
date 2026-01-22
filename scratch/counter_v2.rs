// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct Counter {
    pub count: i32,
}

impl Counter {
    pub fn new(count: i32) -> Self {
        Self {
            count,
        }
    }
}

impl Component for Counter {
    type Msg = i32;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            1 => {
            }
            2 => {
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
    .child(View::button("+", 1))
    .child(View::text(&""))
    .child(View::button("-", 2))
    .build()
    }
}


