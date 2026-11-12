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
    use auto_ui_runtime::ui::dynamic::DynamicComponent;
    use auto_ui_runtime::ui::{
        interpreter::DynamicMessage,
        view::View,
    };
    use auto_ui_runtime::Component;
    fn modal_open(view: &View<DynamicMessage>) -> bool {
        match view {
            View::Popover {
                placement, open, ..
            } if placement.is_modal_chrome() => *open,
            View::Column { children, .. } | View::Row { children, .. } => {
                children.iter().any(modal_open)
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => modal_open(child),
            _ => false,
        }
    }
    fn checkboxes(view: &View<DynamicMessage>, out: &mut Vec<DynamicMessage>) {
        match view {
            View::Checkbox {
                on_toggle: Some(message), label, ..
            } if label.ends_with(".txt") => out.push(message.clone()),
            View::Popover {
                open: true,
                content,
                ..
            } => checkboxes(content, out),
            View::Column { children, .. } | View::Row { children, .. } => {
                for child in children {
                    checkboxes(child, out);
                }
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => {
                checkboxes(child, out)
            }
            _ => {}
        }
    }
    fn wait(
        comp: &mut DynamicComponent,
        condition: impl Fn(&View<DynamicMessage>) -> bool,
    ) -> View<DynamicMessage> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            comp.on_with_input_for_instance_type("FileDialog", "Tick", None);
            let view = comp.view();
            if condition(&view) {
                return view;
            }
            assert!(
                Instant::now() < deadline,
                "fixture did not reach expected state: {view:?}"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    let app = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/file-dialog/src/front/app.at");
    let source = std::fs::read_to_string(&app).unwrap();
    let mut component = auto_ui_runtime::build_dynamic_component(&source, app.to_str()).unwrap();
    let _ = component.view();
    component.drive_scheduler_to_quiescence(16);
    component.on_with_input_for_instance_type("selector", "Open", None);
    let _ = component.view();
    component.drive_scheduler_to_quiescence(16);
    let view = wait(&mut component, modal_open);
    assert!(format!("{view:?}").contains("转到"));
    let files = tempfile::tempdir().unwrap();
    let selected = files.path().join("中文 book.txt");
    std::fs::write(&selected, "independent fixture").unwrap();
    std::fs::write(files.path().join("中文 book2.txt"), "second").unwrap();
    assert!(component.on_with_input_for_instance_type(
        "FileDialog",
        "SetPath",
        Some(files.path().to_str().unwrap().into())
    ));
    assert!(component.on_with_input_for_instance_type("FileDialog", "Go", None));
    let view = wait(&mut component, |view| {
        format!("{view:?}").contains("中文 book2.txt")
    });
    fn disabled_button(view: &View<DynamicMessage>, name: &str) -> Option<bool> {
        match view {
            View::Button { label, disabled, .. } if label == name => Some(*disabled),
            View::Popover { open: true, content, .. } => disabled_button(content, name),
            View::Column { children, .. } | View::Row { children, .. } => children.iter().find_map(|child| disabled_button(child, name)),
            View::Container { child, .. } | View::Scrollable { child, .. } => disabled_button(child, name),
            _ => None,
        }
    }
    for label in ["打开", "清空选择", "上一页", "下一页", "书籍"] {
        assert_eq!(disabled_button(&view, label), Some(true), "{label} must be disabled");
    }
    assert_eq!(disabled_button(&view, "图片"), Some(false));
    let mut actions = Vec::new();
    checkboxes(&view, &mut actions);
    assert_eq!(actions.len(), 2);
    // Replay two actual view-bound checkbox events without a timer between
    // them. A single undrained command slot would silently lose the first.
    for action in actions {
        let DynamicMessage::TypedInst {
            widget_name,
            event_name,
            args,
            instance,
        } = action
        else {
            panic!("checkbox event lost its instance");
        };
        let [auto_val::Value::Str(id)] = args.as_slice() else {
            panic!("checkbox row id missing: {args:?}");
        };
        let event = format!("{event_name}\u{1f}s\u{1f}{id}");
        component.on_with_input_for_instance(&widget_name, &event, None, Some(&instance));
    }
    let view = wait(&mut component, |view| {
        format!("{view:?}").contains("已选 2 项")
    });
    assert_eq!(disabled_button(&view, "打开"), Some(false));
    assert_eq!(disabled_button(&view, "清空选择"), Some(false));
    assert!(component.on_with_input_for_instance_type("FileDialog", "Confirm", None));
    let canonical = std::fs::canonicalize(&selected).unwrap();
    let expected = format!("{:?}", format!("选择结果 {}", canonical.display()));
    wait(&mut component, |view| {
        format!("{view:?}").contains(&expected)
    });
    // A real consumer can reopen as soon as the callback closes the parent;
    // no extra component clock is allowed to repair the missed closed edge.
    component.on_with_input_for_instance_type("selector", "Open", None);
    let _ = component.view();
    wait(&mut component, modal_open);
    component.on_with_input_for_instance_type("FileDialog", "Cancel", None);
    wait(&mut component, |view| {
        format!("{view:?}").contains("取消次数 1")
    });
    component.on_with_input_for_instance_type("FileDialog", "Cancel", None);
    for _ in 0..3 {
        component.on_with_input_for_instance_type("FileDialog", "Tick", None);
    }
    assert!(format!("{:?}", component.view()).contains("取消次数 1"));
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
    let b = auto_ui_runtime::build_dynamic_component(
        &source.replace("onemit: .Got($event)", "onemit: .Other($event)"),
        None,
    )
    .unwrap();
    let _ = b.view();
    a.on_with_input_for_instance("Emitter", "Fire", None, Some("/Parent#left/Emitter#0"));
    let view = format!("{:?}", a.view());
    assert!(
        view.contains("parent:left:left"),
        "callback must reach its own parent: {view}"
    );
    assert!(
        view.contains("content: \"parent:right:\""),
        "sibling must remain unchanged: {view}"
    );
    assert!(
        !view.contains("WRONG"),
        "another App's callback must not replace this App's binding"
    );
}

#[test]
fn each_file_dialog_clock_targets_its_real_instance() {
    use auto_ui_runtime::Component;
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("clock-owner.txt"), "bytes").unwrap();
    let package = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/file-dialog");
    let source = r#"
widget App {
    use { package: file_dialog from __PACKAGE__ }
    view {
        col {
            FileDialog (key: "first", open: true, initialDirectory: __DIRECTORY__) {}
            FileDialog (key: "second", open: false, initialDirectory: __DIRECTORY__) {}
        }
    }
}
"#.replace("__PACKAGE__", &serde_json::to_string(&package.to_str().unwrap().replace('\\', "/")).unwrap())
  .replace("__DIRECTORY__", &serde_json::to_string(temp.path().to_str().unwrap()).unwrap());
    let mut component = auto_ui_runtime::build_dynamic_component(&source, None).unwrap();
    let _ = component.view();
    let sources = component.subscribable_instance_timesources().into_iter().filter(|s| s.widget=="FileDialog").collect::<Vec<_>>();
    assert_eq!(sources.len(), 2);
    assert_ne!(sources[0].path, sources[1].path);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        for source in &sources { component.on_with_input_for(source.path.as_str(), &source.event, None); }
        let tree = format!("{:?}", component.view());
        if tree.contains("clock-owner.txt") { break; }
        assert!(Instant::now() < deadline, "active first owner was starved by the closed second instance: {tree}");
        std::thread::sleep(Duration::from_millis(2));
    }
}
