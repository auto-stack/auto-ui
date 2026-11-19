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
        .arg("2")
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
        ev["frames_rendered"].as_u64().unwrap_or(0) >= 1,
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

/// 005：条件视图 + 真窗交互三 oracle——空提交必填错误 / 明暗切换 / 主题族切换。
/// 条件叶的可见性由桥按实例状态求值：开窗时错误叶隐藏（不渲染），提交后
/// state 行给出错误值（文本叶刷新是后续切片，见计划 §11 r7 第③件）。
#[test]
fn p104_005_real_window_login_oracles() {
    let exe = window_exe();
    assert_eq!(
        std::env::var("AUTO_UI_ACE_NODE_E2E").ok().as_deref(),
        Some("1"),
        "not scheduled: set AUTO_UI_ACE_NODE_E2E=1 after the prep stage"
    );
    assert!(
        exe.is_file(),
        "preparation failure: {} missing — build `--example ace_node_desktop_window` first",
        exe.display()
    );
    let src = repo_root().join("examples/ui/005-login/src/front/app.at");
    let ev = std::env::temp_dir().join(format!("plan104-005-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&ev);
    let out = std::process::Command::new(&exe)
        .arg("--source")
        .arg(&src)
        .arg("--evidence")
        .arg(&ev)
        .arg("--instances")
        .arg("1")
        .arg("--hold-ms")
        .arg("8000")
        .arg("--min-frames")
        .arg("2")
        .arg("--script")
        .arg("3:click:0:3,7:click:0:2,11:click:0:1")
        .output()
        .expect("spawn window exe");
    assert!(
        out.status.success() && ev.is_file(),
        "005 window failed: {} / {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ev).unwrap()).unwrap();
    let _ = std::fs::remove_file(&ev);
    assert_eq!(json["node"].as_str(), Some("App"));
    assert!(json["frames_rendered"].as_u64().unwrap_or(0) >= 1);
    assert!(json["errors"].as_array().map(|a| a.is_empty()).unwrap_or(false), "no lane errors: {json}");

    let session = &json["sessions"][0];
    let shown = texts(session);
    assert!(
        shown.contains("Welcome back!"),
        "login copy applied: {shown}"
    );
    // PLAN-104 第③件：提交后动态文本叶随状态刷新（错误文案进入叶值）。
    assert!(
        shown.contains("Email is required") && shown.contains("Password is required"),
        "dynamic error leaves must refresh after Submit: {shown}"
    );

    let log = json["dispatch_log"].as_array().cloned().unwrap_or_default();
    let state = |msg: &str, field: &str| -> Option<String> {
        log.iter()
            .find(|d| d["msg"].as_str() == Some(msg))
            .and_then(|d| {
                d["states"].as_array().and_then(|rows| {
                    rows.iter().find_map(|kv| {
                        let pair = kv.as_array()?;
                        (pair[0].as_str() == Some(field))
                            .then(|| pair[1].as_str().unwrap_or_default().to_string())
                    })
                })
            })
    };
    assert_eq!(
        state("Submit", "email_error").as_deref(),
        Some("Email is required"),
        "empty submit must raise the required-field error"
    );
    assert_eq!(
        state("ToggleMode", "dark_mode").as_deref(),
        Some("true"),
        "theme-mode toggle must flip dark_mode"
    );
    assert_eq!(
        state("UseSolarized", "theme_name").as_deref(),
        Some("Solarized"),
        "family switch must set theme_name"
    );
}

/// 010：表单页——循环体是**模板叶**（不进模块作用域叶），本 harness 走
/// states-only 面：真窗开窗 + Submit→`submitted=true`（列表渲染归后续刀，
/// 见计划 §11 r10）。
#[test]
fn p104_010_real_window_submit_state() {
    let exe = window_exe();
    assert_eq!(
        std::env::var("AUTO_UI_ACE_NODE_E2E").ok().as_deref(),
        Some("1"),
        "not scheduled: set AUTO_UI_ACE_NODE_E2E=1 after the prep stage"
    );
    assert!(
        exe.is_file(),
        "preparation failure: {} missing — build `--example ace_node_desktop_window` first",
        exe.display()
    );
    let src = repo_root().join("examples/ui/010-contact-form/src/front/app.at");
    let ev = std::env::temp_dir().join(format!("plan104-010-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&ev);
    let out = std::process::Command::new(&exe)
        .arg("--source")
        .arg(&src)
        .arg("--evidence")
        .arg(&ev)
        .arg("--instances")
        .arg("1")
        .arg("--hold-ms")
        .arg("6000")
        .arg("--min-frames")
        .arg("2")
        .arg("--script")
        .arg("3:click:0:0")
        .output()
        .expect("spawn window exe");
    assert!(
        out.status.success() && ev.is_file(),
        "010 window failed: {} / {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ev).unwrap()).unwrap();
    let _ = std::fs::remove_file(&ev);
    assert_eq!(json["node"].as_str(), Some("App"));
    assert!(json["frames_rendered"].as_u64().unwrap_or(0) >= 1);
    assert!(
        json["errors"].as_array().map(|a| a.is_empty()).unwrap_or(false),
        "no lane errors: {json}"
    );
    let log = json["dispatch_log"].as_array().cloned().unwrap_or_default();
    assert_eq!(log.len(), 1, "one Submit dispatch: {log:?}");
    // PLAN-104：循环体模板叶按视图数据展开（4 项 × q/a = 8 叶），真窗可读。
    let session = &json["sessions"][0];
    let shown = texts(session);
    assert_eq!(
        session["texts"][0].as_array().map(|a| a.len()).unwrap_or(0),
        8,
        "expanded FAQ leaves: {shown}"
    );
    assert!(
        shown.contains("What are your business hours?") && shown.contains("Do you offer phone support?"),
        "FAQ items rendered from view data: {shown}"
    );
    let states = log[0]["states"].as_array().cloned().unwrap_or_default();
    let submitted = states.iter().find_map(|kv| {
        let pair = kv.as_array()?;
        (pair[0].as_str() == Some("submitted"))
            .then(|| pair[1].as_str().unwrap_or_default().to_string())
    });
    assert_eq!(submitted.as_deref(), Some("true"), "Submit must set submitted");
}

/// 006：Hero 页——真窗渲染 + `dark_mode=true` 的 bool 提交（与 004 相反的极性）。
#[test]
fn p104_006_real_window_renders_hero() {
    let ev = run_window("006-hero-section", "006", "4000");
    assert_eq!(ev["node"].as_str(), Some("App"));
    assert!(
        ev["frames_rendered"].as_u64().unwrap_or(0) >= 1,
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

/// 007：数据看板——**style 面不参与 AV harness**（空白化保字节长，样式归 A2Vue），
/// 真窗呈现元素 label（`h2`/`span` 族，按状态求值）与状态行。
#[test]
fn p104_007_real_window_labels_from_state() {
    let ev = run_window("007-stats-board", "007", "4000");
    assert_eq!(ev["node"].as_str(), Some("App"));
    assert!(
        ev["errors"].as_array().map(|a| a.is_empty()).unwrap_or(false),
        "no lane errors: {ev}"
    );
    let labels: Vec<String> = ev["sessions"][0]["labels"][0]
        .as_array()
        .map(|rows| rows.iter().filter_map(|l| l.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    assert!(labels.iter().any(|l| l.contains("Dashboard")), "dashboard label: {labels:?}");
    assert!(labels.iter().any(|l| l.contains("123k")), "stat value from state: {labels:?}");
    assert!(labels.len() >= 30, "board labels all rendered (got {})", labels.len());
}