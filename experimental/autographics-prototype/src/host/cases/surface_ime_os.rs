//! Case `surface-ime-os` (PLAN-830 AC-05 OS-level IME evidence).
//!
//! Drives a REAL Windows Chinese IME (the system layout is zh-CN with an IME
//! installed) against a real iced `text_input` living inside a composed
//! surface on a real window:
//!
//! 1. open a window, enable IME (`Window::set_ime_allowed(true)`),
//! 2. focus the window and click the inner text_input through the composition
//!    router (so the text_input owns focus),
//! 3. keep the OS candidate window at the computed anchor
//!    (`Window::set_ime_cursor_area` with the mount-chain-projected caret),
//! 4. send REAL keystrokes ("nihao") and let the IME compose; commit with
//!    Space / a digit,
//! 5. assert the committed text is CJK (non-ASCII) and that a preedit was
//!    observed — proof the OS IME reached the nested control.
//!
//! Guards (no stray input): keystrokes are only sent while the probe window is
//! the FOREGROUND window (checked immediately before each send); otherwise the
//! run aborts without typing and reports the blocked prerequisite.

use crate::core::composition::{CompositionChange, MountKind};
use crate::host::composition_input::CompositionInput;
use crate::host::iced_surface::{SurfaceElement, SurfaceUi};
use crate::host::iced_surface_probe::probe_camera;
use crate::host::report::*;
use crate::host::surface_runtime::{IcedContent, Scene3dContent, SurfaceRuntime};
use crate::renderer::target::{OffscreenTarget, WindowTarget};
use crate::renderer::GpuDevice;
use iced::widget::{column, text_input};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const WINDOW_TITLE: &str = "AGP surface-ime-os";
pub const TOP_SIZE: [f32; 2] = [800.0, 600.0];
pub const UI_SIZE: [f32; 2] = [320.0, 160.0];

#[derive(Default)]
pub struct ImeUi {
    pub input: String,
    pub preedits: Vec<String>,
}

pub const IME_INPUT_ID: &str = "agp-surface-ime-input";

impl SurfaceUi for ImeUi {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![text_input("在这里输入", &self.input)
            .id(iced_wgpu::core::widget::Id::new(IME_INPUT_ID))
            .on_input(|s| s)
            .padding(8),]
        .padding(12)
        .width(iced::Length::Fixed(UI_SIZE[0]))
        .into()
    }
    fn update(&mut self, m: String) {
        self.input = m;
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

#[derive(Debug, Clone, serde::Serialize)]
struct ImeReport {
    adapter: crate::renderer::AdapterInfoReport,
    device_token: String,
    window_created: bool,
    /// "os_ime_commit" only when a real IME composition committed CJK text.
    ime_evidence: String,
    foreground_confirmed: bool,
    ime_enabled_event: bool,
    preedit_samples: Vec<String>,
    committed_text: String,
    is_cjk: bool,
    anchor_window_px: Option<[f32; 2]>,
    /// Projected screen rect of the WHOLE ui plane (the anchor must land in
    /// the caret region of THIS mount, not merely somewhere in the window).
    anchor_plane_rect: Option<[f32; 4]>,
    /// IME events dropped because no surface was focused (no fallback).
    ime_without_focus_dropped: u32,
    focus_cancels: u32,
    gates: Vec<GateResult>,
    all_gates_behaved: bool,
    failures: Vec<String>,
    blocked_reason: Option<String>,
    gpu_timing_available: bool,
    notes: Vec<String>,
}

fn gate(name: &str, ok: bool, reason: String) -> GateResult {
    GateResult {
        gate: name.into(),
        positive_passed: ok,
        negative_fired: false,
        reason,
    }
}

fn is_cjk(s: &str) -> bool {
    s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// Synthetic key event (logic-level evidence only; the OS IME evidence below
/// still comes from real keystrokes).
fn key_event(ch: &str) -> iced_wgpu::core::Event {
    let key = iced_wgpu::core::keyboard::Key::Character(iced_wgpu::core::SmolStr::new(ch));
    iced_wgpu::core::Event::Keyboard(iced_wgpu::core::keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: iced_wgpu::core::keyboard::key::Physical::Unidentified(
            iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
        ),
        location: iced_wgpu::core::keyboard::Location::Standard,
        modifiers: iced_wgpu::core::keyboard::Modifiers::default(),
        text: Some(iced_wgpu::core::SmolStr::new(ch)),
        repeat: false,
    })
}

fn feed_key(
    rt: &mut SurfaceRuntime,
    surface: crate::core::composition::SurfaceId,
    ev: iced_wgpu::core::Event,
    at: [f32; 2],
) {
    let mut events: HashMap<crate::core::composition::SurfaceId, Vec<iced_wgpu::core::Event>> =
        HashMap::new();
    events.entry(surface).or_default().push(ev);
    let mut cursors = HashMap::new();
    cursors.insert(surface, at);
    let _ = rt.frame_with_cursors(None, &events, &cursors);
}

fn type_text(
    rt: &mut SurfaceRuntime,
    surface: crate::core::composition::SurfaceId,
    text: &str,
    at: [f32; 2],
) {
    for ch in text.chars() {
        feed_key(rt, surface, key_event(&ch.to_string()), at);
    }
}

fn type_backspace(
    rt: &mut SurfaceRuntime,
    surface: crate::core::composition::SurfaceId,
    at: [f32; 2],
) {
    let key =
        iced_wgpu::core::keyboard::Key::Named(iced_wgpu::core::keyboard::key::Named::Backspace);
    let ev = iced_wgpu::core::Event::Keyboard(iced_wgpu::core::keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: iced_wgpu::core::keyboard::key::Physical::Unidentified(
            iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
        ),
        location: iced_wgpu::core::keyboard::Location::Standard,
        modifiers: iced_wgpu::core::keyboard::Modifiers::default(),
        text: None,
        repeat: false,
    });
    feed_key(rt, surface, ev, at);
}

/// True when `title`'s window is the current foreground window.
fn is_foreground(title: &str) -> bool {
    let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Fg {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    public static bool IsFg(string title) {
        IntPtr h = FindWindow(null, title);
        return h != IntPtr.Zero && h == GetForegroundWindow();
    }
}
"@
if ([Fg]::IsFg("TITLE")) { exit 0 } else { exit 1 }
"#
    .replace("TITLE", title);
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Send REAL keystrokes (keybd_event → hardware input queue → IME sees them).
/// Guarded: only sends when the probe window is foreground.
fn send_keys_guarded(keys: &str) -> bool {
    if !is_foreground(WINDOW_TITLE) {
        return false;
    }
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.SendKeys]::SendWait("KEYS")
"#
    .replace("KEYS", keys);
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("surface-ime-os: no GPU: {e}");
            return 3;
        }
    };
    let mut rt = match SurfaceRuntime::new(gpu) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("surface-ime-os: runtime: {e}");
            return 3;
        }
    };
    let adapter = rt.gpu().adapter_info().clone();
    let token = rt.gpu().token().to_report();

    // Scene: 3D top ─plane─> real iced text_input surface.
    let mut store = crate::render3d::MeshStore::default();
    let mesh_data = synth(crate::render3d::procedural::ground_mesh());
    let _ = store.get_or_upload(rt.gpu(), &mesh_data, 0);
    let scene = Scene3dContent {
        screen_overlay: None,
        text_overlay: None,
        mesh: crate::render3d::MeshPipeline::new(rt.gpu(), OffscreenTarget::FORMAT).expect("3d"),
        instances: vec![crate::render3d::MeshInstance {
            mesh: (
                mesh_data.source_sha256.clone(),
                mesh_data.meshes[0].name.clone(),
            ),
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
        .expect("top");
    let top_view = rt.add_view(top, true).expect("top view");
    let ui = rt
        .add_surface(
            UI_SIZE,
            crate::host::surface_runtime::SurfaceContent::Iced(Box::new(IcedContent::borrowed(
                rt.gpu(),
                UI_SIZE,
                Box::new(ImeUi::default()),
            ))),
        )
        .expect("ui");
    rt.add_view(ui, false).expect("ui view");
    let plane_mount = MountKind::Plane3d {
        center: [0.0, 1.7, 0.0],
        half: [2.4, 1.8],
    };
    let ui_portal = rt
        .add_portal(top_view, ui, plane_mount, true)
        .expect("plane");
    let mut router = CompositionInput::new(TOP_SIZE, TOP_SIZE);
    router
        .mapper
        .register(top, probe_camera(TOP_SIZE[0] / TOP_SIZE[1]), TOP_SIZE);

    // REAL caret: focus the control through the public operation and run one
    // frame; the focused text_input then reports `InputMethod::Enabled` with
    // the caret it computed from its layout/cursor (review r2 F03). The
    // headless typing below proves the anchor follows input/selection state.
    let focused = rt.focus_iced_text_input(ui, iced_wgpu::core::widget::Id::new(IME_INPUT_ID));
    let _ = rt.frame(None, &std::collections::HashMap::new());
    let caret_before = rt.ime_caret_local(ui);
    let caret_rect_local_before = rt.ime_caret_rect(ui);
    type_text(&mut rt, ui, "ab", caret_before.unwrap_or([0.0, 0.0]));
    let caret_after_typing = rt.ime_caret_local(ui);
    for _ in 0..2 {
        type_backspace(&mut rt, ui, caret_before.unwrap_or([0.0, 0.0]));
    }
    let caret_after_clear = rt.ime_caret_local(ui);
    let caret_moved = match (caret_before, caret_after_typing) {
        (Some(a), Some(b)) => b[0] > a[0] + 1.0,
        _ => false,
    };
    let caret_restored = match (caret_before, caret_after_clear) {
        (Some(a), Some(b)) => (a[0] - b[0]).abs() < 1.0,
        _ => false,
    };

    // Headless anchor evidence: the caret point read from the WIDGET must
    // project to a DIFFERENT window position when the mount moves, and both
    // anchors must land inside the projected CARET rect of their own mount
    // (review r1 F07 + r2 F03 — the plane/window is not a caret).
    let caret_local = caret_before.unwrap_or([0.0, 0.0]);
    let anchor_before = router.anchor_to_window(&rt.graph, &rt, ui, caret_local);
    let caret_rect_before = router.ime_caret_window_rect(&rt.graph, &rt, ui);
    let plane_rect_before = projected_plane_rect(&router, top, &plane_mount);
    let moved_mount = MountKind::Plane3d {
        center: [0.35, 1.7, 0.0],
        half: [2.4, 1.8],
    };
    rt.graph
        .submit_one(CompositionChange::SetMount(ui_portal, moved_mount))
        .expect("move mount");
    let anchor_after = router.anchor_to_window(&rt.graph, &rt, ui, caret_local);
    let caret_rect_after = router.ime_caret_window_rect(&rt.graph, &rt, ui);
    rt.graph
        .submit_one(CompositionChange::SetMount(ui_portal, plane_mount))
        .expect("restore mount");
    let anchor_moved = match (anchor_before, anchor_after) {
        (Some(a), Some(b)) => (a[0] - b[0]).abs() > 20.0,
        _ => false,
    };
    let inside = |a: Option<[f32; 2]>, r: Option<[f32; 4]>| match (a, r) {
        (Some(a), Some([x0, y0, x1, y1])) => {
            a[0] >= x0 - 2.0 && a[0] <= x1 + 2.0 && a[1] >= y0 - 2.0 && a[1] <= y1 + 2.0
        }
        _ => false,
    };
    // The projected caret must be a CARET (narrow) and strictly smaller than
    // the mount, so "anchor inside the plane" cannot stand in for it.
    let caret_is_caret = match (caret_rect_before, plane_rect_before) {
        (Some([cx0, cy0, cx1, cy1]), Some([px0, py0, px1, py1])) => {
            (cx1 - cx0) < 12.0
                && (cy1 - cy0) < 60.0
                && (cx1 - cx0) < (px1 - px0) * 0.5
                && (cy1 - cy0) < (py1 - py0) * 0.5
        }
        _ => false,
    };
    let mut pre_gates = vec![
        gate(
            "real_caret_reported_by_the_widget",
            focused && caret_before.is_some() && caret_rect_local_before.is_some(),
            format!(
                "focused={focused} caret={caret_before:?} caret_rect={caret_rect_local_before:?}"
            ),
        ),
        gate(
            "caret_follows_typing_and_scroll",
            caret_moved && caret_restored,
            format!(
                "caret {caret_before:?} -> typed={caret_after_typing:?} -> cleared={caret_after_clear:?}"
            ),
        ),
        gate(
            "anchor_tracks_the_active_mount",
            anchor_moved
                && inside(anchor_before, caret_rect_before)
                && inside(anchor_after, caret_rect_after),
            format!(
                "before={anchor_before:?} caret={caret_rect_before:?} after={anchor_after:?} caret={caret_rect_after:?}"
            ),
        ),
        gate(
            "anchor_region_is_the_caret_not_the_plane",
            caret_is_caret,
            format!(
                "caret_rect={caret_rect_before:?} plane_rect={plane_rect_before:?}"
            ),
        ),
    ];

    // Hand the runtime to the windowed driver (no second device).
    let (rt_back, facts) = run_windowed(rt, &router, ui, mode, report_dir, &mut pre_gates);
    let rt = rt_back;
    let mut facts = facts;
    facts.gates.splice(0..0, pre_gates);
    facts.all_pass = facts.gates.iter().all(|g| g.positive_passed);

    let report = ImeReport {
        adapter,
        device_token: token,
        window_created: facts.window_created,
        ime_evidence: if facts.committed_cjk {
            "os_ime_commit".into()
        } else {
            "none".into()
        },
        foreground_confirmed: facts.foreground,
        ime_enabled_event: facts.ime_enabled,
        preedit_samples: facts.preedits.clone(),
        committed_text: facts.text.clone(),
        is_cjk: facts.committed_cjk,
        anchor_window_px: facts.anchor,
        anchor_plane_rect: facts.anchor_rect,
        ime_without_focus_dropped: facts.ime_without_focus_dropped,
        focus_cancels: facts.focus_cancels,
        gates: facts.gates,
        all_gates_behaved: facts.all_pass,
        failures: facts.failures.clone(),
        blocked_reason: if facts.all_pass {
            None
        } else {
            Some(facts.blocked.clone())
        },
        gpu_timing_available: false,
        notes: vec![
            "keystrokes are only sent while the probe window is foreground (guarded)".into(),
            "the OS candidate anchor comes from the mount-chain projection, not the texture origin"
                .into(),
            "IME/keyboard events are delivered ONLY to a focused surface (no fallback)".into(),
            format!("replay={replay} mode={mode}"),
        ],
    };
    let exit = if report.all_gates_behaved {
        "pass"
    } else if facts.environment_blocked {
        "blocked"
    } else {
        "fail"
    };
    let _ = write_report(report_dir, "surface-ime-os", replay, exit, &report);
    rt.gpu().wait_idle();
    if report.all_gates_behaved {
        0
    } else if facts.environment_blocked {
        // 4 = blocked by environment (distinct from assertion failure)
        4
    } else {
        // 1 = product assertion failed in a usable environment
        1
    }
}

struct Facts {
    window_created: bool,
    foreground: bool,
    ime_enabled: bool,
    preedits: Vec<String>,
    text: String,
    committed_cjk: bool,
    anchor: Option<[f32; 2]>,
    anchor_rect: Option<[f32; 4]>,
    ime_without_focus_dropped: u32,
    focus_cancels: u32,
    environment_blocked: bool,
    gates: Vec<GateResult>,
    all_pass: bool,
    failures: Vec<String>,
    blocked: String,
}

struct App {
    rt: SurfaceRuntime,
    input: CompositionInput,
    ui_surface: crate::core::composition::SurfaceId,
    window: Option<Arc<winit::window::Window>>,
    target: Option<WindowTarget>,
    pending: HashMap<crate::core::composition::SurfaceId, Vec<iced_wgpu::core::Event>>,
    cursors: HashMap<crate::core::composition::SurfaceId, [f32; 2]>,
    started: Instant,
    phase: u32,
    /// Window position of the inner text_input (for the click and the anchor).
    input_win: [f32; 2],
    /// Projected RECTANGLE OF THE REAL CARET at the moment `anchor` was
    /// computed (the formal oracle — not the whole plane; review r2 F03).
    anchor_caret_rect: Option<[f32; 4]>,
    ime_enabled: bool,
    preedits: Vec<String>,
    foreground: bool,
    sent: u32,
    ascii_sent: bool,
    focus_tries: u32,
    focused_once: bool,
    focus_confirmed: bool,
    created: bool,
    failures: Vec<String>,
    anchor: Option<[f32; 2]>,
    /// IME commits delivered with NO focused surface (must be dropped).
    ime_without_focus_dropped: u32,
    /// Window focus-loss handler runs (capture+focus dropped).
    focus_cancels: u32,
    /// Text at the moment of the deliberate focus loss (must not change when
    /// an unfocused IME commit is offered afterwards).
    text_at_focus_loss: String,
    /// Whether the router had NO keyboard target immediately after the
    /// deliberate focus loss. A late real click may legitimately re-focus
    /// afterwards; that must not hide the lifecycle effect.
    focus_cleared_at_loss: bool,
    /// Whether the deliberately unfocused IME commit was (wrongly) delivered.
    unfocused_ime_delivered: bool,
}

impl App {
    fn text(&self) -> String {
        match self.rt.content(self.ui_surface) {
            Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
                c.ui.as_any()
                    .downcast_ref::<ImeUi>()
                    .map(|u| u.input.clone())
                    .unwrap_or_default()
            }
            _ => String::new(),
        }
    }

    /// Deliver an IME event to the FOCUSED surface only. No fallback: an
    /// unfocused composition is dropped and counted (review r1 F06).
    fn deliver_ime(&mut self, event: iced_wgpu::core::input_method::Event) -> bool {
        match self.input.keyboard_target() {
            Some(target) => {
                self.pending
                    .entry(target)
                    .or_default()
                    .push(iced_wgpu::core::Event::InputMethod(event));
                true
            }
            None => {
                self.ime_without_focus_dropped += 1;
                false
            }
        }
    }

    /// The SAME lifecycle work winit's `Focused(false)` / `CursorLeft` arms
    /// run: drop capture + focus, unfocus iced, close IME.
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

impl winit::application::ApplicationHandler for App {
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
        // IME must be explicitly enabled on winit windows.
        window.set_ime_allowed(true);
        self.window = Some(window.clone());
        match WindowTarget::new(
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
                self.failures.push(format!("target: {e}"));
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
        use crate::core::composition::input::InputOutcome;
        use winit::event::WindowEvent as WE;
        match event {
            WE::CloseRequested => {}
            WE::Ime(ime) => match ime {
                winit::event::Ime::Enabled => self.ime_enabled = true,
                winit::event::Ime::Preedit(s, _) => {
                    if !s.is_empty() {
                        self.preedits.push(s.clone());
                    }
                    self.deliver_ime(iced_wgpu::core::input_method::Event::Preedit(s, None));
                }
                winit::event::Ime::Commit(s) => {
                    self.deliver_ime(iced_wgpu::core::input_method::Event::Commit(s));
                }
                winit::event::Ime::Disabled => {
                    // Composition ended: tell the focused control, no fallback.
                    self.deliver_ime(iced_wgpu::core::input_method::Event::Closed);
                }
            },
            WE::Focused(false) => {
                self.apply_focus_loss();
            }
            WE::CursorLeft { .. } => {
                self.apply_focus_loss();
            }
            WE::ModifiersChanged(m) => {
                use iced_wgpu::core::keyboard::Modifiers as M;
                let st = m.state();
                let mut mods = M::empty();
                mods.set(M::SHIFT, st.shift_key());
                mods.set(M::CTRL, st.control_key());
                mods.set(M::ALT, st.alt_key());
                mods.set(M::LOGO, st.super_key());
                self.pending.entry(self.ui_surface).or_default().push(
                    iced_wgpu::core::Event::Keyboard(
                        iced_wgpu::core::keyboard::Event::ModifiersChanged(mods),
                    ),
                );
            }
            WE::KeyboardInput { event, .. } => {
                // Real keyboard → the FOCUSED iced text_input. With no focus
                // there is no valid destination: keys are dropped rather than
                // falling back to an old edit box (review r1 F06).
                let Some(target) = self.input.keyboard_target() else {
                    return;
                };
                if let Some(ev) = crate::host::iced_surface_probe::winit_key_to_iced(&event) {
                    self.pending
                        .entry(target)
                        .or_default()
                        .push(iced_wgpu::core::Event::Keyboard(ev));
                } else if event.state == winit::event::ElementState::Pressed {
                    if let Some(text) = event.text.as_ref() {
                        if !text.is_empty() {
                            let key = iced_wgpu::core::keyboard::Key::Character(
                                iced_wgpu::core::SmolStr::new(text.as_str()),
                            );
                            let ev = iced_wgpu::core::keyboard::Event::KeyPressed {
                                key: key.clone(),
                                modified_key: key,
                                physical_key:
                                    iced_wgpu::core::keyboard::key::Physical::Unidentified(
                                        iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
                                    ),
                                location: iced_wgpu::core::keyboard::Location::Standard,
                                modifiers: iced_wgpu::core::keyboard::Modifiers::default(),
                                text: Some(iced_wgpu::core::SmolStr::new(text.as_str())),
                                repeat: event.repeat,
                            };
                            self.pending
                                .entry(target)
                                .or_default()
                                .push(iced_wgpu::core::Event::Keyboard(ev));
                        }
                    }
                }
            }
            WE::CursorMoved { position, .. } => {
                let px = [position.x as f32, position.y as f32];
                if let Some(InputOutcome::Delivered { surface, local }) =
                    self.input.pointer_moved(&self.rt.graph, px)
                {
                    self.cursors.insert(surface, local);
                    self.pending
                        .entry(surface)
                        .or_default()
                        .push(iced_wgpu::core::Event::Mouse(
                            iced_wgpu::core::mouse::Event::CursorMoved {
                                position: iced_wgpu::core::Point::new(local[0], local[1]),
                            },
                        ));
                }
            }
            WE::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    match state {
                        winit::event::ElementState::Pressed => {
                            if let Some(px) = self.input.last_window_px {
                                if let Some(InputOutcome::Delivered { surface, local }) =
                                    self.input.pointer_pressed(&self.rt.graph, px)
                                {
                                    self.cursors.insert(surface, local);
                                    self.pending.entry(surface).or_default().push(
                                        iced_wgpu::core::Event::Mouse(
                                            iced_wgpu::core::mouse::Event::ButtonPressed(
                                                iced_wgpu::core::mouse::Button::Left,
                                            ),
                                        ),
                                    );
                                }
                            }
                        }
                        winit::event::ElementState::Released => {
                            if let Some(surface) = self.input.pointer_released() {
                                self.pending.entry(surface).or_default().push(
                                    iced_wgpu::core::Event::Mouse(
                                        iced_wgpu::core::mouse::Event::ButtonReleased(
                                            iced_wgpu::core::mouse::Button::Left,
                                        ),
                                    ),
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &winit::event_loop::ActiveEventLoop) {
        if self.target.is_none() {
            return;
        }
        let events = std::mem::take(&mut self.pending);
        {
            let target = self.target.as_mut().unwrap();
            let _ = self
                .rt
                .frame_with_cursors(Some(target), &events, &self.cursors);
        }
        match self.phase {
            0 => {
                // Focus our window and click the text_input through the router.
                if self.started.elapsed() > Duration::from_millis(300) {
                    let _ = crate::host::iced_surface_probe::os_click_client_once(
                        WINDOW_TITLE,
                        (self.input_win[0], self.input_win[1]),
                        false,
                    );
                    self.phase = 1;
                }
            }
            1 => {
                // Retry focus until our window is really foreground (the
                // desktop races; keystrokes stay withheld until it is).
                if !self.foreground
                    && self.started.elapsed().as_millis() / 700 > self.focus_tries as u128
                {
                    let _ = crate::host::iced_surface_probe::os_click_client_once(
                        WINDOW_TITLE,
                        (self.input_win[0], self.input_win[1]),
                        self.focus_tries % 2 == 1,
                    );
                    if let Some(w) = self.window.as_ref() {
                        w.focus_window();
                        w.set_ime_allowed(true);
                    }
                    self.focus_tries += 1;
                }
                self.foreground = is_foreground(WINDOW_TITLE);
                if self.started.elapsed() > Duration::from_secs(2) && !self.focused_once {
                    // Focus the inner text_input two ways: the public focus
                    // operation AND a routed click at its local point (the
                    // same path a real user click takes). The IME composition
                    // itself remains OS-level.
                    let _ = self.rt.focus_iced_text_input(
                        self.ui_surface,
                        iced_wgpu::core::widget::Id::new(IME_INPUT_ID),
                    );
                    // The SAME click path a user takes must establish the
                    // router's single keyboard target; later key/IME events
                    // are delivered only there.
                    let _ = self.input.pointer_pressed(&self.rt.graph, self.input_win);
                    let _ = self.input.pointer_released();
                    let local = [40.0f32, 26.0];
                    let mut ev: HashMap<_, Vec<iced_wgpu::core::Event>> = HashMap::new();
                    ev.entry(self.ui_surface).or_default().extend([
                        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::CursorMoved {
                            position: iced_wgpu::core::Point::new(local[0], local[1]),
                        }),
                        iced_wgpu::core::Event::Mouse(
                            iced_wgpu::core::mouse::Event::ButtonPressed(
                                iced_wgpu::core::mouse::Button::Left,
                            ),
                        ),
                        iced_wgpu::core::Event::Mouse(
                            iced_wgpu::core::mouse::Event::ButtonReleased(
                                iced_wgpu::core::mouse::Button::Left,
                            ),
                        ),
                    ]);
                    self.cursors.insert(self.ui_surface, local);
                    let _ = self.rt.frame_with_cursors(None, &ev, &self.cursors);
                    self.focused_once = true;
                    // Focus proven through the public probe (report evidence).
                    self.focus_confirmed = self.rt.iced_text_input_focused(
                        self.ui_surface,
                        iced_wgpu::core::widget::Id::new(IME_INPUT_ID),
                    );
                }
                if self.started.elapsed() > Duration::from_secs(4) {
                    // Anchor the OS candidate at the projected caret.
                    if let Some(w) = self.window.as_ref() {
                        let anchor = self.anchor.unwrap_or(self.input_win);
                        w.set_ime_cursor_area(
                            winit::dpi::PhysicalPosition::new(anchor[0], anchor[1]),
                            winit::dpi::PhysicalSize::new(2u32, 24u32),
                        );
                    }
                    self.phase = 2;
                }
            }
            2 => {
                // ASCII typing through REAL keystrokes (proves the input is
                // usable, independent of the IME).
                if self.foreground
                    && !self.ascii_sent
                    && self.started.elapsed() > Duration::from_secs(6)
                {
                    let _ = send_keys_guarded("abc");
                    self.ascii_sent = true;
                }
                if !self.text().is_empty() || self.started.elapsed() > Duration::from_secs(20) {
                    self.phase = 3;
                }
            }
            3 => {
                // IME composition: type "nihao" then commit with Space.
                if self.foreground
                    && self.sent < 24
                    && self.started.elapsed().as_millis() / 700 > self.sent as u128
                {
                    let ok = send_keys_guarded("nihao ");
                    if !ok {
                        self.foreground = false;
                    }
                    self.sent += 1;
                }
                if is_cjk(&self.text()) || self.started.elapsed() > Duration::from_secs(75) {
                    self.phase = 4;
                }
            }
            4 => {
                // Focus loss closes the edit/IME target; an IME commit with
                // no focused surface must be DROPPED (no fallback). Always
                // run the loss here: an earlier real blur may have been
                // followed by a click that re-focused the control.
                self.text_at_focus_loss = self.text();
                self.apply_focus_loss();
                self.focus_cleared_at_loss = self.input.keyboard_target().is_none();
                self.unfocused_ime_delivered = self.deliver_ime(
                    iced_wgpu::core::input_method::Event::Commit("X".to_string()),
                );
                self.phase = 5;
            }
            _ => {
                if self.started.elapsed() > Duration::from_millis(1200) {
                    el.exit();
                }
            }
        }
        // The candidate anchor follows the ACTUAL caret every frame: typing,
        // selection, scrolling and mount moves all change iced's reported
        // caret rectangle, and this is the point the OS is told about
        // (review r2 F03 — a fixed local point is not a caret).
        let fresh = self
            .input
            .ime_anchor_window(&self.rt.graph, &self.rt, self.ui_surface);
        let fresh_rect =
            self.input
                .ime_caret_window_rect(&self.rt.graph, &self.rt, self.ui_surface);
        if fresh.is_some() {
            self.anchor = fresh;
            self.anchor_caret_rect = fresh_rect;
        }
        if let (Some(w), Some(a)) = (self.window.as_ref(), self.anchor) {
            w.set_ime_cursor_area(
                winit::dpi::PhysicalPosition::new(a[0], a[1]),
                winit::dpi::PhysicalSize::new(2u32, 24u32),
            );
        }
    }
}

fn run_windowed(
    mut rt: SurfaceRuntime,
    router: &CompositionInput,
    ui: crate::core::composition::SurfaceId,
    _mode: &str,
    _report_dir: &Path,
    extra_gates: &mut Vec<GateResult>,
) -> (SurfaceRuntime, Facts) {
    use winit::event_loop::EventLoop;
    let Some(top) = rt
        .graph
        .views()
        .iter()
        .find(|v| v.desc.presents_to_window)
        .map(|v| v.desc.surface)
    else {
        return (rt, blocked_facts("no window view"));
    };
    let mut winput = CompositionInput::new(TOP_SIZE, TOP_SIZE);
    winput
        .mapper
        .register(top, probe_camera(TOP_SIZE[0] / TOP_SIZE[1]), TOP_SIZE);
    // The control to click and the caret to anchor at are READ FROM THE REAL
    // WIDGET (bounds via the public focusable traversal; caret via the
    // widget's own InputMethod::Enabled cursor), never a fixed local point.
    let Some(bounds) = rt.iced_widget_bounds(ui, iced_wgpu::core::widget::Id::new(IME_INPUT_ID))
    else {
        return (rt, blocked_facts("real text_input bounds unavailable"));
    };
    let control_center = [
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    ];
    let Some(input_win) = winput.anchor_to_window(&rt.graph, &rt, ui, control_center) else {
        return (rt, blocked_facts("anchor projection failed"));
    };
    let Some(anchor) = router.ime_anchor_window(&rt.graph, &rt, ui) else {
        return (
            rt,
            blocked_facts("no real caret after focusing the control"),
        );
    };
    let anchor_rect = router.ime_caret_window_rect(&rt.graph, &rt, ui);
    let Ok(event_loop) = EventLoop::new() else {
        return (rt, blocked_facts("event loop unavailable"));
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App {
        rt,
        input: winput,
        ui_surface: ui,
        window: None,
        target: None,
        pending: HashMap::new(),
        cursors: HashMap::new(),
        started: Instant::now(),
        phase: 0,
        input_win,
        ime_enabled: false,
        preedits: Vec::new(),
        foreground: false,
        sent: 0,
        ascii_sent: false,
        focus_tries: 0,
        focused_once: false,
        focus_confirmed: false,
        created: false,
        failures: Vec::new(),
        anchor: Some(anchor),
        anchor_caret_rect: anchor_rect,
        ime_without_focus_dropped: 0,
        focus_cancels: 0,
        text_at_focus_loss: String::new(),
        focus_cleared_at_loss: false,
        unfocused_ime_delivered: false,
    };
    let _ = event_loop.run_app(&mut app);
    // Acceptance captures: the inner UI texture and the composed top target
    // (the same pixels the window presented).
    let _ = std::fs::create_dir_all(_report_dir);
    if let Some((_, png, _, _)) = capture_surface(&mut app.rt, ui) {
        let _ = std::fs::write(_report_dir.join("ime_ui.png"), &png);
    }
    if let Some((_, png, _, _)) = capture_surface(&mut app.rt, top) {
        let _ = std::fs::write(_report_dir.join("ime_composite.png"), &png);
    }
    let text = app.text();
    let committed_cjk = is_cjk(&text);
    let renders = app.rt.slot(ui).map(|s| s.renders).unwrap_or(0);
    let frames = app.rt.stats.frames;
    let portals = app.rt.stats.portal_composites;
    let order_len = app.rt.last_order.len();
    let _ = (renders, frames, portals, order_len);
    let mut gates = std::mem::take(extra_gates);
    gates.push(gate(
        "window_created",
        app.created,
        format!("created={}", app.created),
    ));
    gates.push(gate(
        "ime_enabled_event",
        app.ime_enabled,
        "winit delivered Ime::Enabled after set_ime_allowed(true)".into(),
    ));
    // Real keystrokes must produce text. With the system IME active the
    // letters may be converted (e.g. "abc"→"爱彼此"), which still proves the
    // keystrokes reached the control.
    let ascii_ok = !text.is_empty();
    gates.push(gate(
        "inner_input_focused",
        app.focus_confirmed,
        "public focus probe: the nested text_input reported focused".into(),
    ));
    gates.push(gate(
        "real_keyboard_typing_lands",
        ascii_ok,
        format!("text={text:?} (real keystrokes; a broken keyboard bridge stays empty)"),
    ));
    gates.push(gate(
        "preedit_observed",
        !app.preedits.is_empty(),
        format!("preedits={:?}", app.preedits),
    ));
    gates.push(gate(
        "os_ime_commit_is_cjk",
        committed_cjk,
        format!("text={text:?}"),
    ));
    gates.push(gate(
        "anchor_projected_from_mount",
        anchor[0] >= 0.0
            && anchor[1] >= 0.0
            && anchor[0] <= TOP_SIZE[0]
            && anchor[1] <= TOP_SIZE[1],
        format!("anchor={anchor:?}"),
    ));
    // The anchor must land inside the caret REGION OF THIS MOUNT, not merely
    // somewhere in the window (review r1 F07).
    let anchor_in_region = match anchor_rect {
        Some([x0, y0, x1, y1]) => {
            anchor[0] >= x0 - 2.0
                && anchor[0] <= x1 + 2.0
                && anchor[1] >= y0 - 2.0
                && anchor[1] <= y1 + 2.0
        }
        None => false,
    };
    gates.push(gate(
        "anchor_inside_projected_caret_region",
        anchor_in_region,
        format!("anchor={anchor:?} projected CARET rect={anchor_rect:?}"),
    ));
    let focus_cancelled = app.focus_cancels >= 1 && app.focus_cleared_at_loss;
    gates.push(gate(
        "window_focus_loss_cancels_focus",
        focus_cancelled,
        format!(
            "focus_cancels={} cleared_at_loss={} final_keyboard_target={:?}",
            app.focus_cancels,
            app.focus_cleared_at_loss,
            app.input.keyboard_target().map(|s| s.token())
        ),
    ));
    let text_after_drop = app.text();
    gates.push(gate(
        "ime_without_focus_is_dropped",
        app.ime_without_focus_dropped >= 1
            && !app.unfocused_ime_delivered
            && text_after_drop == app.text_at_focus_loss,
        format!(
            "dropped={} delivered={} text_before={:?} text_after={:?}",
            app.ime_without_focus_dropped,
            app.unfocused_ime_delivered,
            app.text_at_focus_loss,
            text_after_drop
        ),
    ));
    let all_pass = gates.iter().all(|g| g.positive_passed);
    // Environment is genuinely usable only when the probe window was
    // foreground, IME was enabled, AND a real composition arrived. A failing
    // assertion in that state is a PRODUCT failure (exit 1), never exit 4
    // (review r1 F09 — the old path reported every failure as environmental).
    let environment_ok =
        app.created && app.foreground && app.ime_enabled && !app.preedits.is_empty();
    let environment_blocked = !all_pass && !environment_ok;
    let blocked = if all_pass {
        String::new()
    } else if !environment_ok {
        if !app.created {
            "probe window was never created".into()
        } else if !app.foreground {
            "probe window never became foreground; keystrokes withheld (no stray input)".into()
        } else if !app.ime_enabled {
            "IME was not enabled on the probe window (no system IME available)".into()
        } else {
            "IME did not deliver composition to this window (IMM/TSF path unavailable)".into()
        }
    } else {
        let failed: Vec<&str> = gates
            .iter()
            .filter(|g| !g.positive_passed)
            .map(|g| g.gate.as_str())
            .collect();
        format!("product assertion failed in a usable IME environment: {failed:?}")
    };
    (
        app.rt,
        Facts {
            window_created: app.created,
            foreground: app.foreground,
            ime_enabled: app.ime_enabled,
            preedits: app.preedits,
            text,
            committed_cjk,
            anchor: app.anchor,
            anchor_rect: app.anchor_caret_rect,
            ime_without_focus_dropped: app.ime_without_focus_dropped,
            focus_cancels: app.focus_cancels,
            environment_blocked,
            gates,
            all_pass,
            failures: app.failures,
            blocked,
        },
    )
}

fn capture_surface(
    rt: &mut SurfaceRuntime,
    id: crate::core::composition::SurfaceId,
) -> Option<(String, Vec<u8>, u32, u32)> {
    rt.capture(id, "ime")
}

fn blocked_facts(reason: &str) -> Facts {
    Facts {
        window_created: false,
        foreground: false,
        ime_enabled: false,
        preedits: Vec::new(),
        text: String::new(),
        committed_cjk: false,
        anchor: None,
        anchor_rect: None,
        ime_without_focus_dropped: 0,
        focus_cancels: 0,
        environment_blocked: true,
        gates: vec![gate("window_created", false, reason.into())],
        all_pass: false,
        failures: vec![reason.into()],
        blocked: reason.into(),
    }
}

/// Projected screen rect [x0,y0,x1,y1] of a plane mount's FULL UV square.
fn projected_plane_rect(
    router: &CompositionInput,
    parent: crate::core::composition::SurfaceId,
    mount: &MountKind,
) -> Option<[f32; 4]> {
    let mut xs = Vec::with_capacity(4);
    let mut ys = Vec::with_capacity(4);
    for uv in [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
        let p = router.mapper.project(parent, mount, uv)?;
        xs.push(p[0]);
        ys.push(p[1]);
    }
    Some([
        xs.iter().copied().fold(f32::INFINITY, f32::min),
        ys.iter().copied().fold(f32::INFINITY, f32::min),
        xs.iter().copied().fold(f32::NEG_INFINITY, f32::max),
        ys.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    ])
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
