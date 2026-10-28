//! Self-checking native fixture: only OS messages drive user commands.
use autographics_auto_driver::authoring::{document::Document, host::Host, interaction};
use glam::Vec3;
use serde_json::{json, Value};
pub struct Probe {
    phase: u8,
    at: u64,
    axis: usize,
    baseline: Option<Document>,
    pixels: Option<String>,
    revision: u64,
    pub records: Vec<Value>,
    pub done: bool,
    pub error: Option<String>,
    wrong: bool,
    drop_release: bool,
    pointer_prepared: bool,
}
impl Probe {
    pub fn new(wrong: bool, drop_release: bool) -> Self {
        Self {
            phase: 0,
            at: 0,
            axis: 0,
            baseline: None,
            pixels: None,
            revision: 0,
            records: vec![],
            done: false,
            error: None,
            wrong,
            drop_release,
            pointer_prepared: false,
        }
    }
    pub fn phase(&self) -> u8 {
        self.phase
    }
    fn advance(&mut self, frame: u64) {
        self.phase += 1;
        self.at = frame;
    }
    pub fn step(&mut self, h: &mut Host, frame: u64, window: &winit::window::Window) {
        if self.done || self.error.is_some() || frame - self.at < 12 {
            return;
        }
        if frame - self.at > 240 {
            self.error = Some(format!("native interaction timeout phase {}", self.phase));
            return;
        }
        let result = self.run(h, frame, window);
        if let Err(error) = result {
            self.error = Some(format!("native phase {}: {error}", self.phase));
        }
    }
    fn check(ok: bool, why: &str) -> Result<(), String> {
        if ok {
            Ok(())
        } else {
            Err(why.into())
        }
    }
    pub(super) fn input(h: &Host, p: [f32; 2], message: u32) -> Result<(), String> {
        post(
            message,
            if message == 0x201 { 1 } else { 0 },
            (
                (p[0] * h.input.window_size[0] / 1200.0) as i32,
                (p[1] * h.input.window_size[1] / 800.0) as i32,
            ),
        )
    }
    pub(super) fn down(h: &Host, p: [f32; 2]) -> Result<(), String> {
        Self::input(h, p, 0x200)?;
        Self::input(h, p, 0x201)
    }
    pub(super) fn up(h: &Host, p: [f32; 2]) -> Result<(), String> {
        Self::input(h, p, 0x200)?;
        Self::input(h, p, 0x202)
    }
    pub(super) fn scroll(h: &Host, down: bool) -> Result<(), String> {
        let point = (
            (1000.0 * h.input.window_size[0] / 1200.0) as i32,
            (500.0 * h.input.window_size[1] / 800.0) as i32,
        );
        post(0x200, 0, point)?;
        let delta = if down { -240i16 } else { 240i16 };
        post(0x20a, ((delta as u16 as u32) << 16) as usize, point)
    }
    pub(super) fn click(h: &Host, p: [f32; 2]) -> Result<(), String> {
        Self::down(h, p)?;
        Self::up(h, p)
    }
    fn axis_points(&self, h: &Host) -> Result<([f32; 2], [f32; 2]), String> {
        let o = h
            .editor
            .document
            .objects
            .iter()
            .find(|o| o.id == 3)
            .ok_or("player missing")?;
        let (_, a, d) = interaction::axes(o, h.camera())
            .into_iter()
            .find(|x| x.0 == self.axis)
            .ok_or("axis disabled")?;
        let start = a + d.normalize() * 40.0;
        let end = start + d * 0.4;
        Ok(([start.x, start.y + 400.0], [end.x, end.y + 400.0]))
    }
    fn run(
        &mut self,
        h: &mut Host,
        frame: u64,
        window: &winit::window::Window,
    ) -> Result<(), String> {
        match self.phase {
            0 => {
                // Posted WM_MOUSEMOVE alone does not move the physical pointer. TrackMouseEvent
                // can otherwise deliver a real CursorLeft immediately after the synthetic down.
                // Establish native input preconditions; never suppress the application's cancel path.
                if !window.has_focus() {
                    window.focus_window();
                    return Ok(());
                }
                let point = if self.wrong {
                    [455.0, 190.0]
                } else {
                    [345.0, 190.0]
                };
                if !self.pointer_prepared {
                    window
                        .set_cursor_position(winit::dpi::PhysicalPosition::new(
                            point[0] as f64 * h.input.window_size[0] as f64 / 1200.0,
                            point[1] as f64 * h.input.window_size[1] as f64 / 800.0,
                        ))
                        .map_err(|e| format!("native pointer precondition: {e}"))?;
                    self.pointer_prepared = true;
                    self.at = frame;
                    return Ok(());
                }
                self.baseline = Some(h.editor.document.clone());
                Self::down(h, point)?;
            }
            1 => {
                Self::check(
                    h.editor.selection == Some(1) && h.gesture_active(),
                    "wrong native target/down",
                )?;
                self.pixels = Some(h.rt.capture(h.two, "native-selected").ok_or("capture")?.0);
                Self::input(h, [365.0, 200.0], 0x200)?;
            }
            2 => {
                let state = h.interaction_snapshot();
                if state["shown_sample"].as_u64().unwrap_or(0) == 0 {
                    return Ok(());
                }
                Self::check(
                    h.editor.document == *self.baseline.as_ref().unwrap()
                        && h.editor.history() == (0, 0, 0)
                        && !h.editor.dirty(),
                    "temporary changed accepted state",
                )?;
                Self::check(
                    h.rt.capture(h.two, "native-temporary").ok_or("capture")?.0
                        != *self.pixels.as_ref().unwrap(),
                    "temporary pixels unchanged",
                )?;
                Self::check(
                    state["presented"][0]["translation"] == json!([-35.0, -10.0, 0.0]),
                    "temporary wrong Auto position",
                )?;
                self.records
                    .push(json!({"case":"native-temporary","state":state}));
                Self::input(h, [375.0, 205.0], 0x200)?;
            }
            3 => {
                if !self.drop_release {
                    Self::up(h, [375.0, 205.0])?;
                }
            }
            4 => {
                if h.gesture_active() {
                    return Ok(());
                }
                let mut expected = self.baseline.clone().unwrap();
                expected.objects[0].translation = [-25.0, -15.0, 0.0];
                Self::check(
                    h.editor.document == expected
                        && h.editor.history().0 == 1
                        && h.editor.stamp.revision == 1,
                    "release not exactly one final endpoint",
                )?;
                Self::click(h, [842.0, 146.0])?;
            }
            5 => {
                if h.editor.pending() {
                    return Ok(());
                }
                Self::check(
                    h.editor.document == *self.baseline.as_ref().unwrap()
                        && !h.editor.dirty()
                        && h.editor.history().0 == 0,
                    "native undo",
                )?;
                Self::click(h, [901.0, 146.0])?;
            }
            6 => {
                if h.editor.pending() {
                    return Ok(());
                }
                Self::check(
                    h.editor.document.objects[0].translation == [-25.0, -15.0, 0.0]
                        && h.editor.history().0 == 1,
                    "native redo",
                )?;
                Self::click(h, [842.0, 146.0])?;
            }
            7 => {
                if h.editor.pending() {
                    return Ok(());
                }
                self.revision = h.editor.stamp.revision;
                Self::down(h, [345.0, 190.0])?;
            }
            8 => {
                Self::input(h, [365.0, 200.0], 0x200)?;
            }
            9 => {
                if h.interaction_snapshot()["shown_sample"]
                    .as_u64()
                    .unwrap_or(0)
                    == 0
                {
                    return Ok(());
                }
                post(0x100, 0x1b, (0, 0))?;
                post(0x101, 0x1b, (0, 0))?;
            }
            10 => {
                Self::check(
                    !h.gesture_active()
                        && h.editor.document == *self.baseline.as_ref().unwrap()
                        && h.editor.stamp.revision == self.revision,
                    "native Esc did not cancel",
                )?;
                Self::check(
                    h.rt.capture(h.two, "native-esc").ok_or("capture")?.0
                        == *self.pixels.as_ref().unwrap(),
                    "Esc pixels not restored",
                )?;
                Self::up(h, [365.0, 200.0])?;
                let player = h
                    .editor
                    .document
                    .objects
                    .iter()
                    .find(|o| o.id == 3)
                    .unwrap();
                let p = interaction::project(h.camera(), Vec3::from(player.translation))
                    .ok_or("project")?;
                Self::click(h, [p.x, p.y + 400.0])?;
            }
            11 => {
                Self::check(h.editor.selection == Some(3), "native GLB pick")?;
                let (a, _) = self.axis_points(h)?;
                self.revision = h.editor.stamp.revision;
                self.pixels = Some(h.rt.capture(h.three, "axis-start").ok_or("capture")?.0);
                Self::down(h, a)?;
            }
            12 => {
                Self::check(h.gesture_active(), "axis down")?;
                let (_, b) = self.axis_points(h)?;
                Self::input(h, b, 0x200)?;
            }
            13 => {
                if h.interaction_snapshot()["shown_sample"]
                    .as_u64()
                    .unwrap_or(0)
                    == 0
                {
                    return Ok(());
                }
                Self::check(
                    h.editor.stamp.revision == self.revision,
                    "axis temporary committed",
                )?;
                Self::check(
                    h.rt.capture(h.three, "axis-temporary").ok_or("capture")?.0
                        != *self.pixels.as_ref().unwrap(),
                    "axis temporary pixels unchanged",
                )?;
                let (_, b) = self.axis_points(h)?;
                Self::up(h, b)?;
            }
            14 => {
                if h.gesture_active() {
                    return Ok(());
                }
                let mut expected = self.baseline.clone().unwrap();
                let (start, end) = self.axis_points(h)?;
                let scale = glam::Vec2::new(
                    h.input.window_size[0] / 1200.0,
                    h.input.window_size[1] / 800.0,
                );
                let quantize = |p: [f32; 2]| (glam::Vec2::from(p) * scale).floor() / scale;
                let delta = quantize(end) - quantize(start);
                let (_, _, projection) =
                    interaction::axes(&h.editor.document.objects[2], h.camera())
                        .into_iter()
                        .find(|x| x.0 == self.axis)
                        .ok_or("axis projection")?;
                let actual_scalar = delta.dot(projection) / projection.length_squared();
                expected.objects[2].translation[self.axis] += actual_scalar;
                for (a, b) in h.editor.document.objects.iter().zip(&expected.objects) {
                    let mut a = a.clone();
                    for i in 0..3 {
                        Self::check(
                            (a.translation[i] - b.translation[i]).abs() < 1e-3,
                            "axis endpoint tolerance",
                        )?;
                    }
                    a.translation = b.translation;
                    Self::check(a == *b, "axis changed unrelated object/property")?;
                }
                Self::check(
                    h.editor.stamp.revision == self.revision + 1 && h.editor.history().0 == 1,
                    "axis history",
                )?;
                self.records.push(json!({"axis":self.axis,"translation":h.editor.document.objects[2].translation}));
                Self::click(h, [842.0, 146.0])?;
            }
            15 => {
                if h.editor.pending() {
                    return Ok(());
                }
                Self::check(
                    h.editor.document == *self.baseline.as_ref().unwrap(),
                    "axis undo",
                )?;
                self.axis += 1;
                if self.axis < 3 {
                    self.phase = 11;
                    self.at = frame;
                    return Ok(());
                }
                Self::down(h, [345.0, 190.0])?;
            }
            16 => {
                Self::input(h, [365.0, 200.0], 0x200)?;
            }
            17 => {
                if h.interaction_snapshot()["shown_sample"]
                    .as_u64()
                    .unwrap_or(0)
                    == 0
                {
                    return Ok(());
                }
                self.revision = h.editor.stamp.revision;
                h.fail_resource = true;
                Self::up(h, [395.0, 215.0])?;
            }
            18 => {
                if h.gesture_active() {
                    return Ok(());
                }
                Self::check(
                    h.editor.document == *self.baseline.as_ref().unwrap()
                        && h.editor.stamp.revision == self.revision
                        && h.editor.history().0 == 0,
                    "final resource failure committed prior preview",
                )?;
                Self::check(!h.ui().error.is_empty(), "failure not reported")?;
                h.fail_resource = false;
                Self::down(h, [345.0, 190.0])?;
            }
            19 => {
                Self::up(h, [355.0, 190.0])?;
            }
            20 => {
                if h.gesture_active() {
                    return Ok(());
                }
                Self::check(
                    h.editor.document.objects[0].translation == [-45.0, 0.0, 0.0]
                        && h.editor.history().0 == 1,
                    "postfailure recovery",
                )?;
                Self::click(h, [901.0, 110.0])?;
            }
            21 => {
                Self::check(!h.editor.dirty() && h.storage.path.is_some(), "native Save")?;
                self.records.push(json!({"case":"saved","path":h.storage.path,"hash":h.editor.document.hash()?,"revision":h.editor.stamp.revision,"compile_us":h.gesture_compile_us,"pump_us":h.gesture_pump_us,"resources":h.resource_snapshot()}));
                self.revision = h.editor.stamp.revision;
                Self::down(h, [355.0, 190.0])?;
            }
            22 => {
                let _ = window.request_inner_size(winit::dpi::LogicalSize::new(1000.0, 700.0));
            }
            23 => {
                Self::check(
                    !h.gesture_active() && h.editor.stamp.revision == self.revision,
                    "resize failed to cancel gesture",
                )?;
                Self::up(h, [375.0, 190.0])?;
                Self::down(h, [355.0, 190.0])?;
            }
            24 => {
                Self::check(h.gesture_active(), "resized down route")?;
                Self::input(h, [367.0, 190.0], 0x200)?;
            }
            25 => {
                Self::up(h, [367.0, 190.0])?;
            }
            26 => {
                if h.gesture_active() {
                    return Ok(());
                }
                Self::check(
                    (h.editor.document.objects[0].translation[0] + 33.0).abs() < 1e-3
                        && h.editor.stamp.revision == self.revision + 1,
                    "resized endpoint/dpi route",
                )?;
                self.records.push(json!({"case":"resize-recovery","window_px":h.input.window_size,"translation":h.editor.document.objects[0].translation}));
                self.done = true;
            }
            _ => return Err("unexpected native phase".into()),
        }
        self.advance(frame);
        Ok(())
    }
}
/// Establish reachability for the native fixture without changing normal windows.
#[cfg(windows)]
pub(super) fn place_window(window: &winit::window::Window) -> Result<(), String> {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn SystemParametersInfoW(action: u32, param: u32, data: *mut [i32; 4], flags: u32) -> i32;
    }
    let mut work = [0i32; 4];
    if unsafe { SystemParametersInfoW(48, 0, &mut work, 0) } == 0 {
        return Err("native fixture work area unavailable".into());
    }
    let size = window.outer_size();
    if i64::from(size.width) > i64::from(work[2]) - i64::from(work[0]) - 32
        || i64::from(size.height) > i64::from(work[3]) - i64::from(work[1]) - 32
    {
        return Err("native fixture window does not fit the desktop work area".into());
    }
    window.set_outer_position(winit::dpi::PhysicalPosition::new(
        work[0] + 16,
        work[1] + 16,
    ));
    Ok(())
}
#[cfg(not(windows))]
pub(super) fn place_window(_: &winit::window::Window) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
pub(super) fn post(message: u32, wparam: usize, point: (i32, i32)) -> Result<(), String> {
    #[repr(C)]
    struct NativePoint {
        x: i32,
        y: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn ClientToScreen(h: *mut std::ffi::c_void, point: *mut [i32; 2]) -> i32;
        fn SetCursorPos(x: i32, y: i32) -> i32;
        fn GetCursorPos(point: *mut [i32; 2]) -> i32;
        fn WindowFromPoint(point: NativePoint) -> *mut std::ffi::c_void;
        fn MapVirtualKeyW(code: u32, map_type: u32) -> u32;
        fn FindWindowW(class: *const u16, name: *const u16) -> *mut std::ffi::c_void;
        fn PostMessageW(h: *mut std::ffi::c_void, m: u32, w: usize, l: isize) -> i32;
        fn GetWindowThreadProcessId(h: *mut std::ffi::c_void, p: *mut u32) -> u32;
    }
    let title: Vec<_> = "AutoGraphics Authoring".encode_utf16().chain([0]).collect();
    let h = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
    if h.is_null() {
        return Err("native window missing".into());
    }
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(h, &mut pid) };
    if pid != std::process::id() {
        return Err("refuse another process's window".into());
    }
    // Keep the physical pointer consistent with posted moves. Otherwise a real
    // WM_MOUSEMOVE from cursor tracking can replace the drag sample with the old
    // physical position. Do not suppress that real event or the cancel path.
    if message == 0x200 {
        let mut screen = [point.0, point.1];
        if unsafe { ClientToScreen(h, &mut screen) } == 0 {
            return Err(format!(
                "native ClientToScreen failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        if unsafe { SetCursorPos(screen[0], screen[1]) } == 0 {
            return Err(format!(
                "native SetCursorPos failed at {screen:?}: {}",
                std::io::Error::last_os_error()
            ));
        }
        let mut actual = [0i32; 2];
        let mut hit_pid = 0;
        if unsafe { GetCursorPos(&mut actual) } == 0 || actual != screen {
            return Err(format!(
                "native fixture pointer unreachable: requested {screen:?}, actual {actual:?}"
            ));
        }
        unsafe {
            GetWindowThreadProcessId(
                WindowFromPoint(NativePoint {
                    x: actual[0],
                    y: actual[1],
                }),
                &mut hit_pid,
            )
        };
        if hit_pid != pid {
            return Err("native fixture pointer occluded by another process".into());
        }
    }
    let point = if message == 0x20a {
        let mut p = [point.0, point.1];
        if unsafe { ClientToScreen(h, &mut p) } == 0 {
            return Err("ClientToScreen failed".into());
        }
        (p[0], p[1])
    } else {
        point
    };
    let packed = if message == 0x100 || message == 0x101 {
        let scan = unsafe { MapVirtualKeyW(wparam as u32, 0) };
        let extended = if (0x25..=0x28).contains(&wparam) {
            1u32 << 24
        } else {
            0
        };
        (1 | (scan << 16) | extended | if message == 0x101 { 0xc0000000 } else { 0 }) as isize
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
pub(super) fn post(_: u32, _: usize, _: (i32, i32)) -> Result<(), String> {
    Err("native interaction probe requires Windows".into())
}
