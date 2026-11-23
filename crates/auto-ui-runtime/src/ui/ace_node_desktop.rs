//! PLAN-980 NATIVE r13（B-NATIVE-I/V，MIN-A-UI）：通用 Node 桌面入口。
//!
//! 数据流（§5 合同）：原 Node/普通模块源 → 活动 AGE Schema/CST/唯一 verify →
//! AV 业务与绑定求值 → **967 auto binding/shim 受检 UI 调用**（`ui980`
//! provider，FFI 实际承载 UI 创建/提交/更新/关闭效果）→ UI-owned 更新/
//! typed 事件队列 → 现有 Iced 投影（`run_app` 泛型宿主，与 HELLO/r11 同链）。
//!
//! 接面（B-NATIVE-I 有界决定；967 冻结 AV 外呼可调用域 = view + 2 个 i32
//! 标量（`ViewI32I32ToI32`），view 物理帧首位）：
//! - `ui_open(op, instances, 0) -> session`（显式开 session，返回句柄）；
//! - `ui_apply_text(text, instance*1000+leaf, 0)` / `ui_apply_str(value,
//!   instance*1000+slot, 0)`——view 即载荷，按**当前 session**（单活动窗
//!   纪律；过期/未开 -2 具名拒）；
//! - `ui_apply_int(op, instance*1000+slot, value)` / `ui_apply_bool(op,
//!   instance*1000+slot, 0|1)`——同上（事件热路径）；
//! - `ui_poll_event(op, session, 0)` / `ui_close(op, session, 0)`——显式
//!   句柄（未知句柄 -1 具名拒）。
//!
//! 事件回 AV：窗口点击 → typed 队列（UI session 所有，世代纪律）→ tick 受控
//! 调用点 AV 外调 `ui_poll_event` 取事件 → `AceStateEngine::dispatch`（场景
//! 重放，全部求值在 AV；计数/业务绝不在 Rust）→ 状态差量经 AV 外调
//! `ui_apply_*` 提交实际 UI 更新。不能以独立 `text_byte_len` 调用充 UI
//! dispatch（反例 `p980_node_desktop_byte_len_lane_is_not_ui_dispatch`）。
//!
//! 与 r11 `ace_native` 的边界：r11 = 宿主直读 AV 快照（001 主题翻转面，保留
//! 自己的测试/exe）；本模块 = r13 新范围（通用项目入口 + 实际 UI-FFI 效果与
//! typed 事件）。001/002 完整主题/选择/全部组件与 Widgets 归 W1 B-AV。

use crate::ui::ace_package_bridge::{PackageEventInput, PackageFocusNotice, PackageFrame, PackageQueuedAction, PackageStateEngine};
use crate::ui::iced::image_surface_input::ImageSurfaceCarry;
use super::ace_route::{front_package_units, front_route_modules, package_candidates, RouteAction, RouteModulePaths, RouteNavigation, RouteTable};
use super::ace_route_desktop::{DesktopRoutes, RouteHandlers, RouteLink, route_links};
use crate::ui::ace_state_bridge::{
    AceStateEngine, AceStateValue, DispatchOutcome, EngineExecutionAudit,
    DropReason,
};
use crate::ui::{Component, View};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use ace_av::age_component::InitialValue as EngineInitialValue;
use ace_av::age_component::{
    compile_component_scenario_with_initials, observe_component_states_and_texts_flat,
    observe_component_texts, project_component_candidates, ComponentInitials, ComponentScenario,
    LegacyWidgetAlias,
};
use ace_core::source::packages::PackageModuleInput;

mod widgets;
mod package_images;
mod package_view;
pub use widgets::Action as WidgetAction;

/// PLAN-980 ④-A #2b：桥的后端面——组件面单源引擎 / AGE 包闭包引擎。
enum BridgeEngine {
    Component(AceStateEngine),
    Package(PackageStateEngine),
}

impl BridgeEngine {
    fn mount(
        &mut self,
        identity: &str,
        initials: Vec<(String, EngineInitialValue)>,
    ) -> Result<u64, String> {
        match self {
            Self::Component(e) => e.mount(identity, initials),
            Self::Package(e) => e.mount(identity, initials),
        }
    }

    fn read_all(&self, identity: &str) -> Option<Vec<(String, AceStateValue)>> {
        match self {
            Self::Component(e) => e.read_all(identity),
            Self::Package(e) => e.read_all(identity),
        }
    }

    fn dispatch(
        &mut self,
        identity: &str,
        msg: &str,
        args: Option<&[AceStateValue]>,
    ) -> Result<(DispatchOutcome, Option<Vec<String>>), String> {
        match self {
            Self::Component(e) => match args {
                Some(args) => e.dispatch_with_args(identity, msg, args),
                None => e.dispatch(identity, msg),
            }.map(|outcome| (outcome, None)),
            Self::Package(e) if e.has_checked_view() => match args {
                Some(args) => e.dispatch_with_args(identity, msg, args),
                None => e.dispatch(identity, msg),
            }.map(|outcome| (outcome, None)),
            Self::Package(e) => e.dispatch_snapshot(identity, msg, args).map(|snapshot| {
                (DispatchOutcome::Delivered(snapshot.states), Some(snapshot.texts))
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// ui980 provider：hooks trampoline（AV 侧受检车道；宿主侧状态由本 crate 持有）
// ---------------------------------------------------------------------------

/// UI-FFI 具名状态码（provider 文档与本模块共用；负值=具名拒绝）。
pub mod ui980_status {
    pub const OK: i32 = 0;
    pub const UNKNOWN_HANDLE: i32 = -1;
    pub const SESSION_CLOSED: i32 = -2;
    pub const BAD_SLOT: i32 = -3;
    pub const BAD_BYTES: i32 = -4;
    pub const BAD_OP: i32 = -5;
    pub const HOOKS_MISSING: i32 = -100;
    pub const HOOKS_DOUBLE_INSTALL: i32 = -101;
}

/// `ui980` provider 源（依赖无关；`ace_abi::rust_provider::build_source` 构建，
/// 注解即元数据）。宿主经 `ui980_install` 安装 hooks——同一 DLL 模块经
/// LoadLibraryW 引用计数共享，AV 受检调用与宿主读写同一份 statics。
///
/// 车道纪律（967 冻结）：AV 外呼域 = 1 个 view + ≤3 标量，view 在物理帧
/// 首位——text/str 的 view 即载荷，其余函数的 view 为车道标签（宿主逐符号
/// 校验，错标签具名拒 BAD_OP）。
pub const UI980_PROVIDER_SOURCE: &str = r#"//! auto-ffi provider ui980_native/1 target=x86_64-pc-windows-msvc profile=ui-native.v1
//!
//! PLAN-980 NATIVE (B-NATIVE-I) MIN-A-UI provider. A hooks trampoline: the
//! UI session state lives in the HOST process (auto-ui-runtime); the
//! annotated functions below are the AV-facing checked lane (967
//! auto-ffi domain: one borrowed UTF-8 view FIRST on the physical wire
//! plus up to three i32/bool scalars).

/// Host hook table (installed once per process via `ui980_install`).
#[repr(C)]
pub struct Ui980Hooks {
    pub open: unsafe extern "C" fn(op: *const u8, op_len: i32, instances: i32, reserved: i32) -> i32,
    pub apply_text: unsafe extern "C" fn(data: *const u8, byte_len: i32, instance_leaf: i32, reserved: i32) -> i32,
    pub apply_int: unsafe extern "C" fn(op: *const u8, op_len: i32, instance_slot: i32, value: i32) -> i32,
    pub apply_bool: unsafe extern "C" fn(op: *const u8, op_len: i32, instance_slot: i32, value: i32) -> i32,
    pub apply_str: unsafe extern "C" fn(data: *const u8, byte_len: i32, instance_slot: i32, reserved: i32) -> i32,
    pub poll_event: unsafe extern "C" fn(op: *const u8, op_len: i32, session: i32, reserved: i32) -> i32,
    pub close: unsafe extern "C" fn(op: *const u8, op_len: i32, session: i32, reserved: i32) -> i32,
}

static UI980_HOOKS: std::sync::RwLock<Option<Ui980Hooks>> = std::sync::RwLock::new(None);

/// Install the host hooks (host side only). 0 = installed; -100 = already
/// installed (uninstall first); -101 = null table.
#[no_mangle]
pub extern "C" fn ui980_install(hooks: *const Ui980Hooks) -> i32 {
    if hooks.is_null() {
        return -101;
    }
    let Ok(mut cell) = UI980_HOOKS.write() else { return -100 };
    if cell.is_some() {
        return -100;
    }
    unsafe { *cell = Some(std::ptr::read_unaligned(hooks)) };
    0
}

/// Drop the installed hooks (host session teardown). -100 = none installed.
#[no_mangle]
pub extern "C" fn ui980_uninstall() -> i32 {
    let Ok(mut cell) = UI980_HOOKS.write() else { return -100 };
    if cell.is_none() {
        return -100;
    }
    *cell = None;
    0
}

fn with_hooks(f: impl FnOnce(&Ui980Hooks) -> i32) -> i32 {
    match UI980_HOOKS.read() {
        Ok(guard) => match guard.as_ref() {
            Some(h) => f(h),
            None => -100,
        },
        Err(_) => -100,
    }
}

/// Open a UI session for `instances` component instances. Returns the
/// session handle (>= 1) or a negative named status.
// auto-ffi fn ui_open symbol=ui980.open.v1 form=pure ret=i32
pub fn ui_open(op: &str, instances: i32, reserved: i32) -> i32 {
    with_hooks(|h| unsafe { (h.open)(op.as_ptr(), op.len() as i32, instances, reserved) })
}

/// Apply one text-LEAF update to the CURRENT session (leaf table in
/// declaration order; bytes are copied to UI ownership before the
/// provider returns). `instance_leaf` = instance*1000 + leaf.
// auto-ffi fn ui_apply_text symbol=ui980.apply_text.v1 form=pure ret=i32
pub fn ui_apply_text(text: &str, instance_leaf: i32, reserved: i32) -> i32 {
    with_hooks(|h| unsafe {
        (h.apply_text)(text.as_ptr(), text.len() as i32, instance_leaf, reserved)
    })
}

/// Apply one int state-SLOT update to the CURRENT session (event hot
/// path; single active-window discipline). `instance_slot` =
/// instance*1000 + slot.
// auto-ffi fn ui_apply_int symbol=ui980.apply_int.v1 form=pure ret=i32
pub fn ui_apply_int(op: &str, instance_slot: i32, value: i32) -> i32 {
    with_hooks(|h| unsafe { (h.apply_int)(op.as_ptr(), op.len() as i32, instance_slot, value) })
}

/// Apply one bool state-SLOT update to the CURRENT session (value rides
/// as 0/1). `instance_slot` = instance*1000 + slot.
// auto-ffi fn ui_apply_bool symbol=ui980.apply_bool.v1 form=pure ret=i32
// PLAN-104：value 走 **i32 标量**（0/1）而不是 bool——ABI 的 bool 参与调用时
// packed 槽位实参错位（真窗实证：调用串 slot=6，hook 实收 instance_slot=48），
// 与桥内 hook 表的 i32 签名也对齐；hook 侧按 `value != 0` 解释。
pub fn ui_apply_bool(op: &str, instance_slot: i32, value: i32) -> i32 {
    with_hooks(|h| unsafe { (h.apply_bool)(op.as_ptr(), op.len() as i32, instance_slot, value) })
}

/// Apply one str state-SLOT update to the CURRENT session (field table;
/// distinct lane from text leaves). `instance_slot` = instance*1000 + slot.
// auto-ffi fn ui_apply_str symbol=ui980.apply_str.v1 form=pure ret=i32
pub fn ui_apply_str(value: &str, instance_slot: i32, reserved: i32) -> i32 {
    with_hooks(|h| unsafe {
        (h.apply_str)(value.as_ptr(), value.len() as i32, instance_slot, reserved)
    })
}

/// Poll one typed event from the session: 0 = none; > 0 =
/// (instance+1)*1000 + (target+1); negative = named refusal (stale or
/// closed sessions refuse before dequeue).
// auto-ffi fn ui_poll_event symbol=ui980.poll_event.v1 form=pure ret=i32
pub fn ui_poll_event(op: &str, session: i32, reserved: i32) -> i32 {
    with_hooks(|h| unsafe { (h.poll_event)(op.as_ptr(), op.len() as i32, session, reserved) })
}

/// Close the session: later apply/poll refuse with the stale status and
/// late events never revive it. 0 = closed; negative = named refusal.
// auto-ffi fn ui_close symbol=ui980.close.v1 form=pure ret=i32
pub fn ui_close(op: &str, session: i32, reserved: i32) -> i32 {
    with_hooks(|h| unsafe { (h.close)(op.as_ptr(), op.len() as i32, session, reserved) })
}
"#;

/// ui980 provider 的注册名（E0 请求 `provider` 字段使用）。
pub const UI980_PROVIDER_NAME: &str = "p980_ui980_native";

/// ui980 构建根（同 HELLO F0 纪律：环境显式指定，否则本仓 target/ 下；
/// 首次构建后走 build_source 新鲜度缓存，测试/子进程零 rustc）。
pub fn stable_ui980_root() -> PathBuf {
    if let Ok(p) = std::env::var("ACE_NODE_DESKTOP_ROOT") {
        return PathBuf::from(p);
    }
    let mut p = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    while p.file_name().map(|n| n != "target").unwrap_or(true) && p.pop() {}
    p.join("ace-node-desktop")
}

/// Returns the process's registered provider, preparing it once when needed.
static UI980_PROVIDER_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn ensure_ui980_provider() -> Result<PathBuf, String> {
    let mut cached = UI980_PROVIDER_PATH.lock()
        .map_err(|_| "ui980 provider path lock was poisoned".to_string())?;
    if let Some(path) = cached.as_ref() {
        return Ok(path.clone());
    }
    auto_lang_ace_host::install().map_err(|e| format!("ace host install: {e}"))?;
    let root = stable_ui980_root();
    let path = match std::env::var("ACE_NODE_DESKTOP_PREPARED_RUSTC") {
        Ok(rustc) => auto_lang_ace_host::register_prepared_source_provider(
            UI980_PROVIDER_NAME,
            &root,
            UI980_PROVIDER_SOURCE,
            &rustc,
        )?,
        Err(std::env::VarError::NotPresent) => auto_lang_ace_host::register_source_provider(
            UI980_PROVIDER_NAME,
            &root,
            UI980_PROVIDER_SOURCE,
        )?,
        Err(error) => return Err(error.to_string()),
    };
    *cached = Some(path.clone());
    Ok(path)
}

// ---------------------------------------------------------------------------
// UI session host：显示模型 + typed 事件队列 + 证据（UI 所有，世代纪律）
// ---------------------------------------------------------------------------

/// 一次实际 UI 提交的证据行。
#[derive(Clone, Debug)]
pub struct AppliedUpdate {
    pub instance: usize,
    pub kind: &'static str,
    pub slot: usize,
    pub value: String,
}

/// 一个 UI session 的宿主侧状态（窗口渲染与证据的直接来源）。
#[derive(Debug, Default)]
pub struct HostSession {
    pub id: i32,
    pub generation: u64,
    pub instances: usize,
    /// 每实例文本叶（叶声明序）。
    pub texts: Vec<Vec<String>>,
    /// 每实例状态槽（字段声明序）。
    pub states: Vec<Vec<(String, String)>>,
    /// PLAN-104：每实例、每文本叶的可见性（条件分支求值；叶序与 texts 一致）。
    pub visibility: Vec<Vec<bool>>,
    /// PLAN-104：元素 label 展示行（`h2 .title` / `span "✓"` 族；仅显示）。
    pub labels: Vec<Vec<String>>,
    /// 事件目标（声明序）：(label, msg)。
    pub buttons: Vec<(String, String)>,
    /// PLAN-980 B-AV U01（输入组件面）：文本输入目标（声明序，接 buttons
    /// 之后编号）：(绑定字段名, 数值型？)。渲染值取该实例状态行——受控单
    /// 向数据流（键入→typed 事件→AV 重放→apply→回显）。
    pub inputs: Vec<(String, bool)>,
    widget_tree: Option<Arc<[ace_core::source::age_component::ComponentViewItem]>>,
    widget_instances: Vec<widgets::State>,
    widget_values: Vec<Arc<[(String, AceStateValue)]>>,
    package_frames: Vec<Arc<PackageFrame>>,
    package_images: Option<Arc<package_images::PackageImages>>,
    focus_epoch: Arc<AtomicU64>,
    focus_request: Option<crate::ui::component::FocusRequest>,
    package_events: VecDeque<QueuedPackageEvent>,
    package_workers_pending: Vec<bool>,
    package_focus_notices: VecDeque<(usize, u64, PackageFocusNotice)>,
    queue: VecDeque<(usize, usize)>,
    closed: bool,
    /// 证据：实际 UI 提交计数/轨迹、poll 计数、late/stale 拒绝计数。
    pub applies: u64,
    pub applied: Vec<AppliedUpdate>,
    pub polled: u64,
    consumed_events: usize,
    pub late_dropped: u64,
    pub stale_rejected: u64,
}

#[derive(Debug)]
struct QueuedPackageEvent {
    instance: usize,
    revision: u64,
    target: usize,
    action: Option<PackageQueuedAction>,
    input: PackageEventInput,
}

impl HostSession {
    fn new(id: i32, generation: u64, instances: usize) -> Self {
        Self {
            id,
            generation,
            instances,
            texts: vec![Vec::new(); instances],
            states: vec![Vec::new(); instances],
            visibility: vec![Vec::new(); instances],
            labels: vec![Vec::new(); instances],
            buttons: Vec::new(),
            inputs: Vec::new(),
            widget_tree: None,
            widget_instances: Vec::new(),
            widget_values: Vec::new(),
            package_frames: Vec::new(),
            package_images: None,
            focus_epoch: Arc::new(AtomicU64::new(0)),
            focus_request: None,
            package_events: VecDeque::new(),
            package_workers_pending: vec![false; instances],
            package_focus_notices: VecDeque::new(),
            queue: VecDeque::new(),
            closed: false,
            applies: 0,
            applied: Vec::new(),
            polled: 0,
            consumed_events: 0,
            late_dropped: 0,
            stale_rejected: 0,
        }
    }

    /// Checked view and click targets for the currently published revision.
    pub fn package_frame(&self, instance: usize) -> Option<&PackageFrame> {
        self.package_frames.get(instance).map(Arc::as_ref)
    }

    fn publish_focus(&mut self, target: Option<String>) {
        let expected = self.focus_epoch.fetch_add(1, Ordering::AcqRel).wrapping_add(1);
        self.focus_request = target.map(|target| {
            crate::ui::component::FocusRequest::new(target, Arc::clone(&self.focus_epoch), expected)
        });
    }
}

/// 宿主全局：session 表 + 当前 session + 晚到事件靶与无处路由计数。
struct NodeDesktopHost {
    sessions: Vec<HostSession>,
    next_session: i32,
    next_generation: u64,
    current: Option<i32>,
    /// 关闭 session 后保留其 id（late 事件的定向靶）。
    stale_targets: Vec<i32>,
    /// 无 session 可路由的事件（尚未 open 或已全关）。
    no_session_events: u64,
}

impl NodeDesktopHost {
    fn new() -> Self {
        Self {
            sessions: Vec::new(),
            next_session: 1,
            next_generation: 1,
            current: None,
            stale_targets: Vec::new(),
            no_session_events: 0,
        }
    }

    fn get(&mut self, id: i32) -> Option<&mut HostSession> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }
}

static HOST: OnceLock<RwLock<NodeDesktopHost>> = OnceLock::new();
/// 窗口渲染帧计数（真实渲染观察面）。
pub static NODE_DESKTOP_FRAMES: AtomicUsize = AtomicUsize::new(0);

fn host() -> &'static RwLock<NodeDesktopHost> {
    HOST.get_or_init(|| RwLock::new(NodeDesktopHost::new()))
}

/// 当前 session 的只读快照（窗口 view 消费）。
pub fn current_session<R>(f: impl FnOnce(&HostSession) -> R) -> Option<R> {
    let h = host().read().ok()?;
    let id = h.current?;
    h.sessions.iter().find(|s| s.id == id).map(f)
}

fn select_host_session(id: i32) {
    let mut host = host().write().unwrap();
    if host.get(id).is_some_and(|session| !session.closed) {
        host.current = Some(id);
    }
}

pub fn push_session_event(id: i32, instance: usize, target: usize) {
    let mut host = host().write().unwrap();
    if let Some(session) = host.get(id) {
        if session.closed {
            session.late_dropped += 1;
        } else {
            session.queue.push_back((instance, target));
        }
    }
}

fn push_package_event(id: i32, instance: usize, revision: u64, target: usize, input: PackageEventInput) {
    let mut host = host().write().unwrap();
    if let Some(session) = host.get(id) {
        if session.closed {
            session.late_dropped += 1;
        } else if let Some(frame) = session.package_frame(instance).filter(|frame| {
            frame.revision == revision && frame.accepts_target(target, &input)
        }) {
            // Ordinary clicks may wait for the worker already processing this
            // frame. Clicks queued before a worker starts retain their frame-only semantics.
            let action = frame.queued_action(target).filter(|_| {
                session.package_workers_pending[instance]
                    || !frame.actions()[target].source.key.eq_ignore_ascii_case("onclick")
            });
            session.package_events.push_back(QueuedPackageEvent {
                instance, revision, target, action, input,
            });
        } else {
            session.stale_rejected += 1;
        }
    }
}

fn push_package_focus(id: i32, instance: usize, revision: u64, control: String, gained: bool) {
    let mut host = host().write().unwrap();
    let Some(session) = host.get(id) else { return };
    if session.closed {
        session.late_dropped += 1;
        return;
    }
    let notice = session.package_frame(instance)
        .filter(|frame| frame.revision == revision)
        .and_then(|frame| frame.focus_notice(control, gained));
    if let Some(notice) = notice {
        session.package_focus_notices.push_back((instance, revision, notice));
    } else {
        session.stale_rejected += 1;
    }
}

fn push_script_click(id: i32, instance: usize, target: usize) {
    let revision = host().write().unwrap().get(id)
        .and_then(|session| session.package_frame(instance))
        .map(|frame| frame.revision);
    if let Some(revision) = revision {
        push_package_event(id, instance, revision, target, PackageEventInput::Activation);
    } else {
        push_session_event(id, instance, target);
    }
}

/// Admit package input through the same revisioned queue in both desktop hosts.
pub(crate) fn queue_package_message(message: NodeDesktopMsg) -> bool {
    match message {
        NodeDesktopMsg::PackageClick { session, instance, revision, target } => {
            push_package_event(session, instance, revision, target, PackageEventInput::Activation);
        }
        NodeDesktopMsg::PackagePointer { session, instance, revision, target, x, y } => {
            push_package_event(session, instance, revision, target, PackageEventInput::Pointer { x, y });
        }
        NodeDesktopMsg::PackageText { session, instance, revision, target, text } => {
            push_package_event(session, instance, revision, target, PackageEventInput::Text(text));
        }
        NodeDesktopMsg::PackageImageSurface { session, instance, revision, target, carry } => {
            push_package_event(session, instance, revision, target, PackageEventInput::ImageSurface(carry));
        }
        NodeDesktopMsg::PackageFocus { session, instance, revision, control, gained } => {
            push_package_focus(session, instance, revision, control, gained);
        }
        _ => return false,
    }
    true
}

pub(crate) fn selected_package_view() -> Option<View<NodeDesktopMsg>> {
    let frame = current_session(|session| {
        (!session.package_frames.is_empty()).then(|| host_frame(session))
    }).flatten()?;
    let mut dialogs = Vec::new();
    let content = render_host_frame(frame, None, false, &mut dialogs);
    Some(View::col().child(content).children(dialogs).build())
}

/// 向当前 session 推入 typed 事件（窗口点击/脚本注入的同一队列边界）。
/// 无活动 session 或已关闭 → 丢弃并计数（不复活、不静默成功）。
pub fn push_event(instance: usize, target: usize) {
    if let Ok(mut h) = host().write() {
        let routed = h.current.is_some_and(|id| {
            h.get(id).is_some_and(|s| {
                if s.closed {
                    false
                } else {
                    s.queue.push_back((instance, target));
                    true
                }
            })
        });
        if !routed {
            let stale = h.stale_targets.last().copied();
            match stale {
                Some(id) => {
                    if let Some(s) = h.get(id) {
                        s.late_dropped += 1;
                    }
                }
                None => h.no_session_events += 1,
            }
        }
    }
}

/// 向指定（旧代）session 推 late 事件：须被丢弃并计数（世代纪律反例靶）。
pub fn push_late_event(stale_id: i32) {
    if let Ok(mut h) = host().write() {
        match h.get(stale_id) {
            Some(s) => s.late_dropped += 1,
            None => h.no_session_events += 1,
        }
    }
}

/// 最近一个已关闭 session 的 id（late 事件脚本用）。
pub fn last_closed_session() -> Option<i32> {
    let h = host().read().ok()?;
    h.stale_targets.last().copied()
}

/// 单个 session 的证据快照（CPU 测试/宿主观察面；不含队列内部）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostSessionStats {
    pub id: i32,
    pub generation: u64,
    pub closed: bool,
    pub applies: u64,
    pub polled: u64,
    pub late_dropped: u64,
    pub stale_rejected: u64,
}

/// 读取指定 session 的证据快照（不存在 → None）。
pub fn host_session_stats(id: i32) -> Option<HostSessionStats> {
    let h = host().read().ok()?;
    h.sessions
        .iter()
        .find(|s| s.id == id)
        .map(|s| HostSessionStats {
            id: s.id,
            generation: s.generation,
            closed: s.closed,
            applies: s.applies,
            polled: s.polled,
            late_dropped: s.late_dropped,
            stale_rejected: s.stale_rejected,
        })
}

/// 读取指定 session 的显示模型（texts+states；窗口渲染同源数据）。
pub fn host_session_display(id: i32) -> Option<(Vec<Vec<String>>, Vec<Vec<(String, String)>>)> {
    let h = host().read().ok()?;
    h.sessions
        .iter()
        .find(|s| s.id == id)
        .map(|s| (s.texts.clone(), s.states.clone()))
}

pub(crate) fn host_session_text_applies(id: i32) -> Vec<String> {
    let Ok(host) = host().read() else {
        return Vec::new();
    };
    host.sessions
        .iter()
        .find(|s| s.id == id)
        .into_iter()
        .flat_map(|s| &s.applied)
        .filter(|apply| apply.kind == "text")
        .map(|apply| apply.value.clone())
        .collect()
}

// --- hooks 实现（provider DLL 经函数指针回调；签名与 Ui980Hooks 逐一同型）---

fn tag_matches(op: *const u8, op_len: i32, expected: &[u8]) -> bool {
    if op_len < 0 || op_len as usize != expected.len() {
        return false;
    }
    unsafe { std::slice::from_raw_parts(op, op_len as usize) == expected }
}

unsafe extern "C" fn hook_open(op: *const u8, op_len: i32, instances: i32, _reserved: i32) -> i32 {
    if !tag_matches(op, op_len, b"open") {
        return ui980_status::BAD_OP;
    }
    let n = if instances < 1 { 1 } else { instances as usize };
    let mut h = host().write().unwrap();
    let id = h.next_session;
    h.next_session += 1;
    let generation = h.next_generation;
    h.next_generation += 1;
    h.sessions.push(HostSession::new(id, generation, n));
    h.current = Some(id);
    id
}

fn utf8_payload(data: *const u8, byte_len: i32) -> Result<String, i32> {
    if byte_len < 0 {
        return Err(ui980_status::BAD_BYTES);
    }
    let bytes = unsafe { std::slice::from_raw_parts(data, byte_len as usize) };
    std::str::from_utf8(bytes)
        .map(|t| t.to_string())
        .map_err(|_| ui980_status::BAD_BYTES)
}

/// 打包位解码：instance*1000 + slot/leaf（MIN-A 界：实例 ≤4、槽 <1000）。
fn unpack(a: i32) -> Result<(usize, usize), i32> {
    if a < 0 {
        return Err(ui980_status::BAD_SLOT);
    }
    let v = a as usize;
    Ok((v / 1000, v % 1000))
}

/// 当前 session 槽（热路径/文本/str 车道共用；句柄语义=当前活动 session）。
fn current_or_refuse(h: &mut NodeDesktopHost) -> Result<(), i32> {
    let current = h.current;
    match current {
        Some(id) => {
            if h.get(id).is_some_and(|s| !s.closed) {
                Ok(())
            } else {
                if let Some(s) = h.get(id) {
                    s.stale_rejected += 1;
                }
                Err(ui980_status::SESSION_CLOSED)
            }
        }
        None => Err(ui980_status::SESSION_CLOSED),
    }
}

unsafe extern "C" fn hook_apply_text(
    data: *const u8,
    byte_len: i32,
    instance_leaf: i32,
    _reserved: i32,
) -> i32 {
    let text = match utf8_payload(data, byte_len) {
        Ok(t) => t,
        Err(code) => return code,
    };
    let (i, leaf) = match unpack(instance_leaf) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let mut h = host().write().unwrap();
    if let Err(code) = current_or_refuse(&mut h) {
        return code;
    }
    let id = h.current.unwrap();
    let Some(s) = h.get(id) else {
        return ui980_status::UNKNOWN_HANDLE;
    };
    if i >= s.instances || leaf >= s.texts[i].len() {
        return ui980_status::BAD_SLOT;
    }
    s.texts[i][leaf] = text.clone();
    s.applies += 1;
    s.applied.push(AppliedUpdate {
        instance: i,
        kind: "text",
        slot: leaf,
        value: text,
    });
    ui980_status::OK
}

/// 当前 session 的 int 槽提交（事件热路径；句柄语义=当前活动 session）。
unsafe extern "C" fn hook_apply_int(
    op: *const u8,
    op_len: i32,
    instance_slot: i32,
    value: i32,
) -> i32 {
    if !tag_matches(op, op_len, b"apply-int") {
        return ui980_status::BAD_OP;
    }
    let (i, slot) = match unpack(instance_slot) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let mut h = host().write().unwrap();
    if let Err(code) = current_or_refuse(&mut h) {
        return code;
    }
    let id = h.current.unwrap();
    let Some(s) = h.get(id) else {
        return ui980_status::UNKNOWN_HANDLE;
    };
    if i >= s.instances || slot >= s.states[i].len() {
        return ui980_status::BAD_SLOT;
    }
    s.states[i][slot].1 = value.to_string();
    s.applies += 1;
    s.applied.push(AppliedUpdate {
        instance: i,
        kind: "int",
        slot,
        value: value.to_string(),
    });
    ui980_status::OK
}

unsafe extern "C" fn hook_apply_bool(
    op: *const u8,
    op_len: i32,
    instance_slot: i32,
    value: i32,
) -> i32 {
    if !tag_matches(op, op_len, b"apply-bool") {
        return ui980_status::BAD_OP;
    }
    let (i, slot) = match unpack(instance_slot) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let mut h = host().write().unwrap();
    if let Err(code) = current_or_refuse(&mut h) {
        return code;
    }
    let id = h.current.unwrap();
    let Some(s) = h.get(id) else {
        return ui980_status::UNKNOWN_HANDLE;
    };
    if i >= s.instances || slot >= s.states[i].len() {
        return ui980_status::BAD_SLOT;
    }
    let shown = if value != 0 { "true" } else { "false" };
    s.states[i][slot].1 = shown.to_string();
    s.applies += 1;
    s.applied.push(AppliedUpdate {
        instance: i,
        kind: "bool",
        slot,
        value: shown.to_string(),
    });
    ui980_status::OK
}

unsafe extern "C" fn hook_apply_str(
    data: *const u8,
    byte_len: i32,
    instance_slot: i32,
    _reserved: i32,
) -> i32 {
    let text = match utf8_payload(data, byte_len) {
        Ok(t) => t,
        Err(code) => return code,
    };
    let (i, slot) = match unpack(instance_slot) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let mut h = host().write().unwrap();
    if let Err(code) = current_or_refuse(&mut h) {
        return code;
    }
    let id = h.current.unwrap();
    let Some(s) = h.get(id) else {
        return ui980_status::UNKNOWN_HANDLE;
    };
    if i >= s.instances || slot >= s.states[i].len() {
        return ui980_status::BAD_SLOT;
    }
    s.states[i][slot].1 = text.clone();
    s.applies += 1;
    s.applied.push(AppliedUpdate {
        instance: i,
        kind: "str",
        slot,
        value: text,
    });
    ui980_status::OK
}

unsafe extern "C" fn hook_poll_event(
    op: *const u8,
    op_len: i32,
    session: i32,
    _reserved: i32,
) -> i32 {
    if !tag_matches(op, op_len, b"poll") {
        return ui980_status::BAD_OP;
    }
    let mut h = host().write().unwrap();
    let Some(s) = h.get(session) else {
        return ui980_status::UNKNOWN_HANDLE;
    };
    if s.closed {
        s.stale_rejected += 1;
        return ui980_status::SESSION_CLOSED;
    }
    s.polled += 1;
    match s.queue.pop_front() {
        Some((instance, target)) => {
            s.consumed_events += 1;
            ((instance + 1) * 1000 + (target + 1)) as i32
        }
        None => 0,
    }
}

unsafe extern "C" fn hook_close(op: *const u8, op_len: i32, session: i32, _reserved: i32) -> i32 {
    if !tag_matches(op, op_len, b"close") {
        return ui980_status::BAD_OP;
    }
    close_host_session(session)
}

fn close_host_session(session: i32) -> i32 {
    let mut h = host().write().unwrap();
    let Some(s) = h.get(session) else {
        return ui980_status::UNKNOWN_HANDLE;
    };
    if s.closed {
        s.stale_rejected += 1;
        return ui980_status::SESSION_CLOSED;
    }
    s.closed = true;
    s.queue.clear();
    if let Some(tree) = s.widget_tree.take() {
        widgets::State::release_session(session, &tree, &mut s.widget_instances);
    }
    s.widget_values.clear();
    s.package_frames.clear();
    s.package_images = None;
    s.publish_focus(None);
    s.package_events.clear();
    s.package_focus_notices.clear();
    if h.current == Some(session) {
        h.current = None;
        h.stale_targets.push(session);
    }
    ui980_status::OK
}

/// 宿主侧 hooks 原型（与 provider 源内 `Ui980Hooks` 逐字段同型）。
#[repr(C)]
struct Ui980HostHooks {
    open: unsafe extern "C" fn(*const u8, i32, i32, i32) -> i32,
    apply_text: unsafe extern "C" fn(*const u8, i32, i32, i32) -> i32,
    apply_int: unsafe extern "C" fn(*const u8, i32, i32, i32) -> i32,
    apply_bool: unsafe extern "C" fn(*const u8, i32, i32, i32) -> i32,
    apply_str: unsafe extern "C" fn(*const u8, i32, i32, i32) -> i32,
    poll_event: unsafe extern "C" fn(*const u8, i32, i32, i32) -> i32,
    close: unsafe extern "C" fn(*const u8, i32, i32, i32) -> i32,
}

/// 持有 provider DLL 的模块句柄（进程存活期保持 statics；仅承载 Win32
/// 模块句柄，无别名可变状态）。
struct HeldLibrary(#[allow(dead_code)] ace_abi::rust_provider::RawLibrary);
unsafe impl Send for HeldLibrary {}
unsafe impl Sync for HeldLibrary {}

static UI980_LIB: OnceLock<HeldLibrary> = OnceLock::new();
static HOOKS_INSTALLED: AtomicU64 = AtomicU64::new(0);

/// 构建/注册 provider 并安装宿主 hooks（幂等；首调后零 rustc）。
pub fn install_ui980_lane() -> Result<(), String> {
    if HOOKS_INSTALLED.load(Ordering::SeqCst) == 1 {
        return Ok(());
    }
    let cdylib = ensure_ui980_provider()?;
    let lib = ace_abi::rust_provider::RawLibrary::load(&cdylib)
        .map_err(|e| format!("ui980 dll load: {e}"))?;
    let install_sym = lib
        .symbol("ui980_install")
        .ok_or_else(|| "ui980_install export missing".to_string())?;
    let install: unsafe extern "C" fn(*const Ui980HostHooks) -> i32 =
        unsafe { std::mem::transmute(install_sym) };
    let table = Ui980HostHooks {
        open: hook_open,
        apply_text: hook_apply_text,
        apply_int: hook_apply_int,
        apply_bool: hook_apply_bool,
        apply_str: hook_apply_str,
        poll_event: hook_poll_event,
        close: hook_close,
    };
    let rc = unsafe { install(&table) };
    if rc != ui980_status::OK {
        return Err(format!("ui980_install refused: {rc}"));
    }
    let _ = UI980_LIB.set(HeldLibrary(lib));
    HOOKS_INSTALLED.store(1, Ordering::SeqCst);
    Ok(())
}

// ---------------------------------------------------------------------------
// 桥：装载（Schema/verify/AV 真值）→ open → 受控事件循环（poll→AV 重放→apply）
// ---------------------------------------------------------------------------

/// 事件目标（按钮）规格：label + msg 名（view 树投影提取）。
#[derive(Clone, Debug)]
pub struct ButtonSpec {
    pub label: String,
    pub msg: String,
    payload: Option<Vec<ButtonPayload>>,
}

#[derive(Clone, Debug)]
enum ButtonPayload {
    Value(AceStateValue),
    State(String),
}

impl ButtonPayload {
    fn from_arg(arg: &ace_core::source::ast::Arg) -> Result<Self, String> {
        use ace_core::source::ast::{Arg, ExprKind};
        let Arg::Pos(expr) = arg else {
            return Err("ace.desktop.event-payload: expected a positional argument".into());
        };
        let value = match &expr.kind {
            ExprKind::StrLit(value) => AceStateValue::Str(value.clone()),
            ExprKind::IntLit(value) => AceStateValue::Int(*value),
            ExprKind::BoolLit(value) => AceStateValue::Bool(*value),
            ExprKind::Var(name) => return Ok(Self::State(name.name.clone())),
            ExprKind::Field { base, name }
                if matches!(&base.kind, ExprKind::Var(id) if id.name == "self") =>
            {
                return Ok(Self::State(name.name.clone()));
            }
            _ => return Err(format!(
                "ace.desktop.event-payload: cannot evaluate button argument @{}..{}",
                expr.span.start, expr.span.end,
            )),
        };
        Ok(Self::Value(value))
    }

    fn read(&self, state: &[(String, AceStateValue)]) -> Result<AceStateValue, String> {
        match self {
            Self::Value(value) => Ok(value.clone()),
            Self::State(field) => state
                .iter()
                .find(|(name, _)| name == field)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| format!("ace.desktop.event-payload: no current state `{field}`")),
        }
    }
}

/// PLAN-980 B-AV U01：**文本输入目标**规格——`input (value: .field)
/// { oninput: .Msg }`（值绑定=受控回显源；oninput=typed 事件消息）。
/// 载荷类型由绑定字段初值决定（int=数值输入：键入文本在桥面解析，
/// 非数值=具名拒零效果；str=文本输入原样入队）。
#[derive(Clone, Debug)]
pub struct InputSpec {
    pub field: String,
    pub msg: String,
    pub numeric: bool,
}

/// 一步的结果（测试/证据消费）。
#[derive(Clone, Debug, PartialEq)]
pub enum StepOutcome {
    Idle,
    Dispatched {
        instance: usize,
        msg: String,
        states: Vec<(String, String)>,
    },
    DispatchRefused(String),
    LaneError(String),
}

/// 事件脚本条目（e2e/CPU 驱动同一受控边界；tick 从 1 计）。
#[derive(Clone, Debug)]
pub enum ScriptStep {
    Click {
        instance: usize,
        target: usize,
    },
    ClickPage { instance: usize, target: usize },
    /// PLAN-980 B-AV U01：文本输入事件（键入全文；目标=输入目标编号，
    /// 接 buttons 之后）。真实键入（窗口）与脚本注入同一边界。
    TypeText {
        instance: usize,
        target: usize,
        text: String,
    },
    Close,
    Reopen,
    /// 向最近关闭的 session 定向晚到事件（世代纪律反例靶）。
    LateClick,
}

pub struct NodeDesktopBridge {
    path: String,
    node_name: String,
    /// None = 静态面（无 model/msg 的 Hello 类源：文本经静态观察面，无事件）。
    engine: Option<BridgeEngine>,
    identities: Vec<String>,
    pub session: Option<i32>,
    instances: usize,
    buttons: Vec<ButtonSpec>,
    /// PLAN-980 B-AV U01：输入目标（目标编号=buttons.len() 起）。
    inputs: Vec<InputSpec>,
    /// 输入事件的键入全文（FIFO，与 provider 事件队列 (instance, target)
    /// 一一配对——poll 取码、桥面取文）。
    pending_texts: VecDeque<(usize, usize, String)>,
    deferred_snapshot: bool,
    text_leaf_count: usize,
    last_texts: Vec<Vec<String>>,
    last_states: Vec<Vec<(String, AceStateValue)>>,
    field_names: Vec<String>,
    /// 证据：dispatch 轨迹（instance, msg, 状态快照）与错误。
    pub dispatch_log: Vec<(usize, String, Vec<(String, String)>)>,
    /// PLAN-108：组件 Tick 合成的每实例毫秒累积（BridgeTick 20ms 拍）。
    tick_accum_ms: Vec<u32>,
    pub errors: Vec<String>,
    script: Vec<(usize, ScriptStep)>,
    tick_no: usize,
    /// PLAN-104：装载时的 view 树（条件可见性求值用；叶序与 texts 一致）。
    view_items: Vec<ace_core::source::age_component::ComponentViewItem>,
    widget_initial_bools: std::collections::BTreeMap<String, bool>,
    public_assets: Option<Arc<std::path::PathBuf>>,
    /// PLAN-104：文本叶来自循环体模板展开（视图数据，非状态派生）——不参与
    /// 状态驱动的叶刷新。（PLAN-1106 T-05：源串与场景 op 历史随增量会话
    /// 接线退役——文本叶改由会话纯读 `AceStateEngine::read_texts`。）
    expanded_static: bool,
    routes: Option<Box<DesktopRoutes>>,
    route_actions: std::collections::BTreeMap<usize, RouteAction>,
    route_button_spans: Vec<usize>,
    package_route_handlers: Option<RouteHandlers>,
    shortcuts: std::collections::BTreeMap<String, String>,
    pending_shortcuts: VecDeque<String>,
    route_request: Option<RouteNavigation>,
    /// PLAN-113 T-03：真实 SQLite 宿主桥（V-03 冻结合同；opt-in，测试入口
    /// `--sqlite-db` 安装；默认 None 零行为变化）。
    sqlite_bridge: Option<crate::ui::ace_sqlite_bridge::SqliteHostBridge>,
}

/// view 树投影：文本叶计数 + 按钮目标（label=StrLit、onclick=.Msg）+
/// 输入目标（value 绑定字段 + oninput 消息）。
#[derive(Default)]
struct ViewShape {
    text_leaf_count: usize,
    buttons: Vec<ButtonSpec>,
    inputs: Vec<InputSpec>,
    button_spans: Vec<usize>,
}

fn project_view_shape(
    items: &[ace_core::source::age_component::ComponentViewItem],
    shape: &mut ViewShape,
    in_loop: bool,
) -> Result<(), String> {
    use ace_core::source::age_component::ComponentViewItem;
    use ace_core::source::ast::ExprKind;
    for item in items {
        match item {
            ComponentViewItem::Text(_) => shape.text_leaf_count += 1,
            ComponentViewItem::Entry(_) => {}
            // PLAN-104：条件两支的叶都计入（与 AV flat 观察的平铺序一致）；
            // 可见性在 `mark_leaf_visibility` 按实例状态求值。
            ComponentViewItem::Cond(c) => {
                project_view_shape(&c.then_items, shape, in_loop)?;
                project_view_shape(&c.else_items, shape, in_loop)?;
            }
            ComponentViewItem::For(f) => {
                project_view_shape(&f.items, shape, true)?;
            }
            ComponentViewItem::Element(el) => {
                if el.tag == "text" {
                    shape.text_leaf_count += usize::from(el.label.is_some());
                } else if el.tag == "input" {
                    // PLAN-980 B-AV U01：`input (value: .field)
                    // { oninput: .Msg }`——值绑定（Entry key="value"，Expr
                    // = .field 字段引用）+ 事件绑定（key 前缀 on）。numeric
                    // 由字段初值定（load 阶段补）。
                    let mut field = None;
                    let mut msg = None;
                    for sub in &el.items {
                        if let ComponentViewItem::Entry(entry) = sub {
                            if let ace_core::source::age_component::ComponentEntryValue::Expr(
                                expr,
                            ) = &entry.value
                            {
                                if let ExprKind::Field { base, name } = &expr.kind {
                                    if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") {
                                        if entry.is_event {
                                            msg = Some(name.name.clone());
                                        } else if entry.key == "value" {
                                            field = Some(name.name.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let (Some(field), Some(msg)) = (field, msg) {
                        shape.inputs.push(InputSpec {
                            field,
                            msg,
                            numeric: false,
                        });
                    }
                } else if el.tag == "button" {
                    let label = el
                        .label
                        .as_ref()
                        .and_then(|e| match &e.kind {
                            ExprKind::StrLit(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let mut msg = None;
                    let mut payload = None;
                    for sub in &el.items {
                        if let ComponentViewItem::Entry(entry) = sub {
                            if entry.is_event && entry.key == "onclick" {
                                if let Some((name, args)) = entry.message_call() {
                                    // PLAN-113（026 真语料）：循环体内带载荷的
                                    // 按钮/复选框（表格删除行 `.DeleteRow(i)`）——
                                    // 载荷是逐行动态值，桥面平铺形状无法静态承载。
                                    // 跳过该目标（不整体拒绝装载）：渲染不受影响，
                                    // 仅该交互不可脚本化；行为证据经场景派发/A2Vue。
                                    if in_loop && !args.is_empty() {
                                        continue;
                                    }
                                    msg = Some(name.name.clone());
                                    payload = Some(args.iter().map(ButtonPayload::from_arg)
                                        .collect::<Result<Vec<_>, _>>()?);
                                    continue;
                                }
                                if let ace_core::source::age_component::ComponentEntryValue::Expr(
                                    expr,
                                ) = &entry.value
                                {
                                    if let ExprKind::Field { base, name } = &expr.kind {
                                        if matches!(&base.kind, ExprKind::Var(v) if v.name == "self")
                                        {
                                            msg = Some(name.name.clone());
                                            payload = None;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(msg) = msg {
                        shape.buttons.push(ButtonSpec { label, msg, payload });
                        shape.button_spans.push(el.span.start);
                    }
                } else if el.tag == "checkbox" {
                    // PLAN-105 W107：`checkbox { onclick: .Msg }` 与 button 同为
                    // 可点击事件目标（013 的 ToggleAll 全选框）——此前落入通用
                    // 递归，checkbox 事件在脚本面不可达。
                    let mut msg = None;
                    let mut payload = None;
                    for sub in &el.items {
                        if let ComponentViewItem::Entry(entry) = sub {
                            if entry.is_event && entry.key == "onclick" {
                                if let Some((name, args)) = entry.message_call() {
                                    // PLAN-113（026 真语料）：循环体内带载荷的
                                    // 按钮/复选框（表格删除行 `.DeleteRow(i)`）——
                                    // 载荷是逐行动态值，桥面平铺形状无法静态承载。
                                    // 跳过该目标（不整体拒绝装载）：渲染不受影响，
                                    // 仅该交互不可脚本化；行为证据经场景派发/A2Vue。
                                    if in_loop && !args.is_empty() {
                                        continue;
                                    }
                                    msg = Some(name.name.clone());
                                    payload = Some(args.iter().map(ButtonPayload::from_arg)
                                        .collect::<Result<Vec<_>, _>>()?);
                                    continue;
                                }
                                if let ace_core::source::age_component::ComponentEntryValue::Expr(
                                    expr,
                                ) = &entry.value
                                {
                                    if let ExprKind::Field { base, name } = &expr.kind {
                                        if matches!(&base.kind, ExprKind::Var(v) if v.name == "self")
                                        {
                                            msg = Some(name.name.clone());
                                            payload = None;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(msg) = msg {
                        shape.buttons.push(ButtonSpec {
                            label: "checkbox".to_string(),
                            msg,
                            payload,
                        });
                        shape.button_spans.push(el.span.start);
                    }
                } else {
                    project_view_shape(&el.items, shape, in_loop)?;
                }
            }
        }
    }
    Ok(())
}

/// PLAN-104：循环体**模板叶**展开（010 `for i, f in .faqs { span f.q … }`）。
///
/// 视图数据源=`ComponentViewDataField.literal`（仅视图数据、惰性原始文本，1071
/// 边界）；本 harness 按最小字面量语法（`[ { k: "v", … }, … ]`）解析并在展开
/// 时把 `f.<field>` / `i` 代入叶表达式。不支持的形状具名拒——不静默丢项。
fn expand_view_for_leaves(
    node: &ace_core::source::age_component::ComponentNode,
) -> Result<Vec<String>, String> {
    use ace_core::source::age_component::{ComponentViewFor, ComponentViewItem};
    use ace_core::source::ast::ExprKind;

    fn eval_item_expr(
        expr: &ace_core::source::ast::Expr,
        index_name: Option<&str>,
        value_name: &str,
        idx: usize,
        item: &[(String, String)],
    ) -> Result<String, String> {
        match &expr.kind {
            ExprKind::StrLit(s) => Ok(s.clone()),
            ExprKind::IntLit(i) => Ok(i.to_string()),
            ExprKind::Var(id) if Some(id.name.as_str()) == index_name => Ok(idx.to_string()),
            ExprKind::Field { base, name } => match &base.kind {
                ExprKind::Var(v) if v.name == value_name => item
                    .iter()
                    .find(|(k, _)| k == &name.name)
                    .map(|(_, v)| v.clone())
                    .ok_or_else(|| {
                        format!(
                            "ace.desktop.view-for-unsupported: item has no field `{}`",
                            name.name
                        )
                    }),
                _ => Err(
                    "ace.desktop.view-for-unsupported: only `<value>.<field>` item reads".into(),
                ),
            },
            _ => Err(format!(
                "ace.desktop.view-for-unsupported: expression shape {:?}",
                expr.kind
            )),
        }
    }

    fn collect_leaves(
        items: &[ComponentViewItem],
        index_name: Option<&str>,
        value_name: &str,
        idx: usize,
        item: &[(String, String)],
        out: &mut Vec<String>,
    ) -> Result<(), String> {
        for it in items {
            match it {
                ComponentViewItem::Text(leaf) => {
                    out.push(eval_item_expr(
                        &leaf.expr, index_name, value_name, idx, item,
                    )?);
                }
                ComponentViewItem::Element(el) => {
                    if let Some(label) = &el.label {
                        out.push(eval_item_expr(label, index_name, value_name, idx, item)?);
                    }
                    collect_leaves(&el.items, index_name, value_name, idx, item, out)?;
                }
                ComponentViewItem::Cond(c) => {
                    let hit = expr_truthy(&c.cond, index_name, value_name, idx, item)?;
                    let branch = if hit { &c.then_items } else { &c.else_items };
                    collect_leaves(branch, index_name, value_name, idx, item, out)?;
                }
                ComponentViewItem::For(_) => {
                    return Err(
                        "ace.desktop.view-for-unsupported: nested `for` is outside this slice"
                            .into(),
                    )
                }
                ComponentViewItem::Entry(_) => {}
            }
        }
        Ok(())
    }

    fn expr_truthy(
        expr: &ace_core::source::ast::Expr,
        index_name: Option<&str>,
        value_name: &str,
        idx: usize,
        item: &[(String, String)],
    ) -> Result<bool, String> {
        use ace_core::source::ast::{BinOp, ExprKind};
        match &expr.kind {
            ExprKind::Binary { op, lhs, rhs } => {
                let l = eval_item_expr(lhs, index_name, value_name, idx, item)?;
                let r = eval_item_expr(rhs, index_name, value_name, idx, item)?;
                let (li, ri) = (l.parse::<i64>().ok(), r.parse::<i64>().ok());
                match (op, li, ri) {
                    (BinOp::Lt, Some(a), Some(b)) => Ok(a < b),
                    (BinOp::Le, Some(a), Some(b)) => Ok(a <= b),
                    (BinOp::Gt, Some(a), Some(b)) => Ok(a > b),
                    (BinOp::Ge, Some(a), Some(b)) => Ok(a >= b),
                    (BinOp::Eq, _, _) => Ok(l == r),
                    (BinOp::Ne, _, _) => Ok(l != r),
                    _ => Err("ace.desktop.view-for-unsupported: condition shape".into()),
                }
            }
            ExprKind::Field { base, name } => match &base.kind {
                ExprKind::Var(v) if v.name == value_name => Ok(item
                    .iter()
                    .find(|(k, _)| k == &name.name)
                    .map(|(_, v)| !v.is_empty())
                    .unwrap_or(false)),
                _ => Err("ace.desktop.view-for-unsupported: condition field".into()),
            },
            ExprKind::BoolLit(b) => Ok(*b),
            _ => Err("ace.desktop.view-for-unsupported: condition expression".into()),
        }
    }

    fn find_for<'a>(items: &'a [ComponentViewItem]) -> Option<&'a ComponentViewFor> {
        for it in items {
            match it {
                ComponentViewItem::For(f) => return Some(f),
                ComponentViewItem::Element(el) => {
                    if let Some(f) = find_for(&el.items) {
                        return Some(f);
                    }
                }
                ComponentViewItem::Cond(c) => {
                    if let Some(f) = find_for(&c.then_items).or_else(|| find_for(&c.else_items)) {
                        return Some(f);
                    }
                }
                _ => {}
            }
        }
        None
    }

    let Some(f) = find_for(&node.view.tree) else {
        return Ok(Vec::new());
    };
    // 数据源：`<iter>` 必须是 `.field`（仅视图数据字段）。
    let data_name = match &f.iter.kind {
        ExprKind::Field { base, name } if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") => {
            name.name.clone()
        }
        _ => {
            return Err(
                "ace.desktop.view-for-unsupported: iteration source must be `.viewDataField`"
                    .into(),
            )
        }
    };
    let Some(field) = node.view_data.iter().find(|d| d.name == data_name) else {
        return Err(format!(
            "ace.desktop.view-for-unsupported: no view-data field `{data_name}`"
        ));
    };
    let items = parse_view_data_literal(&field.literal)?;
    let mut out = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        collect_leaves(&f.items, f.index.as_deref(), &f.value, idx, item, &mut out)?;
    }
    Ok(out)
}

#[cfg(test)]
#[test]
fn desktop_view_loop_keeps_item_order_and_optional_index() {
    let source = r#"widget ListPage {
    model {
        items = [
            { label: "first" },
            { label: "second" },
        ]
    }
    view {
        for item in .items {
            text item.label {}
        }
    }
}
"#;
    let indexed = source
        .replace("for item", "for index, item")
        .replace(
            "text item.label {}",
            "if index == 0 { text \"index=0\" {} }\n            text item.label {}",
        );
    for (source, expected) in [
        (source, vec!["first", "second"]),
        (indexed.as_str(), vec!["index=0", "first", "second"]),
    ] {
        let bridge = NodeDesktopBridge::load(source, "list-page.at", 1).expect("load view");
        assert_eq!(bridge.last_texts[0], expected);
    }
}

/// PLAN-104：视图数据字面量的最小解析（`[ { k: "v", … }, … ]`）。
fn parse_view_data_literal(src: &str) -> Result<Vec<Vec<(String, String)>>, String> {
    let mut chars = src.chars().peekable();
    let mut out: Vec<Vec<(String, String)>> = Vec::new();
    let skip_ws = |c: &mut std::iter::Peekable<std::str::Chars<'_>>| {
        while c.peek().is_some_and(|ch| ch.is_whitespace()) {
            c.next();
        }
    };
    skip_ws(&mut chars);
    if chars.next() != Some('[') {
        return Err("ace.desktop.view-for-unsupported: view data is not a list literal".into());
    }
    loop {
        skip_ws(&mut chars);
        match chars.peek() {
            Some(']') => {
                chars.next();
                break;
            }
            Some(',') => {
                chars.next();
                continue;
            }
            Some('{') => {
                chars.next();
            }
            _ => return Err("ace.desktop.view-for-unsupported: view data item shape".into()),
        }
        let mut item: Vec<(String, String)> = Vec::new();
        loop {
            skip_ws(&mut chars);
            match chars.peek() {
                Some('}') => {
                    chars.next();
                    break;
                }
                Some(',') => {
                    chars.next();
                    continue;
                }
                _ => {}
            }
            // key
            let mut key = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    key.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            if key.is_empty() {
                return Err("ace.desktop.view-for-unsupported: item key".into());
            }
            skip_ws(&mut chars);
            if chars.next() != Some(':') {
                return Err("ace.desktop.view-for-unsupported: item key separator".into());
            }
            skip_ws(&mut chars);
            if chars.next() != Some('"') {
                return Err("ace.desktop.view-for-unsupported: item value must be a string".into());
            }
            let mut value = String::new();
            let mut escaped = false;
            for ch in chars.by_ref() {
                if escaped {
                    value.push(ch);
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    break;
                } else {
                    value.push(ch);
                }
            }
            item.push((key, value));
        }
        out.push(item);
    }
    Ok(out)
}

/// PLAN-104：**harness 不消费 style 面**——`style …` 声明行与 `style:` 条目行
/// （含跨行 `if … { … } else { … }` 继续行）按原**字节**长度空白化，span 不漂移；
/// 样式由 A2Vue（1072 生产链）覆盖。
///
/// 行宽保持包含行尾 `\r`（CRLF 源：`\r` 也替换成空格，字节数不变）；花括号
/// 计数忽略字符串字面量内的括号（Tailwind 任意值可能含 `{}`）。
pub(crate) fn blank_style_surface(src: &str) -> String {
    /// 行内花括号净差（跳过字符串）。
    fn depth_delta(s: &str) -> i32 {
        let mut d = 0i32;
        let mut in_str = false;
        let mut esc = false;
        for ch in s.chars() {
            if in_str {
                if esc {
                    esc = false;
                } else if ch == '\\' {
                    esc = true;
                } else if ch == '"' {
                    in_str = false;
                }
            } else if ch == '"' {
                in_str = true;
            } else if ch == '{' {
                d += 1;
            } else if ch == '}' {
                d -= 1;
            }
        }
        d
    }

    /// 在一行里找 `style:` 条目（字符串外）并给出**值区间**的起止与净开括号数。
    /// 值在 `,`/`}`（同层）/行尾结束；多行条件样式返回净开括号数供续行空白化。
    fn style_entry_span(body: &str) -> Option<(usize, usize, i32)> {
        let mut in_str = false;
        let mut esc = false;
        for (i, ch) in body.char_indices() {
            if in_str {
                if esc {
                    esc = false;
                } else if ch == '\\' {
                    esc = true;
                } else if ch == '"' {
                    in_str = false;
                }
                continue;
            }
            if ch == '"' {
                in_str = true;
                continue;
            }
            if body[i..].starts_with("style:") {
                // 值区间：从 `style:` 后到同层 `,`/`}` 或行尾。
                let start = i;
                // 同层 `,` 先截断（同一条目行上的后续 entry 不受影响）；
                // 否则取本行**最后一个** `}` 之前（元素自身的收尾大括号保留；
                // 值内的条件分支大括号随之消失——整段样式语法块消失）。
                let mut j = i + "style:".len();
                let mut d = 0i32;
                let mut in_s = false;
                let mut es = false;
                let mut comma_at: Option<usize> = None;
                let mut first_close: Option<usize> = None;
                let mut last_close: Option<usize> = None;
                while j < body.len() {
                    let c = body.as_bytes()[j] as char;
                    if in_s {
                        if es {
                            es = false;
                        } else if c == '\\' {
                            es = true;
                        } else if c == '"' {
                            in_s = false;
                        }
                    } else if c == '"' {
                        in_s = true;
                    } else if c == '{' {
                        d += 1;
                    } else if c == '}' {
                        if d == 0 {
                            if first_close.is_none() {
                                first_close = Some(j);
                            }
                            last_close = Some(j);
                        } else {
                            d -= 1;
                        }
                    } else if c == ',' && d == 0 {
                        comma_at = Some(j);
                        break;
                    }
                    j += 1;
                }
                let (end, opens) = if let Some(c) = comma_at {
                    (c, 0)
                } else if d > 0 && last_close.is_none() {
                    // 值在本行未收口（多行条件样式）——本行尾整段空白化，
                    // 续行按深度整行空白化。
                    (body.len(), d)
                } else if body[start..].contains(" else ") {
                    // 单行条件样式 `if … { … } else { … }`：值到本行最后一个 `}`
                    // 之前（保留外层元素的收尾括号）。
                    (last_close.unwrap_or(body.len()), 0)
                } else {
                    // 普通条目：到第一个同层 `}` 之前（嵌套单行元素只带走自己的
                    // 收尾括号，不越界吃外层）。
                    (first_close.unwrap_or(body.len()), 0)
                };
                return Some((start, end, opens));
            }
            // (char_indices 自增——无手工 i += 1)
        }
        None
    }

    let mut out = String::with_capacity(src.len());
    // Some(depth>0) = 跨行样式值未收口（继续整行空白化）。
    let mut depth: Option<i32> = None;
    for seg in src.split_inclusive('\n') {
        let (body, nl) = match seg.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (seg, ""),
        };
        if let Some(d) = depth {
            let nd = d + depth_delta(body);
            // 多行样式值的续行整行空白化（含花括号——整个语法块消失，
            // 不留匿名块）。
            out.push_str(&" ".repeat(body.len()));
            out.push_str(nl);
            depth = if nd > 0 { Some(nd) } else { None };
            continue;
        }
        // 独立 style 声明（`style name = …`）整行空白化。
        let trim = body.trim_start();
        if trim.starts_with("style ") {
            out.push_str(&" ".repeat(body.len()));
            out.push_str(nl);
            continue;
        }
        // 行内 entry 形（`text "x" { style: ref }`）——只空白化值区间。
        if let Some((start, end, opens)) = style_entry_span(body) {
            let mut s = String::with_capacity(body.len());
            if opens > 0 {
                // 多行条件样式：本行尾部整段空白化（含 `{`），续行整行空白化。
                s.push_str(&body[..start]);
                s.push_str(&" ".repeat(body.len() - start));
                out.push_str(&s);
                out.push_str(nl);
                depth = Some(opens);
                continue;
            }
            s.push_str(&body[..start]);
            s.push_str(&" ".repeat(end - start));
            s.push_str(&body[end..]);
            out.push_str(&s);
            out.push_str(nl);
            continue;
        }
        out.push_str(seg);
    }
    out
}
/// PLAN-104：**元素 label 展示**（`h2 .dashboard_title` / `span "✓"` 族）——
/// 这些不是 `text` 叶（不进 AV 叶表），但 harness 显示面需要；按实例状态求值
/// 成展示行（仅显示，不参与 FFI 叶编号）。
fn collect_element_labels(
    items: &[ace_core::source::age_component::ComponentViewItem],
    states: &[(String, AceStateValue)],
    out: &mut Vec<String>,
) {
    use ace_core::source::age_component::ComponentViewItem;
    use ace_core::source::ast::ExprKind;
    for item in items {
        match item {
            ComponentViewItem::Element(el) => {
                if let Some(label) = &el.label {
                    let shown = match &label.kind {
                        ExprKind::StrLit(s) => Some(s.clone()),
                        ExprKind::IntLit(i) => Some(i.to_string()),
                        ExprKind::Field { base, name } if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") => {
                            states
                                .iter()
                                .find(|(n, _)| n == &name.name)
                                .map(|(_, v)| state_display(v))
                        }
                        _ => None,
                    };
                    if let Some(s) = shown {
                        out.push(format!("{}: {s}", el.tag));
                    }
                }
                collect_element_labels(&el.items, states, out);
            }
            ComponentViewItem::Cond(c) => {
                collect_element_labels(&c.then_items, states, out);
                collect_element_labels(&c.else_items, states, out);
            }
            // 循环体已在模板叶展开（含 label）。
            ComponentViewItem::For(_)
            | ComponentViewItem::Text(_)
            | ComponentViewItem::Entry(_) => {}
        }
    }
}

/// PLAN-104：条件表达式求值（`.field` / `.field ==/!= 字面量` / `!expr`）。
/// 桌面 harness 的语料形；其他形状具名拒（渲染刀后续）。
fn eval_view_cond(
    expr: &ace_core::source::ast::Expr,
    states: &[(String, AceStateValue)],
) -> Result<bool, String> {
    use ace_core::source::ast::{BinOp, ExprKind, UnaryOp};
    match &expr.kind {
        ExprKind::BoolLit(value) => Ok(*value),
        ExprKind::Field { base, name } => {
            if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") {
                match states.iter().find(|(n, _)| n == &name.name).map(|(_, v)| v) {
                    Some(AceStateValue::Bool(b)) => Ok(*b),
                    Some(AceStateValue::Int(v)) => Ok(*v != 0),
                    other => Err(format!(
                        "ace.desktop.view-cond-unsupported-expr: condition field `{}` is not a live bool/int state ({other:?})",
                        name.name
                    )),
                }
            } else {
                Err("ace.desktop.view-cond-unsupported-expr: nested field condition".into())
            }
        }
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand,
        } => Ok(!eval_view_cond(operand, states)?),
        ExprKind::Binary { op: BinOp::And, lhs, rhs } => {
            Ok(eval_view_cond(lhs, states)? && eval_view_cond(rhs, states)?)
        }
        ExprKind::Binary { op: BinOp::Or, lhs, rhs } => {
            Ok(eval_view_cond(lhs, states)? || eval_view_cond(rhs, states)?)
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let l = cond_atom(lhs, states)?;
            let r = cond_atom(rhs, states)?;
            match (op, l, r) {
                (BinOp::Eq, Some(l), Some(r)) => Ok(cond_atom_eq(&l, &r)),
                (BinOp::Ne, Some(l), Some(r)) => Ok(!cond_atom_eq(&l, &r)),
                _ => {
                    Err("ace.desktop.view-cond-unsupported-expr: only `field ==/!= literal`".into())
                }
            }
        }
        _ => Err("ace.desktop.view-cond-unsupported-expr: expression shape".into()),
    }
}

fn unsupported_widget_condition(
    items: &[ace_core::source::age_component::ComponentViewItem],
    inside_svg: bool,
) -> Option<ace_core::atom_text::Span> {
    use ace_core::source::age_component::ComponentViewItem;
    items.iter().find_map(|item| match item {
        ComponentViewItem::Cond(condition) if !inside_svg => Some(condition.span),
        ComponentViewItem::Cond(condition) => unsupported_widget_condition(&condition.then_items, true)
            .or_else(|| unsupported_widget_condition(&condition.else_items, true)),
        ComponentViewItem::Element(element) => {
            unsupported_widget_condition(&element.items, inside_svg || element.tag == "svg")
        }
        ComponentViewItem::For(each) => unsupported_widget_condition(&each.items, inside_svg),
        _ => None,
    })
}

fn cond_atom(
    expr: &ace_core::source::ast::Expr,
    states: &[(String, AceStateValue)],
) -> Result<Option<AceStateValue>, String> {
    use ace_core::source::ast::ExprKind;
    match &expr.kind {
        ExprKind::Field { base, name } => {
            if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") {
                Ok(states
                    .iter()
                    .find(|(n, _)| n == &name.name)
                    .map(|(_, v)| v.clone()))
            } else {
                Ok(None)
            }
        }
        ExprKind::StrLit(s) => Ok(Some(AceStateValue::Str(s.clone()))),
        ExprKind::IntLit(i) => Ok(Some(AceStateValue::Int(*i as i32))),
        ExprKind::BoolLit(b) => Ok(Some(AceStateValue::Bool(*b))),
        _ => Ok(None),
    }
}

fn cond_atom_eq(l: &AceStateValue, r: &AceStateValue) -> bool {
    match (l, r) {
        (AceStateValue::Str(a), AceStateValue::Str(b)) => a == b,
        (AceStateValue::Int(a), AceStateValue::Int(b)) => a == b,
        (AceStateValue::Bool(a), AceStateValue::Bool(b)) => a == b,
        _ => false,
    }
}

/// PLAN-104：按 view 树声明序遍历文本叶，标记可见性（叶序与
/// `project_view_shape` 完全一致）。
fn mark_leaf_visibility(
    items: &[ace_core::source::age_component::ComponentViewItem],
    states: &[(String, AceStateValue)],
    active: bool,
    out: &mut Vec<bool>,
) -> Result<(), String> {
    use ace_core::source::age_component::ComponentViewItem;
    for item in items {
        match item {
            ComponentViewItem::Text(_) => out.push(active),
            ComponentViewItem::Entry(_) => {}
            ComponentViewItem::Cond(c) => {
                let hit = active && eval_view_cond(&c.cond, states)?;
                mark_leaf_visibility(&c.then_items, states, active && hit, out)?;
                mark_leaf_visibility(&c.else_items, states, active && !hit, out)?;
            }
            ComponentViewItem::For(f) => {
                mark_leaf_visibility(&f.items, states, active, out)?;
            }
            ComponentViewItem::Element(el) => {
                if el.tag == "text" {
                    if el.label.is_some() {
                        out.push(active);
                    }
                } else if el.tag == "input" || el.tag == "button" {
                    // input/button 自身 label 不计文本叶（与 shape 一致）。
                } else {
                    mark_leaf_visibility(&el.items, states, active, out)?;
                }
            }
        }
    }
    Ok(())
}

/// AV 外调执行器：`use.rs ui980_native` 前门 + run_ace（同步、调用者线程）。
fn av_call(body: &str) -> Result<i32, String> {
    let src = format!("use.rs ui980_native\nfn main() int {{ return {body} }}");
    let mut request = auto_lang_core::AceRequest::simple(&src);
    request.provider = Some(UI980_PROVIDER_NAME);
    let report = auto_lang_core::run_ace(auto_lang_core::AceEngineChoice::AceAV, &request);
    match &report.outcome {
        auto_lang_core::AceOutcome::Success {
            value: auto_lang_core::AceValue::I32(n),
            ..
        } => Ok(*n),
        other => Err(format!("ui980 av call `{body}` failed: {other:?}")),
    }
}

/// Auto 源字符串字面量转义（复用 HELLO 冻结口径）。
fn escape(s: &str) -> String {
    crate::ui::ace_hello::escape_auto_string_for_source(s)
}

fn diags(ds: &[ace_core::atom_text::Diagnostic]) -> String {
    ds.iter()
        .map(|d| format!("{}: {} @{:?}", d.code, d.message, d.span))
        .collect::<Vec<_>>()
        .join("; ")
}

fn on_widget_stack<T: Send>(
    label: &str,
    task: impl FnOnce() -> Result<T, String> + Send,
) -> Result<T, String> {
    // Package compilation can require more stack than the window thread
    // reserves. Keep the window on its owning thread.
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name(format!("ace-widget-{label}"))
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, task)
            .map_err(|error| format!("start widget {label}: {error}"))?
            .join()
            .map_err(|_| format!("widget {label} thread panicked"))?
    })
}

impl NodeDesktopBridge {
    /// 装载：投影 view 形状（文本叶/按钮目标）+ 文本叶 AV 真值（静态观察面）
    /// + states-only 引擎挂载（keyed 实例、世代纪律；无 model/msg 的 Hello 类
    /// 源为静态面——事件面缺席具名可辨）。
    pub fn load(src: &str, path: &str, instances: usize) -> Result<Self, String> {
        let mut bridge = on_widget_stack("compile", || Self::load_source(src, path, instances))?;
        bridge.set_source_assets(std::path::Path::new(path));
        Ok(bridge)
    }

    /// Open the component lane used by the opt-in SQLite message mirror.
    pub fn load_sqlite_component(src: &str, path: &str, instances: usize) -> Result<Self, String> {
        let mut bridge = on_widget_stack("compile", || Self::load_component_source(src, path, instances))?;
        bridge.set_source_assets(std::path::Path::new(path));
        Ok(bridge)
    }

    fn load_source(src: &str, path: &str, instances: usize) -> Result<Self, String> {
        use ace_core::source::age_package::{FILE_LIST_BINDING, FILE_MUTATE_BINDING, FILE_OPEN_BINDING};
        // Preserve typed child-event bindings on the package front door.
        let package_source = super::ace_widget_packages::normalize_package_module_placeholders(src);
        let package_src = package_source.as_str();
        if let Ok(candidates) = package_candidates(package_src) {
            if candidates.first().is_some_and(|node| node.members.iter().any(|row| row.role == "route")) {
                let app = std::path::Path::new(path);
                let (units, paths) = front_package_units(app, package_src, &front_route_modules(app)?)?;
                return Self::load_package_paths(units, "app::app", instances, paths);
            }
            if candidates.iter().any(|node| !node.package_imports.is_empty())
                || candidates.first().is_some_and(|node| node.members.iter().any(|row| {
                    row.role == "timer" || (row.role == "bind"
                        && matches!(row.name.as_str(), FILE_LIST_BINDING | FILE_MUTATE_BINDING | FILE_OPEN_BINDING))
                }))
            {
                let mut units = vec![ace_core::source::packages::PackageModuleInput {
                    package: "app".into(),
                    version: "1.0.0".into(),
                    id: "app".into(),
                    source: package_src.into(),
                    provider: None,
                }];
                super::ace_widget_packages::append_widget_packages(
                    std::path::Path::new(path),
                    package_src,
                    &candidates,
                    &mut units,
                )?;
                let source = std::path::Path::new(path);
                let backend = super::ace_back_api::append_contract(source, &mut units)?;
                let mut bridge = Self::load_package_paths(units, "app::app", instances, RouteModulePaths::new())?;
                if let Some(back_entry) = backend {
                    let Some(BridgeEngine::Package(engine)) = bridge.engine.as_mut() else {
                        return Err("checked backend requires a live desktop package".into());
                    };
                    let spec = crate::back_proxy::SessionSpec { app_id: "checked-desktop".into(), back_entry };
                    engine.set_file_backend(spec, source.parent().unwrap_or(std::path::Path::new(".")));
                }
                return Ok(bridge);
            }
        }
        Self::load_component_source(src, path, instances)
    }

    fn load_component_source(src: &str, path: &str, instances: usize) -> Result<Self, String> {
        let standalone_source = super::ace_widget_packages::normalize_event_placeholders(src, false);
        let src = standalone_source.as_str();
        // PLAN-104：harness 不消费 style 面（A2Vue 覆盖样式）；空白化保字节长。
        // PLAN-105 T-02：**先用原源**投影（本地/内联样式可解析），仅当原源投影
        // 失败才退回空白化（104 既有语料语义不变；真实工程源不再因样式引用被
        // 误伤）。
        let candidates = match project_component_candidates(src) {
            Ok(c) => c,
            Err(ds) => {
                // 只有样式解析类失败才退回空白化；其它具名拒（store/结构/语法）
                // 必须原样透出——否则会被空白化路径的次生错误掩盖。
                let style_only = ds
                    .iter()
                    .all(|d| d.code == "source.age.style-call");
                if style_only {
                    project_component_candidates(&blank_style_surface(src))
                        .map_err(|ds| diags(&ds))?
                } else {
                    return Err(diags(&ds));
                }
            }
        };
        let node = candidates
            .first()
            .ok_or_else(|| "no AGE node candidate in source".to_string())?;
        // PLAN-104：view 条件项由本 harness 消费——flat 观察 + 桥侧按实例状态
        // 求值可见性（见 `mark_leaf_visibility`），不再在装载点整例拒绝。
        // PLAN-104：循环项（010 `for i, f in .faqs { … }`）——循环体是**模板叶**，
        // 不进 `view.texts`（投影只收集模块作用域叶）。全例只有循环体文本时，
        // 本 harness 走 states-only 面（无文本叶可观察），列表渲染归后续刀。
        let mut shape = ViewShape::default();
        project_view_shape(&node.view.tree, &mut shape, false)?;

        // 文本叶 AV 真值。两段面（B-NATIVE-I 有界决定）：
        // ① 免驱动 full 车道（带 model 的源，含 on-handler 场景面）；
        // ② 静态观察面（Hello 类：无 model 字段可观察——①具名拒后的回退）。
        // f-string 文本叶在两条车道都具名拒——KNOWN-DEBT 边界，完整 W1 义务。
        let caps = ["ac.runtime.text.v1".to_string()];
        let trial = ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        };
        let trial_initials = ComponentInitials {
            per_instance: vec![Vec::new()],
        };
        // 事件引擎：有 model 的源挂 N 实例；Hello 类（无 model 字段可观察）
        // 为静态面（engine=None——点击/键入无从产生，事件面缺席具名可辨）。
        let mut engine = None;
        let mut identities = Vec::new();
        let mut last_states = Vec::new();
        let mut field_names = Vec::new();
        match AceStateEngine::new(src, path, LegacyWidgetAlias::Accept) {
            Ok(mut e) => {
                identities = (0..instances).map(|i| format!("i{i}")).collect();
                for id in &identities {
                    e.mount(id, Vec::new())?;
                    last_states.push(e.read_all(id).unwrap_or_default());
                }
                field_names = last_states
                    .first()
                    .map(|row| row.iter().map(|(n, _)| n.clone()).collect())
                    .unwrap_or_default();
                // B-AV U01：输入载荷类型按绑定字段初值定（Int=数值输入，
                // 键入文本桥面解析；Str=文本输入原样）。绑定字段不在状态行
                // =投影/状态面失配，装载期具名拒。
                for spec in &mut shape.inputs {
                    let kind = last_states
                        .first()
                        .and_then(|row| row.iter().find(|(n, _)| *n == spec.field).map(|(_, v)| v));
                    match kind {
                        Some(AceStateValue::Int(_)) => spec.numeric = true,
                        Some(AceStateValue::Str(_)) => {}
                        _ => {
                            return Err(format!(
                                "input target binds field `{}` which is not a live str/int state",
                                spec.field
                            ))
                        }
                    }
                }
                // PLAN-108（T-03c M2）：宿主 HTTP provider 注入（含 Http 效应
                // 的程序派发整体路由 worker 线程；UI 线程不等待）。
                e.set_http_provider(std::sync::Arc::new(std::sync::Mutex::new(
                    super::ace_state_bridge::UiHttpProvider::new(),
                )));
                engine = Some(BridgeEngine::Component(e));
            }
            Err(e) if e.contains("requires at least one") => {
                if !shape.buttons.is_empty() || !shape.inputs.is_empty() {
                    return Err(format!(
                        "event targets need a model/msg surface, but the state face refused: {e}"
                    ));
                }
            }
            Err(e) => return Err(e),
        }

        let session_texts = match engine.as_mut() {
            Some(BridgeEngine::Component(engine)) if !node.view.texts.is_empty() => identities
                .first()
                .and_then(|id| engine.read_texts(id).ok())
                .filter(|texts| texts.len() == shape.text_leaf_count),
            _ => None,
        };
        // Reuse the live checked program and its initial state for text leaves.
        // Preserve the observation fallback for static and legacy sources.
        let per_node_texts = if let Some(texts) = session_texts {
            vec![texts]
        } else {
            if node.view.texts.is_empty() {
                // PLAN-104：无模块作用域文本叶（循环体模板叶）——按视图数据把
                // 循环体展开为具体叶值（010 的 FAQ 列表）；states-only 面照常。
                let expanded = expand_view_for_leaves(node)?;
                if expanded.is_empty() {
                    vec![vec![String::new(); 0]]
                } else {
                    shape.text_leaf_count = expanded.len();
                    vec![expanded]
                }
            } else {
                match compile_component_scenario_with_initials(
                    src,
                    path,
                    LegacyWidgetAlias::Accept,
                    &caps,
                    &trial,
                    None,
                    &trial_initials,
                ) {
                    Ok(compiled) => {
                        // 状态真值经事件引擎（mount 后 read_all）取；此处只消费文本叶。
                        let (_states0, texts0) =
                            observe_component_states_and_texts_flat(&compiled)?;
                        let observed_leaves = texts0.first().map(|t| t.len()).unwrap_or(0);
                        if observed_leaves != shape.text_leaf_count {
                            // The scenario may expand loop leaves or observe
                            // just the active branch of a dynamic list view.
                            let expected = match expand_view_for_leaves(node) {
                                Ok(expanded) => Some(node.view.texts.len() + expanded.len()),
                                Err(error) if error.contains("view-for-unsupported") => None,
                                Err(error) => return Err(error),
                            };
                            if expected.is_some_and(|count| count != observed_leaves) {
                                return Err(format!(
                                    "text leaf mismatch: view tree has {}, AV observed {observed_leaves}",
                                    shape.text_leaf_count
                                ));
                            }
                            shape.text_leaf_count = observed_leaves;
                        }
                        texts0
                    }
                    Err(ds)
                        if ds
                            .iter()
                            .any(|d| d.message.contains("requires at least one")) =>
                    {
                        // Hello 类（无 model 字段）：静态观察面（带 on 的源在此被
                        // 具名拒——`on` handler 属场景面，不属静态面）。
                        let texts0 = observe_component_texts(src, path, LegacyWidgetAlias::Accept)?;
                        let observed_leaves = texts0.first().map(|t| t.len()).unwrap_or(0);
                        if observed_leaves != shape.text_leaf_count {
                            return Err(format!(
                                "text leaf mismatch: view tree has {}, AV observed {observed_leaves}",
                                shape.text_leaf_count
                            ));
                        }
                        texts0
                    }
                    Err(ds) => return Err(diags(&ds)),
                }
            }
        };

        let expanded_static = node.view.texts.is_empty()
            && !per_node_texts.first().map(|r| r.is_empty()).unwrap_or(true);
        Ok(Self {
            path: path.to_string(),
            node_name: node.name.name.clone(),
            engine,
            identities,
            session: None,
            instances,
            buttons: shape.buttons,
            inputs: shape.inputs,
            pending_texts: VecDeque::new(),
            deferred_snapshot: false,
            text_leaf_count: shape.text_leaf_count,
            last_texts: vec![per_node_texts.first().cloned().unwrap_or_default(); instances],
            last_states,
            field_names,
            dispatch_log: Vec::new(),
            tick_accum_ms: Vec::new(),
            errors: Vec::new(),
            script: Vec::new(),
            tick_no: 0,
            view_items: node.view.tree.clone(),
            widget_initial_bools: widgets::initial_bools(&node.fields),
            public_assets: None,
            expanded_static,
            routes: None,
            route_actions: Default::default(),
            route_button_spans: Vec::new(),
            package_route_handlers: None,
            shortcuts: Default::default(),
            pending_shortcuts: VecDeque::new(),
            route_request: None,
            sqlite_bridge: None,
        })
    }

    /// 事件引擎面（静态面源 → false——Hello 类无事件）。
    pub fn engine_face(&self) -> bool {
        self.engine.is_some()
    }

    /// PLAN-980 ④-A #2b：**AGE 包闭包**装载（多单元 + 入口）。view 形状取入口
    /// unit 的组件面投影；状态/事件走 [`PackageStateEngine`]（AV 包观察链）。
    /// 边界（具名）：包面场景无 props/initials——实例种子为空；状态只含 int
    /// 字段；带载荷 dispatch 本片未接线；非 int 输入绑定在装载期具名拒。
    pub fn load_package(
        units: Vec<PackageModuleInput>,
        entry: &str,
        instances: usize,
    ) -> Result<Self, String> {
        on_widget_stack("compile", || Self::load_package_paths(units, entry, instances, RouteModulePaths::new()))
    }

    fn load_package_paths(
        units: Vec<PackageModuleInput>,
        entry: &str,
        instances: usize,
        paths: RouteModulePaths,
    ) -> Result<Self, String> {
        let unit = units.iter().find(|unit| format!("{}::{}", unit.package, unit.id) == entry)
            .ok_or_else(|| format!("entry module `{entry}` is not in the package units"))?;
        let candidates = match package_candidates(&unit.source) {
            Ok(candidates) => candidates,
            // The single-unit inspection cannot resolve a Store imported
            // from another package unit. The complete package remains checked.
            Err(error) if error.contains("Declare the store in the same package unit") => {
                return Self::load_package_leaf(units, entry, instances);
            }
            Err(error) => return Err(error),
        };
        let node = candidates.first()
            .ok_or_else(|| "no AGE node candidate in the entry module".to_string())?;
        // A desktop session admits only the page it is about to mount.
        let Some(table) = RouteTable::from_root(node, &units, unit, false)? else {
            return Self::load_package_leaf(units, entry, instances);
        };
        let prepared = super::ace_route::prepare_page(&unit.source, Some(&Default::default()))?;
        let mut shell_units = table.entry_units(&units, entry);
        let shell_unit = shell_units.iter_mut().find(|unit| format!("{}::{}", unit.package, unit.id) == entry)
            .expect("entry was present in the package units");
        shell_unit.source = prepared.source;
        if let Some(path) = paths.get(entry) {
            let source = shell_unit.source.clone();
            let nodes = package_candidates(&source)?;
            super::ace_widget_packages::append_widget_packages(path, &source, &nodes, &mut shell_units)?;
        }
        let mut bridge = Self::load_package_leaf(shell_units, entry, instances)?;
        let links = if bridge.has_checked_package_view() {
            Vec::new()
        } else {
            if let Some(span) = ace_core::source::age_component::view_contains_cond(&node.view.tree) {
                return Err(format!(
                    "ace.desktop.view-cond-unsupported: conditional routed shell at byte {}",
                    span.start
                ));
            }
            route_links(node)?
        };
        bridge.set_route_controls(prepared.commands, links);
        bridge.routes = Some(Box::new(DesktopRoutes::new(table, units, paths)));
        Ok(bridge)
    }

    pub(super) fn load_package_leaf(
        units: Vec<PackageModuleInput>,
        entry: &str,
        instances: usize,
    ) -> Result<Self, String> {
        if instances == 0 {
            return Err("a package bridge needs at least one instance".to_string());
        }
        let (mut engine, node) =
            PackageStateEngine::new_with_root(units, entry, LegacyWidgetAlias::Accept)?;
        let checked = engine.has_checked_view();
        let mut shape = ViewShape::default();
        if !checked {
        project_view_shape(&node.view.tree, &mut shape, false)?;
        let rejected_condition = if shape.inputs.is_empty() {
            unsupported_widget_condition(&node.view.tree, false)
        } else {
            ace_core::source::age_component::view_contains_cond(&node.view.tree)
        };
        if let Some(sp) = rejected_condition {
            return Err(format!(
                "ace.desktop.view-cond-unsupported: view 条件项 @{}..{} 本腿未承载（桌面渲染语义待后续刀；不静默展开）",
                sp.start, sp.end
            ));
        }
        }
        let identities: Vec<String> = (0..instances).map(|i| format!("i{i}")).collect();
        let mut last_states = Vec::new();
        let mut last_texts = Vec::with_capacity(instances);
        for id in &identities {
            if checked { break; }
            engine.mount(id, Vec::new())?;
            let snapshot = engine.read_snapshot(id).unwrap_or_default();
            if snapshot.texts.len() != shape.text_leaf_count {
                return Err(format!(
                    "text leaf mismatch: view tree has {}, AV observed {}",
                    shape.text_leaf_count,
                    snapshot.texts.len()
                ));
            }
            let mut visibility = Vec::new();
            mark_leaf_visibility(&node.view.tree, &snapshot.states, true, &mut visibility)?;
            last_states.push(snapshot.states);
            last_texts.push(snapshot.texts);
        }
        let field_names = last_states
            .first()
            .map(|row| row.iter().map(|(n, _)| n.clone()).collect())
            .unwrap_or_default();
        for spec in &mut shape.inputs {
            let kind = last_states
                .first()
                .and_then(|row| row.iter().find(|(n, _)| *n == spec.field).map(|(_, v)| v));
            match kind {
                Some(AceStateValue::Int(_)) => spec.numeric = true,
                _ => {
                    return Err(format!(
                        "input target binds field `{}` which is not a live int state on the \
                         package face",
                        spec.field
                    ))
                }
            }
        }
        Ok(Self {
            path: entry.to_string(),
            node_name: node.name.name.clone(),
            engine: Some(BridgeEngine::Package(engine)),
            identities,
            session: None,
            instances,
            buttons: shape.buttons,
            inputs: shape.inputs,
            pending_texts: VecDeque::new(),
            deferred_snapshot: false,
            text_leaf_count: shape.text_leaf_count,
            last_texts,
            last_states,
            field_names,
            dispatch_log: Vec::new(),
            tick_accum_ms: Vec::new(),
            errors: Vec::new(),
            script: Vec::new(),
            tick_no: 0,
            view_items: node.view.tree.clone(),
            widget_initial_bools: widgets::initial_bools(&node.fields),
            public_assets: None,
            expanded_static: false,
            routes: None,
            route_actions: Default::default(),
            route_button_spans: Vec::new(),
            package_route_handlers: None,
            shortcuts: if checked { super::ace_shortcuts::widget_shortcuts(&node.members, &node.messages) } else { Default::default() },
            pending_shortcuts: VecDeque::new(),
            route_request: None,
            sqlite_bridge: None,
        })
    }

    pub(super) fn set_source_assets(&mut self, source: &std::path::Path) {
        self.public_assets = widgets::public_assets(source).map(Arc::new);
    }

    pub(super) fn set_route_controls(&mut self, handlers: RouteHandlers, links: Vec<RouteLink>) {
        if self.has_checked_package_view() {
            self.package_route_handlers = Some(handlers);
            return;
        }
        for (index, button) in self.buttons.iter().enumerate() {
            if let Some(action) = handlers.get(&button.msg) {
                self.route_actions.insert(index, action.clone());
            }
        }
        for link in links {
            self.route_actions.insert(self.buttons.len(), RouteAction::literal(link.destination));
            self.buttons.push(ButtonSpec { label: link.label, msg: "navigate".into(), payload: None });
            self.route_button_spans.push(link.span_start);
        }
    }

    pub(super) fn has_checked_package_view(&self) -> bool {
        matches!(&self.engine, Some(BridgeEngine::Package(engine)) if engine.has_checked_view())
    }

    /// Bind a checked package to a local service before its first mount.
    pub fn install_todo_service(
        &mut self,
        binding: ase_m0::todo_service::ServiceBinding,
        realm: ase_m0::todo_service::RealmId,
    ) -> Result<(), String> {
        let Some(BridgeEngine::Package(engine)) = self.engine.as_mut() else {
            return Err("ace.desktop.todo-binding: expected a checked package".into());
        };
        engine.install_todo_service(binding, realm)
    }

    pub(crate) fn step_todo_service(&mut self) -> StepOutcome {
        let Some(BridgeEngine::Package(engine)) = self.engine.as_mut() else {
            return StepOutcome::Idle;
        };
        for (instance, identity) in self.identities.iter().enumerate() {
            match engine.poll_todo_service(identity) {
                Ok(Some((message, states))) => {
                    return self.publish_package_frame(instance, &message, states);
                }
                Ok(None) => {}
                Err(error) => {
                    self.errors.push(error.clone());
                    return StepOutcome::DispatchRefused(error);
                }
            }
        }
        StepOutcome::Idle
    }

    pub(super) fn initialize_route_page(&mut self) -> Result<(), String> {
        for instance in 0..self.instances {
            match self.dispatch_event(instance, "Init", None) {
                StepOutcome::Dispatched { .. } => {}
                StepOutcome::DispatchRefused(error) | StepOutcome::LaneError(error) => return Err(error),
                StepOutcome::Idle => return Err("page Init did not dispatch".into()),
            }
        }
        Ok(())
    }

    /// 事件目标清单（声明序；测试/证据消费）。
    pub fn buttons(&self) -> &[ButtonSpec] {
        &self.buttons
    }

    /// B-AV U01：输入目标清单（声明序；目标编号=buttons.len() 起）。
    pub fn inputs(&self) -> &[InputSpec] {
        &self.inputs
    }

    /// B-AV U01：文本输入事件注入（真实键入与脚本同一边界）——provider
    /// 队列取 (instance, target) 码、桥面 FIFO 配对键入全文。
    pub fn inject_text(&mut self, instance: usize, target: usize, text: &str) {
        if let Some(session) = self.session {
            push_session_event(session, instance, target);
        }
        self.pending_texts
            .push_back((instance, target, text.to_string()));
    }

    /// open：AV `ui_open` → session；就位显示模型并提交文本叶与状态初值
    /// （FFI 实效果——UI 创建/提交效果经 provider 车道承载）。
    pub fn open(&mut self) -> Result<i32, String> {
        let shell = self.open_lane(false)?;
        if let Some(routes) = self.routes.as_mut() {
            let result = routes.navigate(RouteNavigation::Push("/".to_string()), self.instances);
            select_host_session(shell);
            if let Err(error) = result {
                let _ = self.close_for_shutdown();
                return Err(error);
            }
        }
        Ok(shell)
    }

    /// Open a fresh session with file access supplied by the trusted window host.
    pub fn open_with_file_access(
        &mut self,
        root: &std::path::Path,
        writes: auto_lang_core::file_session::FileWritePermissions,
    ) -> Result<i32, String> {
        let session = self.open()?;
        for instance in 0..self.instances {
            if let Err(error) = self.grant_file_access(instance, root, writes) {
                let _ = self.close_for_shutdown();
                return Err(format!("ace.desktop.file-grant: {error}"));
            }
        }
        Ok(session)
    }

    /// Open the frame layout and defer its values until a fresh event arrives.
    pub(crate) fn open_awaiting_event(&mut self) -> Result<i32, String> {
        self.open_lane(true)
    }

    fn open_lane(&mut self, deferred_snapshot: bool) -> Result<i32, String> {
        install_ui980_lane()?;
        let (frames, mounted_here) = self.mount_package_views()?;
        let session = match av_call(&format!("ui_open(\"open\", {}, 0)", self.instances)) {
            Ok(session) => session,
            Err(error) => {
                if mounted_here {
                    self.release_package_views();
                }
                return Err(error);
            }
        };
        if session < 1 {
            if mounted_here {
                self.release_package_views();
            }
            return Err(format!("ui_open refused: {session}"));
        }
        self.session = Some(session);
        self.deferred_snapshot = deferred_snapshot;
        {
            let mut h = host().write().unwrap();
            if let Some(s) = h.get(session) {
                let instances = s.instances;
                s.texts = vec![vec![String::new(); self.text_leaf_count]; instances];
                s.states = self
                    .last_states
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|(n, v)| {
                                let value = if deferred_snapshot {
                                    String::new()
                                } else {
                                    state_display(v)
                                };
                                (n.clone(), value)
                            })
                            .collect()
                    })
                    .collect();
                // PLAN-104：可见性按各实例状态在开窗时求值（叶序与 texts 一致）。
                s.visibility = self
                    .last_states
                    .iter()
                    .map(|row| {
                        let mut vis = Vec::new();
                        mark_leaf_visibility(&self.view_items, row, true, &mut vis)
                            .unwrap_or_default();
                        vis
                    })
                    .collect();
                s.buttons = self
                    .buttons
                    .iter()
                    .map(|b| (b.label.clone(), b.msg.clone()))
                    .collect();
                // PLAN-104：元素 label 展示行（元素标签不是 text 叶）。
                s.labels = self
                    .last_states
                    .iter()
                    .map(|row| {
                        let mut labels = Vec::new();
                        collect_element_labels(&self.view_items, row, &mut labels);
                        labels
                    })
                    .collect();
                s.inputs = self
                    .inputs
                    .iter()
                    .map(|i| (i.field.clone(), i.numeric))
                    .collect();
                let focus = frames.iter().enumerate().rev().find_map(|(instance, frame)| {
                    frame.focus_target().map(|target| package_view::focus_id(session, instance, target))
                });
                s.package_frames = frames;
                if !s.package_frames.is_empty() {
                    s.package_images = Some(Arc::new(package_images::PackageImages::new(self.public_assets.clone())));
                }
                s.publish_focus(focus);
                let has_authored_buttons = self.buttons.len() > self.route_button_spans.len();
                if s.package_frames.is_empty() && self.routes.is_none()
                    && self.inputs.is_empty()
                    && unsupported_widget_condition(&self.view_items, false).is_none()
                    && (widgets::contains_svg(&self.view_items) || (!has_authored_buttons
                        && widgets::contains_preview(&self.view_items)))
                {
                    let mut shape = ViewShape::default();
                    project_view_shape(&self.view_items, &mut shape, false).expect("view shape was checked during load");
                    let bindings = Arc::new(
                        shape
                            .button_spans
                            .into_iter()
                            .chain(self.route_button_spans.iter().copied())
                            .enumerate()
                            .map(|(target, span)| (span, target))
                            .collect(),
                    );
                    s.widget_tree = Some(self.view_items.clone().into());
                    s.widget_instances = vec![
                        widgets::State::new(self.widget_initial_bools.clone())
                            .bind_buttons(bindings)
                            .with_public_assets(self.public_assets.clone());
                        instances
                    ];
                    s.widget_values = self
                        .last_states
                        .iter()
                        .map(|row| Arc::from(row.as_slice()))
                        .collect();
                }
            }
        }
        if !deferred_snapshot {
            if let Err(error) = self.apply_initial_snapshot() {
                let _ = self.close_lane(mounted_here);
                return Err(error);
            }
        }
        if let Some(BridgeEngine::Package(engine)) = self.engine.as_mut() {
            engine.start_timers(std::time::Instant::now());
        }
        Ok(session)
    }

    fn mount_package_views(&mut self) -> Result<(Vec<Arc<PackageFrame>>, bool), String> {
        let Some(BridgeEngine::Package(engine)) = self.engine.as_mut() else {
            return Ok((Vec::new(), false));
        };
        if !engine.has_checked_view() {
            return Ok((Vec::new(), false));
        }
        let retained = self.identities.iter()
            .filter_map(|id| engine.package_frame(id))
            .collect::<Vec<_>>();
        if !retained.is_empty() {
            if retained.len() != self.identities.len() {
                return Err("checked package has an incomplete retained mount".into());
            }
            return Ok((retained, false));
        }
        let mut frames = Vec::new();
        for id in &self.identities {
            if let Err(error) = engine.mount(id, Vec::new()) {
                for mounted in &self.identities[..frames.len()] {
                    engine.unmount(mounted);
                }
                return Err(error);
            }
            frames.push(engine.package_frame(id).expect("mounted package frame"));
        }
        Ok((frames, true))
    }

    fn release_package_views(&mut self) {
        if let Some(BridgeEngine::Package(engine)) = self.engine.as_mut() {
            if engine.has_checked_view() {
                for id in &self.identities {
                    engine.unmount(id);
                }
            }
        }
    }

    fn apply_initial_snapshot(&self) -> Result<(), String> {
        // 文本叶（AV 真值）逐实例逐叶提交（view 即载荷，显式句柄）。
        for i in 0..self.instances {
            for leaf in 0..self.text_leaf_count {
                let text = self.last_texts.get(i).and_then(|row| row.get(leaf))
                    .map_or("", String::as_str);
                let rc = av_call(&format!(
                    "ui_apply_text({}, {}, 0)",
                    escape(text),
                    i * 1000 + leaf
                ))?;
                if rc != ui980_status::OK {
                    return Err(format!("initial text apply leaf {leaf} refused: {rc}"));
                }
            }
        }
        // 状态初值提交（每实例每字段；int/bool 走当前 session 热路径）。
        for (i, row) in self.last_states.iter().enumerate() {
            for (slot, (_, v)) in row.iter().enumerate() {
                self.apply_value(i, slot, v)?;
            }
        }
        Ok(())
    }

    fn apply_value(
        &self,
        instance: usize,
        slot: usize,
        value: &AceStateValue,
    ) -> Result<(), String> {
        let packed = instance * 1000 + slot;
        let body = match value {
            AceStateValue::Int(v) => {
                format!("ui_apply_int(\"apply-int\", {packed}, {v})")
            }
            AceStateValue::Bool(v) => {
                // 0/1 标量（provider 面也是 i32；见上方 provider 声明注释）。
                format!("ui_apply_bool(\"apply-bool\", {packed}, {})", *v as i32)
            }
            AceStateValue::Str(v) => {
                format!("ui_apply_str({}, {packed}, 0)", escape(v))
            }
            // PLAN-981 F32 消费同窗：呈现=Rust 32 Display（可往返、与地区无关）；
            // 宿主显示模型只有 str 车道，故 F32 以格式化串提交（值语义仍在引擎）。
            AceStateValue::F32(v) => {
                format!("ui_apply_str({}, {packed}, 0)", escape(&format!("{v}")))
            }
        };
        let rc = av_call(&body)?;
        if rc != ui980_status::OK {
            return Err(format!("apply `{body}` refused: {rc}"));
        }
        if let (Some(id), Ok(mut host)) = (self.session, host().write()) {
            if let Some(row) = host
                .get(id)
                .and_then(|session| session.widget_values.get_mut(instance))
            {
                if let Some((_, published)) = Arc::make_mut(row).get_mut(slot) {
                    *published = value.clone();
                }
            }
        }
        Ok(())
    }

    /// 受控事件步：注入到期脚本 → AV poll → AV 重放 dispatch → 差量 apply。
    pub fn step(&mut self) -> StepOutcome {
        self.tick_no += 1;
        self.run_due_script();
        // PLAN-108（T-03c M2）：收取在途 Http 派发（不阻塞）；完成/迟到
        // 落账或丢弃（世代核对——卸载后迟到结果隔离）。
        if let Some(BridgeEngine::Component(engine)) = self.engine.as_mut() {
            for completion in engine.poll_http_completions() {
                let Some(instance) = self.identities.iter().position(|id| id == &completion.identity) else {
                    continue;
                };
                match completion.result {
                    Ok(states) => {
                        let snapshot: Vec<(String, String)> = states
                            .iter()
                            .map(|(n, v)| (n.clone(), state_display(v)))
                            .collect();
                        self.dispatch_log
                            .push((instance, completion.msg.clone(), snapshot));
                        let _ = self.diff_and_apply(instance, &states);
                    }
                    Err(reason) => {
                        // 迟到丢弃 / worker 失败：具名记录（取消与迟到隔离面）。
                        self.dispatch_log.push((instance, completion.msg.clone(), Vec::new()));
                        self.errors.push(format!("http completion dropped: {reason}"));
                    }
                }
            }
        }
        self.step_component_ticks();
        let shell_outcome = self.step_lane();
        let Some(routes) = self.routes.as_mut() else {
            return shell_outcome;
        };
        let Some(shell_session) = self.session else {
            return shell_outcome;
        };
        let (outcome, action) = if let Some(action) = self.route_request.take() {
            (shell_outcome, Some(action))
        } else if let (StepOutcome::Idle, Some(page)) = (&shell_outcome, routes.page_mut()) {
            if let Some(session) = page.session {
                select_host_session(session);
            }
            let outcome = page.step_lane();
            (outcome, page.route_request.take())
        } else {
            (shell_outcome, None)
        };
        let result = action.map_or(Ok(()), |action| routes.navigate(action, self.instances));
        select_host_session(shell_session);
        if let Err(error) = result {
            self.errors.push(error.clone());
            return StepOutcome::LaneError(error);
        }
        outcome
    }

    /// PLAN-108：组件面 Tick 合成——程序含 `.Tick`+`interval` 时按间隔
    /// （250ms 语义）派发 Tick；挂起期照常（M3 park/resume：非 http 派发
    /// 同步提交）——032 探针的挂起期可交互见证面。
    fn step_component_ticks(&mut self) {
        let interval = match self.engine.as_ref() {
            Some(BridgeEngine::Component(e)) => e.tick_interval_ms(),
            _ => None,
        };
        let Some(interval) = interval else { return };
        let live: Vec<usize> = match self.engine.as_ref() {
            Some(BridgeEngine::Component(e)) => e
                .live_identities()
                .iter()
                .filter_map(|id| self.identities.iter().position(|x| x == id))
                .collect(),
            _ => return,
        };
        for instance in live {
            // 宿主会话须已挂该实例的 widget 面（close 后/reopen 重发布前的
            // 窗口跳过——对新会话发 apply 会 SESSION_CLOSED/-2 噪音）。
            // close→reopen 窗口跳过（宿主会话已关；对新会话的 apply 会
            // SESSION_CLOSED 噪音）。post-reopen 的 provider 侧重绑缺口
            // 见 PLAN-108 §9 已知项（显示层，模型语义不受影响）。
            let mounted = self.session.is_some_and(|session| {
                host().read().is_ok_and(|host| {
                    host.sessions
                        .iter()
                        .find(|s| s.id == session)
                        .is_some_and(|s| !s.closed)
                })
            });
            if !mounted {
                continue;
            }
            while self.tick_accum_ms.len() <= instance {
                self.tick_accum_ms.push(0);
            }
            let acc = &mut self.tick_accum_ms[instance];
            *acc += 20;
            if *acc >= interval {
                *acc = 0;
                self.dispatch_event(instance, "Tick", None);
            }
        }
    }

    fn step_lane(&mut self) -> StepOutcome {
        let Some(session) = self.session else {
            return StepOutcome::Idle;
        };
        if self.has_checked_package_view() {
            return self.step_package_lane(session);
        }
        let code = match av_call(&format!("ui_poll_event(\"poll\", {session}, 0)")) {
            Ok(c) => c,
            Err(e) => {
                self.errors.push(e);
                return StepOutcome::LaneError("poll failed".into());
            }
        };
        if code == 0 {
            return StepOutcome::Idle;
        }
        if code < 0 {
            let msg = format!("ui_poll_event refused: {code}");
            self.errors.push(msg.clone());
            return StepOutcome::LaneError(msg);
        }
        let instance = (code / 1000) as usize - 1;
        let target = (code % 1000) as usize - 1;
        if let Some(action) = self.route_actions.get(&target) {
            let Some(state) = self.last_states.get(instance) else {
                return StepOutcome::DispatchRefused(format!("ace.route.event: unknown instance {instance}"));
            };
            return match action.native(state) {
                Ok(navigation) => {
                    self.route_request = Some(navigation);
                    StepOutcome::Idle
                }
                Err(error) => StepOutcome::DispatchRefused(error),
            };
        }
        // B-AV U01：目标编号统一解析——buttons 段（0..buttons.len()）走
        // unit 点击；inputs 段（buttons.len() 起）走 typed 键入（载荷=桥面
        // FIFO 配对的键入全文；int 绑定桥面解析，非数值=非法输入具名拒零
        // 效果）。先解析全部 owned 数据再借引擎，避免借用交叠。
        enum Plan {
            Unit {
                msg: String,
            },
            Typed {
                msg: String,
                args: Vec<AceStateValue>,
            },
        }
        let plan = if let Some(button) = self.buttons.get(target) {
            let msg = button.msg.clone();
            match &button.payload {
                Some(payload) => {
                    let Some(state) = self.last_states.get(instance) else {
                        return StepOutcome::DispatchRefused(format!("unknown instance {instance}"));
                    };
                    match payload.iter().map(|arg| arg.read(state)).collect::<Result<_, _>>() {
                        Ok(args) => Plan::Typed { msg, args },
                        Err(error) => return StepOutcome::DispatchRefused(error),
                    }
                }
                None => Plan::Unit { msg },
            }
        } else {
            let Some(spec) = target
                .checked_sub(self.buttons.len())
                .and_then(|i| self.inputs.get(i))
                .cloned()
            else {
                let msg = format!("event target {target} has no button/input binding");
                self.errors.push(msg.clone());
                return StepOutcome::LaneError(msg);
            };
            // 配对取文：与 push 同序 FIFO（同 (instance, target) 多次键入
            // 依序消费；无配对文本=协议断裂，具名拒）。
            let pos = self
                .pending_texts
                .iter()
                .position(|(i, t, _)| *i == instance && *t == target);
            let Some(pos) = pos else {
                let msg = format!("input event target {target} has no paired typed text");
                self.errors.push(msg.clone());
                return StepOutcome::LaneError(msg);
            };
            let (_, _, text) = self.pending_texts.remove(pos).unwrap();
            let arg = if spec.numeric {
                match text.trim().parse::<i32>() {
                    Ok(v) => AceStateValue::Int(v),
                    Err(_) => {
                        let msg = format!("invalid numeric input for `{}`: {text:?}", spec.field);
                        self.errors.push(msg.clone());
                        return StepOutcome::DispatchRefused(msg);
                    }
                }
            } else {
                AceStateValue::Str(text)
            };
            Plan::Typed {
                msg: spec.msg,
                args: vec![arg],
            }
        };
        match plan {
            Plan::Unit { msg } => self.dispatch_event(instance, &msg, None),
            Plan::Typed { msg, args } => self.dispatch_event(instance, &msg, Some(&args)),
        }
    }

    fn step_package_lane(&mut self, session: i32) -> StepOutcome {
        let Some(BridgeEngine::Package(engine)) = self.engine.as_mut() else {
            unreachable!("package lane has a package engine");
        };
        if let Some((instance, message, result)) = engine.poll_http_update() {
            self.set_package_worker_pending(instance, false);
            return match result {
                Ok(states) => self.publish_package_frame(instance, &message, states),
                Err(error) => {
                    self.errors.push(error.clone());
                    StepOutcome::DispatchRefused(error)
                }
            };
        }
        let ready = |instance: usize| self.identities.get(instance)
            .is_some_and(|identity| !engine.http_pending(identity));
        if ready(0) {
            if let Some(message) = self.pending_shortcuts.pop_front() {
                return self.dispatch_typed(0, &message, &[]);
            }
        }
        let Some((instance, revision, target, input)) = ({
            let mut host = host().write().unwrap();
            host.get(session).and_then(|front| {
                if let Some(position) = front.package_events.iter().position(|event| ready(event.instance)) {
                    let event = front.package_events.remove(position).expect("ready package event");
                    front.polled += 1;
                    front.consumed_events += 1;
                    let resolved = front.package_frame(event.instance).and_then(|frame| {
                        let target = if frame.revision == event.revision {
                            event.target
                        } else {
                            frame.queued_action_target(event.action.as_ref()?)?
                        };
                        Some((frame.revision, target))
                    });
                    if let Some((revision, target)) = resolved {
                        return Some((event.instance, revision, target, event.input));
                    }
                    front.stale_rejected += 1;
                    return None;
                }
                let first = front.package_focus_notices.iter().position(|(instance, _, _)| ready(*instance));
                let Some(first) = first else {
                    let (instance, revision, target) = front.package_images.as_ref()?.next_event(|instance, notice| {
                        if !ready(instance) {
                            return None;
                        }
                        let frame = front.package_frame(instance)?;
                        frame.queued_action_target(notice).map(|target| (frame.revision, target))
                    })?;
                    front.polled += 1;
                    return Some((instance, revision, target, PackageEventInput::Activation));
                };
                let (instance, revision, _) = &front.package_focus_notices[first];
                let position = front.package_focus_notices.iter()
                    .skip(first)
                    .take_while(|(root, frame, _)| root == instance && frame == revision)
                    .position(|(_, _, notice)| !notice.gained()).map_or(first, |offset| first + offset);
                let (instance, _, notice) = front.package_focus_notices.remove(position)?;
                front.polled += 1;
                let target = front.package_frame(instance).and_then(|frame| {
                    frame.focus_notice_target(&notice)
                        .map(|target| (instance, frame.revision, target, PackageEventInput::Activation))
                });
                if target.is_none() {
                    front.stale_rejected += 1;
                }
                target
            })
        }) else {
            return self.step_package_timers();
        };
        let id = &self.identities[instance];
        if let Some(frame) = engine.package_frame(id) {
            if let Some(path) = frame.navigation_target(target) {
                if self.package_route_handlers.is_some() {
                    self.route_request = Some(RouteNavigation::Push(path.to_owned()));
                }
                return StepOutcome::Idle;
            }
            if let Some(action) = frame.action_message(target).and_then(|message| {
                self.package_route_handlers.as_ref()?.get(message)
            }) {
                return match action.native(frame.model_values()) {
                    Ok(navigation) => {
                        self.route_request = Some(navigation);
                        StepOutcome::Idle
                    }
                    Err(error) => StepOutcome::DispatchRefused(error),
                };
            }
        }
        match engine.dispatch_frame(id, revision, target, input) {
            Ok((message, DispatchOutcome::Delivered(states))) => {
                self.publish_package_frame(instance, &message, states)
            }
            Ok((_, DispatchOutcome::InFlight)) => {
                self.set_package_worker_pending(instance, true);
                StepOutcome::Idle
            }
            Ok((_, DispatchOutcome::Dropped(reason))) => StepOutcome::DispatchRefused(format!("{reason:?}")),
            Err(error) => {
                self.errors.push(error.clone());
                StepOutcome::DispatchRefused(error)
            }
        }
    }

    fn step_package_timers(&mut self) -> StepOutcome {
        let service = self.step_todo_service();
        if !matches!(service, StepOutcome::Idle) {
            return service;
        }
        let Some(BridgeEngine::Package(engine)) = self.engine.as_mut() else {
            unreachable!("package timers require a package engine");
        };
        let Some((instance, message, outcome)) = engine.dispatch_due_timer(std::time::Instant::now()) else {
            return StepOutcome::Idle;
        };
        match outcome {
            Ok(DispatchOutcome::Delivered(states)) => self.publish_package_frame(instance, &message, states),
            Ok(DispatchOutcome::InFlight) => {
                self.set_package_worker_pending(instance, true);
                StepOutcome::Idle
            }
            Ok(DispatchOutcome::Dropped(reason)) => StepOutcome::DispatchRefused(format!("{reason:?}")),
            Err(error) => {
                self.errors.push(error.clone());
                StepOutcome::DispatchRefused(error)
            }
        }
    }

    fn publish_package_frame(
        &mut self, instance: usize, message: &str, states: Vec<(String, AceStateValue)>,
    ) -> StepOutcome {
        let Some(BridgeEngine::Package(engine)) = self.engine.as_ref() else {
            unreachable!("publishing a package frame requires a package engine");
        };
        let frame = engine.package_frame(&self.identities[instance]).expect("checked dispatch produced a frame");
        let states: Vec<_> = states.iter().map(|(name, value)| (name.clone(), state_display(value))).collect();
        if let Some(session) = self.session {
            let mut host = host().write().unwrap();
            if let Some(front) = host.get(session) {
                let focus = frame.focus_target().map(|target| package_view::focus_id(session, instance, target));
                let pending_before = front.package_focus_notices.len();
                front.package_focus_notices.retain(|(root, _, notice)| {
                    *root != instance || frame.focus_notice_target(notice).is_some()
                });
                front.stale_rejected += (pending_before - front.package_focus_notices.len()) as u64;
                let pending_before = front.package_events.len();
                front.package_events.retain(|event| {
                    event.instance != instance || event.action.as_ref()
                        .is_none_or(|notice| frame.queued_action_target(notice).is_some())
                });
                front.stale_rejected += (pending_before - front.package_events.len()) as u64;
                front.package_frames[instance] = frame;
                front.publish_focus(focus);
                front.applies += 1;
            }
        }
        self.dispatch_log.push((instance, message.to_owned(), states.clone()));
        StepOutcome::Dispatched { instance, msg: message.to_owned(), states }
    }

    fn set_package_worker_pending(&self, instance: usize, pending: bool) {
        if let Some(session) = self.session {
            if let Some(front) = host().write().unwrap().get(session) {
                front.package_workers_pending[instance] = pending;
            }
        }
    }

    /// Dispatch a typed host event to its current lane using the normal AV and frame path.
    pub fn dispatch_typed(
        &mut self,
        instance: usize,
        msg: &str,
        args: &[AceStateValue],
    ) -> StepOutcome {
        if !self.can_dispatch_to_current_lane() {
            return StepOutcome::LaneError("typed event refused: no current display lane".into());
        }
        self.dispatch_event(instance, msg, Some(args))
    }

    /// Grant a directory to one mounted checked widget instance.
    pub fn grant_file_read(&self, instance: usize, root: &std::path::Path) -> Result<(), String> {
        let (engine, identity) = self.file_instance(instance)?;
        engine.grant_file_read(identity, root)
    }

    /// Grant the trusted host's selected operations to a mounted instance.
    pub fn grant_file_access(
        &self, instance: usize, root: &std::path::Path,
        writes: auto_lang_core::file_session::FileWritePermissions,
    ) -> Result<(), String> {
        let (engine, identity) = self.file_instance(instance)?;
        engine.grant_file_access(identity, root, writes)
    }

    pub fn revoke_file_read(&self, instance: usize) -> Result<(), String> {
        let (engine, identity) = self.file_instance(instance)?;
        engine.revoke_file_read(identity)
    }

    fn file_instance(&self, instance: usize) -> Result<(&PackageStateEngine, &str), String> {
        let Some(BridgeEngine::Package(engine)) = self.engine.as_ref() else {
            return Err("ace.desktop.file-session: no checked widget package is mounted".into());
        };
        let identity = self.identities.get(instance).ok_or("ace.desktop.file-session: unknown instance")?;
        Ok((engine, identity))
    }

    pub(crate) fn can_dispatch_to_current_lane(&self) -> bool {
        self.session.is_some()
            && current_session(|s| s.id) == self.session
            && auto_lang_ace_host::provider_is_admitted(UI980_PROVIDER_NAME)
    }

    fn dispatch_event(
        &mut self,
        instance: usize,
        msg_name: &str,
        args: Option<&[AceStateValue]>,
    ) -> StepOutcome {
        let Some(engine) = self.engine.as_mut() else {
            let msg = "static-face source has no event engine".to_string();
            self.errors.push(msg.clone());
            return StepOutcome::LaneError(msg);
        };
        let Some(identity) = self.identities.get(instance) else {
            return StepOutcome::DispatchRefused(format!("unknown instance {instance}"));
        };
        // PLAN-113 T-03：宿主桥在引擎派发**之前**镜像消息（app 面惰性
        // handler 跳过不阻塞真实执行；样本语义与真实 SQL 结果两层独立）。
        let states_snapshot = self.last_states.get(instance).cloned();
        if let (Some(bridge), Some(states)) = (self.sqlite_bridge.as_mut(), states_snapshot.as_ref())
        {
            bridge.notify(msg_name, states);
        }
        let dispatch_result = engine.dispatch(identity, msg_name, args);
        match dispatch_result {
            Ok((DispatchOutcome::Delivered(states), _))
                if matches!(engine, BridgeEngine::Package(engine) if engine.has_checked_view()) =>
            {
                self.publish_package_frame(instance, msg_name, states)
            }
            Ok((DispatchOutcome::Delivered(states), texts)) => {
                if let Err(e) = self.refresh_texts(instance, texts) {
                    self.errors.push(format!("text refresh: {e}"));
                }
                let snapshot: Vec<(String, String)> = states
                    .iter()
                    .map(|(n, v)| (n.clone(), state_display(v)))
                    .collect();
                self.dispatch_log
                    .push((instance, msg_name.to_owned(), snapshot));
                let states = self.diff_and_apply(instance, &states);
                self.deferred_snapshot = false;
                StepOutcome::Dispatched {
                    instance,
                    msg: msg_name.to_owned(),
                    states,
                }
            }
            Ok((DispatchOutcome::InFlight, _)) => {
                if matches!(engine, BridgeEngine::Package(_)) {
                    self.set_package_worker_pending(instance, true);
                }
                // PLAN-108：Http 效应派发已上 worker 线程在途——本拍不阻塞；
                // 完成经 step() 的 poll_http_completions 回写（dispatch_log
                // 与状态发布在 completion 臂统一发生，证据不提前）。
                StepOutcome::Idle
            }
            // PLAN-108：HttpBusy=同实例派发worker在途的正常背压（Tick 拍
            // 期间恒见）——静默，不记错误。
            Ok((DispatchOutcome::Dropped(DropReason::HttpBusy), _)) => StepOutcome::Idle,
            Ok((DispatchOutcome::Dropped(reason), _)) => {
                let msg = format!("event dropped: {reason:?}");
                self.errors.push(msg.clone());
                StepOutcome::DispatchRefused(msg)
            }
            // 未声明消息等：ACE 面具名拒（引擎已回滚，不被毒化）——无 UI 效果。
            Err(e) => {
                self.errors.push(e.clone());
                StepOutcome::DispatchRefused(e)
            }
        }
    }

    /// Apply changed text slots. Package dispatch has already evaluated its
    /// text row. The component session reads its current text without replay.
    fn refresh_texts(&mut self, instance: usize, texts: Option<Vec<String>>) -> Result<(), String> {
        if self.text_leaf_count == 0 || self.expanded_static {
            // PLAN-104：无模块作用域文本叶（循环体模板叶）——states-only 面，
            // 没有可刷新的叶。
            return Ok(());
        }
        let row = match (texts, self.engine.as_mut()) {
            (Some(row), _) => row,
            (None, Some(BridgeEngine::Component(engine))) => {
                engine.read_texts(&self.identities[instance])?
            }
            (None, _) => return Ok(()),
        };
        // Diff against the rendered session, which may still have deferred text.
        let current: Vec<String> = host()
            .read()
            .ok()
            .and_then(|h| {
                h.current.and_then(|id| {
                    h.sessions
                        .iter()
                        .find(|s| s.id == id)
                        .and_then(|s| s.texts.get(instance).cloned())
                })
            })
            .unwrap_or_default();
        for (leaf, text) in row.iter().enumerate() {
            let same = current.get(leaf).map(|c| c == text).unwrap_or(false);
            if same {
                continue;
            }
            let rc = av_call(&format!(
                "ui_apply_text({}, {}, 0)",
                escape(text),
                instance * 1000 + leaf
            ))?;
            if rc != ui980_status::OK {
                return Err(format!("refresh text leaf {leaf} refused: {rc}"));
            }
        }
        self.last_texts[instance] = row;
        Ok(())
    }

    /// 差量提交：只对变化的 (实例, 字段) 发 AV apply（实际 UI 效果载体）。
    fn diff_and_apply(
        &mut self,
        instance: usize,
        states: &[(String, AceStateValue)],
    ) -> Vec<(String, String)> {
        let mut shown = Vec::new();
        let row: Vec<(String, AceStateValue)> = states.to_vec();
        for (slot, (name, value)) in row.iter().enumerate() {
            let changed = self
                .last_states
                .get(instance)
                .and_then(|prev| prev.get(slot))
                .map(|(pn, pv)| pn == name && pv != value)
                .unwrap_or(true);
            if changed || self.deferred_snapshot {
                if let Err(e) = self.apply_value(instance, slot, value) {
                    self.errors.push(e);
                }
            }
            shown.push((name.clone(), state_display(value)));
        }
        // 全行写回（含未变化槽——保持 last 与引擎一致）。
        if let Some(prev) = self.last_states.get_mut(instance) {
            *prev = row;
        }
        // PLAN-104：状态变化后重算该实例的条件可见性与元素 label 展示行。
        if let Some(row) = self.last_states.get(instance).cloned() {
            let mut vis = Vec::new();
            let _ = mark_leaf_visibility(&self.view_items, &row, true, &mut vis);
            let mut labels = Vec::new();
            collect_element_labels(&self.view_items, &row, &mut labels);
            if let Ok(mut h) = host().write() {
                if let Some(id) = self.session {
                    if let Some(s) = h.get(id) {
                        if s.visibility.len() <= instance {
                            s.visibility.resize(instance + 1, Vec::new());
                        }
                        s.visibility[instance] = vis;
                        if s.labels.len() <= instance {
                            s.labels.resize(instance + 1, Vec::new());
                        }
                        s.labels[instance] = labels;
                    }
                }
            }
        }
        shown
    }

    /// 关闭当前 session（AV `ui_close`）；后续 apply/poll 具名拒、late 事件丢弃。
    pub fn close_session(&mut self) -> Result<(), String> {
        self.close_lane(true)
    }

    /// ASE switches withdraw the output lane while the checked mount stays live.
    pub(crate) fn hide_ase_session(&mut self) -> Result<(), String> {
        self.close_lane(false)
    }

    fn close_lane(&mut self, release_mount: bool) -> Result<(), String> {
        let Some(session) = self.session else {
            if release_mount {
                self.release_package_views();
            }
            return Ok(());
        };
        let route_result = self.routes.as_mut().map_or(Ok(()), |routes| routes.close());
        self.route_request = None;
        let rc = av_call(&format!("ui_close(\"close\", {session}, 0)"))?;
        if rc != ui980_status::OK {
            return Err(format!("ui_close refused: {rc}"));
        }
        self.session = None;
        if release_mount {
            self.release_package_views();
        }
        // B-AV U01：键入配对随 session 关闭一并丢弃（provider 队列已清，
        // 残留文本会成为无码孤儿）。
        self.pending_texts.clear();
        self.pending_shortcuts.clear();
        route_result
    }

    /// Release the host-owned copy at shutdown even when its provider refuses closure.
    pub(crate) fn close_for_shutdown(&mut self) -> Result<(), String> {
        let result = self.close_session();
        if result.is_err() {
            if let Some(session) = self.session.take() {
                close_host_session(session);
            }
            self.pending_texts.clear();
            self.pending_shortcuts.clear();
            self.release_package_views();
        }
        result
    }

    /// 重开：新 session（新世代），按当前业务状态重新提交显示模型。
    pub fn reopen(&mut self) -> Result<i32, String> {
        self.open()
    }

    /// 安装事件脚本（tick 从 1 起；CPU 测试与 e2e exe 同一边界）。
    pub fn install_script(&mut self, script: Vec<(usize, ScriptStep)>) {
        self.script = script;
    }

    /// PLAN-113 T-03：安装真实 SQLite 宿主桥（V-03 opt-in 入口）。
    pub fn install_sqlite_bridge(
        &mut self,
        bridge: crate::ui::ace_sqlite_bridge::SqliteHostBridge,
    ) {
        self.sqlite_bridge = Some(bridge);
    }

    fn run_due_script(&mut self) {
        let due: Vec<ScriptStep> = self
            .script
            .iter()
            .filter(|(t, _)| *t == self.tick_no)
            .map(|(_, s)| s.clone())
            .collect();
        for step in due {
            match step {
                ScriptStep::Click { instance, target } => {
                    if let Some(session) = self.session {
                        push_script_click(session, instance, target);
                    } else {
                        push_event(instance, target);
                    }
                }
                ScriptStep::ClickPage { instance, target } => {
                    if let Some(session) = self.routes.as_ref().and_then(|routes| routes.page_session()) {
                        push_script_click(session, instance, target);
                    }
                }
                ScriptStep::TypeText {
                    instance,
                    target,
                    text,
                } => self.inject_text(instance, target, &text),
                ScriptStep::Close => {
                    if let Err(e) = self.close_session() {
                        self.errors.push(e);
                    }
                }
                ScriptStep::Reopen => {
                    if let Err(e) = self.reopen() {
                        self.errors.push(e);
                    }
                }
                ScriptStep::LateClick => {
                    if let Some(stale) = last_closed_session() {
                        push_late_event(stale);
                    }
                }
            }
        }
    }

    pub(crate) fn accepts_clock_tick(&self) -> bool {
        matches!(&self.engine, Some(BridgeEngine::Component(engine)) if engine.accepts_clock_tick())
    }

    pub(crate) fn close_engine(&mut self) -> Option<EngineExecutionAudit> {
        self.release_package_views();
        let Some(BridgeEngine::Component(engine)) = self.engine.as_mut() else {
            return None;
        };
        for identity in &self.identities {
            engine.unmount(identity);
        }
        Some(engine.execution_audit())
    }

    pub fn execution_audit(&self) -> Option<EngineExecutionAudit> {
        match &self.engine {
            Some(BridgeEngine::Component(engine)) => Some(engine.execution_audit()),
            _ => None,
        }
    }

    /// 当前各实例业务状态（AV 真值；证据/断言消费）。
    pub fn states(&self, instance: usize) -> Option<Vec<(String, AceStateValue)>> {
        self.engine.as_ref()?.read_all(&self.identities[instance])
    }

    /// 脚本是否已全部注入（e2e 收口观察面之一）。
    pub fn script_done(&self) -> bool {
        self.script.iter().all(|(t, _)| *t <= self.tick_no)
    }

    /// A refused input still finishes once the host has taken it from the queue.
    pub fn script_settled(&self) -> bool {
        let expected_events = self
            .script
            .iter()
            .filter(|(_, step)| {
                matches!(
                    step,
                    ScriptStep::Click { .. } | ScriptStep::ClickPage { .. } | ScriptStep::TypeText { .. }
                )
            })
            .count();
        self.script_done()
            && host().read().is_ok_and(|host| {
                host.sessions.iter().map(|s| s.consumed_events).sum::<usize>() >= expected_events
            })
    }

    /// 宿主侧全部证据（JSON 面；exe/test 消费）。
    pub fn evidence(&self) -> serde_json::Value {
        let sessions = host()
            .read()
            .map(|h| {
                h.sessions
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "generation": s.generation,
                            "closed": s.closed,
                            "applies": s.applies,
                            "polled": s.polled,
                            "late_dropped": s.late_dropped,
                            "stale_rejected": s.stale_rejected,
                            "texts": s.texts,
                            "labels": s.labels,
                            "states": s.states,
                            "widget_state": s.widget_instances.iter().map(widgets::State::evidence).collect::<Vec<_>>(),
                            "applied": s.applied.iter().map(|a| serde_json::json!({
                                "instance": a.instance, "kind": a.kind,
                                "slot": a.slot, "value": a.value,
                            })).collect::<Vec<_>>(),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        serde_json::json!({
            "source_path": self.path,
            "session": self.session,
            "node": self.node_name,
            "buttons": self.buttons.iter().map(|b| serde_json::json!({
                "label": b.label, "msg": b.msg,
            })).collect::<Vec<_>>(),
            "field_names": self.field_names,
            "dispatch_log": self.dispatch_log.iter().map(|(i, m, st)| serde_json::json!({
                "instance": i, "msg": m, "states": st,
            })).collect::<Vec<_>>(),
            "errors": self.errors,
            "sessions": sessions,
            "frames_rendered": NODE_DESKTOP_FRAMES.load(Ordering::Relaxed),
            "routes": self.routes.as_ref().map(|routes| routes.evidence()),
            "sqlite_bridge": self.sqlite_bridge.as_ref().map(|b| b.evidence()),
        })
    }
}

/// 状态值显示串（宿主显示模型；业务值仍来自 AV）。
fn state_display(v: &AceStateValue) -> String {
    match v {
        AceStateValue::Int(i) => i.to_string(),
        AceStateValue::Bool(b) => b.to_string(),
        AceStateValue::Str(s) => s.clone(),
        AceStateValue::F32(f) => format!("{f}"),
    }
}

// ---------------------------------------------------------------------------
// 窗口组件（run_app 泛型宿主；tick 受控调用点驱动桥）
// ---------------------------------------------------------------------------

static BRIDGE: OnceLock<RwLock<Option<NodeDesktopBridge>>> = OnceLock::new();

pub fn install_bridge(bridge: NodeDesktopBridge) {
    let cell = BRIDGE.get_or_init(|| RwLock::new(None));
    *cell.write().unwrap() = Some(bridge);
}

pub fn with_bridge<R>(f: impl FnOnce(&mut NodeDesktopBridge) -> R) -> Option<R> {
    let cell = BRIDGE.get()?;
    let mut guard = cell.write().ok()?;
    guard.as_mut().map(f)
}

#[derive(Clone, Debug)]
pub enum NodeDesktopMsg {
    Click {
        instance: usize,
        target: usize,
    },
    /// B-AV U01：文本输入事件（键入全文经 iced INPUT_TEXT 线程局部暂存，
    /// on() 处理时读取——与 View::Input on_change 的既有机制同口）。
    Type {
        instance: usize,
        target: usize,
    },
    SessionClick { session: i32, instance: usize, target: usize },
    SessionType { session: i32, instance: usize, target: usize },
    Pointer {
        session: i32,
        instance: usize,
        message: String,
        coordinates: Option<(f32, f32)>,
    },
    PackageClick { session: i32, instance: usize, revision: u64, target: usize },
    PackagePointer {
        session: i32,
        instance: usize,
        revision: u64,
        target: usize,
        x: f32,
        y: f32,
    },
    PackageText { session: i32, instance: usize, revision: u64, target: usize, text: String },
    PackageImageSurface {
        session: i32,
        instance: usize,
        revision: u64,
        target: usize,
        carry: ImageSurfaceCarry,
    },
    PackageFocus { session: i32, instance: usize, revision: u64, control: String, gained: bool },
    PackageShortcut { session: i32, message: String },
    Widget { session: i32, instance: usize, action: WidgetAction },
    BridgeTick,
}

/// MIN-A-UI 桌面窗口：显示模型渲染自 UI session（AV→FFI 提交的真实值），
/// 点击入 typed 队列，tick 步进桥（poll→AV 重放→apply——全部主线程受控点）。
#[derive(Debug, Default)]
pub struct AceNodeDesktopWindow;

impl Component for AceNodeDesktopWindow {
    type Msg = NodeDesktopMsg;

    fn take_focus_request(&mut self) -> Option<crate::ui::component::FocusRequest> {
        let page = with_bridge(|bridge| bridge.routes.as_deref().and_then(DesktopRoutes::page_session)).flatten();
        let mut host = host().write().ok()?;
        let current = host.current;
        page.into_iter().chain(current).find_map(|session| {
            let front = host.get(session)?;
            if front.closed { return None }
            front.focus_request.take()
        })
    }

    fn key_bindings(&self) -> std::collections::HashMap<String, String> {
        let mut bindings = with_bridge(|bridge| bridge.shortcuts.clone().into_iter().collect::<std::collections::HashMap<_, _>>())
            .unwrap_or_default();
        if widget_keyboard_session().is_some() {
            for key in widgets::KEYS {
                bindings.entry(key.to_string()).or_insert_with(|| key.to_string());
            }
        }
        bindings
    }

    fn key_message(&self, key: &str) -> Option<Self::Msg> {
        with_widget_view(|session, tree, texts, values, state| {
            let action = widgets::keyboard_action(tree, texts, values, state, key)?;
            Some(NodeDesktopMsg::Widget {
                session,
                instance: 0,
                action,
            })
        })
        .flatten()
        .or_else(|| with_bridge(|bridge| Some(NodeDesktopMsg::PackageShortcut {
            session: bridge.session?,
            message: bridge.shortcuts.get(key)?.clone(),
        })).flatten())
    }

    #[cfg(windows)]
    fn native_accessibility_enabled() -> bool {
        widget_keyboard_session().is_some()
    }

    #[cfg(windows)]
    fn native_accessibility_tree(&self) -> Option<accesskit::TreeUpdate> {
        with_widget_view(widgets::accessibility_tree)
    }

    #[cfg(windows)]
    fn native_accessibility_message(&self, request: accesskit::ActionRequest) -> Option<Self::Msg> {
        with_widget_view(|session, tree, texts, values, state| {
            let action = widgets::accessibility_action(session, tree, texts, values, state, request)?;
            Some(NodeDesktopMsg::Widget {
                session,
                instance: 0,
                action,
            })
        })
        .flatten()
    }

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            NodeDesktopMsg::Click { instance, target } => push_event(instance, target),
            NodeDesktopMsg::Type { instance, target } => {
                // 真实键入全文（iced on_input 暂存）→ 桥注入（push_event
                // + 配对文本，与脚本 TypeText 同一边界）。
                let text = crate::ui::iced::renderer::last_input_text();
                with_bridge(|b| b.inject_text(instance, target, &text));
            }
            NodeDesktopMsg::SessionClick { session, instance, target } => {
                push_session_event(session, instance, target);
            }
            NodeDesktopMsg::SessionType { session, instance, target } => {
                let text = crate::ui::iced::renderer::last_input_text();
                with_bridge(|bridge| {
                    if bridge.session == Some(session) {
                        bridge.inject_text(instance, target, &text);
                    } else if let Some(page) = bridge.routes.as_mut().and_then(|routes| routes.page_mut()) {
                        if page.session == Some(session) {
                            page.inject_text(instance, target, &text);
                        }
                    }
                });
            }
            NodeDesktopMsg::Pointer { session, instance, message, coordinates } => {
                with_bridge(|bridge| bridge.dispatch_pointer(session, instance, &message, coordinates));
            }
            NodeDesktopMsg::BridgeTick => {
                with_bridge(|b| b.step());
            }
            message @ (NodeDesktopMsg::PackageClick { .. }
            | NodeDesktopMsg::PackagePointer { .. }
            | NodeDesktopMsg::PackageText { .. }
            | NodeDesktopMsg::PackageImageSurface { .. }
            | NodeDesktopMsg::PackageFocus { .. }) => {
                queue_package_message(message);
            }
            NodeDesktopMsg::PackageShortcut { session, message } => {
                with_bridge(|bridge| {
                    if bridge.session == Some(session)
                        && bridge.shortcuts.values().any(|handler| handler == &message)
                        && !bridge.pending_shortcuts.contains(&message)
                    {
                        bridge.pending_shortcuts.push_back(message);
                    }
                });
            }
            NodeDesktopMsg::Widget { session, instance, action } => {
                if let WidgetAction::DispatchButton(target) = action {
                    push_session_event(session, instance, target);
                    return;
                }
                let mut host = host().write().unwrap();
                if let Some(target) = host.get(session) {
                    if target.closed {
                        target.stale_rejected += 1;
                    } else if let Some(state) = target.widget_instances.get_mut(instance) {
                        state.apply(action);
                    }
                }
            }
        }
    }

    fn tick_interval_ms(&self) -> Option<u32> {
        Some(20)
    }

    fn tick_msg(&self) -> Option<Self::Msg> {
        Some(NodeDesktopMsg::BridgeTick)
    }

    fn view(&self) -> View<Self::Msg> {
        NODE_DESKTOP_FRAMES.fetch_add(1, Ordering::Relaxed);
        let mounted = with_bridge(|bridge| {
            bridge.routes.as_ref().map(|routes| {
                (routes.outlet_after(), routes.page_session(), routes.error().map(str::to_owned))
            })
        }).flatten();
        let Some(shell) = current_session(host_frame) else {
            return View::col()
                .padding(30)
                .child(View::text("ace-node-desktop: no session"))
                .build();
        };
        let mut dialogs = Vec::new();
        let outlet = mounted.map(|(after, page_id, diagnostic)| {
            let page = page_id.and_then(|id| {
                let host = host().read().ok()?;
                host.sessions.iter().find(|session| session.id == id && !session.closed).map(host_frame)
            });
            let view = page.map(|frame| render_host_frame(frame, None, true, &mut dialogs))
                .or_else(|| diagnostic.map(View::text))
                .unwrap_or_else(View::empty);
            (after, view)
        });
        let routed = outlet.is_some();
        let content = render_host_frame(shell, outlet, routed, &mut dialogs);
        View::col().padding(24).child(content).children(dialogs).build()
    }
}

impl NodeDesktopBridge {
    fn dispatch_pointer(
        &mut self,
        session: i32,
        instance: usize,
        message: &str,
        coordinates: Option<(f32, f32)>,
    ) {
        if self.session == Some(session) {
            self.apply_pointer(instance, message, coordinates);
            return;
        }
        if let Some(page) = self.routes.as_deref_mut().and_then(DesktopRoutes::page_mut) {
            if page.session == Some(session) {
                select_host_session(session);
                page.apply_pointer(instance, message, coordinates);
                if let Some(shell) = self.session {
                    select_host_session(shell);
                }
                return;
            }
        }
        if let Ok(mut host) = host().write() {
            if let Some(stale) = host.get(session).filter(|session| session.closed) {
                stale.late_dropped += 1;
            }
        }
    }

    fn apply_pointer(&mut self, instance: usize, message: &str, coordinates: Option<(f32, f32)>) {
        if !self.can_dispatch_to_current_lane() {
            self.errors.push(format!(
                "ace.desktop.pointer: message `{message}` refused without an admitted display lane"
            ));
            return;
        }
        match coordinates {
            Some((x, y)) => {
                self.dispatch_event(
                    instance,
                    message,
                    Some(&[AceStateValue::F32(x), AceStateValue::F32(y)]),
                );
            }
            None => {
                self.dispatch_event(instance, message, None);
            }
        }
    }
}

fn widget_keyboard_session() -> Option<i32> {
    let page = with_bridge(|bridge| {
        bridge.routes.as_deref().and_then(DesktopRoutes::page_session)
    })
    .flatten();
    let host = host().read().ok()?;
    let session = page.or(host.current)?;
    host.sessions
        .iter()
        .any(|entry| entry.id == session && !entry.closed && entry.widget_tree.is_some())
        .then_some(session)
}

fn with_widget_view<R>(
    f: impl FnOnce(
        i32,
        &[ace_core::source::age_component::ComponentViewItem],
        &[String],
        &[(String, AceStateValue)],
        &widgets::State,
    ) -> R,
) -> Option<R> {
    let session = widget_keyboard_session()?;
    let host = host().read().ok()?;
    let target = host
        .sessions
        .iter()
        .find(|entry| entry.id == session && !entry.closed)?;
    let tree = target.widget_tree.as_deref()?;
    let state = target.widget_instances.first()?;
    let texts = target.texts.first().map(Vec::as_slice).unwrap_or_default();
    let values = target.widget_values.first().map(AsRef::as_ref).unwrap_or_default();
    Some(f(session, tree, texts, values, state))
}

struct HostFrame {
    id: i32,
    generation: u64,
    texts: Vec<Vec<String>>,
    states: Vec<Vec<(String, String)>>,
    buttons: Vec<(String, String)>,
    inputs: Vec<(String, bool)>,
    visibility: Vec<Vec<bool>>,
    labels: Vec<Vec<String>>,
    widget_tree: Option<Arc<[ace_core::source::age_component::ComponentViewItem]>>,
    widget_instances: Vec<widgets::State>,
    widget_values: Vec<Arc<[(String, AceStateValue)]>>,
    package_frames: Vec<Arc<PackageFrame>>,
    package_images: Option<Arc<package_images::PackageImages>>,
}

fn host_frame(session: &HostSession) -> HostFrame {
    HostFrame {
        id: session.id,
        generation: session.generation,
        texts: session.texts.clone(),
        states: session.states.clone(),
        buttons: session.buttons.clone(),
        inputs: session.inputs.clone(),
        visibility: session.visibility.clone(),
        labels: session.labels.clone(),
        widget_tree: session.widget_tree.clone(),
        widget_instances: session.widget_instances.clone(),
        widget_values: session.widget_values.clone(),
        package_frames: session.package_frames.clone(),
        package_images: session.package_images.clone(),
    }
}

fn render_host_frame(
    frame: HostFrame,
    mut outlet: Option<(usize, View<NodeDesktopMsg>)>,
    routed: bool,
    dialogs: &mut Vec<View<NodeDesktopMsg>>,
) -> View<NodeDesktopMsg> {
    let HostFrame {
        id, generation, texts, states, buttons, inputs, visibility, labels, widget_tree,
        widget_instances, widget_values, package_frames, package_images,
    } = frame;
    if !package_frames.is_empty() {
        if let [frame] = package_frames.as_slice() {
            if (!routed || outlet.is_some()) && package_view::owns_viewport(frame) {
                let page = outlet.as_ref().map(|(_, page)| page);
                return package_view::view(id, 0, frame, package_images.as_deref(), page, routed, dialogs);
            }
        }
        let views = package_frames.iter().enumerate()
            .map(|(index, frame)| {
                let page = if index == 0 { outlet.take().map(|(_, page)| page) } else { None };
                package_view::view(id, index, frame, package_images.as_deref(), page.as_ref(), routed, dialogs)
            });
        let content = View::col().children(views).build();
        return if routed { content } else { View::scrollable(content).build() };
    }
    if let Some(tree) = widget_tree {
        let children = widget_instances
            .iter()
            .enumerate()
            .map(|(instance, widget_state)| {
                let text = texts.get(instance).map(Vec::as_slice).unwrap_or_default();
                let values = widget_values.get(instance).map(AsRef::as_ref).unwrap_or_default();
                let rendered = widgets::view(id, instance, &tree, text, values, widget_state);
                let source_buttons = widget_state.source_button_count();
                if source_buttons == buttons.len() {
                    return rendered;
                }
                let extra = buttons.iter().enumerate().skip(source_buttons).map(|(target, (label, _))| {
                    View::button_styled(
                        label.clone(),
                        NodeDesktopMsg::SessionClick { session: id, instance, target },
                        "px-3 py-1.5 rounded-lg bg-primary text-primary-foreground",
                    )
                });
                View::col().child(rendered).children(extra).build()
            });
        return View::scrollable(View::col().children(children).build()).build();
    }
    let mut root = View::col().spacing(10).child(View::text_styled(
        &format!("session generation {generation}"),
        "text-sm",
    ));
        for i in 0..texts.len().max(states.len()) {
            let mut col = View::col().spacing(6);
            col = col.child(View::text_styled(
                &format!("instance {i}"),
                "text-lg font-semibold",
            ));
            for (leaf, text) in texts.get(i).into_iter().flatten().enumerate() {
                if outlet.as_ref().is_some_and(|(after, _)| *after == leaf) {
                    col = col.child(outlet.take().expect("outlet present").1);
                }
                // PLAN-104：条件分支的隐藏叶不渲染（可见性按实例状态求值）。
                let shown = visibility
                    .get(i)
                    .and_then(|row| row.get(leaf))
                    .copied()
                    .unwrap_or(true);
                if shown {
                    col = col.child(View::text(format!("t{leaf}: {text}")));
                }
            }
            for (_slot, (name, value)) in states.get(i).into_iter().flatten().enumerate() {
                col = col.child(View::text(format!("{name} = {value}")));
            }
            // PLAN-104：元素 label（`h2 .title` / `span "✓"` 族）展示行——仅显示。
            for label in labels.get(i).into_iter().flatten() {
                col = col.child(View::text(format!("· {label}")));
            }
            // B-AV U01：输入目标（受控单向数据流——显示值=该实例状态行，
            // 键入→Type 消息→typed 队列→AV 重放→apply→回显）。
            for (idx, (field, _numeric)) in inputs.iter().enumerate() {
                let value = states
                    .get(i)
                    .and_then(|row| row.iter().find(|(n, _)| n == field).map(|(_, v)| v.clone()))
                    .unwrap_or_default();
                col = col.child(
                    View::input(format!("{field}:"))
                        .value(value)
                        .on_change(NodeDesktopMsg::SessionType {
                            session: id,
                            instance: i,
                            target: buttons.len() + idx,
                        })
                        .build(),
                );
            }
            let mut row = View::row().spacing(8);
            for (target, (label, _msg)) in buttons.iter().enumerate() {
                row = row.child(View::button_styled(
                    label.clone(),
                    NodeDesktopMsg::SessionClick {
                        session: id,
                        instance: i,
                        target,
                    },
                    "px-3 py-1.5 rounded-lg bg-primary text-primary-foreground",
                ));
            }
            col = col.child(row.build());
            if let Some((_, page)) = outlet.take() {
                col = col.child(page);
            }
            root = root.child(col.build());
        }
        root.build()
}

/// MIN-A-UI 入口（`AUTO_UI_ENGINE=ace` 消费）：装载→open（FFI 实效果）→
/// run_app 真窗（tick 驱动受控事件循环）。窗口阻塞至关闭。
pub fn run_ace_node_desktop_ui(src: &str, path: &str, instances: usize) -> Result<(), String> {
    run_ace_node_desktop_ui_with_access(src, path, instances, None)
}

pub(crate) fn run_ace_node_desktop_ui_with_access(
    src: &str,
    path: &str,
    instances: usize,
    access: Option<(&std::path::Path, auto_lang_core::file_session::FileWritePermissions)>,
) -> Result<(), String> {
    let mut bridge = NodeDesktopBridge::load(src, path, instances)?;
    match access {
        Some((root, writes)) => bridge.open_with_file_access(root, writes)?,
        None => bridge.open()?,
    };
    install_bridge(bridge);
    #[cfg(windows)]
    let _pointer_clock = crate::ui::iced::native_pointer_time::Capture::start();
    use crate::ui::iced::renderer::{run_app_with_theme, shadcn_theme};
    use crate::ui::style::theme::{dark_mode, set_dark_mode, theme_pref_from_env};
    let previous_dark = dark_mode();
    let dark = match theme_pref_from_env() {
        "light" => false,
        "system" => crate::ui::system_theme::system_prefers_dark().unwrap_or(previous_dark),
        _ => true,
    };
    set_dark_mode(dark);
    let result = run_app_with_theme::<AceNodeDesktopWindow>(shadcn_theme(dark))
        .map_err(|e| format!("node desktop window: {e:?}"));
    set_dark_mode(previous_dark);
    let bridge = BRIDGE.get().expect("installed desktop bridge")
        .write().unwrap().take();
    let close = bridge.map_or(Ok(()), |mut bridge| bridge.close_for_shutdown());
    result.and(close)
}

#[cfg(test)]
mod style_blank_tests {
    use super::blank_style_surface;

    #[test]
    fn blank_single_line_style_keeps_width() {
        let src = "widget App {\n    view { col {\n        style: \"a b\"\n    } }\n}\n";
        let out = blank_style_surface(src);
        assert_eq!(out.len(), src.len());
        assert!(!out.contains("style"));
        assert!(out.contains("view { col {"));
    }

    #[test]
    fn blank_multiline_style_until_brace_closes() {
        let src = "view { col {\n    style: if .dark {\n        \"a\"\n    } else {\n        \"b\"\n    }\n    text \"x\"\n} }\n";
        let out = blank_style_surface(src);
        assert_eq!(out.len(), src.len());
        assert!(!out.contains("style"));
        assert!(!out.contains("else"));
        assert!(out.contains("text \"x\""));
    }

    #[test]
    fn blank_handles_crlf_lines() {
        let src = "widget App {\r\n    view { col {\r\n        style: \"a\"\r\n    } }\r\n}\r\n";
        let out = blank_style_surface(src);
        assert_eq!(out.len(), src.len(), "byte width must be preserved");
        assert!(!out.contains("style"));
    }

    #[test]
    fn blank_handles_missing_trailing_newline() {
        let src = "widget App {\n    view { col {\n        style: \"a\"\n    } }";
        let out = blank_style_surface(src);
        assert_eq!(out.len(), src.len());
        assert!(!out.contains("style"));
    }

    #[test]
    fn blanked_real_006_source_projects() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for app in ["006-hero-section", "010-contact-form", "007-stats-board"] {
            let p = root.join("examples/ui").join(app).join("src/front/app.at");
            let src = std::fs::read_to_string(&p).unwrap();
            let blanked = blank_style_surface(&src);
            assert_eq!(blanked.len(), src.len(), "{app}: width preserved");
            let r = ace_core::source::age_component::project_component_candidates(&blanked);
            assert!(
                r.is_ok(),
                "{app} must project after blanking: {:?}",
                r.err().map(|ds| ds
                    .iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect::<Vec<_>>())
            );
            // 编译面同源（窗口 load 的第二阶段）：确认空白化后场景编译也通过。
            let caps = ["ac.runtime.text.v1".to_string()];
            let scenario = ace_av::age_component::ComponentScenario {
                instances: 1,
                ops: Vec::new(),
            };
            let initials = ace_av::age_component::ComponentInitials {
                per_instance: vec![Vec::new()],
            };
            if let Err(ds) = ace_av::age_component::compile_component_scenario_with_initials(
                &blanked,
                "fixture.at",
                ace_av::age_component::LegacyWidgetAlias::Accept,
                &caps,
                &scenario,
                None,
                &initials,
            ) {
                let msgs: Vec<String> = ds
                    .iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect();
                // 006/010 的视图叶全在元素 label 上（无 `text` 叶）时，文本车道
                // 会以 "requires at least one text leaf" 具名拒——这是载入分支
                // 已处理的既有边界，不算空白化失败。
                if !msgs.iter().any(|m| m.contains("requires at least one")) {
                    panic!("{app}: compile after blanking failed: {msgs:?}");
                }
            }
        }
    }
}
    #[test]
    fn blank_inline_entry_keeps_braces() {
        let src = "widget App {\n    view { col {\n        text \"x\" { style: settings_label }\n    } }\n}\n";
        let out = blank_style_surface(src);
        assert_eq!(out.len(), src.len(), "width preserved");
        assert!(!out.contains("settings_label"), "inline style ref blanked");
        assert_eq!(out.matches('{').count(), src.matches('{').count());
        assert_eq!(out.matches('}').count(), src.matches('}').count());
    }

    #[test]
    fn blank_multiline_conditional_style() {
        let src = "widget App {\n    button \"b\" {\n        style: if .a {\n            active_style\n        } else {\n            idle_style\n        }\n    }\n}\n";
        let out = blank_style_surface(src);
        assert_eq!(out.len(), src.len(), "width preserved");
        assert!(!out.contains("active_style"), "then arm blanked");
        assert!(!out.contains("idle_style"), "else arm blanked");
        assert!(!out.contains("if .a"), "the conditional block disappears whole");
        // 条件块的两个 `{`/`}` 一并消失；元素自身的大括号保留。
        assert_eq!(out.matches('{').count(), src.matches('{').count() - 2);
        assert_eq!(out.matches('}').count(), src.matches('}').count() - 2);
    }
