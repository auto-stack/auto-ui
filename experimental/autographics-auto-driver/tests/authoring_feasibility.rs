use agp::{
    assets::load_glb_bytes,
    core::scene::Color,
    host::{
        iced_surface::{IcedSurface, SurfaceElement, SurfaceUi},
        surface_runtime::{Game2dContent, Scene3dContent, SurfaceContent, SurfaceRuntime},
    },
    render3d::{MeshInstance, MeshPipeline, MeshStore},
    renderer::{
        pipeline::{DrawCmd, Shape},
        target::OffscreenTarget,
        GpuDevice,
    },
};
use autographics_auto_driver::{ResultKind, Scalar::Float, Session};
use glam::{Mat4, Vec3};
#[derive(Default)]
struct Ui {
    value: f32,
}
impl SurfaceUi for Ui {
    type Message = ();
    fn view(&self) -> SurfaceElement<'_, ()> {
        iced::widget::button("Apply candidate").on_press(()).into()
    }
    fn update(&mut self, _: ()) {
        self.value = 2.0;
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
#[test]
fn widget_auto_pixels_and_local_replace() {
    let gpu = GpuDevice::new().unwrap();
    let mut ui = Ui::default();
    let mut iced = IcedSurface::new(&gpu, iced::Size::new(300.0, 100.0), 1.0).unwrap();
    let cursor = iced_wgpu::core::mouse::Cursor::Available(iced::Point::new(30.0, 15.0));
    iced.frame(&mut ui, &[], cursor);
    let events = [
        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
            iced_wgpu::core::mouse::Button::Left,
        )),
        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonReleased(
            iced_wgpu::core::mouse::Button::Left,
        )),
    ];
    assert_eq!(
        iced.frame(&mut ui, &events, cursor).len(),
        1,
        "real widget must consume click"
    );
    assert_eq!(ui.value, 2.0);
    let model = load_glb_bytes(include_bytes!(
        "../../autographics-prototype/assets/squash_the_creeps/art/mob.glb"
    ))
    .unwrap();
    let mut store = MeshStore::default();
    for i in 0..model.meshes.len() {
        store.get_or_upload(&gpu, &model, i).unwrap();
    }
    let scene = Scene3dContent {
        screen_overlay: None,
        text_overlay: None,
        mesh: MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap(),
        store,
        instances: vec![],
        sky: [0.1, 0.2, 0.3, 1.0],
        view_proj: agp::render3d::Camera3d::sample_camera(1.5).view_proj(),
    };
    let g = Game2dContent::simple(&gpu, Color::rgb(0.02, 0.03, 0.04));
    let mut rt = SurfaceRuntime::new(gpu).unwrap();
    let two = rt
        .add_surface([450.0, 300.0], SurfaceContent::Game2d(Box::new(g)))
        .unwrap();
    rt.add_view(two, true).unwrap();
    let three = rt
        .add_surface([450.0, 300.0], SurfaceContent::Scene3d(Box::new(scene)))
        .unwrap();
    rt.add_view(three, true).unwrap();
    let mut hashes = vec![];
    let mut auto = Session::new(include_str!("../research/authoring-probe/main.at")).unwrap();
    for v in [0.0, ui.value, 0.0] {
        auto.call("configure", &[Float(v)], ResultKind::Unit)
            .unwrap();
        let x = auto.float("field", &[]).unwrap();
        assert_eq!(x, v);
        if let Some(SurfaceContent::Game2d(g)) = rt.content_mut(two) {
            g.commands = vec![DrawCmd {
                center: [225.0 + x * 40.0, 150.0],
                size: [50.0, 50.0],
                z: 0.0,
                color: Color::rgb(1.0, 0.0, 0.0),
                shape: Shape::Circle,
            }];
        }
        if let Some(SurfaceContent::Scene3d(c)) = rt.content_mut(three) {
            c.instances = model
                .meshes
                .iter()
                .map(|mesh| MeshInstance {
                    mesh: (model.source_sha256.clone(), mesh.name.clone()),
                    model: Mat4::from_translation(Vec3::new(x, 0.0, 0.0)),
                })
                .collect();
        }
        rt.graph.mark_content_dirty(two);
        rt.graph.mark_content_dirty(three);
        rt.frame(None, &Default::default()).unwrap();
        hashes.push((
            rt.capture(two, "authoring-2d").unwrap().0,
            rt.capture(three, "authoring-3d").unwrap().0,
        ));
    }
    assert_ne!(hashes[0].0, hashes[1].0);
    assert_ne!(hashes[0].1, hashes[1].1);
    assert_eq!(hashes[0], hashes[2]);
    let dir = std::env::temp_dir().join(format!("ag-authoring-probe-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("scene.agscene");
    let temp = dir.join("owned-stage");
    std::fs::write(&path, b"old").unwrap();
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .unwrap();
    use std::io::Write;
    let bytes = serde_json::to_vec(&serde_json::json!({"x":ui.value})).unwrap();
    f.write_all(&bytes).unwrap();
    f.sync_all().unwrap();
    drop(f);
    assert_eq!(std::fs::read(&path).unwrap(), b"old");
    std::fs::rename(&temp, &path).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let reopened: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(reopened["x"], 2.0);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
    println!("widget Auto original3mesh GPU restore and same-directory replacement passed; compile_us={}",auto.compile_us);
}
