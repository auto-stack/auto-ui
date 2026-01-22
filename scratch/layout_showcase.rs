// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct LayoutShowcase {
    pub count: i32,
}

impl LayoutShowcase {
    pub fn new(count: i32) -> Self {
        Self {
            count,
        }
    }
}

impl Component for LayoutShowcase {
    type Msg = i32;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            1 => {
                self.count += 1;
            }
            2 => {
                self.count -= 1;
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
        .child(View::col().spacing(10).padding(20)
    .child(View::text(&"Example 1: Basic Spacing & Padding"))
    .child(View::text(&"Items have 10px spacing between them"))
    .child(View::text(&"Container has 20px padding"))
    .build())
        .child(View::col().spacing(15).padding(0)
    .child(View::text(&"Example 2: Nested Layouts"))
    .child(View::row().spacing(10).padding(5)
    .child(View::text(&"Row Item 1"))
    .child(View::text(&"Row Item 2"))
    .child(View::text(&"Row Item 3"))
    .build())
    .build())
        .child(View::container(View::text(&"Example 3: Centered Content"))
    .center()
    .build())
        .child(View::container(View::text(&"Example 4: Container with padding")).padding(15)
    .build())
        .child(View::row().spacing(20).padding(10)
    .child(View::text(&"Example 5: Row Layout"))
    .child(View::text(&"Item 2"))
    .child(View::text(&"Item 3"))
    .build())
        .child(View::col().spacing(10).padding(0)
    .child(View::text(&"Example 6: Interactive Counter"))
    .child(View::text(&self.count.to_string()))
    .child(View::row().spacing(10).padding(0)
    .child(View::button("Increment", 1))
    .child(View::button("Decrement", 2))
    .build())
    .build())
        .build()
    }
}


