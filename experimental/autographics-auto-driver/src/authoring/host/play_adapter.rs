//! Single-Play host wiring and source admission.
use super::*;

impl Host {
    fn begin_audio_run(&mut self, run: i32) -> Result<(), String> {
        if self
            .audio
            .as_ref()
            .is_none_or(|a| a.mixer.lock().unwrap().fault().is_some())
        {
            // Release a failed stream before opening its replacement: only one
            // native manager is resident, including at round-restart boundaries.
            self.audio = None;
            self.audio = Some(crate::authoring::public_audio::native::Native::open()?);
        }
        self.audio
            .as_ref()
            .unwrap()
            .mixer
            .lock()
            .unwrap()
            .begin(run)
    }
    pub(super) fn sync_play_ui(&mut self) {
        if self.play.is_none() {
            for (i, two) in [true, false].into_iter().enumerate() {
                if !self.actors[i].is_some_and(|id| {
                    self.editor
                        .document
                        .objects
                        .iter()
                        .any(|o| o.id == id && o.kind.is_2d() == two)
                }) {
                    self.actors[i] = self
                        .editor
                        .document
                        .objects
                        .iter()
                        .find(|o| o.kind.is_2d() == two)
                        .map(|o| o.id);
                }
            }
        }
        let state = self.play.as_ref().map(|p| format!("{:?}", p.state));
        let tick = self.play.as_ref().map_or(0, |p| p.tick());
        let token = self.play_token;
        let actors = self.actors;
        let error = self.play.as_ref().and_then(|p| p.error.clone());
        let resolved = if self.project_default {
            self.editor
                .document
                .behavior
                .as_ref()
                .map(|b| {
                    format!(
                        "Project: {} {} actors {:?}",
                        b.api,
                        b.source.sha256,
                        b.actors.ids()
                    )
                })
                .unwrap_or_else(|| "Project: Builtin6".into())
        } else if self.ui().behavior_user {
            crate::authoring::behavior::resolve(
                Path::new(&self.ui().behavior_path),
                self.storage.path.as_deref(),
            )
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| e)
        } else {
            "Builtin6".into()
        };
        let binding_busy = self.project_pending;
        let project_default = self.project_default;
        let ui = self.ui_mut();
        let changed = ui.project_default != project_default
            || ui.binding_busy != binding_busy
            || ui.play_state != state
            || ui.behavior_resolved != resolved
            || ui.play_tick != tick
            || ui.play_token != token
            || ui.actors != actors;
        ui.project_default = project_default;
        ui.binding_busy = binding_busy;
        ui.play_state = state;
        ui.play_tick = tick;
        ui.play_token = token;
        ui.actors = actors;
        ui.behavior_resolved = resolved;
        if let Some(error) = error {
            ui.error = error;
        }
        if changed {
            self.rt.graph.mark_content_dirty(self.panel);
        }
    }
    pub fn start_play(&mut self, source: &str) -> Result<(), String> {
        self.start_program(source, None)
    }
    pub(super) fn play_admission(&self) -> Result<(), String> {
        if self.play.is_some()
            || self.editor.pending()
            || self.gesture.is_some()
            || self.asset_pending
            || self.leave.is_some()
            || self.storage.unverified()
        {
            return Err("Run requires idle accepted editor".into());
        }
        if self.ui().draft != self.ui().original_draft {
            return Err("Apply or Cancel property draft before Run".into());
        }
        Ok(())
    }
    pub fn select_behavior_api(
        &mut self,
        api: crate::authoring::behavior::Api,
    ) -> Result<(), String> {
        self.play_admission()?;
        self.ui_mut().behavior_api = api;
        Ok(())
    }
    pub fn select_behavior(&mut self, path: Option<String>) -> Result<(), String> {
        self.play_admission()?;
        if path.as_ref().is_some_and(|p| p.len() > 4096) {
            return Err("behavior path exceeds 4096 bytes".into());
        }
        self.project_default = false;
        self.ui_mut().behavior_user = path.is_some();
        if let Some(path) = path {
            self.ui_mut().behavior_path = path;
        }
        self.sync_play_ui();
        Ok(())
    }
    pub fn start_selected_play(&mut self) -> Result<(), String> {
        self.play_admission()?;
        if self.project_default && self.editor.document.behavior.is_some() {
            let code = self
                .editor
                .code
                .as_ref()
                .ok_or("missing accepted behavior payload")?;
            crate::authoring::behavior_project::validate_payload(
                &self.editor.document,
                Some(code),
            )?;
            let path = self.storage.source_origin(code.identity());
            let source = code.snapshot_for(
                &path,
                crate::authoring::behavior::Api::parse(
                    &self.editor.document.behavior.as_ref().unwrap().api,
                )?,
            )?;
            self.start_program("", Some(source))
        } else if self.ui().behavior_user && !self.project_default {
            let path = crate::authoring::behavior::resolve(
                Path::new(&self.ui().behavior_path),
                self.storage.path.as_deref(),
            )?;
            let api = self.ui().behavior_api;
            let source = crate::authoring::behavior::SourceSnapshot::read_for(&path, api)?;
            self.start_program("", Some(source))
        } else {
            let source = self
                .play_source_for_probe
                .clone()
                .unwrap_or_else(|| crate::authoring::play::SOURCE.into());
            self.start_play(&source)
        }
    }
    pub fn start_user_snapshot(
        &mut self,
        snapshot: crate::authoring::behavior::SourceSnapshot,
    ) -> Result<(), String> {
        self.play_admission()?;
        self.start_program("", Some(snapshot))
    }
    pub(super) fn start_program(
        &mut self,
        source: &str,
        snapshot: Option<crate::authoring::behavior::SourceSnapshot>,
    ) -> Result<(), String> {
        let begin = Instant::now();
        if self.play.is_some()
            || self.editor.pending()
            || self.gesture.is_some()
            || self.asset_pending
            || self.leave.is_some()
            || self.storage.unverified()
        {
            return Err("Run requires idle accepted editor".into());
        }
        if self.ui().draft != self.ui().original_draft {
            return Err("Apply or Cancel property draft before Run".into());
        }
        if self.navigation.is_some() || self.navigation3d.is_some() {
            return Err("2D navigation busy".into());
        }
        let restore = self.prepare(&self.editor.preview)?;
        let actors = if self.project_default {
            self.editor
                .document
                .behavior
                .as_ref()
                .map_or(self.actors, |b| b.actors.ids())
        } else {
            self.actors
        };
        let mut play = if let Some(snapshot) = snapshot {
            if snapshot.version != crate::authoring::behavior::VERSION {
                // Frozen construction envelope for the object profiles: every
                // reserved slot pays its worst eligible template mesh count,
                // hidden or not; the whole candidate is rejected before a Run
                // exists (contract section 4; prepare re-checks per frame).
                if let Ok(api) = crate::authoring::behavior::Api::parse(&snapshot.version) {
                    if api.objects().is_some() {
                        let mut base_potential = 0usize;
                        let mut worst_template = 0usize;
                        for o in &self.editor.document.objects {
                            if o.kind.is_2d() {
                                continue;
                            }
                            let meshes = self
                                .model(o)
                                .ok_or("missing resource payload")?
                                .meshes
                                .len();
                            base_potential += meshes;
                            if matches!(
                                o.kind,
                                Kind::Cylinder | Kind::Player | Kind::Mob | Kind::Model
                            ) {
                                worst_template = worst_template.max(meshes);
                            }
                        }
                        let slots = crate::authoring::runtime_objects::SLOTS;
                        let total = base_potential + worst_template * slots;
                        if total > agp::render3d::pipeline::MAX_INSTANCES {
                            return Err(format!(
                                "objects mesh envelope: base {base_potential} + {slots} slots x worst template {worst_template} exceeds 512"
                            ));
                        }
                    }
                }
                let descriptor =
                    crate::authoring::spatial::Descriptor::build(&self.editor.document, |o| {
                        self.model(o)
                    })?;
                crate::authoring::play::Play::new_user_spatial(
                    &self.editor,
                    actors,
                    snapshot,
                    descriptor,
                )?
            } else {
                crate::authoring::play::Play::new_user(&self.editor, actors, snapshot)?
            }
        } else {
            crate::authoring::play::Play::new(&self.editor, actors, source)?
        };
        play.vm_output_measurement
            .enable(self.prepare_measurement.enabled);
        play.admission_measurement
            .enable(self.prepare_measurement.enabled);
        if let Some(s) = &play.user_source {
            self.ui_mut().last_behavior = format!(
                "Last Run: {} {} {}",
                s.path.display(),
                s.version,
                s.user_hash
            );
        } else {
            self.ui_mut().last_behavior = format!("Last Run: Builtin6 {}", play.source_hash);
        }
        let editing_view = self.view2d;
        let editing_view3d = self.view3d;
        self.install_3d(Default::default())?;
        self.view2d = Default::default();
        let runtime_view = self.prepare(&self.editor.preview);
        self.view2d = editing_view;
        let runtime_view = match runtime_view {
            Ok(p) => p,
            Err(e) => {
                self.install_3d(editing_view3d)?;
                return Err(e);
            }
        };
        if play.audio_cues.is_some() {
            if let Err(error) = self.begin_audio_run(play.run_id()) {
                self.install_3d(editing_view3d)?;
                return Err(error);
            }
        }
        self.play_restore = Some(restore);
        self.play = Some(play);
        self.publish(runtime_view);
        self.input.window_focus_lost();
        self.events.clear();
        self.ui_mut().queue.clear();
        self.rt.release_iced_focus(self.panel);
        self.ui_mut().error.clear();
        self.sync_play_ui();
        bounded_time(&mut self.play_start_us, begin.elapsed().as_micros());
        Ok(())
    }
    pub fn stop_play(&mut self) {
        if let Some(audio) = &self.audio {
            audio.mixer.lock().unwrap().stop();
        }
        self.cancel_camera_drag();
        if let Some(mut play) = self.play.take() {
            let begin = Instant::now();
            play.stop();
            self.play_token = self.play_token.saturating_add(1);
            let mut restore_error = None;
            if let Some(restore) = self.play_restore.take() {
                restore_error = self.install_3d(self.view3d).err();
                self.publish(restore);
            }
            self.presented = self.editor.preview.objects.clone();
            self.presented_nodes = self.editor.preview.nodes.clone();
            self.presented_stamp = self.editor.stamp;
            self.events.clear();
            self.input.window_focus_lost();
            self.rt.release_iced_focus(self.panel);
            self.ui_mut().queue.clear();
            self.ui_mut().error = restore_error.unwrap_or_default();
            self.sync_ui();
            bounded_time(&mut self.play_stop_us, begin.elapsed().as_micros());
        }
    }
    pub fn pause_play(&mut self, reason: &str) {
        self.cancel_camera_drag();
        if let Some(p) = &mut self.play {
            p.input.clear();
            p.pause(reason);
            if let Some(audio) = &self.audio {
                if p.audio_cues.is_some() {
                    let _ = audio.mixer.lock().unwrap().pause(p.run_id(), true);
                }
            }
        }
        self.sync_play_ui();
    }
    pub(super) fn dispatch_play(&mut self, v: &Value) -> Result<(), String> {
        if v["epoch"].as_u64() != Some(self.editor.stamp.epoch)
            || v["revision"].as_u64() != Some(self.editor.stamp.revision)
            || v["play_token"].as_u64() != Some(self.play_token)
        {
            return Err("stale play command token".into());
        }
        match v["action"].as_str().unwrap_or("") {
            "play-project-default" => {
                self.play_admission()?;
                self.project_default = true;
            }
            "play-run" => {
                self.start_selected_play()?;
            }
            "play-source-toggle" => {
                let path = if self.ui().behavior_user {
                    None
                } else {
                    Some(self.ui().behavior_path.clone())
                };
                self.select_behavior(path)?;
            }
            "play-source-path" => {
                let path = v["payload"]
                    .as_str()
                    .ok_or("behavior path string")?
                    .to_string();
                self.select_behavior(Some(path))?;
            }
            "play-stop" => {
                if self.play.is_none() {
                    return Err("no running world".into());
                }
                self.stop_play();
                return Ok(());
            }
            "play-pause" => {
                let p = self.play.as_mut().ok_or("no running world")?;
                if p.state != crate::authoring::play::State::Running {
                    return Err("pause requires Running".into());
                }
                p.pause("");
            }
            "play-resume" => self.play.as_mut().ok_or("no running world")?.resume()?,
            "play-step" => self
                .play
                .as_mut()
                .ok_or("no running world")?
                .single_step()?,
            action @ ("play-actor2" | "play-actor3") => {
                if self.play.is_some()
                    || self.editor.pending()
                    || self.gesture.is_some()
                    || self.leave.is_some()
                {
                    return Err("actor choice requires Editing".into());
                }
                let i = usize::from(action == "play-actor3");
                let candidates: Vec<_> = self
                    .editor
                    .document
                    .objects
                    .iter()
                    .filter(|o| o.kind.is_2d() == (i == 0))
                    .map(|o| o.id)
                    .collect();
                if let Some(current) =
                    self.actors[i].and_then(|id| candidates.iter().position(|v| *v == id))
                {
                    self.actors[i] = Some(candidates[(current + 1) % candidates.len()]);
                } else {
                    self.actors[i] = candidates.first().copied();
                }
            }
            _ => return Err("unknown play command".into()),
        }
        if let (Some(audio), Some(p)) = (&self.audio, &self.play) {
            if p.audio_cues.is_some() {
                audio
                    .mixer
                    .lock()
                    .unwrap()
                    .pause(p.run_id(), p.audio_paused())?;
            }
        }
        self.play_token = self
            .play_token
            .checked_add(1)
            .ok_or("play token exhausted")?;
        self.ui_mut().error.clear();
        self.sync_play_ui();
        Ok(())
    }
    pub(super) fn pump_play(&mut self, now: Instant) {
        let Some(mut play) = self.play.take() else {
            return;
        };
        if matches!(
            play.state,
            crate::authoring::play::State::Faulted | crate::authoring::play::State::Stopping
        ) {
            self.play = Some(play);
            return;
        }
        let deadline = now + std::time::Duration::from_millis(8);
        let mut round_requested = false;
        let result = (|| {
            if play.base_identity["stamp"]
                != json!([self.editor.stamp.epoch, self.editor.stamp.revision])
            {
                return Err("accepted editor changed during play".into());
            }
            if play.audio_cues.is_some() {
                let audio = self
                    .audio
                    .as_ref()
                    .ok_or("audio_unavailable: Host manager")?;
                if let Some(error) = audio.mixer.lock().unwrap().fault() {
                    return Err(error.to_owned());
                }
            }
            play.clock(now);
            if let Some(audio) = &self.audio {
                if play.audio_cues.is_some() {
                    audio
                        .mixer
                        .lock()
                        .unwrap()
                        .pause(play.run_id(), play.audio_paused())?;
                }
            }
            let mut dispatched = 0;
            loop {
                play.pump_until(deadline)?;
                if let Some(frame) = play.candidate() {
                    let measurement_start = self.prepare_measurement.start();
                    let prepared = self.prepare_frame(
                        &frame.document,
                        &frame.objects,
                        &frame.nodes,
                        frame.camera.as_ref(),
                        frame.instances.as_ref(),
                        frame.sprites.as_ref(),
                        frame.animations.as_ref(),
                    );
                    self.prepare_measurement.finish(measurement_start);
                    let prepared = prepared?;
                    // All preparation and transaction limits precede visible publication.
                    if let Some(packet) = frame.audio.clone() {
                        let audio = self
                            .audio
                            .as_ref()
                            .ok_or("audio_unavailable: Host manager")?;
                        let mut mixer = audio.mixer.lock().unwrap();
                        let cues = play
                            .audio_cues
                            .as_ref()
                            .ok_or("audio_invalid: frozen cues")?
                            .clone();
                        let step = play.is_neutral_step();
                        let disposition =
                            mixer.accept_inflight(packet, &cues, step, || play.accept())?;
                        play.last.as_mut().unwrap().audio_disposition = Some(disposition);
                        mixer.pause(play.run_id(), play.audio_paused())?;
                    } else {
                        play.accept()?;
                    }
                    self.camera_saturation = play.camera_saturation();
                    let frame = play.last.as_ref().unwrap();
                    self.presented = frame.objects.clone();
                    self.presented_nodes = frame.nodes.clone();
                    self.publish(prepared);
                    // The restart marker is consumed only after the carrying
                    // candidate was fully accepted, and only while Running:
                    // Starting/Paused/step ticks never process a new round.
                    if play.state == crate::authoring::play::State::Running
                        && frame.rounds.as_ref().is_some_and(|r| r.restart_requested)
                    {
                        round_requested = true;
                        break;
                    }
                }
                if Instant::now() >= deadline || dispatched >= 4 || !play.dispatch_credit()? {
                    break;
                }
                dispatched += 1;
            }
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            if let Some(audio) = &self.audio {
                if play.audio_cues.is_some() {
                    audio.mixer.lock().unwrap().stop();
                }
            }
            play.fail(error);
        }
        self.play = Some(play);
        if round_requested
            && self
                .play
                .as_ref()
                .is_some_and(|p| p.state == crate::authoring::play::State::Running)
        {
            self.restart_round();
        }
        self.sync_play_ui();
        bounded_time(&mut self.play_pump_us, now.elapsed().as_micros());
    }

    /// The audited round restart (PLAN-002 contract section 6): terminate the
    /// old Run on the single scheduling stack, then rebuild a new Run/VM from
    /// the same accepted source bytes, actors and configuration. The editing
    /// `play_restore` and the last-good presentation stay untouched, so a
    /// failed new round keeps the old complete presentation under a named
    /// Fault and Stop still recovers the editing Prepared.
    fn restart_round(&mut self) {
        self.cancel_camera_drag();
        let Some(mut old) = self.play.take() else {
            return;
        };
        if old.audio_cues.is_some() {
            if let Some(audio) = &self.audio {
                audio.mixer.lock().unwrap().stop();
            }
        }
        let built = (|| -> Result<crate::authoring::play::Play, String> {
            let snapshot = old
                .user_source
                .clone()
                .ok_or("round restart requires an accepted user snapshot")?;
            let descriptor =
                crate::authoring::spatial::Descriptor::build(&self.editor.document, |o| {
                    self.model(o)
                })?;
            let fresh = crate::authoring::play::Play::new_user_spatial(
                &self.editor,
                old.actors,
                snapshot,
                descriptor,
            )?;
            if fresh.audio_cues.is_some() {
                self.begin_audio_run(fresh.run_id())?;
            }
            Ok(fresh)
        })();
        match built {
            Ok(mut fresh) => {
                old.stop();
                fresh
                    .vm_output_measurement
                    .enable(self.prepare_measurement.enabled);
                fresh
                    .admission_measurement
                    .enable(self.prepare_measurement.enabled);
                self.events.clear();
                self.input.window_focus_lost();
                self.rt.release_iced_focus(self.panel);
                self.ui_mut().queue.clear();
                self.ui_mut().error.clear();
                self.play = Some(fresh);
            }
            Err(error) => {
                old.stop();
                old.fail(format!("round restart: {error}"));
                // The old last-good frame stays published and installed; only
                // the VM is gone and the state is the named Fault.
                self.play = Some(old);
            }
        }
    }
}

#[cfg(test)]
mod audio_restart_tests {
    use super::*;
    #[test]
    fn preaccept_audio_start_failure_restores_editing_view_and_keeps_host_idle() {
        use crate::authoring::behavior::{Api, SourceSnapshot};
        let mut doc = Document::empty("audio-start-refusal".into());
        doc.schema = "autographics.scene/7".into();
        doc.cues = Some(vec![]);
        let mut actor = Object::new(1, Kind::Quad);
        actor.scale = [0.8, 1.0, 1.0];
        doc.objects.push(actor);
        let mut host = Host::with_assets(doc, Default::default()).unwrap();
        let mut view = host.view3d;
        view.sample_default = false;
        view.yaw = 0.4;
        host.set_view3d(view).unwrap();
        let view = host.view3d;
        let camera = host.camera();
        host.audio = Some(crate::authoring::public_audio::native::Native::open().unwrap());
        // Exercise the real admission error after runtime-view preparation,
        // without disconnecting the user's device or substituting a fake sink.
        host.audio
            .as_ref()
            .unwrap()
            .mixer
            .lock()
            .unwrap()
            .begin(i32::MAX)
            .unwrap();
        let source = SourceSnapshot::from_bytes_for(
            Path::new("start-refusal.at"),
            b"fn app_start(){}\nfn app_update(dt float){}\n".to_vec(),
            Api::Audio2V1,
        )
        .unwrap();
        let error = host.start_user_snapshot(source).unwrap_err();
        assert!(error.contains("stale/exhausted Run"), "{error}");
        assert_eq!(
            host.view3d, view,
            "preaccept failure restores editing camera"
        );
        assert_eq!(
            host.camera(),
            camera,
            "installed GPU camera must match editing state"
        );
        assert!(host.play.is_none());
        assert!(host.play_restore.is_none());
    }
    #[test]
    fn round_restart_reopens_faulted_stream_and_invalidates_old_run_owner() {
        use crate::authoring::behavior::{Api, SourceSnapshot};
        let mut doc = Document::empty("restart-audio".into());
        doc.schema = "autographics.scene/7".into();
        doc.cues = Some(vec![]);
        let mut actor = Object::new(1, Kind::Quad);
        actor.scale = [0.8, 1.0, 1.0];
        doc.objects.push(actor);
        let mut host = Host::with_assets(doc, Default::default()).unwrap();
        let source=b"var ok=0\nfn app_start(){ok=ag_char_bind(1);ok=ag_camera_follow_2d(1,-10.0,-10.0,10.0,10.0,4.0,4.0)}\nfn app_update(dt float){}\n";
        let source =
            SourceSnapshot::from_bytes_for(Path::new("restart.at"), source.to_vec(), Api::Audio2V1)
                .unwrap();
        host.start_user_snapshot(source).unwrap();
        host.pause_play("");
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        while host.play.as_ref().unwrap().pending() {
            assert!(Instant::now() < deadline);
            host.frame(None).unwrap();
        }
        assert!(
            host.play.as_ref().unwrap().error.is_none(),
            "{:?}",
            host.play.as_ref().unwrap().error
        );
        let run = host.play.as_ref().unwrap().run_id();
        let owner = host.play.as_ref().unwrap().owner_probe().unwrap();
        let old = std::sync::Arc::downgrade(&host.audio.as_ref().unwrap().mixer);
        host.audio
            .as_ref()
            .unwrap()
            .mixer
            .lock()
            .unwrap()
            .output_fault("restart-boundary disconnect".into());
        host.restart_round();
        assert!(
            old.upgrade().is_none(),
            "failed stream must be dropped, not cleared by Mixer::begin"
        );
        assert!(!owner(), "old VM terminated at round boundary");
        let fresh = host.play.as_ref().unwrap();
        assert!(fresh.run_id() > run);
        assert_ne!(fresh.state, crate::authoring::play::State::Faulted);
        let mut m = host.audio.as_ref().unwrap().mixer.lock().unwrap();
        assert_eq!(m.fault(), None);
        assert_eq!(m.run(), fresh.run_id());
        assert!(m.pause(run, false).is_err());
    }
}
