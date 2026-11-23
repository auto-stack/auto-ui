#![cfg(feature = "ace-bridge")]

use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
use std::path::Path;

#[test]
fn preparation_keeps_package_imports_and_injects_fields_inside_the_model() {
    let project = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/ui/029-photo-gallery"
    ));
    let source = r#"style panel = "p-2"
widget App {
    use { package: color_picker from "../../../../../packages/color-picker" }
    msg { Restore }
    model {
        // Preserve punctuation in the comment } .
        var title str = "quote: \" }"
        var rows = [{ id: 1, label: "row" }]
        var trailing int = 3
    }
    on { .Restore -> { .title = storage.get("sample") } }
    view { text .title { style: panel } }
}
"#;

    let prepared = prepare_component_source(project, source).unwrap().source;

    assert!(prepared.contains(
        r#"use { package: color_picker from "../../../../../packages/color-picker" }"#
    ));
    assert!(prepared.contains(r#"style: "p-2""#));
    assert!(!prepared.contains("style panel ="));
    assert!(prepared.contains("// Preserve punctuation in the comment } ."));
    assert!(prepared.contains(".title = .__st_sample"));

    let handler = prepared.find("\n    on {").expect("handler after the model");
    let model_end = prepared[..handler].rfind('}').expect("model closing brace");
    let last_field = prepared.find("var trailing int = 3").expect("last authored field");
    let host_field = prepared.find("var __st_sample").expect("injected host field");
    assert!(prepared[model_end + 1..handler].trim().is_empty(), "{prepared}");
    assert!(last_field < host_field && host_field < model_end, "{prepared}");
}
