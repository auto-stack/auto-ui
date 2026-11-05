//! PLAN-054 组件/实例活区域适配器（driver 侧，schema v2）。
//!
//! 与 [`crate::node_mixed`]（049 v1，root 区域切片、固定装配）严格隔离：
//! 本档产物 schema `autographics.node-ui-mixed-regions/2`，区域声明放行到
//! **任意 def**——普通 `use` 组件内的具名区域按**实例路径**装配，身份＝
//! `{Run 代次, 实例路径, 区域局部名, 挂载代次}`。行为 ABI 与 048/049 完全
//! 同面：区域点击走同一 `{prefix}_ui_click_{region}(def_idx?)` 派发器、
//! 场景绑定走同一 int getter（键 `{实例路径}#{区域}#{元素}#{属性}`）——
//! 零新增世界 ABI 行。
//!
//! 条件挂卸/代次/装配在 [`crate::node_mixed_regions_host::MixedRegionsHost`]
//! （metadata 驱动，无固定 App/区域名）；本模块只提供解析＋区域目录＋
//! 实例寻址的场景状态与点击入队。

use crate::authoring::behavior::Api;
use crate::node_mixed::{RegionState, SceneElemState};
use crate::node_ui::{
    parse_artifact_common, ArtifactClick, ArtifactElement, ArtifactPropValue, NodeUiApp,
    RegionClaim, UiArtifact,
};

/// 产物 schema 身份（与 pilot `NODE_UI_MIXED_REGIONS_SCHEMA` 逐字一致）。
pub const NODE_UI_MIXED_REGIONS_SCHEMA: &str = "autographics.node-ui-mixed-regions/2";
/// 每 def 区域上限（adapter 侧复检；与 049 冻结数字一致）。
pub const MAX_REGIONS_PER_DEF: usize = 4;
/// 每区域场景元素上限（adapter 侧复检）。
pub const MAX_SCENE_ELEMS: usize = 8;
/// 区域占位边长上限（adapter 侧复检）。
pub const MAX_REGION_SIDE: i64 = 640;

/// 解析 v2 混合区域产物＋容量复检（具名、执行前拒绝）。与 v1 的唯一差别：
/// 非 root def 的区域按实例挂载身份放行（v1 入口对新 schema 具名拒绝，
/// 本入口拒 v1——双向隔离）。
pub fn parse_mixed_regions_artifact(json: &str) -> Result<UiArtifact, String> {
    let a = parse_artifact_common(json, NODE_UI_MIXED_REGIONS_SCHEMA)?;
    for (name, def) in &a.defs {
        let regions = def
            .elements
            .iter()
            .filter(|e| matches!(e, ArtifactElement::Embed { .. }))
            .count();
        if regions > MAX_REGIONS_PER_DEF {
            return Err(format!(
                "def '{name}': {regions} regions exceed the frozen {MAX_REGIONS_PER_DEF}"
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
            }
        }
    }
    Ok(a)
}

/// 一个 def 内的区域声明（def 静场面；实例化后身份由行路径给出）。
#[derive(Debug, Clone)]
pub struct RegionDecl {
    pub name: String,
    pub dim: String,
    pub width: i64,
    pub height: i64,
    /// 区域点击派发器（None = 纯呈现区域）。
    pub click: Option<ArtifactClick>,
}

fn collect_decls(els: &[ArtifactElement], out: &mut Vec<RegionDecl>) {
    for e in els {
        match e {
            ArtifactElement::Embed {
                name,
                dim,
                width,
                height,
                click,
                ..
            } => out.push(RegionDecl {
                name: name.clone(),
                dim: dim.clone(),
                width: *width,
                height: *height,
                click: click.clone(),
            }),
            ArtifactElement::Container { children, .. } => collect_decls(children, out),
            ArtifactElement::If { then, .. } => collect_decls(then, out),
            _ => {}
        }
    }
}

/// 一个实例行上的区域条目（身份键 `{实例路径}#{区域局部名}`）。
#[derive(Debug, Clone)]
pub struct RegionEntry {
    pub key: String,
    /// 实例路径（root＝def 名）。
    pub path: String,
    /// 区域局部名。
    pub name: String,
    pub dim: String,
    pub width: i64,
    pub height: i64,
    /// 实例行下标（rows() 序）。
    pub instance: usize,
    /// def 层实例下标（非 root 派发器实参）。
    pub def_idx: usize,
    /// 区域点击派发器（None = 纯呈现区域）。
    pub dispatch: Option<String>,
}

/// 混合区域应用：一个 NodeUiApp（唯一 Session owner）＋实例寻址的区域面。
pub struct MixedRegionsUiApp {
    pub ui: NodeUiApp,
}

impl MixedRegionsUiApp {
    /// 构造（不启动 Run）。schema/容量门＋行为源身份链由
    /// [`NodeUiApp::new_with_artifact`] 复检（损坏副本具名拒绝）。
    pub fn new(artifact_json: &str, world_at: &str, name: &str) -> Result<Self, String> {
        let artifact = parse_mixed_regions_artifact(artifact_json)?;
        let ui = NodeUiApp::new_with_artifact(artifact, world_at, name, Api::MixedRegions2)?;
        Ok(MixedRegionsUiApp { ui })
    }

    /// 启动一个新 Run（一次 app_start；区域场景缓存随 UI 缓存同批就绪）。
    pub fn start_run(&mut self) -> Result<(), String> {
        self.ui.start_run()
    }

    /// 一个逻辑步（仅显式调用；重绘/layout 永不进入此处）。
    pub fn step(&mut self, dt: f32) -> Result<(), String> {
        self.ui.step(dt)
    }

    /// 队列派发（陈旧 Run 代次/区域挂载代次具名拒绝；故障相关域停止）。
    pub fn drain(&mut self) -> crate::node_ui::DrainReport {
        self.ui.drain()
    }

    /// 已接受状态副本（镜像回推面；有界）。
    pub fn presentation(&self) -> crate::node_ui::UiPresentation {
        self.ui.presentation()
    }

    /// 全部实例行上的区域声明（metadata 目录；装配唯一来源——无固定名）。
    /// 条件上下文不在此判定（活区域表由 host 按真实布局/条件逐帧决定）。
    pub fn region_catalog(&self) -> Vec<RegionEntry> {
        let mut out = Vec::new();
        for (idx, row) in self.ui.rows().iter().enumerate() {
            let mut decls = Vec::new();
            collect_decls(&row.elements, &mut decls);
            for d in decls {
                let dispatch = d
                    .click
                    .as_ref()
                    .map(|c: &ArtifactClick| c.fn_name.clone());
                out.push(RegionEntry {
                    key: format!("{}#{}", row.path, d.name),
                    path: row.path.clone(),
                    name: d.name,
                    dim: d.dim,
                    width: d.width,
                    height: d.height,
                    instance: idx,
                    def_idx: row.def_idx,
                    dispatch,
                });
            }
        }
        out
    }

    /// 区域点击入队（实例寻址）。`epoch`＝点击命中时 host 观察到的该区域
    /// 挂载代次；drain 派发前与注册表对账（卸载/重挂 → 具名拒绝）。
    /// 与按钮同一有界队列与派发器 ABI（非 root 带 def_idx）。
    pub fn enqueue_region_click_at(&mut self, path: &str, region: &str, epoch: u64) -> bool {
        enum Resolved {
            Missing,
            Presentational,
            Click {
                dispatch: String,
                arg: Option<usize>,
            },
        }
        let resolved = match self.ui.instance_index(path) {
            None => Resolved::Missing,
            Some(idx) => {
                let rows = self.ui.rows();
                let row = &rows[idx];
                let mut decls = Vec::new();
                collect_decls(&row.elements, &mut decls);
                match decls.iter().find(|d| d.name == region) {
                    None => Resolved::Missing,
                    Some(d) => match &d.click {
                        None => Resolved::Presentational,
                        Some(click) => Resolved::Click {
                            dispatch: click.fn_name.clone(),
                            arg: if row.is_root { None } else { Some(row.def_idx) },
                        },
                    },
                }
            }
        };
        match resolved {
            Resolved::Missing | Resolved::Presentational => false,
            Resolved::Click { dispatch, arg } => {
                let key = format!("{path}#{region}");
                self.ui.enqueue_click_claimed(
                    dispatch,
                    arg,
                    key.clone(),
                    Some(RegionClaim { key, epoch }),
                )
            }
        }
    }

    /// 全部实例行上区域的已接受场景状态（纯缓存读；view/render 路径无 VM
    /// 访问）。键面与 collect_bindings 一致：`{实例路径}#{区域}#{元素}#{属性}`。
    pub fn scene_state(&self) -> Vec<RegionState> {
        let mut out = Vec::new();
        for row in self.ui.rows() {
            let mut decls = Vec::new();
            collect_decls(&row.elements, &mut decls);
            for d in decls {
                let num = |prop: &ArtifactPropValue, key: String| -> f32 {
                    match prop {
                        ArtifactPropValue::LitNum { v } => *v as f32,
                        ArtifactPropValue::Field { .. } => {
                            self.ui.layout_value(&key).unwrap_or(0.0)
                        }
                    }
                };
                // 场景元素从 def 静态树取（声明序）；绑定值按实例路径键读。
                let def = &self.ui.artifact.defs[&row.def];
                let scene: &[crate::node_ui::ArtifactSceneElem] =
                    find_scene(&def.elements, &d.name).map(|v| v.as_slice()).unwrap_or(&[]);
                let elems = scene
                    .iter()
                    .map(|el| match el {
                        crate::node_ui::ArtifactSceneElem::Quad {
                            name: en,
                            x,
                            y,
                            w,
                            h,
                            color,
                        } => SceneElemState::Quad {
                            name: en.clone(),
                            x: num(x, format!("{}#{}#{}#x", row.path, d.name, en)),
                            y: num(y, format!("{}#{}#{}#y", row.path, d.name, en)),
                            w: num(w, format!("{}#{}#{}#w", row.path, d.name, en)),
                            h: num(h, format!("{}#{}#{}#h", row.path, d.name, en)),
                            color: *color,
                        },
                        crate::node_ui::ArtifactSceneElem::Circle {
                            name: en,
                            x,
                            y,
                            r,
                            color,
                        } => SceneElemState::Circle {
                            name: en.clone(),
                            x: num(x, format!("{}#{}#{}#x", row.path, d.name, en)),
                            y: num(y, format!("{}#{}#{}#y", row.path, d.name, en)),
                            r: num(r, format!("{}#{}#{}#r", row.path, d.name, en)),
                            color: *color,
                        },
                        crate::node_ui::ArtifactSceneElem::Cylinder {
                            name: en,
                            translation,
                            color,
                        } => {
                            let t = |i: usize, tag: &str| {
                                translation
                                    .get(i)
                                    .map(|v| num(v, format!("{}#{}#{}#{}", row.path, d.name, en, tag)))
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
                    path: row.path.clone(),
                    name: d.name.clone(),
                    dim: d.dim.clone(),
                    width: d.width as f32,
                    height: d.height as f32,
                    elems,
                });
            }
        }
        out
    }
}

/// 在 def 元素树内找区域的场景元素集（声明序；同 v1 collect 语义）。
fn find_scene<'a>(
    els: &'a [ArtifactElement],
    region: &str,
) -> Option<&'a Vec<crate::node_ui::ArtifactSceneElem>> {
    fn rec<'a>(
        els: &'a [ArtifactElement],
        region: &str,
    ) -> Option<&'a Vec<crate::node_ui::ArtifactSceneElem>> {
        for e in els {
            match e {
                ArtifactElement::Embed { name, scene, .. } if name == region => {
                    return Some(scene)
                }
                ArtifactElement::Container { children, .. } => {
                    if let Some(s) = rec(children, region) {
                        return Some(s);
                    }
                }
                ArtifactElement::If { then, .. } => {
                    if let Some(s) = rec(then, region) {
                        return Some(s);
                    }
                }
                _ => {}
            }
        }
        None
    }
    rec(els, region)
}
