//! Render targets: swapchain-backed window target and offscreen texture
//! target. Zero-size/minimize handling: a zero-size configuration is
//! rejected (`ZeroSizeTarget`) and the previous valid configuration (or
//! presentation suspension) is kept — never clobbered.

use crate::renderer::device::GpuDevice;
use crate::renderer::error::RendererError;

/// Result of one frame submission against a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderOutcome {
    Presented,
    /// Offscreen frame submitted (nothing to present).
    Submitted,
    /// Target is minimized/zero-size; nothing was drawn this frame. This is
    /// NOT an error path — the last presented frame stays visible.
    SuspendedZeroSize,
}

/// Offscreen RGBA8 texture target with readback support.
pub struct OffscreenTarget {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampled: bool,
    /// Process-wide creation ordinal: lets acceptance gates count REAL target
    /// creations / warm-object rebuilds instead of inferring them from loop
    /// structure (review r1 F09).
    pub build_seq: u64,
}

fn next_build_seq() -> u64 {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl OffscreenTarget {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn new(gpu: &GpuDevice, width: u32, height: u32) -> Result<Self, RendererError> {
        Self::new_with_usages(
            gpu,
            width,
            height,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            false,
        )
    }

    /// PLAN-830 usage contract (§3): an offscreen target whose content is
    /// CONSUMED as a texture by other passes carries `TEXTURE_BINDING` in
    /// addition to the legacy render/readback usages. Legacy `new` callers
    /// keep the old (non-sampled) usage — the two contracts stay distinct
    /// and a non-sampled target must never be claimed as samplable.
    pub fn new_sampled(gpu: &GpuDevice, width: u32, height: u32) -> Result<Self, RendererError> {
        Self::new_with_usages(
            gpu,
            width,
            height,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            true,
        )
    }

    fn new_with_usages(
        gpu: &GpuDevice,
        width: u32,
        height: u32,
        usage: wgpu::TextureUsages,
        sampled: bool,
    ) -> Result<Self, RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::ZeroSizeTarget { width, height });
        }
        // The device limit is checked BEFORE `create_texture`: wgpu treats an
        // oversized texture as a validation error (panic), which would leave
        // the caller half-committed. A classified Err keeps the last good
        // state and lets the caller decide.
        let limit = gpu.device().limits().max_texture_dimension_2d;
        if width > limit || height > limit {
            return Err(RendererError::TextureTooLarge {
                width,
                height,
                limit,
            });
        }
        let texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("agp offscreen"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Ok(OffscreenTarget {
            width,
            height,
            format: Self::FORMAT,
            texture,
            view,
            sampled,
            build_seq: next_build_seq(),
        })
    }

    /// True when this target was created under the sampled usage contract
    /// (PLAN-830) and may be bound as a `TEXTURE_BINDING` input.
    pub fn is_sampled(&self) -> bool {
        self.sampled
    }

    /// Resize: only the texture object is recreated; pipeline and device are
    /// untouched (PLAN-829 §5.3: local change, no global rebuild). The
    /// usage contract (sampled or not) is PRESERVED across the resize — a
    /// sampled target must stay samplable for its consumers.
    pub fn resize(
        &mut self,
        gpu: &GpuDevice,
        width: u32,
        height: u32,
    ) -> Result<(), RendererError> {
        if width == 0 || height == 0 {
            // Reject; keep the last valid target.
            return Err(RendererError::ZeroSizeTarget { width, height });
        }
        let sampled = self.sampled;
        let usage = if sampled {
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING
        } else {
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
        };
        *self = Self::new_with_usages(gpu, width, height, usage, sampled)?;
        Ok(())
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

/// Swapchain-backed window target. Presentation suspends while minimized.
pub struct WindowTarget {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pub width: u32,
    pub height: u32,
    minimized: bool,
}

impl WindowTarget {
    pub fn new(
        gpu: &GpuDevice,
        window: std::sync::Arc<winit::window::Window>,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::ZeroSizeTarget { width, height });
        }
        let surface = gpu
            .instance()
            .create_surface(window)
            .map_err(|e| RendererError::SurfaceLost(format!("create_surface: {e}")))?;
        let caps = surface.get_capabilities(gpu.adapter());
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(gpu.device(), &config);
        Ok(WindowTarget {
            surface,
            config,
            width,
            height,
            minimized: false,
        })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Handle a window resize event. Zero-size (minimize) suspends and keeps
    /// the last valid config; restore reconfigures.
    pub fn handle_resize(
        &mut self,
        gpu: &GpuDevice,
        width: u32,
        height: u32,
    ) -> Result<RenderOutcome, RendererError> {
        if width == 0 || height == 0 {
            self.minimized = true;
            return Ok(RenderOutcome::SuspendedZeroSize);
        }
        self.minimized = false;
        self.width = width;
        self.height = height;
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(gpu.device(), &self.config);
        Ok(RenderOutcome::Presented)
    }

    pub fn is_minimized(&self) -> bool {
        self.minimized
    }

    /// Acquire the current frame and let the caller encode ALL draws for it
    /// (geometry + text + 3D) before presenting. Returns `SuspendedZeroSize`
    /// while minimized without touching the surface.
    pub fn present_frame(
        &mut self,
        gpu: &GpuDevice,
        encode: impl FnOnce(&wgpu::TextureView),
    ) -> Result<RenderOutcome, RendererError> {
        if self.minimized || self.config.width == 0 || self.config.height == 0 {
            return Ok(RenderOutcome::SuspendedZeroSize);
        }
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Recoverable: reconfigure from the last valid config.
                self.surface.configure(gpu.device(), &self.config);
                self.surface
                    .get_current_texture()
                    .map_err(|e| RendererError::SurfaceLost(format!("{e:?}")))?
            }
            Err(wgpu::SurfaceError::Timeout) => {
                return Ok(RenderOutcome::SuspendedZeroSize);
            }
            Err(e) => return Err(RendererError::SurfaceLost(format!("{e:?}"))),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        encode(&view);
        frame.present();
        Ok(RenderOutcome::Presented)
    }
}
