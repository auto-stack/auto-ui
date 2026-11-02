//! PLAN-024 components-3d：view-3d 三维作者入口适配模块。
//!
//! 复用 020 的共享组件 IR（DefIR/InstanceIR/WorldIR/Plan/plan_dispatch/
//! instance_ctx/root_ctx）与生成机制（SoA、绑定表、池化纯函数、getter、
//! 冻结 app_start/app_update 序），只替换维度面：场景收集（cylinder3d/
//! model3d/group3d → Document 三维 objects/groups）与呈现同步（v1
//! `ag_set_local`/`ag_set_visible` 直写文档节点，行句柄 = driver
//! `hierarchy::ordered()` 先序 + 1）。不复制 parser/词法/表达式/VM。
//!
//! T-02 冻结合同（本模块即其落地）：
//! - 唯一装配：behavior=null（纯场景）或 `autographics.behavior/1`；
//!   不复制 objects3d、不强迫角色/相机。
//! - 图元/静态模型只映射现有 Document 能力：cylinder3d→kind "cylinder"、
//!   model3d→kind "model"+resource；group3d→3d 组（仅平移/可见）。
//! - 动态面仅叶 position 三轴与 visible（`.field` 绑定）；scale/yaw/pitch/
//!   材质/模型绑定 Run 内不可变；group3d 平移/可见静态（叶绑 model）。
//! - 场景 parent 只由显式 group3d 决定；同维 f32 逐层平移与祖先可见 AND
//!   由 driver derive 承担（本模块只发声明）。
//! - v1 无 32 计费池：空间写按 VM 原额（1000 steps/slice、100000 steps/
//!   120 resumes/250ms 每回调）+ 完整 tick 记账；呈现写按变更计费门
//!   （平方差/int != 缓存，与 020 同型）。

use std::collections::{BTreeMap, BTreeSet};

use auto_parser_api::ast::Expr;
use auto_ui_dialect::node2d::{EventPattern, Prop, PropValue, View3dElement};

use crate::load::{node_of, Project, ResourceKind};
use crate::lower::numeric_lit;
use crate::lower_basic2d::{fmt_f, life_handler, LowerOutputBasic2D};
use crate::lower_components::{
    at, decl_line, def_order, instance_ctx, line_at, model_field_is_int, model_init_value,
    prop_line, prop_of, ptype_of_3d, rename_module_calls, root_ctx, walk_stmts, CVal, DefIR,
    InstanceIR, PType, Plan, WalkOut, WorldIR,
};
use crate::translate::{translate_handler, LowerError, SpaceEnv};

const API: &str = "autographics.behavior/1";

// ------------------------------------------------------------------ 3D IR --

/// view-3d 内一个三维叶（cylinder3d/model3d/player3d/mob3d）的静态面。
#[derive(Debug, Clone)]
pub(crate) struct ElemIR3 {
    pub(crate) elem: String,
    /// "cylinder" | "model" | "player" | "mob"（Document kind 拼写）。
    pub(crate) kind: &'static str,
    /// model 引用（仅 model3d）。
    pub(crate) model: Option<ModelRef>,
    /// position 三元组（数值/参数/`.field` 引用）。
    pub(crate) position: Vec<CVal>,
    /// 静态 scale（数值/参数；缺省 [1,1,1]；内置视觉叶固定 [1,1,1]）。
    pub(crate) scale: Vec<CVal>,
    pub(crate) yaw: CVal,
    pub(crate) pitch: CVal,
    /// visible（静态数值/参数或 `.field` 引用）。
    pub(crate) visible: CVal,
    pub(crate) dyn_pos: bool,
    pub(crate) dyn_vis: bool,
    /// PLAN-027：`solid: true` —— 本叶标记为角色 solid（cylinder3d 限定，
    /// 全静态；bind/solid/camera 配置调用由 runtime 降层注入 Life.Start）。
    pub(crate) solid: bool,
    /// 所在 group 链（外→内，组名）。
    pub(crate) groups: Vec<String>,
    pub(crate) line: usize,
}

/// 叶元素模型引用：`resources.<name>` 直连或 Model3D 参数。
#[derive(Debug, Clone)]
pub(crate) enum ModelRef {
    Direct(String),
    Param(String),
}

/// view-3d 内一个显式 group3d 的静态面（平移/可见静态，叶才绑 model）。
#[derive(Debug, Clone)]
pub(crate) struct GroupIR3 {
    pub(crate) name: String,
    /// 所在上级 group 链（外→内）。
    pub(crate) parent_chain: Vec<String>,
    pub(crate) position: Vec<CVal>,
    pub(crate) visible: bool,
    #[allow(dead_code)] // 诊断保留（行号随行进 identity/夹具定位）。
    pub(crate) line: usize,
}

/// 一个定义的 view-3d 静态面（按定义名索引）。
#[derive(Debug, Default)]
pub(crate) struct View3dIR {
    pub(crate) elems: Vec<ElemIR3>,
    pub(crate) groups: Vec<GroupIR3>,
}

/// 3D 世界 IR：共享 WorldIR + 每定义 view-3d 面。
pub(crate) struct World3d {
    pub(crate) world: WorldIR,
    pub(crate) views: BTreeMap<String, View3dIR>,
}

/// 视图静态解析上下文（DefIR 构造前的轻量面）。
pub(crate) struct ViewCtx3<'a> {
    pub(crate) module: &'a str,
    pub(crate) def_line: usize,
    pub(crate) params: &'a [(String, PType)],
    pub(crate) model: &'a [(String, bool)],
}

impl<'a> ViewCtx3<'a> {
    pub(crate) fn is_param(&self, name: &str) -> bool {
        self.params.iter().any(|(n, _)| n == name)
    }
    pub(crate) fn is_field(&self, name: &str) -> bool {
        self.model.iter().any(|(n, _)| n == name)
    }
}

// -------------------------------------------------------------- def build --

pub(crate) fn build_def_3d(
    project: &Project,
    name: &str,
    is_root: bool,
) -> Result<(DefIR, View3dIR), LowerError> {
    build_def_3d_with(project, name, is_root, false, false)
}

/// PLAN-028：`builtin_transform` 只在 components-3d-events 档为真（内置
/// mob3d/player3d 叶允许静态 scale/yaw/pitch = 模板身份）；其余档保持
/// 024 冻结拒绝面，旧字节不变。
/// PLAN-030：`refs` 只在 components-3d-bindings 档为真（Slot/Template/
/// Pair/Actor/msg 名义引用参数；无 view-3d 的纯逻辑 Node）。其余档保持
/// 旧拒绝面，旧字节不变。
pub(crate) fn build_def_3d_with(
    project: &Project,
    name: &str,
    is_root: bool,
    builtin_transform: bool,
    refs: bool,
) -> Result<(DefIR, View3dIR), LowerError> {
    let module = project.node_modules.get(name).cloned().ok_or_else(|| {
        at(
            "",
            0,
            name,
            &format!("instance references unknown node '{name}' (missing use?)"),
        )
    })?;
    let decl = node_of(project, name);
    let line = decl_line(project, &module, decl);
    let module_src = &project.modules[&module].source;

    let mut params = Vec::new();
    for (pname, ty) in &decl.params {
        // PLAN-030：refs 档接受名义引用参数（ABI 载体 int；名义类别由
        // 装配按 decl 类型核，数值载体不进 PType 以免旧档类型面扩大）。
        let pt = match ptype_of_3d(ty) {
            Some(pt) => pt,
            None if refs && crate::node3d_binding_refs::ref_kind_of(ty).is_some() => PType::Int,
            None => {
                return Err(at(
                    &module,
                    line,
                    pname,
                    "param has unsupported type (components-3d: int/float/Model3D)",
                ))
            }
        };
        params.push((pname.clone(), pt));
    }
    if is_root && !params.is_empty() {
        return Err(at(
            &module,
            line,
            name,
            "root node must not declare params (root model init is a numeric literal)",
        ));
    }

    let mut model = Vec::new();
    if let Some(m) = &decl.model {
        for f in &m.fields {
            let is_int = model_field_is_int(project, &module, line, &params, f)?;
            model.push((f.name.to_string(), is_int));
        }
    }

    let mut msgs = BTreeMap::new();
    if let Some(m) = &decl.msg {
        for v in &m.variants {
            let mut pts = Vec::new();
            for t in &v.payload {
                let pt = ptype_of_3d(t).ok_or_else(|| {
                    at(
                        &module,
                        line,
                        &v.name.to_string(),
                        "msg payload type unsupported (components-3d: int/float)",
                    )
                })?;
                if pt != PType::Int && pt != PType::Float {
                    return Err(at(
                        &module,
                        line,
                        &v.name.to_string(),
                        "msg payload must be numeric (components-3d)",
                    ));
                }
                pts.push(pt);
            }
            let lower = v.name.to_string().to_lowercase();
            if matches!(lower.as_str(), "lifestart" | "lifetick") {
                return Err(at(
                    &module,
                    line,
                    &v.name.to_string(),
                    "business event name collides with the reserved Life handler names",
                ));
            }
            msgs.insert(v.name.to_string(), (v.payload.len(), pts));
        }
    }

    let mut members = BTreeMap::new();
    let mut life_start = false;
    let mut life_tick = false;
    if let Some(on) = &decl.on {
        for h in &on.handlers {
            let hline = line_at(module_src, h.span.0);
            match &h.pattern {
                EventPattern::Life { event, .. } => match event.as_str() {
                    "Start" => life_start = true,
                    "Tick" => life_tick = true,
                    other => {
                        return Err(at(
                            &module,
                            hline,
                            &format!("Life.{other}"),
                            "unsupported Life event (components-3d: Life.Start/Life.Tick)",
                        ))
                    }
                },
                EventPattern::Member { name, params } => {
                    if !msgs.contains_key(name.as_str()) {
                        return Err(at(
                            &module,
                            hline,
                            name,
                            &format!("handler '.{name}' has no declared msg variant"),
                        ));
                    }
                    let (payload, _) = msgs[name.as_str()];
                    if params.len() != payload {
                        return Err(at(
                            &module,
                            hline,
                            name,
                            &format!(
                                "handler '.{name}' declares {} param(s) but the msg payload carries {payload}",
                                params.len()
                            ),
                        ));
                    }
                    let lower = name.to_lowercase();
                    if matches!(lower.as_str(), "lifestart" | "lifetick") {
                        return Err(at(
                            &module,
                            hline,
                            name,
                            "business handler name collides with the reserved Life handler names",
                        ));
                    }
                    members.insert(name.clone(), (params.len(), hline));
                }
            }
        }
    }

    // view-3d 静态面（DefIR 构造前解析；dyn/static 列表由叶动态面决定）。
    let vctx = ViewCtx3 {
        module: &module,
        def_line: line,
        params: &params,
        model: &model,
    };
    let empty_view;
    let view = match decl.view3d.as_ref() {
        Some(v) => v,
        // PLAN-027：携带 character3d/camera3d 的定义可无 view-3d（如仅
        // 相机配置的运行 root）；其余定义保持旧具名拒绝。
        None if decl.character3d.is_some() || decl.camera3d.is_some() => {
            empty_view = auto_ui_dialect::node2d::View3dBlock {
                elements: Vec::new(),
                span: decl.span,
            };
            &empty_view
        }
        // PLAN-030：bindings 档纯逻辑 Node 可无 view-3d（局部状态/msg/on
        // 组件不要求可见 leaf；§5.3）。
        None if refs => {
            empty_view = auto_ui_dialect::node2d::View3dBlock {
                elements: Vec::new(),
                span: decl.span,
            };
            &empty_view
        }
        None => {
            return Err(at(&module, line, name, "component definition has no view-3d"));
        }
    };
    let mut v3 = View3dIR::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    walk_view3d_elements(
        project,
        &vctx,
        &view.elements,
        Vec::new(),
        &mut seen,
        &mut v3,
        builtin_transform,
    )?;

    let mut dyn_elems = Vec::new();
    let mut static_elems = Vec::new();
    for e in &v3.elems {
        if e.dyn_pos || e.dyn_vis {
            dyn_elems.push(e.elem.clone());
        } else {
            static_elems.push(e.elem.clone());
        }
    }

    let def = DefIR {
        name: name.to_string(),
        module,
        line,
        params,
        model,
        msgs,
        members,
        life_start,
        life_tick,
        dyn_elems,
        static_elems,
        char_elem: None,
        sprite_elems: Vec::new(),
        stateful: decl.model.is_some() || decl.msg.is_some() || decl.on.is_some(),
    };
    Ok((def, v3))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn walk_view3d_elements(
    project: &Project,
    vctx: &ViewCtx3,
    elements: &[View3dElement],
    groups: Vec<String>,
    seen: &mut BTreeSet<String>,
    out: &mut View3dIR,
    builtin_transform: bool,
) -> Result<(), LowerError> {
    let module_src = &project.modules[vctx.module].source;
    for el in elements {
        match el {
            View3dElement::Primitive { name, props }
            | View3dElement::Model { name, props }
            | View3dElement::Player { name, props }
            | View3dElement::Mob { name, props } => {
                if !seen.insert(name.clone()) {
                    return Err(at(
                        vctx.module,
                        vctx.def_line,
                        name,
                        "duplicate view-3d element name",
                    ));
                }
                let (is_model, is_builtin) = match el {
                    View3dElement::Model { .. } => (true, false),
                    View3dElement::Player { .. } | View3dElement::Mob { .. } => (false, true),
                    _ => (false, false),
                };
                let kind: &'static str = match el {
                    View3dElement::Primitive { .. } => "cylinder",
                    View3dElement::Model { .. } => "model",
                    View3dElement::Player { .. } => "player",
                    View3dElement::Mob { .. } => "mob",
                    _ => unreachable!("leaf arm"),
                };
                let kind_label = match el {
                    View3dElement::Primitive { .. } => "cylinder3d",
                    View3dElement::Model { .. } => "model3d",
                    View3dElement::Player { .. } => "player3d",
                    _ => "mob3d",
                };
                let line = props
                    .first()
                    .map(|p| prop_line(module_src, p))
                    .unwrap_or(vctx.def_line);
                let mut model = None;
                if is_model {
                    let m = prop_of(props, "model").ok_or_else(|| {
                        at(
                            vctx.module,
                            line,
                            name,
                            "model3d element missing 'model' prop",
                        )
                    })?;
                    model = Some(match &m.value {
                        PropValue::One(Expr::Dot(outer, r))
                            if matches!(&**outer, Expr::Ident(b) if b == "resources") =>
                        {
                            ModelRef::Direct(r.to_string())
                        }
                        PropValue::One(Expr::Dot(outer, p))
                            if matches!(&**outer, Expr::Ident(b) if b == "self") =>
                        {
                            if !vctx.is_param(p) {
                                return Err(at(
                                    vctx.module,
                                    line,
                                    name,
                                    &format!("model param binding references unknown param '{p}'"),
                                ));
                            }
                            ModelRef::Param(p.to_string())
                        }
                        PropValue::One(Expr::Ident(p)) => {
                            if !vctx.is_param(p) {
                                return Err(at(
                                    vctx.module,
                                    line,
                                    name,
                                    &format!("model param binding references unknown param '{p}'"),
                                ));
                            }
                            ModelRef::Param(p.to_string())
                        }
                        other => {
                            return Err(at(
                                vctx.module,
                                line,
                                name,
                                &format!(
                                    "model prop must be resources.<name> or a Model3D param, got {other:?}"
                                ),
                            ))
                        }
                    });
                } else if prop_of(props, "model").is_some() {
                    return Err(at(
                        vctx.module,
                        line,
                        name,
                        &format!("{kind_label} does not take a 'model' prop (use model3d)"),
                    ));
                }
                // PLAN-027：solid 标记（v1 限 cylinder3d 叶；全静态 AABB）。
                let solid = match prop_of(props, "solid") {
                    None => false,
                    Some(pr) => {
                        if is_model || is_builtin {
                            return Err(at(
                                vctx.module,
                                line,
                                name,
                                &format!("'solid' is a cylinder3d prop, not {kind_label}"),
                            ));
                        }
                        let es = prop_exprs_3d(pr);
                        match es.as_slice() {
                            [Expr::Bool(true)] | [Expr::Int(1)] => true,
                            [Expr::Bool(false)] | [Expr::Int(0)] => {
                                return Err(at(
                                    vctx.module,
                                    line,
                                    "solid",
                                    "'solid' must be true (omit the prop for a non-solid leaf)",
                                ));
                            }
                            other => {
                                return Err(at(
                                    vctx.module,
                                    line,
                                    "solid",
                                    &format!("'solid' must be the literal true, got {other:?}"),
                                ));
                            }
                        }
                    }
                };
                let pos_pr = prop_of(props, "position").ok_or_else(|| {
                    at(
                        vctx.module,
                        line,
                        name,
                        "view-3d element missing 'position'",
                    )
                })?;
                let es = prop_exprs_3d(pos_pr);
                if es.len() != 3 {
                    return Err(at(
                        vctx.module,
                        line,
                        "position",
                        "'position' must have 3 components (x, y, z)",
                    ));
                }
                let position = es
                    .iter()
                    .map(|e| resolve_cval_3d(vctx, e))
                    .collect::<Result<Vec<_>, _>>()?;
                let dyn_pos = es.iter().any(|e| expr_binds_field(e, vctx));
                let scale = match prop_of(props, "scale") {
                    None => vec![CVal::Num(1.0), CVal::Num(1.0), CVal::Num(1.0)],
                    Some(pr) => {
                        if is_builtin && !builtin_transform {
                            return Err(at(
                                vctx.module,
                                line,
                                name,
                                &format!(
                                    "{kind_label} has intrinsic built-in geometry and takes no 'scale' prop"
                                ),
                            ));
                        }
                        let es = prop_exprs_3d(pr);
                        if es.len() != 3 {
                            return Err(at(
                                vctx.module,
                                line,
                                "scale",
                                "'scale' must have 3 components (sx, sy, sz)",
                            ));
                        }
                        for e in &es {
                            if expr_binds_field(e, vctx) {
                                return Err(at(
                                    vctx.module,
                                    line,
                                    "scale",
                                    "scale must be static (Run-time scale is out of the PLAN-024 scope)",
                                ));
                            }
                        }
                        es.iter()
                            .map(|e| resolve_cval_3d(vctx, e))
                            .collect::<Result<Vec<_>, _>>()?
                    }
                };
                let (yaw, pitch) = {
                    let mut vals: Option<CVal> = None;
                    let mut pv: Option<CVal> = None;
                    for (label, slot) in [("yaw", 0usize), ("pitch", 1usize)] {
                        if let Some(pr) = prop_of(props, label) {
                            if is_builtin && !builtin_transform {
                                return Err(at(
                                    vctx.module,
                                    line,
                                    name,
                                    &format!(
                                        "{kind_label} has intrinsic built-in geometry and takes no '{label}' prop"
                                    ),
                                ));
                            }
                            let es = prop_exprs_3d(pr);
                            if es.len() != 1 {
                                return Err(at(
                                    vctx.module,
                                    line,
                                    label,
                                    &format!("'{label}' must be a single value"),
                                ));
                            }
                            if expr_binds_field(&es[0], vctx) {
                                return Err(at(
                                    vctx.module,
                                    line,
                                    label,
                                    &format!(
                                        "{label} must be static (Run-time rotation is out of the PLAN-024 scope)"
                                    ),
                                ));
                            }
                            let v = resolve_cval_3d(vctx, &es[0])?;
                            if slot == 0 {
                                vals = Some(v);
                            } else {
                                pv = Some(v);
                            }
                        }
                    }
                    (vals.unwrap_or(CVal::Num(0.0)), pv.unwrap_or(CVal::Num(0.0)))
                };
                let visible = match prop_of(props, "visible") {
                    None => CVal::Num(1.0),
                    Some(pr) => {
                        let es = prop_exprs_3d(pr);
                        if es.len() != 1 {
                            return Err(at(
                                vctx.module,
                                line,
                                "visible",
                                "'visible' must be a single value",
                            ));
                        }
                        resolve_cval_3d(vctx, &es[0])?
                    }
                };
                let dyn_vis = prop_of(props, "visible")
                    .map(|pr| prop_exprs_3d(pr).iter().any(|e| expr_binds_field(e, vctx)))
                    .unwrap_or(false);
                for pr in props {
                    let known: &[&str] = if is_builtin {
                        // PLAN-028：events 档内置叶接受静态 scale/yaw/pitch
                        // （模板身份）；solid 仍由专用检查具名拒绝。
                        if builtin_transform {
                            &["position", "scale", "yaw", "pitch", "visible"]
                        } else {
                            &["position", "visible", "solid"]
                        }
                    } else if is_model {
                        &["position", "scale", "yaw", "pitch", "visible", "model"]
                    } else {
                        &["position", "scale", "yaw", "pitch", "visible", "solid"]
                    };
                    if !known.contains(&pr.name.as_str()) {
                        return Err(at(
                            vctx.module,
                            line,
                            &pr.name,
                            &format!(
                                "unknown prop on view-3d element '{name}' (known: {})",
                                known.join("/")
                            ),
                        ));
                    }
                }
                out.elems.push(ElemIR3 {
                    elem: name.clone(),
                    kind,
                    model,
                    position,
                    scale,
                    yaw,
                    pitch,
                    visible,
                    dyn_pos,
                    dyn_vis,
                    solid,
                    groups: groups.clone(),
                    line,
                });
            }
            View3dElement::Group {
                name,
                props,
                children,
            } => {
                if !seen.insert(name.clone()) {
                    return Err(at(
                        vctx.module,
                        vctx.def_line,
                        name,
                        "duplicate view-3d element name",
                    ));
                }
                let line = props
                    .first()
                    .map(|p| prop_line(module_src, p))
                    .unwrap_or(vctx.def_line);
                let pos_pr = prop_of(props, "position")
                    .ok_or_else(|| at(vctx.module, line, name, "group3d missing 'position'"))?;
                let es = prop_exprs_3d(pos_pr);
                if es.len() != 3 {
                    return Err(at(
                        vctx.module,
                        line,
                        "position",
                        "'position' must have 3 components (x, y, z)",
                    ));
                }
                for e in &es {
                    if expr_binds_field(e, vctx) {
                        return Err(at(
                            vctx.module,
                            line,
                            name,
                            "group3d translation is static (leaf elements bind model fields)",
                        ));
                    }
                }
                let position = es
                    .iter()
                    .map(|e| resolve_cval_3d(vctx, e))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut visible = true;
                if let Some(pr) = prop_of(props, "visible") {
                    let es = prop_exprs_3d(pr);
                    if es.len() != 1 {
                        return Err(at(
                            vctx.module,
                            line,
                            "visible",
                            "'visible' must be a single value",
                        ));
                    }
                    match resolve_cval_3d(vctx, &es[0])? {
                        CVal::Num(v) => visible = v != 0.0,
                        CVal::Ref(_) => return Err(at(
                            vctx.module,
                            line,
                            name,
                            "group3d visible must be a literal (leaf elements bind model fields)",
                        )),
                    }
                }
                for pr in props {
                    if !matches!(pr.name.as_str(), "position" | "visible") {
                        return Err(at(
                            vctx.module,
                            line,
                            &pr.name,
                            "unknown prop on group3d (known: position/visible)",
                        ));
                    }
                }
                out.groups.push(GroupIR3 {
                    name: name.clone(),
                    parent_chain: groups.clone(),
                    position,
                    visible,
                    line,
                });
                let mut child_chain = groups.clone();
                child_chain.push(name.clone());
                walk_view3d_elements(project, vctx, children, child_chain, seen, out, builtin_transform)?;
            }
            View3dElement::Instance { .. } => {}
        }
    }
    Ok(())
}

/// prop 值按序取分量（Pair / Tuple / 单值）。
fn prop_exprs_3d(pr: &Prop) -> Vec<Expr> {
    match &pr.value {
        PropValue::Pair(es) => es.clone(),
        PropValue::One(Expr::Tuple(es)) => es.clone(),
        PropValue::One(e) => vec![e.clone()],
    }
}

/// 视图表达式 → CVal（数值/bool 字面量、参数、`.field`）。
fn resolve_cval_3d(vctx: &ViewCtx3, e: &Expr) -> Result<CVal, LowerError> {
    if let Some(v) = numeric_lit(e) {
        return Ok(CVal::Num(v));
    }
    if let Expr::Bool(b) = e {
        return Ok(CVal::Num(if *b { 1.0 } else { 0.0 }));
    }
    if let Expr::Ident(p) = e {
        if vctx.is_param(p) {
            // 参数引用以控制前缀区分（\x01 不会出现在源文本里）。
            return Ok(CVal::Ref(format!("\x01{p}")));
        }
    }
    if let Expr::Dot(inner, p) = e {
        if matches!(&**inner, Expr::Ident(b) if b == "self") {
            if vctx.is_param(p) {
                return Ok(CVal::Ref(format!("\x01{p}")));
            }
            if vctx.is_field(p) {
                return Ok(CVal::Ref(p.to_string()));
            }
        }
    }
    Err(at(
        vctx.module,
        vctx.def_line,
        &format!("{e:?}"),
        "view expression must be a numeric/bool literal, param or '.field' model ref",
    ))
}

/// 表达式是否绑定本定义 model 字段（`.field`）。
fn expr_binds_field(e: &Expr, vctx: &ViewCtx3) -> bool {
    fn scan(e: &Expr, fields: &[String]) -> bool {
        match e {
            Expr::Dot(inner, name) => {
                matches!(&**inner, Expr::Ident(b) if b == "self")
                    && fields.contains(&name.to_string())
            }
            Expr::Bina(l, _, r) => scan(l, fields) || scan(r, fields),
            Expr::Tuple(items) => items.iter().any(|i| scan(i, fields)),
            _ => false,
        }
    }
    let fields: Vec<String> = vctx.model.iter().map(|(n, _)| n.clone()).collect();
    scan(e, &fields)
}

// ------------------------------------------------------------- world build --

pub(crate) fn build_world_3d(project: &Project) -> Result<World3d, LowerError> {
    build_world_3d_with(project, false, false)
}

/// PLAN-028：`builtin_transform` 只在 components-3d-events 档为真。
pub(crate) fn build_world_3d_with(
    project: &Project,
    builtin_transform: bool,
    refs: bool,
) -> Result<World3d, LowerError> {
    let root_name = project.root.clone();
    let root_module = project
        .node_modules
        .get(&root_name)
        .cloned()
        .ok_or_else(|| at("", 0, &root_name, "root node module missing"))?;
    let (root_def, root_view) =
        build_def_3d_with(project, &root_name, true, builtin_transform, refs)?;

    let mut world = WorldIR {
        root: root_name.clone(),
        root_module: root_module.clone(),
        defs: BTreeMap::new(),
        instances: Vec::new(),
        // 有状态实例总数（root 含 model/msg/on 面才计入；纯场景为 0）。
        stateful: usize::from(root_def.stateful),
        solids: Vec::new(),
        camera: None,
        images: BTreeMap::new(),
        frames: BTreeMap::new(),
        models: BTreeMap::new(),
    };
    world.defs.insert(root_name.clone(), root_def);

    // GLB 模型资源（声明序跨 resources 模块；名字唯一）。
    for module in project.modules.values() {
        for r in &module.resources {
            if let ResourceKind::Model3D {
                sha256,
                byte_length,
                ..
            } = &r.kind
            {
                if world.models.contains_key(&r.name) {
                    return Err(at(
                        &module.name,
                        0,
                        &r.name,
                        "duplicate model resource name",
                    ));
                }
                world
                    .models
                    .insert(r.name.clone(), (sha256.clone(), *byte_length));
            }
        }
    }

    let mut views = BTreeMap::new();
    views.insert(root_name.clone(), root_view);

    // 实例展开（020 同型：路径唯一/环/深度/参数绑定/8 包络），外加
    // caller group 上下文记录（parent 只由显式 group3d 决定）。
    fn expand_3d(
        project: &Project,
        w: &mut WorldIR,
        views: &mut BTreeMap<String, View3dIR>,
        node_name: &str,
        inst_name: &str,
        path: &str,
        parent: Option<usize>,
        parent_group: Option<String>,
        props: &[Prop],
        line: usize,
        in_module: &str,
        stack: &mut Vec<(String, String)>,
        depth: usize,
        builtin_transform: bool,
        refs: bool,
    ) -> Result<(), LowerError> {
        if depth > 16 {
            return Err(at(
                in_module,
                line,
                inst_name,
                "component nesting deeper than 16",
            ));
        }
        if w.instances.iter().any(|i| i.path == path) {
            return Err(at(
                in_module,
                line,
                inst_name,
                &format!("duplicate instance path '{path}'"),
            ));
        }
        if !project.node_modules.contains_key(node_name) {
            return Err(at(
                in_module,
                line,
                node_name,
                &format!(
                    "instance '{inst_name}' references unknown node '{node_name}' (missing use?)"
                ),
            ));
        }
        if stack.iter().any(|(d, _)| d == node_name) {
            let chain = stack
                .iter()
                .map(|(d, _)| d.as_str())
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(at(
                in_module,
                line,
                inst_name,
                &format!("instance cycle: {chain} -> {node_name}"),
            ));
        }
        if !w.defs.contains_key(node_name) {
            let (def, view) =
                build_def_3d_with(project, node_name, false, builtin_transform, refs)?;
            views.insert(node_name.to_string(), view);
            w.defs.insert(node_name.to_string(), def);
        }
        let def_module = w.defs[node_name].module.clone();
        let decl = node_of(project, node_name);

        use crate::lower_components::ParamVal;
        let parent_params: Vec<(String, ParamVal)> = parent
            .and_then(|p| w.instances.get(p).map(|i| i.params.clone()))
            .unwrap_or_default();
        let mut params: Vec<(String, ParamVal)> = Vec::new();
        let def_ptypes: Vec<(String, PType)> = w.defs[node_name]
            .params
            .iter()
            .map(|(n, t)| (n.clone(), *t))
            .collect();
        for (pname, pty) in &def_ptypes {
            let pr = props.iter().find(|p| &p.name == pname).ok_or_else(|| {
                at(
                    &def_module,
                    line,
                    pname,
                    &format!("instance '{path}' misses param '{pname}'"),
                )
            })?;
            let module_src = &project.modules[&def_module].source;
            let pline = prop_line(module_src, pr);
            // PLAN-030：本参数的名义引用类别（refs 档才有；旧档 None）。
            let refk = if refs {
                decl.params
                    .iter()
                    .find(|(n, _)| n.as_str() == pname.as_str())
                    .and_then(|(_, ty)| crate::node3d_binding_refs::ref_kind_of(ty))
            } else {
                None
            };
            let parent_def: Option<String> = parent
                .and_then(|p| w.instances.get(p))
                .map(|i| i.def.clone());
            let val = match &pr.value {
                // —— PLAN-030：名义引用参数绑定面 ——
                // Slot/Pair/Template：根池/模板裸标识符，或同名同类别父参
                // 转传。Actor：refs.<实例路径>，或同类别父参转传。
                // msg：`.根事件`（零预绑实参），或父 msg 参数转传。
                // 值以 Res 文本承载（名字/路径/`.事件`），数值 ABI 由装配
                // 下降期解析；运行期身份不变（Run 池/行表冻结）。
                _ if refs && refk.is_some() => {
                    let rk = refk.expect("checked");
                    use crate::node3d_binding_refs::RefKind;
                    let forward =
                        |m: &str| -> Result<ParamVal, crate::translate::LowerError> {
                            let pd = parent_def.clone().ok_or_else(|| {
                                at(
                                    &def_module,
                                    pline,
                                    pname,
                                    &format!(
                                        "instance '{path}' param '{pname}' forwards '.{m}' but the definition is instanced at the root (no parent params)"
                                    ),
                                )
                            })?;
                            let pdecl = node_of(project, &pd);
                            let pk = pdecl
                                .params
                                .iter()
                                .find(|(n, _)| n.as_str() == m)
                                .and_then(|(_, ty)| crate::node3d_binding_refs::ref_kind_of(ty));
                            if pk != Some(rk) {
                                return Err(at(
                                    &def_module,
                                    pline,
                                    pname,
                                    &format!(
                                        "instance '{path}' param '{pname}' forwards '.{m}' with kind mismatch (want {})",
                                        rk.spell()
                                    ),
                                ));
                            }
                            parent_params
                                .iter()
                                .find(|(n, _)| n.as_str() == m)
                                .map(|(_, v)| v.clone())
                                .ok_or_else(|| {
                                    at(
                                        &def_module,
                                        pline,
                                        pname,
                                        &format!(
                                            "instance '{path}' param '{pname}' references unknown parent member '.{m}'"
                                        ),
                                    )
                                })
                        };
                    match (&pr.value, rk) {
                        // 根池/模板点根符号（Slot: objects3d.<槽>；
                        // Template: templates.<模板>；Pair: triggers.<pair>；
                        // 类别错配的根由装配期池查表具名拒绝）。
                        (PropValue::One(Expr::Dot(outer, n)), RefKind::Slot)
                            if matches!(&**outer, Expr::Ident(b) if b == "objects3d") =>
                        {
                            ParamVal::Res(format!("objects3d.{n}"))
                        }
                        (PropValue::One(Expr::Dot(outer, n)), RefKind::Template)
                            if matches!(&**outer, Expr::Ident(b) if b == "templates") =>
                        {
                            ParamVal::Res(format!("templates.{n}"))
                        }
                        (PropValue::One(Expr::Dot(outer, n)), RefKind::Pair)
                            if matches!(&**outer, Expr::Ident(b) if b == "triggers") =>
                        {
                            ParamVal::Res(format!("triggers.{n}"))
                        }
                        // 类别错配的点根（如 slot: triggers.x / pair:
                        // objects3d.y）——具名拒绝，不靠池查表假证。
                        (PropValue::One(Expr::Dot(outer, _)), RefKind::Slot)
                        | (PropValue::One(Expr::Dot(outer, _)), RefKind::Template)
                        | (PropValue::One(Expr::Dot(outer, _)), RefKind::Pair)
                            if matches!(&**outer, Expr::Ident(b)
                                if b == "objects3d" || b == "triggers" || b == "templates") =>
                        {
                            return Err(at(
                                &def_module,
                                pline,
                                pname,
                                &format!(
                                    "instance '{path}' param '{pname}' binds a {} port with the wrong symbol root (SlotRef3D takes objects3d.<name>, TemplateRef3D takes templates.<name>, PairRef3D takes triggers.<name>)",
                                    rk.spell()
                                ),
                            ))
                        }
                        // 父 msg 参数转传（裸名形态；`.Ev` 是绑定形态）。
                        (PropValue::One(Expr::Ident(n)), RefKind::MsgEvent) => {
                            forward(n.as_str())?
                        }
                        (PropValue::One(Expr::Ident(n)), RefKind::Actor) => {
                            return Err(at(
                                &def_module,
                                pline,
                                pname,
                                &format!(
                                    "instance '{path}' param '{pname}' binds bare '{n}'; ActorRef3D takes refs.<instance path>"
                                ),
                            ));
                        }
                        // `.x`：msg 档 = `.根事件` 绑定；数值类别 = 父参转传。
                        (PropValue::One(Expr::Dot(outer, m)), _)
                            if matches!(&**outer, Expr::Ident(b) if b == "self") =>
                        {
                            match rk {
                                RefKind::MsgEvent => ParamVal::Res(format!(".{m}")),
                                _ => forward(m.as_str())?,
                            }
                        }
                        // refs.<实例路径>（一段或两段；实例路径 = 声明先序
                        // 实例名链）。
                        (PropValue::One(Expr::Dot(outer, inst)), RefKind::Actor)
                            if matches!(&**outer, Expr::Ident(b) if b == "refs") =>
                        {
                            ParamVal::Res(format!("refs.{inst}"))
                        }
                        (PropValue::One(Expr::Dot(outer, leaf)), RefKind::Actor)
                            if matches!(&**outer, Expr::Dot(o2, _)
                                if matches!(&**o2, Expr::Ident(b) if b == "refs")) =>
                        {
                            if let Expr::Dot(_, mid) = &**outer {
                                ParamVal::Res(format!("refs.{mid}.{leaf}"))
                            } else {
                                unreachable!("matched above")
                            }
                        }
                        // `.Ev()` 空括号形态 = `.Ev`；带实参 v1 具名拒绝。
                        (PropValue::One(Expr::Call(c)), RefKind::MsgEvent) => {
                            let is_self_ev = matches!(&*c.name, Expr::Dot(i, _)
                                if matches!(&**i, Expr::Ident(b) if b == "self"));
                            if !is_self_ev || !c.args.args.is_empty() {
                                return Err(at(
                                    &def_module,
                                    pline,
                                    pname,
                                    &format!(
                                        "instance '{path}' param '{pname}': msg port binding must be `.RootEvent` (zero pre-bound args in components-3d-bindings v1)"
                                    ),
                                ));
                            }
                            match &*c.name {
                                Expr::Dot(_, ev) => ParamVal::Res(format!(".{ev}")),
                                _ => unreachable!("checked above"),
                            }
                        }
                        _ => {
                            return Err(at(
                                &def_module,
                                pline,
                                pname,
                                &format!(
                                    "instance '{path}' param '{pname}' must bind a {} symbol in the allowed form (pool/template bare symbol, refs.<instance path>, or .RootEvent)",
                                    rk.spell()
                                ),
                            ))
                        }
                    }
                }
                PropValue::One(Expr::Dot(outer, m))
                    if matches!(&**outer, Expr::Ident(b) if b == "self") =>
                {
                    parent_params
                        .iter()
                        .find(|(n, _)| n.as_str() == m)
                        .map(|(_, v)| v.clone())
                        .ok_or_else(|| {
                            at(
                                &def_module,
                                pline,
                                pname,
                                &format!(
                                    "instance '{path}' param '{pname}' references unknown parent member '.{}'",
                                    m
                                ),
                            )
                        })?
                }
                PropValue::One(Expr::Ident(p)) => parent_params
                    .iter()
                    .find(|(n, _)| n == p.as_str())
                    .map(|(_, v)| v.clone())
                    .ok_or_else(|| {
                        at(
                            &def_module,
                            pline,
                            pname,
                            &format!(
                                "instance '{path}' param '{pname}' references unknown parent param '{p}'"
                            ),
                        )
                    })?,
                PropValue::One(Expr::Dot(outer, m))
                    if matches!(&**outer, Expr::Ident(b) if b == "resources") =>
                {
                    if !matches!(pty, PType::Model) {
                        return Err(at(
                            &def_module,
                            pline,
                            pname,
                            &format!(
                                "instance '{path}' param '{pname}' is numeric but binds resource 'resources.{}'",
                                m
                            ),
                        ));
                    }
                    if !w.models.contains_key(m.as_str()) {
                        return Err(at(
                            &def_module,
                            pline,
                            pname,
                            &format!(
                                "instance '{path}' param '{pname}' binds 'resources.{}' which is not a model3d.glb resource",
                                m
                            ),
                        ));
                    }
                    ParamVal::Res(m.to_string())
                }
                PropValue::One(e) => {
                    if matches!(pty, PType::Model) {
                        return Err(at(
                            &def_module,
                            pline,
                            pname,
                            "resource param binding must be resources.<name> or a parent param",
                        ));
                    }
                    let v = numeric_lit(e).ok_or_else(|| {
                        at(
                            &def_module,
                            pline,
                            pname,
                            &format!(
                                "instance '{path}' param '{pname}' must be a numeric literal, got {e:?}"
                            ),
                        )
                    })?;
                    ParamVal::Num(v)
                }
                other => {
                    return Err(at(
                        &def_module,
                        pline,
                        pname,
                        &format!(
                            "instance '{path}' param '{pname}' must be a single value, got {other:?}"
                        ),
                    ))
                }
            };
            params.push((pname.clone(), val));
        }
        for pr in props {
            if !decl.params.iter().any(|(n, _)| n == &pr.name) {
                let module_src = &project.modules[&def_module].source;
                return Err(at(
                    &def_module,
                    prop_line(module_src, pr),
                    &pr.name,
                    &format!(
                        "instance '{path}' passes unknown prop '{}' (node '{}' params: {})",
                        pr.name,
                        node_name,
                        decl.params
                            .iter()
                            .map(|(n, _)| n.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                ));
            }
        }

        let def_idx = w.def_len(node_name);
        let stateful_inst = w.defs[node_name].stateful;
        let index = w.instances.len();
        w.instances.push(InstanceIR {
            path: path.to_string(),
            def: node_name.to_string(),
            parent,
            def_idx,
            params,
            line,
            parent_group,
        });
        if stateful_inst {
            w.stateful += 1;
            if w.stateful > 8 {
                return Err(at(
                    &def_module,
                    line,
                    inst_name,
                    &format!(
                        "stateful component instance '{path}' exceeds the 8-instance envelope (root included, total {})",
                        w.stateful
                    ),
                ));
            }
        }
        stack.push((node_name.to_string(), path.to_string()));
        // PLAN-030：refs 档纯逻辑 Node 可无 view-3d（空子树展开）；旧档
        // 保持具名拒绝。
        let no_view_elements: Vec<View3dElement>;
        let decl_view_elements: &[View3dElement] = match decl.view3d.as_ref() {
            Some(v) => &v.elements,
            None if refs => {
                no_view_elements = Vec::new();
                &no_view_elements
            }
            None => {
                return Err(at(
                    &def_module,
                    w.defs[node_name].line,
                    node_name,
                    "component definition has no view-3d",
                ))
            }
        };
        expand_children_3d(
            project,
            w,
            views,
            decl_view_elements,
            node_name,
            path,
            Some(index),
            None,
            stack,
            depth + 1,
            builtin_transform,
            refs,
        )?;
        stack.pop();
        Ok(())
    }

    fn expand_children_3d(
        project: &Project,
        w: &mut WorldIR,
        views: &mut BTreeMap<String, View3dIR>,
        elements: &[View3dElement],
        owner_def: &str,
        owner_path: &str,
        owner_index: Option<usize>,
        current_group: Option<String>,
        stack: &mut Vec<(String, String)>,
        depth: usize,
        builtin_transform: bool,
        refs: bool,
    ) -> Result<(), LowerError> {
        for el in elements {
            match el {
                View3dElement::Instance { node, name, props } => {
                    let line = props
                        .first()
                        .map(|p| {
                            prop_line(
                                &project.modules[w.defs[owner_def].module.as_str()].source,
                                p,
                            )
                        })
                        .unwrap_or(w.defs[owner_def].line);
                    let child_path = format!("{owner_path}/{name}");
                    let owner_module = w.defs[owner_def].module.clone();
                    expand_3d(
                        project,
                        w,
                        views,
                        node,
                        name,
                        &child_path,
                        owner_index,
                        current_group.clone(),
                        props,
                        line,
                        &owner_module,
                        stack,
                        depth,
                        builtin_transform,
                        refs,
                    )?;
                }
                View3dElement::Group { name, children, .. } => {
                    let mut group = current_group.clone().unwrap_or_default();
                    if !group.is_empty() {
                        group.push('/');
                    }
                    group.push_str(name);
                    expand_children_3d(
                        project,
                        w,
                        views,
                        children,
                        owner_def,
                        owner_path,
                        owner_index,
                        Some(group),
                        stack,
                        depth,
                        builtin_transform,
                        refs,
                    )?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    let mut stack: Vec<(String, String)> = Vec::new();
    let root_name = world.root.clone();
    let root_decl = node_of(project, &root_name);
    // PLAN-027：root 可无 view-3d（如仅携带 camera3d/on 的运行 root）；
    // 旧档根必须有视图，缺视图的旧拒绝文本保持不变。
    let root_view_elements: Vec<View3dElement> = match &root_decl.view3d {
        Some(v) => v.elements.clone(),
        None => {
            if root_decl.character3d.is_none() && root_decl.camera3d.is_none() {
                return Err(at(
                    &root_module,
                    world.defs[&root_name].line,
                    &root_name,
                    "root node must declare a view-3d (components-3d frozen surface)",
                ));
            }
            Vec::new()
        }
    };
    // root 视图的直接实例 parent = None（root 不是实例）；实例点所在
    // caller group 记录到 InstanceIR.parent_group。
    expand_children_3d(
        project,
        &mut world,
        &mut views,
        &root_view_elements,
        &root_name,
        &root_name,
        None,
        None,
        &mut stack,
        0,
        builtin_transform,
        refs,
    )?;

    Ok(World3d { world, views })
}

// ------------------------------------------------------------ scene build --

/// 一行 3D 场景节点（组或对象）。
#[derive(Debug, Clone)]
pub(crate) struct Row3 {
    pub is_group: bool,
    pub name: String,
    pub kind: &'static str,
    pub translation: [f64; 3],
    pub scale: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub visible: bool,
    pub parent: Option<u64>,
    /// model3d 资源解析结果（"sha256:<hex>"）。
    pub resource: Option<String>,
}

pub(crate) struct SceneBuild3 {
    /// 先序（= driver ordered() 序）；行 id = 下标 + 1。
    pub rows: Vec<Row3>,
    /// 行名 → 1-based 句柄。
    pub handles: BTreeMap<String, i64>,
    /// 每定义 → 每元素 → 每实例行名（def_idx 序；root 定义为单行）。
    pub elem_rows: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    /// 根 view 元素行名。
    pub root_rows: BTreeMap<String, String>,
}

/// 行规划中间态。
struct PendingRow {
    key: String,
    row: Row3,
    parent_key: Option<String>,
}

/// CVal → 初始化数值（数值/参数逐实例常量/`.field` 经 model init）。
pub(crate) fn cval_init_value(
    project: &Project,
    w3: &World3d,
    def: &DefIR,
    inst: Option<&InstanceIR>,
    v: &CVal,
    label: &str,
) -> Result<f64, LowerError> {
    match v {
        CVal::Num(n) => Ok(*n),
        CVal::Ref(r) => {
            if let Some(p) = r.strip_prefix('\x01') {
                let inst = inst.ok_or_else(|| {
                    at(
                        &def.module,
                        def.line,
                        p,
                        "param ref outside an instance context",
                    )
                })?;
                match inst.params.iter().find(|(n, _)| n == p).map(|(_, v)| v) {
                    Some(crate::lower_components::ParamVal::Num(v)) => Ok(*v),
                    Some(crate::lower_components::ParamVal::Res(_)) => Err(at(
                        &def.module,
                        def.line,
                        p,
                        "resource param cannot initialize a numeric prop",
                    )),
                    None => Err(at(
                        &def.module,
                        def.line,
                        p,
                        "instance param binding missing",
                    )),
                }
            } else {
                let inst = inst.ok_or_else(|| {
                    at(
                        &def.module,
                        def.line,
                        label,
                        "model ref outside an instance context",
                    )
                })?;
                model_init_value(project, &w3.world, def, r, inst)
            }
        }
    }
}

/// 一个定义实例（或 root）的 view-3d 行发射（组/叶，声明序）。
#[allow(clippy::too_many_arguments)]
fn emit_def_rows(
    project: &Project,
    w3: &World3d,
    def_name: &str,
    path: &str,
    inst: Option<&InstanceIR>,
    group_key: Option<String>,
    pending: &mut Vec<PendingRow>,
) -> Result<(), LowerError> {
    let world = &w3.world;
    let def = &world.defs[def_name];
    let v3 = &w3.views[def_name];

    for g in &v3.groups {
        let key = format!("{path}/{}", g.name);
        let mut t = [0.0f64; 3];
        for (i, v) in g.position.iter().enumerate() {
            t[i] = cval_init_value(project, w3, def, inst, v, &format!("{key}:position"))?;
        }
        let parent_key = g
            .parent_chain
            .last()
            .map(|p| format!("{path}/{p}"))
            .or_else(|| group_key.clone());
        pending.push(PendingRow {
            key: key.clone(),
            row: Row3 {
                is_group: true,
                name: key.clone(),
                kind: "group",
                translation: t,
                scale: [1.0, 1.0, 1.0],
                yaw: 0.0,
                pitch: 0.0,
                visible: g.visible,
                parent: None,
                resource: None,
            },
            parent_key,
        });
    }
    for e in &v3.elems {
        let key = format!("{path}/{}", e.elem);
        let mut t = [0.0f64; 3];
        for (i, v) in e.position.iter().enumerate() {
            t[i] = cval_init_value(project, w3, def, inst, v, &format!("{key}:position"))?;
        }
        let mut s = [1.0f64; 3];
        for (i, v) in e.scale.iter().enumerate() {
            s[i] = cval_init_value(project, w3, def, inst, v, &format!("{key}:scale"))?;
        }
        let yaw = cval_init_value(project, w3, def, inst, &e.yaw, &format!("{key}:yaw"))?;
        let pitch = cval_init_value(project, w3, def, inst, &e.pitch, &format!("{key}:pitch"))?;
        let visible = cval_init_value(
            project,
            w3,
            def,
            inst,
            &e.visible,
            &format!("{key}:visible"),
        )? != 0.0;
        let resource = resolve_model_3d(w3, def, inst, e)?;
        let parent_key = e
            .groups
            .last()
            .map(|g| format!("{path}/{g}"))
            .or_else(|| group_key.clone());
        pending.push(PendingRow {
            key: key.clone(),
            row: Row3 {
                is_group: false,
                name: key.clone(),
                kind: e.kind,
                translation: t,
                scale: s,
                yaw,
                pitch,
                visible,
                parent: None,
                resource,
            },
            parent_key,
        });
    }
    Ok(())
}

/// model3d 引用解析（直连资源或逐实例 Model3D 参数）。
fn resolve_model_3d(
    w3: &World3d,
    def: &DefIR,
    inst: Option<&InstanceIR>,
    e: &ElemIR3,
) -> Result<Option<String>, LowerError> {
    let world = &w3.world;
    let Some(m) = &e.model else {
        return Ok(None);
    };
    match m {
        ModelRef::Direct(r) => {
            let (sha, _) = world.models.get(r).ok_or_else(|| {
                at(
                    &def.module,
                    e.line,
                    &e.elem,
                    &format!("model binding 'resources.{r}' is not a model3d.glb resource"),
                )
            })?;
            Ok(Some(format!("sha256:{sha}")))
        }
        ModelRef::Param(p) => {
            let inst = inst.ok_or_else(|| {
                at(
                    &def.module,
                    e.line,
                    &e.elem,
                    "model param outside an instance context",
                )
            })?;
            match inst.params.iter().find(|(n, _)| n == p).map(|(_, v)| v) {
                Some(crate::lower_components::ParamVal::Res(r)) => {
                    let (sha, _) = world.models.get(r).ok_or_else(|| {
                        at(
                            &def.module,
                            e.line,
                            &e.elem,
                            &format!("model param '{p}' binds 'resources.{r}' which is not a model3d.glb resource"),
                        )
                    })?;
                    Ok(Some(format!("sha256:{sha}")))
                }
                _ => Err(at(
                    &def.module,
                    e.line,
                    &e.elem,
                    &format!("model param '{p}' binding is not a model3d resource"),
                )),
            }
        }
    }
}

/// 收集三维场景：树先序行 + driver `ordered()` 同规则句柄。
pub(crate) fn collect_scene_3d(project: &Project, w3: &World3d) -> Result<SceneBuild3, LowerError> {
    collect_scene_3d_with(project, w3, &[])
}

/// PLAN-028：spawn 模板行输入（events 档；行名 = 模板定义名，值全部
/// 为已求值常量）。模板行在 root/实例行之后按给定序追加。
#[derive(Clone)]
pub(crate) struct TemplateRowIn {
    pub name: String,
    pub kind: &'static str,
    pub translation: [f64; 3],
    pub scale: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub visible: bool,
    pub resource: Option<String>,
}

pub(crate) fn collect_scene_3d_with(
    project: &Project,
    w3: &World3d,
    templates: &[TemplateRowIn],
) -> Result<SceneBuild3, LowerError> {
    let world = &w3.world;
    let mut pending: Vec<PendingRow> = Vec::new();

    // —— root 视图（组/叶直接用原名；无 caller group）——
    let root_def_name = world.root.clone();
    {
        let v3 = &w3.views[&root_def_name];
        let def = &world.defs[&root_def_name];
        for g in &v3.groups {
            let key = g.name.clone();
            let mut t = [0.0f64; 3];
            for (i, v) in g.position.iter().enumerate() {
                t[i] = cval_init_value(project, w3, def, None, v, &format!("{key}:position"))?;
            }
            pending.push(PendingRow {
                key: key.clone(),
                row: Row3 {
                    is_group: true,
                    name: key.clone(),
                    kind: "group",
                    translation: t,
                    scale: [1.0, 1.0, 1.0],
                    yaw: 0.0,
                    pitch: 0.0,
                    visible: g.visible,
                    parent: None,
                    resource: None,
                },
                parent_key: g.parent_chain.last().cloned(),
            });
        }
        for e in &v3.elems {
            let key = e.elem.clone();
            let mut t = [0.0f64; 3];
            for (i, v) in e.position.iter().enumerate() {
                t[i] = cval_init_value(project, w3, def, None, v, &format!("{key}:position"))?;
            }
            let mut s = [1.0f64; 3];
            for (i, v) in e.scale.iter().enumerate() {
                s[i] = cval_init_value(project, w3, def, None, v, &format!("{key}:scale"))?;
            }
            let yaw = cval_init_value(project, w3, def, None, &e.yaw, &format!("{key}:yaw"))?;
            let pitch = cval_init_value(project, w3, def, None, &e.pitch, &format!("{key}:pitch"))?;
            let visible = cval_init_value(
                project,
                w3,
                def,
                None,
                &e.visible,
                &format!("{key}:visible"),
            )? != 0.0;
            if e.model.is_some() {
                if let Some(ModelRef::Param(p)) = &e.model {
                    return Err(at(
                        &def.module,
                        e.line,
                        &e.elem,
                        &format!(
                            "root view model binding cannot use param '{p}' (root has no params)"
                        ),
                    ));
                }
            }
            let resource = resolve_model_3d(w3, def, None, e)?;
            pending.push(PendingRow {
                key: key.clone(),
                row: Row3 {
                    is_group: false,
                    name: key.clone(),
                    kind: e.kind,
                    translation: t,
                    scale: s,
                    yaw,
                    pitch,
                    visible,
                    parent: None,
                    resource,
                },
                parent_key: e.groups.last().cloned(),
            });
        }
    }
    // —— 实例行（声明先序；嵌套实例随后序由其 InstanceIR 顺序覆盖）——
    for i in &world.instances {
        emit_def_rows(
            project,
            w3,
            &i.def,
            &i.path,
            Some(i),
            i.parent_group.clone(),
            &mut pending,
        )?;
    }

    // —— PLAN-028 spawn 模板行（声明序；parked 由作者声明携带）——
    for t in templates {
        pending.push(PendingRow {
            key: t.name.clone(),
            row: Row3 {
                is_group: false,
                name: t.name.clone(),
                kind: t.kind,
                translation: t.translation,
                scale: t.scale,
                yaw: t.yaw,
                pitch: t.pitch,
                visible: t.visible,
                parent: None,
                resource: t.resource.clone(),
            },
            parent_key: None,
        });
    }

    // 名字唯一 + 128 包络。
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for p in &pending {
        if !seen.insert(p.key.clone()) {
            return Err(at(
                &world.root_module,
                0,
                &p.key,
                "duplicate scene row name",
            ));
        }
    }
    if pending.len() > 128 {
        return Err(at(
            &world.root_module,
            0,
            "scene",
            &format!(
                "components-3d scene exceeds 128 nodes ({} rows)",
                pending.len()
            ),
        ));
    }

    // 句柄：复刻 driver ordered()——节点表 = 组（声明序）++ 对象（声明序），
    // 根/兄弟按该表序先序 DFS，id = 位置 + 1。
    let mut doc: Vec<&PendingRow> = Vec::new();
    doc.extend(pending.iter().filter(|p| p.row.is_group));
    doc.extend(pending.iter().filter(|p| !p.row.is_group));
    let mut stack: Vec<usize> = doc
        .iter()
        .enumerate()
        .filter(|(_, p)| p.parent_key.is_none())
        .map(|(i, _)| i)
        .rev()
        .collect();
    let mut preorder: Vec<usize> = Vec::new();
    let mut visited: BTreeSet<usize> = BTreeSet::new();
    while let Some(i) = stack.pop() {
        if !visited.insert(i) {
            return Err(at(&world.root_module, 0, &doc[i].key, "scene graph cycle"));
        }
        preorder.push(i);
        let children: Vec<usize> = doc
            .iter()
            .enumerate()
            .filter(|(_, c)| c.parent_key.as_deref() == Some(doc[i].key.as_str()))
            .map(|(j, _)| j)
            .collect();
        for c in children.into_iter().rev() {
            stack.push(c);
        }
    }
    if preorder.len() != doc.len() {
        return Err(at(
            &world.root_module,
            0,
            "scene",
            "scene graph incomplete (missing parent or cycle)",
        ));
    }

    // v1 行句柄 = ordered() 0-based 下标（ag_set_local/ag_set_visible 按
    // fields[node*12+axis] 寻址；ag_valid 允许 0）。文档行 id 仍是 1-based
    // 标签（见 scene_json 的 id = 下标 + 1），两套序号在此分叉。
    let mut handles: BTreeMap<String, i64> = BTreeMap::new();
    for (pos, &i) in preorder.iter().enumerate() {
        handles.insert(doc[i].key.clone(), pos as i64);
    }
    let mut rows: Vec<Row3> = Vec::with_capacity(pending.len());
    for p in pending.iter_mut() {
        if let Some(pk) = &p.parent_key {
            p.row.parent = handles.get(pk.as_str()).copied().map(|v| (v + 1) as u64);
        }
    }
    for &i in &preorder {
        rows.push(pending[i].row.clone());
    }

    // 元素行表（生成面：行句柄数组 / root 字面量）。
    let mut elem_rows: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    let mut root_rows = BTreeMap::new();
    {
        let v3 = &w3.views[&root_def_name];
        for e in &v3.elems {
            root_rows.insert(e.elem.clone(), e.elem.clone());
            elem_rows
                .entry(root_def_name.clone())
                .or_default()
                .insert(e.elem.clone(), vec![e.elem.clone()]);
        }
    }
    for i in &world.instances {
        let v3 = &w3.views[&i.def];
        for e in &v3.elems {
            let key = format!("{}/{}", i.path, e.elem);
            let list = elem_rows
                .entry(i.def.clone())
                .or_default()
                .entry(e.elem.clone())
                .or_default();
            while list.len() <= i.def_idx {
                list.push(String::new());
            }
            list[i.def_idx] = key;
        }
    }
    for (dname, elems) in &elem_rows {
        for (ename, list) in elems {
            if list.iter().any(|s| s.is_empty()) {
                let def = &world.defs[dname];
                return Err(at(
                    &def.module,
                    def.line,
                    ename,
                    "element row table has an unregistered instance slot",
                ));
            }
        }
    }

    Ok(SceneBuild3 {
        rows,
        handles,
        elem_rows,
        root_rows,
    })
}

// ------------------------------------------------------------- generation --

/// 生成用户源（world.at，v1 行为装配）。
/// PLAN-027：runtime 档注入配置（None = 024 字节不变路径）。
#[derive(Debug, Default)]
pub(crate) struct RuntimeCfg3 {
    /// 恰一角色定义名（ctx 角色服务授予）。
    pub character_def: Option<String>,
    /// 恰一角色：绑定声明 id（1-based 文档 id）。
    pub character_bind_id: Option<i64>,
    /// solid 叶：注入序（声明先序）的 `ag_char3_solid` 文档 id。
    pub solid_ids: Vec<i64>,
    /// 相机配置：`ag_camera_orbit_3d(角色 id, distance, pitch, projection, ortho_size)`。
    pub camera: Option<(i64, f64, f64, i64, f64)>,
}

/// PLAN-027：runtime 环境盖章（六动作表 + 相机读取；角色定义另含角色
/// 服务）。None（024 档）不触碰任何字段。
fn stamp_runtime_env(
    ctx: &mut crate::translate::Ctx,
    def_name: &str,
    rt: Option<&RuntimeCfg3>,
    events: Option<&crate::lower_components3d_events::EventsCfg3>,
) {
    if let Some(rt) = rt {
        ctx.six_actions = true;
        ctx.cam3 = true;
        if rt.character_def.as_deref() == Some(def_name) {
            ctx.char3 = Some(crate::translate::Char3Env);
        }
    }
    if let Some(ev) = events {
        ctx.events3d = Some(crate::translate::EventsEnv {
            slots: ev.slot_ids.clone(),
            pairs: ev.pair_ids.clone(),
            templates: ev.template_ids.clone(),
            subjects: ev.subject_ids.clone(),
        });
    }
}

pub(crate) fn gen_user_source_3d(
    project: &Project,
    w3: &World3d,
    plan: &Plan,
    scene: &SceneBuild3,
    runtime: Option<&RuntimeCfg3>,
    events: Option<&crate::lower_components3d_events::EventsCfg3>,
    bindings: Option<&crate::lower_components3d_bindings::BindingsCfg3>,
    gen_note: &str,
    regen_note: &str,
) -> Result<String, LowerError> {
    let world = &w3.world;
    let mut src = String::new();
    let root_def = &world.defs[&world.root];
    let root_decl = node_of(project, &world.root);

    src.push_str(&format!(
        "// GENERATED by autographics-node2d-pilot ({gen_note}, {API}).\n"
    ));
    let inst_list = world
        .instances
        .iter()
        .map(|i| format!("{}:{}", i.path, i.def))
        .collect::<Vec<_>>()
        .join(",");
    src.push_str(&format!(
        "// Root: {}; instances (declaration pre-order): {}\n",
        world.root, inst_list
    ));
    src.push_str(&format!(
        "// Regeneration must be byte-identical ({regen_note}).\n"
    ));
    src.push_str("var ok=0.0\n");

    // 枚举常量（019 约定，同 020）。
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

    // 根标量（r_<field>）。
    if !root_def.model.is_empty() {
        src.push_str("// root scalars (r_<field>)\n");
        if let Some(model) = &root_decl.model {
            for f in &model.fields {
                let lit = match &f.init {
                    Expr::Int(v) => format!("{v}"),
                    Expr::Float(_, raw) => raw.to_string(),
                    other => {
                        return Err(at(
                            &root_def.module,
                            root_def.line,
                            &f.name.to_string(),
                            &format!("root model init must be a numeric literal, got {other:?}"),
                        ))
                    }
                };
                src.push_str(&format!("var r_{}={lit}\n", f.name));
            }
        }
    }

    // 每 SoA：model 数组、运行期参数数组（handler 引用面）、行表与缓存。
    for dname in def_order(world) {
        let def = &world.defs[&dname];
        let idxs = world.instances_of(&dname);
        let paths = idxs
            .iter()
            .map(|&k| format!("{}={}", world.instances[k].path, world.instances[k].def_idx))
            .collect::<Vec<_>>()
            .join(", ");
        let head = if idxs.len() == 1 {
            "instance"
        } else {
            "instances"
        };
        src.push_str(&format!("// {} SoA ({} {})\n", dname, head, paths));
        for (fname, is_int) in &def.model {
            let mut vals = Vec::new();
            for &k in &idxs {
                let v = model_init_value(project, world, def, fname, &world.instances[k])?;
                vals.push(if *is_int {
                    format!("{}", v as i64)
                } else {
                    fmt_f(v)
                });
            }
            src.push_str(&format!(
                "var {}_{}=[{}]\n",
                def.prefix(),
                fname,
                vals.join(",")
            ));
        }
        for (pname, pt) in def_runtime_params_3d(project, def) {
            // PLAN-030：引用参数列值 = 解析后的原数值 ABI（池/模板身份）。
            let refk = node_of(project, &def.name)
                .params
                .iter()
                .find(|(n, _)| n.as_str() == pname.as_str())
                .and_then(|(_, ty)| crate::node3d_binding_refs::ref_kind_of(ty));
            let mut vals = Vec::new();
            for &k in &idxs {
                let inst = &world.instances[k];
                let (_, val) = inst
                    .params
                    .iter()
                    .find(|(n, _)| n == &pname)
                    .ok_or_else(|| {
                        at(
                            &def.module,
                            inst.line,
                            &pname,
                            "runtime param binding missing",
                        )
                    })?;
                if let (
                    crate::lower_components::ParamVal::Res(raw),
                    Some(rk),
                ) = (val, refk)
                {
                    use crate::node3d_binding_refs::RefKind;
                    // 点根剥离（装配期已核存在性；此处按同表取数值身份）。
                    let (rk_s, name): (&str, &str) = match rk {
                        RefKind::Slot => ("objects3d.", raw.strip_prefix("objects3d.").unwrap_or(raw)),
                        RefKind::Template => ("templates.", raw.strip_prefix("templates.").unwrap_or(raw)),
                        RefKind::Pair => ("triggers.", raw.strip_prefix("triggers.").unwrap_or(raw)),
                        _ => unreachable!("norun kinds filtered from SoA selection"),
                    };
                    let table = match rk {
                        RefKind::Slot => events.map(|ev| &ev.slot_ids),
                        RefKind::Template => events.map(|ev| &ev.template_ids),
                        RefKind::Pair => events.map(|ev| &ev.pair_ids),
                        _ => unreachable!("norun kinds filtered from SoA selection"),
                    };
                    let id = table
                        .and_then(|t| t.get(name))
                        .ok_or_else(|| {
                            at(
                                &def.module,
                                inst.line,
                                &pname,
                                &format!(
                                    "ref param '{pname}' binds unknown {} symbol '{name}'",
                                    rk_s.trim_end_matches('.')
                                ),
                            )
                        })?;
                    vals.push(id.to_string());
                    continue;
                }
                vals.push(val.text(pt == PType::Int));
            }
            src.push_str(&format!(
                "var {}_{}=[{}]\n",
                def.prefix(),
                pname,
                vals.join(",")
            ));
        }
        // PLAN-028：bind 主题文档 id 表（events 档；每元素每实例 1-based
        // 文档 id，def_idx 序；bind 端口按 [i] 取本实例主题行 id）。
        if let Some(ev) = events {
            for e in &w3.views[&dname].elems {
                let ids = ev
                    .subject_ids
                    .get(&(def.prefix().to_string(), e.elem.clone()))
                    .expect("events subject table covers every def elem");
                src.push_str(&format!(
                    "var {}_{}_subject_ids=[{}]\n",
                    def.prefix(),
                    e.elem,
                    ids.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
        }
        for e in &w3.views[&dname].elems {
            if !def.dyn_elems.contains(&e.elem) {
                // PLAN-028：events 档静态元素也发行表（Start place 与 bind
                // 主题按行寻址；常量行表，其余档保持 dyn-only 字节不变）。
                if let Some(ev) = events {
                    let rows = scene
                        .elem_rows
                        .get(&dname)
                        .and_then(|m| m.get(&e.elem))
                        .expect("elem rows recorded");
                    src.push_str(&format!(
                        "var {}_{}_rows=[{}]\n",
                        def.prefix(),
                        e.elem,
                        rows.iter()
                            .map(|r| scene.handles[r].to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    ));
                }
                continue;
            }
            // 行表（每实例 1-based 句柄；def_idx 序）。
            let rows = scene
                .elem_rows
                .get(&dname)
                .and_then(|m| m.get(&e.elem))
                .expect("elem rows recorded");
            src.push_str(&format!(
                "var {}_{}_rows=[{}]\n",
                def.prefix(),
                e.elem,
                rows.iter()
                    .map(|r| scene.handles[r].to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
            // 缓存初始化 = 静态初值（逐实例）。
            let mut px = Vec::new();
            let mut py = Vec::new();
            let mut pz = Vec::new();
            for &k in &idxs {
                let inst = &world.instances[k];
                px.push(fmt_f(cval_init_value(
                    project,
                    w3,
                    def,
                    Some(inst),
                    &e.position[0],
                    "px",
                )?));
                py.push(fmt_f(cval_init_value(
                    project,
                    w3,
                    def,
                    Some(inst),
                    &e.position[1],
                    "py",
                )?));
                pz.push(fmt_f(cval_init_value(
                    project,
                    w3,
                    def,
                    Some(inst),
                    &e.position[2],
                    "pz",
                )?));
            }
            src.push_str(&format!(
                "var {}_{}_px=[{}]\n",
                def.prefix(),
                e.elem,
                px.join(",")
            ));
            src.push_str(&format!(
                "var {}_{}_py=[{}]\n",
                def.prefix(),
                e.elem,
                py.join(",")
            ));
            src.push_str(&format!(
                "var {}_{}_pz=[{}]\n",
                def.prefix(),
                e.elem,
                pz.join(",")
            ));
            if e.dyn_vis {
                let mut vv = Vec::new();
                for &k in &idxs {
                    let inst = &world.instances[k];
                    let v = cval_init_value(project, w3, def, Some(inst), &e.visible, "vis")?;
                    vv.push(format!("{}", v as i64));
                }
                src.push_str(&format!(
                    "var {}_{}_vis=[{}]\n",
                    def.prefix(),
                    e.elem,
                    vv.join(",")
                ));
            }
        }
    }
    // root 动态呈现标量（root 行是字面量句柄）。
    for e in &w3.views[&world.root].elems {
        if !root_def.dyn_elems.contains(&e.elem) {
            continue;
        }
        let row = scene.handles[&scene.root_rows[&e.elem]];
        src.push_str(&format!("var r_{}_row={row}\n", e.elem));
        let px = cval_init_value(project, w3, root_def, None, &e.position[0], "px")?;
        let py = cval_init_value(project, w3, root_def, None, &e.position[1], "py")?;
        let pz = cval_init_value(project, w3, root_def, None, &e.position[2], "pz")?;
        src.push_str(&format!("var r_{}_px={}\n", e.elem, fmt_f(px)));
        src.push_str(&format!("var r_{}_py={}\n", e.elem, fmt_f(py)));
        src.push_str(&format!("var r_{}_pz={}\n", e.elem, fmt_f(pz)));
        if e.dyn_vis {
            let v = cval_init_value(project, w3, root_def, None, &e.visible, "vis")?;
            src.push_str(&format!("var r_{}_vis={}\n", e.elem, v as i64));
        }
    }

    // 事件绑定表（发射实例 i → 接收下标；root 发射方直呼无表）。
    let mut tables: Vec<String> = Vec::new();
    for ((edef, _, _), d) in &plan.child {
        if edef == &world.root || d.entries.is_empty() {
            continue;
        }
        tables.push(format!(
            "var {}=[{}]",
            d.table,
            d.entries
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    for ((edef, _), d) in &plan.anc {
        if edef == &world.root || d.to_root || d.table.is_empty() || d.entries.is_empty() {
            continue;
        }
        if d.entries.iter().all(|e| e.is_some()) {
            tables.push(format!(
                "var {}=[{}]",
                d.table,
                d.entries
                    .iter()
                    .map(|e| e.unwrap().to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    }
    if !tables.is_empty() {
        src.push_str("// event binding tables (emitter instance i -> receiver index)\n");
        tables.sort();
        tables.dedup();
        for t in tables {
            src.push_str(&t);
            src.push('\n');
        }
    }

    // 池化纯函数（模块作用域重命名；序稳定，与 020 同一发射器）。
    let mut fn_names: Vec<(String, String)> = Vec::new();
    for module in project.modules.values() {
        for (fname, _) in &module.fns {
            fn_names.push((module.name.clone(), fname.clone()));
        }
    }
    fn_names.sort();
    for (module_name, fname) in &fn_names {
        let module = &project.modules[module_name];
        let text = crate::lower::slice_fn_source(&module.source, fname)
            .ok_or_else(|| at(module_name, 0, fname, "fn source slice"))?;
        let siblings: Vec<String> = module.fns.iter().map(|(n, _)| n.clone()).collect();
        let renamed = rename_module_calls(&text, &siblings, module_name);
        src.push_str(&format!("// pooled fn {fname} from {module_name}\n"));
        src.push_str(renamed.trim_end());
        src.push('\n');
    }

    // 空间服务环境：root refs = 元素 → 字面量句柄；实例 refs = 行表下标。
    let root_refs: BTreeMap<String, String> = w3.views[&world.root]
        .elems
        .iter()
        .map(|e| {
            let row = &scene.root_rows[&e.elem];
            (e.elem.clone(), scene.handles[row].to_string())
        })
        .collect();
    let root_primary = w3.views[&world.root]
        .elems
        .first()
        .map(|e| scene.handles[&scene.root_rows[&e.elem]].to_string());

    // root Life/成员/呈现 fns。
    let mut rctx = root_ctx(project, world, plan, None, None, None);
    rctx.space3 = Some(SpaceEnv {
        refs: &root_refs,
        self_row: root_primary.clone(),
    });
    // PLAN-027：runtime 环境盖章（None = 024 档不触碰，字节不变）。
    stamp_runtime_env(&mut rctx, &world.root, runtime, events);
    // PLAN-030：引用参数环境盖章（None = 旧档不触碰，字节不变）。
    apply_bindings_ctx(
        &mut rctx,
        &world.root,
        &root_def.module,
        project,
        plan,
        bindings,
        "",
    );
    src.push_str("fn main_lifestart() {\n");
    if let Some(h) = life_handler(root_decl, "Start") {
        translate_handler(&h.body.stmts, &rctx, &mut src, 1)?;
    }
    src.push_str("}\n");
    src.push_str("fn main_lifetick(dt float) {\n");
    if let Some(h) = life_handler(root_decl, "Tick") {
        translate_handler(&h.body.stmts, &rctx, &mut src, 1)?;
    }
    src.push_str("}\n");
    let mut root_recvs: Vec<(&String, &String)> = plan
        .recv_fns
        .iter()
        .filter(|(_, (r, _))| r.is_empty())
        .map(|(f, (_, e))| (f, e))
        .collect();
    root_recvs.sort();
    for (fname, ev) in root_recvs {
        gen_recv_fn_3d(
            project,
            world,
            plan,
            fname,
            "",
            ev,
            &root_refs,
            root_primary.as_deref(),
            runtime,
            events,
            bindings,
            &mut src,
        )?;
    }
    if root_def.life_tick && !root_def.dyn_elems.is_empty() {
        gen_present_3d(project, w3, root_def, true, scene, &mut src)?;
    }

    // 每定义 fns（首个实例声明先序）。
    for dname in def_order(world) {
        let def = &world.defs[&dname];
        let prefix = def.prefix();
        let decl = node_of(project, &dname);
        if world.def_len(&dname) == 0 {
            continue;
        }
        let mut inst_refs: BTreeMap<String, String> = BTreeMap::new();
        for e in &w3.views[&dname].elems {
            inst_refs.insert(e.elem.clone(), format!("{prefix}_{}_rows[i]", e.elem));
        }
        let inst_primary = w3.views[&dname]
            .elems
            .first()
            .map(|e| format!("{prefix}_{}_rows[i]", e.elem));
        let mut ictx = instance_ctx(project, plan, def, &prefix, "i", None, None, None)?;
        ictx.space3 = Some(SpaceEnv {
            refs: &inst_refs,
            self_row: inst_primary.clone(),
        });
        stamp_runtime_env(&mut ictx, &dname, runtime, events);
        // PLAN-030：引用参数环境盖章 + msg 参数投递覆盖（None = 旧档不变）。
        apply_bindings_ctx(&mut ictx, &dname, &def.module, project, plan, bindings, "i");
        if def.life_tick {
            src.push_str(&format!("fn {prefix}_lifetick(i int, dt float) {{\n"));
            if let Some(h) = life_handler(decl, "Tick") {
                translate_handler(&h.body.stmts, &ictx, &mut src, 1)?;
            }
            src.push_str("}\n");
        }
        if def.life_start {
            src.push_str(&format!("fn {prefix}_lifestart(i int) {{\n"));
            if let Some(h) = life_handler(decl, "Start") {
                translate_handler(&h.body.stmts, &ictx, &mut src, 1)?;
            }
            src.push_str("}\n");
        }
        let mut mine: Vec<(&String, &String)> = plan
            .recv_fns
            .iter()
            .filter(|(_, (r, _))| r == &dname)
            .map(|(f, (_, e))| (f, e))
            .collect();
        mine.sort();
        for (fname, ev) in mine {
            gen_recv_fn_3d(
                project,
                world,
                plan,
                fname,
                &dname,
                ev,
                &inst_refs,
                inst_primary.as_deref(),
                runtime,
                events,
                bindings,
                &mut src,
            )?;
        }
        if def.life_tick && !def.dyn_elems.is_empty() {
            gen_present_3d(project, w3, def, false, scene, &mut src)?;
        }
    }

    // getters（root + 每实例；观测全部 model 字段）。
    for (fname, is_int) in &root_def.model {
        let ty = if *is_int { "int" } else { "float" };
        src.push_str(&format!("fn get_r_{fname}() {ty} {{return r_{fname}}}\n"));
    }
    for dname in def_order(world) {
        let def = &world.defs[&dname];
        let prefix = def.prefix();
        for inst in world.instances.iter().filter(|i| i.def == dname) {
            for (fname, is_int) in &def.model {
                let ty = if *is_int { "int" } else { "float" };
                let pathu = inst.path.replace('/', "_");
                src.push_str(&format!(
                    "fn get_{prefix}_{fname}_{idx}() {ty} {{return {prefix}_{fname}[{idx}]}}\n",
                    idx = inst.def_idx
                ));
                src.push_str(&format!(
                    "fn get_{pathu}_{fname}() {ty} {{return {prefix}_{fname}[{idx}]}}\n",
                    idx = inst.def_idx
                ));
            }
        }
    }

    // app_start：Life.Start 们（根先、声明先序），随后 runtime 档注入
    // bind/solid/camera 配置（作者代码在前；主机在 app_start 之后织入
    // _ag_char3_freeze/_ag_cam3_freeze，配置只需落在 app_start 内）。
    src.push_str("fn app_start() {\n");
    if root_def.life_start {
        src.push_str(" main_lifestart()\n");
    }
    for inst in &world.instances {
        let def = &world.defs[&inst.def];
        if def.life_start {
            src.push_str(&format!(" {}_lifestart({})\n", def.prefix(), inst.def_idx));
        }
    }
    if let Some(rt) = runtime {
        if let Some(bind) = rt.character_bind_id {
            src.push_str(&format!(" ok=ok+ag_char3_bind({bind})\n"));
        }
        for solid in &rt.solid_ids {
            src.push_str(&format!(" ok=ok+ag_char3_solid({solid})\n"));
        }
        if let Some((target, distance, pitch, projection, ortho_size)) = &rt.camera {
            src.push_str(&format!(
                " ok=ok+ag_camera_orbit_3d({target}, {}, {}, {projection}, {})\n",
                fmt_f(*distance),
                fmt_f(*pitch),
                fmt_f(*ortho_size)
            ));
        }
    }
    src.push_str("}\n");

    // app_update（冻结序：根 → 声明先序 tick → present）。
    src.push_str("fn app_update(dt float) {\n");
    src.push_str(" main_lifetick(dt)\n");
    if root_def.life_tick && !root_def.dyn_elems.is_empty() {
        src.push_str(" main_present()\n");
    }
    for inst in &world.instances {
        let def = &world.defs[&inst.def];
        if def.life_tick {
            src.push_str(&format!(
                " {}_lifetick({},dt)\n",
                def.prefix(),
                inst.def_idx
            ));
        }
        if def.life_tick && !def.dyn_elems.is_empty() {
            src.push_str(&format!(" {}_present({})\n", def.prefix(), inst.def_idx));
        }
    }
    src.push_str("}\n");
    Ok(src)
}

/// 运行期参数数组（handler 引用到的数值参数；视图静态引用逐实例内联）。
fn def_runtime_params_3d(project: &Project, def: &DefIR) -> Vec<(String, PType)> {
    let decl = node_of(project, &def.name);
    let mut out = WalkOut {
        calls: Vec::new(),
        members: BTreeSet::new(),
        assign_targets: Vec::new(),
        child_emits: Vec::new(),
        anc_emits: Vec::new(),
        query_events: Vec::new(),
        render_calls: Vec::new(),
    };
    if let Some(on) = &decl.on {
        for h in &on.handlers {
            walk_stmts(&h.body.stmts, def.line, &mut out, false, false);
        }
    }
    // PLAN-030：Actor/msg 引用参数无运行列（下降期内联解析/投递文本）；
    // Slot/Template/Pair 参数保留 int SoA 列（handler `.参数` 读面）。
    // 旧档定义不可能携带引用类型参数（ptypes 拒绝），过滤恒空集。
    let ref_norun: BTreeSet<String> = decl
        .params
        .iter()
        .filter(|(_, ty)| {
            matches!(
                crate::node3d_binding_refs::ref_kind_of(ty),
                Some(crate::node3d_binding_refs::RefKind::Actor)
                    | Some(crate::node3d_binding_refs::RefKind::MsgEvent)
            )
        })
        .map(|(n, _)| n.clone())
        .collect();
    def.params
        .iter()
        .filter(|(n, pt)| {
            !matches!(pt, PType::Model)
                && out.members.contains(n.as_str())
                && !ref_norun.contains(n.as_str())
        })
        .cloned()
        .collect()
}

/// 呈现同步 fn：`fn <prefix>_present(i int)`（root 无索引）。位置写
/// `ag_set_local(row, 0/1/2, …)`、可见写 `ag_set_visible(row, …)`，按
/// 变更计费门（平方差 / int != 缓存；020 同型）。
fn gen_present_3d(
    project: &Project,
    w3: &World3d,
    def: &DefIR,
    is_root: bool,
    _scene: &SceneBuild3,
    src: &mut String,
) -> Result<(), LowerError> {
    let world = &w3.world;
    let prefix = if is_root {
        "main".to_string()
    } else {
        def.prefix()
    };
    if is_root {
        src.push_str("fn main_present() {\n");
    } else {
        src.push_str(&format!("fn {prefix}_present(i int) {{\n"));
    }
    let index = if is_root { "" } else { "i" };
    for e in &w3.views[&def.name].elems {
        if !def.dyn_elems.contains(&e.elem) {
            continue;
        }
        let row_ref = if is_root {
            format!("r_{}_row", e.elem)
        } else {
            format!("{prefix}_{}_rows[{index}]", e.elem)
        };
        let (px, py, pz) = if is_root {
            (
                format!("r_{}_px", e.elem),
                format!("r_{}_py", e.elem),
                format!("r_{}_pz", e.elem),
            )
        } else {
            (
                format!("{prefix}_{}_px[{index}]", e.elem),
                format!("{prefix}_{}_py[{index}]", e.elem),
                format!("{prefix}_{}_pz[{index}]", e.elem),
            )
        };
        let props = elem_props_3d(project, def, &e.elem)?;
        let es = prop_exprs_3d(prop_of(&props, "position").expect("position"));
        let x_text = expr_text_3d(&es[0], world, def, is_root, index)?;
        let y_text = expr_text_3d(&es[1], world, def, is_root, index)?;
        let z_text = expr_text_3d(&es[2], world, def, is_root, index)?;
        src.push_str(&format!(
            " if ((((({px}-{x_text})*({px}-{x_text}))+(({py}-{y_text})*({py}-{y_text})))+(({pz}-{z_text})*({pz}-{z_text})))>0.0) {{\n  {px}={x_text}\n  {py}={y_text}\n  {pz}={z_text}\n  ag_set_local({row_ref},0,{x_text})\n  ag_set_local({row_ref},1,{y_text})\n  ag_set_local({row_ref},2,{z_text})\n }}\n"
        ));
        if e.dyn_vis {
            let vexpr = prop_of(&props, "visible")
                .map(|p| prop_exprs_3d(p)[0].clone())
                .ok_or_else(|| at(&def.module, def.line, "visible", "missing visible"))?;
            let v_text = expr_text_3d(&vexpr, world, def, is_root, index)?;
            let vis = if is_root {
                format!("r_{}_vis", e.elem)
            } else {
                format!("{prefix}_{}_vis[{index}]", e.elem)
            };
            src.push_str(&format!(
                " if {vis}!={v_text} {{\n  {vis}={v_text}\n  ag_set_visible({row_ref},{v_text})\n }}\n"
            ));
        }
    }
    src.push_str("}\n");
    Ok(())
}

/// 呈现表达式文本（`.field` → SoA；参数 → 内联常量；字面量逐字）。
fn expr_text_3d(
    e: &Expr,
    world: &WorldIR,
    def: &DefIR,
    is_root: bool,
    index: &str,
) -> Result<String, LowerError> {
    if let Some(v) = numeric_lit(e) {
        return Ok(fmt_f(v));
    }
    if let Expr::Bool(b) = e {
        return Ok(if *b { "1" } else { "0" }.to_string());
    }
    if let Expr::Dot(inner, name) = e {
        if matches!(&**inner, Expr::Ident(b) if b == "self") {
            if def.model.iter().any(|(n, _)| n == name.as_str()) {
                return Ok(if is_root {
                    format!("r_{name}")
                } else {
                    format!("{}_{name}[{index}]", def.prefix())
                });
            }
            return Err(at(
                &def.module,
                def.line,
                name,
                "present expression references unknown member",
            ));
        }
    }
    if let Expr::Ident(p) = e {
        if def.params.iter().any(|(n, _)| n == p.as_str()) {
            // 参数是下降期常量：逐实例同值，取首实例内联。
            if let Some(inst) = world.instances.iter().find(|i| i.def == def.name) {
                if let Some(crate::lower_components::ParamVal::Num(v)) = inst
                    .params
                    .iter()
                    .find(|(n, _)| n == p.as_str())
                    .map(|(_, v)| v)
                {
                    return Ok(fmt_f(*v));
                }
            }
            return Err(at(
                &def.module,
                def.line,
                p,
                "present expression param binding missing",
            ));
        }
    }
    if let Expr::Bina(l, op, r) = e {
        let lt = expr_text_3d(l, world, def, is_root, index)?;
        let rt = expr_text_3d(r, world, def, is_root, index)?;
        let op_text = match op {
            auto_parser_api::ast::Op::Add => "+",
            auto_parser_api::ast::Op::Sub => "-",
            auto_parser_api::ast::Op::Mul => "*",
            auto_parser_api::ast::Op::Div => "/",
            auto_parser_api::ast::Op::Lt => "<",
            auto_parser_api::ast::Op::Gt => ">",
            auto_parser_api::ast::Op::Le => "<=",
            auto_parser_api::ast::Op::Ge => ">=",
            auto_parser_api::ast::Op::Eq => "==",
            auto_parser_api::ast::Op::Neq => "!=",
            other => {
                return Err(at(
                    &def.module,
                    def.line,
                    "expr",
                    &format!("unsupported present operator {other:?}"),
                ))
            }
        };
        return Ok(format!("(({lt}) {op_text} ({rt}))"));
    }
    Err(at(
        &def.module,
        def.line,
        &format!("{e:?}"),
        "present expression must be a literal, param or '.field' model ref",
    ))
}

fn elem_props_3d(project: &Project, def: &DefIR, elem: &str) -> Result<Vec<Prop>, LowerError> {
    let decl = node_of(project, &def.name);
    let view = decl.view3d.as_ref().expect("view-3d");
    fn find<'a>(elements: &'a [View3dElement], elem: &str) -> Option<Vec<Prop>> {
        for e in elements {
            match e {
                View3dElement::Primitive { name, props } | View3dElement::Model { name, props }
                    if name == elem =>
                {
                    return Some(props.clone())
                }
                View3dElement::Group { children, .. } => {
                    if let Some(p) = find(children, elem) {
                        return Some(p);
                    }
                }
                _ => {}
            }
        }
        None
    }
    find(&view.elements, elem)
        .ok_or_else(|| at(&def.module, def.line, elem, "view-3d props missing"))
}

/// 成员接收 fn（020 同型 + space3 环境 + PLAN-027 runtime 环境）。
#[allow(clippy::too_many_arguments)]
/// PLAN-030：引用参数环境盖章 + msg 参数投递覆盖（life/recv ctx 共用；
/// None = 旧档不触碰，字节不变）。
pub(crate) fn apply_bindings_ctx<'a>(
    ctx: &mut crate::translate::Ctx<'a>,
    dname: &str,
    def_module: &str,
    project: &Project,
    plan: &'a Plan,
    bindings: Option<&'a crate::lower_components3d_bindings::BindingsCfg3>,
    index: &'static str,
) {
    crate::lower_components3d_bindings::stamp_bindings_env(ctx, dname, project, bindings);
    let Some(b) = bindings else { return };
    let Some(pmap) = b.refs.get(dname) else { return };
    let mtext: BTreeMap<String, String> = pmap
        .iter()
        .filter_map(|(p, rb)| match rb {
            crate::lower_components3d_bindings::RefBinding::MsgEvent { call } => {
                Some((p.clone(), format!("{call}()")))
            }
            _ => None,
        })
        .collect();
    if mtext.is_empty() {
        return;
    }
    // `emit .<msg参数>` → 绑定根事件 recv fn 直呼（017 血统）；其余祖先
    // 投递沿原表（字节一致）。
    let edef = dname.to_string();
    let errmod = def_module.to_string();
    let anc = &plan.anc;
    ctx.emit_hook = Box::new(move |ev: &str| -> Result<String, LowerError> {
        if let Some(t) = mtext.get(ev) {
            return Ok(t.clone());
        }
        match anc.get(&(edef.clone(), ev.to_string())) {
            Some(d) => {
                if d.to_root {
                    Ok(format!("{}()", d.fn_name))
                } else {
                    Ok(format!("{}({}[{}])", d.fn_name, d.table, index))
                }
            }
            None => Err(at(
                &errmod,
                0,
                ev,
                &format!("emit .{ev} has no lowering-time receiver"),
            )),
        }
    });
}

fn gen_recv_fn_3d(
    project: &Project,
    world: &WorldIR,
    plan: &Plan,
    fname: &str,
    recv_def_name: &str,
    ev: &str,
    refs: &BTreeMap<String, String>,
    self_row: Option<&str>,
    runtime: Option<&RuntimeCfg3>,
    events: Option<&crate::lower_components3d_events::EventsCfg3>,
    bindings: Option<&crate::lower_components3d_bindings::BindingsCfg3>,
    src: &mut String,
) -> Result<(), LowerError> {
    let def = if recv_def_name.is_empty() {
        &world.defs[&world.root]
    } else {
        &world.defs[recv_def_name]
    };
    let decl = node_of(project, &def.name);
    let on = decl
        .on
        .as_ref()
        .ok_or_else(|| at(&def.module, def.line, ev, "receiver has no on block"))?;
    let h = on
        .handlers
        .iter()
        .find(|h| matches!(&h.pattern, EventPattern::Member { name, .. } if name == ev))
        .ok_or_else(|| at(&def.module, def.line, ev, "receiver handler missing"))?;
    let param_names = match &h.pattern {
        EventPattern::Member { params, .. } => params.clone(),
        _ => Vec::new(),
    };
    let payload = def.msgs[ev].1.clone();
    let index = if recv_def_name.is_empty() { "" } else { "j" };
    let mut sig = String::new();
    if !index.is_empty() {
        sig.push_str("j int");
    }
    for (k, pt) in payload.iter().enumerate() {
        if !sig.is_empty() {
            sig.push_str(", ");
        }
        sig.push_str(&format!("{} {}", param_names[k], pt.text()));
    }
    src.push_str(&format!("fn {fname}({sig}) {{\n"));
    let mut ctx = if index.is_empty() {
        root_ctx(project, world, plan, None, None, None)
    } else {
        instance_ctx(project, plan, def, &def.prefix(), "j", None, None, None)?
    };
    ctx.space3 = Some(SpaceEnv {
        refs,
        self_row: self_row.map(|s| s.to_string()),
    });
    stamp_runtime_env(
        &mut ctx,
        if recv_def_name.is_empty() { &world.root } else { recv_def_name },
        runtime,
        events,
    );
    apply_bindings_ctx(
        &mut ctx,
        if recv_def_name.is_empty() { &world.root } else { recv_def_name },
        &def.module,
        project,
        plan,
        bindings,
        if index.is_empty() { "" } else { "j" },
    );
    translate_handler(&h.body.stmts, &ctx, src, 1)?;
    src.push_str("}\n");
    Ok(())
}

// ------------------------------------------------------------------- json --

fn obj_json_3d(r: &Row3, id: u64) -> String {
    let parent = r
        .parent
        .map(|p| p.to_string())
        .unwrap_or_else(|| "null".to_string());
    let resource = match &r.resource {
        None => "null".to_string(),
        Some(s) => format!("\"{s}\""),
    };
    format!(
        "  {{\n   \"id\": {id},\n   \"name\": \"{}\",\n   \"kind\": \"{}\",\n   \"translation\": [\n    {},\n    {},\n    {}\n   ],\n   \"scale\": [\n    {},\n    {},\n    {}\n   ],\n   \"yaw\": {},\n   \"pitch\": {},\n   \"rgba\": [\n    0.9,\n    0.3,\n    0.2,\n    1.0\n   ],\n   \"visible\": {},\n   \"parent\": {},\n   \"resource\": {},\n   \"sprite\": null\n  }}",
        r.name,
        r.kind,
        fmt_f(r.translation[0]),
        fmt_f(r.translation[1]),
        fmt_f(r.translation[2]),
        fmt_f(r.scale[0]),
        fmt_f(r.scale[1]),
        fmt_f(r.scale[2]),
        fmt_f(r.yaw),
        fmt_f(r.pitch),
        r.visible,
        parent,
        resource
    )
}

fn group_json_3d(r: &Row3, id: u64) -> String {
    let parent = r
        .parent
        .map(|p| p.to_string())
        .unwrap_or_else(|| "null".to_string());
    format!(
        "  {{\n   \"id\": {id},\n   \"name\": \"{}\",\n   \"dimension\": \"3d\",\n   \"parent\": {},\n   \"translation\": [\n    {},\n    {},\n    {}\n   ],\n   \"visible\": {}\n  }}",
        r.name,
        parent,
        fmt_f(r.translation[0]),
        fmt_f(r.translation[1]),
        fmt_f(r.translation[2]),
        r.visible
    )
}

pub(crate) fn scene_json_3d(
    behavior_api: &str,
    doc_prefix: &str,
    entry: &str,
    scene: &SceneBuild3,
    has_behavior: bool,
    sha: &str,
    bytes: usize,
    assets: &[(String, usize, &str)],
) -> String {
    let groups = scene
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.is_group)
        .map(|(i, r)| group_json_3d(r, i as u64 + 1))
        .collect::<Vec<_>>()
        .join(",\n");
    let objects = scene
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.is_group)
        .map(|(i, r)| obj_json_3d(r, i as u64 + 1))
        .collect::<Vec<_>>()
        .join(",\n");
    let asset_rows = assets
        .iter()
        .map(|(sha, len, name)| {
            format!(
                "  {{\n   \"format\": \"glb\",\n   \"sha256\": \"{sha}\",\n   \"byte_length\": {len},\n   \"name\": \"{name}\"\n  }}"
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let behavior = if has_behavior {
        format!(
            "{{\n  \"api\": \"{behavior_api}\",\n  \"source\": {{\n   \"format\": \"auto-source\",\n   \"sha256\": \"{sha}\",\n   \"byte_length\": {bytes}\n  }},\n  \"actors\": {{\n   \"two_d\": null,\n   \"three_d\": null\n  }}\n }}"
        )
    } else {
        "null".to_string()
    };
    format!(
        "{{\n \"behavior\": {behavior},\n \"schema\": \"autographics.scene/6\",\n \"document_id\": \"{doc_prefix}-{entry}\",\n \"resources\": [\n  \"circle\",\n  \"quad\",\n  \"cylinder\",\n  \"player\",\n  \"mob\"\n ],\n \"groups\": [\n{groups}\n ],\n \"objects\": [\n{objects}\n ],\n \"assets\": [\n{asset_rows}\n ]\n}}\n"
    )
}

pub(crate) fn roles_json_3d(scene: &SceneBuild3) -> String {
    let rows = scene
        .handles
        .iter()
        .map(|(k, v)| format!("\n  \"{k}\": {v}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\n \"scene_rows\": {{{rows}\n }}\n}}")
}

/// components-3d 降低入口：共享 IR 世界 → 投递计划 → 场景 → v1 用户源。
pub fn lower_project_components3d(project: &Project) -> Result<LowerOutputBasic2D, LowerError> {
    lower_project_components3d_cfg(project, Cfg3 {
        runtime: None,
        events: None,
        bindings: None,
        doc_prefix: "plan024-components-3d",
        gen_note: "PLAN-024 components-3d",
        regen_note: "tools/verify_plan024.py",
        behavior_api: API,
    })
}

/// PLAN-027：共享降低管线（024 唯一实现 + runtime 注入位）。
pub(crate) struct Cfg3<'c> {
    /// runtime 注入配置（None = 024 路径，字节不变）。
    pub runtime: Option<&'c RuntimeCfg3>,
    /// PLAN-028：components-3d-events 配置（None = 旧档，字节不变）。
    pub events: Option<&'c crate::lower_components3d_events::EventsCfg3>,
    /// PLAN-030：components-3d-bindings 引用参数配置（None = 旧档，字节
    /// 不变；Some = 池常量/actor 主题/msg 投递/引用 SoA 列随档发行）。
    pub bindings: Option<&'c crate::lower_components3d_bindings::BindingsCfg3>,
    /// scene.agscene document_id 前缀。
    pub doc_prefix: &'static str,
    /// 生成头注释身份段。
    pub gen_note: &'static str,
    /// 生成头再生成门注释（verifier 脚本名）。
    pub regen_note: &'static str,
    /// scene.agscene behavior.api（024 = autographics.behavior/1；runtime =
    /// character3d/1 或 runtime_camera3d/1，驱动 start_selected_play 据此装配）。
    pub behavior_api: &'static str,
}

pub(crate) fn lower_project_components3d_cfg(
    project: &Project,
    cfg: Cfg3<'_>,
) -> Result<LowerOutputBasic2D, LowerError> {
    lower_project_components3d_cfg_with(project, cfg, false)
}

/// PLAN-030：refs 变体（bindings 档世界构建接受名义引用参数；旧档 false
/// 字节不变）。
pub(crate) fn lower_project_components3d_cfg_with(
    project: &Project,
    cfg: Cfg3<'_>,
    refs: bool,
) -> Result<LowerOutputBasic2D, LowerError> {
    let w3 = build_world_3d_with(project, false, refs)?;
    let plan = crate::lower_components::plan_dispatch_events(
        project,
        &w3.world,
        false,
        false,
        false,
        cfg.events.is_some(),
        cfg.bindings.is_some(),
    )?;
    // PLAN-028：events 档模板行随 cfg 进共享管线（旧档空切片，字节不变）。
    let no_templates: Vec<TemplateRowIn> = Vec::new();
    let templates: &[TemplateRowIn] = match cfg.events {
        Some(ev) => &ev.template_rows,
        None => &no_templates,
    };
    let scene = collect_scene_3d_with(project, &w3, templates)?;
    let user_source = gen_user_source_3d(project, &w3, &plan, &scene, cfg.runtime, cfg.events, cfg.bindings, cfg.gen_note, cfg.regen_note)?;
    let sha = crate::scene::sha256_hex(user_source.as_bytes());
    let bytes = user_source.len();
    let has_behavior =
        w3.world.stateful > 0 || w3.world.defs.values().any(|d| d.life_start || d.life_tick);
    let entry = w3.world.root_module.clone();
    let mut assets: Vec<(String, usize, String)> = Vec::new();
    for (name, (sha256, len)) in &w3.world.models {
        assets.push((sha256.clone(), *len, name.clone()));
    }
    assets.sort();
    assets.dedup();
    let asset_refs: Vec<(String, usize, &str)> = assets
        .iter()
        .map(|(s, l, n)| (s.clone(), *l, n.as_str()))
        .collect();
    let scene_json = scene_json_3d(
        cfg.behavior_api,
        cfg.doc_prefix,
        &entry,
        &scene,
        has_behavior,
        &sha,
        bytes,
        &asset_refs,
    );
    let roles_json = roles_json_3d(&scene);
    Ok(LowerOutputBasic2D {
        user_source: if has_behavior {
            Some(user_source)
        } else {
            None
        },
        scene_json,
        source_sha256: sha,
        source_bytes: bytes,
        roles_json,
    })
}
