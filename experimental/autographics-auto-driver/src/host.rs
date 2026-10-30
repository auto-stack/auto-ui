use crate::{
    consumer::SceneConsumer,
    script::{Action, Script},
};
use agp::core::composition::input::InputOutcome;
use agp::{
    core::{
        composition::{MountKind, Rect, SurfaceId},
        scene::Color,
    },
    host::{
        cases::surface_scenarios::PauseUi,
        composition_input::CompositionInput,
        surface_runtime::{
            Game2dContent, IcedContent, Scene3dContent, SurfaceContent, SurfaceRuntime,
        },
    },
    render3d::{MeshInstance, MeshPipeline, MeshStore},
    renderer::{
        target::{OffscreenTarget, WindowTarget},
        GpuDevice,
    },
};
use glam::{Mat4, Vec3};
use std::{collections::HashMap, time::Instant};
/// One script per content instance; views never own or advance script state.
struct Audio {
    manager: kira::manager::AudioManager,
    data: kira::sound::static_sound::StaticSoundData,
}

pub struct Host {
    pub squash: Option<crate::squash_host::SquashRuntime>,
    audio: Option<Audio>,
    pub sounds_played: u64,
    pub rt: SurfaceRuntime,
    pub input: CompositionInput,
    pub scripts: Vec<Script>,
    pub consumers: Vec<SceneConsumer>,
    pub game_surfaces: Vec<SurfaceId>,
    pub mesh_surface: SurfaceId,
    pub panel: SurfaceId,
    pub top: SurfaceId,
    source_text: String,
    last_sounds: Vec<i32>,
    pub seen_ui_resets: u32,
    pub paused: bool,
    pub key_deliveries: u64,
    pub drop_output: bool,
    pub corrupt_local: bool,
    pub duplicate_delivery: bool,
    pub bypass_focus: bool,
    pub direction: i32,
    pub errors: Vec<String>,
    pub sound_requests: u64,
    pub events: std::collections::VecDeque<(u64, u64, [i32; 3])>,
    published: Vec<u64>,
    pub pointer_deliveries: u64,
    pub ui_events: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>>,
    pub cursors: HashMap<SurfaceId, [f32; 2]>,
}
impl Host {
    pub fn new(source: &str) -> Result<Self, String> {
        Self::construct(source, true)
    }
    fn construct(source: &str, with_scripts: bool) -> Result<Self, String> {
        let gpu = GpuDevice::new().map_err(|e| format!("GPU: {e:?}"))?;
        let mut rt = SurfaceRuntime::new(gpu).map_err(|e| format!("composition: {e:?}"))?;
        let blank = |rt: &SurfaceRuntime| {
            SurfaceContent::Game2d(Box::new(Game2dContent::simple(
                rt.gpu(),
                Color::rgb(0.04, 0.05, 0.08),
            )))
        };
        let top = rt
            .add_surface([1200.0, 800.0], blank(&rt))
            .map_err(|e| format!("{e:?}"))?;
        let view = rt.add_view(top, true).map_err(|e| format!("{e:?}"))?;
        let panel = rt
            .add_surface(
                [320.0, 320.0],
                SurfaceContent::Iced(Box::new(IcedContent::borrowed(
                    rt.gpu(),
                    [320.0, 320.0],
                    Box::new(PauseUi::default()),
                ))),
            )
            .map_err(|e| format!("{e:?}"))?;
        let mut game_surfaces = vec![];
        for _ in 0..2 {
            game_surfaces.push(
                rt.add_surface([900.0, 600.0], blank(&rt))
                    .map_err(|e| format!("{e:?}"))?,
            );
        }
        for surface in game_surfaces.iter().filter(|_| with_scripts) {
            let text = agp::render2d::TextRenderer::new(
                rt.gpu(),
                OffscreenTarget::FORMAT,
                include_bytes!(
                    "../../autographics-prototype/assets/breakout/fonts/FiraMono-Medium.ttf"
                ),
                33.0,
                "Score: 0123456789",
            )
            .map_err(|e| format!("font: {e:?}"))?;
            if let Some(SurfaceContent::Game2d(g)) = rt.content_mut(*surface) {
                g.text_overlay = Some(agp::host::surface_runtime::TextOverlay {
                    renderer: text,
                    glyphs: vec![],
                });
            }
        }
        let mesh = Scene3dContent {
            screen_overlay: None,
            text_overlay: None,
            hud_overlay: None,
            mesh: MeshPipeline::new(rt.gpu(), OffscreenTarget::FORMAT)
                .map_err(|e| format!("{e:?}"))?,
            store: MeshStore::default(),
            instances: vec![],
            sky: [0.05, 0.06, 0.1, 1.0],
            view_proj: agp::host::iced_surface_probe::probe_camera(1.5),
        };
        let mesh_surface = rt
            .add_surface([450.0, 300.0], SurfaceContent::Scene3d(Box::new(mesh)))
            .map_err(|e| format!("{e:?}"))?;
        for (surface, offset, scale, size) in [
            (game_surfaces[0], [0.0, 0.0], [0.5, 0.5], [450.0, 300.0]),
            (game_surfaces[0], [450.0, 0.0], [0.5, 0.5], [450.0, 300.0]),
            (game_surfaces[1], [0.0, 320.0], [0.5, 0.5], [450.0, 300.0]),
            (mesh_surface, [450.0, 320.0], [1.0, 1.0], [450.0, 300.0]),
            (panel, [900.0, 0.0], [0.9375, 1.0], [300.0, 320.0]),
        ] {
            rt.add_portal(
                view,
                surface,
                MountKind::Flat2d {
                    offset,
                    scale,
                    clip: Rect::at(offset[0], offset[1], size[0], size[1]),
                },
                true,
            )
            .map_err(|e| format!("{e:?}"))?;
        }
        Ok(Self {
            squash: None,
            audio: None,
            sounds_played: 0,
            rt,
            input: CompositionInput::new([1200.0, 800.0], [1200.0, 800.0]),
            scripts: if with_scripts {
                vec![Script::new(source)?, Script::new(source)?]
            } else {
                vec![]
            },
            consumers: if with_scripts {
                vec![SceneConsumer::default(), SceneConsumer::default()]
            } else {
                vec![]
            },
            game_surfaces,
            mesh_surface,
            panel,
            top,
            source_text: source.to_owned(),
            last_sounds: vec![0, 0],
            seen_ui_resets: 0,
            paused: false,
            key_deliveries: 0,
            drop_output: false,
            corrupt_local: false,
            duplicate_delivery: false,
            bypass_focus: false,
            direction: 0,
            errors: vec![],
            sound_requests: 0,
            events: Default::default(),
            published: vec![0, 0],
            pointer_deliveries: 0,
            ui_events: HashMap::new(),
            cursors: HashMap::new(),
        })
    }
    pub fn new_squash(source: &str) -> Result<Self, String> {
        let mut host = Self::construct("", false)?;
        host.squash = Some(crate::squash_host::SquashRuntime::prepare(
            &mut host, source,
        )?);
        Ok(host)
    }
    pub fn enable_audio(&mut self) -> Result<(), String> {
        if let Some(squash) = &mut self.squash {
            return squash.enable_audio();
        }
        let settings = kira::manager::AudioManagerSettings {
            capacities: kira::manager::Capacities {
                sound_capacity: 1024,
                ..Default::default()
            },
            ..Default::default()
        };
        let manager = kira::manager::AudioManager::new(settings)
            .map_err(|e| format!("audio backend: {e:?}"))?;
        let data = kira::sound::static_sound::StaticSoundData::from_cursor(std::io::Cursor::new(
            include_bytes!(
                "../../autographics-prototype/assets/breakout/sounds/breakout_collision.ogg"
            )
            .to_vec(),
        ))
        .map_err(|e| format!("audio asset: {e:?}"))?;
        self.audio = Some(Audio { manager, data });
        Ok(())
    }
    fn script_index(&self, surface: SurfaceId) -> Option<usize> {
        self.game_surfaces
            .iter()
            .position(|s| *s == surface)
            .or_else(|| {
                (surface == self.mesh_surface && !self.consumers[0].meshes().is_empty())
                    .then_some(0)
            })
    }
    pub(crate) fn record_error(&mut self, error: String) {
        if self.errors.last() == Some(&error) {
            return;
        }
        if self.errors.len() == 128 {
            self.errors.remove(0);
        }
        self.errors.push(error);
    }
    pub fn resize(&mut self, w: u32, h: u32) {
        self.input.set_sizes([w as f32, h as f32], [1200.0, 800.0]);
    }
    pub fn click(&mut self, p: [f32; 2]) -> Option<(SurfaceId, [f32; 2])> {
        let outcome = self.input.pointer_pressed(&self.rt.graph, p);
        let Some(InputOutcome::Delivered { surface, local }) = outcome else {
            return None;
        };
        let local = if self.corrupt_local {
            [local[0] + 100.0, local[1] + 100.0]
        } else {
            local
        };
        if surface == self.panel {
            self.cursors.insert(surface, local);
            self.ui_events.entry(surface).or_default().extend([
                iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::CursorMoved {
                    position: iced_wgpu::core::Point::new(local[0], local[1]),
                }),
                iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
                    iced_wgpu::core::mouse::Button::Left,
                )),
                iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonReleased(
                    iced_wgpu::core::mouse::Button::Left,
                )),
            ]);
        } else if self.squash.is_some() {
            self.pointer_deliveries += 1;
        } else if let Some(i) = self.script_index(surface) {
            let id = self.scripts[i].id();
            if self.scripts[i].enqueue(id, Action::Pointer(local)).is_ok() {
                self.pointer_deliveries += 1;
                if self.duplicate_delivery {
                    let _ = self.scripts[i].enqueue(id, Action::Pointer(local));
                }
            }
        }
        if surface != self.panel {
            self.rt.release_iced_focus(self.panel);
            self.direction = 0;
        }
        if let Some(squash) = &mut self.squash {
            squash.clear_input();
        }
        self.input.pointer_released();
        Some((surface, local))
    }
    pub fn key(&mut self, e: &winit::event::KeyEvent) {
        if self.squash.is_some() && self.input.state.revalidate_focus(&self.rt.graph).is_some() {
            self.squash.as_mut().unwrap().clear_input();
        }
        if self.input.keyboard_target() == Some(self.panel) {
            if let Some(e) = agp::host::iced_surface_probe::winit_key_to_iced(e) {
                self.ui_events
                    .entry(self.panel)
                    .or_default()
                    .push(iced_wgpu::core::Event::Keyboard(e));
            }
            self.direction = 0;
            if !self.bypass_focus {
                return;
            }
        }
        if self.input.keyboard_target().is_none() {
            self.direction = 0;
            return;
        }
        if let Some(mut squash) = self.squash.take() {
            squash.key(self, e);
            self.squash = Some(squash);
            return;
        }
        use winit::keyboard::{Key, NamedKey};
        let down = e.state == winit::event::ElementState::Pressed;
        if down {
            if let Some(i) = self
                .input
                .keyboard_target()
                .and_then(|s| self.script_index(s))
                .or_else(|| self.bypass_focus.then_some(0))
            {
                let id = self.scripts[i].id();
                if self.scripts[i].enqueue(id, Action::Key(1)).is_ok() {
                    self.key_deliveries += 1;
                }
            }
        }
        match &e.logical_key {
            Key::Named(NamedKey::ArrowLeft) => self.direction = if down { -1 } else { 0 },
            Key::Named(NamedKey::ArrowRight) => self.direction = if down { 1 } else { 0 },
            Key::Character(c) if down && c.eq_ignore_ascii_case("r") => self.reset(),
            _ => {}
        }
    }
    pub fn reset(&mut self) {
        if let Some(squash) = &mut self.squash {
            for i in 0..2 {
                if let Err(e) = squash.reset_one(i) {
                    self.errors.push(e);
                }
            }
            return;
        }
        self.ui_events.clear();
        self.cursors.clear();
        self.events.clear();
        for i in 0..self.scripts.len() {
            let source = self.source_text.clone();
            self.scripts[i].close();
            match Script::new(&source) {
                Ok(s) => {
                    self.scripts[i] = s;
                    self.consumers[i] = SceneConsumer::default();
                    self.published[i] = 0;
                    self.last_sounds[i] = 0;
                }
                Err(e) => self.record_error(e),
            }
        }
        self.direction = 0;
        self.input.window_focus_lost();
    }
    pub fn tick(&mut self, t: u64) {
        if self.squash.is_some() && self.input.state.revalidate_focus(&self.rt.graph).is_some() {
            self.squash.as_mut().unwrap().clear_input();
        }
        if let Some(mut squash) = self.squash.take() {
            squash.tick(self, t);
            self.squash = Some(squash);
            return;
        }
        if self.paused {
            return;
        }
        let focused_owner = self
            .input
            .keyboard_target()
            .and_then(|s| self.script_index(s));
        for i in 0..self.scripts.len() {
            if self.scripts[i].session.closed() {
                continue;
            }
            let d = if focused_owner == Some(i) {
                self.direction
            } else {
                0
            };
            if let Err(e) = self.scripts[i].tick(t, d) {
                self.record_error(e)
            }
        }
    }
    pub fn frame(&mut self, window: Option<&mut WindowTarget>) -> Result<(), String> {
        if let Some(mut squash) = self.squash.take() {
            let result = squash.frame(self, window);
            self.squash = Some(squash);
            return result;
        }
        for i in 0..self.scripts.len() {
            if self.scripts[i].session.closed() {
                continue;
            }
            if let Err(e) = self.scripts[i].pump() {
                self.record_error(e);
            }
            while let Some(p) = self.scripts[i].outputs.pop_front() {
                if p.sequence == self.published[i] {
                    continue;
                }
                let requested = (p.sounds - self.last_sounds[i]).max(0) as u64;
                self.sound_requests += requested;
                if let Some(audio) = &mut self.audio {
                    for _ in 0..requested {
                        audio
                            .manager
                            .play(audio.data.clone())
                            .map_err(|e| format!("audio request: {e:?}"))?;
                        self.sounds_played += 1;
                    }
                }
                self.last_sounds[i] = p.sounds;
                if self.drop_output {
                    continue;
                }
                self.consumers[i].sync(&p)?;
                for event in &p.events {
                    if self.events.len() == 4096 {
                        self.events.pop_front();
                    }
                    self.events.push_back((p.session, p.sequence, *event));
                }
                self.published[i] = p.sequence;
                if let Some(SurfaceContent::Game2d(g)) = self.rt.content_mut(self.game_surfaces[i])
                {
                    g.clear = Color::rgba(
                        p.background[0],
                        p.background[1],
                        p.background[2],
                        p.background[3],
                    );
                    g.commands = self.consumers[i].draws([900.0, 600.0]);
                    if let Some(text) = &mut g.text_overlay {
                        text.glyphs =
                            text.renderer
                                .layout("Score: ", 5.0, 5.0, Color::rgb(0.5, 0.5, 1.0));
                        let score_x = 5.0 + text.renderer.measure("Score: ");
                        text.glyphs.extend(text.renderer.layout(
                            &p.score.to_string(),
                            score_x,
                            5.0,
                            Color::rgb(1.0, 0.5, 0.5),
                        ));
                    }
                }
                self.rt.graph.mark_content_dirty(self.game_surfaces[i]);
            }
        }
        self.sync_mesh()?;
        let events = std::mem::take(&mut self.ui_events);
        self.rt
            .frame_with_cursors(window, &events, &self.cursors)
            .map_err(|e| format!("frame: {e:?}"))?;
        let mut reset = false;
        if let Some(SurfaceContent::Iced(ui)) = self.rt.content(self.panel) {
            if let Some(p) = ui.ui.as_any().downcast_ref::<PauseUi>() {
                self.paused = p.paused;
                if p.resets != self.seen_ui_resets {
                    self.seen_ui_resets = p.resets;
                    reset = true;
                }
            }
        }
        if reset {
            self.reset();
        }
        Ok(())
    }
    fn sync_mesh(&mut self) -> Result<(), String> {
        let objects = self.consumers[0].meshes();
        let mut models = Vec::new();
        let mut instances = Vec::new();
        for o in objects {
            let mut mesh = agp::render3d::cylinder_mesh();
            mesh.material.base_color = [o.color.r, o.color.g, o.color.b, o.color.a];
            let mut identity = b"auto-cylinder-v1".to_vec();
            for value in mesh.positions.iter().chain(&mesh.normals).flatten() {
                identity.extend_from_slice(&value.to_le_bytes());
            }
            for value in &mesh.indices {
                identity.extend_from_slice(&value.to_le_bytes());
            }
            for value in mesh.material.base_color {
                identity.extend_from_slice(&value.to_le_bytes());
            }
            let key = agp::renderer::capture::sha256_hex(&identity);
            let name = mesh.name.clone();
            models.push(agp::assets::ModelData {
                source_sha256: key.clone(),
                meshes: vec![mesh],
            });
            let t = o.transform;
            instances.push(MeshInstance {
                mesh: (key, name),
                model: Mat4::from_scale_rotation_translation(
                    t.scale / 40.0,
                    t.rotation,
                    Vec3::new(
                        t.translation.x / 40.0,
                        t.translation.y / 40.0,
                        t.translation.z / 40.0,
                    ),
                ),
            });
        }
        let needs_resources = match self.rt.content(self.mesh_surface) {
            Some(SurfaceContent::Scene3d(s)) => {
                s.instances.iter().map(|v| &v.mesh).collect::<Vec<_>>()
                    != instances.iter().map(|v| &v.mesh).collect::<Vec<_>>()
            }
            _ => true,
        };
        let mut replacement = None;
        if needs_resources {
            let mut store = MeshStore::default();
            for model in models {
                store
                    .get_or_upload(self.rt.gpu(), &model, 0)
                    .map_err(|e| format!("mesh: {e:?}"))?;
            }
            replacement = Some(store);
        }
        if let Some(SurfaceContent::Scene3d(s)) = self.rt.content_mut(self.mesh_surface) {
            if let Some(store) = replacement {
                s.store = store;
            }
            s.instances = instances;
        }

        Ok(())
    }
    pub fn settle(&mut self) -> Result<(), String> {
        let now = Instant::now();
        loop {
            self.frame(None)?;
            if self.scripts.iter().all(Script::idle)
                && self
                    .squash
                    .as_ref()
                    .is_none_or(|s| s.scripts.iter().all(crate::squash::SquashScript::idle))
            {
                return Ok(());
            }
            if now.elapsed().as_secs() > 10 {
                return Err(format!("settle timeout: {:?}", self.errors));
            }
        }
    }
}
