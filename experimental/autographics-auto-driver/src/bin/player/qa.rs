//! PLAN-072 T-02 bounded proof driver: each real AG025 game closure plays
//! through the standalone player with real posted OS keyboard/mouse messages:
//! normal start -> real input -> real Gameplay tick -> visible GPU frame ->
//! actual device audio -> gameplay death -> fresh-Enter round restart ->
//! clean close. The 3D chain adds a left-drag orbit and a wheel zoom. Every
//! observation is read from the same Host the player presents from; the
//! script is deterministic against the frozen level contracts (2D fail at
//! y <= -3 in the x 8..10 gap; 3D fail at y <= -4 past the z 0 edge).
use super::native::{self, *};
use autographics_auto_driver::authoring::host::Host;
use autographics_auto_driver::authoring::play::{Play, State};
use serde_json::{json, Value};
use winit::window::Window;

pub enum Script {
    TwoD,
    ThreeD,
}

pub struct Qa {
    script: Script,
    title: String,
    phase: u8,
    at: u64,
    pub records: Vec<Value>,
    pub done: bool,
    pub error: Option<String>,
    spawn: [f32; 3],
    run1: i32,
    audio_t0: Value,
    died_y: Option<f32>,
    cam_before: Option<[f32; 3]>,
    zoom_before: Option<f32>,
}

impl Qa {
    pub fn new(script: Script, title: String) -> Self {
        Self {
            script,
            title,
            phase: 0,
            at: 0,
            records: vec![],
            done: false,
            error: None,
            spawn: [0.0; 3],
            run1: 0,
            audio_t0: Value::Null,
            died_y: None,
            cam_before: None,
            zoom_before: None,
        }
    }

    pub fn script_name(&self) -> &'static str {
        match self.script {
            Script::TwoD => "platformer-2d",
            Script::ThreeD => "platformer-3d",
        }
    }

    pub fn passed(&self) -> bool {
        self.done && self.error.is_none()
    }

    fn actor_translation(h: &Host, two_d: bool) -> Result<[f32; 3], String> {
        let binding = h
            .editor
            .document
            .behavior
            .as_ref()
            .ok_or_else(|| "qa: scene has no project behavior".to_string())?;
        let id = if two_d {
            binding.actors.two_d
        } else {
            binding.actors.three_d
        }
        .ok_or_else(|| "qa: scene has no actor for this dimension".to_string())?;
        let frame = h
            .play
            .as_ref()
            .and_then(|p| p.last.as_ref())
            .ok_or_else(|| "qa: no published frame".to_string())?;
        let object = frame
            .objects
            .iter()
            .find(|o| o.id == id)
            .ok_or_else(|| "qa: actor missing from published frame".to_string())?;
        Ok(object.translation)
    }

    fn audio(h: &Host) -> Value {
        let s = h.audio_snapshot();
        json!({"manager":s["manager"],"device":s["device"],"rate":s["rate"],"callbacks":s["callbacks"],"nonzero":s["nonzero"],"peak":s["peak"],"run":s["run"]})
    }

    fn play(h: &Host) -> Result<&Play, String> {
        h.play.as_ref().ok_or_else(|| "qa: no run".to_string())
    }

    /// Frame budget per phase; every wait phase expires by name.
    fn gate(&mut self, h: &Host, frame: u64, settle: u64, limit: u64) -> Result<bool, String> {
        if self.done || self.error.is_some() {
            return Ok(false);
        }
        if let Some(p) = &h.play {
            if p.state == State::Faulted {
                self.error = Some(format!(
                    "qa: run faulted at phase {}: {:?}",
                    self.phase, p.error
                ));
                return Ok(false);
            }
        }
        let since = frame - self.at;
        if since < settle {
            return Ok(false);
        }
        if since > limit {
            return Err(format!(
                "qa timeout phase {} (play state {:?}, tick {}, pending {}, error {:?})",
                self.phase,
                h.play.as_ref().map(|p| p.state),
                h.play.as_ref().map(|p| p.tick()).unwrap_or(0),
                h.play.as_ref().map(|p| p.pending()).unwrap_or(false),
                h.play.as_ref().and_then(|p| p.error.clone()),
            ));
        }
        Ok(true)
    }

    fn send(&self, code: usize, down: bool) -> Result<(), String> {
        let message = if down { WM_KEYDOWN } else { WM_KEYUP };
        native::post(&self.title, message, code, (0, 0))
    }

    fn next(&mut self, frame: u64) {
        self.phase += 1;
        self.at = frame;
    }

    pub fn step(&mut self, h: &mut Host, w: &Window, frame: u64) {
        if self.done || self.error.is_some() {
            return;
        }
        if frame > 7200 {
            self.error = Some("qa global timeout".into());
            return;
        }
        let two_d = matches!(self.script, Script::TwoD);
        let limit = if matches!(self.phase, 0 | 3 | 6) { 1800 } else { 400 };
        if let Err(e) = self.gate(h, frame, 8, limit) {
            self.error = Some(e);
            return;
        }
        if self.phase == 0 && !w.has_focus() {
            // Posted messages reach this window's queue regardless of
            // foreground state; keep asking for focus but never gate on it.
            w.focus_window();
        }
        let result: Result<(), String> = (|| {
            match self.phase {
                // 0: the normal start is already Playing; freeze the spawn.
                0 => {
                    let p = Self::play(h)?;
                    if p.state != State::Running || p.pending() || p.last.is_none() {
                        return Ok(());
                    }
                    self.spawn = Self::actor_translation(h, two_d)?;
                    self.run1 = p.run_id();
                    self.audio_t0 = Self::audio(h);
                    self.records.push(
                        json!({"case":"started","spawn":self.spawn,"run":self.run1,"audio":self.audio_t0}),
                    );
                    self.send(if two_d { VK_SPACE } else { VK_KEY_W }, true)?;
                    self.next(frame);
                    Ok(())
                }
                // 1: 2D releases the jump edge; 3D taps jump mid-walk.
                1 => {
                    if two_d {
                        self.send(VK_SPACE, false)?;
                        self.records.push(json!({"case":"space-tap"}));
                    } else {
                        self.send(VK_SPACE, true)?;
                    }
                    self.next(frame);
                    Ok(())
                }
                2 => {
                    if two_d {
                        let y = Self::actor_translation(h, two_d)?[1];
                        if y <= self.spawn[1] + 0.4 {
                            return Ok(());
                        }
                        self.records.push(
                            json!({"case":"jumped","y":y,"audio_after":Self::audio(h)}),
                        );
                        self.send(VK_KEY_D, true)?;
                    } else {
                        self.send(VK_SPACE, false)?;
                        self.records.push(json!({"case":"space-tap"}));
                    }
                    self.next(frame);
                    Ok(())
                }
                // 3: hold movement until the run is unrecoverable. The 2D hero
                // crosses y=-1.5 inside the x 8..10 gap; the 3D player walks
                // off the z=0 edge and is caught below y=-3.4 (fail -4).
                3 => {
                    let (release_code, line) = if two_d {
                        (VK_KEY_D, -1.5)
                    } else {
                        (VK_KEY_W, -3.4)
                    };
                    let y = Self::actor_translation(h, two_d)?[1];
                    if y > line {
                        return Ok(());
                    }
                    self.send(release_code, false)?;
                    self.died_y = Some(y);
                    self.records
                        .push(json!({"case":"fell-below-fail-line","y":y}));
                    self.next(frame);
                    Ok(())
                }
                // 4: let the defeat cue reach the device, then fresh-Enter.
                4 => {
                    if frame - self.at < 60 {
                        return Ok(());
                    }
                    self.records.push(
                        json!({"case":"audio-after-defeat","audio":Self::audio(h),"audio_at_start":self.audio_t0}),
                    );
                    self.send(VK_RETURN, true)?;
                    self.next(frame);
                    Ok(())
                }
                5 => {
                    self.send(VK_RETURN, false)?;
                    self.next(frame);
                    Ok(())
                }
                // 6: the fresh round is a new Run back at the original spawn.
                6 => {
                    let p = Self::play(h)?;
                    if p.state != State::Running || p.pending() || p.last.is_none() {
                        return Ok(());
                    }
                    let run = p.run_id();
                    if run <= self.run1 {
                        return Ok(());
                    }
                    let respawn = Self::actor_translation(h, two_d)?;
                    let drift = (0..3)
                        .map(|i| (respawn[i] - self.spawn[i]).abs())
                        .fold(0.0f32, f32::max);
                    if drift > 0.7 {
                        return Err(format!("qa respawn drifted {drift} from spawn"));
                    }
                    self.records.push(
                        json!({"case":"respawned","old_run":self.run1,"new_run":run,"respawn":respawn,"drift":drift}),
                    );
                    match self.script {
                        Script::TwoD => {
                            native::post(&self.title, WM_CLOSE, 0, (0, 0))?;
                            self.records.push(json!({"case":"close-requested"}));
                            self.done = true;
                        }
                        Script::ThreeD => {
                            let cam =
                                p.camera_frame().ok_or_else(|| "qa: 3D run has no camera".to_string())?;
                            self.cam_before = Some([cam.yaw, cam.pitch, cam.distance]);
                            let size = w.inner_size();
                            let (cx, cy) = (size.width as i32 / 2, size.height as i32 / 2);
                            native::post(&self.title, WM_LBUTTONDOWN, 1, (cx, cy))?;
                            self.records.push(json!({"case":"orbit-press","at":[cx, cy]}));
                        }
                    }
                    self.next(frame);
                    Ok(())
                }
                // 7..10 (3D): left-drag orbit, then a wheel zoom notch.
                7 => {
                    let size = w.inner_size();
                    let (cx, cy) = (size.width as i32 / 2, size.height as i32 / 2);
                    native::post(&self.title, WM_MOUSEMOVE, 0, (cx + 90, cy + 24))?;
                    self.next(frame);
                    Ok(())
                }
                8 => {
                    let size = w.inner_size();
                    let (cx, cy) = (size.width as i32 / 2, size.height as i32 / 2);
                    native::post(&self.title, WM_LBUTTONUP, 0, (cx, cy))?;
                    self.next(frame);
                    Ok(())
                }
                9 => {
                    let p = Self::play(h)?;
                    if p.pending() {
                        return Ok(());
                    }
                    let cam = p
                        .camera_frame()
                        .ok_or_else(|| "qa: 3D run has no camera".to_string())?;
                    let before = self
                        .cam_before
                        .ok_or_else(|| "qa: missing camera baseline".to_string())?;
                    self.records.push(
                        json!({"case":"orbit-applied","before":before,"after":[cam.yaw, cam.pitch, cam.distance]}),
                    );
                    if (cam.yaw - before[0]).abs() < 0.5 && (cam.pitch - before[1]).abs() < 0.5 {
                        return Err("qa: left-drag orbit changed neither yaw nor pitch".into());
                    }
                    self.zoom_before = Some(cam.distance);
                    let size = w.inner_size();
                    let (cx, cy) = (size.width as i32 / 2, size.height as i32 / 2);
                    native::post(&self.title, WM_MOUSEWHEEL, 120 << 16, (cx, cy))?;
                    self.next(frame);
                    Ok(())
                }
                10 => {
                    let p = Self::play(h)?;
                    if p.pending() {
                        return Ok(());
                    }
                    let cam = p
                        .camera_frame()
                        .ok_or_else(|| "qa: 3D run has no camera".to_string())?;
                    let before = self
                        .zoom_before
                        .ok_or_else(|| "qa: missing zoom baseline".to_string())?;
                    self.records.push(
                        json!({"case":"zoom-applied","before":before,"after":cam.distance}),
                    );
                    if (cam.distance - before).abs() < 0.05 {
                        return Err("qa: wheel notch changed no camera distance".into());
                    }
                    native::post(&self.title, WM_CLOSE, 0, (0, 0))?;
                    self.records.push(json!({"case":"close-requested"}));
                    self.done = true;
                    Ok(())
                }
                _ => {
                    self.done = true;
                    Ok(())
                }
            }
        })();
        if let Err(e) = result {
            self.error = Some(format!("qa phase {}: {e}", self.phase));
        }
    }
}
