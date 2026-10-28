//! Permanent F-836-R1-01 CLI counterexamples: a truncated self-check cannot succeed.
use serde_json::Value;
use std::{path::PathBuf, process::Command};
static NATIVE_WINDOW: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn run(name: &str, args: &[&str]) -> (std::process::ExitStatus, Value) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let report = root
        .join("research")
        .join(format!("probe-exit-{}-{name}.json", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_autographics-authoring"))
        .current_dir(&root)
        .args(args)
        .arg("--report")
        .arg(&report)
        .output()
        .unwrap();
    let data: Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    (output.status, data)
}
#[test]
fn incomplete_checks_fail_but_observational_limits_remain_legal() {
    let _guard = NATIVE_WINDOW.lock().unwrap_or_else(|p| p.into_inner());
    for (name, args) in [
        ("ordinary", vec!["--interaction-check", "--frames", "1"]),
        (
            "wrong",
            vec!["--interaction-check", "--sabotage-target", "--frames", "1"],
        ),
        (
            "drop",
            vec!["--interaction-check", "--drop-release", "--frames", "1"],
        ),
        ("legacy", vec!["--window-probe", "--frames", "1"]),
        ("hierarchy", vec!["--hierarchy-check", "--frames", "1"]),
        (
            "hierarchy-wrong",
            vec!["--hierarchy-check", "--wrong-parent", "--frames", "1"],
        ),
        (
            "hierarchy-drop",
            vec!["--hierarchy-check", "--drop-release", "--frames", "1"],
        ),
    ] {
        let (status, report) = run(name, &args);
        assert!(
            !status.success(),
            "{name}: incomplete self-check returned success"
        );
        assert_eq!(report["passed"], false);
        assert!(
            report["error"]
                .as_str()
                .is_some_and(|s| s.contains("incomplete")),
            "{name}: report must carry incomplete reason: {report}"
        );
    }
    for (name, args) in [
        ("observe", vec!["--interaction-probe", "--frames", "1"]),
        ("ordinary-window", vec!["--frames", "1"]),
    ] {
        let (status, report) = run(name, &args);
        assert!(status.success(), "{name}: observation remains legal");
        assert!(report["error"].is_null());
        assert_eq!(report["passed"], false);
    }
    let missing = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("research")
        .join(format!("missing-scene-{}.agscene", std::process::id()));
    assert!(!missing.exists());
    let missing = missing.to_str().unwrap();
    let (status, report) = run("startup-error", &["--interaction-check", "--open", missing]);
    assert!(!status.success());
    assert_eq!(report["passed"], false);
    assert!(report["error"].as_str().is_some_and(|s| !s.is_empty()));
}

#[test]
fn legacy_scroll_errors_fail_before_global_timeout() {
    let _guard = NATIVE_WINDOW.lock().unwrap_or_else(|p| p.into_inner());
    let (status, report) = run(
        "legacy-scroll-fault",
        &["--window-probe", "--legacy-scroll-fault"],
    );
    assert!(!status.success());
    assert_eq!(report["passed"], false);
    assert_eq!(report["accepted"], 0);
    assert_eq!(
        report["error"],
        "legacy phase 1: legacy scroll input precondition fault"
    );
    assert!(report["frames"].as_u64().is_some_and(|n| n < 1400));
}

fn assert_prepared_before_input(report: &Value, records: &str) {
    assert_eq!(report["passed"], true, "{report}");
    let prepared = report[records]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["case"] == "input-prepared")
        .unwrap()["frame"]
        .as_u64()
        .unwrap();
    assert!(prepared >= 36);
    let inputs: Vec<_> = report["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| matches!(r["event"].as_str(), Some("wheel" | "down" | "up")))
        .collect();
    assert!(!inputs.is_empty());
    assert!(inputs
        .iter()
        .all(|r| r["frame"].as_u64().unwrap() > prepared));
}
fn assert_preparation_deadline(name: &str, args: &[&str], records: &str) {
    let mut args = args.to_vec();
    args.extend(["--probe-prepare-delay-frames", "241", "--frames", "250"]);
    let (status, report) = run(name, &args);
    assert!(!status.success());
    assert_eq!(report["passed"], false);
    assert!(
        report["error"]
            .as_str()
            .unwrap()
            .contains("timeout phase 0"),
        "{report}"
    );
    assert!(report[records]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["case"] != "input-prepared"));
    assert!(report["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| !matches!(r["event"].as_str(), Some("wheel" | "down" | "up"))));
}
#[test]
fn hierarchy_preparation_precedes_clipped_widget_scroll() {
    let _guard = NATIVE_WINDOW.lock().unwrap_or_else(|p| p.into_inner());
    let (status, report) = run(
        "hierarchy-preparation",
        &["--hierarchy-check", "--probe-prepare-delay-frames", "36"],
    );
    assert!(status.success(), "{report}");
    assert_eq!(report["hierarchy_phase"], 16);
    assert_prepared_before_input(&report, "hierarchy_records");
    assert_preparation_deadline(
        "hierarchy-prepare-deadline",
        &["--hierarchy-check"],
        "hierarchy_records",
    );
}
#[test]
fn asset_preparation_precedes_clipped_widget_scroll() {
    let _guard = NATIVE_WINDOW.lock().unwrap_or_else(|p| p.into_inner());
    let save = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("research")
        .join(format!("probe-exit-{}-asset-save", std::process::id()))
        .join("scene.agscene");
    std::fs::create_dir_all(save.parent().unwrap()).unwrap();
    let (status, report) = run(
        "asset-preparation",
        &[
            "--asset-check",
            "--asset-file",
            "research/asset-probe/two-parts.glb",
            "--asset-save",
            save.to_str().unwrap(),
            "--probe-prepare-delay-frames",
            "36",
        ],
    );
    assert!(status.success(), "{report}");
    assert_eq!(report["asset_phase"], 20);
    assert_prepared_before_input(&report, "asset_records");
    assert_preparation_deadline(
        "asset-prepare-deadline",
        &["--asset-check"],
        "asset_records",
    );
}
