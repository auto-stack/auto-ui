//! Bounded editor-only depth-tested line pass; never contributes shadow casters.
use crate::renderer::{RendererError, UploadCounters, UploadStats};
use glam::Mat4;

pub const MAX_EDITOR_GRID_VERTICES: usize = 256;
const STRIDE: u64 = 28;
const SHADER: &str = r#"
@group(0) @binding(0) var<uniform> vp: mat4x4<f32>;
struct Output { @builtin(position) position: vec4<f32>, @location(0) color: vec4<f32> };
@vertex fn vs_main(@location(0) p: vec3<f32>, @location(1) color: vec4<f32>) -> Output {
    var out: Output; out.position = vp * vec4<f32>(p, 1.0); out.color = color; return out;
}
@fragment fn fs_main(in: Output) -> @location(0) vec4<f32> { return in.color; }
"#;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditorGridVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

pub(crate) struct EditorGrid {
    owner: std::sync::Arc<()>,
    pipeline: wgpu::RenderPipeline,
    camera: wgpu::Buffer,
    vertices: wgpu::Buffer,
    bind: wgpu::BindGroup,
    data: Vec<EditorGridVertex>,
    uploaded: Option<Vec<EditorGridVertex>>,
    last_vp: Option<[f32; 16]>,
    pub uploads: UploadCounters,
}
impl EditorGrid {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Result<Self, RendererError> {
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("editor grid"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let camera = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("editor grid camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("editor grid bounded vertices"),
            size: STRIDE * MAX_EDITOR_GRID_VERTICES as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("editor grid layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("editor grid binding"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("editor grid pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let attrs = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 12,
                shader_location: 1,
            },
        ];
        crate::renderer::record_pipeline_creation();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("editor grid depth pass"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: STRIDE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: -1,
                    slope_scale: -1.0,
                    clamp: 0.0,
                },
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        if let Some(message) = crate::renderer::pipeline::pop_error_scope_blocking(device) {
            return Err(RendererError::BadShader {
                stage: "editor grid",
                message,
            });
        }
        Ok(Self {
            owner: std::sync::Arc::new(()),
            pipeline,
            camera,
            vertices,
            bind,
            data: Vec::new(),
            uploaded: None,
            last_vp: None,
            uploads: UploadCounters::default(),
        })
    }
    pub fn set(&mut self, vertices: &[EditorGridVertex]) -> Result<bool, String> {
        if vertices.len() > MAX_EDITOR_GRID_VERTICES
            || vertices.len() % 2 != 0
            || vertices.iter().any(|v| {
                v.position
                    .iter()
                    .chain(v.color.iter())
                    .any(|x| !x.is_finite())
            })
        {
            return Err("invalid or over-capacity editor grid".into());
        }
        if self.data == vertices {
            return Ok(false);
        }
        self.data.clear();
        self.data.extend_from_slice(vertices);
        Ok(true)
    }
    pub fn owned_bytes(&self) -> u64 {
        self.camera.size() + self.vertices.size()
    }
    pub fn owner_probe(&self) -> std::sync::Weak<()> {
        std::sync::Arc::downgrade(&self.owner)
    }
    pub fn cpu_capacity_bytes(&self) -> usize {
        (self.data.capacity() + self.uploaded.as_ref().map_or(0, Vec::capacity))
            * std::mem::size_of::<EditorGridVertex>()
    }
    pub fn stats(&self) -> UploadStats {
        self.uploads.snapshot()
    }
    pub fn encode(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        vp: &Mat4,
    ) {
        if self.data.is_empty() {
            return;
        }
        let matrix = vp.to_cols_array();
        if self.last_vp != Some(matrix) {
            let bytes: Vec<u8> = matrix.iter().flat_map(|v| v.to_ne_bytes()).collect();
            queue.write_buffer(&self.camera, 0, &bytes);
            self.uploads.record(bytes.len() as u64);
            self.last_vp = Some(matrix);
        }
        if self.uploaded.as_ref() != Some(&self.data) {
            let bytes: Vec<u8> = self
                .data
                .iter()
                .flat_map(|v| {
                    v.position
                        .iter()
                        .chain(v.color.iter())
                        .flat_map(|f| f.to_ne_bytes())
                })
                .collect();
            queue.write_buffer(&self.vertices, 0, &bytes);
            self.uploads.record(bytes.len() as u64);
            self.uploaded = Some(self.data.clone());
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("editor grid depth pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.draw(0..self.data.len() as u32, 0..1);
    }
}
