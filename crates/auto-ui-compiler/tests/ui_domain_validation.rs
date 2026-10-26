use auto_ui_compiler::ui_gen::api::{
    generate_component_from_file, transpile_aura, transpile_vue_aura, ComponentGenOptions,
};
use auto_ui_compiler::{lower_ui, parse_ui};

const GOOD: &str = "widget App { view { text \"ok\" {} } }";

#[test]
fn public_generators_reject_invalid_domain_before_writing() {
    let bad = format!("{GOOD}\n{GOOD}\n");
    let parsed = parse_ui(&bad).expect("duplicate declarations are syntactically valid");
    assert!(lower_ui(&parsed)
        .expect_err("lowering rejects duplicate widgets")
        .to_string()
        .contains("duplicate widget declaration"));

    let dir = tempfile::tempdir().expect("tempdir");
    let rust_out = dir.path().join("App.rs");
    let vue_out = dir.path().join("App.vue");
    for result in [
        transpile_aura(&bad, rust_out.to_str()),
        transpile_vue_aura(&bad, vue_out.to_str()),
    ] {
        let error = result.expect_err("public generator must reject invalid domain data");
        assert!(error.contains("UI domain validation failed"), "{error}");
        assert!(error.contains("duplicate widget declaration"), "{error}");
    }
    assert!(!rust_out.exists());
    assert!(!vue_out.exists());
    assert!(transpile_aura(GOOD, None).is_ok());
    assert!(transpile_vue_aura(GOOD, None).is_ok());
}

#[test]
fn file_generation_rejects_invalid_domain_families() {
    let cases = [
        (
            "duplicate widget",
            format!("{GOOD}\n{GOOD}\n"),
            "duplicate widget declaration",
        ),
        (
            "duplicate store",
            "store Shared { model { count int = 0 } }\nstore Shared { model { count int = 0 } }"
                .into(),
            "duplicate store declaration",
        ),
        (
            "duplicate model field",
            "widget App {\n model { count int = 0\n count int = 1 }\n view { text \"ok\" {} }\n}"
                .into(),
            "duplicate model field",
        ),
        (
            "duplicate message variant",
            "widget App {\n msg { Go\n Go }\n view { text \"ok\" {} }\n}".into(),
            "duplicate message variant",
        ),
    ];
    for (label, source, expected) in cases {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("app.at");
        std::fs::write(&path, source).expect("source");
        let error = generate_component_from_file(&path, ComponentGenOptions::default())
            .expect_err(label)
            .to_string();
        assert!(
            error.contains("UI domain validation failed"),
            "{label}: {error}"
        );
        assert!(error.contains(expected), "{label}: {error}");
    }
}

#[test]
fn file_generation_validates_cross_file_modules() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("child.at");
    let app = dir.path().join("app.at");
    let valid_child = "widget Imported { view { text \"child\" {} } }";
    std::fs::write(
        &app,
        "use child: Imported\nwidget App { view { Imported {} } }",
    )
    .expect("app source");
    std::fs::write(&module, format!("{valid_child}\n{valid_child}"))
        .expect("invalid module source");
    let options = ComponentGenOptions {
        sub_widgets: Some(vec!["Imported".into()]),
        ..Default::default()
    };
    let error = generate_component_from_file(&app, options.clone())
        .expect_err("invalid imported module must fail generation");
    assert!(error.contains("UI module validation failed"), "{error}");
    assert!(error.contains("duplicate widget declaration"), "{error}");

    std::fs::write(&module, valid_child).expect("valid module source");
    generate_component_from_file(&app, options).expect("valid imported module still generates");
}
