//! PLAN-049 混合组合宿主链（T-04 生产化公共链）。
//!
//! 把 T-02 探针的内联装配收敛为单一生产入口：组合装配（UI 窗口面 view ＋
//! 每区域 Game2d/Scene3d child surface ＋ Flat2d 交互挂载）、每帧桥（镜像
//! 出站 → Session 队列 → drain → 已接受状态副本回推 → 场景呈现重建）、
//! 区域路由（CompositionInput 门户优先命中；区域内→区域事件入同一队列，
//! 区域外→UI 控件）、resize（iced 逻辑尺寸＋GPU target＋图尺寸事务重建，
//! 挂载几何随真实布局 SetMount）。窗口 bin 与离屏 flow 测试共用本链——
//! 不存在第二份装配代码。
//!
//! 单 owner 不变量：一个 [`MixedHost`] 持有一个 [`MixedUiApp`]（一个
//! Session）；`app_start` 每 Run 一次、`app_update` 仅显式逻辑步；UI 文字/
//! 条件/布局与区域场景绑定同一 accept_state 遍历读取。

use crate::node_mixed::{MixedMirror, MixedUiApp, RegionState, SceneElemState};
use crate::node_ui::{PlatformKey, UiPresentation};
use agp::assets::glb::{MaterialDef, MeshData, ModelData};
use agp::core::composition::input::InputOutcome;
use agp::core::composition::{MountKind, PortalId, Rect, SurfaceId, ViewId};
use agp::core::scene::Color;
use agp::host::composition_input::CompositionInput;
use agp::host::surface_runtime::{
    Game2dContent, IcedContent, Scene3dContent, SurfaceContent, SurfaceRuntime,
};
use agp::render3d::MeshInstance;
use agp::renderer::pipeline::{DrawCmd, Shape};
use agp::renderer::{capture, GpuDevice, WindowTarget};
use glam::{Mat4, Vec3};
use std::collections::HashMap;

/// 每 color 一个过程化圆柱模型（身份 = 网格几何字节＋color 的 sha；与
/// T-02 探针同一公式）。
pub fn cylinder_model(color: [u8; 3]) -> ModelData {
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

/// 3D 区域观察相机（eye(0,0,8)→origin，fov45，aspect=区域 160×90；
/// render3d 显式投影面——免投影 API 漂移）。
pub fn mixed_view_proj() -> Mat4 {
    agp::render3d::Camera3d {
        eye: Vec3::new(0.0, 0.0, 8.0),
        target: Vec3::ZERO,
        up: Vec3::Y,
        projection: agp::render3d::Projection::Perspective {
            fov_y_deg: 45.0,
            aspect: 160.0 / 90.0,
            near: 0.1,
            far: 100.0,
        },
    }
    .view_proj()
}

const REGION2: &str = "reg2";
const REGION3: &str = "reg3";
const REGION2_KEY: &str = "Main#reg2";
const REGION3_KEY: &str = "Main#reg3";
/// 冻结区域内容逻辑尺寸（与产物 width/height 一致；挂载 scale 分母）。
const REGION_CONTENT: [f32; 2] = [160.0, 90.0];

/// 单一混合组合宿主（装配＋每帧桥＋路由＋resize）。
pub struct MixedHost {
    pub rt: SurfaceRuntime,
    pub ci: CompositionInput,
    pub ui: SurfaceId,
    pub s2d: SurfaceId,
    pub s3d: SurfaceId,
    app: MixedUiApp,
    ui_view: ViewId,
    portals: HashMap<String, PortalId>,
    last_mount: HashMap<String, [f32; 4]>,
    ui_events: Vec<iced_wgpu::core::Event>,
    cursors: HashMap<SurfaceId, [f32; 2]>,
    logical: [f32; 2],
    frames: u64,
    steps: u64,
}

/// 一次指针路由的可观察结果（测试/宿主证据面）。
#[derive(Debug, Clone, PartialEq)]
pub struct RouteReport {
    /// 命中的 surface 与其本地逻辑 px（release 无坐标）。
    pub delivered: Option<(SurfaceId, [f32; 2])>,
    /// 命中区域时已入队的事件名（reg2/reg3）。
    pub region: Option<&'static str>,
}

/// 一帧的可观察记账。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameReport {
    pub frames: u64,
    /// 本帧派发的事件数（0＝纯重绘；呈现者不重建）。
    pub dispatched: usize,
    /// 显式逻辑步累计（对账面）。
    pub steps: u64,
}

impl MixedHost {
    /// 装配：UI（Iced 镜像，窗口面 view）＋每区域 child surface＋初始
    /// 场景呈现；区域挂载在首帧实际布局后补挂（`frame` 内）。窗口宿主
    /// 传 `window_format`；离屏测试传 None。
    pub fn new(
        gpu: GpuDevice,
        mut app: MixedUiApp,
        logical: [f32; 2],
        window_format: Option<iced_wgpu::wgpu::TextureFormat>,
    ) -> Result<Self, String> {
        app.start_run()?;
        let mirror = MixedMirror::new(app.presentation());
        let ui_content = IcedContent::borrowed(&gpu, logical, Box::new(mirror));
        let game2d = Game2dContent::simple(&gpu, Color::rgb(0.09, 0.1, 0.13));
        let scene3d = Scene3dContent {
            screen_overlay: None,
            text_overlay: None,
            hud_overlay: None,
            mesh: agp::render3d::MeshPipeline::new(
                &gpu,
                agp::renderer::target::OffscreenTarget::FORMAT,
            )
            .map_err(|e| format!("3d pipeline: {e:?}"))?,
            store: Default::default(),
            instances: vec![],
            sky: [0.05, 0.07, 0.1, 1.0],
            view_proj: mixed_view_proj(),
        };
        let mut rt = SurfaceRuntime::new(gpu).map_err(|e| format!("runtime: {e:?}"))?;
        if let Some(fmt) = window_format {
            rt.set_window_format(fmt);
        }
        let ui_surface = rt
            .add_surface(logical, SurfaceContent::Iced(Box::new(ui_content)))
            .map_err(|e| format!("ui surface: {e:?}"))?;
        let s2d = rt
            .add_surface(REGION_CONTENT, SurfaceContent::Game2d(Box::new(game2d)))
            .map_err(|e| format!("2d surface: {e:?}"))?;
        let s3d = rt
            .add_surface(REGION_CONTENT, SurfaceContent::Scene3d(Box::new(scene3d)))
            .map_err(|e| format!("3d surface: {e:?}"))?;
        let ui_view = rt
            .add_view(ui_surface, true)
            .map_err(|e| format!("ui view: {e:?}"))?;
        rt.add_view(s2d, false)
            .map_err(|e| format!("2d view: {e:?}"))?;
        rt.add_view(s3d, false)
            .map_err(|e| format!("3d view: {e:?}"))?;
        // 初始呈现：区域场景命令在首帧前就绪（呈现者从 t0 反映已接受状态）。
        let initial = app.scene_state();
        present_on(&mut rt, s2d, s3d, &initial);
        Ok(MixedHost {
            rt,
            ci: CompositionInput::new(logical, logical),
            ui: ui_surface,
            s2d,
            s3d,
            app,
            ui_view,
            portals: Default::default(),
            last_mount: Default::default(),
            ui_events: Vec::new(),
            cursors: Default::default(),
            logical,
            frames: 0,
            steps: 0,
        })
    }

    /// Session owner 访问（宿主侧；组合内容只持镜像）。
    pub fn app(&mut self) -> &mut MixedUiApp {
        &mut self.app
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

    /// 显式逻辑步（app_update；仅此处，重绘永不代 Tick）。
    pub fn step(&mut self, dt: f32) -> Result<(), String> {
        self.steps += 1;
        self.app.step(dt)?;
        let presentation = self.app.presentation();
        self.sync_presentation(presentation);
        let states = self.app.scene_state();
        present_on(&mut self.rt, self.s2d, self.s3d, &states);
        Ok(())
    }

    /// resize：UI 逻辑尺寸＋GPU target＋图尺寸事务重建＋路由器尺寸；
    /// 挂载几何由后续帧的实际布局 SetMount 刷新。
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

    pub fn key(&mut self, key: PlatformKey) {
        self.app.ui.key(key);
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
                } else if press && surface == self.s2d {
                    let _ = self.app.enqueue_region_click(REGION2);
                    Some(REGION2)
                } else if press && surface == self.s3d {
                    let _ = self.app.enqueue_region_click(REGION3);
                    Some(REGION3)
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

    /// 一帧：挂载刷新＋镜像桥（出站→队列→drain→副本回推）＋场景重建
    /// ＋组合渲染/呈现。
    pub fn frame(
        &mut self,
        window_target: Option<&mut WindowTarget>,
    ) -> Result<FrameReport, String> {
        self.refresh_mounts();
        let mut events: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>> = Default::default();
        events.insert(self.ui, std::mem::take(&mut self.ui_events));
        let cursors = self.cursors.clone();
        self.rt
            .frame_with_cursors(window_target, &events, &cursors)
            .map_err(|e| format!("frame: {e:?}"))?;
        self.frames += 1;
        // Send 镜像桥：出站控件消息 → Session 队列 → 派发 → 副本回推。
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
        let dispatched = self.app.drain().dispatched;
        if dispatched > 0 {
            let presentation = self.app.presentation();
            self.sync_presentation(presentation);
            let states = self.app.scene_state();
            present_on(&mut self.rt, self.s2d, self.s3d, &states);
        }
        Ok(FrameReport {
            frames: self.frames,
            dispatched,
            steps: self.steps,
        })
    }

    fn sync_presentation(&mut self, presentation: UiPresentation) {
        let _ = self.rt.with_ui_mut(self.ui, |u| {
            if let Some(m) = u.as_any_mut().downcast_mut::<MixedMirror>() {
                m.presentation = presentation;
            }
        });
    }

    /// 区域实际 bounds → Flat2d 挂载（变化才 SetMount；真实布局驱动）。
    fn refresh_mounts(&mut self) {
        let reg2 = self.widget_bounds(REGION2_KEY);
        let reg3 = self.widget_bounds(REGION3_KEY);
        let mut ops: Vec<(&'static str, Rect, bool)> = Vec::new();
        if let Some(r) = reg2 {
            ops.push((REGION2, r, false));
        }
        if let Some(r) = reg3 {
            ops.push((REGION3, r, true));
        }
        for (key, rect, is3d) in ops {
            let mark = [rect.x, rect.y, rect.w, rect.h];
            if self.last_mount.get(key) == Some(&mark) {
                continue;
            }
            let mount = MountKind::Flat2d {
                offset: [rect.x, rect.y],
                scale: [rect.w / REGION_CONTENT[0], rect.h / REGION_CONTENT[1]],
                clip: rect,
            };
            let child = if is3d { self.s3d } else { self.s2d };
            match self.portals.get(key) {
                Some(pid) => {
                    let _ = self.rt.set_portal_mount(*pid, mount);
                }
                None => {
                    if let Ok(pid) = self.rt.add_portal(self.ui_view, child, mount, true) {
                        self.portals.insert(key.to_string(), pid);
                    }
                }
            }
            self.last_mount.insert(key.to_string(), mark);
        }
    }
}

/// 场景命令/实例重建（状态呈现者；纯缓存读，无 VM 访问）。
fn present_on(rt: &mut SurfaceRuntime, s2d: SurfaceId, s3d: SurfaceId, states: &[RegionState]) {
    for st in states {
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
                    s.view_proj = mixed_view_proj();
                });
            }
            _ => {}
        }
    }
}
