//! PLAN-034 components-3d-presentation flow（T-02 候选A有界先证）。
//! 生成链产品（pilot components-3d-presentation 两遍：声明 → 既有公开导入
//! 元数据 → 最终产品）→ Storage::load_project → Host → 真实 Auto/GPU：
//! 纹理面 = closure 场景引用装载、原 coin x 摆动（main 0.8 / alt 1.6）、
//! flag tick2 隐 tick3 复现、纹理像素真变；动画面 = ThreeD 档、原模型
//! body、idle/walk/jump 具名 clip 降层、相位推进（alt 2×）、相机相对朝向、
//! 32-credit 预算（无 fault）。CPU 帧证据 + 窗口外 GPU 捕获。
//! 运行：cargo test --locked --offline --test node3d_presentation_flow -- --test-threads=1
use autographics_auto_driver::authoring::document::Document;
use autographics_auto_driver::authoring::{
    assets::Registry,
    host::Host,
    play::{Action, CharProfile},
    storage::Storage,
};
use std::path::{Path, PathBuf};
use std::time::Instant;

const GEN: &str = "../autographics-node2d-pilot/generated-components3d-presentation";

fn product_text(name: &str, file: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(GEN)
        .join(name)
        .join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn row_id(doc: &Document, name: &str) -> u64 {
    doc.objects
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("row {name}"))
        .id
}

fn write_project(scene_text: &str, world_src: &str, product: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan034-presentation")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    if let Some(sha) = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
    {
        std::fs::write(scripts_join(&dir, sha), world_src).expect("write script");
    }
    let src_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(GEN)
        .join(product)
        .join("assets");
    for entry in std::fs::read_dir(&src_assets).expect("generated assets dir") {
        let entry = entry.expect("asset entry");
        let dest = dir.join("assets").join(entry.file_name());
        std::fs::create_dir_all(dest.parent().expect("assets parent")).expect("assets dir");
        std::fs::copy(entry.path(), &dest).expect("asset copy");
    }
    dir
}

fn scripts_join(dir: &Path, sha: &str) -> PathBuf {
    dir.join("scripts").join(format!("{sha}.at"))
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
/// -> pause -> settle（同 030 冻结节奏）。
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

fn play_error(h: &Host) -> String {
    h.play.as_ref().unwrap().error.clone().unwrap_or_default()
}

fn actor(h: &Host, id: u64) -> [f32; 3] {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.nodes.get(&id))
        .map(|d| d.translation)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

fn visible(h: &Host, id: u64) -> bool {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.nodes.get(&id))
        .map(|d| d.visible)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

fn pose_of(h: &Host, id: u64) -> (i32, f32, f32) {
    let p = h.play.as_ref().unwrap();
    let anim = p
        .last
        .as_ref()
        .and_then(|f| f.animations.as_ref())
        .unwrap_or_else(|| panic!("animation frame"));
    let v = anim
        .visual_for(id)
        .unwrap_or_else(|| panic!("visual target {id}"));
    (v.clip, v.phase, v.yaw)
}

fn start_textures(product: &str, tag: &str) -> (Host, Document) {
    let scene = product_text(product, "scene.agscene");
    let world = product_text(product, "world.at");
    let dir = write_project(&scene, &world, product, tag);
    let (doc, registry, code, _) = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .unwrap_or_else(|e| panic!("{tag} load scene project: {e:?}"));
    let mut h = Host::with_project(doc.clone(), registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    settle(&mut h);
    (h, doc)
}

#[test]
fn textured_closure_scene_loads_moves_hides_and_renders_real_gpu() {
    let (mut h, doc) = start_textures("g10-textures", "textures-main");
    // closure 场景引用 + 纯场景无角色。
    let ground = row_id(&doc, "ground");
    let coin = row_id(&doc, "Main/coin/coin");
    let banner = row_id(&doc, "Main/flag/banner");
    let uv = row_id(&doc, "uv");
    for name in ["ground", "uv"] {
        let o = doc.objects.iter().find(|o| o.name == name).expect(name);
        assert!(
            o.resource.as_deref().unwrap_or("").starts_with("closure:"),
            "{name} must bind by inspected closure key, got {:?}",
            o.resource
        );
    }
    assert_eq!(
        h.play.as_ref().unwrap().character_profile(),
        None,
        "actor-free texture scene has no character profile"
    );
    // GPU 首帧真渲染（窗口外捕获，t1: coin@0.8、flag 可见）。
    step(&mut h, 0, 0).expect("tick1");
    assert!(play_error(&h).is_empty(), "tick1 error: {}", play_error(&h));
    let (_, png1, _, _) = h.rt.capture(h.three, "t1").expect("capture t1");
    assert_eq!(actor(&h, coin)[0], 0.8, "coin x after tick1 (main amplitude)");
    assert!(visible(&h, banner), "flag visible initial");
    assert!(actor(&h, ground)[0] == -6.0 && actor(&h, uv)[0] == -5.0, "static rows keep original transforms");
    // tick2: coin 回 0、flag 隐（tick2 隐、tick3 复现）。
    step(&mut h, 0, 0).expect("tick2");
    assert_eq!(actor(&h, coin)[0], 0.0, "coin x after tick2");
    assert!(!visible(&h, banner), "flag hidden exactly at tick2");
    let (_, png2, _, _) = h.rt.capture(h.three, "t2").expect("capture t2");
    assert_ne!(png1, png2, "tick2 pixels must differ (flag hidden + coin moved)");
    // tick3: flag 复现。
    step(&mut h, 0, 0).expect("tick3");
    assert_eq!(actor(&h, coin)[0], 0.8, "coin x after tick3");
    assert!(visible(&h, banner), "flag restored at tick3");
    assert!(play_error(&h).is_empty());
}

#[test]
fn textured_alt_amplitude_only_difference_is_real_on_gpu() {
    let (mut h, doc) = start_textures("g10-textures-alt", "textures-alt");
    let coin = row_id(&doc, "Main/coin/coin");
    step(&mut h, 0, 0).expect("tick1");
    assert_eq!(actor(&h, coin)[0], 1.6, "alt amplitude 1.6 after tick1");
    step(&mut h, 0, 0).expect("tick2");
    assert_eq!(actor(&h, coin)[0], 0.0, "alt coin returns to 0");
    let (_, png, _, _) = h.rt.capture(h.three, "alt").expect("capture alt");
    assert!(!png.is_empty(), "alt renders on GPU");
    assert!(play_error(&h).is_empty());
}

fn start_animation(product: &str, tag: &str) -> (Host, Document) {
    let scene = product_text(product, "scene.agscene");
    let world = product_text(product, "world.at");
    let dir = write_project(&scene, &world, product, tag);
    let (doc, registry, code, _) = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .unwrap_or_else(|e| panic!("{tag} load scene project: {e:?}"));
    let mut h = Host::with_project(doc.clone(), registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    assert_eq!(
        h.play.as_ref().unwrap().character_profile(),
        Some(CharProfile::ThreeD),
        "{tag}: the run must use the 3D character profile (original model body)"
    );
    h.pause_play("");
    settle(&mut h);
    (h, doc)
}

#[test]
fn original_model_body_walks_camera_relative_with_named_clips_on_gpu() {
    let (mut h, doc) = start_animation("g10-animation", "anim-main");
    let body = row_id(&doc, "Main/hero/body");
    // Start 面：base static clip0、phase 0、loop 1、yaw 0。
    let (clip0, phase0, yaw0) = pose_of(&h, body);
    assert_eq!((clip0, yaw0), (0, 0.0), "Start pose = file clip0 static, yaw 0");
    let _ = phase0;
    // idle 相位推进（clip 1；无输入）。
    step(&mut h, 0, 0).expect("idle tick1");
    let (clip_idle, phase1, _) = pose_of(&h, body);
    assert_eq!(clip_idle, 1, "no input = idle clip (named idle=1)");
    step(&mut h, 0, 0).expect("idle tick2");
    let (_, phase2, _) = pose_of(&h, body);
    assert!(phase2 > phase1, "phase advances ({} -> {})", phase1, phase2);
    let idle_rate = phase2 - phase1;
    // 相机相对前进（Forward=bit4）→ walk clip 2 + 朝向 + 位移。
    let p0 = actor(&h, body);
    for _ in 0..3 {
        step(&mut h, 4, 0).expect("walk tick");
    }
    let (clip_walk, phase3, yaw_walk) = pose_of(&h, body);
    assert_eq!(clip_walk, 2, "forward = walk clip (named walk=2)");
    let p1 = actor(&h, body);
    let moved = ((p1[0] - p0[0]).powi(2) + (p1[2] - p0[2]).powi(2)).sqrt();
    assert!(moved > 0.1, "camera-relative forward must move the model body (moved {moved})");
    // 原四向公式：mz>0 → yaw = camyaw(rad)+π（±π/4 对角未按）。
    let expected = std::f32::consts::PI;
    let err = (yaw_walk - expected).abs();
    let err = err.min((err - std::f32::consts::TAU).abs());
    assert!(err < 1e-4, "forward facing yaw ~ pi (cam yaw 0), got {yaw_walk}");
    // 跳（pressed Jump=bit16）→ air clip 3。
    step(&mut h, 0, 16).expect("jump tick");
    let (clip_air, _, _) = pose_of(&h, body);
    assert_eq!(clip_air, 3, "jump = air clip (named jump=3)");
    // 落地后回 idle（跳跃滞空 ~2*8/20s，按 dt 步进等待，无 fault）。
    let mut clip_settled = 3;
    for _ in 0..400 {
        step(&mut h, 0, 0).expect("settle tick");
        clip_settled = pose_of(&h, body).0;
        if clip_settled == 1 {
            break;
        }
    }
    assert_eq!(clip_settled, 1, "grounded settle returns to idle");
    assert!(play_error(&h).is_empty(), "no budget fault: {}", play_error(&h));
    let _ = idle_rate;
    let (_, png, _, _) = h.rt.capture(h.three, "anim").expect("capture anim");
    assert!(!png.is_empty(), "animated textured model renders on GPU");
}

#[test]
fn animation_alt_phase_rate_is_exactly_two_times() {
    let (mut h, doc) = start_animation("g10-animation-alt", "anim-alt");
    let body = row_id(&doc, "Main/hero/body");
    step(&mut h, 0, 0).expect("alt tick1");
    let (_, p1, _) = pose_of(&h, body);
    step(&mut h, 0, 0).expect("alt tick2");
    let (_, p2, _) = pose_of(&h, body);
    let alt_rate = p2 - p1;
    let (mut hm, docm) = start_animation("g10-animation", "anim-main-rate");
    let bodym = row_id(&docm, "Main/hero/body");
    step(&mut hm, 0, 0).expect("main tick1");
    let (_, m1, _) = pose_of(&hm, bodym);
    step(&mut hm, 0, 0).expect("main tick2");
    let (_, m2, _) = pose_of(&hm, bodym);
    let main_rate = m2 - m1;
    assert!(
        (alt_rate - 2.0 * main_rate).abs() < 1e-5,
        "alt phase rate must be exactly 2x main (alt {alt_rate}, main {main_rate})"
    );
    assert!(play_error(&hm).is_empty());
}
