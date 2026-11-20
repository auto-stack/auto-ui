//! PLAN-980 ④-A #3：**AGE `view` 树 → Vue SFC** 的生成与真实挂载。
//!
//! 判据：①同一份源生成的 SFC 含模板（按钮/事件绑定）与脚本（refs/handler/helper/
//! `defineExpose` 状态）；②真实 Vue 挂载后点击 `+`/`Reset` 得 `0 → 1 → 0`（2 次 `+`
//! 得 2）；③未改写 `002-counter` 的**首个实际阻断**被具名记录（不静默）。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_sfc::generate_package_sfc;

fn unit(pkg: &str, id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: pkg.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

/// 派生真实例（0 基计数器 + 跨模块 helper `inc1`）：`+` 走 helper、`-` 内联减一、
/// `Reset` 归零——模板由 view 树生成，脚本由包片段 lane 发射。
fn counter_src() -> String {
    "use util.steps: inc1\n\nnode Counter {\n    msg { Inc, Dec, Reset }\n\n    model {\n        var count int = 0\n    }\n\n    view {\n        col {\n            text \"Counter\"\n            row {\n                button \"Reset\" { onclick: .Reset }\n                button \"-\" { onclick: .Dec }\n                button \"+\" { onclick: .Inc }\n            }\n        }\n    }\n\n    on {\n        .Inc -> { .count = inc1(.count) }\n        .Dec -> { .count = .count - 1 }\n        .Reset -> { .count = 0 }\n    }\n}\n"
        .to_string()
}

const HELPER: &str = "pub fn inc1(n int) int {\n    return n + 1\n}\n";

fn gen() -> String {
    generate_package_sfc(
        &[
            unit("app", "main", &counter_src()),
            unit("util", "steps", HELPER),
        ],
        "app::main",
    )
    .expect("derived counter SFC generates")
}

#[test]
fn p125_package_sfc_progress_preserves_percent_scale_and_bound_value() {
    let source = r#"
widget ProgressView {
    model { var current int = 3 }
    view {
        col {
            progress { value: "33" }
            progress {
                value: .current
                max: 12
            }
            progress {}
        }
    }
}
"#;
    let sfc = generate_package_sfc(&[unit("app", "main", source)], "app::main")
        .expect("progress widget generates through the package route");
    for expected in [
        r#"<progress value="33" max="100"></progress>"#,
        r#"<progress :value="current" max="12"></progress>"#,
        r#"<progress value="0" max="100"></progress>"#,
    ] {
        assert!(sfc.contains(expected), "missing {expected}: {sfc}");
    }
}

#[test]
fn p125_package_sfc_slider_emits_native_range_with_constraints() {
    let source = r#"
widget SliderView {
    view {
        col {
            slider {
                min: "0"
                max: "100"
                value: "50"
            }
            slider {
                min: 2
                max: 20
                step: 3
                value: 8
            }
        }
    }
}
"#;
    let sfc = generate_package_sfc(&[unit("app", "main", source)], "app::main")
        .expect("slider widget generates through the package route");
    for expected in [
        r#"<input type="range" min="0" max="100" step="1" value="50" />"#,
        r#"<input type="range" min="2" max="20" step="3" value="8" />"#,
    ] {
        assert!(sfc.contains(expected), "missing {expected}: {sfc}");
    }
}

#[test]
fn p125_package_sfc_widget_slots_keep_their_state_and_native_roles() {
    let source = r#"
widget Preview {
    model { var shown bool = false }
    view {
        col {
            accordion (type: "single", collapsible: "true") {
                accordion-item (value: "first") {
                    accordion-trigger "First"
                    accordion-content { text "Answer" {} }
                }
            }
            collapsible (open: .shown) {
                collapsible-trigger { text "Toggle" {} }
                collapsible-content { text "Open" {} }
            }
            scroll (style: "h-20") { text "Item" {} }
            pagination (total: 100, per_page: 10) {
                pagination-list {
                    pagination-previous {}
                    pagination-item (value: 10) { text "10" {} }
                    pagination-next {}
                }
            }
        }
    }
}
"#;
    let sfc = generate_package_sfc(&[unit("app", "main", source)], "app::main")
        .expect("controls generate through the package route");
    for expected in [
        "const __ace_widget_1 = ref(null)",
        ":aria-expanded=\"__ace_widget_1 === 0\"",
        "? null : 0",
        ":aria-expanded=\"shown\"",
        "@click=\"shown = !shown\"",
        "v-show=\"shown\"",
        "tabindex=\"0\" class=\"overflow-y-auto h-20\"",
        "Math.ceil(Number(100) / Math.max(1, Number(10)))",
        ":aria-current=\"__ace_widget_3 === Number(10)",
    ] {
        assert!(sfc.contains(expected), "missing {expected}: {sfc}");
    }
}

/// ①生成物结构：模板按钮/事件、refs、handler、helper 定义与公开状态。
#[test]
fn p980_package_sfc_has_template_script_and_state() {
    let sfc = gen();
    assert!(sfc.contains("<template>"), "template block");
    assert!(
        sfc.contains("<button @click=\"onInc\">+</button>"),
        "button + is bound to the generated handler: {sfc}"
    );
    assert!(
        sfc.contains("<button @click=\"onReset\">Reset</button>"),
        "button Reset is bound"
    );
    assert!(
        sfc.contains("const count = ref(0)"),
        "model initial is a ref"
    );
    assert!(sfc.contains("function onInc()"), "handler function emitted");
    assert!(
        sfc.contains("a2x_fn_") && sfc.contains("inc1"),
        "helper definition is emitted with its mapped name"
    );
    assert!(
        sfc.contains("defineExpose({ __state:"),
        "component publishes its state"
    );
}

/// ②真实 Vue 挂载 + 点击：0 → 1 → 0（再两次 `+` → 2）。
#[test]
fn p980_package_sfc_mounts_and_counts() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_package_sfc_mounts_and_counts: not scheduled \
             (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright + vue required)"
        );
        return;
    }
    let dir = std::env::temp_dir().join(format!("p980-sfc-mount-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("component.sfc"), gen()).unwrap();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("../../examples/ui/ace-web-lane/tests/test_package_sfc_mount.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(script.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("SFC_MOUNT_OK"),
        "vue mount: {stdout} / {stderr}"
    );
}

/// ③未改写的 `002-counter` 原源：记录**首个实际阻断**（不通过即具名，不静默）。
#[test]
fn p980_package_sfc_002_first_blocker_is_named() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
    let src = std::fs::read_to_string(&at).expect("002 source readable");
    let err = generate_package_sfc(&[unit("app", "main", &src)], "app::main")
        .expect_err("raw 002 is still blocked by a language gap; record it by name");
    assert!(
        err.contains("source.lex") || err.contains("source.syntax") || err.contains("source.age."),
        "first blocker must be a named diagnostic: {err}"
    );
    eprintln!("002 first blocker: {err}");
}

/// PLAN-980 view-if 刀：view 级条件项在包 SFC 模板腿渲染为
/// `<template v-if/v-else>`（WEB 真语义；else-if 链=嵌套 template）。
#[test]
fn p980_package_sfc_renders_view_if_as_template_directives() {
    let src = "widget W {
    msg { R }

    model {
        var n int = 0
    }

    view {
        col {
            if .n == 1 {
                text \"one\"
            } else {
                text \"many\"
            }
        }
    }

    on {
        .R -> { .n = .n + 1 }
    }
}
";
    let sfc = generate_package_sfc(&[unit("app", "main", src)], "app::main")
        .expect("view-if source generates a package SFC");
    assert!(
        sfc.contains("<template v-if="),
        "then arm renders as template v-if: {sfc}"
    );
    assert!(
        sfc.contains("one") && sfc.contains("many"),
        "both arms' content renders"
    );
    assert!(
        sfc.contains("<template v-else>"),
        "else arm renders as template v-else"
    );
}

/// PLAN-980 view-if 边界收口（静态核对两缺陷）：①含双引号比较条件经
/// escape_attr（`v-if="mode == &quot;dark&quot;""`）；②空 then + else 形保留
/// 空 v-if 锚（不产孤立 v-else）；③else-if 链=嵌套 template。挂载档以真
/// Vue 编译+分支切换验证三形。
const COND_SRC: &str = r#"widget W {
    msg { Toggle, Cycle }

    model {
        var flag bool = false
        var n int = 0
    }

    view {
        col {
            button "T" { onclick: .Toggle }
            button "C" { onclick: .Cycle }
            if .flag != true {} else {
                text "empty-then-fallback"
            }
            if .n == 1 {
                text "one"
            } else if .n == 2 {
                text "two"
            } else {
                text "many"
            }
        }
    }

    on {
        .Toggle -> { .flag = !.flag }
        .Cycle -> { .n = .n + 1 }
    }
}
"#;

fn cond_sfc() -> String {
    generate_package_sfc(&[unit("app", "main", COND_SRC)], "app::main")
        .expect("conditional source generates a package SFC")
}

/// ①生成档：空锚（非孤立 v-else）+else-if 嵌套（转义判据=crate 内单元
/// 测 `p980_cond_render_tests`——包面 checker 的条件值域是另一具名边界）。
#[test]
fn p980_package_sfc_cond_escapes_quotes_and_keeps_empty_then_anchor() {
    let sfc = cond_sfc();
    let anchor = "<template v-if=\"(flag !== true)\"></template><template v-else>";
    assert!(
        sfc.contains(anchor),
        "empty-then keeps the v-if anchor right before v-else"
    );
    let else_at = sfc.find("<template v-else>").expect("v-else present");
    let after = &sfc[else_at..];
    assert!(
        after.contains("<template v-if="),
        "else-if chain nests a v-if inside the v-else arm"
    );
}

/// ②挂载档（排窗）：真 Vue 编译 + 分支切换——字符串比较切臂/空 then 锚
/// 收放/else-if 三态轮换，零 page error。
#[test]
fn p980_package_sfc_cond_branches_switch_in_real_vue() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_package_sfc_cond_branches_switch_in_real_vue: not scheduled              (set AUTO_UI_ACE_WEB_E2E=1; browser tier)"
        );
        return;
    }
    let dir = std::env::temp_dir().join(format!("p980-pkg-cond-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("component.sfc"), cond_sfc()).unwrap();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("../../examples/ui/ace-web-lane/tests/test_pkg_cond_mount.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(script.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("PKG_COND_MOUNT_OK"),
        "pkg cond mount: {stdout} / {stderr}"
    );
    for line in stdout
        .lines()
        .filter(|l| l.starts_with("[+]") || l.starts_with("[-]"))
    {
        eprintln!("{line}");
    }
}

/// F-R21-2（PLAN-981 r21 复审指派，owner 980/UI）：`handler_stmts` 消费门
/// 负例——包面 widget 形 on 体控制流（if，`age-view-if-stmt` 包面文法可
/// 达）在本车道**具名拒**（`source.age.sfc-handler-stmt`），不双支裸喷/
/// 部分复放（lang 侧 `handler_stmts` 契约：部分复放=错状态值，禁止）。
#[test]
fn p980_package_sfc_if_handler_named_reject() {
    let src = "node T {\n    msg { Go }\n\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"T\" }\n    }\n\n    on {\n        .Go -> { if .n == 1 { .n = 2 } else { .n = 3 } }\n    }\n}\n";
    let err = generate_package_sfc(&[unit("app", "main", src)], "app::main")
        .expect_err("if-body handler must named-reject on this lane");
    assert!(
        err.contains("source.age.sfc-handler-stmt"),
        "named rejection code: {err}"
    );
    assert!(
        err.contains('`') && err.contains("Go") && err.contains("if"),
        "names the handler and the statement kind: {err}"
    );
}

/// F-R21-2 门 Let 臂永久例（r21 附带「UI 侧 Let 永久例」收口）：981 r21
/// fold 落主（lang `33895d91c`——`age-on-widget-stmt` 并集规则两面同开
/// let/if）后，widget 形 on 块 `let` 在主线**可解析并到达消费门**——本车
/// 道具名拒 `…contains a \`let\` statement…`（fold 前同源在文法层被拒=
/// `source.age.structure`，门 Let 臂主线不可达的历史归档由此收口；并集
/// 树验收探针已先行证实命中，本例为其永久化）。
#[test]
fn p980_package_sfc_let_handler_named_reject() {
    let src = "node T {\n    msg { Go }\n\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"T\" }\n    }\n\n    on {\n        .Go -> {\n            let q = .n + 1\n            .n = q\n        }\n    }\n}\n";
    let err = generate_package_sfc(&[unit("app", "main", src)], "app::main")
        .expect_err("let-body handler must named-reject on this lane");
    assert!(
        err.contains("source.age.sfc-handler-stmt") && err.contains("let"),
        "named rejection with the let kind: {err}"
    );
    assert!(
        !err.contains("source.age.structure"),
        "post-fold mainline parses the let body (grammar no longer rejects first): {err}"
    );
}
