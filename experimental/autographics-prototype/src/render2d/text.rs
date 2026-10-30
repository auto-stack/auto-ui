//! Glyph-atlas text rendering (T-02 decision: fontdue + custom atlas —
//! glyphon has no wgpu-27 line). One atlas texture per (font, px size);
//! layout is pure CPU (fontdue metrics), instances go through a storage
//! buffer like the quad pipeline.

use crate::core::scene::Color;
use crate::renderer::device::GpuDevice;
use crate::renderer::error::RendererError;

const MAX_GLYPHS: usize = 2048;

const TEXT_VERT: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

struct Glyph {
    dst: vec4<f32>,   // x, y, w, h (world units, top-left origin)
    uv_rect: vec4<f32>, // u0, v0, u1, v1
    color: vec4<f32>,
};

@group(0) @binding(1)
var<storage, read> glyphs: array<Glyph>;

@group(0) @binding(2)
var atlas: texture_2d<f32>;
@group(0) @binding(3)
var atlas_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VertexOutput {
    let glyph = glyphs[ii];
    let corner = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    ) * 0.5; // (0,0) (1,0) (0,1) (1,1)
    let pos = glyph.dst.xy + corner * glyph.dst.zw;
    var out: VertexOutput;
    out.position = camera * vec4<f32>(pos, 0.0, 1.0);
    out.uv = mix(glyph.uv_rect.xy, glyph.uv_rect.zw, corner);
    out.color = glyph.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let a = textureSample(atlas, atlas_sampler, in.uv).r;
    if (a < 0.01) {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * a);
}
"#;

/// One laid-out glyph quad, ready for the GPU instance buffer.
#[derive(Debug, Clone, Copy)]
pub struct GlyphQuad {
    pub dst: [f32; 4],
    pub uv_rect: [f32; 4],
    pub color: [f32; 4],
}

struct GlyphInfo {
    uv_rect: [f32; 4],
    advance: f32,
    /// fontdue ymin (bitmap bottom offset from baseline, y-up).
    bearing_y: f32,
    /// fontdue xmin (bitmap left offset from the pen).
    min_x: f32,
    width: f32,
    height: f32,
}

pub struct TextRenderer {
    pub texture_upload_bytes: u64,
    pub uploads: crate::renderer::UploadCounters,
    pub device_ordinal: u64,
    font_size: f32,
    ascent: f32,
    glyphs: std::collections::HashMap<char, GlyphInfo>,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    camera_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    #[allow(dead_code)]
    atlas: wgpu::Texture,
}

impl TextRenderer {
    /// Rasterize `chars` at `font_size_px` into a single atlas and build the
    /// text pipeline for `target_format`.
    pub fn new(
        gpu: &GpuDevice,
        target_format: wgpu::TextureFormat,
        font_bytes: &[u8],
        font_size_px: f32,
        chars: &str,
    ) -> Result<Self, RendererError> {
        Self::new_inner(gpu, target_format, font_bytes, font_size_px, chars, false)
    }

    pub fn new_checked(
        gpu: &GpuDevice,
        target_format: wgpu::TextureFormat,
        font_bytes: &[u8],
        font_size_px: f32,
        chars: &str,
    ) -> Result<Self, RendererError> {
        Self::new_inner(gpu, target_format, font_bytes, font_size_px, chars, true)
    }

    fn new_inner(
        gpu: &GpuDevice,
        target_format: wgpu::TextureFormat,
        font_bytes: &[u8],
        font_size_px: f32,
        chars: &str,
        strict_glyphs: bool,
    ) -> Result<Self, RendererError> {
        let font = fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default())
            .map_err(|e| RendererError::BadFont(format!("parse: {e:?}")))?;

        // Shelf-pack glyphs into an atlas.
        let px = font_size_px.round().max(8.0) as usize;
        let ascent = font
            .horizontal_line_metrics(px as f32)
            .map(|m| m.ascent)
            .unwrap_or(px as f32);
        let atlas_w = 1024usize;
        let mut atlas_h = 128usize;
        let mut glyphs = std::collections::HashMap::new();
        let mut shelf_x = 0usize;
        let mut shelf_y = 0usize;
        let mut shelf_h = 0usize;
        // First pass: metrics + packing plan.
        struct Plan {
            c: char,
            w: usize,
            h: usize,
            x: usize,
            y: usize,
            advance: f32,
            bearing_y: f32,
            minx: f32,
        }
        let mut plans = Vec::new();
        for c in chars.chars() {
            if glyphs.contains_key(&c) {
                continue;
            }
            if strict_glyphs && font.lookup_glyph_index(c) == 0 {
                return Err(RendererError::BadFont(format!(
                    "missing glyph U+{:04X}",
                    c as u32
                )));
            }
            let (metrics, _bitmap) = font.rasterize(c, px as f32);
            let w = metrics.width.max(1);
            let h = metrics.height.max(1);
            if shelf_x + w > atlas_w {
                shelf_x = 0;
                shelf_y += shelf_h + 1;
                shelf_h = 0;
            }
            shelf_h = shelf_h.max(h);
            if shelf_y + shelf_h > atlas_h {
                atlas_h *= 2;
            }
            plans.push(Plan {
                c,
                w,
                h,
                x: shelf_x,
                y: shelf_y,
                advance: metrics.advance_width,
                bearing_y: metrics.ymin as f32,
                minx: metrics.xmin as f32,
            });
            shelf_x += w + 1;
        }
        let atlas_h = atlas_h.next_power_of_two();
        if atlas_h > 1024
            || atlas_w
                .checked_mul(atlas_h)
                .is_none_or(|n| n > 4 * 1024 * 1024)
        {
            return Err(RendererError::BadFont("atlas capacity".into()));
        }

        // Rasterize into CPU atlas (single-channel), then upload as R8.
        let mut cpu = vec![0u8; atlas_w * atlas_h];
        for plan in &plans {
            let (_m, bitmap) = font.rasterize(plan.c, px as f32);
            for row in 0..plan.h {
                for col in 0..plan.w {
                    // Whitespace has an empty bitmap; missing bytes are 0.
                    let src = bitmap.get(row * plan.w + col).copied().unwrap_or(0);
                    cpu[(plan.y + row) * atlas_w + (plan.x + col)] = src;
                }
            }
            let u0 = plan.x as f32 / atlas_w as f32;
            let v0 = plan.y as f32 / atlas_h as f32;
            let u1 = (plan.x + plan.w) as f32 / atlas_w as f32;
            let v1 = (plan.y + plan.h) as f32 / atlas_h as f32;
            glyphs.insert(
                plan.c,
                GlyphInfo {
                    uv_rect: [u0, v0, u1, v1],
                    advance: plan.advance,
                    bearing_y: plan.bearing_y,
                    min_x: plan.minx,
                    width: plan.w as f32,
                    height: plan.h as f32,
                },
            );
        }

        let device = gpu.device();
        let atlas = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("agp text atlas"),
            size: wgpu::Extent3d {
                width: atlas_w as u32,
                height: atlas_h as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &cpu,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_w as u32),
                rows_per_image: Some(atlas_h as u32),
            },
            wgpu::Extent3d {
                width: atlas_w as u32,
                height: atlas_h as u32,
                depth_or_array_layers: 1,
            },
        );
        let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("agp text sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp text camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp text instances"),
            size: (MAX_GLYPHS * 48) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp text bgl"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("agp text bg"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp text"),
            source: wgpu::ShaderSource::Wgsl(TEXT_VERT.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agp text layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        crate::renderer::record_pipeline_creation();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp text pipeline"),
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
        if let Some(msg) = crate::renderer::pipeline::pop_error_scope_blocking(device) {
            return Err(RendererError::BadShader {
                stage: "text",
                message: msg,
            });
        }

        Ok(TextRenderer {
            texture_upload_bytes: cpu.len() as u64,
            uploads: Default::default(),
            device_ordinal: gpu.token().ordinal,
            font_size: px as f32,
            ascent,
            glyphs,
            pipeline,
            bind_group,
            camera_buf,
            instance_buf,
            atlas,
        })
    }

    pub fn owned_buffer_bytes(&self) -> u64 {
        self.camera_buf.size() + self.instance_buf.size()
    }
    pub fn owned_atlas_bytes(&self) -> u64 {
        let size = self.atlas.size();
        u64::from(size.width) * u64::from(size.height)
    }
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Same y-down ortho mapping as the quad pipeline.
    pub fn set_ortho_camera(&self, gpu: &GpuDevice, left: f32, right: f32, bottom: f32, top: f32) {
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
        gpu.queue().write_buffer(&self.camera_buf, 0, &bytes);
        self.uploads.record(bytes.len() as u64);
    }

    /// Advance width of `text` at this font size (for multi-color layout:
    /// e.g. "Score: " in one color, the number in another).
    pub fn measure(&self, text: &str) -> f32 {
        let mut w = 0.0f32;
        for c in text.chars() {
            match self.glyphs.get(&c) {
                Some(i) => w += i.advance,
                None => w += self.font_size * 0.5,
            }
        }
        w
    }

    pub fn try_measure(&self, text: &str) -> Result<f32, String> {
        let mut width = 0.0;
        for c in text.chars() {
            let info = self
                .glyphs
                .get(&c)
                .ok_or_else(|| format!("missing glyph U+{:04X}", c as u32))?;
            width += info.advance;
        }
        Ok(width)
    }

    pub fn try_layout(
        &self,
        text: &str,
        x: f32,
        y: f32,
        color: Color,
    ) -> Result<Vec<GlyphQuad>, String> {
        for c in text.chars() {
            if !self.glyphs.contains_key(&c) {
                return Err(format!("missing glyph U+{:04X}", c as u32));
            }
        }
        Ok(self.layout(text, x, y, color))
    }

    /// CPU layout: text with pen at (x, y) = TOP-left of the line, in world
    /// units matching the camera. Unknown chars advance by space width.
    pub fn layout(&self, text: &str, x: f32, y: f32, color: Color) -> Vec<GlyphQuad> {
        let mut out = Vec::with_capacity(text.len());
        let mut pen_x = x;
        for c in text.chars() {
            let info = match self.glyphs.get(&c) {
                Some(i) => i,
                None => {
                    pen_x += self.font_size * 0.5;
                    continue;
                }
            };
            // Screen-style y-down layout: baseline sits `ascent` below the
            // line top; fontdue metrics are y-up relative to the baseline
            // (ymin = bitmap bottom offset), so bitmap top = ascent - ymax.
            let ymax = info.bearing_y + info.height;
            let top = y + (self.ascent - ymax);
            out.push(GlyphQuad {
                dst: [pen_x + info.min_x, top, info.width, info.height],
                uv_rect: info.uv_rect,
                color: [color.r, color.g, color.b, color.a],
            });
            pen_x += info.advance;
        }
        out
    }

    /// Draw pre-laid-out glyph quads ON TOP of the current attachment
    /// contents (LoadOp::Load — text pass never clears).
    pub fn render(&self, gpu: &GpuDevice, view: &wgpu::TextureView, quads: &[GlyphQuad]) {
        self.render_inner(gpu, view, quads, None);
    }

    pub fn render_clipped(
        &self,
        gpu: &GpuDevice,
        view: &wgpu::TextureView,
        quads: &[GlyphQuad],
        clip: [u32; 4],
    ) {
        self.render_inner(gpu, view, quads, Some(clip));
    }

    fn render_inner(
        &self,
        gpu: &GpuDevice,
        view: &wgpu::TextureView,
        quads: &[GlyphQuad],
        clip: Option<[u32; 4]>,
    ) {
        if quads.is_empty() {
            return;
        }
        debug_assert!(quads.len() <= MAX_GLYPHS);
        let mut bytes: Vec<u8> = Vec::with_capacity(quads.len() * 48);
        for q in quads {
            for f in q.dst {
                bytes.extend_from_slice(&f.to_ne_bytes());
            }
            for f in q.uv_rect {
                bytes.extend_from_slice(&f.to_ne_bytes());
            }
            for f in q.color {
                bytes.extend_from_slice(&f.to_ne_bytes());
            }
        }
        gpu.queue().write_buffer(&self.instance_buf, 0, &bytes);
        self.uploads.record(bytes.len() as u64);
        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("agp text frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("agp text pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Some([x, y, width, height]) = clip {
                pass.set_scissor_rect(x, y, width, height);
            }
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..4, 0..quads.len() as u32);
        }
        gpu.queue().submit([encoder.finish()]);
    }
}
