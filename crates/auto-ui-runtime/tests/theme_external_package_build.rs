//! PLAN-031 T-04（work）——外部第二根主题包参与真实 build（V-08 生成腿的
//! 最小 oracle）：把主题包复制到仓库外的绝对根、改 Auto light 背景为哨兵色，
//! 应用通过本地 `themes/` 包目录解析 `use themes.themes`；构建后注册的 Auto
//! 必须解析出哨兵色——证明生成链消费外部副本，而非仓库 packages/themes。
//!
//! 说明：pac `dep { path }` 的路径解析属宿主/auto 管线（T-07 端口）；本测试
//! 用等价的外部包目录证明「外部副本 → 真实解析/注册」链路，不冒充 pac dep。

use std::path::Path;

use auto_ui_compiler::design_tokens::registry::TokenName;
use auto_ui_compiler::design_tokens::theme_tree::{compose_mode, resolve_rgb};

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn build_reads_external_package_copy_not_repo_package() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let front = root.join("app/src/front");
    std::fs::create_dir_all(&front).unwrap();
    let app_src =
        std::fs::read_to_string(repo_root().join("examples/theme-lab/src/front/app.at"))
            .expect("theme-lab app.at");
    let app_path = front.join("app.at");
    std::fs::write(&app_path, &app_src).unwrap();

    // 外部包副本：放在消费根内（use 解析面对齐 `themes/themes.at`）。
    let pkg = front.join("themes");
    std::fs::create_dir_all(&pkg).unwrap();
    for f in ["themes.at", "pac.at", "theme-package.json"] {
        std::fs::copy(repo_root().join("packages/themes").join(f), pkg.join(f)).unwrap();
    }
    // 哨兵：只改外部副本的 Auto light 背景（#ffffff → #abcdef）。
    let themes_at = pkg.join("themes.at");
    let text = std::fs::read_to_string(&themes_at).unwrap();
    let sentinel = text.replace("background: \"#ffffff\"", "background: \"#abcdef\"");
    assert_ne!(text, sentinel, "fixture must actually change the package");
    std::fs::write(&themes_at, &sentinel).unwrap();

    let comp = auto_ui_runtime::build_dynamic_component(
        &app_src,
        Some(app_path.to_str().unwrap()),
    )
    .expect("external-package build");
    assert_eq!(comp.widget_name(), "App");

    let tree = auto_ui_compiler::style::theme::lookup_theme_tree("Auto")
        .expect("Auto registered from the external package copy");
    let light = compose_mode(&tree, false).unwrap();
    assert_eq!(
        resolve_rgb(&light, TokenName::Background),
        Some((0xab, 0xcd, 0xef)),
        "build must consume the external package copy, not the repo copy"
    );
}
