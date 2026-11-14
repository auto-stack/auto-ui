//! **测试专用夹具**（PLAN-083 T-03/T-04）——不是产品二进制。
//!
//! 多进程门禁需要"真被 spawn 的服务"：本夹具按 `FP_FIXTURE_*` 环境变量
//! 提供最小 raw HTTP 服务，并在启动时向 `FP_FIXTURE_LOG` 追加一行 pid，
//! 让宿主侧能数出真实 spawn 次数（listener 个数不能证明只 spawn 一次）。
//!
//! 无 `FP_FIXTURE_PORT` 时打印用法并以 2 退出（防止被误当产品入口执行）。
//!
//! Env:
//! - `FP_FIXTURE_PORT`（必填）：监听端口。
//! - `FP_FIXTURE_INTERFACE`：health 接口串（缺省 `notes.fpack/1`）。
//! - `FP_FIXTURE_MODE`：`ready`（缺省）|`wrong_interface`|`bad_json`|`not_ready`。
//! - `FP_FIXTURE_LOG`：spawn 事件日志（每进程一行 JSON）。
//! - `FP_FIXTURE_IDLE_MS`：空闲自杀预算（缺省 60000ms）。
//! - GET `/__fixture/exit`：测试收尾（响应后退出）。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

fn main() {
    let Some(port) = std::env::var("FP_FIXTURE_PORT")
        .ok()
        .and_then(|raw| raw.trim().parse::<u16>().ok())
    else {
        eprintln!("fp_daemon_fixture: test-only fixture; set FP_FIXTURE_PORT to run");
        std::process::exit(2);
    };
    let interface =
        std::env::var("FP_FIXTURE_INTERFACE").unwrap_or_else(|_| "notes.fpack/1".to_string());
    let mode = std::env::var("FP_FIXTURE_MODE").unwrap_or_else(|_| "ready".to_string());
    let idle_ms: u64 = std::env::var("FP_FIXTURE_IDLE_MS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(60_000);

    if let Ok(log) = std::env::var("FP_FIXTURE_LOG") {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis())
            .unwrap_or(0);
        let line = format!(
            "{{\"pid\":{},\"port\":{},\"at\":{}}}\n",
            std::process::id(),
            port,
            now
        );
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("fp_daemon_fixture: bind 127.0.0.1:{port} failed: {err}");
            std::process::exit(1);
        }
    };

    // 空闲自杀：测试异常中断时不留下孤儿监听者。
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(idle_ms));
        std::process::exit(0);
    });

    let started = Instant::now();
    for stream in listener.incoming().flatten() {
        serve(stream, &interface, &mode);
        if started.elapsed() > Duration::from_millis(idle_ms) {
            break;
        }
    }
}

fn serve(mut stream: TcpStream, interface: &str, mode: &str) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(2000)));
    let mut buf = [0u8; 2048];
    let Ok(n) = stream.read(&mut buf) else {
        return;
    };
    let request = String::from_utf8_lossy(&buf[..n]).to_string();
    if request.contains("/__fixture/exit") {
        let body = b"{\"ok\":true}";
        let _ = stream.write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            )
            .as_bytes(),
        );
        let _ = stream.write_all(body);
        let _ = stream.flush();
        std::process::exit(0);
    }
    let (status_line, body) = match mode {
        "wrong_interface" => (
            "HTTP/1.1 200 OK",
            "{\"status\":\"ok\",\"interface\":\"other.fpack/1\"}".to_string(),
        ),
        "bad_json" => ("HTTP/1.1 200 OK", "{not-json".to_string()),
        "not_ready" => ("HTTP/1.1 503 Service Unavailable", "{}".to_string()),
        _ => (
            "HTTP/1.1 200 OK",
            format!("{{\"status\":\"ok\",\"interface\":\"{interface}\"}}"),
        ),
    };
    let response = format!(
        "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}
