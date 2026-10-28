//! Actual OS text/button/viewport route; independent model centroid color oracle.
use autographics_auto_driver::authoring::{
    document::{Kind, Object},
    host::Host,
    interaction,
};
use serde_json::{json, Value};
pub struct Probe {
    pub phase: u8,
    pub done: bool,
    pub error: Option<String>,
    pub records: Vec<Value>,
    at: u64,
    path: String,
    drop_release: bool,
    fault: Option<String>,
    prepared: bool,
    pub preparation_delay_frames: u64,
}
impl Probe {
    pub fn new(path: String, drop_release: bool, fault: Option<String>) -> Self {
        Self {
            phase: 0,
            done: false,
            error: None,
            records: vec![],
            at: 0,
            path,
            drop_release,
            fault,
            prepared: false,
            preparation_delay_frames: 0,
        }
    }
    pub fn step(&mut self, h: &mut Host, frame: u64, w: &winit::window::Window) {
        if self.done || self.error.is_some() || frame - self.at < 12 {
            return;
        }
        if frame - self.at > 240 {
            self.error = Some(format!("asset timeout phase {}", self.phase));
            return;
        }
        // Readiness must precede automatic scrolling as well as clicking.
        // A clipped phase0 widget must never bypass focus/pointer setup.
        if self.phase == 0 {
            if frame < self.preparation_delay_frames {
                return;
            }
            if !w.has_focus() {
                w.focus_window();
                return;
            }
            if !self.prepared {
                if let Err(error) = w.set_cursor_position(winit::dpi::PhysicalPosition::new(
                    400.0 * h.input.window_size[0] as f64 / 1200.0,
                    190.0 * h.input.window_size[1] as f64 / 800.0,
                )) {
                    self.error = Some(error.to_string());
                    return;
                }
                self.prepared = true;
                self.records
                    .push(json!({"case":"input-prepared","frame":frame}));
                return;
            }
        }
        if !h.editor.pending() && !h.gesture_active() {
            let widget = match self.phase {
                0 => Some("action-create-group3"),
                1 => Some("node-1"),
                2 => Some("asset-path"),
                4 => Some("action-import"),
                8 => Some("action-undo"),
                9 => Some("action-redo"),
                10 | 11 => Some("action-toggle-visible"),
                12 => Some("node-2"),
                13 => Some("action-delete-request"),
                14 => Some("action-create"),
                15 => Some("node-3"),
                16 => Some("action-asset-replace"),
                18 => Some("action-save"),
                _ => None,
            };
            if let Some(b) = widget.and_then(|id| h.widget_bounds(id)) {
                if b.y < 0.0 || b.y + b.height > 790.0 {
                    if let Err(e) = super::native_interaction::Probe::scroll(h, b.y >= 0.0) {
                        self.error = Some(e);
                    }
                    return;
                }
            }
        }
        match self.run(h) {
            Ok(true) => {
                self.phase += 1;
                self.at = frame;
            }
            Ok(false) => {}
            Err(e) => self.error = Some(format!("asset phase {}: {e}", self.phase)),
        }
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
        Self::check(
            b.y >= 0.0 && b.y + b.height < 800.0,
            &format!("widget clipped {id}: {}", b.y),
        )?;
        super::native_interaction::Probe::click(
            h,
            [810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5],
        )
    }
    fn points(h: &Host, x: f32) -> Vec<[f32; 2]> {
        [-1.5, 1.5]
            .into_iter()
            .map(|v| {
                let p = interaction::project(h.camera(), glam::Vec3::new(v + x, 0.5, 0.0)).unwrap();
                [p.x, p.y + 400.0]
            })
            .collect()
    }
    fn pixels(&mut self, h: &mut Host, stage: &str, x: Option<f32>) -> Result<(), String> {
        let mut checks = vec![];
        for pos in [0.0, 5.0] {
            for (i, p) in Self::points(h, pos).into_iter().enumerate() {
                let pixel = h.rt.probe_pixel(h.top, p).ok_or("missing GPU readback")?;
                let shown = x == Some(pos);
                let passed = if shown {
                    pixel[i] > 80 && pixel[1 - i] < 20 && pixel[2] < 20
                } else {
                    pixel[0] > 50 && pixel[1] > 50 && pixel[2] > 50
                };
                checks.push(json!({"position":pos,"part":i,"point":p,"pixel":pixel,"expected":if shown{if i==0{"red"}else{"green"}}else{"background"},"passed":passed}));
            }
        }
        let passed = checks.iter().all(|v| v["passed"] == true);
        self.records
            .push(json!({"case":"asset-pixels","stage":stage,"checks":checks,"passed":passed}));
        Self::check(passed, &format!("{stage}: actual asset part/origin pixels"))
    }
    fn drag(h: &Host) -> ([f32; 2], [f32; 2]) {
        let mut o = Object::new(1, Kind::Cylinder);
        o.translation = [0.0; 3];
        let (_, a, d) = interaction::axes(&o, h.camera())
            .into_iter()
            .find(|a| a.0 == 0)
            .unwrap();
        let start = a + d.normalize() * 32.0;
        (
            [start.x, start.y + 400.0],
            [start.x + d.x * 5.0, start.y + d.y * 5.0 + 400.0],
        )
    }
    fn run(&mut self, h: &mut Host) -> Result<bool, String> {
        if !h.ui().error.is_empty() {
            return Err(h.ui().error.clone());
        }
        match self.phase {
            0 => {
                Self::click(h, "action-create-group3")?;
            }
            1 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(h.editor.document.groups.len() == 1, "create 3D group")?;
                Self::click(h, "node-1")?;
            }
            2 => {
                Self::check(h.editor.selection == Some(1), "group selection")?;
                Self::click(h, "asset-path")?;
            }
            3 => {
                Self::check(
                    h.rt.iced_text_input_focused(
                        h.panel,
                        iced_wgpu::core::widget::Id::new("authoring-asset-path"),
                    ),
                    "asset path input must be focused",
                )?;
                for c in self.path.encode_utf16() {
                    super::native_interaction::post(0x100, 0xe7, (0, 0))?;
                    super::native_interaction::post(0x102, c as usize, (0, 0))?;
                    super::native_interaction::post(0x101, 0xe7, (0, 0))?;
                }
            }
            4 => {
                Self::check(
                    h.ui().import_path == self.path,
                    &format!("real OS text path input: observed {:?}", h.ui().import_path),
                )?;
                Self::click(h, "action-import")?;
            }
            5 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.objects.len() == 1
                        && h.editor.document.objects[0].kind == Kind::Model
                        && h.editor.document.objects[0].parent == Some(1),
                    "import/create/group single transaction",
                )?;
                self.pixels(h, "import", Some(0.0))?;
                h.asset_render_fault = self.fault.clone();
                let (start, _) = Self::drag(h);
                super::native_interaction::Probe::down(h, start)?;
            }
            6 => {
                Self::check(h.gesture_active(), "OS group handle down")?;
                let (_, end) = Self::drag(h);
                super::native_interaction::Probe::input(h, end, 0x200)?;
            }
            7 => {
                if h.editor.document.groups[0].translation[0] != 0.0 {
                    return Err("temporary changed accepted document".into());
                }
                if h.gesture_active() {
                    self.pixels(h, "temporary", Some(5.0))?;
                }
                let (_, end) = Self::drag(h);
                if !self.drop_release {
                    super::native_interaction::Probe::up(h, end)?;
                }
            }
            8 => {
                if h.gesture_active() || h.editor.pending() {
                    return Ok(false);
                }
                let (start, end) = Self::drag(h);
                let ratio = h.input.window_size[0] / 1200.0;
                let delta = ((end[0] * ratio) as i32 - (start[0] * ratio) as i32) as f32 / ratio;
                let mut origin = Object::new(1, Kind::Cylinder);
                origin.translation = [0.0; 3];
                let axis = interaction::axes(&origin, h.camera())
                    .into_iter()
                    .find(|a| a.0 == 0)
                    .unwrap()
                    .2;
                let expected = delta / axis.x;
                Self::check(
                    (h.editor.document.groups[0].translation[0] - expected).abs() < 1e-4,
                    "group move endpoint from quantized actual OS coordinates",
                )?;
                self.pixels(h, "move", Some(5.0))?;
                Self::click(h, "action-undo")?;
            }
            9 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                self.pixels(h, "undo", Some(0.0))?;
                Self::click(h, "action-redo")?;
            }
            10 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                self.pixels(h, "redo", Some(5.0))?;
                Self::click(h, "action-toggle-visible")?;
            }
            11 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                self.pixels(h, "hidden", None)?;
                Self::click(h, "action-toggle-visible")?;
            }
            12 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                self.pixels(h, "show", Some(5.0))?;
                Self::click(h, "node-2")?;
            }
            13 => {
                Self::check(h.editor.selection == Some(2), "imported leaf selection")?;
                Self::click(h, "action-delete-request")?;
            }
            14 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.objects.is_empty(),
                    "delete imported leaf preserves registry",
                )?;
                Self::click(h, "action-create")?;
            }
            15 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.objects.len() == 1
                        && h.editor.document.objects[0].id == 3
                        && h.editor.document.objects[0].kind == Kind::Mob,
                    "builtin 3D target",
                )?;
                Self::click(h, "node-3")?;
            }
            16 => {
                Self::check(h.editor.selection == Some(3), "replacement target")?;
                Self::click(h, "action-asset-replace")?;
            }
            17 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.objects[0].id == 3
                        && h.editor.document.objects[0].kind == Kind::Model
                        && h.editor.document.objects[0].parent.is_none()
                        && h.editor.document.objects[0].translation == [0.0; 3],
                    "resource replacement preserves stable ID/local/parent",
                )?;
                self.pixels(h, "replace", Some(0.0))?;
                super::native_interaction::Probe::click(h, [20.0, 20.0])?;
            }
            18 => {
                Self::check(
                    h.editor.selection.is_none(),
                    "deselect before persisted image",
                )?;
                Self::click(h, "action-save")?;
            }
            19 => {
                Self::check(
                    !h.editor.dirty() && h.storage.path.is_some() && !h.storage.unverified(),
                    "confirmed entry and complete asset closure",
                )?;
                self.pixels(h, "saved", Some(0.0))?;
                self.records.push(json!({"case":"saved","path":h.storage.path,"document":h.editor.document,"hash":h.editor.document.hash()?,"world":h.editor.preview.objects,"resources":h.resource_snapshot()}));
                self.done = true;
            }
            _ => return Err("unexpected asset phase".into()),
        }
        Ok(true)
    }
}
