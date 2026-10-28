//! View publication remains separate from scene transactions and history.
use super::*;
use crate::authoring::view2d::View2d;
use glam::Vec2;
impl Host {
    pub fn view2d(&self) -> View2d {
        self.view2d
    }
    pub fn presented_view2d(&self) -> View2d {
        self.presented_view2d
    }
    pub(super) fn active_view2d(&self) -> View2d {
        if self.play.is_some() {
            View2d::default()
        } else {
            self.view2d
        }
    }
    fn view_idle(&self) -> Result<(), String> {
        if self.play.is_some()
            || self.navigation3d.is_some()
            || self.gesture.is_some()
            || self.editor.pending()
            || self.leave.is_some()
        {
            Err("2D view busy".into())
        } else {
            Ok(())
        }
    }
    pub fn set_view2d(&mut self, view: View2d) -> Result<(), String> {
        if self.navigation.is_some() {
            return Err("2D navigation busy".into());
        }
        self.apply_view2d(view)
    }
    fn apply_view2d(&mut self, mut view: View2d) -> Result<(), String> {
        self.view_idle()?;
        if !view.valid() {
            return Err("invalid 2D view".into());
        }
        let old = self.view2d;
        view.revision = old.revision;
        if view == old {
            return Ok(());
        }
        view.revision = old
            .revision
            .checked_add(1)
            .ok_or("view revision exhausted")?;
        self.view2d = view;
        match self.prepare(&self.editor.preview) {
            Ok(p) => {
                self.publish(p);
                // Navigation is not a property transaction; preserve an unsubmitted draft.
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
                Ok(())
            }
            Err(e) => {
                self.view2d = old;
                Err(e)
            }
        }
    }
    pub(super) fn view_command(&mut self, action: &str, payload: &Value) -> Result<(), String> {
        self.view_idle()?;
        if self.navigation.is_some() {
            return Err("2D navigation busy".into());
        }
        let mut v = self.view2d;
        match action {
            "view-reset" => v = View2d::default(),
            "view-grid" => v.grid = !v.grid,
            "view-snap" => v.snap = !v.snap,
            "view-step" => v.step = payload.as_f64().ok_or("snap step")? as f32,
            "view-focus" => {
                let id = self
                    .editor
                    .selection
                    .ok_or("select visible 2D content to focus")?;
                let ids = if self.editor.document.groups.iter().any(|g| g.id == id) {
                    crate::authoring::hierarchy::descendants(&self.editor.document, id)?
                } else {
                    [id].into_iter().collect()
                };
                let mut lo = Vec2::splat(f32::INFINITY);
                let mut hi = Vec2::splat(f32::NEG_INFINITY);
                for o in self.editor.preview.objects.iter().filter(|o| {
                    ids.contains(&o.id) && o.kind.is_2d() && o.visible && o.rgba[3] > 0.0
                }) {
                    let c = Vec2::new(o.translation[0], o.translation[1]);
                    let e = Vec2::new(o.scale[0], o.scale[1]) * 0.5;
                    lo = lo.min(c - e);
                    hi = hi.max(c + e);
                }
                v = v.focus(lo, hi)?;
            }
            _ => return Err("unknown 2D view action".into()),
        }
        self.set_view2d(v)
    }
    pub fn navigation_down(&mut self, p: [f32; 2]) {
        if crate::authoring::interaction::inside(p, self.input.window_size) {
            let top = self.input.window_to_top(p);
            if crate::authoring::interaction::inside([top[0], top[1] - 400.0], [800.0, 400.0]) {
                self.navigation3d_down(p, false);
                return;
            }
        }
        if !crate::authoring::interaction::inside(p, self.input.window_size)
            || self.view_idle().is_err()
            || self.navigation.is_some()
            || self.presented_view2d != self.view2d
        {
            return;
        }
        if let Some(InputOutcome::Delivered { surface, local }) =
            self.input.pointer_pressed(&self.rt.graph, p)
        {
            if surface == self.two {
                self.rt.release_iced_focus(self.panel);
                self.events.clear();
                if let Some(SurfaceContent::Game2d(two)) = self.rt.content(self.two) {
                    self.navigation = Some(Navigation {
                        start: self.view2d,
                        origin: local,
                        draws: two.commands.clone(),
                    });
                    return;
                }
            }
        }
        self.input.pointer_released();
    }
    pub(super) fn navigation_move(&mut self, p: [f32; 2]) {
        let Some(navigation) = self.navigation.as_ref() else {
            return;
        };
        let start = navigation.start;
        let origin = navigation.origin;
        if !crate::authoring::interaction::inside(p, self.input.window_size) {
            self.cancel_navigation();
            return;
        }
        let Some(InputOutcome::Delivered { surface, local }) =
            self.input.pointer_moved(&self.rt.graph, p)
        else {
            self.cancel_navigation();
            return;
        };
        if surface != self.two {
            self.cancel_navigation();
            return;
        }
        let result = start
            .pan(Vec2::from(local) - Vec2::from(origin))
            .and_then(|v| self.apply_view2d(v));
        if let Err(e) = result {
            self.ui_mut().error = e;
            self.cancel_navigation();
        }
    }
    pub fn navigation_up(&mut self, p: [f32; 2]) {
        if self.navigation3d.is_some() {
            self.navigation3d_up_for(p, false);
            return;
        }
        if self.navigation.is_none() {
            return;
        }
        self.navigation_move(p);
        self.navigation = None;
        self.input.pointer_released();
    }
    pub fn cancel_navigation(&mut self) {
        self.cancel_navigation3d();
        if let Some(navigation) = self.navigation.take() {
            self.input.pointer_released();
            // Restore the already-admitted 2D commands directly: cancellation must
            // not depend on another fallible resource preparation or alter faults.
            let mut start = navigation.start;
            if self.view2d != start {
                start.revision = self.view2d.revision.saturating_add(1);
            }
            self.view2d = start;
            if let Some(SurfaceContent::Game2d(two)) = self.rt.content_mut(self.two) {
                two.commands = navigation.draws;
            }
            self.rt.graph.mark_content_dirty(self.two);
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
