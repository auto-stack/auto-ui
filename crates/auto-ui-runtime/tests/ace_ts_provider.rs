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

/// WEB 切片 #4：**逐 handler 精确定位（O-C3 解除）**。状态函数 HIR 的
/// expr span=原源坐标；同 span 多 expr 时索引序大者=组合形——`!on` 的 NOT
/// 保留（`(!l1___age_s_0_on)`），纯读形在后。host 映射后真 node 取反执行。
#[test]
fn p980_web_handler_precise_location() {
    use auto_ui_runtime::ui::ace_ts_provider::ComponentFragmentLane;
    let src = "widget Counter {\n    msg { Inc, Toggle }\n\n    model {\n        var count int = 0\n        var on bool = false\n    }\n\n    view {\n        col { text \"C\" }\n    }\n\n    on {\n        .Inc -> { .count += 1 }\n        .Toggle -> { .on = !.on }\n    }\n}\n";
    let lane = ComponentFragmentLane::compile(src, "p980-locate.at", &["Inc", "Toggle"])
        .expect("lane compiles");

    // Toggle handler：`.on`（`!` 的操作数 span——NOT 组合形同 span 在后）。
    let on_span = {
        let pos = src.find("!.on").expect("`!` present");
        pos + 1..pos + 4
    };
    let not = lane
        .emit_expr_at_span("__age_state_0_on", on_span)
        .expect("NOT-composed fragment locates precisely");
    let joined = not.text.join("\n");
    assert!(
        joined.contains("(!"),
        "negation semantics preserved: {joined}"
    );
    assert!(
        joined.contains("___age_s_0_on") || joined.contains("___age_state_0_on"),
        "state-ref form: {joined}"
    );
    // 宿主映射（发射局部名 → reactive 值）+ 真 node：!false === true。
    let mapped = joined.replace("l1___age_s_0_on", "on.value");
    let expr = not.expr.clone().expect("expr body");
    let driver = format!(
        "const on = {{ value: false }};\n{mapped}\nconst t = {expr};\nif (!(t === true)) {{ throw new Error(\"FRAG_ASSERT_FAILED\"); }}\nconsole.log(\"FRAG_OK\");\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-locate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("host.mjs"), driver).unwrap();
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg("host.mjs")
        .current_dir(&dir)
        .output()
        .expect("node spawn");
    let stdout = string_from_utf8lossy_fix(&out.stdout);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("FRAG_OK"),
        "node: {stdout} / {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Inc handler：`1`（复合赋值右操作数）精确 span 定位（不同 handler 区）。
    let one_pos = src.find("+= 1").expect("inc handler");
    let lit = lane
        .emit_expr_at_span("__age_state_0_count", one_pos + 3..one_pos + 4)
        .expect("literal fragment in the OTHER handler region");
    assert_eq!(
        lit.expr.as_deref(),
        Some("1"),
        "literal located: {:?}",
        lit.text
    );
    // 越界/无命中=具名拒。
    assert!(lane.emit_expr_at_span("__age_state_0_count", 0..1).is_err());
    assert!(lane.emit_expr_at_span("__age_state_9_x", 0..1).is_err());
}

fn string_from_utf8lossy_fix(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

/// WEB 切片 #5：**lane 直接消费真实生产 002 源**（适配器合同证明——
/// 适配器持有 widget 源文本，线程化=把源+span 送到 lane 的管道工程）。
/// 002 全源（msg 6 变体/model 3 字段/design/f-string/内联 lambda）经
/// states-only 组件面编译；ToggleMode 的 `!.dark_mode` NOT 组合形按**原源
/// span** 精确定位发射；宿主 reactive 映射后真 node 取反执行。
#[test]
fn p980_web_lane_consumes_production_002_source() {
    use auto_ui_runtime::ui::ace_ts_provider::ComponentFragmentLane;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("002 fixture: {e}"));

    let lane = ComponentFragmentLane::compile(
        &src,
        "p980-002.at",
        &[
            "Inc",
            "Dec",
            "Reset",
            "UseAuto",
            "UseSolarized",
            "ToggleMode",
        ],
    )
    .expect("production 002 source compiles through the lane");

    // ToggleMode handler：`.dark_mode = !.dark_mode`——坐标基准=脱糖源
    //（f-string 脱糖位移；lane 暴露基准源），`!` 操作数 span 精确定位，
    // NOT 与纯读同 span=操作数——组合形胜出。
    let ds = lane.desugared_source();
    let bang = ds.find("!.dark_mode").expect("`!` in the desugared source");
    let frag = lane
        .emit_expr_at_span("__age_state_0_dark_mode", bang + 1..bang + 11)
        .expect("NOT-composed fragment locates in the production source");
    let joined = frag.text.join("\n");
    assert!(
        joined.contains("(!"),
        "negation preserved from 002: {joined}"
    );
    assert!(
        joined.contains("___age_s_0_dark_mode"),
        "state ref form: {joined}"
    );
    // 宿主 reactive 映射（002 局部索引 l2——字段序 count/theme_name/dark_mode）
    // + 真 node：!false === true。
    let mapped = joined
        .replace("l2___age_s_0_dark_mode", "dark_mode.value")
        .replace("l1___age_s_0_dark_mode", "dark_mode.value");
    let expr = frag.expr.clone().expect("expr body");
    let driver = format!(
        "const dark_mode = {{ value: false }};\n{mapped}\nconst t = {expr};\nif (!(t === true)) {{ throw new Error(\"FRAG_ASSERT_FAILED\"); }}\nconsole.log(\"FRAG_OK\");\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-002-lane-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("host.mjs"), driver).unwrap();
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
        "node: {stdout} / {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Inc handler 在同一 lane 的 count 状态函数内定位（多 handler 区分）。
    // 002 的 `.count += 1` 首现于 **view 内联 lambda**（不进状态函数）——
    // 从 on 块起推导 handler 区 occurrence（脱糖源坐标）。
    let on_pos = ds.find("on {").expect("on block");
    let rel = ds[on_pos..]
        .find(".count += 1")
        .expect("Inc handler in the on block");
    let inc_start = on_pos + rel;
    let inc = lane
        .emit_expr_at_span("__age_state_0_count", inc_start..inc_start + 11)
        .expect("Inc handler locates in the count state fn");
    let inc_joined = inc.text.join("\n");
    assert!(
        inc_joined.contains("___age_s_0_count")
            || inc.expr.as_deref().is_some_and(|e| e.contains("count")),
        "count state reference emitted: {inc_joined} / {:?}",
        inc.expr
    );
}

/// WEB 切片 #6：**提取携带 span → lane 接线**（真实解析管线闭环）。002
/// 经 `CompilerSession`+`Parser` 解析 → `extract_handler_spans`（原源坐
/// 标）→ span 内容核验（含 pattern 与 `!` 形态）→ lane（脱糖坐标）在
/// **提取区间**内锚定 → ToggleMode 的 NOT fragment。铸造形（view 内联
/// lambda）无 span=缺席可辨。
#[test]
fn p980_web_extraction_carried_spans_feed_lane() {
    use auto_ui_compiler::aura::extract_handler_spans;
    use auto_ui_dialect::UiStmt;

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("002 fixture: {e}"));

    // 真实解析管线（vue_capabilities 同链）。
    let session = auto_parser_api::session::CompilerSession::ui();
    let mut parser = auto_lang_core::parser::Parser::from(src.as_str()).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .expect("002 parses");
    let decl = ast
        .domain_stmts()
        .find_map(|s| match s {
            UiStmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");

    // 提取携带 span（原源坐标）：6 个 on-block handler（铸造形缺席）。
    let spans = extract_handler_spans(&decl);
    assert_eq!(spans.len(), 6, "declared on-block handlers carry spans");
    let (off, len) = spans.get(".ToggleMode").copied().expect("ToggleMode span");
    let region = &src[off..off + len];
    // O-C4：span 起点落在 pattern 中段（小偏移；终点正确、body 完整覆盖）——
    // lane 锚定不依赖起点精度（区间内文本锚）。精确起点归切片#6b 校准。
    assert!(
        region.contains("Mode ->") && region.contains("!.dark_mode"),
        "span covers the handler body: {region}"
    );

    // lane（脱糖坐标）：提取区间内容锚（pattern 在脱糖源中定位 → 区间内
    // `!` 操作数）→ NOT fragment（端到端：解析→提取→lane）。
    use auto_ui_runtime::ui::ace_ts_provider::ComponentFragmentLane;
    let lane = ComponentFragmentLane::compile(
        &src,
        "p980-span-feed.at",
        &[
            "Inc",
            "Dec",
            "Reset",
            "UseAuto",
            "UseSolarized",
            "ToggleMode",
        ],
    )
    .expect("lane compiles");
    let ds = lane.desugared_source();
    let ds_toggle = ds.find(".ToggleMode").expect("pattern in desugared source");
    let bang = ds[ds_toggle..]
        .find("!.dark_mode")
        .expect("NOT form within the extraction-carried region");
    let at_ds = ds_toggle + bang;
    let frag = lane
        .emit_expr_at_span("__age_state_0_dark_mode", at_ds + 1..at_ds + 11)
        .expect("NOT fragment via extraction-carried span region");
    let joined = frag.text.join("\n");
    assert!(joined.contains("(!"), "negation preserved: {joined}");
}

/// WEB 切片 #6c（路线 B 核心）：**lane 派生 handler 函数体**（002 真源）。
/// 逐 handler：区间最宽 RHS 发射→reactive 映射→`field.value = rhs;` 组装；
/// Inc 体真 node 语义验证（41→42）；写目标字段自动推导；缺席清单可辨。
#[test]
fn p980_web_route_b_handler_bodies() {
    use auto_ui_runtime::ui::ace_vue_script::lane_handler_bodies;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("002: {e}"));

    let (bodies, absent) = lane_handler_bodies(
        &src,
        "p980-routeb.at",
        &[".Inc", ".Dec", ".Reset", ".ToggleMode", ".UseAuto"],
    )
    .expect("route-B bodies derive");
    assert!(
        absent.is_empty(),
        "002 五 handler 均命中: absent={absent:?}"
    );
    let by_field = |f: &str| {
        bodies
            .iter()
            .find(|b| b.field == f)
            .unwrap_or_else(|| panic!("field {f}"))
    };

    // Inc：count.value = count.value + 1（helper const + 赋值行）。
    let inc = by_field("count");
    assert_eq!(inc.pattern, ".Inc");
    let inc_text = inc.lines.join("\n");
    assert!(
        inc_text.contains("count.value = "),
        "assignment line present: {inc_text}"
    );
    assert!(
        inc_text.contains("count.value")
            && (inc_text.contains("a2x_add") || inc_text.contains("+ 1")),
        "increment semantics (add helper or infix): {inc_text}"
    );

    // 真 node：Inc 体在 reactive 宿主上执行（41→42；a2x_rt helper 就位）。
    let body_js = inc.lines.join("\n");
    let driver = format!(
        "import {{ a2x_add }} from \"./a2x_rt.mjs\";\nconst count = {{ value: 41 }};\n{body_js}\nif (!(count.value === 42)) {{ throw new Error(\"FRAG_ASSERT_FAILED\"); }}\nconsole.log(\"FRAG_OK\");\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-routeb-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("a2x_rt.mjs"),
        ace_a2x_ts::script_runtime_body(ace_a2x_ts::ScriptLang::JavaScript, &[]),
    )
    .unwrap();
    std::fs::write(dir.join("host.mjs"), driver).unwrap();
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
        "node Inc body: {stdout} / {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // ToggleMode：dark_mode 取反；UseAuto：theme_name 字符串字面量。
    let toggle_text = bodies
        .iter()
        .find(|b| b.field == "dark_mode")
        .map(|b| b.lines.join("\n"))
        .expect("ToggleMode body");
    assert!(
        toggle_text.contains("dark_mode.value = !") || toggle_text.contains("!"),
        "negation body: {toggle_text}"
    );
    let theme_text = bodies
        .iter()
        .find(|b| b.field == "theme_name")
        .map(|b| b.lines.join("\n"))
        .expect("UseAuto body");
    assert!(
        theme_text.contains("theme_name.value = ") && theme_text.contains("Auto"),
        "theme literal body: {theme_text}"
    );
}

/// WEB 切片 #6d：**SFC 拼接**——真实管线（parse→extract→VueGenerator SFC）
/// + lane 体 → splice：002 五 handler 函数体替换为 lane 派生体；锚外段落
/// 字节不变（对账：非 handler 部分前后一致）；Inc 体含 reactive 语义。
#[test]
fn p980_web_sfc_splice() {
    use auto_ui_compiler::ui_gen::BackendGenerator;
    use auto_ui_dialect::UiStmt;
    use auto_ui_runtime::ui::ace_vue_script::{lane_handler_bodies, splice_lane_handler_bodies};

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("002: {e}"));

    // 真实编译器管线出 SFC。
    let session = auto_parser_api::session::CompilerSession::ui();
    let mut parser = auto_lang_core::parser::Parser::from(src.as_str()).with_session(session);
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .expect("002 parses");
    let decl = ast
        .domain_stmts()
        .find_map(|s| match s {
            UiStmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .unwrap();
    let widget = auto_ui_compiler::aura::extract_widget_from_decl(decl).expect("extract");
    let sfc = auto_ui_compiler::ui_gen::VueGenerator::new()
        .generate_sfc(&widget)
        .expect("SFC");

    // lane 体 + 拼接。
    let (bodies, absent) = lane_handler_bodies(
        &src,
        "p980-splice.at",
        &[
            ".Inc",
            ".Dec",
            ".Reset",
            ".UseAuto",
            ".UseSolarized",
            ".ToggleMode",
        ],
    )
    .expect("bodies");
    assert!(absent.is_empty(), "absent: {absent:?}");
    let spliced = splice_lane_handler_bodies(&sfc, &bodies).expect("splice");

    // ①Inc 体=lane 派生（reactive 语义）。
    let inc = bodies.iter().find(|b| b.pattern == ".Inc").unwrap();
    let inc_fn_start = spliced.find("function Inc(): void {").unwrap();
    let inc_fn = &spliced[inc_fn_start..inc_fn_start + 200];
    assert!(
        inc_fn.contains("count.value = ") && (inc_fn.contains("a2x_add") || inc_fn.contains("+ 1")),
        "spliced Inc body carries lane semantics: {inc_fn}"
    );
    assert_eq!(inc.field, "count");

    // ②非 handler 段落字节不变：模板/样式/状态声明区（splice 前后对账）。
    fn template_on(s: &str) -> &str {
        let t = s.find("<template>").unwrap();
        &s[t..]
    }
    assert_eq!(
        template_on(&spliced),
        template_on(&sfc),
        "template section byte-identical"
    );
    fn state_decl_on(s: &str) -> &str {
        let t = s.find("const count = ref").unwrap();
        &s[..t]
    }
    assert_eq!(
        state_decl_on(&spliced),
        state_decl_on(&sfc),
        "pre-state script section byte-identical"
    );

    // ③ToggleMode 体=取反；锚精确（不含相邻函数）。
    let tm_start = spliced.find("function ToggleMode(): void {").unwrap();
    let tm_fn = &spliced[tm_start..tm_start + 160];
    assert!(
        tm_fn.contains("dark_mode.value = ") && tm_fn.contains('!'),
        "ToggleMode negation: {tm_fn}"
    );
}

/// WEB 切片 #6e：**一体化入口 + 覆盖普查 v2（冻结表）**——代表性能力族
/// 语料经 `generate_sfc_with_lane` 的结果分类（spliced 数 / absent=旧道
/// 保持 / 编译具名拒）。双轨期合同：absent 不拒绝；表变化须具名修订。
#[test]
fn p980_web_unified_entry_census_v2() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    let corpus: &[(&str, &str)] = &[
        (
            "counter-3h",
            "widget Counter {\n    msg { Inc, Dec, Reset }\n\n    model {\n        var count int = 0\n    }\n\n    view {\n        col { text \"C\" button \"+\" { onclick: .Inc } }\n    }\n\n    on {\n        .Inc -> { .count += 1 }\n        .Dec -> { .count -= 1 }\n        .Reset -> { .count = 0 }\n    }\n}\n",
        ),
        (
            "str-field",
            "widget Theme {\n    msg { UseAuto }\n\n    model {\n        var theme str = \"Auto\"\n    }\n\n    view {\n        col { text \"T\" }\n    }\n\n    on {\n        .UseAuto -> { .theme = \"Auto\" }\n    }\n}\n",
        ),
        (
            "no-on",
            "widget Plain {\n    model {\n        var n int = 1\n    }\n    view {\n        text \"P\"\n    }\n}\n",
        ),
        (
            "watch-only",
            "widget W {\n    model {\n        var n int = 0\n    }\n    watch {\n        .n -> { .n = 1 }\n    }\n    view {\n        text \"W\"\n    }\n}\n",
        ),
        (
            "param-handler",
            "widget L {\n    msg { Add(text) }\n    model {\n        var n int = 0\n    }\n    view {\n        text \"L\"\n    }\n    on {\n        .Add(t) -> { .n = 1 }\n    }\n}\n",
        ),
    ];

    // 冻结表（2026-11-16 实测；变化须具名修订）：
    // counter-3h=3 spliced；str-field=1；no-on/watch-only=0（passthrough）；
    // param-handler=absent（参数化 handler 体 lane 未覆盖——旧道保持，#6f 面）。
    let expect: &[(&str, usize, bool)] = &[
        ("counter-3h", 3, false),
        ("str-field", 1, false),
        ("no-on", 0, false),
        ("watch-only", 0, false),
        ("param-handler", 0, true),
    ];
    for ((name, src), (exp_name, spliced, has_absent)) in corpus.iter().zip(expect) {
        assert_eq!(name, exp_name);
        let (sfc, bodies, absent) =
            generate_sfc_with_lane(src, "p980-census-v2.at").expect("{name} derives");
        assert!(sfc.contains("<script"), "{name}: SFC produced");
        assert_eq!(
            bodies.len(),
            *spliced,
            "{name}: spliced count (frozen census v2)"
        );
        assert_eq!(
            !absent.is_empty(),
            *has_absent,
            "{name}: absent presence (frozen census v2): {absent:?}"
        );
    }
}
