//! Bounded PLAN-838 probe; not the production admission/codec acceptance suite.
use agp::{
    assets::load_glb_bytes,
    host::{
        iced_surface::{IcedSurface, SurfaceElement, SurfaceUi},
        surface_runtime::{Scene3dContent, SurfaceContent, SurfaceRuntime},
    },
    render3d::{MeshInstance, MeshPipeline, MeshStore},
    renderer::{target::OffscreenTarget, GpuDevice},
};
use autographics_auto_driver::{ResultKind, Scalar, Session};
use glam::{Mat4, Vec3};
use std::{path::Path, sync::Arc};
const GLB: &[u8] = include_bytes!("../research/asset-probe/two-parts.glb");
const SOURCE: &str = "var selected=0\nvar world=0.0\nfn main() {}\nfn profile() int {return 5}\nfn configure(resource int, parent float, local float) {selected=resource; world=parent+local}\nfn resource() int {return selected}\nfn x() float {return world}\n";
#[derive(Default)]
struct Import {
    clicked: bool,
}
impl SurfaceUi for Import {
    type Message = ();
    fn view(&self) -> SurfaceElement<'_, ()> {
        iced::widget::button("Import GLB").on_press(()).into()
    }
    fn update(&mut self, _: ()) {
        self.clicked = true;
    }
    fn background(&self) -> iced::Color {
        iced::Color::BLACK
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
fn render(bytes: &[u8], x: f32) -> (String, usize) {
    let mut model = load_glb_bytes(bytes).unwrap();
    assert_eq!(model.meshes.len(), 2);
    // Old loader's duplicate names collide. The proposed adapter uses ordinals.
    assert_eq!(model.meshes[0].name, model.meshes[1].name);
    for (i, m) in model.meshes.iter_mut().enumerate() {
        m.name = format!("primitive-{i}");
    }
    let bounds = autographics_auto_driver::authoring::interaction::bounds(&model).unwrap();
    assert!(
        bounds.0.x < -2.0 && bounds.1.x > 2.0,
        "OBB includes both parts"
    );
    let gpu = GpuDevice::new().unwrap();
    let mut ui = Import::default();
    let mut iced = IcedSurface::new(&gpu, iced::Size::new(300.0, 100.0), 1.0).unwrap();
    let cursor = iced_wgpu::core::mouse::Cursor::Available(iced::Point::new(30.0, 15.0));
    iced.frame(&mut ui, &[], cursor);
    use iced_wgpu::core::{
        mouse::{Button, Event as Mouse},
        Event,
    };
    assert_eq!(
        iced.frame(
            &mut ui,
            &[
                Event::Mouse(Mouse::ButtonPressed(Button::Left)),
                Event::Mouse(Mouse::ButtonReleased(Button::Left))
            ],
            cursor
        )
        .len(),
        1
    );
    assert!(ui.clicked);
    let resident = MeshStore::default();
    let mut candidate = resident.staged();
    for i in 0..2 {
        candidate.get_or_upload(&gpu, &model, i).unwrap();
    }
    assert_eq!(resident.uploaded_count(), 0);
    assert_eq!(
        resident.uploads.snapshot().buffer_rewrites,
        4,
        "rejected writes cannot disappear"
    );
    let pin = candidate
        .pin(&model.source_sha256, &model.meshes[0].name)
        .unwrap();
    let weak = Arc::downgrade(&pin);
    let mut warm = candidate.staged();
    for i in 0..2 {
        warm.get_or_upload(&gpu, &model, i).unwrap();
    }
    assert!(Arc::ptr_eq(
        &pin,
        &warm
            .pin(&model.source_sha256, &model.meshes[0].name)
            .unwrap()
    ));
    assert_eq!(resident.uploads.snapshot().buffer_rewrites, 4);
    drop(warm);
    drop(candidate);
    assert!(weak.upgrade().is_some());
    drop(pin);
    assert!(weak.upgrade().is_none());
    let mut candidate = resident.staged();
    for i in 0..2 {
        candidate.get_or_upload(&gpu, &model, i).unwrap();
    }
    let mut bad = model.clone();
    bad.meshes[0].indices[0] = 65536;
    // A distinct identity is required to exercise admission rather than cache hit.
    bad.source_sha256 = "invalid-probe".into();
    assert!(candidate.get_or_upload(&gpu, &bad, 0).is_err());
    let potential = |parts: usize| -> Result<(), &'static str> {
        if parts > agp::render3d::pipeline::MAX_INSTANCES {
            Err("potential instances")
        } else {
            Ok(())
        }
    };
    assert!(potential(512).is_ok());
    assert!(potential(513).is_err());
    let mut auto = Session::new(SOURCE).unwrap();
    auto.call(
        "configure",
        &[Scalar::Int(0), Scalar::Float(x), Scalar::Float(0.5)],
        ResultKind::Unit,
    )
    .unwrap();
    assert_eq!(auto.int("resource", &[]).unwrap(), 0);
    let world = auto.float("x", &[]).unwrap();
    assert_eq!(world, x + 0.5);
    let instances = model
        .meshes
        .iter()
        .map(|m| MeshInstance {
            mesh: (model.source_sha256.clone(), m.name.clone()),
            model: Mat4::from_translation(Vec3::new(world, 0.0, 0.0)),
        })
        .collect();
    let mesh = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap();
    let mut rt = SurfaceRuntime::new(gpu).unwrap();
    let id = rt
        .add_surface(
            [800.0, 400.0],
            SurfaceContent::Scene3d(Box::new(Scene3dContent {
                store: candidate,
                mesh,
                instances,
                sky: [0.1, 0.2, 0.3, 1.0],
                view_proj: agp::render3d::Camera3d::sample_camera(2.0).view_proj(),
                screen_overlay: None,
                text_overlay: None,
            })),
        )
        .unwrap();
    rt.add_view(id, true).unwrap();
    rt.frame(None, &Default::default()).unwrap();
    for (i, center) in [
        Vec3::new(-1.5 + world, 0.5, 0.0),
        Vec3::new(1.5 + world, 0.5, 0.0),
    ]
    .into_iter()
    .enumerate()
    {
        let p = autographics_auto_driver::authoring::interaction::project(
            agp::render3d::Camera3d::sample_camera(2.0).view_proj(),
            center,
        )
        .unwrap();
        let pixel = rt.probe_pixel(id, p.to_array()).unwrap();
        assert!(
            pixel[i] > 80 && pixel[1 - i] < 20 && pixel[2] < 20,
            "part {i} must have its own red/green GPU color at its independent centroid: {pixel:?}"
        );
    }
    let (hash, _, _, _) = rt.capture(id, "asset-feasibility").unwrap();
    (hash, model.meshes.len())
}
#[test]
fn portable_child() {
    let Some(root) = std::env::var_os("AG_ASSET_PROBE_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let entry: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("scene.agscene")).unwrap()).unwrap();
    let bytes = std::fs::read(root.join("assets").join(entry["blob"].as_str().unwrap())).unwrap();
    assert_eq!(agp::renderer::capture::sha256_hex(&bytes), entry["sha"]);
    assert_eq!(render(&bytes, 2.0).0, entry["image"].as_str().unwrap());
}
#[test]
fn iced_auto_staged_all_parts_and_portable_entry() {
    let initial = render(GLB, 0.0);
    let moved = render(GLB, 2.0);
    assert_ne!(initial.0, moved.0);
    assert_eq!(initial.1, 2);
    let root = std::env::temp_dir().join(format!(
        "ag838-feasibility-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(root.join("assets")).unwrap();
    let sha = agp::renderer::capture::sha256_hex(GLB);
    let name = format!("{sha}.glb");
    let source = root.join("original.glb");
    std::fs::write(&source, GLB).unwrap();
    let entry = root.join("scene.agscene");
    std::fs::write(&entry, b"old entry").unwrap();
    std::fs::write(
        root.join("assets").join(&name),
        std::fs::read(&source).unwrap(),
    )
    .unwrap();
    // Simulate entry-before-commit fault; immutable residual can be reused.
    assert_eq!(std::fs::read(&entry).unwrap(), b"old entry");
    assert_eq!(std::fs::read(root.join("assets").join(&name)).unwrap(), GLB);
    let temp = root.join("entry-stage");
    std::fs::write(
        &temp,
        serde_json::to_vec(&serde_json::json!({"blob":name,"sha":sha,"image":moved.0})).unwrap(),
    )
    .unwrap();
    std::fs::rename(temp, &entry).unwrap();
    std::fs::remove_file(source).unwrap();
    let relocated = root.with_extension("relocated");
    std::fs::rename(&root, &relocated).unwrap();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "portable_child", "--nocapture"])
        .env("AG_ASSET_PROBE_ROOT", &relocated)
        .current_dir(std::env::temp_dir())
        .status()
        .unwrap();
    assert!(child.success(), "fresh-process closure/real GPU reopen");
    // Explicit owned files only; no recursive removal or links.
    std::fs::remove_file(relocated.join("assets").join(name)).unwrap();
    std::fs::remove_file(relocated.join("scene.agscene")).unwrap();
    std::fs::remove_dir(relocated.join("assets")).unwrap();
    std::fs::remove_dir(relocated).unwrap();
}
