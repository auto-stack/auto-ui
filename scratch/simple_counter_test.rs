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
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
    }
}


