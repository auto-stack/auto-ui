//! Official package admission and generation compatibility; native behavior
//! belongs to the live VM product gate, not generated-source string checks.
use std::path::{Path, PathBuf};

fn scratch() -> tempfile::TempDir {
    let root = std::env::var_os("PLAN082_ARTIFACT_ROOT").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if cfg!(windows) { "D:/autostack/plan-artifacts/plan-082" } else { "/mnt/d/autostack/plan-artifacts/plan-082" })
    });
    std::fs::create_dir_all(&root).unwrap();
    tempfile::Builder::new().prefix("package-generation-").tempdir_in(root).unwrap()
}

#[test]
fn official_file_dialog_compiles_through_both_generators() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/file-dialog/file_dialog.at");
    let path = source.to_str().unwrap();
    let rust = auto_ui_compiler::pipeline::ui_build(path, "ui", "rust", None).expect("official package Rust generation");
    assert!(rust.contains("pub struct FileDialog") && rust.contains("impl Component for FileDialog"));
    assert!(rust.contains("fd_command") && rust.contains("selectionMode"));
    let vue = auto_ui_compiler::pipeline::ui_build(path, "ui", "vue", None).expect("official package Vue generation");
    assert!(vue.contains("<template>") && vue.contains("defineProps") && vue.contains("selectionMode"));
    assert!(vue.contains("accepted") && vue.contains("cancelled"));
}

#[test]
fn project_request_loads_the_vendored_official_package_and_generates_its_component() {
    let temporary = scratch();
    let root = temporary.path();
    let front = root.join("src/front");
    std::fs::create_dir_all(&front).unwrap();
    let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/file-dialog").canonicalize().unwrap();
    // Vue's supported package layout is src/front/components. Preserve the
    // official source bytes and use the same package import as native VM.
    let components = front.join("components");
    std::fs::create_dir_all(&components).unwrap();
    for name in ["package.at", "file_dialog.at"] {
        std::fs::copy(package.join(name), components.join(name)).unwrap();
        assert_eq!(std::fs::read(package.join(name)).unwrap(), std::fs::read(components.join(name)).unwrap());
    }
    std::fs::write(root.join("pac.at"), "name: \"file-dialog-package-control\"\nscene: \"ui\"\nrender: \"vue\"\n").unwrap();
    std::fs::write(front.join("app.at"), format!(r#"
widget App {{
    use {{ package: file_dialog from {} }}
    msg {{ Accepted(str), Cancelled }}
    model {{ var open bool = true }}
    view {{ FileDialog (open: .open, selectionMode: "multipleFiles", onaccepted: .Accepted($event), oncancelled: .Cancelled) {{}} }}
    on {{
        .Accepted(value str) -> {{ .open = false }}
        .Cancelled -> {{ .open = false }}
    }}
}}
"#, serde_json::to_string("components").unwrap())).unwrap();
    let request = auto_project_model::GenerationRequest::from_descriptor(&auto_project_model::parse_file(&root.join("pac.at")).unwrap());
    let mut project = auto_ui_tools::vue::VueProject::from_workspace_with_request(root, &request).expect("request imports official external package");
    project.output_dir = root.join("generated");
    project.generate().expect("real source generation");
    let app = std::fs::read_to_string(project.output_dir.join("src/App.vue")).unwrap();
    assert!(app.contains("FileDialog") && app.contains("@accepted") && app.contains("@cancelled"), "{app}");
    let component = std::fs::read_to_string(project.output_dir.join("src/components/FileDialog.vue")).expect("imported package must generate a usable component artifact");
    assert!(component.contains("defineProps") && component.contains("initialDirectory") && component.contains("显示隐藏文件"));
}
