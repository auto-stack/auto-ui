//! PLAN-874 character3d/1 native window probe: real posted winit keyboard
//! events drive the single router -> GameInput -> user Auto -> published
//! frame; pause/blur/stop oracles close the loop. Run stimulus only; Run
//! start uses the public Host API (the keyboard path is the subject).
use super::*;
use autographics_auto_driver::authoring::play::{CharProfile, State};
use serde_json::{json, Value};

pub struct Character3dProbe {
    pub phase: u8,
    pub done: bool,
    pub error: Option<String>,
    pub records: Vec<Value>,
    at: u64,
    run_started: bool,
    spawn: Option<[f32; 3]>,
    moved_z: Option<f32>,
    apex_y: Option<f32>,
}

impl Character3dProbe {
    pub fn new() -> Self {
        Self {
            phase: 0,
            done: false,
            error: None,
            records: vec![],
            at: 0,
            run_started: false,
            spawn: None,
            moved_z: None,
            apex_y: None,
        }
    }

    pub fn step(&mut self, h: &mut Host, frame: u64, w: &winit::window::Window) {
        if self.done || self.error.is_some() || frame - self.at < 20 {
            return;
        }
        if frame - self.at > 480 {
            self.error = Some(format!("character3d timeout phase {}", self.phase));
            return;
        }
        if let Some(p) = &h.play {
            if p.state == State::Faulted {
                self.error = Some(format!("character3d run faulted: {:?}", p.error));
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
            Err(e) => self.error = Some(format!("character3d phase {}: {e}", self.phase)),
        }
    }

    fn check(ok: bool, why: &str) -> Result<(), String> {
        if ok {
            Ok(())
        } else {
            Err(why.into())
        }
    }

    fn translation(h: &Host) -> Result<[f32; 3], String> {
        Ok(h.play
            .as_ref()
            .and_then(|p| p.last.as_ref())
            .ok_or("no published frame")?
            .objects[0]
            .translation)
    }

    fn click_widget(h: &mut Host, id: &str) -> Result<(), String> {
        let b = h
            .widget_bounds(id)
            .ok_or_else(|| format!("missing widget {id}"))?;
        Self::check(
            b.y >= 0.0 && b.y + b.height < 800.0,
            "character3d widget clipped",
        )?;
        super::native_interaction::Probe::click(
            h,
            [810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5],
        )
    }
    fn key(code: usize, down: bool) -> Result<(), String> {
        super::native_interaction::post(if down { 0x100 } else { 0x101 }, code, (0, 0))
    }

    fn run(&mut self, h: &mut Host) -> Result<bool, String> {
        match self.phase {
            0 => {
                // Start through the public Host path with the CLI-selected
                // API/source; the OS keyboard path is what this probe tests.
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
                if p.state == State::Paused {
                    // A legal init pause (resize/DPI/focus): recover through
                    // the same OS resume button, like the play probe does.
                    Self::click_widget(h, "action-play-resume")?;
                    return Ok(false);
                }
                if p.state != State::Running {
                    return Ok(false);
                }
                Self::check(
                    p.character_run(),
                    "the run must use autographics.character3d/1",
                )?;
                Self::check(
                    p.character_profile() == Some(CharProfile::ThreeD),
                    "the run must use autographics.character3d/1",
                )?;
                self.spawn = Some(Self::translation(h)?);
                // Give the 3D surface real keyboard focus.
                super::native_interaction::Probe::click(h, [100.0, 590.0])?;
                self.records.push(json!({"case":"started","spawn":self.spawn,"version":"autographics.character3d/1"}));
                Ok(true)
            }
            1 => {
                Self::key(0x57, true)?; // 'W' down through the real OS path
                self.records.push(json!({"case":"w-down"}));
                Ok(true)
            }
            2 => {
                let z = Self::translation(h)?[2];
                if z < self.spawn.unwrap()[2] - 0.5 {
                    Self::key(0x57, false)?;
                    self.moved_z = Some(z);
                    self.records.push(json!({"case":"moved-forward","z":z}));
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            3 => {
                Self::key(0x20, true)?; // Space tap -> jump edge
                Self::key(0x20, false)?;
                self.records.push(json!({"case":"space-tap"}));
                Ok(true)
            }
            4 => {
                let y = Self::translation(h)?[1];
                if y > self.spawn.unwrap()[1] + 0.5 {
                    self.apex_y = Some(y);
                    self.records.push(json!({"case":"jumped","apex_y":y}));
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            5 => {
                // Focus loss clears held state and edges (auto-Pause).
                h.blur();
                Self::check(
                    h.play.as_ref().unwrap().state == State::Paused,
                    "blur must pause the run",
                )?;
                self.records
                    .push(json!({"case":"blur-paused","z":Self::translation(h)?[2]}));
                Ok(true)
            }
            6 => {
                let p = h.play.as_mut().unwrap();
                if p.state != State::Paused {
                    return Ok(false);
                }
                p.resume()?;
                Ok(true)
            }
            7 => {
                // No key is down after the blur clear: the character holds.
                let z = Self::translation(h)?[2];
                self.records
                    .push(json!({"case":"blur-cleared","z_after":z}));
                Self::check(
                    (z - self.moved_z.unwrap()).abs() < 0.5,
                    "blur must clear held keys (no drift after resume)",
                )?;
                Ok(true)
            }
            8 => {
                let run_pixels =
                    h.rt.capture(h.three, "character3d-run")
                        .ok_or("missing capture")?
                        .0;
                h.stop_play();
                h.frame(None).ok();
                let restored =
                    h.rt.capture(h.three, "character3d-restored")
                        .ok_or("missing capture")?
                        .0;
                self.records.push(json!({
                    "case":"stopped",
                    "run_pixels_sha": run_pixels.len(),
                    "restored_sha": restored.len(),
                    "restored_equals_run": restored == run_pixels,
                }));
                Ok(true)
            }
            9 => {
                self.done = true;
                self.records.push(json!({
                    "case":"completed",
                    "spawn": self.spawn,
                    "moved_z": self.moved_z,
                    "apex_y": self.apex_y,
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
