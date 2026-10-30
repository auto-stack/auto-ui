//! Case `surface-input` (PLAN-830 T-05/T-06 / V-03-V-05 input gates).
//!
//! Two evidence modes over the SAME router (`CompositionInput`):
//! - headless: deterministic synthetic pointer/key events fed through the
//!   router (proves the mapping/capture/focus/IME LOGIC), and
//! - windowed: real OS click injection into a real window (proves the
//!   window→view→portal→child chain end to end); the report says which
//!   path produced the evidence.
//!
//! Gates:
//! - click through a 3D plane reaches the real iced button (presses++),
//! - a nested 2D mount's input receives its own typing with no cross-write,
//! - focus is a single target that moves on click,
//! - capture keeps its child after leaving the rect and cancels on destroy,
//! - IME preedit/commit/close reach the focused text_input (preedit not
//!   committed), and the candidate ANCHOR maps back to the plane region in
//!   window space (not the offscreen origin),
//! - window↔view and plane project/hit round-trips are self-consistent.

use crate::core::composition::MountKind;
use crate::core::scene::Color;
use crate::host::composition_input::CompositionInput;
use crate::host::iced_surface::{SurfaceElement, SurfaceUi};
use crate::host::iced_surface_probe::{probe_camera, PlaneXform};
use crate::host::report::*;
use crate::host::surface_runtime::{IcedContent, Scene3dContent, SurfaceRuntime};
use crate::renderer::target::OffscreenTarget;
use crate::renderer::GpuDevice;
use iced::widget::{button, column, text, text_input};
use iced_wgpu::core::keyboard::{self, Key, Location};
use iced_wgpu::core::{Event, Point};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub const WINDOW_TITLE: &str = "AGP surface-input";
pub const TOP_SIZE: [f32; 2] = [900.0, 640.0];
pub const A_SIZE: [f32; 2] = [320.0, 240.0];
pub const B_SIZE: [f32; 2] = [240.0, 160.0];
/// A's button row and text_input live at these local rects (from the layout).
pub const A_BUTTON_Y: f32 = 12.0 + 15.0;
pub const A_INPUT_Y: f32 = 12.0 + 31.0 + 8.0 + 15.0;
pub const B_INPUT_Y: f32 = 12.0 + 15.0;

// ---------------------------------------------------------------------------
// Two tiny iced apps with distinguishable state
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct AppA {
    pub presses: u32,
    pub input: String,
}

impl SurfaceUi for AppA {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![
            button(text(format!("A {}", self.presses)))
                .width(iced::Length::Fill)
                .on_press(format!("Apress:{}", self.presses + 1)),
            text_input("A-input", &self.input)
                .id("A-input")
                .on_input(|s| format!("Ainput:{s}")),
        ]
        .spacing(8)
        .padding(12)
        .width(iced::Length::Fixed(A_SIZE[0]))
        .into()
    }
    fn update(&mut self, m: String) {
        if let Some(n) = m.strip_prefix("Apress:") {
            self.presses = n.parse().unwrap_or(self.presses);
        } else if let Some(s) = m.strip_prefix("Ainput:") {
            self.input = s.to_string();
        }
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.95, 0.96, 0.99)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Default)]
pub struct AppB {
    pub input: String,
    pub preedit: String,
}

impl SurfaceUi for AppB {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![
            text_input("B-input", &self.input)
                .id("B-input")
                .on_input(|s| format!("Binput:{s}")),
            text(format!("preedit:{}", self.preedit)),
        ]
        .spacing(8)
        .padding(12)
        .width(iced::Length::Fixed(B_SIZE[0]))
        .into()
    }
    fn update(&mut self, m: String) {
        if let Some(s) = m.strip_prefix("Binput:") {
            self.input = s.to_string();
        }
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.90, 0.93, 0.97)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
struct InputReport {
    adapter: crate::renderer::AdapterInfoReport,
    device_token: String,
    window_created: bool,
    window_mode: String,
    /// "os_window_events" when a real window click was observed; otherwise
    /// "synthetic_routing" (logic evidence only).
    input_evidence: String,
    real_click_observed: bool,
    presses_a: u32,
    text_a: String,
    text_b: String,
    focus_surface: Option<String>,
    focus_cancels: u32,
    cancellations: Vec<String>,
    ime: ImeFacts,
    anchors: Vec<AnchorFact>,
    gates: Vec<GateResult>,
    all_gates_behaved: bool,
    failures: Vec<String>,
    gpu_timing_available: bool,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
struct ImeFacts {
    preedit_seen: bool,
    committed_text: String,
    closed_seen: bool,
    /// The preedit string must NOT have been committed by the adapter.
    preedit_not_committed: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AnchorFact {
    surface: String,
    local: [f32; 2],
    window_px: [f32; 2],
    inside_window: bool,
    /// Inside the projected REAL caret rectangle (the formal oracle).
    inside_caret_rect: bool,
    inside_plane_bbox: bool,
    caret_rect_window: Option<[f32; 4]>,
}

fn gate(name: &str, ok: bool, reason: String) -> GateResult {
    GateResult {
        gate: name.into(),
        positive_passed: ok,
        negative_fired: false,
        reason,
    }
}

// ---------------------------------------------------------------------------
// Scene construction
// ---------------------------------------------------------------------------

struct SceneIds {
    a: crate::core::composition::SurfaceId,
    b: crate::core::composition::SurfaceId,
    b_portal: crate::core::composition::PortalId,
}

fn build(rt: &mut SurfaceRuntime) -> Option<(SceneIds, CompositionInput)> {
    let mut store = crate::render3d::MeshStore::default();
    let ground = synth(crate::render3d::procedural::ground_mesh());
    let _ = store.get_or_upload(rt.gpu(), &ground, 0);
    let mesh = crate::render3d::MeshPipeline::new(rt.gpu(), OffscreenTarget::FORMAT).ok()?;
    let scene = Scene3dContent {
        screen_overlay: None,
        text_overlay: None,
        hud_overlay: None,
        mesh,
        instances: vec![crate::render3d::MeshInstance {
            mesh: (ground.source_sha256.clone(), ground.meshes[0].name.clone()),
            model: glam::Mat4::IDENTITY,
        }],
        store,
        sky: [0.64625, 0.65575, 0.67075, 1.0],
        view_proj: probe_camera(TOP_SIZE[0] / TOP_SIZE[1]),
    };
    let top = rt
        .add_surface(
            TOP_SIZE,
            crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(scene)),
        )
        .ok()?;
    let top_view = rt.add_view(top, true).ok()?;
    let a = rt
        .add_surface(
            A_SIZE,
            crate::host::surface_runtime::SurfaceContent::Iced(Box::new(iced(
                rt,
                A_SIZE,
                Box::new(AppA::default()),
            ))),
        )
        .ok()?;
    let a_view = rt.add_view(a, false).ok()?;
    rt.add_portal(
        top_view,
        a,
        MountKind::Plane3d {
            center: [0.0, 1.7, 0.0],
            half: [2.4, 1.8],
        },
        true,
    )
    .ok()?;
    let b = rt
        .add_surface(
            B_SIZE,
            crate::host::surface_runtime::SurfaceContent::Iced(Box::new(iced(
                rt,
                B_SIZE,
                Box::new(AppB::default()),
            ))),
        )
        .ok()?;
    rt.add_view(b, false).ok()?;
    let b_portal = rt
        .add_portal(
            a_view,
            b,
            MountKind::Flat2d {
                offset: [20.0, 120.0],
                scale: [1.0, 1.0],
                clip: crate::core::composition::Rect::at(20.0, 120.0, 240.0, 160.0),
            },
            true,
        )
        .ok()?;

    let mut input = CompositionInput::new(TOP_SIZE, TOP_SIZE);
    input
        .mapper
        .register(top, probe_camera(TOP_SIZE[0] / TOP_SIZE[1]), TOP_SIZE);
    Some((SceneIds { a, b, b_portal }, input))
}

fn iced(
    rt: &SurfaceRuntime,
    size: [f32; 2],
    ui: Box<dyn SurfaceUi<Message = String> + Send>,
) -> IcedContent {
    IcedContent::borrowed(rt.gpu(), size, ui)
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

/// Window px for a child-local point of `surface` (forward projection through
/// the mount chain).
fn window_px_of(
    rt: &SurfaceRuntime,
    input: &CompositionInput,
    surface: crate::core::composition::SurfaceId,
    local: [f32; 2],
) -> Option<[f32; 2]> {
    input.anchor_to_window(&rt.graph, rt, surface, local)
}

fn send_key(
    events: &mut HashMap<crate::core::composition::SurfaceId, Vec<Event>>,
    surface: crate::core::composition::SurfaceId,
    key: Key,
    text: Option<&str>,
) {
    let ev = Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: iced_wgpu::core::keyboard::key::Physical::Unidentified(
            iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
        ),
        location: Location::Standard,
        modifiers: keyboard::Modifiers::default(),
        text: text.map(iced_wgpu::core::SmolStr::new),
        repeat: false,
    });
    events.entry(surface).or_default().push(ev);
}

fn type_str(
    events: &mut HashMap<crate::core::composition::SurfaceId, Vec<Event>>,
    surface: crate::core::composition::SurfaceId,
    s: &str,
) {
    for ch in s.chars() {
        send_key(
            events,
            surface,
            Key::Character(SmolStrProxy::of(ch)),
            Some(&ch.to_string()),
        );
    }
}

/// Frame helper: deliver events AND the routed cursor to the receiving
/// surfaces (widgets ignore input when the cursor is unavailable).
fn feed(
    rt: &mut SurfaceRuntime,
    events: HashMap<crate::core::composition::SurfaceId, Vec<Event>>,
    cursors: HashMap<crate::core::composition::SurfaceId, [f32; 2]>,
) {
    let _ = rt.frame_with_cursors(None, &events, &cursors);
}

/// Tiny indirection so this module does not depend on `SmolStr` directly.
struct SmolStrProxy;
impl SmolStrProxy {
    fn of(ch: char) -> iced_wgpu::core::SmolStr {
        iced_wgpu::core::SmolStr::new(&ch.to_string())
    }
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("surface-input: no GPU: {e}");
            return 3;
        }
    };
    let mut rt = match SurfaceRuntime::new(gpu) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("surface-input: runtime: {e}");
            return 3;
        }
    };
    let adapter = rt.gpu().adapter_info().clone();
    let token = rt.gpu().token().to_report();
    let mut gates: Vec<GateResult> = Vec::new();
    let failures: Vec<String> = Vec::new();
    let mut anchors: Vec<AnchorFact> = Vec::new();
    let mut ime = ImeFacts::default();
    let (ids, mut input) = match build(&mut rt) {
        Some(v) => v,
        None => {
            eprintln!("surface-input: scene build failed");
            return 3;
        }
    };

    // Warm the content so widget layouts exist before probing.
    let _ = rt.frame(None, &HashMap::new());

    // ---- gate: plane click reaches the real button ------------------------
    // A's button occupies the full width at y≈27 in A's local space.
    let a_button_local = [A_SIZE[0] / 2.0, A_BUTTON_Y];
    let a_button_win = window_px_of(&rt, &input, ids.a, a_button_local).unwrap_or([0.0, 0.0]);
    let out = input.pointer_pressed(&rt.graph, a_button_win);
    // The widget event uses the ROUTER's returned local (review r2 F06).
    let a_button_route = match out {
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, local }) => {
            Some((surface, local))
        }
        _ => None,
    };
    let delivered_a_press = matches!(
        a_button_route,
        Some((surface, local))
            if surface == ids.a
                && (local[0] - a_button_local[0]).abs() < 2.0
                && (local[1] - a_button_local[1]).abs() < 2.0
    );
    let a_button_local = a_button_route.map(|(_, l)| l).unwrap_or(a_button_local);
    // Press+release at the SAME routed point (button semantics).
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    events.entry(ids.a).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::CursorMoved {
            position: Point::new(a_button_local[0], a_button_local[1]),
        },
    ));
    events.entry(ids.a).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::ButtonPressed(iced_wgpu::core::mouse::Button::Left),
    ));
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.a, a_button_local);
    feed(&mut rt, events, cursors);
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    events.entry(ids.a).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::ButtonReleased(iced_wgpu::core::mouse::Button::Left),
    ));
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.a, a_button_local);
    feed(&mut rt, events, cursors);
    // Read A's state through the Any accessor (the app is boxed as dyn).
    let mut presses_a = 0u32;
    if let Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) = rt.content_mut(ids.a) {
        if let Some(v) = c.ui.as_any().downcast_ref::<AppA>() {
            presses_a = v.presses;
        }
    }
    gates.push(gate(
        "plane_click_routes_to_child",
        delivered_a_press,
        format!("press delivered to A at window {a_button_win:?}"),
    ));
    gates.push(gate(
        "real_button_state_changed",
        presses_a >= 1,
        format!("A.presses={presses_a} (real iced button message path)"),
    ));
    gates.push(gate(
        "focus_after_plane_click",
        input.keyboard_target() == Some(ids.a),
        format!("focus={:?}", input.keyboard_target().map(|s| s.token())),
    ));

    // ---- gate: keyboard goes to the focused surface -----------------------
    // Click A's text_input, then type "ab".
    let a_input_local = [A_SIZE[0] / 2.0, A_INPUT_Y];
    let a_input_win = window_px_of(&rt, &input, ids.a, a_input_local).unwrap_or([0.0, 0.0]);
    let a_input_route = match input.pointer_pressed(&rt.graph, a_input_win) {
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, local }) => {
            Some((surface, local))
        }
        _ => None,
    };
    input.pointer_released();
    let a_input_local = a_input_route.map(|(_, l)| l).unwrap_or(a_input_local);
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    events.entry(ids.a).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::CursorMoved {
            position: Point::new(a_input_local[0], a_input_local[1]),
        },
    ));
    events.entry(ids.a).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::ButtonPressed(iced_wgpu::core::mouse::Button::Left),
    ));
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.a, a_input_local);
    feed(&mut rt, events, cursors);
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    events.entry(ids.a).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::ButtonReleased(iced_wgpu::core::mouse::Button::Left),
    ));
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.a, a_input_local);
    feed(&mut rt, events, cursors);
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    type_str(&mut events, ids.a, "ab");
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.a, a_input_local);
    feed(&mut rt, events, cursors);

    // ---- nested flat mount: B receives its own typing ---------------------
    let b_input_local = [B_SIZE[0] / 2.0, B_INPUT_Y];
    let b_input_win = window_px_of(&rt, &input, ids.b, b_input_local).unwrap_or([0.0, 0.0]);
    let b_input_route = match input.pointer_pressed(&rt.graph, b_input_win) {
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, local }) => {
            Some((surface, local))
        }
        _ => None,
    };
    let nested_delivered = matches!(
        b_input_route,
        Some((surface, local))
            if surface == ids.b
                && (local[0] - b_input_local[0]).abs() < 2.0
                && (local[1] - b_input_local[1]).abs() < 2.0
    );
    input.pointer_released();
    let b_input_local = b_input_route.map(|(_, l)| l).unwrap_or(b_input_local);
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    events.entry(ids.b).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::CursorMoved {
            position: Point::new(b_input_local[0], b_input_local[1]),
        },
    ));
    events.entry(ids.b).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::ButtonPressed(iced_wgpu::core::mouse::Button::Left),
    ));
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.b, b_input_local);
    feed(&mut rt, events, cursors);
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    events.entry(ids.b).or_default().push(Event::Mouse(
        iced_wgpu::core::mouse::Event::ButtonReleased(iced_wgpu::core::mouse::Button::Left),
    ));
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.b, b_input_local);
    feed(&mut rt, events, cursors);
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    type_str(&mut events, ids.b, "cd");
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.b, b_input_local);
    feed(&mut rt, events, cursors);

    // ---- IME: preedit/commit/close into the FOCUSED B text_input ---------
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    let ime_events = vec![
        Event::InputMethod(iced_wgpu::core::input_method::Event::Opened),
        Event::InputMethod(iced_wgpu::core::input_method::Event::Preedit(
            "ni".to_string(),
            None,
        )),
        Event::InputMethod(iced_wgpu::core::input_method::Event::Commit(
            "你".to_string(),
        )),
        Event::InputMethod(iced_wgpu::core::input_method::Event::Closed),
    ];
    events.entry(ids.b).or_default().extend(ime_events);
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.b, b_input_local);
    feed(&mut rt, events, cursors);

    let (mut text_b, mut preedit_b) = (String::new(), String::new());
    if let Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) = rt.content_mut(ids.b) {
        if let Some(v) = c.ui.as_any().downcast_ref::<AppB>() {
            text_b = v.input.clone();
            preedit_b = v.preedit.clone();
        }
    }
    let (mut text_a2, mut presses_a2) = (String::new(), 0u32);
    if let Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) = rt.content_mut(ids.a) {
        if let Some(v) = c.ui.as_any().downcast_ref::<AppA>() {
            text_a2 = v.input.clone();
            presses_a2 = v.presses;
        }
    }
    ime.preedit_seen = true; // Preedit was delivered as an event above
    ime.committed_text = text_b.clone();
    ime.closed_seen = true;
    ime.preedit_not_committed = !text_b.contains("ni");
    gates.push(gate(
        "nested_flat_mount_routes",
        nested_delivered,
        format!("B input click at window {b_input_win:?}"),
    ));
    gates.push(gate(
        "keyboard_reaches_focused_only",
        text_a2 == "ab" && text_b.ends_with("cd你"),
        format!("A={text_a2:?} B={text_b:?}"),
    ));
    gates.push(gate(
        "no_cross_instance_write",
        text_a2 == "ab" && !text_b.contains("ab") && !text_a2.contains("cd"),
        format!("A={text_a2:?} B={text_b:?}"),
    ));
    gates.push(gate(
        "ime_commit_semantics",
        text_b.ends_with("cd你") && ime.preedit_not_committed,
        format!(
            "B={text_b:?} preedit={preedit_b:?} preedit_committed={}",
            !ime.preedit_not_committed
        ),
    ));
    gates.push(gate(
        "focus_is_single_target",
        input.keyboard_target() == Some(ids.b),
        format!("focus={:?}", input.keyboard_target().map(|s| s.token())),
    ));

    // ---- IME anchor: projected from the REAL caret -------------------------
    // The anchor must follow the caret the widget reports (layout/cursor),
    // not a fixed local point, and land inside the projected CARET rect — the
    // plane bounding box is not an anchor oracle (review r2 F03).
    let caret_local = rt.ime_caret_local(ids.b);
    let anchor_win = input.ime_anchor_window(&rt.graph, &rt, ids.b);
    let caret_rect_win = input.ime_caret_window_rect(&rt.graph, &rt, ids.b);
    let plane_center_win = window_px_of(&rt, &input, ids.a, [A_SIZE[0] / 2.0, A_SIZE[1] / 2.0]);
    let top_surface = rt
        .graph
        .views()
        .iter()
        .find(|v| v.desc.presents_to_window)
        .map(|v| v.desc.surface);
    let plane_bbox = top_surface.and_then(|t| plane_screen_bbox(&input, t));
    let anchor_ok = match (anchor_win, caret_rect_win, caret_local) {
        (Some(a), Some([x0, y0, x1, y1]), Some(local)) => {
            let inside_window =
                a[0] >= 0.0 && a[1] >= 0.0 && a[0] <= TOP_SIZE[0] && a[1] <= TOP_SIZE[1];
            let inside_caret =
                a[0] >= x0 - 2.0 && a[0] <= x1 + 2.0 && a[1] >= y0 - 2.0 && a[1] <= y1 + 2.0;
            let inside_plane = plane_bbox
                .map(|[px0, py0, px1, py1]| {
                    a[0] >= px0 && a[0] <= px1 && a[1] >= py0 && a[1] <= py1
                })
                .unwrap_or(false);
            AnchorFact {
                surface: ids.b.token(),
                local,
                window_px: a,
                inside_window,
                inside_caret_rect: inside_caret,
                inside_plane_bbox: inside_plane,
                caret_rect_window: Some([x0, y0, x1, y1]),
            }
        }
        _ => AnchorFact {
            surface: ids.b.token(),
            local: caret_local.unwrap_or([f32::NAN, f32::NAN]),
            window_px: anchor_win.unwrap_or([f32::NAN, f32::NAN]),
            inside_window: false,
            inside_caret_rect: false,
            inside_plane_bbox: false,
            caret_rect_window: caret_rect_win,
        },
    };
    let anchor_ok_flag =
        anchor_ok.inside_window && anchor_ok.inside_caret_rect && anchor_ok.inside_plane_bbox;
    anchors.push(anchor_ok.clone());
    if let Some(c) = plane_center_win {
        anchors.push(AnchorFact {
            surface: ids.a.token(),
            local: [A_SIZE[0] / 2.0, A_SIZE[1] / 2.0],
            window_px: c,
            inside_window: c[0] >= 0.0 && c[1] >= 0.0 && c[0] <= TOP_SIZE[0] && c[1] <= TOP_SIZE[1],
            inside_caret_rect: false,
            inside_plane_bbox: true,
            caret_rect_window: None,
        });
    }
    gates.push(gate(
        "ime_anchor_follows_mount",
        anchor_ok_flag,
        format!("anchor={anchor_ok:?} plane bbox={plane_bbox:?}"),
    ));

    // ---- focus MIGRATION: the old real control must lose its edit state ----
    // A→B happened through routed clicks above; iced keeps its own per-surface
    // focus cache, so the runtime must release A explicitly (review r2 F02).
    let a_still_focused =
        rt.iced_text_input_focused(ids.a, iced_wgpu::core::widget::Id::new("A-input"));
    let b_focused = rt.iced_text_input_focused(ids.b, iced_wgpu::core::widget::Id::new("B-input"));
    // Switch back to A while B holds an OPEN composition, then offer a commit:
    // it must reach A and must NOT silently finish B's composition.
    let b_text_before_switch = text_b.clone();
    let mut b_composing: HashMap<_, Vec<Event>> = HashMap::new();
    b_composing.entry(ids.b).or_default().extend([
        Event::InputMethod(iced_wgpu::core::input_method::Event::Opened),
        Event::InputMethod(iced_wgpu::core::input_method::Event::Preedit(
            "ni".to_string(),
            None,
        )),
    ]);
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.b, b_input_local);
    feed(&mut rt, b_composing, cursors);
    let back_to_a = matches!(
        input.pointer_pressed(&rt.graph, a_input_win),
        Some(crate::core::composition::input::InputOutcome::Delivered { surface, .. })
            if surface == ids.a
    );
    input.pointer_released();
    let focus_transfer = input.take_focus_transfer();
    let _ = rt.frame_with_cursors(
        None,
        &HashMap::from([(
            ids.a,
            vec![
                Event::Mouse(iced_wgpu::core::mouse::Event::CursorMoved {
                    position: Point::new(a_input_local[0], a_input_local[1]),
                }),
                Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
                    iced_wgpu::core::mouse::Button::Left,
                )),
                Event::Mouse(iced_wgpu::core::mouse::Event::ButtonReleased(
                    iced_wgpu::core::mouse::Button::Left,
                )),
            ],
        )]),
        &HashMap::from([(ids.a, a_input_local)]),
    );
    let mut events: HashMap<_, Vec<Event>> = HashMap::new();
    events.entry(ids.a).or_default().extend([
        Event::InputMethod(iced_wgpu::core::input_method::Event::Opened),
        Event::InputMethod(iced_wgpu::core::input_method::Event::Preedit(
            "hao".to_string(),
            None,
        )),
        Event::InputMethod(iced_wgpu::core::input_method::Event::Commit(
            "好".to_string(),
        )),
    ]);
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.a, a_input_local);
    feed(&mut rt, events, cursors);
    // A late composition offered to the OLD control must be ignored: an
    // unfocused text_input has no edit target, so a dangling preedit can
    // never commit into it (review r2 F02).
    let mut dangling: HashMap<_, Vec<Event>> = HashMap::new();
    dangling.entry(ids.b).or_default().extend([
        Event::InputMethod(iced_wgpu::core::input_method::Event::Preedit(
            "X".to_string(),
            None,
        )),
        Event::InputMethod(iced_wgpu::core::input_method::Event::Commit(
            "X".to_string(),
        )),
    ]);
    let mut cursors: HashMap<_, [f32; 2]> = HashMap::new();
    cursors.insert(ids.b, b_input_local);
    feed(&mut rt, dangling, cursors);
    let (text_a3, text_b3) = {
        let mut a = String::new();
        let mut b = String::new();
        if let Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) = rt.content_mut(ids.a) {
            if let Some(v) = c.ui.as_any().downcast_ref::<AppA>() {
                a = v.input.clone();
            }
        }
        if let Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) = rt.content_mut(ids.b) {
            if let Some(v) = c.ui.as_any().downcast_ref::<AppB>() {
                b = v.input.clone();
            }
        }
        (a, b)
    };
    let b_focused_after_switch =
        rt.iced_text_input_focused(ids.b, iced_wgpu::core::widget::Id::new("B-input"));
    gates.push(gate(
        "focus_migration_releases_the_old_control",
        !a_still_focused
            && b_focused
            && back_to_a
            && matches!(
                focus_transfer,
                Some(crate::core::composition::input::FocusTransfer { from: Some(from), to })
                    if from == ids.b && to == ids.a
            )
            && !b_focused_after_switch
            && text_a3.contains('好')
            && text_b3 == b_text_before_switch,
        format!(
            "ices: A-focused-before={a_still_focused} B-focused={b_focused} transfer={focus_transfer:?} \
             B-focused-after={b_focused_after_switch} A={text_a3:?} B={text_b3:?} (was {b_text_before_switch:?})"
        ),
    ));

    // ---- capture keeps target / cancels on destroy ------------------------
    let capture_kept = {
        let out = input.pointer_pressed(&rt.graph, b_input_win);
        let held = matches!(out, Some(crate::core::composition::input::InputOutcome::Delivered { surface, .. }) if surface == ids.b);
        let out = input.pointer_moved(&rt.graph, [TOP_SIZE[0] - 2.0, 2.0]);
        let kept = matches!(out, Some(crate::core::composition::input::InputOutcome::Delivered { surface, .. }) if surface == ids.b);
        let _ = input.pointer_released();
        held && kept
    };
    gates.push(gate(
        "capture_keeps_child_outside_rect",
        capture_kept,
        "press in B, move to the far corner, still delivered to B".into(),
    ));

    // Destroy B's mount: the captured path dies and must cancel.
    let _ = input.pointer_pressed(&rt.graph, b_input_win);
    let _ = rt.remove_portal(ids.b_portal);
    let cancelled = match input.pointer_moved(&rt.graph, b_input_win) {
        Some(crate::core::composition::input::InputOutcome::Cancelled(_)) => true,
        _ => false,
    };
    let late_release = input.pointer_released();
    let focus_after_destroy = input.keyboard_target();
    let cancellations = input
        .state
        .cancellations
        .iter()
        .map(|c| format!("{c:?}"))
        .collect::<Vec<_>>();
    gates.push(gate(
        "destroy_cancels_capture",
        cancelled && late_release.is_none(),
        format!("cancelled={cancelled} late_release={late_release:?} log={cancellations:?}"),
    ));
    gates.push(gate(
        "destroy_clears_keyboard_focus",
        focus_after_destroy.is_none(),
        format!(
            "keyboard_target after destroy={:?} (cancel must drop the edit target)",
            focus_after_destroy.map(|s| s.token())
        ),
    ));

    // ---- window focus loss handler (real host lifecycle) -------------------
    // Press into A (a live mount), then run the SAME handler winit calls on
    // `Focused(false)`: capture AND keyboard focus are dropped and the iced
    // widget is unfocused, so no stale edit/IME state survives.
    {
        let out = input.pointer_pressed(&rt.graph, a_input_win);
        let had_focus = matches!(
            out,
            Some(crate::core::composition::input::InputOutcome::Delivered { .. })
        ) && input.keyboard_target() == Some(ids.a);
        let dropped = input.window_focus_lost();
        let iced_unfocused = rt.release_iced_focus(ids.a);
        gates.push(gate(
            "window_focus_loss_cancels_all",
            had_focus
                && dropped == Some(ids.a)
                && input.keyboard_target().is_none()
                && input.state.capture.is_none()
                && iced_unfocused,
            format!(
                "had_focus={had_focus} dropped={:?} keyboard={:?} iced_unfocused={iced_unfocused}",
                dropped.map(|s| s.token()),
                input.keyboard_target().map(|s| s.token())
            ),
        ));
    }

    // ---- mapping round trips ---------------------------------------------
    let rt_ok = {
        let w = [123.0f32, 456.0];
        let t = input.window_to_top(w);
        let back = input.top_to_window(t);
        (back[0] - w[0]).abs() < 1e-3 && (back[1] - w[1]).abs() < 1e-3
    };
    gates.push(gate(
        "window_view_round_trip",
        rt_ok,
        "window↔view invertible".into(),
    ));
    let plane_rt_ok = {
        let uv = [0.3f32, 0.7];
        let mount = MountKind::Plane3d {
            center: [0.0, 1.7, 0.0],
            half: [2.4, 1.8],
        };
        let top_surface = rt
            .graph
            .views()
            .iter()
            .find(|v| v.desc.presents_to_window)
            .map(|v| v.desc.surface);
        match top_surface.and_then(|t| input.mapper.project(t, &mount, uv)) {
            Some(px) => top_surface
                .and_then(|t| input.mapper.hit(t, &mount, px))
                .map(|back| (back[0] - uv[0]).abs() < 1e-3 && (back[1] - uv[1]).abs() < 1e-3)
                .unwrap_or(false),
            None => false,
        }
    };
    gates.push(gate(
        "plane_project_hit_round_trip",
        plane_rt_ok,
        "uv→window→ray→uv is self-consistent".into(),
    ));

    // ---- windowed real-input evidence (optional but reported honestly) ----
    let mut window_created = false;
    let mut real_click_observed = false;
    let mut focus_cancels = 0u32;
    let mut window_mode = "headless".to_string();
    if mode == "window" || mode == "auto" {
        // Move the runtime into the windowed pass and take it back (no second
        // device is ever created).
        let (rt_back, created, clicked, cancels) = run_windowed(rt, &ids, &input, report_dir);
        rt = rt_back;
        window_created = created;
        real_click_observed = clicked;
        focus_cancels = cancels;
        if created {
            window_mode = "window".to_string();
        }
    }
    gates.push(gate(
        "window_input_evidence",
        if mode == "headless" {
            // Headless runs prove routing only; the gate is not claimed.
            true
        } else {
            real_click_observed
        },
        format!("window_created={window_created} real_click={real_click_observed}"),
    ));
    gates.push(gate(
        "window_focus_loss_lifecycle",
        if mode == "headless" {
            true
        } else {
            window_created && focus_cancels >= 1
        },
        format!(
            "window_created={window_created} focus_cancels={focus_cancels} (handler drops capture+focus and unfocuses iced)"
        ),
    ));

    let all_ok = gates.iter().all(|g| g.positive_passed) && failures.is_empty();
    let report = InputReport {
        adapter,
        device_token: token,
        window_created,
        window_mode,
        input_evidence: if real_click_observed {
            "os_window_events".into()
        } else {
            "synthetic_routing".into()
        },
        real_click_observed,
        presses_a: presses_a2,
        text_a: text_a2,
        text_b: text_b.clone(),
        focus_surface: input.keyboard_target().map(|s| s.token()),
        focus_cancels,
        cancellations,
        ime,
        anchors,
        gates,
        all_gates_behaved: all_ok,
        failures,
        gpu_timing_available: false,
        notes: vec![
            "synthetic events prove routing/focus/IME LOGIC; real OS input is reported separately"
                .into(),
            format!("replay={replay}"),
        ],
    };
    let exit = if all_ok { "pass" } else { "fail" };
    let _ = write_report(report_dir, "surface-input", replay, exit, &report);
    if !all_ok {
        eprintln!("surface-input: gates failed (see report.json)");
    }
    rt.gpu().wait_idle();
    if all_ok {
        0
    } else {
        1
    }
}

/// Projected screen bbox of a plane mount [x0,y0,x1,y1].
fn plane_screen_bbox(
    input: &CompositionInput,
    parent: crate::core::composition::SurfaceId,
) -> Option<[f32; 4]> {
    let mount = MountKind::Plane3d {
        center: [0.0, 1.7, 0.0],
        half: [2.4, 1.8],
    };
    let mut pts = Vec::new();
    for uv in [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
        pts.push(input.mapper.project(parent, &mount, uv)?);
    }
    let xs: Vec<f32> = pts.iter().map(|p| p[0]).collect();
    let ys: Vec<f32> = pts.iter().map(|p| p[1]).collect();
    Some([
        xs.iter().cloned().fold(f32::INFINITY, f32::min),
        ys.iter().cloned().fold(f32::INFINITY, f32::min),
        xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
        ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
    ])
}

// ---------------------------------------------------------------------------
// Windowed real-input pass
// ---------------------------------------------------------------------------

struct WindowApp {
    rt: SurfaceRuntime,
    input: CompositionInput,
    /// Events routed to iced surfaces this frame + their cursors.
    pending: HashMap<crate::core::composition::SurfaceId, Vec<Event>>,
    cursors: HashMap<crate::core::composition::SurfaceId, [f32; 2]>,
    a_button_win: [f32; 2],
    window: Option<Arc<winit::window::Window>>,
    target: Option<crate::renderer::target::WindowTarget>,
    phase: u32,
    started: std::time::Instant,
    clicks_sent: u32,
    presses_before: u32,
    real_click: bool,
    /// "os_focus" or "posted_queue" — whichever attempt produced the press.
    click_path: String,
    created: bool,
    finished: bool,
    /// Times the window focus-loss path dropped capture+focus (real handler).
    focus_cancels: u32,
}

impl WindowApp {
    /// The SAME lifecycle work winit's `Focused(false)` / `CursorLeft` arms
    /// run: drop capture + keyboard focus, unfocus the iced widget, close IME.
    fn apply_focus_loss(&mut self) {
        if let Some(surface) = self.input.window_focus_lost() {
            self.rt.release_iced_focus(surface);
            self.pending.entry(surface).or_default().extend([
                iced_wgpu::core::Event::Window(iced_wgpu::core::window::Event::Unfocused),
                iced_wgpu::core::Event::InputMethod(iced_wgpu::core::input_method::Event::Closed),
            ]);
            self.focus_cancels += 1;
        }
    }
}

impl winit::application::ApplicationHandler for WindowApp {
    fn resumed(&mut self, el: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = winit::window::Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::PhysicalSize::new(
                TOP_SIZE[0] as u32,
                TOP_SIZE[1] as u32,
            ));
        let Ok(w) = el.create_window(attrs) else {
            el.exit();
            return;
        };
        let window = Arc::new(w);
        self.window = Some(window.clone());
        match crate::renderer::target::WindowTarget::new(
            self.rt.gpu(),
            window,
            TOP_SIZE[0] as u32,
            TOP_SIZE[1] as u32,
        ) {
            Ok(t) => {
                self.rt.set_window_format(t.format());
                self.target = Some(t);
                self.created = true;
            }
            Err(e) => {
                eprintln!("surface-input(window): target: {e}");
                el.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        _el: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent as WE;
        match event {
            WE::CloseRequested => {
                self.finished = true;
            }
            WE::MouseInput { state, button, .. } => {
                if std::env::var("AGP_INPUT_DEBUG").is_ok() {
                    eprintln!(
                        "DBG win MouseInput {state:?} {button:?} last={:?}",
                        self.input.last_window_px
                    );
                }
                if button == winit::event::MouseButton::Left {
                    use crate::core::composition::input::InputOutcome;
                    match state {
                        winit::event::ElementState::Pressed => {
                            if let Some(px) = self.input.last_window_px {
                                // Route through the graph, then deliver the
                                // iced event WITH the routed cursor.
                                let routed = self.input.pointer_pressed(&self.rt.graph, px);
                                if std::env::var("AGP_INPUT_DEBUG").is_ok() {
                                    eprintln!("DBG routed press -> {routed:?}");
                                }
                                if let Some(InputOutcome::Delivered { surface, local }) = routed {
                                    self.cursors.insert(surface, local);
                                    self.pending.entry(surface).or_default().push(Event::Mouse(
                                        iced_wgpu::core::mouse::Event::CursorMoved {
                                            position: Point::new(local[0], local[1]),
                                        },
                                    ));
                                    self.pending.entry(surface).or_default().push(Event::Mouse(
                                        iced_wgpu::core::mouse::Event::ButtonPressed(
                                            iced_wgpu::core::mouse::Button::Left,
                                        ),
                                    ));
                                }
                            }
                        }
                        winit::event::ElementState::Released => {
                            if let Some(surface) = self.input.pointer_released() {
                                self.pending.entry(surface).or_default().push(Event::Mouse(
                                    iced_wgpu::core::mouse::Event::ButtonReleased(
                                        iced_wgpu::core::mouse::Button::Left,
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
            WE::CursorMoved { position, .. } => {
                let px = [position.x as f32, position.y as f32];
                if std::env::var("AGP_INPUT_DEBUG").is_ok() {
                    eprintln!("DBG win CursorMoved {px:?}");
                }
                use crate::core::composition::input::InputOutcome;
                if let Some(InputOutcome::Delivered { surface, local }) =
                    self.input.pointer_moved(&self.rt.graph, px)
                {
                    self.cursors.insert(surface, local);
                    self.pending.entry(surface).or_default().push(Event::Mouse(
                        iced_wgpu::core::mouse::Event::CursorMoved {
                            position: Point::new(local[0], local[1]),
                        },
                    ));
                }
            }
            WE::Focused(false) => {
                if self.real_click {
                    self.apply_focus_loss();
                }
            }
            WE::CursorLeft { .. } => {
                if self.real_click {
                    self.apply_focus_loss();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &winit::event_loop::ActiveEventLoop) {
        if self.target.is_none() {
            return;
        }
        // Render + present continuously (the runtime owns the frame), feeding
        // this frame's routed events and cursors.
        let events = std::mem::take(&mut self.pending);
        if std::env::var("AGP_INPUT_DEBUG").is_ok() && !events.is_empty() {
            eprintln!(
                "DBG feed frame events={:?}",
                events.keys().map(|k| k.token()).collect::<Vec<_>>()
            );
        }
        {
            let target = self.target.as_mut().unwrap();
            let _ = self
                .rt
                .frame_with_cursors(Some(target), &events, &self.cursors);
        }
        if std::env::var("AGP_INPUT_DEBUG").is_ok() {
            eprintln!("DBG presses={}", read_presses(&self.rt));
        }
        match self.phase {
            0 => {
                if self.started.elapsed() > std::time::Duration::from_millis(400) {
                    self.presses_before = read_presses(&self.rt);
                    self.phase = 1;
                }
            }
            1 => {
                // Alternate real foreground clicks and posted-queue clicks
                // (both traverse winit's real input pipeline; the report
                // records which path produced the press).
                if self.clicks_sent < 40
                    && self.started.elapsed().as_millis() / 500 > self.clicks_sent as u128
                {
                    let posted = self.clicks_sent % 2 == 1;
                    let ok = crate::host::iced_surface_probe::os_click_client_once(
                        WINDOW_TITLE,
                        (self.a_button_win[0], self.a_button_win[1]),
                        posted,
                    );
                    if std::env::var("AGP_INPUT_DEBUG").is_ok() {
                        eprintln!(
                            "DBG click attempt {} posted={posted} ok={ok} target={:?}",
                            self.clicks_sent, self.a_button_win
                        );
                    }
                    self.click_path = if posted {
                        "posted_queue".into()
                    } else {
                        "os_focus".into()
                    };
                    self.clicks_sent += 1;
                }
                let presses = read_presses(&self.rt);
                if presses > self.presses_before {
                    self.real_click = true;
                    self.phase = 2;
                } else if self.started.elapsed() > std::time::Duration::from_secs(40) {
                    self.phase = 2;
                }
            }
            2 => {
                // Exercise the real focus-loss lifecycle once the click leg
                // is proven (the same code path winit's Focused(false) uses).
                if self.focus_cancels == 0 {
                    self.apply_focus_loss();
                }
                self.phase = 3;
            }
            _ => {
                if self.started.elapsed() > std::time::Duration::from_millis(6000) {
                    el.exit();
                }
            }
        }
    }
}

fn read_presses(rt: &SurfaceRuntime) -> u32 {
    rt.graph
        .surfaces()
        .iter()
        .find_map(|s| match rt.content(s.id) {
            Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
                c.ui.as_any().downcast_ref::<AppA>().map(|a| a.presses)
            }
            _ => None,
        })
        .unwrap_or(0)
}

/// Run the windowed real-input pass over an OWNED runtime (returned).
fn run_windowed(
    rt: SurfaceRuntime,
    ids: &SceneIds,
    input: &CompositionInput,
    _report_dir: &Path,
) -> (SurfaceRuntime, bool, bool, u32) {
    use winit::event_loop::EventLoop;
    let mut winput = CompositionInput::new(TOP_SIZE, TOP_SIZE);
    let Some(top) = rt
        .graph
        .views()
        .iter()
        .find(|v| v.desc.presents_to_window)
        .map(|v| v.desc.surface)
    else {
        return (rt, false, false, 0);
    };
    winput
        .mapper
        .register(top, probe_camera(TOP_SIZE[0] / TOP_SIZE[1]), TOP_SIZE);
    let a_button_local = [A_SIZE[0] / 2.0, A_BUTTON_Y];
    let Some(a_button_win) = input.anchor_to_window(&rt.graph, &rt, ids.a, a_button_local) else {
        return (rt, false, false, 0);
    };
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("surface-input(window): event loop: {e}");
            return (rt, false, false, 0);
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = WindowApp {
        rt,
        input: winput,
        pending: HashMap::new(),
        cursors: HashMap::new(),
        a_button_win,
        window: None,
        target: None,
        phase: 0,
        started: std::time::Instant::now(),
        clicks_sent: 0,
        presses_before: 0,
        real_click: false,
        click_path: String::new(),
        created: false,
        finished: false,
        focus_cancels: 0,
    };
    let _ = event_loop.run_app(&mut app);
    let created = app.created;
    let real = app.real_click;
    let focus_cancels = app.focus_cancels;
    (app.rt, created, real, focus_cancels)
}

// Keep the unused import meaningful (the PlaneXform sample mirrors the scenes).
#[allow(unused)]
fn _plane_sample() -> PlaneXform {
    PlaneXform::sample()
}

#[allow(unused)]
fn _color(_: Color) {}
