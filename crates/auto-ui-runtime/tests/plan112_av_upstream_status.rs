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
//! - `p112_av_control_*`：今天就被接受的对照——缺口是具体构造，不是「store
//!   语义整体不存在」。包面上真实 `SysStore` 的 21 个 handler 臂里有 20 个
//!   语法通过；仅去掉 4 处（2 个 `nil` 比较、2 个 `.as(float)`）后整个 store
//!   解析通过。
//! - `p112_av_gap_*`：每个缺口一个最小源 + 首阻断诊断；包面的门序见
//!   `p112_av_gap_package_store_gates_in_order`。
//! - `p112_av_real_*`：真实 025 单元的首阻断。
//!
//! 上游（auto-lang AGE）落地某项后，对应 `expect_gap` 会因「编译通过」或
//! 「诊断改名」翻红——届时把该条改为结构断言，并恢复 PLAN-112 的 T-03。
//! 构造级探针一律走生产路径（prep → load；prep 拒绝时退回裸源）——裸源直送
//! 会把 prep 已仿真的构造（字段显示、`storage.set`、`while`）误报成缺口。
//! 探针拼装务必从显式片段构造：把 `raw[..model_start]` 当「头部」会把
//! `store SysStore {` 行一并带入，拼出重复 store 头而得出「全红」的假象。

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

/// 探针根目录（空）。Drop 时删除——钉点翻红、断言失败时也不留临时目录。
struct ProbeRoot(PathBuf);

impl ProbeRoot {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("p112-age-probe-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        Self(dir)
    }
}

impl std::ops::Deref for ProbeRoot {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for ProbeRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn synthetic_root() -> ProbeRoot {
    ProbeRoot::new()
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

/// 真实 `SysStore` 的包面源：`(store 头之前的内容, store 声明本体)`。`use` 行等长
/// 空白化——包面没有 `back.api` / `fmt` 模块，不让「未解析导入」抢在语法之前。
fn real_store_source() -> (String, String) {
    let raw = read("src/front/sys_store.at");
    let store_hdr = raw.find("store SysStore {").expect("store header");
    let head: String = raw[..store_hdr]
        .lines()
        .map(|l| {
            if l.trim_start().starts_with("use ") {
                " ".repeat(l.len())
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    (head, raw[store_hdr..].to_string())
}

/// 把真实 store 的 `on { … }` 切成 `(臂名, 臂文本)`（声明序）。
fn real_store_arms(store: &str) -> Vec<(String, String)> {
    let on_open = store.find("    on {").expect("on");
    let store_close = store.rfind('}').expect("store close");
    let on_tail = &store[on_open + "    on {".len()..store_close];
    let on_close = on_tail.rfind('}').expect("on close");
    let mut arms = Vec::new();
    let mut cur: Option<(String, String)> = None;
    for line in on_tail[..on_close].split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with('.') && t.contains("->") {
            let name = t[1..].split("->").next().unwrap_or("").trim().to_string();
            if let Some((n, b)) = cur.take() {
                arms.push((n, b));
            }
            cur = Some((name, String::new()));
        }
        if let Some((_, body)) = cur.as_mut() {
            body.push_str(line);
        }
    }
    if let Some((n, b)) = cur.take() {
        arms.push((n, b));
    }
    arms
}

/// 用给定臂重装 store：`real` 中的臂取真实体（经 `neutral`），其余打桩。
fn assemble_store(store: &str, real: &[&str], neutral: impl Fn(&str) -> String) -> String {
    let on_open = store.find("    on {").expect("on");
    // 中和 `storage.set` 会写入 `.__noop`——在 model 里声明它，免得抢在目标门前报错。
    let pre = store[..on_open].replacen(
        "    model {\n",
        "    model {\n        var __noop str = \"\"\n",
        1,
    );
    let arms: String = real_store_arms(store)
        .iter()
        .map(|(n, b)| {
            let stem = n.split('(').next().unwrap_or(n);
            if real.contains(&stem) {
                neutral(b)
            } else {
                format!("        .{n} -> {{ .cpu = 0.0 }}\n")
            }
        })
        .collect();
    format!("{pre}    on {{\n{arms}    }}\n}}\n")
}

/// 探针中和（只为揭示下一道门，**不是实现**）：`nil` 比较 → `false`；`.as(float)`
/// 去掉；`storage.get(k)` → `""`；`storage.set(k, v)` → `.__noop = v`。
fn neutralize_syntax(body: &str) -> String {
    body.replace("snap == nil", "false")
        .replace("snap.summary == nil", "false")
        .replace(".as(float)", "")
}

fn neutralize_storage(body: &str) -> String {
    let mut out = neutralize_syntax(body);
    for key in [
        "sysmon.speed",
        "dash.speed",
        "sysmon.sort_column",
        "dash.sort_column",
        "sysmon.sort_dir",
        "dash.sort_dir",
        "sysmon.active_tab",
        "sysmon.perf_tab",
    ] {
        out = out.replace(&format!("storage.get(\"{key}\")"), "\"\"");
    }
    while let Some(at) = out.find("storage.set(\"") {
        let close = out[at..].find('\n').map(|n| at + n).unwrap_or(out.len());
        let line = out[at..close].to_string();
        let value = line
            .trim_start_matches("storage.set(")
            .trim_end_matches(')')
            .splitn(2, ',')
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();
        out.replace_range(at..close, &format!(".__noop = {value}"));
    }
    out
}

const SYS_STORE_ALL_ARMS: &[&str] = &[
    "Init", "Play", "Pause", "SetSpeed", "SelectTab", "SelectPerfTab", "SelectProc", "RowCtx",
    "CloseCtx", "RequestKill", "CancelKill", "ConfirmKill", "SortName", "SortCpu", "SortMem",
    "SortDisk", "SortNet", "SortPid", "Sort", "ApplySort", "Refresh",
];

/// 带最小消费者的包编译首诊断（store 单元 id `sys_store`）。
fn package_store_diag(store_src: &str) -> String {
    let consumer = "use sys_store: SysStore\n\nwidget Main {\n    view {\n        text .store.cpu {}\n        \
                    button \"go\" { onclick: .Go }\n    }\n    on {\n        .Go -> { store.Init() }\n    }\n}\n";
    package_diag(
        &[unit("main", consumer), unit("sys_store", store_src)],
        "app::main",
    )
}

/// 包面文法只被真实 `SysStore` 里的两类构造触发：`nil` 比较（`.Refresh` 守卫）与
/// `.as(float)` 转换。`||`、比较、嵌套 if/else、`while`、列表索引、`storage.*`、
/// 记录列表上的选择排序、f-string、记录字面量 push 都能解析——此前「handler 体被
/// 文法拒」的归因过宽。
#[test]
fn p112_av_gap_package_grammar_rejects_nil_and_float_cast() {
    let probe = |stmt: &str| {
        let src = format!(
            "store S {{\n    model {{\n        var cpu float = 0.0\n        var label str = \"\"\n        var n int = 0\n    }}\n\n    \
             msg {{ Go }}\n\n    on {{\n        .Go -> {{\n            {stmt}\n        }}\n    }}\n}}\n"
        );
        package_diag(&[unit("s", &src)], "app::s")
    };
    for (label, stmt) in [
        ("nil 比较", "if .cpu == nil { .cpu = 0.0 }"),
        ("成员 nil 比较", "if x.y == nil { .cpu = 0.0 }"),
        (".as(float)", "var f float = .n.as(float) / 10.0"),
    ] {
        expect_gap(label, &probe(stmt), &["source.syntax", "expected `fn`, found `store`"]);
    }
    // 对照：去掉这两个构造的同形语句可解析。
    for (label, stmt) in [
        ("bool 比较", "if .cpu < 0.0 || .label == \"\" { .cpu = 0.0 }"),
        ("整型除以浮点", "var f float = .n / 10.0"),
    ] {
        // 单独的 store 单元没有消费者：语法通过的终点就是「无消费者」这条后续诊断。
        let diag = probe(stmt);
        assert!(
            !diag.contains("source.syntax") && diag.contains("no checked widget consumer"),
            "{label}: 应通过语法，实际: {diag}"
        );
    }
}

/// 对照：真实 `SysStore` 只去掉 4 处（2 个 `nil` 比较、2 个 `.as(float)`）后，整个
/// store（真实 model + 全部 21 个真实臂）在包面解析通过——停在「无消费者」。
#[test]
fn p112_av_control_real_store_parses_without_nil_and_float_cast() {
    let (head, store) = real_store_source();
    let src = format!(
        "{head}\n{}",
        assemble_store(&store, SYS_STORE_ALL_ARMS, neutralize_syntax)
    );
    let diag = package_diag(&[unit("sys_store", &src)], "app::sys_store");
    assert!(
        !diag.contains("source.syntax"),
        "真实 SysStore 去掉 nil/.as(float) 后应解析通过，实际: {diag}"
    );
    assert_eq!(
        real_store_arms(&store).len(),
        21,
        "025 SysStore 的 handler 臂数（源变更时同步本钉点）"
    );
}

/// 包面的门序（入口 = 最小消费者；每道门只报首错，逐层中和只为露出下一道）：
/// ① `storage.*` 需所属宿主（`.Init`）→ ② handler 调用词汇：后端 API 调用
/// （`ConfirmKill` 的 `kill_process`）不在允许集内 → ③ store model 引用的记录类型
/// （`ProcInfo`）不是可见的模块符号（在 `types.at`，而 `sys_store.at` 未导入）。
#[test]
fn p112_av_gap_package_store_gates_in_order() {
    let (head, store) = real_store_source();
    let without = |excluded: &[&str], neutral: fn(&str) -> String| {
        let kept: Vec<&str> = SYS_STORE_ALL_ARMS
            .iter()
            .copied()
            .filter(|arm| !excluded.contains(arm))
            .collect();
        format!(
            "{head}\n{}",
            assemble_store(&store, &kept, neutral)
        )
    };
    // ① 全部真实臂，仅中和语法。
    expect_gap(
        "门① storage",
        &package_store_diag(&without(&[], neutralize_syntax)),
        &["storage calls require the owning host"],
    );
    // ② 再中和 storage。
    expect_gap(
        "门② handler 调用词汇",
        &package_store_diag(&without(&[], neutralize_storage)),
        &[
            "handler calls require a declared widget message, same-unit store method, or \
             checked list operation",
        ],
    );
    // ③ 去掉含后端 API 调用的两个臂（`ConfirmKill`、`Refresh`）。
    expect_gap(
        "门③ 记录类型可见性",
        &package_store_diag(&without(&["ConfirmKill", "Refresh"], neutralize_storage)),
        &["source.module-symbol", "unknown type `ProcInfo`"],
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
/// （store 单元因 `nil` 比较 / `.as(float)` 语法红而出不了 store，见上）。
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
