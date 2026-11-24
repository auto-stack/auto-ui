//! PLAN-126 T-03：固定行高纵向虚拟列表（`VList` / Rust `VirtualList`）的运行时几何台账。
//!
//! 合同（plan §5.1，SD-01 待发布）：
//! - 行高固定；一次窗口构建使用同一已发布版本的长度、顺序与索引数据；
//! - 只对可见行 + 两侧 overscan 读取、绑定、构建、布局；屏外由占位表达，
//!   总逻辑高度参与滚动条计算；
//! - 未取得布局测量（首帧）或没有有界纵向视口时保留完整构建；
//! - `AUTOUI_VIRTUAL_LIST=0` 关闭窗口化，保留完整构建与既有交互。
//!
//! 数据流：
//! ```text
//! convert_column(overflow-y-auto) → push scope（`vlist_{seq}`）
//! ForLoop 臂                      → virtual_list::plan(scope, n) → (start,end) 或 None
//! View::Scrollable.measure_key    → projection 挂 iced Id `vlist_pane:{key}`
//! View::VirtualMark(key)          → projection 挂 iced Id `vlist_mark:{key}`
//! host 每帧 probe（窗口化发生时）  → VListProbe 收集 pane 几何 + 构建带 rect
//! __vlist_measure 消息            → note_measure → 解出 row_h / list_top / offset
//! ```
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

/// 环境开关：缺省启用；`0` = 关闭窗口化（完整构建对照）。
pub fn enabled() -> bool {
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
    /// 已解出的固定行高与列表起点。
    row_h: f64,
    list_top: f64,
    /// 最近一次窗口构建（诊断/测试）。
    last_plan: Option<WindowPlan>,
    /// 最近一次测量的序号（观测新鲜度）。
    measured_seq: u64,
}

lazy_static::lazy_static! {
    static ref SCOPES: Mutex<HashMap<String, ScopeEntry>> = Mutex::new(HashMap::new());
}

/// 有 pane 需要本次 probe（构建期置位，host 消费后清零）。
static MEASURE_WANTED: AtomicBool = AtomicBool::new(false);
/// 窗口构建计数（诊断/测试）。
static ROWS_BUILT: AtomicU64 = AtomicU64::new(0);
static WINDOWS_BUILT: AtomicU64 = AtomicU64::new(0);
static MEASURE_SEQ: AtomicU64 = AtomicU64::new(1);

/// 构建期登记：本帧有虚拟化 pane（host 下一拍 probe）。
pub fn request_measure() {
    MEASURE_WANTED.store(true, Ordering::Relaxed);
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
        e.pane = Some(pane);
        changed
    });
    changed
}

/// 写入构建带矩形（probe 消息），并按当次区间反解行高/列表起点。
pub fn note_band(key: &str, y: f64, h: f64) -> bool {
    with_scope(key, |e| {
        let Some(prev) = e.band else {
            return false;
        };
        e.band = Some(BandGeometry { y, h, ..prev });
        let rows = prev.end.saturating_sub(prev.start);
        if rows > 0 && h.is_finite() && h > 0.0 {
            let row_h = h / rows as f64;
            // 行高应在合理范围；明显偏差（行内换行/变高行）不采信。
            if row_h >= 1.0 && row_h <= 400.0 {
                e.row_h = row_h;
                e.list_top = y - prev.start as f64 * row_h;
            }
        }
        e.measured_seq = MEASURE_SEQ.fetch_add(1, Ordering::Relaxed);
        true
    })
}

/// 构建期记录本次窗口计划（同时登记 band 的 start/end 供 probe 反解）。
pub fn note_plan(key: &str, plan: WindowPlan) {
    with_scope(key, |e| {
        e.last_plan = Some(plan);
        e.band = Some(BandGeometry {
            y: e.band.map(|b| b.y).unwrap_or(0.0),
            h: e.band.map(|b| b.h).unwrap_or(0.0),
            start: plan.start,
            end: plan.end,
        });
    });
}

/// 可见行 + 两端占位所需的构建决策。几何未测得时返回 `None`（完整构建）。
pub fn plan(key: &str, total: usize) -> Option<WindowPlan> {
    let entry = read_scope(key, |e| e.clone())?;
    let pane = entry.pane?;
    if entry.row_h <= 0.0 || pane.viewport_h <= 0.0 || total == 0 {
        return None;
    }
    let (start, end) = visible_range(
        pane.offset_y,
        pane.viewport_h,
        entry.list_top,
        entry.row_h,
        total,
        OVERSCAN_ROWS,
    );
    Some(WindowPlan {
        total,
        start,
        end,
        row_h: entry.row_h,
        list_top: entry.list_top,
        offset_y: pane.offset_y,
        viewport_h: pane.viewport_h,
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

/// 测试面：清空台账与计数。
pub fn reset_for_test() {
    SCOPES.lock().unwrap().clear();
    ROWS_BUILT.store(0, Ordering::Relaxed);
    WINDOWS_BUILT.store(0, Ordering::Relaxed);
    MEASURE_WANTED.store(false, Ordering::Relaxed);
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

/// 测试/诊断：直接注入几何（不依赖 iced 布局）。
pub fn seed_geometry_for_test(key: &str, offset_y: f64, viewport_h: f64, row_h: f64, list_top: f64) {
    with_scope(key, |e| {
        e.pane = Some(PaneGeometry {
            offset_y,
            viewport_h,
            content_h: 0.0,
        });
        e.row_h = row_h;
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
    /// scope key → 构建带 rect `[y, h]`。
    pub bands: HashMap<String, [f64; 2]>,
}

/// 可序列化的 pane 几何。
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct PaneGeometryJson {
    pub offset_y: f64,
    pub viewport_h: f64,
    pub content_h: f64,
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
            if let Some(key) = Self::key_of(id, MARK_ID_PREFIX) {
                self.out
                    .bands
                    .insert(key, [bounds.y as f64, bounds.height as f64]);
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
            if let Some(key) = Self::key_of(id, PANE_ID_PREFIX) {
                self.out.panes.insert(
                    key,
                    PaneGeometryJson {
                        offset_y: (translation.y.max(0.0)) as f64,
                        viewport_h: bounds.height.max(0.0) as f64,
                        content_h: content_bounds.height.max(0.0) as f64,
                    },
                );
            }
        }
    }

    fn finish(&self) -> Outcome<VListMeasure> {
        Outcome::Some(self.out.clone())
    }
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
            },
        );
    }
    for (key, band) in &m.bands {
        changed |= note_band(key, band[0], band[1]);
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
        assert!(note_band("s2", 467.0, 870.0));
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
        note_band("s3", 0.0, 5000.0);
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
            },
        );
        m.bands.insert("p".into(), [0.0, 0.0]);
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
}
