use std::path::Path;

use auto_project_model::{descriptor::SourceInfo, generation_request::GenerationRequest};
use auto_ui_tools::vue::{incremental_compile_changed, VueProject};

#[test]
fn ace_route_project_emits_and_refreshes_the_full_page_set() {
    std::env::set_var("AUTO_UI_ACE_VUE", "1");
    let workspace = tempfile::tempdir().expect("temporary Vue project");
    let root = workspace.path();
    std::fs::write(
        root.join("pac.at"),
        "name: \"route_fixture\"\nrender: \"vue\"\nshadcn: off\n",
    )
    .unwrap();
    let front = root.join("src/front");
    let pages = front.join("pages");
    std::fs::create_dir_all(&pages).unwrap();
    let components = front.join("components");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::write(
        components.join("unrelated.at"),
        r#"widget Unrelated {
    model { count int = 0 }
    on { .Run -> { let next int = 1; count = next } }
    view { button "Unrelated" { onclick: .Run } }
}
"#,
    ).unwrap();
    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../auto-ui-runtime/tests/fixtures/plan124");
    std::fs::copy(fixtures.join("app.at"), front.join("app.at")).unwrap();
    for module in ["bookshelf", "book_detail", "settings", "broken", "reading"] {
        let file = format!("{module}.at");
        std::fs::copy(fixtures.join(&file), pages.join(&file)).unwrap();
    }

    std::fs::copy(
        fixtures.join("chapter_math.at"),
        front.join("chapter_math.at"),
    )
    .unwrap();

    let mut request = GenerationRequest::dynamic(SourceInfo::from_path(root.join("pac.at")));
    request.identity.name = Some("route_fixture".into());
    request.package_assets.npm_deps = Some(auto_val::Value::from(vec![
        "@fixture/engine:file:./engine",
        "@fixture/pkg@^1.0.0",
    ]));
    let project = VueProject::from_workspace_with_request(root, &request)
        .expect("ACE routed production project");
    project.generate().expect("publish the full route product");
    let config = std::fs::read_to_string(project.output_dir.join("tsconfig.json")).unwrap();
    assert!(config.contains("\"allowJs\": true"));
    let package: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(project.output_dir.join("package.json")).unwrap(),
    ).unwrap();
    let local = format!("file:{}", root.join("./engine").to_string_lossy().replace('\\', "/"));
    assert_eq!(package["dependencies"]["@fixture/engine"], local);
    assert_eq!(package["dependencies"]["@fixture/pkg"], "^1.0.0");
    let generated = project.output_dir.join("src");
    let read = |path: &str| std::fs::read_to_string(generated.join(path)).unwrap();
    assert!(read("App.vue").contains("<RouterView"));
    assert!(read("router/index.ts").contains("@/pages/book_detail.vue"));
    assert!(read("pages/reading.vue").contains("a2x_fn_"));
    assert!(read("pages/book_detail.vue").contains("aceRouter.back()"));
    assert!(read("pages/ace-route-error-3.vue").contains("module is missing"));
    assert!(!generated.join("components/Unrelated.vue").exists());

    std::fs::write(
        pages.join("missing.at"),
        "widget Restored { view { text \"Restored page\" } }\n",
    )
    .unwrap();
    assert!(incremental_compile_changed(root, None).unwrap() > 0);
    assert!(read("router/index.ts").contains("@/pages/missing.vue"));
    assert!(read("pages/missing.vue").contains("Restored page"));
    assert_eq!(incremental_compile_changed(root, None).unwrap(), 0);
}

#[test]
fn ace_standalone_project_builds_and_refreshes_its_widget_package() {
    std::env::set_var("AUTO_UI_ACE_VUE", "1");
    let workspace = tempfile::tempdir().expect("temporary Vue project");
    let root = workspace.path();
    let manifest = root.join("pac.at");
    std::fs::write(
        &manifest,
        "name: \"package_fixture\"\nrender: \"vue\"\nshadcn: off\n",
    )
    .unwrap();
    let front = root.join("src/front");
    let components = front.join("components");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::write(
        front.join("app.at"),
        r#"widget App {
    use { package: selection_widget from "./components" }
    msg { Selected(str) }
    model { selected str = "" }
    on { .Selected(payload) -> { .selected = payload } }
    view {
        Picker (onselectionRequested: .Selected($event)) {}
        text f"Selected: ${.selected}" {}
    }
}
"#,
    )
    .unwrap();
    std::fs::write(
        components.join("package.at"),
        "name: \"selection-widget\"\nversion: \"1.0.0\"\n",
    )
    .unwrap();
    let picker = components.join("picker.at");
    let source = r#"widget Picker (revision: int = 0) {
    msg { "selectionRequested"(str) }
    view {
        button "Select" { onclick: .selectionRequested("docs/计划") }
    }
}
"#;
    std::fs::write(&picker, source).unwrap();
    std::fs::write(
        components.join("unrelated.at"),
        "widget Unrelated { view { text \"Unrelated\" } }\n",
    )
    .unwrap();

    let mut request = GenerationRequest::dynamic(SourceInfo::from_path(manifest));
    request.identity.name = Some("package_fixture".into());
    let project = VueProject::from_workspace_with_request(root, &request)
        .expect("checked Vue package project");
    project.generate().expect("publish the package product");
    let generated = project.output_dir.join("src");
    let read_app = || std::fs::read_to_string(generated.join("App.vue")).unwrap();
    let sfc = read_app();
    assert!(sfc.contains("<AcePackageView"));
    let config = || std::fs::read_to_string(project.output_dir.join("tsconfig.json")).unwrap();
    assert!(config().contains("\"allowJs\": true"));
    assert!(sfc.contains("[\"selectionRequested\"]: \"Selected\""));
    assert!(sfc.contains("docs/计划"));
    assert!(!sfc.contains("<RouterView"));
    assert!(!generated.join("components/Unrelated.vue").exists());

    std::fs::write(&picker, source.replace("\"Select\"", "\"Select new\"")).unwrap();
    assert!(incremental_compile_changed(root, None).unwrap() > 0);
    assert!(read_app().contains("Select new"));
    assert!(config().contains("\"allowJs\": true"));
    std::fs::write(project.output_dir.join("tsconfig.json"), config().replace("\"allowJs\": true", "\"allowJs\": false")).unwrap();
    assert_eq!(incremental_compile_changed(root, None).unwrap(), 0);
    assert!(config().contains("\"allowJs\": true"));

    std::fs::write(
        components.join("package.at"),
        "name: \"unrelated-widget\"\nversion: \"1.0.0\"\n",
    )
    .unwrap();
    let error = incremental_compile_changed(root, None)
        .expect_err("the authored package identity is required")
        .to_string();
    assert!(error.contains("ace.package.app"), "{error}");
    assert!(error.contains("ace.widget.package-identity"), "{error}");
    assert!(read_app().contains("Select new"));
}
