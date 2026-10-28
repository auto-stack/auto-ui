//! Committed-delivery closure (review r6 F829-R6-03): every runtime input
//! the probe cases and their gates read from the tree must be PRESENT and
//! VERSIONED. R6 proved that `git status` clean is NOT the delivery
//! boundary — the golden-frame manifest was gitignored (`*.json`), so a
//! `git archive HEAD` export could not run the games' content gates at
//! all. This test binds the required-input inventory permanently: each
//! path must exist, and (inside a git checkout) must be tracked and not
//! ignored. Outside git (e.g. an export fixture), existence still binds.

use std::path::Path;
use std::process::Command;

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Runtime-required inputs of the probe cases and their verifier gates.
/// Negative-control paths (e.g. `__missing__.glb`) are deliberately NOT
/// listed — they must not exist.
const REQUIRED: &[&str] = &[
    // F07 content oracle (frame_oracle.rs): manifest + integrity sums +
    // every manifest-declared golden frame.
    "testdata/golden_frames/manifest.json",
    "testdata/golden_frames/SHA256SUMS",
    "testdata/golden_frames/ag2d-breakout/first_paddle_contact.png",
    "testdata/golden_frames/ag2d-breakout/first_brick_contact.png",
    "testdata/golden_frames/ag2d-breakout/final_frame.png",
    "testdata/golden_frames/ag3d-squash/squash_moment.png",
    "testdata/golden_frames/ag3d-squash/retry_overlay.png",
    "testdata/golden_frames/ag3d-squash/final_frame.png",
    // Frozen independent traces the games replay against.
    "testdata/breakout/golden_trace.json",
    "testdata/squash/golden_squash_trace.json",
    // Fonts the HUD text renderers load.
    "assets/breakout/fonts/FiraMono-Medium.ttf",
    "assets/squash_the_creeps/fonts/Montserrat-Medium.ttf",
    // Audio assets the sound/music chains load.
    "assets/breakout/sounds/breakout_collision.ogg",
    "assets/squash_the_creeps/art/House In a Forest Loop.ogg",
    // 3D models (squash case + the iced spike's 3D view).
    "assets/squash_the_creeps/art/mob.glb",
    "assets/squash_the_creeps/art/player.glb",
    // Fingerprint / integrity receipts the cases attach and verify.
    "assets/SHA256SUMS",
    "testdata/SHA256SUMS",
    "testdata/breakout/FONTS.sha256",
];

fn git(args: &[&str]) -> Option<std::process::Output> {
    let out = Command::new("git")
        .args(args)
        .current_dir(manifest_dir())
        .output()
        .ok()?;
    out.status.success().then_some(out)
}

#[test]
fn required_runtime_inputs_exist_and_are_versioned() {
    let mut failures = Vec::new();
    for rel in REQUIRED {
        let path = manifest_dir().join(rel);
        if !path.exists() {
            failures.push(format!("{rel}: MISSING from the tree"));
            continue;
        }
        // Inside a git checkout the input must be tracked (committed
        // closure) — this is the exact R6-03 failure mode.
        if git(&["rev-parse", "--is-inside-work-tree"]).is_some() {
            if git(&["ls-files", "--error-unmatch", rel]).is_none() {
                failures.push(format!(
                    "{rel}: exists but NOT TRACKED — a committed-only export cannot ship it"
                ));
            }
            if Command::new("git")
                .args(["check-ignore", "-q", rel])
                .current_dir(manifest_dir())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                failures.push(format!("{rel}: ignored by a gitignore rule"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "delivery-closure violations:\n  {}",
        failures.join("\n  ")
    );
}

#[test]
fn golden_manifest_declares_only_present_frames() {
    // The manifest's declared frame names must resolve to real frozen
    // files — a declared-but-absent golden would silently disable the gate.
    let raw = std::fs::read(manifest_dir().join("testdata/golden_frames/manifest.json"))
        .expect("golden manifest readable");
    let m: serde_json::Value = serde_json::from_slice(&raw).expect("golden manifest parses");
    for (game, entry) in m["games"].as_object().expect("games object") {
        for frame in entry["frames"].as_array().expect("frames array") {
            let rel = format!(
                "testdata/golden_frames/{}/{}.png",
                game,
                frame.as_str().unwrap()
            );
            assert!(
                manifest_dir().join(&rel).exists(),
                "manifest declares {rel} but the file is absent"
            );
        }
    }
}
