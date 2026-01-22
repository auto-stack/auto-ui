// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct LayoutDemo {
    pub style: String,
}

impl LayoutDemo {
    pub fn new(style: String) -> Self {
        Self {
            style,
        }
    }
}

impl Component for LayoutDemo {
    type Msg = /* unknown type */;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
    .child(View::text(&"Layout Examples"))
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&"Column Layout"))
    .child(View::text(&"Item 1"))
    .child(View::text(&"Item 2"))
    .child(View::text(&"Item 3"))
    .build())
    .child(View::row().spacing(0).padding(0)
    .child(View::text(&"Row Layout"))
    .child(View::text(&"Left"))
    .child(View::text(&"Middle"))
    .child(View::text(&"Right"))
    .build())
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&"Nested Layout"))
    .child(View::row().spacing(0).padding(0)
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&"A1"))
    .child(View::text(&"A2"))
    .build())
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&"B1"))
    .child(View::text(&"B2"))
    .build())
    .build())
    .build())
    .build()
    }
}


