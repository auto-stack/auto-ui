//! PLAN-019 `--profile scene-2d|character-2d|camera-2d` 降层（T-02 冻结）。
//!
//! 基础二维作者入口：零或一个角色、有限静态 solid、可选跟随相机、
//! 纯视觉叶子（shape2d）与同维父组（group2d）。与 g11-2d 的关系：
//! 共享 load/translate/scene 基础设施与 JSON 形状，不复制表达式翻译；
//! 玩法逻辑（gravity/vy/speed/jump 条件）全部在作者 model/on，经共享
//! translate 进入生成用户源，Rust 不实现任何规则。
//!
//! 冻结面（T-02，Plan §9 摘要）：
//! - `shape2d name { kind: circle|quad, position: (x,y), size: (w,h),
//!   material: resources.<color>, visible: true|false? }` —— 纯视觉叶子；
//!   visible=false 保对象身份（场景行 visible=false，不是 if 销毁）；
//!   circle 要求 size.x==size.y（直径）。
//! - `group2d name { position: (x,y)?, visible: bool? } { children }` ——
//!   同维父组，仅平移/可见性继承；仅 scene-2d 允许。
//! - `character2d name { position: (x,y), material: resources.<color> }`
//!   —— 恰一个（character-2d/camera-2d 根 view），盒体钉 0.8×1.0。
//! - `solid2d name { position: (x,y), mesh: resources.<mesh>,
//!   material: resources.<color> }` —— 静态 solid（可见 + 碰撞），仅根
//!   view；声明序生成 ag_char_solid。
//! - `camera2d name { target: refs.<char>.actor, bounds: (min_x,min_y,
//!   max_x,max_y), view_size: (w,h) }` —— camera-2d 恰一个，仅根 view。
//! - 服务拼写：`input.axis(Left, Right)`、`input.pressed(Jump|Enter)`、
//!   `refs.<char>.move_and_slide((dx,dy))`、`refs.<char>.grounded(q)`、
//!   `refs.<char>.support(q)`。
//! - 组件节点（实例化）：参数 + 纯 view（仅 shape2d）；无 model/msg/on。
//! - scene-2d 拒绝 character2d/solid2d/camera2d/trigger2d/model/msg/on/if；
//!   character-2d 拒绝 camera2d/trigger2d/if/group2d；camera-2d 恰一相机。
//! - 场景 id 1 起、对象/组共享命名空间、按声明序深度优先；对象数组即
//!   2D 绘制序；生成 ID 不必等于旧示例 ID（具名 roles 映射定位）。

use auto_parser_api::ast::{Expr, Type};
use auto_ui_dialect::node2d::{NodeDecl, Prop, PropValue, View2dElement};

use crate::load::{node_of, Project, ResourceKind};
use crate::lower::numeric_lit;
use crate::translate::LowerError;

/// 基础二维 profile（Plan §2 三个新作者 profile）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicProfile {
    /// 零角色纯视觉/层级场景（G01/G03）。
    Scene2D,
    /// 恰一角色 + 静态 solids（G06），无相机。
    Character2D,
    /// character-2d + 恰一跟随相机（G08）。
    Camera2D,
}

impl BasicProfile {
    pub fn api_version(self) -> Option<&'static str> {
        match self {
            BasicProfile::Scene2D => None,
            // 受信 assembly 版本串（driver Api::parse 接受的精确值）。
            BasicProfile::Character2D => Some("autographics.character/1"),
            BasicProfile::Camera2D => Some("autographics.runtime_camera2d/1"),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            BasicProfile::Scene2D => "scene-2d",
            BasicProfile::Character2D => "character-2d",
            BasicProfile::Camera2D => "camera-2d",
        }
    }
}

pub struct LowerOutputBasic2D {
    /// scene-2d 为 None（behavior=null，无用户脚本）。
    pub user_source: Option<String>,
    pub scene_json: String,
    pub source_sha256: String,
    pub source_bytes: usize,
    /// 角色/solid/相机/对象 → 场景 id 映射（identity.json 的 roles 段）。
    pub roles_json: String,
}

pub(crate) fn unsupported(what: impl Into<String>) -> LowerError {
    LowerError::Unsupported {
        what: what.into(),
        line: 0,
    }
}

/// 展开后的 scene 节点（对象或组，共享 id 命名空间）。
pub struct SceneNode {
    pub is_group: bool,
    pub id: usize,
    pub name: String,
    pub kind: String,
    pub tx: f64,
    pub ty: f64,
    pub sx: f64,
    pub sy: f64,
    pub rgba: (f64, f64, f64, f64),
    pub visible: bool,
    pub parent: Option<usize>,
    pub is_character: bool,
    pub is_solid: bool,
    /// PLAN-022 sprites-2d：本行的场景 sprite 绑定（旧档恒 None）。
    pub sprite: Option<SceneSprite>,
}

/// PLAN-022：场景对象 sprite 绑定（document SpriteBinding 的生成侧形态）。
#[derive(Clone, Debug)]
pub struct SceneSprite {
    /// 图像完整原始 SHA256（不带 sha256: 前缀）。
    pub asset: String,
    /// 默认帧整数像素矩形 [x, y, w, h]。
    pub frame: [u32; 4],
    /// 默认水平翻向。
    pub flip: bool,
}

pub struct CameraCfg {
    pub target_char_name: String,
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub view_w: f64,
    pub view_h: f64,
}

pub struct Collected {
    pub nodes: Vec<SceneNode>,
    pub camera: Option<CameraCfg>,
    pub char_name: Option<String>,
}

/// 属性取值：数值字面量/组件参数、布尔字面量、`resources.<name>`、组字面量。
#[derive(Debug)]
enum PropVal {
    Num(f64),
    Bool(bool),
    Resource(String),
    Tuple(Vec<f64>),
}

/// 组件参数绑定值：数值或资源 symbol（Material2D/Mesh2D 端口）。
#[derive(Debug, Clone)]
pub(crate) enum ParamVal {
    Num(f64),
    Res(String),
}

fn literal_or_param(e: &Expr, params: &[(String, ParamVal)]) -> Result<f64, LowerError> {
    if let Some(v) = numeric_lit(e) {
        return Ok(v);
    }
    if let Expr::Ident(p) = e {
        if let Some((_, ParamVal::Num(v))) = params.iter().find(|(n, _)| n == p.as_str()) {
            return Ok(*v);
        }
        return Err(unsupported(format!(
            "view prop references unknown or non-numeric param '{p}'"
        )));
    }
    // `.param` 前导点成员形式（组件内成员引用）。
    if let Expr::Dot(inner, p) = e {
        if let Expr::Ident(base) = &**inner {
            if base == "self" {
                if let Some((_, ParamVal::Num(v))) = params.iter().find(|(n, _)| n == p.as_str()) {
                    return Ok(*v);
                }
                return Err(unsupported(format!(
                    "view prop references unknown or non-numeric param '{p}'"
                )));
            }
        }
    }
    Err(unsupported(format!("view prop expression {e:?}")))
}

/// 资源 symbol 参数解引用：`material: <param>` → `resources.<name>`。
fn resource_ref_of_param(e: &Expr, params: &[(String, ParamVal)]) -> Option<String> {
    if let Expr::Ident(p) = e {
        if let Some((_, ParamVal::Res(r))) = params.iter().find(|(n, _)| n == p.as_str()) {
            return Some(r.clone());
        }
        return None;
    }
    if let Expr::Dot(inner, p) = e {
        if let Expr::Ident(base) = &**inner {
            if base == "self" {
                if let Some((_, ParamVal::Res(r))) = params.iter().find(|(n, _)| n == p.as_str()) {
                    return Some(r.clone());
                }
            }
        }
    }
    None
}

fn resolve_prop(pr: &Prop, params: &[(String, ParamVal)]) -> Result<PropVal, LowerError> {
    match &pr.value {
        PropValue::One(e) => {
            if let Expr::Bool(b) = e {
                return Ok(PropVal::Bool(*b));
            }
            if let Expr::Dot(outer, m) = e {
                if let Expr::Ident(base) = &**outer {
                    if base == "resources" {
                        return Ok(PropVal::Resource(m.to_string()));
                    }
                }
            }
            Ok(PropVal::Num(literal_or_param(e, params)?))
        }
        PropValue::Pair(items) => {
            let mut out = Vec::new();
            for e in items {
                out.push(literal_or_param(e, params)?);
            }
            Ok(PropVal::Tuple(out))
        }
    }
}

pub(crate) fn prop_of<'a>(props: &'a [Prop], name: &str) -> Option<&'a Prop> {
    props.iter().find(|p| p.name == name)
}

pub(crate) fn resource_color(
    project: &Project,
    name: &str,
) -> Result<(f64, f64, f64, f64), LowerError> {
    let module = project
        .modules
        .get("resources")
        .ok_or_else(|| unsupported("resources module missing"))?;
    for r in &module.resources {
        if r.name == name {
            return match r.kind {
                ResourceKind::Color(r, g, b, a) => Ok((r, g, b, a)),
                _ => Err(unsupported(format!(
                    "resource '{name}' is not a material2d.color"
                ))),
            };
        }
    }
    Err(unsupported(format!("color resource '{name}' not found")))
}

pub(crate) fn resource_mesh(
    project: &Project,
    name: &str,
) -> Result<(String, f64, f64), LowerError> {
    let module = project
        .modules
        .get("resources")
        .ok_or_else(|| unsupported("resources module missing"))?;
    for r in &module.resources {
        if r.name == name {
            return match r.kind {
                ResourceKind::Quad(w, h) => Ok(("quad".into(), w, h)),
                ResourceKind::Circle(d) => Ok(("circle".into(), d, d)),
                ResourceKind::Color(..) => Err(unsupported(format!(
                    "resource '{name}' is not a mesh2d geometry"
                ))),
                ResourceKind::Image { .. }
                | ResourceKind::Frame { .. }
                | ResourceKind::Model3D { .. }
                | ResourceKind::Model3DTextured { .. }
                | ResourceKind::Model3DAnimated { .. }
                | ResourceKind::Model3D { .. } => Err(unsupported(format!(
                    "resource '{name}' is not a mesh2d geometry"
                ))),
            };
        }
    }
    Err(unsupported(format!("mesh resource '{name}' not found")))
}

/// 实例参数 → 绑定表（数值字面量或资源 symbol）。
fn instance_params(
    project: &Project,
    node_name: &str,
    props: &[Prop],
) -> Result<Vec<(String, ParamVal)>, LowerError> {
    let node = node_of(project, node_name);
    let mut out = Vec::new();
    for (pname, ty) in &node.params {
        let pr = prop_of(props, pname).ok_or_else(|| {
            unsupported(format!(
                "instance missing prop '{pname}' for component '{node_name}'"
            ))
        })?;
        let raw: String = match &pr.value {
            PropValue::One(e) => match e {
                Expr::Dot(outer, m) => {
                    if let Expr::Ident(base) = &**outer {
                        if base == "resources" {
                            m.to_string()
                        } else {
                            return Err(unsupported(format!(
                                "component param '{pname}' binding must be resources.<name>"
                            )));
                        }
                    } else {
                        return Err(unsupported(format!(
                            "component param '{pname}' binding must be resources.<name>"
                        )));
                    }
                }
                Expr::Float(..) | Expr::Int(_) | Expr::Unary(_, _) => {
                    match numeric_lit(e) {
                        Some(_) => String::new(),
                        None => {
                            return Err(unsupported(format!(
                                "component param '{pname}' literal invalid"
                            )))
                        }
                    }
                }
                _ => {
                    return Err(unsupported(format!(
                        "component param '{pname}' binding must be a numeric literal or resources.<name>"
                    )))
                }
            },
            _ => {
                return Err(unsupported(format!(
                    "component param '{pname}' binding must be a single value"
                )))
            }
        };
        match ty {
            Type::Float | Type::Int | Type::Double | Type::Byte => {
                let v = match resolve_prop(pr, &[])? {
                    PropVal::Num(v) => v,
                    other => {
                        return Err(unsupported(format!(
                            "component param '{pname}' must bind a numeric literal, got {other:?}"
                        )))
                    }
                };
                out.push((pname.clone(), ParamVal::Num(v)));
            }
            Type::User(d) if d.name == "Material2D" || d.name == "Mesh2D" => {
                out.push((pname.clone(), ParamVal::Res(raw)));
            }
            other => {
                return Err(unsupported(format!(
                    "component param '{pname}' has unsupported type {other:?} (basic2d: float/int/Material2D/Mesh2D)"
                )))
            }
        }
    }
    Ok(out)
}

/// 展开一个 view 元素列表（声明序深度优先；ids 共享命名空间）。
/// `allow_character`：组件 view 内的 character2d（脚本 profile 的共享
/// Player 定义）；scene-2d 一律拒绝。
fn expand_elements(
    project: &Project,
    elements: &[View2dElement],
    params: &[(String, ParamVal)],
    parent: Option<usize>,
    out: &mut Collected,
    next_id: &mut usize,
    allow_character: bool,
) -> Result<(), LowerError> {
    // 同 view 元素名唯一（roles 具名映射要求无歧义）。
    let mut seen: Vec<&str> = Vec::new();
    for el in elements {
        let ename = match el {
            View2dElement::Shape { name, .. }
            | View2dElement::Group { name, .. }
            | View2dElement::Solid { name, .. }
            | View2dElement::Trigger { name, .. }
            | View2dElement::Character { name, .. }
            | View2dElement::Camera { name, .. }
            | View2dElement::Instance { name, .. } => name.as_str(),
            View2dElement::If { .. } => "",
        };
        if !ename.is_empty() {
            if seen.contains(&ename) {
                return Err(unsupported(format!(
                    "duplicate view element name '{ename}' (basic2d: names are identity)"
                )));
            }
            seen.push(ename);
        }
    }
    for el in elements {
        match el {
            View2dElement::Shape { name, props } => {
                let id = *next_id;
                *next_id += 1;
                out.nodes
                    .push(shape_node(project, name, props, params, parent, id)?);
            }
            View2dElement::Group {
                name,
                props,
                children,
            } => {
                let id = *next_id;
                *next_id += 1;
                let mut tx = 0.0;
                let mut ty = 0.0;
                let mut visible = true;
                for pr in props {
                    match (pr.name.as_str(), resolve_prop(pr, params)?) {
                        ("position", PropVal::Tuple(v)) if v.len() == 2 => {
                            tx = v[0];
                            ty = v[1];
                        }
                        ("visible", PropVal::Bool(b)) => visible = b,
                        (other, _) => {
                            return Err(unsupported(format!(
                                "unknown group2d prop '{other}' (basic2d: position/visible)"
                            )))
                        }
                    }
                }
                out.nodes.push(SceneNode {
                    is_group: true,
                    id,
                    name: name.clone(),
                    kind: "group".into(),
                    tx,
                    ty,
                    sx: 1.0,
                    sy: 1.0,
                    rgba: (0.0, 0.0, 0.0, 0.0),
                    visible,
                    parent,
                    is_character: false,
                    is_solid: false,
                    sprite: None,
                });
                expand_elements(
                    project,
                    children,
                    params,
                    Some(id),
                    out,
                    next_id,
                    allow_character,
                )?;
            }
            View2dElement::Instance { node, name, props } => {
                let iparams = instance_params(project, node, props)?;
                let def = node_of(project, node);
                let view = def
                    .view2d
                    .as_ref()
                    .ok_or_else(|| unsupported(format!("component '{node}' has no view-2d")))?;
                // 组件实例：对象名 `<instance>.<shape>`，继承 parent。
                let mut sub = Collected {
                    nodes: Vec::new(),
                    camera: None,
                    char_name: None,
                };
                let mut sub_id = *next_id;
                expand_elements(
                    project,
                    &view.elements,
                    &iparams,
                    parent,
                    &mut sub,
                    &mut sub_id,
                    allow_character,
                )?;
                if let Some(cn) = sub.char_name {
                    out.char_name = Some(format!("{name}.{cn}"));
                }
                for mut n in sub.nodes {
                    n.name = format!("{name}.{}", n.name);
                    out.nodes.push(n);
                }
                *next_id = sub_id;
            }
            View2dElement::If { .. } => {
                return Err(unsupported(
                    "if in basic2d profiles (static scene: visible=false keeps identity)",
                ))
            }
            View2dElement::Solid { name, .. } => {
                return Err(unsupported(format!(
                    "solid2d '{name}' outside character-2d/camera-2d root view"
                )))
            }
            View2dElement::Trigger { name, .. } => {
                return Err(unsupported(format!(
                    "trigger2d '{name}' outside g11-2d profile"
                )))
            }
            View2dElement::Character { name, props } => {
                if !allow_character {
                    return Err(unsupported(format!(
                        "character2d '{name}' outside character-2d/camera-2d profiles"
                    )));
                }
                if out.char_name.is_some() {
                    return Err(unsupported(format!(
                        "duplicate character2d '{name}' (basic2d: exactly one)"
                    )));
                }
                let id = *next_id;
                *next_id += 1;
                out.nodes
                    .push(character_node(project, name, props, params, parent, id)?);
                out.char_name = Some(name.clone());
            }
            View2dElement::Camera { name, .. } => {
                return Err(unsupported(format!(
                    "camera2d '{name}' outside camera-2d root view"
                )))
            }
        }
    }
    Ok(())
}

/// shape2d → 纯视觉场景对象（不进入碰撞/角色域）。
fn shape_node(
    project: &Project,
    name: &str,
    props: &[Prop],
    params: &[(String, ParamVal)],
    parent: Option<usize>,
    id: usize,
) -> Result<SceneNode, LowerError> {
    let mut kind: Option<String> = None;
    let mut pos: Option<(f64, f64)> = None;
    let mut size: Option<(f64, f64)> = None;
    let mut material: Option<String> = None;
    let mut visible = true;
    for pr in props {
        // kind 是冻结标识符（circle|quad），先于通用取值。
        if pr.name == "kind" {
            match &pr.value {
                PropValue::One(Expr::Ident(k)) => {
                    kind = Some(k.to_string());
                    continue;
                }
                _ => return Err(unsupported("shape2d kind must be circle|quad (identifier)")),
            }
        }
        // material 允许资源 symbol 参数（`material: <Material2D param>`）。
        if pr.name == "material" {
            if let PropValue::One(e) = &pr.value {
                if let Some(r) = resource_ref_of_param(e, params) {
                    material = Some(r);
                    continue;
                }
            }
        }
        match (pr.name.as_str(), resolve_prop(pr, params)?) {
            ("position", PropVal::Tuple(v)) if v.len() == 2 => pos = Some((v[0], v[1])),
            ("size", PropVal::Tuple(v)) if v.len() == 2 => size = Some((v[0], v[1])),
            ("material", PropVal::Resource(r)) => material = Some(r),
            ("visible", PropVal::Bool(b)) => visible = b,
            (other, _) => {
                return Err(unsupported(format!(
                    "unknown shape2d prop '{other}' (basic2d: kind/position/size/material/visible)"
                )))
            }
        }
    }
    let kind = kind.ok_or_else(|| unsupported("shape2d missing kind"))?;
    if kind != "circle" && kind != "quad" {
        return Err(unsupported(format!(
            "shape2d kind '{kind}' unsupported (basic2d: circle|quad)"
        )));
    }
    let (px, py) = pos.ok_or_else(|| unsupported("shape2d missing position"))?;
    let (sx, sy) = size.ok_or_else(|| unsupported("shape2d missing size"))?;
    if kind == "circle" && sx != sy {
        return Err(unsupported(format!(
            "shape2d circle requires size.x==size.y (diameter), got ({sx},{sy})"
        )));
    }
    let m = material.ok_or_else(|| unsupported("shape2d missing material"))?;
    let rgba = resource_color(project, &m)?;
    Ok(SceneNode {
        is_group: false,
        id,
        name: name.to_string(),
        kind,
        tx: px,
        ty: py,
        sx,
        sy,
        rgba,
        visible,
        parent,
        is_character: false,
        is_solid: false,
        sprite: None,
    })
}

/// character2d → 场景角色宿主对象（盒体钉 0.8×1.0）。
fn character_node(
    project: &Project,
    name: &str,
    props: &[Prop],
    params: &[(String, ParamVal)],
    parent: Option<usize>,
    id: usize,
) -> Result<SceneNode, LowerError> {
    let mut pos: Option<(f64, f64)> = None;
    let mut material: Option<String> = None;
    for pr in props {
        if pr.name == "material" {
            if let PropValue::One(e) = &pr.value {
                if let Some(r) = resource_ref_of_param(e, params) {
                    material = Some(r);
                    continue;
                }
            }
        }
        match (pr.name.as_str(), resolve_prop(pr, params)?) {
            ("position", PropVal::Tuple(v)) if v.len() == 2 => pos = Some((v[0], v[1])),
            ("material", PropVal::Resource(r)) => material = Some(r),
            (other, _) => {
                return Err(unsupported(format!(
                    "unknown character2d prop '{other}' (basic2d: position/material; box pinned 0.8x1.0)"
                )))
            }
        }
    }
    let (px, py) = pos.ok_or_else(|| unsupported("character2d missing position"))?;
    let m = material.ok_or_else(|| unsupported("character2d missing material"))?;
    let rgba = resource_color(project, &m)?;
    Ok(SceneNode {
        is_group: false,
        id,
        name: name.to_string(),
        kind: "quad".into(),
        tx: px,
        ty: py,
        sx: 0.8,
        sy: 1.0,
        rgba,
        visible: true,
        parent,
        is_character: true,
        is_solid: false,
        sprite: None,
    })
}

/// solid2d → 可见 + 碰撞静态对象（几何来自 mesh 资源）。
fn solid_node(
    project: &Project,
    name: &str,
    props: &[Prop],
    parent: Option<usize>,
    id: usize,
) -> Result<SceneNode, LowerError> {
    let mut pos: Option<(f64, f64)> = None;
    let mut mesh: Option<String> = None;
    let mut material: Option<String> = None;
    for pr in props {
        match (pr.name.as_str(), resolve_prop(pr, &[])?) {
            ("position", PropVal::Tuple(v)) if v.len() == 2 => pos = Some((v[0], v[1])),
            ("mesh", PropVal::Resource(r)) => mesh = Some(r),
            ("material", PropVal::Resource(r)) => material = Some(r),
            (other, _) => {
                return Err(unsupported(format!(
                    "unknown solid2d prop '{other}' (basic2d: position/mesh/material)"
                )))
            }
        }
    }
    let (px, py) = pos.ok_or_else(|| unsupported("solid2d missing position"))?;
    let mesh = mesh.ok_or_else(|| unsupported("solid2d missing mesh"))?;
    let m = material.ok_or_else(|| unsupported("solid2d missing material"))?;
    let (kind, sx, sy) = resource_mesh(project, &mesh)?;
    let rgba = resource_color(project, &m)?;
    Ok(SceneNode {
        is_group: false,
        id,
        name: name.to_string(),
        kind,
        tx: px,
        ty: py,
        sx,
        sy,
        rgba,
        visible: true,
        parent,
        is_character: false,
        is_solid: true,
        sprite: None,
    })
}

/// camera2d → 相机配置（相机本身不是场景对象）。target 原样读取
/// `refs.<char>.actor`（不走通用取值）。
fn camera_cfg(props: &[Prop]) -> Result<CameraCfg, LowerError> {
    let mut target: Option<String> = None;
    let mut bounds: Option<(f64, f64, f64, f64)> = None;
    let mut view: Option<(f64, f64)> = None;
    for pr in props {
        if pr.name == "target" {
            let name = match &pr.value {
                PropValue::One(Expr::Dot(inner, field)) => {
                    if field.as_str() != "actor" {
                        return Err(unsupported("camera2d target must be refs.<char>.actor"));
                    }
                    match &**inner {
                        Expr::Dot(i2, n) => match &**i2 {
                            Expr::Ident(base) if base == "refs" => n.to_string(),
                            _ => {
                                return Err(unsupported(
                                    "camera2d target must be refs.<char>.actor",
                                ))
                            }
                        },
                        _ => return Err(unsupported("camera2d target must be refs.<char>.actor")),
                    }
                }
                _ => return Err(unsupported("camera2d target must be refs.<char>.actor")),
            };
            target = Some(name);
            continue;
        }
        match (pr.name.as_str(), resolve_prop(pr, &[])?) {
            ("bounds", PropVal::Tuple(v)) if v.len() == 4 => {
                bounds = Some((v[0], v[1], v[2], v[3]))
            }
            ("view_size", PropVal::Tuple(v)) if v.len() == 2 => view = Some((v[0], v[1])),
            (other, _) => {
                return Err(unsupported(format!(
                    "unknown camera2d prop '{other}' (basic2d: target/bounds/view_size)"
                )))
            }
        }
    }
    let target = target.ok_or_else(|| unsupported("camera2d missing target"))?;
    let (min_x, min_y, max_x, max_y) =
        bounds.ok_or_else(|| unsupported("camera2d missing bounds"))?;
    let (view_w, view_h) = view.ok_or_else(|| unsupported("camera2d missing view_size"))?;
    if min_x >= max_x || min_y >= max_y || view_w <= 0.0 || view_h <= 0.0 {
        return Err(unsupported("camera2d bounds/view_size invalid"));
    }
    Ok(CameraCfg {
        target_char_name: target,
        min_x,
        min_y,
        max_x,
        max_y,
        view_w,
        view_h,
    })
}

/// root view 元素（character/camera-2d 的角色/solid/相机叶）。
fn expand_root_scripted(
    project: &Project,
    elements: &[View2dElement],
    out: &mut Collected,
    next_id: &mut usize,
) -> Result<(), LowerError> {
    for el in elements {
        match el {
            View2dElement::Character { name, props } => {
                if out.char_name.is_some() {
                    return Err(unsupported(format!(
                        "duplicate character2d '{name}' (basic2d: exactly one)"
                    )));
                }
                let id = *next_id;
                *next_id += 1;
                out.nodes
                    .push(character_node(project, name, props, &[], None, id)?);
                out.char_name = Some(name.clone());
            }
            View2dElement::Solid { name, props } => {
                let id = *next_id;
                *next_id += 1;
                out.nodes.push(solid_node(project, name, props, None, id)?);
            }
            View2dElement::Camera { name, props } => {
                if out.camera.is_some() {
                    return Err(unsupported(format!(
                        "duplicate camera2d '{name}' (camera-2d: exactly one)"
                    )));
                }
                let _ = name;
                out.camera = Some(camera_cfg(props)?);
            }
            View2dElement::Shape { name, props } => {
                let id = *next_id;
                *next_id += 1;
                out.nodes
                    .push(shape_node(project, name, props, &[], None, id)?);
            }
            View2dElement::Group { name, .. } => {
                return Err(unsupported(format!(
                    "group2d '{name}' outside scene-2d profile (basic2d scripted: flat root)"
                )))
            }
            View2dElement::Instance { node, name, props } => {
                // 组件实例：纯视觉叶 + 恰一角色宿主（共享 Player 定义）。
                let iparams = instance_params(project, node, props)?;
                let def = node_of(project, node);
                let view = def
                    .view2d
                    .as_ref()
                    .ok_or_else(|| unsupported(format!("component '{node}' has no view-2d")))?;
                let mut sub = Collected {
                    nodes: Vec::new(),
                    camera: None,
                    char_name: None,
                };
                let mut sub_id = *next_id;
                expand_elements(
                    project,
                    &view.elements,
                    &iparams,
                    None,
                    &mut sub,
                    &mut sub_id,
                    true,
                )?;
                if let Some(cn) = sub.char_name {
                    out.char_name = Some(format!("{name}.{cn}"));
                }
                for mut n in sub.nodes {
                    n.name = format!("{name}.{}", n.name);
                    out.nodes.push(n);
                }
                *next_id = sub_id;
            }
            View2dElement::If { .. } => {
                return Err(unsupported(
                    "if in basic2d profiles (static scene: visible=false keeps identity)",
                ))
            }
            View2dElement::Trigger { name, .. } => {
                return Err(unsupported(format!(
                    "trigger2d '{name}' outside g11-2d profile"
                )))
            }
        }
    }
    Ok(())
}

/// profile 规则校验：非根节点限制与根 view 收集。
fn collect_basic(project: &Project, profile: BasicProfile) -> Result<Collected, LowerError> {
    let root = node_of(project, &project.root);
    let scripted = profile != BasicProfile::Scene2D;
    if !scripted && (root.model.is_some() || root.msg.is_some() || root.on.is_some()) {
        return Err(unsupported(
            "scene-2d root must be view-2d only (no model/msg/on: nothing runs)",
        ));
    }
    // 组件节点（非根）：参数 + 纯 view；拒绝 model/msg/on。
    for module in project.modules.values() {
        for node in &module.nodes {
            if node.name == project.root {
                continue;
            }
            if node.model.is_some() || node.msg.is_some() || node.on.is_some() {
                return Err(unsupported(format!(
                    "component node '{}' must be params + view-2d only (no model/msg/on) in {}",
                    node.name,
                    profile.name()
                )));
            }
            if let Some(view) = &node.view2d {
                for el in &view.elements {
                    match el {
                        View2dElement::Shape { .. } => {}
                        View2dElement::Character { .. } if scripted => {}
                        other => {
                            return Err(unsupported(format!(
                                "component node '{}' view must contain shape2d{}, got {}",
                                node.name,
                                if scripted { "/character2d" } else { "" },
                                element_kind(other)
                            )))
                        }
                    }
                }
            }
        }
    }
    let view = root
        .view2d
        .as_ref()
        .ok_or_else(|| unsupported("root node has no view-2d"))?;
    let mut out = Collected {
        nodes: Vec::new(),
        camera: None,
        char_name: None,
    };
    let mut next_id = 1usize;
    match profile {
        BasicProfile::Scene2D => {
            expand_elements(
                project,
                &view.elements,
                &[],
                None,
                &mut out,
                &mut next_id,
                false,
            )?;
        }
        BasicProfile::Character2D | BasicProfile::Camera2D => {
            expand_root_scripted(project, &view.elements, &mut out, &mut next_id)?;
            if out.char_name.is_none() {
                return Err(unsupported(format!(
                    "{} requires exactly one character2d in the root view",
                    profile.name()
                )));
            }
            if profile == BasicProfile::Camera2D && out.camera.is_none() {
                return Err(unsupported("camera-2d requires exactly one camera2d"));
            }
            if profile == BasicProfile::Character2D && out.camera.is_some() {
                return Err(unsupported("character-2d must not declare camera2d"));
            }
        }
    }
    // 层级服务上限：objects + groups 总和 ≤128（nodes 已含组，不再重复计）。
    if out.nodes.len() > 128 {
        return Err(unsupported("basic2d scene exceeds 128 nodes"));
    }
    if let Some(cam) = &out.camera {
        // 目标匹配：全名（组件展开为 instance.element）或尾段（root view
        // 直属 character2d / 组件实例名）。
        let char_tail = out
            .char_name
            .as_ref()
            .and_then(|n| n.rsplit('.').next().map(|s| s.to_string()));
        let matched = out
            .char_name
            .as_ref()
            .is_some_and(|n| n == &cam.target_char_name)
            || char_tail
                .as_ref()
                .is_some_and(|t| t == &cam.target_char_name);
        if !matched {
            return Err(unsupported(format!(
                "camera2d target '{}' must reference the root character2d '{}'",
                cam.target_char_name,
                out.char_name.clone().unwrap_or_default()
            )));
        }
    }
    Ok(out)
}

fn element_kind(el: &View2dElement) -> &'static str {
    match el {
        View2dElement::Solid { .. } => "solid2d",
        View2dElement::Trigger { .. } => "trigger2d",
        View2dElement::Character { .. } => "character2d",
        View2dElement::Camera { .. } => "camera2d",
        View2dElement::Shape { .. } => "shape2d",
        View2dElement::Group { .. } => "group2d",
        View2dElement::Instance { .. } => "instance",
        View2dElement::If { .. } => "if",
    }
}

// ---- 场景 JSON（schema/6；字段序与 016/017 冻结形状一致）----

pub(crate) fn fmt_f(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

pub(crate) fn obj_json(n: &SceneNode) -> String {
    // PLAN-022 sprites-2d：图像绑定行携带场景 sprite 字段（资产完整 SHA
    // 身份 + 默认整数帧矩形 + flip）；旧档恒为 null，字节不变。
    let sprite = match &n.sprite {
        None => "null".to_string(),
        Some(s) => format!(
            "{{\"asset\": \"sha256:{}\", \"frame\": [{}, {}, {}, {}], \"flip\": {}}}",
            s.asset, s.frame[0], s.frame[1], s.frame[2], s.frame[3], s.flip
        ),
    };
    format!(
        "  {{\n   \"id\": {},\n   \"name\": \"{}\",\n   \"kind\": \"{}\",\n   \"translation\": [{}, {}, 0.0],\n   \"scale\": [{}, {}, 1.0],\n   \"yaw\": 0.0,\n   \"pitch\": 0.0,\n   \"rgba\": [{}, {}, {}, {}],\n   \"visible\": {},\n   \"parent\": {},\n   \"resource\": null,\n   \"sprite\": {}\n  }}",
        n.id,
        n.name,
        n.kind,
        fmt_f(n.tx),
        fmt_f(n.ty),
        fmt_f(n.sx),
        fmt_f(n.sy),
        n.rgba.0,
        n.rgba.1,
        n.rgba.2,
        n.rgba.3,
        n.visible,
        match n.parent {
            Some(p) => p.to_string(),
            None => "null".into(),
        },
        sprite
    )
}

fn group_json(n: &SceneNode) -> String {
    format!(
        "  {{\n   \"id\": {},\n   \"name\": \"{}\",\n   \"dimension\": \"2d\",\n   \"parent\": {},\n   \"translation\": [{}, {}, 0.0],\n   \"visible\": {}\n  }}",
        n.id,
        n.name,
        match n.parent {
            Some(p) => p.to_string(),
            None => "null".into(),
        },
        fmt_f(n.tx),
        fmt_f(n.ty),
        n.visible
    )
}

fn scene_json_basic(
    entry: &str,
    profile: BasicProfile,
    collected: &Collected,
    source_sha: &str,
    source_bytes: usize,
) -> String {
    let behavior = match profile.api_version() {
        Some(api) => format!(
            " \"behavior\": {{\n  \"api\": \"{api}\",\n  \"source\": {{\n   \"format\": \"auto-source\",\n   \"sha256\": \"{source_sha}\",\n   \"byte_length\": {source_bytes}\n  }},\n  \"actors\": {{\n   \"two_d\": {},\n   \"three_d\": null\n  }}\n }},\n",
            collected.char_name.is_some().then_some(1).unwrap_or(0)
        ),
        None => " \"behavior\": null,\n".to_string(),
    };
    let objects = collected
        .nodes
        .iter()
        .filter(|n| !n.is_group)
        .map(obj_json)
        .collect::<Vec<_>>()
        .join(",\n");
    let groups = collected
        .nodes
        .iter()
        .filter(|n| n.is_group)
        .map(group_json)
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "{{\n{behavior} \"schema\": \"autographics.scene/6\",\n \"document_id\": \"plan019-basic2d-{entry}\",\n \"resources\": [\n  \"circle\",\n  \"quad\",\n  \"cylinder\",\n  \"player\",\n  \"mob\"\n ],\n \"objects\": [\n{objects}\n ],\n \"groups\": [{groups}],\n \"assets\": []\n}}\n"
    )
}

pub(crate) fn roles_json_basic(collected: &Collected) -> String {
    let mut parts = Vec::new();
    match &collected.char_name {
        Some(name) => {
            let id = collected
                .nodes
                .iter()
                .find(|n| !n.is_group && &n.name == name)
                .map(|n| n.id)
                .unwrap_or(0);
            parts.push(format!("\"character\": {id}"));
            parts.push(format!("\"character_name\": \"{name}\""));
        }
        None => parts.push("\"character\": null".to_string()),
    }
    let solids: Vec<String> = collected
        .nodes
        .iter()
        .filter(|n| n.is_solid)
        .map(|n| format!("{{\"id\": {}, \"name\": \"{}\"}}", n.id, n.name))
        .collect();
    parts.push(format!("\"solids\": [{}]", solids.join(", ")));
    parts.push(match &collected.camera {
        Some(_) => "\"camera\": 1".to_string(),
        None => "\"camera\": null".to_string(),
    });
    let objects: Vec<String> = collected
        .nodes
        .iter()
        .filter(|n| !n.is_group)
        .map(|n| format!("\"{}\": {}", n.name, n.id))
        .collect();
    parts.push(format!("\"scene_objects\": {{{}}}", objects.join(", ")));
    format!("{{\n {}\n}}", parts.join(",\n "))
}

// ---- 用户源生成（character-2d/camera-2d）----

use crate::load::node_of as node_of_pub;
use crate::lower::slice_fn_source;
use crate::scene::sha256_hex;
use crate::translate::{translate_stmts, Ctx, Target};
use auto_ui_dialect::node2d::EventPattern;

fn enum_lookup(project: &Project, ename: &str, variant: &str) -> Option<String> {
    for module in project.modules.values() {
        if let Some(variants) = module.enums.get(ename) {
            if variants.contains_key(variant) {
                return Some(format!(
                    "{}_{}",
                    ename.to_uppercase(),
                    variant.to_uppercase()
                ));
            }
        }
    }
    None
}

pub(crate) fn life_handler<'a>(
    node: &'a NodeDecl,
    event: &str,
) -> Option<&'a auto_ui_dialect::node2d::Handler> {
    let on = node.on.as_ref()?;
    on.handlers.iter().find(|h| {
        matches!(
            &h.pattern,
            EventPattern::Life { event: e, .. } if e == event
        )
    })
}

/// 生成用户源：根 model 字段（`r_<field>` 标量存储，translate Root 约定）
/// + 枚举常量 + 作者 fn 原文 + 生成的 app_start 绑定/app_update tick。
fn gen_user_source(
    project: &Project,
    profile: BasicProfile,
    collected: &Collected,
) -> Result<String, LowerError> {
    let root_name = &project.root;
    let root = node_of_pub(project, root_name);
    let mut src = String::new();
    src.push_str(&format!(
        "// GENERATED by autographics-node2d-pilot (PLAN-019 {} profile, T-02 contract).\n",
        profile.name()
    ));
    src.push_str(&format!(
        "// Root: {root_name}; character: {}; solids: {}; camera: {}\n",
        collected.char_name.clone().unwrap_or_default(),
        collected
            .nodes
            .iter()
            .filter(|n| n.is_solid)
            .map(|n| n.name.as_str())
            .collect::<Vec<_>>()
            .join(","),
        collected
            .camera
            .as_ref()
            .map(|_| "follow-2d")
            .unwrap_or("-")
    ));
    src.push_str("// Regeneration must be byte-identical (tools/verify_plan019.py).\n");

    // ---- 枚举常量 ----
    for module in project.modules.values() {
        for (ename, variants) in &module.enums {
            for (vname, value) in variants {
                src.push_str(&format!(
                    "var {}_{}={value}\n",
                    ename.to_uppercase(),
                    vname.to_uppercase()
                ));
            }
        }
    }

    // ---- 根标量状态（`r_<field>`；int/float 字面量初值）----
    if let Some(model) = &root.model {
        for f in &model.fields {
            let lit = match &f.init {
                Expr::Int(v) => format!("{v}"),
                Expr::Float(_, raw) => raw.to_string(),
                other => {
                    return Err(unsupported(format!(
                        "root model init must be a numeric literal, got {other:?}"
                    )))
                }
            };
            src.push_str(&format!("var r_{}={lit}\n", f.name));
        }
    }

    // ---- 作者纯函数（全部模块，按名字序；原文降层）----
    let mut fn_names: Vec<(String, String)> = Vec::new();
    for module in project.modules.values() {
        for (fname, _) in &module.fns {
            fn_names.push((module.name.clone(), fname.clone()));
        }
    }
    fn_names.sort();
    for (module_name, fname) in fn_names {
        let module = &project.modules[&module_name];
        let text = slice_fn_source(&module.source, &fname)
            .ok_or_else(|| unsupported(format!("fn {fname} source slice")))?;
        src.push_str(&format!("// pooled fn {fname} from {module_name}\n"));
        src.push_str(text.trim_end());
        src.push('\n');
    }

    // ---- 处理器翻译上下文（共享 translate；basic2d 无 emit/实例成员）----
    let ctx = Ctx {
        target: Target::Root,
        ahud3: None,
        emit_hook: Box::new(|event: &str| {
            Err(unsupported(format!(
                "emit .{event} without binding (basic2d: no msg events)"
            )))
        }),
        enum_path: Box::new(|ename: &str, variant: &str| enum_lookup(project, ename, variant)),
        axis_differential: true,
        member_hook: None,
        moved_flag: None,
        refs_any_elem: true,
        // PLAN-020 钩子：basic2d 档不用（019 字节不变）。
        call_scope: None,
        child_emit: None,
        query: None,
        sprites: None,
        space3: None,
                        sfx: None,
                        handle_ref: None,
                        six_actions: false,
                        char3: None,
                        cam3: false,
            events3d: None,
            bindings3d: None,
        vis3d: None,
    };

    // ---- app_start：生成绑定（声明序）+ 作者 Life.Start ----
    let char_id = collected
        .nodes
        .iter()
        .find(|n| n.is_character)
        .map(|n| n.id);
    let solid_ids: Vec<usize> = collected
        .nodes
        .iter()
        .filter(|n| n.is_solid)
        .map(|n| n.id)
        .collect();
    src.push_str("fn app_start() {\n");
    if let Some(id) = char_id {
        src.push_str(&format!(" ag_char_bind({id})\n"));
        for sid in &solid_ids {
            src.push_str(&format!(" ag_char_solid({sid})\n"));
        }
        if let Some(cam) = &collected.camera {
            src.push_str(&format!(
                " ag_camera_follow_2d({id}, {}, {}, {}, {}, {}, {})\n",
                crate::lower_basic2d::fmt_f(cam.min_x),
                crate::lower_basic2d::fmt_f(cam.min_y),
                crate::lower_basic2d::fmt_f(cam.max_x),
                crate::lower_basic2d::fmt_f(cam.max_y),
                crate::lower_basic2d::fmt_f(cam.view_w),
                crate::lower_basic2d::fmt_f(cam.view_h),
            ));
        }
    }
    if let Some(h) = life_handler(root, "Start") {
        translate_stmts(&h.body.stmts, &ctx, &mut src, 1)?;
    }
    src.push_str("}\n");

    // ---- app_update：作者 Life.Tick（物理/输入全在作者 Auto）----
    src.push_str("fn app_update(dt float) {\n");
    if let Some(h) = life_handler(root, "Tick") {
        translate_stmts(&h.body.stmts, &ctx, &mut src, 1)?;
    }
    src.push_str("}\n");
    Ok(src)
}

/// PLAN-019 基础二维公共入口：校验 → 场景/用户源 → identity 材料。
pub fn lower_project_basic(
    project: &Project,
    profile: BasicProfile,
) -> Result<LowerOutputBasic2D, LowerError> {
    let collected = collect_basic(project, profile)?;
    let entry = project
        .node_modules
        .get(&project.root)
        .cloned()
        .unwrap_or_else(|| "main".into());
    let (user_source, source_sha, source_bytes) = match profile {
        BasicProfile::Scene2D => (None, sha256_hex(b""), 0usize),
        BasicProfile::Character2D | BasicProfile::Camera2D => {
            let src = gen_user_source(project, profile, &collected)?;
            let bytes = src.as_bytes();
            (Some(src.clone()), sha256_hex(bytes), bytes.len())
        }
    };
    let scene = scene_json_basic(&entry, profile, &collected, &source_sha, source_bytes);
    let roles = roles_json_basic(&collected);
    Ok(LowerOutputBasic2D {
        user_source,
        scene_json: scene,
        source_sha256: source_sha,
        source_bytes,
        roles_json: roles,
    })
}
