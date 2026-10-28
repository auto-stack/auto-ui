//! AGP — AutoGraphics Prototype core library.
//!
//! Layering contract (PLAN-829 §2):
//! - `core`: GPU-agnostic shared model — stable identities, time, actions /
//!   events, scene-change submission. Must not expose wgpu, winit, Iced or
//!   any language/VM types.
//! - `renderer`: wgpu device / target / resource ownership.
//! - `host`: window (winit) and headless hosts owning the single device.
//!
//! Game logic submits scene/resource changes and consumes actions/events;
//! the renderer owns all GPU resources.

pub mod assets;
pub mod core;
pub mod games;
pub mod host;
pub mod render2d;
pub mod render3d;
pub mod renderer;
