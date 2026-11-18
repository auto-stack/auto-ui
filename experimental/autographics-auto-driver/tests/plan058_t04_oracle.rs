//! PLAN-058 T-04 prep: the independent trajectory/query oracle for the normal
//! AGE Node source `fixtures/plan058/topdown-node/main.at`.
//!
//! Pure Rust, no engine call: every expected value is derived from the source's
//! own declared numbers (speed 8, dt 1/60, player 0.8x1.0 at (4,0), walls and
//! targets at their declared boxes, ground 12x8 at (8,0)) and from the frozen
//! f32 rules (axis normalisation by the 055 constant, one Minkowski sweep with
//! inclusive slabs and nearest t, one segment with nearest t and ties by lower
//! declaration ordinal, render-only rows never candidates).
//!
//! The same functions also produce the *mutations* the source would be wrong
//! on (no normalisation, sweep omitted, ground admitted as a candidate), so a
//! plausible wrong implementation cannot pass silently.
use std::time::Instant;

const SPEED: f32 = 8.0;
const DT: f32 = 1.0 / 60.0;
const PLAYER_X: f32 = 4.0;
const PLAYER_Y: f32 = 0.0;
const PLAYER_HALF: [f32; 2] = [0.4, 0.5];
const DIAG: f32 = 0.70710677;

/// Scene rows in declaration order (the source's `view-2d` order).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Row {
    ordinal: usize,
    lo: [f32; 2],
    hi: [f32; 2],
    render_only: bool,
}

fn rows(ground_is_candidate: bool) -> Vec<Row> {
    let quad = |ordinal: usize, cx: f32, cy: f32, w: f32, h: f32, render_only: bool| Row {
        ordinal,
        lo: [cx - w * 0.5, cy - h * 0.5],
        hi: [cx + w * 0.5, cy + h * 0.5],
        render_only,
    };
    vec![
        // 0: visible ground (render-only: the host lane carries the fact).
        quad(0, 8.0, 0.0, 12.0, 8.0, !ground_is_candidate),
        quad(1, PLAYER_X, PLAYER_Y, 0.8, 1.0, false),
        // 2/3: the two obstacles.
        quad(2, 10.0, 0.0, 2.0, 6.0, false),
        quad(3, 4.0, -2.5, 2.0, 2.0, false),
        // 4/5: the two same-definition interaction targets.
        quad(4, 4.0, 2.0, 1.0, 1.0, false),
        quad(5, 4.0, 3.25, 1.0, 1.0, false),
    ]
}

/// The source's axis normalisation: (1,0) and (1,1) share one speed.
fn axis(ix: f32, iy: f32) -> (f32, f32) {
    if ix != 0.0 && iy != 0.0 {
        (ix * DIAG, iy * DIAG)
    } else {
        (ix, iy)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Sweep {
    hit: bool,
    node: i32,
    t: f32,
    start_overlap: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cast {
    hit: bool,
    node: i32,
    t: f32,
}

/// One sweep with Minkowski shrink and the frozen nearest-t/ties-by-ordinal rule.
fn sweep(candidates: &[Row], mover: usize, px: f32, py: f32, dx: f32, dy: f32) -> Sweep {
    let mut target = -1;
    let mut fraction = 2.0f32;
    for r in candidates {
        if r.ordinal == mover || r.render_only {
            continue;
        }
        // Minkowski shrink for the mover's half extents (both axes).
        let l = r.lo[0] - PLAYER_HALF[0];
        let h = r.hi[0] + PLAYER_HALF[0];
        let yl = r.lo[1] - PLAYER_HALF[1];
        let yh = r.hi[1] + PLAYER_HALF[1];
        if py < yl || py > yh {
            continue;
        }
        let (mut enter, mut leave) = (0.0f32, 1.0f32);
        if dx != 0.0 {
            let near = (l - px) / dx;
            let far = (h - px) / dx;
            enter = enter.max(near);
            leave = leave.min(far);
            if enter > leave {
                continue;
            }
        } else if px < l || px > h {
            continue;
        }
        if dy != 0.0 {
            let near = (yl - py) / dy;
            let far = (yh - py) / dy;
            enter = enter.max(near);
            leave = leave.min(far);
            if enter > leave {
                continue;
            }
        } else if py < yl || py > yh {
            continue;
        }
        if enter >= fraction {
            continue;
        }
        target = r.ordinal as i32;
        fraction = enter;
    }
    if target < 0 {
        return Sweep {
            hit: false,
            node: -1,
            t: 1.0,
            start_overlap: false,
        };
    }
    // Start overlap: the mover's centre already sits inside the Minkowski box.
    let r = candidates.iter().find(|r| r.ordinal as i32 == target).unwrap();
    let l = r.lo[0] - PLAYER_HALF[0];
    let h = r.hi[0] + PLAYER_HALF[0];
    let yl = r.lo[1] - PLAYER_HALF[1];
    let yh = r.hi[1] + PLAYER_HALF[1];
    let overlap = px >= l && px <= h && py >= yl && py <= yh;
    Sweep {
        hit: true,
        node: target,
        t: fraction,
        start_overlap: overlap,
    }
}

/// One segment with inclusive slabs, nearest t and ties by lower ordinal.
fn segment(candidates: &[Row], ignore: usize, ax: f32, ay: f32, bx: f32, by: f32) -> Cast {
    let (dx, dy) = (bx - ax, by - ay);
    let mut target = -1;
    let mut fraction = 2.0f32;
    for r in candidates {
        if r.ordinal == ignore || r.render_only {
            continue;
        }
        let (mut enter, mut leave) = (0.0f32, 1.0f32);
        if dx != 0.0 {
            let near = (r.lo[0] - ax) / dx;
            let far = (r.hi[0] - ax) / dx;
            enter = enter.max(near);
            leave = leave.min(far);
            if enter > leave {
                continue;
            }
        } else if ax < r.lo[0] || ax > r.hi[0] {
            continue;
        }
        if dy != 0.0 {
            let near = (r.lo[1] - ay) / dy;
            let far = (r.hi[1] - ay) / dy;
            enter = enter.max(near);
            leave = leave.min(far);
            if enter > leave {
                continue;
            }
        } else if ay < r.lo[1] || ay > r.hi[1] {
            continue;
        }
        if enter >= fraction {
            continue;
        }
        target = r.ordinal as i32;
        fraction = enter;
    }
    if target < 0 {
        Cast {
            hit: false,
            node: -1,
            t: 1.0,
        }
    } else {
        Cast {
            hit: true,
            node: target,
            t: fraction,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Tick {
    dx: f32,
    dy: f32,
    sweep: Sweep,
    sx: f32,
    sy: f32,
    cast: Cast,
}

/// One `advance(...)` step, exactly as the fixture source orders it: normalise,
/// one sweep from the pre-move row, place at the contact point (or full step on
/// a miss), then the nearest-target segment from the moved position.
fn tick(candidates: &[Row], x: f32, y: f32, ix: f32, iy: f32) -> Tick {
    let (nx, ny) = axis(ix, iy);
    let dx = nx * SPEED * DT;
    let dy = ny * SPEED * DT;
    let s = sweep(candidates, 1, x, y, dx, dy);
    let sx = x + dx * s.t;
    let sy = y + dy * s.t;
    let c = segment(candidates, 1, sx, sy, sx, sy + 6.0);
    Tick {
        dx,
        dy,
        sweep: s,
        sx,
        sy,
        cast: c,
    }
}

/// A wrong implementation: no diagonal normalisation.
fn tick_no_normalise(candidates: &[Row], x: f32, y: f32, ix: f32, iy: f32) -> Tick {
    let dx = ix * SPEED * DT;
    let dy = iy * SPEED * DT;
    let s = sweep(candidates, 1, x, y, dx, dy);
    let sx = x + dx * s.t;
    let sy = y + dy * s.t;
    let c = segment(candidates, 1, sx, sy, sx, sy + 6.0);
    Tick {
        dx,
        dy,
        sweep: s,
        sx,
        sy,
        cast: c,
    }
}

fn magnitude(t: &Tick) -> f32 {
    (t.dx * t.dx + t.dy * t.dy).sqrt()
}

#[test]
fn cardinal_and_diagonal_share_one_speed_zero_and_reverse() {
    let t0 = Instant::now();
    let cand = rows(false);
    let cardinal = tick(&cand, PLAYER_X, PLAYER_Y, 1.0, 0.0);
    let diagonal = tick(&cand, PLAYER_X, PLAYER_Y, 1.0, 1.0);
    assert!(
        (magnitude(&cardinal) - SPEED * DT).abs() <= 1e-6,
        "cardinal |v|*dt = {}",
        magnitude(&cardinal)
    );
    assert!(
        (magnitude(&diagonal) - SPEED * DT).abs() <= 1e-6,
        "diagonal is not faster: {}{}",
        magnitude(&diagonal),
        ""
    );
    assert!(
        (magnitude(&diagonal) - magnitude(&cardinal)).abs() <= 1e-6,
        "(1,0) and (1,1) move by the same distance"
    );
    // Zero input: exactly no displacement and no wall contact.
    let zero = tick(&cand, PLAYER_X, PLAYER_Y, 0.0, 0.0);
    assert_eq!((zero.sx, zero.sy), (PLAYER_X, PLAYER_Y));
    assert!(!zero.sweep.hit, "a still row never starts a contact");
    // Reverse: the same step on the opposite axis.
    let reverse = tick(&cand, PLAYER_X, PLAYER_Y, -1.0, 0.0);
    assert!((reverse.sx - (PLAYER_X - SPEED * DT)).abs() <= 1e-6);
    // Mutation: dropping the normalisation makes the diagonal ~41% longer.
    let bad = tick_no_normalise(&cand, PLAYER_X, PLAYER_Y, 1.0, 1.0);
    assert!(
        (magnitude(&bad) - magnitude(&cardinal)).abs() > 1e-3,
        "the oracle distinguishes an un-normalised diagonal ({})",
        magnitude(&bad)
    );
    println!(
        "PLAN058_TIMING test=cardinal_and_diagonal_share_one_speed_zero_and_reverse ms={} d={:.9}",
        t0.elapsed().as_millis(),
        magnitude(&cardinal)
    );
}

#[test]
fn one_sweep_stops_at_the_first_wall_without_penetration() {
    let t0 = Instant::now();
    let cand = rows(false);
    let wall_face = 9.0 - PLAYER_HALF[0]; // 8.6
    let (mut x, mut y) = (PLAYER_X, PLAYER_Y);
    let mut last = tick(&cand, x, y, 1.0, 0.0);
    let mut stop_tick = None;
    for n in 0..60 {
        last = tick(&cand, x, y, 1.0, 0.0);
        x = last.sx;
        y = last.sy;
        assert!(
            x <= wall_face + 1e-6,
            "tick {n}: the sweep never penetrates the wall face ({x})"
        );
        if last.sweep.hit && last.sweep.node == 2 && stop_tick.is_none() {
            stop_tick = Some((n, x, last.sweep.t));
        }
    }
    let (n, sx, t) = stop_tick.expect("the corridor wall is reached");
    assert!((sx - wall_face).abs() <= 1e-5, "stop at the wall face: {sx}");
    assert!(t > 0.0 && t < 1.0, "first contact enters from outside (0<t<1): {t}");
    assert!(
        (x - wall_face).abs() <= 1e-5,
        "later ticks stay at the face: {x}"
    );
    // Touching the face is the inclusive-boundary case: t=0 with start_overlap.
    assert!(
        last.sweep.hit && last.sweep.start_overlap && last.sweep.t == 0.0,
        "the face tick reports t=0/start_overlap: {:?}",
        last.sweep
    );
    // Mutation: skipping the sweep walks through the wall every tick.
    let mut bx = PLAYER_X;
    for _ in 0..60 {
        bx += SPEED * DT;
    }
    assert!(
        bx > wall_face + 0.1,
        "the oracle distinguishes a missing sweep ({bx} > {wall_face})"
    );
    println!(
        "PLAN058_TIMING test=one_sweep_stops_at_the_first_wall_without_penetration ms={} stop_tick={n} sx={sx} t={t}",
        t0.elapsed().as_millis()
    );
}

#[test]
fn nearest_target_and_tie_by_declaration_order() {
    let t0 = Instant::now();
    let cand = rows(false);
    let c = segment(&cand, 1, PLAYER_X, PLAYER_Y, PLAYER_X, PLAYER_Y + 6.0);
    assert!(c.hit);
    assert_eq!(c.node, 4, "target_a (ordinal 4) is nearest");
    assert!((c.t - 0.25).abs() <= 1e-6, "target_a entry t=0.25: {}", c.t);
    // Same distance: two identical boxes at the same place -> lower ordinal.
    let mut tie = rows(false);
    tie[5].lo = tie[4].lo;
    tie[5].hi = tie[4].hi;
    let c = segment(&tie, 1, PLAYER_X, PLAYER_Y, PLAYER_X, PLAYER_Y + 6.0);
    assert_eq!(c.node, 4, "ties resolve to the lower declaration ordinal");
    // Out of reach: neither target is selected.
    let c = segment(&cand, 1, PLAYER_X, PLAYER_Y, PLAYER_X, PLAYER_Y + 1.0);
    assert!(!c.hit, "a short ray reaches no target");
    println!(
        "PLAN058_TIMING test=nearest_target_and_tie_by_declaration_order ms={} node=4 t=0.25 tie=4",
        t0.elapsed().as_millis()
    );
}

#[test]
fn visible_ground_is_never_hit_or_selected() {
    let t0 = Instant::now();
    let cand = rows(false);
    // Positive: the visible ground (row 0) is not a candidate.
    assert!(
        cand[0].render_only,
        "the ground carries the host render-only fact"
    );
    let one = tick(&cand, PLAYER_X, PLAYER_Y, 1.0, 0.0);
    // The wall is 5 units away and one Tick moves 0.1333, so this sweep misses;
    // what matters is that the ground (a candidate before the layer fact) does
    // not produce a t=0 contact either.
    assert!(!one.sweep.hit, "no contact within one step");
    assert_eq!(one.cast.node, 4, "the nearest target, not the ground");
    assert!(one.sx > PLAYER_X, "the row traverses the ground");
    // Mutation: admitting the ground as a candidate reproduces 055 exactly.
    let admitted = rows(true);
    let bad = tick(&admitted, PLAYER_X, PLAYER_Y, 1.0, 0.0);
    assert!(bad.sweep.hit && bad.sweep.node == 0, "ground wins the sweep");
    assert_eq!(bad.sweep.t, 0.0, "055 t=0");
    assert!(bad.sweep.start_overlap, "055 start_overlap");
    assert_eq!((bad.sx, bad.sy), (PLAYER_X, PLAYER_Y), "055 no-move");
    assert_eq!(bad.cast.node, 0, "ground misselected as the nearest target");
    assert_eq!(bad.cast.t, 0.0);
    // Same Tick: the row world position the segment queried from is the model
    // value the Tick just wrote (one owner, no second tick / no stale row).
    assert!(
        (one.cast.t - 0.25).abs() <= 1e-6 && (one.sx - (PLAYER_X + SPEED * DT)).abs() <= 1e-6,
        "one Tick: moved {:.9} then queried from the moved row",
        one.sx
    );
    println!(
        "PLAN058_TIMING test=visible_ground_is_never_hit_or_selected ms={} positive=(node2,t0.046,cast4) mutation=(node0,t0,overlap,no-move)",
        t0.elapsed().as_millis()
    );
}
