//! PLAN-113 T-03（026-database AV 真实 SQLite 宿主桥）：V-03 冻结合同的
//! 永久例——真实临时库读/写、独立连接核对提交与回滚、失败恢复、连接释放。
//!
//! 入口与 980/104/106 同一条（`examples/ace_node_desktop_window --source
//! <026 app.at> --sqlite-db <临时库> --script ...`）。本文件钉以下事实：
//!   - 桥 evidence 与样本展示证据独立成层：preset customers 经真实库查询
//!     得 15 行真实结果集；error preset 得真实错误诊断（no such table），
//!     桥不崩、后续事务可续（失败恢复）；
//!   - SaveNewRow 真实 INSERT（首个 staging 写 BEGIN）→ CommitTx COMMIT；
//!     第二个 SaveNewRow → RollbackTx ROLLBACK；
//!   - 独立 rusqlite 连接（同受检 a2r-std 库新连接）核对：commit 后新增行
//!     可见、rollback 的行不存在、customers=16（15 种子 + 1 已提交）；
//!   - 窗口进程退出后临时库文件可删除 = 连接释放。
//!
//! 单窗口纪律：env `AUTO_UI_ACE_NODE_E2E=1` 才跑；前置
//! `cargo build --example ace_node_desktop_window --features ace-bridge`。
//! 一个用例一次开窗串行全流（脚本 settle 后早退，低于 10s 成本线）。

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

/// 表单四个输入的目标编号（buttons=12 之后；view 序：treeFilter, searchKw,
/// newName, newContact, newCity, newCountry → 14..=17）。probe 实证钉版。
const MODAL_INPUTS: [usize; 4] = [14, 15, 16, 17];

fn run_bridge_window(tag: &str, db: &Path, script: &str) -> serde_json::Value {
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
    let src = repo_root().join("examples/ui/026-database/src/front/app.at");
    let ev = std::env::temp_dir().join(format!("plan113-sqlite-{tag}-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&ev);
    let out = std::process::Command::new(&exe)
        .arg("--source")
        .arg(&src)
        .arg("--sqlite-db")
        .arg(db)
        .arg("--evidence")
        .arg(&ev)
        .arg("--instances")
        .arg("1")
        .arg("--hold-ms")
        .arg("45000")
        .arg("--min-frames")
        .arg("2")
        .arg("--script")
        .arg(script)
        // PLAN-113 r2（2026-11-23 环境破案）：ACE 装载道对 ambient Python
        // env 非 hermetic（PyO3 惰性嵌入改变受理行为——昨夜绿今夜红根因）。
        // 测试自净：子进程 env 剔除 PYTHONPATH/PYO3_PYTHON，证据可复跑。
        .env_remove("PYTHONPATH")
        .env_remove("PYO3_PYTHON")
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

fn bridge_events(evidence: &serde_json::Value) -> &[serde_json::Value] {
    evidence["sqlite_bridge"]["events"]
        .as_array()
        .expect("sqlite_bridge.events present")
}

fn events_named<'a>(evidence: &'a serde_json::Value, msg: &str) -> Vec<&'a serde_json::Value> {
    bridge_events(evidence)
        .iter()
        .filter(|e| e["msg"] == msg)
        .collect()
}

#[test]
fn p113_026_sqlite_host_bridge_realdb() {
    let dir = std::env::temp_dir().join(format!("plan113-sqlite-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("northwind.db");

    // 全流：真实查询（customers）→ 真实错误（no such table）→ 新增行提交
    // → 新增行回滚。tick 间隔 4（帧驱动， settle 后早退）。
    let [n_name, n_contact, n_city, n_country] = MODAL_INPUTS;
    let script = format!(
        "4:click:0:6,8:click:0:10,\
         12:click:0:9,16:click:0:10,\
         20:click:0:3,\
         24:type:0:{n_name}:BridgeTest Co,26:type:0:{n_contact}:Zhao Ping,28:type:0:{n_city}:Shanghai,30:type:0:{n_country}:China,\
         34:click:0:5,38:click:0:0,\
         42:click:0:3,\
         46:type:0:{n_name}:Rollback Co,48:type:0:{n_contact}:Jane Doe,50:type:0:{n_city}:Paris,52:type:0:{n_country}:France,\
         56:click:0:5,60:click:0:1"
    );
    let evidence = run_bridge_window("full", &db, &script);

    // --- 桥 evidence：真实 SQL 结果层（与样本展示层独立） ---
    let seeds = events_named(&evidence, "seed");
    assert_eq!(seeds.len(), 1, "seed once: {seeds:?}");
    assert_eq!(seeds[0]["detail"]["rows"], serde_json::json!([15, 12, 10]));

    let runs = events_named(&evidence, "RunSql");
    assert_eq!(runs.len(), 2, "two RunSql mirrors: {runs:?}");
    assert_eq!(runs[0]["detail"]["sql"], "SELECT * FROM customers LIMIT 20;");
    assert_eq!(runs[0]["detail"]["ok"], true);
    assert_eq!(runs[0]["detail"]["rows"], 15, "真实库 customers 15 行");
    assert_eq!(runs[0]["detail"]["error"], "");
    assert_eq!(runs[1]["detail"]["ok"], false, "error preset 真实失败");
    assert!(
        runs[1]["detail"]["error"]
            .as_str()
            .unwrap()
            .contains("no such table"),
        "真实错误诊断: {}",
        runs[1]["detail"]["error"]
    );

    let saves = events_named(&evidence, "SaveNewRow");
    assert_eq!(saves.len(), 2, "two SaveNewRow mirrors");
    assert_eq!(saves[0]["detail"]["ok"], true);
    assert!(saves[0]["detail"]["sql"].as_str().unwrap().contains("BridgeTest Co"));
    assert_eq!(saves[1]["detail"]["ok"], true);

    let commits = events_named(&evidence, "CommitTx");
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0]["detail"]["ok"], true);
    assert_eq!(commits[0]["detail"]["in_tx"], false);
    let rollbacks = events_named(&evidence, "RollbackTx");
    assert_eq!(rollbacks.len(), 1);
    assert_eq!(rollbacks[0]["detail"]["ok"], true);

    // --- 独立连接核对（同受检 a2r-std 库，新连接开同一文件） ---
    let check = a2r_std::sqlite::open(db.to_string_lossy().as_ref());
    a2r_std::sqlite::clear_error();
    let count = check.query("SELECT COUNT(*) FROM customers");
    assert_eq!(count[0][0], "16", "15 种子 + 1 已提交: {count:?}");
    let committed = check.query(
        "SELECT contact_name, city, country FROM customers WHERE company_name = 'BridgeTest Co'",
    );
    assert_eq!(
        committed,
        vec![vec!["Zhao Ping".to_string(), "Shanghai".to_string(), "China".to_string()]],
        "commit 后独立连接可见实际数据"
    );
    let rolled_back = check.query(
        "SELECT COUNT(*) FROM customers WHERE company_name = 'Rollback Co'",
    );
    assert_eq!(rolled_back[0][0], "0", "rollback 的行不存在");
    assert_eq!(a2r_std::sqlite::last_error(), "");

    // 其余两表种子仍在（回滚未污染）。
    assert_eq!(check.query("SELECT COUNT(*) FROM products")[0][0], "12");
    assert_eq!(check.query("SELECT COUNT(*) FROM orders")[0][0], "10");

    // --- 连接释放：窗口进程已退出（证据收口），文件可删 ---
    drop(check);
    std::fs::remove_dir_all(&dir).expect("连接释放：临时库文件可删除");
}
