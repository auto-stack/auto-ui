//! PLAN-027 components-3d-runtime flow（V-S02/V-S03/V-S04/V-S05；AC-02/03/04/05）。
//! 原 G07 两布局 9+2、G08/3d 13 项 oracle 逐名经真实生成链（pilot
//! components-3d-runtime 产品 -> Storage::load_project -> Host -> GPU）
//! 复刻，另加 V-S03 typed/预算拒绝（two-moves 33 计费 fault 10、stale-token
//! fault 13、坏 camera target fault 30、重复配置 fault 29、deep127 累计步门）、
//! V-S04 输入/相机事件（drag/wheel/Pause/Step/Stop/blur/fresh）、V-S05 内置
//! 视觉保真（原 owner 暖上传对照）。GPU 单实例：--test-threads=1。
//! 运行：cargo test --locked --offline --test node3d_runtime_flow -- --test-threads=1
use agp::host::surface_runtime::SurfaceContent;
use agp::render3d::{Camera3d, Projection};
use autographics_auto_driver::authoring::document::Document;
use autographics_auto_driver::authoring::{
    assets::Registry,
    behavior::SourceSnapshot,
    host::Host,
    play::{Action, CharProfile},
    storage::Storage,
};
use glam::{Mat4, Vec3};
use std::path::{Path, PathBuf};
use std::time::Instant;

const G07A_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/g07a/scene.agscene");
const G07A_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/g07a/world.at");
const G07A_FAST_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/g07a-fast/scene.agscene"
);
const G07A_FAST_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/g07a-fast/world.at"
);
const G07B_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/g07b/scene.agscene");
const G07B_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/g07b/world.at");
const G08_SCENE: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/g08/scene.agscene");
const G08_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/g08/world.at");
const G08_ALT_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/g08-alt/scene.agscene"
);
const G08_ALT_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/g08-alt/world.at");
const STATIC_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/static-actors/scene.agscene"
);
const TWO_MOVES_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/two-moves/scene.agscene"
);
const TWO_MOVES_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/two-moves/world.at");
const STALE_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/stale-token/scene.agscene"
);
const STALE_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/stale-token/world.at");
const DEEP127_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/deep127/scene.agscene"
);
const DEEP127_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/deep127/world.at");

const WIDE128_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/wide128/scene.agscene"
);
const WIDE128_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/wide128/world.at");

const STEPS: usize = 60;

// ---------------------------------------------------------------- harness --

fn write_project(scene_text: &str, world_src: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan027-runtime")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    if let Some(sha) = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
    {
        std::fs::write(scripts.join(format!("{sha}.at")), world_src).expect("write script");
    }
    dir
}

fn start(scene_text: &str, world_src: &str, tag: &str) -> Host {
    let dir = write_project(scene_text, world_src, tag);
    let (doc, registry, code, _) = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .unwrap_or_else(|e| panic!("{tag} load scene project: {e:?}"));
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    assert_eq!(
        h.play.as_ref().unwrap().character_profile(),
        Some(CharProfile::ThreeD),
        "{tag}: the run must use the 3D character profile"
    );
    h.pause_play("");
    settle(&mut h);
    h
}

/// Same chain with an explicit mutated source (negative controls never touch
/// the committed products): the project's hash-verified script stays
/// pristine; the mutated bytes ride the explicit snapshot (gallery pattern).
fn start_with_source(
    scene_text: &str,
    pristine_world: &str,
    mutated_world: &str,
    name: &str,
    api: autographics_auto_driver::authoring::behavior::Api,
    tag: &str,
) -> Host {
    let dir = write_project(scene_text, pristine_world, tag);
    let (doc, registry, code, _) = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .unwrap_or_else(|e| panic!("{tag} load scene project: {e:?}"));
    let snapshot =
        SourceSnapshot::from_bytes_for(Path::new(name), mutated_world.as_bytes().to_vec(), api)
            .unwrap_or_else(|e| panic!("{tag} admission: {e:?}"));
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_user_snapshot(snapshot)
        .unwrap_or_else(|e| panic!("{tag} start: {e:?}"));
    h.pause_play("");
    settle(&mut h);
    h
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or(false);
        if !pending {
            break;
        }
        assert!(Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

/// One logical tick: resume -> restore held bits / inject edges -> dispatch
/// -> pause -> settle (no wall-clock credit can interleave under Paused).
fn step(h: &mut Host, held: u32, pressed: u32) -> Result<(), String> {
    {
        let p = h.play.as_mut().expect("play");
        p.resume()?;
        for (action, bit) in [(0usize, 1u32), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)] {
            if held & bit != 0 {
                p.char_held_restore(action);
            }
            if pressed & bit != 0 {
                p.char_key(action, true, false);
                p.char_key(action, false, false);
            }
        }
        p.dispatch(Action::default())?;
    }
    h.pause_play("");
    settle(h);
    Ok(())
}

fn actor(h: &Host, id: u64) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.nodes
        .get(&id)
        .map(|d| d.translation)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

fn row_id(doc: &Document, name: &str) -> u64 {
    doc.objects
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("row {name}"))
        .id
}

fn scene_vp(h: &Host) -> Mat4 {
    match h.rt.content(h.three) {
        Some(SurfaceContent::Scene3d(c)) => c.view_proj,
        _ => panic!("3D scene content"),
    }
}

fn expected_vp(yaw: f32, pitch: f32, dist: f32, pos: Vec3, ortho: Option<f32>) -> Mat4 {
    let (sy, cy) = yaw.to_radians().sin_cos();
    let (sp, cp) = pitch.to_radians().sin_cos();
    let projection = match ortho {
        Some(size) => Projection::Orthographic {
            size,
            aspect: 2.0,
            near: 0.05,
            far: 256.0,
        },
        None => Projection::Perspective {
            fov_y_deg: 60.0,
            aspect: 2.0,
            near: 0.05,
            far: 256.0,
        },
    };
    Camera3d {
        eye: pos + Vec3::new(sy * cp, sp, cy * cp) * dist,
        target: pos,
        up: Vec3::Y,
        projection,
    }
    .view_proj()
}

fn snap_png(h: &mut Host, name: &str) -> (u32, u32, Vec<u8>) {
    let (_, png, w, ht) = h.rt.capture(h.three, name).expect("capture");
    (w, ht, png)
}

/// The camera family re-enters the SDK parser per start; give it the same
/// 64 MiB runway the other runtime-camera tests use.
fn big_stack(test: impl FnOnce() + Send) {
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(s, test)
            .expect("spawn")
            .join()
            .unwrap_or_else(|e| {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "opaque panic".to_string());
                panic!("big_stack worker: {msg}");
            });
    });
}

const HELD_FORWARD: u32 = 0b0100;
const HELD_RIGHT: u32 = 0b0010;
const HELD_DIAG: u32 = 0b0110;

// ---------------------------------------------- G07 layout-a（9 项 oracle） --

#[test]
fn g07_layout_a_nine_oracles_through_the_authored_chain() {
    big_stack(|| {
        // import_layout_a：文档 3 对象全 cylinder（经真实 load_project）。
        let dir = write_project(G07A_SCENE, G07A_WORLD, "g07a-import");
        let (doc_a, _, _, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc a");
        assert_eq!(doc_a.objects.len(), 3, "layout-a objects");
        assert!(
            doc_a
                .objects
                .iter()
                .all(|o| o.kind == autographics_auto_driver::authoring::document::Kind::Cylinder),
            "layout-a kinds"
        );
        let hero = row_id(&doc_a, "Main/hero/body");

        let mut h = start(G07A_SCENE, G07A_WORLD, "g07a-run");
        // forward_moves_on_z + supported_on_ground：60 forward。
        for _ in 0..STEPS {
            step(&mut h, HELD_FORWARD, 0).expect("forward step");
        }
        let p60 = actor(&h, hero);
        assert!(p60[2].abs() > 3.5, "forward_moves_on_z: z60={}", p60[2]);
        assert!(
            (p60[1] - 0.5).abs() < 1e-3,
            "supported_on_ground: y60={}",
            p60[1]
        );
        // windowless_png：800x400 离屏 GPU 捕获。
        let (w, ht, png) = snap_png(&mut h, "plan027-g07a-f60");
        assert!(w == 800 && ht == 400 && png.len() > 10_000, "png={w}x{ht} bytes={}", png.len());
        // frozen_input_still：60 中性 tick 位置逐位不动。
        let p0 = actor(&h, hero);
        for _ in 0..STEPS {
            step(&mut h, 0, 0).expect("neutral step");
        }
        assert_eq!(p0, actor(&h, hero), "frozen_input_still");
        // speed4_to_5_sensitive：同链 speed 5.0 -> |z| > 慢速 1.2 倍。
        let mut hf = start(G07A_FAST_SCENE, G07A_FAST_WORLD, "g07a-fast");
        for _ in 0..STEPS {
            step(&mut hf, HELD_FORWARD, 0).expect("fast forward");
        }
        let pf60 = actor(&hf, hero);
        assert!(
            pf60[2].abs() > p60[2].abs() * 1.2,
            "speed4_to_5_sensitive: slow_z={} fast_z={}",
            p60[2],
            pf60[2]
        );
        // diagonal_normalized：一个斜向 tick 的欧氏里程 = 一个轴向 tick。
        let mut hd = start(G07A_SCENE, G07A_WORLD, "g07a-diag");
        let p_start = actor(&hd, hero);
        step(&mut hd, HELD_FORWARD, 0).expect("axis tick");
        let axis = actor(&hd, hero);
        let axis_len = (axis[2] - p_start[2]).abs();
        step(&mut hd, 0, 0).expect("neutral tick");
        let diag0 = actor(&hd, hero);
        step(&mut hd, HELD_DIAG, 0).expect("diag tick");
        let diag = actor(&hd, hero);
        let diag_len = ((diag[0] - diag0[0]).powi(2) + (diag[2] - diag0[2]).powi(2)).sqrt();
        assert!(
            (diag_len - axis_len).abs() < 1e-3 && diag_len > 0.0,
            "diagonal_normalized: axis={axis_len} diag={diag_len}"
        );
        // x_wall_stops：200 right -> 5.498 接触包络（墙内面 5.9 - 半宽
        // 0.4 - clearance 0.002）。
        let mut hw = start(G07A_SCENE, G07A_WORLD, "g07a-wall");
        for _ in 0..200 {
            step(&mut hw, HELD_RIGHT, 0).expect("right step");
        }
        let pw = actor(&hw, hero);
        assert!(
            (pw[0] - 5.498).abs() < 0.02,
            "x_wall_stops: x_after_push={}",
            pw[0]
        );
        // jump_edge_once：单次跳沿，顶点 >1.5，落回 0.5±0.01。
        let mut hj = start(G07A_SCENE, G07A_WORLD, "g07a-jump");
        step(&mut hj, 0, 0).expect("grounded latch");
        step(&mut hj, 0, 0b10000).expect("jump edge");
        let mut apex = actor(&hj, hero)[1];
        for _ in 0..STEPS {
            step(&mut hj, 0, 0).expect("flight");
            apex = apex.max(actor(&hj, hero)[1]);
        }
        let landed = actor(&hj, hero);
        assert!(
            apex > 1.5 && (landed[1] - 0.5).abs() < 0.01,
            "jump_edge_once: apex={apex} landed={}",
            landed[1]
        );
    });
}

// ---------------------------------------------- G07 layout-b（2 项 oracle） --

#[test]
fn g07_layout_b_import_and_pit_fall_divergence() {
    let dir = write_project(G07B_SCENE, G07B_WORLD, "g07b-import");
    let (doc_b, _, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc b");
    assert_eq!(doc_b.objects.len(), 4, "import_layout_b");
    let hero = row_id(&doc_b, "Main/hero/body");
    let mut h = start(G07B_SCENE, G07B_WORLD, "g07b-run");
    for _ in 0..STEPS {
        step(&mut h, HELD_FORWARD, 0).expect("forward step");
    }
    let pb60 = actor(&h, hero);
    assert!(pb60[1] < -3.0, "pit_fall_divergence: y60={}", pb60[1]);
}

// -------------------------------------------------- G08/3d（13 项 oracle） --

#[test]
fn g08_3d_thirteen_oracles_through_the_authored_chain() {
    big_stack(|| {
        let dir = write_project(G08_SCENE, G08_WORLD, "g08-import");
        let (doc3, _, _, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc 3d");
        let hero = row_id(&doc3, "Main/hero/body");
        let mut h = start(G08_SCENE, G08_WORLD, "g08-run");
        // orbit_config_from_source：yaw0/pitch45/dist8/perspective 全来自源。
        {
            let cam3 = h.play.as_ref().unwrap().camera_frame().expect("tick0 camera3d").clone();
            assert!(
                cam3.yaw == 0.0
                    && (cam3.pitch - 45.0).abs() < 1e-4
                    && (cam3.distance - 8.0).abs() < 1e-4
                    && cam3.projection == 0,
                "orbit_config_from_source: {cam3:?}"
            );
        }
        // vp_identity_tick0：co-published VP == 独立 glam oracle（精确）。
        let pos0 = Vec3::from(actor(&h, hero));
        assert_eq!(
            scene_vp(&h),
            expected_vp(0.0, 45.0, 8.0, pos0, None),
            "vp_identity_tick0"
        );
        let png_a = snap_png(&mut h, "plan027-g08-yaw0").2;
        // forward_and_copublished_vp：每 tick VP 跟随新位置且非旧位置。
        let mut vp_walk_ok = true;
        for i in 0..10 {
            step(&mut h, HELD_FORWARD, 0).expect("forward");
            let pos = Vec3::from(actor(&h, hero));
            let vp = scene_vp(&h);
            let want = expected_vp(0.0, 45.0, 8.0, pos, None);
            let stale = expected_vp(0.0, 45.0, 8.0, pos0, None);
            let moved = (pos.z - pos0.z).abs() > 1e-4 * (i as f32 + 1.0) || i > 0;
            if vp != want || (moved && vp == stale) {
                vp_walk_ok = false;
                break;
            }
        }
        assert!(vp_walk_ok, "forward_and_copublished_vp");
        // yaw90_viewport：host 拖拽 90 度 -> VP == oracle(90)。
        {
            let p = h.play.as_mut().unwrap();
            p.resume().expect("resume");
            p.cam_drag(90.0, 0.0);
            p.dispatch(Action::default()).expect("dispatch");
        }
        h.pause_play("");
        settle(&mut h);
        let pos90 = Vec3::from(actor(&h, hero));
        assert_eq!(
            scene_vp(&h),
            expected_vp(90.0, 45.0, 8.0, pos90, None),
            "yaw90_viewport"
        );
        let png_b = snap_png(&mut h, "plan027-g08-yaw90").2;
        assert!(png_a != png_b, "orbit_changes_pixels");
        // world_motion_rotates_with_orbit：yaw90 后 forward = +x。
        let before = pos90;
        step(&mut h, HELD_FORWARD, 0).expect("forward after orbit");
        let after = Vec3::from(actor(&h, hero));
        assert!(
            after.x - before.x > 0.05 && (after.z - before.z).abs() < 1e-3,
            "world_motion_rotates_with_orbit: dx={} dz={}",
            after.x - before.x,
            after.z - before.z
        );
        // paused_zoom_dropped：Paused 累积的 wheel 不进候选。
        {
            let p = h.play.as_mut().unwrap();
            p.cam_zoom(-8.0);
        }
        step(&mut h, 0, 0).expect("paused settle");
        let camz = h.play.as_ref().unwrap().camera_frame().unwrap().clone();
        assert!(
            (camz.distance - 8.0).abs() < 1e-4,
            "paused_zoom_dropped: {}",
            camz.distance
        );
        // zoom_clamps_at_ceiling：Running zoom -8 -> 8*1.1^8 夹到 12。
        {
            let p = h.play.as_mut().unwrap();
            p.resume().expect("resume");
            p.cam_zoom(-8.0);
            p.dispatch(Action::default()).expect("dispatch");
        }
        h.pause_play("");
        settle(&mut h);
        let camz = h.play.as_ref().unwrap().camera_frame().unwrap().clone();
        let want = (8.0f32 * 1.1f32.powf(8.0)).clamp(4.0, 12.0);
        assert!(
            (camz.distance - want).abs() < 1e-4,
            "zoom_clamps_at_ceiling: {} want {want}",
            camz.distance
        );
        // stop_restores_document：Stop 前后场景文档逐字节一致。
        let doc_before = serde_json::to_string(&h.editor.document).unwrap();
        h.play.as_mut().unwrap().stop();
        let doc_after = serde_json::to_string(&h.editor.document).unwrap();
        assert_eq!(doc_before, doc_after, "stop_restores_document");
        // restart_at_original_position：全新 Host/Run 从原点起（x/z）。
        let h2 = start(G08_SCENE, G08_WORLD, "g08-fresh");
        let fresh = actor(&h2, hero);
        assert!(
            (fresh[0] - pos0.x).abs() < 1e-4 && (fresh[2] - pos0.z).abs() < 1e-4,
            "restart_at_original_position: {fresh:?} want x0={:?}",
            pos0
        );
        // alt_projection_config + alt_vp_ortho：正交 16 + 6/30 配置。
        let dir_alt = write_project(G08_ALT_SCENE, G08_ALT_WORLD, "g08-alt");
        let (doc_alt, reg_alt, code_alt, _) =
            Storage::load_project(&dir_alt.join("scene.agscene"), &Registry::new())
                .expect("doc alt");
        let hero_alt = row_id(&doc_alt, "Main/hero/body");
        let snapshot = SourceSnapshot::from_bytes_for(
            Path::new("alt-world.at"),
            G08_ALT_WORLD.as_bytes().to_vec(),
            autographics_auto_driver::authoring::behavior::Api::Camera3V1,
        )
        .expect("alt admission");
        let mut ha = Host::with_project(doc_alt, reg_alt, code_alt).expect("host alt");
        ha.start_user_snapshot(snapshot).expect("alt start");
        ha.pause_play("");
        settle(&mut ha);
        let camalt = ha.play.as_ref().unwrap().camera_frame().unwrap().clone();
        assert!(
            (camalt.distance - 6.0).abs() < 1e-4
                && (camalt.pitch - 30.0).abs() < 1e-4
                && camalt.ortho_size > 0.0,
            "alt_projection_config: {camalt:?}"
        );
        let posalt = Vec3::from(actor(&ha, hero_alt));
        assert_eq!(
            scene_vp(&ha),
            expected_vp(0.0, 30.0, 6.0, posalt, Some(16.0)),
            "alt_vp_ortho"
        );
        // neg_world_axis_source_distinguished：冻结世界轴假源在 orbit 后仍
        // 走 +z（真源走 +x），证明读取不是世界轴伪装。
        let mut wa = G08_WORLD
            .replace("let fx = ag_cam3_forward_x()", "let fx = 0.0")
            .replace("let fz = ag_cam3_forward_z()", "let fz = -1.0")
            .replace("let rx = ag_cam3_right_x()", "let rx = 1.0")
            .replace("let rz = ag_cam3_right_z()", "let rz = 0.0");
        assert_ne!(wa, G08_WORLD, "mutation applied");
        let snapshot_wa = SourceSnapshot::from_bytes_for(
            Path::new("world-axis.at"),
            {
                let _ = &mut wa;
                wa.as_bytes().to_vec()
            },
            autographics_auto_driver::authoring::behavior::Api::Camera3V1,
        )
        .expect("wa admission");
        let (doc_wa, reg_wa, code_wa, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc wa");
        let hero_wa = row_id(&doc_wa, "Main/hero/body");
        let mut hw = Host::with_project(doc_wa, reg_wa, code_wa).expect("host wa");
        hw.start_user_snapshot(snapshot_wa).expect("wa start");
        hw.pause_play("");
        settle(&mut hw);
        {
            let p = hw.play.as_mut().unwrap();
            p.resume().expect("resume");
            p.cam_drag(90.0, 0.0);
            p.dispatch(Action::default()).expect("dispatch");
        }
        hw.pause_play("");
        settle(&mut hw);
        let wb = Vec3::from(actor(&hw, hero_wa));
        step(&mut hw, HELD_FORWARD, 0).expect("wa forward");
        let wafter = Vec3::from(actor(&hw, hero_wa));
        assert!(
            wafter.z - wb.z > 0.05 && (wafter.x - wb.x).abs() < 1e-3,
            "neg_world_axis_source_distinguished: dx={} dz={}",
            wafter.x - wb.x,
            wafter.z - wb.z
        );
    });
}

// ------------------------------------- V-S03：typed/预算/配置的运行时拒绝 --

#[test]
fn vs03_runtime_rejections_fuel_the_named_faults() {
    big_stack(|| {
        // 33 计费（第二 move）-> fault 10，无部分帧。
        {
            let mut h = start(TWO_MOVES_SCENE, TWO_MOVES_WORLD, "two-moves");
            let _ = step(&mut h, 0, 0);
            let err = h.play.as_ref().unwrap().error.clone();
            assert!(
                err.as_deref().is_some_and(|e| e.contains("code=10")),
                "two-moves fault: {err:?}"
            );
        }
        // 旧 token 读 -> fault 13（latest-only）。
        {
            let mut h = start(STALE_SCENE, STALE_WORLD, "stale-token");
            step(&mut h, 0, 0).expect("tick 1 mints the token");
            let _ = step(&mut h, 0, 0);
            let err = h.play.as_ref().unwrap().error.clone();
            assert!(
                err.as_deref().is_some_and(|e| e.contains("code=13")),
                "stale-token fault: {err:?}"
            );
        }
        // 相机 target 非角色 -> fault 30（配置期）。
        {
            let bad = G08_WORLD.replace("ag_camera_orbit_3d(5,", "ag_camera_orbit_3d(1,");
            assert_ne!(bad, G08_WORLD, "target mutation applied");
            let h = start_with_source(
                G08_SCENE,
                G08_WORLD,
                &bad,
                "bad-target.at",
                autographics_auto_driver::authoring::behavior::Api::Camera3V1,
                "bad-target",
            );
            let err = h.play.as_ref().unwrap().error.clone();
            assert!(
                err.as_deref().is_some_and(|e| e.contains("code=30")),
                "bad target fault: {err:?}"
            );
        }
        // 重复相机配置 -> fault 29。
        {
            let line = G08_WORLD
                .lines()
                .find(|l| l.contains("ag_camera_orbit_3d("))
                .expect("orbit line")
                .trim()
                .to_string();
            // 重复配置注入在原 orbit 行之后（bind/solid 之后的第二次配置；
            // 注入在 app_start 开头会先撞 fault 30 的 target 未绑定序）。
            let bad = G08_WORLD.replace(&line, &format!("{line}\n {line}"));
            assert_ne!(bad, G08_WORLD, "dup config mutation applied");
            let h = start_with_source(
                G08_SCENE,
                G08_WORLD,
                &bad,
                "dup-config.at",
                autographics_auto_driver::authoring::behavior::Api::Camera3V1,
                "dup-config",
            );
            let err = h.play.as_ref().unwrap().error.clone();
            assert!(
                err.as_deref().is_some_and(|e| e.contains("code=29")),
                "dup config fault: {err:?}"
            );
        }
        // 相机域外配置（distance 99）-> fault 27。
        {
            let bad = G08_WORLD.replace("ag_camera_orbit_3d(5, 8.0,", "ag_camera_orbit_3d(5, 99.0,");
            assert_ne!(bad, G08_WORLD, "domain mutation applied");
            let h = start_with_source(
                G08_SCENE,
                G08_WORLD,
                &bad,
                "bad-domain.at",
                autographics_auto_driver::authoring::behavior::Api::Camera3V1,
                "bad-domain",
            );
            let err = h.play.as_ref().unwrap().error.clone();
            assert!(
                err.as_deref().is_some_and(|e| e.contains("code=27")),
                "domain fault: {err:?}"
            );
        }
        // wide128：128 行文档边界的正面包络（2 注册 solid + 125 非固体柱；
        // 每次 move3d 扫描全部 128 identities 仍在预算内）。
        {
            let mut hw = start(WIDE128_SCENE, WIDE128_WORLD, "wide128-probe");
            assert!(
                hw.play.as_ref().unwrap().error.is_none(),
                "wide128 init: {:?}",
                hw.play.as_ref().unwrap().error
            );
            for _ in 0..10 {
                step(&mut hw, HELD_FORWARD, 0).expect("wide128 forward");
            }
            assert!(
                hw.play.as_ref().unwrap().error.is_none(),
                "wide128 walk: {:?}",
                hw.play.as_ref().unwrap().error
            );
        }
        // deep127：127 根注册 solid 全部与角色足迹重叠 —— 越限反例在 INIT
        // 即被原累计步门整帧拒绝（127solid初始化拒绝，保原负控）。
        {
            let h = start(DEEP127_SCENE, DEEP127_WORLD, "deep127");
            let p = h.play.as_ref().unwrap();
            assert!(
                p.error
                    .as_deref()
                    .is_some_and(|e| e.contains("cumulative CPU budget exhausted")),
                "deep127 must refuse at init via the unchanged cumulative step guard: {:?}",
                p.error
            );
        }
    });
}

// ------------------------------------- V-S04：输入/相机事件与生命周期面 --

#[test]
fn vs04_host_event_battery_over_the_authored_chain() {
    big_stack(|| {
        let dir = write_project(G08_SCENE, G08_WORLD, "g08-events");
        let (doc, _, _, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        let hero = row_id(&doc, "Main/hero/body");
        let mut h = start(G08_SCENE, G08_WORLD, "g08-events-run");
        // blur 清空 held/边沿/相机拖拽：blur 后 resume 中性推进。
        step(&mut h, HELD_FORWARD, 0).expect("hold forward");
        let before = actor(&h, hero);
        h.blur();
        step(&mut h, 0, 0).expect("post-blur step");
        let after = actor(&h, hero);
        assert_eq!(before, after, "blur clears held (no drift after focus loss)");
        // Step 中性：单步推进不携带相机/输入残余。
        let p0 = actor(&h, hero);
        {
            let p = h.play.as_mut().unwrap();
            p.single_step().expect("single step");
        }
        settle(&mut h);
        let p1 = actor(&h, hero);
        assert!(
            ((p1[0] - p0[0]).abs() + (p1[2] - p0[2]).abs()) < 1e-6,
            "neutral step must not drift: {p0:?} -> {p1:?}"
        );
    });
}

// ------------------------------------- V-S06：A->B / SaveAs / 恢复 / 20 轮 --

#[test]
fn vs06_b_run_save_reopen_damaged_restore_and_20_round_lifecycle() {
    big_stack(|| {
        // A->B：仅闭包字节的全新目录（无生产者树）作为 B，真实 Run 从 B 绿。
        let dir = write_project(G07A_SCENE, G07A_WORLD, "vs06-a");
        let b = dir
            .parent()
            .unwrap()
            .join(format!("{}-vs06-b", std::process::id()));
        let _ = std::fs::remove_dir_all(&b);
        std::fs::create_dir_all(b.join("scripts")).expect("b scripts");
        std::fs::copy(
            dir.join("scene.agscene"),
            b.join("scene.agscene"),
        )
        .expect("copy scene");
        for e in std::fs::read_dir(dir.join("scripts")).expect("scripts") {
            let e = e.expect("entry");
            std::fs::copy(e.path(), b.join("scripts").join(e.file_name())).expect("copy script");
        }
        {
            let (doc, registry, code, _) =
                Storage::load_project(&b.join("scene.agscene"), &Registry::new())
                    .expect("B load");
            let mut hb = Host::with_project(doc, registry, code).expect("B host");
            hb.start_selected_play().expect("B start");
            hb.pause_play("");
            settle(&mut hb);
            step(&mut hb, HELD_FORWARD, 0).expect("B forward");
        }
        // SaveAs：项目字节复制到第三个目录，关（drop）后从副本重开再 Run。
        let saveas = dir
            .parent()
            .unwrap()
            .join(format!("{}-vs06-saveas", std::process::id()));
        let _ = std::fs::remove_dir_all(&saveas);
        std::fs::create_dir_all(saveas.join("scripts")).expect("saveas scripts");
        std::fs::copy(dir.join("scene.agscene"), saveas.join("scene.agscene")).expect("save scene");
        for e in std::fs::read_dir(dir.join("scripts")).expect("scripts") {
            let e = e.expect("entry");
            std::fs::copy(e.path(), saveas.join("scripts").join(e.file_name()))
                .expect("save script");
        }
        {
            let (doc, registry, code, _) =
                Storage::load_project(&saveas.join("scene.agscene"), &Registry::new())
                    .expect("saveas load");
            let mut hs = Host::with_project(doc, registry, code).expect("saveas host");
            hs.start_selected_play().expect("saveas start");
            hs.pause_play("");
            settle(&mut hs);
        }
        // 坏件拒绝 -> 精确恢复 -> 再 Run：world 副本单字节损坏在载入拒绝。
        let scripts = dir.join("scripts");
        let world_name = {
            let mut it = std::fs::read_dir(&scripts).expect("scripts");
            let p = it.next().expect("one script").expect("entry").path();
            p.file_name().unwrap().to_str().unwrap().to_string()
        };
        let world_path = scripts.join(&world_name);
        let mut bytes = std::fs::read(&world_path).expect("world bytes");
        let at = bytes.len() / 2;
        bytes[at] = bytes[at].wrapping_sub(1);
        std::fs::write(&world_path, &bytes).expect("damage world");
        assert!(
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).is_err(),
            "damaged world.at must be rejected at load"
        );
        std::fs::write(&world_path, G07A_WORLD).expect("restore world");
        {
            let (doc, registry, code, _) =
                Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
                    .expect("restore load");
            let mut hr = Host::with_project(doc, registry, code).expect("restore host");
            hr.start_selected_play().expect("restore start");
            hr.pause_play("");
            settle(&mut hr);
            step(&mut hr, HELD_FORWARD, 0).expect("restore forward");
        }
        // 20 轮生命周期（单 Host start/stop；暖上传网格不随轮重传）。
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
                .expect("lifecycle load");
        let mut h = Host::with_project(doc, registry, code).expect("lifecycle host");
        let warm = h.resource_snapshot()["mesh_count"].as_u64().expect("warm");
        for round in 0..20 {
            h.start_selected_play()
                .unwrap_or_else(|e| panic!("round {round} start: {e:?}"));
            h.pause_play("");
            settle(&mut h);
            step(&mut h, HELD_FORWARD, 0).unwrap_or_else(|e| panic!("round {round} step: {e}"));
            h.stop_play();
        }
        let after = h.resource_snapshot()["mesh_count"].as_u64().expect("after");
        assert_eq!(warm, after, "20-round lifecycle must keep mesh uploads flat");
        // 最后 owner 释放：文档不改（Stop 后逐字节一致）。
        // （文档恒等由 vs03/其余 Stop-restore 断言覆盖；此处锁暖账。）
    });
}

// ------------------------------------- V-S05：内置视觉原 owner 保真对照 --

#[test]
fn vs05_builtin_visuals_render_via_the_original_owner_upload() {
    big_stack(|| {
        let dir = write_project(STATIC_SCENE, "", "static-load");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
                .expect("static doc");
        // 无行为产品：code None、run 不启动（纯静态场景无用户 VM）。
        assert!(code.is_none(), "static scene carries no behavior payload");
        let mut h = Host::with_project(doc.clone(), registry.clone(), None).expect("static host");
        // 渲染对照：同一文档（Kind::Player/Mob/ground）的 GPU 捕获逐位稳定
        //（两次捕获同 hash），且与去掉内置视觉的 ground-only 文档不同
        //（内置模型贡献真实像素 = 原 owner mesh 被真正渲染）。
        for _ in 0..3 {
            h.frame(None).expect("static frame");
        }
        let cap1 = h.rt.capture(h.three, "plan027-static-a").expect("capture a");
        let cap2 = h.rt.capture(h.three, "plan027-static-b").expect("capture b");
        assert_eq!(cap1.0, cap2.0, "static built-in scene must capture deterministically");
        let mut ground_only = doc.clone();
        ground_only
            .objects
            .retain(|o| o.kind != autographics_auto_driver::authoring::document::Kind::Player && o.kind != autographics_auto_driver::authoring::document::Kind::Mob);
        assert_eq!(ground_only.objects.len(), 1, "ground only");
        let mut hg = Host::with_project(ground_only, registry.clone(), None).expect("ground host");
        for _ in 0..3 {
            hg.frame(None).expect("ground frame");
        }
        let capg = hg.rt.capture(hg.three, "plan027-ground-only").expect("capture ground");
        assert_ne!(cap1.0, capg.0, "built-in visuals must change the rendered frame");
        assert!(cap1.1.len() > 5_000, "png bytes {}", cap1.1.len());
        // 原 owner 暖上传：三个网格（fixed cylinder + player.glb + mob.glb），
        // 不因声明实例数重传（mesh_uploads 在无 Run 时保持 3）。
        // 原 owner 暖上传：常驻 store = 圆柱 1 + player.glb 展开 3 + mob.glb
        // 展开 3 = 7 个网格（声明实例不增加网格；按原 owner 账）。
        let snap = h.resource_snapshot();
        let mesh_count = snap["mesh_count"].as_u64().expect("mesh count");
        assert_eq!(mesh_count, 7, "warm store: cylinder + player(3) + mob(3) meshes");
    });
}
