//! PLAN-104（U02 A2Vue 余例浏览器档）：lane 产物 → 真浏览器（Playwright/Chromium）
//! 通用挂载。覆盖 004/006/007/008/009（005/010 由 1072 正式收据覆盖，见计划 §11 r16）。
//!
//! 判据：lane `generate_sfcs_with_lane_product` 派生 → `component.sfc` + `a2x_rt.mjs`
//! 原样落盘 → node rig 挂载 → 期望子串全部出现 + 零 page/console error。
//! 排窗档：`AUTO_UI_ACE_WEB_E2E=1`（playwright 必需）。
#![allow(dead_code)]

use std::path::Path;

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> std::path::PathBuf {
    manifest().join("../..")
}

fn rig() -> std::path::PathBuf {
    repo_root().join("examples/ui/ace-web-lane/tests/test_p104_generic_mount.mjs")
}

fn node() -> String {
    std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into())
}

fn scheduled() -> bool {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!("not scheduled (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright required)");
        return false;
    }
    true
}

/// 生成 lane 产物并真浏览器挂载；返回 evidence 文本（判据命中行）。
fn mount_app(app: &str, expected: &[&str]) -> String {
    use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product;
    let at = repo_root()
        .join("examples/ui")
        .join(app)
        .join("src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{app}: {e}"));
    // 真实源路径：style 配方（如 007 的 stylekit 依赖）按路径相对解析。
    let product = generate_sfcs_with_lane_product(&src, at.to_str().unwrap(), "./a2x_rt.mjs", "./a2x_rt.mjs")
        .unwrap_or_else(|e| panic!("{app}: lane derive failed: {e}"));
    let dir = std::env::temp_dir().join(format!("p104-vue-{app}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("component.sfc"), &product.widgets[0].sfc).unwrap();
    std::fs::write(dir.join("a2x_rt.mjs"), &product.runtime_mjs).unwrap();
    let out = std::process::Command::new(node())
        .arg(rig())
        .arg(&dir)
        .arg(expected.join(";"))
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("P104_GENERIC_MOUNT_OK"),
        "[{app}] lane-product browser mount failed:\n{stdout}\n{stderr}"
    );
    stdout
}

#[test]
fn p104_004_vue_browser_mount() {
    if !scheduled() {
        return;
    }
    let out = mount_app(
        "004-profile-card",
        &["Jane Cooper", "Full Stack Developer", "Active"],
    );
    eprintln!("{out}");
}

#[test]
fn p104_006_vue_browser_mount() {
    if !scheduled() {
        return;
    }
    // 006 的 GetStarted 是 foreign handler（lane absent），静态文案仍须真渲染。
    let out = mount_app(
        "006-hero-section",
        &["Build Beautiful Apps", "Get Started"],
    );
    eprintln!("{out}");
}

#[test]
fn p104_007_vue_browser_mount() {
    if !scheduled() {
        return;
    }
    let out = mount_app(
        "007-stats-board",
        &["Dashboard", "Campaign Insights", "123k"],
    );
    eprintln!("{out}");
}

#[test]
fn p104_008_vue_browser_mount() {
    if !scheduled() {
        return;
    }
    let out = mount_app(
        "008-pricing-table",
        &["Single Developer", "$39", "Contact Us"],
    );
    eprintln!("{out}");
}

#[test]
fn p104_009_vue_browser_mount() {
    if !scheduled() {
        return;
    }
    let out = mount_app(
        "009-article-feed",
        &["Blog Posts", "Sarah Chen", "Tutorial"],
    );
    eprintln!("{out}");
}
