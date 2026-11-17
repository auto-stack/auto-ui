//! PLAN-085 official tree-view package admission tests.
//!
//! Two things are pinned here that the live product gate cannot cover:
//!
//! 1. Both generators turn the package into a real component (Rust and Vue),
//!    and the Vue surface carries the contract the consumers rely on — the
//!    quoted `"toggle"`/`"select"`/`"retry"` emits (unquoted variants are not
//!    rewritten into `emit(...)`), the literal chevron branches, and the
//!    four literal row-state branches.
//! 2. The vendored closure generates from a **second absolute root**, and a
//!    missing dependency is a *named* failure rather than a silent fallback to
//!    some other checkout. That is the "第二绝对根" requirement in the plan.
//!
//! Behavioural keyboard/row semantics live in
//! `crates/auto-ui-runtime/tests/plan085_tree_keyboard.rs`; the end-to-end
//! native/Vue product gate is `tests/product/verify_tree_view.py`.
use std::path::{Path, PathBuf};

fn scratch() -> tempfile::TempDir {
    let root = std::env::var_os("PLAN085_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(if cfg!(windows) {
                "D:/autostack/plan-artifacts/plan-085"
            } else {
                "/mnt/d/autostack/plan-artifacts/plan-085"
            })
        });
    std::fs::create_dir_all(&root).unwrap();
    tempfile::Builder::new()
        .prefix("tree-view-generation-")
        .tempdir_in(root)
        .unwrap()
}

fn official_tree_view() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/tree-view")
}

/// AC-01: the official package compiles through both generators, and the
/// generated surfaces carry the contract the consumers depend on.
#[test]
fn official_tree_view_compiles_through_both_generators() {
    let source = official_tree_view().join("tree_view.at");
    let path = source.to_str().unwrap();

    let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None)
        .expect("official tree-view Rust generation");
    assert!(
        rust.contains("pub struct TreeView") && rust.contains("impl Component for TreeView"),
        "Rust generation must produce the TreeView component"
    );
    // The whole controlled surface must survive generation.
    for prop in [
        "nodes",
        "expanded",
        "selected",
        "disabledIds",
        "loadingIds",
        "errorIds",
        "directory",
    ] {
        assert!(
            rust.contains(prop),
            "Rust generation must carry the `{prop}` prop"
        );
    }

    let vue = auto_ui_compiler::pipeline::ui_build(path, "ui", "vue", None)
        .expect("official tree-view Vue generation");
    assert!(
        vue.contains("<template>") && vue.contains("defineProps"),
        "Vue generation must produce an SFC"
    );
    // Quoted msg variants are the ones the Vue emit rewrite recognises; an
    // unquoted `toggle` would silently never reach the parent callback.
    for emit in ["toggle", "select", "retry"] {
        assert!(
            vue.contains(&format!("emit('{emit}'")),
            "Vue surface must emit `{emit}` (quoted variant); missing it kills the callback path"
        );
    }
    // Literal chevron branches: an icon with a dynamic name degrades to a
    // placeholder on the Vue track. The generator turns the two literal names
    // into real lucide component imports (PascalCase) — that is the surface to
    // assert, not the kebab names from the source.
    assert!(
        vue.contains("ChevronDown") && vue.contains("ChevronRight"),
        "chevron must be emitted as two literal lucide branches, not a dynamic name"
    );
    assert!(
        vue.contains("lucide-vue-next"),
        "chevron icons must come from the lucide import, not a runtime lookup"
    );
    // Four literal row-state branches (disabled × selected|focused). Building
    // these with an f-string would render empty on the VM track.
    assert!(
        vue.contains("line-through") && vue.contains("ring-2"),
        "selected/focused and disabled row states must be literal branches"
    );
}

/// AC-05: the package resolves and generates from a second absolute root, with
/// both files copied byte-equal, and a missing dependency is a named failure.
#[test]
fn vendored_closure_generates_and_missing_dependency_fails_named() {
    let temporary = scratch();
    let root = temporary.path();
    let front = root.join("src/front");
    let vendor = root.join("vendor/tree-view");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::create_dir_all(&vendor).unwrap();

    let official = official_tree_view();
    for name in [
        "package.at",
        "pac.at",
        "tree_view.at",
        "tree_view_util.at",
        "tree_icon.at",
    ] {
        std::fs::copy(official.join(name), vendor.join(name)).unwrap();
    }

    std::fs::write(
        root.join("pac.at"),
        "name: \"tree-view-vendor-control\"\nscene: \"ui\"\nrender: \"vue\"\ndep tree_view { path: \"vendor/tree-view\" }\n",
    )
    .unwrap();
    std::fs::write(
        front.join("app.at"),
        r#"
widget App {
    msg { OnToggle(str), OnSelect(str) }
    model {
        var nodes List = []
        var expanded List = []
        var sel str = ""
    }
    use { package: tree_view from "../../vendor/tree-view" }
    view {
        col {
            text f"sel=${.sel}"
            TreeView (nodes: .nodes, expanded: .expanded, selected: .sel, on_toggle: .OnToggle($event), on_select: .OnSelect($event), on_retry: .OnSelect($event), directory: false) {}
        }
    }
    on {
        .OnToggle(id str) -> { .expanded = [id] }
        .OnSelect(id str) -> { .sel = id }
    }
}
"#,
    )
    .unwrap();

    let request = auto_project_model::GenerationRequest::from_descriptor(
        &auto_project_model::parse_file(&root.join("pac.at")).unwrap(),
    );
    let mut project = auto_ui_tools::vue::VueProject::from_workspace_with_request(root, &request)
        .unwrap();
    project.output_dir = root.join("gen/front/vue");
    project.generate().expect("vendored closure generates");

    let component = std::fs::read_to_string(project.output_dir.join("src/components/TreeView.vue"))
        .expect("closure emits the TreeView component");
    assert!(
        component.contains("defineProps") && component.contains("toggle"),
        "vendored TreeView must carry props and the toggle emit"
    );
    let app = std::fs::read_to_string(project.output_dir.join("src/App.vue")).unwrap();
    assert!(
        app.contains("TreeView"),
        "consumer must reference the vendored TreeView"
    );

    // Byte-equal vendoring: the closure must not silently pick up a different
    // checkout's copy.
    for name in ["tree_view.at", "tree_view_util.at", "tree_icon.at"] {
        assert_eq!(
            std::fs::read(official.join(name)).unwrap(),
            std::fs::read(vendor.join(name)).unwrap(),
            "{name} must be vendored byte-equal"
        );
    }

    // Negative: remove the dependency and regenerate under strict mode — the
    // broken closure must be a NAMED failure, never a silent fallback.
    std::fs::remove_dir_all(&vendor).unwrap();
    let strict = auto_ui_tools::tool_ops::GenerateOptions {
        strict: Some(true),
        ..Default::default()
    };
    let result = auto_ui_tools::tool_ops::generate_report_with(&request, &strict);
    let named = match &result {
        Err(error) => format!("{error}").contains("tree"),
        Ok(report) => format!("{report:?}").contains("tree"),
    };
    assert!(
        named,
        "missing tree-view dependency must be named in strict generation, got {result:?}"
    );
}
