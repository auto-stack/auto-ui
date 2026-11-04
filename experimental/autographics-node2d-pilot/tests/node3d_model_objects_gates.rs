//! PLAN-043 node3d-model-objects 降层门（V-S01 编译面；AC-01/02/05）。
//! 正面：g12-model actor-free 声明链降层 Ok 且两次再生长字节一致；
//! model3d 模板行携带具名 GLB 资源键（sha256: 平色身份）并与 builtin
//! 模板共用声明序模板 id；typed spawn/move/visible/despawn 端口 1:1
//! 映射、两个同模板槽各自句柄。负面：执行前具名拒绝——未知资源名/
//! model3d 带 solid/参数化 model 绑定/textured3d.glb 档拒绝面保持。
//! 运行：cargo test --offline --test node3d_model_objects_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::{
    load_project, lower_project_components3d_events, LowerOutputBasic2D,
};

const G12_MAIN: &str = include_str!("../fixtures/components3d-events/g12-model/main.at");
const G12_PROPS: &str = include_str!("../fixtures/components3d-events/g12-model/props.at");
const G12_RESOURCES: &str =
    include_str!("../fixtures/components3d-events/g12-model/resources.at");
const G12_SENTINEL: &str =
    include_str!("../fixtures/components3d-events/g12-model/sentinel.at");

fn write_dir(dir: &path::Path, files: &[(&str, &str)]) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower_events(dir: &path::Path) -> Result<LowerOutputBasic2D, String> {
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_components3d_events(&project).map_err(|e| e.to_string())
}

fn case(files: &[(&str, &str)]) -> Result<LowerOutputBasic2D, String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3dm-{}-{n}", std::process::id()));
    write_dir(&dir, files);
    lower_events(&dir)
}

fn reject(files: &[(&str, &str)], needle: &str) {
    let err = case(files).map(|_| ()).expect_err("named rejection expected");
    assert!(
        err.contains(needle),
        "diagnostic '{err}' does not contain '{needle}'"
    );
    assert!(
        err.contains("symbol '") || err.contains(".at") || err.contains("at line "),
        "diagnostic '{err}' lacks a location anchor"
    );
}

// ------------------------------------------- 正面：model3d 模板行/资源身份 --

#[test]
fn g12_model_template_lowers_with_named_resource_and_is_byte_stable() {
    // 资源文件走真实夹具目录（load 以 main.at 所在目录为 base 读 assets/）。
    let a_dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("n3dm-g12-fixture");
    fs::create_dir_all(&a_dir).expect("dir");
    fs::write(a_dir.join("main.at"), G12_MAIN).expect("main");
    fs::write(a_dir.join("props.at"), G12_PROPS).expect("props");
    fs::write(a_dir.join("resources.at"), G12_RESOURCES).expect("resources");
    fs::write(a_dir.join("sentinel.at"), G12_SENTINEL).expect("sentinel");
    let assets = a_dir.join("assets");
    fs::create_dir_all(&assets).expect("assets dir");
    fs::write(
        assets.join("two-parts.glb"),
        &include_bytes!("../fixtures/node3d/g02/assets/two-parts.glb")[..],
    )
    .expect("glb");
    let a = lower_events(&a_dir).expect("g12-model lowers");
    let b = lower_events(&a_dir).expect("g12-model lowers again");
    assert_eq!(a.source_sha256, b.source_sha256, "two-lower byte equality");

    // 模板行 = parked model3d 行：kind "model" + 具名 GLB 的 sha256 身份键。
    let scene = &a.scene_json;
    assert!(
        scene.contains("\"name\": \"Crate\",\n   \"kind\": \"model\""),
        "{scene}"
    );
    assert!(
        scene.contains(
            "\"resource\": \"sha256:42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671\""
        ),
        "{scene}"
    );
    // parked：模板行可见性来自作者声明（false）。
    assert!(
        scene.contains("\"visible\": false"),
        "parked template row must be author-declared invisible: {scene}"
    );

    // typed 端口：两槽同模板（资源共享、句柄/状态独立），生命周期 1:1。
    let world = a.user_source.as_ref().expect("behavior world");
    assert!(
        world.contains("r_h = ag_object_spawn(0, 3, 2.0, 0.5, 0.0)"),
        "{world}"
    );
    assert!(
        world.contains("r_h2 = ag_object_spawn(1, 3, 6.0, 0.5, 0.0)"),
        "{world}"
    );
    assert!(world.contains("ag_object_move("), "{world}");
    assert!(world.contains("ag_object_visible("), "{world}");
    assert!(world.contains("ag_object_despawn("), "{world}");
}

// ------------------------------------------------ 负面：执行前具名拒绝 --

#[test]
fn model_template_rejections_are_named() {
    // 未知具名资源：经 034 公开准入链具名拒绝（定位到模板叶）。
    let main = "use props: Crate\nnode Main {\n    view-3d {\n        cylinder3d pad { position: (4.0, 0.5, 0.0) }\n    }\n    camera3d { distance: 8.0\n              pitch: 45.0\n              projection: perspective }\n    objects3d { crate_a }\n}\n";
    let props = "node Crate {\n    object3d { }\n    view-3d { model3d crate { model: resources.nope\n                              position: (-30.0, 0.5, -3.0)\n                              visible: false } }\n}\n";
    reject(&[("main", main), ("props", props)], "not a model3d.glb resource");

    // model3d 模板叶带 solid：沿用 027 拒绝域（solid 是 cylinder3d 专属）。
    let props_solid = "node Crate {\n    object3d { }\n    view-3d { model3d crate { model: resources.nope\n                              position: (-30.0, 0.5, -3.0)\n                              solid: true } }\n}\n";
    reject(
        &[("main", main), ("props", props_solid)],
        "'solid' is a cylinder3d prop, not model3d",
    );

    // 模板定义禁参数面不变：model3d 参数化绑定（self 无参）具名拒绝。
    let props_param = "node Crate {\n    object3d { }\n    view-3d { model3d crate { model: self.skin\n                              position: (-30.0, 0.5, -3.0)\n                              visible: false } }\n}\n";
    reject(
        &[("main", main), ("props", props_param)],
        "model param binding references unknown param 'skin'",
    );

    // 028 拒绝域保持：模板声明 model/msg/on 仍拒绝（model3d 资源绑定不
    // 是定义 model 面——`model { }` 状态块与 `model:` 叶 prop 是两个语法位）。
    let props_model_block = "node Crate {\n    object3d { }\n    model { var n = 0 }\n    view-3d { model3d crate { model: resources.nope\n                              position: (-30.0, 0.5, -3.0)\n                              visible: false } }\n}\n";
    reject(
        &[("main", main), ("props", props_model_block)],
        "must not declare model/msg/on",
    );
}

#[test]
fn textured_ctor_rejection_domain_is_preserved_in_events_profile() {
    // textured3d.glb 在 events 档保持具名拒绝（旧档拒绝面不拓宽；
    // closure 组合身份是 T-02 冻结的独立入口，不进本档）。资源字节必须
    // 真实可读——准入链先验字节身份，再在世界构建期拒绝档域。
    let main = "use resources: crate_tex\nuse props: Crate\nnode Main {\n    view-3d {\n        cylinder3d pad { position: (4.0, 0.5, 0.0) }\n    }\n    camera3d { distance: 8.0\n              pitch: 45.0\n              projection: perspective }\n    objects3d { crate_a }\n}\n";
    let resources = "let crate_tex = textured3d.glb(\"assets/two-parts.glb\")\n";
    let props = "node Crate {\n    object3d { }\n    view-3d { model3d crate { model: resources.crate_tex\n                              position: (-30.0, 0.5, -3.0)\n                              visible: false } }\n}\n";
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3dm-tex-{}-{n}", std::process::id()));
    write_dir(&dir, &[("main", main), ("resources", resources), ("props", props)]);
    fs::create_dir_all(dir.join("assets")).expect("assets dir");
    fs::write(
        dir.join("assets").join("two-parts.glb"),
        &include_bytes!("../fixtures/node3d/g02/assets/two-parts.glb")[..],
    )
    .expect("glb");
    let err = lower_events(&dir).map(|_| ()).expect_err("named rejection expected");
    assert!(
        err.contains("requires the components-3d-presentation profile"),
        "diagnostic '{err}' misses the profile rejection"
    );
}
