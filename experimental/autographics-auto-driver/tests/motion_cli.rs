use std::{
    path::PathBuf,
    process::{Command, Output},
};
fn invoke(extra: &[&str]) -> Output {
    let scene = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("samples/spatial_projects/zone2d/scene.agscene");
    Command::new(env!("CARGO_BIN_EXE_autographics-authoring"))
        .arg("--inspect")
        .arg(scene)
        .args(extra)
        .output()
        .unwrap()
}
fn usage(output: Output, diagnostic: &str) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains(diagnostic));
    assert!(
        output.stdout.is_empty(),
        "usage failure must precede project inspection"
    );
}
#[test]
fn supplied_api_requires_a_value_before_project_inspection() {
    usage(
        invoke(&["--behavior-api"]),
        "--behavior-api requires v1 or v2",
    );
    usage(
        invoke(&["--behavior", "unused.at", "--behavior-api"]),
        "--behavior-api requires v1 or v2",
    );
    usage(
        invoke(&["--behavior-api", "--frames", "1"]),
        "--behavior-api requires v1 or v2",
    );
}
#[test]
fn orphan_and_unknown_api_values_remain_usage_errors() {
    for api in ["v1", "v2", "v3", "v4", "v5"] {
        usage(
            invoke(&["--behavior-api", api]),
            "--behavior-api requires temporary --behavior",
        );
    }
    usage(
        invoke(&["--behavior", "unused.at", "--behavior-api", "v999"]),
        "--behavior-api expects v1 or v2",
    );
}
#[test]
fn absent_and_valid_temporary_api_choices_preserve_project_binding() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/user_behavior/move/main.at");
    let source = source.to_str().unwrap();
    // Inspect accepts temporary choices without executing them or rebinding the project.
    // spatial_host separately covers actual temporary-file compilation and Run selection.
    for extra in [
        vec![],
        vec!["--behavior", source],
        vec!["--behavior", source, "--behavior-api", "v1"],
        vec!["--behavior", source, "--behavior-api", "v2"],
        vec!["--behavior", source, "--behavior-api", "v3"],
        vec!["--behavior", source, "--behavior-api", "v4"],
        vec!["--behavior", source, "--behavior-api", "v5"],
    ] {
        let output = invoke(&extra);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["closure_verified"], true);
        assert_eq!(
            report["document"]["behavior"]["api"], "autographics.behavior/2",
            "temporary choice must not rewrite the accepted binding"
        );
    }
}

#[test]
fn normal_run_rejects_missing_orphan_unknown_api_before_window_creation() {
    for args in [
        vec!["--behavior-api"],
        vec!["--behavior-api", "v4"],
        vec!["--behavior", "unused.at", "--behavior-api", "v999"],
        vec!["--behavior-api", "--frames", "1"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_autographics-authoring"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&out.stderr).contains("--behavior-api"));
        assert!(out.stdout.is_empty());
    }
}
