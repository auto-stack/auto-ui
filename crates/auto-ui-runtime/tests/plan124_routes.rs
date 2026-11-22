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
fn route_sfc_keeps_hyphenated_and_underscored_page_files_distinct() {
    let app = r#"widget App {
    routes {
        "/code-editor" -> use code-editor
        "/code_editor" -> use code_editor
    }
    view {
        col { outlet }
    }
}
"#;
    let page = |label: &str| format!(r#"fn seed() int {{ return 1 }}
widget Page {{
    model {{ count int = seed() }}
    view {{ text "{label}" {{}} }}
}}
"#);
    let units = [
        unit("app", app),
        unit("code-editor", &page("Dash page")),
        unit("code_editor", &page("Underscore page")),
    ];
    let product = generate_routed_package_sfcs(&units, "fixture::app").unwrap().unwrap();
    assert_eq!(product.paths, [
        ("/code-editor".into(), "code-editor".into()),
        ("/code_editor".into(), "code_editor".into()),
    ]);
    assert!(product.pages["code-editor"].contains("Dash page"));
    assert!(product.pages["code_editor"].contains("Underscore page"));
    let compiled = ace_core::source::age_package::compile_age_package(
        &units, "fixture::app", ace_av::age_component::LegacyWidgetAlias::Accept,
        &[ace_core::source::packages::PACKAGES_CAPABILITY.into()],
    ).unwrap();
    assert_ne!(
        compiled.symbol_of_in("fixture::code-editor", "seed").unwrap(),
        compiled.symbol_of_in("fixture::code_editor", "seed").unwrap(),
    );
}

#[test]
fn route_sfc_publishes_pages_and_local_diagnostics() {
    let product = generate_routed_package_sfcs(&reader(), "fixture::app")
        .expect("routed product")
        .expect("routes were projected");
    assert!(product.app.contains("<RouterView"), "{}", product.app);
    assert!(product.app.contains(r#"<RouterLink to="/book/17">"#), "{}", product.app);
    assert!(product.app.contains(r#"<RouterLink to="/settings">"#), "{}", product.app);
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
fn p123_literal_route_uses_registered_vue_widgets() {
    let units = [
        unit("app", r#"widget App {
    routes {
        "/" -> use gallery
        "/live" -> use live
    }
    view { col { outlet } }
}"#),
        unit("gallery", r#"widget Gallery {
    model { unused int = 0 }
    view {
        col {
            hover_card {
                hover-card-trigger { button "Hover" {} }
                hover_card_content { text "Details" {} }
            }
            alert_dialog {
                alert_dialog_trigger { button "Open" {} }
                alert_dialog_content { alert_dialog_action "Continue" }
            }
            link (text: "Next", href: "/live") {}
        }
    }
}"#),
        unit("live", r#"widget Live {
    msg { Increment }
    model { count int = 0 }
    on { .Increment -> { .count = .count + 1 } }
    view {
        col {
            button "Add" { onclick: .Increment }
            text .count {}
        }
    }
}"#),
    ];
    let product = generate_routed_package_sfcs(&units, "fixture::app").unwrap().unwrap();
    let gallery = &product.pages["gallery"];
    for expected in [
        "from '@/components/ui/hover-card'",
        "from '@/components/ui/alert-dialog'",
        "<HoverCardTrigger",
        "<HoverCardContent",
        "<AlertDialogAction",
        r#"<router-link to="/live""#,
    ] {
        assert!(gallery.contains(expected), "missing {expected}: {gallery}");
    }
    assert!(!gallery.contains("<hover_card"), "{gallery}");
    let live = &product.pages["live"];
    assert!(live.contains("onIncrement") && live.contains("count.value"), "{live}");
}

#[test]
fn native_front_routes_mount_the_checked_page_in_the_authored_outlet() {
    use auto_ui_runtime::ui::{
        ace_node_desktop::{host_session_stats, install_bridge, with_bridge, AceNodeDesktopWindow, NodeDesktopMsg},
        iced::ComponentIced,
        Component, View,
    };

    fn button(view: &View<NodeDesktopMsg>, name: &str) -> Option<NodeDesktopMsg> {
        match view {
            View::Button { label, onclick, .. } if label == name => Some(onclick.clone()),
            View::Column { children, .. } | View::Row { children, .. } => {
                children.iter().find_map(|child| button(child, name))
            }
            View::Scrollable { child, .. } => button(child, name),
            _ => None,
        }
    }

    assert!(std::env::var_os("ACE_NODE_DESKTOP_PREPARED_RUSTC").is_some());
    let dir = tempfile::tempdir().unwrap();
    let front = dir.path();
    let pages = front.join("pages/nested");
    let components = front.join("components");
    std::fs::create_dir_all(&pages).unwrap();
    std::fs::create_dir_all(&components).unwrap();
    let app = r#"widget Gallery {
    use { package: official from "./components" }
    routes {
        "/" -> use intro
        "/tree-page" -> use tree-page
    }
    model { showNavigation bool = true }
    view {
        col (style: "h-screen overflow-hidden") {
            TopBanner {}
            col (style: "md:hidden") {
                text "Mobile only"
            }
            row (style: "flex-1 overflow-hidden") {
                aside (style: "hidden md:flex w-64 flex-col shrink-0") {
                    if .showNavigation {
                        sidebar_menu_button (to: "/tree-page") { text "Open tree" }
                    }
                }
                main (style: "flex-1 overflow-hidden flex flex-col") {
                    sidebar_provider (class: "w-full min-h-0 flex-1 flex-col") {
                        sidebar_content (style: "h-full") {
                            col {
                                text "Before outlet"
                                outlet (memo: true)
                                text "After outlet"
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;
    let app_path = front.join("app.at");
    std::fs::write(&app_path, app).unwrap();
    std::fs::write(
        components.join("package.at"), "name: \"official\"\nversion: \"0.1.0\"\n",
    ).unwrap();
    std::fs::write(
        components.join("banner.at"), "widget TopBanner { view { text \"Gallery banner\" {} } }\n",
    ).unwrap();
    std::fs::write(
        components.join("leaf.at"), "widget TreeLeaf { view { text \"Authored tree leaf\" {} } }\n",
    ).unwrap();
    std::fs::write(pages.join("intro.at"), "widget Intro { view { text \"Original intro\" {} } }\n").unwrap();
    std::fs::write(pages.join("tree-page.at"), r#"widget TreePage {
    use { package: official from "../../components" }
    msg { Init, Back }
    model { count int = 0 }
    on {
        .Init -> { .count = .count + 1 }
        .Back -> { router.back() }
    }
    view {
        TreeLeaf {}
        text f"Mounts: ${.count}" {}
        button "Go back" { onClick: .Back }
    }
}
"#).unwrap();

    let mut bridge = NodeDesktopBridge::load(app, app_path.to_str().unwrap(), 1).unwrap();
    let shell = bridge.open().unwrap();
    install_bridge(bridge);
    let mut window = AceNodeDesktopWindow::default();
    let render = |window: &AceNodeDesktopWindow| format!("{:?}", window.view());
    let click = |window: &mut AceNodeDesktopWindow, name| {
        window.on(button(&window.view(), name).unwrap());
        with_bridge(|bridge| bridge.step()).unwrap()
    };
    assert!(render(&window).contains("Original intro"));
    let mut layout = iced_test::simulator(window.view_iced());
    assert!(layout.find("Mobile only").is_err());
    assert!(layout.find("Before outlet").unwrap().visible_bounds().is_some());
    layout.click("Open tree").expect("authored desktop sidebar link is visible");
    for message in layout.into_messages() {
        window.on(message);
    }
    with_bridge(|bridge| bridge.step()).unwrap();
    let rendered = render(&window);
    assert!(rendered.contains("Gallery banner"));
    assert!(rendered.contains("Mounts: 1"));
    let before = rendered.find("Before outlet").unwrap();
    let leaf = rendered.find("Authored tree leaf").unwrap();
    let after = rendered.find("After outlet").unwrap();
    assert!(before < leaf && leaf < after);
    let mut layout = iced_test::simulator(window.view_iced());
    let target = layout.find("Authored tree leaf").expect("routed leaf in Iced layout");
    let bounds = target.visible_bounds().expect("routed leaf inside the viewport");
    assert!(bounds.width > 0.0 && bounds.height > 0.0);
    assert!(layout.find("Go back").unwrap().visible_bounds().is_some());
    let back = button(&window.view(), "Go back").unwrap();
    let NodeDesktopMsg::PackageClick { session: page, .. } = back else {
        panic!("expected a checked page click");
    };
    assert_ne!(page, shell);
    click(&mut window, "Go back");
    assert!(render(&window).contains("Original intro"));
    window.on(back);
    assert_eq!(host_session_stats(page).unwrap().late_dropped, 1);
    click(&mut window, "Open tree");
    assert!(render(&window).contains("Mounts: 1"));
    with_bridge(|bridge| bridge.close_session()).unwrap().unwrap();
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
