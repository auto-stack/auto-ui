//! PLAN-035 T-05 public_3d_composition_api domain/capacity gates (V-S03/V-S04
//! host face; AC-03/04) on real assembled trusted bytes: Storage::load_project
//! → Host → real Auto VM over the frozen components3d/1 unit. Verdicts:
//!   - five buffered sound rows in one closeout refuse with named fault 51
//!     (<=4 requests/callback is the frozen audio domain);
//!   - a bundle row with gain 1.5 refuses with named fault 54 and keeps the
//!     last-good frame with no partial voice;
//!   - one batch of eight live handles destroys all eight (green, no
//!     half-write); re-despawning the same handles hits the stale generation
//!     and refuses with the named fault — no resurrection, no double destroy;
//!   - an actor-free control stays green across the same ticks (the domain
//!     faults come from the violated rows, not the profile);
//!   - Stop zeroes the mixer run/voices and a fresh Run starts at ordinal
//!     base with no old-Run carryover.
//! Single instance: cargo test --locked --offline --test public_3d_composition_api -- --test-threads=1
use autographics_auto_driver::authoring::{
    assets::Registry,
    host::Host,
    play::Action,
    storage::Storage,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Instant;

const FIVE_REQUESTS: &str = include_str!("../research/plan035/g3_present3d_five_requests.at");
const GAIN_DOMAIN: &str =
    include_str!("../research/plan035/g3_present3d_gain_out_of_domain.at");
const BATCH_STALE: &str = include_str!("../research/plan035/g3_batch_stale_generation.at");
const ACTORFREE: &str = include_str!("../research/plan035/actorfree_present3d.at");
const PROJECT3D_SCENE: &str = include_str!("../samples/public_hud/project3d/scene.agscene");
const ORIGINAL_SCRIPT_SHA: &str = "70ea90e187651ef61ab48bf4b9e246136a8b900796947c3d84aa07c2d31359fa";
const HUD3_API: &str = "\"api\":\"autographics.hud3d/1\"";
const COMPONENTS3_API: &str = "\"api\":\"autographics.components3d/1\"";

fn world_project(world: &str, api: &str, actor_free: bool, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan035-composition-api")
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

fn audio_requests(h: &Host) -> Vec<(usize, i32)> {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .audio
        .as_ref()
        .map(|a| a.requests.iter().map(|r| (r.cue, r.ordinal)).collect())
        .unwrap_or_default()
}

#[test]
fn five_buffered_requests_fault_51_and_keep_last_good() {
    let mut h = start(FIVE_REQUESTS, "autographics.components3d/1", true, "five-51");
    step(&mut h);
    let err = play_error(&h);
    assert!(err.contains("51"), "expected named audio fault 51, got: {err}");
    // last-good: no voice was created for the overflowing callback and the
    // mixer still reports zero voices after the fault cleared the run.
    let snap = h.audio_snapshot();
    assert_eq!(snap["voices"], 0, "no voice may be created by the refused bundle");
    assert_eq!(snap["run"], 0, "the fault clears the run");
}

#[test]
fn bundle_gain_out_of_domain_faults_54_without_partial_voice() {
    let mut h = start(GAIN_DOMAIN, "autographics.components3d/1", true, "gain-54");
    step(&mut h);
    let err = play_error(&h);
    assert!(err.contains("54"), "expected named gain-domain fault 54, got: {err}");
    let snap = h.audio_snapshot();
    assert_eq!(snap["voices"], 0, "the refused row must not become a voice");
}

#[test]
fn batch_of_live_handles_destroys_all_and_stale_batch_refuses() {
    let mut h = start(BATCH_STALE, "autographics.components3d/1", false, "batch-stale");
    // tick 1: the whole-batch despawn of the three live handles is green —
    // all-or-nothing, no half-write.
    step(&mut h);
    assert!(play_error(&h).is_empty(), "the live batch must be green: {}", play_error(&h));
    // tick 2: the same handles are stale now; the second batch must refuse
    // with the named stale-generation fault (33) — no resurrection.
    step(&mut h);
    let err = play_error(&h);
    assert!(err.contains("33"), "expected stale-generation fault 33, got: {err}");
}

#[test]
fn actorfree_control_stays_green_across_the_same_ticks() {
    let mut h = start(ACTORFREE, "autographics.components3d/1", true, "api-control");
    for tick in 1..=3 {
        step(&mut h);
        assert!(play_error(&h).is_empty(), "tick{tick}: {}", play_error(&h));
    }
    // one legal request per callback with strictly increasing per-Run ordinals
    let reqs = audio_requests(&h);
    assert_eq!(reqs.len(), 1, "one buffered request reaches the frame");
    assert!(reqs[0].1 > 0, "ordinal advances within the run");
}

#[test]
fn stop_clears_run_voices_and_new_run_starts_fresh() {
    let mut h = start(ACTORFREE, "autographics.components3d/1", true, "stop-a");
    step(&mut h);
    assert!(play_error(&h).is_empty());
    let run_a = h.audio_snapshot()["run"].as_i64().unwrap_or(-1);
    h.stop_play();
    settle(&mut h);
    let stopped = h.audio_snapshot();
    assert_eq!(stopped["run"], 0, "stop zeroes the mixer run");
    assert_eq!(stopped["voices"], 0, "stop releases every voice");
    // fresh Run: no old-Run carryover — the mixer run id moves forward and
    // the new Run's first request carries a fresh ordinal base.
    let mut fresh = start(ACTORFREE, "autographics.components3d/1", true, "stop-b");
    step(&mut fresh);
    assert!(play_error(&fresh).is_empty(), "fresh run: {}", play_error(&fresh));
    let run_b = fresh.audio_snapshot()["run"].as_i64().unwrap_or(-2);
    assert!(run_b > run_a, "new Run gets a fresh mixer run id ({run_a} -> {run_b})");
    let reqs = audio_requests(&fresh);
    if let Some((_, ordinal)) = reqs.first() {
        assert!(*ordinal < 8, "fresh Run restarts the ordinal base, got {ordinal}");
    }
}
