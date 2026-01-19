// auto-ui: Core framework crate
// Backend-agnostic UI framework built on Auto language

pub mod prelude {
    pub use crate::widget::Widget;
    pub use crate::view::View;
}

pub mod widget;
pub mod view;
pub mod component;

// Core traits and types will be defined here
