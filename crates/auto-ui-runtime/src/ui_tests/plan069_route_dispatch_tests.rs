//! PLAN-069 T-03（round-19 家族 F3）与 T-04（F4）永久反例：
//! 路由页实例的名字合成派发解析 + 卡行双参派发的路由提交。
//!
//! 缺陷形态（T-00 探针实测，修前）：
//! - 路由页实例身份带 outlet 路由装饰（`reader@/book/7/chapter/1`），旧解析
//!   `identity.rsplit('/').next().split('#')` 取到的是**路由尾段**（"1"，页面
//!   在 "/" 时是 ""）→ 类型名恒不匹配 → 名字合成派发（无实例标识的通道，
//!   含部分真实输入/合成动作）退到根态 → 页 `.route_id` 读到空 →
//!   Back 链 `router.push("/book/" + "")` = `/book/`（READER-009 现场）。
//!   修前探针日志：name-only `Probe` 后页实例 `seen_back` 保持 ""；
//!   修后同一派发写入 "/book/b1"。
//! - 卡行（`for` 内 row `onclick: .CardClick(id, selected)`，双参）经合成/
//!   真实派发必须提交路由并让目标页 Init 读回正确 param（F4 的 VM 链路 oracle）。
//!
//! 修复口径：实例注册记录携带 widget 类型名，名字类解析一律按注册类型精确
//! 匹配（`VmBridge::child_instance_identities_of_type`），不再从身份串后缀猜。

use crate::build_dynamic_component;
use crate::ui::interpreter::DynamicMessage;

fn write_fixture(dir: &std::path::Path) -> std::path::PathBuf {
    let pages = dir.join("pages");
    std::fs::create_dir_all(&pages).expect("pages dir");
    std::fs::write(
        dir.join("app.at"),
        r#"
widget App {
    routes {
        "/" -> use home
        "/book/:id" -> use detail
        "/book/:id/chapter/:ch" -> use reader
    }
    view {
        outlet
    }
}
"#,
    )
    .expect("app.at");
    std::fs::write(
        pages.join("home.at"),
        r#"
widget home {
    msg { Init, CardClick(str, bool) }
    model {
        var books = []
        var last_selected str = ""
        var select_mode bool = false
    }
    view {
        col {
            for b in .books {
                row {
                    id: b.id
                    key: b.id
                    onclick: .CardClick(b.id, b.selected)
                    text b.title
                }
            }
        }
    }
    on {
        .Init -> {
            .books.push({ id: "b1", title: "Book One", selected: false })
            .books.push({ id: "b2", title: "Book Two", selected: true })
        }
        .CardClick(bid str, selected bool) -> {
            // Reader bookshelf.CardClick 同型：先读页模型字段（select_mode），
            // 再按选择态分派。名字合成派发落到根态时该字段不存在 → handler
            // 早退/崩 → 路由不提交（F4 现场「ok 含 handler 名但无路由变更」）。
            if .select_mode {
                .last_selected = bid
                return
            }
            if selected {
                .last_selected = bid
            } else {
                router.push("/book/" + bid + "/chapter/1")
            }
        }
    }
}
"#,
    )
    .expect("home.at");
    std::fs::write(
        pages.join("detail.at"),
        r#"
widget detail {
    msg { Init }
    model {
        var route_id str = ""
    }
    view {
        text .route_id {}
    }
    on {
        .Init -> {
            .route_id = router.param("id")
        }
    }
}
"#,
    )
    .expect("detail.at");
    std::fs::write(
        pages.join("reader.at"),
        r#"
widget reader {
    msg { Init, Back, Probe }
    model {
        var route_id str = ""
        var chapter_num int = 1
        var seen_back str = ""
        var init_runs int = 0
    }
    view {
        col {
            button "back" { onclick: .Back }
            button "probe" { onclick: .Probe }
            text .route_id {}
        }
    }
    on {
        .Init -> {
            .init_runs = .init_runs + 1
            .route_id = router.param("id")
            .chapter_num = router.param("ch").to_int()
            router.push("/book/" + .route_id + "/chapter/" + .chapter_num.to_string())
        }
        .Back -> {
            .seen_back = "/book/" + .route_id
            router.push("/book/" + .route_id)
        }
        .Probe -> {
            .seen_back = "/book/" + .route_id
        }
    }
}
"#,
    )
    .expect("reader.at");
    dir.join("app.at")
}

fn build(dir: &std::path::Path) -> crate::ui::dynamic::DynamicComponent {
    let app = write_fixture(dir);
    let code = std::fs::read_to_string(&app).expect("read app.at");
    build_dynamic_component(&code, Some(&app.to_string_lossy())).expect("route fixture compiles")
}

fn enter_reader(dc: &mut crate::ui::dynamic::DynamicComponent) {
    dc.fire_init();
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);
    // 卡行派发（与真实 click / MCP press 同型：双参载荷编码）。
    let ev = crate::ui::mcp_types::encode_payload(
        "CardClick",
        &[auto_val::Value::str("b1"), auto_val::Value::Bool(false)],
    );
    dc.on_with_input_for("home", &ev, None);
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);
}

fn page_field(
    dc: &crate::ui::dynamic::DynamicComponent,
    widget: &str,
    field: &str,
) -> Option<auto_val::Value> {
    let identity = dc
        .bridge()
        .child_instance_identities_of_type(widget)
        .into_iter()
        .next()?;
    dc.bridge().read_instance_state(&identity, field).ok()
}

fn router_state(dc: &crate::ui::dynamic::DynamicComponent) -> Option<auto_val::Value> {
    dc.read_state("__current_route").ok()
}

/// T-04 / AC-05 VM 链路：`for` 内卡行双参 onclick 派发提交路由，目标页
/// `.Init` 的 `router.param("id")` 读回同一 book id（所见=所用）。
#[test]
fn f4_loop_card_click_commits_route_and_param() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut dc = build(dir.path());
    enter_reader(&mut dc);

    assert_eq!(
        router_state(&dc),
        Some(auto_val::Value::Str("/book/b1/chapter/1".into())),
        "卡行派发必须提交路由（修前现场：ActionResult ok 但无页面挂载）"
    );
    assert_eq!(
        page_field(&dc, "reader", "route_id"),
        Some(auto_val::Value::Str("b1".into())),
        "目标页 Init 的 router.param(\"id\") 必须读回 b1"
    );
    assert_eq!(
        page_field(&dc, "reader", "chapter_num"),
        Some(auto_val::Value::Int(1)),
        "chapter 段 param 必须读回 1"
    );
}

/// T-03 / AC-04：名字合成派发（无实例标识）必须落到路由页实例——
/// 修前落到根态，页 `.route_id` 读空 → Back 链 `/book/`。
#[test]
fn f3_name_only_dispatch_reaches_route_page_instance() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut dc = build(dir.path());
    enter_reader(&mut dc);

    dc.on_with_input_for("reader", "Probe", None);
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);

    assert_eq!(
        page_field(&dc, "reader", "seen_back"),
        Some(auto_val::Value::Str("/book/b1".into())),
        "名字合成派发必须写页实例字段（修前写根态 → 页 route_id 空 → Back /book/）"
    );
    assert_eq!(
        page_field(&dc, "reader", "route_id"),
        Some(auto_val::Value::Str("b1".into())),
        "页实例的 route_id 不得因派发解析丢失"
    );
}

/// T-03 / AC-04 端到端：Back 链从路由页构建正确前缀路径并装载详情页
/// （`Back -> router.push("/book/" + .route_id)`）。
#[test]
fn f3_back_chain_builds_param_route_and_mounts_detail() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut dc = build(dir.path());
    enter_reader(&mut dc);

    // 实例身份形态（真实 click 的 IcedMessage 携带）与名字合成形态都应正确。
    dc.on_with_input_for("reader", "Back", None);
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);

    assert_eq!(
        router_state(&dc),
        Some(auto_val::Value::Str("/book/b1".into())),
        "Back 必须 push 带 book id 的详情路由（修前 \"/book/\" 无路由可挂）"
    );
    assert!(
        dc.bridge()
            .child_instance_identities_of_type("detail")
            .len()
            == 1,
        "详情页实例必须挂载；实际实例表：{:?}",
        dc.bridge().child_instance_identities()
    );
    assert_eq!(
        page_field(&dc, "detail", "route_id"),
        Some(auto_val::Value::Str("b1".into())),
        "详情页 Init 必须读到同一 book id"
    );
}

/// 对照：实例身份形态派发（真实输入携带 identity）不受影响。
#[test]
fn f3_instance_carrying_dispatch_unchanged() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut dc = build(dir.path());
    enter_reader(&mut dc);

    let (view, _, _) = dc.view_with_debug_gated(true);
    fn find_probe(view: &crate::ui::view::View<DynamicMessage>) -> Option<(String, String)> {
        if let Some((w, e)) = crate::ui::mcp_server::extract_action_from_view(view, "press") {
            if e.contains("Probe") {
                return Some((w, e));
            }
        }
        for c in crate::ui::vnode_converter::extract_children_ref(view) {
            if let Some(hit) = find_probe(c) {
                return Some(hit);
            }
        }
        None
    }
    let (widget, event) = find_probe(&view).expect("Probe 按钮必须可从 view 提取");
    assert!(
        widget.contains('\u{1}'),
        "提取面必须携带实例身份（修后不得退回名字态）；实际 widget={widget:?}"
    );
    dc.on_with_input_for(&widget, &event, None);
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);
    assert_eq!(
        page_field(&dc, "reader", "seen_back"),
        Some(auto_val::Value::Str("/book/b1".into()))
    );
}
