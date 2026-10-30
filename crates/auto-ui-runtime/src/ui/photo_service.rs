//! Compatibility path for the renderer-neutral local photo service.
//! PLAN-890: implementation and original tests have one owner in auto-media-core;
//! the UI feature gate and public names stay; values use the neutral owner.
pub use auto_media_core::photo_service::*;
