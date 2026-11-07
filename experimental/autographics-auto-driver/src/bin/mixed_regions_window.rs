//! PLAN-054 混合区域公开原生窗口入口（v2 装配/挂卸/路由全部经
//! `MixedRegionsHost` 单一生产链——本 bin 只持窗口/探针相位/验证报告）。
//! 一个真实 OS 窗口 = MixedRegionsHost 组合：UI（Iced 控件）为窗口面
//! view；组件/root 实例区域按 metadata 目录＋真实布局激活（应用活区域
//! ≤4）；条件隐藏＝撤呈现；输入经 CompositionInput 门户优先命中。
//!
//! --probe   正控相位（全部真实 OS 通道 posted 注入）：
//!           bump 按钮×1 → p1 区域×1 → strip 区域×1（布局 A）→
//!           Tab+Enter（真实 OS 键盘；toggle→隐藏 p1/p2，撤呈现）→
//!           strip 于移动后布局×1 → Tab+Enter（恢复；新挂载代次）→
//!           p1 重挂后×1 → resize（真实窗 resize；区域保持活跃）→
//!           空闲零 Tick。验证 Auto 状态文字、逐实例区域状态、挂卸代次、
//!           布局跟随路由；写 JSON 报告；pass exit 0。
//!           真实 native SendInput/焦点/DPI 切换通道沿 048/049 手工取证
//!           先例（不在本自动链；scale_factor 与 posted 坐标=逻辑×scale
//!           在报告记录）。
//! (默认)    素窗（无自动化、无报告）
//! --manual  可执行手工门（PLAN-066 T-02）：人真实操作本窗（Tab/Enter、点击、
//!           失焦、移出、resize、第二 DPI scale）；关窗时写
//!           `manual-native-gate.json` 记录真实 OS 观察，auto_pass=false
//!           （手工证据，不计自动 pass）。
//! GPU/窗口/焦点单实例运行。
use agp::host::iced_surface_probe as probe;
use agp::renderer::{GpuDevice, WindowTarget};
use autographics_auto_driver::node_mixed_regions::MixedRegionsUiApp;
use autographics_auto_driver::node_mixed_regions_host::MixedRegionsHost;
use autographics_auto_driver::node_ui::PlatformKey;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::Key as WKey;
use winit::keyboard::NamedKey as WNamed;
use winit::window::{Window, WindowId};

const TITLE: &str = "AGE mixed-regions (PLAN-054)";
const LOGICAL: [f32; 2] = [480.0, 560.0];
const RESIZED: [f32; 2] = [400.0, 420.0];
const P1: &str = "Dash/p1#arc";
const STRIP: &str = "Dash#strip";
const BUMP: &str = "Dash#bump";
const VK_TAB: u32 = 0x09;
const VK_RETURN: u32 = 0x0D;

struct App {
    probe: bool,
    /// PLAN-066 T-02 manual gate: record real OS events (human-run) without
    /// claiming an automatic pass.
    manual: bool,
    report: PathBuf,
    ui_path: String,
    world_path: String,
    window: Option<Arc<Window>>,
    target: Option<WindowTarget>,
    host: Option<MixedRegionsHost>,
    cursor_px: [f32; 2],
    frames: u64,
    started: Instant,
    phase: usize,
    passed: bool,
    posted_clicks: u32,
    posted_keys: u32,
    scale_observed: f32,
    logical_sizes: Vec<[f32; 2]>,
    physical_sizes: Vec<[u32; 2]>,
    epochs_before_toggle: [Option<u64>; 2],
    epochs_after_remount: [Option<u64>; 2],
    marks: Vec<(String, u64)>,
    manual_seen: ManualSeen,
}

/// Real OS-channel observations for the manual gate (never auto-passed).
#[derive(Default)]
struct ManualSeen {
    tabs: u32,
    enters: u32,
    clicks: u32,
    focus_lost: u32,
    cursor_left: u32,
    resizes: u32,
    scales: Vec<f32>,
}

impl App {
    fn new(
        probe: bool,
        manual: bool,
        ui_path: String,
        world_path: String,
        report: PathBuf,
    ) -> Self {
        Self {
            probe,
            manual,
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
            posted_keys: 0,
            scale_observed: 1.0,
            logical_sizes: Vec::new(),
            physical_sizes: Vec::new(),
            epochs_before_toggle: [None; 2],
            epochs_after_remount: [None; 2],
            marks: Vec::new(),
            manual_seen: ManualSeen::default(),
        }
    }

    /// 逻辑控件 bounds → 物理客户区 px（posted 点击通道＝逻辑×scale）。
    fn bounds_px(&mut self, key: &str) -> Option<(f32, f32)> {
        let window = self.window.as_ref()?;
        let r = self.host.as_mut()?.widget_bounds(key)?;
        let scale = window.scale_factor() as f32;
        Some(((r.x + r.w / 2.0) * scale, (r.y + r.h / 2.0) * scale))
    }

    fn click(&mut self, key: &str) {
        if let Some(px) = self.bounds_px(key) {
            probe::os_click_client_once(TITLE, px, true);
            self.posted_clicks += 1;
        }
    }

    fn scene_values(&mut self) -> (f32, f32, f32) {
        let h = self.host.as_mut().expect("host");
        let st = h.app().scene_state();
        let qx = |path: &str| -> f32 {
            match &st.iter().find(|s| s.path == path).expect(path).elems[0] {
                autographics_auto_driver::node_mixed::SceneElemState::Quad { x, .. } => *x,
                autographics_auto_driver::node_mixed::SceneElemState::Cylinder { tx, .. } => *tx,
                _ => 0.0,
            }
        };
        (qx("Dash/p1"), qx("Dash/p2"), qx("Dash/m3"))
    }

    fn verify_and_finish(&mut self, e: &ActiveEventLoop) {
        if self.manual {
            let seen = &self.manual_seen;
            let report = serde_json::json!({
                "title": TITLE,
                "mode": "manual-gate (PLAN-066 T-02)",
                "auto_pass": false,
                "observed": {
                    "tabs": seen.tabs,
                    "enters": seen.enters,
                    "clicks": seen.clicks,
                    "focus_lost": seen.focus_lost,
                    "cursor_left": seen.cursor_left,
                    "resizes": seen.resizes,
                    "scales": seen.scales,
                },
                "checklist": [
                    "Tab + Enter activate the next button exactly once",
                    "a click outside every control changes nothing",
                    "Alt-Tab away then back: the stale focused button does not activate on Enter",
                    "mouse leave cancels the press (no region click)",
                    "resize; the next click lands on the widget's new position",
                    "repeat on a second DPI scale if the machine has one",
                ],
                "note": "real OS input evidence only; never an automatic pass",
            });
            std::fs::create_dir_all(&self.report).ok();
            std::fs::write(
                self.report.join("manual-native-gate.json"),
                serde_json::to_string_pretty(&report).unwrap(),
            )
            .expect("manual report");
            eprintln!("MIXED-REGIONS MANUAL GATE report: {report}");
            e.exit();
            return;
        }
        let state = self.host.as_mut().map(|h| {
            (
                h.app().ui.texts(0).to_vec(),
                h.active_keys(),
                h.app().ui.callbacks,
            )
        });
        let report = match state {
            Some((texts, active, callbacks)) => {
                let count = texts.get(0).cloned().unwrap_or_default();
                let (p1x, p2x, m3tx) = self.scene_values();
                let mark = |k: &str| self.marks.iter().find(|(n, _)| n == k).map(|(_, v)| *v);
                let idle_ok =
                    mark("idle_start").is_some() && mark("idle_start") == mark("idle_end");
                let toggled = self.epochs_before_toggle != self.epochs_after_remount
                    && self.epochs_after_remount.iter().all(|e| e.is_some());
                self.passed = !self.probe
                    || (count == "202"
                        && p1x == 24.0
                        && p2x == 0.0
                        && m3tx >= 0.0
                        && active.len() == 4
                        && active.iter().any(|k| k == P1)
                        && active.iter().any(|k| k == STRIP)
                        && toggled
                        && idle_ok);
                serde_json::json!({
                    "title": TITLE,
                    "host_chain": "MixedRegionsHost (PLAN-054 metadata assembly)",
                    "count": count,
                    "post_resize_hit": count == "202",
                    "p1_x": p1x,
                    "p2_x": p2x,
                    "m3_tx": m3tx,
                    "active_regions": active,
                    "epochs_before_toggle": self.epochs_before_toggle,
                    "epochs_after_remount": self.epochs_after_remount,
                    "os_channel": "posted (WM clicks + WM keys direct injection; real native SendInput/focus/DPI-switch run is the manual gate per 048/049 precedent)",
                    "posted_clicks": self.posted_clicks,
                    "posted_keys": self.posted_keys,
                    "scale_factor_observed": self.scale_observed,
                    "logical_sizes": self.logical_sizes,
                    "physical_sizes": self.physical_sizes,
                    "frames": self.frames,
                    "callback_marks": self.marks,
                    "callbacks_total": callbacks,
                    "idle_no_tick": idle_ok,
                    "passed": self.passed,
                })
            }
            _ => serde_json::json!({"passed": false, "reason": "missing app"}),
        };
        std::fs::create_dir_all(&self.report).ok();
        std::fs::write(
            self.report.join("mixed-regions-native.json"),
            serde_json::to_string_pretty(&report).unwrap(),
        )
        .expect("report");
        eprintln!("MIXED-REGIONS NATIVE report: {report}");
        e.exit();
    }

    fn run_phase(&mut self, e: &ActiveEventLoop) {
        if !self.probe {
            return;
        }
        let t = self.started.elapsed().as_millis();
        // 相位节奏：每相 700ms（settle 首相 1500ms）。
        let at = |phase: usize, ms: u128| self.phase == phase && t >= ms;
        match self.phase {
            0 if t >= 1500 => {
                self.phase = 1;
                if let Some(w) = &self.window {
                    self.scale_observed = w.scale_factor() as f32;
                }
                if let Some(h) = self.host.as_mut() {
                    let cb = h.app().ui.callbacks;
                    self.marks.push(("start".into(), cb));
                    self.epochs_before_toggle = [h.region_epoch(P1), h.region_epoch(STRIP)];
                    self.logical_sizes.push(h.logical());
                }
            }
            1 if at(1, 2200) => {
                self.phase = 2;
                self.click(BUMP);
            }
            2 if at(2, 2900) => {
                self.phase = 3;
                self.click(P1);
            }
            3 if at(3, 3600) => {
                self.phase = 4;
                self.click(STRIP); // count 1 → 101
            }
            4 if at(4, 4300) => {
                self.phase = 5;
                if let Some(w) = &self.window {
                    if probe::os_key_client_once(TITLE, VK_TAB) {
                        self.posted_keys += 1;
                    }
                    let _ = w;
                }
            }
            5 if at(5, 5000) => {
                self.phase = 6;
                if probe::os_key_client_once(TITLE, VK_RETURN) {
                    self.posted_keys += 1;
                }
            }
            6 if at(6, 5700) => {
                self.phase = 7;
                // 隐藏态：p1/p2 撤呈现；strip 布局上移——移动后布局点击。
                let cb = self
                    .host
                    .as_mut()
                    .map(|h| h.app().ui.callbacks)
                    .unwrap_or(0);
                self.marks.push(("hidden".into(), cb));
                self.click(STRIP); // count 101 → 201（真实布局跟随路由）
            }
            // 二次回切：Enter 激活后焦点仍在 toggle（环=bump/toggle）；
            // Tab×2 确定性回到 toggle（1→0→1），Enter 激活回 show=1。
            7 if at(7, 6400) => {
                self.phase = 8;
                if probe::os_key_client_once(TITLE, VK_TAB) {
                    self.posted_keys += 1;
                }
            }
            8 if at(8, 6800) => {
                self.phase = 9;
                if probe::os_key_client_once(TITLE, VK_TAB) {
                    self.posted_keys += 1;
                }
            }
            9 if at(9, 7400) => {
                self.phase = 10;
                if probe::os_key_client_once(TITLE, VK_RETURN) {
                    self.posted_keys += 1;
                }
            }
            10 if at(10, 8100) => {
                self.phase = 11;
                if let Some(h) = self.host.as_mut() {
                    self.epochs_after_remount = [h.region_epoch(P1), h.region_epoch(STRIP)];
                }
                self.click(P1); // 新代次：x 12 → 24
            }
            11 if at(11, 8800) => {
                self.phase = 12;
                // 真实窗 resize：Windows 同步路径 request_inner_size 直接
                // 返回新物理尺寸（无后续 Resized 事件）；异步平台由
                // Resized 事件驱动 host.resize。两条路径同一 host 入口。
                if let (Some(w), Some(h)) = (self.window.as_ref(), self.host.as_mut()) {
                    let req =
                        w.request_inner_size(winit::dpi::LogicalSize::new(RESIZED[0], RESIZED[1]));
                    self.marks
                        .push(("request_returned_sync".into(), req.is_some() as u64));
                    if let Some(phys) = req {
                        let scale = w.scale_factor() as f32;
                        let logical = [
                            phys.width.max(1) as f32 / scale,
                            phys.height.max(1) as f32 / scale,
                        ];
                        {
                            let gpu = h.rt.gpu();
                            if let Some(target) = self.target.as_mut() {
                                let _ = target.handle_resize(
                                    gpu,
                                    phys.width.max(1),
                                    phys.height.max(1),
                                );
                            }
                        }
                        let _ = h.resize(logical);
                    }
                }
            }
            12 if at(12, 9500) => {
                self.phase = 13;
                if let (Some(w), Some(h)) = (self.window.as_ref(), self.host.as_mut()) {
                    let phys = w.inner_size();
                    self.physical_sizes.push([phys.width, phys.height]);
                    self.logical_sizes.push(h.logical());
                }
                // PLAN-066 T-03: the post-resize hit uses the same posted
                // physical channel (logical layout × scale); bump +1 (201→202)
                // proves the hit follows the new layout.
                self.click(BUMP);
            }
            13 if at(13, 10300) => {
                self.phase = 14;
                let cb = self
                    .host
                    .as_mut()
                    .map(|h| h.app().ui.callbacks)
                    .unwrap_or(0);
                self.marks.push(("idle_start".into(), cb));
            }
            14 if at(14, 11100) => {
                self.phase = 15;
                let cb = self
                    .host
                    .as_mut()
                    .map(|h| h.app().ui.callbacks)
                    .unwrap_or(0);
                self.marks.push(("idle_end".into(), cb));
            }
            15 if at(15, 11900) => {
                self.phase = 16;
                self.verify_and_finish(e);
            }
            _ => {}
        }
    }

    fn redraw(&mut self, e: &ActiveEventLoop) {
        let Some(host) = self.host.as_mut() else {
            return;
        };
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

fn window_scale_of(window: Option<&Arc<Window>>) -> f32 {
    window.map(|w| w.scale_factor() as f32).unwrap_or(1.0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let probe_flag = args.iter().any(|a| a == "--probe");
    let manual = args.iter().any(|a| a == "--manual");
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
            PathBuf::from("D:/autostack/plan-artifacts/plan-054/auto-ui/t04/reports")
        });
    let mut app = App::new(probe_flag, manual, ui, world, report);
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
        let _ = &gpu;
        let ui_json = std::fs::read_to_string(&self.ui_path).expect("ui.json");
        let world_at = std::fs::read_to_string(&self.world_path).expect("world.at");
        let app = MixedRegionsUiApp::new(&ui_json, &world_at, "plan054-window").expect("v2 app");
        let fmt = target.format();
        let host = MixedRegionsHost::new(gpu, app, LOGICAL, Some(fmt)).expect("v2 host");
        self.window = Some(window);
        self.target = Some(target);
        self.host = Some(host);
    }

    fn window_event(&mut self, e: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.verify_and_finish(e),
            WindowEvent::Resized(size) => {
                self.marks.push(("resized_event".into(), size.width as u64));
                self.manual_seen.resizes += u32::from(self.manual);
                if self.manual {
                    if let Some(w) = &self.window {
                        self.manual_seen.scales.push(w.scale_factor() as f32);
                    }
                }
                let (Some(target), Some(host)) = (self.target.as_mut(), self.host.as_mut()) else {
                    return;
                };
                let scale = window_scale_of(self.window.as_ref());
                let logical = [
                    size.width.max(1) as f32 / scale,
                    size.height.max(1) as f32 / scale,
                ];
                {
                    // 单设备合同：GPU 由 host 独占持有（rt.gpu() 借用面）。
                    let gpu = host.rt.gpu();
                    let _ = target.handle_resize(gpu, size.width.max(1), size.height.max(1));
                }
                match host.resize(logical) {
                    Ok(_) => self
                        .marks
                        .push(("resized_applied".into(), logical[0] as u64)),
                    Err(_) => self.marks.push(("resize_err".into(), 0)),
                }
            }
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
                if self.cursor_px[0] < 0.0 {
                    return;
                }
                if state == ElementState::Pressed {
                    self.manual_seen.clicks += u32::from(self.manual);
                }
                let scale = window_scale_of(self.window.as_ref());
                let logical = [self.cursor_px[0] / scale, self.cursor_px[1] / scale];
                if let Some(h) = self.host.as_mut() {
                    if state == ElementState::Pressed {
                        h.pointer_pressed(logical);
                    } else {
                        h.pointer_released();
                    }
                }
            }
            WindowEvent::Focused(false) => {
                // PLAN-066 T-03: a real OS focus loss cancels in-flight capture
                // and invalidates the keyboard focus (no stale Enter target).
                self.manual_seen.focus_lost += u32::from(self.manual);
                self.marks.push(("focus_lost".into(), 1));
                if let Some(h) = self.host.as_mut() {
                    h.window_focus_lost();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.manual_seen.cursor_left += u32::from(self.manual);
                if let Some(h) = self.host.as_mut() {
                    h.cancel_all(agp::core::composition::input::CancelReason::PointerLeftWindow);
                    self.marks.push(("cursor_left".into(), 1));
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let key = match &event.logical_key {
                        WKey::Named(WNamed::Tab) => Some(PlatformKey::Tab),
                        WKey::Named(WNamed::Enter) => Some(PlatformKey::Enter),
                        _ => None,
                    };
                    if let Some(k) = key {
                        if self.manual {
                            match k {
                                PlatformKey::Tab => self.manual_seen.tabs += 1,
                                PlatformKey::Enter => self.manual_seen.enters += 1,
                            }
                        }
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
        // Poll：探针窗需要确定性泵送（沿 049 实测——WaitUntil 在节流桌面
        // 上休眠丢相位）。
        e.set_control_flow(ControlFlow::Poll);
    }
}
