//! PLAN-804: OS 文件对话框宿主实现（rfd，同步 API）——`dialog_open`/
//! `dialog_save`（natives 2927/2928）的单一 OS 真源。
//!
//! 迁移自两处重复构建：`vm/native.rs` 原 shim 内联实现（parented——
//! Plan 428 债修的父窗锚定）与 `ui/dialog.rs` a2r host（unparented——
//! PLAN-681 §9 F1 缝）。公共算法（过滤解析、默认文件名、取消空串约定）
//! 只在本模块维护一份；两调用方仅选择不同父窗策略：
//! - VM 默认 adapter（[`ParentedOsDialogProvider`]）：Windows 下锚定主窗
//!   （[`dialog_parent`]，自 vm/native.rs 原样迁入）；
//! - a2r 高层门面（[`unparented_open`]/[`unparented_save`]，`ui::dialog`
//!   薄委托）：无父窗（旧 a2r 策略诚实保留——模态父窗对 a2r shell 是
//!   独立接线关注点，非语义分叉）。
//!
//! 门控：ui-dialog 单门控（rfd + raw-window-handle 依赖面），**独立于
//! ui renderer 门**——ui-dialog-only（无 ui）构建恢复迁移前"高层对话
//! 框路径"（经 lib.rs 统一安装入口装配兼容默认）。
//!
//! 两者返回 "" on cancel/unavailable（headless CI included）。

use auto_lang_core::vm::host_capabilities::DialogHostProvider;
use std::sync::Arc;

/// 过滤解析（T-01 冻结语义，自 vm/native.rs 与 ui/dialog.rs 两处重复
/// 收敛为单一实现）：`,. ;`（逗号/分号/空格/tab）分割 → trim → 去前导
/// 点 → 弃空项。空表 = 不加 filter（不过滤）。
fn filter_exts(filter: &str) -> Vec<String> {
    filter
        .split([',', ';', ' ', '\t'])
        .map(|s| s.trim().trim_start_matches('.').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Plan 428 debt fix: parent the blocking dialogs to the app's main
/// window. The dialogs previously had NO owner — they could open BEHIND
/// the app window ("frozen" UX) or, destroyed abnormally, leak the owner
/// disabled forever (428 acceptance finding, KNOWN-DEBT-AND-RISKS 428).
/// The HWND is discovered via Win32 (largest visible top-level window of
/// this process — no iced plumbing needed) and cached for the process
/// lifetime. Non-Windows targets keep the plain dialog.
///（PLAN-804 自 vm/native.rs 原样迁入——旧 HWND 进程期缓存策略不在本
/// 计划重写范围，见计划 §1 非目标。）
#[cfg(target_os = "windows")]
mod dialog_parent {
    use raw_window_handle::{
        DisplayHandle, HasDisplayHandle, HasWindowHandle, RawWindowHandle, Win32WindowHandle,
        WindowHandle,
    };
    use std::cell::Cell;
    use std::num::NonZeroIsize;
    use std::sync::OnceLock;

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    const GW_OWNER: u32 = 4;

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, lparam: isize) -> i32;
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
        fn GetClassNameW(hwnd: isize, class: *mut u16, max: i32) -> i32;
        fn GetWindowRect(hwnd: isize, rect: *mut Rect) -> i32;
        fn GetWindowTextW(hwnd: isize, buf: *mut u16, max: i32) -> i32;
        fn IsWindowEnabled(hwnd: isize) -> i32;
        fn GetWindow(hwnd: isize, cmd: u32) -> isize;
        fn GetCurrentThreadId() -> u32;
    }

    fn win_class(hwnd: isize) -> String {
        let mut buf = [0u16; 64];
        let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) }.max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }

    fn win_title(hwnd: isize) -> String {
        let mut buf = [0u16; 128];
        let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) }.max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }

    thread_local! {
        /// (hwnd, area) of the best main-window candidate so far.
        static BEST: Cell<(isize, i64)> = const { Cell::new((0, -1)) };
    }

    extern "system" fn enum_cb(hwnd: isize, _lparam: isize) -> i32 {
        // Skip invisible helpers (GPU/IME) and non-app windows.
        if unsafe { IsWindowVisible(hwnd) } == 0 {
            return 1;
        }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid != std::process::id() {
            return 1;
        }
        let mut buf = [0u16; 64];
        let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) }.max(0) as usize;
        let class = String::from_utf16_lossy(&buf[..n]);
        // #32770 = Win32 dialog (a dialog can't own our dialog); the
        // winit event target is a message-only-ish shim, not the window.
        if class == "#32770" || class == "Winit Thread Event Target" {
            return 1;
        }
        let mut r = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { GetWindowRect(hwnd, &mut r) } == 0 {
            return 1;
        }
        let area = (r.right - r.left).max(0) as i64 * (r.bottom - r.top).max(0) as i64;
        BEST.with(|b| {
            if area > b.get().1 {
                b.set((hwnd, area));
            }
        });
        1
    }

    fn main_hwnd() -> Option<NonZeroIsize> {
        static HWND: OnceLock<Option<NonZeroIsize>> = OnceLock::new();
        *HWND.get_or_init(|| {
            BEST.with(|b| b.set((0, -1)));
            unsafe { EnumWindows(enum_cb, 0) };
            let (hwnd, area) = BEST.with(|b| b.get());
            (area > 0).then(|| NonZeroIsize::new(hwnd)).flatten()
        })
    }

    /// rfd's `set_parent` bound: a Win32 HWND + the trivial Windows
    /// display handle.
    struct Owner(NonZeroIsize);

    impl HasWindowHandle for Owner {
        fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
            let raw = RawWindowHandle::Win32(Win32WindowHandle::new(self.0));
            // SAFETY: the HWND came from EnumWindows on a live window and
            // is cached for the process lifetime.
            Ok(unsafe { WindowHandle::borrow_raw(raw) })
        }
    }

    impl HasDisplayHandle for Owner {
        fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
            Ok(DisplayHandle::windows())
        }
    }

    /// Attach `dialog` to the main window when one was found.
    pub fn attach(dialog: rfd::FileDialog) -> rfd::FileDialog {
        match main_hwnd() {
            Some(hwnd) => {
                // PLAN-081 T-01 父窗取证：锚定窗的类名/标题/可见/禁用/
                // owner 链/归属线程 vs 调用线程——错窗假设判据。
                if std::env::var_os("AUTO_DIALOG_DIAG").is_some() {
                    let h = hwnd.get();
                    let mut buf = [0u16; 128];
                    let n = unsafe { GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32) }.max(0)
                        as usize;
                    let title = String::from_utf16_lossy(&buf[..n]);
                    let mut cls = [0u16; 64];
                    let cn = unsafe { GetClassNameW(h, cls.as_mut_ptr(), cls.len() as i32) }.max(0)
                        as usize;
                    let class = String::from_utf16_lossy(&cls[..cn]);
                    let mut pid = 0u32;
                    let owner_tid = unsafe { GetWindowThreadProcessId(h, &mut pid) };
                    let cur_tid = unsafe { GetCurrentThreadId() };
                    let owner = unsafe { GetWindow(h, GW_OWNER) };
                    let owner_desc = if owner == 0 {
                        "none".to_string()
                    } else {
                        format!("{owner:#x}({} \"{}\")", win_class(owner), win_title(owner))
                    };
                    eprintln!(
                        "[DLG-PROBE] parent hwnd={h:#x} class={class} title=\"{title}\" \
                         visible={} enabled={} owner_tid={owner_tid} cur_tid={cur_tid} \
                         owner={owner_desc}",
                        unsafe { IsWindowVisible(h) },
                        unsafe { IsWindowEnabled(h) },
                    );
                }
                dialog.set_parent(&Owner(hwnd))
            }
            None => dialog,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// The handle wrappers must satisfy rfd's `set_parent` bound and
        /// construct without error (the HWND value itself is opaque).
        ///（PLAN-804 自 vm/native.rs 原样迁入。）
        #[test]
        fn owner_satisfies_rfd_parent_bound() {
            let owner = Owner(NonZeroIsize::new(1).unwrap());
            assert!(owner.window_handle().is_ok());
            assert!(owner.display_handle().is_ok());
            // set_parent takes `&W: HasWindowHandle + HasDisplayHandle`.
            fn bound<W: HasWindowHandle + HasDisplayHandle>(_: &W) {}
            bound(&owner);
        }
    }
}

#[cfg(target_os = "windows")]
use dialog_parent::attach;
#[cfg(not(target_os = "windows"))]
fn attach(dialog: rfd::FileDialog) -> rfd::FileDialog {
    dialog
}

/// PLAN-081 T-01 插桩探针（AUTO_DIALOG_DIAG 门控，Windows only）：mode A
/// 自毁/mode B 不生成的生命周期取证。四路证据一次覆盖——
/// 1. `CoGetApartmentType`：调用线程既有 COM 套间（MTA ⇒ rfd 的
///    APARTMENTTHREADED init 失败 ⇒ 静默 n=0，mode B 候选根因）；
/// 2. 父窗取证（在 [`dialog_parent::attach`] 内）：类名/标题/可见/禁用/
///    owner 链/归属线程——错窗假设判据；
/// 3. `WH_CBT` 线程钩子：show 期间本线程每个窗口的 create/destroy/
///    activate/SC_CLOSE 时序——"对话框是否真出现过、何时消失"判据；
/// 4. 首个 `#32770` 子类化：WM_CLOSE/WM_COMMAND(IDCANCEL/IDOK)/
///    WM_SYSCOMMAND(SC_CLOSE)/WM_DESTROY 时序+当时前台窗/UIA 监听态——
///    "谁经哪条消息关闭"判据。
/// 门未设时零安装（无钩无子类无额外 OS 面）；钩子/子类化回调只读日志、
/// 恒转发原 proc，不改变对话框行为。单模态假设：同一时刻至多一个对话框
/// （主线程同步 pick），探针状态用原子量、单线程语义即可。
#[cfg(target_os = "windows")]
mod diag_probe {
    use std::cell::Cell;
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

    const WH_CBT: i32 = 5;
    const HCBT_CREATEWND: i32 = 3;
    const HCBT_DESTROYWND: i32 = 4;
    const HCBT_ACTIVATE: i32 = 5;
    const HCBT_SYSCOMMAND: i32 = 8;
    const GWLP_WNDPROC: i32 = -4;
    const WM_DESTROY: usize = 0x0002;
    const WM_NCDESTROY: usize = 0x0082;
    const WM_CLOSE: usize = 0x0010;
    const WM_COMMAND: usize = 0x0111;
    const WM_SYSCOMMAND: usize = 0x0112;
    const SC_CLOSE: usize = 0xF060;
    const GW_OWNER: u32 = 4;

    #[link(name = "user32")]
    extern "system" {
        fn SetWindowsHookExW(
            idhook: i32,
            proc_fn: extern "system" fn(i32, usize, isize) -> isize,
            hmod: isize,
            threadid: u32,
        ) -> isize;
        fn UnhookWindowsHookEx(hhk: isize) -> i32;
        fn CallNextHookEx(hhk: isize, code: i32, wparam: usize, lparam: isize) -> isize;
        fn GetCurrentThreadId() -> u32;
        fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
        fn CallWindowProcW(prev: isize, hwnd: isize, msg: usize, wp: usize, lp: isize) -> isize;
        fn GetWindowTextW(hwnd: isize, buf: *mut u16, max: i32) -> i32;
        fn IsWindowEnabled(hwnd: isize) -> i32;
        fn GetWindow(hwnd: isize, cmd: u32) -> isize;
        fn GetForegroundWindow() -> isize;
        fn GetClassNameW(hwnd: isize, class: *mut u16, max: i32) -> i32;
    }
    #[link(name = "ole32")]
    extern "system" {
        fn CoGetApartmentType(ptype: *mut i32, pqual: *mut i32) -> i32;
    }
    #[link(name = "uiautomationcore")]
    extern "system" {
        fn UiaClientsAreListening() -> i32;
    }

    thread_local! {
        /// 探针安装时刻（钩子/子类化回调恒在本线程触发，thread_local 即可）。
        static T0: Cell<Option<std::time::Instant>> = const { Cell::new(None) };
    }

    static HOOK: AtomicIsize = AtomicIsize::new(0);
    static DLG_HWND: AtomicIsize = AtomicIsize::new(0);
    static OLD_PROC: AtomicIsize = AtomicIsize::new(0);
    /// 防重入（嵌套 show 不存在，但守卫 drop 顺序异常时不得二次卸钩）。
    static INSTALLED: AtomicBool = AtomicBool::new(false);

    fn ms() -> u128 {
        T0.with(|t| t.get().map_or(0, |t0| t0.elapsed().as_millis()))
    }

    fn diag(tag: &str, body: &str) {
        eprintln!("[DLG-PROBE] {tag} {body} t={}ms", ms());
    }

    fn hwnd_class(hwnd: isize) -> String {
        let mut buf = [0u16; 64];
        let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) }.max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }

    fn hwnd_title(hwnd: isize) -> String {
        let mut buf = [0u16; 128];
        let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) }.max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }

    /// 调用线程既有套间 + 前台窗/UIA 监听快照（进入/关闭两点各记一次）。
    pub fn note_state(tag: &str) {
        let (mut apt, mut qual) = (-1i32, -1i32);
        let hr = unsafe { CoGetApartmentType(&mut apt, &mut qual) };
        let apt_name = match apt {
            0 => "STA",
            1 => "MTA",
            2 => "NA",
            3 => "MAINSTA",
            _ => "?",
        };
        let fg = unsafe { GetForegroundWindow() };
        let fg_desc = if fg == 0 {
            "none".to_string()
        } else {
            format!("{}({})", hwnd_class(fg), hwnd_title(fg))
        };
        let uia = unsafe { UiaClientsAreListening() };
        diag(
            tag,
            &format!(
                "apt={apt_name} hr=0x{hr:08x} tid={} fg={fg_desc} uia_listening={uia}",
                unsafe { GetCurrentThreadId() }
            ),
        );
    }

    /// RAII 探针：install ⇒ CBT 钩子；drop ⇒ 卸钩 + 关闭态快照。
    pub struct Probe;

    impl Probe {
        pub fn install() -> Option<Probe> {
            if INSTALLED.swap(true, Ordering::Relaxed) {
                return None;
            }
            T0.with(|t| t.set(Some(std::time::Instant::now())));
            DLG_HWND.store(0, Ordering::Relaxed);
            OLD_PROC.store(0, Ordering::Relaxed);
            let hook =
                unsafe { SetWindowsHookExW(WH_CBT, cbt_proc, 0, unsafe { GetCurrentThreadId() }) };
            if hook == 0 {
                INSTALLED.store(false, Ordering::Relaxed);
                diag("install", "CBT hook FAILED");
                return None;
            }
            HOOK.store(hook, Ordering::Relaxed);
            diag("install", "CBT hook ok");
            Some(Probe)
        }
    }

    impl Drop for Probe {
        fn drop(&mut self) {
            note_state("close-time");
            let hook = HOOK.swap(0, Ordering::Relaxed);
            if hook != 0 {
                unsafe { UnhookWindowsHookEx(hook) };
            }
            INSTALLED.store(false, Ordering::Relaxed);
            diag("remove", "CBT hook uninstalled");
        }
    }

    extern "system" fn cbt_proc(code: i32, wparam: usize, lparam: isize) -> isize {
        match code {
            HCBT_CREATEWND => {
                let hwnd = wparam as isize;
                let class = hwnd_class(hwnd);
                diag("create", &format!("hwnd={hwnd:#x} class={class}"));
                // 子类化本 show 的首个 #32770（IFileDialog 宿主窗）。
                if class == "#32770"
                    && DLG_HWND
                        .compare_exchange(0, hwnd, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                {
                    let prev = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, dlg_proc as isize) };
                    OLD_PROC.store(prev, Ordering::Relaxed);
                    diag("subclass", &format!("hwnd={hwnd:#x} prev={prev:#x}"));
                }
            }
            HCBT_DESTROYWND => {
                let hwnd = wparam as isize;
                diag(
                    "destroy",
                    &format!("hwnd={hwnd:#x} class={}", hwnd_class(hwnd)),
                );
            }
            HCBT_ACTIVATE => {
                let hwnd = wparam as isize;
                // lparam = 被"挤下"的激活窗（句柄低半）——前后台切换链。
                let deact = (lparam as usize & 0xffff_ffff) as isize;
                diag(
                    "activate",
                    &format!(
                        "hwnd={hwnd:#x} class={} title={} deact={deact:#x}({})",
                        hwnd_class(hwnd),
                        hwnd_title(hwnd),
                        if deact == 0 {
                            "none".to_string()
                        } else {
                            hwnd_class(deact)
                        }
                    ),
                );
            }
            HCBT_SYSCOMMAND => {
                diag(
                    "syscommand",
                    &format!("cmd={:#x} hwnd={:#x}", wparam, lparam),
                );
            }
            _ => {}
        }
        unsafe { CallNextHookEx(0, code, wparam, lparam) }
    }

    /// #32770 子类 proc：只读日志 + 恒转发。NCDESTROY 先还原原 proc 再转发。
    extern "system" fn dlg_proc(hwnd: isize, msg: usize, wp: usize, lp: isize) -> isize {
        match msg {
            WM_CLOSE => {
                let fg = unsafe { GetForegroundWindow() };
                diag(
                    "wm_close",
                    &format!(
                        "hwnd={hwnd:#x} fg={fg:#x}({}) uia={}",
                        if fg == 0 {
                            "none".to_string()
                        } else {
                            hwnd_class(fg)
                        },
                        unsafe { UiaClientsAreListening() }
                    ),
                );
            }
            WM_SYSCOMMAND => {
                if wp & 0xfff0 == SC_CLOSE {
                    diag("wm_syscommand", &format!("SC_CLOSE hwnd={hwnd:#x}"));
                }
            }
            WM_COMMAND => {
                let id = wp & 0xffff;
                if id == 1 || id == 2 {
                    diag(
                        "wm_command",
                        &format!(
                            "id={id} ({}) hwnd={hwnd:#x}",
                            if id == 1 { "IDOK" } else { "IDCANCEL" }
                        ),
                    );
                }
            }
            WM_DESTROY => diag("wm_destroy", &format!("hwnd={hwnd:#x}")),
            WM_NCDESTROY => {
                let prev = OLD_PROC.swap(0, Ordering::Relaxed);
                DLG_HWND.store(0, Ordering::Relaxed);
                if prev != 0 {
                    unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, prev) };
                    return unsafe { CallWindowProcW(prev, hwnd, msg, wp, lp) };
                }
            }
            _ => {}
        }
        let prev = OLD_PROC.load(Ordering::Relaxed);
        if prev != 0 {
            unsafe { CallWindowProcW(prev, hwnd, msg, wp, lp) }
        } else {
            0
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod diag_probe {
    /// 非 Windows：探针不装配（现网诊断只在 Windows 形态运行）。
    pub fn note_state(_tag: &str) {}
    pub struct Probe;
    impl Probe {
        pub fn install() -> Option<Probe> {
            None
        }
    }
}

/// 单一 OS open 实现（策略仅父窗一项）：filter 解析 → 非空加 filter →
/// 按策略锚定父窗 → pick_file → `to_string_lossy`（取消/不可用 = ""）。
/// PLAN-079 T-04: 插桩点④——host 对话框生成入口（线程/时刻）。
fn os_open(attach_parent: bool, filter: &str) -> String {
    let diag = std::env::var_os("AUTO_DIALOG_DIAG").is_some();
    if diag {
        eprintln!(
            "[DLG-DIAG] host open enter filter={} thread={:?} t={}ms",
            filter,
            std::thread::current().id(),
            crate::ui::dynamic::sched_diag_t0().elapsed().as_millis(),
        );
        diag_probe::note_state("open enter");
    }
    let exts = filter_exts(filter);
    let mut dialog = rfd::FileDialog::new();
    if !exts.is_empty() {
        dialog = dialog.add_filter("Files", &exts);
    }
    if attach_parent {
        dialog = attach(dialog);
    }
    let _probe = if diag {
        diag_probe::Probe::install()
    } else {
        None
    };
    let out: String = dialog
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    drop(_probe);
    if diag {
        eprintln!(
            "[DLG-DIAG] host open exit len={} t={}ms",
            out.len(),
            crate::ui::dynamic::sched_diag_t0().elapsed().as_millis(),
        );
    }
    out
}

/// Multi-select shares the same filter and parent policy as single-select.
fn os_open_files(attach_parent: bool, filter: &str) -> Vec<String> {
    let diag = std::env::var_os("AUTO_DIALOG_DIAG").is_some();
    if diag {
        eprintln!(
            "[DLG-DIAG] host open_files enter filter={} thread={:?} t={}ms",
            filter,
            std::thread::current().id(),
            crate::ui::dynamic::sched_diag_t0().elapsed().as_millis(),
        );
        diag_probe::note_state("open_files enter");
    }
    let exts = filter_exts(filter);
    let mut dialog = rfd::FileDialog::new();
    if !exts.is_empty() {
        dialog = dialog.add_filter("Files", &exts);
    }
    if attach_parent {
        dialog = attach(dialog);
    }
    let _probe = if diag {
        diag_probe::Probe::install()
    } else {
        None
    };
    let out: Vec<String> = dialog
        .pick_files()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    drop(_probe);
    if diag {
        eprintln!(
            "[DLG-DIAG] host open_files exit n={} t={}ms",
            out.len(),
            crate::ui::dynamic::sched_diag_t0().elapsed().as_millis(),
        );
    }
    out
}

/// 单一 OS save 实现：default_name 非空才 set_file_name → 按策略锚定
/// 父窗 → save_file → `to_string_lossy`（取消 = ""）。
fn os_save(attach_parent: bool, default_name: &str) -> String {
    let mut dialog = rfd::FileDialog::new();
    if !default_name.is_empty() {
        dialog = dialog.set_file_name(default_name);
    }
    if attach_parent {
        dialog = attach(dialog);
    }
    dialog
        .save_file()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// VM 默认 dialog host provider：**parented** 策略（迁移前 vm/native.rs
/// shim 内联实现的逐条语义——Windows 下锚定主窗，Plan 428 债修保持）。
pub struct ParentedOsDialogProvider;

impl DialogHostProvider for ParentedOsDialogProvider {
    fn open_files(&self, filter: &str) -> Result<Vec<String>, String> {
        Ok(os_open_files(true, filter))
    }

    fn open(&self, filter: &str) -> String {
        os_open(true, filter)
    }

    fn save(&self, default_name: &str) -> String {
        os_save(true, default_name)
    }
}

/// a2r 高层 open 门面（**unparented** 策略，旧 `ui::dialog::open` 语义——
/// PLAN-681 §9 F1 缝诚实保留）。`ui::dialog::open` 薄委托本函数。
pub fn unparented_open_files(filter: &str) -> String {
    serde_json::to_string(&os_open_files(false, filter))
        .expect("a vector of paths is JSON serializable")
}

pub fn unparented_open(filter: &str) -> String {
    os_open(false, filter)
}

/// a2r 高层 save 门面（**unparented** 策略，旧 `ui::dialog::save` 语义）。
pub fn unparented_save(default_name: &str) -> String {
    os_save(false, default_name)
}

/// 装配 UI 兼容默认 dialog host provider = parented OS adapter
///（**仅当进程默认槽为空时**安装——`set_..._if_absent` 同锁原子；优先
/// 关系与 779/785/790/794 逐条相同）：
/// 1. 调用方**显式**注册（`set_process_default_dialog_host_provider`）
///    恒胜——本兼容安装不覆盖；
/// 2. 空槽时补缺（ui-dialog 构建下内建宿主入口恢复迁移前对话框接线，
///    含 ui-dialog-only 无 ui 形态）；
/// 3. 实例级 `AutoVM::set_dialog_host_provider` 在 1/2 之上按 VM 独立
///    生效（构造期快照 + 实例覆盖，不受后续进程默认变化影响）。
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_dialog_host_provider_if_absent(
        Arc::new(ParentedOsDialogProvider),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F779-R2-01 优先关系镜像锚（779/785/790/794 同款纪律）：兼容安装
    /// 不覆盖显式默认；空槽才补缺；显式 set 恒可替换。Fake provider——
    /// 不弹真 OS 对话框（真模态证据 = V04 env 门控实机档）。F-785-R1-01：
    /// 本测试 clear/set/restore 写默认槽——须持 crate 级共享隔离锁
    /// （test_default_slot_lock），与缺席断言/桥装配测试互斥。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        struct Fake(&'static str);
        impl DialogHostProvider for Fake {
            fn open(&self, _filter: &str) -> String {
                format!("{}:open", self.0)
            }
            fn save(&self, _default_name: &str) -> String {
                format!("{}:save", self.0)
            }
        }

        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved = auto_lang_core::vm::host_capabilities::process_default_dialog_host_provider();

        // 空槽 → 补缺成功（仅入槽，不弹 OS 对话框——ParentedOsDialogProvider
        // 惰性：不调用即零 OS 面）。
        auto_lang_core::vm::host_capabilities::clear_process_default_dialog_host_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_dialog_host_provider().is_some(),
            "空槽时兼容默认须补缺"
        );

        // 显式默认 → 兼容安装不覆盖（哨兵可区分）。
        auto_lang_core::vm::host_capabilities::set_process_default_dialog_host_provider(Arc::new(
            Fake("@plan804-explicit:哨兵-🎉"),
        ));
        install_process_default();
        let cur =
            auto_lang_core::vm::host_capabilities::process_default_dialog_host_provider().unwrap();
        assert_eq!(
            DialogHostProvider::open(&*cur, "f"),
            "@plan804-explicit:哨兵-🎉:open",
            "兼容安装不得覆盖显式默认"
        );

        // 还原。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_dialog_host_provider(p)
            }
            None => {
                auto_lang_core::vm::host_capabilities::clear_process_default_dialog_host_provider()
            }
        }
    }

    /// 过滤解析单一实现锚（T-01 冻结语义）：分割符全集、去前导点、弃空、
    /// 混合分隔、全空输入 = 无 filter。
    #[test]
    fn filter_exts_frozen_semantics() {
        assert_eq!(filter_exts(".at, .au"), vec!["at", "au"]);
        assert_eq!(filter_exts("at;au\tmd  png"), vec!["at", "au", "md", "png"]);
        assert_eq!(filter_exts(" . . ;;; ,,"), Vec::<String>::new());
        assert_eq!(filter_exts(""), Vec::<String>::new());
        // 前导多点只剥一层点前缀的等效效果（trim_start_matches 全剥点）。
        assert_eq!(filter_exts("..hidden"), vec!["hidden"]);
        // Unicode 过滤词不受影响（非 ASCII 扩展名原样保留）。
        assert_eq!(filter_exts("文件型🎉"), vec!["文件型🎉"]);
    }
}
