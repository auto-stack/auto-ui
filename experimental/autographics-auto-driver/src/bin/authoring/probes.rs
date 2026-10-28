//! Explicitly selected native probes and completion admission.
use super::*;

impl App {
    pub(super) fn finalize_checks(&mut self) {
        if let Some(p) = &self.viewport3d_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done || p.phase < 17)
                        .then(|| format!("viewport3d incomplete phase {}", p.phase))
                });
            }
        }
        if let Some(p) = &self.viewport2d_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done || p.phase < 20)
                        .then(|| format!("viewport2d incomplete phase {}", p.phase))
                });
            }
        }
        if let Some(p) = &self.character_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done || p.phase < 10).then(|| {
                        format!(
                            "character probe incomplete at phase {} after {} frames",
                            p.phase, self.frames
                        )
                    })
                });
            }
        }
        if let Some(p) = &self.character3d_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done || p.phase < 10).then(|| {
                        format!(
                            "character3d probe incomplete at phase {} after {} frames",
                            p.phase, self.frames
                        )
                    })
                });
            }
        }
        if let Some(p) = &self.runtime_camera_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done || p.phase < 16).then(|| {
                        format!(
                            "runtime-camera probe incomplete at phase {} after {} frames",
                            p.phase, self.frames
                        )
                    })
                });
            }
        }
        if let Some(p) = &self.play_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done || p.phase < 27).then(|| {
                        format!(
                            "native play check incomplete at phase {} after {} frames",
                            p.phase, self.frames
                        )
                    })
                });
            }
        }

        if let Some(p) = &self.spatial_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done || p.phase < 14).then(|| {
                        format!(
                            "native spatial check incomplete at phase {} after {} frames",
                            p.phase, self.frames
                        )
                    })
                });
            }
        }
        let legacy_complete = !self.probe || (self.passed && self.phase >= 9);
        let interaction_complete = self
            .native_probe
            .as_ref()
            .is_none_or(|p| p.done && p.error.is_none());
        if self.error.is_none() {
            if let Some(probe) = &self.native_probe {
                if let Some(error) = &probe.error {
                    self.error = Some(error.clone());
                } else if !probe.done {
                    self.error = Some(format!(
                        "native interaction check incomplete at phase {} after {} frames",
                        probe.phase(),
                        self.frames
                    ));
                }
            }
            if self.error.is_none() && !legacy_complete {
                self.error = Some(format!(
                    "native window probe incomplete at phase {} after {} frames",
                    self.phase, self.frames
                ));
            }
        }
        if let Some(p) = &self.hierarchy_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done).then(|| {
                        format!(
                            "native hierarchy check incomplete at phase {} after {} frames",
                            p.phase, self.frames
                        )
                    })
                });
            }
        }
        if let Some(p) = &self.asset_probe {
            if self.error.is_none() {
                self.error = p.error.clone().or_else(|| {
                    (!p.done || p.phase < 20).then(|| {
                        format!(
                            "native asset check incomplete at phase {} after {} frames",
                            p.phase, self.frames
                        )
                    })
                });
            }
        }
        if self.viewport2d_probe.is_some()
            || self.viewport3d_probe.is_some()
            || self.spatial_probe.is_some()
            || self.play_probe.is_some()
            || self.character_probe.is_some()
            || self.character3d_probe.is_some()
            || self.runtime_camera_probe.is_some()
            || self.probe
            || self.native_probe.is_some()
            || self.hierarchy_probe.is_some()
            || self.asset_probe.is_some()
        {
            self.passed = self.error.is_none() && legacy_complete && interaction_complete;
        } else if self.error.is_some() {
            self.passed = false;
        }
    }
}

impl App {
    pub(super) fn legacy_scroll(h: &Host, down: bool) -> Result<(), String> {
        if std::env::args().any(|v| v == "--legacy-scroll-fault") {
            return Err("legacy scroll input precondition fault".into());
        }
        native_interaction::Probe::scroll(h, down)
    }
    pub(super) fn probe_step(&mut self) -> Result<(), String> {
        if !self.probe || self.frames - self.phase_frame < 30 {
            return Ok(());
        }
        if self.phase == 0 && !self.window.as_ref().unwrap().has_focus() {
            self.window.as_ref().unwrap().focus_window();
            return Ok(());
        }
        let h = self.host.as_mut().unwrap();
        let next = match self.phase {
            0 => {
                self.before = Some(h.editor.document.clone());
                self.before_pixels = Some(h.rt.capture(h.two, "before").unwrap().0);
                let id = if std::env::args().any(|v| v == "--sabotage-target") {
                    2
                } else {
                    1
                };
                let Some(b) = h.widget_bounds(&format!("node-{id}")) else {
                    return Ok(());
                };
                native_interaction::Probe::click(h, [810.0 + b.x + 20.0, b.y + b.height * 0.5])?;
                true
            }
            1 => {
                if h.editor.selection != Some(1) {
                    return Ok(());
                }
                let Some(b) = h.widget_bounds("authoring-field-0") else {
                    return Ok(());
                };
                if b.y < 0.0 || b.y + b.height > 790.0 {
                    Self::legacy_scroll(h, b.y >= 0.0)?;
                    return Ok(());
                }
                let point = [810.0 + b.x + 30.0, b.y + 12.0];
                native_interaction::Probe::click(h, point)?;
                true
            }
            2 => {
                if !h.rt.iced_text_input_focused(
                    h.panel,
                    iced_wgpu::core::widget::Id::new("authoring-field-0"),
                ) {
                    return Ok(());
                }
                native_interaction::post(0x100, 0x23, (0, 0))?;
                native_interaction::post(0x101, 0x23, (0, 0))?;
                native_interaction::post(0x100, 0xe7, (0, 0))?;
                native_interaction::post(0x102, b'1' as usize, (0, 0))?;
                native_interaction::post(0x101, 0xe7, (0, 0))?;
                true
            }
            3 => {
                if h.ui().draft.get(1).is_none_or(|s| s == "-55") {
                    return Ok(());
                }
                let Some(b) = h.widget_bounds("action-apply") else {
                    return Ok(());
                };
                if b.y < 0.0 || b.y + b.height > 790.0 {
                    Self::legacy_scroll(h, b.y >= 0.0)?;
                    return Ok(());
                }
                native_interaction::Probe::click(h, [810.0 + b.x + 20.0, b.y + b.height * 0.5])?;
                true
            }
            4 => {
                if h.editor.pending() || h.editor.stamp.revision == 0 {
                    return Ok(());
                }
                let before = self.before.as_ref().unwrap();
                let ok = h.editor.document.objects[0].translation[0] == -551.0
                    && h.editor.document.objects[1..] == before.objects[1..]
                    && h.editor.dirty()
                    && h.accepted == 1
                    && h.rt.capture(h.two, "after").unwrap().0
                        != *self.before_pixels.as_ref().unwrap();
                if !ok {
                    return Err("native widget->Auto->target/pixels oracle".into());
                }
                let Some(b) = h.widget_bounds("action-save") else {
                    return Ok(());
                };
                if b.y < 0.0 || b.y + b.height > 790.0 {
                    Self::legacy_scroll(h, b.y >= 0.0)?;
                    return Ok(());
                }
                native_interaction::Probe::click(
                    h,
                    [810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5],
                )?;
                true
            }
            5 => {
                if h.editor.dirty() || h.storage.path.is_none() {
                    return Ok(());
                }
                let Some(b) = h.widget_bounds("action-undo") else {
                    return Ok(());
                };
                if b.y < 0.0 || b.y + b.height > 790.0 {
                    Self::legacy_scroll(h, b.y >= 0.0)?;
                    return Ok(());
                }
                native_interaction::Probe::click(
                    h,
                    [810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5],
                )?;
                true
            }
            6 => {
                if h.editor.pending()
                    || !h.editor.dirty()
                    || h.editor.document.objects[0].translation[0] != -55.0
                {
                    return Ok(());
                }
                let Some(b) = h.widget_bounds("action-redo") else {
                    return Ok(());
                };
                if b.y < 0.0 || b.y + b.height > 790.0 {
                    Self::legacy_scroll(h, b.y >= 0.0)?;
                    return Ok(());
                }
                native_interaction::Probe::click(
                    h,
                    [810.0 + b.x + b.width * 0.5, b.y + b.height * 0.5],
                )?;
                true
            }
            7 => {
                if h.editor.pending()
                    || h.editor.dirty()
                    || h.editor.document.objects[0].translation[0] != -551.0
                {
                    return Ok(());
                }
                let _ = self
                    .window
                    .as_ref()
                    .unwrap()
                    .request_inner_size(winit::dpi::LogicalSize::new(1000.0, 700.0));
                true
            }
            8 => {
                self.passed = true;
                self.limit = Some(self.frames + 1);
                true
            }
            _ => false,
        };
        if next {
            self.phase += 1;
            self.phase_frame = self.frames;
        }
        Ok(())
    }
}
