//! GPU composition primitives (PLAN-830 T-02/T-04).
//!
//! Layering contract: this module owns the wgpu pipelines that CONSUME one
//! surface's offscreen texture inside another surface's render pass — a
//! perspective 3D portal plane (`SurfacePlanePipeline`) and later the 2D
//! portal quad compositor. It does not know Surface/Portal identities (the
//! GPU-agnostic composition graph lives in `core::composition`); callers
//! hand it validated transforms and a sampled offscreen texture.
//!
//! Color contract (T-02 frozen, formalized in T-04): child content textures
//! are `Rgba8UnormSrgb` (sRGB-encoded, PREMULTIPLIED alpha as produced by
//! iced_wgpu). Sampling yields linear premultiplied values; writing into an
//! sRGB render target re-encodes. Compositing uses the premultiplied-over
//! blend (src: One, dst: OneMinusSrcAlpha). Never sample a texture in the
//! same pass that writes it (submit-order DAG enforced by the caller).

use crate::renderer::error::RendererError;
use glam::Mat4;

fn mat4_bytes(m: &Mat4) -> [u8; 64] {
    let mut out = [0u8; 64];
    for (i, f) in m.to_cols_array().iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&f.to_ne_bytes());
    }
    out
}

/// Renders a textured plane (portal quad) as part of a 3D scene.
///
/// The plane is real scene geometry: `model` places/rotates it in world
/// space and the perspective `view_proj` transforms it, so the child
/// surface's content undergoes true perspective mapping (V-02/V-03).
pub struct SurfacePlanePipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    bind_group: Option<(wgpu::BindGroup, wgpu::TextureView)>,
    /// Real buffer-rewrite accounting for the COMPOSITION path (review r3
    /// F-830-R3-03): every set_transform writes 128B of uniforms.
    pub uploads: crate::renderer::UploadCounters,
    /// V-08 negative-control hook: when false the REAL 128B write still
    /// happens but is not recorded, so an acceptance gate that cannot detect
    /// the omission is proved false-green (review r4 F-830-R4-02).
    pub accounting: bool,
}

const PLANE_WGSL: &str = r#"
struct Uniforms {
    model: mat4x4<f32>,
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var content: texture_2d<f32>;
@group(0) @binding(2) var smp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    // Two CCW triangles covering the [-1,1] quad facing +Z; local X maps to
    // U and local -Y maps to V (V grows downward in texture space).
    var p = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    var uv = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 0.0),
    );
    var out: VsOut;
    let world = u.model * vec4<f32>(p[vi], 0.0, 1.0);
    out.pos = u.view_proj * world;
    out.uv = uv[vi];
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Premultiplied-over compositing of the child surface content.
    return textureSample(content, smp, in.uv);
}
"#;

impl SurfacePlanePipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Result<Self, RendererError> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp surface plane"),
            source: wgpu::ShaderSource::Wgsl(PLANE_WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp surface plane layout"),
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agp surface plane pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        crate::renderer::record_pipeline_creation();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp surface plane"),
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
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("agp surface plane sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp surface plane uniforms"),
            size: 128,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(SurfacePlanePipeline {
            pipeline,
            layout,
            sampler,
            uniforms,
            bind_group: None,
            uploads: crate::renderer::UploadCounters::default(),
            accounting: true,
        })
    }

    pub fn new_with(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        Self::new(device, format)
    }

    /// Bind the child content texture and upload the matrices. Must be
    /// called with the texture's OWN pass not active — callers render the
    /// child first, then draw the plane (submit-order DAG).
    pub fn set_transform(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        content: &wgpu::TextureView,
        model: Mat4,
        view_proj: Mat4,
    ) {
        let mut bytes = [0u8; 128];
        bytes[0..64].copy_from_slice(&mat4_bytes(&model));
        bytes[64..128].copy_from_slice(&mat4_bytes(&view_proj));
        queue.write_buffer(&self.uniforms, 0, &bytes);
        if self.accounting {
            self.uploads.record(bytes.len() as u64);
        }
        self.bind_group = Some((
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("agp surface plane bind group"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            self.uniforms.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(content),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            }),
            content.clone(),
        ));
    }

    /// The content view currently bound (input focus/inspection).
    pub fn bound_content(&self) -> Option<&wgpu::TextureView> {
        self.bind_group.as_ref().map(|(_, v)| v)
    }

    /// Record the plane draw into an EXISTING render pass (the 3D scene's
    /// pass). Load-op keeps prior scene content; the premultiplied-over
    /// blend composites the child content on top.
    pub fn draw_in_pass(&self, pass: &mut wgpu::RenderPass<'_>) -> bool {
        let Some((group, _)) = self.bind_group.as_ref() else {
            return false;
        };
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.draw(0..6, 0..1);
        true
    }
}

/// Opaque full-screen blit of a composite offscreen texture onto the
/// window surface (PLAN-830 window presentation; the offscreen composite
/// IS the window content — the same texture backs acceptance captures).
pub struct FullScreenBlit {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

const BLIT_WGSL: &str = r#"
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0), vec2<f32>(-1.0, 1.0), vec2<f32>(3.0, 1.0),
    );
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0), vec2<f32>(0.0, 0.0), vec2<f32>(2.0, 0.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(p[vi], 0.0, 1.0);
    out.uv = uv[vi];
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(src, smp, in.uv);
    return vec4<f32>(c.rgb, 1.0);
}
"#;

impl FullScreenBlit {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp fullscreen blit"),
            source: wgpu::ShaderSource::Wgsl(BLIT_WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp fullscreen blit layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agp fullscreen blit pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        crate::renderer::record_pipeline_creation();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp fullscreen blit"),
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
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("agp fullscreen blit sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        FullScreenBlit {
            pipeline,
            layout,
            sampler,
        }
    }

    /// Draw `source` over the whole target view of a fresh render pass in
    /// `encoder` (clear then blit). Used to present the offscreen composite
    /// onto the window surface.
    pub fn blit(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        source: &wgpu::TextureView,
    ) {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("agp fullscreen blit bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("agp composite present"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}

// ---------------------------------------------------------------------------
// 2D portal compositor (PLAN-830 T-04)
// ---------------------------------------------------------------------------

/// Composites one child surface texture into a parent 2D target at a
/// pixel-space rect, clipped by the mount's clip rect (scissor), with the
/// SAME premultiplied-over contract as the 3D plane.
///
/// The rect is given in the PARENT target's pixel space; the clip is applied
/// through `set_scissor_rect` so content never bleeds outside the declared
/// geometry (V-02 clip gate).
pub struct PortalQuadPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    /// Real buffer-rewrite accounting for the COMPOSITION path (review r3
    /// F-830-R3-03): every drawn 2D portal writes 160B of uniforms.
    pub uploads: crate::renderer::UploadCounters,
}

const PORTAL_QUAD_WGSL: &str = r#"
struct Uniforms {
    // x, y, w, h in parent pixel space; parent size in zw.
    rect: vec4<f32>,
    parent_size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var content: texture_2d<f32>;
@group(0) @binding(2) var smp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let c = corners[vi];
    let px = u.rect.xy + c * u.rect.zw;
    // Pixel space (origin top-left, y down) -> clip space.
    let ndc = vec2<f32>(
        px.x / u.parent_size.x * 2.0 - 1.0,
        1.0 - px.y / u.parent_size.y * 2.0,
    );
    var out: VsOut;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = c;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(content, smp, in.uv);
}
"#;

impl PortalQuadPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self::new_with_alpha(device, format, true)
    }

    /// `premultiplied = true` is the production contract. `false` builds the
    /// deliberately WRONG (replace/opaque) pipeline used only by the
    /// negative-control gate to prove the alpha gate observes a real blend
    /// change (review r1 F09 — a constant expectation is not a fault).
    pub fn new_with_alpha(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        premultiplied: bool,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp portal quad"),
            source: wgpu::ShaderSource::Wgsl(PORTAL_QUAD_WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp portal quad layout"),
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agp portal quad pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        crate::renderer::record_pipeline_creation();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp portal quad"),
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
                    format,
                    blend: if premultiplied {
                        Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        })
                    } else {
                        // Opaque replace — the WRONG contract the alpha gate
                        // must catch when injected.
                        None
                    },
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("agp portal quad sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp portal quad uniforms"),
            size: 160,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        PortalQuadPipeline {
            pipeline,
            layout,
            sampler,
            uniforms,
            uploads: crate::renderer::UploadCounters::default(),
        }
    }

    /// Draw `content` into the parent target `pass` at `rect` (parent px),
    /// clipped to `clip` (parent px; `None` deliberately skips the scissor —
    /// the negative-control fault). Both rects are in the parent's pixel
    /// space; the caller guarantees `pass` writes the parent target.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        content: &wgpu::TextureView,
        rect: [f32; 4],
        clip: Option<[f32; 4]>,
        parent_size: [f32; 2],
    ) {
        // Scissor is integral; clamp to the parent target to stay valid.
        if let Some(clip) = clip {
            let x0 = clip[0].max(0.0).min(parent_size[0]) as u32;
            let y0 = clip[1].max(0.0).min(parent_size[1]) as u32;
            let x1 = (clip[0] + clip[2]).max(0.0).min(parent_size[0]) as u32;
            let y1 = (clip[1] + clip[3]).max(0.0).min(parent_size[1]) as u32;
            if x1 <= x0 || y1 <= y0 {
                return; // fully clipped
            }
            pass.set_scissor_rect(x0, y0, x1 - x0, y1 - y0);
        }
        let mut bytes = [0u8; 160];
        for (i, f) in [rect[0], rect[1], rect[2], rect[3]].iter().enumerate() {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&f.to_ne_bytes());
        }
        for (i, f) in [parent_size[0], parent_size[1]].iter().enumerate() {
            bytes[16 + i * 4..16 + i * 4 + 4].copy_from_slice(&f.to_ne_bytes());
        }
        queue.write_buffer(&self.uniforms, 0, &bytes);
        self.uploads.record(bytes.len() as u64);
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("agp portal quad bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        self.uniforms.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(content),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..6, 0..1);
        // Restore the whole-target scissor so later draws are unclipped.
        if clip.is_some() {
            pass.set_scissor_rect(0, 0, parent_size[0] as u32, parent_size[1] as u32);
        }
    }
}
