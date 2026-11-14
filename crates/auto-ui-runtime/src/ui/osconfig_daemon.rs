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
    pub fn declaring(
        url: &str,
        interface: Option<&str>,
        timeout: std::time::Duration,
    ) -> Self {
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
fn read_health_response(
    spec: &HealthSpec,
) -> Result<RawHealthResponse, HealthVerdict> {
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
            Err(_) => {
                return Err(HealthVerdict::BadBody(
                    "Content-Length 非法".to_string(),
                ))
            }
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
                Ok(0) => {
                    return Err(HealthVerdict::BadBody(
                        "chunked 响应截断".to_string(),
                    ))
                }
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
                value = serde_json::from_str(&inner).map_err(|err| format!("信封内 JSON 非法（{err}）"))?;
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

/// 桌面本会话是否拉起过 daemon（复用判定/UX 文案；detached 语义下不持
/// Child——桌面退出不杀 daemon，句柄即刻放掉，只留"曾拉起"记录）。
static DID_SPAWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
}

impl GenericDaemon {
    /// PLAN-083 T-01：本次就绪预算（manifest 声明优先于调用方缺省）。
    pub fn effective_ready_timeout(&self, fallback: std::time::Duration) -> std::time::Duration {
        self.ready_timeout.unwrap_or(fallback)
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
    let env = vec![(def.env_port_key.clone(), port_of(&def.url).to_string())];
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
    let precheck = HealthSpec::declaring(&def.url, interface, std::time::Duration::from_millis(800));
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
    let env = vec![(def.env_port_key.clone(), port_of(&def.url).to_string())];
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
        let mut spec = HealthSpec::declaring(
            url,
            Some(iface),
            std::time::Duration::from_millis(800),
        );
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
            HealthVerdict::WrongInterface {
                status, actual, ..
            } => {
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

    #[test]
    fn ensure_strict_reuses_ready_without_spawn() {
        let mut io = FakeIo::new(vec![true]).with_health(vec![HealthVerdict::Ready]);
        let def = generic_def_declaring(Some(PathBuf::from("D:/notes/notes-service.exe")), "notes.fpack/1");
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
        let def = generic_def_declaring(Some(PathBuf::from("D:/notes/notes-service.exe")), "notes.fpack/1");
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
        let def = generic_def_declaring(Some(PathBuf::from("D:/notes/notes-service.exe")), "notes.fpack/1");
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
        let mut def = generic_def_declaring(Some(PathBuf::from("D:/notes/notes-service.exe")), "notes.fpack/1");
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
                assert!(reason.contains("未配置 bin") && reason.contains("015-notes"), "{reason}")
            }
            other => panic!("应 Offline，实际 {other:?}"),
        }
        assert_eq!(io.spawn_calls, 0);
    }
}
