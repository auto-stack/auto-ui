//! PLAN-1119 AutoWeb 只读网络导航 worker（auto-ui 侧网络执行面）。
//!
//! 职责边界（T-01 冻结）：协议/缓存/准入全部在 ase-m0 `autoweb` 层；本
//! 模块只做网络执行——专用 reqwest blocking client（native-tls/schannel
//! 后端 + 系统可信根，构建期可注入附加根 PEM：信任配置**仅作用于本客户
//! 端**，不碰 OS 信任库；证书链+主机名恒常校验；redirect `Policy::none`，
//! 跳转策略归协议层）、有界后台请求（响应经 `io::Read` 分块读取 + 取消
//! 旗标检查 + 单请求超时 + 硬读上限兜底）与事件回投（`tab_id`+`generation`，
//! 外壳按代次裁决迟到完成）。
//!
//! worker 生命周期：单线程串行处理导航命令（单飞行——最新指令经外壳代次
//! 抢占旧请求：旧请求的取消旗标由外壳在停止/新导航/关页时置位，传输在
//! 下一块/下一跳前中止并释放连接）；命令通道随宿主 drop 关闭，线程自然
//! 退出。缓存由 worker 独占持有（规范 URL×表示；会话内有效）。

use ase_m0::autoweb::{
    AutowebCache, AutowebTransport, NavLimits, NavOutcome, ReadonlyNavigator, TransportFailure,
    TransportRequest, TransportResponse, WebAddress,
};
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 网络导航构造选项（宿主注入；测试注入测试 CA 与更小限额）。
#[derive(Clone)]
pub struct AutowebNavOptions {
    /// 附加信任根（PEM 字节；仅本客户端生效——隔离测试 CA 的受控注入面）。
    pub extra_root_pem: Option<Vec<u8>>,
    pub limits: NavLimits,
}

impl Default for AutowebNavOptions {
    fn default() -> Self {
        AutowebNavOptions {
            extra_root_pem: None,
            limits: NavLimits::default(),
        }
    }
}

/// 一次成功导航的可提交载荷（外壳 `CommittedPage.web` 的来源）。
#[derive(Clone, Debug)]
pub struct WebNavOutcome {
    pub display_address: String,
    pub canonical_url: String,
    pub title: String,
    pub release: String,
    pub app_id: String,
    pub document: String,
    pub document_sha256_hex: String,
    pub cache_reused: bool,
    pub request_count: usize,
}

/// worker → 宿主事件（step 内 drain；按代次裁决）。
pub enum WebNavEvent {
    Outcome {
        tab_id: usize,
        generation: u64,
        address: String,
        result: Result<WebNavOutcome, String>,
    },
}

struct WebStart {
    tab_id: usize,
    generation: u64,
    address: WebAddress,
    cancel: Arc<AtomicBool>,
}

/// 网络导航 worker 句柄（宿主持有；drop 后线程随通道关闭退出）。
pub struct WebNavWorker {
    cmd_tx: Sender<WebStart>,
    /// `Receiver` 非 `Sync`——宿主在全局静态中共享，经 `Mutex` 提供线程
    /// 安全 drain（锁只在 step 内短暂持有）。
    events: Mutex<Receiver<WebNavEvent>>,
    in_flight: Arc<AtomicUsize>,
}

impl WebNavWorker {
    /// 启动 worker 线程（缓存与 client 归 worker 独占）。
    pub fn spawn(options: AutowebNavOptions) -> Self {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<WebStart>();
        let (event_tx, event_rx) = std::sync::mpsc::channel::<WebNavEvent>();
        let in_flight = Arc::new(AtomicUsize::new(0));
        let worker_in_flight = Arc::clone(&in_flight);
        std::thread::Builder::new()
            .name("autoweb-nav".to_string())
            .spawn(move || {
                let client = build_client(&options);
                let mut cache = AutowebCache::new();
                // 硬读上限：协议限额之上的兜底（chunk 循环提前停——协议层
                // 对超限正文具名拒）。
                let hard_cap = options
                    .limits
                    .document_bytes
                    .max(options.limits.descriptor_bytes)
                    + 1024;
                while let Ok(start) = cmd_rx.recv() {
                    worker_in_flight.store(1, Ordering::Release);
                    let result = match &client {
                        Ok(client) => {
                            let mut transport = ReqwestAutowebTransport {
                                client: client.clone(),
                                cancel: Arc::clone(&start.cancel),
                                hard_cap,
                            };
                            let mut navigator =
                                ReadonlyNavigator::new(&mut transport, &mut cache, options.limits);
                            match navigator.navigate(&start.address) {
                                NavOutcome::Success(success) => Ok(WebNavOutcome {
                                    display_address: success.display_address,
                                    canonical_url: success.canonical_url,
                                    title: success.title,
                                    release: success.release,
                                    app_id: success.app_id,
                                    document: success.document,
                                    document_sha256_hex: success.document_sha256_hex,
                                    cache_reused: success.trace.cache_reused,
                                    request_count: success.trace.requests.len(),
                                }),
                                NavOutcome::Refused(refusal, _) => Err(refusal.message()),
                            }
                        }
                        Err(message) => Err(format!("autoweb client unavailable: {message}")),
                    };
                    let _ = event_tx.send(WebNavEvent::Outcome {
                        tab_id: start.tab_id,
                        generation: start.generation,
                        address: start.address.display_address(),
                        result,
                    });
                    worker_in_flight.store(0, Ordering::Release);
                }
            })
            .expect("autoweb nav worker thread");
        WebNavWorker {
            cmd_tx,
            events: Mutex::new(event_rx),
            in_flight,
        }
    }

    /// 受理一次导航；返回取消旗标（外壳在停止/抢占/关页时置位）。
    pub fn start(&self, tab_id: usize, generation: u64, address: WebAddress) -> Arc<AtomicBool> {
        let cancel = Arc::new(AtomicBool::new(false));
        let _ = self.cmd_tx.send(WebStart {
            tab_id,
            generation,
            address,
            cancel: Arc::clone(&cancel),
        });
        cancel
    }

    /// step 内非阻塞 drain。
    pub fn try_recv_event(&self) -> Option<WebNavEvent> {
        self.events.lock().ok()?.try_recv().ok()
    }

    /// 在途导航计数（0/1——worker 单飞行；释放证据面）。
    pub fn in_flight(&self) -> usize {
        self.in_flight.load(Ordering::Acquire)
    }
}

fn build_client(options: &AutowebNavOptions) -> Result<reqwest::blocking::Client, String> {
    // 默认 TLS 后端（Windows=schannel；证书链+主机名恒常校验）。附加根
    // 仅增补本客户端的信任面——OS 信任库不被修改。
    let mut builder = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5));
    if let Some(pem) = &options.extra_root_pem {
        let certificate = reqwest::Certificate::from_pem(pem)
            .map_err(|e| format!("invalid extra root PEM: {e}"))?;
        builder = builder.add_root_certificate(certificate);
    }
    builder
        .build()
        .map_err(|e| format!("reqwest client build failed: {e}"))
}

/// reqwest 传输适配（协议层 [`AutowebTransport`] 的网络执行）。
struct ReqwestAutowebTransport {
    client: reqwest::blocking::Client,
    cancel: Arc<AtomicBool>,
    hard_cap: usize,
}

impl AutowebTransport for ReqwestAutowebTransport {
    fn fetch(&mut self, req: &TransportRequest) -> Result<TransportResponse, TransportFailure> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(TransportFailure::Cancelled);
        }
        let mut builder = self
            .client
            .get(&req.url)
            .timeout(Duration::from_millis(req.timeout_ms));
        for (key, value) in &req.headers {
            builder = builder.header(*key, value);
        }
        let mut response = builder.send().map_err(map_reqwest_error)?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(key, value)| {
                Some((key.as_str().to_string(), value.to_str().ok()?.to_string()))
            })
            .collect();
        // 分块读取：块间检查取消旗标（置位即弃响应——连接随之释放）；
        // 硬上限兜底防无界读（协议层对超限正文具名拒）。
        let mut body = Vec::new();
        let mut buffer = [0u8; 16 * 1024];
        loop {
            if self.cancel.load(Ordering::Relaxed) {
                return Err(TransportFailure::Cancelled);
            }
            match response.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    body.extend_from_slice(&buffer[..read]);
                    if body.len() > self.hard_cap {
                        break;
                    }
                }
                Err(error) => {
                    return Err(if error.kind() == std::io::ErrorKind::TimedOut {
                        TransportFailure::Timeout
                    } else {
                        TransportFailure::Transport(format!("read body: {error}"))
                    })
                }
            }
        }
        Ok(TransportResponse {
            status,
            headers,
            body,
        })
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

fn map_reqwest_error(error: reqwest::Error) -> TransportFailure {
    if error.is_timeout() {
        TransportFailure::Timeout
    } else {
        TransportFailure::Transport(error.to_string())
    }
}
