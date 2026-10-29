//! PLAN-004 finite sprite drawing: one nearest-sampled, clamped, sRGB RGBA8
//! texture per accepted image and one instanced quad per admitted visual.
//!
//! The frame rectangle and the horizontal flip are UV facts only: the world
//! centre/size come from the scene, so a frame crop never changes an entity's
//! bounds. Textures are content-identified (original SHA) and cached by a
//! staged store, so a warm frame that only changes the frame or the flip
//! rewrites instance data and never re-uploads pixels.

use crate::assets::ImageData;
use crate::core::scene::Color;
use crate::renderer::device::GpuDevice;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Upper bound of sprite instances per 2D surface (16 admitted visuals plus
/// selection overlays stay far below this).
pub const MAX_SPRITES: usize = 256;

/// One immutable GPU image. Identity is the original content SHA, never a path.
pub struct SpriteTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub source_sha256: String,
    pub device_ordinal: Option<u64>,
}

/// The persistent, staged cache of accepted images for one 2D surface.
#[derive(Clone)]
pub struct SpriteStore {
    textures: HashMap<String, Arc<SpriteTexture>>,
    /// Real texture creations and uploaded pixel bytes (warm reuse adds none).
    pub uploads: Arc<crate::renderer::UploadCounters>,
    pub created: u64,
    pub bytes_uploaded: u64,
}

impl Default for SpriteStore {
    fn default() -> Self {
        Self {
            textures: HashMap::new(),
            uploads: Arc::new(crate::renderer::UploadCounters::default()),
            created: 0,
            bytes_uploaded: 0,
        }
    }
}

impl SpriteStore {
    /// A candidate view: existing immutable textures are shared, insertions
    /// stay local until the frame is published.
    pub fn staged(&self) -> Self {
        Self {
            textures: self.textures.clone(),
            uploads: self.uploads.clone(),
            created: self.created,
            bytes_uploaded: self.bytes_uploaded,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Arc<SpriteTexture>> {
        self.textures.get(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> + '_ {
        self.textures.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// Reuse the content-identical texture or create it once. The declared
    /// dimensions are re-checked so a drifted payload can never paint as an
    /// existing identity.
    pub fn get_or_upload(
        &mut self,
        gpu: &GpuDevice,
        key: &str,
        image: &ImageData,
    ) -> Result<Arc<SpriteTexture>, crate::renderer::error::RendererError> {
        if let Some(existing) = self.textures.get(key) {
            if existing.width != image.width
                || existing.height != image.height
                || existing.source_sha256 != image.source_sha256
            {
                return Err(crate::renderer::error::RendererError::ReportIo(
                    "sprite texture identity/dimension drift".into(),
                ));
            }
            if existing.device_ordinal != Some(gpu.token().ordinal) {
                return Err(crate::renderer::error::RendererError::ReportIo(
                    "sprite texture belongs to another device".into(),
                ));
            }
            return Ok(existing.clone());
        }
        self.get_or_upload_with(
            gpu.device(),
            gpu.queue(),
            key,
            image,
            Some(gpu.token().ordinal),
        )
    }

    /// Raw-device variant for the Iced seam (same device/queue ownership rule).
    pub fn get_or_upload_with(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: &str,
        image: &ImageData,
        ordinal: Option<u64>,
    ) -> Result<Arc<SpriteTexture>, crate::renderer::error::RendererError> {
        let expected = (image.width as usize)
            .checked_mul(image.height as usize)
            .and_then(|p| p.checked_mul(4))
            .ok_or_else(|| {
                crate::renderer::error::RendererError::ReportIo(
                    "sprite texture dimension overflow".into(),
                )
            })?;
        if expected != image.rgba.len() {
            return Err(crate::renderer::error::RendererError::ReportIo(
                "sprite texture payload length".into(),
            ));
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("agp sprite texture"),
            size: wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // The accepted bytes are sRGB RGBA8; the surface target is sRGB, so
            // sampling converts to linear and blending happens in linear space.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width * 4),
                rows_per_image: Some(image.height),
            },
            wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
        );
        self.uploads.record(expected as u64);
        self.created += 1;
        self.bytes_uploaded = self.bytes_uploaded.saturating_add(expected as u64);
        let texture = Arc::new(SpriteTexture {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
            width: image.width,
            height: image.height,
            source_sha256: image.source_sha256.clone(),
            device_ordinal: ordinal,
        });
        self.textures.insert(key.to_owned(), texture.clone());
        Ok(texture)
    }

    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.textures.retain(|k, _| keep(k.as_str()));
    }

    /// Every cached texture's creating device, for the same-device check.
    pub fn device_owners(&self) -> impl Iterator<Item = Option<u64>> + '_ {
        self.textures.values().map(|t| t.device_ordinal)
    }
}

/// One admitted visual in world space. `uv` is (x0, y0, x1, y1) normalised;
/// the flip is already baked into the U axis, so nothing else moves.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteCmd {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub uv: [f32; 4],
    pub color: [f32; 4],
    pub texture: String,
}

const VERT: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

struct Sprite {
    center_size: vec4<f32>, // cx, cy, w, h
    uv_rect: vec4<f32>,     // x0, y0, x1, y1
    color: vec4<f32>,       // tint (straight alpha)
};

@group(0) @binding(1)
var<storage, read> sprites: array<Sprite>;

@group(0) @binding(2)
var sprite_texture: texture_2d<f32>;

@group(0) @binding(3)
var sprite_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VertexOutput {
    let s = sprites[ii];
    let corner = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    ) * 0.5;
    let local = (corner - vec2<f32>(0.5, 0.5)) * s.center_size.zw;
    let world = vec3<f32>(s.center_size.xy + local, 0.0);
    var out: VertexOutput;
    out.position = camera * vec4<f32>(world, 1.0);
    // The 2D camera is screen-style (y=0 at the top), so corner.y=0 is the
    // image's first row and U keeps the corner order, which carries the flip.
    out.uv = vec2<f32>(
        s.uv_rect.x + corner.x * (s.uv_rect.z - s.uv_rect.x),
        s.uv_rect.y + corner.y * (s.uv_rect.w - s.uv_rect.y),
    );
    out.color = s.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(sprite_texture, sprite_sampler, in.uv) * in.color;
}
"#;

pub struct SpritePipeline {
    pipeline: wgpu::RenderPipeline,
    camera_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    groups: RefCell<HashMap<String, wgpu::BindGroup>>,
    pub uploads: crate::renderer::UploadCounters,
    pub device_ordinal: Option<u64>,
}

impl SpritePipeline {
    pub fn new(
        gpu: &GpuDevice,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, crate::renderer::error::RendererError> {
        let mut out = Self::new_with(gpu.device(), gpu.queue(), target_format)?;
        out.device_ordinal = Some(gpu.token().ordinal);
        Ok(out)
    }

    pub fn new_with(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, crate::renderer::error::RendererError> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp sprite"),
            source: wgpu::ShaderSource::Wgsl(VERT.into()),
        });
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp sprite camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp sprite instances"),
            size: (MAX_SPRITES * 48) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp sprite bgl"),
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
        // The frozen sampling rule: nearest, clamp to edge, no mipmaps.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("agp sprite sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agp sprite layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        crate::renderer::record_pipeline_creation();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp sprite pipeline"),
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
        if let Some(message) = crate::renderer::pipeline::pop_error_scope_blocking(device) {
            return Err(crate::renderer::error::RendererError::BadShader {
                stage: "sprite",
                message,
            });
        }
        Ok(Self {
            pipeline,
            camera_buf,
            instance_buf,
            layout,
            sampler,
            groups: RefCell::new(HashMap::new()),
            uploads: crate::renderer::UploadCounters::default(),
            device_ordinal: None,
        })
    }

    pub fn owned_buffer_bytes(&self) -> u64 {
        self.camera_buf.size() + self.instance_buf.size()
    }

    /// Drop cached bind groups (called when the accepted texture set changes).
    pub fn invalidate_bind_groups(&self) {
        self.groups.borrow_mut().clear();
    }

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

    /// Draw the admitted visuals over the existing 2D attachment. Order is the
    /// command order: every command is one four-vertex instance with its own
    /// texture binding, so z-order never depends on texture grouping.
    pub fn render_with(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        commands: &[SpriteCmd],
        store: &SpriteStore,
    ) -> Result<(), crate::renderer::error::RendererError> {
        if commands.is_empty() {
            return Ok(());
        }
        if commands.len() > MAX_SPRITES {
            return Err(crate::renderer::error::RendererError::ReportIo(
                "sprite command capacity".into(),
            ));
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(commands.len() * 48);
        for cmd in commands {
            for f in [
                cmd.center[0],
                cmd.center[1],
                cmd.size[0],
                cmd.size[1],
                cmd.uv[0],
                cmd.uv[1],
                cmd.uv[2],
                cmd.uv[3],
                cmd.color[0],
                cmd.color[1],
                cmd.color[2],
                cmd.color[3],
            ] {
                if !f.is_finite() {
                    return Err(crate::renderer::error::RendererError::ReportIo(
                        "sprite command is not finite".into(),
                    ));
                }
                bytes.extend_from_slice(&f.to_ne_bytes());
            }
        }
        queue.write_buffer(&self.instance_buf, 0, &bytes);
        self.uploads.record(bytes.len() as u64);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("agp sprite frame"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("agp sprite pass"),
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
            pass.set_pipeline(&self.pipeline);
            for (index, cmd) in commands.iter().enumerate() {
                let texture = store.get(&cmd.texture).ok_or_else(|| {
                    crate::renderer::error::RendererError::ReportIo(
                        "sprite texture missing at draw time".into(),
                    )
                })?;
                let mut groups = self.groups.borrow_mut();
                if !groups.contains_key(&cmd.texture) {
                    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("agp sprite bg"),
                        layout: &self.layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.camera_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.instance_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(&texture.view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    });
                    groups.insert(cmd.texture.clone(), group);
                }
                pass.set_bind_group(0, &groups[&cmd.texture], &[]);
                pass.draw(0..4, index as u32..index as u32 + 1);
            }
        }
        queue.submit([encoder.finish()]);
        Ok(())
    }
}

/// Convert an admitted pixel frame and flip into normalised UVs. The image is
/// sampled with the flip baked into U only: V and the world size never change.
pub fn frame_uv(frame: [u32; 4], width: u32, height: u32, flip: bool) -> Result<[f32; 4], String> {
    if width == 0 || height == 0 {
        return Err("sprite image has no extent".into());
    }
    let [x, y, w, h] = frame;
    if w == 0 || h == 0 || x > width - w || y > height - h {
        return Err("sprite frame outside image".into());
    }
    let (x0, x1) = (
        x as f32 / width as f32,
        (x as f32 + w as f32) / width as f32,
    );
    let (y0, y1) = (
        y as f32 / height as f32,
        (y as f32 + h as f32) / height as f32,
    );
    Ok(if flip {
        [x1, y0, x0, y1]
    } else {
        [x0, y0, x1, y1]
    })
}

/// The tint of an accepted visual is the object colour (straight alpha).
pub fn tint(color: Color) -> [f32; 4] {
    [color.r, color.g, color.b, color.a]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_uv_crops_and_flips_without_moving_the_world_rect() {
        // 96x32 sheet, three 32x32 frames.
        assert_eq!(
            frame_uv([0, 0, 32, 32], 96, 32, false).unwrap(),
            [0.0, 0.0, 32.0 / 96.0, 1.0]
        );
        assert_eq!(
            frame_uv([32, 0, 32, 32], 96, 32, false).unwrap(),
            [32.0 / 96.0, 0.0, 64.0 / 96.0, 1.0]
        );
        // Flip only swaps U.
        assert_eq!(
            frame_uv([32, 0, 32, 32], 96, 32, true).unwrap(),
            [64.0 / 96.0, 0.0, 32.0 / 96.0, 1.0]
        );
        assert!(frame_uv([80, 0, 32, 32], 96, 32, false).is_err());
        assert!(frame_uv([0, 0, 0, 32], 96, 32, false).is_err());
        assert!(frame_uv([0, 0, 32, 32], 0, 32, false).is_err());
    }
}
