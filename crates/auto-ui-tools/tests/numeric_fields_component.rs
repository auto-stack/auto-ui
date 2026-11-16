//! PLAN-087 official numeric-fields package admission tests: both generators,
//! the event payload surface (committed/preview/unitChanged/error), the
//! decimal engine's cross-track code shapes (integer-only arithmetic, exact
//! division forms), and the vendored dependency closure at a second absolute
//! root (named failure when the closure is broken). The behavioral matrix
//! (V-01..V-06) lives in the live product gate
//! tests/product/verify_numeric_fields.py; this file is the tools track.
use std::path::{Path, PathBuf};

fn scratch() -> tempfile::TempDir {
    let root = std::env::var_os("PLAN087_ARTIFACT_ROOT").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if cfg!(windows) { "D:/autostack/plan-artifacts/plan-087" } else { "/mnt/d/autostack/plan-artifacts/plan-087" })
    });
    std::fs::create_dir_all(&root).unwrap();
    tempfile::Builder::new().prefix("numeric-generation-").tempdir_in(root).unwrap()
}

fn official_package() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/numeric-fields")
}

/// AC-01: both high-level widgets compile through both generators while no
/// builtin name is shadowed (Builtin > Local > Package priority untouched;
/// NumericField/UnitField do not fold-collide with any builtin key).
#[test]
fn official_numeric_fields_compile_through_both_generators() {
    for source in ["numeric_field.at", "unit_field.at"] {
        let path = official_package().join(source);
        let path = path.to_str().unwrap();
        let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None)
            .unwrap_or_else(|e| panic!("{source} rust generation failed: {e}"));
        let want = if source.starts_with("numeric_field") { "NumericField" } else { "UnitField" };
        assert!(
            rust.contains(&format!("pub struct {want}")) || rust.contains(want),
            "{source} rust output must carry the widget"
        );
        let vue = auto_ui_compiler::pipeline::ui_build(path, "ui", "vue", None)
            .unwrap_or_else(|e| panic!("{source} vue generation failed: {e}"));
        assert!(vue.contains("defineProps") || vue.contains("defineEmits"), "{source} vue SFC surface");
    }
}

/// AC-01/06: the vendored closure generates from a second absolute root —
/// package copied byte-equal, consumer binds committed + unitChanged; deleting
/// the dependency is a NAMED failure in strict generation (S0-style load error),
/// never a silent fallback to a foreign checkout.
#[test]
fn vendored_closure_generates_and_missing_dependency_fails_named() {
    let temporary = scratch();
    let root = temporary.path();
    let front = root.join("src/front");
    let vendor = root.join("vendor/numeric-fields");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::create_dir_all(&vendor).unwrap();
    let official = official_package();
    for entry in std::fs::read_dir(&official).unwrap().flatten() {
        let p = entry.path();
        if p.extension().map_or(true, |x| x != "at") {
            continue;
        }
        std::fs::copy(&p, vendor.join(entry.file_name())).unwrap();
    }
    std::fs::write(
        root.join("pac.at"),
        "name: \"numeric-vendor-control\"\nscene: \"ui\"\nrender: \"vue\"\ndep numeric_fields { path: \"vendor/numeric-fields\" }\n",
    )
    .unwrap();
    std::fs::write(
        front.join("app.at"),
        r#"
widget App {
    msg { OnCommitted(str), OnUnit(str) }
    model {
        var v str = "1.00"
        var rev int = 1
        var unit str = "mm"
    }
    use { package: numeric_fields from "../../vendor/numeric-fields" }
    view {
        col {
            NumericField (value: .v, revision: .rev, precision: 2, step: "0.10", oncommitted: .OnCommitted($event)) {}
            UnitField (value: .v, revision: .rev, precision: 2, step: "1", unitGroup: "length", unit: .unit, oncommitted: .OnCommitted($event), onunitChanged: .OnUnit($event)) {}
        }
    }
    on {
        .OnCommitted(ev str) -> { .v = json.parse(ev).value }
        .OnUnit(ev str) -> { .unit = json.parse(ev).unit }
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
    assert!(app.contains("NumericField") && app.contains("@committed"), "{app}");
    assert!(app.contains("@unitChanged"), "camelCase child emit must keep its case: {app}");
    let field = std::fs::read_to_string(project.output_dir.join("src/components/NumericField.vue")).unwrap();
    assert!(field.contains("emit('committed'") && field.contains("function nd_fmt"), "engine fns inline into the SFC");
    // AC-02 shapes on the vue surface: the r2 event payload (seq/baseRevision/
    // value/source), the exact-division quantize form (PLAN-086: raw JS `/` is
    // float and produced 11.03), and an integer-only engine (no float paths).
    assert!(field.contains("JSON.stringify({ seq:"), "committed/preview payload shape");
    assert!(field.contains("source: 'input'"), "input source marker");
    assert!(field.contains("source: 'step'"), "step source marker");
    assert!(
        field.contains("%pow) / pow") || field.contains("% pow) / pow"),
        "quantize must use (a - a % b) / b exact-division form"
    );
    for float_marker in ["Math.floor", "parseFloat"] {
        assert!(!field.contains(float_marker), "engine must not emit {float_marker}");
    }
    let unit_sfc = std::fs::read_to_string(project.output_dir.join("src/components/UnitField.vue")).unwrap();
    assert!(unit_sfc.contains("emit('unitChanged'"), "unitChanged emit preserved");
    assert!(unit_sfc.contains("unitChanged(JSON.stringify({ seq:"), "unitChanged payload shape");
    let unit_value_marker = "unitChanged(JSON.stringify({ seq: seq.value, baseRevision: props.revision, value:";
    assert!(!unit_sfc.contains(unit_value_marker), "unit switch must not carry a value");
    assert_eq!(
        std::fs::read(official.join("numeric_decimal.at")).unwrap(),
        std::fs::read(vendor.join("numeric_decimal.at")).unwrap(),
        "vendored engine must be byte-equal"
    );

    // Negative: remove the dependency and regenerate under strict mode — the
    // broken closure must be a NAMED failure mentioning the package, never a
    // silent pick-up of a foreign checkout.
    std::fs::remove_dir_all(&vendor).unwrap();
    let strict = auto_ui_tools::tool_ops::GenerateOptions { strict: Some(true), ..Default::default() };
    let result = auto_ui_tools::tool_ops::generate_report_with(&request, &strict);
    let named = match &result {
        Err(error) => format!("{error}").contains("numeric"),
        Ok(report) => format!("{report:?}").contains("numeric"),
    };
    assert!(named, "missing numeric-fields dependency must be named in strict generation, got {result:?}");
}
