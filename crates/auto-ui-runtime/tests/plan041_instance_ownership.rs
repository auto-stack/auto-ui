//! PLAN-041 — component & store instance ownership (T-01 bounded controls).
//!
//! Fixture: `tests/fixtures/plan041_instances/` — two same-type keyed Counter
//! children (start 10 / 20) under P041Host, sharing P041Store.
//!
//! T-01 scope: prove the oracle discriminates BEFORE implementation.
//! - Control leg (green today, must stay green after T-02): build + render +
//!   host shell intact + store fields merged/readable.
//! - Isolation leg (target semantics, `#[ignore]` = T-02 flips it): per-
//!   instance props/model. Run with `cargo test -p auto-ui-runtime --test
//!   plan041_instance_ownership -- --ignored` to replay the pre-fix failure
//!   (T-01 red evidence).

use auto_ui_runtime::ui::dynamic::DynamicComponent;
use std::path::Path;

fn build_fixture() -> DynamicComponent {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/plan041_instances/src/front/app.at");
    let code = std::fs::read_to_string(&fixture).expect("fixture app.at readable");
    let fixture_str = fixture.to_string_lossy().to_string();
    auto_ui_runtime::build_dynamic_component(&code, Some(fixture_str.as_str()))
        .expect("fixture component builds")
}

fn render_debug(comp: &mut DynamicComponent) -> String {
    let (view, _, _) = comp.view_with_debug_gated(false);
    format!("{view:?}")
}

/// Control leg (green before and after the instance split): the fixture
/// builds, both Counter call sites instantiate under the intact host shell,
/// and the store's merged fields are reachable through the root state read.
#[test]
fn t01_control_fixture_builds_and_renders_both_children() {
    let mut comp = build_fixture();
    let debug = render_debug(&mut comp);

    assert!(
        debug.contains("host above counters") && debug.contains("host below counters"),
        "host shell must stay intact: {}",
        snip(&debug)
    );
    // Both same-type call sites instantiated: the two distinct `start`
    // initial seeds (10 last-write-wins over 20 pre-fix; both present
    // post-fix) — at minimum one counter's numeric text node renders.
    assert!(
        debug.contains("content: \"20\"") || debug.contains("content: \"10\""),
        "counter start text must render: {}",
        snip(&debug)
    );
    // Store contract (Plan 370 D-GAP-4): store model fields merge into the
    // root state and read through the unified state.
    let total = comp
        .bridge()
        .read_state("total")
        .expect("store field `total` merged into root state");
    assert_eq!(total, auto_val::Value::Int(0), "initial store total");
}

/// Isolation leg — TARGET semantics. Ignored at T-01 (fails on the pre-fix
/// merged-state reality: both counters render start=20 and one Inc moves
/// both). T-02 removes the ignore and makes this the permanent AC-01 oracle.
#[test]
#[ignore = "T-01 baseline probe: asserts post-T-02 instance semantics; flips in T-02"]
fn t02_keyed_same_type_children_own_props_model_and_share_store() {
    let mut comp = build_fixture();
    let before = render_debug(&mut comp);

    // Prop ownership: A renders its start=10, B renders start=20.
    assert!(
        before.contains("content: \"10\"") && before.contains("content: \"20\""),
        "keyed same-type children must render their own props (want 10 and 20): {}",
        snip(&before)
    );

    // One Inc on the Counter family must move ONE instance (A) and the
    // shared store, leaving B's model untouched.
    comp.call_widget_handler("Counter", "Inc", &[]).expect("Inc dispatches");
    let after = render_debug(&mut comp);

    let ones = after.matches("content: \"1\"").count();
    assert_eq!(
        ones, 2,
        "after one Inc exactly A.count=1 and shared store total=1 render (B.count stays 0): {}",
        snip(&after)
    );
    assert!(
        after.contains("content: \"0\""),
        "B.count must stay 0 (instance isolation): {}",
        snip(&after)
    );
    assert!(
        after.contains("content: \"10\"") && after.contains("content: \"20\""),
        "starts must stay per-instance after dispatch: {}",
        snip(&after)
    );
}

fn snip(s: &str) -> String {
    let mut out = s.chars().take(1600).collect::<String>();
    if out.len() < s.len() {
        out.push('…');
    }
    out
}
