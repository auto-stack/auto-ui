//! PLAN-112 T-02/T-03（AV 腿上游状态钉点）。
//!
//! 025 `SysStore` 是实后端 store：记录列表 model 字段、`storage.*`、`use
//! back.api` 调用、`fmt.at` helper、`.as(float)`/`nil`、滑窗与排序循环。AV
//! （受检 AGE）两端——组件面 `prepare_component_source` →
//! `NodeDesktopBridge::load`，与包面 `load_package`（A2Vue 包面
//! `generate_package_sfc` 同一 AGE 包编译）——在 auto-lang `8a52d7364` 上均
//! 无法装载原样 025。
//!
//! 本文件是**状态钉点**而非功能用例（先例：`plan105_u03_sfc_status`）：
//! - `p112_av_control_*`：今天就被接受的对照（最小跨单元 store、同单元 store
//!   的 handler 调用/字段显示/`storage.set`/`while`）——缺口是具体构造，不是
//!   「store 语义整体不存在」。
//! - `p112_av_gap_*`：每个缺口一个最小源 + 首阻断诊断。
//! - `p112_av_real_*`：真实 025 单元的首阻断。
//!
//! 上游（auto-lang AGE）落地某项后，对应 `expect_gap` 会因「编译通过」或
//! 「诊断改名」翻红——届时把该条改为结构断言，并恢复 PLAN-112 的 T-03。
//! 构造级探针一律走生产路径（prep → load；prep 拒绝时退回裸源）——裸源直送
//! 会把 prep 已仿真的构造（字段显示、`storage.set`、`while`）误报成缺口。

#![cfg(feature = "ace-bridge")]

use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use auto_ui_runtime::ui::ace_node_desktop::NodeDesktopBridge;
use auto_ui_runtime::ui::ace_package_sfc::generate_package_sfc;
use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
use std::path::{Path, PathBuf};

fn app_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/025-sys-monitor")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(app_dir().join(rel)).unwrap_or_else(|_| panic!("read {rel}"))
}

fn unit(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

/// auto-lang AGE 包编译入口（诊断码 + 消息拼成一行；通过 = 空串）。
fn package_diag(units: &[PackageModuleInput], entry: &str) -> String {
    let scenario = ace_av::age_component::ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    match ace_core::source::age_package::compile_age_package_with_scenario(
        units,
        entry,
        ace_av::age_component::LegacyWidgetAlias::Accept,
        &[PACKAGES_CAPABILITY.into()],
        &scenario,
    ) {
        Ok(_) => String::new(),
        Err(diags) => diags
            .iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

/// 组件面生产路径：prep → load。返回首阻断（装载成功 = `OK`）。
fn component_face(root: &Path, src: &str) -> String {
    let prepped = prepare_component_source(root, src)
        .map(|p| p.source)
        .unwrap_or_else(|_| src.to_string());
    NodeDesktopBridge::load(&prepped, "p112-probe.at", 1)
        .err()
        .unwrap_or_else(|| "OK".into())
}

fn expect_gap(label: &str, diag: &str, needles: &[&str]) {
    assert!(!diag.is_empty() && diag != "OK", "{label}: 现已可装载——把本条改为结构断言");
    for needle in needles {
        assert!(
            diag.contains(needle),
            "{label}: 首阻断应含 `{needle}`，实际: {diag}"
        );
    }
}

fn expect_accepted(label: &str, diag: &str) {
    assert!(diag.is_empty() || diag == "OK", "{label}: 应被接受，实际首阻断: {diag}");
}

const STORE_UNIT: &str = r#"
store S {
    model {
        var n int = 0
    }
    msg { Inc }
    on {
        .Inc -> { .n = .n + 1 }
    }
}
"#;

const CONSUMER_READS_AND_CALLS: &str = r#"
use s: S

widget Main {
    msg { Go }
    view {
        text .store.n { }
        button "go" { onclick: .Go }
    }
    on {
        .Go -> { store.Inc() }
    }
}
"#;

/// 同单元 store + widget 探针源：store `S` 的 `.Go` handler = `handler`，widget
/// 经按钮调 `store.Go()`。
fn store_probe(top: &str, model_extra: &str, types: &str, handler: &str, view_extra: &str) -> String {
    format!(
        "{top}store S {{\n    model {{\n        var n int = 0\n        var label str = \"\"\n{model_extra}    }}\n    \
         msg {{ Go }}\n    on {{\n        .Go -> {{\n{handler}        }}\n    }}\n}}\n{types}\n\
         widget Main {{\n    msg {{ Go }}\n    view {{\n        button \"go\" {{ onclick: .Go }}\n        \
         text .label {{ }}\n{view_extra}    }}\n    \
         on {{\n        .Go -> {{ store.Go() }}\n    }}\n}}\n"
    )
}

fn synthetic_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("p112-age-probe-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

// ── 接受对照 ────────────────────────────────────────────────────────────────

/// 包面收最小跨单元 store：`use s: S` 消费者 + 视图读 `.store.n` + handler 调
/// `store.Inc()` 编译通过。
#[test]
fn p112_av_control_minimal_cross_unit_store_compiles_on_package_face() {
    expect_accepted(
        "最小跨单元 store（包面）",
        &package_diag(
            &[unit("main", CONSUMER_READS_AND_CALLS), unit("s", STORE_UNIT)],
            "app::main",
        ),
    );
}

/// 组件面（经 prep）收同单元 store：handler 调 `store.X()`、视图显示 store 字段。
#[test]
fn p112_av_control_store_call_and_field_display_load_on_component_face() {
    let root = synthetic_root();
    let src = store_probe("", "", "", "            .n = .n + 1\n", "        text .store.n { }\n");
    expect_accepted("同单元 store：调用 + 字段显示", &component_face(&root, &src));
    let _ = std::fs::remove_dir_all(&root);
}

/// 组件面（经 prep）收 `storage.set` 与 `while`（prep 的宿主存储仿真与循环改写）。
#[test]
fn p112_av_control_storage_set_and_while_load_through_prep() {
    let root = synthetic_root();
    let src = store_probe(
        "",
        "",
        "",
        "            storage.set(\"sysmon.speed\", .label)\n            var i int = 0\n            \
         while i < 3 {\n                i = i + 1\n            }\n            .n = i\n",
        "",
    );
    expect_accepted("同单元 store：storage.set + while", &component_face(&root, &src));
    let _ = std::fs::remove_dir_all(&root);
}

// ── 缺口钉点（组件面，经 prep）────────────────────────────────────────────

/// `storage.get` 的结果绑定到局部量撞 owned-Str 所有权规则（025 `.Init` 的
/// `var sp = storage.get("sysmon.speed")`）。
#[test]
fn p112_av_gap_storage_get_binding_needs_ownership_transfer() {
    let root = synthetic_root();
    let src = store_probe(
        "",
        "",
        "",
        "            var sp = storage.get(\"sysmon.speed\")\n            .label = sp\n",
        "",
    );
    expect_gap(
        "storage.get 绑定",
        &component_face(&root, &src),
        &["verify.ownership-required"],
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// 记录列表 model 字段（025 `var procs []ProcInfo = []`）不在组件切片内。
/// 裸源直送：prep 对 `[]Rec` 的改写会先撞内部诊断，裸源给出稳定的切片边界。
#[test]
fn p112_av_gap_component_slice_rejects_list_of_record_model_field() {
    let src = store_probe(
        "",
        "        var segs []Rec = []\n",
        "type Rec {\n    pid int\n}\n",
        "            .n = .segs.len()\n",
        "",
    );
    expect_gap(
        "记录列表 model 字段",
        &NodeDesktopBridge::load(&src, "p112-probe.at", 1)
            .err()
            .unwrap_or_default(),
        &["the component slice supports `int`/`i32`/`bool`/`str`/`f32` model fields"],
    );
}

/// `.as(float)` 转换与 `nil` 比较被 store handler 文法拒绝（025 `.Refresh` 的
/// `totalG_int.as(float)`、`snap == nil`）。
#[test]
fn p112_av_gap_float_cast_and_nil_compare_are_rejected_grammar() {
    let root = synthetic_root();
    let cast = store_probe(
        "",
        "",
        "",
        "            var f float = .n.as(float)\n            .label = \"x\"\n",
        "",
    );
    expect_gap(
        ".as(float)",
        &component_face(&root, &cast),
        &["source.syntax", "expected `fn`, found `store`"],
    );
    let nil_cmp = store_probe(
        "",
        "",
        "",
        "            var snap str = \"\"\n            if snap == nil {\n                .label = \"nil\"\n            \
         } else {\n                .label = \"ok\"\n            }\n",
        "",
    );
    expect_gap(
        "nil 比较",
        &component_face(&root, &nil_cmp),
        &["source.syntax", "expected `fn`, found `store`"],
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// store handler 调后端 API（`use back.api`）无解析面——S1 宿主 API 面缺席
/// （025 `system_snapshot()` / `kill_process()`）。
#[test]
fn p112_av_gap_store_cannot_call_backend_api() {
    let root = synthetic_root();
    let src = store_probe(
        "use back.api: ping\n",
        "",
        "",
        "            let snap = ping()\n            .label = snap\n",
        "",
    );
    expect_gap(
        "store 调后端 API",
        &component_face(&root, &src),
        &["no function named `ping` in this module"],
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// 局部字符串重赋值 `str`↔owned `Str` 类型不匹配；带计算初值的局部 `var` 不在
/// profile 内（025 `.Refresh`/`.ApplySort` 的路径拼串与 `var xf float = 36.0 +
/// ic * stepf`）。
#[test]
fn p112_av_gap_local_string_reassign_and_computed_var_initializers() {
    let root = synthetic_root();
    let concat = store_probe(
        "",
        "",
        "",
        "            var pc str = \"M\"\n            pc = pc + \" L 1 2\"\n            .label = pc\n",
        "",
    );
    expect_gap(
        "局部字符串重赋值",
        &component_face(&root, &concat),
        &["source.type-mismatch: expected str, found Str"],
    );
    let computed = store_probe(
        "",
        "        var cpu float = 0.0\n",
        "",
        "            var yf float = 120.0 - .cpu / 100.0 * 106.0\n            \
         if yf < 14.0 { yf = 14.0 }\n            .n = 1\n",
        "",
    );
    // 同一构造的两种失败签名（随 prep 改写落点而异）：局部 `var` 被 profile
    // 拒，或改写后 float 局部落进 int 槽的 type-mismatch。
    let diag = component_face(&root, &computed);
    assert!(
        diag.contains("`var` is not part of this source profile")
            || diag.contains("source.type-mismatch"),
        "计算初值局部 var: 首阻断应为 profile 拒或 type-mismatch，实际: {diag}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// 记录字面量局部量（025 `filled.push(ProcInfo { pid: raw.pid, … })`）：同单元
/// 声明的记录类型在 handler 内不可见。
#[test]
fn p112_av_gap_record_literal_local_has_no_record_type() {
    let root = synthetic_root();
    let src = store_probe(
        "",
        "",
        "type Rec {\n    pid int\n}\n",
        "            var r = Rec { pid: 2 }\n            .n = r.pid\n",
        "",
    );
    expect_gap(
        "记录字面量局部量",
        &component_face(&root, &src),
        &["no record type named `Rec` in this module"],
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ── 缺口钉点（AGE 包编译）─────────────────────────────────────────────────

/// 内建方法不在受检 profile（`fmt.at` 的 `n1`/`rate`：`f32.to_string()`、`str.sub`）。
#[test]
fn p112_av_gap_profile_lacks_string_and_number_builtins() {
    let src = r#"
fn n1(x float) str {
    let s str = x.to_string()
    return s.sub(0, 2)
}
"#;
    expect_gap(
        "内建方法 profile",
        &package_diag(&[unit("fmt", src)], "app::fmt"),
        &[
            "no builtin method `to_string` on receiver type f32 in this profile",
            "no builtin method `sub` on receiver type str in this profile",
        ],
    );
}

/// 可选字段 `?T` 不被包面类型声明接纳（`types.at` 的 `summary ?SysSummary`）。
#[test]
fn p112_av_gap_package_types_reject_optional_field() {
    let src = r#"
type S {
    cpu float
}

type Snapshot {
    summary ?S
}
"#;
    expect_gap(
        "可选字段",
        &package_diag(&[unit("types", src)], "app::types"),
        &["field type annotation (`int`/`i32`/`bool` or a record type) is required"],
    );
}

/// 真实 `SysStore` 在包面：model 可解析（只差消费者），**handler 体**被文法
/// 拒绝——缺口在 `on { … }` 的语言构造，不在 store 声明本身。
#[test]
fn p112_av_gap_real_store_handlers_rejected_by_package_grammar() {
    let raw = read("src/front/sys_store.at");
    let (head, rest) = raw.split_once("    model {").expect("model block");
    let (model, rest) = rest.split_once("    msg {").expect("msg block");
    let (_, on) = rest.split_once("    on {").expect("on block");
    let head = format!("{head}    model {{");
    let msg_trivial = "    msg {\n        Go\n    }\n\n";

    // 真 model + 平凡 handler：只剩「无消费者」——model 被接受。
    let mut trivial_arms = String::new();
    for m in [
        "Init", "Refresh", "Play", "Pause", "SetSpeed", "SelectTab", "SelectPerfTab", "SelectProc",
        "RowCtx", "CloseCtx", "RequestKill", "CancelKill", "ConfirmKill", "Sort", "ApplySort",
        "SortName", "SortCpu", "SortMem", "SortDisk", "SortNet", "SortPid",
    ] {
        trivial_arms.push_str(&format!("        .{m} -> {{ .cpu = 0.0 }}\n"));
    }
    let real_model = format!(
        "{head}{model}{msg_trivial}    on {{\n{trivial_arms}    }}\n}}\n"
    );
    let diag = package_diag(&[unit("sys_store", &real_model)], "app::sys_store");
    assert!(
        !diag.contains("source.syntax"),
        "真实 model 应被包面接受（只差消费者），实际: {diag}"
    );

    // 平凡 model + 真 handler：文法拒绝。
    let real_on = format!(
        "{head}\n        var cpu float = 0.0\n    }}\n\n{msg_trivial}    on {{{on}"
    );
    expect_gap(
        "真实 SysStore handler（包面）",
        &package_diag(&[unit("sys_store", &real_on)], "app::sys_store"),
        &["source.syntax", "expected `fn`, found `store`"],
    );
}

// ── 真实源钉点 ─────────────────────────────────────────────────────────────

/// AV 组件面：widget 引用 `.store.*` 而源内无 store 声明 → S1 未接线。
#[test]
fn p112_av_gap_component_face_cross_unit_store_not_wired() {
    let widget = "widget Main {\n    view {\n        text .store.n { }\n    }\n}\n";
    expect_gap(
        "跨单元 store（组件面）",
        &NodeDesktopBridge::load(widget, "p112-probe.at", 1)
            .err()
            .unwrap_or_default(),
        &["cross-unit store loading is not wired yet (S1)"],
    );
}

/// 025 `app.at` 经 prep 投影后首阻断即 S1（prep 未把独立 store 单元内联进来）。
#[test]
fn p112_av_real_app_first_blocker_on_component_face() {
    let src = read("src/front/app.at");
    expect_gap(
        "025 app.at（组件面）",
        &component_face(&app_dir(), &src),
        &["cross-unit store loading is not wired yet (S1)"],
    );
}

/// AV 包面 / A2Vue 包面：025 六单元原样装配首阻断 = 消费者找不到 store 字段
/// （store 单元自身的 handler 文法被拒，见上）。
#[test]
fn p112_av_real_app_first_blocker_on_package_faces() {
    let units: Vec<PackageModuleInput> = [
        ("main", "src/front/app.at"),
        ("sys_store", "src/front/sys_store.at"),
        ("processes", "src/front/processes.at"),
        ("performance", "src/front/performance.at"),
        ("fmt", "src/front/fmt.at"),
        ("types", "src/front/types.at"),
    ]
    .into_iter()
    .map(|(id, rel)| unit(id, &read(rel)))
    .collect();
    expect_gap(
        "025 六单元（AV 包面）",
        &NodeDesktopBridge::load_package(units.clone(), "app::main", 1)
            .err()
            .unwrap_or_default(),
        &["model has no field `store`"],
    );
    expect_gap(
        "025 六单元（A2Vue 包面）",
        &generate_package_sfc(&units, "app::main").err().unwrap_or_default(),
        &["model has no field `store`"],
    );
}
