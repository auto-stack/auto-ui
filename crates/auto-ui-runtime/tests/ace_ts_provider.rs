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
            "widget L {\n    msg Msg { Add(str) }\n\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"L\" }\n    }\n\n    on {\n        .Add(t) -> { .n = 1 }\n    }\n}\n",
        ),
        (
            // 切片#6j：param handler 实读参（QueryInput 形）——区别于常量体
            // 行：spliced 的判别力在"形参进入体"（p980_web_param_environment
            // 断言体文本/node/浏览器三层）。
            "param-read",
            "widget R {\n    msg Msg { Query(str) }\n\n    model {\n        var last str = \"\"\n    }\n\n    view {\n        col { text \"R\" }\n    }\n\n    on {\n        .Query(q) -> { .last = q }\n    }\n}\n",
        ),
        (
            // 切片#6j 负控行：float 载荷不可承载（981 派发守卫同族
            // fail-closed）→ 具名缺席（旧道承载），不造假数值。
            "float-payload",
            "widget F2 {\n    msg Msg { Resize(float) }\n\n    model {\n        var size int = 0\n    }\n\n    view {\n        col { text \"F\" }\n    }\n\n    on {\n        .Resize(w) -> { .size = 1 }\n    }\n}\n",
        ),
        (
            // 内联 lambda 铸造形（view onclick 无 on 块）：铸造 handler 无源
            // span/场景面不可重放（与参数化同族域缺口）→ 旧道。
            "inline-lambda-only",
            "widget B {\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"B\" button \"+\" { onclick: () => { .n += 1 } } }\n    }\n}\n",
        ),
        (
            // PLAN-980 lambda-if 刀（2026-11-19）：if 体 lambda（005 族——str
            // `!=` 条件+分支赋值）——结构化铸造+If 整语句 lane 发射。
            "lambda-if",
            "widget P {\n    model {\n        var email str = \"\"\n        var email_error str = \"err\"\n    }\n\n    view {\n        col { input { oninput: () => { if .email != \"\" { .email_error = \"\" } } } }\n    }\n}\n",
        ),
    ];

    // 冻结表（2026-11-16 实测；#6i 语法扩展后修订——具名 msg+带参 handler
    // 现在可拼接：grammar opt msg_name + opt params + splice `): void {` 锚）。
    // **2026-11-17 具名修订（PLAN-981 B-W1 / IF-981-EVENT-PAYLOAD-B）**：
    // `Send` 恒等价空载荷——向载荷变体（`Add(str)`）空 Send 现在是 arity 域
    // 错误（A 期组件面允许"丢参跑常量体"的拼接随之失效）；带参 handler 的
    // lane 拼接改由 980 B-WEB 消费 typed 接口（`SendArgs`/参数环境）后恢复。
    // param-handler 因此 1→0+absent；其余行不变。
    // **2026-11-17 具名修订 #2（切片#6j：980 B-WEB 消费 981 声明级检查函
    // 数，参数环境落地）**：param-handler 0→1（常量体经 `__age_hv_*` 检查
    // 函数发射恢复拼接）；新增 param-read=1（实读参行——与常量体行分列，
    // "spliced=1 不等价动态载荷支持"的判别由 p980_web_param_environment 承
    // 担）；新增 float-payload=0+absent（不可承载载荷具名缺席，不造假数值）。
    // counter-3h=3；str-field=1；no-on/watch-only=0（passthrough）。
    // **2026-11-17 具名修订 #3（切片#6k：V-EVT-B-ORIGIN）**：inline-lambda-
    // only 0+absent→1——语言侧铸 `__age_vl_<seq>` 值函数（赋值 RHS 原源
    // span），lane 以源 `=> {` 区间配对（铸造计数序）发射，SFC 合成
    // `__evt_*` 体经 lane 拼接（真实 body/origin，不用旧 TS fallback）。
    // **2026-11-19 具名修订 #10（PLAN-980 lambda-if 刀）**：新增 lambda-if=1
    // 行——if 体 lambda（005 `oninput: () => { if .email != "" { … } }` 族）
    // 从 foreign 不铸（缺席）翻转为结构化铸造+If 整语句 lane 发射；判别力
    // 由 `p980_lambda_if_lane`（区间发现/If 整语句片段/a2x_add 判别器/调用
    // 形负控）与浏览器档承担。005 整例仍缺席=use/style/design 既名墙连带
    //（剥离墙片段 3/3 派生，plan-artifacts 探针摘要）。
    let expect: &[(&str, usize, bool)] = &[
        ("counter-3h", 3, false),
        ("str-field", 1, false),
        ("no-on", 0, false),
        ("watch-only", 0, false),
        ("param-handler", 1, false),
        ("param-read", 1, false),
        ("float-payload", 0, true),
        ("inline-lambda-only", 1, false),
        ("lambda-if", 1, false),
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

/// WEB 切片 #6g：**全量双轨普查（冻结聚合表）**——vue_capabilities 全部
/// 源语料（文件级提取 r#"…"#）经 `generate_sfc_with_lane` 的聚合结果。
/// 双轨期合同：absent=旧道。表变化须具名修订（防静默漂移）。
#[test]
fn p980_web_full_dual_track_census() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;
    let vc_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vue_capabilities.rs");
    let vc = std::fs::read_to_string(&vc_path).expect("vue_capabilities source readable");
    let mut corpus: Vec<&str> = Vec::new();
    let mut rest = vc.as_str();
    while let Some(i) = rest.find("r#\"") {
        let after = &rest[i + 3..];
        match after.find("\"#") {
            Some(j) => {
                corpus.push(&after[..j]);
                rest = &after[j + 2..];
            }
            None => break,
        }
    }
    assert!(
        corpus.len() >= 80,
        "corpus extraction sanity: {}",
        corpus.len()
    );
    let mut ok = 0usize;
    let mut err = 0usize;
    let mut spliced = 0usize;
    let mut absent = 0usize;
    for src in &corpus {
        match generate_sfc_with_lane(src, "p980-dual.at") {
            Ok((_sfc, bodies, abs)) => {
                ok += 1;
                spliced += bodies.len();
                absent += abs.len();
            }
            Err(_) => err += 1,
        }
    }
    // 冻结聚合（2026-11-16 实测；变化须具名修订）：
    // ok=72（含 state-slot 归一化），err=11（2 store+其余=vue 10 预存红
    // 同族解析错误——"duplicate view block"族，master 预存，KNOWN-DEBT
    // 在案归 triage），spliced=0/absent=23：**真语料主导形态=具名
    // `msg Msg {...}`/带参 handler/铸造 lambda——组件面语法域外**（语言侧
    // 前置×3：ScenarioOp 无 payload、具名 msg 语法、参数绑定面）。
    //
    // **2026-11-18 具名修订 #4（切片#6l：V-B-WEB 处置——text-leaf 门放
    // 宽（states-only 不要求 text 叶）+ 方言 `state` 槽位别名 + 多赋值逐
    // 语句发射）**：(72,11,0,23) → **(75, 8, 1, 27)**。ok+3=state 别名解
    // 锁 parse 族 3 源（err-3：vue 门 cap_unused/null_init/event_arg 同根
    // 因修复）；spliced+1=[15]（无 text 叶源经 states-only 编译拼接）；
    // absent+4=新 parse 通源的 handler 族进入缺席分母。**缺席分母语料
    // taxonomy（逐族具名，见规范 §13.13）**：①`$event` 视图实参/元素括号
    // 属性行（`input (placeholder: …)`）等年龄面语法缺口族（语言面收敛
    // 决策，非 UI 适配器可闭）；②float/f64 模型字段与 float 载荷（981 明
    // 确不支撑域——fail-closed）；③双 widget 同文件源（组件面单 node 形
    // 状——IF-981-MODULE 域）；④控制流/let/var/调用 handler 体（组件面
    // 只收平铺赋值——语法层拒，不能静默展平）；⑤空体 handler（无可发射
    // 语句）；⑥引号事件键/handler（`."save"` 形）；⑦store/HTML 片段伪源
    // （提取伪影，非 widget 源）。
    // **2026-11-18 具名修订 #5（切片#6m：年龄面语法收敛——事件绑定调用
    // 形（`.Msg($event)`，UI 侧 $event→nevent 同长归一化）/事件键修饰符
    //（`onclick.self`）/元素括号属性行（`input (placeholder: …)`））**：
    // (75,8,1,27)→**(75, 8, 4, 24)**——spliced+3（[4] modifier 键源、
    // [18]/[59] $event 绑定源——含括号属性形）；absent-3。grammar-id 仅
    // age-component/age-package 两值漂移（具名重基线，981 grammar_scope
    // 测试内注明），其余 13 profile 逐字节不变。
    // **2026-11-18 具名修订 #6（切片#6n：③ node 选择器——多 widget 源逐节
    // 点编译/聚合）**：(75,8,4,24)→**(75, 8, 4, 26)**——absent+2=[65][66]
    // 第二 widget（空体 handler `.DeleteBlock(id) -> { }`）经选择器进入分
    // 母（无可发射语句=缺席，语义正确）；两源第一 widget（载荷声明+参数
    // handler）**仍缺**：widget 头参数表（`widget N(props)`）+ ④ 调用体/`;
    // ` 分隔=下一层收敛（§13.15 边界）。
    // **2026-11-18 具名修订 #7（切片#6o：widget 头参数表（age-node-params
    // 对 age-component 开放，参数=宿主通道 prop）+`;` 单行分隔+调用语句语
    // 法收全（body_foreign fail-closed：检查函数不铸/派发具名拒——
    // a2ts.foreign-unsupported 域合同））**：(75,8,4,26)→**(75,8,7,23)**
    // ——[66] 全源+头部参数源族 +3 spliced；调用表达式形（wide-ops pratt
    // 无调用后缀）=[65] 等源的下一层（表达式面扩展）。
    // **2026-11-18 具名修订 #8（切片#6p：调用表达式形·通道调用承载——
    // "wide-ops 无调用后缀"经探针证伪：pratt 调用后缀在场（`.x = on_x(1)`
    // 解析通过、Bind 拒）；真缺口=通道调用（callee=头参数）无承载+[65]
    // `.Bump` 部分体拼接**静默丢调用语句**+body_foreign 检查函数照铸）**：
    // 聚合 (75,8,7,23) **不变**——[65] 计数已在此前（部分拼接态）计入；
    // 本切片修的是**内容正确性**：`.Bump` 体补全 `emit('Select', next);`
    //（源序 merge）、域外调用 handler 缺席时**兄弟保全**（ops 豁免——
    // 修正前连带全体缺席）、lambda 调用体不铸（不部分拼接）、值位置通道
    // 具名拒。判别力由 `p980_web_channel_call_lane` 六臂断言承担（完整体
    // 文本/兄弟保全/props 形/缺席负控）。
    // **2026-11-18 具名修订 #9（④-A #6 jade-garden 刀：lang 基线前进——
    // 981 G1/G2（on-body hook/view lambda/watch 适配）+980 P3 合并后组件面
    // lane 派生面变化）**：(75,8,7,23)→**(75, 8, 8, 22)**——net spliced+1/
    // absent-1（逐项 A/B 实证：本刀 ui 侧模块 fn 重挂对计数零影响——
    // stash 对照逐项同值；翻转全部来自 lang 侧 lane 派生面（[60] 族进入
    // 拼接、[18] 族移出））。本修订在 lang `010e26f08` 基线上实测。
    assert_eq!(corpus.len(), 83, "corpus size frozen");
    assert_eq!(
        (ok, err, spliced, absent),
        (75, 8, 8, 22),
        "dual-track aggregates frozen"
    );
}

/// WEB 切片 #6p（调用表达式形·通道调用承载）：widget 头参数（宿主通道
/// prop）作 callee 的调用语句——lane 完整体拼接（赋值+通道行按源序），
/// 兄弟保全（ops 豁免），emit/props 双形，域外/值位置 fail-closed。
/// 修正前实况（探针实证）：[65] `.Bump` 部分体拼接、`on_select(next)`
/// **静默丢弃**（检查函数只铸赋值）；域外调用 handler 进 ops=整 lane 编
/// 译失败=兄弟全体缺席。
#[test]
fn p980_web_channel_call_lane() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    // ① [65] 形：赋值+通道行完整体（源序保持；emit 形=on_select 的 Pascal
    // 形 Select ∈ msg 变体——SFC defineEmits 已声明）。
    let src = "widget Counter(on_select: msg) {
    msg Msg { Select(int), Bump(int) }
    model { var n int = 0 }
    view { col { button \"increment\" { onclick: .Bump(.n + 1) } text .n { } } }
    on { .Bump(next) -> { .n = next; on_select(next) } }
}
";
    let (_sfc, bodies, absent) =
        generate_sfc_with_lane(src, "p980-6p.at").expect("channel lane derives");
    assert!(absent.is_empty(), "no absent: {absent:?}");
    let bump = bodies.iter().find(|b| b.pattern == ".Bump").expect("Bump");
    assert_eq!(
        bump.lines,
        vec!["n.value = next;", "emit('Select', next);"],
        "complete body: assignment + channel emit in source order"
    );

    // ② 混合：unit 通道 handler（检查函数道）+ 兄弟 unit handler（状态函
    // 数道）——ops 豁免保全兄弟。
    let mixed = "widget M(on_go: msg) {
    msg Msg { Go, Stop }
    model { var x int = 0 }
    view { col { button \"b\" { } } }
    on { .Go -> { .x = 1; on_go(.x) }
    .Stop -> { .x = 0 } }
}
";
    let (_s, bodies, absent) =
        generate_sfc_with_lane(mixed, "p980-6p.at").expect("mixed lane derives");
    assert!(absent.is_empty(), "sibling preserved: {absent:?}");
    let go = bodies.iter().find(|b| b.pattern == ".Go").expect("Go");
    assert_eq!(go.lines, vec!["x.value = 1;", "emit('Go', x.value);"]);
    let stop = bodies.iter().find(|b| b.pattern == ".Stop").expect("Stop");
    assert_eq!(stop.lines, vec!["x.value = 0;"]);

    // ③ 域外体（非头参数调用，④外部名面族）：缺席 + **兄弟保全**（修正
    // 前：进 ops=编译失败=全体缺席）。
    let foreign = "widget F {
    msg Msg { Go, Stop }
    model { var x int = 0 }
    view { col { button \"b\" { } } }
    on { .Go -> { .x = 1; fs_read(2) }
    .Stop -> { .x = 0 } }
}
";
    let (_s, bodies, absent) =
        generate_sfc_with_lane(foreign, "p980-6p.at").expect("foreign body compiles");
    assert_eq!(absent, vec![".Go"], "foreign handler absent (fail-closed)");
    let stop = bodies.iter().find(|b| b.pattern == ".Stop").expect("Stop");
    assert_eq!(stop.lines, vec!["x.value = 0;"], "sibling still spliced");
    // 判别：缺席 handler 的体不落旧 fallback 形（无 fs_read 伪发射）。
    assert!(
        bodies
            .iter()
            .flat_map(|b| b.lines.iter())
            .all(|l| !l.contains("fs_read")),
        "no fabricated foreign emission"
    );

    // ④ props 形：on_delete 的 Pascal 形（Delete）∉ 变体（DeleteBlock）→
    // 回调 prop 形（老道 PLAN-668 R-23/Plan 444 同约定）。
    let props_form = "widget D(on_delete: msg) {
    msg Msg { DeleteBlock(int) }
    model { var x int = 0 }
    view { col { button \"b\" { } } }
    on { .DeleteBlock(id) -> { .x = id; on_delete(id) } }
}
";
    let (_s, bodies, absent) =
        generate_sfc_with_lane(props_form, "p980-6p.at").expect("props-form derives");
    assert!(absent.is_empty(), "props form: {absent:?}");
    let del = bodies
        .iter()
        .find(|b| b.pattern == ".DeleteBlock")
        .expect("DeleteBlock");
    assert_eq!(
        del.lines,
        vec!["x.value = id;", "props.on_delete(id);"],
        "callback-prop form when no matching variant"
    );

    // ⑤ 值位置（RHS）通道调用：具名拒 → handler 缺席（不做效应作值）。
    let rhs = "widget T(on_select: msg) {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { button \"b\" { } } }
    on { .Go -> { .x = on_select(1) } }
}
";
    match generate_sfc_with_lane(rhs, "p980-6p.at") {
        Ok((_s, bodies, absent)) => {
            assert_eq!(absent, vec![".Go"], "value-position channel absent");
            assert!(bodies.is_empty());
        }
        Err(e) => assert!(!e.is_empty(), "named rejection: {e}"),
    }

    // ⑥ lambda 调用体（单语句形——lambda 体 list-nl，单行多语句=文法边
    // 界）：foreign lambda 不铸 → 缺席（不部分拼接）。
    let lambda_call = "widget L(on_go: msg) {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { button \"b\" { onclick: () => { on_go(.x) } } } }
}
";
    let (_s, bodies, absent) =
        generate_sfc_with_lane(lambda_call, "p980-6p.at").expect("lambda form compiles");
    assert!(
        bodies.is_empty() && absent.len() == 1,
        "foreign lambda absent (not partially spliced): {absent:?}"
    );
    assert!(
        bodies
            .iter()
            .flat_map(|b| b.lines.iter())
            .all(|l| !l.contains("on_go")),
        "no fabricated channel emission from an uncast lambda"
    );
}

/// WEB 切片 #6h：**浏览器挂载档**——lane 派生 handler 体在真实浏览器
/// reactive 宿主中的执行验证（013/017 playwright 先例）。产物：
/// index.html（计数显示+按钮，挂 lane 体+a2x_rt helper 内联）→ 无头
/// chromium 点击 → 计数 41→42（lane 体浏览器语义正确）。
#[test]
fn p980_web_lane_browser_mount() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_web_lane_browser_mount: not scheduled (set AUTO_UI_ACE_WEB_E2E=1; \
             browser tier, playwright required)"
        );
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::lane_handler_bodies;

    // lane 体（002 真源 Inc handler——已由 #6c node 档验证语义）。
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("002: {e}"));
    let (bodies, absent) = lane_handler_bodies(&src, "p980-browser.at", &[".Inc"]).expect("bodies");
    assert!(absent.is_empty(), "Inc handler lane-derived");
    let inc = bodies.iter().find(|b| b.pattern == ".Inc").unwrap();
    let body_js = inc.lines.join("\n");

    // a2x_rt helper 内联——ES module 源不能直接入 `<script>`（export 非法
    // +多行签名）；Inc 只需 a2x_add（checked-add），按 rt_body 冻结语义
    // 手写内联（与 a2x-ts runtime/body.js 同式）。
    let rt_inline = "function a2x_add(a, b) {\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a + b)) throw new RangeError('a2x_add overflow');\n  return a + b;\n}\n";
    let html = format!(
        "<!DOCTYPE html>\n<html><body>\n<div id=\"count\">41</div>\n<button id=\"inc\" onclick=\"Inc()\">+</button>\n<script>\n{rt_inline}\nconst count = {{ value: 41, notify() {{ document.getElementById('count').textContent = String(this.value); }} }};\nfunction Inc() {{\n{body_js}\ncount.notify();\n}}\n</script>\n</body></html>\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-browser-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), html).unwrap();

    // 跑浏览器档脚本。
    let test_mjs = manifest.join("../../examples/ui/ace-web-lane/tests/test_lane_browser.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(test_mjs.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("LANE_BROWSER_OK"),
        "browser: {stdout} / {stderr}"
    );
}

/// WEB 切片#6j（PLAN-981 B-W1 消费，V-EVT-WEB-CONSUME CPU 面）：**param
/// handler 的 lane 参数环境**——声明级检查函数发射（`__age_hv_*` 真形参）
/// + 按位宿主映射。分列：typed/fragment/splice/node + 负控（float 具名缺
/// 席、无参数环境=拒绝拼接常量体）。
#[test]
fn p980_web_param_environment() {
    use auto_ui_runtime::ui::ace_ts_provider::ComponentFragmentLane;
    use auto_ui_runtime::ui::ace_vue_script::{
        generate_sfc_with_lane, lane_handler_bodies_spec, LaneHandlerSpec,
    };

    // 同作者业务源族：QueryInput 实读参（str）、多载荷 (int,int)、bool。
    let q_src = "widget Q {\n    msg Msg { Query(str) }\n\n    model {\n        var last str = \"\"\n    }\n\n    view {\n        col { text \"Q\" }\n    }\n\n    on {\n        .Query(q) -> { .last = q }\n    }\n}\n";
    let m_src = "widget M {\n    msg Msg { Move(int, int) }\n\n    model {\n        var x int = 0\n    }\n\n    view {\n        col { text \"M\" }\n    }\n\n    on {\n        .Move(dx, dy) -> { .x = dx + dy }\n    }\n}\n";
    let b_src = "widget B {\n    msg Msg { SetDark(bool) }\n\n    model {\n        var dark bool = false\n    }\n\n    view {\n        col { text \"B\" }\n    }\n\n    on {\n        .SetDark(on) -> { .dark = on }\n    }\n}\n";
    let f_src = "widget F {\n    msg Msg { Resize(float) }\n\n    model {\n        var size int = 0\n    }\n\n    view {\n        col { text \"F\" }\n    }\n\n    on {\n        .Resize(w) -> { .size = 1 }\n    }\n}\n";
    let q = vec!["q".to_string()];
    let m = vec!["dx".to_string(), "dy".to_string()];
    let b = vec!["on".to_string()];
    let f = vec!["w".to_string()];

    // ① typed：声明级检查函数在视图（981 B-W1 交付面）——span 匹配发现。
    {
        let lane = ComponentFragmentLane::compile(q_src, "p980-6j.at", &[]).expect("lane");
        let ds = lane.desugared_source();
        let arrow = ds.find("-> {").expect("arrow");
        let region = arrow..ds[arrow..].find('}').map(|e| arrow + e + 1).expect("close");
        let check_fn = lane
            .check_fn_covering(region)
            .expect("declared check fn covers the Query handler region");
        assert!(
            check_fn.starts_with("__age_hv_"),
            "check fn name: {check_fn}"
        );
    }

    // ② fragment：str 实读参（owned 转移=a2x_str_clone 单次求值拷贝——
    // 981 PARAMS-B 语义），发射体引用**形参名**而非卫生局部/假值。
    let (q_bodies, q_absent) = {
        let specs = [LaneHandlerSpec {
            pattern: ".Query",
            params: &q,
            lambda_region: None,
        }];
        lane_handler_bodies_spec(q_src, "p980-6j.at", &specs).expect("q bodies")
    };
    assert!(q_absent.is_empty(), "Query lane-derived: {q_absent:?}");
    let q_body = q_bodies.first().expect("q body");
    assert_eq!(q_body.pattern, ".Query");
    assert_eq!(q_body.field, "last");
    assert_eq!(q_body.params, vec!["q".to_string()]);
    let q_text = q_body.lines.join("\n");
    assert!(
        q_text.contains("a2x_str_clone(q)") && q_text.contains("last.value = "),
        "str param read with owned-transfer clone: {q_text}"
    );
    assert!(
        !q_text.contains("___age_hv_")
            && !q_text.contains("___age_s_")
            && !q_text.contains("___age_a_"),
        "no hygienic-local leakage into the host body: {q_text}"
    );

    // ③ 多载荷/bool fragment + splice：SFC 函数签名携形参、体内实读参。
    let (sfc, bodies, absent) = generate_sfc_with_lane(m_src, "p980-6j-m.at").expect("m SFC");
    assert!(absent.is_empty(), "Move spliced: {absent:?}");
    assert_eq!(bodies.len(), 1);
    assert!(
        sfc.contains("function Move(dx: any, dy: any): void {"),
        "SFC signature carries positional params: {}",
        &sfc[sfc.find("function Move").unwrap_or(0)
            ..sfc.len().min(sfc.find("function Move").unwrap_or(0) + 120)]
    );
    assert!(
        sfc.contains("a2x_add(dx, dy)") && sfc.contains("x.value = "),
        "spliced body reads both params: search `Move` fn"
    );
    let (sfc_b, _, absent_b) = generate_sfc_with_lane(b_src, "p980-6j-b.at").expect("b SFC");
    assert!(absent_b.is_empty());
    assert!(
        sfc_b.contains("function SetDark(on: any): void {") && sfc_b.contains("dark.value = on;"),
        "bool param read directly"
    );

    // ④ node：宿主绑定实读参（多载荷 3+4=7；真 a2x_rt helper 就位）。
    let m_body = bodies.first().expect("m body").lines.join("\n");
    let driver = format!(
        "import {{ a2x_add }} from \"./a2x_rt.mjs\";\nconst x = {{ value: 0 }};\nfunction Move(dx, dy) {{\n{m_body}\nreturn x.value;\n}}\nconst r = Move(3, 4);\nif (!(r === 7 && x.value === 7)) {{ throw new Error(\"FRAG_ASSERT_FAILED\"); }}\nconsole.log(\"FRAG_OK\");\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-6j-node-{}", std::process::id()));
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
        "node param env: {stdout} / {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // ⑤ 负控——float 载荷：无检查函数（981 派发守卫同族 fail-closed），
    // 具名缺席（旧道承载），不造假数值。
    {
        let specs = [LaneHandlerSpec {
            pattern: ".Resize",
            params: &f,
            lambda_region: None,
        }];
        let (f_bodies, f_absent) =
            lane_handler_bodies_spec(f_src, "p980-6j.at", &specs).expect("f derives");
        assert!(
            f_bodies.is_empty(),
            "float payload must not fabricate a body"
        );
        assert_eq!(f_absent, vec![".Resize".to_string()], "named absence");
    }

    // ⑥ 负控——无参数环境：param handler 以 unit spec（无形参）求导=**编
    // 译期具名拒**（981 arity 域错误，零效果——不再像 981 前拼接常量体，
    // "spliced=1 不等价动态载荷支持"）。
    {
        let specs = [LaneHandlerSpec {
            pattern: ".Query",
            params: &[],
            lambda_region: None,
        }];
        let err = lane_handler_bodies_spec(q_src, "p980-6j.at", &specs)
            .expect_err("unit spec on a payload variant must be a named arity rejection");
        assert!(
            err.contains("component-event") && err.contains("payload slot"),
            "named arity rejection: {err}"
        );
    }
}

/// WEB 切片#6j 浏览器档（V-EVT-WEB-CONSUME browser 面，排窗
/// `AUTO_UI_ACE_WEB_E2E=1`）：param handler 体在**真实浏览器事件**中实读
/// 参——str 输入事件（QueryInput）、多载荷点击、bool 点击。
#[test]
fn p980_web_param_browser_mount() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_web_param_browser_mount: not scheduled (set AUTO_UI_ACE_WEB_E2E=1; \
             browser tier, playwright required)"
        );
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::{lane_handler_bodies_spec, LaneHandlerSpec};

    let q_src = "widget Q {\n    msg Msg { Query(str) }\n\n    model {\n        var last str = \"\"\n    }\n\n    view {\n        col { text \"Q\" }\n    }\n\n    on {\n        .Query(q) -> { .last = q }\n    }\n}\n";
    let m_src = "widget M {\n    msg Msg { Move(int, int) }\n\n    model {\n        var x int = 0\n    }\n\n    view {\n        col { text \"M\" }\n    }\n\n    on {\n        .Move(dx, dy) -> { .x = dx + dy }\n    }\n}\n";
    let b_src = "widget B {\n    msg Msg { SetDark(bool) }\n\n    model {\n        var dark bool = false\n    }\n\n    view {\n        col { text \"B\" }\n    }\n\n    on {\n        .SetDark(on) -> { .dark = on }\n    }\n}\n";
    let q = vec!["q".to_string()];
    let m = vec!["dx".to_string(), "dy".to_string()];
    let b = vec!["on".to_string()];
    let (q_bodies, a) = {
        let specs = [LaneHandlerSpec {
            pattern: ".Query",
            params: &q,
            lambda_region: None,
        }];
        lane_handler_bodies_spec(q_src, "p980-browser.at", &specs).expect("q")
    };
    assert!(a.is_empty());
    let (m_bodies, a) = {
        let specs = [LaneHandlerSpec {
            pattern: ".Move",
            params: &m,
            lambda_region: None,
        }];
        lane_handler_bodies_spec(m_src, "p980-browser.at", &specs).expect("m")
    };
    assert!(a.is_empty());
    let (b_bodies, a) = {
        let specs = [LaneHandlerSpec {
            pattern: ".SetDark",
            params: &b,
            lambda_region: None,
        }];
        lane_handler_bodies_spec(b_src, "p980-browser.at", &specs).expect("b")
    };
    assert!(a.is_empty());
    let q_body = q_bodies.first().unwrap().lines.join("\n");
    let m_body = m_bodies.first().unwrap().lines.join("\n");
    let b_body = b_bodies.first().unwrap().lines.join("\n");

    // a2x_rt helper 内联（rt_body.js 冻结语义手写——同 #6h a2x_add 先例；
    // a2x_str_clone 在 JS-string 宿主=单次求值拷贝（immutable 值拷贝即本值）；
    // a2x_str_free 同宿主归约=no-op（字节账本无可观察语义；#6l 语句片段带
    // ownership 簿记行）。
    let rt_inline = "function a2x_add(a, b) {\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a + b)) throw new RangeError('a2x_add overflow');\n  return a + b;\n}\nfunction a2x_str_clone(x) {\n  return String(x);\n}\nfunction a2x_str_free(x) {\n}\n";
    let html = format!(
        "<!DOCTYPE html>\n<html><body>\n<div id=\"last\">-</div><div id=\"x\">0</div><div id=\"dark\">false</div>\n<input id=\"q\" oninput=\"Query(this.value)\">\n<button id=\"move\" onclick=\"Move(3, 4)\">move</button>\n<button id=\"dark-btn\" onclick=\"SetDark(true)\">dark</button>\n<script>\n{rt_inline}\nconst last = {{ value: \"\", notify() {{ document.getElementById('last').textContent = this.value; }} }};\nconst x = {{ value: 0, notify() {{ document.getElementById('x').textContent = String(this.value); }} }};\nconst dark = {{ value: false, notify() {{ document.getElementById('dark').textContent = String(this.value); }} }};\nfunction Query(q) {{\n{q_body}\nlast.notify();\n}}\nfunction Move(dx, dy) {{\n{m_body}\nx.notify();\n}}\nfunction SetDark(on) {{\n{b_body}\ndark.notify();\n}}\n</script>\n</body></html>\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-param-browser-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), html).unwrap();

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_mjs =
        manifest.join("../../examples/ui/ace-web-lane/tests/test_lane_browser_params.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(test_mjs.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("LANE_PARAM_BROWSER_OK"),
        "param browser: {stdout} / {stderr}"
    );
}

/// WEB 切片#6k（V-EVT-B-ORIGIN，981 B-W1 联合）：view 内联 lambda 的
/// 真实 body/origin——语言侧铸 `__age_vl_<seq>` 值函数（赋值 RHS 原源
/// span）→ lane `=> {` 区间配对（铸造计数序）→ 发射 → SFC 合成
/// `__evt_*` 体拼接。负控：拼接体**不得**是旧 TS fallback 形
/// （`n.value += 1`——lane 形=a2x_add helper）。
#[test]
fn p980_web_lambda_origin() {
    use auto_ui_runtime::ui::ace_ts_provider::ComponentFragmentLane;
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    let b_src = "widget B {\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"B\" button \"+\" { onclick: () => { .n += 1 } } }\n    }\n}\n";

    // ① typed：值函数在视图（span 相交发现——原 span→checked 函数可追）。
    {
        let lane = ComponentFragmentLane::compile(b_src, "p980-6k.at", &[]).expect("lane");
        let ds = lane.desugared_source();
        let arrow = ds.find("=> {").expect("lambda arrow");
        let open = ds[arrow..].find('{').map(|o| arrow + o).expect("open");
        let region = open..ds[open..].find('}').map(|c| open + c + 1).expect("close");
        let vl = lane
            .lambda_fn_covering(region)
            .expect("__age_vl_ fn covers the lambda region");
        assert!(vl.starts_with("__age_vl_"), "value fn name: {vl}");
    }

    // ② origin→SFC：拼接体=lane 发射形（a2x_add helper），非旧 TS fallback
    // （`n.value += 1`——同语义不同车道；拼接必须来自 checked 函数）。
    let (sfc, bodies, absent) = generate_sfc_with_lane(b_src, "p980-6k.at").expect("SFC");
    assert!(absent.is_empty(), "lambda handler lane-derived: {absent:?}");
    assert_eq!(bodies.len(), 1);
    let body = bodies.first().unwrap();
    assert_eq!(body.pattern, ".__evt_onclick_1");
    let body_text = body.lines.join("\n");
    assert!(
        body_text.contains("a2x_add(n.value, 1)") && body_text.contains("n.value = "),
        "lane-origin body: {body_text}"
    );
    let fn_start = sfc.find("function __evt_onclick_1").expect("synth fn");
    let fn_end = sfc[fn_start..]
        .find('}')
        .map(|e| fn_start + e)
        .unwrap_or(sfc.len());
    let fn_text = &sfc[fn_start..fn_end];
    assert!(
        fn_text.contains("a2x_add") && !fn_text.contains("+= 1"),
        "spliced synth fn carries the lane body (old TS fallback must not stand in): {fn_text}"
    );

    // ③ 002 真源端到端：on 块 6 handler + view 3 内联 lambda 全 lane 派生
    // （`__evt_onclick_{1,2,3}` 语义=dec/reset/inc 对应）。
    {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
        let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("002: {e}"));
        let (sfc, bodies, absent) = generate_sfc_with_lane(&src, "p980-6k-002.at").expect("002");
        assert!(absent.is_empty(), "002 all-lane: {absent:?}");
        assert_eq!(bodies.len(), 9, "002: 6 on-block + 3 view lambdas");
        let find = |pat: &str| {
            bodies.iter().find(|b| b.pattern == pat).unwrap_or_else(|| {
                panic!(
                    "{pat} in {:?}",
                    bodies.iter().map(|b| &b.pattern).collect::<Vec<_>>()
                )
            })
        };
        assert!(
            find(".__evt_onclick_1")
                .lines
                .join("\n")
                .contains("a2x_sub"),
            "lambda 1 = dec semantics"
        );
        assert!(
            find(".__evt_onclick_2")
                .lines
                .join("\n")
                .contains("count.value = 0;"),
            "lambda 2 = reset semantics"
        );
        assert!(
            find(".__evt_onclick_3")
                .lines
                .join("\n")
                .contains("a2x_add"),
            "lambda 3 = inc semantics"
        );
        let _ = &sfc;
    }

    // ④ node：lambda 体宿主执行（n=5 → 6）。
    {
        let node_body = body.lines.join("\n");
        let driver = format!(
            "import {{ a2x_add }} from \"./a2x_rt.mjs\";\nconst n = {{ value: 5 }};\n{node_body}\nif (!(n.value === 6)) {{ throw new Error(\"FRAG_ASSERT_FAILED\"); }}\nconsole.log(\"FRAG_OK\");\n"
        );
        let dir = std::env::temp_dir().join(format!("p980-6k-node-{}", std::process::id()));
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
            "node lambda body: {stdout} / {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// WEB 切片#6k 浏览器档（V-EVT-B-ORIGIN browser 面，排窗
/// `AUTO_UI_ACE_WEB_E2E=1`）：inline-lambda 铸造 handler 体在**真实浏览
/// 器点击**中执行（lane 体=a2x_add 形，非旧 TS fallback）。
#[test]
fn p980_web_lambda_browser_mount() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_web_lambda_browser_mount: not scheduled (set AUTO_UI_ACE_WEB_E2E=1; \
             browser tier, playwright required)"
        );
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    let b_src = "widget B {\n    model {\n        var n int = 5\n    }\n\n    view {\n        col { text \"B\" button \"+\" { onclick: () => { .n += 1 } } }\n    }\n}\n";
    let (sfc, bodies, absent) =
        generate_sfc_with_lane(b_src, "p980-lambda-browser.at").expect("SFC");
    assert!(
        absent.is_empty() && bodies.len() == 1,
        "lane-derived lambda"
    );
    let body_js = bodies.first().unwrap().lines.join("\n");
    let rt_inline = "function a2x_add(a, b) {\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a + b)) throw new RangeError('a2x_add overflow');\n  return a + b;\n}\n";
    let html = format!(
        "<!DOCTYPE html>\n<html><body>\n<div id=\"n\">5</div>\n<button id=\"inc\" onclick=\"__evt_onclick_1()\">+</button>\n<script>\n{rt_inline}\nconst n = {{ value: 5, notify() {{ document.getElementById('n').textContent = String(this.value); }} }};\nfunction __evt_onclick_1() {{\n{body_js}\nn.notify();\n}}\n</script>\n</body></html>\n"
    );
    let _ = &sfc;
    let dir = std::env::temp_dir().join(format!("p980-lambda-browser-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), html).unwrap();

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_mjs =
        manifest.join("../../examples/ui/ace-web-lane/tests/test_lane_browser_lambda.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(test_mjs.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("LANE_LAMBDA_BROWSER_OK"),
        "lambda browser: {stdout} / {stderr}"
    );
}

/// PLAN-980 lambda-if 刀（UI lane 面，CPU 档）：view 内联 lambda 体 if 形
/// 的端到端 lane 派生——语言侧铸 `__age_vl_`（含 If 整语句）→ `=> {` 区
/// 间配对（括号深度容纳分支嵌套）→ `stmt_fragments_in` 根块 If 整语句发
/// 射（分支赋值不裸喷成无条件序列、条件不丢）→ SFC 合成 `__evt_*` 拼接
/// 体。判别器：then 臂复合赋值=lane 形 `a2x_add(n.value, 1)`，旧 TS
/// fallback `n.value += 1` 不得在位（#6k 同族）。负控：分支内调用形
/// lambda 具名缺席（#6p fail-closed 合同）。
#[test]
fn p980_lambda_if_lane() {
    use auto_ui_runtime::ui::ace_ts_provider::ComponentFragmentLane;
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    let src = "widget L {\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"L\" button \"t\" { onclick: () => { if .n == 0 { .n += 1 } else { .n = 0 } } } }\n    }\n}\n";

    // ① typed：if 体值函数区间发现（If cond/RHS span 与 `=> {` 区间相交
    // ——括号深度配对，分支嵌套不截断）+ If 整语句片段（单条，非裸赋值
    // 序列）。
    {
        let lane = ComponentFragmentLane::compile(src, "p980-lif.at", &[]).expect("lane");
        let ds = lane.desugared_source();
        let arrow = ds.find("=> {").expect("lambda arrow");
        let open = ds[arrow..].find('{').map(|o| arrow + o).expect("open");
        let bytes = ds.as_bytes();
        let mut depth = 0usize;
        let mut close = None;
        for (i, &b) in bytes.iter().enumerate().skip(open) {
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    close = Some(i + 1);
                    break;
                }
            }
        }
        let region = open..close.expect("depth-matched close");
        let vl = lane
            .lambda_fn_covering(region.clone())
            .expect("__age_vl_ fn covers the if-lambda region");
        assert!(vl.starts_with("__age_vl_"), "value fn name: {vl}");
        let frags = lane
            .stmt_fragments_in(&vl, region)
            .expect("stmt fragments in the lambda region");
        assert_eq!(frags.len(), 1, "one whole-statement If fragment");
        let text = frags[0].1.text.join("\n");
        assert!(
            text.contains("if ("),
            "branch structure stands in the emitted fragment: {text}"
        );
    }

    // ② origin→SFC：缺席空 + 拼接体=lane 形（a2x_add 判别器 + else 臂平
    // 铺赋值 + 宿主映射 n.value）。
    let (_sfc, bodies, absent) = generate_sfc_with_lane(src, "p980-lif.at").expect("SFC");
    assert!(absent.is_empty(), "if-lambda lane-derived: {absent:?}");
    assert_eq!(bodies.len(), 1);
    let body = bodies.first().unwrap();
    assert_eq!(body.pattern, ".__evt_onclick_1");
    let body_text = body.lines.join("\n");
    assert!(
        body_text.contains("a2x_add(n.value, 1)"),
        "then arm = lane helper form (discriminator vs old fallback): {body_text}"
    );
    assert!(
        body_text.contains("n.value = 0;"),
        "else arm plain assign carries host mapping: {body_text}"
    );
    assert!(
        !body_text.contains("+= 1"),
        "old TS fallback must not stand in: {body_text}"
    );

    // ③ 负控：分支内调用形（值函数不铸——#6p 合同）→ 具名缺席。
    let bad = "widget L {\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"L\" button \"t\" { onclick: () => { if .n == 0 {\n            .n += 1\n            side()\n        } } } }\n    }\n}\n";
    let (_sfc, _bodies, absent) =
        generate_sfc_with_lane(bad, "p980-lif-neg.at").expect("SFC (foreign lambda: old track)");
    assert_eq!(
        absent,
        vec![".__evt_onclick_1".to_string()],
        "call inside a branch: named absence, no partial body"
    );
}

/// PLAN-980 lambda-if 刀浏览器档（排窗 `AUTO_UI_ACE_WEB_E2E=1`）：if 体
/// lambda 的 lane 派生拼接在真实浏览器点击中执行——toggle 语义（then 臂
/// a2x_add 增、else 臂清零），两分支都被真实走到。
#[test]
fn p980_lambda_if_browser_mount() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_lambda_if_browser_mount: not scheduled (set AUTO_UI_ACE_WEB_E2E=1; \
             browser tier, playwright required)"
        );
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    let src = "widget L {\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"L\" button \"t\" { onclick: () => { if .n == 0 { .n += 1 } else { .n = 0 } } } }\n    }\n}\n";
    let (_sfc, bodies, absent) = generate_sfc_with_lane(src, "p980-lif-browser.at").expect("SFC");
    assert!(
        absent.is_empty() && bodies.len() == 1,
        "lane-derived if-lambda"
    );
    let body_js = bodies.first().unwrap().lines.join("\n");
    let rt_inline = "function a2x_add(a, b) {\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a + b)) throw new RangeError('a2x_add overflow');\n  return a + b;\n}\n";
    let html = format!(
        "<!DOCTYPE html>\n<html><body>\n<div id=\"n\">0</div>\n<button id=\"toggle\" onclick=\"__evt_onclick_1()\">t</button>\n<script>\n{rt_inline}\nconst n = {{ value: 0, notify() {{ document.getElementById('n').textContent = String(this.value); }} }};\nfunction __evt_onclick_1() {{\n{body_js}\nn.notify();\n}}\n</script>\n</body></html>\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-lif-browser-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), html).unwrap();

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_mjs =
        manifest.join("../../examples/ui/ace-web-lane/tests/test_lane_browser_lambda_if.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(test_mjs.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("LANE_LAMBDA_IF_BROWSER_OK"),
        "lambda-if browser: {stdout} / {stderr}"
    );
}

/// 复核 authored 反例（#6j/#6k 批级复核，永久）：PARAMS-B 裸形参遮蔽同拼
/// 写 model 字段——`.Set(n) -> { .n = n }`：宿主映射 `field.value` 的基名
/// 会被 SFC 函数形参遮蔽（node 实证 `n.value = n` 在形参数字上写属性=
/// TypeError；ACE 语义「`.field` 恒指实例字段」在 TS 宿主**不可表达**）→
/// **具名缺席**（F-1 修复=fail-closed 旧道承载），不生成错误体。
#[test]
fn p980_web_param_shadowing_is_param_not_field() {
    use auto_ui_runtime::ui::ace_vue_script::{lane_handler_bodies_spec, LaneHandlerSpec};
    let src = "widget S {\n    msg Msg { Set(int) }\n\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"S\" }\n    }\n\n    on {\n        .Set(n) -> { .n = n }\n    }\n}\n";
    let p = vec!["n".to_string()];
    let specs = [LaneHandlerSpec {
        pattern: ".Set",
        params: &p,
        lambda_region: None,
    }];
    let (bodies, absent) =
        lane_handler_bodies_spec(src, "p980-shadow.at", &specs).expect("shadow derives");
    assert!(
        bodies.is_empty(),
        "param/field name collision must not fabricate a shadowed-broken body"
    );
    assert_eq!(absent, vec![".Set".to_string()], "named absence (F-1)");
}

/// WEB 切片#6l（真语料全量双轨·V-B-WEB 处置）：①text-leaf 形状门只约束
/// 文本观测车道——states-only 编译无 `text` 叶视图（button/input-only 真
/// 实工程形态）可编译可拼接；②方言 `state` 槽位别名（=model，vue 门
/// parse 族预存红根因修复）；③多赋值 handler 体**逐语句顺序发射**（语句
/// 序=源序，[57] 七连赋值族；控制流/let/调用在语法层被拒=不可能静默展
/// 平）。node 判别：完整七句执行后终值可区分（丢句即红）。
#[test]
fn p980_web_textless_state_alias_multi_assign() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    // ①+②：无 text 叶 + `state {}` 拼写（方言别名）+ 单赋值 unit handler。
    let t_src = "widget T {\n    msg Msg { Go }\n    state { var x int = 0 }\n    view { col { button \"go\" { onclick: .Go } } }\n    on { .Go -> { .x = 1 } }\n}\n";
    let (sfc, bodies, absent) =
        generate_sfc_with_lane(t_src, "p980-6l.at").expect("textless+state derives");
    assert!(
        absent.is_empty(),
        "text-less view splices via states-only: {absent:?}"
    );
    assert_eq!(bodies.len(), 1);
    assert!(sfc.contains("x.value = 1;"), "unit handler body spliced");

    // ③：多赋值顺序体（int 域七连——除/模；[57] 族的 int 化形）。
    let m_src = "widget M {
    msg Msg { Go }
    model {
        var a int = 84
        var b int = 2
        var c int = 3
        var d int = 0
        var e int = 0
        var f int = 0
    }
    view { col { text \"M\" button \"go\" { onclick: .Go } } }
    on {
        .Go -> {
            .d = .a / .b
            .e = .a % .c
            .f = .d + .e
        }
    }
}
";
    let (sfc, bodies, absent) =
        generate_sfc_with_lane(m_src, "p980-6l-m.at").expect("multi derives");
    assert!(absent.is_empty(), "multi-assign body splices: {absent:?}");
    assert_eq!(bodies.len(), 1);
    let body = bodies.first().unwrap();
    let text = body.lines.join(
        "
",
    );
    assert!(
        text.contains("a2x_div(a.value, b.value)") && text.contains("a2x_rem(a.value, c.value)"),
        "division/modulo helpers emitted: {text}"
    );
    // 语句序=源序：d 在 e 前、f 最后（按行序断言）。
    let d_pos = text.find("d.value = ").expect("d assign");
    let e_pos = text.find("e.value = ").expect("e assign");
    let f_pos = text.find("f.value = ").expect("f assign");
    assert!(
        d_pos < e_pos && e_pos < f_pos,
        "source order preserved: {text}"
    );
    let _ = &sfc;

    // node 判别：完整执行 a=84,b=2,c=3 → d=42, e=0, f=42（丢任一句即红）。
    let driver = format!(
        "import {{ a2x_div, a2x_rem, a2x_add }} from \"./a2x_rt.mjs\";\nconst a = {{ value: 84 }};\nconst b = {{ value: 2 }};\nconst c = {{ value: 3 }};\nconst d = {{ value: 0 }};\nconst e = {{ value: 0 }};\nconst f = {{ value: 0 }};\nfunction Go() {{\n{text}\n}}\nGo();\nif (!(d.value === 42 && e.value === 0 && f.value === 42)) {{ throw new Error(\"FRAG_ASSERT_FAILED\"); }}\nconsole.log(\"FRAG_OK\");\n"
    );
    let dir = std::env::temp_dir().join(format!("p980-6l-node-{}", std::process::id()));
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
        "multi-assign node leg: {stdout} / {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// WEB 切片#6m（年龄面语法收敛，V-B-WEB 处置续）：事件绑定调用形
/// （`.QueryInput($event)`——UI 侧 `$event`→`nevent` 同长归一化+语言面
/// age-event-binding 调用形）、事件键修饰符（`onclick.self`）、元素括号
/// 属性行（`input (placeholder: …)`）——三形真语料主导形态经组件面编译
/// 并 lane 拼接（spliced=1 各形）。
#[test]
fn p980_web_event_arg_forms() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;
    let cases: Vec<(&str, String)> = vec![
        (
            "evt-binding",
            "widget P {
    msg Msg { QueryInput(str) }
    state { var query str = \"\" }
    view {
        input { value: .query, oninput: .QueryInput($event) }
    }
    on {
        .QueryInput(v) -> { .query = v }
    }
}
"
            .to_string(),
        ),
        (
            "paren-props",
            "widget P {
    msg Msg { QueryInput(str) }
    state { var query str = \"\" }
    view {
        input (placeholder: \"search\") { oninput: .QueryInput($event) }
    }
    on {
        .QueryInput(v) -> { .query = v }
    }
}
"
            .to_string(),
        ),
        (
            "key-modifier",
            "widget W {
    msg Msg { Cancel }
    state { var x int = 0 }
    view {
        col { button \"x\" { onclick.self: .Cancel } }
    }
    on {
        .Cancel -> { .x = 0 }
    }
}
"
            .to_string(),
        ),
    ];
    for (name, src) in cases {
        let (sfc, bodies, absent) =
            generate_sfc_with_lane(&src, "p980-6m.at").expect("{name} derives");
        assert!(absent.is_empty(), "{name}: lane-derived: {absent:?}");
        assert_eq!(bodies.len(), 1, "{name}: one handler spliced");
        let body = bodies.first().unwrap();
        assert!(
            body.lines.iter().any(|l| l.contains(".value = ")),
            "{name}: assignment emitted"
        );
        let _ = &sfc;
    }
}

/// WEB 切片#6n（③ 双 widget 源·node 选择器）：多 widget 同文件源按声明
/// 序逐节点编译/lane 拼接并**聚合**（语言面 `compile_component_scenario_
/// states_at`——"选一编译"，无组合/跨引用语义=IF-981-MODULE 域边界保持）。
/// 负控：越界 node_index=编译期具名拒。
#[test]
fn p980_web_multi_widget_node_selector() {
    use auto_ui_runtime::ui::ace_ts_provider::ComponentFragmentLane;
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    let src = "widget A {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { text \"a\" button \"go\" { onclick: .Go } } }
    on { .Go -> { .x = 1 } }
}

widget B {
    msg Msg { Run }
    model { var y int = 0 }
    view { col { text \"b\" button \"run\" { onclick: .Run } } }
    on { .Run -> { .y = 2 } }
}
";

    // ① lane 逐节点：node0 的 handler 体发射 x；node1 发射 y（选择器正确
    // 分离同形规则）。
    let l0 = ComponentFragmentLane::compile_at(src, "p980-6n.at", &["Go"], 0)
        .unwrap_or_else(|e| panic!("node0: {e}"));
    let r0 = l0
        .stmt_fragments_in("__age_state_0_x", {
            let ds = l0.desugared_source();
            let a = ds.find(".Go ->").expect("go arrow");
            let o = ds[a..].find('{').unwrap() + a;
            o..ds[o..].find('}').unwrap() + o + 1
        })
        .expect("node0 stmts");
    assert!(!r0.is_empty(), "node0 .Go assigns emitted");
    let l1 = ComponentFragmentLane::compile_at(src, "p980-6n.at", &["Run"], 1)
        .unwrap_or_else(|e| panic!("node1: {e}"));
    let r1 = l1
        .stmt_fragments_in("__age_state_0_y", {
            let ds = l1.desugared_source();
            let a = ds.find(".Run ->").expect("run arrow");
            let o = ds[a..].find('{').unwrap() + a;
            o..ds[o..].find('}').unwrap() + o + 1
        })
        .expect("node1 stmts");
    assert!(!r1.is_empty(), "node1 .Run assigns emitted");

    // ② 一体化入口聚合：双 widget 各自 handler 全 lane 派生。
    let (_sfc, bodies, absent) = generate_sfc_with_lane(src, "p980-6n.at").expect("multi derives");
    assert!(absent.is_empty(), "both widgets lane-derived: {absent:?}");
    assert_eq!(bodies.len(), 2, "aggregated across nodes");
    assert!(bodies.iter().any(|b| b.field == "x") && bodies.iter().any(|b| b.field == "y"));

    // ③ 负控：越界 node_index=具名拒。
    let err = match ComponentFragmentLane::compile_at(src, "p980-6n.at", &[], 2) {
        Err(e) => e,
        Ok(_) => panic!("out-of-range node index must be a named rejection"),
    };
    assert!(err.contains("node index 2"), "named: {err}");
}

/// WEB 切片#6o（④语法收全·fail-closed 承载）：widget 头参数表（宿主通道
/// prop）+`;` 分隔+多赋值经一体入口拼接；**调用体=具名缺席**（body_foreign
/// ——检查函数不铸/派发具名拒，a2ts.foreign-unsupported 域合同；不做未验
/// 证旁路）。负控：调用体 handler 不产 lane 体（旧道承载）。
#[test]
fn p980_web_head_props_semi_and_foreign_body_fail_closed() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

    // ① 头参数表 + `;` 分隔多赋值：全 lane 拼接。
    let src = "widget App(on_delete: msg) {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { text \"t\" button \"go\" { onclick: .Go } } }
    on { .Go -> { .x = 1; .x = 2 } }
}
";
    let (sfc, bodies, absent) =
        generate_sfc_with_lane(src, "p980-6o.at").expect("head-props derives");
    assert!(absent.is_empty(), "head-props + semi spliced: {absent:?}");
    assert_eq!(bodies.len(), 1);
    let text = bodies.first().unwrap().lines.join(
        "
",
    );
    assert!(
        text.contains("x.value = 1") && text.contains("x.value = 2"),
        "both: {text}"
    );
    let _ = &sfc;

    // ② 负控：调用体（on_select(1)）=具名缺席（fail-closed 旧道，不伪造）。
    let f_src = "widget T {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { text \"t\" } }
    on { .Go -> { .x = on_select(1) } }
}
";
    match generate_sfc_with_lane(f_src, "p980-6o.at") {
        Ok((_s, bodies, absent)) => {
            // 调用在 RHS 表达式内——pratt 无调用后缀=语法层拒（编译失败→全
            // 缺席）或 body_foreign 缺席；两形都不产 lane 体。
            assert!(
                bodies.iter().all(|b| b.pattern != ".Go" || !b.lines.iter().any(|l| l.contains("on_select"))),
                "no fabricated foreign call emission"
            );
            let _ = absent;
        }
        Err(e) => assert!(!e.is_empty(), "named rejection"),
    }
}

/// PLAN-980 on-if 刀：param handler 的 if 体**结构化派生**（010 的
/// `.SetTheme` 形）——检查函数语句整体发射（带 `if (…)` 分支结构，非扁平
/// 无条件序列）、条件经 a2x_str_eq（Str==str 放行）、宿主映射（形参 `t`、
/// `dark_mode.value`）。
#[test]
fn p980_on_if_param_handler_if_body_derives_structured() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;
    let src = "widget W {
    msg { SetTheme(str) }
    model { var dark_mode bool = true }
    view { col { text \"T\" } }
    on {
        .SetTheme(t) -> {
            if t == \"light\" {
                .dark_mode = false
            }
            if t == \"dark\" {
                .dark_mode = true
            }
        }
    }
}
";
    let (sfc, bodies, absent) = generate_sfc_with_lane(src, "p980-onif.at").expect("SFC");
    assert!(absent.is_empty(), "if-body param handler derives: {absent:?}");
    assert_eq!(bodies.len(), 1);
    let text = bodies[0].lines.join("
");
    assert!(
        text.contains("a2x_str_eq(t, a2x_str_lit(\"light\"))"),
        "str comparison condition emits via a2x_str_eq with the mapped param: {text}"
    );
    assert!(text.contains("if ("), "branch structure is emitted: {text}");
    assert!(
        text.contains("dark_mode.value"),
        "field host mapping applies inside branches: {text}"
    );
    let at = sfc.find("function SetTheme").expect("spliced fn");
    let tail = &sfc[at..(at + 700).min(sfc.len())];
    assert!(
        tail.contains("if (") && tail.contains("a2x_str_eq"),
        "spliced handler carries the structured branch body"
    );
}
