//! PLAN-109 source generation and relocated package admission.
use std::{fs, path::Path, time::Instant};

use auto_project_model::GenerationRequest;
use auto_ui_tools::{
    tool_ops::{generate_report_with, GenerateOptions},
    vue::VueProject,
};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const PACKAGE_FILES: &[&str] = &[
    "package.at",
    "pac.at",
    "collection_view.at",
    "collection_view_util.at",
    "thumbnail.at",
    "thumbnail_content.at",
];

#[test]
fn collection_and_thumbnail_generate_separately() {
    let start = Instant::now();
    for (file, widget) in [
        ("collection_view.at", "CollectionView"),
        ("thumbnail.at", "Thumbnail"),
    ] {
        let path = Path::new(ROOT).join("packages/collection-view").join(file);
        let path = path.to_str().unwrap();
        let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None).unwrap();
        assert!(rust.contains(widget), "{file} missing in Rust output");
        let vue = auto_ui_compiler::pipeline::ui_build(path, "ui", "vue", None).unwrap();
        assert!(vue.contains("defineProps"), "{file} missing in Vue output");
    }
    eprintln!(
        "collection_and_thumbnail_generate_separately {:?}",
        start.elapsed()
    );
}

#[test]
fn relocated_consumer_requires_the_collection_package() {
    let start = Instant::now();
    let mut scratch = tempfile::Builder::new();
    scratch.prefix("collection-view-generation-");
    let dir = match std::env::var_os("PLAN109_ARTIFACT_ROOT") {
        Some(root) => scratch.tempdir_in(root),
        None => scratch.tempdir(),
    }
    .unwrap();
    let app = dir.path().join("relocated-app");
    let package = app.join("vendor/collection-view");
    let front = app.join("src/front");
    fs::create_dir_all(&package).unwrap();
    fs::create_dir_all(&front).unwrap();
    for name in PACKAGE_FILES {
        fs::copy(
            Path::new(ROOT).join("packages/collection-view").join(name),
            package.join(name),
        )
        .unwrap();
    }

    let descriptor = app.join("pac.at");
    fs::write(
        &descriptor,
        "name: \"relocated-collection\"\nscene: \"ui\"\nrender: \"vue\"\n\
         dep collection_view { path: \"vendor/collection-view\" }\n",
    )
    .unwrap();
    fs::write(
        front.join("app.at"),
        r#"
widget App {
    use { package: collection_view from "../../vendor/collection-view" }
    msg { Selected(str) }
    model { var selection str = "" }
    view {
        col {
            Thumbnail (alt: "Independent thumbnail") {}
            CollectionView (onselectionRequested: .Selected($event)) {}
            text .selection
        }
    }
    on { .Selected(value str) -> { .selection = value } }
}
"#,
    )
    .unwrap();

    let request =
        GenerationRequest::from_descriptor(&auto_project_model::parse_file(&descriptor).unwrap());
    let mut project = VueProject::from_workspace_with_request(&app, &request).unwrap();
    project.output_dir = app.join("gen/front/vue");
    project.generate().expect("relocated package must generate");
    let output = project.output_dir.join("src");
    let app_vue = fs::read_to_string(output.join("App.vue")).unwrap();
    assert!(app_vue.contains("<Thumbnail") && app_vue.contains("<CollectionView"));
    assert!(app_vue.contains("@selectionRequested=\"Selected($event)\""));
    let collection = fs::read_to_string(output.join("components/CollectionView.vue")).unwrap();
    assert!(
        collection.contains("<Thumbnail"),
        "CollectionView must use the shared Thumbnail"
    );

    fs::rename(&package, dir.path().join("withdrawn-collection-view")).unwrap();
    let options = GenerateOptions {
        strict: Some(true),
        ..Default::default()
    };
    let error = generate_report_with(&request, &options).expect_err("missing dependency must fail");
    assert!(error.to_string().contains("collection-view"), "{error}");
    dir.close().unwrap();
    eprintln!(
        "relocated_consumer_requires_the_collection_package {:?}",
        start.elapsed()
    );
}
