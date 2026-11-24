//! PLAN-126 T-03：固定行高纵向虚拟列表（`VList` / Rust `VirtualList`）的运行时几何台账。
//!
//! 合同（plan §5.1，SD-01 待发布）：
//! - 行高固定；一次窗口构建使用同一已发布版本的长度、顺序与索引数据；
//! - 只对可见行 + 两侧 overscan 读取、绑定、构建、布局；屏外由占位表达，
//!   总逻辑高度参与滚动条计算；
//! - 未取得布局测量（首帧）或没有有界纵向视口时保留完整构建；
//! - `AUTOUI_VIRTUAL_LIST=0` 关闭窗口化，保留完整构建与既有交互。
//!
//! 数据流（T-03 实测接线，2026-11-24）：
//! ```text
//! convert_column(overflow-y-auto)  → push scope（`vlist_{seq}`）
//! ForLoop 臂                       → virtual_list::plan(scope, n) → (start,end) 或 None
//! View::Scrollable.measure_key     → build_scrollable 把该 scrollable 的实际 iced
//!                                    widget id（vnode_*）登记进台账，probe 据此认领
//!                                    offset/viewport/content（不新增 id、不改滚动目标）
//! View::VirtualMark(key,start,end) → dynamic 渲染轨挂 iced Id
//!                                    `vlist_mark:{key}:{start}:{end}` 并**递归**渲染行
//!                                    （保住 vnode id / bounds / 事件目标）
//! host 每帧 probe（窗口化后）       → VListProbe 收集 pane 几何 + 构建带 rect+行区间
//! __vlist_measure 消息             → note_measure → 解出 row_h / list_top
//! offset 变化                      → 置脏，下一帧按新偏移重建窗口行集合
//! ```
//!
//! 渲染入口实测：生产 VM 窗走 `dynamic_view_impl_inner`
//!（`iced/renderer/projection.rs`）→ `render_dynamic_view`
//!（`iced/renderer/devtools.rs`）。本组件此前在 `convert_view_messages`
//!（`iced/renderer/mod.rs`）的 `_ => Empty` 兜底处被整体吞掉——标记与占位
//! 必须在 View→View 类型转换里显式透传（同 PLAN-082 Rich / PLAN-029 Custom
//! 定罪犯例）。
//!
//! 台账是进程级、按 scope key 持久的；视图重建不丢测量（同 `scroll::managed`
//! 先例）。同一 pane 只支持一个虚拟化循环（首个认领 scope），其余循环保留完整
//! 构建——多列表同 pane 的窗口化不在首期范围。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use iced::advanced::widget::operation::{Outcome, Scrollable as OpScrollable};
use iced::advanced::widget::Operation;
use iced::widget::Id;
use iced::{Rectangle, Vector};

/// pane iced id 前缀（probe 只认此前缀）。
pub const PANE_ID_PREFIX: &str = "vlist_pane:";
/// 构建带 mark iced id 前缀（probe 只认此前缀）。
pub const MARK_ID_PREFIX: &str = "vlist_mark:";
/// probe 结果消息名（host update 拦截，不派发 VM）。
pub const MEASURE_EVENT: &str = "__vlist_measure";

/// 两侧各保留的预渲染行数（plan §5.1.2「两侧各最多 2 行」）。
pub const OVERSCAN_ROWS: usize = 2;

/// 无几何时的标定帧最多构建的行数（构建侧与测量侧共用：标定带
/// （start=0、行数 = min(n, CALIBRATION_ROWS)）是唯一可信的「全局平均
/// 行高」样本）。
pub const CALIBRATION_ROWS: usize = 256;

/// 标定采样节流：相邻两次标定测量至少间隔该时长。首秒内世界/字型尚未
/// 稳定（025 实测全表高 3456 → 3477，差 0.6%），连续帧采样会锁死早期值。
pub const CALIB_SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);

/// 已确认行高的复核周期：周期性做一次完整构建标定，纠偏运行期度量变化。
pub const CALIB_VERIFY_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3);

thread_local! {
    /// 测试面开关覆盖（线程本地：libtest 每测试一线程、nextest 同进程单测试；
    /// 不污染其他测试线程）。`None` = 走环境变量。
    #[cfg(test)]
    static ENABLED_OVERRIDE: Cell<Option<bool>> = const { Cell::new(None) };
}

/// 测试面：强制本线程的窗口化开关（`None` = 恢复环境变量语义）。
#[cfg(test)]
pub fn set_enabled_override_for_test(v: Option<bool>) {
    ENABLED_OVERRIDE.with(|c| c.set(v));
}

/// 环境开关：缺省启用；`0` = 关闭窗口化（完整构建对照）。
pub fn enabled() -> bool {
    #[cfg(test)]
    if let Some(v) = ENABLED_OVERRIDE.with(|c| c.get()) {
        return v;
    }
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| match std::env::var("AUTOUI_VIRTUAL_LIST").as_deref() {
        Ok("0") => false,
        _ => true,
    })
}

/// 诊断开关：`AUTOUI_VLIST_DIAG=1` 时每次窗口构建打一行 `[VLIST]`。
pub fn diag() -> bool {
    static DIAG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *DIAG.get_or_init(|| std::env::var("AUTOUI_VLIST_DIAG").as_deref() == Ok("1"))
}

fn f64_from_var(name: &str) -> Option<f64> {
    std::env::var(name).ok().and_then(|v| v.parse::<f64>().ok())
}

// ---------------------------------------------------------------------------
// 纯窗口计算（测试靶；不持锁、不做 IO）
// ---------------------------------------------------------------------------

/// 由 pane 几何 + 列表几何算出本次构建的行区间（半开 `[start,end)`，已夹紧）。
///
/// - `offset`：pane 的纵向滚动偏移（content 空间 px）；
/// - `viewport_h`：有界纵向视口高；
/// - `list_top`：列表首行在 content 空间的 y；
/// - `row_h`：固定行高（> 0）；
/// - `n`：列表长度；
/// - `overscan`：两侧各扩展行数。
///
/// 列表本体完全在视口之外时返回空区间（`start == end`），但仍保留总高度。
pub fn visible_range(
    offset: f64,
    viewport_h: f64,
    list_top: f64,
    row_h: f64,
    n: usize,
    overscan: usize,
) -> (usize, usize) {
    if n == 0 || row_h <= 0.0 || viewport_h <= 0.0 || !offset.is_finite() || !list_top.is_finite() {
        return (0, 0);
    }
    let band_top = (offset - list_top) / row_h;
    let band_bottom = (offset + viewport_h - list_top) / row_h;
    // 与行区间相交的行：行 i 覆盖 [i, i+1)。上界用 ceil，下界用 floor。
    let first = band_top.floor();
    let last = band_bottom.ceil();
    let mut start = if first.is_finite() { first } else { 0.0 };
    let mut end = if last.is_finite() { last } else { 0.0 };
    // 完全在视口上方/下方时按方向钳成空区间。
    if end <= 0.0 || start >= n as f64 {
        return (clamp_usize(start, n), clamp_usize(start, n));
    }
    start -= overscan as f64;
    end += overscan as f64;
    (clamp_usize(start, n), clamp_usize(end, n))
}

fn clamp_usize(v: f64, n: usize) -> usize {
    if !v.is_finite() || v <= 0.0 {
        0
    } else if v >= n as f64 {
        n
    } else {
        v as usize
    }
}

// ---------------------------------------------------------------------------
// pane 作用域（线程本地：子 widget 的 builder 在同一构建栈内共享）
// ---------------------------------------------------------------------------

use std::cell::{Cell, RefCell};

thread_local! {
    /// 当前构建栈上的 pane 作用域 key 栈（最内层在末尾）。
    static SCOPE_STACK: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// 本帧 pane 序号（文档序稳定 → 跨帧同名；根作用域清空时归零）。
    static SCOPE_SEQ: Cell<u32> = const { Cell::new(0) };
}

/// 压入一个 pane 作用域，返回本帧稳定 key（`vlist_{seq}`）。
pub fn push_scope() -> String {
    let seq = SCOPE_SEQ.with(|s| {
        let n = s.get();
        s.set(n + 1);
        n
    });
    let key = format!("vlist_{seq}");
    SCOPE_STACK.with(|st| st.borrow_mut().push(key.clone()));
    key
}

/// 弹出当前 pane 作用域；根作用域清空后本帧序号归零。
pub fn pop_scope() {
    let emptied = SCOPE_STACK.with(|st| {
        let mut b = st.borrow_mut();
        b.pop();
        b.is_empty()
    });
    if emptied {
        SCOPE_SEQ.with(|s| s.set(0));
    }
}

/// 当前最内层 pane 作用域 key。
pub fn current_scope() -> Option<String> {
    SCOPE_STACK.with(|st| st.borrow().last().cloned())
}

// ---------------------------------------------------------------------------
// 台账
// ---------------------------------------------------------------------------

/// 一个虚拟化 pane 的几何测量（由 probe 消息写入）。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PaneGeometry {
    /// 纵向滚动偏移（px）。
    pub offset_y: f64,
    /// 有界纵向视口高（px）。
    pub viewport_h: f64,
    /// 内容总高（px）。
    pub content_h: f64,
    /// 视口顶在窗口坐标系中的 y（px）——构建带 rect 是窗口绝对坐标，
    /// 反解列表起点时须扣除视口原点。
    pub viewport_y: f64,
    /// 视口宽（px）——宽度变化会改变行内换行/字形度量，进而改变行高，
    /// 台账据此作废旧行高重新标定。
    pub viewport_w: f64,
}

/// 构建带（窗口行 Column）的布局矩形与当次区间，用于反解行高与列表起点。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BandGeometry {
    /// 构建带 rect（content 空间 px）。
    pub y: f64,
    pub h: f64,
    /// 该 rect 对应的行区间（构建时记录）。
    pub start: usize,
    pub end: usize,
}

/// 一次窗口构建的决策记录（诊断 + 测试观测面）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPlan {
    pub total: usize,
    pub start: usize,
    pub end: usize,
    pub row_h: f64,
    pub list_top: f64,
    pub offset_y: f64,
    pub viewport_h: f64,
}

impl WindowPlan {
    /// 本次构建的行数。
    pub fn rows(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

#[derive(Debug, Default, Clone)]
struct ScopeEntry {    pane: Option<PaneGeometry>,
    band: Option<BandGeometry>,
    /// 已解出的固定行高与列表起点（内容空间）。
    row_h: f64,
    list_top: f64,
    /// 最近一次窗口构建（诊断/测试）。
    last_plan: Option<WindowPlan>,
    /// 最近一次测量的序号（观测新鲜度）。
    measured_seq: u64,
    /// 本 scope 是否已被某个 `for` 循环认领（`note_plan` 置位）——只有
    /// 认领过的 pane 才需要登记 iced widget id / 参与 probe。
    claimed: bool,
    /// pane 滚动容器的 iced widget id（渲染器在 build_scrollable 登记）。
    /// probe 用它在 `Operation::scrollable` 里认领视口几何。
    pane_widget_id: Option<String>,
    /// 本 scope 认领时的列表长度（判定标定带用）。
    total: usize,
    /// 标定行高候选（渲染期首帧/字型加载前的测量可能偏 0.6%，需间隔采样
    /// 两次一致才采信——025 实测首秒内 3456、稳定后 3477）。
    calib_candidate: Option<f64>,
    /// 行高是否已被两次间隔采样一致的标定确认（未确认时继续标定构建）。
    row_h_trusted: bool,
    /// 最近一次标定采样时刻（节流：相邻样本至少相隔 [`CALIB_SAMPLE_INTERVAL`]）。
    last_calib_at: Option<std::time::Instant>,
    /// 已确认行高后的下一次复核时刻（周期性完整构建复核，纠偏字型/DPI 等
    /// 运行期变化；复核带只更新行高，不改变窗口语义）。
    next_verify_at: Option<std::time::Instant>,
    /// 显式固定行高（`scroll (row-height: N)` prop）。声明后该 pane **不再**
    /// 标定/周期复核：行高由创作侧给定，视口/偏移仍由 probe 提供。
    explicit_row_h: Option<f64>,
}

lazy_static::lazy_static! {
    static ref SCOPES: Mutex<HashMap<String, ScopeEntry>> = Mutex::new(HashMap::new());
}

/// 有 pane 需要本次 probe（构建期置位，host 消费后清零）。
static MEASURE_WANTED: AtomicBool = AtomicBool::new(false);
/// 本进程已有窗口化 pane（host 每帧轻量 probe，用于滚动后即时重建窗口）。
static ACTIVE: AtomicBool = AtomicBool::new(false);
/// 窗口构建计数（诊断/测试）。
static ROWS_BUILT: AtomicU64 = AtomicU64::new(0);
static WINDOWS_BUILT: AtomicU64 = AtomicU64::new(0);
/// V-01 取证：`for` 循环实际读取的列表行数（窗口路径按 `list_get` 逐行计，
/// 完整构建路径按整表物化计）与 `list_get` 调用数。
static ROWS_READ: AtomicU64 = AtomicU64::new(0);
static LIST_GETS: AtomicU64 = AtomicU64::new(0);
static MEASURE_SEQ: AtomicU64 = AtomicU64::new(1);

/// 构建期登记：本帧有虚拟化 pane（host 下一拍 probe）。
pub fn request_measure() {
    MEASURE_WANTED.store(true, Ordering::Relaxed);
    ACTIVE.store(true, Ordering::Relaxed);
}

/// 本进程是否已进入窗口化（host 每帧 probe 以感知滚动位置变化）。
pub fn is_active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// host 消费：是否需要 probe，并清零请求位。
pub fn take_measure_request() -> bool {
    MEASURE_WANTED.swap(false, Ordering::Relaxed)
}

/// 重置请求位（测试）。
pub fn clear_measure_request() {
    MEASURE_WANTED.store(false, Ordering::Relaxed);
}

fn with_scope<R>(key: &str, f: impl FnOnce(&mut ScopeEntry) -> R) -> R {
    let mut map = SCOPES.lock().unwrap();
    let e = map.entry(key.to_string()).or_default();
    f(e)
}

fn read_scope<R>(key: &str, f: impl FnOnce(&ScopeEntry) -> R) -> Option<R> {
    let map = SCOPES.lock().unwrap();
    map.get(key).map(f)
}

/// 写入 pane 几何（probe 消息）。返回是否首次写入或发生变化。
pub fn note_pane(key: &str, pane: PaneGeometry) -> bool {
    let changed = with_scope(key, |e| {
        let changed = e.pane != Some(pane);
        // 视口宽度变化 → 行内换行/字形度量可能改变行高：作废已解出的行高，
        // 让下一帧重新标定（一次完整构建，随后继续窗口化）。显式行高是
        // 创作侧固定合同，不因宽度变化作废。
        if let Some(prev) = e.pane {
            if e.explicit_row_h.is_none() && (prev.viewport_w - pane.viewport_w).abs() > 1.0 {
                e.row_h = 0.0;
                e.row_h_trusted = false;
                e.calib_candidate = None;
                e.last_calib_at = None;
                e.next_verify_at = None;
            }
        }
        e.pane = Some(pane);
        changed
    });
    changed
}

/// 渲染期登记：pane 滚动容器的 iced widget id（仅已认领 scope 有效）。
/// `key` = pane 作用域 key；`widget_id` = 该 scrollable 实际挂的 iced id
/// （vnode_* / blocklist_scroll / scroll_ctl_*）。probe 遍历时按此认领。
pub fn note_pane_widget_id(key: &str, widget_id: &str) {
    with_scope(key, |e| {
        if e.claimed {
            e.pane_widget_id = Some(widget_id.to_string());
        }
    });
}

/// probe 期反查：某 iced widget id 是否是已认领的虚拟列表 pane。
pub fn pane_key_for_widget_id(widget_id: &str) -> Option<String> {
    let map = SCOPES.lock().unwrap();
    map.iter().find_map(|(key, e)| {
        if e.claimed && e.pane_widget_id.as_deref() == Some(widget_id) {
            Some(key.clone())
        } else {
            None
        }
    })
}

/// 写入构建带矩形（probe 消息），并按**该 rect 自带的行区间**反解行高/
/// 列表起点。
///
/// `start..end` 来自构建带 iced id（`vlist_mark:{key}:{start}:{end}`）——
/// 不走台账「最近一次构建」，因为测量回执到达时台账可能已被下一帧构建
/// 改写（实测错配会把 h 除以错误行数 → 行高自激振荡、窗口塌缩）。
/// `y` 是 iced 布局给出的**窗口绝对坐标**（scrollable 内容在布局中不被
/// 滚动偏移平移，滚动只在绘制/命中阶段换算），`h` 是构建带自身高。列表
/// 起点（内容空间）＝ 带顶 - 已构建行数×行高 - 视口顶。
///
/// 行高**只从标定带采信**（start=0 且行数=n，或 n 超过标定上限时取顶部
/// 标定段）：窗口带只是 n 的一个连续片段，其平均行高受行内容影响可偏
/// 0.5%（025 实测：全局 28.975 / 尾部窗口 28.82），若反复用窗口带覆盖
/// 行高，内容总高会随滚动位置漂移。窗口带只用于重算 `list_top`。
pub fn note_band(key: &str, start: usize, end: usize, y: f64, h: f64) -> bool {
    with_scope(key, |e| {
        e.claimed = true;
        e.band = Some(BandGeometry {
            y,
            h,
            start,
            end,
        });
        let rows = end.saturating_sub(start);
        let total = e.total;
        // 显式 `row-height` pane：构建带只用于精修 `list_top`，不得用实测
        // 平均行高覆盖创作侧固定行高（行内容换行会让窗口带均值偏 0.5%）。
        if rows > 0 && h.is_finite() && h > 0.0 && e.explicit_row_h.is_none() {
            let cand = h / rows as f64;
            if cand >= 1.0 && cand <= 400.0 {
                let is_calibration = if total > 0 {
                    start == 0 && (rows == total || (total > CALIBRATION_ROWS && rows == CALIBRATION_ROWS))
                } else {
                    false
                };
                if is_calibration {
                    let now = std::time::Instant::now();
                    let due = match e.last_calib_at {
                        None => true,
                        Some(t) => now.duration_since(t) >= CALIB_SAMPLE_INTERVAL,
                    };
                    if due {
                        e.last_calib_at = Some(now);
                        if !e.row_h_trusted {
                            // 两次**间隔采样**一致（≤0.5%）才采信。
                            let agree = e.calib_candidate.map_or(false, |prev| {
                                (prev - cand).abs() / prev.max(1.0) <= 0.005
                            });
                            e.row_h = cand;
                            if agree {
                                e.row_h_trusted = true;
                                e.next_verify_at = Some(now + CALIB_VERIFY_INTERVAL);
                            } else {
                                e.calib_candidate = Some(cand);
                            }
                        } else {
                            // 已确认行高的周期性复核：显著偏差才纠偏。
                            e.next_verify_at = Some(now + CALIB_VERIFY_INTERVAL);
                            if (e.row_h - cand).abs() / e.row_h.max(1.0) > 0.005 {
                                e.row_h = cand;
                            }
                        }
                    } else if e.row_h <= 0.0 {
                        e.row_h = cand;
                    }
                } else if e.row_h <= 0.0 {
                    // 兜底：会话中途首次测量（无标定带可用时）——可用但未确认。
                    e.row_h = cand;
                }
            }
        }
        if e.row_h > 0.0 {
            let viewport_y = e.pane.map(|p| p.viewport_y).unwrap_or(0.0);
            e.list_top = y - start as f64 * e.row_h - viewport_y;
        }
        e.measured_seq = MEASURE_SEQ.fetch_add(1, Ordering::Relaxed);
        true
    })
}

/// 兼容旧签名（测试）：用台账最近一次构建的区间反解。
#[cfg(test)]
pub fn note_band_legacy(key: &str, y: f64, h: f64) -> bool {
    let range = read_scope(key, |e| e.band).flatten().map(|b| (b.start, b.end));
    match range {
        Some((start, end)) => note_band(key, start, end, y, h),
        None => false,
    }
}



/// 构建期记录本次窗口计划（同时登记 band 的 start/end 供 probe 反解）。
pub fn note_plan(key: &str, plan: WindowPlan) {
    with_scope(key, |e| {
        e.claimed = true;
        e.total = plan.total;
        // 标定构建（start=0 且行数 = n 或顶部标定段）——若行高已确认，
        // 这是一次周期复核：立刻把下次复核时间推后，避免复核测量回执到达
        // 前的连续帧重复做完整构建。
        let rows = plan.rows();
        let calibrating = plan.start == 0
            && (rows == plan.total
                || (plan.total > CALIBRATION_ROWS && rows == CALIBRATION_ROWS));
        if calibrating && e.row_h_trusted && e.explicit_row_h.is_none() {
            e.next_verify_at = Some(std::time::Instant::now() + CALIB_VERIFY_INTERVAL);
        }
        e.last_plan = Some(plan);
        e.band = Some(BandGeometry {
            y: e.band.map(|b| b.y).unwrap_or(0.0),
            h: e.band.map(|b| b.h).unwrap_or(0.0),
            start: plan.start,
            end: plan.end,
        });
    });
}

/// 可见行 + 两端占位所需的构建决策。几何未测得（或行高尚未被两次间隔
/// 采样一致的标定确认、或到了周期复核时刻）时返回 `None`（构建侧改做
/// 标定构建）。
pub fn plan(key: &str, total: usize) -> Option<WindowPlan> {
    let entry = read_scope(key, |e| e.clone())?;
    let pane = entry.pane?;
    // PLAN-126 T-03 续：显式 `row-height` 视为已确认行高——不要求标定样本，
    // 也不参与周期复核（创作侧声明的是固定行高合同）。
    let explicit = entry.explicit_row_h.filter(|h| *h > 0.0);
    let row_h = explicit.unwrap_or(entry.row_h);
    let trusted = explicit.is_some() || entry.row_h_trusted;
    let verify_due = explicit.is_none()
        && entry
            .next_verify_at
            .map_or(false, |t| std::time::Instant::now() >= t);
    if row_h <= 0.0 || !trusted || verify_due || pane.viewport_h <= 0.0 || total == 0 {
        return None;
    }
    let (start, end) = visible_range(
        pane.offset_y,
        pane.viewport_h,
        entry.list_top,
        row_h,
        total,
        OVERSCAN_ROWS,
    );
    Some(WindowPlan {
        total,
        start,
        end,
        row_h,
        list_top: entry.list_top,
        offset_y: pane.offset_y,
        viewport_h: pane.viewport_h,
    })
}

/// 声明某 pane 的显式固定行高（`scroll (row-height: N)` prop）。
///
/// 与标定路径的区别：行高直接采信（`row_h_trusted`），标定候选/复核计时
/// 清空；`list_top` 可随 prop 一并给出（内容空间，缺省沿用台账当前值 0），
/// 首次构建带测量到达后仍会被实测值精修。非法值（非有限/≤0/>400）忽略。
pub fn note_explicit_row_height(key: &str, row_h: f64, list_top: Option<f64>) {
    if !row_h.is_finite() || !(1.0..=400.0).contains(&row_h) {
        return;
    }
    with_scope(key, |e| {
        e.explicit_row_h = Some(row_h);
        e.row_h = row_h;
        e.row_h_trusted = true;
        e.calib_candidate = Some(row_h);
        e.last_calib_at = None;
        e.next_verify_at = None;
        if let Some(t) = list_top {
            if t.is_finite() {
                e.list_top = t;
            }
        }
    });
}

/// 本 scope 的显式行高（测试/诊断）。
pub fn explicit_row_height(key: &str) -> Option<f64> {
    read_scope(key, |e| e.explicit_row_h).flatten()
}

/// 显式行高 pane 在**尚无视口几何**时的有界顶窗构建计划。
///
/// 首帧 probe 之前没有 viewport/offset，无法算可见窗口；但行高已知，可以
/// 先构建顶部至多 [`CALIBRATION_ROWS`] 行并用尾部占位把总逻辑高度一次补齐
/// （滚动条范围从首帧起就正确，不会在标定后跳变）。视口几何一到就转为
/// [`plan`] 的真实窗口。非显式 pane 返回 `None`（仍走标定构建）。
pub fn explicit_plan(key: &str, total: usize) -> Option<WindowPlan> {
    let entry = read_scope(key, |e| e.clone())?;
    let row_h = entry.explicit_row_h.filter(|h| *h > 0.0)?;
    if total == 0 {
        return None;
    }
    Some(WindowPlan {
        total,
        start: 0,
        end: total.min(CALIBRATION_ROWS),
        row_h,
        list_top: entry.list_top,
        offset_y: 0.0,
        viewport_h: 0.0,
    })
}

/// 最近一次窗口计划（测试/诊断）。
pub fn last_plan(key: &str) -> Option<WindowPlan> {
    read_scope(key, |e| e.last_plan).flatten()
}

/// 台账是否已有该 scope 的行高（测试）。
pub fn has_row_height(key: &str) -> bool {
    read_scope(key, |e| e.row_h > 0.0).unwrap_or(false)
}

/// 计数：本次进程内窗口构建的行数/次数（测试与 V-01 取证）。
pub fn counters() -> (u64, u64) {
    (
        ROWS_BUILT.load(Ordering::Relaxed),
        WINDOWS_BUILT.load(Ordering::Relaxed),
    )
}

/// V-01 取证计数快照：列表行读取 / `list_get` 调用 / 窗口构建行数 / 窗口数。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildCounters {
    /// `for` 循环读取的列表行数（窗口路径 = 逐行 `list_get`；完整构建路径 =
    /// 整表物化行数）。
    pub rows_read: u64,
    /// `list_get` 调用次数（仅窗口路径 > 0；完整构建路径走整表物化）。
    pub list_gets: u64,
    /// 窗口构建实际构建的行数合计。
    pub rows_built: u64,
    /// 窗口构建次数。
    pub windows_built: u64,
}

/// 读取 V-01 取证计数快照（不重置）。
pub fn build_counters() -> BuildCounters {
    BuildCounters {
        rows_read: ROWS_READ.load(Ordering::Relaxed),
        list_gets: LIST_GETS.load(Ordering::Relaxed),
        rows_built: ROWS_BUILT.load(Ordering::Relaxed),
        windows_built: WINDOWS_BUILT.load(Ordering::Relaxed),
    }
}

/// 窗口路径逐行读取计数（`try_virtual_for` 每调用一次 `list_get` 记一次）。
pub fn record_list_get() {
    LIST_GETS.fetch_add(1, Ordering::Relaxed);
    ROWS_READ.fetch_add(1, Ordering::Relaxed);
}

/// 完整构建路径整表物化计数（`for` 循环取到整表后按行数记一次）。
pub fn record_rows_materialized(rows: usize) {
    ROWS_READ.fetch_add(rows as u64, Ordering::Relaxed);
}

/// 测试面：清空台账与计数。
pub fn reset_for_test() {
    SCOPES.lock().unwrap().clear();
    ROWS_BUILT.store(0, Ordering::Relaxed);
    WINDOWS_BUILT.store(0, Ordering::Relaxed);
    ROWS_READ.store(0, Ordering::Relaxed);
    LIST_GETS.store(0, Ordering::Relaxed);
    MEASURE_WANTED.store(false, Ordering::Relaxed);
    ACTIVE.store(false, Ordering::Relaxed);
}

/// 测试面：把本 scope 的下一次标定采样置为「已到期」（跳过 500ms 节流）。
#[cfg(test)]
pub fn force_calib_sample_due_for_test(key: &str) {
    with_scope(key, |e| {
        e.last_calib_at = e
            .last_calib_at
            .and_then(|t| t.checked_sub(CALIB_SAMPLE_INTERVAL));
    });
}

/// 测试面：把本 scope 的周期复核置为「已到期」。
#[cfg(test)]
pub fn force_verify_due_for_test(key: &str) {
    with_scope(key, |e| e.next_verify_at = Some(std::time::Instant::now()));
}

/// 构建期计数 + 诊断行。
pub fn record_build(key: &str, plan: &WindowPlan) {
    ROWS_BUILT.fetch_add(plan.rows() as u64, Ordering::Relaxed);
    WINDOWS_BUILT.fetch_add(1, Ordering::Relaxed);
    if diag() {
        eprintln!(
            "[VLIST] scope={key} n={} window=[{}, {}) rows={} row_h={:.2} list_top={:.2} offset={:.2} viewport_h={:.2}",
            plan.total,
            plan.start,
            plan.end,
            plan.rows(),
            plan.row_h,
            plan.list_top,
            plan.offset_y,
            plan.viewport_h,
        );
    }
}

/// 测试/诊断：直接注入几何（不依赖 iced 布局）。视口原点/宽按 0 注入。
pub fn seed_geometry_for_test(key: &str, offset_y: f64, viewport_h: f64, row_h: f64, list_top: f64) {
    seed_geometry_full_for_test(key, offset_y, viewport_h, row_h, list_top, 0.0, 0.0);
}

/// 测试/诊断：含视口原点与宽度的完整几何注入。
pub fn seed_geometry_full_for_test(
    key: &str,
    offset_y: f64,
    viewport_h: f64,
    row_h: f64,
    list_top: f64,
    viewport_y: f64,
    viewport_w: f64,
) {
    with_scope(key, |e| {
        e.claimed = true;
        e.pane = Some(PaneGeometry {
            offset_y,
            viewport_h,
            content_h: 0.0,
            viewport_y,
            viewport_w,
        });
        e.row_h = row_h;
        e.row_h_trusted = row_h > 0.0;
        e.calib_candidate = (row_h > 0.0).then_some(row_h);
        e.last_calib_at = None;
        e.next_verify_at = None;
        e.list_top = list_top;
    });
}

// ---------------------------------------------------------------------------
// probe：一次遍历同时收 pane 几何与构建带 rect
// ---------------------------------------------------------------------------

/// probe 结果（host 以 JSON 传入 `__vlist_measure`，或直接调用 `note_measure`）。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VListMeasure {
    /// scope key → pane 几何。
    pub panes: HashMap<String, PaneGeometryJson>,
    /// scope key → 构建带 rect（含该 rect 对应的行区间）。
    pub bands: HashMap<String, BandRectJson>,
}

/// 构建带测量回执：rect + 行区间（区间来自 mark id，测量与构建天然配对）。
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct BandRectJson {
    pub y: f64,
    pub h: f64,
    pub start: usize,
    pub end: usize,
}

/// 可序列化的 pane 几何。
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct PaneGeometryJson {
    pub offset_y: f64,
    pub viewport_h: f64,
    pub content_h: f64,
    #[serde(default)]
    pub viewport_y: f64,
    #[serde(default)]
    pub viewport_w: f64,
}

/// `Operation`：遍历 iced 树，按 id 前缀收 pane（scrollable）与 mark（container）。
pub struct VListProbe {
    out: VListMeasure,
}

impl VListProbe {
    pub fn new() -> Self {
        Self {
            out: VListMeasure::default(),
        }
    }

    fn key_of(id: &Id, prefix: &str) -> Option<String> {
        let debug = format!("{id:?}");
        let start = debug.find(prefix)?;
        let rest = &debug[start + prefix.len()..];
        let end = rest.find('"').unwrap_or(rest.len());
        let key = rest[..end].trim();
        if key.is_empty() {
            None
        } else {
            Some(key.to_string())
        }
    }

    /// `Id` 的原始字符串（Debug 形如 `Id("vnode_123")`）。
    fn id_string(id: &Id) -> String {
        let debug = format!("{id:?}");
        let inner = debug
            .find('"')
            .map(|s| &debug[s + 1..])
            .unwrap_or(debug.as_str());
        let inner = inner.rsplit_once('"').map(|(head, _)| head).unwrap_or(inner);
        inner.to_string()
    }

    /// 构建带 mark id → `(scope key, start, end)`。
    fn parse_mark(id: &Id) -> Option<(String, usize, usize)> {
        let rest = Self::key_of(id, MARK_ID_PREFIX)?;
        let mut parts = rest.split(':');
        let key = parts.next()?.to_string();
        let start = parts.next()?.trim().parse::<usize>().ok()?;
        let end = parts.next()?.trim().parse::<usize>().ok()?;
        Some((key, start, end))
    }
}

impl Default for VListProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl Operation<VListMeasure> for VListProbe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<VListMeasure>)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if let Some(id) = id {
            if diag() {
                let dbg = format!("{id:?}");
                if dbg.contains("vlist") || dbg.contains("aura_0") {
                    eprintln!(
                        "[VLIST-PROBE] container id={dbg} rect=({:.0},{:.0},{:.0}x{:.0})",
                        bounds.x, bounds.y, bounds.width, bounds.height
                    );
                }
            }
            if let Some((key, start, end)) = Self::parse_mark(id) {
                self.out.bands.insert(
                    key,
                    BandRectJson {
                        y: bounds.y as f64,
                        h: bounds.height as f64,
                        start,
                        end,
                    },
                );
            }
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn OpScrollable,
    ) {
        if let Some(id) = id {
            // 生产路径：渲染器登记过该 scrollable 的 iced id（窗口化 pane）。
            let by_widget = pane_key_for_widget_id(&Self::id_string(id));
            let key = by_widget.or_else(|| Self::key_of(id, PANE_ID_PREFIX));
            if let Some(key) = key {
                self.out.panes.insert(
                    key,
                    PaneGeometryJson {
                        offset_y: (translation.y.max(0.0)) as f64,
                        viewport_h: bounds.height.max(0.0) as f64,
                        content_h: content_bounds.height.max(0.0) as f64,
                        viewport_y: bounds.y as f64,
                        viewport_w: bounds.width.max(0.0) as f64,
                    },
                );
            }
        }
    }

    fn finish(&self) -> Outcome<VListMeasure> {
        Outcome::Some(self.out.clone())
    }
}

/// 本帧 probe 是否观测到 pane 滚动偏移变化（用于把「滚动」变成一次视图
/// 重建：窗口化后行集合随偏移变化，*不重建*就会沿用旧窗口的行集合，滚动
/// 后屏内可能整段空白——实测定罪）。
pub fn offset_moved(m: &VListMeasure) -> bool {
    m.panes.iter().any(|(key, p)| {
        read_scope(key, |e| e.pane)
            .flatten()
            .map_or(false, |prev| (prev.offset_y - p.offset_y).abs() > 0.5)
    })
}

/// host 收到 probe 结果：落台账。返回是否有 scope 变化。
pub fn note_measure(m: &VListMeasure) -> bool {
    let mut changed = false;
    for (key, p) in &m.panes {
        changed |= note_pane(
            key,
            PaneGeometry {
                offset_y: p.offset_y,
                viewport_h: p.viewport_h,
                content_h: p.content_h,
                viewport_y: p.viewport_y,
                viewport_w: p.viewport_w,
            },
        );
    }
    for (key, band) in &m.bands {
        changed |= note_band(key, band.start, band.end, band.y, band.h);
    }
    changed
}

/// 诊断覆盖：`AUTOUI_VLIST_ROW_H` / `AUTOUI_VLIST_LIST_TOP` 可强制几何
/// （仅用于无 iced 布局的纯构建测试与故障定位；生产缺省不用）。
pub fn env_geometry_override() -> Option<(f64, f64)> {
    match (f64_from_var("AUTOUI_VLIST_ROW_H"), f64_from_var("AUTOUI_VLIST_LIST_TOP")) {
        (Some(r), Some(t)) if r > 0.0 => Some((r, t)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_range_top_of_list() {
        // 视口 740、行高 29、列表顶 177：offset=0 → 行 0..ceil((740-177)/29)=20 +2 overscan
        let (s, e) = visible_range(0.0, 740.0, 177.0, 29.0, 120, OVERSCAN_ROWS);
        assert_eq!(s, 0);
        // (740-177)/29 = 19.41 → ceil 20，+2 overscan = 22
        assert_eq!(e, 22);
    }

    #[test]
    fn visible_range_middle() {
        let (s, e) = visible_range(1450.0, 740.0, 177.0, 29.0, 120, OVERSCAN_ROWS);
        // (1450-177)/29 = 43.9 → floor 43 -2 = 41；(1450+740-177)/29=69.4 → ceil 70 +2 = 72
        assert_eq!((s, e), (41, 72));
    }

    #[test]
    fn visible_range_tail_clamps() {
        let (s, e) = visible_range(2929.0, 740.0, 177.0, 29.0, 120, OVERSCAN_ROWS);
        assert_eq!(e, 120);
        assert!(s < 120 && s > 90, "start={s}");
    }

    #[test]
    fn visible_range_outside_above_and_below() {
        // 列表整体在视口下方
        let (s, e) = visible_range(0.0, 300.0, 400.0, 29.0, 50, OVERSCAN_ROWS);
        assert_eq!((s, e), (0, 0));
        // 列表整体在视口上方
        let (s, e) = visible_range(5000.0, 300.0, 400.0, 29.0, 50, OVERSCAN_ROWS);
        assert_eq!(s, e);
    }

    #[test]
    fn visible_range_degenerate_inputs() {
        assert_eq!(visible_range(0.0, 0.0, 0.0, 29.0, 10, 2), (0, 0));
        assert_eq!(visible_range(0.0, 100.0, 0.0, 0.0, 10, 2), (0, 0));
        assert_eq!(visible_range(0.0, 100.0, 0.0, 29.0, 0, 2), (0, 0));
    }

    #[test]
    fn plan_requires_geometry_then_windows() {
        reset_for_test();
        assert!(plan("s1", 120).is_none());
        seed_geometry_for_test("s1", 0.0, 740.0, 29.0, 177.0);
        let p = plan("s1", 120).expect("plan");
        assert_eq!((p.start, p.end), (0, 22));
        assert_eq!(p.rows(), 22);
    }

    #[test]
    fn band_solves_row_height_and_list_top() {
        reset_for_test();
        seed_geometry_for_test("s2", 0.0, 740.0, 0.0, 0.0);
        note_plan(
            "s2",
            WindowPlan {
                total: 120,
                start: 10,
                end: 40,
                row_h: 0.0,
                list_top: 0.0,
                offset_y: 0.0,
                viewport_h: 740.0,
            },
        );
        // 构建带 30 行、高 870、起点 y = list_top + 10*29 = 177 + 290 = 467
        assert!(note_band_legacy("s2", 467.0, 870.0));
        let e = read_scope("s2", |e| (e.row_h, e.list_top)).unwrap();
        assert!((e.0 - 29.0).abs() < 1e-6, "row_h={}", e.0);
        assert!((e.1 - 177.0).abs() < 1e-6, "list_top={}", e.1);
    }

    #[test]
    fn band_rejects_implausible_row_height() {
        reset_for_test();
        note_plan(
            "s3",
            WindowPlan {
                total: 10,
                start: 0,
                end: 5,
                row_h: 0.0,
                list_top: 0.0,
                offset_y: 0.0,
                viewport_h: 100.0,
            },
        );
        // 5 行却有 1000px → 单行 200px 属可疑但仍在 1..=400 内；改测 5000/5=1000
        note_band_legacy("s3", 0.0, 5000.0);
        assert!(!has_row_height("s3"));
    }

    #[test]
    fn measure_roundtrip_requests_and_notes() {
        reset_for_test();
        assert!(!take_measure_request());
        request_measure();
        assert!(take_measure_request());
        assert!(!take_measure_request());
        let mut m = VListMeasure::default();
        m.panes.insert(
            "p".into(),
            PaneGeometryJson {
                offset_y: 100.0,
                viewport_h: 300.0,
                content_h: 2000.0,
                viewport_y: 60.0,
                viewport_w: 1087.0,
            },
        );
        m.bands.insert(
            "p".into(),
            BandRectJson {
                y: 0.0,
                h: 0.0,
                start: 0,
                end: 1,
            },
        );
        let _ = note_measure(&m);
        assert!(read_scope("p", |e| e.pane.is_some()).unwrap());
    }

    #[test]
    fn enabled_defaults_true_and_switch_reads_env() {
        // 默认（未设）= 启用；显式 0 关闭由记载的 env 约定保证。
        assert!(enabled() || std::env::var("AUTOUI_VIRTUAL_LIST").as_deref() == Ok("0"));
    }

    #[test]
    fn probe_key_parsing() {
        let id = Id::from(format!("{PANE_ID_PREFIX}abc"));
        assert_eq!(VListProbe::key_of(&id, PANE_ID_PREFIX).as_deref(), Some("abc"));
        let other = Id::from("vnode_1");
        assert_eq!(VListProbe::key_of(&other, PANE_ID_PREFIX), None);
    }

    fn pane(offset_y: f64, viewport_h: f64, viewport_w: f64) -> PaneGeometry {
        PaneGeometry {
            offset_y,
            viewport_h,
            content_h: 3669.0,
            viewport_y: 60.0,
            viewport_w,
        }
    }

    fn calib_plan(total: usize, rows: usize) -> WindowPlan {
        WindowPlan {
            total,
            start: 0,
            end: rows,
            row_h: 0.0,
            list_top: 0.0,
            offset_y: 0.0,
            viewport_h: 739.6,
        }
    }

    #[test]
    fn calibration_requires_two_consistent_measurements() {
        reset_for_test();
        note_pane("c1", pane(0.0, 739.6, 1087.0));
        note_plan("c1", calib_plan(120, 120));
        // 首测（世界/字型稳定前实测偏高）→ 候选但不采信。
        note_band("c1", 0, 120, 237.0, 3456.0);
        assert!(plan("c1", 120).is_none(), "unconfirmed height must keep calibrating");
        // 第二次与首次不一致 → 仍不采信，但候选更新到新值。
        force_calib_sample_due_for_test("c1");
        note_band("c1", 0, 120, 237.0, 3477.0);
        assert!(plan("c1", 120).is_none());
        // 第三次与候选一致 → 采信。
        force_calib_sample_due_for_test("c1");
        note_band("c1", 0, 120, 237.0, 3477.0);
        let p = plan("c1", 120).expect("trusted row height");
        assert!((p.row_h - 28.975).abs() < 1e-3, "row_h={}", p.row_h);
        assert!((p.list_top - 177.0).abs() < 1e-3, "list_top={}", p.list_top);
        assert_eq!((p.start, p.end), (0, 22));
    }

    #[test]
    fn windowed_band_does_not_override_calibrated_row_height() {
        reset_for_test();
        note_pane("w1", pane(2909.0, 739.6, 1087.0));
        note_plan("w1", calib_plan(120, 120));
        note_band("w1", 0, 120, 237.0, 3477.0);
        force_calib_sample_due_for_test("w1");
        note_band("w1", 0, 120, 237.0, 3477.0);
        let before = plan("w1", 120).expect("trusted").row_h;
        assert!((before - 28.975).abs() < 1e-3);
        // 尾部窗口带平均行高略低（28.79）——不得覆盖标定行高，只重算 list_top。
        note_plan(
            "w1",
            WindowPlan {
                total: 120,
                start: 92,
                end: 120,
                row_h: before,
                list_top: 177.0,
                offset_y: 2909.0,
                viewport_h: 739.6,
            },
        );
        note_band("w1", 92, 120, 2886.0, 806.0);
        let e = read_scope("w1", |e| (e.row_h, e.list_top)).unwrap();
        assert!((e.0 - 28.975).abs() < 1e-6, "row_h={}", e.0);
        // list_top = 带顶 - start*row_h - 视口顶
        let expect = 2886.0 - 92.0 * 28.975 - 60.0;
        assert!((e.1 - expect).abs() < 1e-3, "list_top={}", e.1);
    }

    #[test]
    fn viewport_width_change_invalidates_row_height() {
        reset_for_test();
        seed_geometry_full_for_test("r1", 0.0, 739.6, 29.0, 177.0, 60.0, 1087.0);
        assert!(plan("r1", 120).is_some());
        // 同宽重测 → 保留。
        note_pane("r1", pane(0.0, 739.6, 1087.0));
        assert!(has_row_height("r1"));
        // 窗口缩放（视口宽变） → 行高作废，下一帧重新标定。
        note_pane("r1", pane(0.0, 739.6, 900.0));
        assert!(!has_row_height("r1"), "width change must invalidate row height");
        assert!(plan("r1", 120).is_none());
    }

    #[test]
    fn periodic_verification_adopts_settled_row_height() {
        reset_for_test();
        note_pane("v2", pane(0.0, 739.6, 1087.0));
        note_plan("v2", calib_plan(120, 120));
        note_band("v2", 0, 120, 237.0, 3456.0);
        force_calib_sample_due_for_test("v2");
        note_band("v2", 0, 120, 237.0, 3456.0);
        assert!((plan("v2", 120).expect("trusted").row_h - 28.8).abs() < 1e-3);
        // 复核到期 → 下一次构建改做标定；稳定后的全表测量纠偏行高。
        force_verify_due_for_test("v2");
        assert!(plan("v2", 120).is_none(), "verification build must be a calibration");
        note_plan("v2", calib_plan(120, 120));
        force_calib_sample_due_for_test("v2");
        note_band("v2", 0, 120, 237.0, 3477.0);
        let p = plan("v2", 120).expect("trusted after verification");
        assert!((p.row_h - 28.975).abs() < 1e-3, "row_h={}", p.row_h);
    }

    #[test]
    fn mark_id_carries_band_range() {
        let id = Id::from(format!("{MARK_ID_PREFIX}vlist_3:10:40"));
        assert_eq!(
            VListProbe::parse_mark(&id),
            Some(("vlist_3".to_string(), 10, 40))
        );
        let legacy = Id::from(format!("{MARK_ID_PREFIX}vlist_3"));
        assert_eq!(VListProbe::parse_mark(&legacy), None);
        assert_eq!(VListProbe::id_string(&id), format!("{MARK_ID_PREFIX}vlist_3:10:40"));
    }

    // ------------------------------------------------------------------
    // PLAN-126 T-03 续：显式 row-height 入口 + V-02 定向用例
    // ------------------------------------------------------------------

    #[test]
    fn plan_follows_offset_change() {
        reset_for_test();
        seed_geometry_for_test("o1", 0.0, 740.0, 28.0, 0.0);
        let top = plan("o1", 1000).expect("top window");
        assert_eq!((top.start, top.end), (0, 29));
        // 滚动到 700：floor(700/28)=25-2=23；ceil(1440/28)=52+2=54。
        seed_geometry_for_test("o1", 700.0, 740.0, 28.0, 0.0);
        let scrolled = plan("o1", 1000).expect("scrolled window");
        assert_eq!((scrolled.start, scrolled.end), (23, 54));
        assert!(scrolled.start > top.start);
    }

    #[test]
    fn offset_moved_detects_scroll_and_note_measure_updates_window() {
        reset_for_test();
        // 完整的种子几何（含视口宽）——probe 回执同宽，避免「宽度变化作废
        // 行高」分支干扰本用例。
        seed_geometry_full_for_test("om", 0.0, 740.0, 28.0, 0.0, 0.0, 1087.0);
        assert!(plan("om", 1000).is_some());
        let mut m = VListMeasure::default();
        m.panes.insert(
            "om".into(),
            PaneGeometryJson {
                offset_y: 700.0,
                viewport_h: 740.0,
                content_h: 28_000.0,
                viewport_y: 0.0,
                viewport_w: 1087.0,
            },
        );
        assert!(offset_moved(&m), "offset change must dirty the frame");
        assert!(note_measure(&m));
        assert_eq!(
            plan("om", 1000).map(|p| (p.start, p.end)),
            Some((23, 54)),
            "host probe result recomputes the window"
        );
        // 同一偏移重复 probe → 不置脏。
        assert!(!offset_moved(&m));
    }

    #[test]
    fn viewport_height_change_recomputes_window() {
        reset_for_test();
        seed_geometry_full_for_test("vh1", 0.0, 740.0, 28.0, 0.0, 0.0, 1087.0);
        assert_eq!(plan("vh1", 1000).map(|p| (p.start, p.end)), Some((0, 29)));
        // 宽度不变 → 行高保留；视口变矮 → 窗口行数收缩。
        note_pane("vh1", pane(0.0, 300.0, 1087.0));
        assert_eq!(plan("vh1", 1000).map(|p| (p.start, p.end)), Some((0, 13)));
    }

    #[test]
    fn window_clamps_tail_short_empty_and_outside() {
        reset_for_test();
        // 尾部：内容 28000、视口 740 → 最大偏移 27260，end 夹到 n。
        seed_geometry_for_test("c1", 27_260.0, 740.0, 28.0, 0.0);
        let tail = plan("c1", 1000).expect("tail");
        assert_eq!(tail.end, 1000);
        assert_eq!(tail.start, 971);
        // 短于视口：整表都在窗口内，且无 overscan 越界。
        seed_geometry_for_test("c1", 0.0, 740.0, 28.0, 0.0);
        assert_eq!(plan("c1", 10).map(|p| (p.start, p.end)), Some((0, 10)));
        // 空列表：不产生窗口（调用方保留完整构建的空结果）。
        assert!(plan("c1", 0).is_none());
        // 完全在视口下方（列表起点 400 > 视口底 300）。
        seed_geometry_for_test("c1", 0.0, 300.0, 28.0, 400.0);
        assert_eq!(
            visible_range(0.0, 300.0, 400.0, 28.0, 50, OVERSCAN_ROWS),
            (0, 0)
        );
        // 完全在视口上方（偏移远超内容）。
        seed_geometry_for_test("c1", 100_000.0, 740.0, 28.0, 0.0);
        let out = plan("c1", 50).expect("outside still plans an empty band");
        assert_eq!(out.start, out.end);
        assert_eq!(out.start, 50);
    }

    #[test]
    fn explicit_row_height_skips_calibration() {
        reset_for_test();
        assert!(plan("e1", 1000).is_none());
        note_explicit_row_height("e1", 28.0, None);
        assert_eq!(explicit_row_height("e1"), Some(28.0));
        // 无视口几何 → 有界顶窗（尾部占位把总高度补齐）。
        let boot = explicit_plan("e1", 1000).expect("bounded bootstrap");
        assert_eq!((boot.start, boot.end), (0, 256));
        assert!((boot.row_h - 28.0).abs() < 1e-9);
        assert!(
            plan("e1", 1000).is_none(),
            "viewport still comes from probe"
        );
        // 只有视口/偏移的 probe 回执（无构建带）即可窗口化：行高已采信。
        note_measure(&{
            let mut m = VListMeasure::default();
            m.panes.insert(
                "e1".into(),
                PaneGeometryJson {
                    offset_y: 700.0,
                    viewport_h: 740.0,
                    content_h: 28_000.0,
                    viewport_y: 0.0,
                    viewport_w: 1087.0,
                },
            );
            m
        });
        let p = plan("e1", 1000).expect("explicit window");
        assert_eq!((p.start, p.end), (23, 54));
        // 实测构建带不得覆盖显式行高（窗口带均值可偏 0.5%）。
        note_band("e1", 23, 54, 23.0 * 28.0, 31.0 * 28.4);
        assert_eq!(explicit_row_height("e1"), Some(28.0));
        assert!((plan("e1", 1000).expect("still windowed").row_h - 28.0).abs() < 1e-9);
        // 视口宽度变化（窗口缩放）不作废显式行高。
        note_pane("e1", pane(700.0, 740.0, 900.0));
        assert!(plan("e1", 1000).is_some());
    }

    #[test]
    fn explicit_row_height_rejects_invalid_values() {
        reset_for_test();
        note_explicit_row_height("bad", 0.0, None);
        note_explicit_row_height("bad", -5.0, None);
        note_explicit_row_height("bad", f64::NAN, None);
        note_explicit_row_height("bad", 500.0, None);
        assert_eq!(explicit_row_height("bad"), None);
        assert!(explicit_plan("bad", 100).is_none());
    }

    #[test]
    fn build_counters_record_window_reads_and_builds() {
        reset_for_test();
        let c = build_counters();
        assert_eq!(c, BuildCounters::default());
        seed_geometry_for_test("bc", 0.0, 740.0, 28.0, 0.0);
        let p = plan("bc", 1000).expect("plan");
        for _ in 0..p.rows() {
            record_list_get();
        }
        record_build("bc", &p);
        record_rows_materialized(1000);
        let c = build_counters();
        assert_eq!(c.list_gets, p.rows() as u64);
        assert_eq!(c.rows_built, p.rows() as u64);
        assert_eq!(c.windows_built, 1);
        assert_eq!(c.rows_read, p.rows() as u64 + 1000);
        reset_for_test();
        assert_eq!(build_counters(), BuildCounters::default());
    }

    #[test]
    fn enabled_override_is_thread_local_for_tests() {
        reset_for_test();
        set_enabled_override_for_test(Some(false));
        assert!(!enabled());
        set_enabled_override_for_test(Some(true));
        assert!(enabled());
        set_enabled_override_for_test(None);
        assert!(enabled() || std::env::var("AUTOUI_VIRTUAL_LIST").as_deref() == Ok("0"));
    }
}
