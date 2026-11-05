//! PLAN-049 T-03（mixed-ui-2d-3d 永久门）：类型化嵌入 AST/降层/挂载产品。
//! 正例：提交夹具降层（区域表＋实例链＋世界 ABI 针脚）＋再生成逐字节
//! 一致＋已交付产物 byte-pin。负例（具名拒绝）：非 root 区域、区域边长
//! 越界、未知区域 prop、场景元素超界、坏 kind、跨维元素、float 绑定；
//! 旧档守卫：默认 dialect 拒嵌入头、混合 dialect 拒 node 级双出口/共存
//! （039 有限 marker 档身份不扩大）。
//! Run: cargo test -p autographics-node2d-pilot --test node_mixed_views_gates
use autographics_node2d_pilot::{
    load_project, load_project_dual, load_project_mixed, lower_project_node_mixed,
    lower_project_node_ui,
};
use std::fs;
use std::path;

const MAIN: &str = include_str!("../fixtures/node-mixed/main.at");
const COUNTER: &str = include_str!("../fixtures/node-mixed/counter.at");
const COMMITTED_UI_JSON: &str = include_str!("../generated-node-mixed/ui-ab-mixed/ui.json");
const COMMITTED_WORLD: &str = include_str!("../generated-node-mixed/ui-ab-mixed/world.at");

fn write(dir: &path::Path, files: &[(&str, &str)]) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower_mixed(
    dir: &path::Path,
) -> Result<autographics_node2d_pilot::LowerOutputNodeMixed, String> {
    let project = load_project_mixed(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_node_mixed(&project).map_err(|e| e.to_string())
}

fn scratch(tag: &str) -> path::PathBuf {
    path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("plan049-gates-{tag}-{}", std::process::id()))
}

// ------------------------------------------------------------ positives --

#[test]
fn mixed_fixture_lowers_with_regions_instance_chain_and_world_abi() {
    let dir = scratch("pos");
    write(&dir, &[("main", MAIN), ("counter", COUNTER)]);
    let out = lower_mixed(&dir).expect("mixed fixture lowers");
    // 产物身份。
    assert!(out
        .ui_json
        .contains("\"schema\": \"autographics.node-ui-mixed/1\""));
    assert!(out.ui_json.contains("plan049 node-ui-mixed"));
    // 区域元素在 def 元素树内（挂载关系；root 切片）。
    for needle in [
        "\"kind\": \"Embed\"",
        "\"name\": \"reg2\"",
        "\"dim\": \"2d\"",
        "\"name\": \"reg3\"",
        "\"dim\": \"3d\"",
        "\"kind\": \"quad\"",
        "\"kind\": \"cylinder\"",
        "\"fn\": \"main_ui_click_reg2\"",
        "\"fn\": \"main_ui_click_reg3\"",
    ] {
        assert!(out.ui_json.contains(needle), "ui.json missing `{needle}`");
    }
    // 实例链（嵌套纯 UI 实例不受区域切片影响）。
    assert!(out.ui_json.contains("\"path\": \"Main/a\""));
    // 世界 ABI：同一派发器面＋半缓冲拆分（≤128 每半）＋既有 int getter。
    let src = &out.user_source;
    for needle in [
        "fn main_ui_click_reg2()",
        "fn main_ui_click_reg3()",
        "fn main_region2()",
        "fn main_region3()",
        "var main_ui_text_0=[",
        "var main_ui_text_b_0=[",
        "var main_ui_text_len_0=[0,0,0,0]",
        "fn get_r_count() int",
        "fn get_Main_a_count() int",
        "fn app_start()",
        "fn app_update(dt float)",
    ] {
        assert!(src.contains(needle), "world.at missing `{needle}`");
    }
    // 半缓冲安全界：任一字节缓冲数组不得达到 256 元素（SDK 恰 256 截断）。
    for line in src.lines() {
        if line.starts_with("var main_ui_text") && line.contains("=[") {
            let commas = line.matches(',').count() + 1;
            assert!(commas < 256, "byte buffer at SDK truncation edge: {commas}");
        }
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mixed_regeneration_and_committed_products_are_byte_identical() {
    let dir = scratch("regen");
    write(&dir, &[("main", MAIN), ("counter", COUNTER)]);
    let a = lower_mixed(&dir).expect("first");
    let b = lower_mixed(&dir).expect("second");
    assert_eq!(a.user_source, b.user_source, "world.at byte-identical");
    assert_eq!(a.ui_json, b.ui_json, "ui.json byte-identical");
    // 已交付冻结产物 byte-pin（本提交的 generated-node-mixed 产品）。
    assert_eq!(a.user_source, COMMITTED_WORLD, "committed world.at drifted");
    assert_eq!(a.ui_json, COMMITTED_UI_JSON, "committed ui.json drifted");
    let _ = fs::remove_dir_all(&dir);
}

// ------------------------------------------------------ old profiles ----

#[test]
fn old_node_ui_surface_unchanged_and_rejects_embed_heads() {
    // 048 入口（node-ui profile，默认 dialect 载入）：嵌入头不获识别，
    // 载入期即具名语法拒绝（048 基线同一错误面；node-ui 产物身份不含
    // 任何 Embed 元素）。深层 `PLAN-049 mixed-profile only` 具名守卫为
    // 降层不变量（防绕过载入直接构造）。
    let dir = scratch("old048");
    write(&dir, &[("main", MAIN), ("counter", COUNTER)]);
    let err = load_project(&dir.join("main.at"))
        .map_err(|e| e.to_string())
        .expect_err("default-dialect load must reject embed heads");
    assert!(
        err.contains("Colon") || err.contains("Unexpected"),
        "048-baseline parse rejection expected: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn node_level_dual_and_coexistence_guards_stay_in_mixed_dialect() {
    // node 级 view+view-2d 共存（048 守卫字节不变）与 view-2d+view-3d
    // 双出口（039 档）在混合 dialect 下同样具名拒绝。
    let coexist = r#"
node Main {
    model {
        var count = 0
    }
    view {
        text t { text: "x" }
    }
    view-2d {
        shape2d s { kind: quad, x: 0, y: 0, w: 8, h: 8 }
    }
}
fn main() {}
"#;
    let dual = r#"
node Main {
    model {
        var count = 0
    }
    view-2d {
        shape2d s { kind: quad, x: 0, y: 0, w: 8, h: 8 }
    }
    view-3d {
        cylinder3d c { translation: (0, 0, 0) }
    }
}
fn main() {}
"#;
    for (tag, src) in [("coexist", coexist), ("dual", dual)] {
        let dir = scratch(tag);
        write(&dir, &[("main", src)]);
        let err = load_project_mixed(&dir.join("main.at"))
            .map_err(|e| e.to_string())
            .expect_err(tag);
        assert!(
            err.contains("view together with view-2d") || err.contains("declares both"),
            "{tag}: guard message expected, got: {err}"
        );
        // 默认 dialect 同拒（嵌入/双出口都不属于默认档）。
        assert!(
            load_project(&dir.join("main.at")).is_err(),
            "{tag}: default must reject"
        );
        // queries-3d 档不受影响地解析双出口（039 身份保持——仅语法面；
        // 其降层准入由 039 自己的门负责，此处不重复）。
        if tag == "dual" {
            assert!(
                load_project_dual(&dir.join("main.at"), true).is_ok(),
                "039 dual-view parse tier must stay open"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }
}

// ----------------------------------------------------------- negatives --

fn assert_mixed_reject(tag: &str, files: &[(&str, &str)], needle: &str) {
    let dir = scratch(tag);
    write(&dir, files);
    let err = lower_mixed(&dir).expect_err(tag);
    assert!(
        err.contains(needle),
        "{tag}: expected rejection containing `{needle}`, got: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

const NEG_ROOT_ONLY: &str = r#"
use counter: Counter
node Main {
    model {
        var count = 0
    }
    view {
        Counter a { start: 0, step: 1 }
    }
    on {}
}
fn main() {}
"#;
const NEG_COUNTER_WITH_REGION: &str = r#"
node Counter(start: int, step: int) {
    model {
        var count = .start
    }
    view {
        row card {
            view-2d badge {
                width: 40
                height: 40
                shape2d dot { kind: circle, x: 5, y: 5, r: 4 }
            }
        }
    }
    on {}
}
"#;
const NEG_BIG_SIDE: &str = r#"
node Main {
    model {
        var count = 0
    }
    view {
        view-2d reg {
            width: 641
            height: 90
            shape2d s { kind: quad, x: 0, y: 0, w: 8, h: 8 }
        }
    }
    on {}
}
fn main() {}
"#;
const NEG_UNKNOWN_PROP: &str = r#"
node Main {
    model {
        var count = 0
    }
    view {
        view-2d reg {
            width: 40
            height: 40
            color: (1, 2, 3)
            shape2d s { kind: quad, x: 0, y: 0, w: 8, h: 8 }
        }
    }
    on {}
}
fn main() {}
"#;
const NEG_SCENE_BUDGET: &str = r#"
node Main {
    model {
        var count = 0
    }
    view {
        view-2d reg {
            width: 80
            height: 60
            shape2d s0 { kind: quad, x: 0, y: 0, w: 4, h: 4 }
            shape2d s1 { kind: quad, x: 4, y: 0, w: 4, h: 4 }
            shape2d s2 { kind: quad, x: 8, y: 0, w: 4, h: 4 }
            shape2d s3 { kind: quad, x: 12, y: 0, w: 4, h: 4 }
            shape2d s4 { kind: quad, x: 16, y: 0, w: 4, h: 4 }
            shape2d s5 { kind: quad, x: 20, y: 0, w: 4, h: 4 }
            shape2d s6 { kind: quad, x: 24, y: 0, w: 4, h: 4 }
            shape2d s7 { kind: quad, x: 28, y: 0, w: 4, h: 4 }
            shape2d s8 { kind: quad, x: 32, y: 0, w: 4, h: 4 }
        }
    }
    on {}
}
fn main() {}
"#;
const NEG_BAD_KIND: &str = r#"
node Main {
    model {
        var count = 0
    }
    view {
        view-2d reg {
            width: 40
            height: 40
            shape2d s { kind: triangle, x: 0, y: 0, w: 8, h: 8 }
        }
    }
    on {}
}
fn main() {}
"#;
const NEG_CROSS_DIM_ELEMENT: &str = r#"
node Main {
    model {
        var count = 0
    }
    view {
        view-3d reg {
            width: 40
            height: 40
            shape2d s { kind: quad, x: 0, y: 0, w: 8, h: 8 }
        }
    }
    on {}
}
fn main() {}
"#;
const NEG_FLOAT_BINDING: &str = r#"
node Main {
    model {
        var count = 0
    }
    view {
        view-2d reg {
            width: 40
            height: 40
            shape2d s { kind: quad, x: 1.5, y: 0, w: 8, h: 8 }
        }
    }
    on {}
}
fn main() {}
"#;

#[test]
fn mixed_negatives_are_named_rejections() {
    // 非 root def 区域（T-02 冻结切片：实例挂载身份归 T-04+）。
    assert_mixed_reject(
        "neg-root-only",
        &[
            ("main", NEG_ROOT_ONLY),
            ("counter", NEG_COUNTER_WITH_REGION),
        ],
        "non-root def requires instance-mount identity",
    );
    // 区域边长越界（1..=640 冻结界）。
    assert_mixed_reject("neg-big-side", &[("main", NEG_BIG_SIDE)], "1..=640");
    // 未知区域 prop（冻结 width/height/onclick）。
    assert_mixed_reject(
        "neg-unknown-prop",
        &[("main", NEG_UNKNOWN_PROP)],
        "unknown prop 'color' on region 'reg'",
    );
    // 场景元素超界（每区域 ≤8 冻结）。
    assert_mixed_reject(
        "neg-scene-budget",
        &[("main", NEG_SCENE_BUDGET)],
        "8-element budget",
    );
    // 坏 kind：未知 kind 标识在共享表达式解析层具名拒绝（UndefinedVariable；
    // 已知标识的 kind 白名单由降层 `quad|circle` 具名裁决——见 mixed 门正例
    // 集外的 kind拼写 漂移由该白名 unit 承接）。
    assert_mixed_reject(
        "neg-bad-kind",
        &[("main", NEG_BAD_KIND)],
        "UndefinedVariable",
    );
    // 跨维元素：view-3d 区域体复用现有 view-3d 元素解析，shape2d 在解析
    // 层即具名拒绝（维度分离在文法面成立；降层 3D 冻结集守卫承接其余
    // 元素的 T-02 集外拒绝）。
    assert_mixed_reject(
        "neg-cross-dim",
        &[("main", NEG_CROSS_DIM_ELEMENT)],
        "unknown view-3d element 'shape2d'",
    );
    // float 场景绑定（int getter ABI 面）。
    assert_mixed_reject(
        "neg-float-binding",
        &[("main", NEG_FLOAT_BINDING)],
        "int getter ABI",
    );
}
