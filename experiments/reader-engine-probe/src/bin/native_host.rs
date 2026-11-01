//! PLAN-026 T-03 原生宿主实验（probe-only，非生产代码）。
//!
//! 证明链：iced 0.14 主窗口（本进程）→ `window::run(id, |win| …)` 拿到
//! `&dyn Window`（HasWindowHandle）→ wry `WebViewBuilder::new_as_child(win)`
//! 在 iced 客户区内挂 WebView2 子表面 → 表面内 JS 经 IPC 回传 → 收到即
//! PROBE_T03_PASS（退出码 0）。20s deadline 未回传 = FAIL（退出码 1）。
//! IPC 回执在 WebView2 线程直接打印并退出（AtomicBool 防误杀 deadline）。
//!
//! 后续 V-01 原生臂把 data: 探针页换成消费 app 的引擎资产面；本文件闭合
//! 「iced 进程内嵌 WebView2 子表面 + 桥回传」这一最小未知。

use std::sync::atomic::{AtomicBool, Ordering};

use iced::window;
use iced::{Element, Subscription, Task};
use wry::WebViewBuilder;
use wry::raw_window_handle::HasWindowHandle;

static DONE: AtomicBool = AtomicBool::new(false);

// engine 模式：装载消费 app 引擎资产面（--url 传入），IPC 仅作状态回报，
// 不自动退出（由驱动收尾 kill）；spike 模式（缺省）保留 20s 自动判定。
fn engine_mode() -> bool {
    std::env::var("PROBE_T03_MODE").as_deref() == Ok("engine")
}

const PROBE_HTML: &str = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>plan-026-t03</title></head>
<body style="margin:0;background:#101820;color:#dfe;font:15px sans-serif;padding:20px">
  <div id="mark">PLAN-026 native child surface alive (iced host)</div>
  <script>
    if (window.ipc) {
      window.ipc.postMessage(JSON.stringify({
        schema_version: 1, probe: "t03", host: "iced", surface_alive: true
      }));
    }
  </script>
</body></html>"#;

thread_local! {
    // WebView2/COM 句柄须留在创建线程；iced 的 window::run 闭包即主线程。
    static WEBVIEW: std::cell::RefCell<Option<wry::WebView>> = const { std::cell::RefCell::new(None) };
}

#[derive(Debug, Clone)]
enum Msg {
    WindowOpened(Option<window::Id>),
}

#[derive(Default)]
struct App {
    seam_done: bool,
}

fn update(app: &mut App, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::WindowOpened(opt) => {
            let Some(id) = opt else { return Task::none() };
            eprintln!("PROBE_T03_OPEN_EVENT id={id:?}");
            if app.seam_done {
                return Task::none();
            }
            app.seam_done = true;
            window::run(id, |win| {
                // &dyn Window 是未定尺寸类型——借具体 WindowHandle 包装。
                eprintln!("PROBE_T03_RUN_CLOSURE entered");
                let handle = win.window_handle().expect("iced window handle");
                let mut builder = WebViewBuilder::new();
                builder = if engine_mode() {
                    let url = std::env::var("PROBE_T03_URL")
                        .unwrap_or_else(|_| "http://127.0.0.1:17827/api/probe/asset/native-host.html?doc=MISSING&session=MISSING&hash=MISSING".to_string());
                    builder.with_url(&url)
                } else {
                    builder.with_html(PROBE_HTML)
                };
                let built = builder
                    .with_ipc_handler(|req| {
                        let body = req.body().to_string();
                        if engine_mode() {
                            // 引擎模式：状态回执直出（PASS/FAIL 判定由驱动做）。
                            println!("IPC_RECEIVED {body}");
                            if body.contains("t03-engine") {
                                DONE.store(true, Ordering::SeqCst);
                            }
                            return;
                        }
                        if body.contains("\"surface_alive\":true") {
                            DONE.store(true, Ordering::SeqCst);
                            println!("PROBE_T03_PASS iced-child-webview ipc round-trip ok :: {body}");
                            std::process::exit(0);
                        } else {
                            println!("IPC_RECEIVED {body}");
                        }
                    })
                    .with_initialization_script(
                        "if (window.ipc) { window.ipc.postMessage(JSON.stringify({schema_version:1,probe:'t03',host:'iced',surface_alive:true})); }",
                    )
                    .build_as_child(&handle);
                match built {
                    Ok(wv) => {
                        // 子表面默认 200x200（wry 缺省 bounds，2026-11-02 实测
                        // WRY_WEBVIEW@0,0 200x200）——按宿主客户区物理尺寸撑满。
                        if let wry::raw_window_handle::RawWindowHandle::Win32(h) =
                            handle.as_raw()
                        {
                            #[repr(C)]
                            struct RECT {
                                left: i32,
                                top: i32,
                                right: i32,
                                bottom: i32,
                            }
                            extern "system" {
                                fn GetClientRect(hwnd: isize, rect: *mut RECT) -> i32;
                            }
                            let hwnd = h.hwnd.get() as isize;
                            unsafe {
                                let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                                if GetClientRect(hwnd, &mut rect) != 0 {
                                    let w = (rect.right - rect.left).max(0) as u32;
                                    let hh = (rect.bottom - rect.top).max(0) as u32;
                                    let _ = wv.set_bounds(wry::Rect {
                                        position: wry::dpi::PhysicalPosition::new(0, 0).into(),
                                        size: wry::dpi::PhysicalSize::new(w, hh).into(),
                                    });
                                    println!("PROBE_T03_BOUNDS {w}x{hh}");
                                }
                            }
                        }
                        WEBVIEW.with(|c| *c.borrow_mut() = Some(wv));
                        println!("PROBE_T03_WEBVIEW_MOUNTED as child of iced window");
                    }
                    Err(e) => {
                        eprintln!("PROBE_T03_BUILD_FAIL {e:?}");
                        std::process::exit(2);
                    }
                }
            })
            .map(move |()| Msg::WindowOpened(Some(id)))
        }
    }
}

fn view(_app: &App) -> Element<'_, Msg> {
    iced::widget::column![
        iced::widget::text("PLAN-026 T-03: iced host + wry child seam"),
        iced::widget::text("WebView2 child surface fills the area below"),
    ]
    .height(iced::Length::Fill)
    .into()
}

fn subscription(_app: &App) -> Subscription<Msg> {
    window::open_events().map(|id| Msg::WindowOpened(Some(id)))
}

fn boot() -> (App, Task<Msg>) {
    (App::default(), window::latest().map(Msg::WindowOpened))
}

fn main() {
    // deadline 守护：20s 内未 PASS 则 FAIL 退出（1）。
    std::thread::spawn(|| {
        let budget = if engine_mode() { 120 } else { 20 };
        std::thread::sleep(std::time::Duration::from_secs(budget));
        if !DONE.load(Ordering::SeqCst) {
            eprintln!("PROBE_T03_FAIL deadline ({budget}s) without ipc round-trip");
            std::process::exit(1);
        }
    });
    let _ = iced::application(boot, update, view).run();
}
