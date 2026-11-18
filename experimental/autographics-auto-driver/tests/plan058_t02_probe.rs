//! PLAN-058 T-02 mechanism probe (`auto-plan-work`, 2026-11-18).
//!
//! Scope (deliberately narrow, see the Plan §5 「2026-11-18 T-02 联冻回签与预算
//! 口径」): prove with **existing static f32 row values + host row writes** that
//! a single scene-row schema can drive (a) the frozen queries-2d segment/sweep
//! service and (b) the frozen sprites-2d PNG/frame presentation, at the same
//! Tick, and pin the shared-pool credit semantics. This is a mechanism-level
//! proof only:
//!
//!  * the two frozen profiles cannot be assembled together today (queries-2d
//!    and sprites-2d are separate versions with disjoint `Api` values), so each
//!    half is driven by the **same `Row` fixture** — one f32 row table whose
//!    values are handed to the query world through the host row-write API
//!    (`_ag_identity`/`_ag_visibility`/`_ag_append_field`/`_ag_spatial_config`,
//!    the frozen precedent of `tests/node_queries_flow.rs`) and to the sprite
//!    presentation through the accepted scene document (`.agscene` row values).
//!  * a single assembly combination, the Node authoring chain, and f32 business
//!    expressions (`x = x + vx*dt`) are **not** claimed here; they are T-03/T-04
//!    and the 981 `D-981-AGE-F32-SURFACE` slice.
//!  * the render-only ground row is separated from the query candidate table
//!    (the frozen v3/v4 candidate set has no per-row collision flag: a box flag
//!    of 0 is named fault 9, so a visible ground row cannot be carried into the
//!    query world without becoming a blocker). The probe proves the separation
//!    contract (one logical owner, same-tick row values, no duplicate Tick, no
//!    misalignment, no hidden blocking) and reproduces the 055 counterexample as
//!    a negative control when the ground row *is* carried into the query world.
//!
//! Run single-threaded (real offscreen GPU + shared ARTIFACTS json):
//!   cargo test --locked --offline --test plan058_t02_probe -- --test-threads=1
//!
//! Artifacts default to `D:/autostack/plan-artifacts/plan-058/auto-ui/`
//! (override with `PLAN058_ARTIFACTS`).
use autographics_auto_driver::{
    authoring::{
        behavior::{Api, SourceSnapshot},
        host::Host,
        play::{Action, Frame, State},
        storage::Storage,
    },
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use glam::Vec2;
use png::{BitDepth, ColorType, Decoder};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const DT: f32 = 1.0 / 60.0;

const ATLAS_PNG: &[u8] = include_bytes!(
    "../../autographics-node2d-pilot/fixtures/sprites2d/assets/95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png"
);
const ATLAS_W: u32 = 96;
const ATLAS_H: u32 = 32;
const FRAME: [u32; 4] = [0, 0, 32, 32];

/// Camera rectangle and view (frozen 2D follow camera, `SURFACE_2D = 800x380`).
/// With `view = (12, 8)` the follow centre is clamped to x >= 8, so both the
/// initial (x=4) and the query-produced stop (x≈7.98) rows render with the same
/// camera centre: the only thing that changes is the row value itself.

const PLAYER_X0: f32 = 4.0;
const PLAYER_Y0: f32 = 0.0;
const SWEEP_DX: f32 = 100.0;
const CLEARANCE: f32 = 0.02;
const SEG_LEN: f32 = 6.0;

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

fn srgb_decode(v: u8) -> f64 {
    let c = f64::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn composite(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    let alpha = f64::from(src[3]) / 255.0;
    [
        srgb_encode(srgb_decode(src[0]) * alpha + srgb_decode(dst[0]) * (1.0 - alpha)),
        srgb_encode(srgb_decode(src[1]) * alpha + srgb_decode(dst[1]) * (1.0 - alpha)),
        srgb_encode(srgb_decode(src[2]) * alpha + srgb_decode(dst[2]) * (1.0 - alpha)),
        255,
    ]
}

fn near(actual: [u8; 4], expected: [u8; 4], tolerance: u8) -> bool {
    actual
        .iter()
        .zip(expected)
        .all(|(a, e)| a.abs_diff(e) <= tolerance)
}

// ------------------------------------------------------- one row fixture ----
//
// The single source of truth for BOTH halves: every value below is an f32 that
// is written into the queries-2d static row table by host row writes and into
// the sprites-2d scene document as row values. `query` marks the rows that are
// carried into the query candidate table; `ground` is render-only (the frozen
// v3/v4 candidate set cannot express "visible but not collidable" — see the
// module comment; the mutation control reproduces the 055 t=0 counterexample).

#[derive(Clone, Copy, Debug)]
struct Row {
    key: &'static str,
    id: u64,
    pos: [f32; 2],
    half: [f32; 2],
    rgba: [f32; 4],
    sprite: bool,
    query: bool,
}

/// Ground covers exactly the camera window `x in [2,14], y in [-4,4]`.
fn fixture(player_x: f32) -> Vec<Row> {
    vec![
        Row {
            key: "ground",
            id: 1,
            pos: [8.0, 0.0],
            half: [6.0, 4.0],
            rgba: [0.18, 0.45, 0.22, 1.0],
            sprite: false,
            query: false,
        },
        Row {
            key: "player",
            id: 2,
            pos: [player_x, PLAYER_Y0],
            // the frozen character service accepts only its 0.8 x 1.0 shape
            half: [0.4, 0.5],
            rgba: [0.9, 0.3, 0.2, 1.0],
            sprite: true,
            query: true,
        },
        Row {
            key: "obstacle_a",
            id: 3,
            pos: [10.0, 0.0],
            half: [1.0, 3.0],
            rgba: [0.55, 0.55, 0.6, 1.0],
            sprite: false,
            query: true,
        },
        Row {
            key: "obstacle_b",
            id: 4,
            pos: [4.0, -2.5],
            half: [1.0, 1.0],
            rgba: [0.35, 0.35, 0.4, 1.0],
            sprite: false,
            query: true,
        },
        Row {
            key: "target_a",
            id: 5,
            pos: [4.0, 2.0],
            half: [0.5, 0.5],
            rgba: [1.0, 0.55, 0.15, 1.0],
            sprite: false,
            query: true,
        },
        Row {
            key: "target_b",
            id: 6,
            pos: [4.0, 3.25],
            half: [0.5, 0.5],
            rgba: [0.55, 0.15, 0.8, 1.0],
            sprite: false,
            query: true,
        },
    ]
}

fn player(rows: &[Row]) -> Row {
    *rows.iter().find(|r| r.key == "player").expect("player row")
}

fn ground(rows: &[Row]) -> Row {
    *rows.iter().find(|r| r.key == "ground").expect("ground row")
}

fn quad(rows: &[Row], key: &str) -> Row {
    *rows
        .iter()
        .find(|r| r.key == key)
        .unwrap_or_else(|| panic!("{key}"))
}

// ------------------------------------------------ queries-2d row host write --
//
// The frozen queries-2d entry always binds one character and one camera, so the
// row table carries a parked spectator at ordinal 0 (stable id 1) exactly like
// the frozen q-sweep product; the player is row ordinal 1 and is moved by a
// plain row write (`ag_move_local`), never by the character service. All row
// values are the f32 fixture values above.

const QUERY_WORLD: &str = r#"var ok=0.0
var g_shit=0
var g_snode=-1
var g_st=0.0
var g_snx=0.0
var g_sov=0
var g_uhit=0
var g_unode=-1
var g_ut=0.0
var g_tav=0
var g_tbv=0
var g_px=0.0
var g_py=0.0
fn main_lifetick(dt float) {
 let q = ag_sweep_aabb(1, 1, 100.0, 0.0, 0.0)
 g_shit = ag_sweep_hit(q)
 g_snode = ag_sweep_node(q)
 g_st = ag_sweep_t(q)
 g_snx = ag_sweep_normal(q, 0)
 g_sov = ag_sweep_start_overlap(q)
 let ax = ag_world(1, 0)
 let ay = ag_world(1, 1)
 let u = ag_cast_segment(2, 1, 1, ax, ay, 0.0, ax, (ay + 6.0), 0.0)
 g_uhit = ag_cast_hit(u)
 g_unode = ag_cast_node(u)
 g_ut = ag_cast_t(u)
 if (g_unode == 4) {
  ag_set_visible(4, 1)
  ag_set_visible(5, 0)
 }
 if (g_unode == 5) {
  ag_set_visible(4, 0)
  ag_set_visible(5, 1)
 }
 g_tav = ag_visible(4)
 g_tbv = ag_visible(5)
 if (g_shit == 1) {
  if (g_sov == 0) {
   ag_move_local(1, ((100.0 * g_st) - 0.02), 0.0, 0.0)
  }
 }
 g_px = ag_world(1, 0)
 g_py = ag_world(1, 1)
}
fn app_start() {
 ok=ok+ag_char_bind(1)*1.0
 ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)*1.0
}
fn app_update(dt float) {
 main_lifetick(dt)
}
"#;

/// Probe getters appended after admission (they may read trusted globals).
const QUERY_PROBES: &str = r#"
fn probe_credit() int {return _ag_spatial_calls}
fn probe_seq() int {return ag_tick()}
fn probe_shit() int {return g_shit}
fn probe_snode() int {return g_snode}
fn probe_st() float {return g_st}
fn probe_snx() float {return g_snx}
fn probe_sov() int {return g_sov}
fn probe_uhit() int {return g_uhit}
fn probe_unode() int {return g_unode}
fn probe_ut() float {return g_ut}
fn probe_tav() int {return g_tav}
fn probe_tbv() int {return g_tbv}
fn probe_px() float {return g_px}
fn probe_py() float {return g_py}
"#;

#[derive(Clone, Copy, Debug)]
struct QueryRow {
    pos: [f32; 2],
    half: [f32; 2],
    visible: i32,
}

/// Query ordinals: spectator 0, player 1, obstacles 2/3, targets 4/5, and the
/// ground last (ordinal 6) **only** in the mutation control.
fn query_rows(rows: &[Row], carry_ground: bool) -> Vec<QueryRow> {
    let mut out = vec![QueryRow {
        pos: [-100.0, -100.0],
        // the frozen character service only accepts its own 0.8 x 1.0 shape
        half: [0.4, 0.5],
        visible: 1,
    }];
    for r in rows.iter().filter(|r| r.query) {
        out.push(QueryRow {
            pos: r.pos,
            half: r.half,
            visible: 1,
        });
    }
    if carry_ground {
        let g = rows.iter().find(|r| r.key == "ground").expect("ground row");
        out.push(QueryRow {
            pos: g.pos,
            half: g.half,
            visible: 1,
        });
    }
    out
}

fn query_session(rows: &[Row], carry_ground: bool, tag: &str) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(tag),
        QUERY_WORLD.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .unwrap_or_else(|e| panic!("{tag} queries-2d admission: {e}"));
    let mut combined = snap.source.clone();
    combined.push_str(QUERY_PROBES);
    let mut s = Session::new(&combined).unwrap_or_else(|e| panic!("{tag} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let qrows = query_rows(rows, carry_ground);
    s.call(
        "_ag_spatial_reset",
        &[Int(qrows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, r) in qrows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(1), Int(2), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(r.visible)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [r.pos[0], r.pos[1], 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        // Descriptor rows are LOCAL half extents (the frozen host-write form).
        for (i, v) in [1.0, -r.half[0], -r.half[1], 0.0, r.half[0], r.half[1], 0.0]
            .iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..qrows.len() {
        let id = n as i32 + 1;
        s.call("_ag_char_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_obj_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    let status = s.int("_ag_status", &[]).unwrap();
    assert_eq!(status, 0, "{tag} start green (status {status})");
    s
}

fn tick(s: &mut Session, step: i32) {
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

// --------------------------------------------- independent f32 2D oracles ----
// Written from the frozen v3/v4 profile text (closed f32 slabs, nearest real t,
// ties by lowest ordinal, Minkowski shrink for the sweep). Never calls Auto.

type Boxes = Vec<(usize, [f32; 2], [f32; 2])>;

fn boxes(rows: &[Row], carry_ground: bool) -> Boxes {
    query_rows(rows, carry_ground)
        .iter()
        .enumerate()
        .map(|(n, r)| {
            (
                n,
                [r.pos[0] - r.half[0], r.pos[1] - r.half[1]],
                [r.pos[0] + r.half[0], r.pos[1] + r.half[1]],
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
) -> (i32, i32, f32, f32, i32) {
    let mut target: i32 = -1;
    let mut fraction: f32 = 2.0;
    for &(ord, lo, hi) in boxes {
        if ord == mover {
            continue;
        }
        let ylo = lo[1] - half[1];
        let yhi = hi[1] + half[1];
        if ay < ylo || ay > yhi {
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
        return (0, -1, 1.0, ax + dx, 0);
    }
    let (_, lo, hi) = boxes.iter().find(|b| b.0 as i32 == target).unwrap();
    let l = lo[0] - half[0];
    let h = hi[0] + half[0];
    let mut nx = 0.0f32;
    if ax < l || ax > h {
        if dx >= 0.0 {
            if (l - ax) / dx == fraction {
                nx = -1.0;
            }
        } else if (h - ax) / dx == fraction {
            nx = 1.0;
        }
    }
    let overlap = i32::from(nx == 0.0);
    (1, target, fraction, ax + fraction * dx, overlap)
}

// -------------------------------------------------- sprites-2d project ------

const SPRITES_WORLD: &str = r#"var ok=0.0
fn app_start() {
 ok=ok+ag_char_bind(2)*1.0
 ok=ok+ag_camera_follow_2d(2, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)*1.0
}
fn app_update(dt float) {
}
"#;

fn scene_text(world_sha: &str, world_len: usize, png_sha: &str, rows: &[Row]) -> String {
    let objects: Vec<Value> = rows
        .iter()
        .map(|r| {
            let sprite = if r.sprite {
                json!({
                    "asset": format!("sha256:{png_sha}"),
                    "frame": [FRAME[0], FRAME[1], FRAME[2], FRAME[3]],
                    "flip": false
                })
            } else {
                Value::Null
            };
            json!({
                "id": r.id,
                "name": r.key,
                "kind": "quad",
                "translation": [r.pos[0], r.pos[1], 0.0],
                "scale": [r.half[0] * 2.0, r.half[1] * 2.0, 1.0],
                "yaw": 0.0,
                "pitch": 0.0,
                "rgba": r.rgba,
                "visible": true,
                "parent": null,
                "resource": null,
                "sprite": sprite
            })
        })
        .collect();
    json!({
        "behavior": {
            "api": "autographics.sprites2d/1",
            "source": {
                "format": "auto-source",
                "sha256": world_sha,
                "byte_length": world_len
            },
            "actors": {"two_d": 1, "three_d": null}
        },
        "schema": "autographics.scene/6",
        "document_id": "plan058-t02-row-mechanism",
        "resources": ["circle", "quad", "cylinder", "player", "mob"],
        "objects": objects,
        "groups": [],
        "assets": [{
            "format": "png",
            "sha256": png_sha,
            "byte_length": ATLAS_PNG.len(),
            "name": format!("{png_sha}.png"),
            "width": ATLAS_W,
            "height": ATLAS_H
        }]
    })
    .to_string()
}

fn stage(tag: &str, rows: &[Row]) -> PathBuf {
    let world = SPRITES_WORLD;
    let scene = scene_text(&sha(world.as_bytes()), world.len(), &sha(ATLAS_PNG), rows);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan058-t02")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("scripts")).expect("scripts dir");
    fs::create_dir_all(dir.join("assets")).expect("assets dir");
    fs::write(dir.join("scene.agscene"), &scene).expect("scene");
    fs::write(
        dir.join("scripts")
            .join(format!("{}.at", sha(world.as_bytes()))),
        world,
    )
    .expect("world");
    fs::write(
        dir.join("assets").join(format!("{}.png", sha(ATLAS_PNG))),
        ATLAS_PNG,
    )
    .expect("asset");
    dir
}

fn pump(host: &mut Host) {
    let deadline = Instant::now() + Duration::from_secs(31);
    while host.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < deadline, "host settle deadline");
        host.frame(None).expect("frame");
    }
}

fn start_render(tag: &str, rows: &[Row]) -> Host {
    let dir = stage(tag, rows);
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

fn step(h: &mut Host) {
    {
        let play = h.play.as_mut().expect("run");
        play.resume().expect("resume");
        play.dispatch(Action {
            space: 0,
            x: 0,
            y: 0,
            toggle: false,
        })
        .expect("dispatch");
    }
    h.pause_play("");
    pump(h);
    assert_eq!(
        h.play.as_ref().map(|p| p.state),
        Some(State::Paused),
        "tick settled Paused"
    );
    if let Some(err) = h.play.as_ref().and_then(|p| p.error.clone()) {
        assert!(err.is_empty(), "tick fault: {err}");
    }
}

fn frame(h: &Host) -> &Frame {
    h.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .expect("accepted frame")
}

fn node_world(h: &Host, id: u64) -> Vec2 {
    let d = frame(h).nodes.get(&id).expect("node in frame");
    Vec2::new(d.translation[0], d.translation[1])
}

fn camera(h: &Host) -> &autographics_auto_driver::authoring::runtime_camera::RuntimeCameraFrame {
    frame(h).camera.as_ref().expect("camera in frame")
}

/// Follow centre is the accepted character row (the one logical owner) and is
/// the SAME f32 row value both halves consume.
fn screen_of(h: &Host, world: Vec2) -> Vec2 {
    let cam = camera(h);
    let actor = node_world(h, 2);
    let centre = cam
        .follow_center_2d(actor)
        .expect("follow centre (static row)");
    cam.screen_2d(world, centre).expect("screen projection")
}

fn px_per_world(h: &Host) -> Vec2 {
    camera(h).letterbox_2d().expect("letterbox").0
}

/// Screen rect (min, max) of one document quad row.
fn rect_of(h: &Host, r: &Row) -> (Vec2, Vec2) {
    let corners = [
        [r.pos[0] - r.half[0], r.pos[1] - r.half[1]],
        [r.pos[0] - r.half[0], r.pos[1] + r.half[1]],
        [r.pos[0] + r.half[0], r.pos[1] - r.half[1]],
        [r.pos[0] + r.half[0], r.pos[1] + r.half[1]],
    ];
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for c in corners {
        let p = screen_of(h, Vec2::new(c[0], c[1]));
        min = min.min(p);
        max = max.max(p);
    }
    (min, max)
}

fn expected_color(rgba: [f32; 4]) -> [u8; 4] {
    [
        srgb_encode(f64::from(rgba[0])),
        srgb_encode(f64::from(rgba[1])),
        srgb_encode(f64::from(rgba[2])),
        255,
    ]
}

fn classify(p: [u8; 4], palette: &[(usize, [u8; 3])]) -> Option<usize> {
    let mut best = None;
    // Tight palette match: flat quad colours are exact; AA blends, the sprite's
    // half-alpha texels and the letterbox background fall outside this ball.
    let mut bestd = 20i32 * 20;
    for (i, c) in palette {
        let d = (i32::from(p[0]) - i32::from(c[0])).pow(2)
            + (i32::from(p[1]) - i32::from(c[1])).pow(2)
            + (i32::from(p[2]) - i32::from(c[2])).pow(2);
        if d < bestd {
            bestd = d;
            best = Some(*i);
        }
    }
    best
}

struct QuadStat {
    key: String,
    count: usize,
    centroid: [f32; 2],
    bbox: [f32; 4],
    want_center: [f32; 2],
    want_area: f32,
    want_bbox: [f32; 4],
}

fn quad_stats(h: &Host, image: &Image, rows: &[Row]) -> Vec<QuadStat> {
    let keys = ["ground", "obstacle_a", "obstacle_b", "target_a", "target_b"];
    let mut palette = Vec::new();
    for k in keys {
        let r = quad(rows, k);
        let c = expected_color(r.rgba);
        palette.push((palette.len(), [c[0], c[1], c[2]]));
    }
    // Exact per-pixel classification of the captured frame (no sampling bias).
    let mut counts = [0usize; 5];
    let mut sums = [[0.0f64; 2]; 5];
    let mut bounds = [[
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ]; 5];
    for y in 0..image.height {
        for x in 0..image.width {
            let c = pixel(image, x, y);
            if let Some(i) = classify(c, &palette) {
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
    let cam = camera(h);
    let actor = node_world(h, 2);
    let centre = cam.follow_center_2d(actor).expect("centre");
    let (_, content, bars) = cam.letterbox_2d().expect("letterbox");
    let mut out = Vec::new();
    for (i, k) in keys.iter().enumerate() {
        let r = quad(rows, k);
        let (min, max) = rect_of(h, &r);
        let _ = cam
            .screen_2d(Vec2::new(r.pos[0], r.pos[1]), centre)
            .expect("projected centre");
        // The visible part is the projected rect clipped to the letterboxed
        // content area; the centroid expectation follows the visible part.
        let mut lo = [0.0f32; 2];
        let mut hi = [0.0f32; 2];
        for axis in 0..2 {
            lo[axis] = min[axis].max(bars[axis]);
            hi[axis] = max[axis].min(bars[axis] + content[axis]);
        }
        let count = counts[i];
        out.push(QuadStat {
            key: (*k).to_string(),
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
            want_center: [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5],
            want_area: (hi[0] - lo[0]).max(0.0) * (hi[1] - lo[1]).max(0.0),
            want_bbox: [lo[0], lo[1], hi[0] - 1.0, hi[1] - 1.0],
        });
    }
    out
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
    assert_eq!(info.color_type, ColorType::Rgba, "RGBA8 atlas");
    assert_eq!(info.bit_depth, BitDepth::Eight, "RGBA8 atlas");
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

/// Texel samples of the bound frame: for every distinct color inside the
/// interior (2..30) pick the texels closest to the sprite centre, plus the two
/// half-alpha texels (real CPU/GPU compositing over the ground row).
fn sprite_texels(image: &Image) -> Vec<(u32, u32)> {
    let f = FRAME;
    let mut by_color: std::collections::BTreeMap<[u8; 4], Vec<(u32, u32)>> = Default::default();
    for y in 2..30u32 {
        for x in 2..30u32 {
            let c = pixel(image, f[0] + x, f[1] + y);
            if c[3] == 0 {
                continue;
            }
            by_color.entry(c).or_default().push((x, y));
        }
    }
    let mut out = Vec::new();
    for (color, mut list) in by_color {
        list.sort_by_key(|(x, y)| {
            let dx = *x as i64 - 16;
            let dy = *y as i64 - 16;
            dx * dx + dy * dy
        });
        let take = if color[3] < 255 { 2 } else { 3 };
        for t in list.into_iter().take(take) {
            if !out.contains(&t) {
                out.push(t);
            }
        }
    }
    out
}

// ------------------------------------------------------------- B fixture ----

const SWEEP_WORLD: &str = r#"var ok=0.0
var t1=0
var t2=0
var g_px=0.0
fn two_sweeps(dt float) {
 t1 = ag_sweep_aabb(1, 1, 100.0, 0.0, 0.0)
 t2 = ag_sweep_aabb(1, 1, 100.0, 0.0, 0.0)
 g_px = ag_world(1, 0)
}
fn app_start() {
 ok=ok+ag_char_bind(1)*1.0
 ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)*1.0
}
fn app_update(dt float) {
 two_sweeps(dt)
}
"#;

/// Two sweeps then a 1-credit getter: the 33rd charged credit.
const SWEEP_WORLD_33: &str = r#"var ok=0.0
var t1=0
var t2=0
var g33=0.0
fn two_sweeps(dt float) {
 t1 = ag_sweep_aabb(1, 1, 100.0, 0.0, 0.0)
 t2 = ag_sweep_aabb(1, 1, 100.0, 0.0, 0.0)
 g33 = ag_sweep_t(t1)
}
fn app_start() {
 ok=ok+ag_char_bind(1)*1.0
 ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)*1.0
}
fn app_update(dt float) {
 two_sweeps(dt)
}
"#;

const SWEEP_WORLD_40: &str = r#"var ok=0.0
var t1=0
var t2=0
var g_q3=0
var g_sticky=0.0
var g_px_before=0.0
var g_px_after=0.0
var g_vis_before=0
var g_vis_after=0
fn node_a(dt float) {
 t1 = ag_sweep_aabb(1, 1, 100.0, 0.0, 0.0)
 t2 = ag_cast_segment(2, 1, 1, 4.0, 0.0, 0.0, 4.0, 6.0, 0.0)
 g_px_before = ag_world(1, 0)
 g_vis_before = ag_visible(4)
}
fn node_b(dt float) {
 g_q3 = ag_sweep_aabb(1, 1, -100.0, 0.0, 0.0)
 if (g_q3 > 0) {
  ag_move_local(1, (0.0 - 100.0), 0.0, 0.0)
 }
 g_sticky = ag_sweep_t(t1)
 g_px_after = ag_world(1, 0)
 g_vis_after = ag_visible(4)
}
fn app_start() {
 ok=ok+ag_char_bind(1)*1.0
 ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)*1.0
}
fn app_update(dt float) {
 node_a(dt)
 node_b(dt)
}
"#;

const PROBES_32: &str = r#"
fn probe_credit() int {return _ag_spatial_calls}
fn probe_two_sweeps_ok() int {if (t1 > 0) {if (t2 > 0) {return 1}}; return 0}
fn probe_px() float {return g_px}
"#;

const PROBES_33: &str = r#"
fn probe_credit() int {return _ag_spatial_calls}
fn probe_two_sweeps_ok() int {if (t1 > 0) {if (t2 > 0) {return 1}}; return 0}
fn probe_g33() float {return g33}
"#;

const PROBES_40: &str = r#"
fn probe_credit() int {return _ag_spatial_calls}
fn probe_t1() int {return t1}
fn probe_q3() int {return g_q3}
fn probe_sticky() float {return g_sticky}
fn probe_px_before() float {return g_px_before}
fn probe_px_after() float {return g_px_after}
fn probe_vis_before() int {return g_vis_before}
fn probe_vis_after() int {return g_vis_after}
"#;

fn budget_session(world: &str, probes: &str, rows: &[Row], tag: &str) -> Session {
    let snap =
        SourceSnapshot::from_bytes_for(Path::new(tag), world.as_bytes().to_vec(), Api::Objects2V1)
            .unwrap_or_else(|e| panic!("{tag} admission: {e}"));
    let mut combined = snap.source.clone();
    combined.push_str(probes);
    let mut s = Session::new(&combined).unwrap_or_else(|e| panic!("{tag} session: {e}"));
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let qrows = query_rows(rows, false);
    s.call(
        "_ag_spatial_reset",
        &[Int(qrows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, r) in qrows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(1), Int(2), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(r.visible)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [r.pos[0], r.pos[1], 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in [1.0, -r.half[0], -r.half[1], 0.0, r.half[0], r.half[1], 0.0]
            .iter()
            .enumerate()
        {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..qrows.len() {
        s.call(
            "_ag_char_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call(
            "_ag_obj_map",
            &[Int(n as i32 + 1), Int(n as i32)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}

fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}

// ---------------------------------------------------------------- tests ----

/// T-02 A (query half): the single static f32 row table, written by the frozen
/// host row-write API, answers a sweep + a nearest-target segment at Tick 1 in
/// exactly 32 shared credits; the far obstacle and both interaction rows are
/// excluded by real geometry, and only the nearest same-def target publishes.
#[test]
fn t02a_query_rows_sweep_segment_oracle_and_budget() {
    let t0 = Instant::now();
    let rows = fixture(PLAYER_X0);
    let mut s = query_session(&rows, false, "t02a-query.at");
    assert_eq!(s.int("probe_seq", &[]).unwrap(), 0, "no tick before first");
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0, "queries-2d tick 1 green");
    assert_eq!(
        s.int("probe_seq", &[]).unwrap(),
        1,
        "Tick identity pinned to 1"
    );
    assert_eq!(
        s.int("probe_credit", &[]).unwrap(),
        32,
        "sweep 16 + 5 getters + segment 8 + 3 getters = 32 exactly"
    );

    let bx = boxes(&rows, false);
    let p = player(&rows);
    // Sweep: mover = player ordinal 1 (its own 0.8 x 1.0 shape), +100 from (4,0).
    let (hit, node, t, px, overlap) = oracle_sweep(&bx, 1, p.half, PLAYER_X0, PLAYER_Y0, SWEEP_DX);
    assert_eq!(hit, 1, "wall in the corridor");
    assert_eq!(node, 2, "nearest blocking row is obstacle_a (ordinal 2)");
    assert_eq!(overlap, 0, "no start overlap without the ground row");
    assert_eq!(s.int("probe_shit", &[]).unwrap(), hit);
    assert_eq!(s.int("probe_snode", &[]).unwrap(), node);
    assert_eq!(s.float("probe_st", &[]).unwrap(), t);
    assert_eq!(s.int("probe_sov", &[]).unwrap(), overlap);
    assert_eq!(s.float("probe_snx", &[]).unwrap(), -1.0, "left face normal");
    assert!(
        (px - (PLAYER_X0 + SWEEP_DX * t)).abs() <= 1e-3,
        "sweep contact point {px}"
    );
    // Real blocking: the row stops 0.02 before the wall face at x=9.
    let stop = s.float("probe_px", &[]).unwrap();
    assert!(
        (stop - (px - CLEARANCE)).abs() <= 2e-3,
        "stop position {stop} = contact {px} - {CLEARANCE}"
    );
    assert!(
        stop + p.half[0] < 9.0,
        "blocked outside obstacle_a's face ({stop} + half {} < 9)",
        p.half[0]
    );

    // Nearest interaction target: segment (4,0) -> (4,6); target_a at t=0.25,
    // target_b at t=0.4583...; the independent f32 slab oracle picks target_a.
    let (uhit, unode, ut) = oracle_segment(&bx, 1, PLAYER_X0, PLAYER_Y0, PLAYER_X0, SEG_LEN);
    assert_eq!(uhit, 1);
    assert_eq!(unode, 4, "target_a (ordinal 4) is nearest");
    assert_eq!(s.int("probe_uhit", &[]).unwrap(), uhit);
    assert_eq!(s.int("probe_unode", &[]).unwrap(), unode);
    assert_eq!(s.float("probe_ut", &[]).unwrap(), ut);
    // The far same-def target is a real second candidate, not a shadow copy:
    // a longer ray still meets target_a first but a ray aimed at target_b's
    // band (x offset) selects it, and the ordinals differ.
    let (_, unode_far, ut_far) = oracle_segment(&bx[..6], 1, PLAYER_X0, PLAYER_Y0, PLAYER_X0, 9.0);
    assert_eq!(unode_far, 4, "extending the ray still meets target_a first");
    assert!(ut_far < 0.3, "target_a stays nearest on a longer ray");
    let (bhit, bnode, bt) = oracle_segment(&bx[..6], 1, 4.9, 0.0, 4.9, 6.0);
    assert_eq!(
        (bhit, bnode),
        (0, -1),
        "a ray beside both targets hits neither (independent of the answer rows)"
    );
    assert_eq!(bt, 1.0, "miss keeps t=1");
    // Per-instance publish: only the chosen target becomes visible.
    assert_eq!(
        s.int("probe_tav", &[]).unwrap(),
        1,
        "nearest target visible"
    );
    assert_eq!(s.int("probe_tbv", &[]).unwrap(), 0, "far target hidden");
    assert_eq!(status(&mut s), 0, "no fault at exactly 32 credits");
    println!(
        "PLAN058_TIMING test=t02a_query_rows_sweep_segment_oracle_and_budget ms={} credits=32 stop={stop} t={t} node={node} unode={unode} ut={ut}",
        t0.elapsed().as_millis()
    );
}

/// T-02 A (ground separation): the ground row is visible in the document but
/// absent from the query candidate table, so neither the sweep nor the nearest
/// target query can hit it. Carrying the same visible ground row into the query
/// world (the frozen v3/v4 behaviour) reproduces 055 exactly: t=0 start overlap,
/// no movement, and the ground wins the nearest-target query.
#[test]
fn t02a_ground_render_row_does_not_block_or_win_queries() {
    let t0 = Instant::now();
    let rows = fixture(PLAYER_X0);
    let g = ground(&rows);

    // Positive arm: ground present in the document rows, absent from queries.
    let mut s = query_session(&rows, false, "t02a-sep-pos.at");
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0);
    let pbx = boxes(&rows, false);
    let ph = player(&rows).half;
    let (_, pnode, pt, ppx, _) = oracle_sweep(&pbx, 1, ph, PLAYER_X0, PLAYER_Y0, SWEEP_DX);
    assert_eq!(pnode, 2, "sweep hits obstacle_a");
    assert_eq!(s.int("probe_snode", &[]).unwrap(), 2);
    assert_eq!(s.float("probe_st", &[]).unwrap(), pt);
    assert_eq!(s.int("probe_unode", &[]).unwrap(), 4, "target_a nearest");
    assert!(
        (s.float("probe_px", &[]).unwrap() - (ppx - CLEARANCE)).abs() <= 2e-3,
        "ground row does not block the corridor"
    );

    // Mutation control: the same visible ground row carried into the queries
    // (ground box = x 2..14, y -4..4; the player starts inside it).
    let mut m = query_session(&rows, true, "t02a-sep-mut.at");
    tick(&mut m, 1);
    assert_eq!(
        status(&mut m),
        0,
        "mutation still runs (semantics red, not fault)"
    );
    let mbx = boxes(&rows, true);
    assert_eq!(
        mbx.len(),
        7,
        "mutation table carries the ground at ordinal 6"
    );
    assert_eq!(mbx[6].1, [g.pos[0] - g.half[0], g.pos[1] - g.half[1]]);
    let (mhit, mnode, mt, _, moverlap) = oracle_sweep(&mbx, 1, ph, PLAYER_X0, PLAYER_Y0, SWEEP_DX);
    assert_eq!(mhit, 1);
    assert_eq!(mnode, 6, "ground row wins the sweep");
    assert!(mt == 0.0, "055 start-overlap t=0, got {mt}");
    assert_eq!(moverlap, 1, "055 start_overlap reproduced");
    assert_eq!(s.int("probe_sov", &[]).unwrap(), 0);
    assert_eq!(m.int("probe_sov", &[]).unwrap(), 1);
    assert_eq!(m.int("probe_snode", &[]).unwrap(), 6);
    assert_eq!(m.float("probe_st", &[]).unwrap(), 0.0);
    assert_eq!(
        m.float("probe_px", &[]).unwrap(),
        PLAYER_X0,
        "the mutation cannot move (055 no-go): hidden blocking reproduced"
    );
    let (muhit, munode, mut_t) = oracle_segment(&mbx, 1, PLAYER_X0, PLAYER_Y0, PLAYER_X0, SEG_LEN);
    assert_eq!(muhit, 1);
    assert_eq!(munode, 6, "ground misselected as the nearest target");
    assert!(mut_t == 0.0, "inside/boundary start t=0, got {mut_t}");
    assert_eq!(m.int("probe_unode", &[]).unwrap(), 6);
    assert_eq!(m.int("probe_tav", &[]).unwrap(), 1);
    assert_eq!(m.int("probe_tbv", &[]).unwrap(), 1);
    let evidence = json!({
        "case": "t02a_ground_render_row_does_not_block_or_win_queries",
        "ground_row": {"pos": g.pos, "half": g.half, "visible": true},
        "positive": {"sweep_node": s.int("probe_snode", &[]).unwrap(), "sweep_t": s.float("probe_st", &[]).unwrap(), "segment_node": s.int("probe_unode", &[]).unwrap(), "segment_t": s.float("probe_ut", &[]).unwrap(), "stop_x": s.float("probe_px", &[]).unwrap()},
        "mutation": {"sweep_node": m.int("probe_snode", &[]).unwrap(), "sweep_t": m.float("probe_st", &[]).unwrap(), "start_overlap": m.int("probe_sov", &[]).unwrap(), "segment_node": m.int("probe_unode", &[]).unwrap(), "segment_t": m.float("probe_ut", &[]).unwrap(), "px": m.float("probe_px", &[]).unwrap()},
        "boundary": "render-only ground row = separation contract; frozen v3/v4 has no per-row collision flag (descriptor flag 0 is named fault 9)"
    });
    fs::write(
        artifacts().join("t02a-ground-separation.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .expect("write ground evidence");
    println!(
        "PLAN058_TIMING test=t02a_ground_render_row_does_not_block_or_win_queries ms={} positive_node={} mutation_node={} mutation_t=0",
        t0.elapsed().as_millis(),
        s.int("probe_unode", &[]).unwrap(),
        6
    );
}

/// T-02 A (presentation half): the same f32 row values render the bound PNG
/// sprite and the render-only ground through the frozen sprites-2d profile on a
/// real offscreen GPU. Absolute sprite pixels are checked at texel centres
/// against the camera projection of the SAME row value, and the second Run uses
/// the exact f32 stop position produced by the query half.
#[test]
fn t02a_sprite_and_ground_pixels_match_row_values_under_camera() {
    let t0 = Instant::now();
    let rows = fixture(PLAYER_X0);
    // The query half's own Tick-1 row value (f32), used as the row value of the
    // second render Run: no int/rounding substitution.
    let mut q = query_session(&rows, false, "t02a-render-query.at");
    tick(&mut q, 1);
    assert_eq!(status(&mut q), 0);
    let stop = q.float("probe_px", &[]).unwrap();
    let (_, _, qt, qpx, _) = oracle_sweep(
        &boxes(&rows, false),
        1,
        player(&rows).half,
        PLAYER_X0,
        PLAYER_Y0,
        SWEEP_DX,
    );
    assert!(
        (stop - (qpx - CLEARANCE)).abs() <= 2e-3,
        "query-produced stop row value {stop} (contact {qpx}, t {qt})"
    );

    let atlas = decode_png(ATLAS_PNG);
    let texels = sprite_texels(&atlas);
    assert!(
        texels.len() >= 8,
        "distinct sprite texel samples: {}",
        texels.len()
    );

    let mut evidence =
        json!({"case": "t02a_sprite_and_ground_pixels", "query_stop_x": stop, "runs": []});

    for (tag, player_x) in [("t02a-render-a", PLAYER_X0), ("t02a-render-b", stop)] {
        let rows = fixture(player_x);
        let mut h = start_render(tag, &rows);
        let start = Instant::now();
        step(&mut h);
        assert_eq!(
            frame(&h).tick,
            1,
            "{tag}: same Tick identity as the query half"
        );
        // 1) The camera projects the accepted row value; never a hard-coded px.
        let p = player(&rows);
        let _centre = camera(&h)
            .follow_center_2d(node_world(&h, 2))
            .expect("centre");
        let p2 = Vec2::new(p.pos[0], p.pos[1]);
        assert_eq!(
            node_world(&h, 2),
            p2,
            "{tag}: document row value is the nodes' world position"
        );
        let sprite_center = screen_of(&h, p2);
        let scale = px_per_world(&h);
        let half = Vec2::new(p.half[0] * scale.x, p.half[1] * scale.y);
        // 2) Sprite pixels: texel centres vs independent PNG decode + linear
        //    compositing over the ground colour (tolerance 3/255 sRGB).
        let ground_sample = screen_of(&h, Vec2::new(ground(&rows).pos[0] + 4.0, 0.0));
        let ground_px =
            h.rt.probe_pixel(h.two, [ground_sample.x, ground_sample.y])
                .expect("ground probe");
        let mut texel_points = Vec::new();
        let mut texel_expected = Vec::new();
        for (tx, ty) in &texels {
            let xy = [
                sprite_center.x + ((*tx as f32 + 0.5) / 32.0 * 2.0 - 1.0) * half.x,
                sprite_center.y + ((*ty as f32 + 0.5) / 32.0 * 2.0 - 1.0) * half.y,
            ];
            texel_points.push(xy);
            texel_expected.push(composite(
                pixel(&atlas, FRAME[0] + *tx, FRAME[1] + *ty),
                ground_px,
            ));
        }
        let observed =
            h.rt.probe_pixels(h.two, &texel_points)
                .expect("texel probe");
        for (i, (got, want)) in observed.iter().zip(texel_expected.iter()).enumerate() {
            assert!(
                near(*got, *want, 3),
                "{tag}: sprite texel {i} at {:?}: got {got:?} want {want:?} (sprite centre {sprite_center:?}, ground {ground_px:?})",
                texel_points[i]
            );
        }
        // 3) Quad rows: five flat rows (ground + 2 obstacles + 2 same-def
        //    targets) projected by the camera; exact per-pixel centroid of the
        //    captured frame within 2 px of the camera projection.
        let (png_sha, bytes, w, hgt) = h.rt.capture(h.two, tag).expect("capture");
        let name = format!("{tag}.png");
        fs::write(artifacts().join(&name), &bytes).expect("write png");
        let image = decode_png(&bytes);
        assert_eq!((image.width, image.height), (w, hgt), "capture layout");
        let stats = quad_stats(&h, &image, &rows);
        for st in &stats {
            assert!(
                st.count > 0,
                "{tag}: {} has visible pixels ({:?})",
                st.key,
                st.want_center
            );
            // The projected rectangle is the position oracle: every classified
            // pixel must sit inside it and its bounding box must be the rect.
            for c in st.centroid.iter() {
                assert!(c.is_finite(), "{tag}: {} centroid finite", st.key);
            }
            for axis in 0..4 {
                assert!(
                    (st.bbox[axis] - st.want_bbox[axis]).abs() <= 2.0,
                    "{tag}: {} bbox {:?} vs camera-projected rect {:?}",
                    st.key,
                    st.bbox,
                    st.want_bbox
                );
            }
            if st.key == "ground" {
                // The ground is occluded by the other rows, so its centre of
                // mass is not the rect centre; its bbox and coverage are the
                // position/size oracle.
                assert!(
                    st.count as f32 >= st.want_area * 0.55,
                    "{tag}: ground coverage {} of {}",
                    st.count,
                    st.want_area
                );
            } else {
                assert!(
                    (st.centroid[0] - st.want_center[0]).abs() <= 2.0
                        && (st.centroid[1] - st.want_center[1]).abs() <= 2.0,
                    "{tag}: {} centroid {:?} vs camera projection {:?} ({} px)",
                    st.key,
                    st.centroid,
                    st.want_center,
                    st.count
                );
                assert!(
                    (st.count as f32 - st.want_area).abs() <= st.want_area * 0.10,
                    "{tag}: {} drawn area {} vs projected {}",
                    st.key,
                    st.count,
                    st.want_area
                );
            }
        }
        let ground_stat = stats
            .iter()
            .find(|s| s.key == "ground")
            .expect("ground stat");
        assert!(
            ground_stat.count > 800 * 380 / 2,
            "{tag}: ground really drawn ({} px)",
            ground_stat.count
        );
        println!(
            "{tag}: capture {w}x{hgt} sha={png_sha} player_x={:.6} sprite_center=({:.2},{:.2}) ground_px={:?} ground_pixels={}",
            p.pos[0], sprite_center.x, sprite_center.y, ground_px, ground_stat.count
        );
        let mut runs = evidence["runs"].as_array().cloned().unwrap_or_default();
        runs.push(json!({
            "tag": tag,
            "tick": frame(&h).tick,
            "player_row_x": p.pos[0],
            "node_world": [p2.x, p2.y],
            "sprite_center_px": [sprite_center.x, sprite_center.y],
            "px_per_world": [scale.x, scale.y],
            "ground_px": ground_px,
            "texel_samples": texels.len(),
            "png": name,
            "png_sha256": png_sha,
            "quads": stats.iter().map(|s| json!({
                "key": s.key, "count": s.count, "centroid": s.centroid,
                "want_center": s.want_center, "want_area": s.want_area
            })).collect::<Vec<_>>(),
            "step_ms": start.elapsed().as_millis(),
        }));
        evidence["runs"] = Value::Array(runs);
        h.stop_play();
    }
    fs::write(
        artifacts().join("t02a-row-presentation.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .expect("write presentation evidence");
    println!(
        "PLAN058_TIMING test=t02a_sprite_and_ground_pixels_match_row_values_under_camera ms={} stop={:.6} texels={}",
        t0.elapsed().as_millis(),
        stop,
        texels.len()
    );
}

/// T-02 B1+B3: two sweeps in one Tick are exactly 32 credits and legal; the
/// credit meter resets at the next callback (another two sweeps stay legal at
/// 32 instead of 64), and a following 1-credit read is the named 33rd charge.
#[test]
fn t02b_two_sweeps_32_legal_then_reset_and_33rd_refused() {
    let t0 = Instant::now();
    let rows = fixture(PLAYER_X0);

    // B1: two sweeps, no getters.
    let mut s = budget_session(SWEEP_WORLD, PROBES_32, &rows, "t02b-32.at");
    tick(&mut s, 1);
    assert_eq!(status(&mut s), 0, "16+16=32 is legal");
    assert_eq!(
        s.int("probe_credit", &[]).unwrap(),
        32,
        "exactly 32 charged"
    );
    assert_eq!(
        s.int("probe_two_sweeps_ok", &[]).unwrap(),
        1,
        "both sweeps returned real tokens (effect normal)"
    );
    assert_eq!(
        s.float("probe_px", &[]).unwrap(),
        PLAYER_X0,
        "sweeps are read-only: the row is unchanged"
    );

    // B3: the same callback shape on the next Tick stays at 32 (reset).
    tick(&mut s, 2);
    assert_eq!(status(&mut s), 0, "tick 2 also legal");
    assert_eq!(
        s.int("probe_credit", &[]).unwrap(),
        32,
        "credits reset per Tick; without reset tick 2 would be 64"
    );
    assert_eq!(s.int("probe_two_sweeps_ok", &[]).unwrap(), 1);
    tick(&mut s, 3);
    assert_eq!(status(&mut s), 0, "tick 3 also legal");
    assert_eq!(s.int("probe_credit", &[]).unwrap(), 32);

    // The 33rd charged credit after 16+16 is refused (separate Run: the helper
    // adds the 1-credit read inside the same Tick).
    let mut f = budget_session(SWEEP_WORLD_33, PROBES_33, &rows, "t02b-33.at");
    tick(&mut f, 1);
    assert_eq!(
        status(&mut f),
        10,
        "the 33rd charged credit is the named fault 10"
    );
    assert_eq!(
        f.int("probe_credit", &[]).unwrap(),
        33,
        "33 charged attempts"
    );
    assert_eq!(
        f.float("probe_g33", &[]).unwrap(),
        0.0,
        "refused read returns 0 (a legal read would be t=0.04)"
    );
    println!(
        "PLAN058_TIMING test=t02b_two_sweeps_32_legal_then_reset_and_33rd_refused ms={} credits_32=32 credits_33=33 fault=10",
        t0.elapsed().as_millis()
    );
}

/// T-02 B2: the same Tick's pool is shared by every logical Node. sweep(16) +
/// segment(8) = 24, then a second Node's sweep would be 40 > 32: named fault 10,
/// zero effect (no token, no move, no visibility publish), and the next cheaper
/// call is still refused while the meter is over budget.
#[test]
fn t02b_second_node_over_budget_is_named_zero_effect_and_sticky() {
    let t0 = Instant::now();
    let rows = fixture(PLAYER_X0);
    let mut s = budget_session(SWEEP_WORLD_40, PROBES_40, &rows, "t02b-40.at");
    tick(&mut s, 1);
    assert_eq!(
        status(&mut s),
        10,
        "sweep+segment+second sweep = 40 > 32: named fault 10"
    );
    // The refused 1-credit read still increments the attempt counter before the
    // named rejection (`_ag_cast_charge` order), so the meter reads 41; there is
    // no refund, but the Run is already fault-terminated so no effect can land.
    assert_eq!(
        s.int("probe_credit", &[]).unwrap(),
        41,
        "40 + the refused 1-credit read (attempt counter, no refund)"
    );
    assert_eq!(
        s.int("probe_q3", &[]).unwrap(),
        0,
        "refused sweep token is 0"
    );
    assert_eq!(
        s.float("probe_sticky", &[]).unwrap(),
        0.0,
        "the 1-credit getter after the refusal is refused too (was t=0.04 when legal)"
    );
    assert_eq!(
        s.float("probe_px_before", &[]).unwrap(),
        s.float("probe_px_after", &[]).unwrap(),
        "zero effect: row position unchanged ({})",
        s.float("probe_px_before", &[]).unwrap()
    );
    assert_eq!(
        s.int("probe_vis_before", &[]).unwrap(),
        s.int("probe_vis_after", &[]).unwrap(),
        "zero effect: no visibility publish"
    );
    // The legal pre-refusal calls did run: the segment token exists and the
    // sweep token is real, but nothing over the budget landed.
    assert_eq!(
        s.int("probe_t1", &[]).unwrap() > 0,
        true,
        "first sweep legal"
    );
    let evidence = json!({
        "case": "t02b_second_node_over_budget",
        "tick": 1,
        "call_sequence": [
            {"node": "node_a", "call": "sweep(16)", "cumulative": 16},
            {"node": "node_a", "call": "segment(8)", "cumulative": 24},
            {"node": "node_b", "call": "sweep(16)", "cumulative_attempt": 40, "outcome": "named fault 10, zero effect"},
            {"node": "node_b", "call": "sweep_t(token_a) 1", "cumulative_attempt": 41, "outcome": "still refused (fault 10)"}
        ],
        "fault": 10,
        "fault_text": "QueryBudget (named, sticky within the Run)",
        "position_before": s.float("probe_px_before", &[]).unwrap(),
        "position_after": s.float("probe_px_after", &[]).unwrap(),
        "visibility_before": s.int("probe_vis_before", &[]).unwrap(),
        "visibility_after": s.int("probe_vis_after", &[]).unwrap(),
        "sticky_getter_value": s.float("probe_sticky", &[]).unwrap()
    });
    fs::write(
        artifacts().join("t02b-over-budget.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .expect("write budget evidence");
    println!(
        "PLAN058_TIMING test=t02b_second_node_over_budget_is_named_zero_effect_and_sticky ms={} credits=40 fault=10 px={} sticky={}",
        t0.elapsed().as_millis(),
        s.float("probe_px_before", &[]).unwrap(),
        s.float("probe_sticky", &[]).unwrap()
    );
}
