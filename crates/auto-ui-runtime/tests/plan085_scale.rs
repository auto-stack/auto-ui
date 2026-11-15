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
/// N-02 (OPEN LIMITATION, not a pass): above the per-frame validation bound, a
/// duplicate id introduced by a controlled update is NOT detected, and the
/// component publishes two actionable rows for the same id.
///
/// This is a genuine limitation rather than a closed finding, and it cannot be
/// closed inside this package:
///   - a full per-frame scan is O(n^2); measured ~35k comparisons/sec on the VM
///     means 1000 nodes costs ~14s and exhausts the instruction budget;
///   - the language has no hash/set to make it O(n), and no sort for adjacency;
///   - the component receives no props-changed callback, and the DSL has no
///     parent->child msg dispatch, so validation cannot be deferred to "only when
///     the data changed".
/// `.Validate` detects it but is only reachable from a Rust host, not from `.at`.
///
/// This test pins the CURRENT behaviour so the gap stays visible and actionable:
/// it fails the day the limitation is fixed, which is the intended signal.
#[test]
fn n02_large_tree_duplicates_are_a_documented_limitation() {
    let mut c = build("src/front/scale_dup_large.at");
    let tree = {
        settle(&mut c);
        c.bridge()
            .child_instance_identities_of_type("TreeView")
            .first()
            .cloned()
            .expect("tree instance mounted")
    };

    let before = settle(&mut c);
    let before_rows = before.matches("WantSelect").count();
    assert!(before_rows >= 600, "the large fixture mounts valid");

    // A controlled update collides an id. Above the bound this is NOT detected.
    c.on_with_input_for("scaleduplarge", "InjectDup", None);
    let after = settle(&mut c);
    let after_rows = after.matches("WantSelect").count();

    assert_eq!(
        after_rows,
        before_rows + 1,
        "documented limitation: the colliding row IS published and stays actionable \
         ({before_rows} -> {after_rows}); if this changes, the limitation was fixed \
         and this test should become a positive assertion"
    );
    assert!(
        !after.contains("树结构非法"),
        "documented limitation: no named report is raised for a >{0}-node tree",
        512
    );

    // NOTE: `.Validate` would detect it, but dispatching it costs a full O(n^2)
    // scan on this 601-node tree (measured ~6s), which would push this case over
    // the 10s budget. The limitation itself is what this test pins; the Validate
    // path is covered by the smaller-tree F-04 tests. `tree` is bound above to
    // document that a Rust host CAN reach it while `.at` cannot.
    let _ = &tree;
}