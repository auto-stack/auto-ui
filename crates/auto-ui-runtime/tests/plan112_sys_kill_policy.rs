//! PLAN-112 T-02 / V-02（025-sys-monitor 真实服务 + 宿主授权门）。
//!
//! 原样 `examples/ui/025-sys-monitor/src/back/api.at` 经 `back_proxy` 会话运行
//! （AutoVM + sysinfo 真实采集，源零改动）；`back_proxy::SysKillPolicy` 在会话
//! 派发**之前**拦截 `POST /api/system/kill`。
//!
//! - 正例：真实采集随刷新出现；授权 pid 结束成功；无策略 = 旧行为逐字不变。
//! - 反例：未授权具名拒（403 `sys.kill.permission_denied`），目标进程不被
//!   触碰，快照面保持健康；门只作用于声明路由；策略随 app 卸下。
//!
//! 破坏性操作一律落在**隔离自建子进程**或具名拒绝路径（Plan §2）——绝不对
//! 测试进程自身或任意既有进程发起结束。

use auto_ui_runtime::back_proxy::{
    start, BackProxyConfig, SessionSpec, SysKillPolicy, SYS_KILL_PERMISSION_DENIED,
};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const APP: &str = "025-sys-monitor";

fn app_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/025-sys-monitor")
}

/// 临时端口（同 `back_proxy_tests::ephemeral_port`：`port: 0` 是默认端口而非
/// OS 临时分配，并行用例须各持一端口）。
fn ephemeral_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind :0")
        .local_addr()
        .expect("local_addr")
        .port()
}

fn http(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(&addr).expect("connect back_proxy");
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    let payload = body.unwrap_or("");
    let req = if body.is_some() {
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
            payload.len()
        )
    } else {
        format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n")
    };
    stream.write_all(req.as_bytes()).unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).unwrap();
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    (status, body.to_string())
}

fn start_proxy() -> auto_ui_runtime::back_proxy::RunningProxy {
    start(BackProxyConfig {
        port: ephemeral_port(),
        sessions: vec![SessionSpec {
            app_id: APP.into(),
            back_entry: app_dir().join("src/back/api.at"),
        }],
        lazy_sessions: false,
        native_media: Vec::new(),
        native_photos: Vec::new(),
    })
    .expect("start back_proxy")
}

fn snapshot(port: u16) -> Value {
    let (status, body) = http(
        port,
        "GET",
        &format!("/apps/{APP}/api/system/snapshot"),
        None,
    );
    assert_eq!(status, 200, "snapshot must stay 200: {body}");
    serde_json::from_str(&body).expect("snapshot JSON")
}

fn kill(port: u16, pid: i32) -> (u16, Value) {
    let (status, body) = http(
        port,
        "POST",
        &format!("/apps/{APP}/api/system/kill"),
        Some(&format!(r#"{{"pid":{pid}}}"#)),
    );
    (status, serde_json::from_str(&body).expect("kill reply is JSON"))
}

/// 隔离自建子进程：空闲等待 30s，Drop 兜底结束——结束进程只对它发起。
struct IsolatedChild(Child);

impl IsolatedChild {
    fn spawn() -> Self {
        #[cfg(windows)]
        let child = Command::new("ping")
            .args(["-n", "30", "127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn isolated child");
        #[cfg(not(windows))]
        let child = Command::new("sleep")
            .args(["30"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn isolated child");
        Self(child)
    }

    fn pid(&self) -> i32 {
        self.0.id() as i32
    }

    fn is_alive(&mut self) -> bool {
        matches!(self.0.try_wait(), Ok(None))
    }

    fn exited_nonzero(&mut self) -> bool {
        let start = std::time::Instant::now();
        loop {
            if let Ok(Some(status)) = self.0.try_wait() {
                return !status.success();
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "isolated child {} was not terminated",
                self.0.id()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for IsolatedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 正例：真实 sysinfo 采集随刷新出现。`uptime_s` 整秒单调——相隔 ≥1.1s 的
/// 两次快照必然增大（确定性刷新 oracle）；两快照之间拉起的隔离子进程在进程
/// 表未截断时必然可见（`sys.processes()` 按 CPU 降序截断到 512——繁忙主机上
/// 空闲子进程可能落在表外，故仅在未截断时断言）。
#[test]
fn p112_snapshot_is_real_sysinfo_and_refreshes() {
    let proxy = start_proxy();
    let s1 = snapshot(proxy.port);
    assert!(s1["summary"]["cpu_count"].as_i64().unwrap_or(0) > 0, "{s1}");
    assert!(s1["summary"]["mem_total_mb"].as_i64().unwrap_or(0) > 0, "{s1}");
    assert_eq!(
        s1["cores"].as_array().map(Vec::len),
        s1["summary"]["cpu_count"].as_u64().map(|n| n as usize),
        "one per-core reading per logical core"
    );

    let mut child = IsolatedChild::spawn();
    std::thread::sleep(Duration::from_millis(1100));
    let s2 = snapshot(proxy.port);
    let up1 = s1["summary"]["uptime_s"].as_i64().unwrap();
    let up2 = s2["summary"]["uptime_s"].as_i64().unwrap();
    assert!(up2 > up1, "refresh must observe a later uptime: {up1} -> {up2}");

    let procs = s2["procs"].as_array().expect("procs array");
    assert!(!procs.is_empty(), "expected live processes from sysinfo");
    let visible = procs.iter().any(|p| p["pid"].as_i64() == Some(child.pid() as i64));
    if procs.len() < 512 {
        assert!(visible, "untruncated process table must list the isolated child");
    } else {
        eprintln!("[note] process table truncated at 512 on this host; pid-visibility branch not exercised");
    }
    assert!(child.is_alive(), "snapshot must not touch the child");
}

/// 正例/对照：未登记策略（及显式 `AllowAll`）= 旧行为——请求原样到达后端，
/// 结束进程对隔离子进程成功。授权门对既有 `auto run` 零行为变化。
#[test]
fn p112_kill_without_policy_keeps_legacy_behavior() {
    let proxy = start_proxy();
    let mut child = IsolatedChild::spawn();
    let (status, body) = kill(proxy.port, child.pid());
    assert_eq!(status, 200, "no policy: request reaches the backend: {body}");
    assert_eq!(body, Value::Bool(true), "sys.kill reports the termination");
    assert!(child.exited_nonzero(), "backend must have terminated the child");

    proxy.set_sys_kill_policy(APP, SysKillPolicy::AllowAll);
    let mut child2 = IsolatedChild::spawn();
    let (status, body) = kill(proxy.port, child2.pid());
    assert_eq!(status, 200, "explicit AllowAll lifts the gate: {body}");
    assert_eq!(body, Value::Bool(true));
    assert!(child2.exited_nonzero());
}

/// 反例：宿主不持结束进程授权（`Deny`）——具名拒、目标不被触碰、快照面保持
/// 健康；门只作用于声明路由（另一 POST 路由不经授权门）。
#[test]
fn p112_kill_denied_when_host_holds_no_authorization() {
    let proxy = start_proxy();
    proxy.set_sys_kill_policy(
        APP,
        SysKillPolicy::Deny {
            reason: "session holds no process-termination authorization".into(),
        },
    );
    let mut child = IsolatedChild::spawn();

    let (status, body) = kill(proxy.port, child.pid());
    assert_eq!(status, 403, "denied kill must be 403: {body}");
    assert_eq!(body["ok"], Value::Bool(false));
    assert_eq!(body["code"], SYS_KILL_PERMISSION_DENIED);
    assert_eq!(
        body["error"], "session holds no process-termination authorization"
    );
    assert_eq!(body["pid"], child.pid());
    std::thread::sleep(Duration::from_millis(150));
    assert!(child.is_alive(), "a denied kill must not touch the target");

    // 恢复性：拒绝之后快照面照常（UI 的数据刷新不被拒绝拖垮）。
    let snap = snapshot(proxy.port);
    assert!(!snap["procs"].as_array().unwrap().is_empty());

    // 路由范围：同 app 的另一 POST 路由不经授权门（到达后端，由后端应答）。
    let (status, body) = http(
        proxy.port,
        "POST",
        &format!("/apps/{APP}/api/system/snapshot"),
        Some("{}"),
    );
    assert_ne!(status, 403, "gate must match only the kill route: {body}");
    assert!(
        !body.contains(SYS_KILL_PERMISSION_DENIED),
        "non-kill route must not carry the policy diagnostic: {body}"
    );
}

/// 正反并列：授权清单（`AllowedPids`）——未授权 pid 具名拒且存活，授权 pid
/// 经后端真实结束。两个目标都是隔离自建子进程。
#[test]
fn p112_kill_allowed_pids_denies_ungranted_and_terminates_granted() {
    let proxy = start_proxy();
    let mut ungranted = IsolatedChild::spawn();
    let mut granted = IsolatedChild::spawn();
    proxy.set_sys_kill_policy(
        APP,
        SysKillPolicy::AllowedPids([granted.pid()].into_iter().collect()),
    );

    let (status, body) = kill(proxy.port, ungranted.pid());
    assert_eq!(status, 403, "ungranted pid must be denied: {body}");
    assert_eq!(body["code"], SYS_KILL_PERMISSION_DENIED);
    assert_eq!(body["pid"], ungranted.pid());
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains(&ungranted.pid().to_string()),
        "diagnostic names the denied pid: {body}"
    );

    let (status, body) = kill(proxy.port, granted.pid());
    assert_eq!(status, 200, "granted pid passes the gate: {body}");
    assert_eq!(body, Value::Bool(true));
    assert!(granted.exited_nonzero(), "backend must terminate the granted child");
    assert!(ungranted.is_alive(), "ungranted child must survive both requests");
}

/// 生命周期：策略随 app 供给面一并卸下——`remove_app` 后重新登记的会话不继承
/// 旧策略（回到无门旧行为）。
#[test]
fn p112_kill_policy_follows_app_lifecycle() {
    let proxy = start_proxy();
    proxy.set_sys_kill_policy(
        APP,
        SysKillPolicy::Deny {
            reason: "stale".into(),
        },
    );
    {
        let mut child = IsolatedChild::spawn();
        let (status, _) = kill(proxy.port, child.pid());
        assert_eq!(status, 403, "armed policy denies before removal");
        assert!(child.is_alive());
    }

    if let Some(join) = proxy.remove_app(APP) {
        join.join().expect("session thread exits");
    }
    proxy
        .add_session(SessionSpec {
            app_id: APP.into(),
            back_entry: app_dir().join("src/back/api.at"),
        })
        .expect("re-register the session");

    let mut child = IsolatedChild::spawn();
    let (status, body) = kill(proxy.port, child.pid());
    assert_eq!(status, 200, "policy must not outlive its app: {body}");
    assert_eq!(body, Value::Bool(true));
    assert!(child.exited_nonzero());
}
