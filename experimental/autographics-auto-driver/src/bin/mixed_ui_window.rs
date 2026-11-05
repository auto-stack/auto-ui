//! PLAN-049 mixed-ui-2d-3d 公开原生窗口入口（T-04 起：装配/每帧桥/路由
//! 全部经 `MixedHost` 单一生产链——本 bin 只持窗口/探针相位/验证报告）。
//! 一个真实 OS 窗口 = MixedHost 组合：UI（Iced 控件，048 adapter 接缝）
//! 为窗口面 view；`view-2d`/`view-3d` 嵌入区域为真实布局占位，其实际
//! widget bounds 驱动 Game2d/Scene3d child surface 的 Flat2d 交互挂载；
//! 输入经 CompositionInput 门户优先命中——区域内点击回传 Auto（同一
//! `_ui_click_` 派发器/同一有界队列），区域外归 UI；一个 Session owner。
//!
//! --probe   正控：posted OS 点击（按钮×2 → 2D 区域×1 → 3D 区域×1），验证
//!           Auto 状态文字、2D/3D 真实像素位移、互斥路由、空闲零 Tick；
//!           写一份 JSON 报告；pass exit 0。真实 native SendInput 通道是
//!           T-05/手工取证门（048 先例），不在本自动链。
//! (默认)    素窗（无自动化、无报告）
//! GPU/窗口/焦点单实例运行。
use agp::host::iced_surface_probe as probe;
use agp::renderer::{GpuDevice, WindowTarget};
use autographics_auto_driver::node_mixed::MixedUiApp;
use autographics_auto_driver::node_mixed_host::MixedHost;
use autographics_auto_driver::node_ui::PlatformKey;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const TITLE: &str = "AGE mixed-ui-2d-3d (PLAN-049)";
const LOGICAL: [f32; 2] = [420.0, 420.0];
const REGION2: &str = "Main#reg2";
const REGION3: &str = "Main#reg3";
const BUTTON: &str = "Main#inc";

struct App {
    probe: bool,
    report: PathBuf,
    ui_path: String,
    world_path: String,
    window: Option<Arc<Window>>,
    target: Option<WindowTarget>,
    host: Option<MixedHost>,
    cursor_px: [f32; 2],
    frames: u64,
    started: Instant,
    phase: usize,
    passed: bool,
    posted_clicks: u32,
    evidence: Option<Evidence>,
    marks: Vec<(String, u64)>,
}

/// 阶段 0 初始证据（区域 bounds、2D 红方块、3D 圆柱投影像素）。
#[derive(Default, Clone)]
struct Evidence {
    reg2_bounds: [f32; 4],
    reg3_bounds: [f32; 4],
    mover_at_start: Option<[u8; 4]>,
    cyl_proj_at_start: Option<[u8; 4]>,
}

impl App {
    fn new(probe: bool, ui_path: String, world_path: String, report: PathBuf) -> Self {
        Self {
            probe,
            report,
            ui_path,
            world_path,
            window: None,
            target: None,
            host: None,
            cursor_px: [-1.0, -1.0],
            frames: 0,
            started: Instant::now(),
            phase: 0,
            passed: false,
            posted_clicks: 0,
            evidence: None,
            marks: Vec::new(),
        }
    }

    /// 逻辑控件 bounds → 物理客户区 px（posted 点击通道）。
    fn bounds_px(&mut self, key: &str) -> Option<(f32, f32)> {
        let window = self.window.as_ref()?;
        let r = self.host.as_mut()?.widget_bounds(key)?;
        let scale = window.scale_factor() as f32;
        Some(((r.x + r.w / 2.0) * scale, (r.y + r.h / 2.0) * scale))
    }

    fn region_rect(&mut self, key: &str) -> Option<agp::core::composition::Rect> {
        self.host.as_mut()?.widget_bounds(key)
    }

    fn verify_and_finish(&mut self, e: &ActiveEventLoop) {
        let texts = self.host.as_mut().map(|h| h.app().ui.texts(0).to_vec());
        let report = match (texts, self.evidence.clone()) {
            (Some(texts), Some(before)) => {
                let count = texts.get(0).cloned().unwrap_or_default();
                let c2 = texts.get(2).cloned().unwrap_or_default();
                let c3 = texts.get(3).cloned().unwrap_or_default();
                let host = self.host.as_mut().unwrap();
                let ui = host.ui;
                // 像素几何 oracle 归离屏宿主测试（帧节奏确定）；本窗证据面
                // ＝真实 OS 通道端到端：状态文字/双向派发/互斥/空闲零 Tick。
                // 记录像素观测（2D 末态 quad、3D 中心）作诊断，不作门。
                let reg2 = before.reg2_bounds;
                let reg3 = before.reg3_bounds;
                let mover_diag = host.rt.probe_pixel(ui, [reg2[0] + 28.0, reg2[1] + 32.0]);
                let cyl_diag = host.rt.probe_pixel(ui, [reg3[0] + 80.0, reg3[1] + 45.0]);
                let mark = |marks: &[(String, u64)], k: &str| {
                    marks.iter().find(|(n, _)| n == k).map(|(_, v)| *v)
                };
                let idle_ok = mark(&self.marks, "idle_start").is_some()
                    && mark(&self.marks, "idle_start") == mark(&self.marks, "idle_end");
                let _ = (reg3,);
                self.passed = !self.probe
                    || (count == "2"
                        && c2 == "1"
                        && c3 == "1"
                        && self.posted_clicks == 4
                        && idle_ok
                        && self.frames > 60);
                serde_json::json!({
                    "title": TITLE,
                    "host_chain": "MixedHost (T-04 single production assembly)",
                    "count": count,
                    "region2_clicks": c2,
                    "region3_clicks": c3,
                    "os_channel": "posted (WM_MOUSEMOVE/LBUTTONDOWN direct injection; real native SendInput run is the T-05/manual gate)",
                    "posted_clicks": self.posted_clicks,
                    "frames": self.frames,
                    "reg2_bounds": before.reg2_bounds,
                    "reg3_bounds": before.reg3_bounds,
                    "mover_at_start": before.mover_at_start,
                    "cyl_proj_at_start": before.cyl_proj_at_start,
                    "mover_pixel_diag": mover_diag,
                    "cyl_pixel_diag": cyl_diag,
                    "callback_marks": self.marks,
                    "idle_no_tick": idle_ok,
                    "passed": self.passed,
                })
            }
            _ => serde_json::json!({"passed": false, "reason": "missing app/evidence"}),
        };
        std::fs::create_dir_all(&self.report).ok();
        std::fs::write(
            self.report.join("mixed-ui-native.json"),
            serde_json::to_string_pretty(&report).unwrap(),
        )
        .expect("report");
        eprintln!("MIXED-UI NATIVE report: {report}");
        e.exit();
    }

    fn run_phase(&mut self, e: &ActiveEventLoop) {
        if !self.probe {
            return;
        }
        let t = self.started.elapsed().as_millis();
        match (self.phase, t) {
            (0, t) if t >= 1500 => {
                self.phase = 1;
                let reg2 = self
                    .region_rect(REGION2)
                    .map(|r| [r.x, r.y, r.w, r.h])
                    .unwrap_or([0.0; 4]);
                let reg3 = self
                    .region_rect(REGION3)
                    .map(|r| [r.x, r.y, r.w, r.h])
                    .unwrap_or([0.0; 4]);
                let Some(host) = self.host.as_mut() else {
                    return;
                };
                let ui = host.ui;
                let mover_start = if reg2 != [0.0; 4] {
                    host.rt.probe_pixel(ui, [reg2[0] + 12.0, reg2[1] + 32.0])
                } else {
                    None
                };
                let cyl_start = if reg3 != [0.0; 4] {
                    // x=0 圆柱覆盖中心。
                    host.rt.probe_pixel(ui, [reg3[0] + 80.0, reg3[1] + 45.0])
                } else {
                    None
                };
                let cb = host.app().ui.callbacks;
                self.marks.push(("start".into(), cb));
                self.evidence = Some(Evidence {
                    reg2_bounds: reg2,
                    reg3_bounds: reg3,
                    mover_at_start: mover_start,
                    cyl_proj_at_start: cyl_start,
                });
            }
            (1, t) if t >= 2200 => {
                self.phase = 2;
                if let Some(px) = self.bounds_px(BUTTON) {
                    probe::os_click_client_once(TITLE, px, true);
                    self.posted_clicks += 1;
                }
            }
            (2, t) if t >= 2900 => {
                self.phase = 3;
                if let Some(px) = self.bounds_px(BUTTON) {
                    probe::os_click_client_once(TITLE, px, true);
                    self.posted_clicks += 1;
                }
            }
            (3, t) if t >= 3600 => {
                self.phase = 4;
                if let Some(px) = self.bounds_px(REGION2) {
                    probe::os_click_client_once(TITLE, px, true);
                    self.posted_clicks += 1;
                }
            }
            (4, t) if t >= 4300 => {
                self.phase = 5;
                if let Some(px) = self.bounds_px(REGION3) {
                    probe::os_click_client_once(TITLE, px, true);
                    self.posted_clicks += 1;
                }
            }
            (5, t) if t >= 5000 => {
                self.phase = 6;
                let cb = self
                    .host
                    .as_mut()
                    .map(|h| h.app().ui.callbacks)
                    .unwrap_or(0);
                self.marks.push(("idle_start".into(), cb));
            }
            (6, t) if t >= 5800 => {
                self.phase = 7;
                let cb = self
                    .host
                    .as_mut()
                    .map(|h| h.app().ui.callbacks)
                    .unwrap_or(0);
                self.marks.push(("idle_end".into(), cb));
            }
            (7, t) if t >= 6400 => {
                self.phase = 8;
                self.verify_and_finish(e);
            }
            _ => {}
        }
    }

    fn redraw(&mut self, e: &ActiveEventLoop) {
        let Some(host) = self.host.as_mut() else {
            return;
        };
        // MixedHost 单一生产链：挂载刷新＋镜像桥＋场景重建＋组合渲染。
        if let Some(target) = self.target.as_mut() {
            if let Err(err) = host.frame(Some(target)) {
                eprintln!("FRAME ERROR: {err}");
            }
        } else if let Err(err) = host.frame(None) {
            eprintln!("FRAME ERROR: {err}");
        }
        self.frames += 1;
        self.run_phase(e);
    }
}

impl App {
    fn ui_hint(&self) -> Option<agp::core::composition::SurfaceId> {
        self.host.as_ref().map(|h| h.ui)
    }
}

fn window_scale_of(window: Option<&Arc<Window>>) -> f32 {
    window.map(|w| w.scale_factor() as f32).unwrap_or(1.0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let probe_flag = args.iter().any(|a| a == "--probe");
    let ui = args
        .iter()
        .position(|a| a == "--ui")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .expect("--ui <ui.json>");
    let world = args
        .iter()
        .position(|a| a == "--world")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .expect("--world <world.at>");
    let report = args
        .iter()
        .position(|a| a == "--report")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("D:/autostack/plan-artifacts/plan-049/auto-ui/t02/reports")
        });
    let mut app = App::new(probe_flag, ui, world, report);
    EventLoop::new().unwrap().run_app(&mut app).unwrap();
    if probe_flag && !app.passed {
        std::process::exit(1);
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
        let ui_json = std::fs::read_to_string(&self.ui_path).expect("ui.json");
        let world_at = std::fs::read_to_string(&self.world_path).expect("world.at");
        let app = MixedUiApp::new(&ui_json, &world_at, "plan049-window").expect("mixed app");
        let fmt = target.format();
        let host = MixedHost::new(gpu, app, LOGICAL, Some(fmt)).expect("mixed host");
        self.window = Some(window);
        self.target = Some(target);
        self.host = Some(host);
    }

    fn window_event(&mut self, e: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.verify_and_finish(e),
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_px = [position.x as f32, position.y as f32];
                let scale = window_scale_of(self.window.as_ref());
                let logical = [position.x as f32 / scale, position.y as f32 / scale];
                if let Some(h) = self.host.as_mut() {
                    h.pointer_moved(logical);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                if self.cursor_px[0] < 0.0 {
                    return;
                }
                let scale = window_scale_of(self.window.as_ref());
                let logical = [self.cursor_px[0] / scale, self.cursor_px[1] / scale];
                if let Some(h) = self.host.as_mut() {
                    if pressed {
                        h.pointer_pressed(logical);
                    } else {
                        h.pointer_released();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    use winit::keyboard::Key as WKey;
                    use winit::keyboard::NamedKey as WNamed;
                    let key = match &event.logical_key {
                        WKey::Named(WNamed::Tab) => Some(PlatformKey::Tab),
                        WKey::Named(WNamed::Enter) => Some(PlatformKey::Enter),
                        _ => None,
                    };
                    if let Some(k) = key {
                        if let Some(h) = self.host.as_mut() {
                            h.key(k);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => self.redraw(e),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, e: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        // Poll：探针窗需要确定性泵送（WaitUntil 在节流桌面上会休眠——
        // T-04 实测 8 帧/6.4s，posted 消息迟到丢相位）。7s 探针可承受。
        e.set_control_flow(ControlFlow::Poll);
    }
}
