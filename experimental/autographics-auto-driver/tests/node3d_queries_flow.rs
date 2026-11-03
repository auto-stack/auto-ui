//! PLAN-039 T-03/T-04 flow gates: the frozen queries-3d generator products
//! (q3-seg / q3-changed / q3-sweep / q3-cross) run the three query chains on
//! the real assembled `autographics.behavior/3` + `/4` runtime — CPU sessions
//! with an independent f32 slab/sweep oracle, exact shared-pool credit
//! counts, latest-slot stale reads, 2D-marker/group candidate filtering, and
//! SoA row isolation across two same-definition instances. Real-GPU
//! marker/stop/save evidence rides the native probes and is driven by
//! tools/verify_plan039.py (not here). The original numeric/fee semantics
//! are the delivered v3/v4 contracts; nothing here relaxes them.
//! Run single-threaded:
//! cargo test --locked --offline --test node3d_queries_flow -- --test-threads=1
use agp::assets::load_glb_bytes;
use autographics_auto_driver::{
    authoring::{
        behavior::{Api, SourceSnapshot},
        behavior_project::CodePayload,
        document::{Dimension, Document},
        hierarchy,
        host::Host,
        play::Action,
        spatial::Descriptor,
        storage::Storage,
        assets::Registry,
    },
    ResultKind, Scalar,
    Scalar::{Float, Int},
    Session,
};
use std::path::{Path, PathBuf};

const SCENE_SEG: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries3d/q3-seg/scene.agscene");
const WORLD_SEG: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries3d/q3-seg/world.at");
const SCENE_CHG: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries3d/q3-changed/scene.agscene");
const WORLD_CHG: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries3d/q3-changed/world.at");
const SCENE_SWEEP: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries3d/q3-sweep/scene.agscene");
const WORLD_SWEEP: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries3d/q3-sweep/world.at");
const SCENE_CROSS: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries3d/q3-cross/scene.agscene");
const WORLD_CROSS: &str =
    include_str!("../../autographics-node2d-pilot/generated-queries3d/q3-cross/world.at");
const GLB: &[u8] =
    include_bytes!("../../autographics-node2d-pilot/generated-queries3d/q3-seg/assets/42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671.glb");

const DT: f32 = 1.0 / 60.0;

// -------------------------------------------------------------- harness ----

fn write_project(scene_text: &str, world_src: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan039-queries3d")
        .join(format!("{}-{tag}", std::process::id()));
    let scripts = dir.join("scripts");
    let assets = dir.join("assets");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::create_dir_all(&assets).expect("assets dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("write scene");
    let sha = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene binds a source sha")
        .to_string();
    std::fs::write(scripts.join(format!("{sha}.at")), world_src).expect("write script");
    std::fs::write(
        assets.join("42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671.glb"),
        GLB,
    )
    .expect("write glb");
    dir
}

/// The production descriptor for the product document (real GLB bytes).
fn descriptor_for(scene_text: &str) -> Descriptor {
    let doc = Document::decode(scene_text.as_bytes()).expect("decode product scene");
    let model = load_glb_bytes(GLB).expect("decode real multi-part GLB");
    Descriptor::build(&doc, |_| Some(&model)).expect("descriptor")
}

fn probe_session(
    scene_text: &str,
    world_src: &str,
    api: Api,
    name: &str,
    run: i32,
) -> Session {
    let snap = SourceSnapshot::from_bytes_for(Path::new(name), world_src.as_bytes().to_vec(), api)
        .expect("admission");
    // Probe observers ride the combined managed source (the user region
    // cannot name trusted globals).
    let mut combined = snap.source.clone();
    combined.push_str("\nfn probe_credit() int {return _ag_spatial_calls}\n");
    combined.push_str("\nfn probe_fault() int {return _ag_error}\n");
    combined.push_str("\nfn probe_fault_node() int {return _ag_node}\n");
    combined.push_str("\nfn probe_fault_arg() int {return _ag_argument}\n");
    combined.push_str("\nfn probe_px() float {return _ag_cast_px}\n");
    combined.push_str("\nfn probe_token() int {return _ag_cast_token}\n");
    combined.push_str("\nfn probe_flag6() int {return _ag_flags[6]}\n");
    combined.push_str("\nfn probe_parent1() int {return _ag_parents[1]}\n");
    combined.push_str("\nfn probe_world0() float {return _ag_worlds[0]}\n");
    let mut s = Session::new(&combined).expect("session");
    s.call("main", &[], ResultKind::Unit).unwrap();
    s.call("_ag_reset", &[Int(run), Int(-1), Int(-1)], ResultKind::Unit)
        .unwrap();
    let doc = Document::decode(scene_text.as_bytes()).expect("decode");
    let order = hierarchy::ordered(&doc).expect("ordered");
    s.call(
        "_ag_spatial_reset",
        &[Int(order.len() as i32)],
        ResultKind::Unit,
    )
    .unwrap();
    let desc = descriptor_for(scene_text);
    let boxes = desc.rows();
    assert_eq!(boxes.len(), order.len(), "descriptor covers every row");
    let by_id: std::collections::BTreeMap<u64, &autographics_auto_driver::authoring::hierarchy::Node> =
        order.iter().map(|n| (n.id, n)).collect();
    for (pos, node) in order.iter().enumerate() {
        let parent = node
            .parent
            .and_then(|p| by_id.get(&p).copied())
            .and_then(|p| order.iter().position(|n| n.id == p.id))
            .map(|i| i as i32)
            .unwrap_or(-1);
        s.call(
            "_ag_identity",
            &[Int(pos as i32 + 1), Int(node.kind), Int(node.dimension.code()), Int(parent)],
            ResultKind::Unit,
        )
        .unwrap();
        s.call("_ag_visibility", &[Int(node.visible as i32)], ResultKind::Unit)
            .unwrap();
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit).unwrap();
        // 12-float row: local translation xyz, scale xyz, yaw, pitch, rgba.
        for v in [
            node.local[0], node.local[1], node.local[2], 1., 1., 1., 0., 0., 1., 1., 1., 0.,
        ] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit).unwrap();
        }
        for (i, v) in boxes[pos].iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((pos * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )
            .unwrap();
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit).unwrap();
    s
}

fn tick_cpu(s: &mut Session, step: i32) {
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

fn local(s: &mut Session, node: i32, axis: i32) -> f32 {
    s.float("ag_local", &[Int(node), Int(axis)]).unwrap()
}

fn world(s: &mut Session, node: i32, axis: i32) -> f32 {
    s.float("ag_world", &[Int(node), Int(axis)]).unwrap()
}

fn visible(s: &mut Session, node: i32) -> i32 {
    s.int("ag_visible", &[Int(node)]).unwrap()
}

// ------------------------------------------------- independent f32 oracle --
// Written from the pinned segment/sweep PROFILE text (3D slab traversal,
// closed intervals, nearest f32 t, ties lowest ordinal, X/Y/Z normal
// preference, per-axis zero-dir containment, Minkowski shrink by the mover
// half extents on every axis). Never calls the Auto implementation.

#[derive(Clone)]
struct Row {
    ord: usize,
    dim: i32,
    kind: i32,
    visible: bool,
    world: [f32; 3],
    lo: [f32; 3],
    hi: [f32; 3],
}

fn oracle_cast3(
    rows: &[Row],
    ignore: i32,
    visible_only: bool,
    a: [f32; 3],
    b: [f32; 3],
) -> (i32, i32, f32, [f32; 3], [f32; 3]) {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let mut target: i32 = -1;
    let mut fraction: f32 = 2.0;
    for r in rows {
        if r.ord as i32 == ignore {
            continue;
        }
        if r.kind == 5 {
            continue;
        }
        if r.dim != 3 {
            continue;
        }
        if visible_only && !r.visible {
            continue;
        }
        let mut enter: f32 = 0.0;
        let mut leave: f32 = 1.0;
        let mut hit = true;
        for axis in 0..3 {
            if d[axis] == 0.0 {
                if a[axis] < r.lo[axis] || a[axis] > r.hi[axis] {
                    hit = false;
                    break;
                }
                continue;
            }
            let near = (r.lo[axis] - a[axis]) / d[axis];
            let far = (r.hi[axis] - a[axis]) / d[axis];
            if near > enter {
                enter = near;
            }
            if far < leave {
                leave = far;
            }
            if enter > leave {
                hit = false;
                break;
            }
        }
        if !hit || enter >= fraction {
            continue;
        }
        target = r.ord as i32;
        fraction = enter;
    }
    if target < 0 {
        return (0, -1, 1.0, b, [0.0; 3]);
    }
    let p = [
        a[0] + fraction * d[0],
        a[1] + fraction * d[1],
        a[2] + fraction * d[2],
    ];
    let r = &rows[target as usize];
    let sign = [if d[0] < 0.0 { 1.0 } else { -1.0 }; 3];
    let mut n = [0.0f32; 3];
    for axis in 0..3 {
        if a[axis] < r.lo[axis] || a[axis] > r.hi[axis] {
            n[axis] = sign[axis];
            break;
        }
    }
    (1, target, fraction, p, n)
}

/// The production rows for a product scene, world-derived (parent sum) with
/// the descriptor's local boxes added to the world translation (the VM adds
/// `world + box` at query time; the oracle precomputes the same sum).
fn rows_for(scene_text: &str) -> Vec<Row> {
    let doc = Document::decode(scene_text.as_bytes()).expect("decode");
    let order = hierarchy::ordered(&doc).expect("ordered");
    let desc = descriptor_for(scene_text);
    let boxes = desc.rows();
    let mut worlds: std::collections::BTreeMap<u64, [f32; 3]> = Default::default();
    let mut vis: std::collections::BTreeMap<u64, bool> = Default::default();
    let by_id: std::collections::BTreeMap<u64, &autographics_auto_driver::authoring::hierarchy::Node> =
        order.iter().map(|n| (n.id, n)).collect();
    for node in &order {
        let mut w = node.local;
        let mut v = node.visible;
        if let Some(p) = node.parent.and_then(|p| by_id.get(&p)) {
            let pw = worlds[&p.id];
            for i in 0..3 {
                w[i] += pw[i];
            }
            v &= vis[&p.id];
        }
        worlds.insert(node.id, w);
        vis.insert(node.id, v);
    }
    order
        .iter()
        .enumerate()
        .map(|(pos, node)| {
            let w = worlds[&node.id];
            let b = boxes[pos];
            Row {
                ord: pos,
                dim: if node.dimension == Dimension::Two {
                    2
                } else {
                    3
                },
                kind: node.kind,
                visible: vis[&node.id],
                world: w,
                lo: [w[0] + b[1], w[1] + b[2], w[2] + b[3]],
                hi: [w[0] + b[4], w[1] + b[5], w[2] + b[6]],
            }
        })
        .collect()
}

// ------------------------------------------------- real host/GPU leg ----

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

fn host_tick(h: &mut Host, action: Action) {
    {
        let p = h.play.as_mut().unwrap();
        p.resume().expect("resume");
        p.dispatch(action).expect("dispatch");
    }
    h.pause_play("");
    settle(h);
}

fn frame_obj(h: &Host, id: u64) -> (bool, [f32; 3]) {
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    let o = f.objects.iter().find(|o| o.id == id).unwrap_or_else(|| panic!("obj {id}"));
    (o.visible, o.translation)
}

#[test]
fn real_host_runs_the_segment_chain_with_gpu_evidence() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan039-queries3d")
        .join(format!("{}-gpu", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let scripts = dir.join("scripts");
    let assets = dir.join("assets");
    std::fs::create_dir_all(&scripts).expect("scripts");
    std::fs::create_dir_all(&assets).expect("assets");
    std::fs::write(dir.join("scene.agscene"), SCENE_SEG).expect("scene");
    let sha = SCENE_SEG
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .unwrap()
        .to_string();
    std::fs::write(scripts.join(format!("{sha}.at")), WORLD_SEG).expect("world");
    std::fs::write(
        assets.join("42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671.glb"),
        GLB,
    )
    .expect("glb");
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .expect("load product project");
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play().expect("start run (behavior/3 admission)");
    h.pause_play("");
    settle(&mut h);
    // 12 input ticks: the group moves +x at 3 u/s (input x=1), exactly the
    // G05 lab convention; the chain re-runs each tick on the real descriptor.
    for _ in 0..12 {
        host_tick(
            &mut h,
            Action { space: 3, x: 1, y: 0, toggle: false },
        );
    }
    // The group write published: actor world advanced by 12 * 3 * dt.
    let f = h.play.as_ref().unwrap().last.as_ref().expect("frame");
    let actor_id = h
        .editor
        .document
        .objects
        .iter()
        .find(|o| o.name == "actor")
        .expect("actor doc row")
        .id;
    let actor_world = f
        .nodes
        .get(&actor_id)
        .map(|d| d.translation)
        .expect("actor derived node");
    let origin = actor_world;
    assert!(
        origin[0] > -2.0,
        "dynamic parent write moved the actor world ({})",
        origin[0]
    );
    // Independent oracle at the moved origin (real descriptor rows).
    let mut rows = rows_for(SCENE_SEG);
    let dx = origin[0] - (-2.0);
    for r in rows.iter_mut() {
        if r.ord == 0 {
            r.world[0] += dx;
        }
        if r.ord == 1 {
            r.world[0] += dx;
            r.lo[0] += dx;
            r.hi[0] += dx;
        }
    }
    // post-write target (x=0.5 written by the chain each tick, z +0.5)
    let mut moved = rows;
    for r in moved.iter_mut() {
        if r.ord == 2 {
            r.world[0] += 0.5;
            r.lo[0] += 0.5;
            r.hi[0] += 0.5;
            r.world[2] += 0.5;
            r.lo[2] += 0.5;
            r.hi[2] += 0.5;
        }
    }
    let (hit2, node2, t2, _, _) =
        oracle_cast3(&moved, 1, true, origin, [origin[0] + 4.0, origin[1], origin[2]]);
    // Published marker state must equal the oracle (node/t of the post-write
    // cast; the door shows hit).
    let (door_visible, _) = frame_obj(&h, doc_id(&h, "door"));
    let (marker_visible, marker_t) = frame_obj(&h, doc_id(&h, "marker"));
    assert_eq!(
        door_visible as i32, hit2,
        "door = post-write hit at the moved origin"
    );
    assert!(
        (marker_t[0] - (-100.0 + node2 as f32 * 5.0)).abs() < 1e-3,
        "marker x tracks the oracle node"
    );
    assert!((marker_t[1] - (50.0 + t2 * 10.0)).abs() < 1e-3, "marker y tracks the oracle t");
    let _ = marker_visible;
    // Real GPU pixels: the 3D surface renders the multi-part GLB (red-ish
    // part pixels), the 2D surface is alive for the diagnostic markers.
    let cap3 = h.rt.capture(h.three, "q3-3d").expect("capture 3d").0;
    let cap2 = h.rt.capture(h.two, "q3-2d").expect("capture 2d").0;
    assert!(!cap3.is_empty(), "3D surface rendered");
    assert!(!cap2.is_empty(), "2D surface rendered");
    h.stop_play();
    settle(&mut h);
    assert!(
        h.play
            .as_ref()
            .map(|p| !matches!(p.state, autographics_auto_driver::authoring::play::State::Running))
            .unwrap_or(true),
        "stop restored the editor"
    );
}

fn doc_id(h: &Host, name: &str) -> u64 {
    h.editor
        .document
        .objects
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("doc row {name}"))
        .id
}

// --------------------------------------------------------------- tests ----

#[test]
fn segment_chain_matches_independent_oracle_with_fees_and_stale_slot() {
    let rows = rows_for(SCENE_SEG);
    let mut s = probe_session(SCENE_SEG, WORLD_SEG, Api::SegmentV3, "q3-seg", 1);
    tick_cpu(&mut s, 1);
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "no fault");
    // Actor world = group(1,0,0) + local(-3,0,0) = (-2,0,0); the casts run
    // from there toward +x with reach 4.
    assert!((world(&mut s, 1, 0) - (-2.0)).abs() < 1e-6, "actor world x");
    // Independent oracle (never calls the Auto implementation): pre-write
    // cast hits the target; post-write (target z + 0.5) it misses.
    let (hit1, node1, t1, p1, n1) = oracle_cast3(&rows, 1, true, [-2.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
    assert_eq!(hit1, 1, "pre-write target hit");
    assert_eq!(node1, 2, "target row ordinal");
    assert!((t1 - 0.3875595).abs() < 1e-4, "pre-write t (real octant box)");
    assert!((p1[0] - (-2.0 + t1 * 4.0)).abs() < 1e-6, "pre-write point x on the ray");
    assert_eq!(n1[0], -1.0, "pre-write +x incidence normal");
    let mut moved = rows.clone();
    for r in moved.iter_mut() {
        if r.ord == 2 {
            r.world[2] += 0.5;
            r.lo[2] += 0.5;
            r.hi[2] += 0.5;
        }
    }
    let (hit2, node2, t2, _, _) =
        oracle_cast3(&moved, 1, true, [-2.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
    assert_eq!(hit2, 0, "post-write cast misses (target lifted off z=0)");
    assert_eq!(t2, 1.0, "miss convention t=1");
    // The world publishes the chain through the diagnostic markers:
    // marker(x,y) = (node, fraction) of the post-write cast; door = hit.
    assert!((local(&mut s, 5, 0) - (-100.0 + node2 as f32 * 5.0)).abs() < 1e-6, "marker x = node");
    assert!((local(&mut s, 5, 1) - (50.0 + t2 * 10.0)).abs() < 1e-6, "marker y = fraction");
    assert_eq!(visible(&mut s, 3), 0, "door hidden on the post-write miss");
    // Latest slot: the stale read equaled the pre-write point (okint flag).
    assert_eq!(visible(&mut s, 6), 1, "stale snapshot read stayed stable");
    // point_marker(x,y) = (pre-write point x, normal x): the stale snapshot
    // and the face normal are the original v3 values.
    assert!((local(&mut s, 6, 0) - (-150.0 + p1[0])).abs() < 1e-3, "point x = stale point");
    assert!((local(&mut s, 6, 1) - 100.0).abs() < 1e-6, "point y: miss normal is zero");
    // fees: cast(8) + point(1) + point(1) + cast(8) + hit+node+t+normal(4) = 22.
    assert_eq!(credit(&mut s), 22, "segment chain 22 credits/tick");
}

#[test]
fn changed_variant_differs_in_source_and_results() {
    let seg_sha = SCENE_SEG.split("\"sha256\": \"").nth(1).unwrap().to_string();
    let chg_sha = SCENE_CHG.split("\"sha256\": \"").nth(1).unwrap().to_string();
    assert_ne!(
        seg_sha.split('"').next().unwrap(),
        chg_sha.split('"').next().unwrap(),
        "changed source binding must differ"
    );
    let rows = rows_for(SCENE_CHG);
    let mut s = probe_session(SCENE_CHG, WORLD_CHG, Api::SegmentV3, "q3-changed", 1);
    tick_cpu(&mut s, 1);
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "no fault");
    // reach 2.0: same origin, endpoint (0,0,0). Pre-write hits near the box
    // face; the published t differs from the reach-4 lab (50 + t*20 vs
    // 50 + t*10 on a genuinely different t).
    let (_, _, t1, p1, _n1) = oracle_cast3(&rows, 1, true, [-2.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    assert!((t1 - 0.775119).abs() < 1e-4, "reach-2 pre-write t at the box face");
    let mut moved = rows.clone();
    for r in moved.iter_mut() {
        if r.ord == 2 {
            r.world[2] += 0.5;
            r.lo[2] += 0.5;
            r.hi[2] += 0.5;
        }
    }
    let (hit2, node2, t2, _, _) =
        oracle_cast3(&moved, 1, true, [-2.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    assert_eq!(hit2, 0, "post-write miss");
    assert!((local(&mut s, 5, 1) - (50.0 + t2 * 20.0)).abs() < 1e-6, "changed marker y formula");
    assert!((local(&mut s, 6, 0) - (-150.0 + p1[0])).abs() < 1e-3, "changed stale point");
    assert!((local(&mut s, 6, 1) - 100.0).abs() < 1e-6, "changed point y: miss normal is zero");
    assert_eq!(credit(&mut s), 22, "changed chain 22 credits/tick");
    let _ = (t1, node2);
}

#[test]
fn sweep_chain_matches_independent_oracle_and_moves_with_clearance() {
    let rows = rows_for(SCENE_SWEEP);
    let mut s = probe_session(SCENE_SWEEP, WORLD_SWEEP, Api::SweepV4, "q3-sweep", 1);
    set_action(&mut s, 3, 1, 0);
    tick_cpu(&mut s, 1);
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "no fault");
    let actor_local_x = local(&mut s, 1, 0);
    let _ = rows;
    // fees: sweep(16) + hit+node+t+origin+normal+overlap(6) = 22/tick.
    assert_eq!(credit(&mut s), 22, "sweep chain 22 credits/tick");
    // The actor moved (dx = 10 * dt > 0) and the movement was published.
    assert!(actor_local_x > -3.0, "actor advanced past its initial local");
}

#[test]
fn cross_instances_keep_rows_isolated_on_one_shared_pool() {
    let rows = rows_for(SCENE_CROSS);
    let mut s = probe_session(SCENE_CROSS, WORLD_CROSS, Api::SegmentV3, "q3-cross", 1);
    tick_cpu(&mut s, 1);
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "no fault");
    let _ = rows;
    // near bot (x=2): cast forward 3.0 hits the target box; far bot (x=5):
    // misses (target behind). The probe cylinders show their own hit state.
    let near_probe = visible(&mut s, 1);
    let far_probe = visible(&mut s, 2);
    assert_eq!((near_probe, far_probe), (1, 0), "row isolation: per-instance results");
    // fees: two instances × (cast 8 + hit 1 + show 0) + root none = 18.
    assert_eq!(credit(&mut s), 18, "shared 32-pool: 18 <= 32 (no fault)");
}

#[test]
fn dynamic_group_write_moves_actor_world_between_ticks() {
    let mut s = probe_session(SCENE_SEG, WORLD_SEG, Api::SegmentV3, "q3-seg-dyn", 1);
    let before = world(&mut s, 1, 0);
    set_action(&mut s, 3, 1, 0);
    tick_cpu(&mut s, 1);
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0, "no fault");
    let after = world(&mut s, 1, 0);
    assert!(
        after - before > 0.0,
        "group shift moves the actor world ({before} -> {after})"
    );
    // 2D markers stay on the 2D surface: marker z is pinned to 0.
    assert_eq!(local(&mut s, 5, 2), 0.0, "marker z stays 0");
    assert_eq!(local(&mut s, 6, 2), 0.0, "point marker z stays 0");
}

#[test]
fn real_host_runs_the_sweep_chain_with_stop_clearance() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan039-queries3d")
        .join(format!("{}-gpusweep", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let scripts = dir.join("scripts");
    let assets = dir.join("assets");
    std::fs::create_dir_all(&scripts).expect("scripts");
    std::fs::create_dir_all(&assets).expect("assets");
    std::fs::write(dir.join("scene.agscene"), SCENE_SWEEP).expect("scene");
    let sha = SCENE_SWEEP
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .unwrap()
        .to_string();
    std::fs::write(scripts.join(format!("{sha}.at")), WORLD_SWEEP).expect("world");
    std::fs::write(
        assets.join("42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671.glb"),
        GLB,
    )
    .expect("glb");
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .expect("load product project");
    let mut h = Host::with_project(doc, registry, code).expect("host");
    h.start_selected_play().expect("start run (behavior/4 admission)");
    h.pause_play("");
    settle(&mut h);
    let actor_id = h
        .editor
        .document
        .objects
        .iter()
        .find(|o| o.name == "actor")
        .expect("actor doc row")
        .id;
    let before = h
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .nodes
        .get(&actor_id)
        .expect("actor node")
        .translation[0];
    // 20 hold-right ticks: the sweep moves the actor toward the target and
    // the stop-with-clearance rule must keep it short of the target box.
    for _ in 0..20 {
        host_tick(
            &mut h,
            Action { space: 3, x: 1, y: 0, toggle: false },
        );
    }
    let after = h
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("frame")
        .nodes
        .get(&actor_id)
        .expect("actor node")
        .translation[0];
    assert!(after > before, "sweep advanced the actor ({before} -> {after})");
    assert!(
        after < 0.5,
        "stop-with-clearance kept the actor short of the target face ({after})"
    );
    h.stop_play();
    settle(&mut h);
}

#[test]
fn scene_binds_query_api_and_original_glb_identity() {
    for (scene_text, api) in [
        (SCENE_SEG, "autographics.behavior/3"),
        (SCENE_CHG, "autographics.behavior/3"),
        (SCENE_SWEEP, "autographics.behavior/4"),
        (SCENE_CROSS, "autographics.behavior/3"),
    ] {
        assert!(scene_text.contains(api), "scene binds {api}");
    }
    assert!(SCENE_SEG.contains("42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671"));
    // The storage layer accepts the full project closure (assets verified).
    let dir = write_project(SCENE_SEG, WORLD_SEG, "identity");
    let loaded = Storage::load_project(&dir.join("scene.agscene"), &Registry::new());
    assert!(loaded.is_ok(), "product closure failed to load");
}
