//! PLAN-005 independent numerical pixel oracles, without windows or OS input.
use agp::{
    assets::{
        textured::{MeshVisual, Sampling},
        ImageData, MaterialDef, MeshData, ModelData,
    },
    render3d::{LightDef, MeshInstance, MeshPipeline, MeshStore},
    renderer::{
        capture::{capture_png, sha256_hex},
        target::OffscreenTarget,
        GpuDevice,
    },
};
use glam::{Mat4, Vec3};
use std::sync::Arc;

fn image() -> Arc<ImageData> {
    let mut rgba = vec![];
    for y in 0..4 {
        for x in 0..4 {
            rgba.extend([
                20 + x * 40 + y * 10,
                15 + x * 15 + y * 45,
                240 - x * 30 - y * 20,
                255,
            ]);
        }
    }
    Arc::new(ImageData {
        width: 4,
        height: 4,
        source_sha256: sha256_hex(&rgba),
        rgba,
    })
}
fn solid(rgb: [u8; 3]) -> Arc<ImageData> {
    let rgba = vec![rgb[0], rgb[1], rgb[2], 255];
    Arc::new(ImageData {
        width: 1,
        height: 1,
        source_sha256: sha256_hex(&rgba),
        rgba,
    })
}
fn quad(
    name: &str,
    z: f32,
    image: Arc<ImageData>,
    sampling: Sampling,
    rect: [f32; 4],
    double: bool,
    reverse: bool,
) -> ModelData {
    let uv = vec![
        [rect[0], rect[1] + rect[3]],
        [rect[0] + rect[2], rect[1] + rect[3]],
        [rect[0] + rect[2], rect[1]],
        [rect[0], rect[1]],
    ];
    ModelData {
        source_sha256: name.into(),
        meshes: vec![MeshData {
            name: "primitive-0".into(),
            positions: vec![[-1., -1., z], [1., -1., z], [1., 1., z], [-1., 1., z]],
            normals: vec![[0., 0., 1.]; 4],
            indices: if reverse {
                vec![0, 2, 1, 0, 3, 2]
            } else {
                vec![0, 1, 2, 0, 2, 3]
            },
            material: MaterialDef {
                name: "factor".into(),
                base_color: [0.35, 0.65, 0.8, 1.],
                metallic: 0.,
                roughness: 1.,
            },
            visual: Some(MeshVisual {
                uv,
                image: Some(image),
                sampling,
                double_sided: double,
                node_index: 0,
                mesh_index: 0,
                primitive_index: 0,
                world: Mat4::IDENTITY.to_cols_array_2d(),
            }),
        }],
    }
}
fn lighting(direction: Vec3) -> LightDef {
    LightDef {
        dir_to_light: direction,
        color: [0.9, 1., 0.7],
        energy: 0.5,
        shadow_vp: Mat4::IDENTITY,
        shadow_bias: 0.0005,
    }
}
fn instance(m: &ModelData) -> MeshInstance {
    MeshInstance {
        mesh: (m.source_sha256.clone(), m.meshes[0].name.clone()),
        model: Mat4::IDENTITY,
    }
}
fn draw(
    gpu: &GpuDevice,
    pipeline: &mut MeshPipeline,
    store: &MeshStore,
    instances: &[MeshInstance],
    size: u32,
    light: &LightDef,
    shadows: bool,
) -> image::RgbaImage {
    let target = OffscreenTarget::new(gpu, size, size).unwrap();
    let stats = pipeline.render(
        gpu,
        target.view(),
        size,
        size,
        [0., 0., 0., 1.],
        &Mat4::IDENTITY,
        light,
        store,
        instances,
        shadows,
    );
    assert_eq!(stats.instances, instances.len() as u32);
    capture_png(gpu, &target).unwrap().1
}
fn linear(v: u8) -> f32 {
    let x = v as f32 / 255.;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(v: f32) -> u8 {
    let x = v.clamp(0., 1.);
    let x = if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1. / 2.4) - 0.055
    };
    (x * 255.).round() as u8
}
fn index(i: i32, n: i32, wrap: u32) -> usize {
    match wrap {
        33071 => i.clamp(0, n - 1) as usize,
        10497 => i.rem_euclid(n) as usize,
        33648 => {
            let j = i.rem_euclid(2 * n);
            if j < n {
                j as usize
            } else {
                (2 * n - 1 - j) as usize
            }
        }
        _ => panic!("unsupported test wrap"),
    }
}
fn sample(image: &ImageData, uv: [f32; 2], s: Sampling, filter: u32) -> [f32; 3] {
    let texel = |x, y| {
        let x = index(x, image.width as i32, s.wrap_s);
        let y = index(y, image.height as i32, s.wrap_t);
        let at = (y * image.width as usize + x) * 4;
        [
            linear(image.rgba[at]),
            linear(image.rgba[at + 1]),
            linear(image.rgba[at + 2]),
        ]
    };
    if filter == 9728 {
        return texel(
            (uv[0] * image.width as f32).floor() as i32,
            (uv[1] * image.height as f32).floor() as i32,
        );
    }
    let x = uv[0] * image.width as f32 - 0.5;
    let y = uv[1] * image.height as f32 - 0.5;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x.fract().rem_euclid(1.);
    let fy = y.fract().rem_euclid(1.);
    let a = texel(ix, iy);
    let b = texel(ix + 1, iy);
    let c = texel(ix, iy + 1);
    let d = texel(ix + 1, iy + 1);
    std::array::from_fn(|i| {
        (a[i] * (1. - fx) + b[i] * fx) * (1. - fy) + (c[i] * (1. - fx) + d[i] * fx) * fy
    })
}
fn expected(tex: [f32; 3], factor: [f32; 4], light: &LightDef, lit: f32) -> [u8; 3] {
    std::array::from_fn(|i| srgb(tex[i] * factor[i] * light.color[i] * lit))
}
fn error(pixel: [u8; 4], expected: [u8; 3]) -> u8 {
    (0..3)
        .map(|i| pixel[i].abs_diff(expected[i]))
        .max()
        .unwrap()
}
fn report(name: &str, value: serde_json::Value) {
    println!("UI005 {name}: {value}");
    if let Some(root) = std::env::var_os("UI005_REPORT_DIR") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn sampling_modes_srgb_and_material_factor_match_an_independent_cpu_oracle() {
    let gpu = GpuDevice::new().unwrap();
    let mut pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap();
    let mut store = MeshStore::default();
    let image = image();
    let light = lighting(Vec3::Z);
    let rect = [-0.45, -0.33, 2.7, 2.3];
    let mut cases = 0;
    let mut probes = 0;
    let mut maximum = 0;
    for minifying in [false, true] {
        for filter in [9728, 9729] {
            for s in [33071, 10497, 33648] {
                for t in [33071, 10497, 33648] {
                    let opposite = if filter == 9728 { 9729 } else { 9728 };
                    let sampling = Sampling {
                        mag: if minifying { opposite } else { filter },
                        min: if minifying { filter } else { opposite },
                        wrap_s: s,
                        wrap_t: t,
                    };
                    let model = quad(
                        &format!("sampling-{cases}"),
                        0.5,
                        image.clone(),
                        sampling,
                        rect,
                        false,
                        false,
                    );
                    store.get_or_upload(&gpu, &model, 0).unwrap();
                    let size = if minifying { 4 } else { 64 };
                    let pixels = draw(
                        &gpu,
                        &mut pipeline,
                        &store,
                        &[instance(&model)],
                        size,
                        &light,
                        false,
                    );
                    let points = if minifying {
                        vec![(0, 1), (1, 3), (3, 0)]
                    } else {
                        vec![(5, 9), (17, 25), (42, 51), (60, 2), (31, 33)]
                    };
                    for (x, y) in points {
                        let uv = [
                            rect[0] + rect[2] * (x as f32 + 0.5) / size as f32,
                            rect[1] + rect[3] * (y as f32 + 0.5) / size as f32,
                        ];
                        let expect = expected(
                            sample(&image, uv, sampling, filter),
                            model.meshes[0].material.base_color,
                            &light,
                            0.75,
                        );
                        let got = pixels.get_pixel(x, y).0;
                        let e = error(got, expect);
                        maximum = maximum.max(e);
                        probes += 1;
                        assert!(e<=3,"case {cases} minify={minifying} filter={filter} wrap={s}/{t} uv={uv:?} {x},{y}: GPU={got:?} CPU={expect:?} error={e}");
                    }
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 36);
    assert_eq!(probes, 144);
    assert_eq!(
        store.texture_count(),
        1,
        "same image is one real GPU texture"
    );
    // A frozen UV and a flat-colour substitution are plausible wrong paths.
    let sampling = Sampling {
        mag: 9728,
        min: 9728,
        wrap_s: 33071,
        wrap_t: 33071,
    };
    let mut mutations = 0;
    for frozen in [true, false] {
        let mut model = quad(
            &format!("sampling-mutation-{frozen}"),
            0.5,
            image.clone(),
            sampling,
            [0., 0., 1., 1.],
            false,
            false,
        );
        if frozen {
            model.meshes[0]
                .visual
                .as_mut()
                .unwrap()
                .uv
                .fill([0.125, 0.125]);
        } else {
            model.meshes[0].visual.as_mut().unwrap().image = None;
        }
        store.get_or_upload(&gpu, &model, 0).unwrap();
        let pixels = draw(
            &gpu,
            &mut pipeline,
            &store,
            &[instance(&model)],
            64,
            &light,
            false,
        );
        let failures = [(10, 10), (50, 10), (10, 50), (50, 50)]
            .into_iter()
            .filter(|&(x, y)| {
                let expect = expected(
                    sample(
                        &image,
                        [(x as f32 + 0.5) / 64., (y as f32 + 0.5) / 64.],
                        sampling,
                        9728,
                    ),
                    model.meshes[0].material.base_color,
                    &light,
                    0.75,
                );
                error(pixels.get_pixel(x, y).0, expect) > 10
            })
            .count();
        assert!(
            failures >= 3,
            "wrong UV/flat colour must fail the real pixel oracle"
        );
        mutations += 1;
    }
    report(
        "sampling",
        serde_json::json!({"cases":cases,"probes":probes,"max_error":maximum,"tolerance":3,"mutation_rejections":mutations,"texture_count":store.texture_count()}),
    );
}

#[test]
fn textured_depth_and_double_sided_normal_orientation_are_visible_in_real_pixels() {
    let gpu = GpuDevice::new().unwrap();
    let mut pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap();
    let mut store = MeshStore::default();
    let sampling = Sampling::default();
    let mut red = quad(
        "front-red",
        0.25,
        solid([255, 0, 0]),
        sampling,
        [0., 0., 1., 1.],
        false,
        false,
    );
    let mut blue = quad(
        "back-blue",
        0.75,
        solid([0, 0, 255]),
        sampling,
        [0., 0., 1., 1.],
        false,
        false,
    );
    red.meshes[0].material.base_color = [1.; 4];
    blue.meshes[0].material.base_color = [1.; 4];
    store.get_or_upload(&gpu, &red, 0).unwrap();
    store.get_or_upload(&gpu, &blue, 0).unwrap();
    let light = lighting(Vec3::Z);
    for order in [
        vec![instance(&red), instance(&blue)],
        vec![instance(&blue), instance(&red)],
    ] {
        let p = draw(&gpu, &mut pipeline, &store, &order, 64, &light, false)
            .get_pixel(31, 27)
            .0;
        assert!(p[0] > 180 && p[2] < 5, "depth red front: {p:?}");
    }
    let p = draw(
        &gpu,
        &mut pipeline,
        &store,
        &[instance(&blue)],
        64,
        &light,
        false,
    )
    .get_pixel(31, 27)
    .0;
    assert!(p[2] > 160 && p[0] < 5, "remove front exposes blue: {p:?}");
    let image = solid([160, 120, 80]);
    let mut maximum = 0;
    let mut dark = None;
    let mut bright = None;
    for (name, double, reverse, direction, lit) in [
        ("single-back", false, true, -Vec3::Z, 0.),
        ("double-back-lit", true, true, -Vec3::Z, 0.75),
        ("double-back-dark", true, true, Vec3::Z, 0.25),
        ("single-front-lit", false, false, Vec3::Z, 0.75),
    ] {
        let model = quad(
            name,
            0.5,
            image.clone(),
            sampling,
            [0., 0., 1., 1.],
            double,
            reverse,
        );
        store.get_or_upload(&gpu, &model, 0).unwrap();
        let light = lighting(direction);
        let p = draw(
            &gpu,
            &mut pipeline,
            &store,
            &[instance(&model)],
            64,
            &light,
            false,
        )
        .get_pixel(31, 27)
        .0;
        let expect = if lit == 0. {
            [0; 3]
        } else {
            expected(
                sample(&image, [0.5, 0.5], sampling, 9729),
                model.meshes[0].material.base_color,
                &light,
                lit,
            )
        };
        let e = error(p, expect);
        maximum = maximum.max(e);
        assert!(e <= 2, "{name}: {p:?} != {expect:?}");
        if name == "double-back-lit" {
            bright = Some(p);
        }
        if name == "double-back-dark" {
            dark = Some(p);
        }
    }
    assert!(
        bright.unwrap()[0] > dark.unwrap()[0] + 20,
        "back normal must flip with facing"
    );
    report(
        "depth-and-facing",
        serde_json::json!({"draw_order_controls":2,"reveal_controls":1,"facing_cases":4,"max_error":maximum,"tolerance":2}),
    );
}

#[test]
fn double_sided_shadow_culling_and_shadow_disable_controls_use_the_same_geometry() {
    let gpu = GpuDevice::new().unwrap();
    let mut pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap();
    let mut store = MeshStore::default();
    let image = solid([255; 3]);
    let mut receiver = quad(
        "receiver",
        0.25,
        image.clone(),
        Sampling::default(),
        [0., 0., 1., 1.],
        false,
        false,
    );
    receiver.meshes[0].material.base_color = [0.5, 0.5, 0.5, 1.];
    store.get_or_upload(&gpu, &receiver, 0).unwrap();
    let mut light = lighting(Vec3::Z);
    light.color = [1.; 3];
    light.shadow_vp = Mat4::from_cols_array(&[
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., -1., 0., 0., 0., 1., 1.,
    ]);
    let mut dark_pixel = None;
    let mut bright_pixel = None;
    for (name, double, reverse, shadow_expected) in [
        ("back-single", false, true, false),
        ("back-double", true, true, true),
        ("front-single", false, false, true),
    ] {
        let mut blocker = quad(
            name,
            0.75,
            image.clone(),
            Sampling::default(),
            [0., 0., 1., 1.],
            double,
            reverse,
        );
        for p in &mut blocker.meshes[0].positions {
            p[0] *= 0.3;
            p[1] *= 0.3;
        }
        store.get_or_upload(&gpu, &blocker, 0).unwrap();
        let instances = [instance(&receiver), instance(&blocker)];
        let pixels = draw(&gpu, &mut pipeline, &store, &instances, 128, &light, true);
        let centre = pixels.get_pixel(64, 64).0;
        let outside = pixels.get_pixel(20, 20).0;
        let lit = if shadow_expected { 0.25 } else { 0.75 };
        let expect = expected([1.; 3], receiver.meshes[0].material.base_color, &light, lit);
        assert!(
            error(centre, expect) <= 2,
            "{name}: centre {centre:?}, expected {expect:?}"
        );
        assert!(outside[0] > 150, "outside is illuminated: {outside:?}");
        if name == "back-double" {
            dark_pixel = Some(centre);
            pipeline.set_shadow_sabotage(true);
            let control = draw(&gpu, &mut pipeline, &store, &instances, 128, &light, true)
                .get_pixel(64, 64)
                .0;
            assert!(
                control[0] > centre[0] + 40,
                "disabling real shadow must fail shadow oracle: {control:?} vs {centre:?}"
            );
            pipeline.set_shadow_sabotage(false);
            assert_eq!(
                draw(&gpu, &mut pipeline, &store, &instances, 128, &light, true)
                    .get_pixel(64, 64)
                    .0,
                centre
            );
        } else if !shadow_expected {
            bright_pixel = Some(centre);
        }
    }
    assert!(bright_pixel.unwrap()[0] > dark_pixel.unwrap()[0] + 40);
    report(
        "shadow",
        serde_json::json!({"culling_cases":3,"disable_control_rejected":true,"restore_control_passed":true,"shadow_pixel":dark_pixel,"lit_pixel":bright_pixel}),
    );
}
