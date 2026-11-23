//! PLAN-112 T-02/T-03（AV 腿上游状态钉点）。
//!
//! 025 `SysStore` 是实后端 store：记录列表 model 字段、`storage.*`、`use
//! back.api` 调用、`fmt.at` helper、`.as(float)`/`nil`、滑窗与排序循环。AV
//! （受检 AGE）两端——组件面 `prepare_component_source` →
//! `NodeDesktopBridge::load`，与包面 `load_package`（A2Vue 包面
//! `generate_package_sfc` 同一 AGE 包编译）——在 PLAN-117 合入后的 master 上
//! 仍无法装载原样 025。
//!
//! 本文件是**状态钉点**而非功能用例（先例：`plan105_u03_sfc_status`）：
//! - `p112_av_control_*`：今天就被接受的对照——缺口是具体构造，不是「store
//!   语义整体不存在」。原样真实 `SysStore`（21 个 handler 臂）在包面解析通过
//!   （nil 字面量由 PLAN-117 开到两面；`.as(T)` 由 PLAN-112 开到包面，组件面仍拒）。
//!   PLAN-117 还关掉了旧的 handler 调用词汇门，并在组件面放行了 nil 比较与
//!   记录字面量局部量——这些原是缺口，现为对照。
//! - `p112_av_gap_*`：每个缺口一个最小源 + 首阻断诊断；包面的门序见
//!   `p112_av_gap_package_store_gates_in_order`（钉的是**裸 AGE 包编译**）。过语法
//!   不等于可消费：包面的 `.as(T)` / `.to(int)` 停在 Atom bind（TextRecord 描述符
//!   无 `expr.cast`）。
//! - 宿主行为与语言行为分开钉：AV 包面生产路径（`load_package`）对纯 store 单元做
//!   `storage` 宿主服务仿真，门① 在该路径不再出现；门序用例里的门① 仍对裸 AGE 成立。
//! - `p112_av_real_*`：真实 025 单元的首阻断。
//!
//! 某项缺口关闭后，对应 `expect_gap` 会因「编译通过」或「诊断改名」翻红——届时
//! 把该条改为结构断言（`p112_av_control_*`）。全部关闭后恢复 PLAN-112 的 T-03。
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
    component_face_at(root, src, "p112-probe.at")
}

/// 同上，`path` = 生产入口传入的源路径（`--source <app.at>` 的语义）。
fn component_face_at(root: &Path, src: &str, path: &str) -> String {
    let prepped = prepare_component_source(root, src)
        .map(|p| p.source)
        .unwrap_or_else(|_| src.to_string());
    NodeDesktopBridge::load(&prepped, path, 1)
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

/// `.as(float)` 转换被**组件面**的 store handler 文法拒绝（025 `.Refresh` 的
/// `totalG_int.as(float)`）；包面已收，见
/// `p112_av_control_package_grammar_admits_nil_and_float_cast`。
#[test]
fn p112_av_gap_float_cast_is_rejected_grammar() {
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
}

/// 对照：`nil` 比较在组件面经 prep → load 可装载（PLAN-117 把 nil 字面量开到
/// AGE 两面，含 `if` 条件位）。025 `.Refresh` 的守卫 `snap == nil` 不再是语法红。
#[test]
fn p112_av_control_component_face_admits_nil_compare() {
    let root = synthetic_root();
    let nil_cmp = store_probe(
        "",
        "",
        "",
        "            var snap str = \"\"\n            if snap == nil {\n                .label = \"nil\"\n            } \
             else {\n                .label = \"ok\"\n            }\n",
        "",
    );
    expect_accepted("组件面 nil 比较", &component_face(&root, &nil_cmp));
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

/// 局部字符串重赋值：handler 进到验证器后停在 `verify.borrow-escape`——重赋值
/// 存了一个临时量的视图（025 `.Refresh`/`.ApplySort` 的路径拼串）。带计算初值的
/// 局部 `var`：改写后 float 局部落进 int 槽的 type-mismatch（`var xf float =
/// 36.0 + ic * stepf`）。
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
        &["verify.borrow-escape: assignment to `l2` stores a view of a temporary"],
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

/// 对照：记录字面量局部量（025 `filled.push(ProcInfo { pid: raw.pid, … })`）在组件
/// 面可装载——PLAN-117 给受检 AGE 组件表达式加了命名字段记录构造。
#[test]
fn p112_av_control_record_literal_local_loads() {
    let root = synthetic_root();
    let src = store_probe(
        "",
        "",
        "type Rec {\n    pid int\n}\n",
        "            var r = Rec { pid: 2 }\n            .n = r.pid\n",
        "",
    );
    expect_accepted("记录字面量局部量", &component_face(&root, &src));
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

/// 探针中和（只为揭示下一道门，**不是实现**）：`storage.get(k)` → `""`；
/// `storage.set(k, v)` → `.__noop = v`。
fn neutralize_storage(body: &str) -> String {
    let mut out = body.to_string();
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

/// 探针中和（只为揭示后面的门，**不是实现**）：无类型空列表字段 `cpu_segs` /
/// `mem_segs` 改成 `List<int>`——受检 AGE 不把 `var x = []` 当状态。
fn typed_segments(store_src: &str) -> String {
    store_src
        .replace("var cpu_segs = []", "var cpu_segs List<int> = []")
        .replace("var mem_segs = []", "var mem_segs List<int> = []")
}

/// 探针中和：删掉往段字段 push 记录字面量的语句（字段已改成 `List<int>`）。
fn neutralize_segments(body: &str) -> String {
    let mut out = String::new();
    for line in body.split_inclusive('\n') {
        if line.contains(".cpu_segs.push(") || line.contains(".mem_segs.push(") {
            continue;
        }
        out.push_str(line);
    }
    out
}

/// 带最小消费者的包编译首诊断（store 单元 id `sys_store`）。
fn package_store_diag(store_src: &str) -> String {
    let consumer = "use sys_store: SysStore\n\nwidget Main {\n    view {\n        text .store.cpu {}\n        \
                    button \"go\" { onclick: .Go }\n    }\n    on {\n        .Go -> { store.Init() }\n    }\n}\n";
    package_diag(
        &[unit("main", consumer), unit("sys_store", store_src)],
        "app::main",
    )
}

/// 单独 store 单元（无消费者）的包编译诊断：语法通过的终点是「无消费者」。
fn lone_store_diag(store_src: &str) -> String {
    package_diag(&[unit("s", store_src)], "app::s")
}

/// `store S { model { cpu: float, label: str, n: int }, .Go -> <stmt> }`。
fn store_with_arm(stmt: &str) -> String {
    format!(
        "store S {{\n    model {{\n        var cpu float = 0.0\n        var label str = \"\"\n        var n int = 0\n    }}\n\n    \
         msg {{ Go }}\n\n    on {{\n        .Go -> {{\n            {stmt}\n        }}\n    }}\n}}\n"
    )
}

/// `use s: S` 消费者：视图行 `view` + 按钮调 `store.Go()`。
fn go_consumer(view: &str) -> String {
    format!(
        "use s: S\n\nwidget Main {{\n    msg {{ Go }}\n    view {{\n{view}        \
         button \"go\" {{ onclick: .Go }}\n    }}\n    on {{\n        .Go -> {{ store.Go() }}\n    }}\n}}\n"
    )
}

/// AGE **包面**文法已收 `nil` 字面量与 `.as(T)` 数值转换（PLAN-112，auto-lang
/// `auto-source.at`）。此前这两类是真实 `SysStore` 仅有的语法红——`||`、比较、嵌套
/// if/else、`while`、列表索引、`storage.*`、记录列表上的选择排序、f-string、记录
/// 字面量 push 本就能解析。组件面两者仍拒（见 `…_float_cast_and_nil_compare_…`）。
#[test]
fn p112_av_control_package_grammar_admits_nil_and_float_cast() {
    for (label, stmt) in [
        ("nil 比较", "if .cpu == nil { .cpu = 0.0 }"),
        ("成员 nil 比较", "if x.y == nil { .cpu = 0.0 }"),
        ("nil || nil", "if x == nil || y == nil { .cpu = 0.0 }"),
        (".as(float)", "var f float = .n.as(float) / 10.0"),
        (".as(int)", "var i int = .cpu.as(int)"),
        (".as(f32)", "var f float = .n.as(f32) / 10.0"),
    ] {
        let diag = lone_store_diag(&store_with_arm(stmt));
        assert!(
            !diag.contains("source.syntax") && diag.contains("no checked widget consumer"),
            "{label}: 应通过包面语法，实际: {diag}"
        );
    }
    // `nil` 与非可空字段比较在包面 store handler 这条路径上编译通过（PLAN-117
    // 之后不再报 `source.nil-op`；运行期真值不在本例断言）。真实 `SysStore` 的
    // nil 比较落在后端调用返回值与可选记录字段上。
    let store = store_with_arm("if .cpu == nil {\n                .cpu = 0.0\n            }");
    let diag = package_diag(
        &[unit("main", &go_consumer("        text .store.n { }\n")), unit("s", &store)],
        "app::main",
    );
    assert!(diag.is_empty(), "nil 比较应随消费者整体编译通过，实际: {diag}");
}

/// 缺口：包面数值转换（`.as(T)` 与既有拼写 `.to(int)`）过语法、过 resolve，停在
/// Atom bind——AGE 两面的 HIR 域是 TextRecord，其描述符没有 `expr.cast` 分支
/// （numeric / aggregates / auto-language 描述符有）。用户书写的转换以内部错误
/// （`source.internal`）浮出，而非类型化的 `source.unsupported`。
#[test]
fn p112_av_gap_package_numeric_cast_fails_at_atom_bind() {
    for (label, stmt) in [
        (".as(float)", "var t float = .n.as(float) / 10.0\n            .cpu = t"),
        (".as(int)", "var i int = .cpu.as(int)"),
        (".to(int)", "var i int = .cpu.to(int)"),
    ] {
        let diag = package_diag(
            &[
                unit("main", &go_consumer("        text .store.n { }\n")),
                unit("s", &store_with_arm(stmt)),
            ],
            "app::main",
        );
        expect_gap(
            label,
            &diag,
            &[
                "generated Atom text failed to bind",
                "no accepted case `expr.cast` in descriptor",
            ],
        );
    }
}

/// 缺口：包面视图显示 float 状态字段——字段读取处的隐式 `clone` 在本 profile 没有
/// `f32` 实现。与 `.as(T)` 无关：同一消费者显示 int 字段通过（见上面的接受对照）。
#[test]
fn p112_av_gap_package_float_state_display_needs_clone() {
    let diag = package_diag(
        &[
            unit("main", &go_consumer("        text .store.cpu { }\n")),
            unit("s", &store_with_arm(".cpu = .cpu + 1.0")),
        ],
        "app::main",
    );
    expect_gap(
        "包面显示 float 状态字段",
        &diag,
        &["numeric receiver has no method `clone` in this profile"],
    );
}

/// 对照：原样的真实 `SysStore`（真实 model + 全部 21 个真实臂，零中和）在包面解析
/// 通过——停在「无消费者」。
#[test]
fn p112_av_control_real_store_parses_on_package_face() {
    let (head, store) = real_store_source();
    let diag = lone_store_diag(&format!("{head}\n{store}"));
    assert!(
        !diag.contains("source.syntax"),
        "原样 SysStore 应通过包面语法，实际: {diag}"
    );
    assert_eq!(
        real_store_arms(&store).len(),
        21,
        "025 SysStore 的 handler 臂数（源变更时同步本钉点）"
    );
}

/// 包面的门序（入口 = 最小消费者；每道门只报首错，逐层中和只为露出下一道）。
/// PLAN-117 合入后重测：
/// - ⓪ model 形态：无类型空列表字段（`var cpu_segs = []`）不再是状态，handler
///   一写就报 `assigns undeclared model field`。
/// - ① `storage.*` 需所属宿主（`.SetSpeed` / `.SelectTab` / `.Sort`）。
/// - ③ store model 引用的记录类型（`ProcInfo`，声明在 `types.at`，`sys_store.at`
///   既未导入、类型也非 `pub`）对内联后的消费者不可见。
///
/// 旧门②（handler 调用词汇）已由 PLAN-117 关掉：`ConfirmKill` 的
/// `kill_process(.kill_target)` 语句不再在这一层被拒。
#[test]
fn p112_av_gap_package_store_gates_in_order() {
    let (head, store) = real_store_source();
    let identity = |body: &str| body.to_string();
    let all = SYS_STORE_ALL_ARMS;
    // ⓪ 全部真实臂，零中和：先撞 model 形态。
    expect_gap(
        "门⓪ 无类型空列表字段",
        &package_store_diag(&format!("{head}\n{}", assemble_store(&store, all, identity))),
        &["assigns undeclared model field `cpu_segs`"],
    );
    // 之后各门都以「段字段已带类型」为底：字段改成 `List<int>`，段 push 语句删掉
    // （只为露出后面的门，不是实现）。
    let typed = |neutral: fn(&str) -> String| {
        typed_segments(&assemble_store(&store, all, |body| neutralize_segments(&neutral(body))))
    };
    expect_gap(
        "门① storage",
        &package_store_diag(&format!("{head}\n{}", typed(identity))),
        &["storage calls require the owning host"],
    );
    expect_gap(
        "门③ 记录类型可见性",
        &package_store_diag(&format!("{head}\n{}", typed(neutralize_storage))),
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
/// 路径取真实 `app.at`（生产 `--source` 入口的语义）。
#[test]
fn p112_av_real_app_first_blocker_on_component_face() {
    let src = read("src/front/app.at");
    let path = app_dir().join("src/front/app.at");
    expect_gap(
        "025 app.at（组件面）",
        &component_face_at(&app_dir(), &src, &path.to_string_lossy()),
        &["cross-unit store loading is not wired yet (S1)"],
    );
}

/// AV 包面生产路径上的正对照：纯 store 单元里的 `storage.get/set` 经宿主服务仿真
/// （`ace_source_prep::simulate_store_host_services`）改写为 store 模型字段后，
/// 装载不再停在 AGE 的门①。
#[test]
fn p112_av_control_av_package_path_simulates_store_storage() {
    let store = store_with_arm(
        "            .label = storage.get(\"sysmon.speed\")\n            \
         storage.set(\"sysmon.speed\", .label)\n            .n = .n + 1",
    );
    let consumer = || unit("main", &go_consumer("        text .store.n { }\n"));
    // 同一源直送 AGE 包编译：门① 仍在（仿真是宿主行为，不是语言行为）。
    expect_gap(
        "裸 AGE 包编译（store 用 storage）",
        &package_diag(&[consumer(), unit("s", &store)], "app::main"),
        &["storage calls require the owning host"],
    );
    let diag = NodeDesktopBridge::load_package(
        vec![consumer(), unit("s", &store)],
        "app::main",
        1,
    )
    .err()
    .unwrap_or_default();
    assert!(
        !diag.contains("storage calls require the owning host") && !diag.contains("source.syntax"),
        "AV 包面装载不应再停在 storage 门，实际: {diag}"
    );
}

/// AV 包面 / A2Vue 包面：025 六单元**原样**装配（零中和）。
///
/// AV 包面（生产路径 `NodeDesktopBridge::load_package`）：`SysStore` 过语法之后，
/// 纯 store 单元里的 `storage.get/set` 由宿主服务仿真改写为 store 的模型字段
/// （PLAN-112，与组件面 prep 同一映射——AGE 的门① 在这条路径上不再出现，见
/// `p112_av_gap_package_store_gates_in_order` 对裸 AGE 的钉点），首阻断落在门⓪
/// model 形态：无类型空列表字段不是状态。A2Vue 包面（`generate_package_sfc`）
/// 不做仿真，且先走 routed 车道，车道对入口单元**单独**做组件面解析
/// （`package_candidates`）——`app.at` 单独解析时 `use sys_store: SysStore` 不可见，
/// 首阻断是 `model has no field \`store\``，轮不到包编译。
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
        &["assigns undeclared model field `cpu_segs`"],
    );
    let a2vue = generate_package_sfc(&units, "app::main")
        .err()
            .unwrap_or_default();
    expect_gap("025 六单元（A2Vue 包面）", &a2vue, &["model has no field `store`"]);
}
