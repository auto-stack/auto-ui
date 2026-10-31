//! PLAN-107: an isolated process must not inherit an installed font callback.
#![cfg(all(feature = "autodown", feature = "code-editor"))]
use auto_ui_runtime::ui::{autodown_editor as ade, iced::renderer::IntoIcedElement, View};

#[test]
fn first_fence_after_core_probe_has_backend_fonts() {
    let key = "plan107-first-fence";
    ade::autodown_editor(key);
    let source = "# 中文迁移\n\n````json\n{\"original\": true}\n````\n";
    let view: View<()> = View::AutodownEditor {
        key: key.into(),
        value: source.into(),
        is_final: true,
        on_change: None,
        on_focus: None,
        on_link: None,
        placeholder: None,
        style: None,
    };
    let _element = view.into_iced();
    let emitted = ade::autodown_editor_text(key).unwrap();
    assert!(emitted.contains("中文迁移"));
    let json = emitted
        .split("```json\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(json).unwrap(),
        serde_json::json!({"original": true})
    );
}
