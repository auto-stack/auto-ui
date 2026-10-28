//! PLAN-875 runtime camera native window probe: real posted winit mouse
//! events drive the LMB orbit drag, the wheel zoom, the leave/late-event
//! clearing and the fresh-press resume rule against the published camera;
//! Stop restores the editing view. Run stimulus only; Run start uses the
//! public Host path (the OS mouse path is the subject).
use super::*;
use autographics_auto_driver::authoring::runtime_camera::CameraDim;
use serde_json::{json, Value};

pub struct RuntimeCameraProbe {
    pub phase: u8,
    pub done: bool,
    pub error: Option<String>,
    pub records: Vec<Value>,
    at: u64,
    run_started: bool,
    drag_x: f32,
    drag_y: f32,
    drag_origin: Option<[f32; 3]>,
    yaw_before_resume: Option<f32>,
    zoom_before: Option<f32>,
}

impl RuntimeCameraProbe {
    pub fn new() -> Self {
        Self {
            phase: 0,
            done: false,
            error: None,
            records: vec![],
            at: 0,
            run_started: false,
            drag_x: 0.0,
            drag_y: 0.0,
            drag_origin: None,
            yaw_before_resume: None,
            zoom_before: None,
        }
    }

    pub fn step(&mut self, h: &mut Host, frame: u64, w: &winit::window::Window) {
        if self.done || self.error.is_some() || frame - self.at < 20 {
            return;
        }
        if frame - self.at > 600 {
            self.error = Some(format!("runtime-camera timeout phase {}", self.phase));
            return;
        }
        if let Some(p) = &h.play {
            if p.state == autographics_auto_driver::authoring::play::State::Faulted {
                self.error = Some(format!("runtime-camera run faulted: {:?}", p.error));
                return;
            }
        }
        if !w.has_focus() {
            w.focus_window();
            return;
        }
        match self.run(h) {
            Ok(true) => {
                self.phase += 1;
                self.at = frame;
            }
            Ok(false) => {}
            Err(e) => self.error = Some(format!("runtime-camera phase {}: {e}", self.phase)),
        }
    }

    fn check(ok: bool, why: &str) -> Result<(), String> {
        if ok {
            Ok(())
        } else {
            Err(why.into())
        }
    }

    fn camera(h: &Host) -> Result<(f32, f32, f32), String> {
        let cam = h
            .play
            .as_ref()
            .and_then(|p| p.camera_frame())
            .ok_or("no admitted camera frame")?;
        Ok((cam.yaw, cam.pitch, cam.distance))
    }

    fn actor(h: &Host) -> Result<[f32; 3], String> {
        Ok(h.play
            .as_ref()
            .and_then(|p| p.last.as_ref())
            .ok_or("no published frame")?
            .objects[0]
            .translation)
    }

    /// Posted points are LOGICAL 1200x800 coordinates; the physical window
    /// may scale them, so convert exactly like Probe::input does.
    fn physical(h: &Host, p: [f32; 2]) -> (i32, i32) {
        (
            (p[0] * h.input.window_size[0] / 1200.0) as i32,
            (p[1] * h.input.window_size[1] / 800.0) as i32,
        )
    }

    fn button(h: &Host, point: [f32; 2], down: bool) -> Result<(), String> {
        super::native_interaction::post(0x200, 0, Self::physical(h, point))?;
        super::native_interaction::post(
            if down { 0x201 } else { 0x202 },
            1,
            Self::physical(h, point),
        )
    }

    fn move_to(h: &Host, point: [f32; 2]) -> Result<(), String> {
        super::native_interaction::post(0x200, 0, Self::physical(h, point))
    }

    fn wheel(h: &Host, delta_notches: i16, point: [f32; 2]) -> Result<(), String> {
        // One wheel event of `delta_notches` notches: HIWORD(wparam) carries
        // +/-120 WHEEL_DELTA per notch.
        let raw = ((delta_notches * 120) as u16 as u32) << 16;
        super::native_interaction::post(0x20a, raw as usize, Self::physical(h, point))
    }

    fn key(code: usize, down: bool) -> Result<(), String> {
        super::native_interaction::post(if down { 0x100 } else { 0x101 }, code, (0, 0))
    }

    const DRAG_POINT: [f32; 2] = [400.0, 600.0];
    const WHEEL_POINT: [f32; 2] = [400.0, 600.0];

    fn run(&mut self, h: &mut Host) -> Result<bool, String> {
        match self.phase {
            0 => {
                if !self.run_started {
                    if h.play.is_none() {
                        h.start_selected_play()?;
                        return Ok(false);
                    }
                    self.run_started = true;
                }
                let p = h.play.as_ref().ok_or("missing play")?;
                if p.pending() {
                    return Ok(false);
                }
                if p.state != autographics_auto_driver::authoring::play::State::Running {
                    return Ok(false);
                }
                Self::check(
                    p.camera_dim() == Some(CameraDim::ThreeD),
                    "the run must use autographics.runtime_camera3d/1",
                )?;
                let (yaw, pitch, dist) = Self::camera(h)?;
                Self::check(yaw == 0.0 && pitch == 45.0 && dist == 8.0, "camera init")?;
                self.drag_origin = Some(Self::actor(h)?);
                // Real keyboard focus on the 3D surface for the W key later.
                super::native_interaction::Probe::click(h, [100.0, 590.0])?;
                self.records
                    .push(json!({"case":"started","yaw":yaw,"pitch":pitch,"distance":dist}));
                Ok(true)
            }
            1 => {
                // LMB press inside the 3D content area begins one capture.
                Self::button(h, Self::DRAG_POINT, true)?;
                self.drag_x = Self::DRAG_POINT[0];
                self.drag_y = Self::DRAG_POINT[1];
                self.records.push(json!({"case":"lmb-down"}));
                Ok(true)
            }
            2 => {
                // Real posted moves: +12 logical px right per step until the
                // frozen 0.25 deg/px rate accumulates ~+90 degrees.
                Self::move_to(h, [self.drag_x, self.drag_y])?;
                self.drag_x += 12.0;
                Self::move_to(h, [self.drag_x, self.drag_y])?;
                let (yaw, _, _) = Self::camera(h)?;
                if self.drag_x >= Self::DRAG_POINT[0] + 360.0 {
                    self.records.push(
                        json!({"case":"dragged","yaw":yaw,"px":self.drag_x - Self::DRAG_POINT[0]}),
                    );
                    return Ok(true);
                }
                Ok(false)
            }
            3 => {
                // Post the release; the messages settle on later cycles.
                Self::button(h, [self.drag_x, self.drag_y], false)?;
                Ok(true)
            }
            4 => {
                let (yaw, pitch, dist) = Self::camera(h)?;
                Self::check(
                    (yaw - 90.0).abs() < 8.0,
                    &format!("drag must orbit to yaw ~90, got {yaw}"),
                )?;
                Self::check(h.camera_drag.is_none(), "Up must clear the capture")?;
                self.records
                    .push(json!({"case":"lmb-up","yaw":yaw,"pitch":pitch,"distance":dist}));
                Ok(true)
            }
            5 => {
                // Forward key with the rotated basis: the sample moves along
                // -F(90) = +x. Wait for real rotated travel.
                Self::key(0x57, true)?; // 'W' down
                self.records.push(json!({"case":"w-down-yaw90"}));
                Ok(true)
            }
            6 => {
                let pos = Self::actor(h)?;
                let origin = self.drag_origin.unwrap();
                if pos[0] > origin[0] + 0.5 {
                    Self::key(0x57, false)?;
                    self.records
                        .push(json!({"case":"moved-rotated","dx":pos[0] - origin[0],"dz":pos[2] - origin[2]}));
                    Self::check(
                        (pos[2] - origin[2]).abs() < 1.0,
                        "rotated forward must not drift in z",
                    )?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            7 => {
                // Wheel down: the frozen 1.1^notches zoom out.
                Self::wheel(h, -1, Self::WHEEL_POINT)?;
                self.zoom_before = Some(Self::camera(h)?.2);
                Ok(true)
            }
            8 => {
                let (_, _, dist) = Self::camera(h)?;
                if dist > self.zoom_before.unwrap() + 0.05 {
                    Self::check(dist <= 12.0, "zoom must stay inside [4,12]")?;
                    self.records
                        .push(json!({"case":"zoomed-out","distance":dist}));
                    return Ok(true);
                }
                Self::wheel(h, -1, Self::WHEEL_POINT)?;
                Ok(false)
            }
            9 => {
                // Blur clears the capture and pauses; late moves stay inert.
                h.blur();
                Self::check(
                    h.play.as_ref().unwrap().state
                        == autographics_auto_driver::authoring::play::State::Paused,
                    "blur must pause the camera run",
                )?;
                Self::check(h.camera_drag.is_none(), "blur must clear the capture")?;
                let (yaw, _, _) = Self::camera(h)?;
                Self::move_to(h, [500.0, 650.0])?;
                Self::move_to(h, [520.0, 620.0])?;
                let (yaw_after, _, _) = Self::camera(h)?;
                Self::check((yaw_after - yaw).abs() < 1e-4, "late moves must not orbit")?;
                self.records
                    .push(json!({"case":"blur-late-inert","yaw":yaw_after}));
                Ok(true)
            }
            10 => {
                // Resume needs a NEW press: moves without a fresh LMB press
                // must not orbit.
                h.play.as_mut().unwrap().resume()?;
                self.yaw_before_resume = Some(Self::camera(h)?.0);
                Self::move_to(h, [410.0, 610.0])?;
                Self::move_to(h, [430.0, 630.0])?;
                Self::move_to(h, [450.0, 650.0])?;
                let (yaw, _, _) = Self::camera(h)?;
                Self::check(
                    (yaw - self.yaw_before_resume.unwrap()).abs() < 1e-4,
                    "resume without a new press must not orbit",
                )?;
                self.records
                    .push(json!({"case":"resume-needs-press","yaw":yaw}));
                Ok(true)
            }
            11 => {
                // A fresh press-drag-release orbits again.
                Self::button(h, [400.0, 600.0], true)?;
                Ok(true)
            }
            12 => {
                // Post the fresh-press drag; the fold lands on later cycles.
                Self::move_to(h, [440.0, 560.0])?;
                Self::button(h, [440.0, 560.0], false)?;
                Ok(true)
            }
            13 => {
                let (yaw, _, _) = Self::camera(h)?;
                Self::check(
                    (yaw - self.yaw_before_resume.unwrap()).abs() > 1.0,
                    "a fresh press-drag must orbit",
                )?;
                self.records
                    .push(json!({"case":"fresh-press-orbits","yaw":yaw}));
                Ok(true)
            }
            14 => {
                let run_pixels =
                    h.rt.capture(h.three, "runtime-camera-run")
                        .ok_or("missing capture")?
                        .0;
                h.stop_play();
                h.frame(None).ok();
                let restored =
                    h.rt.capture(h.three, "runtime-camera-restored")
                        .ok_or("missing capture")?
                        .0;
                self.records.push(json!({
                    "case":"stopped",
                    "restored_equals_run": restored == run_pixels,
                }));
                Self::check(
                    restored != run_pixels,
                    "Stop must restore the editing camera",
                )?;
                Ok(true)
            }
            15 => {
                self.done = true;
                self.records.push(json!({
                    "case":"completed",
                    "saturation": h.camera_saturation,
                }));
                Ok(true)
            }
            _ => {
                self.done = true;
                Ok(true)
            }
        }
    }
}
