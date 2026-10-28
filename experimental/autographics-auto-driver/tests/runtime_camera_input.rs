//! PLAN-875 runtime camera input tests (contract V-03): the real OS window
//! probe — posted LMB drag orbits at the frozen rate, wheel zoom stays inside
//! the frozen domain, blur/late-move/resume-no-press stay inert, a fresh
//! press orbits again, and Stop restores the editing camera. The probe runs
//! the real authoring binary against the public 3D camera sample. Run
//! single-threaded:
//! cargo test --locked --test runtime_camera_input -- --test-threads=1
use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[test]
fn runtime_camera_window_probe_passes_on_the_public_sample() {
    let report: PathBuf =
        std::env::temp_dir().join(format!("runtime-camera-input-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&report);
    let exe = env!("CARGO_BIN_EXE_autographics-authoring");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // --behavior resolves against the opened scene when relative, so the
    // sample source goes in as an absolute path.
    let behavior = root.join("samples/runtime-camera/three/main.at");
    let scene = root.join("samples/runtime-camera/three/layout_a.agscene");
    let status = std::process::Command::new(exe)
        .args(["--runtime-camera-check", "--behavior"])
        .arg(&behavior)
        .args(["--behavior-api", "camera3d", "--open"])
        .arg(&scene)
        .args(["--report"])
        .arg(&report)
        .args(["--frames", "2400"])
        .current_dir(&root)
        .env("WINIT_X11_SCALE_FACTOR", "1")
        .status()
        .expect("spawn the authoring window probe");
    assert!(status.success(), "window probe exited {status}");
    let data: Value = serde_json::from_slice(&std::fs::read(&report).expect("probe report"))
        .expect("report json");
    assert_eq!(data["passed"], true, "probe passed: {}", data["error"]);
    assert_eq!(data["runtime_camera_check"], true);
    let phase = data["runtime_camera_phase"].as_u64().unwrap_or(0);
    assert!(phase >= 16, "probe phases incomplete: {phase}");
    let records = data["runtime_camera_records"].as_array().expect("records");
    let cases: Vec<&str> = records.iter().filter_map(|r| r["case"].as_str()).collect();
    for required in [
        "started",
        "lmb-down",
        "dragged",
        "lmb-up",
        "moved-rotated",
        "zoomed-out",
        "blur-late-inert",
        "resume-needs-press",
        "fresh-press-orbits",
        "stopped",
        "completed",
    ] {
        assert!(
            cases.contains(&required),
            "window record missing {required}: {cases:?}"
        );
    }
    // The frozen 0.25 deg/px rate: 360 posted logical px orbit to ~90 deg.
    // The mid-flight reading coalesces; the settled lmb-up record is exact.
    let settled = records.iter().find(|r| r["case"] == "lmb-up").unwrap();
    let yaw = settled["yaw"].as_f64().unwrap();
    assert!((yaw - 90.0).abs() < 2.0, "drag orbit yaw {yaw}");
    let zoomed = records.iter().find(|r| r["case"] == "zoomed-out").unwrap();
    let dist = zoomed["distance"].as_f64().unwrap();
    assert!((8.0..=12.0).contains(&dist), "zoomed distance {dist}");
    let _ = Instant::now(); // keep the time import for platform parity
    let _ = Duration::from_secs(0);
    let _ = std::fs::remove_file(&report);
}
