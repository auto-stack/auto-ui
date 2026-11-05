//! PLAN-048 通用基础 UI 适配器（driver 侧）。
//!
//! 消费版本化 UI 产物（`ui.json`，schema `autographics.node-ui/1`）＋行为
//! 源（`world.at`，经 `SourceSnapshot` 的 `Api::NodeUi` 受检装配），把元素
//! 树呈现为**真实 stock Iced 控件**（`agp::host::iced_surface::SurfaceUi`），
//! 把真实控件消息送入有界队列并按稳定次序派发到固定 SDK `Session`。
//!
//! T-02 冻结合同（两侧对账的数字在此复检）：
//! - 消费入口：`main()` 一次 → `app_start()` 每 Run → `app_update(dt)` 每
//!   逻辑步（重绘/layout 永不代 Tick）→ 事件经生成派发器
//!   `{prefix}_ui_click_{elem}`（root 无 i）。
//! - 队列 ≤64 待处理；超界具名丢弃并计数，不静默派发到同名组件；陈旧
//!   （旧 Run 代次）事件派发前具名拒绝。
//! - 文字/属性从**一次已接受状态**的缓存读取（`view()` 无 VM 访问）；
//!   Session 故障关闭时保留最后可用呈现并记录具名相关域错误。
//! - 容量复检：实例 ≤8（root 计入）、每 def 元素 ≤32、文字槽 ≤4、容器
//!   子元素 ≤8、队列 64。

use crate::authoring::behavior::{Api, SourceSnapshot};
use crate::{ResultKind, Scalar, Session};
use serde::Deserialize;
use std::collections::VecDeque;
use std::path::Path;

/// 产物 schema 身份（与 pilot `NODE_UI_SCHEMA` 逐字一致；两图隔离故各自
/// 冻结，node_ui_storage 负面校验拼写漂移拒绝）。
pub const NODE_UI_SCHEMA: &str = "autographics.node-ui/1";
/// 事件队列容量（T-02 冻结）。
pub const QUEUE_CAP: usize = 64;
/// 每 def 元素上限（T-02 冻结，adapter 侧复检）。
pub const MAX_ELEMENTS: usize = 32;
/// 每 def 文字槽上限（T-02 冻结，adapter 侧复检）。
pub const MAX_TEXTS: usize = 4;
/// 容器子元素上限（T-02 冻结，adapter 侧复检）。
pub const MAX_CHILDREN: usize = 8;
/// 有状态实例上限（root 计入；T-02 冻结）。
pub const MAX_INSTANCES: usize = 8;

// ----------------------------------------------------------- artifact --

#[derive(Deserialize, Debug, Clone)]
pub struct UiArtifact {
    pub schema: String,
    pub generator: String,
    pub world_sha256: String,
    pub root: String,
    pub instances: Vec<ArtifactInstance>,
    pub defs: std::collections::BTreeMap<String, ArtifactDef>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ArtifactInstance {
    pub path: String,
    pub node: String,
    pub def_idx: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ArtifactDef {
    pub prefix: String,
    pub is_root: bool,
    pub texts: usize,
    pub elements: Vec<ArtifactElement>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum ArtifactElement {
    Container {
        name: String,
        container: String,
        #[serde(default)]
        props: Vec<ArtifactProp>,
        #[serde(default)]
        children: Vec<ArtifactElement>,
    },
    Text {
        name: String,
        slot: usize,
        #[serde(default)]
        visible: Option<ArtifactCond>,
    },
    Button {
        name: String,
        slot: usize,
        #[serde(default)]
        enabled: Option<ArtifactCond>,
        #[serde(default)]
        visible: Option<ArtifactCond>,
        click: ArtifactClick,
    },
    If {
        cond: ArtifactCond,
        then: Vec<ArtifactElement>,
    },
    Instance {
        name: String,
        node: String,
    },
    /// PLAN-049 混合档嵌入区域（`autographics.node-ui-mixed/1` 专属；048
    /// schema 消费入口在解析后具名拒绝本变体——旧档身份不变）。UI 占位
    /// rect（width/height）＋可选点击端口 ＋ 有界场景元素；场景 Field 绑定
    /// 经同一 accept_state getter 面读取（单 owner 一次接受）。
    Embed {
        name: String,
        dim: String,
        width: i64,
        height: i64,
        #[serde(default)]
        click: Option<ArtifactClick>,
        #[serde(default)]
        scene: Vec<ArtifactSceneElem>,
    },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactSceneElem {
    Quad {
        name: String,
        x: ArtifactPropValue,
        y: ArtifactPropValue,
        w: ArtifactPropValue,
        h: ArtifactPropValue,
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

#[derive(Deserialize, Debug, Clone)]
pub struct ArtifactProp {
    pub name: String,
    #[serde(flatten)]
    pub value: ArtifactPropValue,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactPropValue {
    LitNum {
        v: f64,
    },
    /// 序列键 `field`（与 ArtifactProp 的 `name` flatten 同层不冲突；
    /// PLAN-049 T-04，见 pilot 侧同注）。
    Field {
        #[serde(rename = "field")]
        name: String,
    },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactTextValue {
    LitStr { v: String },
    IntField { name: String },
    BoolField { name: String },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactCond {
    LitBool { v: bool },
    Field { name: String },
    BoolField { name: String },
}

#[derive(Deserialize, Debug, Clone)]
pub struct ArtifactClick {
    #[serde(rename = "fn")]
    pub fn_name: String,
    /// "index" | "none"。
    pub arg: String,
}

/// 解析产物＋容量复检（具名、执行前拒绝）。048 档身份：任何嵌入区域
/// 变体在解析后具名拒绝（混合 schema 走 `node_mixed::parse_mixed_artifact`）。
pub fn parse_artifact(json: &str) -> Result<UiArtifact, String> {
    let a = parse_artifact_common(json, NODE_UI_SCHEMA)?;
    for (name, def) in &a.defs {
        if has_embed(&def.elements) {
            return Err(format!(
                "def '{name}': embed regions are not part of the {NODE_UI_SCHEMA} identity (mixed profile required)"
            ));
        }
    }
    Ok(a)
}

fn has_embed(els: &[ArtifactElement]) -> bool {
    els.iter().any(|e| match e {
        ArtifactElement::Embed { .. } => true,
        ArtifactElement::Container { children, .. } => has_embed(children),
        ArtifactElement::If { then, .. } => has_embed(then),
        _ => false,
    })
}

/// 共同解析＋容量复检（schema 由调用方冻结；混合档复用同一数字面）。
pub fn parse_artifact_common(json: &str, schema: &str) -> Result<UiArtifact, String> {
    let a: UiArtifact = serde_json::from_str(json).map_err(|e| format!("ui.json parse: {e}"))?;
    if a.schema != schema {
        return Err(format!(
            "ui.json schema mismatch: expected {schema}, got {}",
            a.schema
        ));
    }
    if a.instances.len() + 1 > MAX_INSTANCES {
        return Err(format!(
            "artifact instances {} exceed the frozen {MAX_INSTANCES}-instance envelope",
            a.instances.len() + 1
        ));
    }
    for (name, def) in &a.defs {
        if def.texts > MAX_TEXTS {
            return Err(format!(
                "def '{name}': {} text slots exceed the frozen {MAX_TEXTS}",
                def.texts
            ));
        }
        let count = count_elements(&def.elements);
        if count > MAX_ELEMENTS {
            return Err(format!(
                "def '{name}': {count} elements exceed the frozen {MAX_ELEMENTS}"
            ));
        }
        check_children(name, &def.elements)?;
    }
    Ok(a)
}

fn count_elements(els: &[ArtifactElement]) -> usize {
    els.iter()
        .map(|e| match e {
            ArtifactElement::Container { children, .. } => 1 + count_elements(children),
            ArtifactElement::If { then, .. } => 1 + count_elements(then),
            _ => 1,
        })
        .sum()
}

fn check_children(def: &str, els: &[ArtifactElement]) -> Result<(), String> {
    for e in els {
        match e {
            ArtifactElement::Container { name, children, .. } => {
                if children.len() > MAX_CHILDREN {
                    return Err(format!(
                        "def '{def}': container '{name}' has {} children (frozen {MAX_CHILDREN})",
                        children.len()
                    ));
                }
                check_children(def, children)?;
            }
            ArtifactElement::If { then, .. } => check_children(def, then)?,
            _ => {}
        }
    }
    Ok(())
}

// --------------------------------------------------------- adapter app --

/// 一条待派发 UI 事件（稳定次序 = 入队次序）。
#[derive(Debug, Clone, PartialEq)]
pub struct QueuedEvent {
    /// 入队时的 Run 代次；派发时已陈旧的具名拒绝。
    pub generation: u64,
    /// 生成派发器名（唯一派发入口）。
    pub dispatch: String,
    /// def 层派发器的实例下标（root 派发器为 None）。
    pub arg: Option<usize>,
    /// 溯源（诊断/测试证据用）：instance#element。
    pub origin: String,
    /// PLAN-054 区域事件声明（None = 非区域事件，仅 Run 代次检查）。携带
    /// 入队时的区域身份与挂载代次；派发时经 host 注册表对账——区域已
    /// 卸载或代次不同（卸载后重挂）则具名拒绝，不得派发进新代次。
    pub region: Option<RegionClaim>,
}

/// PLAN-054 区域事件身份（`{实例路径}#{区域局部名}` ＋ 入队时挂载代次）。
#[derive(Debug, Clone, PartialEq)]
pub struct RegionClaim {
    pub key: String,
    pub epoch: u64,
}

/// 派发报告（可观察记账）。
#[derive(Debug, Default, Clone)]
pub struct DrainReport {
    pub dispatched: usize,
    pub stale_rejected: usize,
    pub faults: usize,
}

/// 实例行（root 在 0；def_idx 是 def 内实例下标 = 生成层 i）。
#[derive(Debug, Clone)]
pub struct InstanceRow {
    pub path: String,
    pub def: String,
    pub def_idx: usize,
    pub is_root: bool,
    pub elements: Vec<ArtifactElement>,
}

/// 已接受状态的呈现面（PLAN-049 Send 桥：纯数据副本——Session/
/// auto-lang-core 持 Rc，线程受限；组合内容只携带版本化呈现状态）。
/// 容量由产物冻结面限定（实例 ≤8、文字 ≤4/def、元素 ≤32），副本有界。
#[derive(Debug, Clone)]
pub struct UiPresentation {
    pub rows: Vec<InstanceRow>,
    /// 行 × 文字槽。
    pub texts: Vec<Vec<String>>,
    /// `{path}#{name}#en|$vis` / `{path}$if{n}`。
    pub conds: std::collections::BTreeMap<String, bool>,
    /// `{path}#{name}#{prop}` → f32（UI 与区域场景绑定同面）。
    pub layout: std::collections::BTreeMap<String, f32>,
}

impl UiPresentation {
    pub fn cond(&self, key: &str) -> bool {
        self.conds.get(key).copied().unwrap_or(true)
    }

    pub fn instance_index(&self, path: &str) -> Option<usize> {
        self.rows.iter().position(|r| r.path == path)
    }

    /// 借用视图（控件树/文字桥以引用工作——Element 借用数据源本身；
    /// 名避 SurfaceUi::view）。
    pub fn presentation_view(&self) -> UiPresentationView<'_> {
        UiPresentationView {
            rows: &self.rows,
            texts: &self.texts,
            conds: &self.conds,
            layout: &self.layout,
        }
    }
}

/// [`UiPresentation`] 的借用视图：控件树构建只读这些数据，Element 直接
/// 借用源（048 直连路径借 NodeUiApp 字段；混合镜像借已接受状态副本）。
pub struct UiPresentationView<'a> {
    pub rows: &'a [InstanceRow],
    pub texts: &'a [Vec<String>],
    pub conds: &'a std::collections::BTreeMap<String, bool>,
    pub layout: &'a std::collections::BTreeMap<String, f32>,
}

impl<'a> UiPresentationView<'a> {
    pub fn cond(&self, key: &str) -> bool {
        self.conds.get(key).copied().unwrap_or(true)
    }

    pub fn instance_index(&self, path: &str) -> Option<usize> {
        self.rows.iter().position(|r| r.path == path)
    }

    /// 同一控件树以 String 消息呈现（原型 `IcedContent` 统一消息面）。
    /// 编码 `click	{dispatch}	{arg}	{origin}`——派发器名与路径不含
    /// 制表符，往返无损。
    pub fn view_string(&self) -> SurfaceElement<'a, String> {
        let root = &self.rows[0];
        self.widget_tree(&root.elements, &root.path, 0)
            .map(|m| match m {
                UiMsg::Click {
                    dispatch,
                    arg,
                    origin,
                    ..
                } => format!(
                    "click	{}	{}	{}",
                    dispatch,
                    arg.map(|a| a.to_string()).unwrap_or_default(),
                    origin
                ),
            })
    }

    /// def 元素树 → 真实控件（多根包一列；实例化点展开为子实例路径树）。
    /// if 序 = 本 def 遍历序（Cell 计数与 collect_bindings 同序同键）。
    pub fn widget_tree(
        &self,
        elements: &[ArtifactElement],
        path: &str,
        instance: usize,
    ) -> SurfaceElement<'a, UiMsg> {
        let if_i = std::cell::Cell::new(0usize);
        let children: Vec<UiEl<'a>> = elements
            .iter()
            .map(|e| self.widget_if(e, path, instance, &if_i))
            .collect();
        let col = iced::widget::Column::<UiMsg, iced::Theme, iced_wgpu::Renderer>::with_children(
            children,
        );
        col.into()
    }

    #[allow(clippy::only_used_in_recursion)]
    fn widget_if(
        &self,
        el: &ArtifactElement,
        path: &str,
        instance: usize,
        if_i: &std::cell::Cell<usize>,
    ) -> UiEl<'a> {
        if let ArtifactElement::If { then, .. } = el {
            let n = if_i.get();
            if_i.set(n + 1);
            let visible = self.cond(&NodeUiApp::if_key(path, n));
            // 子树恒遍历（if 序对齐 collect_bindings）；不可见时不挂载。
            let kids: Vec<UiEl<'a>> = then
                .iter()
                .map(|c| self.widget_if(c, path, instance, if_i))
                .collect();
            if visible {
                iced::widget::Column::<UiMsg, iced::Theme, iced_wgpu::Renderer>::with_children(kids)
                    .into()
            } else {
                iced::widget::Column::<UiMsg, iced::Theme, iced_wgpu::Renderer>::with_children(
                    Vec::new(),
                )
                .into()
            }
        } else {
            self.widget(el, path, instance, if_i)
        }
    }

    fn widget(
        &self,
        el: &ArtifactElement,
        path: &str,
        instance: usize,
        if_i: &std::cell::Cell<usize>,
    ) -> UiEl<'a> {
        match el {
            ArtifactElement::Container {
                name,
                container,
                props,
                children,
            } => {
                let kids: Vec<UiEl<'a>> = children
                    .iter()
                    .map(|c| self.widget_if(c, path, instance, if_i))
                    .collect();
                // 布局值：字面量或已接受字段缓存（accept_state 已缓存）。
                let get_v = |pl: &ArtifactProp| -> Option<f32> {
                    match &pl.value {
                        ArtifactPropValue::LitNum { v } => Some(*v as f32),
                        ArtifactPropValue::Field { name: f } => self
                            .layout
                            .get(&format!("{path}#{name}#{}", pl.name))
                            .copied(),
                    }
                };
                let prop_of = |props: &[ArtifactProp], key: &str| -> Option<f32> {
                    props.iter().find(|pl| pl.name == key).and_then(get_v)
                };
                if container.as_str() == "row" {
                    let mut r =
                        iced::widget::Row::<UiMsg, iced::Theme, iced_wgpu::Renderer>::with_children(
                            kids,
                        );
                    if let Some(v) = prop_of(props, "spacing") {
                        r = r.spacing(v);
                    }
                    if let Some(v) = prop_of(props, "padding") {
                        r = r.padding(v);
                    }
                    if let Some(v) = prop_of(props, "width") {
                        r = r.width(iced::Length::Fixed(v));
                    }
                    if let Some(v) = prop_of(props, "height") {
                        r = r.height(iced::Length::Fixed(v));
                    }
                    r.into()
                } else {
                    let mut c =
                        iced::widget::Column::<UiMsg, iced::Theme, iced_wgpu::Renderer>::with_children(kids);
                    if let Some(v) = prop_of(props, "spacing") {
                        c = c.spacing(v);
                    }
                    if let Some(v) = prop_of(props, "padding") {
                        c = c.padding(v);
                    }
                    if let Some(v) = prop_of(props, "width") {
                        c = c.width(iced::Length::Fixed(v));
                    }
                    if let Some(v) = prop_of(props, "height") {
                        c = c.height(iced::Length::Fixed(v));
                    }
                    c.into()
                }
            }
            ArtifactElement::Text {
                name,
                slot,
                visible,
            } => {
                let content = self
                    .texts
                    .get(instance)
                    .and_then(|v| v.get(*slot))
                    .cloned()
                    .unwrap_or_default();
                let visible = visible
                    .as_ref()
                    .map(|_| self.cond(&format!("{path}#{name}#vis")));
                let t: UiEl<'_> = iced::widget::container(iced::widget::text(content))
                    .id(iced::widget::Id::from(format!("{path}#{name}")))
                    .into();
                if visible == Some(false) {
                    // 条件不可见 = 本帧不挂载（真实卸载语义）。
                    iced::widget::Column::<UiMsg, iced::Theme, iced_wgpu::Renderer>::with_children(
                        Vec::new(),
                    )
                    .into()
                } else {
                    t
                }
            }
            ArtifactElement::Button {
                name,
                slot,
                enabled,
                visible,
                click,
            } => {
                let label = self
                    .texts
                    .get(instance)
                    .and_then(|v| v.get(*slot))
                    .cloned()
                    .unwrap_or_default();
                let is_enabled = enabled
                    .as_ref()
                    .map(|_| self.cond(&format!("{path}#{name}#en")))
                    .unwrap_or(true);
                let is_visible = visible
                    .as_ref()
                    .map(|_| self.cond(&format!("{path}#{name}#vis")))
                    .unwrap_or(true);
                if !is_visible {
                    return iced::widget::Column::<UiMsg, iced::Theme, iced_wgpu::Renderer>::with_children(Vec::new()).into();
                }
                let msg = UiMsg::Click {
                    instance,
                    dispatch: click.fn_name.clone(),
                    arg: if click.arg == "index" {
                        Some(self.rows[instance].def_idx)
                    } else {
                        None
                    },
                    origin: format!("{path}#{name}"),
                };
                let btn: UiEl<'_> = iced::widget::button(iced::widget::text(label))
                    .on_press_maybe(is_enabled.then_some(msg))
                    .into();
                // 容器 id = 元素路径名（真实 bounds 探测面）。
                iced::widget::container(btn)
                    .id(iced::widget::Id::from(format!("{path}#{name}")))
                    .into()
            }
            ArtifactElement::If { then, .. } => {
                unreachable!("if handled by widget_if")
            }
            // PLAN-049：区域占位（真实布局参与面）。定尺寸容器＋稳定
            // widget Id（`{path}#{name}`）；宿主经 Operation 读真实 bounds
            // 驱动挂载几何与输入逆变换——不猜屏幕常量。场景内容不在本树
            // 渲染（挂载的 Game2d/Scene3d surface 由组合层采样）。
            ArtifactElement::Embed {
                name,
                dim,
                width,
                height,
                ..
            } => {
                let placeholder = iced::widget::container(
                    iced::widget::text(if dim == "3d" { "3D" } else { "2D" }).size(10),
                )
                .width(iced::Length::Fixed(*width as f32))
                .height(iced::Length::Fixed(*height as f32))
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb8(24, 26, 32).into()),
                    border: iced::Border {
                        color: iced::Color::from_rgb8(90, 96, 110),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                });
                iced::widget::container(placeholder)
                    .id(iced::widget::Id::from(format!("{path}#{name}")))
                    .into()
            }
            ArtifactElement::Instance { name, node } => {
                let child_path = format!("{path}/{name}");
                if let Some(idx) = self.instance_index(&child_path) {
                    let elements = self.rows[idx].elements.clone();
                    self.widget_tree(&elements, &child_path, idx)
                } else {
                    let _ = node;
                    iced::widget::Column::<UiMsg, iced::Theme, iced_wgpu::Renderer>::with_children(
                        Vec::new(),
                    )
                    .into()
                }
            }
        }
    }
}

/// 通用基础 UI 应用：固定 SDK Session ＋ 产物驱动的真实控件树。
pub struct NodeUiApp {
    pub artifact: UiArtifact,
    session: Session,
    /// 行为源受检快照（诊断/身份面）。
    pub snapshot: SourceSnapshot,
    queue: VecDeque<QueuedEvent>,
    /// 当前 Run 代次（start_run 递增）。
    pub generation: u64,
    rows: Vec<InstanceRow>,
    /// 已接受状态的文字缓存：行 × 文字槽。
    texts: Vec<Vec<String>>,
    /// 条件缓存：`{path}#{name}#en|$vis` / `{path}$if{n}`。
    conds: std::collections::BTreeMap<String, bool>,
    /// 布局字段缓存：`{path}#{name}#{prop}` → f32。
    layout: std::collections::BTreeMap<String, f32>,
    /// 具名相关域错误（Session 故障后非 None；呈现保留最后可用）。
    pub error: Option<String>,
    /// 记账：Session 回调次数（对账预算面）。
    pub callbacks: u64,
    /// 记账：队列满被具名丢弃的事件数。
    pub dropped_overflow: usize,
    /// PLAN-054 区域挂载代次注册表（host 注入；None = 无区域事件面）。
    /// 键 `{实例路径}#{区域局部名}` → 当前挂载代次；None = 区域当前未挂载。
    region_epochs: Option<std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, u64>>>>,
    /// 键盘焦点环（可见且可用的按钮，树序）。
    focus_ring: Vec<FocusEntry>,
    focus_index: usize,
    started: bool,
}

impl NodeUiApp {
    /// 从产物 JSON＋行为源构造（不启动 Run）。冻结消费合同：`main()` 在
    /// 任何其它调用前恰一次（全局数组字面量在此初始化）。
    pub fn new(artifact_json: &str, world_at: &str, name: &str) -> Result<Self, String> {
        let artifact = parse_artifact(artifact_json)?;
        Self::new_with_artifact(artifact, world_at, name, Api::NodeUi)
    }

    /// 从已解析产物构造（混合档入口：schema/区域档已由调用方解析冻结；
    /// `api` 冻结消费身份——048 直连 `Api::NodeUi`，混合档
    /// `Api::MixedUi2d3d`）。
    pub fn new_with_artifact(
        artifact: UiArtifact,
        world_at: &str,
        name: &str,
        api: Api,
    ) -> Result<Self, String> {
        // 存储身份链（V-S05）：行为源 sha 必须与产物绑定一致；损坏副本在
        // 消费入口具名拒绝（修复后下一次实际 Run 恢复）。
        let world_sha = agp::renderer::capture::sha256_hex(world_at.as_bytes());
        if world_sha != artifact.world_sha256 {
            return Err(format!(
                "behavior source identity: expected sha {}, got {} (damaged world copy rejected at load)",
                artifact.world_sha256, world_sha
            ));
        }
        let snapshot = SourceSnapshot::from_bytes_for(
            Path::new(&format!("{name}.at")),
            world_at.as_bytes().to_vec(),
            api,
        )
        .map_err(|e| format!("admission: {e}"))?;
        let session = Session::new_with_diagnostics(&snapshot.source, |stage, e| {
            snapshot.sdk_diagnostic(stage, e)
        })
        .map_err(|e| format!("compile: {e}"))?;
        let mut app = Self {
            artifact,
            session,
            snapshot,
            queue: VecDeque::new(),
            generation: 0,
            rows: Vec::new(),
            texts: Vec::new(),
            conds: Default::default(),
            layout: Default::default(),
            error: None,
            callbacks: 0,
            dropped_overflow: 0,
            region_epochs: None,
            focus_ring: Vec::new(),
            focus_index: 0,
            started: false,
        };
        app.build_rows()?;
        app.call_unit("main", &[])?;
        Ok(app)
    }

    fn build_rows(&mut self) -> Result<(), String> {
        let root_def = self.artifact.root.clone();
        let root = self
            .artifact
            .defs
            .get(&root_def)
            .ok_or("artifact root def missing")?;
        if !root.is_root {
            return Err(format!(
                "def '{root_def}' is the artifact root but not marked is_root"
            ));
        }
        let mut rows = vec![InstanceRow {
            path: root_def.clone(),
            def: root_def.clone(),
            def_idx: 0,
            is_root: true,
            elements: root.elements.clone(),
        }];
        for inst in &self.artifact.instances {
            let def = self.artifact.defs.get(&inst.node).ok_or_else(|| {
                format!(
                    "instance '{}' references missing def '{}'",
                    inst.path, inst.node
                )
            })?;
            rows.push(InstanceRow {
                path: inst.path.clone(),
                def: inst.node.clone(),
                def_idx: inst.def_idx,
                is_root: false,
                elements: def.elements.clone(),
            });
        }
        self.texts = rows
            .iter()
            .map(|r| vec![String::new(); self.artifact.defs[&r.def].texts])
            .collect();
        self.rows = rows;
        Ok(())
    }

    fn call_unit(&mut self, name: &str, args: &[Scalar]) -> Result<(), String> {
        if let Some(e) = &self.error {
            return Err(format!("session closed: {e}"));
        }
        self.callbacks += 1;
        match self.session.call(name, args, ResultKind::Unit) {
            Ok(_) => Ok(()),
            Err(e) => {
                let d = self.snapshot.diagnostic("node-ui/dispatch", name, &e);
                self.error = Some(d.clone());
                // 相关域停止：队列不再消费（具名；不静默继续）。
                self.queue.clear();
                Err(d)
            }
        }
    }

    /// 启动一个新 Run：代次递增、app_start、全量接受状态。
    pub fn start_run(&mut self) -> Result<(), String> {
        self.generation += 1;
        self.started = true;
        self.call_unit("app_start", &[])?;
        self.accept_state();
        Ok(())
    }

    /// 一个逻辑步（仅显式调用；重绘/layout 永不进入此处）。
    pub fn step(&mut self, dt: f32) -> Result<(), String> {
        if !self.started {
            return Err("step before start_run".into());
        }
        self.call_unit("app_update", &[Scalar::Float(dt)])?;
        self.accept_state();
        Ok(())
    }

    /// 入队一次控件事件（bounded；满则具名拒绝新事件——不无限追加）。
    pub fn enqueue_click(
        &mut self,
        instance: usize,
        dispatch: String,
        arg: Option<usize>,
        origin: String,
    ) -> bool {
        self.enqueue_click_claimed(dispatch, arg, origin, None)
    }

    /// 入队一次控件事件（可携带区域身份声明；队列语义与 [`Self::enqueue_click`]
    /// 完全同一有界面）。`claim` 的对账在 drain 派发前进行。
    pub fn enqueue_click_claimed(
        &mut self,
        dispatch: String,
        arg: Option<usize>,
        origin: String,
        claim: Option<RegionClaim>,
    ) -> bool {
        if self.queue.len() >= QUEUE_CAP {
            self.dropped_overflow += 1;
            return false;
        }
        let generation = self.generation;
        self.queue.push_back(QueuedEvent {
            generation,
            dispatch,
            arg,
            origin,
            region: claim,
        });
        true
    }

    /// 注入区域挂载代次注册表（PLAN-054 host 面；v1 路径不调用＝行为不变）。
    pub fn set_region_epochs(
        &mut self,
        epochs: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, u64>>>,
    ) {
        self.region_epochs = Some(epochs);
    }

    /// 按稳定次序派发队列；陈旧（旧 Run 代次；或区域已卸载/挂载代次不同）
    /// 具名拒绝。
    pub fn drain(&mut self) -> DrainReport {
        let mut report = DrainReport::default();
        let generation = self.generation;
        while let Some(ev) = self.queue.pop_front() {
            if ev.generation != generation {
                report.stale_rejected += 1;
                continue;
            }
            if let Some(claim) = &ev.region {
                let current = self
                    .region_epochs
                    .as_ref()
                    .and_then(|r| r.borrow().get(&claim.key).copied());
                if current != Some(claim.epoch) {
                    // 区域未挂载（注册表无此键）或已卸载重挂（代次不同）：
                    // 旧区域事件不得派发进新代次/新呈现。
                    report.stale_rejected += 1;
                    continue;
                }
            }
            let args: Vec<Scalar> = ev.arg.iter().map(|i| Scalar::Int(*i as i32)).collect();
            match self.call_unit(&ev.dispatch, &args) {
                Ok(()) => report.dispatched += 1,
                Err(_) => {
                    report.faults += 1;
                    break; // 相关域停止（Session 已关闭，呈现保留）
                }
            }
        }
        if report.dispatched > 0 {
            self.accept_state();
        }
        report
    }

    /// 接受状态：从 VM 读回全部文字/条件/布局到缓存（一次性、有界；
    /// 故障后保留最后可用，不读）。
    fn accept_state(&mut self) {
        if self.error.is_some() {
            return;
        }
        for row_i in 0..self.rows.len() {
            let row = self.rows[row_i].clone();
            let def = self.artifact.defs[&row.def].clone();
            for slot in 0..def.texts {
                if let Some(text) = self.read_text(&row, slot) {
                    self.texts[row_i][slot] = text;
                }
            }
            collect_bindings(
                &row.elements,
                &row.path,
                row.is_root,
                &mut self.session,
                &mut self.conds,
                &mut self.layout,
                &mut self.callbacks,
            );
        }
        self.rebuild_focus_ring();
    }

    /// 焦点环：树序收集可见且可用的按钮（键 = {path}#{name}）。
    fn rebuild_focus_ring(&mut self) {
        let mut ring = Vec::new();
        for row_i in 0..self.rows.len() {
            let row = self.rows[row_i].clone();
            let conds = &self.conds;
            fn rec(
                els: &[ArtifactElement],
                path: &str,
                focus_i: usize,
                conds: &std::collections::BTreeMap<String, bool>,
                ring: &mut Vec<FocusEntry>,
            ) {
                for e in els {
                    match e {
                        ArtifactElement::Container { children, .. } => {
                            rec(children, path, focus_i, conds, ring);
                        }
                        ArtifactElement::If { then, .. } => {
                            rec(then, path, focus_i, conds, ring);
                        }
                        ArtifactElement::Button {
                            name,
                            enabled,
                            visible,
                            click,
                            ..
                        } => {
                            let enabled = enabled
                                .as_ref()
                                .map(|_| {
                                    conds
                                        .get(&format!("{path}#{name}#en"))
                                        .copied()
                                        .unwrap_or(true)
                                })
                                .unwrap_or(true);
                            let visible = visible
                                .as_ref()
                                .map(|_| {
                                    conds
                                        .get(&format!("{path}#{name}#vis"))
                                        .copied()
                                        .unwrap_or(true)
                                })
                                .unwrap_or(true);
                            if enabled && visible {
                                ring.push(FocusEntry {
                                    instance: focus_i,
                                    key: format!("{path}#{name}"),
                                    dispatch: click.fn_name.clone(),
                                    arg: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            rec(&row.elements, &row.path, row_i, conds, &mut ring);
        }
        // def 层派发器实参 = 行的 def_idx（root 行 None）。
        for entry in &mut ring {
            let row = &self.rows[entry.instance];
            entry.arg = if row.is_root { None } else { Some(row.def_idx) };
        }
        if self.focus_index >= ring.len() {
            self.focus_index = 0;
        }
        self.focus_ring = ring;
    }

    /// 平台键盘路由：Tab 循环焦点，Enter 激活当前焦点按钮（与鼠标同
    /// 队列同派发器）。返回是否处理。
    pub fn key(&mut self, key: PlatformKey) -> bool {
        match key {
            PlatformKey::Tab => {
                if self.focus_ring.is_empty() {
                    return false;
                }
                self.focus_index = (self.focus_index + 1) % self.focus_ring.len();
                true
            }
            PlatformKey::Enter => {
                let Some(entry) = self.focus_ring.get(self.focus_index) else {
                    return false;
                };
                let entry = entry.clone();
                self.enqueue_click(
                    entry.instance,
                    entry.dispatch,
                    entry.arg,
                    format!("{}#keyboard", entry.key),
                )
            }
        }
    }

    /// 当前焦点元素键（测试/诊断/呈现面）。
    pub fn focused_key(&self) -> Option<&str> {
        self.focus_ring
            .get(self.focus_index)
            .map(|e| e.key.as_str())
    }

    /// 生成层文字 ABI：i = def 内实例下标（row.def_idx）。
    fn read_text(&mut self, row: &InstanceRow, slot: usize) -> Option<String> {
        let pfx = self.artifact.defs[&row.def].prefix.clone();
        let i = row.def_idx as i32;
        let len = self
            .session
            .int(
                &format!("{pfx}_ui_text_len"),
                &[Scalar::Int(i), Scalar::Int(slot as i32)],
            )
            .ok()?;
        let mut bytes = Vec::new();
        for j in 0..len {
            bytes.push(
                self.session
                    .int(
                        &format!("{pfx}_ui_text_at"),
                        &[Scalar::Int(i), Scalar::Int(slot as i32), Scalar::Int(j)],
                    )
                    .ok()? as u8,
            );
        }
        String::from_utf8(bytes).ok()
    }

    /// 实例行下标（root=0；按路径）。
    pub fn instance_index(&self, path: &str) -> Option<usize> {
        self.rows.iter().position(|r| r.path == path)
    }

    /// 实例行表（root 在 0；PLAN-054 区域目录构建面）。
    pub fn rows(&self) -> &[InstanceRow] {
        &self.rows
    }

    pub fn instance_path(&self, index: usize) -> &str {
        &self.rows[index].path
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 已接受状态的文字缓存（行 × 槽）。
    pub fn texts(&self, index: usize) -> &[String] {
        &self.texts[index]
    }

    pub fn cond(&self, key: &str) -> bool {
        self.conds.get(key).copied().unwrap_or(true)
    }

    pub fn layout_value(&self, key: &str) -> Option<f32> {
        self.layout.get(key).copied()
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn session_closed(&self) -> bool {
        self.session.closed()
    }

    pub fn dropped_overflow(&self) -> usize {
        self.dropped_overflow
    }

    /// PLAN-049 运行时路径桥：同一控件树以 String 消息呈现（已接受
    /// 状态副本；Session 不进入组合内容——Send 隔离见 UiPresentation）。
    pub fn view_string(&self) -> SurfaceElement<'_, String> {
        self.presentation_view().view_string()
    }

    /// 借用呈现视图（048 直连路径：Element 借 NodeUiApp 字段本身）。
    pub fn presentation_view(&self) -> UiPresentationView<'_> {
        UiPresentationView {
            rows: &self.rows,
            texts: &self.texts,
            conds: &self.conds,
            layout: &self.layout,
        }
    }

    /// 已接受状态的呈现副本（有界；组合内容唯一携带面）。
    pub fn presentation(&self) -> UiPresentation {
        UiPresentation {
            rows: self.rows.clone(),
            texts: self.texts.clone(),
            conds: self.conds.clone(),
            layout: self.layout.clone(),
        }
    }

    /// String 消息回注（解析失败静默丢弃＝非本适配器消息）。
    pub fn handle_string_message(&mut self, message: &str) {
        let Some(("click", rest)) = message.split_once('\t') else {
            return;
        };
        let Some((dispatch, rest)) = rest.split_once('\t') else {
            return;
        };
        let Some((arg, origin)) = rest.split_once('\t') else {
            return;
        };
        self.update(UiMsg::Click {
            instance: 0,
            dispatch: dispatch.to_string(),
            arg: arg.parse::<usize>().ok(),
            origin: origin.to_string(),
        });
    }

    /// 条件键（collect_bindings 与呈现构建共用；if 序 = 本实例 def 遍历序）。
    fn if_key(path: &str, if_index: usize) -> String {
        format!("{path}#if{if_index}")
    }

    /// 条件/布局的确定性键枚举（按 def 元素声明序；测试/诊断面）。
    pub fn binding_keys(path: &str, elements: &[ArtifactElement]) -> Vec<String> {
        let mut keys = Vec::new();
        fn rec(path: &str, els: &[ArtifactElement], if_i: &mut usize, keys: &mut Vec<String>) {
            for e in els {
                match e {
                    ArtifactElement::Container {
                        name,
                        props,
                        children,
                        ..
                    } => {
                        for p in props {
                            if matches!(p.value, ArtifactPropValue::Field { .. }) {
                                keys.push(format!("{path}#{name}#{}", p.name));
                            }
                        }
                        rec(path, children, if_i, keys);
                    }
                    ArtifactElement::Text { name, visible, .. } => {
                        if visible.is_some() {
                            keys.push(format!("{path}#{name}#vis"));
                        }
                    }
                    ArtifactElement::Button {
                        name,
                        enabled,
                        visible,
                        ..
                    } => {
                        if enabled.is_some() {
                            keys.push(format!("{path}#{name}#en"));
                        }
                        if visible.is_some() {
                            keys.push(format!("{path}#{name}#vis"));
                        }
                    }
                    ArtifactElement::If { then, .. } => {
                        keys.push(NodeUiApp::if_key(path, *if_i));
                        *if_i += 1;
                        rec(path, then, if_i, keys);
                    }
                    // PLAN-049：区域场景 Field 绑定键（与 collect_bindings
                    // 同键同序同判据——仅 Field 分量；cylinder tx/ty/tz）。
                    ArtifactElement::Embed {
                        name: rn, scene, ..
                    } => {
                        fn push_if_field(
                            path: &str,
                            rn: &str,
                            en: &str,
                            pn: &str,
                            v: &ArtifactPropValue,
                            keys: &mut Vec<String>,
                        ) {
                            if matches!(v, ArtifactPropValue::Field { .. }) {
                                keys.push(format!("{path}#{rn}#{en}#{pn}"));
                            }
                        }
                        for el in scene {
                            match el {
                                ArtifactSceneElem::Quad {
                                    name: en,
                                    x,
                                    y,
                                    w,
                                    h,
                                    ..
                                } => {
                                    push_if_field(path, rn, en, "x", x, keys);
                                    push_if_field(path, rn, en, "y", y, keys);
                                    push_if_field(path, rn, en, "w", w, keys);
                                    push_if_field(path, rn, en, "h", h, keys);
                                }
                                ArtifactSceneElem::Circle {
                                    name: en, x, y, r, ..
                                } => {
                                    push_if_field(path, rn, en, "x", x, keys);
                                    push_if_field(path, rn, en, "y", y, keys);
                                    push_if_field(path, rn, en, "r", r, keys);
                                }
                                ArtifactSceneElem::Cylinder {
                                    name: en,
                                    translation,
                                    ..
                                } => {
                                    for (pn, v) in ["tx", "ty", "tz"].iter().zip(translation.iter())
                                    {
                                        push_if_field(path, rn, en, pn, v, keys);
                                    }
                                }
                            }
                        }
                    }
                    ArtifactElement::Instance { .. } => {}
                }
            }
        }
        let mut if_i = 0usize;
        rec(path, elements, &mut if_i, &mut keys);
        keys
    }
}

/// 条件/布局读取（def 元素声明序；与 binding_keys 同序）。
fn lit0() -> ArtifactPropValue {
    ArtifactPropValue::LitNum { v: 0.0 }
}

#[allow(clippy::too_many_arguments)]
fn collect_bindings(
    els: &[ArtifactElement],
    path: &str,
    is_root: bool,
    session: &mut Session,
    conds: &mut std::collections::BTreeMap<String, bool>,
    layout: &mut std::collections::BTreeMap<String, f32>,
    callbacks: &mut u64,
) {
    let callbacks_cell = std::cell::Cell::new(*callbacks);
    let callbacks_cell = std::cell::Cell::new(*callbacks);
    let mut field_get = |name: &str| -> Option<i32> {
        let getter = if is_root {
            format!("get_r_{name}")
        } else {
            format!("get_{}_{}", path.replace('/', "_"), name)
        };
        callbacks_cell.set(callbacks_cell.get() + 1);
        session.int(&getter, &[]).ok()
    };
    let mut eval_cond =
        |field_get: &mut dyn FnMut(&str) -> Option<i32>, c: &ArtifactCond| -> Option<bool> {
            match c {
                ArtifactCond::LitBool { v } => Some(*v),
                ArtifactCond::BoolField { name } => field_get(name).map(|v| v != 0),
                ArtifactCond::Field { name } => field_get(name).map(|v| v != 0),
            }
        };
    fn rec(
        els: &[ArtifactElement],
        path: &str,
        if_i: &mut usize,
        conds: &mut std::collections::BTreeMap<String, bool>,
        layout: &mut std::collections::BTreeMap<String, f32>,
        field_get: &mut dyn FnMut(&str) -> Option<i32>,
        eval_cond: &mut dyn FnMut(
            &mut dyn FnMut(&str) -> Option<i32>,
            &ArtifactCond,
        ) -> Option<bool>,
    ) {
        for e in els {
            match e {
                ArtifactElement::Container {
                    name,
                    props,
                    children,
                    ..
                } => {
                    for p in props {
                        if let ArtifactPropValue::Field { name: f } = &p.value {
                            if let Some(v) = field_get(f) {
                                layout.insert(format!("{path}#{name}#{}", p.name), v as f32);
                            }
                        }
                    }
                    rec(children, path, if_i, conds, layout, field_get, eval_cond);
                }
                ArtifactElement::Text { name, visible, .. } => {
                    if let Some(c) = visible {
                        if let Some(v) = eval_cond(field_get, c) {
                            conds.insert(format!("{path}#{name}#vis"), v);
                        }
                    }
                }
                ArtifactElement::Button {
                    name,
                    enabled,
                    visible,
                    ..
                } => {
                    if let Some(c) = enabled {
                        if let Some(v) = eval_cond(field_get, c) {
                            conds.insert(format!("{path}#{name}#en"), v);
                        }
                    }
                    if let Some(c) = visible {
                        if let Some(v) = eval_cond(field_get, c) {
                            conds.insert(format!("{path}#{name}#vis"), v);
                        }
                    }
                }
                ArtifactElement::If { cond, then } => {
                    if let Some(v) = eval_cond(field_get, cond) {
                        conds.insert(NodeUiApp::if_key(path, *if_i), v);
                    }
                    *if_i += 1;
                    rec(then, path, if_i, conds, layout, field_get, eval_cond);
                }
                // PLAN-049：区域场景绑定走同一 Field getter 面（key
                // `{path}#{region}#{elem}#{prop}`；cylinder 三分量 #tx/#ty/#tz）。
                ArtifactElement::Embed { name, scene, .. } => {
                    let mut num = |e: &ArtifactPropValue,
                               key: String,
                               layout: &mut std::collections::BTreeMap<String, f32>| {
                        if let ArtifactPropValue::Field { name: f } = e {
                            if let Some(v) = field_get(f) {
                                layout.insert(key, v as f32);
                            }
                        }
                    };
                    for el in scene {
                        match el {
                            ArtifactSceneElem::Quad {
                                name: en,
                                x,
                                y,
                                w,
                                h,
                                ..
                            } => {
                                num(x, format!("{path}#{name}#{en}#x"), layout);
                                num(y, format!("{path}#{name}#{en}#y"), layout);
                                num(w, format!("{path}#{name}#{en}#w"), layout);
                                num(h, format!("{path}#{name}#{en}#h"), layout);
                            }
                            ArtifactSceneElem::Circle {
                                name: en, x, y, r, ..
                            } => {
                                num(x, format!("{path}#{name}#{en}#x"), layout);
                                num(y, format!("{path}#{name}#{en}#y"), layout);
                                num(r, format!("{path}#{name}#{en}#r"), layout);
                            }
                            ArtifactSceneElem::Cylinder {
                                name: en,
                                translation,
                                ..
                            } => {
                                num(
                                    translation.first().unwrap_or(&lit0()),
                                    format!("{path}#{name}#{en}#tx"),
                                    layout,
                                );
                                num(
                                    translation.get(1).unwrap_or(&lit0()),
                                    format!("{path}#{name}#{en}#ty"),
                                    layout,
                                );
                                num(
                                    translation.get(2).unwrap_or(&lit0()),
                                    format!("{path}#{name}#{en}#tz"),
                                    layout,
                                );
                            }
                        }
                    }
                }
                ArtifactElement::Instance { .. } => {}
            }
        }
    }
    let mut if_i = 0usize;
    rec(
        els,
        path,
        &mut if_i,
        conds,
        layout,
        &mut field_get,
        &mut eval_cond,
    );
    *callbacks = callbacks_cell.get();
}

/// 平台键（adapter 键盘路由面；宿主窗口层把真实 OS 键事件转发进来）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKey {
    Tab,
    Enter,
}

/// 平台键（adapter 键盘路由面；宿主窗口层把真实 OS 键事件转发进来）。

/// 焦点环条目（可激活按钮）。
#[derive(Debug, Clone)]
struct FocusEntry {
    instance: usize,
    key: String,
    dispatch: String,
    arg: Option<usize>,
}

/// 真实控件消息（入队载荷；drain 时派发）。
#[derive(Clone, Debug, PartialEq)]
pub enum UiMsg {
    Click {
        instance: usize,
        dispatch: String,
        arg: Option<usize>,
        origin: String,
    },
}

use agp::host::iced_surface::{SurfaceElement, SurfaceUi};

/// 适配器统一元素类型（真实 wgpu renderer；SurfaceElement 同型）。
type UiEl<'a> = iced_wgpu::core::Element<'a, UiMsg, iced_wgpu::core::Theme, iced_wgpu::Renderer>;

impl SurfaceUi for NodeUiApp {
    type Message = UiMsg;

    /// 当前已接受状态的控件树（真实 stock widgets；无 VM 访问）。
    fn view(&self) -> SurfaceElement<'_, UiMsg> {
        let root = &self.rows[0];
        self.presentation_view()
            .widget_tree(&root.elements, &root.path, 0)
    }

    /// 真实控件交互结果 → 有界队列（派发在 drain）。
    fn update(&mut self, message: UiMsg) {
        let UiMsg::Click {
            instance,
            dispatch,
            arg,
            origin,
        } = message;
        let _ = instance;
        // 队列满：具名丢弃计数（发送方视觉无效，不无限追加）。
        let _ = self.enqueue_click(instance, dispatch, arg, origin);
    }

    fn background(&self) -> iced_wgpu::core::Color {
        iced::Color::WHITE
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
