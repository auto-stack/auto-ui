use auto_project_model::GenerationRequest;
use std::path::Path;

const GOOD: &str = "widget App { view { text \"ok\" {} } }";

fn write_project(root: &Path, front: &str, source: &str, backend: &str) {
    std::fs::create_dir_all(root.join(front)).expect("front dir");
    std::fs::write(
        root.join("pac.at"),
        format!("name: \"review841\"\nscene: \"ui\"\nrender: \"{backend}\"\n"),
    )
    .expect("pac");
    std::fs::write(root.join(front).join("app.at"), source).expect("app source");
}

fn request(root: &Path) -> GenerationRequest {
    GenerationRequest::from_descriptor(
        &auto_project_model::parse_file(&root.join("pac.at")).expect("request descriptor"),
    )
}

fn generate(root: &Path, backend: &str, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let req = request(root);
    match backend {
        "vue" => {
            let mut project =
                auto_ui_tools::vue::VueProject::from_workspace_with_request(root, &req)?;
            project.output_dir = output.to_path_buf();
            project.generate()
        }
        "jet" => {
            auto_ui_tools::jet::generate_jet_project_with_request(root, Some(output), false, &req)
        }
        "ark" => {
            auto_ui_tools::ark::generate_ark_project_with_request(root, Some(output), false, &req)
        }
        _ => auto_ui_tools::rust_ui::generate_rust_ui_with_request(root, Some(output), false, &req),
    }
}

#[test]
fn request_engineering_generators_require_validation_and_valid_controls() {
    for backend in ["vue", "jet", "ark", "rust"] {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let output = root.join("review-output");
        write_project(root, "src/front", GOOD, backend);
        generate(root, backend, &output)
            .unwrap_or_else(|error| panic!("valid {backend} control failed: {error}"));
        assert!(output.exists(), "valid {backend} produced no output");
    }
}

#[test]
fn request_engineering_generators_reject_domain_families_before_output() {
    let cases = [
        (format!("{GOOD}\n{GOOD}\n"), "duplicate widget declaration"),
        (
            "store Shared { model { count int = 0 } }\nstore Shared { model { count int = 0 } }"
                .into(),
            "duplicate store declaration",
        ),
        (
            "widget App {\n model { count int = 0\n count int = 1 }\n view { text \"ok\" {} }\n}"
                .into(),
            "duplicate model field",
        ),
        (
            "widget App {\n msg { Go\n Go }\n view { text \"ok\" {} }\n}".into(),
            "duplicate message variant",
        ),
    ];
    for (source, expected) in cases {
        for backend in ["vue", "jet", "ark", "rust"] {
            let dir = tempfile::tempdir().expect("tempdir");
            let root = dir.path();
            let output = root.join("review-output");
            write_project(root, "src/front", &source, backend);
            let error = generate(root, backend, &output)
                .expect_err("invalid domain must fail the real engineering entry");
            let message = error.to_string();
            assert!(message.contains("UI domain validation failed"), "{message}");
            assert!(message.contains(expected), "{message}");
            assert!(!output.exists(), "{backend} wrote output for {expected}");
        }
    }
}

#[test]
fn request_engineering_generators_validate_imported_modules() {
    for backend in ["vue", "jet", "ark", "rust"] {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let output = root.join("review-output");
        write_project(
            root,
            "src/front",
            "use child: Imported\nwidget App { view { Imported {} } }",
            backend,
        );
        let child = "widget Imported { view { text \"child\" {} } }";
        std::fs::write(root.join("src/front/child.at"), format!("{child}\n{child}"))
            .expect("invalid child");
        let error = generate(root, backend, &output)
            .expect_err("invalid imported module must fail generation");
        assert!(error.to_string().contains("duplicate widget declaration"));
        assert!(!output.exists());
    }
}

#[test]
fn incremental_entries_validate_before_cache_or_output() {
    let jet_dir = tempfile::tempdir().expect("jet tempdir");
    let jet_root = jet_dir.path();
    write_project(jet_root, "source/front", &format!("{GOOD}\n{GOOD}"), "jet");
    let jet_req = request(jet_root);
    let error =
        auto_ui_tools::jet::JetProject::from_workspace_incremental_with_request(jet_root, &jet_req)
            .err()
            .expect("Jet incremental entry must reject invalid source");
    assert!(error.to_string().contains("duplicate widget declaration"));
    assert!(!jet_root.join("gen/front/jet").exists());

    let vue_dir = tempfile::tempdir().expect("vue tempdir");
    let vue_root = vue_dir.path();
    write_project(vue_root, "src/front", &format!("{GOOD}\n{GOOD}"), "vue");
    let vue_req = request(vue_root);
    let error = auto_ui_tools::vue::incremental_compile_changed(vue_root, Some(&vue_req))
        .expect_err("Vue incremental entry must reject invalid source");
    assert!(error.to_string().contains("duplicate widget declaration"));
    assert!(!vue_root.join("gen/front/vue").exists());
}

#[test]
fn cold_recipe_imports_generate_actual_classes_in_full_and_incremental() {
    for (label, prefix) in [
        ("local", "style chip = \"rounded-full px-4\"\n"),
        ("named", "use styles: chip\n"),
        ("wildcard", "use styles: *\n"),
    ] {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let source = format!(
            "{prefix}widget App {{ view {{ button (style: chip) {{ text \"ok\" {{}} }} }} }}\n"
        );
        write_project(root, "src/front", &source, "vue");
        std::fs::write(
            root.join("src/front/styles.at"),
            "pub style chip = \"rounded-full px-4\"\n",
        )
        .expect("style module");
        let app = root.join("src/front/app.at");
        let req = request(root);

        auto_ui_compiler::design_tokens::recipe::clear_style_recipes();
        auto_ui_compiler::ui_gen::api::validate_ui_file_with_modules(&app)
            .unwrap_or_else(|error| panic!("cold {label} preflight failed: {error}"));

        auto_ui_compiler::design_tokens::recipe::clear_style_recipes();
        let project = auto_ui_tools::vue::VueProject::from_workspace_with_request(root, &req)
            .unwrap_or_else(|error| panic!("cold {label} full generation failed: {error}"));
        assert!(
            project.app_vue_code.contains("rounded-full"),
            "cold {label} full generation did not expand the recipe"
        );

        auto_ui_compiler::design_tokens::recipe::clear_style_recipes();
        auto_ui_tools::vue::incremental_compile_changed(root, Some(&req))
            .unwrap_or_else(|error| panic!("cold {label} incremental failed: {error}"));
        let generated = std::fs::read_to_string(root.join("gen/front/vue/src/App.vue"))
            .expect("incremental App.vue");
        assert!(
            generated.contains("rounded-full"),
            "cold {label} incremental did not expand the recipe"
        );
    }
}

#[test]
fn recipe_preflight_rejects_private_missing_and_stale_registry_cases() {
    let private_dir = tempfile::tempdir().expect("private tempdir");
    let private_root = private_dir.path();
    write_project(
        private_root,
        "src/front",
        "use styles: chip\nwidget App { view { button (style: chip) { text \"ok\" {} } } }",
        "vue",
    );
    std::fs::write(
        private_root.join("src/front/styles.at"),
        "style chip = \"rounded-full\"\n",
    )
    .expect("private recipe");
    auto_ui_compiler::design_tokens::recipe::clear_style_recipes();
    let error = auto_ui_compiler::ui_gen::api::validate_ui_file_with_modules(
        &private_root.join("src/front/app.at"),
    )
    .expect_err("private named recipe import must remain rejected");
    assert!(error.to_string().contains("chip"), "{error}");

    let valid_dir = tempfile::tempdir().expect("valid tempdir");
    let valid_root = valid_dir.path();
    write_project(
        valid_root,
        "src/front",
        "use styles: *\nwidget App { view { button (style: chip) { text \"ok\" {} } } }",
        "vue",
    );
    std::fs::write(
        valid_root.join("src/front/styles.at"),
        "pub style chip = \"rounded-full\"\n",
    )
    .expect("public recipe");
    auto_ui_compiler::design_tokens::recipe::clear_style_recipes();
    auto_ui_compiler::ui_gen::api::validate_ui_file_with_modules(
        &valid_root.join("src/front/app.at"),
    )
    .expect("valid imported recipe initializes the registry");

    let missing_dir = tempfile::tempdir().expect("missing tempdir");
    let missing_root = missing_dir.path();
    write_project(
        missing_root,
        "src/front",
        "widget App { view { button (style: missing) { text \"ok\" {} } } }",
        "vue",
    );
    let error = auto_ui_compiler::ui_gen::api::validate_ui_file_with_modules(
        &missing_root.join("src/front/app.at"),
    )
    .expect_err("a previous file's recipe must not make a missing name valid");
    assert!(error.to_string().contains("undefined variable"), "{error}");
}
