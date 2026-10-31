//! PLAN-019 basic2d 降层门（V-01/AC-02；T-02 冻结面）。
//! 负面：profile 越界（scene-2d 的角色/solid/相机/逻辑、character-2d 的
//! 相机、错相机目标、坏 kind/尺寸/属性/资源、重复名）执行前定位拒绝；
//! 正面：再生字节等价、声明序=场景序、组件参数敏感性。运行：
//! cargo test --offline --test node2d_basic_gates
use std::fs;
use std::path;

const G01_MAIN: &str = include_str!("../fixtures/basic2d/g01/main.at");
const G01_RES: &str = include_str!("../fixtures/basic2d/g01/resources.at");
const G01_SHAPES: &str = include_str!("../fixtures/basic2d/g01/shapes.at");
const G03_MAIN: &str = include_str!("../fixtures/basic2d/g03/main.at");
const G03_RES: &str = include_str!("../fixtures/basic2d/g03/resources.at");
const G06_MAIN: &str = include_str!("../fixtures/basic2d/g06/main.at");
const G06_RES: &str = include_str!("../fixtures/basic2d/g06/resources.at");
const G06_WALK: &str = include_str!("../fixtures/basic2d/g06/walkjump.at");
const G08_MAIN: &str = include_str!("../fixtures/basic2d/g08/main.at");
const G08_RES: &str = include_str!("../fixtures/basic2d/g08/resources.at");
const G08_WALK: &str = include_str!("../fixtures/basic2d/g08/walkjump.at");

use autographics_node2d_pilot::{load_project, lower_project_basic, BasicProfile};

fn write(dir: &path::Path, files: &[(&str, &str)]) {
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower(
    dir: &path::Path,
    entry: &str,
    profile: BasicProfile,
) -> Result<autographics_node2d_pilot::LowerOutputBasic2D, String> {
    let project = load_project(&dir.join(entry)).map_err(|e| e.to_string())?;
    lower_project_basic(&project, profile).map_err(|e| e.to_string())
}

fn scene2d_tag(tag: &str, files: &[(&str, &str)]) -> Result<autographics_node2d_pilot::LowerOutputBasic2D, String> {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("b2g-{tag}"));
    let _ = fs::remove_dir_all(&dir);
    write(&dir, files);
    lower(&dir, "main.at", BasicProfile::Scene2D)
}

fn scene2d(files: &[(&str, &str)]) -> Result<autographics_node2d_pilot::LowerOutputBasic2D, String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    scene2d_tag(&format!("t{n}"), files)
}

fn character2d(
    files: &[(&str, &str)],
) -> Result<autographics_node2d_pilot::LowerOutputBasic2D, String> {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("b2c-{}", std::process::id()) + &format!("-{}", files.as_ptr() as usize));
    let _ = fs::remove_dir_all(&dir);
    write(&dir, files);
    lower(&dir, "main.at", BasicProfile::Character2D)
}

fn camera2d(
    files: &[(&str, &str)],
) -> Result<autographics_node2d_pilot::LowerOutputBasic2D, String> {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("b2k-{}", files.as_ptr() as usize));
    let _ = fs::remove_dir_all(&dir);
    write(&dir, files);
    lower(&dir, "main.at", BasicProfile::Camera2D)
}

fn g01() -> Vec<(&'static str, &'static str)> {
    vec![("main", G01_MAIN), ("resources", G01_RES), ("shapes", G01_SHAPES)]
}

// ------------------------------------------------------- 正面：基线与再生 --

#[test]
fn g01_base_scene_matches_original_baseline_data() {
    let out = scene2d(&g01()).expect("g01 lowers");
    assert!(out.user_source.is_none(), "scene-2d has no user source");
    // 原 dbb4b75 数据：red(-20,10) d30 a0.5 / blue(0,0) 30 a0.5 /
    // green(25,-15) d24 a0.6 / yellow(-45,-25) 12 a1.0 hidden。
    for (name, tx, ty, sx, sy, a, visible) in [
        ("red_disc.disc", -20.0, 10.0, 30.0, 30.0, 0.5, true),
        ("blue_quad", 0.0, 0.0, 30.0, 30.0, 0.5, true),
        ("green_disc.disc", 25.0, -15.0, 24.0, 24.0, 0.6, true),
        ("yellow_quad", -45.0, -25.0, 12.0, 12.0, 1.0, false),
    ] {
        let row = extract_object(&out.scene_json, name);
        assert_eq!(row.tx, tx, "{name} x");
        assert_eq!(row.ty, ty, "{name} y");
        assert_eq!(row.sx, sx, "{name} w");
        assert_eq!(row.sy, sy, "{name} h");
        assert!((row.a - a).abs() < 1e-9, "{name} alpha");
        assert_eq!(row.visible, visible, "{name} visible");
    }
    assert!(out.scene_json.contains("\"behavior\": null"), "no behavior");
}

#[test]
fn regeneration_is_byte_identical_and_ids_follow_declaration() {
    let a = scene2d(&g01()).expect("first");
    let b = scene2d(&g01()).expect("second");
    assert_eq!(a.scene_json, b.scene_json, "scene regeneration");
    let c = character2d(&g06_files()).expect("g06 first");
    let d = character2d(&g06_files()).expect("g06 second");
    assert_eq!(c.scene_json, d.scene_json, "g06 scene regeneration");
    assert_eq!(c.user_source, d.user_source, "g06 source regeneration");
    // 声明序 = 场景 id 序（角色宿主声明第一 → id 1，solids 2..5）。
    assert!(c.scene_json.contains("\"id\": 1,\n   \"name\": \"body.body\""));
    assert!(c.scene_json.contains("\"id\": 2,\n   \"name\": \"ground\""));
    assert!(c.scene_json.contains("\"id\": 5,\n   \"name\": \"block\""));
}

#[test]
fn g03_groups_carry_parent_chain_and_visibility() {
    let out = scene2d(&[("main", G03_MAIN), ("resources", G03_RES)]).expect("g03");
    // root(20,10) > child(10,-5)；deep quad(5,0) 挂 child，leaf(0,0) 挂 root。
    assert!(out.scene_json.contains("\"id\": 1,\n   \"name\": \"root_2d\""));
    // group JSON：parent 在 translation 前。
    assert!(out
        .scene_json
        .contains("\"parent\": 1,\n   \"translation\": [10.0, -5.0, 0.0]"));
    // object JSON：translation 在 parent 前；deep_quad 挂 child(id2)，leaf 挂 root(id1)。
    let deep_seg = segment_after(&out.scene_json, "\"name\": \"deep_quad\"");
    assert!(
        deep_seg.contains("\"translation\": [5.0, 0.0, 0.0]") && deep_seg.contains("\"parent\": 2"),
        "deep: {deep_seg}"
    );
    let leaf_seg = segment_after(&out.scene_json, "\"name\": \"leaf_circle\"");
    assert!(
        leaf_seg.contains("\"translation\": [0.0, 0.0, 0.0]") && leaf_seg.contains("\"parent\": 1"),
        "leaf: {leaf_seg}"
    );
}

#[test]
fn component_param_change_moves_expanded_object() {
    let mut moved = G01_MAIN.replace("x: 25.0\n            y: -15.0", "x: 11.0\n            y: 4.0");
    moved = moved.replace("diameter: 24.0", "diameter: 10.0");
    let out = scene2d(&[
        ("main", moved.as_str()),
        ("resources", G01_RES),
        ("shapes", G01_SHAPES),
    ])
    .expect("moved instance");
    let row = extract_object(&out.scene_json, "green_disc.disc");
    assert_eq!((row.tx, row.ty, row.sx), (11.0, 4.0, 10.0), "param sensitivity");
}

fn g06_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("main", G06_MAIN),
        ("resources", G06_RES),
        ("walkjump", G06_WALK),
    ]
}

// ------------------------------------------------------- 负面：profile 门 --

#[test]
fn scene2d_rejects_character_solid_camera_and_logic() {
    for (tag, main) in [
        (
            "character",
            G01_MAIN.replace("view-2d {", "view-2d {\n        character2d body { position: (0.0, 0.0)  material: resources.red_alpha }"),
        ),
        (
            "solid",
            G01_MAIN.replace("view-2d {", "view-2d {\n        solid2d floor { position: (0.0, -1.0)  mesh: resources.blue_quad_m  material: resources.red_alpha }"),
        ),
        (
            "camera",
            G01_MAIN.replace("view-2d {", "view-2d {\n        camera2d cam {
            target: refs.body.actor
            bounds: (0.0, -4.0, 30.0, 12.0)
            view_size: (12.0, 8.0)
        }"),
        ),
    ] {
        let err = scene2d(&[
            ("main", main.as_str()),
            ("resources", G01_RES),
            ("shapes", G01_SHAPES),
        ])
        .err()
        .unwrap_or_else(|| panic!("scene-2d must reject {tag}"));
        assert!(!err.is_empty(), "{tag} diagnostic present");
    }
    // scene-2d 根节点带 on：拒绝（无 Tick 必要性）。
    let logical = G01_MAIN.replace(
        "node Main {\n    view-2d {",
        "node Main {\n    on {\n        Life.Tick(dt) -> {}\n    }\n    view-2d {",
    );
    let err = scene2d(&[
        ("main", logical.as_str()),
        ("resources", G01_RES),
        ("shapes", G01_SHAPES),
    ])
    .err()
    .expect("scene-2d must reject root on");
    assert!(err.contains("root") || !err.is_empty());
}

#[test]
fn character2d_requires_exactly_one_character_and_no_camera() {
    let missing_use = G06_MAIN.replace("use walkjump: Player\n", "");
    let err = character2d(&[("main", missing_use.as_str()), ("resources", G06_RES)])
        .err()
        .expect("missing module rejected");
    assert!(!err.is_empty(), "missing Player module rejected: {err}");
    // 0 角色：Player 组件 view 为空（实例在、角色宿主不在）。
    let nochar_player = G06_WALK.replace(
        "        character2d body {\n            position: (px, py)\n            material: tint\n        }\n",
        "",
    );
    let err = character2d(&[
        ("main", G06_MAIN),
        ("resources", G06_RES),
        ("walkjump", nochar_player.as_str()),
    ])
    .err()
    .expect("0 characters rejected");
    assert!(err.contains("exactly one character2d"), "{err}");
    // character-2d + camera2d：拒绝。
    let withcam = G06_MAIN.replace(
        "    }\n    on {",
        "        camera2d cam {
            target: refs.body.actor
            bounds: (0.0, -4.0, 30.0, 12.0)
            view_size: (12.0, 8.0)
        }\n    }\n    on {",
    );
    let err = character2d(&[
        ("main", withcam.as_str()),
        ("resources", G06_RES),
        ("walkjump", G06_WALK),
    ])
    .err()
    .expect("character-2d camera rejected");
    assert!(err.contains("camera2d"), "{err}");
}

#[test]
fn camera2d_requires_exactly_one_camera_with_valid_target() {
    // camera-2d 无相机。
    let nocam = G08_MAIN.replace(
        "        camera2d camera {\n            target: refs.body.actor\n            bounds: (0.0, -4.0, 30.0, 12.0)\n            view_size: (12.0, 8.0)\n        }\n",
        "",
    );
    let files = || vec![
        ("main", nocam.as_str()),
        ("resources", G08_RES),
        ("walkjump", G08_WALK),
    ];
    let err = camera2d(&files()).err().expect("camera-2d needs a camera");
    assert!(err.contains("exactly one camera2d"), "{err}");
    // 错目标。
    let badtarget = G08_MAIN.replace("refs.body.actor", "refs.ghost.actor");
    let files2 = vec![
        ("main", badtarget.as_str()),
        ("resources", G08_RES),
        ("walkjump", G08_WALK),
    ];
    let err = camera2d(&files2).err().expect("bad target rejected");
    assert!(err.contains("must reference") || err.contains("unknown"), "{err}");
}

#[test]
fn shape_gates_reject_bad_kind_size_prop_and_resource() {
    fn files(main: &str) -> Vec<(&str, &str)> {
        vec![("main", main), ("resources", G01_RES), ("shapes", G01_SHAPES)]
    }
    // 坏 kind。
    let bad = G01_MAIN.replace("kind: quad", "kind: triangle");
    let err = scene2d(&files(bad.as_str())).err().expect("bad kind");
    assert!(err.contains("triangle") || err.contains("kind"), "{err}");
    // circle 非等边（直声明 circle 用不等 size）。
    let uneven = "use resources: red_alpha\n\nnode Main {\n    view-2d {\n        shape2d odd_circle {\n            kind: circle\n            position: (0.0, 0.0)\n            size: (30.0, 12.0)\n            material: resources.red_alpha\n        }\n    }\n}";
    let err = scene2d(&[("main", uneven), ("resources", G01_RES)]).err().expect("uneven circle");
    assert!(err.contains("size.x==size.y"), "{err}");
    // 未知属性。
    let unknown = G01_MAIN.replace("visible: false", "visible: false\n            glow: true");
    let err = scene2d(&files(unknown.as_str())).err().expect("unknown prop");
    assert!(err.contains("glow"), "{err}");
    // 未知资源。
    let nores = G01_MAIN.replace("resources.blue_alpha", "resources.ghost_pink");
    let err = scene2d(&files(nores.as_str())).err().expect("unknown resource");
    assert!(err.contains("ghost_pink"), "{err}");
    // 重复元素名。
    let dup = G01_MAIN.replace("shape2d yellow_quad {", "shape2d blue_quad {");
    let err = scene2d(&files(dup.as_str())).err().expect("duplicate name");
    assert!(err.contains("duplicate view element name"), "{err}");
}

// ---------------------------------------------------------------- 工具 --

/// 取 name 标记之后到本对象行结束前的片段（parent/translation 同段断言用）。
fn segment_after<'a>(scene: &'a str, marker: &str) -> &'a str {
    let pos = scene.find(marker).expect(marker);
    let rest = &scene[pos..];
    let end = rest.find("\n  }").unwrap_or(rest.len());
    &rest[..end]
}

struct Row {
    tx: f64,
    ty: f64,
    sx: f64,
    sy: f64,
    a: f64,
    visible: bool,
}

/// 极简场景对象行解析（生成 JSON 形状冻结；不引入 JSON 依赖）。
fn extract_object(scene: &str, name: &str) -> Row {
    let marker = format!("\"name\": \"{name}\"");
    let pos = scene.find(&marker).unwrap_or_else(|| panic!("{name} in scene"));
    let seg = &scene[pos..];
    let num_after = |key: &str| -> f64 {
        let ki = seg.find(key).expect(key);
        let rest = &seg[ki + key.len()..];
        let rest = rest.trim_start_matches(|c: char| c == '[' || c == ' ' || c == ',');
        let end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(rest.len());
        rest[..end].parse().expect("number")
    };
    let tx = num_after("\"translation\": [");
    let ty = {
        let ki = seg.find("\"translation\": [").expect("translation");
        let rest = &seg[ki..];
        let comma = rest.find(',').expect("comma");
        let rest2 = rest[comma + 1..].trim_start();
        let end = rest2
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(rest2.len());
        rest2[..end].parse().expect("ty")
    };
    let sx = num_after("\"scale\": [");
    let sy = {
        let ki = seg.find("\"scale\": [").expect("scale");
        let rest = &seg[ki..];
        let comma = rest.find(',').expect("comma");
        let rest2 = rest[comma + 1..].trim_start();
        let end = rest2
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(rest2.len());
        rest2[..end].parse().expect("sy")
    };
    // alpha = rgba 第 4 分量。
    let ki = seg.find("\"rgba\": [").expect("rgba");
    let rest = &seg[ki..];
    let mut commas = 0;
    let mut idx = 0;
    for (i, c) in rest.char_indices() {
        if c == ',' {
            commas += 1;
            if commas == 3 {
                idx = i + 1;
                break;
            }
        }
    }
    let rest2 = rest[idx..].trim_start();
    let end = rest2
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest2.len());
    let a: f64 = rest2[..end].parse().expect("alpha");
    let visible = seg.find("\"visible\": true").is_some();
    Row {
        tx,
        ty,
        sx,
        sy,
        a,
        visible,
    }
}
