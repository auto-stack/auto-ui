//! PLAN-1119 AutoWeb HTTPS 只读导航永久 case（auto-ui 消费面）。
//!
//! 覆盖 V-01（真实 TLS 全链 + 双写法共用来源/缓存）、V-02（对真实服务器
//! 的 `If-None-Match`/`304`/新版本/坏更新/`no-store` 轨迹）、V-03（不信任
//! 证书/主机名不匹配/状态/媒体/重定向/限额/只读准入负控）、V-04（停止/
//! 关页/关窗/抢占/后台页签完成与真实请求释放、慢服务不阻塞外壳）的 CPU
//! 档；真窗档（V-04 单窗面）在 `AUTO_UI_ASE_M0_E2E=1` 排窗后跑（窗口单
//! 例纪律）。协议/缓存纯语义面在 auto-lang `ase-m0` 的
//! `tests/autoweb_readonly.rs`。
//!
//! 测试 TLS 站（受控真实 TLS）：rcgen 运行期生成隔离测试 CA +
//! `localhost` 服务器证书；rustls 同步服务端。测试 CA 的 PEM **仅注入被测
//! AutoWeb 导航客户端**（`with_autoweb_options`），OS 信任库不动；第二
//! "恶意 CA" 签发不受信证书做负控。所有真实请求走 127.0.0.1 回环。
//!
//! 准备（全部档，不在 case 计时内；2026-11-14 测试纪律）：Counter 互操作
//! case 需 ui980 provider 预构建——`cargo build -p auto-ui-runtime
//! --features ace-bridge --example autoscape_window` 后运行
//! `autoscape_window.exe --prep-only`（与 1118 同约定）。

#![cfg(all(windows, feature = "ace-bridge"))]

use ase_m0::autoweb::NavLimits;
use auto_ui_runtime::ui::ase_browser_shell::{resolve_address, AutoscapeHost};
use auto_ui_runtime::ui::autoweb_nav::AutowebNavOptions;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 受控 TLS 测试站（rcgen 测试 CA + rustls 同步服务端）
// ---------------------------------------------------------------------------

mod tls_site {
    use super::*;

    /// 一个可编程端点（状态/响应头/正文/慢体 trickle）。
    #[derive(Clone)]
    pub struct Endpoint {
        pub status: u16,
        pub headers: Vec<(String, String)>,
        pub body: Vec<u8>,
        /// Some((chunks, per_chunk_ms))：分块慢体（取消/释放证据面）。
        pub trickle: Option<(usize, u64)>,
    }

    impl Endpoint {
        pub fn ok(content_type: &str, body: &str, extra: &[(&str, String)]) -> Self {
            let mut headers = vec![("Content-Type".to_string(), content_type.to_string())];
            for (key, value) in extra {
                headers.push(((*key).to_string(), value.clone()));
            }
            Endpoint {
                status: 200,
                headers,
                body: body.as_bytes().to_vec(),
                trickle: None,
            }
        }
    }

    /// 服务端观察到的单次请求。
    #[derive(Clone, Debug)]
    pub struct ServedRequest {
        pub path: String,
        pub accept: String,
        pub if_none_match: Option<String>,
        /// 该连接实际送达的状态（对账协商面；aborted 连接为首个响应状态）。
        #[allow(dead_code)]
        pub status_served: u16,
        pub body_bytes_sent: usize,
        pub outcome: &'static str, // "served" | "aborted"
    }

    pub struct TestSite {
        pub port: u16,
        pub requests: Arc<Mutex<Vec<ServedRequest>>>,
        /// 请求头解析即记（路径）——"请求已到达"信号；带结果的记录仍在
        /// 连接终止时落账（RF-1：两阶段观察面）。
        pub seen: Arc<Mutex<Vec<String>>>,
        pub routes: Arc<Mutex<HashMap<String, Endpoint>>>,
    }

    /// 生成一个测试 CA（自签）与其签发的 `localhost` 服务器证书。
    /// 返回 (server cert der, server key der, ca pem)。
    fn mint_certificate(authority: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mut ca_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.distinguished_name.push(
            rcgen::DnType::CommonName,
            format!("AutoWeb Test CA {authority}"),
        );
        ca_params.key_usages = vec![
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];
        let ca_key = rcgen::KeyPair::generate().unwrap();
        let ca_cert = ca_params.self_signed(&ca_key).unwrap();

        let mut server_params =
            rcgen::CertificateParams::new(vec!["localhost".to_string()]).unwrap();
        server_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "localhost");
        let server_key = rcgen::KeyPair::generate().unwrap();
        let server_cert = server_params
            .signed_by(&server_key, &ca_cert, &ca_key)
            .unwrap();
        (
            server_cert.der().to_vec(),
            server_key.serialize_der().to_vec(),
            ca_cert.pem().as_bytes().to_vec(),
        )
    }

    /// 启动受控 TLS 站（127.0.0.1:0 随机端口）。`authority` 仅区分 CA 身份
    /// （trusted / rogue / wrong-host）。
    pub fn spawn(authority: &str, server_names: &[&str]) -> (TestSite, Vec<u8>, Vec<u8>) {
        let (cert_der, key_der, ca_pem) = if server_names.is_empty() {
            mint_certificate(authority)
        } else {
            // 指定 SAN（主机名不匹配负控：证书给别的名字）。
            mint_certificate_for(server_names)
        };
        let _ = rustls::crypto::ring::default_provider().install_default();
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![rustls::pki_types::CertificateDer::from(cert_der.clone())],
                rustls::pki_types::PrivateKeyDer::Pkcs8(
                    rustls::pki_types::PrivatePkcs8KeyDer::from(key_der.clone()),
                ),
            )
            .expect("test server certificate");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let routes = Arc::new(Mutex::new(HashMap::new()));
        let config = Arc::new(config);
        let accept_requests = Arc::clone(&requests);
        let accept_seen = Arc::clone(&seen);
        let accept_routes = Arc::clone(&routes);
        let accept_config = Arc::clone(&config);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { break };
                let config = Arc::clone(&accept_config);
                let requests = Arc::clone(&accept_requests);
                let seen = Arc::clone(&accept_seen);
                let routes = Arc::clone(&accept_routes);
                std::thread::spawn(move || {
                    serve_connection(stream, config, requests, seen, routes)
                });
            }
        });
        (
            TestSite {
                port,
                requests,
                seen,
                routes,
            },
            ca_pem,
            cert_der,
        )
    }

    fn mint_certificate_for(server_names: &[&str]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mut ca_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "AutoWeb Test CA wrong-host");
        let ca_key = rcgen::KeyPair::generate().unwrap();
        let ca_cert = ca_params.self_signed(&ca_key).unwrap();
        let names: Vec<String> = server_names.iter().map(|s| s.to_string()).collect();
        let mut server_params = rcgen::CertificateParams::new(names).unwrap();
        server_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, server_names[0].to_string());
        let server_key = rcgen::KeyPair::generate().unwrap();
        let server_cert = server_params
            .signed_by(&server_key, &ca_cert, &ca_key)
            .unwrap();
        (
            server_cert.der().to_vec(),
            server_key.serialize_der().to_vec(),
            ca_cert.pem().as_bytes().to_vec(),
        )
    }

    fn serve_connection(
        stream: std::net::TcpStream,
        config: Arc<rustls::ServerConfig>,
        requests: Arc<Mutex<Vec<ServedRequest>>>,
        seen: Arc<Mutex<Vec<String>>>,
        routes: Arc<Mutex<HashMap<String, Endpoint>>>,
    ) {
        let stream = stream;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
        let Ok(conn) = rustls::ServerConnection::new(Arc::clone(&config)) else {
            return;
        };
        let mut tls = rustls::StreamOwned::new(conn, stream);
        // 请求头（GET 无正文）。
        let mut head = Vec::new();
        let mut buffer = [0u8; 1024];
        loop {
            match tls.read(&mut buffer) {
                Ok(0) => return,
                Ok(n) => {
                    head.extend_from_slice(&buffer[..n]);
                    if head.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(_) => return,
            }
        }
        let head = String::from_utf8_lossy(&head).to_string();
        let request_line = head.lines().next().unwrap_or_default().to_string();
        let path = request_line.split(' ').nth(1).unwrap_or("/").to_string();
        seen.lock()
            .unwrap()
            .push(path.split('?').next().unwrap_or("/").to_string());
        let header = |name: &str| -> Option<String> {
            head.lines().skip(1).find_map(|line| {
                let (key, value) = line.split_once(':')?;
                key.trim()
                    .eq_ignore_ascii_case(name)
                    .then(|| value.trim().to_string())
            })
        };
        let accept = header("Accept").unwrap_or_default();
        let if_none_match = header("If-None-Match");
        let route_path = path.split('?').next().unwrap_or("/").to_string();
        let endpoint = routes.lock().unwrap().get(&route_path).cloned();
        let Some(endpoint) = endpoint else {
            requests.lock().unwrap().push(ServedRequest {
                path,
                accept,
                if_none_match,
                status_served: 404,
                body_bytes_sent: 0,
                outcome: "served",
            });
            let _ = tls.write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
            return;
        };
        let reason = match endpoint.status {
            200 => "OK",
            302 => "Found",
            304 => "Not Modified",
            404 => "Not Found",
            406 => "Not Acceptable",
            500 => "Internal Server Error",
            _ => "Status",
        };
        let mut response = format!("HTTP/1.1 {} {}\r\n", endpoint.status, reason);
        for (key, value) in &endpoint.headers {
            response.push_str(&format!("{key}: {value}\r\n"));
        }
        response.push_str(&format!("Content-Length: {}\r\n", endpoint.body.len()));
        response.push_str("Connection: close\r\n\r\n");
        if tls.write_all(response.as_bytes()).is_err() {
            requests.lock().unwrap().push(ServedRequest {
                path,
                accept,
                if_none_match,
                status_served: endpoint.status,
                body_bytes_sent: 0,
                outcome: "aborted",
            });
            return;
        }
        // 慢体：分块间隔写（取消观察面）。
        if let Some((chunks, per_chunk_ms)) = endpoint.trickle {
            let chunk_size = (endpoint.body.len() / chunks.max(1)).max(1);
            for chunk in endpoint.body.chunks(chunk_size) {
                std::thread::sleep(Duration::from_millis(per_chunk_ms));
                if tls.write_all(chunk).is_err() {
                    requests.lock().unwrap().push(ServedRequest {
                        path,
                        accept,
                        if_none_match,
                        status_served: endpoint.status,
                        body_bytes_sent: 0,
                        outcome: "aborted",
                    });
                    return;
                }
            }
        } else if !endpoint.body.is_empty() && tls.write_all(&endpoint.body).is_err() {
            requests.lock().unwrap().push(ServedRequest {
                path,
                accept,
                if_none_match,
                status_served: endpoint.status,
                body_bytes_sent: 0,
                outcome: "aborted",
            });
            return;
        }
        requests.lock().unwrap().push(ServedRequest {
            path,
            accept,
            if_none_match,
            status_served: endpoint.status,
            body_bytes_sent: endpoint.body.len(),
            outcome: "served",
        });
    }

    impl TestSite {
        pub fn url(&self, path: &str) -> String {
            format!("https://localhost:{}{}", self.port, path)
        }

        pub fn set(&self, path: &str, endpoint: Endpoint) {
            self.routes
                .lock()
                .unwrap()
                .insert(path.to_string(), endpoint);
        }

        pub fn log(&self) -> Vec<ServedRequest> {
            self.requests.lock().unwrap().clone()
        }

        /// 已见请求数（路径前缀；头解析即计）。
        pub fn seen_count(&self, path: &str) -> usize {
            self.seen
                .lock()
                .unwrap()
                .iter()
                .filter(|p| *p == path)
                .count()
        }

        pub fn requests_to(&self, path: &str) -> Vec<ServedRequest> {
            self.log().into_iter().filter(|r| r.path == path).collect()
        }
    }
}

use tls_site::{Endpoint, TestSite};

// ---------------------------------------------------------------------------
// 站点素材与宿主装备
// ---------------------------------------------------------------------------

const DOC_BODY: &str = "# Gallery 指南\n\n只读文档正文，含 [显式链接](https://example.com/next)。\n\n- 项目一\n- 项目二\n";

fn doc_sha_hex(body: &str) -> String {
    // 复用 ase-m0 协议层的摘要实现（单一语义源；不引测试侧第二份 SHA-256）。
    ase_m0::autoweb::sha256_hex(body.as_bytes())
}

fn descriptor_json(doc_href: &str, sha_hex: &str, title: &str) -> String {
    format!(
        r#"{{"protocolVersion":1,"appId":"auto.demo.guide","release":"1.0.0","title":"{title}","entry":{{"kind":"autodown","href":"{doc_href}","sha256":"{sha_hex}"}},"assets":[],"services":[]}}"#
    )
}

/// 标准指南站点：`/guide/` 描述 + `/guide/guide.ad` 文档（ETag + Vary）。
fn install_guide_site(site: &TestSite) {
    site.set(
        "/guide/",
        Endpoint::ok(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./guide.ad", &doc_sha_hex(DOC_BODY), "Gallery 指南"),
            &[
                ("ETag", "\"d1\"".to_string()),
                ("Vary", "Accept".to_string()),
            ],
        ),
    );
    site.set(
        "/guide/guide.ad",
        Endpoint::ok(
            "text/vnd.autodown",
            DOC_BODY,
            &[
                ("ETag", "\"b1\"".to_string()),
                ("Vary", "Accept".to_string()),
            ],
        ),
    );
}

fn nav_limits(request_timeout_ms: u64) -> NavLimits {
    let mut limits = NavLimits::default();
    limits.request_timeout_ms = request_timeout_ms;
    limits
}

fn web_options(ca_pem: &[u8]) -> AutowebNavOptions {
    let mut options = AutowebNavOptions::default();
    options.extra_root_pem = Some(ca_pem.to_vec());
    options
}

fn fixture_root() -> PathBuf {
    let root = match std::env::var("ASE_M0_TEXT_ROOT") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            manifest.join("../../../auto-lang/experimental/ase-m0/fixtures")
        }
    };
    assert!(
        root.join("counter-a/main.at").is_file() && root.join("clock/main.at").is_file(),
        "preparation failure: ASE fixtures missing (ASE_M0_TEXT_ROOT or the group sibling \
         auto-lang tree)"
    );
    root
}

mod tempdir {
    use std::path::PathBuf;
    pub struct TempDirGuard(pub PathBuf);
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    pub fn new(tag: &str) -> TempDirGuard {
        let path = std::env::temp_dir().join(format!(
            "p1119-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        TempDirGuard(path)
    }
}

fn web_host(ca_pem: &[u8], config: &tempdir::TempDirGuard) -> AutoscapeHost {
    AutoscapeHost::with_roots(fixture_root(), config.0.clone())
        .with_autoweb_options(web_options(ca_pem))
}

/// 步进直到谓词命中或预算耗尽（返回是否命中；每轮 5ms）。
fn step_until(
    host: &mut AutoscapeHost,
    budget: Duration,
    mut predicate: impl FnMut(&AutoscapeHost) -> bool,
) -> bool {
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        host.step();
        if predicate(host) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    host.step();
    predicate(host)
}

/// 轮询站点日志直到谓词命中（RF-1：请求记录仅在连接终止事件落账——
/// aborted 条目在取消后延迟出现，断言前必须等它落账；预算 3s）。
fn wait_for_log(
    site: &TestSite,
    mut predicate: impl FnMut(&[tls_site::ServedRequest]) -> bool,
) -> bool {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if predicate(&site.log()) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    predicate(&site.log())
}

/// 轮询直到服务器**已见**（头解析即记）指定路径的请求数达标——取消类
/// 用例须先确认请求在途，再触发停止/关页（RF-1 两阶段观察面）。
fn wait_for_seen(site: &TestSite, path: &str, at_least: usize) -> bool {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if site.seen_count(path) >= at_least {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    site.seen_count(path) >= at_least
}

fn committed_web<'a>(
    host: &'a AutoscapeHost,
    tab: usize,
) -> Option<&'a auto_ui_runtime::ui::ase_browser_shell::WebDocPage> {
    host.tabs()
        .get(tab)
        .and_then(|t| t.committed.as_ref())
        .and_then(|page| page.web.as_ref())
}

// ---------------------------------------------------------------------------
// V-01：真实 TLS 全链 + 协商 + 已下载描述≠已打开文档
// ---------------------------------------------------------------------------

#[test]
fn p1119_v01_real_tls_chain_negotiation_and_single_commit() {
    let started = Instant::now();
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    install_guide_site(&site);
    let config = tempdir::new("v01");
    let mut host = web_host(&ca_pem, &config);

    host.navigate(&site.url("/guide/"))
        .expect("navigate accepted");
    assert!(
        step_until(&mut host, Duration::from_secs(5), |h| {
            committed_web(h, h.active_index()).is_some()
        }),
        "web document commits within budget; nav_log: {:?}",
        host.nav_log
    );

    let web = committed_web(&host, host.active_index()).unwrap();
    assert_eq!(web.title, "Gallery 指南");
    assert_eq!(web.release, "1.0.0");
    assert_eq!(web.body, DOC_BODY);
    assert_eq!(web.sha256_hex, doc_sha_hex(DOC_BODY));
    assert_eq!(web.canonical, site.url("/guide/"));
    // 历史：Gallery → 指南（标题=文档标题）。
    let tab = host.active_tab();
    assert_eq!(tab.history.len(), 2);
    assert_eq!(tab.history[1].address, site.url("/guide/"));
    assert_eq!(tab.history[1].title, "Gallery 指南");
    assert_eq!(tab.committed_address(), Some(site.url("/guide/").as_str()));
    assert!(host.status_line().contains("网络文档"));
    // 恰一次提交；描述下载不构成打开（提交事件只出现一次）。
    let commits = host
        .nav_log
        .iter()
        .filter(|e| e.action == "web-commit" && e.outcome.starts_with("committed"))
        .count();
    assert_eq!(commits, 1, "{:?}", host.nav_log);
    // 服务器观察：协商头精确；两跳真实 TLS 请求。
    let descriptor_hits = site.requests_to("/guide/");
    assert_eq!(descriptor_hits.len(), 1);
    assert_eq!(
        descriptor_hits[0].accept,
        "application/vnd.autoweb.deployment+json"
    );
    assert_eq!(descriptor_hits[0].outcome, "served");
    let doc_hits = site.requests_to("/guide/guide.ad");
    assert_eq!(doc_hits.len(), 1);
    assert_eq!(doc_hits[0].accept, "text/vnd.autodown");
    assert_eq!(doc_hits[0].if_none_match, None);
    assert!(started.elapsed().as_secs() < 10);
}

#[test]
fn p1119_v01_alias_spellings_share_origin_history_and_cache() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    install_guide_site(&site);
    let config = tempdir::new("v01-alias");
    let mut host = web_host(&ca_pem, &config);

    // https 写法打开。
    host.navigate(&site.url("/guide/")).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| {
        committed_web(h, h.active_index()).is_some()
    }));
    // 原生专用写法打开同一资源（新页签）。
    let alias = format!("autoweb://localhost:{}/guide/", site.port);
    host.new_tab();
    host.navigate(&alias).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| {
        committed_web(h, h.active_index()).is_some()
    }));

    let tab = host.active_tab();
    assert_eq!(tab.committed_address(), Some(alias.as_str()));
    assert_eq!(tab.history[1].address, alias, "历史保留原生专用写法");
    assert_eq!(
        committed_web(&host, host.active_index()).unwrap().canonical,
        site.url("/guide/"),
        "规范来源同为 https"
    );
    // 共用来源=共用缓存：第二次导航对描述与文档都发 If-None-Match。
    let descriptor_hits = site.requests_to("/guide/");
    assert_eq!(descriptor_hits.len(), 2);
    assert_eq!(descriptor_hits[1].if_none_match.as_deref(), Some("\"d1\""));
    let doc_hits = site.requests_to("/guide/guide.ad");
    assert_eq!(doc_hits.len(), 2);
    assert_eq!(doc_hits[1].if_none_match.as_deref(), Some("\"b1\""));
    // 收藏保留打开方式，重开仍走该入口（RF-7：真实断言——服务器侧
    // 观察第三次复验请求 + 提交地址/历史不动 + 回退/前进换页语义）。
    host.add_favorite().unwrap();
    let bookmark_address = host.bookmarks()[0].address.clone();
    assert_eq!(bookmark_address, alias);
    host.navigate(&bookmark_address).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| {
        h.active_tab().pending.is_none()
    }));
    assert_eq!(host.active_tab().committed_address(), Some(alias.as_str()));
    assert!(
        wait_for_log(&site, |log| log
            .iter()
            .filter(|r| r.path == "/guide/")
            .count()
            >= 3),
        "favorite reopen revalidates again"
    );
    let reopen_hits = site.requests_to("/guide/");
    assert_eq!(reopen_hits.len(), 3);
    assert!(
        reopen_hits[2].if_none_match.is_some(),
        "重开走条件复验: {:?}",
        reopen_hits[2]
    );
    assert_eq!(
        host.active_tab().history.len(),
        2,
        "同地址刷新/重开不加历史"
    );
    // 回退 → Gallery；前进 → 回到网络文档（未驻留历史页显式重开语义）。
    host.back().unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| h
        .active_tab()
        .pending
        .is_none()));
    assert_eq!(
        host.active_tab().committed_address(),
        Some("app://auto.gallery/"),
        "回退到 Gallery"
    );
    host.forward().unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| h
        .active_tab()
        .pending
        .is_none()));
    assert_eq!(
        host.active_tab().committed_address(),
        Some(alias.as_str()),
        "前进回到网络文档（复验打开）"
    );
    assert!(site.requests_to("/guide/").len() >= 4);
}

// ---------------------------------------------------------------------------
// V-03：TLS/证书负控 + 状态/媒体/描述/文档负控 + 重定向
// ---------------------------------------------------------------------------

#[test]
fn p1119_v03_untrusted_certificate_and_hostname_mismatch_refused() {
    // 恶意站：证书由独立"恶意 CA"签发（不在被测客户端信任面内）。
    let (rogue, _rogue_ca_pem, _rogue_cert) = tls_site::spawn("rogue", &[]);
    rogue.set(
        "/guide/",
        Endpoint::ok("application/vnd.autoweb.deployment+json", "{}", &[]),
    );
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    install_guide_site(&site);
    let config = tempdir::new("v03-cert");
    let mut host = web_host(&ca_pem, &config);

    // 不信任证书：站点存在但客户端信任面只含受信 CA。
    host.navigate(&rogue.url("/guide/")).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(6), |h| {
        h.active_tab().pending.is_none()
    }));
    let failure = host.active_tab().failure.as_ref().expect("named failure");
    assert!(
        failure.message.contains("transport") || failure.message.contains("certificate"),
        "untrusted cert named failure: {}",
        failure.message
    );
    assert_eq!(
        host.active_tab().committed_address(),
        Some("app://auto.gallery/"),
        "已提交页面保持不变"
    );

    // 主机名不匹配：证书 SAN 给别的名字（受信 CA 签发）。
    let (wrong_host, wrong_ca_pem, _c3) = tls_site::spawn("wrong-host", &["wrong.test"]);
    wrong_host.set(
        "/guide/",
        Endpoint::ok("application/vnd.autoweb.deployment+json", "{}", &[]),
    );
    let config2 = tempdir::new("v03-host");
    let mut host2 = web_host(&wrong_ca_pem, &config2);
    host2.navigate(&wrong_host.url("/guide/")).unwrap();
    assert!(step_until(&mut host2, Duration::from_secs(6), |h| {
        h.active_tab().pending.is_none()
    }));
    let failure = host2.active_tab().failure.as_ref().expect("named failure");
    assert!(
        failure.message.contains("transport") || failure.message.contains("certificate"),
        "hostname mismatch named failure: {}",
        failure.message
    );
}

#[test]
fn p1119_v03_status_media_and_descriptor_negatives() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    let config = tempdir::new("v03-neg");
    let mut host = web_host(&ca_pem, &config);

    let cases: Vec<(&str, Endpoint, &str)> = vec![
        (
            "/missing/",
            Endpoint {
                status: 404,
                headers: vec![],
                body: vec![],
                trickle: None,
            },
            "404",
        ),
        (
            "/nope/",
            Endpoint {
                status: 406,
                headers: vec![],
                body: vec![],
                trickle: None,
            },
            "406",
        ),
        (
            "/boom/",
            Endpoint {
                status: 500,
                headers: vec![],
                body: vec![],
                trickle: None,
            },
            "server error",
        ),
        (
            "/html/",
            Endpoint::ok("text/html", "<html><body>hi</body></html>", &[]),
            "HTML",
        ),
        (
            "/json/",
            Endpoint::ok("application/json", "{}", &[]),
            "unexpected media type",
        ),
        (
            "/dupkey/",
            Endpoint::ok(
                "application/vnd.autoweb.deployment+json",
                r#"{"protocolVersion":1,"protocolVersion":1,"appId":"a","release":"1","title":"t","entry":{"kind":"autodown","href":"./g.ad","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"assets":[],"services":[]}"#,
                &[],
            ),
            "strict v1",
        ),
        (
            "/huge/",
            Endpoint::ok(
                "application/vnd.autoweb.deployment+json",
                &"x".repeat(70 * 1024),
                &[],
            ),
            "exceeds limit",
        ),
    ];
    for (path, endpoint, needle) in cases {
        site.set(path, endpoint);
        host.navigate(&site.url(path)).unwrap();
        assert!(
            step_until(&mut host, Duration::from_secs(6), |h| h
                .active_tab()
                .pending
                .is_none()),
            "{path}: pending resolves"
        );
        let failure = host.active_tab().failure.as_ref().expect(path);
        assert!(
            failure.message.contains(needle),
            "{path}: {} lacks {needle:?}",
            failure.message
        );
        assert_eq!(
            host.active_tab().committed_address(),
            Some("app://auto.gallery/"),
            "{path}: committed unchanged"
        );
    }
    // 明文 http 地址在解析层即拒（降级），不触网。
    assert!(host
        .navigate(&format!("http://localhost:{}/guide/", site.port))
        .is_err());
    assert!(site.requests_to("/guide/").is_empty());
    // 未支持 scheme 同样具名拒。
    let refused = resolve_address("ftp://example.com/x", host.apps()).unwrap_err();
    assert!(refused.0.contains("app") || refused.0.contains("scheme"));
}

#[test]
fn p1119_v03_document_integrity_and_readonly_admission_negatives() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    let tampered = "# 被篡改的正文\n";
    let bad_doc = "# 标题\n\n![隐式图片](https://evil.example/x.png)\n";
    let embed_doc = "# 标题\n\n$embed(src=https://evil.example/e)\n";
    // 摘要不符：描述声明 DOC_BODY 的摘要，正文却是 tampered。
    site.set(
        "/tampered/",
        Endpoint::ok(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./body.ad", &doc_sha_hex(DOC_BODY), "T"),
            &[],
        ),
    );
    site.set(
        "/tampered/body.ad",
        Endpoint::ok("text/vnd.autodown", tampered, &[]),
    );
    // 只读准入：图片 / 组件指令（摘要各自正确）。
    site.set(
        "/image/",
        Endpoint::ok(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./body.ad", &doc_sha_hex(bad_doc), "I"),
            &[],
        ),
    );
    site.set(
        "/image/body.ad",
        Endpoint::ok("text/vnd.autodown", bad_doc, &[]),
    );
    site.set(
        "/embed/",
        Endpoint::ok(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./body.ad", &doc_sha_hex(embed_doc), "E"),
            &[],
        ),
    );
    site.set(
        "/embed/body.ad",
        Endpoint::ok("text/vnd.autodown", embed_doc, &[]),
    );

    let config = tempdir::new("v03-doc");
    let mut host = web_host(&ca_pem, &config);
    for (path, needle) in [
        ("/tampered/", "mismatch"),
        ("/image/", "image"),
        ("/embed/", "`$`"),
    ] {
        host.navigate(&site.url(path)).unwrap();
        assert!(
            step_until(&mut host, Duration::from_secs(6), |h| h
                .active_tab()
                .pending
                .is_none()),
            "{path}: pending resolves"
        );
        let failure = host.active_tab().failure.as_ref().expect(path);
        assert!(
            failure.message.contains(needle),
            "{path}: {} lacks {needle:?}",
            failure.message
        );
        assert!(
            committed_web(&host, host.active_index()).is_none(),
            "{path} 无提交"
        );
        // 描述与正文都已实际请求（准入发生在渲染前、传输后）。
        assert_eq!(
            site.requests_to(&format!("{path}body.ad")).len(),
            1,
            "{path}"
        );
    }
}

#[test]
fn p1119_v03_redirect_negatives_refuse_before_next_request() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    install_guide_site(&site);
    let redirect = |location: &str| Endpoint {
        status: 302,
        headers: vec![("Location".to_string(), location.to_string())],
        body: vec![],
        trickle: None,
    };
    site.set("/cross/", redirect("https://evil.example/guide/"));
    site.set(
        "/down/",
        redirect(&format!("http://localhost:{}/guide/", site.port)),
    );
    site.set("/a/", redirect("/b/"));
    site.set("/b/", redirect("/c/"));
    site.set("/c/", redirect("/d/"));
    site.set("/d/", redirect("/e/"));

    let config = tempdir::new("v03-redir");
    let mut host = web_host(&ca_pem, &config);
    for (path, needle) in [
        ("/cross/", "cross-origin"),
        ("/down/", "https"),
        ("/a/", "hops"),
    ] {
        host.navigate(&site.url(path)).unwrap();
        assert!(
            step_until(&mut host, Duration::from_secs(6), |h| h
                .active_tab()
                .pending
                .is_none()),
            "{path}: pending resolves"
        );
        let failure = host.active_tab().failure.as_ref().expect(path);
        assert!(
            failure.message.contains(needle),
            "{path}: {} lacks {needle:?}",
            failure.message
        );
    }
    // 跨源/降级在下一请求前拒绝：各自恰好 1 个入口请求；同源跳链在
    // 第 4 跳前拒（/a/→/b/→/c/→/d/ 均为本站请求，第 5 目标 /e/ 未触达）。
    assert_eq!(site.requests_to("/cross/").len(), 1, "跨源只发入口请求");
    assert_eq!(site.requests_to("/down/").len(), 1, "降级只发入口请求");
    for path in ["/a/", "/b/", "/c/", "/d/"] {
        assert_eq!(site.requests_to(path).len(), 1, "{path} 恰一次");
    }
    assert!(site.requests_to("/e/").is_empty(), "超跳后的目标未请求");
    // 同源 3 跳内允许（正例）。
    site.set("/ok3/", redirect("/ok2/"));
    site.set("/ok2/", redirect("/guide/"));
    host.navigate(&site.url("/ok3/")).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(6), |h| {
        committed_web(h, h.active_index()).is_some()
    }));
}

// ---------------------------------------------------------------------------
// V-02：真实服务器的缓存轨迹（304/新版本/坏更新/no-store）
// ---------------------------------------------------------------------------

#[test]
fn p1119_v02_revalidation_304_and_new_version_on_real_tls() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    install_guide_site(&site);
    let config = tempdir::new("v02");
    let mut host = web_host(&ca_pem, &config);

    // 首访：全量下载。
    host.navigate(&site.url("/guide/")).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| {
        committed_web(h, h.active_index()).is_some()
    }));
    // 再访：If-None-Match + 304 → 内容一致、正文不重复下载。
    site.set(
        "/guide/",
        Endpoint::ok(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./guide.ad", &doc_sha_hex(DOC_BODY), "Gallery 指南"),
            &[
                ("ETag", "\"d1\"".to_string()),
                ("Vary", "Accept".to_string()),
            ],
        ),
    );
    // 304 由路由表无法表达——换站点响应：直接给 304 端点。
    let not_modified = Endpoint {
        status: 304,
        headers: vec![("ETag".to_string(), "\"d1\"".to_string())],
        body: vec![],
        trickle: None,
    };
    site.set("/guide/", not_modified.clone());
    site.set(
        "/guide/guide.ad",
        Endpoint {
            status: 304,
            headers: vec![("ETag".to_string(), "\"b1\"".to_string())],
            body: vec![],
            trickle: None,
        },
    );
    host.navigate(&site.url("/guide/")).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| {
        committed_web(h, h.active_index()).is_some() && h.active_tab().pending.is_none()
    }));
    assert_eq!(
        committed_web(&host, host.active_index()).unwrap().body,
        DOC_BODY
    );
    let doc_hits = site.requests_to("/guide/guide.ad");
    assert_eq!(doc_hits.len(), 2);
    assert_eq!(doc_hits[1].if_none_match.as_deref(), Some("\"b1\""));
    assert_eq!(doc_hits[1].body_bytes_sent, 0, "304 不重下正文");
    let commit = host
        .nav_log
        .iter()
        .rev()
        .find(|e| e.action == "web-commit" && e.outcome.starts_with("committed"))
        .unwrap();
    assert!(
        commit.outcome.contains("cache_reused=true"),
        "{}",
        commit.outcome
    );

    // 新版本：描述+文档内容与 ETag 更新 → 新正文提交。
    let new_doc = "# v2\n\n更新后的正文。\n";
    site.set(
        "/guide/",
        Endpoint::ok(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./guide.ad", &doc_sha_hex(new_doc), "Gallery 指南 v2"),
            &[
                ("ETag", "\"d2\"".to_string()),
                ("Vary", "Accept".to_string()),
            ],
        ),
    );
    site.set(
        "/guide/guide.ad",
        Endpoint::ok(
            "text/vnd.autodown",
            new_doc,
            &[
                ("ETag", "\"b2\"".to_string()),
                ("Vary", "Accept".to_string()),
            ],
        ),
    );
    host.navigate(&site.url("/guide/")).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| {
        committed_web(h, h.active_index())
            .map(|w| w.body == new_doc)
            .unwrap_or(false)
    }));
    assert_eq!(
        committed_web(&host, host.active_index()).unwrap().title,
        "Gallery 指南 v2"
    );
}

#[test]
fn p1119_v02_bad_update_keeps_old_commit_and_no_store_bypasses_cache() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    install_guide_site(&site);
    let config = tempdir::new("v02-bad");
    let mut host = web_host(&ca_pem, &config);

    host.navigate(&site.url("/guide/")).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| {
        committed_web(h, h.active_index()).is_some()
    }));
    let old_body = committed_web(&host, host.active_index())
        .unwrap()
        .body
        .clone();

    // 坏更新：正文换了字节但描述摘要未更新 → 拒；已提交保留旧有效内容。
    let tampered = "# 中间人替换的正文\n";
    site.set(
        "/guide/guide.ad",
        Endpoint::ok(
            "text/vnd.autodown",
            tampered,
            &[
                ("ETag", "\"b3\"".to_string()),
                ("Vary", "Accept".to_string()),
            ],
        ),
    );
    host.navigate(&site.url("/guide/")).unwrap();
    assert!(step_until(&mut host, Duration::from_secs(5), |h| h
        .active_tab()
        .pending
        .is_none()));
    let failure = host.active_tab().failure.as_ref().expect("named refusal");
    assert!(failure.message.contains("mismatch"), "{}", failure.message);
    assert_eq!(
        committed_web(&host, host.active_index()).unwrap().body,
        old_body,
        "旧有效提交保留"
    );

    // no-store：描述带 no-store → 每次全量 GET，永不带条件头。
    site.set(
        "/nostore/",
        Endpoint::ok(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./body.ad", &doc_sha_hex(DOC_BODY), "N"),
            &[
                ("ETag", "\"nd1\"".to_string()),
                ("Cache-Control", "no-store".to_string()),
            ],
        ),
    );
    site.set(
        "/nostore/body.ad",
        Endpoint::ok(
            "text/vnd.autodown",
            DOC_BODY,
            &[
                ("ETag", "\"nb1\"".to_string()),
                ("Cache-Control", "no-store".to_string()),
            ],
        ),
    );
    for _ in 0..2 {
        host.navigate(&site.url("/nostore/")).unwrap();
        // 等待本次导航完成（pending 置空=完成裁决已过），再对账缓存轨迹。
        assert!(step_until(&mut host, Duration::from_secs(5), |h| h
            .active_tab()
            .pending
            .is_none()));
    }
    assert!(committed_web(&host, host.active_index())
        .unwrap()
        .canonical
        .ends_with("/nostore/"));
    let hits = site.requests_to("/nostore/");
    assert_eq!(hits.len(), 2);
    assert!(
        hits.iter().all(|r| r.if_none_match.is_none()),
        "no-store 不复验"
    );
    assert!(
        hits.iter().all(|r| r.body_bytes_sent > 0),
        "no-store 全量下载"
    );
}

// ---------------------------------------------------------------------------
// V-04：停止/关页/关窗/抢占/后台页签完成 + 真实请求释放
// ---------------------------------------------------------------------------

/// 慢体端点（trickle：分块间隔）——取消/释放观察面。
fn slow_endpoint(content_type: &str, body: &str, chunks: usize, per_chunk_ms: u64) -> Endpoint {
    let mut endpoint = Endpoint::ok(content_type, body, &[]);
    endpoint.trickle = Some((chunks, per_chunk_ms));
    endpoint
}

#[test]
fn p1119_v04_stop_invalidates_and_releases_request() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    site.set(
        "/slowdoc/",
        slow_endpoint(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./body.ad", &doc_sha_hex(DOC_BODY), "S"),
            12,
            300,
        ),
    );
    site.set(
        "/slowdoc/body.ad",
        Endpoint::ok("text/vnd.autodown", DOC_BODY, &[]),
    );
    let config = tempdir::new("v04-stop");
    let mut host = AutoscapeHost::with_roots(fixture_root(), config.0.clone())
        .with_autoweb_options({
            let mut options = web_options(&ca_pem);
            options.limits = nav_limits(8000);
            options
        });

    host.navigate(&site.url("/slowdoc/")).unwrap();
    assert!(host.active_tab().pending.is_some());
    host.step();
    assert!(
        wait_for_seen(&site, "/slowdoc/", 1),
        "slow descriptor reaches the server before stop"
    );
    // 停止：pending 立即清空，已提交页面不变。
    host.stop().unwrap();
    assert!(host.active_tab().pending.is_none());
    assert_eq!(
        host.active_tab().committed_address(),
        Some("app://auto.gallery/")
    );
    // 真实请求释放：worker 在取消后 ≤2s 回到 0；服务器侧连接中止记录
    // 落账（NF-1：终止事件延迟落账——断言前必须等它出现）后判定。
    let released = step_until(&mut host, Duration::from_secs(2), |h| {
        h.web_in_flight() == 0
    });
    assert!(
        released,
        "worker released after stop (in_flight={})",
        host.web_in_flight()
    );
    assert!(
        wait_for_log(&site, |log| log
            .iter()
            .any(|r| r.path == "/slowdoc/" && r.outcome == "aborted")),
        "stop abort lands in the server log"
    );
    let slow_hits = site.requests_to("/slowdoc/");
    assert_eq!(slow_hits.len(), 1, "{:?}", slow_hits);
    assert!(
        slow_hits.iter().all(|r| r.outcome == "aborted"),
        "慢体连接被中止: {:?}",
        slow_hits
    );
    // 迟到完成零副作用：已提交地址与历史不变，无 web 提交事件。
    std::thread::sleep(Duration::from_millis(400));
    host.step();
    assert_eq!(
        host.active_tab().committed_address(),
        Some("app://auto.gallery/")
    );
    assert!(host
        .nav_log
        .iter()
        .all(|e| !(e.action == "web-commit" && e.outcome.starts_with("committed"))));
    // 停止不产生失败条目（1118 语义：用户停止≠失败重试面）；恢复快速
    // 端点后同一地址新代次直接可达。
    assert!(
        host.retry_failed().is_err(),
        "用户停止无失败条目（retry 面归失败请求）"
    );
    site.set(
        "/slowdoc/",
        Endpoint::ok(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./body.ad", &doc_sha_hex(DOC_BODY), "S"),
            &[],
        ),
    );
    host.navigate(&site.url("/slowdoc/")).unwrap();
    assert!(
        step_until(&mut host, Duration::from_secs(5), |h| {
            committed_web(h, h.active_index()).is_some()
        }),
        "nav_log: {:?}",
        host.nav_log
    );
}

#[test]
fn p1119_v04_background_tab_completion_and_counter_responsiveness() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    site.set(
        "/slow/",
        slow_endpoint(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./body.ad", &doc_sha_hex(DOC_BODY), "Slow"),
            6,
            150,
        ),
    );
    site.set(
        "/slow/body.ad",
        Endpoint::ok("text/vnd.autodown", DOC_BODY, &[]),
    );
    let config = tempdir::new("v04-bg");
    let mut host = web_host(&ca_pem, &config);

    // 页签 0：慢网络文档；页签 1：真实 Counter（外壳保持响应的证据）。
    host.navigate(&site.url("/slow/")).unwrap();
    let counter_tab = host.tabs().len();
    host.new_tab();
    host.navigate("app://auto.demo.counter/").unwrap();
    assert!(step_until(&mut host, Duration::from_secs(6), |h| {
        h.tabs()
            .get(counter_tab)
            .and_then(|t| t.committed.as_ref())
            .and_then(|p| p.session)
            .is_some()
    }));
    // Counter 真实业务输入（慢网络导航在途时仍接受输入；引擎权威 count=1）。
    let counter_session = host.tabs()[counter_tab]
        .committed
        .as_ref()
        .and_then(|page| page.session)
        .expect("counter session mounted");
    host.click(2).unwrap(); // Counter：2=+
    assert!(
        step_until(&mut host, Duration::from_secs(4), |h| {
            h.session_host()
                .engine_states(counter_session)
                .map(|row| {
                    row.iter().any(|(name, value)| {
                        name == "count"
                            && matches!(
                                value,
                                auto_ui_runtime::ui::ace_state_bridge::AceStateValue::Int(1)
                            )
                    })
                })
                .unwrap_or(false)
        }),
        "Counter 接受输入（+1）: {:?}",
        host.session_host().evidence()
    );
    // 网络完成只更新所属页签 0（后台完成）；页签 1 的 Counter 提交不动。
    assert!(step_until(&mut host, Duration::from_secs(6), |h| {
        committed_web(h, 0).is_some()
    }));
    assert_eq!(host.active_index(), counter_tab);
    assert!(
        host.tabs()[counter_tab]
            .committed
            .as_ref()
            .unwrap()
            .session
            .is_some(),
        "Counter 页签不受后台完成影响"
    );
    assert!(
        host.status_line().contains("本地应用"),
        "{}",
        host.status_line()
    );
}

#[test]
fn p1119_v04_supersede_a_to_b_only_newest_commits() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    for (path, title) in [("/a/", "A"), ("/b/", "B")] {
        site.set(
            path,
            slow_endpoint(
                "application/vnd.autoweb.deployment+json",
                &descriptor_json("./body.ad", &doc_sha_hex(DOC_BODY), title),
                8,
                150,
            ),
        );
        site.set(
            &format!("{path}body.ad"),
            Endpoint::ok("text/vnd.autodown", DOC_BODY, &[]),
        );
    }
    let config = tempdir::new("v04-ab");
    let mut host = web_host(&ca_pem, &config);

    host.navigate(&site.url("/a/")).unwrap();
    host.step();
    std::thread::sleep(Duration::from_millis(120));
    host.navigate(&site.url("/b/")).unwrap(); // 抢占 A
    host.step();
    std::thread::sleep(Duration::from_millis(120));
    host.navigate(&site.url("/a/")).unwrap(); // 抢占 B（A 新代次）
    assert!(step_until(&mut host, Duration::from_secs(6), |h| {
        committed_web(h, h.active_index())
            .map(|w| w.title == "A")
            .unwrap_or(false)
    }));
    // 只有最新 A 提交；被抢占的 A1/B 连接中止（服务器观察）。
    let commits: Vec<_> = host
        .nav_log
        .iter()
        .filter(|e| e.action == "web-commit" && e.outcome.starts_with("committed"))
        .collect();
    assert_eq!(commits.len(), 1, "{:?}", host.nav_log);
    assert_eq!(commits[0].address, site.url("/a/"));
    // 迟到完成按代次丢弃（nav_log 记录 stale drop ≥1）。
    std::thread::sleep(Duration::from_millis(600));
    host.step();
    assert!(
        host.nav_log
            .iter()
            .any(|e| e.action == "web-commit" && e.outcome.contains("stale completion dropped")),
        "{:?}",
        host.nav_log
    );
}

#[test]
fn p1119_v04_close_tab_and_close_all_release_pending_navigation() {
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    site.set(
        "/slow/",
        slow_endpoint(
            "application/vnd.autoweb.deployment+json",
            &descriptor_json("./body.ad", &doc_sha_hex(DOC_BODY), "S"),
            10,
            200,
        ),
    );
    let config = tempdir::new("v04-close");
    let mut host = web_host(&ca_pem, &config);

    // 关页释放：先确认服务器已见到慢导航请求（消除接取竞态）→ 关闭该
    // 页签 → 中止记录落账后断言（RF-1：站点记录仅在终止事件落账）。
    host.navigate(&site.url("/slow/")).unwrap();
    host.step();
    assert!(
        wait_for_seen(&site, "/slow/", 1),
        "slow request reaches the server before close"
    );
    host.close_tab(0).unwrap();
    assert!(host.web_in_flight() <= 1);
    assert!(step_until(&mut host, Duration::from_secs(2), |h| h
        .web_in_flight()
        == 0));
    assert!(
        wait_for_log(&site, |log| log
            .iter()
            .any(|r| r.path == "/slow/" && r.outcome == "aborted")),
        "close-tab abort lands in the server log"
    );
    let slow_hits = site.requests_to("/slow/");
    assert_eq!(slow_hits.len(), 1, "{:?}", slow_hits);
    assert!(
        slow_hits.iter().all(|r| r.outcome == "aborted"),
        "{:?}",
        slow_hits
    );

    // 关窗收口：页签 A 慢导航（确认触网后）+ 页签 B 排队命令 → close_all。
    // B 的取消旗标在 worker 接取前已置——永不触网（单飞行语义），服务
    // 器只见到飞行中的 A；A 中止落账后总账恰 2 条且全 aborted。
    host.navigate(&site.url("/slow/")).unwrap();
    host.step();
    assert!(
        wait_for_seen(&site, "/slow/", 2),
        "second slow request reaches the server before close_all"
    );
    host.new_tab();
    host.navigate(&site.url("/slow/")).unwrap(); // 排队（worker 单飞行）
    host.close_all().unwrap();
    assert!(step_until(&mut host, Duration::from_secs(2), |h| h
        .web_in_flight()
        == 0));
    assert!(
        wait_for_log(&site, |log| log
            .iter()
            .filter(|r| r.path == "/slow/")
            .all(|r| r.outcome == "aborted")
            && log.iter().filter(|r| r.path == "/slow/").count() >= 2),
        "close-all abort lands for the in-flight request"
    );
    let aborted_all = site.requests_to("/slow/");
    assert_eq!(
        aborted_all.len(),
        2,
        "关页 1 条 + 关窗飞行中 1 条；排队命令零触网: {:?}",
        aborted_all
    );
    assert!(
        aborted_all.iter().all(|r| r.outcome == "aborted"),
        "{:?}",
        aborted_all
    );
}

// ---------------------------------------------------------------------------
// V-04 真窗档（AUTO_UI_ASE_M0_E2E=1 排窗单例后跑）
// ---------------------------------------------------------------------------

#[test]
fn p1119_v04_window_e2e_autoweb_readonly_navigation() {
    if std::env::var("AUTO_UI_ASE_M0_E2E").ok().as_deref() != Some("1") {
        return;
    }
    let (site, ca_pem, _cert) = tls_site::spawn("trusted", &[]);
    install_guide_site(&site);
    let ca_dir = tempdir::new("v04-win-ca");
    let ca_path = ca_dir.0.join("test-ca.pem");
    std::fs::write(&ca_path, &ca_pem).unwrap();
    let out_dir = tempdir::new("v04-win-out");
    let evidence_path = out_dir.0.join("evidence.json");

    // CARGO_TARGET_TMPDIR = <target>/tmp；示例二进制在 <target>/debug/examples。
    let exe =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("../debug/examples/autoscape_window.exe");
    assert!(
        exe.is_file(),
        "preparation failure: build examples/autoscape_window first"
    );
    let script = format!("2:open:{}", site.url("/guide/"));
    let status = std::process::Command::new(&exe)
        .arg("--evidence")
        .arg(&evidence_path)
        .arg("--autoweb-root-ca")
        .arg(&ca_path)
        .arg("--config-home")
        .arg(out_dir.0.join("config"))
        .arg("--hold-ms")
        .arg("9000")
        // 网络导航是异步完成（worker→事件→提交）：观察线程退出前须给
        // TLS 链路留足时间——120 帧 ≈ 2.4s @20ms tick（localhost TLS 完成
        // ~300ms 的 8 倍裕量；首次 8 帧早退会与完成竞态）。
        .arg("--min-frames")
        .arg("120")
        .arg("--script")
        .arg(&script)
        .status()
        .expect("window exe runs");
    assert!(status.success(), "window exit: {status}");
    let evidence_text = std::fs::read_to_string(&evidence_path).unwrap();
    let evidence: serde_json::Value = serde_json::from_str(&evidence_text)
        .unwrap_or_else(|e| panic!("evidence parse: {e}; raw={evidence_text}"));
    let tabs = evidence["tabs"]
        .as_array()
        .unwrap_or_else(|| panic!("no tabs in evidence: {evidence_text}"));
    let web = tabs
        .iter()
        .find_map(|tab| tab["committed_web"].as_object())
        .unwrap_or_else(|| panic!("web document committed in the real window: {evidence_text}"));
    assert_eq!(web["title"], "Gallery 指南");
    assert_eq!(web["canonical"], site.url("/guide/"));
    assert!(evidence["frames_rendered"].as_u64().unwrap_or(0) >= 8);
    // 真实 TLS 全链对账（窗口进程实际发出的两跳）。
    assert_eq!(site.requests_to("/guide/").len(), 1);
    assert_eq!(site.requests_to("/guide/guide.ad").len(), 1);
}
