// auto-ui: Core framework crate
// Backend-agnostic UI framework built on Auto language
//
// This crate provides the abstraction layer that can be adapted to multiple backends
// (iced, gpui, vue.js, etc.) through a unified Component trait and View system.

pub mod prelude {
    pub use crate::app::{App, AppResult};
    pub use crate::component::Component;
    pub use crate::view::{View, ViewBuilder, ViewContainerBuilder, ViewScrollableBuilder, ViewListBuilder, ViewInputBuilder, ViewTableBuilder};
}

pub mod app;
pub mod component;
pub mod view;
pub mod style; // Unified styling system

// Transpiler module for Auto language support
#[cfg(feature = "transpiler")]
pub mod trans;

// Re-export core types for convenience
pub use app::{App, AppResult};
pub use component::Component;
pub use view::{View, ViewBuilder, ViewContainerBuilder, ViewScrollableBuilder, ViewListBuilder, ViewInputBuilder, ViewTableBuilder};

// Note: widget.rs is kept for backward compatibility but may be deprecated
// The new design uses Component trait directly instead of Widget trait
pub mod widget;
