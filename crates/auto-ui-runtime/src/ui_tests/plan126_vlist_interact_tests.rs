//! PLAN-126 T-04：窗口化 VList 的行交互语义（点击 / 右键 / 排序后 / 双 pane 隔离）。
//!
//! 全部走真实管线（inline `.at` 源 → `build_dynamic_component` →
//! `view_with_debug_gated` → `View` 树），与 T-03 续的规模用例同款，无 mock。
//!
//! 覆盖：
//! - 窗口行的事件实参保持**绝对数组索引**语义：窗口带第 k 行携带
//!   `rows[start+k]` 的 id，不前移、不重编号（点击 `onclick` 与
//!   `oncontextmenu` 两条绑定同判据）；
//! - 排序（数据源换序）后同一窗口位置携带**新顺序**对应的 id；
//! - 双 pane 隔离（AC-04）：同一窗口内两个 `scroll (row-height:)` 各自持有
//!   `vlist_0` / `vlist_1` 台账与窗口，行事件不串线；
//! - MCP 取址面：窗口行的 vnode path 经 `find_view_by_path` 精确落回该行
//!   （`autoui_action press` 的真实寻址路径），且 debug-id/probe 的已知缺口
//!   被显式钉住（窗口行由 untracked 转换构建 → 无 `for` 迭代条目）。
//!
//! 运行（每个测试 < 10s）：
//! `cargo nextest run --locked -j8 -p auto-ui-runtime --lib --features ace-bridge virtual_list`

use crate::ui::dynamic::DynamicComponent;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::view::View;
use crate::ui::virtual_list;

const ROW_H: f64 = 28.0;
const VIEWPORT_H: f64 = 740.0;
const VIEWPORT_W: f64 = 1087.0;

/// 单 pane fixture：每行 `row (onclick: .Sel(pid), oncontextmenu.prevent: .Ctx(pid))`。
///
/// 行列形状与 025 的 `table-row` 同构（行级 onclick + oncontextmenu，行内
/// 子节点为文本），但不依赖 examples。
fn fixture_single() -> &'static str {
    r#"widget InteractFixture {
    model {
        var rows = []
    }
    view {
        scroll (row-height: 28) {
            for r in .rows {
                row (onclick: .Sel(r.pid), oncontextmenu.prevent: .Ctx(r.pid)) {
                    text r.label {}
                }
            }
        }
    }
}
"#
}

/// 双 pane fixture：两个 `scroll (row-height: 28)` 各自一个循环。
fn fixture_two_panes() -> &'static str {
    r#"widget TwoPaneFixture {
    model {
        var rows = []
        var rows2 = []
    }
    view {
        row {
            scroll (row-height: 28) {
                for r in .rows {
                    row (onclick: .Sel(r.pid), oncontextmenu.prevent: .Ctx(r.pid)) {
                        text r.label {}
                    }
                }
            }
            scroll (row-height: 28) {
                for r in .rows2 {
                    row (onclick: .Sel2(r.pid), oncontextmenu.prevent: .Ctx2(r.pid)) {
                        text r.label {}
                    }
                }
            }
        }
    }
}
"#
}

fn row_value(pid: i32, label: &str) -> auto_val::Value {
    auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
        ("pid", auto_val::Value::Int(pid)),
        ("label", auto_val::Value::Str(label.to_string().into())),
    ])))
}

fn single(n: usize, pid_base: i32) -> DynamicComponent {
    let mut dc = crate::build_dynamic_component(fixture_single(), None).expect("fixture builds");
    let rows: Vec<auto_val::Value> = (0..n)
        .map(|i| row_value(pid_base + i as i32, &format!("proc-{i}")))
        .collect();
    dc.write_state_vec("rows", rows).expect("rows injected");
    dc
}

fn seed(scope: &str, offset: f64) {
    virtual_list::seed_geometry_full_for_test(
        scope, offset, VIEWPORT_H, ROW_H, 0.0, 0.0, VIEWPORT_W,
    );
}

/// 窗口行的 `(pid, has_onclick, has_context)`，按构建顺序。
fn row_pids(view: &View<DynamicMessage>) -> Vec<(i32, bool, bool)> {
    fn walk(v: &View<DynamicMessage>, out: &mut Vec<(i32, bool, bool)>) {
        match v {
            View::Row {
                children,
                onclick,
                on_right_click,
                ..
            } => {
                let pid = |m: &Option<DynamicMessage>| -> Option<i32> {
                    match m {
                        Some(DynamicMessage::Typed { args, .. }) => match args.first() {
                            Some(auto_val::Value::Int(i)) => Some(*i),
                            _ => None,
                        },
                        _ => None,
                    }
                };
                let click = pid(onclick);
                let ctx = pid(on_right_click);
                if click.is_none() && ctx.is_none() {
                    // layout-only row (e.g. the fixture's outer wrapper)
                    for c in children {
                        walk(c, out);
                    }
                    return;
                }
                out.push((
                    click.or(ctx).unwrap_or(-1),
                    click.is_some(),
                    ctx.is_some(),
                ));
                for c in children {
                    walk(c, out);
                }
            }
            View::Column { children, .. } => {
                for c in children {
                    walk(c, out);
                }
            }
            View::SpacePx { .. } | View::VirtualMark { .. } => {
                if let View::VirtualMark { child, .. } = v {
                    walk(child, out);
                }
            }
            View::Scrollable { child, .. } | View::Container { child, .. } => walk(child, out),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(view, &mut out);
    out
}

/// 收集 view 里所有 Scrollable 的 measure_key（scope 级登记）。
fn collect_scroll_keys(v: &View<DynamicMessage>) -> Vec<Option<String>> {
    let mut out = Vec::new();
    fn walk(v: &View<DynamicMessage>, out: &mut Vec<Option<String>>) {
        match v {
            View::Scrollable {
                child, measure_key, ..
            } => {
                out.push(measure_key.clone());
                walk(child, out);
            }
            View::Column { children, .. } | View::Row { children, .. } => {
                for c in children {
                    walk(c, out);
                }
            }
            View::VirtualMark { child, .. } | View::Container { child, .. } => {
                walk(child, out)
            }
            View::SpacePx { .. } => {}
            _ => {}
        }
    }
    walk(v, &mut out);
    out
}

fn windowing_enabled_or_skip(test: &str) -> bool {
    if virtual_list::enabled() {
        return true;
    }
    eprintln!("[T-04] skip {test}: windowing disabled (AUTOUI_VIRTUAL_LIST=0)");
    false
}

/// 窗口行的事件实参必须指向**绝对索引**：中间窗口第 k 行 = rows[start+k]。
#[test]
fn plan126_vlist_windowed_row_event_args_keep_absolute_index() {
    if !windowing_enabled_or_skip("windowed_row_event_args_keep_absolute_index") {
        return;
    }
    virtual_list::reset_for_test();
    seed("vlist_0", 700.0);
    let mut dc = single(120, 1000);
    let (view, _, _) = dc.view_with_debug_gated(false);
    let plan = virtual_list::last_plan("vlist_0").expect("window plan");
    assert_eq!((plan.start, plan.end), (23, 54), "window math");

    let rows = row_pids(&view);
    assert_eq!(rows.len(), plan.rows(), "one Row per window row");
    for (k, (pid, has_click, has_ctx)) in rows.iter().enumerate() {
        let abs = plan.start + k;
        assert!(has_click, "row {k} keeps its onclick binding");
        assert!(has_ctx, "row {k} keeps its oncontextmenu binding");
        assert_eq!(
            *pid,
            1000 + abs as i32,
            "window row {k} must carry rows[{abs}]'s pid (absolute index)"
        );
    }
}

/// 排序（数据源换序）后：同一窗口位置携带新顺序的 id，且窗口几何不变。
#[test]
fn plan126_vlist_windowed_row_event_args_follow_reorder() {
    if !windowing_enabled_or_skip("windowed_row_event_args_follow_reorder") {
        return;
    }
    virtual_list::reset_for_test();
    seed("vlist_0", 700.0);
    let mut dc = single(120, 1000);
    let (view, _, _) = dc.view_with_debug_gated(false);
    let before = row_pids(&view);
    assert_eq!(before[0].0, 1023);

    // 真实 VM 状态面换序（等价 025 的 `.ApplySort()` 重写 `.procsView`）。
    let reversed: Vec<auto_val::Value> = (0..120)
        .rev()
        .map(|i| row_value(1000 + i, &format!("proc-{i}")))
        .collect();
    dc.write_state_vec("rows", reversed).expect("reorder");
    let (view2, _, _) = dc.view_with_debug_gated(false);
    let plan = virtual_list::last_plan("vlist_0").expect("window plan after reorder");
    assert_eq!((plan.start, plan.end), (23, 54), "reorder keeps the window");
    let after = row_pids(&view2);
    assert_eq!(after.len(), before.len());
    for (k, (pid, _c, _x)) in after.iter().enumerate() {
        let abs = plan.start + k;
        // reversed[abs] has pid 1000 + (119 - abs)
        assert_eq!(
            *pid,
            1000 + (119 - abs as i32),
            "row {k} must follow the new order (reversed[{abs}])"
        );
    }
    assert_ne!(before[0].0, after[0].0, "the row identity really changed");
}

/// AC-04 实例隔离：同窗口两个 pane 各自台账/窗口，行事件不串线。
#[test]
fn plan126_vlist_two_panes_do_not_cross_wire() {
    if !windowing_enabled_or_skip("two_panes_do_not_cross_wire") {
        return;
    }
    virtual_list::reset_for_test();
    // pane A：120 行 pid 1000+；pane B：60 行 pid 2000+。
    seed("vlist_0", 0.0);
    seed("vlist_1", 0.0);

    let mut dc = crate::build_dynamic_component(fixture_two_panes(), None).expect("fixture");
    let rows_a: Vec<auto_val::Value> = (0..120)
        .map(|i| row_value(1000 + i, &format!("a-{i}")))
        .collect();
    let rows_b: Vec<auto_val::Value> = (0..60)
        .map(|i| row_value(2000 + i, &format!("b-{i}")))
        .collect();
    dc.write_state_vec("rows", rows_a).expect("a");
    dc.write_state_vec("rows2", rows_b).expect("b");

    let (view, _, _) = dc.view_with_debug_gated(false);
    let plan_a = virtual_list::last_plan("vlist_0").expect("pane A plan");
    let plan_b = virtual_list::last_plan("vlist_1").expect("pane B plan");
    assert_eq!(plan_a.total, 120, "pane A total");
    assert_eq!(plan_b.total, 60, "pane B total");
    // 同一视口/行高 → 两个窗口一致（各自独立计算，相同几何）。
    assert_eq!(plan_a.rows(), plan_b.rows());

    // 每行各自的 pid 段：A 段 1000..1120，B 段 2000..2060。
    let pids = row_pids(&view);
    assert_eq!(pids.len(), plan_a.rows() + plan_b.rows(), "both panes built");
    let (a, b) = pids.split_at(plan_a.rows());
    for (pid, has_click, has_ctx) in a {
        assert!(*has_click && *has_ctx, "pane A row keeps both bindings");
        assert!(
            (1000..1200).contains(pid),
            "pane A row must not carry pane B's pid: {pid}"
        );
    }
    for (pid, has_click, has_ctx) in b {
        assert!(*has_click && *has_ctx, "pane B row keeps both bindings");
        assert!(
            (2000..2100).contains(pid),
            "pane B row must not carry pane A's pid: {pid}"
        );
    }
    // 每个 pane 的窗口容器都有自己的 measure_key（scope 级登记，不共享）。
    let keys: Vec<String> = collect_scroll_keys(&view).into_iter().flatten().collect();
    assert_eq!(
        keys,
        vec!["vlist_0".to_string(), "vlist_1".to_string()],
        "each pane carries its own scope key"
    );
}

/// MCP 取址面：窗口行的 vnode path 经 `find_view_by_path` 精确落回该行。
///
/// 这是 `autoui_action press` 的真实寻址路径（vnode.path → typed View）。
/// `View::VirtualMark` / `View::SpacePx` 是 T-03 新增的窗口化包装，必须对
/// vnode 枚举透明（vnode_converter 透传标记），否则窗口行的 press 会落到
/// 相邻节点——本用例把这条不变量钉在真实构建产物上。
#[test]
fn plan126_vlist_windowed_row_vnode_paths_resolve_to_the_row() {
    if !windowing_enabled_or_skip("windowed_row_vnode_paths_resolve_to_the_row") {
        return;
    }
    use crate::ui::vnode_converter::extract_children_ref;
    use crate::ui::mcp_server::find_view_by_path;
    use crate::ui::vnode_converter::view_to_vtree_with_paths;

    virtual_list::reset_for_test();
    seed("vlist_0", 700.0);
    let mut dc = single(120, 1000);
    let (view, _, _) = dc.view_with_debug_gated(false);
    let plan = virtual_list::last_plan("vlist_0").expect("window plan");

    // 真实 View 树上每个带去重的 Row 的路径与 pid（与 vnode 枚举同构下探）。
    let mut rows: Vec<(Vec<u16>, i32)> = Vec::new();
    fn walk(
        v: &View<DynamicMessage>,
        path: &mut Vec<u16>,
        out: &mut Vec<(Vec<u16>, i32)>,
        extract: &dyn Fn(&View<DynamicMessage>) -> Vec<&View<DynamicMessage>>,
    ) {
        if let View::Row {
            onclick: Some(DynamicMessage::Typed { args, .. }),
            ..
        } = v
        {
            if let Some(auto_val::Value::Int(pid)) = args.first() {
                out.push((path.clone(), *pid));
            }
        }
        for (i, c) in extract(v).into_iter().enumerate() {
            path.push(i as u16);
            walk(c, path, out, extract);
            path.pop();
        }
    }
    let mut path = Vec::new();
    walk(&view, &mut path, &mut rows, &|v| extract_children_ref(v));
    assert_eq!(rows.len(), plan.rows(), "one resolvable row per window row");

    let vtree = view_to_vtree_with_paths(view.clone(), |_| None);
    for (row_path, pid) in &rows {
        assert!(
            vtree.nodes.iter().any(|n| &n.path == row_path),
            "vnode tree has no node at window-row path {row_path:?} (pid {pid})"
        );
        let target = find_view_by_path(&view, row_path)
            .unwrap_or_else(|| panic!("find_view_by_path miss at {row_path:?}"));
        match target {
            View::Row {
                onclick: Some(DynamicMessage::Typed { args, .. }),
                ..
            } => assert_eq!(
                args.first(),
                Some(&auto_val::Value::Int(*pid)),
                "path {row_path:?} resolved to the wrong row"
            ),
            other => panic!("path {row_path:?} resolved to non-row {other:?}"),
        }
    }
}

/// 探针保真缺口（T-04 结论，非回归）：窗口行由 `try_virtual_for` 的
/// **untracked** 转换构建，因此 tracked 轨的 `for` 迭代条目只覆盖完整构建。
///
/// 025 目标不受影响：`table-row` 在 tracked 轨同样 delegate 到 untracked
/// `convert_element`（tracked.rs:2248），行列的内部探针在开/关两侧都为空；
/// 受影响的只有「可见行的 `for` 迭代上下文」一条。此处把差异钉住，窗口行
/// 接入 tracked 转换时本用例应改为断言条目存在。
#[test]
fn plan126_vlist_windowed_rows_have_no_for_probe_entries() {
    if !windowing_enabled_or_skip("windowed_rows_have_no_for_probe_entries") {
        return;
    }
    fn for_entries(dc: &mut DynamicComponent) -> usize {
        let (_view, _ids, probe) = dc.view_with_debug_gated(true);
        probe
            .snapshot()
            .values()
            .filter(|e| e.for_context.is_some())
            .count()
    }

    // 窗口化：窗口行没有 for 迭代条目（untracked 构建）。
    virtual_list::reset_for_test();
    seed("vlist_0", 700.0);
    let mut dc = single(120, 1000);
    let windowed = for_entries(&mut dc);
    assert_eq!(windowed, 0, "windowed rows are built untracked");

    // 完整构建对照：每行一条迭代上下文。
    virtual_list::reset_for_test();
    virtual_list::set_enabled_override_for_test(Some(false));
    let mut dc2 = single(120, 1000);
    let full = for_entries(&mut dc2);
    virtual_list::set_enabled_override_for_test(None);
    assert_eq!(full, 120, "full build records one for-context per row");
}

