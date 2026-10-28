use autographics_auto_driver::{ResultKind, Scalar::*, Session};
#[test]
fn numeric_and_ordered_random_probe() {
    let mut s = Session::new(include_str!("../samples/squash-probe/main.at")).unwrap();
    for n in [
        0.0f32,
        0.000000000001,
        0.000001,
        0.5,
        1.0,
        2.0,
        196.0,
        961.0,
    ] {
        let got = s.float("root", &[Float(n)]).unwrap();
        if n == 0.0 {
            assert_eq!(got, 0.0)
        } else {
            assert!((got / n.sqrt() - 1.0).abs() < 0.000001, "sqrt {n}: {got}");
        }
    }
    for (y, x) in [
        (1.0f32, 1.0f32),
        (1.0, -1.0),
        (-1.0, -1.0),
        (-1.0, 1.0),
        (0.0, 0.0),
    ] {
        let got = s.float("angle", &[Float(y), Float(x)]).unwrap();
        assert!(
            (got - y.atan2(x)).abs() < 0.00001,
            "atan2 {y}/{x}: got {got} expected {}",
            y.atan2(x)
        );
    }
    for i in -64..=64 {
        let x = i as f32 * std::f32::consts::PI / 16.0;
        assert!((s.float("sine", &[Float(x)]).unwrap() - x.sin()).abs() < 0.00001);
        assert!((s.float("cosine", &[Float(x)]).unwrap() - x.cos()).abs() < 0.00001);
    }
    let mut rng = agp::core::DeterministicRng::new(27);
    for sequence in 1..=135 {
        let v = rng.next_f32();
        assert_eq!(
            s.int("accept_random", &[Int(sequence), Float(v)]).unwrap(),
            1
        );
        assert_eq!(s.float("value", &[]).unwrap().to_bits(), v.to_bits());
        assert_eq!(
            s.int("accept_random", &[Int(sequence), Float(v)]).unwrap(),
            0
        );
        assert_eq!(s.int("count", &[]).unwrap(), sequence);
    }
    s.call("reset", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("count", &[]).unwrap(), 0);
}

#[test]
fn auto_transform_reaches_original_glb_pixels() {
    use agp::{
        assets::load_glb_bytes,
        host::surface_runtime::{Scene3dContent, SurfaceContent, SurfaceRuntime},
        render3d::{MeshInstance, MeshPipeline, MeshStore},
        renderer::{target::OffscreenTarget, GpuDevice},
    };
    use glam::{Mat4, Vec3};
    let source = include_str!("../samples/squash-probe/main.at");
    let mut auto = Session::new(source).unwrap();
    let gpu = GpuDevice::new().unwrap();
    let model = load_glb_bytes(include_bytes!(
        "../../autographics-prototype/assets/squash_the_creeps/art/mob.glb"
    ))
    .unwrap();
    assert_eq!(model.meshes.len(), 3);
    let mut store = MeshStore::default();
    for i in 0..model.meshes.len() {
        store.get_or_upload(&gpu, &model, i).unwrap();
    }
    let content = Scene3dContent {
        screen_overlay: None,
        text_overlay: None,
        mesh: MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap(),
        store,
        instances: vec![],
        sky: [0.1, 0.2, 0.3, 1.0],
        view_proj: agp::render3d::Camera3d::sample_camera(1.5).view_proj(),
    };
    let mut rt = SurfaceRuntime::new(gpu).unwrap();
    let id = rt
        .add_surface([450.0, 300.0], SurfaceContent::Scene3d(Box::new(content)))
        .unwrap();
    rt.add_view(id, true).unwrap();
    let mut hashes = vec![];
    for (seq, value) in [(1, 0.1), (2, 0.7)] {
        assert_eq!(
            auto.int("accept_random", &[Int(seq), Float(value)])
                .unwrap(),
            1
        );
        let x = auto.float("transform", &[Int(0)]).unwrap();
        let sy = auto.float("transform", &[Int(1)]).unwrap();
        let cy = auto.float("transform", &[Int(2)]).unwrap();
        let yaw = auto.float("angle", &[Float(sy), Float(cy)]).unwrap();
        let m = Mat4::from_translation(Vec3::new(x, 0.0, 0.0)) * Mat4::from_rotation_y(yaw);
        if let Some(SurfaceContent::Scene3d(c)) = rt.content_mut(id) {
            c.instances = model
                .meshes
                .iter()
                .map(|mesh| MeshInstance {
                    mesh: (model.source_sha256.clone(), mesh.name.clone()),
                    model: m,
                })
                .collect();
        }
        rt.graph.mark_content_dirty(id);
        rt.frame(None, &Default::default()).unwrap();
        hashes.push(rt.capture(id, "probe").unwrap().0);
    }
    assert_ne!(
        hashes[0], hashes[1],
        "Auto state did not reach actual GLB pixels"
    );
    println!("original_mob_meshes=3 images={hashes:?}");
}

#[test]
fn bounded_math_load() {
    let mut s = Session::new(include_str!("../samples/squash-probe/main.at")).unwrap();
    let now = std::time::Instant::now();
    let result = s.float("pathological_math", &[Int(64)]);
    println!(
        "64-mob numeric loop elapsed_us={} slices={} result={result:?}",
        now.elapsed().as_micros(),
        s.yielded
    );
    assert!(result.is_ok());
}
