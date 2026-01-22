use auto_ui::{Component, View};

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

impl Default for Counter {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Component for Counter {
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
        .child(View::button("+", 1))
        .child(View::text(&self.count.to_string()))
        .child(View::button("-", 2))
        .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    auto_ui_gpui::run_app::<Counter>("Counter Test")
}
