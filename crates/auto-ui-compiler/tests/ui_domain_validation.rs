use auto_ui_compiler::ui_gen::api::{
    generate_component_from_file, transpile_aura, transpile_vue_aura, ComponentGenOptions,
};
use auto_ui_compiler::{lower_ui, parse_ui};

const GOOD: &str = "widget App { view { text \"ok\" {} } }";

#[test]
fn public_generators_reject_invalid_domain_before_writing() {
    let bad = format!("{GOOD}\n{GOOD}\n");
    let parsed = parse_ui(&bad).expect("duplicate declarations are syntactically valid");
    assert!(lower_ui(&parsed)
        .expect_err("lowering rejects duplicate widgets")
        .to_string()
        .contains("duplicate widget declaration"));

    let dir = tempfile::tempdir().expect("tempdir");
    let rust_out = dir.path().join("App.rs");
    let vue_out = dir.path().join("App.vue");
    for result in [
        transpile_aura(&bad, rust_out.to_str()),
        transpile_vue_aura(&bad, vue_out.to_str()),
    ] {
        let error = result.expect_err("public generator must reject invalid domain data");
        assert!(error.contains("UI domain validation failed"), "{error}");
        assert!(error.contains("duplicate widget declaration"), "{error}");
    }
    assert!(!rust_out.exists());
    assert!(!vue_out.exists());
    assert!(transpile_aura(GOOD, None).is_ok());
    assert!(transpile_vue_aura(GOOD, None).is_ok());
}

#[test]
fn file_generation_rejects_invalid_domain_families() {
    let cases = [
        (
            "duplicate widget",
            format!("{GOOD}\n{GOOD}\n"),
            "duplicate widget declaration",
        ),
        (
            "duplicate store",
            "store Shared { model { count int = 0 } }\nstore Shared { model { count int = 0 } }"
                .into(),
            "duplicate store declaration",
        ),
        (
            "duplicate model field",
            "widget App {\n model { count int = 0\n count int = 1 }\n view { text \"ok\" {} }\n}"
                .into(),
            "duplicate model field",
        ),
        (
            "duplicate message variant",
            "widget App {\n msg { Go\n Go }\n view { text \"ok\" {} }\n}".into(),
            "duplicate message variant",
        ),
    ];
    for (label, source, expected) in cases {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("app.at");
        std::fs::write(&path, source).expect("source");
        let error = generate_component_from_file(&path, ComponentGenOptions::default())
            .expect_err(label)
            .to_string();
        assert!(
            error.contains("UI domain validation failed"),
            "{label}: {error}"
        );
        assert!(error.contains(expected), "{label}: {error}");
    }
}

#[test]
fn file_generation_validates_cross_file_modules() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("child.at");
    let app = dir.path().join("app.at");
    let valid_child = "widget Imported { view { text \"child\" {} } }";
    std::fs::write(
        &app,
        "use child: Imported\nwidget App { view { Imported {} } }",
    )
    .expect("app source");
    std::fs::write(&module, format!("{valid_child}\n{valid_child}"))
        .expect("invalid module source");
    let options = ComponentGenOptions {
        sub_widgets: Some(vec!["Imported".into()]),
        ..Default::default()
    };
    let error = generate_component_from_file(&app, options.clone())
        .expect_err("invalid imported module must fail generation");
    assert!(error.contains("UI module validation failed"), "{error}");
    assert!(error.contains("duplicate widget declaration"), "{error}");

    std::fs::write(&module, valid_child).expect("valid module source");
    generate_component_from_file(&app, options).expect("valid imported module still generates");
}

// ---------------------------------------------------------------------------
// PLAN-980（B-DIALECT-I/V，IF-980-FIELDS）永久 case：
// `p980_required_fields_handoff` / `p980_required_field_mapping_controls`。
// 完整 WidgetDecl 字段分母（auto-ui-dialect::p980_fields）× 真实 002-counter
// 原源（含 sha256 冻结身份）× 共享字段级覆盖/清单机制（ace-core
// ::source::field_manifest，接既有 mapping_complete/component_coverage）。
// ---------------------------------------------------------------------------

use ace_core::source::domain::DomainVersion;
use ace_core::source::field_manifest::{
    field_coverage, validate_field_manifest, FieldApplicability as CoreApplicability,
    FieldManifest, FieldPresented, RequiredFieldRow,
};
use auto_ui_dialect::p980_fields::{
    reconcile_field_manifest, widget_field_manifest, widget_field_occurrences,
    widget_field_occurrences_with, FieldOccurrence, WidgetFieldApplicability, WidgetFieldRow,
    P980_CONSTRUCTS, P980_FIELD_MANIFEST_DOMAIN, P980_FIELD_MANIFEST_VERSION,
};
use sha2::{Digest, Sha256};

const SRC_002: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/ui/002-counter/src/front/app.at"
);
/// 冻结身份（2026-11-16 读取）：examples/ui/002-counter/src/front/app.at
/// sha256 前 16 hex（与 PLAN-980 r10 §4 记录的 `04937c35325faa74` 一致）。
const SRC_002_SHA16: &str = "04937c35325faa74";

fn sha16(bytes: &[u8]) -> String {
    let d = Sha256::digest(bytes);
    let hex: String = d.iter().map(|b| format!("{b:02x}")).collect();
    hex[..16].to_string()
}

/// UI 清单行 → 共享机制行（语义字段一一对应；AST 无 span 的槽位 span=(0,0)）。
fn to_core_rows(rows: &[WidgetFieldRow]) -> Vec<RequiredFieldRow> {
    rows.iter()
        .map(|r| RequiredFieldRow {
            id: r.id.to_string(),
            origin: r.origin.clone(),
            source_span: r
                .source_span
                .map(|(s, e)| ace_core::atom_text::Span {
                    start: s,
                    end: s + e,
                })
                .unwrap_or(ace_core::atom_text::Span { start: 0, end: 0 }),
            parent_role: r.parent_role.to_string(),
            type_rule: r.type_rule.to_string(),
            applicability: match &r.applicability {
                WidgetFieldApplicability::Required { mode } => {
                    CoreApplicability::Required { mode: mode.clone() }
                }
                WidgetFieldApplicability::Optional { mode } => {
                    CoreApplicability::Optional { mode: mode.clone() }
                }
                WidgetFieldApplicability::Conditional { mode, reason } => {
                    CoreApplicability::Conditional {
                        mode: mode.clone(),
                        reason: reason.clone(),
                    }
                }
            },
            projection_path: r.projection_path.to_string(),
            check_id: r.check_id.to_string(),
            consumer_owner: r.consumer_owner.to_string(),
        })
        .collect()
}

fn core_manifest() -> FieldManifest {
    let rows = widget_field_manifest();
    FieldManifest {
        domain: P980_FIELD_MANIFEST_DOMAIN.to_string(),
        version: DomainVersion {
            major: P980_FIELD_MANIFEST_VERSION.0,
            minor: P980_FIELD_MANIFEST_VERSION.1,
        },
        rows: to_core_rows(&rows),
    }
}

fn parse_002() -> (String, auto_ui_dialect::UiCode) {
    let src = std::fs::read_to_string(SRC_002).expect("002-counter fixture");
    assert_eq!(
        sha16(src.as_bytes()),
        SRC_002_SHA16,
        "frozen fixture identity"
    );
    let ui = parse_ui(&src).expect("002-counter parses through the UI dialect");
    (src, ui)
}

fn count(occ: &[FieldOccurrence], id: &str) -> usize {
    occ.iter().filter(|o| o.id == id).count()
}

fn presented(occ: &[FieldOccurrence]) -> Vec<FieldPresented> {
    occ.iter()
        .map(|o| FieldPresented {
            id: o.id.to_string(),
            span: o
                .span
                .map(|(s, e)| ace_core::atom_text::Span {
                    start: s,
                    end: s + e,
                })
                .unwrap_or(ace_core::atom_text::Span { start: 0, end: 0 }),
        })
        .collect()
}

#[test]
fn p980_required_fields_handoff() {
    let (src, ui) = parse_002();

    // 1) 真实原源 → 穷尽出现集（每个实际构造一个出现；未知构造会具名拒绝）。
    let occ = widget_field_occurrences(&ui).expect("every 002-counter construct maps");

    // 2) 清单可执行：共享机制自检通过；清单 ↔ 映射实现双向对账通过。
    let manifest = core_manifest();
    validate_field_manifest(&manifest).expect("the field manifest is executable");
    let ui_rows = widget_field_manifest();
    reconcile_field_manifest(&ui_rows, P980_CONSTRUCTS)
        .expect("manifest and mapping implementation reconcile");

    // 3) 共享覆盖检查（接既有 mapping_complete/component_coverage 的字段粒度层）：
    //    必需行（002-counter 锚定）全覆盖、无未知构造。
    let cov = field_coverage(&presented(&occ), &manifest);
    assert!(
        cov.is_complete(),
        "coverage must close on the 002 anchor: {cov:?}"
    );
    assert_eq!(
        cov.required.len(),
        16,
        "the 002-counter required subset: {cov:?}"
    );

    // 4) 独立 oracle（fixture 事实，非实现回声）：槽位计数与探针冻结值一致。
    let facts: &[(&str, usize)] = &[
        ("widget.msg", 1),
        ("widget.msg.variant", 9), // 6 声明 + 3 内联 lambda 铸造的 __evt
        ("widget.model", 1),
        ("widget.model.field", 3),
        ("widget.model.field.var", 3), // var count/theme_name/dark_mode
        ("widget.design", 1),
        ("widget.design.entry", 2), // theme + mode
        ("widget.on", 1),
        ("widget.on.handler", 9),      // 6 声明 + 3 铸造
        ("widget.on.handler.span", 6), // PLAN-980 WEB 切片#6：on 块解析捕获（铸造形无 span）
        ("view.node.element", 10),     // col + text + 2×row + 6×button
        ("view.prop", 11),             // col.style + text.text + 3×1 + 3×2
        ("view.event", 6), // 6 onclick（3 个内联 lambda 在 parse 期铸造为 __evt handler）
    ];
    for (id, expected) in facts {
        assert_eq!(count(&occ, id), *expected, "occurrence count for {id}");
    }

    // 5) 原 span 验证（view 树为真实字节区间）：元素出现携带的 (offset,len)
    //    与探针冻结值逐一相等（2026-11-16 lang-980 worktree 对 sha 冻结源的
    //    实测；span 覆盖子元素区到闭括号）。
    let mut spans: Vec<(usize, usize)> = occ
        .iter()
        .filter(|o| o.id == "view.node.element" && o.span.is_some())
        .map(|o| {
            let (s, l) = o.span.unwrap();
            let _body = &src[s..s + l]; // 边界/长度有效（panic 即红）
            (s, l)
        })
        .collect();
    spans.sort_unstable();
    let frozen: Vec<(usize, usize)> = vec![
        (917, 727),  // col（外壳）
        (935, 25),   // text `Counter: ${.count}`
        (973, 202),  // row（-/Reset/+）
        (995, 43),   // button "-"
        (1055, 46),  // button "Reset"
        (1118, 43),  // button "+"
        (1188, 402), // row（主题三键）
        (1210, 103), // button "Auto"
        (1330, 117), // button "Solarized"
        (1464, 112), // button "切换明暗"
    ];
    assert_eq!(
        spans, frozen,
        "element spans must equal the probe-frozen bytes"
    );

    // 6) 块级槽位独立定位器（行注释剥离后的锚点顺序 = 声明顺序）。
    let stripped: String = src
        .lines()
        .map(|l| match l.find("//") {
            Some(at) => &l[..at],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let anchors = [
        "use themes",
        "widget App",
        "msg {",
        "model {",
        "design {",
        "view {",
        "on {",
    ];
    let offsets: Vec<usize> = anchors
        .iter()
        .map(|a| {
            stripped
                .find(a)
                .unwrap_or_else(|| panic!("anchor {a} missing"))
        })
        .collect();
    assert!(
        offsets.windows(2).all(|w| w[0] < w[1]),
        "block anchors must appear in declaration order: {offsets:?}"
    );

    // 7) 普通 Node 迁移副本（只改声明头）：字节恒等除头 token；themes/model/design
    //    等原业务不在清单/副本中被裁掉（design/model 行仍在第 3 步覆盖内）。
    let head_at = src
        .find("widget App")
        .expect("widget head (first occurrence = real decl)");
    let node_copy = format!(
        "{}node App{}",
        &src[..head_at],
        &src[head_at + "widget App".len()..]
    );
    assert_eq!(node_copy.len(), src.len() - ("widget".len() - "node".len()));
    assert_eq!(&src[..head_at], &node_copy[..head_at]);
    assert_eq!(
        &src[head_at + "widget App".len()..],
        &node_copy[head_at + "node App".len()..]
    );
    // `node` 规范头经 AGE 组件面同一字段 ID 连接（语言侧
    // p980_age_face_legacy_widget_alias_head_join 永久核对；UI 面解析 `widget`）。
}

#[test]
fn p980_required_field_mapping_controls() {
    let (_src, ui) = parse_002();

    // 反控 A：删一项映射实现（构造映射被删）→ 抽取具名拒绝，错误指名构造；
    // 恢复后同一入口绿。与解析错误（反控 C）可区分：code=ui.field-unknown。
    let reduced: Vec<&'static str> = P980_CONSTRUCTS
        .iter()
        .copied()
        .filter(|c| *c != "widget.msg")
        .collect();
    let reduced: &'static [&'static str] = Box::leak(reduced.into_boxed_slice());
    let err = widget_field_occurrences_with(&ui, reduced)
        .expect_err("a deleted mapping entry must reject, not silently drop");
    assert_eq!(err.code, "ui.field-unknown");
    assert_eq!(err.construct, "widget.msg");
    widget_field_occurrences(&ui).expect("restored mapping is green");

    // 反控 B：删一条必需清单行 → 双向对账红（映射实现仍认识该构造）；
    // 共享覆盖上该 ID 变 unknown（红）。恢复后绿。
    let mut rows = widget_field_manifest();
    rows.retain(|r| r.id != "widget.design.entry");
    let problems = reconcile_field_manifest(&rows, P980_CONSTRUCTS)
        .expect_err("deleting a manifest row must be caught by reconciliation");
    assert!(
        problems.iter().any(|p| p.contains("widget.design.entry")),
        "{problems:?}"
    );
    let occ = widget_field_occurrences(&ui).expect("occurrences still map");
    let mut without_row = core_manifest();
    without_row.rows.retain(|r| r.id != "widget.design.entry");
    let cov = field_coverage(&presented(&occ), &without_row);
    assert_eq!(cov.unknown, vec!["widget.design.entry".to_string()]);
    assert!(!cov.is_complete());
    let cov = field_coverage(&presented(&occ), &core_manifest());
    assert!(cov.is_complete(), "restored row is green: {cov:?}");

    // 反控 C：同类型非法父槽（design 的 theme: 绑定写进 model 块）——共享前门
    // （UI 方言解析）具名拒绝并带位置；不是静默忽略。
    let bad = "widget A {\n    model {\n        theme: .theme_name\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let err = parse_ui(bad).expect_err("a theme binding is not a model field");
    let msg = err.to_string();
    assert!(
        msg.contains("theme") || msg.contains("Expected") || msg.contains("expected"),
        "the rejection must name the offending construct: {msg}"
    );

    // 修复后同入口恢复（同类型槽各归其位：theme 回 design 块，model 块为合法
    //    字段；fixture 补齐 002 锚定必需面 msg/on/event）。
    let fixed = "widget A {\n    msg { Inc }\n    model {\n        var theme_name str = \"Auto\"\n    }\n    design {\n        theme: .theme_name\n    }\n    view {\n        button \"Go\" { onclick: .Inc }\n    }\n    on {\n        .Inc -> { .theme_name = \"B\" }\n    }\n}\n";
    let ui_fixed = parse_ui(fixed).expect("recovered source parses");
    let occ = widget_field_occurrences(&ui_fixed).expect("recovered occurrences map");
    let cov = field_coverage(&presented(&occ), &core_manifest());
    assert!(
        cov.unknown.is_empty() && cov.missing.is_empty(),
        "recovered fixture is green: {cov:?}"
    );
}
