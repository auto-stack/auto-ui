//! PLAN-026 T-00 候选B 最小启动证据（probe-only，非生产代码）。
//!
//! 证明：本机 WebView2 运行时可被 wry 拉起；页面内 JS 可经 IPC 把真实
//! 消息回传 Rust 进程；进程在有限时间内干净退出（退出码 0 = 全链通）。
//! 不加载真实引擎/语料——那是 T-03 的任务；本文件只闭合 T-00 的
//! “依赖未知项先闭合”。

use std::sync::mpsc;

use tao::event_loop::{ControlFlow, EventLoopBuilder};
use wry::WebViewBuilder;

const PROBE_HTML: &str = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>plan-026-t00-probe</title></head>
<body style="margin:0;background:#102030;color:#dfe">
  <div id="surface" style="font:16px sans-serif;padding:24px">
    PLAN-026 candidate-B surface alive
  </div>
  <script>
    // 模拟未来引擎 adapter 的首批回传：一条带 schema 的 JSON IPC。
    if (window.ipc !== undefined) {
      window.ipc.postMessage(JSON.stringify({
        schema_version: 1,
        probe: "t00",
        engine_revision: "none",
        surface_alive: true
      }));
    }
  </script>
</body></html>"#;

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    let (tx, rx) = mpsc::channel::<String>();

    let window = tao::window::WindowBuilder::new()
        .with_title("plan-026-t00-probe")
        .with_inner_size(tao::dpi::LogicalSize::new(640.0, 360.0))
        .build(&event_loop)
        .expect("window");

    let webview = WebViewBuilder::new()
        .with_html(PROBE_HTML)
        .with_ipc_handler(move |req| {
            let _ = tx.send(req.body().to_string());
        })
        .with_initialization_script(
            "if (window.ipc) { window.ipc.postMessage(JSON.stringify({schema_version:1,probe:'t00',surface_alive:true})); }",
        )
        .build(&window)
        .expect("webview");

    let mut received: Option<String> = None;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(deadline);
        if let Some(msg) = rx.try_recv().ok() {
            println!("IPC_RECEIVED {}", msg);
            received = Some(msg);
        }
        if let tao::event::Event::WindowEvent { event: tao::event::WindowEvent::CloseRequested, .. } = event {
            *control_flow = ControlFlow::Exit;
        }
        if std::time::Instant::now() >= deadline {
            match &received {
                Some(m) if m.contains("\"surface_alive\":true") => {
                    println!("PROBE_PASS webview2 ipc round-trip ok");
                    std::process::exit(0);
                }
                other => {
                    eprintln!("PROBE_FAIL no valid ipc message: {:?}", other);
                    std::process::exit(1);
                }
            }
        }
        let _ = &webview;
    });
}
