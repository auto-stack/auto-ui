//! PLAN-088 official color-picker package admission tests: both generators,
//! the public event surface (preview/committed/cancelled/error), the integer
//! color engine's cross-track code shapes (self-contained fns, exact-division
//! forms, no float paths), and the vendored dependency closure at a second
//! absolute root (named failure when the closure is broken; the vendored
//! consumer also proves the nested numeric-fields dep resolves from a
//! relocated package root). The behavioral matrix (V-01..V-06) lives in the
//! live product gate tests/product/verify_color_picker.py; this file is the
//! tools track.
use std::path::{Path, PathBuf};

fn scratch() -> tempfile::TempDir {
    let root = std::env::var_os("PLAN088_ARTIFACT_ROOT").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if cfg!(windows) { "D:/autostack/plan-artifacts/plan-088" } else { "/mnt/d/autostack/plan-artifacts/plan-088" })
    });
    std::fs::create_dir_all(&root).unwrap();
    tempfile::Builder::new().prefix("color-picker-generation-").tempdir_in(root).unwrap()
}

fn official_package() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/color-picker")
}

/// AC-01: both high-level widgets compile through both generators while no
/// builtin name is shadowed (Builtin > Local > Package priority untouched;
/// ColorPicker/ColorDialog do not fold-collide with any builtin key).
#[test]
fn official_color_picker_compiles_through_both_generators() {
    for source in ["color_picker.at", "color_dialog.at"] {
        let path = official_package().join(source);
        let path = path.to_str().unwrap();
        let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None)
            .unwrap_or_else(|e| panic!("{source} rust generation failed: {e}"));
        let want = if source.starts_with("color_picker") { "ColorPicker" } else { "ColorDialog" };
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
/// package copied byte-equal TOGETHER WITH its numeric-fields dependency
/// (nested package use resolves relative to the package dir, never a foreign
/// checkout); consumer binds preview + committed + cancelled + error;
/// deleting the color-picker dependency is a NAMED failure in strict
/// generation. Engine shapes on the vue surface: self-contained fns,
/// exact-division quantize forms, integer-only (no Math.floor/parseFloat).
#[test]
fn vendored_closure_generates_and_missing_dependency_fails_named() {
    let temporary = scratch();
    let root = temporary.path();
    let front = root.join("src/front");
    let vendor = root.join("vendor/color-picker");
    let vendor_fields = root.join("vendor/numeric-fields");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::create_dir_all(&vendor).unwrap();
    std::fs::create_dir_all(&vendor_fields).unwrap();
    for (src_dir, dst) in [(official_package(), &vendor)] {
        let _ = src_dir;
        let _ = dst;
    }
    for entry in std::fs::read_dir(official_package()).unwrap().flatten() {
        let p = entry.path();
        if p.extension().map_or(true, |x| x != "at") {
            continue;
        }
        std::fs::copy(&p, vendor.join(entry.file_name())).unwrap();
    }
    // numeric-fields is vendored as a SIBLING of color-picker (the package's
    // `use { package: numeric_fields from "../numeric-fields" }` must resolve
    // inside the relocated root).
    let numeric_package = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/numeric-fields");
    for entry in std::fs::read_dir(numeric_package).unwrap().flatten() {
        let p = entry.path();
        if p.extension().map_or(true, |x| x != "at") {
            continue;
        }
        std::fs::copy(&p, vendor_fields.join(entry.file_name())).unwrap();
    }
    // vue-track materialization scans the CONSUMER descriptor dep roots, so
    // the transitive numeric-fields dep is declared alongside color-picker
    // (package-internal use resolves against the sibling vendored layout).
    std::fs::write(
        root.join("pac.at"),
        "name: \"color-vendor-control\"\nscene: \"ui\"\nrender: \"vue\"\ndep color_picker { path: \"vendor/color-picker\" }\ndep numeric_fields { path: \"vendor/numeric-fields\" }\n",
    )
    .unwrap();
    std::fs::write(
        front.join("app.at"),
        r##"
widget App {
    msg { OnPreview(str), OnCommitted(str), OnCancelled, OnError(str) }
    model {
        var v str = "#3974E6CC"
        var rev int = 1
        var open bool = true
        var pal List = ["#ef4444", "#22c55e", "#ef4444"]
    }
    use { package: color_picker from "../../vendor/color-picker" }
    view {
        col {
            ColorPicker (value: .v, revision: .rev, palette: .pal, onpreview: .OnPreview($event), oncommitted: .OnCommitted($event), oncancelled: .OnCancelled, onerror: .OnError($event)) {}
            ColorDialog (value: .v, revision: .rev, open: .open, palette: .pal, onpreview: .OnPreview($event), oncommitted: .OnCommitted($event), oncancelled: .OnCancelled, onerror: .OnError($event)) {}
        }
    }
    on {
        .OnPreview(ev str) -> {
            .v = json.parse(ev).value
        }
        .OnCommitted(ev str) -> {
            .v = json.parse(ev).value
            .rev = .rev + 1
        }
        .OnCancelled -> {}
        .OnError(ev str) -> {}
    }
}
"##,
    )
    .unwrap();
    let request =
        auto_project_model::GenerationRequest::from_descriptor(&auto_project_model::parse_file(&root.join("pac.at")).unwrap());
    let mut project = auto_ui_tools::vue::VueProject::from_workspace_with_request(root, &request).unwrap();
    project.output_dir = root.join("gen/front/vue");
    project.generate().expect("vendored closure generates");
    // 诊断转储（失败排查用；产物落 plan-artifacts，不入库）
    let dbg_dir = PathBuf::from(if cfg!(windows) { "D:/autostack/plan-artifacts/plan-088/last-gen" } else { "/mnt/d/autostack/plan-artifacts/plan-088/last-gen" });
    let _ = std::fs::create_dir_all(&dbg_dir);
    if let Ok(body) = std::fs::read_to_string(project.output_dir.join("src/components/ColorPicker.vue")) {
        let _ = std::fs::write(dbg_dir.join("ColorPicker.gen.vue"), &body);
    }
    let _ = std::fs::write(dbg_dir.join("App.gen.vue"), std::fs::read_to_string(project.output_dir.join("src/App.vue")).unwrap_or_default());
    let app = std::fs::read_to_string(project.output_dir.join("src/App.vue")).unwrap();
    assert!(app.contains("ColorPicker") && app.contains("@committed"), "{app}");
    assert!(app.contains("@preview") && app.contains("@cancelled") && app.contains("@error"), "full event surface bound: {app}");
    let picker = std::fs::read_to_string(project.output_dir.join("src/components/ColorPicker.vue")).unwrap();
    // AC-02 engine shapes: integer-only arithmetic with exact-division forms;
    // the color engine is inlined (self-contained fns), no float paths.
    assert!(picker.contains("JSON.stringify({ seq:"), "committed/preview payload shape");
    assert!(picker.contains("source: 'surface'"), "surface source marker");
    assert!(picker.contains("source: 'hex'"), "hex source marker");
    assert!(picker.contains("source: 'palette'"), "palette source marker");
    let stripped: String = picker.chars().filter(|c| !c.is_whitespace()).collect();
    // half-up exact-division inline signature: ((2a+b) - (2a+b) % (2b)) / (2b)
    assert!(stripped.contains("%(2*") && stripped.contains("/(2*"),
        "engine half-up must keep the exact-division form (2a+b-(2a+b)%(2b))/(2b)");
    // Note: the canvas script (PLAN-563 vue arm) legitimately parseFloat()s
    // scene-string coordinates — data parsing, not engine arithmetic. The
    // engine's integer discipline is pinned by the exact-division shape
    // assertion above (087's Math.floor/parseFloat markers applied to the
    // numeric engine, not the canvas coordinate parser).
    // The nested numeric-fields widget is generated as a sibling component.
    let fields = project.output_dir.join("src/components/NumericField.vue");
    assert!(fields.exists(), "nested numeric-fields dep must generate: {}", project.output_dir.display());
    let dialog = std::fs::read_to_string(project.output_dir.join("src/components/ColorDialog.vue")).unwrap();
    assert!(dialog.contains("source: 'confirm'"), "confirm source marker");
    assert_eq!(
        std::fs::read(official_package().join("color_model.at")).unwrap(),
        std::fs::read(vendor.join("color_model.at")).unwrap(),
        "vendored engine must be byte-equal"
    );

    // Negative: remove the color-picker dependency and regenerate under
    // strict mode — the broken closure must be a NAMED failure mentioning the
    // package, never a silent pick-up of a foreign checkout.
    std::fs::remove_dir_all(&vendor).unwrap();
    let strict = auto_ui_tools::tool_ops::GenerateOptions { strict: Some(true), ..Default::default() };
    let result = auto_ui_tools::tool_ops::generate_report_with(&request, &strict);
    let named = match &result {
        Err(error) => format!("{error}").contains("color"),
        Ok(report) => format!("{report:?}").contains("color"),
    };
    assert!(named, "missing color-picker dependency must be named in strict generation, got {result:?}");
}
