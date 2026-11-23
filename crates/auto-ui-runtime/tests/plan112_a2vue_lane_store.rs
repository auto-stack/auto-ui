//! PLAN-112 T-02 / V-02（A2Vue lane 的 store composable 发射）。
//!
//! 025 `SysStore` 是实后端 store：`use back.api: system_snapshot, kill_process`
//! + `use fmt: n1, rate`。lane 入口（`generate_sfcs_with_lane_product_stores`）
//! 此前把 store 的 `api_imports` 与模块函数池都留空——composable 无
//! `@/lib/api` 导入、API 调用不带 `await`、裸调 `n1`/`rate`（运行期
//! ReferenceError）。本文件钉住修复后的发射面，并以合成源做正反对照。
//!
//! 判别力（改动前产物不过）：基线二进制 dump 的 composable
//! （plan-artifacts/plan-112/probes/a2vue/stores/useSysStore.ts）只有一行
//! `vue` 导入、无任何 helper fn——下列 API 导入 / `await` / `n1`·`rate`·`n2`
//! 定义断言逐条失败。

#![cfg(feature = "ace-bridge")]

use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product_stores;
use std::path::{Path, PathBuf};

fn app_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/025-sys-monitor")
}

/// 入口可达的前端单元原样拼接（app + 两张子视图 + store 单元）。**不经过**
/// `prepare_component_source`：那是 AV 组件面单源投影的职责，会把 `use` 行
/// 空白化；lane 侧靠 `use` 行解析函数池与 API 导入（与 A2Vue 官方链同口径）。
/// `path` 取 store 单元的真实路径——`use fmt: …` 据此在磁盘上解析到 `fmt.at`。
fn lane_product_025() -> (
    auto_ui_runtime::ui::ace_vue_script::LaneSfcProductMulti,
    Vec<String>,
) {
    let front = app_dir().join("src/front");
    let mut merged = String::new();
    for unit in ["app.at", "processes.at", "performance.at", "sys_store.at"] {
        merged.push_str(&std::fs::read_to_string(front.join(unit)).expect("front unit"));
        merged.push('\n');
    }
    let store_path = front.join("sys_store.at");
    let (multi, stores) = generate_sfcs_with_lane_product_stores(
        &merged,
        &store_path.to_string_lossy(),
        "./a2x_rt.mjs",
        "../a2x_rt.mjs",
    )
    .expect("025 lane product derives");
    (multi, stores.into_iter().map(|s| s.code).collect())
}

#[test]
fn p112_lane_store_imports_backend_api_and_awaits_calls() {
    let (_multi, stores) = lane_product_025();
    assert_eq!(stores.len(), 1, "025 has exactly one store unit");
    let code = &stores[0];
    assert!(
        code.contains("import { system_snapshot, kill_process } from '@/lib/api'"),
        "SysStore must import the declared backend API:\n{}",
        &code[..code.len().min(400)]
    );
    // `use back.api` 的调用点按 API 语义 `await`（handler 随之 async）。
    assert!(code.contains("await system_snapshot()"), "snapshot call is awaited");
    assert!(code.contains("await kill_process("), "kill call is awaited");
    assert!(code.contains("const Refresh = async"), "Refresh is async");
    assert!(code.contains("const ConfirmKill = async"), "ConfirmKill is async");
}

#[test]
fn p112_lane_store_inlines_use_imported_helpers_with_closure() {
    let (_multi, stores) = lane_product_025();
    let code = &stores[0];
    // 显式导入名 `n1`/`rate` 被 handler 引用 → 拉取；`rate` 的被调 `n2` 不在
    // 导入清单里，经闭包一并拉取。各恰定义一次（同文件优先、不重复发射）。
    for helper in ["n1", "rate", "n2"] {
        assert_eq!(
            code.matches(&format!("function {helper}(")).count(),
            1,
            "helper `{helper}` must be defined exactly once"
        );
    }
    // 持久化语义与 vue 轨一致（`storage.*` → localStorage）。
    assert!(
        code.contains("localStorage.getItem('sysmon.speed')"),
        "storage.get projects to localStorage"
    );
    assert!(
        code.contains("localStorage.setItem('sysmon.speed', speed)"),
        "storage.set projects to localStorage"
    );
    // 动作暴露面完整：SFC 经 `store.X()` 调用，返回对象逐名在案。
    for action in [
        "Init", "Refresh", "Play", "Pause", "SetSpeed", "SelectTab", "SelectPerfTab",
        "SelectProc", "RequestKill", "CancelKill", "ConfirmKill", "RowCtx", "CloseCtx",
        "Sort", "ApplySort", "SortName", "SortCpu", "SortMem", "SortDisk", "SortNet", "SortPid",
    ] {
        assert!(
            code.contains(&format!("\n        {action},\n")),
            "store exposes action `{action}`"
        );
    }
}

#[test]
fn p112_lane_widgets_bind_the_store_surface() {
    let (multi, _stores) = lane_product_025();
    let sfc = |name: &str| -> String {
        multi
            .widgets
            .iter()
            .find(|w| w.name == name)
            .unwrap_or_else(|| {
                panic!(
                    "missing {name}; got {:?}",
                    multi.widgets.iter().map(|w| w.name.as_str()).collect::<Vec<_>>()
                )
            })
            .sfc
            .clone()
    };
    let app = sfc("App");
    assert!(app.contains("import { useSysStore } from '@/stores/useSysStore'"));
    // 原诊断面保留：`backend_ok` 两分支（「后端正常」↔「后端离线」+ 红点/徽标）。
    assert!(
        app.contains("<template v-if=\"store.backend_ok == 'true'\">")
            && app.contains("<template v-if=\"store.backend_ok != 'true'\">"),
        "backend_ok branches"
    );
    assert!(app.contains("后端正常") && app.contains("后端离线"), "status labels");
    // 关闭即释放刷新任务：卸载钩子清 Tick 定时器。
    assert!(app.contains("onUnmounted("), "unmount hook");
    assert!(app.contains("clearInterval(tickTimer.value)"), "tick timer is cleared");
    // 原 oracle T6（smoke.spec.ts）：速率档与暂停/继续控件在模板里。
    assert!(app.contains("@click=\"SetFast\">250ms") && app.contains("@click=\"SetNormal\">1s"));
    assert!(app.contains("@click=\"Pause\">⏸</button>") && app.contains("@click=\"Play\">▶</button>"));
    assert!(app.contains("<ProcessesView ") && app.contains("<PerformanceView "));

    // 表格族：Plain（lane）模式落原生表格元素——原 smoke 的 `table tbody tr` 选择器成立。
    let processes = sfc("ProcessesView");
    for needle in [
        "<table class=\"w-full border-collapse\">",
        "<thead>",
        "<tbody>",
        "<th class=\"text-right text-xs text-muted-foreground font-medium px-2 py-1.5 cursor-pointer select-none tabular-nums\" @click=\"SortMem\">",
        "<td class=\"text-xs font-medium px-2 py-1 truncate max-w-[140px]\">{{ p.name }}</td>",
        "@contextmenu.prevent=\"RowCtx(p.pid)\"",
    ] {
        assert!(processes.contains(needle), "ProcessesView lacks `{needle}`");
    }
    let row_at = processes.find("v-for=\"p in store.procsView\"").expect("row v-for");
    let row_open = processes[..row_at].rfind('<').expect("row open tag");
    assert!(
        processes[row_open..].starts_with("<tr "),
        "process rows must be native <tr>: {}",
        &processes[row_open..row_open + 40]
    );
    assert!(!processes.contains("<div :open="), "dialog `open` must not leak onto a div");

    // alert-dialog 族：原生模态（v-if 门控的遮罩 + 关闭语义 + 面板/标题/按钮语义）。
    for needle in [
        "<div v-if=\"store.kill_dialog_open\" role=\"alertdialog\" aria-modal=\"true\" class=\"fixed inset-0 z-50 flex items-center justify-center bg-black/80\" @click.self=\"store.kill_dialog_open = false\">",
        "class=\"bg-background text-foreground rounded-lg border shadow-lg p-6 w-full max-w-lg\"",
        "<h2 class=\"text-lg font-semibold\">结束任务</h2>",
        "<p class=\"text-sm text-muted-foreground\">确定结束 {{ store.kill_target_name }} (PID {{ store.kill_target }})",
        "<button class=\"inline-flex items-center justify-center rounded-md border bg-background px-4 py-2 text-sm font-medium\" @click=\"($event) => { store.kill_dialog_open = false; return CancelKill() }\">",
        "<button class=\"inline-flex items-center justify-center rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium\" @click=\"($event) => { store.kill_dialog_open = false; return ConfirmKill() }\">",
    ] {
        assert!(processes.contains(needle), "ProcessesView lacks `{needle}`");
    }
    // 按钮标签 = 位置参数文本（`alert-dialog-cancel "取消"`）——浏览器档靠 role+name 定位，
    // 空按钮会让「取消」不可达。
    assert_eq!(button_label(&processes, "($event) => { store.kill_dialog_open = false; return CancelKill() }"), "取消");
    assert_eq!(button_label(&processes, "($event) => { store.kill_dialog_open = false; return ConfirmKill() }"), "结束进程");

    let performance = sfc("PerformanceView");
    assert!(performance.contains("CPU 使用率实时曲线"), "performance tab (smoke T4)");
    assert!(
        performance.contains("<path :d=\"store.cpuLine\"") && performance.contains("<path :d=\"store.cpuArea\""),
        "live SVG curve bound to store geometry"
    );
}

/// 以 `@click="<click>"` 定位按钮，取其开标签之后、`</button>` 之前的可见文本。
fn button_label(sfc: &str, click: &str) -> String {
    let at = sfc
        .find(&format!("@click=\"{click}\""))
        .unwrap_or_else(|| panic!("no button with `{click}`"));
    let rest = &sfc[at..];
    let end = rest.find("</button>").expect("button close");
    rest[..end]
        .rsplit('>')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 用给定模式的生成器直接产单个 widget 的 SFC（对照 Plain 与 shadcn 两种模式）。
fn widget_sfc(src: &str, shadcn: bool) -> String {
    use auto_ui_dialect::UiStmt;
    let session = auto_parser_api::session::CompilerSession::ui();
    let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .expect("parse");
    let decl = ast
        .domain_stmts()
        .find_map(|s| match s {
            UiStmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = auto_ui_compiler::aura::extract_widget_from_decl(decl).expect("extract widget");
    let mut gen = if shadcn {
        auto_ui_compiler::ui_gen::VueGenerator::new_shadcn()
    } else {
        auto_ui_compiler::ui_gen::VueGenerator::new()
    };
    gen.generate_sfc(&widget).expect("sfc")
}

const TABLE_AND_DIALOG: &str = r#"
widget W {
    model {
        var open bool = false
        var name str = "x"
    }
    msg { Go, Stop }
    view {
        table {
            table-header {
                table-row {
                    table-head (onclick: .Go) { text .name {} }
                }
            }
            table-body {
                table-row (key: .name) {
                    table-cell (text: .name) {}
                }
            }
        }
        alert-dialog (open: .open) {
            alert-dialog-content {
                alert-dialog-header {
                    alert-dialog-title "T" {}
                    alert-dialog-description "D" {}
                }
                alert-dialog-footer {
                    alert-dialog-cancel "取消" { onclick: .Stop }
                    alert-dialog-action "结束" { onclick: .Go }
                }
            }
        }
    }
    on {
        .Go -> { .open = true }
        .Stop -> { .open = false }
    }
}
"#;

/// 共享生成器语义变更的对照面（Plain ↔ shadcn）：同一份源，Plain 落原生表格 +
/// 原生模态，shadcn 仍走 `TableRow`/`TableCell`/`AlertDialog*` 组件——新映射只在
/// Plain 生效，产品默认（shadcn）道逐字不变。
#[test]
fn p112_generator_plain_native_table_and_modal_leave_shadcn_mode_untouched() {
    let plain = widget_sfc(TABLE_AND_DIALOG, false);
    for needle in ["<thead>", "<tbody>", "<th ", "<tr ", "<td"] {
        assert!(plain.contains(needle), "plain: `{needle}` missing:\n{}", &plain[..plain.len().min(600)]);
    }
    assert!(plain.contains("v-if=\"open\" role=\"alertdialog\""), "plain modal is v-if gated");
    assert!(plain.contains("@click=\"($event) => { open = false; return Stop() }\""), "cancel closes + explicit handler call");
    assert!(plain.contains("@click=\"($event) => { open = false; return Go() }\""), "action closes + explicit handler call");
    assert_eq!(button_label(&plain, "($event) => { open = false; return Stop() }"), "取消", "cancel keeps its label");
    assert_eq!(button_label(&plain, "($event) => { open = false; return Go() }"), "结束", "action keeps its label");
    assert!(plain.contains(">T</h2>") && plain.contains(">D</p>"), "title/description keep their labels");

    let shadcn = widget_sfc(TABLE_AND_DIALOG, true);
    for needle in ["<TableRow", "<TableHead", "<TableCell", "<AlertDialog", "<AlertDialogCancel", "<AlertDialogAction"] {
        assert!(shadcn.contains(needle), "shadcn: `{needle}` missing:\n{}", &shadcn[..shadcn.len().min(600)]);
    }
    assert!(!shadcn.contains("role=\"alertdialog\""), "shadcn arm untouched by the native modal");
}

/// 合成正例：同文件 helper（PLAN-116 形态）与 `use back.api` 并存——两者都
/// 进发射面；**未被引用**的同文件 fn 不发射（门控 + 闭包，非整模块倾倒）。
/// 合成路径不落盘（`use back.api` 在函数池解析中被跳过，无 IO）。
#[test]
fn p112_lane_store_emits_same_file_helper_and_api_for_synthetic_source() {
    let src = r#"
use back.api: ping

fn twice(x int) int {
    return x * 2
}

fn unused_helper(x int) int {
    return x + 1
}

store CalcStore {
    model {
        var n int = 0
    }
    msg { Inc, Check }
    on {
        .Inc -> { .n = twice(.n) }
        .Check -> { ping() }
    }
}

widget App {
    view { text "x" {} }
}
"#;
    let (_multi, stores) = generate_sfcs_with_lane_product_stores(
        src,
        "p112-synthetic.at",
        "./a2x_rt.mjs",
        "../a2x_rt.mjs",
    )
    .expect("synthetic lane product derives");
    assert_eq!(stores.len(), 1);
    let code = &stores[0].code;
    assert!(code.contains("import { ping } from '@/lib/api'"), "{code}");
    assert!(code.contains("await ping()"), "api call is awaited: {code}");
    assert!(code.contains("function twice("), "same-file helper is emitted: {code}");
    assert!(
        !code.contains("function unused_helper("),
        "an unreferenced pool fn must not be emitted: {code}"
    );
}

/// 合成反例：不引 API、无 helper 的 store——发射面不得凭空出现 `@/lib/api`
/// 导入或 `await`（门控不误伤纯状态 store）。
#[test]
fn p112_lane_store_plain_store_stays_free_of_api_and_helpers() {
    let src = r#"
store PlainStore {
    model {
        var n int = 0
    }
    msg { Inc }
    on {
        .Inc -> { .n = .n + 1 }
    }
}

widget App {
    view { text "x" {} }
}
"#;
    let (_multi, stores) = generate_sfcs_with_lane_product_stores(
        src,
        "p112-synthetic.at",
        "./a2x_rt.mjs",
        "../a2x_rt.mjs",
    )
    .expect("plain synthetic lane product derives");
    let code = &stores[0].code;
    assert!(
        !code.contains("@/lib/api"),
        "no API import without `use back.api`: {code}"
    );
    assert!(!code.contains("await "), "no await without an API call: {code}");
}
