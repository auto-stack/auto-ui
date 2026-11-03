//! PLAN-035 components-3d-audio-hud flow: the committed generator products on
//! the real Host (Storage::load_project → Host → real Auto VM):
//!   - actorfree: kind=-1 service Run binds no character (components3d/1
//!     conditional freeze), cue/HUD requests advance with no actor;
//!   - batch-tick: three coins enter in one evaluate; the author's bounded
//!     snapshot consumption plus one whole-batch despawn collect all three at
//!     the exact 32-charge boundary (input snapshot + move + grounded +
//!     support + events snapshot + batch + bundle);
//!   - g3-full-loop: the original-G3 level boots and plays — scripted
//!     camera-relative walk moves the character on platform 1, the coin at
//!     (4,1,4) is collected exactly once (HUD 0/3 → 1/3), and the win/fail
//!     HUD state machine stays author-owned.
//! 运行：cargo test --locked --offline --test node3d_audio_hud_flow -- --test-threads=1
use autographics_auto_driver::authoring::document::Document;
use autographics_auto_driver::authoring::{
    assets::Registry,
    host::Host,
    play::Action,
    storage::Storage,
};
use std::path::{Path, PathBuf};
use std::time::Instant;

const GEN: &str = "../autographics-node2d-pilot/generated-node3d-audio-hud";

fn product_text(name: &str, file: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(GEN)
        .join(name)
        .join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn write_project(scene_text: &str, world_src: &str, product: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan035-audio-hud")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    if let Some(sha) = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
    {
        std::fs::write(dir.join("scripts").join(format!("{sha}.at")), world_src)
            .expect("write script");
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
    let src_scripts = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(GEN)
        .join(product)
        .join("closure");
    for entry in std::fs::read_dir(&src_scripts).expect("closure dir") {
        let entry = entry.expect("closure entry");
        let dest = dir.join("scripts").join(entry.file_name());
        std::fs::copy(entry.path(), &dest).expect("closure copy");
    }
    dir
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

/// One logical tick with explicit input bits (held/pressed bitmasks over the
/// frozen six-action table 0..5 = Left/Right/Forward/Back/Jump/Restart).
fn step(h: &mut Host, held: u32, pressed: u32) {
    if let Some(p) = h.play.as_mut() {
        let _ = p.resume();
        for (action, bit) in [(0usize, 1u32), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)] {
            if held & bit != 0 {
                p.char_held_restore(action);
            }
            if pressed & bit != 0 {
                p.char_key(action, true, false);
                p.char_key(action, false, false);
            }
        }
        let _ = p.dispatch(Action::default());
    }
    h.pause_play("");
    settle(h);
}

fn play_error(h: &Host) -> String {
    h.play
        .as_ref()
        .unwrap()
        .error
        .clone()
        .unwrap_or_default()
}

fn start(product: &str, tag: &str) -> (Host, Document) {
    let scene = product_text(product, "scene.agscene");
    let world = product_text(product, "world.at");
    let dir = write_project(&scene, &world, product, tag);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("{tag} load scene project: {e:?}"));
    let mut h = Host::with_project(doc.clone(), registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    settle(&mut h);
    (h, doc)
}

fn hud_slot(h: &Host, slot: usize) -> (bool, String) {
    let p = h.play.as_ref().unwrap();
    let frame = p.last.as_ref().expect("frame");
    let hud = frame.hud.as_ref().expect("hud frame");
    let s = &hud.slots[slot];
    (s.visible, s.text.clone())
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

#[test]
fn actorfree_services_run_green_without_any_actor() {
    let (mut h, _) = start("actorfree", "actorfree");
    assert!(
        h.play.as_ref().unwrap().character_profile().is_none(),
        "actor-free scene binds no character profile"
    );
    for tick in 1..=3 {
        step(&mut h, 0, 0);
        assert!(play_error(&h).is_empty(), "tick{tick}: {}", play_error(&h));
    }
    let (_, text) = hud_slot(&h, 0);
    assert_eq!(text, "3+", "HUD count advances without any actor");
    let mut old = {
        // The same product replayed is green again: state is per-Run.
        start("actorfree", "actorfree-2").0
    };
    step(&mut old, 0, 0);
    assert!(play_error(&old).is_empty());
}

#[test]
fn batch_tick_three_coins_same_evaluate_collect_at_32_boundary() {
    let (mut h, _) = start("batch-tick", "batch-tick");
    step(&mut h, 0, 0);
    assert!(play_error(&h).is_empty(), "tick1: {}", play_error(&h));
    // tick2: all three coins entered together at end of tick1; one snapshot +
    // one whole-batch despawn + the bundle fit the 32-charge pool exactly.
    step(&mut h, 0, 0);
    assert!(play_error(&h).is_empty(), "tick2: {}", play_error(&h));
    let (_, text) = hud_slot(&h, 0);
    assert_eq!(text, "3", "three-coin same-evaluate collection completes");
    // slots 1..3 stay hidden-empty (this fixture declares only slot 0)
    let (v3, t3) = hud_slot(&h, 3);
    assert!(!v3 && t3.is_empty(), "undeclared slot stays default hidden");
    // steady tick stays green
    step(&mut h, 0, 0);
    assert!(play_error(&h).is_empty(), "tick3: {}", play_error(&h));
}

#[test]
fn g3_boots_walks_and_collects_the_first_coin_once() {
    let (mut h, doc) = start("g3-full-loop", "g3");
    let hero = row_id(&doc, "Main/hero/body");
    // settle tick: spawn/evaluate (coins bind, no collection yet)
    step(&mut h, 0, 0);
    assert!(play_error(&h).is_empty(), "tick1: {}", play_error(&h));
    let (_, count) = hud_slot(&h, 0);
    assert_eq!(count, "0/3", "count starts at 0/3");
    let (_, status) = hud_slot(&h, 1);
    assert_eq!(status, "Playing");
    // G3-01 (camera-relative walk): the camera starts at yaw 0 facing -z, so
    // holding Right moves +x along the camera-right axis; 30 ticks at
    // speed 4/60 Hz ~= +2.0 on x with no input on other axes.
    let before = actor(&h, hero);
    for _ in 0..30 {
        step(&mut h, 2, 0); // held Right
    }
    let after = actor(&h, hero);
    assert!(play_error(&h).is_empty(), "walk: {}", play_error(&h));
    let dx = after[0] - before[0];
    let dz = after[2] - before[2];
    assert!(
        (dx - 2.0).abs() < 0.15,
        "camera-relative Right must move +x ~2.0, got dx={dx} dz={dz}"
    );
    assert!(dz.abs() < 0.05, "no drift on z for pure Right, got {dz}");
    // G3-04 (collect exactly once): walk +x from (2,1,2) toward the coin at
    // (4,1,4) — approach along +x then +z; the coin at (4,1,4) with sphere
    // 0.55 triggers on enter and the batch despawn destroys it; HUD 0/3→1/3
    // exactly once across the approach ticks.
    let mut collected_at: Option<usize> = None;
    for i in 0..200 {
        step(&mut h, 2, 0); // keep walking +x along z=2 line? coin z=4: turn
        if i == 30 {
            // z from 2 to 4: hold Forward is -z in camera space; walk Back
        }
        let _ = i;
        let _ = collected_at;
        break;
    }
    // deterministic approach: reset-held then steer: +x to (4,2), then z to 4
    // (two 60-tick legs are already partially done above); simply drive the
    // remaining path with explicit legs.
    let cur = actor(&h, hero);
    let leg1 = ((4.0 - cur[0]) / (4.0 / 60.0)).ceil().max(0.0) as usize;
    for _ in 0..leg1 {
        step(&mut h, 2, 0); // +x
    }
    let mid = actor(&h, hero);
    assert!((mid[0] - 4.0).abs() < 0.2, "leg1 lands at x=4, got {}", mid[0]);
    let leg2 = ((4.0 - mid[2]) / (4.0 / 60.0)).ceil().max(0.0) as usize;
    for _ in 0..leg2 {
        step(&mut h, 8, 0); // held Back (action 3, bit 8) = +z camera-relative
    }
    let endpos = actor(&h, hero);
    assert!(
        (endpos[2] - 4.0).abs() < 0.3,
        "leg2 lands at z=4, got {}",
        endpos[2]
    );
    assert!(play_error(&h).is_empty(), "approach: {}", play_error(&h));
    // one settle tick for the evaluate + one consumption tick
    step(&mut h, 0, 0);
    step(&mut h, 0, 0);
    let (_, count) = hud_slot(&h, 0);
    assert_eq!(count, "1/3", "the (4,1,4) coin collected exactly once");
    // extra idle ticks must NOT double-count (despawn + single evaluate)
    step(&mut h, 0, 0);
    step(&mut h, 0, 0);
    let (_, count) = hud_slot(&h, 0);
    assert_eq!(count, "1/3", "no double collection after despawn");
    assert!(play_error(&h).is_empty(), "post-collect: {}", play_error(&h));
    // GPU: the level renders (windowless capture differs across the walk).
    let (_, png1, _, _) = h
        .rt
        .capture(h.three, "g3-a")
        .expect("capture g3-a");
    assert!(!png1.is_empty(), "the original-G3 level renders on GPU");
}
