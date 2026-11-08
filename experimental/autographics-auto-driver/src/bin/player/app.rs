//! The standalone player's winit application: one window, one game surface,
//! straight into Playing. All run-path decisions (admission, fixed-dt pump,
//! round restart, pause/blur input clearing, audio runs) stay inside the
//! shared Host; this shell only forwards real OS events and owns the exit.
use super::config::{self, PlayerConfig};
use super::qa::{Qa, Script};
use agp::renderer::target::WindowTarget;
use autographics_auto_driver::authoring::host::Host;
use autographics_auto_driver::authoring::play::State;
use autographics_auto_driver::authoring::storage::Storage;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::NamedKey,
    window::{Window, WindowId},
};

/// Keep winit's physical PixelDelta intact until Host performs the
/// physical-to-logical conversion (same handoff as the authoring bin).
fn iced_scroll_delta(delta: winit::event::MouseScrollDelta) -> iced_wgpu::core::mouse::ScrollDelta {
    match delta {
        winit::event::MouseScrollDelta::LineDelta(x, y) => {
            iced_wgpu::core::mouse::ScrollDelta::Lines { x, y }
        }
        winit::event::MouseScrollDelta::PixelDelta(v) => {
            let finite_f32 = |n: f64| {
                if n.is_finite() {
                    n.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32
                } else {
                    n as f32
                }
            };
            iced_wgpu::core::mouse::ScrollDelta::Pixels {
                x: finite_f32(v.x),
                y: finite_f32(v.y),
            }
        }
    }
}

pub struct App {
    config: PlayerConfig,
    root: PathBuf,
    qa: Option<Qa>,
    report: Option<PathBuf>,
    /// User-writable run-data scope: QA passes `--user-data-dir` explicitly;
    /// the product default is %APPDATA%/<game_id>. The readable player log
    /// lives here, never beside the (possibly read-only) release layout.
    user_data: PathBuf,
    host: Option<Host>,
    window: Option<Arc<Window>>,
    target: Option<WindowTarget>,
    cursor: [f32; 2],
    frames: u64,
    error: Option<String>,
    /// Last resize/DPI event, for the settle window before auto-resume.
    last_resize: Instant,
    /// QA only: throttled focus re-request while a focus pause persists (a
    /// game a user can see but that never regains focus cannot proceed; the
    /// product keeps waiting for the user instead of fighting for focus).
    last_focus_request: Instant,
}

/// Auto pauses are transient by construction: the pause already cleared held
/// keys and capture (the safety property), and the player resumes when its
/// concrete cause ends. A game Fault never matches the prefix and therefore
/// never auto-resumes.
fn maybe_auto_resume(h: &mut Host, reason_prefix: &str) -> Result<(), String> {
    let paused = h.play.as_ref().is_some_and(|p| {
        p.state == State::Paused
            && p.error
                .as_deref()
                .is_some_and(|r| r.starts_with(reason_prefix))
    });
    if paused {
        h.play.as_mut().unwrap().resume()
    } else {
        Ok(())
    }
}

impl App {
    pub fn new(
        config: PlayerConfig,
        root: PathBuf,
        qa: Option<Qa>,
        report: Option<PathBuf>,
        user_data: PathBuf,
    ) -> Self {
        Self {
            config,
            root,
            qa,
            report,
            user_data,
            host: None,
            window: None,
            target: None,
            cursor: [0.0; 2],
            frames: 0,
            error: None,
            last_resize: Instant::now(),
            last_focus_request: Instant::now(),
        }
    }

    /// The readable log beside the report/stderr: one append line per event,
    /// best effort — a logging failure never blocks the game.
    pub fn log(&self, line: &str) {
        let path = self.user_data.join("player-log.txt");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            use std::io::Write;
            let _ = writeln!(
                f,
                "{} {line}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            );
        }
    }

    /// Load the scene under the exe root, cross-check the profile against the
    /// scene's project API, and start the run — the named-rejection boundary
    /// for every bad layout: no window content exists before this passes.
    fn enter_game(&mut self, e: &ActiveEventLoop) -> Result<(), String> {
        self.log(&format!(
            "launch game_id={} profile={} scene={:?} exe_root={:?}",
            self.config.game_id, self.config.profile, self.config.scene, self.root
        ));
        let scene = self.config.scene_path(&self.root)?;
        let (doc, assets, code, _) =
            Storage::load_project(&scene, &Default::default())
                .map_err(|e| format!("scene load failed: {e}"))?;
        let expected = config::profile_api(&self.config.profile);
        let binding = doc
            .behavior
            .as_ref()
            .ok_or("scene has no project behavior; the player only runs profile games")?;
        if binding.api != expected {
            return Err(format!(
                "profile mismatch: player.json profile {:?} expects API {expected}, scene carries {:?}",
                self.config.profile, binding.api
            ));
        }
        let two_d = self.config.two_d();
        let (w, h) = (self.config.display.width, self.config.display.height);
        let mut host = Host::with_project_player(doc, assets, code, two_d, [w as f32, h as f32])?;
        let window = Arc::new(
            e.create_window(
                Window::default_attributes()
                    .with_title(&self.config.display.title)
                    .with_inner_size(winit::dpi::LogicalSize::new(w as f64, h as f64)),
            )
            .map_err(|e| e.to_string())?,
        );
        // Take the foreground at creation: a launcher process holds startup
        // foreground rights, and a game that opens unfocused collects a
        // focus-lost pause before the first frame.
        window.focus_window();
        let size = window.inner_size();
        host.resize(size.width, size.height);
        let target = WindowTarget::new(host.rt.gpu(), window.clone(), size.width, size.height)
            .map_err(|e| format!("{e:?}"))?;
        host.rt.set_window_format(target.format());
        host.start_selected_play()?;
        self.host = Some(host);
        self.window = Some(window);
        self.target = Some(target);
        self.log("run started (Playing)");
        Ok(())
    }

    fn fail(&mut self, e: &ActiveEventLoop, error: String) {
        self.error = Some(error);
        e.exit();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, e: &ActiveEventLoop) {
        if let Err(error) = self.enter_game(e) {
            self.log(&format!("fatal: {error}"));
            // The dialog is the double-click failure surface. Automated
            // entries (QA or AGE_PLAYER_HEADLESS) must fail non-interactively
            // so a hung dialog can never stall a run.
            if self.qa.is_none() && std::env::var("AGE_PLAYER_HEADLESS").as_deref() != Ok("1") {
                super::native::fatal_message(&self.config.display.title, &error);
            }
            self.fail(e, error);
        }
    }

    fn window_event(&mut self, e: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(h) = self.host.as_mut() else { return };
        match event {
            WindowEvent::CloseRequested => h.request_close(),
            WindowEvent::Resized(size) => {
                h.resize(size.width, size.height);
                self.last_resize = Instant::now();
                if let Some(t) = &mut self.target {
                    if let Err(error) = t.handle_resize(h.rt.gpu(), size.width, size.height) {
                        self.error = Some(format!("{error:?}"));
                        e.exit();
                        return;
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(w) = &self.window {
                    let s = w.inner_size();
                    h.resize(s.width, s.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = [position.x as f32, position.y as f32];
                h.pointer_move(self.cursor);
                if let Err(error) = maybe_auto_resume(h, "auto-Pause: cursor left") {
                    self.error = Some(format!("auto resume failed: {error}"));
                    e.exit();
                    return;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = iced_scroll_delta(delta);
                h.scroll(self.cursor, delta);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    h.pointer_down(self.cursor);
                } else {
                    h.pointer_up(self.cursor);
                }
            }
            WindowEvent::CursorLeft { .. } => {
                // The pointer leaving is not a focus loss: the game keeps
                // running, but no stale drag capture survives it (the same
                // capture-clearing contract as the editor, without the
                // editor's pause — a game must not stop because the mouse
                // crossed the window edge).
                h.cancel_camera_drag();
                h.input.pointer_left_window();
            }
            WindowEvent::Focused(false) => {
                h.input.window_focus_lost();
                h.blur();
            }
            WindowEvent::Focused(true) => {
                // No editor panel exists to click Resume on: the focus pause
                // ends when the player is looking at the game again.
                if let Err(error) = maybe_auto_resume(h, "auto-Pause: focus lost") {
                    self.error = Some(format!("auto resume failed: {error}"));
                    e.exit();
                    return;
                }
            }
            WindowEvent::Occluded(occluded) => {
                if occluded {
                    h.pause_play("auto-Pause: window occluded");
                } else if let Err(error) = maybe_auto_resume(h, "auto-Pause: window occluded") {
                    self.error = Some(format!("auto resume failed: {error}"));
                    e.exit();
                    return;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(qa) = &self.qa {
                    qa.keys_seen.set(qa.keys_seen.get() + 1);
                    qa.key_log.borrow_mut().push(format!(
                        "{:?} repeat={} state={:?} text={:?}",
                        event.logical_key, event.repeat, event.state, event.text
                    ));
                }
                // Esc is the player's named quit: the shared Host stop path
                // runs inside request_close, so audio/stream teardown is the
                // tested one. All other keys flow through the single router.
                if event.state == ElementState::Pressed
                    && !event.repeat
                    && event.logical_key == winit::keyboard::Key::Named(NamedKey::Escape)
                {
                    h.request_close();
                } else {
                    h.key(&event);
                }
            }
            WindowEvent::RedrawRequested => {
                // A resize/DPI pause ends once the window has stopped
                // resizing for a settle window (startup DPI changes land
                // here; the editor's manual Resume has no player equivalent).
                if self.last_resize.elapsed() > Duration::from_millis(600) {
                    if let Err(error) = maybe_auto_resume(h, "auto-Pause: resize/DPI") {
                        self.error = Some(format!("auto resume failed: {error}"));
                        e.exit();
                        return;
                    }
                }
                if self.qa.is_some()
                    && !self.window.as_ref().is_none_or(|w| w.has_focus())
                    && self.last_focus_request.elapsed() > Duration::from_secs(2)
                {
                    self.last_focus_request = Instant::now();
                    let focus_paused = h.play.as_ref().is_some_and(|p| {
                        p.state == State::Paused
                            && p.error
                                .as_deref()
                                .is_some_and(|r| r.starts_with("auto-Pause: focus lost"))
                    });
                    if focus_paused {
                        if let Some(w) = &self.window {
                            w.focus_window();
                        }
                    }
                }
                if let Err(error) = h.frame(self.target.as_mut()) {
                    self.error = Some(error);
                    e.exit();
                    return;
                }
                self.frames += 1;
                let qa_error = match (&mut self.qa, self.window.clone()) {
                    (Some(qa), Some(w)) => {
                        qa.step(h, &w, self.frames);
                        qa.error.clone()
                    }
                    _ => None,
                };
                if let Some(error) = qa_error {
                    self.error = Some(error);
                    e.exit();
                }
            }
            _ => {}
        }
        if h.closed {
            e.exit();
        }
    }

    fn about_to_wait(&mut self, e: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        e.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(16),
        ));
    }
}

/// One exit gate for every loop return: the QA report (if any) plus the
/// process status. A faulted run is a failure with the named reason.
pub fn finalize(mut app: App, loop_error: Option<String>) -> i32 {
    if let Some(error) = loop_error {
        app.error = Some(format!("native event loop failed: {error}"));
    }
    let passed = app.error.is_none() && app.qa.as_ref().is_none_or(Qa::passed);
    app.log(&format!(
        "exit passed={passed} error={:?} frames={}",
        app.error, app.frames
    ));
    if let Some(path) = &app.report {
        let audio = app
            .host
            .as_ref()
            .map(|h| h.audio_snapshot())
            .unwrap_or(json!({"manager": false}));
        let runner = std::env::current_exe().ok();
        let data = json!({
            "schema": "age-game-player-qa/1",
            "script": app.qa.as_ref().map(|q| q.script_name()),
            "records": app.qa.as_ref().map(|q| &q.records),
            "key_log": app.qa.as_ref().map(|q| q.key_log.borrow().clone()),
            "audio_final": audio,
            "frames": app.frames,
            "passed": passed,
            "error": app.error,
            "runner_path": runner,
            "runner_sha256": runner.as_ref().and_then(|p| std::fs::read(p).ok()).map(|b| agp::renderer::capture::sha256_hex(&b)),
        });
        if let Err(e) = std::fs::write(path, serde_json::to_vec_pretty(&data).unwrap()) {
            eprintln!("qa report unwritable: {e}");
        }
    }
    match (&app.error, &app.qa) {
        (Some(error), _) => {
            eprintln!("{error}");
            1
        }
        (None, Some(qa)) if !qa.passed() => {
            eprintln!("qa chain incomplete");
            1
        }
        _ => 0,
    }
}

pub fn run(launch: App) -> i32 {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = launch;
    let loop_error = event_loop.run_app(&mut app).err().map(|e| e.to_string());
    finalize(app, loop_error)
}

pub fn script_from(name: &str) -> Option<Script> {
    match name {
        "platformer-2d" | "2d" => Some(Script::TwoD),
        "platformer-3d" | "3d" => Some(Script::ThreeD),
        _ => None,
    }
}
