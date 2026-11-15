//! PLAN-085 T-05 — the empty state must follow the row derivation.
//!
//! Regression for a measured VM-level defect: the component used to expose an
//! `is_empty` computed (`tv_count_rows(.nodes, ...) == 0`) and the view gated
//! the empty-state line on it. On the VM track that computed **stays stale**
//! once the app mutates `nodes` — driving the tree 2 rows → empty → 2 rows →
//! empty re-derived `rows` correctly every time (2,0,2,0) while the empty
//! branch never rendered. A tree that is empty from the first frame *did*
//! render it, which is what pinned this as staleness rather than a wiring bug.
//!
//! The fix carries emptiness on the derivation that is proven live:
//! `tv_derive_rows` emits one sentinel row (`empty: true`, empty id, disabled)
//! and the view renders its label as the muted empty text. `is_empty` is gone,
//! so there is a single source.
//!
//! Fixture: `src/front/empty_toggle.at` toggles between populated and empty.

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
    for _ in 0..4 {
        c.poll_frame_pump();
        let _ = c.view_with_debug_gated(false);
    }
    let (v, _, _) = c.view_with_debug_gated(false);
    format!("{v:?}")
}

#[test]
fn plan085_empty_state_appears_when_the_app_empties_the_tree() {
    let mut c = build("src/front/empty_toggle.at");

    // Populated: two rows, no empty-state text.
    let populated = settle(&mut c);
    assert_eq!(
        populated.matches("WantSelect").count(),
        2,
        "the fixture seeds two rows"
    );
    assert_eq!(
        populated.matches("空树无可操作行").count(),
        0,
        "a populated tree must not show the empty state"
    );

    // Emptied at runtime: this is the case that used to silently stay stale.
    c.on_with_input_for("emptytoggle", "SetEmpty", None);
    let empty = settle(&mut c);
    assert_eq!(
        empty.matches("WantSelect").count(),
        0,
        "emptying the tree must remove every actionable row"
    );
    assert_eq!(
        empty.matches("空树无可操作行").count(),
        1,
        "the empty state must appear exactly once after the app empties the tree"
    );

    // Re-populate then empty again: the state must track every transition, not
    // just the first one.
    c.on_with_input_for("emptytoggle", "SetFull", None);
    let again = settle(&mut c);
    assert_eq!(
        again.matches("WantSelect").count(),
        2,
        "re-populating must restore both rows"
    );
    assert_eq!(
        again.matches("空树无可操作行").count(),
        0,
        "the empty state must leave once rows exist again"
    );

    c.on_with_input_for("emptytoggle", "SetEmpty", None);
    let empty2 = settle(&mut c);
    assert_eq!(
        empty2.matches("空树无可操作行").count(),
        1,
        "the empty state must come back on a second transition"
    );
}
