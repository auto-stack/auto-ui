// Counter example using auto-ui abstraction layer with Iced adapter
//
// This demonstrates how to use the Component trait and View abstraction
// together with the auto-ui-iced adapter to render with Iced framework.

use auto_ui::{Component, View};
use auto_ui_iced::ComponentIced;
use std::fmt::Debug;

#[derive(Debug, Default)]
struct Counter {
    count: i64,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Increment,
    Decrement,
}

impl Component for Counter {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(10)
            .padding(20)
            .child(View::button("+", Message::Increment))
            .child(View::text(self.count.to_string()))
            .child(View::button("-", Message::Decrement))
            .build()
    }
}

fn main() -> iced::Result {
    // Use the ComponentIced trait to run with iced
    iced::run(Counter::update, Counter::view_iced)
}

// Note: ComponentIced trait provides:
// - update() method that delegates to on()
// - view_iced() method that converts View<M> to iced::Element<'static, M>
