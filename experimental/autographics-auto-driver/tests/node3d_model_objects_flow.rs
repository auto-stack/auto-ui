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
use std::path::PathBuf;
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

// ---------------------------------------------------------------- harness --

fn write_product(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan043-model-objects")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let assets = dir.join("assets");
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&assets).expect("assets dir");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), G12_SCENE).expect("write scene");
    std::fs::write(assets.join(format!("{GLB_SHA}.glb")), GLB).expect("write glb");
    let sha = G12_SCENE
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene binds a source sha")
        .to_string();
    std::fs::write(scripts.join(format!("{sha}.at")), G12_WORLD).expect("write script");
    dir
}

fn start(tag: &str) -> (Host, PathBuf) {
    let dir = write_product(tag);
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
