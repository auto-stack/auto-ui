//! PLAN-980 MIN-A-UI 桌面窗口 exe：Node 源 → AV→ui980 FFI 实际 UI 更新 →
//! Iced 真窗；typed 事件脚本（0→1→0→0 / 两实例 2:1 / close-reopen-late）经
//! tick 受控点回 AV 重放；证据 JSON 收口后自动退出（窗口单例纪律）。
//!
//! 用法：ace_node_desktop_window.exe --source <app.at> --evidence <out.json>
//!   [--instances 2] [--hold-ms 8000] [--min-frames 4]
//!   [--script "3:click:0:2,5:click:0:2,7:click:1:2,9:close,11:late,13:reopen"]
//!   （脚本语法 tick:动作[:实例:目标]；动作 click/close/late/reopen。）

use auto_ui_runtime::ui::ace_node_desktop::{
    install_bridge, AceNodeDesktopWindow, NodeDesktopBridge, ScriptStep,
};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn main() {
    auto_lang_core::set_ui_host_install_hook(auto_ui_runtime::install_ui_host_capabilities);
    auto_ui_runtime::install_ui_host_capabilities();

    let mut source: Option<String> = None;
    let mut evidence_path: Option<String> = None;
    let mut instances: usize = 2;
    let mut hold_ms: u64 = 8000;
    let mut min_frames: usize = 4;
    let mut script_spec: Option<String> = None;
    // PLAN-980 ④-A #2b：AGE 包闭包输入（多单元 + 入口）——与 --source 二选一。
    let mut package_spec: Option<String> = None;
    let mut package_entry: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--source" => source = args.next(),
            "--evidence" => evidence_path = args.next(),
            "--instances" => instances = args.next().and_then(|v| v.parse().ok()).unwrap_or(2),
            "--hold-ms" => hold_ms = args.next().and_then(|v| v.parse().ok()).unwrap_or(8000),
            "--min-frames" => min_frames = args.next().and_then(|v| v.parse().ok()).unwrap_or(4),
            "--script" => script_spec = args.next(),
            "--package" => package_spec = args.next(),
            "--entry" => package_entry = args.next(),
            "--prep-only" => {
                // 预构建档：注册 provider（构建/命中缓存）后即退出——排窗前的
                // 准备阶段，不真开窗。
                auto_ui_runtime::ui::ace_node_desktop::ensure_ui980_provider()
                    .expect("ui980 provider prep");
                return;
            }
            other => {
                eprintln!("ace_node_desktop_window: unknown arg `{other}`");
                std::process::exit(2);
            }
        }
    }
    let evidence_path = evidence_path.expect("--evidence <out.json> is required");
    let mut bridge = if let Some(spec) = &package_spec {
        // --package "app::main=<path>; util::steps=<path>" --entry app::main
        let entry = package_entry.expect("--entry <pkg::module> is required with --package");
        let mut units = Vec::new();
        for item in spec.split(';') {
            let Some((key, path)) = item.trim().split_once('=') else {
                eprintln!("ace_node_desktop_window: bad --package item {item}");
                std::process::exit(2);
            };
            let Some((pkg, id)) = key.trim().split_once("::") else {
                eprintln!("ace_node_desktop_window: package unit key must be pkg::module: {key}");
                std::process::exit(2);
            };
            let src = match std::fs::read_to_string(path.trim()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ace_node_desktop_window: read unit {key} failed: {e}");
                    std::process::exit(3);
                }
            };
            units.push(ace_core::source::packages::PackageModuleInput {
                package: pkg.trim().to_string(),
                version: "1.0.0".to_string(),
                id: id.trim().to_string(),
                source: src,
                provider: None,
            });
        }
        match NodeDesktopBridge::load_package(units, &entry, instances) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("ace_node_desktop_window: package load failed: {e}");
                std::process::exit(4);
            }
        }
    } else {
        let source_path = source.expect("--source <app.at> is required (or --package)");
        let src = match std::fs::read_to_string(&source_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ace_node_desktop_window: read source failed: {e}");
                std::process::exit(3);
            }
        };
        // PLAN-105 T-02：真实工程源（带 `use`/依赖样式/类型化列表数据字段）先过
        // 组件面源准备（工程根=最近的 `pac.at` 祖先）；无 `pac.at` 的源原样装载
        // （104 的简单源行为不变）。
        let prepared = std::path::Path::new(&source_path)
            .ancestors()
            .find(|p| p.join("pac.at").is_file())
            .map(|root| {
                auto_ui_runtime::ui::ace_source_prep::prepare_component_source(root, &src)
                    .map(|p| p.source)
                    .unwrap_or_else(|_| src.clone())
            })
            .unwrap_or_else(|| src.clone());
        match NodeDesktopBridge::load(&prepared, &source_path, instances) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("ace_node_desktop_window: load failed: {e}");
                std::process::exit(4);
            }
        }
    };
    if let Some(spec) = &script_spec {
        bridge.install_script(parse_script(spec));
    }
    let expected_dispatches = count_clicks(&script_spec);
    if let Err(e) = bridge.open() {
        eprintln!("ace_node_desktop_window: open failed: {e}");
        std::process::exit(5);
    }
    install_bridge(bridge);

    // 收口线程（只观察不驱动——桥步进只在窗口 tick 主线程）：脚本注完 +
    // dispatch 齐或 hold 超时 + 帧数达标 → 证据退出。
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_millis(hold_ms.max(1000));
        loop {
            let settled = auto_ui_runtime::ui::ace_node_desktop::with_bridge(|b| {
                b.script_done() && b.dispatch_log.len() >= expected_dispatches
            })
            .unwrap_or(false);
            let frames =
                auto_ui_runtime::ui::ace_node_desktop::NODE_DESKTOP_FRAMES.load(Ordering::Relaxed);
            if (settled && frames >= min_frames) || Instant::now() > deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        let json = auto_ui_runtime::ui::ace_node_desktop::with_bridge(|b| b.evidence())
            .unwrap_or_else(|| serde_json::json!({"error": "bridge gone"}));
        let _ = std::fs::write(&evidence_path, serde_json::to_string_pretty(&json).unwrap());
        std::process::exit(0);
    });

    if let Err(e) = auto_ui_runtime::ui::iced::run_app::<AceNodeDesktopWindow>() {
        eprintln!("ace_node_desktop_window: window run failed: {e:?}");
        std::process::exit(7);
    }
}

fn count_clicks(spec: &Option<String>) -> usize {
    spec.as_ref()
        .map(|s| {
            s.split(',')
                .filter(|p| matches!(p.split(':').nth(1), Some("click") | Some("type")))
                .count()
        })
        .unwrap_or(0)
}

fn parse_script(spec: &str) -> Vec<(usize, ScriptStep)> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut fields = part.split(':');
        let Some(tick) = fields.next().and_then(|t| t.parse::<usize>().ok()) else {
            eprintln!("ace_node_desktop_window: bad script entry `{part}`");
            std::process::exit(2);
        };
        let action = fields.next().unwrap_or("");
        let step = match action {
            "click" | "pageclick" => {
                let instance = fields.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let target = fields.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                if action == "pageclick" {
                    ScriptStep::ClickPage { instance, target }
                } else {
                    ScriptStep::Click { instance, target }
                }
            }
            // B-AV U01：键入事件——tick:type:实例:目标:文本（文本=余段，
            // 不含逗号；中文可用）。
            "type" => {
                let instance = fields.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let target = fields.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let text = fields.collect::<Vec<_>>().join(":");
                ScriptStep::TypeText {
                    instance,
                    target,
                    text,
                }
            }
            "close" => ScriptStep::Close,
            "reopen" => ScriptStep::Reopen,
            "late" => ScriptStep::LateClick,
            other => {
                eprintln!("ace_node_desktop_window: unknown script action `{other}`");
                std::process::exit(2);
            }
        };
        out.push((tick, step));
    }
    out
}
