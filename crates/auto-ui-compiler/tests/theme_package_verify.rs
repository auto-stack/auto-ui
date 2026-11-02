//! PLAN-031 T-04（work）——正式主题包外部获得/校验/拒绝/恢复（V-08 的
//! 来源校验腿）。真实包目录 + 第二绝对根 + 等长篡改 + 缺件 + 错版 + 逃逸路径。

use std::path::{Path, PathBuf};

use auto_ui_compiler::design_tokens::theme_package::{
    verify_theme_package, ThemePackageError, ThemePackageExpectation,
};
use auto_ui_compiler::design_tokens::registry::TokenName;
use auto_ui_compiler::design_tokens::theme_tree::{compose_mode, resolve_rgb};

fn pkg() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/themes")
}

fn expect() -> ThemePackageExpectation<'static> {
    ThemePackageExpectation {
        owner: "auto-ui",
        package: "themes",
        version: "0.2.0",
        schema: 1,
    }
}

fn copy_pkg(dst: &Path) -> PathBuf {
    std::fs::create_dir_all(dst).unwrap();
    for name in ["theme-package.json", "themes.at", "pac.at", "README.md"] {
        std::fs::copy(pkg().join(name), dst.join(name)).unwrap();
    }
    dst.to_path_buf()
}

#[test]
fn verifies_real_package_and_second_absolute_root() {
    let p = verify_theme_package(&pkg(), &expect()).expect("正式包应通过校验");
    assert_eq!((p.owner.as_str(), p.package.as_str()), ("auto-ui", "themes"));
    assert_eq!(p.version, "0.2.0");
    assert_eq!(p.families.len(), 2);
    assert!(p.families.contains_key("Auto") && p.families.contains_key("Solarized"));
    let auto = p.trees.get("Auto").expect("Auto");
    let light = compose_mode(auto, false).unwrap();
    assert_eq!(resolve_rgb(&light, TokenName::Background), Some((255, 255, 255)));

    // 第二绝对根（外部获得/移动后仍可校验，不依赖 producer 路径）。
    let dir = tempfile::tempdir().unwrap();
    let second = copy_pkg(&dir.path().join("vendor/themes"));
    let p2 = verify_theme_package(&second, &expect()).expect("第二根包应通过校验");
    assert_eq!(p2.trees.len(), p.trees.len());
}

#[test]
fn rejects_same_length_tamper_missing_file_bad_version_and_path_escape() {
    let dir = tempfile::tempdir().unwrap();
    let root = copy_pkg(&dir.path().join("themes"));

    // 等长篡改：翻转一个字节 → 具名 HashMismatch；恢复后正常。
    let themes_at = root.join("themes.at");
    let original = std::fs::read(&themes_at).unwrap();
    let mut tampered = original.clone();
    tampered[original.len() / 2] ^= 0x01;
    std::fs::write(&themes_at, &tampered).unwrap();
    match verify_theme_package(&root, &expect()) {
        Err(ThemePackageError::HashMismatch { path, .. }) => {
            assert_eq!(path, "themes.at", "错误必须定位到被篡改文件");
        }
        other => panic!("等长篡改必须 HashMismatch，得到 {other:?}"),
    }
    std::fs::write(&themes_at, &original).unwrap();
    verify_theme_package(&root, &expect()).expect("恢复后应重新通过");

    // 缺件：删除清单列出的 pac.at → MissingFile。
    std::fs::remove_file(root.join("pac.at")).unwrap();
    match verify_theme_package(&root, &expect()) {
        Err(ThemePackageError::MissingFile(f)) => assert_eq!(f, "pac.at"),
        other => panic!("缺件必须 MissingFile，得到 {other:?}"),
    }
    std::fs::copy(pkg().join("pac.at"), root.join("pac.at")).unwrap();

    // 错版：清单 version 改 0.2.1（长度相同）→ 具名 FieldMismatch(version)。
    let manifest = root.join("theme-package.json");
    let text = std::fs::read_to_string(&manifest).unwrap();
    let wrong = text.replace("\"version\": \"0.2.0\"", "\"version\": \"0.2.1\"");
    assert_ne!(wrong, text);
    std::fs::write(&manifest, &wrong).unwrap();
    match verify_theme_package(&root, &expect()) {
        Err(ThemePackageError::FieldMismatch { field, .. }) => assert_eq!(field, "version"),
        other => panic!("错版必须 FieldMismatch(version)，得到 {other:?}"),
    }

    // 逃逸路径：清单文件路径指向包外 → PathEscape。
    let escaped = text.replace("\"path\": \"pac.at\"", "\"path\": \"../evil.at\"");
    assert_ne!(escaped, text);
    std::fs::write(&manifest, &escaped).unwrap();
    match verify_theme_package(&root, &expect()) {
        Err(ThemePackageError::PathEscape(p)) => assert_eq!(p, "../evil.at"),
        other => panic!("逃逸路径必须 PathEscape，得到 {other:?}"),
    }
}
