// Core widget abstraction
use crate::view::View;
use std::marker::PhantomData;

/// Trait for all UI widgets
pub trait Widget: Sized {
    type Message;
    type Props;

    /// Create a view for this widget
    fn view(&self) -> View;

    /// Handle messages
    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }
}

/// Command type for side effects
pub enum Command<M> {
    None,
    Batch(Vec<Box<Command<M>>>),
    // More to be added
    _Marker(PhantomData<M>),
}

impl<M> Command<M> {
    pub fn none() -> Self {
        Command::None
    }

    pub fn batch(commands: Vec<Command<M>>) -> Self {
        Command::Batch(commands.into_iter().map(Box::new).collect())
    }
}
