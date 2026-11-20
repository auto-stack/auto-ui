use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_node_desktop::{push_session_event, NodeDesktopBridge};
use auto_ui_runtime::ui::ace_package_sfc::generate_routed_package_sfcs;

fn unit(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "fixture".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn reader() -> Vec<PackageModuleInput> {
    vec![
        unit("app", include_str!("fixtures/plan124/app.at")),
        unit("bookshelf", include_str!("fixtures/plan124/bookshelf.at")),
        unit(
            "book_detail",
            include_str!("fixtures/plan124/book_detail.at"),
        ),
        unit("settings", include_str!("fixtures/plan124/settings.at")),
        unit("broken", include_str!("fixtures/plan124/broken.at")),
        unit("reading", include_str!("fixtures/plan124/reading.at")),
        unit(
            "chapter_math",
            include_str!("fixtures/plan124/chapter_math.at"),
        ),
    ]
}

#[test]
fn route_sfc_publishes_pages_and_local_diagnostics() {
    let product = generate_routed_package_sfcs(&reader(), "fixture::app")
        .expect("routed product")
        .expect("routes were projected");
    assert!(product.app.contains("<RouterView"), "{}", product.app);
    assert!(product.app.contains("<RouterLink"), "{}", product.app);
    let path = |name: &str| {
        product
            .paths
            .iter()
            .find(|(p, _)| p == name)
            .unwrap()
            .1
            .as_str()
    };
    assert_eq!(path("/"), "bookshelf");
    assert_eq!(
        path("/book/:id"),
        "book_detail",
        "{}",
        product.pages[path("/book/:id")]
    );
    let detail = &product.pages[path("/book/:id")];
    assert!(detail.contains("aceParam(\"id\", \"int\")"), "{detail}");
    assert!(
        detail.contains("onMounted(onInit)") && detail.contains("aceRouter.back()"),
        "{detail}"
    );
    assert!(detail.contains("Book detail page"), "{detail}");
    assert!(product.pages[path("/missing")].contains("module is missing"));
    assert!(product.pages[path("/broken")].contains("module has no page widget"));
    assert!(product.pages[path("/settings")].contains("Settings page"));
    let chapter = &product.pages[path("/book/:id/chapter/:ch")];
    assert!(
        chapter.contains("Chapter page") && chapter.contains("a2x_fn_"),
        "{chapter}"
    );
    assert!(
        chapter.contains("aceParam(\"id\"") && chapter.contains("aceParam(\"ch\""),
        "{chapter}"
    );
}

#[test]
fn av_package_routes_preserve_owner_across_navigation_and_reopen() {
    assert!(
        std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_some(),
        "prepare ui980 and set ACE_NODE_DESKTOP_PREPARED_RUSTC before running the AV route test"
    );
    let mut app =
        NodeDesktopBridge::load_package(reader(), "fixture::app", 1).expect("routed bridge");
    let shell = app.open().expect("shell open");
    let page_id = |app: &NodeDesktopBridge| {
        let observed = app.evidence();
        let route = &observed["routes"];
        assert!(route["error"].is_null(), "{route}");
        route["page"]["session"].as_i64().unwrap() as i32
    };
    let click = |app: &mut NodeDesktopBridge, session, target| {
        push_session_event(session, 0, target);
        app.step();
    };
    assert_eq!(app.evidence()["routes"]["current_path"], "/");
    let bookshelf = page_id(&app);
    click(&mut app, bookshelf, 0);
    assert_eq!(app.evidence()["routes"]["current_path"], "/book/17");
    let detail = page_id(&app);
    click(&mut app, bookshelf, 0);
    click(&mut app, detail, 0);
    let observed = app.evidence();
    let state = observed["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == detail)
        .unwrap();
    assert_eq!(
        state["states"][0],
        serde_json::json!([["book_id", "17"], ["total", "17"]])
    );
    let stale = observed["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == bookshelf)
        .unwrap();
    assert_eq!(stale["late_dropped"], 1);

    click(&mut app, shell, 1);
    assert_eq!(app.evidence()["routes"]["current_path"], "/book/42");
    click(&mut app, detail, 0);
    let next = page_id(&app);
    click(&mut app, next, 1);
    assert_eq!(app.evidence()["routes"]["current_path"], "/book/17");
    assert_ne!(page_id(&app), detail);

    click(&mut app, shell, 3);
    assert!(app.evidence()["routes"]["error"]
        .as_str()
        .unwrap()
        .contains("ace.route.no-match"));
    click(&mut app, shell, 2);
    assert_eq!(app.evidence()["routes"]["current_path"], "/settings");
    click(&mut app, shell, 6);
    let chapter = page_id(&app);
    click(&mut app, chapter, 0);
    let observed = app.evidence();
    let chapter_state = observed["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|session| session["id"] == chapter)
        .unwrap();
    assert_eq!(
        chapter_state["states"][0],
        serde_json::json!([["score", "77"]])
    );
    let old_page = page_id(&app);
    app.close_session().unwrap();
    assert!(app.evidence()["routes"]["current_path"].is_null());
    app.reopen().unwrap();
    assert_eq!(app.evidence()["routes"]["current_path"], "/");
    assert_ne!(page_id(&app), old_page);
    app.close_session().unwrap();
}
