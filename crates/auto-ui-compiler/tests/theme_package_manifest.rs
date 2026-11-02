//! PLAN-031 T-04（work）——正式主题包受管清单（SD-03 前置合同）。
//!
//! `packages/themes/theme-package.json` 是来源/能力清单：owner、包版本、
//! schema、文件 sha256、family 支持模式与默认。它**不是第二份颜色数据**——
//! 颜色唯一源是 `themes.at`；本测试钉住两者一致性与迁移边界。

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

fn pkg_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/themes")
}

#[test]
fn manifest_is_consistent_with_package_files_and_families() {
    let pkg = pkg_dir();
    let text = std::fs::read_to_string(pkg.join("theme-package.json"))
        .expect("packages/themes/theme-package.json 必须存在");
    let v: serde_json::Value = serde_json::from_str(&text).expect("manifest 必须是合法 JSON");

    assert_eq!(v["owner"], "auto-ui");
    assert_eq!(v["package"], "themes");
    assert_eq!(v["version"], "0.2.0");
    assert_eq!(v["schema"], 1);
    assert_eq!(v["module"], "themes");

    // 文件清单：路径不得逃出包目录；sha256 必须与内容一致。
    let files = v["files"].as_array().expect("manifest.files 数组");
    assert!(!files.is_empty(), "files 清单不得为空");
    for f in files {
        let rel = f["path"].as_str().expect("file.path");
        assert!(
            !rel.contains("..") && !rel.starts_with('/') && !rel.contains('\\'),
            "manifest 路径必须留在包内: {rel}"
        );
        let bytes = std::fs::read(pkg.join(rel)).unwrap_or_else(|e| panic!("{rel} 读取失败: {e}"));
        let digest = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(
            digest,
            f["sha256"].as_str().unwrap_or_default(),
            "sha256 不一致（改了 {rel} 必须同步 theme-package.json）"
        );
    }

    // pac 描述与清单一致（版本/包名），pac 不是第二权威源。
    let pac = std::fs::read_to_string(pkg.join("pac.at")).expect("pac.at");
    assert!(pac.contains("name: \"themes\""), "{pac}");
    assert!(pac.contains("version: \"0.2.0\""), "{pac}");

    // family/模式能力必须与真实数据一致（经真实主题解析，不按清单空口）。
    let themes_at = pkg.join("themes.at");
    let src = std::fs::read_to_string(&themes_at).expect("themes.at");
    let known = auto_ui_compiler::design_tokens::theme_tree::collect_named_themes(&themes_at, &src)
        .expect("themes.at 通过真实解析/schema 校验");
    let families = v["families"].as_array().expect("manifest.families 数组");
    assert!(!families.is_empty(), "families 清单不得为空");
    for fam in families {
        let name = fam["name"].as_str().expect("family.name");
        let tree = known
            .get(name)
            .unwrap_or_else(|| panic!("family `{name}` 在 themes.at 中不存在"));
        let modes = fam["modes"].as_array().expect("family.modes");
        assert!(!modes.is_empty(), "family `{name}` 必须声明支持模式");
        for mode in modes {
            let mode = mode.as_str().expect("mode string");
            let dark = match mode {
                "light" => false,
                "dark" => true,
                other => panic!("未知模式 `{other}`"),
            };
            assert!(
                auto_ui_compiler::design_tokens::theme_tree::mode_available(tree, dark),
                "manifest 声明 `{name}` 支持 {mode}，但主题数据未声明"
            );
        }
    }

    // 迁移边界：旧教学目录不再保留可编辑 .at 镜像。
    let legacy = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/themes");
    assert!(
        !legacy.join("themes.at").exists() && !legacy.join("pac.at").exists(),
        "examples/ui/themes 不得保留可编辑 .at（第二权威源）"
    );
    assert!(
        legacy.join("README.md").exists(),
        "旧目录应保留迁移 README"
    );
}
