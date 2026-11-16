//! PLAN-980 HELLO：首个 ACE AGE Node 文字窗口（真实 Iced 主线程窗口）。
//!
//! 链路：fixture AGE node 源 → H0 共享 verify → AV 真值文本 → F0 受检调用
//! （967 冻结 provider，经 969 E0 门面）→ `run_app` 泛型宿主渲染（现有
//! View→Iced 投影/Inter 字体/主题服务）。
//!
//! 用法（测试/验收驱动，不交互）：
//!   ace_hello_window.exe --source <app.at> --evidence <out.json> \
//!                        [--hold-ms 1500] [--provider f0_hello]
//!
//! 窗口至少渲染 `--min-frames` 帧或 hold 超时后写证据并退出（Process.exit
//! 语义，同 examples/plan804_real_host.rs）。F0 provider 须已预构建（root
//! 命中新鲜度缓存——本进程零 rustc）。

use auto_ui_runtime::ui::ace_hello::{
    install_hello_session, stable_f0_root, AceHelloWindow, HelloSession,
};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn main() {
    auto_lang_core::set_ui_host_install_hook(auto_ui_runtime::install_ui_host_capabilities);
    auto_ui_runtime::install_ui_host_capabilities();

    let mut source: Option<String> = None;
    let mut evidence_path: Option<String> = None;
    let mut hold_ms: u64 = 1500;
    let mut min_frames: usize = 2;
    let mut provider: Option<String> = None;
    let mut prep_only = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--source" => source = args.next(),
            "--evidence" => evidence_path = args.next(),
            "--hold-ms" => hold_ms = args.next().and_then(|v| v.parse().ok()).unwrap_or(1500),
            "--min-frames" => min_frames = args.next().and_then(|v| v.parse().ok()).unwrap_or(2),
            "--provider" => provider = args.next(),
            "--prep-only" => prep_only = true,
            other => {
                eprintln!("ace_hello_window: unknown arg `{other}`");
                std::process::exit(2);
            }
        }
    }
    // 预构建模式：只建 F0 provider（稳定 root，缓存 stamp）即退出——显式
    // 准备阶段入口，窗口面与测试都不触发 rustc。
    if prep_only {
        let provider = provider
            .as_deref()
            .expect("--provider is required with --prep-only");
        auto_lang_ace_host::install().expect("ace host installs");
        auto_lang_ace_host::build_f0_text_provider(provider, &stable_f0_root())
            .expect("f0 provider prebuilt");
        println!("f0 provider prebuilt at {}", stable_f0_root().display());
        return;
    }

    let source_path = source.expect("--source <app.at> is required");
    let evidence_path = evidence_path.expect("--evidence <out.json> is required");
    let src = match std::fs::read_to_string(&source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ace_hello_window: read source failed: {e}");
            std::process::exit(3);
        }
    };

    let mut session = match HelloSession::load(&src, &source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ace_hello_window: AGE load failed: {e}");
            std::process::exit(4);
        }
    };
    if let Some(provider) = provider.as_deref() {
        if let Err(e) = session.run_f0(provider) {
            eprintln!("ace_hello_window: F0 dispatch failed: {e}");
            std::process::exit(5);
        }
    }

    let frames = session.frames.clone();
    let f0 = session.evidence.f0.clone();
    let ev = session.evidence.clone();
    if install_hello_session(session).is_err() {
        eprintln!("ace_hello_window: session already installed");
        std::process::exit(6);
    }

    // 自动退出线程：渲染达标或 hold 超时 → 写证据 → 退出（窗口随进程关闭）。
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_millis(hold_ms.max(300));
        while Instant::now() < deadline {
            if frames.load(Ordering::Relaxed) >= min_frames {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let rendered = frames.load(Ordering::Relaxed);
        let json = serde_json::json!({
            "source_path": ev.source_path,
            "node_name": ev.node_name,
            "head_role": ev.head_role,
            "grammar_id": ev.grammar_id,
            "av_text": ev.av_text,
            "av_text_bytes": ev.av_text.len(),
            "f0": f0.as_ref().map(|f| serde_json::json!({
                "provider": f.provider,
                "byte_len": f.byte_len,
                "callee_count": f.callee_count,
                "rejected_count": f.rejected_count,
            })),
            "frames_rendered": rendered,
            "f0_root": stable_f0_root().display().to_string(),
            "window_result": "closed",
        });
        let _ = std::fs::write(&evidence_path, serde_json::to_string_pretty(&json).unwrap());
        std::process::exit(0);
    });

    // 主线程：真实 Iced 窗口（run_app 泛型宿主；Default 组件经 OnceLock 会话）。
    if let Err(e) = auto_ui_runtime::ui::iced::run_app::<AceHelloWindow>() {
        eprintln!("ace_hello_window: window run failed: {e:?}");
        std::process::exit(7);
    }
}
