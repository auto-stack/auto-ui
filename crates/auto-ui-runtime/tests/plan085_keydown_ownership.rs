//! PLAN-085 T-03 — `onkeydown.*` key ownership and dispatch for child widgets.
//!
//! ## What was actually wrong
//!
//! `key_bindings` is a flat `normalized-key -> "Widget.Handler"` map whose
//! child half used to come from `registry.all()`. That iterates a `HashMap`
//! over the **whole module**, imports included, so a merely-imported component
//! that the view never renders could win a key from the live page — the key
//! then did nothing at all, with no error and no diagnostic — and the winner
//! could differ between runs. T-03 (commit 815fec34) scans only components the
//! view can mount, in first-appearance order, so ownership is deterministic
//! and unrendered imports cannot shadow live keys.
//!
//! ## Two false leads, recorded so they are not re-derived
//!
//! 1. Routed pages live in `src/front/pages/`. A page placed in `src/front/`
//!    is silently not registered — an early "routed pages are missing from
//!    key collection" reading was this fixture bug, not a runtime defect.
//! 2. A child widget that **declares `Init`** renders a
//!    `Loading… (<Name>)` placeholder until the frame pump services the Init
//!    demand. A view dump taken without pumping therefore contains no rows and
//!    no text — which reads exactly like a failed dispatch. The dispatch
//!    fixtures here declare no `Init` for that reason, and dispatch is
//!    measured by the rendered text changing (see `dispatch_hits` below).
//!
//! External-package children dispatch correctly: `ChildBox` arrives through a
//! package `use`, its `onkeydown.left` is bound to `ChildBox.Left`, and the
//! handler runs (hits 0 → 1). `plan085_package_child_keydown_dispatches` pins
//! that, with the same-file child as the control.

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

fn view_text(comp: &mut auto_ui_runtime::ui::dynamic::DynamicComponent) -> String {
    let (view, _, _) = comp.view_with_debug_gated(false);
    format!("{view:?}")
}

/// The rendered `<marker>N` value, i.e. the child's `hits` as the view shows it.
fn dispatch_hits(comp: &mut auto_ui_runtime::ui::dynamic::DynamicComponent, marker: &str) -> u32 {
    let text = view_text(comp);
    for part in text.split("content: ").skip(1) {
        let body = part.trim_start_matches('"');
        if let Some(rest) = body.strip_prefix(marker) {
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            return digits.parse().unwrap_or(0);
        }
    }
    panic!("marker {marker:?} absent from rendered view: {}", snip(&text));
}

fn snip(s: &str) -> String {
    let mut out: String = s.chars().take(600).collect();
    if out.len() < s.len() {
        out.push('…');
    }
    out
}

// ── Key ownership ───────────────────────────────────────────────────────────

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

/// The regression T-03 fixed: a component that is imported but never rendered
/// must not take key ownership from the page that is on screen.
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

// ── Dispatch ────────────────────────────────────────────────────────────────

/// Control: a same-file child's `onkeydown` handler runs.
#[test]
fn plan085_same_file_child_keydown_dispatches() {
    let mut comp = build("src/front/samefile_diag.at");
    assert_eq!(dispatch_hits(&mut comp, "solo hits="), 0, "baseline");
    comp.on_with_input_for("SoloChildDiag", "Down", None);
    assert_eq!(
        dispatch_hits(&mut comp, "solo hits="),
        1,
        "a same-file child's onkeydown must dispatch to that child's handler"
    );
}

/// An external-package child dispatches exactly like a same-file child. This is
/// the counterpart of the control above: the two differ only in where the
/// widget is declared.
#[test]
fn plan085_package_child_keydown_dispatches() {
    let mut comp = build("src/front/pkg_host_diag.at");

    // The binding must be collected and point at the package widget.
    assert_eq!(
        comp.key_bindings().get("ArrowLeft").map(String::as_str),
        Some("ChildBox.Left"),
        "the package child's onkeydown.left must be bound to the package widget"
    );
    assert_eq!(dispatch_hits(&mut comp, "child box hits="), 0, "baseline");

    comp.on_with_input_for("ChildBox", "Left", None);
    assert_eq!(
        dispatch_hits(&mut comp, "child box hits="),
        1,
        "an external-package child's onkeydown must dispatch (same as a same-file child)"
    );
}
