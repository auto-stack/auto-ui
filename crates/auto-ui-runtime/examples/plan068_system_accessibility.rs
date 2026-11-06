//! Real Windows acceptance surface for PLAN-068; no fixture-state injection.
fn main() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/plan068_accessibility/app.at");
    let source = std::fs::read_to_string(&path).unwrap();
    let component = auto_ui_runtime::build_dynamic_component(&source, path.to_str()).unwrap();
    auto_ui_runtime::ui::iced::run_dynamic_iced(component).unwrap();
}
