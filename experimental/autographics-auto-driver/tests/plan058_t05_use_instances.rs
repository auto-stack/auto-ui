//! PLAN-058 T-05 (AV phase, `use` instances): the two same-definition
//! interaction targets become instances of a declared `Target` node carrying
//! its own model and `Touch` handler. This file first pins the language facts
//! the slice rests on (package-face msg handlers, instance scene rows with
//! per-instance cpu geometry, per-instance state functions), then runs the
//! V-S03 slice: two Touch events on target A and one on target B leave the
//! per-instance states at 2/1 with the Tick business untouched.
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use std::time::Instant;

fn compile_with(ops: Vec<ScenarioOp>) -> ace_core::source::age_package::AgePackageCompilation {
    let target = "node Target {\n    model {\n        hits int = 0\n    }\n    msg {\n        Touch\n    }\n    on {\n        .Touch -> {\n            .hits += 1\n        }\n    }\n    view-2d {\n        shape2d body {\n            kind: quad\n            size: (1.0, 1.0)\n            visible: true\n        }\n    }\n}\n";
    let player = "node Player {\n    model {\n        last_node i32 = -1\n        speed f32 = 8.0\n    }\n    on Life.Tick {\n        last_node = 4\n    }\n    view-2d {\n        shape2d ground {\n            kind: quad\n            position: (8.0, 0.0)\n            size: (12.0, 8.0)\n            visible: true\n            layer: render-only\n        }\n        Target target_a {\n            position: (4.0, 2.0)\n            size: (1.0, 1.0)\n        }\n        Target target_b {\n            position: (4.0, 3.25)\n            size: (1.0, 1.0)\n        }\n    }\n}\n";
    let input = |id: &str, source: &str| PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    };
    compile_age_package_with_scenario(
        &[input("main", &format!("{target}{player}"))],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 2,
            ops,
        },
    )
    .unwrap_or_else(|ds| {
        panic!(
            "front door rejected the use-instance source: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })
}

/// The instance facts: the package face compiles msg handlers, the Player's
/// scene carries two `Target` instance rows resolved to the declaring node,
/// and the per-instance state functions are reachable by name.
#[test]
fn use_instance_facts_compile_and_resolve() {
    let t0 = Instant::now();
    let compiled = compile_with(Vec::new());
    let components = &compiled.description.components;
    assert_eq!(
        components.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        vec!["Target", "Player"],
        "both nodes are components in declaration order"
    );
    let player = components
        .iter()
        .find(|c| c.name == "Player")
        .expect("Player component");
    assert_eq!(player.scene.len(), 3, "ground + two Target instances");
    let a = &player.scene[1];
    let b = &player.scene[2];
    assert_eq!(a.kind, "Target");
    assert_eq!(a.name, "target_a");
    // Per-instance cpu geometry for the host rows.
    let floats = |row: &ace_core::source::age_package::AgeSceneRow, key: &str| -> Option<Vec<f32>> {
        row.cpu.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
            if let ace_core::source::age_package::AgeSceneValue::Floats(f) = v {
                Some(f.clone())
            } else {
                None
            }
        })
    };
    assert_eq!(floats(a, "position").expect("a position"), vec![4.0, 2.0]);
    assert_eq!(floats(b, "position").expect("b position"), vec![4.0, 3.25]);
    assert_eq!(floats(a, "size").expect("a size"), vec![1.0, 1.0]);
    // The handler statement face: Touch assigns hits.
    let target = components.iter().find(|c| c.name == "Target").expect("Target");
    assert!(
        target
            .handler_msgs
            .iter()
            .any(|m| m == "Touch"),
        "the Touch handler is registered"
    );
    // Per-instance state functions resolve. The Target module's node is its
    // local node 0, so the frozen compat spelling applies:
    // `__age_state_<inst>_hits` (verified against the registered symbol table).
    for inst in [0usize, 1] {
        let name = format!("__age_state_{inst}_hits");
        assert!(
            compiled.symbols.contains_key(&name),
            "the compat state symbol {name} is registered"
        );
        assert!(!compiled.symbol_of(&name).expect("symbol").is_empty());
    }
    println!(
        "PLAN058_TIMING test=use_instance_facts_compile_and_resolve ms={}",
        t0.elapsed().as_millis()
    );
}

/// V-S03 slice: two Touch events on target A and one on target B leave the
/// per-instance states at 2/1 — same definition, independent state.
#[test]
fn two_instances_keep_2_1_event_state_isolation() {
    let t0 = Instant::now();
    let touch = |instance: usize| ScenarioOp::ForNode {
        node: 0,
        op: Box::new(ScenarioOp::Send {
            instance,
            msg: "Touch".into(),
        }),
    };
    let compiled = compile_with(vec![touch(0), touch(0), touch(1)]);
    let hits = |inst: usize| -> i64 {
        let name = format!("__age_state_{inst}_hits");
        let symbol = compiled
            .symbol_of(&name)
            .unwrap_or_else(|| panic!("no state symbol for {name}"));
        let session = ace_av::age_gfx::AgeGfxSession::new(Box::new(NoGfx));
        let mut session = session;
        session.begin_tick();
        match session
            .run_def(&compiled.checked, &format!("d_{symbol}"))
            .expect("state replay")
        {
            ace_av_core::contract::Value::I32(v) => i64::from(v),
            other => panic!("unexpected state value {other:?}"),
        }
    };
    assert_eq!(hits(0), 2, "target A took two Touch events");
    assert_eq!(hits(1), 1, "target B took one Touch event");
    println!(
        "PLAN058_TIMING test=two_instances_keep_2_1_event_state_isolation ms={}",
        t0.elapsed().as_millis()
    );
}

struct NoGfx;

impl ace_av::age_gfx::AgeGfxHost for NoGfx {
    fn dispatch(
        &mut self,
        symbol: &str,
        _args: &[ace_av_core::contract::Value],
    ) -> Result<ace_av::age_gfx::AgeGfxReturn, ace_av::age_gfx::AgeGfxFault> {
        Err(ace_av::age_gfx::AgeGfxFault::new(
            "unknown-op",
            format!("`{symbol}`: the 2:1 isolation slice runs no graphics ops"),
        ))
    }
}

/// The MAIN topdown fixture's Target instances: the same 2:1 isolation runs on
/// the minimum chain's source — the Tick business (Player, node 1) and the
/// per-instance Touch states (Target, node 0) coexist in one checked package.
#[test]
fn main_fixture_target_instances_keep_2_1_isolation() {
    let t0 = Instant::now();
    let source = include_str!("../fixtures/plan058/topdown-node/main.at");
    let touch = |instance: usize| ScenarioOp::ForNode {
        node: 0,
        op: Box::new(ScenarioOp::Send {
            instance,
            msg: "Touch".into(),
        }),
    };
    let input = PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: source.into(),
        provider: None,
    };
    let compiled = compile_age_package_with_scenario(
        &[input],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 2,
            ops: vec![
                touch(0),
                touch(0),
                touch(1),
                ScenarioOp::ForNode {
                    node: 1,
                    op: Box::new(ScenarioOp::Tick { instance: 0 }),
                },
            ],
        },
    )
    .expect("the main fixture compiles with the 2:1 + Tick scenario");
    let hits = |inst: usize| -> i64 {
        let name = format!("__age_state_{inst}_hits");
        let symbol = compiled
            .symbol_of(&name)
            .unwrap_or_else(|| panic!("no state symbol for {name}"));
        let mut session = ace_av::age_gfx::AgeGfxSession::new(Box::new(NoGfx));
        session.begin_tick();
        match session
            .run_def(&compiled.checked, &format!("d_{symbol}"))
            .expect("state replay")
        {
            ace_av_core::contract::Value::I32(v) => i64::from(v),
            other => panic!("unexpected state value {other:?}"),
        }
    };
    assert_eq!(hits(0), 2, "target A took two Touch events");
    assert_eq!(hits(1), 1, "target B took one Touch event");
    println!(
        "PLAN058_TIMING test=main_fixture_target_instances_keep_2_1_isolation ms={}",
        t0.elapsed().as_millis()
    );
}
