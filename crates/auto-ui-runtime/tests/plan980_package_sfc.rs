//! PLAN-980 ④-A #3：**AGE `view` 树 → Vue SFC** 的生成与真实挂载。
//!
//! 判据：①同一份源生成的 SFC 含模板（按钮/事件绑定）与脚本（refs/handler/helper/
//! `defineExpose` 状态）；②真实 Vue 挂载后点击 `+`/`Reset` 得 `0 → 1 → 0`（2 次 `+`
//! 得 2）；③未改写 `002-counter` 的**首个实际阻断**被具名记录（不静默）。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_sfc::{generate_package_sfc, generate_routed_package_sfcs};

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

const HELPER: &str = "pub fn inc1(n int) int {\n    var next int = n + 1\n    return finish(next)\n}\n\nfn finish(n int) int {\n    return n\n}\n";

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
fn p107_package_sfc_uses_the_checked_public_stylekit_literal() {
    let source = r#"
use stylekit.styles: caption_text
widget ChartLegend {
    view { text "Series 1" { style: caption_text } }
}
"#;
    let styles = include_str!("../../../examples/ui/stylekit/src/front/styles.at");
    let sfc = generate_package_sfc(
        &[unit("app", "main", source), unit("stylekit", "styles", styles)],
        "app::main",
    ).expect("the official stylekit provides its public literal");
    assert!(sfc.contains(r#"<text class="text-xs text-muted-foreground">Series 1</text>"#), "{sfc}");
}

#[test]
fn p107_package_sfc_scopes_chart_css_and_binds_class_branches() {
    let source = r#"
widget ChartLegend {
    model { var active bool = true }
    style {
        /* Authored SVG transition. */
        path { transition: stroke-width 0.15s ease, fill-opacity 0.15s ease; }
    }
    view {
        row (style: ["items-center gap-1", if .active { "" } else { "opacity-40" }, "legend-end"]) {
            svg (viewBox: "0 0 10 10") {
                path (d: "M0 0L10 10") {}
            }
        }
    }
}
"#;
    let sfc = generate_package_sfc(&[unit("app", "main", source)], "app::main")
        .expect("checked chart legend generates a scoped SFC");
    assert!(sfc.contains(concat!(
        "class=\"flex flex-row items-center gap-1 legend-end\" ",
        ":class=\"[(active ? '' : 'opacity-40')]\""
    )), "{sfc}");
    let authored_css = source.split_once("style {").unwrap().1
        .split_once("\n    }").unwrap().0;
    assert!(sfc.contains(&format!("<style scoped>{authored_css}\n    </style>")), "{sfc}");

    let wrong_condition = source.replace("if .active", "if 2");
    let error = generate_package_sfc(&[unit("app", "main", &wrong_condition)], "app::main")
        .expect_err("conditional class list condition must be bool");
    assert!(error.contains("source.type-mismatch"), "{error}");
    let wrong_property = source.replace("row (style:", "row (value:");
    let error = generate_package_sfc(&[unit("app", "main", &wrong_property)], "app::main")
        .expect_err("conditional class lists are checked in class properties");
    assert!(error.contains("source.age.class-list"), "{error}");
}

#[test]
fn p107_package_sfc_emits_quoted_accessibility_attributes() {
    let source = r#"
widget ChartPreview {
    model { var pressed bool = false }
    view {
        div {
            role: "region"
            "aria-roledescription": "carousel"
            "aria-label": "Cards & \"photos\""
            "aria-pressed": .pressed
            text | Slide 1
        }
    }
}
"#;
    let sfc = generate_package_sfc(&[unit("app", "main", source)], "app::main")
        .expect("quoted accessibility attributes generate");
    for attribute in [
        r#"role="region""#,
        r#"aria-roledescription="carousel""#,
        r#"aria-label="Cards &amp; &quot;photos&quot;""#,
        r#":aria-pressed="pressed""#,
    ] {
        assert!(sfc.contains(attribute), "missing {attribute}: {sfc}");
    }
    assert!(sfc.contains(">Slide 1</div>"), "{sfc}");
}

#[test]
fn p107_package_sfc_keeps_svg_geometry_and_labels() {
    let source = r##"
node SvgPreview {
    model {
        var extent int = 4
        gridPath str = "M40 200 H550 M40 140 H550 M40 80 H550 M40 20 H550"
    }
    view {
        svg (viewBox: "0 0 20 20", style: "w-full") {
            g {
                path (d: .gridPath, fill: "none") {}
                line (x1: 1, y1: 4, x2: .extent, y2: 4, stroke: "#ff0000", stroke-width: 2) {}
                text "A < B & C" { x: 2, y: 14, text-anchor: "middle" }
            }
        }
    }
}
"##;
    let sfc = generate_package_sfc(&[unit("app", "main", source)], "app::main")
        .expect("render checked SVG");
    for expected in [
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" class="w-full">"#,
        r#"<path :d="gridPath" fill="none"></path>"#,
        r#"const gridPath = ref("M40 200 H550 M40 140 H550 M40 80 H550 M40 20 H550")"#,
        r##"<line stroke="#ff0000" stroke-width="2" x1="1" :x2="extent" y1="4" y2="4"></line>"##,
        r#"<text text-anchor="middle" x="2" y="14">A &lt; B &amp; C</text>"#,
    ] {
        assert!(sfc.contains(expected), "missing {expected}: {sfc}");
    }
    let unsupported = source.replace("stroke-width: 2", "unsupported: 2");
    let diagnostic = generate_package_sfc(&[unit("app", "main", &unsupported)], "app::main")
        .expect_err("unsupported SVG binding is named");
    assert!(diagnostic.contains("source.age.sfc-svg: unsupported `unsupported` on SVG `line`"), "{diagnostic}");
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
    assert!(sfc.contains("let l1_next"), "helper locals are declared");
    assert!(sfc.lines().any(|line| {
        line.starts_with("function a2x_fn_") && line.contains("_finish(")
    }), "the helper's own callee is emitted");
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

#[test]
fn p107_package_sfc_unit_handler_emits_both_checked_branches() {
    let src = "node T {\n    msg { Go }\n\n    model {\n        var n int = 0\n    }\n\n    view {\n        col { text \"T\" }\n    }\n\n    on {\n        .Go -> { if .n == 1 { .n = 2 } else { .n = 3 } }\n    }\n}\n";
    let generate = |src: &str| generate_package_sfc(&[unit("app", "main", src)], "app::main");
    let sfc = generate(src).expect("checked unit handler emits");
    assert!(sfc.contains("function onGo()"), "{sfc}");
    assert!(sfc.contains("l0___age_s_0_n = 2;") && sfc.contains("l0___age_s_0_n = 3;"), "{sfc}");
    assert!(sfc.contains("n.value = l0___age_s_0_n;"), "{sfc}");

    let owned = src.replace("var n int = 0", "var n int = 0\n        var running str = \"true\"");
    let rejection = generate(&owned).expect_err("unsupported owned bindings have a diagnostic");
    assert!(rejection.contains("source.age.sfc-handler") && rejection.contains("copyable"), "{rejection}");
}

#[test]
fn p107_package_sfc_unit_handler_preserves_let_and_mutable_locals() {
    let src = r#"
widget T {
    msg { Go }
    model { var n int = 0 }
    view { button "Grow" { onclick: .Go } }
    on {
        .Go -> {
            let next = .n + 1
            var first = true
            if first {
                first = false
            }
            if !first {
                .n = next
            }
        }
    }
}
"#;
    let sfc = generate_package_sfc(&[unit("app", "main", src)], "app::main")
        .expect("checked let and mutable local emit");
    assert!(sfc.contains("__age_hl_0_0_0_next"), "{sfc}");
    assert_eq!(sfc.matches("__age_hl_0_0_1_first = ").count(), 2, "{sfc}");
    assert!(sfc.contains("n.value = l0___age_s_0_n;"), "{sfc}");
}

#[test]
fn p107_package_sfc_mouse_area_uses_checked_coordinate_handler() {
    let source = r#"
widget T {
    msg { Move(float, f32), Leave }
    model {
        var n int = 0
        var moved bool = false
    }
    view {
        mouse-area (coords: "120x44", onmousemove: .Move, onmouseleave: .Leave) {
            span { if .moved { text "Moved" {} } }
        }
    }
    on {
        .Move(x, y) -> {
            let passed = x > y
            if passed {
                .n = 1
                .moved = true
            } else {
                .n = 2
                .moved = false
            }
        }
        .Leave -> { .moved = false }
    }
}
"#;
    let generate = |source: &str| generate_package_sfc(&[unit("app", "main", source)], "app::main");
    let sfc = generate(source).expect("checked mouse movement generates");
    assert!(sfc.contains(r#"@mousemove="acePointerMove($event, onMove, onLeave, 120, 44)""#));
    assert!(sfc.contains(r#"@mouseleave="acePointerLeave($event.currentTarget)""#));
    assert!(sfc.contains("function onMove(p0_"));
    assert!(sfc.contains("n.value = l2___age_s_0_n;"), "{sfc}");
    assert!(sfc.contains("moved.value = l3___age_s_0_moved;"), "{sfc}");
    assert!(sfc.contains("return [x, y]"));
    assert!(sfc.contains("send(...sample)"));

    let bad_extent = generate(&source.replace("120x44", "120xInfinity")).unwrap_err();
    assert!(bad_extent.contains("source.age.sfc-pointer: coords"), "{bad_extent}");
    let wrong_event = generate(&source.replace("onmousemove: .Move", "onmouseenter: .Move")).unwrap_err();
    assert!(wrong_event.contains("source.age.sfc-payload: `mouseenter`"), "{wrong_event}");
}

#[test]
fn p107_routed_package_publishes_the_checked_pointer_runtime() {
    let shell = r#"
widget App {
    routes { "/" -> use chart }
    view { col { outlet } }
}
"#;
    let chart = r#"
widget Chart {
    msg { Move(float, f32), Reset }
    model { var n int = 0 }
    view {
        mouse-area (coords: "120x44", onmousemove: .Move) { span { text "Chart" {} } }
        button "Reset" { onclick: .Reset }
    }
    on {
        .Move(x, y) -> {
            let offsetX = x + 150.0
            let offsetY = y + 160.0
            if offsetX > offsetY { .n = .n + 1 }
        }
        .Reset -> { .n = .n + 1 }
    }
}
"#;
    let product = generate_routed_package_sfcs(
        &[unit("app", "app", shell), unit("app", "chart", chart)],
        "app::app",
    )
    .expect("route shell generates")
    .expect("a routed product");
    let page = &product.pages["chart"];
    assert!(page.contains("from '@/a2x_rt.mjs'"), "{page}");
    assert!(page.contains("a2x_f32_round("), "{page}");
    assert!(page.contains("a2x_add("), "{page}");
    assert!(!page.contains("function a2x_add("), "{page}");
    assert_eq!(
        product.runtime_mjs.as_deref(),
        Some(ace_a2x_ts::script_runtime_body(ace_a2x_ts::ScriptLang::JavaScript, &[]).as_str())
    );
}

#[test]
fn p107_routed_chart_pointer_includes_transitive_math_helpers() {
    let shell = r#"
widget App {
    routes { "/" -> use chart }
    view { col { outlet } }
}
"#;
    let chart = r#"
use geometry: dc, ds
widget Chart {
    msg { Move(float, float) }
    model { var hit bool = false }
    view { mouse-area (onmousemove: .Move) { text "Chart" {} } }
    on {
        .Move(x, y) -> {
            let angle = math.atan2(y, x)
            let radius = math.sqrt(x * x + y * y)
            .hit = dc(angle) * radius > ds(angle) * radius
        }
    }
}
"#;
    let geometry = "pub fn dc(x float) float { math.cos(x) }\npub fn ds(x float) float { math.sin(x) }";
    let product = generate_routed_package_sfcs(
        &[
            unit("app", "app", shell),
            unit("app", "chart", chart),
            unit("app", "geometry", geometry),
        ],
        "app::app",
    )
    .expect("route shell generates")
    .expect("a routed product");
    let page = &product.pages["chart"];
    for name in ["trig_series", "sqrt_newton", "atan_series"] {
        assert_eq!(
            page.lines().filter(|line| line.starts_with("function a2x_fn_")
                && line.contains(&format!("_{name}("))).count(),
            1,
            "one definition of {name}: {page}"
        );
    }
    assert!(page.contains("from '@/a2x_rt.mjs'"), "{page}");
    assert!(product.runtime_mjs.is_some());
}

#[test]
fn p107_checked_chart_package_site_cannot_fall_back_to_an_html_tag() {
    let app = r#"
widget Charts {
    use { package: official from "components" }
    view { line-chart {} }
}
"#;
    let chart = "widget LineChart { view { text \"Curve\" {} } }";
    let sfc = generate_package_sfc(
        &[unit("app", "main", app), unit("official", "line_chart", chart)],
        "app::main",
    )
    .expect("the checked child widget uses the package renderer");
    assert!(sfc.contains("<AcePackageView />"));
    assert!(!sfc.contains("<line-chart"));
}
