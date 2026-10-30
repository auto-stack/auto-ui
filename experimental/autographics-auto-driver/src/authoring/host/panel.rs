//! Stock Iced panel presentation and local drafts.
use super::*;

impl Panel {
    pub(super) fn sync(&mut self, e: &Editor) {
        self.document = e.document.clone();
        self.stamp = e.stamp;
        self.selection = e.selection;
        self.derived = e.preview.nodes.clone();
        self.busy = e.pending();
        self.dirty = e.dirty();
        self.reset_draft();
    }
    pub(super) fn reset_draft(&mut self) {
        self.bounds.lock().unwrap().clear();
        self.draft.clear();
        self.parent_choice = self
            .selection
            .and_then(|id| self.document.parent(id).ok().flatten());
        self.delete_confirmation = None;
        if let Some(o) = self.document.local_target(self.selection.unwrap_or(0)) {
            self.draft.push(o.name.clone());
            self.draft.extend(
                o.translation
                    .iter()
                    .chain(&o.scale)
                    .chain([&o.yaw, &o.pitch])
                    .chain(&o.rgba)
                    .map(ToString::to_string),
            );
            self.draft.push(o.visible.to_string());
        }
        self.original_draft = self.draft.clone();
    }
    pub(super) fn draft_message(&self, index: usize, value: String) -> String {
        format!(
            "draft:{}:{}:{}:{}:{}",
            self.stamp.epoch,
            self.stamp.revision,
            self.selection.unwrap_or(0),
            index,
            value
        )
    }
    pub(super) fn message(&self, action: &str, payload: Value) -> String {
        json!({"view_revision":self.view2d.revision,"view3d_revision":self.view3d.revision,"epoch":self.stamp.epoch,"revision":self.stamp.revision,"selection":self.selection,"asset_generation":self.asset_token,"play_token":self.play_token,"action":action,"payload":payload}).to_string()
    }
}
impl SurfaceUi for Panel {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        use iced::widget::{button, column, row, scrollable, text, text_input};
        let enabled = !self.busy && !self.dialog && self.play_state.is_none();
        let cmd = |label: &'static str, action: &str, payload: Value| {
            let b = button(label);
            let b = if enabled {
                b.on_press(self.message(action, payload))
            } else {
                b
            };
            crate::authoring::tagged::tag(b, format!("action-{action}"), self.bounds.clone())
        };
        let mut c = column![
            text(if self.busy {
                "Preparing preview…"
            } else if self.dirty {
                "Scene • modified"
            } else {
                "Scene • saved"
            }),
            text(self.error.clone()),
            text_input("Scene file path", &self.path)
                .id("authoring-scene-path")
                .on_input(|v| format!("path:{v}")),
            row![
                cmd("Open", "open", json!(self.path)),
                cmd("Save", "save", json!(self.path)),
                cmd("Save as", "saveas", json!(self.path)),
                cmd("New", "new", Value::Null)
            ],
            row![
                cmd("Undo", "undo", Value::Null),
                cmd("Redo", "redo", Value::Null)
            ]
        ]
        .spacing(5);
        let play_button = |label: String, action: &str, enabled: bool| {
            let b = button(text(label));
            let b = if enabled {
                b.on_press(self.message(action, Value::Null))
            } else {
                b
            };
            crate::authoring::tagged::tag(b, format!("action-{action}"), self.bounds.clone())
        };
        let editing = self.play_state.is_none();
        let paused = self.play_state.as_deref() == Some("Paused");
        let running = self.play_state.as_deref() == Some("Running");
        let mut behavior_path =
            text_input("User .at behavior file", &self.behavior_path).id("authoring-behavior-path");
        if editing {
            behavior_path = behavior_path.on_input(|v| self.message("play-source-path", json!(v)));
        }
        c = c
            .push(play_button(
                "Cancel binding".into(),
                "cancel-binding",
                self.binding_busy,
            ))
            .push(row![
                play_button(
                    if self.behavior_user {
                        "Use built-in"
                    } else {
                        "Use user script"
                    }
                    .into(),
                    "play-source-toggle",
                    editing
                ),
                text(if self.project_default {
                    self.document.behavior.as_ref().map_or("Builtin6",|b|b.api.as_str())
                } else if self.behavior_user {self.behavior_api.version()} else {"Builtin6"}),
            ])
            .push(play_button(
                "Use project default".into(),
                "play-project-default",
                editing,
            ))
            .push(row![
                cmd(
                    "Bind/update script",
                    "behavior-bind",
                    json!(self.behavior_path)
                ),
                cmd("Apply bound actors", "behavior-actors", Value::Null),
                cmd("Detach script", "behavior-detach", Value::Null)
            ])
            .push(row![
                cmd("No bound 2D", "behavior-actors", json!({"two_d":null,"three_d":self.document.behavior.as_ref().and_then(|b|b.actors.three_d)})),
                cmd("No bound 3D", "behavior-actors", json!({"two_d":self.document.behavior.as_ref().and_then(|b|b.actors.two_d),"three_d":null}))
            ])
            .push(row![cmd("API v1", "behavior-api-v1", Value::Null),cmd("API v2 spatial", "behavior-api-v2", Value::Null),cmd("API v3 segment", "behavior-api-v3", Value::Null)])
            .push(row![cmd("API v4 sweep", "behavior-api-v4", Value::Null),cmd("API v5 motion", "behavior-api-v5", Value::Null),cmd("API character/1", "behavior-api-character", Value::Null)])
            .push(row![
                cmd("API character3d/1", "behavior-api-character3d", Value::Null),
                cmd("API camera2d/1", "behavior-api-camera2d", Value::Null),
                cmd("API camera3d/1", "behavior-api-camera3d", Value::Null)
            ])
            .push(behavior_path)
            .push(text(self.behavior_resolved.clone()))
            .push(text(self.last_behavior.clone()));
        c = c
            .push(text(format!(
                "{} / Auto tick {}",
                self.play_state.as_deref().unwrap_or("Editing"),
                self.play_tick
            )))
            .push(row![
                play_button(
                    "Run".into(),
                    "play-run",
                    editing && !self.busy && !self.dialog
                ),
                play_button("Pause".into(), "play-pause", running),
                play_button("Resume".into(), "play-resume", paused)
            ])
            .push(row![
                play_button("Step".into(), "play-step", paused),
                play_button("Stop".into(), "play-stop", !editing)
            ])
            .push(row![
                play_button(
                    format!(
                        "2D actor {}: Next",
                        self.actors[0]
                            .map(|id| format!("#{id}"))
                            .unwrap_or_else(|| "None".into())
                    ),
                    "play-actor2",
                    editing
                ),
                play_button(
                    format!(
                        "3D actor {}: Next",
                        self.actors[1]
                            .map(|id| format!("#{id}"))
                            .unwrap_or_else(|| "None".into())
                    ),
                    "play-actor3",
                    editing
                )
            ]);
        if self.asset_busy {
            c = c.push(
                button("Cancel import")
                    .on_press(self.message("cancel-import", json!(self.asset_token))),
            );
        }
        if self.dialog {
            c = c
                .push(text("Unsaved changes: Save / Discard / Cancel"))
                .push(row![
                    button("Save").on_press(self.message("leave-save", Value::Null)),
                    button("Discard").on_press(self.message("leave-discard", Value::Null)),
                    button("Cancel").on_press(self.message("leave-cancel", Value::Null))
                ]);
        }
        if let Ok(nodes) = crate::authoring::hierarchy::ordered(&self.document) {
            for dimension in [Dimension::Two, Dimension::Three] {
                c = c.push(text(if dimension == Dimension::Two {
                    "2D scene"
                } else {
                    "3D scene"
                }));
                for n in nodes.iter().filter(|n| n.dimension == dimension) {
                    let mut depth = 0;
                    let mut parent = n.parent;
                    while let Some(id) = parent {
                        depth += 1;
                        parent = self.document.parent(id).ok().flatten();
                    }
                    let name = self
                        .document
                        .local_target(n.id)
                        .map(|o| o.name)
                        .unwrap_or_default();
                    let b = button(text(format!(
                        "{}{} {} #{} {}",
                        "  ".repeat(depth.min(8)),
                        if Some(n.id) == self.selection {
                            "▶"
                        } else {
                            ""
                        },
                        if n.kind == 5 { "Group" } else { "Leaf" },
                        n.id,
                        name
                    )));
                    let b = if enabled {
                        b.on_press(self.message("select", json!(n.id)))
                    } else {
                        b
                    };
                    c = c.push(crate::authoring::tagged::tag(
                        b,
                        format!("node-{}", n.id),
                        self.bounds.clone(),
                    ));
                }
            }
        }
        c = c
            .push(crate::authoring::tagged::tag(
                text_input("Local asset path (GLB/Ogg/WAV/TTF)", &self.import_path)
                    .id("authoring-asset-path")
                    .on_input(|v| format!("asset-path:{v}")),
                "asset-path".into(),
                self.bounds.clone(),
            ))
            .push(cmd("Import GLB", "import", json!(self.import_path)))
            .push(cmd("Import Audio", "import-audio", json!(self.import_path)))
            .push(cmd("Import Font", "import-font", json!(self.import_path)))
            .push(cmd(
                "Import Animated GLB",
                "import-animated",
                json!(self.import_path),
            ))
            .push(crate::authoring::tagged::tag(
                text_input("Local PNG path", &self.import_sprite_path)
                    .id("authoring-sprite-path")
                    .on_input(|v| format!("sprite-path:{v}")),
                "sprite-path".into(),
                self.bounds.clone(),
            ))
            .push(cmd(
                "Import PNG",
                "import-sprite",
                json!(self.import_sprite_path),
            ))
            .push(row![
                cmd("+Circle", "create", json!("circle")),
                cmd("+Quad", "create", json!("quad")),
                cmd("+Cylinder", "create", json!("cylinder"))
            ])
            .push(row![
                cmd("+Player", "create", json!("player")),
                cmd("+Mob", "create", json!("mob")),
                cmd("Delete", "delete-request", json!(self.selection))
            ]);
        c = c.push(row![
            cmd("+2D group", "create-group2", json!("2d")),
            cmd("+3D group", "create-group3", json!("3d"))
        ]);
        for a in self
            .document
            .assets
            .iter()
            .filter(|a| !a.is_image() && !a.is_audio())
        {
            let key = a.key();
            c = c
                .push(text(format!(
                    "{} {}",
                    a.name.as_deref().unwrap_or("GLB"),
                    &a.sha256[..8]
                )))
                .push(row![
                    cmd("Create instance", "asset-create", json!(key)),
                    cmd("Use on selected 3D", "asset-replace", json!(key))
                ]);
        }
        // PLAN-004: an accepted image is bound to a 2D leaf; the frame is the
        // whole image until Auto (or the document) chooses a pixel rectangle.
        for a in self.document.assets.iter().filter(|a| a.is_image()) {
            let key = a.key();
            c = c
                .push(text(format!(
                    "PNG {} {}x{}",
                    &a.sha256[..8],
                    a.width.unwrap_or(0),
                    a.height.unwrap_or(0)
                )))
                .push(row![
                    cmd("Bind to selected 2D", "sprite-bind", json!(key)),
                    cmd("Create sprite leaf", "sprite-create", json!(key)),
                    cmd("Detach selected 2D", "sprite-detach", json!(key))
                ]);
        }
        if let Some(id) = self.delete_confirmation {
            let count = crate::authoring::hierarchy::descendants(&self.document, id)
                .map(|x| x.len().saturating_sub(1))
                .unwrap_or(0);
            c = c
                .push(text(format!("Delete group #{id} and {count} descendants?")))
                .push(row![
                    cmd("Delete subtree", "delete-confirm", json!(id)),
                    cmd("Cancel deletion", "delete-cancel", Value::Null)
                ]);
        }
        if let Some(id) = self.selection {
            if let Some(world) = self.derived.get(&id) {
                c = c.push(text(format!(
                    "World {:?} / effective visible {}",
                    world.translation, world.visible
                )));
            }
            if let Some(reason) = &self.group_handle_reason {
                c = c.push(text(reason));
            }
            let b = button("Root");
            let b = if enabled {
                b.on_press(self.message("parent-choice", json!({"target":id,"parent":null})))
            } else {
                b
            };
            let mut choices = column![crate::authoring::tagged::tag(
                b,
                "parent-root".into(),
                self.bounds.clone()
            )]
            .spacing(4);
            for g in self.document.groups.iter().filter(|g| g.id != id) {
                let b = button(text(format!("#{} {}", g.id, g.name)));
                let b = if enabled {
                    b.on_press(self.message("parent-choice", json!({"target":id,"parent":g.id})))
                } else {
                    b
                };
                choices = choices.push(crate::authoring::tagged::tag(
                    b,
                    format!("parent-{}", g.id),
                    self.bounds.clone(),
                ));
            }
            c=c.push(text(format!("Parent target: {}",self.parent_choice.map(|id|format!("Group #{id}")).unwrap_or_else(||"Root".into())))).push(choices)
                .push(cmd("Keep world / attach","reparent",json!({"target":id,"expected_parent":self.document.parent(id).ok().flatten(),"parent":self.parent_choice})));
        }
        if !self.draft.is_empty() {
            c = c.push(row![
                cmd(
                    "Apply",
                    "apply",
                    json!({"target":self.selection,"draft":self.draft})
                ),
                cmd("Cancel draft", "cancel", Value::Null),
                cmd("Toggle visible", "toggle-visible", json!(self.selection))
            ]);
            let mut fields = column![text_input("Name", &self.draft[0])
                .id("authoring-name")
                .on_input(|v| self.draft_message(0, v))]
            .spacing(3);
            for (i, label) in FIELDS.iter().enumerate().filter(|(i, _)| {
                self.selection
                    .is_none_or(|id| !self.document.groups.iter().any(|g| g.id == id))
                    || *i < 3
            }) {
                fields = fields.push(row![
                    text(*label).width(70),
                    text_input(label, &self.draft[i + 1])
                        .id(format!("authoring-field-{i}"))
                        .on_input(move |v| self.draft_message(i + 1, v))
                ]);
            }
            fields = fields.push(row![
                text("visible"),
                text_input("true / false", &self.draft[13]).on_input(|v| self.draft_message(13, v))
            ]);
            c = c.push(fields);
        }
        c=c.push(text("3D selection uses model bounds (may include empty space). X red / Y green / Z blue: world translation. Short or clipped axes are disabled; use properties."));
        c = c
            .push(row![
                cmd(
                    if self.view2d.grid {
                        "Grid on"
                    } else {
                        "Grid off"
                    },
                    "view-grid",
                    Value::Null
                ),
                cmd(
                    if self.view2d.snap {
                        "Snap on"
                    } else {
                        "Snap off"
                    },
                    "view-snap",
                    Value::Null
                )
            ])
            .push(row![
                cmd("Focus 2D", "view-focus", Value::Null),
                cmd("Reset 2D", "view-reset", Value::Null)
            ])
            .push(row![
                cmd("1", "view-step", json!(1)),
                cmd("5", "view-step", json!(5)),
                cmd("10", "view-step", json!(10)),
                cmd("50", "view-step", json!(50)),
                cmd("100", "view-step", json!(100))
            ])
            .push(text(format!(
                "2D zoom {:.3} • snap {} • grid {} / {} lines",
                self.view2d.zoom,
                self.view2d.step,
                self.view2d.grid_lines().0,
                self.view2d.grid_lines().1.len()
            )));
        c = c
            .push(row![
                cmd(
                    if self.view3d.grid {
                        "3D grid on"
                    } else {
                        "3D grid off"
                    },
                    "view3d-grid",
                    Value::Null
                ),
                cmd(
                    if self.view3d.snap {
                        "3D snap on"
                    } else {
                        "3D snap off"
                    },
                    "view3d-snap",
                    Value::Null
                )
            ])
            .push(row![
                cmd("Focus 3D", "view3d-focus", Value::Null),
                cmd("Reset 3D", "view3d-reset", Value::Null),
                cmd("Projection", "view3d-projection", Value::Null)
            ])
            .push(row![
                cmd("1", "view3d-step", json!(1)),
                cmd("5", "view3d-step", json!(5)),
                cmd("10", "view3d-step", json!(10)),
                cmd("50", "view3d-step", json!(50)),
                cmd("100", "view3d-step", json!(100))
            ])
            .push(text(format!(
                "3D {:?} • snap {} • grid {}",
                self.view3d.mode,
                self.view3d.step,
                if self.view3d.grid { "on" } else { "off" }
            )));
        crate::authoring::tagged::observe_scroll(
            scrollable(c.padding(8)),
            self.scroll_offset.clone(),
            self.bounds.clone(),
        )
    }
    fn update(&mut self, m: String) {
        if self.play_state.is_some()
            && serde_json::from_str::<Value>(&m)
                .ok()
                .is_none_or(|v| !v["action"].as_str().is_some_and(|a| a.starts_with("play-")))
        {
            return;
        }
        if self.busy
            && serde_json::from_str::<Value>(&m)
                .ok()
                .is_none_or(|v| v["action"] != "cancel-import" && v["action"] != "cancel-binding")
        {
            return;
        }
        if let Some(v) = m.strip_prefix("path:") {
            self.path = v.into();
            return;
        }
        if let Some(v) = m.strip_prefix("asset-path:") {
            self.import_path = v.into();
            return;
        }
        if let Some(v) = m.strip_prefix("sprite-path:") {
            self.import_sprite_path = v.into();
            return;
        }
        if m.starts_with("draft:") {
            let parts: Vec<_> = m.splitn(6, ':').collect();
            if parts.len() == 6
                && !self.busy
                && !self.dialog
                && parts[1].parse::<u64>().ok() == Some(self.stamp.epoch)
                && parts[2].parse::<u64>().ok() == Some(self.stamp.revision)
                && parts[3].parse::<u64>().ok() == self.selection
            {
                if let Ok(i) = parts[4].parse::<usize>() {
                    if let Some(s) = self.draft.get_mut(i) {
                        *s = parts[5].into();
                    }
                }
            }
            return;
        }
        self.queue.push(m);
    }
    fn background(&self) -> iced::Color {
        iced::Color::from_rgb(0.1, 0.12, 0.15)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
