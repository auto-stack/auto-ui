//! PLAN-126 T-03 续：普通（非表格）列表的显式窗口化入口 + V-01/V-02 取证。
//!
//! 全部走真实管线：inline `.at` 源 → `build_dynamic_component`
//!（`convert_node_with` 的 ForLoop 臂）→ `view_with_debug_gated`
//! → `View` 树 / `view_to_vtree`，**无 mock 视图树**。
//!
//! 覆盖：
//! - V-01：1,000 / 10,000 行在相同 pane 几何下的窗口行读取/构建/布局节点数
//!   （窗口化 vs `AUTOUI_VIRTUAL_LIST=0` 完整构建对照）；
//! - 显式入口：`scroll (row-height: N)` 声明固定行高 → 首帧有界顶窗 + 正确
//!   总逻辑高度，probe 到达视口几何后直接按窗口裁行（无标定等待）；
//! - V-02：窗口跟随滚动 / 视口高度变化 / 尾部夹紧 / 空列表 / 短于视口的
//!   列表 / 列表完全在视口之外。
//!
//! 运行（每个测试 < 10s）：
//! `cargo nextest run --locked -j8 -p auto-ui-runtime --lib --features ace-bridge virtual_list`

use std::time::Instant;

use crate::ui::dynamic::DynamicComponent;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::view::View;
use crate::ui::virtual_list;

/// 单 `scroll` 的 fixture → 每帧首个 pane 作用域 key 恒为 `vlist_0`。
const PANE: &str = "vlist_0";

/// 固定行高（`row-height` prop 与种子几何共用，便于对照）。
const ROW_H: f64 = 28.0;
/// 视口高（≈ 025 实测 739.6）。
const VIEWPORT_H: f64 = 740.0;
/// 视口宽（宽度不变才不会作废行高）。
const VIEWPORT_W: f64 = 1087.0;

/// 普通列表 fixture：`scroll` + `for` 循环，每行一个文本节点。
///
/// 不是表格、不碰 examples；`row-height` 是 PLAN-126 T-03 的显式入口。
fn fixture(explicit_row_height: bool) -> String {
    let prop = if explicit_row_height {
        " (row-height: 28)"
    } else {
        ""
    };
    format!(
        r#"widget ScaleFixture {{
    model {{
        var rows = []
    }}
    view {{
        scroll{prop} {{
            for r in .rows {{
                text r.label {{ style: "h-7" }}
            }}
        }}
    }}
}}
"#
    )
}

/// 构建 fixture 并注入 `n` 行合成数据（`write_state_vec`，真实 VM 状态面）。
fn component(explicit_row_height: bool, n: usize) -> DynamicComponent {
    let mut dc = crate::build_dynamic_component(&fixture(explicit_row_height), None)
        .expect("fixture builds");
    let rows: Vec<auto_val::Value> = (0..n)
        .map(|i| {
            auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([(
                "label",
                auto_val::Value::Str(format!("row {i}").into()),
            )])))
        })
        .collect();
    dc.write_state_vec("rows", rows).expect("rows injected");
    dc
}

/// 从真实 `View` 树收集窗口证据：构建带区间、占位高度、文本行数。
#[derive(Debug, Default, Clone)]
struct ViewScan {
    marks: Vec<(String, usize, usize)>,
    space_heights: Vec<f32>,
    text_nodes: usize,
}

fn scan(view: &View<DynamicMessage>, out: &mut ViewScan) {
    match view {
        View::Text { .. } => out.text_nodes += 1,
        View::SpacePx { height, .. } => out.space_heights.push(*height),
        View::VirtualMark {
            key,
            start,
            end,
            child,
        } => {
            out.marks.push((key.clone(), *start, *end));
            scan(child, out);
        }
        View::Column { children, .. } | View::Row { children, .. } => {
            for c in children {
                scan(c, out);
            }
        }
        View::Scrollable { child, .. } | View::Container { child, .. } => scan(child, out),
        _ => {}
    }
}

fn scan_view(view: &View<DynamicMessage>) -> ViewScan {
    let mut out = ViewScan::default();
    scan(view, &mut out);
    out
}

/// 一次测量的 V-01 证据行。
#[derive(Debug, Clone)]
struct Metrics {
    n: usize,
    windowed: bool,
    window: (usize, usize),
    rows_read: u64,
    list_gets: u64,
    rows_built: u64,
    windows: u64,
    text_nodes: usize,
    total_nodes: usize,
    layout_nodes: usize,
    space_sum: f32,
    build_ms: f64,
}

impl Metrics {
    fn print(&self, tag: &str) {
        println!(
            "[V-01] tag={tag} n={} windowed={} window=[{}, {}) rows_read={} list_gets={} rows_built={} windows={} text_nodes={} total_nodes={} layout_nodes={} space_sum={:.1} build_ms={:.3}",
            self.n,
            self.windowed,
            self.window.0,
            self.window.1,
            self.rows_read,
            self.list_gets,
            self.rows_built,
            self.windows,
            self.text_nodes,
            self.total_nodes,
            self.layout_nodes,
            self.space_sum,
            self.build_ms,
        );
    }
}

/// 窗口化构建一次（台账已预置几何）：真实 ForLoop + View 管线。
fn run_windowed(n: usize, explicit_row_height: bool) -> (Metrics, DynamicComponent) {
    let dc = component(explicit_row_height, n);
    virtual_list::reset_for_test();
    virtual_list::seed_geometry_full_for_test(PANE, 700.0, VIEWPORT_H, ROW_H, 0.0, 0.0, VIEWPORT_W);
    let mut dc = dc;
    let t0 = Instant::now();
    let (view, _, _) = dc.view_with_debug_gated(false);
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let scan = scan_view(&view);
    let stats = crate::ui::view_to_vtree(view).stats();
    let plan = virtual_list::last_plan(PANE).expect("window plan recorded");
    let counters = virtual_list::build_counters();
    let metrics = Metrics {
        n,
        windowed: explicit_row_height,
        window: (plan.start, plan.end),
        rows_read: counters.rows_read,
        list_gets: counters.list_gets,
        rows_built: counters.rows_built,
        windows: counters.windows_built,
        text_nodes: scan.text_nodes,
        total_nodes: stats.total_nodes,
        layout_nodes: stats.layout_nodes,
        space_sum: scan.space_heights.iter().copied().sum(),
        build_ms,
    };
    metrics.print(if explicit_row_height {
        "windowed-explicit"
    } else {
        "windowed"
    });
    (metrics, dc)
}

/// 完整构建一次（开关关闭）：真实 ForLoop + View 管线，无窗口。
fn run_full(n: usize) -> Metrics {
    let mut dc = component(true, n);
    virtual_list::reset_for_test();
    let t0 = Instant::now();
    let (view, _, _) = dc.view_with_debug_gated(false);
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let scan = scan_view(&view);
    let stats = crate::ui::view_to_vtree(view).stats();
    let counters = virtual_list::build_counters();
    let metrics = Metrics {
        n,
        windowed: false,
        window: (0, scan.text_nodes),
        rows_read: counters.rows_read,
        list_gets: counters.list_gets,
        rows_built: counters.rows_built,
        windows: counters.windows_built,
        text_nodes: scan.text_nodes,
        total_nodes: stats.total_nodes,
        layout_nodes: stats.layout_nodes,
        space_sum: scan.space_heights.iter().copied().sum(),
        build_ms,
    };
    metrics.print("full-build");
    metrics
}

// ============================================================================
// V-01：1,000 / 10,000 行在相同几何下的窗口内容量
// ============================================================================

/// 窗口化断言类用例的环境前置：`AUTOUI_VIRTUAL_LIST=0` 时跳过本用例
///（开关关闭行为由 `plan126_virtual_list_scale_switch_off_control` 单独覆盖，
/// 以便整套 lib 在全局关闭窗口化的环境下仍可信）。
fn windowing_enabled_or_skip(test: &str) -> bool {
    if virtual_list::enabled() {
        return true;
    }
    eprintln!("[V-01] skip {test}: windowing disabled (AUTOUI_VIRTUAL_LIST=0)");
    false
}

#[test]
fn plan126_virtual_list_scale_windowed_1000() {
    if !windowing_enabled_or_skip("scale_windowed_1000") {
        return;
    }
    let (m, _) = run_windowed(1_000, true);
    // 窗口 = floor(700/28)-2 .. ceil(1440/28)+2 = [23, 54)
    assert_eq!(m.window, (23, 54), "window math");
    assert_eq!(m.text_nodes, 31, "only window rows are built");
    assert_eq!(m.list_gets, 31, "only window rows are read via list_get");
    assert_eq!(m.rows_read, 31, "rows read == window rows");
    assert_eq!(m.rows_built, 31);
    assert_eq!(m.windows, 1);
    // 顶部占位 23*28、尾部占位 (1000-54)*28 → 总逻辑高度 28000。
    assert!((m.space_sum - (1000.0 - 31.0) as f32 * 28.0).abs() < 1.0);
    assert!(m.build_ms < 10_000.0);
}

#[test]
fn plan126_virtual_list_scale_windowed_10000() {
    if !windowing_enabled_or_skip("scale_windowed_10000") {
        return;
    }
    let (m, _) = run_windowed(10_000, true);
    assert_eq!(m.window, (23, 54), "identical geometry → identical window");
    assert_eq!(m.text_nodes, 31);
    assert_eq!(m.list_gets, 31);
    assert_eq!(m.rows_read, 31, "10k rows still read only the window");
    assert_eq!(m.rows_built, 31);
    // 总逻辑高度 = 10000*28。
    assert!((m.space_sum - (10_000.0 - 31.0) as f32 * 28.0).abs() < 1.0);
}

/// V-01 核心：相同几何下 1,000 vs 10,000 的窗口内容量**逐项相等**（O(window)）。
#[test]
fn plan126_virtual_list_scale_is_bounded_not_linear() {
    if !windowing_enabled_or_skip("scale_is_bounded_not_linear") {
        return;
    }
    let (a, _) = run_windowed(1_000, true);
    let (b, _) = run_windowed(10_000, true);
    assert_eq!(a.window, b.window, "window independent of n");
    assert_eq!(a.rows_read, b.rows_read, "rows read bounded by window");
    assert_eq!(a.list_gets, b.list_gets, "list_get calls bounded by window");
    assert_eq!(a.rows_built, b.rows_built);
    assert_eq!(a.text_nodes, b.text_nodes);
    assert_eq!(
        a.total_nodes, b.total_nodes,
        "layout/vnode count bounded by window + constant"
    );
    assert_eq!(a.layout_nodes, b.layout_nodes);
    assert!(
        a.total_nodes < 64,
        "constant-size subtree: {}",
        a.total_nodes
    );
}

#[test]
fn plan126_virtual_list_scale_switch_off_control() {
    // 真实环境开关对照：`AUTOUI_VIRTUAL_LIST=0` 时 enabled() 必须为 false
    //（nextest 每测试独立进程，OnceLock 无跨测试污染）；未设 env 时用线程
    // 本地测试覆盖走同一条 enabled()==false 分支，保证本测试在默认档也确定。
    let env_off = std::env::var("AUTOUI_VIRTUAL_LIST").as_deref() == Ok("0");
    if env_off {
        assert!(
            !virtual_list::enabled(),
            "AUTOUI_VIRTUAL_LIST=0 must disable windowing"
        );
    } else {
        virtual_list::set_enabled_override_for_test(Some(false));
    }
    let a = run_full(1_000);
    let b = run_full(10_000);
    virtual_list::set_enabled_override_for_test(None);

    assert_eq!(a.list_gets, 0, "full build reads no rows via list_get");
    assert_eq!(a.rows_read, 1_000, "full build materializes the whole list");
    assert_eq!(b.rows_read, 10_000);
    assert_eq!(a.text_nodes, 1_000, "every row is built when disabled");
    assert_eq!(b.text_nodes, 10_000);
    assert_eq!(a.rows_built, 0);
    assert_eq!(b.rows_built, 0);
    assert_eq!(a.windows, 0);
    assert!(
        b.total_nodes > a.total_nodes * 5,
        "disabled control scales with n: {} vs {}",
        b.total_nodes,
        a.total_nodes
    );
}

// ============================================================================
// 显式入口：`scroll (row-height: N)` 无需标定
// ============================================================================

#[test]
fn plan126_virtual_list_explicit_row_height_entry() {
    if !windowing_enabled_or_skip("explicit_row_height_entry") {
        return;
    }
    virtual_list::reset_for_test();
    let mut dc = component(true, 1_000);

    // 首帧：无 probe 几何，但行高已知 → 有界顶窗 + 尾部占位（总高度正确）。
    let (view, _, _) = dc.view_with_debug_gated(false);
    let scan = scan_view(&view);
    assert_eq!(scan.marks.len(), 1, "explicit list is windowed");
    let (_, start, end) = scan.marks[0].clone();
    assert_eq!((start, end), (0, 256), "bounded top bootstrap");
    assert_eq!(
        scan.space_heights,
        vec![(1_000.0 - 256.0) as f32 * 28.0],
        "trailing spacer gives the full logical height from frame 1"
    );
    let boot = virtual_list::last_plan(PANE).expect("bootstrap plan");
    assert!((boot.row_h - ROW_H).abs() < 1e-9, "row height from prop");
    assert_eq!(virtual_list::explicit_row_height(PANE), Some(ROW_H));

    // probe 只回视口/偏移（无构建带也行）：行高已采信，直接按窗口裁行。
    let mut m = virtual_list::VListMeasure::default();
    m.panes.insert(
        PANE.to_string(),
        virtual_list::PaneGeometryJson {
            offset_y: 700.0,
            viewport_h: VIEWPORT_H,
            content_h: 28_000.0,
            viewport_y: 0.0,
            viewport_w: VIEWPORT_W,
        },
    );
    assert!(virtual_list::note_measure(&m));

    let (view2, _, _) = dc.view_with_debug_gated(false);
    let scan2 = scan_view(&view2);
    let (_, s2, e2) = scan2.marks[0].clone();
    assert_eq!(
        (s2, e2),
        (23, 54),
        "window follows the probe without a calibration sample"
    );
    assert_eq!(scan2.text_nodes, 31);

    // 实测构建带即使均值略偏也不得覆盖显式行高（只精修 list_top）。
    virtual_list::note_band(PANE, 23, 54, 23.0 * ROW_H, 31.0 * 28.4);
    assert_eq!(
        virtual_list::explicit_row_height(PANE),
        Some(ROW_H),
        "explicit row height survives band measurement"
    );
    // 视口宽度变化（窗口缩放）同样不作废显式行高。
    virtual_list::note_pane(
        PANE,
        virtual_list::PaneGeometry {
            offset_y: 700.0,
            viewport_h: VIEWPORT_H,
            content_h: 28_000.0,
            viewport_y: 0.0,
            viewport_w: 900.0,
        },
    );
    assert!(virtual_list::plan(PANE, 1_000).is_some());
}

/// 对照：未声明 `row-height` 的普通列表仍走标定（一次 probe 不足以窗口化）。
///
/// 走 **untracked** `Component::view()` 轨（`AuraViewBuilder::build`）：该轨
/// 对任意 `scroll` 都压 pane 作用域，因此能展示「无显式行高 → 标定帧」的
/// 对照；生产 tracked 轨对无 `row-height` 的显式 `scroll` 保持完整构建
///（见 `convert_scroll_tracked_ctx` 注释：窗口行 probe 保真待 T-04）。
#[test]
fn plan126_virtual_list_no_row_height_still_calibrates() {
    if !windowing_enabled_or_skip("no_row_height_still_calibrates") {
        return;
    }
    use crate::ui::component::Component;

    virtual_list::reset_for_test();
    let dc = component(false, 1_000);
    let view = dc.view();
    let scan = scan_view(&view);
    assert_eq!(scan.marks[0].1, 0);
    assert_eq!(scan.marks[0].2, 256, "calibration top band");
    assert_eq!(virtual_list::last_plan(PANE).map(|p| p.row_h), Some(0.0));

    let mut m = virtual_list::VListMeasure::default();
    m.panes.insert(
        PANE.to_string(),
        virtual_list::PaneGeometryJson {
            offset_y: 700.0,
            viewport_h: VIEWPORT_H,
            content_h: 28_000.0,
            viewport_y: 0.0,
            viewport_w: VIEWPORT_W,
        },
    );
    let _ = virtual_list::note_measure(&m);
    let _ = dc.view();
    assert_eq!(
        virtual_list::last_plan(PANE).map(|p| (p.start, p.end)),
        Some((0, 256)),
        "without row-height the pane keeps calibrating"
    );
}

// ============================================================================
// V-02：定向用例（台账/窗口数学 + 真实构建带）
// ============================================================================

/// 窗口跟随滚动：probe 偏移变化 → offset_moved 置脏信号 + 下一次构建换窗。
#[test]
fn plan126_virtual_list_v02_window_follows_scroll() {
    if !windowing_enabled_or_skip("v02_window_follows_scroll") {
        return;
    }
    virtual_list::reset_for_test();
    virtual_list::seed_geometry_full_for_test(PANE, 0.0, VIEWPORT_H, ROW_H, 0.0, 0.0, VIEWPORT_W);
    let mut dc = component(true, 1_000);
    let _ = dc.view_with_debug_gated(false);
    let top = virtual_list::last_plan(PANE).expect("top plan");
    assert_eq!((top.start, top.end), (0, 29));

    let mut m = virtual_list::VListMeasure::default();
    m.panes.insert(
        PANE.to_string(),
        virtual_list::PaneGeometryJson {
            offset_y: 700.0,
            viewport_h: VIEWPORT_H,
            content_h: 28_000.0,
            viewport_y: 0.0,
            viewport_w: VIEWPORT_W,
        },
    );
    assert!(
        virtual_list::offset_moved(&m),
        "scroll offset change must dirty the frame (host.rs:2047)"
    );
    let _ = virtual_list::note_measure(&m);
    let _ = dc.view_with_debug_gated(false);
    let scrolled = virtual_list::last_plan(PANE).expect("scrolled plan");
    assert_eq!((scrolled.start, scrolled.end), (23, 54));
    assert!(scrolled.start > top.start);
    // 同一偏移重复 probe → 不再置脏。
    assert!(!virtual_list::offset_moved(&m));
}

/// 视口高度变化：窗口行数随之收缩（宽度不变 → 行高保留）。
#[test]
fn plan126_virtual_list_v02_viewport_height_change() {
    if !windowing_enabled_or_skip("v02_viewport_height_change") {
        return;
    }
    virtual_list::reset_for_test();
    virtual_list::seed_geometry_full_for_test(PANE, 0.0, VIEWPORT_H, ROW_H, 0.0, 0.0, VIEWPORT_W);
    let mut dc = component(true, 1_000);
    let _ = dc.view_with_debug_gated(false);
    assert_eq!(
        virtual_list::last_plan(PANE).map(|p| (p.start, p.end)),
        Some((0, 29))
    );

    virtual_list::note_pane(
        PANE,
        virtual_list::PaneGeometry {
            offset_y: 0.0,
            viewport_h: 300.0,
            content_h: 28_000.0,
            viewport_y: 0.0,
            viewport_w: VIEWPORT_W,
        },
    );
    let _ = dc.view_with_debug_gated(false);
    // ceil(300/28)=11 +2 overscan = 13
    assert_eq!(
        virtual_list::last_plan(PANE).map(|p| (p.start, p.end)),
        Some((0, 13))
    );
}

/// 尾部夹紧 + 列表短于视口 + 空列表 + 列表完全在视口外。
#[test]
fn plan126_virtual_list_v02_clamp_short_empty_outside() {
    if !windowing_enabled_or_skip("v02_clamp_short_empty_outside") {
        return;
    }
    // ── 尾部夹紧：滚动到底部 → end 夹到 n，start = n - 窗口行数 ──
    virtual_list::reset_for_test();
    virtual_list::seed_geometry_full_for_test(
        PANE, 27_260.0, VIEWPORT_H, ROW_H, 0.0, 0.0, VIEWPORT_W,
    );
    let mut dc = component(true, 1_000);
    let (view, _, _) = dc.view_with_debug_gated(false);
    let scan = scan_view(&view);
    let (_, s, e) = scan.marks[0].clone();
    assert_eq!(e, 1_000, "tail clamped to n");
    assert_eq!(s, 971, "29-row tail window");
    assert_eq!(scan.text_nodes, 29);

    // ── 列表短于视口：全建、无占位 ──
    virtual_list::reset_for_test();
    virtual_list::seed_geometry_full_for_test(PANE, 0.0, VIEWPORT_H, ROW_H, 0.0, 0.0, VIEWPORT_W);
    let mut dc = component(true, 10);
    let (view, _, _) = dc.view_with_debug_gated(false);
    let scan = scan_view(&view);
    let (_, s, e) = scan.marks[0].clone();
    assert_eq!((s, e), (0, 10));
    assert_eq!(scan.text_nodes, 10);
    assert!(
        scan.space_heights.is_empty(),
        "short list needs no spacers: {:?}",
        scan.space_heights
    );

    // ── 空列表：不认领窗口，完整构建空内容 ──
    virtual_list::reset_for_test();
    virtual_list::seed_geometry_full_for_test(PANE, 0.0, VIEWPORT_H, ROW_H, 0.0, 0.0, VIEWPORT_W);
    let mut dc = component(true, 0);
    let (view, _, _) = dc.view_with_debug_gated(false);
    let scan = scan_view(&view);
    assert!(scan.marks.is_empty(), "empty list is not windowed");
    assert_eq!(scan.text_nodes, 0);
    assert_eq!(virtual_list::last_plan(PANE), None);
    assert_eq!(virtual_list::build_counters().rows_read, 0);

    // ── 列表完全在视口外（内容起点远在上方）：空窗口但总高度保留 ──
    virtual_list::reset_for_test();
    virtual_list::seed_geometry_full_for_test(
        PANE, 100_000.0, VIEWPORT_H, ROW_H, 0.0, 0.0, VIEWPORT_W,
    );
    let mut dc = component(true, 50);
    let (view, _, _) = dc.view_with_debug_gated(false);
    let scan = scan_view(&view);
    let (_, s, e) = scan.marks[0].clone();
    assert_eq!(s, e, "no visible rows → empty band");
    assert_eq!(scan.text_nodes, 0);
    assert_eq!(
        scan.space_heights.iter().sum::<f32>(),
        50.0 * 28.0,
        "content height preserved by the leading spacer"
    );
}
