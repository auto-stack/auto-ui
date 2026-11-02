//! PLAN-029 T-02：doc-surface 实验文档表面 leaf（probe-only，PLAN-029 合同 §2）。
//!
//! 证明链：真实 AutoUI 原生 VM 窗口内，**组件布局分配的区域**上挂 wry
//! WebView2 子表面（026 native_host 已证同缝：`build_as_child` + 客户区物理
//! bounds）。与 026 独立宿主的差异：表面由 .at 页 `doc-surface (url: ...)`
//! 声明，经 NativeWidgetRegistry Element 通道派发（`View::Custom`），iced
//! canvas painter 在 draw 期把组件 bounds 同步给子表面——表面处于组件树/
//! 布局/生命周期内，宿主 chrome（标题/按钮行/页边距）保持可见，不再盖满
//! 整个客户区。工具链零改动：窗口进程 = auto-ui-tool.exe（发现序第一键
//! `AUTO_UI_TOOL_BIN`，PLAN-816 T-04），本 crate 以 feature `doc-surface`
//! 编入该 exe。
//!
//! 纪律（计划 §5.3/SD-02 边界）：
//! - 不得把 remote-frame 的 DisplayList 语义偷换成任意 DOM——本 leaf 是
//!   独立注册名 `doc-surface`，不触碰 remote-frame 任何臂；
//! - 子表面是 HWND 子层，z 序高于 iced 内容：bounds 严格取组件布局 rect
//!   （逻辑 × 窗口 DPI → 物理）；url 空/draw 停止（滚出视口/隐藏）时同步
//!   `set_visible(false)`，不遮邻接 chrome；多窗重叠布局不在本实验承诺内
//!   （SD-02 记边界）；
//! - webview 的 COM 初始化须在创建线程（winit 主线程；026 native_host
//!   `window::run` 同缝实测成立，painter draw 即主线程渲染路径）；
//! - 宿主窗定位：painter draw 在 winit 主线程——EnumThreadWindows(当前
//!   线程) 可见顶层窗中，canvas 中心点（按候选窗 DPI 变换）落在其客户区
//!   的第一窗即宿主（探针 profile 单窗；多窗属 SD-02 边界外）。
//!
//! 诊断开关：`AUTO_DOC_SURFACE_DEBUG=1` 在 stderr 打每次同步（动作/bounds/
//! hwnd/url 长度），供 T-04/T-05 取证。

use std::collections::HashMap;
use std::num::NonZeroIsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use iced::widget::canvas::{self, Geometry, Path};
use iced::{mouse, Rectangle, Size};
use wry::raw_window_handle::{
    HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle,
};

// ---------------------------------------------------------------------------
// Win32 面（窄声明，同 026 native_host 纪律：只取本缝所需原型）。
// ---------------------------------------------------------------------------

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

extern "system" {
    fn EnumThreadWindows(dw_thread: u32, callback: isize, lparam: isize) -> i32;
    fn IsWindowVisible(hwnd: isize) -> i32;
    fn GetClientRect(hwnd: isize, rect: *mut RECT) -> i32;
    fn ClientToScreen(hwnd: isize, pt: *mut POINT) -> i32;
    fn GetDpiForWindow(hwnd: isize) -> u32;
    fn GetCurrentThreadId() -> u32;
}

fn dpi_scale(hwnd: isize) -> f32 {
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi == 0 { 1.0 } else { dpi as f32 / 96.0 }
}

thread_local! {
    /// 枚举中转：待测 canvas 中心（逻辑）。
    static CENTER: std::cell::Cell<Option<(f32, f32)>> = const { std::cell::Cell::new(None) };
    /// 枚举结果：命中宿主 hwnd。
    static HIT: std::cell::Cell<Option<isize>> = const { std::cell::Cell::new(None) };
}

extern "system" fn enum_host_callback(hwnd: isize, _lparam: isize) -> i32 {
    unsafe {
        let Some((cx, cy)) = CENTER.with(|c| c.get()) else { return 1 };
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }
        let mut client = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if GetClientRect(hwnd, &mut client) == 0 {
            return 1;
        }
        let mut origin = POINT { x: 0, y: 0 };
        if ClientToScreen(hwnd, &mut origin) == 0 {
            return 1;
        }
        let scale = dpi_scale(hwnd);
        let px = origin.x + (cx * scale) as i32;
        let py = origin.y + (cy * scale) as i32;
        if px >= client.left + origin.x
            && px < client.right + origin.x
            && py >= client.top + origin.y
            && py < client.bottom + origin.y
        {
            HIT.with(|c| c.set(Some(hwnd)));
            return 0; // 命中即停
        }
    }
    1
}

/// draw 期定位宿主窗：canvas 中心点按候选窗 DPI 变换后落在客户区。
fn locate_host(bounds: &Rectangle) -> Option<isize> {
    CENTER.with(|c| c.set(Some((bounds.center_x(), bounds.center_y()))));
    HIT.with(|c| c.set(None));
    unsafe { EnumThreadWindows(GetCurrentThreadId(), enum_host_callback as isize, 0) };
    HIT.with(|c| c.get())
}

/// canvas 逻辑 bounds → 宿主客户区坐标系的物理 rect（子 HWND 定位口径）。
fn canvas_physical_rect(hwnd: isize, bounds: &Rectangle) -> Option<(i32, i32, u32, u32)> {
    unsafe {
        let mut client = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if GetClientRect(hwnd, &mut client) == 0 {
            return None;
        }
        let scale = dpi_scale(hwnd);
        let x = (bounds.x * scale) as i32;
        let y = (bounds.y * scale) as i32;
        let w = (bounds.width * scale) as i32;
        let h = (bounds.height * scale) as i32;
        // 逻辑面可能超出客户区（负坐标/超界）——夹到客户区交集，全出界=None。
        let cl = x.max(0);
        let ct = y.max(0);
        let cr = (x + w).min(client.right - client.left);
        let cb = (y + h).min(client.bottom - client.top);
        if cr <= cl || cb <= ct {
            return None;
        }
        Some((cl, ct, (cr - cl) as u32, (cb - ct) as u32))
    }
}

// ---------------------------------------------------------------------------
// 表面注册表：key → Surface（url/hwnd/webview/物理 bounds/可见性）。
// ---------------------------------------------------------------------------

/// 表面元数据（Send 静态注册表面）。webview 本体是 COM 对象（非 Send），
/// 留在创建线程（winit 主线程）的 WEBVIEWS thread_local——sync/release 都
/// 只从 painter draw（主线程渲染路径）进入，两侧状态严格同线程配对。
#[derive(Default)]
struct SurfaceMeta {
    url: String,
    hwnd: isize,
    phys: (i32, i32, u32, u32),
    visible: bool,
}

static SURFACES: OnceLock<Mutex<HashMap<String, SurfaceMeta>>> = OnceLock::new();
static SYNC_SEQ: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static WEBVIEWS: std::cell::RefCell<HashMap<String, wry::WebView>> =
        std::cell::RefCell::new(HashMap::new());
}

fn surfaces() -> &'static Mutex<HashMap<String, SurfaceMeta>> {
    SURFACES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn debug_enabled() -> bool {
    std::env::var("AUTO_DOC_SURFACE_DEBUG").as_deref() == Ok("1")
}

/// 裸 HWND 宿主实现（wry build_as_child 的 HasWindowHandle 入参）。
struct RawHwndHost(isize);

impl HasWindowHandle for RawHwndHost {
    fn window_handle(
        &self,
    ) -> Result<WindowHandle<'_>, wry::raw_window_handle::HandleError> {
        let Some(nz) = NonZeroIsize::new(self.0) else {
            return Err(wry::raw_window_handle::HandleError::Unavailable);
        };
        let mut h = Win32WindowHandle::new(nz);
        h.hinstance = None;
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(h)) })
    }
}

/// painter draw 期同步：定位宿主、按需创建/导航/定位/显隐子表面。
pub fn sync_surface(key: &str, url: &str, bounds: &Rectangle) {
    let seq = SYNC_SEQ.fetch_add(1, Ordering::Relaxed);
    let host = locate_host(bounds);
    let mut map = surfaces().lock().unwrap();
    if url.is_empty() || host.is_none() {
        // 未就绪：已有表面保持隐藏（不遮 chrome）。
        if let Some(s) = map.get_mut(key) {
            if s.visible {
                WEBVIEWS.with(|w| {
                    if let Some(wv) = w.borrow_mut().get(key) {
                        let _ = wv.set_visible(false);
                    }
                });
                s.visible = false;
            }
        }
        if debug_enabled() {
            eprintln!(
                "[doc-surface #{seq}] defer key={key} url_empty={} host_found={}",
                url.is_empty(),
                host.is_some()
            );
        }
        return;
    }
    let hwnd = host.unwrap();
    let Some(phys) = canvas_physical_rect(hwnd, bounds) else {
        // 完全滚出客户区：隐藏。
        if let Some(s) = map.get_mut(key) {
            if s.visible {
                WEBVIEWS.with(|w| {
                    if let Some(wv) = w.borrow_mut().get(key) {
                        let _ = wv.set_visible(false);
                    }
                });
                s.visible = false;
            }
        }
        return;
    };

    if let std::collections::hash_map::Entry::Vacant(e) = map.entry(key.to_string()) {
        // phys 以 (0,0,0,0) 哨兵插入——首帧 create 后必然走 set_bounds
        //（wry 缺省 200x200 逻辑 bounds，否则子表面永远停在缺省矩形）。
        e.insert(SurfaceMeta { url: String::new(), hwnd, phys: (0, 0, 0, 0), visible: false });
    }
    let s = map.get_mut(key).unwrap();
    let mut action = "sync";
    let has_webview =
        WEBVIEWS.with(|w| w.borrow_mut().contains_key(key));
    if !has_webview || s.hwnd != hwnd {
        // 首建或宿主更换（会话子进程单窗；更换=异常重建路径）。
        let host_handle = RawHwndHost(hwnd);
        let built = wry::WebViewBuilder::new().with_url(url).build_as_child(&host_handle);
        match built {
            Ok(wv) => {
                WEBVIEWS.with(|w| {
                    w.borrow_mut().insert(key.to_string(), wv);
                });
                s.hwnd = hwnd;
                action = "create";
            }
            Err(err) => {
                eprintln!("[doc-surface] build_as_child failed: {err:?}");
                return;
            }
        }
    } else if s.url != url {
        WEBVIEWS.with(|w| {
            if let Some(wv) = w.borrow_mut().get(key) {
                let _ = wv.load_url(url);
            }
        });
        action = "navigate";
    }
    s.url = url.to_string();
    WEBVIEWS.with(|w| {
        let mut ws = w.borrow_mut();
        if let Some(wv) = ws.get_mut(key) {
            if s.phys != phys {
                let _ = wv.set_bounds(wry::Rect {
                    position: wry::dpi::PhysicalPosition::new(phys.0, phys.1).into(),
                    size: wry::dpi::PhysicalSize::new(phys.2, phys.3).into(),
                });
                s.phys = phys;
                action = if action == "sync" { "move" } else { action };
            }
            if !s.visible {
                let _ = wv.set_visible(true);
                s.visible = true;
                action = if action == "sync" { "show" } else { action };
            }
        }
    });
    if debug_enabled() {
        eprintln!(
            "[doc-surface #{seq}] {action} key={key} hwnd={:#x} phys={}x{}@{},{} url_len={}",
            hwnd,
            phys.2,
            phys.3,
            phys.0,
            phys.1,
            url.len()
        );
    }
}

/// 显式释放（同进程重建路径；会话退出由进程回收兜底）。须在主线程调用。
pub fn release_surface(key: &str) {
    surfaces().lock().unwrap().remove(key);
    WEBVIEWS.with(|w| {
        w.borrow_mut().remove(key);
    });
}

// ---------------------------------------------------------------------------
// canvas painter：draw 同步 + 底色占位（挂载后被子表面覆盖）。
// ---------------------------------------------------------------------------

pub struct DocSurfacePainter<M> {
    pub key: String,
    pub url: String,
    _message: std::marker::PhantomData<fn() -> M>,
}

impl<M> DocSurfacePainter<M> {
    pub fn new(key: String, url: String) -> Self {
        Self { key, url, _message: std::marker::PhantomData }
    }
}

impl<M> canvas::Program<M> for DocSurfacePainter<M> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        sync_surface(&self.key, &self.url, &bounds);
        let mut frame =
            canvas::Frame::new(renderer, Size::new(bounds.width, bounds.height));
        let rect = Path::rectangle(iced::Point::ORIGIN, frame.size());
        frame.fill(&rect, iced::Color::from_rgba8(0xF6, 0xF7, 0xF9, 1.0));
        vec![frame.into_geometry()]
    }
}
