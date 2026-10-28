//! PLAN-857 T-02: actual GPU mesh-depth alternative versus screen-overlay control.
use agp::{
    assets::{MaterialDef, MeshData, ModelData},
    core::scene::Color,
    host::surface_runtime::{Scene3dContent, ScreenOverlay, SurfaceContent, SurfaceRuntime},
    render3d::{Camera3d, MeshInstance, MeshPipeline, MeshStore, Projection},
    renderer::{
        pipeline::{DrawCmd, Shape},
        target::OffscreenTarget,
        GpuDevice,
    },
};
use glam::{Mat4, Vec3};

fn mesh(name: &str, positions: Vec<[f32; 3]>, normal: [f32; 3], color: [f32; 4]) -> MeshData {
    MeshData {
        name: name.into(),
        positions,
        normals: vec![normal; 4],
        indices: vec![0, 1, 2, 0, 2, 3],
        material: MaterialDef {
            name: name.into(),
            base_color: color,
            metallic: 0.0,
            roughness: 1.0,
        },
    }
}
fn pixel(vp: Mat4, p: Vec3) -> [f32; 2] {
    let c = vp * p.extend(1.0);
    [(c.x / c.w + 1.0) * 200.0, (1.0 - c.y / c.w) * 100.0]
}
#[test]
fn ground_mesh_depth_and_top_overlay_are_distinguishable_on_real_gpu() {
    let gpu = GpuDevice::new().unwrap();
    let camera = Camera3d {
        eye: Vec3::new(0.0, 5.0, 5.0),
        target: Vec3::ZERO,
        up: Vec3::Y,
        projection: Projection::Orthographic {
            size: 4.0,
            aspect: 2.0,
            near: 0.05,
            far: 40.0,
        },
    };
    let vp = camera.view_proj();
    let model = ModelData {
        source_sha256: "plan857-depth-decision".into(),
        meshes: vec![
            mesh(
                "ground-line",
                vec![
                    [-2.0, 0.0, 0.05],
                    [2.0, 0.0, 0.05],
                    [2.0, 0.0, -0.05],
                    [-2.0, 0.0, -0.05],
                ],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0, 1.0],
            ),
            mesh(
                "occluder",
                vec![
                    [-0.3, 0.0, 0.5],
                    [0.3, 0.0, 0.5],
                    [0.3, 1.0, 0.5],
                    [-0.3, 1.0, 0.5],
                ],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
            ),
        ],
    };
    let mut store = MeshStore::default();
    for i in 0..model.meshes.len() {
        store.get_or_upload(&gpu, &model, i).unwrap();
    }
    let pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap();
    let instances = model
        .meshes
        .iter()
        .map(|m| MeshInstance {
            mesh: (model.source_sha256.clone(), m.name.clone()),
            model: Mat4::IDENTITY,
        })
        .collect();
    let scene = Scene3dContent {
        screen_overlay: None,
        text_overlay: None,
        mesh: pipeline,
        store,
        instances,
        sky: [0.0, 0.0, 0.0, 1.0],
        view_proj: vp,
    };
    let mut rt = SurfaceRuntime::new(gpu).unwrap();
    let id = rt
        .add_surface([400.0, 200.0], SurfaceContent::Scene3d(Box::new(scene)))
        .unwrap();
    rt.add_view(id, true).unwrap();
    rt.frame(None, &Default::default()).unwrap();
    let Some(SurfaceContent::Scene3d(scene)) = rt.content(id) else {
        panic!("missing scene")
    };
    assert_eq!(
        scene.mesh.editor_grid_owned_bytes(),
        0,
        "grid off creates no auxiliary buffers"
    );
    let mesh_uploads = scene.store.uploads.snapshot();
    let visible = rt
        .probe_pixel(id, pixel(vp, Vec3::new(1.0, 0.0, 0.0)))
        .unwrap();
    let blocked = rt.probe_pixel(id, pixel(vp, Vec3::ZERO)).unwrap();
    assert!(
        visible[1] > visible[0] + 20 && visible[1] > 30,
        "visible ground line: {visible:?}"
    );
    assert!(
        blocked[0] > blocked[1] + 20 && blocked[0] > 30,
        "foreground occluder: {blocked:?}"
    );
    let mut overlay = ScreenOverlay::new(rt.gpu()).unwrap();
    overlay.commands.push(DrawCmd {
        center: [200.0, 100.0],
        size: [200.0, 5.0],
        z: 0.0,
        color: Color::rgb(0.0, 1.0, 0.0),
        shape: Shape::Rect,
    });
    let Some(SurfaceContent::Scene3d(scene)) = rt.content_mut(id) else {
        panic!("missing scene")
    };
    scene.screen_overlay = Some(overlay);
    rt.graph.mark_content_dirty(id);
    rt.frame(None, &Default::default()).unwrap();
    let wrong = rt.probe_pixel(id, pixel(vp, Vec3::ZERO)).unwrap();
    assert!(
        wrong[1] > wrong[0] + 20,
        "top overlay must fail occlusion oracle: {wrong:?}"
    );
    let Some(SurfaceContent::Scene3d(scene)) = rt.content_mut(id) else {
        panic!("missing scene")
    };
    scene.screen_overlay = None;
    use agp::render3d::editor_grid::EditorGridVertex;
    let grid = [
        EditorGridVertex {
            position: [-2.0, 0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        EditorGridVertex {
            position: [2.0, 0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];
    rt.set_editor_grid(id, Some(&grid)).unwrap();
    rt.frame(None, &Default::default()).unwrap();
    let blocked_grid = rt.probe_pixel(id, pixel(vp, Vec3::ZERO)).unwrap();
    assert!(
        blocked_grid[0] > blocked_grid[2] + 20,
        "depth grid must remain behind blocker: {blocked_grid:?}"
    );
    let at = pixel(vp, Vec3::new(1.0, 0.0, 0.0));
    let samples: Vec<_> = (-2..=2)
        .map(|dy| rt.probe_pixel(id, [at[0], at[1] + dy as f32]).unwrap())
        .collect();
    assert!(
        samples.iter().any(|v| v[2] > v[0] + 20 && v[2] > v[1] + 20),
        "depth grid visible outside blocker: {samples:?}"
    );
    let Some(SurfaceContent::Scene3d(scene)) = rt.content(id) else {
        panic!("missing scene")
    };
    assert!(scene.mesh.editor_grid_owned_bytes() <= 65536);
    assert_eq!(
        scene.store.uploads.snapshot().buffer_rewrites,
        mesh_uploads.buffer_rewrites,
        "grid does not reupload static model"
    );
    let uploads = scene.mesh.editor_grid_uploads();
    rt.frame(None, &Default::default()).unwrap();
    let Some(SurfaceContent::Scene3d(scene)) = rt.content(id) else {
        panic!("missing scene")
    };
    let after = scene.mesh.editor_grid_uploads();
    assert_eq!(
        uploads.buffer_rewrites, after.buffer_rewrites,
        "unchanged grid must not upload"
    );
    let revision = scene.mesh.editor_grid_revision();
    let mut bad = grid;
    bad[0].position[0] = f32::NAN;
    assert!(rt.set_editor_grid(id, Some(&bad)).is_err());
    let Some(SurfaceContent::Scene3d(scene)) = rt.content(id) else {
        panic!("missing scene")
    };
    assert_eq!(
        scene.mesh.editor_grid_revision(),
        revision,
        "invalid grid keeps old state"
    );
    rt.set_editor_grid(id, None).unwrap();
    rt.frame(None, &Default::default()).unwrap();
    let Some(SurfaceContent::Scene3d(scene)) = rt.content(id) else {
        panic!("missing scene")
    };
    assert_eq!(
        after.buffer_rewrites,
        scene.mesh.editor_grid_uploads().buffer_rewrites,
        "grid off must not upload"
    );
}
