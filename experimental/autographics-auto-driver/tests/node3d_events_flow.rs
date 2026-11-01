//! PLAN-028 components-3d-events flow（V-S02/V-S03/V-S04/V-S05；AC-02/03/04/05）。
//! 原 G11/3d 25 项 oracle 逐名经真实生成链（pilot components-3d-events
//! 产品 -> Storage::load_project -> Host -> Triggers3V1 装配 -> GPU）复刻：
//! 三模板/Start 高度、六实例六 pair、相机相对运动、三收集胜、alt 失败、
//! 球切触/corner、同 tick 三件、旧代/显式重绑、accepted 新局、stale 拒绝。
//! 另加 V-S03 typed/预算负控（双 move 33 计费 fault 10、33 lifecycle write
//! fault 37、槽 8 fault 31、tick0 重开 fault 41）、sensor 非游戏路由与第二
//! 有限读取策略、V-S05 Stop 恢复与 20 轮暖账。GPU 单实例：--test-threads=1。
//! 运行：cargo test --locked --offline --test node3d_events_flow -- --test-threads=1
use autographics_auto_driver::authoring::document::{Document, Kind};
use autographics_auto_driver::authoring::{
    assets::Registry,
    behavior::SourceSnapshot,
    host::Host,
    play::{Action, CharProfile},
    storage::Storage,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

const MAIN_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-main/scene.agscene"
);
const MAIN_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-main/world.at"
);
const ALT_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-alt/scene.agscene"
);
const ALT_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-alt/world.at"
);
const TRIPLE_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-triple/scene.agscene"
);
const TRIPLE_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-triple/world.at"
);
const GEO_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-geometry/scene.agscene"
);
const GEO_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-geometry/world.at"
);
const OLDGEN_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-oldgen/scene.agscene"
);
const OLDGEN_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-oldgen/world.at"
);
const STALE_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-stale/scene.agscene"
);
const STALE_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/g11-stale/world.at"
);
const SENSOR_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/sensor/scene.agscene"
);
const SENSOR_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-events/sensor/world.at"
);

const WALK_BUDGET: usize = 8000;

// pair 池声明序（typed 名字 -> 运行 pair）：shard_a=0 shard_b=1 shard_c=2
// hazard=3 goal=4 voidp=5；sensor: sense_a=0 sense_b=1；geometry:
// corner_aabb=0 corner_sphere=1 tangent_sphere=2；oldgen: shard_a=0。

// ---------------------------------------------------------------- harness --

fn write_project(scene_text: &str, world_src: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan028-events")
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ev {
    tick: i32,
    pair: i32,
    phase: i32,
    source_ref: i32,
    bind_generation: i32,
}

fn events(h: &Host) -> Vec<Ev> {
    let p = h.play.as_ref().unwrap();
    let f = p.last.as_ref().expect("frame");
    let tick = f.rounds.as_ref().map(|r| r.tick).unwrap_or_default();
    f.rounds
        .as_ref()
        .map(|r| {
            r.events
                .iter()
                .map(|e| Ev {
                    tick,
                    pair: e.pair,
                    phase: e.phase,
                    source_ref: e.source_ref,
                    bind_generation: e.bind_generation,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn live(h: &Host) -> usize {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.instances.as_ref())
        .map(|i| i.live_count())
        .unwrap_or(0)
}

fn slot(h: &Host, i: usize) -> Option<(i32, u64, [f32; 3])> {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.instances.as_ref())
        .and_then(|inst| inst.get(i))
        .map(|s| (s.generation, s.template_id, s.translation))
}

fn bindings(h: &Host) -> usize {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.rounds.as_ref())
        .map(|r| r.bindings.iter().flatten().count())
        .unwrap_or(0)
}

fn play_error(h: &Host) -> String {
    h.play.as_ref().unwrap().error.clone().unwrap_or_default()
}

fn frame_tick(h: &Host) -> i32 {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.rounds.as_ref())
        .map(|r| r.tick)
        .unwrap_or_default()
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

fn row_id(doc: &Document, name: &str) -> u64 {
    doc.objects
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("row {name}"))
        .id
}

fn snap_png(h: &mut Host, name: &str) -> (u32, u32, Vec<u8>) {
    let (_, png, w, ht) = h.rt.capture(h.three, name).expect("capture");
    (w, ht, png)
}

fn restart_once(h: &mut Host) -> bool {
    // 重试入口可能是 Running（上一轮 marker 未消费即再试）；先归位 Paused
    // 再注入 fresh Enter 边沿。
    h.pause_play("");
    settle(h);
    {
        let p = h.play.as_mut().expect("play");
        p.resume().expect("resume for restart");
        // fresh Enter 边沿：仅按下（与原 harness restart_once 逐字同型；
        // 释放会清掉 pending 边沿，marker 不再置位）。
        p.char_key(5, true, false);
        p.dispatch(Action::default()).expect("restart dispatch");
    }
    let requested = h
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .and_then(|f| f.rounds.as_ref())
        .map(|r| r.restart_requested)
        .unwrap_or(false);
    settle(h);
    requested
}

fn wait_new_run(h: &mut Host, old_run: i32, rounds: usize) -> bool {
    for i in 0..rounds {
        if h.play.as_ref().map(|p| p.run_id).unwrap_or(old_run) != old_run {
            settle(h);
            return true;
        }
        let _ = i;
        h.frame(None).expect("frame pump");
    }
    settle(h);
    h.play.as_ref().map(|p| p.run_id).unwrap_or(old_run) != old_run
}

/// Independent closest-distance oracle for the actor box (half extents
/// 0.4/0.5/0.4) against a point — the same math the gallery harness uses.
fn closest_distance_3d(actor: [f32; 3], half: [f32; 3], p: [f32; 3]) -> f32 {
    let dx = (p[0] - actor[0]).abs() - half[0];
    let dy = (p[1] - actor[1]).abs() - half[1];
    let dz = (p[2] - actor[2]).abs() - half[2];
    let clamp = |v: f32| if v > 0.0 { v } else { 0.0 };
    (clamp(dx) * clamp(dx) + clamp(dy) * clamp(dy) + clamp(dz) * clamp(dz)).sqrt()
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

macro_rules! oracle {
    ($name:literal, $cond:expr, $($arg:tt)*) => {
        assert!($cond, "oracle {}: {}", $name, format!($($arg)*));
    };
}

// ------------------------------------------------------------------ flow --

#[test]
fn g11_3d_oracles_on_the_typed_authoring_chain() {
    big_stack(|| {
        // ---- import / scene identity (1, 2) ----
        let dir = write_project(MAIN_SCENE, MAIN_WORLD, "identity");
        let (doc, _registry, _code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        oracle!(
            "import_three_templates",
            doc.objects.len() == 8
                && doc
                    .objects
                    .iter()
                    .map(|o| o.kind)
                    .collect::<Vec<_>>()
                    == vec![
                        Kind::Cylinder,
                        Kind::Cylinder,
                        Kind::Mob,
                        Kind::Mob,
                        Kind::Mob,
                        Kind::Cylinder,
                        Kind::Cylinder,
                        Kind::Cylinder
                    ],
            "objects {:?}",
            doc.objects.iter().map(|o| (o.name.clone(), o.kind)).collect::<Vec<_>>()
        );
        let shard_scales: Vec<[f32; 3]> = doc.objects[2..5].iter().map(|o| o.scale).collect();
        oracle!(
            "three_distinct_shard_templates",
            shard_scales[0] != shard_scales[1]
                && shard_scales[1] != shard_scales[2]
                && shard_scales[0] != shard_scales[2]
                && shard_scales.iter().all(|s| (s[0] - s[1]).abs() < 1e-6),
            "scales {shard_scales:?}"
        );

        // ---- Start surface (3-6): tick0 registry, camera, no fault ----
        let hero_id = row_id(&doc, "Main/hero/body");
        let mut h = start(MAIN_SCENE, MAIN_WORLD, "main-run");
        oracle!("start_without_fault", play_error(&h).is_empty(), "{}", play_error(&h));
        oracle!("tick0_six_pairs_bound", bindings(&h) == 6, "{}", bindings(&h));
        oracle!("tick0_six_instances", live(&h) == 6, "{}", live(&h));
        {
            let cam3 = h.play.as_ref().unwrap().camera_frame().expect("tick0 camera3d").clone();
            oracle!(
                "orbit_camera_from_source",
                cam3.yaw == 0.0
                    && (cam3.pitch - 45.0).abs() < 1e-4
                    && (cam3.distance - 8.0).abs() < 1e-4
                    && cam3.projection == 0,
                "{cam3:?}"
            );
        }

        // ---- camera-relative walk collects three and wins (7, 8) ----
        let mut all_events: Vec<Ev> = events(&h);
        let mut won = false;
        for i in 0..WALK_BUDGET {
            step(&mut h, 2, 0).unwrap_or_else(|e| panic!("walk step {i}: {e}"));
            all_events.extend(events(&h));
            if play_error(&h).is_empty()
                && live(&h) == 3
                && all_events.iter().any(|e| e.pair == 4 && e.phase == 1)
            {
                won = true;
                break;
            }
        }
        let enters: Vec<&Ev> = all_events
            .iter()
            .filter(|e| e.pair < 3 && e.phase == 1)
            .collect();
        oracle!(
            "walk_collects_three_and_wins",
            won,
            "live={} error={} shard_enters={} pos={:?}",
            live(&h),
            play_error(&h),
            enters.len(),
            actor(&h, hero_id)
        );
        let pairs: BTreeSet<i32> = enters.iter().map(|e| e.pair).collect();
        let refs: BTreeSet<i32> = enters.iter().map(|e| e.source_ref).collect();
        oracle!(
            "three_distinct_ids_each_once",
            enters.len() == 3
                && pairs.contains(&0)
                && pairs.contains(&1)
                && pairs.contains(&2)
                && refs.len() == 3,
            "enters={enters:?}"
        );
        // V-S03 补充（§6 另列）：despawn 撤销资格——被收集碎片不再产生
        // stay/exit（销毁发生在 enter 后同一回调，资格随即消失）。
        let shard_exits = all_events
            .iter()
            .filter(|e| e.pair < 3 && (e.phase == 2 || e.phase == 3))
            .count();
        assert_eq!(shard_exits, 0, "despawned shards must not emit stay/exit");

        // ---- windowless GPU capture of the won state (9) ----
        let (w, ht, png) = snap_png(&mut h, "g11-3d-events-won");
        oracle!(
            "windowless_png_three",
            w > 0 && ht > 0 && png.len() > 3_000,
            "{w}x{ht} {} bytes",
            png.len()
        );

        // ---- orbit drag changes pixels; walk rotates with the camera (14-16) ----
        // 与原 oracle 序一致：won host 停止，运动学 oracle 在新的 Playing
        // host 上复刻（won 冻结移动，冻结面上无位移可观测）。
        drop(h);
        let mut h = start(MAIN_SCENE, MAIN_WORLD, "orbit-run");
        let (_, _, png0) = snap_png(&mut h, "g11-3d-events-orbit-0");
        {
            let p = h.play.as_mut().unwrap();
            p.resume().expect("resume for drag");
            p.cam_drag(90.0, 0.0);
            p.dispatch(Action::default()).expect("drag dispatch");
        }
        h.pause_play("");
        settle(&mut h);
        let (_, _, png90) = snap_png(&mut h, "g11-3d-events-orbit-90");
        oracle!("orbit_changes_pixels", png0 != png90, "orbit drag must change the view");
        {
            let cam90 = h.play.as_ref().unwrap().camera_frame().expect("camera").clone();
            let p0 = actor(&h, hero_id);
            step(&mut h, 2, 0).expect("post-orbit step");
            let p1 = actor(&h, hero_id);
            oracle!(
                "camera_relative_walk_rotates_world_direction",
                (cam90.yaw - 90.0).abs() < 1e-2
                    && (p1[0] - p0[0]).abs() < 1e-2
                    && (p1[2] - p0[2]).abs() > 0.02,
                "cam={:?} p0={p0:?} p1={p1:?}",
                cam90
            );
        }
        let mut oob = false;
        for i in 0..WALK_BUDGET {
            step(&mut h, 2, 0).unwrap_or_else(|e| panic!("void walk step {i}: {e}"));
            if events(&h).iter().any(|e| e.pair == 5 && e.phase == 1) {
                oob = true;
                break;
            }
        }
        oracle!(
            "camera_relative_walk_reaches_the_void_zone",
            oob && actor(&h, hero_id)[2] < -6.0,
            "z={:?}",
            actor(&h, hero_id)
        );

        // ---- accepted restart on the failed state (12, 13) ----
        // Drive the alt program to the hazard failure first: real alt chain.
        let mut ha = start(ALT_SCENE, ALT_WORLD, "alt-run");
        let mut hz_tick = 0;
        let mut hz_x = 0.0f32;
        for i in 0..WALK_BUDGET {
            step(&mut ha, 2, 0).unwrap_or_else(|e| panic!("alt walk step {i}: {e}"));
            let evs = events(&ha);
            if evs.iter().any(|e| e.pair == 3 && e.phase == 1) {
                hz_tick = frame_tick(&ha);
                hz_x = actor(&ha, hero_id)[0];
                break;
            }
        }
        oracle!(
            "alt_fails_at_hazard",
            (7.0..9.6).contains(&hz_x),
            "first saw enter at tick {hz_tick} x={hz_x} (sphere 0.5 on the line)"
        );
        // freeze on failure: two more steps reach the frozen state, then the
        // position must hold against further right-held steps.
        step(&mut ha, 2, 0).expect("alt freeze step");
        step(&mut ha, 2, 0).expect("alt freeze step 2");
        let frozen = actor(&ha, hero_id);
        for _ in 0..10 {
            step(&mut ha, 2, 0).expect("alt frozen step");
        }
        oracle!("failed_state_freezes_motion", actor(&ha, hero_id) == frozen, "{:?} vs {:?}", actor(&ha, hero_id), frozen);

        // restart: fresh Enter edge opens a new run at tick 0.
        let old_run = ha.play.as_ref().unwrap().run_id;
        let mut swapped = false;
        for _ in 0..4 {
            restart_once(&mut ha);
            if wait_new_run(&mut ha, old_run, 60) {
                swapped = true;
                break;
            }
        }
        oracle!("restart_new_run_tick0", swapped && frame_tick(&ha) == 0, "swapped={swapped} tick={}", frame_tick(&ha));
        // 新 Run 在 Running（marker 在泵中已消费）；归位 Paused 再走一步。
        ha.pause_play("");
        settle(&mut ha);
        step(&mut ha, 0, 0).expect("new run settle step");
        oracle!(
            "restart_rebirths_items",
            live(&ha) == 6
                && (0..3).all(|s| slot(&ha, s).map(|(g, _, _)| g) == Some(1))
                && (0..3).all(|s| slot(&ha, s).map(|(_, t, _)| t) == Some(3 + s as u64)),
            "live={} slots {:?}",
            live(&ha),
            (0..3).map(|s| slot(&ha, s)).collect::<Vec<_>>()
        );
    });
}

#[test]
fn g11_3d_geometry_sphere_aabb_oracles() {
    big_stack(|| {
        // (17-19) corner outside / tangency / AABB distinguishability.
        let gdir = write_project(GEO_SCENE, GEO_WORLD, "geo-identity");
        let (gdoc, _, _, _) =
            Storage::load_project(&gdir.join("scene.agscene"), &Registry::new()).expect("geo doc");
        let hero_id = row_id(&gdoc, "Main/hero/body");
        let mut h = start(GEO_SCENE, GEO_WORLD, "geo-run");
        let mut seen: Vec<Ev> = Vec::new();
        for _ in 0..6 {
            step(&mut h, 0, 0).expect("geo step");
            seen.extend(events(&h));
        }
        let pos = actor(&h, hero_id);
        let corner = [pos[0] + 0.9, pos[1] + 0.9, pos[2]];
        let tangent = [pos[0] + 0.9, pos[1] + 0.5, pos[2]];
        let half = [0.4f32, 0.5, 0.4];
        let d_corner = closest_distance_3d(pos, half, corner);
        let d_tangent = closest_distance_3d(pos, half, tangent);
        oracle!(
            "sphere_corner_outside_does_not_trigger",
            !seen.iter().any(|e| e.pair == 1) && d_corner > 0.5,
            "pair1 enters={:?} d_corner={d_corner}",
            seen.iter().filter(|e| e.pair == 1).collect::<Vec<_>>()
        );
        oracle!(
            "sphere_tangent_hits",
            seen.iter().any(|e| e.pair == 2 && e.phase == 1) && (d_tangent - 0.5).abs() < 0.01,
            "pair2 enters={:?} d_tangent={d_tangent}",
            seen.iter().filter(|e| e.pair == 2).collect::<Vec<_>>()
        );
        oracle!(
            "aabb_and_sphere_are_distinguishable",
            seen.iter().any(|e| e.pair == 0 && e.phase == 1),
            "corner AABB pair 0 must enter while its sphere pair 1 stays silent"
        );
    });
}

#[test]
fn g11_3d_same_tick_three_oracles() {
    big_stack(|| {
        // (20, 21) one tick emits a three-enter block; the author ladder
        // collects each exactly once (live 6 -> 3).
        let mut h = start(TRIPLE_SCENE, TRIPLE_WORLD, "triple-run");
        let mut t3: Vec<Ev> = Vec::new();
        for _ in 0..8 {
            step(&mut h, 0, 0).expect("triple step");
            let evs = events(&h);
            let enters: Vec<Ev> = evs.iter().filter(|e| e.phase == 1).copied().collect();
            if enters.len() >= 3 {
                t3 = enters;
                break;
            }
        }
        let t3_pairs: BTreeSet<i32> = t3.iter().map(|e| e.pair).collect();
        oracle!(
            "same_tick_three_engine_block",
            t3.len() == 3
                && t3_pairs.contains(&0)
                && t3_pairs.contains(&1)
                && t3_pairs.contains(&2),
            "block {t3:?}"
        );
        let before = live(&h);
        step(&mut h, 0, 0).expect("triple collect step");
        oracle!(
            "same_tick_three_counted_once_each",
            before == 6 && live(&h) == 3,
            "before={before} after={}",
            live(&h)
        );
    });
}

#[test]
fn g11_3d_generation_lifecycle_oracles() {
    big_stack(|| {
        // (22-24) same-slot respawn is never auto-bound; explicit rebind
        // re-enters; the slot carries the new generation identity.
        let mut h = start(OLDGEN_SCENE, OLDGEN_WORLD, "oldgen-run");
        let mut seen: Vec<Ev> = Vec::new();
        let mut gens: Vec<i32> = Vec::new();
        for _ in 0..12 {
            seen.extend(events(&h));
            if let Some((g, _, _)) = slot(&h, 0) {
                gens.push(g);
            }
            step(&mut h, 0, 0).expect("oldgen step");
        }
        let enter_ticks: Vec<i32> = seen
            .iter()
            .filter(|e| e.pair == 0 && e.phase == 1)
            .map(|e| e.tick)
            .collect();
        oracle!(
            "old_generation_never_auto_reattaches",
            enter_ticks.first() == Some(&1)
                && !seen
                    .iter()
                    .any(|e| e.pair == 0 && e.phase == 1 && e.tick > 1 && e.tick < 8),
            "pair0 enter ticks {enter_ticks:?}"
        );
        oracle!(
            "explicit_rebind_enters_new_generation",
            enter_ticks.len() == 2 && *enter_ticks.last().unwrap() >= 8,
            "enter ticks {enter_ticks:?}"
        );
        oracle!(
            "same_slot_new_generation_identity",
            gens.iter().any(|g| *g == 2),
            "slot0 generations {gens:?}"
        );

        // (25) the previous generation's handle cannot operate the new
        // instance: ObjectNotAlive faults the callback (play.error).
        let mut hs = start(STALE_SCENE, STALE_WORLD, "stale-run");
        let mut err = String::new();
        for _ in 0..8 {
            step(&mut hs, 0, 0).expect("stale step");
            err = play_error(&hs);
            if !err.is_empty() {
                break;
            }
        }
        oracle!(
            "stale_handle_operation_is_rejected",
            !err.is_empty() && (err.contains("code=33") || err.contains("33")),
            "error '{err}'"
        );
    });
}

#[test]
fn sensor_flow_routes_msgs_and_uses_a_second_finite_policy() {
    big_stack(|| {
        // Non-game sensing: visitor sweep, static-leaf subjects, msg down to
        // two same-def markers and back up; read policy = up to two events
        // per tick regardless of movement (distinct from the g11 ladder).
        let mut h = start(SENSOR_SCENE, SENSOR_WORLD, "sensor-run");
        let mut confirmed_a = 0;
        let mut confirmed_b = 0;
        for i in 0..4000 {
            step(&mut h, 0, 0).unwrap_or_else(|e| panic!("sensor step {i}: {e}"));
            // 逐 tick 观测 pair 0/1 的 enter（每 pad 恰一次 ping 上报）。
            let evs = events(&h);
            if evs.iter().any(|e| e.pair == 0 && e.phase == 1) {
                confirmed_a += 1;
            }
            if evs.iter().any(|e| e.pair == 1 && e.phase == 1) {
                confirmed_b += 1;
            }
            if confirmed_a >= 1 && confirmed_b >= 1 {
                break;
            }
        }
        oracle!(
            "sensor_sweeps_both_pads_and_routes",
            confirmed_a >= 1 && confirmed_b >= 1 && play_error(&h).is_empty(),
            "a={confirmed_a} b={confirmed_b} error={}",
            play_error(&h)
        );
        // Isolation: two same-def Marker instances keep independent SoA state
        // (marker_pings has two distinct slots; the receiver tables route by
        // instance), and the second marker instance's ping never mutates the
        // first instance's counter — visible in the generated SoA layout.
        let world = SENSOR_WORLD;
        assert!(world.contains("var marker_pings=[0,0]"), "two isolated marker slots");
        assert!(world.contains("fn m_a_ping(j int, n int)"), "receiver per instance");
        assert!(world.contains("fn m_b_ping(j int, n int)"), "receiver per instance");
        // Second finite policy: the sensor ladder reads up to TWO events per
        // tick unconditionally (vs g11's move-gated 1-or-3).
        assert!(
            world.matches("ag_event_next()").count() >= 2
                && !world.contains("ag_action_held"),
            "sensor policy is movement-independent"
        );
    });
}

#[test]
fn typed_budget_faults_are_named_on_the_new_chain() {
    big_stack(|| {
        // (V-S03) 33rd credit on a walking tick -> error 10: duplicate the
        // character move (26+26 > 32) in the committed product bytes.
        let mutated = MAIN_WORLD.replace(
            "   ag_char3_move(",
            "   ag_char3_move(1.0, 0.0, 0.0, 0.002);\n   ag_char3_move(",
        );
        assert_ne!(mutated, MAIN_WORLD, "mutation must apply");
        let dir = write_project(MAIN_SCENE, MAIN_WORLD, "fault-credit");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        let snapshot = SourceSnapshot::from_bytes_for(
            Path::new("two-moves.at"),
            mutated.as_bytes().to_vec(),
            autographics_auto_driver::authoring::behavior::Api::Triggers3V1,
        )
        .expect("admission");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        h.start_user_snapshot(snapshot).expect("start");
        h.pause_play("");
        settle(&mut h);
        step(&mut h, 2, 0).expect("walk into the fault");
        let err = play_error(&h);
        assert!(err.contains("code=10"), "33rd credit must fault 10, got '{err}'");

        // (V-S03) 33rd lifecycle write in Start -> error 37 (write cap is
        // checked before the credit charge): 3 place writes + 1 spawn + 29
        // extra object writes = write #33 exactly on the last extra move
        // (the bind calls are trigger ops, not object lifecycle writes).
        let mut extra = String::new();
        for _ in 0..29 {
            extra.push_str(" ag_object_move(character_h0[i], 8.0, 1.0, (-3.0))\n");
        }
        let mutated = MAIN_WORLD.replace(
            " character_h0[i] = ag_object_spawn(0, 3, 8.0, 1.0, (-3.0))\n",
            &format!(" character_h0[i] = ag_object_spawn(0, 3, 8.0, 1.0, (-3.0))\n{extra}"),
        );
        assert_ne!(mutated, MAIN_WORLD, "write-cap mutation must apply");
        let dir = write_project(MAIN_SCENE, MAIN_WORLD, "fault-write");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        let snapshot = SourceSnapshot::from_bytes_for(
            Path::new("write-cap.at"),
            mutated.as_bytes().to_vec(),
            autographics_auto_driver::authoring::behavior::Api::Triggers3V1,
        )
        .expect("admission");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        h.start_user_snapshot(snapshot).expect("start");
        h.pause_play("");
        settle(&mut h);
        // 该 fault 在 Start 回调内触发（回调失败即记录），无需再 step。
        let err = play_error(&h);
        assert!(err.contains("code=37"), "33rd write must fault 37, got '{err}'");

        // (V-S03) spawn into slot 8 -> error 31 (slot domain) at Start.
        let mutated = MAIN_WORLD.replace(
            "ag_object_spawn(5, 8, 3.0, 1.0, (-8.0))",
            "ag_object_spawn(8, 8, 3.0, 1.0, (-8.0))",
        );
        assert_ne!(mutated, MAIN_WORLD, "slot mutation must apply");
        let dir = write_project(MAIN_SCENE, MAIN_WORLD, "fault-slot");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        let snapshot = SourceSnapshot::from_bytes_for(
            Path::new("slot-nine.at"),
            mutated.as_bytes().to_vec(),
            autographics_auto_driver::authoring::behavior::Api::Triggers3V1,
        )
        .expect("admission");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        h.start_user_snapshot(snapshot).expect("start");
        h.pause_play("");
        settle(&mut h);
        // 该 fault 在 Start 回调内触发（回调失败即记录），无需再 step。
        let err = play_error(&h);
        assert!(err.contains("code=31"), "slot 8 must fault 31, got '{err}'");

        // (V-S03/V-S05) round.request at tick 0 -> error 41 (rejected).
        let mutated = MAIN_WORLD.replace(
            "fn character_lifestart(i int) {\n",
            "fn character_lifestart(i int) {\n ag_round_request();\n",
        );
        assert_ne!(mutated, MAIN_WORLD, "tick0-request mutation must apply");
        let dir = write_project(MAIN_SCENE, MAIN_WORLD, "fault-tick0");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        let snapshot = SourceSnapshot::from_bytes_for(
            Path::new("tick0-request.at"),
            mutated.as_bytes().to_vec(),
            autographics_auto_driver::authoring::behavior::Api::Triggers3V1,
        )
        .expect("admission");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        h.start_user_snapshot(snapshot).expect("start");
        h.pause_play("");
        settle(&mut h);
        // 该 fault 在 Start 回调内触发（回调失败即记录），无需再 step。
        let err = play_error(&h);
        assert!(err.contains("code=41"), "tick0 request must fault 41, got '{err}'");
    });
}

#[test]
fn hidden_instance_still_triggers_and_collects() {
    big_stack(|| {
        // V-S03 补充（§6 另列）：隐藏仍触发——Start 隐藏 Beacon（可见性写
        // 不撤销触发资格），三收集 + 目标进入照常胜。
        let mutated = MAIN_WORLD.replace(
            " character_h4[i] = ag_object_spawn(4, 7, 16.0, 1.0, (-3.0))
",
            " character_h4[i] = ag_object_spawn(4, 7, 16.0, 1.0, (-3.0))
 ag_object_visible(character_h4[i], 0)
",
        );
        assert_ne!(mutated, MAIN_WORLD, "hide mutation must apply");
        let dir = write_project(MAIN_SCENE, MAIN_WORLD, "hidden-run");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        let snapshot = SourceSnapshot::from_bytes_for(
            Path::new("hidden-beacon.at"),
            mutated.as_bytes().to_vec(),
            autographics_auto_driver::authoring::behavior::Api::Triggers3V1,
        )
        .expect("admission");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        h.start_user_snapshot(snapshot).expect("start");
        h.pause_play("");
        settle(&mut h);
        let mut won = false;
        for i in 0..WALK_BUDGET {
            step(&mut h, 2, 0).unwrap_or_else(|e| panic!("hidden walk step {i}: {e}"));
            let evs = events(&h);
            if play_error(&h).is_empty()
                && live(&h) == 3
                && evs.iter().any(|e| e.pair == 4 && e.phase == 1)
            {
                won = true;
                break;
            }
        }
        assert!(won, "hidden beacon must still trigger (visible write keeps eligibility)");
    });
}

#[test]
fn vs05_stop_restores_document_and_20_rounds_keep_warm_uploads_flat() {
    big_stack(|| {
        let dir = write_project(MAIN_SCENE, MAIN_WORLD, "vs05");
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("doc");
        let mut h = Host::with_project(doc, registry, code).expect("host");
        let doc_bytes_before = serde_json::to_string(&h.editor.document).expect("doc bytes");
        let warm = h.resource_snapshot()["mesh_count"].as_u64().expect("warm");
        for round in 0..20 {
            h.start_selected_play()
                .unwrap_or_else(|e| panic!("round {round} start: {e:?}"));
            h.pause_play("");
            settle(&mut h);
            step(&mut h, 2, 0).unwrap_or_else(|e| panic!("round {round} step: {e}"));
            h.stop_play();
        }
        let after = h.resource_snapshot()["mesh_count"].as_u64().expect("after");
        assert_eq!(warm, after, "20-round lifecycle must keep mesh uploads flat");
        // Stop 释放运行所有权：文档字节在 Stop 后逐字一致（no round leak）。
        let doc_bytes_after = serde_json::to_string(&h.editor.document).expect("doc bytes after");
        assert_eq!(doc_bytes_before, doc_bytes_after, "document must be unchanged after stop");
    });
}
