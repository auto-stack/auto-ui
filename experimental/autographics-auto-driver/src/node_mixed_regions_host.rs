//! PLAN-054 组件/实例活区域组合宿主（T-03 公共装配链，schema v2）。
//!
//! metadata 驱动：区域目录来自产物实例行（`MixedRegionsUiApp::region_
//! catalog`），挂载/卸载由**真实 UI 布局 bounds ＋ 条件可见性**逐帧决定——
//! 不存在固定 App/区域名/尺寸常数。每活动区域一个 child surface
//! （Game2d/Scene3d 由 dim 选择 adapter，声明内容尺寸定 target 与 3D
//! aspect），Flat2d 门户挂载进 UI view；应用活区域总量 ≤4（具名拒绝超界
//! 挂载，应用保持可用）。条件不可见＝撤呈现：remove_portal＋drop_surface
//! （最后 owner 释放 target/content），实例状态（VM 行）不受影响；挂载
//! 代次单调递增，卸载/重挂后旧区域事件在 drain 派发前具名拒绝。
//!
//! 单 owner 不变量沿 049：一个 host 持一个 [`MixedRegionsUiApp`]（一个
//! Session）；`app_start` 每 Run 一次、`app_update` 仅显式逻辑步；UI 文字/
//! 条件/布局与区域场景绑定同一 accept_state 遍历读取；呈现者零 VM 访问。

use crate::node_mixed::{MixedMirror, RegionState, SceneElemState};
use crate::node_mixed_regions::{MixedRegionsUiApp, RegionEntry};
use crate::node_ui::PlatformKey;
use agp::core::composition::input::InputOutcome;
use agp::core::composition::{CompositionError, MountKind, PortalId, Rect, SurfaceId, ViewId};
use agp::core::scene::Color;
use agp::host::composition_input::CompositionInput;
use agp::host::surface_runtime::{
    Game2dContent, IcedContent, Scene3dContent, SurfaceContent, SurfaceRuntime,
};
use agp::render3d::MeshInstance;
use agp::renderer::pipeline::{DrawCmd, Shape};
use agp::renderer::{capture, GpuDevice, WindowTarget};
use glam::{Mat4, Vec3};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// 应用活区域总量上限（T-02 冻结；超界挂载具名拒绝，应用保持可用）。
pub const MAX_ACTIVE_REGIONS: usize = 4;

/// 一个活动区域条目（surface/portal 全套呈现资源）。
struct ActiveRegion {
    surface: SurfaceId,
    view: ViewId,
    portal: PortalId,
    /// 本次挂载的代次（drain 对账面）。
    epoch: u64,
    /// 最近挂载 mark（变化才 SetMount）。
    mark: [f32; 4],
    /// 声明内容尺寸（挂载 scale 分母；3D aspect）。
    width: i64,
    height: i64,
}

/// 指针路由的可观察结果（测试/宿主证据面）。
#[derive(Debug, Clone, PartialEq)]
pub struct RouteReport {
    /// 命中的 surface 与其本地逻辑 px（release 无坐标）。
    pub delivered: Option<(SurfaceId, [f32; 2])>,
    /// 命中区域时的身份键 `{实例路径}#{区域局部名}`（已入队）。
    pub region: Option<String>,
}

/// 区域挂载代次注册表（host 与 NodeUiApp drain 对账共享）。
type Epochs = Rc<RefCell<HashMap<String, u64>>>;

/// 单一混合区域组合宿主（装配＋每帧桥＋路由＋resize＋条件挂卸）。
pub struct MixedRegionsHost {
    pub rt: SurfaceRuntime,
    pub ci: CompositionInput,
    pub ui: SurfaceId,
    app: MixedRegionsUiApp,
    ui_view: ViewId,
    /// 活区域表（键 `{实例路径}#{区域局部名}`）。
    active: HashMap<String, ActiveRegion>,
    /// surface → 区域键（输入路由反查）。
    by_surface: HashMap<SurfaceId, String>,
    epochs: Epochs,
    next_epoch: u64,
    cursors: HashMap<SurfaceId, [f32; 2]>,
    ui_events: Vec<iced_wgpu::core::Event>,
    logical: [f32; 2],
    frames: u64,
    steps: u64,
    /// 记账：已呈现的（dispatched, steps）——变化才重建区域呈现。
    presented: (usize, u64),
    /// 记账：因活跃预算具名拒绝的挂载尝试（键；应用保持可用）。
    pub mount_rejected: Vec<String>,
    /// 记账：激活资源失败（surface/view/portal）的具名诊断。
    pub mount_errors: Vec<String>,
    /// 记账：条件卸载次数（每次释放一套 target/content）。
    pub region_unmounts: u64,
}

impl MixedRegionsHost {
    /// 装配：UI（Iced 镜像，窗口面 view）。区域 surface 在首帧实际布局后
    /// 按区域目录激活（`frame` 内）——无固定名/固定尺寸预建。窗口宿主传
    /// `window_format`；离屏测试传 None。
    pub fn new(
        gpu: GpuDevice,
        mut app: MixedRegionsUiApp,
        logical: [f32; 2],
        window_format: Option<iced_wgpu::wgpu::TextureFormat>,
    ) -> Result<Self, String> {
        app.start_run()?;
        let epochs: Epochs = Rc::new(RefCell::new(HashMap::new()));
        app.ui.set_region_epochs(epochs.clone());
        let mirror = MixedMirror::new(app.presentation());
        let ui_content = IcedContent::borrowed(&gpu, logical, Box::new(mirror));
        let mut rt = SurfaceRuntime::new(gpu).map_err(|e| format!("runtime: {e:?}"))?;
        if let Some(fmt) = window_format {
            rt.set_window_format(fmt);
        }
        let ui_surface = rt
            .add_surface(logical, SurfaceContent::Iced(Box::new(ui_content)))
            .map_err(|e| format!("ui surface: {e:?}"))?;
        let ui_view = rt
            .add_view(ui_surface, true)
            .map_err(|e| format!("ui view: {e:?}"))?;
        Ok(MixedRegionsHost {
            rt,
            ci: CompositionInput::new(logical, logical),
            ui: ui_surface,
            app,
            ui_view,
            active: Default::default(),
            by_surface: Default::default(),
            epochs,
            next_epoch: 1,
            cursors: Default::default(),
            ui_events: Vec::new(),
            logical,
            frames: 0,
            steps: 0,
            presented: (0, 0),
            mount_rejected: Vec::new(),
            mount_errors: Vec::new(),
            region_unmounts: 0,
        })
    }

    /// Session owner 访问（宿主侧；组合内容只持镜像）。
    pub fn app(&mut self) -> &mut MixedRegionsUiApp {
        &mut self.app
    }

    /// 当前活动区域键（装配对账面）。
    pub fn active_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.active.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// 一个区域键的当前挂载代次（未挂载 = None）。
    pub fn region_epoch(&self, key: &str) -> Option<u64> {
        self.active.get(key).map(|a| a.epoch)
    }

    /// 一个活动区域的 child surface（像素/资源 oracle 面）。
    pub fn region_surface(&self, key: &str) -> Option<SurfaceId> {
        self.active.get(key).map(|a| a.surface)
    }

    /// 活动区域数（对账 ≤4 与 target 收支）。
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// 逻辑控件 bounds（UI 面本地逻辑 px；真实布局探测面）。
    pub fn widget_bounds(&mut self, key: &str) -> Option<Rect> {
        let b = self
            .rt
            .iced_widget_bounds(self.ui, iced_wgpu::core::widget::Id::from(key.to_string()))?;
        Some(Rect::at(b.x, b.y, b.width, b.height))
    }

    /// 当前顶层逻辑尺寸。
    pub fn logical(&self) -> [f32; 2] {
        self.logical
    }

    /// 活区域表逐帧对账（真实布局＋条件可见性驱动；无固定名）：
    /// 可见且未活动 → 激活（预算 ≤4，超界具名拒绝）；可见且活动 →
    /// 挂载几何跟随（变化才 SetMount）；不可见且活动 → 撤呈现＋释放
    /// 最后 owner（portal＋surface），代次递增。
    fn refresh_regions(&mut self) {
        let catalog: Vec<RegionEntry> = self.app.region_catalog();
        let mut wanted: Vec<RegionEntry> = Vec::new();
        for entry in catalog {
            let visible = self
                .rt
                .iced_widget_bounds(self.ui, iced_wgpu::core::widget::Id::from(entry.key.clone()))
                .is_some();
            if visible {
                wanted.push(entry);
            }
        }
        // 撤呈现（先于激活，释放的名额本帧即可复用）。
        let live: std::collections::BTreeSet<String> =
            wanted.iter().map(|e| e.key.clone()).collect();
        let stale: Vec<String> = self
            .active
            .keys()
            .filter(|k| !live.contains(*k))
            .cloned()
            .collect();
        for key in stale {
            if let Some(a) = self.active.remove(&key) {
                if let Err(e) = self.rt.remove_portal(a.portal) {
                    self.mount_errors
                        .push(format!("unmount '{key}': portal: {e:?}"));
                }
                if let Err(e) = self.rt.remove_view(a.view) {
                    self.mount_errors
                        .push(format!("unmount '{key}': view: {e:?}"));
                }
                if let Err(e) = self.rt.drop_surface(a.surface) {
                    self.mount_errors
                        .push(format!("unmount '{key}': surface: {e:?}"));
                }
                self.by_surface.remove(&a.surface);
                self.cursors.remove(&a.surface);
                self.epochs.borrow_mut().remove(&key);
                self.region_unmounts += 1;
            }
        }
        // 激活＋挂载几何。
        for entry in wanted {
            if self.active.contains_key(&entry.key) {
                if let Some(rect) = self.widget_bounds(&entry.key) {
                    let mark = [rect.x, rect.y, rect.w, rect.h];
                    let changed = {
                        let a = self.active.get_mut(&entry.key).expect("checked");
                        if a.mark != mark {
                            a.mark = mark;
                            true
                        } else {
                            false
                        }
                    };
                    if changed {
                        let portal = self.active[&entry.key].portal;
                        let mount = MountKind::Flat2d {
                            offset: [rect.x, rect.y],
                            scale: [
                                rect.w / entry.width as f32,
                                rect.h / entry.height as f32,
                            ],
                            clip: rect,
                        };
                        let _ = self.rt.set_portal_mount(portal, mount);
                    }
                }
                continue;
            }
            if self.active.len() >= MAX_ACTIVE_REGIONS {
                // 具名拒绝：区域不挂载；应用与其它区域保持可用（键去重记账）。
                if !self.mount_rejected.iter().any(|k| k == &entry.key) {
                    self.mount_rejected.push(entry.key.clone());
                }
                continue;
            }
            if let Some(rect) = self.widget_bounds(&entry.key) {
                let size = [entry.width as f32, entry.height as f32];
                let created = match entry.dim.as_str() {
                    "2d" => {
                        let content = Game2dContent::simple(
                            self.rt.gpu(),
                            Color::rgb(0.09, 0.1, 0.13),
                        );
                        self.rt
                            .add_surface(size, SurfaceContent::Game2d(Box::new(content)))
                    }
                    "3d" => {
                        let mesh = match agp::render3d::MeshPipeline::new(
                            self.rt.gpu(),
                            agp::renderer::target::OffscreenTarget::FORMAT,
                        ) {
                            Ok(m) => m,
                            Err(e) => {
                                self.mount_errors
                                    .push(format!("region '{}': 3d pipeline: {e:?}", entry.key));
                                continue;
                            }
                        };
                        let content = Scene3dContent {
                            screen_overlay: None,
                            text_overlay: None,
                            hud_overlay: None,
                            mesh,
                            store: Default::default(),
                            instances: vec![],
                            sky: [0.05, 0.07, 0.1, 1.0],
                            view_proj: region_view_proj(entry.width, entry.height),
                        };
                        self.rt
                            .add_surface(size, SurfaceContent::Scene3d(Box::new(content)))
                    }
                    other => Err(CompositionError::InvalidSurface {
                        detail: format!("region dim '{other}' is not 2d|3d"),
                    }),
                };
                let surface = match created {
                    Ok(s) => s,
                    Err(e) => {
                        self.mount_errors
                            .push(format!("region '{}': surface: {e:?}", entry.key));
                        continue;
                    }
                };
                let view = match self.rt.add_view(surface, false) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = self.rt.drop_surface(surface);
                        self.mount_errors
                            .push(format!("region '{}': view: {e:?}", entry.key));
                        continue;
                    }
                };
                let mount = MountKind::Flat2d {
                    offset: [rect.x, rect.y],
                    scale: [
                        rect.w / entry.width as f32,
                        rect.h / entry.height as f32,
                    ],
                    clip: rect,
                };
                let portal =
                    match self.rt.add_portal(self.ui_view, surface, mount, true) {
                        Ok(p) => p,
                        Err(e) => {
                            let _ = self.rt.drop_surface(surface);
                            self.mount_errors
                                .push(format!("region '{}': portal: {e:?}", entry.key));
                            continue;
                        }
                    };
                self.next_epoch += 1;
                let epoch = self.next_epoch;
                let mark = [rect.x, rect.y, rect.w, rect.h];
                self.active.insert(
                    entry.key.clone(),
                    ActiveRegion {
                        surface,
                        view,
                        portal,
                        epoch,
                        mark,
                        width: entry.width,
                        height: entry.height,
                    },
                );
                self.by_surface.insert(surface, entry.key.clone());
                self.epochs.borrow_mut().insert(entry.key.clone(), epoch);
                // 新 surface 首呈现（t0 反映已接受状态）。
                let state = self
                    .app
                    .scene_state()
                    .into_iter()
                    .find(|s| s.path == entry.path && s.name == entry.name);
                if let Some(st) = state {
                    present_region(&mut self.rt, surface, entry.width, entry.height, &st);
                }
            }
        }
    }

    /// 一帧：区域对账＋镜像桥（出站→队列→drain 对账派发→副本回推）＋
    /// 场景重建（状态变化时）＋组合渲染/呈现。
    pub fn frame(
        &mut self,
        window_target: Option<&mut WindowTarget>,
    ) -> Result<RegionFrameReport, String> {
        self.refresh_regions();
        let mut events: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>> = Default::default();
        events.insert(self.ui, std::mem::take(&mut self.ui_events));
        let cursors = self.cursors.clone();
        self.rt
            .frame_with_cursors(window_target, &events, &cursors)
            .map_err(|e| format!("frame: {e:?}"))?;
        self.frames += 1;
        // Send 镜像桥：出站控件消息 → Session 队列 → drain（区域代次对账）
        // → 副本回推。
        let outbound = self
            .rt
            .with_ui_mut(self.ui, |u| {
                u.as_any_mut()
                    .downcast_mut::<MixedMirror>()
                    .map(|m| std::mem::take(&mut m.outbound))
            })
            .flatten()
            .unwrap_or_default();
        for msg in outbound {
            self.app.ui.handle_string_message(&msg);
        }
        let report = self.app.drain();
        if report.dispatched > 0 || self.steps != self.presented.1 {
            let presentation = self.app.presentation();
            self.sync_presentation(presentation);
            self.present_active();
            self.presented = (report.dispatched, self.steps);
        }
        Ok(RegionFrameReport {
            frames: self.frames,
            dispatched: report.dispatched,
            stale_rejected: report.stale_rejected,
            steps: self.steps,
        })
    }

    fn sync_presentation(&mut self, presentation: crate::node_ui::UiPresentation) {
        let _ = self.rt.with_ui_mut(self.ui, |u| {
            if let Some(m) = u.as_any_mut().downcast_mut::<MixedMirror>() {
                m.presentation = presentation;
            }
        });
    }

    /// 全部活动区域的场景重建（呈现者：纯缓存读，无 VM 访问）。
    fn present_active(&mut self) {
        let states: Vec<RegionState> = self
            .app
            .scene_state()
            .into_iter()
            .filter(|s| self.active.contains_key(&format!("{}#{}", s.path, s.name)))
            .collect();
        for st in &states {
            let key = format!("{}#{}", st.path, st.name);
            if let Some(a) = self.active.get(&key) {
                present_region(&mut self.rt, a.surface, a.width, a.height, st);
            }
        }
    }

    /// 显式逻辑步（app_update；仅此处，重绘永不代 Tick）。步进即状态变化
    /// ——下一 frame 重建区域呈现（子场景自主 Tick 的呈现面）。
    pub fn step(&mut self, dt: f32) -> Result<(), String> {
        self.steps += 1;
        self.app.step(dt)?;
        let presentation = self.app.presentation();
        self.sync_presentation(presentation);
        self.present_active();
        self.presented.1 = self.steps;
        Ok(())
    }

    /// resize：UI 逻辑尺寸＋GPU target＋图尺寸事务重建＋路由器尺寸；
    /// 区域挂载几何由后续帧的实际布局 SetMount 刷新。
    pub fn resize(&mut self, logical: [f32; 2]) -> Result<(), String> {
        self.rt
            .resize_surface(self.ui, logical)
            .map_err(|e| format!("resize: {e:?}"))?;
        self.ci.set_sizes(logical, logical);
        self.logical = logical;
        Ok(())
    }

    pub fn pointer_moved(&mut self, window_px: [f32; 2]) -> Option<RouteReport> {
        let outcome = self.ci.pointer_moved(&self.rt.graph, window_px)?;
        Some(self.absorb(outcome, false))
    }

    pub fn pointer_pressed(&mut self, window_px: [f32; 2]) -> Option<RouteReport> {
        let outcome = self.ci.pointer_pressed(&self.rt.graph, window_px)?;
        Some(self.absorb(outcome, true))
    }

    pub fn pointer_released(&mut self) -> Option<RouteReport> {
        let delivered = self.ci.pointer_released();
        if delivered == Some(self.ui) {
            self.ui_events.push(iced_wgpu::core::Event::Mouse(
                iced_wgpu::core::mouse::Event::ButtonReleased(iced_wgpu::core::mouse::Button::Left),
            ));
        }
        Some(RouteReport {
            delivered: delivered.map(|s| (s, [0.0, 0.0])),
            region: None,
        })
    }

    /// 失焦/离窗取消捕获（held/capture 面；区域点击在途不入队）。
    pub fn cancel_all(&mut self, reason: agp::core::composition::input::CancelReason) {
        let cancelled = self.ci.cancel_all(reason);
        self.absorb_cancel(cancelled);
    }

    pub fn window_focus_lost(&mut self) {
        let cancelled = self.ci.window_focus_lost();
        self.absorb_cancel(cancelled);
    }

    fn absorb_cancel(&mut self, cancelled: Option<SurfaceId>) {
        if cancelled == Some(self.ui) {
            self.ui_events.push(iced_wgpu::core::Event::Mouse(
                iced_wgpu::core::mouse::Event::ButtonReleased(iced_wgpu::core::mouse::Button::Left),
            ));
        }
    }

    /// 平台键盘路由（Tab/Enter；UI 控件消费的键盘不进入图形区域）。
    pub fn key(&mut self, key: PlatformKey) -> bool {
        self.app.ui.key(key)
    }

    /// 当前焦点元素键（测试/诊断/呈现面）。
    pub fn focused_key(&self) -> Option<&str> {
        self.app.ui.focused_key()
    }

    fn absorb(&mut self, outcome: InputOutcome, press: bool) -> RouteReport {
        match outcome {
            InputOutcome::Delivered { surface, local } => {
                self.cursors.insert(surface, local);
                let region = if surface == self.ui {
                    if press {
                        self.ui_events.push(iced_wgpu::core::Event::Mouse(
                            iced_wgpu::core::mouse::Event::ButtonPressed(
                                iced_wgpu::core::mouse::Button::Left,
                            ),
                        ));
                    }
                    None
                } else if press {
                    // 反查区域身份（无固定名）；命中即按当前挂载代次入队。
                    let hit = self.by_surface.get(&surface).cloned();
                    match hit {
                        Some(key) => {
                            let epoch = self.active.get(&key).map(|a| a.epoch);
                            let (path, name) = match key.rsplit_once('#') {
                                Some((p, n)) => (p.to_string(), n.to_string()),
                                None => (key.clone(), key.clone()),
                            };
                            if let Some(epoch) = epoch {
                                let _ = self
                                    .app
                                    .enqueue_region_click_at(&path, &name, epoch);
                            }
                            Some(format!("{path}#{name}"))
                        }
                        None => None,
                    }
                } else {
                    None
                };
                RouteReport {
                    delivered: Some((surface, local)),
                    region,
                }
            }
            InputOutcome::Occluded | InputOutcome::Cancelled(_) => RouteReport {
                delivered: None,
                region: None,
            },
        }
    }
}

/// 一个帧的可观察记账（v2 面：含区域陈旧拒绝计数）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionFrameReport {
    pub frames: u64,
    /// 本帧派发的事件数（0＝纯重绘；呈现者不重建）。
    pub dispatched: usize,
    /// 本帧被具名拒绝的陈旧事件数（Run 代次或区域挂载代次）。
    pub stale_rejected: usize,
    /// 显式逻辑步累计（对账面）。
    pub steps: u64,
}

/// 3D 区域观察相机（eye(0,0,8)→origin，fov45；aspect＝区域声明内容比——
/// render3d 显式投影面，实际区域尺寸定投影）。
pub fn region_view_proj(width: i64, height: i64) -> Mat4 {
    agp::render3d::Camera3d {
        eye: Vec3::new(0.0, 0.0, 8.0),
        target: Vec3::ZERO,
        up: Vec3::Y,
        projection: agp::render3d::Projection::Perspective {
            fov_y_deg: 45.0,
            aspect: width as f32 / height as f32,
            near: 0.1,
            far: 100.0,
        },
    }
    .view_proj()
}

/// 每区域一个的圆柱模型（身份＝网格几何字节＋color 的 sha；与 049 同式）。
pub fn region_cylinder_model(color: [u8; 3]) -> agp::assets::glb::ModelData {
    let mesh = agp::render3d::cylinder_mesh();
    let mut bytes = Vec::new();
    for p in &mesh.positions {
        for c in p {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
    }
    bytes.extend_from_slice(&color);
    agp::assets::glb::ModelData {
        source_sha256: capture::sha256_hex(&bytes),
        meshes: vec![agp::assets::glb::MeshData {
            visual: None,
            name: "primitive-0".into(),
            positions: mesh.positions,
            normals: mesh.normals,
            indices: mesh.indices,
            material: agp::assets::glb::MaterialDef {
                name: "mixed-regions-cylinder".into(),
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

/// 场景命令/实例重建（dim 只决定 adapter；声明内容尺寸定投影/缩放）。
fn present_region(
    rt: &mut SurfaceRuntime,
    surface: SurfaceId,
    width: i64,
    height: i64,
    st: &RegionState,
) {
    match st.dim.as_str() {
        "2d" => {
            let mut cmds: Vec<DrawCmd> = Vec::new();
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
            rt.with_game2d_mut(surface, |g| g.commands = cmds);
        }
        "3d" => {
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
            rt.with_scene3d_mut(surface, |s, gpu| {
                let mut instances = Vec::new();
                for (t, color) in &cyls {
                    let model = region_cylinder_model(*color);
                    if s.store.get_or_upload(gpu, &model, 0).is_ok() {
                        instances.push(MeshInstance {
                            mesh: (model.source_sha256.clone(), "primitive-0".into()),
                            model: Mat4::from_translation(Vec3::from(*t)),
                        });
                    }
                }
                s.instances = instances;
                s.view_proj = region_view_proj(width, height);
            });
        }
        _ => {}
    }
}
