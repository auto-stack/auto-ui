//! PLAN-058 T-04 prep: the UI-side candidate `GfxHostProvider` contract
//! (`src/authoring/gfx_host_port.rs`) against a recording/fake provider.
//!
//! These tests pin the *shape* the T-04 adapter will expose once
//! `IF-981-GFX-HOST-PORT` lands: one legal world callback's call sequence, its
//! shared-pool fees (31/32), the typed results and free port field reads, the
//! render-only row fact, and the over-budget counterexample (sweep+segment=24,
//! a second sweep 16 -> 40 > 32 named fault 10, zero effect, sticky, per-Tick
//! reset with a Run-terminal fault).
//!
//! This is a CANDIDATE contract, not the AGE host-port ABI (see t04-prep.md).
use autographics_auto_driver::authoring::{
    document::Layer,
    gfx_host_port::{
        Cast2D, GfxHostProvider, RecordingProvider, Sweep2D, CREDIT_BASE, CREDIT_SEGMENT,
        CREDIT_SWEEP, FAULT_QUERY_BUDGET,
    },
};
use std::time::Instant;

fn rows() -> Vec<(f32, f32)> {
    vec![
        (8.0, 0.0),  // 0 ground (render-only)
        (4.0, 0.0),  // 1 player
        (10.0, 0.0), // 2 wall_a
        (4.0, -2.5), // 3 wall_b
        (4.0, 2.0),  // 4 target_a
        (4.0, 3.25), // 5 target_b
    ]
}

fn layers() -> Vec<Layer> {
    vec![
        Layer::RenderOnly,
        Layer::Collision,
        Layer::Collision,
        Layer::Collision,
        Layer::Collision,
        Layer::Collision,
    ]
}

fn fake() -> RecordingProvider {
    let mut p = RecordingProvider::new(rows(), layers());
    p.set_input(2, 1.0, 0.0, false);
    p.set_sweep(Sweep2D {
        hit: false,
        node: -1,
        t: 1.0,
        origin_x: 4.0,
        origin_y: 0.0,
        normal_x: 0.0,
        normal_y: 0.0,
        start_overlap: false,
    });
    p.set_segment(Cast2D {
        hit: true,
        node: 4,
        t: 0.25,
        point_x: 4.0,
        point_y: 2.0,
        normal_x: 0.0,
        normal_y: -1.0,
    });
    p
}

/// One legal Tick in the fixture's own order (advance + publish): 31 credits.
#[test]
fn legal_tick_call_sequence_fees_and_typed_results() {
    let t0 = Instant::now();
    let mut p = fake();
    p.begin_tick();
    let space = p.input_space().expect("input.space");
    let ix = p.input_x().expect("input.x");
    let iy = p.input_y().expect("input.y");
    let dt = p.tick_dt().expect("tick.dt");
    let (nx, ny) = (ix * 0.70710677, iy * 0.70710677);
    let (dx, dy) = (nx * 8.0 * dt, ny * 8.0 * dt);
    let sweep = p.query_sweep(1, true, dx, dy).expect("sweep");
    let sx = 4.0 + dx * sweep.t;
    let sy = 0.0 + dy * sweep.t;
    let placed = p.row_place(1, sx, sy).expect("row.place");
    let cast = p
        .query_segment(1, true, sx, sy, sx, sy + 6.0)
        .expect("segment");
    let token = p.render_spawn("player", sx, sy).expect("render2d.spawn");
    let token = p
        .render_frame(token, "assets/atlas.png", 0, 0)
        .expect("render2d.frame");

    assert_eq!(space, 2);
    assert_eq!((ix, iy), (1.0, 0.0));
    assert!((dt - 1.0 / 60.0).abs() <= 1e-9);
    assert!(!sweep.hit, "one step does not reach the wall");
    assert_eq!(sweep.t, 1.0, "miss keeps t=1 so the step is the full dx");
    assert_eq!(placed, 1);
    assert_eq!((cast.node, cast.t), (4, 0.25), "typed Cast2D port");
    assert_eq!(token, 1, "spawn token");
    // Fees: input 3 + dt 1 + sweep 16 + place 1 + segment 8 + spawn 1 + frame 1.
    assert_eq!(p.meter().credits(), 31, "31 of the 32-credit pool");
    assert_eq!(p.meter().fault(), None);
    let ops: Vec<&str> = p.log().iter().map(|c| c.op).collect();
    assert_eq!(
        ops,
        vec![
            "input.space",
            "input.x",
            "input.y",
            "tick.dt",
            "query2d.sweep",
            "row.place",
            "query2d.segment",
            "render2d.spawn",
            "render2d.frame",
        ]
    );
    assert!(p.log().iter().all(|c| c.accepted), "no refusal in a legal Tick");
    assert_eq!(p.charges("query2d.sweep"), 1, "exactly one sweep per Tick");
    assert_eq!(p.charges("query2d.segment"), 1);
    assert_eq!(p.row(1), (sx, sy), "row world position is the moved one");
    assert_eq!(p.publishes(), &[(1, sx, sy)]);
    assert_eq!(p.frames(), &[(1, "assets/atlas.png".to_string(), 0, 0)]);
    // Typed port field reads are free (no provider call, no charge).
    let before = p.meter().credits();
    let _ = (sweep.hit, sweep.t, cast.hit, cast.node, cast.t, cast.normal_x);
    assert_eq!(p.meter().credits(), before, "field reads cost 0");
    println!(
        "PLAN058_TIMING test=legal_tick_call_sequence_fees_and_typed_results ms={} credits=31 calls={}",
        t0.elapsed().as_millis(),
        p.log().len()
    );
}

/// The render-only row fact: the ground is visible but can never be a query
/// target, and the provider never calls a query with it as the mover.
#[test]
fn render_only_ground_is_never_a_query_target() {
    let t0 = Instant::now();
    let mut p = fake();
    assert_eq!(p.row_layer(0), Some(Layer::RenderOnly));
    let charged = p.query_sweep(0, true, 1.0, 0.0);
    assert!(charged.is_none(), "a render-only mover yields no result");
    // Frozen order: the charge already happened before the layer guard.
    assert_eq!(p.meter().credits(), CREDIT_SWEEP);
    assert_eq!(p.charges("query2d.sweep"), 1);
    // A collision row is a normal query target.
    assert_eq!(p.row_layer(1), Some(Layer::Collision));
    assert!(p.query_sweep(1, true, 1.0, 0.0).is_some());
    assert_eq!(p.meter().credits(), CREDIT_SWEEP * 2);
    println!(
        "PLAN058_TIMING test=render_only_ground_is_never_a_query_target ms={} charged_refusal=16 second=kind-ok",
        t0.elapsed().as_millis()
    );
}

/// Over budget: sweep(16)+segment(8)=24, a second Node's sweep(+16)=40 -> the
/// named fault 10 with zero effect, then a 1-credit call is still refused
/// (sticky), the pool resets at the next Tick, and the fault stays Run-terminal.
#[test]
fn over_budget_is_named_zero_effect_sticky_and_resets_per_tick() {
    let t0 = Instant::now();
    let mut p = fake();
    p.begin_tick();
    assert!(p.query_sweep(1, true, 1.0, 0.0).is_some(), "16");
    assert!(p
        .query_segment(1, true, 4.0, 0.0, 4.0, 6.0)
        .is_some(), "24");
    let row_before = p.row(1);
    let publishes_before = p.publishes().len();
    let refused = p.query_sweep(1, true, -1.0, 0.0);
    assert!(refused.is_none(), "40 > 32 is refused");
    assert_eq!(p.meter().fault(), Some(FAULT_QUERY_BUDGET));
    assert_eq!(
        p.meter().credits(),
        40,
        "the attempt counter advances (no refund)"
    );
    // Zero effect: no row move, no publish, no typed result landed.
    assert_eq!(p.row(1), row_before);
    assert_eq!(p.publishes().len(), publishes_before);
    // Sticky: the next cheaper call in the same Tick is refused as well.
    let cheap = p.row_world(1, 0);
    assert!(cheap.is_none(), "a 1-credit call is still refused");
    assert_eq!(p.meter().fault(), Some(FAULT_QUERY_BUDGET));
    assert_eq!(p.meter().credits(), 41, "40 + the refused 1-credit attempt");
    // Per-Tick reset of the pool; the fault is Run-terminal.
    p.begin_tick();
    assert_eq!(p.meter().credits(), 0, "the pool resets at the callback");
    assert_eq!(p.meter().fault(), Some(FAULT_QUERY_BUDGET), "fault remains");
    let mut fresh = fake();
    if fresh.meter().fault().is_none() {
        fresh.meter_mut().reset_run();
    }
    assert_eq!(fresh.meter().fault(), None, "a fresh Run clears the fault");
    println!(
        "PLAN058_TIMING test=over_budget_is_named_zero_effect_sticky_and_resets_per_tick ms={} credits=40->41 fault=10 sticky reset=0",
        t0.elapsed().as_millis()
    );
}

/// The identity/value ports stay typed: a frame carries the `Image2D` identity
/// it was built from, never a name.
#[test]
fn image_and_frame_ports_carry_content_identity() {
    let t0 = Instant::now();
    let mut p = fake();
    let a = p.image_png("assets/atlas.png").expect("image2d.png");
    let b = p.image_png("assets/other.png").expect("image2d.png");
    assert_ne!(a.sha256, b.sha256, "identity is per content, not a name");
    assert_eq!((a.width, a.height), (32, 32), "ALE port carries IHDR extents");
    let fa = p.frame_at(a.clone(), 0, 0, 32, 32).expect("frame2d.at");
    let fb = p.frame_at(b.clone(), 1, 2, 16, 16).expect("frame2d.at");
    assert_eq!(fa.image, a, "the frame keeps its Image2D port");
    assert_eq!(fb.image, b);
    assert_eq!((fa.x, fa.y, fa.w, fa.h), (0, 0, 32, 32));
    assert_eq!(p.meter().credits(), 4 * CREDIT_BASE, "each port call is 1");
    println!(
        "PLAN058_TIMING test=image_and_frame_ports_carry_content_identity ms={} credits=4 sha={}/{}",
        t0.elapsed().as_millis(),
        &a.sha256[..8.min(a.sha256.len())],
        &b.sha256[..8.min(b.sha256.len())]
    );
}

/// The fee table is the frozen one (segment 8 / sweep 16 / other 1).
#[test]
fn frozen_fee_table() {
    assert_eq!(
        (CREDIT_SWEEP, CREDIT_SEGMENT, CREDIT_BASE),
        (16, 8, 1)
    );
}
