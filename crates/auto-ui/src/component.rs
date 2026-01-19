// Component abstraction

use std::fmt::Debug;

/// Core component trait
pub trait Component: Debug {
    type Message: Debug;

    fn view(&self) -> crate::view::View;

    fn update(&mut self, _message: Self::Message) -> crate::widget::Command<Self::Message> {
        crate::widget::Command::none()
    }
}
