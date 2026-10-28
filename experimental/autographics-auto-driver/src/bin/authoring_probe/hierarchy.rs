//! OS -> real Iced widgets/router -> actual Auto -> composed scene oracle.
use autographics_auto_driver::authoring::{host::Host, interaction};
use serde_json::{json, Value};
pub struct Probe {
    pub phase: u8,
    pub done: bool,
    pub error: Option<String>,
    pub records: Vec<Value>,
    at: u64,
    wrong_parent: bool,
    drop_release: bool,
    prepared: bool,
    pub preparation_delay_frames: u64,
    color: Option<[u8; 4]>,
    background: Option<[u8; 4]>,
}
impl Probe {
    pub fn new(wrong_parent: bool, drop_release: bool) -> Self {
        Self {
            phase: 0,
            done: false,
            error: None,
            records: vec![],
            at: 0,
            wrong_parent,
            drop_release,
            prepared: false,
            preparation_delay_frames: 0,
            color: None,
            background: None,
        }
    }
    pub fn step(&mut self, h: &mut Host, frame: u64, w: &winit::window::Window) {
        if self.done || self.error.is_some() || frame - self.at < 12 {
            return;
        }
        if frame - self.at > 240 {
            self.error = Some(format!("hierarchy timeout phase {}", self.phase));
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
                self.at = frame;
                return;
            }
        }
        if !h.editor.pending() && !h.gesture_active() {
            let widget = match self.phase {
                0 => Some("action-create-group2"),
                1 => Some("node-1"),
                2 => Some(if self.wrong_parent {
                    "parent-root"
                } else {
                    "parent-5"
                }),
                4 => Some("action-reparent"),
                5 => Some("node-5"),
                9 => Some("action-undo"),
                10 => Some("action-redo"),
                11 | 12 => Some("action-toggle-visible"),
                13 => Some("action-save"),
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
            Err(e) => self.error = Some(format!("hierarchy phase {}: {e}", self.phase)),
        }
    }
    fn check(ok: bool, why: &str) -> Result<(), String> {
        if ok {
            Ok(())
        } else {
            Err(why.into())
        }
    }
    /// Read the final composition independently of Auto state and GPU command inspection.
    fn pixels(
        &mut self,
        h: &mut Host,
        stage: &str,
        moved: bool,
        visible: bool,
    ) -> Result<(), String> {
        let color = self.color.ok_or("missing initial leaf pixel")?;
        let background = self.background.ok_or("missing initial background pixel")?;
        Self::check(
            color != background,
            "indistinguishable initial pixel controls",
        )?;
        let expected_origin = if visible && !moved { color } else { background };
        let expected_endpoint = if visible && moved { color } else { background };
        let origin = h.rt.probe_pixel(h.top, [345.0, 190.0]);
        let endpoint = h.rt.probe_pixel(h.top, [375.0, 170.0]);
        let passed = origin == Some(expected_origin) && endpoint == Some(expected_endpoint);
        self.records.push(json!({"case":"composed-pixels","stage":stage,
            "origin_point":[345,190],"endpoint_point":[375,170],
            "origin":origin,"endpoint":endpoint,"expected_origin":expected_origin,
            "expected_endpoint":expected_endpoint,"color":color,"background":background,"passed":passed}));
        Self::check(
            passed,
            &format!("{stage}: composed endpoint/origin pixels mismatch"),
        )
    }
    fn click(h: &mut Host, id: &str) -> Result<(), String> {
        let b = h
            .widget_bounds(id)
            .ok_or_else(|| format!("missing widget {id}"))?;
        if b.y < 0.0 || b.y + b.height > 800.0 {
            return Err(format!("widget outside viewport {id}: {}", b.y));
        }
        super::native_interaction::Probe::click(
            h,
            [810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5],
        )
    }
    fn run(&mut self, h: &mut Host) -> Result<bool, String> {
        match self.phase {
            0 => {
                self.color = h.rt.probe_pixel(h.top, [345.0, 190.0]);
                self.background = h.rt.probe_pixel(h.top, [375.0, 170.0]);
                Self::check(
                    self.color.is_some()
                        && self.background.is_some()
                        && self.color != self.background,
                    "initial composed pixel control",
                )?;
                Self::click(h, "action-create-group2")?;
            }
            1 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.groups.len() == 1 && h.editor.document.groups[0].id == 5,
                    "native create group",
                )?;
                Self::click(h, "node-1")?;
            }
            2 => {
                Self::check(h.editor.selection == Some(1), "tree leaf selection")?;
                Self::click(
                    h,
                    if self.wrong_parent {
                        "parent-root"
                    } else {
                        "parent-5"
                    },
                )?;
            }
            3 => {
                Self::check(
                    h.ui().parent_choice == if self.wrong_parent { None } else { Some(5) },
                    "parent choice widget",
                )?;
            }
            4 => {
                Self::click(h, "action-reparent")?;
            }
            5 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.objects[0].parent == Some(5)
                        && h.editor.preview.objects[0].translation == [-55.0, 0.0, 0.0],
                    "wrong parent / keep-world gate",
                )?;
                Self::click(h, "node-5")?;
            }
            6 => {
                Self::check(h.editor.selection == Some(5), "tree group selection")?;
                super::native_interaction::Probe::down(h, [400.0, 190.0])?;
            }
            7 => {
                Self::check(h.gesture_active(), "group origin handle")?;
                super::native_interaction::Probe::input(h, [430.0, 170.0], 0x200)?;
            }
            8 => {
                if h.interaction_snapshot()["shown_sample"]
                    .as_u64()
                    .unwrap_or(0)
                    == 0
                {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.groups[0].translation == [0.0; 3]
                        && h.editor.document.objects[0].translation == [-55.0, 0.0, 0.0],
                    "temporary changed accepted",
                )?;
                self.pixels(h, "temporary", true, true)?;
                if !self.drop_release {
                    super::native_interaction::Probe::up(h, [430.0, 170.0])?;
                }
            }
            9 => {
                if h.gesture_active() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.groups[0].translation == [30.0, 20.0, 0.0]
                        && h.editor.document.objects[0].translation == [-55.0, 0.0, 0.0]
                        && h.editor.history().0 == 3,
                    "group endpoint / single history",
                )?;
                Self::check(
                    h.editor.preview.objects[0].translation == [-25.0, 20.0, 0.0]
                        && interaction::pick2(&h.editor.preview.objects, [375.0, 170.0]) == Some(1)
                        && interaction::pick2(&h.editor.preview.objects, [345.0, 190.0]) != Some(1),
                    "world render/pick consumer",
                )?;
                // Actual GPU commands at the expected descendant center, alongside captured composition.
                let Some(agp::host::surface_runtime::SurfaceContent::Game2d(content)) =
                    h.rt.content(h.two)
                else {
                    return Err("2d consumer".into());
                };
                Self::check(
                    content
                        .commands
                        .iter()
                        .any(|c| c.center == [375.0, 170.0] && c.size == [50.0, 50.0]),
                    "GPU descendant command",
                )?;
                self.pixels(h, "move", true, true)?;
                let two =
                    h.rt.capture(h.two, "hierarchy-moved")
                        .ok_or("GPU capture")?
                        .0;
                self.records.push(json!({"case":"group-move","world":h.editor.preview.objects[0].translation,"history":h.editor.history(),"pixels":two,"local":h.editor.document.objects[0].translation,"color":self.color,"background":self.background}));
                Self::click(h, "action-undo")?;
            }
            10 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.groups[0].translation == [0.0; 3]
                        && h.editor.preview.objects[0].translation == [-55.0, 0.0, 0.0],
                    "undo whole group",
                )?;
                self.pixels(h, "undo", false, true)?;
                Self::click(h, "action-redo")?;
            }
            11 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    h.editor.document.groups[0].translation == [30.0, 20.0, 0.0],
                    "redo whole group",
                )?;
                self.pixels(h, "redo", true, true)?;
                Self::click(h, "action-toggle-visible")?;
            }
            12 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(
                    !h.editor.preview.objects[0].visible
                        && interaction::pick2(&h.editor.preview.objects, [375.0, 170.0]).is_none(),
                    "effective visibility / pick",
                )?;
                self.pixels(h, "hidden", true, false)?;
                Self::click(h, "action-toggle-visible")?;
            }
            13 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::check(h.editor.preview.objects[0].visible, "show descendants")?;
                self.pixels(h, "show", true, true)?;
                Self::click(h, "action-save")?;
            }
            14 => {
                Self::check(
                    !h.editor.dirty() && h.storage.path.is_some(),
                    "native hierarchy save",
                )?;
                self.records.push(json!({"case":"saved","path":h.storage.path,"hash":h.editor.document.hash()?,"document":h.editor.document,"world":h.editor.preview.objects,"resources":h.resource_snapshot()}));
                super::native_interaction::Probe::click(h, [10.0, 10.0])?;
            }
            15 => {
                Self::check(
                    h.editor.selection.is_none() && !h.editor.dirty(),
                    "saved unselected composition",
                )?;
                self.pixels(h, "saved", true, true)?;
                self.done = true;
            }
            _ => return Err("unexpected hierarchy phase".into()),
        }
        Ok(true)
    }
}
