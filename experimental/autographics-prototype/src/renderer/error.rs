//! Classified renderer failures.
//!
//! Every fallible renderer operation reports one of these; the probe report
//! maps them 1:1 into its failure-injection section.

use crate::core::id::ObjectId;
use std::path::PathBuf;

#[derive(Debug)]
pub enum RendererError {
    /// No adapter satisfied the request.
    AdapterUnavailable,
    /// Device was lost (usually GPU reset / driver crash).
    DeviceLost(String),
    /// Swapchain surface failure.
    SurfaceLost(String),
    /// Target configured with a zero dimension (minimized window or invalid
    /// offscreen size). The previous valid configuration is kept.
    ZeroSizeTarget { width: u32, height: u32 },
    /// Requested target exceeds the device's texture limits. Checked BEFORE
    /// `create_texture` so the failure is a classified error, never a wgpu
    /// validation panic and never a half-committed graph (review r1 F03).
    TextureTooLarge { width: u32, height: u32, limit: u32 },
    /// Referenced file does not exist.
    MissingFile(PathBuf),
    /// Font bytes could not be parsed/rasterized.
    BadFont(String),
    /// Shader source failed to compile/validate.
    BadShader {
        stage: &'static str,
        message: String,
    },
    /// An identity was stale (freed/reused) or from another session.
    StaleIdentity { id: ObjectId, detail: String },
    /// Bounded cache is full and the entry is not reusable.
    CacheFull { capacity: usize },
    /// Late change aimed at a destroyed resource.
    ResourceReleased { key: String },
    /// Report/IO failure while writing probe artifacts.
    ReportIo(String),
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RendererError::AdapterUnavailable => write!(f, "no suitable GPU adapter"),
            RendererError::DeviceLost(m) => write!(f, "device lost: {m}"),
            RendererError::SurfaceLost(m) => write!(f, "surface lost: {m}"),
            RendererError::ZeroSizeTarget { width, height } => {
                write!(f, "zero-size target {width}x{height}")
            }
            RendererError::TextureTooLarge {
                width,
                height,
                limit,
            } => write!(f, "texture {width}x{height} exceeds device limit {limit}"),
            RendererError::MissingFile(p) => write!(f, "missing file {}", p.display()),
            RendererError::BadFont(m) => write!(f, "bad font: {m}"),
            RendererError::BadShader { stage, message } => {
                write!(f, "bad shader ({stage}): {message}")
            }
            RendererError::StaleIdentity { id, detail } => {
                write!(f, "stale identity {} ({detail})", id.token())
            }
            RendererError::CacheFull { capacity } => {
                write!(f, "resource cache full (cap {capacity})")
            }
            RendererError::ResourceReleased { key } => {
                write!(f, "resource already released: {key}")
            }
            RendererError::ReportIo(m) => write!(f, "report io: {m}"),
        }
    }
}

impl std::error::Error for RendererError {}

/// Stable machine-readable classification name for reports.
impl RendererError {
    pub fn classify(&self) -> &'static str {
        match self {
            RendererError::AdapterUnavailable => "adapter_unavailable",
            RendererError::DeviceLost(_) => "device_lost",
            RendererError::SurfaceLost(_) => "surface_lost",
            RendererError::ZeroSizeTarget { .. } => "zero_size_target",
            RendererError::TextureTooLarge { .. } => "texture_too_large",
            RendererError::MissingFile(_) => "missing_file",
            RendererError::BadFont(_) => "bad_font",
            RendererError::BadShader { .. } => "bad_shader",
            RendererError::StaleIdentity { .. } => "stale_identity",
            RendererError::CacheFull { .. } => "cache_full",
            RendererError::ResourceReleased { .. } => "resource_released",
            RendererError::ReportIo(_) => "report_io",
        }
    }
}
