//! PLAN-1072（AutoUI Vue 与生产 TS/JS 接管）work 面测试文件。
//!
//! 刀A（组件面 style + type 键）：005-login 真源整例经 `generate_sfc_with_lane`
//! 全量 lane 派生（文件级 style 声明+view `style:` ident 引用+`type:` 键+
//! 双 if-else Submit 体+lambda-if）。红相（刀前）：`source.syntax: expected
//! 'fn', found 'style'`——整源编译失败=6 handler 全体缺席；style 声明开
//! age-component（981 (c)3 替换路径）+ age-event-key type-kw 键臂后翻转。
//! 刀B（010）：contact-form 真源（faqs 列表字段+view for）——三墙勘定中。

/// 刀A 交付态：005 真源整例——6 handler 全派生（Submit/UseAuto/UseSolarized/
/// ToggleMode + λ×2）、零缺席；SFC 侧 style 配方展开为字面量 class（与旧
/// 管线同源——`generate_sfc_with_lane` 现显式注册本文件配方）。
#[test]
fn p1072_005_lane_whole_source_derives() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/005-login/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("005: {e}"));
    let (sfc, bodies, absent) =
        generate_sfc_with_lane(&src, "p1072-005.at").expect("005 whole-source derives");
    assert!(absent.is_empty(), "005 zero absent handlers: {absent:?}");
    assert_eq!(
        bodies.len(),
        6,
        "6 handlers lane-derived (4 on + 2 lambdas)"
    );
    // Submit 体：双 if-else 结构化（str 相等条件+分支赋值）。
    let submit = bodies
        .iter()
        .find(|b| b.pattern.trim_start_matches('.') == "Submit")
        .unwrap();
    let joined = submit.lines.join("\n");
    // 刀C marshal 形：宿主裸串读 → a2x_str_lit 包裹（边界统一裁定）。
    assert!(
        joined.contains("a2x_str_eq(a2x_str_lit(email.value")
            && joined.contains("a2x_str_eq(a2x_str_lit(password.value"),
        "Submit body carries both if-else arms (marshalled reads): {joined}"
    );
    assert!(
        !joined.contains("a2x_ret_owned_skip"),
        "the owned-swap old-value free block is dropped at the boundary (host GC owns strings)"
    );
    // λ-if 体：`!=` 条件+分支赋值（lambda-if 刀面在真源整例上的承载）。
    for pat in ["__evt_oninput_1", "__evt_oninput_2"] {
        let b = bodies
            .iter()
            .find(|b| b.pattern.trim_start_matches('.') == pat)
            .unwrap();
        assert!(
            b.lines.iter().any(|l| l.contains("a2x_str_ne")),
            "{pat} carries the str != condition"
        );
    }
    // SFC style 展开：配方注册生效（field_label→字面量 class；修复前落
    // `:style="field_label"`=运行时 undefined）。
    assert!(
        sfc.contains("text-sm font-medium text-foreground mt-6"),
        "field_label recipe expands to the literal class in the SFC template"
    );
    assert!(
        !sfc.contains(":style=\"field_label\""),
        "no unexpanded recipe idents leak into the SFC"
    );
}

/// 刀A 判别力（SFC 侧）：style 引用不展开时的坏产物形态——未注册配方的
/// 独立入口此前产 `:style="field_label"`（回归防线：显式注册不可再退化为
/// 裸 ident 绑定）。
#[test]
fn p1072_005_style_expansion_is_literal_not_binding() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/005-login/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("005: {e}"));
    let (sfc, _bodies, _absent) =
        generate_sfc_with_lane(&src, "p1072-005.at").expect("005 derives");
    for recipe in ["input_login", "field_label", "error_text"] {
        assert!(
            !sfc.contains(&format!(":style=\"{recipe}\"")),
            "recipe {recipe} must not survive as a bare ident binding"
        );
    }
}

/// 刀B 交付态：010 真源整例——3 handler 全派生（Submit 赋值/SetTheme 双
/// bare-if 结构化/SetAccent 实读参）、零缺席；view `for i, f in .faqs` 与
/// 无类型列表字段经 lang 侧 view-data 承载（1071 边界）；SFC 侧 v-for
/// 发射+field_col 配方展开；`__bind_*` 方言合成件不进 lane 规格集（值绑
/// 定由模板 :value 承载——红相实证：进 ops=整 lane 编译失败连带全体缺席）。
#[test]
fn p1072_010_lane_whole_source_derives() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/010-contact-form/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("010: {e}"));
    let (sfc, bodies, absent) =
        generate_sfc_with_lane(&src, "p1072-010.at").expect("010 whole-source derives");
    assert!(absent.is_empty(), "010 zero absent handlers: {absent:?}");
    assert_eq!(bodies.len(), 3, "3 on-handlers lane-derived");
    // SetTheme：双 bare-if 结构化（str 相等条件+布尔赋值）。
    let st = bodies
        .iter()
        .find(|b| b.pattern.trim_start_matches('.') == "SetTheme")
        .unwrap();
    let joined = st.lines.join(
        "
",
    );
    assert!(
        joined.contains("a2x_str_eq") && joined.contains("light") && joined.contains("dark"),
        "SetTheme carries both if arms: {joined}"
    );
    // SetAccent：实读参（param handler 检查函数道）。
    let sa = bodies
        .iter()
        .find(|b| b.pattern.trim_start_matches('.') == "SetAccent")
        .unwrap();
    assert!(
        sa.lines.iter().any(|l| l.contains("accent_color")),
        "SetAccent writes the accent field"
    );
    // 模板面：v-for（faqs 只读数据源——旧管线方言 AST 承载）+ field_col 展开。
    assert!(
        sfc.contains("v-for=") && sfc.contains("faqs"),
        "the FAQ loop renders via v-for over faqs"
    );
    assert!(
        sfc.contains("gap-1.5 w-full") && !sfc.contains(":style=\"field_col\""),
        "field_col recipe expands to the literal class"
    );
}

/// T-03（V-B-WEB 真浏览器档·005）：lane 产品产物（`generate_sfcs_with_lane_
/// product` 原样落盘，零注入）→ @vue/compiler-sfc+esbuild+HTTP+chromium——
/// 必填校验（Submit 双 if-else 受检体）/λ-if 错误清理（`!=` 体）/主题按钮/
/// 零 page error。排窗档（AUTO_UI_ACE_WEB_E2E=1）。
#[test]
fn p1072_005_product_browser_mount() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p1072_005_product_browser_mount: not scheduled (set AUTO_UI_ACE_WEB_E2E=1; \
             browser tier, playwright required)"
        );
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/005-login/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("005: {e}"));
    let product =
        generate_sfcs_with_lane_product(&src, "p1072-005.at", "./a2x_rt.mjs", "./a2x_rt.mjs")
            .expect("005 product derives");
    assert!(product.widgets.iter().all(|w| w.absent.is_empty()));
    let dir = std::env::temp_dir().join(format!("p1072-005-mount-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("component.sfc"), &product.widgets[0].sfc).unwrap();
    std::fs::write(dir.join("a2x_rt.mjs"), &product.runtime_mjs).unwrap();
    let rig = manifest.join("../../examples/ui/ace-web-lane/tests/test_005_login.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(rig.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("P1072_005_LOGIN_OK"),
        "005 lane-product browser mount: {stdout} / {stderr}"
    );
}

/// T-03（V-B-WEB 真浏览器档·010）：**同一行为 oracle**（test_010_form.mjs：
/// 三字段输入绑定+提交回执+零 page error——980 ④-A #9 的 legacy 基线判据）
/// 在 **lane 受检产物**上复放——"对照页面事件与源 expected" 的实证件。
#[test]
fn p1072_010_product_browser_mount() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p1072_010_product_browser_mount: not scheduled (set AUTO_UI_ACE_WEB_E2E=1; \
             browser tier, playwright required)"
        );
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/010-contact-form/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("010: {e}"));
    let product =
        generate_sfcs_with_lane_product(&src, "p1072-010.at", "./a2x_rt.mjs", "./a2x_rt.mjs")
            .expect("010 product derives");
    assert!(product.widgets.iter().all(|w| w.absent.is_empty()));
    let dir = std::env::temp_dir().join(format!("p1072-010-mount-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("App.sfc"), &product.widgets[0].sfc).unwrap();
    std::fs::write(dir.join("a2x_rt.mjs"), &product.runtime_mjs).unwrap();
    let rig = manifest.join("../../examples/ui/ace-web-lane/tests/test_010_form.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(rig.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("P980_010_FORM_OK"),
        "010 lane-product browser mount (same oracle as legacy): {stdout} / {stderr}"
    );
}

/// T-03 回归防线：`marshal_str_boundary` 的 `wrap_host_reads` 跳过引号字面
/// 量——形参名 `light` 与字符串字面量 `"light"` 碰撞时，只包裹标识符形
/// 参 `a2x_str_lit(light)`，绝不篡改 `"light"` 或 `"say \"light\""`。
#[test]
fn p1072_marshal_str_boundary_preserves_string_literals() {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;
    let src = r#"widget ThemeProbe {
    msg { SetTheme(str) }
    model {
        var theme str = ""
    }
    view {
        col { text .theme }
    }
    on {
        .SetTheme(light) -> {
            if light == "light" {
                .theme = "say \"light\""
            }
        }
    }
}
"#;
    let (_sfc, bodies, absent) =
        generate_sfc_with_lane(src, "p1072-marshal-lit.at").expect("theme probe lane compiles");
    assert!(absent.is_empty(), "zero absent handlers: {absent:?}");
    let body = bodies
        .iter()
        .find(|b| b.pattern.trim_start_matches('.') == "SetTheme")
        .expect("SetTheme body");
    let joined = body.lines.join("\n");
    assert!(
        joined.contains("a2x_str_eq(a2x_str_lit(light), a2x_str_lit(\"light\"))"),
        "ident param wrapped while string literal \"light\" untouched: {joined}"
    );
    assert!(
        !joined.contains("\"a2x_str_lit(light)\""),
        "must never wrap inside string literal quotes: {joined}"
    );
    assert!(
        joined.contains("say \\\"light\\\""),
        "escaped inner quotes preserved intact: {joined}"
    );
}

const WATCH_EXPOSE_WIDGET_A: &str = r#"style badge = "px-2 py-1 text-xs"

widget WatchExposeApp {
    msg { Inc }
    model {
        var count int = 0
        var watch_hits int = 0
    }
    expose { .Inc, .count, .watch_hits }
    watch {
        .count -> { .watch_hits += 1 }
    }
    view {
        col {
            text `C:${.count} W:${.watch_hits}` { style: badge }
            button "+" { onclick: .Inc }
        }
    }
    on {
        .Inc -> { .count += 1 }
    }
}
"#;

const SETUP_EXT_WIDGET_B: &str = r#"widget SetupBoundaryApp {
    use { fn: record_setup from "src/front/utils/boot.ts" }
    setup {
        let boot = record_setup()
    }
    msg { Bump }
    model {
        var n int = 0
        var seen int = 0
    }
    expose { .Bump, .n, .seen }
    watch {
        .n -> { .seen += 1 }
    }
    view {
        col {
            text .boot
            text `N:${.n} S:${.seen}`
            button "Bump" { onclick: .Bump }
        }
    }
    on {
        .Bump -> { .n += 1 }
    }
}
"#;

/// T-03（CPU 生成面）：①Widget A（style 配方 + watch + `.name` expose + on）
/// 走 ACE lane 全派生（`absent=[]`），SFC 含 `a2x_add`、`./a2x_rt.mjs`、
/// `watch(count,`、`defineExpose({ Inc, count, watch_hits })` 与字面量 style；
/// ②Widget B（`setup` + `use { fn }` + `watch`）诚实落双轨边界——emits
/// `import { record_setup }` from `@/ext/...`、`const boot = record_setup()`、
/// `watch(n,` 与 `absent=[".Bump"]`（component face 拒 setup/use{}）。
#[test]
fn p1072_watch_expose_style_lane_and_setup_boundary() {
    use auto_ui_runtime::ui::ace_vue_script::{
        generate_sfc_with_lane_product, generate_sfcs_with_lane_product,
    };
    let prod_a = generate_sfcs_with_lane_product(
        WATCH_EXPOSE_WIDGET_A,
        "p1072-watch-expose.at",
        "./a2x_rt.mjs",
        "./a2x_rt.mjs",
    )
    .expect("Widget A derives");
    let w_a = &prod_a.widgets[0];
    assert!(
        w_a.absent.is_empty(),
        "Widget A zero absent: {:?}",
        w_a.absent
    );
    assert_eq!(w_a.bodies.len(), 1, "Inc derived through ACE lane");
    assert!(
        w_a.sfc.contains("a2x_add(count.value, 1)") && !w_a.sfc.contains("count.value += 1"),
        "Inc uses ACE lane a2x_add (no legacy fallback): {}",
        w_a.sfc
    );
    assert!(
        w_a.sfc.contains("from \"./a2x_rt.mjs\""),
        "rt import present"
    );
    assert!(w_a.sfc.contains("watch(count,"), "watch(count) emitted");
    assert!(
        w_a.sfc.contains("defineExpose({ Inc, count, watch_hits })"),
        "defineExpose emitted"
    );
    assert!(
        w_a.sfc.contains("px-2 py-1 text-xs") && !w_a.sfc.contains(":style=\"badge\""),
        "style recipe expands to literal class"
    );

    let prod_b = generate_sfc_with_lane_product(SETUP_EXT_WIDGET_B, "p1072-setup-boundary.at")
        .expect("Widget B dual-track product generates");
    assert_eq!(
        prod_b.absent,
        vec![".Bump".to_string()],
        "setup/use{{}} keeps component-face fail-closed boundary (dual-track)"
    );
    assert!(
        prod_b
            .sfc
            .contains("import { record_setup } from '@/ext/src/front/utils/boot'"),
        "ext_imports emitted: {}",
        prod_b.sfc
    );
    assert!(
        prod_b.sfc.contains("const boot = record_setup()"),
        "setup statement emitted at top level: {}",
        prod_b.sfc
    );
    assert!(prod_b.sfc.contains("watch(n,"), "watch(n) emitted");
}

/// T-03（真浏览器排窗 `AUTO_UI_ACE_WEB_E2E=1`）：Widget A（lane watch +
/// expose + unmount 停 watch）与 Widget B（setup 仅一次 + ext_imports +
/// watch + unmount 停 watch）在无头 Chromium 上全链验证。
#[test]
fn p1072_watch_expose_setup_unmount_browser_mount() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p1072_watch_expose_setup_unmount_browser_mount: not scheduled \
             (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright required)"
        );
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::{
        generate_sfc_with_lane_product, generate_sfcs_with_lane_product,
    };
    let prod_a = generate_sfcs_with_lane_product(
        WATCH_EXPOSE_WIDGET_A,
        "p1072-watch-expose.at",
        "./a2x_rt.mjs",
        "./a2x_rt.mjs",
    )
    .expect("Widget A derives");
    assert!(prod_a.widgets[0].absent.is_empty());
    let prod_b = generate_sfc_with_lane_product(SETUP_EXT_WIDGET_B, "p1072-setup-boundary.at")
        .expect("Widget B derives");

    let dir = std::env::temp_dir().join(format!("p1072-lifecycle-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("compA.sfc"), &prod_a.widgets[0].sfc).unwrap();
    std::fs::write(dir.join("a2x_rt.mjs"), &prod_a.runtime_mjs).unwrap();
    std::fs::write(dir.join("compB.sfc"), &prod_b.sfc).unwrap();

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let rig =
        manifest.join("../../examples/ui/ace-web-lane/tests/test_p1072_lifecycle_watch_expose.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(rig.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("P1072_LIFECYCLE_WATCH_EXPOSE_OK"),
        "lifecycle/watch/expose/setup browser mount: {stdout} / {stderr}"
    );
}
