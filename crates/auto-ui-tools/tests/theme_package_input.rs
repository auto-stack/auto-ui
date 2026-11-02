//! PLAN-031 T-04（work）——UI tools 主题包输入闸门：官方身份常量 + 具名拒绝。

use std::path::Path;

use auto_ui_tools::theme_package::{
    publish_theme_package_snapshot, verify_official_theme_package, verify_theme_package, theme_package_content_digest,
    read_theme_inputs_receipt, theme_file_sha256, verify_app_theme_inputs, write_theme_inputs_receipt,
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
#[test]
fn content_digest_is_location_independent_and_content_sensitive() {
    let d1 = theme_package_content_digest(&pkg()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("vendor/themes");
    publish_theme_package_snapshot(&pkg(), &copy).unwrap();
    assert_eq!(
        theme_package_content_digest(&copy).unwrap(),
        d1,
        "digest must not depend on the absolute root"
    );
    let themes_at = copy.join("themes.at");
    let original = std::fs::read(&themes_at).unwrap();
    let mut changed = original.clone();
    changed[original.len() / 2] ^= 0x01;
    std::fs::write(&themes_at, changed).unwrap();
    assert_ne!(theme_package_content_digest(&copy).unwrap(), d1);
}
// ── PLAN-031 T-07（V-09）：生成准入 + 生成输入 receipt ──────────────────

/// 保留 mtime（Windows 下用 std File::set_modified，不引额外依赖）。
fn set_mtime(path: &Path, t: std::time::SystemTime) {
    let f = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    f.set_modified(t).unwrap();
}
#[test]
fn app_theme_inputs_admit_official_package_and_receipt_records_digest() {
    let app = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui/010-contact-form")
        .canonicalize()
        .expect("010 例程目录");
    let root = verify_app_theme_inputs(&app)
        .expect("声明官方包就必须校验通过")
        .expect("应从 pac dep 解析出包根");
    assert_eq!(root, pkg().canonicalize().unwrap());

    let out = tempfile::tempdir().unwrap();
    let d1 = write_theme_inputs_receipt(out.path(), &app).expect("receipt 可写");
    assert_eq!(d1, theme_package_content_digest(&pkg()).unwrap());
    assert_eq!(
        read_theme_inputs_receipt(out.path()).as_deref(),
        Some(d1.as_str()),
        "receipt 读回必须等于写入摘要"
    );
}

#[test]
fn tampered_package_is_rejected_at_generation_admission() {
    let dir = tempfile::tempdir().unwrap();
    let pkg_dir = dir.path().join("themes");
    publish_theme_package_snapshot(&pkg(), &pkg_dir).unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir_all(app.join("src/front")).unwrap();
    std::fs::write(
        app.join("pac.at"),
        "name: \"t\"\nversion: \"0.1.0\"\ndep themes {\n    path: \"../themes\"\n}\n",
    )
    .unwrap();

    assert!(
        verify_app_theme_inputs(&app).unwrap().is_some(),
        "合法包应通过生成准入"
    );

    // 内容改但清单 hash 未更新 = 篡改 → 具名拒绝、不落半成品
    let themes_at = pkg_dir.join("themes.at");
    let mut bytes = std::fs::read(&themes_at).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x01;
    std::fs::write(&themes_at, &bytes).unwrap();
    let err = verify_app_theme_inputs(&app).expect_err("篡改必须拒绝");
    assert!(
        matches!(err, ThemePackageError::HashMismatch { .. }),
        "应具名为 HashMismatch，得到 {err:?}"
    );
}

/// V-09 核心：同名主题**内容变更且保留 mtime**（清单 hash 同步更新=合法发布）
/// 也必须让生成输入摘要变化；内容不变时摘要稳定。
#[test]
fn receipt_follows_content_not_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let pkg_dir = dir.path().join("themes");
    publish_theme_package_snapshot(&pkg(), &pkg_dir).unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir_all(app.join("src/front")).unwrap();
    std::fs::write(
        app.join("pac.at"),
        "name: \"t\"\nversion: \"0.1.0\"\ndep themes {\n    path: \"../themes\"\n}\n",
    )
    .unwrap();
    let out = tempfile::tempdir().unwrap();

    let d1 = write_theme_inputs_receipt(out.path(), &app).unwrap();

    // 未变：再写一次摘要稳定
    let d_same = write_theme_inputs_receipt(out.path(), &app).unwrap();
    assert_eq!(d1, d_same, "内容未变时生成输入摘要必须稳定");

    // 合法内容变更：改色 + 同步清单 sha256 + 还原 mtime
    let themes_at = pkg_dir.join("themes.at");
    let manifest_path = pkg_dir.join("theme-package.json");
    let mtime_themes = std::fs::metadata(&themes_at).unwrap().modified().unwrap();
    let mtime_manifest = std::fs::metadata(&manifest_path).unwrap().modified().unwrap();

    let src = std::fs::read_to_string(&themes_at).unwrap();
    let new_src = src.replace("primary: \"#8b5cf6\"", "primary: \"#123456\"");
    assert_ne!(src, new_src, "夹具必须真的改了内容");
    std::fs::write(&themes_at, &new_src).unwrap();

    let new_hash = theme_file_sha256(&themes_at).unwrap();
    let manifest_text = std::fs::read_to_string(&manifest_path).unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(&manifest_text).unwrap();
    let files = manifest.get_mut("files").unwrap().as_array_mut().unwrap();
    let mut patched = false;
    for f in files.iter_mut() {
        if f.get("path").and_then(|p| p.as_str()) == Some("themes.at") {
            f["sha256"] = serde_json::Value::String(new_hash.clone());
            patched = true;
        }
    }
    assert!(patched, "清单必须列出 themes.at");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap()).unwrap();

    // 保留 mtime（证明新鲜度按内容而不是时间戳）
    set_mtime(&themes_at, mtime_themes);
    set_mtime(&manifest_path, mtime_manifest);
    assert_eq!(
        std::fs::metadata(&themes_at).unwrap().modified().unwrap(),
        mtime_themes
    );

    let d2 = write_theme_inputs_receipt(out.path(), &app).unwrap();
    assert_ne!(d1, d2, "内容变化（mtime 不变）必须让生成输入摘要变化");
    assert_eq!(
        read_theme_inputs_receipt(out.path()).as_deref(),
        Some(d2.as_str())
    );
}