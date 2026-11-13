//! PLAN-086 official pagination package admission tests: both generators, the
//! dual-integer-precision derive contract on the vue surface, and the vendored
//! dependency closure (both packages at a second absolute root, named failure
//! when the closure is broken). Behavioral matrix lives in the live product
//! gate (tests/product/verify_pagination.py); this file is the tools track.
use std::path::{Path, PathBuf};

fn scratch() -> tempfile::TempDir {
    let root = std::env::var_os("PLAN086_ARTIFACT_ROOT").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if cfg!(windows) { "D:/autostack/plan-artifacts/plan-086" } else { "/mnt/d/autostack/plan-artifacts/plan-086" })
    });
    std::fs::create_dir_all(&root).unwrap();
    tempfile::Builder::new().prefix("pagination-generation-").tempdir_in(root).unwrap()
}

fn official_pagination() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/pagination")
}

/// AC-01: the high-level entry compiles through both generators while the
/// builtin pagination family stays untouched (Builtin > Local > Package
/// priority is never modified; the package name PaginationControl does not
/// fold-collide with any builtin key).
#[test]
fn official_pagination_compiles_through_both_generators() {
    let source = official_pagination().join("pagination_control.at");
    let path = source.to_str().unwrap();
    let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None).expect("official pagination Rust generation");
    assert!(rust.contains("pub struct PaginationControl") && rust.contains("impl Component for PaginationControl"));
    assert!(rust.contains("page_count") && rust.contains("pageSizes"));
    let vue = auto_ui_compiler::pipeline::ui_build(path, "ui", "vue", None).expect("official pagination Vue generation");
    assert!(vue.contains("<template>") && vue.contains("defineProps"));
    // AC-02 dual-track integer precision: the vue computed must carry the
    // float-safe ceil shape `(total - total % pageSize) / pageSize` — plain
    // `total / pageSize + 1` renders 11.03-style fractional page counts in JS.
    assert!(vue.contains("(props.total - props.total % props.pageSize) / props.pageSize"), "vue derive must use the float-safe ceil form");
    assert!(vue.contains("emit('changed',"));
}

/// AC-01/06: the vendored closure generates from a second absolute root —
/// both packages copied byte-equal, both dep roots declared, consumer binds
/// the changed event; deleting the pagination dependency is a named failure
/// (the consumer page must not silently pick up a foreign checkout).
#[test]
fn vendored_closure_generates_and_missing_dependency_fails_named() {
    let temporary = scratch();
    let root = temporary.path();
    let front = root.join("src/front");
    let vendor = root.join("vendor/pagination");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::create_dir_all(&vendor).unwrap();
    let official = official_pagination();
    for name in ["package.at", "pac.at", "pagination_control.at"] {
        std::fs::copy(official.join(name), vendor.join(name)).unwrap();
    }
    std::fs::write(root.join("pac.at"), "name: \"pagination-vendor-control\"\nscene: \"ui\"\nrender: \"vue\"\ndep pagination { path: \"vendor/pagination\" }\n").unwrap();
    std::fs::write(front.join("app.at"), r#"
widget App {
    msg { OnChanged(str) }
    model {
        var page int = 1
        var page_size int = 10
    }
    use { package: pagination from "../../vendor/pagination" }
    view {
        col {
            text f"p=${.page} ps=${.page_size}"
            PaginationControl (page: .page, pageSize: .page_size, total: 1003, onchanged: .OnChanged($event)) {}
        }
    }
    on { .OnChanged(ev str) -> { .page = 1 } }
}
"#).unwrap();
    let request = auto_project_model::GenerationRequest::from_descriptor(&auto_project_model::parse_file(&root.join("pac.at")).unwrap());
    let mut project = auto_ui_tools::vue::VueProject::from_workspace_with_request(root, &request).unwrap();
    project.output_dir = root.join("gen/front/vue");
    project.generate().expect("vendored closure generates");
    let pager = std::fs::read_to_string(project.output_dir.join("src/components/PaginationControl.vue")).expect("closure emits the pagination component");
    assert!(pager.contains("defineProps") && pager.contains("changed"));
    let app = std::fs::read_to_string(project.output_dir.join("src/App.vue")).unwrap();
    assert!(app.contains("PaginationControl") && app.contains("@changed"), "{app}");
    assert_eq!(std::fs::read(official.join("pagination_control.at")).unwrap(), std::fs::read(vendor.join("pagination_control.at")).unwrap());

    // Negative: remove the dependency and regenerate under strict mode — the
    // broken closure must be a NAMED failure (S003 pagination load), never a
    // silent fallback to some other checkout's pagination.
    std::fs::remove_dir_all(&vendor).unwrap();
    let strict = auto_ui_tools::tool_ops::GenerateOptions { strict: Some(true), ..Default::default() };
    let result = auto_ui_tools::tool_ops::generate_report_with(&request, &strict);
    let named = match &result {
        Err(error) => format!("{error}").contains("pagination"),
        Ok(report) => format!("{report:?}").contains("pagination"),
    };
    assert!(named, "missing pagination dependency must be named in strict generation, got {result:?}");
}
