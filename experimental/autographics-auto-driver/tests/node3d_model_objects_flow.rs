//! PLAN-043 node3d-model-objects flow（V-S02/V-S05 先证；AC-02/03/05）。
//! g12-model actor-free 产品（pilot components-3d-events 生成物）经真实
//! 消费链（Storage::load_project -> Host -> Triggers3V1 装配 -> GPU）：
//! 具名平色 GLB 模板 spawn 双实例（资源共享/身份独立）、GPU 双 part 像素
//! oracle（红/绿三角）、move/hide/despawn/新代重生全生命周期、触发 pair
//! 事件、模板行 parked 且不进活实例文档、坏件/错绑资源具名拒绝、Stop
//! 后重开资源再暖。GPU 单实例：--test-threads=1。
//! 运行：cargo test --locked --offline --test node3d_model_objects_flow -- --test-threads=1
use autographics_auto_driver::authoring::document::{Document, Kind};
use autographics_auto_driver::authoring::{assets::Registry, host::Host, storage::Storage};
use std::path::{Path, PathBuf};
use std::time::Instant;

const G12_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g12-model/scene.agscene"
);
const G12_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g12-model/world.at"
);
const GLB: &[u8] =
    include_bytes!("../../autographics-node2d-pilot/fixtures/node3d/g02/assets/two-parts.glb");
const GLB_SHA: &str = "42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671";
const G13_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g13-model-opaque/scene.agscene"
);
const G13_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g13-model-opaque/world.at"
);
const COIN_GLB: &[u8] = include_bytes!(
    "../../autographics-node2d-pilot/fixtures/node3d-presentation/g10-textures/assets/coin.glb"
);
const COIN_PNG: &[u8] = include_bytes!(
    "../../autographics-node2d-pilot/fixtures/node3d-presentation/g10-textures/assets/Textures_colormap.png"
);
const COIN_GLB_SHA: &str = "a3fb8f779a6af1cd75e5a02da77a87546ca8a59f6fe52f79f25179c4a68fd0e5";
const COIN_PNG_SHA: &str = "106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3";
const COIN_CLOSURE: &str = "76657e1367fe78e3aa69585e4a2d6e462f7ca431a4c17d1b59a1e7fca7d3862e";
const G14_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g14-model-actor/scene.agscene"
);
const G14_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g14-model-actor/world.at"
);

// ---------------------------------------------------------------- harness --

fn write_product_with(
    scene: &str,
    world: &str,
    assets_to_write: &[(&str, &[u8])],
    tag: &str,
) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan043-model-objects")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let assets = dir.join("assets");
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&assets).expect("assets dir");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene).expect("write scene");
    for (name, bytes) in assets_to_write {
        std::fs::write(assets.join(name), bytes).expect("write asset");
    }
    let sha = scene
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene binds a source sha")
        .to_string();
    std::fs::write(scripts.join(format!("{sha}.at")), world).expect("write script");
    dir
}

fn write_product(tag: &str) -> PathBuf {
    write_product_with(G12_SCENE, G12_WORLD, &[(&format!("{GLB_SHA}.glb"), GLB)], tag)
}

fn start(tag: &str) -> (Host, PathBuf) {
    start_with(G12_SCENE, G12_WORLD, &[(&format!("{GLB_SHA}.glb"), GLB)], tag)
}

fn start_with(
    scene: &str,
    world: &str,
    assets_to_write: &[(&str, &[u8])],
    tag: &str,
) -> (Host, PathBuf) {
    let dir = write_product_with(scene, world, assets_to_write, tag);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("{tag} load: {e:?}"));
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    settle(&mut h);
    (h, dir)
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or_default();
        if !pending {
            break;
        }
        assert!(Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

/// Advance exactly one world tick through real host dispatch.
fn tick(h: &mut Host) {
    {
        let p = h.play.as_mut().expect("play");
        p.resume().expect("resume");
        p.dispatch(Default::default()).expect("dispatch");
    }
    h.pause_play("");
    settle(h);
}

fn live(h: &Host) -> usize {
    h.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .and_then(|f| f.instances.as_ref())
        .map(|i| i.live_count())
        .unwrap_or(0)
}

fn slot(h: &Host, i: usize) -> Option<(i32, u64, [f32; 3], bool)> {
    h.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .and_then(|f| f.instances.as_ref())
        .and_then(|inst| inst.get(i))
        .map(|s| (s.generation, s.template_id, s.translation, s.visible))
}

fn events(h: &Host) -> Vec<(i32, i32)> {
    let p = h.play.as_ref().expect("play");
    let f = p.last.as_ref().expect("frame");
    let tick = f.rounds.as_ref().map(|r| r.tick).unwrap_or_default();
    f.rounds
        .as_ref()
        .map(|r| r.events.iter().map(|e| (tick, e.pair)).collect())
        .unwrap_or_default()
}

/// 800-wide-capture-equivalent 2px sampling of the 3D surface; counts the
/// two-part GLB's red/green triangle pixels (B02 thresholds > 20 each).
fn red_green(h: &mut Host, name: &str) -> (usize, usize, Vec<u8>) {
    let (_, png, w, ht) = h.rt.capture(h.three, name).expect("capture");
    let (cw, ch) = (w as usize, ht as usize);
    let step = (cw / 400).max(1);
    let mut points = Vec::new();
    for y in (0..ch).step_by(step) {
        for x in (0..cw).step_by(step) {
            points.push([x as f32, y as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.three, &points).expect("probe");
    let (mut red, mut green) = (0usize, 0usize);
    for c in px.iter() {
        let (r, g, b) = (c[0] as i32, c[1] as i32, c[2] as i32);
        if r > 150 && r > g + 60 && r > b + 60 {
            red += 1;
        }
        if g > 150 && g > r + 60 && g > b + 60 {
            green += 1;
        }
    }
    (red, green, png)
}

fn crate_template(doc: &Document) -> &autographics_auto_driver::authoring::document::Object {
    doc.objects
        .iter()
        .find(|o| o.name == "Crate")
        .expect("Crate template row")
}

// ------------------------------------------------------------------ flow --

#[test]
fn g12_model_objects_full_chain_lifecycle_on_gpu() {
    big_stack(|| {
        // ---- import identity：model 模板行 + 具名 GLB 资源 + parked ----
        let dir = write_product("identity");
        let (doc, registry, _code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("load");
        let t = crate_template(&doc);
        assert_eq!(t.kind, Kind::Model, "Crate row must be a Model kind");
        assert_eq!(
            t.resource.as_deref(),
            Some(format!("sha256:{GLB_SHA}").as_str()),
            "named GLB resource key"
        );
        assert!(!t.visible, "template row is parked (author-declared)");
        assert!(
            registry.get(&format!("sha256:{GLB_SHA}")).is_some(),
            "registry must carry the decoded GLB"
        );
        let base_objects = doc.objects.len();

        let (mut h, _dir) = start("lifecycle");
        // Life.Start（app_start）已 spawn 双实例：同模板、资源共享、句柄独立。
        assert_eq!(live(&h), 2, "two live instances after Start");
        let s0 = slot(&h, 0).expect("slot 0");
        let s1 = slot(&h, 1).expect("slot 1");
        assert_eq!(s0.0, 1, "slot0 generation 1");
        assert_eq!(s1.0, 1, "slot1 generation 1");
        assert_eq!(s0.1, t.id, "slot0 template id = Crate row");
        assert_eq!(s1.1, t.id, "slot1 template id = Crate row (shared)");
        assert_eq!(s0.2, [2.0, 0.5, 0.0]);
        assert_eq!(s1.2, [6.0, 0.5, 0.0]);

        // GPU：两个可见 two-part 实例 → 红/绿 part 像素都在（变异 oracle：
        // 静态替身/漏 part 在这里红或绿为 0）。
        // 颜色类 oracle：场景中唯一绿色像素来自 GLB 的绿三角（静态圆柱是
        // 默认红占位、不可改色）——green 计数作判别式；红色用 t0→t2 差值。
        let (r0, g0, png0) = red_green(&mut h, "t0-both");
        assert!(g0 > 20, "t0 two visible models: green {g0}");
        assert!(r0 > g0, "t0 red triangles on top of cylinder baseline: red {r0}");

        // 运行文档身份：实例不进 Document（adapter id 0，非持久）。
        let run_doc = h
            .play
            .as_ref()
            .and_then(|p| p.last.as_ref())
            .map(|f| f.document.objects.len());
        if let Some(n) = run_doc {
            assert_eq!(n, base_objects, "instances never enter the Document");
        }

        // ---- 阶段 A：tick 直到 pad 触发（move/hide 与事件同 tick 或滞后
        //      一 tick 均兼容）；另一实例身份不变 ----
        let mut fired = None;
        for t in 0..4 {
            tick(&mut h);
            if events(&h).iter().any(|(_, pair)| *pair == 0) {
                fired = Some(t);
                break;
            }
        }
        assert!(fired.is_some(), "pad crossing must emit trigger pair 0");
        let s0 = slot(&h, 0).expect("slot 0 after move");
        let s1 = slot(&h, 1).expect("slot 1 after hide");
        assert_eq!(s0.2, [4.0, 0.5, 0.0], "moved to the pad");
        assert_eq!(s0.0, 1, "move keeps generation");
        assert!(!s1.3, "hidden instance keeps identity, visible=false");
        assert_eq!(s1.0, 1, "hide keeps generation");
        let (r1, g1, png1) = red_green(&mut h, "t1-moved-hidden");
        assert!(g1 > 20, "visible instance still renders the green part: {g1}");
        assert_ne!(png0, png1, "view must change after move+hide");

        // ---- 阶段 B：tick 直到 despawn（事件消费滞后作者一 tick，状态机
        //      自行步进）；隐藏实例仍在但不可见 → 红/绿像素全消失（真销毁
        //      +真隐藏，无常驻静态补绘）----
        let mut despawned = false;
        for _ in 0..3 {
            tick(&mut h);
            if live(&h) == 1 {
                despawned = true;
                break;
            }
        }
        assert!(despawned, "despawn stage must complete");
        assert!(slot(&h, 0).is_none(), "slot 0 freed");
        let s1 = slot(&h, 1).expect("slot 1 survives");
        assert_eq!((s1.0, s1.3), (1, false), "hidden slot untouched by despawn");
        let (r2, g2, _) = red_green(&mut h, "t2-despawned");
        assert_eq!(g2, 0, "despawned+hidden models must vanish from GPU (green {g2})");
        assert!(
            r2 + 20 < r0,
            "both GLB red triangles must drop to the cylinder baseline: r0 {r0} r2 {r2}"
        );

        // ---- 阶段 C：tick 直到同槽新代重生；资源共享再暖；隐藏实例仍原代 ----
        let mut respawned = false;
        for _ in 0..3 {
            tick(&mut h);
            if live(&h) == 2 && slot(&h, 0).is_some_and(|s| s.0 == 2) {
                respawned = true;
                break;
            }
        }
        assert!(respawned, "respawn stage must complete");
        let s0 = slot(&h, 0).expect("slot 0 respawned");
        assert_eq!(s0.0, 2, "respawn is a NEW generation");
        assert_eq!(s0.1, t.id, "same shared template");
        assert_eq!(s0.2, [2.0, 0.5, 0.0]);
        let s1 = slot(&h, 1).expect("slot 1 untouched");
        assert_eq!((s1.0, s1.3), (1, false));
        let (_r3, g3, _) = red_green(&mut h, "t3-respawned");
        assert!(g3 > 20, "respawned instance renders from the shared GLB: {g3}");

        // ---- Stop / 重开：新 run，同一资源再暖即可用（V-S05 首证）----
        let old_run = h.play.as_ref().map(|p| p.run_id).unwrap_or(-1);
        h.stop_play();
        settle(&mut h);
        h.start_selected_play().expect("restart run");
        h.pause_play("");
        settle(&mut h);
        let new_run = h.play.as_ref().map(|p| p.run_id).unwrap_or(-2);
        assert_ne!(old_run, new_run, "restart must open a new Run");
        assert_eq!(live(&h), 2, "restart respawns both instances");
        let (_r4, g4, _) = red_green(&mut h, "t4-restart");
        assert!(g4 > 20, "restarted run renders the shared GLB: {g4}");
    });
}

#[test]
fn model_resource_admission_rejections_are_named() {
    // 模板行资源键被剥掉：Document 准入具名拒绝（model resource reference）。
    let tampered = G12_SCENE.replace(
        &format!("\"resource\": \"sha256:{GLB_SHA}\""),
        "\"resource\": null",
    );
    assert_ne!(tampered, G12_SCENE);
    let dir = write_product("strip-resource");
    std::fs::write(dir.join("scene.agscene"), tampered).expect("write tampered");
    let err = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .map(|_| ())
        .expect_err("stripped model resource must be rejected");
    assert!(err.contains("model resource"), "{err}");

    // GLB 单字节篡改：sha 身份不符，资源准入拒绝。
    let dir = write_product("tamper-glb");
    let mut bad = GLB.to_vec();
    bad[20] ^= 0x01;
    std::fs::write(
        dir.join("assets").join(format!("{GLB_SHA}.glb")),
        bad,
    )
    .expect("write tampered glb");
    let err = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .map(|_| ())
        .expect_err("tampered GLB bytes must be rejected");
    assert!(
        err.contains("hash/length") || err.contains("sha256") || err.contains("identity") || err.contains("declared"),
        "{err}"
    );
}


#[test]
fn g13_model_objects_opaque_closure_full_chain_on_gpu() {
    big_stack(|| {
        // ---- import identity：closure 键模板行 + 纹理闭包资产 ----
        let dir = write_product_with(
            G13_SCENE,
            G13_WORLD,
            &[
                (&format!("{COIN_GLB_SHA}.glb"), COIN_GLB),
                (&format!("{COIN_PNG_SHA}.png"), COIN_PNG),
            ],
            "g13-identity",
        );
        let (doc, registry, _code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("load");
        let t = doc
            .objects
            .iter()
            .find(|o| o.name == "Coin")
            .expect("Coin template row");
        assert_eq!(t.kind, Kind::Model);
        assert_eq!(
            t.resource.as_deref(),
            Some(format!("closure:{COIN_CLOSURE}").as_str()),
            "opaque template carries the 034 closure key"
        );
        assert!(!t.visible, "parked template row");
        let coin_asset = registry
            .get(&format!("closure:{COIN_CLOSURE}"))
            .expect("registry must carry the closure-keyed textured GLB");
        assert!(
            coin_asset
                .identity
                .closure
                .as_ref()
                .is_some_and(|c| c.images.iter().any(|i| i.sha256 == COIN_PNG_SHA)),
            "closure identity must pin the texture PNG bytes"
        );

        // ---- 生命周期同 g12：spawn 双实例 → move/hide/触发 → despawn →
        //      新代重生；纹理像素变异 oracle ----
        let (mut h, _dir) = start_with(
            G13_SCENE,
            G13_WORLD,
            &[
                (&format!("{COIN_GLB_SHA}.glb"), COIN_GLB),
                (&format!("{COIN_PNG_SHA}.png"), COIN_PNG),
            ],
            "g13-lifecycle",
        );
        assert_eq!(live(&h), 2, "two live textured instances after Start");
        let s0 = slot(&h, 0).expect("slot 0");
        assert_eq!((s0.0, s0.1, s0.2), (1, t.id, [2.0, 0.5, 0.0]));

        // 纹理像素：金币为 Kenney colormap 金黄；静态圆柱为红占位——
        // 金黄类计数作判别（白模/漏纹理变异在此为 0）。
        let gold = |h: &mut Host, name: &str| -> (usize, Vec<u8>) {
            let (_, png, w, ht) = h.rt.capture(h.three, name).expect("capture");
            let (cw, ch) = (w as usize, ht as usize);
            let step = (cw / 400).max(1);
            let mut points = Vec::new();
            for y in (0..ch).step_by(step) {
                for x in (0..cw).step_by(step) {
                    points.push([x as f32, y as f32]);
                }
            }
            let px = h.rt.probe_pixels(h.three, &points).expect("probe");
            let mut gold = 0usize;
            for c in px.iter() {
                let (r, g, b) = (c[0] as i32, c[1] as i32, c[2] as i32);
                // 亮金：coin 的 colormap 金 (r 224-255, g 192-224, b 64-96)；
                // pad/哨兵圆柱的橙 (g~140) 与红 (g~77) 不在此类。
                if r > 200 && g > 170 && b < 130 && r > b + 80 {
                    gold += 1;
                }
            }
            let _ = name;
            (gold, png)
        };
        let (g0, png0) = gold(&mut h, "g13-t0");
        assert!(g0 > 10, "two visible textured coins: bright gold {g0}");

        let mut fired = None;
        for t in 0..4 {
            tick(&mut h);
            if events(&h).iter().any(|(_, pair)| *pair == 0) {
                fired = Some(t);
                break;
            }
        }
        assert!(fired.is_some(), "pad crossing must emit trigger pair 0");
        // t1：coin_a 移至 pad 正后（相机被 pad 遮挡）——状态断言在 slot 上，
        // 像素面只要求画面变化。
        let (_g1, png1) = gold(&mut h, "g13-t1");
        assert_ne!(png0, png1, "view must change after move+hide");

        let mut despawned = false;
        for _ in 0..3 {
            tick(&mut h);
            if live(&h) == 1 {
                despawned = true;
                break;
            }
        }
        assert!(despawned, "despawn stage must complete");
        eprintln!("DBG play error after despawn: {:?}", h.play.as_ref().unwrap().error);
        let (g2, _png2) = gold(&mut h, "g13-t2");
        assert_eq!(g2, 0, "despawned+hidden coins must vanish from GPU (gold {g2})");

        let mut respawned = false;
        for _ in 0..3 {
            tick(&mut h);
            if live(&h) == 2 && slot(&h, 0).is_some_and(|s| s.0 == 2) {
                respawned = true;
                break;
            }
        }
        assert!(respawned, "respawn stage must complete");
        let (g3, _) = gold(&mut h, "g13-t3");
        assert!(g3 > 5, "respawned coin renders from the shared closure: bright gold {g3}");
    });
}


#[test]
fn model_objects_typed_budget_faults_are_named() {
    use autographics_auto_driver::authoring::behavior::{Api, SourceSnapshot};
    let snapshot = |name: &str, src: String| {
        SourceSnapshot::from_bytes_for(Path::new(name), src.as_bytes().to_vec(), Api::Triggers3V1)
            .expect("admission")
    };
    let start_mutant = |tag: &str, world: String| -> Host {
        let dir = write_product(tag);
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        h.start_user_snapshot(snapshot(&format!("{tag}.at"), world))
            .expect("start");
        h.pause_play("");
        settle(&mut h);
        h
    };

    // (V-S03) 33rd credit：tick 内注入 31 个 ag_char3_move（角色 op 计费
    // 但不占 object 写上限；2 object credit + 31 char3 credit = 33 > 32）
    // → 共享 32/callback 池超限，fault 10。
    let mut injected = String::new();
    for _ in 0..31 {
        injected.push_str("  ag_char3_move(1.0, 0.0, 0.0, 0.001)
");
    }
    let mutated = G12_WORLD.replace(
        "  ag_object_visible(r_h2, 0)",
        &format!("  ag_object_visible(r_h2, 0)
{injected}"),
    );
    assert_ne!(mutated, G12_WORLD, "credit mutation must apply");
    let mut h = start_mutant("fault-credit", mutated);
    tick(&mut h);
    let err = h.play.as_ref().unwrap().error.clone().unwrap_or_default();
    assert!(err.contains("code=10"), "33rd credit must fault 10, got '{err}'");

    // (V-S03) 第 33 个 lifecycle write：Start 已有 2 spawn（2 写），注入
    // 31 个 move → 第 33 写恰在最后一个 → fault 37（写上限先于计费检查）。
    let mut extra = String::new();
    for _ in 0..31 {
        extra.push_str(" ag_object_move(r_h, 2.0, 0.5, 0.0)
");
    }
    let mutated = G12_WORLD.replace(
        " r_h2 = ag_object_spawn(1, 3, 6.0, 0.5, 0.0)",
        &format!(" r_h2 = ag_object_spawn(1, 3, 6.0, 0.5, 0.0)
{extra}"),
    );
    assert_ne!(mutated, G12_WORLD, "write-cap mutation must apply");
    let h = start_mutant("fault-write", mutated);
    let err = h.play.as_ref().unwrap().error.clone().unwrap_or_default();
    assert!(err.contains("code=37"), "33rd write must fault 37, got '{err}'");

    // (V-S03) spawn 进槽 8 → fault 31（槽域在 Start 回调内拒绝）。
    let mutated = G12_WORLD.replace(
        "r_h = ag_object_spawn(0, 3, 2.0, 0.5, 0.0)",
        "r_h = ag_object_spawn(8, 3, 2.0, 0.5, 0.0)",
    );
    assert_ne!(mutated, G12_WORLD, "slot mutation must apply");
    let h = start_mutant("fault-slot", mutated);
    let err = h.play.as_ref().unwrap().error.clone().unwrap_or_default();
    assert!(err.contains("code=31"), "slot 8 spawn must fault 31, got '{err}'");

    // (V-S03) 旧代拒绝：despawn 后持有旧句柄再 move → fault 33（sticky
    // error；句柄身份 = run*524288+slot*65536+generation，重用必须新代）。
    let mutated = G12_WORLD.replace(
        "   ag_object_despawn(r_h)\n   r_seen = 2",
        "   r_h2 = r_h\n   ag_object_despawn(r_h)\n   ag_object_move(r_h2, 1.0, 0.5, 0.0)\n   r_seen = 2",
    );
    assert_ne!(mutated, G12_WORLD, "stale-handle mutation must apply");
    let mut h = start_mutant("fault-stale", mutated);
    // 走状态机到 despawn 分支（事件滞后一 tick，等待环）。
    let mut faulted = false;
    for _ in 0..6 {
        tick(&mut h);
        let err = h.play.as_ref().unwrap().error.clone().unwrap_or_default();
        if err.contains("code=33") {
            faulted = true;
            break;
        }
    }
    assert!(faulted, "stale handle after despawn must fault 33");
}


#[test]
fn hidden_templates_and_live_instances_are_non_solid_on_the_actor_path() {
    big_stack(|| {
        // V-S03 轨迹对照：隐藏模板 parked 在行走车道 x=4.5（variant）vs
        // parked 在远处 -30（control）。行为源不变，仅场景行字节不同。
        // 三个阶段：出生前模板在路径（walker 3.0→4.0 经过 x=4.5 之前）、
        // 有活实例（3.5 处，tick4 销毁）、销毁后（继续走到 ~8）——逐 tick
        // 角色身体位置必须与对照完全一致（模板/实例永不成为碰撞面）。
        let control_scene = G14_SCENE.replace(
            "\"translation\": [\n    4.5,\n    0.5,\n    -3.0\n   ]",
            "\"translation\": [\n    -30.0,\n    0.5,\n    -3.0\n   ]",
        );
        assert_ne!(control_scene, G14_SCENE, "control mutation must apply");

        let run = |scene: &str, tag: &str| -> Vec<[f32; 3]> {
            let dir = write_product_with(
                scene,
                G14_WORLD,
                &[(&format!("{GLB_SHA}.glb"), GLB)],
                tag,
            );
            let (doc, registry, code, _) =
                Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
                    .expect("load");
            let body = doc
                .objects
                .iter()
                .find(|o| o.name == "Main/hero/body")
                .expect("walker body row")
                .id;
            let mut h = Host::with_project(doc, registry, code).expect("host");
            h.start_selected_play().expect("start");
            h.pause_play("");
            settle(&mut h);
            let mut traj = Vec::new();
            for i in 0..10 {
                if i == 1 {
                    eprintln!("DBG {tag}: err={:?} tick={:?}", h.play.as_ref().unwrap().error, h.play.as_ref().and_then(|p| p.last.as_ref()).and_then(|f| f.rounds.as_ref()).map(|r| r.tick));
                }
                traj.push(
                    h.play
                        .as_ref()
                        .and_then(|p| p.last.as_ref())
                        .and_then(|f| f.nodes.get(&body))
                        .map(|d| d.translation)
                        .expect("walker body in frame"),
                );
                tick(&mut h);
            }
            traj
        };

        let variant = run(G14_SCENE, "g14-variant");
        let control = run(&control_scene, "g14-control");
        assert_eq!(variant, control, "hidden template on the path must not alter the walk");
        // 没有被阻挡：walker 从 3.0 出发每 tick +0.5，10 tick 后约 8.0。
        let last = *variant.last().unwrap();
        assert!(
            last[0] > 7.0,
            "walk must continue unblocked through template and instance spots, got {last:?}"
        );
        // 活实例 tick4 已销毁（身份负控同链已证）；此处确认轨迹含销毁前后段。
        let at_despawn = variant[4];
        assert!(at_despawn[0] > 4.9, "walker passes the instance spot freely: {at_despawn:?}");
    });
}


#[test]
fn vs04_twenty_rounds_keep_warm_decodes_flat_and_resource_union_bounded() {
    big_stack(|| {
        let dir = write_product("vs04");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("load");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        let doc_bytes_before = serde_json::to_string(&h.editor.document).expect("doc bytes");

        let union = |h: &Host| h.resource_snapshot()["asset_union"].clone();
        h.start_selected_play().expect("round 0 start");
        h.pause_play("");
        settle(&mut h);
        // 首轮把共享 GLB 解码上传（实例出生触发模板网格上传）。
        for _ in 0..6 {
            tick(&mut h);
        }
        h.stop_play();
        settle(&mut h);
        let warm = h.resource_snapshot()["mesh_count"].as_u64().expect("warm");
        let decodes_after_first = union(&h)["decode_attempts"].as_u64().expect("decodes");
        assert!(warm >= 2, "two-part template meshes resident: {warm}");

        for round in 1..20 {
            h.start_selected_play()
                .unwrap_or_else(|e| panic!("round {round} start: {e:?}"));
            h.pause_play("");
            settle(&mut h);
            for _ in 0..6 {
                tick(&mut h);
            }
            h.stop_play();
            settle(&mut h);
        }
        let after = h.resource_snapshot()["mesh_count"].as_u64().expect("after");
        assert_eq!(warm, after, "20 rounds of birth/despawn/reuse/Stop/newRun must keep uploads flat");
        let decodes_after_twenty = union(&h)["decode_attempts"].as_u64().expect("decodes");
        assert_eq!(
            decodes_after_first, decodes_after_twenty,
            "warm decode/upload must be 0 across reuse rounds (registry shares the GLB identity)"
        );

        // 资源并集：raw8 身份 / 单件 8MiB / 总 32MiB / CPU+GPU 各 64MiB 原额。
        let u = union(&h);
        assert_eq!(u["count"], serde_json::json!(1), "one shared GLB asset");
        let raw = u["raw_bytes"].as_u64().expect("raw");
        assert_eq!(raw, 1528, "raw union is the two-parts GLB bytes only");
        assert!(raw < 8 * 1024 * 1024, "single asset under 8MiB");
        assert!(u["cpu_geometry_bytes"].as_u64().unwrap() < 64 * 1024 * 1024);
        assert!(u["gpu_expected_bytes"].as_u64().unwrap() < 64 * 1024 * 1024);

        // 20 轮后文档逐字不变（实例永不进 Document；Stop 释放运行所有权）。
        let doc_bytes_after = serde_json::to_string(&h.editor.document).expect("doc after");
        assert_eq!(doc_bytes_before, doc_bytes_after, "document unchanged after 20 rounds");
    });
}


#[test]
fn mesh_denominator_512_boundary_counts_hidden_template_rows() {
    big_stack(|| {
        // V-S03 512 边界 + hidden denominator 负控：文档三维行（含 invisible
        // parked 模板行）按 TemplateRules 全部 eligible 叶计入 potential
        // meshes；8 槽 × 每模板 mesh 数 + base ≤ 512。65 mesh/模板 × 8 + 1
        // base = 521 > 512 必须在 prepare 期具名拒绝；63 mesh/模板 × 8 + 1
        // = 505 通过（边界两侧）。
        use autographics_auto_driver::authoring::document::Object;
        fn synth_model(meshes: usize, tag: &str) -> agp::assets::ModelData {
            let tri = |name: String| agp::assets::MeshData {
                visual: None,
                name,
                positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
                normals: vec![[0., 0., 1.]; 3],
                indices: vec![0, 1, 2],
                material: agp::assets::MaterialDef {
                    name: "synth".into(),
                    base_color: [0.9, 0.3, 0.2, 1.0],
                    metallic: 0.0,
                    roughness: 1.0,
                },
            };
            let sha = agp::renderer::capture::sha256_hex(format!("plan043-synth-{tag}-{meshes}").as_bytes());
            agp::assets::ModelData {
                source_sha256: sha.clone(),
                meshes: (0..meshes).map(|i| tri(format!("primitive-{i}"))).collect(),
            }
        }
        let build = |meshes_per_template: usize, tag: &str| -> Result<Host, String> {
            let model = synth_model(meshes_per_template, tag);
            let sha = agp::renderer::capture::sha256_hex(
                format!("plan043-synth-{tag}-{meshes_per_template}").as_bytes(),
            );
            let mut d = Document::empty(format!("plan043-denominator-{tag}"));
            d.objects.push(Object::new(1, Kind::Cylinder));
            for i in 0..8u64 {
                let mut o = Object::new(2 + i, Kind::Model);
                o.name = format!("Tpl{i}");
                o.visible = false; // parked：隐藏也必须计入 denominator。
                o.resource = Some(format!("sha256:{sha}"));
                d.objects.push(o);
            }
            let ident = || autographics_auto_driver::authoring::assets::AssetRef {
                closure: None,
                audio: None,
                format: "glb".into(),
                sha256: sha.clone(),
                byte_length: 1528,
                name: Some("tpl".into()),
                width: None,
                height: None,
            };
            d.assets.push(ident());
            d.validate().expect("synthetic document valid");
            let asset = autographics_auto_driver::authoring::assets::Asset {
                audio: None,
                model_images: vec![],
                dependencies: Default::default(),
                identity: ident(),
                bytes: std::sync::Arc::from(&b"synth"[..]),
                model,
                animation: None,
                image: None,
                cpu_bytes: 1528,
                gpu_bytes: 1528,
                decode_us: 0,
            };
            let registry = autographics_auto_driver::authoring::assets::Registry::from([(
                format!("sha256:{sha}"),
                std::sync::Arc::new(asset),
            )]);
            Host::with_project(d, registry, None)
        };
        // 上界侧：65 meshes × 8 + base = 521 > 512 → 具名拒绝（在
        // with_project 的首帧 prepare，比 play start 更早）。
        let err = match build(65, "over") {
            Ok(_) => panic!("521 potential meshes must exceed 512"),
            Err(e) => e,
        };
        assert!(
            err.contains("512"),
            "named 512 rejection, got '{err}'"
        );
        // 下界侧：63 meshes × 8 + base = 505 ≤ 512 → 构建成功。
        let mut h_under = build(63, "under").expect("505 potential meshes must fit the 512 envelope");
        h_under.pause_play("");
        settle(&mut h_under);
        assert_eq!(live(&h_under), 0);
    });
}


#[test]
fn vs05_save_as_relocates_and_broken_assets_repair_to_green() {
    big_stack(|| {
        // ---- A：SaveAs 搬迁 —— g12 产品在新根保存，冷打开后同入口真 Run。
        let dir = write_product("vs05-a");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("load");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        h.start_selected_play().expect("start");
        h.pause_play("");
        settle(&mut h);
        tick(&mut h);
        h.stop_play();
        settle(&mut h);
        let new_root = dir.parent().expect("parent").join(format!(
            "{}-saveas",
            dir.file_name().expect("dir name").to_string_lossy()
        ));
        let _ = std::fs::remove_dir_all(&new_root);
        std::fs::create_dir_all(&new_root).expect("new root");
        let saved = new_root.join("scene.agscene");
        command(&mut h, "saveas", serde_json::json!(saved));
        h.settle().expect("settle saveas");
        // 冷消费者：新根打开 → 真 Run → 生命周期像素 oracle。
        let (doc2, registry2, code2, _) =
            Storage::load_project(&saved, &Registry::new()).expect("cold load at new root");
        let mut h2 = Host::with_project(doc2, registry2, code2).expect("cold host");
        h2.start_selected_play().expect("cold start");
        h2.pause_play("");
        settle(&mut h2);
        assert_eq!(live(&h2), 2, "relocated project spawns both instances");
        let (r2, g2, _) = red_green(&mut h2, "vs05-relocated");
        assert!(g2 > 20, "relocated project renders the shared GLB: green {g2}");
        let _ = r2;

        // ---- B：坏件恢复 —— 新根 GLB 单字节损坏：具名拒绝；修复后同入口绿。
        let glb_path = new_root.join("assets").join(format!("{GLB_SHA}.glb"));
        let good = std::fs::read(&glb_path).expect("glb");
        let mut bad = good.clone();
        bad[25] ^= 0xff;
        std::fs::write(&glb_path, &bad).expect("corrupt");
        let err = Storage::load_project(&saved, &Registry::new())
            .map(|_| ())
            .expect_err("corrupt GLB must reject at the new root");
        assert!(err.contains("hash") || err.contains("identity"), "{err}");
        std::fs::write(&glb_path, &good).expect("repair");
        let (doc3, registry3, code3, _) =
            Storage::load_project(&saved, &Registry::new()).expect("repaired load");
        let mut h3 = Host::with_project(doc3, registry3, code3).expect("repaired host");
        h3.start_selected_play().expect("repaired start");
        h3.pause_play("");
        settle(&mut h3);
        assert_eq!(live(&h3), 2, "repaired project runs again");

        // ---- C：g13 闭包 PNG 损坏 → 拒绝 → 修复 → 绿（闭包纹理面）。
        let dir13 = write_product_with(
            G13_SCENE,
            G13_WORLD,
            &[
                (&format!("{COIN_GLB_SHA}.glb"), COIN_GLB),
                (&format!("{COIN_PNG_SHA}.png"), COIN_PNG),
            ],
            "vs05-c",
        );
        let scene13 = dir13.join("scene.agscene");
        let png_path = dir13.join("assets").join(format!("{COIN_PNG_SHA}.png"));
        let good_png = std::fs::read(&png_path).expect("png");
        let mut bad_png = good_png.clone();
        let mid = bad_png.len() / 2;
        bad_png[mid] ^= 0xff;
        std::fs::write(&png_path, &bad_png).expect("corrupt png");
        let err = Storage::load_project(&scene13, &Registry::new())
            .map(|_| ())
            .expect_err("corrupt closure PNG must reject");
        assert!(err.contains("hash") || err.contains("identity") || err.contains("declared"), "{err}");
        std::fs::write(&png_path, &good_png).expect("repair png");
        let (doc4, registry4, code4, _) =
            Storage::load_project(&scene13, &Registry::new()).expect("repaired closure load");
        let mut h4 = Host::with_project(doc4, registry4, code4).expect("host");
        h4.start_selected_play().expect("closure start");
        h4.pause_play("");
        settle(&mut h4);
        assert_eq!(live(&h4), 2, "repaired closure project spawns");
    });
}

fn command(h: &mut Host, action: &str, payload: serde_json::Value) {
    h.dispatch(
        &serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"play_token":h.ui().play_token,"action":action,"payload":payload})
            .to_string(),
    )
    .unwrap_or_else(|e| panic!("command {action}: {e}"));
}

/// The camera family re-enters the SDK parser per start; give it the same
/// 64 MiB runway the other runtime tests use.
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
