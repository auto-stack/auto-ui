//! PLAN-104（U02 组件面真窗）：004 / 006 的 AV/Iced 真窗渲染收据。
//!
//! 入口与 980 同一条（`examples/ace_node_desktop_window --source <app.at>
//! --evidence <json>`，组件面桥）；本文件把"真窗能开 + 文本/状态真提交"钉成
//! 永久例。`ui_apply_bool` 的 ABI 面（i32 0/1）由 004 的 `dark_mode=false`
//! 与 006 的 `dark_mode=true` 两实例覆盖。
//!
//! 单窗口纪律：env `AUTO_UI_ACE_NODE_E2E=1` 才跑；前置 `cargo build
//! --example ace_node_desktop_window --features ace-bridge`。

#![allow(dead_code)]

use std::path::{Path, PathBuf};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    manifest().join("../..")
}

fn window_exe() -> PathBuf {
    manifest().join("../../target/debug/examples/ace_node_desktop_window.exe")
}

fn run_window(app: &str, tag: &str, hold_ms: &str) -> serde_json::Value {
    assert_eq!(
        std::env::var("AUTO_UI_ACE_NODE_E2E").ok().as_deref(),
        Some("1"),
        "not scheduled: set AUTO_UI_ACE_NODE_E2E=1 after the prep stage (single-window discipline)"
    );
    let exe = window_exe();
    assert!(
        exe.is_file(),
        "preparation failure: {} missing — build `--example ace_node_desktop_window` first",
        exe.display()
    );
    let src = repo_root()
        .join("examples/ui")
        .join(app)
        .join("src/front/app.at");
    let ev = std::env::temp_dir().join(format!("plan104-window-{tag}-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&ev);
    let out = std::process::Command::new(&exe)
        .arg("--source")
        .arg(&src)
        .arg("--evidence")
        .arg(&ev)
        .arg("--instances")
        .arg("1")
        .arg("--hold-ms")
        .arg(hold_ms)
        .arg("--min-frames")
        .arg("3")
        .output()
        .expect("spawn window exe");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success() && ev.is_file(),
        "[{tag}] window exe failed: {stdout} / {stderr}"
    );
    let json = serde_json::from_str(&std::fs::read_to_string(&ev).unwrap()).unwrap();
    let _ = std::fs::remove_file(&ev);
    json
}

fn texts(session: &serde_json::Value) -> String {
    session["texts"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|r| {
                    r.as_array()
                        .map(|ts| {
                            ts.iter()
                                .filter_map(|t| t.as_str())
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .unwrap_or_default()
}

fn applied(session: &serde_json::Value) -> Vec<(String, u64, String)> {
    session["applied"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|r| {
                    (
                        r["kind"].as_str().unwrap_or("").to_string(),
                        r["slot"].as_u64().unwrap_or(u64::MAX),
                        r["value"].as_str().unwrap_or("").to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 004：静态资料卡——真窗渲染 + 7 字段（含 `dark_mode=false` 的 bool 提交）。
#[test]
fn p104_004_real_window_renders_profile_card() {
    let ev = run_window("004-profile-card", "004", "4000");
    assert_eq!(ev["node"].as_str(), Some("App"));
    assert!(
        ev["frames_rendered"].as_u64().unwrap_or(0) >= 3,
        "real window must render frames: {ev}"
    );
    let session = &ev["sessions"][0];
    assert!(session["applies"].as_u64().unwrap_or(0) >= 10);
    let shown = texts(session);
    assert!(shown.contains("Jane Cooper"), "card text applied: {shown}");
    assert!(
        shown.contains("Full Stack Developer"),
        "role text applied: {shown}"
    );
    let ap = applied(session);
    assert!(
        ap.iter()
            .any(|(k, s, v)| k == "bool" && *s == 6 && v == "false"),
        "bool state (dark_mode) must be applied on the ABI i32 lane: {ap:?}"
    );
}

/// 006：Hero 页——真窗渲染 + `dark_mode=true` 的 bool 提交（与 004 相反的极性）。
#[test]
fn p104_006_real_window_renders_hero() {
    let ev = run_window("006-hero-section", "006", "4000");
    assert_eq!(ev["node"].as_str(), Some("App"));
    assert!(
        ev["frames_rendered"].as_u64().unwrap_or(0) >= 3,
        "real window must render frames: {ev}"
    );
    let session = &ev["sessions"][0];
    let shown = texts(session);
    assert!(
        shown.contains("Build Beautiful Apps"),
        "hero headline applied: {shown}"
    );
    let ap = applied(session);
    assert!(
        ap.iter()
            .any(|(k, s, v)| k == "bool" && *s == 0 && v == "true"),
        "bool state (dark_mode) must be applied on the ABI i32 lane: {ap:?}"
    );
}
