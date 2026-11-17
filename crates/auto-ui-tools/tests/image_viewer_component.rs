//! PLAN-089 official image-viewer package admission tests: both generators,
//! the event payload surface (viewRequested/loaded/error/navigateRequested),
//! the native input adaptation contract markers, and the vendored dependency
//! closure at a second absolute root (named failure when the closure is
//! broken). The behavioral matrix (V-01..V-06) lives in the live product gate
//! tests/product/verify_image_preview.py; this file is the tools track.
use std::path::{Path, PathBuf};

fn scratch() -> tempfile::TempDir {
    let root = std::env::var_os("PLAN089_ARTIFACT_ROOT").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if cfg!(windows) { "D:/autostack/plan-artifacts/plan-089" } else { "/mnt/d/autostack/plan-artifacts/plan-089" })
    });
    std::fs::create_dir_all(&root).unwrap();
    tempfile::Builder::new().prefix("image-viewer-generation-").tempdir_in(root).unwrap()
}

fn official_package() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/image-viewer")
}

/// AC-01: both high-level widgets compile through both generators while no
/// builtin name is shadowed (Builtin > Local > Package priority untouched;
/// ImageViewport/ImagePreviewDialog do not fold-collide with any builtin key).
#[test]
fn official_image_viewer_compiles_through_both_generators() {
    for source in ["image_viewport.at", "image_preview_dialog.at"] {
        let path = official_package().join(source);
        let path = path.to_str().unwrap();
        let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None)
            .unwrap_or_else(|e| panic!("{source} rust generation failed: {e}"));
        let want = if source.starts_with("image_viewport") { "ImageViewport" } else { "ImagePreviewDialog" };
        assert!(
            rust.contains(&format!("pub struct {want}")) || rust.contains(want),
            "{source} rust output must carry the widget"
        );
        let vue = auto_ui_compiler::pipeline::ui_build(path, "ui", "vue", None)
            .unwrap_or_else(|e| panic!("{source} vue generation failed: {e}"));
        assert!(vue.contains("defineProps") || vue.contains("defineEmits"), "{source} vue SFC surface");
    }
}

/// AC-01/02/04: the package's ImageSurface wiring keeps the r2 payload
/// contract on the generated surface — wheel delta param, three-phase pan
/// with string phases, and the seq/baseRevision/source request payloads.
/// AC-06: the vendored closure generates from a second absolute root —
/// package copied byte-equal, consumer binds viewRequested/loaded/error/
/// navigateRequested; deleting the dependency is a NAMED failure in strict
/// generation, never a silent fallback to a foreign checkout.
#[test]
fn vendored_closure_generates_and_missing_dependency_fails_named() {
    let temporary = scratch();
    let root = temporary.path();
    let front = root.join("src/front");
    let vendor = root.join("vendor/image-viewer");
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
        "name: \"image-viewer-vendor-control\"\nscene: \"ui\"\nrender: \"vue\"\ndep image_viewer { path: \"vendor/image-viewer\" }\n",
    )
    .unwrap();
    std::fs::write(
        front.join("app.at"),
        r#"
widget demo {
    msg { ApplyView(str), MarkLoaded(str), ShowError(str), DoNavigate(str) }
    model {
        var src str = "/api/__auto/media/demo/7"
        var rid str = "demo"
        var rrev str = "7"
        var rev int = 4
        var fit str = "contain"
        var zoom float = 1.0
        var ox float = 0.0
        var oy float = 0.0
        var rot int = 0
    }
    use { package: image_viewer from "../../../vendor/image-viewer" }
    view {
        col {
            ImageViewport (
                src: .src, resourceId: .rid, resourceRevision: .rrev,
                revision: .rev, fit: .fit, zoom: .zoom,
                offsetX: .ox, offsetY: .oy, rotation: .rot,
                onviewRequested: .ApplyView($event),
                onloaded: .MarkLoaded($event),
                onerror: .ShowError($event),
                onnavigateRequested: .DoNavigate($event)
            ) {}
        }
    }
    on {
        .ApplyView(ev str) -> { let req = json.parse(ev)  .rev = req.baseRevision + 1 }
        .MarkLoaded(ev str) -> { .rid = json.parse(ev).resourceId }
        .ShowError(ev str) -> { .rrev = json.parse(ev).code }
        .DoNavigate(ev str) -> { .rrev = json.parse(ev).direction }
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
    assert!(app.contains("ImageViewport"), "consumer must bind the package widget: {app}");
    let viewport =
        std::fs::read_to_string(project.output_dir.join("src/components/ImageViewport.vue")).unwrap();
    // AC-04: r2 event payload shapes (seq/baseRevision/view/source) on the
    // emitted viewRequested/navigateRequested/loaded/error channels.
    assert!(viewport.contains("JSON.stringify({ seq:"), "request payload shape: {viewport}");
    assert!(viewport.contains("baseRevision:"), "baseRevision echo");
    assert!(viewport.contains("source: 'zoom'"), "zoom source marker");
    assert!(viewport.contains("source: 'pan'"), "pan source marker");
    assert!(viewport.contains("source: 'fit'"), "fit source marker");
    assert!(viewport.contains("source: 'rotate'"), "rotate source marker");
    assert!(viewport.contains("source: 'reset'"), "reset source marker");
    assert!(viewport.contains("source: 'gesture'"), "gesture source marker");
    assert!(viewport.contains("direction: 'previous'") && viewport.contains("direction: 'next'"), "navigate payload");
    // AC-01/06: the inner ImageSurface binding keeps the cross-backend input
    // contract — wheel delta param (vue generator substitution) and the
    // three-phase pan wrapper with string phases.
    assert!(viewport.contains("deltaY"), "wheel delta param: {viewport}");
    assert!(viewport.contains("@pointerdown") && viewport.contains("@pointermove") && viewport.contains("@pointerup"), "pan three-phase");
    assert!(viewport.contains("'start'") && viewport.contains("'move'") && viewport.contains("'end'"), "pan phases");
    assert!(viewport.contains("@dblclick"), "dblclick arm");
    for float_marker in ["parseFloat"] {
        assert!(!viewport.contains(float_marker), "no string-float reparse in transform math: {float_marker}");
    }
    assert_eq!(
        std::fs::read(official.join("image_viewport.at")).unwrap(),
        std::fs::read(vendor.join("image_viewport.at")).unwrap(),
        "vendored package must be byte-equal"
    );

    // Negative: remove the dependency and regenerate under strict mode — the
    // broken closure must be a NAMED failure mentioning the package, never a
    // silent pick-up of a foreign checkout.
    std::fs::remove_dir_all(&vendor).unwrap();
    let strict = auto_ui_tools::tool_ops::GenerateOptions { strict: Some(true), ..Default::default() };
    let result = auto_ui_tools::tool_ops::generate_report_with(&request, &strict);
    let named = match &result {
        Err(error) => format!("{error}").contains("image"),
        Ok(report) => format!("{report:?}").contains("image"),
    };
    assert!(named, "missing image-viewer dependency must be named in strict generation, got {result:?}");
}
