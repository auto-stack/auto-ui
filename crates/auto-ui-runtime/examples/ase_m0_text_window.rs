//! PLAN-1022 ASE M0 文字浏览器原生验证入口：同一真实宿主窗口内轮换独立
//! 普通 Node 页面（打开A/B、重新打开、关闭、错误/当前地址）。页面源经
//! **显式路径**消费（权威 fixtures 在 auto-lang `experimental/ase-m0/
//! fixtures/text-{a,b}/`，不复制权威）；显示/关闭复用 980 MIN-A-UI 公共
//! ui980 车道（AV 真值 → 真实 UI-FFI → frame）。导航脚本（tick:动作）与
//! 窗口按钮同一边界；证据 JSON 收口后自动退出（窗口单例纪律）。
//!
//! 用法：ase_m0_text_window.exe --page a=<path> --page b=<path>
//!   --evidence <out.json> [--hold-ms 8000] [--min-frames 8]
//!   [--script "3:open:a,9:close,13:open:b,19:reopen,25:close"]
//!   （脚本语法 tick:动作[:key]；动作 open/close/reopen。）
//!   --prep-only：预构建 ui980 provider 后即退出（排窗前准备，不开窗）。

use auto_ui_runtime::ui::ase_m0_frame::{
    install_ase_host, with_ase_host, AseFrameHost, AseM0TextWindow, AseNavAction, AsePage,
    ASE_M0_FRAMES,
};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn main() {
    auto_lang_core::set_ui_host_install_hook(auto_ui_runtime::install_ui_host_capabilities);
    auto_ui_runtime::install_ui_host_capabilities();

    let mut pages: Vec<AsePage> = Vec::new();
    let mut evidence_path: Option<String> = None;
    let mut hold_ms: u64 = 8000;
    let mut min_frames: usize = 8;
    let mut script_spec: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--page" => {
                let Some(spec) = args.next() else {
                    eprintln!("ase_m0_text_window: --page needs <key>=<path>");
                    std::process::exit(2);
                };
                let Some((key, path)) = spec.split_once('=') else {
                    eprintln!("ase_m0_text_window: bad --page item {spec} (want <key>=<path>)");
                    std::process::exit(2);
                };
                if !key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    eprintln!("ase_m0_text_window: page key must be [A-Za-z0-9_-]: {key}");
                    std::process::exit(2);
                }
                pages.push(AsePage {
                    key: key.to_string(),
                    path: path.to_string(),
                });
            }
            "--evidence" => evidence_path = args.next(),
            "--hold-ms" => hold_ms = args.next().and_then(|v| v.parse().ok()).unwrap_or(8000),
            "--min-frames" => min_frames = args.next().and_then(|v| v.parse().ok()).unwrap_or(8),
            "--script" => script_spec = args.next(),
            "--prep-only" => {
                auto_ui_runtime::ui::ace_node_desktop::ensure_ui980_provider()
                    .expect("ui980 provider prep");
                return;
            }
            other => {
                eprintln!("ase_m0_text_window: unknown arg `{other}`");
                std::process::exit(2);
            }
        }
    }
    if pages.is_empty() {
        eprintln!("ase_m0_text_window: at least one --page <key>=<path> is required");
        std::process::exit(2);
    }
    let evidence_path = evidence_path.expect("--evidence <out.json> is required");
    let mut host = AseFrameHost::new(pages);
    if let Some(spec) = &script_spec {
        host.install_script(parse_script(spec));
    }
    install_ase_host(host);

    // 收口线程（只观察不驱动——导航/桥步进只在窗口 tick 主线程）：脚本
    // 注完 + 帧数达标或 hold 超时 → 证据退出。
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_millis(hold_ms.max(1000));
        loop {
            let settled = with_ase_host(|h| h.script_done()).unwrap_or(false);
            let frames = ASE_M0_FRAMES.load(Ordering::Relaxed);
            if (settled && frames >= min_frames) || Instant::now() > deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        let json = with_ase_host(|h| h.evidence())
            .unwrap_or_else(|| serde_json::json!({"error": "ase host gone"}));
        let _ = std::fs::write(&evidence_path, serde_json::to_string_pretty(&json).unwrap());
        std::process::exit(0);
    });

    if let Err(e) =
        auto_ui_runtime::ui::iced::run_app_with_title::<AseM0TextWindow>(Some("ASE M0 文字浏览器"))
    {
        eprintln!("ase_m0_text_window: window run failed: {e:?}");
        std::process::exit(7);
    }
}

fn parse_script(spec: &str) -> Vec<(usize, AseNavAction)> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut fields = part.split(':');
        let Some(tick) = fields.next().and_then(|t| t.parse::<usize>().ok()) else {
            eprintln!("ase_m0_text_window: bad script entry `{part}`");
            std::process::exit(2);
        };
        let action = fields.next().unwrap_or("");
        let step = match action {
            "open" => {
                let Some(key) = fields.next() else {
                    eprintln!("ase_m0_text_window: open needs a page key: {part}");
                    std::process::exit(2);
                };
                AseNavAction::Open(key.to_string())
            }
            "close" => AseNavAction::Close,
            "reopen" => AseNavAction::Reopen,
            other => {
                eprintln!("ase_m0_text_window: unknown script action `{other}`");
                std::process::exit(2);
            }
        };
        out.push((tick, step));
    }
    out
}
