//! PLAN-021 T-04/T-05 flow gates: the frozen queries-2d generator products
//! (q-segment / q-changed / q-sweep / q-cross) run the three query chains on
//! the real assembled `autographics.objects2d/1` runtime — CPU sessions with
//! an independent f32 slab oracle and exact shared-pool credit counts, plus
//! real-GPU marker/stop evidence. The original numeric/fee semantics are the
//! delivered v3/v4 contracts; nothing here relaxes them.
//! Run single-threaded:
//! cargo test --locked --offline --test node_queries_flow -- --test-threads=1
use agp::renderer::capture::sha256_hex;
use autographics_auto_driver::{
    authoring::{
        assets::Registry,
        behavior::{Api, SourceSnapshot},
        behavior_project::CodePayload,
        host::Host,
        play::Action,
        storage::Storage,
    },
    ResultKind, Scalar,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const SCENE_SEG: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-segment/scene.agscene");
const WORLD_SEG: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-segment/world.at");
const SCENE_CHG: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-changed/scene.agscene");
const WORLD_CHG: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-changed/world.at");
const SCENE_SWEEP: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-sweep/scene.agscene");
const WORLD_SWEEP: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-sweep/world.at");
const SCENE_CROSS: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-cross/scene.agscene");
const WORLD_CROSS: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-cross/world.at");
const SCENE_CROSS_PLUS: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-cross-plus/scene.agscene");
const WORLD_CROSS_PLUS: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries2d/q-cross-plus/world.at");

const DT: f32 = 1.0 / 60.0;

// -------------------------------------------------------------- harness ----

fn write_project(scene_text: &str, world_src: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan021-queries")
        .join(format!("{}-{tag}", std::process::id()));
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    let sha = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene binds a source sha")
        .to_string();
    std::fs::write(scripts.join(format!("{sha}.at")), world_src).expect("write script");
    dir
}

fn load_code(scene_text: &str, world_src: &str, tag: &str) -> std::sync::Arc<CodePayload> {
    // Identity chain spot check: the scene binds sha256(world bytes).
    let bound = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("bound sha");
    assert_eq!(
        bound,
        sha256_hex(world_src.as_bytes()),
        "scene binds the world source sha"
    );
    let dir = write_project(scene_text, world_src, tag);
    let (_, _, code, _) = Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
        .unwrap_or_else(|e| panic!("load scene project: {e:?}"));
    code.expect("behavior code loaded")
}

fn probe_session(scene_text: &str, world_src: &str, name: &str, run: i32) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world_src.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .expect("admission");
    // Probe observers ride the combined managed source (the user region
    // cannot name trusted globals).
    let mut combined = snap.source.clone();
    combined.push_str("\nfn probe_credit() int {return _ag_spatial_calls}\n");
    let mut s = Session::new(&combined).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let rows = scene_rows(scene_text);
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, (kind, _, _, pos, visible, hx, hy)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(2), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(*visible as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [pos[0], pos[1], 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        // Descriptor rows are LOCAL half-extents relative to the world
        // position ([1, -hx, -hy, 0, +hx, +hy, 0]).
        for (i, v) in [1.0, -*hx, -*hy, 0.0, *hx, *hy, 0.0].iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..rows.len() {
        let id = n as i32 + 1;
        s.call("_ag_char_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_obj_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}

/// (kind, lo, hi, translation, declared_visible, half extents) per row.
fn scene_rows(scene_text: &str) -> Vec<(i32, [f32; 3], [f32; 3], [f32; 2], i32, f32, f32)> {
    let objects = scene_text
        .split("\"objects\": [")
        .nth(1)
        .expect("objects list");
    let objects = &objects[..objects.find("\n ]").expect("objects end")];
    let mut rows = Vec::new();
    for chunk in objects.split("\"id\": ").skip(1) {
        let num = |key: &str| -> f64 {
            let ki = chunk.find(key).expect(key);
            let rest = &chunk[ki + key.len()..];
            let rest = rest.trim_start_matches(|c: char| c == '[' || c == ' ');
            let end = rest
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest.len());
            rest[..end].parse().expect("number")
        };
        let kind = if chunk.contains("\"kind\": \"quad\"") {
            1
        } else if chunk.contains("\"kind\": \"circle\"") {
            0
        } else {
            panic!("unexpected scene kind: {}", &chunk[..60.min(chunk.len())]);
        };
        let tx = num("\"translation\": [");
        let rest = &chunk[chunk.find("\"translation\": [").expect("t")..];
        let comma = rest.find(',').expect("comma");
        let rest2 = rest[comma + 1..].trim_start();
        let end = rest2
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(rest2.len());
        let ty: f64 = rest2[..end].parse().expect("ty");
        let sx = num("\"scale\": [");
        let rests = &chunk[chunk.find("\"scale\": [").expect("s")..];
        let commas = rests.find(',').expect("comma");
        let rests2 = rests[commas + 1..].trim_start();
        let ends = rests2
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(rests2.len());
        let sy: f64 = rests2[..ends].parse().expect("sy");
        let visible = if chunk.contains("\"visible\": true") {
            1
        } else {
            0
        };
        rows.push((
            kind,
            [(tx - sx / 2.0) as f32, (ty - sy / 2.0) as f32, 0.0],
            [(tx + sx / 2.0) as f32, (ty + sy / 2.0) as f32, 0.0],
            [tx as f32, ty as f32],
            visible,
            (sx / 2.0) as f32,
            (sy / 2.0) as f32,
        ));
    }
    rows
}

fn status(s: &mut Session) -> i32 {
    s.int("_ag_status", &[]).unwrap()
}

fn fcall(s: &mut Session, name: &str, args: &[Scalar]) -> f32 {
    s.float(name, args).unwrap()
}

fn icall(s: &mut Session, name: &str, args: &[Scalar]) -> i32 {
    s.int(name, args).unwrap()
}

fn tick_cpu(s: &mut Session, step: i32) {
    s.call("_ag_char_input", &[Int(0), Int(0)], ResultKind::Unit)
        .unwrap();
    s.call("_ag_tick", &[Int(step), Float(DT)], ResultKind::Unit)
        .unwrap();
}

fn set_action(s: &mut Session, space: i32, x: i32, y: i32) {
    s.call(
        "_ag_action",
        &[Int(space), Int(x), Int(y), Int(0)],
        ResultKind::Unit,
    )
    .unwrap();
}

fn credit(s: &mut Session) -> i32 {
    s.int("probe_credit", &[]).unwrap()
}

// ------------------------------------------------- independent f32 oracle --
// Written from the pinned segment/sweep PROFILE text (slab traversal, closed
// intervals, nearest f32 t, ties lowest ordinal, X/Y normal preference,
// Minkowski shrink with half extents on every axis). Never calls the Auto
// implementation.

fn oracle_cast(
    boxes: &[(usize, [f32; 2], [f32; 2])],
    ignore: i32,
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
) -> (i32, i32, f32, f32, f32) {
    let dx = bx - ax;
    let dy = by - ay;
    let mut target: i32 = -1;
    let mut fraction: f32 = 2.0;
    for &(ord, lo, hi) in boxes {
        if ord as i32 == ignore {
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
        if enter >= fraction {
            continue;
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
    let mut found = 0;
    let mut px = bx;
    let mut nx = 0.0f32;
    if target >= 0 {
        found = 1;
        px = ax + fraction * dx;
        let (_, lo, hi) = boxes.iter().find(|b| b.0 as i32 == target).unwrap();
        if ax < lo[0] || ax > hi[0] {
            if dx >= 0.0 {
                if (lo[0] - ax) / dx == fraction {
                    nx = -1.0;
                }
            } else if (hi[0] - ax) / dx == fraction {
                nx = 1.0;
            }
        }
    } else {
        fraction = 1.0;
    }
    (found, target, fraction, px, nx)
}

fn oracle_sweep_x(
    mover_half_x: f32,
    mover_cy: f32,
    mover_half_y: f32,
    boxes: &[(usize, [f32; 2], [f32; 2])],
    self_ord: i32,
    ax: f32,
    dx: f32,
) -> (i32, i32, f32, f32, f32, i32) {
    let mut target: i32 = -1;
    let mut fraction: f32 = 2.0;
    for &(ord, lo, hi) in boxes {
        if ord as i32 == self_ord {
            continue;
        }
        let ylo = lo[1] - mover_half_y;
        let yhi = hi[1] + mover_half_y;
        if mover_cy < ylo || mover_cy > yhi {
            continue;
        }
        let l = lo[0] - mover_half_x;
        let h = hi[0] + mover_half_x;
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
    let mut found = 0;
    let mut px = ax + dx;
    let mut nx = 0.0f32;
    let mut overlap = 0;
    if target >= 0 {
        found = 1;
        px = ax + fraction * dx;
        let (_, lo, hi) = boxes.iter().find(|b| b.0 as i32 == target).unwrap();
        let l = lo[0] - mover_half_x;
        let h = hi[0] + mover_half_x;
        if ax < l || ax > h {
            if dx >= 0.0 {
                if (l - ax) / dx == fraction {
                    nx = -1.0;
                }
            } else if (h - ax) / dx == fraction {
                nx = 1.0;
            }
        }
        if nx == 0.0 {
            overlap = 1;
        }
    } else {
        fraction = 1.0;
    }
    (found, target, fraction, px, nx, overlap)
}

fn seg_boxes(scene_text: &str) -> Vec<(usize, [f32; 2], [f32; 2])> {
    scene_rows(scene_text)
        .iter()
        .enumerate()
        .map(|(n, (_, lo, hi, ..))| (n, [lo[0], lo[1]], [hi[0], hi[1]]))
        .collect()
}

fn approx(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

// --------------------------------------------------------------- tests ----

/// V-02 (chain 1) on the frozen q-segment product: oracle-checked
/// marker/door/point rows across a 120-tick walk, exactly 13 credits per
/// tick (cast 8 + 5 getters), t=0.5 nearest entry at the start pose, and the
/// 8ms host cap independently reported.
#[test]
fn v02_segment_chain_cpu_oracle_and_credits() {
    let mut s = probe_session(SCENE_SEG, WORLD_SEG, "q-segment.at", 1);
    let boxes = seg_boxes(SCENE_SEG);
    assert_eq!(status(&mut s), 0, "start green (zero-solid app_start ok)");

    let mut wall_max = 0u128;
    let mut wall_sum = 0u128;
    for tick in 1..=120i32 {
        let t0 = std::time::Instant::now();
        tick_cpu(&mut s, tick);
        wall_sum += t0.elapsed().as_micros();
        wall_max = wall_max.max(t0.elapsed().as_micros());
        assert_eq!(status(&mut s), 0, "tick {tick} green");
        assert_eq!(
            credit(&mut s),
            13,
            "segment chain charges exactly 13 at tick {tick}"
        );
    }
    println!(
        "v02 segment chain CPU tick wall: avg {}us / max {wall_max}us (8000us host cap, independent report)",
        wall_sum / 120
    );
    assert!(wall_max < 8000, "8ms host cap");

    // Start pose: runner center x=6, target lo 16 -> nearest entry t=0.5.
    let (hit, node, t, px, nx) = oracle_cast(&boxes, 1, 6.0, 2.0, 26.0, 2.0);
    assert_eq!(hit, 1);
    assert_eq!(node, 2, "target ordinal");
    assert!(approx(t, 0.5, 1e-6), "nearest entry t=0.5: {t}");
    assert!(approx(px, 16.0, 1e-3));
    assert!(approx(nx, -1.0, 0.0), "left-face normal -1");
    assert!(approx(
        fcall(&mut s, "ag_local", &[Int(3), Int(0)]),
        2.0 + node as f32 * 2.5,
        1e-3
    ));
    assert!(approx(
        fcall(&mut s, "ag_local", &[Int(3), Int(1)]),
        7.0 - t * 1.5,
        1e-3
    ));
    assert!(approx(
        fcall(&mut s, "ag_local", &[Int(5), Int(0)]),
        1.0 + px * 0.5,
        1e-3
    ));
    assert_eq!(
        icall(&mut s, "ag_visible", &[Int(4)]),
        1,
        "door visible on hit"
    );
    assert_eq!(icall(&mut s, "ag_visible", &[Int(3)]), 1, "marker shown");

    // Walk right; oracle-check display rows at corridor checkpoints.
    set_action(&mut s, 2, 1, 0);
    let checkpoints = [26usize, 34, 40, 46];
    for tick in 1..=120i32 {
        tick_cpu(&mut s, tick);
        assert_eq!(status(&mut s), 0, "walk tick {tick}");
        if checkpoints.contains(&(tick as usize)) {
            let ax = fcall(&mut s, "ag_world", &[Int(1), Int(0)]);
            let (hit, node, t, px, _) = oracle_cast(&boxes, 1, ax, 2.0, ax + 20.0, 2.0);
            assert!(
                approx(
                    fcall(&mut s, "ag_local", &[Int(3), Int(0)]),
                    2.0 + node as f32 * 2.5,
                    1e-3
                ),
                "marker x at tick {tick} (runner {ax})"
            );
            assert!(
                approx(
                    fcall(&mut s, "ag_local", &[Int(3), Int(1)]),
                    7.0 - t * 1.5,
                    1e-3
                ),
                "marker y at tick {tick} (runner {ax})"
            );
            assert_eq!(
                icall(&mut s, "ag_visible", &[Int(4)]),
                hit,
                "door==hit at tick {tick}"
            );
            assert!(
                approx(
                    fcall(&mut s, "ag_local", &[Int(5), Int(0)]),
                    1.0 + px * 0.5,
                    1e-3
                ),
                "point marker at tick {tick}"
            );
            println!("v02 checkpoint tick {tick}: runner {ax:.3} hit {hit} node {node} t {t:.4}");
        }
    }
}

/// V-02 (chain 3) on the frozen q-changed product: the source-only reach
/// change (20 -> 10) produces different actual query results (boundary
/// t=1.0 vs t=0.5, mid-walk 0.6 vs 0.3) and different display rows.
#[test]
fn v02_changed_reach_produces_real_result_difference() {
    // Q8: the changed source binding really differs (identity), while the
    // scene geometry stays the same.
    assert_ne!(
        sha256_hex(WORLD_SEG.as_bytes()),
        sha256_hex(WORLD_CHG.as_bytes()),
        "changed source hash differs"
    );
    assert_eq!(
        scene_rows(SCENE_SEG).len(),
        scene_rows(SCENE_CHG).len(),
        "same scene geometry"
    );
    let mut a = probe_session(SCENE_SEG, WORLD_SEG, "q-changed-a.at", 1);
    let mut b = probe_session(SCENE_CHG, WORLD_CHG, "q-changed-b.at", 2);
    tick_cpu(&mut a, 1);
    tick_cpu(&mut b, 1);
    assert_eq!(status(&mut a), 0);
    assert_eq!(status(&mut b), 0);
    let ya = fcall(&mut a, "ag_local", &[Int(3), Int(1)]);
    let yb = fcall(&mut b, "ag_local", &[Int(3), Int(1)]);
    println!("v02 changed: marker y reach20={ya} reach10={yb}");
    assert!(approx(ya, 7.0 - 0.5 * 1.5, 1e-3), "reach20 t=0.5 marker y");
    assert!(
        approx(yb, 7.0 - 1.0 * 1.5, 1e-3),
        "reach10 closed-boundary t=1.0 marker y"
    );
    assert!((ya - yb).abs() > 0.5, "actual display difference");

    set_action(&mut a, 2, 1, 0);
    set_action(&mut b, 2, 1, 0);
    for tick in 2..=21i32 {
        tick_cpu(&mut a, tick);
        tick_cpu(&mut b, tick);
    }
    assert_eq!(status(&mut a), 0);
    assert_eq!(status(&mut b), 0);
    let ax = fcall(&mut a, "ag_world", &[Int(1), Int(0)]);
    assert!(approx(ax, 26.0, 0.2), "walked to ~26: {ax}");
    let ya = fcall(&mut a, "ag_local", &[Int(3), Int(1)]);
    let yb = fcall(&mut b, "ag_local", &[Int(3), Int(1)]);
    let ta = (7.0 - ya) / 1.5;
    let tb = (7.0 - yb) / 1.5;
    println!("v02 changed mid-walk: t reach20={ta:.4} reach10={tb:.4}");
    assert!(approx(ta, (32.0 - ax) / 20.0, 2e-3), "far target t reach20");
    assert!(approx(tb, (32.0 - ax) / 10.0, 2e-3), "far target t reach10");
    assert!((ta - tb).abs() > 0.1, "t genuinely differs");
}

/// V-03 (chain 2) on the frozen q-sweep product: zero-displacement miss,
/// approach/stop convergence at wall_lo - half - clearance, start-overlap
/// blocked at t0 (including dx=0), away full travel, origin snapshot
/// invariance, exactly 23 credits per tick (sweep 16 + 6 getters + re-read).
#[test]
fn v03_sweep_chain_cpu_oracle_and_credits() {
    let mut s = probe_session(SCENE_SWEEP, WORLD_SWEEP, "q-sweep.at", 1);
    let boxes = seg_boxes(SCENE_SWEEP);

    tick_cpu(&mut s, 1);
    assert_eq!(status(&mut s), 0);
    assert_eq!(credit(&mut s), 23, "sweep chain charges exactly 23");
    let (hit, node, t, px, nx, overlap) = oracle_sweep_x(2.0, 2.0, 2.0, &boxes, 1, 6.0, 0.0);
    assert_eq!(hit, 0);
    assert_eq!(node, -1);
    assert!(approx(t, 1.0, 0.0));
    assert!(approx(px, 6.0, 1e-3), "miss origin = O+D");
    assert_eq!(overlap, 0);
    let _ = (nx, t);
    assert_eq!(
        icall(&mut s, "ag_visible", &[Int(4)]),
        0,
        "door hidden on miss"
    );
    assert!(
        approx(fcall(&mut s, "ag_world", &[Int(1), Int(0)]), 6.0, 1e-3),
        "no move"
    );

    set_action(&mut s, 2, 1, 0);
    for tick in 2..=8i32 {
        let x0 = fcall(&mut s, "ag_world", &[Int(1), Int(0)]);
        tick_cpu(&mut s, tick);
        assert_eq!(status(&mut s), 0, "sweep tick {tick}");
        assert_eq!(credit(&mut s), 23, "credits stay 23 at tick {tick}");
        let x1 = fcall(&mut s, "ag_world", &[Int(1), Int(0)]);
        let (hit, _, t, px, _, overlap) = oracle_sweep_x(2.0, 2.0, 2.0, &boxes, 1, x0, 100.0);
        let want = if hit == 1 && overlap == 0 {
            x0 + (100.0 * t - 0.02).max(0.0)
        } else if hit == 1 {
            x0
        } else {
            x0 + 100.0
        };
        assert!(
            approx(x1, want, 2e-3),
            "sweep position at tick {tick}: {x1} vs {want}"
        );
        assert!(
            approx(px, x0 + 100.0 * t, 2e-3),
            "origin snapshot at tick {tick}"
        );
        assert_eq!(
            icall(&mut s, "ag_visible", &[Int(4)]),
            1,
            "door visible on contact"
        );
    }
    let stopped = fcall(&mut s, "ag_world", &[Int(1), Int(0)]);
    assert!(
        approx(stopped, 19.5 - 2.0 - 0.02, 2e-3),
        "converged stop {stopped}"
    );
    for tick in 9..=12i32 {
        tick_cpu(&mut s, tick);
        assert_eq!(status(&mut s), 0);
        let x = fcall(&mut s, "ag_world", &[Int(1), Int(0)]);
        assert!(approx(x, stopped, 1e-3), "no penetration after stop: {x}");
    }

    // Away movement: miss -> full requested travel in the signed direction.
    let mut away = probe_session(SCENE_SWEEP, WORLD_SWEEP, "q-sweep-away.at", 2);
    set_action(&mut away, 2, -1, 0);
    tick_cpu(&mut away, 1);
    assert_eq!(status(&mut away), 0);
    let x = fcall(&mut away, "ag_world", &[Int(1), Int(0)]);
    assert!(approx(x, 6.0 - 100.0, 1e-3), "away full move: {x}");

    // Start overlap: blocked at t=0 with start_overlap, including dx=0.
    // The frozen scene places the runner at 6; this variant re-places the
    // runner row at 19.5 (overlapping the wall) through the CPU harness only
    // (fields/boxes fed by hand), reusing the same generated world source.
    let mut ov_rows = scene_rows(SCENE_SWEEP);
    ov_rows[1].3[0] = 19.5;
    let mut ov = probe_session_rows(&ov_rows, WORLD_SWEEP, "q-sweep-ov.at", 3);
    tick_cpu(&mut ov, 1); // dx=0 overlapping
    assert_eq!(status(&mut ov), 0);
    let (hit, _, t, _, nx, overlap) =
        oracle_sweep_x(2.0, 2.0, 2.0, &seg_rows_boxes(&ov_rows), 1, 19.5, 0.0);
    assert_eq!(hit, 1);
    assert!(approx(t, 0.0, 0.0), "overlap t0");
    assert_eq!(overlap, 1, "start_overlap");
    assert!(approx(nx, 0.0, 0.0), "overlap normal 0");
    let _ = hit;
    assert!(
        approx(fcall(&mut ov, "ag_world", &[Int(1), Int(0)]), 19.5, 1e-3),
        "blocked at zero move"
    );
    set_action(&mut ov, 2, 1, 0);
    tick_cpu(&mut ov, 2); // dx=100 while overlapping: still blocked (t0)
    assert_eq!(status(&mut ov), 0);
    assert!(
        approx(fcall(&mut ov, "ag_world", &[Int(1), Int(0)]), 19.5, 1e-3),
        "overlap blocks travel"
    );
}

fn seg_rows_boxes(
    rows: &[(i32, [f32; 3], [f32; 3], [f32; 2], i32, f32, f32)],
) -> Vec<(usize, [f32; 2], [f32; 2])> {
    rows.iter()
        .enumerate()
        .map(|(n, (_, lo, hi, ..))| (n, [lo[0], lo[1]], [hi[0], hi[1]]))
        .collect()
}

/// probe_session over hand-fed rows (overlap variant reuses the frozen world
/// source with a relocated mover row).
fn probe_session_rows(
    rows: &[(i32, [f32; 3], [f32; 3], [f32; 2], i32, f32, f32)],
    world_src: &str,
    name: &str,
    run: i32,
) -> Session {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new(name),
        world_src.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .expect("admission");
    let mut combined = snap.source.clone();
    combined.push_str("\nfn probe_credit() int {return _ag_spatial_calls}\n");
    let mut s = Session::new(&combined).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    for (n, (kind, _, _, pos, visible, hx, hy)) in rows.iter().enumerate() {
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(2), Int(-1)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(*visible as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)
            .unwrap();
        for v in [pos[0], pos[1], 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)
                .unwrap();
        }
        for (i, v) in [1.0, -*hx, -*hy, 0.0, *hx, *hy, 0.0].iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    for n in 0..rows.len() {
        let id = n as i32 + 1;
        s.call("_ag_char_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_obj_map", &[Int(id), Int(n as i32)], ResultKind::Unit)
            .unwrap();
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}

/// V-04 early evidence (chain wiring 2) on the frozen q-cross product: two
/// same-def Querier instances cast and read their own typed results per tick
/// in declaration order; model state stays per instance; 2 casts + 6 getters
/// = 22 credits exactly.
#[test]
fn v04_cross_component_queries_isolated_and_charged() {
    let mut s = probe_session(SCENE_CROSS, WORLD_CROSS, "q-cross.at", 1);
    assert_eq!(status(&mut s), 0, "cross lab start green");
    for tick in 1..=120i32 {
        tick_cpu(&mut s, tick);
        assert_eq!(status(&mut s), 0, "cross tick {tick}");
        assert_eq!(
            credit(&mut s),
            22,
            "2 casts + 6 getters = 22 at tick {tick}"
        );
    }
    assert_eq!(icall(&mut s, "get_qa_hits", &[]), 120, "qa hit every tick");
    assert_eq!(icall(&mut s, "get_qb_hits", &[]), 120, "qb hit every tick");
    assert_eq!(
        icall(&mut s, "get_qa_last_node", &[]),
        1,
        "qa nearest = target ord"
    );
    assert_eq!(
        icall(&mut s, "get_qb_last_node", &[]),
        2,
        "qb nearest = far ord"
    );
    assert!(
        approx(fcall(&mut s, "get_qa_last_t", &[]), 0.4, 1e-3),
        "qa t"
    );
    assert!(
        approx(fcall(&mut s, "get_qb_last_t", &[]), 0.1, 1e-3),
        "qb t"
    );
    // VM guards at the original amounts (independent report).
    println!(
        "v04 cross VM: calls {} max_call_steps {} max_call_slices {}",
        s.calls, s.max_call_steps, s.max_call_slices
    );
    assert!(s.max_call_steps < 100_000, "callback step guard");
    assert!(s.max_call_slices <= 120, "resume/slice guard");
}

// --------------------------------------------------------- GPU sessions ----

type Px = ([f32; 2], [u8; 4]);

fn start_with(scene_text: &str, world_src: &str, tag: &str) -> Host {
    load_code(scene_text, world_src, tag);
    let dir = write_project(scene_text, world_src, tag);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("reload: {e:?}"));
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start run: {e:?}"));
    h.pause_play("");
    settle(&mut h);
    h
}

fn settle(h: &mut Host) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or(false);
        if !pending {
            break;
        }
        assert!(std::time::Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

fn step_action(h: &mut Host, space: i32, x: i32, y: i32) {
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        if p.dispatch(Action {
            space,
            x,
            y,
            toggle: false,
        })
        .is_err()
        {
            return;
        }
    }
    h.pause_play("");
    settle(h);
}

fn actor(h: &Host, id: u64) -> [f32; 3] {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    f.nodes
        .get(&id)
        .map(|d| d.translation)
        .unwrap_or_else(|| panic!("node {id} in frame"))
}

fn play_error(h: &Host) -> String {
    h.play.as_ref().unwrap().error.clone().unwrap_or_default()
}

fn pixel_classes(h: &mut Host) -> Vec<Px> {
    let mut points = Vec::new();
    for gy in 0..95 {
        for gx in 0..200 {
            points.push([(gx * 4 + 2) as f32, (gy * 4 + 2) as f32]);
        }
    }
    let px = h.rt.probe_pixels(h.top, &points).expect("probe");
    points
        .iter()
        .zip(px.iter())
        .map(|(p, c)| (*p, *c))
        .collect()
}

fn class_hits(px: &[Px], class: &str) -> Vec<[f32; 2]> {
    px.iter()
        .filter(|(_, c)| {
            // sRGB-encoded canvas; tolerant relative classes.
            let (r8, g8, b8) = (c[0] as i32, c[1] as i32, c[2] as i32);
            match class {
                "red" => r8 - g8 > 35 && r8 - b8 > 35,
                "white" => r8.min(g8).min(b8) > 200 && r8 - b8 < 35,
                "orange" => r8 > 200 && g8 > 180 && b8 < 160,
                "wall" => {
                    (r8 - g8).abs() < 10 && b8 - r8 >= 5 && b8 - r8 < 25 && r8 > 140 && r8 < 200
                }
                _ => false,
            }
        })
        .map(|(p, _)| *p)
        .collect()
}

fn center(points: &[[f32; 2]], axis: usize) -> f32 {
    assert!(!points.is_empty(), "class absent");
    points.iter().map(|p| p[axis]).sum::<f32>() / points.len() as f32
}

/// V-02 GPU: the marker row visibly moves with the typed result, the door
/// row appears on hit, and the reach-20 vs reach-10 variants render
/// different marker positions from the same scene geometry.
#[test]
fn v02_gpu_segment_marker_chain_and_changed_pixels() {
    let mut h20 = start_with(SCENE_SEG, WORLD_SEG, "gpu-seg20");
    assert!(play_error(&h20).is_empty(), "fault: {}", play_error(&h20));
    let before = pixel_classes(&mut h20);
    let red0 = class_hits(&before, "red");
    let white0 = class_hits(&before, "white");
    assert!(!red0.is_empty(), "declared marker visible at (2, 7)");
    assert!(white0.is_empty(), "door hidden before any tick");
    step_action(&mut h20, 0, 0, 0); // one neutral tick: cast runs, marker maps result
    let after = pixel_classes(&mut h20);
    let red1 = class_hits(&after, "red");
    let white1 = class_hits(&after, "white");
    assert!(!red1.is_empty());
    assert!(!white1.is_empty(), "door visible after hit");
    let dx_px = center(&red1, 0) - center(&red0, 0);
    let dy_px = center(&red1, 1) - center(&red0, 1);
    println!("v02 gpu marker shift px: {dx_px:.1}, {dy_px:.1} (want ~+100, ~+15)");
    assert!(
        (dx_px - 100.0).abs() < 24.0,
        "marker x +5 world * 20px: {dx_px}"
    );
    assert!(
        (dy_px - 15.0).abs() < 24.0,
        "marker 0.75 world lower on screen: {dy_px}"
    );
    let runner20 = actor(&h20, 2);
    h20.stop_play();

    let mut h10 = start_with(SCENE_CHG, WORLD_CHG, "gpu-seg10");
    assert!(play_error(&h10).is_empty(), "fault: {}", play_error(&h10));
    step_action(&mut h10, 0, 0, 0);
    let px10 = pixel_classes(&mut h10);
    let red10 = class_hits(&px10, "red");
    assert!(!red10.is_empty());
    let shift = center(&red10, 1) - center(&red1, 1);
    println!("v02 gpu changed marker y shift px: {shift:.1} (want ~+15)");
    assert!(
        (shift - 15.0).abs() < 24.0,
        "reach10 boundary t=1.0 sits 0.75 world lower"
    );
    let runner10 = actor(&h10, 2);
    assert!(
        approx(runner20[0], runner10[0], 1e-3),
        "same scene, same pose"
    );
    h10.stop_play();
}

/// V-03 GPU: the swept runner visibly stops at the clearance distance before
/// the thin wall; the wall row never moves.
#[test]
fn v03_gpu_sweep_stops_before_wall() {
    let mut h = start_with(SCENE_SWEEP, WORLD_SWEEP, "gpu-sweep");
    assert!(play_error(&h).is_empty(), "fault: {}", play_error(&h));
    for _ in 0..6 {
        step_action(&mut h, 2, 1, 0);
    }
    let runner = actor(&h, 2);
    let wall = actor(&h, 3);
    println!("v03 gpu sweep: runner {:?} wall {:?}", runner, wall);
    assert!(
        approx(runner[0], 19.5 - 2.0 - 0.02, 2e-3),
        "stopped at clearance: {}",
        runner[0]
    );
    assert!(approx(wall[0], 20.0, 1e-3), "wall never moves");
    assert!(
        runner[0] + 2.0 < wall[0] - 0.5,
        "visible gap before the wall face"
    );
    let px = pixel_classes(&mut h);
    let orange = class_hits(&px, "orange");
    let wall_px = class_hits(&px, "wall");
    assert!(!orange.is_empty() && !wall_px.is_empty());
    assert!(
        center(&orange, 0) < center(&wall_px, 0),
        "runner left of wall on canvas"
    );
    h.stop_play();
}

/// V-05 (Q9 budget control) on the frozen q-cross-plus product: the third
/// Querier instance makes the 33rd charged credit of the callback the named
/// sticky fault 10 (3 casts + 9 getters = 33 > 32 shared pool).
#[test]
fn v05_budget_plus_faults_at_the_33rd_charge() {
    let mut s = probe_session(SCENE_CROSS_PLUS, WORLD_CROSS_PLUS, "q-cross-plus.at", 1);
    tick_cpu(&mut s, 1);
    assert_eq!(
        status(&mut s),
        10,
        "named QueryBudget fault at the 33rd charge"
    );
    tick_cpu(&mut s, 2);
    assert_eq!(status(&mut s), 10, "sticky");
}

/// V-04 runtime control (frozen scene rows + minimal variant world): a
/// second query in the same callback invalidates the prior typed token;
/// reading the stale token is the named fault 13.
#[test]
fn v04_second_query_invalidates_prior_token_named_13() {
    const WORLD_STALE: &str = r#"var ok=0.0
var r_reach=20.0
fn main_lifetick(dt float) {
 let ax=ag_world(1,0); let ay=ag_world(1,1)
 let q=ag_cast_segment(2,1,1,ax,ay,0.0,ax+r_reach,ay,0.0)
 let hit=ag_cast_hit(q)
 let q2=ag_cast_segment(2,1,1,ax,ay,0.0,ax+5.0,ay,0.0)
 let stale=ag_cast_hit(q)
}
fn app_start() {ag_char_bind(1)*1.0; ag_camera_follow_2d(1,0.0,-4.0,40.0,12.0,40.0,12.0)*1.0}
fn app_update(dt float) {main_lifetick(dt)}
"#;
    let mut s = probe_session_rows(&scene_rows(SCENE_SEG), WORLD_STALE, "stale.at", 1);
    tick_cpu(&mut s, 1);
    assert_eq!(status(&mut s), 13, "stale token named rejection");
}

/// V-04 runtime control: sweep getters on a cast token charge 1 then reject
/// named 13 (kind guard in the sweep read).
#[test]
fn v04_wrong_kind_getter_named_13() {
    const WORLD_KIND: &str = r#"var ok=0.0
var r_reach=20.0
fn main_lifetick(dt float) {
 let ax=ag_world(1,0); let ay=ag_world(1,1)
 let q=ag_cast_segment(2,1,1,ax,ay,0.0,ax+r_reach,ay,0.0)
 let bad=ag_sweep_origin(q,0)
}
fn app_start() {ag_char_bind(1)*1.0; ag_camera_follow_2d(1,0.0,-4.0,40.0,12.0,40.0,12.0)*1.0}
fn app_update(dt float) {main_lifetick(dt)}
"#;
    let mut s = probe_session_rows(&scene_rows(SCENE_SEG), WORLD_KIND, "kind.at", 1);
    tick_cpu(&mut s, 1);
    assert_eq!(status(&mut s), 13, "wrong-kind named rejection");
}

/// V-04 lifecycle on the query chain (GPU): the pre-tick settle renders
/// without advancing any tick (declared marker/hidden door), each Step
/// advances exactly one tick with input, paused frames never move the
/// runner further, and 20 Stop/fresh rounds rebuild every Run from the
/// declared scene (runner back at 6, door re-hidden, chain re-runs green).
#[test]
fn v04_gpu_pause_step_and_twenty_fresh_rounds_query_chain() {
    {
        let mut h = start_with(SCENE_SEG, WORLD_SEG, "v04-lifecycle");
        assert!(play_error(&h).is_empty(), "fault: {}", play_error(&h));
        // Render-only settle: no tick ran (door still hidden).
        let px = pixel_classes(&mut h);
        assert!(class_hits(&px, "white").is_empty(), "no tick during settle");
        // One input tick: runner walks 1 world unit, cast runs, door opens.
        step_action(&mut h, 2, 1, 0);
        assert!(
            approx(actor(&h, 2)[0], 7.0, 1e-3),
            "one tick moved 60*dt: {}",
            actor(&h, 2)[0]
        );
        let px = pixel_classes(&mut h);
        assert!(
            !class_hits(&px, "white").is_empty(),
            "door visible after tick"
        );
        // Paused frames (settle inside step_action already paused; further
        // frames without resume must not advance the runner).
        for _ in 0..5 {
            settle(&mut h);
        }
        assert!(
            approx(actor(&h, 2)[0], 7.0, 1e-3),
            "paused frames advance nothing"
        );
        h.stop_play();
    }
    for round in 0..20 {
        let mut h = start_with(SCENE_SEG, WORLD_SEG, &format!("v04-fresh-{round}"));
        assert!(
            play_error(&h).is_empty(),
            "round {round} fault: {}",
            play_error(&h)
        );
        step_action(&mut h, 2, 1, 0);
        let x = actor(&h, 2)[0];
        assert!(
            approx(x, 7.0, 1e-3),
            "round {round}: fresh Run rebuilt at declared pose ({x})"
        );
        let px = pixel_classes(&mut h);
        assert!(
            !class_hits(&px, "white").is_empty(),
            "round {round}: chain re-ran (door open)"
        );
        h.stop_play();
    }
}

/// V-06 damaged-consumption control on the frozen q-segment product: the
/// intact product loads and runs green; corrupting ONE byte of the world
/// copy under the scene-bound sha name is rejected at load with the closure
/// identity naming the sha; the exact restore makes the same project load
/// and run green again (the query chain advances with input).
#[test]
fn v06_damaged_queries_world_copy_rejected_then_recovery_green() {
    let sha = SCENE_SEG
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene binds a source sha")
        .to_string();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan021-queries")
        .join(format!("{}-v06-damaged", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), SCENE_SEG).expect("scene");
    let script = scripts.join(format!("{sha}.at"));
    std::fs::write(&script, WORLD_SEG).expect("world copy");

    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("intact load: {e:?}"));
    assert!(code.is_some(), "intact behavior admitted");
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play().expect("intact start");
    h.pause_play("");
    settle(&mut h);
    assert!(
        play_error(&h).is_empty(),
        "intact run fault: {}",
        play_error(&h)
    );
    h.stop_play();

    let mut damaged = WORLD_SEG.as_bytes().to_vec();
    let at = damaged
        .iter()
        .position(|b| *b == b'2')
        .expect("damage site");
    damaged[at] = b'3';
    assert!(damaged != WORLD_SEG.as_bytes());
    std::fs::write(&script, &damaged).expect("damaged world copy");
    let broken = Storage::load_project(&dir.join("scene.agscene"), &Registry::new());
    assert!(
        broken.is_err(),
        "damaged world copy must be rejected at load"
    );
    let msg = format!("{:?}", broken.err().unwrap());
    println!("V-06 damaged load rejected: {msg}");
    assert!(
        msg.contains("closure identity") && msg.contains(&sha),
        "named rejection binding the sha, got: {msg}"
    );

    std::fs::write(&script, WORLD_SEG).expect("restored world copy");
    let (doc2, registry2, code2, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("restored load: {e:?}"));
    assert!(code2.is_some(), "restored behavior admitted");
    let mut h2 = Host::with_project(doc2, registry2, code2).expect("host2");
    h2.start_selected_play().expect("restored start");
    h2.pause_play("");
    settle(&mut h2);
    assert!(
        play_error(&h2).is_empty(),
        "restored run fault: {}",
        play_error(&h2)
    );
    step_action(&mut h2, 2, 1, 0);
    let a = actor(&h2, 2);
    assert!(
        (a[0] - 7.0).abs() < 1e-3,
        "restored chain advances with input: {a:?}"
    );
    h2.stop_play();
}
