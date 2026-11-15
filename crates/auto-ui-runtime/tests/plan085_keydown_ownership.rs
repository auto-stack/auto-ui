//! PLAN-085 T-03 — `onkeydown.*` key ownership must follow what the view can
//! actually mount.
//!
//! `key_bindings` is a flat `normalized-key -> "Widget.Handler"` map built from
//! the root view tree plus child widgets. Before T-03 the child half came from
//! `registry.all()`, which iterates a `HashMap` over the *whole module* —
//! imports included. Two consequences, both silent:
//!
//! 1. **Nondeterministic ownership**: a merely-`use`d component that the view
//!    never renders could land its `onkeydown.*` entry first and take the key
//!    from the live page. The page handler then never ran — no error, no
//!    diagnostic (T-01 P-5 GUI matrix: import-only → `page_hits=0`).
//! 2. **Unstable replay**: the winner could differ between runs.
//!
//! T-03 restricts the scan to components the root view can mount: element tags
//! resolving to a registered widget, plus routed page targets (mounted through
//! `outlet`, so they never appear as a tag), in first-appearance order.
//!
//! Fixtures: `tests/fixtures/plan085_keydown/`. Note routed pages live in
//! `src/front/pages/` — a page in `src/front/` is silently not registered,
//! which is how the first T-03 attempt mis-read this surface.

use std::path::Path;

fn build(rel: &str) -> auto_ui_runtime::ui::dynamic::DynamicComponent {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/plan085_keydown")
        .join(rel);
    let code = std::fs::read_to_string(&fixture).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let fixture_str = fixture.to_string_lossy().to_string();
    auto_ui_runtime::build_dynamic_component(&code, Some(fixture_str.as_str()))
        .expect("fixture component builds")
}

/// Root + routed page: the root keeps its own key, and the page contributes
/// the key the root did not claim.
#[test]
fn plan085_root_and_routed_page_keys_coexist() {
    let comp = build("src/front/app.at");
    let bindings = comp.key_bindings();

    assert_eq!(
        bindings.get("ArrowDown").map(String::as_str),
        Some("App.RootDown"),
        "the root's own onkeydown.down must win its key"
    );
    assert_eq!(
        bindings.get("ArrowUp").map(String::as_str),
        Some("page.Two"),
        "the routed page's non-colliding key must be bound to the page"
    );
}

/// The regression: a component that is imported but never rendered must not
/// take key ownership from the page that is on screen.
#[test]
fn plan085_unrendered_import_does_not_steal_page_keys() {
    let comp = build("src/front/app_import_only.at");
    let bindings = comp.key_bindings();

    assert_eq!(
        bindings.get("ArrowDown").map(String::as_str),
        Some("importonly.One"),
        "page's own onkeydown.down must stay bound; an unrendered import \
         (ChildBox declares left/right only) must not interfere"
    );
    assert_eq!(
        bindings.get("ArrowUp").map(String::as_str),
        Some("importonly.Two"),
        "page's second key must also stay bound"
    );
    // ChildBox is imported but never rendered, so its private keys are absent.
    for key in ["ArrowLeft", "ArrowRight"] {
        assert!(
            !bindings.contains_key(key),
            "{key} belongs to a component this view never renders and must not be bound"
        );
    }
}

/// Control: when the view does render the child, its bindings count.
#[test]
fn plan085_rendered_child_contributes_its_keys() {
    let comp = build("src/front/app_import_render.at");
    let bindings = comp.key_bindings();

    assert_eq!(
        bindings.get("ArrowDown").map(String::as_str),
        Some("importrender.One"),
        "page key must stay bound alongside a rendered child"
    );
    assert_eq!(
        bindings.get("ArrowLeft").map(String::as_str),
        Some("ChildBox.Left"),
        "a rendered child's onkeydown.left must be bound to the child"
    );
    assert_eq!(
        bindings.get("ArrowRight").map(String::as_str),
        Some("ChildBox.Right"),
        "a rendered child's onkeydown.right must be bound to the child"
    );
}
