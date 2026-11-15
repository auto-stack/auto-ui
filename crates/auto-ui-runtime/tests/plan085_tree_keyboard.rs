//! PLAN-085 T-03 — the shipped TreeView's keyboard semantics.
//!
//! The tree's focus is instance state, and the page can only observe what the
//! component reports back. So the strongest available oracle is: **Enter
//! requests selection of the focused row**, therefore the selected ID after a
//! key sequence *is* the row focus reached.
//!
//! Traps this file avoids, each of which made earlier attempts look like total
//! keyboard failure while the keyboard was in fact fine:
//!
//! 1. `on_with_input_for(widget, event)` takes the **msg variant** name, not
//!    the key name. `Enter` is `Activate`, `Home`/`End` are `GoHome`/`GoEnd`,
//!    `Right` is `Expand`. Dispatching `"Enter"` is a no-op that leaves every
//!    observable unchanged.
//! 2. `autoui_state` reports the **page's** fields, so a page that does not
//!    mirror the tree's focus always shows `focus_index: 0`.
//! 3. A child that declares `Init` renders a `Loading… (<Name>)` placeholder
//!    until the frame pump services the demand.
//!
//! Rows here: `p1` (expanded, child `s1`) then `p2` → reachable order
//! p1, s1, p2.

use std::path::Path;

fn build(rel: &str) -> auto_ui_runtime::ui::dynamic::DynamicComponent {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/plan085_keydown")
        .join(rel);
    let code = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let s = path.to_string_lossy().to_string();
    auto_ui_runtime::build_dynamic_component(&code, Some(s.as_str())).expect("builds")
}

fn mounted() -> auto_ui_runtime::ui::dynamic::DynamicComponent {
    let mut comp = build("src/front/tree_kbd_host.at");
    for _ in 0..6 {
        comp.poll_frame_pump();
        let _ = comp.view_with_debug_gated(false);
    }
    comp
}

fn state_str(comp: &auto_ui_runtime::ui::dynamic::DynamicComponent, field: &str) -> String {
    match comp.bridge().read_state(field) {
        Ok(auto_val::Value::Str(s)) => s.to_string(),
        other => panic!("{field} unreadable: {other:?}"),
    }
}

fn state_int(comp: &auto_ui_runtime::ui::dynamic::DynamicComponent, field: &str) -> i32 {
    match comp.bridge().read_state(field) {
        Ok(auto_val::Value::Int(v)) => v,
        other => panic!("{field} unreadable: {other:?}"),
    }
}

/// Dispatch a TreeView msg variant (not a key name — see the module docs).
fn send(comp: &mut auto_ui_runtime::ui::dynamic::DynamicComponent, msg: &str) {
    comp.on_with_input_for("TreeView", msg, None);
}

/// Enter requests selection of whatever row currently holds focus.
fn select_focused(comp: &mut auto_ui_runtime::ui::dynamic::DynamicComponent) -> String {
    send(comp, "Activate");
    state_str(comp, "selected")
}

#[test]
fn plan085_tree_starts_focused_on_first_reachable_row() {
    let mut comp = mounted();
    assert_eq!(
        select_focused(&mut comp),
        "p1",
        "focus must start on the first reachable row"
    );
}

#[test]
fn plan085_tree_down_walks_rows_in_order() {
    let mut comp = mounted();

    send(&mut comp, "Down");
    assert_eq!(
        select_focused(&mut comp),
        "s1",
        "one Down moves from p1 into its child"
    );

    send(&mut comp, "Down");
    assert_eq!(
        select_focused(&mut comp),
        "p2",
        "a second Down reaches the last reachable row"
    );
}

#[test]
fn plan085_tree_navigation_saturates_at_the_ends() {
    let mut comp = mounted();

    send(&mut comp, "GoEnd");
    for _ in 0..4 {
        send(&mut comp, "Down");
    }
    assert_eq!(
        select_focused(&mut comp),
        "p2",
        "Down past the last row must not wrap or move focus elsewhere"
    );

    send(&mut comp, "GoHome");
    for _ in 0..4 {
        send(&mut comp, "Up");
    }
    assert_eq!(
        select_focused(&mut comp),
        "p1",
        "Up past the first row must not wrap or move focus elsewhere"
    );
}

#[test]
fn plan085_tree_home_and_end_jump_to_the_ends() {
    let mut comp = mounted();

    send(&mut comp, "GoEnd");
    assert_eq!(select_focused(&mut comp), "p2", "GoEnd → last row");

    send(&mut comp, "GoHome");
    assert_eq!(select_focused(&mut comp), "p1", "GoHome → first row");
}

/// Expand on an already-open parent descends into its first child instead of
/// asking the app to expand again.
#[test]
fn plan085_tree_expand_on_open_parent_descends_without_retoggling() {
    let mut comp = mounted();
    let before = state_int(&comp, "toggles");

    send(&mut comp, "GoHome");
    send(&mut comp, "Expand");
    assert_eq!(
        state_int(&comp, "toggles"),
        before,
        "Expand on an already-open parent must not request another expansion"
    );
    assert_eq!(
        select_focused(&mut comp),
        "s1",
        "Expand on an open parent moves focus to its first child"
    );
}

/// Enter only ever requests selection — never expansion.
#[test]
fn plan085_tree_enter_does_not_toggle() {
    let mut comp = mounted();
    let before = state_int(&comp, "toggles");
    send(&mut comp, "Activate");
    send(&mut comp, "Activate");
    assert_eq!(
        state_int(&comp, "toggles"),
        before,
        "Enter must only request selection, never expansion"
    );
}

/// A disabled row is shown but skipped by keyboard navigation.
#[test]
fn plan085_tree_skips_disabled_rows() {
    let mut comp = build("src/front/tree_kbd_disabled.at");
    for _ in 0..6 {
        comp.poll_frame_pump();
        let _ = comp.view_with_debug_gated(false);
    }
    // Rows: p1, s1, p2 where p2 is disabled → reachable order p1, s1.
    send(&mut comp, "Down");
    assert_eq!(select_focused(&mut comp), "s1", "Down from p1 reaches s1");

    send(&mut comp, "Down");
    assert_eq!(
        select_focused(&mut comp),
        "s1",
        "Down must not land on the disabled row"
    );

    send(&mut comp, "GoEnd");
    assert_eq!(
        select_focused(&mut comp),
        "s1",
        "GoEnd must stop at the last REACHABLE row, skipping disabled ones"
    );
}

// ── Nested-tree derivation (the 026 consumer shape) ─────────────────────────

/// The package must derive a **nested** tree, not just a flat list: 026 feeds
/// it a recursive `filter_tree` result whose children carry `is_leaf` flags,
/// and expands the root plus a group. Rows must come out as root → group →
/// leaf in that order, with the leaf's own children absent (it is expanded
/// into none).
#[test]
fn plan085_derives_nested_filtered_tree_like_the_026_consumer() {
    let mut comp = build("src/front/db_shape.at");
    for _ in 0..6 {
        comp.poll_frame_pump();
        let _ = comp.view_with_debug_gated(false);
    }
    let (view, _, _) = comp.view_with_debug_gated(false);
    let d = format!("{view:?}");

    // Order matters: the root appears before its expanded group, which appears
    // before the leaf it contains.
    let root = d.find("app (SQLite)").expect("root row must render");
    let group = d.find("Tables").expect("expanded group row must render");
    let leaf = d.find("customers").expect("leaf row under the group must render");
    assert!(
        root < group && group < leaf,
        "rows must derive root → group → leaf (got {root} / {group} / {leaf})"
    );

    // One bound selection handler per rendered row.
    assert_eq!(
        d.matches("WantSelect").count(),
        3,
        "exactly the three visible rows carry a bound selection action"
    );
}