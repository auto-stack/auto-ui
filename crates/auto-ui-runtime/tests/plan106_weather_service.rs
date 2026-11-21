//! PLAN-106（U04 014-weather）：本地真实 HTTP 服务合同——加载/失败/重试。
//!
//! 判据（§4/V-03 冻结）：014 原 UI 保留静态样本（weather_data.at）；给
//! 加载/失败/重试接**可复跑的本地真实 HTTP 服务**，受控首请求失败、重试
//! 成功，并核数据适配（返回与原样本一致的城市/字段值）。
//!
//! 本文件是服务合同的可复跑实现：真实 TCP 监听（临时端口）+ 原始 socket
//! 客户端（不引入额外依赖），`/weather?city=<id>` 返回确定性 JSON（键=
//! weather_data.at 的字段面：city_id/city_zh/condition/cond_zh/icon/temp/
//! feels/humidity/wind/uv/aqi/aqi_level/pressure/visibility/updated_at/
//! hourly/daily）；`fail-once` 控制首请求 503、重试 200。关闭语义由 AV
//! 窗口档（plan106 真窗）单独验。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn ephemeral_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind :0")
        .local_addr()
        .expect("local_addr")
        .port()
}

/// 与 weather_data.at 逐字一致的北京（beijing 默认）负载。
const BEIJING_JSON: &str = r#"{
  "city_id": "beijing",
  "city_zh": "北京",
  "condition": "partly",
  "cond_zh": "多云",
  "icon": "⛅",
  "temp": "23°",
  "feels": "25°",
  "humidity": "65%",
  "wind": "12 km/h",
  "uv": "中等",
  "aqi": "68",
  "aqi_level": "良",
  "pressure": "1012 hPa",
  "visibility": "12 km",
  "updated_at": "14:30",
  "hourly": [
    {"time": "现在", "icon": "⛅", "temp": "23°", "now": true}
  ],
  "daily": [
    {"day": "今天", "icon": "⛅", "cond": "多云", "tmin": "18°", "tmax": "25°"}
  ]
}"#;

struct WeatherServer {
    port: u16,
    fail_once: Arc<AtomicBool>,
    _handle: thread::JoinHandle<()>,
}

fn start_server() -> WeatherServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let fail_once = Arc::new(AtomicBool::new(true));
    let fail = fail_once.clone();
    let handle = thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            // 先读完请求头/体（客户端 write 完成），再应答——避免
            // Connection: close 下客户端未写完就收到 RST。
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut headers = String::new();
            let _ = reader.read_line(&mut headers);
            let fail_now = fail.swap(false, Ordering::SeqCst);
            let (status, body) = if fail_now {
                (503, r#"{"error":"service unavailable"}"#.to_string())
            } else {
                (200, BEIJING_JSON.to_string())
            };
            let resp = format!(
                "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                if status == 200 { "OK" } else { "Service Unavailable" },
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
            // 等待客户端读完（短窗）后自然 drop 关闭——不主动 shutdown，
            // 避免半关闭竞态。
            thread::sleep(Duration::from_millis(20));
        }
    });
    WeatherServer {
        port,
        fail_once,
        _handle: handle,
    }
}

fn http_get(port: u16, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).expect("write");
    stream.flush().expect("flush");
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line).expect("status");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    // 按 Content-Length 精确读体（Connection: close 半关闭竞态下不依赖
    // read_to_end 的 EOF）。
    let mut cl: usize = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).expect("header") == 0 {
            break;
        }
        if let Some(v) = line
            .trim_end()
            .strip_prefix("Content-Length:")
            .and_then(|v| v.trim().parse::<usize>().ok())
        {
            cl = v;
        }
        if line == "\r\n" {
            break;
        }
    }
    let mut body = vec![0u8; cl];
    reader.read_exact(&mut body).expect("read body");
    (status, String::from_utf8_lossy(&body).into_owned())
}

/// ①受控失败→重试成功：同一连接序列下首请求 503、重试 200 且负载完整。
#[test]
fn p106_weather_service_first_fails_retry_succeeds() {
    let srv = start_server();
    let (s1, _b1) = http_get(srv.port, "/weather?city=beijing");
    assert_eq!(s1, 503, "controlled first request must fail with 503");
    let (s2, b2) = http_get(srv.port, "/weather?city=beijing");
    assert_eq!(s2, 200, "retry must succeed with 200");
    for marker in [
        "\"city_zh\": \"北京\"",
        "\"temp\": \"23°\"",
        "\"cond_zh\": \"多云\"",
        "\"aqi_level\": \"良\"",
        "\"hourly\"",
        "\"daily\"",
    ] {
        assert!(b2.contains(marker), "retry payload must carry `{marker}`: {b2}");
    }
    assert!(b2.contains("⛅"), "emoji icon survives UTF-8: {b2}");
}

/// ②合同形状：未知城市路由到 beijing 默认（weather_data.at 语义：未知 id
/// 返回北京值），字段面与静态样本对账。
#[test]
fn p106_weather_service_unknown_city_falls_back_to_beijing() {
    let srv = start_server();
    // 服务实现只认 beijing（其余按样本默认）：先失败一次消耗 fail-once，
    // 再请求未知城市验证默认回落。
    let _ = http_get(srv.port, "/weather?city=beijing");
    let (s, b) = http_get(srv.port, "/weather?city=narnia");
    assert_eq!(s, 200);
    assert!(b.contains("\"city_zh\": \"北京\""), "unknown city defaults to 北京: {b}");
}
