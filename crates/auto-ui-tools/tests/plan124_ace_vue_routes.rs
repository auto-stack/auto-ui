use std::path::Path;

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
    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../auto-ui-runtime/tests/fixtures/plan124");
    std::fs::copy(fixtures.join("app.at"), front.join("app.at")).unwrap();
    for module in ["bookshelf", "book_detail", "settings", "broken"] {
        let file = format!("{module}.at");
        std::fs::copy(fixtures.join(&file), pages.join(&file)).unwrap();
    }

    let project = VueProject::from_workspace(root).expect("ACE routed production project");
    project.generate().expect("publish the full route product");
    let generated = project.output_dir.join("src");
    let read = |path: &str| std::fs::read_to_string(generated.join(path)).unwrap();
    assert!(read("App.vue").contains("<RouterView"));
    assert!(read("router/index.ts").contains("@/pages/book_detail.vue"));
    assert!(read("pages/book_detail.vue").contains("aceRouter.back()"));
    assert!(read("pages/ace-route-error-3.vue").contains("module is missing"));

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
