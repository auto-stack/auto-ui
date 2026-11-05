//! PLAN-048 node-ui 降层：基础 UI Node（`view` 块）→ 版本化 UI 产物
//! （`ui.json`，schema `autographics.node-ui/1`）＋ 单文件真实 Auto 用户源
//! （`world.at`）。业务全部落在生成的 Auto 源里（SoA 实例表/事件
//! handler/文字字节缓冲/格式化/绑定 getter）；Rust 侧只做编译、控件宿主
//! 与有界队列适配。文字经 UTF-8 字节缓冲（int 全局）＋有界 len/at getter
//! 过桥——不假定 String ABI（T-02 冻结合同）。
//!
//! 与 components-2d 同构的冻结约定：声明先序实例表（root = root def 的
//! 实例 0）、`{prefix}_{field}[i]` SoA、root 标量 `r_{field}`、root
//! lifestart/lifetick 无下标、def 层带 `i`、recv fn `{prefix}_{ev}(i)`、
//! getter `get_{prefix}_{field}_{idx}` ＋ `get_{pathu}_{field}` ＋
//! `get_r_{field}`。UI 专属冻结面：
//! - 事件派发 ABI：`{prefix}_ui_click_{elem}()` / `(i int)`（root 无 i）
//!   —— adapter 只认派发器名，不直呼业务 fn；绑定表
//!   `{prefix}_bind_{child}_{EV}` 在派发器内部解析（错目标/陈旧消息在
//!   队列侧具名拒绝，不静默派到同名组件）。
//! - 文字 ABI：`{prefix}_ui_text[_len]` 平铺缓冲（每实例×每文字元素 64
//!   字节），`{prefix}_ui_text_len(i,e)` / `{prefix}_ui_text_at(i,e,j)`
//!   有界读取；`{prefix}_ui_refresh(i)`（root 恒 0）在 app_start/
//!   app_update/事件派发尾部统一重建——重绘/layout 不代 Tick，refresh
//!   只在已接受状态后运行（T-02 冻结：无输入不靠空 Tick 刷新状态）。
//! - 容量（T-02 冻结）：有状态实例 ≤8（root 计入）、产物元素 ≤32、
//!   容器子元素 ≤8、每文字元素 64 字节；文字 >63 字节或越界数值具名
//!   拒绝。旧预算（32 credits/回调、旧 VM/物理/场景/资源界）不动。

use crate::load::Project;
use crate::scene::sha256_hex;
use crate::translate::{translate_handler, Ctx, LowerError, Target};
use auto_parser_api::ast::{Expr, Stmt, Type};
use auto_ui_dialect::node2d::{PropValue, UiContainerKind, UiElement};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// 产物 schema 身份（driver 侧消费准入冻结）。
pub const NODE_UI_SCHEMA: &str = "autographics.node-ui/1";
/// 生成器身份（identity/产物头冻结）。
pub const NODE_UI_GENERATOR: &str = "autographics-node2d-pilot plan048 node-ui";
/// 每实例每文字元素的字节容量（T-02 冻结）。
pub const TEXT_BYTES: usize = 64;
/// 产物元素总数上限（T-02 冻结，per def）。
pub const MAX_ELEMENTS: usize = 32;
/// 单容器子元素上限（T-02 冻结）。
pub const MAX_CHILDREN: usize = 8;
/// 有状态实例上限（沿 020 envelope，root 计入）。
pub const MAX_INSTANCES: usize = 8;
/// 每 def 文字元素上限（T-02 冻结；字节缓冲按半缓冲拆分——SDK 字面量
/// 数组恰 256 元素静默截断为 0，048 复审实测 + PLAN-049 T-02 复证）。
pub const MAX_TEXTS_PER_DEF: usize = 4;
/// 每半缓冲的文字槽数（≤2 槽×64B = 128 < 255，安全界）。
pub const TEXT_SLOTS_PER_BUFFER: usize = 2;
/// PLAN-049 产物 schema 身份（混合 UI＋双维嵌入；driver 侧消费准入冻结）。
pub const NODE_UI_MIXED_SCHEMA: &str = "autographics.node-ui-mixed/1";
/// PLAN-049 生成器身份（identity/产物头冻结）。
pub const NODE_UI_MIXED_GENERATOR: &str = "autographics-node2d-pilot plan049 node-ui-mixed";
/// PLAN-054 产物 schema 身份（组件/实例活区域＋条件挂卸；与 v1 严格隔离——
/// v1 消费入口对新 schema 具名拒绝，v2 入口拒 v1）。
pub const NODE_UI_MIXED_REGIONS_SCHEMA: &str = "autographics.node-ui-mixed-regions/2";
/// PLAN-054 生成器身份（identity/产物头冻结）。
pub const NODE_UI_MIXED_REGIONS_GENERATOR: &str =
    "autographics-node2d-pilot plan054 node-ui-mixed-regions";
/// 每 def 嵌入区域上限（T-02 冻结首切片）。
pub const MAX_REGIONS_PER_DEF: usize = 4;
/// 每区域场景元素上限（沿容器子元素 8 同界；T-02 冻结）。
pub const MAX_SCENE_ELEMS_PER_REGION: usize = 8;
/// 区域占位 rect 单边上限（UI 逻辑 px；T-02 冻结）。
pub const MAX_REGION_SIDE: i64 = 640;

/// 降层产物（world.at ＋ ui.json ＋ 身份数）。
#[derive(Debug)]
pub struct LowerOutputNodeUi {
    pub user_source: String,
    pub ui_json: String,
    pub source_sha256: String,
    pub source_bytes: usize,
}

// ------------------------------------------------------ artifact model --

#[derive(Serialize)]
struct UiArtifact {
    schema: &'static str,
    generator: &'static str,
    world_sha256: String,
    root: String,
    instances: Vec<ArtifactInstance>,
    defs: BTreeMap<String, ArtifactDef>,
}

#[derive(Serialize)]
struct ArtifactInstance {
    path: String,
    node: String,
    def_idx: usize,
}

#[derive(Serialize)]
struct ArtifactDef {
    prefix: String,
    is_root: bool,
    /// 本 def 的文字元素数（缓冲寻址用）。
    texts: usize,
    elements: Vec<ArtifactElement>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind")]
enum ArtifactElement {
    Container {
        name: String,
        container: &'static str,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        props: Vec<ArtifactProp>,
        children: Vec<ArtifactElement>,
    },
    Text {
        name: String,
        /// 缓冲槽位（本 def 文字声明序；adapter 按槽取文字）。
        slot: usize,
        text: ArtifactTextValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        visible: Option<ArtifactCond>,
    },
    Button {
        name: String,
        /// 缓冲槽位（同上；按钮标签同占缓冲）。
        slot: usize,
        text: ArtifactTextValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<ArtifactCond>,
        #[serde(skip_serializing_if = "Option::is_none")]
        visible: Option<ArtifactCond>,
        click: ArtifactClick,
    },
    If {
        cond: ArtifactCond,
        then: Vec<ArtifactElement>,
    },
    /// 同定义实例引用（实例 identity 在顶层 instances 表按路径解析）。
    Instance { name: String, node: String },
    /// PLAN-049 嵌入区域（mixed 档专属；UI 占位 rect + 可选点击端口 +
    /// 有界场景元素。driver 侧同一形状——绑定经 Field getter，点击走
    /// 同一 `_ui_click_` 派发 ABI）。
    Embed {
        name: String,
        dim: &'static str,
        width: i64,
        height: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        click: Option<ArtifactClick>,
        scene: Vec<ArtifactSceneElem>,
    },
}

/// 区域内场景元素（T-02 冻结：quad/circle/cylinder；数值 LitNum/Field）。
#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactSceneElem {
    Quad {
        name: String,
        x: ArtifactPropValue,
        y: ArtifactPropValue,
        w: ArtifactPropValue,
        h: ArtifactPropValue,
        /// RGB 0..255 字面量。
        color: [u8; 3],
    },
    Circle {
        name: String,
        x: ArtifactPropValue,
        y: ArtifactPropValue,
        r: ArtifactPropValue,
        color: [u8; 3],
    },
    Cylinder {
        name: String,
        /// (tx, ty, tz)。
        translation: Vec<ArtifactPropValue>,
        color: [u8; 3],
    },
}

#[derive(Serialize, Clone)]
struct ArtifactProp {
    name: String,
    #[serde(flatten)]
    value: ArtifactPropValue,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ArtifactPropValue {
    /// 数字字面量（width/height/spacing/padding）。
    LitNum { v: f64 },
    /// int/bool 模型字段绑定（adapter 逐帧 getter 求值）。PLAN-049 T-04：
    /// 序列键 `field`——ArtifactProp 自身有 `name`，flatten 同层会与之
    /// 冲突（重复键；048 夹具从未用 Field 布局 prop，spacer 首次暴露）。
    Field {
        #[serde(rename = "field")]
        name: String,
    },
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ArtifactTextValue {
    /// 常量 UTF-8 文字（生成期内联为字节字面量）。
    LitStr { v: String },
    /// int 模型字段 → 十进制数字。
    IntField { name: String },
    /// bool 模型字段 → "true"/"false"。
    BoolField { name: String },
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ArtifactCond {
    LitBool {
        v: bool,
    },
    /// int 字段（非 0 = 真）。
    Field {
        name: String,
    },
    /// bool 字段。
    BoolField {
        name: String,
    },
}

#[derive(Serialize, Clone)]
struct ArtifactClick {
    /// 生成派发器名（`{prefix}_ui_click_{elem}`；adapter 唯一派发入口）。
    #[serde(rename = "fn")]
    fn_name: String,
    /// "index"（def 层派发器带 i）| "none"（root）。
    arg: &'static str,
}

// ----------------------------------------------- PLAN-049 mixed artifact --

/// PLAN-049 混合产物：与 [`UiArtifact`] 同一实例/def/文字/派发 ABI，
/// def 内新增 `regions`（嵌入区域；挂载关系，非跨维 parent）。世界源
/// 与 048 完全同一生成器面（区域点击走同一 `_ui_click_` 派发器 ABI，
/// 场景绑定复用既有 int getter——不新增世界 ABI 行）。
/// PLAN-049 降层产物（world.at ＋ ui.json ＋ 身份数；字段与
/// [`LowerOutputNodeUi`] 同形，world_sha256 即世界源 sha）。
#[derive(Debug)]
pub struct LowerOutputNodeMixed {
    pub user_source: String,
    pub ui_json: String,
    pub world_sha256: String,
    pub source_bytes: usize,
}

// ----------------------------------------------------------- def plan --

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Int,
    Bool,
    Str,
}

#[derive(Clone)]
enum Init {
    Int(i64),
    Bool(bool),
    Str(String),
    /// `.param` 引用（数值参数；按实例字面量内联）。
    Param(String),
}

#[derive(Clone)]
struct FieldPlan {
    name: String,
    kind: Kind,
    init: Init,
}

struct DefPlan {
    name: String,
    prefix: String,
    is_root: bool,
    /// PLAN-054 v2 档：非 root def 允许声明嵌入区域（实例挂载身份由
    /// 消费侧按实例路径装配）。v1（048/049）恒 false——诊断逐字不变。
    regions_v2: bool,
    module: String,
    node: auto_ui_dialect::node2d::NodeDecl,
    fields: Vec<FieldPlan>,
    /// 本 def view 的文字元素（声明序；缓冲槽位 = 声明序）。
    texts: Vec<(String, ArtifactTextValue)>,
    elements: Vec<ArtifactElement>,
    /// (elem, event, child-instance-name?)——click 派发器生成面。
    clicks: Vec<(String, String, Option<String>)>,
    /// 本 def 携带的零参业务事件名。
    events: Vec<String>,
    /// 本 def 视图内的实例名 → def 名（子事件解析）。
    local_instances: BTreeMap<String, String>,
}

struct InstancePlan {
    path: String,
    def: String,
    def_idx: usize,
    /// 参数 → 数值（实例字面量；缺失参数按 0）。
    params: BTreeMap<String, f64>,
}

fn err_at(module: &str, span: (usize, usize), message: String) -> String {
    let line = crate::load::line_of_pub(module, span.0);
    format!("{module}:{line}: {message}")
}

// --------------------------------------------------- field classification

fn classify_field(
    module: &str,
    def_name: &str,
    f: &auto_ui_dialect::ModelField,
    params: &[(String, Type)],
) -> Result<FieldPlan, String> {
    let name = f.name.to_string();
    let declared = match f.ty {
        Type::Int => Some(Kind::Int),
        Type::Bool => Some(Kind::Bool),
        Type::StrSlice | Type::StrOwned | Type::StrFixed(_) => Some(Kind::Str),
        _ => None,
    };
    let init = match (&f.init, declared) {
        (Expr::Int(v), None | Some(Kind::Int)) => Init::Int(*v as i64),
        (Expr::Bool(v), None | Some(Kind::Bool)) => Init::Bool(*v),
        (Expr::Str(v), None | Some(Kind::Str)) => Init::Str(v.to_string()),
        (Expr::Float(..), _) | (Expr::Double(..), _) => {
            return Err(err_at(
                module,
                (0, 0),
                format!(
                    "node '{def_name}': model field '{name}' is float; basic-UI v1 model is int/bool/constant-text (T-02 frozen)"
                ),
            ))
        }
        (Expr::Dot(inner, p), _) => {
            let is_self = matches!(&**inner, Expr::Ident(b) if b == "self");
            if !is_self {
                return Err(err_at(
                    module,
                    (0, 0),
                    format!(
                        "node '{def_name}': model field '{name}' init must be a literal or a leading-dot param ref"
                    ),
                ));
            }
            let ty = params
                .iter()
                .find(|(n, _)| n.as_str() == p.as_str())
                .map(|(_, t)| t)
                .ok_or_else(|| {
                    err_at(
                        module,
                        (0, 0),
                        format!(
                            "node '{def_name}': model field '{name}' references unknown param '{p}'"
                        ),
                    )
                })?;
            match ty {
                Type::Int => Init::Param(p.to_string()),
                _ => {
                    return Err(err_at(
                        module,
                        (0, 0),
                        format!(
                            "node '{def_name}': model field '{name}' references non-int param '{p}'; basic-UI v1 params are int"
                        ),
                    ))
                }
            }
        }
        _ => {
            return Err(err_at(
                module,
                (0, 0),
                format!(
                    "node '{def_name}': model field '{name}' init must be an int/bool/string literal or a leading-dot param ref"
                ),
            ))
        }
    };
    let kind = match declared {
        Some(k) => k,
        None => match init {
            Init::Int(_) | Init::Param(_) => Kind::Int,
            Init::Bool(_) => Kind::Bool,
            Init::Str(_) => Kind::Str,
        },
    };
    Ok(FieldPlan { name, kind, init })
}

// ---------------------------------------------- prop value classification

fn one_value<'a>(
    module: &str,
    def_name: &str,
    at: &str,
    prop: &str,
    v: &'a PropValue,
) -> Result<&'a Expr, String> {
    match v {
        PropValue::One(e) => Ok(e),
        PropValue::Pair(_) => Err(err_at(
            module,
            (0, 0),
            format!("node '{def_name}': prop '{prop}' on '{at}' takes a single value"),
        )),
    }
}

fn self_field<'a>(e: &'a Expr) -> Option<&'a str> {
    if let Expr::Dot(inner, f) = e {
        if matches!(&**inner, Expr::Ident(b) if b == "self") {
            return Some(f.as_str());
        }
    }
    None
}

fn classify_num_prop(
    module: &str,
    def_name: &str,
    elem: &str,
    prop: &str,
    v: &PropValue,
    fields: &[FieldPlan],
) -> Result<ArtifactPropValue, String> {
    let e = one_value(module, def_name, elem, prop, v)?;
    match e {
        Expr::Int(n) => Ok(ArtifactPropValue::LitNum { v: *n as f64 }),
        Expr::Float(n, _) | Expr::Double(n, _) => Ok(ArtifactPropValue::LitNum { v: *n }),
        _ => match self_field(e) {
            Some(f) => {
                let hit = fields.iter().find(|fd| fd.name == f).map(|fd| fd.kind);
                match hit {
                    Some(Kind::Int) | Some(Kind::Bool) => {
                        Ok(ArtifactPropValue::Field { name: f.to_string() })
                    }
                    _ => Err(err_at(
                        module,
                        (0, 0),
                        format!(
                            "node '{def_name}': prop '{prop}' on '{elem}' binds unknown numeric field '{f}'"
                        ),
                    )),
                }
            }
            None => Err(err_at(
                module,
                (0, 0),
                format!(
                    "node '{def_name}': prop '{prop}' on '{elem}' must be a number literal or a leading-dot field"
                ),
            )),
        },
    }
}

fn classify_text_value(
    module: &str,
    def_name: &str,
    elem: &str,
    v: &PropValue,
    fields: &[FieldPlan],
) -> Result<ArtifactTextValue, String> {
    let e = one_value(module, def_name, elem, "text", v)?;
    match e {
        Expr::Str(s) => {
            if s.as_bytes().len() >= TEXT_BYTES {
                return Err(err_at(
                    module,
                    (0, 0),
                    format!(
                        "node '{def_name}': text on '{elem}' exceeds the frozen {}-byte budget ({} bytes)",
                        TEXT_BYTES - 1,
                        s.as_bytes().len()
                    ),
                ));
            }
            Ok(ArtifactTextValue::LitStr { v: s.to_string() })
        }
        _ => match self_field(e) {
            Some(f) => match fields.iter().find(|fd| fd.name == f).map(|fd| (fd.kind, &fd.init)) {
                Some((Kind::Int, _)) => Ok(ArtifactTextValue::IntField {
                    name: f.to_string(),
                }),
                Some((Kind::Bool, _)) => Ok(ArtifactTextValue::BoolField {
                    name: f.to_string(),
                }),
                // 常量 UTF-8 文字字段：按字面量内联（不过 VM，不占缓冲绑定）。
                Some((Kind::Str, Init::Str(v))) => {
                    if v.as_bytes().len() >= TEXT_BYTES {
                        return Err(err_at(
                            module,
                            (0, 0),
                            format!(
                                "node '{def_name}': text field '{f}' on '{elem}' exceeds the frozen {}-byte budget",
                                TEXT_BYTES - 1
                            ),
                        ));
                    }
                    Ok(ArtifactTextValue::LitStr { v: v.clone() })
                }
                _ => Err(err_at(
                    module,
                    (0, 0),
                    format!(
                        "node '{def_name}': text binding on '{elem}' references unknown int/bool/constant-text field '{f}'"
                    ),
                )),
            },
            None => Err(err_at(
                module,
                (0, 0),
                format!(
                    "node '{def_name}': text on '{elem}' must be a string literal or a leading-dot int/bool/constant-text field (basic-UI v1)"
                ),
            )),
        },
    }
}

fn classify_cond_expr(
    module: &str,
    def_name: &str,
    at: &str,
    e: &Expr,
    fields: &[FieldPlan],
) -> Result<ArtifactCond, String> {
    match e {
        Expr::Bool(b) => Ok(ArtifactCond::LitBool { v: *b }),
        Expr::Int(n) => Ok(ArtifactCond::LitBool { v: *n != 0 }),
        _ => match self_field(e) {
            Some(f) => match fields.iter().find(|fd| fd.name == f).map(|fd| fd.kind) {
                Some(Kind::Int) => Ok(ArtifactCond::Field { name: f.to_string() }),
                Some(Kind::Bool) => Ok(ArtifactCond::BoolField { name: f.to_string() }),
                _ => Err(err_at(
                    module,
                    (0, 0),
                    format!(
                        "node '{def_name}': condition on '{at}' references unknown int/bool field '{f}'"
                    ),
                )),
            },
            None => Err(err_at(
                module,
                (0, 0),
                format!(
                    "node '{def_name}': condition on '{at}' must be a bool/int field or literal (basic-UI v1)"
                ),
            )),
        },
    }
}

/// onclick 值：`.Ev`（本实例）| `inst.Ev`（子实例）。零参 v1。
fn classify_click(
    module: &str,
    def_name: &str,
    elem: &str,
    v: &PropValue,
) -> Result<(String, Option<String>), String> {
    let e = one_value(module, def_name, elem, "onclick", v)?;
    if let Expr::Dot(inner, ev) = e {
        if let Expr::Ident(b) = &**inner {
            if b == "self" {
                return Ok((ev.to_string(), None));
            }
            return Ok((ev.to_string(), Some(b.to_string())));
        }
    }
    Err(err_at(
        module,
        (0, 0),
        format!(
            "node '{def_name}': onclick on '{elem}' must be '.Event' (self) or 'instance.Event' (child); basic-UI v1 emits zero-arg events"
        ),
    ))
}

// ------------------------------------------------------ view → artifact

/// PLAN-049：区域占位边长（int 字面量；1..=MAX_REGION_SIDE）。
fn classify_region_side(
    module: &str,
    def_name: &str,
    region: &str,
    prop: &str,
    v: &PropValue,
) -> Result<i64, String> {
    let e = one_value(module, def_name, region, prop, v)?;
    match e {
        Expr::Int(n) if *n > 0 && (*n as i64) <= MAX_REGION_SIDE => Ok(*n as i64),
        Expr::Int(n) => Err(err_at(
            module,
            (0, 0),
            format!(
                "node '{def_name}': {prop} on region '{region}' must be 1..={MAX_REGION_SIDE} (got {n}; frozen T-02 region budget)"
            ),
        )),
        _ => Err(err_at(
            module,
            (0, 0),
            format!(
                "node '{def_name}': {prop} on region '{region}' must be an int literal (frozen T-02 region sizing)"
            ),
        )),
    }
}

/// 场景数值（LitNum | 前导点 int 字段绑定；复用既有 int getter 面）。
fn classify_scene_num(
    module: &str,
    def_name: &str,
    at: &str,
    prop: &str,
    e: &Expr,
    fields: &[FieldPlan],
) -> Result<ArtifactPropValue, String> {
    match e {
        Expr::Int(n) => Ok(ArtifactPropValue::LitNum { v: *n as f64 }),
        Expr::Float(v, _) | Expr::Double(v, _) => {
            if v.fract() == 0.0 {
                Ok(ArtifactPropValue::LitNum { v: *v })
            } else {
                Err(err_at(
                    module,
                    (0, 0),
                    format!(
                        "node '{def_name}': {prop} on '{at}' must be an int or int field (frozen T-02 scene bindings reuse the int getter ABI)"
                    ),
                ))
            }
        }
        _ => match self_field(e) {
            Some(f) => match fields.iter().find(|fd| fd.name == f).map(|fd| fd.kind) {
                Some(Kind::Int) => Ok(ArtifactPropValue::Field { name: f.to_string() }),
                _ => Err(err_at(
                    module,
                    (0, 0),
                    format!(
                        "node '{def_name}': {prop} on '{at}' references unknown int field '{f}'"
                    ),
                )),
            },
            None => Err(err_at(
                module,
                (0, 0),
                format!(
                    "node '{def_name}': {prop} on '{at}' must be an int literal or leading-dot int field"
                ),
            )),
        },
    }
}

/// 区域内 AST 元素（维度在区域头固定；跨维混排由解析面分离）。
enum RegionElements<'a> {
    D2(&'a [auto_ui_dialect::node2d::View2dElement]),
    D3(&'a [auto_ui_dialect::node2d::View3dElement]),
}

/// 场景色（`(r, g, b)` 0..255 int 字面量三元组；T-02 首切片不绑定）。
fn classify_scene_color(
    module: &str,
    def_name: &str,
    at: &str,
    v: &PropValue,
) -> Result<[u8; 3], String> {
    let es = v.exprs();
    if es.len() != 3 {
        return Err(err_at(
            module,
            (0, 0),
            format!("node '{def_name}': color on '{at}' must be an (r, g, b) literal triple"),
        ));
    }
    let mut out = [0u8; 3];
    for (i, e) in es.iter().enumerate() {
        match e {
            Expr::Int(n) if (0..=255).contains(n) => out[i] = *n as u8,
            _ => {
                return Err(err_at(
                    module,
                    (0, 0),
                    format!("node '{def_name}': color on '{at}' must be 0..=255 int literals"),
                ))
            }
        }
    }
    Ok(out)
}

/// PLAN-049 嵌入区域降层（mix 档专用）：props（width/height/onclick）＋
/// 有界场景元素集；区域点击并入 plan.clicks（与按钮同一派发器 ABI 生成
/// 面）。区域内 If/Group/Instance/其余元素在首切片具名拒绝（T-03+ 扩面）。
#[allow(clippy::too_many_arguments)]
fn lower_embed_region(
    module: &str,
    def_name: &str,
    name: &str,
    dim: &'static str,
    props: &[auto_ui_dialect::node2d::Prop],
    elements: RegionElements<'_>,
    fields: &[FieldPlan],
    plan: &mut DefPlan,
    region_count: &mut usize,
) -> Result<ArtifactElement, String> {
    if *region_count >= MAX_REGIONS_PER_DEF {
        return Err(err_at(
            module,
            (0, 0),
            format!(
                "node '{def_name}': region count exceeds the frozen {MAX_REGIONS_PER_DEF}-region budget"
            ),
        ));
    }
    if !plan.is_root && !plan.regions_v2 {
        return Err(err_at(
            module,
            (0, 0),
            format!(
                "node '{def_name}': region '{name}' in a non-root def requires instance-mount identity (frozen T-02 slice admits root regions only)"
            ),
        ));
    }
    let mut width = None;
    let mut height = None;
    let mut click = None;
    let mut click_span = (0, 0);
    for p in props {
        match p.name.as_str() {
            "width" => width = Some(classify_region_side(module, def_name, name, "width", &p.value)?),
            "height" => {
                height = Some(classify_region_side(module, def_name, name, "height", &p.value)?)
            }
            "onclick" => {
                click_span = p.span;
                click = Some(classify_click(module, def_name, name, &p.value)?);
            }
            other => {
                return Err(err_at(
                    module,
                    p.span,
                    format!(
                        "unknown prop '{other}' on region '{name}' of node '{def_name}' (frozen: width/height/onclick)",
                    ),
                ))
            }
        }
    }
    let click = match click {
        Some((ev, child)) => {
            match &child {
                None => {
                    if !plan.events.iter().any(|e| e == &ev) {
                        return Err(err_at(
                            module,
                            click_span,
                            format!(
                                "node '{def_name}': onclick on region '{name}' references event '.{ev}' with no handler"
                            ),
                        ));
                    }
                }
                Some(child_name) => {
                    if !plan.local_instances.contains_key(child_name) {
                        return Err(err_at(
                            module,
                            click_span,
                            format!(
                                "node '{def_name}': onclick on region '{name}' references unknown instance '{child_name}'"
                            ),
                        ));
                    }
                }
            }
            plan.clicks.push((name.to_string(), ev, child));
            Some(ArtifactClick {
                fn_name: format!("{}_ui_click_{}", plan.prefix, name.to_lowercase()),
                arg: if plan.is_root { "none" } else { "index" },
            })
        }
        None => None,
    };
    // 场景元素（有界；首切片 quad/circle/cylinder）。
    let mut elems: Vec<ArtifactSceneElem> = Vec::new();
    fn check_scene_budget(
        len: usize,
        module: &str,
        def_name: &str,
        region: &str,
    ) -> Result<(), String> {
        // 检查在 push 之前：`>=` 恰好放行 8 个（canonical「每区域 ≤8」）、
        // 第 9 个具名拒绝——T-03 门曾疑 off-by-one，门实测证伪后原样保留。
        if len >= MAX_SCENE_ELEMS_PER_REGION {
            return Err(err_at(
                module,
                (0, 0),
                format!(
                    "node '{def_name}': region '{region}' exceeds the frozen {MAX_SCENE_ELEMS_PER_REGION}-element budget"
                ),
            ));
        }
        Ok(())
    }
    match elements {
        RegionElements::D2(els) => {
            for el in els {
                match el {
                    auto_ui_dialect::node2d::View2dElement::Shape {
                        name: en,
                        props: eps,
                    } => {
                        check_scene_budget(elems.len(), module, def_name, name)?;
                        let mut kind = String::new();
                        let mut x = ArtifactPropValue::LitNum { v: 0.0 };
                        let mut y = ArtifactPropValue::LitNum { v: 0.0 };
                        let mut w = ArtifactPropValue::LitNum { v: 16.0 };
                        let mut h = ArtifactPropValue::LitNum { v: 16.0 };
                        let mut r = ArtifactPropValue::LitNum { v: 8.0 };
                        let mut color: Option<[u8; 3]> = None;
                        for p in eps {
                            let one = || -> Result<&Expr, String> {
                                match &p.value {
                                    PropValue::One(e) => Ok(e),
                                    PropValue::Pair(_) => Err(err_at(
                                        module,
                                        p.span,
                                        format!(
                                            "node '{def_name}': {} on '{en}' takes a single value",
                                            p.name
                                        ),
                                    )),
                                }
                            };
                            match p.name.as_str() {
                                "kind" => kind = match one()? {
                                    Expr::Ident(s) => s.to_string(),
                                    _ => {
                                        return Err(err_at(
                                            module,
                                            p.span,
                                            format!(
                                                "node '{def_name}': kind on '{en}' must be quad|circle"
                                            ),
                                        ))
                                    }
                                },
                                "x" => x = classify_scene_num(module, def_name, en, "x", one()?, fields)?,
                                "y" => y = classify_scene_num(module, def_name, en, "y", one()?, fields)?,
                                "w" => w = classify_scene_num(module, def_name, en, "w", one()?, fields)?,
                                "h" => h = classify_scene_num(module, def_name, en, "h", one()?, fields)?,
                                "r" => r = classify_scene_num(module, def_name, en, "r", one()?, fields)?,
                                "color" => color = Some(classify_scene_color(module, def_name, en, &p.value)?),
                                other => {
                                    return Err(err_at(
                                        module,
                                        p.span,
                                        format!(
                                            "unknown prop '{other}' on shape2d '{en}' of node '{def_name}' (frozen: kind/x/y/w/h/r/color)",
                                        ),
                                    ))
                                }
                            }
                        }
                        match kind.as_str() {
                            "quad" => elems.push(ArtifactSceneElem::Quad {
                                name: en.clone(),
                                x,
                                y,
                                w,
                                h,
                                color: color.unwrap_or([200, 40, 40]),
                            }),
                            "circle" => elems.push(ArtifactSceneElem::Circle {
                                name: en.clone(),
                                x,
                                y,
                                r,
                                color: color.unwrap_or([40, 120, 220]),
                            }),
                            other => {
                                return Err(err_at(
                                    module,
                                    (0, 0),
                                    format!(
                                        "node '{def_name}': shape2d '{en}' kind '{other}' not in the frozen T-02 set (quad|circle)"
                                    ),
                                ))
                            }
                        }
                    }
                    other => {
                        return Err(err_at(
                            module,
                            (0, 0),
                            format!(
                                "node '{def_name}': region '{name}' element '{other:?}' is outside the frozen T-02 2D set (shape2d quad/circle only)"
                            ),
                        ))
                    }
                }
            }
        }
        RegionElements::D3(els) => {
            for el in els {
                match el {
                    auto_ui_dialect::node2d::View3dElement::Primitive {
                        name: en,
                        props: eps,
                    } => {
                        check_scene_budget(elems.len(), module, def_name, name)?;
                        let mut translation: Vec<ArtifactPropValue> = Vec::new();
                        let mut color: Option<[u8; 3]> = None;
                        for p in eps {
                            match p.name.as_str() {
                                "translation" => {
                                    let es = p.value.exprs();
                                    if es.len() != 3 {
                                        return Err(err_at(
                                            module,
                                            p.span,
                                            format!(
                                                "node '{def_name}': translation on '{en}' must be an (x, y, z) triple"
                                            ),
                                        ));
                                    }
                                    translation = es
                                        .iter()
                                        .map(|e| {
                                            classify_scene_num(
                                                module, def_name, en, "translation", e, fields,
                                            )
                                        })
                                        .collect::<Result<Vec<_>, _>>()?;
                                }
                                "color" => {
                                    color = Some(classify_scene_color(module, def_name, en, &p.value)?)
                                }
                                other => {
                                    return Err(err_at(
                                        module,
                                        p.span,
                                        format!(
                                            "unknown prop '{other}' on cylinder3d '{en}' of node '{def_name}' (frozen: translation/color)",
                                        ),
                                    ))
                                }
                            }
                        }
                        if translation.len() != 3 {
                            return Err(err_at(
                                module,
                                (0, 0),
                                format!(
                                    "node '{def_name}': cylinder3d '{en}' requires a 'translation' triple (frozen T-02 dynamic surface)"
                                ),
                            ));
                        }
                        elems.push(ArtifactSceneElem::Cylinder {
                            name: en.clone(),
                            translation,
                            color: color.unwrap_or([90, 160, 90]),
                        });
                    }
                    other => {
                        return Err(err_at(
                            module,
                            (0, 0),
                            format!(
                                "node '{def_name}': region '{name}' element '{other:?}' is outside the frozen T-02 3D set (cylinder3d only)"
                            ),
                        ))
                    }
                }
            }
        }
    }
    *region_count += 1;
    Ok(ArtifactElement::Embed {
        name: name.to_string(),
        dim,
        width: width.unwrap_or(160),
        height: height.unwrap_or(120),
        click,
        scene: elems,
    })
}

#[allow(clippy::too_many_arguments)]
fn lower_view(
    module: &str,
    def_name: &str,
    elements: &[UiElement],
    fields: &[FieldPlan],
    events: &[String],
    plan: &mut DefPlan,
    count: &mut usize,
    mix: bool,
    regions: &mut usize,
) -> Result<Vec<ArtifactElement>, String> {
    let mut out = Vec::new();
    for el in elements {
        *count += 1;
        if *count > MAX_ELEMENTS {
            return Err(err_at(
                module,
                (0, 0),
                format!(
                    "node '{def_name}': basic-UI element count exceeds the frozen {MAX_ELEMENTS}-element budget"
                ),
            ));
        }
        match el {
            UiElement::Container {
                kind,
                name,
                props,
                children,
            } => {
                if children.len() > MAX_CHILDREN {
                    return Err(err_at(
                        module,
                        (0, 0),
                        format!(
                            "node '{def_name}': container '{name}' exceeds the frozen {MAX_CHILDREN}-child budget"
                        ),
                    ));
                }
                let mut aprops = Vec::new();
                for p in props {
                    match p.name.as_str() {
                        "width" | "height" | "spacing" | "padding" => aprops.push(ArtifactProp {
                            name: p.name.clone(),
                            value: classify_num_prop(
                                module,
                                def_name,
                                name,
                                &p.name,
                                &p.value,
                                fields,
                            )?,
                        }),
                        other => {
                            return Err(err_at(
                                module,
                                p.span,
                                format!(
                                    "unknown prop '{other}' on container '{name}' of node '{def_name}' (frozen: width/height/spacing/padding)",
                                ),
                            ))
                        }
                    }
                }
                let children = lower_view(
                    module, def_name, children, fields, events, plan, count, mix, regions,
                )?;
                out.push(ArtifactElement::Container {
                    name: name.clone(),
                    container: match kind {
                        UiContainerKind::Row => "row",
                        UiContainerKind::Column => "column",
                    },
                    props: aprops,
                    children,
                });
            }
            UiElement::Text { name, props } => {
                let mut text = None;
                let mut visible = None;
                for p in props {
                    match p.name.as_str() {
                        "text" => {
                            text = Some(classify_text_value(
                                module, def_name, name, &p.value, fields,
                            )?)
                        }
                        "visible" => {
                            visible = Some(classify_cond_expr(
                                module,
                                def_name,
                                name,
                                match &p.value {
                                    PropValue::One(e) => e,
                                    PropValue::Pair(_) => {
                                        return Err(err_at(
                                            module,
                                            p.span,
                                            format!(
                                                "node '{def_name}': visible on '{name}' takes a single value"
                                            ),
                                        ))
                                    }
                                },
                                fields,
                            )?)
                        }
                        other => {
                            return Err(err_at(
                                module,
                                p.span,
                                format!(
                                    "unknown prop '{other}' on text '{name}' of node '{def_name}' (frozen: text/visible)",
                                ),
                            ))
                        }
                    }
                }
                let text = text.ok_or_else(|| {
                    err_at(
                        module,
                        (0, 0),
                        format!("node '{def_name}': text element '{name}' requires a 'text' prop"),
                    )
                })?;
                if plan.texts.len() >= MAX_TEXTS_PER_DEF {
                    return Err(err_at(
                        module,
                        (0, 0),
                        format!(
                            "node '{def_name}': text element count exceeds the frozen {MAX_TEXTS_PER_DEF}-element budget (64-byte buffers must stay within the SDK 256-literal array cap)"
                        ),
                    ));
                }
                let slot = plan.texts.len();
                plan.texts.push((name.clone(), text.clone()));
                out.push(ArtifactElement::Text {
                    name: name.clone(),
                    slot,
                    text,
                    visible,
                });
            }
            UiElement::Button { name, props } => {
                let mut text = None;
                let mut enabled = None;
                let mut visible = None;
                let mut click = None;
                let mut click_span = (0, 0);
                for p in props {
                    match p.name.as_str() {
                        "text" => {
                            text = Some(classify_text_value(
                                module, def_name, name, &p.value, fields,
                            )?)
                        }
                        "enabled" => {
                            enabled = Some(classify_cond_expr(
                                module,
                                def_name,
                                name,
                                match &p.value {
                                    PropValue::One(e) => e,
                                    PropValue::Pair(_) => {
                                        return Err(err_at(
                                            module,
                                            p.span,
                                            format!(
                                                "node '{def_name}': enabled on '{name}' takes a single value"
                                            ),
                                        ))
                                    }
                                },
                                fields,
                            )?)
                        }
                        "visible" => {
                            visible = Some(classify_cond_expr(
                                module,
                                def_name,
                                name,
                                match &p.value {
                                    PropValue::One(e) => e,
                                    PropValue::Pair(_) => {
                                        return Err(err_at(
                                            module,
                                            p.span,
                                            format!(
                                                "node '{def_name}': visible on '{name}' takes a single value"
                                            ),
                                        ))
                                    }
                                },
                                fields,
                            )?)
                        }
                        "onclick" => {
                            click_span = p.span;
                            click = Some(classify_click(module, def_name, name, &p.value)?)
                        }
                        other => {
                            return Err(err_at(
                                module,
                                p.span,
                                format!(
                                    "unknown prop '{other}' on button '{name}' of node '{def_name}' (frozen: text/enabled/visible/onclick)",
                                ),
                            ))
                        }
                    }
                }
                let text = text.ok_or_else(|| {
                    err_at(
                        module,
                        (0, 0),
                        format!(
                            "node '{def_name}': button element '{name}' requires a 'text' prop"
                        ),
                    )
                })?;
                let (ev, child) = click.ok_or_else(|| {
                    err_at(
                        module,
                        (0, 0),
                        format!(
                            "node '{def_name}': button element '{name}' requires an 'onclick' prop"
                        ),
                    )
                })?;
                match &child {
                    None => {
                        if !plan.events.iter().any(|e| e == &ev) {
                            return Err(err_at(
                                module,
                                click_span,
                                format!(
                                    "node '{def_name}': onclick on '{name}' references event '.{ev}' with no handler (basic-UI v1 emits zero-arg events)"
                                ),
                            ));
                        }
                    }
                    Some(child_name) => {
                        if !plan.local_instances.contains_key(child_name) {
                            return Err(err_at(
                                module,
                                click_span,
                                format!(
                                    "node '{def_name}': onclick on '{name}' references unknown instance '{child_name}'"
                                ),
                            ));
                        }
                    }
                }
                let prefix = plan.prefix.clone();
                plan.clicks.push((name.clone(), ev, child.clone()));
                // PLAN-054 T-05 补漏（注明兼容面）：按钮标签同占文字缓冲槽
                // ——此前仅文字元素检查 ≤4 预算，按钮标签使槽位可到 5，
                // 产物只在 driver 消费入口被拒（fail-late）。补上同面具名
                // 拒绝后，合法 app（≤4 含标签）产物字节不变，仅越界源从
                // 「可生成但消费被拒」改为「生成侧具名拒绝」。
                if plan.texts.len() >= MAX_TEXTS_PER_DEF {
                    return Err(err_at(
                        module,
                        (0, 0),
                        format!(
                            "node '{def_name}': text element count exceeds the frozen {MAX_TEXTS_PER_DEF}-element budget (64-byte buffers must stay within the SDK 256-literal array cap)"
                        ),
                    ));
                }
                let slot = plan.texts.len();
                plan.texts.push((name.clone(), text.clone()));
                out.push(ArtifactElement::Button {
                    name: name.clone(),
                    slot,
                    text,
                    enabled,
                    visible,
                    click: ArtifactClick {
                        fn_name: format!("{}_ui_click_{}", prefix, name.to_lowercase()),
                        arg: if plan.is_root { "none" } else { "index" },
                    },
                });
            }
            UiElement::If { cond, then } => {
                let cond = classify_cond_expr(module, def_name, "if", cond, fields)?;
                let then = lower_view(
                    module, def_name, then, fields, events, plan, count, mix, regions,
                )?;
                out.push(ArtifactElement::If { cond, then });
            }
            UiElement::Instance { node, name, .. } => {
                if !plan.local_instances.contains_key(name) {
                    // load.rs 已拒绝未声明实例；此处保持不变量。
                    return Err(err_at(
                        module,
                        (0, 0),
                        format!("node '{def_name}': instance '{name}' of undeclared node '{node}'"),
                    ));
                }
                out.push(ArtifactElement::Instance {
                    name: name.clone(),
                    node: node.clone(),
                });
            }
            UiElement::Embed2d {
                name,
                props,
                elements,
                span,
            } => {
                if !mix {
                    return Err(err_at(
                        module,
                        *span,
                        format!(
                            "node '{def_name}': embed region '{name}' is PLAN-049 mixed-profile only (node-ui rejects any view-2d/view-3d surface)"
                        ),
                    ));
                }
                let embed = lower_embed_region(
                    module,
                    def_name,
                    name,
                    "2d",
                    props,
                    RegionElements::D2(elements),
                    fields,
                    plan,
                    regions,
                )?;
                out.push(embed);
            }
            UiElement::Embed3d {
                name,
                props,
                elements,
                span,
            } => {
                if !mix {
                    return Err(err_at(
                        module,
                        *span,
                        format!(
                            "node '{def_name}': embed region '{name}' is PLAN-049 mixed-profile only (node-ui rejects any view-2d/view-3d surface)"
                        ),
                    ));
                }
                let embed = lower_embed_region(
                    module,
                    def_name,
                    name,
                    "3d",
                    props,
                    RegionElements::D3(elements),
                    fields,
                    plan,
                    regions,
                )?;
                out.push(embed);
            }
        }
    }
    Ok(out)
}

// ------------------------------------------------------ instance table --

fn collect_ui_instances(
    plans: &BTreeMap<String, DefPlan>,
    root_name: &str,
    module_for: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<InstancePlan>, String> {
    let mut out = Vec::new();
    let mut def_counts: BTreeMap<String, usize> = BTreeMap::new();
    fn props_to_params(
        module: &str,
        path: &str,
        props: &[auto_ui_dialect::node2d::Prop],
    ) -> Result<BTreeMap<String, f64>, String> {
        let mut params = BTreeMap::new();
        for p in props {
            let e = match &p.value {
                PropValue::One(e) => e,
                PropValue::Pair(_) => {
                    return Err(err_at(
                        module,
                        p.span,
                        format!(
                            "instance prop '{}' on '{path}' takes a single value",
                            p.name
                        ),
                    ))
                }
            };
            match e {
                Expr::Int(v) => {
                    params.insert(p.name.clone(), *v as f64);
                }
                Expr::Float(v, _) | Expr::Double(v, _) => {
                    params.insert(p.name.clone(), *v);
                }
                _ => {
                    return Err(err_at(
                        module,
                        p.span,
                        format!(
                        "instance prop '{}' on '{path}' must be a numeric literal (basic-UI v1)",
                        p.name
                    ),
                    ))
                }
            }
        }
        Ok(params)
    }
    fn rec(
        plans: &BTreeMap<String, DefPlan>,
        def_name: &str,
        parent_path: &str,
        def_counts: &mut BTreeMap<String, usize>,
        out: &mut Vec<InstancePlan>,
        module_for: &dyn Fn(&str) -> Option<String>,
    ) -> Result<(), String> {
        let plan = plans.get(def_name).expect("def plan");
        let Some(view) = &plan.node.view else {
            return Ok(());
        };
        fn walk(
            plans: &BTreeMap<String, DefPlan>,
            els: &[UiElement],
            parent_path: &str,
            def_counts: &mut BTreeMap<String, usize>,
            out: &mut Vec<InstancePlan>,
            module_for: &dyn Fn(&str) -> Option<String>,
        ) -> Result<(), String> {
            for el in els {
                match el {
                    UiElement::Instance { node, name, props } => {
                        let child = plans.get(node).ok_or_else(|| {
                            format!("instance '{name}' of undeclared node '{node}'")
                        })?;
                        if child.node.view.is_none() {
                            let module = module_for(node).unwrap_or_default();
                            return Err(err_at(
                                &module,
                                (0, 0),
                                format!(
                                    "node '{node}' is instantiated in a basic-UI project but declares no view (graphics embedding is PLAN-049)"
                                ),
                            ));
                        }
                        let def_idx = *def_counts.entry(node.clone()).or_insert(0);
                        def_counts.insert(node.clone(), def_idx + 1);
                        let module = module_for(node).unwrap_or_default();
                        let path = format!("{parent_path}/{name}");
                        let params = props_to_params(&module, &path, props)?;
                        out.push(InstancePlan {
                            path,
                            def: node.clone(),
                            def_idx,
                            params,
                        });
                        let child_path = out.last().unwrap().path.clone();
                        rec(plans, node, &child_path, def_counts, out, module_for)?;
                    }
                    UiElement::Container { children, .. } => {
                        walk(plans, children, parent_path, def_counts, out, module_for)?
                    }
                    UiElement::If { then, .. } => {
                        walk(plans, then, parent_path, def_counts, out, module_for)?
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        walk(
            plans,
            &view.elements,
            parent_path,
            def_counts,
            out,
            module_for,
        )
    }
    rec(
        plans,
        root_name,
        root_name,
        &mut def_counts,
        &mut out,
        module_for,
    )?;
    if out.len() + 1 > MAX_INSTANCES {
        let module = module_for(root_name).unwrap_or_default();
        return Err(err_at(
            &module,
            (0, 0),
            format!(
                "basic-UI instance count exceeds the frozen {MAX_INSTANCES}-instance envelope (root included, total {})",
                out.len() + 1
            ),
        ));
    }
    Ok(out)
}

// ---------------------------------------------------------- generation --

fn init_text_of(value: &Init) -> Option<i64> {
    match value {
        Init::Int(v) => Some(*v),
        Init::Bool(b) => Some(*b as i64),
        _ => None,
    }
}

/// 单文字元素的重建语句（写入 + len）。槽位基于实例 i。
/// 每实例缓冲写（实例下标 k 为字面量；数组名带 _{k} 后缀）。
fn gen_text_write_k(
    plan: &DefPlan,
    k: usize,
    elem_index: usize,
    value: &ArtifactTextValue,
    src: &mut String,
) {
    let p = &plan.prefix;
    // 半缓冲寻址：e >= TEXT_SLOTS_PER_BUFFER 写 b 半缓冲且按半内偏移
    // （(e-槽)*64）——整槽偏移会越界（门证据：Counter 槽2 标签 128/64）。
    let (buf, half_index) = if elem_index < TEXT_SLOTS_PER_BUFFER {
        (format!("{p}_ui_text_{k}"), elem_index)
    } else {
        (
            format!("{p}_ui_text_b_{k}"),
            elem_index - TEXT_SLOTS_PER_BUFFER,
        )
    };
    let base = format!("({half_index}*{TEXT_BYTES})");
    match value {
        ArtifactTextValue::LitStr { v } => {
            for (j, b) in v.as_bytes().iter().enumerate() {
                let _ = writeln!(src, " {buf}[({base}+{j})]=0x{b:02X}");
            }
            let _ = writeln!(
                src,
                " {p}_ui_text_len_{k}[{elem_index}]={}",
                v.as_bytes().len()
            );
        }
        ArtifactTextValue::IntField { name } => {
            let fv = if plan.is_root {
                format!("r_{name}")
            } else {
                format!("{p}_{name}[{k}]")
            };
            let _ = writeln!(
                src,
                " {p}_ui_text_len_{k}[{elem_index}]={p}_ui_digits_{k}({elem_index},{fv})"
            );
        }
        ArtifactTextValue::BoolField { name } => {
            let fv = if plan.is_root {
                format!("r_{name}")
            } else {
                format!("{p}_{name}[{k}]")
            };
            // PLAN-054 T-05 修正（AG021 实测反例；注明兼容面）：bool 文字
            // 曾硬写缓冲 A（`_ui_text_`）而忽略半缓冲寻址——槽 ≥2 时覆盖
            // 槽 0/1 的字节而 len[] 仍报旧内容（len/字节不一致、显示错文）。
            // 修正后与 LitStr/IntField 同一面：`{buf}`＋半内偏移。仅
            // 「bool 恰在槽 ≥2」的生成字节从错写改对写；槽 0/1 与
            // int/const 文字路径逐字不变。
            let _ = writeln!(src, " if {fv}==1 {{");
            for (j, b) in b"true".iter().enumerate() {
                let _ = writeln!(src, "  {buf}[({base}+{j})]=0x{b:02X}");
            }
            let _ = writeln!(src, "  {p}_ui_text_len_{k}[{elem_index}]=4");
            let _ = writeln!(src, " }} else {{");
            for (j, b) in b"false".iter().enumerate() {
                let _ = writeln!(src, "  {buf}[({base}+{j})]=0x{b:02X}");
            }
            let _ = writeln!(src, "  {p}_ui_text_len_{k}[{elem_index}]=5");
            let _ = writeln!(src, " }}");
        }
    }
}

/// 每实例重建 fn（无 i；k 为字面量）。
fn gen_refresh_instance(plan: &DefPlan, k: usize, src: &mut String) {
    let p = &plan.prefix;
    let _ = writeln!(src, "fn {p}_ui_refresh_{k}() {{");
    for (e, (_, value)) in plan.texts.iter().enumerate() {
        gen_text_write_k(plan, k, e, value, src);
    }
    let _ = writeln!(src, "}}");
}

/// 重建派发器（公开 ABI：i = 实例下标，root 恒 0）。
fn gen_refresh_dispatch(plan: &DefPlan, n: usize, src: &mut String) {
    if plan.texts.is_empty() || n == 0 {
        return;
    }
    let p = &plan.prefix;
    let _ = writeln!(src, "fn {p}_ui_refresh(i int) {{");
    for k in 0..n {
        let _ = writeln!(src, " if i=={k} {{");
        let _ = writeln!(src, "  {p}_ui_refresh_{k}()");
        let _ = writeln!(src, " }}");
    }
    let _ = writeln!(src, "}}");
}

/// 数字格式化（每实例；at = e*64）＋有界最小位数十进制。
/// 数字格式化（每实例；at = e*64）＋有界最小位数十进制（1..=7 位）。
fn gen_digits_fn(prefix: &str, k: usize, src: &mut String) {
    let _ = writeln!(src, "fn {prefix}_ui_digits_{k}(e int, n int) int {{");
    // 半缓冲分派：e >= TEXT_SLOTS_PER_BUFFER 写 b 半缓冲（同一 at 公式）。
    let _ = writeln!(
        src,
        " let buf=if e<{SLOTS}{{1}}else{{0}}",
        SLOTS = TEXT_SLOTS_PER_BUFFER
    );
    let _ = writeln!(
        src,
        " let at=((e-({SLOTS}*(1-buf)))*{TEXT_BYTES})",
        SLOTS = TEXT_SLOTS_PER_BUFFER
    );
    let _ = writeln!(src, " if n>9999999 {{");
    for (j, b) in b"OVER".iter().enumerate() {
        let _ = writeln!(src, "  if buf==1 {{{prefix}_ui_text_{k}[(at+{j})]=0x{b:02X}}}else{{{prefix}_ui_text_b_{k}[(at+{j})]=0x{b:02X}}}");
    }
    let _ = writeln!(src, "  return 4");
    let _ = writeln!(src, " }}");
    let _ = writeln!(src, " if n<0 {{");
    let _ = writeln!(
        src,
        "  if buf==1 {{{prefix}_ui_text_{k}[at]=0x2D}}else{{{prefix}_ui_text_b_{k}[at]=0x2D}}"
    );
    let _ = writeln!(
        src,
        "  return (1+{prefix}_ui_digits_mag_{k}(buf,(at+1),(0-n)))"
    );
    let _ = writeln!(src, " }}");
    let _ = writeln!(src, " return {prefix}_ui_digits_mag_{k}(buf,at,n)");
    let _ = writeln!(src, "}}");
    // n 的十进制位数 = d（分支自高到低；digit j = (n/10^j)%10）。
    let _ = writeln!(
        src,
        "fn {prefix}_ui_digits_mag_{k}(buf int, at int, n int) int {{"
    );
    for d in (1..=7).rev() {
        let pow = 10i64.pow(d as u32 - 1);
        let cond = if d == 1 {
            "1".to_string()
        } else {
            format!("n>={pow}")
        };
        let _ = writeln!(src, " if {cond} {{");
        // MSB-first（自然阅读序）：at+j = 第 (d-1-j) 位数字。
        for j in 0..d {
            let p10 = 10i64.pow(d as u32 - 1 - j as u32);
            let _ = writeln!(src, "  if buf==1 {{{prefix}_ui_text_{k}[(at+{j})]=(48+((n/{p10})%10))}}else{{{prefix}_ui_text_b_{k}[(at+{j})]=(48+((n/{p10})%10))}}");
        }
        let _ = writeln!(src, "  return {d}");
        let _ = writeln!(src, " }}");
    }
    let _ = writeln!(src, " return 1");
    let _ = writeln!(src, "}}");
}

/// 文字 getter（公开 ABI 不变：len(i,e)/at(i,e,j)；内部按实例分派）。
fn gen_text_getters(plan: &DefPlan, n: usize, src: &mut String) {
    if plan.texts.is_empty() || n == 0 {
        return;
    }
    let p = &plan.prefix;
    let _ = writeln!(src, "fn {p}_ui_text_len(i int, e int) int {{");
    for k in 0..n {
        let _ = writeln!(src, " if i=={k} {{");
        let _ = writeln!(src, "  return {p}_ui_text_len_{k}[e]");
        let _ = writeln!(src, " }}");
    }
    let _ = writeln!(src, " return 0");
    let _ = writeln!(src, "}}");
    let _ = writeln!(src, "fn {p}_ui_text_at(i int, e int, j int) int {{");
    for k in 0..n {
        let _ = writeln!(src, " if i=={k} {{");
        let _ = writeln!(
            src,
            "  if e<{SLOTS} {{return {p}_ui_text_{k}[((e*{TEXT_BYTES})+j)]}}else{{return {p}_ui_text_b_{k}[(((e-{SLOTS})*{TEXT_BYTES})+j)]}}",
            SLOTS = TEXT_SLOTS_PER_BUFFER
        );
        let _ = writeln!(src, " }}");
    }
    let _ = writeln!(src, " return 0");
    let _ = writeln!(src, "}}");
}

/// handler 体降层上下文（emit/child_emit/call_scope 最小面）。
fn ui_ctx<'a>(
    target: Target,
    project: &'a Project,
    plan: &'a DefPlan,
    child_recv: &'a BTreeMap<(String, String), String>,
) -> Ctx<'a> {
    let prefix = plan.prefix.clone();
    let index_text = match &target {
        Target::Root => String::new(),
        Target::Instance { index, .. } => index.clone(),
    };
    let emit_prefix = prefix.clone();
    let has_index = !index_text.is_empty();
    let idx = index_text.clone();
    let emit_hook = Box::new(move |ev: &str| -> Result<String, LowerError> {
        if has_index {
            Ok(format!("{emit_prefix}_{}({idx})", ev.to_lowercase()))
        } else {
            Ok(format!("{emit_prefix}_{}()", ev.to_lowercase()))
        }
    });
    let module = plan.module.clone();
    let child_emit = Box::new(
        move |child: &str, ev: &str, _argc: usize| -> Result<String, LowerError> {
            match child_recv.get(&(child.to_string(), ev.to_string())) {
                Some(head) => Ok(format!("{head}")),
                None => Err(LowerError::Unsupported {
                    what: format!("emit {child}.{ev} has no lowering-time receiver"),
                    line: 0,
                }),
            }
        },
    );
    let scope_module = plan.module.clone();
    let call_scope = Box::new(move |f: &str| -> Option<String> {
        let m = project.modules.get(&scope_module)?;
        if m.fns.iter().any(|(n, _)| n == f) {
            Some(format!("{scope_module}_{f}"))
        } else {
            None
        }
    });
    Ctx {
        target,
        ahud3: None,
        emit_hook,
        enum_path: Box::new(|_, _| None),
        axis_differential: false,
        member_hook: None,
        moved_flag: None,
        refs_any_elem: false,
        call_scope: Some(call_scope),
        child_emit: Some(child_emit),
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
    }
}

fn collect_local_instances(
    view: &auto_ui_dialect::node2d::ViewUiBlock,
) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    fn walk(els: &[UiElement], m: &mut BTreeMap<String, String>) {
        for el in els {
            match el {
                UiElement::Instance { node, name, .. } => {
                    m.insert(name.clone(), node.clone());
                }
                UiElement::Container { children, .. } => walk(children, m),
                UiElement::If { then, .. } => walk(then, m),
                _ => {}
            }
        }
    }
    walk(&view.elements, &mut m);
    m
}

fn root_has_life(plans: &BTreeMap<String, DefPlan>, root: &str, event: &str) -> bool {
    plans
        .get(root)
        .and_then(|p| p.node.on.as_ref())
        .is_some_and(|on| {
            on.handlers.iter().any(|h| {
                matches!(
                    &h.pattern,
                    auto_ui_dialect::node2d::EventPattern::Life { event: e, .. } if e == event
                )
            })
        })
}

fn def_has_life(plans: &BTreeMap<String, DefPlan>, def: &str, event: &str) -> bool {
    plans
        .get(def)
        .and_then(|p| p.node.on.as_ref())
        .is_some_and(|on| {
            on.handlers.iter().any(|h| {
                matches!(
                    &h.pattern,
                    auto_ui_dialect::node2d::EventPattern::Life { event: e, .. } if e == event
                )
            })
        })
}

/// def 层 handler（lifestart/lifetick/recv，带实例下标 i）。
fn gen_def_handlers(
    project: &Project,
    plans: &BTreeMap<String, DefPlan>,
    def_name: &str,
    plan: &DefPlan,
    recv_heads: &BTreeMap<(String, String), String>,
    src: &mut String,
) -> Result<(), String> {
    let p = &plan.prefix;
    let Some(on) = &plan.node.on else {
        return Ok(());
    };
    let indexed = if plan.is_root {
        Target::Root
    } else {
        Target::Instance {
            prefix: p.clone(),
            index: "i".into(),
        }
    };
    for h in &on.handlers {
        let head = match &h.pattern {
            auto_ui_dialect::node2d::EventPattern::Life { event, .. } => match event.as_str() {
                "Start" => {
                    if plan.is_root {
                        format!("fn {p}_lifestart() {{\n")
                    } else {
                        format!("fn {p}_lifestart(i int) {{\n")
                    }
                }
                "Tick" => {
                    if plan.is_root {
                        format!("fn {p}_lifetick(dt float) {{\n")
                    } else {
                        format!("fn {p}_lifetick(i int, dt float) {{\n")
                    }
                }
                other => {
                    return Err(err_at(
                        &plan.module,
                        h.span,
                        format!(
                            "unknown Life event '{other}' (research surface: Life.Start/Life.Tick)"
                        ),
                    ))
                }
            },
            auto_ui_dialect::node2d::EventPattern::Member { name, .. } => {
                if plan.is_root {
                    format!("fn {p}_{}() {{\n", name.to_lowercase())
                } else {
                    format!("fn {p}_{}(i int) {{\n", name.to_lowercase())
                }
            }
        };
        src.push_str(&head);
        let ctx = ui_ctx(indexed.clone(), project, plan, recv_heads);
        translate_handler(&h.body.stmts, &ctx, src, 1).map_err(|e| format!("{e}"))?;
        src.push_str("}\n");
    }
    Ok(())
}

fn gen_click_dispatchers(
    plans: &BTreeMap<String, DefPlan>,
    root_name: &str,
    instances: &[InstancePlan],
    def_name: &str,
    plan: &DefPlan,
    src: &mut String,
) -> Result<(), String> {
    let p = &plan.prefix;
    for (elem, ev, child) in &plan.clicks {
        let el = elem.to_lowercase();
        let ev_low = ev.to_lowercase();
        match (&plan.is_root, child) {
            (true, None) => {
                let _ = writeln!(src, "fn {p}_ui_click_{el}() {{");
                let _ = writeln!(src, " {p}_{ev_low}()");
                if !plan.texts.is_empty() {
                    let _ = writeln!(src, " {p}_ui_refresh(0)");
                }
                let _ = writeln!(src, "}}");
            }
            (true, Some(child_name)) => {
                // root 的子实例 def_idx 是常量（root 下唯一实例化点）。
                let child_def = plan
                    .local_instances
                    .get(child_name)
                    .ok_or_else(|| format!("node '{def_name}': unknown instance '{child_name}'"))?;
                let cp = plans.get(child_def).unwrap().prefix.clone();
                let idx = instances
                    .iter()
                    .find(|i| i.path == format!("{root_name}/{child_name}"))
                    .map(|i| i.def_idx)
                    .ok_or_else(|| {
                        format!(
                            "node '{def_name}': onclick on '{elem}' references instance '{child_name}' not instantiated under the root"
                        )
                    })?;
                let _ = writeln!(src, "fn {p}_ui_click_{el}() {{");
                let _ = writeln!(src, " {cp}_{ev_low}({idx})");
                if !plans.get(child_def).unwrap().texts.is_empty() {
                    let _ = writeln!(src, " {cp}_ui_refresh({idx})");
                }
                if !plan.texts.is_empty() {
                    let _ = writeln!(src, " {p}_ui_refresh(0)");
                }
                let _ = writeln!(src, "}}");
            }
            (false, None) => {
                let _ = writeln!(src, "fn {p}_ui_click_{el}(i int) {{");
                let _ = writeln!(src, " {p}_{ev_low}(i)");
                if !plan.texts.is_empty() {
                    let _ = writeln!(src, " {p}_ui_refresh(i)");
                }
                let _ = writeln!(src, "}}");
            }
            (false, Some(child_name)) => {
                let child_def = plan
                    .local_instances
                    .get(child_name)
                    .ok_or_else(|| format!("node '{def_name}': unknown instance '{child_name}'"))?;
                let cp = plans.get(child_def).unwrap().prefix.clone();
                let ev_up = ev.to_uppercase();
                let _ = writeln!(src, "fn {p}_ui_click_{el}(i int) {{");
                let _ = writeln!(src, " let j={p}_bind_{child_name}_{ev_up}[i]");
                let _ = writeln!(src, " {cp}_{ev_low}(j)");
                if !plans.get(child_def).unwrap().texts.is_empty() {
                    let _ = writeln!(src, " {cp}_ui_refresh(j)");
                }
                if !plan.texts.is_empty() {
                    let _ = writeln!(src, " {p}_ui_refresh(i)");
                }
                let _ = writeln!(src, "}}");
            }
        }
    }
    Ok(())
}

/// 管线产物（048/049 两入口共用的内部面；产物 JSON 由各入口自组装）。
struct PipelineOut {
    user_source: String,
    root: String,
    instances: Vec<ArtifactInstance>,
    world_sha256: String,
    source_bytes: usize,
    def_plans: BTreeMap<String, (String, bool, usize, Vec<ArtifactElement>)>,
}

/// 共享降层管线：基础 UI 工程 → world.at（048/049 字节同面；区域点击
/// 走同一 `_ui_click_` 派发器 ABI）。`mix=false` 即 PLAN-048 路径——
/// 区域解析头具名拒绝，全部产物字节不变。
fn lower_node_ui_pipeline(
    project: &Project,
    mix: bool,
    regions_v2: bool,
) -> Result<PipelineOut, String> {
    let root_name = project.root.clone();
    // ---- def 计划 ----
    let mut plans: BTreeMap<String, DefPlan> = BTreeMap::new();
    for module in project.modules.values() {
        for node in &module.nodes {
            let module_name = module.name.clone();
            let is_root = node.name == root_name;
            if is_root && node.view.is_none() {
                return Err(err_at(
                    &module_name,
                    node.span,
                    format!("root node '{root_name}' must declare a basic-UI view (PLAN-048)"),
                ));
            }
            let params = node.params.clone();
            let mut fields = Vec::new();
            if let Some(model) = &node.model {
                for f in &model.fields {
                    fields.push(classify_field(&module_name, &node.name, f, &params)?);
                }
            }
            let mut events = Vec::new();
            let mut payload_events: Vec<String> = Vec::new();
            if let Some(msg) = &node.msg {
                for v in &msg.variants {
                    if !v.payload.is_empty() {
                        payload_events.push(v.name.to_string());
                    }
                }
            }
            if let Some(on) = &node.on {
                for h in &on.handlers {
                    if let auto_ui_dialect::node2d::EventPattern::Member { name, params: hp } =
                        &h.pattern
                    {
                        if !hp.is_empty() {
                            return Err(err_at(
                                &module_name,
                                h.span,
                                format!(
                                    "node '{}': basic-UI v1 emits zero-arg events ('.{}' declares params)",
                                    node.name, name
                                ),
                            ));
                        }
                        let low = name.to_lowercase();
                        if low == "lifestart" || low == "lifetick" {
                            return Err(err_at(
                                &module_name,
                                h.span,
                                format!(
                                    "node '{}': event name '{name}' collides with the reserved Life handlers",
                                    node.name
                                ),
                            ));
                        }
                        if payload_events.iter().any(|v| v.to_lowercase() == low) {
                            return Err(err_at(
                                &module_name,
                                h.span,
                                format!(
                                    "node '{}': basic-UI v1 emits zero-arg events ('.{}' has a declared payload)",
                                    node.name, name
                                ),
                            ));
                        }
                        events.push(name.clone());
                    }
                }
            }
            let local_instances = node
                .view
                .as_ref()
                .map(|v| collect_local_instances(v))
                .unwrap_or_default();
            let name = node.name.clone();
            plans.insert(
                name.clone(),
                DefPlan {
                    name: name.clone(),
                    prefix: name.to_lowercase(),
                    is_root,
                    regions_v2,
                    module: module_name,
                    node: node.clone(),
                    fields,
                    texts: Vec::new(),
                    elements: Vec::new(),
                    clicks: Vec::new(),
                    events,
                    local_instances,
                },
            );
        }
    }
    // ---- 元素树（每 def）----
    let def_names: Vec<String> = plans.keys().cloned().collect();
    for def_name in &def_names {
        let Some(view) = plans.get(def_name).unwrap().node.view.clone() else {
            continue;
        };
        let (module, fields, events) = {
            let p = plans.get(def_name).unwrap();
            (p.module.clone(), p.fields.clone(), p.events.clone())
        };
        let mut count = 0usize;
        let mut region_count = 0usize;
        let elements = {
            let plan = plans.get_mut(def_name).unwrap();
            lower_view(
                &module,
                def_name,
                &view.elements,
                &fields,
                &events,
                plan,
                &mut count,
                mix,
                &mut region_count,
            )?
        };
        plans.get_mut(def_name).unwrap().elements = elements;
    }
    // ---- 实例表 ----
    let module_for = |node: &str| -> Option<String> {
        project
            .node_modules
            .get(node)
            .and_then(|m| project.modules.get(m))
            .map(|m| m.name.clone())
    };
    let instances = collect_ui_instances(&plans, &root_name, &module_for)?;
    // ---- 生成 world.at ----
    let root_prefix = plans.get(&root_name).unwrap().prefix.clone();
    let mut src = String::new();
    let _ = writeln!(
        src,
        "// GENERATED by autographics-node2d-pilot (PLAN-048 node-ui, T-03/T-04 contract)."
    );
    let _ = write!(
        src,
        "// Root: {root_name}; instances (declaration pre-order): "
    );
    let _ = writeln!(
        src,
        "{}",
        instances
            .iter()
            .map(|i| format!("{}:{}", i.path, i.def))
            .collect::<Vec<_>>()
            .join(",")
    );
    let _ = writeln!(
        src,
        "// Regeneration must be byte-identical (tools/verify_plan048.py)."
    );
    let _ = writeln!(src, "var ok=0.0");
    // root 标量（int/bool；str 字段只作文字内联，不过 VM）。
    let root_plan = plans.get(&root_name).unwrap();
    for f in &root_plan.fields {
        if f.kind == Kind::Str {
            continue;
        }
        let v = init_text_of(&f.init).unwrap_or(0);
        let _ = writeln!(src, "var r_{}={v}", f.name);
    }
    // 每 def SoA（非 root；param 引用按实例内联；str 字段不过 VM）。
    for (def_name, plan) in &plans {
        if plan.is_root {
            continue;
        }
        let own: Vec<_> = instances.iter().filter(|i| i.def == *def_name).collect();
        if own.is_empty() {
            continue;
        }
        for f in &plan.fields {
            if f.kind == Kind::Str {
                continue;
            }
            let vals = own
                .iter()
                .map(|inst| match &f.init {
                    Init::Param(name) => {
                        format!("{}", inst.params.get(name).copied().unwrap_or(0.0) as i64)
                    }
                    other => format!("{}", init_text_of(other).unwrap_or(0)),
                })
                .collect::<Vec<_>>()
                .join(",");
            let _ = writeln!(src, "var {}_{}=[{vals}]", plan.prefix, f.name);
        }
        // 运行时参数 SoA（int 参数；handler/绑定引用经成员映射取址）。
        for (pname, pty) in &plan.node.params {
            if !matches!(pty, Type::Int) {
                continue;
            }
            let vals = own
                .iter()
                .map(|inst| {
                    format!(
                        "{}",
                        inst.params.get(pname.as_str()).copied().unwrap_or(0.0) as i64
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let _ = writeln!(src, "var {}_{}=[{vals}]", plan.prefix, pname);
        }
    }
    // 文字缓冲 + len（每实例一数组；PLAN-049 T-02 冻结：字节缓冲按半缓冲
    // 拆分——SDK 字面量数组恰 256 元素静默截断为 0；每半 ≤128。T ≤ 4 在
    // 降层具名拒绝）。
    for plan in plans.values() {
        if plan.texts.is_empty() {
            continue;
        }
        let n: usize = if plan.is_root {
            1
        } else {
            instances.iter().filter(|i| i.def == plan.name).count()
        };
        for k in 0..n {
            // PLAN-049 T-02 冻结：SDK 字面量数组恰 256 元素静默截断为 0
            // （048 复审实测 256→0；4×64 恰在界上）。字节缓冲按半缓冲拆分
            // （每半 ≤2 槽×64=128）；公开 len/at ABI 不变（getter 内部
            // 分派）。len 数组 ≤4 元素，不触界。
            let lo = plan.texts.len().min(TEXT_SLOTS_PER_BUFFER);
            let hi = plan.texts.len().saturating_sub(TEXT_SLOTS_PER_BUFFER);
            let zeros = vec!["0"; lo * TEXT_BYTES].join(",");
            let _ = writeln!(src, "var {}_ui_text_{k}=[{zeros}]", plan.prefix);
            if hi > 0 {
                let zeros = vec!["0"; hi * TEXT_BYTES].join(",");
                let _ = writeln!(src, "var {}_ui_text_b_{k}=[{zeros}]", plan.prefix);
            }
            let zeros = vec!["0"; plan.texts.len()].join(",");
            let _ = writeln!(src, "var {}_ui_text_len_{k}=[{zeros}]", plan.prefix);
        }
    }
    // 绑定表（非 root 父 def 的子事件派发）。
    for (def_name, plan) in &plans {
        if plan.is_root {
            continue;
        }
        let own: Vec<_> = instances.iter().filter(|i| i.def == *def_name).collect();
        if own.is_empty() {
            continue;
        }
        for (_, ev, child) in &plan.clicks {
            let Some(child_name) = child else { continue };
            let entries = own
                .iter()
                .map(|parent| {
                    instances
                        .iter()
                        .find(|i| i.path == format!("{}/{}", parent.path, child_name))
                        .map(|i| i.def_idx.to_string())
                        .unwrap_or_else(|| "0".into())
                })
                .collect::<Vec<_>>()
                .join(",");
            let _ = writeln!(
                src,
                "var {p}_bind_{c}_{ev}=[{entries}]",
                p = plan.prefix,
                c = child_name,
                ev = ev.to_uppercase()
            );
        }
    }
    // 数字助手（有 IntField 文字的 def；每实例一对）。
    for plan in plans.values() {
        let needs = plan
            .texts
            .iter()
            .any(|(_, v)| matches!(v, ArtifactTextValue::IntField { .. }));
        if !needs {
            continue;
        }
        let n: usize = if plan.is_root {
            1
        } else {
            instances.iter().filter(|i| i.def == plan.name).count()
        };
        for k in 0..n {
            gen_digits_fn(&plan.prefix, k, &mut src);
        }
    }
    // 子事件 recv 头（handler 内 `emit child.Ev` 与 onclick 派发共用面）。
    let mut recv_heads: BTreeMap<String, BTreeMap<(String, String), String>> = BTreeMap::new();
    for (def_name, plan) in &plans {
        let mut heads = BTreeMap::new();
        for (elem, ev, child) in &plan.clicks {
            let Some(child_name) = child else { continue };
            let child_def = plan
                .local_instances
                .get(child_name)
                .ok_or_else(|| format!("node '{def_name}': unknown instance '{child_name}'"))?;
            let child_prefix = plans.get(child_def).unwrap().prefix.clone();
            let ev_low = ev.to_lowercase();
            if plan.is_root {
                let idx = instances
                    .iter()
                    .find(|i| i.path == format!("{root_name}/{child_name}"))
                    .map(|i| i.def_idx)
                    .ok_or_else(|| {
                        format!(
                            "node '{def_name}': onclick on '{elem}' references instance '{child_name}' not instantiated under the root"
                        )
                    })?;
                heads.insert(
                    (child_name.clone(), ev.clone()),
                    format!("{child_prefix}_{ev_low}({idx})"),
                );
            } else {
                let ev_up = ev.to_uppercase();
                heads.insert(
                    (child_name.clone(), ev.clone()),
                    format!(
                        "{child_prefix}_{ev_low}({}_bind_{child_name}_{ev_up}[i])",
                        plan.prefix
                    ),
                );
            }
        }
        recv_heads.insert(def_name.clone(), heads);
    }
    // handlers（lifestart/lifetick/recv）+ refresh + click 派发器。
    for (def_name, plan) in &plans {
        let empty_heads = BTreeMap::new();
        let heads = recv_heads.get(def_name).unwrap_or(&empty_heads);
        gen_def_handlers(project, &plans, def_name, plan, heads, &mut src)?;
        if !plan.texts.is_empty() {
            let n: usize = if plan.is_root {
                1
            } else {
                instances.iter().filter(|i| i.def == plan.name).count()
            };
            for k in 0..n {
                gen_refresh_instance(plan, k, &mut src);
            }
            gen_refresh_dispatch(plan, n, &mut src);
        }
        gen_click_dispatchers(&plans, &root_name, &instances, def_name, plan, &mut src)?;
    }
    // getters（状态绑定读取面）。
    for plan in plans.values() {
        for f in &plan.fields {
            if f.kind == Kind::Str {
                continue;
            }
            if plan.is_root {
                let _ = writeln!(src, "fn get_r_{f}() int {{return r_{f}}}", f = f.name);
            }
        }
        if plan.is_root {
            continue;
        }
        let own: Vec<_> = instances.iter().filter(|i| i.def == plan.name).collect();
        for inst in &own {
            for f in &plan.fields {
                if f.kind == Kind::Str {
                    continue;
                }
                let pathu = inst.path.replace('/', "_");
                let _ = writeln!(
                    src,
                    "fn get_{p}_{f}_{idx}() int {{return {p}_{f}[{idx}]}}",
                    p = plan.prefix,
                    f = f.name,
                    idx = inst.def_idx
                );
                let _ = writeln!(
                    src,
                    "fn get_{pathu}_{f}() int {{return {p}_{f}[{idx}]}}",
                    pathu = pathu,
                    p = plan.prefix,
                    f = f.name,
                    idx = inst.def_idx
                );
            }
        }
    }
    // UI 文字 getter（每 def；公开 ABI 不变，内部按实例分派）。
    for plan in plans.values() {
        if plan.texts.is_empty() {
            continue;
        }
        let n: usize = if plan.is_root {
            1
        } else {
            instances.iter().filter(|i| i.def == plan.name).count()
        };
        gen_text_getters(plan, n, &mut src);
    }
    // app_start / app_update（声明先序派发 + 全量 refresh）。
    let _ = writeln!(src, "fn app_start() {{");
    if root_has_life(&plans, &root_name, "Start") {
        let _ = writeln!(src, " {root_prefix}_lifestart()");
    }
    for inst in &instances {
        if def_has_life(&plans, &inst.def, "Start") {
            let _ = writeln!(
                src,
                " {}_lifestart({})",
                plans[&inst.def].prefix, inst.def_idx
            );
        }
    }
    if !plans.get(&root_name).unwrap().texts.is_empty() {
        let _ = writeln!(src, " {root_prefix}_ui_refresh(0)");
    }
    for inst in &instances {
        if !plans[&inst.def].texts.is_empty() {
            let _ = writeln!(
                src,
                " {}_ui_refresh({})",
                plans[&inst.def].prefix, inst.def_idx
            );
        }
    }
    let _ = writeln!(src, "}}");
    let _ = writeln!(src, "fn app_update(dt float) {{");
    if root_has_life(&plans, &root_name, "Tick") {
        let _ = writeln!(src, " {root_prefix}_lifetick(dt)");
    }
    for inst in &instances {
        if def_has_life(&plans, &inst.def, "Tick") {
            let _ = writeln!(
                src,
                " {}_lifetick({},dt)",
                plans[&inst.def].prefix, inst.def_idx
            );
        }
    }
    if !plans.get(&root_name).unwrap().texts.is_empty() {
        let _ = writeln!(src, " {root_prefix}_ui_refresh(0)");
    }
    for inst in &instances {
        if !plans[&inst.def].texts.is_empty() {
            let _ = writeln!(
                src,
                " {}_ui_refresh({})",
                plans[&inst.def].prefix, inst.def_idx
            );
        }
    }
    let _ = writeln!(src, "}}");
    // ---- 产物 ----
    let source_sha256 = sha256_hex(src.as_bytes());
    let def_plans = plans
        .into_iter()
        .map(|(name, p)| {
            (
                name,
                (p.prefix, p.is_root, p.texts.len(), p.elements.clone()),
            )
        })
        .collect();
    Ok(PipelineOut {
        source_bytes: src.len(),
        user_source: src,
        root: root_name,
        instances: instances
            .into_iter()
            .map(|i| ArtifactInstance {
                path: i.path,
                node: i.def,
                def_idx: i.def_idx,
            })
            .collect(),
        world_sha256: source_sha256,
        def_plans,
    })
}

/// 主降层入口（PLAN-048）：基础 UI 工程 → world.at ＋ ui.json。
pub fn lower_project_node_ui(project: &Project) -> Result<LowerOutputNodeUi, String> {
    let out = lower_node_ui_pipeline(project, false, false)?;
    let source_sha256 = out.world_sha256.clone();
    let defs_json: BTreeMap<String, ArtifactDef> = out
        .def_plans
        .into_iter()
        .map(|(name, (prefix, is_root, texts, elements))| {
            (
                name,
                ArtifactDef {
                    prefix,
                    is_root,
                    texts,
                    elements,
                },
            )
        })
        .collect();
    let artifact = UiArtifact {
        schema: NODE_UI_SCHEMA,
        generator: NODE_UI_GENERATOR,
        world_sha256: out.world_sha256,
        root: out.root,
        instances: out.instances,
        defs: defs_json,
    };
    let ui_json = serde_json::to_string_pretty(&artifact).expect("ui.json");
    Ok(LowerOutputNodeUi {
        source_bytes: out.source_bytes,
        user_source: out.user_source,
        source_sha256,
        ui_json,
    })
}

/// 主降层入口（PLAN-049 mixed-ui-2d-3d）：同一管线，产物为混合 schema
/// （def 内嵌 regions；世界源与 048 字节同面——区域点击派发器同一 ABI）。
pub fn lower_project_node_mixed(project: &Project) -> Result<LowerOutputNodeMixed, String> {
    let out = lower_node_ui_pipeline(project, true, false)?;
    let defs_json: BTreeMap<String, ArtifactDef> = out
        .def_plans
        .into_iter()
        .map(|(name, (prefix, is_root, texts, elements))| {
            (
                name,
                ArtifactDef {
                    prefix,
                    is_root,
                    texts,
                    elements,
                },
            )
        })
        .collect();
    let artifact = UiArtifact {
        schema: NODE_UI_MIXED_SCHEMA,
        generator: NODE_UI_MIXED_GENERATOR,
        world_sha256: out.world_sha256.clone(),
        root: out.root,
        instances: out.instances,
        defs: defs_json,
    };
    let ui_json = serde_json::to_string_pretty(&artifact).expect("ui.json");
    Ok(LowerOutputNodeMixed {
        source_bytes: out.source_bytes,
        user_source: out.user_source,
        world_sha256: out.world_sha256,
        ui_json,
    })
}

/// 主降层入口（PLAN-054 node-ui-mixed-regions）：同一管线，区域声明放行到
/// 任意 def（组件/实例活区域；实例挂载身份＝实例路径）。世界源与 048/049
/// 字节同面（区域点击派发器、场景 getter、文字缓冲全部同一 ABI）；v1
/// schema/generator 常量与诊断逐字不变。
pub fn lower_project_node_mixed_regions(
    project: &Project,
) -> Result<LowerOutputNodeMixed, String> {
    let out = lower_node_ui_pipeline(project, true, true)?;
    let defs_json: BTreeMap<String, ArtifactDef> = out
        .def_plans
        .into_iter()
        .map(|(name, (prefix, is_root, texts, elements))| {
            (
                name,
                ArtifactDef {
                    prefix,
                    is_root,
                    texts,
                    elements,
                },
            )
        })
        .collect();
    let artifact = UiArtifact {
        schema: NODE_UI_MIXED_REGIONS_SCHEMA,
        generator: NODE_UI_MIXED_REGIONS_GENERATOR,
        world_sha256: out.world_sha256.clone(),
        root: out.root,
        instances: out.instances,
        defs: defs_json,
    };
    let ui_json = serde_json::to_string_pretty(&artifact).expect("ui.json");
    Ok(LowerOutputNodeMixed {
        source_bytes: out.source_bytes,
        user_source: out.user_source,
        world_sha256: out.world_sha256,
        ui_json,
    })
}
