//! Whole-path surface probe (PLAN-830 T-02 / V-02).
//!
//! Proves, on ONE wgpu device and ONE window, the full chain:
//!
//!   real iced UI (stock `button` + `text_input`)
//!     → `iced_wgpu::Renderer::present` into an OFFSCREEN sampled texture
//!     → perspective 3D plane (`SurfacePlanePipeline`) in a real 3D scene
//!     → offscreen composite → same window
//!     → input return path: window mouse/key → ray → plane UV → UI-local
//!       coords → real `UserInterface::update` (button presses + typed text
//!       observed on the REAL widgets).
//!
//! Everything uses PUBLIC iced APIs (Engine/Renderer/UserInterface/present);
//! no `iced::application`, no production UI code, no second device. The
//! window is a plain winit window owned by this host.

use crate::host::iced_surface::{IcedSurface, SurfaceElement, SurfaceUi};
use crate::host::report::Keyframe;
use crate::renderer::capture::{capture_png, sha256_hex};
use crate::renderer::composition::{FullScreenBlit, SurfacePlanePipeline};
use crate::renderer::target::{OffscreenTarget, WindowTarget};
use crate::renderer::GpuDevice;
use glam::{Mat4, Vec2, Vec3, Vec4};
use iced::widget::{button, column, text, text_input};
use iced_wgpu::core::keyboard::{self, Key, Location};
use iced_wgpu::core::mouse;
use iced_wgpu::core::widget::Operation;
use iced_wgpu::core::{Event, Point, Rectangle, Size};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const WINDOW_TITLE: &str = "AGP surface-offscreen probe";

/// Observed inner-UI state, mirrored by the host each tick so OS input
/// injection threads can poll success without borrowing the probe.
pub static PRESS_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static TEXT_LEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub const INNER_LOGICAL: (f32, f32) = (480.0, 360.0);
/// Expected typed text in the canonical run (`ok8`).
pub const TYPED_TEXT: &str = "ok8";

// ---------------------------------------------------------------------------
// The inner UI: real stock widgets, driven like a headless application.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum InnerMessage {
    Pressed,
    InputChanged(String),
}

#[derive(Default)]
pub struct InnerUi {
    pub presses: u32,
    pub input: String,
    pub placeholder: String,
}

impl SurfaceUi for InnerUi {
    type Message = InnerMessage;

    fn view(&self) -> SurfaceElement<'_, Self::Message> {
        column![
            button(text(format!("count: {}", self.presses)))
                .width(iced::Length::Fill)
                .on_press(InnerMessage::Pressed),
            text_input("type here", &self.input).on_input(InnerMessage::InputChanged),
            text(format!("echo: {}", self.input)),
        ]
        .spacing(12)
        .padding(20)
        .width(iced::Length::Fixed(INNER_LOGICAL.0))
        .into()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            InnerMessage::Pressed => self.presses += 1,
            InnerMessage::InputChanged(s) => self.input = s,
        }
    }

    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.93, 0.94, 0.97)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Exact widget bounds harvested through the public `widget::Operation`
/// traversal (never guessed from layout constants). Buttons report via
/// `focusable`; the button's inner label additionally reports via `text`
/// (starting with "count:") as a structural fallback.
#[derive(Default)]
pub struct BoundsProbe {
    pub button: Option<Rectangle>,
    pub text_input: Option<Rectangle>,
    button_text: Option<Rectangle>,
}

impl BoundsProbe {
    /// The button's exact bounds. iced 0.14 `button` does not report via
    /// `focusable` (the first focusable in this UI is the text_input), so
    /// the button's own label text ("count: …") is the PRIMARY anchor and
    /// any focusable hit is only a fallback.
    pub fn button_bounds(&self) -> Option<Rectangle> {
        self.button_text.or(self.button)
    }
}

impl Operation<()> for BoundsProbe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self)
    }

    fn focusable(
        &mut self,
        _id: Option<&iced_wgpu::core::widget::Id>,
        bounds: Rectangle,
        _state: &mut dyn iced_wgpu::core::widget::operation::Focusable,
    ) {
        if self.button.is_none() {
            self.button = Some(bounds);
        }
    }

    fn text_input(
        &mut self,
        _id: Option<&iced_wgpu::core::widget::Id>,
        bounds: Rectangle,
        _state: &mut dyn iced_wgpu::core::widget::operation::TextInput,
    ) {
        self.text_input = Some(bounds);
    }

    fn text(&mut self, _id: Option<&iced_wgpu::core::widget::Id>, bounds: Rectangle, t: &str) {
        if t.starts_with("count:") && self.button_text.is_none() {
            self.button_text = Some(bounds);
        }
    }
}

// ---------------------------------------------------------------------------
// Plane geometry + mapping math (window px ⇄ plane world ⇄ UI local).
// T-05 moves the general contract into `core::composition::input`; this is
// the probe's bounded, single-plane instance of the same math.
// ---------------------------------------------------------------------------

pub struct PlaneXform {
    pub center: Vec3,
    pub half: Vec2,
}

impl PlaneXform {
    pub const fn sample() -> Self {
        PlaneXform {
            center: Vec3::new(0.0, 1.7, 0.0),
            half: Vec2::new(2.4, 1.8),
        }
    }

    /// UI-local point (logical px, origin top-left) → world position.
    pub fn ui_to_world(&self, p: Point, logical: Size) -> Vec3 {
        let u = p.x / logical.width.max(1.0);
        let v = p.y / logical.height.max(1.0);
        self.uv_to_world([u, v])
    }

    pub fn uv_to_world(&self, uv: [f32; 2]) -> Vec3 {
        let x = (uv[0] * 2.0 - 1.0) * self.half.x;
        let y = (1.0 - uv[1] * 2.0) * self.half.y;
        self.center + Vec3::new(x, y, 0.0)
    }

    /// World hit → UV (None outside the plane rectangle).
    pub fn world_to_uv(&self, hit: Vec3) -> Option<[f32; 2]> {
        let d = hit - self.center;
        let u = (d.x / (2.0 * self.half.x)) + 0.5;
        let v = 0.5 - (d.y / (2.0 * self.half.y));
        if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
            Some([u, v])
        } else {
            None
        }
    }

    pub fn uv_to_local(&self, uv: [f32; 2], logical: Size) -> Point {
        Point::new(uv[0] * logical.width, uv[1] * logical.height)
    }
}

/// Perspective camera for the probe scene (Camera3d's own look-at and
/// perspective constructors — the same math the frozen 3D renderer uses).
pub fn probe_camera(aspect: f32) -> Mat4 {
    let camera = crate::render3d::Camera3d {
        eye: Vec3::new(0.0, 1.8, 7.0),
        target: Vec3::new(0.0, 1.7, 0.0),
        up: Vec3::Y,
        projection: crate::render3d::Projection::Perspective {
            fov_y_deg: 45.0,
            aspect,
            near: 0.1,
            far: 100.0,
        },
    };
    camera.projection_matrix() * camera.view()
}

/// Window physical pixel → world-space ray hit against the plane (front
/// hit only; the probe scene has no occluders in front of the plane).
pub fn window_pixel_to_plane(
    px: (f32, f32),
    view_proj: Mat4,
    plane: &PlaneXform,
    viewport: (u32, u32),
) -> Option<[f32; 2]> {
    let (w, h) = (viewport.0 as f32, viewport.1 as f32);
    let ndc_x = 2.0 * px.0 / w.max(1.0) - 1.0;
    let ndc_y = 1.0 - 2.0 * px.1 / h.max(1.0);
    let inv = view_proj.inverse();
    let unproject = |z: f32| -> Vec3 {
        let v = inv * Vec4::new(ndc_x, ndc_y, z, 1.0);
        Vec3::new(v.x / v.w, v.y / v.w, v.z / v.w)
    };
    let near = unproject(-1.0);
    let far = unproject(1.0);
    let dir = (far - near).normalize();
    let n = Vec3::Z;
    let denom = dir.dot(n);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (plane.center - near).dot(n) / denom;
    if t <= 0.0 {
        return None;
    }
    plane.world_to_uv(near + dir * t)
}

/// UI-local point → window physical pixel (forward projection; used to
/// place REAL OS clicks on the widgets).
pub fn ui_point_to_window_pixel(
    p: Point,
    logical: Size,
    view_proj: Mat4,
    plane: &PlaneXform,
    viewport: (u32, u32),
) -> (f32, f32) {
    let world = plane.ui_to_world(p, logical);
    let clip = view_proj * Vec4::new(world.x, world.y, world.z, 1.0);
    let ndc_x = clip.x / clip.w;
    let ndc_y = clip.y / clip.w;
    let (w, h) = (viewport.0 as f32, viewport.1 as f32);
    ((ndc_x + 1.0) * 0.5 * w, (1.0 - ndc_y) * 0.5 * h)
}

/// winit key event -> iced keyboard event (common keys; IME is T-06).
/// `physical_key` is recorded as unidentified (widgets key off `key`;
/// physical identity is not consumed on this path).
pub fn winit_key_to_iced(event: &winit::event::KeyEvent) -> Option<keyboard::Event> {
    use iced_wgpu::core::keyboard::key::{Named, NativeCode, Physical};
    use iced_wgpu::core::SmolStr;
    use winit::keyboard::Key as WKey;
    use winit::keyboard::NamedKey as WNamed;
    let key = match &event.logical_key {
        WKey::Character(ref s) => Key::Character(s.clone()),
        WKey::Named(WNamed::Enter) => Key::Named(Named::Enter),
        WKey::Named(WNamed::Backspace) => Key::Named(Named::Backspace),
        WKey::Named(WNamed::Delete) => Key::Named(Named::Delete),
        WKey::Named(WNamed::Escape) => Key::Named(Named::Escape),
        WKey::Named(WNamed::Tab) => Key::Named(Named::Tab),
        WKey::Named(WNamed::Space) => Key::Named(Named::Space),
        WKey::Named(WNamed::ArrowLeft) => Key::Named(Named::ArrowLeft),
        WKey::Named(WNamed::ArrowRight) => Key::Named(Named::ArrowRight),
        WKey::Named(WNamed::ArrowUp) => Key::Named(Named::ArrowUp),
        WKey::Named(WNamed::ArrowDown) => Key::Named(Named::ArrowDown),
        WKey::Named(WNamed::Home) => Key::Named(Named::Home),
        WKey::Named(WNamed::End) => Key::Named(Named::End),
        _ => return None,
    };
    let location = match event.location {
        winit::keyboard::KeyLocation::Standard => Location::Standard,
        winit::keyboard::KeyLocation::Left => Location::Left,
        winit::keyboard::KeyLocation::Right => Location::Right,
        winit::keyboard::KeyLocation::Numpad => Location::Numpad,
    };
    let text: Option<SmolStr> = event.text.as_deref().map(SmolStr::new);
    Some(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location,
        modifiers: keyboard::Modifiers::default(),
        text,
        repeat: event.repeat,
    })
}

/// Synthetic character key event for headless fallback (labeled as such in
/// the report; never claimed as OS input).
pub fn synthetic_char_event(ch: char) -> keyboard::Event {
    use iced_wgpu::core::keyboard::key::{NativeCode, Physical};
    use iced_wgpu::core::SmolStr;
    let key = Key::Character(SmolStr::new(&ch.to_string()));
    keyboard::Event::KeyPressed {
        modified_key: key.clone(),
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: Location::Standard,
        modifiers: keyboard::Modifiers::default(),
        text: Some(SmolStr::new(&ch.to_string())),
        repeat: false,
        key,
    }
}

// ---------------------------------------------------------------------------
// Evidence
// ---------------------------------------------------------------------------

#[derive(Default, Debug, Clone)]
pub struct SurfaceProbeEvidence {
    pub window_created: bool,
    pub window_mode: String,
    pub inner_ui_pressed: u32,
    pub inner_input_text: String,
    pub clicks_mapped_to_plane: u32,
    pub keyboard_events_delivered: u32,
    pub plane_passes_drawn: u64,
    pub ui_frames_presented: u64,
    pub readbacks: u64,
    pub keyframes: Vec<Keyframe>,
    pub gates: Vec<(String, bool, String)>,
    pub notes: Vec<String>,
    /// Layout bounds harvested via `Operation` (exact, not guessed).
    pub button_bounds: Option<[f32; 4]>,
    pub input_bounds: Option<[f32; 4]>,
    pub device_token: String,
}

// ---------------------------------------------------------------------------
// The probe application (winit host)
// ---------------------------------------------------------------------------

pub struct SurfaceProbe {
    pub replay: String,
    pub report_dir: PathBuf,
    pub evidence: SurfaceProbeEvidence,
    pub gpu: Option<GpuDevice>,
    pub window: Option<Arc<winit::window::Window>>,
    pub window_target: Option<WindowTarget>,
    pub inner: Option<IcedSurface>,
    pub ui: InnerUi,
    pub mesh3d: Option<crate::render3d::MeshPipeline>,
    pub store3d: Option<crate::render3d::MeshStore>,
    pub instances: Vec<crate::render3d::MeshInstance>,
    pub plane: Option<SurfacePlanePipeline>,
    pub blit: Option<FullScreenBlit>,
    pub composite: Option<OffscreenTarget>,
    pub pending_events: Vec<Event>,
    pub cursor_local: Option<Point>,
    pub plane_xform: PlaneXform,
    pub view_proj: Mat4,
    pub phase: u32,
    pub phase_started: Instant,
    pub wall_tick: u64,
    pub finished: bool,
    pub exit_code: i32,
    pub start: Instant,
    pub failures: Vec<String>,
}

impl SurfaceProbe {
    pub fn new(replay: String, report_dir: PathBuf) -> Self {
        SurfaceProbe {
            replay,
            report_dir,
            evidence: SurfaceProbeEvidence::default(),
            gpu: None,
            window: None,
            window_target: None,
            inner: None,
            ui: InnerUi {
                presses: 0,
                input: String::new(),
                placeholder: "type here".into(),
            },
            mesh3d: None,
            store3d: None,
            instances: Vec::new(),
            plane: None,
            blit: None,
            composite: None,
            pending_events: Vec::new(),
            cursor_local: None,
            plane_xform: PlaneXform::sample(),
            view_proj: Mat4::IDENTITY,
            phase: 0,
            phase_started: Instant::now(),
            wall_tick: 0,
            finished: false,
            exit_code: 3,
            start: Instant::now(),
            failures: Vec::new(),
        }
    }

    fn viewport_size(&self) -> (u32, u32) {
        self.composite
            .as_ref()
            .map(|c| (c.width, c.height))
            .or_else(|| self.window_target.as_ref().map(|t| (t.width, t.height)))
            .unwrap_or((800, 600))
    }

    /// Warm up GPU resources (called on resume with a window, or headless).
    pub fn warmup(&mut self, has_window: bool) -> Result<(), String> {
        let gpu = GpuDevice::new().map_err(|e| format!("gpu: {e}"))?;
        self.evidence.device_token = gpu.token().to_report();
        let size = if has_window {
            let w = self.window.as_ref().unwrap();
            let s = w.inner_size();
            (s.width, s.height)
        } else {
            (800, 600)
        };
        let aspect = size.0 as f32 / size.1.max(1) as f32;
        self.view_proj = probe_camera(aspect);
        let composite = OffscreenTarget::new_sampled(&gpu, size.0, size.1)
            .map_err(|e| format!("composite: {e}"))?;
        let plane = SurfacePlanePipeline::new(gpu.device(), OffscreenTarget::FORMAT)
            .map_err(|e| format!("plane: {e}"))?;
        let blit = FullScreenBlit::new(
            gpu.device(),
            self.window_target
                .as_ref()
                .map(|t| t.format())
                .unwrap_or(OffscreenTarget::FORMAT),
        );
        // Real 3D content behind the plane (frozen squash scene pieces).
        let mut store = crate::render3d::MeshStore::default();
        let mut instances = Vec::new();
        let synth = |mesh: crate::assets::MeshData| -> crate::assets::ModelData {
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
        };
        let ground = synth(crate::render3d::procedural::ground_mesh());
        let _ = store.get_or_upload(&gpu, &ground, 0);
        instances.push(crate::render3d::MeshInstance {
            mesh: (ground.source_sha256.clone(), ground.meshes[0].name.clone()),
            model: Mat4::IDENTITY,
        });
        for rel in ["assets/squash_the_creeps/art/player.glb"] {
            let model = crate::assets::load_glb_file(
                &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel),
            )
            .map_err(|e| format!("glb {rel}: {e:?}"))?;
            for i in 0..model.meshes.len() {
                let (name, _) = store
                    .get_or_upload(&gpu, &model, i)
                    .map_err(|e| format!("upload {rel}: {e:?}"))?;
                instances.push(crate::render3d::MeshInstance {
                    mesh: (model.source_sha256.clone(), name),
                    model: Mat4::from_translation(Vec3::new(-3.0, -0.75, -2.0)),
                });
            }
        }
        let mesh3d = crate::render3d::MeshPipeline::new(&gpu, OffscreenTarget::FORMAT)
            .map_err(|e| format!("mesh3d: {e}"))?;
        let inner = IcedSurface::new(&gpu, Size::new(INNER_LOGICAL.0, INNER_LOGICAL.1), 1.0)
            .map_err(|e| format!("iced surface: {e}"))?;
        self.gpu = Some(gpu);
        self.mesh3d = Some(mesh3d);
        self.store3d = Some(store);
        self.instances = instances;
        self.plane = Some(plane);
        self.blit = Some(blit);
        self.composite = Some(composite);
        self.inner = Some(inner);
        Ok(())
    }

    /// Exact widget bounds of the inner UI (public Operation traversal).
    pub fn probe_bounds(&mut self) -> BoundsProbe {
        let Some(inner) = self.inner.as_mut() else {
            return BoundsProbe::default();
        };
        let mut op = BoundsProbe::default();
        inner.operate(&self.ui, &mut op);
        op
    }
    /// Render one frame: inner UI offscreen → 3D + plane composite → blit
    /// to window (when present). Returns the widget messages produced.
    pub fn render_frame(&mut self) -> Vec<InnerMessage> {
        let Some(inner) = self.inner.as_mut() else {
            return Vec::new();
        };
        let cursor = match self.cursor_local {
            Some(p) => mouse::Cursor::Available(p),
            None => mouse::Cursor::Unavailable,
        };
        let events = std::mem::take(&mut self.pending_events);
        let messages = inner.frame(&mut self.ui, &events, cursor);
        self.evidence.ui_frames_presented += 1;

        let gpu = self.gpu.as_ref().unwrap();
        let composite = self.composite.as_mut().unwrap();
        let vp = self.view_proj;
        // 1) 3D scene (meshes) into the composite with its own depth pass.
        self.mesh3d.as_mut().unwrap().render_with(
            gpu.device(),
            gpu.queue(),
            composite.view(),
            composite.width,
            composite.height,
            [0.64625, 0.65575, 0.67075, 1.0],
            &vp,
            &crate::render3d::sample_light(),
            self.store3d.as_ref().unwrap(),
            &self.instances,
            true,
        );
        // 2) the portal plane composites the UI texture over the scene
        //    (separate submit AFTER the UI present + mesh submit — the
        //    DAG order children→parent; the plane never writes the texture
        //    it samples).
        let plane = self.plane.as_mut().unwrap();
        let model = Mat4::from_translation(self.plane_xform.center)
            * Mat4::from_scale(Vec3::new(
                self.plane_xform.half.x,
                self.plane_xform.half.y,
                1.0,
            ));
        plane.set_transform(gpu.device(), gpu.queue(), inner.content(), model, vp);
        {
            let mut encoder =
                gpu.device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("agp plane pass"),
                    });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("agp plane"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: composite.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if plane.draw_in_pass(&mut pass) {
                self.evidence.plane_passes_drawn += 1;
            }
            drop(pass);
            gpu.queue().submit(Some(encoder.finish()));
        }
        // 3) present the composite to the window (the offscreen composite
        //    IS the window content — same texture backs captures).
        if let Some(wt) = self.window_target.as_mut() {
            let blit = self.blit.as_ref().unwrap();
            let _ = wt.present_frame(gpu, |view| {
                let mut encoder =
                    gpu.device()
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("agp present"),
                        });
                blit.blit(gpu.device(), &mut encoder, view, composite.view());
                gpu.queue().submit(Some(encoder.finish()));
            });
        }
        messages
    }

    /// Capture acceptance keyframes (UI texture + composite) — GPU readback
    /// happens ONLY here, never in the frame path.
    pub fn capture(&mut self, name: &str) {
        enum Shot {
            Ui(Vec<u8>, u32, u32),
            Composite(Vec<u8>, u32, u32),
        }
        let mut shots: Vec<Shot> = Vec::new();
        {
            let gpu = self.gpu.as_ref().unwrap();
            if let Some(inner) = self.inner.as_ref() {
                match capture_png(gpu, inner.target()) {
                    Ok((png, _)) => shots.push(Shot::Ui(
                        png,
                        inner.physical_size().0,
                        inner.physical_size().1,
                    )),
                    Err(e) => self.failures.push(format!("capture {name}/ui: {e}")),
                }
            }
            if let Some(composite) = self.composite.as_ref() {
                match capture_png(gpu, composite) {
                    Ok((png, _)) => {
                        shots.push(Shot::Composite(png, composite.width, composite.height))
                    }
                    Err(e) => self.failures.push(format!("capture {name}/composite: {e}")),
                }
            }
        }
        for shot in shots {
            match shot {
                Shot::Ui(png, w, h) => {
                    self.save_keyframe(name, "ui", &png, w, h);
                    self.evidence.readbacks += 1;
                }
                Shot::Composite(png, w, h) => {
                    self.save_keyframe(name, "composite", &png, w, h);
                    self.evidence.readbacks += 1;
                }
            }
        }
    }

    fn save_keyframe(&mut self, name: &str, kind: &str, png: &[u8], w: u32, h: u32) {
        let _ = std::fs::create_dir_all(&self.report_dir);
        let file = format!("{name}_{kind}.png");
        let _ = std::fs::write(self.report_dir.join(&file), png);
        self.evidence.keyframes.push(Keyframe {
            name: format!("{name}_{kind}"),
            file,
            width: w,
            height: h,
            sha256: sha256_hex(png),
        });
    }

    pub fn map_window_pixel(&self, px: (f32, f32)) -> Option<[f32; 2]> {
        window_pixel_to_plane(px, self.view_proj, &self.plane_xform, self.viewport_size())
    }

    pub fn ui_local_click_target(&self, local: Point) -> (f32, f32) {
        ui_point_to_window_pixel(
            local,
            Size::new(INNER_LOGICAL.0, INNER_LOGICAL.1),
            self.view_proj,
            &self.plane_xform,
            self.viewport_size(),
        )
    }
}

// ---------------------------------------------------------------------------
// Canonical phase machine (bounded waits; deadlines diagnose, never hang).
// ---------------------------------------------------------------------------

const WARMUP_TICKS: u64 = 30;
const CLICK_DEADLINE_SECS: u64 = 30;
const TEXT_DEADLINE_SECS: u64 = 30;

pub fn canonical_step(probe: &mut SurfaceProbe, action: &mut Option<PendingAction>) -> bool {
    // Returns true when the loop should exit.
    let wall = probe.wall_tick;
    let age = probe.phase_started.elapsed();
    let presses_before = probe.ui.presses;
    let input_before = probe.ui.input.clone();
    match probe.phase {
        0 => {
            if wall >= WARMUP_TICKS {
                // harvest exact bounds for the record + click targeting
                let bounds = probe.probe_bounds();
                probe.evidence.button_bounds = bounds
                    .button_bounds()
                    .map(|r| [r.x, r.y, r.width, r.height]);
                probe.evidence.input_bounds =
                    bounds.text_input.map(|r| [r.x, r.y, r.width, r.height]);
                probe.capture("initial");
                probe.phase = 1;
                probe.phase_started = Instant::now();
                // click the button center through the REAL window input path
                if let Some(b) = bounds.button_bounds() {
                    let c = Point::new(b.x + b.width / 2.0, b.y + b.height / 2.0);
                    let target = probe.ui_local_click_target(c);
                    if std::env::var("AGP_INPUT_DEBUG").is_ok() {
                        eprintln!(
                            "DBG click target: ui={c:?} -> px={target:?} viewport={:?} scale_hint={:?}",
                            probe.viewport_size(),
                            probe.window.as_ref().map(|w| w.scale_factor())
                        );
                    }
                    *action = Some(PendingAction::ClickButton(target));
                } else {
                    probe.failures.push("button bounds not found".into());
                    probe.phase = 99;
                }
            }
        }
        1 => {
            // wait for the button message; re-dispatch periodically
            if probe.ui.presses > presses_before || probe.ui.presses > 0 {
                probe.phase = 2;
                probe.phase_started = Instant::now();
                let target = probe
                    .evidence
                    .input_bounds
                    .map(|b| Point::new(b[0] + b[2] / 2.0, b[1] + b[3] / 2.0));
                match target {
                    Some(p) => {
                        *action = Some(PendingAction::ClickInput(probe.ui_local_click_target(p)));
                    }
                    None => {
                        probe.failures.push("text_input bounds not found".into());
                        probe.phase = 99;
                    }
                }
            } else if age > Duration::from_secs(CLICK_DEADLINE_SECS) {
                probe
                    .failures
                    .push("button click not observed (bounded wait)".into());
                probe.phase = 99;
            } else if wall % 90 == 0 {
                let b = probe.evidence.button_bounds;
                if let Some(b) = b {
                    let c = Point::new(b[0] + b[2] / 2.0, b[1] + b[3] / 2.0);
                    *action = Some(PendingAction::ClickButton(probe.ui_local_click_target(c)));
                }
            }
        }
        2 => {
            // text_input clicked → type the real keys
            if age > Duration::from_millis(600) {
                *action = Some(PendingAction::TypeText(TYPED_TEXT.into()));
                probe.phase = 3;
                probe.phase_started = Instant::now();
            }
        }
        3 => {
            if probe.ui.input == TYPED_TEXT {
                probe.phase = 4;
                probe.phase_started = Instant::now();
            } else if age > Duration::from_secs(TEXT_DEADLINE_SECS) {
                probe.failures.push(format!(
                    "typed text not observed (bounded wait): {:?} != {:?}",
                    probe.ui.input, TYPED_TEXT
                ));
                probe.phase = 99;
            } else if wall % 90 == 0 && probe.ui.input == input_before {
                *action = Some(PendingAction::TypeText(TYPED_TEXT.into()));
            }
        }
        4 => {
            probe.capture("after_input");
            probe.phase = 5;
        }
        5 | 99 => {
            if !probe.finished {
                probe.finished = true;
                return true;
            }
        }
        _ => {}
    }
    false
}

/// A deferred OS input action (executed by the host thread outside the
/// event loop callback, then cleared).
pub enum PendingAction {
    ClickButton((f32, f32)),
    ClickInput((f32, f32)),
    TypeText(String),
}

pub fn run_action(action: PendingAction) {
    std::thread::spawn(move || match &action {
        PendingAction::ClickButton(p) | PendingAction::ClickInput(p) => {
            os_click_client(WINDOW_TITLE, *p);
        }
        PendingAction::TypeText(t) => os_type_text(WINDOW_TITLE, t),
    });
}

/// Public OS-input helpers shared by the surface cases (real foreground
/// input first, posted-queue fallback; both stay in winit's real pipeline).
pub fn os_click_client(title: &str, client_px: (f32, f32)) {
    click_with_fallback(title, client_px.0, client_px.1);
}

pub fn os_type_text(title: &str, text: &str) {
    type_with_fallback(title, text);
}

/// Single-shot synchronous click at a CLIENT position (no retry loop).
/// Used by phase-driven windowed cases that poll for the outcome themselves.
pub fn os_click_client_once(title: &str, client_px: (f32, f32), posted: bool) -> bool {
    use std::process::Command;
    let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class OneClick {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint a, uint b, uint d, UIntPtr e);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr h, int c);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, UIntPtr w, IntPtr l);
    public struct RECT { public int Left, Top, Right, Bottom; }
    public struct POINT { public int X, Y; }
    public static void Click(string title, int cx, int cy, bool post) {
        SetProcessDPIAware();
        IntPtr h = FindWindow(null, title);
        if (h == IntPtr.Zero) { Environment.Exit(1); }
        RECT cr; GetClientRect(h, out cr);
        int x = Math.Min(cx, cr.Right - 2);
        int y = Math.Min(cy, cr.Bottom - 2);
        if (post) {
            IntPtr l = (IntPtr)((y << 16) | (x & 0xffff));
            PostMessage(h, 0x0200, UIntPtr.Zero, l);
            System.Threading.Thread.Sleep(30);
            PostMessage(h, 0x0201, (UIntPtr)1, l);
            System.Threading.Thread.Sleep(30);
            PostMessage(h, 0x0202, UIntPtr.Zero, l);
            return;
        }
        POINT p = new POINT { X = x, Y = y };
        ClientToScreen(h, ref p);
        ShowWindowAsync(h, 9);
        SetForegroundWindow(h);
        System.Threading.Thread.Sleep(120);
        SetCursorPos(p.X, p.Y);
        System.Threading.Thread.Sleep(60);
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
        System.Threading.Thread.Sleep(40);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
    }
}
"@
[OneClick]::Click("TITLE", CX, CY, $POSTLIT)
"#
    .replace("TITLE", title)
    .replace("POSTLIT", if posted { "true" } else { "false" })
    .replace("CX", &client_px.0.round().to_string())
    .replace("CY", &client_px.1.round().to_string());
    if std::env::var("AGP_DUMP_CLICK").is_ok() {
        let _ = std::fs::write("scratch/click_debug.ps1", &script);
    }
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
    match out {
        Ok(o) => {
            if std::env::var("AGP_INPUT_DEBUG").is_ok() && !o.status.success() {
                eprintln!(
                    "DBG click script failed: {} | {}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                );
                eprintln!(
                    "DBG script tail: {}",
                    &script[script.len().saturating_sub(220)..]
                );
            }
            o.status.success()
        }
        Err(_) => false,
    }
}

fn click_with_fallback(title: &str, px: f32, py: f32) {
    use std::sync::atomic::Ordering;
    let baseline = PRESS_COUNT.load(Ordering::SeqCst);
    let clicker = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Clicker {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint a, uint b, uint d, UIntPtr e);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr h, int c);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    public struct RECT { public int Left, Top, Right, Bottom; }
    public struct POINT { public int X, Y; }
    public static void Click(string title, int cx, int cy) {
        SetProcessDPIAware();
        IntPtr h = FindWindow(null, title);
        if (h == IntPtr.Zero) { Environment.Exit(1); }
        RECT cr; GetClientRect(h, out cr);
        POINT p = new POINT { X = Math.Min(cx, cr.Right - 2), Y = Math.Min(cy, cr.Bottom - 2) };
        ClientToScreen(h, ref p);
        ShowWindowAsync(h, 9);
        SetForegroundWindow(h);
        System.Threading.Thread.Sleep(150);
        SetCursorPos(p.X, p.Y);
        System.Threading.Thread.Sleep(80);
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
        System.Threading.Thread.Sleep(40);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
    }
}
"@
[Clicker]::Click("TITLE", CX, CY)
"#
    .replace("TITLE", title)
    .replace("CX", &px.round().to_string())
    .replace("CY", &py.round().to_string());
    let poster = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Poster {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, UIntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    public static bool Post(string title, int x, int y) {
        SetProcessDPIAware();
        IntPtr h = FindWindow(null, title);
        if (h == IntPtr.Zero) return false;
        IntPtr l = (IntPtr)((y << 16) | (x & 0xffff));
        PostMessage(h, 0x0200, UIntPtr.Zero, l);
        System.Threading.Thread.Sleep(40);
        PostMessage(h, 0x0201, (UIntPtr)1, l);
        System.Threading.Thread.Sleep(40);
        PostMessage(h, 0x0202, UIntPtr.Zero, l);
        return true;
    }
}
"@
if ([Poster]::Post("TITLE", CX, CY)) { exit 0 } else { exit 1 }
"#
    .replace("TITLE", title)
    .replace("CX", &px.round().to_string())
    .replace("CY", &py.round().to_string());
    let run = |script: &str| {
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map(|o| o.status.code().unwrap_or(1) == 0)
            .unwrap_or(false)
    };
    for round in 0..12 {
        // Real OS click first (needs foreground); posted-queue fallback
        // (same winit pipeline, synthetic delivery) when focus is denied.
        let _ = run(&clicker);
        std::thread::sleep(Duration::from_millis(1200));
        if PRESS_COUNT.load(Ordering::SeqCst) > baseline {
            return;
        }
        let _ = run(&poster);
        std::thread::sleep(Duration::from_millis(1200));
        if PRESS_COUNT.load(Ordering::SeqCst) > baseline {
            return;
        }
        if round == 3 {
            eprintln!("surface-offscreen: OS click not landing; posting-queue fallback active");
        }
    }
}

fn type_with_fallback(title: &str, text: &str) {
    use std::sync::atomic::Ordering;
    let want = text.chars().map(|c| c as u32).collect::<Vec<u32>>();
    let sendkeys = r#"
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.SendKeys]::SendWait("TEXT")
"#
    .replace("TEXT", text);
    // winit derives KeyEvent text from WM_KEYDOWN via ToUnicode (it does not
    // process WM_CHAR), so the fallback posts real key-down/up pairs with
    // proper VK codes + scan codes.
    let poster = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class KeyPoster {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, UIntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern ushort VkKeyScan(char ch);
    [DllImport("user32.dll")] public static extern uint MapVirtualKey(uint vk, uint type);
    public static void Type(string title, string text) {
        SetProcessDPIAware();
        IntPtr h = FindWindow(null, title);
        if (h == IntPtr.Zero) { Environment.Exit(1); }
        SetForegroundWindow(h);
        System.Threading.Thread.Sleep(120);
        foreach (char ch in text) {
            uint vk = (uint)(VkKeyScan(ch) & 0xff);
            uint scan = MapVirtualKey(vk, 0);
            IntPtr lp = (IntPtr)(1 | ((int)scan << 16));
            PostMessage(h, 0x0100, (UIntPtr)vk, lp);
            System.Threading.Thread.Sleep(60);
            PostMessage(h, 0x0101, (UIntPtr)vk, (IntPtr)(1 | ((int)scan << 30) | ((int)scan << 16)));
            System.Threading.Thread.Sleep(60);
        }
    }
}
"@
[KeyPoster]::Type("TITLE", "TEXT")
"#.replace("TITLE", title).replace("TEXT", text);
    let run = |script: &str| {
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map(|o| o.status.code().unwrap_or(1) == 0)
            .unwrap_or(false)
    };
    for round in 0..12 {
        let _ = run(&sendkeys);
        std::thread::sleep(Duration::from_millis(1200));
        if TEXT_LEN.load(Ordering::SeqCst) as usize == want.len() {
            return;
        }
        let _ = run(&poster);
        std::thread::sleep(Duration::from_millis(1200));
        if TEXT_LEN.load(Ordering::SeqCst) as usize == want.len() {
            return;
        }
        if round == 3 {
            eprintln!("surface-offscreen: SendKeys not landing; WM_CHAR posting fallback active");
        }
    }
}

/// One posted key tap through native winit, without retrying a game action.
pub fn os_key_client_once(title: &str, vk: u32) -> bool {
    os_key_client_sequence(title, vk, false)
}
pub fn os_key_client_repeat_once(title: &str, vk: u32) -> bool {
    os_key_client_sequence(title, vk, true)
}
fn os_key_client_sequence(title: &str, vk: u32, repeat: bool) -> bool {
    let script = r#"
Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices;
public class OneKey {
 [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c,string n);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern uint MapVirtualKey(uint code,uint mode);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,UIntPtr w,IntPtr l);
 public static bool Tap(string title,uint code,bool repeat){var h=FindWindow(null,title);if(h==IntPtr.Zero)return false;SetForegroundWindow(h);System.Threading.Thread.Sleep(120);uint scan=MapVirtualKey(code,0);long bits=1L|((long)scan<<16);bool down=PostMessage(h,0x100,(UIntPtr)code,(IntPtr)bits);System.Threading.Thread.Sleep(60);if(repeat){PostMessage(h,0x100,(UIntPtr)code,(IntPtr)(bits|0x40000000L));System.Threading.Thread.Sleep(60);}return down&&PostMessage(h,0x101,(UIntPtr)code,(IntPtr)(bits|0xC0000000L));}
}
"@
if(-not [OneKey]::Tap($env:AGP_KEY_TITLE,[uint32]$env:AGP_KEY_CODE,($env:AGP_KEY_REPEAT -eq "1"))){exit 1}
"#;
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .env("AGP_KEY_TITLE", title)
        .env("AGP_KEY_CODE", vk.to_string())
        .env("AGP_KEY_REPEAT", if repeat { "1" } else { "0" })
        .status()
        .is_ok_and(|s| s.success())
}
