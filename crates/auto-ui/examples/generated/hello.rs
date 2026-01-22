// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct Hello {
    pub msg: String,
    pub style: String,
}

impl Hello {
    pub fn new(msg: String, style: String) -> Self {
        Self {
            msg,
            style,
        }
    }
}

impl Component for Hello {
    type Msg = /* unknown type */;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::text(&self.msg)
    }
}


