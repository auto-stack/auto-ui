//! PLAN-1118 AutoScape 本地浏览器七区外壳原生验证入口：单真实宿主窗口
//! （标题 AutoScape）承载七区 + guest AutoFrame。应用身份走冻结登记表
//! （`app://auto.demo.counter/` / `app://auto.demo.clock/` /
//! `app://auto.gallery/`）；收藏/配置落宿主用户配置目录（`--config-home`
//! 或 env `AUTOSCAPE_HOME` 可隔离）。外壳脚本（tick:动作）与窗口按钮/
//! 地址栏/菜单同一 on 消息边界；证据 JSON 收口后自动退出（窗口单例纪律）。
//!
//! 用法：autoscape_window.exe --evidence <out.json>
//!   [--config-home <dir>] [--fixture-root <dir>]
//!   [--hold-ms 8000] [--min-frames 8] [--await-window-close]
//!   [--script "2:open:app://auto.demo.counter/,5:click:2,9:open:Clock,13:back,
//!              17:forward,21:favadd,25:stop"]
//!   （脚本语法 tick:动作[:arg]；动作 open:<地址或应用名>/back/forward/
//!   refresh/stop/newtab/selecttab:<页签序>/closetab:<页签序>/click:<按钮序>
//!   /favadd/favdel:<收藏序>/bar/panel/hoverin:<地址>/hoverout。
//!   click 的 target=guest 声明序（Counter：0=-、1=Reset、2=+）。）
//!   窄窗基线：env `AUTO_VM_WINDOW=520x700`（常用档为默认 1280x800）。
//!   --prep-only：预构建 ui980 provider 后即退出（排窗前准备，不开窗）。

use auto_ui_runtime::ui::ase_browser_shell::{
    install_shell_host, with_shell_host, AutoscapeHost, AutoscapeWindow, ShellAction,
    AUTOSCAPE_FRAMES,
};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn main() {
    auto_lang_core::set_ui_host_install_hook(auto_ui_runtime::install_ui_host_capabilities);
    auto_ui_runtime::install_ui_host_capabilities();

    let mut evidence_path: Option<String> = None;
    let mut config_home: Option<PathBuf> = None;
    let mut fixture_root: Option<PathBuf> = None;
    let mut autoweb_root_ca: Option<Vec<u8>> = None;
    let mut hold_ms: u64 = 8000;
    let mut min_frames: usize = 8;
    let mut script_spec: Option<String> = None;
    let mut await_window_close = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--config-home" => config_home = args.next().map(PathBuf::from),
            "--fixture-root" => fixture_root = args.next().map(PathBuf::from),
            // PLAN-1119：网络导航附加信任根（真窗档 e2e 注入测试 CA；
            // 信任仅作用于 AutoWeb 导航客户端，不碰 OS 信任库）。
            "--autoweb-root-ca" => {
                let pem_path = args
                    .next()
                    .map(PathBuf::from)
                    .expect("--autoweb-root-ca <pem>");
                autoweb_root_ca = Some(std::fs::read(&pem_path).unwrap_or_else(|e| {
                    eprintln!("autoscape_window: read {}: {e}", pem_path.display());
                    std::process::exit(2);
                }));
            }
            "--evidence" => evidence_path = args.next(),
            "--hold-ms" => hold_ms = args.next().and_then(|v| v.parse().ok()).unwrap_or(8000),
            "--min-frames" => min_frames = args.next().and_then(|v| v.parse().ok()).unwrap_or(8),
            "--script" => script_spec = args.next(),
            "--await-window-close" => await_window_close = true,
            "--prep-only" => {
                auto_ui_runtime::ui::ace_node_desktop::ensure_ui980_provider()
                    .expect("ui980 provider prep");
                return;
            }
            other => {
                eprintln!("autoscape_window: unknown arg `{other}`");
                std::process::exit(2);
            }
        }
    }
    let evidence_path = evidence_path.expect("--evidence <out.json> is required");

    let mut host = if let Some(fixture_root) = fixture_root {
        let config_root = config_home.unwrap_or_else(|| {
            std::env::temp_dir().join(format!("autoscape-window-{}", std::process::id()))
        });
        AutoscapeHost::with_roots(fixture_root, config_root)
    } else {
        match AutoscapeHost::new() {
            Ok(mut host) => {
                if let Some(config_home) = config_home {
                    // 指定配置根：以真实 fixture 根重建（保持单一构造路径）。
                    let fixture_root = host.fixture_root().to_path_buf();
                    host = AutoscapeHost::with_roots(fixture_root, config_home);
                }
                host
            }
            Err(error) => {
                eprintln!("autoscape_window: {error}");
                std::process::exit(2);
            }
        }
    };
    if let Some(pem) = autoweb_root_ca {
        let mut options = auto_ui_runtime::ui::autoweb_nav::AutowebNavOptions::default();
        options.extra_root_pem = Some(pem);
        host = host.with_autoweb_options(options);
    }
    if let Some(spec) = &script_spec {
        host.install_script(parse_script(spec));
    }
    install_shell_host(host);

    if !await_window_close {
        let observer_path = evidence_path.clone();
        // 收口线程（只观察不驱动——导航提交/会话步进只在窗口 tick 主线程）：
        // 脚本注完 + 帧数达标或 hold 超时 → 证据退出。
        std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_millis(hold_ms.max(1000));
            loop {
                let settled = with_shell_host(|h| h.script_done()).unwrap_or(false);
                let frames = AUTOSCAPE_FRAMES.load(Ordering::Relaxed);
                if (settled && frames >= min_frames) || Instant::now() > deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
            let mut json = with_shell_host(|h| h.evidence())
                .unwrap_or_else(|| serde_json::json!({"error": "autoscape host gone"}));
            json["frames_rendered"] = serde_json::json!(AUTOSCAPE_FRAMES.load(Ordering::Relaxed));
            let _ = std::fs::write(&observer_path, serde_json::to_string_pretty(&json).unwrap());
            std::process::exit(0);
        });
    }

    if let Err(e) =
        auto_ui_runtime::ui::iced::run_app_with_title::<AutoscapeWindow>(Some("AutoScape"))
    {
        eprintln!("autoscape_window: window run failed: {e:?}");
        std::process::exit(7);
    }

    let outcome = with_shell_host(|h| {
        let before = h.evidence();
        let cleanup = h.close_all();
        (before, h.evidence(), cleanup)
    });
    let (before, mut json, cleanup) = outcome.expect("AutoScape host available at window shutdown");
    json["before_window_close"] = before;
    json["frames_rendered"] = serde_json::json!(AUTOSCAPE_FRAMES.load(Ordering::Relaxed));
    json["cleanup_error"] = serde_json::json!(cleanup.as_ref().err());
    std::fs::write(&evidence_path, serde_json::to_string_pretty(&json).unwrap())
        .expect("write AutoScape window evidence");
    if cleanup.is_err() {
        std::process::exit(8);
    }
}

fn parse_script(spec: &str) -> Vec<(usize, ShellAction)> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut fields = part.split(':');
        let Some(tick) = fields.next().and_then(|t| t.parse::<usize>().ok()) else {
            eprintln!("autoscape_window: bad script entry `{part}`");
            std::process::exit(2);
        };
        let action = fields.next().unwrap_or("");
        let rest: Vec<&str> = fields.collect();
        let step = match action {
            "open" => {
                if rest.is_empty() {
                    eprintln!("autoscape_window: open needs an address: {part}");
                    std::process::exit(2);
                }
                // 地址可含 ":"（app://…）——按原样重组。
                ShellAction::Navigate(rest.join(":"))
            }
            "back" => ShellAction::Back,
            "forward" => ShellAction::Forward,
            "refresh" => ShellAction::Refresh,
            "stop" => ShellAction::Stop,
            "newtab" => ShellAction::NewTab,
            "selecttab" | "closetab" | "click" | "favdel" => {
                let Some(value) = rest.first().and_then(|v| v.parse::<usize>().ok()) else {
                    eprintln!("autoscape_window: {action} needs a number: {part}");
                    std::process::exit(2);
                };
                match action {
                    "selecttab" => ShellAction::SelectTab(value),
                    "closetab" => ShellAction::CloseTab(value),
                    "click" => ShellAction::Click(value),
                    _ => ShellAction::FavoriteDelete(value),
                }
            }
            "favadd" => ShellAction::FavoriteAdd,
            "bar" => ShellAction::ToggleBar,
            "panel" => ShellAction::TogglePanel,
            "hoverin" => {
                if rest.is_empty() {
                    eprintln!("autoscape_window: hoverin needs an address: {part}");
                    std::process::exit(2);
                }
                ShellAction::HoverIn(rest.join(":"))
            }
            "hoverout" => ShellAction::HoverOut,
            other => {
                eprintln!("autoscape_window: unknown script action `{other}`");
                std::process::exit(2);
            }
        };
        out.push((tick, step));
    }
    out
}
