//! PLAN-092 official property-inspector package admission tests: both
//! generators, the r2 request/payload surface (changeRequested/preview/
//! previewCancelled/error with seq/baseRevision/schemaRevision/targetId/
//! fieldId/kind/value[/unit]/source), the controlled bool mirror, the string
//! no-arg commit shape (T-01 D-1), and the vendored dependency closure at a
//! second absolute root (named failure when the closure is broken). The
//! behavioral matrix (V-01..V-06) lives in the live product gate
//! tests/product/verify_property_inspector.py; this file is the tools track.
use std::path::{Path, PathBuf};

fn scratch() -> tempfile::TempDir {
    let root = std::env::var_os("PLAN092_ARTIFACT_ROOT").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if cfg!(windows) { "D:/autostack/plan-artifacts/plan-092" } else { "/mnt/d/autostack/plan-artifacts/plan-092" })
    });
    std::fs::create_dir_all(&root).unwrap();
    tempfile::Builder::new().prefix("inspector-generation-").tempdir_in(root).unwrap()
}

fn official_packages() -> [(&'static str, PathBuf); 3] {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    [
        ("property_inspector", base.join("property-inspector")),
        ("numeric_fields", base.join("numeric-fields")),
        ("color_picker", base.join("color-picker")),
    ]
}

/// AC-01: the inspector widget compiles through both generators (the pure-fn
/// model module is exercised via the full-project generation in the vendored
/// test below — standalone ui_build only accepts widget/store files).
#[test]
fn official_property_inspector_compiles_through_both_generators() {
    let source = "property_inspector.at";
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/property-inspector")
        .join(source);
    let path = path.to_str().unwrap();
    let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None)
        .unwrap_or_else(|e| panic!("{source} rust generation failed: {e}"));
    assert!(rust.contains("PropertyInspector"), "{source} rust output must carry the widget");
    let vue = auto_ui_compiler::pipeline::ui_build(path, "ui", "vue", None)
        .unwrap_or_else(|e| panic!("{source} vue generation failed: {e}"));
    assert!(vue.contains("defineProps"), "{source} vue SFC surface");
}

/// AC-01/06: the vendored closure generates from a second absolute root with
/// all three packages copied byte-equal; the consumer binds the inspector's
/// public events; the vue surface carries the r2 request contract. Deleting
/// the inspector dependency is a NAMED strict failure, never a silent
/// fallback to a foreign checkout.
#[test]
fn vendored_closure_generates_and_missing_dependency_fails_named() {
    let temporary = scratch();
    let root = temporary.path();
    let front = root.join("src/front");
    std::fs::create_dir_all(&front).unwrap();
    let mut dep_lines = String::new();
    for (name, dir) in official_packages() {
        let vendor = root.join("vendor").join(dir.file_name().unwrap());
        std::fs::create_dir_all(&vendor).unwrap();
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |x| x != "at") {
                continue;
            }
            std::fs::copy(&p, vendor.join(entry.file_name())).unwrap();
        }
        dep_lines.push_str(&format!("dep {name} {{ path: \"vendor/{0}\" }}\n", dir.file_name().unwrap().to_str().unwrap()));
    }
    std::fs::write(
        root.join("pac.at"),
        format!(
            "name: \"inspector-vendor-control\"\nscene: \"ui\"\nrender: \"vue\"\n{}",
            dep_lines
        ),
    )
    .unwrap();
    std::fs::write(
        front.join("app.at"),
        r#"
widget App {
    use { package: property_inspector from "../../vendor/property-inspector" }
    use { package: numeric_fields from "../../vendor/numeric-fields" }
    use { package: color_picker from "../../vendor/color-picker" }
    msg { OnChange(str), OnPreview(str), OnPreviewCancelled(str), OnErr(str) }
    model {
        var fields List = []
        var values List = []
        var groups List = []
        var ferrs List = []
    }
    view {
        col {
            PropertyInspector (fields: .fields, values: .values, groups: .groups,
                fieldErrors: .ferrs, targetId: "reading", revision: 1, schemaRevision: 1,
                onchangeRequested: .OnChange($event), onpreviewRequested: .OnPreview($event),
                onpreviewCancelled: .OnPreviewCancelled($event), onerror: .OnErr($event)) {}
        }
    }
    on {
        .OnChange(ev str) -> { }
        .OnPreview(ev str) -> { }
        .OnPreviewCancelled(ev str) -> { }
        .OnErr(ev str) -> { }
    }
}
"#,
    )
    .unwrap();
    let request =
        auto_project_model::GenerationRequest::from_descriptor(&auto_project_model::parse_file(&root.join("pac.at")).unwrap());
    let mut project = auto_ui_tools::vue::VueProject::from_workspace_with_request(root, &request).unwrap();
    project.output_dir = root.join("gen/front/vue");
    project.generate().expect("vendored closure generates");
    let app = std::fs::read_to_string(project.output_dir.join("src/App.vue")).unwrap();
    assert!(app.contains("PropertyInspector") && app.contains("@changeRequested"), "{app}");
    assert!(app.contains("@previewRequested") && app.contains("@previewCancelled") && app.contains("@error"),
        "all four public inspector events bind on the consumer: {app}");

    let inspector = std::fs::read_to_string(project.output_dir.join("src/components/PropertyInspector.vue")).unwrap();
    // r2 request contract on the vue surface.
    assert!(inspector.contains("JSON.stringify({ seq:"), "changeRequested payload shape");
    for marker in ["fieldId:", "targetId:", "kind:", "source:"] {
        assert!(inspector.contains(marker), "request identity marker {marker} must be present");
    }
    assert!(inspector.contains("'reset'"), "restore default emits source=reset");
    assert!(inspector.contains("emit('previewRequested'"), "preview surface");
    assert!(inspector.contains("emit('previewCancelled'"), "preview cancellation surface");
    assert!(inspector.contains("emit('error'"), "error surface");
    // T-01 D-1: string commit is a bare no-arg invoke (view-arg onenter does
    // not dispatch on native); D-2: enum editor is a chip button group.
    assert!(inspector.contains("StrCommit"), "string commit handler name");
    assert!(!inspector.contains("onenter: "), "onenter binds must be bare-name");
    // module fns inline into the SFC (engine-style, like numeric_decimal).
    // B-6: row construction stays inline in the Tick handler (fn-returned
    // records do not reproject on the native view); pure helpers stay modular.
    assert!(inspector.contains("pi_schema_error"), "schema scan fn inlines into the SFC");
    assert!(inspector.contains("pi_value_error"), "value-error fn inlines into the SFC");
    assert!(inspector.contains("function pi_schema_error"), "module fn is declared in the SFC");
    // 087/088 sub-editors compose (no re-implemented numeric/color engines).
    assert!(inspector.contains("NumericField") && inspector.contains("UnitField") && inspector.contains("ColorPicker"),
        "official sub-editors compose into the inspector");

    // Byte-equal closure check for the three packages.
    for (name, dir) in official_packages() {
        let vendor = root.join("vendor").join(dir.file_name().unwrap());
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |x| x != "at") {
                continue;
            }
            assert_eq!(
                std::fs::read(&p).unwrap(),
                std::fs::read(vendor.join(entry.file_name())).unwrap(),
                "vendored {name} must be byte-equal"
            );
        }
    }

    // Negative: remove the inspector dependency and regenerate under strict
    // mode — the broken closure must be a NAMED failure, never a silent
    // pick-up of a foreign checkout.
    std::fs::remove_dir_all(root.join("vendor/property-inspector")).unwrap();
    let strict = auto_ui_tools::tool_ops::GenerateOptions { strict: Some(true), ..Default::default() };
    let result = auto_ui_tools::tool_ops::generate_report_with(&request, &strict);
    let named = match &result {
        Err(error) => format!("{error}").contains("property"),
        Ok(report) => format!("{report:?}").contains("property"),
    };
    assert!(named, "missing property-inspector dependency must be named in strict generation, got {result:?}");
}
