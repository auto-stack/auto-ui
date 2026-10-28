//! Profile 2 presentation/input adapter. Generic input routing and GPU/audio stay host-owned.
use crate::{
    host::Host,
    squash::{Object3d, SquashScript},
};
use agp::{
    assets::{load_glb_bytes, ModelData},
    core::scene::Color,
    games::squash::live::{LiveEffect, LiveKey, LiveSquashInput},
    host::surface_runtime::{Scene3dContent, SurfaceContent, TextOverlay},
    render3d::{MeshInstance, MeshPipeline, MeshStore},
    renderer::target::OffscreenTarget,
};
use glam::Mat4;
fn validate_fixed_resource(kind: &str, bytes: &[u8]) -> Result<(), String> {
    let expected = match kind {
        "player" => "05d8588e8032a674e309ccb4d04578fb578b8da1629096b1036f628593801942",
        "mob" => "8a82adb5bfba8c22f9c1a0d692e4d25c2f9055cb7e6f6acd6f61ac169079818c",
        "font" => "92b3d3c6e135eb1dc95f88e6ca75bd6113d9eb3261a95ca39f733e3897e53675",
        "music" => "32680f2b2631eb9f6e2902ebbbec1b43ab82b4dbf154fdc9bfbfa17b03270504",
        _ => return Err("fixed resource kind".into()),
    };
    if agp::renderer::capture::sha256_hex(bytes) != expected {
        return Err(format!("fixed resource identity: {kind}"));
    }
    Ok(())
}
pub struct Music {
    _manager: kira::manager::AudioManager,
    handle: kira::sound::static_sound::StaticSoundHandle,
    pub decoded_bytes: usize,
    pub duration_secs: f64,
    pub loop_enabled: bool,
}
impl Music {
    pub fn position_secs(&self) -> f64 {
        self.handle.position()
    }
    pub fn is_playing(&self) -> bool {
        self.handle.state() == kira::sound::PlaybackState::Playing
    }
}
impl Drop for Music {
    fn drop(&mut self) {
        let _ = self.handle.stop(kira::tween::Tween::default());
    }
}
pub struct SquashRuntime {
    pub consumer_publications: Vec<crate::squash::Publication3d>,
    pub events: std::collections::VecDeque<(u64, u64, [i32; 4])>,
    pub scripts: Vec<SquashScript>,
    pub inputs: Vec<LiveSquashInput>,
    pub accepted: Vec<Option<crate::squash::Packet3d>>,
    models: Vec<ModelData>,
    environment: Vec<MeshInstance>,
    pub music_plays: u64,
    audio: Option<std::sync::Arc<Music>>,
    source: String,
}
impl SquashRuntime {
    pub fn prepare(host: &mut Host, source: &str) -> Result<Self, String> {
        let mut scripts = vec![SquashScript::new(source)?, SquashScript::new(source)?];
        for s in &mut scripts {
            s.settle()?;
        }
        validate_fixed_resource(
            "player",
            include_bytes!("../../autographics-prototype/assets/squash_the_creeps/art/player.glb"),
        )?;
        validate_fixed_resource(
            "mob",
            include_bytes!("../../autographics-prototype/assets/squash_the_creeps/art/mob.glb"),
        )?;
        validate_fixed_resource(
            "font",
            include_bytes!(
                "../../autographics-prototype/assets/squash_the_creeps/fonts/Montserrat-Medium.ttf"
            ),
        )?;
        let player = load_glb_bytes(include_bytes!(
            "../../autographics-prototype/assets/squash_the_creeps/art/player.glb"
        ))
        .map_err(|e| e.to_string())?;
        let mob = load_glb_bytes(include_bytes!(
            "../../autographics-prototype/assets/squash_the_creeps/art/mob.glb"
        ))
        .map_err(|e| e.to_string())?;
        fn procedural(mesh: agp::assets::MeshData) -> ModelData {
            let mut bytes = vec![];
            for p in &mesh.positions {
                for v in p {
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
            }
            for i in &mesh.indices {
                bytes.extend_from_slice(&i.to_le_bytes());
            }
            for v in mesh.material.base_color {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            ModelData {
                source_sha256: agp::renderer::capture::sha256_hex(&bytes),
                meshes: vec![mesh],
            }
        }
        let ground = procedural(agp::render3d::ground_mesh());
        let cylinder = procedural(agp::render3d::cylinder_mesh());
        let mut environment = vec![MeshInstance {
            mesh: (ground.source_sha256.clone(), ground.meshes[0].name.clone()),
            model: Mat4::IDENTITY,
        }];
        for position in agp::render3d::CYLINDER_POSITIONS {
            environment.push(MeshInstance {
                mesh: (
                    cylinder.source_sha256.clone(),
                    cylinder.meshes[0].name.clone(),
                ),
                model: Mat4::from_translation(position.into()),
            });
        }
        let models = vec![player, mob, ground, cylinder];
        let mut contents = vec![];
        for _ in 0..2 {
            let mut store = MeshStore::default();
            for model in &models {
                for i in 0..model.meshes.len() {
                    store
                        .get_or_upload(host.rt.gpu(), model, i)
                        .map_err(|e| format!("mesh upload: {e:?}"))?;
                }
            }
            let text=agp::render2d::TextRenderer::new(host.rt.gpu(),OffscreenTarget::FORMAT,include_bytes!("../../autographics-prototype/assets/squash_the_creeps/fonts/Montserrat-Medium.ttf"),32.0,"Score: 0123456789Prs pantEy").map_err(|e|format!("font: {e:?}"))?;
            contents.push(Scene3dContent {
                screen_overlay: Some(
                    agp::host::surface_runtime::ScreenOverlay::new(host.rt.gpu())
                        .map_err(|e| format!("overlay: {e:?}"))?,
                ),
                text_overlay: Some(TextOverlay {
                    renderer: text,
                    glyphs: vec![],
                }),
                mesh: MeshPipeline::new(host.rt.gpu(), OffscreenTarget::FORMAT)
                    .map_err(|e| format!("mesh pipeline: {e:?}"))?,
                store,
                instances: environment.clone(),
                sky: [0.64625, 0.65575, 0.67075, 1.0],
                view_proj: agp::render3d::Camera3d::sample_camera(1.5).view_proj(),
            });
        }
        // Prepare every resource before replacing visible content.
        for (surface, content) in host.game_surfaces.iter().zip(contents) {
            *host.rt.content_mut(*surface).unwrap() = SurfaceContent::Scene3d(Box::new(content));
            host.rt.graph.mark_content_dirty(*surface);
        }
        let consumer_publications = scripts
            .iter()
            .map(|s| crate::squash::Publication3d::new(s.id()))
            .collect();
        Ok(Self {
            consumer_publications,
            events: std::collections::VecDeque::new(),
            scripts,
            inputs: vec![LiveSquashInput::default(), LiveSquashInput::default()],
            accepted: vec![None, None],
            models,
            environment,
            music_plays: 0,
            audio: None,
            source: source.to_owned(),
        })
    }
    pub fn enable_audio(&mut self) -> Result<(), String> {
        if self.audio.is_some() {
            return Ok(());
        }
        validate_fixed_resource("music",include_bytes!("../../autographics-prototype/assets/squash_the_creeps/art/House In a Forest Loop.ogg"))?;
        let mut manager =
            kira::manager::AudioManager::new(kira::manager::AudioManagerSettings::default())
                .map_err(|e| format!("music backend: {e:?}"))?;
        let data=kira::sound::static_sound::StaticSoundData::from_cursor(std::io::Cursor::new(include_bytes!("../../autographics-prototype/assets/squash_the_creeps/art/House In a Forest Loop.ogg").to_vec())).map_err(|e|format!("music: {e:?}"))?.loop_region(..);
        let decoded_bytes = std::mem::size_of_val(data.frames.as_ref());
        let duration_secs = data.duration().as_secs_f64();
        let loop_enabled = data.settings.loop_region.is_some();
        let handle = manager
            .play(data)
            .map_err(|e| format!("music play: {e:?}"))?;
        self.music_plays += 1;
        self.audio = Some(std::sync::Arc::new(Music {
            _manager: manager,
            handle,
            decoded_bytes,
            duration_secs,
            loop_enabled,
        }));
        Ok(())
    }
    pub fn resource_metrics(&self, host: &Host) -> serde_json::Value {
        let mut parts:Vec<_>=host.game_surfaces.iter().filter_map(|id|match host.rt.content(*id){Some(SurfaceContent::Scene3d(c))=>Some(serde_json::json!({"overlay_buffers":c.screen_overlay.as_ref().map_or(0,|o|o.quad.owned_buffer_bytes()),"mesh_buffers":c.store.owned_buffer_bytes(),"mesh_uploaded":c.store.uploads.snapshot(),"pipeline_buffers":c.mesh.owned_buffer_bytes(),"depth_shadow_textures":c.mesh.owned_depth_texture_bytes(),"glyph_buffers":c.text_overlay.as_ref().map_or(0,|t|t.renderer.owned_buffer_bytes()),"glyph_atlas":c.text_overlay.as_ref().map_or(0,|t|t.renderer.owned_atlas_bytes()),"glyph_texture_upload":c.text_overlay.as_ref().map_or(0,|t|t.renderer.texture_upload_bytes)})),_=>None}).collect();
        for part in &mut parts {
            let total = [
                "overlay_buffers",
                "mesh_buffers",
                "pipeline_buffers",
                "depth_shadow_textures",
                "glyph_buffers",
                "glyph_atlas",
            ]
            .iter()
            .map(|key| part[*key].as_u64().unwrap())
            .sum::<u64>();
            part["owned_bytes"] = serde_json::json!(total);
        }
        let selected_total = parts
            .iter()
            .map(|p| p["owned_bytes"].as_u64().unwrap())
            .sum::<u64>()
            + self.audio.as_ref().map_or(0, |m| m.decoded_bytes as u64)
            + host.rt.owned_target_bytes();
        serde_json::json!({"selected_owned_total":selected_total,"profile2_surfaces":parts,"music_decoded_bytes":self.audio.as_ref().map_or(0,|m|m.decoded_bytes),"music_source_bytes":include_bytes!("../../autographics-prototype/assets/squash_the_creeps/art/House In a Forest Loop.ogg").len(),"target_bytes_all_surfaces":host.rt.owned_target_bytes(),"buffer_uploads":host.rt.upload_stats(),"boundary":"owned explicit profile2 buffers/textures/audio plus all offscreen targets; excludes driver residency, Iced internals, swapchain, allocator overhead; GPU time unavailable"})
    }
    pub fn pin_music(&self) -> Option<std::sync::Arc<Music>> {
        self.audio.clone()
    }
    pub fn music_owner_probe(&self) -> Option<impl Fn() -> bool> {
        self.audio.as_ref().map(|owner| {
            let weak = std::sync::Arc::downgrade(owner);
            move || weak.upgrade().is_some()
        })
    }
    pub fn clear_input(&mut self) {
        for input in &mut self.inputs {
            *input = LiveSquashInput::default();
        }
    }
    pub fn key(&mut self, host: &mut Host, e: &winit::event::KeyEvent) {
        let Some(index) = host
            .input
            .keyboard_target()
            .and_then(|s| host.game_surfaces.iter().position(|id| *id == s))
            .or_else(|| host.bypass_focus.then_some(0))
        else {
            self.clear_input();
            return;
        };
        use winit::keyboard::{Key, NamedKey};
        let key = match &e.logical_key {
            Key::Named(NamedKey::Space) => Some(LiveKey::Space),
            Key::Named(NamedKey::Enter) => Some(LiveKey::Enter),
            Key::Named(NamedKey::ArrowLeft) => Some(LiveKey::ArrowLeft),
            Key::Named(NamedKey::ArrowRight) => Some(LiveKey::ArrowRight),
            Key::Named(NamedKey::ArrowUp) => Some(LiveKey::ArrowUp),
            Key::Named(NamedKey::ArrowDown) => Some(LiveKey::ArrowDown),
            Key::Character(c) => match c.to_ascii_lowercase().as_str() {
                "w" => Some(LiveKey::KeyW),
                "a" => Some(LiveKey::KeyA),
                "s" => Some(LiveKey::KeyS),
                "d" => Some(LiveKey::KeyD),
                "r" => Some(LiveKey::KeyR),
                _ => None,
            },
            _ => None,
        };
        let Some(key) = key else { return };
        let down = e.state == winit::event::ElementState::Pressed;
        if down && e.repeat {
            return;
        }
        let dead = self.accepted[index].as_ref().is_some_and(|p| p.dead);
        let effect = self.inputs[index].key_event(key, down, dead);
        host.key_deliveries += 1;
        if matches!(
            effect,
            Some(LiveEffect::RetryPressed | LiveEffect::ResetPressed)
        ) {
            if let Err(e) = self.reset_one(index) {
                host.record_error(e)
            }
        }
    }
    pub fn reset_one(&mut self, index: usize) -> Result<(), String> {
        let old_id = self.scripts[index].id();
        self.events.retain(|(id, _, _)| *id != old_id);
        self.scripts[index].close();
        self.scripts[index] = SquashScript::new(&self.source)?;
        self.consumer_publications[index] =
            crate::squash::Publication3d::new(self.scripts[index].id());
        self.inputs[index] = LiveSquashInput::default();
        self.accepted[index] = None;
        Ok(())
    }
    pub fn tick(&mut self, host: &mut Host, t: u64) {
        if host.paused {
            return;
        }
        let focused = host
            .input
            .keyboard_target()
            .and_then(|s| host.game_surfaces.iter().position(|id| *id == s));
        for i in 0..2 {
            if self.scripts[i].session.closed() {
                continue;
            }
            if focused != Some(i) {
                self.inputs[i] = LiveSquashInput::default();
            }
            let action = self.inputs[i].action(self.accepted[i].as_ref().is_some_and(|p| p.dead));
            let id = self.scripts[i].id();
            if let Err(e) = self.scripts[i].tick(
                id,
                t,
                [
                    action.move_x as i32,
                    action.move_z as i32,
                    action.jump as i32,
                ],
            ) {
                host.record_error(e)
            }
        }
    }
    fn instances(&self, objects: &[Object3d]) -> Vec<MeshInstance> {
        let mut out = self.environment.clone();
        for o in objects {
            let model = &self.models[o.resource as usize];
            for mesh in &model.meshes {
                out.push(MeshInstance {
                    mesh: (model.source_sha256.clone(), mesh.name.clone()),
                    model: o.matrix(),
                });
            }
        }
        out
    }
    pub fn pump_outputs(&mut self, host: &mut Host) -> Result<(), String> {
        for i in 0..2 {
            if self.scripts[i].session.closed() {
                continue;
            }
            if let Err(e) = self.scripts[i].pump() {
                host.record_error(e);
                continue;
            }
            while let Some(packet) = self.scripts[i].outputs.pop_front() {
                packet.validate()?;
                let instances = self.instances(&packet.objects);
                if host.drop_output {
                    continue;
                }
                let target = host
                    .rt
                    .content(host.game_surfaces[i])
                    .ok_or("3d consumer surface missing")?;
                target
                    .validate_device(host.rt.gpu().token().ordinal)
                    .map_err(|e| format!("3d resource admission: {e:?}"))?;
                let SurfaceContent::Scene3d(prepared) = target else {
                    return Err("3d consumer resource kind".into());
                };
                if prepared.text_overlay.is_none()
                    || prepared.screen_overlay.is_none()
                    || instances.iter().any(|instance| {
                        prepared
                            .store
                            .get(&instance.mesh.0, &instance.mesh.1)
                            .is_none()
                    })
                {
                    return Err("3d consumer resources missing".into());
                }
                self.consumer_publications[i].publish(packet.clone())?;
                if let Some(SurfaceContent::Scene3d(content)) =
                    host.rt.content_mut(host.game_surfaces[i])
                {
                    let text = content.text_overlay.as_ref().unwrap();
                    let mut glyphs = text.renderer.layout(
                        &format!("Score: {}", packet.score),
                        16.0,
                        16.0,
                        Color::rgb(0.0, 0.0, 0.0),
                    );
                    if packet.dead {
                        let label = "Press Space or Enter to retry";
                        glyphs.extend(text.renderer.layout(
                            label,
                            (900.0 - text.renderer.measure(label)) * 0.5,
                            284.0,
                            Color::rgb(1.0, 1.0, 1.0),
                        ));
                    }
                    content.screen_overlay.as_mut().unwrap().commands = if packet.dead {
                        vec![agp::renderer::DrawCmd {
                            center: [450.0, 300.0],
                            size: [900.0, 600.0],
                            z: 0.0,
                            color: Color::rgba(0.0, 0.0, 0.0, 0.447059),
                            shape: agp::renderer::Shape::Rect,
                        }]
                    } else {
                        vec![]
                    };
                    content.instances = instances;
                    content.text_overlay.as_mut().unwrap().glyphs = glyphs;
                }
                for event in &packet.events {
                    if self.events.len() == 4096 {
                        self.events.pop_front();
                    }
                    self.events
                        .push_back((packet.session, packet.sequence, *event));
                }
                self.accepted[i] = Some(packet.clone());
                if host.duplicate_delivery {
                    self.consumer_publications[i].publish(packet)?;
                }
                host.rt.graph.mark_content_dirty(host.game_surfaces[i]);
            }
        }
        Ok(())
    }
    pub fn frame(
        &mut self,
        host: &mut Host,
        window: Option<&mut agp::renderer::target::WindowTarget>,
    ) -> Result<(), String> {
        self.pump_outputs(host)?;
        let events = std::mem::take(&mut host.ui_events);
        host.rt
            .frame_with_cursors(window, &events, &host.cursors)
            .map_err(|e| format!("3d frame: {e:?}"))?;
        let mut reset = false;
        if let Some(SurfaceContent::Iced(ui)) = host.rt.content(host.panel) {
            if let Some(p) = ui
                .ui
                .as_any()
                .downcast_ref::<agp::host::cases::surface_scenarios::PauseUi>()
            {
                host.paused = p.paused;
                if p.resets != host.seen_ui_resets {
                    host.seen_ui_resets = p.resets;
                    reset = true;
                }
            }
        }
        if reset {
            for i in 0..2 {
                self.reset_one(i)?;
            }
        }
        Ok(())
    }
    pub fn close(&mut self) {
        self.events.clear();
        for publication in &mut self.consumer_publications {
            publication.close();
        }
        for script in &mut self.scripts {
            script.close();
        }
        self.clear_input();
        self.audio = None;
    }
}
impl Drop for SquashRuntime {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod fixed_resource_tests {
    use super::validate_fixed_resource;
    #[test]
    fn fixed_resource_kind_and_content_are_rejected_before_loading() {
        let player =
            include_bytes!("../../autographics-prototype/assets/squash_the_creeps/art/player.glb");
        assert!(validate_fixed_resource("player", player).is_ok());
        assert!(validate_fixed_resource("mob", player)
            .unwrap_err()
            .contains("identity"));
        assert!(validate_fixed_resource("unknown", player)
            .unwrap_err()
            .contains("kind"));
        let mut bad = player.to_vec();
        bad[0] ^= 1;
        assert!(validate_fixed_resource("player", &bad).is_err());
    }
}
