//! PLAN-980 WEB 首切片永久 case：976 TS/JS provider 的 **UI fragment 消费**。
//!
//! 链路（`ui/ace_ts_provider.rs`）：UI 表达式/语句源文本 → 包装函数 →
//! ACE text-record 前门（共享 verify）→ `SemanticView` → 976
//! `emit_{expr,stmt}_fragment`（HIR 真值、`a2x_*` helper 义务）→ **宿主映射**
//! （发射名→源名→reactive 宿主名，如 `count`→`count.value`——976 合同的反应
//! 性/宿主变量职责）→ 真 node 执行（FRAG_OK 协议）。
//!
//! 边界：本切片证明生产表达式形态（绑定表达式/赋值语句）经新 provider 可
//! 发射可执行且宿主映射成立；完整适配器切换（ts_adapter 四处旧
//! TypeScriptTrans 委托 + 浏览器挂载档）为 WEB 批后续步（计划 §9 收据）。
//! 新车道**无旧 emitter 回退**：未覆盖形态=具名拒绝。

#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_ts_provider::{
    emit_ui_expr_fragment, emit_ui_stmt_fragment, run_fragment_on_node,
};

/// 绑定表达式形态（002/013 族）：`count + 1` 经新 provider 发射；宿主映射
/// `count` → `count.value`（reactive 职责在适配器）；`a2x_add` helper 义务
/// 报告；真 node 执行 41+1=42。
#[test]
fn p980_web_expr_fragment_binds_and_maps_reactive_names() {
    let body = "return count + 1";
    let frag = emit_ui_expr_fragment(
        body,
        "count + 1",
        &["count"],
        &[("count".into(), "count.value".into())],
    )
    .expect("binding-shape expression fragment emits");
    // checked-add 模式（tsfrag 同形）：操作数住 helper 行（宿主映射后为
    // count.value），表达式本体是临时引用（t0）。
    let joined = frag.lines.join(
        "
",
    );
    assert!(
        joined.contains("count.value"),
        "reactive host name substituted in helper line: {joined}"
    );
    assert!(
        !joined.contains("count.value.value"),
        "no double substitution: {joined}"
    );
    assert!(
        frag.helpers.iter().any(|h| h == "a2x_add"),
        "helper obligations reported: {:?}",
        frag.helpers
    );
    let expr = frag.expr.as_deref().expect("expression body present");
    // 真 node：预声明宿主变量（映射后命名），导入报告的 helper，断言临时值。
    let out = run_fragment_on_node(
        &frag,
        "const count = { value: 41 };",
        &format!("({expr}) === 42"),
    )
    .expect("node runs the fragment");
    assert!(out.contains("FRAG_OK"), "{out}");
}

/// 赋值语句形态（watch/handler 族）：`sum = sum + item` 按语句定位发射，
/// 宿主映射后真 node 执行 40+2=42。
#[test]
fn p980_web_stmt_fragment_assigns_and_runs() {
    let body = "var sum = 40
sum = sum + item";
    // 第 3 条语句（idx 2）是赋值本体。
    let frag = emit_ui_stmt_fragment(body, 1, &["item"], &[("sum".into(), "sum.value".into())])
        .expect("assignment stmt fragment emits");
    let joined = frag.lines.join("\n");
    assert!(joined.contains('='), "assignment kept: {joined}");
    assert!(
        joined.contains("sum.value"),
        "host mapping applied: {joined}"
    );
    let out = run_fragment_on_node(
        &frag,
        "let sum = { value: 40 }; const item = 2;",
        "sum.value === 42",
    )
    .expect("node runs the stmt");
    assert!(out.contains("FRAG_OK"), "{out}");
}

/// 未覆盖/未定位形态=具名拒绝（不静默、无旧 emitter 回退）：
/// ①表达式源文本不在包装源内；②越界语句索引。
#[test]
fn p980_web_fragment_uncovered_shapes_refuse_by_name() {
    let body = "return count";
    let err = emit_ui_expr_fragment(body, "not_in_source", &["count"], &[])
        .expect_err("absent expression source refuses");
    assert!(err.contains("not found in the wrapper source"), "{err}");

    let err2 =
        emit_ui_stmt_fragment(body, 99, &["count"], &[]).expect_err("out-of-range stmt refuses");
    assert!(
        err2.contains("a2ts.fragment") || err2.contains("fragment"),
        "named fragment rejection: {err2}"
    );
}

/// 编译期拒绝透传：包装源带类型错误 → ACE 前门诊断带码（共享 verify 通道，
/// 旧 emitter 永远拿不到这种源）。
#[test]
fn p980_web_fragment_source_errors_carry_ace_codes() {
    let bad = "return 1 + \"s\"";
    let err = emit_ui_expr_fragment(bad, "\"text\" + 1", &[], &[])
        .expect_err("type error refuses at the shared front door");
    assert!(
        err.contains("source."),
        "ACE diagnostic code present: {err}"
    );
}
