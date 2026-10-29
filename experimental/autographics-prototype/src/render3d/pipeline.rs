//! Lit mesh pipeline with directional shadow mapping.
//!
//! The frozen GLBs are untextured (material baseColorFactor only — T-01
//! manifest §2.5), so the vertex format is position+normal+color and the
//! fragment does Lambert × directional light × shadow. Shadow mapping is
//! MANDATORY for the sample look (manifest §2.2; AC-04: no silent omission).

use crate::assets::ModelData;
use crate::renderer::device::GpuDevice;
use crate::renderer::error::RendererError;
use glam::{camera::rh::view::look_at_mat4, Mat4, Vec3};
use std::collections::HashMap;

pub const SHADOW_MAP_SIZE: u32 = 2048;
pub const MAX_INSTANCES: usize = 512;

/// A GPU-resident mesh uploaded from `MeshData`.
pub struct GpuMesh {
    pub texture: Option<std::sync::Arc<crate::renderer::sprite::SpriteTexture>>,
    texture_bind_group: Option<wgpu::BindGroup>,
    double_sided: bool,
    pub device_ordinal: Option<u64>,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
}

/// Identity-keyed mesh store: (model sha256, mesh name) → GPU buffers.
/// Uploaded once per identity; never rebuilt for unchanged models.
#[derive(Default)]
pub struct MeshStore {
    device: Option<wgpu::Device>,
    textures: crate::renderer::sprite::SpriteStore,
    meshes: HashMap<(String, String), std::sync::Arc<GpuMesh>>,
    pub uploads: std::sync::Arc<crate::renderer::UploadCounters>,
}

impl MeshStore {
    /// Private candidate resource view: immutable buffers are shared, insertions
    /// remain local. Attempts stay counted even when the candidate is discarded.
    pub fn staged(&self) -> Self {
        Self {
            device: self.device.clone(),
            textures: self.textures.staged(),
            meshes: self.meshes.clone(),
            uploads: self.uploads.clone(),
        }
    }

    /// Release map ownership after current/history/candidate owners are reconciled.
    pub fn retain_models(&mut self, keep: impl Fn(&str) -> bool) {
        self.meshes.retain(|(sha, _), _| keep(sha));
        let ids: std::collections::HashSet<_> = self
            .meshes
            .values()
            .filter_map(|m| m.texture.as_ref().map(|t| t.source_sha256.clone()))
            .collect();
        self.textures.retain(|id| ids.contains(id));
    }
    pub fn get_or_upload(
        &mut self,
        gpu: &GpuDevice,
        model: &ModelData,
        mesh_index: usize,
    ) -> Result<(String, u32), RendererError> {
        let key = (
            model.source_sha256.clone(),
            model
                .meshes
                .get(mesh_index)
                .ok_or_else(|| RendererError::ReportIo("mesh index".into()))?
                .name
                .clone(),
        );
        if let Some(existing) = self.meshes.get(&key) {
            if existing.device_ordinal != Some(gpu.token().ordinal) {
                return Err(RendererError::ReportIo("mesh device mismatch".into()));
            }
            return Ok((key.1.clone(), existing.index_count));
        }
        let result = self.get_or_upload_with(gpu.device(), gpu.queue(), model, mesh_index)?;
        std::sync::Arc::get_mut(self.meshes.get_mut(&key).unwrap())
            .unwrap()
            .device_ordinal = Some(gpu.token().ordinal);
        Ok(result)
    }

    /// Raw-device variant (Iced seam shares iced's device/queue).
    pub fn get_or_upload_with(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model: &ModelData,
        mesh_index: usize,
    ) -> Result<(String, u32), RendererError> {
        if self.device.as_ref().is_some_and(|old| old != device) {
            return Err(RendererError::ReportIo(
                "mesh/texture device mismatch".into(),
            ));
        }
        self.device = Some(device.clone());
        let mesh = model.meshes.get(mesh_index).ok_or_else(|| {
            RendererError::ReportIo(format!("mesh index {mesh_index} out of range"))
        })?;
        if mesh.positions.is_empty()
            || mesh.positions.len() != mesh.normals.len()
            || mesh.indices.is_empty()
            || mesh.indices.len() % 3 != 0
            || mesh
                .indices
                .iter()
                .any(|i| *i as usize >= mesh.positions.len())
        {
            return Err(RendererError::ReportIo("invalid mesh geometry".into()));
        }
        let key = (model.source_sha256.clone(), mesh.name.clone());
        if let Some(m) = self.meshes.get(&key) {
            return Ok((mesh.name.clone(), m.index_count));
        }
        let visual = mesh.visual.as_ref();
        let textured = visual.is_some_and(|v| v.image.is_some());
        if visual.is_some_and(|v| {
            v.uv.len() != mesh.positions.len() || v.uv.iter().flatten().any(|n| !n.is_finite())
        }) {
            return Err(RendererError::ReportIo("invalid mesh UV".into()));
        }
        let mut verts: Vec<f32> =
            Vec::with_capacity(mesh.positions.len() * if textured { 12 } else { 10 });
        for (i, p) in mesh.positions.iter().enumerate() {
            verts.extend_from_slice(p);
            verts.extend_from_slice(&mesh.normals[i]);
            verts.extend_from_slice(&mesh.material.base_color);
            if textured {
                verts.extend_from_slice(&visual.unwrap().uv[i]);
            }
        }
        if mesh.indices.iter().any(|i| *i > u16::MAX as u32) {
            return Err(RendererError::ReportIo(format!(
                "mesh '{}' exceeds u16 index range (this slice targets the small frozen GLBs)",
                mesh.name
            )));
        }
        let mut indices: Vec<u8> = Vec::with_capacity(mesh.indices.len() * 2);
        for idx in &mesh.indices {
            indices.extend_from_slice(&(*idx as u16).to_ne_bytes());
        }
        // Queue writes require four-byte alignment; draw count excludes padding.
        indices.resize((indices.len() + 3) & !3, 0);
        device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp3d vertices"),
            size: (verts.len() * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.uploads.record((verts.len() * 4) as u64);
        queue.write_buffer(&vertex_buf, 0, bytemuck_f32(&verts));
        let index_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp3d indices"),
            size: indices.len() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.uploads.record(indices.len() as u64);
        queue.write_buffer(&index_buf, 0, &indices);
        let index_count = mesh.indices.len() as u32;
        let texture = if let Some(image) = visual.and_then(|v| v.image.as_ref()) {
            if let Some(t) = self.textures.get(&image.source_sha256) {
                Some(t.clone())
            } else {
                Some(self.textures.get_or_upload_with(
                    device,
                    queue,
                    &image.source_sha256,
                    image,
                    None,
                )?)
            }
        } else {
            None
        };
        let texture_bind_group = texture.as_ref().map(|t| {
            let sampling = visual.unwrap().sampling;
            let filter = |n| {
                if n == 9728 {
                    wgpu::FilterMode::Nearest
                } else {
                    wgpu::FilterMode::Linear
                }
            };
            let wrap = |n| match n {
                33071 => wgpu::AddressMode::ClampToEdge,
                33648 => wgpu::AddressMode::MirrorRepeat,
                _ => wgpu::AddressMode::Repeat,
            };
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("model sampler"),
                address_mode_u: wrap(sampling.wrap_s),
                address_mode_v: wrap(sampling.wrap_t),
                mag_filter: filter(sampling.mag),
                min_filter: filter(sampling.min),
                lod_min_clamp: 0.,
                // Keep lambda available for min/mag selection. The image has
                // exactly one mip level, so the texture still samples level0.
                // Clamping sampler lambda itself to zero incorrectly selects
                // mag_filter even when the footprint is minifying.
                lod_max_clamp: 32.,
                ..Default::default()
            });
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("model texture binding"),
                layout: &model_texture_layout(device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&t.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        });
        self.meshes.insert(
            key.clone(),
            std::sync::Arc::new(GpuMesh {
                texture,
                texture_bind_group,
                double_sided: visual.is_some_and(|v| v.double_sided),
                device_ordinal: None,
                vertex_buf,
                index_buf,
                index_count,
            }),
        );
        let validation = crate::renderer::pipeline::pop_error_scope_blocking(device);
        let memory = crate::renderer::pipeline::pop_error_scope_blocking(device);
        if let Some(error) = validation.or(memory) {
            self.meshes.remove(&key);
            return Err(RendererError::ReportIo(format!(
                "mesh preparation: {error}"
            )));
        }
        Ok((mesh.name.clone(), index_count))
    }

    pub fn get(&self, model_sha: &str, mesh_name: &str) -> Option<&GpuMesh> {
        self.meshes
            .get(&(model_sha.to_string(), mesh_name.to_string()))
            .map(|mesh| mesh.as_ref())
    }

    pub fn pin(&self, model_sha: &str, mesh_name: &str) -> Option<std::sync::Arc<GpuMesh>> {
        self.meshes
            .get(&(model_sha.to_owned(), mesh_name.to_owned()))
            .cloned()
    }
    pub fn device_owners(&self) -> impl Iterator<Item = Option<u64>> + '_ {
        self.meshes.values().map(|m| m.device_ordinal)
    }
    pub fn owned_buffer_bytes(&self) -> u64 {
        self.meshes
            .values()
            .map(|m| m.vertex_buf.size() + m.index_buf.size())
            .sum()
    }
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }
    pub fn texture_uploads(&self) -> u64 {
        self.textures.created
    }
    pub fn owned_texture_bytes(&self) -> u64 {
        self.textures
            .keys()
            .filter_map(|k| self.textures.get(k))
            .map(|t| t.width as u64 * t.height as u64 * 4)
            .sum()
    }
    pub fn uploaded_count(&self) -> usize {
        self.meshes.len()
    }
}

fn bytemuck_f32(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}

/// One draw instance.
#[derive(Debug, Clone)]
pub struct MeshInstance {
    /// (model sha256, mesh name)
    pub mesh: (String, String),
    pub model: Mat4,
}

/// Directional light + shadow frustum.
#[derive(Debug, Clone, Copy)]
pub struct LightDef {
    /// Unit vector pointing FROM the surface TOWARD the light.
    pub dir_to_light: Vec3,
    pub color: [f32; 3],
    pub energy: f32,
    /// Orthographic view-projection for the shadow map.
    pub shadow_vp: Mat4,
    pub shadow_bias: f32,
}

/// Build the light from the frozen `Main.tscn` DirectionalLight3D
/// (lines 84–91): basis z-axis (0.382355, 0.897258, 0.220753) shines along
/// -z; shadow frustum covers the arena (±32 world units).
pub fn sample_light() -> LightDef {
    let dir_to_light = Vec3::new(0.382_355, 0.897_258, 0.220_753).normalize();
    let center = Vec3::new(0.5, 0.0, 0.5);
    // The shadow camera sits AT the light (center + dir), looking at the
    // arena — placing it on the opposite side puts the ground plane's
    // back face toward the depth pass and culling empties the map.
    let eye = center + dir_to_light * 60.0;
    let view = look_at_mat4(eye, center, Vec3::Y);
    let r = 32.0f32;
    let (near, far) = (1.0f32, 160.0f32);
    let proj = Mat4::from_cols_array(&[
        1.0 / r,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0 / r,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0 / (far - near),
        0.0,
        0.0,
        0.0,
        -near / (far - near),
        1.0,
    ]);
    LightDef {
        dir_to_light,
        color: [1.0, 1.0, 1.0],
        energy: 1.0,
        shadow_vp: proj * view,
        shadow_bias: 0.0005,
    }
}

const SHADER: &str = r#"
struct Camera { vp: mat4x4<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

struct Light {
    dir_to_light: vec4<f32>,
    color_energy: vec4<f32>,
    shadow_vp: mat4x4<f32>,
    params: vec4<f32>, // x: bias, y: shadow_enabled, z: ambient, w: unused
};
@group(0) @binding(1) var<uniform> light: Light;

struct Instance { model: mat4x4<f32> };
@group(0) @binding(2) var<storage, read> instances: array<Instance>;

@group(0) @binding(3) var shadow_map: texture_depth_2d;
@group(0) @binding(4) var shadow_cmp: sampler_comparison;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) shadow_pos: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(instance_index) ii: u32,
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
) -> VsOut {
    let model = instances[ii].model;
    let world = model * vec4<f32>(pos, 1.0);
    var out: VsOut;
    out.clip = camera.vp * world;
    out.world_normal = normalize((model * vec4<f32>(normal, 0.0)).xyz);
    out.color = color;
    out.shadow_pos = light.shadow_vp * world;
    return out;
}

/// Shadow-pass vertex: identical transform math, but only touches bindings
/// the shadow layout actually provides (light + instances).
@vertex
fn vs_shadow(
    @builtin(instance_index) ii: u32,
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
) -> VsOut {
    let model = instances[ii].model;
    let world = model * vec4<f32>(pos, 1.0);
    var out: VsOut;
    out.clip = light.shadow_vp * world;
    out.world_normal = normalize((model * vec4<f32>(normal, 0.0)).xyz);
    out.color = color;
    out.shadow_pos = light.shadow_vp * world;
    return out;
}

@fragment
fn fs_main(in: VsOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal) * select(-1.0, 1.0, front);
    let l = normalize(light.dir_to_light.xyz);
    let ndl = max(dot(n, l), 0.0);
    var shadow = 1.0;
    if (light.params.y > 0.5) {
        let sp = in.shadow_pos;
        let uv = vec2<f32>(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);
        if (uv.x > 0.0 && uv.x < 1.0 && uv.y > 0.0 && uv.y < 1.0 && sp.z > 0.0 && sp.z < 1.0) {
            shadow = textureSampleCompare(shadow_map, shadow_cmp, uv, sp.z - light.params.x);
        }
    }
    let ambient = light.params.z;
    let lit = ambient + ndl * light.color_energy.a * shadow;
    return vec4<f32>(in.color.rgb * light.color_energy.rgb * lit, in.color.a);
}
"#;

/// Stats recorded per frame (probe evidence).
#[derive(Debug, Clone, Default)]
pub struct Render3dStats {
    pub draw_calls: u32,
    pub instances: u32,
    pub shadow_enabled: bool,
}

pub struct MeshPipeline {
    color_double: wgpu::RenderPipeline,
    color_texture: wgpu::RenderPipeline,
    color_texture_double: wgpu::RenderPipeline,
    shadow_double: wgpu::RenderPipeline,
    shadow_texture: wgpu::RenderPipeline,
    shadow_texture_double: wgpu::RenderPipeline,
    target_format: wgpu::TextureFormat,
    editor_grid: Option<super::editor_grid::EditorGrid>,
    editor_grid_enabled: bool,
    editor_grid_revision: u64,
    shadow_sabotage: bool,
    color_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    color_bind_group: wgpu::BindGroup,
    shadow_bind_group: wgpu::BindGroup,
    camera_buf: wgpu::Buffer,
    light_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    shadow_view: wgpu::TextureView,
    depth: Option<(u32, u32, wgpu::TextureView)>,
    /// Buffer-rewrite accounting (F08), public for honest measurement.
    pub uploads: crate::renderer::UploadCounters,
    /// Host device that created this pipeline (when known). A composition
    /// runtime refuses 3D content built on a FOREIGN device exactly like 2D
    /// content (review r2 F04: the old `Scene3d` branch returned `None` and
    /// silently passed the ownership check).
    pub device_ordinal: Option<u64>,
}

impl MeshPipeline {
    /// Update an optional bounded, depth-tested editor-only line pass.
    /// Invalid input leaves the previous state untouched; disabled means no draw or upload.
    pub fn set_editor_grid(
        &mut self,
        gpu: &GpuDevice,
        vertices: Option<&[super::editor_grid::EditorGridVertex]>,
    ) -> Result<(), String> {
        if self.device_ordinal != Some(gpu.token().ordinal) {
            return Err("editor grid device mismatch".into());
        }
        if let Some(vertices) = vertices {
            if vertices.len() > super::editor_grid::MAX_EDITOR_GRID_VERTICES
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
            let next = self
                .editor_grid_revision
                .checked_add(1)
                .ok_or("editor grid revision exhausted")?;
            if self.editor_grid.is_none() {
                self.editor_grid = Some(
                    super::editor_grid::EditorGrid::new(gpu.device(), self.target_format)
                        .map_err(|e| format!("{e:?}"))?,
                );
            }
            let changed =
                self.editor_grid.as_mut().unwrap().set(vertices)? || !self.editor_grid_enabled;
            if changed {
                self.editor_grid_revision = next;
            }
            self.editor_grid_enabled = true;
        } else if self.editor_grid_enabled {
            self.editor_grid_revision = self
                .editor_grid_revision
                .checked_add(1)
                .ok_or("editor grid revision exhausted")?;
            self.editor_grid_enabled = false;
        }
        Ok(())
    }
    pub fn editor_grid_revision(&self) -> u64 {
        self.editor_grid_revision
    }
    pub fn editor_grid_owned_bytes(&self) -> u64 {
        self.editor_grid.as_ref().map_or(0, |g| g.owned_bytes())
    }
    /// Weak ownership probe for lifecycle verification.
    pub fn editor_grid_owner_probe(&self) -> Option<std::sync::Weak<()>> {
        self.editor_grid.as_ref().map(|g| g.owner_probe())
    }
    pub fn editor_grid_cpu_capacity_bytes(&self) -> usize {
        self.editor_grid
            .as_ref()
            .map_or(0, |g| g.cpu_capacity_bytes())
    }
    pub fn editor_grid_uploads(&self) -> crate::renderer::UploadStats {
        self.editor_grid
            .as_ref()
            .map_or_else(Default::default, |g| g.stats())
    }
    pub fn set_shadow_sabotage(&mut self, value: bool) {
        self.shadow_sabotage = value;
    }
    pub fn owned_buffer_bytes(&self) -> u64 {
        self.camera_buf.size()
            + self.light_buf.size()
            + self.instance_buf.size()
            + self.editor_grid_owned_bytes()
    }
    pub fn owned_depth_texture_bytes(&self) -> u64 {
        u64::from(SHADOW_MAP_SIZE) * u64::from(SHADOW_MAP_SIZE) * 4
            + self
                .depth
                .as_ref()
                .map_or(0, |(w, h, _)| u64::from(*w) * u64::from(*h) * 4)
    }
    pub fn new(gpu: &GpuDevice, target_format: wgpu::TextureFormat) -> Result<Self, RendererError> {
        let mut out = Self::new_with(gpu.device(), gpu.queue(), target_format)?;
        out.device_ordinal = Some(gpu.token().ordinal);
        Ok(out)
    }

    /// Raw-device variant (Iced seam).
    pub fn new_with(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp3d"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp3d camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp3d light"),
            size: 160,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("agp3d instances"),
            size: (MAX_INSTANCES * 64) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("agp3d shadow map"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_cmp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("agp3d shadow cmp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let color_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp3d color bgl"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX),
                uniform_entry(1, wgpu::ShaderStages::VERTEX_FRAGMENT),
                storage_entry(2, wgpu::ShaderStages::VERTEX),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });
        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("agp3d color bg"),
            layout: &color_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: instance_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&shadow_cmp),
                },
            ],
        });

        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp3d shadow bgl"),
            entries: &[
                uniform_entry(1, wgpu::ShaderStages::VERTEX),
                storage_entry(2, wgpu::ShaderStages::VERTEX),
            ],
        });
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("agp3d shadow bg"),
            layout: &shadow_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: instance_buf.as_entire_binding(),
                },
            ],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: 40,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };
        let color_target = Some(wgpu::ColorTargetState {
            format: target_format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        });
        let depth_state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };
        let shadow_depth_state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2,
                slope_scale: 2.0,
                clamp: 0.0,
            },
        };

        let mk_layout = |label: &str, bgl: &wgpu::BindGroupLayout| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[bgl],
                push_constant_ranges: &[],
            })
        };

        device.push_error_scope(wgpu::ErrorFilter::Validation);
        crate::renderer::record_pipeline_creation();
        let color_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp3d color"),
            layout: Some(&mk_layout("agp3d color layout", &color_layout)),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: std::slice::from_ref(&vertex_layout),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[color_target.clone()],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(depth_state.clone()),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        crate::renderer::record_pipeline_creation();
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp3d shadow"),
            layout: Some(&mk_layout("agp3d shadow layout", &shadow_layout)),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_shadow"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: std::slice::from_ref(&vertex_layout),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(shadow_depth_state.clone()),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let texture_source=SHADER
            .replace("struct VsOut {", "@group(1) @binding(0) var model_image: texture_2d<f32>;\n@group(1) @binding(1) var model_sampler: sampler;\nstruct VsOut {")
            .replace("@location(2) shadow_pos: vec4<f32>,", "@location(2) shadow_pos: vec4<f32>,\n    @location(3) uv: vec2<f32>,")
            .replace("@location(2) color: vec4<f32>,\n)", "@location(2) color: vec4<f32>,\n    @location(3) uv: vec2<f32>,\n)")
            .replace("out.color = color;", "out.color = color; out.uv = uv;")
            .replace("in.color.rgb * light.color_energy.rgb", "in.color.rgb * textureSample(model_image, model_sampler, in.uv).rgb * light.color_energy.rgb");
        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("model texture shader"),
            source: wgpu::ShaderSource::Wgsl(texture_source.into()),
        });
        let texture_layout = model_texture_layout(device);
        let color_texture_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("model textured layout"),
            bind_group_layouts: &[&color_layout, &texture_layout],
            push_constant_ranges: &[],
        });
        let mut attributes = vertex_layout.attributes.to_vec();
        attributes.push(wgpu::VertexAttribute {
            offset: 40,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32x2,
        });
        let texture_vertices = wgpu::VertexBufferLayout {
            array_stride: 48,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attributes,
        };
        let make_color = |textured: bool, double: bool| {
            let plain_layout = mk_layout("model double sided color", &color_layout);
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("model visual color"),
                layout: Some(if textured {
                    &color_texture_layout
                } else {
                    &plain_layout
                }),
                vertex: wgpu::VertexState {
                    module: if textured { &texture_shader } else { &shader },
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: std::slice::from_ref(if textured {
                        &texture_vertices
                    } else {
                        &vertex_layout
                    }),
                },
                fragment: Some(wgpu::FragmentState {
                    module: if textured { &texture_shader } else { &shader },
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[color_target.clone()],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: if double { None } else { Some(wgpu::Face::Back) },
                    ..Default::default()
                },
                depth_stencil: Some(depth_state.clone()),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let make_shadow = |textured: bool, double: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("model visual shadow"),
                layout: Some(&mk_layout("model visual shadow layout", &shadow_layout)),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_shadow"),
                    compilation_options: Default::default(),
                    buffers: std::slice::from_ref(if textured {
                        &texture_vertices
                    } else {
                        &vertex_layout
                    }),
                },
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    cull_mode: if double { None } else { Some(wgpu::Face::Back) },
                    ..Default::default()
                },
                depth_stencil: Some(shadow_depth_state.clone()),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let color_double = make_color(false, true);
        let color_texture = make_color(true, false);
        let color_texture_double = make_color(true, true);
        let shadow_double = make_shadow(false, true);
        let shadow_texture = make_shadow(true, false);
        let shadow_texture_double = make_shadow(true, true);
        if let Some(msg) = crate::renderer::pipeline::pop_error_scope_blocking(device) {
            return Err(RendererError::BadShader {
                stage: "3d mesh",
                message: msg,
            });
        }

        Ok(MeshPipeline {
            color_double,
            color_texture,
            color_texture_double,
            shadow_double,
            shadow_texture,
            shadow_texture_double,
            target_format,
            editor_grid: None,
            editor_grid_enabled: false,
            editor_grid_revision: 0,
            shadow_sabotage: false,
            color_pipeline,
            shadow_pipeline,
            color_bind_group,
            shadow_bind_group,
            camera_buf,
            light_buf,
            instance_buf,
            shadow_view,
            depth: None,
            uploads: crate::renderer::UploadCounters::default(),
            // Raw-device construction: the caller owns the device and we
            // cannot verify identity. `new()` stamps the host ordinal.
            device_ordinal: None,
        })
    }

    /// Render all instances (shadow pass + lit pass) into `view`.
    pub fn render(
        &mut self,
        gpu: &GpuDevice,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        clear: [f32; 4],
        camera_vp: &Mat4,
        light: &LightDef,
        store: &MeshStore,
        instances: &[MeshInstance],
        shadows: bool,
    ) -> Render3dStats {
        self.render_with(
            gpu.device(),
            gpu.queue(),
            view,
            width,
            height,
            clear,
            camera_vp,
            light,
            store,
            instances,
            shadows,
        )
    }

    /// Raw-device variant (Iced seam shares iced's device/queue).
    pub fn render_with(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        clear: [f32; 4],
        camera_vp: &Mat4,
        light: &LightDef,
        store: &MeshStore,
        instances: &[MeshInstance],
        shadows: bool,
    ) -> Render3dStats {
        let shadows = shadows && !self.shadow_sabotage;
        // Ensure a depth buffer matching the target.
        match &self.depth {
            Some((w, h, _)) if *w == width && *h == height => {}
            _ => {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("agp3d depth"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                self.depth = Some((
                    width,
                    height,
                    tex.create_view(&wgpu::TextureViewDescriptor::default()),
                ));
            }
        }
        let depth_view = self.depth.as_ref().unwrap().2.clone();

        // Uniforms.
        let cam_bytes = mat4_bytes(camera_vp);
        queue.write_buffer(&self.camera_buf, 0, &cam_bytes);
        self.uploads.record(64);
        let mut light_bytes = [0u8; 160];
        let dir = light.dir_to_light.normalize();
        light_bytes[0..16]
            .copy_from_slice(&[dir.x, dir.y, dir.z, 0.0].map(|f| f.to_ne_bytes()).concat());
        let ce = [light.color[0], light.color[1], light.color[2], light.energy];
        light_bytes[16..32].copy_from_slice(&ce.map(|f| f.to_ne_bytes()).concat());
        light_bytes[32..96].copy_from_slice(&mat4_bytes(&light.shadow_vp));
        let params = [
            light.shadow_bias,
            if shadows { 1.0 } else { 0.0 },
            0.25,
            0.0,
        ];
        light_bytes[96..112].copy_from_slice(&params.map(|f| f.to_ne_bytes()).concat());
        queue.write_buffer(&self.light_buf, 0, &light_bytes);
        self.uploads.record(160);

        // Instance buffer, grouped by mesh for one draw per mesh group.
        let mut model_bytes: Vec<u8> = Vec::with_capacity(instances.len() * 64);
        let mut groups: Vec<(&GpuMesh, u32, u32)> = Vec::new(); // (mesh, first, count)
        for inst in instances {
            if let Some(mesh) = store.get(&inst.mesh.0, &inst.mesh.1) {
                if model_bytes.len() / 64 >= MAX_INSTANCES {
                    break;
                }
                let first = (model_bytes.len() / 64) as u32;
                model_bytes.extend_from_slice(&mat4_bytes(&inst.model));
                match groups.last_mut() {
                    Some((g, _, count)) if std::ptr::eq(*g, mesh) => *count += 1,
                    _ => groups.push((mesh, first, 1)),
                }
            }
        }
        if !model_bytes.is_empty() {
            queue.write_buffer(&self.instance_buf, 0, &model_bytes);
            self.uploads.record(model_bytes.len() as u64);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("agp3d frame"),
        });

        // Shadow pass.
        if shadows && !groups.is_empty() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("agp3d shadow pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &self.shadow_bind_group, &[]);
            for (mesh, first, count) in &groups {
                pass.set_pipeline(match (mesh.texture.is_some(), mesh.double_sided) {
                    (true, true) => &self.shadow_texture_double,
                    (true, false) => &self.shadow_texture,
                    (false, true) => &self.shadow_double,
                    (false, false) => &self.shadow_pipeline,
                });
                pass.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
                pass.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_indexed(0..mesh.index_count, 0, *first..*first + *count);
            }
        }

        // Lit pass.
        let mut draws = 0u32;
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("agp3d color pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear[0] as f64,
                            g: clear[1] as f64,
                            b: clear[2] as f64,
                            a: clear[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.color_pipeline);
            pass.set_bind_group(0, &self.color_bind_group, &[]);
            for (mesh, first, count) in &groups {
                pass.set_pipeline(match (mesh.texture.is_some(), mesh.double_sided) {
                    (true, true) => &self.color_texture_double,
                    (true, false) => &self.color_texture,
                    (false, true) => &self.color_double,
                    (false, false) => &self.color_pipeline,
                });
                if let Some(binding) = &mesh.texture_bind_group {
                    pass.set_bind_group(1, binding, &[]);
                }
                pass.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
                pass.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_indexed(0..mesh.index_count, 0, *first..*first + *count);
                draws += 1;
            }
        }
        if self.editor_grid_enabled {
            if let Some(grid) = &mut self.editor_grid {
                grid.encode(queue, &mut encoder, view, &depth_view, camera_vp);
            }
        }
        queue.submit([encoder.finish()]);
        Render3dStats {
            draw_calls: draws,
            instances: model_bytes.len() as u32 / 64,
            shadow_enabled: shadows,
        }
    }
}

fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn mat4_bytes(m: &Mat4) -> [u8; 64] {
    let arr: [f32; 16] = m.to_cols_array();
    unsafe { std::mem::transmute(arr) }
}

fn model_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("model texture bgl"),
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
    })
}
