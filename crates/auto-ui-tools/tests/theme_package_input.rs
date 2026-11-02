//! PLAN-031 T-04（work）——UI tools 主题包输入闸门：官方身份常量 + 具名拒绝。

use std::path::Path;

use auto_ui_tools::theme_package::{
    publish_theme_package_snapshot, verify_official_theme_package, verify_theme_package,
    ThemePackageError, ThemePackageExpectation,
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


#[test]
fn snapshot_publish_is_independent_and_minimal() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("vendor/themes");
    let p = publish_theme_package_snapshot(&pkg(), &dest).expect("snapshot publish");
    assert_eq!(p.root, dest);
    verify_official_theme_package(&dest).expect("snapshot must verify independently");

    let mut names: Vec<String> = std::fs::read_dir(&dest)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec!["pac.at", "theme-package.json", "themes.at"],
        "snapshot must contain only manifest + listed files"
    );
}

#[test]
fn snapshot_publish_refuses_tampered_source_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("themes");
    std::fs::create_dir_all(&src).unwrap();
    for f in ["theme-package.json", "themes.at", "pac.at"] {
        std::fs::copy(pkg().join(f), src.join(f)).unwrap();
    }
    let original = std::fs::read(src.join("themes.at")).unwrap();
    let mut bad = original.clone();
    bad[original.len() / 2] ^= 0x01;
    std::fs::write(src.join("themes.at"), bad).unwrap();

    let dest = dir.path().join("vendor/themes");
    let err = publish_theme_package_snapshot(&src, &dest)
        .expect_err("tampered source must be refused");
    assert!(matches!(err, ThemePackageError::HashMismatch { .. }), "{err:?}");
    assert!(!dest.exists(), "refusal must not create a snapshot dir");
}