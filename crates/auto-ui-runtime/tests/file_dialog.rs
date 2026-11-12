use auto_ui_runtime::ui::file_dialog::{list_directory, DirectoryRequest};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

#[test]
fn directory_identity_sorting_unicode_and_errors() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("z directory")).unwrap();
    std::fs::write(temp.path().join("中文 long name.TXT"), b"book").unwrap();
    std::fs::write(temp.path().join(".hidden"), b"secret").unwrap();
    std::fs::write(temp.path().join("A.txt"), b"a").unwrap();
    let list = list_directory(temp.path(), &AtomicBool::new(false)).unwrap();
    assert_eq!(
        list.entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["z directory", ".hidden", "A.txt", "中文 long name.TXT"]
    );
    assert!(list.entries[0].is_dir);
    assert!(list.entries[1].hidden);
    assert_eq!(list.entries[3].size, 4);
    for entry in list.entries {
        assert!(std::path::Path::new(&entry.path).is_absolute());
        assert_eq!(
            std::path::Path::new(&entry.path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            entry.name
        );
    }
    let err = list_directory(&temp.path().join("absent"), &AtomicBool::new(false)).unwrap_err();
    assert_eq!(err.code, "not_found");
}

#[test]
fn superseded_request_and_cancel_cannot_publish_old_results() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    std::fs::write(a.path().join("old.txt"), b"old").unwrap();
    std::fs::write(b.path().join("current.txt"), b"new").unwrap();
    let mut io = DirectoryRequest::default();
    io.start(a.path().to_owned()).unwrap();
    // Even an already-completed A is excluded when B replaces its receiver.
    io.start(b.path().to_owned()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let result = loop {
        if let Some(value) = io.poll() {
            break value.unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "directory request did not complete"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(result.entries[0].name, "current.txt");
    assert!(io.poll().is_none());
    io.start(a.path().to_owned()).unwrap();
    io.cancel();
    assert!(io.poll().is_none());
    io.start(b.path().to_owned()).unwrap();
    while io.poll().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn package_fixture_builds_and_opens_a_real_component_instance() {
    use auto_ui_runtime::Component;
    let app = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/file-dialog/src/front/app.at");
    let source = std::fs::read_to_string(&app).unwrap();
    let mut component = auto_ui_runtime::build_dynamic_component(&source, app.to_str()).unwrap();
    let _ = component.view();
    component.drive_scheduler_to_quiescence(16);
    component.on_with_input_for_instance_type("selector", "Open", None);
    let _ = component.view();
    component.drive_scheduler_to_quiescence(16);
    component.on_with_input_for_instance_type("FileDialog", "Tick", None);
    let debug = format!("{:?}", component.view());
    assert!(
        debug.contains("转到"),
        "package modal must contain an actionable path control: {debug}"
    );
    assert!(debug.contains("Popover") && debug.contains("Modal"), "must use the production modal primitive: {debug}");
    let files = tempfile::tempdir().unwrap();
    let selected = files.path().join("中文 book.txt");
    std::fs::write(&selected, "independent fixture").unwrap();
    assert!(component.on_with_input_for_instance_type("FileDialog", "SetPath", Some(files.path().to_str().unwrap().into())));
    assert!(component.on_with_input_for_instance_type("FileDialog", "Go", None));
    let deadline = Instant::now() + Duration::from_secs(5);
    let canonical = std::fs::canonicalize(&selected).unwrap();
    loop {
        component.on_with_input_for_instance_type("FileDialog", "Tick", None);
        let tree = format!("{:?}", component.view());
        if tree.contains("中文 book.txt") { break; }
        assert!(Instant::now() < deadline, "current IO result never rendered: {tree}");
        std::thread::sleep(Duration::from_millis(10));
    }
    let choose = format!("Choose\u{1f}s\u{1f}{}", canonical.to_str().unwrap());
    assert!(component.on_with_input_for_instance_type("FileDialog", &choose, None));
    assert!(component.on_with_input_for_instance_type("FileDialog", "Confirm", None));
    let tree = format!("{:?}", component.view());
    // This text belongs to the routed parent, not to the FileDialog projection.
    let expected = format!("{:?}", format!("选择结果 {}", canonical.display()));
    assert!(tree.contains(&expected), "accepted event must mutate the actual routed parent, not root state or the dialog preview: {tree}");
    component.on_with_input_for_instance_type("FileDialog", "Tick", None);
    component.on_with_input_for_instance_type("selector", "Open", None);
    let _ = component.view();
    component.on_with_input_for_instance_type("FileDialog", "Tick", None);
    component.on_with_input_for_instance_type("FileDialog", "Cancel", None);
    let tree = format!("{:?}", component.view());
    assert!(tree.contains("取消次数 1"), "cancel must reach the routed parent exactly once: {tree}");
    component.on_with_input_for_instance_type("FileDialog", "Cancel", None);
    let tree = format!("{:?}", component.view());
    assert!(tree.contains("取消次数 1"), "duplicate cancellation must not publish twice: {tree}");
}

#[test]
fn nested_callbacks_are_bound_to_their_parent_instance_and_app() {
    use auto_ui_runtime::Component;
    let source = r#"
widget Root {
    view {
        col {
            Parent (key: "left", tag: "left") {}
            Parent (key: "right", tag: "right") {}
        }
    }
}
widget Parent (tag: str) {
    msg { Got(str), Other(str) }
    model { var got str = "" }
    view {
        col {
            text f"parent:${.tag}:${.got}"
            Emitter (tag: .tag, onemit: .Got($event)) {}
        }
    }
    on {
        .Got(value str) -> { .got = value }
        .Other(value str) -> { .got = "WRONG" }
    }
}
widget Emitter (tag: str) {
    msg { Fire, emit(str) }
    view { button "emit" { onclick: .Fire } }
    on { .Fire -> { ."emit"(.tag) } }
}
"#;
    let mut a = auto_ui_runtime::build_dynamic_component(source, None).unwrap();
    let _ = a.view();
    let mut b = auto_ui_runtime::build_dynamic_component(
        &source.replace("onemit: .Got($event)", "onemit: .Other($event)"), None).unwrap();
    let _ = b.view();
    a.on_with_input_for_instance("Emitter", "Fire", None, Some("/Parent#left/Emitter#0"));
    let view = format!("{:?}", a.view());
    assert!(view.contains("parent:left:left"), "callback must reach its own parent: {view}");
    assert!(view.contains("content: \"parent:right:\""), "sibling must remain unchanged: {view}");
    assert!(!view.contains("WRONG"), "another App's callback must not replace this App's binding");
}
