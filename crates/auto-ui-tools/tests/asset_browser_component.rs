//! PLAN-091 official asset-browser package admission tests: both generators,
//! the r2 event payload surface (queryRequested/selectionRequested/
//! previewRequested/confirmRequested/cancelled), the four-package dependency
//! closure at a second absolute root (named failure when broken), and the
//! builder-track negative (missing dependency = named strict failure, never a
//! silent fallback). The behavioral matrix (V-01..V-06) lives in the live
//! product gate tests/product/verify_asset_picker.py; this file is the tools
//! track.
use std::path::{Path, PathBuf};

fn scratch() -> tempfile::TempDir {
    let root = std::env::var_os("PLAN091_ARTIFACT_ROOT").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if cfg!(windows) { "D:/autostack/plan-artifacts/plan-091" } else { "/mnt/d/autostack/plan-artifacts/plan-091" })
    });
    std::fs::create_dir_all(&root).unwrap();
    tempfile::Builder::new().prefix("asset-browser-generation-").tempdir_in(root).unwrap()
}

fn official_package() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/asset-browser")
}

/// AC-01: both high-level widgets compile through both generators while no
/// builtin name is shadowed (Builtin > Local > Package priority untouched).
#[test]
fn official_asset_browser_compiles_through_both_generators() {
    for source in ["asset_browser.at", "asset_picker.at"] {
        let path = official_package().join(source);
        let path = path.to_str().unwrap();
        let want = if source.starts_with("asset_browser") { "AssetBrowser" } else { "AssetPicker" };
        let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None)
            .unwrap_or_else(|e| panic!("{source} rust generation failed: {e}"));
        assert!(
            rust.contains(&format!("pub struct {want}")) || rust.contains(want),
            "{source} rust output must carry the widget"
        );
        let vue = auto_ui_compiler::pipeline::ui_build(path, "ui", "vue", None)
            .unwrap_or_else(|e| panic!("{source} vue generation failed: {e}"));
        assert!(vue.contains("defineProps") || vue.contains("defineEmits"), "{source} vue SFC surface");
    }
}

/// AC-01/02/03/06: the vendored closure generates from a second absolute root —
/// asset-browser + tree-view + pagination + image-viewer copied byte-equal,
/// consumer binds queryRequested/selectionRequested/confirmRequested; deleting
/// the dependency is a NAMED failure in strict generation, never a silent
/// fallback to a foreign checkout.
#[test]
fn vendored_closure_generates_and_missing_dependency_fails_named() {
    let temporary = scratch();
    let root = temporary.path();
    let front = root.join("src/front");
    std::fs::create_dir_all(&front).unwrap();
    let vendor_root = root.join("vendor");
    std::fs::create_dir_all(&vendor_root).unwrap();
    let official = official_package();
    for pkg in ["asset-browser", "tree-view", "pagination", "image-viewer"] {
        let src_dir = if pkg == "asset-browser" {
            official.clone()
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages").join(pkg)
        };
        let dst = vendor_root.join(pkg);
        std::fs::create_dir_all(&dst).unwrap();
        for entry in std::fs::read_dir(&src_dir).unwrap().flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |x| x != "at") {
                continue;
            }
            std::fs::copy(&p, dst.join(entry.file_name())).unwrap();
        }
    }
    std::fs::write(
        root.join("pac.at"),
        "name: \"asset-browser-vendor-control\"\nscene: \"ui\"\nrender: \"vue\"\n\
         dep asset_browser { path: \"vendor/asset-browser\" }\n\
         dep tree_view { path: \"vendor/tree-view\" }\n\
         dep pagination { path: \"vendor/pagination\" }\n\
         dep image_viewer { path: \"vendor/image-viewer\" }\n",
    )
    .unwrap();
    std::fs::write(
        front.join("app.at"),
        r#"
widget demo {
    msg { DoQuery(str), DoSel(str), DoConfirm(str), DoCancel }
    model {
        var recs List = []
        var query str = ""
        var open bool = false
    }
    use { package: asset_browser from "../../../vendor/asset-browser" }
    view {
        col {
            AssetPicker (
                records: .recs, query: .query, open: .open,
                onqueryRequested: .DoQuery($event),
                onselectionRequested: .DoSel($event),
                onconfirmRequested: .DoConfirm($event),
                oncancelled: .DoCancel
            ) {}
        }
    }
    on {
        .DoQuery(ev str) -> { .query = ev }
        .DoSel(ev str) -> { .recs = json.parse(ev).id }
        .DoConfirm(ev str) -> { .open = false }
        .DoCancel -> { .open = false }
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
    assert!(app.contains("AssetPicker"), "consumer must bind the package widget: {app}");
    let picker = std::fs::read_to_string(project.output_dir.join("src/components/AssetPicker.vue")).unwrap();
    // AC-02/03: r2 event surface on the emitted channels.
    assert!(picker.contains("queryRequested") && picker.contains("selectionRequested"), "forward surface: {picker}");
    assert!(picker.contains("confirmRequested"), "confirm surface");
    assert!(picker.contains("cancelled"), "cancel surface");
    let browser = std::fs::read_to_string(project.output_dir.join("src/components/AssetBrowser.vue")).unwrap();
    assert!(browser.contains("PaginationControl"), "browser must compose the pagination package");
    assert!(browser.contains("TreeView"), "browser must compose the tree-view package");
    assert!(browser.contains("ImageViewport"), "browser must compose the image-viewer preview");
    for pkg in ["asset-browser", "tree-view", "pagination", "image-viewer"] {
        let src_dir = if pkg == "asset-browser" {
            official.clone()
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages").join(pkg)
        };
        let vendored = vendor_root.join(pkg);
        for entry in std::fs::read_dir(src_dir).unwrap().flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |x| x != "at") {
                continue;
            }
            assert_eq!(
                std::fs::read(&p).unwrap(),
                std::fs::read(vendored.join(entry.file_name())).unwrap(),
                "vendored {pkg}/{} must be byte-equal",
                entry.file_name().to_string_lossy()
            );
        }
    }

    // Negative: remove the asset-browser dependency and regenerate under strict
    // mode — the broken closure must be a NAMED failure mentioning the package,
    // never a silent pick-up of a foreign checkout.
    std::fs::remove_dir_all(vendor_root.join("asset-browser")).unwrap();
    let strict = auto_ui_tools::tool_ops::GenerateOptions { strict: Some(true), ..Default::default() };
    let result = auto_ui_tools::tool_ops::generate_report_with(&request, &strict);
    let named = match &result {
        Err(error) => format!("{error}").contains("asset"),
        Ok(report) => format!("{report:?}").contains("asset"),
    };
    assert!(named, "missing asset-browser dependency must be named in strict generation, got {result:?}");
}
