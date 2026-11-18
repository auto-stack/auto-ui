//! PLAN-058 T-05: consuming 981's dynamic token/generation interface
//! (`AgeGfxCall` via `dispatch_with_ctx`, auto-lang `0a167f611`). The session
//! delivers the facts (tick, callback generation, in-callback query epoch,
//! post-accounting credits, symbol) per dispatch; **the fault-13/33 judgments
//! and payloads live in this lane** (PLAN-938/UI058) — 981 does not judge.
//!
//! Rides the already-proven chains: the 36-tick continuous AV chain, the
//! dual-instance 2:1 scenario, and the budget hooks. Verified here:
//! * same callback, second query ⇒ epoch 1→2 — the earlier token is stale
//!   (the host's fault-13 judgment names both sides' facts);
//! * callback = generation boundary (epoch slot resets; a previous callback's
//!   token is dead), `begin_tick` = the Tick boundary (tick advances, credits
//!   reset);
//! * stale render handle (despawn then frame through the real lane) is the
//!   fault-33 refusal carrying the session facts.
use ace_av::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn, AgeGfxSession};
use ace_av_core::contract::Value;
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use autographics_auto_driver::authoring::gfx_host_arms::{NeutralGfxHost, QueryTokenBook};
use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Instant};

const SOURCE: &str = include_str!("../fixtures/plan058/topdown-node/main.at");
const DT: f32 = 1.0 / 60.0;
const PLAYER_ORDINAL: usize = 1;

fn compile(scenario: ComponentScenario) -> ace_core::source::age_package::AgePackageCompilation {
    let input = PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: SOURCE.into(),
        provider: None,
    };
    compile_age_package_with_scenario(
        &[input],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    )
    .unwrap_or_else(|ds| {
        panic!(
            "front door rejected the T-05 source: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })
}

fn tick_scenario() -> ComponentScenario {
    ComponentScenario {
        instances: 2,
        ops: vec![ScenarioOp::ForNode {
            node: 1,
            op: Box::new(ScenarioOp::Tick { instance: 0 }),
        }],
    }
}

struct SharedHost(Rc<RefCell<NeutralGfxHost>>);

impl AgeGfxHost for SharedHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
        self.0.borrow_mut().dispatch(symbol, args)
    }
    fn dispatch_with_ctx(
        &mut self,
        symbol: &str,
        args: &[Value],
        ctx: &ace_av::age_gfx::AgeGfxCall<'_>,
    ) -> Result<AgeGfxReturn, AgeGfxFault> {
        self.0.borrow_mut().dispatch_with_ctx(symbol, args, ctx)
    }
}

fn assets_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../autographics-node2d-pilot/fixtures/sprites2d/assets")
}

/// Resolve a state function by its frozen spelling candidates (the state
/// spelling is per-module node indexing; never guess a mangling rule).
fn def_of(compiled: &ace_core::source::age_package::AgePackageCompilation, name: &str) -> String {
    let candidates: &[&str] = match name.strip_prefix("__age_state_n1_") {
        Some(field) => &[
            name,
            Box::leak(format!("__age_state_0_{field}").into_boxed_str()),
        ],
        None => &[name],
    };
    for candidate in candidates {
        if let Some(symbol) = compiled.symbol_of(candidate) {
            return format!("d_{symbol}");
        }
    }
    panic!("no symbol for `{name}`")
}

/// One AV Tick (input (1,0)) on the prepared session; returns the callback seq.
fn run_tick(
    session: &mut AgeGfxSession,
    host: &Rc<RefCell<NeutralGfxHost>>,
    compiled: &ace_core::source::age_package::AgePackageCompilation,
    def: &str,
) -> u64 {
    host.borrow_mut().set_input(2, 1.0, 0.0, false);
    host.borrow_mut().set_dt(DT);
    session.begin_tick();
    session
        .run_def(&compiled.checked, def)
        .unwrap_or_else(|e| panic!("tick: {e}"));
    session.callback_seq()
}

/// The continuous chain rides the epoch facts: per Tick (one callback) the
/// first dispatched op carries epoch 0, the sweep moves it to 1, the segment
/// to 2, every scalar op in between carries the current epoch unchanged, the
/// credits are the post-accounting values, and the tick/callback boundaries
/// advance exactly as the session accessors report.
#[test]
fn epochs_ride_the_continuous_chain_and_the_boundaries_hold() {
    let t0 = Instant::now();
    let compiled = compile(tick_scenario());
    let def = def_of(&compiled, "__age_state_n1_0_last_node");
    let (mut session, host) = spawn(&compiled);

    for tick in 1u64..=3 {
        let callback = run_tick(&mut session, &host, &compiled, &def);
        assert_eq!(callback, tick, "one run_def per Tick advances the callback");
        assert_eq!(session.query_epoch(), 2, "tick {tick}: two queries ran");
        let calls: Vec<_> = host
            .borrow()
            .calls()
            .iter()
            .filter(|c| c.callback == callback)
            .cloned()
            .collect();
        assert!(!calls.is_empty(), "tick {tick}: dispatches recorded");
        // First dispatched op of the callback: no query yet (epoch slot at 0).
        assert_eq!(calls[0].symbol, "age.input.x.v1");
        assert_eq!(calls[0].query_epoch, 0, "tick {tick}: epoch slot reset");
        assert_eq!(calls[0].credits_used, 1);
        // The sweep is the first query (epoch 1), the segment the second
        // (epoch 2) — the same callback's second query invalidates the first.
        let sweep = calls
            .iter()
            .find(|c| c.symbol == "age.query2d.sweep.v1")
            .expect("sweep fact");
        assert_eq!(sweep.query_epoch, 1, "tick {tick}: sweep opens the epoch");
        assert_eq!(sweep.credits_used, 21, "tick {tick}: 5 + 16");
        let segment = calls
            .iter()
            .find(|c| c.symbol == "age.query2d.segment.v1")
            .expect("segment fact");
        assert_eq!(
            segment.query_epoch, 2,
            "tick {tick}: the second query moves the epoch"
        );
        assert_eq!(segment.credits_used, 30, "tick {tick}: 22 + 8");
        // Non-query ops carry the current epoch unchanged; the last dispatch of
        // the Tick is the frame publish at exactly 32 credits.
        let last = calls.last().expect("last fact");
        assert_eq!(last.symbol, "age.render2d.frame.v1");
        assert_eq!(last.query_epoch, 2);
        assert_eq!(last.credits_used, 32, "tick {tick}: the whole pool");
        for fact in &calls {
            assert_eq!(fact.tick, tick, "tick {tick}: the ctx names this Tick");
            assert_eq!(fact.callback, callback);
        }
        host.borrow_mut().clear_calls();
    }
    println!(
        "PLAN058_TIMING test=epochs_ride_the_continuous_chain_and_the_boundaries_hold ms={}",
        t0.elapsed().as_millis()
    );
}

fn spawn(
    compiled: &ace_core::source::age_package::AgePackageCompilation,
) -> (AgeGfxSession, Rc<RefCell<NeutralGfxHost>>) {
    let rows: Vec<autographics_auto_driver::authoring::gfx_host_arms::HostRow> = compiled
        .description
        .components
        .iter()
        .find(|c| c.name == "Player")
        .expect("Player component")
        .scene
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
            let visible = row
                .cpu
                .iter()
                .find(|(k, _)| k == "visible")
                .map(|(_, v)| matches!(v, ace_core::source::age_package::AgeSceneValue::Bool(true)))
                .unwrap_or(true);
            autographics_auto_driver::authoring::gfx_host_arms::HostRow {
                ordinal,
                world: [position[0], position[1]],
                half: [size[0] * 0.5, size[1] * 0.5],
                layer: match row.layer {
                    Some(ace_core::source::age_package::AgeRowLayer::RenderOnly) => {
                        autographics_auto_driver::authoring::document::Layer::RenderOnly
                    }
                    None => autographics_auto_driver::authoring::document::Layer::Collision,
                },
                visible,
            }
        })
        .collect();
    let host = Rc::new(RefCell::new(NeutralGfxHost::new(rows, assets_root())));
    let session = AgeGfxSession::new(Box::new(SharedHost(host.clone())));
    (session, host)
}

/// Same callback: the second query moved the epoch 1→2, so the first token is
/// stale — the host's fault-13 judgment refuses it with a payload naming both
/// sides; the current token resolves; a previous callback's token is dead by
/// the generation boundary; a previous Tick's token by the Tick boundary.
#[test]
fn fault13_judgment_names_the_stale_token_facts() {
    let t0 = Instant::now();
    let compiled = compile(tick_scenario());
    let def = def_of(&compiled, "__age_state_n1_0_last_node");
    let (mut session, host) = spawn(&compiled);

    // Tick 1, callback 1: sweep (epoch 1) then segment (epoch 2).
    run_tick(&mut session, &host, &compiled, &def);
    let book = host.borrow().query_book().expect("query book");
    assert_eq!(
        book,
        QueryTokenBook {
            tick: 1,
            callback: 1,
            epoch: 2,
            sweep: false,
        },
        "the segment is the latest token of callback 1"
    );
    // The current token resolves.
    assert_eq!(host.borrow().resolve_query_token(book), Ok(()));
    // The sweep's token (epoch 1, same callback) is stale: fault 13, payload
    // names the held facts and the epoch that killed it.
    let sweep_token = QueryTokenBook {
        epoch: 1,
        ..book
    };
    let err = host
        .borrow()
        .resolve_query_token(sweep_token)
        .expect_err("the invalidated token is stale");
    assert_eq!(err.code, "age-gfx.stale-query-token");
    assert!(
        err.message.contains("epoch to 2"),
        "payload names the killing epoch: {}",
        err.message
    );

    // Pure callback boundary (same Tick): sweep_only opens this Tick's epoch
    // at callback 1 (16 credits), peek is callback 2 (1 credit, 17 total) —
    // only the callback moved, and the callback-1 token is already dead.
    let sweep_only = def_of(&compiled, "sweep_only");
    let peek = def_of(&compiled, "peek");
    host.borrow_mut().set_input(2, 1.0, 0.0, false);
    host.borrow_mut().set_dt(DT);
    session.begin_tick();
    session
        .run_def(&compiled.checked, &sweep_only)
        .expect("sweep-only callback");
    // (The first part of this test consumed Tick 1 / callback 1, so this is
    // Tick 2 / callback 2.)
    let sweep_book = host.borrow().query_book().expect("query book");
    assert_eq!(
        sweep_book,
        QueryTokenBook {
            tick: 2,
            callback: 2,
            epoch: 1,
            sweep: true,
        }
    );
    session
        .run_def(&compiled.checked, &peek)
        .expect("peek callback");
    assert_eq!(session.callback_seq(), 3, "peek ran its own callback");
    assert_eq!(session.query_epoch(), 0, "no new query in the peek callback");
    assert_eq!(session.credits_used(), 17, "16 + 1 within the pool");
    let err = host
        .borrow()
        .resolve_query_token(sweep_book)
        .expect_err("the callback-2 token is dead in callback 3");
    assert!(
        err.message.contains("moved to callback 3"),
        "{}",
        err.message
    );

    // Cross-Tick: a fresh Tick runs the full 32-credit callback (its own epoch
    // slot re-issued 1→2), and the Tick-1 token is dead in Tick 2.
    run_tick(&mut session, &host, &compiled, &def);
    assert_eq!(session.callback_seq(), 4);
    assert_eq!(session.credits_used(), 32, "the full pool again");
    let book = host.borrow().query_book().expect("tick-3 book");
    assert_eq!(
        book,
        QueryTokenBook {
            tick: 3,
            callback: 4,
            epoch: 2,
            sweep: false,
        }
    );
    let err = host
        .borrow()
        .resolve_query_token(sweep_book)
        .expect_err("the Tick-2 token is dead in Tick 3");
    assert!(
        err.message.contains("moved to tick 3"),
        "{}",
        err.message
    );
    println!(
        "PLAN058_TIMING test=fault13_judgment_names_the_stale_token_facts ms={}",
        t0.elapsed().as_millis()
    );
}

/// The dual-instance ride: the Target Touch callbacks (no graphics ops) and
/// the Player Tick callback interleave in one session — each run_def is its
/// own callback generation, the epoch facts only appear where queries run,
/// and the 2:1 isolation values hold on the same source.
#[test]
fn dual_instance_callbacks_carry_the_generation_boundary() {
    let t0 = Instant::now();
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::ForNode {
                node: 0,
                op: Box::new(ScenarioOp::Send {
                    instance: 0,
                    msg: "Touch".into(),
                }),
            },
            ScenarioOp::ForNode {
                node: 0,
                op: Box::new(ScenarioOp::Send {
                    instance: 0,
                    msg: "Touch".into(),
                }),
            },
            ScenarioOp::ForNode {
                node: 0,
                op: Box::new(ScenarioOp::Send {
                    instance: 1,
                    msg: "Touch".into(),
                }),
            },
            ScenarioOp::ForNode {
                node: 1,
                op: Box::new(ScenarioOp::Tick { instance: 0 }),
            },
        ],
    };
    let compiled = compile(scenario);
    let hits_def = |inst: usize| def_of(&compiled, &format!("__age_state_{inst}_hits"));
    let (mut session, host) = spawn(&compiled);

    host.borrow_mut().set_input(2, 1.0, 0.0, false);
    host.borrow_mut().set_dt(DT);
    session.begin_tick();
    let hits_a = session
        .run_def(&compiled.checked, &hits_def(0))
        .expect("target A replay");
    assert_eq!(hits_a, Value::I32(2), "target A took two Touch events");
    assert_eq!(session.callback_seq(), 1, "the replay ran one callback");
    assert!(
        host.borrow()
            .calls()
            .iter()
            .all(|c| c.callback == 1 && c.query_epoch == 0),
        "the Touch replay dispatches no graphics ops"
    );
    let hits_b = session
        .run_def(&compiled.checked, &hits_def(1))
        .expect("target B replay");
    assert_eq!(hits_b, Value::I32(1), "target B took one Touch event");
    assert_eq!(session.callback_seq(), 2, "the second replay is a new callback");
    // The Tick callback re-issues the query epochs from zero in its own slot.
    session.begin_tick();
    let last_node = session
        .run_def(&compiled.checked, &def_of(&compiled, "__age_state_n1_0_last_node"))
        .expect("tick replay");
    assert_eq!(last_node, Value::I32(4), "the nearest target is target_a");
    assert_eq!(session.callback_seq(), 3, "the Tick is the third callback");
    assert_eq!(session.query_epoch(), 2, "sweep then segment in the Tick");
    let query_facts: Vec<(String, u32)> = host
        .borrow()
        .calls()
        .iter()
        .filter(|c| c.callback == 3)
        .filter(|c| c.symbol.contains("query2d"))
        .map(|c| (c.symbol.clone(), c.query_epoch))
        .collect();
    assert_eq!(
        query_facts,
        vec![
            ("age.query2d.sweep.v1".to_string(), 1),
            ("age.query2d.segment.v1".to_string(), 2),
        ],
        "the Tick callback's queries carry epochs 1→2"
    );
    println!(
        "PLAN058_TIMING test=dual_instance_callbacks_carry_the_generation_boundary ms={}",
        t0.elapsed().as_millis()
    );
}

/// Fault 33 (stale render handle) through the real lane: `frame_after_despawn`
/// spawns, despawns, then frames the dead handle — the host's fault-33
/// judgment refuses the write with the session facts in the payload, and the
/// frame never publishes.
#[test]
fn stale_render_handle_is_fault33_with_payload() {
    let t0 = Instant::now();
    let compiled = compile(tick_scenario());
    let (mut session, host) = spawn(&compiled);
    let drive = def_of(&compiled, "frame_after_despawn");
    host.borrow_mut().set_input(2, 1.0, 0.0, false);
    host.borrow_mut().set_dt(DT);
    session.begin_tick();
    let err = session
        .run_def(&compiled.checked, &drive)
        .expect_err("framing a despawned handle is refused");
    assert!(
        err.contains("age-gfx.stale-render-handle"),
        "named fault-33 refusal: {err}"
    );
    assert!(
        err.contains("old generation"),
        "payload names the dead generation: {err}"
    );
    // Zero effect: the refused frame never published.
    let frames = host.borrow().frames().len();
    assert_eq!(frames, 0, "the stale write published nothing");
    println!(
        "PLAN058_TIMING test=stale_render_handle_is_fault33_with_payload ms={}",
        t0.elapsed().as_millis()
    );
}
