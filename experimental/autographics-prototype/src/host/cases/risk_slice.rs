//! Case `ag3d-risk-slice` (T-05): proves the hard parts of AG-3D-01 on real
//! assets: GLB load (gltf 1.4), lit mesh rendering with directional shadow
//! mapping, the frozen inline bob animation, the sample's orthographic
//! camera + environment, and the collision queries the sample semantics need.
//! No game logic here (that is T-06); no cubes standing in for the sample.

use crate::assets::{load_glb_file, MeshData, ModelData};
use crate::host::report::{
    write_report, AssetFact, AssetMeshFact, CollisionQueryResult, Keyframe, RiskSliceReport,
};
use crate::render3d::collision::*;
use crate::render3d::procedural::{cylinder_mesh, ground_mesh};
use crate::render3d::{
    sample_light, BobAnimation, Camera3d, MeshInstance, MeshPipeline, MeshStore, Render3dStats,
    CYLINDER_POSITIONS,
};
use crate::renderer::capture::{capture_png, sha256_hex};
use crate::renderer::error::RendererError;
use crate::renderer::target::{OffscreenTarget, WindowTarget};
use crate::renderer::GpuDevice;
use glam::{Mat4, Vec3};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const W: u32 = 900;
const H: u32 = 600;
/// Main.tscn WorldEnvironment horizon color (lines 17–19).
const SKY_HORIZON: [f32; 4] = [0.64625, 0.65575, 0.67075, 1.0];

fn agp_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Slice constants: mob spawn points on the frozen spawn rectangle
/// (Main.tscn Curve3D, lines 37–42) with fixed yaws/speeds standing in for
/// T-06's seeded RNG (documented in 3d-feasibility.md).
const MOB_SPAWNS: [(Vec3, f32, f32); 4] = [
    (Vec3::new(14.0, 0.0, -15.0), 0.3, 12.0),
    (Vec3::new(-13.0, 0.0, -15.0), -0.6, 14.5),
    (Vec3::new(-13.0, 0.0, 16.0), 1.1, 16.0),
    (Vec3::new(14.0, 0.0, 16.0), -1.3, 11.0),
];

const KEYFRAME_PLAN: [(u64, &str, bool); 4] = [
    (0, "anim_t0_shadow_on", true),
    (45, "anim_t45_shadow_on", true),
    (45, "anim_t45_shadow_off", false),
    (90, "anim_t90_shadow_on", true),
];

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    if replay != "canonical" {
        eprintln!("ag3d-risk-slice: unknown replay '{replay}' (canonical)");
        return 2;
    }
    if mode == "headless" {
        return run_headless(report_dir);
    }
    run_windowed(report_dir)
}

// ---------------------------------------------------------------------------
// Scene construction (shared)
// ---------------------------------------------------------------------------

struct SceneData {
    player: ModelData,
    mob: ModelData,
    player_keys: Vec<(String, String)>,
    mob_keys: Vec<(String, String)>,
    ground_key: (String, String),
    cylinder_key: (String, String),
    camera: Camera3d,
}

/// Wrap a procedural mesh in a ModelData under a content-derived identity.
fn synth_model(mesh: &MeshData) -> ModelData {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    for p in &mesh.positions {
        h.update(p[0].to_ne_bytes());
        h.update(p[1].to_ne_bytes());
        h.update(p[2].to_ne_bytes());
    }
    h.update(mesh.name.as_bytes());
    ModelData {
        source_sha256: format!("{:x}", h.finalize()),
        meshes: vec![mesh.clone()],
    }
}

fn build_scene(gpu: &GpuDevice, store: &mut MeshStore) -> Result<SceneData, RendererError> {
    let player = load_glb_file(&agp_file("assets/squash_the_creeps/art/player.glb"))
        .map_err(|e| RendererError::ReportIo(format!("player.glb: {e}")))?;
    let mob = load_glb_file(&agp_file("assets/squash_the_creeps/art/mob.glb"))
        .map_err(|e| RendererError::ReportIo(format!("mob.glb: {e}")))?;

    let mut player_keys = Vec::new();
    for i in 0..player.meshes.len() {
        let (name, _count) = store.get_or_upload(gpu, &player, i)?;
        player_keys.push((player.source_sha256.clone(), name));
    }
    let mut mob_keys = Vec::new();
    for i in 0..mob.meshes.len() {
        let (name, _count) = store.get_or_upload(gpu, &mob, i)?;
        mob_keys.push((mob.source_sha256.clone(), name));
    }
    let ground = synth_model(&ground_mesh());
    let ground_key = (ground.source_sha256.clone(), ground.meshes[0].name.clone());
    store.get_or_upload(gpu, &ground, 0)?;
    let cylinder = synth_model(&cylinder_mesh());
    let cylinder_key = (
        cylinder.source_sha256.clone(),
        cylinder.meshes[0].name.clone(),
    );
    store.get_or_upload(gpu, &cylinder, 0)?;

    Ok(SceneData {
        player,
        mob,
        player_keys,
        mob_keys,
        ground_key,
        cylinder_key,
        camera: Camera3d::sample_camera(W as f32 / H as f32),
    })
}

/// Instance transforms for one visual tick (60 Hz, deterministic).
fn frame_instances(scene: &SceneData, tick: u64) -> Vec<MeshInstance> {
    let bob = BobAnimation::default();
    let dt = 1.0f32 / 60.0;
    let mut out = Vec::new();
    out.push(MeshInstance {
        mesh: scene.ground_key.clone(),
        model: Mat4::IDENTITY,
    });
    for pos in CYLINDER_POSITIONS {
        out.push(MeshInstance {
            mesh: scene.cylinder_key.clone(),
            model: Mat4::from_translation(Vec3::new(pos[0], pos[1], pos[2])),
        });
    }
    {
        // Player at the arena center, moving ⇒ speed_scale 4 (Player.gd:31).
        let (y, rot_x) = bob.eval(bob.time_at(tick, dt, 4.0));
        let base = Mat4::from_translation(Vec3::new(0.0, y, 0.0)) * Mat4::from_rotation_x(rot_x);
        for key in &scene.player_keys {
            out.push(MeshInstance {
                mesh: key.clone(),
                model: base,
            });
        }
    }
    for (pos, yaw, speed) in MOB_SPAWNS {
        let (y, rot_x) = bob.eval(bob.time_at(tick, dt, speed / 10.0));
        let base = Mat4::from_translation(pos)
            * Mat4::from_rotation_y(yaw)
            * Mat4::from_translation(Vec3::new(0.0, y, 0.0))
            * Mat4::from_rotation_x(rot_x);
        for key in &scene.mob_keys {
            out.push(MeshInstance {
                mesh: key.clone(),
                model: base,
            });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// GPU kit
// ---------------------------------------------------------------------------

struct WindowKit {
    window: Arc<winit::window::Window>,
    target: WindowTarget,
    pipeline: MeshPipeline,
    width: u32,
    height: u32,
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

fn asset_fact(file: &str, model: &ModelData) -> AssetFact {
    AssetFact {
        file: file.to_string(),
        sha256: model.source_sha256.clone(),
        meshes: model
            .meshes
            .iter()
            .map(|m| AssetMeshFact {
                name: m.name.clone(),
                vertices: m.positions.len(),
                indices: m.indices.len(),
                material: m.material.name.clone(),
                base_color: m.material.base_color,
            })
            .collect(),
    }
}

/// Pure collision evidence (the frozen squash scheme; render3d::collision).
fn collision_evidence() -> Vec<CollisionQueryResult> {
    let mob_center = Vec3::new(0.0, MOB_BOX_OFFSET_Y, 0.0);
    let mut out = Vec::new();
    let top = sphere_vs_obb(
        Vec3::new(0.0, MOB_BOX_OFFSET_Y + MOB_BOX_HALF[1] + 0.4, 0.0),
        PLAYER_SPHERE_RADIUS,
        mob_center,
        MOB_BOX_HALF,
        0.0,
    );
    out.push(CollisionQueryResult {
        name: "top_landing_classifies_as_squash".into(),
        outcome: match top {
            Some((n, _)) => format!("{:?}", classify_player_mob_contact(n)),
            None => "NO CONTACT (bug)".into(),
        },
    });
    let side = sphere_vs_obb(
        Vec3::new(MOB_BOX_HALF[0] + 0.6, MOB_BOX_OFFSET_Y, 0.0),
        PLAYER_SPHERE_RADIUS,
        mob_center,
        MOB_BOX_HALF,
        0.0,
    );
    out.push(CollisionQueryResult {
        name: "side_contact_classifies_as_fatal".into(),
        outcome: match side {
            Some((n, _)) => format!("{:?}", classify_player_mob_contact(n)),
            None => "NO CONTACT (bug)".into(),
        },
    });
    let (mut c, mut v, _) = step_player(
        Vec3::new(0.0, PLAYER_SPHERE_OFFSET_Y, 0.0),
        Vec3::ZERO,
        false,
    );
    for _ in 0..240 {
        let r = step_player(c, v, false);
        c = r.0;
        v = r.1;
    }
    out.push(CollisionQueryResult {
        name: "player_settles_on_ground".into(),
        outcome: format!("rest_y={:.6}", c.y),
    });
    let clamped = clamp_to_arena(Vec3::new(100.0, 1.0, 100.0), PLAYER_SPHERE_RADIUS);
    out.push(CollisionQueryResult {
        name: "arena_clamp".into(),
        outcome: format!("({:.3}, {:.3})", clamped.x, clamped.z),
    });
    out
}

fn build_report_with_scene_gpu(
    gpu: &GpuDevice,
    store: &MeshStore,
    scene: &SceneData,
    keyframes: &[Keyframe],
    stats: &Render3dStats,
) -> (RiskSliceReport, bool) {
    let mut uniq: Vec<&str> = keyframes.iter().map(|k| k.sha256.as_str()).collect();
    uniq.sort_unstable();
    uniq.dedup();
    let shadow_on = keyframes.iter().find(|k| k.name == "anim_t45_shadow_on");
    let shadow_off = keyframes.iter().find(|k| k.name == "anim_t45_shadow_off");
    let shadow_distinct =
        matches!((shadow_on, shadow_off), (Some(a), Some(b)) if a.sha256 != b.sha256);
    let pass = keyframes.len() == 4
        && uniq.len() == 4
        && shadow_distinct
        && scene.player.meshes.len() == 3
        && scene.mob.meshes.len() == 3
        && store.uploaded_count() >= 8;
    let animation = serde_json::json!({
        "source": "Player.tscn lines 17-43 / Mob.tscn lines 12-38 (identical tracks)",
        "length_s": 1.2,
        "loop": true,
        "position_keys_y": [[0.22, 0.329753], [0.7, 0.660351], [1.18, 0.349734]],
        "rotation_keys_x_deg": [[0.1, -5.0326], [0.46, 10.0], [1.18, -10.0]],
        "visual_dt_s": 1.0 / 60.0,
        "player_speed_scale": 4.0,
        "mob_speed_scales": MOB_SPAWNS.iter().map(|(_, _, sp)| sp / 10.0).collect::<Vec<_>>(),
    });
    let report = RiskSliceReport {
        adapter: gpu.adapter_info().clone(),
        device_token: gpu.token().to_report(),
        window_created: true,
        window_mode: "window".into(),
        assets: vec![
            asset_fact("player.glb", &scene.player),
            asset_fact("mob.glb", &scene.mob),
        ],
        uploaded_meshes: store.uploaded_count(),
        draw_calls_per_frame: stats.draw_calls,
        instances_per_frame: stats.instances,
        camera: "orthographic size=24 far=40 (Main.tscn lines 101-106), eye (0.183514, 13.435, 13.435)".into(),
        environment_clear: SKY_HORIZON,
        animation,
        keyframes: keyframes.to_vec(),
        shadow_on_off_hashes_distinct: shadow_distinct,
        collision_queries: collision_evidence(),
        gpu_timing_available: false,
        notes: vec![
            "mobs spawn at LOCAL y=0 (Main.gd:37 `.position`; SpawnPath transform y=1 never reaches the mob — F829-R6-01); mobs render grounded here".into(),
            "WorldBoundaryShape3D one-sided semantics approximated by inward planes at frozen positions (Main.tscn lines 59-82)".into(),
            "mob.glb node carries rotation quat (-0.5,-0.5,0.5,0.5) + scale (0.953,1,1) — baked into vertices by the loader".into(),
        ],
    };
    (report, pass)
}

fn write(report_dir: &Path, report: RiskSliceReport, pass: bool) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    let exit = if pass { "pass" } else { "fail" };
    match write_report(report_dir, "ag3d-risk-slice", "canonical", exit, report) {
        Ok(()) => {
            if pass {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("ag3d-risk-slice: report: {e}");
            3
        }
    }
}

fn render_keyframe(
    gpu: &GpuDevice,
    store: &MeshStore,
    offscreen: &OffscreenTarget,
    off_pipeline: &mut MeshPipeline,
    scene: &SceneData,
    tick: u64,
    shadows: bool,
) -> Result<Vec<u8>, RendererError> {
    let instances = frame_instances(scene, tick);
    off_pipeline.render(
        gpu,
        offscreen.view(),
        W,
        H,
        SKY_HORIZON,
        &scene.camera.view_proj(),
        &sample_light(),
        store,
        &instances,
        shadows,
    );
    gpu.wait_idle();
    let (png, _) = capture_png(gpu, offscreen)?;
    Ok(png)
}

// ---------------------------------------------------------------------------
// Headless path
// ---------------------------------------------------------------------------

fn run_headless(report_dir: &Path) -> i32 {
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ag3d-risk-slice(headless): device: {e}");
            return 3;
        }
    };
    let offscreen = match OffscreenTarget::new(&gpu, W, H) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ag3d-risk-slice(headless): offscreen: {e}");
            return 3;
        }
    };
    let off_pipeline = match MeshPipeline::new(&gpu, OffscreenTarget::FORMAT) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ag3d-risk-slice(headless): pipeline: {e}");
            return 3;
        }
    };
    let mut store = MeshStore::default();
    let scene = match build_scene(&gpu, &mut store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ag3d-risk-slice(headless): scene: {e}");
            return 3;
        }
    };
    let mut keyframes = Vec::new();
    let mut off_pipeline = off_pipeline;
    for (tick, name, shadows) in KEYFRAME_PLAN {
        match render_keyframe(
            &gpu,
            &store,
            &offscreen,
            &mut off_pipeline,
            &scene,
            tick,
            shadows,
        ) {
            Ok(png) => {
                let file = format!("{name}.png");
                let path = report_dir.join(&file);
                if let Err(e) = std::fs::write(&path, &png) {
                    eprintln!("ag3d-risk-slice(headless): keyframe write: {e}");
                    return 3;
                }
                keyframes.push(Keyframe {
                    name: name.to_string(),
                    file,
                    width: W,
                    height: H,
                    sha256: sha256_hex(&png),
                });
            }
            Err(e) => {
                eprintln!("ag3d-risk-slice(headless): capture: {e}");
                return 3;
            }
        }
    }
    let stats = Render3dStats {
        draw_calls: (scene.player_keys.len()
            + scene.mob_keys.len() * MOB_SPAWNS.len()
            + 1
            + CYLINDER_POSITIONS.len()) as u32,
        instances: frame_instances(&scene, 45).len() as u32,
        shadow_enabled: true,
    };
    let (mut report, pass) = build_report_with_scene_gpu(&gpu, &store, &scene, &keyframes, &stats);
    report.window_created = false;
    report.window_mode = "headless".into();
    write(report_dir, report, pass)
}

// ---------------------------------------------------------------------------
// Windowed path
// ---------------------------------------------------------------------------

fn run_windowed(report_dir: &Path) -> i32 {
    use winit::event_loop::EventLoop;
    let event_loop = match EventLoop::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ag3d-risk-slice: event loop: {e}; falling back to headless");
            return run_headless(report_dir);
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = RiskApp {
        report_dir: report_dir.to_path_buf(),
        gpu: None,
        scene: None,
        window_kit: None,
        offline: None,
        keyframes: Vec::new(),
        tick: 0,
        pending_plan: 0,
        exit_code: 0,
    };
    match event_loop.run_app(&mut app) {
        Ok(()) => app.exit_code,
        Err(e) => {
            eprintln!("ag3d-risk-slice: {e}");
            3
        }
    }
}

struct RiskApp {
    report_dir: PathBuf,
    gpu: Option<GpuDevice>,
    scene: Option<SceneData>,
    window_kit: Option<WindowKit>,
    offline: Option<(MeshStore, OffscreenTarget, MeshPipeline)>,
    keyframes: Vec<Keyframe>,
    tick: u64,
    pending_plan: usize,
    exit_code: i32,
}

impl winit::application::ApplicationHandler for RiskApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        if self.window_kit.is_none() && self.exit_code != 0 {
            // headless fallback already ran during init
            event_loop.exit();
            return;
        }
        match self.init(event_loop) {
            Ok(()) => {
                if let Some(kit) = &self.window_kit {
                    kit.window.request_redraw();
                } else {
                    event_loop.exit();
                }
            }
            Err(code) => {
                self.exit_code = code;
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(kit) = &self.window_kit {
            if self.tick <= 92 {
                kit.window.request_redraw();
            }
        }
    }
}

impl RiskApp {
    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) -> Result<(), i32> {
        let gpu = GpuDevice::new().map_err(|e| {
            eprintln!("ag3d-risk-slice: device: {e}");
            3
        })?;
        let attrs = winit::window::Window::default_attributes()
            .with_title("AGP ag3d-risk-slice")
            .with_inner_size(winit::dpi::LogicalSize::new(W as f64, H as f64));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("ag3d-risk-slice: create_window: {e}; falling back to headless");
                self.exit_code = run_headless(&self.report_dir);
                return Ok(());
            }
        };
        let size = window.inner_size();
        let target =
            WindowTarget::new(&gpu, window.clone(), size.width, size.height).map_err(|e| {
                eprintln!("ag3d-risk-slice: surface: {e}");
                3
            })?;
        let fmt = target.format();
        let pipeline = MeshPipeline::new(&gpu, fmt).map_err(|e| {
            eprintln!("ag3d-risk-slice: pipeline: {e}");
            3
        })?;
        let offscreen = OffscreenTarget::new(&gpu, W, H).map_err(|e| {
            eprintln!("ag3d-risk-slice: offscreen: {e}");
            3
        })?;
        let off_pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).map_err(|e| {
            eprintln!("ag3d-risk-slice: offscreen pipeline: {e}");
            3
        })?;
        let mut store = MeshStore::default();
        let scene = build_scene(&gpu, &mut store).map_err(|e| {
            eprintln!("ag3d-risk-slice: scene: {e}");
            3
        })?;
        self.window_kit = Some(WindowKit {
            window,
            target,
            pipeline,
            width: size.width,
            height: size.height,
        });
        self.gpu = Some(gpu);
        self.scene = Some(scene);
        self.offline = Some((store, offscreen, off_pipeline));
        Ok(())
    }

    fn frame(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let tick = self.tick;
        while self.pending_plan < KEYFRAME_PLAN.len() && KEYFRAME_PLAN[self.pending_plan].0 == tick
        {
            let (_, name, shadows) = KEYFRAME_PLAN[self.pending_plan];
            self.pending_plan += 1;
            let png = {
                let (gpu, scene, offline) = (
                    self.gpu.as_ref().unwrap(),
                    self.scene.as_ref().unwrap(),
                    self.offline.as_mut().unwrap(),
                );
                match render_keyframe(
                    gpu,
                    &offline.0,
                    &offline.1,
                    &mut offline.2,
                    scene,
                    tick,
                    shadows,
                ) {
                    Ok(png) => png,
                    Err(e) => {
                        eprintln!("ag3d-risk-slice: capture: {e}");
                        self.exit_code = 3;
                        event_loop.exit();
                        return;
                    }
                }
            };
            let file = format!("{name}.png");
            let path = self.report_dir.join(&file);
            if let Err(e) = std::fs::write(&path, &png) {
                eprintln!("ag3d-risk-slice: keyframe write: {e}");
                self.exit_code = 3;
                event_loop.exit();
                return;
            }
            self.keyframes.push(Keyframe {
                name: name.to_string(),
                file,
                width: W,
                height: H,
                sha256: sha256_hex(&png),
            });
        }

        // Present the live scene to the window.
        {
            let kit = self.window_kit.as_mut().unwrap();
            let gpu = self.gpu.as_ref().unwrap();
            let scene = self.scene.as_ref().unwrap();
            let store = &self.offline.as_ref().unwrap().0;
            let instances = frame_instances(scene, tick);
            let w = kit.width;
            let h = kit.height;
            let outcome = kit.target.present_frame(gpu, |view| {
                kit.pipeline.render(
                    gpu,
                    view,
                    w,
                    h,
                    SKY_HORIZON,
                    &scene.camera.view_proj(),
                    &sample_light(),
                    store,
                    &instances,
                    true,
                );
            });
            if let Err(e) = outcome {
                eprintln!("ag3d-risk-slice: present: {e}");
                self.exit_code = 3;
                event_loop.exit();
                return;
            }
        }

        self.tick += 1;
        if self.tick > 92 {
            self.exit_code = self.finish();
            event_loop.exit();
        } else {
            self.window_kit.as_ref().unwrap().window.request_redraw();
        }
    }

    fn finish(&mut self) -> i32 {
        let gpu = self.gpu.as_ref().unwrap();
        let scene = self.scene.as_ref().unwrap();
        let (store, _, _) = self.offline.as_ref().unwrap();
        let stats = Render3dStats {
            draw_calls: (scene.player_keys.len()
                + scene.mob_keys.len() * MOB_SPAWNS.len()
                + 1
                + CYLINDER_POSITIONS.len()) as u32,
            instances: frame_instances(scene, self.tick - 1).len() as u32,
            shadow_enabled: true,
        };
        let (report, pass) =
            build_report_with_scene_gpu(gpu, store, scene, &self.keyframes, &stats);
        write(&self.report_dir, report, pass)
    }
}
