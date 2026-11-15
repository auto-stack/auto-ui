//! PLAN-085 T-03 — `onkeydown.*` coverage for widget key ownership and for the
//! remaining external-package dispatch gap.
//!
//! Two independent surfaces are pinned here:
//!
//! **A. Key ownership (fixed).** `key_bindings` is a flat
//! `normalized-key -> "Widget.Handler"` map. The child half used to come from
//! `registry.all()`, which iterates a `HashMap` over the whole module —
//! imports included — so a merely-imported component the view never renders
//! could win a key from the live page (the key then did nothing, with no error
//! and no diagnostic), and the winner could differ between runs. T-03 scans
//! only components the view can mount, in first-appearance order.
//!
//! **B. External-package dispatch gap (open, AC-03).** A child declared in the
//! *same file* dispatches its `onkeydown.*` handler; a byte-equivalent child
//! that arrives through a package `use` collects the same binding but the
//! handler never runs. `plan085_external_package_child_keydown_dispatch_gap`
//! records the current behaviour explicitly so it cannot regress silently, and
//! `plan085_same_file_child_keydown_dispatches` is its required control.
//!
//! Fixtures: `tests/fixtures/plan085_keydown/`. Routed pages live in
//! `src/front/pages/` — a page placed in `src/front/` is silently not
//! registered, which is how the first T-03 attempt mis-read this surface.

use std::path::Path;

fn fixture(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/plan085_keydown")
        .join(rel)
}

fn build(rel: &str) -> auto_ui_runtime::ui::dynamic::DynamicComponent {
    let path = fixture(rel);
    let code = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let fixture_str = path.to_string_lossy().to_string();
    auto_ui_runtime::build_dynamic_component(&code, Some(fixture_str.as_str()))
        .expect("fixture component builds")
}

// ── A. Key ownership ────────────────────────────────────────────────────────

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
        "page's own onkeydown.down must stay bound; the unrendered import must not interfere"
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

// ── B. Dispatch: same-file control vs external-package gap ──────────────────

fn dispatch_and_read(rel: &str, widget: &str, event: &str, field: &str) -> i64 {
    let mut comp = build(rel);
    // Render once so child instances mount.
    let (_, _, _) = comp.view_with_debug_gated(false);
    comp.on_with_input_for(widget, event, None);
    match comp.bridge().read_state(field) {
        Ok(auto_val::Value::Int(v)) => i64::from(v),
        other => panic!("{field} unreadable after dispatch: {other:?}"),
    }
}

/// **Control (must stay green):** a same-file child's `onkeydown` handler runs.
#[test]
fn plan085_same_file_child_keydown_dispatches() {
    let hits = dispatch_and_read("src/front/samefile_host.at", "SoloChild", "Down", "hits");
    assert_eq!(
        hits, 1,
        "a same-file child's onkeydown must dispatch to that child's handler"
    );
}

/// **Open gap (AC-03):** a byte-equivalent child arriving through a package
/// `use` collects the binding but its handler does not run.
///
/// This asserts the CURRENT, incorrect behaviour so the gap is explicit and
/// cannot silently change in either direction. When the routing fix lands this
/// assertion must flip to `1` together with the V-03 keyboard evidence.
#[test]
fn plan085_external_package_child_keydown_dispatch_gap() {
    let comp = build("src/front/pkg_host.at");
    let bindings = comp.key_bindings();
    assert_eq!(
        bindings.get("ArrowLeft").map(String::as_str),
        Some("ChildBox.Left"),
        "precondition: the external package child's key IS collected"
    );

    let hits = dispatch_and_read("src/front/pkg_host.at", "ChildBox", "Left", "hits");
    assert_eq!(
        hits, 0,
        "current known gap: an external-package child's onkeydown is collected \
         but never dispatched (AC-03 open). If this now returns 1, the routing \
         fix landed — flip this assertion and update the Plan receipt."
    );
}
