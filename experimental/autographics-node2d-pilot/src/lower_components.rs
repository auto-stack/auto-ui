//! PLAN-020 components-2d profile（T-03 世界 IR + T-04 生成）。
//!
//! 有状态 Node 组件降层，全部落在 T-02 实测冻结的两套既有机制上：
//! (1) 每定义 SoA 数组 + 按实例索引 handler（PLAN-017 形态：
//!     model→`<prefix>_<field>[i]`、参数只读数组、`<prefix>_lifetick(i,dt)`
//!     声明先序父先子后、根标量 `r_<field>`）；
//! (2) 下降期绑定表同步直呼（017 emit_hook 血统：`on .Ev(p)` →
//!     `fn <name>_<ev>(j, p..)`、`emit <child>.Ev(args)` →
//!     `<name>_<ev>(<child>_bind_<EV>[i], args..)`、`emit .Ev(args)` →
//!     最近祖先接收方 `<prefix>_emit_<EV>[i]`，无宿主队列）。
//! 零 parser/解释器复制：node2d dialect 已解析 model/msg/on/emit/instances。
//!
//! 冻结观察序（每 tick）：宿主采样输入 → 声明先序每实例：当前参数 →
//! `Life.Tick`（emit 同 tick 内联直达 receiver）→ 绑定视觉同步
//! （`<prefix>_present(i)`，tick 末；位置/可见按变更计费）→ 根先行。
//! 运行服务保持 T-02 冻结的 objects2d/1 装配（character + camera +
//! objects）；必须声明恰一 camera2d（入口 `_ag_cam2_freeze` 缺相机即
//! fault 27，T-02 实测）。
//!
//! 生成门只使用「原始比较」或「纯算术平方差 + 一次关系比较」（present 的
//! 变更检测）。实测禁用形态：比较结果的和再关系化
//! `((a!=b)+(c!=d))>0` / `((a==b)+(c==d))<2` 在钉死 VM 上恒真 → 每 tick
//! 重计费越 32 池（T-04 实测，变体对照 E3-VG 隔离）。另注：VM 的 `/` 是
//! 实除，作者的偶数 tick 门须用整型翻转（fixture parity 字段），不能写
//! `ticks/2*2==ticks`。
//!
//! 具名拒绝（V-01）：每条诊断带 `module <m>, line <n>, symbol '<s>'`；
//! 非法图（实例环/重复路径/缺 use/错参数元数/未知端口/未知 helper/第 9
//! 有状态实例/缺相机）执行前拒绝，不 panic。
//!
//! PLAN-023 F-8 修复（T-02 实测缺陷）：
//! - 呈现缓存改名 `<prefix>_visq`：020 以 `<prefix>_vis` 命名可见性缓存，
//!   与作者 model 字段 `vis` 的 SoA 数组重名 → `var zone_vis=[..]` 声明
//!   两次，admission 拒绝（T-02 探针手改 `zone_visq` 复现）。缓存统一
//!   改为 `_visq`，并对与生成缓存符号撞名的 model 字段（`handle`/`px`/
//!   `py`/`visq` 后缀）给出具名拒绝；020 冻结产物字节不受影响（绿色
//!   夹具无一携带撞名字段——携带即 admission 致命，不可能为绿色）。
//! - 区域规则（F-6 region rule）：goal/hazard 触发区必须是运行时实例
//!   （带 model 绑定 shape 的组件 → 模板行 + ag_object_spawn），不能是
//!   静态场景叶。实测依据：frozen 角色扫移（behavior_motion_v5.at 的
//!   ag_move_and_slide）把每个场景叶当碰撞候选（仅跳过 self/group/其他
//!   维度/隐藏），行走角色要么被叶挡住（status 3）要么停在 clearance
//!   (0.002) 之上——闭区间触发对被触及的静态叶永不进入。运行时实例
//!   非固体（合同），角色可自由重叠：触发区只能取实例形态。降层面：
//!   触发目标实例的定义若无动态 shape（无 spawn/无 handle）→ 具名拒绝
//!   "region rule: trigger regions must be runtime instances, not static
//!   scene leaves"。
//! - CPU 会话空间盒约定（F-8.3）：CPU 会话 `_ag_spatial_config` 按宿主
//!   Descriptor 行（spatial.rs）同型供盒 —— **平移相对半边长**
//!   (translation-relative half-extents: [-sx/2, -sy/2, -sz/2, +sx/2,
//!   +sy/2, +sz/2])，平移由 `_ag_append_field` 的行载荷单独携带。绝对盒
//!   （min/max 世界坐标）会把平移重复计入、静默平移每个触发/几何——
//!   020 probe_session 曾以绝对盒供参（只影响 CPU 侧触发测试，GPU 门
//!   不受影响）；T-02 探针与 PLAN-023 t03 门均按相对半边长供参。
use auto_parser_api::ast::{Expr, Stmt, Type};
use auto_ui_dialect::node2d::{EventPattern, NodeDecl, Prop, PropValue, View2dElement};

use crate::load::{node_of, Project};
use crate::lower::numeric_lit;
use crate::lower_basic2d::{
    fmt_f, life_handler, obj_json, resource_color, resource_mesh, roles_json_basic, CameraCfg,
    Collected, LowerOutputBasic2D, SceneNode,
};
use crate::scene::sha256_hex;
use crate::translate::{
    query_call_kind, translate_handler, translate_stmts, Ctx, LowerError, QueryEnv, QueryKind,
    RefRow, Target,
};
use std::collections::{BTreeMap, BTreeSet};

// ------------------------------------------------------------ diagnose ----

pub(crate) fn at(module: &str, line: usize, symbol: &str, what: &str) -> LowerError {
    LowerError::Unsupported {
        what: format!("{what} (module {module}, line {line}, symbol '{symbol}')"),
        line,
    }
}

pub(crate) fn line_at(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        + 1
}

pub(crate) fn decl_line(project: &Project, module: &str, decl: &NodeDecl) -> usize {
    project
        .modules
        .get(module)
        .map(|m| line_at(&m.source, decl.span.0))
        .unwrap_or(0)
}

pub(crate) fn prop_line(module_src: &str, pr: &Prop) -> usize {
    line_at(module_src, pr.span.0)
}

// ------------------------------------------------------------------ IR ----

/// 参数/载荷类型面（组件合同：int/float 数值 + Material2D/Mesh2D/Image2D
/// 资源端口——Image2D 为 PLAN-022 sprites-2d 增补）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PType {
    Int,
    Float,
    Res,
    /// PLAN-022：Image2D（typed 图像端口；绑定必须是 image2d.png 资源）。
    Img,
    /// PLAN-024：Model3D（typed 静态平色 GLB 端口；绑定必须是 model3d.glb
    /// 资源）。仅 components-3d 构造；2D 档 ptype_of 不产生。
    Model,
}

fn ptype_of(ty: &Type) -> Option<PType> {
    match ty {
        Type::Int | Type::Byte => Some(PType::Int),
        Type::Float | Type::Double => Some(PType::Float),
        Type::User(d) if d.name == "Material2D" || d.name == "Mesh2D" => Some(PType::Res),
        Type::User(d) if d.name == "Image2D" => Some(PType::Img),
        _ => None,
    }
}

/// PLAN-024：components-3d 参数类型面（int/float/Model3D；Material2D/
/// Mesh2D/Image2D 端口具名拒绝）。
pub(crate) fn ptype_of_3d(ty: &Type) -> Option<PType> {
    match ty {
        Type::Int | Type::Byte => Some(PType::Int),
        Type::Float | Type::Double => Some(PType::Float),
        Type::User(d) if d.name == "Model3D" => Some(PType::Model),
        _ => None,
    }
}

impl PType {
    pub(crate) fn text(self) -> &'static str {
        match self {
            PType::Int | PType::Res | PType::Img | PType::Model => "int",
            PType::Float => "float",
        }
    }
}

/// 实例参数绑定值：数值或资源 symbol（下降期常量）。
#[derive(Debug, Clone)]
pub(crate) enum ParamVal {
    Num(f64),
    Res(String),
}

impl ParamVal {
    pub(crate) fn text(&self, is_int: bool) -> String {
        match self {
            ParamVal::Num(v) => {
                if is_int {
                    format!("{}", *v as i64)
                } else {
                    fmt_f(*v)
                }
            }
            ParamVal::Res(r) => r.clone(),
        }
    }
}

/// PLAN-022：视图图像绑定上的 frame/flip 值形态（字面量序数/参数/model 引用）。
#[derive(Debug, Clone)]
pub(crate) enum FrameBind {
    /// 字面量：frame = 帧表 ordinal；flip = 0/1。
    Lit(u32),
    /// 数值参数（逐实例下降期常量）。
    Param(String),
    /// `.field`（本定义 model 字段；运行期逐 tick 值）。
    Field(String),
}

/// PLAN-022：一个图像绑定视图元素（shape2d 或 character2d）。
#[derive(Debug, Clone)]
pub(crate) struct SpriteElemIR {
    pub(crate) elem: String,
    /// character2d 行（帧写走 ag_sprite_frame(行 id)）还是 shape2d。
    pub(crate) is_char: bool,
    /// image2d.png 资源名（Direct 形态；Param 形态时为空）。
    pub(crate) image: String,
    /// Image2D 参数名（`image: <param>` 传参形态；逐实例解析）。
    pub(crate) image_param: Option<String>,
    pub(crate) frame: FrameBind,
    pub(crate) flip: FrameBind,
    pub(crate) line: usize,
}

/// 一个组件定义（root 或被实例化 node）的静态面。
pub(crate) struct DefIR {
    pub(crate) name: String,
    pub(crate) module: String,
    pub(crate) line: usize,
    /// 声明参数（序保持）：名字 → 类型面。
    pub(crate) params: Vec<(String, PType)>,
    /// model 字段（序保持）：名字 → 是否 int。
    pub(crate) model: Vec<(String, bool)>,
    /// msg 事件：名字 → (payload 个数, payload 类型面)。
    pub(crate) msgs: BTreeMap<String, (usize, Vec<PType>)>,
    /// `on .Ev` 成员处理器：事件名 → (形参个数, 行)。
    pub(crate) members: BTreeMap<String, (usize, usize)>,
    pub(crate) life_start: bool,
    pub(crate) life_tick: bool,
    /// view 内绑定本定义 model 字段的 shape2d 元素（动态呈现，序保持）。
    pub(crate) dyn_elems: Vec<String>,
    /// 静态 shape2d/solid2d 元素名（序保持）。
    pub(crate) static_elems: Vec<String>,
    /// character2d 元素（元素名, 行）。
    pub(crate) char_elem: Option<(String, usize)>,
    /// PLAN-022（sprites-2d）：图像绑定元素（序保持；与 dyn/static 互斥）。
    pub(crate) sprite_elems: Vec<SpriteElemIR>,
    /// 定义是否带状态面（model/msg/on）→ 实例计入 8 包络。
    pub(crate) stateful: bool,
}

impl DefIR {
    pub(crate) fn prefix(&self) -> String {
        self.name.to_lowercase()
    }
}

/// 一个实例（root 之外）：声明先序，def_idx 即其定义 SoA 下标。
pub(crate) struct InstanceIR {
    pub(crate) path: String,
    pub(crate) def: String,
    pub(crate) parent: Option<usize>,
    pub(crate) def_idx: usize,
    /// 参数名 → 下降期常量（数值或资源 symbol）。
    pub(crate) params: Vec<(String, ParamVal)>,
    pub(crate) line: usize,
    /// PLAN-024：实例化点所在的显式 group3d 名（caller 视图内；根层
    /// 直接实例为 None）。2D 档恒 None。
    pub(crate) parent_group: Option<String>,
}

/// 组件世界 IR（验证后的中间表示）。
pub(crate) struct WorldIR {
    pub(crate) root: String,
    pub(crate) root_module: String,
    pub(crate) defs: BTreeMap<String, DefIR>,
    pub(crate) instances: Vec<InstanceIR>,
    /// 有状态实例总数（含 root），≤8（016 有限实例包络的组件合同）。
    pub(crate) stateful: usize,
    /// 根 view 的 solid2d（序保持）与 camera2d（≤1）。
    pub(crate) solids: Vec<String>,
    pub(crate) camera: Option<Vec<Prop>>,
    /// PLAN-022：工程图像资源（声明序）：名字 → (sha256, byte_length, w, h)。
    pub(crate) images: BTreeMap<String, (String, usize, u32, u32)>,
    /// PLAN-022：帧表（图像名 → [(帧资源名, rect)]，声明序；ordinal 即序）。
    pub(crate) frames: BTreeMap<String, Vec<(String, [u32; 4])>>,
    /// PLAN-024：工程 GLB 模型资源（声明序）：名字 → (sha256, byte_length)。
    pub(crate) models: BTreeMap<String, (String, usize)>,
}

pub(crate) fn inst_name_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

impl WorldIR {
    pub(crate) fn instances_of(&self, def: &str) -> Vec<usize> {
        self.instances
            .iter()
            .enumerate()
            .filter(|(_, i)| i.def == def)
            .map(|(k, _)| k)
            .collect()
    }

    pub(crate) fn def_len(&self, def: &str) -> usize {
        self.instances.iter().filter(|i| i.def == def).count()
    }

    fn children_of(&self, inst: usize, name: &str) -> Option<usize> {
        self.instances
            .iter()
            .position(|i| i.parent == Some(inst) && inst_name_of(&i.path) == name)
    }

    pub(crate) fn param_value(
        &self,
        def: &DefIR,
        inst: &InstanceIR,
        pname: &str,
    ) -> Result<f64, LowerError> {
        let (_, val) = inst
            .params
            .iter()
            .find(|(n, _)| n == pname)
            .ok_or_else(|| {
                at(
                    &def.module,
                    inst.line,
                    pname,
                    &format!("instance '{}' param '{pname}' missing", inst.path),
                )
            })?;
        match val {
            ParamVal::Num(v) => Ok(*v),
            ParamVal::Res(_) => Err(at(
                &def.module,
                inst.line,
                pname,
                "resource param used as a numeric value",
            )),
        }
    }
}

// ------------------------------------------------------ def scanning ----

pub(crate) fn model_field_is_int(
    project: &Project,
    module: &str,
    line: usize,
    params: &[(String, PType)],
    f: &auto_ui_dialect::ModelField,
) -> Result<bool, LowerError> {
    match &f.init {
        Expr::Int(_) => Ok(true),
        Expr::Float(..) => Ok(false),
        Expr::Unary(auto_parser_api::ast::Op::Sub, inner) => Ok(matches!(&**inner, Expr::Int(_))),
        Expr::Ident(p) => param_int(project, module, line, params, &f.name.to_string(), p),
        Expr::Dot(inner, p) => {
            if matches!(&**inner, Expr::Ident(b) if b == "self") {
                param_int(project, module, line, params, &f.name.to_string(), p)
            } else {
                Err(at(
                    module,
                    line,
                    &f.name.to_string(),
                    "model init path must be '.param'",
                ))
            }
        }
        other => Err(at(
            module,
            line,
            &f.name.to_string(),
            &format!("model init must be numeric literal or '.param', got {other:?}"),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn param_int(
    _project: &Project,
    module: &str,
    line: usize,
    params: &[(String, PType)],
    field: &str,
    pname: &auto_parser_api::ast::Name,
) -> Result<bool, LowerError> {
    let pt = params
        .iter()
        .find(|(n, _)| n == pname.as_str())
        .map(|(_, t)| *t)
        .ok_or_else(|| {
            at(
                module,
                line,
                field,
                &format!("model init references unknown param '{pname}'"),
            )
        })?;
    if pt == PType::Res || pt == PType::Img {
        return Err(at(
            module,
            line,
            field,
            &format!("model init references resource param '{pname}'"),
        ));
    }
    Ok(pt == PType::Int)
}

/// PLAN-022：视图 image 绑定的引用形态（直连 resources.X 或 Image2D 参数）。
#[derive(Debug, Clone)]
enum ImageRef {
    Direct(String),
    Param(String),
}

/// 扫描一个定义的静态面（root 或被实例化 node）。
fn build_def(
    project: &Project,
    name: &str,
    is_root: bool,
    sprites: bool,
) -> Result<DefIR, LowerError> {
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
        let pt = ptype_of(ty).ok_or_else(|| {
            at(
                &module,
                line,
                pname,
                "param has unsupported type (components-2d: int/float/Material2D/Mesh2D)",
            )
        })?;
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
                let pt = ptype_of(t).ok_or_else(|| {
                    at(
                        &module,
                        line,
                        &v.name.to_string(),
                        "msg payload type unsupported (components-2d: int/float)",
                    )
                })?;
                if pt == PType::Res {
                    return Err(at(
                        &module,
                        line,
                        &v.name.to_string(),
                        "msg payload must be numeric (components-2d)",
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
                            "unsupported Life event (components-2d: Life.Start/Life.Tick)",
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

    let mut def = DefIR {
        name: name.to_string(),
        module: module.clone(),
        line,
        params,
        model,
        msgs,
        members,
        life_start,
        life_tick,
        dyn_elems: Vec::new(),
        static_elems: Vec::new(),
        char_elem: None,
        sprite_elems: Vec::new(),
        stateful: decl.model.is_some() || decl.msg.is_some() || decl.on.is_some(),
    };

    // PLAN-022：图像绑定 prop 解析（sprites-2d 档；旧档不走此路径）。
    let parse_image_ref = |pr: &Prop| -> Result<ImageRef, LowerError> {
        match &pr.value {
            PropValue::One(Expr::Dot(outer, r)) if matches!(&**outer, Expr::Ident(b) if b == "resources") => {
                Ok(ImageRef::Direct(r.to_string()))
            }
            PropValue::One(Expr::Dot(outer, p)) if matches!(&**outer, Expr::Ident(b) if b == "self") => {
                Ok(ImageRef::Param(p.to_string()))
            }
            PropValue::One(Expr::Ident(p)) => Ok(ImageRef::Param(p.to_string())),
            other => Err(at(
                &module,
                line,
                &pr.name,
                &format!("image prop must be resources.<name> or an Image2D param, got {other:?}"),
            )),
        }
    };
    let parse_frame_bind = |pr: &Prop| -> Result<FrameBind, LowerError> {
        match &pr.value {
            PropValue::One(Expr::Int(v)) => Ok(FrameBind::Lit(*v as u32)),
            PropValue::One(Expr::Dot(outer, p)) if matches!(&**outer, Expr::Ident(b) if b == "self") => {
                Ok(FrameBind::Field(p.to_string()))
            }
            PropValue::One(Expr::Ident(p)) => Ok(FrameBind::Param(p.to_string())),
            other => Err(at(
                &module,
                line,
                &pr.name,
                &format!(
                    "frame/flip must be an int literal, param or '.field' model ref, got {other:?}"
                ),
            )),
        }
    };

    let view = decl
        .view2d
        .as_ref()
        .ok_or_else(|| at(&module, line, name, "component definition has no view-2d"))?;
    let model_fields: BTreeSet<String> = def.model.iter().map(|(n, _)| n.clone()).collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for el in &view.elements {
        match el {
            View2dElement::Shape { name, props } => {
                if !seen.insert(name.clone()) {
                    return Err(at(&module, line, name, "duplicate view element name"));
                }
                // PLAN-022：image 绑定元素（sprites-2d 档）。
                if sprites {
                    if let Some(ip) = prop_of(props, "image") {
                        let image = parse_image_ref(ip)?;
                        let frame_pr = prop_of(props, "frame").ok_or_else(|| {
                            at(&module, line, name, "image element missing 'frame'")
                        })?;
                        let flip_pr = prop_of(props, "flip").ok_or_else(|| {
                            at(&module, line, name, "image element missing 'flip'")
                        })?;
                        let frame = parse_frame_bind(frame_pr)?;
                        let flip = parse_frame_bind(flip_pr)?;
                        if let FrameBind::Lit(v) = flip {
                            if v > 1 {
                                return Err(at(
                                    &module,
                                    line,
                                    name,
                                    &format!("literal flip must be 0/1, got {v}"),
                                ));
                            }
                        }
                        if prop_of(props, "position")
                            .map(|p| prop_binds_model(p, &model_fields))
                            .unwrap_or(false)
                        {
                            return Err(at(
                                &module,
                                line,
                                name,
                                "image element position must be static (move it through the character service or a dynamic non-image shape)",
                            ));
                        }
                        let known = [
                            "image", "frame", "flip", "position", "kind", "size", "visible",
                            "material",
                        ];
                        for pr in props {
                            if !known.contains(&pr.name.as_str()) {
                                return Err(at(
                                    &module,
                                    line,
                                    &pr.name,
                                    &format!(
                                        "unknown prop on image element '{name}' (known: {known:?})"
                                    ),
                                ));
                            }
                        }
                        def.sprite_elems.push(match image {
                            ImageRef::Direct(r) => SpriteElemIR {
                                elem: name.clone(),
                                is_char: false,
                                image: r,
                                image_param: None,
                                frame,
                                flip,
                                line,
                            },
                            ImageRef::Param(p) => SpriteElemIR {
                                elem: name.clone(),
                                is_char: false,
                                image: String::new(),
                                image_param: Some(p),
                                frame,
                                flip,
                                line,
                            },
                        });
                        continue;
                    }
                }
                if props.iter().any(|p| prop_binds_model(p, &model_fields)) {
                    def.dyn_elems.push(name.clone());
                } else {
                    def.static_elems.push(name.clone());
                }
            }
            View2dElement::Character { name, props } => {
                if !seen.insert(name.clone()) {
                    return Err(at(&module, line, name, "duplicate view element name"));
                }
                if def.char_elem.is_some() {
                    return Err(at(
                        &module,
                        line,
                        name,
                        "duplicate character2d (exactly one)",
                    ));
                }
                // PLAN-022：character2d 携带 image 绑定（frame/flip 可绑定
                // 本定义 model；position/size 仍静态）。
                if sprites {
                    if let Some(ip) = prop_of(props, "image") {
                        let image = parse_image_ref(ip)?;
                        let frame_pr = prop_of(props, "frame").ok_or_else(|| {
                            at(&module, line, name, "image element missing 'frame'")
                        })?;
                        let flip_pr = prop_of(props, "flip").ok_or_else(|| {
                            at(&module, line, name, "image element missing 'flip'")
                        })?;
                        let frame = parse_frame_bind(frame_pr)?;
                        let flip = parse_frame_bind(flip_pr)?;
                        if let FrameBind::Lit(v) = flip {
                            if v > 1 {
                                return Err(at(
                                    &module,
                                    line,
                                    name,
                                    &format!("literal flip must be 0/1, got {v}"),
                                ));
                            }
                        }
                        def.sprite_elems.push(match image {
                            ImageRef::Direct(r) => SpriteElemIR {
                                elem: name.clone(),
                                is_char: true,
                                image: r,
                                image_param: None,
                                frame,
                                flip,
                                line,
                            },
                            ImageRef::Param(p) => SpriteElemIR {
                                elem: name.clone(),
                                is_char: true,
                                image: String::new(),
                                image_param: Some(p),
                                frame,
                                flip,
                                line,
                            },
                        });
                        def.char_elem = Some((name.clone(), line));
                        continue;
                    }
                }
                if props.iter().any(|p| prop_binds_model(p, &model_fields)) {
                    return Err(at(
                        &module,
                        line,
                        name,
                        "character2d position/material must be static (the character service owns the run position)",
                    ));
                }
                def.char_elem = Some((name.clone(), line));
            }
            View2dElement::Solid { name, .. } => {
                if !is_root {
                    return Err(at(
                        &module,
                        line,
                        name,
                        "solid2d is root-view only in components-2d (static collision surface)",
                    ));
                }
                if !seen.insert(name.clone()) {
                    return Err(at(&module, line, name, "duplicate view element name"));
                }
                def.static_elems.push(name.clone());
            }
            View2dElement::Camera { name, .. } => {
                if !is_root {
                    return Err(at(
                        &module,
                        line,
                        name,
                        "camera2d is root-view only in components-2d",
                    ));
                }
                if !seen.insert(name.clone()) {
                    return Err(at(&module, line, name, "duplicate view element name"));
                }
            }
            View2dElement::Instance { name, .. } => {
                if !seen.insert(name.clone()) {
                    return Err(at(&module, line, name, "duplicate view element name"));
                }
            }
            View2dElement::Group { name, .. } => {
                return Err(at(
                    &module,
                    line,
                    name,
                    "group2d is out of the components-2d scope (flat root, per-instance shapes)",
                ))
            }
            View2dElement::Trigger { name, .. } => {
                return Err(at(
                    &module,
                    line,
                    name,
                    "trigger2d is out of the components-2d scope (g11-2d profile)",
                ))
            }
            View2dElement::If { .. } => {
                return Err(at(
                    &module,
                    line,
                    &def.name,
                    "if elements are out of the components-2d scope (visible=false keeps identity)",
                ))
            }
        }
    }
    Ok(def)
}

/// view 属性是否绑定本定义 model 字段（`.field`，前导点 self 成员）。
fn prop_binds_model(p: &Prop, model_fields: &BTreeSet<String>) -> bool {
    fn expr_bound(e: &Expr, model_fields: &BTreeSet<String>) -> bool {
        match e {
            Expr::Dot(inner, name) => {
                matches!(&**inner, Expr::Ident(b) if b == "self")
                    && model_fields.contains(name.as_str())
            }
            Expr::Bina(l, _, r) => expr_bound(l, model_fields) || expr_bound(r, model_fields),
            Expr::Tuple(items) => items.iter().any(|i| expr_bound(i, model_fields)),
            _ => false,
        }
    }
    match &p.value {
        PropValue::One(e) => expr_bound(e, model_fields),
        PropValue::Pair(es) => es.iter().any(|e| expr_bound(e, model_fields)),
    }
}

// ------------------------------------------------------ world build ----

pub(crate) fn model_init_value(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    field: &str,
    inst: &InstanceIR,
) -> Result<f64, LowerError> {
    let decl = node_of(project, &def.name);
    let model = decl.model.as_ref().ok_or_else(|| {
        at(
            &def.module,
            def.line,
            field,
            "definition has no model block",
        )
    })?;
    let f = model
        .fields
        .iter()
        .find(|f| f.name.as_str() == field)
        .ok_or_else(|| at(&def.module, def.line, field, "unknown model field"))?;
    match &f.init {
        Expr::Int(v) => Ok(*v as f64),
        Expr::Float(v, _) => Ok(*v),
        Expr::Unary(auto_parser_api::ast::Op::Sub, inner) => numeric_lit(inner)
            .map(|v| -v)
            .ok_or_else(|| at(&def.module, def.line, field, "model init must be numeric")),
        Expr::Ident(p) => world.param_value(def, inst, p.as_str()),
        Expr::Dot(inner, p) => {
            if matches!(&**inner, Expr::Ident(b) if b == "self") {
                world.param_value(def, inst, p.as_str())
            } else {
                Err(at(
                    &def.module,
                    def.line,
                    field,
                    "model init path must be '.param'",
                ))
            }
        }
        other => Err(at(
            &def.module,
            def.line,
            field,
            &format!("model init must be numeric literal or '.param', got {other:?}"),
        )),
    }
}

/// 构建组件世界 IR：root + 声明先序实例展开（嵌套/同定义多实例/环/包络）。
pub(crate) fn build_world(
    project: &Project,
    sprites: bool,
    camera_required: bool,
) -> Result<WorldIR, LowerError> {
    let root_name = project.root.clone();
    let root_module = project
        .node_modules
        .get(&root_name)
        .cloned()
        .ok_or_else(|| at("", 0, &root_name, "root node module missing"))?;
    let root_def = build_def(project, &root_name, true, sprites)?;

    let mut world = WorldIR {
        root: root_name.clone(),
        root_module: root_module.clone(),
        defs: BTreeMap::new(),
        instances: Vec::new(),
        stateful: 1, // root 计入
        solids: Vec::new(),
        camera: None,
        images: BTreeMap::new(),
        frames: BTreeMap::new(),
        models: BTreeMap::new(),
    };

    // PLAN-022：图像/帧表资源装配（sprites-2d 档）——声明序跨 resources
    // 模块收集；帧 rect 按图像 IHDR 尺寸边界具名校验（完整准入在 driver）。
    if sprites {
        let mut frame_names: BTreeSet<String> = BTreeSet::new();
        for module in project.modules.values() {
            for r in &module.resources {
                match &r.kind {
                    crate::load::ResourceKind::Image {
                        file,
                        sha256,
                        byte_length,
                        width,
                        height,
                    } => {
                        if world.images.contains_key(&r.name) {
                            return Err(at(
                                &module.name,
                                0,
                                &r.name,
                                "duplicate image resource name",
                            ));
                        }
                        world.images.insert(
                            r.name.clone(),
                            (sha256.clone(), *byte_length, *width, *height),
                        );
                        let _ = file;
                    }
                    crate::load::ResourceKind::Frame { image, rect } => {
                        if !frame_names.insert(r.name.clone()) {
                            return Err(at(
                                &module.name,
                                0,
                                &r.name,
                                "duplicate frame resource name",
                            ));
                        }
                        let entry = world.frames.entry(image.clone()).or_default();
                        entry.push((r.name.clone(), *rect));
                    }
                    _ => {}
                }
            }
        }
        for (image, frames) in &world.frames {
            let Some((_, _, w, h)) = world.images.get(image) else {
                return Err(at(
                    "",
                    0,
                    image,
                    "frame table references unknown image resource",
                ));
            };
            let (w, h) = (*w as i64, *h as i64);
            for (fname, rect) in frames {
                let [x, y, rw, rh] = rect.map(|v| v as i64);
                if rw < 1 || rh < 1 || x < 0 || y < 0 || x + rw > w || y + rh > h {
                    return Err(at(
                        "",
                        0,
                        fname,
                        &format!(
                            "frame rect [{x}, {y}, {rw}, {rh}] exceeds image '{image}' ({w}x{h})"
                        ),
                    ));
                }
            }
        }
    }
    world.defs.insert(root_name.clone(), root_def);

    let root_decl = node_of(project, &world.root);
    let view = root_decl
        .view2d
        .as_ref()
        .ok_or_else(|| at(&root_module, 0, &world.root, "root node has no view-2d"))?;
    let root_src = &project.modules[&root_module].source;

    for el in &view.elements {
        match el {
            View2dElement::Solid { name, .. } => world.solids.push(name.clone()),
            View2dElement::Camera { name, props } => {
                if world.camera.is_some() {
                    return Err(at(
                        &root_module,
                        0,
                        name,
                        "duplicate camera2d (components-2d: exactly one)",
                    ));
                }
                world.camera = Some(props.clone());
            }
            _ => {}
        }
    }

    fn expand(
        project: &Project,
        world: &mut WorldIR,
        node_name: &str,
        inst_name: &str,
        path: &str,
        parent: Option<usize>,
        props: &[Prop],
        line: usize,
        in_module: &str,
        stack: &mut Vec<(String, String)>,
        depth: usize,
        sprites: bool,
    ) -> Result<(), LowerError> {
        if depth > 16 {
            return Err(at(
                in_module,
                line,
                inst_name,
                "component nesting deeper than 16",
            ));
        }
        if world.instances.iter().any(|i| i.path == path) {
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
        if !world.defs.contains_key(node_name) {
            let def = build_def(project, node_name, false, sprites)?;
            world.defs.insert(node_name.to_string(), def);
        }
        let def_line = world.defs[node_name].line;
        let def_module = world.defs[node_name].module.clone();
        let decl = node_of(project, node_name);
        let def_param_count = world.defs[node_name].params.len();

        // 参数绑定：每个声明参数必须由实例提供（数值字面量 / resources.X /
        // 父参数引用 `.p` 或裸 `p`）。
        let parent_params: Vec<(String, ParamVal)> = parent
            .and_then(|p| world.instances.get(p).map(|i| i.params.clone()))
            .unwrap_or_default();
        let mut params: Vec<(String, ParamVal)> = Vec::new();
        let def_ptypes: Vec<(String, PType)> = world.defs[node_name]
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
            let val = match &pr.value {
                PropValue::One(Expr::Dot(outer, m))
                    if matches!(&**outer, Expr::Ident(b) if b == "self") =>
                {
                    parent_params
                        .iter()
                        .find(|(n, _)| n == m.as_str())
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
                    if !matches!(pty, PType::Res | PType::Img) {
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
                    if *pty == PType::Img && !world.images.contains_key(m.as_str()) {
                        return Err(at(
                            &def_module,
                            pline,
                            pname,
                            &format!(
                                "instance '{path}' param '{pname}' binds 'resources.{}' which is not an image2d resource",
                                m
                            ),
                        ));
                    }
                    ParamVal::Res(m.to_string())
                }
                PropValue::One(e) => {
                    if matches!(pty, PType::Res | PType::Img) {
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
        let _ = def_param_count;
        // 未知 prop：具名拒绝（带 prop 行号）。
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

        let def_idx = world.def_len(node_name);
        let stateful_inst = world.defs[node_name].stateful;
        let index = world.instances.len();
        world.instances.push(InstanceIR {
            path: path.to_string(),
            def: node_name.to_string(),
            parent,
            def_idx,
            params,
            line,
            parent_group: None,
        });
        if stateful_inst {
            world.stateful += 1;
            if world.stateful > 8 {
                return Err(at(
                    &def_module,
                    line,
                    inst_name,
                    &format!(
                        "stateful component instance '{path}' exceeds the 8-instance envelope (root included, total {})",
                        world.stateful
                    ),
                ));
            }
        }
        // 嵌套子实例：声明先序（父先子后）。
        stack.push((node_name.to_string(), path.to_string()));
        let view = decl.view2d.as_ref().ok_or_else(|| {
            at(
                &def_module,
                def_line,
                node_name,
                "component definition has no view-2d",
            )
        })?;
        for el in &view.elements {
            if let View2dElement::Instance { node, name, props } = el {
                let child_line = props
                    .first()
                    .map(|p| prop_line(&project.modules[&def_module].source, p))
                    .unwrap_or(def_line);
                let child_path = format!("{path}/{name}");
                expand(
                    project,
                    world,
                    node,
                    name,
                    &child_path,
                    Some(index),
                    props,
                    child_line,
                    &def_module,
                    stack,
                    depth + 1,
                    sprites,
                )?;
            }
        }
        stack.pop();
        Ok(())
    }

    let mut stack: Vec<(String, String)> = Vec::new();
    for el in &view.elements {
        if let View2dElement::Instance { node, name, props } = el {
            let line = props
                .first()
                .map(|p| prop_line(root_src, p))
                .unwrap_or_else(|| decl_line(project, &root_module, root_decl));
            let path = name.clone();
            expand(
                project,
                &mut world,
                node,
                name,
                &path,
                None,
                props,
                line,
                &root_module,
                &mut stack,
                0,
                sprites,
            )?;
        }
    }

    // PLAN-022：图像绑定面的具名校验（V-01）：直连 image 必须是 image2d
    // 资源、必须有非空帧表；字面量 frame ordinal 必须落在帧表内；字面量
    // flip 必须 0/1（Param 形态在实例解析处核，Field 形态运行期受信边界核）。
    if sprites {
        for (dname, d) in &world.defs {
            for se in &d.sprite_elems {
                if se.image_param.is_some() {
                    continue;
                }
                let img = &se.image;
                if !world.images.contains_key(img) {
                    return Err(at(
                        &d.module,
                        se.line,
                        &se.elem,
                        &format!(
                            "image element '{}' binds 'resources.{img}' which is not an image2d.png resource",
                            se.elem
                        ),
                    ));
                }
                let frames = world.frames.get(img).ok_or_else(|| {
                    at(
                        &d.module,
                        se.line,
                        &se.elem,
                        &format!(
                            "image element '{}' binds image '{img}' which declares no frame table",
                            se.elem
                        ),
                    )
                })?;
                if let FrameBind::Lit(v) = &se.frame {
                    if *v as usize >= frames.len() {
                        return Err(at(
                            &d.module,
                            se.line,
                            &se.elem,
                            &format!(
                                "literal frame ordinal {v} exceeds the frame table of image '{img}' ({} frame(s))",
                                frames.len()
                            ),
                        ));
                    }
                }
                if let FrameBind::Lit(v) = &se.flip {
                    if *v > 1 {
                        return Err(at(
                            &d.module,
                            se.line,
                            &se.elem,
                            &format!("literal flip must be 0/1, got {v}"),
                        ));
                    }
                }
            }
        }
    }

    // character 归属验证：root 直属或恰一实例的定义；全局恰一。
    let mut chars: Vec<(String, String, usize, String)> = Vec::new(); // (def, 元素名, 行, module)
    if let Some((elem, line)) = world.defs[&world.root].char_elem.clone() {
        chars.push((world.root.clone(), elem, line, world.root_module.clone()));
    }
    for (dname, d) in &world.defs {
        if let Some((elem, line)) = &d.char_elem {
            if dname == &world.root {
                continue;
            }
            let n = world.def_len(dname);
            if n != 1 {
                return Err(at(
                    &d.module,
                    *line,
                    elem,
                    &format!(
                        "character2d in component '{dname}' requires exactly one instance (found {n}); the character service is one-per-Run"
                    ),
                ));
            }
            chars.push((dname.clone(), elem.clone(), *line, d.module.clone()));
        }
    }
    if chars.len() > 1 {
        let (_src, elem, line, m) = chars[1].clone();
        return Err(at(
            &m,
            line,
            &elem,
            "multiple character2d declarations (the character service is one-per-Run)",
        ));
    }

    // 相机必须存在（Objects2V1 入口注入 _ag_cam2_freeze，缺相机 fault 27）。
    // PLAN-025：presentation-2d 无角色组合装配不注入相机 freeze（actor-free
    // 装配），相机可省；其余 profile 维持强制。
    if world.camera.is_none() && camera_required {
        return Err(at(
            &world.root_module,
            0,
            &world.root,
            "components-2d requires exactly one declared camera2d in the root view (Objects2V1 entry freezes camera state; absent camera faults 27)",
        ));
    }

    Ok(world)
}

// ------------------------------------------------------ dispatch plan ----

/// 子投递：发射定义实例 i → 自名子实例的接收下标。
#[derive(Debug, Clone)]
pub(crate) struct ChildDispatch {
    pub(crate) table: String,
    pub(crate) fn_name: String,
    #[allow(dead_code)] // 诊断/未来消费面保留（端口名）。
    ev: String,
    // 每发射实例的接收下标（root 发射方恰一项）。
    pub(crate) entries: Vec<usize>,
}

/// 祖先投递：发射定义实例 i → 最近声明 `on .Ev` 的祖先实例（或 root）。
#[derive(Debug, Clone)]
pub(crate) struct AncDispatch {
    pub(crate) table: String,
    pub(crate) fn_name: String,
    pub(crate) to_root: bool,
    #[allow(dead_code)] // 诊断/未来消费面保留（端口名）。
    ev: String,
    // 每发射实例的接收下标；None = root 接收（直呼）。
    pub(crate) entries: Vec<Option<usize>>,
}

/// 下降期事件投递计划（V-03；全部在生成前解析，无宿主队列）。
#[derive(Default)]
pub(crate) struct Plan {
    pub(crate) child: BTreeMap<(String, String, String), ChildDispatch>,
    pub(crate) anc: BTreeMap<(String, String), AncDispatch>,
    /// 接收 fn 名 → (接收定义（""=root）, 事件名)。
    pub(crate) recv_fns: BTreeMap<String, (String, String)>,
    /// PLAN-022：render2d.spawn 站点（元素名, 行），声明先序；具体
    /// (slot, row) 由 collect_scene 按同一顺序分配。
    pub(crate) render_spawns: Vec<(String, usize)>,
}

/// 遍历处理器语句树，收集：裸函数调用、`.member` 引用、emit 站点。
pub(crate) struct WalkOut {
    pub(crate) calls: Vec<(String, usize)>,
    pub(crate) members: BTreeSet<String>,
    /// `.member = ...` 赋值目标（参数只读合同的检测面）。
    pub(crate) assign_targets: Vec<(String, usize)>,
    pub(crate) child_emits: Vec<(String, String, usize, usize)>,
    pub(crate) anc_emits: Vec<(String, usize, usize)>,
    /// PLAN-021：queries-2d 事件序（绑定/读取/调用/引用/自身行）。
    pub(crate) query_events: Vec<QEvent>,
    /// PLAN-022：render2d.<op> 调用（op, spawn 目标元素, 元数, 行）。
    pub(crate) render_calls: Vec<(String, Option<String>, usize, usize)>,
}

/// PLAN-021：查询 handle 实参形态（plan 层验证引用存在/维度合法）。
#[derive(Debug, Clone)]
pub(crate) enum QHandle {
    Named(String),
    SelfRow,
    MinusOne,
}

/// PLAN-021：query2d 构造的有序事件（绑定模拟 + 具名验证的输入）。
#[derive(Debug, Clone)]
pub(crate) enum QEvent {
    /// `let q = query2d.segment/sweep(...)`（在 QueryCall 之后）。
    Bind {
        name: String,
        kind: QueryKind,
        line: usize,
    },
    /// `<结果变量>.<getter>(…)`。
    Getter {
        base: String,
        method: String,
        line: usize,
    },
    /// `query2d.segment/sweep(...)`；handle 实参在 walk 层解析形态。
    QueryCall {
        name: String,
        argc: usize,
        handle: QHandle,
        vo_ok: bool,
        line: usize,
    },
    /// `refs.<inst>` 值读或只读行 op（world/local/show）。
    RefRead { name: String, line: usize },
    /// `refs.<inst>.shift/place`（位置写；character 行具名拒绝）。
    RefWrite { name: String, line: usize },
    /// `self.row()` / self 行写（root 无行）。
    SelfRow { write: bool, line: usize },
    /// `var q = query2d.…`（结果只允许 let 绑定）。
    VarBind { name: String, line: usize },
    /// `q = …`（typed 结果不可再赋值）。
    ResultAssign { name: String, line: usize },
}

pub(crate) fn walk_stmts(
    stmts: &[Stmt],
    line: usize,
    out: &mut WalkOut,
    queries: bool,
    sprites: bool,
) {
    for s in stmts {
        match s {
            Stmt::Expr(e) => walk_expr_q(e, line, out, queries, sprites),
            Stmt::If(if_) => {
                for b in &if_.branches {
                    walk_expr_q(&b.cond, line, out, queries, sprites);
                    walk_stmts(&b.body.stmts, line, out, queries, sprites);
                }
                if let Some(e) = &if_.else_ {
                    walk_stmts(&e.stmts, line, out, queries, sprites);
                }
            }
            Stmt::Store(v) => {
                // PLAN-021：let 查询绑定先于其后的 getter 读取。
                if let Some(kind) = query_call_kind(&v.expr) {
                    if matches!(v.kind, auto_parser_api::ast::StoreKind::Var) {
                        out.query_events.push(QEvent::VarBind {
                            name: v.name.to_string(),
                            line,
                        });
                    } else {
                        out.query_events.push(QEvent::Bind {
                            name: v.name.to_string(),
                            kind,
                            line,
                        });
                    }
                }
                walk_expr_q(&v.expr, line, out, queries, sprites)
            }
            Stmt::Return(e) => walk_expr_q(e, line, out, queries, sprites),
            Stmt::EmptyLine(_) | Stmt::Comment(_) => {}
            _ => {}
        }
    }
}

/// PLAN-021：typed 结果 getter 方法名（cast5/sweep6 的并集）。
fn is_getter_method(m: &str) -> bool {
    matches!(
        m,
        "hit" | "node" | "t" | "point" | "normal" | "origin" | "start_overlap"
    )
}

pub(crate) fn walk_expr_q(e: &Expr, line: usize, out: &mut WalkOut, queries: bool, sprites: bool) {
    match e {
        Expr::Call(c) => {
            let cline = c.pos.as_ref().map(|p| p.line).unwrap_or(line);
            match &*c.name {
                Expr::Dot(inner, ev) => match &**inner {
                    Expr::Ident(base) => match base.as_str() {
                        "emit" => out
                            .anc_emits
                            .push((ev.to_string(), c.args.args.len(), cline)),
                        // PLAN-027：camera3d 读站点（runtime 档；translator
                        // 降层为受信 ag_cam3_* 读 getter，非子投递）。
                        "input" | "refs" | "self" | "query2d" | "camera3d" => {}
                        // PLAN-025：typed 声音请求站点（pres 档；translator
                        // 降层为生成收集器调用，非子投递）。
                        "sfx" => {}
                        // PLAN-022：render2d.<op> 站点（sprites 档记录）。
                        "render2d" => {
                            let target = match ev.as_str() {
                                "spawn" => c.args.args.first().and_then(|a| match a {
                                    auto_parser_api::ast::Arg::Pos(Expr::Str(n)) => {
                                        Some(n.to_string())
                                    }
                                    _ => None,
                                }),
                                // frame：记录图像名（帧表生成面）。
                                "frame" => c.args.args.get(1).and_then(|a| match a {
                                    auto_parser_api::ast::Arg::Pos(Expr::Str(n)) => {
                                        Some(n.to_string())
                                    }
                                    _ => None,
                                }),
                                _ => None,
                            };
                            out.render_calls.push((
                                ev.to_string(),
                                target,
                                c.args.args.len(),
                                cline,
                            ));
                        }
                        // PLAN-021：typed 结果 getter 事件（仅 queries 档；
                        // 其余保持子投递语义）。
                        child if queries && is_getter_method(ev) => {
                            out.query_events.push(QEvent::Getter {
                                base: child.to_string(),
                                method: ev.to_string(),
                                line: cline,
                            })
                        }
                        child => out.child_emits.push((
                            child.to_string(),
                            ev.to_string(),
                            c.args.args.len(),
                            cline,
                        )),
                    },
                    _ => {}
                },
                Expr::Ident(f) => out.calls.push((f.to_string(), cline)),
                _ => {}
            }
            // PLAN-021：query2d 调用事件（在实参递归前记录，序与文本序
            // 一致；实参中的 refs 值读随后逐个入列）。
            if let Expr::Dot(inner, m) = &*c.name {
                if let Expr::Ident(base) = &**inner {
                    if base == "query2d" && matches!(m.as_str(), "segment" | "sweep") {
                        let handle = match c.args.args.first() {
                            Some(auto_parser_api::ast::Arg::Pos(Expr::Int(v))) if *v == -1 => {
                                QHandle::MinusOne
                            }
                            // `-1` 解析为一元负号字面量。
                            Some(auto_parser_api::ast::Arg::Pos(Expr::Unary(
                                auto_parser_api::ast::Op::Sub,
                                inner,
                            ))) if matches!(&**inner, Expr::Int(v) if *v == 1) => QHandle::MinusOne,
                            Some(auto_parser_api::ast::Arg::Pos(Expr::Dot(dn, dn2))) => {
                                if matches!(&**dn, Expr::Ident(r) if r == "refs") {
                                    QHandle::Named(dn2.to_string())
                                } else {
                                    QHandle::Named(String::new())
                                }
                            }
                            Some(auto_parser_api::ast::Arg::Pos(Expr::Call(cc))) => {
                                if matches!(
                                    &*cc.name,
                                    Expr::Dot(dn, dm)
                                        if matches!(&**dn, Expr::Ident(b) if b == "self") && dm == "row"
                                ) {
                                    QHandle::SelfRow
                                } else {
                                    QHandle::Named(String::new())
                                }
                            }
                            _ => QHandle::Named(String::new()),
                        };
                        let vo_ok = match c.args.args.get(1) {
                            Some(auto_parser_api::ast::Arg::Pos(Expr::Int(v))) => {
                                *v == 0 || *v == 1
                            }
                            Some(auto_parser_api::ast::Arg::Pos(Expr::Bool(_))) => true,
                            _ => false,
                        };
                        out.query_events.push(QEvent::QueryCall {
                            name: m.to_string(),
                            argc: c.args.args.len(),
                            handle,
                            vo_ok,
                            line: cline,
                        });
                    }
                }
            }
            // PLAN-021：refs.<inst>.<op> 行 op（读/写事件分流）。
            if let Expr::Dot(op_node, op) = &*c.name {
                if let Expr::Dot(root, inst) = &**op_node {
                    if matches!(&**root, Expr::Ident(r) if r == "refs") {
                        match op.as_str() {
                            "shift" | "place" => out.query_events.push(QEvent::RefWrite {
                                name: inst.to_string(),
                                line: cline,
                            }),
                            "show" | "world" | "local" => out.query_events.push(QEvent::RefRead {
                                name: inst.to_string(),
                                line: cline,
                            }),
                            _ => {}
                        }
                    }
                }
            }
            // PLAN-021：self 行 op。
            if let Expr::Dot(inner, op) = &*c.name {
                if matches!(&**inner, Expr::Ident(b) if b == "self") {
                    match op.as_str() {
                        "row" | "world" | "local" => out.query_events.push(QEvent::SelfRow {
                            write: false,
                            line: cline,
                        }),
                        "shift" | "place" => out.query_events.push(QEvent::SelfRow {
                            write: true,
                            line: cline,
                        }),
                        _ => {}
                    }
                }
            }
            for a in &c.args.args {
                if let auto_parser_api::ast::Arg::Pos(x) = a {
                    walk_expr_q(x, line, out, queries, sprites);
                }
            }
        }
        Expr::Dot(inner, name) => {
            if matches!(&**inner, Expr::Ident(b) if b == "self") {
                out.members.insert(name.to_string());
            } else {
                // PLAN-021：裸 `refs.<inst>` 值读。
                if let Expr::Ident(b) = &**inner {
                    if b == "refs" {
                        out.query_events.push(QEvent::RefRead {
                            name: name.to_string(),
                            line,
                        });
                        return;
                    }
                }
                walk_expr_q(inner, line, out, queries, sprites);
            }
        }
        Expr::Bina(l, op, r) => {
            // 赋值目标记录：`.member = ...` 的左值（成员写；参数只读合
            // 同）。仅检 Asn 左侧——右侧是读取（`.ticks = .phase0` 合法，
            // .phase0 为参数）；嵌套写入（let x = (.p = v)、
            // if (.p = v) == v {…}）由递归进入内层 Asn 同样命中。
            if matches!(op, auto_parser_api::ast::Op::Asn) {
                if let Expr::Dot(inner, name) = &**l {
                    if matches!(&**inner, Expr::Ident(b) if b == "self") {
                        out.assign_targets.push((name.to_string(), line));
                    }
                }
                // PLAN-021（F-R1-03）：typed 结果变量再赋值（plan 层具名
                // module/line/symbol；translate 侧为 backstop）。
                if queries {
                    if let Expr::Ident(name) = &**l {
                        out.query_events.push(QEvent::ResultAssign {
                            name: name.to_string(),
                            line,
                        });
                    }
                }
            }
            walk_expr_q(l, line, out, queries, sprites);
            walk_expr_q(r, line, out, queries, sprites);
        }
        Expr::Unary(_, i) => walk_expr_q(i, line, out, queries, sprites),
        Expr::Tuple(items) => {
            for i in items {
                walk_expr_q(i, line, out, queries, sprites);
            }
        }
        _ => {}
    }
}

fn root_member(project: &Project, world: &WorldIR, ev: &str) -> Option<(usize, usize)> {
    let decl = node_of(project, &world.root);
    let on = decl.on.as_ref()?;
    let h = on
        .handlers
        .iter()
        .find(|h| matches!(&h.pattern, EventPattern::Member { name, .. } if name == ev))?;
    let line = line_at(&project.modules[&world.root_module].source, h.span.0);
    match &h.pattern {
        EventPattern::Member { params, .. } => Some((params.len(), line)),
        _ => None,
    }
}

fn find_child_node(project: &Project, def_name: &str, child: &str) -> Option<String> {
    let decl = node_of(project, def_name);
    if let Some(view) = &decl.view2d {
        for el in &view.elements {
            if let View2dElement::Instance { node, name, .. } = el {
                if name == child {
                    return Some(node.clone());
                }
            }
        }
    }
    // PLAN-024：view-3d 子实例（group3d 子树递归；同一 node 只有一个
    // view 块，两分支互斥）。
    if let Some(view) = &decl.view3d {
        return find_child_node3d(&view.elements, child);
    }
    None
}

/// PLAN-024：view-3d 子树内按实例名查子 node 定义。
fn find_child_node3d(
    elements: &[auto_ui_dialect::node2d::View3dElement],
    child: &str,
) -> Option<String> {
    for el in elements {
        match el {
            auto_ui_dialect::node2d::View3dElement::Instance { node, name, .. } => {
                if name == child {
                    return Some(node.clone());
                }
            }
            auto_ui_dialect::node2d::View3dElement::Group { children, .. } => {
                if let Some(node) = find_child_node3d(children, child) {
                    return Some(node);
                }
            }
            _ => {}
        }
    }
    None
}

/// 构建投递计划并验证全部 emit 站点（端口存在 + 元数一致）与 handler 面
/// （未知成员 / 未知 helper fn）。
pub(crate) fn plan_dispatch(
    project: &Project,
    world: &WorldIR,
    queries: bool,
    sprites: bool,
    pres: bool,
) -> Result<Plan, LowerError> {
    plan_dispatch_events(project, world, queries, sprites, pres, false, false)
}

/// PLAN-028：`events` = components-3d-events 档（spawn/bind 裸调用是
/// typed 服务端口，不是模块 helper fn；其余档保持旧拒绝面）。
pub(crate) fn plan_dispatch_events(
    project: &Project,
    world: &WorldIR,
    queries: bool,
    sprites: bool,
    pres: bool,
    events: bool,
    bindings: bool,
) -> Result<Plan, LowerError> {
    let mut plan = Plan::default();
    let root_prefix = world.root.to_lowercase();

    // PLAN-021: plan-time named-row name table (top-level instances; the
    // character row is addressable but position-read-only). Ordinals are
    // assigned by collect_scene under the same rule.
    let mut refs_names: BTreeMap<String, bool> = BTreeMap::new();
    for inst in &world.instances {
        if inst.path.contains('/') {
            continue;
        }
        let d = &world.defs[&inst.def];
        let is_char = d.char_elem.is_some();
        // F-R1-01（§9冻结合同）：具名行引用要求恰一静态 shape；多静态
        // shape 实例具名拒绝（否则附加 shape 无声成为可查询几何行）。
        if queries && !is_char && d.static_elems.len() > 1 {
            return Err(at(
                &d.module,
                d.line,
                &inst.path,
                &format!(
                    "instance '{}' has {} static shapes; a named row ref resolves exactly one (make the extra shapes dynamic or split the definition)",
                    inst.path,
                    d.static_elems.len()
                ),
            ));
        }
        if is_char || !d.static_elems.is_empty() {
            refs_names.insert(inst.path.clone(), !is_char);
        }
    }

    let mut emitters: Vec<String> = vec![world.root.clone()];
    for i in &world.instances {
        if !emitters.contains(&i.def) {
            emitters.push(i.def.clone());
        }
    }

    for edef in &emitters {
        let def = &world.defs[edef];
        let decl = node_of(project, edef);
        let Some(on) = &decl.on else { continue };
        let is_root = edef == &world.root;
        let model_fields: BTreeSet<String> = def.model.iter().map(|(n, _)| n.clone()).collect();
        for h in &on.handlers {
            let hline = line_at(&project.modules[&def.module].source, h.span.0);
            let mut out = WalkOut {
                calls: Vec::new(),
                members: BTreeSet::new(),
                assign_targets: Vec::new(),
                child_emits: Vec::new(),
                anc_emits: Vec::new(),
                query_events: Vec::new(),
                render_calls: Vec::new(),
            };
            walk_stmts(&h.body.stmts, hline, &mut out, queries, sprites);

            // ---- PLAN-022: render2d surface validation (named
            // module/line/symbol diagnostics; slot budget checked below) ----
            for (op, target, argc, rline) in &out.render_calls {
                // PLAN-025：presentation 档仅开放 despawn（作者 on 决定销毁；
                // token = self.handle 只读句柄），其余 render2d 面仍归 sprites。
                if !sprites && !(pres && op == "despawn") {
                    return Err(at(
                        &def.module,
                        *rline,
                        "render2d",
                        "render2d is only available in the sprites-2d profile (presentation-2d allows render2d.despawn only)",
                    ));
                }
                if !sprites && op == "despawn" {
                    if *argc != 1 {
                        return Err(at(
                            &def.module,
                            *rline,
                            "render2d",
                            &format!("render2d.despawn arity {argc} (want <handle>)"),
                        ));
                    }
                    continue;
                }
                match op.as_str() {
                    "spawn" => {
                        if *argc != 3 {
                            return Err(at(
                                &def.module,
                                *rline,
                                "render2d",
                                &format!("render2d.spawn arity {argc} (want <elem>, x, y)"),
                            ));
                        }
                        let Some(t) = target else {
                            return Err(at(
                                &def.module,
                                *rline,
                                "render2d",
                                "render2d.spawn target must be a string literal element name",
                            ));
                        };
                        let root_def = &world.defs[&world.root];
                        let el = root_def
                            .sprite_elems
                            .iter()
                            .find(|e| e.elem == *t && !e.is_char)
                            .ok_or_else(|| {
                                at(
                                    &def.module,
                                    *rline,
                                    t,
                                    &format!(
                                        "unknown render2d.spawn target '{t}' (no bound image leaf of that name in the root view)"
                                    ),
                                )
                            })?;
                        if el.image_param.is_some() {
                            return Err(at(
                                &def.module,
                                *rline,
                                t,
                                "render2d.spawn template must bind resources.<image> directly (the root has no params)",
                            ));
                        }
                        if !matches!(el.frame, FrameBind::Lit(_))
                            || !matches!(el.flip, FrameBind::Lit(_))
                        {
                            return Err(at(
                                &def.module,
                                *rline,
                                t,
                                &format!(
                                    "render2d.spawn template '{t}' must declare literal frame/flip (spawned slots reset to the document default)"
                                ),
                            ));
                        }
                        plan.render_spawns.push((t.clone(), *rline));
                    }
                    "despawn" => {
                        if *argc != 1 {
                            return Err(at(
                                &def.module,
                                *rline,
                                "render2d",
                                &format!("render2d.despawn arity {argc} (want <token>)"),
                            ));
                        }
                    }
                    "frame" => {
                        if *argc != 4 {
                            return Err(at(
                                &def.module,
                                *rline,
                                "render2d",
                                &format!(
                                    "render2d.frame arity {argc} (want <token>, <image>, <ordinal>, <flip>)"
                                ),
                            ));
                        }
                        if let Some(img) = target {
                            if !world.frames.contains_key(img.as_str()) {
                                return Err(at(
                                    &def.module,
                                    *rline,
                                    &img,
                                    &format!(
                                        "render2d.frame image '{img}' is not an image2d resource with a frame table"
                                    ),
                                ));
                            }
                        } else {
                            return Err(at(
                                &def.module,
                                *rline,
                                "render2d",
                                "render2d.frame image must be a string literal image resource name",
                            ));
                        }
                    }
                    other => {
                        return Err(at(
                            &def.module,
                            *rline,
                            "render2d",
                            &format!("unknown render2d service '{other}' (spawn/despawn/frame)"),
                        ));
                    }
                }
            }

            // ---- PLAN-021: queries-2d surface validation (named
            // module/line/symbol diagnostics) ----
            let mut q_binds: Vec<(String, QueryKind)> = Vec::new();
            for qe in &out.query_events {
                match qe {
                    QEvent::QueryCall {
                        name,
                        argc,
                        handle,
                        vo_ok,
                        line,
                    } => {
                        if !queries {
                            return Err(at(
                                &def.module,
                                *line,
                                "query2d",
                                "query2d is only available in the queries-2d profile",
                            ));
                        }
                        if name != "segment" && name != "sweep" {
                            return Err(at(
                                &def.module,
                                *line,
                                name,
                                "unknown query service (query2d.segment/sweep)",
                            ));
                        }
                        let want = if name == "segment" { 6 } else { 4 };
                        if *argc != want {
                            return Err(at(
                                &def.module,
                                *line,
                                name,
                                &format!("query2d.{name} arity {argc} (want {want} args)"),
                            ));
                        }
                        match handle {
                            QHandle::Named(n) => {
                                if !refs_names.contains_key(n) {
                                    return Err(at(
                                        &def.module,
                                        *line,
                                        n,
                                        &format!("unknown query row reference 'refs.{n}'"),
                                    ));
                                }
                            }
                            QHandle::SelfRow => {
                                if is_root {
                                    return Err(at(
                                        &def.module,
                                        *line,
                                        "self",
                                        "self.row() has no instance row at the root",
                                    ));
                                }
                            }
                            QHandle::MinusOne => {
                                if name == "sweep" {
                                    return Err(at(
                                        &def.module,
                                        *line,
                                        "sweep",
                                        "sweep mover must be a named row (refs.<inst>) or self.row()",
                                    ));
                                }
                            }
                        }
                        if !vo_ok {
                            return Err(at(
                                &def.module,
                                *line,
                                name,
                                "visible_only must be the literal 0/1 (or true/false)",
                            ));
                        }
                    }
                    QEvent::Bind { name, kind, line } => {
                        if q_binds.iter().any(|(n, _)| n == name) {
                            return Err(at(
                                &def.module,
                                *line,
                                name,
                                &format!(
                                    "query result '{name}' rebinds (one let per query result name)"
                                ),
                            ));
                        }
                        q_binds.push((name.clone(), *kind));
                    }
                    QEvent::Getter { base, method, line } => {
                        if !queries {
                            continue;
                        }
                        let found = q_binds.iter().find(|(n, _)| n == base);
                        let Some((_, k)) = found else {
                            return Err(at(
                                &def.module,
                                *line,
                                base,
                                &format!(
                                    "unknown query result '{base}' (bind it with a let from query2d)"
                                ),
                            ));
                        };
                        let ok = match k {
                            QueryKind::Cast => {
                                matches!(method.as_str(), "hit" | "node" | "t" | "point" | "normal")
                            }
                            QueryKind::Sweep => matches!(
                                method.as_str(),
                                "hit" | "node" | "t" | "origin" | "normal" | "start_overlap"
                            ),
                        };
                        if !ok {
                            let this = if *k == QueryKind::Cast {
                                "cast"
                            } else {
                                "sweep"
                            };
                            let other = if *k == QueryKind::Cast {
                                "sweep"
                            } else {
                                "cast"
                            };
                            return Err(at(
                                &def.module,
                                *line,
                                base,
                                &format!(
                                    "'.{method}()' is a {other} result getter; '{base}' is a {this} result"
                                ),
                            ));
                        }
                    }
                    // PLAN-028：具名行读/写校验属 queries-2d 面（refs.<inst>
                    // = 实例路径）；components-3d 的 refs.<elem>.place/shift
                    // 由 space3 行环境在 translate 期校验，这里不再拦截。
                    QEvent::RefRead { name, line } => {
                        if queries && !refs_names.contains_key(name) {
                            return Err(at(
                                &def.module,
                                *line,
                                name,
                                &format!("unknown query row reference 'refs.{name}'"),
                            ));
                        }
                    }
                    QEvent::RefWrite { name, line } => {
                        let Some(writable) = refs_names.get(name) else {
                            if !queries {
                                continue;
                            }
                            return Err(at(
                                &def.module,
                                *line,
                                name,
                                &format!("unknown query row reference 'refs.{name}'"),
                            ));
                        };
                        if !*writable {
                            return Err(at(
                                &def.module,
                                *line,
                                name,
                                &format!(
                                    "row 'refs.{name}' is the character row; its position is owned by the character service"
                                ),
                            ));
                        }
                    }
                    QEvent::ResultAssign { name, line } => {
                        let bound = q_binds.iter().any(|(n, _)| n == name);
                        if bound {
                            return Err(at(
                                &def.module,
                                *line,
                                name,
                                "query result cannot be reassigned (read the values out instead)",
                            ));
                        }
                    }
                    QEvent::VarBind { name, line } => {
                        return Err(at(
                            &def.module,
                            *line,
                            name,
                            "query results bind with 'let' only ('var' would claim mutable ownership of a shared latest-slot token)",
                        ));
                    }
                    QEvent::SelfRow { write, line } => {
                        if is_root {
                            return Err(at(
                                &def.module,
                                *line,
                                "self",
                                "self.row() has no instance row at the root",
                            ));
                        }
                        if *write && def.char_elem.is_some() {
                            return Err(at(
                                &def.module,
                                *line,
                                "self",
                                "self row is the character row; its position is owned by the character service",
                            ));
                        }
                    }
                }
            }
            for m in &out.members {
                // PLAN-025：`.handle` 是本定义恰一动态 shape2d 的生成只读
                // 句柄（F-8 保留名；仅实例 handler，root 无槽）。
                if m == "handle" {
                    if !pres {
                        return Err(at(
                            &def.module,
                            hline,
                            m,
                            "handler references unknown member '.handle' (the generated slot handle read is presentation-2d only)",
                        ));
                    }
                    if def.dyn_elems.len() != 1 || is_root {
                        return Err(at(
                            &def.module,
                            hline,
                            m,
                            "self.handle is only available on instance handlers of a definition with exactly one dynamic shape2d",
                        ));
                    }
                    continue;
                }
                if !model_fields.contains(m) && !def.params.iter().any(|(n, _)| n == m) {
                    return Err(at(
                        &def.module,
                        hline,
                        m,
                        &format!("handler references unknown member '.{m}'"),
                    ));
                }
            }
            // 参数只读（AC-02/06）：`.param` 作为赋值目标具名拒绝；model
            // 字段赋值不受影响。
            for (m, tline) in &out.assign_targets {
                if def.params.iter().any(|(n, _)| n == m) {
                    return Err(at(
                        &def.module,
                        *tline,
                        m,
                        &format!(
                            "params are read-only: '.{m}' cannot be assigned (model fields are the mutable state)"
                        ),
                    ));
                }
            }
            for (f, fline) in &out.calls {
                if events && (f == "spawn" || f == "bind") {
                    continue;
                }
                let fmod = &project.modules[&def.module];
                if !fmod.fns.iter().any(|(n, _)| n == f) {
                    return Err(at(
                        &def.module,
                        *fline,
                        f,
                        &format!("unknown helper fn '{f}' in module '{}' scope", def.module),
                    ));
                }
            }
            // ---- 子投递 ----
            for (child, ev, argc, cline) in &out.child_emits {
                // PLAN-028：event.*/object.*/round.* 是 typed 服务前缀，
                // 不是子投递选择器（events 档跳过；其余档保持旧拒绝面；
                // round.request() 的受信映射沿 025/027 既有面）。
                if (events && (child == "event" || child == "object" || child == "round")) || child == "obj" {
                    continue;
                }
                let key = (edef.clone(), child.clone(), ev.clone());
                if plan.child.contains_key(&key) {
                    continue;
                }
                let child_node = find_child_node(project, edef, child).ok_or_else(|| {
                    at(
                        &def.module,
                        *cline,
                        child,
                        &format!(
                            "emit target instance '{child}' is not declared in the '{edef}' view-2d"
                        ),
                    )
                })?;
                let cdef = &world.defs[&child_node];
                let (_, payload_types) = cdef.msgs.get(ev.as_str()).ok_or_else(|| {
                    at(
                        &cdef.module,
                        *cline,
                        ev,
                        &format!(
                            "unknown event port '.{ev}' on node '{child_node}' (declared: {})",
                            cdef.msgs.keys().cloned().collect::<Vec<_>>().join(", ")
                        ),
                    )
                })?;
                let (params, mline) = cdef.members.get(ev.as_str()).copied().ok_or_else(|| {
                    at(
                        &cdef.module,
                        *cline,
                        ev,
                        &format!("event port '.{ev}' has no `on .{ev}` handler on '{child_node}'"),
                    )
                })?;
                if *argc != payload_types.len() || *argc != params {
                    return Err(at(
                        &cdef.module,
                        mline.min(*cline),
                        ev,
                        &format!(
                            "emit '{child}.{ev}' carries {argc} arg(s) but the payload declares {} and the handler accepts {params}",
                            payload_types.len()
                        ),
                    ));
                }
                let table = format!("{child}_bind_{}", ev.to_uppercase());
                let fn_name = format!("{child}_{}", ev.to_lowercase());
                let mut entries = Vec::new();
                if is_root {
                    let recv = world
                        .instances
                        .iter()
                        .find(|i| i.parent.is_none() && inst_name_of(&i.path) == child)
                        .ok_or_else(|| {
                            at(
                                &cdef.module,
                                *cline,
                                child,
                                &format!(
                                    "root view has no instance '{child}' for emit '{child}.{ev}'"
                                ),
                            )
                        })?;
                    entries.push(recv.def_idx);
                } else {
                    for (k, inst) in world.instances.iter().enumerate() {
                        if inst.def != *edef {
                            continue;
                        }
                        let recv = world.children_of(k, child).ok_or_else(|| {
                            at(
                                &cdef.module,
                                *cline,
                                child,
                                &format!(
                                    "instance '{}' has no child instance '{child}' for emit '{child}.{ev}'",
                                    inst.path
                                ),
                            )
                        })?;
                        entries.push(world.instances[recv].def_idx);
                    }
                }
                plan.recv_fns
                    .insert(fn_name.clone(), (child_node.clone(), ev.clone()));
                plan.child.insert(
                    key,
                    ChildDispatch {
                        table,
                        fn_name,
                        ev: ev.clone(),
                        entries,
                    },
                );
            }
            // ---- 祖先投递 ----
            for (ev, argc, cline) in &out.anc_emits {
                let key = (edef.clone(), ev.clone());
                if plan.anc.contains_key(&key) {
                    continue;
                }
                // PLAN-030：`emit .<msg参数>` 经绑定表投递（下降期解析为
                // 绑定根事件 recv fn 的直呼文本），不走祖先查表。根事件
                // 变体 + handler + 零载荷在此核验并注册 main 前缀接收 fn
                // （cfg 档重建 plan 时注册随 plan 走，不丢失）。
                if bindings
                    && decl
                        .params
                        .iter()
                        .any(|(n, ty)| {
                            n.as_str() == ev.as_str()
                                && crate::node3d_binding_refs::ref_kind_of(ty)
                                    == Some(crate::node3d_binding_refs::RefKind::MsgEvent)
                        })
                {
                    if *argc != 0 {
                        return Err(at(
                            &def.module,
                            *cline,
                            ev,
                            &format!(
                                "emit .{ev} carries {argc} arg(s); a msg param port forwards to its bound root event verbatim (zero-arg form)"
                            ),
                        ));
                    }
                    // 绑定事件名 = 该参数的实例绑定 Res(".Ev")（uniformity
                    // 由 bindings 装配核；此处取首实例绑定值）。
                    let bound = world
                        .instances
                        .iter()
                        .filter(|i| i.def == *edef)
                        .find_map(|i| {
                            i.params.iter().find(|(n, _)| n.as_str() == ev.as_str()).and_then(
                                |(_, v)| match v {
                                    crate::lower_components::ParamVal::Res(t) => {
                                        t.strip_prefix('.').map(|e| e.to_string())
                                    }
                                    _ => None,
                                },
                            )
                        })
                        .ok_or_else(|| {
                            at(
                                &def.module,
                                *cline,
                                ev,
                                &format!(
                                    "emit .{ev} via a msg param has no symbol binding (bind it as `.RootEvent` at the instance)"
                                ),
                            )
                        })?;
                    let rdecl = node_of(project, &world.root);
                    let variant = rdecl
                        .msg
                        .as_ref()
                        .and_then(|m| m.variants.iter().find(|v| v.name.as_str() == bound))
                        .ok_or_else(|| {
                            at(
                                &def.module,
                                *cline,
                                ev,
                                &format!(
                                    "emit .{ev} via a msg param binds '.{bound}' which is not a root msg variant"
                                ),
                            )
                        })?;
                    if !variant.payload.is_empty() {
                        return Err(at(
                            &def.module,
                            *cline,
                            ev,
                            &format!(
                                "emit .{ev} via a msg param binds '.{bound}' which carries a payload (zero-arg form only in components-3d-bindings v1)"
                            ),
                        ));
                    }
                    let has_handler = rdecl.on.as_ref().is_some_and(|on| {
                        on.handlers.iter().any(|h| {
                            matches!(&h.pattern,
                                auto_ui_dialect::node2d::EventPattern::Member { name, .. }
                                    if name.as_str() == bound)
                        })
                    });
                    if !has_handler {
                        return Err(at(
                            &def.module,
                            *cline,
                            ev,
                            &format!(
                                "emit .{ev} via a msg param binds '.{bound}' which has no `on .{bound}` handler on the root"
                            ),
                        ));
                    }
                    let fn_name = format!("{root_prefix}_{}", bound.to_lowercase());
                    plan.recv_fns
                        .insert(fn_name.clone(), (String::new(), bound.clone()));
                    continue;
                }
                if is_root {
                    // root 发射 .Ev → 自身成员处理器（直呼，无索引）。
                    let (params, mline) = root_member(project, world, ev).ok_or_else(|| {
                        at(
                            &world.root_module,
                            *cline,
                            ev,
                            &format!("emit .{ev} has no `on .{ev}` handler on the root"),
                        )
                    })?;
                    let payload = world.defs[&world.root].msgs[ev.as_str()].1.clone();
                    if *argc != payload.len() || *argc != params {
                        return Err(at(
                            &world.root_module,
                            mline,
                            ev,
                            &format!(
                                "emit .{ev} carries {argc} arg(s) but the payload declares {} and the handler accepts {params}",
                                payload.len()
                            ),
                        ));
                    }
                    let fn_name = format!("{root_prefix}_{}", ev.to_lowercase());
                    plan.recv_fns
                        .insert(fn_name.clone(), (String::new(), ev.clone()));
                    plan.anc.insert(
                        key,
                        AncDispatch {
                            table: String::new(),
                            fn_name,
                            to_root: true,
                            ev: ev.clone(),
                            entries: Vec::new(),
                        },
                    );
                    continue;
                }
                let mut to_root: Option<bool> = None;
                let mut recv_def: Option<String> = None;
                let mut entries: Vec<Option<usize>> = Vec::new();
                for inst in world.instances.iter().filter(|i| i.def == *edef) {
                    let mut cur = inst.parent;
                    let mut hit: Option<(String, usize)> = None;
                    while let Some(p) = cur {
                        let pdef = world.instances[p].def.clone();
                        if world.defs[&pdef].members.contains_key(ev.as_str()) {
                            hit = Some((pdef, p));
                            break;
                        }
                        cur = world.instances[p].parent;
                    }
                    let (rdef, ridx) = match hit {
                        Some((d, p)) => (d, Some(p)),
                        None => {
                            if root_member(project, world, ev).is_some() {
                                (String::new(), Some(usize::MAX))
                            } else {
                                return Err(at(
                                    &def.module,
                                    *cline,
                                    ev,
                                    &format!(
                                        "emit .{ev} from instance '{}' finds no ancestor with an `on .{ev}` handler",
                                        inst.path
                                    ),
                                ));
                            }
                        }
                    };
                    let tr = rdef.is_empty();
                    if let (Some(et), Some(ed)) = (to_root, &recv_def) {
                        if et != tr || ed != &rdef {
                            return Err(at(
                                &def.module,
                                *cline,
                                ev,
                                &format!(
                                    "emit .{ev} resolves to different receivers across instances of '{edef}'"
                                ),
                            ));
                        }
                    }
                    to_root = Some(tr);
                    recv_def = Some(rdef.clone());
                    entries.push(match ridx {
                        Some(usize::MAX) => None,
                        Some(ri) => Some(world.instances[ri].def_idx),
                        None => None,
                    });
                }
                let rdef = recv_def.unwrap_or_default();
                let payload = if rdef.is_empty() {
                    world.defs[&world.root].msgs[ev.as_str()].1.clone()
                } else {
                    world.defs[&rdef].msgs[ev.as_str()].1.clone()
                };
                if *argc != payload.len() {
                    return Err(at(
                        &def.module,
                        *cline,
                        ev,
                        &format!(
                            "emit .{ev} carries {argc} arg(s) but the payload declares {}",
                            payload.len()
                        ),
                    ));
                }
                let fn_name = if rdef.is_empty() {
                    format!("{root_prefix}_{}", ev.to_lowercase())
                } else {
                    format!("{}_{}", rdef.to_lowercase(), ev.to_lowercase())
                };
                let table = format!("{}_emit_{}", edef.to_lowercase(), ev.to_uppercase());
                plan.recv_fns
                    .insert(fn_name.clone(), (rdef.clone(), ev.clone()));
                plan.anc.insert(
                    key,
                    AncDispatch {
                        table,
                        fn_name,
                        to_root: rdef.is_empty(),
                        ev: ev.clone(),
                        entries,
                    },
                );
            }
        }
    }

    // PLAN-022：运行时对象槽预算（SLOTS=8 冻结合同）：app_start 声明先序
    // spawn（每实例每动态/图像模板元素一个）+ render2d.spawn 站点 ≤ 8；
    // 超出在降层具名拒绝（运行时第 9 槽 fault 31 不再可达）。
    if sprites {
        let mut slots = 0usize;
        for inst in &world.instances {
            let def = &world.defs[&inst.def];
            let sprite_dyn = def
                .sprite_elems
                .iter()
                .filter(|e| !e.is_char)
                .filter(|e| {
                    matches!(e.frame, FrameBind::Field(_)) || matches!(e.flip, FrameBind::Field(_))
                })
                .count();
            slots += def.dyn_elems.len() + sprite_dyn;
        }
        slots += plan
            .render_spawns
            .iter()
            .map(|(e, _)| e)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        if slots > 8 {
            return Err(at(
                &world.root_module,
                0,
                &world.root,
                &format!(
                    "sprites-2d runtime object slots exceed the frozen 8-slot pool (dynamic instance templates + render2d.spawn sites = {slots})"
                ),
            ));
        }
    }
    Ok(plan)
}

// ------------------------------------------------------ view values ----

/// 位置/尺寸/可见分量求值：编译期数值或运行期 model 引用。
#[derive(Debug, Clone)]
pub(crate) enum CVal {
    Num(f64),
    Ref(String),
}

pub(crate) fn resolve_cval(
    def: &DefIR,
    e: &Expr,
    params: &[(String, ParamVal)],
) -> Result<CVal, LowerError> {
    if let Some(v) = numeric_lit(e) {
        return Ok(CVal::Num(v));
    }
    if let Expr::Ident(p) = e {
        if let Some((_, ParamVal::Num(v))) = params.iter().find(|(n, _)| n == p.as_str()) {
            return Ok(CVal::Num(*v));
        }
    }
    if let Expr::Dot(inner, p) = e {
        if matches!(&**inner, Expr::Ident(b) if b == "self") {
            if let Some((_, ParamVal::Num(v))) = params.iter().find(|(n, _)| n == p.as_str()) {
                return Ok(CVal::Num(*v));
            }
            if def.model.iter().any(|(n, _)| n == p.as_str()) {
                return Ok(CVal::Ref(p.to_string()));
            }
        }
    }
    Err(at(
        &def.module,
        def.line,
        &format!("{e:?}"),
        "view expression must be a numeric literal, param or '.field' model ref",
    ))
}

pub(crate) fn prop_exprs(pr: &Prop) -> Vec<Expr> {
    match &pr.value {
        PropValue::Pair(es) => es.clone(),
        PropValue::One(Expr::Tuple(es)) => es.clone(),
        PropValue::One(e) => vec![e.clone()],
    }
}

pub(crate) fn prop_of<'a>(props: &'a [Prop], key: &str) -> Option<&'a Prop> {
    props.iter().find(|p| p.name == key)
}

/// 位置二元组 → 两个分量（字面量/参数/model 引用）。
fn resolve_pos(
    def: &DefIR,
    props: &[Prop],
    params: &[(String, ParamVal)],
) -> Result<Vec<CVal>, LowerError> {
    let pr = prop_of(props, "position")
        .ok_or_else(|| at(&def.module, def.line, "position", "missing 'position' prop"))?;
    let es = prop_exprs(pr);
    if es.len() != 2 {
        return Err(at(
            &def.module,
            def.line,
            "position",
            "'position' must have 2 components",
        ));
    }
    es.iter().map(|e| resolve_cval(def, e, params)).collect()
}

fn cval_value(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    v: &CVal,
    inst: Option<&InstanceIR>,
) -> Result<f64, LowerError> {
    match v {
        CVal::Num(n) => Ok(*n),
        CVal::Ref(f) => {
            let inst = inst.ok_or_else(|| {
                at(
                    &def.module,
                    def.line,
                    f,
                    "model ref outside an instance context",
                )
            })?;
            model_init_value(project, world, def, f, inst)
        }
    }
}

/// shape2d 几何：(kind, w, h)。`kind: circle|quad` + size，或 `mesh` 资源
/// （直接量或 Mesh2D 参数）。
fn resolve_geom(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    props: &[Prop],
    params: &[(String, ParamVal)],
    inst: Option<&InstanceIR>,
) -> Result<(String, f64, f64), LowerError> {
    if let Some(m) = prop_of(props, "mesh") {
        let name = match &m.value {
            PropValue::One(Expr::Dot(outer, r)) if matches!(&**outer, Expr::Ident(b) if b == "resources") => {
                r.to_string()
            }
            PropValue::One(Expr::Dot(outer, r)) if matches!(&**outer, Expr::Ident(b) if b == "self") => {
                match params.iter().find(|(n, _)| n == r.as_str()).map(|(_, v)| v) {
                    Some(ParamVal::Res(r)) => r.clone(),
                    Some(ParamVal::Num(_)) => {
                        return Err(at(
                            &def.module,
                            def.line,
                            "mesh",
                            "mesh param must be a Mesh2D resource",
                        ))
                    }
                    None => {
                        return Err(at(&def.module, def.line, r.as_str(), "unknown mesh param"))
                    }
                }
            }
            PropValue::One(Expr::Ident(r)) => {
                match params.iter().find(|(n, _)| n == r.as_str()).map(|(_, v)| v) {
                    Some(ParamVal::Res(r)) => r.clone(),
                    _ => return Err(at(&def.module, def.line, r.as_str(), "unknown mesh param")),
                }
            }
            _ => {
                return Err(at(
                    &def.module,
                    def.line,
                    "mesh",
                    "mesh must be resources.<name> or a Mesh2D param",
                ))
            }
        };
        return resource_mesh(project, &name);
    }
    let kind = match prop_of(props, "kind").map(|p| &p.value) {
        Some(PropValue::One(Expr::Ident(k))) => k.to_string(),
        _ => {
            return Err(at(
                &def.module,
                def.line,
                "kind",
                "shape2d needs kind: circle|quad or mesh: <Mesh2D>",
            ))
        }
    };
    if kind != "circle" && kind != "quad" {
        return Err(at(
            &def.module,
            def.line,
            &kind,
            "shape2d kind must be circle|quad",
        ));
    }
    let size = prop_of(props, "size")
        .ok_or_else(|| at(&def.module, def.line, "size", "shape2d missing size"))?;
    let es = prop_exprs(size);
    if es.len() != 2 {
        return Err(at(
            &def.module,
            def.line,
            "size",
            "'size' must have 2 components",
        ));
    }
    let sx = cval_value(
        project,
        world,
        def,
        &resolve_cval(def, &es[0], params)?,
        inst,
    )?;
    let sy = cval_value(
        project,
        world,
        def,
        &resolve_cval(def, &es[1], params)?,
        inst,
    )?;
    if kind == "circle" && sx != sy {
        return Err(at(
            &def.module,
            def.line,
            "size",
            &format!("shape2d circle requires size.x==size.y (diameter), got ({sx},{sy})"),
        ));
    }
    Ok((kind, sx, sy))
}

/// material 资源名解析（直接量或 Material2D 参数）。
fn resolve_material(
    def: &DefIR,
    props: &[Prop],
    params: &[(String, ParamVal)],
) -> Result<String, LowerError> {
    let m = prop_of(props, "material").ok_or_else(|| {
        at(
            &def.module,
            def.line,
            "material",
            "shape2d missing material",
        )
    })?;
    match &m.value {
        PropValue::One(Expr::Dot(outer, r)) if matches!(&**outer, Expr::Ident(b) if b == "resources") => {
            Ok(r.to_string())
        }
        PropValue::One(Expr::Dot(outer, r)) if matches!(&**outer, Expr::Ident(b) if b == "self") => {
            match params.iter().find(|(n, _)| n == r.as_str()).map(|(_, v)| v) {
                Some(ParamVal::Res(r)) => Ok(r.clone()),
                _ => Err(at(
                    &def.module,
                    def.line,
                    "material",
                    "material param must be a Material2D resource",
                )),
            }
        }
        PropValue::One(Expr::Ident(r)) => {
            match params.iter().find(|(n, _)| n == r.as_str()).map(|(_, v)| v) {
                Some(ParamVal::Res(r)) => Ok(r.clone()),
                _ => Err(at(
                    &def.module,
                    def.line,
                    r.as_str(),
                    "material must be resources.<name> or a Material2D param",
                )),
            }
        }
        _ => Err(at(
            &def.module,
            def.line,
            "material",
            "material must be resources.<name> or a Material2D param",
        )),
    }
}

/// visible 求值：bool 字面量 / 参数 / model 引用。
fn resolve_visible(
    def: &DefIR,
    props: &[Prop],
    params: &[(String, ParamVal)],
) -> Result<CVal, LowerError> {
    match prop_of(props, "visible") {
        None => Ok(CVal::Num(1.0)),
        Some(pr) => match &pr.value {
            PropValue::One(Expr::Bool(b)) => Ok(CVal::Num(if *b { 1.0 } else { 0.0 })),
            PropValue::One(e) => resolve_cval(def, e, params),
            _ => Err(at(
                &def.module,
                def.line,
                "visible",
                "visible must be a bool/param/model ref",
            )),
        },
    }
}

// ------------------------------------------------------ scene build ----

pub(crate) struct SceneBuild {
    pub(crate) rows: Vec<SceneNode>,
    pub(crate) char: Option<(usize, String)>,
    pub(crate) solid_ids: Vec<usize>,
    pub(crate) camera: Option<CameraCfg>,
    /// app_start spawn 行（声明先序）。
    pub(crate) spawns: Vec<String>,
    /// PLAN-021：顶层实例名 → primary 静态行 ordinal（引用解析表；
    /// character 行可寻址但位置只读）。
    refs: BTreeMap<String, RefRow>,
    /// PLAN-021：定义名 → [(def_idx, primary 静态行 ordinal)]（self 行
    /// 寻址数组 `{prefix}_qrow` 的来源；character 实例=0，无静态行=-1）。
    self_rows: BTreeMap<String, Vec<(usize, i64)>>,
    /// PLAN-022：render2d.spawn 元素名 → (slot, 模板行 id)（旧档为空）。
    spawn_sites: BTreeMap<String, (i64, i64)>,
    /// PLAN-022：场景 assets JSON 条目（png 身份；旧档为空）。
    assets_json: Vec<String>,
    /// PLAN-022：被引用图像名（帧表数组生成面，声明序去重）。
    used_images: Vec<String>,
}

/// 表达式 → 生成文本（共享 translate；ctx 决定 `.field`/emit/服务映射）。
fn expr_text(e: &Expr, ctx: &Ctx) -> Result<String, LowerError> {
    let mut buf = String::new();
    translate_stmts(&[Stmt::Expr(e.clone())], ctx, &mut buf, 0)?;
    Ok(buf.trim().to_string())
}

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

/// 相机配置解析（target refs.<inst>.actor + bounds + view_size）。
fn camera_cfg_components(
    project: &Project,
    world: &WorldIR,
    props: &[Prop],
    char_name: &str,
    char_path_name: &str,
) -> Result<CameraCfg, LowerError> {
    let root_def = &world.defs[&world.root];
    let mut target: Option<String> = None;
    let mut bounds: Option<(f64, f64, f64, f64)> = None;
    let mut view: Option<(f64, f64)> = None;
    for pr in props {
        if pr.name == "target" {
            let name = match &pr.value {
                PropValue::One(Expr::Dot(inner, field)) => {
                    if field.as_str() != "actor" {
                        return Err(at(
                            &root_def.module,
                            root_def.line,
                            "target",
                            "camera2d target must be refs.<char>.actor",
                        ));
                    }
                    match &**inner {
                        Expr::Dot(i2, n) => match &**i2 {
                            Expr::Ident(base) if base == "refs" => n.to_string(),
                            _ => {
                                return Err(at(
                                    &root_def.module,
                                    root_def.line,
                                    "target",
                                    "camera2d target must be refs.<char>.actor",
                                ))
                            }
                        },
                        _ => {
                            return Err(at(
                                &root_def.module,
                                root_def.line,
                                "target",
                                "camera2d target must be refs.<char>.actor",
                            ))
                        }
                    }
                }
                _ => {
                    return Err(at(
                        &root_def.module,
                        root_def.line,
                        "target",
                        "camera2d target must be refs.<char>.actor",
                    ))
                }
            };
            target = Some(name);
            continue;
        }
        let num = |e: &Expr| -> Result<f64, LowerError> {
            numeric_lit(e).ok_or_else(|| {
                at(
                    &root_def.module,
                    root_def.line,
                    &pr.name,
                    "camera prop must be numeric",
                )
            })
        };
        match (pr.name.as_str(), &pr.value) {
            ("bounds", PropValue::Pair(es)) if es.len() == 4 => {
                bounds = Some((num(&es[0])?, num(&es[1])?, num(&es[2])?, num(&es[3])?));
            }
            ("view_size", PropValue::Pair(es)) if es.len() == 2 => {
                view = Some((num(&es[0])?, num(&es[1])?));
            }
            (other, _) => {
                return Err(at(
                    &root_def.module,
                    root_def.line,
                    other,
                    "unknown camera2d prop (components-2d: target/bounds/view_size)",
                ))
            }
        }
        let _ = project;
    }
    let target = target.ok_or_else(|| {
        at(
            &root_def.module,
            root_def.line,
            "target",
            "camera2d missing target",
        )
    })?;
    let (min_x, min_y, max_x, max_y) = bounds.ok_or_else(|| {
        at(
            &root_def.module,
            root_def.line,
            "bounds",
            "camera2d missing bounds",
        )
    })?;
    let (view_w, view_h) = view.ok_or_else(|| {
        at(
            &root_def.module,
            root_def.line,
            "view_size",
            "camera2d missing view_size",
        )
    })?;
    if min_x >= max_x || min_y >= max_y || view_w <= 0.0 || view_h <= 0.0 {
        return Err(at(
            &root_def.module,
            root_def.line,
            "bounds",
            "camera2d bounds/view_size invalid",
        ));
    }
    // 目标匹配：角色全名（instance.element）或实例路径名 / root 元素名。
    let matched = target == char_name || target == char_path_name;
    if !matched {
        return Err(at(
            &root_def.module,
            root_def.line,
            "target",
            &format!("camera2d target '{target}' must reference the character2d '{char_name}'"),
        ));
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

/// 收集场景行 + app_start spawn 行。角色行 id=1，其余按声明序。
pub(crate) fn collect_scene(
    project: &Project,
    world: &WorldIR,
    plan: &Plan,
) -> Result<SceneBuild, LowerError> {
    let _ = plan;
    let mut rows: Vec<SceneNode> = Vec::new();
    let mut next_id = 1usize;
    let mut spawns: Vec<String> = Vec::new();
    let mut slot: usize = 0;
    let mut refs: BTreeMap<String, RefRow> = BTreeMap::new();
    let mut self_rows: BTreeMap<String, Vec<(usize, i64)>> = BTreeMap::new();

    let root_def = &world.defs[&world.root];
    let root_decl = node_of(project, &world.root);
    let root_view = root_decl.view2d.as_ref().expect("root view");
    let root_params: Vec<(String, ParamVal)> = Vec::new();

    // ---- 角色宿主（root 直属或恰一实例定义）----
    let mut char: Option<(usize, String)> = None;
    let mut char_path_name = String::new();
    let char_host = if root_def.char_elem.is_some() {
        Some((world.root.clone(), String::new()))
    } else {
        world
            .defs
            .iter()
            .find(|(n, d)| d.char_elem.is_some() && *n != &world.root)
            .map(|(n, _)| {
                let inst = world
                    .instances
                    .iter()
                    .find(|i| i.def == *n)
                    .expect("single instance");
                (n.clone(), inst.path.clone())
            })
    };
    if let Some((host_def_name, host_path)) = char_host {
        let host_def = &world.defs[&host_def_name];
        let (elem, _) = host_def.char_elem.clone().unwrap();
        let decl = node_of(project, &host_def_name);
        let view = decl.view2d.as_ref().expect("view");
        let el = view.elements.iter().find_map(|e| match e {
            View2dElement::Character { name, props } if name == &elem => Some(props.clone()),
            _ => None,
        });
        let Some(props) = el else {
            return Err(at(
                &host_def.module,
                host_def.line,
                &elem,
                "character2d props missing",
            ));
        };
        let iparams: Vec<(String, ParamVal)> = if host_path.is_empty() {
            root_params.clone()
        } else {
            world
                .instances
                .iter()
                .find(|i| i.path == host_path)
                .map(|i| i.params.clone())
                .unwrap_or_default()
        };
        let pos = resolve_pos(host_def, &props, &iparams)?;
        let tx = cval_value(project, world, host_def, &pos[0], None)?;
        let ty = cval_value(project, world, host_def, &pos[1], None)?;
        let material = resolve_material(host_def, &props, &iparams)?;
        let rgba = resource_color(project, &material)?;
        let name = if host_path.is_empty() {
            elem.clone()
        } else {
            format!("{}.{}", host_path.replace('/', "."), elem)
        };
        char_path_name = if host_path.is_empty() {
            elem.clone()
        } else {
            host_path.clone()
        };
        rows.push(SceneNode {
            is_group: false,
            id: next_id,
            name: name.clone(),
            kind: "quad".into(),
            tx,
            ty,
            sx: 0.8,
            sy: 1.0,
            rgba,
            visible: true,
            parent: None,
            is_character: true,
            is_solid: false,
            sprite: None,
        });
        char = Some((next_id, name));
        next_id += 1;
        if !host_path.is_empty() {
            refs.insert(
                host_path.clone(),
                RefRow {
                    row: 0,
                    writable: false,
                },
            );
            if let Some(inst) = world.instances.iter().find(|i| i.path == host_path) {
                self_rows
                    .entry(host_def_name.clone())
                    .or_default()
                    .push((inst.def_idx, 0));
            }
        }
    }

    // ---- root 静态 solid2d（碰撞 + 可见）----
    let mut solid_ids: Vec<usize> = Vec::new();
    for sname in &world.solids {
        let el = root_view.elements.iter().find_map(|e| match e {
            View2dElement::Solid { name, props } if name == sname => Some(props.clone()),
            _ => None,
        });
        let Some(props) = el else {
            return Err(at(
                &root_def.module,
                root_def.line,
                sname,
                "solid2d props missing",
            ));
        };
        let pos = resolve_pos(root_def, &props, &root_params)?;
        let tx = cval_value(project, world, root_def, &pos[0], None)?;
        let ty = cval_value(project, world, root_def, &pos[1], None)?;
        let (kind, sx, sy) = resolve_geom(project, world, root_def, &props, &root_params, None)?;
        let material = resolve_material(root_def, &props, &root_params)?;
        let rgba = resource_color(project, &material)?;
        rows.push(SceneNode {
            is_group: false,
            id: next_id,
            name: sname.clone(),
            kind,
            tx,
            ty,
            sx,
            sy,
            rgba,
            visible: true,
            parent: None,
            is_character: false,
            is_solid: true,
            sprite: None,
        });
        solid_ids.push(next_id);
        next_id += 1;
    }

    // 相机（校验 target 指向角色）。
    let camera = match &world.camera {
        Some(props) => {
            let (cname, cpn) = match &char {
                Some((_, n)) => (n.clone(), char_path_name.clone()),
                None => {
                    return Err(at(
                        &root_def.module,
                        root_def.line,
                        "camera2d",
                        "components-2d camera2d requires a declared character2d (root view or a single-instance component)",
                    ))
                }
            };
            Some(camera_cfg_components(project, world, props, &cname, &cpn)?)
        }
        None => None,
    };

    // ---- root 视图静态 shape2d 与动态 shape2d（root model 绑定）----
    let mut spawn = |rows: &mut Vec<SceneNode>,
                     next_id: &mut usize,
                     slot: &mut usize,
                     handle: &str,
                     indexed: bool,
                     idx: usize,
                     kind: String,
                     w: f64,
                     h: f64,
                     rgba: (f64, f64, f64, f64),
                     x_text: String,
                     y_text: String,
                     template_idx: usize,
                     name: String| {
        // 模板行（停靠玩法区外，不可见；保身份）。
        rows.push(SceneNode {
            is_group: false,
            id: *next_id,
            name,
            kind,
            tx: -30.0,
            ty: -40.0 - 2.0 * template_idx as f64,
            sx: w,
            sy: h,
            rgba,
            visible: false,
            parent: None,
            is_character: false,
            is_solid: false,
            sprite: None,
        });
        let template = *next_id;
        *next_id += 1;
        let lhs = if indexed {
            format!("{handle}[{idx}]")
        } else {
            handle.to_string()
        };
        spawns.push(format!(
            " {lhs}=ag_object_spawn({slot},{template},{x},{y},0.0)",
            x = x_text,
            y = y_text
        ));
        *slot += 1;
    };

    let root_dyn = root_def.dyn_elems.clone();
    for el in &root_view.elements {
        match el {
            View2dElement::Shape { name, props } => {
                if root_dyn.contains(name) {
                    let (kind, w, h) =
                        resolve_geom(project, world, root_def, props, &root_params, None)?;
                    let material = resolve_material(root_def, props, &root_params)?;
                    let rgba = resource_color(project, &material)?;
                    let ctx = Ctx {
                        target: Target::Root,
                        emit_hook: Box::new(|ev: &str| {
                            Err(at(
                                &root_def.module,
                                root_def.line,
                                ev,
                                "emit inside a view prop is not expressible",
                            ))
                        }),
                        enum_path: Box::new(|e: &str, v: &str| enum_lookup(project, e, v)),
                        axis_differential: true,
                        member_hook: None,
                        moved_flag: None,
                        refs_any_elem: true,
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
                    };
                    let pos = prop_of(props, "position").ok_or_else(|| {
                        at(
                            &root_def.module,
                            root_def.line,
                            "position",
                            "missing position",
                        )
                    })?;
                    let es = prop_exprs(pos);
                    let x_text = expr_text(&es[0], &ctx)?;
                    let y_text = expr_text(&es[1], &ctx)?;
                    // 模板名 = 身份：单动态元素保持 {elem}_template 形态；
                    // 多动态元素按元素消歧（唯一名）。
                    let tmpl_name = if root_def.dyn_elems.len() == 1 {
                        format!("{name}_template")
                    } else {
                        format!("main_{}_template", name.to_lowercase())
                    };
                    spawn(
                        &mut rows,
                        &mut next_id,
                        &mut slot,
                        "main_handle",
                        false,
                        0,
                        kind,
                        w,
                        h,
                        rgba,
                        x_text,
                        y_text,
                        0,
                        tmpl_name,
                    );
                } else if !world.solids.contains(name) {
                    let pos = resolve_pos(root_def, props, &root_params)?;
                    let tx = cval_value(project, world, root_def, &pos[0], None)?;
                    let ty = cval_value(project, world, root_def, &pos[1], None)?;
                    let (kind, sx, sy) =
                        resolve_geom(project, world, root_def, props, &root_params, None)?;
                    let material = resolve_material(root_def, props, &root_params)?;
                    let rgba = resource_color(project, &material)?;
                    let vis = resolve_visible(root_def, props, &root_params)?;
                    let visible = cval_value(project, world, root_def, &vis, None)? > 0.5;
                    rows.push(SceneNode {
                        is_group: false,
                        id: next_id,
                        name: name.clone(),
                        kind,
                        tx,
                        ty,
                        sx,
                        sy,
                        rgba,
                        visible,
                        parent: None,
                        is_character: false,
                        is_solid: false,
                        sprite: None,
                    });
                    next_id += 1;
                }
            }
            _ => {}
        }
    }

    // ---- 每实例（声明先序）：静态克隆 + 动态模板 ----
    for inst in &world.instances {
        let def = &world.defs[&inst.def];
        let decl = node_of(project, &inst.def);
        let view = decl.view2d.as_ref().expect("view");
        let dyn_elems = def.dyn_elems.clone();
        let prefix = def.prefix();
        let single_dyn = dyn_elems.len() == 1;
        for el in &view.elements {
            match el {
                View2dElement::Shape { name, props } => {
                    if dyn_elems.contains(name) {
                        let (kind, w, h) =
                            resolve_geom(project, world, def, props, &inst.params, Some(inst))?;
                        let material = resolve_material(def, props, &inst.params)?;
                        let rgba = resource_color(project, &material)?;
                        let ctx = instance_ctx(
                            project,
                            plan,
                            def,
                            &prefix,
                            &inst.def_idx.to_string(),
                            None,
                            None,
                            None,
                        )?;
                        let pos = prop_of(props, "position").ok_or_else(|| {
                            at(&def.module, def.line, "position", "missing position")
                        })?;
                        let es = prop_exprs(pos);
                        let x_text = expr_text(&es[0], &ctx)?;
                        let y_text = expr_text(&es[1], &ctx)?;
                        let handle = if single_dyn {
                            format!("{prefix}_handle")
                        } else {
                            format!("{prefix}_{name}_handle")
                        };
                        spawn(
                            &mut rows,
                            &mut next_id,
                            &mut slot,
                            &handle,
                            true,
                            inst.def_idx,
                            kind,
                            w,
                            h,
                            rgba,
                            x_text,
                            y_text,
                            inst.def_idx,
                            // 模板名 = 身份（唯一）：单动态元素保持探针冻结
                            // 形态 {path}_template；多动态元素按元素消歧。
                            if single_dyn {
                                format!("{}_template", inst.path.replace('/', "_"))
                            } else {
                                format!(
                                    "{}_{}_template",
                                    inst.path.replace('/', "_"),
                                    name.to_lowercase()
                                )
                            },
                        );
                    } else {
                        let pos = resolve_pos(def, props, &inst.params)?;
                        let tx = cval_value(project, world, def, &pos[0], Some(inst))?;
                        let ty = cval_value(project, world, def, &pos[1], Some(inst))?;
                        let (kind, sx, sy) =
                            resolve_geom(project, world, def, props, &inst.params, Some(inst))?;
                        let material = resolve_material(def, props, &inst.params)?;
                        let rgba = resource_color(project, &material)?;
                        let vis = resolve_visible(def, props, &inst.params)?;
                        let visible = cval_value(project, world, def, &vis, Some(inst))? > 0.5;
                        rows.push(SceneNode {
                            is_group: false,
                            id: next_id,
                            name: format!("{}.{}", inst.path.replace('/', "."), name),
                            kind,
                            tx,
                            ty,
                            sx,
                            sy,
                            rgba,
                            visible,
                            parent: None,
                            is_character: false,
                            is_solid: false,
                            sprite: None,
                        });
                        // PLAN-021：顶层实例 primary 静态行（首个静态
                        // shape；character 定义实例不在此分支）。
                        if !inst.path.contains('/') {
                            refs.entry(inst.path.clone()).or_insert(RefRow {
                                row: (next_id - 1) as i64,
                                writable: def.char_elem.is_none(),
                            });
                        }
                        {
                            let e = self_rows.entry(inst.def.clone()).or_default();
                            if !e.iter().any(|(idx, _)| *idx == inst.def_idx) {
                                e.push((inst.def_idx, (next_id - 1) as i64));
                            }
                        }
                        next_id += 1;
                    }
                }
                _ => {}
            }
        }
    }

    if rows.len() > 128 {
        return Err(at(
            &world.root_module,
            0,
            &world.root,
            &format!(
                "components-2d scene exceeds 128 nodes ({} rows)",
                rows.len()
            ),
        ));
    }

    Ok(SceneBuild {
        rows,
        char,
        solid_ids,
        camera,
        spawns,
        refs,
        self_rows,
        spawn_sites: BTreeMap::new(),
        assets_json: Vec::new(),
        used_images: Vec::new(),
    })
}

/// 实例处理器/视图表达式的翻译上下文（SoA 索引 + 投递钩子）。
/// PLAN-021：queries-2d 环境构造参数（None = 旧档行为）。
pub struct QueryParts<'a> {
    pub refs: &'a BTreeMap<String, RefRow>,
    /// 本实例 self 行寻址表达式（`{prefix}_qrow[i]`；root=None）。
    pub self_row: Option<String>,
    pub self_writable: bool,
    pub binds: &'a std::cell::RefCell<Vec<(String, QueryKind)>>,
}

/// PLAN-022 sprites-2d 场景收集：声明序 = 文档序（作者显式层序），图像
/// 绑定行携带场景 sprite 字段（默认 frame/flip 来自声明初值），动态图像
/// 元素走模板+spawn+逐 tick slot 写，render2d.spawn 站点分配冻结槽预算
/// 内的槽位。
fn collect_scene_sprites(
    project: &Project,
    world: &WorldIR,
    plan: &Plan,
) -> Result<SceneBuild, LowerError> {
    let mut rows: Vec<SceneNode> = Vec::new();
    let mut next_id = 1usize;
    let mut spawns: Vec<String> = Vec::new();
    let mut slot: usize = 0;
    let mut solid_ids: Vec<usize> = Vec::new();
    let mut char: Option<(usize, String)> = None;
    let mut camera: Option<CameraCfg> = None;
    let mut leaf_rows: BTreeMap<String, usize> = BTreeMap::new();
    let mut spawn_sites: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    let mut used_images: Vec<String> = Vec::new();
    let mut used = |img: &str, used_images: &mut Vec<String>| {
        if !used_images.iter().any(|i| i == img) {
            used_images.push(img.to_string());
        }
    };

    let root_def = &world.defs[&world.root];
    let root_decl = node_of(project, &world.root);
    let root_view = root_decl.view2d.as_ref().expect("root view");

    // sprite 元素约束：每定义至多一个（char 或 shape）；root 直属 char 与
    // 实例 char 的既有归属规则不变。
    for (dname, d) in &world.defs {
        if d.sprite_elems.len() > 1 {
            return Err(at(
                &d.module,
                d.line,
                &d.sprite_elems[1].elem,
                &format!(
                    "definition '{dname}' declares {} image elements (sprites-2d: exactly one per definition)",
                    d.sprite_elems.len()
                ),
            ));
        }
        let dynamic_sprites = d
            .sprite_elems
            .iter()
            .filter(|e| {
                matches!(e.frame, FrameBind::Field(_)) || matches!(e.flip, FrameBind::Field(_))
            })
            .count();
        if d.dyn_elems.len() + dynamic_sprites > 1 {
            return Err(at(
                &d.module,
                d.line,
                &d.name,
                "definition mixes a dynamic shape with a dynamic image element (sprites-2d: one dynamic element per definition)",
            ));
        }
    }

    // 单实例展开（声明序：静态克隆行 / 静态图像行 / 动态图像模板+spawn）。
    // 返回该实例的主静态行 id（refs/qrow 面；无静态行=None）。
    fn emit_instance(
        project: &Project,
        world: &WorldIR,
        rows: &mut Vec<SceneNode>,
        next_id: &mut usize,
        spawns: &mut Vec<String>,
        slot: &mut usize,
        solid_ids: &mut Vec<usize>,
        used_images: &mut Vec<String>,
        leaf_rows: &mut BTreeMap<String, usize>,
        def_name: &str,
        inst: Option<&InstanceIR>,
        params: &[(String, ParamVal)],
        def_idx_for_arrays: Option<usize>,
        depth: usize,
    ) -> Result<Option<usize>, LowerError> {
        if depth > 16 {
            return Err(at(
                &world.root_module,
                0,
                def_name,
                "component nesting deeper than 16",
            ));
        }
        let def = &world.defs[def_name];
        let decl = node_of(project, def_name);
        let view = decl.view2d.as_ref().expect("view");
        let mut primary: Option<usize> = None;
        for el in &view.elements {
            match el {
                View2dElement::Shape { name, props } => {
                    let sprite = def.sprite_elems.iter().find(|s| &s.elem == name);
                    if let Some(se) = sprite {
                        let image = if let Some(p) = &se.image_param {
                            let inst = inst.ok_or_else(|| {
                                at(
                                    &def.module,
                                    def.line,
                                    p,
                                    "image param outside an instance context",
                                )
                            })?;
                            match inst.params.iter().find(|(n, _)| n == p).map(|(_, v)| v) {
                                Some(ParamVal::Res(r)) => r.clone(),
                                _ => {
                                    return Err(at(
                                        &def.module,
                                        inst.line,
                                        p,
                                        "image param binding is not an image2d resource",
                                    ))
                                }
                            }
                        } else {
                            se.image.clone()
                        };
                        let Some((sha, bytes_len, _, _)) = world.images.get(&image) else {
                            return Err(at(
                                &def.module,
                                se.line,
                                &se.elem,
                                &format!("image element '{}' binds '{image}' which is not an image2d resource", se.elem),
                            ));
                        };
                        let (sha, bytes_len) = (sha.clone(), *bytes_len);
                        let frames = world.frames.get(&image).ok_or_else(|| {
                            at(
                                &def.module,
                                se.line,
                                &se.elem,
                                &format!("image element '{}' binds image '{image}' which declares no frame table", se.elem),
                            )
                        })?;
                        let dyn_bind = matches!(se.frame, FrameBind::Field(_))
                            || matches!(se.flip, FrameBind::Field(_));
                        let ordinal = default_ordinal_pub(project, world, def, &se.frame, inst)?;
                        let flip = default_ordinal_pub(project, world, def, &se.flip, inst)? != 0;
                        if ordinal as usize >= frames.len() {
                            return Err(at(
                                &def.module,
                                se.line,
                                &se.elem,
                                &format!(
                                    "default frame ordinal {ordinal} exceeds the frame table of image '{image}' ({} frame(s))",
                                    frames.len()
                                ),
                            ));
                        }
                        let rect = frames[ordinal as usize].1;
                        let pos = resolve_pos(def, props, params)?;
                        let tx = cval_value_pub(project, world, def, &pos[0], inst)?;
                        let ty = cval_value_pub(project, world, def, &pos[1], inst)?;
                        let (sx, sy) = resolve_size_pub(project, world, def, props, params, inst)?;
                        let visible = match prop_of(props, "visible") {
                            None => true,
                            Some(p) => {
                                cval_value_pub(
                                    project,
                                    world,
                                    def,
                                    &resolve_visible(def, props, params)?,
                                    inst,
                                )? > 0.5
                            }
                        };
                        let _ = visible;
                        if !used_images.iter().any(|i| i == &image) {
                            used_images.push(image.clone());
                        }
                        let name = match inst {
                            Some(i) => format!("{}.{}", i.path.replace('/', "."), name),
                            None => name.clone(),
                        };
                        if dyn_bind {
                            // 模板行（停靠玩法区外，不可见；身份 = 默认绑定）。
                            rows.push(SceneNode {
                                is_group: false,
                                id: *next_id,
                                name: format!("{}_template", name.replace('.', "_")),
                                kind: "quad".into(),
                                tx: -30.0,
                                ty: -40.0 - 2.0 * (*slot as f64),
                                sx,
                                sy,
                                rgba: (0.9, 0.3, 0.2, 1.0),
                                visible: false,
                                parent: None,
                                is_character: false,
                                is_solid: false,
                                sprite: Some(crate::lower_basic2d::SceneSprite {
                                    asset: sha.clone(),
                                    frame: rect,
                                    flip,
                                }),
                            });
                            let template = *next_id;
                            *next_id += 1;
                            let handle = format!("{}_handle", def.prefix());
                            let lhs = match def_idx_for_arrays {
                                Some(i) => format!("{handle}[{i}]"),
                                None => handle,
                            };
                            spawns.push(format!(
                                " {lhs}=ag_object_spawn({slot},{template},{},{},0.0)",
                                crate::lower_basic2d::fmt_f(tx),
                                crate::lower_basic2d::fmt_f(ty)
                            ));
                            *slot += 1;
                        } else {
                            rows.push(SceneNode {
                                is_group: false,
                                id: *next_id,
                                name: name.clone(),
                                kind: "quad".into(),
                                tx,
                                ty,
                                sx,
                                sy,
                                rgba: (0.9, 0.3, 0.2, 1.0),
                                visible: true,
                                parent: None,
                                is_character: false,
                                is_solid: false,
                                sprite: Some(crate::lower_basic2d::SceneSprite {
                                    asset: sha,
                                    frame: rect,
                                    flip,
                                }),
                            });
                            if def_idx_for_arrays.is_none() {
                                leaf_rows.insert(se.elem.clone(), *next_id);
                            }
                            if primary.is_none() {
                                primary = Some(*next_id);
                            }
                            *next_id += 1;
                        }
                    } else if def.dyn_elems.contains(name) {
                        // 非 image 动态 shape：sprites 档拒绝（一个动态元素
                        // 约束下未列支持面）。
                        return Err(at(
                            &def.module,
                            def.line,
                            name,
                            "dynamic non-image shape2d is out of the sprites-2d scope (image elements drive the per-tick visual)",
                        ));
                    } else {
                        let pos = resolve_pos(def, props, params)?;
                        let tx = cval_value_pub(project, world, def, &pos[0], inst)?;
                        let ty = cval_value_pub(project, world, def, &pos[1], inst)?;
                        let (kind, sx, sy) =
                            resolve_geom_pub(project, world, def, props, params, inst)?;
                        let material = resolve_material(def, props, params)?;
                        let rgba = resource_color(project, &material)?;
                        let visible = cval_value_pub(
                            project,
                            world,
                            def,
                            &resolve_visible(def, props, params)?,
                            inst,
                        )? > 0.5;
                        let name = match inst {
                            Some(i) => format!("{}.{}", i.path.replace('/', "."), name),
                            None => name.clone(),
                        };
                        rows.push(SceneNode {
                            is_group: false,
                            id: *next_id,
                            name,
                            kind,
                            tx,
                            ty,
                            sx,
                            sy,
                            rgba,
                            visible,
                            parent: None,
                            is_character: false,
                            is_solid: false,
                            sprite: None,
                        });
                        if primary.is_none() {
                            primary = Some(*next_id);
                        }
                        *next_id += 1;
                    }
                }
                View2dElement::Solid { name, props } => {
                    let pos = resolve_pos(def, props, params)?;
                    let tx = cval_value_pub(project, world, def, &pos[0], None)?;
                    let ty = cval_value_pub(project, world, def, &pos[1], None)?;
                    let (kind, sx, sy) =
                        resolve_geom_pub(project, world, def, props, params, None)?;
                    let material = resolve_material(def, props, params)?;
                    let rgba = resource_color(project, &material)?;
                    rows.push(SceneNode {
                        is_group: false,
                        id: *next_id,
                        name: name.clone(),
                        kind,
                        tx,
                        ty,
                        sx,
                        sy,
                        rgba,
                        visible: true,
                        parent: None,
                        is_character: false,
                        is_solid: true,
                        sprite: None,
                    });
                    solid_ids.push(*next_id);
                    *next_id += 1;
                }
                View2dElement::Instance { node, name, .. } => {
                    let child_inst = world
                        .instances
                        .iter()
                        .find(|i| inst_name_of(&i.path) == name.as_str())
                        .ok_or_else(|| {
                            at(&def.module, def.line, name, "instance row missing in IR")
                        })?;
                    emit_instance(
                        project,
                        world,
                        rows,
                        next_id,
                        spawns,
                        slot,
                        solid_ids,
                        used_images,
                        leaf_rows,
                        node,
                        Some(child_inst),
                        &child_inst.params,
                        Some(child_inst.def_idx),
                        depth + 1,
                    )?;
                }
                _ => {}
            }
        }
        Ok(primary)
    }

    // ---- 根视图声明序发射 ----
    // character 行（root 直属或恰一实例定义）在声明位置发射。
    let char_host: Option<(String, String)> = if root_def.char_elem.is_some() {
        Some((world.root.clone(), String::new()))
    } else {
        world
            .defs
            .iter()
            .find(|(n, d)| d.char_elem.is_some() && *n != &world.root)
            .map(|(n, _)| {
                let inst = world
                    .instances
                    .iter()
                    .find(|i| i.def == *n)
                    .expect("single instance");
                (n.clone(), inst.path.clone())
            })
    };

    let mut emitted_char = false;
    for el in &root_view.elements {
        match el {
            View2dElement::Character { name, props } => {
                // root 直属 character2d（带/不带 image）。
                let sprite = root_def
                    .sprite_elems
                    .iter()
                    .find(|s| s.is_char && &s.elem == name);
                let pos = resolve_pos(root_def, props, &[])?;
                let tx = cval_value_pub(project, world, root_def, &pos[0], None)?;
                let ty = cval_value_pub(project, world, root_def, &pos[1], None)?;
                if let Some(se) = sprite {
                    if se.image_param.is_some() {
                        return Err(at(
                            &root_def.module,
                            se.line,
                            &se.elem,
                            "root view image elements must bind resources.<image> directly (the root has no params)",
                        ));
                    }
                    let image = se.image.clone();
                    let Some((sha, _, _, _)) = world.images.get(&image) else {
                        return Err(at(
                            &root_def.module,
                            se.line,
                            &se.elem,
                            &format!("image element '{}' binds '{image}' which is not an image2d resource", se.elem),
                        ));
                    };
                    let sha = sha.clone();
                    let ordinal = default_ordinal_pub(project, world, root_def, &se.frame, None)?;
                    let flip = default_flip_pub(project, world, root_def, &se.flip, None)?;
                    let frames = world.frames.get(&image).ok_or_else(|| {
                        at(
                            &root_def.module,
                            se.line,
                            &se.elem,
                            &format!("image element '{}' binds image '{image}' which declares no frame table", se.elem),
                        )
                    })?;
                    if ordinal as usize >= frames.len() {
                        return Err(at(
                            &root_def.module,
                            se.line,
                            &se.elem,
                            &format!(
                                "default frame ordinal {ordinal} exceeds the frame table of image '{image}' ({} frame(s))",
                                frames.len()
                            ),
                        ));
                    }
                    let rect = frames[ordinal as usize].1;
                    if !used_images.iter().any(|i| i == &image) {
                        used_images.push(image.clone());
                    }
                    let (sx, sy) = resolve_size_pub(project, world, root_def, props, &[], None)?;
                    rows.push(SceneNode {
                        is_group: false,
                        id: next_id,
                        name: name.clone(),
                        kind: "quad".into(),
                        tx,
                        ty,
                        sx,
                        sy,
                        rgba: (0.9, 0.3, 0.2, 1.0),
                        visible: true,
                        parent: None,
                        is_character: true,
                        is_solid: false,
                        sprite: Some(crate::lower_basic2d::SceneSprite {
                            asset: sha,
                            frame: rect,
                            flip,
                        }),
                    });
                    char = Some((next_id, name.clone()));
                    emitted_char = true;
                    next_id += 1;
                    continue;
                }
                let material = resolve_material(root_def, props, &[])?;
                let rgba = resource_color(project, &material)?;
                rows.push(SceneNode {
                    is_group: false,
                    id: next_id,
                    name: name.clone(),
                    kind: "quad".into(),
                    tx,
                    ty,
                    sx: 0.8,
                    sy: 1.0,
                    rgba,
                    visible: true,
                    parent: None,
                    is_character: true,
                    is_solid: false,
                    sprite: None,
                });
                char = Some((next_id, name.clone()));
                emitted_char = true;
                next_id += 1;
            }
            View2dElement::Instance { node, name, .. } => {
                // 恰一实例定义的 character 宿主：其 char 行在实例展开内发射。
                if let Some((host_def, _)) = &char_host {
                    if host_def == node && !emitted_char {
                        let def = &world.defs[host_def];
                        let inst = world
                            .instances
                            .iter()
                            .find(|i| i.def == *host_def)
                            .expect("single instance");
                        let se = def.sprite_elems.iter().find(|s| s.is_char);
                        let decl = node_of(project, host_def);
                        let view = decl.view2d.as_ref().expect("view");
                        let celel = view.elements.iter().find_map(|e| match e {
                            View2dElement::Character { name, props } => {
                                Some((name.clone(), props.clone()))
                            }
                            _ => None,
                        });
                        let (cname, cprops) = celel.expect("char element");
                        let _ = cname;
                        let pos = resolve_pos(def, &cprops, &inst.params)?;
                        let tx = cval_value_pub(project, world, def, &pos[0], Some(inst))?;
                        let ty = cval_value_pub(project, world, def, &pos[1], Some(inst))?;
                        if let Some(se) = se {
                            let image = if let Some(p) = &se.image_param {
                                match inst.params.iter().find(|(n, _)| n == p).map(|(_, v)| v) {
                                    Some(ParamVal::Res(r)) => r.clone(),
                                    _ => return Err(at(
                                        &def.module,
                                        se.line,
                                        p,
                                        "character image param binding is not an image2d resource",
                                    )),
                                }
                            } else {
                                se.image.clone()
                            };
                            let Some((sha, _, _, _)) = world.images.get(&image) else {
                                return Err(at(
                                    &def.module,
                                    se.line,
                                    &se.elem,
                                    &format!(
                                        "character image '{image}' is not an image2d resource"
                                    ),
                                ));
                            };
                            let sha = sha.clone();
                            let ordinal =
                                default_ordinal_pub(project, world, def, &se.frame, Some(inst))?;
                            let flip = default_flip_pub(project, world, def, &se.flip, Some(inst))?;
                            let frames = world.frames.get(&image).ok_or_else(|| {
                                at(
                                    &def.module,
                                    se.line,
                                    &se.elem,
                                    &format!("character image '{image}' declares no frame table"),
                                )
                            })?;
                            if ordinal as usize >= frames.len() {
                                return Err(at(
                                    &def.module,
                                    se.line,
                                    &se.elem,
                                    &format!(
                                        "default frame ordinal {ordinal} exceeds the frame table of image '{image}' ({} frame(s))",
                                        frames.len()
                                    ),
                                ));
                            }
                            let rect = frames[ordinal as usize].1;
                            if !used_images.iter().any(|i| i == &image) {
                                used_images.push(image.clone());
                            }
                            let (sx, sy) = resolve_size_pub(
                                project,
                                world,
                                def,
                                &cprops,
                                &inst.params,
                                Some(inst),
                            )?;
                            rows.push(SceneNode {
                                is_group: false,
                                id: next_id,
                                name: format!("{}.{}", inst.path.replace('/', "."), "body"),
                                kind: "quad".into(),
                                tx,
                                ty,
                                sx,
                                sy,
                                rgba: (0.9, 0.3, 0.2, 1.0),
                                visible: true,
                                parent: None,
                                is_character: true,
                                is_solid: false,
                                sprite: Some(crate::lower_basic2d::SceneSprite {
                                    asset: sha,
                                    frame: rect,
                                    flip,
                                }),
                            });
                        } else {
                            let material = resolve_material(def, &cprops, &inst.params)?;
                            let rgba = resource_color(project, &material)?;
                            rows.push(SceneNode {
                                is_group: false,
                                id: next_id,
                                name: format!("{}.{}", inst.path.replace('/', "."), "body"),
                                kind: "quad".into(),
                                tx,
                                ty,
                                sx: 0.8,
                                sy: 1.0,
                                rgba,
                                visible: true,
                                parent: None,
                                is_character: true,
                                is_solid: false,
                                sprite: None,
                            });
                        }
                        char = Some((next_id, inst.path.replace('/', ".")));
                        emitted_char = true;
                        next_id += 1;
                        // 继续实例展开（其余元素照常发射；Character 元素在
                        // emit_instance 内为 no-op）。
                    }
                }
                let inst = world
                    .instances
                    .iter()
                    .find(|i| inst_name_of(&i.path) == name.as_str())
                    .ok_or_else(|| {
                        at(
                            &root_def.module,
                            root_def.line,
                            name,
                            "instance missing in IR",
                        )
                    })?;
                emit_instance(
                    project,
                    world,
                    &mut rows,
                    &mut next_id,
                    &mut spawns,
                    &mut slot,
                    &mut solid_ids,
                    &mut used_images,
                    &mut leaf_rows,
                    node,
                    Some(inst),
                    &inst.params,
                    Some(inst.def_idx),
                    0,
                )?;
            }
            View2dElement::Solid { name, props } => {
                let pos = resolve_pos(root_def, props, &[])?;
                let tx = cval_value_pub(project, world, root_def, &pos[0], None)?;
                let ty = cval_value_pub(project, world, root_def, &pos[1], None)?;
                let (kind, sx, sy) = resolve_geom_pub(project, world, root_def, props, &[], None)?;
                let material = resolve_material(root_def, props, &[])?;
                let rgba = resource_color(project, &material)?;
                rows.push(SceneNode {
                    is_group: false,
                    id: next_id,
                    name: name.clone(),
                    kind,
                    tx,
                    ty,
                    sx,
                    sy,
                    rgba,
                    visible: true,
                    parent: None,
                    is_character: false,
                    is_solid: true,
                    sprite: None,
                });
                solid_ids.push(next_id);
                next_id += 1;
            }
            View2dElement::Shape { name, props } => {
                let sprite = root_def.sprite_elems.iter().find(|s| &s.elem == name);
                if let Some(se) = sprite {
                    if se.image_param.is_some() {
                        return Err(at(
                            &root_def.module,
                            se.line,
                            &se.elem,
                            "root view image elements must bind resources.<image> directly (the root has no params)",
                        ));
                    }
                    let image = se.image.clone();
                    let Some((sha, _, _, _)) = world.images.get(&image) else {
                        return Err(at(
                            &root_def.module,
                            se.line,
                            &se.elem,
                            &format!("image element '{}' binds '{image}' which is not an image2d resource", se.elem),
                        ));
                    };
                    let sha = sha.clone();
                    let ordinal = default_ordinal_pub(project, world, root_def, &se.frame, None)?;
                    let flip = default_flip_pub(project, world, root_def, &se.flip, None)?;
                    let frames = world.frames.get(&image).ok_or_else(|| {
                        at(
                            &root_def.module,
                            se.line,
                            &se.elem,
                            &format!("image element '{}' binds image '{image}' which declares no frame table", se.elem),
                        )
                    })?;
                    if ordinal as usize >= frames.len() {
                        return Err(at(
                            &root_def.module,
                            se.line,
                            &se.elem,
                            &format!(
                                "default frame ordinal {ordinal} exceeds the frame table of image '{image}' ({} frame(s))",
                                frames.len()
                            ),
                        ));
                    }
                    let rect = frames[ordinal as usize].1;
                    if !used_images.iter().any(|i| i == &image) {
                        used_images.push(image.clone());
                    }
                    let pos = resolve_pos(root_def, props, &[])?;
                    let tx = cval_value_pub(project, world, root_def, &pos[0], None)?;
                    let ty = cval_value_pub(project, world, root_def, &pos[1], None)?;
                    let (sx, sy) = resolve_size_pub(project, world, root_def, props, &[], None)?;
                    let visible = cval_value_pub(
                        project,
                        world,
                        root_def,
                        &resolve_visible(root_def, props, &[])?,
                        None,
                    )? > 0.5;
                    rows.push(SceneNode {
                        is_group: false,
                        id: next_id,
                        name: name.clone(),
                        kind: "quad".into(),
                        tx,
                        ty,
                        sx,
                        sy,
                        rgba: (0.9, 0.3, 0.2, 1.0),
                        visible,
                        parent: None,
                        is_character: false,
                        is_solid: false,
                        sprite: Some(crate::lower_basic2d::SceneSprite {
                            asset: sha,
                            frame: rect,
                            flip,
                        }),
                    });
                    leaf_rows.insert(se.elem.clone(), next_id);
                    next_id += 1;
                    continue;
                }
                if root_def.dyn_elems.contains(name) {
                    return Err(at(
                        &root_def.module,
                        root_def.line,
                        name,
                        "dynamic non-image shape2d is out of the sprites-2d scope",
                    ));
                }
                let pos = resolve_pos(root_def, props, &[])?;
                let tx = cval_value_pub(project, world, root_def, &pos[0], None)?;
                let ty = cval_value_pub(project, world, root_def, &pos[1], None)?;
                let (kind, sx, sy) = resolve_geom_pub(project, world, root_def, props, &[], None)?;
                let material = resolve_material(root_def, props, &[])?;
                let rgba = resource_color(project, &material)?;
                let visible = cval_value_pub(
                    project,
                    world,
                    root_def,
                    &resolve_visible(root_def, props, &[])?,
                    None,
                )? > 0.5;
                rows.push(SceneNode {
                    is_group: false,
                    id: next_id,
                    name: name.clone(),
                    kind,
                    tx,
                    ty,
                    sx,
                    sy,
                    rgba,
                    visible,
                    parent: None,
                    is_character: false,
                    is_solid: false,
                    sprite: None,
                });
                next_id += 1;
            }
            View2dElement::Camera { props, .. } => {
                let (_, cname) = char.clone().ok_or_else(|| {
                    at(
                        &root_def.module,
                        root_def.line,
                        "camera2d",
                        "sprites-2d camera requires a declared character2d",
                    )
                })?;
                camera = Some(camera_cfg_components(
                    project, world, props, &cname, &cname,
                )?);
            }
            _ => {}
        }
    }

    if !emitted_char {
        return Err(at(
            &root_def.module,
            root_def.line,
            &world.root,
            "sprites-2d requires a declared character2d (root view or a single-instance component)",
        ));
    }

    // ---- render2d.spawn 站点（声明先序；槽预算已在 plan 层核过）----
    let mut site_slots: BTreeMap<String, i64> = BTreeMap::new();
    for (elem, rline) in &plan.render_spawns {
        let Some(row) = leaf_rows.get(elem) else {
            return Err(at(
                &root_def.module,
                *rline,
                elem,
                "render2d.spawn target row missing (not a static image leaf in the root view)",
            ));
        };
        let slot_of = *site_slots.entry(elem.clone()).or_insert_with(|| {
            let s = slot as i64;
            slot += 1;
            s
        });
        spawn_sites.insert(elem.clone(), (slot_of, *row as i64));
    }

    // ---- assets JSON（图像身份；名字序稳定）----
    let mut assets_json: Vec<String> = Vec::new();
    let mut img_names: Vec<&String> = world.images.keys().collect();
    img_names.sort();
    for name in img_names {
        let (sha, bytes, w, h) = &world.images[name];
        assets_json.push(format!(
            "{{\"format\": \"png\", \"sha256\": \"{sha}\", \"byte_length\": {bytes}, \"name\": \"{sha}.png\", \"width\": {w}, \"height\": {h}}}"
        ));
    }

    if rows.len() > 128 {
        return Err(at(
            &world.root_module,
            0,
            &world.root,
            &format!("sprites-2d scene exceeds 128 nodes ({} rows)", rows.len()),
        ));
    }

    Ok(SceneBuild {
        rows,
        char,
        solid_ids,
        camera,
        spawns,
        refs: BTreeMap::new(),
        self_rows: BTreeMap::new(),
        spawn_sites,
        assets_json,
        used_images,
    })
}

/// model 字段初值（帧 ordinal/flip 默认的 Field 形态解析）。
fn model_init_value_of_field(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    field: &str,
    inst: Option<&InstanceIR>,
) -> Result<u32, LowerError> {
    let v = match inst {
        Some(i) => model_init_value(project, world, def, field, i)?,
        None => {
            // root：model 初值只允许数值字面量。
            let decl = node_of(project, &def.name);
            let model = decl.model.as_ref().ok_or_else(|| {
                at(
                    &def.module,
                    def.line,
                    field,
                    "definition has no model block",
                )
            })?;
            let f = model
                .fields
                .iter()
                .find(|f| f.name.as_str() == field)
                .ok_or_else(|| at(&def.module, def.line, field, "unknown model field"))?;
            match &f.init {
                Expr::Int(v) => *v as f64,
                Expr::Float(v, _) => *v,
                other => {
                    return Err(at(
                        &def.module,
                        def.line,
                        field,
                        &format!("root model init must be a numeric literal, got {other:?}"),
                    ))
                }
            }
        }
    };
    if v < 0.0 {
        return Err(at(
            &def.module,
            def.line,
            field,
            "frame ordinal model init must be >= 0",
        ));
    }
    Ok(v as u32)
}

/// collect_scene_sprites 内部发射器使用的 pub 别名（同文件私有 fn 的重导出
/// 形态，避免闭包借用冲突）。
fn default_ordinal_pub(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    bind: &FrameBind,
    inst: Option<&InstanceIR>,
) -> Result<u32, LowerError> {
    match bind {
        FrameBind::Lit(v) => Ok(*v),
        FrameBind::Param(p) => {
            let inst = inst.ok_or_else(|| {
                at(
                    &def.module,
                    def.line,
                    p,
                    "frame/flip param outside an instance context",
                )
            })?;
            match inst.params.iter().find(|(n, _)| n == p).map(|(_, v)| v) {
                Some(ParamVal::Num(v)) => {
                    if *v < 0.0 {
                        return Err(at(&def.module, inst.line, p, "frame ordinal must be >= 0"));
                    }
                    Ok(*v as u32)
                }
                _ => Err(at(
                    &def.module,
                    inst.line,
                    p,
                    "frame ordinal param is not numeric",
                )),
            }
        }
        FrameBind::Field(f) => model_init_value_of_field(project, world, def, f, inst),
    }
}

fn default_flip_pub(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    bind: &FrameBind,
    inst: Option<&InstanceIR>,
) -> Result<bool, LowerError> {
    Ok(default_ordinal_pub(project, world, def, bind, inst)? != 0)
}

fn cval_value_pub(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    v: &CVal,
    inst: Option<&InstanceIR>,
) -> Result<f64, LowerError> {
    cval_value(project, world, def, v, inst)
}

fn resolve_geom_pub(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    props: &[Prop],
    params: &[(String, ParamVal)],
    inst: Option<&InstanceIR>,
) -> Result<(String, f64, f64), LowerError> {
    resolve_geom(project, world, def, props, params, inst)
}

/// PLAN-022：图像元素的显示尺寸（场景对象 scale；w,h > 0）。
#[allow(clippy::type_complexity)]
fn resolve_size_pub(
    project: &Project,
    world: &WorldIR,
    def: &DefIR,
    props: &[Prop],
    params: &[(String, ParamVal)],
    inst: Option<&InstanceIR>,
) -> Result<(f64, f64), LowerError> {
    let size = prop_of(props, "size")
        .ok_or_else(|| at(&def.module, def.line, "size", "image element missing size"))?;
    let es = prop_exprs(size);
    if es.len() != 2 {
        return Err(at(
            &def.module,
            def.line,
            "size",
            "'size' must have 2 components",
        ));
    }
    let sx = cval_value(
        project,
        world,
        def,
        &resolve_cval(def, &es[0], params)?,
        inst,
    )?;
    let sy = cval_value(
        project,
        world,
        def,
        &resolve_cval(def, &es[1], params)?,
        inst,
    )?;
    if sx <= 0.0 || sy <= 0.0 {
        return Err(at(
            &def.module,
            def.line,
            "size",
            &format!("image element size must be positive, got ({sx},{sy})"),
        ));
    }
    Ok((sx, sy))
}

pub(crate) fn instance_ctx<'a>(
    project: &'a Project,
    plan: &'a Plan,
    def: &'a DefIR,
    prefix: &str,
    index: &str,
    query: Option<&'a QueryParts<'a>>,
    sprites: Option<&crate::translate::SpriteEnv<'a>>,
    sfx: Option<&crate::translate::SfxEnv<'a>>,
) -> Result<Ctx<'a>, LowerError> {
    let edef = def.name.clone();
    let errmod = def.module.clone();
    let idx0 = index.to_string();
    let emit_hook = Box::new(move |ev: &str| -> Result<String, LowerError> {
        match plan.anc.get(&(edef.clone(), ev.to_string())) {
            Some(d) => {
                if d.to_root {
                    Ok(format!("{}()", d.fn_name))
                } else {
                    Ok(format!("{}({}[{}])", d.fn_name, d.table, idx0))
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
    let edef = def.name.clone();
    let errmod = def.module.clone();
    let idx1 = index.to_string();
    let child_emit = Box::new(
        move |child: &str, ev: &str, argc: usize| -> Result<String, LowerError> {
            // 头部含分隔符：实参前缀（argc==0 时无尾随逗号；root 直呼带
            // 左括号——PLAN-024 修复，实例投递字节不变）。
            let sep = if argc == 0 { "" } else { ", " };
            if child == "emit" {
                let d = plan
                    .anc
                    .get(&(edef.clone(), ev.to_string()))
                    .ok_or_else(|| {
                        at(
                            &errmod,
                            0,
                            ev,
                            &format!("emit .{ev} has no lowering-time receiver"),
                        )
                    })?;
                return Ok(if d.to_root {
                    format!("{}(", d.fn_name)
                } else {
                    format!("{}({}[{}]{sep}", d.fn_name, d.table, idx1)
                });
            }
            match plan
                .child
                .get(&(edef.clone(), child.to_string(), ev.to_string()))
            {
                Some(d) => Ok(format!("{}({}[{}]{sep}", d.fn_name, d.table, idx1)),
                None => Err(at(
                    &errmod,
                    0,
                    ev,
                    &format!("emit {child}.{ev} has no lowering-time receiver"),
                )),
            }
        },
    );
    let scope_module = def.module.clone();
    let call_scope = Box::new(move |f: &str| -> Option<String> {
        let m = project.modules.get(&scope_module)?;
        if m.fns.iter().any(|(n, _)| n == f) {
            Some(format!("{scope_module}_{f}"))
        } else {
            None
        }
    });
    let qenv = query.map(|q| QueryEnv {
        refs: q.refs,
        self_row: q.self_row.clone(),
        self_writable: q.self_writable,
        binds: q.binds,
    });
    Ok(Ctx {
        target: Target::Instance {
            prefix: prefix.to_string(),
            index: index.to_string(),
        },
        emit_hook,
        enum_path: Box::new(|e: &str, v: &str| enum_lookup(project, e, v)),
        axis_differential: true,
        member_hook: None,
        moved_flag: None,
        refs_any_elem: true,
        call_scope: Some(call_scope),
        child_emit: Some(child_emit),
        query: qenv,
        sprites: sprites.map(|e| crate::translate::SpriteEnv {
            spawns: e.spawns,
            frames: e.frames,
        }),
        space3: None,
        sfx: sfx.map(|e| crate::translate::SfxEnv { cues: e.cues }),
        handle_ref: if def.dyn_elems.len() == 1 {
            Some(format!("{prefix}_handle[{index}]"))
        } else {
            None
        },
        six_actions: false,
        char3: None,
        cam3: false,
            events3d: None,
            bindings3d: None,
    })
}

pub(crate) fn root_ctx<'a>(
    project: &'a Project,
    world: &'a WorldIR,
    plan: &'a Plan,
    query: Option<&'a QueryParts<'a>>,
    sprites: Option<&crate::translate::SpriteEnv<'a>>,
    sfx: Option<&crate::translate::SfxEnv<'a>>,
) -> Ctx<'a> {
    let root = world.root.clone();
    let errmod = world.root_module.clone();
    let emit_hook = Box::new(move |ev: &str| -> Result<String, LowerError> {
        match plan.anc.get(&(root.clone(), ev.to_string())) {
            Some(d) => Ok(format!("{}()", d.fn_name)),
            None => Err(at(
                &errmod,
                0,
                ev,
                &format!("emit .{ev} has no lowering-time receiver"),
            )),
        }
    });
    let root = world.root.clone();
    let errmod = world.root_module.clone();
    let child_emit = Box::new(
        move |child: &str, ev: &str, argc: usize| -> Result<String, LowerError> {
            let sep = if argc == 0 { "" } else { ", " };
            if child == "emit" {
                let d = plan
                    .anc
                    .get(&(root.clone(), ev.to_string()))
                    .ok_or_else(|| {
                        at(
                            &errmod,
                            0,
                            ev,
                            &format!("emit .{ev} has no lowering-time receiver"),
                        )
                    })?;
                return Ok(format!("{}(", d.fn_name));
            }
            match plan
                .child
                .get(&(root.clone(), child.to_string(), ev.to_string()))
            {
                Some(d) => Ok(format!("{}({}{sep}", d.fn_name, d.entries[0])),
                None => Err(at(
                    &errmod,
                    0,
                    ev,
                    &format!("emit {child}.{ev} has no lowering-time receiver"),
                )),
            }
        },
    );
    let scope_module = world.root_module.clone();
    let call_scope = Box::new(move |f: &str| -> Option<String> {
        let m = project.modules.get(&scope_module)?;
        if m.fns.iter().any(|(n, _)| n == f) {
            Some(format!("{scope_module}_{f}"))
        } else {
            None
        }
    });
    let qenv = query.map(|q| QueryEnv {
        refs: q.refs,
        self_row: q.self_row.clone(),
        self_writable: q.self_writable,
        binds: q.binds,
    });
    Ctx {
        target: Target::Root,
        emit_hook,
        enum_path: Box::new(|e: &str, v: &str| enum_lookup(project, e, v)),
        axis_differential: true,
        member_hook: None,
        moved_flag: None,
        refs_any_elem: true,
        call_scope: Some(call_scope),
        child_emit: Some(child_emit),
        query: qenv,
        sprites: sprites.map(|e| crate::translate::SpriteEnv {
            spawns: e.spawns,
            frames: e.frames,
        }),
        space3: None,
        sfx: sfx.map(|e| crate::translate::SfxEnv { cues: e.cues }),
        handle_ref: None,
        six_actions: false,
        char3: None,
        cam3: false,
            events3d: None,
            bindings3d: None,
    }
}

// ------------------------------------------------------- generation ----

/// 定义输出序：首个实例的声明先序（root 除外）。
pub(crate) fn def_order(world: &WorldIR) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for i in &world.instances {
        if !out.contains(&i.def) {
            out.push(i.def.clone());
        }
    }
    out
}

/// 定义处理器 + 动态视图属性引用到的成员名（model 字段 ∪ 运行期参数）。
fn def_runtime_members(project: &Project, def: &DefIR) -> BTreeSet<String> {
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
    if let Some(view) = &decl.view2d {
        for el in &view.elements {
            if let View2dElement::Shape { name, props } = el {
                if def.dyn_elems.contains(name) {
                    for pr in props {
                        for e in prop_exprs(pr) {
                            walk_expr_q(&e, def.line, &mut out, false, false);
                        }
                    }
                }
            }
        }
    }
    out.members
}

/// 运行期参数数组（声明序；handler/动态视图引用到的数值参数）。
fn def_runtime_params(project: &Project, def: &DefIR) -> Vec<(String, PType)> {
    let members = def_runtime_members(project, def);
    // PLAN-030：Actor/msg 引用参数无运行列（下降期内联解析/投递文本）；
    // Slot/Template/Pair 参数保留 int SoA 列（handler `.参数` 读面）。
    // 旧档定义不可能携带引用类型参数（ptypes 拒绝），过滤恒空集。
    let decl = node_of(project, &def.name);
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
        .filter(|(n, pt)| *pt != PType::Res && members.contains(n) && !ref_norun.contains(n))
        .cloned()
        .collect()
}

/// 每定义动态呈现数组面（handle + 位置/可见缓存）。
pub(crate) struct DynArr {
    pub(crate) elem: String,
    pub(crate) handle: String,
    pub(crate) px: Option<String>,
    pub(crate) py: Option<String>,
    pub(crate) vis: Option<String>,
}

pub(crate) fn def_dyn_arrays(
    project: &Project,
    def: &DefIR,
    prefix: &str,
    soa_prefix: &str,
    vis_suffix: &str,
) -> Result<Vec<DynArr>, LowerError> {
    let mut out = Vec::new();
    let single = def.dyn_elems.len() == 1;
    let decl = node_of(project, &def.name);
    let view = decl.view2d.as_ref().expect("view");
    for elem in &def.dyn_elems {
        let props = view
            .elements
            .iter()
            .find_map(|e| match e {
                View2dElement::Shape { name, props } if name == elem => Some(props.clone()),
                _ => None,
            })
            .ok_or_else(|| at(&def.module, def.line, elem, "dynamic shape props missing"))?;
        let base = if single {
            prefix.to_string()
        } else {
            format!("{prefix}_{}", elem.to_lowercase())
        };
        let has_model_pos = match prop_of(&props, "position") {
            Some(p) => prop_exprs(p).iter().any(|e| expr_binds_model(def, e)),
            None => false,
        };
        let vis_model = match prop_of(&props, "visible") {
            Some(p) => prop_exprs(p).iter().any(|e| expr_binds_model(def, e)),
            None => false,
        };
        out.push(DynArr {
            elem: elem.clone(),
            handle: format!("{base}_handle"),
            px: if has_model_pos {
                Some(format!("{base}_px"))
            } else {
                None
            },
            py: if has_model_pos {
                Some(format!("{base}_py"))
            } else {
                None
            },
            // PLAN-023 F-8.1：可见性缓存名自适应——默认 `{base}_vis`
            // （frozen components-2d 冻结字节，F-1）；仅当与作者 model
            // 字段 SoA 名撞名（重复 `var`，admission 致命；T-02 实测
            // `zone_vis`）时改名 `{base}_visq`（game 档，与 T-02 手扩
            // 世界同形：coin_vis 不变、zone_vis→zone_visq）。与改名后
            // 仍撞名的字段在本函数尾部具名拒绝。
            vis: if vis_model {
                let plain = format!("{base}_vis");
                let collides = def
                    .model
                    .iter()
                    .any(|(f, _)| format!("{prefix}_{f}") == plain);
                Some(if collides {
                    format!("{base}_{vis_suffix}")
                } else {
                    plain
                })
            } else {
                None
            },
        });
    }
    // 缓存符号保留名检查：`{prefix}_{field}` 命中任一生成缓存数组名即
    // 为重复根符号（重复 `var`，admission 拒绝），下降期提前具名定位。
    let mut caches: Vec<String> = Vec::new();
    for arr in &out {
        caches.push(arr.handle.clone());
        if let Some(px) = &arr.px {
            caches.push(px.clone());
        }
        if let Some(py) = &arr.py {
            caches.push(py.clone());
        }
        if let Some(vis) = &arr.vis {
            caches.push(vis.clone());
        }
    }
    for (fname, _) in &def.model {
        let soa = format!("{soa_prefix}_{fname}");
        if caches.contains(&soa) {
            return Err(at(
                &def.module,
                def.line,
                fname,
                &format!(
                    "model field '{fname}' collides with the generated present-cache symbol '{soa}' (cache names <prefix>_handle/_px/_py/_visq are reserved); rename the model field"
                ),
            ));
        }
    }
    Ok(out)
}

/// 表达式是否引用本定义 model 字段（运行期值）。
fn expr_binds_model(def: &DefIR, e: &Expr) -> bool {
    match e {
        Expr::Dot(inner, name) => {
            matches!(&**inner, Expr::Ident(b) if b == "self")
                && def.model.iter().any(|(n, _)| n == name.as_str())
        }
        Expr::Bina(l, _, r) => expr_binds_model(def, l) || expr_binds_model(def, r),
        Expr::Tuple(items) => items.iter().any(|i| expr_binds_model(def, i)),
        _ => false,
    }
}

/// 词边界重命名：`fn` 名 → `{module}_{fn}`（池化纯函数的模块作用域）。
pub(crate) fn rename_module_calls(text: &str, names: &[String], module: &str) -> String {
    let mut out = text.to_string();
    for n in names {
        let mut res = String::new();
        let bytes = out.as_bytes();
        let mut i = 0usize;
        while i < out.len() {
            if out[i..].starts_with(n.as_str()) {
                let prev_ok =
                    i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
                let next = i + n.len();
                let next_ok = next < out.len() && bytes[next] == b'(';
                if prev_ok && next_ok {
                    res.push_str(module);
                    res.push('_');
                    res.push_str(n);
                    i = next;
                    continue;
                }
            }
            let ch = out[i..].chars().next().unwrap();
            res.push(ch);
            i += ch.len_utf8();
        }
        out = res;
    }
    out
}

fn scene_json_components(
    entry: &str,
    rows: &[SceneNode],
    has_char: bool,
    sha: &str,
    bytes: usize,
) -> String {
    let objects = rows.iter().map(obj_json).collect::<Vec<_>>().join(",\n");
    format!(
        "{{\n \"behavior\": {{\n  \"api\": \"autographics.objects2d/1\",\n  \"source\": {{\n   \"format\": \"auto-source\",\n   \"sha256\": \"{sha}\",\n   \"byte_length\": {bytes}\n  }},\n  \"actors\": {{\n   \"two_d\": {},\n   \"three_d\": null\n  }}\n }},\n \"schema\": \"autographics.scene/6\",\n \"document_id\": \"plan020-components-2d-{entry}\",\n \"resources\": [\n  \"circle\",\n  \"quad\",\n  \"cylinder\",\n  \"player\",\n  \"mob\"\n ],\n \"objects\": [\n{objects}\n ],\n \"groups\": [],\n \"assets\": []\n}}\n",
        if has_char { 1 } else { 0 }
    )
}

fn elem_props(project: &Project, def: &DefIR, elem: &str) -> Result<Vec<Prop>, LowerError> {
    let decl = node_of(project, &def.name);
    let view = decl.view2d.as_ref().expect("view");
    view.elements
        .iter()
        .find_map(|e| match e {
            View2dElement::Shape { name, props } if name == elem => Some(props.clone()),
            _ => None,
        })
        .ok_or_else(|| at(&def.module, def.line, elem, "shape props missing"))
}

/// 生成用户源（world.at）。
pub(crate) fn gen_user_source(
    project: &Project,
    world: &WorldIR,
    plan: &Plan,
    scene: &SceneBuild,
    queries: bool,
    game: Option<&crate::lower_components_game::GameGen>,
    sprites: bool,
    pres: Option<&crate::lower_presentation::PresGen>,
) -> Result<String, LowerError> {
    let mut src = String::new();
    let root_def = &world.defs[&world.root];
    let root_decl = node_of(project, &world.root);
    let qbinds = std::cell::RefCell::<Vec<(String, QueryKind)>>::default();

    // ---- 头部 ----
    if queries {
        src.push_str(
            "// GENERATED by autographics-node2d-pilot (PLAN-021 queries-2d, T-03/T-04 contract).\n",
        );
    } else if sprites {
        src.push_str(
            "// GENERATED by autographics-node2d-pilot (PLAN-022 sprites-2d, T-02 contract).\n",
        );
    } else {
        src.push_str(
            "// GENERATED by autographics-node2d-pilot (PLAN-020 components-2d, T-03/T-04 contract).\n",
        );
    }
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
    if sprites {
        src.push_str("// Regeneration must be byte-identical (tools/verify_plan022.py).\n");
    } else {
        src.push_str("// Regeneration must be byte-identical (tools/verify_plan020.py).\n");
    }
    src.push_str("var ok=0.0\n");

    // ---- 枚举常量（019 约定）----
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

    // ---- 根标量（r_<field>）----
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
    // PLAN-023 F-4（game 面）：生成基础设施标量（gate/ord/np/p0/p1）。
    // 作者只声明 count/state；生成器不发明游戏状态。
    if let Some(g) = game {
        src.push_str("// game closeout scalars (generated: gate/ord/np/p0/p1)\n");
        for name in g.infra_field_names() {
            src.push_str(&format!("var r_{name}=0\n"));
        }
    }
    // PLAN-025：presentation 基础设施标量（ord/np + 有界请求行）。
    // PLAN-032：components2d 档换用运行时标量（snap/巡逻/四行缓冲）。
    if let Some(pg) = pres {
        if pg.mode == crate::lower_presentation::PresMode::Components2dRuntime {
            src.push_str(&crate::lower_components2d_runtime::cmp_scalars_src(pg));
        } else {
            src.push_str(&crate::lower_presentation::pres_scalars_src());
        }
    }

    // PLAN-023 F-8.1（game 面）：呈现缓存后缀 visq（frozen 档保持 vis）。
    let vis_suffix: &str = if game.is_some() {
        crate::lower_components_game::GameGen::vis_suffix()
    } else {
        "vis"
    };

    // ---- 每定义 SoA ----
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
        for (pname, pt) in def_runtime_params(project, def) {
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
                vals.push(val.text(pt == PType::Int));
            }
            src.push_str(&format!(
                "var {}_{}=[{}]\n",
                def.prefix(),
                pname,
                vals.join(",")
            ));
        }
        for arr in def_dyn_arrays(project, def, &def.prefix(), &def.prefix(), vis_suffix)? {
            let zeros = vec!["0"; idxs.len()].join(",");
            src.push_str(&format!("var {}=[{zeros}]\n", arr.handle));
            if let (Some(px), Some(py)) = (&arr.px, &arr.py) {
                let props = elem_props(project, def, &arr.elem)?;
                let mut vx = Vec::new();
                let mut vy = Vec::new();
                for &k in &idxs {
                    let inst = &world.instances[k];
                    let pos = resolve_pos(def, &props, &inst.params)?;
                    vx.push(fmt_f(cval_value(project, world, def, &pos[0], Some(inst))?));
                    vy.push(fmt_f(cval_value(project, world, def, &pos[1], Some(inst))?));
                }
                src.push_str(&format!("var {px}=[{}]\n", vx.join(",")));
                src.push_str(&format!("var {py}=[{}]\n", vy.join(",")));
            }
            if let Some(vis) = &arr.vis {
                let props = elem_props(project, def, &arr.elem)?;
                let mut vv = Vec::new();
                for &k in &idxs {
                    let inst = &world.instances[k];
                    let v = resolve_visible(def, &props, &inst.params)?;
                    // 可见缓存是 int 面（0/1；与 ag_object_visible 同型）。
                    vv.push(format!(
                        "{}",
                        cval_value(project, world, def, &v, Some(inst))? as i64
                    ));
                }
                src.push_str(&format!("var {vis}=[{}]\n", vv.join(",")));
            }
        }
    }
    // root 动态呈现标量。
    for arr in def_dyn_arrays(project, root_def, "main", "r", vis_suffix)? {
        src.push_str(&format!("var {}=0\n", arr.handle));
        if let (Some(px), Some(py)) = (&arr.px, &arr.py) {
            let props = elem_props(project, root_def, &arr.elem)?;
            let pos = resolve_pos(root_def, &props, &[])?;
            let x = cval_value(project, world, root_def, &pos[0], None)?;
            let y = cval_value(project, world, root_def, &pos[1], None)?;
            src.push_str(&format!("var {px}={}\n", fmt_f(x)));
            src.push_str(&format!("var {py}={}\n", fmt_f(y)));
        }
        if let Some(vis) = &arr.vis {
            let props = elem_props(project, root_def, &arr.elem)?;
            let v = resolve_visible(root_def, &props, &[])?;
            let vv = cval_value(project, world, root_def, &v, None)?;
            // 可见缓存是 int 面（0/1；与 ag_object_visible 同型）。
            src.push_str(&format!("var {vis}={}\n", vv as i64));
        }
    }

    // ---- 事件绑定表（发射实例 i → 接收下标；root 发射方直呼无表）----
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

    // ---- 池化纯函数（模块作用域重命名；序稳定）----
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

    // ---- PLAN-021：self 行寻址数组（定义序；无静态行实例=-1）----
    if queries {
        for dname in def_order(world) {
            let Some(rows) = scene.self_rows.get(&dname) else {
                continue;
            };
            if rows.is_empty() {
                continue;
            }
            let prefix = world.defs[&dname].prefix();
            let mut sorted = rows.clone();
            sorted.sort_by_key(|(idx, _)| *idx);
            let list = sorted
                .iter()
                .map(|(_, r)| r.to_string())
                .collect::<Vec<_>>()
                .join(",");
            src.push_str(&format!("var {prefix}_qrow=[{list}]\n"));
        }
    }

    // ---- PLAN-022：图像帧表全局数组 + 图像模板 handle 数组 ----
    let mut frames_env: BTreeMap<String, crate::translate::FrameTable> = BTreeMap::new();
    if sprites {
        for img in &scene.used_images {
            let frames = world
                .frames
                .get(img)
                .ok_or_else(|| at(&world.root_module, 0, img, "used image has no frame table"))?;
            let fx: Vec<String> = frames.iter().map(|(_, r)| r[0].to_string()).collect();
            let fy: Vec<String> = frames.iter().map(|(_, r)| r[1].to_string()).collect();
            let fw: Vec<String> = frames.iter().map(|(_, r)| r[2].to_string()).collect();
            let fh: Vec<String> = frames.iter().map(|(_, r)| r[3].to_string()).collect();
            src.push_str(&format!(
                "var {img}_fx=[{}]
var {img}_fy=[{}]
var {img}_fw=[{}]
var {img}_fh=[{}]
",
                fx.join(","),
                fy.join(","),
                fw.join(","),
                fh.join(",")
            ));
            frames_env.insert(
                img.clone(),
                crate::translate::FrameTable {
                    image: img.clone(),
                    len: frames.len(),
                },
            );
        }
        // 图像模板 handle 数组（每实例一槽；app_start spawn 写入）。
        for dname in def_order(world) {
            let def = &world.defs[&dname];
            let dyn_sprites = def
                .sprite_elems
                .iter()
                .filter(|e| !e.is_char)
                .filter(|e| {
                    matches!(e.frame, FrameBind::Field(_)) || matches!(e.flip, FrameBind::Field(_))
                })
                .count();
            if dyn_sprites == 0 {
                continue;
            }
            let idxs = world.instances_of(&dname);
            if idxs.is_empty() {
                continue;
            }
            let zeros = vec!["0"; idxs.len()].join(",");
            src.push_str(&format!(
                "var {}_handle=[{zeros}]
",
                def.prefix()
            ));
        }
    }

    // ---- root Life/成员/呈现 fns ----
    let root_query = queries.then(|| QueryParts {
        refs: &scene.refs,
        self_row: None,
        self_writable: false,
        binds: &qbinds,
    });
    let sprite_env = sprites.then(|| crate::translate::SpriteEnv {
        spawns: &scene.spawn_sites,
        frames: &frames_env,
    });
    let pres_sfx = pres.map(|g| crate::translate::SfxEnv { cues: &g.cues });
    let rctx = root_ctx(
        project,
        world,
        plan,
        root_query.as_ref(),
        sprite_env.as_ref(),
        pres_sfx.as_ref(),
    );
    src.push_str("fn main_lifestart() {\n");
    if let Some(h) = life_handler(root_decl, "Start") {
        translate_handler(&h.body.stmts, &rctx, &mut src, 1)?;
    }
    src.push_str("}\n");
    src.push_str("fn main_lifetick(dt float) {\n");
    // PLAN-023 F-2c/F-6（game 面）：poll 前奏（每回调恰一次 next 首读）。
    if game.is_some() {
        src.push_str(" game_events()\n");
    }
    // PLAN-025（presentation 面）：触发存在时才有 poll 前奏。
    // PLAN-032（components2d 面）：快照先存 r_snap，再 poll（作者
    // input.snapshot() 读缓存；pressed 恰一次消费）。
    if let Some(pg) = pres {
        if pg.mode == crate::lower_presentation::PresMode::Components2dRuntime {
            src.push_str(" r_snap = ag_actions_snapshot()\n");
            src.push_str(" cmp_events()\n");
        } else if !pg.triggers.is_empty() {
            src.push_str(" pres_events()\n");
        }
    }
    if let Some(h) = life_handler(root_decl, "Tick") {
        translate_handler(&h.body.stmts, &rctx, &mut src, 1)?;
    }
    src.push_str("}\n");
    // root 成员接收 fn（直呼）。
    let mut root_recvs: Vec<(&String, &String)> = plan
        .recv_fns
        .iter()
        .filter(|(_, (r, _))| r.is_empty())
        .map(|(f, (_, e))| (f, e))
        .collect();
    root_recvs.sort();
    for (fname, ev) in root_recvs {
        gen_recv_fn(
            project,
            world,
            plan,
            fname,
            "",
            ev,
            root_query.as_ref(),
            sprite_env.as_ref(),
            pres_sfx.as_ref(),
            &mut src,
        )?;
    }
    // root 呈现 fn（含 PLAN-022 字符行帧写）。
    if root_def.life_tick
        && (!root_def.dyn_elems.is_empty()
            || root_def.sprite_elems.iter().any(|e| {
                e.is_char
                    && (matches!(e.frame, FrameBind::Field(_))
                        || matches!(e.flip, FrameBind::Field(_)))
            }))
    {
        gen_present_fn(
            project,
            world,
            plan,
            root_def,
            true,
            vis_suffix,
            sprite_env.as_ref(),
            pres_sfx.as_ref(),
            scene,
            &mut src,
        )?;
    }

    // ---- 每定义 fns（首个实例声明先序）----
    for dname in def_order(world) {
        let def = &world.defs[&dname];
        let prefix = def.prefix();
        let decl = node_of(project, &dname);
        if world.def_len(&dname) == 0 {
            continue;
        }
        let inst_query = queries.then(|| QueryParts {
            refs: &scene.refs,
            self_row: Some(format!("{prefix}_qrow[i]")),
            self_writable: def.char_elem.is_none(),
            binds: &qbinds,
        });
        let ictx = instance_ctx(
            project,
            plan,
            def,
            &prefix,
            "i",
            inst_query.as_ref(),
            sprite_env.as_ref(),
            pres_sfx.as_ref(),
        )?;
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
        // 本定义的成员接收 fn（名字序）。
        let mut mine: Vec<(&String, &String)> = plan
            .recv_fns
            .iter()
            .filter(|(_, (r, _))| r == &dname)
            .map(|(f, (_, e))| (f, e))
            .collect();
        mine.sort();
        let recv_query = queries.then(|| QueryParts {
            refs: &scene.refs,
            self_row: Some(format!("{prefix}_qrow[j]")),
            self_writable: def.char_elem.is_none(),
            binds: &qbinds,
        });
        for (fname, ev) in mine {
            gen_recv_fn(
                project,
                world,
                plan,
                fname,
                &dname,
                ev,
                recv_query.as_ref(),
                sprite_env.as_ref(),
                pres_sfx.as_ref(),
                &mut src,
            )?;
        }
        let has_sprite_write = def.sprite_elems.iter().any(|e| {
            matches!(e.frame, FrameBind::Field(_)) || matches!(e.flip, FrameBind::Field(_))
        });
        if def.life_tick && (!def.dyn_elems.is_empty() || has_sprite_write) {
            gen_present_fn(
                project,
                world,
                plan,
                def,
                false,
                vis_suffix,
                sprite_env.as_ref(),
                pres_sfx.as_ref(),
                scene,
                &mut src,
            )?;
        }
    }

    // ---- getters（root + 每实例；观测全部 model 字段）----
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

    // ---- app_start ----
    src.push_str("fn app_start() {\n");
    if let Some((cid, _)) = scene.char {
        src.push_str(&format!(" ok=ok+ag_char_bind({cid})*1.0\n"));
        for sid in &scene.solid_ids {
            src.push_str(&format!(" ok=ok+ag_char_solid({sid})*1.0\n"));
        }
        if let Some(cam) = &scene.camera {
            src.push_str(&format!(
                " ok=ok+ag_camera_follow_2d({cid}, {}, {}, {}, {}, {}, {})*1.0\n",
                fmt_f(cam.min_x),
                fmt_f(cam.min_y),
                fmt_f(cam.max_x),
                fmt_f(cam.max_y),
                fmt_f(cam.view_w),
                fmt_f(cam.view_h)
            ));
        }
    }
    for line in &scene.spawns {
        src.push_str(line);
        src.push('\n');
    }
    // PLAN-023 F-2a（game 面）：每 authoring 触发行一条 ag_trigger_bind
    // （pair = 行序，subject 恒为 character，source = 实例 handle）。
    if let Some(g) = game {
        let char_id = scene.char.clone().map(|(id, _)| id).ok_or_else(|| {
            at(
                &root_def.module,
                root_def.line,
                "trigger",
                "game2d triggers require the components character2d subject",
            )
        })?;
        src.push_str(&crate::lower_components_game::trigger_binds_src(g, char_id));
    }
    // PLAN-025（presentation 面）：每 authoring 触发行一条 ag_trigger_bind
    // （pair = 声明序，subject 恒为 character；触发存在时要求恰一角色）。
    if let Some(pg) = pres {
        if !pg.triggers.is_empty() {
            let char_id = scene.char.clone().map(|(id, _)| id).ok_or_else(|| {
                at(
                    &root_def.module,
                    root_def.line,
                    "trigger",
                    "presentation2d triggers require the components character2d subject",
                )
            })?;
            src.push_str(&crate::lower_presentation::pres_binds_src(pg, char_id));
        }
    }
    if root_def.life_start {
        src.push_str(" main_lifestart()\n");
    }
    for inst in &world.instances {
        let def = &world.defs[&inst.def];
        if def.life_start {
            src.push_str(&format!(" {}_lifestart({})\n", def.prefix(), inst.def_idx));
        }
    }
    src.push_str("}\n");

    // ---- app_update（冻结序：根 → 声明先序 tick → present）----
    src.push_str("fn app_update(dt float) {\n");
    src.push_str(" main_lifetick(dt)\n");
    let root_presents = root_def.life_tick
        && (!root_def.dyn_elems.is_empty()
            || root_def.sprite_elems.iter().any(|e| {
                e.is_char
                    && (matches!(e.frame, FrameBind::Field(_))
                        || matches!(e.flip, FrameBind::Field(_)))
            }));
    if root_presents {
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
        let presents = def.life_tick
            && (!def.dyn_elems.is_empty()
                || def.sprite_elems.iter().any(|e| {
                    matches!(e.frame, FrameBind::Field(_)) || matches!(e.flip, FrameBind::Field(_))
                }));
        if presents {
            src.push_str(&format!(" {}_present({})\n", def.prefix(), inst.def_idx));
        }
    }
    // PLAN-023 F-6（game 面）：每回调恰一次 closeout 收口。
    if game.is_some() {
        src.push_str(" game_closeout()\n");
    }
    // PLAN-025（presentation 面）：每回调恰一次 closeout 收口。
    // PLAN-032（components2d 面）：巡逻 + 终局 Enter + kind=0 帧收口。
    if let Some(pg) = pres {
        if pg.mode == crate::lower_presentation::PresMode::Components2dRuntime {
            if pg.cmp.as_ref().and_then(|c| c.patrol.as_ref()).is_some() {
                src.push_str(" cmp_patrol(dt)\n");
            }
            src.push_str(" cmp_restart()\n");
            src.push_str(" cmp_closeout()\n");
        } else {
            src.push_str(" pres_closeout()\n");
        }
    }
    src.push_str("}\n");
    // PLAN-023 F-3（game 面）：closeout 骨架（game_cue / game_events /
    // hud_<field>_text / game_closeout）。
    if let Some(g) = game {
        src.push_str(&crate::lower_components_game::game_fns_src(g));
    }
    if let Some(pg) = pres {
        if pg.mode == crate::lower_presentation::PresMode::Components2dRuntime {
            src.push_str(&crate::lower_components2d_runtime::cmp_fns_src(pg));
        } else {
            src.push_str(&crate::lower_presentation::pres_fns_src(pg));
        }
    }
    Ok(src)
}

/// 成员接收 fn：`fn <name>_<ev>(j int, p..)` / root `fn main_<ev>(p..)`。
pub(crate) fn gen_recv_fn(
    project: &Project,
    world: &WorldIR,
    plan: &Plan,
    fname: &str,
    recv_def_name: &str,
    ev: &str,
    query: Option<&QueryParts>,
    sprites: Option<&crate::translate::SpriteEnv>,
    sfx: Option<&crate::translate::SfxEnv>,
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
    let ctx = if index.is_empty() {
        root_ctx(project, world, plan, query, sprites, sfx)
    } else {
        instance_ctx(project, plan, def, &def.prefix(), "j", query, sprites, sfx)?
    };
    translate_handler(&h.body.stmts, &ctx, src, 1)?;
    src.push_str("}\n");
    Ok(())
}

/// 呈现同步 fn：`fn <prefix>_present(i int)`（root 无索引）；位置/可见按
/// 变更计费（缓存比较），预算与 T-02 探针一致（走跳 tick ≤32）。
#[allow(clippy::too_many_arguments)]
fn gen_present_fn(
    project: &Project,
    world: &WorldIR,
    plan: &Plan,
    def: &DefIR,
    is_root: bool,
    vis_suffix: &str,
    sprites: Option<&crate::translate::SpriteEnv>,
    sfx: Option<&crate::translate::SfxEnv>,
    scene: &SceneBuild,
    src: &mut String,
) -> Result<(), LowerError> {
    let prefix = if is_root {
        "main".to_string()
    } else {
        def.prefix()
    };
    let arrays = def_dyn_arrays(project, def, &prefix, &prefix, vis_suffix)?;
    if is_root {
        src.push_str("fn main_present() {\n");
    } else {
        src.push_str(&format!("fn {prefix}_present(i int) {{\n"));
    }
    let index = if is_root { "" } else { "i" };
    for arr in &arrays {
        let props = elem_props(project, def, &arr.elem)?;
        let ctx = if is_root {
            root_ctx(project, world, plan, None, sprites, sfx)
        } else {
            instance_ctx(project, plan, def, &prefix, index, None, sprites, sfx)?
        };
        let pos = prop_of(&props, "position")
            .ok_or_else(|| at(&def.module, def.line, "position", "missing position"))?;
        let es = prop_exprs(pos);
        let x_text = expr_text(&es[0], &ctx)?;
        let y_text = expr_text(&es[1], &ctx)?;
        let handle_ref = if is_root {
            arr.handle.clone()
        } else {
            format!("{}[{index}]", arr.handle)
        };
        if let (Some(px), Some(py)) = (&arr.px, &arr.py) {
            let px_ref = if is_root {
                px.clone()
            } else {
                format!("{px}[{index}]")
            };
            let py_ref = if is_root {
                py.clone()
            } else {
                format!("{py}[{index}]")
            };
            src.push_str(&format!(
                " if (((({px_ref}-{x_text})*({px_ref}-{x_text}))+(({py_ref}-{y_text})*({py_ref}-{y_text})))>0.0) {{\n  {px_ref}={x_text}\n  {py_ref}={y_text}\n  ag_object_move({handle_ref},{x_text},{y_text},0.0)\n }}\n"
            ));
        }
        if let Some(vis) = &arr.vis {
            let vexpr = prop_of(&props, "visible")
                .map(|p| prop_exprs(p)[0].clone())
                .ok_or_else(|| at(&def.module, def.line, "visible", "missing visible"))?;
            let v_text = expr_text(&vexpr, &ctx)?;
            let vis_ref = if is_root {
                vis.clone()
            } else {
                format!("{vis}[{index}]")
            };
            src.push_str(&format!(
                " if {vis_ref}!={v_text} {{\n  {vis_ref}={v_text}\n  ag_object_visible({handle_ref},{v_text})\n }}\n"
            ));
        }
    }
    // ---- PLAN-022：逐 tick 图像帧写（if-chain 帧表；越界具名 fault 42）----
    if let Some(se) = def
        .sprite_elems
        .iter()
        .find(|e| matches!(e.frame, FrameBind::Field(_)) || matches!(e.flip, FrameBind::Field(_)))
    {
        // 每定义恰一图像（多实例 Param 形态必须解析到同一 image2d 资源）。
        let mut resolved: Option<String> = None;
        if let Some(p) = &se.image_param {
            for inst in world.instances.iter().filter(|i| i.def == def.name) {
                let img = match inst.params.iter().find(|(n, _)| n == p).map(|(_, v)| v) {
                    Some(ParamVal::Res(r)) => r.clone(),
                    _ => {
                        return Err(at(
                            &def.module,
                            se.line,
                            p,
                            "image param binding is not an image2d resource",
                        ))
                    }
                };
                match &resolved {
                    None => resolved = Some(img),
                    Some(prev) if *prev == img => {}
                    Some(prev) => {
                        return Err(at(
                            &def.module,
                            se.line,
                            p,
                            &format!(
                                "instances of '{}' bind different images ('{prev}' vs '{img}'); a per-tick write needs one shared image per definition",
                                def.name
                            ),
                        ));
                    }
                }
            }
            if resolved.is_none() {
                return Err(at(
                    &def.module,
                    se.line,
                    p,
                    "image param has no instance to resolve",
                ));
            }
        }
        let image = resolved.unwrap_or_else(|| se.image.clone());
        let frames = world.frames.get(&image).ok_or_else(|| {
            at(
                &def.module,
                def.line,
                &image,
                "bound image has no frame table",
            )
        })?;
        let n = frames.len();
        let frame_ref = match &se.frame {
            FrameBind::Field(f) => {
                if is_root {
                    format!("r_{f}")
                } else {
                    format!("{prefix}_{f}[{index}]")
                }
            }
            _ => unreachable!("dynamic write requires a Field frame binding"),
        };
        let flip_text = match &se.flip {
            FrameBind::Lit(v) => v.to_string(),
            FrameBind::Param(p) => p.clone(),
            FrameBind::Field(f) => {
                if is_root {
                    format!("r_{f}")
                } else {
                    format!("{prefix}_{f}[{index}]")
                }
            }
        };
        let mut chain = String::new();
        for ordinal in 0..n {
            let call_body = if se.is_char {
                let Some((cid, _)) = scene.char else {
                    return Err(at(
                        &def.module,
                        def.line,
                        &se.elem,
                        "character sprite write without a character row",
                    ));
                };
                format!(
                    "ag_sprite_frame({cid}, {image}_fx[{ordinal}], {image}_fy[{ordinal}], {image}_fw[{ordinal}], {image}_fh[{ordinal}], {flip_text})"
                )
            } else if is_root {
                return Err(at(
                    &def.module,
                    def.line,
                    &se.elem,
                    "root view image shapes are static rows (no slot to write)",
                ));
            } else {
                let handle_ref = format!("{prefix}_handle[{index}]");
                format!(
                    "ag_sprite_slot({handle_ref}, {image}_fx[{ordinal}], {image}_fy[{ordinal}], {image}_fw[{ordinal}], {image}_fh[{ordinal}], {flip_text})"
                )
            };
            if ordinal == 0 {
                chain.push_str(&format!(" if {frame_ref}=={ordinal} {{ {call_body} }}"));
            } else {
                chain.push_str(&format!(
                    " else if {frame_ref}=={ordinal} {{ {call_body} }}"
                ));
            }
        }
        // 越界序数：写入退化矩形，由受信单元自身的矩形域检查具名拒绝
        // （fault 42；用户源不可直呼保留传输 _ag_fail——admission 实测）。
        let fallback = if se.is_char {
            let (cid, _) = scene.char.clone().ok_or_else(|| {
                at(
                    &def.module,
                    def.line,
                    &se.elem,
                    "character sprite write without a character row",
                )
            })?;
            format!("ag_sprite_frame({cid}, 0, 0, 0, 0, 0)")
        } else {
            let handle_ref = format!("{prefix}_handle[{index}]");
            format!("ag_sprite_slot({handle_ref}, 0, 0, 0, 0, 0)")
        };
        chain.push_str(&format!(" else {{ {fallback} }}\n"));
        src.push_str(&chain);
    }
    src.push_str("}\n");
    Ok(())
}

// ----------------------------------------------------------- entry ----

/// PLAN-020 components-2d 公共入口：校验 → 场景/用户源 → identity 材料。
pub fn lower_project_components(project: &Project) -> Result<LowerOutputBasic2D, LowerError> {
    lower_components_mode(project, false, None, false)
}

/// PLAN-021：queries-2d 显式作者档（components-2d 的超集 + typed 查询
/// 面；api/运行时装配不变，identity generator 字符串区分）。
pub fn lower_project_queries(project: &Project) -> Result<LowerOutputBasic2D, LowerError> {
    lower_components_mode(project, true, None, false)
}

/// PLAN-022：sprites-2d 作者档——typed 图像/帧表/组件局部帧翻转绑定 +
/// render2d 渲染实例服务，装配 autographics.sprites2d/1。
pub fn lower_project_sprites(project: &Project) -> Result<LowerOutputBasic2D, LowerError> {
    lower_components_mode(project, false, None, true)
}

/// PLAN-022：场景 JSON（sprites2d/1 装配 + png 资产清单）。
fn scene_json_sprites(
    entry: &str,
    rows: &[SceneNode],
    has_char: bool,
    sha: &str,
    bytes: usize,
    assets: &[String],
) -> String {
    let objects = rows.iter().map(obj_json).collect::<Vec<_>>().join(",\n");
    let assets = assets.join(",\n  ");
    format!(
        "{{\n \"behavior\": {{\n  \"api\": \"autographics.sprites2d/1\",\n  \"source\": {{\n   \"format\": \"auto-source\",\n   \"sha256\": \"{sha}\",\n   \"byte_length\": {bytes}\n  }},\n  \"actors\": {{\n   \"two_d\": {},\n   \"three_d\": null\n  }}\n }},\n \"schema\": \"autographics.scene/6\",\n \"document_id\": \"plan022-sprites-2d-{entry}\",\n \"resources\": [\n  \"circle\",\n  \"quad\",\n  \"cylinder\",\n  \"player\",\n  \"mob\"\n ],\n \"objects\": [\n{objects}\n ],\n \"groups\": [],\n \"assets\": [\n  {assets}\n ]\n}}\n",
        if has_char { 1 } else { 0 }
    )
}

fn lower_components_mode(
    project: &Project,
    queries: bool,
    game: Option<&crate::lower_components_game::GameGen>,
    sprites: bool,
) -> Result<LowerOutputBasic2D, LowerError> {
    lower_components_mode_ex(project, queries, game, sprites, false)
}

/// PLAN-025：pres 变体（presentation-2d 走 Some 注入的独立入口，不经此
/// 分派；保留冻结档的签名形状）。
fn lower_components_mode_ex(
    project: &Project,
    queries: bool,
    game: Option<&crate::lower_components_game::GameGen>,
    sprites: bool,
    pres: bool,
) -> Result<LowerOutputBasic2D, LowerError> {
    // PLAN-023：game2d 作者块只被 components-2d-game 消费；frozen 档拒绝
    // （不静默忽略）。
    if game.is_none() {
        for (mname, module) in &project.modules {
            for n in &module.nodes {
                if n.game2d.is_some() {
                    let line = line_at(&module.source, n.span.0);
                    return Err(at(
                        mname,
                        line,
                        &n.name,
                        "game2d block requires --profile components-2d-game (the frozen components-2d profile keeps its 020 lowering)",
                    ));
                }
            }
        }
    }
    let world = build_world(project, sprites, true)?;
    let plan = plan_dispatch(project, &world, queries, sprites, pres)?;
    let scene = if sprites {
        collect_scene_sprites(project, &world, &plan)?
    } else {
        collect_scene(project, &world, &plan)?
    };
    let src = gen_user_source(project, &world, &plan, &scene, queries, game, sprites, None)?;
    let source_bytes = src.len();
    let source_sha256 = sha256_hex(src.as_bytes());
    let entry = world.root_module.clone();
    let scene_json = if sprites {
        scene_json_sprites(
            &entry,
            &scene.rows,
            scene.char.is_some(),
            &source_sha256,
            source_bytes,
            &scene.assets_json,
        )
    } else {
        scene_json_components(
            &entry,
            &scene.rows,
            scene.char.is_some(),
            &source_sha256,
            source_bytes,
        )
    };
    let collected = Collected {
        nodes: scene.rows,
        camera: scene.camera,
        char_name: scene.char.map(|(_, n)| n),
    };
    let roles_json = roles_json_basic(&collected);
    Ok(LowerOutputBasic2D {
        user_source: Some(src),
        scene_json,
        source_sha256,
        source_bytes,
        roles_json,
    })
}
