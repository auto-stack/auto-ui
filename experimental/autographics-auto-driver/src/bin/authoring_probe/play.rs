//! OS -> real Iced/router -> retained Auto -> actual top-composition pixel oracle.
use autographics_auto_driver::authoring::{
    assets,
    document::{Dimension, Document, Group, Kind, Object},
    host::Host,
    interaction,
    play::State,
};
use serde_json::{json, Value};
// The top texture readback floors the requested point. Predict coverage at
// that pixel's center, using the independent two-triangle fixture geometry.
fn patrol_origin_part(
    camera: glam::Mat4,
    point: [f32; 2],
    three: f32,
) -> Result<Option<usize>, String> {
    let sample = glam::Vec2::new(point[0].floor() + 0.5, point[1].floor() + 0.5 - 400.0);
    for (part, center) in [-1.5f32, 1.5].into_iter().enumerate() {
        let mut vertices = Vec::with_capacity(3);
        for (x, y) in [(-0.7, 0.0), (0.7, 0.0), (0.0, 1.5)] {
            vertices.push(
                interaction::project(camera, glam::Vec3::new(center + three + x, y, 0.0))
                    .ok_or("project independent triangle")?,
            );
        }
        let edges = [
            (vertices[1] - vertices[0]).perp_dot(sample - vertices[0]),
            (vertices[2] - vertices[1]).perp_dot(sample - vertices[1]),
            (vertices[0] - vertices[2]).perp_dot(sample - vertices[2]),
        ];
        if edges.iter().all(|v| *v > 0.0) || edges.iter().all(|v| *v < 0.0) {
            return Ok(Some(part));
        }
    }
    Ok(None)
}
pub fn fixture(path: &str) -> Result<Host, String> {
    let asset = assets::import(std::path::Path::new(path), &Default::default())?;
    let mut d = Document::empty("play-native".into());
    let mut group = Group::new(10, Dimension::Three);
    group.translation = [1.0, 0.0, 0.0];
    d.groups.push(group);
    let mut o = Object::new(11, Kind::Model);
    o.parent = Some(10);
    o.translation = [-1.0, 0.0, 0.0];
    o.resource = Some(asset.identity.key());
    d.objects.push(o);
    let mut o = Object::new(12, Kind::Quad);
    o.translation = [-100.0, 0.0, 0.0];
    o.scale = [8.0, 8.0, 1.0];
    o.rgba = [0.1, 0.8, 0.9, 1.0];
    d.objects.push(o);
    d.assets.push(asset.identity.clone());
    Host::with_assets(d, assets::Registry::from([(asset.identity.key(), asset)]))
}
pub struct Probe {
    pub project_bind: bool,
    pub step_delay_frames: u64,
    pub drop_step: bool,
    pub double_step: bool,
    step_click_sent: bool,
    second_step_sent: bool,
    step_observation: Option<(State, i32, bool)>,
    project_stage: u8,
    project_before: Option<Value>,
    project_after: Option<Value>,
    pub behavior_reference: Option<String>,
    pub phase: u8,
    pub done: bool,
    pub error: Option<String>,
    pub records: Vec<Value>,
    at: u64,
    path: String,
    wrong_focus: bool,
    gpu_fault: Option<String>,
    before: Option<Value>,
    images: Option<[String; 2]>,
    tick: i32,
    run: i32,
    two_x: f32,
    three_x: f32,
    recovering_clock_pause: bool,
    pending_v: Option<(i32, u8)>,
    repost_v: Option<u8>,
}
impl Probe {
    pub fn new(
        path: String,
        wrong_focus: bool,
        gpu_fault: Option<String>,
        _drop_tick: bool,
    ) -> Self {
        Self {
            project_bind: false,
            step_delay_frames: 0,
            drop_step: false,
            double_step: false,
            step_click_sent: false,
            second_step_sent: false,
            step_observation: None,
            project_stage: 0,
            project_before: None,
            project_after: None,
            behavior_reference: None,
            phase: 0,
            done: false,
            error: None,
            records: vec![],
            at: 0,
            path,
            wrong_focus,
            gpu_fault,
            before: None,
            images: None,
            tick: 0,
            run: 0,
            two_x: 0.0,
            three_x: 0.0,
            recovering_clock_pause: false,
            pending_v: None,
            repost_v: None,
        }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn step(&mut self, h: &mut Host, frame: u64, w: &winit::window::Window) {
        let delay = if self.phase == 4 {
            36
        } else if self.phase == 10 {
            120
        } else {
            12
        };
        if self.done || self.error.is_some() || frame - self.at < delay {
            return;
        }
        if frame - self.at > 360 {
            self.error = Some(format!("play timeout phase {}", self.phase));
            return;
        }
        if let Some(p) = &h.play {
            if p.state == State::Faulted {
                self.error = Some(format!("play fault: {:?}", p.error));
                return;
            }
        }
        // A legal clock pause clears input. Recover through the same OS button,
        // before sending more stimulus; never restart this phase's deadline.
        if self.live_input_phase() {
            if let Some(p) = &h.play {
                if p.state == State::Paused && clock_pause(p.error.as_deref()) {
                    self.records.push(json!({"case":"clock-pause-recovery","phase":self.phase,"tick":p.tick(),"reason":p.error}));
                    self.recovering_clock_pause = true;
                    self.repost_v = self
                        .pending_v
                        .and_then(|(_, edges)| (!self.v_accepted(h)).then_some(edges));
                    if let Err(e) = Self::click(h, "action-play-resume") {
                        self.error = Some(e);
                    }
                    return;
                }
                if self.recovering_clock_pause {
                    if p.state != State::Running {
                        return;
                    }
                    let tick = p.tick();
                    let result = (|| {
                        if matches!(self.phase, 3 | 4 | 9 | 10 | 16 | 17 | 20 | 21) {
                            // Focus/input may have been cleared by clicking Resume.
                            // Use the original intended surface, including wrong-focus.
                            super::native_interaction::Probe::click(
                                h,
                                if self.wrong_focus {
                                    [1000.0, 20.0]
                                } else if matches!(self.phase, 9 | 10) {
                                    [100.0, 600.0]
                                } else {
                                    [100.0, 190.0]
                                },
                            )?;
                            if matches!(self.phase, 4 | 10) {
                                Self::key(0x27, true)?;
                            }
                        }
                        if self.phase == 13 {
                            Self::click(h, "authoring-scene-path")?;
                        }
                        if let Some(edges) = self.repost_v.take() {
                            // An accepted tick must never receive this batch twice.
                            super::native_interaction::Probe::click(h, [100.0, 190.0])?;
                            for _ in 0..edges {
                                Self::key(0x56, true)?;
                                Self::key(0x56, false)?;
                            }
                            self.records.push(json!({"case":"reposted-unaccepted-v","phase":self.phase,"tick":tick,"edges":edges}));
                            self.pending_v = Some((tick, edges));
                        }
                        Ok::<_, String>(())
                    })();
                    self.recovering_clock_pause = false;
                    if let Err(e) = result {
                        self.error = Some(e);
                    }
                    return;
                }
            }
        }
        match self.run(h, w, frame) {
            Ok(true) => {
                self.phase += 1;
                self.at = frame;
            }
            Ok(false) => {}
            Err(e) => self.error = Some(format!("play phase {}: {e}", self.phase)),
        }
    }
    fn live_input_phase(&self) -> bool {
        matches!(self.phase, 2..=4 | 8..=10 | 12..=17 | 19..=21)
    }
    fn v_accepted(&self, h: &Host) -> bool {
        self.pending_v.is_none_or(|(tick, edges)| {
            h.play.as_ref().is_some_and(|p| {
                p.reports
                    .iter()
                    .any(|t| t.tick > tick && (edges == 2 || t.toggle))
            })
        })
    }
    fn check(ok: bool, why: &str) -> Result<(), String> {
        if ok {
            Ok(())
        } else {
            Err(why.into())
        }
    }
    fn click(h: &mut Host, id: &str) -> Result<(), String> {
        let b = h
            .widget_bounds(id)
            .ok_or_else(|| format!("missing widget {id}"))?;
        Self::check(b.y >= 0.0 && b.y + b.height < 800.0, "play widget clipped")?;
        super::native_interaction::Probe::click(
            h,
            [810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5],
        )
    }
    fn key(code: usize, down: bool) -> Result<(), String> {
        super::native_interaction::post(if down { 0x100 } else { 0x101 }, code, (0, 0))?;
        if down && code == 0x56 {
            super::native_interaction::post(0x102, 0x76, (0, 0))?;
        }
        Ok(())
    }
    fn images(h: &mut Host) -> Result<[String; 2], String> {
        Ok([
            h.rt.capture(h.two, "play2").ok_or("missing 2D capture")?.0,
            h.rt.capture(h.three, "play3")
                .ok_or("missing 3D capture")?
                .0,
        ])
    }
    fn pixels(&mut self, h: &mut Host, stage: &str, two: f32, three: f32) -> Result<(), String> {
        let patrol = self.behavior_reference.as_deref() == Some("patrol");
        let (two, three) = if patrol && h.play.is_some() {
            self.trace(h)?;
            (self.two_x, self.three_x)
        } else {
            (two, three)
        };
        let two_y = if patrol && h.play.is_some() {
            three
        } else {
            0.0
        };
        let mut checks = vec![];
        // Small cyan quad's endpoint and protected original location.
        for (name, dx, shown) in [("two-end", two, true), ("two-origin", 0.0, two.abs() < 1.0)] {
            let point = [
                300.0 + dx,
                190.0 - if name == "two-end" { two_y } else { 0.0 },
            ];
            let pixel =
                h.rt.probe_pixel(h.top, point)
                    .ok_or("top pixel unavailable")?;
            let pass = if shown {
                pixel[0].abs_diff(89) <= 4
                    && pixel[1].abs_diff(231) <= 4
                    && pixel[2].abs_diff(243) <= 4
            } else {
                pixel[0].abs_diff(39) <= 4
                    && pixel[1].abs_diff(48) <= 4
                    && pixel[2].abs_diff(56) <= 4
            };
            checks.push(json!({"point":point,"pixel":pixel,"case":name,"passed":pass}));
        }
        for (name, x, shown) in [
            ("three-end", three, true),
            ("three-origin", 0.0, three.abs() < 0.01),
        ] {
            for (part, dx) in [-1.5, 1.5].into_iter().enumerate() {
                let point = interaction::project(h.camera(), glam::Vec3::new(dx + x, 0.5, 0.0))
                    .ok_or("project model centroid")?;
                let point = [point.x, point.y + 400.0];
                let pixel =
                    h.rt.probe_pixel(h.top, point)
                        .ok_or("top part pixel unavailable")?;
                // Patrol can move one triangle over the other triangle's old
                // witness. Compute its expected color from the independent
                // triangle geometry, rather than assuming every origin is empty.
                let expected_part = if patrol && name == "three-origin" {
                    patrol_origin_part(h.camera(), point, three)?
                } else if shown {
                    Some(part)
                } else {
                    None
                };
                let pass = if let Some(expected) = expected_part {
                    pixel[expected] > 80 && pixel[1 - expected] < 20 && pixel[2] < 20
                } else {
                    pixel[0] > 50 && pixel[1] > 50 && pixel[2] > 50
                };
                checks.push(
                    json!({"point":point,"pixel":pixel,"case":name,"part":part,"expected_part":expected_part,"passed":pass}),
                );
            }
        }
        let passed = checks.iter().all(|v| v["passed"] == true);
        self.records.push(json!({"case":"play-pixels","stage":stage,"two":two,"three":three,"checks":checks,"passed":passed}));
        Self::check(passed, "actual play endpoint/origin/all-part pixels")
    }
    fn trace(&mut self, h: &Host) -> Result<(), String> {
        let p = h.play.as_ref().ok_or("missing play")?;
        let mut two = 0.0f32;
        let mut three = 0.0f32;
        let mut n2 = 0;
        let mut n3 = 0;
        for (i, t) in p.reports.iter().enumerate() {
            Self::check(t.tick == i as i32, "accepted tick missing/repeated")?;
            if t.space == 2 {
                two += t.x as f32
                    * if self.behavior_reference.as_deref() == Some("move-fast") {
                        90.0
                    } else {
                        60.0
                    }
                    * (1.0 / 60.0);
                if t.x != 0 {
                    n2 += 1;
                }
            }
            if t.space == 3 {
                if self.behavior_reference.as_deref() != Some("patrol") {
                    three += t.x as f32 * 3.0 * (1.0 / 60.0);
                }
                if t.x != 0 {
                    n3 += 1;
                }
            }
            if self.behavior_reference.as_deref() == Some("patrol") && i > 0 {
                three += if ((i - 1) / 160) % 2 == 0 { 1.0 } else { -1.0 } * 3.0 * (1.0 / 60.0);
            }
        }
        let f = p.last.as_ref().ok_or("no complete frame")?;
        Self::check(
            (f.document.objects[1].translation[0] - (-100.0 + two)).abs() < 1e-3
                && (f.objects[0].translation[0] - three).abs() < 1e-3
                && (self.behavior_reference.as_deref() != Some("patrol")
                    || (f.objects[1].translation[1] - three).abs() < 1e-3),
            "Auto motion differs from independent f32 action reference",
        )?;
        Self::check(n2 >= 5, "wrong-focus: no real 2D actions")?;
        if self.phase >= 11 {
            Self::check(n3 >= 90, "wrong-focus: no real 3D actions")?;
        }
        self.two_x = two;
        self.three_x = three;
        self.records.push(json!({"case":"tick-trace","phase":self.phase,"tick":p.tick(),"run":p.run_id,"source":p.source_hash,"behavior":p.user_source.as_ref().map(|s|json!({"path":s.path,"hash":s.user_hash,"api":s.api_hash,"bridge":s.bridge_hash,"version":s.version})),"reference":self.behavior_reference,"compile_us":p.compile_us,"two_actions":n2,"three_actions":n3,"trace":p.reports}));
        Ok(())
    }
    fn run(&mut self, h: &mut Host, w: &winit::window::Window, frame: u64) -> Result<bool, String> {
        if self.project_bind && self.project_stage < 5 {
            if !w.has_focus() {
                w.focus_window();
                return Ok(false);
            }
            if h.editor.pending() {
                return Ok(false);
            }
            if !h.ui().error.is_empty() {
                return Err(h.ui().error.clone());
            }
            match self.project_stage {
                0 => {
                    self.project_before = Some(json!(h.editor.document.behavior));
                    Self::click(h, "action-behavior-bind")?;
                    self.project_stage = 1;
                }
                1 => {
                    Self::check(
                        h.editor.code.is_some() && h.editor.dirty(),
                        "OS bind must accept changed payload",
                    )?;
                    self.project_after = Some(json!(h.editor.document.behavior));
                    Self::click(h, "action-undo")?;
                    self.project_stage = 2;
                }
                2 => {
                    Self::check(
                        Some(json!(h.editor.document.behavior)) == self.project_before,
                        "OS Undo restores previous binding",
                    )?;
                    Self::click(h, "action-redo")?;
                    self.project_stage = 3;
                }
                3 => {
                    Self::check(
                        Some(json!(h.editor.document.behavior)) == self.project_after,
                        "OS Redo restores accepted source",
                    )?;
                    let args: Vec<_> = std::env::args().collect();
                    if let Some(target) = args
                        .iter()
                        .position(|a| a == "--project-saveas")
                        .and_then(|i| args.get(i + 1))
                    {
                        h.ui_mut().path = target.clone();
                        h.rt.graph.mark_content_dirty(h.panel);
                        Self::click(h, "action-saveas")?;
                    } else {
                        Self::click(h, "action-save")?;
                    }
                    self.project_stage = 4;
                }
                4 => {
                    Self::check(
                        !h.editor.dirty() && !h.storage.unverified(),
                        "OS binding Save must confirm complete closure",
                    )?;
                    self.records.push(json!({"case":"project-bound-saved","document":h.editor.document,"path":h.storage.path,"project_default":h.project_default,"undo_redo":true}));
                    self.project_stage = 5;
                }
                _ => unreachable!(),
            }
            return Ok(false);
        }
        match self.phase {
            0 => {
                if !w.has_focus() {
                    w.focus_window();
                    return Ok(false);
                }
                w.set_cursor_position(winit::dpi::PhysicalPosition::new(
                    400.0 * h.input.window_size[0] as f64 / 1200.0,
                    190.0 * h.input.window_size[1] as f64 / 800.0,
                ))
                .map_err(|e| e.to_string())?;
            }
            1 => {
                self.before = Some(h.editor.isolation_identity());
                self.images = Some(Self::images(h)?);
                self.pixels(h, "editing", 0.0, 0.0)?;
                Self::click(h, "action-play-run")?;
            }
            2 => {
                let Some(p) = &h.play else {
                    if !h.ui().error.is_empty() {
                        return Err(h.ui().error.clone());
                    }
                    return Ok(false);
                };
                if p.state == State::Starting {
                    return Ok(false);
                }
                Self::check(p.state == State::Running, "Run must reach Running")?;
                self.run = p.run_id;
                h.asset_render_fault = self.gpu_fault.clone();
                super::native_interaction::Probe::click(
                    h,
                    if self.wrong_focus {
                        [1000.0, 20.0]
                    } else {
                        [100.0, 190.0]
                    },
                )?;
            }
            3 => {
                Self::key(0x27, true)?;
            }
            4 => {
                if !self.wrong_focus
                    && h.play
                        .as_ref()
                        .unwrap()
                        .reports
                        .iter()
                        .filter(|t| t.space == 2 && t.x != 0)
                        .count()
                        < 5
                {
                    return Ok(false);
                }
                Self::key(0x27, false)?;
                Self::click(h, "action-play-pause")?;
            }
            5 => {
                let p = h.play.as_ref().ok_or("play start missing")?;
                if p.state == State::Pausing {
                    return Ok(false);
                }
                Self::check(p.state == State::Paused, "Pause must settle")?;
                self.tick = p.tick();
                self.trace(h)?;
                self.pixels(h, "2d-moved", self.two_x, 0.0)?;
                let b = h.widget_bounds("action-play-step").ok_or("step bounds")?;
                self.records.push(json!({"case":"step-request","tick":self.tick,"bounds":[b.x,b.y,b.width,b.height],"ui_state":h.ui().play_state,"ui_error":h.ui().error}));
                if !self.drop_step && self.step_delay_frames == 0 {
                    Self::click(h, "action-play-step")?;
                    self.step_click_sent = true;
                }
            }
            6 => {
                if !self.step_click_sent
                    && !self.drop_step
                    && frame - self.at >= self.step_delay_frames
                {
                    Self::click(h, "action-play-step")?;
                    self.step_click_sent = true;
                    return Ok(false);
                }
                let p = h.play.as_ref().unwrap();
                let observed = (p.state, p.tick(), p.pending());
                if self.step_observation != Some(observed) {
                    self.step_observation = Some(observed);
                    self.records.push(json!({"case":"step-observed","tick":p.tick(),"expected":self.tick+1,"state":format!("{:?}",p.state),"pending":p.pending(),"ui_state":h.ui().play_state,"ui_error":h.ui().error}));
                }
                Self::check(
                    p.state == State::Paused && p.tick() <= self.tick + 1,
                    "Single Step must advance exactly one actual Auto tick",
                )?;
                // A posted click is asynchronous. Await its actual accepted tick,
                // never dispatch a replacement or reset the original phase deadline.
                if p.pending() || p.tick() == self.tick {
                    return Ok(false);
                }
                if self.double_step && !self.second_step_sent {
                    Self::click(h, "action-play-step")?;
                    self.second_step_sent = true;
                    return Ok(false);
                }
                Self::check(
                    p.tick() == self.tick + 1,
                    "Single Step must advance exactly one actual Auto tick",
                )?;
                self.tick = p.tick();
                self.records.push(json!({"case":"step","tick":self.tick}));
            }
            7 => {
                Self::check(
                    h.play.as_ref().unwrap().tick() == self.tick,
                    "paused tick advanced",
                )?;
                Self::click(h, "action-play-resume")?;
            }
            8 => {
                super::native_interaction::Probe::click(h, [100.0, 600.0])?;
            }
            9 => {
                Self::key(0x27, true)?;
            }
            10 => {
                if h.play
                    .as_ref()
                    .unwrap()
                    .reports
                    .iter()
                    .filter(|t| t.space == 3 && t.x != 0)
                    .count()
                    < 90
                {
                    return Ok(false);
                }
                Self::key(0x27, false)?;
                Self::click(h, "action-play-pause")?;
            }
            11 => {
                if h.play.as_ref().unwrap().state == State::Pausing {
                    return Ok(false);
                }
                Self::check(h.play.as_ref().unwrap().state == State::Paused, "3D Pause")?;
                self.trace(h)?;
                self.pixels(h, "3d-moved", self.two_x, self.three_x)?;
                Self::check(
                    h.editor.isolation_identity() == *self.before.as_ref().unwrap(),
                    "accepted edit state changed",
                )?;
                Self::click(h, "action-play-resume")?;
            }
            12 => {
                Self::check(
                    h.play.as_ref().unwrap().state == State::Running,
                    "Resume must take effect before panel keys",
                )?;
                Self::click(h, "authoring-scene-path")?;
            }
            13 => {
                Self::check(
                    h.rt.iced_text_input_focused(
                        h.panel,
                        iced_wgpu::core::widget::Id::new("authoring-scene-path"),
                    ),
                    "real Iced path input focus",
                )?;
                Self::key(0x25, true)?;
                Self::key(0x25, false)?;
                Self::key(0x56, true)?;
                Self::key(0x56, false)?;
            }
            14 => {
                self.trace(h)?;
                self.pixels(h, "ui-isolated", self.two_x, self.three_x)?;
                Self::check(
                    h.play
                        .as_ref()
                        .unwrap()
                        .last
                        .as_ref()
                        .unwrap()
                        .document
                        .objects[1]
                        .visible,
                    "panel V leaked to actor",
                )?;
                super::native_interaction::Probe::click(h, [100.0, 190.0])?;
            }
            15 => {
                let p = h.play.as_ref().unwrap();
                if p.state == State::Paused {
                    self.records
                        .push(json!({"case":"readback-auto-pause","reason":p.error}));
                    Self::click(h, "action-play-resume")?;
                    return Ok(false);
                }
                if p.state == State::Pausing {
                    return Ok(false);
                }
                Self::check(p.state == State::Running, "V parity requires Running")?;
                if h.input.keyboard_target() != Some(h.two) {
                    super::native_interaction::Probe::click(h, [100.0, 190.0])?;
                    return Ok(false);
                }
                Self::key(0x56, true)?;
                Self::key(0x56, false)?;
                Self::key(0x56, true)?;
                Self::key(0x56, false)?;
                self.pending_v = Some((p.tick(), 2));
            }
            16 => {
                if !self.v_accepted(h) {
                    return Ok(false);
                }
                Self::check(
                    h.play
                        .as_ref()
                        .unwrap()
                        .last
                        .as_ref()
                        .unwrap()
                        .document
                        .objects[1]
                        .visible,
                    "two V edges must cancel",
                )?;
                Self::key(0x56, true)?;
                Self::key(0x56, false)?;
                self.pending_v = Some((h.play.as_ref().unwrap().tick(), 1));
            }
            17 => {
                if !self.v_accepted(h) {
                    return Ok(false);
                }
                Self::check(
                    !h.play
                        .as_ref()
                        .unwrap()
                        .last
                        .as_ref()
                        .unwrap()
                        .document
                        .objects[1]
                        .visible,
                    "single V must hide Auto actor",
                )?;
                self.pending_v = None;
                Self::click(h, "action-play-pause")?;
            }
            18 => {
                if h.play.as_ref().unwrap().state == State::Pausing {
                    return Ok(false);
                }
                Self::check(
                    h.play.as_ref().unwrap().state == State::Paused,
                    "pause hidden before GPU readback",
                )?;
                if self.behavior_reference.as_deref() == Some("patrol") {
                    self.trace(h)?;
                }
                let point = [
                    300.0 + self.two_x,
                    190.0
                        - if self.behavior_reference.as_deref() == Some("patrol") {
                            self.three_x
                        } else {
                            0.0
                        },
                ];
                let pixel = h.rt.probe_pixel(h.top, point).ok_or("hidden readback")?;
                Self::check(
                    pixel[1] < 100 && pixel[2] < 100,
                    "hidden actor must disappear from actual top pixels",
                )?;
                self.records
                    .push(json!({"case":"v-hidden","point":point,"pixel":pixel}));
                Self::click(h, "action-play-resume")?;
            }
            19 => {
                super::native_interaction::Probe::click(h, [100.0, 190.0])?;
            }
            20 => {
                Self::check(
                    h.play.as_ref().unwrap().state == State::Running,
                    "restoring V requires Running",
                )?;
                Self::key(0x56, true)?;
                Self::key(0x56, false)?;
                self.pending_v = Some((h.play.as_ref().unwrap().tick(), 1));
            }
            21 => {
                if !self.v_accepted(h) {
                    return Ok(false);
                }
                Self::check(
                    h.play
                        .as_ref()
                        .unwrap()
                        .last
                        .as_ref()
                        .unwrap()
                        .document
                        .objects[1]
                        .visible,
                    "V must restore Auto actor",
                )?;
                self.pending_v = None;
                self.pixels(h, "shown", self.two_x, self.three_x)?;
                Self::key(0x27, true)?;
                super::native_interaction::post(0x8, 0, (0, 0))?;
            }
            22 => {
                let p = h.play.as_ref().unwrap();
                Self::check(p.state == State::Paused, "OS focus loss must auto-Pause")?;
                // Observe the focus-loss pause first, then reacquire the real
                // window before posting the Stop click. Never click through
                // another process or replace Stop with a Host command.
                w.focus_window();
                Self::key(0x27, false)?;
                self.records
                    .push(json!({"case":"focus-paused","tick":p.tick(),"reason":p.error}));
                Self::click(h, "action-play-stop")?;
            }
            23 => {
                Self::check(h.play.is_none(), "Stop must discard runtime")?;
                Self::check(
                    Self::images(h)? == *self.images.as_ref().unwrap(),
                    "Stop original edit canvases",
                )?;
                self.pixels(h, "stopped", 0.0, 0.0)?;
                self.records
                    .push(json!({"case":"stopped","identity":h.editor.isolation_identity()}));
                Self::click(h, "action-play-run")?;
            }
            24 => {
                let Some(p) = &h.play else {
                    return Ok(false);
                };
                if p.state == State::Starting {
                    return Ok(false);
                }
                Self::check(p.run_id != self.run, "new Run identity")?;
                Self::check(
                    p.initial_document_hash.as_ref() == Some(&h.editor.document.hash()?),
                    "new Run starts from original accepted scene",
                )?;
                Self::click(h, "action-play-stop")?;
            }
            25 => {
                Self::check(h.play.is_none(), "second Stop")?;
                Self::click(h, "action-save")?;
            }
            26 => {
                Self::check(
                    h.storage.path.is_some() && !h.editor.dirty(),
                    "Save after Stop must confirm",
                )?;
                self.records.push(json!({"case":"saved","document":h.editor.document,"path":h.storage.path,"two":h.rt.capture(h.two,"saved2").ok_or("saved2")?.0,"three":h.rt.capture(h.three,"saved3").ok_or("saved3")?.0}));
                self.done = true;
            }
            _ => return Err("unexpected play probe phase".into()),
        }
        Ok(true)
    }
}

fn clock_pause(error: Option<&str>) -> bool {
    matches!(
        error,
        Some("auto-Pause: tick backlog >4" | "auto-Pause: display gap >250ms")
    )
}

#[cfg(test)]
mod pixel_sample_tests {
    use super::*;
    use autographics_auto_driver::authoring::session::Edit;

    #[test]
    fn actual_gpu_floor_sample_boundary_clear_and_opposite_part() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("research/asset-probe/two-parts.glb");
        let mut h = fixture(path.to_str().unwrap()).unwrap();
        h.editor.select(Some(11)).unwrap();
        let mut probe = Probe::new(path.to_string_lossy().into(), false, None, false);
        probe.behavior_reference = Some("patrol".into());
        // Recorded F-855-R2-01 input and legitimate nearby control, plus the
        // opposite part/direction. Each candidate travels through real Auto.
        for (three, dx, expected) in [
            (2.4999988f32, 1.5, Some(0)),
            (2.349999f32, 1.5, None),
            (-2.6f32, -1.5, Some(1)),
            (-2.35f32, -1.5, None),
        ] {
            let mut object = h.editor.document.objects[0].clone();
            object.translation[0] = -1.0 + three;
            h.editor
                .begin(h.editor.stamp, Edit::Replace(object))
                .unwrap();
            h.settle().unwrap();
            h.frame(None).unwrap();
            let p = interaction::project(h.camera(), glam::Vec3::new(dx, 0.5, 0.0)).unwrap();
            let point = [p.x, p.y + 400.0];
            assert_eq!(
                patrol_origin_part(h.camera(), point, three).unwrap(),
                expected
            );
            // The same floored pixel must have the same prediction even
            // when the caller's fractional coordinate changes.
            for fraction in [0.1, 0.9] {
                let same_pixel = [point[0].floor() + fraction, point[1].floor() + fraction];
                assert_eq!(
                    patrol_origin_part(h.camera(), same_pixel, three).unwrap(),
                    expected
                );
            }
            // Exercise the actual probe consumer too: reverting just its call
            // site to the former ideal-point formula must fail this case.
            probe
                .pixels(&mut h, "sample-regression", 0.0, three)
                .unwrap();
            let pixel = h.rt.probe_pixel(h.top, point).unwrap();
            if let Some(part) = expected {
                assert!(
                    pixel[part] > 80 && pixel[1 - part] < 20 && pixel[2] < 20,
                    "actual fixture part at sampled origin: {point:?} {three} {pixel:?}"
                );
            } else {
                assert!(
                    pixel[0] > 50 && pixel[1] > 50 && pixel[2] > 50,
                    "legitimately clear sampled origin: {point:?} {three} {pixel:?}"
                );
            }
        }
    }
}
