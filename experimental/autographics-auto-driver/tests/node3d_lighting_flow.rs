//! PLAN-080 T-03 permanent case: neutral light service through the frozen
//! typed port, verified on a real GPU with independent CPU oracles.
//!
//! Covers the neutral slice of G3-C01..C06 (V-01 声明层、V-02 数值、V-03
//! 阴影、V-05 准入、V-06 寿命) that does NOT require the 979/981 A slices;
//! the author-chain consumption (Node 声明 → 979 → 981 AV/A2R) lands in
//! T-04 when those A slices are published.
//!
//! Run: cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml
//!      --test node3d_lighting_flow -- --test-threads=1
use agp::{
    assets::{MaterialDef, MeshData, ModelData},
    render3d::{LightParams, MeshInstance, MeshPipeline, MeshStore},
    renderer::{
        capture::capture_png,
        device::{DeviceToken, GpuDevice},
        target::OffscreenTarget,
    },
};
use glam::{Mat4, Vec3};
use std::time::Instant;

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

/// Independent CPU Lambert oracle on stable interior pixels (059 §9).
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
    size: u32,
) -> Vec<u8> {
    let target = OffscreenTarget::new(g, size, size).unwrap();
    let before = p.uploads.snapshot();
    p.render(
        g,
        target.view(),
        size,
        size,
        [0., 0., 0., 1.],
        &Mat4::IDENTITY,
        light,
        s,
        ins,
        shadows,
    );
    g.wait_idle();
    let delta = p.uploads.snapshot().delta_since(&before);
    // 64 camera + 160 light + instance bytes (059 §9 constant per frame).
    assert_eq!(delta.buffer_rewrites, 3);
    assert_eq!(delta.bytes_rewritten, 224 + 64 * ins.len() as u64);
    capture_png(g, &target).unwrap().1.into_raw()
}

fn px(img: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * size + x) * 4) as usize;
    [img[at], img[at + 1], img[at + 2], img[at + 3]]
}

fn light(dir: Vec3, energy: f32, shadow: bool) -> agp::render3d::LightDef {
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
fn v02_energy_direction_and_normals_match_the_cpu_oracle() {
    let g = GpuDevice::new().unwrap();
    let mut store = MeshStore::default();
    let recv = quad("receiver", 0.95, 0.3);
    store.get_or_upload(&g, &recv, 0).unwrap();
    let mut p = MeshPipeline::new(&g, OffscreenTarget::FORMAT).unwrap();
    let r = MeshInstance {
        mesh: ("receiver".into(), "quad".into()),
        model: Mat4::IDENTITY,
    };

    // energy=1 正面光：lit = 0.25(ambient) + 1.0 = 1.25。
    let l = light(Vec3::Z, 1.0, false);
    expect_pixel(px(&draw(&g, &mut p, &store, &[r.clone()], &l, false, 128), 128, 83, 64), 1.25);
    // energy=0：只剩 ambient 0.25。
    let l0 = light(Vec3::Z, 0.0, false);
    expect_pixel(px(&draw(&g, &mut p, &store, &[r.clone()], &l0, false, 128), 128, 83, 64), 0.25);
    // 方向反转（-Z 对 +Z 面法线无贡献）。
    let rev = light(-Vec3::Z, 0.5, false);
    expect_pixel(px(&draw(&g, &mut p, &store, &[r.clone()], &rev, false, 128), 128, 83, 64), 0.25);
    // 斜向 45°：ndl = cos45。
    let d45 = light(Vec3::new(-1., 0., 1.).normalize(), 0.5, false);
    expect_pixel(
        px(&draw(&g, &mut p, &store, &[r.clone()], &d45, false, 128), 128, 83, 64),
        0.25 + 0.5 / 2f32.sqrt(),
    );

    // 非均匀缩放：独立逆对角法线 oracle（059 §9：法线 (1,0,1) 缩放 (2,1,.5)）。
    let mut tilted = quad("tilted", 0.8, 0.4);
    for n in &mut tilted.meshes[0].normals {
        *n = [1., 0., 1.];
    }
    store.get_or_upload(&g, &tilted, 0).unwrap();
    let ti = MeshInstance {
        mesh: ("tilted".into(), "quad".into()),
        model: Mat4::from_scale(Vec3::new(2., 1., 0.5)),
    };
    let lt = light(Vec3::Z, 0.5, false);
    expect_pixel(
        px(&draw(&g, &mut p, &store, &[ti], &lt, false, 128), 128, 83, 64),
        0.25 + 0.5 * 2. / (4.25f32).sqrt(),
    );
}

#[test]
fn v03_shadow_on_off_move_and_negative_control_are_distinguishable() {
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
    let sh = shadow_light(Vec3::new(-1., 0., 1.).normalize(), 0.5);
    let lit = 0.25 + 0.5 / 2f32.sqrt();

    // 遮挡：受影点 (83,64) 被 caster 遮住；旁点 (25,64) 亮。
    let on = draw(&g, &mut p, &store, &[r.clone(), c.clone()], &sh, true, 128);
    expect_pixel(px(&on, 128, 83, 64), 0.25);
    expect_pixel(px(&on, 128, 25, 64), lit);
    // shadow 关闭：全亮（正控）。
    let off = draw(&g, &mut p, &store, &[r.clone(), c.clone()], &sh, false, 128);
    expect_pixel(px(&off, 128, 83, 64), lit);
    // 遮挡物移开：受影点恢复亮（正控）。
    let moved = MeshInstance {
        model: Mat4::from_translation(Vec3::new(0.7, 0., 0.)),
        ..c
    };
    let im = draw(&g, &mut p, &store, &[r.clone(), moved], &sh, true, 128);
    expect_pixel(px(&im, 128, 83, 64), lit);
}

#[test]
fn v05_illegal_light_is_rejected_before_gpu_and_last_good_kept() {
    // 第二灯 / 超域 / 坏数值都必须在 GPU 上传前被 typed 端口拒绝。
    let g = GpuDevice::new().unwrap();
    let mut store = MeshStore::default();
    let recv = quad("receiver", 0.95, 0.3);
    store.get_or_upload(&g, &recv, 0).unwrap();
    let mut p = MeshPipeline::new(&g, OffscreenTarget::FORMAT).unwrap();
    let r = MeshInstance {
        mesh: ("receiver".into(), "quad".into()),
        model: Mat4::IDENTITY,
    };
    let good = light(Vec3::Z, 0.5, false);
    let im0 = draw(&g, &mut p, &store, &[r.clone()], &good, false, 128);
    expect_pixel(px(&im0, 128, 83, 64), 0.75);

    for (label, params) in [
        ("bad_energy", LightParams {
            dir_to_light: Vec3::Z,
            color: [1., 1., 1.],
            energy: 1.5,
            shadow_enabled: false,
            center: Vec3::ZERO,
            radius: 1.0,
            near: 0.0,
            far: 1.0,
            bias: 0.0005,
        }),
        ("bad_radius", LightParams {
            dir_to_light: Vec3::Z,
            color: [1., 1., 1.],
            energy: 0.5,
            shadow_enabled: false,
            center: Vec3::ZERO,
            radius: 0.0,
            near: 0.0,
            far: 1.0,
            bias: 0.0005,
        }),
        ("bad_bias", LightParams {
            dir_to_light: Vec3::Z,
            color: [1., 1., 1.],
            energy: 0.5,
            shadow_enabled: false,
            center: Vec3::ZERO,
            radius: 1.0,
            near: 0.0,
            far: 1.0,
            bias: 0.02,
        }),
        ("zero_direction", LightParams {
            dir_to_light: Vec3::ZERO,
            color: [1., 1., 1.],
            energy: 0.5,
            shadow_enabled: false,
            center: Vec3::ZERO,
            radius: 1.0,
            near: 0.0,
            far: 1.0,
            bias: 0.0005,
        }),
    ] {
        assert!(params.admit().is_err(), "{label} must be rejected");
    }

    // 同一受检灯重复渲染像素不变（last-good 语义）。
    let re = light(Vec3::Z, 0.5, false);
    let im1 = draw(&g, &mut p, &store, &[r.clone()], &re, false, 128);
    let im2 = draw(&g, &mut p, &store, &[r.clone()], &re, false, 128);
    assert_eq!(px(&im1, 128, 83, 64), px(&im2, 128, 83, 64));
}

#[test]
fn v06_twenty_rounds_resize_zero_size_and_device_epoch() {
    let g = GpuDevice::new().unwrap();
    let epoch: DeviceToken = g.token();
    let mut store = MeshStore::default();
    let recv = quad("receiver", 0.95, 0.3);
    store.get_or_upload(&g, &recv, 0).unwrap();
    let r = MeshInstance {
        mesh: ("receiver".into(), "quad".into()),
        model: Mat4::IDENTITY,
    };
    let l = light(Vec3::Z, 0.5, false);
    let start = Instant::now();
    for i in 0..20 {
        let mut p = MeshPipeline::new(&g, OffscreenTarget::FORMAT).unwrap();
        // device epoch 必须与管道一致（旧 epoch 拒绝）。
        assert_eq!(p.device_ordinal, Some(epoch.ordinal));
        // 零尺寸拒绝且不破坏后续合法目标。
        assert!(OffscreenTarget::new(&g, 0, 128).is_err());
        for size in [64u32, 128u32] {
            let im = draw(&g, &mut p, &store, &[r.clone()], &l, false, size);
            expect_pixel(px(&im, size, size / 3, size / 2), 0.75);
        }
        drop(p);
        g.wait_idle();
    }
    assert!(start.elapsed().as_secs() < 120, "20 rounds too slow");
}
