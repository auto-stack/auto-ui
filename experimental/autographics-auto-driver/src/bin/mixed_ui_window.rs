//! PLAN-049 mixed-ui-2d-3d 公开原生窗口入口（T-02 全链先证）。一个真实
//! OS 窗口 = 一个 AGP SurfaceRuntime 组合：UI（Iced 控件，048 adapter
//! 接缝）为窗口面 view；`view-2d`/`view-3d` 嵌入区域为真实布局占位，其
//! 实际 widget bounds 驱动 Game2d/Scene3d child surface 的 Flat2d 交互
//! 挂载（首帧布局后挂载，bounds 变化走 SetMount）。输入经 CompositionInput
//! 门户优先命中——区域内点击回传 Auto（同一 `_ui_click_` 派发器/同一有界
//! 队列），区域外归 UI；一个 Session owner，`app_start` 每 Run 一次、
//! `app_update` 仅显式调用（本探针零逻辑步：全部证据来自真实事件派发）。
//!
//! --probe   正控：真实 OS 点击（按钮×2 → 2D 区域×1 → 3D 区域×1），验证
//!           Auto 状态文字、2D/3D 真实像素位移、互斥路由、空闲零 Tick；
//!           写一份 JSON 报告；pass exit 0
//! (默认)    素窗（无自动化、无报告）
//! GPU/窗口/焦点单实例运行。
use agp::assets::glb::{MaterialDef, MeshData, ModelData};
use agp::core::composition::input::InputOutcome;
use agp::core::composition::{MountKind, Rect, SurfaceId};
use agp::core::scene::Color;
use agp::host::composition_input::CompositionInput;
use agp::host::iced_surface_probe as probe;
use agp::host::surface_runtime::{
    Game2dContent, IcedContent, Scene3dContent, SurfaceContent, SurfaceRuntime,
};
use agp::render3d::MeshInstance;
use agp::render3d::{Camera3d, Projection};
use agp::renderer::pipeline::{DrawCmd, Shape};
use agp::renderer::{capture, GpuDevice, WindowTarget};
use autographics_auto_driver::node_mixed::{MixedMirror, MixedUiApp, SceneElemState};
use autographics_auto_driver::node_ui::PlatformKey;
use glam::Mat4;
use glam::Vec3;
use std::collections::HashMap;
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

/// 每 color 一个过程化圆柱模型（身份 = 网格几何字节＋color 的 sha）。
fn cylinder_model(color: [u8; 3]) -> ModelData {
    let mesh = agp::render3d::cylinder_mesh();
    let mut bytes = Vec::new();
    for p in &mesh.positions {
        for c in p {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
    }
    bytes.extend_from_slice(&color);
    ModelData {
        source_sha256: capture::sha256_hex(&bytes),
        meshes: vec![MeshData {
            visual: None,
            name: "primitive-0".into(),
            positions: mesh.positions,
            normals: mesh.normals,
            indices: mesh.indices,
            material: MaterialDef {
                name: "mixed-cylinder".into(),
                base_color: [
                    color[0] as f32 / 255.0,
                    color[1] as f32 / 255.0,
                    color[2] as f32 / 255.0,
                    1.0,
                ],
                metallic: 0.0,
                roughness: 0.5,
            },
        }],
    }
}

/// 3D 观察相机：eye(0,0,8)→origin，fov45，aspect=区域（160×90）。
/// 圆柱（radius 1，x 每次平移 +2）两次 +4 后离轴：中心像素绿→天空。
fn region_view_proj() -> Mat4 {
    Camera3d {
        eye: Vec3::new(0.0, 0.0, 8.0),
        target: Vec3::ZERO,
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_deg: 45.0,
            aspect: 160.0 / 90.0,
            near: 0.1,
            far: 100.0,
        },
    }
    .view_proj()
}

struct App {
    probe: bool,
    report: PathBuf,
    ui_path: String,
    world_path: String,
    window: Option<Arc<Window>>,
    target: Option<WindowTarget>,
    rt: Option<SurfaceRuntime>,
    ci: Option<CompositionInput>,
    app: Option<MixedUiApp>,
    ui_surface: Option<SurfaceId>,
    s2d: Option<SurfaceId>,
    s3d: Option<SurfaceId>,
    portal2: Option<agp::core::composition::PortalId>,
    portal3: Option<agp::core::composition::PortalId>,
    ui_events: Vec<iced_wgpu::core::Event>,
    cursor_px: [f32; 2],
    cursors: HashMap<SurfaceId, [f32; 2]>,
    last_mount: HashMap<String, [f32; 4]>,
    frames: u64,
    started: Instant,
    phase: usize,
    passed: bool,
    native_clicks: u32,
    evidence: Option<Evidence>,
    marks: Vec<(String, u64)>,
}

/// 阶段 0 初始证据（区域 bounds、2D 红方块、3D 中心绿圆柱像素）。
#[derive(Default, Clone)]
struct Evidence {
    reg2_bounds: [f32; 4],
    reg3_bounds: [f32; 4],
    mover_at_start: Option<[u8; 4]>,
    cyl_center_at_start: Option<[u8; 4]>,
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
            rt: None,
            ci: None,
            app: None,
            ui_surface: None,
            s2d: None,
            s3d: None,
            portal2: None,
            portal3: None,
            ui_events: Vec::new(),
            cursor_px: [-1.0, -1.0],
            cursors: Default::default(),
            last_mount: Default::default(),
            frames: 0,
            started: Instant::now(),
            phase: 0,
            passed: false,
            native_clicks: 0,
            evidence: None,
            marks: Vec::new(),
        }
    }

    /// 混合应用（唯一 Session owner；宿主线程受限持有——Send 镜像桥见
    /// node_mixed::MixedMirror，Session 不进入组合内容）。
    fn with_app<R>(&mut self, f: impl FnOnce(&mut MixedUiApp) -> R) -> Option<R> {
        self.app.as_mut().map(f)
    }

    /// 逻辑控件 bounds → 物理客户区 px（真实 OS 点击通道）。
    fn bounds_px(&mut self, key: &str) -> Option<(f32, f32)> {
        let window = self.window.as_ref()?;
        let rt = self.rt.as_mut()?;
        let ui = self.ui_surface?;
        let b = rt.iced_widget_bounds(ui, iced_wgpu::core::widget::Id::from(key.to_string()))?;
        let scale = window.scale_factor() as f32;
        Some((
            (b.x + b.width / 2.0) * scale,
            (b.y + b.height / 2.0) * scale,
        ))
    }

    /// 区域 bounds（UI 面本地逻辑 px）。
    fn region_rect(&mut self, key: &str) -> Option<Rect> {
        let rt = self.rt.as_mut()?;
        let ui = self.ui_surface?;
        let b = rt.iced_widget_bounds(ui, iced_wgpu::core::widget::Id::from(key.to_string()))?;
        Some(Rect::at(b.x, b.y, b.width, b.height))
    }

    /// 区域实际 bounds → Flat2d 挂载（变化才 SetMount；真实 layout 驱动，
    /// 不猜屏幕常量）。
    fn refresh_mounts(&mut self) {
        let reg2 = self.region_rect(REGION2);
        let reg3 = self.region_rect(REGION3);
        let ui_view = self.rt.as_ref().and_then(|rt| {
            rt.graph
                .views()
                .iter()
                .find(|v| v.desc.presents_to_window)
                .map(|v| v.id)
        });
        let Some(ui_view) = ui_view else { return };
        let (Some(s2d), Some(s3d)) = (self.s2d, self.s3d) else {
            return;
        };
        let mut ops: Vec<(String, Rect, [f32; 2], bool)> = Vec::new();
        if let Some(r) = reg2 {
            ops.push((REGION2.to_string(), r, [160.0, 90.0], false));
        }
        if let Some(r) = reg3 {
            ops.push((REGION3.to_string(), r, [160.0, 90.0], true));
        }
        let Some(rt) = self.rt.as_mut() else { return };
        for (key, rect, content, is3d) in ops {
            let mark = [rect.x, rect.y, rect.w, rect.h];
            if self.last_mount.get(&key) == Some(&mark) {
                continue;
            }
            let mount = MountKind::Flat2d {
                offset: [rect.x, rect.y],
                scale: [rect.w / content[0], rect.h / content[1]],
                clip: rect,
            };
            let child = if is3d { s3d } else { s2d };
            let existing = if is3d {
                &mut self.portal3
            } else {
                &mut self.portal2
            };
            match existing {
                Some(pid) => {
                    let _ = rt.set_portal_mount(*pid, mount);
                }
                None => {
                    if let Ok(pid) = rt.add_portal(ui_view, child, mount, true) {
                        *existing = Some(pid);
                    }
                }
            }
            self.last_mount.insert(key, mark);
        }
    }

    /// 场景命令/实例重建（状态呈现者；仅 drain 后调用；无 VM 访问）。
    fn present_state(&mut self) {
        let states = self.with_app(|app| app.scene_state());
        let Some(states) = states else { return };
        let (Some(rt), Some(s2d), Some(s3d)) = (self.rt.as_mut(), self.s2d, self.s3d) else {
            return;
        };
        for st in &states {
            match (st.name.as_str(), st.dim.as_str()) {
                ("reg2", "2d") => {
                    let mut cmds = Vec::new();
                    for el in &st.elems {
                        match el {
                            SceneElemState::Quad {
                                x, y, w, h, color, ..
                            } => cmds.push(DrawCmd {
                                center: [x + w / 2.0, y + h / 2.0],
                                size: [*w, *h],
                                z: 0.0,
                                color: Color::rgb(
                                    color[0] as f32 / 255.0,
                                    color[1] as f32 / 255.0,
                                    color[2] as f32 / 255.0,
                                ),
                                shape: Shape::Rect,
                            }),
                            SceneElemState::Circle { x, y, r, color, .. } => cmds.push(DrawCmd {
                                center: [*x, *y],
                                size: [r * 2.0, r * 2.0],
                                z: 0.0,
                                color: Color::rgb(
                                    color[0] as f32 / 255.0,
                                    color[1] as f32 / 255.0,
                                    color[2] as f32 / 255.0,
                                ),
                                shape: Shape::Circle,
                            }),
                            _ => {}
                        }
                    }
                    rt.with_game2d_mut(s2d, |g| g.commands = cmds);
                }
                ("reg3", "3d") => {
                    let cyls: Vec<([f32; 3], [u8; 3])> = st
                        .elems
                        .iter()
                        .filter_map(|el| match el {
                            SceneElemState::Cylinder {
                                tx, ty, tz, color, ..
                            } => Some(([*tx, *ty, *tz], *color)),
                            _ => None,
                        })
                        .collect();
                    rt.with_scene3d_mut(s3d, |s, gpu| {
                        let mut instances = Vec::new();
                        for (t, color) in &cyls {
                            let model = cylinder_model(*color);
                            if s.store.get_or_upload(gpu, &model, 0).is_ok() {
                                instances.push(MeshInstance {
                                    mesh: (model.source_sha256.clone(), "primitive-0".into()),
                                    model: Mat4::from_translation(Vec3::from(*t)),
                                });
                            }
                        }
                        s.instances = instances;
                        s.view_proj = region_view_proj();
                    });
                }
                _ => {}
            }
        }
    }

    fn verify_and_finish(&mut self, e: &ActiveEventLoop) {
        let texts = self.with_app(|app| app.ui.texts(0).to_vec());
        let report = match (texts, self.evidence.clone()) {
            (Some(texts), Some(before)) => {
                let count = texts.get(0).cloned().unwrap_or_default();
                let c2 = texts.get(2).cloned().unwrap_or_default();
                let c3 = texts.get(3).cloned().unwrap_or_default();
                let rt = self.rt.as_mut().unwrap();
                let ui = self.ui_surface.unwrap();
                // 2D：mover 两次 +8 → x=16；新位置红、旧位置暗（真实像素
                // 位移对照，不是计数回读）。
                let reg2 = before.reg2_bounds;
                let new_red = rt.probe_pixel(ui, [reg2[0] + 28.0, reg2[1] + 32.0]);
                let old_dark = rt.probe_pixel(ui, [reg2[0] + 12.0, reg2[1] + 32.0]);
                // 3D：圆柱两次 +2 → x=4 离轴；中心由绿变天空（投影真实变化）。
                let reg3 = before.reg3_bounds;
                let cyl_now = rt.probe_pixel(ui, [reg3[0] + 80.0, reg3[1] + 45.0]);
                // 产物像素为 sRGB 编码：quad(200,40,40) 读回 ~(229,110,110)、
                // 场景清底 (0.09,0.1,0.13) 读回 ~(85,89,101)、圆柱绿
                // (90,200,90) 读回 ~(158,230,158)、3D 天空读回 ~(63,75,89)。
                let red = |p: &Option<[u8; 4]>| {
                    p.is_some_and(|q| q[0] > 150 && q[0] > q[1] + 50 && q[0] > q[2] + 50)
                };
                let green = |p: &Option<[u8; 4]>| p.is_some_and(|q| q[1] > 140 && q[1] > q[0] + 30);
                let mark = |marks: &[(String, u64)], k: &str| {
                    marks.iter().find(|(n, _)| n == k).map(|(_, v)| *v)
                };
                let idle_ok = mark(&self.marks, "idle_start").is_some()
                    && mark(&self.marks, "idle_start") == mark(&self.marks, "idle_end");
                self.passed = !self.probe
                    || (count == "2"
                        && c2 == "1"
                        && c3 == "1"
                        && self.native_clicks == 4
                        && red(&new_red)
                        && !red(&old_dark)
                        && green(&before.cyl_center_at_start)
                        && !green(&cyl_now)
                        && idle_ok
                        && self.frames > 60);
                serde_json::json!({
                    "title": TITLE,
                    "count": count,
                    "region2_clicks": c2,
                    "region3_clicks": c3,
                    "posted_clicks": self.native_clicks,
                    "frames": self.frames,
                    "reg2_bounds": before.reg2_bounds,
                    "reg3_bounds": before.reg3_bounds,
                    "mover_at_start": before.mover_at_start,
                    "cyl_center_at_start": before.cyl_center_at_start,
                    "mover_new_pixel": new_red,
                    "mover_old_pixel": old_dark,
                    "cyl_center_now": cyl_now,
                    "callback_marks": self.marks,
                    "idle_no_tick": idle_ok,
                    "passed": self.passed,
                })
            }
            _ => serde_json::json!({"passed": false, "reason": "missing app/evidence"}),
        };
        std::fs::create_dir_all(&self.report);
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
                let (Some(rt), Some(ui)) = (self.rt.as_mut(), self.ui_surface) else {
                    return;
                };
                let mover_start = if reg2 != [0.0; 4] {
                    rt.probe_pixel(ui, [reg2[0] + 12.0, reg2[1] + 32.0])
                } else {
                    None
                };
                let cyl_start = if reg3 != [0.0; 4] {
                    rt.probe_pixel(ui, [reg3[0] + 80.0, reg3[1] + 45.0])
                } else {
                    None
                };
                let cb = self.with_app(|a| a.ui.callbacks).unwrap_or(0);
                self.marks.push(("start".into(), cb));
                self.evidence = Some(Evidence {
                    reg2_bounds: reg2,
                    reg3_bounds: reg3,
                    mover_at_start: mover_start,
                    cyl_center_at_start: cyl_start,
                });
            }
            (1, t) if t >= 2200 => {
                self.phase = 2;
                if let Some(px) = self.bounds_px(BUTTON) {
                    probe::os_click_client_once(TITLE, px, true);
                    self.native_clicks += 1;
                }
            }
            (2, t) if t >= 2900 => {
                self.phase = 3;
                if let Some(px) = self.bounds_px(BUTTON) {
                    probe::os_click_client_once(TITLE, px, true);
                    self.native_clicks += 1;
                }
            }
            (3, t) if t >= 3600 => {
                self.phase = 4;
                if let Some(px) = self.bounds_px(REGION2) {
                    probe::os_click_client_once(TITLE, px, true);
                    self.native_clicks += 1;
                }
            }
            (4, t) if t >= 4300 => {
                self.phase = 5;
                if let Some(px) = self.bounds_px(REGION3) {
                    probe::os_click_client_once(TITLE, px, true);
                    self.native_clicks += 1;
                }
            }
            (5, t) if t >= 5000 => {
                self.phase = 6;
                let cb = self.with_app(|a| a.ui.callbacks).unwrap_or(0);
                self.marks.push(("idle_start".into(), cb));
            }
            (6, t) if t >= 5800 => {
                self.phase = 7;
                let cb = self.with_app(|a| a.ui.callbacks).unwrap_or(0);
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
        self.refresh_mounts();
        let (Some(rt), Some(target)) = (self.rt.as_mut(), self.target.as_mut()) else {
            return;
        };
        let mut events: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>> = Default::default();
        if let Some(ui) = self.ui_surface {
            events.insert(ui, std::mem::take(&mut self.ui_events));
        }
        // 游标是持久状态（048 先例；按帧消费会让释放帧丢游标）。
        let frame_err = rt.frame_with_cursors(Some(target), &events, &self.cursors);
        self.frames += 1;
        if let Err(e) = frame_err {
            eprintln!("FRAME ERROR: {e:?}");
        }
        // Send 镜像桥：控件消息出站 → 宿主侧 Session 队列 → 派发 →
        // 已接受状态副本回推（唯一过桥数据，容量冻结有界）。
        let ui = self.ui_surface;
        let outbound = ui.and_then(|ui| {
            rt.with_ui_mut(ui, |u| {
                u.as_any_mut()
                    .downcast_mut::<MixedMirror>()
                    .map(|m| std::mem::take(&mut m.outbound))
            })
            .flatten()
        });
        let mut dispatched = 0usize;
        if let Some(app) = self.app.as_mut() {
            for msg in outbound.unwrap_or_default() {
                app.ui.handle_string_message(&msg);
            }
            dispatched = app.drain().dispatched;
            if dispatched > 0 {
                let presentation = app.presentation();
                if let Some(ui) = ui {
                    rt.with_ui_mut(ui, |u| {
                        if let Some(m) = u.as_any_mut().downcast_mut::<MixedMirror>() {
                            m.presentation = presentation;
                        }
                    });
                }
            }
        }
        if dispatched > 0 {
            drop(rt);
            self.present_state();
        }
        self.run_phase(e);
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
        // 内容先建（全部引用 &gpu——同设备合同），runtime 后建。
        let ui_json = std::fs::read_to_string(&self.ui_path).expect("ui.json");
        let world_at = std::fs::read_to_string(&self.world_path).expect("world.at");
        let mut app = MixedUiApp::new(&ui_json, &world_at, "plan049-window").expect("mixed app");
        app.start_run().expect("start run");
        let mirror = MixedMirror::new(app.presentation());
        let ui_content = IcedContent::borrowed(&gpu, LOGICAL, Box::new(mirror));
        let game2d = Game2dContent::simple(&gpu, Color::rgb(0.09, 0.1, 0.13));
        let scene3d = Scene3dContent {
            screen_overlay: None,
            text_overlay: None,
            hud_overlay: None,
            mesh: agp::render3d::MeshPipeline::new(
                &gpu,
                agp::renderer::target::OffscreenTarget::FORMAT,
            )
            .unwrap(),
            store: Default::default(),
            instances: vec![],
            sky: [0.05, 0.07, 0.1, 1.0],
            view_proj: region_view_proj(),
        };
        let mut rt = SurfaceRuntime::new(gpu).unwrap();
        rt.set_window_format(target.format());
        let ui_surface = rt
            .add_surface(LOGICAL, SurfaceContent::Iced(Box::new(ui_content)))
            .unwrap();
        let s2d = rt
            .add_surface([160.0, 90.0], SurfaceContent::Game2d(Box::new(game2d)))
            .unwrap();
        let s3d = rt
            .add_surface([160.0, 90.0], SurfaceContent::Scene3d(Box::new(scene3d)))
            .unwrap();
        rt.add_view(ui_surface, true).unwrap();
        rt.add_view(s2d, false).unwrap();
        rt.add_view(s3d, false).unwrap();
        self.window = Some(window);
        self.target = Some(target);
        self.rt = Some(rt);
        self.ci = Some(CompositionInput::new(LOGICAL, LOGICAL));
        self.app = Some(app);
        self.ui_surface = Some(ui_surface);
        self.s2d = Some(s2d);
        self.s3d = Some(s3d);
        self.present_state();
    }

    fn window_event(&mut self, e: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.verify_and_finish(e),
            WindowEvent::CursorMoved { position, .. } => {
                let scale = window_scale_of(self.window.as_ref());
                self.cursor_px = [position.x as f32, position.y as f32];
                let logical = [position.x as f32 / scale, position.y as f32 / scale];
                if let (Some(ci), Some(rt)) = (self.ci.as_mut(), self.rt.as_ref()) {
                    if let Some(InputOutcome::Delivered { surface, local }) =
                        ci.pointer_moved(&rt.graph, logical)
                    {
                        self.cursors.insert(surface, local);
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                eprintln!("DBG mouse_input pressed={}", state == ElementState::Pressed);
                let pressed = state == ElementState::Pressed;
                let pos = self.cursor_px;
                if pos[0] < 0.0 {
                    return;
                }
                let scale = window_scale_of(self.window.as_ref());
                let logical = [pos[0] / scale, pos[1] / scale];
                if pressed {
                    if let (Some(ci), Some(rt)) = (self.ci.as_mut(), self.rt.as_ref()) {
                        let outcome = ci.pointer_pressed(&rt.graph, logical);
                        match outcome {
                            Some(InputOutcome::Delivered { surface, local }) => {
                                if Some(surface) == self.ui_surface {
                                    self.ui_events.push(iced_wgpu::core::Event::Mouse(
                                        iced_wgpu::core::mouse::Event::ButtonPressed(
                                            iced_wgpu::core::mouse::Button::Left,
                                        ),
                                    ));
                                    self.cursors.insert(surface, local);
                                } else if Some(surface) == self.s2d {
                                    self.with_app(|app| {
                                        app.enqueue_region_click("reg2");
                                    });
                                } else if Some(surface) == self.s3d {
                                    self.with_app(|app| {
                                        app.enqueue_region_click("reg3");
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    let released = self.ci.as_mut().and_then(|ci| ci.pointer_released());
                    if released == self.ui_surface {
                        self.ui_events.push(iced_wgpu::core::Event::Mouse(
                            iced_wgpu::core::mouse::Event::ButtonReleased(
                                iced_wgpu::core::mouse::Button::Left,
                            ),
                        ));
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
                        self.with_app(|app| app.ui.key(k));
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
        e.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + std::time::Duration::from_millis(8),
        ));
    }
}
