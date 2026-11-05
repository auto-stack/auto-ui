//! PLAN-054 T-03（node-ui-mixed-regions 永久门）：组件/实例活区域降层与
//! 产品身份。正例：夹具降层（非 root def 区域＋实例链＋世界 ABI 针脚）＋
//! 再生成逐字节一致＋已提交产物 byte-pin。负例（具名拒绝）：v1 档对组件
//! 区域的冻结诊断逐字、区域 per-def 预算、边长界、场景元素界（含恰 8
//! 合法界的 T-03 修正定向回归）、未知 prop/元素、float 绑定。
//! Run: cargo test -p autographics-node2d-pilot --test mixed_regions_lifecycle_gates
use autographics_node2d_pilot::{
    load_project_mixed, lower_project_node_mixed, lower_project_node_mixed_regions,
};
use std::fs;
use std::path;

const DASH: &str = include_str!("../fixtures/node-mixed-regions/dash.at");
const GAUGE: &str = include_str!("../fixtures/node-mixed-regions/gauge.at");
const METER: &str = include_str!("../fixtures/node-mixed-regions/meter.at");
const COMMITTED_UI_JSON: &str = include_str!(
    "../generated-node-mixed-regions/ui-dash-regions/ui.json"
);
const COMMITTED_WORLD: &str = include_str!(
    "../generated-node-mixed-regions/ui-dash-regions/world.at"
);

fn write(dir: &path::Path, files: &[(&str, &str)]) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower_v2(
    dir: &path::Path,
) -> Result<autographics_node2d_pilot::LowerOutputNodeMixed, String> {
    let project = load_project_mixed(&dir.join("dash.at")).map_err(|e| e.to_string())?;
    lower_project_node_mixed_regions(&project).map_err(|e| e.to_string())
}

fn scratch(tag: &str) -> path::PathBuf {
    path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("plan054-gates-{tag}-{}", std::process::id()))
}

// ------------------------------------------------------------ positives --

#[test]
fn regions_fixture_lowers_with_component_regions_and_world_abi() {
    let dir = scratch("pos");
    write(&dir, &[("dash", DASH), ("gauge", GAUGE), ("meter", METER)]);
    let out = lower_v2(&dir).expect("v2 fixture lowers");
    // 产物身份（T-02 冻结拼写）。
    assert!(out
        .ui_json
        .contains("\"schema\": \"autographics.node-ui-mixed-regions/2\""));
    assert!(out.ui_json.contains("plan054 node-ui-mixed-regions"));
    // 组件 def 内区域（实例挂载身份面）＋root 区域＋实例链（声明先序）。
    for needle in [
        "\"kind\": \"Embed\"",
        "\"name\": \"arc\"",
        "\"dim\": \"2d\"",
        "\"name\": \"dial\"",
        "\"dim\": \"3d\"",
        "\"name\": \"strip\"",
        "\"fn\": \"gauge_ui_click_arc\"",
        "\"fn\": \"meter_ui_click_dial\"",
        "\"fn\": \"dash_ui_click_strip\"",
        "\"path\": \"Dash/p1\"",
        "\"path\": \"Dash/p2\"",
        "\"path\": \"Dash/m3\"",
        "\"path\": \"Dash/p3\"",
        "\"kind\": \"quad\"",
        "\"kind\": \"cylinder\"",
    ] {
        assert!(out.ui_json.contains(needle), "ui.json missing `{needle}`");
    }
    // 世界 ABI：与 048/049 完全同面——组件派发器带实例下标、实例路径
    // getter、SoA 行、半缓冲文字、逐实例 Life.Tick、app 骨架。
    let src = &out.user_source;
    for needle in [
        "fn gauge_ui_click_arc(i int)",
        "fn meter_ui_click_dial(i int)",
        "fn dash_ui_click_strip()",
        "fn get_Dash_p1_x() int",
        "fn get_Dash_p2_x() int",
        "fn get_Dash_p3_x() int",
        "fn get_Dash_m3_tx() int",
        "fn get_r_strip_x() int",
        "fn gauge_lifetick(i int, dt float)",
        "fn meter_lifetick(i int, dt float)",
        "var gauge_x=[",
        "var meter_tx=[",
        "var dash_ui_text_0=[",
        "var dash_ui_text_b_0=[",
        "var dash_ui_text_len_0=[",
        "fn app_start()",
        "fn app_update(dt float)",
    ] {
        assert!(src.contains(needle), "world.at missing `{needle}`");
    }
}

#[test]
fn regeneration_is_byte_identical_to_committed_products() {
    let dir = scratch("regen");
    write(&dir, &[("dash", DASH), ("gauge", GAUGE), ("meter", METER)]);
    let out = lower_v2(&dir).expect("v2 fixture lowers");
    assert_eq!(out.ui_json, COMMITTED_UI_JSON, "ui.json must regenerate byte-identical");
    assert_eq!(out.user_source, COMMITTED_WORLD, "world.at must regenerate byte-identical");
}

// ------------------------------------------------------------- negatives --

#[test]
fn v1_profile_still_rejects_component_regions_with_frozen_diagnostic() {
    let dir = scratch("v1guard");
    write(&dir, &[("dash", DASH), ("gauge", GAUGE), ("meter", METER)]);
    let project = load_project_mixed(&dir.join("dash.at")).map_err(|e| e.to_string()).expect("loads");
    let err = lower_project_node_mixed(&project).expect_err("v1 must reject component regions");
    // 049 冻结诊断逐字（T-02 切片措辞不改）。
    assert!(
        err.contains(
            "region 'arc' in a non-root def requires instance-mount identity (frozen T-02 slice admits root regions only)"
        ),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn per_def_region_budget_is_named_rejection() {
    let mut regions = String::new();
    for n in 'a'..='f' {
        regions.push_str(&format!(
            "view-2d r{n} {{\n                width: 10\n                height: 10\n                shape2d q{n} {{ kind: quad, x: 0, y: 0, w: 4, h: 4, color: (1, 2, 3) }}\n            }}\n"
        ));
    }
    let src = format!(
        "node R {{\n    view {{\n        column root {{\n{regions}        }}\n    }}\n}}\nfn main() {{}}\n"
    );
    let dir = scratch("cap");
    write(&dir, &[("dash", &src)]);
    let err = lower_v2(&dir).expect_err("5+ regions must be named-rejected");
    assert!(
        err.contains("region count exceeds the frozen 4-region budget"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn region_side_budget_is_named_rejection() {
    for (side, tag) in [("641", "over"), ("0", "zero")] {
        let src = format!(
            "node R {{\n    view {{\n        column root {{\n            view-2d r {{\n                width: {side}\n                height: 10\n                shape2d q {{ kind: quad, x: 0, y: 0, w: 4, h: 4, color: (1, 2, 3) }}\n            }}\n        }}\n    }}\n}}\nfn main() {{}}\n"
        );
        let dir = scratch(tag);
        write(&dir, &[("dash", &src)]);
        let err = lower_v2(&dir).expect_err("side out of budget must be named-rejected");
        assert!(
            err.contains("must be 1..=640"),
            "unexpected diagnostic for {side}: {err}"
        );
    }
}

#[test]
fn scene_element_budget_has_exact_eight_as_legal_boundary() {
    // T-03 修正定向回归：恰 8 元素＝合法界（canonical「每区域 ≤8」；driver
    // 消费门同界）——旧 `>=` off-by-one 拒绝恰 8；修正后 8 绿、9 具名拒。
    let region = |count: usize| -> String {
        let mut els = String::new();
        for n in 0..count {
            els.push_str(&format!(
                "shape2d q{n} {{ kind: quad, x: 0, y: 0, w: 4, h: 4, color: (1, 2, 3) }}\n"
            ));
        }
        format!(
            "node R {{\n    view {{\n        column root {{\n            view-2d r {{\n                width: 60\n                height: 40\n                {els}}}\n        }}\n    }}\n}}\nfn main() {{}}\n"
        )
    };
    let dir8 = scratch("scene8");
    write(&dir8, &[("dash", &region(8))]);
    lower_v2(&dir8).expect("exactly 8 scene elements must lower");
    let dir9 = scratch("scene9");
    write(&dir9, &[("dash", &region(9))]);
    let err = lower_v2(&dir9).expect_err("9 scene elements must be named-rejected");
    assert!(
        err.contains("exceeds the frozen 8-element budget"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn unknown_region_props_and_scene_elements_are_named_rejections() {
    let base = |region_body: &str| -> String {
        format!(
            "node R {{\n    view {{\n        column root {{\n            view-2d r {{\n                width: 60\n                height: 40\n                {region_body}}}\n        }}\n    }}\n}}\nfn main() {{}}\n"
        )
    };
    // 未知区域 prop。
    let dir = scratch("prop");
    write(
        &dir,
        &[(
            "dash",
            &base("visible: 1
                shape2d q { kind: quad, x: 0, y: 0, w: 4, h: 4, color: (1, 2, 3) }"),
        )],
    );
    let err = lower_v2(&dir).expect_err("unknown region prop must be named-rejected");
    assert!(
        err.contains("unknown prop 'visible' on region 'r'"),
        "unexpected diagnostic: {err}"
    );
    // 跨维场景元素（解析面具名拒绝；2D 区域不接受 3D 图元）。
    let dir = scratch("kind");
    write(
        &dir,
        &[(
            "dash",
            &base("shape2d q { kind: quad, x: 0, y: 0, w: 4, h: 4, color: (1, 2, 3) }
                cylinder3d bad { translation: (0, 0, 0), color: (1, 2, 3) }"),
        )],
    );
    let err = lower_v2(&dir).expect_err("cross-dim scene element must be named-rejected");
    assert!(
        err.contains("unknown view-2d element 'cylinder3d'"),
        "unexpected diagnostic: {err}"
    );
    // 3D 区域接受集外 3D 元素（解析放行 model3d，降层冻结集只收 cylinder3d）。
    let base3d = |region_body: &str| -> String {
        format!(
            "node R {{\n    view {{\n        column root {{\n            view-3d r {{\n                width: 60\n                height: 40\n                {region_body}}}\n        }}\n    }}\n}}\nfn main() {{}}\n"
        )
    };
    let dir = scratch("kind3d");
    write(
        &dir,
        &[("dash", &base3d("cylinder3d ok { translation: (0, 0, 0), color: (1, 2, 3) }\n                model3d bad { model: \"m\" }"))],
    );
    let err = lower_v2(&dir).expect_err("out-of-set 3D element must be named-rejected");
    assert!(
        err.contains("outside the frozen T-02 3D set"),
        "unexpected diagnostic: {err}"
    );
    // float 场景绑定拒绝（int getter 面）。
    let dir = scratch("float");
    write(
        &dir,
        &[(
            "dash",
            &base("shape2d q { kind: quad, x: 0.5, y: 0, w: 4, h: 4, color: (1, 2, 3) }"),
        )],
    );
    let err = lower_v2(&dir).expect_err("float scene binding must be named-rejected");
    assert!(
        err.contains("must be an int or int field"),
        "unexpected diagnostic: {err}"
    );
}
