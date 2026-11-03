//! PLAN-034 components-3d-presentation 降层门（V-S01 声明面；AC-01/04/05）。
//! 正面：g10-textures 三步（声明→受检元数据→产品）与提交产品字节一致
//! （closure 场景引用、外部图像闭包行）；g10-animation 具名 clip 降层形状
//! （idle/walk/jump 序数、base/显式序数、pose 1基文档 id、快照前置）；双
//! 同定义动画实例独立行表/相位。负面全部执行前具名拒绝（带定位锚）：
//! 未知 clip 名、跨模型 clip、越界显式序数、纹理叶 pose、旧档 animated3d
//! 构造器、陈旧/多余元数据、重名 clip、平色构造器携纹理内容、未知资源
//! clip 值。运行：cargo test --offline --test node3d_presentation_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::lower_node3d_presentation::{
    load_resources_meta, lower_project_components3d_presentation, resources_request_json,
};
use autographics_node2d_pilot::{load_project, LowerOutputBasic2D};

const TEX_MAIN: &str = include_str!("../fixtures/node3d-presentation/g10-textures/main.at");
const TEX_RES: &str = include_str!("../fixtures/node3d-presentation/g10-textures/resources.at");
const TEX_MAIN_COMMITTED: &str =
    include_str!("../generated-components3d-presentation/g10-textures/world.at");
const TEX_SCENE_COMMITTED: &str =
    include_str!("../generated-components3d-presentation/g10-textures/scene.agscene");
const TEX_ALT_MAIN: &str =
    include_str!("../fixtures/node3d-presentation/g10-textures-alt/main.at");
const TEX_ALT_SCENE_COMMITTED: &str =
    include_str!("../generated-components3d-presentation/g10-textures-alt/scene.agscene");
const ANIM_MAIN: &str = include_str!("../fixtures/node3d-presentation/g10-animation/main.at");
const ANIM_RES: &str = include_str!("../fixtures/node3d-presentation/g10-animation/resources.at");
const ANIM_WORLD_COMMITTED: &str =
    include_str!("../generated-components3d-presentation/g10-animation/world.at");

const PLATFORM_GLB: &[u8] =
    include_bytes!("../fixtures/node3d-presentation/g10-textures/assets/platform.glb");
const COIN_GLB: &[u8] =
    include_bytes!("../fixtures/node3d-presentation/g10-textures/assets/coin.glb");
const FLAG_GLB: &[u8] =
    include_bytes!("../fixtures/node3d-presentation/g10-textures/assets/flag.glb");
const PROBE_GLB: &[u8] =
    include_bytes!("../fixtures/node3d-presentation/g10-textures/uv/uv-material-probe.glb");
const HERO_GLB: &[u8] =
    include_bytes!("../fixtures/node3d-presentation/g10-animation/assets/character.glb");

/// 受检身份（与提交产品/公共导入逐字节一致；T-01/T-02 证据哈希）。
const COLORMAP_SHA: &str = "106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3";
const PLATFORM_CLOSURE: &str = "47fc0befa3a4b29fe90533fac4f13e90700c2c592ae639c73abe40a859c206b8";
const COIN_CLOSURE: &str = "76657e1367fe78e3aa69585e4a2d6e462f7ca431a4c17d1b59a1e7fca7d3862e";
const FLAG_CLOSURE: &str = "239a4ea37124b16664125ff424ce3574805449a56c2022aa3689a65c3a2a0ccd";
const PROBE_CLOSURE: &str = "663f848231bb87f7ebe4e05b90f2d8c5651f64ad01a990ebf0dc4c21d0fb7172";
const HERO_CLOSURE: &str = "6da9227ee326d34b997e7d859abb0c3f9fc9af3f53659feaa5a48982472fbd9d";

fn write_file(dir: &path::Path, rel: &str, bytes: &[u8]) {
    let p = dir.join(rel);
    fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
    fs::write(p, bytes).expect("write");
}

struct Case {
    dir: path::PathBuf,
}

fn setup(files: &[(&str, &str)], blobs: &[(&str, &[u8])]) -> Case {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3dp-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (name, body) in files {
        write_file(&dir, &format!("{name}.at"), body.as_bytes());
    }
    for (rel, bytes) in blobs {
        write_file(&dir, rel, bytes);
    }
    Case { dir }
}

/// 三步的库级面：声明请求 → 受检元数据 → 产品（与 CLI 两遍同一函数）。
fn present(case: &Case) -> Result<LowerOutputBasic2D, String> {
    let project = load_project(&case.dir.join("main.at")).map_err(|e| e.to_string())?;
    let meta =
        load_resources_meta(&case.dir.join("resources.json"), &project).map_err(|e| e.to_string())?;
    lower_project_components3d_presentation(&project, &meta)
        .map(|(out, _api)| out)
        .map_err(|e| e.to_string())
}

fn present_bindings(case: &Case) -> Result<LowerOutputBasic2D, String> {
    let project = load_project(&case.dir.join("main.at")).map_err(|e| e.to_string())?;
    autographics_node2d_pilot::lower_project_components3d_bindings(&project)
        .map_err(|e| e.to_string())
}

fn reject(files: &[(&str, &str)], blobs: &[(&str, &[u8])], meta: &str, needle: &str) {
    let case = setup(files, blobs);
    write_file(&case.dir, "resources.json", meta.as_bytes());
    let err = present(&case).err().expect("named rejection expected");
    assert!(
        err.contains(needle),
        "diagnostic '{err}' does not contain '{needle}'"
    );
    assert!(
        err.contains("symbol '") || err.contains(".at") || err.contains("at line "),
        "diagnostic '{err}' lacks a location anchor"
    );
}

// ------------------------------------------------ 受检元数据夹具 --

fn tex_meta() -> String {
    let colormap = format!(
        r#"{{"source": "uri:Textures/colormap.png", "sha256": "{COLORMAP_SHA}", "byte_length": 10108, "width": 512, "height": 512, "path": "assets/Textures/colormap.png"}}"#
    );
    let checker = format!(
        r#"{{"source": "uri:asymmetric-checker.png", "sha256": "6cc329163c9dadf2e531d5095196cc91631823c5b9ddce69f6e2b6690173dc2f", "byte_length": 332, "width": 8, "height": 8, "path": "uv/asymmetric-checker.png"}}"#
    );
    let entry = |name: &str,
                 kind: &str,
                 file: &str,
                 sha: &str,
                 len: u64,
                 closure: &str,
                 image: &str| {
        format!(
            r#"{{"name": "{name}", "kind": "{kind}", "file": "{file}", "animated": false, "identity": {{"closure": {{"id": "{closure}", "images": [{image}]}}, "format": "glb", "sha256": "{sha}", "byte_length": {len}, "name": "{file}"}}, "clips": [], "images": [{image}], "bounds_min": [0.0, 0.0, 0.0], "bounds_max": [1.0, 1.0, 1.0]}}"#
        )
    };
    format!(
        r#"{{"schema": "autographics.resources-meta/1", "entries": [{},{},{},{}]}}"#,
        entry("platform", "textured3d", "assets/platform.glb", "78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c", 13352, PLATFORM_CLOSURE, &colormap),
        entry("coin", "textured3d", "assets/coin.glb", "a3fb8f779a6af1cd75e5a02da77a87546ca8a59f6fe52f79f25179c4a68fd0e5", 22124, COIN_CLOSURE, &colormap),
        entry("flag", "textured3d", "assets/flag.glb", "0ba84ff43f0fea0a9f0b080cfa1be54ebda36c8c35174d1ac2553ae6da9f65ae", 13276, FLAG_CLOSURE, &colormap),
        entry("uv_probe", "textured3d", "uv/uv-material-probe.glb", "f9a8f15627ca0d2060192a429e16dc7145c64a2f2e9315681d42a3f99d99bee3", 3692, PROBE_CLOSURE, &checker),
    )
}

fn anim_meta(clips: &str) -> String {
    let image = format!(
        r#"{{"source": "uri:Textures/colormap.png", "sha256": "{COLORMAP_SHA}", "byte_length": 10108, "width": 512, "height": 512, "path": "assets/Textures/colormap.png"}}"#
    );
    format!(
        r#"{{"schema": "autographics.resources-meta/1", "entries": [{{"name": "hero_glb", "kind": "animated3d", "file": "assets/character.glb", "animated": true, "identity": {{"closure": {{"node_animation": true, "id": "{HERO_CLOSURE}", "images": [{image}]}}, "format": "glb", "sha256": "7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1", "byte_length": 87624, "name": "character.glb"}}, "clips": {clips}, "images": [{image}], "bounds_min": [-0.6, 0.0, -0.32375001], "bounds_max": [0.6, 1.1, 0.31125000]}}]}}"#
    )
}

const HERO_CLIPS: &str = r#"[{"ordinal": 0, "name": "static"}, {"ordinal": 1, "name": "idle"}, {"ordinal": 2, "name": "walk"}, {"ordinal": 3, "name": "jump"}]"#;

fn tex_files() -> Vec<(&'static str, &'static str)> {
    vec![("main", TEX_MAIN), ("resources", TEX_RES)]
}

fn anim_files() -> Vec<(&'static str, &'static str)> {
    vec![("main", ANIM_MAIN), ("resources", ANIM_RES)]
}

fn tex_blobs() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("assets/platform.glb", PLATFORM_GLB),
        ("assets/coin.glb", COIN_GLB),
        ("assets/flag.glb", FLAG_GLB),
        ("uv/uv-material-probe.glb", PROBE_GLB),
        ("assets/Textures/colormap.png", b"png"),
    ]
}

fn anim_blobs() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("assets/character.glb", HERO_GLB),
        ("assets/Textures/colormap.png", b"png"),
    ]
}

// ------------------------------------------------ 正面：声明面形状 --

#[test]
fn request_covers_all_model_kinds_with_constructors() {
    let case = setup(&tex_files(), &tex_blobs());
    let project = load_project(&case.dir.join("main.at")).expect("load");
    let request = resources_request_json(&project);
    assert_eq!(
        request.matches("\"kind\": \"textured3d\"").count(),
        4,
        "four textured3d declarations"
    );
}

#[test]
fn textures_three_pass_matches_committed_products_byte_for_byte() {
    let case = setup(&tex_files(), &tex_blobs());
    write_file(&case.dir, "resources.json", tex_meta().as_bytes());
    let lowered = present(&case).expect("textures lowers");
    assert_eq!(
        lowered.scene_json, TEX_SCENE_COMMITTED,
        "textures scene must be byte-identical to the committed product"
    );
    let alt_case = setup(
        &[("main", TEX_ALT_MAIN), ("resources", TEX_RES)],
        &tex_blobs(),
    );
    write_file(&alt_case.dir, "resources.json", tex_meta().as_bytes());
    let alt = present(&alt_case).expect("alt lowers");
    assert_eq!(alt.scene_json, TEX_ALT_SCENE_COMMITTED, "alt scene byte-identical");
    let world = lowered.user_source.as_ref().expect("behavior world");
    assert_eq!(world, TEX_MAIN_COMMITTED, "textures world byte-identical");
}

#[test]
fn animation_named_clips_pose_doc_ids_and_snapshot_shape() {
    let case = setup(&anim_files(), &anim_blobs());
    write_file(&case.dir, "resources.json", anim_meta(HERO_CLIPS).as_bytes());
    let lowered = present(&case).expect("animation lowers");
    let world = lowered.user_source.as_ref().expect("behavior world");
    assert_eq!(world, ANIM_WORLD_COMMITTED, "animation world byte-identical");
    // 具名 clip 降层（文件原序：idle 1/walk 2/jump 3；Start = 显式序数 0）。
    assert!(world.contains("var clip = 1"), "{world}");
    assert!(world.contains("clip = 2\n"), "{world}");
    assert!(world.contains("clip = 3\n"), "{world}");
    assert!(world.contains("ag_visual_pose(0, (player_body_rows[i]) + 1, 0, 0.0, 1, 0.0)"), "{world}");
    // 快照前置（1 计费输入面）与 pose 1 基文档 id。
    assert!(world.contains(" let packet=ag_actions_snapshot()\n let held_bits=packet%64\n let pressed_bits=packet/64\n"), "{world}");
    assert!(world.contains("ag_visual_pose(0, (player_body_rows[i]) + 1, clip, player_phase[i], 1, player_yaw[i])"), "{world}");
    // 角色体模型叶 + 原地板 solid 的装配形状。
    assert!(world.contains("ok=ok+ag_char3_bind(1)"), "{world}");
    assert!(world.contains("ok=ok+ag_char3_solid(2)"), "{world}");
    assert!(world.contains("ag_trigger_bind(0, player_body_subject_ids[i], 0, ground_floor_subject_ids[0], 1, 40.0)"), "{world}");
}

#[test]
fn two_same_def_animated_instances_keep_independent_rows() {
    let main = r#"use resources: hero_glb

node Hero {
    model {
        var phase = 0.0
        var yaw = 0.0
    }
    view-3d {
        model3d body {
            model: resources.hero_glb
            position: (0.0, 0.0020000027093020076, 0.0)
            scale: (0.6666666534211902, 0.9090908967759002, 1.2598424794780512)
        }
    }
    on {
        Life.Tick(dt) -> {
            .phase = .phase + dt
            refs.body.pose(clips.hero_glb.idle, .phase, 1, .yaw)
        }
    }
}

node Main {
    view-3d {
        Hero a { }
        Hero b { }
    }
}
"#;
    let case = setup(
        &[("main", main), ("resources", ANIM_RES)],
        &anim_blobs(),
    );
    write_file(&case.dir, "resources.json", anim_meta(HERO_CLIPS).as_bytes());
    let lowered = present(&case).expect("twin lowers");
    let world = lowered.user_source.as_ref().expect("world");
    // 双实例独立行表与独立 pose 行（作者零数字 ID，接收方不串）。
    assert!(world.contains("var hero_phase=[0.0,0.0]") || world.contains("var hero_phase=[0.0, 0.0]"), "{world}");
    assert!(world.contains("var hero_yaw=[0.0,0.0]") || world.contains("var hero_yaw=[0.0, 0.0]"), "{world}");
    assert!(world.contains("hero_body_rows=["), "{world}");
    assert!(world.contains("hero_lifetick(0,dt)"), "{world}");
    assert!(world.contains("hero_lifetick(1,dt)"), "{world}");
    // 每 def 一个 lifetick fn（按实例下标 i 索引独立行）；pose 文本一次、
    // 双实例双调用 = 独立相位/朝向行。
    assert_eq!(
        world.matches("ag_visual_pose(0, (hero_body_rows[i]) + 1").count(),
        1,
        "one indexed pose emission"
    );
    assert!(world.matches("hero_lifetick(").count() >= 2, "two instance calls");
}

#[test]
fn twin_animated_character3d_instances_named_rejected() {
    let main = r#"use resources: hero_glb

node Hero {
    model { var phase = 0.0 }
    view-3d {
        model3d body {
            model: resources.hero_glb
            position: (0.0, 0.0020000027093020076, 0.0)
            scale: (0.6666666534211902, 0.9090908967759002, 1.2598424794780512)
        }
    }
    character3d { }
    on {
        Life.Tick(dt) -> {
            .phase = .phase + dt
        }
    }
}

node Main {
    view-3d {
        Hero a { }
        Hero b { }
    }
}
"#;
    let case = setup(&[("main", main), ("resources", ANIM_RES)], &anim_blobs());
    write_file(&case.dir, "resources.json", anim_meta(HERO_CLIPS).as_bytes());
    let err = present(&case).err().expect("twin character3d must reject");
    assert!(
        err.contains("character3d must bind exactly one instance per Run"),
        "{err}"
    );
}

// ------------------------------------------------ 负面：具名拒绝 --

#[test]
fn unknown_clip_name_named_rejected() {
    let main = ANIM_MAIN.replace("clips.hero_glb.idle", "clips.hero_glb.nothere");
    reject(
        &[("main", main.as_str()), ("resources", ANIM_RES)],
        &anim_blobs(),
        &anim_meta(HERO_CLIPS),
        "resource 'hero_glb' has no clip named 'nothere'",
    );
}

#[test]
fn cross_model_clip_named_rejected() {
    // 第二个动画资源在案；pose 用它的 clip（跨模型错配）。
    let resources = r#"let hero_glb = animated3d.glb("assets/character.glb")
let other_glb = animated3d.glb("assets/character.glb")
"#;
    let main = ANIM_MAIN
        .replace(
            "use resources: hero_glb",
            "use resources: hero_glb, other_glb",
        )
        .replace("refs.body.pose(clips.hero_glb.0, 0.0, 1, 0.0)", "refs.body.pose(clips.other_glb.0, 0.0, 1, 0.0)");
    let entry = |name: &str| {
        format!(
            r#"{{"name": "{name}", "kind": "animated3d", "file": "assets/character.glb", "animated": true, "identity": {{"closure": {{"node_animation": true, "id": "{HERO_CLOSURE}", "images": []}}, "format": "glb", "sha256": "7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1", "byte_length": 87624, "name": "character.glb"}}, "clips": {HERO_CLIPS}, "images": [], "bounds_min": [0.0, 0.0, 0.0], "bounds_max": [1.0, 1.0, 1.0]}}"#
        )
    };
    let meta = format!(
        r#"{{"schema": "autographics.resources-meta/1", "entries": [{}, {}]}}"#,
        entry("hero_glb"),
        entry("other_glb")
    );
    reject(
        &[("main", main.as_str()), ("resources", resources)],
        &anim_blobs(),
        &meta,
        "does not belong to element",
    );
}

#[test]
fn clip_ordinal_out_of_range_named_rejected() {
    let main = ANIM_MAIN.replace("clips.hero_glb.idle", "clips.hero_glb.9");
    reject(
        &[("main", main.as_str()), ("resources", ANIM_RES)],
        &anim_blobs(),
        &anim_meta(HERO_CLIPS),
        "no clip ordinal 9 (clips: 4)",
    );
}

#[test]
fn pose_on_texture_leaf_named_rejected() {
    // floor 是圆柱叶（无模型绑定）—— pose 具名拒绝。
    let main2 = ANIM_MAIN.replace(
        "refs.body.pose(clip, .phase, 1, .yaw)",
        "refs.floor.pose(clip, .phase, 1, .yaw)",
    );
    reject(
        &[("main", main2.as_str()), ("resources", ANIM_RES)],
        &anim_blobs(),
        &anim_meta(HERO_CLIPS),
        "is not a model leaf bound to an animated3d.glb resource",
    );
}

#[test]
fn animated_ctor_outside_presentation_profile_named_rejected() {
    // bindings 档不得出现新构造器（不静默升档）。
    let resources = r#"let hero_glb = animated3d.glb("assets/character.glb")
"#;
    let main = r#"use resources: hero_glb

node Main {
    view-3d {
        model3d body {
            model: resources.hero_glb
            position: (0.0, 0.0, 0.0)
        }
    }
}
"#;
    let case = setup(&[("main", main), ("resources", resources)], &anim_blobs());
    let err = present_bindings(&case).err().expect("bindings-profile ctor must reject");
    assert!(
        err.contains("requires the components-3d-presentation profile"),
        "{err}"
    );
}

#[test]
fn stale_meta_identity_named_rejected() {
    let meta = anim_meta(HERO_CLIPS).replace("87624", "87625");
    reject(
        &anim_files(),
        &anim_blobs(),
        &meta,
        "does not match declaration",
    );
}

#[test]
fn unknown_meta_entry_named_rejected() {
    let meta = anim_meta(HERO_CLIPS).replace(
        "\"name\": \"hero_glb\"",
        "\"name\": \"not_declared\"",
    );
    reject(
        &anim_files(),
        &anim_blobs(),
        &meta,
        "has no matching model3d/textured3d/animated3d declaration",
    );
}

#[test]
fn duplicate_clip_names_named_rejected() {
    let dup = r#"[{"ordinal": 0, "name": "same"}, {"ordinal": 1, "name": "same"}, {"ordinal": 2, "name": "walk"}, {"ordinal": 3, "name": "jump"}]"#;
    reject(
        &anim_files(),
        &anim_blobs(),
        &anim_meta(dup),
        "duplicate clip name 'same'",
    );
}

#[test]
fn plain_ctor_with_textured_content_named_rejected() {
    // 平色构造器声明 platform（真实内容携图像边）→ 旧构造器不静默接受。
    let resources = r#"let platform = model3d.glb("assets/platform.glb")
"#;
    let main = r#"use resources: platform

node Main {
    view-3d {
        model3d ground {
            model: resources.platform
            position: (-6.0, 0.0, 0.0)
            scale: (2.0, 2.0, 2.0)
        }
    }
}
"#;
    let meta = format!(
        r#"{{"schema": "autographics.resources-meta/1", "entries": [{{"name": "platform", "kind": "model3d", "file": "assets/platform.glb", "animated": false, "identity": {{"closure": {{"id": "{PLATFORM_CLOSURE}", "images": [{{"source": "uri:Textures/colormap.png", "sha256": "{COLORMAP_SHA}", "byte_length": 10108, "width": 512, "height": 512}}]}}, "format": "glb", "sha256": "78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c", "byte_length": 13352, "name": "platform.glb"}}, "clips": [], "images": [], "bounds_min": [0.0, 0.0, 0.0], "bounds_max": [1.0, 1.0, 1.0]}}]}}"#
    );
    reject(
        &[("main", main), ("resources", resources)],
        &tex_blobs(),
        &meta,
        "plain model3d.glb declaration carries textured/animated content",
    );
}

#[test]
fn clip_value_unknown_resource_named_rejected() {
    let main = ANIM_MAIN.replace("clips.hero_glb.idle", "clips.other_glb.idle");
    reject(
        &[("main", main.as_str()), ("resources", ANIM_RES)],
        &anim_blobs(),
        &anim_meta(HERO_CLIPS),
        "references unknown resource 'other_glb'",
    );
}
