//! PLAN-034 动画面原八项oracle复刻（V-S02/V-S03/V-S04/V-S05；AC-02..05）。
//! 原 tools/gallery_g10b.py 八项逐名经新链（pilot components-3d-presentation
//! 产品 + 公开 NodeAnimation3V1 控制面 / 作者夹具）成立：
//! original_four_clips_source_gpu（独立f64 reference：原始GLB JSON/BIN直读、
//! lerp/slerp/层级、21态逐顶点<2e-5、GPU轮廓≤2、删轨负控>2、共享实例）、
//! camera_relative_input_facing（7例朝向<1e-5 + 轮廓负控 + 停步保朝向 +
//! ±π回绕）、phase_main_alt_frozen_control（作者夹具 [0.2,0.4,0.0] 逐字）、
//! pose_shadow_physics_independent（240Tick同输入trace逐元素相同，仅Run
//! identity置零）、pause_step_stop_fresh_run（dt=1/60逐字、新局初帧逐字）、
//! stale_identity_last_good（错clip46/旧代33/prepare失败保真——具名作者面
//! 33计费夹具真fault，其余沿006已交付运行时门）、resource_union_budget
//! （两工程并集8身份/150787B逐字）、save_close_reopen_run。
//! GPU 单实例：--test-threads=1。
//! 运行：cargo test --locked --offline --test node3d_presentation_oracles_anim -- --test-threads=1
use agp::render3d::{LightDef, MeshInstance, MeshPipeline};
use agp::renderer::capture::capture_png;
use agp::renderer::device::GpuDevice;
use agp::renderer::target::OffscreenTarget;
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object},
    host::Host,
    interaction,
    play::{Action, CharProfile},
    runtime_objects,
    storage::{Fault, Saved, Storage},
};
use glam::{Mat4, Quat, Vec2, Vec3};
use serde_json::Value;
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

fn materialize(product: &str, tag: &str) -> PathBuf {
    let scene = product_text(product, "scene.agscene");
    let world = product_text(product, "world.at");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan034-oracles-anim")
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

fn settle(h: &mut Host, expect_error: bool) {
    let end = Instant::now() + std::time::Duration::from_secs(40);
    while h.play.as_ref().map(|p| p.pending()).unwrap_or(false) {
        assert!(Instant::now() < end, "settle deadline");
        h.frame(None).expect("frame");
    }
    if expect_error {
        let err = h.play.as_ref().and_then(|p| p.error.clone());
        assert!(err.is_none(), "unexpected play error: {err:?}");
    }
}

fn start_bound(h: &mut Host, tag: &str) {
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start: {e:?}"));
    assert_eq!(
        h.play.as_ref().unwrap().character_profile(),
        Some(CharProfile::ThreeD),
        "{tag}: 3D character profile"
    );
    h.pause_play("");
    settle(h, true);
}

fn start_control(h: &mut Host, source: &str, tag: &str) {
    h.start_user_snapshot(
        SourceSnapshot::from_bytes_for(Path::new("gallery-control.at"), source.as_bytes().to_vec(), Api::NodeAnimation3V1)
            .unwrap_or_else(|e| panic!("{tag} snapshot: {e:?}")),
    )
    .unwrap_or_else(|e| panic!("{tag} start control: {e:?}"));
    h.pause_play("");
    settle(h, true);
}

fn stop(h: &mut Host) {
    h.stop_play();
    settle(h, false);
}

fn input(h: &mut Host, bits: u32, drag: f32, expect_ok: bool) {
    if let Some(p) = h.play.as_mut() {
        let _ = p.resume();
        for k in 0..6usize {
            p.char_key(k, bits & (1 << k) != 0, false);
        }
        p.cam_drag(drag, 0.0);
        let _ = p.dispatch(Action::default());
    }
    h.pause_play("");
    settle(h, expect_ok);
}

fn capture(h: &mut Host, label: &str) -> String {
    h.frame(None).expect("frame");
    h.rt.capture(h.three, label).expect("capture").0
}

fn visual_of(h: &Host, id: u64) -> (i32, f32, bool, f32) {
    let p = h.play.as_ref().unwrap();
    let anim = p
        .last
        .as_ref()
        .and_then(|f| f.animations.as_ref())
        .unwrap_or_else(|| panic!("animation frame"));
    let v = anim
        .visual_for(id)
        .unwrap_or_else(|| panic!("visual target {id}"));
    (v.clip, v.phase, v.looping, v.yaw)
}

// ---------------- 独立 f64 reference（原始GLB JSON/BIN直读；不用生产sampler） --

fn chunks(a: &assets::Asset) -> (Value, Vec<u8>) {
    let source = &*a.bytes;
    let n = u32::from_le_bytes(source[12..16].try_into().unwrap()) as usize;
    let j: Value = serde_json::from_slice(&source[20..20 + n]).expect("json");
    (j, source[28 + n..].to_vec())
}

fn read_acc(j: &Value, bin: &[u8], index: usize, width: usize) -> Vec<Vec<f64>> {
    let acc = &j["accessors"][index];
    assert_eq!(acc["componentType"].as_u64(), Some(5126), "f32 only");
    let bv = &j["bufferViews"][acc["bufferView"].as_u64().unwrap() as usize];
    let base = bv["byteOffset"].as_u64().unwrap_or(0) as usize
        + acc["byteOffset"].as_u64().unwrap_or(0) as usize;
    let stride = bv["byteStride"].as_u64().unwrap_or((width * 4) as u64) as usize;
    let count = acc["count"].as_u64().unwrap() as usize;
    (0..count)
        .map(|i| {
            (0..width)
                .map(|k| {
                    let at = base + i * stride + k * 4;
                    f32::from_le_bytes(bin[at..at + 4].try_into().unwrap()) as f64
                })
                .collect()
        })
        .collect()
}

fn reference(j: &Value, bin: &[u8], clip: usize, phase: f32, looping: bool) -> Vec<Mat4> {
    let nodes = j["nodes"].as_array().unwrap();
    let mut trs: Vec<(Vec<f64>, Vec<f64>, Vec<f64>)> = nodes
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
            read_acc(j, bin, s["input"].as_u64().unwrap() as usize, 1)
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
        let times = read_acc(j, bin, s["input"].as_u64().unwrap() as usize, 1);
        let values = read_acc(j, bin, s["output"].as_u64().unwrap() as usize, width);
        let hi = times
            .iter()
            .position(|v| v[0] > time)
            .unwrap_or(times.len() - 1);
        let lo = if time >= times[hi][0] { hi } else { hi.saturating_sub(1) };
        let weight = if hi == lo || time <= times[0][0] || s["interpolation"] == "STEP" {
            0.
        } else {
            (time - times[lo][0]) / (times[hi][0] - times[lo][0])
        };
        let target_node = c["target"]["node"].as_u64().unwrap() as usize;
        match path {
            "translation" => {
                for k in 0..3 {
                    trs[target_node].0[k] =
                        (values[lo][k] * (1. - weight) + values[hi][k] * weight) as f64;
                }
            }
            "scale" => {
                for k in 0..3 {
                    trs[target_node].2[k] =
                        (values[lo][k] * (1. - weight) + values[hi][k] * weight) as f64;
                }
            }
            "rotation" => {
                let (a, b) = (&values[lo], &values[hi]);
                let mut qa = Quat::from_xyzw(a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32)
                    .normalize();
                let mut qb = Quat::from_xyzw(b[0] as f32, b[1] as f32, b[2] as f32, b[3] as f32)
                    .normalize();
                if qa.dot(qb) < 0. {
                    qb = -qb;
                }
                let dot = qa.dot(qb).clamp(-1., 1.);
                let q = if dot > 0.999999 {
                    qa.lerp(qb, weight as f32)
                } else {
                    let theta = dot.acos();
                    let sin = theta.sin();
                    let wa = ((1. - weight as f32) * theta).sin() / sin;
                    let wb = (weight as f32 * theta).sin() / sin;
                    qa * wa + qb * wb
                }
                .normalize();
                qa = q;
                trs[target_node].1 = vec![
                    qa.x as f64,
                    qa.y as f64,
                    qa.z as f64,
                    qa.w as f64,
                ];
            }
            other => panic!("unsupported channel path {other}"),
        }
    }
    let mut out = vec![Mat4::IDENTITY; nodes.len()];
    fn visit(
        j: &Value,
        trs: &[(Vec<f64>, Vec<f64>, Vec<f64>)],
        out: &mut Vec<Mat4>,
        index: usize,
        parent: Mat4,
    ) {
        let (t, r, s) = &trs[index];
        let local = Mat4::from_scale_rotation_translation(
            Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32),
            Quat::from_xyzw(r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32),
            Vec3::new(t[0] as f32, t[1] as f32, t[2] as f32),
        );
        out[index] = parent * local;
        if let Some(children) = j["nodes"][index]["children"].as_array() {
            for c in children {
                visit(j, trs, out, c.as_u64().unwrap() as usize, out[index]);
            }
        }
    }
    for root in j["scenes"][0]["nodes"].as_array().unwrap() {
        visit(j, &trs, &mut out, root.as_u64().unwrap() as usize, Mat4::IDENTITY);
    }
    out
}

fn fixed_view() -> Mat4 {
    let camera = agp::render3d::Camera3d {
        eye: Vec3::new(1.6, 1.0, 5.0),
        target: Vec3::new(0.0, 0.5, 0.0),
        up: Vec3::Y,
        projection: agp::render3d::Projection::Orthographic {
            size: 2.8,
            aspect: 1.0,
            near: 0.1,
            far: 20.0,
        },
    };
    camera.view_proj()
}

fn fixed_light() -> LightDef {
    LightDef {
        dir_to_light: Vec3::Z,
        color: [1.; 3],
        energy: 0.5,
        shadow_vp: Mat4::IDENTITY,
        shadow_bias: 0.0005,
    }
}

const TARGET: usize = 320;

fn mask(triangles: &[[Vec3; 3]], vp: Mat4) -> Vec<bool> {
    let mut mask = vec![false; TARGET * TARGET];
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
                if [edge(p[0], p[1], q), edge(p[1], p[2], q), edge(p[2], p[0], q)]
                    .iter()
                    .all(|v| *v * area >= 0.)
                {
                    mask[y * TARGET + x] = true;
                }
            }
        }
    }
    mask
}

fn silhouette_errors(m: &[bool], rgba: &[u8]) -> (usize, usize) {
    let (mut probes, mut errors) = (0, 0);
    for y in 2..TARGET - 2 {
        for x in 2..TARGET - 2 {
            let value = m[y * TARGET + x];
            let interior = (-1..=1).all(|dy| {
                (-1..=1).all(|dx| m[((y as i32 + dy) * TARGET as i32 + x as i32 + dx) as usize] == value)
            });
            if !interior {
                continue;
            }
            let at = (y * TARGET + x) * 4;
            let ink = (rgba[at] as i32 - 255).abs() > 4
                || rgba[at + 1] > 4
                || (rgba[at + 2] as i32 - 255).abs() > 4;
            probes += 1;
            errors += usize::from(ink != value);
        }
    }
    (probes, errors)
}

/// 独立f64姿态的actor三角形（含 yaw；node bake 逆阵还原）。
fn actor_triangles(
    a: &assets::Asset,
    j: &Value,
    bin: &[u8],
    entity: Mat4,
    clip: usize,
    phase: f32,
    yaw: f32,
) -> Vec<[Vec3; 3]> {
    let worlds = reference(j, bin, clip, phase, false);
    let yawed = entity * Mat4::from_rotation_y(yaw);
    let mut out = vec![];
    for mesh in &a.model.meshes {
        let meta = mesh.visual.as_ref().expect("staged visual");
        let base = yawed
            * worlds[meta.node_index]
            * Mat4::from_cols_array_2d(&meta.world).inverse();
        for tri in mesh.indices.chunks_exact(3) {
            out.push([
                base.transform_point3(Vec3::from(mesh.positions[tri[0] as usize])),
                base.transform_point3(Vec3::from(mesh.positions[tri[1] as usize])),
                base.transform_point3(Vec3::from(mesh.positions[tri[2] as usize])),
            ]);
        }
    }
    out
}

/// 像素文档：角色（产品闭包身份）+ 不可见模板，控制面驱动。
fn pixel_doc(a: &assets::Asset) -> Document {
    let mut d = Document::empty(format!("plan034-pixels-{}", a.identity.sha256[..8].to_string()));
    d.schema = "autographics.scene/6".into();
    d.assets = vec![a.identity.clone()];
    let resource = Some(a.identity.key());
    let mut o = Object::new(1, Kind::Model);
    o.resource = resource.clone();
    // 原 pixels 文档合同：缩放使角色世界 AABB 恰为受信 0.8×1.0×0.8 门。
    if let Some((lo, hi)) = interaction::bounds(&a.model) {
        let size = hi - lo;
        o.scale = [
            0.8 / size.x.max(1e-4),
            1.0 / size.y.max(1e-4),
            0.8 / size.z.max(1e-4),
        ];
    }
    d.objects.push(o);
    let mut template = Object::new(2, Kind::Model);
    template.resource = resource;
    template.visible = false;
    d.objects.push(template);
    d
}

// ---------------- 八项之一：original_four_clips_source_gpu ----------------

const FOUR_CLIPS_CONTROL: &str = r#"var ticks=0
var ok=0
var slot=0
var clip=0
var state=0
var duration=0.10000000149011612
var phase=0.0
var looping=0
fn app_start() {
 ok=ag_char3_bind(1)
 ok=ag_camera_orbit_3d(1,4.0,30.0,0,0.0)
 slot=ag_object_spawn(0,2,1.5,0.0,0.0)
 ok=ag_visual_pose(0,1,0,0.0,0,0.45)
}
fn app_update(dt float) {
 ticks=ticks+1
 clip=(ticks-1)/5
 state=(ticks-1)%5
 duration=0.10000000149011612
 if clip==1 {duration=0.6666666865348816}
 if clip==2 {duration=0.4333333373069763}
 if clip==3 {duration=0.5}
 phase=0.0
 looping=0
 if state==1 {phase=duration*0.37}
 if state>1 {phase=duration}
 if state==3 {phase=duration*2.0}
 if state==4 {looping=1}
 if ticks>20 {clip=2;phase=0.19;looping=0}
 ok=ag_visual_pose(0,1,clip,phase,looping,0.45)
 ok=ag_visual_pose(1,slot,(clip+1)%4,phase+0.13,1,-0.6)
}
"#;

#[test]
fn g10b_original_four_clips_source_gpu_independent_f64_reference() {
    let dir = materialize("g10-animation", "fourclips");
    let (doc, registry, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("load");
    let a = registry[&doc.assets[0].key()].clone();
    assert!(a.animation.is_some(), "character carries the animation payload");
    let (j, bin) = chunks(&a);
    assert_eq!(
        j["animations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["name"].as_str().unwrap().to_string())
            .collect::<Vec<_>>(),
        ["static", "idle", "walk", "jump"],
        "original clip ordinals/names"
    );
    let mut d = pixel_doc(&a);
    d.objects[1].visible = false;
    let mut reg = Registry::new();
    assets::insert(&mut reg, a.clone()).expect("seed");
    let mut h = Host::with_assets(d, reg).expect("host");
    start_control(&mut h, FOUR_CLIPS_CONTROL, "fourclips");
    let vp = fixed_view();
    let light = fixed_light();
    // 宿主GPU：离屏 target/pipeline 必须与 content.store 同设备。
    let target = OffscreenTarget::new(h.rt.gpu(), TARGET as u32, TARGET as u32).expect("target");
    let mut pipeline = MeshPipeline::new(h.rt.gpu(), OffscreenTarget::FORMAT).expect("pipeline");
    let mesh_count = a.model.meshes.len();
    let mut first_rgba: Option<Vec<u8>> = None;
    let mut end_rgba: Option<Vec<u8>> = None;
    let mut removed_track_errors = vec![];
    let mut total_probes = 0usize;
    for tick in 1..=21i32 {
        // 一 tick：pump → dispatch → pump（pixels.rs 节奏）。
        settle(&mut h, true);
        h.play.as_mut().unwrap().dispatch(Action::default()).expect("tick");
        settle(&mut h, true);
        let (clip, phase, looping, yaw) = visual_of(&h, 1);
        let expect_clip = if tick == 21 { 2 } else { (tick - 1) / 5 };
        assert_eq!(clip, expect_clip, "tick{tick} clip");
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
        let anim_frame = frame.animations.as_ref().unwrap();
        let slot_vis = anim_frame.slots[0].as_ref().expect("spawned slot visual");
        assert_eq!(
            slot_vis.visual.clip,
            (clip + 1) % 4,
            "tick{tick}: shared instance clip offset"
        );
        let content = match h.rt.content(h.three).expect("content") {
            agp::host::surface_runtime::SurfaceContent::Scene3d(s) => s,
            _ => panic!("scene3d"),
        };
        let instances = &content.instances;
        assert_eq!(instances.len(), 2 * mesh_count, "tick{tick}: actor+instance rows");
        // 行0（角色文档叶）与行1（spawn 实例）独立f64逐顶点核对。
        for row in 0..2usize {
            let (v_clip, v_phase, v_loop, v_yaw) = if row == 0 {
                (clip, phase, looping, yaw)
            } else {
                let sv = &slot_vis.visual;
                (sv.clip, sv.phase, sv.looping, sv.yaw)
            };
            let obj = if row == 0 {
                frame.objects[0].clone()
            } else {
                runtime_objects::instance_object(&frame.document.objects[1], frame.instances.as_ref().unwrap().get(0).unwrap())
                    .expect("instance object")
            };
            let worlds = reference(&j, &bin, v_clip as usize, v_phase, v_loop);
            let entity = interaction::transform(&obj) * Mat4::from_rotation_y(v_yaw);
            for (mi, mesh) in a.model.meshes.iter().enumerate() {
                let meta = mesh.visual.as_ref().expect("visual");
                let base = entity * worlds[meta.node_index]
                    * Mat4::from_cols_array_2d(&meta.world).inverse();
                let actual = &instances[row * mesh_count + mi];
                for (vi, p) in mesh.positions.iter().enumerate() {
                    let expected = base.transform_point3(Vec3::from(*p));
                    let got = actual.model.transform_point3(Vec3::from(*p));
                    let err = (expected - got).length();
                    assert!(
                        err < 2e-5,
                        "clip{v_clip} node{} vertex{vi}: {err} (row {row}, tick {tick})",
                        meta.node_index
                    );
                }
            }
        }
        // GPU 轮廓（仅 actor 行；magenta 底）。
        let row0_obj = frame.objects[0].clone();
        let entity0 = interaction::transform(&row0_obj) * Mat4::from_rotation_y(yaw);
        let mut triangles = vec![];
        for mesh in &a.model.meshes {
            let meta = mesh.visual.as_ref().expect("visual");
            let worlds = reference(&j, &bin, clip as usize, phase, looping);
            let base = entity0 * worlds[meta.node_index]
                * Mat4::from_cols_array_2d(&meta.world).inverse();
            for tri in mesh.indices.chunks_exact(3) {
                triangles.push([
                    base.transform_point3(Vec3::from(mesh.positions[tri[0] as usize])),
                    base.transform_point3(Vec3::from(mesh.positions[tri[1] as usize])),
                    base.transform_point3(Vec3::from(mesh.positions[tri[2] as usize])),
                ]);
            }
        }
        let actor_instances: Vec<MeshInstance> = instances[..mesh_count].to_vec();
        pipeline.render(h.rt.gpu(), target.view(), TARGET as u32, TARGET as u32, [1., 0., 1., 1.], &vp, &light, &content.store, &actor_instances, false);
        let shot = capture_png(h.rt.gpu(), &target).expect("shot").1;
        let rgba = shot.into_raw();
        let m = mask(&triangles, vp);
        let (probes, errors) = silhouette_errors(&m, &rgba);
        total_probes += probes;
        assert!(errors <= 2, "clip{clip} state{} tick{tick}: GPU silhouette misses {errors}", (tick - 1) % 5);
        // 删轨负控：ticks 7/12/17（clip 1/2/3 中态）刚体姿态必须可见差异。
        if tick <= 20 && (tick - 1) % 5 == 1 && clip > 0 {
            let rigid: Vec<MeshInstance> = actor_instances
                .iter()
                .map(|i| MeshInstance { mesh: i.mesh.clone(), model: entity0 })
                .collect();
            pipeline.render(h.rt.gpu(), target.view(), TARGET as u32, TARGET as u32, [1., 0., 1., 1.], &vp, &light, &content.store, &rigid, false);
            let rigid_shot = capture_png(h.rt.gpu(), &target).expect("shot").1;
            let (_, rigid_errors) = silhouette_errors(&m, &rigid_shot.into_raw());
            assert!(rigid_errors > 2, "tick{tick}: removed-track control must fail");
            removed_track_errors.push(rigid_errors);
        }
        // 状态断言（tick21 跳过）。
        if tick <= 20 {
            let state = (tick - 1) % 5;
            match state {
                0 => first_rgba = Some(rgba.clone()),
                1 => {
                    let first = first_rgba.as_ref().unwrap();
                    let diff = first.iter().zip(rgba.iter()).filter(|(a, b)| a != b).count();
                    if clip == 0 {
                        assert_eq!(diff, 0, "static is a held rest pose");
                    } else {
                        assert!(diff > 10, "clip{clip} mid state must visibly animate");
                    }
                }
                2 => end_rgba = Some(rgba.clone()),
                3 => {
                    assert_eq!(
                        end_rgba.as_ref().unwrap().as_slice(),
                        &rgba[..],
                        "one-shot holds beyond exact end"
                    );
                }
                4 => {
                    assert_eq!(
                        first_rgba.as_ref().unwrap().as_slice(),
                        &rgba[..],
                        "exact duration loop wraps to first pose"
                    );
                }
                _ => unreachable!(),
            }
        }
    }
    assert!(total_probes > 100000, "silhouette probe volume {total_probes}");
    assert_eq!(removed_track_errors.len(), 3);
    stop(&mut h);
}

// ---------------- 八项之二：camera_relative_input_facing ----------------

#[test]
fn g10b_camera_relative_input_facing_seven_cases() {
    let dir = materialize("g10-animation", "facing");
    let (doc, registry, _, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("load");
    let a = registry[&doc.assets[0].key()].clone();
    let (j, bin) = chunks(&a);
    let mut d = pixel_doc(&a);
    d.objects[1].translation[0] = 30.; // 模板停远（030 语义：spawn 目标远离原点）
    let mut reg = Registry::new();
    assets::insert(&mut reg, a.clone()).expect("seed");
    let mut h = Host::with_assets(d, reg).expect("host");
    let facing_control = r#"var ok=0
fn app_start() {
 ok=ag_char3_bind(1)
 ok=ag_camera_orbit_3d(1,4.0,30.0,0,0.0)
}
fn app_update(dt float) {
 let packet=ag_actions_snapshot()
 let held=packet%64
 if held%16!=0 {
  yaw=ag_cam3_yaw()*0.01745329252
  if held==4 {yaw=yaw+3.14159265359}
  if held==2 {yaw=yaw+1.57079632679}
  if held==1 {yaw=yaw-1.57079632679}
 }
 ok=ag_visual_pose(0,1,2,0.19,0,yaw)
}
"#;
    // held 变量需声明：修正为完整源（yaw/ok 声明）。
    let facing_control = facing_control.replace("var ok=0\n", "var ok=0\nvar yaw=0.0\n");
    start_control(&mut h, &facing_control, "facing");
    let vp = fixed_view();
    let light = fixed_light();
    let target = OffscreenTarget::new(h.rt.gpu(), TARGET as u32, TARGET as u32).expect("target");
    let mut pipeline = MeshPipeline::new(h.rt.gpu(), OffscreenTarget::FORMAT).expect("pipeline");
    let mesh_count = a.model.meshes.len();
    let cases: [(u32, f32, f32); 7] = [
        (4, 30.0, -150.0),
        (8, 0.0, 30.0),
        (2, 0.0, 120.0),
        (1, 0.0, -60.0),
        (0, 0.0, -60.0),
        (8, 149.9, 179.9),
        (8, 0.2, -179.9),
    ];
    let mut prior_rgba: Option<Vec<u8>> = None;
    for (i, (held, drag, expect_deg)) in cases.iter().enumerate() {
        input(&mut h, *held, *drag, true);
        let (clip, _phase, _looping, yaw) = visual_of(&h, 1);
        assert_eq!(clip, 2, "case{i}: walk pose");
        let expected = expect_deg.to_radians();
        let err = (yaw - expected).abs();
        let err = err.min((err - std::f32::consts::TAU).abs());
        assert!(err < 1e-5, "case{i}: yaw {} vs expected {}", yaw, expected);
        // GPU 轮廓（期望朝向的独立f64 mask）。
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap().clone();
        let entity = interaction::transform(&frame.objects[0]) * Mat4::from_rotation_y(expected);
        let triangles = actor_triangles(&a, &j, &bin, interaction::transform(&frame.objects[0]), 2, 0.19, expected);
        let _ = entity;
        let content = match h.rt.content(h.three).expect("content") {
            agp::host::surface_runtime::SurfaceContent::Scene3d(s) => s,
            _ => panic!("scene3d"),
        };
        pipeline.render(h.rt.gpu(), target.view(), TARGET as u32, TARGET as u32, [1., 0., 1., 1.], &vp, &light, &content.store, &content.instances[..mesh_count], false);
        let shot = capture_png(h.rt.gpu(), &target).expect("shot").1;
        let rgba = shot.into_raw();
        let m = mask(&triangles, vp);
        let (_, errors) = silhouette_errors(&m, &rgba);
        assert!(errors <= 2, "facing{i}: silhouette misses {errors}");
        // 负控：错yaw（0.7 rad 前乘）必须 >2。
        let wrong: Vec<MeshInstance> = content.instances[..mesh_count]
            .iter()
            .map(|x| MeshInstance { mesh: x.mesh.clone(), model: Mat4::from_rotation_y(0.7) * x.model })
            .collect();
        pipeline.render(h.rt.gpu(), target.view(), TARGET as u32, TARGET as u32, [1., 0., 1., 1.], &vp, &light, &content.store, &wrong, false);
        let wrong_shot = capture_png(h.rt.gpu(), &target).expect("shot").1;
        let (_, wrong_errors) = silhouette_errors(&m, &wrong_shot.into_raw());
        if i < 4 {
            assert!(wrong_errors > 2, "facing{i}: wrong-yaw control must fail");
        }
        if i == 4 {
            assert_eq!(
                prior_rgba.as_ref().unwrap().as_slice(),
                &rgba[..],
                "stopped retains facing pixels"
            );
        }
        prior_rgba = Some(rgba);
    }
    stop(&mut h);
}

// ---------------- 八项之三：phase_main_alt_frozen（作者夹具） ----------------

#[test]
fn g10b_phase_main_alt_frozen_authored_control() {
    // 每源独立宿主一局12拍：main=0.2、alt=0.4（2×速率）、frozen=0.0。
    let mut phases = vec![];
    let mut images = vec![];
    for (product, label) in [
        ("g10-animation", "main"),
        ("g10-animation-alt", "alt"),
        ("g10-animation-frozen", "frozen"),
    ] {
        let dir = materialize(product, &format!("phase-{label}"));
        let mut h = load_dir(&dir);
        start_bound(&mut h, label);
        for _ in 0..12 {
            input(&mut h, 0, 0.0, true);
        }
        phases.push(visual_of(&h, 1).1);
        images.push(capture(&mut h, label));
        stop(&mut h);
    }
    assert!((phases[0] - 0.2).abs() < 1e-5, "main 12 ticks = 0.2, got {}", phases[0]);
    assert!((phases[1] - 0.4).abs() < 1e-5, "alt 12 ticks = 0.4, got {}", phases[1]);
    assert_eq!(phases[2], 0.0, "frozen phase constant 0");
    assert_ne!(images[0], images[1], "main vs alt pixels differ");
    assert_ne!(images[0], images[2], "main vs frozen pixels differ");
}

// ---------------- 八项之四：pose_shadow_physics_independent（240Tick trace） --

#[test]
fn g10b_pose_physics_independent_240_tick_trace() {
    let mut traces = vec![];
    for product in ["g10-animation", "g10-animation-neutral"] {
        let dir = materialize(product, &format!("trace-{product}"));
        let mut h = load_dir(&dir);
        start_bound(&mut h, product);
        let mut trace = vec![];
        for tick in 0..240i32 {
            let bits: u32 = if tick == 5 {
                2 | 16
            } else if tick < 120 {
                2
            } else {
                1
            };
            let drag = if tick == 50 { 15.0 } else { 0.0 };
            input(&mut h, bits, drag, true);
            let last = h.play.as_ref().unwrap().last.as_ref().unwrap();
            let mut camera = last.camera.clone();
            if let Some(c) = &mut camera {
                c.run_id = 0;
            }
            let mut rounds = last.rounds.clone();
            if let Some(r) = &mut rounds {
                r.run_id = 0;
            }
            trace.push(format!("{:?}|{:?}|{:?}", last.nodes, camera, rounds));
        }
        stop(&mut h);
        traces.push(trace);
    }
    assert_eq!(
        traces[0], traces[1],
        "240-tick same-input physics/camera/events must be identical excluding Run identity"
    );
}

// ---------------- 八项之五：pause_step_stop_fresh_run（动画面） ----------------

#[test]
fn g10b_pause_step_stop_fresh_run_animation() {
    let dir = materialize("g10-animation", "lifecycle-anim");
    let mut h = load_dir(&dir);
    start_bound(&mut h, "anim");
    let initial = capture(&mut h, "initial-static");
    let fresh_run = h.play.as_ref().unwrap().last.as_ref().unwrap().run_id;
    for _ in 0..3 {
        h.frame(None).expect("paused frame");
    }
    assert_eq!(h.play.as_ref().unwrap().last.as_ref().unwrap().tick, 0);
    h.play.as_mut().unwrap().single_step().expect("step");
    settle(&mut h, true);
    let last = h.play.as_ref().unwrap().last.as_ref().unwrap();
    assert_eq!(last.tick, 1);
    assert!((last.animations.as_ref().unwrap().visual_for(1).unwrap().phase - 1.0 / 60.0).abs() < 1e-6, "exact dt");
    stop(&mut h);
    start_bound(&mut h, "fresh");
    assert_ne!(h.play.as_ref().unwrap().last.as_ref().unwrap().run_id, fresh_run);
    assert_eq!(h.play.as_ref().unwrap().last.as_ref().unwrap().tick, 0);
    assert_eq!(capture(&mut h, "fresh-static"), initial, "fresh initial pixels");
    // 保存/搬移/重开（八项之八）。
    let saved = dir.parent().unwrap().join("save-a");
    let _ = std::fs::remove_dir_all(&saved);
    std::fs::create_dir_all(&saved).expect("save dir");
    copy_dir(&dir, &saved);
    let result = h
        .storage
        .save(&mut h.editor, &saved.join("scene.agscene"), Fault::None)
        .expect("save");
    assert!(matches!(result, Saved::Confirmed));
    stop(&mut h);
    drop(h);
    let moved = dir.parent().unwrap().join("save-b");
    let _ = std::fs::remove_dir_all(&moved);
    std::fs::rename(&saved, &moved).expect("cold move");
    let mut h = load_dir(&moved);
    start_bound(&mut h, "reopened");
    assert_eq!(capture(&mut h, "reopened-static"), initial, "reopened fresh static");
    input(&mut h, 2, 0.0, true);
    assert_eq!(visual_of(&h, 1).0, 2, "walk input after reopen");
    stop(&mut h);
}

// ---------------- 八项之六：resource_union_budget ----------------

#[test]
fn g10b_resource_union_budget_eight_identities() {
    let anim_dir = materialize("g10-animation", "union-anim");
    let tex_dir = materialize("g10-textures", "union-tex");
    let mut h = load_dir(&anim_dir);
    start_bound(&mut h, "union");
    let snapshot = h.resource_snapshot();
    let uploads = snapshot["mesh_uploads"].clone();
    let textures = snapshot["model_texture_uploads"].clone();
    for _ in 0..60 {
        input(&mut h, 4, 0.0, true);
    }
    let budget = h
        .play
        .as_ref()
        .unwrap()
        .callback_high_water()
        .expect("callback high water");
    assert!(budget.0 <= 100000, "callback steps {}", budget.0);
    assert!(budget.1 <= 121, "callback slices {}", budget.1);
    let after = h.resource_snapshot();
    assert_eq!(after["mesh_uploads"], uploads, "warm mesh uploads delta 0");
    assert_eq!(
        after["model_texture_uploads"], textures,
        "warm texture uploads delta 0"
    );
    // 两工程并集 + alt 闭包。
    let (tex_doc, tex_assets, _, _) =
        Storage::load_project(&tex_dir.join("scene.agscene"), &h.editor.assets)
            .expect("textures into animation registry");
    let (anim_doc, mut union, _, _) =
        Storage::load_project(&anim_dir.join("scene.agscene"), &tex_assets).expect("union");
    for asset in tex_assets.values() {
        assets::insert(&mut union, asset.clone())
            .unwrap_or_else(|e| panic!("union insert: {e}"));
    }
    // alt 闭包（同根异图，checker-alt）。
    let original = union[&tex_doc.assets[0].key()].clone();
    let resident_alt = anim_dir.parent().unwrap().join("resident-alt");
    let _ = std::fs::remove_dir_all(&resident_alt);
    std::fs::create_dir_all(resident_alt.join("Textures")).expect("alt dir");
    std::fs::write(resident_alt.join("model.glb"), &*original.bytes).expect("model");
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../autographics-node2d-pilot/fixtures/node3d-presentation/g10-textures/uv/checker-alt.png"),
        resident_alt.join("Textures/colormap.png"),
    )
    .expect("alt image");
    let alt_asset = assets::import(&resident_alt.join("model.glb"), &union).expect("alt import");
    let mut d = tex_doc.clone();
    let mut alt_object = d.objects[0].clone();
    alt_object.id = 102;
    alt_object.resource = Some(alt_asset.identity.key());
    alt_object.translation[0] = 30.;
    d.objects.push(alt_object);
    d.assets.push(alt_asset.identity.clone());
    assets::insert(&mut union, alt_asset).expect("union alt");
    let union_assets = union.values().count();
    assert_eq!(union_assets, 6, "4 texture roots + character + alt closure");
    for a in &anim_doc.assets {
        d.assets.push(a.clone());
    }
    let mut actor = anim_doc.objects[0].clone();
    actor.id = 100;
    actor.translation[0] = 20.;
    let mut floor = anim_doc.objects[1].clone();
    floor.id = 101;
    d.objects.push(actor);
    d.objects.push(floor);
    d.behavior = None;
    let mut union_host = Host::with_assets(d, union).expect("union host");
    union_host.frame(None).expect("frame");
    let residency = union_host.resource_snapshot()["asset_union"].clone();
    assert_eq!(residency["count"], 8, "8 raw identities (4 texture roots + 2 animation + character + alt image union)");
    assert_eq!(residency["raw_bytes"], 150787, "exact union bytes");
    assert!(residency["cpu_total_with_pose_bytes"].as_u64().unwrap_or(u64::MAX) <= 67108864);
    assert!(residency["gpu_expected_bytes"].as_u64().unwrap_or(u64::MAX) <= 67108864);
    stop(&mut union_host);
    stop(&mut h);
}
