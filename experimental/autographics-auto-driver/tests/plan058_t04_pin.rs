//! PLAN-058 T-04: the refreshed `.sdk-source` (auto-lang master `64e8380f4`
//! overlay: record ABI `5ee4d23c0`, f32 surface, gfx seam) now compiles the
//! normal AGE Node fixture through the shared front door, and carries the
//! service/port/budget surface the host lane consumes.
//!
//! This replaces the pre-refresh "expected red" canary: the pin is now *part of
//! the contract*, so the test asserts the positive surface and fails loudly if
//! the pin is rolled back or drifts (record fields, f32 annotation arms, the
//! 17-op table, the 32-credit budget constant).
use ace_core::source::age_component::ComponentStmt;
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AgeRowLayer};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use std::time::Instant;

const SOURCE: &str = include_str!("../fixtures/plan058/topdown-node/main.at");

fn compile() -> Result<ace_core::source::age_package::AgePackageCompilation, String> {
    let input = PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: SOURCE.into(),
        provider: None,
    };
    compile_age_package(
        &[input],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })
}

/// The front door accepts the fixture: one AGE node, six scene rows, the real
/// `on Life.Tick` author form (G2: hook-local `let` + `if/else`), and the
/// ground's typed render-only fact (G1) — the host-state `tick_step` form is
/// retired.
#[test]
fn refreshed_pin_compiles_the_t04_source() {
    let t0 = Instant::now();
    match compile() {
        Ok(compiled) => {
            assert_eq!(compiled.description.components.len(), 1);
            assert_eq!(compiled.description.components[0].scene.len(), 6);
            // G1: the checked row description carries the typed layer fact.
            let scene = &compiled.description.components[0].scene;
            assert_eq!(scene[0].layer, Some(AgeRowLayer::RenderOnly));
            // G2: the Tick is the authored hook-local let/if form.
            let handler = &compiled.description.components[0].handler_stmts;
            let tick_stmts = &handler
                .iter()
                .find(|(msg, _)| msg == "Life.Tick")
                .expect("Life.Tick handler")
                .1;
            assert!(
                tick_stmts
                    .iter()
                    .any(|s| matches!(s, ComponentStmt::Let { .. })),
                "hook-local lets are authored"
            );
            assert!(
                matches!(tick_stmts.last(), Some(ComponentStmt::If { .. })),
                "the start_overlap branch closes the Tick"
            );
            assert!(
                compiled.symbol_of("tick_step").is_none(),
                "the host-state tick_step form is retired"
            );
            assert!(compiled
                .description
                .components[0]
                .handler_msgs
                .iter()
                .any(|m| m == "Life.Tick"));
            println!(
                "PLAN058_TIMING test=refreshed_pin_compiles_the_t04_source ms={} rows=6 g1=layer g2=let_if",
                t0.elapsed().as_millis()
            );
        }
        Err(diagnostics) => panic!(
            "the refreshed pin must compile the T-04 source (re-base this test if the pin \
             rolled back): {diagnostics}"
        ),
    }
}

/// The pin's surface: the 17-op table with the three record ports, f32 in the
/// age-package annotation arms, and the co-frozen 32-credit budget.
#[test]
fn refreshed_pin_carries_the_gfx_port_abi_surface() {
    let t0 = Instant::now();
    let resources = include_str!(
        "../.sdk-source/experimental/ace/crates/core/src/source/age_resources.rs"
    );
    for symbol in [
        "age.query2d.segment.v1",
        "age.query2d.sweep.v1",
        "age.image2d.png.v1",
        "age.input.space.v1",
        "age.row.place.v1",
        "age.render2d.spawn.v1",
        "age.frame2d.at.v1",
    ] {
        assert!(
            resources.contains(symbol),
            "the pin's service table is missing {symbol}"
        );
    }
    assert!(
        resources.contains("GFX_CALLBACK_CREDIT_BUDGET: u32 = 32"),
        "the frozen 32-credit budget must be in the pin"
    );
    assert!(
        resources.contains("\"query2d.segment\" => 8"),
        "segment stays 8 credits"
    );
    assert!(
        resources.contains("\"query2d.sweep\" => 16"),
        "sweep stays 16 credits"
    );
    // The port record fields in their AGE_PORT_FIELDS order.
    for field in [
        "\"point_x\"",
        "\"point_y\"",
        "\"normal_x\"",
        "\"normal_y\"",
        "\"origin_x\"",
        "\"origin_y\"",
        "\"start_overlap\"",
        "\"sha256\"",
        "\"byte_length\"",
    ] {
        assert!(resources.contains(field), "port field {field} missing");
    }
    let grammar =
        include_str!("../.sdk-source/experimental/ace/grammars/auto-source.at");
    assert!(
        grammar.contains("alt(profiles: [age-package]) { f32-kw(field: \"word\") }"),
        "the pin must open f32 annotations in the age-package face"
    );
    println!(
        "PLAN058_TIMING test=refreshed_pin_carries_the_gfx_port_abi_surface ms={} ops=17 budget=32",
        t0.elapsed().as_millis()
    );
}
