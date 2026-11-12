//! Runs the FileDialog fixture through the production VM/iced/MCP host.
fn main() {
    auto_ui_runtime::install_ui_host_capabilities();
    let app = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/file-dialog/src/front/app.at");
    let source = std::fs::read_to_string(&app).expect("FileDialog fixture source");
    if let Err(error) = auto_ui_runtime::run_file_dynamic_ui(&source, app.to_str()) {
        eprintln!("{error:?}");
        std::process::exit(1);
    }
}
