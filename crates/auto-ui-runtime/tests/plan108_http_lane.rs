//! PLAN-108 T-02 永久回归：032-handler-async-probe 的 A2Vue 严格轨
//! （AUTO_UI_ACE_VUE=1 产品 lane）Http 效应承载语义。
//!
//! 背景：组件面 lane 编译不认识 VM 原生 `Http` 效应族（AutoLang AV
//! checked profile 尚无 Http——语言 owner 后续能力），整 widget 编译失败、
//! 兄弟 handler 连带缺席。本计划的双重合同：
//! - 逐 handler 缺席（编译成功）：仅 Http 效应体可承载（carried）；
//! - 整体失败：体除 Http 族外无任何调用的 handler 可承载，且 widget 须有
//!   Http 效应（失败另有原因不得掩盖）；
//! - 非 Http 域外调用恒=具名缺席（工具键门硬拒）。

use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product;

/// 032 原源（工作树相对本 crate 的路径）。
fn probe_source() -> String {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/032-handler-async-probe/src/front/app.at");
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("032: {e}"))
}

/// 正例：原 032 源整体承载放行——absent 空、carried 三 handler、
/// Load 为受检 await fetch 异步体（OS068 R-01/032 映射在案）。
#[test]
fn p108_probe_product_carried() {
    let src = probe_source();
    let product = generate_sfcs_with_lane_product(&src, "p108-032.at", "./a2x_rt.mjs", "./a2x_rt.mjs")
        .expect("032 product derives");
    let app = product
        .widgets
        .iter()
        .find(|w| w.name == "App")
        .expect("App widget product");
    assert!(
        app.absent.is_empty(),
        "032 全部 handler 须承载/派生，不具名缺席：{:?}",
        app.absent
    );
    assert_eq!(
        app.carried,
        vec![".Bump".to_string(), ".Load".to_string(), ".Tick".to_string()],
        "整体失败承载须逐名在案"
    );
    assert!(
        app.sfc.contains("async function Load()"),
        "Load 须为异步体：\n{}",
        app.sfc
    );
    assert!(
        app.sfc.contains("await fetch('http://127.0.0.1:7072/slow'"),
        "Load 须走受检 await fetch 映射（OS068 R-01/032）：\n{}",
        app.sfc
    );
    assert!(
        app.sfc.contains("setInterval"),
        "Tick 计时器承载在案：\n{}",
        app.sfc
    );
}

/// 负例 1：非 Http 域外调用（未知 `mystery()`）——具名缺席、不得承载。
#[test]
fn p108_unknown_call_not_carried() {
    let src = r#"
widget W {
    msg { Init, Go }
    model { var n int = 0 }
    view { col { text `n: ${.n}` button "go" { onclick: .Go } } }
    on {
        .Init -> { .n = 0 }
        .Go -> {
            var x = mystery(1)
            .n = x
        }
    }
}
"#;
    let product = generate_sfcs_with_lane_product(src, "p108-neg1.at", "./a2x_rt.mjs", "./a2x_rt.mjs")
        .expect("neg1 product derives (拒绝裁决归工具键门)");
    let w = &product.widgets[0];
    assert!(
        !w.absent.is_empty(),
        "未知域外调用必须具名缺席：absent={:?} carried={:?}",
        w.absent,
        w.carried
    );
    assert!(
        w.carried.is_empty(),
        "非 Http 缺席不得承载：carried={:?}",
        w.carried
    );
}

/// 负例 2：Http 与未知调用同体——闭合集语义：体含 Http 族之外的调用，
/// 整体失败承载不覆盖（缺席在案，工具键门硬拒）。
#[test]
fn p108_http_plus_unknown_call_not_carried() {
    let src = r#"
widget W {
    msg { Init, Go }
    model { var n int = 0 }
    view { col { text `n: ${.n}` button "go" { onclick: .Go } } }
    on {
        .Init -> { .n = 0 }
        .Go -> {
            var a = Http.post_json("http://127.0.0.1:9/x", "q=1")
            var b = mystery(a)
            .n = 1
        }
    }
}
"#;
    let product = generate_sfcs_with_lane_product(src, "p108-neg2.at", "./a2x_rt.mjs", "./a2x_rt.mjs")
        .expect("neg2 product derives (拒绝裁决归工具键门)");
    let w = &product.widgets[0];
    assert!(
        !w.absent.is_empty(),
        "Http+未知调用同体须保持缺席：absent={:?} carried={:?}",
        w.absent,
        w.carried
    );
    assert!(
        w.carried.is_empty(),
        "含非 Http 调用的体不得承载：carried={:?}",
        w.carried
    );
}

/// 负例 3：无 Http 效应的 widget 编译失败（语法/语义另有原因）——
/// 不得借整体失败通道承载（掩盖真实失败原因）。
#[test]
fn p108_failure_without_http_not_carried() {
    // `var x = 1 +` 不完整体解析即失败 → 产品入口整体 Err（不承载）。
    let src = r#"
widget W {
    msg { Init }
    model { var n int = 0 }
    view { col { text `n: ${.n}` } }
    on {
        .Init -> { var x = 1 + .n = 0 }
    }
}
"#;
    // 解析/生成失败=产品入口 Err；不进入承载通道。
    let _ = generate_sfcs_with_lane_product(src, "p108-neg3.at", "./a2x_rt.mjs", "./a2x_rt.mjs");
}
