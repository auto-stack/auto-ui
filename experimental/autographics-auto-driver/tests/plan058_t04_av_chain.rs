//! PLAN-058 T-04: the real AV chain — the normal AGE Node fixture compiled by
//! the refreshed `.sdk-source` front door and executed Tick by Tick through the
//! 981 `AgeGfxSession` (typed dispatch, shared 32-credit pool) with
//! `NeutralGfxHost` as the registered provider.
//!
//! Since the auto-lang `6e456794d` pin (PLAN-981 G1+G2) the fixture is in the
//! REAL author form: the Tick lives in `on Life.Tick` with hook-local `let` +
//! `if/else` (G2), and the render-only ground is a SOURCE declaration
//! (`layer: render-only`, G1) read from the checked row description's typed
//! `AgeSceneRow.layer` fact — no host-side row-name table anymore.
//!
//! Rows are **source-bound**: they are derived from the compiled description's
//! `scene` rows (declaration order = ordinal), not from a test-side table.
//! Every Tick's position/query results are compared against the independent
//! f32 oracle (`plan058_t04_oracle.rs` rules, re-implemented here as the same
//! recurrence) and the session's per-Tick credit use is asserted.
use ace_av::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn, AgeGfxSession};
use ace_av_core::contract::Value;
use ace_core::source::age_component::{ComponentScenario, ComponentStmt, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{
    compile_age_package_with_scenario, AgePackageCompilation, AgeRowLayer,
};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use autographics_auto_driver::authoring::{
    document::Layer,
    gfx_host_arms::{HostRow, NeutralGfxHost},
};
use autographics_auto_driver::authoring::{
    assets::Registry,
    host::Host,
    play::{Action, Frame, State},
    storage::Storage,
};
use glam::Vec2;
use png::{BitDepth, ColorType, Decoder};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    fs,
    io::Cursor,
    path::PathBuf,
    rc::Rc,
    time::Instant,
};

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
    // One instance, one Life.Tick: the scenario generates the per-field state
    // functions (`__age_state_0_<field>` = field init + handler replay) the
    // runner drives through the session.
    compile_age_package_with_scenario(
        &[input],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Tick { instance: 0 }],
        },
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
/// `visible`. The render-only ground layer is the SOURCE's typed fact (G1):
/// `AgeSceneRow.layer` on the same checked description every lane consumes.
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
                layer: match row.layer {
                    Some(AgeRowLayer::RenderOnly) => Layer::RenderOnly,
                    None => Layer::Collision,
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

/// The Tick is the REAL `on Life.Tick` handler now (G2): the scenario compiles
/// it into per-field state functions (`__age_state_0_<field>` = field init +
/// handler replay); the runner drives the `last_node` one — the handler's only
/// outcome field — so the whole body executes exactly once per Tick.
fn tick_def(compiled: &AgePackageCompilation) -> String {
    tick_symbol(compiled, "__age_state_0_last_node")
}

fn row_world(host: &Rc<RefCell<NeutralGfxHost>>, ordinal: usize) -> [f32; 2] {
    host.borrow().row(ordinal).expect("row").world
}

#[test]
fn av_tick_chain_matches_the_oracle_and_stops_at_the_wall() {
    let t0 = Instant::now();
    let compiled = compile(SOURCE);
    let def = tick_def(&compiled);
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
        // The state function's observable result is the handler's outcome
        // field: the start_overlap branch records -2, the else branch records
        // the segment node (G2 branch facts, driven every Tick).
        let want_last = if sweep.start_overlap { -2 } else { cast_node };
        assert_eq!(value, Value::I32(want_last), "tick {tick}: def result");
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
    let def = tick_def(&compiled);
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
    // Changed source: the model's speed init 8.0 -> 4.0 halves the per-Tick
    // displacement (no test-side fudge — the compiled source itself changes).
    let slow = SOURCE.replace("speed f32 = 8.0", "speed f32 = 4.0");
    let compiled = compile(&slow);
    let def = tick_def(&compiled);
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
    let tick = tick_def(&compiled);
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

/// G1/G2 are source facts on the same checked description: the ground's typed
/// `layer: render-only` reaches the AV lane's row table (and its removal
/// re-admits the ground — the 055 counterexample), and the `on Life.Tick`
/// hook-local `let` + `if q.start_overlap` branch decides what the Tick
/// records (blocked -2 vs the nearest target).
#[test]
fn g2_branch_and_g1_layer_facts_are_source_bound() {
    let t0 = Instant::now();
    let compiled = compile(SOURCE);

    // G1 typed fact on the checked row description (+ the cpu Symbol twin).
    let scene = &compiled.description.components[0].scene;
    assert_eq!(scene[0].name, "ground");
    assert_eq!(scene[0].layer, Some(AgeRowLayer::RenderOnly));
    assert!(
        scene[0].cpu.iter().any(|(k, v)| k == "layer"
            && matches!(
                v,
                ace_core::source::age_package::AgeSceneValue::Symbol(s) if s == "render-only"
            )),
        "the cpu face carries the render-only symbol twin"
    );
    for (ordinal, row) in scene.iter().enumerate().skip(1) {
        assert_eq!(
            row.layer, None,
            "row {ordinal} carries no layer declaration"
        );
    }
    // G2 handler statement facts: hook-local lets (across the three axis
    // branches) and the closing branch on the same handler.
    let handler = &compiled.description.components[0].handler_stmts;
    let tick_stmts = &handler
        .iter()
        .find(|(msg, _)| msg == "Life.Tick")
        .expect("Life.Tick handler")
        .1;
    fn count_lets(stmts: &[ComponentStmt]) -> usize {
        stmts
            .iter()
            .map(|s| match s {
                ComponentStmt::Let { .. } => 1,
                ComponentStmt::If { then, els, .. } => {
                    count_lets(then) + els.as_ref().map(|e| count_lets(e)).unwrap_or(0)
                }
                _ => 0,
            })
            .sum()
    }
    assert!(
        count_lets(tick_stmts) >= 10,
        "hook-local lets are projected: {tick_stmts:?}"
    );
    assert!(
        matches!(tick_stmts.last(), Some(ComponentStmt::If { .. })),
        "the Tick records through the axis branches"
    );

    // Overlap run: the player starts inside wall_b — the sweep reports
    // start_overlap at t=0 and the then-branch records -2; the ground
    // (render-only) is NOT the hit even though it also contains the mover.
    let overlapped = compile(&SOURCE.replace("position: (4.0, 0.0)", "position: (4.0, -2.5)"));
    let def = tick_def(&overlapped);
    let (mut session, host) = spawn_session(&overlapped);
    host.borrow_mut().set_input(2, 1.0, 0.0, false);
    host.borrow_mut().set_dt(DT);
    session.begin_tick();
    let value = session
        .run_def(&overlapped.checked, &def)
        .unwrap_or_else(|e| panic!("overlap tick: {e}"));
    assert_eq!(value, Value::I32(-2), "then branch: blocked, not a target");
    assert_eq!(
        session.credits_used(),
        32,
        "the branch only chooses what is recorded"
    );
    let sweep = host.borrow().last_sweep().expect("sweep ran");
    assert_eq!(sweep.node, 3, "wall_b is the contact, not the ground");
    assert_eq!(sweep.t, 0.0, "the overlap contact is at t=0");
    assert!(sweep.start_overlap, "the mover starts inside wall_b");
    assert_eq!(
        row_world(&host, PLAYER_ORDINAL),
        [4.0, -2.5],
        "no movement out of the overlap"
    );
    let publishes = host.borrow().publishes().to_vec();
    assert_eq!(publishes.len(), 1, "same-Tick publish still runs");
    assert!((publishes[0].x - 4.0).abs() <= 1e-5);

    // Negative control: remove the ground's `layer: render-only` declaration —
    // the typed fact disappears and the ground re-enters the candidate set,
    // winning this same query by declaration ordinal (the 055 counterexample).
    let bare = compile(&SOURCE.replace("            layer: render-only\n", ""));
    assert_eq!(
        bare.description.components[0].scene[0].layer,
        None,
        "no declaration, no fact"
    );
    let def = tick_def(&bare);
    let (mut session, host) = spawn_session(&bare);
    host.borrow_mut().set_input(2, 1.0, 0.0, false);
    host.borrow_mut().set_dt(DT);
    session.begin_tick();
    let value = session
        .run_def(&bare.checked, &def)
        .unwrap_or_else(|e| panic!("bare-ground tick: {e}"));
    let sweep = host.borrow().last_sweep().expect("sweep ran");
    assert_eq!(sweep.node, 0, "the unlayered ground is misselected first");
    assert_eq!(sweep.t, 0.0);
    assert!(sweep.start_overlap);
    assert_eq!(value, Value::I32(-2), "still the blocked record");
    println!(
        "PLAN058_TIMING test=g2_branch_and_g1_layer_facts_are_source_bound ms={} overlap=node3,t=0 bare_ground=node0",
        t0.elapsed().as_millis()
    );
}

// ------------------------------------------------ AV/GPU same-Tick pre-proof --
//
// The task's AV/GPU sub-face: the AV Tick moves the host row (provider state),
// and the *same* Tick's row position is pushed into a real offscreen renderer
// (sprites2d scene, atlas sprite on the player, camera bound to a parked
// spectator so the world->screen mapping stays fixed). The frame is captured
// immediately after one step, and the sprite's colour-mask centroid is compared
// with the camera projection of the oracle row position. Ticks 34/35/36 (1-based)
// straddle the first wall contact (stop 8.6 at tick 35).

const ATLAS_PNG: &[u8] = include_bytes!(
    "../../autographics-node2d-pilot/fixtures/sprites2d/assets/95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png"
);
const ATLAS_SHA: &str = "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1";

const RENDER_WORLD: &str = "var ok=0.0\nfn app_start() {\n ok=ok+ag_char_bind(1)*1.0\n ok=ok+ag_camera_follow_2d(1, 2.0, -4.0, 34.0, 12.0, 12.0, 8.0)*1.0\n}\nfn app_update(dt float) {\n}\n";

fn artifacts() -> PathBuf {
    let path = PathBuf::from(
        std::env::var("PLAN058_ARTIFACTS")
            .unwrap_or_else(|_| "D:/autostack/plan-artifacts/plan-058/auto-ui".into()),
    );
    fs::create_dir_all(&path).expect("artifacts dir");
    path
}

fn sha_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The renderer document: the same six scene rows, the player at `player_x`
/// carrying the atlas frame 0, and a parked spectator as the camera target.
fn gpu_scene(player_x: f32) -> String {
    let world_sha = sha_hex(RENDER_WORLD.as_bytes());
    let objects = vec![
        ("spectator", 1u64, [-100.0f32, -100.0], [0.8f32, 1.0], false, false),
        ("ground", 2, [8.0, 0.0], [12.0, 8.0], true, false),
        ("player", 3, [player_x, 0.0], [0.8, 1.0], true, true),
        ("wall_a", 4, [10.0, 0.0], [2.0, 6.0], true, false),
        ("wall_b", 5, [4.0, -2.5], [2.0, 2.0], true, false),
        ("target_a", 6, [4.0, 2.0], [1.0, 1.0], true, false),
        ("target_b", 7, [4.0, 3.25], [1.0, 1.0], true, false),
    ];
    let rows: Vec<serde_json::Value> = objects
        .into_iter()
        .map(|(name, id, pos, scale, visible, sprite)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "kind": "quad",
                "translation": [pos[0], pos[1], 0.0],
                "scale": [scale[0], scale[1], 1.0],
                "yaw": 0.0,
                "pitch": 0.0,
                "rgba": sprite_rgba(name),
                "visible": visible,
                "parent": null,
                "resource": null,
                "sprite": if sprite {
                    serde_json::json!({"asset": format!("sha256:{ATLAS_SHA}"), "frame": [0, 0, 32, 32], "flip": false})
                } else {
                    serde_json::Value::Null
                }
            })
        })
        .collect();
    serde_json::json!({
        "behavior": {
            "api": "autographics.sprites2d/1",
            "source": {"format": "auto-source", "sha256": world_sha, "byte_length": RENDER_WORLD.len()},
            "actors": {"two_d": 1, "three_d": null}
        },
        "schema": "autographics.scene/6",
        "document_id": "plan058-t04-av-gpu",
        "resources": ["circle", "quad", "cylinder", "player", "mob"],
        "objects": rows,
        "groups": [],
        "assets": [{
            "format": "png",
            "sha256": ATLAS_SHA,
            "byte_length": ATLAS_PNG.len(),
            "name": format!("{ATLAS_SHA}.png"),
            "width": 96,
            "height": 32
        }]
    })
    .to_string()
}

fn sprite_rgba(name: &str) -> [f64; 4] {
    match name {
        "ground" => [0.18, 0.45, 0.22, 1.0],
        "player" => [0.9, 0.3, 0.2, 1.0],
        "wall_a" => [0.55, 0.55, 0.6, 1.0],
        "wall_b" => [0.35, 0.35, 0.4, 1.0],
        "target_a" => [1.0, 0.55, 0.15, 1.0],
        "target_b" => [0.55, 0.15, 0.8, 1.0],
        _ => [0.5, 0.5, 0.55, 1.0],
    }
}

fn stage_gpu(tag: &str, player_x: f32) -> PathBuf {
    let scene = gpu_scene(player_x);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan058-t04-gpu")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("scripts")).expect("scripts");
    fs::create_dir_all(dir.join("assets")).expect("assets");
    fs::write(dir.join("scene.agscene"), &scene).expect("scene");
    fs::write(
        dir.join("scripts").join(format!("{}.at", sha_hex(RENDER_WORLD.as_bytes()))),
        RENDER_WORLD,
    )
    .expect("world");
    fs::write(
        dir.join("assets").join(format!("{ATLAS_SHA}.png")),
        ATLAS_PNG,
    )
    .expect("asset");
    dir
}

fn pump(host: &mut Host) {
    let deadline = Instant::now() + std::time::Duration::from_secs(31);
    while host.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < deadline, "host settle");
        host.frame(None).expect("frame");
    }
}

fn gpu_host(tag: &str, player_x: f32) -> Host {
    let dir = stage_gpu(tag, player_x);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .unwrap_or_else(|e| panic!("{tag} load: {e:?}"));
    let mut host = Host::with_project(doc, registry, code).unwrap_or_else(|e| panic!("{tag}: {e}"));
    host.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start: {e:?}"));
    host.pause_play("");
    pump(&mut host);
    if let Some(err) = host.play.as_ref().and_then(|p| p.error.clone()) {
        assert!(err.is_empty(), "{tag} start fault: {err}");
    }
    host
}

fn step_host(host: &mut Host) {
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
    if let Some(err) = host.play.as_ref().and_then(|p| p.error.clone()) {
        assert!(err.is_empty(), "tick fault: {err}");
    }
    assert_eq!(
        host.play.as_ref().map(|p| p.state),
        Some(State::Paused),
        "tick settled"
    );
}

fn frame_of(host: &Host) -> &Frame {
    host.play
        .as_ref()
        .and_then(|p| p.last.as_ref())
        .expect("frame")
}

/// Follow centre comes from the accepted camera target (the parked spectator),
/// so the world->surface mapping is fixed while the player moves.
fn screen_of(host: &Host, world: Vec2) -> Vec2 {
    let f = frame_of(host);
    let cam = f.camera.as_ref().expect("camera");
    let target = f.nodes.get(&cam.target_id).expect("camera target");
    let centre = cam
        .follow_center_2d(Vec2::new(target.translation[0], target.translation[1]))
        .expect("centre");
    cam.screen_2d(world, centre).expect("projection")
}

fn px_per_world(host: &Host) -> Vec2 {
    frame_of(host)
        .camera
        .as_ref()
        .expect("camera")
        .letterbox_2d()
        .expect("letterbox")
        .0
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
    assert_eq!(info.color_type, ColorType::Rgba);
    assert_eq!(info.bit_depth, BitDepth::Eight);
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

fn srgb_encode(v: f64) -> u8 {
    let c = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (c.mul_add(255.0, 0.5).floor() as i32).clamp(0, 255) as u8
}

fn colour_class(rgba: [f64; 4]) -> [u8; 3] {
    [
        srgb_encode(rgba[0]),
        srgb_encode(rgba[1]),
        srgb_encode(rgba[2]),
    ]
}

/// The four opaque frame-0 colours and their alpha-255 texel centroid, in texel
/// coordinates with half-texel centres (the mask the screenshot uses).
fn atlas_mask() -> (Vec<[u8; 3]>, f32, f32, usize) {
    let image = decode_png(ATLAS_PNG);
    let mut colours: Vec<[u8; 3]> = Vec::new();
    let mut sum = (0.0f64, 0.0f64);
    let mut count = 0usize;
    for y in 0..32u32 {
        for x in 0..32u32 {
            let p = pixel(&image, x, y);
            if p[3] != 255 {
                continue;
            }
            let c = [p[0], p[1], p[2]];
            if !colours.contains(&c) {
                colours.push(c);
            }
            sum.0 += f64::from(x) + 0.5;
            sum.1 += f64::from(y) + 0.5;
            count += 1;
        }
    }
    assert_eq!(colours.len(), 4, "four opaque atlas colours");
    (
        colours,
        (sum.0 / count as f64) as f32,
        (sum.1 / count as f64) as f32,
        count,
    )
}

fn mask_matches(p: [u8; 4], colours: &[[u8; 3]]) -> bool {
    colours.iter().any(|c| {
        (i32::from(p[0]) - i32::from(c[0])).abs() <= 4
            && (i32::from(p[1]) - i32::from(c[1])).abs() <= 4
            && (i32::from(p[2]) - i32::from(c[2])).abs() <= 4
    })
}

/// Colour-mask centroid of one class in the captured frame, in surface pixels.
fn class_centroid(image: &Image, wanted: [u8; 3]) -> Option<(f64, f64, usize)> {
    let mut sum = (0.0f64, 0.0f64);
    let mut count = 0usize;
    for y in 0..image.height {
        for x in 0..image.width {
            let p = pixel(image, x, y);
            if (i32::from(p[0]) - i32::from(wanted[0])).abs() <= 6
                && (i32::from(p[1]) - i32::from(wanted[1])).abs() <= 6
                && (i32::from(p[2]) - i32::from(wanted[2])).abs() <= 6
            {
                sum.0 += f64::from(x);
                sum.1 += f64::from(y);
                count += 1;
            }
        }
    }
    (count > 0).then(|| (sum.0 / count as f64, sum.1 / count as f64, count))
}

fn sprite_centroid(image: &Image, colours: &[[u8; 3]]) -> (f64, f64, usize) {
    let mut sum = (0.0f64, 0.0f64);
    let mut count = 0usize;
    for y in 0..image.height {
        for x in 0..image.width {
            if mask_matches(pixel(image, x, y), colours) {
                sum.0 += f64::from(x);
                sum.1 += f64::from(y);
                count += 1;
            }
        }
    }
    assert!(count > 0, "sprite mask present");
    (sum.0 / count as f64, sum.1 / count as f64, count)
}

#[test]
fn av_gpu_same_tick_pixels_at_the_wall_contact() {
    let t0 = Instant::now();
    let compiled = compile(SOURCE);
    let def = tick_def(&compiled);
    let (mut session, host) = spawn_session(&compiled);
    let (colours, texel_cx, texel_cy, opaque) = atlas_mask();

    // Oracle + AV rows for ticks 34/35/36 (1-based), 32 credits each.
    let mut oracle = row_world(&host, PLAYER_ORDINAL);
    let mut rows: Vec<(i32, [f32; 2])> = Vec::new();
    for tick in 1..=36 {
        host.borrow_mut().set_input(2, 1.0, 0.0, false);
        host.borrow_mut().set_dt(DT);
        session.begin_tick();
        session
            .run_def(&compiled.checked, &def)
            .unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        assert_eq!(session.credits_used(), 32, "tick {tick} budget");
        let (px, py) = (oracle[0], oracle[1]);
        let dx = SPEED * DT;
        let (hit, _node, t, _overlap) = oracle_sweep(px, py, dx, 0.0);
        oracle = if hit {
            [px + dx * t, py]
        } else {
            [px + dx, py]
        };
        if (34..=36).contains(&tick) {
            let row = row_world(&host, PLAYER_ORDINAL);
            assert!(
                (row[0] - oracle[0]).abs() <= 1e-5 && (row[1] - oracle[1]).abs() <= 1e-5,
                "tick {tick}: AV row {row:?} vs oracle {oracle:?}"
            );
            rows.push((tick, row));
        }
    }
    assert_eq!(rows.len(), 3);
    assert!(
        (rows[1].1[0] - WALL_FACE).abs() <= 1e-5,
        "tick 35 stops at the wall face: {:?}",
        rows[1].1
    );
    assert_eq!(rows[2].1, rows[1].1, "tick 36 does not penetrate");

    let mut evidence = Vec::new();
    let mut captured: Vec<((f64, f64), (f64, f64))> = Vec::new();
    for (tick, row) in &rows {
        let t1 = Instant::now();
        let mut gpu = gpu_host(&format!("tick{tick}"), row[0]);
        step_host(&mut gpu);
        assert_eq!(frame_of(&gpu).tick, 1, "one rendered step after the AV Tick");
        let (sha, bytes, w, h) = gpu
            .rt
            .capture(gpu.two, &format!("t04-gpu-tick{tick}"))
            .expect("capture");
        let name = format!("t04-gpu-tick{tick}.png");
        fs::write(artifacts().join(&name), &bytes).expect("write png");
        let image = decode_png(&bytes);
        assert_eq!((image.width, image.height), (w, h));

        // Expected mask centroid: camera projection of the oracle row position
        // plus the atlas texel-centroid offset (sub-pixel).
        let scale = px_per_world(&gpu);
        let half = Vec2::new(PLAYER_HALF[0] * scale.x, PLAYER_HALF[1] * scale.y);
        let projected = screen_of(&gpu, Vec2::new(row[0], row[1]));
        let expected = Vec2::new(
            projected.x + (texel_cx / 32.0 * 2.0 - 1.0) * half.x,
            projected.y + (texel_cy / 32.0 * 2.0 - 1.0) * half.y,
        );
        let (cx, cy, mask_px) = sprite_centroid(&image, &colours);
        let dx_px = cx - f64::from(expected.x);
        let dy_px = cy - f64::from(expected.y);
        assert!(
            dx_px.abs() <= 1.0 && dy_px.abs() <= 1.0,
            "tick {tick}: centroid ({cx:.2}, {cy:.2}) vs projection ({:.2}, {:.2})",
            expected.x,
            expected.y
        );
        // Scene sanity: the ground fills the view and the wall sits where the
        // camera projects it.
        let ground_px = class_centroid(&image, colour_class(sprite_rgba("ground")))
            .map(|(_, _, n)| n)
            .unwrap_or(0);
        assert!(
            ground_px > (800 * 380) / 2,
            "tick {tick}: ground visible pixels {ground_px}"
        );
        let wall = class_centroid(&image, colour_class(sprite_rgba("wall_a")))
            .expect("wall pixels");
        let wall_expected = screen_of(&gpu, Vec2::new(10.0, 0.0));
        assert!(
            (wall.0 - f64::from(wall_expected.x)).abs() <= 2.0
                && (wall.1 - f64::from(wall_expected.y)).abs() <= 2.0,
            "tick {tick}: wall centroid ({:.2}, {:.2}) vs projection ({:.2}, {:.2})",
            wall.0,
            wall.1,
            wall_expected.x,
            wall_expected.y
        );
        println!(
            "tick {tick}: row {:.6} projected ({:.2}, {:.2}) centroid ({:.2}, {:.2}) dx {dx_px:+.2} dy {dy_px:+.2} mask {mask_px}px ground {ground_px}px capture {w}x{h} sha={} step {}ms",
            row[0],
            expected.x,
            expected.y,
            cx,
            cy,
            &sha[..12],
            t1.elapsed().as_millis()
        );
        captured.push(((cx, cy), (f64::from(expected.x), f64::from(expected.y))));
        evidence.push(serde_json::json!({
            "tick": tick,
            "row_world": [row[0], row[1]],
            "projection": [f64::from(projected.x), f64::from(projected.y)],
            "expected_mask_centroid": [f64::from(expected.x), f64::from(expected.y)],
            "observed_mask_centroid": [cx, cy],
            "delta_px": [dx_px, dy_px],
            "mask_pixels": mask_px,
            "opaque_texels": opaque,
            "ground_pixels": ground_px,
            "credits": 32,
            "frame_tick": frame_of(&gpu).tick,
            "png": name,
            "png_sha256": sha,
        }));
        gpu.stop_play();
    }
    // Same-Tick discrimination: tick 35's centroid is at its own projection and
    // differs from tick 34 by the one-step world delta (8.533333 -> 8.6).
    let (c34, _e34) = captured[0];
    let (c35, _e35) = captured[1];
    let (c36, e36) = captured[2];
    let shift = ((c35.0 - c34.0).powi(2) + (c35.1 - c34.1).powi(2)).sqrt();
    let want_shift = f64::from((8.6f32 - 8.533333f32) * 47.5);
    assert!(
        (shift - want_shift).abs() <= 1.0,
        "tick 34 -> 35 centroid shift {shift:.2}px, want ~{want_shift:.2}px"
    );
    assert!(shift > 1.0, "the one-step shift is distinguishable: {shift:.2}px");
    let still = ((c36.0 - c35.0).powi(2) + (c36.1 - c35.1).powi(2)).sqrt();
    assert!(
        still <= 0.5,
        "tick 36 stays at tick 35 (no penetration): {still:.2}px"
    );
    assert!((c36.0 - e36.0).abs() <= 1.0);
    let json = serde_json::json!({
        "case": "av_gpu_same_tick_pixels_at_the_wall_contact",
        "atlas_sha256": ATLAS_SHA,
        "opaque_texels": opaque,
        "atlas_texel_centroid": [texel_cx, texel_cy],
        "ticks": evidence,
        "tick34_to_35_shift_px": shift,
        "expected_shift_px": want_shift,
        "tick35_to_36_shift_px": still,
        "wall_face": WALL_FACE,
        "dt": DT,
        "speed": SPEED,
    });
    fs::write(
        artifacts().join("t04-gpu-same-tick.json"),
        serde_json::to_vec_pretty(&json).expect("json"),
    )
    .expect("write json");
    println!(
        "PLAN058_TIMING test=av_gpu_same_tick_pixels_at_the_wall_contact ms={} shift34_35={shift:.2}px shift35_36={still:.2}px",
        t0.elapsed().as_millis()
    );
}
