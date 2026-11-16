//! PLAN-980 NATIVE：ACE 原生窗口 exe——001 形真源 → AV 真值（texts+states）
//! → `run_app` 宿主渲染；状态写（--set-state name=bool）在指定帧后应用，
//! 证据 JSON 记录前后状态/主题翻转/帧数。窗口单例纪律：单实例、串行。
//!
//! 用法：ace_native_window.exe --source <app.at> --evidence <out.json>
//!   [--hold-ms 4000] [--min-frames 2] [--set-state dark_mode=true]
//!   [--set-at-frame 2]

use auto_ui_runtime::ui::ace_native::{
    install_native_session, native_session_mut, with_native_session, AceNativeSession,
    AceNativeWindow,
};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn main() {
    auto_lang_core::set_ui_host_install_hook(auto_ui_runtime::install_ui_host_capabilities);
    auto_ui_runtime::install_ui_host_capabilities();

    let mut source: Option<String> = None;
    let mut evidence_path: Option<String> = None;
    let mut hold_ms: u64 = 4000;
    let mut min_frames: usize = 2;
    let mut set_state: Option<(String, bool)> = None;
    let mut set_at_frame: usize = 2;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--source" => source = args.next(),
            "--evidence" => evidence_path = args.next(),
            "--hold-ms" => hold_ms = args.next().and_then(|v| v.parse().ok()).unwrap_or(4000),
            "--min-frames" => min_frames = args.next().and_then(|v| v.parse().ok()).unwrap_or(2),
            "--set-at-frame" => {
                set_at_frame = args.next().and_then(|v| v.parse().ok()).unwrap_or(2)
            }
            "--set-state" => {
                if let Some(spec) = args.next() {
                    if let Some((name, val)) = spec.split_once('=') {
                        set_state = Some((name.to_string(), val == "true"));
                    }
                }
            }
            other => {
                eprintln!("ace_native_window: unknown arg `{other}`");
                std::process::exit(2);
            }
        }
    }
    let source_path = source.expect("--source <app.at> is required");
    let evidence_path = evidence_path.expect("--evidence <out.json> is required");
    let src = match std::fs::read_to_string(&source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ace_native_window: read source failed: {e}");
            std::process::exit(3);
        }
    };
    let mut session = match AceNativeSession::load(&src, &source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ace_native_window: ACE native load failed: {e}");
            std::process::exit(4);
        }
    };
    let states_before = session.snapshot.states.clone();
    let texts = session
        .snapshot
        .texts
        .iter()
        .map(|t| {
            serde_json::json!({
                "value": t.value,
                "style": t.style,
                "selectable": t.selectable,
            })
        })
        .collect::<Vec<_>>();
    let frames = session.frames.clone();
    let node_name = session.snapshot.node_name.clone();
    let legacy = session.snapshot.legacy_widget;
    install_native_session(session);

    // 状态写 + 自动退出线程：set-at-frame 达标时写状态（重编译重观察+主题
    // 翻转），min-frames/hold 收口写证据退出。
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_millis(hold_ms.max(300));
        let mut applied: Option<Result<(), String>> = None;
        loop {
            let f = frames.load(Ordering::Relaxed);
            if applied.is_none() && set_state.is_some() && f >= set_at_frame {
                applied = Some(
                    native_session_mut(|s| {
                        s.write_state_bool(
                            &set_state.as_ref().unwrap().0,
                            set_state.as_ref().unwrap().1,
                        )
                    })
                    .unwrap_or_else(|| Err("session gone".to_string())),
                );
            }
            if f >= min_frames && (applied.is_some() || set_state.is_none()) {
                break;
            }
            if Instant::now() > deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        let (states_after, dark_after) = with_native_session(|s| {
            (
                s.snapshot.states.clone(),
                auto_ui_runtime::ui::style_iced_adapter::dark_mode(),
            )
        })
        .unwrap_or_default();
        let json = serde_json::json!({
            "source_path": source_path,
            "node_name": node_name,
            "legacy_widget": legacy,
            "texts": texts,
            "states_before": states_before_map(&states_before),
            "states_after": states_after
                .iter()
                .map(|(n, v)| {
                    (n.clone(), serde_json::Value::String(format!("{v:?}")))
                })
                .collect::<serde_json::Map<String, serde_json::Value>>(),
            "dark_mode_after": dark_after,
            "set_state_applied": applied.is_some(),
            "set_state_ok": applied.as_ref().map(|r| r.is_ok()).unwrap_or(false),
            "frames_rendered": frames.load(Ordering::Relaxed),
            "window_result": "closed",
        });
        let _ = std::fs::write(&evidence_path, serde_json::to_string_pretty(&json).unwrap());
        std::process::exit(0);
    });

    if let Err(e) = auto_ui_runtime::ui::iced::run_app::<AceNativeWindow>() {
        eprintln!("ace_native_window: window run failed: {e:?}");
        std::process::exit(7);
    }
}

fn states_before_map(
    states: &[(String, auto_ui_runtime::ui::ace_state_bridge::AceStateValue)],
) -> serde_json::Map<String, serde_json::Value> {
    states
        .iter()
        .map(|(n, v)| (n.clone(), serde_json::Value::String(format!("{v:?}"))))
        .collect()
}
