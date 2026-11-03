//! PLAN-035 components-3d-audio-hud 降层门（V-S01 声明面；AC-01/04）。
//! 正面：三个永久夹具（actorfree / batch-tick / g3-full-loop）经公开入口
//! `lower_project_components3d_audio_hud` 再生成与提交产品逐字节一致
//! （world.at + scene.agscene；kind=-1 服务 Run / 恰 32 计费边界 kind=0 /
//! 原G3八状态Node），生成面携带 plan035 再生身份注记。负面全部执行前
//! 具名拒绝（module/line/symbol，不 panic）——块归属（缺失/重复/非根）、
//! presentation3d 触发行回流、相机/角色一致性（有角色无相机 / actor-free
//! 带相机）、保留名（根字段/作者fn）、cue 重复、audio 缺失、HUD 槽
//! （越界/重复/版式/owner/text_fn 未声明/签名/text_field/visible）、未知
//! cue、actor-free pose、快照与批销毁元数。运行：cargo test --offline
//! --test node3d_audio_hud_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::lower_components3d_audio_hud::
    lower_project_components3d_audio_hud;
use autographics_node2d_pilot::lower_node3d_presentation::load_resources_meta;
use autographics_node2d_pilot::load_project;

// ------------------------------------------------ 冻结夹具源（T-01 身份）--

const AF_MAIN: &str =
    include_str!("../fixtures/node3d-presentation-services/actorfree/main.at");
const AF_HUDMAP: &str =
    include_str!("../fixtures/node3d-presentation-services/actorfree/hudmap.at");
const BT_MAIN: &str =
    include_str!("../fixtures/node3d-presentation-services/batch-tick/main.at");
const BT_HUDMAP: &str =
    include_str!("../fixtures/node3d-presentation-services/batch-tick/hudmap.at");
const BT_RESOURCES: &str =
    include_str!("../fixtures/node3d-presentation-services/batch-tick/resources.at");
const G3_MAIN: &str = include_str!("../fixtures/g3-full-loop/main.at");
const G3_HUDMAP: &str = include_str!("../fixtures/g3-full-loop/hudmap.at");
const G3_RESOURCES: &str = include_str!("../fixtures/g3-full-loop/resources.at");

// ------------------------------------------------ 冻结受检元数据与产品 --

const AF_META: &str =
    include_str!("../generated-node3d-audio-hud/actorfree/resources.json");
const BT_META: &str =
    include_str!("../generated-node3d-audio-hud/batch-tick/resources.json");
const G3_META: &str =
    include_str!("../generated-node3d-audio-hud/g3-full-loop/resources.json");
const AF_WORLD: &str = include_str!("../generated-node3d-audio-hud/actorfree/world.at");
const AF_SCENE: &str =
    include_str!("../generated-node3d-audio-hud/actorfree/scene.agscene");
const BT_WORLD: &str =
    include_str!("../generated-node3d-audio-hud/batch-tick/world.at");
const BT_SCENE: &str =
    include_str!("../generated-node3d-audio-hud/batch-tick/scene.agscene");
const G3_WORLD: &str =
    include_str!("../generated-node3d-audio-hud/g3-full-loop/world.at");
const G3_SCENE: &str =
    include_str!("../generated-node3d-audio-hud/g3-full-loop/scene.agscene");

// ------------------------------------------------ 冻结素材字节（7 身份并集子集）--

const CHARACTER_GLB: &[u8] =
    include_bytes!("../fixtures/g3-full-loop/assets/character.glb");
const FLAG_GLB: &[u8] = include_bytes!("../fixtures/g3-full-loop/assets/flag.glb");
const COLORMAP_PNG: &[u8] =
    include_bytes!("../fixtures/g3-full-loop/assets/Textures/colormap.png");
const BANK_WAV: &[u8] = include_bytes!(
    "../fixtures/g3-full-loop/assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav"
);
const FONT_TTF: &[u8] = include_bytes!(
    "../fixtures/g3-full-loop/assets/4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa.ttf"
);
const LICENSE_TXT: &str =
    include_str!("../fixtures/g3-full-loop/assets/license.txt");

const BANK_NAME: &str =
    "assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav";
const FONT_NAME: &str =
    "assets/4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa.ttf";

// ------------------------------------------------ 夹具装配 --

fn write_file(dir: &path::Path, rel: &str, bytes: &[u8]) {
    let p = dir.join(rel);
    fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
    fs::write(p, bytes).expect("write");
}

struct Case {
    dir: path::PathBuf,
}

fn setup(files: &[(&str, String)], blobs: &[(&str, &[u8])]) -> Case {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3ah-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (name, body) in files {
        write_file(&dir, &format!("{name}.at"), body.as_bytes());
    }
    for (rel, bytes) in blobs {
        write_file(&dir, rel, bytes);
    }
    Case { dir }
}

/// 精确变异（from 必须恰好出现一次且变异生效）。
fn patch(src: &str, from: &str, to: &str) -> String {
    assert!(src.contains(from), "patch source lacks needle: {from:?}");
    let out = src.replacen(from, to, 1);
    assert!(out != src, "patch did not change the source");
    out
}

fn af_files() -> Vec<(&'static str, String)> {
    vec![("main", AF_MAIN.to_string()), ("hudmap", AF_HUDMAP.to_string())]
}

fn af_blobs() -> Vec<(&'static str, &'static [u8])> {
    vec![
        (BANK_NAME, BANK_WAV),
        (FONT_NAME, FONT_TTF),
        ("assets/license.txt", LICENSE_TXT.as_bytes()),
    ]
}

fn bt_files() -> Vec<(&'static str, String)> {
    vec![
        ("main", BT_MAIN.to_string()),
        ("hudmap", BT_HUDMAP.to_string()),
        ("resources", BT_RESOURCES.to_string()),
    ]
}

fn bt_blobs() -> Vec<(&'static str, &'static [u8])> {
    let mut v = af_blobs();
    v.push(("assets/character.glb", CHARACTER_GLB));
    v.push(("assets/Textures/colormap.png", COLORMAP_PNG));
    v
}

fn g3_files() -> Vec<(&'static str, String)> {
    vec![
        ("main", G3_MAIN.to_string()),
        ("hudmap", G3_HUDMAP.to_string()),
        ("resources", G3_RESOURCES.to_string()),
    ]
}

fn g3_blobs() -> Vec<(&'static str, &'static [u8])> {
    let mut v = bt_blobs();
    v.push(("assets/flag.glb", FLAG_GLB));
    v
}

/// 公开生成面（与 CLI 两遍同一函数链）：声明 → 受检元数据 → 产品。
fn ahud(files: &[(&str, String)], blobs: &[(&str, &[u8])], meta: &str)
    -> Result<(Option<String>, String), String> {
    let case = setup(files, blobs);
    write_file(&case.dir, "resources.json", meta.as_bytes());
    let project = load_project(&case.dir.join("main.at")).map_err(|e| e.to_string())?;
    let meta = load_resources_meta(&case.dir.join("resources.json"), &project)
        .map_err(|e| e.to_string())?;
    lower_project_components3d_audio_hud(&project, &case.dir, &meta)
        .map(|out| (out.base.user_source, out.base.scene_json))
        .map_err(|e| e.to_string())
}

fn reject(files: &[(&str, String)], blobs: &[(&str, &[u8])], meta: &str, needle: &str) {
    let err = ahud(files, blobs, meta).err().expect("named rejection expected");
    assert!(
        err.contains(needle),
        "diagnostic '{err}' does not contain '{needle}'"
    );
    // 定位锚三形：symbol '<name>' / <path>.at / at line N / module:N: 前缀
    // （research parse 诊断走 module:line 形式）。
    let module_line = err.splitn(2, ' ').next().map_or(false, |head| {
        head.rsplit_once(':')
            .map_or(false, |(pre, _)| {
                pre.chars().last().map_or(false, |c| c.is_ascii_digit())
                    && head.chars().filter(|c| *c == ':').count() >= 2
            })
    });
    assert!(
        err.contains("symbol '")
            || err.contains(".at")
            || err.contains("at line ")
            || module_line,
        "diagnostic '{err}' lacks a location anchor"
    );
}

// ------------------------------------------------ 正面：字节冻结 --

#[test]
fn actorfree_services_match_committed_products_byte_for_byte() {
    let (world, scene) = ahud(&af_files(), &af_blobs(), AF_META).expect("actorfree lowers");
    assert_eq!(
        world.as_deref(),
        Some(AF_WORLD),
        "actorfree world.at must be byte-identical to the committed product"
    );
    assert_eq!(
        scene, AF_SCENE,
        "actorfree scene.agscene must be byte-identical to the committed product"
    );
    // kind=-1：无角色 Run 的合法 bundle 形状（byte-freeze 已钉，具名断言存证）。
    assert!(AF_WORLD.contains("ag_present3d(-1"), "actor-free closeout is kind=-1");
    assert!(AF_SCENE.contains("\"three_d\": null"), "actor-free scene binds no 3D actor");
}

#[test]
fn batch_tick_boundary_matches_committed_products_byte_for_byte() {
    let (world, scene) = ahud(&bt_files(), &bt_blobs(), BT_META).expect("batch-tick lowers");
    assert_eq!(
        world.as_deref(),
        Some(BT_WORLD),
        "batch-tick world.at must be byte-identical to the committed product"
    );
    assert_eq!(
        scene, BT_SCENE,
        "batch-tick scene.agscene must be byte-identical to the committed product"
    );
    // 恰 32 计费边界的组成形状：有界快照 + 整批销毁 + kind=0 bundle。
    assert!(BT_WORLD.contains("ag_events_snapshot()"), "bounded event snapshot");
    assert!(BT_WORLD.contains("ag_object_despawn_batch(3"), "whole-batch despawn");
    assert!(BT_SCENE.contains("\"three_d\": 1"), "character scene binds one 3D actor");
}

#[test]
fn g3_full_loop_matches_committed_products_byte_for_byte() {
    let (world, scene) = ahud(&g3_files(), &g3_blobs(), G3_META).expect("g3-full-loop lowers");
    assert_eq!(
        world.as_deref(),
        Some(G3_WORLD),
        "g3-full-loop world.at must be byte-identical to the committed product"
    );
    assert_eq!(
        scene, G3_SCENE,
        "g3-full-loop scene.agscene must be byte-identical to the committed product"
    );
}

#[test]
fn products_carry_plan035_regeneration_identity() {
    // regen_note/doc_prefix 条件化：ahud 产品携带 plan035 再生身份，
    // 旧档产品不含本注记（旧档字节安全由 034 gates/verify_plan034 钉）。
    for (name, w) in [("actorfree", AF_WORLD), ("batch-tick", BT_WORLD), ("g3-full-loop", G3_WORLD)] {
        assert!(
            w.contains("tools/verify_plan035.py"),
            "{name} product lacks the plan035 regeneration identity note"
        );
    }
    assert!(!AF_WORLD.contains("verify_plan034"), "plan034 note must not leak into plan035 products");
}

// ------------------------------------------------ 负面：具名拒绝矩阵 --

#[test]
fn neg_presentation3d_trigger_row_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "        cue tick {",
        "        trigger hit {\n            shape: circle radius: 1.0\n            target: \"Main\"\n            event: Poked\n        }\n        cue tick {",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "presentation3d has no trigger rows");
}

#[test]
fn neg_missing_presentation3d_block_is_named_rejected() {
    const NO_BLOCK: &str = r#"node Main {
    view-3d {
        cylinder3d pad {
            position: (0.0, -0.5, 0.0)
            scale: (4.0, 0.5, 4.0)
        }
    }
}
"#;
    let files = vec![("main", NO_BLOCK.to_string()), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "requires exactly one root presentation3d block");
}

#[test]
fn neg_second_presentation3d_block_is_named_rejected() {
    let decor = "\nnode Decor {\n    presentation3d {\n        audio {\n            bank: \"assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav\"\n            font: \"assets/4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa.ttf\"\n            license: \"assets/license.txt\"\n        }\n        cue other {\n            span: (0, 100)\n            gain: 0.5\n        }\n    }\n    view-3d {\n        cylinder3d deco {\n            position: (5.0, -0.5, 0.0)\n            scale: (1.0, 0.5, 1.0)\n        }\n    }\n}\n";
    // Decor 必须被实例化（否则先触发"恰一根 node"装载拒绝）。
    let main = patch(
        AF_MAIN,
        "    view-3d {\n        cylinder3d pad {",
        "    view-3d {\n        Decor d { }\n        cylinder3d pad {",
    );
    let files = vec![
        ("main", format!("{main}{decor}")),
        ("hudmap", AF_HUDMAP.to_string()),
    ];
    reject(&files, &af_blobs(), AF_META, "duplicate presentation3d block");
}

#[test]
fn neg_nonroot_presentation3d_block_is_named_rejected() {
    const NONROOT: &str = r#"node Decor {
    presentation3d {
        audio {
            bank: "assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav"
            font: "assets/4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa.ttf"
            license: "assets/license.txt"
        }
        cue other {
            span: (0, 100)
            gain: 0.5
        }
    }
    view-3d {
        cylinder3d deco {
            position: (5.0, -0.5, 0.0)
            scale: (1.0, 0.5, 1.0)
        }
    }
}

node Main {
    view-3d {
        Decor d { }
    }
}
"#;
    let files = vec![("main", NONROOT.to_string()), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "presentation3d block is root-only");
}

#[test]
fn neg_character_without_camera_is_named_rejected() {
    let main = patch(
        BT_MAIN,
        "    camera3d { distance: 8.0\n              pitch: 30.0\n              projection: perspective }\n",
        "",
    );
    let files = vec![
        ("main", main),
        ("hudmap", BT_HUDMAP.to_string()),
        ("resources", BT_RESOURCES.to_string()),
    ];
    reject(&files, &bt_blobs(), BT_META, "requires the root camera3d block");
}

#[test]
fn neg_actorfree_with_camera_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "    view-3d {",
        "    camera3d { distance: 8.0\n              pitch: 30.0\n              projection: perspective }\n    view-3d {",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "must not declare camera3d");
}

#[test]
fn neg_reserved_root_field_is_named_rejected() {
    let main = patch(AF_MAIN, "        var n = 0", "        var n = 0\n        var ord = 0");
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(
        &files,
        &af_blobs(),
        AF_META,
        "root field collides with generated presentation3d infrastructure",
    );
}

#[test]
fn neg_reserved_author_fn_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "use hudmap: tick_text",
        "use hudmap: tick_text\n\nfn sfx_play(c int, g float) float {\n    return 0.0\n}",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(
        &files,
        &af_blobs(),
        AF_META,
        "author fn collides with generated presentation3d infrastructure",
    );
}

#[test]
fn neg_duplicate_cue_name_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "        cue tick {\n            span: (0, 7403)\n            gain: 0.5\n        }",
        "        cue tick {\n            span: (0, 7403)\n            gain: 0.5\n        }\n        cue tick {\n            span: (0, 7403)\n            gain: 0.5\n        }",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "duplicate presentation3d cue name");
}

#[test]
fn neg_missing_audio_block_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "        audio {\n            bank: \"assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav\"\n            font: \"assets/4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa.ttf\"\n            license: \"assets/license.txt\"\n        }\n",
        "",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "requires an `audio");
}

#[test]
fn neg_hud_slot_index_out_of_range_is_named_rejected() {
    let main = patch(AF_MAIN, "            slot 0 {", "            slot 4 {");
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "hud slot index must be 0..3");
}

#[test]
fn neg_duplicate_hud_slot_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "                init: \"0\"\n            }",
        "                init: \"0\"\n            }\n            slot 0 {\n                owner: \"Main\"\n                anchor: 0\n                offset: (8.0, 8.0)\n                size: 32.0\n                rgba: (1.0, 1.0, 1.0, 1.0)\n                text_fn: \"tick_text\"\n                text_field: \"n\"\n                init: \"0\"\n            }",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "duplicate hud slot index");
}

#[test]
fn neg_hud_slot_layout_out_of_range_is_named_rejected() {
    let main = patch(AF_MAIN, "                size: 32.0", "                size: 64.0");
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "hud slot layout out of range");
}

#[test]
fn neg_hud_slot_unknown_owner_is_named_rejected() {
    let main = patch(AF_MAIN, "                owner: \"Main\"", "                owner: \"Ghost\"");
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(
        &files,
        &af_blobs(),
        AF_META,
        "hud slot owner must be a root-view component instance",
    );
}

#[test]
fn neg_hud_text_fn_undeclared_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "                text_fn: \"tick_text\"",
        "                text_fn: \"nope_fn\"",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "is not a declared fn");
}

#[test]
fn neg_hud_text_fn_signature_is_named_rejected() {
    let hudmap = patch(AF_HUDMAP, "fn tick_text(n int) str {", "fn tick_text(n float) str {");
    let files = vec![("main", AF_MAIN.to_string()), ("hudmap", hudmap)];
    reject(&files, &af_blobs(), AF_META, "must have signature (int) -> str");
}

#[test]
fn neg_hud_text_field_not_int_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "        var n = 0",
        "        var n = 0\n        var f = 0.0",
    );
    let main = patch(&main, "                text_field: \"n\"", "                text_field: \"f\"");
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "is not an int model field");
}

#[test]
fn neg_hud_visible_not_int_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "        var n = 0",
        "        var n = 0\n        var f = 0.0",
    );
    let main = patch(
        &main,
        "                text_fn: \"tick_text\"",
        "                visible: \"f\"\n                text_fn: \"tick_text\"",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "is not an int model field");
}

#[test]
fn neg_unknown_sfx_cue_is_named_rejected() {
    let main = patch(AF_MAIN, "sfx.play(\"tick\", 0.5)", "sfx.play(\"nope\", 0.5)");
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "unknown sfx cue 'nope'");
}

#[test]
fn neg_actorfree_pose_is_named_rejected() {
    const STATUE: &str = r#"use resources: hero_glb
use hudmap: tick_text

node Statue {
    view-3d {
        model3d body {
            model: resources.hero_glb
            position: (0.0, 0.0, 0.0)
            scale: (1.0, 1.0, 1.0)
        }
    }
    on {
        Life.Tick(dt) -> {
            refs.body.pose(clips.hero_glb.idle, 0.0, 1, 0.0)
        }
    }
}

node Main {
    model {
        var n = 0
    }
    presentation3d {
        audio {
            bank: "assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav"
            font: "assets/4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa.ttf"
            license: "assets/license.txt"
        }
        cue tick {
            span: (0, 7403)
            gain: 0.5
        }
        hud {
            slot 0 {
                owner: "Main"
                anchor: 0
                offset: (8.0, 8.0)
                size: 32.0
                rgba: (1.0, 1.0, 1.0, 1.0)
                text_fn: "tick_text"
                text_field: "n"
                init: "0"
            }
        }
    }
    view-3d {
        Statue s { }
        cylinder3d pad {
            position: (0.0, -0.5, 0.0)
            scale: (4.0, 0.5, 4.0)
        }
    }
    on {
        Life.Tick(dt) -> {
            .n = .n + 1
            sfx.play("tick", 0.5)
        }
    }
}
"#;
    let files = vec![
        ("main", STATUE.to_string()),
        ("hudmap", AF_HUDMAP.to_string()),
        ("resources", BT_RESOURCES.to_string()),
    ];
    reject(
        &files,
        &bt_blobs(),
        BT_META,
        "actor-free components3d bundle carries no visual pose",
    );
}

#[test]
fn neg_events_snapshot_arity_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "            sfx.play(\"tick\", 0.5)",
        "            let n2 = events.snapshot(1)",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "events.snapshot arity");
}

#[test]
fn neg_despawn_batch_nine_handles_is_named_rejected() {
    let main = patch(
        AF_MAIN,
        "            sfx.play(\"tick\", 0.5)",
        "            objects.despawn_batch(0, 0, 0, 0, 0, 0, 0, 0, 0)",
    );
    let files = vec![("main", main), ("hudmap", AF_HUDMAP.to_string())];
    reject(&files, &af_blobs(), AF_META, "objects.despawn_batch arity 9");
}
