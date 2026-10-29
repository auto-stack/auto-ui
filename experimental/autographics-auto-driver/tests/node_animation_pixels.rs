//! Independent f64 source-key oracle -> actual Auto/Host matrices -> GPU ROI.
//! Never uses Animation::sample, its channels, or mesh_pose to form expectations.
use agp::{
    render3d::{LightDef, MeshInstance, MeshPipeline},
    renderer::{capture::capture_png, target::OffscreenTarget},
};
use autographics_auto_driver::authoring::{
    assets::{self, Asset, Registry},
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    interaction,
    play::Action,
    runtime_objects,
};
use glam::{Mat4, Quat, Vec2, Vec3};
use serde_json::Value;
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
fn pump(h: &mut Host) {
    let end = Instant::now() + Duration::from_secs(60);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < end);
        h.frame(None).unwrap();
    }
    assert!(
        h.play.as_ref().unwrap().error.is_none(),
        "{:?}",
        h.play.as_ref().unwrap().error
    );
}
fn step(h: &mut Host) {
    pump(h);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .unwrap();
    pump(h);
}
#[test]
fn nonuniform_node_scale_uses_inverse_transpose_normals_in_actual_gpu_lighting() {
    use agp::{
        assets::{node_animation::Animation, MaterialDef, MeshData, ModelData},
        render3d::MeshStore,
        renderer::GpuDevice,
    };
    let j = serde_json::json!({"nodes":[{}],"bufferViews":[{"buffer":0,"byteLength":8},{"buffer":0,"byteOffset":8,"byteLength":24}],"accessors":[{"bufferView":0,"componentType":5126,"count":2,"type":"SCALAR"},{"bufferView":1,"componentType":5126,"count":2,"type":"VEC3"}],"animations":[{"samplers":[{"input":0,"output":1}],"channels":[{"sampler":0,"target":{"node":0,"path":"scale"}}]}]});
    let bin: Vec<u8> = [0f32, 1., 1., 1., 1., 2., 1., 0.5]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let animation = Animation::from_parts(&j, &bin).unwrap();
    let model = ModelData {
        source_sha256: "normal-scale-oracle".into(),
        meshes: vec![MeshData {
            visual: None,
            name: "tilted".into(),
            positions: vec![
                [-0.25, -0.5, 0.25],
                [0.25, -0.5, -0.25],
                [0.25, 0.5, -0.25],
                [-0.25, 0.5, 0.25],
            ],
            normals: vec![Vec3::new(1., 0., 1.).normalize().to_array(); 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            material: MaterialDef {
                name: "grey".into(),
                base_color: [0.5, 0.5, 0.5, 1.],
                metallic: 0.,
                roughness: 1.,
            },
        }],
    };
    let gpu = GpuDevice::new().unwrap();
    let target = OffscreenTarget::new(&gpu, 64, 64).unwrap();
    let mut pipeline = MeshPipeline::new(&gpu, OffscreenTarget::FORMAT).unwrap();
    let mut store = MeshStore::default();
    store.get_or_upload(&gpu, &model, 0).unwrap();
    let matrix = Mat4::from_translation(Vec3::new(0., 0., 0.5))
        * animation.sample(Some(0), 1., false).unwrap()[0];
    let light = LightDef {
        dir_to_light: Vec3::Z,
        color: [1.; 3],
        energy: 0.5,
        shadow_vp: Mat4::IDENTITY,
        shadow_bias: 0.0005,
    };
    pipeline.render(
        &gpu,
        target.view(),
        64,
        64,
        [0., 0., 0., 1.],
        &Mat4::IDENTITY,
        &light,
        &store,
        &[MeshInstance {
            mesh: (model.source_sha256.clone(), model.meshes[0].name.clone()),
            model: matrix,
        }],
        false,
    );
    let image = capture_png(&gpu, &target).unwrap().1;
    let pixel = image.get_pixel(32, 32).0;
    let cosine = 2f64 / (0.5f64 * 0.5 + 4.).sqrt();
    let linear = 0.5 * (0.25 + 0.5 * cosine);
    let expected = (255. * (1.055 * linear.powf(1. / 2.4) - 0.055)).round() as u8;
    assert!(
        pixel[..3].iter().all(|v| v.abs_diff(expected) <= 2),
        "nonuniform normal: GPU={pixel:?}, independent expected={expected}"
    );
    println!("nonuniform node scale GPU normal pixel={pixel:?}, expected={expected}, tolerance=2");
}
fn character() -> Arc<Asset> {
    assets::import_animated(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/textured_models/models/character.glb"),
        &Registry::new(),
    )
    .unwrap()
}
fn scene(a: &Arc<Asset>) -> Document {
    let mut d = Document::empty("ui006-pixels".into());
    d.schema = "autographics.scene/6".into();
    d.assets.push(a.identity.clone());
    let (lo, hi) = interaction::bounds(&a.model).unwrap();
    let size = hi - lo;
    let mut o = Object::new(1, Kind::Model);
    o.resource = Some(a.identity.key());
    o.scale = [0.8 / size.x, 1. / size.y, 0.8 / size.z];
    d.objects.push(o.clone());
    o.id = 2;
    o.name = "template".into();
    o.visible = false;
    d.objects.push(o);
    d
}
fn chunks(a: &Asset) -> (Value, &[u8]) {
    let n = u32::from_le_bytes(a.bytes[12..16].try_into().unwrap()) as usize;
    (
        serde_json::from_slice(&a.bytes[20..20 + n]).unwrap(),
        &a.bytes[28 + n..],
    )
}
fn read(j: &Value, bin: &[u8], index: usize, width: usize) -> Vec<Vec<f64>> {
    let a = &j["accessors"][index];
    assert_eq!(a["componentType"], 5126);
    let v = &j["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
    let start = v["byteOffset"].as_u64().unwrap_or(0) as usize
        + a["byteOffset"].as_u64().unwrap_or(0) as usize;
    let stride = v["byteStride"]
        .as_u64()
        .map(|v| v as usize)
        .unwrap_or(width * 4);
    (0..a["count"].as_u64().unwrap() as usize)
        .map(|i| {
            (0..width)
                .map(|c| {
                    f32::from_le_bytes(
                        bin[start + i * stride + c * 4..start + i * stride + c * 4 + 4]
                            .try_into()
                            .unwrap(),
                    ) as f64
                })
                .collect()
        })
        .collect()
}
fn normalized(mut q: [f64; 4]) -> [f64; 4] {
    let len = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    for v in &mut q {
        *v /= len;
    }
    q
}
fn spherical(a: [f64; 4], mut b: [f64; 4], t: f64) -> [f64; 4] {
    let a = normalized(a);
    b = normalized(b);
    let mut dot = a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>();
    if dot < 0. {
        dot = -dot;
        for x in &mut b {
            *x = -*x;
        }
    }
    let (wa, wb) = if dot > 0.999999 {
        (1. - t, t)
    } else {
        let theta = dot.clamp(-1., 1.).acos();
        (
            ((1. - t) * theta).sin() / theta.sin(),
            (t * theta).sin() / theta.sin(),
        )
    };
    normalized(std::array::from_fn(|i| a[i] * wa + b[i] * wb))
}
fn reference(j: &Value, bin: &[u8], clip: usize, phase: f32, looping: bool) -> Vec<Mat4> {
    let nodes = j["nodes"].as_array().unwrap();
    let mut trs: Vec<_> = nodes
        .iter()
        .map(|n| {
            let values = |key: &str, default: Vec<f64>| {
                n.get(key)
                    .map(|a| {
                        a.as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_f64().unwrap())
                            .collect()
                    })
                    .unwrap_or(default)
            };
            (
                values("translation", vec![0.; 3]),
                values("rotation", vec![0., 0., 0., 1.]),
                values("scale", vec![1.; 3]),
            )
        })
        .collect();
    let anim = &j["animations"][clip];
    let duration = anim["samplers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            read(j, bin, s["input"].as_u64().unwrap() as usize, 1)
                .last()
                .unwrap()[0]
        })
        .fold(0., f64::max);
    let time = if duration == 0. {
        0.
    } else if looping {
        phase as f64 % duration
    } else {
        (phase as f64).min(duration)
    };
    for c in anim["channels"].as_array().unwrap() {
        let s = &anim["samplers"][c["sampler"].as_u64().unwrap() as usize];
        let path = c["target"]["path"].as_str().unwrap();
        let width = if path == "rotation" { 4 } else { 3 };
        let times = read(j, bin, s["input"].as_u64().unwrap() as usize, 1);
        let values = read(j, bin, s["output"].as_u64().unwrap() as usize, width);
        let hi = times
            .iter()
            .position(|v| v[0] > time)
            .unwrap_or(times.len() - 1);
        let lo = if time >= times[hi][0] {
            hi
        } else {
            hi.saturating_sub(1)
        };
        let weight = if hi == lo || time <= times[0][0] || s["interpolation"] == "STEP" {
            0.
        } else {
            (time - times[lo][0]) / (times[hi][0] - times[lo][0])
        };
        let value = if path == "rotation" {
            spherical(
                values[lo].clone().try_into().unwrap(),
                values[hi].clone().try_into().unwrap(),
                weight,
            )
            .to_vec()
        } else {
            (0..width)
                .map(|i| values[lo][i] * (1. - weight) + values[hi][i] * weight)
                .collect()
        };
        let t = &mut trs[c["target"]["node"].as_u64().unwrap() as usize];
        match path {
            "translation" => t.0 = value,
            "rotation" => t.1 = value,
            "scale" => t.2 = value,
            _ => panic!("original TRS"),
        }
    }
    let mut result = vec![Mat4::IDENTITY; nodes.len()];
    fn visit(
        i: usize,
        parent: Mat4,
        j: &Value,
        trs: &[(Vec<f64>, Vec<f64>, Vec<f64>)],
        out: &mut [Mat4],
    ) {
        let (t, r, s) = &trs[i];
        let vec3 = |v: &Vec<f64>| Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32);
        let local = Mat4::from_scale_rotation_translation(
            vec3(s),
            Quat::from_array(std::array::from_fn(|k| r[k] as f32)),
            vec3(t),
        );
        out[i] = parent * local;
        if let Some(children) = j["nodes"][i]["children"].as_array() {
            for child in children {
                visit(child.as_u64().unwrap() as usize, out[i], j, trs, out);
            }
        }
    }
    for n in j["scenes"][0]["nodes"].as_array().unwrap() {
        visit(
            n.as_u64().unwrap() as usize,
            Mat4::IDENTITY,
            j,
            &trs,
            &mut result,
        );
    }
    result
}
struct Shot {
    rgba: Vec<u8>,
    png: Vec<u8>,
}
fn shot(
    h: &Host,
    pipeline: &mut MeshPipeline,
    target: &OffscreenTarget,
    vp: Mat4,
    light: &LightDef,
    instances: &[MeshInstance],
    shadow: bool,
) -> Shot {
    let store = match h.rt.content(h.three).unwrap() {
        agp::host::surface_runtime::SurfaceContent::Scene3d(s) => &s.store,
        _ => panic!(),
    };
    pipeline.render(
        h.rt.gpu(),
        target.view(),
        320,
        320,
        [1., 0., 1., 1.],
        &vp,
        light,
        store,
        instances,
        shadow,
    );
    let (png, image) = capture_png(h.rt.gpu(), target).unwrap();
    Shot {
        rgba: image.into_raw(),
        png,
    }
}
fn mask(triangles: &[[Vec3; 3]], vp: Mat4) -> Vec<bool> {
    let mut mask = vec![false; 320 * 320];
    for tri in triangles {
        let p: Vec<_> = tri
            .iter()
            .map(|p| {
                let v = vp * p.extend(1.);
                Vec2::new((v.x / v.w + 1.) * 160., (1. - v.y / v.w) * 160.)
            })
            .collect();
        let edge = |a: Vec2, b: Vec2, c: Vec2| (b - a).perp_dot(c - a);
        let area = edge(p[0], p[1], p[2]);
        if area.abs() < 1e-6 {
            continue;
        }
        let lo = p
            .iter()
            .fold(Vec2::splat(320.), |a, b| a.min(*b))
            .floor()
            .max(Vec2::ZERO);
        let hi = p
            .iter()
            .fold(Vec2::ZERO, |a, b| a.max(*b))
            .ceil()
            .min(Vec2::splat(319.));
        for y in lo.y as usize..=hi.y as usize {
            for x in lo.x as usize..=hi.x as usize {
                let q = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                if [
                    edge(p[0], p[1], q),
                    edge(p[1], p[2], q),
                    edge(p[2], p[0], q),
                ]
                .iter()
                .all(|v| *v * area >= 0.)
                {
                    mask[y * 320 + x] = true;
                }
            }
        }
    }
    mask
}
fn silhouette_errors(mask: &[bool], rgba: &[u8]) -> (usize, usize) {
    let (mut probes, mut errors) = (0, 0);
    for y in 2..318 {
        for x in 2..318 {
            let value = mask[y * 320 + x];
            let interior = (-1..=1).all(|dy| {
                (-1..=1).all(|dx| mask[((y as i32 + dy) * 320 + x as i32 + dx) as usize] == value)
            });
            if interior {
                probes += 1;
                let p = &rgba[(y * 320 + x) * 4..][..3];
                let ink = p[0].abs_diff(255) > 4 || p[1] > 4 || p[2].abs_diff(255) > 4;
                if ink != value {
                    errors += 1;
                }
            }
        }
    }
    (probes, errors)
}
fn report(name: &str, png: &[u8]) {
    if let Some(root) = std::env::var_os("UI006_REPORT_DIR") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(format!("{name}.png")), png).unwrap();
    }
}

#[test]
fn original_four_clips_phase_boundaries_and_independent_shared_instances_match_source_vertices_and_gpu_rois(
) {
    let a = character();
    let (j, bin) = chunks(&a);
    let d = scene(&a);
    let mut r = Registry::new();
    assets::insert(&mut r, a.clone()).unwrap();
    let mut h = Host::with_assets(d, r).unwrap();
    let source="var ok=0\nvar ticks=0\nvar slot=0\nfn app_start(){ok=ag_char3_bind(1);ok=ag_camera_orbit_3d(1,4.0,30.0,0,0.0);slot=ag_object_spawn(0,2,1.5,0.0,0.0);ok=ag_visual_pose(0,1,0,0.0,0,0.45)}\nfn app_update(dt float){ticks=ticks+1;var clip=(ticks-1)/5;let state=(ticks-1)%5;var duration=0.10000000149011612;if clip==1 {duration=0.6666666865348816};if clip==2 {duration=0.4333333373069763};if clip==3 {duration=0.5};var phase=0.0;var looping=0;if state==1 {phase=duration*0.37};if state>1 {phase=duration};if state==3 {phase=duration*2.0};if state==4 {looping=1};if ticks>20 {clip=2;phase=0.19;looping=0};ok=ag_visual_pose(0,1,clip,phase,looping,0.45);ok=ag_visual_pose(1,slot,(clip+1)%4,phase+0.13,1,-0.6)}\n";
    h.start_user_snapshot(
        SourceSnapshot::from_bytes_for(
            Path::new("four-clips.at"),
            source.as_bytes().to_vec(),
            Api::NodeAnimation3V1,
        )
        .unwrap(),
    )
    .unwrap();
    h.pause_play("");
    pump(&mut h);
    let target = OffscreenTarget::new(h.rt.gpu(), 320, 320).unwrap();
    let mut pipeline = MeshPipeline::new(h.rt.gpu(), OffscreenTarget::FORMAT).unwrap();
    let vp = agp::render3d::Camera3d {
        eye: Vec3::new(1.6, 1., 5.),
        target: Vec3::new(0., 0.5, 0.),
        up: Vec3::Y,
        projection: agp::render3d::Projection::Orthographic {
            size: 2.8,
            aspect: 1.,
            near: 0.1,
            far: 20.,
        },
    }
    .view_proj();
    let light = LightDef {
        dir_to_light: Vec3::Z,
        color: [1.; 3],
        energy: 0.5,
        shadow_vp: Mat4::IDENTITY,
        shadow_bias: 0.0005,
    };
    let mut first = None;
    let mut end_pose = None;
    let mut controls = Vec::new();
    let mut dynamic_differences = Vec::new();
    let mut maximum = 0f32;
    let mut vertex_checks = 0;
    let mut pixels = 0;
    for tick in 1..=21 {
        step(&mut h);
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
        let visuals = frame.animations.as_ref().unwrap();
        let visual = visuals.visual_for(1).unwrap();
        assert_eq!(visual.clip, if tick == 21 { 2 } else { (tick - 1) / 5 });
        assert_eq!(
            visuals.slots[0].as_ref().unwrap().visual.clip,
            (visual.clip + 1) % 4
        );
        let instances = match h.rt.content(h.three).unwrap() {
            agp::host::surface_runtime::SurfaceContent::Scene3d(s) => s.instances.clone(),
            _ => panic!(),
        };
        assert_eq!(instances.len(), a.model.meshes.len() * 2);
        let mut triangles = Vec::new();
        for (row, v, entity) in [
            (0, visual, frame.objects[0].clone()),
            (
                1,
                &visuals.slots[0].as_ref().unwrap().visual,
                runtime_objects::instance_object(
                    &frame.document.objects[1],
                    frame.instances.as_ref().unwrap().get(0).unwrap(),
                )
                .unwrap(),
            ),
        ] {
            let reference = reference(&j, bin, v.clip as usize, v.phase, v.looping);
            let entity = interaction::transform(&entity) * Mat4::from_rotation_y(v.yaw);
            for (mi, mesh) in a.model.meshes.iter().enumerate() {
                let meta = mesh.visual.as_ref().unwrap();
                let primitive = &j["meshes"][meta.mesh_index]["primitives"][meta.primitive_index];
                let raw = read(
                    &j,
                    bin,
                    primitive["attributes"]["POSITION"].as_u64().unwrap() as usize,
                    3,
                );
                let expected: Vec<_> = raw
                    .iter()
                    .map(|p| {
                        (entity * reference[meta.node_index]).transform_point3(Vec3::new(
                            p[0] as f32,
                            p[1] as f32,
                            p[2] as f32,
                        ))
                    })
                    .collect();
                for (i, p) in mesh.positions.iter().enumerate() {
                    let actual = instances[row * a.model.meshes.len() + mi]
                        .model
                        .transform_point3((*p).into());
                    maximum = maximum.max((actual - expected[i]).length());
                    assert!(
                        (actual - expected[i]).length() < 2e-5,
                        "clip{} node{} vertex{i}",
                        v.clip,
                        meta.node_index
                    );
                    vertex_checks += 1;
                }
                if row == 0 {
                    for t in mesh.indices.chunks_exact(3) {
                        triangles.push([
                            expected[t[0] as usize],
                            expected[t[1] as usize],
                            expected[t[2] as usize],
                        ]);
                    }
                }
            }
        }
        let actual = shot(
            &h,
            &mut pipeline,
            &target,
            vp,
            &light,
            &instances[..a.model.meshes.len()],
            false,
        );
        let expected = mask(&triangles, vp);
        let (probes, errors) = silhouette_errors(&expected, &actual.rgba);
        assert!(
            errors <= 2,
            "clip{} state{} GPU silhouette misses {errors}/{probes}",
            visual.clip,
            (tick - 1) % 5
        );
        pixels += probes;
        if tick <= 20 && (tick - 1) % 5 == 1 && visual.clip > 0 {
            let mut frozen = instances[..a.model.meshes.len()].to_vec();
            for inst in &mut frozen {
                inst.model =
                    interaction::transform(&frame.objects[0]) * Mat4::from_rotation_y(visual.yaw);
            }
            let wrong = shot(&h, &mut pipeline, &target, vp, &light, &frozen, false);
            let (_, errors) = silhouette_errors(&expected, &wrong.rgba);
            assert!(
                errors > 2,
                "removing node tracks must fail the independent GPU silhouette oracle for clip{}",
                visual.clip
            );
            controls.push(errors);
        }
        report(
            &format!("clip{}-state{}-tick{tick}", visual.clip, (tick - 1) % 5),
            &actual.png,
        );
        if tick == 21 {
            continue;
        }
        match (tick - 1) % 5 {
            0 => first = Some(actual.rgba),
            1 => {
                let diff = first
                    .as_ref()
                    .unwrap()
                    .iter()
                    .zip(&actual.rgba)
                    .filter(|(a, b)| a != b)
                    .count();
                if visual.clip == 0 {
                    assert_eq!(diff, 0, "static is a held rest pose");
                } else {
                    assert!(diff > 10, "clip{} must visibly animate", visual.clip);
                    dynamic_differences.push(diff);
                }
            }
            2 => end_pose = Some(actual.rgba),
            3 => assert_eq!(
                end_pose.as_ref().unwrap(),
                &actual.rgba,
                "one-shot holds beyond exact end"
            ),
            4 => assert_eq!(
                first.as_ref().unwrap(),
                &actual.rgba,
                "exact duration loop wraps to first pose"
            ),
            _ => {}
        }
    }
    println!("four original clips20 phase states+nonzero switch;source vertices={vertex_checks} max_error={maximum};GPU stable ROI probes={pixels};idle/walk/jump changed channels={dynamic_differences:?};shared instances independent;frozen-node ROI rejections={controls:?}");
    h.stop_play();
}
