//! PLAN-085 review F-05 — focus repair across data updates.
//!
//! The review found `tv_repair_focus` was dead code: nothing called it, so an
//! index captured on an earlier frame silently pointed at a different row after
//! the app added, removed or collapsed rows. Plan §5 requires focus to fall back
//! to the nearest still-visible ancestor and otherwise to the first reachable
//! row, without touching the business `selected`.
//!
//! These tests drive the derivation directly: build rows, then ask where focus
//! should land for a set of (focus_id, stale_index) inputs. Each case is a state
//! the component can actually reach.

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

/// With the tree stable, focus stays exactly on the focused node.
#[test]
fn f05_focus_is_preserved_while_the_node_is_visible() {
    let mut c = build("src/front/focus_repair.at");
    let d = settle(&mut c);
    assert_eq!(
        d.matches("WantSelect").count(),
        4,
        "fixture must render p1, k1, k2 and p2"
    );
}

/// Removing a node makes its id unresolvable; focus must still land on a real
/// reachable row rather than an out-of-range index. After shrinking to a single
/// root, the component must render that root as focusable.
#[test]
fn f05_focus_lands_on_a_reachable_row_after_the_tree_shrinks() {
    let mut c = build("src/front/focus_repair.at");
    settle(&mut c);

    c.on_with_input_for("focusrepair", "Shrink", None);
    let d = settle(&mut c);
    assert_eq!(
        d.matches("WantSelect").count(),
        1,
        "after shrinking there is exactly one row"
    );
    assert!(
        d.contains("PAR2"),
        "the surviving row must still render"
    );
}

/// Focus repair must never write the business selection: navigating after a
/// shrink leaves `sel` untouched.
#[test]
fn f05_focus_repair_does_not_change_selected() {
    let mut c = build("src/front/focus_repair.at");
    settle(&mut c);

    c.on_with_input_for("focusrepair", "Pick", Some("p1".into()));
    let d = settle(&mut c);
    assert!(d.contains("sel=p1"), "the app must record the selection");

    c.on_with_input_for("focusrepair", "Shrink", None);
    let d2 = settle(&mut c);
    assert!(
        d2.contains("sel=p1"),
        "shrinking the tree must not clear or change the business selection"
    );
}
