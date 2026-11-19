//! PLAN-104（U02 A2Vue 余例浏览器档）：lane 产物 → **单浏览器会话**跑多应用。
//!
//! T-07 复审 F-05 收口：一次 Chromium 启动、逐应用起停本地 http（每应用独立
//! 目录），不再为每例冷启浏览器；逐例 wall time 随 `[t] <label> <ms>` 打印。
//!
//! 判据：lane 派生（真实源路径）→ SFC 含期望样式 token → 真浏览器挂载 →
//! 期望子串全部出现 + 零 page error / 零功能性 console error。
//! 排窗档：`AUTO_UI_ACE_WEB_E2E=1`。
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

/// (label, app, 期望文本, 期望样式 token)
const APPS: &[(&str, &str, &[&str], &[&str])] = &[
    (
        "004",
        "004-profile-card",
        &["Jane Cooper", "Full Stack Developer", "Active"],
        &["bg-gradient-to-r", "rounded-t-lg", "rounded-full"],
    ),
    (
        "006",
        "006-hero-section",
        &["Build Beautiful Apps", "Get Started"],
        &["bg-background", "text-foreground"],
    ),
    (
        "007",
        "007-stats-board",
        &["Dashboard", "Campaign Insights", "123k"],
        &["text-2xl", "font-bold", "text-foreground"],
    ),
    (
        "008",
        "008-pricing-table",
        &["Single Developer", "$39", "Contact Us"],
        &["bg-card", "border-border"],
    ),
    (
        "009",
        "009-article-feed",
        &["Blog Posts", "Sarah Chen", "Tutorial"],
        &["text-muted-foreground", "rounded-xl"],
    ),
];

#[test]
fn p104_vue_browser_mounts() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!("not scheduled (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright required)");
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product;
    let root = std::env::temp_dir().join(format!("p104-vue-batch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let mut specs = Vec::new();
    for (label, app, expected, tokens) in APPS {
        let at = repo_root()
            .join("examples/ui")
            .join(app)
            .join("src/front/app.at");
        let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{app}: {e}"));
        // 真实源路径：style 配方（如 007 的 stylekit 依赖）按路径相对解析。
        let product =
            generate_sfcs_with_lane_product(&src, at.to_str().unwrap(), "./a2x_rt.mjs", "./a2x_rt.mjs")
                .unwrap_or_else(|e| panic!("{app}: lane derive failed: {e}"));
        for tok in *tokens {
            assert!(
                product.widgets[0].sfc.contains(tok),
                "[{label}] generated SFC missing style token `{tok}`"
            );
        }
        let dir = root.join(label);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("component.sfc"), &product.widgets[0].sfc).unwrap();
        std::fs::write(dir.join("a2x_rt.mjs"), &product.runtime_mjs).unwrap();
        specs.push(serde_json::json!({
            "label": label,
            "dir": dir.to_str().unwrap(),
            "expected": expected,
        }));
    }
    let spec_path = root.join("spec.json");
    std::fs::write(
        &spec_path,
        serde_json::to_string(&serde_json::json!({ "apps": specs })).unwrap(),
    )
    .unwrap();

    let node_version = std::process::Command::new(node())
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let out = std::process::Command::new(node())
        .arg(rig())
        .arg(&spec_path)
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&root);
    eprintln!("node {node_version}");
    eprintln!("{stdout}");
    assert!(
        out.status.success() && stdout.contains("P104_GENERIC_MOUNT_OK apps=5"),
        "lane-product browser mounts failed:\n{stdout}\n{stderr}"
    );
    // 逐例耗时行必须齐（F-05 可审计）。
    for (label, _, _, _) in APPS {
        assert!(
            stdout.contains(&format!("[t] {label} ")),
            "missing per-app timing for {label}"
        );
    }
}
