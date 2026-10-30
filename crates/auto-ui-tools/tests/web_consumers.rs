//! Real UI tool VM entry keeps its caller's explicit Web configuration.
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

struct OwnedChild(std::process::Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn ui_vm_entry_installs_web_and_preserves_launch_across_worker() {
    const CHILD: &str = "PLAN890_UI_WEB_CHILD";
    if let Some(path) = std::env::var_os(CHILD) {
        let port: u16 = std::env::var("PLAN890_UI_WEB_PORT")
            .unwrap()
            .parse()
            .unwrap();
        let config = auto_web::contract::resolve_service_config(
            &format!(r#"{{"profile":"development","listen":{{"port":{port}}},"limits":{{"drain_timeout_ms":100}}}}"#), None).unwrap();
        let launch = auto_web::service_launch(std::sync::Arc::new(config));
        auto_lang_core::vm::host_capabilities::http_service::with_http_service_launch(
            Some(launch),
            || {
                auto_ui_tools::run_vm::run_entry_file(path.to_str().unwrap()).unwrap();
            },
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("api.at");
    std::fs::write(
        &source,
        "#[api(method = \"GET\", path = \"/nonce\")]\nfn nonce() str { \"ui-web-890\" }\n",
    )
    .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut child = OwnedChild(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "ui_vm_entry_installs_web_and_preserves_launch_across_worker",
                "--nocapture",
            ])
            .env(CHILD, &source)
            .env("PLAN890_UI_WEB_PORT", port.to_string())
            .env_remove("AUTO_HTTP_SERVICE_JSON")
            .env_remove("AUTO_HTTP_PORT")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "UI VM entry exited before bind"
        );
        assert!(
            Instant::now() < deadline,
            "caller launch lost at UI worker boundary"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let exchange = |wire: &[u8]| {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream.write_all(wire).unwrap();
        let mut reply = String::new();
        stream.read_to_string(&mut reply).unwrap();
        reply
    };
    let reply = exchange(b"GET /nonce HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
    assert_eq!(reply.split_once("\r\n\r\n").unwrap().1, "\"ui-web-890\"");
    let reply = exchange(b"POST /__auto/shutdown HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    assert!(reply.starts_with("HTTP/1.1 202"), "{reply}");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success(), "{status}");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "UI worker retained after shutdown"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
}
