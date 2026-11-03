//! PLAN-035 T-03 full-chain spike probes (candidates A/B + actor-free + named
//! rejections) on real assembled trusted bytes: Storage::load_project → Host
//! → real Auto VM. The spike scene is the delivered project3d sample scene
//! (hud3d/1, scene/8: character/platform/coin/flag GLBs, three-cue bank, four
//! HUD slots) with only the behavior source sha/length, the behavior api and
//! (for the actor-free probe) the actor declaration patched; the probe worlds
//! live under research/plan035/. Verdicts:
//!   - candidate A (hud3d/1 cursor + per-object despawn) dies on the
//!     collection tick with the named credit fault 10;
//!   - candidate B (components3d/1 snapshot + batch despawn) runs the same
//!     collection at 31/32 (plus the 32/32 boundary variant) and wins the
//!     three-coin same-tick case;
//!   - actor-free kind=-1 present3d is legal only under components3d/1 (the
//!     derived conditional freeze); the same source faults 25 under hud3d/1;
//!   - snapshot/cursor mixing faults 58, batch n=9 faults 59.
//! Single instance: cargo test --locked --offline --test plan035_probe -- --test-threads=1
use autographics_auto_driver::authoring::document::Document;
use autographics_auto_driver::authoring::{
    assets::Registry,
    host::Host,
    play::Action,
    storage::Storage,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Instant;

const A_CURSOR: &str = include_str!("../research/plan035/g3_a_cursor.at");
const B_SNAPSHOT: &str = include_str!("../research/plan035/g3_b_snapshot.at");
const B_SNAPSHOT_MAX: &str = include_str!("../research/plan035/g3_b_snapshot_max.at");
const ACTORFREE: &str = include_str!("../research/plan035/actorfree_present3d.at");
const MIX_VIOLATION: &str = include_str!("../research/plan035/g3_cmp_mix_violation.at");
const BATCH9_VIOLATION: &str = include_str!("../research/plan035/g3_cmp_batch9_violation.at");
const PROJECT3D_SCENE: &str = include_str!("../samples/public_hud/project3d/scene.agscene");
const ORIGINAL_SCRIPT_SHA: &str = "70ea90e187651ef61ab48bf4b9e246136a8b900796947c3d84aa07c2d31359fa";
const HUD3_API: &str = "\"api\":\"autographics.hud3d/1\"";
const COMPONENTS3_API: &str = "\"api\":\"autographics.components3d/1\"";

fn world_project(world: &str, api: &str, actor_free: bool, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan035-probe")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).expect("scripts dir");
    let sha = format!("{:x}", Sha256::digest(world.as_bytes()));
    let mut scene = PROJECT3D_SCENE.replacen(ORIGINAL_SCRIPT_SHA, &sha, 1);
    scene = scene.replacen(
        "\"byte_length\":1016",
        &format!("\"byte_length\":{}", world.as_bytes().len()),
        1,
    );
    scene = scene.replacen(HUD3_API, &format!("\"api\":\"{api}\""), 1);
    if actor_free {
        scene = scene.replacen("\"three_d\":1", "\"three_d\":null", 1);
    }
    std::fs::write(dir.join("scene.agscene"), &scene).expect("write scene");
    std::fs::write(dir.join("scripts").join(format!("{sha}.at")), world).expect("write script");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/public_hud/project3d/assets");
    for entry in std::fs::read_dir(&src).expect("project3d assets dir") {
        let entry = entry.expect("asset entry");
        let dest = dir.join("assets").join(entry.file_name());
        std::fs::create_dir_all(dest.parent().expect("assets parent")).expect("assets dir");
        std::fs::copy(entry.path(), &dest).expect("asset copy");
    }
    dir
}

fn start(world: &str, api: &str, actor_free: bool, tag: &str) -> Host {
    let dir = world_project(world, api, actor_free, tag);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("{tag} load scene project: {e:?}"));
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
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

/// One logical tick with neutral input; faults surface in play_error, never
/// as step failures (the candidate-A death must be observable as state).
fn step(h: &mut Host) {
    if let Some(p) = h.play.as_mut() {
        let _ = p.resume();
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

fn hud_slot(h: &Host, slot: usize) -> (bool, String) {
    let p = h.play.as_ref().unwrap();
    let frame = p.last.as_ref().expect("frame");
    let hud = frame.hud.as_ref().expect("hud frame");
    let s = &hud.slots[slot];
    (s.visible, s.text.clone())
}

#[test]
fn candidate_a_cursor_collection_overflows_credit_pool() {
    let mut h = start(A_CURSOR, "autographics.hud3d/1", false, "a-tick1");
    // tick1: spawns evaluate; consumption still fits (32 exactly).
    step(&mut h);
    assert!(
        play_error(&h).is_empty(),
        "tick1 must be green, got: {}",
        play_error(&h)
    );
    // tick2: the collection tick — cursor reads + per-object despawn overflow
    // the frozen pool; the named credit fault stops the run (last-good kept).
    step(&mut h);
    let err = play_error(&h);
    assert!(!err.is_empty(), "candidate A must die on the collection tick");
    assert!(err.contains("10"), "expected credit fault 10, got: {err}");
}

#[test]
fn candidate_b_snapshot_batch_collection_fits_pool() {
    let mut h = start(B_SNAPSHOT, "autographics.components3d/1", false, "b-run");
    step(&mut h);
    assert!(play_error(&h).is_empty(), "tick1: {}", play_error(&h));
    // tick2: three coins entered together; one snapshot + one whole-batch
    // despawn + the bundle stay at 31 of 32 and all three are destroyed.
    step(&mut h);
    assert!(play_error(&h).is_empty(), "tick2: {}", play_error(&h));
    let (_, text) = hud_slot(&h, 0);
    assert_eq!(text, "3", "three-coin same-tick collection must complete");
    let (_, end_text) = hud_slot(&h, 3);
    assert_eq!(end_text, "Won", "win slot flips exactly at 3/3");
    // tick3: steady state (30 charges) stays green.
    step(&mut h);
    assert!(play_error(&h).is_empty(), "tick3: {}", play_error(&h));
}

#[test]
fn candidate_b_pool_boundary_32_is_green() {
    let mut h = start(
        B_SNAPSHOT_MAX,
        "autographics.components3d/1",
        false,
        "b-max",
    );
    for tick in 1..=3 {
        step(&mut h);
        assert!(
            play_error(&h).is_empty(),
            "tick{tick} at the 32 boundary: {}",
            play_error(&h)
        );
    }
    let (_, text) = hud_slot(&h, 0);
    assert_eq!(text, "3", "boundary run still collects all three coins");
}

#[test]
fn actorfree_kind_minus1_present3d_is_components3d_only() {
    let mut h = start(
        ACTORFREE,
        "autographics.components3d/1",
        true,
        "actorfree-c3",
    );
    for tick in 1..=3 {
        step(&mut h);
        assert!(
            play_error(&h).is_empty(),
            "actor-free tick{tick}: {}",
            play_error(&h)
        );
    }
    assert!(
        h.play.as_ref().unwrap().character_profile().is_none(),
        "actor-free run binds no character"
    );
    let (_, text) = hud_slot(&h, 0);
    assert_eq!(text, "3", "HUD state advances without any actor");
    // The same source under the delivered hud3d/1 stack hits the unconditional
    // character freeze: the old base has no legal actor-free path.
    let mut old = start(ACTORFREE, "autographics.hud3d/1", true, "actorfree-h3");
    step(&mut old);
    let err = play_error(&old);
    assert!(
        err.contains("25"),
        "hud3d/1 actor-free must hit freeze fault 25, got: {err}"
    );
}

#[test]
fn snapshot_cursor_mix_faults_58() {
    let mut h = start(
        MIX_VIOLATION,
        "autographics.components3d/1",
        false,
        "mix-58",
    );
    step(&mut h);
    let err = play_error(&h);
    assert!(err.contains("58"), "expected named fault 58, got: {err}");
}

#[test]
fn batch_length_nine_faults_59() {
    let mut h = start(
        BATCH9_VIOLATION,
        "autographics.components3d/1",
        false,
        "batch9-59",
    );
    step(&mut h);
    let err = play_error(&h);
    assert!(err.contains("59"), "expected named fault 59, got: {err}");
}
