//! DAG-driven surface composition runtime (PLAN-830 T-04).
//!
//! One `SurfaceRuntime` per host. It owns the `CompositionGraph` (core,
//! GPU-agnostic) plus the per-surface CONTENT adapters and their sampled
//! offscreen targets, and turns the graph into GPU work:
//!
//! ```text
//! for surface in graph.render_order():        // children before consumers
//!     render surface's OWN content            // iced / 3D scene / 2D game
//!     for each portal into its views:         // declaration order = layering
//!         composite child texture (3D plane | 2D quad+scissor)
//! present the window-facing view's surface on the window
//! ```
//!
//! Contracts honoured here:
//! - Each surface renders its content once per frame at most (shared mounts
//!   never duplicate the content pass).
//! - Every consumed texture was produced by an EARLIER submission
//!   (`render_order`), so no pass samples a texture it writes.
//! - Offscreen targets are the sampled-usage contract (`new_sampled`); a
//!   non-sampled target is never bound as input.
//! - Same device/queue for everything (host `GpuDevice`), no second device.
//! - Steady state has zero GPU→CPU readback; captures are explicit.

use crate::core::composition::{
    CompositionChange, CompositionError, CompositionGraph, ContentKind, MountKind, PortalId, Rect,
    SurfaceDesc, SurfaceId, ViewDesc, ViewId,
};
use crate::host::iced_surface::{IcedSurface, SurfaceUi};
use crate::renderer::capture::sha256_hex;
use crate::renderer::composition::{FullScreenBlit, PortalQuadPipeline, SurfacePlanePipeline};
use crate::renderer::pipeline::{DrawCmd, QuadPipeline, Shape};
use crate::renderer::target::{OffscreenTarget, WindowTarget};
use crate::renderer::GpuDevice;
use glam::Mat4;

// ---------------------------------------------------------------------------
// Content adapters
// ---------------------------------------------------------------------------

/// What a surface actually renders. Adapters own their own state; the graph
/// only records the kind and the logical size.
pub enum SurfaceContent {
    /// Real iced UI (offscreen renderer + app logic).
    Iced(Box<IcedContent>),
    /// A 3D scene: meshes + camera; plane portals composite over it.
    Scene3d(Box<Scene3dContent>),
    /// 2D content drawn from explicit commands (games, canvas).
    Game2d(Box<Game2dContent>),
}

pub struct IcedContent {
    pub surface: IcedSurface,
    /// The app logic; boxed so the runtime can drive it through the trait.
    pub ui: Box<dyn SurfaceUi<Message = UiMessage> + Send>,
    /// Messages observed in the last frame (evidence + drives the app).
    pub last_messages: Vec<String>,
}

impl IcedContent {
    /// Content for the RUNTIME path: no self-owned texture; every frame
    /// renders into the surface's slot target (one texture per surface).
    pub fn borrowed(
        gpu: &GpuDevice,
        size: [f32; 2],
        ui: Box<dyn SurfaceUi<Message = UiMessage> + Send>,
    ) -> Self {
        IcedContent {
            surface: IcedSurface::new_borrowed(
                gpu,
                iced_wgpu::core::Size::new(size[0], size[1]),
                1.0,
            ),
            ui,
            last_messages: Vec::new(),
        }
    }
}

/// Unified message type for the prototype's iced surfaces (the runtime
/// records them as text so any app logic can share one adapter).
pub type UiMessage = String;

pub struct ScreenOverlay {
    pub quad: QuadPipeline,
    pub commands: Vec<DrawCmd>,
}
impl ScreenOverlay {
    pub fn new(gpu: &GpuDevice) -> Result<Self, crate::renderer::RendererError> {
        Ok(Self {
            quad: QuadPipeline::new(gpu, OffscreenTarget::FORMAT)?,
            commands: vec![],
        })
    }
}
pub struct Scene3dContent {
    pub screen_overlay: Option<ScreenOverlay>,
    pub text_overlay: Option<TextOverlay>,
    pub mesh: crate::render3d::MeshPipeline,
    pub store: crate::render3d::MeshStore,
    pub instances: Vec<crate::render3d::MeshInstance>,
    /// Clear color (sky).
    pub sky: [f32; 4],
    /// 3D-surface camera (planes are placed by the portal mounts).
    pub view_proj: Mat4,
}

pub struct TextOverlay {
    pub renderer: crate::render2d::TextRenderer,
    pub glyphs: Vec<crate::render2d::GlyphQuad>,
}

pub struct Game2dContent {
    /// Optional renderer-owned text data; contains no language or game state.
    pub text_overlay: Option<TextOverlay>,
    pub quad: QuadPipeline,
    /// PLAN-004 textured visual pass drawn over the flat quads, plus the
    /// persistent content-identified image cache it draws from.
    pub sprite: crate::renderer::sprite::SpritePipeline,
    pub sprite_store: crate::renderer::sprite::SpriteStore,
    pub sprites: Vec<crate::renderer::sprite::SpriteCmd>,
    pub clear: crate::core::scene::Color,
    pub commands: Vec<DrawCmd>,
}

impl Game2dContent {
    pub fn simple(gpu: &GpuDevice, clear: crate::core::scene::Color) -> Self {
        Game2dContent {
            text_overlay: None,
            quad: QuadPipeline::new(gpu, OffscreenTarget::FORMAT).expect("2d quad pipeline"),
            sprite: crate::renderer::sprite::SpritePipeline::new(gpu, OffscreenTarget::FORMAT)
                .expect("2d sprite pipeline"),
            sprite_store: Default::default(),
            sprites: Vec::new(),
            clear,
            commands: Vec::new(),
        }
    }
}

impl SurfaceContent {
    pub fn kind(&self) -> ContentKind {
        match self {
            SurfaceContent::Iced(_) => ContentKind::Iced,
            SurfaceContent::Scene3d(_) => ContentKind::Scene3d,
            SurfaceContent::Game2d(_) => ContentKind::Canvas2d,
        }
    }

    /// The host device that created this content's GPU objects, when known.
    /// A composition runtime refuses content from another device BEFORE any
    /// state change (review r1 F09 — a same-device claim must be checked on
    /// the real object, not a report string). EVERY adapter reports its
    /// identity; `None` means unknown or heterogeneous device identity and is refused
    /// rather than silently admitted (review r2 F04).
    pub fn device_ordinal(&self) -> Option<u64> {
        match self {
            SurfaceContent::Iced(c) => Some(c.surface.device_ordinal),
            SurfaceContent::Game2d(g) => {
                let quad = g.quad.device_ordinal?;
                if let Some(text) = &g.text_overlay {
                    if text.renderer.device_ordinal != quad {
                        return None;
                    }
                }
                if g.sprite.device_ordinal? != quad
                    || g.sprite_store.device_owners().any(|o| o != Some(quad))
                {
                    return None;
                }
                Some(quad)
            }
            SurfaceContent::Scene3d(s) => {
                let owner = s.mesh.device_ordinal?;
                if s.store.device_owners().any(|o| o != Some(owner))
                    || s.text_overlay
                        .as_ref()
                        .is_some_and(|t| t.renderer.device_ordinal != owner)
                    || s.screen_overlay
                        .as_ref()
                        .is_some_and(|o| o.quad.device_ordinal != Some(owner))
                {
                    return None;
                }
                Some(owner)
            }
        }
    }
    /// Validate every resource, including heterogeneous adapters, before allocation/work.
    pub fn validate_device(&self, expected: u64) -> Result<(), CompositionError> {
        let check = |ordinal: Option<u64>| match ordinal {
            Some(actual) if actual == expected => Ok(()),
            Some(actual) => Err(CompositionError::DeviceMismatch { expected, actual }),
            None => Err(CompositionError::DeviceUnverifiable {
                detail: "content adapter provenance unavailable".into(),
            }),
        };
        match self {
            Self::Game2d(game) => {
                check(game.quad.device_ordinal)?;
                check(game.sprite.device_ordinal)?;
                for owner in game.sprite_store.device_owners() {
                    check(owner)?;
                }
                if let Some(text) = &game.text_overlay {
                    check(Some(text.renderer.device_ordinal))?;
                }
                Ok(())
            }
            Self::Scene3d(scene) => {
                check(scene.mesh.device_ordinal)?;
                for owner in scene.store.device_owners() {
                    check(owner)?;
                }
                if let Some(overlay) = &scene.screen_overlay {
                    check(overlay.quad.device_ordinal)?;
                }
                for instance in &scene.instances {
                    if scene
                        .store
                        .get(&instance.mesh.0, &instance.mesh.1)
                        .is_none()
                    {
                        return Err(CompositionError::InvalidSurface {
                            detail: "3d instance mesh missing".into(),
                        });
                    }
                    if !instance.model.is_finite() {
                        return Err(CompositionError::InvalidSurface {
                            detail: "nonfinite 3d matrix".into(),
                        });
                    }
                }
                if let Some(text) = &scene.text_overlay {
                    check(Some(text.renderer.device_ordinal))?;
                }
                Ok(())
            }
            _ => check(self.device_ordinal()),
        }
    }

    fn uploads(&self) -> Option<crate::renderer::UploadStats> {
        match self {
            Self::Game2d(game) => {
                let mut out = game.quad.uploads.snapshot();
                out.merge(&game.sprite.uploads.snapshot());
                if let Some(text) = &game.text_overlay {
                    out.merge(&text.renderer.uploads.snapshot());
                }
                Some(out)
            }
            Self::Scene3d(scene) => {
                let mut out = scene.mesh.uploads.snapshot();
                out.merge(&scene.mesh.editor_grid_uploads());
                out.merge(&scene.store.uploads.snapshot());
                if let Some(overlay) = &scene.screen_overlay {
                    out.merge(&overlay.quad.uploads.snapshot());
                }
                if let Some(text) = &scene.text_overlay {
                    out.merge(&text.renderer.uploads.snapshot());
                }
                Some(out)
            }
            Self::Iced(_) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-surface slot
// ---------------------------------------------------------------------------

pub struct Slot {
    pub target: OffscreenTarget,
    pub content: SurfaceContent,
    /// Frames of content rendered since creation (evidence).
    pub renders: u64,
    /// Value of `stats.frames` when this slot's content last ran. Used to
    /// prove a dirty child was rendered BEFORE its consumer sampled it
    /// (dependency-order gate, review r1 F09).
    pub last_render_frame: u64,
    /// Fingerprint of the last 3D content pass (instances/camera/clear). A
    /// 3D surface renders every frame, so consumers must be invalidated when
    /// the fingerprint changes even though the graph is clean (review r2
    /// F01: live content that consumers sample).
    pub scene_fingerprint: u64,
}

/// Per-frame accounting (V-09 owns the full metrics; these are the counters
/// the composition gates read).
#[derive(Debug, Clone, Default)]
pub struct CompositionStats {
    pub surfaces_rendered: u64,
    pub portal_composites: u64,
    pub plane_composites: u64,
    pub quad_composites: u64,
    /// Content passes that were SKIPPED because the surface was clean.
    pub dirty_skips: u64,
    pub readbacks: u64,
    pub frames: u64,
    /// REAL GPU target creations / drops (resize or surface add/remove).
    /// Acceptance gates count these instead of inferring rebuilds from loop
    /// shape (review r1 F09).
    pub target_creations: u64,
    pub target_drops: u64,
    /// Per-surface content passes in the LAST frame (shared-mount and
    /// one-pass-per-surface evidence).
    pub last_frame_renders: std::collections::BTreeMap<SurfaceId, u64>,
}

impl CompositionStats {
    /// Targets the runtime still owns: every creation is balanced by a real
    /// drop when a target is replaced, a failed preparation is discarded, or
    /// a slot is removed. No external pin exists, so this MUST equal the
    /// number of live slot targets (V-07/V-09 accounting, review r2 F05).
    pub fn live_targets(&self) -> u64 {
        self.target_creations.saturating_sub(self.target_drops)
    }
}

pub struct SurfaceRuntime {
    pub graph: CompositionGraph,
    gpu: GpuDevice,
    slots: std::collections::HashMap<SurfaceId, Slot>,
    plane: SurfacePlanePipeline,
    portal_quad: PortalQuadPipeline,
    blit: FullScreenBlit,
    window_format: Option<wgpu::TextureFormat>,
    pub stats: CompositionStats,
    /// Per-surface render order of the last frame (evidence).
    pub last_order: Vec<SurfaceId>,
    /// False = deliberately skip the scissor in the real quad path
    /// (negative-control fault injection; production is always `true`).
    scissor_enabled: bool,
    /// True = deliver each surface's event list twice (negative-control fault).
    duplicate_events: bool,
    /// True = run composites BEFORE content passes (negative-control fault:
    /// a dirty child would be sampled stale).
    composite_before_render: bool,
    /// False = the quad pipeline uses opaque replace instead of the
    /// premultiplied-over contract (negative-control fault).
    premultiplied_portal_blend: bool,
    /// False = `upload_stats()` omits the CONTENT adapters (V-08 hook: the
    /// real writes still happen, only the aggregation is skipped, so the
    /// total-vs-parts gate must detect it — review r4 F-830-R4-02).
    content_upload_aggregation: bool,
}

impl SurfaceRuntime {
    pub fn new(gpu: GpuDevice) -> Result<Self, CompositionError> {
        let plane =
            SurfacePlanePipeline::new(gpu.device(), OffscreenTarget::FORMAT).map_err(|e| {
                CompositionError::InvalidSurface {
                    detail: format!("plane pipeline: {e}"),
                }
            })?;
        let portal_quad = PortalQuadPipeline::new(gpu.device(), OffscreenTarget::FORMAT);
        let blit = FullScreenBlit::new(gpu.device(), OffscreenTarget::FORMAT);
        Ok(SurfaceRuntime {
            graph: CompositionGraph::new(),
            gpu,
            slots: Default::default(),
            plane,
            portal_quad,
            blit,
            window_format: None,
            stats: CompositionStats::default(),
            last_order: Vec::new(),
            scissor_enabled: true,
            duplicate_events: false,
            composite_before_render: false,
            premultiplied_portal_blend: true,
            content_upload_aggregation: true,
        })
    }

    /// Negative-control hooks (V-08). Each changes the REAL frame path; the
    /// acceptance gates prove the corresponding observation catches it.
    pub fn set_scissor_enabled(&mut self, enabled: bool) {
        self.scissor_enabled = enabled;
    }

    /// V-08 measurement hook: when false the plane pipeline's REAL 128B write
    /// still happens but is not recorded (review r4 F-830-R4-02).
    pub fn set_plane_upload_accounting(&mut self, enabled: bool) {
        self.plane.accounting = enabled;
    }

    /// V-08 measurement hook: when false the runtime's total omits the
    /// content adapters while their real writes continue.
    pub fn set_content_upload_aggregation(&mut self, enabled: bool) {
        self.content_upload_aggregation = enabled;
    }

    pub fn set_duplicate_events(&mut self, duplicate: bool) {
        self.duplicate_events = duplicate;
    }

    pub fn set_composite_before_render(&mut self, sabotage: bool) {
        self.composite_before_render = sabotage;
    }

    /// Rebuild the portal quad pipeline with the WRONG (opaque) blend. The
    /// same alpha gate then observes a different pixel — proving the gate
    /// reads a real blend change.
    pub fn set_premultiplied_portal_blend(&mut self, premultiplied: bool) {
        if self.premultiplied_portal_blend == premultiplied {
            return;
        }
        self.premultiplied_portal_blend = premultiplied;
        self.portal_quad = PortalQuadPipeline::new_with_alpha(
            self.gpu.device(),
            OffscreenTarget::FORMAT,
            premultiplied,
        );
    }

    pub fn gpu(&self) -> &GpuDevice {
        &self.gpu
    }

    /// Rebuild the present blit for a window's actual surface format.
    pub fn set_window_format(&mut self, format: wgpu::TextureFormat) {
        self.blit = FullScreenBlit::new(self.gpu.device(), format);
        self.window_format = Some(format);
    }

    pub fn slot(&self, id: SurfaceId) -> Option<&Slot> {
        self.slots.get(&id)
    }

    /// Number of GPU targets the runtime currently owns (one per slot).
    /// Together with `stats.live_targets()` this closes the accounting: no
    /// target may outlive its slot and no drop may be unrecorded.
    pub fn live_target_slots(&self) -> usize {
        self.slots.len()
    }

    /// Byte size of the offscreen targets the runtime OWNS right now, from
    /// their real dimensions (`Rgba8UnormSrgb` = 4 B/px). This is owned
    /// texture memory, NOT the process working set and NOT driver residency
    /// (review r3 F-830-R3-03: the two must be reported separately).
    pub fn owned_target_bytes(&self) -> u64 {
        self.slots
            .values()
            .map(|s| s.target.width as u64 * s.target.height as u64 * 4)
            .sum()
    }

    /// Upload counters of the 3D plane composition pipeline (one 128B uniform
    /// write per plane composite).
    pub fn plane_uploads(&self) -> crate::renderer::UploadStats {
        self.plane.uploads.snapshot()
    }

    /// Upload counters of the 2D portal composition pipeline (one 160B
    /// uniform write per drawn portal).
    pub fn portal_quad_uploads(&self) -> crate::renderer::UploadStats {
        self.portal_quad.uploads.snapshot()
    }

    /// Upload counters of ONE content adapter, when it is instrumented
    /// (`None` for iced content, whose uploads happen inside
    /// `iced_wgpu::Renderer::present`).
    pub fn content_uploads_of(&self, id: SurfaceId) -> Option<crate::renderer::UploadStats> {
        let slot = self.slots.get(&id)?;
        slot.content.uploads()
    }

    /// Aggregate of every instrumented CONTENT adapter (2D quads + text overlays + 3D mesh).
    /// Kept as its own entry point so an acceptance gate can compare the
    /// runtime's total against its parts (review r4 F-830-R4-02: omitting
    /// this aggregation must be observable).
    pub fn content_uploads(&self) -> crate::renderer::UploadStats {
        let mut out = crate::renderer::UploadStats::default();
        for slot in self.slots.values() {
            if let Some(uploads) = slot.content.uploads() {
                out.merge(&uploads);
            }
        }
        out
    }

    /// Real buffer-rewrite accounting for the whole runtime path: the two
    /// composition pipelines plus each content adapter that has counters.
    /// Iced's INTERNAL uploads are not instrumented here (they happen inside
    /// `iced_wgpu::Renderer::present`); that boundary is named in the report.
    pub fn upload_stats(&self) -> crate::renderer::UploadStats {
        let mut out = self.plane_uploads();
        out.merge(&self.portal_quad_uploads());
        if self.content_upload_aggregation {
            out.merge(&self.content_uploads());
        }
        out
    }

    /// The composition pipelines' own uploads (the per-composite uniform
    /// writes), reported separately so a gate can check them against the
    /// actual composite counts.
    pub fn composition_upload_stats(&self) -> crate::renderer::UploadStats {
        let mut out = self.plane_uploads();
        out.merge(&self.portal_quad_uploads());
        out
    }

    pub fn content(&self, id: SurfaceId) -> Option<&SurfaceContent> {
        self.slots.get(&id).map(|s| &s.content)
    }

    pub fn content_mut(&mut self, id: SurfaceId) -> Option<&mut SurfaceContent> {
        self.slots.get_mut(&id).map(|s| &mut s.content)
    }

    /// Configure the editor-only depth grid on the same device as its scene.
    pub fn set_editor_grid(
        &mut self,
        id: SurfaceId,
        vertices: Option<&[crate::render3d::editor_grid::EditorGridVertex]>,
    ) -> Result<(), String> {
        let slot = self
            .slots
            .get_mut(&id)
            .ok_or("editor grid surface missing")?;
        let SurfaceContent::Scene3d(scene) = &mut slot.content else {
            return Err("editor grid requires a 3D surface".into());
        };
        let before = scene.mesh.editor_grid_revision();
        scene.mesh.set_editor_grid(&self.gpu, vertices)?;
        if before != scene.mesh.editor_grid_revision() {
            self.graph.mark_content_dirty(id);
        }
        Ok(())
    }

    pub fn target(&self, id: SurfaceId) -> Option<&OffscreenTarget> {
        self.slots.get(&id).map(|s| &s.target)
    }

    /// Add a surface + its content adapter transactionally: the GPU target is
    /// created FIRST and the graph is committed only when every resource step
    /// succeeded. A failed target leaves the graph/identity space untouched
    /// (review r1 F03).
    pub fn add_surface(
        &mut self,
        size: [f32; 2],
        content: SurfaceContent,
    ) -> Result<SurfaceId, CompositionError> {
        // Fail closed: content whose adapter cannot prove same-device identity
        // is NOT admitted under a contract that claims one verified device.
        content.validate_device(self.gpu.token().ordinal)?;
        let target = OffscreenTarget::new_sampled(
            &self.gpu,
            size[0].round().max(1.0) as u32,
            size[1].round().max(1.0) as u32,
        )
        .map_err(|e| CompositionError::InvalidSurface {
            detail: format!("target: {e}"),
        })?;
        // The texture now EXISTS: count it even if a later step fails, so the
        // accounting never hides a created-then-discarded resource.
        self.stats.target_creations += 1;
        let mints = match self.graph.submit_one(CompositionChange::AddSurface {
            desc: SurfaceDesc {
                logical_size: size,
                content: content.kind(),
            },
        }) {
            Ok(m) => m,
            Err(errors) => {
                self.stats.target_drops += 1;
                let first = errors
                    .into_iter()
                    .next()
                    .unwrap_or(CompositionError::InvalidSurface {
                        detail: "batch rejected".into(),
                    });
                return Err(first);
            }
        };
        let id = mints.first_surface().expect("mint");
        self.slots.insert(
            id,
            Slot {
                target,
                content,
                renders: 0,
                last_render_frame: 0,
                scene_fingerprint: 0,
            },
        );
        Ok(id)
    }

    pub fn add_view(
        &mut self,
        surface: SurfaceId,
        presents_to_window: bool,
    ) -> Result<ViewId, CompositionError> {
        self.graph
            .submit_one(CompositionChange::AddView {
                desc: ViewDesc {
                    surface,
                    presents_to_window,
                },
            })
            .map_err(|e| {
                e.into_iter()
                    .next()
                    .unwrap_or(CompositionError::UnknownSurface(surface))
            })
            .map(|m| m.first_view().expect("mint"))
    }

    pub fn add_portal(
        &mut self,
        parent: ViewId,
        child: SurfaceId,
        mount: MountKind,
        interactive: bool,
    ) -> Result<PortalId, CompositionError> {
        self.graph
            .submit_one(CompositionChange::AddPortal {
                desc: crate::core::composition::PortalDesc {
                    parent,
                    child,
                    mount,
                    interactive,
                },
            })
            .map_err(|e| {
                e.into_iter()
                    .next()
                    .unwrap_or(CompositionError::UnknownSurface(child))
            })
            .map(|m| m.first_portal().expect("mint"))
    }

    /// Remove a portal (keeps both surfaces and their content).
    pub fn remove_portal(&mut self, portal: PortalId) -> Result<(), CompositionError> {
        self.graph
            .submit_one(CompositionChange::RemovePortal(portal))
            .map(|_| ())
            .map_err(|e| {
                e.into_iter()
                    .next()
                    .unwrap_or(CompositionError::UnknownPortal(portal))
            })
    }

    /// Drop a surface's GPU slot (after the graph allows the removal).
    pub fn drop_surface(&mut self, id: SurfaceId) -> Result<(), CompositionError> {
        self.graph
            .submit_one(CompositionChange::RemoveSurface(id))
            .map_err(|e| {
                e.into_iter()
                    .next()
                    .unwrap_or(CompositionError::UnknownSurface(id))
            })?;
        if self.slots.remove(&id).is_some() {
            self.stats.target_drops += 1;
        }
        Ok(())
    }

    /// Resize a surface transactionally: the new GPU target is prepared (and
    /// size-checked) BEFORE the graph records the new size, so an invalid or
    /// failed resource leaves the last-good surface intact (review r1 F03).
    pub fn resize_surface(
        &mut self,
        id: SurfaceId,
        size: [f32; 2],
    ) -> Result<(), CompositionError> {
        if !size.iter().all(|v| v.is_finite()) || size[0] <= 0.0 || size[1] <= 0.0 {
            return Err(CompositionError::InvalidSurface {
                detail: "logical size must be finite and positive".into(),
            });
        }
        // Classified "unknown surface" before touching any resource.
        let current_size = self.graph.surface(id)?.desc.logical_size;
        if current_size == size {
            // No-op: no texture recreation, no dirtying (the target is warm
            // and the content unchanged). The lifecycle gate counts REAL
            // rebuilds, so identical resizes must not fake one.
            return Ok(());
        }
        if !self.slots.contains_key(&id) {
            return Err(CompositionError::InvalidSurface {
                detail: "surface has no GPU slot".into(),
            });
        }
        let new_target = OffscreenTarget::new_sampled(
            &self.gpu,
            size[0].round().max(1.0) as u32,
            size[1].round().max(1.0) as u32,
        )
        .map_err(|e| CompositionError::InvalidSurface {
            detail: format!("resize: {e}"),
        })?;
        // Counted as soon as the texture exists: a failed commit discards it
        // and must record that drop too (review r2 F05).
        self.stats.target_creations += 1;
        // Iced's logical size is part of the content adapter; for the borrowed
        // (runtime) path this cannot fail, but keep the fallible step before
        // the graph commit so a failure never half-commits.
        if let Some(SurfaceContent::Iced(iced)) = self.slots.get_mut(&id).map(|s| &mut s.content) {
            if let Err(e) = iced
                .surface
                .resize(&self.gpu, iced_wgpu::core::Size::new(size[0], size[1]))
            {
                self.stats.target_drops += 1;
                return Err(CompositionError::InvalidSurface {
                    detail: format!("iced resize: {e}"),
                });
            }
        }
        if let Err(errors) = self
            .graph
            .submit_one(CompositionChange::SetSurfaceSize(id, size))
        {
            self.stats.target_drops += 1;
            return Err(errors
                .into_iter()
                .next()
                .unwrap_or(CompositionError::UnknownSurface(id)));
        }
        if let Some(slot) = self.slots.get_mut(&id) {
            // The previous target is dropped HERE by the assignment; count
            // that real release, not just explicit slot removals.
            slot.target = new_target;
            self.stats.target_drops += 1;
        }
        Ok(())
    }

    // -- frame --------------------------------------------------------------

    /// One composition frame. `events_per_surface` lets the host deliver
    /// input to specific iced surfaces (T-05/T-06 wire the real routing).
    pub fn frame(
        &mut self,
        window_target: Option<&mut WindowTarget>,
        events: &std::collections::HashMap<SurfaceId, Vec<iced_wgpu::core::Event>>,
    ) -> Result<(), CompositionError> {
        self.frame_with_cursors(window_target, events, &Default::default())
    }

    /// Frame with an explicit per-surface cursor position (local logical px).
    /// Widgets only react when the cursor is `Available` at their bounds, so
    /// a routed pointer MUST be handed to the receiving surface.
    pub fn frame_with_cursors(
        &mut self,
        window_target: Option<&mut WindowTarget>,
        events: &std::collections::HashMap<SurfaceId, Vec<iced_wgpu::core::Event>>,
        cursors: &std::collections::HashMap<SurfaceId, [f32; 2]>,
    ) -> Result<(), CompositionError> {
        // Neutral content is mutable; recheck every adapter before GPU work,
        // including a text overlay replaced after add_surface admission.
        for slot in self.slots.values() {
            slot.content.validate_device(self.gpu.token().ordinal)?;
        }
        self.stats.frames += 1;
        let frame_index = self.stats.frames;
        self.stats.last_frame_renders.clear();
        let order = self.graph.render_order();
        self.last_order = order.clone();
        let mut dirty: std::collections::BTreeSet<SurfaceId> =
            self.graph.dirty_surfaces().collect();

        // 0) SINGLE REAL EDIT TARGET (review r2 F02 / r3 F-830-R3-01 /
        //    r4 F-830-R4-01). The ROUTER's state is authoritative:
        //    - Target(id): only that surface may keep widget focus;
        //    - Cancelled: focus was lost / the path died AFTER routing, so a
        //      still-pending batch must NOT revive any editor state;
        //    - Unset: nothing has routed yet (hosts may focus freely).
        //    A target whose surface no longer exists becomes Cancelled.
        let mut focus_state = self.graph.routed_focus_state();
        if let crate::core::composition::RoutedFocus::Target(target) = focus_state {
            if !self.slots.contains_key(&target) {
                self.graph.cancel_routed_focus();
                focus_state = crate::core::composition::RoutedFocus::Cancelled;
            }
        }
        let focus_target = match focus_state {
            crate::core::composition::RoutedFocus::Target(id) => Some(id),
            _ => None,
        };
        let deny_focus = matches!(
            focus_state,
            crate::core::composition::RoutedFocus::Cancelled
        );

        if self.composite_before_render {
            // Negative control: sample dirty consumers BEFORE their content
            // pass. The dependency-order check below must reject the frame.
            for id in &order {
                if dirty.contains(id) {
                    self.composite_portals_of(*id, &dirty, frame_index)?;
                }
            }
        }

        let mut rendered: std::collections::BTreeSet<SurfaceId> = Default::default();
        for id in &order {
            // 1) The surface's OWN content (skip clean surfaces).
            let needs_render = dirty.contains(id)
                || matches!(
                    self.slots.get(id).map(|s| &s.content),
                    Some(SurfaceContent::Iced(_)) | Some(SurfaceContent::Scene3d(_))
                );
            if needs_render {
                let empty: Vec<iced_wgpu::core::Event> = Vec::new();
                let slice = events.get(id).map(|v| v.as_slice()).unwrap_or(&empty);
                let allowed = !deny_focus && focus_target.map(|t| t == *id).unwrap_or(true);
                let changed = if self.duplicate_events {
                    let mut doubled = slice.to_vec();
                    doubled.extend_from_slice(slice);
                    self.render_content(*id, &doubled, cursors.get(id).copied(), allowed)?
                } else {
                    self.render_content(*id, slice, cursors.get(id).copied(), allowed)?
                };
                self.stats.surfaces_rendered += 1;
                rendered.insert(*id);
                if changed {
                    // The content pass changed the surface's OUTPUT (a real
                    // widget message, a live 3D state update, an explicit
                    // content edit). Every consumer must re-sample it in the
                    // SAME frame; `render_order` guarantees they come later.
                    self.mark_frame_dirty(&mut dirty, *id);
                }
            } else {
                self.stats.dirty_skips += 1;
            }
        }

        // 2) Composite every portal into its parent's target, in DAG order.
        //    A parent is composited ONLY when its own content was (re)rendered
        //    this frame — its texture is then a fresh base. Compositing onto a
        //    clean parent would blend the same translucent layers again over
        //    last frame's result (review r1 F02) and keep an unmounted layer's
        //    old pixels alive (F01).
        if !self.composite_before_render {
            for id in &order {
                if rendered.contains(id) {
                    self.composite_portals_of(*id, &dirty, frame_index)?;
                }
            }
        }

        // 3) Present the window-facing view (the top surface's texture).
        if let Some(window) = window_target {
            if let Some(top) = self.window_surface() {
                let blit = &self.blit;
                let gpu = &self.gpu;
                let src = self.slots.get(&top).map(|s| s.target.view());
                if let Some(src) = src {
                    let _ = window.present_frame(gpu, |view| {
                        let mut encoder =
                            gpu.device()
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("agp surface present"),
                                });
                        blit.blit(gpu.device(), &mut encoder, view, src);
                        gpu.queue().submit(Some(encoder.finish()));
                    });
                }
            }
        }
        self.graph.clear_dirty();
        Ok(())
    }

    /// Mark a surface AND every consumer chain dirty for THIS frame.
    fn mark_frame_dirty(&self, dirty: &mut std::collections::BTreeSet<SurfaceId>, id: SurfaceId) {
        dirty.insert(id);
        for consumer in self.graph.consumer_chain(id) {
            dirty.insert(consumer);
        }
    }

    /// Release the real iced widget focus of one surface WHEN it holds any.
    /// Returns whether a focus was actually released (only then does the
    /// caller treat the output as changed / close a composition).
    fn release_iced_focus_if_active(&mut self, id: SurfaceId) -> bool {
        let Some(mut slot) = self.slots.remove(&id) else {
            return false;
        };
        let released = match &mut slot.content {
            SurfaceContent::Iced(iced) => {
                let ui: &mut dyn SurfaceUi<Message = String> = iced.ui.as_mut();
                iced.surface.unfocus(&*ui)
            }
            _ => false,
        };
        self.slots.insert(id, slot);
        released
    }

    /// The REAL caret rectangle of a surface's focused iced widget, in that
    /// surface's LOCAL logical px (review r2 F03). `None` when the surface has
    /// no focused text input (or is not iced content).
    pub fn ime_caret_rect(&self, id: SurfaceId) -> Option<iced_wgpu::core::Rectangle> {
        match self.slots.get(&id).map(|s| &s.content) {
            Some(SurfaceContent::Iced(iced)) => iced.surface.caret_rect(),
            _ => None,
        }
    }

    /// The REAL caret anchor (top-left) in LOCAL logical px.
    pub fn ime_caret_local(&self, id: SurfaceId) -> Option<[f32; 2]> {
        self.ime_caret_rect(id).map(|r| [r.x, r.y])
    }

    /// Real bounds of a focusable widget in a surface's LOCAL logical px
    /// (used by gates to click the ACTUAL control instead of a fixed point).
    pub fn iced_widget_bounds(
        &mut self,
        id: SurfaceId,
        widget: iced_wgpu::core::widget::Id,
    ) -> Option<iced_wgpu::core::Rectangle> {
        let Some(mut slot) = self.slots.remove(&id) else {
            return None;
        };
        let out = match &mut slot.content {
            SurfaceContent::Iced(iced) => {
                let ui: &mut dyn SurfaceUi<Message = String> = iced.ui.as_mut();
                iced.surface.widget_bounds(&*ui, widget)
            }
            _ => None,
        };
        self.slots.insert(id, slot);
        out
    }

    /// The surface of the window-facing view, if any.
    pub fn window_surface(&self) -> Option<SurfaceId> {
        self.graph
            .views()
            .iter()
            .find(|v| v.desc.presents_to_window)
            .map(|v| v.desc.surface)
            .filter(|s| self.slots.contains_key(s))
    }

    fn render_content(
        &mut self,
        id: SurfaceId,
        events: &[iced_wgpu::core::Event],
        cursor_local: Option<[f32; 2]>,
        focus_allowed: bool,
    ) -> Result<bool, CompositionError> {
        let (w, h) = {
            let Some(slot) = self.slots.get(&id) else {
                return Ok(false);
            };
            (slot.target.width, slot.target.height)
        };
        // Take the content out to satisfy the borrow checker; it goes back.
        let mut slot = match self.slots.remove(&id) {
            Some(s) => s,
            None => return Ok(false),
        };
        let gpu = &self.gpu;
        let mut changed = false;
        let mut sprite_error: Option<CompositionError> = None;
        match &mut slot.content {
            SurfaceContent::Iced(iced) => {
                let cursor = match cursor_local {
                    Some(p) => iced_wgpu::core::mouse::Cursor::Available(
                        iced_wgpu::core::Point::new(p[0], p[1]),
                    ),
                    None => iced_wgpu::core::mouse::Cursor::Unavailable,
                };
                // Render into the SLOT target (the single texture that is
                // composited, captured and sampled) — never a second one.
                // Only the router's target may keep widget focus this frame.
                iced.surface.focus_allowed = focus_allowed;
                let messages =
                    iced.surface
                        .frame_into(iced.ui.as_mut(), events, cursor, &slot.target);
                iced.last_messages = messages;
                changed = iced.surface.last_change.output_changed;
            }
            SurfaceContent::Scene3d(scene) => {
                scene.mesh.render_with(
                    gpu.device(),
                    gpu.queue(),
                    slot.target.view(),
                    w,
                    h,
                    scene.sky,
                    &scene.view_proj,
                    &crate::render3d::sample_light(),
                    &scene.store,
                    &scene.instances,
                    true,
                );
                if let Some(overlay) = &scene.screen_overlay {
                    overlay
                        .quad
                        .set_ortho_camera_with(gpu.queue(), 0.0, w as f32, h as f32, 0.0);
                    overlay.quad.render_with(
                        gpu.device(),
                        gpu.queue(),
                        slot.target.view(),
                        None,
                        &overlay.commands,
                    );
                }
                if let Some(text) = &scene.text_overlay {
                    text.renderer
                        .set_ortho_camera(gpu, 0.0, w as f32, h as f32, 0.0);
                    text.renderer.render(gpu, slot.target.view(), &text.glyphs);
                }
                // A 3D surface renders every frame; consumers must refresh
                // when its live state actually changed (review r2 F01).
                let fingerprint = scene_fingerprint(scene);
                changed = fingerprint != slot.scene_fingerprint;
                slot.scene_fingerprint = fingerprint;
            }
            SurfaceContent::Game2d(game) => {
                game.quad
                    .set_ortho_camera_with(gpu.queue(), 0.0, w as f32, h as f32, 0.0);
                game.quad.render_with(
                    gpu.device(),
                    gpu.queue(),
                    slot.target.view(),
                    Some(game.clear),
                    &game.commands,
                );
                // PLAN-004: textured visuals draw over the flat quads with the
                // same ortho camera; a missing texture rejects the frame rather
                // than silently drawing nothing. The slot is re-inserted before
                // the error propagates, so a failed pass never loses the surface
                // (its last accepted target stays readable).
                if !game.sprites.is_empty() {
                    game.sprite
                        .set_ortho_camera_with(gpu.queue(), 0.0, w as f32, h as f32, 0.0);
                    if let Err(e) = game.sprite.render_with(
                        gpu.device(),
                        gpu.queue(),
                        slot.target.view(),
                        &game.sprites,
                        &game.sprite_store,
                    ) {
                        sprite_error = Some(CompositionError::InvalidSurface {
                            detail: format!("sprite pass: {e:?}"),
                        });
                    }
                }
                if sprite_error.is_none() {
                    if let Some(text) = &game.text_overlay {
                        text.renderer
                            .set_ortho_camera(gpu, 0.0, w as f32, h as f32, 0.0);
                        text.renderer.render(gpu, slot.target.view(), &text.glyphs);
                    }
                }
            }
        }
        slot.renders += 1;
        slot.last_render_frame = self.stats.frames;
        *self.stats.last_frame_renders.entry(id).or_insert(0) += 1;
        self.slots.insert(id, slot);
        if let Some(error) = sprite_error {
            return Err(error);
        }
        Ok(changed)
    }

    /// Composite every portal mounted into `parent_surface`'s views.
    fn composite_portals_of(
        &mut self,
        parent_surface: SurfaceId,
        dirty: &std::collections::BTreeSet<SurfaceId>,
        frame_index: u64,
    ) -> Result<(), CompositionError> {
        let views: Vec<ViewId> = self
            .graph
            .surface(parent_surface)
            .map(|s| s.views.clone())
            .unwrap_or_default();
        for view_id in views {
            let portals = self
                .graph
                .view(view_id)
                .map(|v| v.portals.clone())
                .unwrap_or_default();
            for portal_id in portals {
                self.composite_portal(parent_surface, portal_id, dirty, frame_index)?;
            }
        }
        Ok(())
    }

    /// Draw one portal's child texture into the parent surface's target.
    fn composite_portal(
        &mut self,
        parent_surface: SurfaceId,
        portal_id: PortalId,
        dirty: &std::collections::BTreeSet<SurfaceId>,
        frame_index: u64,
    ) -> Result<(), CompositionError> {
        let portal = self.graph.portal(portal_id)?.clone();
        // A child whose content changed THIS frame must have been rendered
        // before any consumer samples it (submit-order DAG). When the order
        // hook inverts the phases, this classifies the violation instead of
        // silently compositing a stale texture.
        if dirty.contains(&portal.desc.child) {
            if let Some(slot) = self.slots.get(&portal.desc.child) {
                if slot.last_render_frame != frame_index {
                    return Err(CompositionError::DependencyOrder {
                        detail: format!(
                            "child {} was dirty this frame but its content has not rendered yet",
                            portal.desc.child.token()
                        ),
                    });
                }
            }
        }
        // Extract everything we need from the slots up front (owned clones),
        // so no borrow of `self.slots` is alive across the mutable pipeline
        // use below.
        let (child_view, child_size) = match self.slots.get(&portal.desc.child) {
            Some(slot) => (
                slot.target.view().clone(),
                [slot.target.width as f32, slot.target.height as f32],
            ),
            None => return Ok(()),
        };
        let (parent_view, pw, ph, parent_vp) = match self.slots.get(&parent_surface) {
            Some(slot) => {
                let vp = match &slot.content {
                    SurfaceContent::Scene3d(scene) => scene.view_proj,
                    _ => Mat4::IDENTITY,
                };
                (
                    slot.target.view().clone(),
                    slot.target.width,
                    slot.target.height,
                    vp,
                )
            }
            None => return Ok(()),
        };
        let gpu = &self.gpu;
        match portal.desc.mount {
            MountKind::Plane3d { center, half } => {
                let model = Mat4::from_translation(glam::Vec3::from_array(center))
                    * Mat4::from_scale(glam::Vec3::new(half[0], half[1], 1.0));
                self.plane
                    .set_transform(gpu.device(), gpu.queue(), &child_view, model, parent_vp);
                let mut encoder =
                    gpu.device()
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("agp plane composite"),
                        });
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("agp plane composite"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &parent_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    self.plane.draw_in_pass(&mut pass);
                }
                gpu.queue().submit(Some(encoder.finish()));
                self.stats.plane_composites += 1;
            }
            MountKind::Flat2d {
                offset,
                scale,
                clip,
            } => {
                let rect = [
                    offset[0],
                    offset[1],
                    child_size[0] * scale[0],
                    child_size[1] * scale[1],
                ];
                let clip_px = if self.scissor_enabled {
                    Some([clip.x, clip.y, clip.w, clip.h])
                } else {
                    None
                };
                let parent_size = [pw as f32, ph as f32];
                let mut encoder =
                    gpu.device()
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("agp quad composite"),
                        });
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("agp quad composite"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &parent_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    self.portal_quad.draw(
                        gpu.device(),
                        gpu.queue(),
                        &mut pass,
                        &child_view,
                        rect,
                        clip_px,
                        parent_size,
                    );
                }
                gpu.queue().submit(Some(encoder.finish()));
                self.stats.quad_composites += 1;
            }
        }
        self.stats.portal_composites += 1;
        Ok(())
    }

    /// Acceptance capture of one surface's texture (explicit readback).
    pub fn capture(&mut self, id: SurfaceId, name: &str) -> Option<(String, Vec<u8>, u32, u32)> {
        let slot = self.slots.get(&id)?;
        let png = crate::renderer::capture::capture_png(&self.gpu, &slot.target)
            .ok()?
            .0;
        self.stats.readbacks += 1;
        let _ = name;
        Some((sha256_hex(&png), png, slot.target.width, slot.target.height))
    }

    /// A pixel probe of one surface's CURRENT texture (acceptance only).
    /// Returns the RGBA of `local` (logical px) — one readback per call.
    pub fn probe_pixel(&mut self, id: SurfaceId, local: [f32; 2]) -> Option<[u8; 4]> {
        let slot = self.slots.get(&id)?;
        let (w, h) = (slot.target.width, slot.target.height);
        let img = crate::renderer::capture::capture_png(&self.gpu, &slot.target)
            .ok()?
            .1;
        self.stats.readbacks += 1;
        let x = (local[0] as u32).min(w.saturating_sub(1));
        let y = (local[1] as u32).min(h.saturating_sub(1));
        let p = img.get_pixel(x, y);
        Some([p[0], p[1], p[2], p[3]])
    }

    /// Read several acceptance points from the same composed frame with one readback.
    pub fn probe_pixels(&mut self, id: SurfaceId, points: &[[f32; 2]]) -> Option<Vec<[u8; 4]>> {
        let slot = self.slots.get(&id)?;
        let (w, h) = (slot.target.width, slot.target.height);
        let img = crate::renderer::capture::capture_png(&self.gpu, &slot.target)
            .ok()?
            .1;
        self.stats.readbacks += 1;
        Some(
            points
                .iter()
                .map(|p| {
                    let x = (p[0] as u32).min(w.saturating_sub(1));
                    let y = (p[1] as u32).min(h.saturating_sub(1));
                    let rgba = img.get_pixel(x, y);
                    [rgba[0], rgba[1], rgba[2], rgba[3]]
                })
                .collect(),
        )
    }

    /// Focus a surface's iced text_input by widget id (public `focus`
    /// operation). Returns false when the surface is not iced content.
    pub fn focus_iced_text_input(
        &mut self,
        id: SurfaceId,
        widget: iced_wgpu::core::widget::Id,
    ) -> bool {
        // VALIDATE THE TARGET FIRST: a cross-session / stale / non-iced /
        // missing Surface is rejected with NO side effect. Releasing the
        // current valid edit state before the lookup destroyed focus on a
        // refused request (review r3 F-830-R3-02).
        let is_iced = matches!(
            self.slots.get(&id).map(|s| &s.content),
            Some(SurfaceContent::Iced(_))
        );
        if !is_iced {
            return false;
        }
        // Only now move the single keyboard target: a public focus call
        // behaves like a routed click (review r2 F02).
        let others: Vec<SurfaceId> = self.slots.keys().copied().filter(|s| *s != id).collect();
        for other in others {
            self.release_iced_focus_if_active(other);
        }
        let Some(mut slot) = self.slots.remove(&id) else {
            return false;
        };
        let done = match &mut slot.content {
            SurfaceContent::Iced(iced) => {
                let ui: &mut dyn crate::host::iced_surface::SurfaceUi<Message = String> =
                    iced.ui.as_mut();
                iced.surface.focus_text_input(ui, widget);
                true
            }
            _ => false,
        };
        self.slots.insert(id, slot);
        // A successful public focus becomes the router's target, so the
        // single-edit-target rule keeps holding on later frames.
        if done {
            self.graph.note_routed_focus(id);
        }
        done
    }

    /// Release the iced keyboard focus of a surface (window blur, capture
    /// cancel, or surface destroy). Returns false for non-iced content, or
    /// when the surface no longer exists. A matching routed target is
    /// forgotten so the next frame does not keep enforcing a dead target.
    pub fn release_iced_focus(&mut self, id: SurfaceId) -> bool {
        // A cancelled/destroyed target becomes Cancelled (not Unset), so any
        // still-pending event batch cannot revive an editor state in the next
        // frame (review r4 F-830-R4-01).
        if self.graph.routed_focus() == Some(id) {
            self.graph.cancel_routed_focus();
        }
        let Some(mut slot) = self.slots.remove(&id) else {
            return false;
        };
        let done = match &mut slot.content {
            SurfaceContent::Iced(iced) => {
                let ui: &mut dyn crate::host::iced_surface::SurfaceUi<Message = String> =
                    iced.ui.as_mut();
                iced.surface.unfocus(ui);
                true
            }
            _ => false,
        };
        self.slots.insert(id, slot);
        done
    }

    /// Is the surface's iced text_input focused? (diagnostics/evidence)
    pub fn iced_text_input_focused(
        &mut self,
        id: SurfaceId,
        widget: iced_wgpu::core::widget::Id,
    ) -> bool {
        let Some(mut slot) = self.slots.remove(&id) else {
            return false;
        };
        let out = match &mut slot.content {
            SurfaceContent::Iced(iced) => {
                let ui: &mut dyn crate::host::iced_surface::SurfaceUi<Message = String> =
                    iced.ui.as_mut();
                iced.surface.is_focused(ui, widget)
            }
            _ => false,
        };
        self.slots.insert(id, slot);
        out
    }

    /// TEST HOOK (V-08 negative control): render one surface's content a
    /// SECOND time inside the current frame window and return the resulting
    /// per-surface count. The count comes from the REAL content-pass counter
    /// (`render_content`), so the gate observes an actual double pass — not a
    /// fabricated increment.
    pub fn force_double_render(&mut self, id: SurfaceId) -> u64 {
        let allowed = match self.graph.routed_focus_state() {
            crate::core::composition::RoutedFocus::Target(target) => target == id,
            crate::core::composition::RoutedFocus::Cancelled => false,
            crate::core::composition::RoutedFocus::Unset => true,
        };
        let _ = self.render_content(id, &[], None, allowed);
        self.stats.last_frame_renders.get(&id).copied().unwrap_or(0)
    }

    /// Convenience: a 2D rect helper for mounts.
    pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::at(x, y, w, h)
    }
}

// Silences the unused-import warning for Shape (2D content authors would use
// any shape; keep the type reachable from this module's surface).
#[allow(unused)]
fn _shapes(_: Shape) {}

/// Cheap, allocation-free fingerprint of a 3D content state: the clear color,
/// the camera and every instance transform. Used only to decide whether a
/// consumer must re-sample the surface this frame (no GPU readback).
fn scene_fingerprint(scene: &Scene3dContent) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    scene.mesh.editor_grid_revision().hash(&mut hasher);
    for c in scene.view_proj.to_cols_array() {
        c.to_bits().hash(&mut hasher);
    }
    for c in scene.sky {
        c.to_bits().hash(&mut hasher);
    }
    if let Some(overlay) = &scene.screen_overlay {
        for cmd in &overlay.commands {
            for v in cmd.center.iter().chain(&cmd.size) {
                v.to_bits().hash(&mut hasher);
            }
            cmd.color.a.to_bits().hash(&mut hasher);
        }
    }
    if let Some(text) = &scene.text_overlay {
        for glyph in &text.glyphs {
            for v in glyph.dst.iter().chain(&glyph.uv_rect).chain(&glyph.color) {
                v.to_bits().hash(&mut hasher);
            }
        }
    }
    scene.instances.len().hash(&mut hasher);
    for inst in &scene.instances {
        inst.mesh.0.hash(&mut hasher);
        inst.mesh.1.hash(&mut hasher);
        for c in inst.model.to_cols_array() {
            c.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}
