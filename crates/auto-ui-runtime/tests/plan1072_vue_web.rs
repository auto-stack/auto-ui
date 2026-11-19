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
    assert!(
        joined.contains("a2x_str_eq(email.value") && joined.contains("a2x_str_eq(password.value"),
        "Submit body carries both if-else arms: {joined}"
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
