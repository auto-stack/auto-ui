//! PLAN-031 T-04（work）——UI tools 主题包输入闸门：官方身份常量 + 具名拒绝。

use std::path::Path;

use auto_ui_tools::theme_package::{
    verify_official_theme_package, verify_theme_package, ThemePackageError,
    ThemePackageExpectation,
};

fn pkg() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/themes")
}

#[test]
fn tools_input_accepts_official_package_and_reports_families() {
    let p = verify_official_theme_package(&pkg()).expect("官方包应通过 tools 输入闸门");
    assert_eq!(p.package, "themes");
    assert_eq!(p.version, "0.2.0");
    assert!(p.families.contains_key("Auto") && p.families.contains_key("Solarized"));
}

#[test]
fn tools_input_rejects_wrong_version_with_named_error() {
    let err = verify_theme_package(
        &pkg(),
        &ThemePackageExpectation {
            owner: "auto-ui",
            package: "themes",
            version: "0.1.0",
            schema: 1,
        },
    )
    .expect_err("错版必须拒绝");
    match err {
        ThemePackageError::FieldMismatch { field, .. } => assert_eq!(field, "version"),
        other => panic!("错版必须 FieldMismatch(version)，得到 {other:?}"),
    }
}
