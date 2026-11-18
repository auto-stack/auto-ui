//! PLAN-058 T-04 prep: the pinned SDK front-door probe for the normal AGE Node
//! source (`fixtures/plan058/topdown-node/main.at`).
//!
//! The driver builds against the pinned `.sdk-source` snapshot (auto-lang
//! `637faa210` overlay). That pin predates `IF-981-GFX-AV-SEAM` and the
//! `D-981-AGE-F32-SURFACE` closure: it has no `AGE_GFX_SERVICE_OPS` and its
//! `age-package` grammar has no f32 annotation arm. This test therefore asserts
//! the *expected red* — the source must be rejected before execution — and acts
//! as a canary: the moment the pin is refreshed to a commit with the seam and
//! f32 surface, this test goes red and forces the T-04 gate to be re-based
//! (refresh the expected state, then wire the AV chain). The pin itself is never
//! modified here.
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use std::time::Instant;

const SOURCE: &str = include_str!("../fixtures/plan058/topdown-node/main.at");

#[test]
fn pinned_sdk_rejects_the_t04_source_expected_red_canary() {
    let t0 = Instant::now();
    let input = PackageModuleInput {
        package: "plan058".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: SOURCE.into(),
        provider: None,
    };
    let result = compile_age_package(
        &[input],
        "plan058::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    );
    match result {
        Ok(_) => panic!(
            "the pinned SDK now accepts the T-04 source: the SDK pin evidently contains \
             IF-981-GFX-AV-SEAM + D-981-AGE-F32-SURFACE. Re-base this canary: record the new \
             pin commit, then wire the T-04 AV chain (do not silently pass)."
        ),
        Err(diagnostics) => {
            assert!(
                !diagnostics.is_empty(),
                "a rejected source must carry at least one named diagnostic"
            );
            let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
            for d in diagnostics.iter().take(6) {
                println!("PIN_RED code={} message={}", d.code, d.message);
            }
            // The pin is older than the seam: the source cannot even reach the
            // service/typing lanes, so at least one diagnostic is reported.
            assert!(
                !codes.is_empty(),
                "expected at least one named diagnostic on the old pin"
            );
            println!(
                "PLAN058_TIMING test=pinned_sdk_rejects_the_t04_source_expected_red_canary ms={} diagnostics={} codes={:?}",
                t0.elapsed().as_millis(),
                diagnostics.len(),
                codes
            );
        }
    }
}

/// The pin's service table must not pretend to have the seam: no `query2d.*`
/// registration and no host-port table. This is the evidence behind the
/// "SDK pin refresh" T-04 todo in t04-prep.md.
#[test]
fn pinned_sdk_has_no_gfx_service_table() {
    let t0 = Instant::now();
    let source = include_str!("../.sdk-source/experimental/ace/crates/core/src/source/age_resources.rs");
    assert!(
        !source.contains("AGE_GFX_SERVICE_OPS"),
        "the pin unexpectedly carries the gfx service table"
    );
    assert!(
        !source.contains("query2d.segment"),
        "the pin unexpectedly registers query2d.segment"
    );
    let grammar = include_str!("../.sdk-source/experimental/ace/grammars/auto-source.at");
    // The f32 annotation arms exist only outside the age-package trim in this
    // pin; the T-04 source needs `f32` annotations inside age-package.
    let annotation_arm =
        "alt(profiles: [numeric, aggregates, scheduler, packages-aggregates-ct]) { f32-kw(field: \"word\") }";
    assert!(
        grammar.contains(annotation_arm),
        "the pin's f32 annotation arm shape changed; re-read t04-prep.md"
    );
    assert!(
        !grammar.contains(
            "alt(profiles: [numeric, aggregates, scheduler, packages-aggregates-ct, age-package]) { f32-kw(field: \"word\") }"
        ),
        "the pin now opens f32 annotations in age-package: the canary above must be re-based"
    );
    println!(
        "PLAN058_TIMING test=pinned_sdk_has_no_gfx_service_table ms={} pin=637faa210 overlay",
        t0.elapsed().as_millis()
    );
}
