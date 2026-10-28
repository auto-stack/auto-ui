//! Hosts: window (winit) and headless owners of the single device.

pub mod cases;
pub mod composition_input;
pub mod gates;
pub mod iced_spike;
pub mod iced_surface;
pub mod iced_surface_probe;
pub mod report;
pub mod surface_runtime;

pub use report::{
    FailureInjection, GpuFoundationReport, Keyframe, ProbeReport, SharedRuntimeReport,
    SurfaceReport,
};
