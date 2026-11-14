//! PLAN-083 T-03/T-04：共享 provider 的多进程门禁（U-03/U-04）。
//!
//! 这些用例**真的**起多个独立宿主进程并让它们**同时**走生产
//! `resolve_manifest_host_shared` 路径；服务是测试固化夹具
//! `fp_daemon_fixture`（`CARGO_BIN_EXE_fp_daemon_fixture`），每次启动向
//! spawn 日志追加一行 pid——spawn 事件总数是可观察 oracle（listener 个数
//! 不能证明只 spawn 一次）。
//!
//! 平台：Windows 合同平台（进程间独占锁语义）；关闭时整文件空转。
#![cfg(windows)]

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const FIXTURE_EXE: &str = env!("CARGO_BIN_EXE_fp_daemon_fixture");

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "autoui-083-u03-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

fn escape_json(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}

/// 写单条目 manifest：daemon 指向夹具、端口/接口/实例键/数据根/服务 env。
fn write_manifest(root: &Path, port: u16, fixture_log: &Path, interface: &str) {
    let os_root = root.join("auto-os");
    std::fs::create_dir_all(&os_root).unwrap();
    let data_root = root.join("data");
    std::fs::create_dir_all(&data_root).unwrap();
    let manifest = format!(
        r#"{{"apps":[{{"id":"015-notes","repo":"../auto-notes","daemon":{{
            "port": {port},
            "bin": "{}",
            "env_port": "FP_FIXTURE_PORT",
            "health_interface": "{interface}",
            "ready_timeout_ms": 20000,
            "instance_key": "notes-it",
            "data_root": "{}",
            "env": {{
                "FP_FIXTURE_LOG": "{}",
                "FP_FIXTURE_INTERFACE": "{interface}",
                "FP_FIXTURE_IDLE_MS": "60000"
            }}
        }}}}]}}"#,
        escape_json(Path::new(FIXTURE_EXE)),
        escape_json(&data_root),
        escape_json(fixture_log),
    );
    std::fs::write(os_root.join("apps.manifest"), manifest).unwrap();
}

fn probe_ready(url: &str) -> bool {
    auto_ui_runtime::ui::osconfig_daemon::probe_health(
        &auto_ui_runtime::ui::osconfig_daemon::HealthSpec::declaring(
            url,
            Some("notes.fpack/1"),
            Duration::from_millis(500),
        ),
    )
    .is_ready()
}

fn http_get(url: &str, path: &str) -> Option<u16> {
    let addr: std::net::SocketAddr = url.trim_start_matches("http://").parse().ok()?;
    let mut stream = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(800)).ok()?;
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n").as_bytes(),
        )
        .ok()?;
    let mut buf = [0u8; 128];
    let n = stream.read(&mut buf).ok()?;
    String::from_utf8_lossy(&buf[..n])
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

/// 进程级 `AUTO_UI_DAEMON_SCOPE_DIR` 的测试串行化（同二进制内并行用例）。
static SCOPE_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn fixture_pids(log: &Path) -> Vec<u32> {
    let Ok(raw) = std::fs::read_to_string(log) else {
        return Vec::new();
    };
    raw.lines()
        .filter_map(|line| {
            let start = line.find("\"pid\":")? + "\"pid\":".len();
            let rest = &line[start..];
            let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            rest[..end].parse().ok()
        })
        .collect()
}

fn listening_count(port: u16) -> usize {
    let output = Command::new("netstat").args(["-ano"]).output().expect("netstat");
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter(|line| {
            line.contains(&format!("127.0.0.1:{port}"))
                && line.to_ascii_uppercase().contains("LISTENING")
        })
        .count()
}

/// 子进程 helper：`FP083_*` 缺席时是普通空转测试（父进程运行整套用例时）。
#[test]
fn host_ensure_child() {
    let Ok(result_path) = std::env::var("FP083_RESULT") else {
        return;
    };
    let parent = PathBuf::from(std::env::var("FP083_PARENT").expect("FP083_PARENT"));
    let name = std::env::var("FP083_NAME").expect("FP083_NAME");
    let barrier = PathBuf::from(std::env::var("FP083_BARRIER").expect("FP083_BARRIER"));
    // 屏障：所有宿主进程等到同一时刻再进入 ensure（真实并发冷启）。
    let deadline = Instant::now() + Duration::from_secs(30);
    while !barrier.exists() {
        if Instant::now() > deadline {
            let _ = std::fs::write(&result_path, r#"{"outcome":"offline","reason":"barrier timeout"}"#);
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let outcome = auto_ui_runtime::ui::osconfig_daemon::resolve_manifest_host_shared(
        &parent,
        &name,
        Duration::from_secs(20),
    );
    let payload = match outcome {
        auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::Ready { url, .. } => {
            format!("{{\"outcome\":\"running\",\"url\":\"{url}\"}}")
        }
        auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::Offline(reason) => {
            format!(
                "{{\"outcome\":\"offline\",\"reason\":{}}}",
                serde_json::to_string(&reason).unwrap()
            )
        }
        auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::Invalid(reason) => {
            format!(
                "{{\"outcome\":\"invalid\",\"reason\":{}}}",
                serde_json::to_string(&reason).unwrap()
            )
        }
        auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::NotDeclared => {
            "{\"outcome\":\"not_declared\"}".to_string()
        }
    };
    let _ = std::fs::write(&result_path, payload);
}

fn spawn_host_child(scope: &Path, parent: &Path, name: &str, barrier: &Path, result: &Path) -> std::process::Child {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "host_ensure_child", "--nocapture"])
        .env("AUTO_UI_DAEMON_SCOPE_DIR", scope)
        .env("FP083_PARENT", parent)
        .env("FP083_NAME", name)
        .env("FP083_BARRIER", barrier)
        .env("FP083_RESULT", result)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn host child")
}

fn wait_child(child: &mut std::process::Child, budget: Duration) -> bool {
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    false
}

fn read_result(path: &Path) -> serde_json::Value {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({"outcome":"missing","raw":raw}))
}

/// U-03：两个独立宿主进程同刻冷启同一 daemon —— 真 spawn 事件必须恰好 1 次，
/// 两者都 ready 同一实例；宿主退出后共享服务仍存活；端点只有一份监听。
#[test]
fn u03_two_hosts_concurrent_ensure_spawn_once() {
    let root = temp_root("concurrent");
    let scope = root.join("scope");
    std::fs::create_dir_all(&scope).unwrap();
    let port = free_port();
    let log = root.join("spawn.log");
    write_manifest(&root, port, &log, "notes.fpack/1");
    let barrier = root.join("go");
    let result_a = root.join("a.json");
    let result_b = root.join("b.json");

    let mut child_a = spawn_host_child(&scope, &root, "015-notes", &barrier, &result_a);
    let mut child_b = spawn_host_child(&scope, &root, "015-notes", &barrier, &result_b);
    // 两个宿主都已进入 helper、等屏障；此刻放行 = 并发冷启。
    std::thread::sleep(Duration::from_millis(400));
    std::fs::write(&barrier, b"go").unwrap();
    assert!(wait_child(&mut child_a, Duration::from_secs(60)), "宿主 A 超时");
    assert!(wait_child(&mut child_b, Duration::from_secs(60)), "宿主 B 超时");

    let a = read_result(&result_a);
    let b = read_result(&result_b);
    assert_eq!(a["outcome"], "running", "A 应 ready: {a}");
    assert_eq!(b["outcome"], "running", "B 应 ready: {b}");
    assert_eq!(a["url"], b["url"], "两宿主复用同一实例");
    let spawns = fixture_pids(&log).len();
    assert_eq!(spawns, 1, "并发冷启只允许一次 spawn（实际 {spawns} 次）");
    assert_eq!(
        listening_count(port),
        1,
        "端点只能有一份监听（{port}）"
    );

    // U-04 语义（共享服务独立于宿主窗口/进程寿命）：宿主进程都退出后仍在服务。
    assert!(
        probe_ready(&format!("http://127.0.0.1:{port}")),
        "宿主退出后共享 daemon 仍存活"
    );

    // 收尾：通知夹具退出并等待端口释放（测试自有对象）。
    let _ = http_get(&format!("http://127.0.0.1:{port}"), "/__fixture/exit");
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && listening_count(port) > 0 {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(listening_count(port), 0, "夹具已退出、端口已释放");
    let _ = std::fs::remove_dir_all(&root);
}

/// U-01 反例（R-1/F-1）：**旧语义**（无 `health_interface`）声明遇到"已 bind
/// 但先回 503、随后 200"的服务——必须像旧 `ensure_generic_io` 一样 spawn 并
/// 轮询收敛到 Running，不能被初始 503 当终局拒绝，也不能因 body 预算/spawn
/// 前探测改变旧行为。
#[test]
fn u01_legacy_declaration_converges_through_initial_503() {
    let _guard = SCOPE_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_root("legacy-503");
    let scope = root.join("scope");
    std::fs::create_dir_all(&scope).unwrap();
    let port = free_port();
    let log = root.join("spawn.log");
    let data_root = root.join("data");
    std::fs::create_dir_all(&data_root).unwrap();
    // 注意：无 health_interface（旧声明）；fixture 先回 3 次 503。
    let os_root = root.join("auto-os");
    std::fs::create_dir_all(&os_root).unwrap();
    std::fs::write(
        os_root.join("apps.manifest"),
        format!(
            r#"{{"apps":[{{"id":"015-notes","repo":"../auto-notes","daemon":{{
                "port": {port},
                "bin": "{}",
                "env_port": "FP_FIXTURE_PORT",
                "ready_timeout_ms": 20000,
                "instance_key": "notes-legacy",
                "env": {{
                    "FP_FIXTURE_LOG": "{}",
                    "FP_FIXTURE_MODE": "flaky",
                    "FP_FIXTURE_FLAKY_COUNT": "3",
                    "FP_FIXTURE_IDLE_MS": "60000"
                }}
            }}}}]}}"#,
            escape_json(Path::new(FIXTURE_EXE)),
            escape_json(&log),
        ),
    )
    .unwrap();
    std::env::set_var("AUTO_UI_DAEMON_SCOPE_DIR", &scope);

    let outcome = auto_ui_runtime::ui::osconfig_daemon::resolve_manifest_host_shared(
        &root,
        "015-notes",
        Duration::from_secs(20),
    );
    std::env::remove_var("AUTO_UI_DAEMON_SCOPE_DIR");
    let url = match outcome {
        auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::Ready { url, .. } => url,
        other => panic!("旧语义声明应经初始 503 收敛到 ready，实际 {other:?}"),
    };
    assert_eq!(fixture_pids(&log).len(), 1, "旧语义冷启仍只 spawn 一次");
    // 旧语义就绪判定 = 状态行 2xx（与 tcp_ping 同义）。
    assert!(
        auto_ui_runtime::ui::osconfig_daemon::tcp_ping(&url, Duration::from_millis(800)),
        "旧语义就绪可被 tcp_ping 观察（2xx）"
    );
    let _ = http_get(&url, "/__fixture/exit");
    let _ = std::fs::remove_dir_all(&root);
}

/// U-03 反例：相同端点、不同数据根 → 具名冲突拒绝（不得悄悄采用另一根的库）。
#[test]
fn u03_same_endpoint_different_data_root_is_rejected() {
    let _guard = SCOPE_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_root("conflict");
    let scope = root.join("scope");
    std::fs::create_dir_all(&scope).unwrap();
    let port = free_port();
    let log = root.join("spawn.log");
    write_manifest(&root, port, &log, "notes.fpack/1");
    std::env::set_var("AUTO_UI_DAEMON_SCOPE_DIR", &scope);

    // 先起一份实例（数据根 = manifest 声明根），等它就绪。
    let first =
        auto_ui_runtime::ui::osconfig_daemon::resolve_manifest_host_shared(
            &root,
            "015-notes",
            Duration::from_secs(15),
        );
    let url = match first {
        auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::Ready { url, .. } => url,
        other => panic!("首启应 ready，实际 {other:?}"),
    };
    assert!(probe_ready(&url), "实例就绪");

    // 同端点、不同数据根的第二宿主：身份不一致必须拒绝复用。
    let mut io = auto_ui_runtime::ui::osconfig_daemon::RealDaemonIo::new();
    let (repo_dir, def) =
        auto_ui_runtime::ui::app_registry::manifest_daemon_lookup(Path::new(&root), "015-notes")
            .expect("声明命中");
    let generic = auto_ui_runtime::ui::osconfig_daemon::generic_daemon_from_manifest(
        "015-notes", &repo_dir, &def,
    );
    let other_identity = auto_ui_runtime::ui::daemon_instance::InstanceIdentity::from_declaration(
        "015-notes",
        def.port,
        Some("notes-it"),
        Some("D:/other-data-root"),
    );
    let status = auto_ui_runtime::ui::daemon_instance::ensure_shared_io(
        &generic,
        &other_identity,
        &mut io,
        &auto_ui_runtime::ui::daemon_instance::SharedEnsureOptions::default(),
    );
    match status {
        auto_ui_runtime::ui::osconfig_daemon::DaemonStatus::Offline(reason) => {
            assert!(reason.contains("不同实例占用"), "{reason}");
        }
        other => panic!("不同数据根应具名冲突，实际 {other:?}"),
    }
    std::env::remove_var("AUTO_UI_DAEMON_SCOPE_DIR");

    let _ = http_get(&url, "/__fixture/exit");
    let _ = std::fs::remove_dir_all(&root);
}

/// U-03 反例：端点被错版服务占用 → 拒绝复用、不 spawn、占用者存活。
#[test]
fn u03_wrong_interface_occupant_is_not_spawned_over() {
    let _guard = SCOPE_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_root("wrong-iface");
    let scope = root.join("scope");
    std::fs::create_dir_all(&scope).unwrap();
    let port = free_port();
    let log = root.join("spawn.log");
    write_manifest(&root, port, &log, "notes.fpack/1");
    std::env::set_var("AUTO_UI_DAEMON_SCOPE_DIR", &scope);

    // 手工起一份错版夹具（independent of the runtime path）。
    let fixture = Command::new(FIXTURE_EXE)
        .env("FP_FIXTURE_PORT", port.to_string())
        .env("FP_FIXTURE_MODE", "wrong_interface")
        .env("FP_FIXTURE_LOG", &log)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("start wrong-version fixture");
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && !probe_ready(&format!("http://127.0.0.1:{port}")) {
        // 接口不匹配时 probe_ready 恒 false；用 TCP 可连作为"已占用"信号。
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    std::env::set_var("AUTO_UI_DAEMON_SCOPE_DIR", &scope);
    let mut io = auto_ui_runtime::ui::osconfig_daemon::RealDaemonIo::new();
    let (repo_dir, def) =
        auto_ui_runtime::ui::app_registry::manifest_daemon_lookup(Path::new(&root), "015-notes")
            .expect("声明命中");
    let generic = auto_ui_runtime::ui::osconfig_daemon::generic_daemon_from_manifest(
        "015-notes", &repo_dir, &def,
    );
    let identity = auto_ui_runtime::ui::daemon_instance::InstanceIdentity::from_declaration(
        "015-notes",
        def.port,
        Some("notes-it"),
        Some(root.join("data").to_string_lossy().as_ref()),
    );
    let status = auto_ui_runtime::ui::daemon_instance::ensure_shared_io(
        &generic,
        &identity,
        &mut io,
        &auto_ui_runtime::ui::daemon_instance::SharedEnsureOptions::default(),
    );
    std::env::remove_var("AUTO_UI_DAEMON_SCOPE_DIR");
    match status {
        auto_ui_runtime::ui::osconfig_daemon::DaemonStatus::Offline(reason) => {
            assert!(reason.contains("不可复用"), "{reason}");
            assert!(reason.contains("notes.fpack/2") || reason.contains("other.fpack/1"), "{reason}");
        }
        other => panic!("错版占用应 Offline，实际 {other:?}"),
    }
    assert_eq!(
        fixture_pids(&log).len(),
        1,
        "只有手工起的错版夹具启动过（运行时没有 spawn）"
    );
    assert!(
        std::net::TcpStream::connect(("127.0.0.1", port)).is_ok(),
        "占用者存活（未被清理）"
    );

    let _ = http_get(&format!("http://127.0.0.1:{port}"), "/__fixture/exit");
    let mut fixture = fixture;
    let _ = fixture.wait();
    let _ = std::fs::remove_dir_all(&root);
}
