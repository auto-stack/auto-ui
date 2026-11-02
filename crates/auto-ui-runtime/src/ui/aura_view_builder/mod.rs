//! # AuraViewBuilder - Converts AuraNode templates into View<DynamicMessage>
//!
//! This module traverses an AuraNode tree and builds a `View<DynamicMsg>` for
//! rendering, reading state values from `VmBridge` for state bindings like
//! `${.count}`.
//!
//! ## Architecture
//!
//! ```text
//! AuraNode tree (from AuraWidget.view_tree)
//!    |
//!    v
//! AuraViewBuilder
//!  - Resolves Expr::Ident state references via VmBridge::read_state()
//!  - Resolves ${.field} interpolations in text content
//!  - Maps AuraNode::Element tags to View variants
//!  - Creates DynamicMessage for event handlers
//!    |
//!    v
//! View<DynamicMessage> (ready for rendering)
//! ```
//!
//! ## Plan 205 Phase 2
//!
//! Phase 2 implements core widget conversion:
//! - text, button, column, row (core layout)
//! - State binding resolution from VmBridge
//! - String interpolation for `${.field}` patterns
//! - Event handler → DynamicMessage mapping
//!
//! ## Plan 040 VM v1 豁免登记（autodown 契约扩展；044 收口后）
//!
//! `autodown` 只读臂与 `autodown_editor` 编辑臂：streaming 恒按 final
//! （豁免保留）；placeholder_block_id/placeholder_height（ghost 占位）自
//! PLAN-044 起**真消费**（只读臂绑定求值进 render——命中块前置定高灰盒；
//! 编辑臂 onfocus 事件进 on_focus 消息读出）；`scroll_sync` 自 PLAN-043 起
//! **真消费**（外包 View::Scrollable：scroll_top 绑定写入臂 + onscroll
//! 消息读出臂 + ondetailsclick 折叠回路，EDITOR-CONTRACT §11 在册）。

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;

use auto_val::{Op, Value};

use auto_lang_core::ast::Expr;
use auto_ui_compiler::aura::{
    aura_events_get_base, AuraEvent, AuraNode, AuraPropValue, AuraTextContent,
};

/// Loop variable bindings: variable name → current Value.
/// Passed through the conversion call chain to resolve `FieldAccess`
/// expressions like `note.title` where `note` is a loop variable.
type Bindings = HashMap<String, Value>;
use crate::ui::debug::{BuildProbe, ForIter};
use crate::ui::debug_id_map::DebugIdMap;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::style::{SizeValue, Style, StyleClass};
use crate::ui::view::{PopoverAnchor, PopoverPlacement, View, ViewBuilder};
use crate::ui::vm_bridge::VmBridge;
use auto_ui_compiler::ui_gen::vue::VueGenerator;

// Plan 409 §10 续 10: 当前 category-section 的 color,供 component-card 继承。
// VM builder 递归 convert 无父子参数传递,用 thread-local 模拟:category-section
// 分支在 convert children 前设此值,component-card 读它决定卡片边框/icon 颜色,
// 对齐 vue codegen(vue.rs:1459 category_color_classes)。
thread_local! {
    static CATEGORY_COLOR: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
    /// PLAN-712 T-17 第三层：outlet 页渲染期的路由身份后缀。Init demand 的
    /// 代际判定（`child_init_identity` → `init_identity_changed`）此前对
    /// outlet 页恒为裸 widget 名（outlet 不传 props，路由参数走
    /// `__route_params` 状态）——同页不同参（/book/1 → /book/2）同代际，
    /// Init 不重派 → 页面状态滞留首次装载（018 三卡同内容实机实证）。
    /// outlet 渲染期置 Some(当前路由路径)（guard 保证任一返回路径清零），
    /// 页 Init 身份并入路由 → 参数变化 = 新代际 = 重派（导航=新装载语义，
    /// 与书架回退重取数的既有行为一致）。
    static OUTLET_INIT_IDENTITY: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// [`OUTLET_INIT_IDENTITY`] 的作用域守卫：outlet 渲染是同步单线程的，
/// 构造即置值、Drop 即清零——`render_outlet_inner` 的多返回路径无需各自
/// 清理。
struct OutletInitIdentityGuard;
impl OutletInitIdentityGuard {
    fn set(route: &str) -> Self {
        OUTLET_INIT_IDENTITY.with(|c| {
            *c.borrow_mut() = Some(route.to_string());
        });
        Self
    }
}
impl Drop for OutletInitIdentityGuard {
    fn drop(&mut self) {
        OUTLET_INIT_IDENTITY.with(|c| *c.borrow_mut() = None);
    }
}

// ============================================================================
// Tracked conversion side-channels — Plan 307 Task 9
// ============================================================================
//
// The *tracked* path (`build_with_debug`) threads three mutable accumulators
// down through every converter: `path` (the AuraNode-structural descent path,
// each segment = child index in its parent's `children` slice), `id_map`
// (path → AuraNodeId), and `probe` (per-path AutoUI data).
//
// `path` is the AuraNode-structural descent path. The probe stores it as
// `Vec<u16>` (cast from `usize`). For plain col/row containers this matches
// the View-structural path used by `view_to_vtree_with_paths`; for ForLoop
// output the two schemes may diverge — see the OPEN ISSUE in the Task 9
// report (reconciled in Task 12).
//
// The **untracked** `build()` path never reaches these methods, so its
// behaviour is byte-for-byte identical to before Task 9.

// ============================================================================
// AuraViewBuilder
// ============================================================================

/// Builds a `View<DynamicMessage>` from an AuraNode template, reading state
/// from a `VmBridge`.
///
/// Each `AuraViewBuilder` is scoped to a single widget. The `widget_name` is
/// embedded in `DynamicMessage::Typed` variants so the event router can
/// dispatch messages back to the correct handler.
///
/// # Example
///
/// ```ignore
/// let bridge = VmBridge::new(&widget)?;
/// let builder = AuraViewBuilder::new(&bridge, "Counter");
/// let view = builder.build(&widget.view_tree);
/// ```
pub struct AuraViewBuilder<'a> {
    /// Reference to the VmBridge that holds widget state
    bridge: &'a VmBridge,

    /// Widget name, used in DynamicMessage routing
    widget_name: String,

    /// Optional widget registry for child widget rendering
    widget_registry: Option<&'a crate::ui::widget_registry::WidgetRegistry>,

    /// PLAN-066: 原生外部组件注册表（NativeWidgetRegistry）。缺省=进程级
    /// global（feature 门控内置注册）；测试经 with_native_registry 注入。
    native_registry: Option<&'a crate::ui::native_widget::NativeWidgetRegistry>,

    /// Plan 320: when rendering a child widget's view tree, this overrides the
    /// bridge's root state_obj_id so read_state reads from the child's state
    /// object instead of the root widget's. None = use root state.
    override_state_obj_id: Option<u64>,

    /// Plan 401/VM-routing: the root widget's route table (`routes {}` block).
    /// `outlet` reads `__current_route` state and renders the matching page
    /// widget from this table. None = no routes (outlet renders empty).
    routes: Option<&'a [auto_ui_compiler::aura::AuraRoute]>,

    /// EDGE-16 第五层(VM computed 求值):当前 widget 的 computed 属性表。
    /// resolve_expr_to_value / read_state_as_string_with 解析 `.foo` 时,先查
    /// 此表:命中则用 computed.expr 在当前 bindings 下求值(递归),未命中再
    /// 回退 state。None = 根 widget 在 build_with_debug_gated 时由调用方传入。
    computed: Option<&'a [auto_ui_compiler::aura::AuraComputed]>,
    /// Plan 409 §10 续 19: preview-card 的 UI 局部 state(show/tab)。
    preview_states:
        Option<&'a std::collections::HashMap<String, crate::ui::dynamic::PreviewCardUiState>>,
    /// Plan 482: nav-group 内置开合态（未绑定 `open` 的可折叠组）。
    /// 由 DynamicComponent 维护，`__nav_toggle` 内部消息翻转。
    nav_group_states: Option<&'a std::collections::HashMap<String, bool>>,
    /// Plan 476: 调用位传入的 slot 填充（widget-call children 中的
    /// `slot(name: X) { … }` / 裸子节点）。子 widget 视图里的 outlet 按
    /// 名字匹配后切回 `SlotFills::parent`（父作用域 builder）求值填充。
    /// None = 本层无填充（普通构建路径零变化）。
    slot_fills: Option<&'a SlotFills<'a>>,
    /// os-007（origin PLAN-577/P530-D1 根因修复）：子件渲染进行中集合。
    /// 未知 tag 经 widget_registry 折叠兜底（P435 P8-6：剥 `-`/`_`+小写）
    /// 可命中组件自身名——gallery 页面 `BreadcrumbPage` 的 demo 用
    /// `breadcrumb-page` tag，fold 后同名，无守卫时无限自递归栈溢出
    /// （新会话直达 breadcrumb 页 100% 复现）。命中环时渲染 Empty 占位；
    /// 集合按分支克隆传递，兄弟复用同一 widget 不受影响。
    active_child_widgets: RefCell<HashSet<String>>,
    /// PLAN-652: 装配期 mounted 登记 sink（与 DynamicComponent 共享）。
    /// `render_child_widget*` 每实例化一个 registry 命中的 child 即 insert 其名。
    mounted_sink: Option<&'a RefCell<HashSet<String>>>,
    /// PLAN-654: path 级挂载登记（InstancePath + per-type seq）。
    /// 与类型级 `mounted_sink` 并行写入；None 时仅类型级（652 兼容）。
    mount_path_sink: Option<crate::ui::dynamic::MountPathSinkRef<'a>>,
    /// PLAN-045: sidebar nav memo 武装旗标——`sidebar_provider (memo: true)`
    /// 在其子树转换期间置位，`sidebar_group` 臂据此入 nav memo 门（键随
    /// group 骨架指纹，无需 path 通道）。builder 每帧新建，天然帧界。
    nav_memo_armed: std::cell::Cell<bool>,
}

/// Plan 476: widget 调用位的 slot 填充集。
///
/// `parent` 是父作用域 builder 的直接引用——widget_name /
/// override_state_obj_id / computed / routes / registry 等"父作用域"全部由它
/// 载，fill 求值时切回它即可，无需拷贝字段或改造作用域栈。`bindings` 是调用
/// 位的循环变量绑定（fill 内 `.item` 等按调用位解析）。`fills` 的键 "" 为
/// 默认槽（bare `slot { … }` 的 children + 非 slot 裸子节点，Vue 语义）。
///
/// 生命周期自洽：本结构在 `render_child_widget*` 调用栈内构造，借给子
/// builder（`slot_fills` 字段），子构建在同一栈内完成，随后一同消亡。
struct SlotFills<'a> {
    parent: &'a AuraViewBuilder<'a>,
    fills: HashMap<String, Vec<&'a AuraNode>>,
    bindings: &'a Bindings,
}

/// Plan 476: 节点是否为 slot outlet 元素；是则返回其 (props, children)。
fn slot_outlet_parts(node: &AuraNode) -> Option<(&HashMap<String, AuraPropValue>, &[AuraNode])> {
    match node {
        AuraNode::Element {
            tag,
            props,
            children,
            ..
        } if tag == "slot" || tag == "Slot" => Some((props, children)),
        _ => None,
    }
}

/// Plan 476: 从 widget 调用位的 children 提取 slot 填充（对齐 vue.rs
/// `slot_child_to_html` 语义）：`slot(name: X) { … }` → fills[X] = 其
/// children；bare `slot { … }` → children 追加 fills[""]；非 slot 裸子节点
/// → 追加 fills[""]（Vue 默认槽语义——此前这些子节点在 VM 轨被整体丢弃）。
/// 无任何填充时返回空 map（调用方据此跳过 SlotFills 装配）。
fn extract_slot_fills(children: &[AuraNode]) -> HashMap<String, Vec<&AuraNode>> {
    let mut fills: HashMap<String, Vec<&AuraNode>> = HashMap::new();
    for child in children {
        if let Some((props, fill_children)) = slot_outlet_parts(child) {
            let name = auto_ui_compiler::aura::slot_element_name(props);
            fills.entry(name).or_default().extend(fill_children.iter());
        } else {
            fills.entry(String::new()).or_default().push(child);
        }
    }
    fills
}

/// PLAN-041 T8：autodown 只读 widget 的流式增量缓存注册表（按节点 path
/// 身份键）。帧间结构键 diff，未变块复用上帧 View；尾块重同步。
/// PLAN-043 T5：容量上限 32（对齐 DOC_EDITORS/413 §5.4 口径，DEBTS #041
/// 「StreamCache 注册表无容量上限」销号）——插入序 FIFO 淘汰（超容逐
/// 最旧 path 键，计划允许的最小实现；伴随 VecDeque 维持插入序）。
#[cfg(feature = "autodown")]
const AUTODOWN_STREAM_REGISTRY_CAP: usize = 32;

#[cfg(feature = "autodown")]
fn autodown_stream_registry() -> &'static std::sync::Mutex<(
    std::collections::HashMap<
        String,
        crate::ui::autodown_render::StreamCache<crate::ui::interpreter::DynamicMessage>,
    >,
    std::collections::VecDeque<String>,
)> {
    static REG: std::sync::OnceLock<
        std::sync::Mutex<(
            std::collections::HashMap<
                String,
                crate::ui::autodown_render::StreamCache<crate::ui::interpreter::DynamicMessage>,
            >,
            std::collections::VecDeque<String>,
        )>,
    > = std::sync::OnceLock::new();
    REG.get_or_init(|| {
        std::sync::Mutex::new((
            std::collections::HashMap::new(),
            std::collections::VecDeque::new(),
        ))
    })
}

/// PLAN-043 T5：取（或建）注册表槽位——新键插入前若已满（≥ 容量）按
/// 插入序逐出最旧键；已存在的键原地复用（不刷新插入序位，FIFO 口径）。
#[cfg(feature = "autodown")]
fn stream_registry_entry(
    reg: &mut (
        std::collections::HashMap<
            String,
            crate::ui::autodown_render::StreamCache<crate::ui::interpreter::DynamicMessage>,
        >,
        std::collections::VecDeque<String>,
    ),
    key: String,
) -> &mut crate::ui::autodown_render::StreamCache<crate::ui::interpreter::DynamicMessage> {
    let (map, order) = reg;
    if !map.contains_key(&key) {
        while map.len() >= AUTODOWN_STREAM_REGISTRY_CAP {
            match order.pop_front() {
                Some(oldest) => {
                    map.remove(&oldest);
                }
                None => break,
            }
        }
        order.push_back(key.clone());
    }
    map.entry(key).or_default()
}

/// PLAN-533 T7: 模态对话框族别（转换参数化）。
enum ModalDialogFamily {
    Alert,
    Dialog,
    DropdownMenu,
}

impl<'a> AuraViewBuilder<'a> {
    /// Create a new builder bound to a VmBridge instance.
    ///
    /// # Arguments
    ///
    /// * `bridge` - VmBridge holding the widget's state
    /// * `widget_name` - Name of the widget (for message routing)
    pub fn new(bridge: &'a VmBridge, widget_name: &str) -> Self {
        Self {
            bridge,
            widget_name: widget_name.to_string(),
            widget_registry: None,
            native_registry: Some(crate::ui::native_widget::global()),
            override_state_obj_id: None,
            routes: None,
            computed: None,
            preview_states: None,
            nav_group_states: None,
            nav_memo_armed: std::cell::Cell::new(false),
            slot_fills: None,
            active_child_widgets: RefCell::new(HashSet::new()),
            mounted_sink: None,
            mount_path_sink: None,
        }
    }

    /// Create a builder with widget registry for child widget support.
    pub fn with_registry(
        bridge: &'a VmBridge,
        widget_name: &str,
        registry: &'a crate::ui::widget_registry::WidgetRegistry,
    ) -> Self {
        Self {
            bridge,
            widget_name: widget_name.to_string(),
            widget_registry: Some(registry),
            native_registry: Some(crate::ui::native_widget::global()),
            override_state_obj_id: None,
            routes: None,
            computed: None,
            preview_states: None,
            nav_group_states: None,
            nav_memo_armed: std::cell::Cell::new(false),
            slot_fills: None,
            active_child_widgets: RefCell::new(HashSet::new()),
            mounted_sink: None,
            mount_path_sink: None,
        }
    }

    /// PLAN-652: 挂载登记 sink（DynamicComponent.mounted_types 共享 RefCell）。
    pub fn with_mounted_sink(mut self, sink: &'a RefCell<HashSet<String>>) -> Self {
        self.mounted_sink = Some(sink);
        self
    }

    /// PLAN-654: path 级挂载登记 sink（DynamicComponent.mounted_paths + seq）。
    pub fn with_mount_path_sink(mut self, sink: crate::ui::dynamic::MountPathSinkRef<'a>) -> Self {
        self.mount_path_sink = Some(sink);
        self
    }

    /// PLAN-066: 注入自有 NativeWidgetRegistry（测试隔离面；生产构造器缺省
    /// 绑定进程级 global）。
    pub fn with_native_registry(
        mut self,
        registry: &'a crate::ui::native_widget::NativeWidgetRegistry,
    ) -> Self {
        self.native_registry = Some(registry);
        self
    }

    /// Plan 401/VM-routing: attach the root widget's route table so `outlet`
    /// can resolve the current route to a page widget.
    pub fn with_routes(mut self, routes: &'a [auto_ui_compiler::aura::AuraRoute]) -> Self {
        self.routes = Some(routes);
        self
    }

    /// EDGE-16 第五层:传入当前 widget 的 computed 属性表,供 resolve 时
    /// 求值 computed 引用(如 `.status_glyph`)。
    pub fn with_computed(mut self, computed: &'a [auto_ui_compiler::aura::AuraComputed]) -> Self {
        self.computed = Some(computed);
        self
    }

    /// Plan 409 §10 续 19: 传入 preview-card 的 UI 局部 state(show/tab),
    /// 供 preview-card 渲染时决定是否展开代码 + 哪个 tab。
    pub fn with_preview_states(
        mut self,
        states: &'a std::collections::HashMap<String, crate::ui::dynamic::PreviewCardUiState>,
    ) -> Self {
        self.preview_states = Some(states);
        self
    }

    /// Plan 482: 传入 nav-group 内置开合态（未绑定 `open` 的可折叠组），
    /// `__nav_toggle` 内部消息在 update 循环翻转、view 重建时读取。
    pub fn with_nav_group_states(
        mut self,
        states: &'a std::collections::HashMap<String, bool>,
    ) -> Self {
        self.nav_group_states = Some(states);
        self
    }

    /// Build a `View<DynamicMessage>` from an AuraNode template.
    ///
    /// Recursively traverses the AuraNode tree, converting each node into the
    /// corresponding View variant. State references are resolved from the
    /// VmBridge at build time.
    pub fn build(&self, node: &AuraNode) -> View<DynamicMessage> {
        self.convert_node_with(node, &Bindings::new())
    }

}

// PLAN-014 T-05：按职责家族拆分的 impl 子模块（原单文件 impl 块的纯移动）。
// mod.rs 保留：类型（AuraViewBuilder/SlotFills/OutletInitIdentityGuard/
// ModalDialogFamily）、thread-locals 与 autodown 注册表、自由助手、
// build/build_with_debug 入口、文件尾自由函数族与全部测试；
// &VmBridge 仍由 builder 借用（owner=dynamic.rs），求值次序/guard/挂载身份零改。
mod slots;
mod eval;
mod tracked;
mod convert_element;
mod nav_sidebar;
mod child_widgets;
mod tags_container;
mod tags_panels;
mod tags_controls;
mod events;
mod props;
mod route_memo;

impl<'a> AuraViewBuilder<'a> {

    /// Build a `View<DynamicMessage>` with debug sideband data (Plan 274 / 307 Task 9).
    ///
    /// Returns `(View, DebugIdMap, BuildProbe)` where:
    /// - the `DebugIdMap` records which AuraNodeId produced each View node, keyed
    ///   by the AuraNode-structural path (`Vec<usize>`);
    /// - the `BuildProbe` records AutoUI-specific per-path data (state bindings,
    ///   for-context, events) captured while walking the node tree. Task 9 fills
    ///   text-interpolation state bindings only.
    ///
    /// The probe is **enabled** (records normally). This preserves the
    /// historical behaviour relied on by Task 9-11 tests. For F12-off / MCP
    /// zero-overhead capture bypass (Plan 307 Task 18), use
    /// [`build_with_debug_gated`] with `capture_probe = false`.
    pub fn build_with_debug(
        &self,
        node: &AuraNode,
    ) -> (View<DynamicMessage>, DebugIdMap, BuildProbe) {
        self.build_with_debug_gated(node, true)
    }

    /// Gated variant of [`build_with_debug`] (Plan 307 Task 18 perf gate).
    ///
    /// When `capture_probe` is false, the returned `BuildProbe` is constructed
    /// disabled via [`BuildProbe::new_disabled`], so every `record_*` call
    /// during the node walk is a no-op — giving near-zero overhead when the
    /// debug layer is inactive (F12 off) or for the MCP sync path (which never
    /// needs probe data). The `DebugIdMap` is still populated (it is cheap and
    /// required by MCP), but no probe work happens.
    pub fn build_with_debug_gated(
        &self,
        node: &AuraNode,
        capture_probe: bool,
    ) -> (View<DynamicMessage>, DebugIdMap, BuildProbe) {
        // PLAN-007 T-05：design 块 theme/mode 数据绑定每帧求值（C-06/07/08）。
        // 先于任何元素转换——整帧一致使用新槽位；评估/校验失败保留旧有效
        // 设计并给可定位诊断（C-11 事务发布）。
        self.eval_app_design();
        let mut id_map = DebugIdMap::default();
        let mut probe = if capture_probe {
            BuildProbe::new()
        } else {
            BuildProbe::new_disabled()
        };
        let mut path = Vec::new();
        let view = self.convert_node_tracked_ctx(
            node,
            &mut path,
            &mut id_map,
            &mut probe,
            &Bindings::new(),
        );
        (view, id_map, probe)
    }

    /// PLAN-007 T-05（C-06/07/08）：design 块每帧求值。
    ///
    /// - `theme: <expr>` → 主题名 → 注册表 lookup → [`set_theme_tree`]；
    ///   未注册名尝试内置主题；两者皆 miss → 诊断 + 保旧（C-11）。
    /// - `mode: <expr>` → "light"/"dark"/"system"（system 经 OS 探测）。
    ///   显式请求树未声明模式 → 拒绝（保旧 + 诊断，C-07）。
    /// - 求值失败（绑定缺失等）→ 该帧跳过发布（旧槽位自然保持）。
    fn eval_app_design(&self) {
        use auto_ui_compiler::design_tokens::theme_tree as tt;
        let entries = auto_ui_compiler::style::theme::app_design_entries();
        if entries.is_empty() {
            return;
        }
        let bindings = Bindings::new();
        let mut theme_name: Option<String> = None;
        let mut mode_str: Option<String> = None;
        for (key, expr) in &entries {
            let value = match self.resolve_expr_to_value(expr, &bindings) {
                Some(v) => v,
                None => {
                    eprintln!("[PLAN-007] design.{key} 求值失败——本帧保留旧设计");
                    continue;
                }
            };
            let text = match &value {
                auto_val::Value::Str(s) => Some(s.as_str().to_string()),
                auto_val::Value::Bool(b) => Some(b.to_string()),
                auto_val::Value::Int(i) => Some(i.to_string()),
                _ => None,
            };
            match key.as_str() {
                "theme" => theme_name = text,
                "mode" => mode_str = text,
                other => {
                    eprintln!("[PLAN-007] design 槽 `{other}` 在试点支持面外（theme/mode）")
                }
            }
        }
        // mode 先行（compose 需要知道目标模式）
        if let Some(mode) = &mode_str {
            let dark = match mode.as_str() {
                "dark" => Some(true),
                "light" => Some(false),
                "system" => crate::ui::system_theme::system_prefers_dark(),
                other => {
                    eprintln!("[PLAN-007] design.mode `{other}` 非 light|dark|system——拒绝（C-07）");
                    None
                }
            };
            if let Some(d) = dark {
                auto_ui_compiler::style::theme::set_dark_mode(d);
            }
        }
        if let Some(name) = &theme_name {
            let is_dark = auto_ui_compiler::style::theme::dark_mode();
            if let Some(tree) = auto_ui_compiler::style::theme::lookup_theme_tree(name) {
                if tt::mode_available(&tree, is_dark) {
                    auto_ui_compiler::style::theme::set_theme_tree(tree);
                } else {
                    let want = if is_dark { "dark" } else { "light" };
                    eprintln!(
                        "[PLAN-007] 主题 `{name}` 未声明 {want} 模式——拒绝换肤，保留旧有效设计（C-07）"
                    );
                }
            } else if auto_ui_compiler::style::theme::theme_name() != *name {
                if !auto_ui_compiler::style::theme::set_theme(name) {
                    eprintln!(
                        "[PLAN-007] design.theme `{name}` 未注册且非内置主题——保留旧有效主题（C-11）"
                    );
                }
            }
        }
    }

    // ========================================================================
    // Internal conversion
    // ========================================================================

}

// ============================================================================
// Free helper functions
// ============================================================================

/// PLAN-014 F-03: Plan 339「if 分支体 = 单表达式」合同的语句筛。注释
/// （Stmt::Comment）与空行（Stmt::EmptyLine）是解析器语句流的正式成员，
/// 不计入合同——此前 `stmts.len() == 1` 严格判定遇分支体内注释即整链
/// 报废：desktop.at 图标格样式链嵌套 else 内含注释 → 条件样式求值落
/// 空串 → hover 变体类 VM 轨整体丢失（布局件 hover 消费面归零）。
fn first_meaningful_stmt(body: &auto_lang_core::ast::Body) -> Option<&auto_lang_core::ast::Stmt> {
    body.stmts.iter().find(|s| {
        !matches!(
            s,
            auto_lang_core::ast::Stmt::Comment(_) | auto_lang_core::ast::Stmt::EmptyLine(_)
        )
    })
}

/// PLAN-748 T-07（jade 件一根修）：滚动转写的视觉类上移。CSS 语义里
/// `overflow-y-auto` 元素的 bg/border/radius/shadow 属**滚动容器本体**
/// （视口层，随容器在 stretch/bounded 语境拉伸满高），不属滚动内容
/// （内容止于内容高）。此前转写把整串 style clone 给内容 col——
/// `flex-col` 等显式显示类阻断 overflow-y 的 height=Fill 推断时，内容
/// col 止于内容高，bg/border 随之止步（jade 侧栏 bg 底边=树内容底
/// 550/786 的根因；视口 Scrollable 本身 Fill 拉伸正常）。
///
/// 拆分：`(视口视觉子集, 内容布局余集)`。视口子集为空 → 零包装层
/// （无视觉类的滚动 col 现行为不变）。hover/variant 类随内容（现状
/// 保持；纯视觉 hover 上移留后续按需）。
///
/// 视口子集附带头寸镜像：拷贝原显式 Width/Height 类，缺省补 Full——
/// iced 0.14 `Limits::(Shrink)` 置 compression 位，Fill 子件在压缩轴
/// 解析为内容高（Scrollable 在 Shrink 容器内塌 42px 实证）；视口容器
/// 须与 Scrollable 同 Length 才能传递拉伸。缺省 Full 与 from_style 对
/// overflow-y 的 height=Fill 推断、Scrollable 缺省 width=Fill 同口径。
fn split_scroll_visual_classes(s: &Style) -> (Option<Style>, Style) {
    let mut visual = Vec::new();
    let mut rest = Vec::new();
    let mut width_cls: Option<crate::ui::style::StyleClass> = None;
    let mut height_cls: Option<crate::ui::style::StyleClass> = None;
    for c in &s.classes {
        match c {
            crate::ui::style::StyleClass::Width(_) => {
                width_cls = Some(c.clone());
                rest.push(c.clone());
            }
            crate::ui::style::StyleClass::Height(_) => {
                height_cls = Some(c.clone());
                rest.push(c.clone());
            }
            _ if c.is_visual_paint() => visual.push(c.clone()),
            _ => rest.push(c.clone()),
        }
    }
    let viewport = (!visual.is_empty()).then(|| {
        let mut classes = visual;
        classes.push(width_cls.unwrap_or(crate::ui::style::StyleClass::Width(
            crate::ui::style::SizeValue::Full,
        )));
        classes.push(height_cls.unwrap_or(crate::ui::style::StyleClass::Height(
            crate::ui::style::SizeValue::Full,
        )));
        Style {
            classes,
            hover_classes: Vec::new(),
            variant_classes: Vec::new(),
        }
    });
    (
        viewport,
        Style {
            classes: rest,
            hover_classes: s.hover_classes.clone(),
            variant_classes: s.variant_classes.clone(),
        },
    )
}

/// PLAN-748 T-07：视口包装分支的 Scrollable 头寸补全——外部尺寸语义已由
/// 视口容器镜像承担，Scrollable 在容器内以 Fill×Fill 铺满（无显式
/// Width/Height 类时补 Full；Shrink 会在容器压缩语境塌内容高，J1 实测
/// 42px）。显式 w-*/h-* 保留原类（容器已镜像，同值无双关）。
fn ensure_full_scroll_dims(s: &Style) -> Style {
    let mut classes = s.classes.clone();
    if !classes
        .iter()
        .any(|c| matches!(c, crate::ui::style::StyleClass::Width(_)))
    {
        classes.push(crate::ui::style::StyleClass::Width(
            crate::ui::style::SizeValue::Full,
        ));
    }
    if !classes
        .iter()
        .any(|c| matches!(c, crate::ui::style::StyleClass::Height(_)))
    {
        classes.push(crate::ui::style::StyleClass::Height(
            crate::ui::style::SizeValue::Full,
        ));
    }
    Style {
        classes,
        hover_classes: s.hover_classes.clone(),
        variant_classes: s.variant_classes.clone(),
    }
}

/// PLAN-050 T7 (C5): 生产管线的 use.web component 名单（lib.rs 装载期从
/// root+child decls 的 ext_imports 收集注册；每次装载整体替换）。builder
/// 的图标组件臂按此判定。
static IMPORTED_COMPONENT_NAMES: std::sync::RwLock<Option<std::collections::HashSet<String>>> =
    std::sync::RwLock::new(None);

pub fn register_imported_components(names: Vec<String>) {
    if let Ok(mut guard) = IMPORTED_COMPONENT_NAMES.write() {
        *guard = Some(names.into_iter().collect());
    }
}

pub fn clear_imported_components() {
    if let Ok(mut guard) = IMPORTED_COMPONENT_NAMES.write() {
        *guard = None;
    }
}

fn imported_components_registry() -> Option<std::collections::HashSet<String>> {
    IMPORTED_COMPONENT_NAMES.read().ok().and_then(|g| g.clone())
}

/// PLAN-050 T7 (C5): PascalCase 图标名 → lucide kebab-case（MessageSquare →
/// message-square;Loader2/Trash2 → loader-2/trash-2,数字段前也加连字符）。
/// PLAN-621: icon 的有效盒 px（w/h 取大者）——描边基档判 ≥48px 细线用，
/// 与 renderer 的 Plan 518 G4② 规则同式。`Fixed(n)` = Tailwind 间距刻度
/// n×4px（P-10），`Pixels(p)` 直读，百分比/缺省 → 20px 默认盒。
fn effective_icon_box_px(s: &Style) -> f32 {
    const DEFAULT_ICON_PX: f32 = 20.0;
    let px_of = |v: &SizeValue| match v {
        SizeValue::Fixed(n) => *n as f32 * 4.0,
        SizeValue::Pixels(p) => *p,
        _ => DEFAULT_ICON_PX,
    };
    let mut px = DEFAULT_ICON_PX;
    for c in &s.classes {
        match c {
            StyleClass::Width(v) => px = px.max(px_of(v)),
            StyleClass::Height(v) => px = px.max(px_of(v)),
            _ => {}
        }
    }
    px
}

fn pascal_to_kebab_icon(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(ch.to_lowercase().next().unwrap_or(ch));
        } else if ch.is_numeric() && !out.ends_with('-') {
            out.push('-');
            out.push(ch);
        } else {
            out.push(ch);
        }
    }
    out
}

/// PLAN-748 T-02 续腿：imported tag 的图标/网页组件判别——以 **lucide
/// 全量名集成员资格**（kebab 化后查表）为准，含 "icon" 字样保持 glyph
/// 向后兼容（PLAN-625 T-09(b) 兼容面）。PLAN-625 T-09(b) 的
/// `tag.contains("icon")` 粗闸把 PascalCase lucide 名（Plus/Info/Trash2…）
/// 全数误判为 web-ecosystem 组件 → 占位卡，musk 图标子件整体消失
/// （p054_t1/t4 两红，P733-R2 族）；AppViewport 等真网页组件 ∉ 名集，
/// 占位卡路径（T-09(b) 意图）保持。
fn is_icon_component_tag(tag: &str) -> bool {
    tag.contains("icon") || crate::ui::iced::lucide_icon_known(&pascal_to_kebab_icon(tag))
}


/// PLAN-050 T9 (C7): 模板里的 `$t(KEY)` 标记 → i18n 查表替换（未命中回落
/// key 本身）。提取侧对 `t("k")`/`i18n.t("k")` 文本产出该标记（见
/// extract.rs call_name_t_key）。
fn resolve_i18n_template(template: &str) -> String {
    if !template.contains("$t(") {
        return template.to_string();
    }
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("$t(") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 3..];
        if let Some(end) = after.find(')') {
            let key = &after[..end];
            out.push_str(&crate::ui::i18n_lookup::lookup(key).unwrap_or_else(|| key.to_string()));
            rest = &after[end + 1..];
        } else {
            out.push_str("$t(");
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// PLAN-051 P2-②b: t("k", { param: expr }) 的第二实参提取——记录字面量
/// （Expr::Object）逐字段 bindings 感知求值为字符串对。非记录/无第二参
/// 返回空（模板 {k} 原样保留）。
fn t_call_params(
    build: &dyn Fn(&auto_lang_core::ast::Expr) -> Option<String>,
    call: &auto_lang_core::ast::Call,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(auto_lang_core::ast::Arg::Pos(auto_lang_core::ast::Expr::Object(pairs))) =
        call.args.args.get(1)
    {
        for pair in pairs {
            if let Some(v) = build(&pair.value) {
                out.push((pair.key.to_astr().to_string(), v));
            }
        }
    }
    out
}

/// PLAN-050 T9 (C7): 表达式位 t("k")/i18n.t("k") 识别（title 等 prop 位）。
fn call_expr_t_key(call: &auto_lang_core::ast::Call) -> Option<String> {
    let is_t_name = match call.name.as_ref() {
        Expr::Ident(name) => name.as_str() == "t",
        Expr::Dot(_, method) => method.as_str() == "t",
        _ => false,
    };
    if !is_t_name {
        return None;
    }
    match call.args.args.first()? {
        auto_lang_core::ast::Arg::Pos(Expr::Str(s)) => Some(s.to_string()),
        _ => None,
    }
}

/// Plan 049: evaluate a child model-var `initial` expression to a Value WITHOUT
/// a VM. Mirrors `eval_expr_to_value` (vm_bridge.rs) for the literal cases so
/// `var collapsed bool = false` → `Value::Bool(false)`. Complex initializers
/// (type literals / `.new(...)` calls, which need the VM heap) fall back to
/// `Value::Nil` — the same default the VM path uses for unresolvable exprs.
fn eval_initial_without_vm(expr: &Expr) -> Value {
    match expr {
        Expr::Int(i) => Value::Int(*i),
        Expr::I64(i) => Value::Int(*i as i32),
        Expr::Uint(u) => Value::Uint(*u),
        Expr::U64(u) => Value::Uint(*u as u32),
        Expr::Byte(b) => Value::Int(*b as i32),
        Expr::I8(i) => Value::Int(*i as i32),
        Expr::U8(u) => Value::Int(*u as i32),
        Expr::Float(f, _) => Value::Double(*f),
        Expr::Double(f, _) => Value::Double(*f),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Char(c) => Value::Int(*c as i32),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::CStr(s) => Value::Str(s.clone()),
        Expr::Unary(op, operand) => {
            let val = eval_initial_without_vm(operand);
            match op {
                Op::Sub => match val {
                    Value::Int(i) => Value::Int(-i),
                    Value::Double(f) => Value::Double(-f),
                    Value::Float(f) => Value::Float(-f),
                    _ => Value::Int(0),
                },
                Op::Not => match val {
                    Value::Bool(b) => Value::Bool(!b),
                    _ => Value::Bool(true),
                },
                _ => Value::Int(0),
            }
        }
        Expr::Array(elements) => {
            let values: Vec<Value> = elements.iter().map(eval_initial_without_vm).collect();
            Value::Array(auto_val::Array::from(values))
        }
        Expr::Object(pairs) => {
            let mut obj = auto_val::Obj::new();
            for pair in pairs {
                obj.set(pair.key.to_astr(), eval_initial_without_vm(&pair.value));
            }
            Value::Obj(Box::new(obj))
        }
        _ => Value::Nil,
    }
}

/// Plan 412 F1: 布局语义重派生结果。容器/div/col/row 元素的 style 里出现
/// 显式 Tailwind 布局类时,布局语义由类覆盖(与 CSS 中 `.grid` 在样式表中
/// 后于 `.flex` 声明、`display` 覆盖语义一致):`grid grid-cols-N` 优先于
/// `flex-col`,`flex-col` 优先于 `flex`。`*-reverse` 只影响渲染方向
/// (build_row/build_column 反序),不改变派生出的 View 类型。
#[derive(Debug, Clone, Copy, PartialEq)]
enum RederivedLayout {
    /// flex / flex-row / flex-row-reverse → 横向
    Row,
    /// flex-col / flex-col-reverse → 纵向
    Column,
    /// grid + grid-cols-N → 真实网格(gap 取 gap 类像素值,无则 0)
    Grid { cols: usize, gap: u16 },
}

/// Plan 412 F1: 从 style 类重派生布局语义。无显式布局类 → None(元素默认)。
/// 响应式前缀(md:grid-cols-2)已在 StyleClass::parse_single 剥离,自动参与。
fn rederive_layout(style: Option<&crate::ui::style::Style>) -> Option<RederivedLayout> {
    use crate::ui::style::StyleClass;
    let s = style?;
    let has = |pred: &dyn Fn(&StyleClass) -> bool| s.classes.iter().any(pred);
    let is_col = |c: &StyleClass| matches!(c, StyleClass::FlexCol | StyleClass::FlexColReverse);
    let is_row = |c: &StyleClass| {
        matches!(
            c,
            StyleClass::Flex | StyleClass::FlexRow | StyleClass::FlexRowReverse
        )
    };
    let is_grid = |c: &StyleClass| matches!(c, StyleClass::Grid);
    // 多个 grid-cols-N 并存时取最后一个:Tailwind mobile-first,后写的断点类
    // 覆盖基础类(md:grid-cols-2 剥离前缀后排在 grid-cols-1 之后)。
    let grid_cols = s.classes.iter().rev().find_map(|c| match c {
        StyleClass::GridCols(n) => Some(*n as usize),
        _ => None,
    });
    // 仲裁顺序与 CSS display 覆盖一致:grid > flex-col > flex。
    if has(&is_grid) {
        if let Some(cols) = grid_cols {
            let gap = s
                .classes
                .iter()
                .rev()
                .find_map(|c| match c {
                    StyleClass::Gap(size) => Some(size.to_pixels()),
                    _ => None,
                })
                .unwrap_or(0);
            return Some(RederivedLayout::Grid {
                cols: cols.max(1),
                gap,
            });
        }
    }
    if has(&is_col) {
        return Some(RederivedLayout::Column);
    }
    if has(&is_row) {
        return Some(RederivedLayout::Row);
    }
    None
}

/// Plan 370 (Issue 1): returns true for views that carry no visible content
/// and should be dropped from layouts to avoid spurious blank space. Covers
/// `View::Empty` and `View::Text { content: "", .. }` (the latter renders as
/// a one-line-tall `text("")` spacer in iced).
/// Plan 409 §10 续 5: 读 View 的 style,若 position:absolute 返回其
/// top/right/bottom/left offset(用于 Overlay 浮层定位)。
fn extract_absolute_position(v: &View<DynamicMessage>) -> Option<crate::ui::view::OverlayPosition> {
    // PLAN-814 R9-03：改为直接消费 compiler StyleClass IR 的纯扫描
    // （headless 构建同轨；与 style_iced_adapter 同一单源输入，仅取
    // overlay 相关四族变体——解析单源在 compiler，双消费不构成第二解析）。
    use crate::ui::style::StyleClass;
    let style = match v {
        View::Row { style, .. } | View::Column { style, .. } | View::Container { style, .. } => {
            style.as_ref()?
        }
        // PLAN-536 T6(题5 ②): button(会话卡 × 删除钮)/mouse_area(Plan 484
        // 注:其 style 参与 absolute/z 判定)同样可作悬浮载体。
        View::Button { style, .. } | View::MouseArea { style, .. } => style.as_ref()?,
        _ => return None,
    };
    let mut is_abs = false;
    let mut has_z = false;
    let (mut top_px, mut right_px, mut bottom_px, mut left_px) = (None, None, None, None::<f32>);
    let (mut top_pct, mut right_pct, mut bottom_pct, mut left_pct) =
        (None, None, None, None::<f32>);
    for c in &style.classes {
        match c {
            // R10-F01：position 族后继覆盖仲裁——与 IcedStyle::from_style
            // 同一字段按声明序覆盖的语义一致（后出现的 position 类覆盖
            // 前者：`absolute relative` 终态=relative，不 hoist）。
            StyleClass::Absolute => is_abs = true,
            StyleClass::Relative | StyleClass::Fixed | StyleClass::Sticky => is_abs = false,
            StyleClass::ZIndex(_) => has_z = true,
            StyleClass::Inset(px) => {
                top_px = Some(*px);
                bottom_px = Some(*px);
                right_px = Some(*px);
                left_px = Some(*px);
            }
            StyleClass::TopOffset(v) => top_px = Some(*v),
            StyleClass::RightOffset(v) => right_px = Some(*v),
            StyleClass::BottomOffset(v) => bottom_px = Some(*v),
            StyleClass::LeftOffset(v) => left_px = Some(*v),
            StyleClass::TopPercent(v) => top_pct = Some(*v),
            StyleClass::RightPercent(v) => right_pct = Some(*v),
            StyleClass::BottomPercent(v) => bottom_pct = Some(*v),
            StyleClass::LeftPercent(v) => left_pct = Some(*v),
            _ => {}
        }
    }
    // 只 hoist popup-like:absolute + z-N(如主题色板 z-50)。Home hero 的
    // absolute 装饰背景(渐变 div,无 z)留在流式,不破坏 hero 结构。
    if !is_abs || !has_z {
        return None;
    }
    let px_or_pct = |px: Option<f32>, pct: Option<f32>| match (px, pct) {
        (Some(v), _) => Some(crate::ui::view::OverlayLength::Px(v)),
        (None, Some(p)) => Some(crate::ui::view::OverlayLength::Percent(p)),
        (None, None) => None,
    };
    Some(crate::ui::view::OverlayPosition {
        top: px_or_pct(top_px, top_pct),
        right: px_or_pct(right_px, right_pct),
        bottom: px_or_pct(bottom_px, bottom_pct),
        left: px_or_pct(left_px, left_pct),
    })
}

fn is_visually_empty(v: &View<DynamicMessage>) -> bool {
    match v {
        View::Empty => true,
        View::Text { content, .. } => content.is_empty(),
        _ => false,
    }
}

/// PLAN-536 T10(D1 次生缺陷根修): absolute 浮层全部保留——按源序嵌套
/// Overlay 折叠(首个最外=栈序最低,后续逐层叠上,对齐 CSS 同 z 源序)。
/// 此前四处 hoist 臂(col/row × tracked/untracked)只取 `.next()` 首个,
/// 其余 absolute 子节点已被滤出流内却未进任何浮层,整体丢弃。
fn fold_floats(
    base: View<DynamicMessage>,
    floats: Vec<(View<DynamicMessage>, crate::ui::view::OverlayPosition)>,
) -> View<DynamicMessage> {
    let mut acc = base;
    for (content, position) in floats {
        acc = View::Overlay {
            base: Box::new(acc),
            content: Box::new(content),
            position,
        };
    }
    acc
}

/// Plan 499 M2: coords 逻辑幅面解析("WxH",如 "560x300")。
/// 解析失败返回 None(= raw px 模式),不致命中错误坐标语义。
fn parse_coords_extent(s: &str) -> Option<(f32, f32)> {
    let s = s.trim();
    let (w, h) = s.split_once(['x', 'X'])?;
    let w: f32 = w.trim().parse().ok()?;
    let h: f32 = h.trim().parse().ok()?;
    if w > 0.0 && h > 0.0 {
        Some((w, h))
    } else {
        None
    }
}

/// Plan 563: 场景数据契约解析(纯函数,双端映射规约的 Rust 侧)——
/// pts 项 "x1,y1|x2,y2|..." / meta 项 "color,width,eraser"(逗号分隔,
/// eraser 位 "0"/"1")。宽容解析:坏点/坏 meta 位跳过或取默认,不炸。
fn parse_canvas_scene(pts: &[Value], meta: &[Value]) -> crate::ui::view::CanvasScene {
    let val_str = |v: &Value| -> String {
        match v {
            Value::Str(s) => s.as_str().to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => String::new(),
        }
    };
    let mut strokes = Vec::with_capacity(pts.len());
    for (i, pts_item) in pts.iter().enumerate() {
        let pts_s = val_str(pts_item);
        if pts_s.is_empty() {
            continue;
        }
        let points: Vec<(f32, f32)> = pts_s
            .split('|')
            .filter_map(|pair| {
                let (x, y) = pair.split_once(',')?;
                let x: f32 = x.trim().parse().ok()?;
                let y: f32 = y.trim().parse().ok()?;
                Some((x, y))
            })
            .collect();
        if points.is_empty() {
            continue;
        }
        // meta 项 "color,width,eraser";缺表/缺项宽容取默认。
        let meta_s = meta.get(i).map(val_str).unwrap_or_default();
        let mut meta_parts = meta_s.split(',');
        let color = meta_parts
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "#111827".to_string());
        let width: f32 = meta_parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .filter(|w| *w > 0.0)
            .unwrap_or(3.0);
        let eraser = meta_parts
            .next()
            .map(|s| s.trim() == "1" || s.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        strokes.push(crate::ui::view::CanvasStroke {
            points,
            color,
            width,
            eraser,
        });
    }
    crate::ui::view::CanvasScene {
        strokes,
        ..Default::default()
    }
}

/// PLAN-661 T-05: `<前缀>_nodes` 项 `"id,x,y,shape,color[,r|w,h]"` →
/// CanvasNode。宽容解析：畸形项跳过（pts/meta 现行容错同款）；shape 非
/// circle|rect 按 circle 收；尺寸缺省 r=16 / w=h=40。
pub(crate) fn parse_canvas_nodes(items: &[Value]) -> Vec<crate::ui::view::CanvasNode> {
    let val_str = canvas_val_str;
    let mut nodes = Vec::with_capacity(items.len());
    for item in items {
        let s = val_str(item);
        if s.is_empty() {
            continue;
        }
        let parts: Vec<&str> = s.split(',').map(str::trim).collect();
        if parts.len() < 5 || parts[0].is_empty() {
            continue;
        }
        let (Ok(x), Ok(y)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>()) else {
            continue;
        };
        let shape = parts[3].to_ascii_lowercase();
        let shape = if shape == "rect" { "rect" } else { "circle" }.to_string();
        let color = if parts[4].is_empty() {
            "#3b82f6"
        } else {
            parts[4]
        }
        .to_string();
        let (r, w, h) = if shape == "rect" {
            let w = parts
                .get(5)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(40.0);
            let h = parts
                .get(6)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(w);
            (None, Some(w), Some(h))
        } else {
            (
                Some(
                    parts
                        .get(5)
                        .and_then(|s| s.parse::<f32>().ok())
                        .unwrap_or(16.0),
                ),
                None,
                None,
            )
        };
        nodes.push(crate::ui::view::CanvasNode {
            id: parts[0].to_string(),
            x,
            y,
            shape,
            color,
            r,
            w,
            h,
        });
    }
    nodes
}

/// PLAN-661 T-05: `<前缀>_edges` 项 `"x1,y1,x2,y2[,color[,width]]"` →
/// CanvasEdge（线段；绝对坐标——from/to 语义归上游，R-3）。
pub(crate) fn parse_canvas_edges(items: &[Value]) -> Vec<crate::ui::view::CanvasEdge> {
    let val_str = canvas_val_str;
    let mut edges = Vec::with_capacity(items.len());
    for item in items {
        let s = val_str(item);
        if s.is_empty() {
            continue;
        }
        let parts: Vec<&str> = s.split(',').map(str::trim).collect();
        if parts.len() < 4 {
            continue;
        }
        let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
            parts[0].parse::<f32>(),
            parts[1].parse::<f32>(),
            parts[2].parse::<f32>(),
            parts[3].parse::<f32>(),
        ) else {
            continue;
        };
        let color = parts
            .get(4)
            .filter(|s| !s.is_empty())
            .unwrap_or(&"#9ca3af")
            .to_string();
        let width = parts
            .get(5)
            .and_then(|s| s.parse::<f32>().ok())
            .filter(|w| *w > 0.0)
            .unwrap_or(2.0);
        edges.push(crate::ui::view::CanvasEdge {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
        });
    }
    edges
}

/// PLAN-661 T-05: `<前缀>_labels` 项 `"x,y,text[,color[,size]]"` →
/// CanvasLabel（text 段含逗号时仅在恰 3/5 段形态下取 color/size——
/// 宽容优先保文本完整）。
pub(crate) fn parse_canvas_labels(items: &[Value]) -> Vec<crate::ui::view::CanvasLabel> {
    let val_str = canvas_val_str;
    let mut labels = Vec::with_capacity(items.len());
    for item in items {
        let s = val_str(item);
        if s.is_empty() {
            continue;
        }
        let parts: Vec<&str> = s.split(',').map(str::trim).collect();
        if parts.len() < 3 {
            continue;
        }
        let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) else {
            continue;
        };
        // 严格形态 x,y,text[,color[,size]]：恰 3 段 = 纯文本；4 段带色；
        // 5 段带色带字号；更多段 = 文本含逗号，color/size 走默认。
        let (text, color, size) = match parts.len() {
            3 => (parts[2].to_string(), "#111827".to_string(), 14.0),
            4 => {
                let c = if parts[3].is_empty() {
                    "#111827".to_string()
                } else {
                    parts[3].to_string()
                };
                (parts[2].to_string(), c, 14.0)
            }
            5 => {
                let c = if parts[3].is_empty() {
                    "#111827".to_string()
                } else {
                    parts[3].to_string()
                };
                let sz = parts[4]
                    .parse::<f32>()
                    .ok()
                    .filter(|s| *s > 0.0)
                    .unwrap_or(14.0);
                (parts[2].to_string(), c, sz)
            }
            _ => (parts[2..].join(","), "#111827".to_string(), 14.0),
        };
        if text.is_empty() {
            continue;
        }
        labels.push(crate::ui::view::CanvasLabel {
            x,
            y,
            text,
            color,
            size,
        });
    }
    labels
}

fn canvas_val_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.as_str().to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// Extract a clean handler name from an event pattern.
///
/// Patterns:
/// - ".Inc"       -> "Inc"
/// - "Msg::Inc"   -> "Inc"
/// - "Inc"        -> "Inc"
fn extract_handler_name(pattern: &str) -> &str {
    let name = pattern.trim_start_matches('.');
    if let Some(pos) = name.rfind("::") {
        &name[pos + 2..]
    } else {
        name
    }
}

/// Parse an event-handler parameter that is NOT a loop-variable binding into
/// a literal Value. The parser stores onclick args as raw strings; a quoted
/// string like `"indigo"` (with the quotes preserved) becomes a Str, a number
/// becomes Int, anything else is treated as a bare string identifier.
fn parse_event_param_literal(param: &str) -> Value {
    let trimmed = param.trim();
    // Quoted string literal: "indigo" → Str("indigo")
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        return Value::Str(trimmed[1..trimmed.len() - 1].into());
    }
    // Integer literal: 42 → Int(42)
    if let Ok(i) = trimmed.parse::<i32>() {
        return Value::Int(i);
    }
    // Boolean literal
    match trimmed {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }
    // Fallback: treat as a bare string (identifier-like)
    Value::Str(trimmed.into())
}

/// Convert a Value to a display string suitable for UI rendering.
/// PLAN-051 C2: Component 调用位回调 prop 的 Expr → (handler 名, params)。
/// 覆盖两形态：`.SendMessage`（裸引用，无参）与 `.SendInput($event)`
/// （调用，`$event` 占位）。前导点 parser 规整为 `this.` 前缀或裸 `.` 前缀
/// Ident 两种都收。
fn callback_prop_route(expr: &auto_lang_core::ast::Expr) -> Option<(String, Vec<String>)> {
    use auto_lang_core::ast::Expr as E;
    let self_field = |obj: &E, field: &str| -> Option<String> {
        match obj {
            E::Ident(n) => {
                let s = n.as_str();
                (s == "self" || s == "." || s.is_empty()).then(|| field.to_string())
            }
            _ => None,
        }
    };
    match expr {
        E::Call(call) => {
            let handler = match call.name.as_ref() {
                E::Ident(n) => n.as_str().trim_start_matches('.').to_string(),
                E::Dot(obj, field) => self_field(obj, field.as_str())?,
                _ => return None,
            };
            let params = call
                .args
                .args
                .iter()
                .filter_map(|a| match a {
                    auto_lang_core::ast::Arg::Pos(E::Ident(n)) => Some(n.as_str().to_string()),
                    _ => None,
                })
                .collect();
            Some((handler, params))
        }
        E::Ident(n) => Some((n.as_str().trim_start_matches('.').to_string(), vec![])),
        E::Dot(obj, field) => Some((self_field(obj, field.as_str())?, vec![])),
        _ => None,
    }
}

fn value_to_display_string(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format!("{}", f),
        Value::Double(f) => format!("{}", f),
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => s.as_str().to_string(),
        Value::String(s) => s.as_str().to_string(),
        Value::Nil => String::new(),
        _ => value.to_string(),
    }
}

/// PLAN-493: mention 段计算——builder 期纯函数（不经 VM call 链，无堆引用
/// 生命周期问题）。`@\w+` 扫描（\w=[A-Za-z0-9_]，对齐 musk
/// mention_is_word_char），词（小写比较）∈ names → 段 `("@词原文",
/// "mention")`；裸 `@`（词空）与其余字符逐段 `("…", "text")`。无 display
/// 替换——编辑器内容即 value，VM/Vue 两轨着色原文一致。不变式：段文本
/// 顺序拼接 == value。names 为空 → 整体单段 text。
fn mention_segments(value: &str, names: &[String]) -> Vec<(String, String)> {
    let known: std::collections::HashSet<String> = names.iter().map(|n| n.to_lowercase()).collect();
    let mut segs: Vec<(String, String)> = Vec::new();
    let mut text = String::new();
    let bytes = value.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            let mut end = i + 1;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            // @ 是单字节 ASCII，词字符均为 ASCII，切片边界安全。
            let word = &value[i + 1..end];
            if !word.is_empty() && known.contains(&word.to_lowercase()) {
                if !text.is_empty() {
                    segs.push((std::mem::take(&mut text), "text".to_string()));
                }
                segs.push((value[i..end].to_string(), "mention".to_string()));
            } else {
                text.push_str(&value[i..end]);
            }
            i = end;
        } else {
            // 非_word 多字节字符（CJK/emoji 等）逐字入 text 段。
            let ch_len = utf8_char_len(bytes[i]);
            text.push_str(&value[i..i + ch_len]);
            i += ch_len;
        }
    }
    if !text.is_empty() {
        segs.push((text, "text".to_string()));
    }
    segs
}

/// PLAN-493: 首字节 → UTF-8 字符长度（mention_segments 逐字推进用）。
fn utf8_char_len(first: u8) -> usize {
    match first {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        _ => 4,
    }
}

/// PLAN-493: Vec<Value> → Vec<String>（Str/String 元素，其余跳过）——
/// mentions 名单同步深拷贝的元素提取。
fn strings_from_values(items: &[Value]) -> Vec<String> {
    items
        .iter()
        .filter_map(|v| match v {
            Value::Str(s) => Some(s.as_str().to_string()),
            Value::String(s) => Some(s.as_str().to_string()),
            _ => None,
        })
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_lang_core::ast::Type;
    use auto_ui_compiler::aura::{AuraEvent, AuraStateDef, AuraWidget};
    use auto_ui_dialect::UiCode;
    use auto_ui_dialect::UiStmt;
    use std::collections::HashMap;

    /// PLAN-014 F-03：分支体语句筛合同——注释/空行不计入「单表达式」合同，
    /// 首条有效语句命中；纯注释体 = None。
    #[test]
    fn first_meaningful_stmt_skips_comments_and_blank_lines() {
        use auto_lang_core::ast::{Body, Stmt};
        let mk = |stmts: Vec<Stmt>| Body {
            stmts,
            has_new_line: false,
            source_lines: vec![],
        };
        let with_comments = mk(vec![
            Stmt::Comment("注释行".into()),
            Stmt::Expr(auto_lang_core::ast::Expr::Int(7)),
            Stmt::EmptyLine(1),
        ]);
        assert!(matches!(
            first_meaningful_stmt(&with_comments),
            Some(Stmt::Expr(auto_lang_core::ast::Expr::Int(7)))
        ));
        let comments_only = mk(vec![Stmt::Comment("a".into()), Stmt::EmptyLine(2)]);
        assert!(first_meaningful_stmt(&comments_only).is_none());
    }

    /// PLAN-760 T-02：remote-frame 宿主数据 leaf——untracked/tracked 双路径
    /// 分派 + props 直通（url 字面量、class→Style）+ 别名归一。
    #[test]
    fn remote_frame_leaf_dispatch_both_paths() {
        let widget = make_test_widget("App", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let mk = |tag: &str| {
            AuraNode::element(tag)
                .with_prop("url", Expr::Str("ws://127.0.0.1:17800/?token=t".into()))
                .with_prop("class", Expr::Str("w-full h-full".into()))
        };
        let bindings = Bindings::new();

        let check = |v: &View<DynamicMessage>| match v {
            View::RemoteFrame { url, style } => {
                assert_eq!(url, "ws://127.0.0.1:17800/?token=t");
                assert!(style.is_some(), "class w-full h-full 应入 Style");
            }
            _ => panic!("期望 View::RemoteFrame（tag 分派未命中 remote-frame 臂）"),
        };

        // untracked 路径（convert_node_with → convert_element）。
        check(&builder.convert_node_with(&mk("remote-frame"), &bindings));
        // tracked 路径（convert_node_tracked_ctx → convert_element_tracked_ctx）。
        let mut path = Vec::new();
        let mut id_map = crate::ui::debug_id_map::DebugIdMap::default();
        let mut probe = crate::ui::debug::BuildProbe::default();
        check(&builder.convert_node_tracked_ctx(
            &mk("remote-frame"),
            &mut path,
            &mut id_map,
            &mut probe,
            &bindings,
        ));
    }

    /// Helper: create a minimal AuraWidget for testing.
    /// toolbar-icons r8（musk）：icon size 绑定解析回归——见下方测试。
    #[test]
    fn r8_marker_probe() {
        assert!(true);
    }

    /// toolbar-icons r8（musk）：icon 的 size 走绑定（widget 参数引用
    /// `icon { size: .size }`）时必须解析——原 extract_u16 非字面量恒
    /// None，size 恒落默认 20px（12/14/16 全部无效，musk 消息工具栏
    /// 实测定罪）。直测 with_icon_size 的 bindings 通道。
    #[test]
    fn r8_icon_size_binding_resolves_widget_param() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let registry = crate::ui::widget_registry::WidgetRegistry::new();
        let builder = AuraViewBuilder::with_registry(&bridge, "Test", &registry);
        let node = AuraNode::element("icon")
            .with_prop("name", Expr::Str("copy".into()))
            .with_prop("size", Expr::Ident(auto_lang_core::ast::Name::from("size")));
        let AuraNode::Element { props, .. } = &node else {
            panic!("期望 Element 节点")
        };
        let bindings: Bindings = HashMap::from([("size".to_string(), auto_val::Value::Int(12))]);
        let style = builder
            .with_icon_size(None, props, &bindings)
            .expect("绑定 size 应产出样式");
        assert!(
            style.classes.iter().any(|c| matches!(c, StyleClass::Width(SizeValue::Pixels(p)) if (*p - 12.0).abs() < f32::EPSILON)),
            "icon 应为 12px（绑定 size=.size 应解析）; got {:?}",
            style.classes
        );
    }

    fn make_test_widget(name: &str, state_vars: Vec<AuraStateDef>) -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: name.to_string(),
            state_vars,
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    // ========== PLAN-641 T-03：tabs 复合组件折叠 ==========

    /// 构造 tabs 树的触发器/内容件快捷方式。
    fn p641_trigger(value: &str, label: &str) -> AuraNode {
        AuraNode::element("tabs-trigger")
            .with_prop("value", auto_lang_core::ast::Expr::Str(value.into()))
            .with_child(AuraNode::text(label))
    }
    fn p641_content() -> AuraNode {
        AuraNode::element("tabs-content").with_child(AuraNode::element("text"))
    }
    fn p641_run_fold(builder: &AuraViewBuilder, tabs_root: AuraNode) -> View<DynamicMessage> {
        let (props, events, children) = match &tabs_root {
            AuraNode::Element {
                props,
                events,
                children,
                ..
            } => (props.clone(), events.clone(), children.clone()),
            _ => panic!("tabs root must be an element"),
        };
        builder.convert_tabs(&props, &events, &children, &Bindings::new())
    }

    #[test]
    fn plan717_calendar_is_visible_in_tracked_and_untracked_vm_trees() {
        fn cell_text(cell: &View<DynamicMessage>) -> &str {
            match cell {
                View::Container { child, .. } => match child.as_ref() {
                    View::Text { content, .. } => content.as_str(),
                    _ => "",
                },
                _ => "",
            }
        }

        fn assert_month_grid(view: &View<DynamicMessage>) {
            let View::Column { children, .. } = view else {
                panic!("calendar should render as a column, got {view:?}");
            };
            assert!(
                matches!(children.first(), Some(View::Text { content, .. }) if !content.is_empty())
            );
            let Some(View::Grid { cols, cells, .. }) = children.get(1) else {
                panic!("calendar should contain a native date grid: {children:?}");
            };
            assert_eq!(*cols, 7);
            assert_eq!(cells.len(), 49, "seven weekday labels plus six date rows");
            assert!(matches!(
                &cells[0],
                View::Container { child, .. }
                    if matches!(child.as_ref(), View::Text { content, .. } if content == "Su")
            ));
        }

        let widget = make_test_widget("App", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let calendar = AuraNode::element("calendar");
        let bindings = Bindings::new();

        assert_month_grid(&builder.convert_node_with(&calendar, &bindings));

        let mut path = Vec::new();
        let mut id_map = crate::ui::debug_id_map::DebugIdMap::default();
        let mut probe = crate::ui::debug::BuildProbe::default();
        assert_month_grid(&builder.convert_node_tracked_ctx(
            &calendar,
            &mut path,
            &mut id_map,
            &mut probe,
            &bindings,
        ));

        let selected_props = HashMap::from([(
            "selected".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("2024-02-29".into())),
        )]);
        let february = builder.convert_calendar(&selected_props, &bindings);
        let View::Column { children, .. } = february else {
            panic!("selected month should render as a column");
        };
        assert!(matches!(
            children.first(),
            Some(View::Text { content, .. }) if content == "February 2024"
        ));
        let Some(View::Grid { cells, .. }) = children.get(1) else {
            panic!("selected month should retain its date grid");
        };
        assert_eq!(
            cell_text(&cells[11]),
            "1",
            "February 2024 begins on Thursday"
        );
        assert_eq!(
            cell_text(&cells[39]),
            "29",
            "leap day remains in the final row"
        );
        assert_eq!(
            cells[7..]
                .iter()
                .filter(|cell| !cell_text(cell).is_empty())
                .count(),
            29,
            "leap February contains twenty-nine visible date cells"
        );
    }

    #[test]
    fn plan641_tabs_fold_wrappers_triggers_contents() {
        let widget = make_test_widget("App", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");

        // tabs > tabs-list(trigger×2) + tabs-content×2（别名拼写混用）。
        let tabs = AuraNode::element("tabs")
            .with_child(
                AuraNode::element("tabs-list")
                    .with_child(p641_trigger("a", "Alpha"))
                    .with_child(
                        AuraNode::element("tabstrigger")
                            .with_prop("value", auto_lang_core::ast::Expr::Str("b".into()))
                            .with_prop("text", auto_lang_core::ast::Expr::Str("Beta".into())),
                    ),
            )
            .with_child(p641_content())
            .with_child(p641_content());

        match p641_run_fold(&builder, tabs) {
            View::Tabs {
                labels,
                contents,
                selected,
                variant,
                ..
            } => {
                assert_eq!(labels, vec!["Alpha", "Beta"]);
                assert_eq!(contents.len(), 2);
                assert_eq!(selected, 0);
                assert_eq!(variant, crate::ui::view::TabsVariant::Default);
            }
            other => panic!("Expected View::Tabs, got {:?}", other),
        }
    }

    /// PLAN-661 T-03: slider 臂闭环——convert_slider 产 View::Slider（min/
    /// max/step/value 提取）+ SliderChangeHandler 标签旁路（快照 actions 的
    /// handler 名）+ f32 载荷派发（`.SetVol(v float)` 语义，025 基准）。
    #[test]
    fn plan661_t03_slider_arm_payload_and_label() {
        let widget = make_test_widget("App", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");

        let node = AuraNode::element("slider")
            .with_prop("min", auto_lang_core::ast::Expr::Int(0))
            .with_prop("max", auto_lang_core::ast::Expr::Int(100))
            .with_prop(
                "step",
                auto_lang_core::ast::Expr::Float(1.0, Default::default()),
            )
            .with_prop(
                "value",
                auto_lang_core::ast::Expr::Float(30.0, Default::default()),
            )
            .with_event("onchange", ".SetVol");
        let (props, events, _children) = match &node {
            AuraNode::Element {
                props,
                events,
                children,
                ..
            } => (props.clone(), events.clone(), children.clone()),
            _ => panic!("slider node must be an element"),
        };
        match builder.convert_slider(&props, &events, &Bindings::new()) {
            View::Slider {
                min,
                max,
                value,
                step,
                on_change: Some(handler),
                ..
            } => {
                assert_eq!((min, max, value, step), (0.0, 100.0, 30.0, Some(1.0)));
                assert_eq!(
                    handler.label(),
                    Some("SetVol"),
                    "label bypass feeds snapshot actions"
                );
                match handler.call(75.0) {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "SetVol");
                        assert_eq!(args, vec![Value::Float(75.0)], "f32 payload as first arg");
                    }
                    other => panic!("expected Typed message, got {other:?}"),
                }
            }
            View::Slider {
                on_change: None, ..
            } => panic!("onchange declared but handler missing"),
            other => panic!("Expected View::Slider, got {:?}", other),
        }

        // 无 onchange：无动作面（handler None——a2r/旧占位闭合形态退役）。
        let bare = AuraNode::element("slider").with_prop(
            "value",
            auto_lang_core::ast::Expr::Float(7.0, Default::default()),
        );
        let (props2, events2, _) = match &bare {
            AuraNode::Element {
                props,
                events,
                children,
                ..
            } => (props.clone(), events.clone(), children.clone()),
            _ => panic!("bare node must be an element"),
        };
        match builder.convert_slider(&props2, &events2, &Bindings::new()) {
            View::Slider {
                on_change: None, ..
            } => {}
            other => panic!("Expected bare slider with no handler, got {:?}", other),
        }
    }

    /// PLAN-661 T-05: 图元三表解析——规范项全字段、畸形项跳过（宽容）、
    /// 缺省值（r=16/w=40/color 三族各自缺省、edge width=2、label size=14）。
    #[test]
    fn plan661_t05_parse_canvas_primitive_tables() {
        use auto_val::Value;
        let nodes = vec![
            Value::str("n1,100,200,circle,#ff0000,12"),
            Value::str("n2,300,400,rect,#00ff00,80,50"),
            Value::str("bogus"),          // <5 段 → 跳过
            Value::str("n3,xx,400,rect"), // 坐标非数值 → 跳过
            Value::Int(0),                // 空串形态 → 跳过
        ];
        let parsed = parse_canvas_nodes(&nodes);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].id, "n1");
        assert_eq!(parsed[0].shape, "circle");
        assert_eq!(parsed[0].r, Some(12.0));
        assert_eq!(parsed[0].color, "#ff0000");
        assert_eq!(parsed[1].shape, "rect");
        assert_eq!((parsed[1].w, parsed[1].h), (Some(80.0), Some(50.0)));

        let edges = vec![
            Value::str("0,0,100,100"),
            Value::str("10,20,30,40,#123456,1.5"),
            Value::str("bad"),
        ];
        let edges = parse_canvas_edges(&edges);
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].color, "#9ca3af", "edge color 缺省");
        assert_eq!(edges[0].width, 2.0, "edge width 缺省");
        assert_eq!(edges[1].width, 1.5);

        let labels = vec![
            Value::str("50,60,hello"),
            Value::str("70,80,world,#ff8800,18"),
            Value::str("90,100,a,b,c,d"), // 文本含逗号 → 余段合并
            Value::str("bad"),
        ];
        let labels = parse_canvas_labels(&labels);
        assert_eq!(labels.len(), 3);
        assert_eq!(labels[0].text, "hello");
        assert_eq!(labels[0].size, 14.0, "label size 缺省");
        assert_eq!(labels[1].color, "#ff8800");
        assert_eq!(labels[1].size, 18.0);
        assert_eq!(labels[2].text, "a,b,c,d", "含逗号文本合并");
    }

    #[test]
    fn plan641_tabs_fold_variant_and_selected_resolution() {
        let widget = make_test_widget("App", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");

        let mk = |root_props: Vec<(&str, auto_lang_core::ast::Expr)>| {
            let mut root = AuraNode::element("tabs")
                .with_child(p641_trigger("a", "Alpha"))
                .with_child(p641_trigger("b", "Beta"))
                .with_child(p641_content())
                .with_child(p641_content());
            for (k, v) in root_props {
                root = root.with_prop(k, v);
            }
            root
        };

        // variant: enclosed 生效；非法值回退 Default（AC-01 兜底）。
        match p641_run_fold(
            &builder,
            mk(vec![(
                "variant",
                auto_lang_core::ast::Expr::Str("enclosed".into()),
            )]),
        ) {
            View::Tabs { variant, .. } => {
                assert_eq!(variant, crate::ui::view::TabsVariant::Enclosed)
            }
            other => panic!("Expected View::Tabs, got {:?}", other),
        }
        match p641_run_fold(
            &builder,
            mk(vec![(
                "variant",
                auto_lang_core::ast::Expr::Str("bogus".into()),
            )]),
        ) {
            View::Tabs { variant, .. } => {
                assert_eq!(variant, crate::ui::view::TabsVariant::Default)
            }
            other => panic!("Expected View::Tabs, got {:?}", other),
        }

        // selected：value 值串匹配 > active 索引 > defaultvalue > 0；越界钳制。
        match p641_run_fold(
            &builder,
            mk(vec![("value", auto_lang_core::ast::Expr::Str("b".into()))]),
        ) {
            View::Tabs { selected, .. } => assert_eq!(selected, 1),
            other => panic!("Expected View::Tabs, got {:?}", other),
        }
        match p641_run_fold(
            &builder,
            mk(vec![("active", auto_lang_core::ast::Expr::Int(1))]),
        ) {
            View::Tabs { selected, .. } => assert_eq!(selected, 1),
            other => panic!("Expected View::Tabs, got {:?}", other),
        }
        match p641_run_fold(
            &builder,
            mk(vec![(
                "defaultvalue",
                auto_lang_core::ast::Expr::Str("b".into()),
            )]),
        ) {
            View::Tabs { selected, .. } => assert_eq!(selected, 1),
            other => panic!("Expected View::Tabs, got {:?}", other),
        }
        match p641_run_fold(
            &builder,
            mk(vec![("active", auto_lang_core::ast::Expr::Int(9))]),
        ) {
            View::Tabs { selected, .. } => assert_eq!(selected, 1),
            other => panic!("Expected View::Tabs, got {:?}", other),
        }
    }

    /// 复审 P641-R1 回归：trigger 文本走 text-like 元素子件形态
    /// （`tabstrigger (value: "a") { text "Alpha" }`）时标签不得回退 "Tab N"。
    #[test]
    fn plan641_tabs_fold_trigger_text_child_element() {
        let widget = make_test_widget("App", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");

        let tabs = AuraNode::element("tabs")
            .with_child(
                AuraNode::element("tabstrigger")
                    .with_prop("value", auto_lang_core::ast::Expr::Str("a".into()))
                    .with_child(AuraNode::element("text").with_child(AuraNode::text("Alpha"))),
            )
            .with_child(
                AuraNode::element("tabstrigger")
                    .with_prop("value", auto_lang_core::ast::Expr::Str("b".into()))
                    .with_child(AuraNode::element("text").with_child(AuraNode::text("Beta"))),
            )
            .with_child(p641_content())
            .with_child(p641_content());

        match p641_run_fold(&builder, tabs) {
            View::Tabs { labels, .. } => assert_eq!(labels, vec!["Alpha", "Beta"]),
            other => panic!("Expected View::Tabs, got {:?}", other),
        }
    }

    #[test]
    fn plan641_tabs_fold_onselect_and_flat_tab_form() {
        let widget = make_test_widget("App", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");

        // 根 onselect：回调产出 Typed 消息，event_name 取 handler 名、
        // args=[选中索引]（Q2 受控契约）。
        let tabs = AuraNode::element("tabs")
            .with_event("onselect", ".SelectTab")
            .with_child(p641_trigger("a", "Alpha"))
            .with_child(p641_trigger("b", "Beta"))
            .with_child(p641_content())
            .with_child(p641_content());
        match p641_run_fold(&builder, tabs) {
            View::Tabs { on_select, .. } => {
                let cb = on_select.expect("onselect should wire on_select");
                match cb.call(1) {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "SelectTab");
                        assert_eq!(args, vec![Value::str("b")]);
                    }
                    other => panic!("Expected Typed message, got {:?}", other),
                }
            }
            other => panic!("Expected View::Tabs, got {:?}", other),
        }

        // 平铺 tab 形态（A2UI import 折叠产物）：label prop → labels，
        // 子件 → contents。
        let flat = AuraNode::element("tabs")
            .with_child(
                AuraNode::element("tab")
                    .with_prop("label", auto_lang_core::ast::Expr::Str("One".into()))
                    .with_child(AuraNode::element("text")),
            )
            .with_child(
                AuraNode::element("tab")
                    .with_prop("label", auto_lang_core::ast::Expr::Str("Two".into()))
                    .with_child(AuraNode::element("text")),
            );
        match p641_run_fold(&builder, flat) {
            View::Tabs {
                labels,
                contents,
                selected,
                ..
            } => {
                assert_eq!(labels, vec!["One", "Two"]);
                assert_eq!(contents.len(), 2);
                assert_eq!(selected, 0);
            }
            other => panic!("Expected View::Tabs, got {:?}", other),
        }
    }

    /// Plan 448 H2: block-bodied computed executes as a hidden VM fn.
    /// Previously `resolve_expr_to_value` had no Block arm — every block
    /// computed evaluated to None (rendered empty). The real chain
    /// (parse → extract → VmBridge synthesis → builder) must synthesize
    /// `__computed_<W>_<p>`, execute it with statement semantics (let
    /// scoping, tail-return), and re-evaluate on state change. Expression
    /// computeds stay on the inline resolver.
    #[test]
    fn plan448_h2_block_computed_executes_as_vm_fn() {
        use auto_lang_core::parser::Parser;
        let src = concat!(
            "widget App {\n",
            "    model {\n",
            "        var w int = 3\n",
            "        var label str = \"n\"\n",
            "    }\n",
            "    computed {\n",
            "        doubled => {\n",
            "            let base = .w * 2\n",
            "            base + 1\n",
            "        }\n",
            "        labeled => .label + \"!\"\n",
            "    }\n",
            "    view { col { text `d: ${.doubled}` } }\n",
            "}\n",
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");

        // Hidden fn synthesized and exported (invocable through the public
        // bridge API).
        let bridge = VmBridge::new(&widget).unwrap();
        assert_eq!(
            bridge.call_computed_fn("App", "doubled").unwrap(),
            Value::Int(7),
            "block computed compiles to an invocable hidden VM fn"
        );

        // Statement semantics + tail return; re-evaluates on state change.
        let builder = AuraViewBuilder::new(&bridge, "App").with_computed(&widget.computed);
        assert_eq!(
            builder.eval_computed("doubled", &Bindings::new()),
            Some(Value::Int(7)),
            "let base = 3*2; base + 1"
        );
        let mut bridge2 = VmBridge::new(&widget).unwrap();
        bridge2.write_state("w", auto_val::Value::Int(10)).unwrap();
        let builder2 = AuraViewBuilder::new(&bridge2, "App").with_computed(&widget.computed);
        assert_eq!(
            builder2.eval_computed("doubled", &Bindings::new()),
            Some(Value::Int(21)),
            "computed re-evaluates against updated state"
        );

        // Expression computeds keep the inline resolver path.
        assert_eq!(
            builder.eval_computed("labeled", &Bindings::new()),
            Some(Value::Str("n!".into())),
            "expression computed unchanged"
        );
    }

    /// PLAN-626 T-01: literal-text `${.store.field}` multi-segment
    /// interpolation rides the same resolver conditions use — `.store.X`
    /// flattens to a root-state read, so the auto-edit statusbar shows real
    /// values instead of the raw template. An unresolvable path keeps the
    /// raw template text (leading dots intact, no stripped placeholder).
    #[test]
    fn plan626_literal_interpolation_multi_segment_dot_path() {
        use auto_lang_core::parser::Parser;
        let src = concat!(
            "widget App {\n",
            "    model {\n",
            "        var line int = 3\n",
            "        var col int = 9\n",
            "    }\n",
            "    view { col { text \"${.store.line}:${.store.col}\" {} } }\n",
            "}\n",
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");
        // PLAN-668 A-06：PLAN-633 起「无任何 store 工程时字面 `store` 别名
        // 不展平」（view_store_alias_real_name 空快照语义）——裸装载单测
        // 须镜像合成期的 set_store_context 落别名快照（真身语料 041
        // status_bar.at 同式经全管线合成）。nextest 每测独立进程，无跨测
        // 快照污染。
        crate::ui::handler_codegen::set_store_context(
            std::collections::HashMap::from([(
                "EditorStore".to_string(),
                vec!["line".to_string(), "col".to_string()],
            )]),
            std::collections::HashMap::from([("store".to_string(), "EditorStore".to_string())]),
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let (view, _id_map, _probe) = builder.build_with_debug(&widget.view_tree);
        assert!(
            view_contains_text(&view, "3:9"),
            "multi-segment interpolation must resolve from state; got {:?}",
            view
        );

        // Unresolvable dotted path: raw template preserved with dots.
        let src = concat!(
            "widget App {\n",
            "    model { var line int = 3 }\n",
            "    view { col { text \"${.store.nope}\" {} } }\n",
            "}\n",
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let (view, _id_map, _probe) = builder.build_with_debug(&widget.view_tree);
        assert!(
            view_contains_text(&view, "${.store.nope}"),
            "unresolvable dotted path must keep the raw template (dots intact); got {:?}",
            view
        );
    }

    /// PLAN-642 T-15: store 真名限定的 `.len() > 0` 视图条件——`.len()` 后缀
    /// 快路径此前把 "Store.field" 整段当单字段名 read_state 必 miss → 恒
    /// false（015 内嵌 "No notes yet" 实锤）。快路径经 store_source_field
    /// 展平为根态裸字段后按长度求值；别名快照随 bridge 存档（多组件工程
    /// 下线程级快照被后续合成覆盖）。
    #[test]
    fn plan642_store_qualified_len_condition_resolves() {
        use auto_lang_core::parser::Parser;
        let src = concat!(
            "widget NotesStore {\n",
            "    model {\n",
            "        var notes List<str> = [\"a\", \"b\"]\n",
            "    }\n",
            "}\n",
            "widget App {\n",
            "    view {\n",
            "        col {\n",
            "            if .NotesStore.notes.len() > 0 {\n",
            "                text \"HAS\"\n",
            "            } else {\n",
            "                text \"EMPTY\"\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let mut decls = ast.domain_stmts().filter_map(|s| match s {
            UiStmt::WidgetDecl(d) => Some(d),
            _ => None,
        });
        let store_decl = decls.next().expect("store decl");
        let app_decl = decls.next().expect("app decl");
        // 合并轨等价：root + store 子件同合成（store 无 view → 别名表注册）。
        let bridge = VmBridge::new_from_decls(
            app_decl,
            std::slice::from_ref(store_decl),
            Vec::new(),
            &std::collections::HashMap::new(),
            false,
        )
        .unwrap();
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(app_decl).expect("extract");
        let builder = AuraViewBuilder::new(&bridge, "App");
        let (view, _id_map, _probe) = builder.build_with_debug(&widget.view_tree);
        assert!(
            view_contains_text(&view, "HAS"),
            "store-qualified len condition must resolve via root bare field; got {:?}",
            view
        );
        assert!(
            !view_contains_text(&view, "EMPTY"),
            "else branch must not render when store list is seeded"
        );
    }

    /// PLAN-626 T-02: menubar dropdown synthesis — item buttons carry
    /// explicit left alignment (justify-start/text-left beat the Plan-414
    /// Center default), `sep` renders the horizontal full-width hairline
    /// (not the toolbar vertical box), and the panel column keeps an
    /// explicit width class (w-44) under the PLAN-526 T27-style guard
    /// against block-level Fill fallback.
    #[test]
    fn plan626_menubar_panel_left_align_horizontal_sep_fixed_width() {
        use crate::ui::action_config::{
            extract_actions_from_source, set_dsl_action_config_from_block, set_menubar_open,
        };
        let src = concat!(
            "widget App {\n",
            "    msg { ActNew, ActOpen }\n",
            "    actions {\n",
            "        action (id: \"file.new\",  handler: .ActNew,  title: \"新建\")\n",
            "        action (id: \"file.open\", handler: .ActOpen, title: \"打开…\")\n",
            "        menubar {\n",
            "            menu (id: \"file\", title: \"文件\") {\n",
            "                item (action: \"file.new\")\n",
            "                sep\n",
            "                item (action: \"file.open\")\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "    view { col { menubar {} } }\n",
            "    on {\n",
            "        .ActNew -> { }\n",
            "        .ActOpen -> { }\n",
            "    }\n",
            "}\n",
        );
        let block = extract_actions_from_source(src).expect("actions block");
        set_dsl_action_config_from_block(&block, None);
        set_menubar_open(Some("file".to_string()));

        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let (view, _id_map, _probe) = builder.build_with_debug(&widget.view_tree);

        fn find_popover(v: &View<DynamicMessage>) -> Option<&View<DynamicMessage>> {
            match v {
                View::Popover { content, .. } => Some(content),
                View::Column { children, .. } | View::Row { children, .. } => {
                    children.iter().find_map(find_popover)
                }
                View::Button {
                    content: Some(c), ..
                } => find_popover(c),
                View::Container { child, .. } => find_popover(child),
                _ => None,
            }
        }
        let content = find_popover(&view).expect("menubar popover synthesized");
        let (items, panel_style) = match content {
            View::Column {
                children, style, ..
            } => (children, style),
            other => panic!("panel must be a Column, got {other:?}"),
        };
        assert_eq!(items.len(), 3, "two action items + one sep");
        // Panel width class survives (w-44 → Fixed(44) units) — the guard
        // must keep an explicit width so the column never falls back to Fill.
        let panel_style = panel_style.as_ref().expect("panel style present");
        assert!(
            panel_style
                .classes
                .iter()
                .any(|c| matches!(c, StyleClass::Width(crate::ui::style::SizeValue::Fixed(44)))),
            "panel must keep explicit w-44 width, got {:?}",
            panel_style.classes
        );
        // Item button: explicit left alignment classes on the button style.
        match &items[0] {
            View::Button { style, content, .. } => {
                let st = style.as_ref().expect("item button style");
                assert!(
                    st.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::JustifyStart)),
                    "item button must carry justify-start, got {:?}",
                    st.classes
                );
                assert!(content.is_some(), "item button must use content row");
            }
            other => panic!("item must be a Button, got {other:?}"),
        }
        // Sep: horizontal variant renders a bare Column hairline — the
        // toolbar/vertical variant is a centered Container box.
        match &items[1] {
            View::Column { style, .. } => {
                let st = style.as_ref().expect("sep style");
                assert!(
                    st.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Height(_))),
                    "horizontal sep must carry its hairline height, got {:?}",
                    st.classes
                );
            }
            other => panic!("menubar sep must be horizontal Column, got {other:?}"),
        }
        set_menubar_open(None);
    }

    /// PLAN-630 T-01: declarative menubar component family — items carry
    /// icon/shortcut/checked (expression-driven), separators render
    /// horizontal, and the checked expression flips with state.
    #[test]
    fn plan630_declarative_menubar_component() {
        let src = concat!(
            "widget App {\n",
            "    model { var console_open bool = false }\n",
            "    view {\n",
            "        col {\n",
            "            menubar {\n",
            "                menubar-menu (value: \"view\") {\n",
            "                    menubar-trigger \"视图\"\n",
            "                    menubar-content {\n",
            "                        menubar-checkbox-item (title: \"切换 Console\", checked: .console_open) { onclick: .ActConsole }\n",
            "                        menubar-separator\n",
            "                        menubar-item (title: \"全选\", shortcut: \"Ctrl+A\") { onclick: .ActSelectAll }\n",
            "                    }\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "    on { .ActConsole -> { } .ActSelectAll -> { } }\n",
            "}\n",
        );
        use crate::ui::action_config::set_menubar_open;
        set_menubar_open(Some("view".to_string()));
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let (view, _id_map, _probe) = builder.build_with_debug(&widget.view_tree);

        fn find_popover(v: &View<DynamicMessage>) -> Option<&View<DynamicMessage>> {
            match v {
                View::Popover { content, .. } => Some(content),
                View::Column { children, .. } | View::Row { children, .. } => {
                    children.iter().find_map(find_popover)
                }
                View::Button {
                    content: Some(c), ..
                } => find_popover(c),
                View::Container { child, .. } => find_popover(child),
                _ => None,
            }
        }
        let content = find_popover(&view).expect("declarative menubar popover");
        let (items, panel_style) = match content {
            View::Column {
                children, style, ..
            } => (children, style),
            other => panic!("panel must be a Column, got {other:?}"),
        };
        // checkbox item + separator + plain item
        assert_eq!(items.len(), 3, "menu items count");
        assert!(
            panel_style
                .as_ref()
                .unwrap()
                .classes
                .iter()
                .any(|c| matches!(c, StyleClass::Width(crate::ui::style::SizeValue::Fixed(44)))),
            "panel width class"
        );
        // Item 0: checkbox item with icon-free leading (unchecked → blank),
        // title 切换 Console.
        match &items[0] {
            View::Button {
                label,
                content: Some(inner),
                ..
            } => {
                assert_eq!(label, "切换 Console");
                match inner.as_ref() {
                    View::Row { children, .. } => {
                        assert_eq!(children.len(), 2, "left group + shortcut");
                    }
                    other => panic!("item content row, got {other:?}"),
                }
            }
            other => panic!("checkbox item must be a Button, got {other:?}"),
        }
        // Item 1: separator — horizontal hairline Column.
        match &items[1] {
            View::Column { style, .. } => {
                assert!(
                    style
                        .as_ref()
                        .unwrap()
                        .classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Height(_))),
                    "separator horizontal"
                );
            }
            other => panic!("separator must be a Column, got {other:?}"),
        }
        // Item 2: plain item with shortcut.
        match &items[2] {
            View::Button { label, .. } => assert_eq!(label, "全选"),
            other => panic!("plain item must be a Button, got {other:?}"),
        }

        // Checked expression flips with state: rebuild after writing true.
        set_menubar_open(Some("view".to_string()));
        let mut bridge2 = VmBridge::new(&widget).unwrap();
        bridge2
            .write_state("console_open", auto_val::Value::Bool(true))
            .unwrap();
        let builder2 = AuraViewBuilder::new(&bridge2, "App");
        let (view2, _id_map2, _probe2) = builder2.build_with_debug(&widget.view_tree);
        let content2 = find_popover(&view2).expect("popover after state flip");
        match content2 {
            View::Column { children, .. } => match &children[0] {
                View::Button {
                    content: Some(inner),
                    ..
                } => match inner.as_ref() {
                    View::Row { children, .. } => match &children[0] {
                        // left group row
                        View::Row { children, .. } => match &children[0] {
                            View::Image { src, .. } => {
                                assert!(
                                    src.contains("lucide:check"),
                                    "checked slot shows check, got {src}"
                                );
                            }
                            other => panic!("leading must be check image, got {other:?}"),
                        },
                        other => panic!("left group, got {other:?}"),
                    },
                    other => panic!("row, got {other:?}"),
                },
                other => panic!("button, got {other:?}"),
            },
            other => panic!("panel, got {other:?}"),
        }
    }

    /// PLAN-695 T-04: VM 解释态配色主题化 + disabled 置灰——面板底/前景/
    /// 边框消费 popover 语义 token（浅色主题白板深字）；disabled 项与
    /// 无 onclick 项置灰渲染（muted 前景 + opacity-50），不再隐藏。
    #[test]
    fn p695_menubar_panel_tokens_and_disabled_dim() {
        let src = concat!(
            "widget App {\n",
            "    model { var offline bool = false }\n",
            "    view {\n",
            "        col {\n",
            "            menubar {\n",
            "                menubar-menu (value: \"file\") {\n",
            "                    menubar-trigger \"文件\"\n",
            "                    menubar-content {\n",
            "                        menubar-item (title: \"打开\", shortcut: \"Ctrl+O\") { onclick: .ActOpen }\n",
            "                        menubar-item (title: \"断线重连\", disabled: true) { onclick: .ActReconnect }\n",
            "                        menubar-item (title: \"重命名\")\n",
            "                        menubar-separator\n",
            "                    }\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "    on { .ActOpen -> { } .ActReconnect -> { } }\n",
            "}\n",
        );
        use crate::ui::action_config::set_menubar_open;
        set_menubar_open(Some("file".to_string()));
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let (view, _id_map, _probe) = builder.build_with_debug(&widget.view_tree);

        fn find_popover(v: &View<DynamicMessage>) -> Option<&View<DynamicMessage>> {
            match v {
                View::Popover { content, .. } => Some(content),
                View::Column { children, .. } | View::Row { children, .. } => {
                    children.iter().find_map(find_popover)
                }
                View::Button {
                    content: Some(c), ..
                } => find_popover(c),
                View::Container { child, .. } => find_popover(child),
                _ => None,
            }
        }
        let content = find_popover(&view).expect("declarative menubar popover");
        let (items, panel_style) = match content {
            View::Column {
                children, style, ..
            } => (children, style),
            other => panic!("panel must be a Column, got {other:?}"),
        };
        // 面板底色 = Popover 语义变体（不再是字面 Hex(0x16171B)）。
        let st = panel_style.as_ref().expect("panel style");
        assert!(
            st.classes.iter().any(|c| matches!(
                c,
                StyleClass::BackgroundColor(crate::ui::style::Color::Popover)
            )),
            "panel bg must be popover token, got {:?}",
            st.classes
        );
        assert!(
            st.classes.iter().any(|c| matches!(
                c,
                StyleClass::TextColor(crate::ui::style::Color::PopoverForeground)
            )),
            "panel foreground must be popover-foreground token, got {:?}",
            st.classes
        );
        // 三项齐渲染：可点 + disabled 置灰 + 无 handler 置灰（不再隐藏）。
        assert_eq!(items.len(), 4, "enabled + disabled + no-handler + sep");
        match &items[0] {
            View::Button {
                disabled, style, ..
            } => {
                assert!(!disabled, "item 0 enabled");
                // hover 默认（组件族根修）：可点项挂 hover:bg-accent 变体，
                // 基础 classes 不含底色（静止态 chromeless）。
                let s = style.as_ref().expect("item 0 style");
                assert!(
                    s.has_variant(crate::ui::style::Variant::Hover)
                        && s.variant_slice(crate::ui::style::Variant::Hover)
                            .iter()
                            .any(|c| matches!(
                                c,
                                StyleClass::BackgroundColor(crate::ui::style::Color::Accent)
                            )),
                    "enabled item must carry hover:bg-accent, got {:?}",
                    s.variant_classes
                );
                assert!(
                    !s.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::BackgroundColor(_))),
                    "item rest state must stay chromeless, got {:?}",
                    s.classes
                );
            }
            other => panic!("item 0 must be Button, got {other:?}"),
        }
        for (idx, label, as_button) in [(1usize, "断线重连", true), (2, "重命名", false)] {
            let st = match &items[idx] {
                View::Button {
                    disabled, style, ..
                } if as_button => {
                    assert!(disabled, "item {idx} ({label}) must be disabled");
                    style.as_ref().expect("dimmed style")
                }
                View::Row { style, .. } if !as_button => style.as_ref().expect("static row style"),
                other => panic!("item {idx} ({label}) unexpected shape, got {other:?}"),
            };
            assert!(
                st.classes
                    .iter()
                    .any(|c| matches!(c, StyleClass::Opacity(50))),
                "item {idx} must carry opacity-50, got {:?}",
                st.classes
            );
        }
        set_menubar_open(None);
    }

    /// PLAN-695 T-05: 开态 trigger accent 高亮 + hover-switch——开菜单 A
    /// 时，B 的 trigger 包 MouseArea（on_enter = __MenubarToggle(B)），
    /// A 自己与全关态不包裹；高亮走 bg-accent/text-accent-foreground。
    #[test]
    fn p695_menubar_hover_switch_and_open_highlight() {
        let src = concat!(
            "widget App {\n",
            "    view {\n",
            "        col {\n",
            "            menubar {\n",
            "                menubar-menu (value: \"file\") {\n",
            "                    menubar-trigger \"文件\"\n",
            "                    menubar-content {\n",
            "                        menubar-item (title: \"打开\") { onclick: .ActOpen }\n",
            "                    }\n",
            "                }\n",
            "                menubar-menu (value: \"edit\") {\n",
            "                    menubar-trigger \"编辑\"\n",
            "                    menubar-content {\n",
            "                        menubar-item (title: \"重做\") { onclick: .ActRedo }\n",
            "                    }\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "    on { .ActOpen -> { } .ActRedo -> { } }\n",
            "}\n",
        );
        use crate::ui::action_config::set_menubar_open;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");

        fn find_popovers<'a>(
            v: &'a View<DynamicMessage>,
            out: &mut Vec<&'a crate::ui::view::PopoverAnchor<DynamicMessage>>,
        ) {
            match v {
                View::Popover { anchor, .. } => {
                    out.push(anchor);
                }
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children {
                        find_popovers(c, out);
                    }
                }
                View::Button {
                    content: Some(c), ..
                } => find_popovers(c, out),
                View::Container { child, .. } => find_popovers(child, out),
                _ => {}
            }
        }
        fn accent_on(v: &View<DynamicMessage>) -> bool {
            match v {
                View::Button { style: Some(s), .. } => s.classes.iter().any(|c| {
                    matches!(
                        c,
                        StyleClass::BackgroundColor(crate::ui::style::Color::Accent)
                    )
                }),
                View::MouseArea { content, .. } => accent_on(content),
                _ => false,
            }
        }

        // 开 file：file trigger = accent 高亮 Button（无包裹）；edit trigger
        // = MouseArea 包裹且 on_enter = toggle("edit")。
        set_menubar_open(Some("file".to_string()));
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let (view, _id_map, _probe) = builder.build_with_debug(&widget.view_tree);
        let mut anchors = Vec::new();
        find_popovers(&view, &mut anchors);
        assert_eq!(anchors.len(), 2, "two menubar popovers");
        let (file_anchor, edit_anchor) = (anchors[0], anchors[1]);
        match file_anchor {
            crate::ui::view::PopoverAnchor::Widget(b) => {
                assert!(accent_on(b), "open trigger must carry accent bg");
                assert!(
                    matches!(b.as_ref(), View::Button { .. }),
                    "open trigger unwrapped, got {:?}",
                    std::mem::discriminant(b.as_ref())
                );
            }
            other => panic!("file anchor must be Widget, got {other:?}"),
        }
        match edit_anchor {
            crate::ui::view::PopoverAnchor::Widget(b) => {
                assert!(!accent_on(b), "inactive trigger no accent");
                match b.as_ref() {
                    View::MouseArea {
                        on_enter: Some(msg),
                        ..
                    } => {
                        let repr = format!("{msg:?}");
                        assert!(
                            repr.contains("menubar_toggle") && repr.contains("edit"),
                            "hover switch must emit toggle(edit), got {repr}"
                        );
                    }
                    other => panic!(
                        "inactive trigger must be MouseArea-wrapped while open, got {:?}",
                        std::mem::discriminant(other)
                    ),
                }
            }
            other => panic!("edit anchor must be Widget, got {other:?}"),
        }

        // 全关态：两 trigger 均裸 Button、无 accent、无包裹（悬停不开）。
        set_menubar_open(None);
        let bridge2 = VmBridge::new(&widget).unwrap();
        let builder2 = AuraViewBuilder::new(&bridge2, "App");
        let (view2, _id_map2, _probe2) = builder2.build_with_debug(&widget.view_tree);
        let mut anchors2 = Vec::new();
        find_popovers(&view2, &mut anchors2);
        assert_eq!(anchors2.len(), 2);
        for a in &anchors2 {
            match a {
                crate::ui::view::PopoverAnchor::Widget(b) => {
                    assert!(
                        matches!(b.as_ref(), View::Button { .. }),
                        "closed state: bare button"
                    );
                    assert!(!accent_on(b), "closed state: no accent");
                    // hover 默认：闭态 trigger 挂 hover:bg-accent 变体（基础
                    // classes 仍无 accent——静止态与断言上行为不变）。
                    match b.as_ref() {
                        View::Button { style: Some(s), .. } => {
                            assert!(
                                s.variant_slice(crate::ui::style::Variant::Hover)
                                    .iter()
                                    .any(|c| matches!(
                                        c,
                                        StyleClass::BackgroundColor(
                                            crate::ui::style::Color::Accent
                                        )
                                    )),
                                "closed trigger must carry hover:bg-accent, got {:?}",
                                s.variant_classes
                            );
                        }
                        other => panic!("closed trigger style missing, got {other:?}"),
                    }
                }
                other => panic!("closed state anchor must be Widget, got {other:?}"),
            }
        }
    }

    /// PLAN-695 T-06/T-07: submenu 嵌套 + radio/label/shortcut 渲染——
    /// sub-trigger 挂嵌套 Popover（RightTop + 复合键 `file::sub-2`）；
    /// radio 选中项 circle-dot 指示、label/shortcut muted 静态文本。
    #[test]
    fn p695_menubar_submenu_radio_label_shortcut() {
        let src = concat!(
            "widget App {\n",
            "    model { var theme string = \"dark\" }\n",
            "    view {\n",
            "        col {\n",
            "            menubar {\n",
            "                menubar-menu (value: \"file\") {\n",
            "                    menubar-trigger \"文件\"\n",
            "                    menubar-content {\n",
            "                        menubar-label (text: \"外观\")\n",
            "                        menubar-radio-group (value: .theme) {\n",
            "                            menubar-radio-item (value: \"light\", title: \"浅色\") { onclick: .ActLight }\n",
            "                            menubar-radio-item (value: \"dark\", title: \"深色\") { onclick: .ActDark }\n",
            "                        }\n",
            "                        menubar-separator\n",
            "                        menubar-sub {\n",
            "                            menubar-sub-trigger (text: \"导入\")\n",
            "                            menubar-sub-content {\n",
            "                                menubar-item (title: \"从文件\") { onclick: .ActImport }\n",
            "                            }\n",
            "                        }\n",
            "                        menubar-shortcut \"⌘Q\"\n",
            "                    }\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "    on { .ActLight -> { } .ActDark -> { } .ActImport -> { } }\n",
            "}\n",
        );
        use crate::ui::action_config::set_menubar_open;
        set_menubar_open(Some("file".to_string()));
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        let (view, _id_map, _probe) = builder.build_with_debug(&widget.view_tree);

        fn find_popover(v: &View<DynamicMessage>) -> Option<&View<DynamicMessage>> {
            match v {
                View::Popover { content, .. } => Some(content),
                View::Column { children, .. } | View::Row { children, .. } => {
                    children.iter().find_map(find_popover)
                }
                View::Button {
                    content: Some(c), ..
                } => find_popover(c),
                View::Container { child, .. } => find_popover(child),
                _ => None,
            }
        }
        let content = find_popover(&view).expect("main popover");
        let (items, _) = match content {
            View::Column {
                children, style, ..
            } => (children, style),
            other => panic!("panel Column, got {other:?}"),
        };
        // label + radio-group + sep + sub + shortcut = 5 项全渲染。
        assert_eq!(
            items.len(),
            5,
            "label+radio+sep+sub+shortcut, got {items:?}"
        );

        // label：muted 静态文本（外观）。
        match &items[0] {
            View::Text { content, style, .. } => {
                assert_eq!(content, "外观");
                let st = style.as_ref().unwrap();
                assert!(
                    st.classes.iter().any(|c| matches!(
                        c,
                        StyleClass::TextColor(crate::ui::style::Color::OnSurface)
                    )),
                    "label muted color, got {:?}",
                    st.classes
                );
            }
            other => panic!("label must be Text, got {other:?}"),
        }
        // radio-group：dark 选中 → circle-dot 前导（.theme = "dark"）。
        match &items[1] {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 2, "two radio items");
                match &children[1] {
                    View::Button {
                        content: Some(inner),
                        ..
                    } => match inner.as_ref() {
                        View::Row { children, .. } => match &children[0] {
                            View::Row { children, .. } => match &children[0] {
                                View::Image { src, .. } => assert!(
                                    src.contains("circle-dot"),
                                    "selected radio shows circle-dot, got {src}"
                                ),
                                other => panic!("leading slot, got {other:?}"),
                            },
                            other => panic!("left group, got {other:?}"),
                        },
                        other => panic!("radio content row, got {other:?}"),
                    },
                    other => panic!("radio item Button, got {other:?}"),
                }
            }
            other => panic!("radio group Column, got {other:?}"),
        }
        // submenu（PLAN-698 T-03b 浮动式恢复）：trigger 行包装为嵌套
        // Popover（RightTop 顶对齐右弹，T-06 变体）；闭态 open=false，
        // 面板项在 content Column 内。AUTO_MENU_SUBMENU_INLINE=1 降级路径
        // 仍回旧内联形态（左 hairline 缩进节）。
        match &items[3] {
            View::Popover {
                anchor,
                content,
                placement,
                open,
                ..
            } => {
                assert!(!*open, "closed state: nested popover not open");
                assert_eq!(
                    *placement,
                    PopoverPlacement::RightTop,
                    "T-06 top-aligned right-open"
                );
                match anchor {
                    PopoverAnchor::Widget(trigger) => match trigger.as_ref() {
                        View::Button { label, .. } => assert_eq!(label, "导入"),
                        other => panic!("sub trigger Button, got {other:?}"),
                    },
                    other => panic!("widget anchor, got {other:?}"),
                }
                match content.as_ref() {
                    View::Column {
                        children, style, ..
                    } => {
                        assert_eq!(children.len(), 1, "sub panel items");
                        match &children[0] {
                            View::Button { label, .. } => assert_eq!(label, "从文件"),
                            other => panic!("sub item Button, got {other:?}"),
                        }
                        let st = style.as_ref().expect("panel style");
                        assert!(
                            st.classes.iter().any(|c| matches!(
                                c,
                                StyleClass::BackgroundColor(crate::ui::style::Color::Popover)
                            )),
                            "floating panel carries popover chrome, got {:?}",
                            st.classes
                        );
                    }
                    other => panic!("sub panel Column, got {other:?}"),
                }
            }
            other => panic!("sub trigger wrapped in nested Popover, got {other:?}"),
        }
        // shortcut：右对齐 muted 静态文本。
        match &items[4] {
            View::Text { content, .. } => assert_eq!(content, "⌘Q"),
            other => panic!("shortcut must be Text, got {other:?}"),
        }
        set_menubar_open(Some("file::sub-3".to_string()));
        let builder_sub = AuraViewBuilder::new(&bridge, "App");
        let (view_sub, _m, _p) = builder_sub.build_with_debug(&widget.view_tree);
        let content_sub = find_popover(&view_sub).expect("popover with sub open");
        let items_sub = match content_sub {
            View::Column { children, .. } => children,
            other => panic!("panel Column, got {other:?}"),
        };
        assert_eq!(
            items_sub.len(),
            5,
            "open state: same item count (floating popover open flag flips, no inline insert)"
        );
        match &items_sub[3] {
            View::Popover { open, .. } => {
                assert!(*open, "sub open via composite key file::sub-3");
            }
            other => panic!("sub popover present when open, got {other:?}"),
        }
        set_menubar_open(None);
    }

    /// Plan 448 I: grid `cols:` dynamic values. A non-literal expression
    /// evaluates per rebuild through the state resolver (the same cycle
    /// style props ride); literals keep the zero-eval `extract_u16` path;
    /// an unresolvable expression falls to the 1-column default instead of
    /// clamping garbage.
    #[test]
    fn plan448_grid_dynamic_cols_resolution() {
        use auto_lang_core::parser::Parser;
        let src = concat!(
            "widget App {\n",
            "    model { var n int = 3 }\n",
            "    view { col { text \"x\" {} } }\n",
            "}\n",
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let children = vec![AuraNode::Text(AuraTextContent::Literal("x".into()))];
        let dyn_cols = auto_lang_core::ast::Expr::Dot(
            Box::new(auto_lang_core::ast::Expr::Ident("self".into())),
            "n".into(),
        );

        let mut props: HashMap<String, AuraPropValue> = HashMap::new();
        props.insert("cols".to_string(), AuraPropValue::Expr(dyn_cols.clone()));

        // Dynamic: initial state n = 3.
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");
        match builder.convert_grid(&props, &children, &Bindings::new()) {
            View::Grid { cols, .. } => assert_eq!(cols, 3, "dynamic cols from state"),
            other => panic!("expected Grid, got {other:?}"),
        }

        // Dynamic: flipped state n = 16 (038 difficulty switch shape).
        let mut bridge2 = VmBridge::new(&widget).unwrap();
        bridge2.write_state("n", auto_val::Value::Int(16)).unwrap();
        let builder2 = AuraViewBuilder::new(&bridge2, "App");
        match builder2.convert_grid(&props, &children, &Bindings::new()) {
            View::Grid { cols, .. } => assert_eq!(cols, 16, "dynamic cols follows state"),
            other => panic!("expected Grid, got {other:?}"),
        }

        // Literal path unchanged.
        props.insert(
            "cols".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Int(7)),
        );
        match builder2.convert_grid(&props, &children, &Bindings::new()) {
            View::Grid { cols, .. } => assert_eq!(cols, 7, "literal cols"),
            other => panic!("expected Grid, got {other:?}"),
        }

        // Unresolvable expression → 1-column default (no clamp of garbage).
        props.insert(
            "cols".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident("missing_field".into())),
        );
        match builder2.convert_grid(&props, &children, &Bindings::new()) {
            View::Grid { cols, .. } => assert_eq!(cols, 1, "eval failure defaults to 1"),
            other => panic!("expected Grid, got {other:?}"),
        }
    }

    /// Plan 448 D: `style: [...]` array-of-class-parts and manual string
    /// concat resolve through the VM string resolver — parts join with a
    /// single space (empty parts skipped), the conditional part follows the
    /// state, and `"base" + if …` no longer falls to the empty-string
    /// fallback.
    #[test]
    fn plan448_style_array_and_concat_resolution() {
        use auto_lang_core::parser::Parser;
        let src = concat!(
            "widget App {\n",
            "    model { var on bool = true }\n",
            "    view {\n",
            "        col {\n",
            "            row (style: [\"items-center gap-1\", if .on { \"\" } else { \"opacity-40\" }]) { text \"a\" {} }\n",
            "            row (style: [\"gap-2\", \"p-4\"]) { text \"b\" {} }\n",
            "            row (style: \"base\" + if .on { \" on-cls\" } else { \" off-cls\" }) { text \"c\" {} }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");

        // Collect the three row style exprs in view order.
        fn collect_rows(node: &AuraNode, out: &mut Vec<auto_lang_core::ast::Expr>) {
            match node {
                AuraNode::Element {
                    tag,
                    props,
                    children,
                    ..
                } => {
                    if tag == "row" {
                        if let Some(auto_ui_compiler::aura::AuraPropValue::Expr(e)) =
                            props.get("style")
                        {
                            out.push(e.clone());
                        }
                    }
                    for c in children {
                        collect_rows(c, out);
                    }
                }
                AuraNode::ForLoop { body, .. } => {
                    for c in body {
                        collect_rows(c, out);
                    }
                }
                AuraNode::Conditional {
                    then_body,
                    else_body,
                    ..
                } => {
                    for c in then_body {
                        collect_rows(c, out);
                    }
                    if let Some(els) = else_body {
                        for c in els {
                            collect_rows(c, out);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut rows = Vec::new();
        collect_rows(&widget.view_tree, &mut rows);
        assert_eq!(rows.len(), 3, "three styled rows");

        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");

        // on = true (initial): conditional part empty → skipped; concat picks
        // the then-branch.
        assert_eq!(
            builder.resolve_expr_to_string_with(&rows[0], &Bindings::new()),
            "items-center gap-1",
            "array join skips the empty conditional part"
        );
        assert_eq!(
            builder.resolve_expr_to_string_with(&rows[1], &Bindings::new()),
            "gap-2 p-4",
            "literal-only array joins"
        );
        assert_eq!(
            builder.resolve_expr_to_string_with(&rows[2], &Bindings::new()),
            "base on-cls",
            "manual concat resolves (was empty-string fallback)"
        );

        // Flip state: conditional part appears; concat picks else-branch.
        let mut bridge_mut = VmBridge::new(&widget).unwrap();
        bridge_mut
            .write_state("on", auto_val::Value::Bool(false))
            .unwrap();
        let builder2 = AuraViewBuilder::new(&bridge_mut, "App");
        assert_eq!(
            builder2.resolve_expr_to_string_with(&rows[0], &Bindings::new()),
            "items-center gap-1 opacity-40",
            "array join follows state"
        );
        assert_eq!(
            builder2.resolve_expr_to_string_with(&rows[2], &Bindings::new()),
            "base off-cls",
            "concat follows state"
        );
    }

    #[test]
    fn test_build_text_literal() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::text("Hello World");
        let view = builder.build(&node);

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Hello World");
            }
            _ => panic!("Expected View::Text"),
        }
    }

    /// Plan 450 / 019 批次三: AutoDown 面板词汇 → iced 降级(View 分解)。
    /// heading/quote/callout/details 的 VM 转换;codeblock/table 走既有臂。
    #[test]
    fn test_autodown_panel_heading() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        // level 2 → h2 默认样式(text-primary + mt-8 mb-4,与 h1..h6 臂同源;
        // PLAN-075 起带 tracking-tight)。
        let node = AuraNode::element("heading")
            .with_prop("level", Expr::Int(2))
            .with_child(AuraNode::text("Title"));
        match builder.build(&node) {
            View::Text { content, style, .. } => {
                assert_eq!(content, "Title");
                let expected =
                    Style::parse("text-3xl font-bold tracking-tight text-primary mt-8 mb-4")
                        .unwrap();
                assert_eq!(style.expect("heading style").classes, expected.classes);
            }
            _ => panic!("Expected View::Text for heading"),
        }

        // level 钳位:0 → 1 档,9 → 6 档。
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("heading")
            .with_prop("level", Expr::Int(9))
            .with_child(AuraNode::text("small"));
        match builder.build(&node) {
            View::Text { style, .. } => {
                let expected = Style::parse("text-sm font-semibold mb-1").unwrap();
                assert_eq!(style.unwrap().classes, expected.classes);
            }
            _ => panic!("Expected View::Text for clamped heading"),
        }
    }

    #[test]
    fn test_autodown_panel_quote() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("quote")
            .with_child(AuraNode::text("first"))
            .with_child(AuraNode::text("second"));
        match builder.build(&node) {
            View::Container { child, style, .. } => {
                let expected =
                    Style::parse("border-l-4 pl-4 py-2 w-full text-muted-foreground").unwrap();
                assert_eq!(style.expect("quote style").classes, expected.classes);
                // 多子包 Column,Empty 子被过滤。
                match *child {
                    View::Column { children, .. } => assert_eq!(children.len(), 2),
                    _ => panic!("Expected View::Column inside quote"),
                }
            }
            _ => panic!("Expected View::Container for quote"),
        }
    }

    #[test]
    fn test_autodown_panel_callout() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("callout")
            .with_prop("kind", Expr::Str("warning".into()))
            .with_prop("title", Expr::Str("小心".into()))
            .with_child(AuraNode::text("body"));
        match builder.build(&node) {
            View::Container { child, style, .. } => {
                // 颜色 class 解析为 Rgba;amber-500 = rgb(245,158,11)。
                let style_dbg = format!("{:?}", style.expect("callout style"));
                assert!(
                    style_dbg.contains("r: 245, g: 158, b: 11"),
                    "warning tint: {style_dbg}"
                );
                match *child {
                    View::Column { children, .. } => {
                        // title 头 + 1 个内容子
                        assert_eq!(children.len(), 2);
                        match &children[0] {
                            View::Text { content, style, .. } => {
                                assert_eq!(content, "小心");
                                // 无 alpha 的 text-amber-400 保持色板枚举形态
                                // (容器 border-amber-500/40 带 alpha 才转 Rgba)。
                                assert!(
                                    format!("{:?}", style).contains("Amber(400)"),
                                    "title tint"
                                );
                            }
                            _ => panic!("Expected title Text"),
                        }
                    }
                    _ => panic!("Expected View::Column inside callout"),
                }
            }
            _ => panic!("Expected View::Container for callout"),
        }
    }

    #[test]
    fn test_autodown_panel_details() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("details")
            .with_prop("summary", Expr::Str("展开".into()))
            .with_child(AuraNode::text("hidden body"));
        match builder.build(&node) {
            View::Accordion {
                items, on_toggle, ..
            } => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].title, "展开");
                assert!(items[0].expanded, "VM 降级默认展开");
                assert_eq!(items[0].children.len(), 1);
                assert!(on_toggle.is_none(), "无 toggle 回写");
            }
            _ => panic!("Expected View::Accordion for details"),
        }
    }

    /// 注册位面板(math/query/embed):降级为可见源码/引用文本,不静默丢内容。
    #[test]
    fn test_autodown_panel_registry_slots() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        let cases: Vec<(&str, &str, &str)> = vec![
            ("math_block", "source", "E = mc^2"),
            ("query_block", "query", "{{query todo}}"),
            ("embed_block", "target", "((blk-1))"),
        ];
        for (tag, prop, value) in cases {
            let builder = AuraViewBuilder::new(&bridge, "Test");
            let node = AuraNode::element(tag).with_prop(prop, Expr::Str(value.into()));
            // embed 面板带 "↪ " 引用前缀;math/query 原文显示。
            let expected_content = if tag == "embed_block" {
                format!("↪ {value}")
            } else {
                value.to_string()
            };
            match builder.build(&node) {
                View::Container { child, style, .. } => {
                    assert!(style.is_some(), "{tag} needs a boxed style");
                    match *child {
                        View::Text { content, .. } => {
                            assert_eq!(content, expected_content, "{tag} content must be visible");
                        }
                        _ => panic!("Expected inner View::Text for {tag}"),
                    }
                }
                _ => panic!("Expected View::Container for {tag}"),
            }
        }
    }

    /// Plan 019 批次七: markdown/autodown widget 真渲染臂 —— content: 经
    /// autodown-core parse_blocks 解析为面板树（feature autodown；无 feature
    /// 时 textarea 降级由其余测试矩阵覆盖）。
    #[cfg(feature = "autodown")]
    #[test]
    /// PLAN-043 T5：StreamCache 注册表容量上限——插入序 FIFO 淘汰，超容
    /// 后最旧键不在册、容量恒 ≤ 32、既有键复用不占新位。
    #[cfg(feature = "autodown")]
    #[test]
    fn p043_stream_registry_capacity_eviction() {
        let reg = autodown_stream_registry();
        // 全局表独占清场（同进程其它用例可能残留）
        {
            let mut g = reg.lock().unwrap();
            g.0.clear();
            g.1.clear();
        }
        let n = AUTODOWN_STREAM_REGISTRY_CAP + 8;
        for i in 0..n {
            let mut g = reg.lock().unwrap();
            stream_registry_entry(&mut g, format!("p043_reg_{i}"));
        }
        {
            let g = reg.lock().unwrap();
            assert_eq!(g.0.len(), AUTODOWN_STREAM_REGISTRY_CAP, "容量封顶");
            assert!(!g.0.contains_key("p043_reg_0"), "最旧键已逐出");
            assert!(!g.0.contains_key("p043_reg_7"), "第 8 旧键已逐出");
            assert!(g.0.contains_key("p043_reg_8"), "第 9 键起在册（恰留 32）");
            assert!(
                g.0.contains_key(&format!("p043_reg_{}", n - 1)),
                "最新键在册"
            );
        }
        // 既有键复用不占新位、不触发淘汰
        {
            let mut g = reg.lock().unwrap();
            stream_registry_entry(&mut g, "p043_reg_8".to_string());
            assert_eq!(g.0.len(), AUTODOWN_STREAM_REGISTRY_CAP);
            assert!(g.0.contains_key("p043_reg_9"), "复用不淘汰次旧键");
        }
    }

    fn test_autodown_widget_true_render() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("autodown")
            .with_prop("content", Expr::Str("# 标题\n\n段落 **粗**\n".into()))
            .with_prop("final", Expr::Bool(true))
            // Plan 040 契约扩展 props：PLAN-043 T2 起 scroll_sync 真消费——
            // 文档外包 View::Scrollable（本用例未给 scroll_top/onscroll，
            // 绑定/消息两臂为 None）；streaming 恒按 final（豁免保留）；
            // PLAN-044 T4 起 placeholder_*（ghost）真消费——`block-0` 命中
            // 首块前置定高灰盒。
            .with_prop("streaming", Expr::Bool(true))
            .with_prop("placeholder_block_id", Expr::Str("block-0".into()))
            .with_prop("placeholder_height", Expr::Float(96.0, "96".into()))
            .with_prop("scroll_sync", Expr::Bool(true));
        match builder.build(&node) {
            View::Scrollable {
                child,
                offset,
                on_scroll,
                ..
            } => {
                assert_eq!(offset, None, "no scroll_top prop → no offset binding");
                assert!(
                    on_scroll.is_none(),
                    "no onscroll event → no read-out message"
                );
                match *child {
                    View::Column { children, .. } => {
                        assert_eq!(children.len(), 2);
                        // PLAN-044 T4：ghost 消费——首块被 Column[ghost, block]
                        // spacing=0 包裹，ghost = Container{height:96, bg-muted}
                        //（ignored→consumed 断言翻转）。
                        let View::Column {
                            children: wrap,
                            spacing,
                            ..
                        } = &children[0]
                        else {
                            panic!("expected ghost wrap column at block 0")
                        };
                        assert_eq!(*spacing, 0);
                        assert_eq!(wrap.len(), 2);
                        match &wrap[0] {
                            View::Container { height, style, .. } => {
                                assert_eq!(*height, Some(96), "ghost height consumed");
                                let expected = Style::parse("bg-muted rounded-lg w-full").unwrap();
                                assert_eq!(style.as_ref().unwrap().classes, expected.classes);
                            }
                            _ => panic!("expected ghost container"),
                        }
                        match &wrap[1] {
                            View::Text { content, style, .. } => {
                                assert_eq!(content, "标题");
                                let expected = Style::parse(
                                    "text-4xl font-bold tracking-tight text-primary mb-4",
                                )
                                .unwrap();
                                assert_eq!(style.as_ref().unwrap().classes, expected.classes);
                            }
                            _ => panic!("expected heading text inside ghost wrap"),
                        }
                        // 未命中块不包裹。
                        assert!(matches!(&children[1], View::Row { .. }));
                    }
                    _ => panic!("expected document column inside scrollable"),
                }
            }
            _ => panic!("expected View::Scrollable wrapper (scroll_sync consumed)"),
        }

        // markdown 别名同臂；final 缺省 true。
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node =
            AuraNode::element("markdown").with_prop("content", Expr::Str("- 甲\n- 乙\n".into()));
        match builder.build(&node) {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 1);
                assert!(matches!(&children[0], View::Column { .. }), "list column");
            }
            _ => panic!("expected document column"),
        }
    }

    /// PLAN-043 T2：scroll_sync 双臂消费——scroll_top 绑定求值进 offset
    /// 写入臂，onscroll 事件进 on_scroll 读出臂（回调收 ScrollMetrics 六
    /// 测量构造 Typed 消息：args = [top, height, client]）。普通臂
    /// （render_document）与流式臂（render_document_streamed）同一包装。
    #[cfg(feature = "autodown")]
    #[test]
    fn test_autodown_scroll_sync_binding_and_message() {
        use crate::ui::view::ScrollMetrics;

        let widget = make_test_widget("Test", vec![]);
        // plan-446 C1-3 共享 Temp 包竞态（并行用例偶发 InvalidState——
        // plan448 用例同款受害，041 复审在案的隔离 flake）：重试三拍。
        let mut bridge = VmBridge::new(&widget);
        for _ in 0..3 {
            if bridge.is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            bridge = VmBridge::new(&widget);
        }
        let bridge = bridge.unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("autodown")
            .with_prop("content", Expr::Str("# 文档\n\n正文。\n".into()))
            .with_prop("scroll_sync", Expr::Bool(true))
            .with_prop("scroll_top", Expr::Float(128.0, "128".into()))
            .with_event("onscroll", ".LeftScroll");
        match builder.build(&node) {
            View::Scrollable {
                offset,
                on_scroll: Some(cb),
                ..
            } => {
                assert_eq!(
                    offset,
                    Some((0.0, 128.0)),
                    "scroll_top binding → offset write arm"
                );
                match cb.call(ScrollMetrics {
                    offset_x: 0.0,
                    offset_y: 42.0,
                    viewport_w: 100.0,
                    viewport_h: 200.0,
                    content_w: 100.0,
                    content_h: 800.0,
                }) {
                    DynamicMessage::Typed {
                        widget_name,
                        event_name,
                        args,
                    } => {
                        assert_eq!(widget_name, "Test");
                        assert_eq!(event_name, "LeftScroll");
                        // PLAN-043 T6 实参序（height, client, top）——top 居末
                        // 位是 VM 引擎整值 float 实参绑定 bug 的绕道形态
                        //（EDITOR-CONTRACT §11 注记）。
                        assert_eq!(args.len(), 3, "height/client/top three measurements");
                        assert!(
                            matches!(args[0], auto_val::Value::Float(f) if (f - 800.0).abs() < 1e-6),
                            "scrollHeight = args[0], got {:?}",
                            args[0]
                        );
                        assert!(
                            matches!(args[1], auto_val::Value::Float(f) if (f - 200.0).abs() < 1e-6),
                            "clientHeight = args[1], got {:?}",
                            args[1]
                        );
                        assert!(
                            matches!(args[2], auto_val::Value::Float(f) if (f - 42.0).abs() < 1e-6),
                            "scrollTop = args[2], got {:?}",
                            args[2]
                        );
                    }
                    other => panic!("expected Typed message, got {:?}", other),
                }
            }
            _ => panic!("expected View::Scrollable with offset + on_scroll"),
        }

        // 流式臂同包装（build_with_debug_gated → tracked ctx →
        // render_document_streamed）。registry 键 = 节点 path，两用例树形
        // 同构会同键——内容不同则整帧重建，无复用串扰。
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("autodown")
            .with_prop("content", Expr::Str("# 流式\n\n正文。\n".into()))
            .with_prop("final", Expr::Bool(true))
            .with_prop("scroll_sync", Expr::Bool(true))
            .with_prop("scroll_top", Expr::Float(64.0, "64".into()))
            .with_event("onscroll", ".RightScroll");
        let (view, _id_map, _probe) = builder.build_with_debug_gated(&node, false);
        match view {
            View::Scrollable {
                offset,
                on_scroll: Some(cb),
                ..
            } => {
                assert_eq!(
                    offset,
                    Some((0.0, 64.0)),
                    "streamed arm: scroll_top binding"
                );
                match cb.call(ScrollMetrics {
                    offset_x: 0.0,
                    offset_y: 7.0,
                    viewport_w: 100.0,
                    viewport_h: 200.0,
                    content_w: 100.0,
                    content_h: 800.0,
                }) {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "RightScroll");
                        assert!(
                            matches!(args[2], auto_val::Value::Float(f) if (f - 7.0).abs() < 1e-6),
                            "scrollTop = args[2]"
                        );
                    }
                    other => panic!("streamed arm: expected Typed message, got {:?}", other),
                }
            }
            _ => panic!("streamed arm: expected View::Scrollable"),
        }
    }

    /// PLAN-044 T4：编辑壳块聚焦消息通道——onfocus 事件 →
    /// View::AutodownEditor.on_focus = FocusCallback（Typed 消息：
    /// args = [Int block, Float height]；失焦变体 (Int -1, 0.0)）。
    /// 普通臂与 tracked 臂同一装配（两镜像编辑臂对称）。
    #[cfg(feature = "autodown")]
    #[test]
    fn test_autodown_editor_on_focus_message_channel() {
        use crate::ui::view::{FocusCallback, FocusMetrics};

        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("autodown_editor")
            .with_prop("content", Expr::Str("段甲。\n".into()))
            .with_event("onfocusblock", ".OnEditorFocus");
        match builder.build(&node) {
            View::AutodownEditor {
                on_focus: Some(cb), ..
            } => {
                let msg = cb.call(FocusMetrics {
                    block: Some(1),
                    height: 128.0,
                });
                match msg {
                    DynamicMessage::Typed {
                        widget_name,
                        event_name,
                        args,
                    } => {
                        assert_eq!(widget_name, "Test");
                        assert_eq!(event_name, "OnEditorFocus");
                        assert!(
                            matches!(args[0], auto_val::Value::Int(1)),
                            "block index, got {:?}",
                            args[0]
                        );
                        assert!(
                            matches!(args[1], auto_val::Value::Float(f) if (f - 128.0).abs() < 1e-6),
                            "height, got {:?}",
                            args[1]
                        );
                    }
                    other => panic!("expected Typed message, got {:?}", other),
                }
                // 失焦变体：block=None → (Int -1, 0.0)。
                match cb.call(FocusMetrics {
                    block: None,
                    height: 0.0,
                }) {
                    DynamicMessage::Typed { args, .. } => {
                        assert!(
                            matches!(args[0], auto_val::Value::Int(-1)),
                            "blur encodes -1, got {:?}",
                            args[0]
                        );
                    }
                    other => panic!("expected Typed blur message, got {:?}", other),
                }
            }
            _ => panic!("expected View::AutodownEditor with on_focus"),
        }

        // tracked 臂（build_with_debug_gated → tracked ctx）同装配；无 onfocus
        // 事件 → on_focus None。
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("autodown_editor")
            .with_prop("content", Expr::Str("段乙。\n".into()))
            .with_event("onfocusblock", ".OnEditorFocus2");
        let (view, _id_map, _probe) = builder.build_with_debug_gated(&node, false);
        let editor = match &view {
            View::AutodownEditor { on_focus, .. } => on_focus.clone(),
            View::Scrollable { child, .. } => match child.as_ref() {
                View::AutodownEditor { on_focus, .. } => on_focus.clone(),
                _ => panic!("tracked arm: expected editor in scrollable"),
            },
            _ => panic!("tracked arm: expected editor view"),
        };
        let cb: FocusCallback<DynamicMessage> = editor.expect("tracked arm wires on_focus too");
        match cb.call(FocusMetrics {
            block: Some(0),
            height: 64.0,
        }) {
            DynamicMessage::Typed {
                event_name, args, ..
            } => {
                assert_eq!(event_name, "OnEditorFocus2");
                assert!(matches!(args[0], auto_val::Value::Int(0)));
            }
            other => panic!("tracked arm: expected Typed message, got {:?}", other),
        }
    }

    /// PLAN-732 L 组（AURA 环）：wikilink 激活读出装配——事件绑定双形态
    ///（引号完整名 `on "open-wiki-link"`（jade Vue 同名，契约 C-01 主
    /// 形态）与属性简写 `onopenwikilink`）→ View::AutodownEditor.on_link =
    /// LinkCallback → Typed 消息 args = [Str target, Str anchor]（事件名 =
    /// handler 名）。缺省无事件 → on_link None（契约 C-06）。
    #[cfg(feature = "autodown")]
    #[test]
    fn test_autodown_editor_on_link_message_channel() {
        use crate::ui::view::{LinkCallback, WikiLinkMetrics};

        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        // 引号完整名形态（Plan-367 引号式监听键原样 on"open-wiki-link"）。
        let node = AuraNode::element("autodown_editor")
            .with_prop("content", Expr::Str("段甲。\n".into()))
            .with_event("on\"open-wiki-link\"", ".OpenWikiLink");
        match builder.build(&node) {
            View::AutodownEditor {
                on_link: Some(cb), ..
            } => {
                assert_wiki_link_payload(cb, "OpenWikiLink");
            }
            _ => panic!("expected View::AutodownEditor with on_link (quoted full name)"),
        }

        // 属性简写形态。
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("autodown_editor")
            .with_prop("content", Expr::Str("段乙。\n".into()))
            .with_event("onopenwikilink", ".OpenLinkShort");
        match builder.build(&node) {
            View::AutodownEditor {
                on_link: Some(cb), ..
            } => {
                assert_wiki_link_payload(cb, "OpenLinkShort");
            }
            _ => panic!("expected View::AutodownEditor with on_link (attr shorthand)"),
        }

        // 缺省：未声明事件 → on_link None（应用零行为变化）。
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node =
            AuraNode::element("autodown_editor").with_prop("content", Expr::Str("段丙。\n".into()));
        match builder.build(&node) {
            View::AutodownEditor { on_link, .. } => {
                assert!(on_link.is_none(), "undeclared on_link defaults to None");
            }
            _ => panic!("expected View::AutodownEditor"),
        }

        fn assert_wiki_link_payload(cb: LinkCallback<DynamicMessage>, handler: &str) {
            let msg = cb.call(WikiLinkMetrics {
                target: "目标页".to_string(),
                anchor: "锚点甲".to_string(),
            });
            match msg {
                DynamicMessage::Typed {
                    widget_name,
                    event_name,
                    args,
                } => {
                    assert_eq!(widget_name, "Test");
                    assert_eq!(event_name, handler);
                    assert_eq!(args.len(), 2, "双参（第二参非源块身份）");
                    assert!(
                        matches!(&args[0], auto_val::Value::Str(s) if s.as_str() == "目标页"),
                        "args[0] = target, got {:?}",
                        args[0]
                    );
                    assert!(
                        matches!(&args[1], auto_val::Value::Str(s) if s.as_str() == "锚点甲"),
                        "args[1] = target_anchor, got {:?}",
                        args[1]
                    );
                }
                other => panic!("expected Typed message, got {:?}", other),
            }
            // 无锚变体：空串（非 None/undefined 字面）。
            let msg = cb.call(WikiLinkMetrics {
                target: "无锚页".to_string(),
                anchor: String::new(),
            });
            match msg {
                DynamicMessage::Typed { args, .. } => {
                    assert!(
                        matches!(&args[1], auto_val::Value::Str(s) if s.as_str().is_empty()),
                        "no-anchor encodes empty string, got {:?}",
                        args[1]
                    );
                }
                other => panic!("expected Typed message, got {:?}", other),
            }
        }
    }

    /// PLAN-045 T5：表格列宽双臂发射——`table_col_widths` prop（Map 状态
    /// 绑定）+ `oncolresize` 事件 → View::Table col_widths/on_col_resize。
    /// 三态：Nil state（自然宽）→ 命中 state（col_widths 取表键值）→
    /// prop/事件缺席（两 None，vue 臂同面：透传无消费方）。消息载荷 =
    /// [Str "t{表键}", Int col, Float width]（表键与 DSL state map 键同源）。
    #[cfg(feature = "autodown")]
    #[test]
    fn test_autodown_table_col_resize_emission() {
        use crate::ui::view::ColResizeMetrics;

        let widget = make_test_widget(
            "Test",
            vec![AuraStateDef {
                name: "table_widths".to_string(),
                type_info: Type::Map(
                    Box::new(Type::StrSlice),
                    Box::new(Type::List(Box::new(Type::Float))),
                ),
                initial: Expr::Object(vec![]),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();

        let table_src = "| a | b |
| --- | --- |
| 1 | 2 |
";
        let node = AuraNode::element("autodown")
            .with_prop("content", Expr::Str(table_src.into()))
            .with_prop("final", Expr::Bool(true))
            .with_prop("table_col_widths", Expr::Ident(".table_widths".into()))
            .with_event("oncolresize", ".OnColResize");

        // 态一：state 为 Nil → col_widths None（自然宽）+ 通道在；从消息
        // 读表键（"t" 前缀内容哈希）。
        let table_key = {
            let builder = AuraViewBuilder::new(&bridge, "Test");
            match builder.build(&node) {
                View::Column { children, .. } => {
                    // PLAN-018 T-05：AnchorSlot 包装族加入后 children[0] 为
                    // AnchorSlot——本 test 为 autodown 组合下首次可编（831/900
                    // 迁移漂移），解包后再对 Table 断言。
                    let inner = match &children[0] {
                        View::AnchorSlot { child, .. } => child.as_ref(),
                        v => v,
                    };
                    let View::Table {
                        col_widths,
                        on_col_resize: Some(cb),
                        ..
                    } = inner
                    else {
                        panic!(
                            "expected View::Table with resize channel, got {:?}",
                            children[0]
                        )
                    };
                    assert!(col_widths.is_none(), "Nil state → 自然宽");
                    match cb.call(ColResizeMetrics {
                        col: 1,
                        width: 133.0,
                    }) {
                        DynamicMessage::Typed {
                            widget_name,
                            event_name,
                            args,
                        } => {
                            assert_eq!(widget_name, "Test");
                            assert_eq!(event_name, "OnColResize");
                            match &args[0] {
                                auto_val::Value::Str(s) => {
                                    assert!(s.starts_with('t'), "表键 't' 前缀: {s}");
                                    s.to_string()
                                }
                                other => panic!("key str, got {other:?}"),
                            }
                        }
                        other => panic!("expected Typed, got {other:?}"),
                    }
                }
                other => panic!("expected document column, got {other:?}"),
            }
        };
        // 写入列宽 state（命中同表键）。
        {
            let mut obj = auto_val::Obj::new();
            obj.set(
                table_key.as_str(),
                auto_val::Value::Array(auto_val::Array::from_vec(vec![120.0f64, 200.0f64])),
            );
            bridge
                .write_state("table_widths", auto_val::Value::Obj(Box::new(obj)))
                .unwrap();
        }

        // 态二：state 命中 → col_widths 消费。
        {
            let builder = AuraViewBuilder::new(&bridge, "Test");
            match builder.build(&node) {
                View::Column { children, .. } => {
                    let inner = match &children[0] {
                        View::AnchorSlot { child, .. } => child.as_ref(),
                        v => v,
                    };
                    let View::Table {
                        col_widths,
                        on_col_resize: Some(_),
                        ..
                    } = inner
                    else {
                        panic!("expected View::Table")
                    };
                    assert_eq!(
                        col_widths.as_deref(),
                        Some(&[120.0f32, 200.0][..]),
                        "state map 按表键命中"
                    );
                }
                other => panic!("expected document column, got {other:?}"),
            }
        }

        // 态三：prop/事件缺席 → 两 None（绑定是消费开关；vue 臂同面——
        // schema 描述注记 vue arm ignores，透传无消费方）。
        {
            let plain = AuraNode::element("autodown")
                .with_prop("content", Expr::Str(table_src.into()))
                .with_prop("final", Expr::Bool(true));
            let builder = AuraViewBuilder::new(&bridge, "Test");
            match builder.build(&plain) {
                View::Column { children, .. } => {
                    let inner = match &children[0] {
                        View::AnchorSlot { child, .. } => child.as_ref(),
                        v => v,
                    };
                    let View::Table {
                        col_widths,
                        on_col_resize,
                        ..
                    } = inner
                    else {
                        panic!("expected View::Table")
                    };
                    assert!(col_widths.is_none(), "prop 缺席不读 state");
                    assert!(on_col_resize.is_none(), "事件缺席无通道");
                }
                other => panic!("expected document column, got {other:?}"),
            }
        }
    }

    /// PLAN-043 T2：Details 折叠消息通道——ondetailsclick 事件 →
    /// summary 行 onclick = Typed（事件名 = handler 名，args = [d<块结构键>]）。
    /// 键为 block_key 内容哈希（流式复用与全量重建同键）。
    #[cfg(feature = "autodown")]
    #[test]
    fn test_autodown_details_onclick_message_channel() {
        fn find_details_onclick(v: &View<DynamicMessage>) -> Option<&DynamicMessage> {
            match v {
                View::Row {
                    onclick, children, ..
                }
                | View::Column {
                    onclick, children, ..
                } => {
                    if onclick.is_some() {
                        return onclick.as_ref();
                    }
                    children.iter().find_map(find_details_onclick)
                }
                View::Container { child, onclick, .. } => {
                    if onclick.is_some() {
                        return onclick.as_ref();
                    }
                    find_details_onclick(child)
                }
                View::Scrollable { child, .. } => find_details_onclick(child),
                // PLAN-018 T-05：AnchorSlot 包装族——walker 需下钻（831/900
                // 迁移漂移，本 test autodown 组合下首次可编）。
                View::AnchorSlot { child, .. } => find_details_onclick(child),
                _ => None,
            }
        }

        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("autodown")
            .with_prop(
                "content",
                Expr::Str("$details(summary:\"折叠\") {\n藏起来的正文。\n}\n".into()),
            )
            .with_event("ondetailsclick", ".ToggleDetails");
        match builder.build(&node) {
            view @ (View::Column { .. } | View::Scrollable { .. }) => {
                let msg = find_details_onclick(&view)
                    .expect("details summary row must carry onclick message");
                match msg {
                    DynamicMessage::Typed {
                        widget_name,
                        event_name,
                        args,
                    } => {
                        assert_eq!(widget_name, "Test");
                        assert_eq!(event_name, "ToggleDetails");
                        assert!(
                            matches!(args.first(), Some(auto_val::Value::Str(s)) if s.starts_with('d')),
                            "args[0] = d<block_key hash>, got {:?}",
                            args.first()
                        );
                    }
                    other => panic!("expected Typed message, got {:?}", other),
                }
            }
            _ => panic!("expected document column"),
        }
    }

    /// Plan 019 批次九：`autodown_editor` 标签走可编辑变体臂
    /// （Phase 3 编辑壳；key/content/final/oninput 提取）。
    #[cfg(feature = "autodown")]
    #[test]
    fn test_autodown_editor_widget_editable_arm() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("autodown_editor")
            .with_prop("key", Expr::Str("doc-ed".into()))
            .with_prop("content", Expr::Str("# 编辑\nn".into()))
            .with_prop("final", Expr::Bool(true))
            // PLAN-048 T7（W4）：placeholder 真消费——空态文案发射进 View
            // 字段（Plan 040「读取后忽略」豁免摘除）。
            .with_prop("placeholder", Expr::Str("Start typing...".into()))
            .with_event("oninput", ".DocEdit");
        match builder.build(&node) {
            View::AutodownEditor {
                key,
                value,
                is_final,
                on_change,
                placeholder,
                ..
            } => {
                assert_eq!(key, "doc-ed");
                assert_eq!(value, "# 编辑\nn");
                assert!(is_final);
                assert!(on_change.is_some(), "oninput → on_change message");
                assert_eq!(
                    placeholder.as_deref(),
                    Some("Start typing..."),
                    "placeholder prop → View 字段发射"
                );
            }
            other => panic!("expected AutodownEditor variant"),
        }
        // 无 placeholder prop → None。
        let node2 = AuraNode::element("autodown_editor")
            .with_prop("key", Expr::Str("doc-ed2".into()))
            .with_prop("content", Expr::Str("正文".into()));
        match builder.build(&node2) {
            View::AutodownEditor { placeholder, .. } => {
                assert!(placeholder.is_none(), "missing prop → None");
            }
            other => panic!("expected AutodownEditor variant"),
        }
        use crate::ui::action_config::set_menubar_open;
        set_menubar_open(None);
    }
    /// D-GAP-4: an if/else body spliced into a row records each spliced node
    /// at its own RESULTING slot — not all at the conditional's index (which
    /// merged their probe entries: the 013 todo rows showed one checkbox
    /// carrying Toggle+StartEdit+Delete events and the × button's class).
    #[test]
    fn test_tracked_row_conditional_splice_probe_slots() {
        use crate::ui::debug::BuildProbe;
        use crate::ui::debug_id_map::DebugIdMap;

        fn event_node(tag: &str, handler: &str, label: &str) -> AuraNode {
            let mut events = HashMap::new();
            events.insert(
                "onclick".to_string(),
                AuraEvent {
                    handler: handler.to_string(),
                    params: vec![],
                },
            );
            let mut props = HashMap::new();
            if !label.is_empty() {
                props.insert(
                    "text".to_string(),
                    AuraPropValue::Expr(Expr::Str(label.into())),
                );
            }
            let mut class_props = HashMap::new();
            class_props.insert(
                "class".to_string(),
                AuraPropValue::Expr(Expr::Str(format!("cls-{}", handler).into())),
            );
            AuraNode::Element {
                tag: tag.to_string(),
                props: if tag == "button" {
                    let mut p = props.clone();
                    p.extend(class_props);
                    p
                } else {
                    class_props
                },
                events,
                children: Vec::new(),
                span: None,
                debug_id: None,
            }
        }

        // row { if .x { checkbox(ToggleTodo) text(StartEdit) button(DeleteTodo) } }
        let cond = AuraNode::Conditional {
            // Condition reads a missing state field -> evaluates false -> else
            // body taken (the evaluator has no literal-true arm).
            condition: ".definitely_missing_field_xyz".to_string(),
            then_body: vec![],
            else_body: Some(vec![
                event_node("checkbox", "ToggleTodo", ""),
                event_node("text", "StartEdit", "todo text"),
                event_node("button", "DeleteTodo", "×"),
            ]),
            span: None,
            debug_id: None,
        };
        let row = AuraNode::Element {
            tag: "row".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![cond],
            span: None,
            debug_id: None,
        };

        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let mut id_map = DebugIdMap::default();
        let mut probe = BuildProbe::new();
        let mut path = Vec::new();
        let _ = builder.convert_node_tracked_ctx(
            &row,
            &mut path,
            &mut id_map,
            &mut probe,
            &Bindings::new(),
        );

        let snap = probe.snapshot().clone();
        let by_path: Vec<(Vec<u16>, usize)> = snap
            .iter()
            .filter(|(_, e)| !e.events.is_empty())
            .map(|(p, e)| (p.clone(), e.events.len()))
            .collect();
        // Three events must land on THREE distinct paths (row children 0/1/2),
        // one event each — not all merged onto a single path.
        assert_eq!(by_path.len(), 3, "distinct event paths: {:?}", by_path);
        let mut slots: Vec<u16> = by_path
            .iter()
            .map(|(p, n)| {
                assert_eq!(*n, 1, "one event per path, got {} at {:?}", n, p);
                assert_eq!(p.len(), 1, "one-level slot under the row: {:?}", p);
                p[0]
            })
            .collect();
        slots.sort();
        assert_eq!(
            slots,
            vec![0u16, 1, 2],
            "slots 0/1/2 under the row: {:?}",
            by_path
        );
    }

    /// REGRESSION: a button whose label comes from inner `text` children
    /// (not a primary prop) must render those children's text as its label.
    ///
    /// This is the NoteItem pattern from 015-notes (sidebar.at):
    ///   button { text note.title { ... } text note.time { ... } }
    /// Before the fix, convert_button ignored children entirely, so the note
    /// title never appeared in the rendered sidebar.
    #[test]
    fn test_button_label_from_children() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        // button with a text child (literal string as the "text" prop)
        let text_child = AuraNode::Element {
            tag: "text".to_string(),
            props: {
                let mut m = HashMap::new();
                m.insert(
                    "text".to_string(),
                    AuraPropValue::Expr(Expr::Str("Welcome".into())),
                );
                m
            },
            events: HashMap::new(),
            children: Vec::new(),
            span: None,
            debug_id: None,
        };
        let button = AuraNode::Element {
            tag: "button".to_string(),
            props: HashMap::new(), // no "text"/"label" prop — label must come from children
            events: HashMap::new(),
            children: vec![text_child],
            span: None,
            debug_id: None,
        };

        let view = builder.build(&button);
        match view {
            View::Button { label, .. } => {
                assert_eq!(label, "Welcome", "button label should come from children");
            }
            other => panic!("Expected View::Button, got {:?}", other),
        }
    }

    #[test]
    fn test_build_text_with_state_ref() {
        let widget = make_test_widget(
            "Counter",
            vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(42),
                decorators: vec![],
            }],
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Text(AuraTextContent::Interpolated {
            template: "Count: ${.count}".to_string(),
            bindings: vec!["count".to_string()],
        });
        let view = builder.build(&node);

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Count: 42");
            }
            _ => panic!("Expected View::Text"),
        }
    }

    // ========================================================================
    // Plan 484 — mouse-area hover 命中区
    // ========================================================================

    #[test]
    fn test_mouse_area_conversion_events_and_style() {
        let widget = make_test_widget("Chart", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Chart");

        // 事件形态对齐解析器输出:handler 名 + params 分列(parse_event_value),
        // 而非 with_event 辅助的原始串。
        let mut node = AuraNode::element("mouse-area")
            .with_prop("style", Expr::Str("w-[64px] h-[240px]".into()));
        if let AuraNode::Element { events, .. } = &mut node {
            events.insert(
                "onmouseenter".to_string(),
                AuraEvent {
                    handler: ".Hover".to_string(),
                    params: vec!["0".to_string()],
                },
            );
            events.insert(
                "onmouseleave".to_string(),
                AuraEvent {
                    handler: ".HoverOut".to_string(),
                    params: vec![],
                },
            );
        }
        let view = builder.build(&node);

        match view {
            View::MouseArea {
                on_enter,
                on_exit,
                style,
                ..
            } => {
                let enter = on_enter.expect("onmouseenter must resolve to a message");
                match enter {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "Hover");
                        assert_eq!(args.len(), 1, "literal arg 0 must ride along");
                    }
                    other => panic!("Expected Typed message, got: {:?}", other),
                }
                assert!(on_exit.is_some(), "onmouseleave must resolve to a message");
                assert!(style.is_some(), "size classes must parse into style");
            }
            other => panic!("Expected View::MouseArea, got: {:?}", other),
        }
    }

    /// Plan 499 M2: mouse-area onmousemove + coords → on_move handler + 逻辑幅面。
    /// handler 调用时把 (x, y) 追加为 Float 实参(经 encode/decode_payload 管道
    /// 直达 VM handler 形参);coords 解析 "WxH"。
    #[test]
    fn test_mouse_area_onmousemove_coords_and_args() {
        let widget = make_test_widget("Chart", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Chart");

        let mut node =
            AuraNode::element("mouse-area").with_prop("coords", Expr::Str("560x300".into()));
        if let AuraNode::Element { events, .. } = &mut node {
            events.insert(
                "onmousemove".to_string(),
                AuraEvent {
                    handler: ".PointerMove".to_string(),
                    params: vec![],
                },
            );
            events.insert(
                "onmouseenter".to_string(),
                AuraEvent {
                    handler: ".Hover".to_string(),
                    params: vec![],
                },
            );
        }
        match builder.build(&node) {
            View::MouseArea {
                on_move,
                logical_extent,
                on_enter,
                ..
            } => {
                assert_eq!(logical_extent, Some((560.0, 300.0)), "coords WxH 解析");
                let h = on_move.expect("onmousemove must resolve to a move handler");
                match h.call(280.0, 150.0) {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "PointerMove");
                        assert_eq!(args.len(), 2, "坐标必须追加为两个实参");
                        // PLAN-043 T9: 坐标实参 +1e-3 分数化（nanbox 绕道）。
                        assert!(matches!(args[0], Value::Float(f) if (f - 280.001).abs() < 1e-9));
                        assert!(matches!(args[1], Value::Float(f) if (f - 150.001).abs() < 1e-9));
                    }
                    other => panic!("Expected Typed message, got: {:?}", other),
                }
                assert!(on_enter.is_some(), "enter 臂不受影响");
            }
            other => panic!("Expected View::MouseArea, got: {:?}", other),
        }
    }

    /// Plan 499 M2: 无 coords → raw px 模式(extent=None);无 onmousemove →
    /// on_move=None(存量 mouse-area 零变化)。
    #[test]
    fn test_mouse_area_onmousemove_without_coords_is_raw() {
        let widget = make_test_widget("Chart", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Chart");

        let mut node = AuraNode::element("mouse-area")
            .with_prop("style", Expr::Str("w-[64px] h-[64px]".into()));
        if let AuraNode::Element { events, .. } = &mut node {
            events.insert(
                "onmousemove".to_string(),
                AuraEvent {
                    handler: ".RawMove".to_string(),
                    params: vec![],
                },
            );
        }
        match builder.build(&node) {
            View::MouseArea {
                on_move,
                logical_extent,
                ..
            } => {
                assert_eq!(logical_extent, None, "无 coords = raw px 模式");
                assert!(on_move.is_some(), "onmousemove 臂仍接线");
            }
            other => panic!("Expected View::MouseArea, got: {:?}", other),
        }
    }

    // ========================================================================
    // Plan 563 — canvas 场景契约解析 + builder 链
    // ========================================================================

    #[test]
    fn plan563_parse_canvas_scene_pure() {
        use crate::ui::view::CanvasScene;
        // 正常双表:两笔画,meta 全位。
        let pts = vec![
            Value::Str("10,20|30,40|50,60".into()),
            Value::Str("1,2".into()),
        ];
        let meta = vec![
            Value::Str("#111827,3,0".into()),
            Value::Str("#ef4444,5.5,1".into()),
        ];
        let scene = parse_canvas_scene(&pts, &meta);
        assert_eq!(scene.strokes.len(), 2);
        assert_eq!(
            scene.strokes[0].points,
            vec![(10.0, 20.0), (30.0, 40.0), (50.0, 60.0)]
        );
        assert_eq!(scene.strokes[0].color, "#111827");
        assert!((scene.strokes[0].width - 3.0).abs() < f32::EPSILON);
        assert!(!scene.strokes[0].eraser);
        assert!((scene.strokes[1].width - 5.5).abs() < 1e-6);
        assert!(scene.strokes[1].eraser, "eraser 位 '1'");

        // meta 缺表/缺项宽容:默认 #111827/3px/非 eraser。
        let scene = parse_canvas_scene(&pts[..1], &[]);
        assert_eq!(scene.strokes.len(), 1);
        assert_eq!(scene.strokes[0].color, "#111827");
        assert!((scene.strokes[0].width - 3.0).abs() < f32::EPSILON);
        assert!(!scene.strokes[0].eraser);

        // 坏点宽容:坏对跳过;全坏/空串笔画丢弃。
        let pts = vec![
            Value::Str("10,20|bad|30,".into()),
            Value::Str("nope".into()),
            Value::Str("".into()),
        ];
        let scene = parse_canvas_scene(&pts, &[]);
        assert_eq!(scene.strokes.len(), 1, "仅第一笔存活(单好点)");
        assert_eq!(scene.strokes[0].points, vec![(10.0, 20.0)]);

        // 空表 → 空场景(初始态)。
        let empty: CanvasScene = parse_canvas_scene(&[], &[]);
        assert!(empty.strokes.is_empty());
    }

    #[test]
    fn plan563_canvas_builder_leaf_events_and_coords() {
        let widget = make_test_widget("Sketch", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Sketch");

        let mut node = AuraNode::element("canvas")
            .with_prop("scene", Expr::Ident("strokes".into()))
            .with_prop("coords", Expr::Str("400x300".into()))
            .with_prop("clear", Expr::Str("#ffffff".into()));
        if let AuraNode::Element { events, .. } = &mut node {
            events.insert(
                "onpenstart".to_string(),
                AuraEvent {
                    handler: ".PenStart".to_string(),
                    params: vec![],
                },
            );
            events.insert(
                "onpenmove".to_string(),
                AuraEvent {
                    handler: ".PenMove".to_string(),
                    params: vec![],
                },
            );
            events.insert(
                "onpenend".to_string(),
                AuraEvent {
                    handler: ".PenEnd".to_string(),
                    params: vec![],
                },
            );
        }
        match builder.build(&node) {
            View::Canvas {
                scene,
                logical_extent,
                clear,
                on_pen_start,
                on_pen_move,
                on_pen_end,
                ..
            } => {
                // 无 strokes_pts 状态 → 空场景降级(初始态)。
                assert!(scene.strokes.is_empty(), "缺状态表 = 空画布");
                assert_eq!(logical_extent, Some((400.0, 300.0)));
                assert_eq!(clear.as_deref(), Some("#ffffff"));
                // pen handler 坐标追加(mouse_area_move_arm 同型,+0.001 分数化)。
                match on_pen_start.expect("onpenstart 必须接线").call(12.0, 34.0) {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "PenStart");
                        assert!(matches!(args[0], Value::Float(f) if (f - 12.001).abs() < 1e-9));
                        assert!(matches!(args[1], Value::Float(f) if (f - 34.001).abs() < 1e-9));
                    }
                    other => panic!("Expected Typed message, got: {:?}", other),
                }
                let _ = (on_pen_move, on_pen_end);
            }
            other => panic!("Expected View::Canvas, got: {:?}", other),
        }
    }

    // ========================================================================
    // Plan 049 — block 折叠 VM 修复回归测试
    // ========================================================================

    #[test]
    fn test_conditional_negation_renders_then_body() {
        // Plan 049:BlockItem 折叠 body 的条件 `if !.collapsed` 在 collapsed=false
        // 时必须为 true。修复前 eval_condition_with 不认识 `!` 前缀,`! .collapsed`
        // 落入 resolve_binding_path → None → false,导致 ls 结果默认全隐藏。
        let widget = make_test_widget(
            "Fold",
            vec![AuraStateDef {
                name: "collapsed".to_string(),
                type_info: Type::Bool,
                initial: Expr::Bool(false),
                decorators: vec![],
            }],
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Fold");

        let node = AuraNode::Conditional {
            condition: "! .collapsed".to_string(),
            then_body: vec![AuraNode::text("body")],
            else_body: None,
            span: None,
            debug_id: None,
        };
        match builder.build(&node) {
            View::Text { content, .. } => {
                assert_eq!(content, "body", "!false must render the then_body");
            }
            other => panic!("expected View::Text body, got {:?}", other),
        }
    }

    #[test]
    fn test_conditional_negation_hides_when_true() {
        // collapsed=true → `if !.collapsed` → false → body 隐藏(折叠生效)。
        let widget = make_test_widget(
            "Fold",
            vec![AuraStateDef {
                name: "collapsed".to_string(),
                type_info: Type::Bool,
                initial: Expr::Bool(true),
                decorators: vec![],
            }],
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Fold");

        let node = AuraNode::Conditional {
            condition: "! .collapsed".to_string(),
            then_body: vec![AuraNode::text("body")],
            else_body: None,
            span: None,
            debug_id: None,
        };
        assert!(
            matches!(builder.build(&node), View::Empty),
            "!true must hide the body"
        );
    }

    #[test]
    fn test_text_onclick_becomes_toggle_button() {
        // Plan 049:VM 的 convert_row 丢弃 row 的 onclick,所以 block_item.at
        // 把折叠切换挂到 text 元素上 —— text 带 onclick 必须转成 chromeless
        // Button,且消息是 Typed{ event_name: "ToggleCollapse" }。
        let widget = make_test_widget("Fold", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Fold");

        let node = AuraNode::element("text")
            .with_prop("text", Expr::Ident(".collapse_glyph".into()))
            .with_event("onclick", ".ToggleCollapse");
        match builder.build(&node) {
            View::Button { onclick, .. } => match onclick {
                DynamicMessage::Typed { event_name, .. } => {
                    assert_eq!(event_name, "ToggleCollapse");
                }
                other => panic!("expected Typed ToggleCollapse, got {:?}", other),
            },
            other => panic!("text-with-onclick must convert to Button, got {:?}", other),
        }
    }

    // OS-016 D2+D3:select 循环选项收集 + value 契约——for 循环 option 不再
    // 漏收（修复前 options=[] → pick_list 空壳,os-config 016 批实况）;
    // 选中态按 option value 匹配（value≠label 场景）;回调按索引回查 value
    // （非 label）。
    #[test]
    fn test_select_for_loop_options_and_value_contract() {
        let widget = make_test_widget(
            "Sel",
            vec![AuraStateDef {
                name: "providers".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        let mk = |v: &str, l: &str| {
            let mut o = auto_val::Obj::new();
            o.set("value", Value::str(v));
            o.set("label", Value::str(l));
            Value::Obj(Box::new(o))
        };
        bridge
            .write_state(
                "providers",
                Value::Array(auto_val::Array::from(vec![
                    mk("zhipu", "Zhipu"),
                    mk("ds", "DeepSeek"),
                ])),
            )
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Sel");

        // select (value: "ds") { onchange: .Pick
        //   for o in .providers { option (value: o.value, text: o.label) } }
        let node = AuraNode::element("select")
            .with_prop("value", Expr::Str("ds".into()))
            .with_event("onchange", ".Pick")
            .with_child(AuraNode::ForLoop {
                var: "o".to_string(),
                index: None,
                iterable: ".providers".to_string(),
                key_expr: None,
                body: vec![AuraNode::element("option")
                    .with_prop(
                        "value",
                        Expr::Dot(Box::new(Expr::Ident("o".into())), "value".into()),
                    )
                    .with_prop(
                        "text",
                        Expr::Dot(Box::new(Expr::Ident("o".into())), "label".into()),
                    )],
                span: None,
                debug_id: None,
            });
        match builder.build(&node) {
            View::Select {
                options,
                selected_index,
                on_select,
                ..
            } => {
                assert_eq!(
                    options,
                    vec!["Zhipu", "DeepSeek"],
                    "D2: 循环选项全收（修复前 []）"
                );
                assert_eq!(
                    selected_index, Some(1),
                    "D3: value 'ds' 经 value 数组匹配 idx=1（label 文本 'DeepSeek'≠'ds'，纯文本匹配会 miss）"
                );
                let cb = on_select.expect("onchange present");
                match cb.call(1, "DeepSeek") {
                    DynamicMessage::String(s) => assert_eq!(
                        s, "Pick\u{1F}s\u{1F}ds",
                        "D3: 回调回传 option value 'ds'（非 label 'DeepSeek'）"
                    ),
                    other => panic!("expected encoded String message, got {:?}", other),
                }
            }
            other => panic!("expected View::Select, got {:?}", other),
        }
    }

    // OS-016 D4:select onchange 的 $event 参数位——构建期解析静态参数
    // （字面量/绑定）,回调时以所选 value 补位 $event（vue 轨
    // `.Apply(e, $event.target.value)` 的 vm 对位,Typed 消息）。
    #[test]
    fn test_select_event_slot_substitution() {
        let widget = make_test_widget(
            "Sel",
            vec![AuraStateDef {
                name: "providers".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        let mut o = auto_val::Obj::new();
        o.set("value", Value::str("zhipu"));
        o.set("label", Value::str("Zhipu"));
        bridge
            .write_state(
                "providers",
                Value::Array(auto_val::Array::from(vec![Value::Obj(Box::new(o))])),
            )
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Sel");

        let mut node = AuraNode::element("select")
            .with_event("onchange", ".Apply")
            .with_child(AuraNode::ForLoop {
                var: "o".to_string(),
                index: None,
                iterable: ".providers".to_string(),
                key_expr: None,
                body: vec![AuraNode::element("option")
                    .with_prop(
                        "value",
                        Expr::Dot(Box::new(Expr::Ident("o".into())), "value".into()),
                    )
                    .with_prop(
                        "text",
                        Expr::Dot(Box::new(Expr::Ident("o".into())), "label".into()),
                    )],
                span: None,
                debug_id: None,
            });
        // with_event 无参数变体——直填 AuraEvent.params（.at 解析器的
        // `.Apply("fixed", $event.target.value)` 参数形态）。
        if let AuraNode::Element { events, .. } = &mut node {
            if let Some(ev) = events.get_mut("onchange") {
                ev.params = vec!["\"fixed\"".to_string(), "$event.target.value".to_string()];
            }
        }
        match builder.build(&node) {
            View::Select { on_select, .. } => {
                let cb = on_select.expect("onchange present");
                match cb.call(0, "Zhipu") {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "Apply");
                        assert_eq!(args.len(), 2, "静态参数 + $event 补位");
                        assert_eq!(args[0], Value::str("fixed"));
                        assert_eq!(
                            args[1],
                            Value::str("zhipu"),
                            "$event 位收 option value（非 label 'Zhipu'）"
                        );
                    }
                    other => panic!("expected Typed message, got {:?}", other),
                }
            }
            other => panic!("expected View::Select, got {:?}", other),
        }
    }

    #[test]
    fn test_text_selectable_passthrough() {
        // Plan 481:text/label 声明 selectable: true(opt-in)→ View::Text
        // 携带 selectable,供 iced renderer 分流到 SelectableText。
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("text")
            .with_prop("text", Expr::Str("select me".into()))
            .with_prop("selectable", Expr::Bool(true));
        match builder.build(&node) {
            View::Text {
                content,
                selectable,
                ..
            } => {
                assert_eq!(content, "select me");
                assert!(selectable, "selectable: true must ride View::Text");
            }
            other => panic!("expected View::Text, got {:?}", other),
        }

        // label 同臂同型
        let node = AuraNode::element("label")
            .with_prop("text", Expr::Str("label me".into()))
            .with_prop("selectable", Expr::Bool(true));
        match builder.build(&node) {
            View::Text { selectable, .. } => assert!(selectable, "label selectable"),
            other => panic!("expected View::Text, got {:?}", other),
        }

        // 缺省(不声明)= false
        let node = AuraNode::element("text").with_prop("text", Expr::Str("plain".into()));
        match builder.build(&node) {
            View::Text { selectable, .. } => assert!(!selectable, "default off"),
            other => panic!("expected View::Text, got {:?}", other),
        }
    }

    #[test]
    fn test_child_model_var_default_seeded_into_state() {
        // Plan 049:渲染子组件(BlockList → BlockItem)时,子组件 model 变量
        // 的默认值必须种入统一 root state。否则 `.collapsed` 首次读取 Err →
        // 条件 false(内容隐藏)+ 字符串回退 "${collapse_glyph}"。
        let child = make_test_widget(
            "BlockItem",
            vec![AuraStateDef {
                name: "collapsed".to_string(),
                type_info: Type::Bool,
                initial: Expr::Bool(false),
                decorators: vec![],
            }],
        );
        let parent = make_test_widget("BlockList", vec![]);
        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "BlockList", &registry);

        let node = AuraNode::Component {
            name: "BlockItem".to_string(),
            props: vec![],
            events: HashMap::new(),
            children: vec![],
            span: None,
            debug_id: None,
        };
        builder.build(&node);
        assert_eq!(
            bridge.read_state("collapsed").unwrap(),
            Value::Bool(false),
            "child model var default must be seeded into root state"
        );
    }

    // ========================================================================
    // Plan 476 — slot（内容模板）替换机制测试
    // ========================================================================

    /// 递归查找 View 树中是否含指定文本（Text.content 或 Button.label）。
    fn view_contains_text(view: &View<DynamicMessage>, needle: &str) -> bool {
        match view {
            View::Text { content, .. } => content.contains(needle),
            View::Button { label, content, .. } => {
                label.contains(needle)
                    || content
                        .as_ref()
                        .map_or(false, |c| view_contains_text(c, needle))
            }
            View::Row { children, .. } | View::Column { children, .. } => {
                children.iter().any(|c| view_contains_text(c, needle))
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => {
                view_contains_text(child, needle)
            }
            View::Grid { cells, .. } => cells.iter().any(|c| view_contains_text(c, needle)),
            _ => false,
        }
    }

    /// 构造"父视图内调用子 widget 并传 slot 填充"的父 view_tree。
    fn slot_call_node(child_tag: &str, fill_children: Vec<AuraNode>) -> AuraNode {
        AuraNode::Component {
            name: child_tag.to_string(),
            props: vec![],
            events: HashMap::new(),
            children: fill_children,
            span: None,
            debug_id: None,
        }
    }

    /// 子 widget 视图：col > (header_text, slot outlet)。
    fn child_with_outlet(name_prop: Option<&str>, header: &str) -> AuraNode {
        let mut outlet = AuraNode::element("slot");
        if let Some(n) = name_prop {
            outlet = outlet.with_prop("name", Expr::Str(n.into()));
        }
        AuraNode::element("col")
            .with_child(AuraNode::text(header))
            .with_child(outlet)
    }

    #[test]
    fn test_slot_named_fill_renders_at_outlet() {
        // 父调用 Child 传 slot(name:"header"){ text "X" }——填充必须渲染在
        // 子视图 outlet 位置（此前整个填充被丢弃）。
        let child = make_test_widget("Child", vec![]);
        let child_widget_view = child_with_outlet(Some("header"), "title");
        let mut parent = make_test_widget("Parent", vec![]);
        parent.view_tree = slot_call_node(
            "Child",
            vec![AuraNode::element("slot")
                .with_prop("name", Expr::Str("header".into()))
                .with_child(AuraNode::text("FILL-MARKER"))],
        );
        let _ = child_widget_view; // view_tree 在下方整体赋值（child 需 mut）
        let mut child = child;
        child.view_tree = child_widget_view;

        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "Parent", &registry);

        let view = builder.build(&parent.view_tree);
        assert!(
            view_contains_text(&view, "FILL-MARKER"),
            "named slot fill must render at the outlet; got: {:?}",
            view
        );
        assert!(
            view_contains_text(&view, "title"),
            "child's own view must still render"
        );
    }

    #[test]
    fn test_slot_fill_resolves_parent_scope() {
        // 父 state label="parent-value"、子 state label="child-value"——
        // fill 内 text .label 必须解析到父作用域。
        let child = make_test_widget(
            "Child",
            vec![AuraStateDef {
                name: "label".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("child-value".into()),
                decorators: vec![],
            }],
        );
        let mut child = child;
        child.view_tree = child_with_outlet(Some("x"), "hdr");
        let mut parent = make_test_widget(
            "Parent",
            vec![AuraStateDef {
                name: "label".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("parent-value".into()),
                decorators: vec![],
            }],
        );
        parent.view_tree = slot_call_node(
            "Child",
            vec![AuraNode::element("slot")
                .with_prop("name", Expr::Str("x".into()))
                .with_child(
                    AuraNode::element("text").with_prop("text", Expr::Ident(".label".into())),
                )],
        );

        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "Parent", &registry);

        let view = builder.build(&parent.view_tree);
        assert!(
            view_contains_text(&view, "parent-value"),
            "fill must resolve state in the PARENT scope; got: {:?}",
            view
        );
        assert!(
            !view_contains_text(&view, "child-value"),
            "child's same-named field must not leak into the fill"
        );
    }

    #[test]
    fn test_slot_fill_event_routes_to_parent() {
        // fill 内 button onclick: .Clicked → Typed.widget_name 必须是父名。
        let child = make_test_widget("Child", vec![]);
        let mut child = child;
        child.view_tree = child_with_outlet(Some("x"), "hdr");
        let mut parent = make_test_widget("Parent", vec![]);
        parent.view_tree = slot_call_node(
            "Child",
            vec![AuraNode::element("slot")
                .with_prop("name", Expr::Str("x".into()))
                .with_child(AuraNode::element("button").with_event("onclick", ".Clicked"))],
        );

        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "Parent", &registry);

        let view = builder.build(&parent.view_tree);
        // 深度找 Button 断言其 onclick 路由到父 widget。
        fn find_button(view: &View<DynamicMessage>) -> Option<&DynamicMessage> {
            match view {
                View::Button {
                    onclick, content, ..
                } => {
                    if content.is_some() {
                        if let Some(c) = content {
                            if let Some(m) = find_button(c) {
                                return Some(m);
                            }
                        }
                    }
                    Some(onclick)
                }
                View::Row { children, .. } | View::Column { children, .. } => {
                    children.iter().find_map(find_button)
                }
                View::Container { child, .. } | View::Scrollable { child, .. } => {
                    find_button(child)
                }
                _ => None,
            }
        }
        match find_button(&view) {
            Some(DynamicMessage::Typed { widget_name, .. }) => {
                assert_eq!(
                    widget_name, "Parent",
                    "fill events must route to the parent widget"
                );
            }
            other => panic!("fill button must carry a Typed message, got {:?}", other),
        }
    }

    #[test]
    fn test_slot_unmatched_name_not_rendered_and_fallback_renders() {
        // fill 名不匹配任何 outlet → 不渲染；outlet 自带 children 且无匹配
        // fill → fallback 内容（子作用域）渲染。
        let child = make_test_widget("Child", vec![]);
        let mut child = child;
        child.view_tree = AuraNode::element("col").with_child(
            AuraNode::element("slot")
                .with_prop("name", Expr::Str("real".into()))
                .with_child(AuraNode::text("FALLBACK")),
        );
        let mut parent = make_test_widget("Parent", vec![]);
        parent.view_tree = slot_call_node(
            "Child",
            vec![AuraNode::element("slot")
                .with_prop("name", Expr::Str("nope".into()))
                .with_child(AuraNode::text("GHOST"))],
        );

        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "Parent", &registry);

        let view = builder.build(&parent.view_tree);
        assert!(
            !view_contains_text(&view, "GHOST"),
            "unmatched fill name must not render"
        );
        assert!(
            view_contains_text(&view, "FALLBACK"),
            "outlet fallback children must render when no fill matches; got {:?}",
            view
        );
    }

    #[test]
    fn test_slot_default_fill_from_bare_children() {
        // 调用位裸子节点（无 slot 包装）→ 渲染到子视图默认 outlet（slot 无名）。
        let child = make_test_widget("Child", vec![]);
        let mut child = child;
        child.view_tree = child_with_outlet(None, "hdr");
        let mut parent = make_test_widget("Parent", vec![]);
        parent.view_tree = slot_call_node("Child", vec![AuraNode::text("DEFAULT-FILL")]);

        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "Parent", &registry);

        let view = builder.build(&parent.view_tree);
        assert!(
            view_contains_text(&view, "DEFAULT-FILL"),
            "bare children must land in the default outlet; got {:?}",
            view
        );
    }

    #[test]
    fn test_slot_multi_child_fill_spliced_as_siblings() {
        // fill 2 子节点在子视图 row outlet → 2 个兄弟视图（Row children 数
        // 递增），而非单包装视图。
        let child = make_test_widget("Child", vec![]);
        let mut child = child;
        child.view_tree = AuraNode::element("row")
            .with_child(AuraNode::element("slot").with_prop("name", Expr::Str("acts".into())));
        let mut parent = make_test_widget("Parent", vec![]);
        parent.view_tree = slot_call_node(
            "Child",
            vec![AuraNode::element("slot")
                .with_prop("name", Expr::Str("acts".into()))
                .with_child(AuraNode::text("B1"))
                .with_child(AuraNode::text("B2"))],
        );

        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "Parent", &registry);

        let view = builder.build(&parent.view_tree);
        match view {
            View::Row { ref children, .. } => {
                // outlet 展开为 2 个兄弟文本视图（未被包成单一节点）
                assert_eq!(
                    children.len(),
                    2,
                    "fill children must splice as siblings: {:?}",
                    children
                );
                assert!(view_contains_text(&view, "B1"));
                assert!(view_contains_text(&view, "B2"));
            }
            other => panic!("expected Row root, got {:?}", other),
        }
    }

    #[test]
    fn test_slot_fill_probe_tracked() {
        // build_with_debug 后 fill 子树的 state binding 进入 BuildProbe
        // snapshot（tracked 双胎覆盖）。
        let child = make_test_widget("Child", vec![]);
        let mut child = child;
        child.view_tree = child_with_outlet(Some("x"), "hdr");
        let mut parent = make_test_widget(
            "Parent",
            vec![AuraStateDef {
                name: "greeting".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("hello".into()),
                decorators: vec![],
            }],
        );
        parent.view_tree = slot_call_node(
            "Child",
            vec![AuraNode::element("slot")
                .with_prop("name", Expr::Str("x".into()))
                .with_child(AuraNode::Text(AuraTextContent::Interpolated {
                    template: "G: ${.greeting}".to_string(),
                    bindings: vec!["greeting".to_string()],
                }))],
        );

        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "Parent", &registry);

        let (view, _id_map, probe) = builder.build_with_debug(&parent.view_tree);
        assert!(
            view_contains_text(&view, "hello"),
            "interpolated fill content must render (parent scope); got {:?}",
            view
        );
        let snap = probe.snapshot();
        let has_fill_binding = snap
            .values()
            .any(|e| e.state_bindings.iter().any(|b| b.expr.contains("greeting")));
        assert!(
            has_fill_binding,
            "fill subtree's state binding must reach the BuildProbe (tracked path); snap: {:?}",
            snap
        );
    }

    // ========================================================================
    // Plan 307 Task 9 — deep BuildProbe threading tests
    // ========================================================================

    #[test]
    fn build_with_debug_captures_nested_state_binding() {
        let widget = make_test_widget(
            "Counter",
            vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(42),
                decorators: vec![],
            }],
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![AuraNode::Text(AuraTextContent::Interpolated {
                template: "Count: ${.count}".to_string(),
                bindings: vec!["count".to_string()],
            })],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        // exactly one path captured (the nested text node), with one state binding
        assert_eq!(snap.len(), 1, "nested text node should be probed");
        let entry = snap.values().next().unwrap();
        assert_eq!(entry.state_bindings.len(), 1);
        assert_eq!(entry.state_bindings[0].expr, "${.count}");
        assert_eq!(entry.state_bindings[0].current_value, "42");
    }

    #[test]
    fn build_with_debug_skips_literal_text_sibling() {
        // col with two text children: one interpolated, one literal.
        // Only the interpolated one should produce a probe entry.
        let widget = make_test_widget(
            "Counter",
            vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(42),
                decorators: vec![],
            }],
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![
                AuraNode::Text(AuraTextContent::Interpolated {
                    template: "Count: ${.count}".to_string(),
                    bindings: vec!["count".to_string()],
                }),
                AuraNode::Text(AuraTextContent::Literal("static".to_string())),
            ],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        assert_eq!(snap.len(), 1, "only the interpolated text node is probed");
        let entry = snap.values().next().unwrap();
        assert_eq!(entry.state_bindings.len(), 1);
        assert_eq!(entry.state_bindings[0].expr, "${.count}");
    }

    #[test]
    fn build_with_debug_records_nothing_for_literal_text() {
        // A literal-only text node at top level records nothing.
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Text(AuraTextContent::Literal("just text".to_string()));
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        assert!(snap.is_empty(), "literal-only text must not be probed");
    }

    #[test]
    fn build_with_debug_captures_for_loop_context() {
        use crate::ui::debug::ForIter;
        // Widget declares an `items` state field (initial dummy, overwritten
        // below). State expr has no array literal, so we seed via write_state.
        let widget = make_test_widget(
            "List",
            vec![AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "items",
                Value::Array(auto_val::Array::from(vec![
                    Value::str("apple"),
                    Value::str("banana"),
                    Value::str("cherry"),
                ])),
            )
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "List");

        // for item in .items { text("${.item}") }
        let node = AuraNode::ForLoop {
            var: "item".to_string(),
            index: None,
            iterable: ".items".to_string(),
            key_expr: None,
            body: vec![AuraNode::Text(AuraTextContent::Interpolated {
                template: "${.item}".to_string(),
                bindings: vec!["item".to_string()],
            })],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        let for_entries: Vec<&ForIter> = snap
            .values()
            .filter_map(|e| e.for_context.as_ref())
            .collect();
        assert_eq!(for_entries.len(), 3, "three iterations captured");
        let mut by_index: Vec<(usize, &str)> = for_entries
            .iter()
            .map(|f| (f.index.unwrap(), f.value_repr.as_str()))
            .collect();
        by_index.sort_by_key(|(i, _)| *i);
        assert_eq!(by_index, vec![(0, "apple"), (1, "banana"), (2, "cherry")]);
        assert_eq!(for_entries[0].var, "item");
        assert_eq!(for_entries[0].iterable_repr, ".items");
    }

    #[test]
    fn build_with_debug_for_loop_single_body_path_matches_vtree() {
        // Plan 309 Phase 1 (Fix A): a ForLoop whose body is a single node
        // yields that node *directly* per iteration (no wrapping Column), so
        // its flattened VTree path is the one-segment `[i]`. The tracked
        // builder must record under the SAME path, or the renderer's
        // `probe.snapshot().get(&node.path)` lookup misses and the inspector's
        // AutoUI/source data stays empty for loop bodies.
        //
        // Before Fix A the builder recorded at `[i, 0]` (it pushed the
        // body-index unconditionally) — this test would fail with len==2.
        let widget = make_test_widget(
            "List",
            vec![AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "items",
                Value::Array(auto_val::Array::from(vec![
                    Value::str("apple"),
                    Value::str("banana"),
                ])),
            )
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "List");

        // for item in .items { text("${.item}") }  — body.len() == 1
        let node = AuraNode::ForLoop {
            var: "item".to_string(),
            index: None,
            iterable: ".items".to_string(),
            key_expr: None,
            body: vec![AuraNode::Text(AuraTextContent::Interpolated {
                template: "${.item}".to_string(),
                bindings: vec!["item".to_string()],
            })],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        let mut path_keys: Vec<Vec<u16>> = snap.keys().cloned().collect();
        path_keys.sort();
        // one entry per iteration (each combines for_context + state binding),
        // each at a single-segment path
        assert_eq!(
            path_keys,
            vec![vec![0u16], vec![1u16]],
            "Fix A: single-body loop body paths are [i], not [i, 0]"
        );
        // each entry carries both the for-context and the state binding
        for k in &path_keys {
            let entry = snap.get(k).unwrap();
            assert!(
                entry.for_context.is_some(),
                "for_context present at {:?}",
                k
            );
            assert_eq!(
                entry.state_bindings.len(),
                1,
                "state binding present at {:?}",
                k
            );
        }
    }

    #[test]
    fn build_with_debug_for_loop_multi_body_path_keeps_body_index() {
        // Plan 309 Phase 1 (Fix A) regression guard: a multi-node body is
        // wrapped in a Column per iteration, so the body-index level IS
        // present in the VTree — the builder must STILL push it (`[i, bi]`).
        let widget = make_test_widget(
            "List",
            vec![AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "items",
                Value::Array(auto_val::Array::from(vec![Value::str("x")])),
            )
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "List");

        // for item in .items { text("${.item}"); text("tail") }  — body.len() == 2
        let node = AuraNode::ForLoop {
            var: "item".to_string(),
            index: None,
            iterable: ".items".to_string(),
            key_expr: None,
            body: vec![
                AuraNode::Text(AuraTextContent::Interpolated {
                    template: "${.item}".to_string(),
                    bindings: vec!["item".to_string()],
                }),
                AuraNode::Text(AuraTextContent::Literal("tail".to_string())),
            ],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        // first body node at [0, 0]; literal "tail" produces no probe entry.
        // The interpolated node keeps the two-segment path (body-index present).
        assert!(
            snap.contains_key(&vec![0u16, 0u16]),
            "multi-body loop keeps body-index level: key [0,0] expected"
        );
        assert!(
            !snap.contains_key(&vec![0u16]),
            "multi-body loop must NOT collapse to single-segment [0]"
        );
    }

    // ========================================================================
    // Plan 323 — `for` inside `grid` must flatten to one cell per iteration
    // (both the non-tracked `build` path and the tracked DevTools path).
    // ========================================================================

    /// Shared grid-with-for-loop node: a `grid` whose only child is a `for`
    /// over a 7-element `items` array, body = single text per iteration.
    fn grid_with_for_loop_node() -> AuraNode {
        let for_loop = AuraNode::ForLoop {
            var: "item".to_string(),
            index: None,
            iterable: ".items".to_string(),
            key_expr: None,
            body: vec![AuraNode::Text(AuraTextContent::Interpolated {
                template: "${.item}".to_string(),
                bindings: vec!["item".to_string()],
            })],
            span: None,
            debug_id: None,
        };
        AuraNode::element("grid")
            .with_prop("cols", Expr::Int(7))
            .with_child(for_loop)
    }

    fn widget_with_items() -> (AuraWidget, VmBridge) {
        let widget = make_test_widget(
            "Grid",
            vec![AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "items",
                Value::Array(auto_val::Array::from(vec![
                    Value::str("a"),
                    Value::str("b"),
                    Value::str("c"),
                    Value::str("d"),
                    Value::str("e"),
                    Value::str("f"),
                    Value::str("g"),
                ])),
            )
            .unwrap();
        (widget, bridge)
    }

    #[test]
    fn convert_grid_flattens_for_loop_into_cells() {
        // Non-tracked `build` path (used by into_iced / codegen). Before the
        // Plan 323 fix the `for` returned a single wrapping Column, so the grid
        // saw ONE cell instead of 7 → a "calendar" rendered as a single tall
        // column. This test pins the flattened behaviour.
        let (widget, bridge) = widget_with_items();
        let _ = widget;
        let builder = AuraViewBuilder::new(&bridge, "Grid");

        let view = builder.build(&grid_with_for_loop_node());
        match view {
            View::Grid { cols, cells, .. } => {
                assert_eq!(cols, 7);
                assert_eq!(
                    cells.len(),
                    7,
                    "for inside grid must flatten to one cell per iteration"
                );
            }
            other => panic!(
                "Expected View::Grid with 7 cells, got {:?} (kind)",
                std::mem::discriminant(&other)
            ),
        }
    }

    // ========================================================================
    // Plan 412 F1 — style 布局类重派生(grid/flex-col/flex 覆盖元素语义)
    // ========================================================================

    fn styled_node(tag: &str, style: &str, child_count: usize) -> AuraNode {
        let mut node =
            AuraNode::element(tag).with_prop("style", Expr::Str(style.to_string().into()));
        for i in 0..child_count {
            node = node.with_child(AuraNode::Text(AuraTextContent::Literal(format!(
                "{}",
                i + 1
            ))));
        }
        node
    }

    fn build_plain(node: &AuraNode) -> View<DynamicMessage> {
        let widget = make_test_widget("T", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "T");
        builder.build(node)
    }

    #[test]
    fn plan412_container_grid_classes_rederive_to_real_grid() {
        // /grid 存量不一致的根因修复:div + "grid grid-cols-3 gap-4"(CSS grid
        // 写法)在 VM 必须获得真实网格,gap-4 = 4 units × 4px = 16px。
        let view = build_plain(&styled_node("div", "grid grid-cols-3 gap-4", 6));
        match view {
            View::Grid {
                cols,
                gap,
                cells,
                style,
            } => {
                assert_eq!(cols, 3);
                assert_eq!(gap, 16, "gap-4 → 16px");
                assert_eq!(cells.len(), 6);
                assert!(style.is_some(), "style 类保留在结果 View 上");
            }
            other => panic!(
                "Expected View::Grid, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn plan412_responsive_grid_cols_participate() {
        // 响应式前缀剥离(Plan 411 P0-B)与重派生叠加:md:grid-cols-2 生效。
        let view = build_plain(&styled_node(
            "div",
            "grid grid-cols-1 md:grid-cols-2 gap-4",
            4,
        ));
        match view {
            View::Grid { cols, .. } => assert_eq!(cols, 2, "md: 剥离后 grid-cols-2 参与派生"),
            other => panic!(
                "Expected View::Grid, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn plan412_container_flex_classes_rederive() {
        // flex-col → Column;flex → Row(无子节点差异,只看 View 类型)
        match build_plain(&styled_node("div", "flex flex-col gap-2", 2)) {
            View::Column {
                children, style, ..
            } => {
                assert_eq!(children.len(), 2);
                assert!(style.is_some());
            }
            other => panic!(
                "flex flex-col → Column, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
        match build_plain(&styled_node("div", "flex items-center gap-2", 2)) {
            View::Row { children, .. } => assert_eq!(children.len(), 2),
            other => panic!("flex → Row, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn plan412_cross_class_arbitration_on_semantic_elements() {
        // col + flex 类 → Row;row + flex-col → Column(显式类覆盖元素语义)
        match build_plain(&styled_node("col", "flex w-full", 2)) {
            View::Row { .. } => {}
            other => panic!("col+flex → Row, got {:?}", std::mem::discriminant(&other)),
        }
        match build_plain(&styled_node("row", "flex-col", 2)) {
            View::Column { .. } => {}
            other => panic!(
                "row+flex-col → Column, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
        // col + grid 类 → Grid(/grid 页写法)
        match build_plain(&styled_node(
            "col",
            "grid grid-cols-3 gap-4 w-full max-w-lg",
            6,
        )) {
            View::Grid { cols, gap, .. } => {
                assert_eq!((cols, gap), (3, 16));
            }
            other => panic!("col+grid → Grid, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn plan412_grid_element_class_cols_override_prop() {
        // 语义 grid 元素:class 里的 grid-cols-N/gap-N 优先于 prop(同 CSS)。
        let mut node = AuraNode::element("grid")
            .with_prop("cols", Expr::Int(4))
            .with_prop("gap", Expr::Int(8))
            .with_prop("style", Expr::Str("grid-cols-2 gap-4".to_string().into()));
        node = node.with_child(AuraNode::Text(AuraTextContent::Literal("x".to_string())));
        match build_plain(&node) {
            View::Grid { cols, gap, .. } => {
                assert_eq!(cols, 2, "grid-cols-2 类覆盖 cols prop");
                assert_eq!(gap, 16, "gap-4 类(16px)覆盖 gap prop(8px)");
            }
            other => panic!(
                "Expected View::Grid, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn plan412_container_without_layout_classes_stays_container() {
        // 无布局类 → 原 Container 路径(回归保护)
        match build_plain(&styled_node("div", "p-4 bg-card rounded-lg border", 1)) {
            View::Container { .. } => {}
            other => panic!(
                "plain div → Container, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn convert_grid_tracked_flattens_for_loop_into_cells() {
        // Tracked `build_with_debug` path (used by VM `render_dynamic_view` for
        // EVERY frame, even with F12 off). Same flattening requirement, plus the
        // per-cell probe paths must be the sequential cell indices [0..7] so
        // they match the render-time Grid-arm visit order.
        let (widget, bridge) = widget_with_items();
        let _ = widget;
        let builder = AuraViewBuilder::new(&bridge, "Grid");

        let (view, _id_map, probe) = builder.build_with_debug(&grid_with_for_loop_node());
        match view {
            View::Grid { cells, .. } => {
                assert_eq!(
                    cells.len(),
                    7,
                    "tracked for-in-grid must also flatten to 7 cells"
                );
            }
            other => panic!(
                "Expected tracked View::Grid with 7 cells, got {:?} (kind)",
                std::mem::discriminant(&other)
            ),
        }
        let snap = probe.snapshot();
        let mut keys: Vec<Vec<u16>> = snap.keys().cloned().collect();
        keys.sort();
        // Each iteration's for-context recorded at the flat cell path [0..7].
        assert_eq!(
            keys,
            vec![
                vec![0u16],
                vec![1],
                vec![2],
                vec![3],
                vec![4],
                vec![5],
                vec![6]
            ],
            "tracked grid cell paths must be flat sequential [0..7]"
        );
        for k in &keys {
            assert!(
                snap.get(k).unwrap().for_context.is_some(),
                "for_context present at flat cell path {:?}",
                k
            );
        }
    }

    #[test]
    fn convert_grid_for_loop_obj_cells_resolve_field_access() {
        // Plan 323 Phase 2: the calendar's real pattern — a `for` over an array
        // of Obj cells, each cell rendered as a `button cell.label`. This tests
        // that (a) the loop flattens to N cells AND (b) the FieldAccess label
        // `cell.label` actually resolves to each Obj's "label" string. If this
        // fails, an empty day grid is explained by the view path (hypothesis B),
        // not just the grid-flattening.
        let widget = make_test_widget(
            "Cal",
            vec![AuraStateDef {
                name: "days".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();

        fn cell(label: &str, date: &str, other: bool) -> Value {
            let mut o = auto_val::Obj::new();
            o.set("label", Value::str(label));
            o.set("date", Value::str(date));
            o.set("is_other_month", Value::Bool(other));
            Value::Obj(Box::new(o))
        }
        bridge
            .write_state(
                "days",
                Value::Array(auto_val::Array::from(vec![
                    cell("31", "2026-05-31", true),
                    cell("1", "2026-06-01", false),
                    cell("2", "2026-06-02", false),
                ])),
            )
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Cal");

        // grid { for cell in .days { button cell.label { ... } } }
        let for_loop = AuraNode::ForLoop {
            var: "cell".to_string(),
            index: None,
            iterable: ".days".to_string(),
            key_expr: None,
            body: vec![AuraNode::Element {
                tag: "button".to_string(),
                props: HashMap::from([(
                    "label".to_string(),
                    AuraPropValue::Expr(Expr::Dot(
                        Box::new(Expr::Ident(".cell".into())),
                        "label".into(),
                    )),
                )]),
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }],
            span: None,
            debug_id: None,
        };
        let grid = AuraNode::element("grid")
            .with_prop("cols", Expr::Int(7))
            .with_child(for_loop);

        let view = builder.build(&grid);
        match view {
            View::Grid { cells, .. } => {
                assert_eq!(cells.len(), 3, "for over 3 Obj cells → 3 grid cells");
                let labels: Vec<String> = cells
                    .iter()
                    .map(|c| match c {
                        View::Button { label, .. } => label.clone(),
                        _ => "(not button)".to_string(),
                    })
                    .collect();
                assert_eq!(
                    labels,
                    vec!["31", "1", "2"],
                    "button cell.label must resolve each Obj's label field"
                );
            }
            other => panic!(
                "Expected View::Grid, got discriminant {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn plan606_for_loop_image_resolves_bindings_and_fit() {
        let widget = make_test_widget(
            "Gallery",
            vec![AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();

        fn item(id: i32, thumb: &str, title: &str) -> Value {
            let mut o = auto_val::Obj::new();
            o.set("id", Value::Int(id));
            o.set("thumb", Value::str(thumb));
            o.set("title", Value::str(title));
            Value::Obj(Box::new(o))
        }
        bridge
            .write_state(
                "items",
                Value::Array(auto_val::Array::from(vec![
                    item(1, "https://example.com/1.jpg", "Mountain"),
                    item(2, "data:image/svg+xml;utf8,<svg></svg>", "Bridge"),
                ])),
            )
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Gallery");

        // grid { for it in .items { image (src: it.thumb, fit: "cover") } }
        let for_loop = AuraNode::ForLoop {
            var: "it".to_string(),
            index: None,
            iterable: ".items".to_string(),
            key_expr: None,
            body: vec![AuraNode::Element {
                tag: "image".to_string(),
                props: HashMap::from([
                    (
                        "src".to_string(),
                        AuraPropValue::Expr(Expr::Dot(
                            Box::new(Expr::Ident(".it".into())),
                            "thumb".into(),
                        )),
                    ),
                    (
                        "fit".to_string(),
                        AuraPropValue::Expr(Expr::Str("cover".into())),
                    ),
                ]),
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }],
            span: None,
            debug_id: None,
        };
        let grid = AuraNode::element("grid")
            .with_prop("cols", Expr::Int(2))
            .with_child(for_loop);

        let view = builder.build(&grid);
        match view {
            View::Grid { cells, .. } => {
                assert_eq!(cells.len(), 2, "for over 2 items -> 2 cells");
                match &cells[0] {
                    View::Image { src, style } => {
                        assert_eq!(src, "https://example.com/1.jpg");
                        let s = style.as_ref().expect("fit: cover -> style");
                        assert!(
                            s.classes.iter().any(|c| matches!(
                                c,
                                StyleClass::ObjectFit(crate::ui::style::ObjectFit::Cover)
                            )),
                            "style should have ObjectFit::Cover"
                        );
                    }
                    other => panic!("Expected View::Image, got {:?}", other),
                }
                match &cells[1] {
                    View::Image { src, .. } => {
                        assert_eq!(src, "data:image/svg+xml;utf8,<svg></svg>");
                    }
                    other => panic!("Expected View::Image, got {:?}", other),
                }
            }
            other => panic!("Expected View::Grid, got {:?}", other),
        }
    }

    // ========================================================================
    // Plan 307 Task 11 — event handler binding capture
    // ========================================================================

    #[test]
    fn build_with_debug_captures_event_handler() {
        use auto_ui_compiler::aura::AuraEvent;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "button".to_string(),
            props: HashMap::from([(
                "label".to_string(),
                AuraPropValue::Expr(Expr::Str("Inc".into())),
            )]),
            events: HashMap::from([(
                "onclick".to_string(),
                AuraEvent {
                    handler: ".Inc".to_string(),
                    params: vec![],
                },
            )]),
            children: vec![],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        let all_events: Vec<_> = snap.values().flat_map(|e| e.events.iter()).collect();
        assert_eq!(all_events.len(), 1);
        assert_eq!(all_events[0].event, "onclick");
        assert_eq!(all_events[0].handler, ".Inc");
    }

    #[test]
    fn build_with_debug_captures_event_handlers_distinct_paths() {
        use auto_ui_compiler::aura::AuraEvent;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let button = |handler: &str| AuraNode::Element {
            tag: "button".to_string(),
            props: HashMap::new(),
            events: HashMap::from([(
                "onclick".to_string(),
                AuraEvent {
                    handler: handler.to_string(),
                    params: vec![],
                },
            )]),
            children: vec![],
            span: None,
            debug_id: None,
        };

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![button(".Inc"), button(".Dec")],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        // Each button is captured at its own child path with one event each.
        let event_paths: Vec<(Vec<u16>, &str)> = snap
            .iter()
            .flat_map(|(path, e)| {
                e.events
                    .iter()
                    .map(move |ev| (path.clone(), ev.handler.as_str()))
            })
            .collect();
        assert_eq!(event_paths.len(), 2, "two events captured");
        let handlers: Vec<&str> = {
            let mut h = event_paths.iter().map(|(_, h)| *h).collect::<Vec<_>>();
            h.sort();
            h
        };
        assert_eq!(handlers, vec![".Dec", ".Inc"]);
        // distinct paths
        assert_ne!(event_paths[0].0, event_paths[1].0, "distinct child paths");
    }

    #[test]
    fn test_build_column_with_children() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::from([
                ("spacing".to_string(), AuraPropValue::Expr(Expr::Int(10))),
                ("padding".to_string(), AuraPropValue::Expr(Expr::Int(5))),
            ]),
            events: HashMap::new(),
            children: vec![AuraNode::text("Child 1"), AuraNode::text("Child 2")],
            span: None,
            debug_id: None,
        };
        let view = builder.build(&node);

        match view {
            View::Column {
                spacing,
                padding,
                children,
                ..
            } => {
                assert_eq!(spacing, 10);
                assert_eq!(padding, 5);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_build_row() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "row".to_string(),
            props: HashMap::from([("spacing".to_string(), AuraPropValue::Expr(Expr::Int(8)))]),
            events: HashMap::new(),
            children: vec![AuraNode::text("A"), AuraNode::text("B")],
            span: None,
            debug_id: None,
        };
        let view = builder.build(&node);

        match view {
            View::Row {
                spacing, children, ..
            } => {
                assert_eq!(spacing, 8);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Row"),
        }
    }

    #[test]
    fn test_build_button_with_event() {
        let widget = make_test_widget("Counter", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            span: None,
            debug_id: None,
            tag: "button".to_string(),
            props: HashMap::from([(
                "text".to_string(),
                AuraPropValue::Expr(Expr::Str("Increment".into())),
            )]),
            events: HashMap::from([(
                "onclick".to_string(),
                AuraEvent {
                    handler: ".Inc".to_string(),
                    params: vec![],
                },
            )]),
            children: vec![],
        };
        let view = builder.build(&node);

        match view {
            View::Button { label, onclick, .. } => {
                assert_eq!(label, "Increment");
                match onclick {
                    DynamicMessage::Typed {
                        widget_name,
                        event_name,
                        args,
                    } => {
                        assert_eq!(widget_name, "Counter");
                        assert_eq!(event_name, "Inc");
                        assert!(args.is_empty());
                    }
                    _ => panic!("Expected DynamicMessage::Typed"),
                }
            }
            _ => panic!("Expected View::Button"),
        }
    }

    /// PLAN-002 B：布局件 `oncontextmenu` 提取——VM 轨此前只有 button/
    /// mouse-area 两个右键挂点（526 Q6/Q9① 债），Vue 轨元素级 @contextmenu
    /// 早已泛映射（vue.rs:14470）。本测试锁提取面：col 的 oncontextmenu
    /// 落 View::Column::on_right_click，onclick 缺席时不误填。
    #[test]
    fn test_layout_oncontextmenu_becomes_right_click() {
        let widget = make_test_widget("Desktop", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Desktop");

        let node = AuraNode::Element {
            span: None,
            debug_id: None,
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::from([(
                "oncontextmenu".to_string(),
                AuraEvent {
                    handler: ".BlankMenu".to_string(),
                    params: vec![],
                },
            )]),
            children: vec![],
        };
        let view = builder.build(&node);

        match view {
            View::Column {
                onclick,
                on_right_click,
                ..
            } => {
                assert!(onclick.is_none(), "onclick 缺席不应被 oncontextmenu 误填");
                match on_right_click {
                    Some(DynamicMessage::Typed {
                        widget_name,
                        event_name,
                        ..
                    }) => {
                        assert_eq!(widget_name, "Desktop");
                        assert_eq!(event_name, "BlankMenu");
                    }
                    other => panic!("oncontextmenu 未落 on_right_click: {other:?}"),
                }
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_build_nested_layout() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            span: None,
            debug_id: None,
            children: vec![
                AuraNode::Element {
                    tag: "row".to_string(),
                    props: HashMap::new(),
                    events: HashMap::new(),
                    span: None,
                    debug_id: None,
                    children: vec![AuraNode::text("Left"), AuraNode::text("Right")],
                },
                AuraNode::text("Bottom"),
            ],
        };
        let view = builder.build(&node);

        match view {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 2);
                match &children[0] {
                    View::Row {
                        children: row_children,
                        ..
                    } => {
                        assert_eq!(row_children.len(), 2);
                    }
                    _ => panic!("Expected View::Row as first child"),
                }
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_build_unknown_tag_fallback() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "custom_widget".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![AuraNode::text("Content")],
            span: None,
            debug_id: None,
        };
        let view = builder.build(&node);

        // Should render the child directly as fallback
        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Content");
            }
            _ => panic!("Expected View::Text (single child fallback)"),
        }
    }

    /// Plan 442 A4: svg 元素子树 → 序列化 SVG 文档经 View::Image 下发
    /// (此前落 unknown fallback → View::Empty,VM 轨 svg 完全不渲染)。
    #[test]
    fn test_build_svg_element_serializes_document() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let mut svg_props = HashMap::new();
        svg_props.insert(
            "viewBox".to_string(),
            AuraPropValue::Expr(Expr::Str("0 0 24 24".into())),
        );
        svg_props.insert(
            "class".to_string(),
            AuraPropValue::Expr(Expr::Str("w-6 h-6".into())),
        );
        let mut path_props = HashMap::new();
        path_props.insert(
            "d".to_string(),
            AuraPropValue::Expr(Expr::Str("M4 4h16v16h-16z".into())),
        );
        path_props.insert(
            "fill".to_string(),
            AuraPropValue::Expr(Expr::Str("currentColor".into())),
        );
        let node = AuraNode::Element {
            tag: "svg".to_string(),
            props: svg_props,
            events: HashMap::new(),
            children: vec![AuraNode::Element {
                tag: "path".to_string(),
                props: path_props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }],
            span: None,
            debug_id: None,
        };
        let view = builder.build(&node);

        match view {
            View::Image { src, style } => {
                assert!(src.starts_with("svgdoc:"), "svgdoc src: {src}");
                let doc = &src["svgdoc:".len()..];
                assert!(
                    doc.contains(r#"viewBox="0 0 24 24""#),
                    "viewBox attr: {doc}"
                );
                assert!(doc.contains(r#"d="M4 4h16v16h-16z""#), "path d attr: {doc}");
                assert!(doc.contains(r#"fill="currentColor""#), "fill attr: {doc}");
                assert!(
                    !doc.contains("w-6"),
                    "class 不进文档(由 View::Image style 承载): {doc}"
                );
                let style = style.expect("class prop → View::Image style");
                assert!(
                    style
                        .classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Width(_))),
                    "w-6 parsed: {:?}",
                    style.classes
                );
            }
            _ => panic!("Expected View::Image for svg, got {view:?}"),
        }
    }

    /// Plan 547 Task 21: ImageSurface props/events stay typed on the VM path
    /// and retain the path-derived vnode identity used by snapshots.
    #[test]
    fn image_surface_vm_builder() {
        let widget = make_test_widget("Viewer", vec![]);
        let bridge = VmBridge::new(&widget).expect("bridge");
        let builder = AuraViewBuilder::new(&bridge, "Viewer");
        let node = AuraNode::element("ImageSurface")
            .with_prop("src", Expr::Ident(".asset_src".into()))
            .with_prop("alt", Expr::Str("preview".into()))
            .with_prop("width", Expr::Int(640))
            .with_prop("height", Expr::Int(480))
            .with_prop("quality", Expr::Int(75))
            .with_prop("fit", Expr::Str("width".into()))
            .with_prop("zoom", Expr::Float(2.0, "2.0".into()))
            .with_prop("offset_x", Expr::Float(12.5, "12.5".into()))
            .with_prop("offset_y", Expr::Float(-4.0, "-4.0".into()))
            .with_prop("rotation", Expr::Int(90))
            .with_prop("filter", Expr::Str("linear".into()))
            .with_event("onload", ".ImageLoaded")
            .with_event("onerror", ".ImageFailed")
            .with_event("onwheel", ".ZoomAt")
            .with_event("onpan", ".PanBy")
            .with_event("ondblclick", ".ToggleOneToOne");

        let (view, id_map, _probe) = builder.build_with_debug(&node);
        let View::ImageSurface {
            src,
            alt,
            width,
            height,
            quality,
            fit,
            zoom,
            offset_x,
            offset_y,
            rotation,
            filter,
            on_error: Some(error),
            on_loaded: Some(loaded),
            on_wheel: Some(wheel),
            on_pan: Some(pan),
            on_double_click: Some(double_click),
            ..
        } = view
        else {
            panic!("expected ImageSurface view")
        };
        assert_eq!(src, "${asset_src}"); // unresolved state refs remain display-safe text
        assert_eq!(alt, "preview");
        assert_eq!((width, height, quality), (640, 480, 75));
        assert_eq!(fit, "width");
        assert_eq!((zoom, offset_x, offset_y, rotation), (2.0, 12.5, -4.0, 90));
        assert_eq!(filter, "linear");
        let event_name = |message: DynamicMessage| match message {
            DynamicMessage::Typed {
                widget_name,
                event_name,
                args,
            } => {
                assert_eq!(widget_name, "Viewer");
                assert!(args.is_empty());
                event_name
            }
            other => panic!("expected typed message, got {other:?}"),
        };
        assert_eq!(event_name(error), "ImageFailed");
        assert_eq!(event_name(loaded), "ImageLoaded");
        assert_eq!(event_name(wheel), "ZoomAt");
        assert_eq!(event_name(pan), "PanBy");
        assert_eq!(event_name(double_click), "ToggleOneToOne");

        // The tracked path for the root is empty and therefore always maps to
        // the same id across rebuilds; this is the identity used by VM events.
        assert_eq!(id_map.get(&Vec::<usize>::new()), None);
        assert_eq!(
            crate::ui::vnode::id_from_path(&[]),
            crate::ui::vnode::id_from_path(&[]),
            "path-derived vnode id must be stable"
        );
    }

    #[test]
    fn test_extract_handler_name() {
        assert_eq!(extract_handler_name(".Inc"), "Inc");
        assert_eq!(extract_handler_name("Msg::Inc"), "Inc");
        assert_eq!(extract_handler_name("Inc"), "Inc");
        assert_eq!(extract_handler_name(".AddItem"), "AddItem");
    }

    #[test]
    fn test_state_binding_in_text_element() {
        let widget = make_test_widget(
            "Counter",
            vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(7),
                decorators: vec![],
            }],
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            tag: "text".to_string(),
            props: HashMap::from([(
                "text".to_string(),
                AuraPropValue::Expr(Expr::Ident(".count".into())),
            )]),
            events: HashMap::new(),
            span: None,
            debug_id: None,
            children: vec![],
        };
        let view = builder.build(&node);

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "7");
            }
            _ => panic!("Expected View::Text with state-resolved value"),
        }
    }

    #[test]
    fn test_button_msg_variant_handler() {
        let widget = make_test_widget("Counter", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            tag: "button".to_string(),
            props: HashMap::from([(
                "label".to_string(),
                AuraPropValue::Expr(Expr::Str("Reset".into())),
            )]),
            events: HashMap::from([(
                "onclick".to_string(),
                AuraEvent {
                    handler: "Msg::Reset".to_string(),
                    params: vec![],
                },
            )]),
            span: None,
            debug_id: None,
            children: vec![],
        };
        let view = builder.build(&node);

        match view {
            View::Button { label, onclick, .. } => {
                assert_eq!(label, "Reset");
                match onclick {
                    DynamicMessage::Typed { event_name, .. } => {
                        assert_eq!(event_name, "Reset");
                    }
                    _ => panic!("Expected DynamicMessage::Typed"),
                }
            }
            _ => panic!("Expected View::Button"),
        }
    }

    #[test]
    fn test_value_to_display_string() {
        assert_eq!(value_to_display_string(&Value::Int(42)), "42");
        assert_eq!(value_to_display_string(&Value::Bool(true)), "true");
        assert_eq!(value_to_display_string(&Value::str("hello")), "hello");
        assert_eq!(value_to_display_string(&Value::Nil), "");
    }

    #[test]
    fn test_eval_condition_with_bindings() {
        // Simulate a todo item as a binding
        let mut bindings = Bindings::new();
        let mut todo_obj = auto_val::Obj::new();
        todo_obj.set("id", Value::Int(0));
        todo_obj.set("text", Value::str("Buy milk"));
        todo_obj.set("done", Value::Bool(false));
        bindings.insert("todo".to_string(), Value::Obj(Box::new(todo_obj)));

        // Set up state: filter = "active"
        let widget = make_test_widget(
            "App",
            vec![
                AuraStateDef {
                    name: "filter".to_string(),
                    type_info: Type::StrOwned,
                    initial: Expr::Str("active".into()),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "todos".to_string(),
                    type_info: Type::StrOwned,
                    initial: Expr::Str("[]".into()),
                    decorators: vec![],
                },
            ],
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");

        // Test: .filter == "active" && todo.done == false → should be true
        let cond1 = ".filter == \"active\" && todo.done == false";
        let r1 = builder.eval_condition_with(cond1, &bindings);
        eprintln!("cond1='{}' result={}", cond1, r1);
        assert!(
            r1,
            "Expected true for active filter with done=false, got false"
        );

        // Test: .filter == "all" → should be true when filter is "active"? No, false
        let cond2 = ".filter == \"all\"";
        let r2 = builder.eval_condition_with(cond2, &bindings);
        eprintln!("cond2='{}' result={}", cond2, r2);
        assert!(
            !r2,
            "Expected false for 'all' filter when filter is 'active'"
        );

        // Test the full compound condition: .filter == "all" || ( .filter == "active" && todo.done == false )
        let cond_full = ".filter == \"all\" || ( .filter == \"active\" && todo.done == false )";
        let r_full = builder.eval_condition_with(cond_full, &bindings);
        eprintln!("cond_full='{}' result={}", cond_full, r_full);

        // Also test the inner part directly
        let inner = ".filter == \"active\" && todo.done == false";
        eprintln!(
            "inner='{}' result={}",
            inner,
            builder.eval_condition_with(inner, &bindings)
        );

        // Test right side of || directly
        let right = "( .filter == \"active\" && todo.done == false )";
        eprintln!(
            "right='{}' result={}",
            right,
            builder.eval_condition_with(right, &bindings)
        );

        assert!(
            r_full,
            "Expected true for full condition with active filter + undone todo"
        );

        // Test with done=true AND filter="completed"
        let mut todo_done = auto_val::Obj::new();
        todo_done.set("id", Value::Int(0));
        todo_done.set("text", Value::str("Done item"));
        todo_done.set("done", Value::Bool(true));
        let mut bindings_done = Bindings::new();
        bindings_done.insert("todo".to_string(), Value::Obj(Box::new(todo_done)));

        // Create a builder with filter="completed" state
        let widget_completed = make_test_widget(
            "App",
            vec![AuraStateDef {
                name: "filter".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("completed".into()),
                decorators: vec![],
            }],
        );
        let bridge_completed = VmBridge::new(&widget_completed).unwrap();
        let builder_completed = AuraViewBuilder::new(&bridge_completed, "App");

        let cond_completed = "( .filter == \"completed\" && todo.done == true )";
        assert!(
            builder_completed.eval_condition_with(cond_completed, &bindings_done) || {
                // Also try without parens (parser may produce either)
                let cond2 = ".filter == \"completed\" && todo.done == true";
                builder_completed.eval_condition_with(cond2, &bindings_done)
            },
            "Expected true for completed filter + done todo"
        );

        // Active filter should NOT match done item
        let cond_active_done = ".filter == \"active\" && todo.done == false";
        assert!(
            !builder.eval_condition_with(cond_active_done, &bindings_done),
            "Expected false for active filter + done=true todo"
        );

        // Test editing_id conditions (the "double input" bug)
        // When editing_id=-1 and todo.id=0, editing_id != todo.id
        // Need a builder with editing_id state
        let widget_edit = make_test_widget(
            "App",
            vec![AuraStateDef {
                name: "editing_id".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("-1".into()),
                decorators: vec![],
            }],
        );
        let bridge_edit = VmBridge::new(&widget_edit).unwrap();
        let builder_edit = AuraViewBuilder::new(&bridge_edit, "App");

        let cond_edit_eq = ".editing_id == todo.id";
        let r_eq = builder_edit.eval_condition_with(cond_edit_eq, &bindings);
        eprintln!(
            "editing_id==-1, todo.id=0: '.editing_id == todo.id' => {}",
            r_eq
        );
        assert!(!r_eq, "editing_id=-1 should NOT equal todo.id=0");

        let cond_edit_neq = ".editing_id != todo.id";
        let r_neq = builder_edit.eval_condition_with(cond_edit_neq, &bindings);
        eprintln!(
            "editing_id==-1, todo.id=0: '.editing_id != todo.id' => {}",
            r_neq
        );
        assert!(r_neq, "editing_id=-1 should NOT equal todo.id=0 (neq)");
    }

    /// PLAN-048 L1 (musk auth gate): `if store.authenticated != true` must
    /// resolve store COMPUTEDS. Previously eval_condition_with resolved the
    /// LHS via read_state only — computeds are not materialized in state —
    /// so the condition was permanently false and the main UI rendered with
    /// token=nil (047 R9 登录墙缺位根因).
    #[test]
    fn test_plan048_store_computed_condition_auth_gate() {
        let widget = make_test_widget(
            "App",
            vec![AuraStateDef {
                name: "token".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        let computed = vec![auto_ui_compiler::aura::AuraComputed {
            name: "authenticated".to_string(),
            expr: Expr::Bina(
                Box::new(Expr::Dot(
                    Box::new(Expr::Ident(".".into())),
                    "token".to_string().into(),
                )),
                Op::Neq,
                Box::new(Expr::None),
            ),
        }];

        // token 恢复为 nil(未登录) → authenticated=false → `!= true` 为真 → 登录分支
        bridge.write_state("token", Value::Nil).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App").with_computed(&computed);
        assert!(
            builder.eval_condition_with("store.authenticated != true", &Bindings::new()),
            "token=nil: gate must be TRUE (render LoginPage)"
        );

        // token 恢复(JWT) → authenticated=true → gate false → 主 UI 分支
        bridge
            .write_state("token", Value::Str("jwt-abc".into()))
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App").with_computed(&computed);
        assert!(
            !builder.eval_condition_with("store.authenticated != true", &Bindings::new()),
            "token set: gate must be FALSE (render main UI)"
        );

        // 无操作符裸真值形态 `store.authenticated` 同样必须接 computed
        bridge.write_state("token", Value::Nil).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App").with_computed(&computed);
        assert!(
            !builder.eval_condition_with("store.authenticated", &Bindings::new()),
            "token=nil: bare truthy check must be FALSE"
        );
    }

    /// PLAN-077 (auto-musk ReportCard/ToolGateCard): computed 体内的 `??`
    /// (NullCoalesce) 此前落到 `_ => None`，整个 computed 求值为空 → 文本位
    /// 渲染裸 "${name}" 占位。期望：左值非 Nil 取左值；Nil/缺字段落右值。
    #[test]
    fn test_plan077_computed_null_coalesce() {
        let widget = make_test_widget(
            "App",
            vec![AuraStateDef {
                name: "runId".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        let computed = vec![auto_ui_compiler::aura::AuraComputed {
            name: "label".to_string(),
            expr: Expr::NullCoalesce(
                Box::new(Expr::Dot(
                    Box::new(Expr::Ident(".".into())),
                    "runId".to_string().into(),
                )),
                Box::new(Expr::Str("—".into())),
            ),
        }];

        // 有值：取左值
        bridge
            .write_state("runId", Value::Str("run-fixture".into()))
            .unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App").with_computed(&computed);
        assert_eq!(
            builder.eval_computed("label", &Bindings::new()),
            Some(Value::Str("run-fixture".into())),
            "non-Nil left side must win"
        );

        // Nil：落右值默认
        bridge.write_state("runId", Value::Nil).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App").with_computed(&computed);
        assert_eq!(
            builder.eval_computed("label", &Bindings::new()),
            Some(Value::Str("—".into())),
            "Nil left side must fall through to the default"
        );
    }

    /// PLAN-050 T9 (C7): WorkspaceSelector 形态复现——子 widget computed
    /// `currentName => if .current != None { .current.name } else { "选择工作目录" }`
    /// + model `var current obj = None` + 父裸调用。musk VM 实测 rail 底部
    /// 渲染裸 "${currentName}"（vtree content 证据），computed 求值链在该
    /// 形态下断裂落 literal fallback。期望：else 分支值上屏、裸串不漏。
    #[test]
    fn plan050_child_computed_text_resolves_not_raw_placeholder() {
        use auto_lang_core::parser::Parser;

        let child_src = concat!(
            "widget Selector {\n",
            "    computed {\n",
            "        currentName => if .current != None { .current.name } else { \"选择工作目录\" }\n",
            "    }\n",
            "    model {\n",
            "        var current obj = None\n",
            "    }\n",
            "    view {\n",
            "        button {\n",
            "            span { text .currentName }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = Parser::from(child_src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let child =
            auto_ui_compiler::aura::extract::extract_widget_from_decl(decl).expect("extract");

        let mut parent = make_test_widget("Parent", vec![]);
        parent.view_tree = AuraNode::Component {
            name: "Selector".to_string(),
            props: vec![],
            events: HashMap::new(),
            children: vec![],
            span: None,
            debug_id: None,
        };

        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "Parent", &registry);

        let view = builder.build(&parent.view_tree);
        assert!(
            view_contains_text(&view, "选择工作目录"),
            "computed else 分支值必须上屏; got: {:?}",
            view
        );
        assert!(
            !view_contains_text(&view, "${currentName}"),
            "裸 placeholder 不得漏出"
        );
    }

    /// PLAN-051 T2 (C1): musk 形态 Enter 接线——textarea 声明
    /// `onkeydown.enter.exact.prevent` 此前收进 keydown 表的键是
    /// "enter.exact"（只剥 .prevent/.stop），iced key_binding 归一化键名
    /// "enter" 永不命中 → 真键盘 Enter 落默认行为（插换行）不派发。
    /// 期望：修饰段（exact/prevent/stop…）过滤后键名为 "enter"。
    #[test]
    fn plan051_textarea_keydown_enter_exact_normalized() {
        let widget = make_test_widget("W", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "W");

        let mut events = HashMap::new();
        events.insert(
            "onkeydown.enter.exact.prevent".to_string(),
            AuraEvent {
                handler: ".send".to_string(),
                params: vec!["this.text".to_string()],
            },
        );
        events.insert(
            "onkeydown.tab".to_string(),
            AuraEvent {
                handler: ".Tab".to_string(),
                params: vec![],
            },
        );
        let node = AuraNode::Element {
            tag: "textarea".to_string(),
            props: {
                let mut p = HashMap::new();
                p.insert(
                    "value".to_string(),
                    AuraPropValue::Expr(Expr::Str("hi".into())),
                );
                p
            },
            events,
            children: vec![],
            span: None,
            debug_id: None,
        };
        match builder.build(&node) {
            View::Textarea { keydown, .. } => {
                assert!(
                    keydown.contains_key("enter"),
                    "enter.exact.prevent 必须规整为键名 enter; got {:?}",
                    keydown.keys().collect::<Vec<_>>()
                );
                assert!(
                    keydown.contains_key("tab"),
                    "无修饰 onkeydown.tab 保持键名 tab"
                );
                assert!(!keydown.contains_key("enter.exact"), "带修饰残键不得存在");
            }
            _ => panic!("Expected View::Textarea"),
        }
    }

    /// PLAN-051 T2 (C1): keydown 实参烘焙——此前 keydown 收集走
    /// event_to_message（args 恒空），与按钮 onclick 的
    /// event_to_message_with（实参求值烘焙）不一致；.send(.text) 形态
    /// Enter 派发会丢文本。期望：keydown["enter"] 的 Typed args 携带
    /// 构建期求值的实参值。
    #[test]
    fn plan051_textarea_keydown_args_baked() {
        let widget = make_test_widget(
            "W",
            vec![AuraStateDef {
                name: "text".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("  typed draft ".into()),
                decorators: vec![],
            }],
        );
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "W");

        let mut events = HashMap::new();
        events.insert(
            "onkeydown.enter.exact.prevent".to_string(),
            AuraEvent {
                handler: ".send".to_string(),
                params: vec!["this.text".to_string()],
            },
        );
        let node = AuraNode::Element {
            tag: "textarea".to_string(),
            props: {
                let mut p = HashMap::new();
                p.insert(
                    "value".to_string(),
                    AuraPropValue::Expr(Expr::Str("  typed draft ".into())),
                );
                p
            },
            events,
            children: vec![],
            span: None,
            debug_id: None,
        };
        match builder.build(&node) {
            View::Textarea { keydown, .. } => {
                let msg = keydown.get("enter").expect("enter 键必须存在");
                match msg {
                    DynamicMessage::Typed { args, .. } => {
                        assert_eq!(args.len(), 1, "this.text 实参必须烘焙: {:?}", args);
                        assert_eq!(args[0], Value::Str("  typed draft ".into()));
                    }
                    _ => panic!("Expected Typed message"),
                }
            }
            _ => panic!("Expected View::Textarea"),
        }
    }

    /// PLAN-051 T2 (C1): 017-chat 形态回归护栏——单行 input 的
    /// `onenter: .DoSend` 必须继续接到 on_submit（Plan 053 机制），
    /// MCP 键盘 Enter 路由与真键盘 iced on_submit 都消费它。
    #[test]
    fn plan051_input_onenter_wires_on_submit() {
        let widget = make_test_widget("Composer", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Composer");

        let node = AuraNode::element("input")
            .with_prop("placeholder", Expr::Str("Type a message...".into()))
            .with_event("onenter", ".DoSend");
        match builder.build(&node) {
            View::Input { on_submit, .. } => {
                let msg = on_submit.expect("onenter 必须接 on_submit");
                match msg {
                    DynamicMessage::Typed {
                        widget_name,
                        event_name,
                        ..
                    } => {
                        assert_eq!(widget_name, "Composer");
                        assert_eq!(event_name, "DoSend");
                    }
                    _ => panic!("Expected Typed message"),
                }
            }
            _ => panic!("Expected View::Input"),
        }
    }

    /// PLAN-493 T1: mention 段计算——命中段 ("@词原文", "mention")，其余
    /// ("…", "text")；无 display 替换（编辑器内容即 value，跨端一致）。
    #[test]
    fn plan493_mention_segments_hit() {
        let names = vec!["assistant".to_string(), "Planner".to_string()];
        let segs = mention_segments("@assistant hello @Planner!", &names);
        assert_eq!(
            segs,
            vec![
                ("@assistant".to_string(), "mention".to_string()),
                (" hello ".to_string(), "text".to_string()),
                ("@Planner".to_string(), "mention".to_string()),
                ("!".to_string(), "text".to_string()),
            ]
        );
    }

    /// PLAN-493 T1: 裸 @（词空）按字面处理；命中大小写不敏感；部分词/多词
    /// 名单整词匹配（多词条目对 \w+ 扫描永不命中，与 musk render_mentions
    /// 的键语义一致）。
    #[test]
    fn plan493_mention_segments_bare_at_and_case() {
        let names = vec!["Assistant Agent".to_string(), "coder".to_string()];
        let segs = mention_segments("@ @CODER @cod", &names);
        assert_eq!(
            segs,
            vec![
                ("@ ".to_string(), "text".to_string()),
                ("@CODER".to_string(), "mention".to_string()),
                (" @cod".to_string(), "text".to_string()),
            ]
        );
    }

    /// PLAN-493 T1: 空名单 → 无着色，整体单段 text。
    #[test]
    fn plan493_mention_segments_empty_names() {
        let segs = mention_segments("hello @assistant", &[]);
        assert_eq!(
            segs,
            vec![("hello @assistant".to_string(), "text".to_string())]
        );
    }

    /// PLAN-493 T1: 覆盖不变式——段文本顺序拼接 == value（含 CJK/换行等
    /// 非 \w 边界字符，字节无损）。
    #[test]
    fn plan493_mention_segments_covers_value() {
        let names = vec!["assistant".to_string()];
        let value = "你好@assistant世界\nsecond @assistant line@";
        let segs = mention_segments(value, &names);
        let concat: String = segs.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(concat, value);
        assert_eq!(segs.iter().filter(|(_, k)| k == "mention").count(), 2);
    }

    /// PLAN-493 T3: mentions 能力接线——state-rooted 名单（var）+ value →
    /// View::Textarea.highlight 携带 mention 段，段文本拼回 == value。
    #[test]
    fn plan493_textarea_mentions_wired() {
        let widget = make_test_widget(
            "W",
            vec![
                AuraStateDef {
                    name: "text".to_string(),
                    type_info: Type::StrOwned,
                    initial: Expr::Str("@assistant hi".into()),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "mentionNames".to_string(),
                    type_info: Type::List(Box::new(Type::StrSlice)),
                    initial: Expr::Str(String::new().into()),
                    decorators: vec![],
                },
            ],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge.write_state(
            "mentionNames",
            Value::Array(auto_val::Array::from(vec![
                Value::Str("assistant".into()),
                Value::Str("Planner".into()),
            ])),
        );
        let builder = AuraViewBuilder::new(&bridge, "W");

        let node = AuraNode::element("textarea")
            .with_prop("value", Expr::Ident(".text".into()))
            .with_prop("mentions", Expr::Ident(".mentionNames".into()));
        match builder.build(&node) {
            View::Textarea {
                value, highlight, ..
            } => {
                assert_eq!(value, "@assistant hi");
                assert_eq!(
                    highlight,
                    vec![
                        ("@assistant".to_string(), "mention".to_string()),
                        (" hi".to_string(), "text".to_string()),
                    ],
                    "state-rooted 名单必须经 mention_segments 接入 highlight"
                );
            }
            _ => panic!("Expected View::Textarea"),
        }
    }

    /// PLAN-493 T3: 降级契约——computed 名单（解析会进 call 链，UAF 红线）
    /// 不解析：highlight 空、构建不崩（= 迁移前 VM 现状，非回归）。若此处
    /// 变红说明有人把 eval_computed/call_vm_fn 回调进了名单解析。
    #[test]
    fn plan493_mentions_computed_degrades_to_plain() {
        let mut widget = make_test_widget(
            "W",
            vec![AuraStateDef {
                name: "text".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("@assistant hi".into()),
                decorators: vec![],
            }],
        );
        widget.computed = vec![auto_ui_compiler::aura::AuraComputed {
            name: "mentionNames".to_string(),
            expr: Expr::Str("assistant".into()),
        }];
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "W");

        let node = AuraNode::element("textarea")
            .with_prop("value", Expr::Ident(".text".into()))
            .with_prop("mentions", Expr::Ident(".mentionNames".into()));
        match builder.build(&node) {
            View::Textarea { highlight, .. } => {
                assert!(
                    highlight.is_empty(),
                    "computed 名单不得解析（call 链 UAF 红线）: {:?}",
                    highlight
                );
            }
            _ => panic!("Expected View::Textarea"),
        }
    }

    /// PLAN-493 T3: 字面量表名单——`mentions: ["coder"]` 静态形态同样接入。
    #[test]
    fn plan493_mentions_literal_list() {
        let widget = make_test_widget("W", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "W");

        let node = AuraNode::element("textarea")
            .with_prop("value", Expr::Str("@coder!".into()))
            .with_prop("mentions", Expr::Array(vec![Expr::Str("coder".into())]));
        match builder.build(&node) {
            View::Textarea { highlight, .. } => {
                assert_eq!(
                    highlight,
                    vec![
                        ("@coder".to_string(), "mention".to_string()),
                        ("!".to_string(), "text".to_string()),
                    ]
                );
            }
            _ => panic!("Expected View::Textarea"),
        }
    }

    /// PLAN-051 T8: for 循环多实例子组件的 props 逐实例隔离——musk 会话列表
    /// `for s in .store.session_list { NavListItem { id: .s.id, onactivate:
    /// .SelectSession($event) } }`，NavListItem 内部 `onclick: .activate(.id)`。
    /// 此前子 props 全写统一根态（同名字段互相覆盖），`.id` 烘焙读到碰撞值
    /// （实机：activate 载荷恒为当前活跃会话 id，切换失效）。期望：子 builder
    /// 带 per-instance prop 覆盖层，两实例按钮的 onclick 实参各为自己的 id。
    #[test]
    fn plan051_child_prop_override_per_instance() {
        use auto_lang_core::parser::Parser;
        let src = concat!(
            "widget Item51d {
",
            "    msg { activate(str) }
",
            "    view { button { onclick: .activate(.id) } }
",
            "    on { .activate(id) -> {} }
",
            "}
",
            "widget List51d {
",
            "    model { var ids []str = [\"aaa\", \"bbb\"] }
",
            "    view { col { for s in .ids { Item51d { id: .s } } } }
",
            "}
",
        );
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let mut decls: Vec<auto_ui_dialect::WidgetDecl> = vec![];
        for st in &ast.0.items {
            if let auto_parser_api::DomainStmt::Domain(auto_ui_dialect::UiStmt::WidgetDecl(d)) = st
            {
                decls.push(d.clone());
            }
        }
        // 声明序：Item51d 先、List51d 后 —— 根 = List51d（末声明）。
        let root_widget =
            auto_ui_compiler::aura::extract_widget_from_decl(&decls[1]).expect("extract List51d");
        let child_widget = auto_ui_compiler::aura::extract::extract_widget_from_decl(&decls[0])
            .expect("extract Item51d");
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child_widget);
        let bridge = VmBridge::new(&root_widget).unwrap();
        let builder = AuraViewBuilder::with_registry(&bridge, "List51d", &registry);
        let view = builder.build(&root_widget.view_tree);
        // 收集两实例按钮的 onclick 实参。
        fn collect_button_args(v: &View<DynamicMessage>, out: &mut Vec<String>) {
            match v {
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children {
                        collect_button_args(c, out);
                    }
                }
                View::Button { onclick, .. } => {
                    if let DynamicMessage::Typed { args, .. } = onclick {
                        for a in args {
                            if let Value::Str(s) = a {
                                out.push(s.as_str().to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let mut got: Vec<String> = Vec::new();
        collect_button_args(&view, &mut got);
        assert_eq!(got.len(), 2, "两实例各一按钮; got {:?}", got);
        assert!(
            got.contains(&"aaa".to_string()),
            "实例 aaa 的实参必须为己 id: {:?}",
            got
        );
        assert!(
            got.contains(&"bbb".to_string()),
            "实例 bbb 的实参必须为己 id: {:?}",
            got
        );
    }

    /// PLAN-050 T9+C1 残余收尾: 按钮内容子树布局方向——按钮样式带 `flex`
    /// （行,默认）时多子内容必须包 Row（gap→spacing,items-*/justify-* 类
    /// 随 style 下传供 renderer row 臂消费）;`flex-col` 才包 Column。此前
    /// 恒 Column → rail 导航钮图标叠在文字上方、整体居中（vue 轨 flex 行）。
    #[test]
    fn plan050_button_content_subtree_flex_row_direction() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        // 结构子内容:图标组件 + 文本子节点 → content 子树
        let node = AuraNode::element("button")
            .with_prop(
                "style",
                Expr::Str("w-full flex items-center justify-start gap-2 px-3 py-2".into()),
            )
            .with_child(
                AuraNode::element("svg").with_prop("viewBox", Expr::Str("0 0 24 24".into())),
            )
            .with_child(AuraNode::text("会话"));
        let view = builder.build(&node);
        let View::Button {
            content: Some(content),
            ..
        } = view
        else {
            panic!("期望带 content 子树的 Button");
        };
        match *content {
            View::Row { spacing, style, .. } => {
                assert!(
                    (f32::from(spacing) - 8.0).abs() < f32::EPSILON,
                    "gap-2 → spacing 8; got {}",
                    spacing
                );
                let st = style.expect("items-*/justify-* 应随 style 下传");
                assert!(
                    st.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::ItemsCenter)),
                    "items-center 应在 Row style"
                );
                assert!(
                    st.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::JustifyStart)),
                    "justify-start 应在 Row style"
                );
            }
            other => panic!("期望 Row（flex 行）,得到其他布局"),
        }

        // flex-col → Column（原行为保留）
        let node = AuraNode::element("button")
            .with_prop(
                "style",
                Expr::Str("flex flex-col items-center gap-1".into()),
            )
            .with_child(
                AuraNode::element("svg").with_prop("viewBox", Expr::Str("0 0 24 24".into())),
            )
            .with_child(AuraNode::text("B"));
        let view = builder.build(&node);
        let View::Button {
            content: Some(content),
            ..
        } = view
        else {
            panic!("期望带 content 子树的 Button");
        };
        assert!(
            matches!(*content, View::Column { .. }),
            "flex-col 应包 Column"
        );
    }

    /// PLAN-050 T7 (C5): use.web component 声明的图标组件（`Folder { size: 14 }`）
    /// → View::Image{ src: "lucide:folder", size 样式 }，renderer 以既有
    /// lucide glyph 直绘（svg 直绘=resvg currentColor tint）。此前落 unknown
    /// fallback → View::Empty，rail/设置面板图标全空。
    #[test]
    fn plan050_imported_component_icon_maps_to_lucide_image() {
        struct ResetImportedComponents;
        impl Drop for ResetImportedComponents {
            fn drop(&mut self) {
                crate::ui::aura_view_builder::clear_imported_components();
            }
        }
        let _reset = ResetImportedComponents;
        crate::ui::aura_view_builder::clear_imported_components();
        crate::ui::aura_view_builder::register_imported_components(vec![
            "Folder".to_string(),
            "Folder_icon".to_string(),
            "AppViewport".to_string(),
        ]);
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let registry = crate::ui::widget_registry::WidgetRegistry::new();
        let builder = AuraViewBuilder::with_registry(&bridge, "Test", &registry);

        // PLAN-748 T-02：判别器改为 lucide 名集成员资格（含 "icon" 字样
        // 向后兼容）——Folder ∈ lucide 名集恢复 glyph（A-05 曾按
        // `tag.contains("icon")` 粗闸把 PascalCase lucide 名钉成占位卡，
        // musk 图标子件整体消失 = p054_t1/t4 两红根因）。
        let node = AuraNode::element("Folder").with_prop("size", Expr::Int(14));
        let view = builder.build(&node);
        match view {
            View::Image { src, style } => {
                assert_eq!(
                    src, "lucide:folder",
                    "Folder 应映射 lucide:folder; got {src}"
                );
                let s = style.expect("size 应产出样式");
                assert!(
                    s.classes.iter().any(|c| matches!(c, StyleClass::Width(SizeValue::Pixels(p)) if (*p - 14.0).abs() < f32::EPSILON)),
                    "width 应为 14px; got {:?}",
                    s.classes
                );
            }
            other => panic!("期望 View::Image（lucide glyph）,得到 {other:?}"),
        }

        // 含 "icon" 的 tag 保持 glyph 路径（625 向后兼容面）。
        let node_i = AuraNode::element("Folder_icon").with_prop("size", Expr::Int(14));
        let view_i = builder.build(&node_i);
        match view_i {
            View::Image { src, style } => {
                assert!(src.starts_with("lucide:"), "src 应为 lucide 合成名: {src}");
                let s = style.expect("size 应产出样式");
                assert!(
                    s.classes.iter().any(|c| matches!(c, StyleClass::Width(SizeValue::Pixels(p)) if (*p - 14.0).abs() < f32::EPSILON)),
                    "width 应为 14px; got {:?}",
                    s.classes
                );
            }
            other => panic!("icon 字样 tag 期望 View::Image,得到 {other:?}"),
        }

        // PLAN-748 T-02：∉ lucide 名集的网页生态组件（AppViewport 形）保持
        // T-09(b) 占位卡路径（判别器换名集后意图面不动）。
        let builder_vp = AuraViewBuilder::with_registry(&bridge, "Test", &registry);
        let node_vp = AuraNode::element("AppViewport").with_prop("app", Expr::Str("demo-1".into()));
        let view_vp = builder_vp.build(&node_vp);
        match &view_vp {
            View::Column { children, .. } => {
                let first_text = children.iter().find_map(|c| match c {
                    View::Text { content, .. } => Some(content.clone()),
                    _ => None,
                });
                assert_eq!(
                    first_text.as_deref(),
                    Some("⚙ AppViewport（Web 端组件）"),
                    "∉ lucide 名集的组件应保持占位卡; got {:?}",
                    view_vp
                );
            }
            other => panic!("期望占位卡 Column,得到 {other:?}"),
        }

        // 未导入的同名 tag 不映射（registry/常规 fallback 优先语义不变）
        crate::ui::aura_view_builder::register_imported_components(vec![]);
        let widget2 = make_test_widget("Test2", vec![]);
        let bridge2 = VmBridge::new(&widget2).unwrap();
        let registry2 = crate::ui::widget_registry::WidgetRegistry::new();
        let builder2 = AuraViewBuilder::with_registry(&bridge2, "Test2", &registry2);
        let node2 = AuraNode::element("Folder_icon").with_prop("size", Expr::Int(14));
        assert!(
            !matches!(builder2.build(&node2), View::Image { .. }),
            "未导入的组件不得映射为图标"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Plan 482: nav-item / nav-group / nav(search:) 转换测试
    // ─────────────────────────────────────────────────────────────────────

    fn nav_style_classes(v: &View<DynamicMessage>) -> Vec<crate::ui::style::StyleClass> {
        match v {
            View::Button { style, .. } => style.as_ref().expect("nav-item style").classes.clone(),
            other => panic!("期望 View::Button,得到 {other:?}"),
        }
    }

    fn classes_contain(classes: &[crate::ui::style::StyleClass], token: &str) -> bool {
        let parsed = crate::ui::style::Style::parse(token).unwrap().classes;
        !parsed.is_empty() && parsed.iter().all(|want| classes.iter().any(|c| c == want))
    }

    /// hover:* 契约串 → 对照 hover_classes（Style::parse 把它们分到并行表）。
    fn hover_classes_contain(hover: &[crate::ui::style::StyleClass], token: &str) -> bool {
        let parsed = crate::ui::style::Style::parse(token).unwrap().hover_classes;
        !parsed.is_empty() && parsed.iter().all(|want| hover.iter().any(|c| c == want))
    }

    fn nav_hover_classes(v: &View<DynamicMessage>) -> Vec<crate::ui::style::StyleClass> {
        match v {
            View::Button { style, .. } => style
                .as_ref()
                .expect("nav-item style")
                .hover_classes
                .clone(),
            other => panic!("期望 View::Button,得到 {other:?}"),
        }
    }

    /// 路由模式默认态：契约基类 + hover（未选中），无 active 块；onclick 走
    /// __navigate；icon 为 lucide 名 → View::Image。
    #[test]
    fn test_nav_item_router_mode_default() {
        use auto_ui_compiler::ui_gen::nav_contract as nc;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("nav-item")
            .with_prop("to", Expr::Str("/chats".into()))
            .with_prop("icon", Expr::Str("message-square".into()))
            .with_prop("label", Expr::Str("会话".into()));
        match builder.build(&node) {
            View::Button {
                onclick,
                content,
                style,
                ..
            } => {
                match onclick {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "__navigate");
                        assert_eq!(args.first().map(|v| v.as_str()), Some("/chats"));
                    }
                    other => panic!("期望 __navigate 消息,得到 {other:?}"),
                }
                let content = content.expect("content row");
                assert!(
                    matches!(*content, View::Row { .. }),
                    "icon+texts 合成内容应为 Row"
                );
                let st = style.expect("style");
                assert!(
                    classes_contain(&st.classes, nc::ITEM_BASE_MD),
                    "基类缺失: {:?}",
                    st.classes
                );
                assert!(
                    hover_classes_contain(&st.hover_classes, nc::ITEM_HOVER),
                    "未选中应挂 hover"
                );
                assert!(
                    !classes_contain(&st.classes, nc::ITEM_ACTIVE),
                    "不应有 active 块"
                );
            }
            other => panic!("期望 View::Button,得到 {other:?}"),
        }
    }

    /// 路由命中当前路由 → active 块替换 hover（构建期二选一）。
    #[test]
    fn test_nav_item_router_mode_active() {
        use auto_ui_compiler::ui_gen::nav_contract as nc;
        let widget = make_test_widget(
            "Test",
            vec![AuraStateDef {
                name: "__current_route".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("/".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("__current_route", auto_val::Value::str("/chats"))
            .unwrap();

        // 精确命中。
        let node = AuraNode::element("nav-item")
            .with_prop("to", Expr::Str("/chats".into()))
            .with_prop("label", Expr::Str("会话".into()));
        let v = AuraViewBuilder::new(&bridge, "Test").build(&node);
        let classes = nav_style_classes(&v);
        assert!(
            classes_contain(&classes, nc::ITEM_ACTIVE),
            "命中应有 active 块"
        );
        assert!(
            !hover_classes_contain(&nav_hover_classes(&v), nc::ITEM_HOVER),
            "选中不再挂完整 hover 组"
        );
        // PLAN-063 Phase B T19 (KD 061 D23): active 自带中性灰 hover。
        assert!(
            hover_classes_contain(&nav_hover_classes(&v), "hover:bg-accent"),
            "选中项应自带 hover:bg-accent(契约放宽)"
        );

        // 前缀段命中（/chats 对 /chats/1 生效）；exact 收紧。
        let node = AuraNode::element("nav-item").with_prop("to", Expr::Str("/chats".into()));
        let classes = nav_style_classes(&AuraViewBuilder::new(&bridge, "Test").build(&node));
        assert!(classes_contain(&classes, nc::ITEM_ACTIVE));
        bridge
            .write_state("__current_route", auto_val::Value::str("/chats/1"))
            .unwrap();
        let node = AuraNode::element("nav-item")
            .with_prop("to", Expr::Str("/chats".into()))
            .with_prop("exact", Expr::Bool(true));
        let classes = nav_style_classes(&AuraViewBuilder::new(&bridge, "Test").build(&node));
        assert!(
            !classes_contain(&classes, nc::ITEM_ACTIVE),
            "exact 下前缀不命中"
        );
    }

    /// 状态模式：onclick 用户消息 + active 表达式驱动选中；disabled 灰置。
    #[test]
    fn test_nav_item_onclick_mode_states() {
        use auto_ui_compiler::ui_gen::nav_contract as nc;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("nav-item")
            .with_prop("icon", Expr::Str("🔌".into()))
            .with_prop("label", Expr::Str("网络".into()))
            .with_prop("desc", Expr::Str("适配器与代理".into()))
            .with_prop("active", Expr::Bool(true))
            .with_event("onclick", ".Select(1)");
        match builder.build(&node) {
            View::Button {
                onclick,
                disabled,
                style,
                ..
            } => {
                match onclick {
                    DynamicMessage::Typed { event_name, .. } => {
                        assert!(event_name.starts_with("Select"), "用户消息名: {event_name}");
                    }
                    other => panic!("期望用户消息,得到 {other:?}"),
                }
                assert!(!disabled);
                let st = style.expect("style");
                assert!(classes_contain(&st.classes, nc::ITEM_ACTIVE));
                assert!(!hover_classes_contain(&st.hover_classes, nc::ITEM_HOVER));
                // lg 未指定 → md 基类;desc 走 TEXT_DESC 色。
                assert!(classes_contain(&st.classes, nc::ITEM_BASE_MD));
            }
            other => panic!("期望 View::Button,得到 {other:?}"),
        }

        // disabled：ITEM_DISABLED 替代 hover。
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("nav-item")
            .with_prop("label", Expr::Str("禁用".into()))
            .with_prop("disabled", Expr::Bool(true));
        match builder.build(&node) {
            View::Button {
                disabled, style, ..
            } => {
                assert!(disabled);
                let st = style.expect("style");
                assert!(classes_contain(&st.classes, nc::ITEM_DISABLED));
                assert!(!hover_classes_contain(&st.hover_classes, nc::ITEM_HOVER));
            }
            other => panic!("期望 View::Button,得到 {other:?}"),
        }
    }

    /// nav-group：非折叠 → 标签头 + 成员列；折叠默认开（内置态缺省 true），
    /// 内置态 false 时成员不渲染；绑定 open 表达式优先。
    #[test]
    fn test_nav_group_fold_states() {
        use auto_ui_compiler::ui_gen::nav_contract as nc;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();

        // 默认开：头(Button __nav_toggle) + 成员列。
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("nav-group")
            .with_prop("label", Expr::Str("分组".into()))
            .with_prop("collapsible", Expr::Bool(true))
            .with_child(AuraNode::element("nav-item").with_prop("label", Expr::Str("项".into())));
        match builder.build(&node) {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 2, "头 + 成员列");
                match &children[0] {
                    View::Button { onclick, .. } => match onclick.clone() {
                        DynamicMessage::Typed {
                            event_name, args, ..
                        } => {
                            assert_eq!(event_name, "__nav_toggle");
                            assert_eq!(
                                args.first().map(|v| v.as_str()),
                                Some("__nav_group_open:分组")
                            );
                        }
                        other => panic!("期望 __nav_toggle,得到 {other:?}"),
                    },
                    other => panic!("折叠头应为 Button,得到 {other:?}"),
                }
                assert!(matches!(&children[1], View::Column { .. }), "成员列");
            }
            other => panic!("期望 View::Column,得到 {other:?}"),
        }

        // 内置态 false → 收起（成员列缺席）。
        let mut states = std::collections::HashMap::new();
        states.insert("__nav_group_open:分组".to_string(), false);
        let builder = AuraViewBuilder::new(&bridge, "Test").with_nav_group_states(&states);
        match builder.build(&node) {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 1, "收起时仅头");
            }
            other => panic!("期望 View::Column,得到 {other:?}"),
        }

        // 非折叠：标签头是 Text（GROUP_LABEL 类）。
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("nav-group")
            .with_prop("label", Expr::Str("标签组".into()))
            .with_child(AuraNode::element("nav-item").with_prop("label", Expr::Str("项".into())));
        match builder.build(&node) {
            View::Column { children, .. } => {
                assert!(matches!(&children[0], View::Text { .. }));
                assert!(matches!(&children[1], View::Column { .. }));
            }
            other => panic!("期望 View::Column,得到 {other:?}"),
        }
    }

    /// nav(search:true)：首子为搜索行（SEARCH_ROW 类 Row：🔍 文本 +
    /// Input），其余子节点跟列。
    #[test]
    fn test_nav_search_row() {
        use auto_ui_compiler::ui_gen::nav_contract as nc;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("nav")
            .with_prop("search", Expr::Bool(true))
            .with_prop("search_placeholder", Expr::Str("搜索设置".into()))
            .with_event("onsearch", ".SearchChanged")
            .with_child(AuraNode::element("nav-item").with_prop("label", Expr::Str("项".into())));
        match builder.build(&node) {
            View::Column {
                children, style, ..
            } => {
                assert!(style.expect("nav 容器 style").classes.len() > 0);
                assert_eq!(children.len(), 2, "搜索行 + 子节点");
                match &children[0] {
                    View::Row {
                        children: row_children,
                        style,
                        ..
                    } => {
                        let classes = style.as_ref().expect("搜索行 style").classes.clone();
                        assert!(
                            classes_contain(&classes, nc::SEARCH_ROW),
                            "搜索行契约类缺失"
                        );
                        assert!(
                            matches!(&row_children[0], View::Text { content, .. } if content == "🔍"),
                            "搜索图标应为 🔍 文本字形"
                        );
                        assert!(
                            matches!(&row_children[1], View::Input { .. }),
                            "第二个应为输入框"
                        );
                    }
                    other => panic!("搜索行应为 Row,得到 {other:?}"),
                }
                assert!(
                    matches!(&children[1], View::Button { .. }),
                    "nav-item 子节点"
                );
            }
            other => panic!("期望 View::Column,得到 {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Plan 561: sidebar_* VM 契约子集转换测试
    // ─────────────────────────────────────────────────────────────────────

    /// 容器与分区树：provider 透明传递 → sidebar 根列（契约基座 + w-64），
    /// header/content/footer 三分区各就各位；provider `default_open: false`
    /// 时 offcanvas 整树隐藏；collapsible:"none" 不受 open 门控。
    #[test]
    fn test_sidebar_tree_structure_and_provider_gate() {
        use auto_ui_compiler::ui_gen::sidebar_contract as sc;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let sidebar_node = || {
            AuraNode::element("sidebar")
                .with_child(AuraNode::element("sidebar_header").with_child(AuraNode::text("头")))
                .with_child(AuraNode::element("sidebar_content").with_child(AuraNode::text("体")))
                .with_child(AuraNode::element("sidebar_footer").with_child(AuraNode::text("脚")))
        };
        let provider = AuraNode::element("sidebar_provider").with_child(sidebar_node());
        match builder.build(&provider) {
            View::Column {
                children, style, ..
            } => {
                let classes = style.expect("sidebar 根 style").classes;
                assert!(
                    classes_contain(&classes, sc::SIDEBAR_BASE),
                    "根契约基座缺失: {:?}",
                    classes
                );
                assert!(classes_contain(&classes, sc::WIDTH_VM), "VM 宽度缺失");
                assert_eq!(children.len(), 3, "header/content/footer 三分区");
                assert!(matches!(&children[0], View::Column { .. }), "header 列");
                assert!(matches!(&children[2], View::Column { .. }), "footer 列");
            }
            other => panic!("期望 View::Column,得到 {other:?}"),
        }

        // provider default_open:false → offcanvas 桌面 = 整树隐藏。
        let closed = AuraNode::element("sidebar_provider")
            .with_prop("default_open", Expr::Bool(false))
            .with_child(sidebar_node());
        assert!(
            matches!(builder.build(&closed), View::Empty),
            "offcanvas 关 = 隐藏"
        );

        // collapsible:"none" 不受 open 门控。
        let pinned = AuraNode::element("sidebar_provider")
            .with_prop("default_open", Expr::Bool(false))
            .with_child(
                AuraNode::element("sidebar")
                    .with_prop("collapsible", Expr::Str("none".into()))
                    .with_child(AuraNode::text("x")),
            );
        assert!(
            matches!(builder.build(&pinned), View::Column { .. }),
            "collapsible=none 不隐藏"
        );
    }

    /// 可折叠 group：首个 group_label 子节点转 toggle 按钮（__nav_toggle
    /// 消息 + `__sidebar_group_open:` 命名空间键），内置态 false 时内容
    /// 收起；非折叠 group 的 label 走 Text + GROUP_LABEL 契约类。
    #[test]
    fn test_sidebar_group_collapsible_toggle() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("sidebar_group")
            .with_prop("collapsible", Expr::Bool(true))
            .with_child(
                AuraNode::element("sidebar_group_label")
                    .with_prop("text", Expr::Str("工作区".into())),
            )
            .with_child(
                AuraNode::element("sidebar_group_content").with_child(AuraNode::text("项")),
            );
        match builder.build(&node) {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 2, "toggle 头 + 内容");
                match &children[0] {
                    View::Button { onclick, .. } => match onclick.clone() {
                        DynamicMessage::Typed {
                            event_name, args, ..
                        } => {
                            assert_eq!(event_name, "__nav_toggle");
                            assert_eq!(
                                args.first().map(|v| v.as_str()),
                                Some("__sidebar_group_open:工作区")
                            );
                        }
                        other => panic!("期望 __nav_toggle,得到 {other:?}"),
                    },
                    other => panic!("折叠头应为 Button,得到 {other:?}"),
                }
            }
            other => panic!("期望 View::Column,得到 {other:?}"),
        }

        // 内置态 false → 内容收起（仅头）。
        let mut states = std::collections::HashMap::new();
        states.insert("__sidebar_group_open:工作区".to_string(), false);
        let builder = AuraViewBuilder::new(&bridge, "Test").with_nav_group_states(&states);
        match builder.build(&node) {
            View::Column { children, .. } => assert_eq!(children.len(), 1, "收起时仅头"),
            other => panic!("期望 View::Column,得到 {other:?}"),
        }

        // 非折叠：label 是带契约类的 Text。
        let node = AuraNode::element("sidebar_group")
            .with_child(
                AuraNode::element("sidebar_group_label")
                    .with_prop("text", Expr::Str("项目".into())),
            )
            .with_child(AuraNode::text("x"));
        match AuraViewBuilder::new(&bridge, "Test").build(&node) {
            View::Column { children, .. } => {
                assert!(
                    matches!(&children[0], View::Text { .. }),
                    "非折叠 label 为 Text"
                );
            }
            other => panic!("期望 View::Column,得到 {other:?}"),
        }
    }

    /// 菜单层级：menu → MENU_BASE 列；item → MENU_ITEM 列（button + badge
    /// 双子）；badge 为 MENU_BADGE 文本；sub → MENU_SUB 缩进列（border-l）。
    #[test]
    fn test_sidebar_menu_hierarchy() {
        use auto_ui_compiler::ui_gen::sidebar_contract as sc;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("sidebar_menu")
            .with_child(
                AuraNode::element("sidebar_menu_item")
                    .with_child(
                        AuraNode::element("sidebar_menu_button")
                            .with_prop("text", Expr::Str("收件箱".into())),
                    )
                    .with_child(
                        AuraNode::element("sidebar_menu_badge")
                            .with_prop("text", Expr::Str("3".into())),
                    ),
            )
            .with_child(
                AuraNode::element("sidebar_menu_sub").with_child(
                    AuraNode::element("sidebar_menu_sub_item").with_child(
                        AuraNode::element("sidebar_menu_sub_button")
                            .with_prop("text", Expr::Str("子项".into())),
                    ),
                ),
            );
        match builder.build(&node) {
            View::Column {
                children, style, ..
            } => {
                assert!(
                    classes_contain(&style.expect("menu style").classes, sc::MENU_BASE),
                    "menu 契约类缺失"
                );
                assert_eq!(children.len(), 2, "item + sub");
                match &children[0] {
                    View::Column {
                        children: item_children,
                        style,
                        ..
                    } => {
                        assert!(
                            classes_contain(
                                &style.as_ref().expect("item style").classes,
                                sc::MENU_ITEM
                            ),
                            "item 契约类缺失"
                        );
                        assert!(
                            matches!(&item_children[0], View::Button { .. }),
                            "button 子"
                        );
                        match &item_children[1] {
                            View::Text { content, style, .. } => {
                                assert_eq!(content, "3");
                                assert!(
                                    classes_contain(
                                        &style.as_ref().expect("badge style").classes,
                                        sc::MENU_BADGE
                                    ),
                                    "badge 契约类缺失"
                                );
                            }
                            other => panic!("badge 应为 Text,得到 {other:?}"),
                        }
                    }
                    other => panic!("item 应为 Column,得到 {other:?}"),
                }
                match &children[1] {
                    View::Column { style, .. } => {
                        assert!(
                            classes_contain(
                                &style.as_ref().expect("sub style").classes,
                                sc::MENU_SUB
                            ),
                            "sub 缩进契约类缺失"
                        );
                    }
                    other => panic!("sub 应为 Column,得到 {other:?}"),
                }
            }
            other => panic!("期望 View::Column,得到 {other:?}"),
        }
    }

    /// menu_button 三态与路由：to: → __navigate；未命中挂 hover；路由命中
    /// 或显式 active: → active 块替换 hover；disabled 灰置无 hover；
    /// sub_button 走 SUB 基座 + size sm。
    #[test]
    fn test_sidebar_menu_button_states_and_route() {
        use auto_ui_compiler::ui_gen::sidebar_contract as sc;
        let widget = make_test_widget(
            "Test",
            vec![AuraStateDef {
                name: "__current_route".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("/".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("__current_route", auto_val::Value::str("/dash"))
            .unwrap();

        // 默认态：to: → __navigate，未命中 → hover 无 active。
        let node = AuraNode::element("sidebar_menu_button")
            .with_prop("to", Expr::Str("/settings".into()))
            .with_prop("text", Expr::Str("设置".into()));
        match AuraViewBuilder::new(&bridge, "Test").build(&node) {
            View::Button {
                onclick,
                style,
                disabled,
                ..
            } => {
                assert!(!disabled);
                match onclick {
                    DynamicMessage::Typed {
                        event_name, args, ..
                    } => {
                        assert_eq!(event_name, "__navigate");
                        assert_eq!(args.first().map(|v| v.as_str()), Some("/settings"));
                    }
                    other => panic!("期望 __navigate,得到 {other:?}"),
                }
                let st = style.expect("style");
                assert!(
                    classes_contain(&st.classes, sc::MENU_BUTTON_BASE),
                    "基座缺失"
                );
                assert!(
                    classes_contain(&st.classes, sc::MENU_BUTTON_SIZE_DEFAULT),
                    "尺寸缺失"
                );
                assert!(
                    hover_classes_contain(&st.hover_classes, sc::MENU_BUTTON_HOVER),
                    "未选中应挂 hover"
                );
                assert!(
                    !classes_contain(&st.classes, sc::MENU_BUTTON_ACTIVE),
                    "不应有 active 块"
                );
            }
            other => panic!("期望 View::Button,得到 {other:?}"),
        }

        // 路由命中 → active 整串替换 hover（前缀段同 nav-item 语义）。
        let node = AuraNode::element("sidebar_menu_button")
            .with_prop("to", Expr::Str("/dash".into()))
            .with_prop("text", Expr::Str("面板".into()));
        let v = AuraViewBuilder::new(&bridge, "Test").build(&node);
        let st = match &v {
            View::Button { style, .. } => style.as_ref().expect("style"),
            other => panic!("期望 View::Button,得到 {other:?}"),
        };
        assert!(
            classes_contain(&st.classes, sc::MENU_BUTTON_ACTIVE),
            "命中应有 active 块"
        );
        assert!(
            !hover_classes_contain(&st.hover_classes, sc::MENU_BUTTON_HOVER),
            "选中不挂 hover"
        );

        // disabled：灰置 + 无 hover。
        let node = AuraNode::element("sidebar_menu_button")
            .with_prop("text", Expr::Str("禁用".into()))
            .with_prop("disabled", Expr::Bool(true));
        match AuraViewBuilder::new(&bridge, "Test").build(&node) {
            View::Button {
                disabled, style, ..
            } => {
                assert!(disabled);
                let st = style.expect("style");
                assert!(classes_contain(&st.classes, sc::MENU_BUTTON_DISABLED));
                assert!(!hover_classes_contain(
                    &st.hover_classes,
                    sc::MENU_BUTTON_HOVER
                ));
            }
            other => panic!("期望 View::Button,得到 {other:?}"),
        }

        // sub_button：SUB 基座 + size sm 附加 text-xs。
        let node = AuraNode::element("sidebar_menu_sub_button")
            .with_prop("text", Expr::Str("子".into()))
            .with_prop("size", Expr::Str("sm".into()));
        let v = AuraViewBuilder::new(&bridge, "Test").build(&node);
        let st = match &v {
            View::Button { style, .. } => style.as_ref().expect("style"),
            other => panic!("期望 View::Button,得到 {other:?}"),
        };
        assert!(
            classes_contain(&st.classes, sc::MENU_SUB_BUTTON_BASE),
            "sub 基座缺失"
        );
        assert!(
            classes_contain(&st.classes, sc::MENU_SUB_BUTTON_SIZE_SM),
            "sub sm 尺寸缺失"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // PLAN-619 R1 / T-02：sidebar 族用户类合并 + icon size 权威
    // ─────────────────────────────────────────────────────────────────────

    /// PLAN-619 R1 回归：`sidebar_content (style: "px-2 pb-2")` 的用户类必须
    /// 与 CONTENT_BASE 契约**同时**生效。旧实现 `with_class_prop` 只读
    /// `class:` 键，`style:`（DSL 主写法）被整串丢弃——VM 侧栏列表因此少
    /// 8px 左缩进（P4 的横向那一半）。
    #[test]
    fn plan619_sidebar_content_style_prop_keeps_user_classes() {
        use auto_ui_compiler::ui_gen::sidebar_contract as sc;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        for key in ["style", "class"] {
            let node =
                AuraNode::element("sidebar_content").with_prop(key, Expr::Str("px-2 pb-2".into()));
            match builder.build(&node) {
                View::Scrollable { style, .. } => {
                    let classes = style.expect("scroll style").classes;
                    assert!(
                        classes_contain(&classes, sc::CONTENT_BASE),
                        "`{key}:` 下契约基座缺失: {classes:?}"
                    );
                    assert!(
                        classes_contain(&classes, "px-2 pb-2"),
                        "`{key}:` 下用户类丢失: {classes:?}"
                    );
                }
                other => panic!("sidebar_content 应为 Scrollable,得到 {other:?}"),
            }
        }
    }

    /// PLAN-619 R1 回归：`class:` 与 `style:` 两键在下游等价（同串落位）。
    #[test]
    fn plan619_class_and_style_props_are_equivalent() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let classes_of = |key: &str| {
            let node = AuraNode::element("sidebar_group_label")
                .with_prop("text", Expr::Str("Pinned".into()))
                .with_prop(key, Expr::Str("text-xs font-medium".into()));
            match builder.build(&node) {
                View::Text { style, .. } => style.expect("label style").classes,
                other => panic!("sidebar_group_label 应为 Text,得到 {other:?}"),
            }
        };
        assert_eq!(classes_of("class"), classes_of("style"), "两键应同串落位");
    }

    /// PLAN-619 T-02 回归（用户裁决：`.at` 的 `size:` 是权威）：icon 元素的
    /// `size` 必须折成显式 Width/Height 类。旧实现只取 name/src，size 不落
    /// → VM 图标落渲染器缺省 16px，而 Vue 固定 w-5 h-5(20px)，同一 glyph
    /// 两端 ink 差 ≈50%（P2）。
    #[test]
    fn plan619_icon_size_prop_sets_explicit_box() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::element("icon")
            .with_prop("name", Expr::Str("search".into()))
            .with_prop("size", Expr::Int(14))
            .with_prop("style", Expr::Str("text-muted-foreground".into()));
        match builder.build(&node) {
            View::Image { src, style } => {
                assert_eq!(src, "lucide:search");
                let classes = style.expect("icon style").classes;
                assert!(
                    classes_contain(&classes, "w-[14px] h-[14px]"),
                    "size 未落盒: {classes:?}"
                );
                assert!(
                    classes_contain(&classes, "text-muted-foreground"),
                    "icon 的 style: 用户类丢失: {classes:?}"
                );
            }
            other => panic!("icon 应为 Image,得到 {other:?}"),
        }

        // 显式 w-*/h-* 仍可覆盖 size（用户类后应用胜出，与 CSS 同序）。
        let over = AuraNode::element("icon")
            .with_prop("name", Expr::Str("search".into()))
            .with_prop("size", Expr::Int(14))
            .with_prop("style", Expr::Str("w-5 h-5".into()));
        match builder.build(&over) {
            View::Image { style, .. } => {
                let classes = style.expect("icon style").classes;
                // apply_class 后写胜出 → 用户 w-5/h-5 必须排在注入的 size 之后。
                let last_width = classes
                    .iter()
                    .filter_map(|c| match c {
                        crate::ui::style::StyleClass::Width(v) => Some(*v),
                        _ => None,
                    })
                    .next_back()
                    .expect("icon 必有宽度");
                assert_eq!(
                    last_width,
                    crate::ui::style::SizeValue::Fixed(5),
                    "显式 w-5 应排在 size 之后胜出: {classes:?}"
                );
            }
            other => panic!("icon 应为 Image,得到 {other:?}"),
        }
    }

    // ===== PLAN-621: icon state 契约（T-02）=====

    /// state:"on" → primary 色 + 描边加重基档+0.5（默认 20px 盒 → 2.5）；
    /// IcedStyle 适配器必须消费 StrokeWidth（parity 审计「类被消费」语义）。
    #[test]
    fn plan621_state_on_injects_primary_and_stroke() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("icon")
            .with_prop("name", Expr::Str("bell".into()))
            .with_prop("state", Expr::Str("on".into()));
        match builder.build(&node) {
            View::Image { style, .. } => {
                let s = style.as_ref().expect("icon style");
                let classes = &s.classes;
                assert!(
                    classes_contain(classes, "text-primary"),
                    "state:on 未注入 primary 色: {classes:?}"
                );
                assert!(
                    classes.iter()
                        .any(|c| matches!(c, crate::ui::style::StyleClass::StrokeWidth(w) if (*w - 2.5).abs() < f32::EPSILON)),
                    "state:on 未注入描边加重 2.5: {classes:?}"
                );
                let is = crate::ui::style_iced_adapter::IcedStyle::from_style(s);
                assert_eq!(is.stroke_width, Some(2.5), "适配器必须消费 StrokeWidth");
                assert!(
                    is.text_color.is_some(),
                    "primary 语义色必须解析为具体 iced 色（accent 预设驱动）"
                );
            }
            other => panic!("icon 应为 Image,得到 {other:?}"),
        }
    }

    /// state:"off" → muted-foreground dim（OnSurface），不加重。
    #[test]
    fn plan621_state_off_injects_muted_no_stroke() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("icon")
            .with_prop("name", Expr::Str("bell".into()))
            .with_prop("state", Expr::Str("off".into()));
        match builder.build(&node) {
            View::Image { style, .. } => {
                let classes = style.expect("icon style").classes;
                assert!(
                    classes_contain(&classes, "text-muted-foreground"),
                    "state:off 未注入 muted dim: {classes:?}"
                );
                assert!(
                    classes
                        .iter()
                        .all(|c| !matches!(c, crate::ui::style::StyleClass::StrokeWidth(_))),
                    "state:off 不得加重: {classes:?}"
                );
            }
            other => panic!("icon 应为 Image,得到 {other:?}"),
        }
    }

    /// 未声明 state → 零扰动（G3）：不得出现任何注入色/描边类。
    #[test]
    fn plan621_state_undeclared_zero_diff() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("icon")
            .with_prop("name", Expr::Str("search".into()))
            .with_prop("size", Expr::Int(14));
        match builder.build(&node) {
            View::Image { style, .. } => {
                let classes = style.expect("icon style").classes;
                assert!(
                    classes.iter().all(|c| !matches!(
                        c,
                        crate::ui::style::StyleClass::TextColor(_)
                            | crate::ui::style::StyleClass::StrokeWidth(_)
                    )),
                    "未声明 state 时不得注入颜色/描边: {classes:?}"
                );
            }
            other => panic!("icon 应为 Image,得到 {other:?}"),
        }
    }

    /// 显式 text-* 类优先于 state 色（镜像尺寸口径的「作者意图优先」）；
    /// 描边加重无作者字面通道，照常生效。
    #[test]
    fn plan621_explicit_text_class_wins_color_stroke_still_applies() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("icon")
            .with_prop("name", Expr::Str("bell".into()))
            .with_prop("state", Expr::Str("on".into()))
            .with_prop("style", Expr::Str("text-red-500".into()));
        match builder.build(&node) {
            View::Image { style, .. } => {
                let classes = style.expect("icon style").classes;
                let text_colors = classes
                    .iter()
                    .filter(|c| matches!(c, crate::ui::style::StyleClass::TextColor(_)))
                    .count();
                assert_eq!(text_colors, 1, "state 不得追加第二色: {classes:?}");
                assert!(
                    classes_contain(&classes, "text-red-500"),
                    "作者色必须保留: {classes:?}"
                );
                assert!(
                    classes.iter()
                        .any(|c| matches!(c, crate::ui::style::StyleClass::StrokeWidth(w) if (*w - 2.5).abs() < f32::EPSILON)),
                    "描边加重应照常生效: {classes:?}"
                );
            }
            other => panic!("icon 应为 Image,得到 {other:?}"),
        }
    }

    /// bool 绑定形态：`state: true`（Expr::Bool 直读，与 .flag 求值同通道）。
    #[test]
    fn plan621_state_bool_binding() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("icon")
            .with_prop("name", Expr::Str("heart".into()))
            .with_prop("state", Expr::Bool(true));
        match builder.build(&node) {
            View::Image { style, .. } => {
                let classes = style.expect("icon style").classes;
                assert!(classes_contain(&classes, "text-primary"), "{classes:?}");
            }
            other => panic!("icon 应为 Image,得到 {other:?}"),
        }
    }

    /// ≥48px 盒走细线基档（1.5），加重 = 1.5+0.5 = 2.0（renderer 同式）。
    #[test]
    fn plan621_state_on_48px_thin_base_plus_half() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");
        let node = AuraNode::element("icon")
            .with_prop("name", Expr::Str("zap".into()))
            .with_prop("size", Expr::Int(48))
            .with_prop("state", Expr::Str("on".into()));
        match builder.build(&node) {
            View::Image { style, .. } => {
                let classes = style.expect("icon style").classes;
                assert!(
                    classes.iter()
                        .any(|c| matches!(c, crate::ui::style::StyleClass::StrokeWidth(w) if (*w - 2.0).abs() < f32::EPSILON)),
                    "48px 盒加重应为 1.5+0.5=2.0: {classes:?}"
                );
            }
            other => panic!("icon 应为 Image,得到 {other:?}"),
        }
    }
}

/// Plan 422 P4: popover 子标签的 tag/children 读取(闭包无法表达返回借用的
/// 生命周期,落为模块级函数)。
fn popover_node_tag(n: &AuraNode) -> Option<&str> {
    match n {
        AuraNode::Element { tag, .. } => Some(tag.as_str()),
        _ => None,
    }
}

fn popover_node_children(n: &AuraNode) -> &[AuraNode] {
    match n {
        AuraNode::Element { children, .. } => children,
        _ => &[],
    }
}

/// Plan 442 A4: SVG 形状/结构子元素集合(serialize_svg_element 递归白名单;
/// 根元素 svg 由 convert_svg_image 显式发起)。与 ui_gen/vue.rs map_tag 的
/// SVG 直通臂同表。Plan 502 M1: 增 `text`(diagram 标签机制——位置参数
/// 文本经 props["text"] 进元素内容,见 serialize_svg_element)。
fn is_svg_shape_tag(tag: &str) -> bool {
    matches!(
        tag,
        "svg"
            | "path"
            | "circle"
            | "rect"
            | "line"
            | "polyline"
            | "polygon"
            | "ellipse"
            | "g"
            | "defs"
            | "use"
            | "stop"
            | "linearGradient"
            | "radialGradient"
            | "clipPath"
            | "mask"
            | "text"
    )
}

/// Plan 442 A4: 字面量 expr → SVG attribute 值(字符串原文,数值/布尔
/// to_string)。动态表达式返回 None(序列化时跳过)。
fn svg_attr_literal(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Str(s) | Expr::CStr(s) => Some(s.to_string()),
        Expr::Bool(b) => Some(b.to_string()),
        Expr::Int(v) => Some(v.to_string()),
        Expr::Uint(v) => Some(v.to_string()),
        Expr::Float(_, raw) | Expr::Double(_, raw) => Some(raw.to_string()),
        _ => None,
    }
}

/// PLAN-055: v-html 去标签纯文本降级（div_html_stripped_text 的实现辅助）。
/// 去除成对/自闭合 HTML 标签并解码常见命名/数字实体；连续空白不压缩
/// （UserMessage 的 whitespace-pre-wrap 语义交由容器 style 承担）。
fn strip_html_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut chars = html.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '<' {
            // 跳到下一个 '>'（无闭合则丢弃余下——残缺标签防御）
            let mut closed = false;
            for (_, c2) in chars.by_ref() {
                if c2 == '>' {
                    closed = true;
                    break;
                }
            }
            if !closed {
                break;
            }
            // 标签后补一个空格，避免相邻文本粘连（"<b>a</b>b" → "a b"）
            out.push(' ');
        } else if c == '&' {
            // 实体解码：截到 ';'（≤10 字符）查表
            let rest = &html[i + 1..];
            let end = rest.find(';').filter(|&p| p <= 10);
            if let Some(p) = end {
                let entity = &rest[..p];
                let decoded = match entity {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    "nbsp" => Some('\u{00a0}'),
                    _ => {
                        // 数字实体 &#NNN; / &#xHH;
                        if let Some(num) = entity.strip_prefix('#') {
                            let code = if let Some(hex) =
                                num.strip_prefix('x').or_else(|| num.strip_prefix('X'))
                            {
                                u32::from_str_radix(hex, 16).ok()
                            } else {
                                num.parse::<u32>().ok()
                            };
                            code.and_then(char::from_u32)
                        } else {
                            None
                        }
                    }
                };
                if let Some(ch) = decoded {
                    out.push(ch);
                    // 消费实体名 + 结尾分号（p 个名字字符后跟 ';'）
                    for _ in 0..=p {
                        chars.next();
                    }
                } else {
                    out.push(c);
                }
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod plan055_strip_html_tests {
    use super::strip_html_tags;
    #[test]
    fn strips_tags_and_decodes_entities() {
        // PLAN-668 R-09 语义裁定：strip_html_tags 职责 = 去标签 + 实体
        // 解码，不做空白重排——标签两侧原有空格原样保留（`</span> 你好`
        // 的前导空格 + 标签前空格各留一格）。改运行时折叠会波及 aura
        // markdown 等全消费面，非本批口径。
        assert_eq!(strip_html_tags("<span>@x</span> 你好"), " @x  你好");
        assert_eq!(strip_html_tags("a &amp; b &lt;c&gt;"), "a & b <c>");
        assert_eq!(strip_html_tags("plain"), "plain");
        assert_eq!(strip_html_tags("unclosed <b>text"), "unclosed  text");
    }
}

// ── PLAN-571: button default variant 一等化（VM 臂行为锁定）────────
#[cfg(test)]
mod plan571_button_default_variant_tests {
    use super::*;
    use crate::ui::style::Color;
    use auto_ui_compiler::aura::AuraWidget;

    fn make_test_widget(name: &str) -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: name.to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    fn build_button(variant: Option<&str>) -> View<DynamicMessage> {
        let widget = make_test_widget("Counter");
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");
        let mut props = HashMap::from([(
            "label".to_string(),
            AuraPropValue::Expr(Expr::Str("Save".into())),
        )]);
        if let Some(v) = variant {
            props.insert(
                "variant".to_string(),
                AuraPropValue::Expr(Expr::Str(v.into())),
            );
        }
        let node = AuraNode::Element {
            tag: "button".to_string(),
            props,
            events: HashMap::from([(
                "onclick".to_string(),
                AuraEvent {
                    handler: "Msg::Reset".to_string(),
                    params: vec![],
                },
            )]),
            span: None,
            debug_id: None,
            children: vec![],
        };
        builder.build(&node)
    }

    fn has_class(view: &View<DynamicMessage>, pred: &dyn Fn(&StyleClass) -> bool) -> bool {
        match view {
            View::Button { style: Some(s), .. } => s.classes.iter().any(|c| pred(c)),
            _ => false,
        }
    }

    #[test]
    fn absent_variant_is_chromeless_vm_arm() {
        // PLAN-080：缺省（无 variant prop）= preflight 等价 chromeless——
        // 无填充/无边框预设，外观完全归显式类（双轨与 vue cva 无
        // defaultVariants 互锁）。
        let view = build_button(None);
        assert!(
            matches!(view, View::Button { style: None, .. }),
            "缺省 button 无任何 preset 类（style=None → iced 透明贴内容）"
        );
    }

    #[test]
    fn explicit_default_is_neutral_baseline_with_hairline_border() {
        let v = Some("default");
        let view = build_button(v);
        assert!(
            has_class(&view, &|c| matches!(
                c,
                StyleClass::BackgroundColor(Color::Muted)
            )),
            "显式 default 基线 = muted 中性填充 (variant={v:?})"
        );
        assert!(
            has_class(&view, &|c| matches!(c, StyleClass::Border)),
            "显式 default 必须有发丝描边（UA 预填等价）(variant={v:?})"
        );
        assert!(
            has_class(&view, &|c| matches!(
                c,
                StyleClass::BorderColor(Color::Border)
            )),
            "描边色 = border 语义色 (variant={v:?})"
        );
        assert!(
            !has_class(&view, &|c| matches!(
                c,
                StyleClass::BackgroundColor(Color::Primary)
            )),
            "显式 default 不得再用主题色填充 (variant={v:?})"
        );
    }

    #[test]
    fn primary_and_submit_are_explicit_accent_fill() {
        for v in ["primary", "submit"] {
            let view = build_button(Some(v));
            assert!(
                has_class(&view, &|c| matches!(
                    c,
                    StyleClass::BackgroundColor(Color::Primary)
                )),
                "{v} = 主题色填充"
            );
            assert!(
                !has_class(&view, &|c| matches!(c, StyleClass::Border)),
                "{v} 填充档无边框"
            );
        }
    }

    #[test]
    fn secondary_is_deeper_fill_without_border() {
        let view = build_button(Some("secondary"));
        assert!(
            has_class(&view, &|c| matches!(
                c,
                StyleClass::BackgroundColor(Color::Secondary)
            )),
            "secondary = 纯填充"
        );
        assert!(
            !has_class(&view, &|c| matches!(c, StyleClass::Border)),
            "secondary 无边框（outline 专属）"
        );
    }
}

/// PLAN-534: sheet/drawer/hovercard 转换探针（全管线:parse 含铸造机器 →
/// DynamicComponent view 构建 → View 树断言）。
#[cfg(test)]
mod plan534_side_panel_tests {
    use crate::ui::interpreter::DynamicMessage;
    use crate::ui::view::{PopoverAnchor, PopoverPlacement, View};

    fn build_view(src: &str) -> View<DynamicMessage> {
        let dc = crate::build_dynamic_component(src, None).expect("build component");
        let (view, _, _) = dc.view_with_debug_gated(false);
        view
    }

    /// PLAN-781 T-01：`scroll-anchor` 属性 → 零布局 `View::ScrollAnchorMark`
    /// 包装（key 透传）；无属性不包装。
    #[test]
    fn plan781_scroll_anchor_wraps_child() {
        fn find_anchor(view: &View<DynamicMessage>) -> Vec<String> {
            let mut out = Vec::new();
            match view {
                View::ScrollAnchorMark { pane, key, child } => {
                    assert!(!pane.is_empty(), "pane injected by convert_scroll");
                    out.push(key.to_string());
                    out.extend(find_anchor(child));
                }
                View::Row { children, .. } | View::Column { children, .. } => {
                    for c in children {
                        out.extend(find_anchor(c));
                    }
                }
                View::Container { child, .. } | View::Scrollable { child, .. } => {
                    out.extend(find_anchor(child))
                }
                _ => {}
            }
            out
        }
        let anchored = build_view(
            "widget T { view { col { scroll-pane (controller: \"p1\") { col { scroll-anchor: \"p40\"; text \"row\" {} } } } } }",
        );
        let keys = find_anchor(&anchored);
        assert_eq!(keys, vec!["p40".to_string()], "anchored child wrapped once");

        let plain =
            build_view("widget T { view { col { scroll-pane { col { text \"row\" {} } } } } }");
        assert!(find_anchor(&plain).is_empty(), "no attr -> no wrapper");
    }

    /// PLAN-617 后续：`progress` 的 `onseek` 在 VM 侧的锚点——
    /// 视图构建必须产出 `on_seek: Some(..)`（缺它则 VM 端进度条不可拖拽，
    /// 而 Web 端可拖 → 双端行为分叉），且 `value`/`max` 仍归一到 0..1。
    #[test]
    fn progress_onseek_builds_seek_handler() {
        fn find_progress(view: &View<DynamicMessage>) -> Option<(f32, bool)> {
            match view {
                View::ProgressBar {
                    progress, on_seek, ..
                } => Some((*progress, on_seek.is_some())),
                View::Row { children, .. } | View::Column { children, .. } => {
                    children.iter().find_map(find_progress)
                }
                View::Container { child, .. } | View::Scrollable { child, .. } => {
                    find_progress(child)
                }
                _ => None,
            }
        }
        let with_seek = build_view(
            "widget App {
             msg { Seek(float) }
             model { pct float = 25.0 }
             on { .Seek(f) -> { .pct = f } }
             view { col { progress (value: .pct, max: 100.0, onseek: .Seek($0)) } }
             }",
        );
        let (progress, has_seek) = find_progress(&with_seek).expect("progress 节点");
        assert!(
            (progress - 0.25).abs() < 1e-4,
            "value/max 归一 0..1，实得 {progress}"
        );
        assert!(
            has_seek,
            "声明 onseek 后必须挂上 on_seek（否则 VM 端不可拖拽）"
        );

        // 未声明 onseek → 纯展示条，零 handler（兼容约束）
        let plain = build_view(
            "widget App {
             model { pct float = 25.0 }
             view { col { progress (value: .pct, max: 100.0) } }
             }",
        );
        let (_, plain_seek) = find_progress(&plain).expect("progress 节点");
        assert!(!plain_seek, "未声明 onseek 不得凭空多出 seek handler");
    }

    /// PLAN-712 T-12 回归锁：`onseek: .Seek($0)` 的 `$0` 是**事件期回调占位**
    /// （Vue 侧由生成器替换成真实取值），VM 侧不得落成 Str("$0") 字面量占据
    /// args[0]——否则 handler 形参绑到垃圾、真载荷错位到 args[1]，030 进度条
    /// 点击恒换算 0。修复后 `($0)` 与 `()` 两形态在 VM 端同义：调用 handler
    /// 后 args[0] 即回调臂追加的比例载荷。
    #[test]
    fn progress_onseek_placeholder_param_never_becomes_literal_arg() {
        fn find_on_seek(
            view: &View<DynamicMessage>,
        ) -> Option<crate::ui::view::PointerMoveHandler<DynamicMessage>> {
            match view {
                View::ProgressBar { on_seek, .. } => on_seek.clone(),
                View::Row { children, .. } | View::Column { children, .. } => {
                    children.iter().find_map(find_on_seek)
                }
                View::Container { child, .. } | View::Scrollable { child, .. } => {
                    find_on_seek(child)
                }
                _ => None,
            }
        }
        // `($0)` 形态（030 controls.at / vue.rs 金样同款）
        let placeholder = build_view(
            "widget App {
             msg { Seek(float) }
             model { pct float = 25.0 }
             on { .Seek(f) -> { .pct = f } }
             view { col { progress (value: .pct, max: 100.0, onseek: .Seek($0)) } }
             }",
        );
        let on_seek = find_on_seek(&placeholder).expect("progress 节点带 on_seek");
        let msg = on_seek.call(0.5, 0.0);
        let DynamicMessage::Typed {
            event_name, args, ..
        } = msg
        else {
            panic!("Seek 应产出 Typed 消息");
        };
        assert_eq!(event_name, "Seek");
        assert!(
            !args
                .iter()
                .any(|a| matches!(a, auto_val::Value::Str(s) if s.as_str().starts_with('$'))),
            "占位串不得进 args（实得 {args:?}）"
        );
        match args.first() {
            Some(auto_val::Value::Float(f)) | Some(auto_val::Value::Double(f)) => assert!(
                (*f - 0.501).abs() < 1e-6,
                "args[0] 必须是回调臂追加的比例 0.501，实得 {f}"
            ),
            other => panic!("args[0] 必须是 Float 载荷，实得 {other:?}"),
        }
        // `()` 空参形态（020 controls.at 同款）与 `($0)` 严格同义。
        let empty = build_view(
            "widget App {
             msg { Seek(float) }
             model { pct float = 25.0 }
             on { .Seek(f) -> { .pct = f } }
             view { col { progress (value: .pct, max: 100.0, onseek: .Seek()) } }
             }",
        );
        let on_seek = find_on_seek(&empty).expect("progress 节点带 on_seek");
        let msg = on_seek.call(0.5, 0.0);
        let DynamicMessage::Typed { args, .. } = msg else {
            panic!("Seek 应产出 Typed 消息");
        };
        match args.first() {
            Some(auto_val::Value::Float(f)) | Some(auto_val::Value::Double(f)) => assert!(
                (*f - 0.501).abs() < 1e-6,
                "空参形态 args[0] 同为比例 0.501，实得 {f}"
            ),
            other => panic!("空参形态 args[0] 必须是 Float 载荷，实得 {other:?}"),
        }
    }

    fn find_popover(view: &View<DynamicMessage>) -> Option<&View<DynamicMessage>> {
        match view {
            View::Popover { .. } => Some(view),
            View::Row { children, .. } | View::Column { children, .. } => {
                children.iter().find_map(find_popover)
            }
            View::Container { child, .. }
            | View::Scrollable { child, .. }
            | View::MouseArea { content: child, .. } => find_popover(child),
            _ => None,
        }
    }

    /// sheet 缺省 side → EdgeRight;铸造形态 dismiss 折算 __dlg_close_1。
    #[test]
    fn plan534_sheet_default_edge_right_with_minted_dismiss() {
        let view = build_view(
            "widget App {
             model { activeTab str = \"preview\" }
             view {
             col {
             sheet (side: \"right\") {
             sheet-trigger {
             button (text: \"Open\", variant: \"outline\") {}
             }
             sheet-content {
             sheet-title \"Edit Profile\"
             }
             }
             }
             }
             }
",
        );
        let pop = find_popover(&view).expect("树内应有 View::Popover");
        match pop {
            View::Popover {
                placement,
                open,
                on_dismiss,
                ..
            } => {
                assert_eq!(
                    placement,
                    &PopoverPlacement::EdgeRight,
                    "side 缺省 right → EdgeRight"
                );
                assert!(!open, "初渲染 open=false");
                match on_dismiss {
                    Some(DynamicMessage::Typed { event_name, .. }) => {
                        assert_eq!(
                            event_name, "__dlg_close_1",
                            "铸造 dismiss 折算 __dlg_close_N"
                        );
                    }
                    other => panic!("sheet 为可关闭族,铸造形态必有 dismiss,得到 {other:?}"),
                }
            }
            other => panic!("应定位到 Popover,得到 {other:?}"),
        }
    }

    /// drawer direction=bottom → EdgeBottom;面板首子为装饰把手 Container。
    #[test]
    fn plan534_drawer_bottom_edgebottom_with_handle_first_child() {
        let view = build_view(
            "widget App {
             model { activeTab str = \"preview\" }
             view {
             col {
             drawer (direction: \"bottom\") {
             drawer-trigger \"Open Drawer\"
             drawer-content {
             drawer-title \"Settings\"
             }
             }
             }
             }
             }
",
        );
        let pop = find_popover(&view).expect("树内应有 View::Popover");
        match pop {
            View::Popover {
                content,
                placement,
                on_dismiss,
                ..
            } => {
                assert_eq!(
                    placement,
                    &PopoverPlacement::EdgeBottom,
                    "direction=bottom → EdgeBottom"
                );
                assert!(on_dismiss.is_some(), "drawer 同 sheet 为可关闭族");
                match content.as_ref() {
                    View::Column { children, .. } => {
                        assert!(
                            matches!(children.first(), Some(View::Container { .. })),
                            "bottom 把手须为首子 Container"
                        );
                    }
                    other => panic!("面板应为 Column,得到 {other:?}"),
                }
            }
            other => panic!("应定位到 Popover,得到 {other:?}"),
        }
    }

    /// hovercard → MouseArea 包锚 + Bottom 非模态（on_dismiss=None）。
    #[test]
    fn plan534_hovercard_mousearea_wrapped_bottom_nonmodal() {
        let view = build_view(
            "widget App {
             model { activeTab str = \"preview\" }
             view {
             col {
             hovercard {
             hover-card-trigger {
             text \"@mentor\"
             }
             hover-card-content {
             text \"bio\"
             }
             }
             }
             }
             }
",
        );
        let pop = find_popover(&view).expect("树内应有 View::Popover");
        match pop {
            View::Popover {
                anchor,
                placement,
                on_dismiss,
                ..
            } => {
                assert_eq!(placement, &PopoverPlacement::Bottom, "hover 卡悬于锚下");
                assert!(on_dismiss.is_none(), "非模态:无 dismiss 折算");
                match anchor {
                    PopoverAnchor::Widget(inner) => match inner.as_ref() {
                        View::MouseArea {
                            on_enter, on_exit, ..
                        } => {
                            assert!(on_enter.is_some(), "铸造 enter 须接进 MouseArea");
                            assert!(on_exit.is_some(), "铸造 leave 须接进 MouseArea");
                        }
                        other => panic!("锚应被 MouseArea 包裹,得到 {other:?}"),
                    },
                    other => panic!("hovercard 锚应为 widget 锚,得到 {other:?}"),
                }
            }
            other => panic!("应定位到 Popover,得到 {other:?}"),
        }
    }
}

// ── PLAN-080 定罪探针：musk 消息行 rowClass 的 IR 解析 ──────────────────
#[cfg(test)]
mod plan080_rowclass_probe_tests {
    use super::*;
    use crate::ui::style::StyleClass;
    use auto_ui_compiler::aura::AuraWidget;

    fn widget() -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: "Probe".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    #[test]
    fn musk_rowclass_max_width_pct_survives_ir() {
        let w = widget();
        let bridge = VmBridge::new(&w).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Probe");
        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::from([(
                "style".to_string(),
                AuraPropValue::Expr(Expr::Str(
                    "flex flex-col gap-[3px] max-w-[70%] self-end items-end".into(),
                )),
            )]),
            events: HashMap::new(),
            span: None,
            debug_id: None,
            children: vec![],
        };
        let view = builder.build(&node);
        match view {
            View::Column { style: Some(s), .. } => {
                assert!(
                    s.classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::MaxWidthPct(70.0))),
                    "MaxWidthPct(70) 应在场: {:?}",
                    s.classes
                );
            }
            other => panic!("非 Column: {other:?}"),
        }
    }
}
