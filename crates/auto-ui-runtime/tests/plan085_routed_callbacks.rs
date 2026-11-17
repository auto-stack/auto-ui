//! PLAN-085 T-05 — routed-page child callbacks.
//!
//! The product gate drives `examples/tree-view`, whose tree sits in a **routed
//! page**. That shape behaves differently from a tree at the App root, so this
//! test pins what the runtime does at library level, independent of the iced
//! loop:
//!
//! * a package child inside a routed page receives its List prop and derives
//!   its rows (`WantSelect` actions appear), and
//! * pressing such a row reaches the **page** handler, writing the page's own
//!   state.
//!
//! Both hold here. The real window shows the rows but the press does not reach
//! the page (see the decisions record), so this test is the library-side
//! reference the runtime fix must keep green while the iced path is corrected.
//!
//! Fixtures: `src/front/routecrash.at` (App + routes) and
//! `src/front/pages/demo_page.at` (the page hosting the package tree).

use std::path::Path;

fn build(rel: &str) -> auto_ui_runtime::ui::dynamic::DynamicComponent {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/plan085_keydown")
        .join(rel);
    let code = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let s = path.to_string_lossy().to_string();
    auto_ui_runtime::build_dynamic_component(&code, Some(s.as_str())).expect("builds")
}

fn settled(rel: &str) -> auto_ui_runtime::ui::dynamic::DynamicComponent {
    let mut c = build(rel);
    for _ in 0..8 {
        c.poll_frame_pump();
        let _ = c.view_with_debug_gated(false);
    }
    c
}

/// A package child mounted inside a routed page still derives its rows from the
/// List prop the page passes.
#[test]
fn plan085_routed_page_package_child_derives_rows() {
    let mut c = settled("src/front/routecrash.at");
    let ids = c.bridge().child_instance_identities_of_type("TreeView");
    assert_eq!(
        ids.len(),
        1,
        "the routed page must mount exactly one tree: {ids:?}"
    );
    assert!(
        ids[0].starts_with("demo_page@"),
        "the instance must be scoped to the routed page, got {:?}",
        ids[0]
    );

    let (view, _, _) = c.view_with_debug_gated(false);
    let d = format!("{view:?}");
    assert_eq!(
        d.matches("WantSelect").count(),
        2,
        "expanded p1 plus its child must yield two bound rows"
    );
}

/// Pressing a row of a routed page's package child reaches the PAGE handler:
/// the page's own `summary` field changes.
#[test]
fn plan085_routed_page_child_callback_reaches_page_state() {
    let mut c = settled("src/front/routecrash.at");
    let page_id = c
        .bridge()
        .child_instance_identities_of_type("demo_page")
        .first()
        .cloned()
        .expect("page instance mounted");
    let tree_id = c
        .bridge()
        .child_instance_identities_of_type("TreeView")
        .first()
        .cloned()
        .expect("tree instance mounted");

    let before = c.bridge().read_instance_state(&page_id, "summary");
    assert_eq!(
        format!("{before:?}"),
        "Ok(Str(\"sel= exp=1\"))",
        "page must start with the seeded summary"
    );

    c.on_with_input_for_instance("TreeView", "WantSelect", Some("p1".into()), Some(&tree_id));

    let after = c.bridge().read_instance_state(&page_id, "summary");
    assert_eq!(
        format!("{after:?}"),
        "Ok(Str(\"sel=p1\"))",
        "the page handler must run and write the page's own state"
    );
}
