//! PLAN-049 混合组合适配器（driver 侧）。
//!
//! 消费版本化混合产物（`ui.json`，schema `autographics.node-ui-mixed/1`；
//! 048 的元素树/实例表/派发 ABI/容量常数**完全复用**），把嵌入区域呈现为
//! 真实布局占位（048 adapter 接缝），场景状态从**同一已接受状态缓存**读取
//! （区域 Field 绑定经既有 int getter——无新增 VM 调用面），区域点击经
//! 同一 `_ui_click_` 派发器 ABI 入同一有界队列。
//!
//! 单 owner 证据面：一个 [`MixedUiApp`] ＝ 一个 `Session`；`app_start` 每
//! Run 一次、`app_update(dt)` 每显式逻辑步一次——UI 文字、2D 四边形、3D
//! 圆柱全部是同一接受状态的呈现者，重绘/layout 永不代 Tick。
//!
//! T-02 冻结切片：区域仅 root def（非 root 实例的区域需要多挂载实例身份，
//! 具名拒绝——T-04 随实例挂载身份扩面）。

use crate::authoring::behavior::Api;
use crate::node_ui::{
    parse_artifact_common, ArtifactClick, ArtifactElement, ArtifactPropValue, ArtifactSceneElem,
    DrainReport, NodeUiApp, UiArtifact, UiPresentation,
};
use agp::host::iced_surface::{SurfaceElement, SurfaceUi};
use std::collections::VecDeque;

/// 产物 schema 身份（与 pilot `NODE_UI_MIXED_SCHEMA` 逐字一致）。
pub const NODE_UI_MIXED_SCHEMA: &str = "autographics.node-ui-mixed/1";
/// 每 def 区域上限（adapter 侧复检；与 pilot 冻结面一致）。
pub const MAX_REGIONS: usize = 4;
/// 每区域场景元素上限（adapter 侧复检）。
pub const MAX_SCENE_ELEMS: usize = 8;
/// 区域占位边长上限（adapter 侧复检）。
pub const MAX_REGION_SIDE: i64 = 640;

/// 混合产物：048 解析面复用（元素表内含 Embed 变体；schema 门不同）。
pub fn parse_mixed_artifact(json: &str) -> Result<UiArtifact, String> {
    let a = parse_artifact_common(json, NODE_UI_MIXED_SCHEMA)?;
    for (name, def) in &a.defs {
        let regions = def
            .elements
            .iter()
            .filter(|e| matches!(e, ArtifactElement::Embed { .. }))
            .count();
        if regions > MAX_REGIONS {
            return Err(format!(
                "def '{name}': {regions} regions exceed the frozen {MAX_REGIONS}"
            ));
        }
        for e in &def.elements {
            if let ArtifactElement::Embed {
                name: rn,
                dim,
                width,
                height,
                scene,
                ..
            } = e
            {
                if dim != "2d" && dim != "3d" {
                    return Err(format!(
                        "def '{name}': region '{rn}' dim '{dim}' is not 2d|3d"
                    ));
                }
                if !(1..=MAX_REGION_SIDE).contains(width) || !(1..=MAX_REGION_SIDE).contains(height)
                {
                    return Err(format!(
                        "def '{name}': region '{rn}' {width}x{height} exceeds the frozen 1..={MAX_REGION_SIDE} side budget"
                    ));
                }
                if scene.len() > MAX_SCENE_ELEMS {
                    return Err(format!(
                        "def '{name}': region '{rn}' has {} scene elements (frozen {MAX_SCENE_ELEMS})",
                        scene.len()
                    ));
                }
                if *name != a.root {
                    return Err(format!(
                        "def '{name}': region '{rn}' in a non-root def requires instance-mount identity (frozen T-02 slice admits root regions only)"
                    ));
                }
            }
        }
    }
    Ok(a)
}

/// 一个区域的已接受场景状态（纯缓存读；无 VM 访问）。
#[derive(Debug, Clone)]
pub struct RegionState {
    pub path: String,
    pub name: String,
    /// "2d" | "3d"。
    pub dim: String,
    pub width: f32,
    pub height: f32,
    pub elems: Vec<SceneElemState>,
}

#[derive(Debug, Clone)]
pub enum SceneElemState {
    Quad {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [u8; 3],
    },
    Circle {
        name: String,
        x: f32,
        y: f32,
        r: f32,
        color: [u8; 3],
    },
    Cylinder {
        name: String,
        tx: f32,
        ty: f32,
        tz: f32,
        color: [u8; 3],
    },
}

/// 混合应用：一个 NodeUiApp（唯一 Session owner）＋区域路由面。
pub struct MixedUiApp {
    pub ui: NodeUiApp,
    /// root def 的区域点击端口（与 048 按钮同一派发 ABI；root 派发器零参）。
    clicks: Vec<(String, String, String)>, // (region, dispatch, origin)
}

impl MixedUiApp {
    /// 构造（不启动 Run）。schema 门＋容量复检＋root 区域切片；行为源身份
    /// 链由 [`NodeUiApp::new_with_artifact`] 复检（损坏副本具名拒绝）。
    pub fn new(artifact_json: &str, world_at: &str, name: &str) -> Result<Self, String> {
        let artifact = parse_mixed_artifact(artifact_json)?;
        let root = artifact.root.clone();
        let mut clicks = Vec::new();
        if let Some(def) = artifact.defs.get(&root) {
            fn walk(
                els: &[ArtifactElement],
                root: &str,
                clicks: &mut Vec<(String, String, String)>,
            ) {
                for e in els {
                    match e {
                        ArtifactElement::Embed {
                            name: rn,
                            click: Some(ArtifactClick { fn_name, .. }),
                            ..
                        } => clicks.push((rn.clone(), fn_name.clone(), format!("{root}#{rn}"))),
                        ArtifactElement::Container { children, .. } => walk(children, root, clicks),
                        ArtifactElement::If { then, .. } => walk(then, root, clicks),
                        _ => {}
                    }
                }
            }
            walk(&def.elements, &root, &mut clicks);
        }
        let ui = NodeUiApp::new_with_artifact(artifact, world_at, name, Api::MixedUi2d3d)?;
        Ok(MixedUiApp { ui, clicks })
    }

    /// 启动一个新 Run（一次 app_start；场景缓存随 UI 缓存同批就绪——
    /// 同一 accept_state 遍历读取区域 Field 绑定）。
    pub fn start_run(&mut self) -> Result<(), String> {
        self.ui.start_run()
    }

    /// 一个逻辑步（仅显式调用；重绘/layout 永不进入此处）。
    pub fn step(&mut self, dt: f32) -> Result<(), String> {
        self.ui.step(dt)
    }

    /// 队列派发（陈旧具名拒绝；故障相关域停止）。
    pub fn drain(&mut self) -> DrainReport {
        self.ui.drain()
    }

    /// 已接受状态副本（镜像回推面；有界）。
    pub fn presentation(&self) -> UiPresentation {
        self.ui.presentation()
    }

    /// 区域点击入队（同一有界队列；满则具名丢弃计数——与按钮同责）。
    pub fn enqueue_region_click(&mut self, region: &str) -> bool {
        let Some((_, dispatch, origin)) = self
            .clicks
            .iter()
            .find(|(rn, _, _)| rn == region)
            .map(|(r, d, o)| (r.clone(), d.clone(), o.clone()))
        else {
            return false;
        };
        self.ui.enqueue_click(0, dispatch, None, origin)
    }

    /// 全部区域的已接受场景状态（纯缓存读；view/render 路径无 VM 访问）。
    pub fn scene_state(&self) -> Vec<RegionState> {
        let mut out = Vec::new();
        for (def_name, def) in &self.ui.artifact.defs {
            let mut embeds = Vec::new();
            fn collect<'a>(els: &'a [ArtifactElement], embeds: &mut Vec<&'a ArtifactElement>) {
                for e in els {
                    match e {
                        ArtifactElement::Embed { .. } => embeds.push(e),
                        ArtifactElement::Container { children, .. } => collect(children, embeds),
                        ArtifactElement::If { then, .. } => collect(then, embeds),
                        _ => {}
                    }
                }
            }
            collect(&def.elements, &mut embeds);
            for e in embeds {
                let ArtifactElement::Embed {
                    name,
                    dim,
                    width,
                    height,
                    scene,
                    ..
                } = e
                else {
                    continue;
                };
                let num = |prop: &ArtifactPropValue, key: String| -> f32 {
                    match prop {
                        ArtifactPropValue::LitNum { v } => *v as f32,
                        ArtifactPropValue::Field { .. } => {
                            self.ui.layout_value(&key).unwrap_or(0.0)
                        }
                    }
                };
                let elems = scene
                    .iter()
                    .map(|el| match el {
                        ArtifactSceneElem::Quad {
                            name: en,
                            x,
                            y,
                            w,
                            h,
                            color,
                        } => SceneElemState::Quad {
                            name: en.clone(),
                            x: num(x, format!("{def_name}#{name}#{en}#x")),
                            y: num(y, format!("{def_name}#{name}#{en}#y")),
                            w: num(w, format!("{def_name}#{name}#{en}#w")),
                            h: num(h, format!("{def_name}#{name}#{en}#h")),
                            color: *color,
                        },
                        ArtifactSceneElem::Circle {
                            name: en,
                            x,
                            y,
                            r,
                            color,
                        } => SceneElemState::Circle {
                            name: en.clone(),
                            x: num(x, format!("{def_name}#{name}#{en}#x")),
                            y: num(y, format!("{def_name}#{name}#{en}#y")),
                            r: num(r, format!("{def_name}#{name}#{en}#r")),
                            color: *color,
                        },
                        ArtifactSceneElem::Cylinder {
                            name: en,
                            translation,
                            color,
                        } => {
                            let t = |i: usize, tag: &str| {
                                translation
                                    .get(i)
                                    .map(|v| num(v, format!("{def_name}#{name}#{en}#{tag}")))
                                    .unwrap_or(0.0)
                            };
                            SceneElemState::Cylinder {
                                name: en.clone(),
                                tx: t(0, "tx"),
                                ty: t(1, "ty"),
                                tz: t(2, "tz"),
                                color: *color,
                            }
                        }
                    })
                    .collect();
                out.push(RegionState {
                    path: def_name.clone(),
                    name: name.clone(),
                    dim: dim.clone(),
                    width: *width as f32,
                    height: *height as f32,
                    elems,
                });
            }
        }
        out
    }
}

/// 运行时路径镜像（Send）：仅携带已接受状态的版本化副本＋出站消息队列
/// ——Session 不进入组合内容（auto-lang-core 持 Rc，线程受限）。
/// 控件树与 048 直连路径同一份代码（`UiPresentationView`）。
pub struct MixedMirror {
    /// 最近一次推入的已接受状态（drain/step 后由宿主刷新）。
    pub presentation: UiPresentation,
    /// 控件消息出站（有界；满则具名丢弃计数）。
    pub outbound: VecDeque<String>,
    /// 出站满被丢弃的消息数（对账面）。
    pub outbound_dropped: usize,
}

impl MixedMirror {
    pub fn new(presentation: UiPresentation) -> Self {
        MixedMirror {
            presentation,
            outbound: VecDeque::new(),
            outbound_dropped: 0,
        }
    }
}

impl SurfaceUi for MixedMirror {
    type Message = String;

    fn view(&self) -> SurfaceElement<'_, String> {
        self.presentation.presentation_view().view_string()
    }

    fn update(&mut self, message: String) {
        const MIRROR_QUEUE_CAP: usize = 64;
        if self.outbound.len() >= MIRROR_QUEUE_CAP {
            self.outbound_dropped += 1;
            return;
        }
        self.outbound.push_back(message);
    }

    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::WHITE
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
