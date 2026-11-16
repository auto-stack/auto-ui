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
        &[("count", "int")],
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
    let frag = emit_ui_stmt_fragment(
        body,
        1,
        &[("item", "int")],
        &[("sum".into(), "sum.value".into())],
    )
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
    let err = emit_ui_expr_fragment(body, "not_in_source", &[("count", "int")], &[])
        .expect_err("absent expression source refuses");
    assert!(err.contains("not found in the wrapper source"), "{err}");

    let err2 = emit_ui_stmt_fragment(body, 99, &[("count", "int")], &[])
        .expect_err("out-of-range stmt refuses");
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

/// WEB 切片 #2：**车道覆盖普查（冻结表）**——生产 UI 表达式/语句形态经
/// 976 fragment 新车道（Core profile 编译入口）的逐形态结果。本表=适配器
/// 切换的可测分母：OK 形态即今日可切换面；具名拒形态=切换前须先扩
/// profile/入口（976 后继或 richer entry），**不是** ts_adapter 线程化能
/// 解决的。表变化须具名修订（防静默漂移）。
#[test]
fn p980_web_lane_census_frozen_forms() {
    use auto_ui_runtime::ui::ace_ts_provider::{emit_ui_expr_fragment, emit_ui_stmt_fragment};
    // (形态名, 体, 定位, 参数, 期望: Ok 或含具名码的 Err 子串)
    let cases: &[(&str, &str, Option<&str>, &[(&str, &str)], ResultKind)] = &[
        // —— OK 面（round 23 已证 + 本轮复核）——
        (
            "int-arith-return",
            "return count + 1",
            Some("count + 1"),
            &[("count", "int")],
            ResultKind::Ok,
        ),
        (
            "int-assign-stmt",
            "var sum = 40\nsum = sum + item",
            None,
            &[("item", "int")],
            ResultKind::Ok,
        ),
        // O-C2（形状缺口）：非字面 var-init（参数引用/表达式初始化）在
        // block-graph 校验拒——字面初始化的两步赋值形态 OK（见上）。
        (
            "bool-assign-stmt",
            "var f2 = flag\nf2 = false\nreturn 0",
            None,
            &[("flag", "bool")],
            ResultKind::Reject("verify.block-structure"),
        ),
        // —— 具名拒面（profile 域缺口——切换前置义务，非线程化可解）——
        (
            "cmp-gt",
            "var ok = count > 1\nreturn 0",
            Some("count > 1"),
            &[("count", "int")],
            ResultKind::Reject("operator `>` is not part of this source profile"),
        ),
        (
            "cmp-eq",
            "var ok = count == 1\nreturn 0",
            Some("count == 1"),
            &[("count", "int")],
            ResultKind::Reject("operator `==` is not part of this source profile"),
        ),
        (
            "bool-not",
            "var f2 = !flag\nreturn 0",
            Some("!flag"),
            &[("flag", "bool")],
            ResultKind::Reject("unary `!` is not part of this source profile"),
        ),
        (
            "bool-logic",
            "var f2 = flag && ok\nreturn 0",
            Some("flag && ok"),
            &[("flag", "bool"), ("ok", "bool")],
            ResultKind::Reject("operator `&&` is not part of this source profile"),
        ),
        (
            "str-type",
            "var t = name\nreturn 0",
            Some("name"),
            &[("name", "str")],
            ResultKind::Reject("type `str` is not part of this source profile"),
        ),
        (
            "if-stmt-cmp",
            "if count > 1 { var t = 2 } else { var t = 3 }\nreturn 0",
            None,
            &[("count", "int")],
            ResultKind::Reject("operator `>` is not part of this source profile"),
        ),
        (
            "interp-fstr",
            "var s = `x${count}`\nreturn 0",
            Some("`x${count}`"),
            &[("count", "int")],
            ResultKind::Reject("source.lex"),
        ),
        (
            "multi-fn-helper",
            "fn h() int { return 3 }",
            None,
            &[],
            ResultKind::Reject("source.syntax"),
        ),
        // —— 形状拒（同入口的可分辨行为；O-C1：var-init+return 形状的
        //    block-structure 拒——赋值两步形态 OK，见上）——
        (
            "var-init-dead-local",
            "var t = 40\nreturn 0",
            None,
            &[("count", "int")],
            ResultKind::Reject("verify.block-structure"),
        ),
    ];
    for (name, body, locator, params, expect) in cases {
        let r = match locator {
            Some(expr) => emit_ui_expr_fragment(body, expr, params, &[]),
            None => emit_ui_stmt_fragment(body, 0, params, &[]),
        };
        match (expect, &r) {
            (ResultKind::Ok, Ok(_)) => {}
            (ResultKind::Reject(want), Err(e)) => {
                assert!(
                    e.contains(want),
                    "census[{name}]: {e}\n expected named rejection containing {want:?}"
                )
            }
            (ResultKind::Ok, Err(e)) => {
                panic!("census[{name}]: expected OK, got {e}（表须具名修订）")
            }
            (ResultKind::Reject(want), Ok(f)) => panic!(
                "census[{name}]: expected rejection {want:?}, got OK {:?}（表须具名修订）",
                f.lines
            ),
        }
    }
}

#[derive(Clone, Copy)]
enum ResultKind {
    Ok,
    Reject(&'static str),
}

/// WEB 切片 #3：**组件面 = fragment 扩域入口（机制钉住）**。生产形态源
/// （`!` 一元/str/bool 字段——Core profile 具名拒的形态）经组件面
/// states-only 编译 → `SemanticView` → 976 fragment 发射器可运行。
/// **形态事实**：handler 内 model 访问脱糖为 `__age_state_<i>_<field>()`
/// 调用——fragment=调用形（helper=`a2x_fn_...`），宿主映射面=状态函数→
/// reactive 名（host_rewrite 词法整词替换可承载）。端到端 node 执行。
#[test]
fn p980_web_component_face_entry_fragments() {
    use ace_a2x_core::SemanticView;
    use ace_a2x_ts::{
        emit_expr_fragment, innermost_expr_in_span, FragmentProgramContext, ScriptLang,
    };
    let src = "widget Counter {\n    msg { Inc, Toggle }\n\n    model {\n        var count int = 0\n        var on bool = false\n    }\n\n    view {\n        col { text \"C\" }\n    }\n\n    on {\n        .Inc -> { .count += 1 }\n        .Toggle -> { .on = !.on }\n    }\n}\n";
    let caps = ["ac.runtime.text.v1".to_string()];
    let scenario = auto_ui_runtime::ui::ace_state_bridge::AceAlias::Accept;
    // 组件面 states-only 编译（真实入口；str/bool/`!` 全部通过——Core
    // profile 同源具名拒的形态在此域内）。
    let compiled = ace_av::age_component::compile_component_scenario_states(
        src,
        "p980-comp-entry.at",
        ace_av::age_component::LegacyWidgetAlias::Accept,
        &caps,
        &ace_av::age_component::ComponentScenario {
            instances: 1,
            ops: vec![],
        },
        &ace_av::age_component::ComponentInitials {
            per_instance: vec![vec![]],
        },
    )
    .expect("component face compiles the production-form source");
    let view = SemanticView::from_checked(&compiled.checked).expect("semantic view");
    let drive = view
        .functions()
        .iter()
        .find(|f| f.source_name == "__age_drive")
        .expect("desugared driver fn");
    // 按源 span 定位（O-C3：脱糖 drive 的 span 精度——`!.on` 源段定位落到
    // count 状态调用，PLAN-964 span 移位同族边界；精确 per-handler 定位=
    // 切片#4 前置。本 case 钉机制：定位→发射→映射→node 读值）。
    let start = src.find("!.on").expect("span source present");
    let expr_id =
        innermost_expr_in_span(drive, start..start + 4).expect("HIR expr located by span");
    let ctx = FragmentProgramContext::from_view(&view);
    let raw = emit_expr_fragment(ScriptLang::TypeScript, &view, drive, expr_id, &ctx)
        .expect("fragment emits from the component-face HIR");
    // 形态事实：脱糖调用形（状态函数调用），非源级算子形。
    let joined = raw.text.join("\n");
    assert!(
        joined.contains("a2x_fn___age_state_0_"),
        "desugared state-fn call form: {joined}"
    );
    // 宿主映射（状态函数发射名 → reactive 宿主名）：调用形 `fn()` →
    // `count.value`（吃掉调用括号——状态读取语义），裸名残留再兜一遍。
    let mapped = joined
        .replace("a2x_fn___age_state_0_count()", "count.value")
        .replace("a2x_fn___age_state_0_on()", "on.value")
        .replace("a2x_fn___age_state_0_count", "count.value")
        .replace("a2x_fn___age_state_0_on", "on.value");
    let expr = raw.expr.clone().expect("expr body");
    // 真 node：reactive 宿主预置，断言读值语义（count.value=7 → t0===7）。
    let driver_src = format!(
        "const count = {{ value: 7 }}; const on = {{ value: false }};\n{mapped}\nconst t = {expr};\nif (!(t === 7)) {{ throw new Error(\"FRAG_ASSERT_FAILED\"); }}\nconsole.log(\"FRAG_OK\");\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-comp-frag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("host.mjs"), driver_src).unwrap();
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg("host.mjs")
        .current_dir(&dir)
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("FRAG_OK"),
        "node run: {} / {}",
        stdout,
        String::from_utf8_lossy(&out.stderr)
    );
}
