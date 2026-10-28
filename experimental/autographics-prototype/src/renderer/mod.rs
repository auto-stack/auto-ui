//! wgpu renderer: device/target/resource ownership.
//!
//! Failure policy (PLAN-829 §5.3): a failed operation returns a classified
//! `RendererError` and MUST NOT clobber the last-good scene/target state.
//! Frame-level failures are reported to the caller; the previous successful
//! frame stays presentable.

pub mod capture;

/// Per-pipeline buffer-rewrite accounting (review r1 F08): what was
/// REWRITTEN into existing buffers (bytes), as opposed to resource
/// CREATIONS. Instrumented at the actual write paths; interior mutability
/// because the render entry points take `&self`.
#[derive(Debug, Default)]
pub struct UploadCounters {
    rewrites: std::sync::atomic::AtomicU64,
    bytes: std::sync::atomic::AtomicU64,
}

/// Immutable snapshot of the counters (for deltas and reports).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct UploadStats {
    pub buffer_rewrites: u64,
    pub bytes_rewritten: u64,
}

impl UploadCounters {
    pub fn record(&self, bytes: u64) {
        use std::sync::atomic::Ordering::Relaxed;
        self.rewrites.fetch_add(1, Relaxed);
        self.bytes.fetch_add(bytes, Relaxed);
    }

    pub fn snapshot(&self) -> UploadStats {
        use std::sync::atomic::Ordering::Relaxed;
        UploadStats {
            buffer_rewrites: self.rewrites.load(Relaxed),
            bytes_rewritten: self.bytes.load(Relaxed),
        }
    }
}

impl UploadStats {
    pub fn delta_since(&self, other: &UploadStats) -> UploadStats {
        UploadStats {
            buffer_rewrites: self.buffer_rewrites.saturating_sub(other.buffer_rewrites),
            bytes_rewritten: self.bytes_rewritten.saturating_sub(other.bytes_rewritten),
        }
    }

    /// Add another pipeline's counters into this aggregate.
    pub fn merge(&mut self, other: &UploadStats) {
        self.buffer_rewrites = self.buffer_rewrites.saturating_add(other.buffer_rewrites);
        self.bytes_rewritten = self.bytes_rewritten.saturating_add(other.bytes_rewritten);
    }
}

pub mod composition;
pub mod device;
pub mod error;
pub mod pipeline;
pub mod resources;
pub mod target;
pub mod witness;

pub use device::{AdapterInfoReport, DeviceToken, GpuDevice};
pub use error::RendererError;
pub use pipeline::{DrawCmd, QuadPipeline, Shape};
pub use resources::{CacheKey, CacheStats, ResourceCache};
pub use target::{OffscreenTarget, RenderOutcome, WindowTarget};
pub use witness::{DropWitness, Tracked};

// PLAN-833: measured at the actual GPU pipeline construction call sites.
static PIPELINE_CREATIONS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub(crate) fn record_pipeline_creation() {
    PIPELINE_CREATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
pub fn pipeline_creations() -> u64 {
    PIPELINE_CREATIONS.load(std::sync::atomic::Ordering::Relaxed)
}
