//! PLAN-048 UI-only public native window entry. One real OS window hosts the
//! generic basic-UI adapter (NodeUiApp over the pinned SDK Session) with REAL
//! stock Iced widgets. The adapter's offscreen render (real widget tree,
//! real event path) is blitted to the window swapchain through the AGP
//! fullscreen-blit (the same composition contract the runtime path uses).
//! Real OS input (SendInput / SetCursorPos via `agp::host::iced_surface_probe`,
//! archived PLAN-029 precedent) drives the native-input evidence rows:
//!   --probe          positive: focus + 2 real clicks on A, 1 on B, Tab and
//!                    Enter via real keyboard; verify Auto state; write one
//!                    JSON report; exit 0 on pass
//!   --sabotage-focus the same probe while denying window focus; MUST fail
//!   (default)        plain window (no automation, no report)
//! GPU/window-focus runs are single-instance.
use agp::host::iced_surface::IcedSurface;
use agp::host::iced_surface_probe as probe;
use agp::renderer::composition::FullScreenBlit;
use agp::renderer::{GpuDevice, WindowTarget};
use autographics_auto_driver::node_ui::{NodeUiApp, PlatformKey};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const TITLE: &str = "AGE node-ui (PLAN-048)";
const LOGICAL: [f32; 2] = [480.0, 900.0];
const ARTIFACT: &str =
    include_str!("../../../autographics-node2d-pilot/generated-node-ui/ui-ab/ui.json");
const WORLD: &str =
    include_str!("../../../autographics-node2d-pilot/generated-node-ui/ui-ab/world.at");

struct App {
    probe: bool,
    sabotage_focus: bool,
    report: PathBuf,
    window: Option<Arc<Window>>,
    target: Option<WindowTarget>,
    blit: Option<FullScreenBlit>,
    iced: Option<IcedSurface>,
    app: Option<NodeUiApp>,
    gpu: Option<GpuDevice>,
    events: Vec<iced_wgpu::core::Event>,
    cursor: [f32; 2],
    started: Instant,
    phase: usize,
    passed: bool,
    native_clicks: u32,
    native_keys: u32,
}

impl App {
    fn new(probe: bool, sabotage_focus: bool, report: PathBuf) -> Self {
        Self {
            probe,
            sabotage_focus,
            report,
            window: None,
            target: None,
            blit: None,
            iced: None,
            app: None,
            gpu: None,
            events: Vec::new(),
            cursor: [-1.0, -1.0],
            started: Instant::now(),
            phase: 0,
            passed: false,
            native_clicks: 0,
            native_keys: 0,
        }
    }

    /// logical widget bounds -> PHYSICAL client px for the OS click channel.
    fn client_px(&mut self, key: &str) -> Option<(f32, f32)> {
        let window = self.window.as_ref()?;
        let app = self.app.as_ref()?;
        let surface = self.iced.as_mut()?;
        let b = container_bounds(surface, app, key)?;
        let scale = window.scale_factor() as f32;
        Some((
            (b.x + b.width / 2.0) * scale,
            (b.y + b.height / 2.0) * scale,
        ))
    }

    fn verify_and_finish(&mut self, e: &ActiveEventLoop) {
        let report = match self.app.as_ref() {
            Some(app) => {
                let a = app.instance_index("Main/a").unwrap();
                let b_idx = app.instance_index("Main/b").unwrap();
                let count_a = app.texts(a)[1].clone();
                let count_b = app.texts(b_idx)[1].clone();
                // expected native chain: click A twice (+1 each), click B
                // once (+2), Tab moves the focus ring to A's button, Enter
                // activates it (+1) => a=3, b=12. The keyboard activation
                // through the SAME generated dispatcher is itself evidence
                // (real OS key events -> ring -> queue -> Session).
                self.passed = !self.probe
                    || (count_a == "3"
                        && count_b == "12"
                        && self.native_clicks >= 3
                        && (self.sabotage_focus || self.native_keys >= 1));
                serde_json::json!({
                    "title": TITLE,
                    "probe": self.probe,
                    "sabotage_focus": self.sabotage_focus,
                    "native_clicks": self.native_clicks,
                    "native_keys": self.native_keys,
                    "count_a": count_a,
                    "count_b": count_b,
                    "scale": self.window.as_ref().map(|w| w.scale_factor()),
                    "passed": self.passed,
                })
            }
            None => serde_json::json!({"passed": false, "reason": "no app"}),
        };
        std::fs::create_dir_all(&self.report).ok();
        std::fs::write(
            self.report.join("node-ui-native.json"),
            serde_json::to_string_pretty(&report).unwrap(),
        )
        .expect("report");
        eprintln!("NODE-UI NATIVE report: {report}");
        e.exit();
    }

    fn run_phase(&mut self, e: &ActiveEventLoop) {
        if !self.probe {
            return;
        }
        if self.sabotage_focus {
            if let Some(w) = &self.window {
                if w.has_focus() {
                    // steer focus away: the OS channel's foreground step then
                    // cannot deliver (029 positive-control discipline).
                    w.set_title(&format!("{TITLE} [unfocused]"));
                }
            }
        }
        let t = self.started.elapsed().as_millis();
        match (self.phase, t) {
            (0, t) if t >= 1500 => {
                self.phase = 1;
                if let Some(px) = self.client_px("Main/a#inc") {
                    probe::os_click_client_once(TITLE, px, false);
                    self.native_clicks += 1;
                }
            }
            (1, t) if t >= 2200 => {
                self.phase = 2;
                if let Some(px) = self.client_px("Main/a#inc") {
                    probe::os_click_client_once(TITLE, px, false);
                    self.native_clicks += 1;
                }
            }
            (2, t) if t >= 2900 => {
                self.phase = 3;
                if let Some(px) = self.client_px("Main/b#inc") {
                    probe::os_click_client_once(TITLE, px, false);
                    self.native_clicks += 1;
                }
            }
            (3, t) if t >= 3600 => {
                self.phase = 4;
                if !self.sabotage_focus {
                    probe::os_key_client_once(TITLE, 0x09); // VK_TAB
                    self.native_keys += 1;
                }
            }
            (4, t) if t >= 4200 => {
                self.phase = 5;
                if !self.sabotage_focus {
                    probe::os_key_client_once(TITLE, 0x0D); // VK_RETURN
                    self.native_keys += 1;
                }
            }
            (5, t) if t >= 5000 => {
                self.phase = 6;
                self.verify_and_finish(e);
            }
            _ => {}
        }
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
                    .with_inner_size(winit::dpi::LogicalSize::new(LOGICAL[0], LOGICAL[1])),
            )
            .unwrap(),
        );
        let gpu = GpuDevice::new().unwrap();
        let size = window.inner_size();
        let target =
            WindowTarget::new(&gpu, window.clone(), size.width.max(1), size.height.max(1)).unwrap();
        let blit = FullScreenBlit::new(gpu.device(), target.format());
        let scale = window.scale_factor() as f32;
        let iced = IcedSurface::new(
            &gpu,
            iced_wgpu::core::Size::new(LOGICAL[0], LOGICAL[1]),
            scale,
        )
        .expect("iced surface");
        let mut app = NodeUiApp::new(ARTIFACT, WORLD, "plan048-window").expect("node-ui app");
        app.start_run().expect("start run");
        self.window = Some(window);
        self.target = Some(target);
        self.blit = Some(blit);
        self.iced = Some(iced);
        self.app = Some(app);
        self.gpu = Some(gpu);
    }

    fn window_event(&mut self, e: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.verify_and_finish(e);
            }
            WindowEvent::Resized(size) => {
                let (Some(target), Some(iced), Some(gpu)) =
                    (self.target.as_mut(), self.iced.as_mut(), self.gpu.as_ref())
                else {
                    return;
                };
                let scale = window_scale_of(self.window.as_ref());
                let logical_w = size.width.max(1) as f32 / scale;
                let logical_h = size.height.max(1) as f32 / scale;
                let _ = target.handle_resize(gpu, size.width.max(1), size.height.max(1));
                let _ = iced.resize(gpu, iced_wgpu::core::Size::new(logical_w, logical_h));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = [position.x as f32, position.y as f32];
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let ev = match state {
                    ElementState::Pressed => iced_wgpu::core::mouse::Event::ButtonPressed(
                        iced_wgpu::core::mouse::Button::Left,
                    ),
                    ElementState::Released => iced_wgpu::core::mouse::Event::ButtonReleased(
                        iced_wgpu::core::mouse::Button::Left,
                    ),
                    _ => return,
                };
                self.events.push(iced_wgpu::core::Event::Mouse(ev));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Some(app) = self.app.as_mut() {
                        use winit::keyboard::Key as WKey;
                        use winit::keyboard::NamedKey as WNamed;
                        match &event.logical_key {
                            WKey::Named(WNamed::Tab) => {
                                app.key(PlatformKey::Tab);
                            }
                            WKey::Named(WNamed::Enter) => {
                                app.key(PlatformKey::Enter);
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let (Some(iced), Some(app), Some(target), Some(gpu), Some(blit)) = (
                    self.iced.as_mut(),
                    self.app.as_mut(),
                    self.target.as_mut(),
                    self.gpu.as_ref(),
                    self.blit.as_ref(),
                ) else {
                    return;
                };
                let scale = window_scale_of(self.window.as_ref());
                let logical_cursor =
                    iced_wgpu::core::Point::new(self.cursor[0] / scale, self.cursor[1] / scale);
                let cursor = iced_wgpu::core::mouse::Cursor::Available(logical_cursor);
                // 1) REAL widget pass: events in, messages applied, draw into
                //    the surface's own target (iced_wgpu Engine + Renderer).
                let _ = iced.frame(app, &self.events, cursor);
                self.events.clear();
                // 2) consume the queue (bounded; stale named-rejected).
                app.drain();
                // 3) blit the accepted frame into the swapchain and present.
                let _ = target.present_frame(gpu, |view| {
                    let mut encoder = gpu.device().create_command_encoder(
                        &iced_wgpu::wgpu::CommandEncoderDescriptor { label: None },
                    );
                    blit.blit(gpu.device(), &mut encoder, view, iced.content());
                    gpu.queue().submit([encoder.finish()]);
                });
                self.run_phase(e);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, e: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        e.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + std::time::Duration::from_millis(8),
        ));
    }
}

/// REAL container bounds via a public Operation (buttons are not focusable
/// in stock iced 0.14; the id'd adapter containers are the probe surface).
fn container_bounds(
    surface: &mut IcedSurface,
    app: &NodeUiApp,
    key: &str,
) -> Option<iced_wgpu::core::Rectangle> {
    struct Probe {
        want: iced_wgpu::core::widget::Id,
        bounds: Option<iced_wgpu::core::Rectangle>,
    }
    impl iced_wgpu::core::widget::Operation<()> for Probe {
        fn traverse(
            &mut self,
            operate: &mut dyn FnMut(&mut dyn iced_wgpu::core::widget::Operation<()>),
        ) {
            operate(self)
        }
        fn container(
            &mut self,
            id: Option<&iced_wgpu::core::widget::Id>,
            bounds: iced_wgpu::core::Rectangle,
        ) {
            if id == Some(&self.want) {
                self.bounds = Some(bounds);
            }
        }
    }
    let mut op = Probe {
        want: iced_wgpu::core::widget::Id::from(key.to_string()),
        bounds: None,
    };
    surface.operate(app, &mut op);
    op.bounds
}

fn window_scale_of(window: Option<&Arc<Window>>) -> f32 {
    window.map(|w| w.scale_factor() as f32).unwrap_or(1.0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let probe = args.iter().any(|a| a == "--probe");
    let sabotage = args.iter().any(|a| a == "--sabotage-focus");
    let report = args
        .iter()
        .position(|a| a == "--report")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("research/node-ui-native"));
    let mut app = App::new(probe, sabotage, report);
    EventLoop::new().unwrap().run_app(&mut app).unwrap();
    if probe && !app.passed {
        std::process::exit(1);
    }
}
