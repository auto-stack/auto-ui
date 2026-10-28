//! Editing-only 3D navigation and transactional scene/camera publication.
use super::*;
use crate::authoring::view3d::View3d;
use glam::Vec2;

impl Host {
    pub fn view3d(&self) -> View3d {
        self.view3d
    }
    pub fn presented_view3d(&self) -> View3d {
        self.presented_view3d
    }
    pub(super) fn install_3d(&mut self, view: View3d) -> Result<(), String> {
        let vp = view.camera()?.view_proj();
        let (_, grid) = view.grid_vertices()?;
        self.rt
            .set_editor_grid(self.three, if view.grid { Some(&grid) } else { None })?;
        let Some(SurfaceContent::Scene3d(scene)) = self.rt.content_mut(self.three) else {
            return Err("3D scene unavailable".into());
        };
        scene.view_proj = vp;
        self.input.mapper.register(self.three, vp, [800.0, 400.0]);
        self.rt.graph.mark_content_dirty(self.three);
        Ok(())
    }
    pub fn set_view3d(&mut self, view: View3d) -> Result<(), String> {
        if self.navigation.is_some() || self.navigation3d.is_some() {
            return Err("viewport navigation busy".into());
        }
        self.apply_view3d(view)
    }
    fn apply_view3d(&mut self, mut view: View3d) -> Result<(), String> {
        if self.play.is_some()
            || self.gesture.is_some()
            || self.editor.pending()
            || self.leave.is_some()
        {
            return Err("3D view busy".into());
        }
        if !view.valid() {
            return Err("invalid 3D view".into());
        }
        let old = self.view3d;
        view.revision = old.revision;
        if view == old {
            return Ok(());
        }
        view.revision = old
            .revision
            .checked_add(1)
            .ok_or("3D view revision exhausted")?;
        let vp = view.camera()?.view_proj();
        let old_vp = self.camera();
        let Some(SurfaceContent::Scene3d(scene)) = self.rt.content_mut(self.three) else {
            return Err("3D scene unavailable".into());
        };
        scene.view_proj = vp;
        self.view3d = view;
        let prepared = self.prepare(&self.editor.preview);
        let prepared = match prepared {
            Ok(p) => p,
            Err(e) => {
                self.view3d = old;
                if let Some(SurfaceContent::Scene3d(scene)) = self.rt.content_mut(self.three) {
                    scene.view_proj = old_vp;
                }
                return Err(e);
            }
        };
        if let Err(e) = self.install_3d(view) {
            self.view3d = old;
            if let Some(SurfaceContent::Scene3d(scene)) = self.rt.content_mut(self.three) {
                scene.view_proj = old_vp;
            }
            return Err(e);
        }
        self.publish(prepared);
        let preserve =
            self.ui().selection == self.editor.selection && self.ui().stamp == self.editor.stamp;
        let draft = self.ui().draft.clone();
        let original = self.ui().original_draft.clone();
        let parent = self.ui().parent_choice;
        self.sync_ui();
        if preserve {
            self.ui_mut().draft = draft;
            self.ui_mut().original_draft = original;
            self.ui_mut().parent_choice = parent;
        }
        Ok(())
    }
    pub(super) fn view3d_command(&mut self, action: &str, payload: &Value) -> Result<(), String> {
        let mut v = self.view3d;
        match action {
            "view3d-reset" => v = View3d::default(),
            "view3d-grid" => v.grid = !v.grid,
            "view3d-snap" => v.snap = !v.snap,
            "view3d-step" => v.step = payload.as_f64().ok_or("3D snap step")? as f32,
            "view3d-projection" => {
                let (next, limited) = v.toggle_projection()?;
                v = next;
                if limited {
                    self.ui_mut().error = "3D projection scale limited".into();
                }
            }
            "view3d-focus" => {
                let id = self
                    .editor
                    .selection
                    .ok_or("select visible 3D content to focus")?;
                let ids = if self.editor.document.groups.iter().any(|g| g.id == id) {
                    crate::authoring::hierarchy::descendants(&self.editor.document, id)?
                } else {
                    [id].into_iter().collect()
                };
                let mut lo = Vec3::splat(f32::INFINITY);
                let mut hi = Vec3::splat(f32::NEG_INFINITY);
                for object in self
                    .editor
                    .preview
                    .objects
                    .iter()
                    .filter(|o| ids.contains(&o.id) && o.visible && !o.kind.is_2d())
                {
                    let model = self.model(object).ok_or("missing model resource")?;
                    let (a, b) = crate::authoring::interaction::bounds(model)
                        .ok_or("invalid model bounds")?;
                    let transform = crate::authoring::interaction::transform(object);
                    for i in 0..8 {
                        let p = transform.transform_point3(Vec3::new(
                            if i & 1 == 0 { a.x } else { b.x },
                            if i & 2 == 0 { a.y } else { b.y },
                            if i & 4 == 0 { a.z } else { b.z },
                        ));
                        if !p.is_finite() {
                            return Err("nonfinite visible 3D bounds".into());
                        }
                        lo = lo.min(p);
                        hi = hi.max(p);
                    }
                }
                v = v.focus(lo, hi)?;
            }
            _ => return Err("unknown 3D view action".into()),
        }
        self.set_view3d(v)
    }
    pub fn navigation3d_down(&mut self, p: [f32; 2], orbit: bool) {
        if !crate::authoring::interaction::inside(p, self.input.window_size)
            || self.play.is_some()
            || self.gesture.is_some()
            || self.editor.pending()
            || self.leave.is_some()
            || self.navigation.is_some()
            || self.navigation3d.is_some()
            || self.presented_view3d != self.view3d
            || self.presented_camera != self.camera()
        {
            return;
        }
        if let Some(InputOutcome::Delivered { surface, local }) =
            self.input.pointer_pressed(&self.rt.graph, p)
        {
            if surface == self.three {
                self.rt.release_iced_focus(self.panel);
                self.events.clear();
                let overlay = match self.rt.content(self.three) {
                    Some(SurfaceContent::Scene3d(s)) => s
                        .screen_overlay
                        .as_ref()
                        .map_or_else(Vec::new, |o| o.commands.clone()),
                    _ => Vec::new(),
                };
                self.navigation3d = Some(Navigation3d {
                    start: self.view3d,
                    origin: local,
                    orbit,
                    overlay,
                });
                return;
            }
        }
        self.input.pointer_released();
    }
    pub(super) fn navigation3d_move(&mut self, p: [f32; 2]) {
        let Some(nav) = self.navigation3d.as_ref() else {
            return;
        };
        let start = nav.start;
        let origin = nav.origin;
        let orbit = nav.orbit;
        if !crate::authoring::interaction::inside(p, self.input.window_size) {
            self.cancel_navigation3d();
            return;
        }
        let Some(InputOutcome::Delivered { surface, local }) =
            self.input.pointer_moved(&self.rt.graph, p)
        else {
            self.cancel_navigation3d();
            return;
        };
        if surface != self.three {
            self.cancel_navigation3d();
            return;
        }
        let delta = Vec2::from(local) - Vec2::from(origin);
        let result = if orbit {
            start.orbit(delta)
        } else {
            start.pan(delta)
        };
        if let Err(e) = result.and_then(|v| self.apply_view3d(v)) {
            self.ui_mut().error = e;
            self.cancel_navigation3d();
        }
    }
    pub fn navigation3d_up(&mut self, p: [f32; 2]) {
        self.navigation3d_up_for(p, true);
    }
    pub(super) fn navigation3d_up_for(&mut self, p: [f32; 2], orbit: bool) {
        if !self
            .navigation3d
            .as_ref()
            .is_some_and(|nav| nav.orbit == orbit)
        {
            return;
        }
        self.navigation3d_move(p);
        self.navigation3d = None;
        self.input.pointer_released();
    }
    pub(super) fn cancel_navigation3d(&mut self) {
        if let Some(nav) = self.navigation3d.take() {
            self.input.pointer_released();
            let mut start = nav.start;
            if start != self.view3d {
                start.revision = self.view3d.revision.saturating_add(1);
            }
            if let Err(e) = self.install_3d(start) {
                self.ui_mut().error = e;
                return;
            }
            self.view3d = start;
            if let Some(SurfaceContent::Scene3d(scene)) = self.rt.content_mut(self.three) {
                if let Some(o) = scene.screen_overlay.as_mut() {
                    o.commands = nav.overlay;
                }
            }
            self.rt.graph.mark_content_dirty(self.three);
            let preserve = self.ui().selection == self.editor.selection
                && self.ui().stamp == self.editor.stamp;
            let draft = self.ui().draft.clone();
            let original = self.ui().original_draft.clone();
            let parent = self.ui().parent_choice;
            self.sync_ui();
            if preserve {
                self.ui_mut().draft = draft;
                self.ui_mut().original_draft = original;
                self.ui_mut().parent_choice = parent;
            }
        }
    }
}
