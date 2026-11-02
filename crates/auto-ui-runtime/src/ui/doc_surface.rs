//! PLAN-029 T-02：doc-surface 实验文档表面 leaf（probe-only，PLAN-029 合同 §2）。
//!
//! 证明链：真实 AutoUI 原生 VM 窗口内，**组件布局分配的区域**上挂 wry
//! WebView2 表面。表面宿主形态（T-04 视觉缺陷修复，用户桌面实测发现）：
//! iced 窗口自身呈现层会把子 HWND（WRY_WEBVIEW）整体盖住——子表面存在、
//! 收输入、CDP 可连，但像素永不出现在屏幕上（占位画布 #F6F7F9 直暴露，
//! 用户截图逐像素证实 RGB(246,247,249)）。表面改挂**归属弹出窗**
//! （WS_POPUP + owner=iced 窗 + TOOLWINDOW/NOACTIVATE）：真正进桌面 z 序，
//! DWM 必合成；bounds 仍逐帧同步组件矩形——布局分配语义不变，实现形态
//! 记入 SD-02。
//!
//! 其余纪律同前：
//! - 不触碰 remote-frame 任何臂（独立注册名 `doc-surface`）；
//! - 弹出窗 rect 严格取组件布局 rect（逻辑 × 窗口 DPI → 客户区物理 →
//!   ClientToScreen 屏幕物理）；url 空/滚出客户区时隐藏；
//! - webview 的 COM 初始化须在创建线程（winit 主线程；026 同缝实测，
//!   painter draw 即主线程渲染路径）；
//! - 宿主窗定位：EnumThreadWindows(当前线程) 可见顶层窗中，canvas 中心点
//!   （按候选窗 DPI 变换）落在其客户区的第一窗即宿主（探针单窗）。
//!
//! 诊断开关：`AUTO_DOC_SURFACE_DEBUG=1` 在 stderr 打每次同步（动作/bounds/
//! hwnd/url 长度），供 T-04/T-05 取证。
//!
//! T-05（N-05 hide-show 门）：url 空/滚出客户区的隐藏路径必须把**弹出窗
//! 本体**一并 SW_HIDE——修复前只隐 webview，弹出窗类底色矩形留在原位
//! （N-05.hide-show.hidden 门红→修复后绿）；展示路径 SW_SHOWNA 还原
//! （NOACTIVATE 语义保持，不抢前台）。

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

#[repr(C)]
struct WNDCLASSW {
    style: u32,
    lpfn_wnd_proc: isize,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: isize,
    h_icon: isize,
    h_cursor: isize,
    h_br_background: isize,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
}

extern "system" {
    fn EnumThreadWindows(dw_thread: u32, callback: isize, lparam: isize) -> i32;
    fn IsWindowVisible(hwnd: isize) -> i32;
    fn GetClientRect(hwnd: isize, rect: *mut RECT) -> i32;
    fn ClientToScreen(hwnd: isize, pt: *mut POINT) -> i32;
    fn GetDpiForWindow(hwnd: isize) -> u32;
    fn GetCurrentThreadId() -> u32;
    fn RegisterClassW(wc: *const WNDCLASSW) -> u16;
    fn CreateWindowExW(
        ex: u32, class: *const u16, title: *const u16, style: u32,
        x: i32, y: i32, w: i32, h: i32, parent: isize, menu: isize,
        inst: isize, param: isize,
    ) -> isize;
    fn DefWindowProcW(hwnd: isize, msg: u32, w: usize, l: isize) -> isize;
    fn GetModuleHandleW(name: *const u16) -> isize;
    fn GetLastError() -> u32;
    fn SetWindowPos(
        hwnd: isize, after: isize, x: i32, y: i32, cx: i32, cy: i32, flags: u32,
    ) -> i32;
    fn ShowWindow(hwnd: isize, cmd: i32) -> i32;
}

const SW_HIDE: i32 = 0;
const SW_SHOWNA: i32 = 8;

const WS_POPUP: u32 = 0x8000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_CLIPSIBLINGS: u32 = 0x0400_0000;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
const WHITE_BRUSH: isize = 0;
/// SWP_NOMOVE|SWP_NOSIZE|SWP_NOACTIVATE
const SWP_Z_ONLY: u32 = 0x0001 | 0x0002 | 0x0010;

unsafe extern "system" fn popup_proc(
    hwnd: isize, msg: u32, w: usize, l: isize,
) -> isize {
    // 最小 proc：弹出窗自身无消息语义，全部交 DefWindowProc。
    DefWindowProcW(hwnd, msg, w, l)
}

/// 注册一次弹出窗窗口类，返回类名指针（进程生命周期有效）。
fn popup_class() -> *const u16 {
    static CLS: OnceLock<&'static [u16]> = OnceLock::new();
    let name = CLS.get_or_init(|| {
        &*Box::leak(
            "AutoUIDocSurfacePopup\0"
                .encode_utf16()
                .collect::<Vec<u16>>()
                .into_boxed_slice(),
        )
    });
    static REGISTERED: OnceLock<()> = OnceLock::new();
    REGISTERED.get_or_init(|| unsafe {
        let hmod = GetModuleHandleW(std::ptr::null());
        let wc = WNDCLASSW {
            style: 0,
            lpfn_wnd_proc: popup_proc as isize,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: hmod,
            h_icon: 0,
            h_cursor: 0,
            h_br_background: WHITE_BRUSH + 1,
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: name.as_ptr(),
        };
        let atom = RegisterClassW(&wc);
        if atom == 0 {
            // 1410 = ERROR_CLASS_ALREADY_EXISTS：重复注册可复用；其余具名。
            let err = GetLastError();
            if err != 1410 {
                eprintln!("[doc-surface] RegisterClassW failed err={err}");
            }
        }
    });
    name.as_ptr()
}

thread_local! {
    /// 弹出窗句柄（主线程创建/复用；单表面探针 profile 一只）。
    static POPUP: std::cell::Cell<Option<isize>> = const { std::cell::Cell::new(None) };
    /// 宿主定位排除面：弹出窗与本线程同窗枚举会自命中（bounds 逐帧自收缩
    /// ——vis5 实测 1696x840 塌缩成 822x246），定位时必须跳过。
    static POPUP_EXCLUDE: std::cell::Cell<Option<isize>> = const { std::cell::Cell::new(None) };
}

/// 创建/复用归属弹出窗（owner=宿主 iced 窗，工具窗样式不进任务栏）。
fn ensure_popup_hwnd(owner: isize, x: i32, y: i32, w: i32, h: i32) -> Option<isize> {
    POPUP.with(|c| {
        if let Some(h) = c.get() {
            return Some(h);
        }
        unsafe {
            let cls = popup_class();
            let title: Vec<u16> = "AutoUI doc-surface\0".encode_utf16().collect();
            let hmod = GetModuleHandleW(std::ptr::null());
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                cls,
                title.as_ptr(),
                WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS,
                x, y, w.max(1), h.max(1),
                owner, 0, hmod, 0,
            );
            if hwnd == 0 {
                eprintln!(
                    "[doc-surface] CreateWindowExW failed err={} owner={owner:#x}",
                    GetLastError()
                );
                return None;
            }
            c.set(Some(hwnd));
            POPUP_EXCLUDE.with(|e| e.set(Some(hwnd)));
            Some(hwnd)
        }
    })
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
        if let Some(p) = POPUP_EXCLUDE.with(|c| c.get()) {
            if hwnd == p {
                return 1; // 跳过弹出窗自身
            }
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

/// canvas 逻辑 bounds → 宿主客户区坐标系的物理 rect。
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
// 表面注册表：key → 元数据（webview 本体非 Send，留主线程 thread_local）。
// ---------------------------------------------------------------------------

#[derive(Default)]
struct SurfaceMeta {
    url: String,
    hwnd: isize,
    phys: (i32, i32, u32, u32),
    visible: bool,
    /// 弹出窗自身可见性（N-05 隐藏门：url 空/滚出客户区时必须真隐藏，
    /// 否则类底色矩形留在原位遮住 iced 内容——修复前实测灰框残留）。
    popup_visible: bool,
}

/// 弹出窗显隐（SW_SHOWNA 不抢前台——NOACTIVATE 语义保持）。
fn set_popup_visible(hwnd: isize, show: bool) {
    unsafe {
        ShowWindow(hwnd, if show { SW_SHOWNA } else { SW_HIDE });
    }
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

/// painter draw 期同步：定位宿主、屏幕坐标换算、按需创建/导航/移动/显隐。
pub fn sync_surface(key: &str, url: &str, bounds: &Rectangle) {
    let seq = SYNC_SEQ.fetch_add(1, Ordering::Relaxed);
    let host = locate_host(bounds);
    let mut map = surfaces().lock().unwrap();
    if url.is_empty() || host.is_none() {
        // 未就绪：已有表面保持隐藏（不遮 chrome）。弹出窗必须一并真隐藏——
        // 只隐 webview 会留下类底色矩形（N-05 hide-show 门修复前实测）。
        if let Some(s) = map.get_mut(key) {
            if s.visible {
                WEBVIEWS.with(|w| {
                    if let Some(wv) = w.borrow_mut().get(key) {
                        let _ = wv.set_visible(false);
                    }
                });
                s.visible = false;
            }
            if s.popup_visible {
                if s.hwnd != 0 {
                    set_popup_visible(s.hwnd, false);
                }
                s.popup_visible = false;
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
    let host = host.unwrap();
    let Some(phys) = canvas_physical_rect(host, bounds) else {
        // 完全滚出客户区：webview 与弹出窗一并隐藏（同上，不留底色框）。
        if let Some(s) = map.get_mut(key) {
            if s.visible {
                WEBVIEWS.with(|w| {
                    if let Some(wv) = w.borrow_mut().get(key) {
                        let _ = wv.set_visible(false);
                    }
                });
                s.visible = false;
            }
            if s.popup_visible {
                if s.hwnd != 0 {
                    set_popup_visible(s.hwnd, false);
                }
                s.popup_visible = false;
            }
        }
        return;
    };
    // 客户区 rect → 屏幕物理 rect（弹出窗坐标系）。
    let mut origin = POINT { x: 0, y: 0 };
    unsafe { ClientToScreen(host, &mut origin) };
    let sx = origin.x + phys.0;
    let sy = origin.y + phys.1;
    let sw = phys.2 as i32;
    let sh = phys.3 as i32;
    let screen_phys = (sx, sy, sw as u32, sh as u32);

    let Some(popup) = ensure_popup_hwnd(host, sx, sy, sw, sh) else {
        eprintln!("[doc-surface] popup CreateWindowExW failed");
        return;
    };
    // 弹出窗跟随组件矩形移动/缩放（屏幕坐标）；SWP_Z_ONLY 的 z 跟随在
    // sync 尾部统一做。
    static LAST_PHYS: OnceLock<Mutex<HashMap<String, (i32, i32, i32, i32)>>> =
        OnceLock::new();
    let last = LAST_PHYS.get_or_init(|| Mutex::new(HashMap::new()));
    let moved = last
        .lock()
        .unwrap()
        .get(key)
        .map(|p| *p != (sx, sy, sw, sh))
        .unwrap_or(true);
    if moved {
        unsafe {
            // SWP_NOZORDER(0x4)|SWP_NOACTIVATE(0x10)
            SetWindowPos(popup, 0, sx, sy, sw, sh, 0x0004 | 0x0010);
        }
        last.lock().unwrap().insert(key.to_string(), (sx, sy, sw, sh));
    }

    if let std::collections::hash_map::Entry::Vacant(e) = map.entry(key.to_string()) {
        // phys 哨兵 (0,0,0,0)：首帧 create 后必然走 set_bounds。
        e.insert(SurfaceMeta {
            url: String::new(),
            hwnd: popup,
            phys: (0, 0, 0, 0),
            visible: false,
            popup_visible: false,
        });
    }
    let s = map.get_mut(key).unwrap();
    let mut action = "sync";
    let has_webview = WEBVIEWS.with(|w| w.borrow_mut().contains_key(key));
    if !has_webview || s.hwnd != popup {
        let popup_handle = RawHwndHost(popup);
        let built = wry::WebViewBuilder::new().with_url(url).build_as_child(&popup_handle);
        match built {
            Ok(wv) => {
                // 弹出窗内部坐标：webview 填满弹出窗客户区 (0,0,sw,sh)——
                // 误用屏幕坐标会把内容推到弹出窗外（用户实测灰/白/小字三层错位）。
                let _ = wv.set_bounds(wry::Rect {
                    position: wry::dpi::PhysicalPosition::new(0, 0).into(),
                    size: wry::dpi::PhysicalSize::new(sw as u32, sh as u32).into(),
                });
                WEBVIEWS.with(|w| {
                    w.borrow_mut().insert(key.to_string(), wv);
                });
                s.hwnd = popup;
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
            if s.phys != screen_phys {
                let _ = wv.set_bounds(wry::Rect {
                    position: wry::dpi::PhysicalPosition::new(0, 0).into(),
                    size: wry::dpi::PhysicalSize::new(sw as u32, sh as u32).into(),
                });
                s.phys = screen_phys;
                action = if action == "sync" { "move" } else { action };
            }
            if !s.popup_visible {
                set_popup_visible(popup, true);
                s.popup_visible = true;
                action = if action == "sync" { "popup-show" } else { action };
            }
            if !s.visible {
                let _ = wv.set_visible(true);
                s.visible = true;
                action = if action == "sync" { "show" } else { action };
            }
        }
    });
    // z 序跟随宿主：弹出窗始终贴在 owner 之上（owner 被别的窗盖住时弹出
    // 窗同样被盖——与普通窗口语义一致，不做 topmost）。
    unsafe { SetWindowPos(popup, host, 0, 0, 0, 0, SWP_Z_ONLY) };
    if debug_enabled() {
        eprintln!(
            "[doc-surface #{seq}] {action} key={key} popup={:#x} phys={}x{}@{},{} url_len={}",
            popup,
            sw,
            sh,
            sx,
            sy,
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
// canvas painter：draw 同步 + 底色占位（挂载后弹出窗覆盖其上）。
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
