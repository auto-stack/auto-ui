// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct Test {
    pub count: i32,
}

impl Test {
    pub fn new(count: i32) -> Self {
        Self {
            count,
        }
    }
}

impl Component for Test {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        View::text(&"hello")
    }
}


