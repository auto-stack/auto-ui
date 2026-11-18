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

use crate::ui::ace_state_bridge::{AceStateEngine, AceStateValue, DispatchOutcome};
use crate::ui::{Component, View};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{OnceLock, RwLock};

use ace_av::age_component::{
    compile_component_scenario_with_initials, observe_component_states_and_texts,
    observe_component_texts, project_component_candidates, ComponentInitials, ComponentScenario,
    LegacyWidgetAlias,
};

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
pub fn ui_apply_bool(op: &str, instance_slot: i32, value: bool) -> i32 {
    with_hooks(|h| unsafe {
        (h.apply_bool)(op.as_ptr(), op.len() as i32, instance_slot, value as i32)
    })
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

/// 构建并注册 ui980 provider，返回 cdylib 路径（宿主安装 hooks 用）。
/// 幂等：已注册时另行经缓存构建取回 cdylib 路径。
pub fn ensure_ui980_provider() -> Result<PathBuf, String> {
    auto_lang_ace_host::install().map_err(|e| format!("ace host install: {e}"))?;
    let root = stable_ui980_root();
    match auto_lang_ace_host::register_source_provider(
        UI980_PROVIDER_NAME,
        &root,
        UI980_PROVIDER_SOURCE,
    ) {
        Ok(cdylib) => Ok(cdylib),
        Err(e) if e.contains("already registered") => {
            // 已注册（本进程早前会话）：同一构建根重放 build_source 只命中
            // 新鲜度缓存，取回 cdylib 路径。
            let build = ace_abi::rust_provider::build_source(&root, UI980_PROVIDER_SOURCE)
                .map_err(|e| format!("ui980 cached build: {e}"))?;
            Ok(build.cdylib)
        }
        Err(e) => Err(format!("ui980 provider register: {e}")),
    }
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
    /// 事件目标（声明序）：(label, msg)。
    pub buttons: Vec<(String, String)>,
    /// PLAN-980 B-AV U01（输入组件面）：文本输入目标（声明序，接 buttons
    /// 之后编号）：(绑定字段名, 数值型？)。渲染值取该实例状态行——受控单
    /// 向数据流（键入→typed 事件→AV 重放→apply→回显）。
    pub inputs: Vec<(String, bool)>,
    queue: VecDeque<(usize, usize)>,
    closed: bool,
    /// 证据：实际 UI 提交计数/轨迹、poll 计数、late/stale 拒绝计数。
    pub applies: u64,
    pub applied: Vec<AppliedUpdate>,
    pub polled: u64,
    pub late_dropped: u64,
    pub stale_rejected: u64,
}

impl HostSession {
    fn new(id: i32, generation: u64, instances: usize) -> Self {
        Self {
            id,
            generation,
            instances,
            texts: vec![Vec::new(); instances],
            states: vec![Vec::new(); instances],
            buttons: Vec::new(),
            inputs: Vec::new(),
            queue: VecDeque::new(),
            closed: false,
            applies: 0,
            applied: Vec::new(),
            polled: 0,
            late_dropped: 0,
            stale_rejected: 0,
        }
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
        Some((instance, target)) => ((instance + 1) * 1000 + (target + 1)) as i32,
        None => 0,
    }
}

unsafe extern "C" fn hook_close(op: *const u8, op_len: i32, session: i32, _reserved: i32) -> i32 {
    if !tag_matches(op, op_len, b"close") {
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
    s.closed = true;
    s.queue.clear();
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
    engine: Option<AceStateEngine>,
    identities: Vec<String>,
    pub session: Option<i32>,
    instances: usize,
    buttons: Vec<ButtonSpec>,
    /// PLAN-980 B-AV U01：输入目标（目标编号=buttons.len() 起）。
    inputs: Vec<InputSpec>,
    /// 输入事件的键入全文（FIFO，与 provider 事件队列 (instance, target)
    /// 一一配对——poll 取码、桥面取文）。
    pending_texts: VecDeque<(usize, usize, String)>,
    text_leaf_count: usize,
    load_texts: Vec<String>,
    last_states: Vec<Vec<(String, AceStateValue)>>,
    field_names: Vec<String>,
    /// 证据：dispatch 轨迹（instance, msg, 状态快照）与错误。
    pub dispatch_log: Vec<(usize, String, Vec<(String, String)>)>,
    pub errors: Vec<String>,
    script: Vec<(usize, ScriptStep)>,
    tick_no: usize,
}

/// view 树投影：文本叶计数 + 按钮目标（label=StrLit、onclick=.Msg）+
/// 输入目标（value 绑定字段 + oninput 消息）。
struct ViewShape {
    text_leaf_count: usize,
    buttons: Vec<ButtonSpec>,
    inputs: Vec<InputSpec>,
}

fn project_view_shape(
    items: &[ace_core::source::age_component::ComponentViewItem],
    shape: &mut ViewShape,
) {
    use ace_core::source::age_component::ComponentViewItem;
    use ace_core::source::ast::ExprKind;
    for item in items {
        match item {
            ComponentViewItem::Text(_) => shape.text_leaf_count += 1,
            ComponentViewItem::Entry(_) => {}
            ComponentViewItem::Element(el) => {
                if el.tag == "text" {
                    shape.text_leaf_count += 1;
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
                    for sub in &el.items {
                        if let ComponentViewItem::Entry(entry) = sub {
                            if entry.is_event && entry.key == "onclick" {
                                if let ace_core::source::age_component::ComponentEntryValue::Expr(
                                    expr,
                                ) = &entry.value
                                {
                                    if let ExprKind::Field { base, name } = &expr.kind {
                                        if matches!(&base.kind, ExprKind::Var(v) if v.name == "self")
                                        {
                                            msg = Some(name.name.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(msg) = msg {
                        shape.buttons.push(ButtonSpec { label, msg });
                    }
                } else {
                    project_view_shape(&el.items, shape);
                }
            }
        }
    }
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
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join("; ")
}

impl NodeDesktopBridge {
    /// 装载：投影 view 形状（文本叶/按钮目标）+ 文本叶 AV 真值（静态观察面）
    /// + states-only 引擎挂载（keyed 实例、世代纪律；无 model/msg 的 Hello 类
    /// 源为静态面——事件面缺席具名可辨）。
    pub fn load(src: &str, path: &str, instances: usize) -> Result<Self, String> {
        let candidates = project_component_candidates(src).map_err(|ds| diags(&ds))?;
        let node = candidates
            .first()
            .ok_or_else(|| "no AGE node candidate in source".to_string())?;
        let mut shape = ViewShape {
            text_leaf_count: 0,
            buttons: Vec::new(),
            inputs: Vec::new(),
        };
        project_view_shape(&node.view.tree, &mut shape);

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
        let per_node_texts = match compile_component_scenario_with_initials(
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
                let (_states0, texts0) = observe_component_states_and_texts(&compiled)?;
                let observed_leaves = texts0.first().map(|t| t.len()).unwrap_or(0);
                if observed_leaves != shape.text_leaf_count {
                    return Err(format!(
                        "text leaf mismatch: view tree has {}, AV observed {observed_leaves}",
                        shape.text_leaf_count
                    ));
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
                engine = Some(e);
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
            text_leaf_count: shape.text_leaf_count,
            load_texts: per_node_texts.first().cloned().unwrap_or_default(),
            last_states,
            field_names,
            dispatch_log: Vec::new(),
            errors: Vec::new(),
            script: Vec::new(),
            tick_no: 0,
        })
    }

    /// 事件引擎面（静态面源 → false——Hello 类无事件）。
    pub fn engine_face(&self) -> bool {
        self.engine.is_some()
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
        push_event(instance, target);
        self.pending_texts
            .push_back((instance, target, text.to_string()));
    }

    /// open：AV `ui_open` → session；就位显示模型并提交文本叶与状态初值
    /// （FFI 实效果——UI 创建/提交效果经 provider 车道承载）。
    pub fn open(&mut self) -> Result<i32, String> {
        install_ui980_lane()?;
        let session = av_call(&format!("ui_open(\"open\", {}, 0)", self.instances))?;
        if session < 1 {
            return Err(format!("ui_open refused: {session}"));
        }
        self.session = Some(session);
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
                            .map(|(n, v)| (n.clone(), state_display(v)))
                            .collect()
                    })
                    .collect();
                s.buttons = self
                    .buttons
                    .iter()
                    .map(|b| (b.label.clone(), b.msg.clone()))
                    .collect();
                s.inputs = self
                    .inputs
                    .iter()
                    .map(|i| (i.field.clone(), i.numeric))
                    .collect();
            }
        }
        // 文本叶（AV 真值）逐实例逐叶提交（view 即载荷，显式句柄）。
        for i in 0..self.instances {
            for leaf in 0..self.text_leaf_count {
                let text = self.load_texts.get(leaf).cloned().unwrap_or_default();
                let rc = av_call(&format!(
                    "ui_apply_text({}, {}, 0)",
                    escape(&text),
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
        Ok(session)
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
                format!("ui_apply_bool(\"apply-bool\", {packed}, {})", *v as i32)
            }
            AceStateValue::Str(v) => {
                format!("ui_apply_str({}, {packed}, 0)", escape(v))
            }
        };
        let rc = av_call(&body)?;
        if rc != ui980_status::OK {
            return Err(format!("apply `{body}` refused: {rc}"));
        }
        Ok(())
    }

    /// 受控事件步：注入到期脚本 → AV poll → AV 重放 dispatch → 差量 apply。
    pub fn step(&mut self) -> StepOutcome {
        self.tick_no += 1;
        self.run_due_script();
        let Some(session) = self.session else {
            return StepOutcome::Idle;
        };
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
            Plan::Unit {
                msg: button.msg.clone(),
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
        let msg_name = match &plan {
            Plan::Unit { msg } | Plan::Typed { msg, .. } => msg.clone(),
        };
        let Some(engine) = self.engine.as_mut() else {
            let msg = "static-face source has no event engine".to_string();
            self.errors.push(msg.clone());
            return StepOutcome::LaneError(msg);
        };
        let identity = self
            .identities
            .get(instance)
            .cloned()
            .unwrap_or_else(|| "i0".to_string());
        let dispatch_result = match plan {
            Plan::Unit { msg } => engine.dispatch(&identity, &msg),
            Plan::Typed { msg, args } => engine.dispatch_with_args(&identity, &msg, &args),
        };
        match dispatch_result {
            Ok(DispatchOutcome::Delivered(states)) => {
                let snapshot: Vec<(String, String)> = states
                    .iter()
                    .map(|(n, v)| (n.clone(), state_display(v)))
                    .collect();
                self.dispatch_log
                    .push((instance, msg_name.clone(), snapshot));
                StepOutcome::Dispatched {
                    instance,
                    msg: msg_name,
                    states: self.diff_and_apply(instance, &states),
                }
            }
            Ok(DispatchOutcome::Dropped(reason)) => {
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
            if changed {
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
        shown
    }

    /// 关闭当前 session（AV `ui_close`）；后续 apply/poll 具名拒、late 事件丢弃。
    pub fn close_session(&mut self) -> Result<(), String> {
        let Some(session) = self.session else {
            return Ok(());
        };
        let rc = av_call(&format!("ui_close(\"close\", {session}, 0)"))?;
        if rc != ui980_status::OK {
            return Err(format!("ui_close refused: {rc}"));
        }
        self.session = None;
        // B-AV U01：键入配对随 session 关闭一并丢弃（provider 队列已清，
        // 残留文本会成为无码孤儿）。
        self.pending_texts.clear();
        Ok(())
    }

    /// 重开：新 session（新世代），按当前业务状态重新提交显示模型。
    pub fn reopen(&mut self) -> Result<i32, String> {
        self.open()
    }

    /// 安装事件脚本（tick 从 1 起；CPU 测试与 e2e exe 同一边界）。
    pub fn install_script(&mut self, script: Vec<(usize, ScriptStep)>) {
        self.script = script;
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
                ScriptStep::Click { instance, target } => push_event(instance, target),
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

    /// 当前各实例业务状态（AV 真值；证据/断言消费）。
    pub fn states(&self, instance: usize) -> Option<Vec<(String, AceStateValue)>> {
        self.engine.as_ref()?.read_all(&self.identities[instance])
    }

    /// 脚本是否已全部注入（e2e 收口观察面之一）。
    pub fn script_done(&self) -> bool {
        self.script.iter().all(|(t, _)| *t <= self.tick_no)
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
                            "states": s.states,
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
        })
    }
}

/// 状态值显示串（宿主显示模型；业务值仍来自 AV）。
fn state_display(v: &AceStateValue) -> String {
    match v {
        AceStateValue::Int(i) => i.to_string(),
        AceStateValue::Bool(b) => b.to_string(),
        AceStateValue::Str(s) => s.clone(),
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
    BridgeTick,
}

/// MIN-A-UI 桌面窗口：显示模型渲染自 UI session（AV→FFI 提交的真实值），
/// 点击入 typed 队列，tick 步进桥（poll→AV 重放→apply——全部主线程受控点）。
#[derive(Debug, Default)]
pub struct AceNodeDesktopWindow;

impl Component for AceNodeDesktopWindow {
    type Msg = NodeDesktopMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            NodeDesktopMsg::Click { instance, target } => push_event(instance, target),
            NodeDesktopMsg::Type { instance, target } => {
                // 真实键入全文（iced on_input 暂存）→ 桥注入（push_event
                // + 配对文本，与脚本 TypeText 同一边界）。
                let text = crate::ui::iced::renderer::last_input_text();
                with_bridge(|b| b.inject_text(instance, target, &text));
            }
            NodeDesktopMsg::BridgeTick => {
                with_bridge(|b| b.step());
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
        let Some(session) = current_session(|s| {
            (
                s.texts.clone(),
                s.states.clone(),
                s.buttons.clone(),
                s.inputs.clone(),
                s.generation,
            )
        }) else {
            return View::col()
                .padding(30)
                .child(View::text("ace-node-desktop: no session"))
                .build();
        };
        let (texts, states, buttons, inputs, generation) = session;
        let mut root = View::col().spacing(10).padding(24);
        root = root.child(View::text_styled(
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
                col = col.child(View::text(format!("t{leaf}: {text}")));
            }
            for (_slot, (name, value)) in states.get(i).into_iter().flatten().enumerate() {
                col = col.child(View::text(format!("{name} = {value}")));
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
                        .on_change(NodeDesktopMsg::Type {
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
                    NodeDesktopMsg::Click {
                        instance: i,
                        target,
                    },
                    "px-3 py-1.5 rounded-lg bg-primary text-primary-foreground",
                ));
            }
            col = col.child(row.build());
            root = root.child(col.build());
        }
        root.build()
    }
}

/// MIN-A-UI 入口（`AUTO_UI_ENGINE=ace` 消费）：装载→open（FFI 实效果）→
/// run_app 真窗（tick 驱动受控事件循环）。窗口阻塞至关闭。
pub fn run_ace_node_desktop_ui(src: &str, path: &str, instances: usize) -> Result<(), String> {
    let mut bridge = NodeDesktopBridge::load(src, path, instances)?;
    bridge.open()?;
    install_bridge(bridge);
    crate::ui::iced::run_app::<AceNodeDesktopWindow>()
        .map_err(|e| format!("node desktop window: {e:?}"))
}
