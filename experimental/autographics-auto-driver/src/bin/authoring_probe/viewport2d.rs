//! Actual OS -> winit -> Host -> Auto -> composite readback oracle.
use super::native_interaction::{post, Probe as Input};
use autographics_auto_driver::authoring::{document::Document, host::Host};
use serde_json::{json, Value};
#[derive(Default)]
pub struct Probe {
    pub phase: u8,
    at: u64,
    pub done: bool,
    pub error: Option<String>,
    pub records: Vec<Value>,
    before: Option<Document>,
    off_pixel: Option<[u8; 4]>,
    start: [f32; 2],
}
impl Probe {
    pub fn step(&mut self, h: &mut Host, frame: u64, w: &winit::window::Window) {
        if self.done || self.error.is_some() || frame - self.at < 12 {
            return;
        }
        if frame - self.at > 300 {
            self.error = Some(format!("viewport2d phase {} timeout", self.phase));
            return;
        }
        match self.run(h, w) {
            Ok(true) => {
                self.records.push(json!({"phase":self.phase,"frame":frame,"view":format!("{:?}",h.presented_view2d()),"document_hash":h.editor.document.hash().ok(),"history":h.editor.history(),"dirty":h.editor.dirty(),"selection":h.editor.selection,"resources":h.resource_snapshot()}));
                self.phase += 1;
                self.at = frame;
            }
            Ok(false) => {}
            Err(e) => self.error = Some(format!("viewport2d phase {}: {e}", self.phase)),
        }
    }
    fn require(ok: bool, why: &str) -> Result<(), String> {
        if ok {
            Ok(())
        } else {
            Err(why.into())
        }
    }
    fn button(h: &mut Host, id: &str) -> Result<bool, String> {
        let b = h.widget_bounds(id).ok_or(format!("missing {id}"))?;
        if b.y < 0.0 || b.y + b.height > 790.0 {
            Input::scroll(h, b.y > 750.0)?;
            return Ok(false);
        }
        Input::click(h, [810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5])?;
        Ok(true)
    }
    fn run(&mut self, h: &mut Host, w: &winit::window::Window) -> Result<bool, String> {
        match self.phase {
            0 => {
                if !w.has_focus() {
                    w.focus_window();
                    return Ok(false);
                }
                self.before = Some(h.editor.document.clone());
                self.off_pixel = Some(
                    h.rt.probe_pixel(h.top, [19.0, 30.0])
                        .ok_or("off readback")?,
                );
                if !Self::button(h, "action-view-grid")? {
                    return Ok(false);
                }
            }
            1 => {
                Self::require(h.view2d().grid, "Grid OS button did not enable")?;
                let line =
                    h.rt.probe_pixel(h.top, [19.0, 30.0])
                        .ok_or("grid readback")?;
                let gap =
                    h.rt.probe_pixel(h.top, [25.0, 35.0])
                        .ok_or("gap readback")?;
                Self::require(
                    Some(line) != self.off_pixel && line != gap,
                    "grid composite pixels missing",
                )?;
                self.records.push(json!({"case":"grid-composite","line":line,"gap":gap,"off":self.off_pixel,"lines":h.view2d().grid_lines().1.len()}));
                Input::input(h, [300.0, 190.0], 0x200)?;
                Input::input(h, [300.0, 190.0], 0x207)?;
            }
            2 => {
                Input::input(h, [340.0, 210.0], 0x200)?;
            }
            3 => {
                Input::input(h, [340.0, 210.0], 0x208)?;
            }
            4 => {
                Self::require(
                    (h.view2d().center - glam::Vec2::new(-40.0, 20.0)).length() < 0.01,
                    "MMB world pan mismatch",
                )?;
                Self::require(
                    h.editor.document == *self.before.as_ref().unwrap()
                        && h.editor.history().0 == 0,
                    "navigation altered content/history",
                )?;
                // A physical wheel lParam is screen coordinates; CompositionInput converts only once.
                Input::input(h, [300.0, 150.0], 0x200)?;
                let point = (
                    (300.0 * h.input.window_size[0] / 1200.0) as i32,
                    (150.0 * h.input.window_size[1] / 800.0) as i32,
                );
                post(0x20a, (120usize) << 16, point)?;
            }
            5 => {
                Self::require((h.view2d().zoom - 1.1).abs() < 0.001, "wheel zoom mismatch")?;
                let anchor = h
                    .view2d()
                    .world(glam::Vec2::new(300.0, 150.0))
                    .ok_or("anchor")?;
                Self::require(
                    (anchor - glam::Vec2::new(-140.0, 60.0)).length() < 1.0,
                    "wheel anchor moved",
                )?;
                // Expected transform assembled independently from the navigation contract.
                self.start = [
                    400.0 + (-55.0 - h.view2d().center.x) * 1.1,
                    190.0 + h.view2d().center.y * 1.1,
                ];
                Input::down(h, self.start)?;
            }
            6 => {
                Self::require(h.editor.selection == Some(1), "transformed OS pick target")?;
                Input::input(h, [self.start[0] + 22.0, self.start[1] - 11.0], 0x200)?;
            }
            7 => {
                Input::up(h, [self.start[0] + 22.0, self.start[1] - 11.0])?;
            }
            8 => {
                if h.gesture_active() {
                    return Ok(false);
                }
                let t = h.editor.document.objects[0].translation;
                Self::require(
                    (t[0] + 35.0).abs() < 1.1
                        && (t[1] - 10.0).abs() < 1.1
                        && h.editor.history().0 == 1,
                    "Auto world drag/history mismatch",
                )?;
                let p = h
                    .presented_view2d()
                    .screen(glam::Vec2::new(t[0], t[1]))
                    .ok_or("object screen")?;
                let pixel =
                    h.rt.probe_pixel(h.top, p.to_array())
                        .ok_or("object readback")?;
                Self::require(
                    pixel[0] > 100 && pixel[0] > pixel[1].saturating_add(30),
                    "transformed composite object missing",
                )?;
                self.records
                    .push(json!({"case":"auto-drag-composite","world":t,"pixel":pixel}));
                if !Self::button(h, "action-undo")? {
                    return Ok(false);
                }
            }
            9 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::require(
                    h.editor.document == *self.before.as_ref().unwrap(),
                    "Undo restore content",
                )?;
                if !Self::button(h, "action-view-reset")? {
                    return Ok(false);
                }
            }
            10 => {
                Self::require(
                    h.view2d().center == glam::Vec2::ZERO
                        && h.view2d().zoom == 1.0
                        && !h.view2d().grid,
                    "reset via OS",
                )?;
                Self::require(
                    h.editor.document == *self.before.as_ref().unwrap(),
                    "reset altered content",
                )?;
                Input::click(h, [345.0, 190.0])?;
            }
            11 => {
                post(0x100, b'F' as usize, (0, 0))?;
                post(0x102, b'f' as usize, (0, 0))?;
                post(0x101, b'F' as usize, (0, 0))?;
            }
            12 => {
                Self::require(
                    (h.view2d().center - glam::Vec2::new(-55.0, 0.0)).length() < 0.01
                        && (h.view2d().zoom - 6.08).abs() < 0.01,
                    "focused F shortcut mapping",
                )?;
                Input::input(h, [300.0, 190.0], 0x200)?;
                Input::input(h, [300.0, 190.0], 0x207)?;
            }
            13 => {
                Input::input(h, [340.0, 210.0], 0x200)?;
            }
            14 => {
                post(0x100, 0x1b, (0, 0))?;
                post(0x101, 0x1b, (0, 0))?;
            }
            15 => {
                Self::require(
                    (h.view2d().center - glam::Vec2::new(-55.0, 0.0)).length() < 0.01,
                    "Esc navigation start restoration",
                )?;
                if !Self::button(h, "authoring-scene-path")? {
                    return Ok(false);
                }
            }
            16 => {
                post(0x100, 0x24, (0, 0))?;
                post(0x101, 0x24, (0, 0))?;
                post(0x100, b'F' as usize, (0, 0))?;
                post(0x102, b'f' as usize, (0, 0))?;
                post(0x101, b'F' as usize, (0, 0))?;
            }
            17 => {
                Self::require(
                    (h.view2d().center - glam::Vec2::new(-55.0, 0.0)).length() < 0.01
                        && (h.view2d().zoom - 6.08).abs() < 0.01,
                    "text focus must isolate F/Home",
                )?;
                Input::click(h, [400.0, 190.0])?;
            }
            18 => {
                post(0x100, 0x24, (0, 0))?;
                post(0x101, 0x24, (0, 0))?;
            }
            19 => {
                Self::require(
                    h.view2d().center == glam::Vec2::ZERO && h.view2d().zoom == 1.0,
                    "Home reset focus shortcut",
                )?;
                Self::require(
                    h.editor.document == *self.before.as_ref().unwrap(),
                    "shortcuts modified content",
                )?;
                self.done = true;
            }
            _ => return Err("unexpected viewport phase".into()),
        }
        Ok(true)
    }
}
