//! PLAN-857 native OS -> winit -> Host -> Auto -> composed pixel oracle.
use super::native_interaction::{post, Probe as Input};
use autographics_auto_driver::authoring::{
    document::Document,
    host::Host,
    interaction::{self, AxisDrag},
    view3d::Mode,
};
use glam::Vec3;
use serde_json::{json, Value};

#[derive(Default)]
pub struct Probe {
    pub phase: u8,
    at: u64,
    pub done: bool,
    pub error: Option<String>,
    pub records: Vec<Value>,
    before: Option<Document>,
    samples: Vec<([f32; 2], [u8; 4])>,
    occlusion: Vec<([f32; 2], [u8; 4])>,
    sky: [u8; 4],
    before_view: Option<autographics_auto_driver::authoring::view3d::View3d>,
    start: [f32; 2],
    end: [f32; 2],
    expected: f32,
    save: Option<std::path::PathBuf>,
    snap_requested: bool,
}
impl Probe {
    pub fn step(&mut self, h: &mut Host, frame: u64, w: &winit::window::Window) {
        if self.done || self.error.is_some() || frame - self.at < 12 {
            return;
        }
        if frame - self.at > 360 {
            self.error = Some(format!("viewport3d phase {} timeout", self.phase));
            return;
        }
        match self.run(h, w) {
            Ok(true) => {
                self.records.push(json!({"phase":self.phase,"frame":frame,"view":format!("{:?}",h.presented_view3d()),"document_hash":h.editor.document.hash().ok(),"history":h.editor.history(),"selection":h.editor.selection,"resources":h.resource_snapshot()}));
                self.phase += 1;
                self.at = frame;
            }
            Ok(false) => {}
            Err(e) => self.error = Some(format!("viewport3d phase {}: {e}", self.phase)),
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
    fn key(code: usize) -> Result<(), String> {
        post(0x100, code, (0, 0))?;
        if (b'A' as usize..=b'Z' as usize).contains(&code) {
            post(0x102, code + 32, (0, 0))?;
        }
        post(0x101, code, (0, 0))?;
        Ok(())
    }
    fn run(&mut self, h: &mut Host, w: &winit::window::Window) -> Result<bool, String> {
        match self.phase {
            0 => {
                if !w.has_focus() {
                    w.focus_window();
                    return Ok(false);
                }
                self.before = Some(h.editor.document.clone());
                self.samples.clear();
                self.occlusion.clear();
                let mut view = h.view3d();
                view.grid = true;
                let (_, lines) = view.grid_vertices()?;
                let vp = h.camera();
                let mut grid_points = Vec::new();
                for pair in lines.chunks_exact(2) {
                    for t in [0.25f32, 0.5, 0.75] {
                        let a = Vec3::from(pair[0].position);
                        let b = Vec3::from(pair[1].position);
                        let world = a + (b - a) * t;
                        if let Some(p) = interaction::project(vp, world) {
                            if p.x > 5.0 && p.x < 795.0 && p.y > 5.0 && p.y < 395.0 {
                                for dy in -1..=1 {
                                    let q = [p.x, p.y + 400.0 + dy as f32];
                                    grid_points.push(q);
                                }
                                if grid_points.len() >= 45 {
                                    break;
                                }
                            }
                        }
                    }
                    if grid_points.len() >= 45 {
                        break;
                    }
                }
                let grid_off =
                    h.rt.probe_pixels(h.top, &grid_points)
                        .ok_or("missing grid baseline readback")?;
                self.samples = grid_points.into_iter().zip(grid_off).collect();
                Self::require(
                    self.samples.len() >= 6,
                    "insufficient visible grid sample positions",
                )?;
                let mut points = vec![[5.0, 405.0]];
                for i in -12..=12 {
                    let x = -1.5 + i as f32 * 0.1;
                    let p = interaction::project(vp, Vec3::new(x, 0.0, 0.0))
                        .ok_or("ground axis outside camera")?;
                    for dy in -1..=1 {
                        points.push([p.x, p.y + 400.0 + dy as f32]);
                    }
                }
                let off =
                    h.rt.probe_pixels(h.top, &points)
                        .ok_or("missing occlusion baseline")?;
                self.sky = off[0];
                self.occlusion = points
                    .into_iter()
                    .skip(1)
                    .zip(off.into_iter().skip(1))
                    .collect();
                if !Self::button(h, "action-view3d-grid")? {
                    return Ok(false);
                }
            }
            1 => {
                Self::require(h.view3d().grid, "3D grid button did not enable")?;
                let grid_points: Vec<_> = self.samples.iter().map(|(p, _)| *p).collect();
                let grid_on =
                    h.rt.probe_pixels(h.top, &grid_points)
                        .ok_or("missing grid readback")?;
                let mut changed = 0;
                let mut examples = Vec::new();
                for (i, (point, old)) in self.samples.iter().enumerate() {
                    let p = grid_on[i];
                    if p != *old {
                        changed += 1;
                        if examples.len() < 4 {
                            examples.push(json!({"point":point,"off":old,"on":p}));
                        }
                    }
                }
                Self::require(changed > 0, "3D grid composite pixels missing")?;
                self.records.push(json!({"case":"grid-composite","changed":changed,"samples":self.samples.len(),"examples":examples}));
                let points: Vec<_> = self.occlusion.iter().map(|(p, _)| *p).collect();
                let on =
                    h.rt.probe_pixels(h.top, &points)
                        .ok_or("missing occlusion readback")?;
                let saved = match h.rt.content_mut(h.three) {
                    Some(agp::host::surface_runtime::SurfaceContent::Scene3d(scene)) => {
                        std::mem::take(&mut scene.instances)
                    }
                    _ => return Err("missing occlusion scene".into()),
                };
                h.rt.graph.mark_content_dirty(h.three);
                let removed = (|| -> Result<(Vec<[u8; 4]>, Vec<[u8; 4]>), String> {
                    h.rt.frame(None, &Default::default())
                        .map_err(|e| format!("occlusion frame: {e:?}"))?;
                    let grid_on =
                        h.rt.probe_pixels(h.top, &points)
                            .ok_or("missing model-removed grid-on readback")?;
                    h.rt.set_editor_grid(h.three, None)?;
                    h.rt.frame(None, &Default::default())
                        .map_err(|e| format!("occlusion frame: {e:?}"))?;
                    let grid_off =
                        h.rt.probe_pixels(h.top, &points)
                            .ok_or("missing model-removed grid-off readback")?;
                    Ok((grid_on, grid_off))
                })();
                match h.rt.content_mut(h.three) {
                    Some(agp::host::surface_runtime::SurfaceContent::Scene3d(scene)) => {
                        scene.instances = saved
                    }
                    _ => return Err("missing scene while restoring occlusion fixture".into()),
                }
                let (_, grid) = h.view3d().grid_vertices()?;
                h.rt.set_editor_grid(h.three, Some(&grid))?;
                h.rt.graph.mark_content_dirty(h.three);
                h.rt.frame(None, &Default::default())
                    .map_err(|e| format!("occlusion frame: {e:?}"))?;
                let (removed_on, removed_off) = removed?;
                let mut hidden = Vec::new();
                for (i, (point, off)) in self.occlusion.iter().enumerate() {
                    if *off != self.sky && *off == on[i] && removed_on[i] != removed_off[i] {
                        hidden.push(json!({"point":point,"with_model_grid_off":off,"with_model_grid_on":on[i],"without_model_grid_on":removed_on[i],"without_model_grid_off":removed_off[i]}));
                    }
                }
                Self::require(
                    !hidden.is_empty(),
                    "grid-occlusion: foreground did not hide real grid",
                )?;
                self.records.push(json!({"case":"grid-occlusion","hidden":hidden.len(),"examples":hidden.iter().take(4).collect::<Vec<_>>()}));
                self.before_view = Some(h.view3d());
                Input::input(h, [400.0, 600.0], 0x200)?;
                Input::input(h, [400.0, 600.0], 0x207)?;
            }
            2 => {
                Input::input(h, [440.0, 620.0], 0x200)?;
            }
            3 => {
                Input::input(h, [440.0, 620.0], 0x208)?;
            }
            4 => {
                let old = self.before_view.unwrap();
                let camera = old.camera()?;
                let f = (camera.target - camera.eye).normalize();
                let right = f.cross(camera.up).normalize();
                let up = right.cross(f).normalize();
                let expected =
                    old.target + (-right * 40.0 + up * 20.0) * (old.visible_height() / 400.0);
                Self::require(
                    (h.view3d().target - expected).length() < 0.05,
                    "MMB camera pan mismatch",
                )?;
                Self::require(
                    h.editor.document == *self.before.as_ref().unwrap()
                        && h.editor.history().0 == 0,
                    "navigation changed document",
                )?;
                Self::key(b'P' as usize)?;
            }
            5 => {
                Self::require(
                    h.view3d().mode == Mode::Perspective,
                    "P did not toggle perspective",
                )?;
                self.before_view = Some(h.view3d());
                Input::input(h, [400.0, 600.0], 0x200)?;
                Input::input(h, [400.0, 600.0], 0x204)?;
            }
            6 => {
                Input::input(h, [430.0, 610.0], 0x200)?;
            }
            7 => {
                Input::input(h, [430.0, 610.0], 0x205)?;
            }
            8 => {
                let old = self.before_view.unwrap();
                Self::require(
                    (h.view3d().yaw - (old.yaw - 30.0 / 800.0 * std::f32::consts::TAU)).abs()
                        < 0.01,
                    "RMB orbit mismatch",
                )?;
                self.before_view = Some(h.view3d());
                Input::input(h, [400.0, 600.0], 0x200)?;
                let p = (
                    (400.0 * h.input.window_size[0] / 1200.0) as i32,
                    (600.0 * h.input.window_size[1] / 800.0) as i32,
                );
                post(0x20a, 120usize << 16, p)?;
            }
            9 => {
                Self::require(
                    (h.view3d().distance - self.before_view.unwrap().distance / 1.1).abs() < 0.02,
                    "perspective wheel mismatch",
                )?;
                let object = h
                    .editor
                    .preview
                    .objects
                    .iter()
                    .find(|o| o.id == 3)
                    .ok_or("sample object 3")?;
                let p = interaction::project(h.camera(), Vec3::from(object.translation))
                    .ok_or("object outside camera")?;
                Input::click(h, [p.x, p.y + 400.0])?;
            }
            10 => {
                Self::require(h.editor.selection == Some(3), "perspective OS pick target")?;
                if !h.view3d().snap {
                    if !self.snap_requested {
                        if !Self::button(h, "action-view3d-snap")? {
                            return Ok(false);
                        }
                        self.snap_requested = true;
                    }
                    return Ok(false);
                }
                if h.presented_view3d() != h.view3d() {
                    return Ok(false);
                }
                let o = h
                    .editor
                    .preview
                    .objects
                    .iter()
                    .find(|o| o.id == 3)
                    .ok_or("selected object")?;
                let vp = h.camera();
                let (_, a, d) = interaction::axes(o, vp)
                    .into_iter()
                    .find(|(axis, _, _)| *axis == 0)
                    .ok_or("X handle disabled")?;
                let start = a + d.normalize() * 40.0;
                Self::require(
                    interaction::handle(o, vp, start.to_array()).is_some_and(|x| x.0 == 0),
                    "wrong X handle",
                )?;
                let drag = AxisDrag::new(vp, Vec3::from(o.translation), 0, start)
                    .ok_or("axis plane invalid")?;
                let mut end = None;
                for n in 15..100 {
                    let p = start + d.normalize() * n as f32;
                    if let Some(v) = drag.delta(p) {
                        if v.abs() > 0.5 && p.x > 0.0 && p.x < 800.0 && p.y > 0.0 && p.y < 400.0 {
                            end = Some((p, v));
                            break;
                        }
                    }
                }
                let (end, delta) = end.ok_or("no valid X movement")?;
                self.expected = h.view3d().snapped_axis(o.translation[0] + delta);
                self.start = [start.x, start.y + 400.0];
                self.end = [end.x, end.y + 400.0];
                Input::down(h, self.start)?;
            }
            11 => {
                Self::require(h.gesture_active(), "axis capture did not begin")?;
                Input::input(h, self.end, 0x200)?;
            }
            12 => {
                Input::up(h, self.end)?;
            }
            13 => {
                if h.gesture_active() {
                    return Ok(false);
                }
                let o = h
                    .editor
                    .document
                    .objects
                    .iter()
                    .find(|o| o.id == 3)
                    .ok_or("edited object")?;
                Self::require(
                    (o.translation[0] - self.expected).abs() < 0.03 && h.editor.history().0 == 1,
                    "Auto perspective drag/history mismatch",
                )?;
                self.records.push(
                    json!({"case":"auto-drag","actual":o.translation,"expected_x":self.expected,"snap":h.view3d().snap,"step":h.view3d().step}),
                );
                if !Self::button(h, "action-undo")? {
                    return Ok(false);
                }
            }
            14 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::require(
                    h.editor.document == *self.before.as_ref().unwrap(),
                    "Undo did not restore document",
                )?;
                let path = self
                    .save
                    .get_or_insert_with(|| {
                        std::env::temp_dir()
                            .join(format!("plan857-native-{}.agscene", std::process::id()))
                    })
                    .clone();
                h.ui_mut().path = path.to_string_lossy().into_owned();
                h.rt.graph.mark_content_dirty(h.panel);
                if !Self::button(h, "action-save")? {
                    return Ok(false);
                }
            }
            15 => {
                let path = self.save.as_ref().unwrap();
                Self::require(path.exists(), "OS save did not write scene")?;
                if !Self::button(h, "action-open")? {
                    return Ok(false);
                }
            }
            16 => {
                if h.editor.pending() {
                    return Ok(false);
                }
                Self::require(
                    h.editor.document == *self.before.as_ref().unwrap(),
                    "reopened document mismatch",
                )?;
                Self::require(
                    h.view3d().sample_default && !h.view3d().grid,
                    "open did not reset 3D view",
                )?;
                self.records.push(
                    json!({"case":"save-reopen","path":self.save,"hash":h.editor.document.hash()?}),
                );
                let object = h
                    .editor
                    .preview
                    .objects
                    .iter()
                    .find(|o| o.id == 3)
                    .ok_or("reopened object")?;
                let p = interaction::project(h.camera(), Vec3::from(object.translation))
                    .ok_or("reopened pick projection")?;
                Input::click(h, [p.x, p.y + 400.0])?;
            }
            17 => {
                Self::require(h.editor.selection == Some(3), "focus selection missing")?;
                self.before_view = Some(h.view3d());
                Self::key(b'F' as usize)?;
            }
            18 => {
                Self::require(
                    h.view3d().target != self.before_view.unwrap().target,
                    "F did not focus selected content",
                )?;
                self.before_view = Some(h.view3d());
                Input::input(h, [400.0, 600.0], 0x200)?;
                Input::input(h, [400.0, 600.0], 0x207)?;
            }
            19 => Input::input(h, [450.0, 620.0], 0x200)?,
            20 => {
                Self::require(
                    h.view3d().target != self.before_view.unwrap().target,
                    "cancel fixture did not navigate",
                )?;
                Self::key(0x1b)?;
            }
            21 => {
                let before = self.before_view.unwrap();
                Self::require(
                    h.view3d().target == before.target
                        && h.camera() == before.camera()?.view_proj(),
                    "Escape did not restore camera",
                )?;
                Input::input(h, [450.0, 620.0], 0x208)?;
                self.records
                    .push(json!({"case":"focus-cancel","target":before.target.to_array()}));
                if !Self::button(h, "action-play-run")? {
                    return Ok(false);
                }
            }
            22 => {
                Self::require(h.play.is_some(), "Run did not start")?;
                Self::require(
                    h.camera() == agp::render3d::Camera3d::sample_camera(2.0).view_proj(),
                    "Run retained editor camera",
                )?;
                if !Self::button(h, "action-play-stop")? {
                    return Ok(false);
                }
            }
            23 => {
                Self::require(h.play.is_none(), "Stop did not return to editor")?;
                Self::require(
                    h.camera() == self.before_view.unwrap().camera()?.view_proj(),
                    "Stop did not restore editor camera",
                )?;
                self.records
                    .push(json!({"case":"play-restore","camera_restored":true}));
                Input::click(h, [30.0, 700.0])?;
                Self::key(0x24)?;
            }
            24 => {
                Self::require(
                    h.view3d().sample_default
                        && h.camera() == agp::render3d::Camera3d::sample_camera(2.0).view_proj(),
                    "Home did not reset camera",
                )?;
                Self::require(
                    h.editor.document == *self.before.as_ref().unwrap(),
                    "focus/cancel/play changed document",
                )?;
                Input::input(h, [400.0, 600.0], 0x200)?;
                Input::input(h, [400.0, 600.0], 0x204)?;
            }
            25 => Input::input(h, [440.0, 600.0], 0x200)?,
            26 => {
                self.before_view = Some(h.view3d());
                Input::input(h, [440.0, 600.0], 0x207)?;
                Input::input(h, [440.0, 600.0], 0x208)?;
            }
            27 => Input::input(h, [480.0, 600.0], 0x200)?,
            28 => {
                Self::require(
                    (h.view3d().yaw - self.before_view.unwrap().yaw).abs() > 0.1,
                    "middle release ended right capture",
                )?;
                Input::input(h, [480.0, 600.0], 0x205)?;
                Input::input(h, [400.0, 600.0], 0x200)?;
                Input::input(h, [400.0, 600.0], 0x207)?;
            }
            29 => Input::input(h, [440.0, 600.0], 0x200)?,
            30 => {
                self.before_view = Some(h.view3d());
                Input::input(h, [440.0, 600.0], 0x204)?;
                Input::input(h, [440.0, 600.0], 0x205)?;
            }
            31 => Input::input(h, [480.0, 600.0], 0x200)?,
            32 => {
                Self::require(
                    (h.view3d().target - self.before_view.unwrap().target).length() > 0.1,
                    "right release ended middle capture",
                )?;
                Input::input(h, [480.0, 600.0], 0x208)?;
                self.records.push(json!({"case":"cross-button-capture"}));
                Self::key(0x24)?;
            }
            33 => {
                Self::require(
                    h.view3d().sample_default
                        && h.camera() == agp::render3d::Camera3d::sample_camera(2.0).view_proj(),
                    "Home after capture did not reset",
                )?;
                self.done = true;
            }
            _ => return Err("unexpected 3D viewport phase".into()),
        }
        Ok(true)
    }
}
