//! Iced shared-device spike (T-08): a real minimal iced 0.14 application
//! whose genuine widgets (button + label) and our 2D/3D graphics views share
//! ONE wgpu device/queue — the one iced_wgpu hands to our custom primitive's
//! `prepare`. Pattern per the production mpv precedent
//! (`crates/auto-lang/src/ui/mpv/widget.rs`): `shader::Program` widget →
//! data-only `Primitive` → `iced_wgpu::primitive::Pipeline` created with
//! iced's device. NO production UI code is used or replicated; this is an
//! independent minimal probe.
//!
//! Shared by the `iced_spike` binary (interactive) and the
//! `iced-shared-device` probe case (scripted canonical run).

use iced::widget::{button, column, container, row, shader, text};
use iced::window;
use iced::{Element, Length, Size, Subscription, Task};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use crate::assets::load_glb_file;
use crate::core::scene::Color;
use crate::render3d::{Camera3d, MeshInstance, MeshPipeline, MeshStore};
use crate::renderer::capture::sha256_hex;
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};

pub const WINDOW_TITLE: &str = "AGP iced-shared-device spike";

// ---------------------------------------------------------------------------
// Mode (set once before run; iced's builder takes plain fn pointers)
// ---------------------------------------------------------------------------

static MODE: OnceLock<SpikeMode> = OnceLock::new();

#[derive(Clone)]
pub enum SpikeMode {
    Live,
    Canonical { report_dir: PathBuf },
}

pub fn set_mode(mode: SpikeMode) {
    let _ = MODE.set(mode);
}

fn mode() -> SpikeMode {
    MODE.get().cloned().unwrap_or(SpikeMode::Canonical {
        report_dir: PathBuf::from("scratch/829/iced-shared-device"),
    })
}

// ---------------------------------------------------------------------------
// Shared evidence (app ↔ primitive ↔ probe case)
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct SpikeEvidence {
    pub view2d_rendered_on_iced_device: bool,
    pub view3d_rendered_on_iced_device: bool,
    pub presses: u32,
    pub window_id: Option<window::Id>,
    pub window_size: Option<Size>,
    pub resized_sizes: Vec<Size>,
    pub scale_factor: Option<f32>,
    pub click_dispatched: bool,
    pub click_observed: bool,
    /// Which input path delivered the press: "os_focus" (real foreground
    /// click) or "posted_queue" (WM_LBUTTON posted to the window's message
    /// queue — still winit's real input pipeline, used when the session
    /// denies focus stealing). Honest evidence per review r1.
    pub click_path: String,
    /// Blit-draw counters sampled at each key moment (name, [2d, 3d]):
    /// proof the final composite draw actually executed.
    pub blit_counts: Vec<(String, [u64; 2])>,
    /// F829-R4-05: per-moment content analysis of the REAL composed window
    /// pixels (iced's in-compositor `window::screenshot` readback — the
    /// identical full-UI draw the window presents, not a re-render of our
    /// own view textures).
    pub composite_shots: Vec<CompositeShotFacts>,
    pub notes: Vec<String>,
}

/// Content facts for one composed-window screenshot (review r4 F829-R4-05).
/// Every check must hold: the gate rejects skipped draws (F05 counters),
/// a fully transparent/blank final composite, and wrong target/clip
/// placement, while the legitimate click/resize/DPI case stays green.
#[derive(Debug, Clone)]
pub struct CompositeShotFacts {
    pub moment: String,
    pub physical_w: u32,
    pub physical_h: u32,
    pub scale_factor: f32,
    pub non_blank: bool,
    pub controls_present: bool,
    pub view2d_present: bool,
    pub view3d_present: bool,
    /// r6 F829-R6-02: the observed 2D view top must match the layout-
    /// anchored expectation — a displaced composite fails here.
    pub position_ok: bool,
    /// r8 F829-R8-01: PER-VIEW placement — each view's observed extent must
    /// match its own anchored rect, independently of the other view.
    pub position_2d_ok: bool,
    pub position_3d_ok: bool,
    pub clip_ok: bool,
    pub detail: String,
}

impl CompositeShotFacts {
    pub fn ok(&self) -> bool {
        self.non_blank
            && self.controls_present
            && self.view2d_present
            && self.view3d_present
            && self.position_ok
            && self.position_2d_ok
            && self.position_3d_ok
            && self.clip_ok
    }
}

pub type SharedEvidence = Arc<Mutex<SpikeEvidence>>;

pub static SHARED_EVIDENCE: std::sync::LazyLock<SharedEvidence> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(SpikeEvidence::default())));

/// Graphics time, advanced by the app each Tick; the widget renders it.
#[derive(Default)]
pub struct SpikeGraphics {
    pub tick: u64,
    pub frozen: bool,
}

pub static CLICK_OBSERVED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Final-composite draw counters (review r1 F05): incremented ONLY adjacent
/// to the real `render_pass.draw` in the blit — the review's blank-blit
/// mutation (skip the draw, return true) never increments them, so the gate
/// can prove the composite actually happened. GDI window capture
/// (PrintWindow/CopyFromScreen) was attempted and HANGS on the presenting
/// wgpu window from a non-owner process in this environment — recorded in
/// the report; these counters are the deterministic in-band evidence.
pub static BLIT_DRAWS: [std::sync::atomic::AtomicU64; 2] = [
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
];

fn blit_counter(kind: ViewKind) -> &'static std::sync::atomic::AtomicU64 {
    match kind {
        ViewKind::View2d => &BLIT_DRAWS[0],
        ViewKind::View3d => &BLIT_DRAWS[1],
    }
}

pub static GRAPHICS: std::sync::LazyLock<Mutex<SpikeGraphics>> =
    std::sync::LazyLock::new(|| Mutex::new(SpikeGraphics::default()));

/// View textures stashed by the primitive pipelines for keyframe capture.
struct ViewStash {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    width: u32,
    height: u32,
}

static VIEW2D_STASH: Mutex<Option<ViewStash>> = Mutex::new(None);
static VIEW3D_STASH: Mutex<Option<ViewStash>> = Mutex::new(None);

/// The device iced_wgpu hands to our pipeline (cloned), for the probe's
/// cross-device discrimination test.
static ICED_DEVICE: Mutex<Option<wgpu::Device>> = Mutex::new(None);

pub fn iced_device() -> Option<wgpu::Device> {
    ICED_DEVICE.lock().unwrap().clone()
}

fn stash_slot(kind: ViewKind) -> &'static Mutex<Option<ViewStash>> {
    match kind {
        ViewKind::View2d => &VIEW2D_STASH,
        ViewKind::View3d => &VIEW3D_STASH,
    }
}

fn stash_view(
    kind: ViewKind,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    w: u32,
    h: u32,
) {
    *stash_slot(kind).lock().unwrap() = Some(ViewStash {
        device: device.clone(),
        queue: queue.clone(),
        texture: texture.clone(),
        width: w,
        height: h,
    });
}

/// Capture a keyframe of a view texture (GPU readback).
pub fn capture_view(kind: ViewKind) -> Option<(String, Vec<u8>, u32, u32)> {
    let stash = stash_slot(kind).lock().unwrap();
    let s = stash.as_ref()?;
    let png = capture_texture(&s.device, &s.queue, &s.texture, s.width, s.height).ok()?;
    Some((sha256_hex(&png), png, s.width, s.height))
}

fn capture_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    let padded = (width * 4 + 255) & !255;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("iced keyframe readback"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let img = {
        let mapped = buffer.slice(..).get_mapped_range();
        let mut raw = Vec::with_capacity((width * 4 * height) as usize);
        for y in 0..height {
            raw.extend_from_slice(
                &mapped[(y * padded) as usize..(y * padded + width * 4) as usize],
            );
        }
        drop(mapped);
        buffer.unmap();
        image::RgbaImage::from_raw(width, height, raw).ok_or("rgba buffer")?
    };
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// Custom shader widget: the graphics views
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewKind {
    View2d,
    View3d,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ButtonPressed,
    Resized(window::Id, Size),
    /// F829-R4-05: the real composed-window pixels (iced in-compositor
    /// screenshot readback) tagged with the capture moment.
    CompositeShot(String, iced::window::Screenshot),
}

pub struct GraphicsProgram {
    pub kind: ViewKind,
}

#[derive(Default)]
pub struct GraphicsState;

#[derive(Debug, Clone)]
pub struct GraphicsPrimitive {
    pub kind: ViewKind,
    pub tick: u64,
    pub presses: u32,
}

impl shader::Program<Message> for GraphicsProgram {
    type State = GraphicsState;
    type Primitive = GraphicsPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        let g = GRAPHICS.lock().unwrap();
        let presses = SHARED_EVIDENCE.lock().unwrap().presses;
        GraphicsPrimitive {
            kind: self.kind,
            tick: g.tick,
            presses,
        }
    }
}

impl iced_wgpu::primitive::Primitive for GraphicsPrimitive {
    type Pipeline = GraphicsPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &iced::Rectangle,
        viewport: &iced::advanced::graphics::Viewport,
    ) {
        let w = bounds.width.round().max(1.0) as u32;
        let h = bounds.height.round().max(1.0) as u32;
        {
            let mut ev = SHARED_EVIDENCE.lock().unwrap();
            if ev.scale_factor.is_none() {
                ev.scale_factor = Some(viewport.scale_factor());
            }
        }
        pipeline.render_content(self.kind, device, queue, self.tick, self.presses, w, h);
        {
            let mut ev = SHARED_EVIDENCE.lock().unwrap();
            match self.kind {
                ViewKind::View2d => ev.view2d_rendered_on_iced_device = true,
                ViewKind::View3d => ev.view3d_rendered_on_iced_device = true,
            }
        }
        stash_view(self.kind, device, queue, pipeline.texture(self.kind), w, h);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        pipeline.blit(self.kind, render_pass)
    }
}

/// Renders our 2D/3D content into a persistent texture created on ICED's
/// device (in `new`), then blits it in iced's render pass (clipped to the
/// widget bounds by iced's layered rendering).
/// Per-view persistent target (texture + blit bind group), one per view
/// kind: iced creates ONE `GraphicsPipeline` per primitive TYPE, so both
/// views share this pipeline and must each own their texture.
pub struct ViewTarget {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    size: (u32, u32),
}

pub struct GraphicsPipeline {
    device: wgpu::Device,
    blit: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    targets: std::collections::HashMap<ViewKind, ViewTarget>,
    quad2d: Option<QuadPipeline>,
    mesh3d: Option<MeshPipeline>,
    store3d: Option<MeshStore>,
    keys3d: Option<Vec<(String, String)>>,
}

const BLIT_WGSL: &str = r#"
@group(0) @binding(0) var view_tex: texture_2d<f32>;
@group(0) @binding(1) var view_smp: sampler;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0),
    );
    var out: VsOut;
    out.position = vec4<f32>(pos[vi], 0.0, 1.0);
    out.uv = uv[vi];
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(view_tex, view_smp, in.uv);
    return vec4<f32>(c.rgb, 1.0);
}
"#;

impl iced_wgpu::primitive::Pipeline for GraphicsPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // Sabotage path (review r4 F829-R4-05 permanent negative control):
        // the final fragment is forced fully transparent — offscreen view
        // textures stay intact and the real draw calls keep advancing, but
        // the composed window shows nothing. The composite CONTENT gate
        // (window::screenshot pixel analysis) must go RED.
        let wgsl: String =
            if std::env::var("AGP_SABOTAGE_ICED").ok().as_deref() == Some("transparent_blit") {
                BLIT_WGSL.replace(
                    "return vec4<f32>(c.rgb, 1.0);",
                    "return vec4<f32>(0.0, 0.0, 0.0, 0.0);",
                )
            } else {
                BLIT_WGSL.to_string()
            };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("agp iced blit"),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("agp iced blit layout"),
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
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("agp iced blit pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let blit = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("agp iced blit"),
            layout: Some(&layout),
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            label: Some("agp iced sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        *ICED_DEVICE.lock().unwrap() = Some(device.clone());
        GraphicsPipeline {
            device: device.clone(),
            blit,
            bind_group_layout,
            sampler,
            targets: std::collections::HashMap::new(),
            quad2d: None,
            mesh3d: None,
            store3d: None,
            keys3d: None,
        }
    }
}

impl GraphicsPipeline {
    pub fn texture(&self, kind: ViewKind) -> &wgpu::Texture {
        &self.targets.get(&kind).expect("texture prepared").texture
    }

    /// Exposed for the cross-device bind test in the probe case.
    pub fn blit_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// The device iced gave us (cloned).
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn render_content(
        &mut self,
        kind: ViewKind,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        tick: u64,
        presses: u32,
        width: u32,
        height: u32,
    ) {
        let need = match self.targets.get(&kind) {
            Some(t) => t.size != (width, height),
            None => true,
        };
        if need {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("agp iced view texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("agp iced blit bind group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.targets.insert(
                kind,
                ViewTarget {
                    texture: tex,
                    bind_group,
                    size: (width, height),
                },
            );
        }
        let view = self
            .targets
            .get(&kind)
            .unwrap()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        match kind {
            ViewKind::View2d => self.render_2d(device, queue, &view, tick, presses),
            ViewKind::View3d => self.render_3d(device, queue, &view, width, height, tick),
        }
    }

    fn render_2d(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        tick: u64,
        presses: u32,
    ) {
        if self.quad2d.is_none() {
            self.quad2d = Some(
                QuadPipeline::new_with(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb)
                    .expect("2d"),
            );
        }
        let quad = self.quad2d.as_ref().unwrap();
        let (tw, th) = {
            let t = self.targets.get(&ViewKind::View2d).unwrap();
            t.size
        };
        quad.set_ortho_camera_with(queue, 0.0, tw as f32, th as f32, 0.0);
        let t = (tick % 120) as f32 / 120.0;
        let hue_shift = (presses % 4) as f32 * 0.22;
        let cmds = vec![
            DrawCmd {
                center: [tw as f32 * (0.2 + 0.6 * t), th as f32 * 0.45],
                size: [70.0, 44.0],
                z: 0.0,
                color: Color::rgb(0.9 - hue_shift, 0.35 + hue_shift, 0.3),
                shape: Shape::Rect,
            },
            DrawCmd {
                center: [tw as f32 * 0.78, th as f32 * (0.3 + 0.4 * t)],
                size: [52.0, 52.0],
                z: 0.0,
                color: Color::rgb(0.2, 0.6, 0.9 - hue_shift),
                shape: Shape::Circle,
            },
            DrawCmd {
                center: [tw as f32 * 0.32, th as f32 * 0.82],
                size: [90.0, 34.0],
                z: 0.0,
                color: Color::rgba(0.3, 0.8, 0.4, 0.85),
                shape: Shape::Rect,
            },
        ];
        quad.render_with(
            device,
            queue,
            view,
            Some(Color::rgb(0.08, 0.09, 0.12)),
            &cmds,
        );
    }

    fn render_3d(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        tick: u64,
    ) {
        if self.mesh3d.is_none() {
            let mut store = MeshStore::default();
            let mut keys = Vec::new();
            let ground = synth(crate::render3d::procedural::ground_mesh());
            let gk = (ground.source_sha256.clone(), ground.meshes[0].name.clone());
            let _ = store.get_or_upload_with(device, queue, &ground, 0);
            keys.push(gk);
            let player = load_glb_file(&agp_file("assets/squash_the_creeps/art/player.glb"))
                .expect("player.glb");
            for i in 0..player.meshes.len() {
                let (name, _) = store
                    .get_or_upload_with(device, queue, &player, i)
                    .expect("upload");
                keys.push((player.source_sha256.clone(), name));
            }
            let mob =
                load_glb_file(&agp_file("assets/squash_the_creeps/art/mob.glb")).expect("mob.glb");
            for i in 0..mob.meshes.len() {
                let (name, _) = store
                    .get_or_upload_with(device, queue, &mob, i)
                    .expect("upload");
                keys.push((mob.source_sha256.clone(), name));
            }
            self.store3d = Some(store);
            self.keys3d = Some(keys);
            self.mesh3d = Some(
                MeshPipeline::new_with(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb)
                    .expect("3d"),
            );
        }
        let keys = self.keys3d.as_ref().unwrap();
        let (ground_keys, model_keys) = keys.split_at(1);
        let pipeline = self.mesh3d.as_mut().unwrap();
        let store = self.store3d.as_ref().unwrap();
        let camera = Camera3d::sample_camera(width as f32 / height.max(1) as f32);
        let bob = crate::render3d::BobAnimation::default();
        let (py, pr) = bob.eval(tick as f32 / 60.0 * 4.0);
        let player_base = glam::Mat4::from_translation(glam::Vec3::new(0.0, py - 0.79, 0.0))
            * glam::Mat4::from_rotation_x(pr);
        let mob_base = glam::Mat4::from_translation(glam::Vec3::new(4.0, 0.0, -3.0))
            * glam::Mat4::from_rotation_y(0.6);
        let mut instances: Vec<MeshInstance> = Vec::new();
        for k in ground_keys {
            instances.push(MeshInstance {
                mesh: k.clone(),
                model: glam::Mat4::IDENTITY,
            });
        }
        for (i, k) in model_keys.iter().enumerate() {
            let model = if i < 3 { player_base } else { mob_base };
            instances.push(MeshInstance {
                mesh: k.clone(),
                model,
            });
        }
        pipeline.render_with(
            device,
            queue,
            view,
            width,
            height,
            [0.64625, 0.65575, 0.67075, 1.0],
            &camera.view_proj(),
            &crate::render3d::sample_light(),
            store,
            &instances,
            true,
        );
    }

    fn blit(&self, kind: ViewKind, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        // Sabotage path (review r1 F05 negative control): pretend to draw
        // the final composite while drawing nothing — the blit counters
        // never advance, so the composite gate turns RED.
        if std::env::var("AGP_SABOTAGE_ICED").ok().as_deref() == Some("blank_blit") {
            return self.targets.get(&kind).is_some();
        }
        let Some(target) = self.targets.get(&kind) else {
            return false;
        };
        render_pass.set_pipeline(&self.blit);
        render_pass.set_bind_group(0, &target.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        // The counter is incremented ONLY next to the real draw call.
        blit_counter(kind).fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        true
    }
}

fn agp_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn synth(mesh: crate::assets::MeshData) -> crate::assets::ModelData {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    for p in &mesh.positions {
        h.update(p[0].to_ne_bytes());
        h.update(p[1].to_ne_bytes());
        h.update(p[2].to_ne_bytes());
    }
    h.update(mesh.name.as_bytes());
    crate::assets::ModelData {
        source_sha256: format!("{:x}", h.finalize()),
        meshes: vec![mesh],
    }
}

// ---------------------------------------------------------------------------
// The iced application
// ---------------------------------------------------------------------------

pub struct SpikeApp {
    phase: u32,
    keyframes: Vec<crate::host::report::Keyframe>,
    resize_pending: bool,
    finished: bool,
    resize_task_sent: bool,
    /// Required composite screenshot currently outstanding (R10-02). Phase
    /// progression must wait for the matching `Message::CompositeShot`, not
    /// for the request to have been issued.
    pending_shot: Option<String>,
    /// Moments whose screenshot request could not even be issued
    /// (window-id not ready) or whose callback never arrived.
    shot_failures: Vec<String>,
    /// Tick when the current phase was entered — bounded waits (R5-01
    /// click debt + R10-02 screenshot deadlines) diagnose instead of hang.
    /// MUST use `wall_tick`, not the freezable graphics animation clock
    /// (R14 F829-R14-01: phase 4 freezes `GRAPHICS.tick`, so `phase_age`
    /// on the animation clock stays 0 and the resized timeout never fires).
    phase_started_tick: u64,
    /// Independent event-loop tick, incremented on every `Message::Tick`
    /// regardless of the animation freeze used by the blank-frame control.
    wall_tick: u64,
}

/// Bounded waits in the canonical sequence (wall ticks at 33ms). Exceeding
/// a deadline is a diagnosed failure, not silent hang (R5-01 / R10-02 /
/// R14 F829-R14-01). Deadlines use `wall_tick` so the frozen-frame
/// negative control cannot disable them.
const SHOT_WAIT_DEADLINE_TICKS: u64 = 180; // ~6s
const CLICK_WAIT_DEADLINE_TICKS: u64 = 900; // ~30s
const WINDOW_READY_DEADLINE_TICKS: u64 = 300; // ~10s

/// Pure readiness rule for one required composite screenshot (R10-02).
/// Phase progression binds to actual `CompositeShot` completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotBarrier {
    /// The moment has been analyzed and stored.
    Complete,
    /// Request outstanding — do not advance.
    Wait,
    /// Not pending and not received: request failed or was never issued.
    Missing,
}

pub fn screenshot_barrier(
    pending: Option<&str>,
    received: &dyn Fn(&str) -> bool,
    moment: &str,
) -> ScreenshotBarrier {
    if received(moment) {
        return ScreenshotBarrier::Complete;
    }
    if pending == Some(moment) {
        ScreenshotBarrier::Wait
    } else {
        ScreenshotBarrier::Missing
    }
}

fn shot_received(moment: &str) -> bool {
    SHARED_EVIDENCE
        .lock()
        .unwrap()
        .composite_shots
        .iter()
        .any(|s| s.moment == moment)
}

/// Issue the screenshot only when the window id is ready; record a
/// diagnosed failure instead of silently returning `Task::none` while the
/// caller still advances (R10-02).
fn try_shot_task(app: &mut SpikeApp, moment: &'static str) -> Task<Message> {
    match SHARED_EVIDENCE.lock().unwrap().window_id {
        Some(id) => {
            app.pending_shot = Some(moment.to_string());
            let tag = moment.to_string();
            window::screenshot(id).map(move |s| Message::CompositeShot(tag.clone(), s))
        }
        None => {
            app.shot_failures.push(moment.to_string());
            eprintln!("iced-shared-device: screenshot {moment} blocked, window id not ready");
            Task::none()
        }
    }
}

pub fn run() -> iced::Result {
    iced::application(boot, update, view)
        .window(iced::window::Settings {
            size: Size::new(1000.0, 800.0),
            resizable: true,
            ..Default::default()
        })
        .title(|_: &SpikeApp| WINDOW_TITLE.to_string())
        .subscription(subscription)
        .run()
}

fn boot() -> (SpikeApp, Task<Message>) {
    (
        SpikeApp {
            phase: 0,
            keyframes: Vec::new(),
            resize_pending: false,
            finished: false,
            resize_task_sent: false,
            pending_shot: None,
            shot_failures: Vec::new(),
            phase_started_tick: 0,
            wall_tick: 0,
        },
        Task::none(),
    )
}

fn subscription(_: &SpikeApp) -> Subscription<Message> {
    Subscription::batch([
        iced::time::every(Duration::from_millis(33)).map(|_| Message::Tick),
        window::resize_events().map(|(id, size)| Message::Resized(id, size)),
    ])
}

/// Content analysis of one composed-window screenshot. Source-anchored
/// where possible: the 2D view clears to `Color::rgb(0.08, 0.09, 0.12)`
/// (≈ sRGB (20,23,31)) and the layout is container padding 10 + spacing 8
/// with Fixed(320) view heights (the spike's own `view()`); the container
/// background is calibrated from the capture corner. Rejects: fully
/// transparent/blank composites, missing controls, missing/misplaced
/// views, and view content bleeding outside its rect (clip).
/// Analyze composed-window pixels (r4 F829-R4-05, anchored r6 F829-R6-02).
/// Takes raw RGBA bytes so the permanent gate controls can drive the SAME
/// consumed path with constructed images (no iced dependency in tests).
pub fn analyze_composite_shot(
    moment: &str,
    rgba: &[u8],
    size_w: u32,
    size_h: u32,
    scale_factor: f32,
) -> CompositeShotFacts {
    let w = size_w.max(1);
    let h = size_h.max(1);
    let b = rgba;
    let px = |x: u32, y: u32| -> [u8; 4] {
        let i = ((y.min(h - 1) * w + x.min(w - 1)) * 4) as usize;
        [b[i], b[i + 1], b[i + 2], b[i + 3]]
    };
    let near = |c: [u8; 4], t: (u8, u8, u8), tol: u8| -> bool {
        c[0].abs_diff(t.0) <= tol && c[1].abs_diff(t.1) <= tol && c[2].abs_diff(t.2) <= tol
    };
    let bg = px(2, 2);
    let far_from_bg = |c: [u8; 4]| -> bool {
        c[0].abs_diff(bg[0]) > 24 || c[1].abs_diff(bg[1]) > 24 || c[2].abs_diff(bg[2]) > 24
    };
    // 2D view background, source-anchored (render_2d clear color).
    let v2 = (20u8, 23u8, 31u8);

    // Independently anchored expected geometry (review r6 F829-R6-02,
    // constrained layout r10 F829-R10-01): the layout comes from the
    // spike's OWN view() — container padding 10, control band height 31
    // (the iced default-theme button row), spacing 8, Fixed(320) view
    // heights — times the screenshot scale factor. When the column does
    // not fit, Iced's layout leaves the first Fixed child at its requested
    // size and squeezes the LAST child to the remaining content height
    // (real canonical resize 860x700@2: 3D target 840x313 logical =
    // physical y=[754,1380), not the nominal 1394). The observed image is
    // NEVER searched to establish where the views should be.
    let scale = scale_factor.max(1.0);
    let pad = (10.0 * scale).round() as u32;
    let gap = (8.0 * scale).round() as u32;
    let ctrl_h = (31.0 * scale).round() as u32;
    let view_nom = (320.0 * scale).round() as u32;
    let content_h = h.saturating_sub(2 * pad);
    // column: ctrl + gap + v2 + gap + v3 <= content_h
    let v2_h = view_nom.min(content_h.saturating_sub(ctrl_h + gap));
    let v3_h = view_nom.min(content_h.saturating_sub(ctrl_h + gap + v2_h + gap));
    let expected_top = pad + ctrl_h + gap;
    let v2_top = expected_top.min(h);
    let v2_bot = v2_top.saturating_add(v2_h).min(h);
    let v3_top = v2_bot.saturating_add(gap).min(h);
    let v3_bot = v3_top.saturating_add(v3_h).min(h);
    let ctrl_bot = v2_top.saturating_sub(gap).min(h);
    let fits = v2_h > 0 && v3_h > 0 && v3_bot <= h;

    // Per-view independent position (review r8 F829-R8-01): EACH view's
    // observed extent must match its own anchored rect — moving only the
    // 3D view fails here even when the 2D view is untouched.
    let pos_tol = 2.0 * scale + 2.0;
    // 2D: rows dominated by the 2D clear color.
    let row_v2_frac = |y: u32| -> f32 {
        let mut hits = 0u32;
        let mut n = 0u32;
        let mut x = 0u32;
        while x < w {
            n += 1;
            if near(px(x, y), v2, 16) {
                hits += 1;
            }
            x += 2;
        }
        hits as f32 / n.max(1) as f32
    };
    // 3D: rows containing view content — the sky signature or anything
    // clearly darker than the white window background.
    let min3 = |c: [u8; 4]| c[0].min(c[1]).min(c[2]);
    let is_v3_content = |c: [u8; 4]| near(c, (165u8, 167u8, 171u8), 20) || min3(c) < 250;
    let row_v3_frac = |y: u32| -> f32 {
        let mut hits = 0u32;
        let mut n = 0u32;
        let mut x = pad;
        while x + pad < w {
            n += 1;
            if is_v3_content(px(x, y)) {
                hits += 1;
            }
            x += 4;
        }
        hits as f32 / n.max(1) as f32
    };
    // observed extents of each view (diagnostic; the EXPECTED rect is the
    // anchor — the image is never searched to define the expectation).
    let mut v2_obs_top = h;
    let mut y = 0u32;
    while y < h {
        if row_v2_frac(y) >= 0.5 {
            v2_obs_top = y;
            break;
        }
        y += 2;
    }
    let mut v2_obs_bot = v2_obs_top;
    y = v2_obs_top;
    while y < h {
        if row_v2_frac(y) < 0.5 {
            break;
        }
        v2_obs_bot = y;
        y += 2;
    }
    // Row threshold 0.03: the real capture's 3D rows measure ~19% content
    // (sky pattern + lit ground) and the resized moment ~7%, while every
    // outside row (gap / below / control band) measures 0.0 — a low
    // threshold separates view rows from background rows with wide margin.
    let mut v3_obs_top = h;
    y = v2_bot;
    while y < h {
        if row_v3_frac(y) >= 0.03 {
            v3_obs_top = y;
            break;
        }
        y += 2;
    }
    let mut v3_obs_bot = v3_obs_top;
    y = v3_obs_top;
    while y < h {
        if row_v3_frac(y) < 0.03 {
            break;
        }
        v3_obs_bot = y;
        y += 2;
    }
    // Horizontal extents (r10 F829-R8-01): each view's content columns
    // must fill its own anchored x=[pad, w-pad) — a left/right translation
    // clipped to the original rect leaves a blank band at one edge and
    // fails here even when the row extents still match.
    let col_v2_frac = |x: u32| -> f32 {
        let mut hits = 0u32;
        let mut n = 0u32;
        let mut y = v2_top;
        while y < v2_bot {
            n += 1;
            if near(px(x, y), v2, 16) {
                hits += 1;
            }
            y += 2;
        }
        hits as f32 / n.max(1) as f32
    };
    let col_v3_frac = |x: u32| -> f32 {
        let mut hits = 0u32;
        let mut n = 0u32;
        let mut y = v3_top;
        while y < v3_bot {
            n += 1;
            if is_v3_content(px(x, y)) {
                hits += 1;
            }
            y += 2;
        }
        hits as f32 / n.max(1) as f32
    };
    let mut v2_obs_left = w;
    let mut v2_obs_right = 0u32;
    let mut x = pad;
    while x + pad < w {
        if col_v2_frac(x) >= 0.5 {
            v2_obs_left = v2_obs_left.min(x);
            v2_obs_right = v2_obs_right.max(x);
        }
        x += 2;
    }
    if v2_obs_left >= w {
        v2_obs_left = w;
        v2_obs_right = 0;
    }
    let mut v3_obs_left = w;
    let mut v3_obs_right = 0u32;
    x = pad;
    while x + pad < w {
        if col_v3_frac(x) >= 0.03 {
            v3_obs_left = v3_obs_left.min(x);
            v3_obs_right = v3_obs_right.max(x);
        }
        x += 2;
    }
    if v3_obs_left >= w {
        v3_obs_left = w;
        v3_obs_right = 0;
    }
    let x_left = pad as f32;
    let x_right = w.saturating_sub(pad + 1) as f32;
    let position_2d_ok = fits
        && (v2_obs_top as f32 - v2_top as f32).abs() <= pos_tol
        && (v2_obs_bot as f32 - (v2_bot as f32 - 1.0)).abs() <= pos_tol + 2.0
        && v2_obs_left < w
        && (v2_obs_left as f32 - x_left).abs() <= pos_tol
        && (v2_obs_right as f32 - x_right).abs() <= pos_tol + 2.0;
    let position_3d_ok = fits
        && v3_obs_top < h
        && (v3_obs_top as f32 - v3_top as f32).abs() <= pos_tol
        && (v3_obs_bot as f32 - (v3_bot as f32 - 1.0)).abs() <= pos_tol + 2.0
        && v3_obs_left < w
        && (v3_obs_left as f32 - x_left).abs() <= pos_tol
        && (v3_obs_right as f32 - x_right).abs() <= pos_tol + 2.0;
    let position_ok = position_2d_ok && position_3d_ok;

    // 1) non-blank: the composite must differ from a flat background.
    let mut distinct = 0u32;
    let mut total = 0u32;
    let mut yy = 0u32;
    while yy < h {
        let mut xx = 0u32;
        while xx < w {
            if far_from_bg(px(xx, yy)) {
                distinct += 1;
            }
            total += 1;
            xx += 4;
        }
        yy += 4;
    }
    let non_blank = total > 0 && distinct * 100 > total * 2; // > 2%

    // 2) controls present: the top band holds a solid non-background fill
    //    (the button) plus text — ≥5% of sampled band pixels differ from bg,
    //    with one dominant non-bg color ≥3% (a real filled button).
    let mut band_diff = 0u32;
    let mut band_total = 0u32;
    let mut dominant = 0u32;
    let mut counts: std::collections::HashMap<(u8, u8, u8), u32> = Default::default();
    yy = 0;
    while yy < ctrl_bot {
        let mut xx = 0u32;
        while xx < w {
            let c = px(xx, yy);
            band_total += 1;
            if far_from_bg(c) {
                band_diff += 1;
                let q = (c[0] / 16 * 16, c[1] / 16 * 16, c[2] / 16 * 16);
                let n = counts.entry(q).or_default();
                *n += 1;
                dominant = dominant.max(*n);
            }
            xx += 2;
        }
        yy += 2;
    }
    let controls_present =
        band_total > 0 && band_diff * 100 >= band_total * 5 && dominant * 100 >= band_total * 3;

    // rect sampler: fraction of sampled pixels satisfying `pred`
    let frac = |top: u32, bot: u32, pred: &dyn Fn([u8; 4]) -> bool| -> f32 {
        if bot <= top {
            return 0.0;
        }
        let mut hits = 0u32;
        let mut n = 0u32;
        let mut yy = top;
        while yy < bot {
            let mut xx = pad;
            while xx + pad < w {
                if pred(px(xx, yy)) {
                    hits += 1;
                }
                n += 1;
                xx += 4;
            }
            yy += 4;
        }
        if n == 0 {
            0.0
        } else {
            hits as f32 / n as f32
        }
    };

    // 3) 2D view present at the expected rect (source-anchored clear color).
    let f_v2 = frac(v2_top, v2_bot, &|c| near(c, v2, 16));
    let view2d_present = f_v2 >= 0.20;
    // 4) 3D view present: (a) the source-anchored sky clear color
    //    ([0.64625, 0.65575, 0.67075] — iced_spike render_3d) is visible in
    //    the rect (the horizon band survives camera-aspect changes; the
    //    resized 860x700 window measures ~6% vs initial ~19%), and (b) the
    //    rect is multi-colored (sky + lit ground + models) — a transparent
    //    or undrawn view is the FLAT single-color window background.
    let sky = (165u8, 167u8, 171u8);
    let f_sky = frac(v3_top, v3_bot, &|c| near(c, sky, 20));
    let mut qcolors: std::collections::HashSet<(u8, u8, u8)> = Default::default();
    yy = v3_top;
    while yy < v3_bot {
        let mut xx = pad;
        while xx + pad < w {
            let c = px(xx, yy);
            qcolors.insert((c[0] / 8, c[1] / 8, c[2] / 8));
            xx += 8;
        }
        yy += 8;
    }
    let view3d_present = qcolors.len() >= 3 && f_sky >= 0.03;
    // 5) clip (r6/r8/r10 signatures; r12 COMPLETE BACKGROUND COMPLEMENT):
    //    sample EVERY background band outside both view rects and the
    //    legitimate controls row. Per-band fractions keep small regions
    //    (outer control gutters, bottom corners) as sensitive as the large
    //    ones — a single whole-image dilute fraction would miss them.
    //    Control interior rejects view-content signatures only, so
    //    button/text stay legal.
    let ground = (224u8, 224u8, 224u8);
    // Real frozen-scene model sample from review-r10 geometry probe.
    let is_modelish =
        |c: [u8; 4]| c[0] > 50 && c[1] < 100 && c[2] < 100 && (c[0] as i32 - c[2] as i32) > 25;
    let is_view_sig =
        |c: [u8; 4]| near(c, v2, 16) || near(c, sky, 20) || near(c, ground, 16) || is_modelish(c);
    let frac_rect = |x0: u32, y0: u32, x1: u32, y1: u32, pred: &dyn Fn([u8; 4]) -> bool| -> f32 {
        if x1 <= x0 || y1 <= y0 {
            return 0.0;
        }
        let mut hits = 0u32;
        let mut n = 0u32;
        let mut y = y0;
        while y < y1.min(h) {
            let mut x = x0;
            while x < x1.min(w) {
                if pred(px(x, y)) {
                    hits += 1;
                }
                n += 1;
                x += 2;
            }
            y += 2;
        }
        if n == 0 {
            0.0
        } else {
            hits as f32 / n as f32
        }
    };
    let f_top = frac_rect(0, 0, w, pad, &far_from_bg);
    let f_pre2d = frac_rect(0, ctrl_bot.min(h), w, v2_top.min(h), &far_from_bg);
    let f_gap = frac_rect(0, v2_bot.min(h), w, v3_top.min(h), &far_from_bg);
    let f_below = frac_rect(0, v3_bot.min(h), w, h, &far_from_bg);
    let f_left = frac_rect(0, 0, pad.min(w), h, &far_from_bg);
    let f_right = frac_rect(w.saturating_sub(pad), 0, w, h, &far_from_bg);
    let f_ctrl = frac_rect(
        pad,
        pad,
        w.saturating_sub(pad),
        ctrl_bot.min(h),
        &is_view_sig,
    );
    let clip_ok = f_top < 0.01
        && f_pre2d < 0.01
        && f_gap < 0.01
        && f_below < 0.01
        && f_left < 0.01
        && f_right < 0.01
        && f_ctrl < 0.01;

    let detail = format!(
        "bg=({},{},{}) v2_obs=({v2_obs_left}..{v2_obs_right},{v2_obs_top}..{v2_obs_bot}) v3_obs=({v3_obs_left}..{v3_obs_right},{v3_obs_top}..{v3_obs_bot}) v2_h={v2_h} v3_h={v3_h} distinct={distinct}/{total} band={band_diff}/{band_total} dominant={dominant} f_v2={f_v2:.3} f_sky={f_sky:.3} v3_colors={} f_top={f_top:.3} f_pre2d={f_pre2d:.3} f_gap={f_gap:.3} f_below={f_below:.3} f_left={f_left:.3} f_right={f_right:.3} f_ctrl={f_ctrl:.3}",
        bg[0], bg[1], bg[2],
        qcolors.len()
    );
    CompositeShotFacts {
        moment: moment.to_string(),
        physical_w: w,
        physical_h: h,
        scale_factor,
        non_blank,
        controls_present,
        view2d_present,
        view3d_present,
        position_ok,
        position_2d_ok,
        position_3d_ok,
        clip_ok,
        detail,
    }
}

fn update(app: &mut SpikeApp, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            // Animation clock may freeze (blank-frame control); the wall
            // clock for bounded waits never does (R14 F829-R14-01).
            app.wall_tick = app.wall_tick.saturating_add(1);
            {
                let mut g = GRAPHICS.lock().unwrap();
                if !g.frozen {
                    g.tick += 1;
                }
            }
            let wall = app.wall_tick;
            match mode() {
                SpikeMode::Live => Task::none(),
                SpikeMode::Canonical { report_dir } => canonical_step(app, wall, &report_dir),
            }
        }
        Message::ButtonPressed => {
            CLICK_OBSERVED.store(true, std::sync::atomic::Ordering::SeqCst);
            let mut ev = SHARED_EVIDENCE.lock().unwrap();
            ev.presses += 1;
            ev.click_observed = true;
            if ev.click_path.is_empty() {
                ev.click_path = "os_focus".into();
            }
            Task::none()
        }
        Message::Resized(id, size) => {
            {
                let mut ev = SHARED_EVIDENCE.lock().unwrap();
                if ev.window_id.is_none() {
                    ev.window_id = Some(id);
                }
                ev.window_size = Some(size);
                ev.resized_sizes.push(size);
            }
            if app.resize_pending {
                app.resize_pending = false;
                app.phase = 4; // capture on the next tick (views re-rendered)
                app.phase_started_tick = app.wall_tick;
            }
            Task::none()
        }
        Message::CompositeShot(moment, shot) => {
            let facts = analyze_composite_shot(
                &moment,
                &shot.rgba,
                shot.size.width,
                shot.size.height,
                shot.scale_factor,
            );
            if let SpikeMode::Canonical { report_dir } = mode() {
                let _ = std::fs::create_dir_all(&report_dir);
                let _ = image::save_buffer(
                    report_dir.join(format!("{moment}_composite.png")),
                    &shot.rgba,
                    shot.size.width,
                    shot.size.height,
                    image::ColorType::Rgba8,
                );
            }
            SHARED_EVIDENCE.lock().unwrap().composite_shots.push(facts);
            // R10-02: completing the callback is what unblocks the phase,
            // not the screenshot request itself.
            if app.pending_shot.as_deref() == Some(moment.as_str()) {
                app.pending_shot = None;
            }
            Task::none()
        }
    }
}

fn view(_: &SpikeApp) -> Element<'_, Message> {
    let presses = SHARED_EVIDENCE.lock().unwrap().presses;
    let controls = row![
        button(text(format!("Step graphics ({presses})"))).on_press(Message::ButtonPressed),
        text(format!("presses: {presses} | iced seam probe")),
    ]
    .spacing(12);
    let view2d = shader(GraphicsProgram {
        kind: ViewKind::View2d,
    })
    .width(Length::Fill)
    .height(Length::Fixed(320.0));
    let view3d = shader(GraphicsProgram {
        kind: ViewKind::View3d,
    })
    .width(Length::Fill)
    .height(Length::Fixed(320.0));
    container(column![controls, view2d, view3d].spacing(8))
        .padding(10)
        .into()
}

// ---------------------------------------------------------------------------
// Canonical scripted sequence (probe case)

fn record_blit_counts(moment: &str) {
    let counts = [
        BLIT_DRAWS[0].load(std::sync::atomic::Ordering::SeqCst),
        BLIT_DRAWS[1].load(std::sync::atomic::Ordering::SeqCst),
    ];
    SHARED_EVIDENCE
        .lock()
        .unwrap()
        .blit_counts
        .push((moment.to_string(), counts));
}

fn canonical_step(app: &mut SpikeApp, wall: u64, report_dir: &PathBuf) -> Task<Message> {
    // Phase order constraint (r4 F829-R4-05): the in-compositor screenshot
    // executes its GPU readback synchronously inside the event loop, which
    // pauses message processing for the duration. Click dispatch therefore
    // happens in a LATER phase than any screenshot, so the input fallbacks'
    // retry window never races a blocked loop.
    //
    // R10-02: every required composite moment is BOUND to actual
    // `CompositeShot` completion (or a diagnosed failure) before the next
    // phase runs. Requesting a screenshot is not completion.
    // R5-01 / R12 / R14: waits are BOUNDED on the WALL tick (independent
    // of the freezable animation clock) — a deadline is a diagnosed
    // failure, never an unbounded hang, including the frozen resized phase.
    if app.phase_started_tick == 0 {
        app.phase_started_tick = wall;
    }
    let phase_age = wall.saturating_sub(app.phase_started_tick);
    let enter = |app: &mut SpikeApp, phase: u32| {
        app.phase = phase;
        app.phase_started_tick = wall;
    };
    let received = |m: &str| shot_received(m);
    match app.phase {
        0 => {
            if wall < 20 {
                return Task::none();
            }
            // window-id readiness: bounded (R14) — diagnose instead of
            // returning forever when no window ever arrives.
            if SHARED_EVIDENCE.lock().unwrap().window_id.is_none() {
                if phase_age > WINDOW_READY_DEADLINE_TICKS {
                    app.shot_failures.push("window-id-timeout".into());
                    eprintln!(
                        "iced-shared-device: window id not ready after {phase_age} wall ticks"
                    );
                    enter(app, 99);
                }
                return Task::none();
            }
            capture_both("initial", app, Some(report_dir.clone()));
            record_blit_counts("initial");
            enter(app, 1);
            try_shot_task(app, "initial")
        }
        1 => {
            match screenshot_barrier(app.pending_shot.as_deref(), &received, "initial") {
                ScreenshotBarrier::Wait => {
                    if phase_age > SHOT_WAIT_DEADLINE_TICKS {
                        app.shot_failures.push("initial-timeout".into());
                        eprintln!(
                            "iced-shared-device: initial composite screenshot timed out after {phase_age} ticks"
                        );
                        enter(app, 99);
                    }
                    return Task::none();
                }
                ScreenshotBarrier::Missing => {
                    app.shot_failures.push("initial".into());
                    eprintln!("iced-shared-device: initial composite screenshot not completed");
                    enter(app, 99);
                    return Task::none();
                }
                ScreenshotBarrier::Complete => {}
            }
            // the initial screenshot is stored: dispatch the click.
            {
                let mut ev = SHARED_EVIDENCE.lock().unwrap();
                ev.click_dispatched = true;
            }
            spawn_real_click();
            spawn_fallback_queue_click();
            enter(app, 2);
            Task::none()
        }
        2 if SHARED_EVIDENCE.lock().unwrap().click_observed => {
            capture_both("after_button", app, Some(report_dir.clone()));
            record_blit_counts("after_button");
            enter(app, 3);
            try_shot_task(app, "after_button")
        }
        2 => {
            // R5-01 bounded click wait: re-dispatch while waiting; diagnose
            // on deadline instead of hanging the case forever.
            if phase_age > CLICK_WAIT_DEADLINE_TICKS {
                app.shot_failures.push("click-timeout".into());
                eprintln!(
                    "iced-shared-device: click not observed after {phase_age} ticks (bounded wait)"
                );
                enter(app, 99);
                return Task::none();
            }
            if phase_age % 90 == 0 {
                spawn_real_click();
                spawn_fallback_queue_click();
            }
            Task::none()
        }
        3 => {
            match screenshot_barrier(app.pending_shot.as_deref(), &received, "after_button") {
                ScreenshotBarrier::Wait => {
                    if phase_age > SHOT_WAIT_DEADLINE_TICKS {
                        app.shot_failures.push("after_button-timeout".into());
                        eprintln!(
                            "iced-shared-device: after_button composite screenshot timed out after {phase_age} ticks"
                        );
                        enter(app, 99);
                    }
                    return Task::none();
                }
                ScreenshotBarrier::Missing => {
                    app.shot_failures.push("after_button".into());
                    eprintln!(
                        "iced-shared-device: after_button composite screenshot not completed"
                    );
                    enter(app, 99);
                    return Task::none();
                }
                ScreenshotBarrier::Complete => {}
            }
            if !app.resize_task_sent {
                app.resize_task_sent = true;
                app.resize_pending = true;
                return match SHARED_EVIDENCE.lock().unwrap().window_id {
                    Some(id) => window::resize(id, Size::new(860.0, 700.0)),
                    None => {
                        app.shot_failures.push("resize-window".into());
                        enter(app, 99);
                        Task::none()
                    }
                };
            }
            if phase_age > CLICK_WAIT_DEADLINE_TICKS {
                app.shot_failures.push("resize-timeout".into());
                eprintln!("iced-shared-device: resize event not applied in time");
                enter(app, 99);
            }
            Task::none()
        }
        4 => {
            capture_both("resized", app, Some(report_dir.clone()));
            record_blit_counts("resized");
            GRAPHICS.lock().unwrap().frozen = true; // freeze for the controls
            enter(app, 5);
            try_shot_task(app, "resized")
        }
        5 => {
            match screenshot_barrier(app.pending_shot.as_deref(), &received, "resized") {
                ScreenshotBarrier::Wait => {
                    if phase_age > SHOT_WAIT_DEADLINE_TICKS {
                        app.shot_failures.push("resized-timeout".into());
                        eprintln!(
                            "iced-shared-device: resized composite screenshot timed out after {phase_age} ticks"
                        );
                        enter(app, 99);
                    }
                    return Task::none();
                }
                ScreenshotBarrier::Missing => {
                    app.shot_failures.push("resized".into());
                    eprintln!("iced-shared-device: resized composite screenshot not completed");
                    enter(app, 99);
                    return Task::none();
                }
                ScreenshotBarrier::Complete => {}
            }
            // blank/undriven negative control: two back-to-back captures of
            // the SAME prepared frozen frame — they must hash identical, and
            // the verifier gate (expecting driven progression) must fire.
            capture_both("blank_control", app, Some(report_dir.clone()));
            capture_both("blank_control_repeat", app, Some(report_dir.clone()));
            enter(app, 6);
            Task::none()
        }
        6 => {
            capture_both("pre_close", app, Some(report_dir.clone()));
            enter(app, 7);
            Task::none()
        }
        7 | 99 => {
            if !app.finished {
                app.finished = true;
                if !app.shot_failures.is_empty() {
                    eprintln!(
                        "iced-shared-device: screenshot failures {:?}",
                        app.shot_failures
                    );
                }
                let verdict = crate::host::cases::iced_shared_device::finalize(
                    report_dir.clone(),
                    std::mem::take(&mut app.keyframes),
                );
                if verdict != 0 {
                    eprintln!("iced-shared-device: verdict {verdict} (see report)");
                }
                // Normal host close: close the window (ends the event loop
                // when the last window closes) and let window/GPU/static
                // destruction complete before the case returns (r1 F06).
                // Missing window-id (window-id-timeout path): `window::close`
                // cannot run — schedule `iced::exit()` so the runtime returns
                // instead of leaving the loop subscribed and presenting into
                // a dying surface (R16 F829-R14-01, native exit 101).
                return match SHARED_EVIDENCE.lock().unwrap().window_id {
                    Some(id) => window::close(id),
                    None => iced::exit(),
                };
            }
            Task::none()
        }
        _ => Task::none(),
    }
}

fn spawn_fallback_queue_click() {
    // If the real OS click did not register (session focus restrictions),
    // deliver the press through the window's own message queue — winit's
    // real input pipeline, synthetic delivery. Retries until observed: the
    // agent session's foreground race makes a single attempt flaky.
    std::thread::spawn(|| {
        let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Poster {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, UIntPtr w, IntPtr l);
    public static bool Post(string title, int x, int y) {
        IntPtr h = FindWindow(null, title);
        if (h == IntPtr.Zero) return false;
        IntPtr l = (IntPtr)((y << 16) | (x & 0xffff));
        PostMessage(h, 0x0200, UIntPtr.Zero, l);
        System.Threading.Thread.Sleep(30);
        PostMessage(h, 0x0201, (UIntPtr)1, l);
        System.Threading.Thread.Sleep(30);
        PostMessage(h, 0x0202, (UIntPtr)0, l);
        return true;
    }
}
"@
if ([Poster]::Post("AGP iced-shared-device spike", 80, 40)) { exit 0 } else { exit 1 }
"#;
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(1000));
            if CLICK_OBSERVED.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }
            let ok = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", script])
                .output()
                .map(|o| o.status.code().unwrap_or(1) == 0)
                .unwrap_or(false);
            if ok {
                let mut ev = SHARED_EVIDENCE.lock().unwrap();
                if !ev.click_observed && ev.click_path.is_empty() {
                    ev.click_path = "posted_queue".into();
                }
            }
        }
    });
}

fn spawn_real_click() {
    // Real OS click on the button: find our window, click at the button's
    // screen position (padding 10 + button row). Runs async; the ButtonPressed
    // message arrives through the real iced event path.
    let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Clicker {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr hWnd, ref POINT lpPoint);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, UIntPtr dwExtraInfo);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr hWnd, int nCmdShow);
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    public struct POINT { public int X; public int Y; }
    public static void Click(string title, int clientX, int clientY) {
        IntPtr h = FindWindow(null, title);
        if (h == IntPtr.Zero) { Console.Error.WriteLine("window not found"); return; }
        // CLIENT-area coordinates: immune to title-bar/border sizes and
        // window position (review r1 F06 hardening of the input evidence).
        RECT cr; GetClientRect(h, out cr);
        POINT p = new POINT { X = System.Math.Min(clientX, cr.Right - 2), Y = System.Math.Min(clientY, cr.Bottom - 2) };
        ClientToScreen(h, ref p);
        // mouse_event goes to the FOREGROUND window: raise ours first.
        ShowWindowAsync(h, 9); // SW_RESTORE
        SetForegroundWindow(h);
        System.Threading.Thread.Sleep(120);
        SetCursorPos(p.X, p.Y);
        System.Threading.Thread.Sleep(80);
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
        System.Threading.Thread.Sleep(40);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
    }
}
"@
[Clicker]::Click("AGP iced-shared-device spike", 80, 40)
"#;
    std::thread::spawn(move || {
        for _ in 0..10 {
            if CLICK_OBSERVED.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", script])
                .output();
            std::thread::sleep(Duration::from_millis(1500));
        }
    });
}

fn capture_both(name: &str, app: &mut SpikeApp, report_dir: Option<PathBuf>) {
    for (kind, suffix) in [(ViewKind::View2d, "2d"), (ViewKind::View3d, "3d")] {
        if let Some((hash, png, w, h)) = capture_view(kind) {
            if let Some(dir) = report_dir.as_ref() {
                let _ = std::fs::create_dir_all(dir);
                let file = format!("{name}_{suffix}.png");
                let _ = std::fs::write(dir.join(&file), &png);
                app.keyframes.push(crate::host::report::Keyframe {
                    name: format!("{name}_{suffix}"),
                    file,
                    width: w,
                    height: h,
                    sha256: hash,
                });
            }
        }
    }
}

#[cfg(test)]
mod r14_timeout_tests {
    use super::*;

    /// R16 F829-R14-01 holdout: missing window-id finalizer must schedule
    /// a real exit action (`iced::exit`), not `Task::none` — otherwise the
    /// event loop stays subscribed and the native path panics in present.
    #[test]
    fn missing_window_id_finalizer_schedules_exit() {
        let dir = std::env::temp_dir().join(format!("agp-r16-missing-id-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        set_mode(SpikeMode::Canonical {
            report_dir: dir.clone(),
        });
        *SHARED_EVIDENCE.lock().unwrap() = SpikeEvidence::default();
        *GRAPHICS.lock().unwrap() = SpikeGraphics::default();
        let (mut app, _) = boot();
        let mut _delivered = 0u32;
        for _ in 0..400 {
            let _ = update(&mut app, Message::Tick);
            _delivered += 1;
            if app.phase == 99 {
                break;
            }
        }
        assert_eq!(app.phase, 99, "window-id-timeout must reach phase 99");
        assert_eq!(app.shot_failures, vec!["window-id-timeout".to_string()]);
        let terminal = update(&mut app, Message::Tick);
        assert!(app.finished, "finalizer must mark finished");
        assert!(
            terminal.units() >= 1,
            "missing-ID finalizer must schedule an exit task, got units={}",
            terminal.units()
        );
        // known-ID sibling still closes the window (one effect).
        *SHARED_EVIDENCE.lock().unwrap() = SpikeEvidence::default();
        let (mut known, _) = boot();
        known.phase = 99;
        SHARED_EVIDENCE.lock().unwrap().window_id = Some(window::Id::unique());
        let close = update(&mut known, Message::Tick);
        assert_eq!(close.units(), 1, "known-ID finalizer still returns close");
    }
}
