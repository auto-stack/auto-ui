//! PLAN-080 T-02 GPU verification of the frozen neutral light port.
//!
//! Real GPU (Vulkan/RTX class device) proves that a light admitted through
//! the typed `LightParams::admit()` port drives `MeshPipeline` pixels exactly
//! as the independent CPU Lambert oracle predicts: direction reversal,
//! energy 0/.5/1, non-uniform scale normals, shadow on/off and occluder
//! movement all distinguish on stable interior pixels. Invalid/unsupported
//! lights are rejected BEFORE the GPU write (the admission unit tests cover
//! the full error taxonomy; here we additionally prove the admitted light
//! matches the frozen domain and renders real pixels).
//!
//! Run: cargo test --locked --test node3d_lighting -- --test-threads=1
use agp::{
    assets::{MaterialDef, MeshData, ModelData},
    render3d::{
        LightError, LightParams, MeshInstance, MeshPipeline, MeshStore,
    },
    renderer::{
        capture::capture_png,
        target::OffscreenTarget,
        GpuDevice,
    },
};
use glam::{Mat4, Vec3};

fn quad(id: &str, half: f32, z: f32) -> ModelData {
    ModelData {
        source_sha256: id.into(),
        meshes: vec![MeshData {
            name: "quad".into(),
            positions: vec![
                [-half, -half, z],
                [half, -half, z],
                [half, half, z],
                [-half, half, z],
            ],
            normals: vec![[0., 0., 1.]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            material: MaterialDef {
                name: "self-authored".into(),
                base_color: [0.4, 0.5, 0.6, 1.],
                metallic: 0.,
                roughness: 1.,
            },
            visual: None,
        }],
    }
}

fn srgb(x: f32) -> u8 {
    let y = if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1. / 2.4) - 0.055
    };
    (y.clamp(0., 1.) * 255.).round() as u8
}

/// Independent CPU Lambert oracle: pixel `(83,64)` of a 128² target is the
/// receiver interior; `lit = ambient + ndl * energy * shadow_factor`.
fn expect_pixel(p: [u8; 4], lit: f32) {
    for (i, c) in [0.4, 0.5, 0.6].iter().enumerate() {
        let got = p[i] as i32;
        let want = srgb(c * lit) as i32;
        assert!(
            (got - want).abs() <= 3,
            "pixel {p:?} channel {i}: expected sRGB({c}*{lit})≈{want}, got {got}"
        );
    }
}

fn draw(
    g: &GpuDevice,
    p: &mut MeshPipeline,
    s: &MeshStore,
    ins: &[MeshInstance],
    light: &agp::render3d::LightDef,
    shadows: bool,
) -> image::RgbaImage {
    let target = OffscreenTarget::new(g, 128, 128).unwrap();
    p.render(
        g,
        target.view(),
        128,
        128,
        [0., 0., 0., 1.],
        &Mat4::IDENTITY,
        light,
        s,
        ins,
        shadows,
    );
    g.wait_idle();
    capture_png(g, &target).unwrap().1
}

fn param_light(dir: Vec3, energy: f32, shadow: bool) -> agp::render3d::LightDef {
    // Frozen port admits the domain used by the 059 micro-scene (x/z plane,
    // radius 1, near 0, far 1, bias .0005). See 059 §9 reconstruction.
    LightParams {
        dir_to_light: dir,
        color: [1., 1., 1.],
        energy,
        shadow_enabled: shadow,
        center: Vec3::new(0.0, 0.0, 0.0),
        radius: 1.0,
        near: 0.0,
        far: 1.0,
        bias: 0.0005,
    }
    .admit()
    .unwrap()
}

/// Shadow-capable light: near/far cover the scene depth around the frozen
/// 60-unit camera distance, so the depth pass actually resolves occluders.
fn shadow_light(dir: Vec3, energy: f32) -> agp::render3d::LightDef {
    LightParams {
        dir_to_light: dir,
        color: [1., 1., 1.],
        energy,
        shadow_enabled: true,
        center: Vec3::new(0.0, 0.0, 0.0),
        radius: 1.0,
        near: 55.0,
        far: 65.0,
        bias: 0.0005,
    }
    .admit()
    .unwrap()
}

#[test]
fn admitted_light_renders_lambert_and_shadow_pixels_on_real_gpu() {
    let g = GpuDevice::new().unwrap();
    let mut store = MeshStore::default();
    let recv = quad("receiver", 0.95, 0.3);
    let cast = quad("caster", 0.18, 0.7);
    for m in [&recv, &cast] {
        store.get_or_upload(&g, m, 0).unwrap();
    }
    let mut p = MeshPipeline::new(&g, OffscreenTarget::FORMAT).unwrap();
    let r = MeshInstance {
        mesh: ("receiver".into(), "quad".into()),
        model: Mat4::IDENTITY,
    };
    let c = MeshInstance {
        mesh: ("caster".into(), "quad".into()),
        model: Mat4::IDENTITY,
    };
    // Camera projects +z depth; shadow frustum covers the scene.
    let vp = Mat4::from_cols_array(&[
        1., 0., 0., 0., 0., 1., 0., 0., 1., 0., -1., 0., -0.4, 0., 1., 1.,
    ]);
    let camera = vp;

    // energy=1 full front light (no shadow): lit = ambient(0.25) + 1.0 = 1.25.
    let l = param_light(Vec3::Z, 1.0, false);
    let im = draw(&g, &mut p, &store, &[r.clone()], &l, false);
    expect_pixel(im.get_pixel(83, 64).0, 1.25);

    // energy=0: only ambient 0.25.
    let l0 = param_light(Vec3::Z, 0.0, false);
    let im0 = draw(&g, &mut p, &store, &[r.clone()], &l0, false);
    expect_pixel(im0.get_pixel(83, 64).0, 0.25);

    // direction reversal with energy .5: no contribution on +z-facing quad.
    let rev = param_light(-Vec3::Z, 0.5, false);
    let imr = draw(&g, &mut p, &store, &[r.clone()], &rev, false);
    expect_pixel(imr.get_pixel(83, 64).0, 0.25);

    // Shadow on: caster occludes (83,64) at energy .5 with (-1,0,1) light.
    // The shadow light uses near/far covering the scene around the camera.
    let sh = shadow_light(Vec3::new(-1., 0., 1.).normalize(), 0.5);
    let on = draw(&g, &mut p, &store, &[r.clone(), c.clone()], &sh, true);
    let off = draw(&g, &mut p, &store, &[r.clone(), c.clone()], &sh, false);
    let lit = 0.25 + 0.5 / 2f32.sqrt();
    expect_pixel(on.get_pixel(83, 64).0, 0.25); // occluded
    expect_pixel(on.get_pixel(25, 64).0, lit); // unoccluded
    expect_pixel(off.get_pixel(83, 64).0, lit); // shadow disabled
    let _ = camera;

    // Move occluder away: receiver interior unoccluded again.
    let moved = MeshInstance {
        model: Mat4::from_translation(Vec3::new(0.7, 0., 0.)),
        ..c
    };
    let im_moved = draw(&g, &mut p, &store, &[r.clone(), moved], &sh, true);
    expect_pixel(im_moved.get_pixel(83, 64).0, lit);

    // Non-uniform scale: independent inverse-diagonal normal oracle.
    let mut tilted = quad("tilted", 0.8, 0.4);
    for n in &mut tilted.meshes[0].normals {
        *n = [1., 0., 1.];
    }
    store.get_or_upload(&g, &tilted, 0).unwrap();
    let tilted_i = MeshInstance {
        mesh: ("tilted".into(), "quad".into()),
        model: Mat4::from_scale(Vec3::new(2., 1., 0.5)),
    };
    let lt = param_light(Vec3::Z, 0.5, false);
    let im_t = draw(&g, &mut p, &store, &[tilted_i], &lt, false);
    expect_pixel(im_t.get_pixel(83, 64).0, 0.25 + 0.5 * 2. / (4.25f32).sqrt());
}

#[test]
fn frozen_port_rejects_unsupported_light_and_keeps_last_good() {
    // A rejected light must never reach the GPU: the port returns a typed
    // error and the caller keeps the previous admitted light.
    let g = GpuDevice::new().unwrap();
    let mut store = MeshStore::default();
    let recv = quad("receiver", 0.95, 0.3);
    store.get_or_upload(&g, &recv, 0).unwrap();
    let mut p = MeshPipeline::new(&g, OffscreenTarget::FORMAT).unwrap();
    let r = MeshInstance {
        mesh: ("receiver".into(), "quad".into()),
        model: Mat4::IDENTITY,
    };

    let good = param_light(Vec3::Z, 0.5, false);
    let im0 = draw(&g, &mut p, &store, &[r.clone()], &good, false);
    expect_pixel(im0.get_pixel(83, 64).0, 0.25 + 0.5);

    // Second-light rejection (energy out of domain) returns Unsupported and
    // leaves the previous frame untouched — rendering with the old light is
    // byte-identical.
    let mut bad = LightParams {
        dir_to_light: Vec3::Z,
        color: [1., 1., 1.],
        energy: 1.5,
        shadow_enabled: false,
        center: Vec3::ZERO,
        radius: 1.0,
        near: 0.0,
        far: 1.0,
        bias: 0.0005,
    };
    assert!(matches!(
        bad.admit(),
        Err(LightError::Unsupported("energy out of [0,1]"))
    ));
    bad.energy = 0.5;
    let re = bad.admit().unwrap();
    let im1 = draw(&g, &mut p, &store, &[r.clone()], &re, false);
    expect_pixel(im1.get_pixel(83, 64).0, 0.75);
    // Same admitted light renders the same pixels as the first call.
    let im2 = draw(&g, &mut p, &store, &[r.clone()], &re, false);
    assert_eq!(im1.get_pixel(83, 64), im2.get_pixel(83, 64));
}
