//! QA-only native message posting, modeled on the authoring probe's helper.
//! Posts go to THIS process's window found by title; another process's window
//! is refused, and every posted mouse move moves the real pointer so real
//! WM_MOUSEMOVE tracking cannot replace the drag sample.
#[cfg(windows)]
pub fn post(title: &str, message: u32, wparam: usize, point: (i32, i32)) -> Result<(), String> {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn ClientToScreen(h: *mut std::ffi::c_void, point: *mut [i32; 2]) -> i32;
        fn SetCursorPos(x: i32, y: i32) -> i32;
        fn FindWindowW(class: *const u16, name: *const u16) -> *mut std::ffi::c_void;
        fn MapVirtualKeyW(code: u32, map_type: u32) -> u32;
        fn PostMessageW(h: *mut std::ffi::c_void, m: u32, w: usize, l: isize) -> i32;
        fn GetWindowThreadProcessId(h: *mut std::ffi::c_void, p: *mut u32) -> u32;
    }
    let wide: Vec<_> = title.encode_utf16().chain([0]).collect();
    let h = unsafe { FindWindowW(std::ptr::null(), wide.as_ptr()) };
    if h.is_null() {
        return Err("player window missing".into());
    }
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(h, &mut pid) };
    if pid != std::process::id() {
        return Err("refuse another process's window".into());
    }
    if message == 0x200 {
        let mut screen = [point.0, point.1];
        if unsafe { ClientToScreen(h, &mut screen) } == 0 {
            return Err("native ClientToScreen failed".into());
        }
        if unsafe { SetCursorPos(screen[0], screen[1]) } == 0 {
            return Err(format!(
                "native SetCursorPos failed at {screen:?}: {}",
                std::io::Error::last_os_error()
            ));
        }
    }
    let point = if message == 0x20a {
        let mut p = [point.0, point.1];
        if unsafe { ClientToScreen(h, &mut p) } == 0 {
            return Err("native ClientToScreen failed".into());
        }
        (p[0], p[1])
    } else {
        point
    };
    let packed = if message == 0x100 || message == 0x101 {
        let scan: u32 = unsafe { MapVirtualKeyW(wparam as u32, 0) };
        let extended: u32 = u32::from((0x25..=0x28).contains(&wparam)) << 24;
        // WM_KEYUP carries previous-state (bit 30) AND transition (bit 31)
        // set; without the transition bit the release reads as a repeat.
        let up: u32 = u32::from(message == 0x101) * 0xC0000000;
        (1u32 | (scan << 16) | extended | up) as isize
    } else {
        ((point.0 as u32 & 0xffff) | ((point.1 as u32 & 0xffff) << 16)) as isize
    };
    if unsafe { PostMessageW(h, message, wparam, packed) } == 0 {
        Err("native PostMessage failed".into())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn post(_: &str, _: u32, _: usize, _: (i32, i32)) -> Result<(), String> {
    Err("the standalone player QA requires Windows".into())
}

/// Virtual-Key codes the QA chains use.
pub const VK_SPACE: usize = 0x20;
pub const VK_RETURN: usize = 0x0D;
pub const VK_KEY_D: usize = 0x44;
pub const VK_KEY_W: usize = 0x57;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_MOUSEWHEEL: u32 = 0x020A;

/// The double-click failure surface: a startup/fatal error is named in a
/// blocking dialog (product entries only; QA entries fail through the report
/// so a hung dialog can never stall an automated run).
#[cfg(windows)]
pub fn fatal_message(title: &str, text: &str) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(h: *mut std::ffi::c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
    }
    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain([0]).collect() };
    let (t, c) = (wide(text), wide(title));
    unsafe {
        MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), 0x10);
    }
}

#[cfg(not(windows))]
pub fn fatal_message(_title: &str, _text: &str) {}
