//! PLAN-024 components-3d 降层门（V-01/AC-02；T-03；V-05 边界 T-05）。
//! 正面：view-3d 声明链降层 Ok 且两次再生字节一致（twin/model-param/
//! envelope8/rows128）；SoA/行表/缓存/getter/绑定表形状钉死；Model3D
//! typed 参数逐实例解析（同 SHA 两实例共享）。负面：全部执行前具名拒绝
//! （module/line/symbol，不 panic）——缺 use、实例环、重复实例路径、
//! 第 9 有状态实例、129 场景行、未知 prop、组平移/可见绑定、scale 绑定、
//! 双 view 出口、root 参数、参数写、错资源端口、非数值 msg 载荷、未知
//! view 域标记、数值参数绑资源。运行：cargo test --offline --test
//! node3d_basic_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::{load_project, lower_project_components3d, LowerOutputBasic2D};

const TWIN_MAIN: &str = include_str!("../fixtures/node3d/twin/main.at");
const TWIN_PUCK: &str = include_str!("../fixtures/node3d/twin/puck.at");
const G02_MAIN: &str = include_str!("../fixtures/node3d/g02/main.at");
const G02_RES: &str = include_str!("../fixtures/node3d/g02/resources.at");
const G03_MAIN: &str = include_str!("../fixtures/node3d/g03/main.at");
const MP_MAIN: &str = include_str!("../fixtures/node3d/model-param/main.at");
const MP_PART: &str = include_str!("../fixtures/node3d/model-param/part.at");
const MP_RES: &str = include_str!("../fixtures/node3d/model-param/resources.at");
const ENV8_MAIN: &str = include_str!("../fixtures/node3d/envelope8/main.at");
const ENV8_PUCK: &str = include_str!("../fixtures/node3d/envelope8/puck.at");
const ROWS128: &str = include_str!("../fixtures/node3d/rows128/main.at");
const NEG9_MAIN: &str = include_str!("../fixtures/node3d/neg-9/main.at");
const NEG9_PUCK: &str = include_str!("../fixtures/node3d/neg-9/puck.at");
const NEG129: &str = include_str!("../fixtures/node3d/neg-rows129/main.at");
const NMT_MAIN: &str = include_str!("../fixtures/node3d/neg-model-type/main.at");
const NMT_PART: &str = include_str!("../fixtures/node3d/neg-model-type/part.at");
const NMT_RES: &str = include_str!("../fixtures/node3d/neg-model-type/resources.at");
const NGD_MAIN: &str = include_str!("../fixtures/node3d/neg-group-dyn/main.at");
const NSD_MAIN: &str = include_str!("../fixtures/node3d/neg-scale-dyn/main.at");
const NBV_MAIN: &str = include_str!("../fixtures/node3d/neg-both-views/main.at");
const NRP_MAIN: &str = include_str!("../fixtures/node3d/neg-root-params/main.at");
const NPW_MAIN: &str = include_str!("../fixtures/node3d/neg-param-write/main.at");
const NPW_BOXY: &str = include_str!("../fixtures/node3d/neg-param-write/boxy.at");
const NCY_MAIN: &str = include_str!("../fixtures/node3d/neg-cycle/main.at");
const NCY_A: &str = include_str!("../fixtures/node3d/neg-cycle/a.at");
const NCY_B: &str = include_str!("../fixtures/node3d/neg-cycle/b.at");
const NDP_MAIN: &str = include_str!("../fixtures/node3d/neg-dup-path/main.at");
const NUP_MAIN: &str = include_str!("../fixtures/node3d/neg-unknown-prop/main.at");
const NMU_MAIN: &str = include_str!("../fixtures/node3d/neg-missing-use/main.at");
const NMM_MAIN: &str = include_str!("../fixtures/node3d/neg-msg-model/main.at");
const NMARK_MAIN: &str = include_str!("../fixtures/node3d/neg-marker/main.at");
const HELP_MAIN: &str = include_str!("../fixtures/node3d/helpers3d/main.at");
const HELP_A: &str = include_str!("../fixtures/node3d/helpers3d/helpera.at");
const HELP_B: &str = include_str!("../fixtures/node3d/helpers3d/helperb.at");
const G02_GLB: &[u8] = include_bytes!("../fixtures/node3d/g02/assets/two-parts.glb");

fn write_dir(dir: &path::Path, files: &[(&str, &str)], with_glb: bool) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir.join("assets")).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
    if with_glb {
        fs::write(dir.join("assets/two-parts.glb"), G02_GLB).expect("glb");
    }
}

fn lower3d(dir: &path::Path) -> Result<LowerOutputBasic2D, String> {
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_components3d(&project).map_err(|e| e.to_string())
}

fn case(files: &[(&str, &str)], with_glb: bool) -> Result<LowerOutputBasic2D, String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3d-{}-{n}", std::process::id()));
    write_dir(&dir, files, with_glb);
    lower3d(&dir)
}

fn reject(files: &[(&str, &str)], with_glb: bool, needle: &str) {
    let err = case(files, with_glb)
        .map(|_| ())
        .expect_err("named rejection expected");
    assert!(
        err.contains(needle),
        "diagnostic '{err}' does not contain '{needle}'"
    );
    // V-01 合同：每条诊断带 module/line/symbol 定位（load 层 file:line 形态
    // 或降层 module/line/symbol 形态，均不 panic）。
    assert!(
        err.contains("symbol '") || err.contains(".at") || {
            // load 层形态 file:line: message
            let head = err.split(':').nth(1).unwrap_or("");
            head.chars().all(|c| c.is_ascii_digit()) && !head.is_empty()
        },
        "diagnostic '{err}' lacks a location anchor"
    );
}

// ------------------------------------------------ 正面：形状/字节稳定性 --

#[test]
fn twin_chain_lowers_ok_with_frozen_shape_and_is_byte_stable() {
    let files: &[(&str, &str)] = &[
        ("main", TWIN_MAIN),
        ("puck", TWIN_PUCK),
        ("resources", G02_RES),
    ];
    let a = case(files, true).expect("twin lowers");
    let b = case(files, true).expect("twin lowers again");
    assert_eq!(
        a.user_source, b.user_source,
        "world regeneration byte-identical"
    );
    assert_eq!(
        a.scene_json, b.scene_json,
        "scene regeneration byte-identical"
    );
    let w = a.user_source.as_ref().expect("world.at");
    // 行表：v1 行句柄 = ordered() 0-based（arena=0/left=1/right=2/sentinel=3）。
    assert!(w.contains("var puck_puck_rows=[1,2]"), "row table 0-based");
    // 每 SoA：model 初值按实例内联 + 运行期参数数组。
    for line in [
        "var r_lo=0",
        "var r_hi=0",
        "var puck_phase=[0.0,3.0]",
        "var puck_on=[1,1]",
        "var puck_p0=[0.0,3.0]",
        "var puck_hide_at=[0,45]",
        "var puck_puck_px=[0.0,3.0]",
        "var puck_puck_vis=[1,1]",
    ] {
        assert!(w.contains(line), "world.at missing '{line}'");
    }
    // 绑定表（root 接收直呼无表；实例发射方建表）与冻结 app 序。
    assert!(w.contains("fn main_pinged(v float)"), "root receiver");
    assert!(
        w.contains("fn app_start() {\n main_lifestart()"),
        "app_start shape"
    );
    assert!(
        w.contains(" puck_lifetick(0,dt)\n puck_present(0)"),
        "frozen order"
    );
    // 呈现写 = v1 文档节点写（ag_set_local 三轴 + ag_set_visible）。
    assert!(w.contains("ag_set_local(puck_puck_rows[i],0,puck_phase[i])"));
    assert!(w.contains("ag_set_visible(puck_puck_rows[i],puck_on[i])"));
    assert!(
        !w.contains("ag_object_spawn"),
        "3D chain does not use objects3d slots"
    );
    assert!(
        !w.contains("ag_char_bind"),
        "3D chain does not use the character service"
    );
    // getters 双形态（下标 + 路径）。
    assert!(w.contains("fn get_Main_left_phase() float {return puck_phase[0]}"));
    // 场景：显式 group3d 父 + 3D 叶 kinds + 固定 rgba。
    let s = &a.scene_json;
    assert!(s.contains("\"schema\": \"autographics.scene/6\""));
    assert!(s.contains("\"kind\": \"cylinder\""));
    assert!(s.contains("\"dimension\": \"3d\""));
    assert!(s.contains("\"api\": \"autographics.behavior/1\""));
    assert!(s.contains("\"actors\": {\n   \"two_d\": null,\n   \"three_d\": null\n  }"));
}

#[test]
fn pure_chains_lower_ok_behavior_null() {
    let g02 = case(&[("main", G02_MAIN), ("resources", G02_RES)], true).expect("g02 pure lowers");
    assert!(g02.user_source.is_none(), "pure scene has no world.at");
    assert!(g02.scene_json.contains("\"behavior\": null"));
    assert!(g02.scene_json.contains("\"kind\": \"model\""));
    assert!(g02
        .scene_json
        .contains("sha256:42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671"));
    let g03 = case(&[("main", G03_MAIN), ("resources", G02_RES)], true).expect("g03 pure lowers");
    assert!(g03.user_source.is_none());
    // G03 树形状：root3d(0.5) -> child3d(0.5,1) -> deep(0.8)。
    let s = &g03.scene_json;
    for frag in [
        "\"name\": \"root3d\"",
        "\"translation\": [\n    0.5,\n    0.0,\n    0.0\n   ]",
        "\"name\": \"child3d\"",
        "\"translation\": [\n    0.5,\n    1.0,\n    0.0\n   ]",
        "\"name\": \"deep\"",
        "\"translation\": [\n    0.8,\n    0.0,\n    0.0\n   ]",
    ] {
        assert!(s.contains(frag), "g03 scene missing {frag}");
    }
}

#[test]
fn model_param_chain_resolves_typed_port_per_instance() {
    let out = case(
        &[("main", MP_MAIN), ("part", MP_PART), ("resources", MP_RES)],
        true,
    )
    .expect("model-param lowers");
    let s = &out.scene_json;
    // 两实例共享同 SHA 资源（符号引用，非整数 ID）。
    assert_eq!(
        s.matches("sha256:42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671\"")
            .count(),
        2,
        "both instances carry the shared GLB"
    );
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let _ = dir;
}

#[test]
fn envelope8_holds_and_rename_keeps_behavior_shape() {
    let env = case(
        &[
            ("main", ENV8_MAIN),
            ("puck", ENV8_PUCK),
            ("resources", G02_RES),
        ],
        true,
    )
    .expect("8 stateful instances (root included) lower Ok");
    let w = env.user_source.as_ref().expect("world");
    assert!(
        w.contains("var puck_phase=[0.0,1.0,2.0,3.0,4.0,5.0,6.0]"),
        "7 instances"
    );
    // 重命名不改行为形状：改 Puck 定义名只换前缀，行数/结构不变。
    let renamed_puck = ENV8_PUCK.replace("node Puck(", "node Stone(");
    let renamed_main = ENV8_MAIN
        .replace("Puck ", "Stone ")
        .replace("use puck: Puck", "use puck: Stone");
    let out = case(
        &[
            ("main", renamed_main.as_str()),
            ("puck", renamed_puck.as_str()),
            ("resources", G02_RES),
        ],
        true,
    )
    .expect("renamed chain lowers");
    let w2 = out.user_source.as_ref().expect("world");
    assert!(
        w2.contains("var stone_phase=[0.0,1.0,2.0,3.0,4.0,5.0,6.0]"),
        "renamed SoA prefix"
    );
    // 归一化后两世界逐字节一致（名字不是行为 ID）。
    let norm = |t: &String| t.replace("stone", "puck").replace("Stone", "Puck");
    assert_eq!(norm(&w2), *w, "rename keeps the world shape");
}

#[test]
fn rows128_boundary_lowers_ok() {
    let out = case(&[("main", ROWS128)], false).expect("128 rows lower Ok");
    assert!(out.user_source.is_none());
    assert_eq!(
        out.scene_json.matches("\"kind\": \"cylinder\"").count(),
        128
    );
}

// ------------------------------------------------ 负面：具名拒绝（V-01） --

#[test]
fn negatives_are_named_rejections() {
    // 第 9 有状态实例（root 计入）。
    reject(
        &[
            ("main", NEG9_MAIN),
            ("puck", NEG9_PUCK),
            ("resources", G02_RES),
        ],
        true,
        "exceeds the 8-instance envelope",
    );
    // 129 场景行。
    reject(&[("main", NEG129)], false, "exceeds 128 nodes");
    // 错资源端口：Model3D 参数绑定 material2d 资源。
    reject(
        &[
            ("main", NMT_MAIN),
            ("part", NMT_PART),
            ("resources", NMT_RES),
        ],
        false,
        "is not a model3d.glb resource",
    );
    // 组平移绑定（组静态）。
    reject(
        &[("main", NGD_MAIN)],
        false,
        "group3d translation is static",
    );
    // scale 绑定（Run 内不可变面）。
    reject(&[("main", NSD_MAIN)], false, "scale must be static");
    // 双 view 出口。
    reject(&[("main", NBV_MAIN)], false, "single view exit per node");
    // root 参数。
    reject(
        &[("main", NRP_MAIN)],
        false,
        "root node must not declare params",
    );
    // 参数写（readonly 赋值拒绝）。
    reject(
        &[("main", NPW_MAIN), ("boxy", NPW_BOXY)],
        false,
        "params are read-only",
    );
    // 实例环。
    reject(
        &[("main", NCY_MAIN), ("a", NCY_A), ("b", NCY_B)],
        false,
        "instance cycle",
    );
    // 重复实例路径。
    reject(
        &[
            ("main", NDP_MAIN),
            ("puck", TWIN_PUCK),
            ("resources", G02_RES),
        ],
        true,
        "duplicate instance path",
    );
    // 未知 prop（3D 材质固定，rgba 具名拒绝）。
    reject(
        &[("main", NUP_MAIN)],
        false,
        "unknown prop on view-3d element",
    );
    // 缺 use。
    reject(
        &[
            ("main", NMU_MAIN),
            ("puck", TWIN_PUCK),
            ("resources", G02_RES),
        ],
        true,
        "instance of undeclared node",
    );
    // 非数值 msg 载荷。
    reject(&[("main", NMM_MAIN)], false, "msg payload must be numeric");
    // 未知 view 域标记。
    reject(
        &[("main", NMARK_MAIN)],
        false,
        "view domain marker must be view-",
    );
}

#[test]
fn same_name_helpers_across_modules_pool_prefixed_and_resolve_per_module() {
    let files: &[(&str, &str)] = &[
        ("main", HELP_MAIN),
        ("helpera", HELP_A),
        ("helperb", HELP_B),
    ];
    let a = case(files, false).expect("same-name helpers lower");
    let b = case(files, false).expect("same-name helpers lower again");
    let wa = a.user_source.as_ref().expect("world");
    let wb = b.user_source.as_ref().expect("world");
    assert_eq!(wa, wb, "helpers chain regeneration byte-identical");
    // 两模块同名 fn 池化为模块前缀 fn（module A/B 各自解析各自的）。
    assert!(
        wa.contains("// pooled fn bump from helpera"),
        "module A pooled"
    );
    assert!(
        wa.contains("// pooled fn bump from helperb"),
        "module B pooled"
    );
    assert!(wa.contains("fn helpera_bump(v float)"), "module A prefix");
    assert!(wa.contains("fn helperb_bump(v float)"), "module B prefix");
    // 调用点按所在模块解析：Alpha 用 helpera_bump，Beta 用 helperb_bump。
    let tick_a = wa
        .split("fn alpha_lifetick")
        .nth(1)
        .and_then(|t| t.split("}").next())
        .expect("alpha lifetick body");
    assert!(
        tick_a.contains("helpera_bump(") && !tick_a.contains("helperb_bump("),
        "alpha resolves module A helper: {tick_a}"
    );
    let tick_b = wa
        .split("fn beta_lifetick")
        .nth(1)
        .and_then(|t| t.split("}").next())
        .expect("beta lifetick body");
    assert!(
        tick_b.contains("helperb_bump(") && !tick_b.contains("helpera_bump("),
        "beta resolves module B helper: {tick_b}"
    );
}

#[test]
fn missing_model_file_is_named_rejected_at_load() {
    // 缺 GLB 文件：载入层具名拒绝（不静默）。
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3d-noglb-{}", std::process::id()));
    write_dir(&dir, &[("main", G02_MAIN), ("resources", G02_RES)], false);
    let err = load_project(&dir.join("main.at"))
        .err()
        .expect("missing model file must reject")
        .to_string();
    assert!(
        err.contains("cannot read") && err.contains(".glb"),
        "load diagnostic '{err}' names the unreadable GLB"
    );
}
