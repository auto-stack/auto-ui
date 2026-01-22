// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct Hello {
    pub msg: String,
}

impl Hello {
    pub fn new(msg: String) -> Self {
        Self {
            msg,
        }
    }
}

impl Component for Hello {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
    .child(View::text(&self.msg))
    .child(View::text(&"World"))
    .build()
    }
}


