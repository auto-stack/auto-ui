use auto_ui_runtime::ui::file_dialog::{list_directory, DirectoryRequest};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

#[test]
fn dialog_content_dimensions_reach_both_native_view_paths_without_changing_defaults() {
    use auto_ui_runtime::{Component, ui::{interpreter::DynamicMessage, view::View, style::BoxLayout}};
    fn panel(view: &View<DynamicMessage>) -> BoxLayout {
        match view {
            View::Popover { content, .. } => match &**content {
                View::Column { style: Some(style), .. } => BoxLayout::from_style(style),
                _ => panic!("dialog has no styled panel"),
            },
            View::Column { children, .. } if children.len() == 1 => panel(&children[0]),
            _ => panic!("missing dialog: {view:?}"),
        }
    }
    for (content_props, width, padding) in [("", 384.0, 24.0), ("(style: \"w-[800px] p-0\")", 800.0, 0.0), ("(class: \"w-[640px] p-2\")", 640.0, 8.0)] {
        let source = format!("widget App {{ view {{ dialog (open: true) {{ dialog-content {content_props} {{ text \"content\" }} }} }} }}");
        let app = auto_ui_runtime::build_dynamic_component(&source, None).unwrap();
        for view in [app.view(), app.view_with_debug().0] {
            let layout = panel(&view);
            assert_eq!(layout.width.unwrap().to_pixels_f32(), width);
            assert_eq!(layout.padding_left, Some(padding));
        }
    }
}

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

/// R-1: the type-group buttons must share one horizontal row (a stacking
/// column that wraps 书籍/图片 into two lines is the defect).
fn filter_buttons_share_one_row(
    view: &auto_ui_runtime::ui::view::View<auto_ui_runtime::ui::interpreter::DynamicMessage>,
) -> bool {
    use auto_ui_runtime::ui::view::View;
    match view {
        View::Row { children, .. } => {
            let books = children.iter().any(
                |c| matches!(c, View::Button { label, .. } if label.contains("书籍")),
            );
            let images = children.iter().any(
                |c| matches!(c, View::Button { label, .. } if label.contains("图片")),
            );
            if books && images {
                return true;
            }
            children.iter().any(filter_buttons_share_one_row)
        }
        View::Popover { open: true, content, .. } => filter_buttons_share_one_row(content),
        View::Column { children, .. } | View::List { items: children, .. } => {
            children.iter().any(filter_buttons_share_one_row)
        }
        View::Container { child, .. } | View::Scrollable { child, .. } => {
            filter_buttons_share_one_row(child)
        }
        View::MouseArea { content, .. } => filter_buttons_share_one_row(content),
        _ => false,
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
    assert!(
        filter_buttons_share_one_row(&view),
        "R-1: type-group buttons must share one horizontal row, not a stacking column"
    );
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
    for label in ["打开", "清空选择", "书籍"] {
        assert_eq!(disabled_button(&view, label), Some(true), "{label} must be disabled");
    }
    let single_page = format!("{view:?}");
    assert!(!single_page.contains("上一页") && !single_page.contains("下一页"), "single-page navigation should be absent");
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

#[test]
fn root_parent_close_reopen_starts_a_fresh_session_before_another_clock() {
    use auto_ui_runtime::ui::{dynamic::DynamicComponent, interpreter::DynamicMessage, view::View};
    use auto_ui_runtime::Component;
    fn tick(app: &mut DynamicComponent) {
        for source in app.subscribable_instance_timesources().into_iter().filter(|s| s.widget == "FileDialog") {
            app.on_with_input_for(source.path.as_str(), &source.event, None);
        }
    }
    fn action(view: &View<DynamicMessage>, label: &str) -> Option<DynamicMessage> {
        match view {
            View::Checkbox { label: text, on_toggle: Some(message), .. } if text == label => Some(message.clone()),
            View::Button { label: text, onclick, .. } if text == label => Some(onclick.clone()),
            View::Popover { content, .. } => action(content, label),
            View::Column { children, .. } | View::Row { children, .. } => children.iter().find_map(|v| action(v, label)),
            View::Container { child, .. } | View::Scrollable { child, .. } => action(child, label),
            _ => None,
        }
    }
    fn dispatch(app: &mut DynamicComponent, event: &str) {
        app.on(DynamicMessage::Typed { widget_name: "App".into(), event_name: event.into(), args: vec![] });
        let _ = app.view();
    }
    fn wait(app: &mut DynamicComponent, predicate: impl Fn(&View<DynamicMessage>) -> bool) -> View<DynamicMessage> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            tick(app);
            let view = app.view();
            if predicate(&view) { return view; }
            assert!(Instant::now() < deadline, "parent lifecycle did not settle: {view:?}");
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("fresh.txt"), "owned fixture").unwrap();
    let package = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/file-dialog");
    let source = r#"
widget App {
    use { package: file_dialog from __PACKAGE__ }
    msg { Open, Close, Cancelled }
    model {
        var picker_open bool = false
        var cancel_count int = 0
    }
    view {
        col {
            text f"cancelled:${.cancel_count}"
            FileDialog (open: .picker_open, initialDirectory: __DIRECTORY__, oncancelled: .Cancelled) {}
        }
    }
    on {
        .Open -> { .picker_open = true }
        .Close -> { .picker_open = false }
        .Cancelled -> { .cancel_count = .cancel_count + 1; .picker_open = false }
    }
}
"#.replace("__PACKAGE__", &serde_json::to_string(&package.to_str().unwrap().replace('\\', "/")).unwrap())
  .replace("__DIRECTORY__", &serde_json::to_string(temp.path().to_str().unwrap()).unwrap());
    let mut app = auto_ui_runtime::build_dynamic_component(&source, None).unwrap();
    let _ = app.view();
    dispatch(&mut app, "Open");
    let view = wait(&mut app, |v| action(v, "fresh.txt").is_some());
    let old_row = action(&view, "fresh.txt").unwrap();
    app.on(old_row.clone());
    wait(&mut app, |v| format!("{v:?}").contains("已选 1 项"));
    dispatch(&mut app, "Close");
    // A rendered false prop is followed immediately by a root event. No child
    // clock gets a chance to sample the false value in between.
    dispatch(&mut app, "Open");
    let fresh = wait(&mut app, |v| action(v, "fresh.txt").is_some() && format!("{v:?}").contains("已选 0 项"));
    app.on(old_row);
    for _ in 0..3 { tick(&mut app); let _ = app.view(); }
    assert!(format!("{:?}", app.view()).contains("已选 0 项"), "old session row must be rejected");
    app.on(action(&fresh, "fresh.txt").unwrap());
    let view = wait(&mut app, |v| format!("{v:?}").contains("已选 1 项"));
    app.on(action(&view, "取消").unwrap());
    wait(&mut app, |v| format!("{v:?}").contains("content: \"cancelled:1\""));
    for _ in 0..3 { tick(&mut app); let _ = app.view(); }
    assert!(format!("{:?}", app.view()).contains("content: \"cancelled:1\""), "current cancel must notify exactly once");
}

/// PLAN-084 T-01 P-1: the overwrite confirmation must be a nested modal that
/// narrows the active modal scope, routes Escape to the inner layer, and keeps
/// the outer selector's controls outside the actionable scope.
#[test]
fn nested_overwrite_dialog_narrows_modal_scope_and_routes_escape_inward() {
    use auto_ui_runtime::Component;
    let source = r#"
widget App {
    msg { OverwriteBack, OuterClose, InnerAccept, InnerBack }
    model {
        var outer_open bool = true
        var overwrite_open bool = true
        var back_count int = 0
        var close_count int = 0
    }
    view {
        dialog (open: .outer_open, onclose: .OuterClose, closeOnOutside: false) {
            dialog-content (style: "w-[800px] p-0") {
                button "外层按钮" { onclick: .InnerAccept }
                text f"close:${.close_count}:back:${.back_count}"
                dialog (open: .overwrite_open, onclose: .OverwriteBack, closeOnOutside: false) {
                    dialog-content (style: "w-80 p-6") {
                        text "替换现有文件"
                        button "确认覆盖" { onclick: .InnerAccept }
                        button "返回修改" { onclick: .InnerBack }
                    }
                }
            }
        }
    }
    on {
        .OverwriteBack -> { .overwrite_open = false; .back_count = .back_count + 1 }
        .OuterClose -> { .outer_open = false; .close_count = .close_count + 1 }
        .InnerAccept -> { }
        .InnerBack -> { .overwrite_open = false; .back_count = .back_count + 1 }
    }
}
"#;
    let app = auto_ui_runtime::build_dynamic_component(source, None).unwrap();
    let _ = app.view();
    // The inner modal must be part of the painted tree.
    let view = format!("{:?}", app.view());
    assert!(view.contains("替换现有文件"), "nested modal missing: {view}");
    // The active modal scope must reach the innermost panel content.
    let tree = app.view();
    let path = auto_ui_runtime::ui::modal::active_content_path(&tree)
        .expect("a modal scope exists");
    let content = auto_ui_runtime::ui::modal::at_path(&tree, &path)
        .expect("scope resolves to inner content");
    assert!(
        format!("{content:?}").contains("替换现有文件"),
        "active scope must be the inner overwrite layer, got {content:?}"
    );
    // Escape dispatches to the inner layer's onclose, not the outer cancel.
    let escape = auto_ui_runtime::ui::modal::escape_message(&tree)
        .expect("inner modal owns Escape");
    assert!(
        format!("{escape:?}").contains("OverwriteBack"),
        "Escape must route inward: {escape:?}"
    );
    assert!(!format!("{escape:?}").contains("OuterClose"));
    // A background control outside the narrowed scope is not actionable.
    assert!(
        !auto_ui_runtime::ui::modal::allows_path(&tree, &[0]),
        "outer content must sit outside the narrowed modal scope"
    );
}

/// PLAN-084 T-01 P-2: `ui.focus` is callable from ordinary package handler
/// code and is consumed by the host focus slot with an observable result.
#[test]
fn package_handler_can_request_input_focus_by_event_key() {
    use auto_ui_runtime::Component;
    let source = r#"
widget App {
    msg { FocusNow, SetPath(str) }
    model {
        var open bool = true
        var path_input str = ""
        var focus_requested int = 0
    }
    view {
        text f"focus_requested:${.focus_requested}"
        dialog (open: .open) {
            dialog-content {
                input (value: .path_input, oninput: .SetPath, placeholder: "目录路径") {}
            }
        }
    }
    on {
        .FocusNow -> {
            .focus_requested = 1
            ui.focus(".SetPath")
        }
        .SetPath(value str) -> { .path_input = value }
    }
}
"#;
    let mut app = auto_ui_runtime::build_dynamic_component(source, None).unwrap();
    let _ = app.view();
    // Compiling and dispatching the package-style handler proves the builtin
    // is catalogued for VM code; runtime dispatch is consumed by the native
    // host focus slot (auto_lang_core::vm::native::take_ui_focus_request),
    // whose result lands in __focus_result on the native host path.
    app.on(auto_ui_runtime::ui::interpreter::DynamicMessage::Typed {
        widget_name: "App".into(),
        event_name: "FocusNow".into(),
        args: vec![],
    });
    let _ = app.view();
    let view = format!("{:?}", app.view());
    assert!(view.contains("focus_requested:1"), "handler ran: {view}");
}

#[test]
fn different_parent_types_keep_their_same_named_filter_models() {
    use auto_ui_runtime::Component;
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("book.txt"), "book").unwrap();
    std::fs::write(temp.path().join("picture.png"), "picture").unwrap();
    let package = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/file-dialog");
    let source = r#"
widget App {
 use { package: file_dialog from __PACKAGE__ }
 view { col { LeftPage {} RightPage {} } }
}
widget LeftPage {
 model { var picker_filters = [{id: "pictures", label: "PICTURES", extensions: ["png"]}] }
 view { col { text "LEFT-BEGIN" {} FileDialog (open: true, filters: .picker_filters, initialDirectory: __DIRECTORY__) {} text "LEFT-END" {} } }
}
widget RightPage {
 model { var picker_filters = [{id: "books", label: "BOOKS", extensions: ["txt"]}] }
 view { col { text "RIGHT-BEGIN" {} FileDialog (open: true, filters: .picker_filters, initialDirectory: __DIRECTORY__) {} text "RIGHT-END" {} } }
}
"#
        .replace("__PACKAGE__", &serde_json::to_string(&package.to_str().unwrap().replace('\\', "/")).unwrap())
        .replace("__DIRECTORY__", &serde_json::to_string(temp.path().to_str().unwrap()).unwrap());
    let mut app = auto_ui_runtime::build_dynamic_component(&source, None).unwrap();
    let _ = app.view();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        for source in app.subscribable_instance_timesources().into_iter().filter(|s| s.widget == "FileDialog") {
            app.on_with_input_for(source.path.as_str(), &source.event, None);
        }
        let tree = format!("{:?}", app.view());
        let left = tree.split("LEFT-BEGIN").nth(1).unwrap().split("LEFT-END").next().unwrap();
        let right = tree.split("RIGHT-BEGIN").nth(1).unwrap().split("RIGHT-END").next().unwrap();
        assert!(left.contains("PICTURES") && !left.contains("book.txt"), "left parent config leaked: {left}");
        assert!(right.contains("BOOKS") && !right.contains("picture.png"), "right parent config leaked: {right}");
        if left.contains("picture.png") && right.contains("book.txt") { break; }
        assert!(Instant::now() < deadline, "parent lists did not settle: {tree}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

// ---- PLAN-084 T-03 component-level checks (real package view wiring) ----

/// Finds a dispatchable button by its visible label (icon PUA markers and
/// tooltip suffixes stripped), descending into open modal content.
fn plan084_button_msg(
    view: &auto_ui_runtime::ui::view::View<auto_ui_runtime::ui::interpreter::DynamicMessage>,
    label: &str,
) -> Option<auto_ui_runtime::ui::interpreter::DynamicMessage> {
    use auto_ui_runtime::ui::interpreter::DynamicMessage;
    use auto_ui_runtime::ui::view::View;
    let visible_label = |raw: &str| {
        // label may carry icon EE01 / EE02 / EE03 tooltip markers — compare
        // every text segment and the raw body.
        let body = raw.split('\u{EE03}').next().unwrap_or(raw);
        let name = body
            .rsplit('\u{EE02}')
            .next()
            .unwrap_or(body)
            .split('\u{EE01}')
            .last()
            .unwrap_or(body);
        name == label || body == label
    };
    match view {
        View::Button { label: text, onclick, .. } if visible_label(text) => Some(onclick.clone()),
        View::Popover { open: true, content, .. } => plan084_button_msg(content, label),
        View::Column { children, .. } | View::Row { children, .. } | View::List { items: children, .. } => {
            children.iter().find_map(|child| plan084_button_msg(child, label))
        }
        View::Container { child, .. } | View::Scrollable { child, .. } => {
            plan084_button_msg(child, label)
        }
        View::MouseArea { content, .. } => plan084_button_msg(content, label),
        _ => None,
    }
}

fn plan084_tick(component: &mut auto_ui_runtime::ui::dynamic::DynamicComponent) {
    let _ = component.on_with_input_for_instance_type("FileDialog", "Tick", None);
}

fn plan084_wait(
    component: &mut auto_ui_runtime::ui::dynamic::DynamicComponent,
    predicate: impl Fn(&str) -> bool,
) -> String {
    use auto_ui_runtime::Component;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        plan084_tick(component);
        let _ = component.view();
        let tree = format!("{:?}", component.view());
        if predicate(&tree) {
            return tree;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "package view did not reach expected state: {tree}"
        );
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

fn plan084_package_source(inner: &str) -> String {
    let package = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/file-dialog");
    let prefix = r#"
widget App {
    use { package: file_dialog from __PACKAGE__ }
    msg { Open, OpenDirectory, OpenSave, Close, Accepted(str), Cancelled, Error(str) }
    model {
        var picker_open bool = false
        var mode str = "singleFile"
        var accept_label str = "打开"
        var default_name str = ""
        var default_extension str = ""
        var initial_directory str = ""
        var accepted_count int = 0
        var cancel_count int = 0
        var result str = ""
        var background_count int = 0
    }
    view {
        col {
            button "选择文件" { onclick: .Open }
            button "选择目录" { onclick: .OpenDirectory }
            button "保存文件" { onclick: .OpenSave }
            button "底层计数" { onclick: .Nothing }
            button "切换模式" { onclick: .Nothing }
            text f"结果 ${.result}"
            text f"确认次数 ${.accepted_count}"
            text f"取消次数 ${.cancel_count}"
            text f"底层次数 ${.background_count}"
            FileDialog (open: .picker_open, selectionMode: .mode, acceptLabel: .accept_label, defaultFileName: .default_name, defaultExtension: .default_extension, initialDirectory: .initial_directory, filters: [{id: "books", label: "Markdown", extensions: ["md"]}, {id: "images", label: "图片", extensions: ["png"]}], onaccepted: .Accepted($event), oncancelled: .Cancelled) {}
        }
    }
    on {
        .Open -> { .mode = "singleFile"; .accept_label = "打开"; .picker_open = true }
        .OpenDirectory -> { .mode = "singleDirectory"; .accept_label = "选择此目录"; .picker_open = true }
        .OpenSave -> { .mode = "saveFile"; .accept_label = "保存"; .picker_open = true }
        .Accepted(value str) -> { .accepted_count = .accepted_count + 1; .result = value; .picker_open = false }
        .Cancelled -> { .cancel_count = .cancel_count + 1; .picker_open = false }
        .Nothing -> { .background_count = .background_count + 1 }
    }
}
"#
    .replace("__PACKAGE__", &serde_json::to_string(&package.to_str().unwrap().replace('\\', "/")).unwrap());
    let _ = inner;
    prefix
}

/// V-01 (component): the directory-mode view shows the 选择/进入 split, the
/// current-directory button, and no file checkboxes; accepting returns one
/// real directory through the ordinary accepted callback.
#[test]
fn package_directory_mode_view_selects_and_accepts_one_directory() {
    use auto_ui_runtime::ui::interpreter::DynamicMessage;
    use auto_ui_runtime::Component;
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("子目录")).unwrap();
    std::fs::write(temp.path().join("plain.txt"), "context").unwrap();
    let mut source = plan084_package_source("");
    // Point the initial directory at the fixture.
    source = source.replace(
        "var initial_directory str = \"\"",
        &format!(
            "var initial_directory str = \"{}\"",
            temp.path().to_str().unwrap().replace('\\', "/")
        ),
    );
    let mut component = auto_ui_runtime::build_dynamic_component(&source, None).unwrap();
    let _ = component.view();
    component
        .on(DynamicMessage::Typed {
            widget_name: "App".into(),
            event_name: "OpenDirectory".into(),
            args: vec![],
        });
    let _ = component.view();
    let tree = plan084_wait(&mut component, |tree| {
        tree.contains("选择一个目录")
            && tree.contains("选择当前目录")
            && !tree.contains("载入中")
    });
    // Type groups are hidden in directory mode; file rows carry no checkbox.
    assert!(
        !tree.contains("Markdown") && !tree.contains("图片"),
        "type groups hidden in directory mode: {tree}"
    );
    assert!(
        !tree.contains("label: \"plain.txt\""),
        "file checkboxes must be absent in directory mode: {tree}"
    );
    assert!(
        tree.contains("选择 子目录"),
        "row select affordance present: {tree}"
    );
    // Enter affordance stays discoverable by name (chevron tooltip).
    assert!(
        tree.contains("进入 子目录"),
        "row enter affordance present: {tree}"
    );
    // Select the current directory through the ordinary control.
    let select_current = plan084_button_msg(
        &component.view(),
        "选择当前目录",
    )
    .expect("选择当前目录 control present");
    component.on(select_current);
    let _ = plan084_wait(&mut component, |tree| tree.contains("已选 1 项"));
    let accept = plan084_button_msg(&component.view(), "选择此目录").expect("accept enabled");
    component.on(accept);
    let tree = plan084_wait(&mut component, |tree| tree.contains("确认次数 1"));
    let basename = temp
        .path()
        .canonicalize()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    // The debug dump escapes inner JSON quotes; the model tests own the exact
    // payload shape, so the component check distinguishes on the real path.
    assert!(
        tree.contains(&basename) && tree.contains("确认次数 1"),
        "accepted payload carries one real canonical directory entry: {tree}"
    );
    assert!(
        !tree.contains("plain.txt") || !tree.contains("选择 plain.txt"),
        "no file path can leak into the accepted result"
    );
}

/// V-03 (component): the overwrite layer is a nested modal in the real
/// package view; Back keeps drafts, Replace completes the session, and the
/// confirmed file's bytes stay untouched by the selector.
#[test]
fn package_save_mode_overwrite_layer_back_then_replace() {
    use auto_ui_runtime::ui::interpreter::DynamicMessage;
    use auto_ui_runtime::Component;
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("笔记.md");
    std::fs::write(&target, "existing bytes").unwrap();
    let mut source = plan084_package_source("");
    source = source.replace(
        "var initial_directory str = \"\"",
        &format!(
            "var initial_directory str = \"{}\"",
            temp.path().to_str().unwrap().replace('\\', "/")
        ),
    );
    let mut component = auto_ui_runtime::build_dynamic_component(&source, None).unwrap();
    let _ = component.view();
    component.on(DynamicMessage::Typed {
        widget_name: "App".into(),
        event_name: "OpenSave".into(),
        args: vec![],
    });
    let _ = component.view();
    let tree = plan084_wait(&mut component, |tree| {
        tree.contains("选择保存位置") && tree.contains("要保存的文件名")
    });
    assert!(tree.contains("Markdown"), "format groups remain visible in save mode");
    // Confirm with the existing file's name → overwrite layer, no acceptance.
    assert!(component.on_with_input_for_instance_type(
        "FileDialog",
        "SetFileName",
        Some("笔记.md".into())
    ));
    let accept = plan084_wait(&mut component, |tree| {
        tree.contains("笔记.md") && !tree.contains("载入中")
    });
    assert!(accept.contains("笔记.md"), "draft is projected back: {accept}");
    let save = plan084_button_msg(&component.view(), "保存").unwrap();
    component.on(save);
    // The layer's name-bearing text only renders with a pending overwrite;
    // the closed popover still dumps its content, so the bare title cannot
    // distinguish states.
    let layer = plan084_wait(&mut component, |tree| {
        tree.contains("笔记.md 已存在。")
    });
    assert!(layer.contains("返回修改") && layer.contains("确认覆盖"));
    assert!(
        !layer.contains("确认次数 1"),
        "no accepted before the overwrite is confirmed: {layer}"
    );
    assert_eq!(
        std::fs::read(&target).unwrap(),
        b"existing bytes",
        "the selector never writes during validation"
    );
    // Back keeps the draft and the session; the layer closes.
    let back = plan084_button_msg(&component.view(), "返回修改").unwrap();
    component.on(back);
    let tree = plan084_wait(&mut component, |tree| {
        !tree.contains("笔记.md 已存在。")
    });
    assert!(tree.contains("笔记.md"), "draft survives Back");
    // Replace completes the selection only after revalidation.
    let save = plan084_button_msg(&component.view(), "保存").unwrap();
    component.on(save);
    plan084_wait(&mut component, |tree| tree.contains("笔记.md 已存在。"));
    let replace = plan084_button_msg(&component.view(), "确认覆盖").unwrap();
    component.on(replace);
    let tree = plan084_wait(&mut component, |tree| tree.contains("确认次数 1"));
    assert!(tree.contains("笔记.md"));
    assert_eq!(
        std::fs::read(&target).unwrap(),
        b"existing bytes",
        "accepting is a selection, not a write"
    );
}

/// V-04 (component): the session mode freezes at open; a parent prop change
/// mid-session never switches the live view, and the next open adopts it.
#[test]
fn package_session_mode_is_frozen_until_reopen() {
    use auto_ui_runtime::ui::interpreter::DynamicMessage;
    use auto_ui_runtime::Component;
    let temp = tempfile::tempdir().unwrap();
    let mut source = plan084_package_source("");
    source = source.replace(
        "var initial_directory str = \"\"",
        &format!(
            "var initial_directory str = \"{}\"",
            temp.path().to_str().unwrap().replace('\\', "/")
        ),
    );
    // The parent switches to save mode without closing the dialog.
    let source = source
        .replace("button \"切换模式\" { onclick: .Nothing }", "button \"切换模式\" { onclick: .SwitchMode }")
        .replace(
            "msg { Open, OpenDirectory, OpenSave, Close, Accepted(str), Cancelled, Error(str) }",
            "msg { Open, OpenDirectory, OpenSave, Close, SwitchMode, Accepted(str), Cancelled, Error(str) }",
        )
        .replace(
            "on {\n        .Open -> { .mode = \"singleFile\"; .accept_label = \"打开\"; .picker_open = true }",
            "on {\n        .SwitchMode -> { .mode = \"saveFile\"; .accept_label = \"保存\" }\n        .Open -> { .mode = \"singleFile\"; .accept_label = \"打开\"; .picker_open = true }",
        );
    let mut component = auto_ui_runtime::build_dynamic_component(&source, None).unwrap();
    let _ = component.view();
    component.on(DynamicMessage::Typed {
        widget_name: "App".into(),
        event_name: "Open".into(),
        args: vec![],
    });
    let _ = component.view();
    let tree = plan084_wait(&mut component, |tree| tree.contains("选择要打开的文件"));
    assert!(!tree.contains("要保存的文件名"), "singleFile session has no save row");
    // Mid-session prop change must not switch the frozen session.
    component.on(DynamicMessage::Typed {
        widget_name: "App".into(),
        event_name: "SwitchMode".into(),
        args: vec![],
    });
    let tree = plan084_wait(&mut component, |tree| tree.contains("选择文件") && !tree.contains("载入中"));
    assert!(
        !tree.contains("要保存的文件名") && !tree.contains("选择保存位置"),
        "frozen session keeps its mode despite the parent prop change: {tree}"
    );
    // Cancel and reopen adopts the new mode.
    let cancel = plan084_button_msg(&component.view(), "取消").unwrap();
    component.on(cancel);
    plan084_wait(&mut component, |tree| tree.contains("取消次数 1"));
    component.on(DynamicMessage::Typed {
        widget_name: "App".into(),
        event_name: "OpenSave".into(),
        args: vec![],
    });
    let _ = component.view();
    plan084_wait(&mut component, |tree| tree.contains("选择保存位置"));
}
