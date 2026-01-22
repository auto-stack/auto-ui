// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct SimpleTest {
    pub count: i32,
    pub enabled: bool,
}

impl SimpleTest {
    pub fn new(count: i32, enabled: bool) -> Self {
        Self {
            count,
            enabled,
        }
    }
}

impl Component for SimpleTest {
    type Msg = i32;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            1 => {
                self.count += 1;
            }
            2 => {
                self.count -= 1;
            }
            3 => {
                self.count = 0;
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
    .child(View::text(&"Simple Components Test"))
    .child(View::text(&"Counter:"))
    .child(View::text(&self.count.to_string()))
    .child(View::text(&"Form elements:"))
    .child(View::input("Enter text").build())
    .child(View::checkbox(false, "Enable"))
    .child(View::text(&"Radio options:"))
    .child(View::radio(true, "Option A"))
    .child(View::radio(false, "Option B"))
    .child(View::text(&"Select:"))
    .child(View::select(vec!["Default"]))
    .child(View::button("Reset", 3))
    .build()
    }
}


