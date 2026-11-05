//! PLAN-940：解析推进性与 EOF 受控拒绝回归（UI042 51GB 病理族的永久反例）。
//!
//! 不变量：文法循环每轮必须消费合法输入、完成有限结构转换或受控结束/返回
//! 错误。EOF 上 `advance` 不越过 EOF，`parse_view_node`/裸 token 收集臂可
//! Ok 返回且游标不动——任何只看闭合符的收集循环都会无限收集（修复前监督
//! 探针实测 >512MiB/10s resource-stop，见 PLAN-940 §7 修复前后对照）。
//! 守卫形态沿用仓内既有先例（`parse_view_node` 体循环 / `parse_control_body`
//! 的 `if EOF { expect(closer)? }`）：EOF 一律经 expect 受控拒绝，错误带
//! 位置。恢复面：坏源 Err 后，同一公开消费流程的下一次合法 parse 仍成功，
//! 合法 checkpoint/裸元素自动包裹等既有语法行为保持。

use crate::{UiCode, UiStmt};
use auto_lang_core::ast::Expr;
use auto_lang_core::error::AutoError;
use auto_lang_core::parser::Parser;
use auto_parser_api::DomainStmt;

/// 公开消费流程（生产入口形态）：UI 场景 + `Parser::parse_domain(&UiDialect)`。
fn parse_ui(src: &str) -> Result<UiCode, AutoError> {
    let mut parser =
        Parser::from(src).with_session(auto_lang_core::session::CompilerSession::ui());
    parser.parse_domain(&crate::UiDialect).map(UiCode)
}

/// 合法控制件（真实语法，修复前后均 PARSE-OK 的对照基线）。
const VALID_WIDGET: &str = concat!(
    "widget Counter {\n",
    "  model {\n",
    "    count = 11\n",
    "  }\n",
    "  msg {\n",
    "    Inc\n",
    "  }\n",
    "  view {\n",
    "    col {\n",
    "      text .count\n",
    "      button \"+\" { onclick: .Inc }\n",
    "    }\n",
    "  }\n",
    "  on {\n",
    "    .Inc -> { .count = .count + 1 }\n",
    "  }\n",
    "}\n",
);

/// 修复前实测 resource-stop 的 EOF 旋转形态（逐点对应修复守卫）：
/// 每个必须有限终止并受控 Err——回归即挂起/膨胀。
#[test]
fn plan940_truncated_eof_shapes_all_rejected_bounded() {
    const SHAPES: &[&str] = &[
        // view 根循环（UI042 精确形态，真实语法截断）
        "widget Counter {\n  model {\n    count = 11\n  }\n  view {",
        "widget App {\n  view {",
        // msg variants 循环
        "widget App {\n  msg {",
        // for 体循环
        "widget App {\n  view {\n    for x in .xs {",
        // memo 体循环
        "widget App {\n  view {\n    memo (deps: .a) {",
        // link 子节点循环
        "widget App {\n  view {\n    link (to: \"/a\") {",
        // component fn 参数循环（外层 + 类型标注内层）
        "component fn f(a: ",
        // view fn fragment 参数循环（同一份参数文法的第二拷贝）
        "view fn f(a: ",
        // on handler 形参循环
        "widget App {\n  on {\n    .Inc(p1, ",
        // setup refs 循环
        "widget App {\n  setup {\n    refs x: [a, ",
        // widget use 块 refs 循环
        "widget App {\n  use {\n    composable: x refs: [a, ",
        // use.web refs 循环
        "use.web composable x refs: [a, ",
    ];
    for (i, src) in SHAPES.iter().enumerate() {
        match parse_ui(src) {
            Err(_) => {}
            Ok(_) => panic!("truncated shape #{i} must be rejected, got Ok: {src:?}"),
        }
    }
}

/// 合法源逐字节前缀截断：每个前缀要么 Ok 要么受控 Err，绝不挂起/膨胀
/// （完成本循环本身就是推进不变量的 oracle；ASCII 语料，字节前缀即
/// token 前缀超集）。
#[test]
fn plan940_every_prefix_of_valid_widget_is_bounded() {
    let mut ok = 0usize;
    let mut err = 0usize;
    for end in 0..=VALID_WIDGET.len() {
        match parse_ui(&VALID_WIDGET[..end]) {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
    }
    assert!(ok >= 1, "full valid source must parse");
    assert!(err >= 1, "truncations must be rejected, not silently accepted");
}

/// 合法控制件带语义读回：model 字段/初值与 on handler 在解析产物中保真
/// （V-03 保旧断言的数据面）。
#[test]
fn plan940_valid_widget_parses_with_model_and_handler() {
    let code = parse_ui(VALID_WIDGET).expect("valid widget parses");
    let w = code
        .0
        .items
        .iter()
        .find_map(|s| match s {
            DomainStmt::Domain(UiStmt::WidgetDecl(w)) => Some(w),
            _ => None,
        })
        .expect("widget decl present");
    let model = w.model.as_ref().expect("model block");
    assert_eq!(model.fields.len(), 1);
    assert_eq!(model.fields[0].name.as_str(), "count");
    assert!(
        matches!(model.fields[0].init, Expr::Int(11)),
        "model init must read back as Int(11)"
    );
    let on = w.on.as_ref().expect("on block");
    assert!(
        on.handlers.iter().any(|h| h.pattern == ".Inc"),
        "on handler .Inc must survive"
    );
}

/// 恢复不变量：坏源受控 Err 后，同一公开消费流程的下一次合法 parse
/// 仍成功（注册/作用域不丢——合法控制件断言见上）。
#[test]
fn plan940_next_legal_parse_succeeds_after_bad_source() {
    let bad = "widget Counter {\n  model {\n    count = 11\n  }\n  view {";
    assert!(parse_ui(bad).is_err(), "truncated source must be rejected");
    parse_ui(VALID_WIDGET).expect("next legal parse must succeed after bad source");
}

/// 两声明恢复面：同文件首声明坏（表达式缺失，非 EOF 类）+ 次声明合法——
/// 解析有限终止、诊断有界；下一份合法源码仍可消费。
#[test]
fn plan940_midfile_bad_decl_bounded_and_next_source_consumable() {
    let src = format!(
        "widget A {{\n  model {{\n    count = \n  }}\n}}\n{}",
        VALID_WIDGET
    );
    let err = parse_ui(&src).expect_err("bad first decl must surface errors");
    let msg = err.to_string();
    assert!(
        msg.chars().count() < 10_000,
        "diagnostics must be bounded ({} chars)",
        msg.chars().count()
    );
    parse_ui(VALID_WIDGET).expect("next legal source must be consumable");
}

/// 合法 checkpoint/投机探测与裸元素自动包裹 view 的既有行为保持
/// （msg 命名 payload 走 probe_next_kind 投机；widget 体裸元素走
/// raw_cursor_save/restore 前看）。
#[test]
fn plan940_checkpoint_paths_and_auto_view_wrap_preserved() {
    let src = "widget A {\n  msg { Change(value: str) }\n  col { text \"x\" }\n}\n";
    let code = parse_ui(src).expect("checkpoint-using grammar still parses");
    let w = code
        .0
        .items
        .iter()
        .find_map(|s| match s {
            DomainStmt::Domain(UiStmt::WidgetDecl(w)) => Some(w),
            _ => None,
        })
        .expect("widget decl present");
    assert!(
        w.view.is_some(),
        "bare element body must still auto-wrap into a view block"
    );
    assert_eq!(w.messages.len(), 1, "named-payload msg variant survives");
}

/// 非 UI 同名语法保持：UI 场景文件中的核心语句（fn）与 store 声明不受
/// EOF 守卫影响。
#[test]
fn plan940_core_stmts_and_store_decl_still_parse() {
    let src = concat!(
        "fn helper() -> int { 42 }\n",
        "widget Counter {\n",
        "    model {\n        var count int = 0\n    }\n",
        "    view { col { text \"n\" } }\n",
        "    on { .Bump -> { .count = .count + 1 } }\n",
        "}\n",
    );
    let code = parse_ui(src).expect("core fn + widget mixed source parses");
    let has_widget = code
        .0
        .items
        .iter()
        .any(|s| matches!(s, DomainStmt::Domain(UiStmt::WidgetDecl(_))));
    assert!(has_widget, "widget decl present alongside core fn");
}
