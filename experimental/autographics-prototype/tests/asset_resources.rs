use agp::{
    assets::{MaterialDef, MeshData, ModelData},
    host::surface_runtime::{Scene3dContent, SurfaceContent, SurfaceRuntime},
    render3d::{Camera3d, MeshInstance, MeshPipeline, MeshStore},
    renderer::{target::OffscreenTarget, GpuDevice},
};
use glam::{Mat4, Vec3};
use std::sync::Arc;
#[test]
fn staged_same_device_last_pin_u16_boundary_and_aligned_actual_draw() {
    let gpu = GpuDevice::new().unwrap();
    let foreign = GpuDevice::new().unwrap();
    let mut positions = vec![[0.0; 3]; 65536];
    positions[0] = [-0.7, 0.0, 0.0];
    positions[1] = [0.7, 0.0, 0.0];
    positions[65535] = [0.0, 1.5, 0.0];
    let model = ModelData {
        source_sha256: "p838-resource".into(),
        meshes: vec![MeshData {
            name: "primitive-0".into(),
            positions,
            normals: vec![[0.0, 0.0, 1.0]; 65536],
            indices: vec![0, 1, 65535],
            material: MaterialDef {
                name: "red".into(),
                base_color: [1.0, 0.0, 0.0, 1.0],
                metallic: 0.0,
                roughness: 1.0,
            },
        }],
    };
    let mut resident = MeshStore::default();
    resident.get_or_upload(&gpu, &model, 0).unwrap();
    let pin = resident.pin(&model.source_sha256, "primitive-0").unwrap();
    assert_eq!(pin.index_count, 3);
    assert_eq!(pin.index_buf.size(), 8);
    let stats = resident.uploads.snapshot();
    let mut candidate = resident.staged();
    candidate.get_or_upload(&gpu, &model, 0).unwrap();
    assert!(Arc::ptr_eq(
        &pin,
        &candidate.pin(&model.source_sha256, "primitive-0").unwrap()
    ));
    assert!(candidate.get_or_upload(&foreign, &model, 0).is_err());
    assert_eq!(
        resident.uploads.snapshot().buffer_rewrites,
        stats.buffer_rewrites
    );
    let mut bad = model.clone();
    bad.source_sha256 = "bad-65536".into();
    bad.meshes[0].positions.push([0.0, 0.0, 0.0]);
    bad.meshes[0].normals.push([0.0, 0.0, 1.0]);
    bad.meshes[0].indices[2] = 65536;
    assert!(candidate.get_or_upload(&gpu, &bad, 0).is_err());
    assert_eq!(
        resident.uploads.snapshot().buffer_rewrites,
        stats.buffer_rewrites
    );
    let weak = Arc::downgrade(&pin);
    drop(candidate);
    drop(resident);
    assert!(weak.upgrade().is_some());
    let mut store = MeshStore::default();
    store.get_or_upload(&gpu, &model, 0).unwrap();
    let mesh = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap();
    let vp = Camera3d::sample_camera(2.0).view_proj();
    let mut rt = SurfaceRuntime::new(gpu).unwrap();
    let id = rt
        .add_surface(
            [800.0, 400.0],
            SurfaceContent::Scene3d(Box::new(Scene3dContent {
                store,
                mesh,
                instances: vec![MeshInstance {
                    mesh: (model.source_sha256.clone(), "primitive-0".into()),
                    model: Mat4::IDENTITY,
                }],
                sky: [0.1, 0.2, 0.3, 1.0],
                view_proj: vp,
                screen_overlay: None,
                text_overlay: None,
            })),
        )
        .unwrap();
    rt.add_view(id, true).unwrap();
    rt.frame(None, &Default::default()).unwrap();
    let c = vp * Vec3::new(0.0, 0.5, 0.0).extend(1.0);
    let p = [(c.x / c.w + 1.0) * 400.0, (1.0 - c.y / c.w) * 200.0];
    let pixel = rt.probe_pixel(id, p).unwrap();
    assert!(
        pixel[0] > 80 && pixel[1] < 20 && pixel[2] < 20,
        "actual index65535 draw: {pixel:?}"
    );
    drop(pin);
    assert!(weak.upgrade().is_none());
}
