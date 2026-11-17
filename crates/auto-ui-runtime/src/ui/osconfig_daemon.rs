//! Plan 501：os-config daemon 生命周期管理（Design 25 S7——系统 settings =
//! auto-os-config 的 UI 面）。桌面侧职责三件：检活（ping `:17701`）、按发现
//! 序 spawn、`AUTOOS_DAEMON` env 注入 App 会话。
//!
//! 分层：本文件纯逻辑（发现序/决策/env 构造，全注入式可单测）+ 进程管理
//! （spawn + ping 就绪轮询，T2）。daemon 本体在 `../auto-os-config` 仓
//! （axum，二进制 `auto-os-config-back-server`；端口经 `AUTOOS_BACK_PORT`
//! 覆盖，缺省 17901——生产约定 17701 由 [`DAEMON_PORT`] 兑现，front api.at
//! 的缺省 base 同为 17701）。桌面退出**不**杀 daemon（共享服务语义：vite/
//! 其他消费方可能复用；待澄清② v1 裁定）。
//!
//! 现场核验修正（2026-08-31）：计划原文写 `../auto-os-config/target/release/
//! auto-os-config-daemon(.exe)`，实际仓库布局为 `auto-os-config-back/target/
//! release/auto-os-config-back-server(.exe)`（Cargo.toml [[bin]] 名）——发现序
//! 按实际路径兑现，序本身不变（storage 键 > 相邻仓 target > PATH）。

use std::path::{Path, PathBuf};

/// 生产约定端口（front `back.api` 的缺省 base 同源；README :17701）。
pub const DAEMON_PORT: u16 = 17701;

/// 注入 App 会话的 env 键（os-config 仓 vm track 既有约定，
/// `auto/src/back/api.at` 的 `Env.get("AUTOOS_DAEMON")` 消费）。
pub const ENV_DAEMON: &str = "AUTOOS_DAEMON";

/// spawn 时覆盖 daemon 缺省端口（17901）用的 env 键（daemon main.rs 既有）。
pub const ENV_BACK_PORT: &str = "AUTOOS_BACK_PORT";

/// daemon 二进制名（相邻仓 `auto-os-config-back` 的 [[bin]]）。
pub const DAEMON_BIN_NAME: &str = "auto-os-config-back-server";

/// daemon 生命周期状态（面板徽标/launch 门控消费）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonStatus {
    /// 已就绪（复用既有或本会话拉起），携带 base url。
    Running(String),
    /// 拉起中（spawn 已发、就绪 ping 未过）。
    Spawning,
    /// 不可用（路径解析失败/spawn 失败/就绪超时），携带原因。
    Offline(String),
}

impl Default for DaemonStatus {
    /// 未检活（DesktopState 初始值——boot 不预检，首次 launch 时才探）。
    fn default() -> Self {
        DaemonStatus::Offline("未检活".to_string())
    }
}

/// 缺省 daemon base url（`http://127.0.0.1:<DAEMON_PORT>`）。
pub fn default_daemon_url() -> String {
    format!("http://127.0.0.1:{DAEMON_PORT}")
}

/// 从 base url 提取端口（spawn 期 `AUTOOS_BACK_PORT` 用）；
/// 非法 url 回退 [`DAEMON_PORT`]。
pub fn port_of(url: &str) -> u16 {
    url.rsplit(':')
        .next()
        .and_then(|p| p.trim_matches('/').parse().ok())
        .unwrap_or(DAEMON_PORT)
}

/// 发现序解析 daemon 可执行文件（G4）：
/// 1. `override_path`（storage 键 `shell.osconfig.daemon`）——用户显式配置，
///    原样采用（spawn 失败时原因携带该路径，不做存在性预判）；
/// 2. 相邻仓 target：`<sibling_root>/auto-os-config-back/target/release/
///    auto-os-config-back-server(.exe)`（现场核验的实际布局，存在才采用）；
/// 3. PATH 查找（`lookup_path` 注入，宿主侧接 `which` 语义）。
///
/// 全部未命中 → None（Offline 的原因之一）。
pub fn resolve_daemon_path(
    override_path: Option<&str>,
    sibling_root: &Path,
    lookup_path: impl Fn(&str) -> Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(explicit) = override_path.map(str::trim).filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(explicit));
    }
    let bin = if cfg!(windows) {
        format!("{DAEMON_BIN_NAME}.exe")
    } else {
        DAEMON_BIN_NAME.to_string()
    };
    let sibling = sibling_root
        .join("auto-os-config-back")
        .join("target")
        .join("release")
        .join(&bin);
    if sibling.is_file() {
        return Some(sibling);
    }
    lookup_path(&bin)
}

/// 检活结果 → 是否需要 spawn（G1 决策纯函数：ping 通即复用，零打扰）。
pub fn should_spawn(ping_ok: bool) -> bool {
    !ping_ok
}

/// App 会话 env 注入表（`AUTOOS_DAEMON=<url>`；os-config api.at 的
/// `daemon_base()` 消费完整 url——含 scheme）。
pub fn env_for(url: &str) -> Vec<(String, String)> {
    vec![(ENV_DAEMON.to_string(), url.to_string())]
}

// ─────────────────────────────────────────────────────────────────────────
// T2：进程管理 —— ping（std 裸 HTTP，零三方依赖；reqwest::blocking 在
// tokio 上下文线程会 panic，桌面宿主持全局 runtime，故不用）+ spawn +
// 就绪轮询状态机。执行环境经 [`DaemonIo`] 注入——单测假实现覆盖状态机
// 全分支，真实路径由 T3/T4 集成消费。
// ─────────────────────────────────────────────────────────────────────────

/// 检活/spawn 执行环境（生产 [`RealDaemonIo`]；单测假实现）。
pub trait DaemonIo {
    /// 一次检活（GET `<url>/api/health`，通 = true）。
    fn ping(&mut self, url: &str) -> bool;
    /// 拉起 daemon（detached；env 为 AUTOOS_BACK_PORT 端口覆盖）。
    fn spawn(&mut self, path: &Path, env: &[(String, String)]) -> Result<(), String>;
    /// PLAN-083 T-01：结构化健康检查。缺省由 [`DaemonIo::ping`] 派生
    /// （旧实现零改）；生产实现走 [`probe_health`]（status/body/interface）。
    fn health(&mut self, spec: &HealthSpec) -> HealthVerdict {
        if self.ping(&spec.url) {
            HealthVerdict::Ready
        } else {
            HealthVerdict::NoListener
        }
    }
}

/// url（`http://127.0.0.1:17701`）→ `127.0.0.1:17701` SocketAddr；
/// 非 loopback http 形态返回 None。
pub fn socket_addr_of(url: &str) -> Option<std::net::SocketAddr> {
    let host_port = url.strip_prefix("http://")?.split('/').next()?;
    use std::net::ToSocketAddrs;
    host_port.to_socket_addrs().ok()?.next()
}

/// 徽标投影（settings 面板召唤快照注入用）：`(state, hint)`。
/// Running → `("ready", "")`；Offline("未检活") → `("unknown", "")`——
/// boot 不预检，首次 launch 前的真实未知态；Offline(其他) →
/// `("offline", reason)`；Spawning → unknown（瞬态，面板不展示）。

/// 裸 TCP 检活：connect → `GET /api/health` → 响应行 2xx 即通。
/// std 单依赖、无 tokio TLS 上下文风险；loopback 足够（daemon 恒本地）。
pub fn tcp_ping(url: &str, timeout: std::time::Duration) -> bool {
    use std::io::{Read, Write};
    let Some(addr) = socket_addr_of(url) else {
        return false;
    };
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let req = format!("GET /api/health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = [0u8; 128];
    let Ok(n) = stream.read(&mut buf) else {
        return false;
    };
    let head = String::from_utf8_lossy(&buf[..n]);
    head.starts_with("HTTP/1.1 2") || head.starts_with("HTTP/1.0 2")
}

// ─────────────────────────────────────────────────────────────────────────
// PLAN-083 T-01：生产健康检查（D-02）——超时与响应字节预算约束下读取
// status/body，核声明接口版本；区分「无监听 / 正确服务 / 占用者不匹配 /
// 坏响应」。旧 2xx-only 语义（[`tcp_ping`]）对无新字段声明逐字保留。
// ─────────────────────────────────────────────────────────────────────────

/// 单次健康检查参数（预算冻结在调用点；不假定无限 body）。
#[derive(Debug, Clone)]
pub struct HealthSpec {
    /// `http://127.0.0.1:<port>`（非 loopback http 形态具名拒绝）。
    pub url: String,
    /// 声明接口版本；`None` = 既有 2xx 语义（只核状态行，不读 body）。
    pub interface: Option<String>,
    /// 连接/读预算（单次探测）。
    pub timeout: std::time::Duration,
    /// 响应体字节预算。
    pub max_body_bytes: usize,
}

impl HealthSpec {
    /// 缺省响应体预算（8 KiB——Notes 当前 health body 22 字节，留双层
    /// JSON 信封与未来字段余量，不按无限 body 假定）。
    pub const DEFAULT_MAX_BODY_BYTES: usize = 8192;

    /// 旧语义（无 interface 声明）——只核状态行。
    pub fn legacy(url: &str, timeout: std::time::Duration) -> Self {
        Self {
            url: url.to_string(),
            interface: None,
            timeout,
            max_body_bytes: Self::DEFAULT_MAX_BODY_BYTES,
        }
    }

    /// 声明语义——完整合法响应且 interface 逐字相等才 Ready。
    pub fn declaring(url: &str, interface: Option<&str>, timeout: std::time::Duration) -> Self {
        Self {
            url: url.to_string(),
            interface: interface.map(str::to_string),
            timeout,
            max_body_bytes: Self::DEFAULT_MAX_BODY_BYTES,
        }
    }
}

/// 健康裁决（`DaemonStatus::Offline` 的原因由 [`HealthVerdict::reason`] 具名
/// 携带；调用方不把 NotReady/BadBody 当 Ready）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthVerdict {
    /// 目标服务在位且（声明时）接口匹配。
    Ready,
    /// 端口无监听者（connect 拒绝/超预算）。
    NoListener,
    /// 端口有监听者但不是 HTTP 目标服务（非 HTTP 响应/连接后无响应）。
    OccupiedElsewhere(String),
    /// HTTP 2xx 但接口与声明不符（错版/普通占用者）——拒绝复用。
    WrongInterface {
        status: u16,
        expected: String,
        actual: Option<String>,
    },
    /// HTTP 非 2xx（占用者未就绪，不当作目标服务）。
    NotReady { status: u16 },
    /// 2xx 但响应体不能证明接口（坏 JSON/超预算/不支持的 framing）。
    BadBody(String),
}

impl HealthVerdict {
    pub fn is_ready(&self) -> bool {
        matches!(self, HealthVerdict::Ready)
    }

    /// 具名原因（日志/徽标/Offline 文案；不含任何凭据）。
    pub fn reason(&self) -> String {
        match self {
            HealthVerdict::Ready => "已就绪".to_string(),
            HealthVerdict::NoListener => "无监听".to_string(),
            HealthVerdict::OccupiedElsewhere(detail) => {
                format!("端口占用者不是目标服务（{detail}）")
            }
            HealthVerdict::WrongInterface {
                status,
                expected,
                actual,
            } => format!(
                "接口不匹配（HTTP {status}，期望 {expected}，实际 {}）",
                actual.as_deref().unwrap_or("<缺席>")
            ),
            HealthVerdict::NotReady { status } => format!("服务未就绪（HTTP {status}）"),
            HealthVerdict::BadBody(detail) => format!("health 响应非法（{detail}）"),
        }
    }
}

/// 响应头预算上限（状态行+头；防无界头）。
const HEALTH_HEAD_BUDGET: usize = 8192;

/// 原始响应（已按 framing 剥离 body）。
struct RawHealthResponse {
    status: u16,
    body: Vec<u8>,
}

/// `GET <url>/api/health`：裸 TCP + 有界读取；只支持 HTTP/1.x 的
/// Content-Length / chunked / 连接关闭三种 body 形态，其余具名拒绝。
fn read_health_response(spec: &HealthSpec) -> Result<RawHealthResponse, HealthVerdict> {
    use std::io::{Read, Write};
    let Some(addr) = socket_addr_of(&spec.url) else {
        return Err(HealthVerdict::OccupiedElsewhere(format!(
            "非 loopback http url `{}`",
            spec.url
        )));
    };
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, spec.timeout) else {
        return Err(HealthVerdict::NoListener);
    };
    let _ = stream.set_read_timeout(Some(spec.timeout));
    let _ = stream.set_write_timeout(Some(spec.timeout));
    let req = format!(
        "GET /api/health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return Err(HealthVerdict::OccupiedElsewhere(
            "请求写入失败（连接被占用者中断）".to_string(),
        ));
    }
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 1024];
    // 头部阶段：有界读取直到 \r\n\r\n。
    let head_end = loop {
        // R-2/N5：旧语义（无 interface）= 与 `tcp_ping` 逐字同义——**状态行
        // 前缀 `HTTP/1.x 2` 即就绪**，不要求完整响应头（头永不终止的 2xx 服务
        // 在旧实现里同样算通）。声明 interface 时才必须拿到完整头+body。
        if spec.interface.is_none() {
            if let Some(status) = legacy_prefix_status(&buf) {
                return Ok(RawHealthResponse {
                    status,
                    body: Vec::new(),
                });
            }
        }
        if let Some(pos) = find_head_end(&buf) {
            break pos;
        }
        if buf.len() > HEALTH_HEAD_BUDGET {
            return Err(HealthVerdict::BadBody("响应头超预算".to_string()));
        }
        match stream.read(&mut chunk) {
            Ok(0) => {
                return Err(HealthVerdict::OccupiedElsewhere(
                    "连接关闭且无完整 HTTP 响应头（非目标服务）".to_string(),
                ))
            }
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(err) => {
                let timed_out = matches!(
                    err.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                );
                if timed_out && buf.is_empty() {
                    return Err(HealthVerdict::OccupiedElsewhere(
                        "连接建立但无 HTTP 响应（占位监听者）".to_string(),
                    ));
                }
                return Err(HealthVerdict::BadBody(format!(
                    "读取响应头失败（{}）",
                    err.kind()
                )));
            }
        }
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| HealthVerdict::OccupiedElsewhere("非 HTTP 状态行".to_string()))?;
    if !head.starts_with("HTTP/1.") {
        return Err(HealthVerdict::OccupiedElsewhere(
            "响应非 HTTP/1.x".to_string(),
        ));
    }
    let headers: Vec<(String, String)> = head
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (k, v) = line.split_once(':')?;
            Some((k.trim().to_ascii_lowercase(), v.trim().to_string()))
        })
        .collect();
    let header = |name: &str| {
        headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    };
    let body_start = head_end + 4;
    let mut body_raw: Vec<u8> = buf[body_start..].to_vec();

    // PLAN-083 R-1/F-1：**旧语义（无 interface 声明）= 只看状态行**——不读
    // body、不核 framing/字节预算（与 `tcp_ping` 逐字同义：2xx 即就绪，body
    // 再大/再慢/再怪都不影响）。声明了 interface 才进入下面的有界 body 路径。
    if spec.interface.is_none() {
        return Ok(RawHealthResponse {
            status,
            body: Vec::new(),
        });
    }

    // body 形态裁定：声明不支持的 framing 直接具名拒绝（不从任意 body 假定）。
    let chunked = header("transfer-encoding")
        .map(|v| v.to_ascii_lowercase().contains("chunked"))
        .unwrap_or(false);
    let unsupported_te = header("transfer-encoding")
        .map(|v| {
            let lower = v.to_ascii_lowercase();
            !(lower.trim() == "chunked" || lower.trim() == "identity")
        })
        .unwrap_or(false);
    if unsupported_te {
        return Err(HealthVerdict::BadBody(
            "不支持的 Transfer-Encoding".to_string(),
        ));
    }
    let content_length: Option<usize> = match header("content-length") {
        Some(v) => match v.parse::<usize>() {
            Ok(n) => Some(n),
            Err(_) => return Err(HealthVerdict::BadBody("Content-Length 非法".to_string())),
        },
        None => None,
    };
    if let Some(cl) = content_length {
        if cl > spec.max_body_bytes {
            return Err(HealthVerdict::BadBody(format!(
                "响应体超预算（{cl} > {} 字节）",
                spec.max_body_bytes
            )));
        }
        while body_raw.len() < cl {
            match stream.read(&mut chunk) {
                Ok(0) => {
                    return Err(HealthVerdict::BadBody(format!(
                        "响应体截断（{} / {cl} 字节）",
                        body_raw.len()
                    )))
                }
                Ok(n) => body_raw.extend_from_slice(&chunk[..n]),
                Err(err) => {
                    return Err(HealthVerdict::BadBody(format!(
                        "读取响应体失败（{}）",
                        err.kind()
                    )))
                }
            }
            if body_raw.len() > cl {
                body_raw.truncate(cl);
            }
        }
        return Ok(RawHealthResponse {
            status,
            body: body_raw,
        });
    }
    if chunked {
        loop {
            match decode_chunked(&body_raw) {
                Ok(Some(decoded)) => {
                    if decoded.len() > spec.max_body_bytes {
                        return Err(HealthVerdict::BadBody(format!(
                            "响应体超预算（{} > {} 字节）",
                            decoded.len(),
                            spec.max_body_bytes
                        )));
                    }
                    return Ok(RawHealthResponse {
                        status,
                        body: decoded,
                    });
                }
                Ok(None) => {}
                Err(detail) => return Err(HealthVerdict::BadBody(detail)),
            }
            if body_raw.len() > spec.max_body_bytes + HEALTH_HEAD_BUDGET {
                return Err(HealthVerdict::BadBody("chunked 响应超预算".to_string()));
            }
            match stream.read(&mut chunk) {
                Ok(0) => return Err(HealthVerdict::BadBody("chunked 响应截断".to_string())),
                Ok(n) => body_raw.extend_from_slice(&chunk[..n]),
                Err(err) => {
                    return Err(HealthVerdict::BadBody(format!(
                        "读取 chunked 响应失败（{}）",
                        err.kind()
                    )))
                }
            }
        }
    }
    // 无长度声明（HTTP/1.0 或 1.1 + Connection: close）→ 读到 EOF，受预算约束。
    loop {
        if body_raw.len() > spec.max_body_bytes {
            return Err(HealthVerdict::BadBody(format!(
                "响应体超预算（> {} 字节）",
                spec.max_body_bytes
            )));
        }
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => body_raw.extend_from_slice(&chunk[..n]),
            Err(err) => {
                return Err(HealthVerdict::BadBody(format!(
                    "读取响应体失败（{}）",
                    err.kind()
                )))
            }
        }
    }
    if body_raw.len() > spec.max_body_bytes {
        return Err(HealthVerdict::BadBody(format!(
            "响应体超预算（{} > {} 字节）",
            body_raw.len(),
            spec.max_body_bytes
        )));
    }
    Ok(RawHealthResponse {
        status,
        body: body_raw,
    })
}

/// 旧语义前缀判定（R-2/N5）：与 [`tcp_ping`] 同义——首段字节以
/// `HTTP/1.1 2` 或 `HTTP/1.0 2` 开头即 2xx。能解析出完整状态码就用它，
/// 否则用 200（前缀本身已经证明 2xx）。
fn legacy_prefix_status(buf: &[u8]) -> Option<u16> {
    let text = std::str::from_utf8(buf).ok()?;
    if !(text.starts_with("HTTP/1.1 2") || text.starts_with("HTTP/1.0 2")) {
        return None;
    }
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|token| token.parse::<u16>().ok());
    Some(match status {
        Some(code) if (200..300).contains(&code) => code,
        _ => 200,
    })
}

/// 头部终止位置（`\r\n\r\n` 起点的 body 偏移 = pos+4）。
fn find_head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

/// chunked body 解码：`Ok(Some(body))` 完整；`Ok(None)` 数据不足；`Err` 非法。
fn decode_chunked(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut pos = 0usize;
    let mut out: Vec<u8> = Vec::new();
    loop {
        let line_end = match raw[pos..].windows(2).position(|w| w == b"\r\n") {
            Some(rel) => pos + rel,
            None => return Ok(None),
        };
        let line = String::from_utf8_lossy(&raw[pos..line_end]).to_string();
        let size_text = line.split(';').next().unwrap_or("").trim();
        let Ok(size) = usize::from_str_radix(size_text, 16) else {
            return Err(format!("chunked 长度行非法（`{line}`）"));
        };
        let data_start = line_end + 2;
        if size == 0 {
            // 末尾 trailer：`\r\n` 即结束；有 trailer 则等 `\r\n\r\n`。
            if raw.len() >= data_start + 2 && &raw[data_start..data_start + 2] == b"\r\n" {
                return Ok(Some(out));
            }
            if let Some(rel) = raw[data_start..].windows(4).position(|w| w == b"\r\n\r\n") {
                let _ = rel;
                return Ok(Some(out));
            }
            return Ok(None);
        }
        if raw.len() < data_start + size + 2 {
            return Ok(None);
        }
        out.extend_from_slice(&raw[data_start..data_start + size]);
        if &raw[data_start + size..data_start + size + 2] != b"\r\n" {
            return Err("chunked 数据块未以 CRLF 结束".to_string());
        }
        pos = data_start + size + 2;
        if out.len() > 1 << 20 {
            return Err("chunked 解码超 1 MiB".to_string());
        }
    }
}

/// 健康响应体解码：JSON，允许有限层字符串信封（当前 Notes
/// `JsonResponse<String>` 双层编码 = 2 层解包；超出预算/坏 JSON 具名拒绝）。
pub fn decode_health_body(body: &[u8]) -> Result<serde_json::Value, String> {
    let mut value: serde_json::Value =
        serde_json::from_slice(body).map_err(|err| format!("坏 JSON（{err}）"))?;
    for _ in 0..2 {
        match value {
            serde_json::Value::String(inner) => {
                value = serde_json::from_str(&inner)
                    .map_err(|err| format!("信封内 JSON 非法（{err}）"))?;
            }
            _ => break,
        }
    }
    Ok(value)
}

/// 一次健康探测（超时+字节预算内）：`Ready` 才允许复用/提前 ready。
pub fn probe_health(spec: &HealthSpec) -> HealthVerdict {
    let response = match read_health_response(spec) {
        Ok(response) => response,
        Err(verdict) => return verdict,
    };
    if !(200..300).contains(&response.status) {
        return HealthVerdict::NotReady {
            status: response.status,
        };
    }
    let Some(expected) = spec.interface.as_deref() else {
        // 旧语义：2xx 即就绪（不读 body，零回归）。
        return HealthVerdict::Ready;
    };
    if response.body.is_empty() {
        return HealthVerdict::BadBody("2xx 但响应体为空".to_string());
    }
    let value = match decode_health_body(&response.body) {
        Ok(value) => value,
        Err(detail) => return HealthVerdict::BadBody(detail),
    };
    let Some(obj) = value.as_object() else {
        return HealthVerdict::BadBody("health 响应体非 JSON 对象".to_string());
    };
    if let Some(status) = obj.get("status") {
        if status.as_str() != Some("ok") {
            return HealthVerdict::BadBody(format!("status 非 ok（{status}）"));
        }
    }
    let actual = obj
        .get("interface")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    if actual.as_deref() != Some(expected) {
        return HealthVerdict::WrongInterface {
            status: response.status,
            expected: expected.to_string(),
            actual,
        };
    }
    HealthVerdict::Ready
}

// ─────────────────────────────────────────────────────────────────────────
// PLAN-083 T-02：服务/宿主配置边界（D-03）——公开 manifest 只存非敏感
// literal 与受管引用；服务进程配置随 spawn 投递；宿主后端配置（地址 +
// literal + 受管凭据）只投递给**独立进程后端**（native exe / Rust API
// launcher 子进程）。in-proc VM 的 `Env.get` 是进程全局（auto-lang
// `a2r-std/src/env.rs`），凭据无法按宿主隔离——这是已记录的最小接口缺口，
// 运行时不得用"轮流设置同名全局变量"冒充隔离。
// ─────────────────────────────────────────────────────────────────────────

use auto_ui_compiler::app_registry::{ManifestDaemon, ManifestDaemonCredential};

/// 受管凭据引用（运行时形态）。值只在 [`ManagedCredentialRef::resolve`] 时
/// 出现，永不进 Debug/日志/回执。
#[derive(Clone, PartialEq, Eq)]
pub struct ManagedCredentialRef {
    /// 投递给宿主后端的 env 键（如 `NOTES_FP_CRED`）。
    pub env_key: String,
    /// 受管 policy 文件绝对/展开后路径。
    pub path: PathBuf,
    /// policy 主体（`entries[].principal`）。
    pub principal: String,
    /// 取值字段（缺省 `cred`）。
    pub field: String,
}

impl ManagedCredentialRef {
    /// manifest 引用 → 运行时引用（`path` 缺席 = `base_dir/fp-policy.json`；
    /// `%VAR%`/`$VAR` 按当前进程环境展开）。
    pub fn from_manifest(cred: &ManifestDaemonCredential, base_dir: &Path) -> Self {
        let path = cred
            .path
            .as_deref()
            .map(|raw| PathBuf::from(expand_env_path(raw)))
            .unwrap_or_else(|| base_dir.join("fp-policy.json"));
        Self {
            env_key: cred.env.clone(),
            path,
            principal: cred.principal.clone(),
            field: cred.field.clone(),
        }
    }

    /// 解析受管配置（错误信息只含主体/路径身份，不含值）。
    pub fn resolve(&self) -> Result<String, String> {
        resolve_managed_credential(&self.path, &self.principal, &self.field)
    }

    /// 非敏感诊断串（主体 + 路径 + 字段；不含凭据值）。
    pub fn diagnostic(&self) -> String {
        format!(
            "env={} principal={} field={} policy={}",
            self.env_key,
            self.principal,
            self.field,
            self.path.display()
        )
    }
}

impl std::fmt::Debug for ManagedCredentialRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ManagedCredentialRef({})", self.diagnostic())
    }
}

/// 从 OS 签发的受管 policy 文件解析凭据（`entries[].principal` 命中后取
/// `field`）。文件形态冻结为 `{"entries":[{principal,cred,operations,book_scopes}]}`
/// （OS `scripts/fp_provision_policy`）；解析失败具名，不回显文件内容/值。
pub fn resolve_managed_credential(
    path: &Path,
    principal: &str,
    field: &str,
) -> Result<String, String> {
    let raw = std::fs::read_to_string(path).map_err(|err| {
        format!(
            "受管配置不可读（{}，principal={principal}）：{err}",
            path.display()
        )
    })?;
    let doc: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| format!("受管配置非合法 JSON（{}）：{err}", path.display()))?;
    let entries = doc
        .get("entries")
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("受管配置缺 entries 数组（{}）", path.display()))?;
    let entry = entries
        .iter()
        .find(|entry| entry.get("principal").and_then(|v| v.as_str()) == Some(principal))
        .ok_or_else(|| {
            format!(
                "受管配置无 principal `{principal}` 条目（{}）",
                path.display()
            )
        })?;
    let value = entry
        .get(field)
        .and_then(|v| v.as_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "受管配置条目缺字段 `{field}`（{}，principal={principal}）",
                path.display()
            )
        })?;
    if value.chars().any(|c| c.is_control()) || value.chars().any(char::is_whitespace) {
        return Err(format!(
            "受管凭据形态非法（含空白/控制符；{} principal={principal}）",
            path.display()
        ));
    }
    if value.len() < 16 || value.len() > 512 {
        return Err(format!(
            "受管凭据长度非法（{} 字符；{} principal={principal}）",
            value.len(),
            path.display()
        ));
    }
    Ok(value.to_string())
}

/// `%VAR%`（Windows 习惯）/`$VAR`/`${VAR}` 展开；未知变量原样保留（配置
/// 诊断可见，不静默变空）。
pub fn expand_env_path(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '%' {
            if let Some(rel_end) = chars[i + 1..].iter().position(|ch| *ch == '%') {
                let name: String = chars[i + 1..i + 1 + rel_end].iter().collect();
                if let Ok(value) = std::env::var(&name) {
                    out.push_str(&value);
                    i = i + 1 + rel_end + 1;
                    continue;
                }
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == '$' {
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                if let Some(rel_end) = chars[i + 2..].iter().position(|ch| *ch == '}') {
                    let name: String = chars[i + 2..i + 2 + rel_end].iter().collect();
                    if let Ok(value) = std::env::var(&name) {
                        out.push_str(&value);
                        i = i + 2 + rel_end + 1;
                        continue;
                    }
                }
            } else {
                let rest: String = chars[i + 1..]
                    .iter()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || **ch == '_')
                    .collect();
                if !rest.is_empty() {
                    if let Ok(value) = std::env::var(&rest) {
                        out.push_str(&value);
                        i += 1 + rest.chars().count();
                        continue;
                    }
                }
            }
            out.push(c);
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// 宿主后端 env（`<NAME>_DAEMON=url` + `host.env` literal + 受管凭据）。
/// 凭据值只在 `entries` 内出现；Debug 对凭据键打码。
#[derive(Clone, Default, PartialEq, Eq)]
pub struct HostBackendEnv {
    /// 要投递的键值（顺序稳定：地址 → literal（键排序）→ 凭据）。
    pub entries: Vec<(String, String)>,
    /// 子进程必须剔除的继承键（同一键在宿主进程的历史值不得泄漏/覆盖）。
    pub remove: Vec<String>,
    /// 承载凭据的键（诊断/打码用；`None` = 无凭据声明）。
    pub credential_env: Option<String>,
}

impl HostBackendEnv {
    /// 非凭据条目（可安全用于 in-proc 进程全局注入与日志）。
    pub fn non_secret_entries(&self) -> impl Iterator<Item = (&str, &str)> {
        let credential = self.credential_env.clone();
        self.entries
            .iter()
            .filter(move |(key, _)| credential.as_deref() != Some(key.as_str()))
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    pub fn has_credentials(&self) -> bool {
        self.credential_env.is_some()
    }

    /// 是否为 key 设了值（投递断言用；不暴露值）。
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.iter().any(|(k, _)| k == key)
    }
}

impl std::fmt::Debug for HostBackendEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let redacted: Vec<(String, String)> = self
            .entries
            .iter()
            .map(|(key, value)| {
                if self.credential_env.as_deref() == Some(key.as_str()) {
                    (key.clone(), "<managed-credential>".to_string())
                } else {
                    (key.clone(), value.clone())
                }
            })
            .collect();
        f.debug_struct("HostBackendEnv")
            .field("entries", &redacted)
            .field("remove", &self.remove)
            .field("credential_env", &self.credential_env)
            .finish()
    }
}

/// 组装宿主后端 env。`daemon_env_key` 由调用方按 manifest id 派生
/// （[`daemon_env_key`]）；`base_dir` 是受管 policy 缺省基线（manifest 条目
/// repo 目录）。
pub fn host_backend_env(
    daemon_env_key: &str,
    daemon_url: &str,
    def: &ManifestDaemon,
    base_dir: &Path,
) -> Result<HostBackendEnv, String> {
    let mut entries: Vec<(String, String)> =
        vec![(daemon_env_key.to_string(), daemon_url.to_string())];
    let mut remove: Vec<String> = vec![daemon_env_key.to_string()];
    let mut credential_env: Option<String> = None;
    if let Some(host) = &def.host {
        for (key, value) in &host.env {
            if key == daemon_env_key {
                return Err(format!(
                    "daemon.host.env.{key} 与地址键冲突（地址由运行时权威提供）"
                ));
            }
            entries.push((key.clone(), value.clone()));
            remove.push(key.clone());
        }
        if let Some(cred) = &host.credential {
            let reference = ManagedCredentialRef::from_manifest(cred, base_dir);
            let value = reference
                .resolve()
                .map_err(|err| format!("宿主凭据不可用（{}）：{err}", reference.diagnostic()))?;
            entries.push((reference.env_key.clone(), value));
            remove.push(reference.env_key.clone());
            credential_env = Some(reference.env_key.clone());
        }
    }
    Ok(HostBackendEnv {
        entries,
        remove,
        credential_env,
    })
}

/// 服务进程 env：声明 `daemon.env` literal + 端口覆盖键（端口以目标 url 为
/// 权威，覆盖同名声明）。
pub fn service_env(def: &ManifestDaemon, port: u16, env_port_key: &str) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = def
        .env
        .iter()
        .filter(|(key, _)| key.as_str() != env_port_key)
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    env.push((env_port_key.to_string(), port.to_string()));
    env
}

/// PLAN-083 T-02/R-1：把宿主后端 env 应用到**子进程命令**——先剔除同名
/// 继承键（宿主历史值不得泄漏/覆盖身份），再写投递值。native exe 子进程与
/// 桌面 Rust API 后端启动器共用本函数（"配置随子进程精确投递、不写宿主
/// 进程全局"只有一处实现）。
pub fn apply_host_env(cmd: &mut std::process::Command, env: &HostBackendEnv) {
    for key in &env.remove {
        cmd.env_remove(key);
    }
    for (key, value) in &env.entries {
        cmd.env(key, value);
    }
}

/// manifest 声明 → 运行时泛化定义（`bin` 按平台补 `.exe`）。
pub fn generic_daemon_from_manifest(
    name: &str,
    repo_dir: &Path,
    def: &ManifestDaemon,
) -> GenericDaemon {
    GenericDaemon {
        name: name.to_string(),
        url: format!("http://127.0.0.1:{}", def.port),
        bin: def.bin.as_deref().map(|rel| {
            let path = repo_dir.join(rel);
            if cfg!(windows) && !path.is_file() {
                PathBuf::from(format!("{}.exe", path.display()))
            } else {
                path
            }
        }),
        env_key: daemon_env_key(name),
        env_port_key: def.env_port.clone().unwrap_or_else(|| {
            format!(
                "{}_BACK_PORT",
                daemon_env_key(name).trim_end_matches("_DAEMON")
            )
        }),
        health_interface: def.health_interface.clone(),
        ready_timeout: def.ready_timeout_ms.map(std::time::Duration::from_millis),
        extra_env: def
            .env
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    }
}

/// [`resolve_manifest_host`] 的结果。
#[derive(Debug, Clone)]
pub enum ManifestHostResolution {
    /// manifest 未声明该名（调用方按既有「未声明」诊断处理）。
    NotDeclared,
    /// 声明存在但可选能力非法（具名拒绝，不退回旧探活/不 spawn）。
    Invalid(String),
    /// 就绪且宿主配置已解析（`env` 可直接投递给独立进程后端）。
    Ready { url: String, env: HostBackendEnv },
    /// 拉起失败/就绪失败（原因具名，占用者存活）。
    Offline(String),
}

/// 生产解析入口：manifest 查表 → ensure（同一 health/预算合同）→ 宿主 env。
/// session 与桌面后端启动器共用；`parent` 语义同
/// [`auto_ui_compiler::app_registry::manifest_daemon_lookup`]。
///
/// **注意**：生产路径统一走 [`resolve_manifest_host_shared`]（带跨进程门与
/// 身份核对）；本函数是"不取锁的直接 ensure"，只保留给不需要实例协调的
/// 调用方/诊断，不构成生产入口。
pub fn resolve_manifest_host(
    parent: &Path,
    name: &str,
    fallback_timeout: std::time::Duration,
) -> ManifestHostResolution {
    let (repo_dir, def) =
        match crate::ui::app_registry::manifest_daemon_lookup_checked(parent, name) {
            Ok(Some(found)) => found,
            Ok(None) => return ManifestHostResolution::NotDeclared,
            Err(reason) => return ManifestHostResolution::Invalid(reason),
        };
    let generic = generic_daemon_from_manifest(name, &repo_dir, &def);
    match ensure_generic_io(&generic, &mut RealDaemonIo::new(), fallback_timeout) {
        DaemonStatus::Running(url) => {
            match host_backend_env(&generic.env_key, &url, &def, &repo_dir) {
                Ok(env) => ManifestHostResolution::Ready { url, env },
                Err(err) => ManifestHostResolution::Offline(err),
            }
        }
        DaemonStatus::Offline(reason) => ManifestHostResolution::Offline(reason),
        DaemonStatus::Spawning => ManifestHostResolution::Offline("daemon 仍在拉起中".to_string()),
    }
}

/// [`probe_manifest_host`] 的结果——只探不孵（不 spawn、不持锁）。
#[derive(Debug, Clone)]
pub enum ManifestHostProbe {
    NotDeclared,
    Invalid(String),
    Ready { url: String, env: HostBackendEnv },
    NotReady(String),
}

/// 桌面后端启动器的"声明 daemon"门（**单源**，R-2/F-4）：未声明 →
/// [`DeclaredBackendConfig::NotDeclared`]；声明非法 → `Invalid`（启动器必须
/// 拒绝启动其后端，不得悄悄退回"自有后端"）；就绪 → `Ready(env)`（spawn 前
/// 投递）；否则 `Offline`（既有 "Offline 不阻断" 语义，由调用方具名降级）。
///
/// 注意：本函数用 [`manifest_daemon_lookup_checked`]（**不**折叠校验错误），
/// 所以非法声明在这里一定可见——宽松入口
/// [`auto_ui_compiler::app_registry::manifest_daemon_lookup`] 会把 Err 折成
/// None，用它当守卫会让 Invalid 分支不可达。
pub fn resolve_declared_backend(
    parent: &Path,
    app_id: &str,
    fallback_timeout: std::time::Duration,
) -> DeclaredBackendConfig {
    match crate::ui::app_registry::manifest_daemon_lookup_checked(parent, app_id) {
        Ok(None) => DeclaredBackendConfig::NotDeclared,
        Err(reason) => DeclaredBackendConfig::Invalid(reason),
        Ok(Some(_)) => match resolve_manifest_host_shared(parent, app_id, fallback_timeout) {
            ManifestHostResolution::Ready { env, .. } => DeclaredBackendConfig::Ready(env),
            ManifestHostResolution::Invalid(reason) => DeclaredBackendConfig::Invalid(reason),
            ManifestHostResolution::Offline(reason) => DeclaredBackendConfig::Offline(reason),
            ManifestHostResolution::NotDeclared => DeclaredBackendConfig::NotDeclared,
        },
    }
}

/// [`resolve_declared_backend`] 的结果。
#[derive(Debug, Clone)]
pub enum DeclaredBackendConfig {
    NotDeclared,
    Invalid(String),
    Ready(HostBackendEnv),
    Offline(String),
}

/// 只读探测入口：manifest 查表 → 健康核验（同一接口合同）→ 宿主 env。
/// 未就绪返回 [`ManifestHostProbe::NotReady`]，调用方决定是否走
/// [`resolve_manifest_host`] 的完整 ensure（拉起/复用）路径。
pub fn probe_manifest_host(parent: &Path, name: &str) -> ManifestHostProbe {
    let (repo_dir, def) =
        match crate::ui::app_registry::manifest_daemon_lookup_checked(parent, name) {
            Ok(Some(found)) => found,
            Ok(None) => return ManifestHostProbe::NotDeclared,
            Err(reason) => return ManifestHostProbe::Invalid(reason),
        };
    let generic = generic_daemon_from_manifest(name, &repo_dir, &def);
    let spec = HealthSpec::declaring(
        &generic.url,
        generic.health_interface.as_deref(),
        std::time::Duration::from_millis(800),
    );
    match probe_health(&spec) {
        HealthVerdict::Ready => {
            match host_backend_env(&generic.env_key, &generic.url, &def, &repo_dir) {
                Ok(env) => ManifestHostProbe::Ready {
                    url: generic.url,
                    env,
                },
                Err(err) => ManifestHostProbe::Invalid(err),
            }
        }
        other => ManifestHostProbe::NotReady(other.reason()),
    }
}

/// 桌面本会话是否拉起过 daemon（复用判定/UX 文案；detached 语义下不持
/// Child——桌面退出不杀 daemon，句柄即刻放掉，只留"曾拉起"记录）。
static DID_SPAWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// PLAN-083 T-03：**共享 provider** ensure——跨进程单实例门（独占锁 +
/// 锁内二次探测 + 实例身份标记）+ 输家有限等待复用。与
/// [`resolve_manifest_host`] 的差别只在"谁可以 spawn"；健康/预算/接口合同
/// 与宿主 env 组装完全复用。
pub fn resolve_manifest_host_shared(
    parent: &Path,
    name: &str,
    fallback_timeout: std::time::Duration,
) -> ManifestHostResolution {
    let (repo_dir, def) =
        match crate::ui::app_registry::manifest_daemon_lookup_checked(parent, name) {
            Ok(Some(found)) => found,
            Ok(None) => return ManifestHostResolution::NotDeclared,
            Err(reason) => return ManifestHostResolution::Invalid(reason),
        };
    let generic = generic_daemon_from_manifest(name, &repo_dir, &def);
    let identity = crate::ui::daemon_instance::InstanceIdentity::from_declaration(
        name,
        def.port,
        def.instance_key.as_deref(),
        def.data_root.as_deref(),
    );
    let budget = generic.effective_ready_timeout(fallback_timeout);
    let options = crate::ui::daemon_instance::SharedEnsureOptions {
        budget,
        lock_wait: budget,
    };
    match crate::ui::daemon_instance::ensure_shared_io(
        &generic,
        &identity,
        &mut RealDaemonIo::new(),
        &options,
    ) {
        DaemonStatus::Running(url) => {
            match host_backend_env(&generic.env_key, &url, &def, &repo_dir) {
                Ok(env) => ManifestHostResolution::Ready { url, env },
                Err(err) => ManifestHostResolution::Offline(err),
            }
        }
        DaemonStatus::Offline(reason) => ManifestHostResolution::Offline(reason),
        DaemonStatus::Spawning => ManifestHostResolution::Offline("daemon 仍在拉起中".to_string()),
    }
}

/// 本会话是否曾 spawn daemon（T4 复用断言用）。

/// 真实执行环境：std TCP ping + detached spawn。
pub struct RealDaemonIo;

impl DaemonIo for RealDaemonIo {
    fn ping(&mut self, url: &str) -> bool {
        // PLAN-012 W1 T2：2s → 250ms——loopback connect 250ms 足够（daemon
        // 恒本地）；首 ping 快失败消除 UI 线程 2s 阻塞窗。spawn 后就绪轮询
        // 语义不变（每拍快失败，直至 ready_timeout 截止）。
        tcp_ping(url, std::time::Duration::from_millis(250))
    }

    /// PLAN-083 T-01：生产健康检查走完整 status/body/interface 核验
    /// （预算由 [`HealthSpec`] 携带；失败原因具名，不含凭据）。
    fn health(&mut self, spec: &HealthSpec) -> HealthVerdict {
        probe_health(spec)
    }

    fn spawn(&mut self, path: &Path, env: &[(String, String)]) -> Result<(), String> {
        let mut cmd = std::process::Command::new(path);
        for (k, v) in env {
            cmd.env(k, v);
        }
        // detached：与桌面生命周期解耦（共享服务语义，待澄清② v1 裁定）。
        // stdio 全 null——无控制台形态下防句柄悬挂/闪窗。
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        cmd.stdin(std::process::Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            // DETACHED_PROCESS：脱离桌面控制台；CREATE_NEW_PROCESS_GROUP：
            // 独立进程组，桌面组的 Ctrl+C 不传播。
            cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        }
        cmd.spawn()
            .map(|child| {
                DID_SPAWN.store(true, std::sync::atomic::Ordering::Relaxed);
                // detached 语义：立即放句柄（不 kill、不 wait——桌面退出
                // daemon 存活；退出码回收交 OS）。
                std::mem::drop(child);
            })
            .map_err(|e| e.to_string())
    }
}

impl RealDaemonIo {
    /// no-op 占位（struct 无字段；保留具名构造便于读点）。
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealDaemonIo {
    fn default() -> Self {
        Self::new()
    }
}

/// 相邻仓探测缺省根（`../auto-os-config`，CWD 相对——vm 桌面开发机形态
/// 即从本仓根起 `auto run`；storage `shell.apps.scan_siblings` 可关整段
/// 探测，见 app_registry 聚合）。
pub fn default_sibling_root() -> PathBuf {
    PathBuf::from("..").join("auto-os-config")
}

/// storage 键 `shell.osconfig.daemon`（G4 发现序第 1 级；缺席/空 = None）。
pub fn daemon_path_override() -> Option<String> {
    auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.osconfig.daemon")
        .filter(|s| !s.trim().is_empty())
}

/// Plan 505 B4（债 P501-1）：发现序第 3 级 PATH 查找（which 语义，纯逻辑
/// 可注入单测）——逐目录 is_file 探测；Unix 附加可执行位判断（Windows 无
/// 执行位概念，存在即候选，不可执行由 spawn 失败兜底）。
pub fn which_in(bin: &str, dirs: impl Iterator<Item = PathBuf>) -> Option<PathBuf> {
    for dir in dirs {
        let cand = dir.join(bin);
        if !cand.is_file() {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let Ok(md) = cand.metadata() else { continue };
            if md.permissions().mode() & 0o111 == 0 {
                continue;
            }
        }
        return Some(cand);
    }
    None
}

/// PATH 环境变量的 which 包装（生产发现序第 3 级注入缝兑现）。
pub fn which_daemon(bin: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    which_in(bin, std::env::split_paths(&path_var))
}

/// G1 主流程：检活 → 未运行则按发现序 spawn → 就绪轮询 ≤`ready_timeout`
/// （200ms 间隔）→ `Running(url)` / `Offline(reason)`。已运行 daemon 零打扰
/// 复用（不 spawn）；spawn 只带 `AUTOOS_BACK_PORT=<port>`（兑现生产端口
/// 17701——daemon 缺省 17901）。发现序三级：storage 覆盖 > 相邻仓 target >
/// PATH（`lookup_path` 注入——生产 [`which_daemon`]，Plan 505 B4 兑现）。
pub fn ensure_ready_io(
    url: &str,
    override_path: Option<&str>,
    sibling_root: &Path,
    lookup_path: impl Fn(&str) -> Option<PathBuf>,
    io: &mut dyn DaemonIo,
    ready_timeout: std::time::Duration,
) -> DaemonStatus {
    if io.ping(url) {
        return DaemonStatus::Running(url.to_string());
    }
    let Some(path) = resolve_daemon_path(override_path, sibling_root, lookup_path) else {
        return DaemonStatus::Offline(
            "daemon 可执行未找到（发现序：shell.osconfig.daemon > ../auto-os-config 相邻仓 target > PATH）".to_string(),
        );
    };
    let env = vec![(ENV_BACK_PORT.to_string(), port_of(url).to_string())];
    if let Err(err) = io.spawn(&path, &env) {
        return DaemonStatus::Offline(format!("spawn {} 失败: {err}", path.display()));
    }
    let deadline = std::time::Instant::now() + ready_timeout;
    while std::time::Instant::now() < deadline {
        if io.ping(url) {
            return DaemonStatus::Running(url.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    DaemonStatus::Offline(format!(
        "daemon 就绪超时（{}ms 内 ping 不通 {url}；{}）",
        ready_timeout.as_millis(),
        path.display()
    ))
}

/// 生产入口（缺省参数组：17701 + storage 覆盖 + 相邻仓根 + PATH which +
/// 5s 就绪）。
pub fn ensure_ready(url: &str) -> DaemonStatus {
    ensure_ready_io(
        url,
        daemon_path_override().as_deref(),
        &default_sibling_root(),
        which_daemon,
        &mut RealDaemonIo::new(),
        std::time::Duration::from_secs(5),
    )
}

// ─────────────────────────────────────────────────────────────────────────
// PLAN-013 T2：per-name 泛化 ensure——manifest `daemon` 字段驱动的非 autoos
// daemon（musk/jade-garden 等）。autoos 名不走此段（D4 零回归门：既有链与
// 测试原样）。健康探针契约固定 `GET /api/health`（[`DaemonIo::ping`] 接口
// 不变）；轮询循环与 [`ensure_ready_io`] 同构（复制不抽共享——旧函数体
// 零触碰优先于 DRY）。
// ─────────────────────────────────────────────────────────────────────────

/// 泛化 daemon 定义（session 层自 manifest `daemon` 字段组装）。
pub struct GenericDaemon {
    /// 可读名（Offline 原因与日志用；同 manifest id）。
    pub name: String,
    /// API base url（`http://127.0.0.1:<port>`）。
    pub url: String,
    /// 二进制绝对路径；`None` = 只探不孵（探不通信即 Offline 有因）。
    pub bin: Option<PathBuf>,
    /// App 会话注入 env 键（`<NAME 大写蛇形>_DAEMON = url`）。
    pub env_key: String,
    /// spawn 期端口覆盖 env 键（各 daemon 自有约定，如 `JADE_GARDEN_PORT`）。
    pub env_port_key: String,
    /// PLAN-083 T-01：声明接口版本（`None` = 既有 2xx 语义，零回归）。
    pub health_interface: Option<String>,
    /// PLAN-083 T-01：声明就绪预算（`None` = 调用方缺省 5s）。
    pub ready_timeout: Option<std::time::Duration>,
    /// PLAN-083 T-02：服务进程额外 env（manifest `daemon.env` 非敏感 literal；
    /// 端口键由 [`GenericDaemon::spawn_env`] 追加并覆盖）。
    pub extra_env: Vec<(String, String)>,
}

impl GenericDaemon {
    /// PLAN-083 T-01：本次就绪预算（manifest 声明优先于调用方缺省）。
    pub fn effective_ready_timeout(&self, fallback: std::time::Duration) -> std::time::Duration {
        self.ready_timeout.unwrap_or(fallback)
    }

    /// PLAN-083 T-02：spawn 投递 env（声明 literal + 端口覆盖键；端口以目标
    /// url 为权威，同名声明被覆盖）。
    pub fn spawn_env(&self) -> Vec<(String, String)> {
        let port_key = self.env_port_key.as_str();
        let mut env: Vec<(String, String)> = self
            .extra_env
            .iter()
            .filter(|(key, _)| key != port_key)
            .cloned()
            .collect();
        env.push((port_key.to_string(), port_of(&self.url).to_string()));
        env
    }
}

/// manifest id → 注入 env 键派生（`auto-musk → AUTO_MUSK_DAEMON`；
/// `autoos → AUTOOS_DAEMON` 与 [`ENV_DAEMON`] 同构——D3 派生规则对旧名
/// 恒等，语义单源）。
pub fn daemon_env_key(name: &str) -> String {
    format!("{}_DAEMON", name.to_ascii_uppercase().replace('-', "_"))
}

/// 泛化主流程：检活 → bin 缺席判定 → spawn（env 仅端口覆盖键）→ 就绪轮询
/// ≤`ready_timeout`（200ms 间隔）。语义与 [`ensure_ready_io`] 一致：已运行
/// 零打扰复用；Offline 不阻断 launch（调用方记徽标）。
///
/// PLAN-083 T-01：`def.health_interface` 存在时走 [`ensure_generic_strict`]
/// （完整 health 核验）；缺席时逐字保留本函数的既有 2xx 语义（D-01 零回归）。
pub fn ensure_generic_io(
    def: &GenericDaemon,
    io: &mut dyn DaemonIo,
    ready_timeout: std::time::Duration,
) -> DaemonStatus {
    if def.health_interface.is_some() {
        return ensure_generic_strict(def, io, ready_timeout);
    }
    if io.ping(&def.url) {
        return DaemonStatus::Running(def.url.clone());
    }
    let Some(path) = &def.bin else {
        return DaemonStatus::Offline(format!(
            "daemon `{}` 未配置 bin（apps.manifest daemon.bin 缺席，只探不孵；ping {} 不通）",
            def.name, def.url
        ));
    };
    let env = def.spawn_env();
    if let Err(err) = io.spawn(path, &env) {
        return DaemonStatus::Offline(format!(
            "daemon `{}` spawn {} 失败: {err}",
            def.name,
            path.display()
        ));
    }
    let deadline = std::time::Instant::now() + ready_timeout;
    while std::time::Instant::now() < deadline {
        if io.ping(&def.url) {
            return DaemonStatus::Running(def.url.clone());
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    DaemonStatus::Offline(format!(
        "daemon `{}` 就绪超时（{}ms 内 ping 不通 {}；{}）",
        def.name,
        ready_timeout.as_millis(),
        def.url,
        path.display()
    ))
}

/// PLAN-083 T-01/D-02：声明接口版本的严格 ensure。
///
/// 冻结语义：
/// 1. spawn 前先做一次完整健康检查（预算 800ms）——`Ready` 复用；`NoListener`
///    才允许 spawn；`WrongInterface`/`NotReady`/`BadBody`/`OccupiedElsewhere`
///    一律具名 Offline，**不 spawn**（占用者存活，避免无意义 spawn 与向非目标
///    服务发送配置/凭据）。
/// 2. spawn 后按同一接口轮询（每拍 250ms，200ms 间隔）：只有 `Ready` 结束；
///    `WrongInterface` 立即 Offline（2xx 早到不算 ready）；其余继续等待到预算
///    截止，超时原因携带最后一次裁决。
pub fn ensure_generic_strict(
    def: &GenericDaemon,
    io: &mut dyn DaemonIo,
    fallback_timeout: std::time::Duration,
) -> DaemonStatus {
    let budget = def.effective_ready_timeout(fallback_timeout);
    let interface = def.health_interface.as_deref();
    let precheck =
        HealthSpec::declaring(&def.url, interface, std::time::Duration::from_millis(800));
    match io.health(&precheck) {
        HealthVerdict::Ready => return DaemonStatus::Running(def.url.clone()),
        HealthVerdict::NoListener => {}
        other => {
            return DaemonStatus::Offline(format!(
                "daemon `{}` 端口 {} 不可复用：{}（占用者存活，未 spawn）",
                def.name,
                port_of(&def.url),
                other.reason()
            ))
        }
    }
    let Some(path) = &def.bin else {
        return DaemonStatus::Offline(format!(
            "daemon `{}` 未配置 bin（apps.manifest daemon.bin 缺席，只探不孵；health {} 不通）",
            def.name, def.url
        ));
    };
    let env = def.spawn_env();
    if let Err(err) = io.spawn(path, &env) {
        return DaemonStatus::Offline(format!(
            "daemon `{}` spawn {} 失败: {err}",
            def.name,
            path.display()
        ));
    }
    let poll = HealthSpec::declaring(&def.url, interface, std::time::Duration::from_millis(250));
    let deadline = std::time::Instant::now() + budget;
    let mut last = HealthVerdict::NoListener;
    loop {
        match io.health(&poll) {
            HealthVerdict::Ready => return DaemonStatus::Running(def.url.clone()),
            HealthVerdict::WrongInterface {
                status,
                expected,
                actual,
            } => {
                return DaemonStatus::Offline(format!(
                    "daemon `{}` 就绪期接口不匹配（HTTP {status}，期望 {expected}，实际 {}；拒绝复用 {}）",
                    def.name,
                    actual.as_deref().unwrap_or("<缺席>"),
                    path.display()
                ))
            }
            other => last = other,
        }
        if std::time::Instant::now() >= deadline {
            return DaemonStatus::Offline(format!(
                "daemon `{}` 就绪超时（{}ms；最后裁决：{}；{}）",
                def.name,
                budget.as_millis(),
                last.reason(),
                path.display()
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sibling_fixture() -> PathBuf {
        // 临时相邻仓根：释放 release/auto-os-config-back-server(.exe)。
        // PLAN-013 T2：路径唯一化（进程内原子计数）——固定共享路径在并行
        // 测试下互相 remove_dir_all 拆台（013 实测：新测试拉大并行窗口后
        // 间歇性失败子集每轮不同），非生产行为变化。
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "autoui-501-daemon-fixture-{}-{n}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(
            root.join("auto-os-config-back")
                .join("target")
                .join("release"),
        )
        .unwrap();
        root
    }

    fn touch_server_exe(root: &Path) -> PathBuf {
        let bin = root
            .join("auto-os-config-back")
            .join("target")
            .join("release")
            .join(if cfg!(windows) {
                format!("{DAEMON_BIN_NAME}.exe")
            } else {
                DAEMON_BIN_NAME.to_string()
            });
        std::fs::write(&bin, b"stub").unwrap();
        bin
    }

    #[test]
    fn resolve_order_explicit_wins() {
        let root = sibling_fixture();
        let exe = touch_server_exe(&root);
        // 显式配置优先——即便相邻仓 target 也存在。
        let got = resolve_daemon_path(Some("D:/custom/daemon.exe"), &root, |_| {
            panic!("PATH 不应触达")
        });
        assert_eq!(got, Some(PathBuf::from("D:/custom/daemon.exe")));
        // 空白串视为缺席（storage 坏值容错），落相邻仓。
        let got = resolve_daemon_path(Some("  "), &root, |_| None);
        assert_eq!(got, Some(exe));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_order_sibling_target_then_path() {
        // 相邻仓无产物 → PATH 兜底。
        let root = sibling_fixture();
        let got = resolve_daemon_path(None, &root, |name| {
            assert_eq!(name, format!("{DAEMON_BIN_NAME}.exe").as_str());
            Some(PathBuf::from("/usr/bin/found"))
        });
        assert_eq!(got, Some(PathBuf::from("/usr/bin/found")));
        // 相邻仓有产物 → PATH 不触达。
        let exe = touch_server_exe(&root);
        let got = resolve_daemon_path(None, &root, |_| panic!("PATH 不应触达"));
        assert_eq!(got, Some(exe));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_all_miss_is_none() {
        let root = sibling_fixture();
        assert_eq!(resolve_daemon_path(None, &root, |_| None), None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn should_spawn_decision() {
        assert!(!should_spawn(true), "ping 通 = 复用既有，零打扰");
        assert!(should_spawn(false), "ping 不通 = 走 spawn");
    }

    #[test]
    fn env_injection_shape() {
        let env = env_for("http://127.0.0.1:17708");
        assert_eq!(
            env,
            vec![(
                "AUTOOS_DAEMON".to_string(),
                "http://127.0.0.1:17708".to_string()
            )]
        );
    }

    #[test]
    fn url_port_helpers() {
        assert_eq!(default_daemon_url(), "http://127.0.0.1:17701");
        assert_eq!(port_of("http://127.0.0.1:17701"), 17701);
        assert_eq!(port_of("http://127.0.0.1:17708"), 17708);
        assert_eq!(port_of("bad"), DAEMON_PORT, "非法 url 回退缺省端口");
    }

    // ---- T2：检活/spawn 状态机（注入式假 IO）+ 真实 TCP ping ----

    /// 假执行环境：ping 按脚本表走（每调一次弹一个）；spawn 记录参数。
    struct FakeIo {
        ping_script: Vec<bool>,
        ping_calls: usize,
        spawn_result: Result<(), String>,
        spawn_path: Option<PathBuf>,
        spawn_env: Vec<(String, String)>,
        /// PLAN-083 T-01：结构化健康脚本（Some 时 health 走脚本，否则回退 ping）。
        health_script: Option<Vec<HealthVerdict>>,
        health_calls: usize,
        spawn_calls: usize,
    }

    impl FakeIo {
        fn new(ping_script: Vec<bool>) -> Self {
            Self {
                ping_script,
                ping_calls: 0,
                spawn_result: Ok(()),
                spawn_path: None,
                spawn_env: Vec::new(),
                health_script: None,
                health_calls: 0,
                spawn_calls: 0,
            }
        }

        fn with_health(mut self, script: Vec<HealthVerdict>) -> Self {
            self.health_script = Some(script);
            self
        }
    }

    impl DaemonIo for FakeIo {
        fn ping(&mut self, _url: &str) -> bool {
            let i = self.ping_calls.min(self.ping_script.len() - 1);
            self.ping_calls += 1;
            self.ping_script[i]
        }
        fn spawn(&mut self, path: &Path, env: &[(String, String)]) -> Result<(), String> {
            self.spawn_calls += 1;
            self.spawn_path = Some(path.to_path_buf());
            self.spawn_env = env.to_vec();
            self.spawn_result.clone()
        }
        fn health(&mut self, spec: &HealthSpec) -> HealthVerdict {
            match &self.health_script {
                Some(script) => {
                    let i = self.health_calls.min(script.len() - 1);
                    self.health_calls += 1;
                    script[i].clone()
                }
                None => {
                    if self.ping(&spec.url) {
                        HealthVerdict::Ready
                    } else {
                        HealthVerdict::NoListener
                    }
                }
            }
        }
    }

    #[test]
    fn ensure_ready_reuses_running_daemon() {
        let mut io = FakeIo::new(vec![true]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            Path::new("Z:/nowhere"),
            |_| None,
            &mut io,
            std::time::Duration::from_secs(1),
        );
        assert_eq!(
            st,
            DaemonStatus::Running("http://127.0.0.1:17701".to_string())
        );
        assert_eq!(io.ping_calls, 1, "ping 通即复用");
        assert!(io.spawn_path.is_none(), "已运行 daemon 零打扰——不 spawn");
    }

    /// PLAN-012 W1 T2：RealDaemonIo 首 ping 快失败参数化——未监听端口
    /// 上 ping 必须在 250ms 档超时（实测墙钟 <1s，即 UI 线程阻塞上限），
    /// Offline 语义不变（false）。
    #[test]
    fn real_daemon_ping_short_timeout_fast_fail() {
        let started = std::time::Instant::now();
        let ok = RealDaemonIo::new().ping("http://127.0.0.1:9");
        let elapsed = started.elapsed();
        // 端口 9（discard）默认无监听 → connect 快失败；墙钟须远小于旧 2s 档。
        assert!(!ok, "无监听端口应 ping 不通");
        assert!(
            elapsed < std::time::Duration::from_millis(1000),
            "首 ping 应 ≤250ms 档快失败（实测 {elapsed:?}）"
        );
    }

    /// Plan 505 B4（债 P501-1）：which_in 纯逻辑——目录序首命中、全空 None。
    #[test]
    fn which_in_scans_dirs_in_order() {
        let root = std::env::temp_dir().join("autoui-505-which-fixture");
        let _ = std::fs::remove_dir_all(&root);
        let hit = root.join("b");
        std::fs::create_dir_all(&hit).unwrap();
        let bin = if cfg!(windows) {
            "daemon-stub.exe"
        } else {
            "daemon-stub"
        };
        std::fs::write(hit.join(bin), b"stub").unwrap();
        let dirs = vec![root.join("a"), hit.clone(), root.join("c")];
        assert_eq!(which_in(bin, dirs.into_iter()), Some(hit.join(bin)));
        assert_eq!(
            which_in("no-such-bin.out", [root.join("a")].into_iter()),
            None
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Plan 505 B4：ensure_ready 发现序第 3 级——相邻仓无产物时 PATH 兜底
    /// （spawn 路径 = lookup 命中值）。
    #[test]
    fn ensure_ready_falls_back_to_path_tier() {
        let mut io = FakeIo::new(vec![false; 10]);
        io.spawn_result = Err("boom".to_string());
        let path_hit = PathBuf::from(if cfg!(windows) {
            "C:\\somewhere\\daemon.exe"
        } else {
            "/somewhere/daemon"
        });
        let expect = path_hit.clone();
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            Path::new("Z:/nowhere"),
            move |_| Some(expect.clone()),
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => assert!(reason.contains("spawn"), "{reason}"),
            other => panic!("应 Offline，实际 {other:?}"),
        }
        assert_eq!(io.spawn_path, Some(path_hit), "发现序落 PATH 第三级");
    }

    #[test]
    fn ensure_ready_offline_when_binary_not_found() {
        let mut io = FakeIo::new(vec![false; 10]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            Path::new("Z:/nowhere"),
            |_| None,
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("未找到"), "原因应说明路径未找到: {reason}")
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
        assert!(io.spawn_path.is_none(), "无路径不 spawn");
    }

    #[test]
    fn ensure_ready_offline_when_spawn_fails() {
        let root = sibling_fixture();
        let exe = touch_server_exe(&root);
        let mut io = FakeIo::new(vec![false; 10]);
        io.spawn_result = Err("拒绝访问".to_string());
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            &root,
            |_| None,
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(
                    reason.contains("spawn") && reason.contains("拒绝访问"),
                    "{reason}"
                )
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
        assert_eq!(io.spawn_path, Some(exe), "发现序落相邻仓 target");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_ready_spawns_then_ready_with_port_env() {
        let root = sibling_fixture();
        touch_server_exe(&root);
        // ping 脚本：首次 false（触发 spawn）→ 之后 true（就绪）。
        let mut io = FakeIo::new(vec![false, true, true]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            &root,
            |_| None,
            &mut io,
            std::time::Duration::from_secs(5),
        );
        assert_eq!(
            st,
            DaemonStatus::Running("http://127.0.0.1:17701".to_string())
        );
        // spawn env 只带端口覆盖（生产 17701——daemon 缺省 17901 需显式改）。
        assert_eq!(
            io.spawn_env,
            vec![(ENV_BACK_PORT.to_string(), "17701".to_string())]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_ready_offline_on_ready_timeout() {
        let root = sibling_fixture();
        touch_server_exe(&root);
        // spawn 成功但 ping 恒不通 → 就绪超时（ready_timeout 压到 1ms 免慢测）。
        let mut io = FakeIo::new(vec![false; 10]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17701",
            None,
            &root,
            |_| None,
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => assert!(reason.contains("就绪超时"), "{reason}"),
            other => panic!("应 Offline，实际 {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_ready_override_path_used_for_spawn() {
        let root = sibling_fixture();
        touch_server_exe(&root);
        let mut io = FakeIo::new(vec![false, true, true]);
        let st = ensure_ready_io(
            "http://127.0.0.1:17708",
            Some("D:/tools/custom-daemon.exe"),
            &root,
            |_| None,
            &mut io,
            std::time::Duration::from_secs(5),
        );
        assert_eq!(
            st,
            DaemonStatus::Running("http://127.0.0.1:17708".to_string())
        );
        assert_eq!(
            io.spawn_path,
            Some(PathBuf::from("D:/tools/custom-daemon.exe"))
        );
        assert_eq!(
            io.spawn_env,
            vec![(ENV_BACK_PORT.to_string(), "17708".to_string())],
            "端口取自目标 url"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 迷你 HTTP 服务（单线程 accept 循环，响应 200）——真实 tcp_ping 正路径。
    fn mini_health_server() -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut s = stream;
                let mut buf = [0u8; 512];
                let _ = s.read(&mut buf);
                let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}");
            }
        });
        url
    }

    #[test]
    fn tcp_ping_real_server_and_dead_port() {
        let url = mini_health_server();
        assert!(
            tcp_ping(&url, std::time::Duration::from_secs(2)),
            "真服务 200 应通"
        );
        // 死端口：bind 后立即 drop → connect 拒绝。
        let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let dead_url = format!("http://127.0.0.1:{}", dead.local_addr().unwrap().port());
        drop(dead);
        assert!(
            !tcp_ping(&dead_url, std::time::Duration::from_secs(2)),
            "死端口应不通"
        );
        // 非 http 形态 → false（不 panic）。
        assert!(!tcp_ping("not-a-url", std::time::Duration::from_secs(1)));
        assert!(socket_addr_of("http://127.0.0.1:17701").is_some());
        assert!(socket_addr_of("ftp://x").is_none());
    }

    #[test]
    fn real_io_ping_wires_tcp_ping() {
        // RealDaemonIo::ping 即 tcp_ping（2s 档）——经 trait 走一遍真路径。
        let url = mini_health_server();
        let mut io = RealDaemonIo::new();
        assert!(io.ping(&url));
    }

    // ---- PLAN-013 T2：per-name 泛化 ensure ----

    #[test]
    fn daemon_env_key_derivation_matches_legacy() {
        // D3 派生规则对旧名恒等——AUTOOS_DAEMON 语义单源。
        assert_eq!(daemon_env_key("autoos"), ENV_DAEMON);
        assert_eq!(daemon_env_key("auto-musk"), "AUTO_MUSK_DAEMON");
        assert_eq!(daemon_env_key("jade-garden"), "JADE_GARDEN_DAEMON");
    }

    fn generic_def(bin: Option<PathBuf>) -> GenericDaemon {
        GenericDaemon {
            name: "auto-musk".to_string(),
            url: "http://127.0.0.1:17201".to_string(),
            bin,
            env_key: daemon_env_key("auto-musk"),
            env_port_key: "MUSK_BACK_PORT".to_string(),
            health_interface: None,
            ready_timeout: None,
            extra_env: Vec::new(),
        }
    }

    /// PLAN-083 T-01：声明接口版本的泛化定义（notes 形态）。
    fn generic_def_declaring(bin: Option<PathBuf>, iface: &str) -> GenericDaemon {
        let mut def = generic_def(bin);
        def.name = "015-notes".to_string();
        def.url = "http://127.0.0.1:17819".to_string();
        def.env_key = daemon_env_key("015-notes");
        def.env_port_key = "AUTO_HTTP_PORT".to_string();
        def.health_interface = Some(iface.to_string());
        def
    }

    #[test]
    fn ensure_generic_reuses_running() {
        let mut io = FakeIo::new(vec![true]);
        let st = ensure_generic_io(
            &generic_def(None),
            &mut io,
            std::time::Duration::from_secs(1),
        );
        assert_eq!(
            st,
            DaemonStatus::Running("http://127.0.0.1:17201".to_string())
        );
        assert_eq!(io.ping_calls, 1, "ping 通即复用");
        assert!(io.spawn_path.is_none(), "已运行零打扰——bin 缺席也不影响");
    }

    #[test]
    fn ensure_generic_offline_when_bin_absent() {
        let mut io = FakeIo::new(vec![false; 5]);
        let st = ensure_generic_io(
            &generic_def(None),
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("未配置 bin"), "{reason}");
                assert!(reason.contains("auto-musk"), "原因携带可读名");
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
        assert!(io.spawn_path.is_none(), "bin 缺席不 spawn");
    }

    #[test]
    fn ensure_generic_spawns_with_port_env() {
        let mut io = FakeIo::new(vec![false, true]);
        let def = generic_def(Some(PathBuf::from(
            "D:/os/auto-musk/backend/target/release/musk.exe",
        )));
        let st = ensure_generic_io(&def, &mut io, std::time::Duration::from_secs(1));
        assert_eq!(
            st,
            DaemonStatus::Running("http://127.0.0.1:17201".to_string())
        );
        assert_eq!(
            io.spawn_env,
            vec![("MUSK_BACK_PORT".to_string(), "17201".to_string())],
            "spawn env = manifest env_port 键 + url 端口"
        );
    }

    #[test]
    fn ensure_generic_spawn_failure_is_offline_with_name() {
        let mut io = FakeIo::new(vec![false; 5]);
        io.spawn_result = Err("boom".to_string());
        let st = ensure_generic_io(
            &generic_def(Some(PathBuf::from("D:/nowhere/musk.exe"))),
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("spawn"), "{reason}");
                assert!(reason.contains("auto-musk"), "{reason}");
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
    }

    #[test]
    fn ensure_generic_ready_timeout_offline() {
        let mut io = FakeIo::new(vec![false; 10]);
        let st = ensure_generic_io(
            &generic_def(Some(PathBuf::from("D:/nowhere/musk.exe"))),
            &mut io,
            std::time::Duration::from_millis(1),
        );
        match st {
            DaemonStatus::Offline(reason) => assert!(reason.contains("就绪超时"), "{reason}"),
            other => panic!("应 Offline，实际 {other:?}"),
        }
    }

    // ─── PLAN-083 T-01：生产健康检查（真 TCP 分段响应） ───────────────────

    /// 每次连接都回同一份原始字节后关闭（真 TCP；覆盖 framing 各形态）。
    fn scripted_health_server(response: Vec<u8>) -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut s = stream;
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf);
                let _ = s.write_all(&response);
                let _ = s.flush();
            }
        });
        url
    }

    /// 接受连接但永不响应（占位监听者）——读预算内必须具名拒绝。
    fn silent_listener() -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        std::thread::spawn(move || {
            let mut held = Vec::new();
            for stream in listener.incoming().flatten() {
                held.push(stream);
            }
        });
        url
    }

    fn declaring_spec(url: &str, iface: &str) -> HealthSpec {
        let mut spec =
            HealthSpec::declaring(url, Some(iface), std::time::Duration::from_millis(800));
        spec.max_body_bytes = 512;
        spec
    }

    #[test]
    fn probe_health_ready_with_notes_double_encoded_body() {
        // 当前 Notes 生产形态：JsonResponse<String> 双层编码 + Content-Length。
        let inner = r#"{"status":"ok","interface":"notes.fpack/1"}"#;
        let body = serde_json::to_string(inner).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let url = scripted_health_server(response.into_bytes());
        assert_eq!(
            probe_health(&declaring_spec(&url, "notes.fpack/1")),
            HealthVerdict::Ready,
            "双层信封解包后 interface 匹配"
        );
    }

    #[test]
    fn probe_health_rejects_wrong_interface_and_bad_json() {
        // 错版接口：2xx 但 interface 不同。
        let wrong_body = r#"{"status":"ok","interface":"notes.fpack/2"}"#;
        let wrong = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            wrong_body.len(),
            wrong_body
        );
        let url = scripted_health_server(wrong.into_bytes());
        match probe_health(&declaring_spec(&url, "notes.fpack/1")) {
            HealthVerdict::WrongInterface { status, actual, .. } => {
                assert_eq!(status, 200);
                assert_eq!(actual.as_deref(), Some("notes.fpack/2"), "实际接口进原因");
            }
            other => panic!("错版应 WrongInterface，实际 {other:?}"),
        }
        // interface 缺席：2xx 但无法证明身份 → WrongInterface{actual: None}。
        let absent = "HTTP/1.1 200 OK\r\nContent-Length: 15\r\n\r\n{\"status\":\"ok\"}";
        let url = scripted_health_server(absent.as_bytes().to_vec());
        match probe_health(&declaring_spec(&url, "notes.fpack/1")) {
            HealthVerdict::WrongInterface { actual, .. } => assert_eq!(actual, None),
            other => panic!("缺席接口应 WrongInterface，实际 {other:?}"),
        }
        // 坏 JSON。
        let bad = "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\n{not-js";
        let url = scripted_health_server(bad.as_bytes().to_vec());
        assert!(matches!(
            probe_health(&declaring_spec(&url, "notes.fpack/1")),
            HealthVerdict::BadBody(_)
        ));
    }

    #[test]
    fn probe_health_rejects_truncated_oversize_and_timeout() {
        // 截断：Content-Length 100 但只发 13 字节即关闭。
        let truncated = "HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n{\"status\":\"ok";
        let url = scripted_health_server(truncated.as_bytes().to_vec());
        match probe_health(&declaring_spec(&url, "notes.fpack/1")) {
            HealthVerdict::BadBody(detail) => assert!(detail.contains("截断"), "{detail}"),
            other => panic!("截断应 BadBody，实际 {other:?}"),
        }
        // 超大：Content-Length 超预算（不读 body 即拒绝）。
        let oversize = "HTTP/1.1 200 OK\r\nContent-Length: 9000\r\n\r\n";
        let url = scripted_health_server(oversize.as_bytes().to_vec());
        match probe_health(&declaring_spec(&url, "notes.fpack/1")) {
            HealthVerdict::BadBody(detail) => assert!(detail.contains("超预算"), "{detail}"),
            other => panic!("超大应 BadBody，实际 {other:?}"),
        }
        // 无长度且超预算（不从任意无限 body 假定）。
        let mut unbounded = b"HTTP/1.1 200 OK\r\n\r\n".to_vec();
        unbounded.extend(std::iter::repeat(b'x').take(900));
        let url = scripted_health_server(unbounded);
        match probe_health(&declaring_spec(&url, "notes.fpack/1")) {
            HealthVerdict::BadBody(detail) => assert!(detail.contains("超预算"), "{detail}"),
            other => panic!("无长度超预算应 BadBody，实际 {other:?}"),
        }
        // 占位监听者：连接建立但无响应 → 读预算内具名拒绝。
        let url = silent_listener();
        match probe_health(&declaring_spec(&url, "notes.fpack/1")) {
            HealthVerdict::OccupiedElsewhere(detail) => {
                assert!(detail.contains("无 HTTP 响应"), "{detail}")
            }
            other => panic!("占位监听者应 OccupiedElsewhere，实际 {other:?}"),
        }
        // 死端口。
        let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let dead_url = format!("http://127.0.0.1:{}", dead.local_addr().unwrap().port());
        drop(dead);
        assert_eq!(
            probe_health(&declaring_spec(&dead_url, "notes.fpack/1")),
            HealthVerdict::NoListener
        );
    }

    #[test]
    fn probe_health_non_2xx_and_chunked_framing() {
        // 非 2xx：占用者未就绪，不当作目标服务。
        let not_ready = "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 2\r\n\r\n{}";
        let url = scripted_health_server(not_ready.as_bytes().to_vec());
        assert_eq!(
            probe_health(&declaring_spec(&url, "notes.fpack/1")),
            HealthVerdict::NotReady { status: 503 }
        );
        // chunked：两段数据 + 结束块。
        let chunked = concat!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
            "1c\r\n{\"status\":\"ok\",\"interface\":\"",
            "\r\n0f\r\nnotes.fpack/1\"}\r\n0\r\n\r\n"
        );
        let url = scripted_health_server(chunked.as_bytes().to_vec());
        assert_eq!(
            probe_health(&declaring_spec(&url, "notes.fpack/1")),
            HealthVerdict::Ready,
            "chunked 分段响应应解出完整 body"
        );
        // 不支持的 framing：具名拒绝而不是猜。
        let te = "HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\nContent-Length: 2\r\n\r\n{}";
        let url = scripted_health_server(te.as_bytes().to_vec());
        match probe_health(&declaring_spec(&url, "notes.fpack/1")) {
            HealthVerdict::BadBody(detail) => {
                assert!(detail.contains("Transfer-Encoding"), "{detail}")
            }
            other => panic!("不支持 framing 应 BadBody，实际 {other:?}"),
        }
    }

    #[test]
    fn probe_health_legacy_mode_keeps_2xx_only() {
        // 无 interface 声明 = 旧语义：2xx 即就绪，不读 body。
        let with_body = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
        let url = scripted_health_server(with_body.as_bytes().to_vec());
        assert_eq!(
            probe_health(&HealthSpec::legacy(
                &url,
                std::time::Duration::from_millis(800)
            )),
            HealthVerdict::Ready
        );
        let not_ready = "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n";
        let url = scripted_health_server(not_ready.as_bytes().to_vec());
        assert_eq!(
            probe_health(&HealthSpec::legacy(
                &url,
                std::time::Duration::from_millis(800)
            )),
            HealthVerdict::NotReady { status: 503 }
        );
    }

    /// R-2/N5：旧语义与 `tcp_ping` 逐字节同义——**2xx 状态行前缀即就绪**，
    /// 即使响应头永不终止（旧实现只读 128 字节核前缀，从不要求完整头）。
    #[test]
    fn probe_health_legacy_prefix_ready_without_complete_headers() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut s = stream;
                let mut buf = [0u8; 256];
                let _ = s.read(&mut buf);
                // 只发状态行 + 半个头，永不终止头块，并保持连接。
                let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: appl");
                let _ = s.flush();
                let _ = rx.recv(); // 由测试结束时 drop tx 释放
            }
        });
        assert_eq!(
            probe_health(&HealthSpec::legacy(
                &url,
                std::time::Duration::from_millis(500)
            )),
            HealthVerdict::Ready,
            "旧语义：2xx 前缀即就绪（不等完整头/body）"
        );
        drop(tx);
        // 声明 interface 时同一响应必须被判坏（没有完整头/body）。
        let listener2 = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url2 = format!(
            "http://127.0.0.1:{}",
            listener2.local_addr().unwrap().port()
        );
        std::thread::spawn(move || {
            for stream in listener2.incoming().flatten() {
                let mut s = stream;
                let mut buf = [0u8; 256];
                let _ = s.read(&mut buf);
                let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: appl");
                let _ = s.flush();
                std::thread::sleep(std::time::Duration::from_millis(400));
            }
        });
        assert!(
            !probe_health(&declaring_spec(&url2, "notes.fpack/1")).is_ready(),
            "声明 interface 时无完整响应头的 2xx 不算就绪"
        );
    }

    /// R-2/F-4：启动器"声明 daemon"门的三态（未声明/非法/降级）——非法
    /// 声明必须可见（宽松入口会折成 None，导致无效分支不可达）。
    #[test]
    fn declared_backend_gate_sees_invalid_declaration() {
        let root = temp_root("declared-gate");
        let os_root = root.join("auto-os");
        std::fs::create_dir_all(&os_root).unwrap();
        std::fs::write(
            os_root.join("apps.manifest"),
            r#"{"apps":[
                {"id":"015-notes","repo":"../auto-notes","daemon":{"port":17819,"health_interface":"bogus"}},
                {"id":"other-app","repo":"../x","daemon":{"port":17999,"bin":"Z:/no-such.exe"}}
            ]}"#,
        )
        .unwrap();
        match resolve_declared_backend(&root, "015-notes", std::time::Duration::from_millis(200)) {
            DeclaredBackendConfig::Invalid(reason) => {
                assert!(reason.contains("health_interface"), "{reason}")
            }
            other => panic!("非法声明必须 Invalid（不可折成 NotDeclared），实际 {other:?}"),
        }
        assert!(matches!(
            resolve_declared_backend(
                &root,
                "not-declared-at-all",
                std::time::Duration::from_millis(200)
            ),
            DeclaredBackendConfig::NotDeclared
        ));
        // 合法声明但可执行缺席：ensure 失败 → Offline（调用方按既有语义降级）。
        match resolve_declared_backend(&root, "other-app", std::time::Duration::from_millis(500)) {
            DeclaredBackendConfig::Offline(reason) => {
                assert!(
                    reason.contains("spawn")
                        || reason.contains("不可复用")
                        || reason.contains("无监听"),
                    "{reason}"
                )
            }
            other => panic!("可执行缺席应 Offline，实际 {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// R-1/F-1：旧语义（无 interface）**只读状态行**——超大 body、
    /// 不支持的 framing、截断 body 都不影响 2xx 就绪；声明 interface 才拒绝。
    /// 与 `tcp_ping`（首 128 字节、状态前缀）逐字同义。
    #[test]
    fn probe_health_legacy_reads_status_line_only() {
        let mut huge = b"HTTP/1.1 200 OK\r\nContent-Length: 20000\r\n\r\n".to_vec();
        huge.extend(std::iter::repeat(b'x').take(20000));
        let url = scripted_health_server(huge);
        assert_eq!(
            probe_health(&HealthSpec::legacy(
                &url,
                std::time::Duration::from_millis(800)
            )),
            HealthVerdict::Ready,
            "旧语义不看 body（20KB 仍 ready）"
        );
        assert!(
            matches!(
                probe_health(&declaring_spec(&url, "notes.fpack/1")),
                HealthVerdict::BadBody(_)
            ),
            "声明 interface 时才按预算拒绝"
        );

        // 不支持的 framing：旧语义不读 body ⇒ 仍 ready；声明时具名拒绝。
        let te = concat!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\nContent-Length: 4\r\n\r\n",
            "abcd"
        );
        let url = scripted_health_server(te.as_bytes().to_vec());
        assert_eq!(
            probe_health(&HealthSpec::legacy(
                &url,
                std::time::Duration::from_millis(800)
            )),
            HealthVerdict::Ready
        );
        assert!(matches!(
            probe_health(&declaring_spec(&url, "notes.fpack/1")),
            HealthVerdict::BadBody(_)
        ));

        // 截断 body：旧语义仍 ready（tcp_ping 从未读 body）。
        let truncated = "HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n{";
        let url = scripted_health_server(truncated.as_bytes().to_vec());
        assert_eq!(
            probe_health(&HealthSpec::legacy(
                &url,
                std::time::Duration::from_millis(800)
            )),
            HealthVerdict::Ready
        );
    }

    #[test]
    fn ensure_strict_reuses_ready_without_spawn() {
        let mut io = FakeIo::new(vec![true]).with_health(vec![HealthVerdict::Ready]);
        let def = generic_def_declaring(
            Some(PathBuf::from("D:/notes/notes-service.exe")),
            "notes.fpack/1",
        );
        let st = ensure_generic_io(&def, &mut io, std::time::Duration::from_secs(1));
        assert_eq!(
            st,
            DaemonStatus::Running("http://127.0.0.1:17819".to_string())
        );
        assert_eq!(io.spawn_calls, 0, "已就绪服务零打扰复用");
    }

    #[test]
    fn ensure_strict_rejects_wrong_interface_before_spawn() {
        let mut io = FakeIo::new(vec![false]).with_health(vec![HealthVerdict::WrongInterface {
            status: 200,
            expected: "notes.fpack/1".to_string(),
            actual: Some("notes.fpack/2".to_string()),
        }]);
        let def = generic_def_declaring(
            Some(PathBuf::from("D:/notes/notes-service.exe")),
            "notes.fpack/1",
        );
        let st = ensure_generic_io(&def, &mut io, std::time::Duration::from_secs(1));
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("不可复用"), "{reason}");
                assert!(reason.contains("notes.fpack/2"), "原因带实际版本: {reason}");
                assert!(reason.contains("未 spawn"), "明确不 spawn: {reason}");
            }
            other => panic!("错版应 Offline，实际 {other:?}"),
        }
        assert_eq!(io.spawn_calls, 0, "占用者存活——不在占用端口 spawn");
    }

    #[test]
    fn ensure_strict_spawns_then_requires_same_interface_to_ready() {
        // spawn 后先 YesListener，再 Ready → Running（spawn 一次）。
        let mut io = FakeIo::new(vec![false]).with_health(vec![
            HealthVerdict::NoListener,
            HealthVerdict::NoListener,
            HealthVerdict::Ready,
        ]);
        let def = generic_def_declaring(
            Some(PathBuf::from("D:/notes/notes-service.exe")),
            "notes.fpack/1",
        );
        let st = ensure_generic_io(&def, &mut io, std::time::Duration::from_secs(1));
        assert_eq!(
            st,
            DaemonStatus::Running("http://127.0.0.1:17819".to_string())
        );
        assert_eq!(io.spawn_calls, 1);
        assert_eq!(
            io.spawn_env,
            vec![("AUTO_HTTP_PORT".to_string(), "17819".to_string())],
            "spawn env = manifest env_port 键 + 声明端口"
        );

        // spawn 后 2xx 早到但接口错 → 立即 Offline（不提前 ready、不等到超时）。
        let mut io = FakeIo::new(vec![false]).with_health(vec![
            HealthVerdict::NoListener,
            HealthVerdict::WrongInterface {
                status: 200,
                expected: "notes.fpack/1".to_string(),
                actual: Some("other.fpack/1".to_string()),
            },
        ]);
        let started = std::time::Instant::now();
        let st = ensure_generic_io(&def, &mut io, std::time::Duration::from_secs(5));
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("就绪期接口不匹配"), "{reason}")
            }
            other => panic!("2xx 错接口应 Offline，实际 {other:?}"),
        }
        assert!(
            started.elapsed() < std::time::Duration::from_secs(3),
            "错接口立即拒绝，不等满预算"
        );
    }

    #[test]
    fn ensure_strict_timeout_reports_last_verdict() {
        let mut io = FakeIo::new(vec![false]).with_health(vec![HealthVerdict::NoListener]);
        let mut def = generic_def_declaring(
            Some(PathBuf::from("D:/notes/notes-service.exe")),
            "notes.fpack/1",
        );
        def.ready_timeout = Some(std::time::Duration::from_millis(200));
        let st = ensure_generic_io(&def, &mut io, std::time::Duration::from_secs(5));
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("就绪超时"), "{reason}");
                assert!(reason.contains("200ms"), "预算取自声明: {reason}");
                assert!(reason.contains("无监听"), "原因带最后裁决: {reason}");
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
    }

    #[test]
    fn ensure_strict_bin_absent_reports_named_offline() {
        let mut io = FakeIo::new(vec![false]).with_health(vec![HealthVerdict::NoListener]);
        let def = generic_def_declaring(None, "notes.fpack/1");
        let st = ensure_generic_io(&def, &mut io, std::time::Duration::from_millis(1));
        match st {
            DaemonStatus::Offline(reason) => {
                assert!(
                    reason.contains("未配置 bin") && reason.contains("015-notes"),
                    "{reason}"
                )
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
        assert_eq!(io.spawn_calls, 0);
    }

    // ─── PLAN-083 T-02：服务/宿主配置边界（D-03） ─────────────────────────

    use auto_ui_compiler::app_registry::{
        ManifestDaemon, ManifestDaemonCredential, ManifestDaemonHost,
    };

    fn temp_root(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "autoui-083-t02-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn manifest_def(port: u16) -> ManifestDaemon {
        ManifestDaemon {
            port,
            bin: Some("target/release/notes-service".to_string()),
            env_port: Some("AUTO_HTTP_PORT".to_string()),
            health_interface: Some("notes.fpack/1".to_string()),
            ready_timeout_ms: None,
            env: Default::default(),
            host: None,
            instance_key: None,
            data_root: None,
        }
    }

    fn write_policy(dir: &Path, entries: &[(&str, &str)]) -> PathBuf {
        let entries_json: Vec<String> = entries
            .iter()
            .map(|(principal, cred)| {
                format!(
                    r#"{{"principal":"{principal}","cred":"{cred}","operations":["read"],"book_scopes":[]}}"#
                )
            })
            .collect();
        let path = dir.join("fp-policy.json");
        std::fs::write(
            &path,
            format!(r#"{{"entries":[{}]}}"#, entries_json.join(",")),
        )
        .unwrap();
        path
    }

    #[test]
    fn managed_credential_resolves_without_leaking_value() {
        let root = temp_root("cred");
        let cred = "ab".repeat(32);
        let policy = write_policy(&root, &[("fp-reader", cred.as_str()), ("other", "cd")]);
        let reference = ManagedCredentialRef {
            env_key: "NOTES_FP_CRED".to_string(),
            path: policy.clone(),
            principal: "fp-reader".to_string(),
            field: "cred".to_string(),
        };
        assert_eq!(reference.resolve().as_deref(), Ok(cred.as_str()));
        assert!(
            !reference.diagnostic().contains(cred.as_str()),
            "诊断不含凭据值：{}",
            reference.diagnostic()
        );
        assert!(
            !format!("{reference:?}").contains(cred.as_str()),
            "Debug 不含凭据值"
        );
        // 主体缺席：具名错误，回显主体/路径但不含任何凭据值。
        let missing = ManagedCredentialRef {
            principal: "no-such".to_string(),
            ..reference.clone()
        };
        let err = missing.resolve().expect_err("缺主体必须失败");
        assert!(
            err.contains("no-such") && err.contains("fp-policy.json"),
            "{err}"
        );
        assert!(!err.contains(cred.as_str()), "错误不回显凭据值: {err}");
        // 文件不可读。
        let absent = ManagedCredentialRef {
            path: root.join("nope.json"),
            ..reference.clone()
        };
        assert!(absent.resolve().is_err());
        // 字段缺席。
        let no_field = ManagedCredentialRef {
            field: "nope".to_string(),
            ..reference.clone()
        };
        assert!(no_field.resolve().is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn host_backend_env_isolates_two_hosts() {
        let root_a = temp_root("host-a");
        let root_b = temp_root("host-b");
        let cred_a = "11".repeat(32);
        let cred_b = "22".repeat(32);
        let policy_a = write_policy(&root_a, &[("fp-reader", cred_a.as_str())]);
        let policy_b = write_policy(&root_b, &[("fp-reader", cred_b.as_str())]);

        let mut def_a = manifest_def(17819);
        def_a.host = Some(ManifestDaemonHost {
            env: [("NOTES_RETRY".to_string(), "1".to_string())]
                .into_iter()
                .collect(),
            credential: Some(ManifestDaemonCredential {
                env: "NOTES_FP_CRED".to_string(),
                path: Some(policy_a.to_string_lossy().to_string()),
                principal: "fp-reader".to_string(),
                field: "cred".to_string(),
            }),
        });
        let mut def_b = manifest_def(17820);
        def_b.host = Some(ManifestDaemonHost {
            env: [("NOTES_RETRY".to_string(), "2".to_string())]
                .into_iter()
                .collect(),
            credential: Some(ManifestDaemonCredential {
                env: "NOTES_FP_CRED".to_string(),
                path: Some(policy_b.to_string_lossy().to_string()),
                principal: "fp-reader".to_string(),
                field: "cred".to_string(),
            }),
        });

        let env_a = host_backend_env("NOTES_DAEMON", "http://127.0.0.1:17819", &def_a, &root_a)
            .expect("A 宿主配置");
        let env_b = host_backend_env("NOTES_DAEMON", "http://127.0.0.1:17820", &def_b, &root_b)
            .expect("B 宿主配置");

        let get = |env: &HostBackendEnv, key: &str| {
            env.entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        assert_eq!(
            get(&env_a, "NOTES_DAEMON").as_deref(),
            Some("http://127.0.0.1:17819")
        );
        assert_eq!(
            get(&env_b, "NOTES_DAEMON").as_deref(),
            Some("http://127.0.0.1:17820")
        );
        assert_eq!(get(&env_a, "NOTES_RETRY").as_deref(), Some("1"));
        assert_eq!(get(&env_b, "NOTES_RETRY").as_deref(), Some("2"));
        assert_eq!(
            get(&env_a, "NOTES_FP_CRED").as_deref(),
            Some(cred_a.as_str())
        );
        assert_eq!(
            get(&env_b, "NOTES_FP_CRED").as_deref(),
            Some(cred_b.as_str())
        );
        assert!(
            !env_a.entries.iter().any(|(_, v)| v == &cred_b),
            "A 不得看到 B 的凭据"
        );
        assert!(
            !env_b.entries.iter().any(|(_, v)| v == &cred_a),
            "B 不得看到 A 的凭据"
        );
        assert_eq!(env_a.credential_env.as_deref(), Some("NOTES_FP_CRED"));
        // remove 覆盖全部投递键：子进程不得继承宿主同名历史值。
        for key in ["NOTES_DAEMON", "NOTES_RETRY", "NOTES_FP_CRED"] {
            assert!(env_a.remove.iter().any(|k| k == key), "remove 缺 {key}");
            assert!(env_b.remove.iter().any(|k| k == key), "remove 缺 {key}");
        }
        // 非凭据条目可用于 in-proc 注入；凭据只走子进程。
        let non_secret: Vec<(&str, &str)> = env_a.non_secret_entries().collect();
        assert!(non_secret.iter().any(|(k, _)| *k == "NOTES_DAEMON"));
        assert!(!non_secret.iter().any(|(k, _)| *k == "NOTES_FP_CRED"));
        assert!(env_a.has_credentials());
        assert!(
            !format!("{env_a:?}").contains(cred_a.as_str()),
            "Debug 打码凭据"
        );
        assert!(format!("{env_a:?}").contains("<managed-credential>"));
        let _ = std::fs::remove_dir_all(&root_a);
        let _ = std::fs::remove_dir_all(&root_b);
    }

    #[test]
    fn host_backend_env_reports_missing_managed_config() {
        let root = temp_root("host-missing");
        let mut def = manifest_def(17819);
        def.host = Some(ManifestDaemonHost {
            env: Default::default(),
            credential: Some(ManifestDaemonCredential {
                env: "NOTES_FP_CRED".to_string(),
                path: Some(root.join("absent.json").to_string_lossy().to_string()),
                principal: "fp-reader".to_string(),
                field: "cred".to_string(),
            }),
        });
        let err = host_backend_env("NOTES_DAEMON", "http://127.0.0.1:17819", &def, &root)
            .expect_err("受管配置缺席必须具名失败");
        assert!(
            err.contains("fp-reader") && err.contains("absent.json"),
            "{err}"
        );
        // 无 host 声明：只有地址键，且无凭据。
        let plain = manifest_def(17819);
        let env = host_backend_env("NOTES_DAEMON", "http://127.0.0.1:17819", &plain, &root)
            .expect("纯地址");
        assert_eq!(env.entries.len(), 1);
        assert!(!env.has_credentials());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn service_env_keeps_declared_config_and_port_authority() {
        let mut def = manifest_def(17819);
        def.env
            .insert("AUTO_HTTP_PORT".to_string(), "9999".to_string());
        def.env
            .insert("NOTES_DATA_DIR".to_string(), "D:/data".to_string());
        let generic = generic_daemon_from_manifest("015-notes", Path::new("Z:/repo"), &def);
        let env = generic.spawn_env();
        assert!(
            env.iter()
                .any(|(k, v)| k == "NOTES_DATA_DIR" && v == "D:/data"),
            "声明服务配置随 spawn 投递: {env:?}"
        );
        assert_eq!(
            env.iter().filter(|(k, _)| k == "AUTO_HTTP_PORT").count(),
            1,
            "端口键唯一"
        );
        assert_eq!(
            env.iter()
                .find(|(k, _)| k == "AUTO_HTTP_PORT")
                .map(|(_, v)| v.as_str()),
            Some("17819"),
            "端口以目标 url 为权威（覆盖同名声明）"
        );
    }

    #[test]
    fn expand_env_path_expands_known_vars_and_keeps_unknown() {
        std::env::set_var("PLAN083_TEST_HOME", "D:/autoui-home");
        assert_eq!(
            expand_env_path("%PLAN083_TEST_HOME%/data"),
            "D:/autoui-home/data"
        );
        assert_eq!(
            expand_env_path("$PLAN083_TEST_HOME/data"),
            "D:/autoui-home/data"
        );
        assert_eq!(
            expand_env_path("${PLAN083_TEST_HOME}/data"),
            "D:/autoui-home/data"
        );
        assert_eq!(
            expand_env_path("%PLAN083_NO_SUCH_VAR%/data"),
            "%PLAN083_NO_SUCH_VAR%/data",
            "未知变量原样保留"
        );
        std::env::remove_var("PLAN083_TEST_HOME");
    }

    #[test]
    fn probe_manifest_host_reports_declared_shapes_without_spawning() {
        // 未声明：临时 parent 下无 auto-os 清单；resolve 序会回落主检出
        // `D:/autostack/auto-os`（真实清单），故用清单里不存在的名。
        let parent = temp_root("probe-none");
        assert!(matches!(
            probe_manifest_host(&parent, "no-such-daemon-083"),
            ManifestHostProbe::NotDeclared
        ));
        let _ = std::fs::remove_dir_all(&parent);

        // 声明非法：具名 Invalid（不 spawn）。
        let parent = temp_root("probe-invalid");
        let os_root = parent.join("auto-os");
        std::fs::create_dir_all(&os_root).unwrap();
        std::fs::write(
            os_root.join("apps.manifest"),
            r#"{"apps":[{"id":"015-notes","repo":"../auto-notes","daemon":{"port":17819,"health_interface":"bogus"}}]}"#,
        )
        .unwrap();
        match probe_manifest_host(&parent, "015-notes") {
            ManifestHostProbe::Invalid(reason) => {
                assert!(reason.contains("health_interface"), "{reason}")
            }
            other => panic!("非法声明应 Invalid，实际 {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&parent);

        // 声明合法但无监听：NotReady（只探不孵）。
        let parent = temp_root("probe-notready");
        let os_root = parent.join("auto-os");
        std::fs::create_dir_all(&os_root).unwrap();
        let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = dead.local_addr().unwrap().port();
        drop(dead);
        std::fs::write(
            os_root.join("apps.manifest"),
            format!(
                r#"{{"apps":[{{"id":"015-notes","repo":"../auto-notes","daemon":{{"port":{port},"health_interface":"notes.fpack/1"}}}}]}}"#
            ),
        )
        .unwrap();
        match probe_manifest_host(&parent, "015-notes") {
            ManifestHostProbe::NotReady(reason) => {
                assert!(reason.contains("无监听"), "{reason}")
            }
            other => panic!("无监听应 NotReady，实际 {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&parent);
    }
}
