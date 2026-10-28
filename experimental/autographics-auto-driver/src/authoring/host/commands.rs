//! Stamped editor and project commands.
use super::*;

impl Host {
    pub fn request_close(&mut self) {
        self.cancel_navigation();
        self.stop_play();
        self.cancel_gesture();
        if !self.editor.pending() {
            self.request_leave(Leave::Close);
        }
    }
    pub(super) fn request_leave(&mut self, action: Leave) {
        if self.editor.dirty() || self.storage.unverified() {
            self.leave = Some(action);
            self.sync_ui();
        } else if let Err(e) = self.perform_leave(action) {
            self.ui_mut().error = e;
        }
    }
    pub(super) fn perform_leave(&mut self, action: Leave) -> Result<(), String> {
        match action {
            Leave::Close => {
                self.editor.close();
                let keep: std::collections::BTreeSet<_> = self
                    .models
                    .iter()
                    .map(|m| m.source_sha256.clone())
                    .collect();
                if let Some(SurfaceContent::Scene3d(c)) = self.rt.content_mut(self.three) {
                    c.instances.clear();
                    c.store.retain_models(|s| keep.contains(s));
                }
                self.closed = true;
            }
            Leave::Open(p) => {
                let (d, assets, code, s) = Storage::load_project(&p, &self.active_assets()?)?;
                self.editor.begin_open_project(d, assets, code)?;
                self.opening = Some(s);
            }
            Leave::New => {
                self.editor
                    .begin_open(Document::empty(format!("new-{}", self.editor.stamp.epoch)))?;
                self.opening = Some(Storage::default());
            }
        }
        Ok(())
    }
    pub fn dispatch(&mut self, message: &str) -> Result<(), String> {
        if let Ok(v) = serde_json::from_str::<Value>(message) {
            if v["action"].as_str().is_some_and(|a| a.starts_with("play-")) {
                return self.dispatch_play(&v);
            }
        }
        if self.navigation.is_some() || self.navigation3d.is_some() {
            return Err("viewport navigation busy".into());
        }
        if self.play.is_some() {
            return Err("scene play busy: Stop before editing/files".into());
        }

        if let Some(path) = message.strip_prefix("asset-path:") {
            if self.editor.pending() || self.gesture.is_some() {
                return Err("busy".into());
            }
            self.ui_mut().import_path = path.to_owned();
            self.rt.graph.mark_content_dirty(self.panel);
            return Ok(());
        }
        if self.gesture.is_some() {
            return Err("viewport gesture busy".into());
        }
        let v: Value = serde_json::from_str(message).map_err(|e| e.to_string())?;
        let stamp = Stamp {
            epoch: v["epoch"].as_u64().ok_or("message epoch")?,
            revision: v["revision"].as_u64().ok_or("message revision")?,
        };
        if stamp != self.editor.stamp {
            return Err("stale widget message".into());
        }
        if v["action"] == "cancel-binding" {
            if !self.project_pending || v["play_token"].as_u64() != Some(self.play_token) {
                return Err("stale binding cancellation".into());
            }
            self.editor.reject();
            self.project_pending = false;
            self.play_token = self
                .play_token
                .checked_add(1)
                .ok_or("binding generation exhausted")?;
            self.sync_ui();
            return Ok(());
        }
        if v["action"] == "cancel-import" && self.asset_pending {
            if v["payload"].as_u64() != Some(self.asset_token) {
                return Err("stale import token".into());
            }
            self.editor.reject();
            self.asset_pending = false;
            self.sync_ui();
            return Ok(());
        }
        if self.editor.pending() {
            return Err("pending transaction".into());
        }
        let action = v["action"].as_str().ok_or("message action")?;
        if action.starts_with("view3d-") {
            if v["view3d_revision"].as_u64() != Some(self.view3d.revision) {
                return Err("stale 3D view widget".into());
            }
            return self.view3d_command(action, &v["payload"]);
        }
        if action.starts_with("view-") {
            if v["view_revision"]
                .as_u64()
                .is_some_and(|revision| revision != self.view2d.revision)
            {
                return Err("stale view widget".into());
            }
            return self.view_command(action, &v["payload"]);
        }
        let p = &v["payload"];
        match action {
            "behavior-api-v1" => {
                self.select_behavior_api(crate::authoring::behavior::Api::V1)?;
            }
            "behavior-api-v5" => {
                self.select_behavior_api(crate::authoring::behavior::Api::MotionV5)?;
            }
            "behavior-api-character" => {
                self.select_behavior_api(crate::authoring::behavior::Api::CharacterV1)?;
            }
            "behavior-api-character3d" => {
                self.select_behavior_api(crate::authoring::behavior::Api::Character3V1)?;
            }
            "behavior-api-camera2d" => {
                self.select_behavior_api(crate::authoring::behavior::Api::Camera2V1)?;
            }
            "behavior-api-camera3d" => {
                self.select_behavior_api(crate::authoring::behavior::Api::Camera3V1)?;
            }
            "behavior-api-v4" => {
                self.select_behavior_api(crate::authoring::behavior::Api::SweepV4)?;
            }
            "behavior-api-v3" => {
                self.select_behavior_api(crate::authoring::behavior::Api::SegmentV3)?;
            }
            "behavior-api-v2" => {
                self.select_behavior_api(crate::authoring::behavior::Api::SpatialV2)?;
            }
            "behavior-bind" | "behavior-actors" | "behavior-detach" => {
                self.play_admission()?;
                if v["play_token"].as_u64() != Some(self.play_token) {
                    return Err("stale binding generation".into());
                }
                let (binding, code) = if action == "behavior-detach" {
                    (None, None)
                } else {
                    let code = if action == "behavior-bind" {
                        let path = crate::authoring::behavior::resolve(
                            Path::new(p.as_str().ok_or("source path")?),
                            self.storage.path.as_deref(),
                        )?;
                        crate::authoring::behavior_project::CodePayload::read_for(
                            &path,
                            self.ui().behavior_api,
                        )?
                    } else {
                        self.editor.code.clone().ok_or("no bound script")?
                    };
                    let binding = crate::authoring::behavior_project::Binding {
                        api: if action == "behavior-actors" {
                            self.editor
                                .document
                                .behavior
                                .as_ref()
                                .ok_or("no binding")?
                                .api
                                .clone()
                        } else {
                            self.ui().behavior_api.version().into()
                        },
                        source: code.identity().clone(),
                        actors: if action == "behavior-actors" && p.is_object() {
                            serde_json::from_value(p.clone()).map_err(|e| e.to_string())?
                        } else {
                            crate::authoring::behavior_project::Actors {
                                two_d: self.actors[0],
                                three_d: self.actors[1],
                            }
                        },
                    };
                    (Some(binding), Some(code))
                };
                self.project_pending =
                    self.editor.begin(stamp, Edit::Behavior { binding, code })?;
            }
            "import" => {
                if v["asset_generation"].as_u64() != Some(self.asset_token) {
                    return Err("stale import generation".into());
                }
                if v["selection"] != json!(self.editor.selection) {
                    return Err("stale asset target selection".into());
                }
                let next_token = self
                    .asset_token
                    .checked_add(1)
                    .ok_or("import token exhausted")?;
                let asset = crate::authoring::assets::import(
                    Path::new(p.as_str().ok_or("GLB path")?),
                    &self.active_assets()?,
                )?;
                let id = self.editor.next_id().ok_or("identity exhausted")?;
                let mut o = Object::new(id, Kind::Model);
                o.resource = Some(asset.identity.key());
                if let Some(g) = self
                    .editor
                    .document
                    .groups
                    .iter()
                    .find(|g| Some(g.id) == self.editor.selection)
                {
                    o.parent = Some(g.id);
                }
                self.editor.begin_source(
                    stamp,
                    Edit::AssetObject {
                        asset: Some(asset),
                        object: o,
                        create: true,
                    },
                    self.asset_source_for_probe
                        .as_deref()
                        .unwrap_or(crate::authoring::preview::SOURCE),
                )?;
                self.asset_pending = true;
                self.asset_token = next_token;
            }
            "asset-create" | "asset-replace" => {
                if v["selection"] != json!(self.editor.selection) {
                    return Err("stale asset target selection".into());
                }
                let key = p.as_str().ok_or("asset key")?;
                if !self.editor.document.assets.iter().any(|a| a.key() == key) {
                    return Err("asset not registered in document".into());
                }
                let create = action == "asset-create";
                let mut o = if create {
                    Object::new(
                        self.editor.next_id().ok_or("identity exhausted")?,
                        Kind::Model,
                    )
                } else {
                    self.editor
                        .document
                        .local_target(self.editor.selection.ok_or("select 3D leaf")?)
                        .ok_or("target missing")?
                };
                if !create
                    && !self
                        .editor
                        .document
                        .objects
                        .iter()
                        .any(|leaf| leaf.id == o.id && !leaf.kind.is_2d())
                {
                    return Err("select 3D leaf".into());
                }
                o.kind = Kind::Model;
                o.resource = Some(key.to_owned());
                if create {
                    if let Some(g) = self
                        .editor
                        .document
                        .groups
                        .iter()
                        .find(|g| Some(g.id) == self.editor.selection)
                    {
                        o.parent = Some(g.id);
                    }
                }
                self.editor.begin(
                    stamp,
                    Edit::AssetObject {
                        asset: None,
                        object: o,
                        create,
                    },
                )?;
            }
            "leave-cancel" => {
                self.leave = None;
            }
            "leave-save" | "leave-discard" => {
                if self.storage.unverified() {
                    self.storage.verify(&mut self.editor)?;
                }
                if action == "leave-save" {
                    let path = self
                        .storage
                        .path
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(&self.ui().path));
                    if self.storage.save(&mut self.editor, &path, Fault::None)? != Saved::Confirmed
                    {
                        return Err("save result pending verification".into());
                    }
                }
                let a = self.leave.take().ok_or("no pending navigation")?;
                self.perform_leave(a)?;
            }
            _ if self.leave.is_some() => return Err("unsaved decision pending".into()),
            "select" => {
                let old_selection = self.editor.selection;
                self.editor.select(Some(p.as_u64().ok_or("target")?))?;
                let prepared = match self.prepare(&self.editor.preview) {
                    Ok(p) => p,
                    Err(e) => {
                        self.editor.selection = old_selection;
                        return Err(e);
                    }
                };
                self.publish(prepared);
                self.blur();
            }
            "apply" => {
                let id = p["target"].as_u64().ok_or("target")?;
                let values = p["draft"].as_array().ok_or("draft")?;
                if values.len() != 14 {
                    return Err("draft fields".into());
                }
                let mut o = self.editor.document.local_target(id).ok_or("target")?;
                o.name = values[0].as_str().ok_or("name")?.into();
                let mut f = [0.0; 12];
                for i in 0..12 {
                    f[i] = values[i + 1]
                        .as_str()
                        .ok_or("number text")?
                        .parse::<f32>()
                        .map_err(|_| format!("invalid {}", FIELDS[i]))?;
                }
                o.translation.copy_from_slice(&f[0..3]);
                o.scale.copy_from_slice(&f[3..6]);
                o.yaw = f[6];
                o.pitch = f[7];
                o.rgba.copy_from_slice(&f[8..12]);
                o.visible = values[13]
                    .as_str()
                    .ok_or("visible")?
                    .parse()
                    .map_err(|_| "visible must be true/false")?;
                if let Some(g) = self.editor.document.groups.iter().find(|g| g.id == id) {
                    let proxy = self.editor.document.local_target(id).ok_or("target")?;
                    if o.scale != proxy.scale
                        || o.yaw != 0.0
                        || o.pitch != 0.0
                        || o.rgba != proxy.rgba
                    {
                        return Err("unsupported group property".into());
                    }
                    let mut g = g.clone();
                    g.name = o.name;
                    g.translation = o.translation;
                    g.visible = o.visible;
                    self.editor.begin(stamp, Edit::ReplaceGroup(g))?;
                } else {
                    self.editor.begin(stamp, Edit::Replace(o))?;
                }
            }
            "create" => {
                let kind: Kind = serde_json::from_value(p.clone()).map_err(|e| e.to_string())?;
                let id = self.editor.next_id().ok_or("identity exhausted")?;
                let mut o = Object::new(id, kind);
                if let Some(g) = self
                    .editor
                    .document
                    .groups
                    .iter()
                    .find(|g| Some(g.id) == self.editor.selection)
                {
                    o.parent = Some(g.id);
                }
                self.editor.begin(stamp, Edit::Create(o))?;
            }
            "toggle-visible" => {
                let id = p.as_u64().ok_or("target")?;
                if let Some(g) = self.editor.document.groups.iter().find(|g| g.id == id) {
                    let mut g = g.clone();
                    g.visible = !g.visible;
                    self.editor.begin(stamp, Edit::ReplaceGroup(g))?;
                } else {
                    let mut o = self.editor.document.local_target(id).ok_or("target")?;
                    o.visible = !o.visible;
                    self.editor.begin(stamp, Edit::Replace(o))?;
                }
            }
            "create-group" | "create-group2" | "create-group3" => {
                let dimension: Dimension =
                    serde_json::from_value(p.clone()).map_err(|e| e.to_string())?;
                let id = self.editor.next_id().ok_or("identity exhausted")?;
                let mut g = Group::new(id, dimension);
                if let Some(parent) = self
                    .editor
                    .document
                    .groups
                    .iter()
                    .find(|g| Some(g.id) == self.editor.selection)
                {
                    g.parent = Some(parent.id);
                }
                self.editor.begin(stamp, Edit::CreateGroup(g))?;
            }
            "parent-choice" => {
                if p["target"].as_u64() != self.editor.selection {
                    return Err("stale selection".into());
                }
                let parent = parse_parent(&p["parent"])?;
                if parent.is_some_and(|id| !self.editor.document.groups.iter().any(|g| g.id == id))
                {
                    return Err("parent missing".into());
                }
                self.ui_mut().parent_choice = parent;
                self.rt.graph.mark_content_dirty(self.panel);
                return Ok(());
            }
            "reparent" => {
                let target = p["target"].as_u64().ok_or("target")?;
                let expected_parent = parse_parent(&p["expected_parent"])?;
                let parent = parse_parent(&p["parent"])?;
                self.editor.begin(
                    stamp,
                    Edit::Reparent {
                        target,
                        expected_parent,
                        parent,
                    },
                )?;
            }
            "delete-request" => {
                let id = p.as_u64().ok_or("select a node")?;
                if self.editor.selection != Some(id) {
                    return Err("stale selection".into());
                }
                if self.editor.document.groups.iter().any(|g| g.id == id) {
                    self.ui_mut().delete_confirmation = Some(id);
                    self.rt.graph.mark_content_dirty(self.panel);
                    return Ok(());
                }
                self.editor.begin(stamp, Edit::Delete(id))?;
            }
            "delete-cancel" => {
                self.ui_mut().delete_confirmation = None;
                self.rt.graph.mark_content_dirty(self.panel);
                return Ok(());
            }
            "delete-confirm" | "delete" => {
                let id = p.as_u64().ok_or("select a node")?;
                if self.editor.document.groups.iter().any(|g| g.id == id)
                    && (action != "delete-confirm"
                        || self.ui().delete_confirmation != Some(id)
                        || self.editor.selection != Some(id))
                {
                    return Err("confirm subtree deletion".into());
                }
                self.editor.begin(stamp, Edit::Delete(id))?;
            }
            "undo" => {
                self.editor.begin(stamp, Edit::Undo)?;
            }
            "redo" => {
                self.editor.begin(stamp, Edit::Redo)?;
            }
            "cancel" => {}
            "open" => self.request_leave(Leave::Open(PathBuf::from(p.as_str().ok_or("path")?))),
            "new" => self.request_leave(Leave::New),
            "save" | "saveas" => {
                let path = if action == "save" {
                    self.storage
                        .path
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(p.as_str().unwrap_or("")))
                } else {
                    PathBuf::from(p.as_str().ok_or("path")?)
                };
                if self.storage.save(&mut self.editor, &path, Fault::None)? != Saved::Confirmed {
                    return Err("save result pending verification".into());
                }
            }
            _ => return Err("unknown command".into()),
        }
        self.sync_ui();
        Ok(())
    }
    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        if self.play.is_some() {
            return Err("scene play busy".into());
        }
        if self.gesture.is_some() || self.editor.pending() {
            return Err("busy".into());
        }
        if self.editor.dirty() || self.storage.unverified() {
            return Err("unsaved decision required".into());
        }
        let (doc, assets, code, s) = Storage::load_project(path, &self.active_assets()?)?;
        self.editor.begin_open_project(doc, assets, code)?;
        self.opening = Some(s);
        self.sync_ui();
        Ok(())
    }
    pub fn settle(&mut self) -> Result<(), String> {
        for _ in 0..10000 {
            self.frame(None)?;
            if !self.editor.pending() && self.gesture.is_none() {
                return Ok(());
            }
        }
        Err("settle deadline".into())
    }
}
