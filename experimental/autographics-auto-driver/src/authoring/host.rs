//! Stock Iced authoring UI and same-device transactional presentation.
mod commands;
mod navigation;
mod navigation3d;
mod panel;
mod play_adapter;
mod viewport;
use super::{
    document::{Dimension, Document, Group, Kind, Object},
    session::{Edit, Editor, Stamp},
    storage::{Fault, Saved, Storage},
};
use agp::{
    assets::{load_glb_bytes, ModelData},
    core::{
        composition::{input::InputOutcome, MountKind, Rect, SurfaceId},
        scene::Color,
    },
    host::{
        composition_input::CompositionInput,
        iced_surface::{SurfaceElement, SurfaceUi},
        surface_runtime::{
            Game2dContent, IcedContent, Scene3dContent, SurfaceContent, SurfaceRuntime,
        },
    },
    render3d::{MeshInstance, MeshPipeline, MeshStore},
    renderer::{
        pipeline::{DrawCmd, Shape},
        target::{OffscreenTarget, WindowTarget},
        GpuDevice,
    },
};
use glam::{Mat4, Vec3};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Instant,
};
#[derive(Clone)]
enum Leave {
    Open(PathBuf),
    New,
    Close,
}
pub struct Panel {
    pub view2d: super::view2d::View2d,
    pub view3d: super::view3d::View3d,
    scroll_offset: std::sync::Arc<std::sync::Mutex<f32>>,
    pub play_state: Option<String>,
    pub binding_busy: bool,
    pub behavior_user: bool,
    pub project_default: bool,
    pub behavior_api: super::behavior::Api,
    pub behavior_path: String,
    pub behavior_resolved: String,
    pub last_behavior: String,
    pub play_tick: i32,
    pub play_token: u64,
    pub actors: [Option<u64>; 2],
    original_draft: Vec<String>,
    pub asset_token: u64,
    pub asset_busy: bool,
    pub import_path: String,
    pub document: Document,
    pub stamp: Stamp,
    pub selection: Option<u64>,
    pub draft: Vec<String>,
    pub path: String,
    pub error: String,
    pub busy: bool,
    pub dirty: bool,
    pub dialog: bool,
    pub queue: Vec<String>,
    pub derived: std::collections::BTreeMap<u64, super::hierarchy::Derived>,
    pub parent_choice: Option<u64>,
    pub delete_confirmation: Option<u64>,
    pub group_handle_reason: Option<String>,
    bounds: super::tagged::Bounds,
}
const FIELDS: [&str; 12] = [
    "x", "y", "z", "scale x", "scale y", "scale z", "yaw", "pitch", "red", "green", "blue", "alpha",
];
fn parse_parent(v: &Value) -> Result<Option<u64>, String> {
    if v.is_null() {
        Ok(None)
    } else {
        v.as_u64().map(Some).ok_or("parent identity".into())
    }
}
struct Prepared {
    store: MeshStore,
    draws: Vec<DrawCmd>,
    instances: Vec<MeshInstance>,
    highlight: Vec<DrawCmd>,
    /// Runtime-camera projection published with the same transaction; None
    /// keeps the installed editor/sample camera.
    camera_vp: Option<Mat4>,
}
fn bounded_time(v: &mut Vec<u128>, us: u128) {
    v.push(us);
    if v.len() > 4096 {
        v.remove(0);
    }
}
struct Navigation {
    start: super::view2d::View2d,
    origin: [f32; 2],
    draws: Vec<DrawCmd>,
}
struct Navigation3d {
    start: super::view3d::View3d,
    origin: [f32; 2],
    orbit: bool,
    overlay: Vec<DrawCmd>,
}
pub struct Host {
    view2d: super::view2d::View2d,
    presented_view2d: super::view2d::View2d,
    view3d: super::view3d::View3d,
    presented_view3d: super::view3d::View3d,
    navigation: Option<Navigation>,
    navigation3d: Option<Navigation3d>,
    pub prepare_measurement: super::measurement::Measurement,
    pub project_default: bool,
    project_pending: bool,
    pub play: Option<super::play::Play>,
    pub play_start_us: Vec<u128>,
    pub play_stop_us: Vec<u128>,
    pub play_pump_us: Vec<u128>,
    play_restore: Option<Prepared>,
    play_token: u64,
    actors: [Option<u64>; 2],
    pub play_source_for_probe: Option<String>,
    asset_token: u64,
    asset_pending: bool,
    pub editor: Editor,
    gesture: Option<super::interaction::Gesture>,
    gesture_token: u64,
    gesture_source: Option<String>,
    pub gesture_compile_us: Vec<u128>,
    pub gesture_pump_us: Vec<u128>,
    presented: Vec<Object>,
    presented_nodes: std::collections::BTreeMap<u64, super::hierarchy::Derived>,
    presented_stamp: Stamp,
    presented_camera: Mat4,
    pub storage: Storage,
    pub rt: SurfaceRuntime,
    pub input: CompositionInput,
    pub panel: SurfaceId,
    pub two: SurfaceId,
    pub three: SurfaceId,
    pub top: SurfaceId,
    models: Vec<ModelData>,
    events: HashMap<SurfaceId, Vec<iced_wgpu::core::Event>>,
    cursors: HashMap<SurfaceId, [f32; 2]>,
    leave: Option<Leave>,
    opening: Option<Storage>,
    pub closed: bool,
    pub fail_resource: bool,
    pub fail_after_asset_upload: bool,
    pub asset_source_for_probe: Option<String>,
    pub asset_render_fault: Option<String>,
    pub frame_us: Vec<u128>,
    pub accepted: u64,
    /// Active runtime-camera LMB drag capture (3D camera Runs only).
    pub camera_drag: Option<CameraDrag>,
    /// Camera presentation facts for probes/tests.
    pub camera_saturation: u32,
}

/// One active orbit drag: the last delivered logical position on the 3D game
/// surface. Cleared on Up/blur/leave/Pause/Stop/Fault/resize/DPI/modal; late
/// move/up events without a capture are inert (contract section 4).
pub struct CameraDrag {
    pub last: [f32; 2],
}
impl Host {
    pub fn new(doc: Document) -> Result<Self, String> {
        Self::with_assets(doc, Default::default())
    }
    pub fn with_assets(doc: Document, assets: super::assets::Registry) -> Result<Self, String> {
        Self::with_project(doc, assets, None)
    }
    pub fn with_project(
        doc: Document,
        assets: super::assets::Registry,
        code: Option<super::behavior_project::Code>,
    ) -> Result<Self, String> {
        let editor = Editor::with_project(doc, assets, code)?;
        let gpu = GpuDevice::new().map_err(|e| format!("{e:?}"))?;
        let mut rt = SurfaceRuntime::new(gpu).map_err(|e| format!("{e:?}"))?;
        let top = rt
            .add_surface(
                [1200.0, 800.0],
                SurfaceContent::Game2d(Box::new(Game2dContent::simple(
                    rt.gpu(),
                    Color::rgb(0.02, 0.03, 0.04),
                ))),
            )
            .map_err(|e| format!("{e:?}"))?;
        let view = rt.add_view(top, true).map_err(|e| format!("{e:?}"))?;
        let two = rt
            .add_surface(
                [800.0, 380.0],
                SurfaceContent::Game2d(Box::new(Game2dContent::simple(
                    rt.gpu(),
                    Color::rgb(0.02, 0.03, 0.04),
                ))),
            )
            .map_err(|e| format!("{e:?}"))?;
        let three = rt
            .add_surface(
                [800.0, 400.0],
                SurfaceContent::Scene3d(Box::new(Scene3dContent {
                    store: MeshStore::default(),
                    screen_overlay: Some(
                        agp::host::surface_runtime::ScreenOverlay::new(rt.gpu())
                            .map_err(|e| format!("{e:?}"))?,
                    ),
                    text_overlay: None,
                    mesh: MeshPipeline::new(rt.gpu(), OffscreenTarget::FORMAT)
                        .map_err(|e| format!("{e:?}"))?,
                    instances: vec![],
                    sky: [0.1, 0.2, 0.3, 1.0],
                    view_proj: agp::render3d::Camera3d::sample_camera(2.0).view_proj(),
                })),
            )
            .map_err(|e| format!("{e:?}"))?;
        let actors = [true, false].map(|two| {
            editor
                .document
                .objects
                .iter()
                .find(|o| o.kind.is_2d() == two)
                .map(|o| o.id)
        });
        let mut ui = Panel {
            view2d: Default::default(),
            view3d: Default::default(),
            scroll_offset: Default::default(),
            play_state: None,
            binding_busy: false,
            behavior_user: false,
            project_default: true,
            behavior_api: super::behavior::Api::V1,
            behavior_path: String::new(),
            behavior_resolved: String::new(),
            last_behavior: String::new(),
            play_tick: 0,
            play_token: 0,
            actors,
            original_draft: vec![],
            asset_token: 0,
            asset_busy: false,
            import_path: String::new(),
            document: editor.document.clone(),
            stamp: editor.stamp,
            selection: None,
            draft: vec![],
            path: "scene.agscene".into(),
            error: String::new(),
            busy: false,
            dirty: false,
            dialog: false,
            queue: vec![],
            derived: editor.preview.nodes.clone(),
            parent_choice: None,
            delete_confirmation: None,
            group_handle_reason: None,
            bounds: Default::default(),
        };
        ui.sync(&editor);
        let panel = rt
            .add_surface(
                [390.0, 800.0],
                SurfaceContent::Iced(Box::new(IcedContent::borrowed(
                    rt.gpu(),
                    [390.0, 800.0],
                    Box::new(ui),
                ))),
            )
            .map_err(|e| format!("{e:?}"))?;
        for (id, xy, size) in [
            (two, [0.0, 0.0], [800.0, 380.0]),
            (three, [0.0, 400.0], [800.0, 400.0]),
            (panel, [810.0, 0.0], [390.0, 800.0]),
        ] {
            rt.add_portal(
                view,
                id,
                MountKind::Flat2d {
                    offset: xy,
                    scale: [1.0, 1.0],
                    clip: Rect::at(xy[0], xy[1], size[0], size[1]),
                },
                true,
            )
            .map_err(|e| format!("{e:?}"))?;
        }
        let mut models = vec![];
        let mesh = agp::render3d::cylinder_mesh();
        models.push(ModelData {
            source_sha256: "authoring-fixed-cylinder".into(),
            meshes: vec![mesh],
        });
        for bytes in [
            include_bytes!(
                "../../../autographics-prototype/assets/squash_the_creeps/art/player.glb"
            )
            .as_slice(),
            include_bytes!("../../../autographics-prototype/assets/squash_the_creeps/art/mob.glb")
                .as_slice(),
        ] {
            models.push(load_glb_bytes(bytes).map_err(|e| format!("{e:?}"))?);
        }
        let mut h = Self {
            view2d: Default::default(),
            presented_view2d: Default::default(),
            view3d: Default::default(),
            presented_view3d: Default::default(),
            navigation: None,
            navigation3d: None,
            prepare_measurement: Default::default(),
            play_start_us: vec![],
            play_stop_us: vec![],
            play_pump_us: vec![],
            play: None,
            play_restore: None,
            play_token: 0,
            actors,
            play_source_for_probe: None,
            asset_token: 0,
            asset_pending: false,
            presented_nodes: editor.preview.nodes.clone(),
            presented: editor.preview.objects.clone(),
            presented_stamp: editor.stamp,
            presented_camera: agp::render3d::Camera3d::sample_camera(2.0).view_proj(),
            gesture: None,
            gesture_token: 0,
            gesture_source: None,
            gesture_compile_us: vec![],
            gesture_pump_us: vec![],
            project_default: true,
            project_pending: false,
            editor,
            storage: Storage::default(),
            rt,
            input: CompositionInput::new([1200.0, 800.0], [1200.0, 800.0]),
            panel,
            two,
            three,
            top,
            models,
            events: HashMap::new(),
            cursors: HashMap::new(),
            leave: None,
            opening: None,
            closed: false,
            fail_resource: false,
            fail_after_asset_upload: false,
            asset_source_for_probe: None,
            asset_render_fault: None,
            frame_us: vec![],
            accepted: 0,
            camera_drag: None,
            camera_saturation: 0,
        };
        h.input.mapper.register(h.three, h.camera(), [800.0, 400.0]);
        // Fixed-resource warm upload: one resident store on the renderer device.
        let mut warm = MeshStore::default();
        for model in &h.models {
            for i in 0..model.meshes.len() {
                warm.get_or_upload(h.rt.gpu(), model, i)
                    .map_err(|e| format!("{e:?}"))?;
            }
        }
        if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(h.three) {
            c.store = warm;
        }
        let p = h.prepare(&h.editor.preview)?;
        h.publish(p);
        Ok(h)
    }
    /// Traverse the actual runtime widget tree, retaining only current layout witnesses.
    pub fn widget_bounds(&mut self, id: &str) -> Option<iced_wgpu::core::Rectangle> {
        self.ui().bounds.lock().unwrap().clear();
        let _ = self.rt.iced_widget_bounds(
            self.panel,
            iced_wgpu::core::widget::Id::new("layout-witness-traversal"),
        );
        let b = self.ui().bounds.lock().unwrap().get(id).copied();
        let mut b = b.or_else(|| {
            self.rt
                .iced_widget_bounds(self.panel, iced_wgpu::core::widget::Id::from(id.to_owned()))
        })?;
        b.y -= *self.ui().scroll_offset.lock().unwrap();
        Some(b)
    }
    pub fn ui(&self) -> &Panel {
        match self.rt.content(self.panel).unwrap() {
            SurfaceContent::Iced(c) => c.ui.as_any().downcast_ref().unwrap(),
            _ => unreachable!(),
        }
    }
    pub fn ui_mut(&mut self) -> &mut Panel {
        match self.rt.content_mut(self.panel).unwrap() {
            SurfaceContent::Iced(c) => c.ui.as_any_mut().downcast_mut().unwrap(),
            _ => unreachable!(),
        }
    }
    fn sync_ui(&mut self) {
        self.prune_resources();
        let doc = self.editor.document.clone();
        let stamp = self.editor.stamp;
        let selection = self.editor.selection;
        let busy = self.editor.pending() || self.gesture.is_some();
        let dirty = self.editor.dirty();
        let dialog = self.leave.is_some();
        let derived = self.editor.preview.nodes.clone();
        let group_handle_reason = selection
            .filter(|id| doc.groups.iter().any(|g| g.id == *id))
            .and_then(|id| {
                let members = super::hierarchy::descendants(&doc, id).ok()?;
                if !self.editor.preview.objects.iter().any(|o| {
                    members.contains(&o.id) && o.visible && (!o.kind.is_2d() || o.rgba[3] > 0.0)
                }) {
                    Some("No visible descendants: edit local properties in the tree.".into())
                } else if self
                    .group_geometry(&doc, &self.editor.preview.objects, &derived, id)
                    .is_none()
                {
                    Some(
                        "Group or its visible descendants are clipped or degenerate: edit local properties in the tree."
                            .into(),
                    )
                } else {
                    None
                }
            });
        let asset_busy = self.asset_pending;
        let asset_token = self.asset_token;
        let view2d = self.view2d;
        let view3d = self.view3d;
        let ui = self.ui_mut();
        ui.view2d = view2d;
        ui.view3d = view3d;
        ui.asset_token = asset_token;
        ui.asset_busy = asset_busy;
        ui.group_handle_reason = group_handle_reason;
        ui.derived = derived;
        ui.document = doc;
        ui.stamp = stamp;
        ui.selection = selection;
        ui.busy = busy;
        ui.dirty = dirty;
        ui.dialog = dialog;
        ui.reset_draft();
        self.sync_play_ui();
        self.rt.graph.mark_content_dirty(self.panel);
    }
    fn active_assets(&self) -> Result<super::assets::Registry, String> {
        let mut assets = self.editor.assets.clone();
        for r in
            std::iter::once(self.editor.candidate_assets()).chain(self.storage.retained_assets())
        {
            for a in r.values() {
                super::assets::insert(&mut assets, a.clone())?;
            }
        }
        super::assets::check_registry(&assets)?;
        Ok(assets)
    }
    fn prune_resources(&mut self) {
        let keys: std::collections::BTreeSet<_> = self
            .models
            .iter()
            .map(|m| m.source_sha256.clone())
            .chain(
                std::iter::once(&self.editor.assets)
                    .chain(std::iter::once(self.editor.candidate_assets()))
                    .chain(self.storage.retained_assets())
                    .flat_map(|r| r.values().map(|a| a.identity.sha256.clone())),
            )
            .collect();
        if let Some(SurfaceContent::Scene3d(c)) = self.rt.content_mut(self.three) {
            c.store.retain_models(|sha| keys.contains(sha));
        }
    }
    fn model(&self, o: &Object) -> Option<&ModelData> {
        if o.kind == Kind::Model {
            self.editor
                .candidate_assets()
                .get(o.resource.as_ref()?)
                .or_else(|| self.editor.assets.get(o.resource.as_ref()?))
                .or_else(|| self.storage.retained_assets()?.get(o.resource.as_ref()?))
                .map(|a| &a.model)
        } else {
            self.models.get(match o.kind {
                Kind::Cylinder => 0,
                Kind::Player => 1,
                Kind::Mob => 2,
                _ => return None,
            })
        }
    }
}

impl Host {
    pub fn resource_snapshot(&self) -> serde_json::Value {
        let scene = match self.rt.content(self.three).unwrap() {
            SurfaceContent::Scene3d(c) => c,
            _ => unreachable!(),
        };
        let two = match self.rt.content(self.two).unwrap() {
            SurfaceContent::Game2d(c) => c,
            _ => unreachable!(),
        };
        let uploads = self.rt.upload_stats();
        let asset_union = match self.active_assets() {
            Ok(assets) => {
                json!({"count":assets.len(),"raw_bytes":assets.values().map(|a|a.bytes.len()).sum::<usize>(),"cpu_geometry_bytes":assets.values().map(|a|a.cpu_bytes).sum::<usize>(),"gpu_expected_bytes":assets.values().map(|a|a.gpu_bytes).sum::<usize>(),"decode_attempts":super::assets::decode_attempts(),"decode_us":assets.values().map(|a|a.decode_us).collect::<Vec<_>>()})
            }
            Err(error) => json!({"error":error}),
        };

        json!({"view3d":{"revision":self.view3d.revision,"presented_revision":self.presented_view3d.revision,"target":self.view3d.target.to_array(),"yaw":self.view3d.yaw,"pitch":self.view3d.pitch,"distance":self.view3d.distance,"projection":format!("{:?}",self.view3d.mode),"grid":self.view3d.grid,"grid_vertices":self.view3d.grid_vertices().map(|(_,v)|v.len()).unwrap_or(0),"grid_gpu_bytes":scene.mesh.editor_grid_owned_bytes(),"grid_cpu_capacity_bytes":scene.mesh.editor_grid_cpu_capacity_bytes(),"grid_uploads":scene.mesh.editor_grid_uploads()},"view2d":{"revision":self.view2d.revision,"presented_revision":self.presented_view2d.revision,"center":self.view2d.center.to_array(),"zoom":self.view2d.zoom,"grid_spacing_world":self.active_view2d().grid_lines().0,"grid_rectangles":self.active_view2d().grid_lines().1.len(),"grid_cpu_capacity_bytes":if self.active_view2d().grid {128 * std::mem::size_of::<super::view2d::GridLine>()}else{0},"grid_payload_bytes_per_draw":self.active_view2d().grid_lines().1.len()*48,"two_quad_uploads":two.quad.uploads.snapshot()},"play_metrics":{"run_command_us":self.play_start_us,"stop_command_us":self.play_stop_us,"pump_round_us":self.play_pump_us},"target_bytes":self.rt.owned_target_bytes(),"target_count":self.rt.live_target_slots(),"mesh_bytes":scene.store.owned_buffer_bytes(),"mesh_count":scene.store.uploaded_count(),"mesh_uploads":scene.store.uploads.snapshot(),"scene_pipeline_buffers":scene.mesh.owned_buffer_bytes(),"depth_texture_bytes":scene.mesh.owned_depth_texture_bytes(),"two_buffers":two.quad.owned_buffer_bytes(),"buffer_rewrites":uploads.buffer_rewrites,"bytes_rewritten":uploads.bytes_rewritten,"asset_union":asset_union,"exclusions":"Transient JSON/parser allocations, Iced internal buffers/atlas and driver allocations are not resident geometry budgets; GPU timestamps unavailable"})
    }
}

impl Host {
    /// Explicit test/probe hook: output still passes ordinary profile/config admission.
    pub fn gesture_source_for_probe(&mut self, source: Option<String>) {
        self.gesture_source = source;
    }
    pub fn interaction_snapshot(&self) -> Value {
        let g = self.gesture.as_ref();
        json!({"presented":self.presented,"accepted":self.editor.document,"revision":self.editor.stamp.revision,
            "gesture_token":g.map(|g|g.token),"latest":g.and_then(|g|g.latest.map(|x|x.0)),
            "shown_sample":g.map(|g|g.shown_sample),"inflight":g.and_then(|g|g.inflight.as_ref().map(|x|x.0)),
            "released":g.is_some_and(|g|g.released),"compile_us":self.gesture_compile_us,"pump_us":self.gesture_pump_us})
    }
    /// Actual VM weak-owner oracles; tests retain these across publish/cancel/drop.
    pub fn gesture_owner_probes(&self) -> Vec<Box<dyn Fn() -> bool>> {
        let mut owners: Vec<Box<dyn Fn() -> bool>> =
            vec![Box::new(self.editor.preview.vm.owner_probe())];
        if let Some(g) = &self.gesture {
            if let Some(p) = &g.shown {
                owners.push(Box::new(p.vm.owner_probe()));
            }
            if let Some((_, _, b)) = &g.inflight {
                owners.push(b.owner_probe());
            }
        }
        owners
    }
}
