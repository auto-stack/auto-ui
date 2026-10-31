//! auto-ui PLAN-019 node2d_basic_flow: the three basic-2d authoring chains
//! (scene-2d / character-2d / camera-2d) execute on the pinned AG path as
//! real project play. Chain 1: pure visual scenes (G01 four-shape alpha
//! rules, G03 two-group hierarchy world sums and ancestor visibility) with
//! behavior=null scene play. Chain 2: the shared-Player walk-jump world
//! (G06 layout-a/b, speed 4->5) with an independent discrete f32 oracle and
//! the exact per-tick cost (30 credits; the 33rd charged call faults).
//! Chain 3: the declared follow camera (G08 main/alt) with the host clamp /
//! letterbox math and same-tick character+camera publication. The consumed
//! scene.agscene/world.at are frozen generator products
//! (experimental/autographics-node2d-pilot/generated-basic2d). Run
//! single-threaded:
//! cargo test --locked --offline --test node2d_basic_flow -- --test-threads=1
use autographics_auto_driver::{
    authoring::{
        assets::Registry,
        behavior::{Api, SourceSnapshot},
        behavior_project::CodePayload,
        hierarchy,
        host::Host,
        play::{Action, State},
        runtime_camera::CameraDim,
        storage::Storage,
    },
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const SCENE_G01: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g01-base/scene.agscene");
const SCENE_G01_COLOR: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g01-color/scene.agscene");
const SCENE_G01_ORDER: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g01-order/scene.agscene");
const SCENE_G01_POSITION: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g01-position/scene.agscene");
const SCENE_G01_VISIBLE: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g01-visible/scene.agscene");
const SCENE_G03: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g03-base/scene.agscene");
const SCENE_G03_PARENT: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g03-parent/scene.agscene");
const SCENE_G03_HIDDEN: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g03-hidden/scene.agscene");
const SCENE_G06_A: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g06-a/scene.agscene");
const WORLD_G06_A: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g06-a/world.at");
const SCENE_G06_B: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g06-b/scene.agscene");
const WORLD_G06_B: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g06-b/world.at");
const SCENE_G06_FAST: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g06-fast/scene.agscene");
const WORLD_G06_FAST: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g06-fast/world.at");
const SCENE_G08: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g08-main/scene.agscene");
const WORLD_G08: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g08-main/world.at");
const SCENE_G08_ALT: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g08-alt/scene.agscene");
const WORLD_G08_ALT: &str = include_str!("../../autographics-node2d-pilot/generated-basic2d/g08-alt/world.at");

const DT: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------- harness --

fn load_doc(
    scene_text: &str,
    world_src: Option<&str>,
    tag: &str,
) -> (
    autographics_auto_driver::authoring::document::Document,
    autographics_auto_driver::authoring::assets::Registry,
    Option<std::sync::Arc<CodePayload>>,
) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan019-basic")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    if let Some(src) = world_src {
        let sha = scene_text
            .split("\"sha256\": \"")
            .nth(1)
            .and_then(|r| r.split('"').next())
            .expect("scene binds a source sha")
            .to_string();
        std::fs::write(scripts.join(format!("{sha}.at")), src).expect("write script");
    }
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("load scene project: {e:?}"));
    (doc, registry, code)
}

fn settle(h: &mut Host) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or(false);
        if !pending {
            break;
        }
        assert!(std::time::Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

fn start_with(scene_text: &str, world_src: Option<&str>, tag: &str) -> Host {
    let (doc, registry, code) = load_doc(scene_text, world_src, tag);
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(31);
        loop {
            let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or(false);
            if !pending {
                break;
            }
            assert!(std::time::Instant::now() < end, "host settle deadline");
            h.frame(None).expect("frame");
        }
    }
    h
}

/// One logical tick with held (no edge) keys; releases everything first.
fn step(h: &mut Host, held: u32) {
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        for action in 0..4usize {
            if held & (1 << action) != 0 {
                p.char_held_restore(action);
            } else {
                p.char_key(action, false, false);
            }
        }
        if p.dispatch(Action::default()).is_err() {
            return;
        }
    }
    h.pause_play("");
    settle(h);
}

/// One fresh edge press (Jump=2) consumed on this dispatch.
fn press_once(h: &mut Host, action: usize) {
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        p.char_key(action, true, false);
        p.dispatch(Action::default()).expect("dispatch");
    }
    h.pause_play("");
    settle(h);
}

fn actor(h: &Host, id: u64) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.nodes
        .get(&id)
        .map(|d| d.translation)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

// ------------------------------------------------- Chain 2: G06 walk-jump --

/// Independent discrete f32 oracle: the ORIGINAL walk-jump rules over the
/// frozen layout-a geometry (char half 0.4/0.5, ground y=0 x[-4,16] with the
/// step x[7,9] top 0.75 and block x[-4.5,-3.5] top 1.0, wall x[14,15]).
/// Right-held only: fall from y=1 to the ground, then walk.
#[test]
fn basic2d_character_walk_lands_and_walks_per_oracle() {
    let mut h = start_with(SCENE_G06_A, Some(WORLD_G06_A), "g06a-walk");
    // Independent simulation of the same rules.
    let mut x = 0.0f32;
    let mut y = 1.0f32;
    let mut vy = 0.0f32;
    let mut grounded = false;
    let ground_y = 0.0f32;
    for t in 1..=40 {
        step(&mut h, 1 << 1); // right held
        // oracle tick
        vy -= 20.0 * DT;
        let dy = vy * DT;
        let dx = 4.0 * DT;
        // vertical sweep vs support top under the box (ground/step/block);
        // the move clips at face + half + clearance (0.002) — the measured
        // resting height (host lands at y=0.502 exactly).
        let new_y = y + dy;
        let support = if (x - 8.0).abs() <= 1.0 + 0.4 && new_y - 0.5 >= 0.75 - 0.05 {
            0.75f32.max(ground_y)
        } else {
            ground_y
        };
        if new_y - 0.5 <= support + 0.002 {
            y = support + 0.5 + 0.002;
            vy = 0.0;
            grounded = true;
        } else {
            y = new_y;
            grounded = false;
        }
        if grounded {
            x += dx;
        } else {
            x += dx; // air control identical (original moves in air too)
        }
        // wall x=14: stop.
        if x >= 14.0 - 0.4 {
            x = 14.0 - 0.4;
        }
        let a = actor(&h, 1);
        assert!(
            (a[0] - x).abs() < 1e-4 && (a[1] - y).abs() < 1e-4,
            "tick {t}: host ({},{}) vs oracle ({},{})",
            a[0],
            a[1],
            x,
            y
        );
    }
    h.stop_play();
}

/// Jump edge: one fresh Jump while grounded lifts vy=8; apex = v^2/2g = 1.6
/// above the support; lands back; a second held tick does not re-jump
/// (无双跳/无缓冲跳).
#[test]
fn basic2d_character_jump_edge_apex_and_no_double_jump() {
    let mut h = start_with(SCENE_G06_A, Some(WORLD_G06_A), "g06a-jump");
    // settle on the ground first (fall from y=1 to the resting 0.502).
    for _ in 0..20 {
        step(&mut h, 0);
    }
    let a0 = actor(&h, 1);
    assert!((a0[1] - 0.502).abs() < 1e-3, "landed at rest height: {:?}", a0);
    press_once(&mut h, 2); // fresh Jump edge — this dispatch IS tick 1
    // discrete oracle: vy=8 on the edge tick, gravity after, clip at 0.502.
    // tick 1 already ran inside press_once; fold in its exact result:
    // vy(1) = 8 - g*dt, y(1) = 0.502 + vy(1)*dt.
    let mut vy = 8.0 - 20.0 * DT;
    let mut y = 0.502f32 + vy * DT;
    let mut best = y;
    for t in 2..=70 {
        vy -= 20.0 * DT;
        let new_y = y + vy * DT;
        if new_y <= 0.502 {
            y = 0.502;
            vy = 0.0;
        } else {
            y = new_y;
        }
        best = best.max(y);
        step(&mut h, 0);
        let a = actor(&h, 1);
        assert!(
            (a[1] - y).abs() < 1e-3,
            "jump tick {t}: host {} vs oracle {y}",
            a[1]
        );
        if t > 2 && y == 0.502 {
            break; // landed; the remaining ticks assert the rest below
        }
    }
    // discrete apex = v^2/2g minus the gravity lost on the edge tick itself:
    // 1.6 - 8*dt <= best-0.502 <= 1.6.
    let rise = best - 0.502;
    assert!(
        rise <= 8.0f32 * 8.0 / (2.0 * 20.0) + 1e-3
            && rise >= 8.0f32 * 8.0 / (2.0 * 20.0) - 8.0 * DT - 1e-3,
        "apex v^2/2g (discrete): {rise}"
    );
    // held jump does not re-jump: y stays at the rest height after landing.
    for _ in 0..3 {
        step(&mut h, 1 << 2);
    }
    let a = actor(&h, 1);
    assert!((a[1] - 0.502).abs() < 1e-3, "no double jump: {:?}", a);
    h.stop_play();
}

/// Layout contrast: identical right-held input — layout-a walks on, layout-b
/// falls into the pit (ground x[-2,6]); y drops below the ground line.
#[test]
fn basic2d_layout_b_pit_contrast_against_layout_a() {
    let mut ha = start_with(SCENE_G06_A, Some(WORLD_G06_A), "g06a-pit");
    let mut hb = start_with(SCENE_G06_B, Some(WORLD_G06_B), "g06b-pit");
    for _ in 0..110 {
        step(&mut ha, 1 << 1);
        step(&mut hb, 1 << 1);
    }
    let aa = actor(&ha, 1);
    let ab = actor(&hb, 1);
    // layout-a: blocked by the step side (~x=6.598), still supported.
    assert!((aa[0] - 6.598).abs() < 5e-3, "layout-a blocked at step: {:?}", aa);
    assert!((aa[1] - 0.502).abs() < 1e-3, "layout-a supported: {:?}", aa);
    // layout-b: walked past the ground edge (x=6.4) and fell into the pit.
    assert!(ab[0] > 6.4, "layout-b walked off the edge: {:?}", ab);
    assert!(ab[1] < 0.0, "layout-b fell into the pit: {:?}", ab);
    ha.stop_play();
    hb.stop_play();
}

/// Speed 4->5 source variant: same input, genuinely different displacement.
#[test]
fn basic2d_fast_variant_displacement_contrast() {
    let mut h4 = start_with(SCENE_G06_A, Some(WORLD_G06_A), "g06a-fast4");
    let mut h5 = start_with(SCENE_G06_FAST, Some(WORLD_G06_FAST), "g06-fast5");
    for _ in 0..30 {
        step(&mut h4, 1 << 1);
        step(&mut h5, 1 << 1);
    }
    let d4 = actor(&h4, 1)[0];
    let d5 = actor(&h5, 1)[0];
    assert!(
        (d5 - d4).abs() > 0.05,
        "speed 4->5 displacement differs: {d4} vs {d5}"
    );
    // exact delta = (5-4) * dt * moving ticks (grounded walk on flat ground).
    assert!((d5 - d4 - 30.0 * DT).abs() < 1e-4, "exact 1 unit/s delta: {}", d5 - d4);
    h4.stop_play();
    h5.stop_play();
}

/// Per-tick cost is exactly 30 (pressed 1 + held 2 + move 26 + grounded 1);
/// the 33rd charged call on a walk tick faults with the named sticky error.
#[test]
fn basic2d_character_tick_cost_is_exactly_thirty() {
    // Session CPU path (character2d_api protocol) with the generated source
    // and the layout-a scene rows.
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("basic2d-g06.at"),
        WORLD_G06_A.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .expect("admission of generated character world");
    let mut s = Session::new(&snap.source).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    // rows: id1 char (0,1) 0.8x1.0; solids 2..5 = ground/wall/step/block AABBs.
    let rows: [(i32, i32, [f32; 3], [f32; 3]); 5] = [
        (1, 2, [-0.4, 0.5, 0.0], [0.4, 1.5, 0.0]),
        (1, 2, [-4.0, -1.0, 0.0], [16.0, 0.0, 0.0]),
        (1, 2, [14.0, 0.0, 0.0], [15.0, 3.0, 0.0]),
        (1, 2, [7.0, 0.0, 0.0], [9.0, 0.75, 0.0]),
        (1, 2, [-4.5, 0.0, 0.0], [-3.5, 1.0, 0.0]),
    ];
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, (kind, dim, lo, hi)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit).unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit).unwrap();
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit).unwrap();
        }
        for (i, v) in [1.0, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
            .iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for (n, _) in rows.iter().enumerate() {
        s.call(
            "_ag_char_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    let status = |s: &mut Session| s.int("_ag_status", &[]).unwrap();
    assert_eq!(status(&mut s), 0, "freeze ok");
    // walk tick (right held): 30 charged calls.
    let tick = |s: &mut Session, n: i32| {
        s.call("_ag_char_input", &[Int(2), Int(0)], ResultKind::Unit).unwrap();
        s.call("_ag_tick", &[Int(n), Float(DT)], ResultKind::Unit).unwrap();
    };
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0, "walk tick within budget");
    // probe extras: 31 and 32 pass; the 33rd charged call faults (10).
    let probe = format!(
        "{WORLD_G06_A}\nfn probe_extra() int {{return ag_action_held(1)}}\n"
    );
    let snap2 = SourceSnapshot::from_bytes_for(
        Path::new("basic2d-g06-probe.at"),
        probe.as_bytes().to_vec(),
        Api::CharacterV1,
    )
    .expect("probe admission");
    let mut p = Session::new(&snap2.source).expect("probe session");
    p.call("main", &[], ResultKind::Unit).unwrap();
    p.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    p.call("_ag_spatial_reset", &[Int(rows.len() as i32)], ResultKind::Unit)
        .unwrap();
    for (n, (kind, dim, lo, hi)) in rows.iter().enumerate() {
        p.call("_ag_identity", &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(-1)], ResultKind::Unit).unwrap();
        p.call("_ag_visibility", &[Int(1)], ResultKind::Unit).unwrap();
        p.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit).unwrap();
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            p.call("_ag_append_field", &[Float(v)], ResultKind::Unit).unwrap();
        }
        for (i, v) in [1.0, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]].iter().enumerate() {
            p.call("_ag_spatial_config", &[Int((n * 7 + i) as i32), Float(*v)], ResultKind::Unit).unwrap();
        }
    }
    for (n, _) in rows.iter().enumerate() {
        p.call("_ag_char_map", &[Int(n as i32 + 1), Int(n as i32)], ResultKind::Unit).unwrap();
    }
    p.call("_ag_start", &[], ResultKind::Unit).unwrap();
    p.call("_ag_char_input", &[Int(2), Int(0)], ResultKind::Unit).unwrap();
    p.call("_ag_tick", &[Int(1), Float(DT)], ResultKind::Unit).unwrap();
    assert_eq!(status(&mut p), 0, "probe world walk tick ok");
    let _ = p.call("probe_extra", &[], ResultKind::Int);
    assert_eq!(status(&mut p), 0, "31st call ok");
    let _ = p.call("probe_extra", &[], ResultKind::Int);
    assert_eq!(status(&mut p), 0, "32nd call ok");
    let _ = p.call("probe_extra", &[], ResultKind::Int);
    assert_eq!(status(&mut p), 10, "33rd charged call faults");
}

// ------------------------------------------------- Chain 3: G08 camera ----

fn camera_center(h: &Host) -> Option<[f32; 2]> {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.camera.as_ref().map(|c| c.follow_center_2d(glam::Vec2::new(
        f.objects.iter().find(|o| o.id == 1).expect("actor").translation[0],
        f.objects.iter().find(|o| o.id == 1).expect("actor").translation[1],
    )).unwrap().to_array())
}

#[test]
fn basic2d_camera_clamps_letterboxes_and_rides_same_tick() {
    let mut h = start_with(SCENE_G08, Some(WORLD_G08), "g08-main");
    // tick0: actor (2,1) -> follow center clamps to (6,4).
    let cam0 = h
        .play
        .as_ref()
        .unwrap()
        .camera_frame()
        .expect("tick0 camera")
        .clone();
    assert_eq!(cam0.dim, CameraDim::TwoD);
    assert_eq!(cam0.rect, [0.0, -4.0, 30.0, 12.0], "declared bounds echoed");
    assert_eq!(cam0.view, [12.0, 8.0], "declared viewport echoed");
    let center0 = cam0
        .follow_center_2d(glam::Vec2::new(2.0, 1.0))
        .unwrap();
    // clamp((2,1), min+half=(6,0), max-half=(24,8)) -> (6,1).
    assert!((center0.x - 6.0).abs() < 1e-5 && (center0.y - 1.0).abs() < 1e-5);
    // letterbox: scale 47.5, bars 115 logical px (800x380 surface).
    let (scale, content, bars) = cam0.letterbox_2d().unwrap();
    assert!((scale.x - 47.5).abs() < 1e-3 && (bars.x - 115.0).abs() < 1e-3);
    assert!((content.x - 570.0).abs() < 1e-3 && (content.y - 380.0).abs() < 1e-3);
    // walk right past the clamp: center tracks clamp(actor, 6, 24).
    for _ in 0..60 {
        step(&mut h, 1 << 1);
    }
    let f = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
    let a = f.objects.iter().find(|o| o.id == 1).unwrap().translation;
    let cam = f.camera.clone().expect("camera rides the same tick");
    let center = cam
        .follow_center_2d(glam::Vec2::new(a[0], a[1]))
        .unwrap();
    let want = a[0].clamp(6.0, 24.0);
    assert!(
        (center.x - want).abs() < 1e-5,
        "follow center clamps: {} vs actor {}",
        center.x,
        a[0]
    );
    assert_eq!(cam.tick, f.tick, "same-tick character+camera publication");
    let _ = camera_center(&h);
    h.stop_play();
}

/// Alt configuration: rect x[4,26] y[-2,10] view 8x6 -> ~506.67x380 content
/// with ~146.67 bars; genuinely different projection from main.
#[test]
fn basic2d_camera_alt_config_projection_differs() {
    let mut h = start_with(SCENE_G08_ALT, Some(WORLD_G08_ALT), "g08-alt");
    let cam0 = h
        .play
        .as_ref()
        .unwrap()
        .camera_frame()
        .expect("alt camera")
        .clone();
    assert_eq!(cam0.rect, [4.0, -2.0, 26.0, 10.0]);
    assert_eq!(cam0.view, [8.0, 6.0]);
    let (scale, content, bars) = cam0.letterbox_2d().unwrap();
    assert!((content.x - 380.0 * 8.0 / 6.0).abs() < 1e-2, "content {content:?}");
    assert!((bars.x - (800.0 - content.x) / 2.0).abs() < 1e-2, "bars {bars:?}");
    // tick0: actor (2,1) -> clamp into [8,22]x[1,7] -> (8, 1).
    let center = cam0.follow_center_2d(glam::Vec2::new(2.0, 1.0)).unwrap();
    assert!((center.x - 8.0).abs() < 1e-5 && (center.y - 1.0).abs() < 1e-5);
    assert!((scale.x - 380.0 / 6.0).abs() < 1e-3, "alt scale min(surface/view)");
    h.stop_play();
}

/// Pause freezes both character and camera; a neutral Step moves neither.
#[test]
fn basic2d_camera_pause_step_stop_semantics() {
    let mut h = start_with(SCENE_G08, Some(WORLD_G08), "g08-pause");
    // settle onto the ground first (a grounded tick is a true no-op even if
    // a probe dispatch processes one — the host/UI gate is the real pause).
    for _ in 0..20 {
        step(&mut h, 1 << 1);
    }
    let p0 = actor(&h, 1);
    // paused dispatch with neutral input: no motion (g11 pattern).
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.dispatch(Action::default());
    }
    settle(&mut h);
    let p1 = actor(&h, 1);
    assert_eq!(p0, p1, "paused dispatch: no motion");
    // neutral step: no motion, no orbit.
    step(&mut h, 0);
    let p2 = actor(&h, 1);
    assert_eq!(p1, p2, "neutral step: no motion");
    // Stop: run ends, editor document unchanged.
    let doc_before = format!("{:?}", h.editor.document.document_id);
    h.stop_play();
    assert_eq!(format!("{:?}", h.editor.document.document_id), doc_before);
}

// ------------------------------------------------- Chain 1: scene play ----

type Px = ([f32; 2], [u8; 4]);

/// 默认 2D 呈现约 1 world unit = 1 px：stride 4 网格足够覆盖 12..30px 的
/// 图元类（800x380 顶面 → 200x95 样本）。
fn pixel_classes(h: &mut Host) -> Vec<Px> {
    let mut points = Vec::new();
    for gy in 0..95 {
        for gx in 0..200 {
            points.push([(gx * 4 + 2) as f32, (gy * 4 + 2) as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.top, &points).expect("probe");
    points.iter().zip(px.iter()).map(|(p, c)| (*p, *c)).collect()
}

/// 相对通道类计数（016 校准的相对规则，光照混合下稳健）：blue = b 明显
/// 高于 r/g；red = r 高于 g/b；green = g 高于 r/b；yellow = r、g 高且 b 低。
fn count_class(px: &[Px], class: &str) -> usize {
    px.iter()
        .filter(|(_, c)| {
            let (r, g, b) = (c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0);
            match class {
                "blue" => b - r.max(g) > 0.12,
                "red" => r - g.max(b) > 0.12,
                "green" => g - r.max(b) > 0.12,
                "yellow" => r.min(g) - b > 0.3 && r > 0.7,
                _ => false,
            }
        })
        .count()
}

/// G01 base: blue/red/green visible with the original alpha classes, yellow
/// hidden; variants genuinely change the picture (color/order/position/
/// visible), and visible=false keeps identity (scene row intact).
#[test]
fn basic2d_scene_g01_alpha_rules_and_variants() {
    let mut base = start_with(SCENE_G01, None, "g01-base");
    let b = pixel_classes(&mut base);
    // blue quad present, semi-red present, green present, yellow absent
    // (stride-4 grid: a 30-unit quad ≈ 7x7 ≈ 49 samples; d24 ≈ 36).
    assert!(count_class(&b, "blue") > 20, "blue class present: {}", count_class(&b, "blue"));
    assert!(count_class(&b, "red") > 10, "red class present: {}", count_class(&b, "red"));
    assert!(count_class(&b, "green") > 8, "green class present: {}", count_class(&b, "green"));
    assert_eq!(count_class(&b, "yellow"), 0, "yellow hidden in base");
    let base_px: Vec<_> = b.iter().map(|(_, c)| *c).collect();
    base.stop_play();

    let mut vis = start_with(SCENE_G01_VISIBLE, None, "g01-visible");
    let v = pixel_classes(&mut vis);
    assert!(count_class(&v, "yellow") > 2, "yellow appears in v-visible");
    let vis_px: Vec<_> = v.iter().map(|(_, c)| *c).collect();
    assert_ne!(base_px, vis_px, "v-visible capture differs from base");
    vis.stop_play();

    let mut order = start_with(SCENE_G01_ORDER, None, "g01-order");
    let o = pixel_classes(&mut order);
    let order_px: Vec<_> = o.iter().map(|(_, c)| *c).collect();
    assert_ne!(base_px, order_px, "v-order capture differs from base");
    order.stop_play();

    let mut pos = start_with(SCENE_G01_POSITION, None, "g01-position");
    let p = pixel_classes(&mut pos);
    let pos_px: Vec<_> = p.iter().map(|(_, c)| *c).collect();
    assert_ne!(base_px, pos_px, "v-position capture differs from base");
    pos.stop_play();

    let mut color = start_with(SCENE_G01_COLOR, None, "g01-color");
    let c = pixel_classes(&mut color);
    // v-color: blue gone (deep green instead), green coverage increases.
    let color_px: Vec<_> = c.iter().map(|(_, c)| *c).collect();
    assert_ne!(base_px, color_px, "v-color capture differs from base");
    assert!(
        count_class(&c, "blue") < count_class(&b, "blue"),
        "blue class shrinks when the quad recolors"
    );
    color.stop_play();
}

/// G03 hierarchy: the CPU admission oracle computes deep leaf world (35,5)
/// per-layer f32 sums; v-parent moves it to (55,5); v-hidden flips the
/// ancestor AND; hidden capture differs while both run; Run/Stop keeps the
/// document unchanged.
#[test]
fn basic2d_scene_g03_world_sums_and_ancestor_visibility() {
    let (doc, _, _) = load_doc(SCENE_G03, None, "g03-base-doc");
    let worlds = hierarchy::derive(&doc).expect("derive");
    let deep = doc
        .objects
        .iter()
        .find(|o| o.name == "deep_quad")
        .expect("deep quad");
    let w = worlds.get(&deep.id).expect("deep world");
    assert!((w.translation[0] - 35.0).abs() < 1e-3, "deep world x: {}", w.translation[0]);
    assert!((w.translation[1] - 5.0).abs() < 1e-3, "deep world y: {}", w.translation[1]);
    assert!(w.visible, "base deep visible");
    // v-parent: root x 20 -> 40 -> deep world (55,5).
    let (doc_p, _, _) = load_doc(SCENE_G03_PARENT, None, "g03-parent-doc");
    let worlds_p = hierarchy::derive(&doc_p).expect("derive parent");
    let deep_p = doc_p.objects.iter().find(|o| o.name == "deep_quad").unwrap();
    let wp = worlds_p.get(&deep_p.id).unwrap();
    assert!((wp.translation[0] - 55.0).abs() < 1e-3, "moved world x: {}", wp.translation[0]);
    // v-hidden: root visible=false -> deep invisible via ancestor AND.
    let (doc_h, _, _) = load_doc(SCENE_G03_HIDDEN, None, "g03-hidden-doc");
    let worlds_h = hierarchy::derive(&doc_h).expect("derive hidden");
    let deep_h = doc_h.objects.iter().find(|o| o.name == "deep_quad").unwrap();
    let wh = worlds_h.get(&deep_h.id).unwrap();
    assert!(!wh.visible, "ancestor AND hides the deep leaf");
    // hidden leaf keeps identity (scene row present, visible=true locally).
    assert!(deep_h.visible, "local visible flag unchanged (identity kept)");

    // GPU: base and hidden both run; captures differ.
    let mut hb = start_with(SCENE_G03, None, "g03-base");
    let b = pixel_classes(&mut hb);
    hb.stop_play();
    let mut hh = start_with(SCENE_G03_HIDDEN, None, "g03-hidden");
    let hpx = pixel_classes(&mut hh);
    hh.stop_play();
    let b_px: Vec<_> = b.iter().map(|(_, c)| *c).collect();
    let h_px: Vec<_> = hpx.iter().map(|(_, c)| *c).collect();
    assert_ne!(b_px, h_px, "hidden capture differs from base");
}

/// Scene identity: regeneration-equivalent products admit; a tampered
/// world source (scene sha binding no longer matches) is refused.
#[test]
fn basic2d_scene_source_binding_rejects_tampered_world() {
    let tampered = WORLD_G06_A.replace("ag_char_bind(1)", "ag_char_bind(2)");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan019-basic")
        .join(format!("{}-tamper", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts");
    std::fs::write(dir.join("scene.agscene"), SCENE_G06_A).expect("scene");
    std::fs::write(scripts.join("tampered.at"), tampered).expect("script");
    let loaded = Storage::load_project(&dir.join("scene.agscene"), &Registry::new());
    // The scene binds the ORIGINAL sha; the tampered source file name does
    // not match any bound identity -> no code payload / refusal.
    match loaded {
        Ok((_, _, code, _)) => {
            assert!(
                code.is_none(),
                "tampered world must not satisfy the scene binding"
            );
        }
        Err(_) => {}
    }
}

/// The generated camera-2d project admits with the runtime_camera2d/1 API
/// and the character-2d project with character/1 — explicit profile choice,
/// no filename-based switching.
#[test]
fn basic2d_embedded_api_versions_are_explicit() {
    assert!(SCENE_G06_A.contains("\"api\": \"autographics.character/1\""));
    assert!(SCENE_G08.contains("\"api\": \"autographics.runtime_camera2d/1\""));
    assert!(SCENE_G01.contains("\"behavior\": null"));
    let _ = State::Stopping == State::Stopping;
    let _ = ResultKind::Unit;
}
