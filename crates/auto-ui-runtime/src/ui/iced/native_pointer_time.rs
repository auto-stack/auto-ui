//! Windows press times for the standalone checked desktop window.

use std::{cell::RefCell, collections::VecDeque};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::GetDoubleClickTime,
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{GetMessageTime, WM_LBUTTONDOWN, WM_NCDESTROY},
    },
};

const POINTER_CLOCK: usize = 0x4155_5043;

#[derive(Default)]
struct Presses {
    window: Option<HWND>,
    times: VecDeque<u32>,
}

thread_local! {
    static PRESSES: RefCell<Option<Presses>> = const { RefCell::new(None) };
}

pub(crate) struct Capture;

impl Capture {
    pub(crate) fn start() -> Self {
        PRESSES.with(|state| *state.borrow_mut() = Some(Presses::default()));
        Self
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        let window = PRESSES.with(|state| state.borrow_mut().take()?.window);
        if let Some(window) = window {
            // The registration and teardown both run on the window thread.
            unsafe { let _ = RemoveWindowSubclass(window, Some(window_proc), POINTER_CLOCK); }
        }
    }
}

pub(crate) fn enabled() -> bool {
    PRESSES.with(|state| state.borrow().is_some())
}

pub(crate) fn attach(native: &dyn HasWindowHandle) {
    let Ok(handle) = native.window_handle() else { return };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else { return };
    let hwnd = HWND(handle.hwnd.get() as _);
    PRESSES.with(|state| {
        let mut state = state.borrow_mut();
        let Some(clock) = state.as_mut() else { return };
        if clock.window.is_none() {
            // Iced calls this with its live HWND on the window thread.
            if unsafe { SetWindowSubclass(hwnd, Some(window_proc), POINTER_CLOCK, 0) }.as_bool() {
                clock.window = Some(hwnd);
            } else {
                log::warn!("could not attach the checked desktop pointer clock");
            }
        }
    });
}

pub(crate) fn next_left_press() -> Option<(u32, u32)> {
    let time = PRESSES.with(|state| state.borrow_mut().as_mut()?.times.pop_front())?;
    // This queries the user's existing system setting.
    Some((time, unsafe { GetDoubleClickTime() }))
}

unsafe extern "system" fn window_proc(
    hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM, _id: usize, _data: usize,
) -> LRESULT {
    if message == WM_LBUTTONDOWN {
        // Sample during native dispatch, before Iced batches its input and can
        // spend time drawing or committing an earlier application event.
        let time = unsafe { GetMessageTime() } as u32;
        PRESSES.with(|state| {
            if let Some(clock) = state.borrow_mut().as_mut().filter(|clock| clock.window == Some(hwnd)) {
                clock.times.push_back(time);
            }
        });
    } else if message == WM_NCDESTROY {
        PRESSES.with(|state| {
            if let Some(clock) = state.borrow_mut().as_mut().filter(|clock| clock.window == Some(hwnd)) {
                *clock = Presses::default();
            }
        });
        unsafe { let _ = RemoveWindowSubclass(hwnd, Some(window_proc), POINTER_CLOCK); }
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}
