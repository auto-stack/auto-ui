//! PLAN-980 ④-A 首片 #2c：**同源 SFC `<script>` 段**——包片段 lane 的 helper 发射
//! 与调用名映射。
//!
//! 输入与 ④-A #2a/#2b 同一派生真实例（`ace-node-counter` 派生 + 点分
//! `use util.steps: step`）。判据（用户 2026-11-18）：
//! ① handler 体里对被调 helper 的调用名 = **映射后的发射名**（不是裸 `step(`）；
//! ② 该发射名同时以**函数定义**落进同一份 `<script>` 段（调用方与被调方同产物）；
//! ③ 改 helper 源 → 函数体常量随之改变（映射不是文本占位）。
//!
//! 边界（诚实）：本片只产 `<script>` 段（handler 体 + 被引用 helper 定义）；完整
//! SFC 的 template/样式仍归既有方言道；包片段 lane 只覆盖 976 fragment 域。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_lane::PackageFragmentLane;

fn unit(pkg: &str, id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: pkg.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

/// 派生真实例：`Inc` 的 handler 体含跨模块 helper 调用 `step(.count)`。
fn main_src() -> String {
    "use util.steps: step\n\
use themes.themes: Auto, Solarized\n\
\n\
node Counter {\n\
    msg { Inc, Reset }\n\
\n\
    model {\n\
        var count int = step(0)\n\
    }\n\
\n\
    view {\n\
        col {\n\
            text \"Counter\"\n\
        }\n\
    }\n\
\n\
    on {\n\
        .Inc -> { .count = step(.count) }\n\
        .Reset -> { .count = 0 }\n\
    }\n\
}\n"
    .to_string()
}

const HELPER: &str = "pub fn step(n int) int {\n    return n + 2\n}\n";
const HELPER_EDITED: &str = "pub fn step(n int) int {\n    return n + 10\n}\n";

/// 是否存在**裸**调用 `name(`（`name` 不是更长标识符的后缀）。
fn has_bare_call(text: &str, name: &str) -> bool {
    let needle = format!("{name}(");
    let mut from = 0;
    while let Some(i) = text[from..].find(&needle) {
        let at = from + i;
        let prev = text[..at].chars().last();
        if !prev.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            return true;
        }
        from = at + needle.len();
    }
    false
}

/// 取 handler 的**被调 helper 发射名**：fragment 的 helpers 列即发射侧权威名；
/// 并交叉核对它确实由包视图里的某个函数映射而来（不是凭空文本）。
fn called_helper(lane: &PackageFragmentLane, helpers: &[String]) -> String {
    let name = helpers
        .first()
        .expect("handler fragment must declare the called helper")
        .clone();
    let view_names = lane.function_names().expect("view names");
    assert!(
        view_names.iter().any(|n| name.ends_with(n.as_str())),
        "emitted helper {name} must map from a package function: {view_names:?}"
    );
    name
}

/// 组装 `<script>` 段：helper 定义（参数名取发射侧局部表）+ handler 体。
fn script_section(lane: &PackageFragmentLane, handler_text: &[String]) -> (String, String) {
    let span = lane.find_span("step(.count)").expect("handler RHS span");
    let handler = lane
        .emit_expr_at_span("__age_state_0_count", span)
        .expect("handler fragment");
    let helper_name = called_helper(lane, &handler.helpers);
    let helper_body = lane
        .emit_function_body("_steps_step")
        .expect("helper body emitted from the same package view");
    let params: Vec<String> = helper_body
        .locals
        .iter()
        .filter(|l| l.role == "param")
        .map(|l| l.emitted_name.clone())
        .collect();
    let mut script = String::new();
    script.push_str(&format!(
        "// emitted by the AGE package lane (same checked module)\nfunction {helper_name}({}) {{\n",
        params.join(", ")
    ));
    for line in &helper_body.text {
        script.push_str("  ");
        script.push_str(line);
        script.push('\n');
    }
    script.push_str("}\n\nfunction __handler_Inc() {\n");
    for line in handler_text {
        script.push_str("  ");
        script.push_str(line);
        script.push('\n');
    }
    script.push_str("}\n");
    (script, helper_name)
}

/// ①调用名=映射名（非裸 `step(`）②同一映射名以函数定义落在同一 `<script>` 段。
#[test]
fn p980_sfc_script_emits_helper_and_mapped_call() {
    let lane = PackageFragmentLane::compile(
        &[
            unit("app", "main", &main_src()),
            unit("util", "steps", HELPER),
        ],
        "app::main",
        &["Inc"],
    )
    .expect("package lane compiles");
    let span = lane.find_span("step(.count)").expect("span");
    let handler = lane
        .emit_expr_at_span("__age_state_0_count", span)
        .expect("handler emitted");
    let handler_text = handler.text.join("\n");
    let helper_name = called_helper(&lane, &handler.helpers);

    // ①调用名是映射后的发射名，且不是裸 `step(`。
    assert!(
        handler_text.contains(&format!("{helper_name}(")),
        "handler calls the mapped helper: {handler_text}"
    );
    assert!(
        !has_bare_call(&handler_text, "step"),
        "bare `step(` must not survive emission: {handler_text}"
    );
    assert!(
        handler.helpers.contains(&helper_name),
        "the mapped helper is declared required: {:?}",
        handler.helpers
    );

    // ②同一份产物：helper 以函数定义出现，参数名来自发射侧局部表。
    let (script, mapped) = script_section(&lane, &handler.text);
    assert_eq!(mapped, helper_name);
    assert!(
        script.contains(&format!("function {helper_name}(")),
        "helper definition present in the same script section: {script}"
    );
    assert!(
        script.contains('2'),
        "helper body carries the source constant: {script}"
    );
}

/// ③改 helper 源 → 同一脚本段的函数体常量随之改变（映射不是文本占位）。
#[test]
fn p980_sfc_script_follows_the_helper_source() {
    let lane = PackageFragmentLane::compile(
        &[
            unit("app", "main", &main_src()),
            unit("util", "steps", HELPER_EDITED),
        ],
        "app::main",
        &["Inc"],
    )
    .expect("package lane compiles");
    let span = lane.find_span("step(.count)").expect("span");
    let handler = lane
        .emit_expr_at_span("__age_state_0_count", span)
        .expect("handler emitted");
    let (script, _) = script_section(&lane, &handler.text);
    assert!(
        script.contains("10"),
        "edited helper constant flows into the script section: {script}"
    );
}
