//! PLAN-1023 ASE M0 交互会话原生验证入口：同一真实宿主窗口内运行两个存活
//! 应用会话（打开 app、tab 切换、frame 按钮真实 typed 事件、ASE 级关闭、
//! 重开新代次）。应用源经**显式路径**消费（权威 fixtures 在 auto-lang
//! `experimental/ase-m0/fixtures/counter-{a,b}/`）；会话/引擎/显示车道全部
//! 复用 980/981 公共面。会话脚本（tick:动作）与窗口按钮同一边界；证据
//! JSON 收口后自动退出（窗口单例纪律）。
//!
//! 用法：ase_m0_sessions_window.exe --app counter-a=<path> --app counter-b=<path>
//!   --evidence <out.json> [--hold-ms 8000] [--min-frames 8]
//!   [--script "3:open:counter-a,5:click:2,11:open:counter-b,25:select:1,31:close:1"]
//!   （脚本语法 tick:动作[:arg]；动作 open:<app>/select:<sid>/close:<sid>/
//!   click:<target>。click 的 target=按钮声明序：0=Dec、1=Reset、2=Inc。）
//!   Checked package actions use press:<message>:<occurrence> or
//!   type:<message>:<occurrence>:<text>, with zero-based occurrences.
//!   --prep-only：预构建 ui980 provider 后即退出（排窗前准备，不开窗）。
//!   --todo-project <013 root> --todo-code-root <directory> [--todo-data <directory>]

use auto_ui_runtime::ui::ase_m0_sessions::{
    install_ase_sessions_host, with_ase_sessions_host, AseApp, AseM0SessionsWindow,
    AseSessionAction, AseSessionHost, ASE_M0_SESSIONS_FRAMES,
};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn main() {
    auto_lang_core::set_ui_host_install_hook(auto_ui_runtime::install_ui_host_capabilities);
    auto_ui_runtime::install_ui_host_capabilities();

    let mut apps: Vec<AseApp> = Vec::new();
    let mut evidence_path: Option<String> = None;
    let mut hold_ms: u64 = 8000;
    let mut min_frames: usize = 8;
    let mut script_spec: Option<String> = None;
    let mut clock_app = None;
    let mut await_window_close = false;
    let mut todo_project = None;
    let mut todo_code_root = None;
    let mut todo_data = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--app" => {
                let Some(spec) = args.next() else {
                    eprintln!("ase_m0_sessions_window: --app needs <key>=<path>");
                    std::process::exit(2);
                };
                let Some((key, path)) = spec.split_once('=') else {
                    eprintln!("ase_m0_sessions_window: bad --app item {spec} (want <key>=<path>)");
                    std::process::exit(2);
                };
                if !key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    eprintln!("ase_m0_sessions_window: app key must be [A-Za-z0-9_-]: {key}");
                    std::process::exit(2);
                }
                apps.push(AseApp {
                    key: key.to_string(),
                    path: path.to_string(),
                });
            }
            "--clock-app" => {
                clock_app = Some(
                    args.next()
                        .expect("--clock-app requires a registered app key"),
                );
            }
            "--await-window-close" => await_window_close = true,
            "--todo-project" => todo_project = args.next().map(std::path::PathBuf::from),
            "--todo-code-root" => todo_code_root = args.next().map(std::path::PathBuf::from),
            "--todo-data" => todo_data = args.next().map(std::path::PathBuf::from),
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
                eprintln!("ase_m0_sessions_window: unknown arg `{other}`");
                std::process::exit(2);
            }
        }
    }
    if apps.is_empty() && todo_project.is_none() {
        eprintln!("ase_m0_sessions_window: at least one --app <key>=<path> is required");
        std::process::exit(2);
    }
    let evidence_path = evidence_path.expect("--evidence <out.json> is required");
    await_window_close |= (clock_app.is_some() || todo_project.is_some()) && script_spec.is_none();
    let mut host = AseSessionHost::new(apps);
    if let Some(project) = todo_project {
        let code = todo_code_root.expect("--todo-code-root is required with --todo-project");
        host = host.with_todo_project(&project, code, todo_data.as_deref())
            .unwrap_or_else(|error| {
                eprintln!("ase_m0_sessions_window: {error}");
                std::process::exit(2);
            });
    } else if todo_code_root.is_some() || todo_data.is_some() {
        eprintln!("ase_m0_sessions_window: --todo-project is required for Todo options");
        std::process::exit(2);
    }
    if let Some(key) = clock_app {
        host = match host.with_clock_app(key) {
            Ok(host) => host,
            Err(error) => {
                eprintln!("ase_m0_sessions_window: {error}");
                std::process::exit(2);
            }
        };
    }
    if let Some(spec) = &script_spec {
        host.install_script(parse_script(spec));
    }
    install_ase_sessions_host(host);

    if !await_window_close {
        let observer_path = evidence_path.clone();
        // 收口线程（只观察不驱动——会话/桥步进只在窗口 tick 主线程）：脚本
        // 注完 + 帧数达标或 hold 超时 → 证据退出。
        std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_millis(hold_ms.max(1000));
            loop {
                let settled = with_ase_sessions_host(|h| h.script_done()).unwrap_or(false);
                let frames = ASE_M0_SESSIONS_FRAMES.load(Ordering::Relaxed);
                if (settled && frames >= min_frames) || Instant::now() > deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
            let mut json = with_ase_sessions_host(|h| h.evidence())
                .unwrap_or_else(|| serde_json::json!({"error": "ase sessions host gone"}));
            json["frames_rendered"] =
                serde_json::json!(ASE_M0_SESSIONS_FRAMES.load(Ordering::Relaxed));
            let _ = std::fs::write(&observer_path, serde_json::to_string_pretty(&json).unwrap());
            std::process::exit(0);
        });
    }

    if let Err(e) = auto_ui_runtime::ui::iced::run_app_with_title::<AseM0SessionsWindow>(Some(
        "ASE M0 交互会话",
    )) {
        eprintln!("ase_m0_sessions_window: window run failed: {e:?}");
        std::process::exit(7);
    }

    let outcome = with_ase_sessions_host(|h| {
        let before = h.evidence();
        let cleanup = h.close_all();
        (before, h.evidence(), cleanup)
    });
    let (before, mut json, cleanup) = outcome.expect("ASE host available at window shutdown");
    json["before_window_close"] = before;
    json["frames_rendered"] = serde_json::json!(ASE_M0_SESSIONS_FRAMES.load(Ordering::Relaxed));
    json["cleanup_error"] = serde_json::json!(cleanup.as_ref().err());
    std::fs::write(&evidence_path, serde_json::to_string_pretty(&json).unwrap())
        .expect("write ASE window evidence");
    if cleanup.is_err() {
        std::process::exit(8);
    }
}

fn parse_script(spec: &str) -> Vec<(usize, AseSessionAction)> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut fields = part.splitn(5, ':');
        let Some(tick) = fields.next().and_then(|t| t.parse::<usize>().ok()) else {
            eprintln!("ase_m0_sessions_window: bad script entry `{part}`");
            std::process::exit(2);
        };
        let action = fields.next().unwrap_or("");
        let step = match action {
            "open" => {
                let Some(app) = fields.next() else {
                    eprintln!("ase_m0_sessions_window: open needs an app key: {part}");
                    std::process::exit(2);
                };
                AseSessionAction::Open(app.to_string())
            }
            "select" | "close" => {
                let Some(sid) = fields.next().and_then(|v| v.parse::<usize>().ok()) else {
                    eprintln!("ase_m0_sessions_window: {action} needs a session id: {part}");
                    std::process::exit(2);
                };
                if action == "select" {
                    AseSessionAction::Select(sid)
                } else {
                    AseSessionAction::Close(sid)
                }
            }
            "click" => {
                let Some(target) = fields.next().and_then(|v| v.parse::<usize>().ok()) else {
                    eprintln!("ase_m0_sessions_window: click needs a button target: {part}");
                    std::process::exit(2);
                };
                AseSessionAction::Click(target)
            }
            "press" | "type" => {
                let message = fields.next().expect("package action needs a handler name");
                let occurrence = fields.next().and_then(|value| value.parse().ok())
                    .expect("package action needs a zero-based occurrence");
                let text = (action == "type")
                    .then(|| fields.next().expect("type action needs text").to_owned());
                AseSessionAction::PackageInput { message: message.into(), occurrence, text }
            }
            other => {
                eprintln!("ase_m0_sessions_window: unknown script action `{other}`");
                std::process::exit(2);
            }
        };
        out.push((tick, step));
    }
    out
}
