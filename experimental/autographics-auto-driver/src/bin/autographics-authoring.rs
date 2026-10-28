#![recursion_limit = "512"]

#[path = "authoring/cli.rs"]
mod cli;
#[path = "authoring_probe/assets.rs"]
mod native_assets;
#[path = "authoring_probe/character2d.rs"]
mod native_character2d;
#[path = "authoring_probe/character3d.rs"]
mod native_character3d;
#[path = "authoring_probe/hierarchy.rs"]
mod native_hierarchy;
#[path = "authoring_probe/mod.rs"]
mod native_interaction;
#[path = "authoring_probe/motion.rs"]
mod native_motion;
#[path = "authoring_probe/play.rs"]
mod native_play;
#[path = "authoring_probe/runtime_camera.rs"]
mod native_runtime_camera;
#[path = "authoring_probe/segment.rs"]
mod native_segment;
#[path = "authoring_probe/spatial.rs"]
mod native_spatial;
#[path = "authoring_probe/sweep.rs"]
mod native_sweep;
#[path = "authoring_probe/viewport2d.rs"]
mod native_viewport2d;
#[path = "authoring_probe/viewport3d.rs"]
mod native_viewport3d;
#[path = "authoring/probes.rs"]
mod probes;
use agp::renderer::target::WindowTarget;
use autographics_auto_driver::authoring::{document::Document, host::Host};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
// Keep winit's physical PixelDelta intact until Host performs the
// CompositionInput physical-to-logical conversion.
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
#[derive(Default)]
struct App {
    viewport2d_probe: Option<native_viewport2d::Probe>,
    viewport3d_probe: Option<native_viewport3d::Probe>,
    play_probe: Option<native_play::Probe>,
    character_probe: Option<native_character2d::CharacterProbe>,
    character3d_probe: Option<native_character3d::Character3dProbe>,
    runtime_camera_probe: Option<native_runtime_camera::RuntimeCameraProbe>,
    spatial_probe: Option<native_spatial::Probe>,
    asset_probe: Option<native_assets::Probe>,
    asset_save: Option<PathBuf>,
    wrong_asset: bool,
    host: Option<Host>,
    window: Option<Arc<Window>>,
    target: Option<WindowTarget>,
    cursor: [f32; 2],
    frames: u64,
    limit: Option<u64>,
    open: Option<PathBuf>,
    behavior: Option<String>,
    behavior_api: Option<autographics_auto_driver::authoring::behavior::Api>,
    report: Option<PathBuf>,
    error: Option<String>,
    probe: bool,
    interaction_probe: bool,
    interaction_fault: Option<String>,
    native_probe: Option<native_interaction::Probe>,
    hierarchy_probe: Option<native_hierarchy::Probe>,
    native_inputs: Vec<serde_json::Value>,
    phase: u8,
    phase_frame: u64,
    before: Option<Document>,
    before_pixels: Option<String>,
    passed: bool,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, e: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let result = (|| {
            let w = Arc::new(
                e.create_window(
                    Window::default_attributes()
                        .with_title("AutoGraphics Authoring")
                        .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0)),
                )
                .map_err(|e| e.to_string())?,
            );
            if self.probe
                || self.viewport2d_probe.is_some()
                || self.viewport3d_probe.is_some()
                || self.interaction_probe
                || self.play_probe.is_some()
                || self.character_probe.is_some()
                || self.character3d_probe.is_some()
                || self.spatial_probe.is_some()
                || self.asset_probe.is_some()
                || self.hierarchy_probe.is_some()
            {
                native_interaction::place_window(&w)?;
            }
            let mut h = if self.play_probe.is_some() && self.open.is_none() {
                native_play::fixture(self.play_probe.as_ref().unwrap().path())?
            } else {
                Host::new(
                    if self.asset_probe.is_some()
                        || self.play_probe.is_some()
                        || self.spatial_probe.is_some()
                    {
                        Document::empty("assets-native".into())
                    } else {
                        Document::sample()
                    },
                )?
            };
            if self.play_probe.is_some() && std::env::args().any(|v| v == "--drop-play-tick") {
                h.play_source_for_probe = Some(
                    autographics_auto_driver::authoring::play::SOURCE
                        .replace("sequence=next;", "sequence=next+1;"),
                );
            }
            if self.wrong_asset {
                h.asset_source_for_probe = Some(
                    autographics_auto_driver::authoring::preview::ASSET_SOURCE
                        .replace("return resources[i]", "return 0"),
                );
            }
            if let Some(p) = &self.asset_save {
                h.ui_mut().path = p.to_string_lossy().into();
            }
            if let Some(p) = &self.open {
                h.open(p)?;
                h.settle()?;
            }
            if let Some(path) = &self.behavior {
                if let Some(api) = self.behavior_api {
                    h.select_behavior_api(api)?;
                }
                h.select_behavior(Some(path.clone()))?;
            }
            let size = w.inner_size();
            h.resize(size.width, size.height);
            let target = WindowTarget::new(h.rt.gpu(), w.clone(), size.width, size.height)
                .map_err(|e| format!("{e:?}"))?;
            h.rt.set_window_format(target.format());
            if self.probe
                || self.viewport2d_probe.is_some()
                || self.viewport3d_probe.is_some()
                || self.interaction_probe
                || self.hierarchy_probe.is_some()
                || self.asset_probe.is_some()
            {
                let path = std::env::current_dir()
                    .map_err(|e| e.to_string())?
                    .join("research")
                    .join(format!("authoring-native-{}.agscene", std::process::id()));
                if self.asset_probe.is_none() {
                    h.ui_mut().path = path.to_string_lossy().into();
                }
            }
            self.host = Some(h);
            self.window = Some(w);
            self.target = Some(target);
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            self.error = Some(error);
            e.exit();
        }
    }
    fn window_event(&mut self, e: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(h) = self.host.as_mut() else { return };
        match event {
            WindowEvent::CloseRequested => h.request_close(),
            WindowEvent::Resized(size) => {
                if self.probe
                    || self.viewport2d_probe.is_some()
                    || self.viewport3d_probe.is_some()
                    || self.interaction_probe
                    || self.hierarchy_probe.is_some()
                    || self.asset_probe.is_some()
                    || self.play_probe.is_some()
                    || self.spatial_probe.is_some()
                {
                    self.native_inputs.push(serde_json::json!({"event":"resize","size":[size.width,size.height],"gesture":h.gesture_active(),"frame":self.frames}));
                }
                h.resize(size.width, size.height);
                if let Some(t) = &mut self.target {
                    if let Err(error) = t.handle_resize(h.rt.gpu(), size.width, size.height) {
                        self.error = Some(format!("{error:?}"));
                        e.exit();
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
                if (self.interaction_probe
                    || self.viewport2d_probe.is_some()
                    || self.viewport3d_probe.is_some())
                    && h.gesture_active()
                {
                    self.native_inputs.push(serde_json::json!({"event":"move","point":self.cursor,"revision":h.editor.stamp.revision}));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if self.asset_probe.is_some() || self.hierarchy_probe.is_some() {
                    self.native_inputs.push(serde_json::json!({"event":"wheel","frame":self.frames,"point":self.cursor}));
                }
                let delta = iced_scroll_delta(delta);
                h.scroll(self.cursor, delta);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Middle,
                ..
            } => {
                if state == ElementState::Pressed {
                    h.navigation_down(self.cursor);
                } else {
                    h.navigation_up(self.cursor);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => {
                if state == ElementState::Pressed {
                    h.navigation3d_down(self.cursor, true);
                } else {
                    h.navigation3d_up(self.cursor);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                h.pointer_down(self.cursor);
                if self.probe
                    || self.viewport2d_probe.is_some()
                    || self.viewport3d_probe.is_some()
                    || self.interaction_probe
                    || self.hierarchy_probe.is_some()
                    || self.asset_probe.is_some()
                    || self.play_probe.is_some()
                    || self.spatial_probe.is_some()
                {
                    self.native_inputs.push(serde_json::json!({"event":"down","point":self.cursor,"frame":self.frames,"play_phase":self.play_probe.as_ref().map(|p|p.phase),"selection":h.editor.selection,"revision":h.editor.stamp.revision,"history":h.editor.history()}));
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if self.probe
                    || self.viewport2d_probe.is_some()
                    || self.viewport3d_probe.is_some()
                    || self.interaction_probe
                    || self.hierarchy_probe.is_some()
                    || self.asset_probe.is_some()
                    || self.play_probe.is_some()
                    || self.spatial_probe.is_some()
                {
                    self.native_inputs.push(serde_json::json!({"event":"up","point":self.cursor,"frame":self.frames,"play_phase":self.play_probe.as_ref().map(|p|p.phase),"revision":h.editor.stamp.revision}));
                }
                match self.interaction_fault.take().as_deref() {
                    Some("cancel") => h.cancel_gesture(),
                    Some("resource-last") => h.fail_resource = true,
                    _ => {}
                }
                h.pointer_up(self.cursor);
            }
            WindowEvent::CursorLeft { .. } => {
                if self.probe
                    || self.viewport2d_probe.is_some()
                    || self.viewport3d_probe.is_some()
                    || self.interaction_probe
                    || self.hierarchy_probe.is_some()
                    || self.asset_probe.is_some()
                    || self.play_probe.is_some()
                    || self.spatial_probe.is_some()
                {
                    self.native_inputs.push(serde_json::json!({"event":"cursor-left","gesture":h.gesture_active(),"frame":self.frames}));
                }
                h.cancel_navigation();
                h.pause_play("auto-Pause: cursor left");
                h.cancel_gesture();
                h.input.pointer_left_window();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if self.probe
                    || self.viewport2d_probe.is_some()
                    || self.viewport3d_probe.is_some()
                    || self.asset_probe.is_some()
                    || self.play_probe.is_some()
                    || self.spatial_probe.is_some()
                {
                    self.native_inputs.push(serde_json::json!({"event":"key","key":format!("{:?}",event.logical_key),"text":event.text.as_deref(),"repeat":event.repeat,"state":format!("{:?}",event.state),"play_state":h.play.as_ref().map(|p|format!("{:?}",p.state)),"target":format!("{:?}",h.input.keyboard_target())}));
                }
                h.key(&event)
            }
            WindowEvent::Occluded(true) => {
                h.pause_play("auto-Pause: window occluded");
            }
            WindowEvent::Focused(false) => {
                if self.probe
                    || self.viewport2d_probe.is_some()
                    || self.viewport3d_probe.is_some()
                    || self.interaction_probe
                    || self.hierarchy_probe.is_some()
                    || self.asset_probe.is_some()
                    || self.play_probe.is_some()
                    || self.spatial_probe.is_some()
                {
                    self.native_inputs.push(serde_json::json!({"event":"focus-lost","gesture":h.gesture_active(),"frame":self.frames}));
                }
                h.input.window_focus_lost();
                h.blur();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = h.frame(self.target.as_mut()) {
                    self.error = Some(error);
                    e.exit();
                }
                if self.interaction_probe && !h.gesture_active() && h.fail_resource {
                    h.fail_resource = false;
                }
                self.frames += 1;
                if self.interaction_probe && self.frames % 15 == 0 {
                    if let Some(path) = &self.report {
                        let h = self.host.as_mut().unwrap();
                        let trace = serde_json::json!({"frames":self.frames,"inputs":self.native_inputs,"compile_us":h.editor.preview.compile_us,"document":h.editor.document,"selection":h.editor.selection,"revision":h.editor.stamp.revision,"history":h.editor.history(),"gesture":h.gesture_active(),"dirty":h.editor.dirty(),"error":h.ui().error,"resources":h.resource_snapshot()});
                        let _ = std::fs::write(
                            path.with_extension("live.json"),
                            serde_json::to_vec_pretty(&trace).unwrap(),
                        );
                        if self.frames % 60 == 0 {
                            if let Some((_, png, _, _)) = h.rt.capture(h.top, "interaction-live") {
                                let _ = std::fs::write(path.with_extension("live.png"), png);
                            }
                        }
                    }
                }
                if let Some(probe) = &mut self.viewport2d_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.viewport3d_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.native_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.spatial_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.hierarchy_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.asset_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.play_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.character_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.character3d_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Some(probe) = &mut self.runtime_camera_probe {
                    probe.step(
                        self.host.as_mut().unwrap(),
                        self.frames,
                        self.window.as_ref().unwrap(),
                    );
                    if probe.done {
                        self.passed = true;
                        e.exit();
                    }
                    if let Some(error) = &probe.error {
                        self.error = Some(error.clone());
                        e.exit();
                    }
                }
                if let Err(error) = self.probe_step() {
                    self.error = Some(format!("legacy phase {}: {error}", self.phase));
                    e.exit();
                }
                if self.probe && self.frames > 1400 {
                    self.error = Some(format!("native input timeout phase {}", self.phase));
                    e.exit();
                }
                if self.limit.is_some_and(|n| self.frames >= n) {
                    e.exit();
                }
            }
            _ => {}
        }
        if self.host.as_ref().is_some_and(|h| h.closed) {
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
fn main() {
    cli::run();
}

#[cfg(test)]
mod completion_tests {
    use super::*;
    #[test]
    fn play_completion_rejects_partial_and_preserves_failure_before_report() {
        for phase in [0, 1, 26] {
            let mut p = native_play::Probe::new(String::new(), false, None, false);
            p.phase = phase;
            let mut a = App {
                play_probe: Some(p),
                passed: true,
                ..Default::default()
            };
            a.finalize_checks();
            assert!(!a.passed);
            assert!(a.error.unwrap().contains("incomplete"));
        }
        let mut p = native_play::Probe::new(String::new(), false, None, false);
        p.phase = 27;
        p.done = true;
        let mut a = App {
            play_probe: Some(p),
            ..Default::default()
        };
        a.finalize_checks();
        assert!(a.passed);
        assert!(a.error.is_none());
        a.error = Some("actual pixel failure".into());
        a.finalize_checks();
        assert!(!a.passed);
        assert_eq!(a.error.as_deref(), Some("actual pixel failure"));
    }

    #[test]
    fn asset_completion_rejects_partial_and_preserves_actual_failure() {
        for phase in [0, 1, 19] {
            let mut p = native_assets::Probe::new(String::new(), false, None);
            p.done = true;
            p.phase = phase;
            let mut app = App {
                asset_probe: Some(p),
                passed: true,
                ..Default::default()
            };
            app.finalize_checks();
            assert!(!app.passed && app.error.unwrap().contains("incomplete"));
        }
        let mut p = native_assets::Probe::new(String::new(), false, None);
        p.done = true;
        p.phase = 20;
        let mut app = App {
            asset_probe: Some(p),
            ..Default::default()
        };
        app.finalize_checks();
        assert!(app.passed && app.error.is_none());
        app.asset_probe.as_mut().unwrap().error = Some("actual asset pixels".into());
        app.finalize_checks();
        assert!(!app.passed && app.error.unwrap().contains("actual asset pixels"));
    }
    #[test]
    fn every_early_return_and_partial_state_is_failure() {
        for frames in [0, 1, 40, 200] {
            let mut app = App {
                native_probe: Some(native_interaction::Probe::new(false, false)),
                frames,
                ..Default::default()
            };
            // A stale success flag from another mode must not bypass the native oracle.
            app.passed = true;
            app.native_probe
                .as_mut()
                .unwrap()
                .records
                .push(serde_json::json!({"case":"partial"}));
            app.finalize_checks();
            assert!(!app.passed);
            assert!(app.error.unwrap().contains("incomplete"));
        }
        let mut clean_early_close = App {
            native_probe: Some(native_interaction::Probe::new(false, false)),
            ..Default::default()
        };
        clean_early_close.finalize_checks();
        assert!(!clean_early_close.passed);
        assert!(clean_early_close.error.is_some());
    }
    #[test]
    fn complete_and_failed_oracles_propagate_before_report() {
        let mut complete = App {
            native_probe: Some(native_interaction::Probe::new(false, false)),
            ..Default::default()
        };
        complete.native_probe.as_mut().unwrap().done = true;
        complete.finalize_checks();
        assert!(complete.passed);
        assert!(complete.error.is_none());
        let mut failed = App {
            native_probe: Some(native_interaction::Probe::new(true, false)),
            error: Some("original oracle failure".into()),
            passed: true,
            ..Default::default()
        };
        failed.finalize_checks();
        assert!(!failed.passed);
        assert_eq!(failed.error.as_deref(), Some("original oracle failure"));
        let mut observation = App {
            interaction_probe: true,
            frames: 1,
            ..Default::default()
        };
        observation.finalize_checks();
        assert!(observation.error.is_none());
        assert!(!observation.passed);
    }
}

#[cfg(test)]
mod spatial_completion_tests {
    use super::*;
    #[test]
    fn spatial_completion_rejects_partial_and_preserves_actual_failure() {
        for (phase, done, expected) in [
            (0, false, false),
            (13, false, false),
            (14, false, false),
            (14, true, true),
        ] {
            let mut p = native_spatial::Probe::new(2, false, None);
            p.phase = phase;
            p.done = done;
            let mut a = App {
                spatial_probe: Some(p),
                ..Default::default()
            };
            a.finalize_checks();
            assert_eq!(a.passed, expected);
            if expected {
                a.error = Some("spatial pixels final-content failure".into());
                a.finalize_checks();
                assert!(!a.passed);
                assert_eq!(
                    a.error.as_deref(),
                    Some("spatial pixels final-content failure")
                );
            }
        }
    }
}

#[cfg(windows)]
fn working_set() -> usize {
    #[repr(C)]
    struct Counters {
        cb: u32,
        faults: u32,
        peak: usize,
        working: usize,
        pool_peak: usize,
        pool: usize,
        nonpool_peak: usize,
        nonpool: usize,
        page: usize,
        peak_page: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
    }
    #[link(name = "psapi")]
    unsafe extern "system" {
        fn GetProcessMemoryInfo(p: *mut std::ffi::c_void, c: *mut Counters, n: u32) -> i32;
    }
    let mut c: Counters = unsafe { std::mem::zeroed() };
    c.cb = std::mem::size_of::<Counters>() as u32;
    if unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) } != 0 {
        c.working
    } else {
        0
    }
}
#[cfg(not(windows))]
fn working_set() -> usize {
    0
}

#[cfg(test)]
mod viewport_scroll_handoff_tests {
    use super::*;
    #[test]
    fn winit_physical_pixel_handoff_reaches_logical_view_once() {
        let mut h = Host::new(Document::sample()).unwrap();
        h.resize(2400, 1600);
        h.frame(None).unwrap();
        let delta = winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(
            0.0, 80.0,
        ));
        h.scroll([600.0, 380.0], iced_scroll_delta(delta));
        assert!((h.view2d().zoom - 1.1).abs() < 1e-6);
        let prior = h.view2d();
        h.scroll(
            [600.0, 380.0],
            iced_scroll_delta(winit::event::MouseScrollDelta::PixelDelta(
                winit::dpi::PhysicalPosition::new(0.0, f64::NAN),
            )),
        );
        assert_eq!(h.view2d(), prior);
        h.scroll(
            [600.0, 380.0],
            iced_scroll_delta(winit::event::MouseScrollDelta::PixelDelta(
                winit::dpi::PhysicalPosition::new(0.0, f64::MAX),
            )),
        );
        assert!((h.view2d().zoom - (1.1f32 * 1.1f32.powi(8))).abs() < 1e-5);
    }
}
