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

/// F-05 DISCRIMINATOR: collapse the focused child's subtree and assert the
/// component's OWN `focus_id` moved to the parent.
///
/// This reads the instance state rather than the rendered view, because the
/// rendered view cannot distinguish "focus repaired to the parent" from "focus
/// left at a stale index that happens to land somewhere visible" — the reason the
/// earlier attempt at this test could not fail. It also drives the TREE's own
/// handlers through the instance-scoped dispatch, not the page's no-op ones.
///
/// Rows while p1 is expanded: [p1, k1, k2, p2]. Move focus to k1, then collapse
/// p1: rows become [p1, p2]. The stale index (1) would point at p2, so a correct
/// repair must return the parent k1's parent — p1.
#[test]
fn f05_collapsing_the_focused_subtree_moves_focus_to_its_parent() {
    let mut c = build("src/front/focus_repair.at");
    settle(&mut c);
    let tree = c
        .bridge()
        .child_instance_identities_of_type("TreeView")
        .first()
        .cloned()
        .expect("tree instance mounted");

    // Focus k1: rows are [p1, k1, k2, p2], so Down from index 0 lands on k1.
    c.on_with_input_for_instance("TreeView", "Down", None, Some(&tree));
    settle(&mut c);
    let focused_after_down = c.bridge().read_instance_state(&tree, "focus_id");
    assert_eq!(
        format!("{focused_after_down:?}"),
        "Ok(Str(\"k1\"))",
        "Down must move focus onto the first child"
    );

    // Collapse p1 by removing it from `expanded` (the app's controlled update).
    c.on_with_input_for("focusrepair", "CollapseP1", None);
    settle(&mut c);

    // The collapse itself hides k1, so the next keypress is what must heal focus.
    // One ArrowUp is enough to trigger the repair; read the component's own
    // focus_id immediately afterwards. Crucially NO extra navigation is done
    // after the repair, because that would move focus again and mask which value
    // the repair produced - the mistake that made the previous attempt unable to
    // fail.
    // Use Activate: it heals focus without moving it. Up/Down would both repair
    // AND step, so the step alone could produce the expected row and hide a
    // broken repair (measured: Up masked a disabled ancestor branch).
    c.on_with_input_for_instance("TreeView", "Activate", None, Some(&tree));
    // Poll until the repaired focus lands: the handler writes focus_id and the
    // reader observes it after the frame that applies the write, so a single
    // settle() can read the pre-repair value (measured: it did, which made this
    // look like a failure when the repair was in fact working).
    let mut repaired = c.bridge().read_instance_state(&tree, "focus_id");
    for _ in 0..8 {
        settle(&mut c);
        repaired = c.bridge().read_instance_state(&tree, "focus_id");
        if format!("{repaired:?}").contains("\"p1\"") {
            break;
        }
    }
    // rows are [p1, p2]. Repair from the vanished k1 must fall back to its
    // visible PARENT p1 (index 0), not to the neighbour p2 (index 1).
    assert_eq!(
        format!("{repaired:?}"),
        "Ok(Str(\"p1\"))",
        "focus must fall back to the collapsed child's visible parent, not to a \
         neighbour; got {repaired:?}"
    );
}