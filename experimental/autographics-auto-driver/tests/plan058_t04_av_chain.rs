//! PLAN-058 T-04: the real AV chain — the normal AGE Node fixture compiled by
//! the refreshed `.sdk-source` front door and executed Tick by Tick through the
//! 981 `AgeGfxSession` (typed dispatch, shared 32-credit pool) with
//! `NeutralGfxHost` as the registered provider.
//!
//! Rows are **source-bound**: they are derived from the compiled description's
//! `scene` rows (declaration order = ordinal), not from a test-side table. The
//! only host-supplied fact is the render-only ground layer (G1: AGE has no
//! render-only declaration yet), which the host marks by row name.
//!
//! Every Tick's position/query results are compared against the independent
//! f32 oracle (`plan058_t04_oracle.rs` rules, re-implemented here as the same
//! recurrence) and the session's per-Tick credit use is asserted.
use ace_av::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn, AgeGfxSession};
use ace_av_core::contract::Value;
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AgePackageCompilation};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use autographics_auto_driver::authoring::{
    document::Layer,
    gfx_host_arms::{HostRow, NeutralGfxHost},
};
use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Instant};

const SOURCE: &str = include_str!("../fixtures/plan058/topdown-node/main.at");
const DT: f32 = 1.0 / 60.0;
const SPEED: f32 = 8.0;
const PLAYER_ORDINAL: usize = 1;
const PLAYER_HALF: [f32; 2] = [0.4, 0.5];
const WALL_FACE: f32 = 9.0 - PLAYER_HALF[0]; // 8.6

fn compile(src: &str) -> AgePackageCompilation {
    let input = PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    };
    compile_age_package(
        &[input],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .unwrap_or_else(|ds| {
        panic!(
            "front door rejected the T-04 source: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })
}

fn assets_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../autographics-node2d-pilot/fixtures/sprites2d/assets")
}

/// Rows from the compiled scene declaration: ordinal = declaration order, world
/// and half extents from the author's `position`/`size`, visibility from
/// `visible`. The render-only ground layer is the host fact (G1).
fn rows_from_scene(compiled: &AgePackageCompilation) -> Vec<HostRow> {
    // The scene rows live on the (single) AGE node's `view-2d`; declaration
    // order is the row ordinal the host uses.
    assert_eq!(compiled.description.components.len(), 1, "one AGE node");
    let scene = &compiled.description.components[0].scene;
    assert_eq!(scene.len(), 6, "six scene rows in the fixture");
    scene
        .iter()
        .enumerate()
        .map(|(ordinal, row)| {
            let floats = |key: &str| -> Option<Vec<f32>> {
                row.cpu.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
                    if let ace_core::source::age_package::AgeSceneValue::Floats(f) = v {
                        Some(f.clone())
                    } else {
                        None
                    }
                })
            };
            let position = floats("position").unwrap_or_else(|| panic!("{row:?} position"));
            let size = floats("size").unwrap_or_else(|| panic!("{row:?} size"));
            assert_eq!(position.len(), 2);
            assert_eq!(size.len(), 2);
            let visible = row
                .cpu
                .iter()
                .find(|(k, _)| k == "visible")
                .map(|(_, v)| matches!(v, ace_core::source::age_package::AgeSceneValue::Bool(true)))
                .unwrap_or(true);
            HostRow {
                ordinal,
                world: [position[0], position[1]],
                half: [size[0] * 0.5, size[1] * 0.5],
                layer: if row.name == "ground" {
                    Layer::RenderOnly
                } else {
                    Layer::Collision
                },
                visible,
            }
        })
        .collect()
}

/// Shared provider handle: `AgeGfxSession` owns the box, the test keeps an Rc.
struct SharedHost(Rc<RefCell<NeutralGfxHost>>);

impl AgeGfxHost for SharedHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
        self.0.borrow_mut().dispatch(symbol, args)
    }
}

fn spawn_session(compiled: &AgePackageCompilation) -> (AgeGfxSession, Rc<RefCell<NeutralGfxHost>>) {
    let host = Rc::new(RefCell::new(NeutralGfxHost::new(
        rows_from_scene(compiled),
        assets_root(),
    )));
    let session = AgeGfxSession::new(Box::new(SharedHost(host.clone())));
    (session, host)
}

fn tick_symbol(compiled: &AgePackageCompilation, name: &str) -> String {
    let symbol = compiled
        .symbol_of(name)
        .unwrap_or_else(|| panic!("no symbol for `{name}`"));
    format!("d_{symbol}")
}

fn row_world(host: &Rc<RefCell<NeutralGfxHost>>, ordinal: usize) -> [f32; 2] {
    host.borrow().row(ordinal).expect("row").world
}

#[test]
fn av_tick_chain_matches_the_oracle_and_stops_at_the_wall() {
    let t0 = Instant::now();
    let compiled = compile(SOURCE);
    let def = tick_symbol(&compiled, "tick_step");
    let (mut session, host) = spawn_session(&compiled);

    // Oracle state: one owner, the same initial row.
    let mut oracle = row_world(&host, PLAYER_ORDINAL);
    let mut first_contact: Option<(i32, f32, f32)> = None;
    for tick in 1..=40 {
        host.borrow_mut().set_input(2, 1.0, 0.0, false);
        host.borrow_mut().set_dt(DT);
        host.borrow_mut().clear_tick_observations();
        session.begin_tick();
        let value = session
            .run_def(&compiled.checked, &def)
            .unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        // Exactly 32 credits: input.x/y (2) + tick.dt (1) + row.world x2 (2) +
        // sweep (16) + row.place (1) + segment (8) + spawn (1) + frame (1).
        assert_eq!(
            session.credits_used(),
            32,
            "tick {tick} charges the whole 32-credit pool"
        );
        assert!(!session.tick_faulted(), "tick {tick} is inside the budget");

        // Independent oracle for this Tick (same recurrence as the arms).
        let (px, py) = (oracle[0], oracle[1]);
        let dx = 1.0 * SPEED * DT;
        let dy = 0.0;
        let (hit, node, t, overlap) = oracle_sweep(px, py, dx, dy);
        let (sx, sy) = if hit {
            (px + dx * t, py + dy * t)
        } else {
            (px + dx, py + dy)
        };
        let (cast_hit, cast_node, cast_t) = oracle_segment(sx, sy);
        oracle = [sx, sy];

        let world = row_world(&host, PLAYER_ORDINAL);
        assert!(
            (world[0] - sx).abs() <= 1e-5 && (world[1] - sy).abs() <= 1e-5,
            "tick {tick}: row world {world:?} vs oracle ({sx}, {sy})"
        );
        let sweep = host.borrow().last_sweep().expect("sweep ran");
        assert_eq!(sweep.node, node, "tick {tick}: sweep node");
        assert!(
            (sweep.t - t).abs() <= 1e-5,
            "tick {tick}: sweep t {} vs oracle {t}",
            sweep.t
        );
        assert_eq!(sweep.start_overlap, overlap, "tick {tick}: start overlap");
        let cast = host.borrow().last_segment().expect("segment ran");
        assert_eq!(cast.hit, cast_hit, "tick {tick}: segment hit");
        assert_eq!(cast.node, cast_node, "tick {tick}: segment node");
        assert!(
            (cast.t - cast_t).abs() <= 1e-5,
            "tick {tick}: segment t {}",
            cast.t
        );
        // The def's observable result is the segment node.
        assert_eq!(value, Value::I32(cast_node), "tick {tick}: def result");
        // Same-Tick presentation: one spawn (same leaf reuses its token) and one
        // frame publish at the moved position.
        let publishes = host.borrow().publishes().to_vec();
        assert_eq!(publishes.len(), 1, "tick {tick}: one spawn publish");
        assert_eq!(publishes[0].leaf, "player");
        assert!((publishes[0].x - sx).abs() <= 1e-5 && (publishes[0].y - sy).abs() <= 1e-5);
        assert_eq!(host.borrow().frames().len(), 1, "tick {tick}: one frame");
        if first_contact.is_none() && hit {
            first_contact = Some((tick, sx, t));
        }
    }
    let (tick, sx, t) = first_contact.expect("the corridor wall is reached");
    assert!(
        (sx - WALL_FACE).abs() <= 1e-5,
        "first contact stops at the wall face: {sx}"
    );
    assert!(t > 0.0 && t <= 1.0, "first contact t in (0,1]: {t}");
    let stopped = row_world(&host, PLAYER_ORDINAL);
    assert!(
        (stopped[0] - WALL_FACE).abs() <= 1e-5,
        "the row never penetrates: {stopped:?}"
    );
    assert_eq!(host.borrow().tokens().len(), 1, "one reused render token");
    assert!(host.borrow().tokens()[0].alive);
    println!(
        "PLAN058_TIMING test=av_tick_chain_matches_the_oracle_and_stops_at_the_wall ms={} first_contact_tick={tick} stop={sx} t={t} credits=32",
        t0.elapsed().as_millis()
    );
}

/// Candidate filtering for the independent oracle: the mover row and the
/// render-only ground are skipped, Minkowski shrink by the player half.
fn oracle_sweep(px: f32, py: f32, dx: f32, dy: f32) -> (bool, i32, f32, bool) {
    // (ordinal, lo, hi)
    let boxes = [
        (0usize, [2.0f32, -4.0], [14.0, 4.0]),
        (2, [9.0, -3.0], [11.0, 3.0]),
        (3, [3.0, -3.5], [5.0, -1.5]),
        (4, [3.5, 1.5], [4.5, 2.5]),
        (5, [3.5, 2.75], [4.5, 3.75]),
    ];
    let mut target = -1i32;
    let mut fraction = 2.0f32;
    for (ordinal, lo, hi) in boxes {
        if ordinal == PLAYER_ORDINAL || ordinal == 0 {
            continue;
        }
        let l = lo[0] - PLAYER_HALF[0];
        let h = hi[0] + PLAYER_HALF[0];
        let yl = lo[1] - PLAYER_HALF[1];
        let yh = hi[1] + PLAYER_HALF[1];
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
        target = ordinal as i32;
        fraction = enter;
    }
    if target < 0 {
        return (false, -1, 1.0, false);
    }
    let (_, lo, hi) = boxes
        .iter()
        .find(|(o, _, _)| *o as i32 == target)
        .expect("target");
    let l = lo[0] - PLAYER_HALF[0];
    let h = hi[0] + PLAYER_HALF[0];
    let yl = lo[1] - PLAYER_HALF[1];
    let yh = hi[1] + PLAYER_HALF[1];
    let overlap = px >= l && px <= h && py >= yl && py <= yh;
    (true, target, fraction, overlap)
}

/// Bare-box segment oracle (no Minkowski), nearest t, ties by lower ordinal.
fn oracle_segment(px: f32, py: f32) -> (bool, i32, f32) {
    let boxes = [
        (2usize, [9.0f32, -3.0], [11.0, 3.0]),
        (3, [3.0, -3.5], [5.0, -1.5]),
        (4, [3.5, 1.5], [4.5, 2.5]),
        (5, [3.5, 2.75], [4.5, 3.75]),
    ];
    let (ax, ay) = (px, py);
    let (bx, by) = (px, py + 6.0);
    let (dx, dy) = (bx - ax, by - ay);
    let mut target = -1i32;
    let mut fraction = 2.0f32;
    for (ordinal, lo, hi) in boxes {
        if ordinal == PLAYER_ORDINAL {
            continue;
        }
        let (mut enter, mut leave) = (0.0f32, 1.0f32);
        if dx != 0.0 {
            let near = (lo[0] - ax) / dx;
            let far = (hi[0] - ax) / dx;
            enter = enter.max(near);
            leave = leave.min(far);
            if enter > leave {
                continue;
            }
        } else if ax < lo[0] || ax > hi[0] {
            continue;
        }
        if dy != 0.0 {
            let near = (lo[1] - ay) / dy;
            let far = (hi[1] - ay) / dy;
            enter = enter.max(near);
            leave = leave.min(far);
            if enter > leave {
                continue;
            }
        } else if ay < lo[1] || ay > hi[1] {
            continue;
        }
        if enter >= fraction {
            continue;
        }
        target = ordinal as i32;
        fraction = enter;
    }
    if target < 0 {
        (false, -1, 1.0)
    } else {
        (true, target, fraction)
    }
}

#[test]
fn av_input_variants_and_changed_speed_follow_the_source() {
    let t0 = Instant::now();
    let compiled = compile(SOURCE);
    let def = tick_symbol(&compiled, "tick_step");
    // (input axis, expected per-Tick displacement magnitude, expected sign)
    for (ix, iy, want) in [
        (1.0f32, 0.0f32, (0.13333334f32, 0.0f32)),
        (1.0, 1.0, (0.09428090, 0.09428090)),
        (0.0, 0.0, (0.0, 0.0)),
        (-1.0, 0.0, (-0.13333334, 0.0)),
    ] {
        let (mut session, host) = spawn_session(&compiled);
        host.borrow_mut().set_input(2, ix, iy, false);
        host.borrow_mut().set_dt(DT);
        session.begin_tick();
        session
            .run_def(&compiled.checked, &def)
            .unwrap_or_else(|e| panic!("input {ix},{iy}: {e}"));
        let world = row_world(&host, PLAYER_ORDINAL);
        assert!(
            (world[0] - (4.0 + want.0)).abs() <= 1e-4 && (world[1] - want.1).abs() <= 1e-4,
            "input ({ix},{iy}) -> {world:?}, want displacement {want:?}"
        );
        let magnitude = ((world[0] - 4.0).powi(2) + world[1].powi(2)).sqrt();
        let expected = if ix == 0.0 && iy == 0.0 {
            0.0
        } else {
            8.0 * DT
        };
        assert!(
            (magnitude - expected).abs() <= 1e-4,
            "input ({ix},{iy}) moves |v|·dt = {expected}: {magnitude}"
        );
        assert_eq!(session.credits_used(), 32);
    }
    // Changed source: speed 4.0 halves the per-Tick displacement (no test-side
    // fudge — the compiled source itself changes).
    let slow = SOURCE.replace("ix * 8.0 * dt", "ix * 4.0 * dt");
    let compiled = compile(&slow);
    let def = tick_symbol(&compiled, "tick_step");
    let (mut session, host) = spawn_session(&compiled);
    host.borrow_mut().set_input(2, 1.0, 0.0, false);
    host.borrow_mut().set_dt(DT);
    session.begin_tick();
    session.run_def(&compiled.checked, &def).expect("slow tick");
    let world = row_world(&host, PLAYER_ORDINAL);
    assert!(
        (world[0] - (4.0 + 4.0 * DT)).abs() <= 1e-5,
        "changed speed 4 moves half: {world:?}"
    );
    println!(
        "PLAN058_TIMING test=av_input_variants_and_changed_speed_follow_the_source ms={} variants=4 speed_change=0.0666667",
        t0.elapsed().as_millis()
    );
}

#[test]
fn av_budget_is_exact_and_over_budget_is_named_zero_effect_sticky() {
    let t0 = Instant::now();
    let compiled = compile(SOURCE);
    let tick = tick_symbol(&compiled, "tick_step");
    let peek = tick_symbol(&compiled, "peek");
    let sweep_only = tick_symbol(&compiled, "sweep_only");
    let (mut session, host) = spawn_session(&compiled);

    session.begin_tick();
    session
        .run_def(&compiled.checked, &tick)
        .expect("the 32-credit Tick");
    assert_eq!(session.credits_used(), 32);
    assert!(!session.tick_faulted());
    let row_before = row_world(&host, PLAYER_ORDINAL);
    let tokens_before = host.borrow().tokens().len();

    // A 1-credit call after the pool is spent is the named 33rd charge.
    let err = session
        .run_def(&compiled.checked, &peek)
        .expect_err("the 33rd credit is refused");
    assert!(
        err.contains("age-gfx.credits"),
        "named budget refusal: {err}"
    );
    assert!(session.tick_faulted(), "sticky for the rest of the Tick");
    assert_eq!(
        session.credits_used(),
        32,
        "a refused charge does not advance"
    );
    // A 16-credit op afterwards is still refused (would be 48 > 32) and never
    // reaches the provider: zero effect on rows and render tokens.
    let err = session
        .run_def(&compiled.checked, &sweep_only)
        .expect_err("48 > 32 stays refused");
    assert!(err.contains("age-gfx.credits"), "{err}");
    session
        .run_def(&compiled.checked, &peek)
        .expect_err("sticky until the Tick ends");
    assert_eq!(row_world(&host, PLAYER_ORDINAL), row_before, "no row move");
    assert_eq!(host.borrow().tokens().len(), tokens_before, "no publish");
    // The next Tick resets the pool (the Run/session stays alive).
    session.begin_tick();
    assert_eq!(session.credits_used(), 0);
    assert!(!session.tick_faulted(), "begin_tick clears the sticky flag");
    session
        .run_def(&compiled.checked, &tick)
        .expect("the next Tick is legal again");
    assert_eq!(session.credits_used(), 32);
    // stop() is Run-terminal: no further execution, named refusal.
    session.stop();
    let err = session
        .run_def(&compiled.checked, &tick)
        .expect_err("stopped session");
    assert!(err.contains("age-gfx.session-stopped"), "{err}");
    println!(
        "PLAN058_TIMING test=av_budget_is_exact_and_over_budget_is_named_zero_effect_sticky ms={} tick=32 peek=refused(33) sweep_only=refused(48) reset=32 stop=named",
        t0.elapsed().as_millis()
    );
}
