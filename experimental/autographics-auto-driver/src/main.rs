use agp::renderer::target::WindowTarget;
use autographics_auto_driver::host::Host;
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
const TITLE: &str = "Auto Source Graphics";
struct App {
    host: Option<Host>,
    squash: bool,
    probe_session: u64,
    probe_game_keys: u64,
    text_space_events: u64,
    native_keys: Vec<String>,
    source: String,
    window: Option<Arc<Window>>,
    target: Option<WindowTarget>,
    cursor: [f32; 2],
    tick: u64,
    next: Instant,
    frames: u64,
    limit: Option<u64>,
    report: PathBuf,
    probe: bool,
    phase: u8,
    phase_frame: u64,
    passed: bool,
    failures: Vec<String>,
}
impl App {
    fn new(source: String, limit: Option<u64>, report: PathBuf, probe: bool) -> Self {
        Self {
            host: None,
            probe_session: 0,
            probe_game_keys: 0,
            text_space_events: 0,
            native_keys: vec![],
            squash: std::env::args()
                .collect::<Vec<_>>()
                .windows(2)
                .any(|p| p == ["--sample", "squash"]),
            source,
            window: None,
            target: None,
            cursor: [0.0; 2],
            tick: 0,
            next: Instant::now(),
            frames: 0,
            limit,
            report,
            probe,
            phase: 0,
            phase_frame: 0,
            passed: false,
            failures: vec![],
        }
    }
    fn finish(&mut self) {
        if let Some(h) = &mut self.host {
            self.failures.extend(h.errors.clone());
            if !self.probe {
                self.passed = self.failures.is_empty();
            }
            let _ = std::fs::create_dir_all(&self.report);
            if let Some((_, png, _, _)) = h.rt.capture(h.top, "auto-source") {
                let _ = std::fs::write(self.report.join("window.png"), png);
            }
            let mut data = serde_json::json!({"frames":self.frames,"ticks":self.tick,"script_metrics":h.scripts.iter().map(|s|serde_json::json!({"completed_ticks":s.completed_ticks,"calls":s.session.calls,"yielded":s.session.yielded,"last_sequence":s.publication.last().map(|p|p.sequence)})).collect::<Vec<_>>(),"source":self.source,"device":h.rt.gpu().adapter_info(),"window_input_probe":self.probe,"passed":self.passed,"pointer_deliveries":h.pointer_deliveries,"key_deliveries":h.key_deliveries,"sound_requests":h.sound_requests,"sounds_played":h.sounds_played,"input_method":"existing AGP helper; single posted OS click and real winit key events","errors":h.errors,"failures":self.failures});
            data["probe_phase"] = serde_json::json!(self.phase);
            data["text_space_events"] = serde_json::json!(self.text_space_events);
            data["native_keys"] = serde_json::json!(self.native_keys);
            data["keyboard_target"] = serde_json::json!(format!("{:?}", h.input.keyboard_target()));
            if let Some(agp::host::surface_runtime::SurfaceContent::Iced(ui)) =
                h.rt.content(h.panel)
            {
                if let Some(p) = ui
                    .ui
                    .as_any()
                    .downcast_ref::<agp::host::cases::surface_scenarios::PauseUi>()
                {
                    data["panel_text"] = serde_json::json!(p.input);
                }
            }
            if let Some(squash) = &h.squash {
                data["script_metrics"]=serde_json::json!(squash.scripts.iter().map(|s|serde_json::json!({"session":s.id(),"completed_ticks":s.completed_ticks,"calls":s.session.calls,"yielded":s.session.yielded,"last_sequence":s.publication.last().map(|p|p.sequence),"score":s.publication.last().map(|p|p.score),"dead":s.publication.last().map(|p|p.dead),"pumps":s.pump_us.len(),"max_pump_us":s.pump_us.iter().max()})).collect::<Vec<_>>());
                data["resource_metrics"] = squash.resource_metrics(h);
                data["music_plays"] = serde_json::json!(squash.music_plays);
                data["owned_target_bytes"] = serde_json::json!(h.rt.owned_target_bytes());
                data["upload_stats"] = serde_json::json!(h.rt.upload_stats());
            }
            let _ = std::fs::write(
                self.report.join("window.json"),
                serde_json::to_vec_pretty(&data).unwrap(),
            );
            if let Some(squash) = &mut h.squash {
                squash.close();
            }
            for s in &mut h.scripts {
                s.close();
            }
        }
    }
    fn squash_probe_step(&mut self) {
        if self.frames - self.phase_frame < 30 {
            return;
        }
        if self.frames % 120 == 0 {
            eprintln!(
                "squash probe phase={} text_space={} native_keys={:?}",
                self.phase, self.text_space_events, self.native_keys
            );
        }
        let h = self.host.as_mut().unwrap();
        match self.phase {
            0 => {
                let size = self.window.as_ref().unwrap().inner_size();
                let p = (
                    80.0 * size.width as f32 / 1200.0,
                    70.0 * size.height as f32 / 800.0,
                );
                std::thread::spawn(move || {
                    agp::host::iced_surface_probe::os_click_client_once(TITLE, p, true);
                });
                self.phase = 1;
            }
            1 if h.input.keyboard_target() == Some(h.game_surfaces[0]) => {
                h.squash.as_mut().unwrap().reset_one(0).unwrap();
                std::thread::spawn(|| {
                    agp::host::iced_surface_probe::os_key_client_once(TITLE, 0x44);
                    agp::host::iced_surface_probe::os_key_client_once(TITLE, 0x20);
                });
                self.phase = 2;
            }
            2 => {
                let q = h.squash.as_mut().unwrap();
                if !q.scripts.iter().all(|s| s.idle()) {
                    return;
                }
                let count = q.scripts[0]
                    .session
                    .int("status", &[autographics_auto_driver::Scalar::Int(5)])
                    .unwrap_or(-1);
                if count < 1 {
                    return;
                }
                if q.scripts[0]
                    .session
                    .float("state", &[autographics_auto_driver::Scalar::Int(0)])
                    .unwrap_or(0.0)
                    <= 0.0
                    || q.scripts[1]
                        .session
                        .float("state", &[autographics_auto_driver::Scalar::Int(0)])
                        .unwrap_or(1.0)
                        != 0.0
                    || count != 1
                    || q.scripts[1]
                        .session
                        .int("status", &[autographics_auto_driver::Scalar::Int(5)])
                        .unwrap_or(-1)
                        != 0
                {
                    self.failures.push("jump/instance routing".into());
                }
                self.probe_session = q.scripts[0].id();
                self.probe_game_keys = h.key_deliveries;
                let size = self.window.as_ref().unwrap().inner_size();
                let p = (
                    1000.0 * size.width as f32 / 1200.0,
                    100.0 * size.height as f32 / 800.0,
                );
                std::thread::spawn(move || {
                    agp::host::iced_surface_probe::os_click_client_once(TITLE, p, true);
                });
                self.phase = 3;
            }
            3 if h.input.keyboard_target() == Some(h.panel) => {
                self.probe_game_keys = h.key_deliveries;
                std::thread::spawn(|| {
                    agp::host::iced_surface_probe::os_key_client_once(TITLE, 0x5a);
                    agp::host::iced_surface_probe::os_key_client_once(TITLE, 0x20);
                });
                self.phase = 4;
            }
            4 => {
                if self.text_space_events == 0 {
                    return;
                }
                if let Some(agp::host::surface_runtime::SurfaceContent::Iced(ui)) =
                    h.rt.content(h.panel)
                {
                    if let Some(p) = ui
                        .ui
                        .as_any()
                        .downcast_ref::<agp::host::cases::surface_scenarios::PauseUi>()
                    {
                        if !p.input.contains('z') {
                            return;
                        }
                    }
                }
                let q = h.squash.as_mut().unwrap();
                if !q.scripts.iter().all(|s| s.idle()) {
                    return;
                }
                if h.key_deliveries != self.probe_game_keys
                    || q.scripts[0].id() != self.probe_session
                    || q.scripts[0]
                        .session
                        .int("status", &[autographics_auto_driver::Scalar::Int(5)])
                        .unwrap_or(-1)
                        != 1
                {
                    self.failures.push(format!(
                        "text focus: keys {}/{}, session {}/{}, jump_count {:?}",
                        h.key_deliveries,
                        self.probe_game_keys,
                        q.scripts[0].id(),
                        self.probe_session,
                        q.scripts[0]
                            .session
                            .int("status", &[autographics_auto_driver::Scalar::Int(5)])
                    ));
                }
                let size = self.window.as_ref().unwrap().inner_size();
                let p = (
                    80.0 * size.width as f32 / 1200.0,
                    70.0 * size.height as f32 / 800.0,
                );
                std::thread::spawn(move || {
                    agp::host::iced_surface_probe::os_click_client_once(TITLE, p, true);
                });
                self.phase = 5;
            }
            5 | 7 => {
                let q = h.squash.as_mut().unwrap();
                if h.input.keyboard_target() != Some(h.game_surfaces[0])
                    || !q.accepted[0].as_ref().is_some_and(|p| p.dead)
                {
                    return;
                }
                self.probe_session = q.scripts[0].id();
                let space = self.phase == 5;
                std::thread::spawn(move || {
                    if space {
                        agp::host::iced_surface_probe::os_key_client_repeat_once(TITLE, 0x20);
                    } else {
                        agp::host::iced_surface_probe::os_key_client_once(TITLE, 0x0d);
                    }
                });
                self.phase += 1;
            }
            6 | 8 => {
                let q = h.squash.as_mut().unwrap();
                if q.scripts[0].id() == self.probe_session
                    || !q.scripts.iter().all(|s| s.idle())
                    || q.accepted[0].is_none()
                {
                    return;
                }
                if q.scripts[0]
                    .session
                    .int("status", &[autographics_auto_driver::Scalar::Int(5)])
                    .unwrap_or(-1)
                    != 0
                    || q.accepted[0].as_ref().unwrap().score != 0
                    || q.accepted[0].as_ref().unwrap().dead
                    || q.music_plays != 1
                {
                    self.failures
                        .push("retry carried old jump/state or replayed music".into());
                }
                if self.phase == 6 {
                    self.phase = 7;
                } else {
                    self.passed = self.failures.is_empty() && h.errors.is_empty();
                    self.limit = Some(self.frames + 2);
                    self.phase = 9;
                }
            }
            _ => return,
        }
        self.phase_frame = self.frames;
    }
    fn probe_step(&mut self) {
        if !self.probe {
            return;
        }
        if self.squash {
            self.squash_probe_step();
            return;
        }
        let h = self.host.as_mut().unwrap();
        if self.frames - self.phase_frame < 30 {
            return;
        }
        match self.phase {
            0 => {
                let w = self.window.as_ref().unwrap().inner_size();
                let p = (
                    80.0 * w.width as f32 / 1200.0,
                    70.0 * w.height as f32 / 800.0,
                );
                std::thread::spawn(move || {
                    let _ = agp::host::iced_surface_probe::os_click_client_once(TITLE, p, true);
                });
                self.phase = 1;
            }
            1 if h.pointer_deliveries > 0 => {
                let p = h.scripts[0].publication.last().unwrap();
                if (p.objects[0].values[0] + 40.0).abs() > 0.01 {
                    self.failures.push(format!(
                        "routed pointer output x={}",
                        p.objects[0].values[0]
                    ));
                }
                let other = h.scripts[1].publication.last().unwrap();
                if other.objects[0].values[0] != 0.0 {
                    self.failures.push("instance isolation".into());
                }
                std::thread::spawn(|| agp::host::iced_surface_probe::os_type_text(TITLE, "d"));
                self.phase = 2;
            }
            2 if h.key_deliveries > 0 => {
                let w = self.window.as_ref().unwrap().inner_size();
                let p = (
                    1000.0 * w.width as f32 / 1200.0,
                    100.0 * w.height as f32 / 800.0,
                );
                std::thread::spawn(move || {
                    let _ = agp::host::iced_surface_probe::os_click_client_once(TITLE, p, true);
                });
                self.phase = 3;
            }
            3 if h.input.keyboard_target() == Some(h.panel) => {
                std::thread::spawn(|| agp::host::iced_surface_probe::os_type_text(TITLE, "z"));
                self.phase = 4;
            }
            4 => {
                if let Some(agp::host::surface_runtime::SurfaceContent::Iced(ui)) =
                    h.rt.content(h.panel)
                {
                    let p = ui
                        .ui
                        .as_any()
                        .downcast_ref::<agp::host::cases::surface_scenarios::PauseUi>()
                        .unwrap();
                    if !p.input.contains('z') {
                        return;
                    }
                }
                if let Some(p) = h.scripts[0].publication.last() {
                    if (p.objects[0].values[0] + 37.0).abs() > 0.01 {
                        self.failures
                            .push(format!("key/focus Auto state x={}", p.objects[0].values[0]));
                    }
                }
                if h.key_deliveries != 1 {
                    self.failures
                        .push(format!("text focus leaked keys: {}", h.key_deliveries));
                }
                self.passed = self.failures.is_empty() && h.errors.is_empty();
                self.limit = Some(self.frames + 2);
                self.phase = 5;
            }
            _ => return,
        }
        self.phase_frame = self.frames;
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, e: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            e.create_window(
                Window::default_attributes()
                    .with_title(TITLE)
                    .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0)),
            )
            .unwrap(),
        );
        let mut host = if self.squash {
            Host::new_squash(&self.source)
        } else {
            Host::new(&self.source)
        }
        .unwrap();
        host.enable_audio().expect("audio backend");
        let size = window.inner_size();
        host.resize(size.width, size.height);
        host.bypass_focus = std::env::args().any(|s| s == "--sabotage-focus");
        let target =
            WindowTarget::new(host.rt.gpu(), window.clone(), size.width, size.height).unwrap();
        host.rt.set_window_format(target.format());
        self.window = Some(window);
        self.target = Some(target);
        self.host = Some(host);
        self.next = Instant::now();
    }
    fn window_event(&mut self, e: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.finish();
                e.exit()
            }
            WindowEvent::Resized(s) => {
                let h = self.host.as_mut().unwrap();
                h.resize(s.width, s.height);
                let _ = self
                    .target
                    .as_mut()
                    .unwrap()
                    .handle_resize(h.rt.gpu(), s.width, s.height);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = [position.x as f32, position.y as f32]
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.host.as_mut().unwrap().click(self.cursor);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if self.native_keys.len() < 32 {
                    self.native_keys
                        .push(format!("{:?} {:?}", event.logical_key, event.state));
                }
                let h = self.host.as_mut().unwrap();
                if self.squash
                    && h.input.keyboard_target() == Some(h.panel)
                    && event.logical_key
                        == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space)
                    && event.state == ElementState::Pressed
                {
                    self.text_space_events += 1;
                }
                h.key(&event);
            }
            WindowEvent::Focused(false) => {
                let h = self.host.as_mut().unwrap();
                h.input.window_focus_lost();
                h.direction = 0;
                if let Some(squash) = &mut h.squash {
                    squash.clear_input();
                }
                h.rt.release_iced_focus(h.panel);
            }
            WindowEvent::RedrawRequested => {
                let h = self.host.as_mut().unwrap();
                for _ in 0..4 {
                    if Instant::now() < self.next {
                        break;
                    }
                    h.tick(self.tick);
                    self.tick += 1;
                    self.next +=
                        Duration::from_nanos(if self.squash { 8_333_333 } else { 16_666_667 });
                }
                if self.squash
                    && Instant::now().saturating_duration_since(self.next)
                        > Duration::from_millis(32)
                {
                    self.next = Instant::now();
                }
                if let Err(error) = h.frame(self.target.as_mut()) {
                    self.failures.push(error);
                    self.finish();
                    e.exit();
                    return;
                }
                self.frames += 1;
                self.probe_step();
                if self.limit.is_some_and(|v| self.frames >= v) || self.probe && self.frames > 1200
                {
                    self.finish();
                    e.exit();
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, e: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        e.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(8),
        ));
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let value = |name: &str| {
        args.iter()
            .position(|v| v == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let probe = args.iter().any(|s| s == "--window-probe");
    let source = if let Some(p) = value("--source") {
        std::fs::read_to_string(p).expect("source.at")
    } else if value("--sample").as_deref() == Some("squash") {
        autographics_auto_driver::squash::source()
    } else if value("--sample").as_deref() == Some("scene") || probe {
        include_str!("../samples/scene-control/main.at").to_owned()
    } else {
        include_str!("../samples/breakout/main.at").to_owned()
    };
    let report = value("--report")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("driver-report"));
    let mut app = App::new(
        source,
        value("--frames").map(|v| v.parse().unwrap()),
        report,
        probe,
    );
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app).unwrap();
    let failed = !app.failures.is_empty() || probe && !app.passed;
    drop(app);
    if failed {
        std::process::exit(1)
    }
}
