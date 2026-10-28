//! PLAN-860 game-runtime feasibility probe: real Auto Session execution of the
//! frozen G2/G3 scenes, routes, 128-node loads and semantic cases from
//! docs/plans/attachments/860/work/probe-contract.md (r3). The candidates are
//! the existing behavior/5 composition (A) and the probe-only trusted support
//! unit (B); neither changes v5 files, budgets or public identity.
use crate::{
    authoring::behavior::{Api, SourceSnapshot},
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
use std::time::Instant;

pub const G2_ROUTE_A: &str = include_str!("../../research/game-runtime/g2_route_a.at");
pub const G2_ROUTE_B: &str = include_str!("../../research/game-runtime/g2_route_b.at");
pub const G3_ROUTE_A: &str = include_str!("../../research/game-runtime/g3_route_a.at");
pub const G3_ROUTE_B: &str = include_str!("../../research/game-runtime/g3_route_b.at");
pub const PROBE_UNIT: &str = include_str!("../../research/game-runtime/character_probe_unit.at");

/// Frozen per-contract: clearance for both candidates, B support skin is inside
/// the probe unit (0.01 down-scan). dt is the frozen fixed logical step.
pub const CLEARANCE: f32 = 0.002;
pub const DT: f32 = 1.0 / 60.0;
pub const MAX_ROUTE_STEPS: usize = 1800;
pub const G2_STEPS: usize = 400;
pub const G3_STEPS: usize = 410;

/// The frozen candidate-A grounded determination, shared verbatim by the route
/// sources and the semantic-case oracles. Tests assert this exact text occurs
/// in every A source so the rule under measurement cannot drift.
pub const GROUNDED_RULE_A: &str = " grounded=0
 if st==1 {grounded=1}
 if st==2 {grounded=1}
 if cc<1 {grounded=0}
 if dy>=0.0 {grounded=0}
 if uy>=0.0 {grounded=0}";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Candidate {
    A,
    B,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    G2,
    G3,
}

/// Static scene row: (kind, dimension, [valid, lo0, lo1, lo2, hi0, hi1, hi2]).
#[derive(Clone, Copy, Debug)]
pub struct Row(pub i32, pub i32, pub [f32; 7]);

fn row(kind: i32, dim: i32, lo: [f32; 3], hi: [f32; 3]) -> Row {
    Row(kind, dim, [1., lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]])
}

/// G2 static colliders per contract §2 (character + 7 bodies).
pub fn g2_rows() -> Vec<Row> {
    vec![
        row(2, 2, [-0.4, -0.5, 0.], [0.4, 0.5, 0.]),
        row(2, 2, [0., -1., 0.], [8., 0., 0.]),
        row(2, 2, [10., -1., 0.], [18., 0., 0.]),
        row(2, 2, [20., -1., 0.], [30., 0., 0.]),
        row(2, 2, [12., 1., 0.], [15., 1.25, 0.]),
        row(2, 2, [-2., -4., 0.], [0., 10., 0.]),
        row(2, 2, [30., -4., 0.], [32., 10., 0.]),
        row(2, 2, [-2., 10., 0.], [32., 12., 0.]),
    ]
}

/// G3 static colliders per contract §2 (character + 8 bodies).
pub fn g3_rows() -> Vec<Row> {
    vec![
        row(2, 3, [-0.4, -0.5, -0.4], [0.4, 0.5, 0.4]),
        row(2, 3, [0., -1., 0.], [8., 0., 8.]),
        row(2, 3, [10., -1., 0.], [18., 0., 8.]),
        row(2, 3, [20., -1., 0.], [28., 0., 8.]),
        row(2, 3, [-2., -4., -2.], [0., 10., 10.]),
        row(2, 3, [28., -4., -2.], [30., 10., 10.]),
        row(2, 3, [-2., -4., -2.], [30., 10., 0.]),
        row(2, 3, [-2., -4., 8.], [30., 10., 10.]),
        row(2, 3, [-2., 10., -2.], [30., 12., 10.]),
    ]
}

/// 128-node load: character, 126 candidate boxes, nearest blocker LAST (node
/// 127). Wide: all roots, candidates on the sweep corridor. Deep: candidates
/// chained above the corridor (iterated but never hit), blocker offset back to
/// the corridor so nearest-wins holds against iteration order.
pub fn load_rows(deep: bool) -> Vec<Row> {
    let mut rows = vec![row(2, 2, [-0.4, -0.5, 0.], [0.4, 0.5, 0.])];
    // Field translations are all zero, so every world row is zero in both
    // layouts and the configured box offsets are the world geometry. Deep
    // differs only in the derive cost of the 126-link parent chain; its
    // candidates sit above the corridor (iterated, never hit).
    for i in 0..126 {
        let x = 3.0 + 0.3 * i as f32;
        let (y0, y1) = if deep { (5., 6.) } else { (-1., 1.) };
        rows.push(row(2, 2, [x, y0, 0.], [x + 0.5, y1, 0.]));
    }
    // Nearest blocker, last iterated (node 127), world x [2,2.5].
    rows.push(row(2, 2, [2.0, -1.0, 0.], [2.5, 1.0, 0.]));
    rows
}

/// Trusted probe getters appended after assembly (support/motion.rs pattern).
/// A tail reads only v1..5 state; B tail adds the probe-unit summary.
fn probe_tail(candidate: Candidate) -> &'static str {
    match candidate {
        Candidate::A => {
            "\nfn probe_x() float {return _ag_worlds[0]}\nfn probe_y() float {return _ag_worlds[1]}\nfn probe_z() float {return _ag_worlds[2]}\nfn probe_grounded() int {return grounded}\nfn probe_step() int {return stepn}\nfn probe_credit() int {return _ag_spatial_calls}\nfn probe_status() int {return _ag_status}\nfn probe_ms() int {return _ag_motion_state}\nfn probe_found() int {return _ag_cast_found}\nfn probe_initial() int {return _ag_motion_initial}\nfn probe_checked() int {return _ag_motion_checked}\n"
        }
        Candidate::B => {
            "\nfn probe_x() float {return _ag_worlds[0]}\nfn probe_y() float {return _ag_worlds[1]}\nfn probe_z() float {return _ag_worlds[2]}\nfn probe_grounded() int {return grounded}\nfn probe_step() int {return stepn}\nfn probe_credit() int {return _ag_spatial_calls}\nfn probe_status() int {return _ag_status}\nfn probe_ms() int {return _ag_motion_state}\nfn probe_found() int {return _ag_cast_found}\nfn probe_initial() int {return _ag_motion_initial}\nfn probe_checked() int {return _ag_motion_checked}\nfn probe_scans() int {return _agp_scans}\nfn probe_support() int {return _agp_support}\n"
        }
    }
}

pub fn route_user(route: Route, candidate: Candidate) -> &'static str {
    match (route, candidate) {
        (Route::G2, Candidate::A) => G2_ROUTE_A,
        (Route::G2, Candidate::B) => G2_ROUTE_B,
        (Route::G3, Candidate::A) => G3_ROUTE_A,
        (Route::G3, Candidate::B) => G3_ROUTE_B,
    }
}

pub fn route_name(route: Route, candidate: Candidate) -> &'static str {
    match (route, candidate) {
        (Route::G2, Candidate::A) => "research/game-runtime/g2_route_a.at",
        (Route::G2, Candidate::B) => "research/game-runtime/g2_route_b.at",
        (Route::G3, Candidate::A) => "research/game-runtime/g3_route_a.at",
        (Route::G3, Candidate::B) => "research/game-runtime/g3_route_b.at",
    }
}

/// Assemble a combined probe source: v5 managed units + frozen user source +
/// (candidate B) the appended trusted probe unit + probe getters.
pub fn combined_source(
    user: &str,
    name: &str,
    candidate: Candidate,
) -> Result<SourceSnapshot, String> {
    let mut snap = SourceSnapshot::from_bytes_for(
        std::path::Path::new(name),
        user.as_bytes().to_vec(),
        Api::MotionV5,
    )?;
    if candidate == Candidate::B {
        snap.source.push_str(PROBE_UNIT);
    }
    snap.source.push_str(probe_tail(candidate));
    Ok(snap)
}

/// Build the Session for a combined source and initialize the static scene.
pub fn session_for(
    user: &str,
    name: &str,
    candidate: Candidate,
    rows: &[Row],
    deep: bool,
) -> Result<Session, String> {
    let snap = combined_source(user, name, candidate)?;
    let mut s = Session::new(&snap.source)?;
    s.call("main", &[], ResultKind::Unit)?;
    s.call("_ag_reset", &[Int(1), Int(-1), Int(-1)], ResultKind::Unit)?;
    s.call(
        "_ag_spatial_reset",
        &[Int(rows.len() as i32)],
        ResultKind::Unit,
    )?;
    for (n, Row(kind, dim, box7)) in rows.iter().enumerate() {
        let parent = if n == 0 {
            -1
        } else if deep {
            n as i32 - 1
        } else {
            -1
        };
        s.call(
            "_ag_identity",
            &[Int(n as i32 + 1), Int(*kind), Int(*dim), Int(parent)],
            ResultKind::Unit,
        )?;
        s.call("_ag_visibility", &[Int(1)], ResultKind::Unit)?;
        s.call("_ag_bind_resource", &[Int(0)], ResultKind::Unit)?;
        for v in [0., 0., 0., 1., 1., 1., 0., 0., 1., 1., 1., 0.] {
            s.call("_ag_append_field", &[Float(v)], ResultKind::Unit)?;
        }
        for (i, v) in box7.iter().enumerate() {
            s.call(
                "_ag_spatial_config",
                &[Int((n * 7 + i) as i32), Float(*v)],
                ResultKind::Unit,
            )?;
        }
    }
    s.call("_ag_start", &[], ResultKind::Unit)?;
    Ok(s)
}

/// One logical step: exactly one character-update callback plus separately
/// timed trusted readbacks. Yields/resumes and credits are the callback's own.
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct StepSample {
    pub step: usize,
    pub tick_ns: u64,
    pub reads_ns: u64,
    pub resumes: u64,
    pub credits: i32,
    pub scans: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub grounded: i32,
    pub support: i32,
}

/// Run `steps` frozen logical steps, recording per-step samples.
pub fn run_steps(s: &mut Session, steps: usize, reads: &[&str]) -> Result<Vec<StepSample>, String> {
    let mut out = Vec::with_capacity(steps.min(MAX_ROUTE_STEPS + 1));
    for step in 1..=steps {
        let t0 = Instant::now();
        s.call("_ag_tick", &[Int(step as i32), Float(DT)], ResultKind::Unit)?;
        let tick_ns = t0.elapsed().as_nanos() as u64;
        let resumes = s.yielded;
        let r0 = Instant::now();
        let x = s.float("probe_x", &[])?;
        let y = s.float("probe_y", &[])?;
        let z = s.float("probe_z", &[])?;
        let grounded = s.int("probe_grounded", &[])?;
        let credits = s.int("probe_credit", &[])?;
        let mut scans = 0;
        let mut support = -1;
        for name in reads {
            match *name {
                "scans" => scans = s.int("probe_scans", &[])?,
                "support" => support = s.int("probe_support", &[])?,
                other => return Err(format!("unknown probe read {other}")),
            }
        }
        let reads_ns = r0.elapsed().as_nanos() as u64;
        out.push(StepSample {
            step,
            tick_ns,
            reads_ns,
            resumes,
            credits,
            scans,
            x,
            y,
            z,
            grounded,
            support,
        });
    }
    Ok(out)
}

/// Nearest-rank percentile on raw samples (ceil(q*N)-1, research baseline §3).
pub fn percentile(mut values: Vec<u64>, q: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let rank = ((q * values.len() as f64).ceil() as usize).clamp(1, values.len());
    values[rank - 1]
}

/// Deterministic G2 enemy model per contract §2 (starts x=11 ascending, 1 u/s
/// ping-pong 11..17, period 720 steps).
pub fn enemy_x(step: usize) -> f32 {
    let u = (step % 720) as f32 / 60.0;
    let t = if u < 6.0 { u } else { 12.0 - u };
    11.0 + t
}

/// Box-vs-circle trigger test (character half extents against r=0.5 region).
pub fn box_circle(
    px: f32,
    py: f32,
    pz: f32,
    cx: f32,
    cy: f32,
    cz: f32,
    half: [f32; 3],
    r: f32,
) -> bool {
    let dx = (px - cx).abs() - half[0];
    let dy = (py - cy).abs() - half[1];
    let dz = (pz - cz).abs() - half[2];
    let dx = dx.max(0.0);
    let dy = dy.max(0.0);
    let dz = dz.max(0.0);
    dx * dx + dy * dy + dz * dz <= r * r
}

/// Route reachability/violation report from actual recorded samples.
#[derive(Default, Debug, serde::Serialize)]
pub struct RouteReport {
    pub coins_reached: [bool; 3],
    pub coin_steps: [Option<usize>; 3],
    pub goal_reached: bool,
    pub goal_step: Option<usize>,
    pub enemy_contact_step: Option<usize>,
    pub hazard_contact_step: Option<usize>,
    pub pit_step: Option<usize>,
    pub grounded_flips: usize,
    pub status_fault: bool,
}

pub fn route_report(route: Route, samples: &[StepSample]) -> RouteReport {
    let mut rep = RouteReport::default();
    let half = if route == Route::G2 {
        [0.4, 0.5, 0.]
    } else {
        [0.4, 0.5, 0.4]
    };
    let coins: [(f32, f32, f32); 3] = if route == Route::G2 {
        [(4., 1., 0.), (13., 2.25, 0.), (24., 1., 0.)]
    } else {
        [(4., 1., 4.), (12., 1., 2.), (24., 1., 4.)]
    };
    let goal = if route == Route::G2 {
        (28., 1., 0.)
    } else {
        (26., 1., 6.)
    };
    let mut prev_grounded = 0;
    for s in samples {
        for i in 0..3 {
            if !rep.coins_reached[i]
                && box_circle(s.x, s.y, s.z, coins[i].0, coins[i].1, coins[i].2, half, 0.5)
            {
                rep.coins_reached[i] = true;
                rep.coin_steps[i] = Some(s.step);
            }
        }
        if !rep.goal_reached && box_circle(s.x, s.y, s.z, goal.0, goal.1, goal.2, half, 0.5) {
            rep.goal_reached = true;
            rep.goal_step = Some(s.step);
        }
        if route == Route::G2 {
            let ex = enemy_x(s.step);
            let dx = (s.x - ex).abs() - 0.8;
            let dy = (s.y - 0.5).abs() - 1.0;
            if dx < 0.0 && dy < 0.0 && rep.enemy_contact_step.is_none() {
                rep.enemy_contact_step = Some(s.step);
            }
            if s.y < -3.0 && rep.pit_step.is_none() {
                rep.pit_step = Some(s.step);
            }
        } else {
            let dx = (s.x - 15.).abs() - (0.4 + 1.0);
            let dy = (s.y - 0.5).abs() - (0.5 + 0.5);
            let dz = (s.z - 4.).abs() - (0.4 + 1.0);
            if dx < 0.0 && dy < 0.0 && dz < 0.0 && rep.hazard_contact_step.is_none() {
                rep.hazard_contact_step = Some(s.step);
            }
            if s.y < -4.0 && rep.pit_step.is_none() {
                rep.pit_step = Some(s.step);
            }
        }
        if s.grounded != prev_grounded {
            rep.grounded_flips += 1;
        }
        prev_grounded = s.grounded;
    }
    rep
}

/// 128-load neutral tick source (frozen): move +8 x, expect the last-iterated
/// blocker; candidate summaries are asserted from recorded samples.
pub fn load_source(candidate: Candidate) -> &'static str {
    match candidate {
        Candidate::A => {
            "var grounded=0\nvar stepn=0\nfn app_start(){}\nfn app_update(dt float){stepn=stepn+1;let q=ag_move_and_slide(0,0,8.0,0.0,0.0,0.002);if ag_motion_status(q)!=1 {ag_set_visible(127,0)}\nif ag_motion_node(q)==127 {stepn=127}}\n"
        }
        Candidate::B => {
            "var grounded=0\nvar stepn=0\nfn app_start(){}\nfn app_update(dt float){stepn=stepn+1;let q=agp_move(0,8.0,0.0,0.0,0.002);if ag_motion_status(q)!=1 {ag_set_visible(127,0)}}\n"
        }
    }
}

pub fn load_name(candidate: Candidate) -> &'static str {
    match candidate {
        Candidate::A => "research/game-runtime/load_a.at",
        Candidate::B => "research/game-runtime/load_b.at",
    }
}

// ------------------------------------------------- host presentation (V-04) --
/// Real Host/Play/GPU publication of the frozen routes. The collision
/// descriptor is derived from the document exactly as production Run does;
/// the caller asserts it equals the Session scene rows before presenting.
pub mod host_case {
    use super::*;
    use crate::authoring::{
        assets::Registry,
        document::{Document, Kind, Object},
        host::Host,
        interaction,
        play::{Action, State},
        spatial::Descriptor,
    };
    use agp::assets::ModelData;
    use std::path::Path;

    /// (center, size) per 2D box: descriptor offsets are +/-scale/2.
    fn g2_objects() -> Vec<([f32; 3], [f32; 3])> {
        vec![
            ([2.0, 1.0, 0.0], [0.8, 1.0, 1.0]),
            ([4.0, -0.5, 0.0], [8.0, 1.0, 1.0]),
            ([14.0, -0.5, 0.0], [8.0, 1.0, 1.0]),
            ([25.0, -0.5, 0.0], [10.0, 1.0, 1.0]),
            ([13.5, 1.125, 0.0], [3.0, 0.25, 1.0]),
            ([-1.0, 3.0, 0.0], [2.0, 14.0, 1.0]),
            ([31.0, 3.0, 0.0], [2.0, 14.0, 1.0]),
            ([15.0, 11.0, 0.0], [34.0, 2.0, 1.0]),
        ]
    }

    pub fn g2_document() -> Result<Document, String> {
        let mut d = Document::empty("game-runtime-g2".into());
        for (i, (translation, scale)) in g2_objects().into_iter().enumerate() {
            let mut o = Object::new(i as u64 + 1, Kind::Quad);
            o.translation = translation;
            o.scale = scale;
            d.objects.push(o);
        }
        d.validate()?;
        Ok(d)
    }

    /// G3 document: fixed cylinder primitives scaled/translated to realize the
    /// frozen collision boxes exactly (world = translation + local*scale, the
    /// same builtin ModelData production Host carries for Kind::Cylinder).
    pub fn g3_document_and_assets() -> Result<(Document, Registry), String> {
        let cyl = ModelData {
            source_sha256: "authoring-fixed-cylinder".into(),
            meshes: vec![agp::render3d::cylinder_mesh()],
        };
        let (blo, bhi) = interaction::bounds(&cyl).ok_or("cylinder has no bounds")?;
        let size = bhi - blo;
        if !(size.x > 0.0 && size.y > 0.0 && size.z > 0.0) {
            return Err("cylinder bounds degenerate".into());
        }
        let mut d = Document::empty("game-runtime-g3".into());
        let mut all = vec![([-0.4f32, -0.5, -0.4], [0.4f32, 0.5, 0.4])];
        for Row(_, _, b) in g3_rows().into_iter().skip(1) {
            all.push(([b[1], b[2], b[3]], [b[4], b[5], b[6]]));
        }
        for (i, (lo, hi)) in all.iter().enumerate() {
            let mut o = Object::new(i as u64 + 1, Kind::Cylinder);
            for a in 0..3 {
                o.scale[a] = (hi[a] - lo[a]) / size[a];
                o.translation[a] = lo[a] - blo[a] * o.scale[a];
            }
            // The character object carries the frozen spawn like the G2
            // document does; app_start re-places it before the first tick.
            if i == 0 {
                o.translation = [2.0, 1.0, 2.0];
            }
            d.objects.push(o);
        }
        d.validate()?;
        Ok((d, Registry::default()))
    }

    fn settle(h: &mut Host) -> Result<(), String> {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(31);
        while h.play.as_ref().ok_or("missing Play")?.pending() {
            if std::time::Instant::now() >= end {
                return Err("host initialization deadline 30s".into());
            }
            h.frame(None)?;
        }
        let p = h.play.as_ref().unwrap();
        if p.state != State::Paused {
            return Err(format!(
                "host rejected: state={:?} tick={} error={:?}",
                p.state,
                p.tick(),
                p.error
            ));
        }
        Ok(())
    }

    fn capture(h: &mut Host, route: Route, name: &str) -> Result<String, String> {
        let id = if route == Route::G2 { h.two } else { h.three };
        let bytes = h.rt.capture(id, name).ok_or("capture")?.0;
        Ok(bytes)
    }

    fn char_translation(h: &Host) -> Result<[f32; 3], String> {
        let p = h.play.as_ref().ok_or("missing Play")?;
        let f = p.last.as_ref().ok_or("no published frame")?;
        Ok(f.objects[0].translation)
    }

    /// One full host publication run: numeric checkpoints bound to the Session
    /// trace, movement diversity, stop-restore pixels and owner release.
    /// `mutate` applies a real source mutation; the named oracle must reject it.
    pub fn run(route: Route, mutate: Option<(&str, &str)>) -> Result<serde_json::Value, String> {
        let candidate = Candidate::A;
        let user = route_user(route, candidate);
        let user = match mutate {
            Some((from, to)) => {
                assert!(user.contains(from), "mutation anchor missing");
                user.replace(from, to)
            }
            None => user.to_string(),
        };
        // Expected trajectory always comes from the UNMUTATED frozen route:
        // a real source mutation must diverge from it (the oracle that makes
        // dropped integration output visible), while the green path matches
        // by construction.
        let rows = if route == Route::G2 {
            g2_rows()
        } else {
            g3_rows()
        };
        let frozen = route_user(route, candidate);
        let mut session = session_for(
            frozen,
            route_name(route, candidate),
            candidate,
            &rows,
            false,
        )?;
        let steps = if route == Route::G2 {
            G2_STEPS
        } else {
            G3_STEPS
        };
        let trace = run_steps(&mut session, steps, &[])?;
        drop(session);
        let checkpoints: Vec<usize> = match route {
            Route::G2 => vec![1, 80, 285, steps],
            Route::G3 => vec![1, 120, 309, steps],
        };
        let expected: Vec<[f32; 3]> = checkpoints
            .iter()
            .map(|s| {
                let t = &trace[s - 1];
                [t.x, t.y, t.z]
            })
            .collect();

        let (document, registry) = match route {
            Route::G2 => (g2_document()?, Registry::default()),
            Route::G3 => g3_document_and_assets()?,
        };
        let mut h = Host::with_project(document, registry, None)?;
        let mut want: Vec<[f32; 7]> = rows.iter().map(|Row(_, _, b)| *b).collect();
        let spawn0 = if route == Route::G2 {
            [2.0f32, 1.0, 0.0]
        } else {
            [2.0f32, 1.0, 2.0]
        };
        want[0][1] += spawn0[0];
        want[0][2] += spawn0[1];
        want[0][3] += spawn0[2];
        want[0][4] += spawn0[0];
        want[0][5] += spawn0[1];
        want[0][6] += spawn0[2];
        {
            let editor = &h.editor;
            let cyl = ModelData {
                source_sha256: "authoring-fixed-cylinder".into(),
                meshes: vec![agp::render3d::cylinder_mesh()],
            };
            let got = Descriptor::build(&editor.document, |_o| Some(&cyl))?;
            // The Session scene pins world rows at zero (boxes are absolute);
            // the document expresses the same world boxes as translation plus
            // relative descriptor offsets. Compare effective world boxes.
            let order = crate::authoring::hierarchy::ordered(&editor.document)?;
            let worlds = crate::authoring::hierarchy::derive(&editor.document)?;
            let mut effective = Vec::new();
            for (i, n) in order.iter().enumerate() {
                let row = got.rows()[i];
                let w = worlds[&n.id].translation;
                effective.push([
                    row[0],
                    row[1] + w[0],
                    row[2] + w[1],
                    row[3] + w[2],
                    row[4] + w[0],
                    row[5] + w[1],
                    row[6] + w[2],
                ]);
            }
            assert_eq!(effective, want, "descriptor world boxes != session scene");
        }

        h.frame(None)?;
        let editing = capture(&mut h, route, "game")?;
        let snapshot = crate::authoring::behavior::SourceSnapshot::from_bytes_for(
            Path::new(route_name(route, candidate)),
            user.into_bytes(),
            crate::authoring::behavior::Api::MotionV5,
        )?;
        h.start_user_snapshot(snapshot.clone())?;
        h.pause_play("");
        let vm_owner = h
            .play
            .as_ref()
            .unwrap()
            .owner_probe()
            .ok_or("no VM owner")?;
        let spatial_owner = h.play.as_ref().unwrap().spatial_owner_probe();
        settle(&mut h)?;
        let init_pixels = capture(&mut h, route, "init")?;
        let init_translation = char_translation(&h)?;

        let mut checkpoint_report = Vec::new();
        let mut capture_hashes = vec![init_pixels.clone()];
        let mut translations = vec![init_translation];
        let mut checkpoint_translations = Vec::new();
        for step in 1..=steps {
            h.play.as_mut().unwrap().dispatch(Action::default())?;
            settle(&mut h)?;
            if checkpoints.contains(&step) {
                let t = char_translation(&h)?;
                let pixels = capture(&mut h, route, "cp")?;
                capture_hashes.push(pixels);
                translations.push(t);
                checkpoint_translations.push(t);
                let idx = checkpoints.iter().position(|c| *c == step).unwrap();
                checkpoint_report.push(serde_json::json!({
                    "step": step,
                    "expected": expected[idx],
                    "published": t,
                    "matches_trace": true,
                }));
            }
        }
        // Numeric binding: published == Session trace at every checkpoint.
        for (i, t) in checkpoint_translations.iter().enumerate() {
            let e = &expected[i];
            let close = t.iter().zip(e.iter()).all(|(p, x)| (p - x).abs() < 1e-4);
            if !close {
                return Err(format!(
                    "numeric oracle: published frame diverges from the Session trace at checkpoint {i}: published={t:?} expected={e:?}"
                ));
            }
        }
        // Movement diversity: the published character must actually travel.
        if !checkpoint_translations
            .iter()
            .any(|t| *t != checkpoint_translations[0])
        {
            return Err("movement oracle: published frames never left the spawn".into());
        }

        let stop_start = std::time::Instant::now();
        h.stop_play();
        let stop_ns = stop_start.elapsed().as_nanos() as u64;
        h.frame(None)?;
        let released = !vm_owner() && spatial_owner.as_ref().is_none_or(|w| w.upgrade().is_none());
        if !released {
            return Err("owner oracle: VM/spatial owner alive after Stop".into());
        }
        let restored = capture(&mut h, route, "game")?;
        if restored != editing {
            return Err("content oracle: Stop did not restore editing pixels".into());
        }

        // 20 run/stop cycles: real owner release every cycle (HT-5). Cycles
        // only exercise the owner lifecycle, so they tolerate either pause
        // state while initialization drains.
        for cycle in 0..20 {
            h.start_user_snapshot(snapshot.clone())?;
            let vm = h
                .play
                .as_ref()
                .unwrap()
                .owner_probe()
                .ok_or("no VM owner")?;
            let spatial = h.play.as_ref().unwrap().spatial_owner_probe();
            let end = std::time::Instant::now() + std::time::Duration::from_secs(31);
            while h.play.as_ref().unwrap().pending() {
                if std::time::Instant::now() >= end {
                    return Err("owner cycle initialization deadline".into());
                }
                h.frame(None)?;
            }
            let live = vm() && spatial.as_ref().is_none_or(|w| w.upgrade().is_some());
            h.stop_play();
            h.frame(None)?;
            let gone = !vm() && spatial.as_ref().is_none_or(|w| w.upgrade().is_none());
            if !live || !gone {
                return Err(format!("owner cycle {cycle}: live={live} released={gone}"));
            }
        }
        // Fresh upload negative: a new Run starts from the frozen spawn.
        h.start_user_snapshot(snapshot.clone())?;
        h.pause_play("");
        settle(&mut h)?;
        let fresh = char_translation(&h)?;
        h.stop_play();
        h.frame(None)?;
        let spawn_world = if route == Route::G2 {
            [2.0f32, 1.0, 0.0]
        } else {
            [2.0f32, 1.0, 2.0]
        };
        if fresh != spawn_world {
            return Err(format!(
                "fresh upload: new Run inherited state {fresh:?} != {spawn_world:?}"
            ));
        }

        Ok(serde_json::json!({
            "schema": "autographics.game-host/1",
            "route": format!("{route:?}").to_lowercase(),
            "checkpoints": checkpoint_report,
            "pixel_hashes": capture_hashes,
            "stop_ns": stop_ns,
            "restored_editing_pixels": true,
            "owner_cycles": 20,
            "fresh_spawn": fresh,
            "completed": true,
        }))
    }
}
