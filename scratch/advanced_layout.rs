// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct LayoutDemo {
}

impl LayoutDemo {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Component for LayoutDemo {
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
        View::col().spacing(20).padding(10)
    .child(View::text(&"Advanced Layout Demo"))
    .child(View::text(&"────────────────────"))
    .child(View::text(&"This example shows spacing and padding"))
    .build()
    }
}


