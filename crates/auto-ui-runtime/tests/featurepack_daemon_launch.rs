//! PLAN-083 T-04：从**真实 `UiSession` launch 路径**汇总 U-02/U-06——
//! 声明 daemon 的 App 经 `DesktopSession::launch_app` 走生产 ensure 链，
//! 观察宿主配置归属（地址/literal 进 in-proc 会话；受管凭据不写进程全局），
//! 并用夹具的 spawn 日志计数复用（第二次 launch 不再 spawn）。
//!
//! 与 `featurepack_daemon_hosting.rs` 的分工：那边是多宿主进程的跨进程门；
//! 这里只验证"真实 launch 面"的注入与归属，不做真实 Vue/Rust 后端编译
//! （那档需要 `PLAN053_PROBE_DIR`，属 OS 联合验收边界，见计划 §6/§9）。
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const FIXTURE_EXE: &str = env!("CARGO_BIN_EXE_fp_daemon_fixture");

/// 进程级 env（AUTO_OS_ROOT / 实例目录 / 凭据键）串行化。
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "autoui-083-u06-{tag}-{}-{}",
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

fn spawn_log_lines(log: &Path) -> usize {
    std::fs::read_to_string(log)
        .map(|raw| raw.lines().filter(|line| !line.trim().is_empty()).count())
        .unwrap_or(0)
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

fn stop_fixture(port: u16) {
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    if let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)) {
        use std::io::Write;
        let _ = stream.write_all(
            b"GET /__fixture/exit HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        );
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_err() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// U-02/U-06：真实 launch 注入（地址 + 非敏感 literal）+ 受管凭据不进
/// 进程全局 + 复用在位服务不重复 spawn。
#[test]
fn u06_real_launch_injects_host_env_and_reuses_verified_instance() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = temp_root("launch");
    let os_root = root.join("auto-os");
    std::fs::create_dir_all(&os_root).unwrap();
    let scope = root.join("scope");
    std::fs::create_dir_all(&scope).unwrap();
    let data_root = root.join("data");
    std::fs::create_dir_all(&data_root).unwrap();
    let log = root.join("spawn.log");
    let port = free_port();

    // 受管 policy（OS 签发形态）：宿主凭据只应出现在子进程 env，不进 in-proc。
    let credential = "cd".repeat(32);
    let policy = data_root.join("fp-policy.json");
    std::fs::write(
        &policy,
        format!(
            r#"{{"entries":[{{"principal":"fp-reader","cred":"{credential}","operations":["read"],"book_scopes":[]}}]}}"#
        ),
    )
    .unwrap();

    std::fs::write(
        os_root.join("apps.manifest"),
        format!(
            r#"{{"apps":[{{"id":"015-notes","repo":"../auto-notes","daemon":{{
                "port": {port},
                "bin": "{}",
                "env_port": "FP_FIXTURE_PORT",
                "health_interface": "notes.fpack/1",
                "ready_timeout_ms": 20000,
                "instance_key": "notes-u06",
                "data_root": "{}",
                "env": {{ "FP_FIXTURE_LOG": "{}", "FP_FIXTURE_INTERFACE": "notes.fpack/1", "FP_FIXTURE_IDLE_MS": "60000" }},
                "host": {{
                    "env": {{ "NOTES_RETRY": "2" }},
                    "credential": {{ "env": "NOTES_FP_CRED", "path": "{}", "principal": "fp-reader" }}
                }}
            }}}}]}}"#,
            escape_json(Path::new(FIXTURE_EXE)),
            escape_json(&data_root),
            escape_json(&log),
            escape_json(&policy),
        ),
    )
    .unwrap();

    // 进程级环境按真实桌面形状准备（AUTO_OS_ROOT 权威、实例目录隔离）。
    std::env::set_var("AUTO_OS_ROOT", &os_root);
    std::env::set_var("AUTO_UI_DAEMON_SCOPE_DIR", &scope);
    std::env::remove_var("015_NOTES_DAEMON");
    std::env::remove_var("NOTES_RETRY");
    std::env::remove_var("NOTES_FP_CRED");

    let daemon_decl = Some("015-notes".to_string());
    let code = "widget NotesLaunchProbe {\n    model { var n int = 0 }\n    view { text \"${.n}\" }\n}\n";
    let mut session = auto_ui_runtime::ui::session::DesktopSession::__test_session();
    session.open_desktop(iced::window::Id::unique());
    let win = session.host.as_ref().expect("desktop").window;
    let primary = auto_ui_runtime::build_dynamic_component(
        "widget HostProbe {\n    model { var n int = 0 }\n    view { text \"${.n}\" }\n}\n",
        None,
    )
    .expect("主窗 probe");
    let primary = session.allocate_app(primary);
    session.register_window(win, primary, iced::Size::new(1280.0, 800.0));
    let decl = daemon_decl.clone();
    session.desktop.app_resolver = Some(std::sync::Arc::new(move |name: &str| {
        (name == "015-notes").then(|| auto_ui_runtime::ui::session::LaunchSpec {
            media_root: None,
            back_entry: None,
            code: code.to_string(),
            source_path: None,
            title: Some("Notes".to_string()),
            name: None,
            fit: false,
            daemon: decl.clone(),
            back_root: None,
            opens: Vec::new(),
            exe: None,
            render_decl: None,
            photo_root: None,
        })
    }));

    session
        .launch_app("015-notes")
        .unwrap_or_else(|err| panic!("launch 015-notes 失败: {err}"));

    // ① 真实 launch 后 daemon 就绪，地址注入到 in-proc 会话（旧 <NAME>_DAEMON 语义）。
    let status = format!("{:?}", session.desktop.osconfig_status);
    assert!(status.contains("Running"), "daemon 应 Running: {status}");
    assert!(
        status.contains(&format!("127.0.0.1:{port}")),
        "状态携带实例 url: {status}"
    );
    assert_eq!(
        std::env::var("015_NOTES_DAEMON").as_deref(),
        Ok(format!("http://127.0.0.1:{port}").as_str()),
        "地址键按 daemon_env_key 派生注入"
    );
    // ② 非敏感 literal 同时可见；受管凭据**不**写进程全局（R3 接口缺口）。
    assert_eq!(std::env::var("NOTES_RETRY").as_deref(), Ok("2"));
    assert!(
        std::env::var("NOTES_FP_CRED").is_err(),
        "in-proc 不得把受管凭据写入进程全局"
    );
    // ③ 服务真被拉起一次，且接口/身份核验通过。
    assert_eq!(spawn_log_lines(&log), 1, "真实 launch 首次冷启 spawn 一次");
    assert!(
        probe_ready(&format!("http://127.0.0.1:{port}")),
        "实例就绪且接口匹配"
    );

    // ④ 在位复用在真实 launch 路径同样生效（第二次不再 spawn）。
    session
        .launch_app("015-notes")
        .unwrap_or_else(|err| panic!("二次 launch 失败: {err}"));
    assert_eq!(
        spawn_log_lines(&log),
        1,
        "在位复用（第二次 launch）不得重复 spawn"
    );

    stop_fixture(port);
    std::env::remove_var("AUTO_OS_ROOT");
    std::env::remove_var("AUTO_UI_DAEMON_SCOPE_DIR");
    std::env::remove_var("015_NOTES_DAEMON");
    std::env::remove_var("NOTES_RETRY");
    let _ = std::fs::remove_dir_all(&root);
}
