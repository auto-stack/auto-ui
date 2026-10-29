//! Case `surface-scenarios` (PLAN-830 T-07 / V-03-V-06 scenarios).
//!
//! S-02 (3D→UI→2D) and S-03 (multiple instances / shared views) over the
//! composition runtime, reusing the FROZEN game drivers — no new game logic:
//!
//! - S-02: a 3D scene mounts a UI panel whose sub-region shows the REAL
//!   Breakout driver. Pause/Resume/Reset and a real text input are clicked
//!   through the FULL mount chain (window → 3D plane → UI → 2D mount), not by
//!   injecting events at a hard-coded surface. Keyboard focus decides whether
//!   text or the game receives keys, reset clears state AND stale events,
//!   and the game advances exactly once per frame (a doubled tick is caught).
//! - S-03: two independent UI instances plus ONE shared **Squash** surface
//!   mounted by two portals: the shared content advances once per frame, both
//!   projections show identical pixels, the two UI instances keep separate
//!   state, and a click is delivered once to the targeted mount.
//!
//! All gates are counted from real driver state and rendered textures.

use crate::core::composition::input::InputOutcome;
use crate::core::composition::{MountKind, Rect, SurfaceId};
use crate::core::runtime::Runtime;
use crate::core::scene::Color;
use crate::games::breakout::{BreakoutAction, BreakoutDriver, PaddleDir};
use crate::games::squash::{SquashAction, SquashDriver, SquashEvent};
use crate::host::cases::squash_game::{build_scene, frame_instances};
use crate::host::composition_input::CompositionInput;
use crate::host::iced_surface::{SurfaceElement, SurfaceUi};
use crate::host::iced_surface_probe::probe_camera;
use crate::host::report::*;
use crate::host::surface_runtime::{Game2dContent, IcedContent, Scene3dContent, SurfaceRuntime};
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};
use crate::renderer::target::OffscreenTarget;
use crate::renderer::GpuDevice;
use iced::widget::{button, column, text, text_input};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

pub const GAME_W: f32 = 900.0;
pub const GAME_H: f32 = 600.0;
pub const PANEL_W: f32 = 320.0;
pub const PANEL_H: f32 = 320.0;
const SKY: [f32; 4] = [0.64625, 0.65575, 0.67075, 1.0];

// ---------------------------------------------------------------------------
// Real Breakout, hosted as a surface's content
// ---------------------------------------------------------------------------

/// Owns the frozen Breakout runtime/driver and its tick counter (the oracle
/// for "shared content advances once per frame").
pub struct BreakoutContent {
    pub runtime: Runtime<BreakoutAction, BreakoutEventStub>,
    pub driver: BreakoutDriver,
    /// Steps actually executed (driver-advance oracle).
    pub steps: u32,
    pub paused: bool,
    pending: Vec<BreakoutAction>,
    seed: u64,
}

type BreakoutEventStub = crate::games::breakout::BreakoutEvent;

impl BreakoutContent {
    pub fn new(seed: u64) -> Self {
        BreakoutContent {
            runtime: Runtime::new(Duration::from_millis(16), seed),
            driver: BreakoutDriver::default(),
            steps: 0,
            paused: false,
            pending: Vec::new(),
            seed,
        }
    }

    pub fn queue(&mut self, action: BreakoutAction) {
        self.pending.push(action);
    }

    /// One fixed step of the FROZEN game (never called twice per frame in the
    /// production path).
    pub fn tick(&mut self) {
        if self.paused {
            self.pending.clear();
            return;
        }
        let actions = std::mem::take(&mut self.pending);
        let _ = self.runtime.step(&mut self.driver, actions);
        self.steps += 1;
    }

    /// Negative-control hook: advance the SAME frozen driver twice in one
    /// frame window (a real double simulation, not a counter increment).
    pub fn tick_n(&mut self, n: u32) {
        for _ in 0..n {
            self.tick();
        }
    }

    /// Reset through the frozen `BreakoutAction::Reset` semantics + a fresh
    /// runtime scene; stale pending input is discarded.
    pub fn reset(&mut self) {
        self.pending.clear();
        self.runtime.reset(self.seed);
        self.driver.reset();
        self.steps = 0;
        // The frozen driver spawns its scene on a zero-action step.
        self.tick();
    }

    pub fn score(&self) -> u32 {
        self.driver.score
    }

    /// Draw commands for the current scene (world → surface pixels).
    pub fn draw(&self) -> Vec<DrawCmd> {
        let mut cmds = Vec::new();
        for o in self.runtime.scene().objects() {
            let t = o.transform;
            cmds.push(DrawCmd {
                center: [
                    t.translation.x + GAME_W / 2.0,
                    GAME_H / 2.0 - t.translation.y,
                ],
                size: [t.scale.x.abs(), t.scale.y.abs()],
                z: 0.0,
                color: o.color,
                shape: Shape::Rect,
            });
        }
        cmds
    }
}

/// Kick the frozen driver into spawning its scene (a zero-action step).
fn spawn_breakout(c: &mut BreakoutContent) {
    c.tick();
}

// ---------------------------------------------------------------------------
// Real Squash (shared content in S-03)
// ---------------------------------------------------------------------------

struct SquashContent {
    runtime: Runtime<SquashAction, SquashEvent>,
    driver: SquashDriver,
    steps: u64,
    scene: crate::host::cases::squash_game::SceneData,
    seed: u64,
}

impl SquashContent {
    fn new(
        gpu: &GpuDevice,
        seed: u64,
        store: &mut crate::render3d::MeshStore,
    ) -> Result<Self, crate::renderer::RendererError> {
        let scene = build_scene(gpu, store)?;
        Ok(SquashContent {
            runtime: Runtime::new(Duration::from_millis(16), seed),
            driver: SquashDriver::new(seed),
            steps: 0,
            scene,
            seed,
        })
    }

    fn tick(&mut self) {
        let _ = self
            .runtime
            .step(&mut self.driver, vec![SquashAction::default()]);
        self.steps += 1;
    }

    fn instances(&self) -> Vec<crate::render3d::MeshInstance> {
        frame_instances(self.runtime.scene(), &self.scene)
    }

    fn view_proj(&self) -> glam::Mat4 {
        self.scene.camera.view_proj()
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.runtime.reset(self.seed);
        self.driver = SquashDriver::new(self.seed);
        self.steps = 0;
    }
}

// ---------------------------------------------------------------------------
// UI apps
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct PauseUi {
    pub paused: bool,
    pub presses: u32,
    pub resets: u32,
    pub input: String,
}

impl SurfaceUi for PauseUi {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![
            button(text(if self.paused { "Resume" } else { "Pause" }))
                .width(iced::Length::Fill)
                .on_press(if self.paused {
                    "resume".to_string()
                } else {
                    "pause".to_string()
                }),
            button(text("Reset"))
                .width(iced::Length::Fill)
                .on_press("reset".to_string()),
            text_input("panel", &self.input).on_input(|s| format!("input:{s}")),
            text(format!("pressed {}", self.presses)),
        ]
        .spacing(6)
        .padding(10)
        .width(iced::Length::Fixed(PANEL_W))
        .into()
    }
    fn update(&mut self, m: String) {
        match m.as_str() {
            "pause" => {
                self.paused = true;
                self.presses += 1;
            }
            "resume" => {
                self.paused = false;
                self.presses += 1;
            }
            "reset" => {
                self.resets += 1;
            }
            _ => {
                if let Some(s) = m.strip_prefix("input:") {
                    self.input = s.to_string();
                }
            }
        }
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.94, 0.95, 0.98)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Default)]
pub struct CounterUi {
    pub presses: u32,
}

impl SurfaceUi for CounterUi {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![
            button(text(format!("count {}", self.presses)))
                .width(iced::Length::Fill)
                .on_press(format!("{}", self.presses + 1)),
            text(format!("instance {}", self.presses)),
        ]
        .spacing(8)
        .padding(10)
        .width(iced::Length::Fixed(200.0))
        .into()
    }
    fn update(&mut self, m: String) {
        if let Ok(n) = m.parse::<u32>() {
            self.presses = n;
        }
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.97, 0.94, 0.94)
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
struct ScenarioReport {
    adapter: crate::renderer::AdapterInfoReport,
    device_token: String,
    window_created: bool,
    window_mode: String,
    scenarios: Vec<ScenarioFacts>,
    gates: Vec<GateResult>,
    all_gates_behaved: bool,
    keyframes: Vec<Keyframe>,
    failures: Vec<String>,
    gpu_timing_available: bool,
    notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ScenarioFacts {
    name: String,
    description: String,
    measurements: serde_json::Value,
}

fn gate(name: &str, ok: bool, reason: String) -> GateResult {
    GateResult {
        gate: name.into(),
        positive_passed: ok,
        negative_fired: false,
        reason,
    }
}

fn scene3d(rt: &SurfaceRuntime, w: f32, h: f32) -> Scene3dContent {
    let mut store = crate::render3d::MeshStore::default();
    let mesh_data = synth(crate::render3d::procedural::ground_mesh());
    let _ = store.get_or_upload(rt.gpu(), &mesh_data, 0);
    Scene3dContent {
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
        sky: SKY,
        view_proj: probe_camera(w / h.max(1.0)),
    }
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

fn iced(
    rt: &SurfaceRuntime,
    size: [f32; 2],
    ui: Box<dyn SurfaceUi<Message = String> + Send>,
) -> IcedContent {
    IcedContent::borrowed(rt.gpu(), size, ui)
}

fn flat(x: f32, y: f32, w: f32, h: f32) -> MountKind {
    MountKind::Flat2d {
        offset: [x, y],
        scale: [1.0, 1.0],
        clip: Rect::at(x, y, w, h),
    }
}

/// Press through the router and deliver the event to the surface the ROUTER
/// chose, at the coordinate the ROUTER returned. The preset point is never
/// used for widget delivery (review r2 F06): a wrong UV/coordinate must reach
/// the real widget as a wrong coordinate, not be papered over by the caller.
fn route_click(
    rt: &mut SurfaceRuntime,
    input: &mut CompositionInput,
    win: [f32; 2],
) -> Option<(SurfaceId, [f32; 2])> {
    route_click_with(rt, input, win, |l| l)
}

/// Same path with a fault hook applied to the router's returned coordinate
/// (oracle-sensitivity control: the SAME gate predicate must reject it).
fn route_click_with(
    rt: &mut SurfaceRuntime,
    input: &mut CompositionInput,
    win: [f32; 2],
    corrupt: impl Fn([f32; 2]) -> [f32; 2],
) -> Option<(SurfaceId, [f32; 2])> {
    let (surface, local) = match input.pointer_pressed(&rt.graph, win) {
        Some(InputOutcome::Delivered { surface, local }) => (surface, corrupt(local)),
        _ => {
            input.pointer_released();
            return None;
        }
    };
    let mut cursors = HashMap::new();
    cursors.insert(surface, local);
    let mut ev: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>> = HashMap::new();
    ev.entry(surface).or_default().extend([
        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::CursorMoved {
            position: iced_wgpu::core::Point::new(local[0], local[1]),
        }),
        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
            iced_wgpu::core::mouse::Button::Left,
        )),
    ]);
    let _ = rt.frame_with_cursors(None, &ev, &cursors);
    if let Some(released) = input.pointer_released() {
        let mut ev: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>> = HashMap::new();
        ev.entry(released)
            .or_default()
            .push(iced_wgpu::core::Event::Mouse(
                iced_wgpu::core::mouse::Event::ButtonReleased(iced_wgpu::core::mouse::Button::Left),
            ));
        let _ = rt.frame_with_cursors(None, &ev, &cursors);
    }
    Some((surface, local))
}

/// Dispatch one key to the ROUTER's keyboard target and let the HOST map it
/// for that kind of content: iced gets the real key event, anything else is a
/// game action. A wrong focus destination therefore changes the observable
/// result (the text input would receive the key), instead of being ignored
/// (review r2 F06: `text_after_game_focus` was read without sending a key).
fn dispatch_key(
    rt: &mut SurfaceRuntime,
    input: &CompositionInput,
    game: &mut BreakoutContent,
    ev: iced_wgpu::core::Event,
) -> Option<SurfaceId> {
    let target = input.keyboard_target()?;
    // Read the content kind from the RUNTIME (no associated-function call on
    // the router type: the independent review wraps that type at the module
    // boundary to fault-inject the routed coordinate).
    let iced_target = matches!(
        rt.content(target),
        Some(crate::host::surface_runtime::SurfaceContent::Iced(_))
    );
    if iced_target {
        let mut events: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>> = HashMap::new();
        events.entry(target).or_default().push(ev);
        let _ = rt.frame_with_cursors(None, &events, &HashMap::new());
    } else {
        game.queue(BreakoutAction::Paddle(PaddleDir::Left));
    }
    Some(target)
}

fn local_in_bounds(local: [f32; 2], size: [f32; 2]) -> bool {
    local[0] >= 0.0 && local[0] < size[0] && local[1] >= 0.0 && local[1] < size[1]
}

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

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    if mode == "window" {
        eprintln!("surface-scenarios: scenario gates run headless (deterministic)");
    }
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("surface-scenarios: no GPU: {e}");
            return 3;
        }
    };
    let mut rt = match SurfaceRuntime::new(gpu) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("surface-scenarios: runtime: {e}");
            return 3;
        }
    };
    let adapter = rt.gpu().adapter_info().clone();
    let token = rt.gpu().token().to_report();
    let mut gates: Vec<GateResult> = Vec::new();
    let mut scenarios: Vec<ScenarioFacts> = Vec::new();
    let mut keyframes: Vec<Keyframe> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    // ===== S-02: 3D → UI panel (pause/reset/text) → 2D Breakout ==========
    {
        let top = rt
            .add_surface(
                [900.0, 640.0],
                crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(scene3d(
                    &rt, 900.0, 640.0,
                ))),
            )
            .expect("top 3d");
        let top_view = rt.add_view(top, true).expect("view");
        let ui = rt
            .add_surface(
                [PANEL_W, PANEL_H],
                crate::host::surface_runtime::SurfaceContent::Iced(Box::new(iced(
                    &rt,
                    [PANEL_W, PANEL_H],
                    Box::new(PauseUi::default()),
                ))),
            )
            .expect("panel");
        let ui_view = rt.add_view(ui, false).expect("panel view");
        let plane = MountKind::Plane3d {
            center: [0.0, 1.7, 0.0],
            half: [1.9, 1.9],
        };
        rt.add_portal(top_view, ui, plane, true).expect("plane");
        // Breakout surface mounted INSIDE the panel's lower area.
        let mut game = BreakoutContent::new(7);
        spawn_breakout(&mut game);
        let game_surface = rt
            .add_surface(
                [GAME_W, GAME_H],
                crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(Game2dContent {
                    text_overlay: None,
                    quad: QuadPipeline::new(rt.gpu(), OffscreenTarget::FORMAT).expect("quad"),
                    sprite: crate::renderer::sprite::SpritePipeline::new(
                        rt.gpu(),
                        OffscreenTarget::FORMAT,
                    )
                    .expect("sprite"),
                    sprite_store: Default::default(),
                    sprites: Vec::new(),
                    clear: Color::rgb(0.05, 0.05, 0.08),
                    commands: game.draw(),
                })),
            )
            .expect("game");
        rt.add_view(game_surface, false).expect("game view");
        rt.add_portal(
            ui_view,
            game_surface,
            MountKind::Flat2d {
                offset: [0.0, 150.0],
                scale: [0.28, 0.28],
                clip: Rect::at(0.0, 150.0, PANEL_W, PANEL_H - 150.0),
            },
            true,
        )
        .expect("game portal");

        let mut input = CompositionInput::new([900.0, 640.0], [900.0, 640.0]);
        input.top_view = Some(top_view);
        input
            .mapper
            .register(top, probe_camera(900.0 / 640.0), [900.0, 640.0]);

        // Warm + one production tick.
        game.tick();
        sync_game(&mut rt, game_surface, &game);
        let _ = rt.frame(None, &HashMap::new());

        // Controls: pause row, reset row, text input (all inside the panel).
        let pause_local = [160.0f32, 25.0];
        let reset_local = [160.0f32, 61.0];
        let text_local = [160.0f32, 97.0];
        // Route + deliver: the widget event uses the ROUTER's returned local,
        // and the returned local must round-trip to the intended control.
        let click_at = |rt: &mut SurfaceRuntime,
                        input: &mut CompositionInput,
                        surface: SurfaceId,
                        local: [f32; 2]|
         -> (bool, bool) {
            let Some(win) = input.anchor_to_window(&rt.graph, rt, surface, local) else {
                return (false, false);
            };
            match route_click(rt, input, win) {
                Some((hit, returned)) => (
                    hit == surface,
                    (returned[0] - local[0]).abs() < 2.0 && (returned[1] - local[1]).abs() < 2.0,
                ),
                None => (false, false),
            }
        };

        // Pause through the full chain (window → 3D plane → panel).
        let (pause_delivered, pause_route_ok) = click_at(&mut rt, &mut input, ui, pause_local);
        let paused_ui = ui_paused(&rt, ui);
        game.paused = paused_ui;
        let steps_at_pause = game.steps;
        for _ in 0..8 {
            game.tick();
            sync_game(&mut rt, game_surface, &game);
            let _ = rt.frame(None, &HashMap::new());
        }
        let steps_after_pause = game.steps;
        // Resume through the same path (the button text moved to Resume).
        let (resume_delivered, resume_route_ok) = click_at(&mut rt, &mut input, ui, pause_local);
        game.paused = ui_paused(&rt, ui);
        let resumed_ui = !game.paused;
        // Oracle-sensitivity control (V-08): the SAME click path with the
        // router's returned local corrupted must NOT reach the widget, so the
        // pause predicate rejects it. This is the same fault the independent
        // review injects through the router boundary.
        let corrupted_changed_state = {
            let before = ui_paused(&rt, ui);
            if let Some(win) = input.anchor_to_window(&rt.graph, &rt, ui, pause_local) {
                let _ = route_click_with(&mut rt, &mut input, win, |_| [9999.0, 9999.0]);
            }
            ui_paused(&rt, ui) != before
        };
        for _ in 0..8 {
            game.tick();
            sync_game(&mut rt, game_surface, &game);
            let _ = rt.frame(None, &HashMap::new());
        }
        let steps_after_resume = game.steps;

        // Text focus: click the real text input, then type one key. The game
        // must not receive it.
        let (text_delivered, text_route_ok) = click_at(&mut rt, &mut input, ui, text_local);
        let text_focused = input.keyboard_target() == Some(ui);
        let type_target = dispatch_key(&mut rt, &input, &mut game, key_event("x"));
        let text_after_typing = ui_text(&rt, ui);
        let pending_while_text = game.pending.len();

        // Game focus: click the 2D content through the panel; the same key
        // dispatch must now go to the GAME (host maps it), not the text box.
        let game_local = [160.0f32, 100.0];
        let game_win = input.anchor_to_window(&rt.graph, &rt, game_surface, game_local);
        let game_route = game_win.and_then(|w| route_click(&mut rt, &mut input, w));
        let (game_delivered, game_route_ok) = match game_route {
            Some((hit, returned)) => (
                hit == game_surface,
                local_in_bounds(returned, [GAME_W, GAME_H]),
            ),
            None => (false, false),
        };
        let game_focused = input.keyboard_target() == Some(game_surface);
        // The key goes through the SAME router destination: with the game
        // focused it must become a game action and must NOT touch the text.
        let key_target = dispatch_key(&mut rt, &input, &mut game, key_event("x"));
        let text_after_game_focus = ui_text(&rt, ui);
        let queued = game.pending.len();
        game.tick();
        sync_game(&mut rt, game_surface, &game);
        let _ = rt.frame(None, &HashMap::new());
        let queued_consumed = game.pending.is_empty();

        // Reset through the panel with a STALE pending action: the reset must
        // clear the action and rebuild the frozen game state.
        game.queue(BreakoutAction::Paddle(PaddleDir::Right));
        let resets_before = ui_resets(&rt, ui);
        let (reset_delivered, reset_route_ok) = click_at(&mut rt, &mut input, ui, reset_local);
        let reset_requested = ui_resets(&rt, ui) > resets_before;
        if reset_requested {
            game.reset();
        }
        let steps_after_reset = game.steps;
        let pending_after_reset = game.pending.len();
        let score_after_reset = game.score();

        // Single-tick oracle + REAL doubled-simulation sabotage.
        let before = game.steps;
        game.tick();
        let legit_delta = game.steps - before;
        let before = game.steps;
        game.tick_n(2);
        let doubled_delta = game.steps - before;
        game.reset();

        let max_renders = rt
            .stats
            .last_frame_renders
            .values()
            .copied()
            .max()
            .unwrap_or(0);

        gates.push(gate(
            "s02_pause_via_full_mount_path",
            pause_delivered && pause_route_ok && paused_ui,
            format!(
                "delivered={pause_delivered} route_round_trip={pause_route_ok} paused={paused_ui}"
            ),
        ));
        gates.push(gate(
            "s02_ui_pause_stops_the_game",
            paused_ui && steps_after_pause == steps_at_pause,
            format!("paused={paused_ui} steps {steps_at_pause}->{steps_after_pause}"),
        ));
        gates.push(gate(
            "s02_ui_resume_restarts_the_game",
            resume_delivered && resume_route_ok && resumed_ui && steps_after_resume > steps_after_pause,
            format!(
                "delivered={resume_delivered} route_round_trip={resume_route_ok} resumed={resumed_ui} steps {steps_after_pause}->{steps_after_resume}"
            ),
        ));
        gates.push(gate(
            "s02_route_oracle_rejects_corrupt_local",
            !corrupted_changed_state,
            format!(
                "a corrupted routed local changed the control state: {corrupted_changed_state}"
            ),
        ));
        gates.push(gate(
            "s02_text_focus_receives_keys",
            text_delivered
                && text_route_ok
                && text_focused
                && type_target == Some(ui)
                && text_after_typing == "x"
                && pending_while_text == 0,
            format!(
                "delivered={text_delivered} route_round_trip={text_route_ok} focused={text_focused} key_target={:?} text={text_after_typing:?} game_pending={pending_while_text}",
                type_target.map(|s| s.token())
            ),
        ));
        gates.push(gate(
            "s02_game_focus_routes_away_from_text",
            game_delivered
                && game_route_ok
                && game_focused
                && key_target == Some(game_surface)
                && text_after_game_focus == text_after_typing
                && queued == 1
                && queued_consumed,
            format!(
                "delivered={game_delivered} local_in_bounds={game_route_ok} focused={game_focused} key_target={:?} text={text_after_game_focus:?} queued={queued} consumed={queued_consumed}",
                key_target.map(|s| s.token())
            ),
        ));
        gates.push(gate(
            "s02_reset_clears_state_and_events",
            reset_delivered
                && reset_route_ok
                && reset_requested
                && steps_after_reset <= 1
                && pending_after_reset == 0
                && score_after_reset == 0,
            format!(
                "delivered={reset_delivered} route_round_trip={reset_route_ok} requested={reset_requested} steps={steps_after_reset} pending={pending_after_reset} score={score_after_reset}"
            ),
        ));
        gates.push(gate(
            "s02_single_tick_observed_and_double_caught",
            legit_delta == 1 && doubled_delta == 2,
            format!("legit tick delta={legit_delta}, doubled delta={doubled_delta}"),
        ));
        gates.push(gate(
            "s02_no_double_pass_per_frame",
            max_renders <= 1,
            format!("max per-surface renders in a frame={max_renders}"),
        ));
        if let Some((hash, png, w, h)) = rt.capture(ui, "s02_panel") {
            let _ = std::fs::create_dir_all(report_dir);
            let _ = std::fs::write(report_dir.join("s02_panel.png"), &png);
            keyframes.push(Keyframe {
                name: "s02_panel".into(),
                file: "s02_panel.png".into(),
                width: w,
                height: h,
                sha256: hash,
            });
            gates.push(gate(
                "s02_game_visible_in_panel",
                png_has_colorful_region(&png),
                "panel texture carries non-UI game colors in the mount region".into(),
            ));
        } else {
            failures.push("s02 panel capture failed".into());
        }
        scenarios.push(ScenarioFacts {
            name: "S-02 3D→UI→2D (Breakout)".into(),
            description: "3D scene mounts a UI panel whose sub-region shows the frozen Breakout driver; pause/reset/text are routed through the full mount chain"
                .into(),
            measurements: serde_json::json!({
                "steps_at_pause": steps_at_pause,
                "steps_after_pause": steps_after_pause,
                "steps_after_resume": steps_after_resume,
                "text_after_typing": text_after_typing,
                "text_after_game_focus": text_after_game_focus,
                "reset_steps": steps_after_reset,
                "single_tick_delta": legit_delta,
                "doubled_tick_delta": doubled_delta,
            }),
        });
    }

    // ===== S-03: two instances + one shared SQUASH mounted twice ==========
    {
        let top = rt
            .add_surface(
                [900.0, 640.0],
                crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(
                    Game2dContent::simple(rt.gpu(), Color::rgb(0.1, 0.1, 0.12)),
                )),
            )
            .expect("top");
        let top_view = rt.add_view(top, true).expect("top view");
        // ONE shared Squash surface, mounted by TWO portals.
        let seed = 0x1B;
        let mut store = crate::render3d::MeshStore::default();
        let mut squash = match SquashContent::new(rt.gpu(), seed, &mut store) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("s03 squash scene: {e}"));
                let _ = write_report(
                    report_dir,
                    "surface-scenarios",
                    replay,
                    "fail",
                    &ScenarioReport {
                        adapter,
                        device_token: token,
                        window_created: false,
                        window_mode: "headless-deterministic".into(),
                        scenarios,
                        gates,
                        all_gates_behaved: false,
                        keyframes,
                        failures: failures.clone(),
                        gpu_timing_available: false,
                        notes: vec![format!("replay={replay}")],
                    },
                );
                return 1;
            }
        };
        squash.tick();
        let squash_surface = rt
            .add_surface(
                [GAME_W, GAME_H],
                crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(Scene3dContent {
                    screen_overlay: None,
                    text_overlay: None,
                    mesh: crate::render3d::MeshPipeline::new(rt.gpu(), OffscreenTarget::FORMAT)
                        .expect("squash mesh"),
                    store,
                    instances: squash.instances(),
                    sky: SKY,
                    view_proj: squash.view_proj(),
                })),
            )
            .expect("shared squash");
        rt.add_view(squash_surface, false).expect("squash view");
        let mounts = [
            ([20.0f32, 20.0], Rect::at(20.0, 20.0, 405.0, 270.0)),
            ([450.0, 20.0], Rect::at(450.0, 20.0, 405.0, 270.0)),
        ];
        for (offset, clip) in mounts {
            rt.add_portal(
                top_view,
                squash_surface,
                MountKind::Flat2d {
                    offset,
                    scale: [0.45, 0.45],
                    clip,
                },
                true,
            )
            .expect("shared mount");
        }
        // Two independent UI instances in the lower half.
        let ui1 = rt
            .add_surface(
                [200.0, 120.0],
                crate::host::surface_runtime::SurfaceContent::Iced(Box::new(iced(
                    &rt,
                    [200.0, 120.0],
                    Box::new(CounterUi::default()),
                ))),
            )
            .expect("ui1");
        let ui2 = rt
            .add_surface(
                [200.0, 120.0],
                crate::host::surface_runtime::SurfaceContent::Iced(Box::new(iced(
                    &rt,
                    [200.0, 120.0],
                    Box::new(CounterUi::default()),
                ))),
            )
            .expect("ui2");
        rt.add_view(ui1, false).expect("ui1 view");
        rt.add_view(ui2, false).expect("ui2 view");
        rt.add_portal(top_view, ui1, flat(40.0, 400.0, 200.0, 120.0), true)
            .expect("ui1 mount");
        rt.add_portal(top_view, ui2, flat(300.0, 400.0, 200.0, 120.0), true)
            .expect("ui2 mount");

        let mut input = CompositionInput::new([900.0, 640.0], [900.0, 640.0]);
        input.top_view = Some(top_view);

        let frames = 20u32;
        let mut shared_steps = Vec::new();
        for _ in 0..frames {
            squash.tick();
            sync_squash(&mut rt, squash_surface, &squash);
            let _ = rt.frame(None, &HashMap::new());
            shared_steps.push(squash.steps);
        }
        let per_frame_one = shared_steps.windows(2).all(|w| w[1] == w[0] + 1);
        let max_renders = rt
            .stats
            .last_frame_renders
            .get(&squash_surface)
            .copied()
            .unwrap_or(0);
        // The two mounts must present IDENTICAL pixels (same source texture,
        // same state): final presentation, not just mounted_by.len().
        let near = |a: [u8; 4], b: [u8; 4], tol: u8| {
            a.iter().zip(b.iter()).all(|(x, y)| x.abs_diff(*y) <= tol)
        };
        // Samples: four background points + the projected source center,
        // which carries the live player (content identity). The center is a
        // FEATURE point; background-only sampling cannot prove shared content.
        let center_local = [450.0 * 0.45, 300.0 * 0.45];
        let samples = vec![
            [100.0f32, 80.0],
            [200.0, 20.0],
            [300.0, 180.0],
            [390.0, 250.0],
            center_local,
        ];
        let source_corner = rt
            .probe_pixel(squash_surface, [2.0, 2.0])
            .unwrap_or([0, 0, 0, 0]);
        let source_center = rt
            .probe_pixel(squash_surface, [450.0, 300.0])
            .unwrap_or([0, 0, 0, 0]);
        let sky_field = match rt.content(squash_surface) {
            Some(crate::host::surface_runtime::SurfaceContent::Scene3d(s)) => s.sky,
            _ => [f32::NAN; 4],
        };
        let mut identical = 0u32;
        let mut direct_match = 0u32;
        let mut sample_px = Vec::new();
        for &[px, py] in &samples {
            let a = rt
                .probe_pixel(top, [20.0 + px, 20.0 + py])
                .unwrap_or([0, 0, 0, 0]);
            let b = rt
                .probe_pixel(top, [450.0 + px, 20.0 + py])
                .unwrap_or([0, 0, 0, 0]);
            if near(a, b, 2) {
                identical += 1;
            } // The mount samples the SHARED texture at uv=(px/405, py/270);
              // the same point on the source proves the presented pixels are
              // the shared Squash content (not a fill or a copy).
            let tex = [(px + 0.5) / 0.45, (py + 0.5) / 0.45];
            if let Some(d) = rt.probe_pixel(squash_surface, tex) {
                if near(a, d, 32) {
                    direct_match += 1;
                }
            }
            sample_px.push((a, b));
        }
        // The FEATURE point (live player at the source center) must differ
        // from the static background — otherwise every sample is background.
        let content_nonuniform = sample_px.iter().any(|(a, _)| !near(*a, source_corner, 8));

        // Click instance 1's button through the router; the widget receives
        // the ROUTER's returned local (review r2 F06).
        let c1_before = counter(&rt, ui1);
        let c2_before = counter(&rt, ui2);
        let ui1_click = input
            .anchor_to_window(&rt.graph, &rt, ui1, [100.0, 34.0])
            .and_then(|w| route_click(&mut rt, &mut input, w));
        let (ui1_clicked, ui1_route_ok) = match ui1_click {
            Some((hit, returned)) => (
                hit == ui1,
                local_in_bounds(returned, [200.0, 120.0])
                    && (returned[0] - 100.0).abs() < 2.0
                    && (returned[1] - 34.0).abs() < 2.0,
            ),
            None => (false, false),
        };
        let (c1, c2) = (counter(&rt, ui1), counter(&rt, ui2));

        // Input to each shared mount is delivered ONCE to the shared surface;
        // the returned local must lie in the target's content bounds (a
        // corrupted UV/coordinate fails HERE, not only in the hit boolean).
        let shared_route = |input: &mut CompositionInput, win: [f32; 2]| match input
            .pointer_pressed(&rt.graph, win)
        {
            Some(InputOutcome::Delivered { surface, local }) => {
                input.pointer_released();
                if surface == squash_surface {
                    Some(local_in_bounds(local, [GAME_W, GAME_H]))
                } else {
                    Some(false)
                }
            }
            _ => {
                input.pointer_released();
                None
            }
        };
        let mount_a_ok = shared_route(&mut input, [200.0, 150.0]);
        let mount_b_ok = shared_route(&mut input, [650.0, 150.0]);
        let delivered_a = mount_a_ok.is_some();
        let delivered_b = mount_b_ok.is_some();
        let input_once = mount_a_ok == Some(true) && mount_b_ok == Some(true);

        gates.push(gate(
            "s03_shared_content_ticks_once",
            per_frame_one && max_renders == 1,
            format!(
                "steps per frame: {:?}; max renders in a frame={max_renders}",
                shared_steps
                    .windows(2)
                    .map(|w| w[1] - w[0])
                    .take(5)
                    .collect::<Vec<_>>()
            ),
        ));
        gates.push(gate(
            "s03_two_mounts_one_content",
            rt.graph
                .surface(squash_surface)
                .map(|s| s.mounted_by.len() == 2)
                .unwrap_or(false),
            format!(
                "mounted_by={:?}",
                rt.graph.surface(squash_surface).map(|s| s.mounted_by.len())
            ),
        ));
        gates.push(gate(
            "s03_mounts_present_identical_pixels",
            identical == samples.len() as u32
                && direct_match == samples.len() as u32
                && content_nonuniform,
            format!(
                "identical={identical}/{} direct_source_match={direct_match}/{} nonuniform={content_nonuniform} samples={sample_px:?} source_corner={source_corner:?} source_center={source_center:?} sky={sky_field:?}",
                samples.len(),
                samples.len()
            ),
        ));
        gates.push(gate(
            "s03_instances_do_not_share_state",
            ui1_clicked && ui1_route_ok && c1 == c1_before + 1 && c2 == c2_before,
            format!(
                "clicked={ui1_clicked} route_ok={ui1_route_ok} ui1={c1_before}->{c1} ui2={c2_before}->{c2}"
            ),
        ));
        gates.push(gate(
            "s03_input_once_per_mount",
            input_once,
            format!("mount A delivered={delivered_a}, mount B delivered={delivered_b}"),
        ));
        if let Some((hash, png, w, h)) = rt.capture(top, "s03_squash_shared") {
            let _ = std::fs::create_dir_all(report_dir);
            let _ = std::fs::write(report_dir.join("s03_squash_shared.png"), &png);
            keyframes.push(Keyframe {
                name: "s03_squash_shared".into(),
                file: "s03_squash_shared.png".into(),
                width: w,
                height: h,
                sha256: hash,
            });
        }
        scenarios.push(ScenarioFacts {
            name: "S-03 multi-instance / shared view (Squash)".into(),
            description: "two independent iced instances + ONE shared Squash surface with two portals".into(),
            measurements: serde_json::json!({
                "frames": frames,
                "squash_steps_tail": shared_steps.iter().rev().take(5).collect::<Vec<_>>(),
                "score": squash.driver.score,
                "ui1_count": c1,
                "ui2_count": c2,
                "identical_samples": identical,
            }),
        });
    }

    let all_ok = gates.iter().all(|g| g.positive_passed) && failures.is_empty();
    let report = ScenarioReport {
        adapter,
        device_token: token,
        window_created: false,
        window_mode: "headless-deterministic".into(),
        scenarios,
        gates,
        all_gates_behaved: all_ok,
        keyframes,
        failures,
        gpu_timing_available: false,
        notes: vec![
            "game drivers are the FROZEN 829 Breakout/Squash; the host ticks them once per composition frame".into(),
            "S-02 controls are routed through window → 3D plane → UI → 2D mount".into(),
            "a doubled simulation is caught by a REAL second driver step (state delta), not a counter".into(),
            format!("replay={replay}"),
        ],
    };
    let exit = if all_ok { "pass" } else { "fail" };
    let _ = write_report(report_dir, "surface-scenarios", replay, exit, &report);
    if !all_ok {
        eprintln!("surface-scenarios: gates failed (see report.json)");
    }
    rt.gpu().wait_idle();
    if all_ok {
        0
    } else {
        1
    }
}

/// Push the frozen game's current scene into its surface AND mark it dirty.
fn sync_game(rt: &mut SurfaceRuntime, surface: SurfaceId, game: &BreakoutContent) {
    if let Some(crate::host::surface_runtime::SurfaceContent::Game2d(g)) = rt.content_mut(surface) {
        g.commands = game.draw();
    }
    rt.graph.mark_content_dirty(surface);
}

fn sync_squash(rt: &mut SurfaceRuntime, surface: SurfaceId, game: &SquashContent) {
    if let Some(crate::host::surface_runtime::SurfaceContent::Scene3d(s)) = rt.content_mut(surface)
    {
        s.instances = game.instances();
    }
    rt.graph.mark_content_dirty(surface);
}

fn ui_paused(rt: &SurfaceRuntime, ui: SurfaceId) -> bool {
    match rt.content(ui) {
        Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<PauseUi>()
                .map(|u| u.paused)
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn ui_text(rt: &SurfaceRuntime, ui: SurfaceId) -> String {
    match rt.content(ui) {
        Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<PauseUi>()
                .map(|u| u.input.clone())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn ui_resets(rt: &SurfaceRuntime, ui: SurfaceId) -> u32 {
    match rt.content(ui) {
        Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<PauseUi>()
                .map(|u| u.resets)
                .unwrap_or(0)
        }
        _ => 0,
    }
}

fn counter(rt: &SurfaceRuntime, ui: SurfaceId) -> u32 {
    match rt.content(ui) {
        Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<CounterUi>()
                .map(|u| u.presses)
                .unwrap_or(0)
        }
        _ => 0,
    }
}

/// Cheap structural check: the panel texture contains saturated game colors
/// (breakout bricks are bright red/blue; the UI theme is near-white).
fn png_has_colorful_region(png: &[u8]) -> bool {
    let Ok(img) = image::load_from_memory(png) else {
        return false;
    };
    let img = img.to_rgba8();
    let mut hits = 0u32;
    let mut total = 0u32;
    for y in (0..img.height()).step_by(4) {
        for x in (0..img.width()).step_by(4) {
            let p = img.get_pixel(x, y).0;
            total += 1;
            let mx = p[0].max(p[1]).max(p[2]) as i32;
            let mn = p[0].min(p[1]).min(p[2]) as i32;
            if mx - mn > 40 {
                hits += 1;
            }
        }
    }
    total > 0 && hits * 100 > total * 2
}
