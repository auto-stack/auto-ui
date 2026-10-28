//! Minimal 2D colored-quad pipeline. Enough for gpu-foundation frames and a
//! direct fit for the frozen Breakout sample (flat-colored rectangles +
//! circle; the circle is a textured/segmented quad variant for later tasks).

use crate::core::scene::Color;
use crate::renderer::device::GpuDevice;

/// Quad or circle shape (the frozen Breakout ball is a circle mesh).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    #[default]
    Rect,
    Circle,
}

/// One colored axis-aligned quad (or circle) in world space.
/// `center`/`size` are world units; `z` in 0..1 for draw order.
#[derive(Debug, Clone, Copy)]
pub struct DrawCmd {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub z: f32,
    pub color: Color,
    pub shape: Shape,
}

const MAX_QUADS: usize = 4096;

const VERT: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) shape: f32,
};

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

struct Quad {
    center_size: vec4<f32>, // cx, cy, w, h
    color: vec4<f32>,
    misc: vec4<f32>,        // x: 0 = rect, 1 = circle
};

@group(0) @binding(1)
var<storage, read> quads: array<Quad>;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VertexOutput {
    let quad = quads[ii];
    let corner = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    ) * 0.5; // (0,0) (1,0) (0,1) (1,1) across the strip
    let local = (corner - 0.5) * quad.center_size.zw;
    let world = vec3<f32>(quad.center_size.xy + local, 0.0);
    var out: VertexOutput;
    out.position = camera * vec4<f32>(world, 1.0);
    out.color = quad.color;
    out.uv = corner;
    out.shape = quad.misc.x;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.shape > 0.5) {
        // Circle: discard outside the inscribed circle (center 0.5, r 0.5).
        if (distance(in.uv, vec2<f32>(0.5, 0.5)) > 0.5) {
            discard;
        }
    }
    return in.color;
}
"#;

pub struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
    camera_buf: wgpu::Buffer,
    quad_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// Buffer-rewrite accounting (F08), public for honest measurement.
    pub uploads: crate::renderer::UploadCounters,
    /// Host device that created this pipeline (when known). The composition
    /// runtime refuses content built on a FOREIGN device (review r1 F09).
    pub device_ordinal: Option<u64>,
}

impl QuadPipeline {
    pub fn owned_buffer_bytes(&self) -> u64 {
        self.camera_buf.size() + self.quad_buf.size()
    }
    /// Compile the pipeline. `Err(BadShader)` carries the validation message.
    pub fn new(
        gpu: &GpuDevice,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, crate::renderer::error::RendererError> {
        let mut out = Self::new_with(gpu.device(), gpu.queue(), target_format)?;
        out.device_ordinal = Some(gpu.token().ordinal);
        Ok(out)
    }

    /// Raw-device variant: the caller owns device/queue lifetime (the Iced
    /// seam hands us ITS device — this is how the graphics view shares the
    /// same device/queue instance).
    pub fn new_with(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, crate::renderer::error::RendererError> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp quad"),
            source: wgpu::ShaderSource::Wgsl(VERT.into()),
        });

        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let quad_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp quads"),
            size: (MAX_QUADS * 48) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp quad bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("agp quad bg"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: quad_buf.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agp quad layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        // Shader validation happens at pipeline creation; capture it via an
        // error scope so BadShader is classified, not panicked.
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        crate::renderer::record_pipeline_creation();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp quad pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let maybe_err = pop_error_scope_blocking(device);
        if let Some(msg) = maybe_err {
            return Err(crate::renderer::error::RendererError::BadShader {
                stage: "quad",
                message: msg,
            });
        }

        Ok(QuadPipeline {
            pipeline,
            camera_buf,
            quad_buf,
            bind_group,
            uploads: crate::renderer::UploadCounters::default(),
            device_ordinal: None,
        })
    }

    /// Compile-check helper for the failure-injection path: same shader
    /// source as `new` but with a caller-provided (possibly broken) module.
    pub fn validate_shader_source(
        gpu: &GpuDevice,
        source: &str,
        stage: &'static str,
    ) -> Result<(), crate::renderer::error::RendererError> {
        let device = gpu.device();
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp validate"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agp validate layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        crate::renderer::record_pipeline_creation();
        let _ = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp validate pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        match pop_error_scope_blocking(device) {
            None => Ok(()),
            Some(message) => {
                Err(crate::renderer::error::RendererError::BadShader { stage, message })
            }
        }
    }

    /// Orthographic 2D camera mapping x∈[left,right]→[-1,1] and
    /// y∈[top,bottom]→[+1,-1] (screen-style Y down), z passthrough.
    /// Written out explicitly to stay free of glam projection API drift.
    pub fn set_ortho_camera(&self, gpu: &GpuDevice, left: f32, right: f32, bottom: f32, top: f32) {
        self.set_ortho_camera_with(gpu.queue(), left, right, bottom, top);
    }

    /// Raw-queue variant (Iced seam).
    pub fn set_ortho_camera_with(
        &self,
        queue: &wgpu::Queue,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
    ) {
        let w = right - left;
        let h = bottom - top;
        let arr: [f32; 16] = [
            2.0 / w,
            0.0,
            0.0,
            0.0,
            0.0,
            -2.0 / h,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            -(right + left) / w,
            (bottom + top) / h,
            0.0,
            1.0,
        ];
        let bytes: [u8; 64] = unsafe { std::mem::transmute(arr) };
        queue.write_buffer(&self.camera_buf, 0, &bytes);
        self.uploads.record(64);
    }

    /// Like `render` but loads the existing attachment content (for overlay
    /// passes on top of an already-rendered frame).
    pub fn render_over(
        &self,
        gpu: &GpuDevice,
        view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        draw: &[DrawCmd],
    ) {
        self.render_internal(gpu, view, format, None, draw);
    }

    /// Raw-device overlay render (Iced seam).
    pub fn render_over_with(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        draw: &[DrawCmd],
    ) {
        self.render_internal_raw(device, queue, view, None, draw);
    }

    /// Raw-device clear+render (Iced seam).
    pub fn render_with(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        clear: Option<Color>,
        draw: &[DrawCmd],
    ) {
        self.render_internal_raw(device, queue, view, clear, draw);
    }

    /// Encode a clear + quad pass into `view` and submit. This is the only
    /// submission point for 2D frames (single queue, ordered).
    pub fn render(
        &self,
        gpu: &GpuDevice,
        view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        clear: Color,
        draw: &[DrawCmd],
    ) {
        self.render_internal(gpu, view, format, Some(clear), draw);
    }

    fn render_internal(
        &self,
        gpu: &GpuDevice,
        view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        clear: Option<Color>,
        draw: &[DrawCmd],
    ) {
        let _ = format;
        self.render_internal_raw(gpu.device(), gpu.queue(), view, clear, draw);
    }

    fn render_internal_raw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        clear: Option<Color>,
        draw: &[DrawCmd],
    ) {
        let _ = device;
        debug_assert!(draw.len() <= MAX_QUADS);
        if !draw.is_empty() {
            let mut bytes: Vec<u8> = Vec::with_capacity(draw.len() * 48);
            for cmd in draw {
                for f in [cmd.center[0], cmd.center[1], cmd.size[0], cmd.size[1]] {
                    bytes.extend_from_slice(&f.to_ne_bytes());
                }
                for f in [cmd.color.r, cmd.color.g, cmd.color.b, cmd.color.a] {
                    bytes.extend_from_slice(&f.to_ne_bytes());
                }
                let shape = match cmd.shape {
                    Shape::Rect => 0.0f32,
                    Shape::Circle => 1.0,
                };
                for f in [shape, 0.0, 0.0, 0.0] {
                    bytes.extend_from_slice(&f.to_ne_bytes());
                }
            }
            queue.write_buffer(&self.quad_buf, 0, &bytes);
            self.uploads.record(bytes.len() as u64);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("agp frame"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("agp quad pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear {
                            Some(c) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: c.r as f64,
                                g: c.g as f64,
                                b: c.b as f64,
                                a: c.a as f64,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if !draw.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.draw(0..4, 0..draw.len() as u32);
            }
        }
        queue.submit([encoder.finish()]);
    }
}

pub(crate) fn pop_error_scope_blocking(device: &wgpu::Device) -> Option<String> {
    use std::future::Future;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    struct Noop;
    impl Wake for Noop {
        fn wake(self: Arc<Self>) {}
    }
    let fut = device.pop_error_scope();
    let waker = Waker::from(Arc::new(Noop));
    let mut cx = Context::from_waker(&waker);
    let mut fut = Box::pin(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(err) => {
                return err.map(|e| format!("{e}"));
            }
            Poll::Pending => std::hint::spin_loop(),
        }
    }
}
