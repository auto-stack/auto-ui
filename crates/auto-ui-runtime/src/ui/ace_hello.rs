//! PLAN-980 HELLO：首个 ACE AGE Node 文字窗口的最小接入桥。
//!
//! 链路（§5 HELLO 合同）：AGE node 普通源 → Schema 驱动解析/结构校验（979
//! H0 面）→ 唯一共享 verify → AV 真值执行文本表达式（`observe_age_h0`）→
//! F0 受检调用（967 冻结 provider，经 969 E0 门面 `run_ace` 的 foreign 车道，
//! 同一文本字节由 `text_byte_len` 证实被 provider 收到）→ 本组件把 AV 文本
//! 交现有 Iced 投影/窗口服务（`run_app` 泛型宿主，与 ui_hello_loader 等
//! example 同链）。
//!
//! 有界决定（T-HELLO-I，两机制比较）：选 `run_app::<C>` 泛型单窗宿主
//! （复用 View→into_iced 投影、Inter 字体、主题服务；组件零旧链依赖）；
//! 弃 daemon `run_session`（硬绑 `DynamicComponent`/旧 bytecode 链，接入需
//! 造假 VmBridge，合同明禁）。
//!
//! 纪律：F0 provider 经 `build_f0_text_provider` 构建于**稳定 root**，命中
//! 967 构建闭包的新鲜度缓存（PLAN-980 lang 侧）——预构建后测试/子进程零
//! rustc；AV 文本在桥内即复制为本组件所有的 Rust `String`（UI session 所有，
//! 不持有任何 AV 视图）。

use crate::ui::{Component, View};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// F0 受检调用证据（同一文本字节被 provider 收到的直接证明）。
#[derive(Clone, Debug)]
pub struct F0Evidence {
    pub provider: String,
    /// provider 计算的文本字节数（== AV 文本 UTF-8 字节数）。
    pub byte_len: i32,
    pub callee_count: u64,
    pub rejected_count: u64,
}

/// 一次 HELLO 装载的证据（AV 真值 + F0 分派 + 窗口渲染计数）。
#[derive(Clone, Debug, Default)]
pub struct HelloEvidence {
    pub source_path: String,
    pub node_name: String,
    pub head_role: String,
    pub grammar_id: String,
    /// AV 执行出的真实 UTF-8 文本（已复制为 UI 所有的 Rust 值）。
    pub av_text: String,
    pub f0: Option<F0Evidence>,
}

/// HELLO 会话（源装载 + AV 观察 + F0 受检调用）。
#[derive(Debug)]
pub struct HelloSession {
    /// 装载时的源文本（分派时重观察，防快照漂移）。
    src: String,
    pub evidence: HelloEvidence,
    /// iced view() 渲染计数（真实窗口渲染的观察面）。
    pub frames: std::sync::Arc<AtomicUsize>,
}

impl HelloSession {
    /// 装载 AGE node 源：Schema 驱动解析/校验 → 共享 verify → AV 真值。
    pub fn load(src: &str, source_path: &str) -> Result<Self, String> {
        let obs = ace_av::age_h0::observe_age_h0(
            src,
            source_path,
            ace_av::age_component::LegacyWidgetAlias::Reject,
        )?;
        let node = obs
            .nodes
            .first()
            .ok_or_else(|| "no AGE node in source".to_string())?;
        let text = node
            .view
            .texts
            .first()
            .ok_or_else(|| "the AGE node has no text leaf".to_string())?;
        Ok(Self {
            src: src.to_string(),
            evidence: HelloEvidence {
                source_path: source_path.to_string(),
                node_name: node.name.clone(),
                head_role: node.head_role.clone(),
                grammar_id: obs.grammar_id.clone(),
                av_text: text.value.clone(),
                f0: None,
            },
            frames: std::sync::Arc::new(AtomicUsize::new(0)),
        })
    }

    /// AXR0 模式（PLAN-980 RUST）：窗口文本来自**新 A2R 产物 exe 的真实
    /// stdout**（非 AV/旧 VM）——由窗口进程派生并显示其输出行。
    pub fn from_a2r_output(stdout_line: &str, source_path: &str) -> Self {
        Self {
            src: String::new(),
            evidence: HelloEvidence {
                source_path: source_path.to_string(),
                node_name: "A2R".to_string(),
                head_role: "a2r".to_string(),
                grammar_id: String::new(),
                av_text: stdout_line.to_string(),
                f0: None,
            },
            frames: std::sync::Arc::new(AtomicUsize::new(0)),
        }
    }

    /// F0 受检调用：合成 TextRecord 源把**同一 AV 文本字节**交给 967 冻结
    /// provider（`text_byte_len`），经 969 E0 门面分派并核对计数。
    pub fn run_f0(&mut self, provider: &str) -> Result<(), String> {
        // 分派时重观察：快照与新鲜 AV 真值必须逐字节一致（防会话陈旧/漂移），
        // provider 收到的字节据此核验——等式两侧不同源（T-HELLO-R 反例修复）。
        let fresh = ace_av::age_h0::observe_age_h0(
            &self.src,
            &self.evidence.source_path,
            ace_av::age_component::LegacyWidgetAlias::Reject,
        )
        .map_err(|e| format!("re-observe before dispatch: {e}"))?;
        let fresh_text = fresh
            .nodes
            .first()
            .and_then(|n| n.view.texts.first())
            .map(|t| t.value.clone())
            .ok_or_else(|| "re-observe found no text leaf".to_string())?;
        if fresh_text != self.evidence.av_text {
            return Err(format!(
                "AV text drifted since load: snapshot {:?} vs fresh {:?} bytes",
                self.evidence.av_text.len(),
                fresh_text.len()
            ));
        }
        auto_lang_ace_host::install().map_err(|e| format!("ace host install: {e}"))?;
        let root = stable_f0_root();
        if let Err(e) = auto_lang_ace_host::build_f0_text_provider(provider, &root) {
            // 幂等：进程内重复注册（缓存构建已就绪）不是错误；其余失败具名上抛。
            if !e.contains("already registered") {
                return Err(format!("f0 provider build (prep expected, cached): {e}"));
            }
        }
        let src = format!(
            "use.rs f0_text\nfn main() int {{ return text_byte_len({}) }}",
            escape_auto_string_for_source(&self.evidence.av_text)
        );
        let mut request = auto_lang_core::AceRequest::simple(&src);
        request.provider = Some(provider);
        let report = auto_lang_core::run_ace(auto_lang_core::AceEngineChoice::AceAV, &request);
        match &report.outcome {
            auto_lang_core::AceOutcome::Success {
                value: auto_lang_core::AceValue::I32(n),
                ..
            } => {
                let expected = self.evidence.av_text.len() as i32;
                if *n != expected {
                    return Err(format!(
                        "provider received {n} bytes but the AV text carries {expected}"
                    ));
                }
                self.evidence.f0 = Some(F0Evidence {
                    provider: provider.to_string(),
                    byte_len: *n,
                    callee_count: report.callee_count.unwrap_or(0),
                    rejected_count: report.rejected_count.unwrap_or(0),
                });
                Ok(())
            }
            other => Err(format!("f0 dispatch did not succeed: {other:?}")),
        }
    }
}

/// F0 provider 的稳定构建 root：环境显式指定，否则定位本仓 `target/` 下
/// （构建中间产物属地；预构建一次后全部走缓存，测试零 rustc）。
pub fn stable_f0_root() -> PathBuf {
    if let Ok(p) = std::env::var("ACE_HELLO_F0_ROOT") {
        return PathBuf::from(p);
    }
    let mut p = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    while p.file_name().map(|n| n != "target").unwrap_or(true) && p.pop() {}
    p.join("ace-hello-f0")
}

/// Auto 源字符串字面量转义（文本经源码字面量进入 F0/A2R 车道）。
pub fn escape_auto_string_for_source(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 真窗口组件（`run_app` 泛型宿主消费；会话经 OnceLock 注入，Default 成立）。
static HELLO_SESSION: OnceLock<HelloSession> = OnceLock::new();

pub fn install_hello_session(session: HelloSession) -> Result<(), HelloSession> {
    HELLO_SESSION.set(session)
}

pub fn hello_session() -> Option<&'static HelloSession> {
    HELLO_SESSION.get()
}

#[derive(Clone, Copy, Debug, Default)]
pub enum HelloWindowMsg {
    #[default]
    Idle,
}

/// HELLO 窗口组件：AV 文本经现有 View→Iced 投影渲染（零旧链依赖）。
#[derive(Debug, Default)]
pub struct AceHelloWindow;

impl Component for AceHelloWindow {
    type Msg = HelloWindowMsg;

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        let Some(session) = hello_session() else {
            return View::col()
                .padding(30)
                .child(View::text_styled("ace-hello: no session", "text-lg"))
                .build();
        };
        session.frames.fetch_add(1, Ordering::Relaxed);
        View::col()
            .padding(30)
            .child(View::text_styled(
                &session.evidence.av_text,
                "text-3xl font-bold text-center",
            ))
            .build()
    }
}
