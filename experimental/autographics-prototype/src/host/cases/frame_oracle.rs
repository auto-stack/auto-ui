//! Content oracle (review r1 F07): per-pixel comparison of canonical
//! keyframes against FROZEN golden frames (`testdata/golden_frames/`,
//! sha256 in SHA256SUMS, tolerances in manifest.json from the oracle
//! contract). Freezing provenance is recorded in manifest.json — the frames
//! are from the current bit-exact implementation; the original-engine
//! ground-truth capture attempt is documented in research/result.md.

use std::path::PathBuf;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/golden_frames")
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FrameManifest {
    games: std::collections::HashMap<String, GameTol>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GameTol {
    mean_tol: f64,
    p99_tol: f64,
    /// Declared golden frame names (documentation of the frozen set).
    #[serde(default)]
    #[allow(dead_code)]
    frames: Vec<String>,
}

/// True if a frozen golden is declared for this frame in the manifest.
pub fn has_golden(game: &str, name: &str) -> bool {
    let Ok(bytes) = std::fs::read(golden_dir().join("manifest.json")) else {
        return false;
    };
    let Ok(manifest) = serde_json::from_slice::<FrameManifest>(&bytes) else {
        return false;
    };
    manifest
        .games
        .get(game)
        .map(|t| t.frames.iter().any(|f| f == name))
        .unwrap_or(false)
        && golden_dir().join(game).join(format!("{name}.png")).exists()
}

/// Per-pixel comparison of a captured frame against the frozen golden.
/// Returns (mean_abs_diff_per_channel, p99_abs_diff_per_channel, ok).
pub fn compare_golden_frame(game: &str, name: &str, png: &[u8]) -> Option<(f64, f64, bool)> {
    let manifest: FrameManifest =
        serde_json::from_slice(&std::fs::read(golden_dir().join("manifest.json")).ok()?).ok()?;
    let tol = manifest.games.get(game)?;
    let golden_path = golden_dir().join(game).join(format!("{name}.png"));
    let golden_bytes = std::fs::read(&golden_path).ok()?;
    // integrity: the frozen frame must match its recorded sha256
    if sha256(&golden_bytes) != recorded_sha(game, name)? {
        return None;
    }
    let a = image::load_from_memory(png).ok()?.to_rgba8();
    let b = image::load_from_memory(&golden_bytes).ok()?.to_rgba8();
    if a.dimensions() != b.dimensions() {
        return Some((f64::MAX, f64::MAX, false));
    }
    let mut diffs: Vec<u8> = Vec::with_capacity(a.len());
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let d = pa[0]
            .abs_diff(pb[0])
            .max(pa[1].abs_diff(pb[1]))
            .max(pa[2].abs_diff(pb[2]));
        diffs.push(d);
    }
    diffs.sort_unstable();
    let mean = diffs.iter().map(|d| *d as f64).sum::<f64>() / diffs.len().max(1) as f64;
    let p99_idx = ((diffs.len() as f64 - 1.0) * 0.99) as usize;
    let p99 = diffs
        .get(p99_idx.min(diffs.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0) as f64;
    let ok = mean <= tol.mean_tol && p99 <= tol.p99_tol;
    Some((mean, p99, ok))
}

fn recorded_sha(game: &str, name: &str) -> Option<String> {
    let sums = std::fs::read_to_string(golden_dir().join("SHA256SUMS")).ok()?;
    let rel = format!("{game}/{name}.png");
    for line in sums.lines() {
        let mut it = line.split_whitespace();
        let sha = it.next()?;
        let path = it.next()?;
        if path == rel {
            return Some(sha.to_string());
        }
    }
    None
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}
