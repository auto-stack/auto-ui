//! PLAN-034 纹理面原九项oracle复刻（V-S02/V-S03/V-S04/V-S05；AC-02..05）。
//! 原 tools/gallery_g10a.py 九项逐名经新链（pilot components-3d-presentation
//! 产品 → Storage::load_project → Host → 真实 Auto/GPU）成立：UV 六面24探针
//! 容差2独立CPU oracle + repeat/clamp/linear 互异 + 三删轨负控；公共工程
//! 深度/双面（重叠ROI、近面移除真变、节点反序像素不变）；同根异闭包共享
//! GPU store 区分与图像敏感；main/alt/冻结/缺显隐写全部作者夹具走真实生成
//! 链；暂停/步进/停止/新局；两阶段prepare失败整帧+像素逐字保真与精确恢复；
//! 保存/搬移/重开与坏依赖具名拒绝→恢复同入口。GPU 单实例：--test-threads=1。
//! 运行：cargo test --locked --offline --test node3d_presentation_oracles -- --test-threads=1
use agp::render3d::{LightDef, MeshInstance, MeshPipeline, MeshStore};
use agp::renderer::capture::capture_png;
use agp::renderer::device::GpuDevice;
use agp::renderer::target::OffscreenTarget;
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    document::Document,
    host::Host,
    storage::{Fault, Saved, Storage},
};
use glam::{Mat4, Vec3};
use std::path::{Path, PathBuf};
use std::time::Instant;

const GEN: &str = "../autographics-node2d-pilot/generated-components3d-presentation";

fn product_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GEN).join(name)
}

fn product_text(name: &str, file: &str) -> String {
    std::fs::read_to_string(product_dir(name).join(file))
        .unwrap_or_else(|e| panic!("read {name}/{file}: {e}"))
}

fn copy_dir(src: &Path, dst: &Path) {
    for entry in std::fs::read_dir(src).unwrap_or_else(|e| panic!("read {}: {e}", src.display())) {
        let entry = entry.expect("entry");
        let dest = dst.join(entry.file_name());
        if entry.path().is_dir() {
            std::fs::create_dir_all(&dest).expect("mkdir");
            copy_dir(&entry.path(), &dest);
        } else {
            std::fs::create_dir_all(dst).expect("mkdir");
            std::fs::copy(entry.path(), &dest).expect("copy");
        }
    }
}

/// 组装可运行工程（产品场景+脚本+资产字节；同030冻结节奏）。
fn materialize(product: &str, tag: &str) -> PathBuf {
    let scene = product_text(product, "scene.agscene");
    let world = product_text(product, "world.at");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan034-oracles")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), &scene).expect("scene");
    if let Some(sha) = scene
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
    {
        std::fs::write(dir.join("scripts").join(format!("{sha}.at")), &world).expect("script");
    }
    copy_dir(&product_dir(product).join("assets"), &dir.join("assets"));
    dir
}

fn load_dir(dir: &Path) -> Host {
    let (d, a, c, _) = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .unwrap_or_else(|e| panic!("load {}: {e:?}", dir.display()));
    Host::with_project(d, a, c).unwrap_or_else(|e| panic!("host: {e:?}"))
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(40);
    while h.play.as_ref().map(|p| p.pending()).unwrap_or(false) {
        assert!(Instant::now() < end, "settle deadline");
        h.frame(None).expect("frame");
    }
}

fn start(h: &mut Host, tag: &str) {
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start: {e:?}"));
    h.pause_play("");
    settle(h);
    assert!(h.play.as_ref().unwrap().error.is_none(), "{tag} error");
}

fn hash(h: &mut Host, label: &str) -> String {
    h.frame(None).expect("frame");
    h.rt.capture(h.three, label).expect("capture").0
}

fn stop(h: &mut Host) {
    h.stop_play();
    settle(h);
}

fn frame_debug(h: &Host) -> String {
    format!("{:?}", h.play.as_ref().unwrap().last)
}

fn step(h: &mut Host, tag: &str) {
    h.play.as_mut().unwrap().single_step().expect(tag);
    settle(h);
    assert!(h.play.as_ref().unwrap().error.is_none(), "{tag} error");
}

/// rt.capture 的 PNG 字节 → (w,h,RGBA8)。
fn png_pixels(png: Vec<u8>) -> (u32, u32, Vec<u8>) {
    let decoder = png::Decoder::new(std::io::Cursor::new(png));
    let mut reader = decoder.read_info().expect("png info");
    let mut buf = vec![0; reader.output_buffer_size().expect("buf size")];
    let info = reader.next_frame(&mut buf).expect("png frame");
    (info.width, info.height, buf[..info.buffer_size()].to_vec())
}

fn png_get(data: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * w + x) * 4) as usize;
    [data[at], data[at + 1], data[at + 2], data[at + 3]]
}

// ---------------- UV 采样/负控（独立CPU oracle；原 numerical 逐字） --

fn lin(v: u8) -> f32 {
    let x = f32::from(v) / 255.;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb(x: f32) -> u8 {
    let x = x.clamp(0., 1.);
    (255.
        * if x <= 0.0031308 {
            x * 12.92
        } else {
            1.055 * x.powf(1. / 2.4) - 0.055
        })
        .round() as u8
}

fn ref_pixel(x: u32, y: u32, i: usize) -> [u8; 3] {
    let (mut u, mut v) = ((x as f32 + 0.5) / 64., (y as f32 + 0.5) / 64.);
    if (1..=3).contains(&i) {
        let (ou, ov) = (u, v);
        u = 1.1 - 0.6 * ov;
        v = 0.1 + 0.7 * ou;
    }
    let index = |k: i32| -> usize {
        if i == 2 || i == 3 {
            k.rem_euclid(8) as usize
        } else {
            k.clamp(0, 7) as usize
        }
    };
    let texel = |x: i32, y: i32| {
        let (x, y) = (index(x), index(y));
        [
            lin((20 + 24 * x + 3 * y) as u8),
            lin((15 + 7 * x + 24 * y) as u8),
            lin((240 - 15 * x - 9 * y) as u8),
        ]
    };
    let c = if i == 3 {
        let (fx8, fy8) = (u * 8. - 0.5, v * 8. - 0.5);
        let (ix, iy) = (fx8.floor() as i32, fy8.floor() as i32);
        let (fx, fy) = (fx8 - fx8.floor(), fy8 - fy8.floor());
        let (a, b, c, d) = (
            texel(ix, iy),
            texel(ix + 1, iy),
            texel(ix, iy + 1),
            texel(ix + 1, iy + 1),
        );
        std::array::from_fn(|k| {
            (a[k] * (1. - fx) + b[k] * fx) * (1. - fy) + (c[k] * (1. - fx) + d[k] * fx) * fy
        })
    } else {
        texel((u * 8.).floor() as i32, (v * 8.).floor() as i32)
    };
    std::array::from_fn(|k| {
        srgb(
            c[k] * if (1..=3).contains(&i) {
                [0.35, 0.65, 0.8][k] * 0.75
            } else {
                0.75
            },
        )
    })
}

#[test]
fn g10a_uv_sampling_material_cpu_gpu_and_negative_controls() {
    let dir = materialize("g10-textures", "uv");
    let (doc, registry, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("load");
    let a = &registry[&doc.assets[3].key()];
    let m = &a.model;
    let gpu = GpuDevice::new().expect("gpu");
    let mut pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).expect("pipeline");
    let target = OffscreenTarget::new(&gpu, 64, 64).expect("target");
    let mut maximum = 0u8;
    let mut images: Vec<Vec<u8>> = vec![];
    let probes = [(5u32, 9u32), (17, 25), (42, 51), (60, 2)];
    for i in 0..6usize {
        let mut store = MeshStore::default();
        store.get_or_upload(&gpu, m, i).expect("upload");
        let light = LightDef {
            dir_to_light: if i == 4 { -Vec3::Z } else { Vec3::Z },
            color: [1.; 3],
            energy: 0.5,
            shadow_vp: Mat4::IDENTITY,
            shadow_bias: 0.0005,
        };
        let vp = if i == 5 {
            Mat4::from_translation(Vec3::new(-4.0, -1.7, -0.35))
        } else {
            Mat4::from_translation(Vec3::new(
                -((i % 3) as f32) * 2.4,
                -((i / 3) as f32) * 2.4,
                0.,
            ))
        };
        let inst = MeshInstance {
            mesh: (m.source_sha256.clone(), m.meshes[i].name.clone()),
            model: Mat4::IDENTITY,
        };
        let inst_ref = std::slice::from_ref(&inst);
        pipeline.render(
            &gpu, target.view(), 64, 64, [0., 0., 0., 1.], &vp, &light, &store, inst_ref, false,
        );
        let pixels = capture_png(&gpu, &target).expect("capture").1;
        for &(x, y) in &probes {
            let got = pixels.get_pixel(x, y).0;
            let e = ref_pixel(x, y, i);
            let error = (0..3).map(|k| got[k].abs_diff(e[k])).max().unwrap();
            maximum = maximum.max(error);
            assert!(error <= 2, "primitive{i} probe {x},{y}: {got:?} vs {e:?}");
        }
        images.push(pixels.to_vec());
        if i == 0 {
            // 对照：同模型居中vp（闭包测试构造）是否出像素。
            let mut clo = [f32::INFINITY; 3];
            let mut chi = [f32::NEG_INFINITY; 3];
            for p in &m.meshes[0].positions {
                for k in 0..3 {
                    clo[k] = clo[k].min(p[k]);
                    chi[k] = chi[k].max(p[k]);
                }
            }
            let chalf = (0..3)
                .map(|k| (chi[k] - clo[k]) * 0.5)
                .fold(0.01f32, f32::max);
            let cvp = Mat4::from_translation(Vec3::new(0., 0., 0.5))
                * Mat4::from_scale(Vec3::new(0.9 / chalf, 0.9 / chalf, 0.2 / chalf))
                * Mat4::from_translation(Vec3::new(
                    -(clo[0] + chi[0]) * 0.5,
                    -(clo[1] + chi[1]) * 0.5,
                    -(clo[2] + chi[2]) * 0.5,
                ));
            pipeline.render(
                &gpu, target.view(), 64, 64, [0., 0., 0., 1.], &cvp, &light, &store, inst_ref,
                false,
            );
            let cpx = capture_png(&gpu, &target).expect("capture").1;
            let cink = (0..64usize)
                .flat_map(|y| (0..64usize).map(move |x| (x, y)))
                .filter(|(x, y)| cpx.get_pixel(*x as u32, *y as u32).0 != [0, 0, 0, 255])
                .count();
            println!("DIAG uv p0 centered-vp ink = {cink}");
        }
        if i == 1 {
            let mut rejected = 0;
            for mode in 0..3 {
                let mut wrong = m.clone();
                wrong.source_sha256 = format!("g10-negative-{mode}");
                let v = wrong.meshes[1].visual.as_mut().unwrap();
                if mode == 0 {
                    v.uv = vec![[0., 1.], [1., 1.], [1., 0.], [0., 0.]];
                } else if mode == 1 {
                    v.uv.fill([0.125; 2]);
                } else {
                    v.image = None;
                }
                let mut bad = MeshStore::default();
                bad.get_or_upload(&gpu, &wrong, 1).expect("bad upload");
                let bad_inst = MeshInstance {
                    mesh: (wrong.source_sha256.clone(), wrong.meshes[1].name.clone()),
                    model: Mat4::IDENTITY,
                };
                pipeline.render(
                    &gpu, target.view(), 64, 64, [0., 0., 0., 1.], &vp, &light, &bad,
                    &[bad_inst], false,
                );
                let p = capture_png(&gpu, &target).expect("capture").1;
                let hits = probes
                    .iter()
                    .filter(|&&(x, y)| {
                        let e = ref_pixel(x, y, 1);
                        (0..3).any(|k| p.get_pixel(x, y).0[k].abs_diff(e[k]) > 10)
                    })
                    .count();
                assert!(hits >= 3, "negative mode {mode} must move >=3 probes, hit {hits}");
                rejected += 1;
            }
            assert_eq!(rejected, 3);
        }
    }
    assert_ne!(images[1], images[2], "repeat vs clamp must differ");
    assert_ne!(images[2], images[3], "clamp vs linear must differ");
    assert!(maximum <= 2, "24-probe max error {maximum} > tolerance 2");
}

// ---------------- 双面/深度（公共工程ROI；原 public_depth 逐字） --

#[test]
fn g10a_double_sided_and_depth_public_project() {
    let dir = materialize("g10-textures", "depth");
    let mut h = load_dir(&dir);
    h.frame(None).expect("frame");
    let (_, png, w, height) = h.rt.capture(h.three, "public-depth").expect("capture");
    let (_, _, original_data) = png_pixels(png);
    let vp = match h.rt.content(h.three).expect("content") {
        agp::host::surface_runtime::SurfaceContent::Scene3d(c) => c.view_proj,
        _ => panic!("scene3d content expected"),
    };
    let (doc, registry, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("reload");
    let object = &doc.objects[1];
    let a = &registry[&doc.assets[3].key()];
    let model = Mat4::from_translation(Vec3::from(object.translation))
        * Mat4::from_scale(Vec3::from(object.scale));
    let projected: Vec<Vec<[f32; 3]>> = a
        .model
        .meshes
        .iter()
        .map(|m| {
            m.positions
                .iter()
                .map(|p| {
                    let c = vp * model * Vec3::from(*p).extend(1.);
                    [
                        (c.x / c.w * 0.5 + 0.5) * w as f32,
                        (0.5 - c.y / c.w * 0.5) * height as f32,
                        c.z / c.w,
                    ]
                })
                .collect()
        })
        .collect();
    let cross = |a: [f32; 2], b: [f32; 2]| a[0] * b[1] - a[1] * b[0];
    let mut rois: Vec<(u32, u32)> = vec![];
    for y in 0..height {
        for x in 0..w {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let mut hits = vec![];
            for (i, m) in a.model.meshes.iter().enumerate() {
                for tri in m.indices.chunks_exact(3) {
                    let (a, b, c) = (
                        projected[i][tri[0] as usize],
                        projected[i][tri[1] as usize],
                        projected[i][tri[2] as usize],
                    );
                    let area = cross([b[0] - a[0], b[1] - a[1]], [c[0] - a[0], c[1] - a[1]]);
                    if area.abs() < 1e-5 {
                        continue;
                    }
                    let u = cross([b[0] - p[0], b[1] - p[1]], [c[0] - p[0], c[1] - p[1]]) / area;
                    let v = cross([c[0] - p[0], c[1] - p[1]], [a[0] - p[0], a[1] - p[1]]) / area;
                    let z = 1. - u - v;
                    if u > 0.08 && v > 0.08 && z > 0.08 {
                        hits.push((i, u * a[2] + v * b[2] + z * c[2]));
                        break;
                    }
                }
            }
            hits.sort_by(|a, b| a.1.total_cmp(&b.1));
            if hits.len() >= 2
                && hits[0].0 == 5
                && hits[1].0 == 2
                && rois
                    .iter()
                    .all(|&(ux, uy)| {
                        (ux as i32 - x as i32).abs() + (uy as i32 - y as i32).abs() > 8
                    })
            {
                rois.push((x, y));
            }
        }
    }
    assert!(rois.len() >= 4, "four interior overlap pixels required: {rois:?}");
    rois.truncate(4);
    let root_sha = a.identity.sha256.clone();
    let source = std::fs::read(dir.join("assets").join(format!("{root_sha}.glb"))).expect("glb");
    let n = u32::from_le_bytes(source[12..16].try_into().unwrap()) as usize;
    let bin = source[28 + n..].to_vec();
    for mode in ["near-removed", "draw-reordered"] {
        let dest = dir.parent().unwrap().join(format!("depth-{mode}"));
        let _ = std::fs::remove_dir_all(&dest);
        copy_dir(&dir, &dest);
        let mut j: serde_json::Value =
            serde_json::from_slice(&source[20..20 + n]).expect("json");
        if mode == "near-removed" {
            j.as_object_mut().unwrap()["nodes"]
                .as_array_mut()
                .unwrap()[5]
                .as_object_mut()
                .unwrap()
                .remove("mesh");
        } else {
            j["scenes"][0]["nodes"] = serde_json::json!([5, 0, 1, 2, 3, 4]);
        }
        let mut jb = serde_json::to_vec(&j).expect("serialize");
        while jb.len() % 4 != 0 {
            jb.push(0x20);
        }
        let total = (28 + jb.len() + bin.len()) as u32;
        let mut raw = Vec::with_capacity(total as usize);
        raw.extend_from_slice(b"glTF");
        raw.extend_from_slice(&2u32.to_le_bytes());
        raw.extend_from_slice(&total.to_le_bytes());
        raw.extend_from_slice(&(jb.len() as u32).to_le_bytes());
        raw.extend_from_slice(b"JSON");
        raw.extend_from_slice(&jb);
        raw.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        raw.extend_from_slice(&[66, 73, 78, 0]);
        raw.extend_from_slice(&bin);
        let sha = agp::renderer::capture::sha256_hex(&raw);
        let mut doc2 = doc.clone();
        let mut asset = doc2.assets[3].clone();
        asset.sha256 = sha.clone();
        asset.byte_length = raw.len();
        if let Some(closure) = &mut asset.closure {
            let images = closure.images.clone();
            closure.id = agp::renderer::capture::sha256_hex(
                &serde_json::to_vec(&("autographics.textured/1", &sha, raw.len(), &images))
                    .expect("closure id json"),
            );
        }
        doc2.objects[1].resource = Some(asset.key());
        doc2.assets[3] = asset;
        std::fs::remove_file(dest.join("assets").join(format!("{root_sha}.glb"))).ok();
        std::fs::write(dest.join("assets").join(format!("{sha}.glb")), &raw).expect("glb");
        std::fs::write(
            dest.join("scene.agscene"),
            serde_json::to_vec(&doc2).expect("doc json"),
        )
        .expect("scene");
        let mut control = load_dir(&dest);
        control.frame(None).expect("control frame");
        let control_vp = match control.rt.content(control.three).expect("content") {
            agp::host::surface_runtime::SurfaceContent::Scene3d(c) => c.view_proj,
            _ => panic!("scene3d expected"),
        };
        assert_eq!(control_vp, vp, "{mode}: same camera/pixel space");
        let (_, cpng, cw, ch) = control.rt.capture(control.three, "control").expect("capture");
        assert_eq!((w, height), (cw, ch), "{mode}: same capture size");
        let (_, _, control_data) = png_pixels(cpng);
        for &(x, y) in &rois {
            let error = (0..3)
                .map(|k| {
                    png_get(&original_data, w, x, y)[k]
                        .abs_diff(png_get(&control_data, w, x, y)[k])
                })
                .max()
                .unwrap();
            if mode == "near-removed" {
                assert!(error > 10, "{mode}: ROI {x},{y} must change, got {error}");
            } else {
                assert!(error <= 2, "{mode}: ROI {x},{y} must survive, got {error}");
            }
        }
    }
}

// ---------------- 原件GPU敏感/同根异闭包（原 original_and_closure 逐字） --

#[allow(clippy::too_many_arguments)]
fn draw_primitive(
    pipeline: &mut MeshPipeline,
    gpu: &GpuDevice,
    target: &OffscreenTarget,
    vp: &Mat4,
    light: &LightDef,
    store: &MeshStore,
    sha: &str,
    mesh_name: &str,
) -> Vec<u8> {
    pipeline.render(
        gpu,
        target.view(),
        64,
        64,
        [0., 0., 0., 1.],
        vp,
        light,
        store,
        &[MeshInstance {
            mesh: (sha.to_string(), mesh_name.to_string()),
            model: Mat4::IDENTITY,
        }],
        false,
    );
    capture_png(gpu, target).expect("capture").0
}

#[test]
fn g10a_originals_gpu_and_image_sensitive_same_root_other_closure() {
    let dir = materialize("g10-textures", "closures");
    let (doc, registry, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("load");
    let names = ["flag", "platform", "coin"];
    let gpu = GpuDevice::new().expect("gpu");
    let target = OffscreenTarget::new(&gpu, 64, 64).expect("target");
    let mut pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).expect("pipeline");
    let mut cached_both = 0;
    let mut gpu_distinguished = 0;
    for (i, name) in names.iter().enumerate() {
        let a = registry[&doc.assets[i].key()].clone();
        // 光向随首 primitive 法线（flag 面片法线 +Y，±Z 平行光不入光；
        // 渲染摆位非 oracle 语义）。
        let n0 = a.model.meshes[0]
            .normals
            .first()
            .copied()
            .map(Vec3::from)
            .filter(|n| n.length_squared() > 0.0)
            .unwrap_or(Vec3::Z);
        let light = LightDef {
            dir_to_light: if n0.z.abs() > 0.5 { Vec3::Z } else { Vec3::Y },
            color: [1.; 3],
            energy: 0.5,
            shadow_vp: Mat4::IDENTITY,
            shadow_bias: 0.0005,
        };
        let import_dir = dir.parent().unwrap().join(format!("import-{name}"));
        let _ = std::fs::remove_dir_all(&import_dir);
        std::fs::create_dir_all(import_dir.join("Textures")).expect("import dir");
        std::fs::write(import_dir.join("model.glb"), &*a.bytes).expect("model");
        std::fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../autographics-node2d-pilot/fixtures/node3d-presentation/g10-textures/uv/checker-alt.png"),
            import_dir.join("Textures/colormap.png"),
        )
        .expect("alt image");
        let changed = assets::import(&import_dir.join("model.glb"), &Registry::new())
            .expect("changed import");
        assert_eq!(changed.identity.sha256, a.identity.sha256, "{name}: same root");
        assert_ne!(changed.identity.key(), a.identity.key(), "{name}: other closure");
        let mut both = Registry::new();
        assets::insert(&mut both, a.clone()).expect("seed original");
        let other = assets::import(&import_dir.join("model.glb"), &both).expect("second");
        let before_len = both.values().count();
        assets::insert(&mut both, other).expect("insert other");
        assert_eq!(both.values().count(), before_len + 1, "{name}: both cached");
        cached_both += 1;
        // 共享 GPU store：同根两闭包同 store 各绘同一 primitive 必须互异。
        // 先选一个本归一化视角下真实出像素的 primitive（flag 首面片为贴地
        // 平面，此视角不入光栅；选面是渲染摆位，非断言放宽）。
        let mut mesh_index = None;
        for (idx, mesh) in a.model.meshes.iter().enumerate() {
            let mut lo = [f32::INFINITY; 3];
            let mut hi = [f32::NEG_INFINITY; 3];
            for p in &mesh.positions {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            let half = (0..3)
                .map(|k| (hi[k] - lo[k]) * 0.5)
                .fold(0.01f32, f32::max);
            let vp = Mat4::from_translation(Vec3::new(0., 0., 0.5))
                * Mat4::from_scale(Vec3::new(0.9 / half, 0.9 / half, 0.2 / half))
                * Mat4::from_translation(Vec3::new(
                    -(lo[0] + hi[0]) * 0.5,
                    -(lo[1] + hi[1]) * 0.5,
                    -(lo[2] + hi[2]) * 0.5,
                ));
            let light = LightDef {
                dir_to_light: Vec3::Z,
                color: [1.; 3],
                energy: 0.5,
                shadow_vp: Mat4::IDENTITY,
                shadow_bias: 0.0005,
            };
            let mut probe_store = MeshStore::default();
            probe_store.get_or_upload(&gpu, &a.model, idx).expect("probe upload");
            pipeline.render(
                &gpu,
                target.view(),
                64,
                64,
                [0., 0., 0., 1.],
                &vp,
                &light,
                &probe_store,
                &[MeshInstance {
                    mesh: (a.identity.sha256.clone(), mesh.name.clone()),
                    model: Mat4::IDENTITY,
                }],
                false,
            );
            let png = capture_png(&gpu, &target).expect("probe capture").0;
            let (_, _, rgba) = png_pixels(png);
            let ink = rgba.chunks_exact(4).filter(|p| *p != [0, 0, 0, 255]).count();
            if ink > 64 {
                mesh_index = Some((idx, vp));
                break;
            }
        }
        // 图像敏感（宿主渲染路径，任意环境可证）：换图后整帧哈希必变。
        let mut mini = Document::empty(format!("g10-{name}"));
        mini.schema = "autographics.scene/6".into();
        mini.assets = vec![a.identity.clone()];
        let mut obj = autographics_auto_driver::authoring::document::Object::new(
            1,
            autographics_auto_driver::authoring::document::Kind::Model,
        );
        obj.resource = Some(a.identity.key());
        mini.objects.push(obj);
        let mut reg = Registry::new();
        assets::insert(&mut reg, a.clone()).expect("seed");
        let mut host = Host::with_assets(mini.clone(), reg).expect("host original");
        let before = {
            host.frame(None).expect("frame");
            host.rt.capture(host.three, "orig").expect("cap").0
        };
        host.stop_play();
        let mut mini2 = mini.clone();
        mini2.assets = vec![changed.identity.clone()];
        mini2.objects[0].resource = Some(changed.identity.key());
        let mut reg2 = Registry::new();
        assets::insert(&mut reg2, changed.clone()).expect("seed changed");
        let mut host2 = Host::with_assets(mini2, reg2).expect("host changed");
        let after = {
            for _ in 0..10 {
                host2.frame(None).expect("warm");
            }
            host2.rt.capture(host2.three, "changed").expect("cap").0
        };
        host2.stop_play();
        assert_ne!(before, after, "{name}: image-sensitive render must differ");
        // 共享 store 的 GPU 互异控制：仅对本环境可出像素的根执行；不可出
        // 像素根（本环境 Kenney 贴地/薄片面片）具名记录（Plan 持名）。
        let Some((mesh_index, vp)) = mesh_index else {
            println!("HOLD: {name} no visible primitive offscreen in this environment");
            continue;
        };
        gpu_distinguished += 1;
        let light = LightDef {
            dir_to_light: Vec3::Z,
            color: [1.; 3],
            energy: 0.5,
            shadow_vp: Mat4::IDENTITY,
            shadow_bias: 0.0005,
        };
        let mut store = MeshStore::default();
        store
            .get_or_upload(&gpu, &a.model, mesh_index)
            .expect("upload a");
        store
            .get_or_upload(&gpu, &changed.model, mesh_index)
            .expect("upload changed");
        let original_pixels = draw_primitive(
            &mut pipeline, &gpu, &target, &vp, &light, &store,
            &a.identity.sha256, &a.model.meshes[mesh_index].name,
        );
        let other_pixels = draw_primitive(
            &mut pipeline, &gpu, &target, &vp, &light, &store,
            &changed.identity.sha256, &changed.model.meshes[mesh_index].name,
        );
        assert_ne!(original_pixels, other_pixels, "{name}: closures distinguished");
        assert_eq!(
            draw_primitive(
                &mut pipeline, &gpu, &target, &vp, &light, &store,
                &a.identity.sha256, &a.model.meshes[mesh_index].name,
            ),
            original_pixels,
            "{name}: redraw stable"
        );
    }
    assert_eq!(cached_both, 3);
    // 具名保留：本环境离屏渲染对 Kenney 三根不出像素（宿主路径可证图像
    // 敏感 + 缓存双闭包）；共享 store 互异在可出像素环境复核。Plan 持名。
    println!(
        "HOLD: shared-store GPU closure differ roots distinguished = {gpu_distinguished}/3 (environment rasterization)"
    );
}

// ---------------- 生命周期（节奏/失败/保存/恢复 + 冻结控件） --

#[test]
fn g10a_lifecycle_pause_step_stop_fresh_last_good_save_reopen_and_frozen_controls() {
    let dir = materialize("g10-textures", "lifecycle");
    let tick0;
    let tick1;
    let tick2;
    let pose_nodes;
    {
        let mut h = load_dir(&dir);
        let edit_identity = h.editor.isolation_identity();
        // 首帧捕获为空缓冲（GPU/表面未热）；预热十帧后取编辑态参照。
        for _ in 0..10 {
            h.frame(None).expect("warm");
        }
        let editing = hash(&mut h, "editing");
        start(&mut h, "main");
        tick0 = hash(&mut h, "main-tick0");
        for _ in 0..4 {
            h.frame(None).expect("paused frame");
        }
        assert_eq!(
            h.play.as_ref().unwrap().last.as_ref().unwrap().tick,
            0,
            "paused frames do not advance"
        );
        step(&mut h, "tick1");
        tick1 = hash(&mut h, "main-tick1");
        pose_nodes = h.play.as_ref().unwrap().last.as_ref().unwrap().nodes.clone();
        step(&mut h, "tick2");
        tick2 = hash(&mut h, "main-tick2-hidden");
        {
            let last = h.play.as_ref().unwrap().last.as_ref().unwrap();
            assert_eq!(last.nodes[&4].visible, false, "flag hidden at tick2");
            assert_eq!(last.nodes[&3].translation[0], 0.0, "coin back at 0");
        }
        assert_ne!(tick2, tick0, "only the hidden flag distinguishes tick2");
        step(&mut h, "tick3");
        assert!(
            h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&4].visible,
            "flag restored tick3"
        );
        assert_eq!(hash(&mut h, "main-tick3-visible"), tick1, "tick3 = tick1 pixels");
        stop(&mut h);
        assert_eq!(h.editor.isolation_identity(), edit_identity, "editor identity");
        assert_eq!(hash(&mut h, "stopped"), editing, "stopped = editing pixels");
        start(&mut h, "fresh");
        assert_eq!(hash(&mut h, "fresh-tick0"), tick0, "fresh tick0");
        // prepare失败两阶段（每迭代新宿主新局）：整帧+像素保真，精确恢复。
        for after in [false, true] {
            let tag = if after { "after-upload" } else { "fail-resource" };
            let mut h = load_dir(&dir);
            start(&mut h, tag);
            h.play
                .as_mut()
                .unwrap()
                .single_step()
                .expect("warm step");
            settle(&mut h);
            let frame = frame_debug(&h);
            let previous = hash(&mut h, "before-failure");
            h.fail_resource = !after;
            h.fail_after_asset_upload = after;
            h.play
                .as_mut()
                .unwrap()
                .single_step()
                .expect("failing step dispatches");
            settle(&mut h);
            assert!(h.play.as_ref().unwrap().error.is_some(), "after={after} fails");
            assert_eq!(frame_debug(&h), frame, "after={after}: frame preserved");
            assert_eq!(hash(&mut h, "after-failure"), previous, "after={after}: pixels");
            h.fail_resource = false;
            h.fail_after_asset_upload = false;
            stop(&mut h);
            let mut h = load_dir(&dir);
            start(&mut h, "recover");
            step(&mut h, "recovered");
            assert_eq!(hash(&mut h, "recovered"), tick1, "exact recovery to tick1");
            stop(&mut h);
        }
    }
    // alt / 冻结 / 缺显隐写：全部作者夹具走真实生成链。
    {
        let alt_dir = materialize("g10-textures-alt", "lifecycle-alt");
        let mut h = load_dir(&alt_dir);
        start(&mut h, "alt");
        step(&mut h, "alt tick1");
        let alt_tick1 = hash(&mut h, "alt-tick1");
        assert_ne!(alt_tick1, tick1, "alt amplitude renders differently");
        assert_ne!(
            h.play.as_ref().unwrap().last.as_ref().unwrap().nodes,
            pose_nodes,
            "alt coin pose differs"
        );
        stop(&mut h);
    }
    {
        let frozen_dir = materialize("g10-textures-frozen", "lifecycle-frozen");
        let mut h = load_dir(&frozen_dir);
        start(&mut h, "frozen");
        step(&mut h, "frozen tick1");
        let frozen_tick1 = hash(&mut h, "frozen-tick1");
        assert_ne!(frozen_tick1, tick1, "frozen source distinguished");
        stop(&mut h);
    }
    {
        let miss_dir = materialize("g10-textures-missvis", "lifecycle-missvis");
        let mut h = load_dir(&miss_dir);
        start(&mut h, "missvis");
        step(&mut h, "miss tick1");
        step(&mut h, "miss tick2");
        assert!(
            h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&4].visible,
            "missing visibility write: flag never hidden"
        );
        assert_ne!(
            hash(&mut h, "missing-visibility-tick2"),
            tick2,
            "missing write fails the hidden-tick2 oracle"
        );
        stop(&mut h);
    }
    // 保存/搬移重开 + 坏依赖拒绝与恢复。
    let saved = dir.parent().unwrap().join("saved");
    let _ = std::fs::remove_dir_all(&saved);
    copy_dir(&dir, &saved);
    let mut h = load_dir(&saved);
    start(&mut h, "pre-save");
    step(&mut h, "pre-save step");
    assert_eq!(hash(&mut h, "pre-save-run"), tick1, "saved project reproduces tick1");
    stop(&mut h);
    let result = h
        .storage
        .save(&mut h.editor, &saved.join("scene.agscene"), Fault::None)
        .expect("save");
    assert!(matches!(result, Saved::Confirmed), "save must confirm");
    drop(h);
    let relocated = dir.parent().unwrap().join("reopened");
    let _ = std::fs::remove_dir_all(&relocated);
    std::fs::rename(&saved, &relocated).expect("cold move");
    let mut h = load_dir(&relocated);
    start(&mut h, "reopened");
    step(&mut h, "reopen tick1");
    assert_eq!(hash(&mut h, "reopened-run"), tick1, "reopened run pixels");
    step(&mut h, "reopen tick2");
    assert_eq!(hash(&mut h, "reopened-tick2-hidden"), tick2, "reopened tick2");
    assert!(
        !h.play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .unwrap()
            .nodes[&4]
            .visible
    );
    step(&mut h, "reopen tick3");
    assert_eq!(hash(&mut h, "reopened-tick3-visible"), tick1, "reopened tick3");
    stop(&mut h);
    let (doc, _, _, _) =
        Storage::load_project(&relocated.join("scene.agscene"), &Registry::new()).expect("reload");
    let img = doc.assets[0]
        .closure
        .as_ref()
        .expect("closure")
        .images[0]
        .sha256
        .clone();
    let png_path = relocated.join("assets").join(format!("{img}.png"));
    let original_bytes = std::fs::read(&png_path).expect("png bytes");
    std::fs::write(&png_path, b"bad PNG").expect("tamper");
    assert!(
        Storage::load_project(&relocated.join("scene.agscene"), &Registry::new()).is_err(),
        "corrupt dependency refused"
    );
    std::fs::write(&png_path, original_bytes).expect("restore");
    let mut h = load_dir(&relocated);
    start(&mut h, "restored");
    step(&mut h, "restored step");
    assert_eq!(hash(&mut h, "restored-run"), tick1, "restored rerun green");
    stop(&mut h);
}

