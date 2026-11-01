//! PLAN-027 components-3d-runtime 降层门（V-S01；AC-01/03/05 编译面）。
//! 正面：character3d/solid/camera3d/player3d/mob3d 声明链降层 Ok 且两次
//! 再生字节一致（g07a/g08/static-actors/wide128/deep127/two-moves）；
//! 注入配置（bind/solid/orbit）、六动作表、角色/相机服务映射、SoA/绑定
//! 表/getter 形状、内置视觉 kind 与 behavior-null 静态场景钉死。负面：
//! 全部执行前具名拒绝（module/line/symbol，不 panic）——双角色、相机
//! 无角色、相机非 root、动态 solid、角色身兼 solid、角色形状、角色动态
//! 位置、内置视觉 solid、相机值域。运行：cargo test --offline --test
//! node3d_runtime_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::{load_project, lower_project_components3d_runtime, LowerOutputBasic2D};

const G07A_MAIN: &str = include_str!("../fixtures/components3d-runtime/g07a/main.at");
const G07A_CHARACTER: &str = include_str!("../fixtures/components3d-runtime/g07a/character.at");
const G07A_FAST_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/g07a-fast/character.at");
const G08_MAIN: &str = include_str!("../fixtures/components3d-runtime/g08/main.at");
const G08_CHARACTER: &str = include_str!("../fixtures/components3d-runtime/g08/character.at");
const G08_ALT_MAIN: &str = include_str!("../fixtures/components3d-runtime/g08-alt/main.at");
const G08_ALT_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/g08-alt/character.at");
const STATIC_MAIN: &str = include_str!("../fixtures/components3d-runtime/static-actors/main.at");
const EVENTS_MAIN: &str = include_str!("../fixtures/components3d-runtime/events3d/main.at");
const EVENTS_BEEPER: &str = include_str!("../fixtures/components3d-runtime/events3d/beeper.at");
const WIDE128_MAIN: &str = include_str!("../fixtures/components3d-runtime/wide128/main.at");
const DEEP127_MAIN: &str = include_str!("../fixtures/components3d-runtime/deep127/main.at");
const TWO_MOVES_MAIN: &str = include_str!("../fixtures/components3d-runtime/two-moves/main.at");
const TWO_MOVES_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/two-moves/character.at");
const NEG2C_MAIN: &str = include_str!("../fixtures/components3d-runtime/neg-two-characters/main.at");
const NEG2C_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/neg-two-characters/character.at");
const NEG_CAM_NONE: &str =
    include_str!("../fixtures/components3d-runtime/neg-camera-without-character/main.at");
const NEG_CAM_ROOT_MAIN: &str =
    include_str!("../fixtures/components3d-runtime/neg-camera-non-root/main.at");
const NEG_CAM_ROOT_GROUND: &str =
    include_str!("../fixtures/components3d-runtime/neg-camera-non-root/ground.at");
const NEG_SOLID_DYN_MAIN: &str =
    include_str!("../fixtures/components3d-runtime/neg-solid-dynamic/main.at");
const NEG_SOLID_DYN_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/neg-solid-dynamic/character.at");
const NEG_SOC_MAIN: &str =
    include_str!("../fixtures/components3d-runtime/neg-solid-on-character/main.at");
const NEG_SOC_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/neg-solid-on-character/character.at");
const NEG_SHAPE_MAIN: &str =
    include_str!("../fixtures/components3d-runtime/neg-character-shape/main.at");
const NEG_SHAPE_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/neg-character-shape/character.at");
const NEG_DYN_MAIN: &str =
    include_str!("../fixtures/components3d-runtime/neg-character-dyn/main.at");
const NEG_DYN_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/neg-character-dyn/character.at");
const NEG_SOLID_BUILTIN: &str =
    include_str!("../fixtures/components3d-runtime/neg-solid-builtin/main.at");
const NEG_DOMAIN_MAIN: &str =
    include_str!("../fixtures/components3d-runtime/neg-camera-domain/main.at");
const NEG_DOMAIN_CHARACTER: &str =
    include_str!("../fixtures/components3d-runtime/neg-camera-domain/character.at");

fn write_dir(dir: &path::Path, files: &[(&str, &str)]) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower_runtime(dir: &path::Path) -> Result<LowerOutputBasic2D, String> {
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_components3d_runtime(&project).map_err(|e| e.to_string())
}

fn case(files: &[(&str, &str)]) -> Result<LowerOutputBasic2D, String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3dr-{}-{n}", std::process::id()));
    write_dir(&dir, files);
    lower_runtime(&dir)
}

fn reject(files: &[(&str, &str)], needle: &str) {
    let err = case(files).map(|_| ()).expect_err("named rejection expected");
    assert!(
        err.contains(needle),
        "diagnostic '{err}' does not contain '{needle}'"
    );
    // V-S01 合同：每条诊断带 module/line/symbol 定位（不 panic）。
    assert!(
        err.contains("symbol '") || err.contains(".at"),
        "diagnostic '{err}' lacks a location anchor"
    );
}

// ------------------------------------------------ 正面：形状/字节稳定性 --

#[test]
fn g07a_chain_lowers_ok_with_injected_config_and_is_byte_stable() {
    let files = [("main", G07A_MAIN), ("character", G07A_CHARACTER)];
    let a = case(&files).expect("g07a lowers");
    let b = case(&files).expect("g07a lowers again");
    let world = a.user_source.as_ref().expect("behavior world");
    // 注入配置：bind=文档 id 3（root 元素先于实例），solids=1/2（文档序）。
    assert!(world.contains(" ok=ok+ag_char3_bind(3)\n"), "{world}");
    assert!(world.contains(" ok=ok+ag_char3_solid(1)\n"), "{world}");
    assert!(world.contains(" ok=ok+ag_char3_solid(2)\n"), "{world}");
    assert!(!world.contains("ag_camera_orbit_3d"), "G07 has no camera");
    // 六动作表（Forward=2/Back=3/Jump=4）与角色服务（clearance 0.002）。
    assert!(world.contains("ag_action_pressed(4)"), "{world}");
    assert!(world.contains("let hf = ag_action_held(2)"), "{world}");
    assert!(world.contains("let hb = ag_action_held(3)"), "{world}");
    assert!(
        world.contains("ag_char3_move(dx, (character_vy[i] * dt), dz, 0.002)"),
        "{world}"
    );
    assert!(world.contains("ag_char3_grounded(q)"), "{world}");
    // 冻结 tick 序（root 先、声明先序）与 SoA 行。
    assert!(world.contains("fn app_update(dt float) {\n main_lifetick(dt)\n character_lifetick(0,dt)\n}"));
    assert!(world.contains("var character_vy=[0.0]"));
    assert!(world.contains("fn get_Main_hero_grounded() int {return character_grounded[0]}"));
    // 场景：kind cylinder、文档 id 序（root 元素 1/2，实例叶 3）。
    let scene = &a.scene_json;
    assert!(scene.contains("\"document_id\": \"plan027-components-3d-runtime-main\""), "{scene}");
    assert!(scene.contains("\"schema\": \"autographics.scene/6\""));
    assert!(scene.contains("\"name\": \"Main/hero/body\""));
    assert_eq!(scene.matches("\"kind\": \"cylinder\"").count(), 3, "{scene}");
    assert!(!scene.contains("\"kind\": \"player\""), "{scene}");
    // 两次再生字节一致（identity 世界 sha 一致）。
    assert_eq!(a.user_source, b.user_source);
    assert_eq!(a.scene_json, b.scene_json);
    assert_eq!(a.source_sha256, b.source_sha256);
}

#[test]
fn speed_twin_changes_only_the_constant_bytes() {
    let slow = case(&[("main", G07A_MAIN), ("character", G07A_CHARACTER)]).expect("slow");
    let fast = case(&[("main", G07A_MAIN), ("character", G07A_FAST_CHARACTER)]).expect("fast");
    let slow_w = slow.user_source.as_ref().unwrap();
    let fast_w = fast.user_source.as_ref().unwrap();
    assert!(slow_w.contains("var character_speed=[4.0]"));
    assert!(fast_w.contains("var character_speed=[5.0]"));
    // 唯一差异 = speed 常量（V-S02 speed4_to_5 的编译面对应物）。
    let normalize = |s: &str| s.replace("var character_speed=[5.0]", "var character_speed=[4.0]");
    assert_eq!(normalize(fast_w).as_str(), *slow_w, "only the speed constant may differ");
    assert_ne!(slow.source_sha256, fast.source_sha256);
}

#[test]
fn g08_chain_orbit_config_and_camera_reads_match_the_frozen_surface() {
    let files = [("main", G08_MAIN), ("character", G08_CHARACTER)];
    let a = case(&files).expect("g08 lowers");
    let b = case(&files).expect("g08 lowers again");
    let world = a.user_source.as_ref().expect("behavior world");
    assert!(world.contains(" ok=ok+ag_char3_bind(5)\n"), "{world}");
    assert!(world.contains(" ok=ok+ag_char3_solid(1)\n"), "{world}");
    assert!(world.contains(" ok=ok+ag_char3_solid(2)\n"), "{world}");
    assert!(
        world.contains(" ok=ok+ag_camera_orbit_3d(5, 8.0, 45.0, 0, 0.0)\n"),
        "{world}"
    );
    // 相对运动保持原源序：mx/mz 先乘 n，基合成，dx/dz 再乘 n。
    assert!(world.contains("let fx = ag_cam3_forward_x()"), "{world}");
    assert!(world.contains("let fz = ag_cam3_forward_z()"), "{world}");
    assert!(world.contains("let rx = ag_cam3_right_x()"), "{world}");
    assert!(world.contains("let rz = ag_cam3_right_z()"), "{world}");
    assert!(world.contains("let wx = ((fx * mz) + (rx * mx))"), "{world}");
    assert!(world.contains("let dx = (((wx * n) * character_speed[i]) * dt)"), "{world}");
    assert_eq!(a.user_source, b.user_source);
    assert_eq!(a.scene_json, b.scene_json);
}

#[test]
fn g08_alt_uses_ortho_projection_and_alt_constants() {
    let out = case(&[("main", G08_ALT_MAIN), ("character", G08_ALT_CHARACTER)])
        .expect("g08-alt lowers");
    let world = out.user_source.as_ref().unwrap();
    assert!(
        world.contains(" ok=ok+ag_camera_orbit_3d(5, 6.0, 30.0, 1, 16.0)\n"),
        "{world}"
    );
    assert!(world.contains("var character_speed=[5.0]"));
    assert!(world.contains("var character_jump=[7.0]"));
    assert!(world.contains("var character_gravity=[18.0]"));
}

#[test]
fn static_actors_is_behavior_null_with_builtin_kinds() {
    let out = case(&[("main", STATIC_MAIN)]).expect("static-actors lowers");
    // 纯静态场景：无用户 VM（behavior null）、无角色/相机服务。
    assert!(out.user_source.is_none(), "static scene must not carry a user VM");
    let scene = &out.scene_json;
    assert!(scene.contains("\"behavior\": null"), "{scene}");
    assert!(scene.contains("\"kind\": \"player\""), "{scene}");
    assert!(scene.contains("\"kind\": \"mob\""), "{scene}");
    assert!(scene.contains("\"resource\": null"), "{scene}");
    // 内置视觉不携带资源字节（driver 原 owner 暖上传）。
    assert!(scene.contains("\"assets\": [\n\n ]"), "{scene}");
    let again = case(&[("main", STATIC_MAIN)]).expect("static-actors again");
    assert_eq!(out.scene_json, again.scene_json);
}

#[test]
fn events3d_keeps_the_020_receiver_isolation_face_on_the_runtime_profile() {
    let files = [("main", EVENTS_MAIN), ("beeper", EVENTS_BEEPER)];
    let out = case(&files).expect("events3d lowers");
    let world = out.user_source.as_ref().expect("events world");
    // 两个同定义实例：每实例局部 phase 独立（SoA），root 接收器按实例分账。
    assert!(world.contains("var beeper_phase=[0.0,3.0]"), "{world}");
    assert!(world.contains("var beeper_dot_rows=["), "{world}");
    assert!(world.contains("fn main_pinged(v float)"), "{world}");
    let roles = &out.roles_json;
    assert!(roles.contains("b0"), "{roles}");
    assert!(roles.contains("b1"), "{roles}");
    let again = case(&files).expect("events3d again");
    assert_eq!(out.user_source, again.user_source);
    assert_eq!(out.scene_json, again.scene_json);
}

#[test]
fn wide128_boundary_and_deep127_counterexample_lower_ok() {
    // 128 行文档边界：wide128 = 角色 + 地面/墙两个注册 solid + 125 根非
    // solid 稀疏柱（扫描包络过 128 identities）；deep127 = 127 根注册
    // solid 重叠瓦片（文档上限内的越限反例，拒绝发生在真实 Run 的累计
    // 步门，属 V-S02 流面，不在编译面）。
    {
        let out = case(&[("main", WIDE128_MAIN), ("character", G07A_CHARACTER)])
            .expect("wide128 must lower");
        let world = out.user_source.as_ref().unwrap();
        // root 自身元素先于实例：ground=1、wall=2 注册 solid，柱 3..127，
        // 角色实例 = 文档 id 128。
        assert!(world.contains(" ok=ok+ag_char3_bind(128)\n"), "{world}");
        assert!(world.contains(" ok=ok+ag_char3_solid(1)\n"), "{world}");
        assert!(world.contains(" ok=ok+ag_char3_solid(2)\n"), "{world}");
        assert_eq!(world.matches("ag_char3_solid(").count(), 2, "{world}");
        let scene = &out.scene_json;
        assert_eq!(scene.matches("\"kind\": \"cylinder\"").count(), 128, "wide128 rows");
    }
    {
        let out = case(&[("main", DEEP127_MAIN), ("character", G07A_CHARACTER)])
            .expect("deep127 must lower");
        let world = out.user_source.as_ref().unwrap();
        assert!(world.contains(" ok=ok+ag_char3_bind(128)\n"), "{world}");
        assert_eq!(world.matches("ag_char3_solid(").count(), 127, "deep127 solids");
    }
}

#[test]
fn two_moves_lowers_ok_and_runtime_refusal_is_flow_side() {
    let out = case(&[("main", TWO_MOVES_MAIN), ("character", TWO_MOVES_CHARACTER)])
        .expect("two-moves lowers");
    let world = out.user_source.as_ref().unwrap();
    assert_eq!(world.matches("ag_char3_move(").count(), 2, "{world}");
}

// ------------------------------------------------ 负面：执行前具名拒绝 --

#[test]
fn negatives_are_named_rejections() {
    reject(
        &[("main", NEG2C_MAIN), ("character", NEG2C_CHARACTER)],
        "character3d must bind exactly one instance per Run",
    );
    reject(
        &[("main", NEG_CAM_NONE)],
        "camera3d requires exactly one character3d instance",
    );
    reject(
        &[("main", NEG_CAM_ROOT_MAIN), ("ground", NEG_CAM_ROOT_GROUND)],
        "camera3d block is root-only",
    );
    reject(
        &[("main", NEG_SOLID_DYN_MAIN), ("character", NEG_SOLID_DYN_CHARACTER)],
        "model ref outside an instance context",
    );
    reject(
        &[("main", NEG_SOC_MAIN), ("character", NEG_SOC_CHARACTER)],
        "character body 'body' must not also be marked solid",
    );
    reject(
        &[("main", NEG_SHAPE_MAIN), ("character", NEG_SHAPE_CHARACTER)],
        "character body 'body' scale must be 0.4 x 0.5 x 0.4",
    );
    reject(
        &[("main", NEG_DYN_MAIN), ("character", NEG_DYN_CHARACTER)],
        "character body 'body' position must be static",
    );
    reject(
        &[("main", NEG_SOLID_BUILTIN)],
        "'solid' is a cylinder3d prop, not player3d",
    );
    reject(
        &[("main", NEG_DOMAIN_MAIN), ("character", NEG_DOMAIN_CHARACTER)],
        "camera3d distance must be within [4, 12]",
    );
}
