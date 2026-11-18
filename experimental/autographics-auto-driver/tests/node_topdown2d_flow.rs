//! PLAN-058 T-03 permanent flow: the public render-only row layer and the
//! visible/blocking query separation (V-S08) on the frozen queries-2d profile.
//!
//! Frozen fixture: `experimental/autographics-node2d-pilot/fixtures/plan058/
//! topdown-ground/` (hand-frozen, NOT generator output): 7 ordered rows =
//! spectator (character row), ground (`"layer": "render-only"`), player,
//! two obstacles, two same-definition targets. One logical owner table drives
//! both halves: the accepted document renders every row, and
//! `spatial::Descriptor` writes the query rows the host sends to the frozen
//! v3/v4 units.
//!
//! V-S08 positive: the ground is drawn (real offscreen GPU pixels) and can
//! never be hit or selected by sweep/segment; the player traverses it and
//! stops at obstacle_a; the nearest same-def target publishes alone.
//! V-S08 mutation: removing the layer (legacy collision) reproduces 055
//! exactly (t=0, start_overlap, no movement, ground wins the nearest query)
//! while all non-ground query rows and every render field stay identical.
//!
//! Run single-threaded:
//!   cargo test --locked --offline --test node_topdown2d_flow -- --test-threads=1
use autographics_auto_driver::{
    authoring::{
        behavior::{Api, SourceSnapshot},
        document::Document,
        document::Layer,
        hierarchy,
        host::Host,
        play::{Action, Frame, State},
        spatial::Descriptor,
        storage::Storage,
    },
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use glam::Vec2;
use png::{BitDepth, ColorType, Decoder};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const SCENE: &str =
    include_str!("../../autographics-node2d-pilot/fixtures/plan058/topdown-ground/scene.agscene");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/fixtures/plan058/topdown-ground/world.at");
const DT: f32 = 1.0 / 60.0;
const PLAYER_STOP: f32 = 8.58;

/// Probe getters ride the combined managed source (the user region cannot name
/// trusted globals), exactly like the frozen PLAN-021 harness.
const PROBES: &str = r#"
fn probe_credit() int {return _ag_spatial_calls}
fn probe_shit() int {return g_shit}
fn probe_snode() int {return g_snode}
fn probe_st() float {return g_st}
fn probe_sov() int {return g_sov}
fn probe_uhit() int {return g_uhit}
fn probe_unode() int {return g_unode}
fn probe_ut() float {return g_ut}
fn probe_px() float {return g_px}
fn probe_tav() int {return g_tav}
fn probe_tbv() int {return g_tbv}
"#;

/// Adversarial query world: two rays around the rejected E1 proxy point and the
/// moved ground box. `-1` ignore is the frozen "no ignore row" form.
const WORLD_ADV: &str = r#"
var ok=0.0
var g_hit1=0
var g_node1=-1
var g_t1=0.0
var g_hit2=0
var g_node2=-1
var g_t2=0.0
fn main_lifetick(dt float) {
 let a = ag_cast_segment(2, -1, 1, 65536.0, 65536.0, 0.0, 65536.0, 65536.1, 0.0)
 g_hit1 = ag_cast_hit(a)
 g_node1 = ag_cast_node(a)
 g_t1 = ag_cast_t(a)
 let b = ag_cast_segment(2, -1, 1, 65540.0, 65530.0, 0.0, 65540.0, 65560.0, 0.0)
 g_hit2 = ag_cast_hit(b)
 g_node2 = ag_cast_node(b)
 g_t2 = ag_cast_t(b)
}
fn app_start() {
 ok=ok+ag_char_bind(1)*1.0
 ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)*1.0
}
fn app_update(dt float) {
 main_lifetick(dt)
}
"#;

const PROBES_ADV: &str = r#"
fn probe_credit() int {return _ag_spatial_calls}
fn probe_hit1() int {return g_hit1}
fn probe_node1() int {return g_node1}
fn probe_t1() float {return g_t1}
fn probe_hit2() int {return g_hit2}
fn probe_node2() int {return g_node2}
fn probe_t2() float {return g_t2}
"#;

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn artifacts() -> PathBuf {
    let path = PathBuf::from(
        std::env::var("PLAN058_ARTIFACTS")
            .unwrap_or_else(|_| "D:/autostack/plan-artifacts/plan-058/auto-ui".into()),
    );
    fs::create_dir_all(&path).expect("create plan058 artifacts dir");
    path
}

fn srgb_encode(v: f64) -> u8 {
    let c = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (c.mul_add(255.0, 0.5).floor() as i32).clamp(0, 255) as u8
}

struct Image {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn decode_png(bytes: &[u8]) -> Image {
    let decoder = Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().expect("png info");
    let mut buffer = vec![0; reader.output_buffer_size().expect("png size")];
    let info = reader.next_frame(&mut buffer).expect("png frame");
    assert_eq!(info.color_type, ColorType::Rgba, "RGBA8 capture");
    assert_eq!(info.bit_depth, BitDepth::Eight, "RGBA8 capture");
    buffer.truncate(info.buffer_size());
    Image {
        width: info.width,
        height: info.height,
        rgba: buffer,
    }
}

fn pixel(image: &Image, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * image.width + x) * 4) as usize;
    image.rgba[at..at + 4].try_into().unwrap()
}

// ------------------------------------------------------------ fixture ------

/// Stage the frozen fixture under target/ exactly like the frozen PLAN-021
/// harness: scene.agscene + scripts/<bound sha>.at (no external asset).
fn stage(tag: &str, scene: &str, world: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan058-t03")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("scripts")).expect("scripts dir");
    fs::write(dir.join("scene.agscene"), scene).expect("scene");
    let bound = serde_json::from_str::<serde_json::Value>(scene)
        .ok()
        .and_then(|v| {
            v["behavior"]["source"]["sha256"]
                .as_str()
                .map(str::to_string)
        })
        .expect("scene binds a source sha");
    assert_eq!(
        bound,
        sha(world.as_bytes()),
        "scene binds the world source sha"
    );
    fs::write(dir.join("scripts").join(format!("{bound}.at")), world).expect("world");
    dir
}

fn pump(host: &mut Host) {
    let deadline = Instant::now() + Duration::from_secs(31);
    while host.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < deadline, "host settle deadline");
        host.frame(None).expect("frame");
    }
}

fn start(tag: &str, scene: &str) -> Host {
    let dir = stage(tag, scene, WORLD);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Default::default())
            .unwrap_or_else(|e| panic!("{tag} load: {e:?}"));
    let code = code.unwrap_or_else(|| panic!("{tag} behavior code"));
    let mut host =
        Host::with_project(doc, registry, Some(code)).unwrap_or_else(|e| panic!("{tag} host: {e}"));
    host.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start: {e:?}"));
    host.pause_play("");
    pump(&mut host);
    if let Some(err) = host.play.as_ref().and_then(|p| p.error.clone()) {
        assert!(err.is_empty(), "{tag} start fault: {err}");
    }
    assert_eq!(
        host.play.as_ref().map(|p| p.state),
        Some(State::Paused),
        "{tag} settled Paused"
    );
    host
}

fn step(host: &mut Host) {
    {
        let play = host.play.as_mut().expect("run");
        play.resume().expect("resume");
        play.dispatch(Action {
            space: 0,
            x: 0,
            y: 0,
            toggle: false,
        })
        .expect("dispatch");
    }
    host.pause_play("");
    pump(host);
    assert_eq!(
        host.play.as_ref().map(|p| p.state),
        Some(State::Paused),
        "tick settled Paused"
    );
    if let Some(err) = host.play.as_ref().and_then(|p| p.error.clone()) {
        assert!(err.is_empty(), "tick fault: {err}");
    }
}

fn frame(host: &Host) -> &Frame {
    host.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .expect("accepted frame")
}

fn screen_of(host: &Host, world: Vec2) -> Vec2 {
    let f = frame(host);
    let cam = f.camera.as_ref().expect("camera in frame");
    let actor = f
        .nodes
        .get(&cam.target_id)
        .expect("camera target node")
        .translation;
    let centre = cam
        .follow_center_2d(Vec2::new(actor[0], actor[1]))
        .expect("follow centre");
    cam.screen_2d(world, centre).expect("screen projection")
}

fn rect_of(host: &Host, pos: [f32; 2], half: [f32; 2]) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for p in [
        [pos[0] - half[0], pos[1] - half[1]],
        [pos[0] - half[0], pos[1] + half[1]],
        [pos[0] + half[0], pos[1] - half[1]],
        [pos[0] + half[0], pos[1] + half[1]],
    ] {
        let s = screen_of(host, Vec2::new(p[0], p[1]));
        min = min.min(s);
        max = max.max(s);
    }
    (min, max)
}

fn quad_class(rgba: [f32; 4]) -> [u8; 3] {
    [
        srgb_encode(f64::from(rgba[0])),
        srgb_encode(f64::from(rgba[1])),
        srgb_encode(f64::from(rgba[2])),
    ]
}

fn classify(p: [u8; 4], palette: &[[u8; 3]]) -> Option<usize> {
    let mut best = None;
    let mut bestd = 20i32 * 20;
    for (i, c) in palette.iter().enumerate() {
        let d = (i32::from(p[0]) - i32::from(c[0])).pow(2)
            + (i32::from(p[1]) - i32::from(c[1])).pow(2)
            + (i32::from(p[2]) - i32::from(c[2])).pow(2);
        if d < bestd {
            bestd = d;
            best = Some(i);
        }
    }
    best
}

/// (key, pos, half, rgba, drawn-after-tick) of the six visible fixture rows.
fn visible_rows(player_x: f32) -> Vec<(&'static str, [f32; 2], [f32; 2], [f32; 4], bool)> {
    vec![
        (
            "ground",
            [8.0, 0.0],
            [6.0, 4.0],
            [0.18, 0.45, 0.22, 1.0],
            true,
        ),
        (
            "player",
            [player_x, 0.0],
            [0.4, 0.5],
            [0.9, 0.3, 0.2, 1.0],
            true,
        ),
        (
            "obstacle_a",
            [10.0, 0.0],
            [1.0, 3.0],
            [0.55, 0.55, 0.6, 1.0],
            true,
        ),
        (
            "obstacle_b",
            [4.0, -2.5],
            [1.0, 1.0],
            [0.35, 0.35, 0.4, 1.0],
            true,
        ),
        (
            "target_a",
            [4.0, 2.0],
            [0.5, 0.5],
            [1.0, 0.55, 0.15, 1.0],
            true,
        ),
        (
            "target_b",
            [4.0, 3.25],
            [0.5, 0.5],
            [0.55, 0.15, 0.8, 1.0],
            false,
        ),
    ]
}

fn capture(host: &mut Host, name: &str) -> Image {
    let (sha_hex, bytes, w, h) = host.rt.capture(host.two, name).expect("capture");
    fs::write(artifacts().join(format!("{name}.png")), &bytes).expect("write png");
    println!("{name}: capture {w}x{h} sha={sha_hex}");
    let image = decode_png(&bytes);
    assert_eq!((image.width, image.height), (w, h), "{name} layout");
    image
}

struct QuadStat {
    key: &'static str,
    count: usize,
    centroid: [f32; 2],
    bbox: [f32; 4],
    want: [f32; 4],
    area: f32,
}

fn quad_stats(host: &Host, image: &Image, player_x: f32) -> Vec<QuadStat> {
    let rows = visible_rows(player_x);
    let palette: Vec<[u8; 3]> = rows.iter().map(|r| quad_class(r.3)).collect();
    let mut counts = vec![0usize; rows.len()];
    let mut sums = vec![[0.0f64; 2]; rows.len()];
    let mut bounds = vec![
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY
        ];
        rows.len()
    ];
    for y in 0..image.height {
        for x in 0..image.width {
            if let Some(i) = classify(pixel(image, x, y), &palette) {
                counts[i] += 1;
                sums[i][0] += f64::from(x);
                sums[i][1] += f64::from(y);
                let (fx, fy) = (x as f32, y as f32);
                bounds[i][0] = bounds[i][0].min(fx);
                bounds[i][1] = bounds[i][1].min(fy);
                bounds[i][2] = bounds[i][2].max(fx);
                bounds[i][3] = bounds[i][3].max(fy);
            }
        }
    }
    let f = frame(host);
    let cam = f.camera.as_ref().expect("camera");
    let actor = f
        .nodes
        .get(&cam.target_id)
        .expect("camera target")
        .translation;
    let centre = cam
        .follow_center_2d(Vec2::new(actor[0], actor[1]))
        .expect("centre");
    let (_, content, bars) = cam.letterbox_2d().expect("letterbox");
    rows.iter()
        .enumerate()
        .map(|(i, (key, pos, half, _, _))| {
            let (min, max) = rect_of(host, *pos, *half);
            let lo = [min.x.max(bars.x), min.y.max(bars.y)];
            let hi = [max.x.min(bars.x + content.x), max.y.min(bars.y + content.y)];
            let want = [lo[0], lo[1], hi[0] - 1.0, hi[1] - 1.0];
            let count = counts[i];
            QuadStat {
                key,
                count,
                centroid: if count == 0 {
                    [f32::NAN, f32::NAN]
                } else {
                    [
                        (sums[i][0] / count as f64) as f32,
                        (sums[i][1] / count as f64) as f32,
                    ]
                },
                bbox: bounds[i],
                want,
                area: (hi[0] - lo[0]).max(0.0) * (hi[1] - lo[1]).max(0.0),
            }
        })
        .collect()
}

fn assert_quad(stat: &QuadStat, image: &Image, drawn: bool, ctx: &str) {
    if !drawn {
        assert_eq!(stat.count, 0, "{ctx}: {} is hidden, no pixels", stat.key);
        return;
    }
    assert!(stat.count > 0, "{ctx}: {} visible", stat.key);
    for axis in 0..4 {
        assert!(
            (stat.bbox[axis] - stat.want[axis]).abs() <= 2.0,
            "{ctx}: {} bbox {:?} vs projected {:?} on {}x{}",
            stat.key,
            stat.bbox,
            stat.want,
            image.width,
            image.height
        );
    }
    // The ground is partly occluded by the rows above it, so its centre of
    // mass is not the rect centre; bbox + coverage are its position oracle.
    if stat.key == "ground" {
        assert!(
            stat.count as f32 >= stat.area * 0.55,
            "{ctx}: ground coverage {} of {}",
            stat.count,
            stat.area
        );
        return;
    }
    let want_centre = |axis: usize| (stat.want[axis] + stat.want[axis + 2]) * 0.5;
    assert!(
        (stat.centroid[0] - want_centre(0)).abs() <= 3.0
            && (stat.centroid[1] - want_centre(1)).abs() <= 3.0,
        "{ctx}: {} centroid {:?} vs projected centre",
        stat.key,
        stat.centroid
    );
    assert!(
        (stat.count as f32 - stat.area) <= stat.area * 0.12,
        "{ctx}: {} area {} vs projected {}",
        stat.key,
        stat.count,
        stat.area
    );
}

// ----------------------------------------------------- query half ---------

/// Build a queries-2d session from the SAME accepted document: the host write
/// pattern of `play.rs` (identity/visibility/fields per ordered row) and the
/// public `Descriptor` rows (which already encode render-only rows as the
/// disjoint proxy).
fn session_from_document(
    doc: &Document,
    world: &str,
    tag: &str,
) -> (Session, Descriptor, Vec<hierarchy::Node>) {
    session_from_document_with(doc, world, PROBES, tag)
}

fn session_from_document_with(
    doc: &Document,
    world: &str,
    probes: &str,
    tag: &str,
) -> (Session, Descriptor, Vec<hierarchy::Node>) {
    let descriptor = Descriptor::build(doc, |_| None).expect("descriptor");
    let order = hierarchy::ordered(doc).expect("order");
    let snap =
        SourceSnapshot::from_bytes_for(Path::new(tag), world.as_bytes().to_vec(), Api::Topdown2V1)
            .unwrap_or_else(|e| panic!("{tag} admission: {e}"));
    let mut combined = snap.source.clone();
    combined.push_str(probes);
    let mut s = Session::new(&combined).unwrap_or_else(|e| panic!("{tag} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call(
        "_ag_spatial_reset",
        &[Int(order.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (i, n) in order.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[
                Int(i as i32 + 1),
                Int(n.kind),
                Int(n.dimension.code()),
                Int(n
                    .parent
                    .and_then(|p| order.iter().position(|x| x.id == p))
                    .map_or(-1, |v| v as i32)),
            ],
            ResultKind::Unit,
        )
        .unwrap();
        s.call(
            "_ag_visibility",
            &[Int(i32::from(n.visible))],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [
            n.local[0], n.local[1], n.local[2], 1., 1., 1., 0., 0., 1., 1., 1., 0.,
        ] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
    }
    for (index, value) in descriptor.rows().iter().flatten().enumerate() {
        s.call(
            "_ag_spatial_config",
            &[Int(index as i32), Float(*value)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    for n in 0..order.len() {
        let id = n as i32 + 1;
        s.call("_ag_char_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_obj_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_trg_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
    }
    // The topdown2d host writes one render-only flag per ordered row before
    // `_ag_start` (0 credit; the row layers are the document's own).
    s.call("_ag_ro_reset", &[Int(order.len() as i32)], ResultKind::Unit)
        .unwrap();
    for i in 0..order.len() {
        let flag = i32::from(descriptor.row_layer(i) == Some(Layer::RenderOnly));
        s.call("_ag_ro_set", &[Int(i as i32), Int(flag)], ResultKind::Unit)
            .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "{tag} start green");
    (s, descriptor, order)
}

/// Independent f32 candidate boxes: only rows in the PUBLIC candidate table,
/// with the row's real document geometry (never the proxy value).
fn candidate_boxes(doc: &Document, descriptor: &Descriptor) -> Vec<(usize, [f32; 2], [f32; 2])> {
    let order = hierarchy::ordered(doc).expect("order");
    descriptor
        .query_candidates()
        .into_iter()
        .map(|i| {
            let id = order[i].id;
            let o = doc.objects.iter().find(|o| o.id == id).expect("row object");
            (
                i,
                [
                    o.translation[0] - o.scale[0] * 0.5,
                    o.translation[1] - o.scale[1] * 0.5,
                ],
                [
                    o.translation[0] + o.scale[0] * 0.5,
                    o.translation[1] + o.scale[1] * 0.5,
                ],
            )
        })
        .collect()
}

fn oracle_segment(
    boxes: &[(usize, [f32; 2], [f32; 2])],
    ignore: usize,
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
) -> (i32, i32, f32) {
    let (dx, dy) = (bx - ax, by - ay);
    let mut target: i32 = -1;
    let mut fraction: f32 = 2.0;
    for &(ord, lo, hi) in boxes {
        if ord == ignore {
            continue;
        }
        let mut enter: f32 = 0.0;
        let mut leave: f32 = 1.0;
        if dx == 0.0 {
            if ax < lo[0] || ax > hi[0] {
                continue;
            }
        } else {
            let near = (lo[0] - ax) / dx;
            let far = (hi[0] - ax) / dx;
            if near > enter {
                enter = near;
            }
            if far < leave {
                leave = far;
            }
            if enter > leave {
                continue;
            }
        }
        if dy == 0.0 {
            if ay < lo[1] || ay > hi[1] {
                continue;
            }
        } else {
            let near = (lo[1] - ay) / dy;
            let far = (hi[1] - ay) / dy;
            if near > enter {
                enter = near;
            }
            if far < leave {
                leave = far;
            }
            if enter > leave {
                continue;
            }
        }
        if enter >= fraction {
            continue;
        }
        target = ord as i32;
        fraction = enter;
    }
    if target >= 0 {
        (1, target, fraction)
    } else {
        (0, -1, 1.0)
    }
}

fn oracle_sweep(
    boxes: &[(usize, [f32; 2], [f32; 2])],
    mover: usize,
    half: [f32; 2],
    ax: f32,
    ay: f32,
    dx: f32,
) -> (i32, i32, f32, i32) {
    let mut target: i32 = -1;
    let mut fraction: f32 = 2.0;
    for &(ord, lo, hi) in boxes {
        if ord == mover {
            continue;
        }
        if ay < lo[1] - half[1] || ay > hi[1] + half[1] {
            continue;
        }
        let l = lo[0] - half[0];
        let h = hi[0] + half[0];
        let mut enter: f32 = 0.0;
        let mut leave: f32 = 1.0;
        if dx == 0.0 {
            if ax < l || ax > h {
                continue;
            }
        } else {
            let near = (l - ax) / dx;
            let far = (h - ax) / dx;
            if near > enter {
                enter = near;
            }
            if far < leave {
                leave = far;
            }
            if enter > leave {
                continue;
            }
        }
        if enter >= fraction {
            continue;
        }
        target = ord as i32;
        fraction = enter;
    }
    if target < 0 {
        return (0, -1, 1.0, 0);
    }
    let (_, lo, hi) = boxes
        .iter()
        .find(|b| b.0 as i32 == target)
        .expect("target box");
    let l = lo[0] - half[0];
    let h = hi[0] + half[0];
    let mut nx = 0.0f32;
    if ax < l || ax > h {
        if dx >= 0.0 && (l - ax) / dx == fraction {
            nx = -1.0;
        } else if dx < 0.0 && (h - ax) / dx == fraction {
            nx = 1.0;
        }
    }
    (1, target, fraction, i32::from(nx == 0.0))
}

fn tick(s: &mut Session, step: i32) {
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

// ------------------------------------------------------------- tests ------

/// V-S08 positive: the same accepted document both renders (real offscreen
/// GPU) and drives the frozen query units; the render-only ground is drawn but
/// never hit, never selected, and never blocks the corridor.
#[test]
fn vs08_ground_renders_but_is_never_a_query_candidate() {
    let t0 = Instant::now();
    let mut host = start("vs08-pos", SCENE);
    assert_eq!(frame(&host).tick, 0, "start frame before any Tick");
    let before = capture(&mut host, "t03-vs08-before");
    let before_stats = quad_stats(&host, &before, 4.0);
    for stat in &before_stats {
        assert_quad(stat, &before, true, "pre-tick");
    }
    let ground = before_stats.iter().find(|s| s.key == "ground").unwrap();
    assert!(
        ground.count > 800 * 380 / 2,
        "ground really drawn ({} px)",
        ground.count
    );

    step(&mut host);
    assert_eq!(frame(&host).tick, 1, "one Tick");
    let after = capture(&mut host, "t03-vs08-after");
    let after_stats = quad_stats(&host, &after, PLAYER_STOP);
    for stat in &after_stats {
        let drawn = visible_rows(PLAYER_STOP)
            .iter()
            .find(|r| r.0 == stat.key)
            .map(|r| r.4)
            .expect("row");
        assert_quad(stat, &after, drawn, "post-tick");
    }
    let player = frame(&host).nodes.get(&3).expect("player node");
    assert!(
        (player.translation[0] - PLAYER_STOP).abs() <= 2e-3,
        "player stopped before obstacle_a: {}",
        player.translation[0]
    );
    assert!(
        frame(&host).nodes.get(&7).expect("target_b").visible == false,
        "far target unpublished after the nearest-target tick"
    );
    assert!(
        frame(&host).nodes.get(&6).expect("target_a").visible,
        "nearest target published"
    );

    // Query half: the SAME document's descriptor rows → frozen v3/v4 units.
    let doc = Document::decode(SCENE.as_bytes()).expect("fixture document");
    let (mut s, descriptor, order) = session_from_document(&doc, WORLD, "vs08-pos.at");
    // The public candidate table excludes the render-only ground (ordinal 1).
    assert_eq!(order.len(), 7, "seven ordered rows");
    assert_eq!(
        descriptor.row_layer(1),
        Some(autographics_auto_driver::authoring::document::Layer::RenderOnly)
    );
    assert!(
        !descriptor.is_query_candidate(1),
        "ground is not a candidate"
    );
    assert_eq!(
        descriptor.query_candidates(),
        vec![0, 2, 3, 4, 5, 6],
        "candidate table = all rows except the render-only ground"
    );
    assert_eq!(
        descriptor.rows()[1],
        [1.0, -6.0, -4.0, 0.0, 6.0, 4.0, 0.0],
        "render-only row keeps its real local half-extent box (no proxy)"
    );
    let boxes = candidate_boxes(&doc, &descriptor);
    let player_row = 2usize;
    let half = [0.4, 0.5];
    let (hit, node, t, overlap) = oracle_sweep(&boxes, player_row, half, 4.0, 0.0, 100.0);
    assert_eq!((hit, node), (1, 3), "sweep hits obstacle_a (ordinal 3)");
    assert_eq!(overlap, 0, "no start overlap: ground is not a candidate");
    tick(&mut s, 1);
    assert_eq!(
        s.int("_ag_status", &[]).unwrap(),
        0,
        "queries-2d tick green"
    );
    assert_eq!(
        s.int("probe_credit", &[]).unwrap(),
        32,
        "sweep 16 + 5 getters + segment 8 + 3 getters = 32 exactly"
    );
    assert_eq!(s.int("probe_snode", &[]).unwrap(), node);
    assert_ne!(s.int("probe_snode", &[]).unwrap(), 1, "ground never hit");
    assert_eq!(s.float("probe_st", &[]).unwrap(), t);
    assert_eq!(s.int("probe_sov", &[]).unwrap(), overlap);
    let (uhit, unode, ut) = oracle_segment(&boxes, player_row, 4.0, 0.0, 4.0, 6.0);
    assert_eq!((uhit, unode), (1, 5), "segment picks target_a (ordinal 5)");
    assert_eq!(s.int("probe_unode", &[]).unwrap(), unode);
    assert_ne!(
        s.int("probe_unode", &[]).unwrap(),
        1,
        "ground never selected"
    );
    assert_eq!(s.float("probe_ut", &[]).unwrap(), ut);
    // Only the nearest same-definition target publishes; the ground row's own
    // document visibility is untouched by the query half.
    assert_eq!(s.int("probe_tav", &[]).unwrap(), 1);
    assert_eq!(s.int("probe_tbv", &[]).unwrap(), 0);
    assert_eq!(
        s.float("probe_px", &[]).unwrap(),
        PLAYER_STOP,
        "the row moved to the sweep stop (ground did not block)"
    );
    println!(
        "PLAN058_TIMING test=vs08_ground_renders_but_is_never_a_query_candidate ms={} stop={} t={} node={} unode={} ut={} credits=32 ground_px={}",
        t0.elapsed().as_millis(),
        PLAYER_STOP,
        t,
        node,
        unode,
        ut,
        ground.count
    );
}

/// V-S08 mutation: removing the layer (legacy collision) reproduces 055 —
/// the ground wins with t=0/start_overlap and the player cannot move — while
/// every non-ground query row and every render field stay identical.
#[test]
fn vs08_legacy_ground_collision_reproduces_055_t0() {
    let t0 = Instant::now();
    let doc = Document::decode(SCENE.as_bytes()).expect("fixture document");
    let (mut pos, positive, order) = session_from_document(&doc, WORLD, "vs08-mut-pos.at");
    tick(&mut pos, 1);
    assert_eq!(pos.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(pos.int("probe_snode", &[]).unwrap(), 3);
    assert_eq!(pos.float("probe_px", &[]).unwrap(), PLAYER_STOP);

    // The only change: the ground row loses its `layer` (legacy collision).
    let legacy: serde_json::Value = serde_json::from_str(SCENE).expect("scene json");
    let mut objects = legacy["objects"].as_array().expect("objects").clone();
    for o in objects.iter_mut() {
        if o["id"] == 2 {
            o.as_object_mut().expect("object").remove("layer");
        }
    }
    let mut mutated = legacy.clone();
    mutated["objects"] = serde_json::Value::Array(objects);
    let doc2 = Document::decode(mutated.to_string().as_bytes()).expect("legacy document");
    let rows2 = Descriptor::build(&doc2, |_| None).expect("legacy descriptor");
    // Appearance rows untouched: everything except the ground row is identical.
    let rows_a = positive.rows();
    let rows_b = rows2.rows();
    assert_eq!(rows_a.len(), rows_b.len());
    for i in 0..rows_a.len() {
        if i == 1 {
            continue;
        }
        assert_eq!(rows_a[i], rows_b[i], "row {i} unchanged by the layer");
    }
    assert_eq!(
        rows_b[1],
        [1.0, -6.0, -4.0, 0.0, 6.0, 4.0, 0.0],
        "legacy ground keeps the same local box as the render-only row"
    );
    assert!(rows2.is_query_candidate(1), "legacy ground is a candidate");
    // Render fields are untouched too (same document objects minus the layer).
    assert_eq!(doc.objects.len(), doc2.objects.len());
    for (a, b) in doc.objects.iter().zip(doc2.objects.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.name, b.name);
        assert_eq!(a.translation, b.translation);
        assert_eq!(a.scale, b.scale);
        assert_eq!(a.rgba, b.rgba);
        assert_eq!(a.visible, b.visible);
    }

    let (mut mut_s, mut_rows, _) = session_from_document(&doc2, WORLD, "vs08-mut.at");
    tick(&mut mut_s, 1);
    assert_eq!(
        mut_s.int("_ag_status", &[]).unwrap(),
        0,
        "mutation still runs"
    );
    let boxes = candidate_boxes(&doc2, &mut_rows);
    let (hit, node, t, overlap) = oracle_sweep(&boxes, 2, [0.4, 0.5], 4.0, 0.0, 100.0);
    assert_eq!(hit, 1);
    assert_eq!(node, 1, "ground wins the sweep");
    assert_eq!(t, 0.0, "055 start-overlap t=0");
    assert_eq!(overlap, 1, "055 start_overlap reproduced");
    assert_eq!(mut_s.int("probe_snode", &[]).unwrap(), 1);
    assert_eq!(mut_s.float("probe_st", &[]).unwrap(), 0.0);
    assert_eq!(mut_s.int("probe_sov", &[]).unwrap(), 1);
    assert_eq!(
        mut_s.float("probe_px", &[]).unwrap(),
        4.0,
        "055 no-move: the visible ground blocks the row"
    );
    let (uhit, unode, ut) = oracle_segment(&boxes, 2, 4.0, 0.0, 4.0, 6.0);
    assert_eq!(
        (uhit, unode),
        (1, 1),
        "ground misselected as nearest target"
    );
    assert_eq!(ut, 0.0);
    assert_eq!(mut_s.int("probe_unode", &[]).unwrap(), 1);

    // Restore the correct layer → green again (same oracle, same source).
    let (mut again, _, _) = session_from_document(&doc, WORLD, "vs08-mut-restored.at");
    tick(&mut again, 1);
    assert_eq!(again.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(again.int("probe_snode", &[]).unwrap(), 3);
    assert_eq!(again.int("probe_unode", &[]).unwrap(), 5);
    assert_eq!(again.float("probe_px", &[]).unwrap(), PLAYER_STOP);
    println!(
        "PLAN058_TIMING test=vs08_legacy_ground_collision_reproduces_055_t0 ms={} mutated_node={} mutated_t=0 restored_node=3",
        t0.elapsed().as_millis(),
        node
    );
    let _ = order;
}

/// Old-profile identity guard: a legacy document (no layer field) serializes
/// and hashes exactly as before, keeps every row a candidate, and the frozen
/// profile/version ids are unchanged.
#[test]
fn legacy_document_serialization_hash_and_profile_ids_unchanged() {
    let t0 = Instant::now();
    const LEGACY: &str =
        include_str!("../../autographics-node2d-pilot/generated-queries2d/q-sweep/scene.agscene");
    let doc = Document::decode(LEGACY.as_bytes()).expect("legacy scene");
    let encoded = doc.encode().expect("encode");
    assert!(
        !String::from_utf8_lossy(&encoded).contains("\"layer\""),
        "legacy serialization never grows a layer field"
    );
    assert_eq!(
        doc.hash().unwrap(),
        sha861(&encoded),
        "document hash is sha256 of the encoded bytes"
    );
    // Stability: decode(encode(doc)) is the same document and the hash does not
    // move on a re-encode (the optional field is omitted, never defaulted in).
    let again = Document::decode(&encoded).expect("re-decode");
    assert_eq!(again, doc);
    assert_eq!(again.hash().unwrap(), doc.hash().unwrap());
    assert_eq!(
        again.objects.iter().filter(|o| o.layer.is_some()).count(),
        0,
        "every legacy row keeps the implicit collision layer"
    );
    let descriptor = Descriptor::build(&doc, |_| None).expect("legacy descriptor");
    assert_eq!(
        descriptor.query_candidates().len(),
        descriptor.rows().len(),
        "every legacy row stays a candidate"
    );
    for i in 0..descriptor.rows().len() {
        assert!(descriptor.is_query_candidate(i));
        assert_eq!(
            descriptor.row_layer(i),
            Some(autographics_auto_driver::authoring::document::Layer::Collision)
        );
    }
    // Frozen profile identity: objects2d/1 = profile 16 / version 105, and it
    // still admits the frozen product source.
    let api = Api::parse("autographics.objects2d/1").expect("api id");
    assert_eq!(api, Api::Objects2V1);
    assert_eq!(api.profile(), 16);
    assert_eq!(api.number(), 105);
    // Negative control: an unknown layer value is a named schema rejection, not
    // a silently ignored field (`deny_unknown_fields` + closed enum).
    let mut bogus: serde_json::Value = serde_json::from_str(SCENE).expect("scene json");
    bogus["objects"][1]["layer"] = serde_json::Value::String("bogus".into());
    let err = Document::decode(bogus.to_string().as_bytes()).expect_err("bogus layer rejected");
    assert!(
        err.contains("unknown variant") && err.contains("bogus"),
        "named layer rejection: {err}"
    );
    // Render-only on a 3D row is a named document rejection too (fixture rows
    // are all 2D, so flip the flag on a deliberately 3D-ized row copy).
    let mut bad3d: serde_json::Value = serde_json::from_str(SCENE).expect("scene json");
    bad3d["objects"][1]["kind"] = serde_json::Value::String("cylinder".into());
    bad3d["objects"][1]["translation"] = serde_json::json!([8.0, 0.0, 1.0]);
    bad3d["objects"][1]["scale"] = serde_json::json!([1.0, 1.0, 1.0]);
    bad3d["objects"][1]["rgba"] = serde_json::json!([0.9, 0.3, 0.2, 1.0]);
    let err3 = Document::decode(bad3d.to_string().as_bytes())
        .expect_err("render-only on a 3D row rejected");
    assert!(
        err3.contains("render-only layer requires a 2D shape"),
        "named 2D requirement: {err3}"
    );
    println!(
        "PLAN058_TIMING test=legacy_document_serialization_hash_and_profile_ids_unchanged ms={} rows={}",
        t0.elapsed().as_millis(),
        descriptor.rows().len()
    );
}

/// V-S08 adversarial: the rejected E1 proxy is not load-bearing. The ground row
/// is moved to the proxy point (65540, 65540) and two rays are aimed through the
/// old disjoint point and across the moved box: the render-only row is skipped in
/// every case, while the same document without the layer is hit at t=0/t=0.2.
#[test]
fn vs08_adversarial_65536_endpoints_never_hit_render_only_ground() {
    let t0 = Instant::now();
    let mut scene: serde_json::Value = serde_json::from_str(SCENE).expect("scene json");
    scene["objects"][1]["translation"] = serde_json::json!([65540.0, 65540.0, 0.0]);
    let doc = Document::decode(scene.to_string().as_bytes()).expect("moved ground document");
    // Descriptor rows are LOCAL half-extents (the host-write form).
    let ground_box = [1.0, -6.0, -4.0, 0.0, 6.0, 4.0, 0.0];

    let (mut ro, ro_desc, _) =
        session_from_document_with(&doc, WORLD_ADV, PROBES_ADV, "vs08-adv-ro.at");
    assert_eq!(
        ro_desc.rows()[1],
        ground_box,
        "the moved row carries the real 12x8 local box at the proxy point"
    );
    assert!(!ro_desc.is_query_candidate(1), "and it is not a candidate");
    tick(&mut ro, 1);
    assert_eq!(ro.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(
        ro.int("probe_credit", &[]).unwrap(),
        22,
        "two segments + 6 getters = 22 credits"
    );
    assert_eq!(
        ro.int("probe_hit1", &[]).unwrap(),
        0,
        "ray through (65536,65536) misses"
    );
    assert_eq!(ro.int("probe_node1", &[]).unwrap(), -1);
    assert_eq!(
        ro.int("probe_hit2", &[]).unwrap(),
        0,
        "ray across the box misses"
    );
    assert_eq!(ro.int("probe_node2", &[]).unwrap(), -1);

    // The same document without the layer: the real AABB is hit, proving the
    // row is geometrically present and only the layer skip prevents the hit.
    let mut legacy = scene.clone();
    legacy["objects"][1]
        .as_object_mut()
        .expect("ground")
        .remove("layer");
    let doc2 = Document::decode(legacy.to_string().as_bytes()).expect("legacy moved document");
    let (mut old, old_desc, _) =
        session_from_document_with(&doc2, WORLD_ADV, PROBES_ADV, "vs08-adv-old.at");
    assert!(old_desc.is_query_candidate(1));
    tick(&mut old, 1);
    assert_eq!(old.int("_ag_status", &[]).unwrap(), 0);
    assert_eq!(
        old.int("probe_hit1", &[]).unwrap(),
        1,
        "legacy hits at the old proxy point"
    );
    assert_eq!(old.int("probe_node1", &[]).unwrap(), 1);
    assert_eq!(
        old.float("probe_t1", &[]).unwrap(),
        0.0,
        "start inside the moved box"
    );
    assert_eq!(old.int("probe_hit2", &[]).unwrap(), 1);
    assert_eq!(old.int("probe_node2", &[]).unwrap(), 1);
    assert!(
        (old.float("probe_t2", &[]).unwrap() - 0.2).abs() <= 1e-6,
        "entry t 0.2 into the moved box"
    );
    println!(
        "PLAN058_TIMING test=vs08_adversarial_65536_endpoints_never_hit_render_only_ground ms={} ro_hits=(0,0) legacy=(t0,t0.2) credits=22",
        t0.elapsed().as_millis()
    );
}

/// Format/profile isolation: only topdown2d honors `layer`; an old profile
/// rejects a layered document by name, and a pre-/9 schema rejects the field.
#[test]
fn layer_requires_topdown2d_profile_and_scene_9() {
    let t0 = Instant::now();
    let mut legacy_profile: serde_json::Value = serde_json::from_str(SCENE).expect("scene json");
    legacy_profile["behavior"]["api"] =
        serde_json::Value::String("autographics.objects2d/1".into());
    let text = legacy_profile.to_string();
    let dir = stage("vs08-legacy-profile", &text, WORLD);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Default::default())
            .expect("the /9 document itself is valid");
    let err = match Host::with_project(doc, registry, code) {
        Err(e) => e,
        Ok(mut h) => h
            .start_selected_play()
            .expect_err("old profile must reject a layered document"),
    };
    assert!(
        err.contains("layer requires the topdown2d profile"),
        "named rejection for an old profile: {err}"
    );

    // A pre-/9 schema cannot carry the field (decode-time named rejection).
    let mut old_schema: serde_json::Value = serde_json::from_str(SCENE).expect("scene json");
    old_schema["schema"] = serde_json::Value::String("autographics.scene/6".into());
    old_schema.as_object_mut().expect("doc").remove("cues");
    let err = Document::decode(old_schema.to_string().as_bytes())
        .expect_err("a /6 document with a layer is rejected");
    assert!(
        err.contains("layer requires autographics.scene/9"),
        "named schema rejection: {err}"
    );
    println!(
        "PLAN058_TIMING test=layer_requires_topdown2d_profile_and_scene_9 ms={} old_profile=named-reject schema6=named-reject",
        t0.elapsed().as_millis()
    );
}

fn sha861(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
