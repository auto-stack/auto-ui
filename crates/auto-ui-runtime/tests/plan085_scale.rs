//! PLAN-085 review F-09 — representative scale.
//!
//! The plan requires the tree to handle a representative sample (1000 nodes and
//! deep nesting) and to keep the last row reachable, as a test sample rather than
//! a new syntax limit.
//!
//! Writing this found a real performance defect: `tv_derive_rows` re-scanned the
//! whole tree for duplicate ids on EVERY frame, which is O(n^2). At 1000 nodes the
//! Init handler died with "instruction budget exhausted (10,000,000 steps)" and the
//! tree rendered zero rows. Structural validity is now computed once on
//! Init/Validate and passed in as `dupGuard`. A dead `reachable` computed (a full
//! tree walk with no consumer) was removed at the same time.

use std::path::Path;

fn build(rel: &str) -> auto_ui_runtime::ui::dynamic::DynamicComponent {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/plan085_keydown")
        .join(rel);
    let code = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let s = path.to_string_lossy().to_string();
    auto_ui_runtime::build_dynamic_component(&code, Some(s.as_str())).expect("builds")
}

fn settle(c: &mut auto_ui_runtime::ui::dynamic::DynamicComponent) -> String {
    for _ in 0..5 {
        c.poll_frame_pump();
        let _ = c.view_with_debug_gated(false);
    }
    let (v, _, _) = c.view_with_debug_gated(false);
    format!("{v:?}")
}

/// 1000 flat nodes must all derive — this is the case that used to exhaust the
/// instruction budget and render nothing.
#[test]
fn f09_thousand_nodes_derive_every_row() {
    let mut c = build("src/front/scale_wide.at");
    let d = settle(&mut c);
    assert_eq!(
        d.matches("WantSelect").count(),
        1000,
        "all 1000 nodes must produce an actionable row"
    );
    assert!(
        d.contains("Node 999"),
        "the last node must be reachable, not truncated"
    );
}

/// 64 levels of nesting must all be visible when expanded, with the deepest row
/// present.
#[test]
fn f09_deep_nesting_renders_the_deepest_row() {
    let mut c = build("src/front/scale_deep.at");
    let d = settle(&mut c);
    assert_eq!(
        d.matches("WantSelect").count(),
        64,
        "64 nested levels must all render"
    );
    assert!(
        d.contains("Deep 63"),
        "the deepest row must be present"
    );
    // Indent is capped so deep nesting does not walk off the pane: depths beyond
    // the last literal branch share one width.
    assert!(
        d.contains("Width(Fixed(5))"),
        "deep rows must use the capped indent width"
    );
}
