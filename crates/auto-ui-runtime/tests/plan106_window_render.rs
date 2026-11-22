//! PLAN-106（U04 组件面真窗）：014-weather **原样源**的 AV/Iced 真窗收据。
//!
//! 入口与 980/104 同一条（`examples/ace_node_desktop_window --source <app.at>
//! --evidence <json>`，组件面桥）。本文件把以下事实钉成永久例：
//!   - 014 原样源（含 `.hourly = [\n { … },\n ]` handler 列表重写、29 处
//!     else-if 链、事件绑定载荷 `.SelectCity("beijing")`）经
//!     `compile_component_event_program` 完整装载（刀前 5085 标量门 + bind
//!     三层拒绝 → `--source` 无法装载）；
//!   - 固定负载显示（北京 23° 全字段文本在案）；
//!   - SelectCity 交互：点"上海" → `city_id/city_zh` 按原样本合同更新；
//!   - Refresh 交互：`updated_at` 14:30→14:31（样本交互收据）；
//!   - 关闭语义：close 后 late 事件被丢弃（不改旧实例），reopen 起新代次。
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

fn run_window(tag: &str, hold_ms: &str, script: Option<&str>) -> serde_json::Value {
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
        .join("examples/ui/014-weather/src/front/app.at");
    let ev = std::env::temp_dir().join(format!("plan106-window-{tag}-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&ev);
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("--source")
        .arg(&src)
        .arg("--evidence")
        .arg(&ev)
        .arg("--instances")
        .arg("1")
        .arg("--hold-ms")
        .arg(hold_ms)
        .arg("--min-frames")
        .arg("2");
    if let Some(script) = script {
        cmd.arg("--script").arg(script);
    }
    let out = cmd.output().expect("spawn window exe");
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

fn states(session: &serde_json::Value) -> std::collections::HashMap<String, String> {
    // states = 实例级行列表（`[[ [k,v], … ]]`）——取实例 0。
    session["states"]
        .as_array()
        .and_then(|insts| insts.first())
        .and_then(|row| row.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|r| {
                    let arr = r.as_array()?;
                    Some((arr[0].as_str()?.to_string(), arr[1].as_str()?.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// ①原样源装载 + 固定负载显示（北京 23° 全字段文本在案）。
#[test]
fn p106_014_original_source_loads_and_displays_fixed_payload() {
    let ev = run_window("load", "5000", None);
    assert_eq!(ev["node"].as_str(), Some("App"));
    assert!(
        ev["frames_rendered"].as_u64().unwrap_or(0) >= 1,
        "real window must render frames"
    );
    assert!(
        ev["errors"].as_array().map(|e| e.is_empty()).unwrap_or(false),
        "the original source must load without diagnostics: {:?}",
        ev["errors"]
    );
    let session = &ev["sessions"][0];
    let all = texts(session);
    for needle in ["北京", "更新于 14:30", "23°", "湿度", "演示样本"] {
        assert!(all.contains(needle), "fixed payload missing `{needle}`: {all}");
    }
    let st = states(session);
    assert_eq!(st.get("city_zh").map(|s| s.as_str()), Some("北京"));
    assert_eq!(st.get("temp").map(|s| s.as_str()), Some("23°"));
}

/// ②SelectCity 载荷交互：点"上海" → city_id/city_zh 按原样本合同更新。
#[test]
fn p106_014_select_city_updates_the_zh_label() {
    let ev = run_window("select", "7000", Some("1:click:0:10,4:click:0:5"));
    assert!(
        ev["errors"].as_array().map(|e| e.is_empty()).unwrap_or(false),
        "interactions must not error: {:?}",
        ev["errors"]
    );
    let log = ev["dispatch_log"]
        .as_array()
        .expect("dispatch log present");
    let select = log
        .iter()
        .find(|e| e["msg"].as_str() == Some("SelectCity"))
        .expect("SelectCity dispatched");
    let after: std::collections::HashMap<String, String> = select["states"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|r| {
                    let arr = r.as_array()?;
                    Some((arr[0].as_str()?.to_string(), arr[1].as_str()?.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(after.get("city_id").map(|s| s.as_str()), Some("shanghai"));
    assert_eq!(after.get("city_zh").map(|s| s.as_str()), Some("上海"));
    let refresh = log
        .iter()
        .find(|e| e["msg"].as_str() == Some("Refresh"))
        .expect("Refresh dispatched");
    assert!(refresh.to_string().contains("14:31"), "updated_at advances");
}

/// ③关闭语义：close 后 late 事件被丢弃（不改旧实例），reopen 起新代次。
#[test]
fn p106_014_close_then_late_is_dropped_and_reopen_gets_a_new_generation() {
    let ev = run_window("close", "9000", Some("1:click:0:5,3:close,5:late,7:reopen,9:click:0:5"));
    assert!(
        ev["errors"].as_array().map(|e| e.is_empty()).unwrap_or(false),
        "close/reopen must not error: {:?}",
        ev["errors"]
    );
    let sessions = ev["sessions"].as_array().expect("sessions list");
    assert!(sessions.len() >= 2, "close leaves a closed session and reopen creates one: {sessions:?}");
    let old = &sessions[0];
    assert_eq!(old["closed"].as_bool(), Some(true), "the first session is closed");
    assert!(
        old["late_dropped"].as_u64().unwrap_or(0) >= 1,
        "the late event is dropped against the closed session (stale-generation discipline)"
    );
    let new = sessions.iter().find(|s| s["closed"].as_bool() == Some(false)).expect("a live session");
    assert!(
        new["generation"].as_u64().unwrap_or(0) > old["generation"].as_u64().unwrap_or(0),
        "reopen carries a new generation"
    );
    let st = states(new);
    assert_eq!(st.get("updated_at").map(|s| s.as_str()), Some("14:32"), "the reopened session continues its own state (14:31→14:32)");
}
