//! Public native-editor regression for Spec document preservation (PLAN-106).
#![cfg(all(feature = "autodown", feature = "code-editor"))]
use auto_ui_runtime::ui::{
    autodown_editor::{autodown_editor, storage_key, DocInput},
    code_editor::{
        core::{
            set_font_system_call, with_font_system, EditorButton, EditorKey, EditorModifiers,
            NullClipboard,
        },
        theme::Rgba,
    },
};
use cosmic_text::FontSystem;
use std::sync::{OnceLock, RwLock};

fn fonts<R>(f: impl FnOnce(&mut FontSystem) -> R) -> R {
    static FONTS: OnceLock<RwLock<FontSystem>> = OnceLock::new();
    set_font_system_call(|with| {
        with(
            &mut FONTS
                .get_or_init(|| RwLock::new(FontSystem::new()))
                .write()
                .unwrap(),
        )
    });
    with_font_system(f)
}
fn edit_after_click(key: &str, motion: Option<EditorKey>, shift: bool) -> String {
    fonts(|_| {});
    let core = autodown_editor(&storage_key(key));
    assert!(core.sync_external("Native baseline 106", true));
    fonts(|fs| {
        core.render_frame(
            fs,
            800.0,
            Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            None,
        );
        for input in [
            DocInput::MousePressed {
                button: EditorButton::Left,
                x: 20.0,
                y: 10.0,
            },
            DocInput::MouseDragged { x: 20.0, y: 10.0 },
            DocInput::MouseReleased {
                button: EditorButton::Left,
                x: 20.0,
                y: 10.0,
            },
        ] {
            core.handle_input(fs, input, &mut NullClipboard);
        }
        assert_eq!(core.focused_block(), Some(0));
        let mut modifiers = EditorModifiers::none();
        modifiers.shift = shift;
        if let Some(key) = motion {
            core.handle_input(
                fs,
                DocInput::KeyPressed {
                    key,
                    text: None,
                    modifiers,
                },
                &mut NullClipboard,
            );
        }
        core.handle_input(
            fs,
            DocInput::KeyPressed {
                key: EditorKey::Char('!'),
                text: None,
                modifiers: EditorModifiers::none(),
            },
            &mut NullClipboard,
        );
    });
    core.emit_document()
}
#[test]
fn spec_doc_plain_typing_keeps_unmodified_text() {
    let text = edit_after_click("spec106-typing", None, false);
    assert!(text.contains('!'));
    assert_eq!(text.replace('!', ""), "Native baseline 106");
}
#[test]
fn spec_doc_plain_end_keeps_unmodified_text() {
    assert_eq!(
        edit_after_click("spec106-end", Some(EditorKey::End), false),
        "Native baseline 106!"
    );
}

#[test]
fn spec_doc_other_plain_motions_keep_unmodified_text() {
    for (name, key) in [
        ("home", EditorKey::Home),
        ("left", EditorKey::Left),
        ("right", EditorKey::Right),
    ] {
        let text = edit_after_click(&format!("spec106-{name}"), Some(key), false);
        assert!(text.contains('!'));
        assert_eq!(text.replace('!', ""), "Native baseline 106");
    }
}
#[test]
fn spec_doc_shift_end_preserves_intentional_selection() {
    assert_eq!(
        edit_after_click("spec106-shift-end", Some(EditorKey::End), true),
        "Na!"
    );
}
