//! T-08 contract tests: seam logic that is pure-state (the windowed/GPU
//! evidence lives in the iced-shared-device probe report).

use agp::host::gates;
use agp::host::iced_spike::ViewKind;

#[test]
fn seam_view_kinds_are_distinct() {
    assert_ne!(ViewKind::View2d, ViewKind::View3d);
}

#[test]
fn seam_blank_frame_gate_reuses_verifier() {
    // the same gate implementation guards the seam and the V-08 verifier
    assert!(gates::verify_frame_progression(true, "a", "b").is_ok());
    assert!(gates::verify_frame_progression(true, "a", "a").is_err());
}

#[test]
fn seam_second_device_gate_reuses_verifier() {
    assert!(gates::verify_same_device("dev0:x", &["dev0:x".into()]).is_ok());
    assert!(gates::verify_same_device("dev0:x", &["dev0:x".into(), "dev9:z".into()]).is_err());
}

#[test]
fn seam_window_title_is_deterministic() {
    // the PowerShell clicker finds the window by this exact title
    assert_eq!(
        agp::host::iced_spike::WINDOW_TITLE,
        "AGP iced-shared-device spike"
    );
}

/// F05 permanent negative control: the iced spike's render path must never
/// construct a second device. wgpu 27 cannot discriminate cross-device
/// resource use at runtime (global resource ids — batch-2 finding), so the
/// foreign-device invariant is enforced structurally: any device
/// construction inside the render path fails this audit. (The review's
/// foreign-device mutation = adding a `GpuDevice::new` to this module.)
#[test]
fn f05_render_path_has_no_second_device() {
    let src = include_str!("../src/host/iced_spike.rs");
    let constructions = src.matches("GpuDevice::new").count();
    assert_eq!(
        constructions, 0,
        "iced spike render path must not construct its own device (foreign-device control)"
    );
    let src_case = include_str!("../src/host/cases/iced_shared_device.rs");
    // the probe's diagnostic device is explicitly labeled in the case module
    assert!(
        src_case.contains("diagnostic") || src_case.contains("foreign"),
        "case-side second-device use must be labeled diagnostic/foreign"
    );
}
