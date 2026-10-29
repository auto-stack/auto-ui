//! Presentation, input capture and real Auto gesture publication.
use super::*;

impl Host {
    pub(super) fn prepare(
        &self,
        preview: &crate::authoring::preview::Preview,
    ) -> Result<Prepared, String> {
        self.prepare_frame(
            preview.document(),
            &preview.objects,
            &preview.nodes,
            None,
            None,
            None,
            None,
        )
    }
    pub(super) fn prepare_frame(
        &self,
        document: &Document,
        objects: &[Object],
        nodes: &std::collections::BTreeMap<u64, crate::authoring::hierarchy::Derived>,
        camera: Option<&crate::authoring::runtime_camera::RuntimeCameraFrame>,
        instances: Option<&crate::authoring::runtime_objects::RuntimeObjectsFrame>,
        sprites: Option<&crate::authoring::runtime_sprites::SpriteFrame>,
        animations: Option<&crate::authoring::runtime_node_animation::AnimationFrame>,
    ) -> Result<Prepared, String> {
        if self.fail_resource {
            return Err("resource preparation rejected".into());
        }
        let Some(SurfaceContent::Scene3d(scene)) = self.rt.content(self.three) else {
            return Err("scene unavailable".into());
        };
        let mut store = scene.store.staged();
        // PLAN-004: the textured pass reuses the accepted image cache of the 2D
        // surface. Existing textures are shared; new ones stay staged until the
        // whole candidate is accepted.
        let mut sprite_store = match self.rt.content(self.two) {
            Some(SurfaceContent::Game2d(g)) => g.sprite_store.staged(),
            _ => return Err("2D sprite surface unavailable".into()),
        };
        let assets = self.active_assets()?;
        crate::authoring::assets::check_registry(&assets)?;
        let template = |id: u64| -> Result<Object, String> {
            document
                .objects
                .iter()
                .find(|o| o.id == id)
                .cloned()
                .ok_or_else(|| "objects template missing".into())
        };
        let mut potential = 0;
        for o in objects.iter().filter(|o| !o.kind.is_2d()) {
            potential += self
                .model(o)
                .ok_or("missing resource payload")?
                .meshes
                .len();
        }
        // Hidden or inactive object slots never shrink the mesh denominator:
        // every live instance pays its full template mesh count.
        if let Some(instances) = instances {
            for slot in instances.slots.iter().filter(|s| s.live()) {
                let t = template(slot.template_id)?;
                if !t.kind.is_2d() {
                    potential += self
                        .model(&t)
                        .ok_or("missing object template model")?
                        .meshes
                        .len();
                }
            }
        }
        if potential > agp::render3d::pipeline::MAX_INSTANCES {
            return Err("resource limit: potential mesh instances 512".into());
        }
        for asset in assets.values() {
            for i in 0..asset.model.meshes.len() {
                store
                    .get_or_upload(self.rt.gpu(), &asset.model, i)
                    .map_err(|e| format!("GPU preparation: {e:?}"))?;
            }
        }
        if self.fail_after_asset_upload {
            return Err("GPU preparation: injected after asset upload".into());
        }
        let mut p = Prepared {
            store,
            draws: vec![],
            instances: vec![],
            highlight: vec![],
            sprites: vec![],
            sprite_store: Default::default(),
            camera_vp: None,
        };
        // Runtime camera transaction: the accepted frame's camera rides the
        // same Prepared, so a preparation failure keeps the last-good pair.
        let runtime = match camera {
            None => None,
            Some(cam) => match cam.dim {
                crate::authoring::runtime_camera::CameraDim::ThreeD => {
                    let target = nodes
                        .get(&cam.target_id)
                        .ok_or("camera target node missing")?;
                    let pos = Vec3::from(target.translation);
                    p.camera_vp = Some(
                        cam.view_proj_3d(pos)
                            .ok_or("camera orbit projection rejected")?,
                    );
                    None
                }
                crate::authoring::runtime_camera::CameraDim::TwoD => {
                    let target = nodes
                        .get(&cam.target_id)
                        .ok_or("camera target node missing")?;
                    let actor = glam::Vec2::new(target.translation[0], target.translation[1]);
                    let center = cam.follow_center_2d(actor).ok_or("camera follow center")?;
                    let (scale, _, _) = cam.letterbox_2d().ok_or("camera letterbox")?;
                    Some((cam, center, scale.x))
                }
            },
        };
        let view = self.active_view2d();
        for line in view.grid_lines().1 {
            let color = match line.class {
                2 => Color::rgb(0.55, 0.2, 0.2),
                3 => Color::rgb(0.2, 0.55, 0.2),
                1 => Color::rgb(0.22, 0.25, 0.3),
                _ => Color::rgb(0.13, 0.15, 0.18),
            };
            p.draws.push(DrawCmd {
                center: line.center,
                size: line.size,
                z: 0.0,
                color,
                shape: Shape::Rect,
            });
        }
        for o in objects.iter().filter(|o| o.visible) {
            if o.kind.is_2d() {
                let (center, zoom) = match &runtime {
                    Some((cam, follow, scale)) => (
                        cam.screen_2d(glam::Vec2::new(o.translation[0], o.translation[1]), *follow)
                            .map(|s| s.to_array())
                            .ok_or("2D camera projection exceeds finite range")?,
                        *scale,
                    ),
                    None => (
                        view.screen(glam::Vec2::new(o.translation[0], o.translation[1]))
                            .map(|s| s.to_array())
                            .ok_or("2D projection exceeds finite range")?,
                        view.zoom,
                    ),
                };
                if self.editor.selection == Some(o.id) {
                    p.draws.push(DrawCmd {
                        center,
                        size: [o.scale[0] * zoom + 8.0, o.scale[1] * zoom + 8.0],
                        z: 0.0,
                        color: Color::rgb(1.0, 1.0, 0.0),
                        shape: Shape::Rect,
                    });
                }
                // A bound leaf draws its accepted PNG instead of the flat shape:
                // Auto's frame/flip while a Run is publishing, the accepted
                // document default otherwise. Transparent pixels therefore show
                // the background, never a hidden tint quad.
                match Self::sprite_visual(document, sprites, o.id) {
                    Some(visual) => p.sprites.push(self.sprite_command(
                        &assets,
                        &mut sprite_store,
                        &visual,
                        center,
                        [o.scale[0] * zoom, o.scale[1] * zoom],
                        o.rgba,
                    )?),
                    None => p.draws.push(DrawCmd {
                        center,
                        size: [o.scale[0] * zoom, o.scale[1] * zoom],
                        z: 0.0,
                        color: Color::rgba(o.rgba[0], o.rgba[1], o.rgba[2], o.rgba[3]),
                        shape: if o.kind == Kind::Circle {
                            Shape::Circle
                        } else {
                            Shape::Rect
                        },
                    }),
                }
            } else {
                if self.editor.selection == Some(o.id) {
                    let vp = self.camera();
                    let model = self.model(o).ok_or("missing model resource")?;
                    if let Some((lo, hi)) = crate::authoring::interaction::bounds(model) {
                        let transform = crate::authoring::interaction::transform(o);
                        let corners: Vec<_> = (0..8)
                            .map(|i| {
                                Vec3::new(
                                    if i & 1 == 0 { lo.x } else { hi.x },
                                    if i & 2 == 0 { lo.y } else { hi.y },
                                    if i & 4 == 0 { lo.z } else { hi.z },
                                )
                            })
                            .collect();
                        for i in 0..8 {
                            for bit in [1, 2, 4] {
                                let j = i ^ bit;
                                if j > i {
                                    if let (Some(a), Some(b)) = (
                                        crate::authoring::interaction::project(
                                            vp,
                                            transform.transform_point3(corners[i]),
                                        ),
                                        crate::authoring::interaction::project(
                                            vp,
                                            transform.transform_point3(corners[j]),
                                        ),
                                    ) {
                                        Self::overlay_line(
                                            &mut p.highlight,
                                            a,
                                            b,
                                            Color::rgb(1.0, 1.0, 0.0),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    for (axis, a, d) in crate::authoring::interaction::axes(o, vp) {
                        let color = match axis {
                            0 => Color::rgb(1.0, 0.2, 0.2),
                            1 => Color::rgb(0.2, 1.0, 0.2),
                            _ => Color::rgb(0.3, 0.5, 1.0),
                        };
                        Self::overlay_line(&mut p.highlight, a, a + d.normalize() * 54.0, color);
                    }
                }
                let model = self.model(o).ok_or("missing model resource")?;
                let transform = Mat4::from_translation(Vec3::from(o.translation))
                    * Mat4::from_rotation_y(o.yaw)
                    * Mat4::from_rotation_x(o.pitch)
                    * Mat4::from_scale(Vec3::from(o.scale));
                for (i, mesh) in model.meshes.iter().enumerate() {
                    let _ = i;
                    if p.store.get(&model.source_sha256, &mesh.name).is_none() {
                        return Err("fixed mesh missing".into());
                    }
                    p.instances.push(MeshInstance {
                        mesh: (model.source_sha256.clone(), mesh.name.clone()),
                        model: crate::authoring::runtime_node_animation::mesh_matrix(
                            transform,
                            animations.and_then(|f| f.visual_for(o.id)),
                            mesh,
                        )?,
                    });
                }
            }
        }
        // Runtime object instances render through the non-persistent adapter
        // (id 0 can never alias a selectable document object): frozen template
        // shape/material, admitted world position and visibility. They are
        // non-solid presentation only; no selection, gesture or spatial state.
        if let Some(instances) = instances {
            for (slot_index, slot) in instances.slots.iter().enumerate().filter(|(_, s)| s.live()) {
                let t = template(slot.template_id)?;
                let o = crate::authoring::runtime_objects::instance_object(&t, slot)
                    .ok_or("objects adapter slot")?;
                if !o.visible {
                    continue;
                }
                if o.kind.is_2d() {
                    let (center, zoom) = match &runtime {
                        Some((cam, follow, scale)) => (
                            cam.screen_2d(
                                glam::Vec2::new(o.translation[0], o.translation[1]),
                                *follow,
                            )
                            .map(|s| s.to_array())
                            .ok_or("2D camera projection exceeds finite range")?,
                            *scale,
                        ),
                        None => (
                            view.screen(glam::Vec2::new(o.translation[0], o.translation[1]))
                                .map(|s| s.to_array())
                                .ok_or("2D projection exceeds finite range")?,
                            view.zoom,
                        ),
                    };
                    match Self::sprite_visual(document, sprites, t.id) {
                        Some(visual) => p.sprites.push(self.sprite_command(
                            &assets,
                            &mut sprite_store,
                            &visual,
                            center,
                            [o.scale[0] * zoom, o.scale[1] * zoom],
                            o.rgba,
                        )?),
                        None => p.draws.push(DrawCmd {
                            center,
                            size: [o.scale[0] * zoom, o.scale[1] * zoom],
                            z: 0.0,
                            color: Color::rgba(o.rgba[0], o.rgba[1], o.rgba[2], o.rgba[3]),
                            shape: if o.kind == Kind::Circle {
                                Shape::Circle
                            } else {
                                Shape::Rect
                            },
                        }),
                    }
                } else {
                    let model = self.model(&o).ok_or("missing object template model")?;
                    let transform = Mat4::from_translation(Vec3::from(o.translation))
                        * Mat4::from_rotation_y(o.yaw)
                        * Mat4::from_rotation_x(o.pitch)
                        * Mat4::from_scale(Vec3::from(o.scale));
                    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
                        // A hidden template never prepared its meshes in the
                        // editing view; the first spawned instance uploads it
                        // once and every later birth reuses the shared mesh.
                        if p.store.get(&model.source_sha256, &mesh.name).is_none() {
                            p.store
                                .get_or_upload(self.rt.gpu(), model, mesh_index)
                                .map_err(|e| format!("GPU preparation: {e:?}"))?;
                        }
                        if p.store.get(&model.source_sha256, &mesh.name).is_none() {
                            return Err("fixed mesh missing".into());
                        }
                        p.instances.push(MeshInstance {
                            mesh: (model.source_sha256.clone(), mesh.name.clone()),
                            model: crate::authoring::runtime_node_animation::mesh_matrix(
                                transform,
                                animations
                                    .and_then(|f| f.slots[slot_index].as_ref())
                                    .map(|s| &s.visual),
                                mesh,
                            )?,
                        });
                    }
                }
            }
        }
        if let Some(id) = self.editor.selection {
            if let Some((origin, lo, hi)) = self.group_geometry(document, objects, nodes, id) {
                let out = if origin.kind.is_2d() {
                    &mut p.draws
                } else {
                    &mut p.highlight
                };
                let color = Color::rgb(1.0, 1.0, 0.0);
                let corners = [
                    lo,
                    glam::Vec2::new(hi.x, lo.y),
                    hi,
                    glam::Vec2::new(lo.x, hi.y),
                ];
                for i in 0..4 {
                    Self::overlay_line(out, corners[i], corners[(i + 1) % 4], color);
                }
                if origin.kind.is_2d() {
                    let a = view
                        .screen(glam::Vec2::new(
                            origin.translation[0],
                            origin.translation[1],
                        ))
                        .ok_or("2D group projection")?;
                    Self::overlay_line(
                        out,
                        a - glam::Vec2::X * 8.0,
                        a + glam::Vec2::X * 8.0,
                        color,
                    );
                    Self::overlay_line(
                        out,
                        a - glam::Vec2::Y * 8.0,
                        a + glam::Vec2::Y * 8.0,
                        color,
                    );
                } else {
                    for (axis, a, d) in crate::authoring::interaction::axes(&origin, self.camera())
                    {
                        Self::overlay_line(
                            out,
                            a,
                            a + d.normalize() * 54.0,
                            match axis {
                                0 => Color::rgb(1.0, 0.2, 0.2),
                                1 => Color::rgb(0.2, 1.0, 0.2),
                                _ => Color::rgb(0.3, 0.5, 1.0),
                            },
                        );
                    }
                }
            }
        }
        if let Some((cam, _, _)) = &runtime {
            let clear = match self.rt.content(self.two) {
                Some(SurfaceContent::Game2d(g)) => g.clear,
                _ => return Err("2D camera surface unavailable".into()),
            };
            let (_, _, bars) = cam.letterbox_2d().ok_or("camera letterbox")?;
            let size = crate::authoring::view2d::SIZE;
            let mut bar_rects: Vec<([f32; 2], [f32; 2])> = Vec::with_capacity(2);
            if bars.x > 0.0 {
                bar_rects.push(([bars.x * 0.5, size.y * 0.5], [bars.x, size.y]));
                bar_rects.push(([size.x - bars.x * 0.5, size.y * 0.5], [bars.x, size.y]));
            }
            if bars.y > 0.0 {
                bar_rects.push(([size.x * 0.5, bars.y * 0.5], [size.x, bars.y]));
                bar_rects.push(([size.x * 0.5, size.y - bars.y * 0.5], [size.x, bars.y]));
            }
            for (center, bar) in bar_rects {
                p.draws.push(DrawCmd {
                    center,
                    size: bar,
                    z: 0.0,
                    color: clear,
                    shape: Shape::Rect,
                });
            }
        }
        if p.draws.len() > 4096
            || p.draws
                .iter()
                .any(|c| c.center.iter().chain(c.size.iter()).any(|v| !v.is_finite()))
        {
            return Err("2D draw capacity or finite projection rejected".into());
        }
        if p.sprites.len() > agp::renderer::sprite::MAX_SPRITES {
            return Err("2D sprite capacity rejected".into());
        }
        p.sprite_store = sprite_store;
        Ok(p)
    }
    /// One textured visual of the accepted frame. The texture is content
    /// identified and prepared here, before the frame is accepted, so a failed
    /// texture keeps the whole last-good frame.
    fn sprite_command(
        &self,
        assets: &crate::authoring::assets::Registry,
        store: &mut agp::renderer::SpriteStore,
        visual: &crate::authoring::runtime_sprites::Visual,
        center: [f32; 2],
        size: [f32; 2],
        rgba: [f32; 4],
    ) -> Result<agp::renderer::SpriteCmd, String> {
        let asset = assets.get(&visual.asset).ok_or("missing sprite payload")?;
        let image = asset.image_data()?;
        let uv = agp::renderer::frame_uv(visual.frame, image.width, image.height, visual.flip)?;
        store
            .get_or_upload(self.rt.gpu(), &visual.asset, image)
            .map_err(|e| format!("sprite texture preparation: {e:?}"))?;
        // The PNG is the art: the object colour is the flat-shape fallback, so
        // a sprite is drawn with a neutral tint and only the object alpha is
        // honoured. Tinting by the default 2D colour would distort every
        // imported image.
        Ok(agp::renderer::SpriteCmd {
            center,
            size,
            uv,
            color: [1.0, 1.0, 1.0, rgba[3]],
            texture: visual.asset.clone(),
        })
    }
    /// The visual of one accepted 2D leaf: Auto's admitted frame when a Run is
    /// publishing, the accepted document default otherwise.
    fn sprite_visual(
        document: &Document,
        frame: Option<&crate::authoring::runtime_sprites::SpriteFrame>,
        id: u64,
    ) -> Option<crate::authoring::runtime_sprites::Visual> {
        if let Some(visual) = frame.and_then(|f| f.visual_for(id)) {
            return Some(visual.clone());
        }
        document
            .objects
            .iter()
            .find(|o| o.id == id)
            .and_then(|o| o.sprite.as_ref())
            .map(|b| crate::authoring::runtime_sprites::Visual {
                asset: b.asset.clone(),
                frame: b.frame,
                flip: b.flip,
            })
    }

    pub(super) fn publish(&mut self, mut p: Prepared) {
        if let Some(fault) = &self.asset_render_fault {
            let asset_shas: std::collections::BTreeSet<_> = self
                .editor
                .assets
                .values()
                .map(|a| a.identity.sha256.as_str())
                .collect();
            if fault == "missing-part" {
                p.instances.retain(|i| {
                    !asset_shas.contains(i.mesh.0.as_str()) || i.mesh.1 != "primitive-1"
                });
            }
            if fault == "transparent" {
                for i in &mut p.instances {
                    if asset_shas.contains(i.mesh.0.as_str()) {
                        i.model = Mat4::from_scale(Vec3::ZERO);
                    }
                }
            }
            if fault == "wrong-asset" {
                for i in &mut p.instances {
                    if asset_shas.contains(i.mesh.0.as_str()) {
                        i.mesh = (
                            self.models[1].source_sha256.clone(),
                            self.models[1].meshes[0].name.clone(),
                        );
                    }
                }
            }
        }
        if let Some(SurfaceContent::Game2d(g)) = self.rt.content_mut(self.two) {
            g.commands = p.draws;
            if g.sprite_store.len() != p.sprite_store.len()
                || g.sprite_store
                    .keys()
                    .any(|k| p.sprite_store.get(k).is_none())
            {
                g.sprite.invalidate_bind_groups();
            }
            g.sprites = p.sprites;
            g.sprite_store = p.sprite_store;
        }
        if let Some(SurfaceContent::Scene3d(c)) = self.rt.content_mut(self.three) {
            c.store = p.store;
            c.instances = p.instances;
            if let Some(vp) = p.camera_vp {
                c.view_proj = vp;
            }
            if let Some(overlay) = &mut c.screen_overlay {
                overlay.commands = p.highlight;
            }
        }
        self.prune_resources();
        self.rt.graph.mark_content_dirty(self.two);
        self.rt.graph.mark_content_dirty(self.three);
    }
    pub fn click(&mut self, p: [f32; 2]) {
        if let Some(InputOutcome::Delivered { surface, local }) =
            self.input.pointer_pressed(&self.rt.graph, p)
        {
            if surface == self.panel {
                self.cursors.insert(surface, local);
                self.events.entry(surface).or_default().extend([
                    iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::CursorMoved {
                        position: iced::Point::new(local[0], local[1]),
                    }),
                    iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
                        iced_wgpu::core::mouse::Button::Left,
                    )),
                    iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonReleased(
                        iced_wgpu::core::mouse::Button::Left,
                    )),
                ]);
            } else {
                self.blur();
            }
        }
        self.input.pointer_released();
    }
    /// Profile key map (contract section 3): character/1 = A/Left, D/Right,
    /// Space, Enter; character3d/1 = A/D/W/S (and arrows) for the horizontal
    /// basis, Space Jump, Enter Restart. OS repeat is dropped downstream.
    fn character_action(
        profile: crate::authoring::play::CharProfile,
        logical: &winit::keyboard::Key,
    ) -> Option<usize> {
        use crate::authoring::game_input as g;
        use crate::authoring::play::CharProfile;
        use winit::keyboard::{Key, NamedKey};
        match profile {
            CharProfile::TwoD => match logical {
                Key::Character(c) if c.eq_ignore_ascii_case("a") => Some(g::ACTION_LEFT),
                Key::Character(c) if c.eq_ignore_ascii_case("d") => Some(g::ACTION_RIGHT),
                Key::Named(NamedKey::ArrowLeft) => Some(g::ACTION_LEFT),
                Key::Named(NamedKey::ArrowRight) => Some(g::ACTION_RIGHT),
                Key::Named(NamedKey::Space) => Some(g::ACTION_JUMP),
                Key::Named(NamedKey::Enter) => Some(g::ACTION_RESTART),
                _ => None,
            },
            CharProfile::ThreeD => match logical {
                Key::Character(c) if c.eq_ignore_ascii_case("a") => Some(g::ACTION3_LEFT),
                Key::Character(c) if c.eq_ignore_ascii_case("d") => Some(g::ACTION3_RIGHT),
                Key::Character(c) if c.eq_ignore_ascii_case("w") => Some(g::ACTION3_FORWARD),
                Key::Character(c) if c.eq_ignore_ascii_case("s") => Some(g::ACTION3_BACK),
                Key::Named(NamedKey::ArrowLeft) => Some(g::ACTION3_LEFT),
                Key::Named(NamedKey::ArrowRight) => Some(g::ACTION3_RIGHT),
                Key::Named(NamedKey::ArrowUp) => Some(g::ACTION3_FORWARD),
                Key::Named(NamedKey::ArrowDown) => Some(g::ACTION3_BACK),
                Key::Named(NamedKey::Space) => Some(g::ACTION3_JUMP),
                Key::Named(NamedKey::Enter) => Some(g::ACTION3_RESTART),
                _ => None,
            },
        }
    }
    pub fn key(&mut self, k: &winit::event::KeyEvent) {
        if self.play.is_some() {
            use winit::keyboard::{Key, NamedKey};
            let down = k.state == winit::event::ElementState::Pressed;
            if down && !k.repeat && k.logical_key == Key::Named(NamedKey::Escape) {
                self.stop_play();
                return;
            }
            let key = match &k.logical_key {
                Key::Named(NamedKey::ArrowLeft) => Some(crate::authoring::play::Key::Left),
                Key::Named(NamedKey::ArrowRight) => Some(crate::authoring::play::Key::Right),
                Key::Named(NamedKey::ArrowUp) => Some(crate::authoring::play::Key::Up),
                Key::Named(NamedKey::ArrowDown) => Some(crate::authoring::play::Key::Down),
                Key::Character(c) if c.eq_ignore_ascii_case("v") => {
                    Some(crate::authoring::play::Key::Toggle)
                }
                _ => None,
            };
            let target = self.input.keyboard_target();
            let space = if target == Some(self.two) {
                2
            } else if target == Some(self.three) {
                3
            } else {
                0
            };
            if let Some(p) = &mut self.play {
                p.input.focus(space);
                if let Some(profile) = p.character_profile() {
                    // Single router: a character run consumes the mapped game
                    // keys here only; the legacy x/y/toggle sampler never sees
                    // them. Unmapped keys keep their editor/panel handling.
                    if let Some(action) = Self::character_action(profile, &k.logical_key) {
                        p.char_key(action, down, k.repeat);
                        if space != 0 {
                            return;
                        }
                    }
                } else if let Some(key) = key {
                    if p.state == crate::authoring::play::State::Running || !down {
                        p.input.key(key, down, k.repeat);
                    }
                }
            }
            // Panel buttons still receive their own widget events; arrows never reach a world there.
            if space != 0 {
                return;
            }
        }

        if k.state == winit::event::ElementState::Pressed
            && k.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)
        {
            self.cancel_navigation();
            self.cancel_gesture();
            return;
        }
        if k.state == winit::event::ElementState::Pressed
            && !k.repeat
            && self.input.keyboard_target() == Some(self.two)
        {
            use winit::keyboard::{Key, NamedKey};
            let result = match &k.logical_key {
                Key::Named(NamedKey::Home) => Some(self.view_command("view-reset", &Value::Null)),
                Key::Character(c) if c.eq_ignore_ascii_case("f") => {
                    Some(self.view_command("view-focus", &Value::Null))
                }
                _ => None,
            };
            if let Some(result) = result {
                if let Err(e) = result {
                    self.ui_mut().error = e;
                }
                return;
            }
        }
        if k.state == winit::event::ElementState::Pressed
            && !k.repeat
            && self.input.keyboard_target() == Some(self.three)
        {
            use winit::keyboard::{Key, NamedKey};
            let action = match &k.logical_key {
                Key::Named(NamedKey::Home) => Some("view3d-reset"),
                Key::Character(c) if c.eq_ignore_ascii_case("f") => Some("view3d-focus"),
                Key::Character(c) if c.eq_ignore_ascii_case("p") => Some("view3d-projection"),
                _ => None,
            };
            if let Some(action) = action {
                if let Err(e) = self.view3d_command(action, &Value::Null) {
                    self.ui_mut().error = e;
                }
                return;
            }
        }
        if self.gesture.is_some() {
            return;
        }
        if self.input.keyboard_target() == Some(self.panel) {
            let event = if k.state == winit::event::ElementState::Pressed
                && matches!(k.logical_key, winit::keyboard::Key::Unidentified(_))
            {
                k.text.as_ref().filter(|s| !s.is_empty()).map(|s| {
                    use iced_wgpu::core::keyboard::{
                        key::{NativeCode, Physical},
                        Event, Key, Location, Modifiers,
                    };
                    let key = Key::Character(s.clone());
                    Event::KeyPressed {
                        key: key.clone(),
                        modified_key: key,
                        physical_key: Physical::Unidentified(NativeCode::Unidentified),
                        location: Location::Standard,
                        modifiers: Modifiers::default(),
                        text: Some(s.clone()),
                        repeat: k.repeat,
                    }
                })
            } else {
                agp::host::iced_surface_probe::winit_key_to_iced(k)
            };
            if let Some(e) = event {
                let e = if k.state == winit::event::ElementState::Released {
                    match e {
                        iced_wgpu::core::keyboard::Event::KeyPressed {
                            key,
                            modified_key,
                            physical_key,
                            location,
                            modifiers,
                            ..
                        } => iced_wgpu::core::keyboard::Event::KeyReleased {
                            key,
                            modified_key,
                            physical_key,
                            location,
                            modifiers,
                        },
                        e => e,
                    }
                } else {
                    e
                };
                self.events
                    .entry(self.panel)
                    .or_default()
                    .push(iced_wgpu::core::Event::Keyboard(e));
            }
        }
    }
    pub fn blur(&mut self) {
        self.cancel_navigation();
        self.pause_play("auto-Pause: focus lost");
        self.cancel_gesture();
        self.cancel_camera_drag();
        self.rt.release_iced_focus(self.panel);
        self.ui_mut().reset_draft();
        self.events.clear();
    }
    pub fn resize(&mut self, w: u32, h: u32) {
        self.cancel_navigation();
        self.pause_play("auto-Pause: resize/DPI");
        self.cancel_gesture();
        self.cancel_camera_drag();
        self.input.set_sizes([w as f32, h as f32], [1200.0, 800.0]);
    }
    pub fn frame(&mut self, window: Option<&mut WindowTarget>) -> Result<(), String> {
        let now = Instant::now();
        self.pump_play(now);
        self.prune_resources();
        if let Err(e) = self.pump_gesture() {
            self.cancel_gesture();
            self.ui_mut().error = e;
        }
        if self.editor.pending() {
            match self.editor.pump() {
                Ok(true) => {
                    let old_view = self.view2d;
                    let old_view3d = self.view3d;
                    if self.opening.is_some() {
                        self.view2d = super::super::view2d::View2d {
                            revision: old_view.revision.saturating_add(1),
                            ..Default::default()
                        };
                    }
                    let prepared = if self.opening.is_some() {
                        let reset = super::super::view3d::View3d {
                            revision: old_view3d.revision.saturating_add(1),
                            ..Default::default()
                        };
                        self.install_3d(reset).and_then(|_| {
                            self.view3d = reset;
                            self.prepare(self.editor.candidate().unwrap())
                        })
                    } else {
                        self.prepare(self.editor.candidate().unwrap())
                    };
                    match prepared {
                        Ok(p) => {
                            self.editor.commit()?;
                            if self.project_pending {
                                self.project_default = true;
                                self.project_pending = false;
                            }
                            self.ui_mut().error.clear();
                            self.asset_pending = false;
                            self.presented = self.editor.preview.objects.clone();
                            self.presented_nodes = self.editor.preview.nodes.clone();
                            self.presented_stamp = self.editor.stamp;
                            if let Some(s) = self.opening.take() {
                                self.project_default = true;
                                self.ui_mut().behavior_user = false;
                                self.ui_mut().behavior_path.clear();
                                self.storage = s;
                            }
                            self.publish(p);
                            self.accepted += 1;
                            self.sync_ui();
                        }
                        Err(e) => {
                            self.view2d = old_view;
                            if self.view3d != old_view3d {
                                self.view3d = old_view3d;
                                if let Err(restore) = self.install_3d(old_view3d) {
                                    self.ui_mut().error = restore;
                                }
                            }
                            self.editor.reject();
                            self.project_pending = false;
                            self.asset_pending = false;
                            self.opening = None;
                            self.ui_mut().error = e;
                            self.sync_ui();
                        }
                    }
                }
                Ok(false) => {}
                Err(e) => {
                    self.project_pending = false;
                    self.asset_pending = false;
                    self.opening = None;
                    self.ui_mut().error = e;
                    self.sync_ui();
                }
            }
        }
        let events = std::mem::take(&mut self.events);
        self.rt
            .frame_with_cursors(window, &events, &self.cursors)
            .map_err(|e| format!("{e:?}"))?;
        self.presented_camera = self.camera();
        self.presented_view2d = self.active_view2d();
        self.presented_view3d = if self.play.is_some() {
            Default::default()
        } else {
            self.view3d
        };
        let messages = std::mem::take(&mut self.ui_mut().queue);
        for m in messages {
            if let Err(e) = self.dispatch(&m) {
                let dialog = self.leave.is_some();
                let busy = self.editor.pending() || self.gesture.is_some();
                self.ui_mut().error = e;
                self.ui_mut().dialog = dialog;
                self.ui_mut().busy = busy;
                self.rt.graph.mark_content_dirty(self.panel);
            }
        }
        self.frame_us.push(now.elapsed().as_micros());
        if self.frame_us.len() > 4096 {
            self.frame_us.remove(0);
        }
        Ok(())
    }
    pub(super) fn group_geometry(
        &self,
        doc: &Document,
        objects: &[Object],
        nodes: &std::collections::BTreeMap<u64, crate::authoring::hierarchy::Derived>,
        id: u64,
    ) -> Option<(Object, glam::Vec2, glam::Vec2)> {
        let group = doc.groups.iter().find(|g| g.id == id)?;
        let members = crate::authoring::hierarchy::descendants(doc, id).ok()?;
        let mut lo = glam::Vec2::splat(f32::INFINITY);
        let mut hi = glam::Vec2::splat(f32::NEG_INFINITY);
        for o in objects.iter().filter(|o| {
            members.contains(&o.id) && o.visible && (!o.kind.is_2d() || o.rgba[3] > 0.0)
        }) {
            let points: Vec<_> = if o.kind.is_2d() {
                let center = self
                    .active_view2d()
                    .screen(glam::Vec2::new(o.translation[0], o.translation[1]))?;
                vec![
                    center
                        - glam::Vec2::new(o.scale[0], o.scale[1])
                            * (0.5 * self.active_view2d().zoom),
                    center
                        + glam::Vec2::new(o.scale[0], o.scale[1])
                            * (0.5 * self.active_view2d().zoom),
                ]
            } else {
                let model = self.model(o)?;
                let (a, b) = crate::authoring::interaction::bounds(model)?;
                let transform = crate::authoring::interaction::transform(o);
                crate::authoring::interaction::clipped_box_points(self.camera(), transform, a, b)
            };
            for point in points {
                lo = lo.min(point);
                hi = hi.max(point);
            }
        }
        if !lo.is_finite() || !hi.is_finite() {
            return None;
        }
        let world = nodes.get(&id)?;
        let mut origin = doc.local_target(id)?;
        origin.translation = world.translation;
        origin.visible = world.visible;
        if group.dimension == Dimension::Two {
            if !crate::authoring::interaction::inside(
                self.active_view2d()
                    .screen(glam::Vec2::new(world.translation[0], world.translation[1]))?
                    .to_array(),
                [800.0, 380.0],
            ) {
                return None;
            }
        } else if crate::authoring::interaction::axes(&origin, self.camera()).is_empty() {
            return None;
        }
        Some((origin, lo, hi))
    }
    pub fn scroll(&mut self, position: [f32; 2], delta: iced_wgpu::core::mouse::ScrollDelta) {
        if !crate::authoring::interaction::inside(position, self.input.window_size) {
            return;
        }
        if self.gesture.is_some() || self.navigation.is_some() || self.navigation3d.is_some() {
            return;
        }
        let wheel_top = self.input.window_to_top(position);
        if let Some(play) = &mut self.play {
            if play.camera_dim() == Some(crate::authoring::runtime_camera::CameraDim::ThreeD)
                && crate::authoring::interaction::inside(
                    [wheel_top[0], wheel_top[1] - 400.0],
                    [800.0, 400.0],
                )
            {
                let notches = match delta {
                    iced_wgpu::core::mouse::ScrollDelta::Lines { x, y }
                        if x.is_finite() && y.is_finite() =>
                    {
                        y
                    }
                    iced_wgpu::core::mouse::ScrollDelta::Pixels { x, y }
                        if x.is_finite() && y.is_finite() =>
                    {
                        ((y as f64 * self.input.top_logical[1] as f64
                            / self.input.window_size[1] as f64
                            / 40.0)
                            .clamp(-8.0, 8.0)) as f32
                    }
                    _ => {
                        self.ui_mut().error = "invalid wheel delta".into();
                        return;
                    }
                };
                play.cam_zoom(notches);
                self.ui_mut().error.clear();
                return;
            }
        }
        let p = self.input.window_to_top(position);
        if crate::authoring::interaction::inside([p[0], p[1] - 400.0], [800.0, 400.0]) {
            let notches = match delta {
                iced_wgpu::core::mouse::ScrollDelta::Lines { x, y }
                    if x.is_finite() && y.is_finite() =>
                {
                    y
                }
                iced_wgpu::core::mouse::ScrollDelta::Pixels { x, y }
                    if x.is_finite() && y.is_finite() =>
                {
                    ((y as f64 * self.input.top_logical[1] as f64
                        / self.input.window_size[1] as f64
                        / 40.0)
                        .clamp(-8.0, 8.0)) as f32
                }
                _ => {
                    self.ui_mut().error = "invalid wheel delta".into();
                    return;
                }
            };
            if let Err(e) = self.view3d.wheel(notches).and_then(|v| self.set_view3d(v)) {
                self.ui_mut().error = e;
            }
            return;
        }
        if crate::authoring::interaction::inside(p, [800.0, 380.0]) {
            let finite = match delta {
                iced_wgpu::core::mouse::ScrollDelta::Lines { x, y }
                | iced_wgpu::core::mouse::ScrollDelta::Pixels { x, y } => {
                    x.is_finite() && y.is_finite()
                }
            };
            if !finite {
                self.ui_mut().error = "invalid wheel delta".into();
                return;
            }
            let notches = match delta {
                iced_wgpu::core::mouse::ScrollDelta::Lines { y, .. } => y,
                iced_wgpu::core::mouse::ScrollDelta::Pixels { y, .. } => {
                    // winit supplies physical pixels; use the same physical-to-top
                    // mapping as pointer coordinates, once, before the notch cap.
                    ((y as f64 * self.input.top_logical[1] as f64
                        / self.input.window_size[1] as f64
                        / 40.0)
                        .clamp(-8.0, 8.0)) as f32
                }
            };
            let result = self
                .view2d
                .wheel(glam::Vec2::from(p), notches)
                .and_then(|v| self.set_view2d(v));
            if let Err(e) = result {
                self.ui_mut().error = e;
            }
            return;
        }
        if p[0] >= 810.0 {
            self.cursors.insert(self.panel, [p[0] - 810.0, p[1]]);
            self.events
                .entry(self.panel)
                .or_default()
                .push(iced_wgpu::core::Event::Mouse(
                    iced_wgpu::core::mouse::Event::WheelScrolled { delta },
                ));
        }
    }
    pub(super) fn overlay_line(out: &mut Vec<DrawCmd>, a: glam::Vec2, b: glam::Vec2, color: Color) {
        // Axis-aligned dots approximate arbitrary projected segments without a new renderer API.
        let n = (b - a).length().ceil().min(240.0) as usize;
        for i in 0..=n {
            if out.len() >= 4096 {
                break;
            }
            let p = a + (b - a) * (i as f32 / n.max(1) as f32);
            out.push(DrawCmd {
                center: p.to_array(),
                size: [3.0, 3.0],
                z: 0.0,
                color,
                shape: Shape::Rect,
            });
        }
    }
    pub fn camera(&self) -> Mat4 {
        match self.rt.content(self.three) {
            Some(SurfaceContent::Scene3d(c)) => c.view_proj,
            _ => Mat4::ZERO,
        }
    }
    pub fn gesture_active(&self) -> bool {
        self.gesture.is_some()
    }
    pub fn pointer_down(&mut self, p: [f32; 2]) {
        if self.play.is_some() {
            if let Some(InputOutcome::Delivered { surface, local }) =
                self.input.pointer_pressed(&self.rt.graph, p)
            {
                // 2D camera letterbox bars never receive the game pointer.
                let bar_hit = surface == self.two
                    && self
                        .play
                        .as_ref()
                        .and_then(|play| play.camera_frame())
                        .and_then(|cam| cam.letterbox_2d())
                        .is_some_and(|(_, content, bars)| {
                            local[0] < bars[0]
                                || local[0] > bars[0] + content.x
                                || local[1] < bars[1]
                                || local[1] > bars[1] + content.y
                        });
                if bar_hit {
                    self.input.pointer_released();
                    return;
                }
                let space = if surface == self.two {
                    2
                } else if surface == self.three {
                    3
                } else {
                    0
                };
                self.play.as_mut().unwrap().input.focus(space);
                if space != 0 {
                    self.rt.release_iced_focus(self.panel);
                    self.events.clear();
                    // 3D camera Runs: a fresh press inside the content area
                    // begins one orbit capture (resume needs a new press).
                    if space == 3
                        && self.play.as_ref().is_some_and(|play| {
                            play.camera_dim()
                                == Some(crate::authoring::runtime_camera::CameraDim::ThreeD)
                        })
                    {
                        self.camera_drag = Some(CameraDrag { last: local });
                    }
                } else {
                    self.input.pointer_released();
                    self.click(p);
                }
            }
            self.input.pointer_released();
            return;
        }

        if self.navigation.is_some()
            || self.navigation3d.is_some()
            || self.gesture.is_some()
            || self.editor.pending()
            || self.leave.is_some()
            || !crate::authoring::interaction::inside(p, self.input.window_size)
        {
            return;
        }
        let Some(InputOutcome::Delivered { surface, local }) =
            self.input.pointer_pressed(&self.rt.graph, p)
        else {
            return;
        };
        if surface == self.panel {
            self.input.pointer_released();
            self.click(p);
            return;
        }
        if surface != self.two && surface != self.three {
            self.input.pointer_released();
            return;
        }
        if self.presented_stamp != self.editor.stamp
            || self.presented_camera != self.camera()
            || self.presented_view2d != self.active_view2d()
            || self.presented_view3d != self.view3d
        {
            self.input.pointer_released();
            return;
        }
        self.rt.release_iced_focus(self.panel);
        self.events.clear();
        let vp = self.camera();
        let group = self.editor.selection.and_then(|id| {
            self.group_geometry(
                &self.editor.document,
                &self.presented,
                &self.presented_nodes,
                id,
            )
        });
        let selected = group.as_ref().map(|g| &g.0).or_else(|| {
            self.presented
                .iter()
                .find(|o| Some(o.id) == self.editor.selection)
        });
        let axis = if surface == self.three {
            selected
                .filter(|o| o.visible && !o.kind.is_2d())
                .and_then(|o| crate::authoring::interaction::handle(o, vp, local))
        } else {
            None
        };
        let group_two = surface == self.two
            && group.as_ref().is_some_and(|(o, _, _)| {
                o.kind.is_2d()
                    && (glam::Vec2::from(local)
                        - self
                            .presented_view2d
                            .screen(glam::Vec2::new(o.translation[0], o.translation[1]))
                            .unwrap())
                    .length()
                        <= 10.0
            });
        let id = if axis.is_some() || group_two {
            self.editor.selection
        } else if surface == self.two {
            crate::authoring::interaction::pick2_view(&self.presented, local, self.presented_view2d)
        } else {
            crate::authoring::interaction::pick3_with(&self.presented, vp, local, |o| self.model(o))
        };
        if let Err(e) = self.editor.select(id) {
            self.ui_mut().error = e;
            self.input.pointer_released();
            return;
        }
        if let Ok(prepared) = self.prepare(&self.editor.preview) {
            self.publish(prepared)
        }
        self.sync_ui();
        if surface == self.three && axis.is_none() {
            self.input.pointer_released();
            return;
        }
        let Some(target) = id.and_then(|id| self.editor.document.local_target(id)) else {
            self.input.pointer_released();
            return;
        };
        let Some(world_origin3d) = self
            .presented_nodes
            .get(&target.id)
            .map(|n| Vec3::from(n.translation))
        else {
            self.input.pointer_released();
            return;
        };
        let axis_drag = if let Some((index, _)) = axis {
            match crate::authoring::interaction::AxisDrag::new(
                vp,
                world_origin3d,
                index,
                glam::Vec2::from(local),
            ) {
                Some(drag) => Some(drag),
                None => {
                    self.ui_mut().error = "degenerate 3D drag plane".into();
                    self.input.pointer_released();
                    return;
                }
            }
        } else {
            None
        };
        self.ui_mut().queue.clear();
        self.ui_mut().error.clear();
        self.gesture_token += 1;
        self.gesture = Some(crate::authoring::interaction::Gesture {
            view2d: self.presented_view2d,
            view3d: self.presented_view3d,
            axis_drag,
            world_origin3d,
            world_origin: world_origin3d.truncate(),
            token: self.gesture_token,
            stamp: self.editor.stamp,
            target,
            surface,
            start: local,
            last_valid: local,
            axis,
            vp,
            latest: None,
            next_sample: 0,
            shown_sample: 0,
            inflight: None,
            shown: None,
            released: false,
            release_time: None,
            moved: false,
        });
        self.sync_ui();
    }
    pub fn pointer_move(&mut self, p: [f32; 2]) {
        if self.camera_drag.is_some() {
            if !crate::authoring::interaction::inside(p, self.input.window_size) {
                self.cancel_camera_drag();
                return;
            }
            let Some(InputOutcome::Delivered { surface, local }) =
                self.input.pointer_moved(&self.rt.graph, p)
            else {
                self.cancel_camera_drag();
                return;
            };
            if surface != self.three || !local.iter().all(|v| v.is_finite()) {
                self.cancel_camera_drag();
                return;
            }
            let drag = self.camera_drag.as_ref().unwrap();
            let dx = local[0] - drag.last[0];
            let dy = local[1] - drag.last[1];
            self.camera_drag.as_mut().unwrap().last = local;
            // Physical -> logical mapping happened once in the input mapper;
            // the frozen rate is 0.25 degrees per logical pixel, down positive.
            if let Some(play) = &mut self.play {
                play.cam_drag(
                    dx * crate::authoring::runtime_camera::ORBIT_DEG_PER_PX,
                    dy * crate::authoring::runtime_camera::ORBIT_DEG_PER_PX,
                );
            }
            return;
        }
        if self.navigation3d.is_some() {
            self.navigation3d_move(p);
            return;
        }
        if self.navigation.is_some() {
            self.navigation_move(p);
            return;
        }
        if self.gesture.as_ref().is_none_or(|g| g.released) {
            return;
        }
        if !crate::authoring::interaction::inside(p, self.input.window_size) {
            self.cancel_gesture();
            return;
        }
        let Some(InputOutcome::Delivered { surface, local }) =
            self.input.pointer_moved(&self.rt.graph, p)
        else {
            self.cancel_gesture();
            return;
        };
        if let Some(g) = &mut self.gesture {
            if g.surface != surface || !local.iter().all(|x| x.is_finite()) {
                self.cancel_gesture();
                return;
            }
            if !g.sample(local) {
                self.ui_mut().error = "invalid 3D drag sample; retained last valid position".into();
            }
        }
    }
    pub fn pointer_up(&mut self, p: [f32; 2]) {
        if self.camera_drag.take().is_some() {
            self.pointer_move(p);
            self.input.pointer_released();
            return;
        }
        if self.navigation3d.is_some() {
            return;
        }
        if self.navigation.is_some() {
            return;
        }
        if self.gesture.as_ref().is_none_or(|g| g.released) {
            self.input.pointer_released();
            return;
        }
        self.pointer_move(p);
        self.input.pointer_released();
        if let Some(g) = &mut self.gesture {
            if g.moved && g.latest.is_none() {
                let last = g.last_valid;
                g.sample(last);
            }
            g.released = true;
            g.release_time = Some(Instant::now());
            if !g.moved {
                self.restore_gesture(false);
            }
        }
    }
    pub fn cancel_gesture(&mut self) {
        self.restore_gesture(true);
    }
    /// Drop the orbit capture and its unconsumed delta; the pendings clear on
    /// the Play clock (Pause/Stop/Fault/Resume paths), so no stale delta can
    /// leak into a later dispatch.
    pub fn cancel_camera_drag(&mut self) {
        if self.camera_drag.take().is_some() {
            self.input.pointer_released();
        }
    }
    fn restore_gesture(&mut self, cancel_focus: bool) {
        if self.gesture.take().is_some() {
            if cancel_focus {
                self.input.pointer_left_window();
            } else {
                self.input.pointer_released();
            }
            self.presented = self.editor.preview.objects.clone();
            self.presented_nodes = self.editor.preview.nodes.clone();
            self.presented_stamp = self.editor.stamp;
            // accepted VM remains owned by Editor; restoration requires no Auto rebuild or mesh upload.
            let fail = self.fail_resource;
            self.fail_resource = false;
            if let Ok(p) = self.prepare(&self.editor.preview) {
                self.publish(p)
            }
            self.fail_resource = fail;
            self.sync_ui();
        }
    }
    pub(super) fn pump_gesture(&mut self) -> Result<(), String> {
        let Some(mut g) = self.gesture.take() else {
            return Ok(());
        };
        // Reinstall on failure so the common cancellation restores the accepted presentation.
        let result = (|| {
            if g.release_time
                .is_some_and(|t| t.elapsed() > std::time::Duration::from_secs(30))
            {
                return Err("gesture finish deadline".into());
            }
            if g.stamp != self.editor.stamp
                || self.editor.selection != Some(g.target.id)
                || g.vp != self.camera()
                || g.view2d != self.active_view2d()
                || g.view3d != self.view3d
            {
                return Err("stale gesture identity/camera".into());
            }
            if g.inflight.is_none() && g.moved {
                if let Some((sample, p)) = g.latest.take() {
                    let doc = g.document(&self.editor.document, p)?;
                    if g.released && doc == self.editor.document {
                        return Ok(true);
                    }
                    let build = crate::authoring::preview::Building::new(
                        &doc,
                        self.gesture_source
                            .as_deref()
                            .unwrap_or(crate::authoring::preview::SOURCE),
                    )?;
                    g.inflight = Some((sample, doc, build));
                }
            }
            let ready = if let Some((_, _, b)) = &mut g.inflight {
                let start = Instant::now();
                let result = b.pump();
                if self.gesture_pump_us.len() == 256 {
                    self.gesture_pump_us.remove(0);
                }
                self.gesture_pump_us.push(start.elapsed().as_micros());
                result?
            } else {
                false
            };
            if ready {
                let (sample, doc, b) = g.inflight.take().unwrap();
                let preview = b.finish()?;
                if self.gesture_compile_us.len() == 256 {
                    self.gesture_compile_us.remove(0);
                }
                self.gesture_compile_us.push(preview.compile_us);
                let prepared = self.prepare(&preview)?;
                if sample <= g.shown_sample || g.token != self.gesture_token {
                    return Err("stale gesture sample".into());
                }
                if g.released && g.latest.is_none() && sample == g.next_sample {
                    self.editor.commit_gesture(g.stamp, doc, preview)?;
                    self.presented = self.editor.preview.objects.clone();
                    self.presented_nodes = self.editor.preview.nodes.clone();
                    self.presented_stamp = self.editor.stamp;
                    self.publish(prepared);
                    self.accepted += 1;
                    return Ok(true);
                }
                self.presented = preview.objects.clone();
                self.presented_nodes = preview.nodes.clone();
                self.publish(prepared);
                g.shown = Some(preview);
                g.shown_sample = sample;
            }
            Ok(false)
        })();
        match result {
            Ok(true) => {
                self.gesture = Some(g);
                self.restore_gesture(false);
                Ok(())
            }
            Ok(false) => {
                self.gesture = Some(g);
                Ok(())
            }
            Err(e) => {
                self.gesture = Some(g);
                Err(e)
            }
        }
    }
}
