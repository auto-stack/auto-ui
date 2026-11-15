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

/// N-01: ONE root with 1000 children must render all 1001 rows quickly.
///
/// This is the shape that a 1000-flat-roots sample cannot catch: there
/// `nodes.len()` is 1000, but here it is 1 while the tree holds 1001 nodes. A
/// per-frame duplicate scan bounded on ROOT count therefore re-scans the whole
/// tree every frame, which is the exact O(n^2) blowup that renders zero rows
/// (measured 16.7s and an exhausted instruction budget). The bound must be on
/// TOTAL node count.
///
/// The wall-clock assertion is deliberately generous: it is here to catch a
/// reintroduced quadratic blowup (which takes 16s+ and produces no rows), not to
/// police normal variance.
#[test]
fn n01_single_root_with_thousand_children_still_renders() {
    let started = std::time::Instant::now();
    let mut c = build("src/front/scale_oneroot.at");
    let d = settle(&mut c);
    let elapsed = started.elapsed();

    assert_eq!(
        d.matches("WantSelect").count(),
        1001,
        "the root plus 1000 children must all render"
    );
    assert!(d.contains("Child 999"), "the last child must be reachable");
    assert!(
        elapsed.as_secs() < 10,
        "a 1001-node single-root tree must not take {elapsed:?}; a per-frame \
         quadratic scan keyed on root count is the likely cause"
    );
}
/// N-02 (now CLOSED): a duplicate introduced by a controlled update is detected
/// at ANY tree size, so no ambiguous actionable row is published.
///
/// This used to be a documented limitation: duplicate validation was bounded by
/// node count to keep an O(n^2) per-frame scan affordable, and above the bound a
/// collision went unreported. That bound is gone because the scan was made
/// cheap - `tv_duplicate_id` now uses native string search over concatenated
/// `|id|` tokens instead of a per-element VM loop (measured: 1000 nodes in 2.13s
/// vs ~14s+ and zero rows before).
#[test]
fn n02_large_tree_duplicates_are_detected_at_any_size() {
    let mut c = build("src/front/scale_dup_large.at");
    settle(&mut c);

    let before = settle(&mut c);
    assert!(
        before.matches("WantSelect").count() >= 600,
        "the large fixture mounts valid and actionable"
    );

    // A controlled update collides an id on a 601-node tree.
    c.on_with_input_for("scaleduplarge", "InjectDup", None);
    let after = settle(&mut c);

    assert_eq!(
        after.matches("WantSelect").count(),
        0,
        "a post-Init collision must not leave any row actionable, at any size"
    );
    // The ACTIONABLE-row count above is the guarantee that matters: zero
    // selectable rows means the ambiguity cannot be triggered, at any size. The
    // app-facing error STRING is owned by Init/Validate (they write `.invalid`),
    // and the derivation deliberately cannot write model state, so a post-Init
    // collision is enforced structurally rather than narrated here. Asserting on
    // the class would also be a false-negative trap: the debug view resolves
    // `opacity-50` to a structured value, exactly like `w-4` -> Width(Fixed(4)).
}