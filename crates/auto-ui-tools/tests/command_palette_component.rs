use auto_lang_core::{parser::Parser, session::CompilerSession};
use auto_ui_runtime::ui::{
    component::Component, dynamic::DynamicComponent, widget_registry::WidgetRegistry,
};
use auto_val::Value;
use serde_json::{json, Value as Json};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const APP: &str = r#"
widget App {
    msg { OnQuery(str), OnInvoke(str), OnClose(str), OnConfig(str) }
    model {
        var mounted bool = true
        var shown bool = true
        var mode str = "local"
        var wrap bool = false
        var version int = 0
        var selected_query str = ""
        var result_session int = 0
        var result_seq int = 0
        var result_query str = ""
        var reply str = ""
        var queries int = 0
        var invocations int = 0
        var closures int = 0
        var errors int = 0
        var latest_query str = ""
        var request str = ""
        var diagnostic str = ""
        var choices List = [
            {id: "first", label: "Dark theme", groupId: "a", groupLabel: "Appearance", keywords: ["深色"], shortcut: "", disabled: false, icon: ""},
            {id: "disabled", label: "Disabled theme", groupId: "a", groupLabel: "Appearance", keywords: [], shortcut: "", disabled: true, icon: ""},
            {id: "second", label: "Theme", groupId: "a", groupLabel: "Appearance", keywords: ["appearance"], shortcut: "", disabled: false, icon: ""},
            {id: "third", label: "Theme", groupId: "b", groupLabel: "Settings", keywords: [], shortcut: "", disabled: false, icon: ""},
            {id: "other", label: "Open file", groupId: "", groupLabel: "", keywords: [], shortcut: "", disabled: false, icon: ""}
        ]
    }
    view {
        col {
            if .mounted {
                CommandPalette (open: .shown, commands: .choices, query: .selected_query, filterMode: .mode,
                    revision: .version, wrapNavigation: .wrap, resultsSession: .result_session,
                    resultsQuerySeq: .result_seq, resultsQuery: .result_query, response: .reply,
                    onqueryChanged: .OnQuery, oninvokeRequested: .OnInvoke, oncloseRequested: .OnClose,
                    onerror: .OnConfig) {}
            }
        }
    }
    on {
        .OnQuery(payload) -> {
            .queries = .queries + 1
            .latest_query = payload
        }
        .OnInvoke(payload) -> {
            .invocations = .invocations + 1
            .request = payload
        }
        .OnClose(payload) -> { .closures = .closures + 1 }
        .OnConfig(payload) -> {
            .errors = .errors + 1
            .diagnostic = payload
        }
    }
}
"#;

fn model() -> DynamicComponent {
    mount(APP)
}

fn mount(app_source: &str) -> DynamicComponent {
    let package = repository().join("packages/command-palette");
    let data = fs::read_to_string(package.join("palette_data.at")).unwrap();
    let search = fs::read_to_string(package.join("palette_search.at"))
        .unwrap()
        .replace("use palette_data: PaletteCommand\n", "");
    let palette = fs::read_to_string(package.join("command_palette.at")).unwrap();
    let palette = palette
        .lines()
        .filter(|line| !line.starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!("{data}\n{search}\n{app_source}\n{palette}");
    let ast = Parser::from(source.as_str())
        .with_session(CompilerSession::ui())
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .unwrap();
    let ui = auto_ui_compiler::ui_ast::lower_ui(&ast).unwrap().ui;
    let declarations: Vec<_> = ui
        .domain_stmts()
        .filter_map(|statement| match statement {
            auto_ui_dialect::UiStmt::WidgetDecl(declaration) => Some(declaration.clone()),
            _ => None,
        })
        .collect();
    let app = auto_ui_compiler::aura::extract_widget_from_decl(&declarations[0]).unwrap();
    let mut registry = WidgetRegistry::new();
    registry.register(auto_ui_compiler::aura::extract_widget_from_decl(&declarations[1]).unwrap());
    let component = DynamicComponent::with_registry_and_imports_from_decls(
        &declarations[0],
        &declarations[1..],
        &app,
        registry,
        ui.core_stmts().cloned().collect(),
        &HashMap::new(),
        false,
    )
    .unwrap();
    let _ = component.view();
    component
}

fn instance(model: &DynamicComponent) -> String {
    let instances = model
        .bridge()
        .child_instance_identities_of_type("CommandPalette");
    assert_eq!(instances.len(), 1);
    instances[0].clone()
}

fn child(model: &DynamicComponent, field: &str) -> Value {
    model
        .bridge()
        .read_instance_state(&instance(model), field)
        .unwrap()
}

fn tick(model: &mut DynamicComponent) {
    let _ = model.view();
    let id = instance(model);
    model.on_with_input_for_instance("CommandPalette", "Tick", None, Some(&id));
    let _ = model.view();
}

fn action(model: &mut DynamicComponent, event: &str, values: &[Value]) {
    model
        .call_widget_handler("CommandPalette", event, values)
        .unwrap();
    let _ = model.view();
}

fn edit(model: &mut DynamicComponent, text: &str) {
    let binding = child(model, "binding");
    action(model, "Edit", &[binding, Value::str(text)]);
}

fn invoke(model: &mut DynamicComponent, id: &str) {
    let binding = child(model, "binding");
    let epoch = child(model, "result_epoch");
    action(model, "Invoke", &[binding, epoch, Value::str(id)]);
}

fn set(model: &mut DynamicComponent, name: &str, value: Value) {
    model.write_state(name, value).unwrap();
}

fn emitted(model: &DynamicComponent, field: &str) -> Json {
    serde_json::from_str(model.read_state(field).unwrap().as_str()).unwrap()
}

#[test]
fn local_keeps_selected_identity_and_ignores_disabled_empty_and_old_rows() {
    let mut model = model();
    tick(&mut model);
    assert_eq!(child(&model, "active_id"), Value::str("first"));
    let binding = child(&model, "binding");
    let old_epoch = child(&model, "result_epoch");
    edit(&mut model, " \tThEmE\r\n");
    assert_eq!(child(&model, "active_id"), Value::str("first"));
    let current_epoch = child(&model, "result_epoch");
    action(
        &mut model,
        "Invoke",
        &[binding.clone(), old_epoch, Value::str("first")],
    );
    invoke(&mut model, "disabled");
    assert_eq!(child(&model, "seq"), Value::Int(0));
    action(
        &mut model,
        "Move",
        &[binding.clone(), current_epoch.clone(), Value::str("up")],
    );
    assert_eq!(child(&model, "active_id"), Value::str("second"));
    action(
        &mut model,
        "Move",
        &[binding.clone(), current_epoch.clone(), Value::str("up")],
    );
    assert_eq!(child(&model, "active_id"), Value::str("second"));
    set(&mut model, "wrap", Value::Bool(true));
    tick(&mut model);
    action(
        &mut model,
        "Move",
        &[binding, current_epoch, Value::str("up")],
    );
    assert_eq!(child(&model, "active_id"), Value::str("third"));
    edit(&mut model, "深色");
    assert_eq!(child(&model, "active_id"), Value::str("first"));
    edit(&mut model, "empty");
    assert_eq!(child(&model, "active_id"), Value::str(""));
    invoke(&mut model, "first");
    assert_eq!(child(&model, "seq"), Value::Int(0));
}

#[test]
fn provided_requires_the_current_request_even_when_query_text_repeats() {
    let mut model = model();
    set(&mut model, "mode", Value::str("provided"));
    set(&mut model, "selected_query", Value::str("A"));
    tick(&mut model);
    let first = child(&model, "query_seq");
    edit(&mut model, "B");
    edit(&mut model, "A");
    let third = child(&model, "query_seq");
    assert_ne!(first, third);
    set(&mut model, "version", Value::Int(99));
    set(&mut model, "result_session", Value::Int(1));
    set(&mut model, "result_seq", first);
    set(&mut model, "result_query", Value::str("A"));
    tick(&mut model);
    invoke(&mut model, "first");
    assert_eq!(child(&model, "seq"), Value::Int(0));
    assert_eq!(child(&model, "result_revision"), Value::Int(0));

    set(&mut model, "version", Value::Int(1));
    set(&mut model, "result_seq", third);
    tick(&mut model);
    assert_eq!(child(&model, "active_id"), Value::str("first"));
    invoke(&mut model, "second");
    assert_eq!(child(&model, "phase"), Value::str("pending"));
    invoke(&mut model, "third");
    edit(&mut model, "ignored");
    let binding = child(&model, "binding");
    action(&mut model, "Close", &[binding, Value::str("escape")]);
    assert_eq!(child(&model, "draft"), Value::str("A"));
    assert_eq!(child(&model, "seq"), Value::Int(1));
}

#[test]
fn feedback_is_tied_to_request_data_and_mount() {
    let mut model = model();
    tick(&mut model);
    invoke(&mut model, "first");
    let old_token = child(&model, "instance_token");
    let response = json!({
        "instanceToken": old_token.as_str(), "session": 1, "seq": 1, "commandId": "first",
        "status": "rejected", "message": "Permission changed"
    });
    set(&mut model, "reply", Value::str(&response.to_string()));
    tick(&mut model);
    assert_eq!(child(&model, "phase"), Value::str("editing"));
    assert_eq!(child(&model, "rejection"), Value::str("Permission changed"));
    let accepted = json!({
        "instanceToken": old_token.as_str(), "session": 1, "seq": 2, "commandId": "first",
        "status": "accepted", "message": ""
    });
    invoke(&mut model, "first");
    set(&mut model, "reply", Value::str(&accepted.to_string()));
    set(&mut model, "version", Value::Int(1));
    tick(&mut model);
    assert_eq!(child(&model, "phase"), Value::str("accepted"));
    invoke(&mut model, "second");
    assert_eq!(child(&model, "seq"), Value::Int(2));

    set(&mut model, "mounted", Value::Bool(false));
    let _ = model.view();
    assert!(model
        .bridge()
        .child_instance_identities_of_type("CommandPalette")
        .is_empty());
    set(&mut model, "mounted", Value::Bool(true));
    set(&mut model, "reply", Value::str(""));
    tick(&mut model);
    assert_ne!(child(&model, "instance_token"), old_token);
    invoke(&mut model, "first");
    assert_eq!(child(&model, "seq"), Value::Int(1));
    set(&mut model, "reply", Value::str(&response.to_string()));
    tick(&mut model);
    assert_eq!(child(&model, "phase"), Value::str("pending"));
}

#[test]
fn first_invalid_configuration_notifies_once_and_can_recover() {
    let mut model = model();
    let good = model.read_state_as_vec("choices").unwrap();
    let duplicate = good[0].clone();
    model
        .write_state_vec("choices", vec![duplicate.clone(), duplicate])
        .unwrap();
    tick(&mut model);
    assert_eq!(
        child(&model, "config_code"),
        Value::str("duplicate-command-id")
    );
    tick(&mut model);
    assert_eq!(model.read_state("errors").unwrap(), Value::Int(1));
    assert_eq!(
        emitted(&model, "diagnostic")["code"],
        "duplicate-command-id"
    );
    invoke(&mut model, "first");
    assert_eq!(child(&model, "seq"), Value::Int(0));
    model.write_state_vec("choices", good).unwrap();
    tick(&mut model);
    invoke(&mut model, "first");
    assert_eq!(child(&model, "seq"), Value::Int(1));
}

#[test]
fn invalid_records_report_missing_id_and_conflicting_group_labels() {
    for (source, expected) in [
        (APP.replace("id: \"first\", ", ""), "missing-command-id"),
        (
            APP.replacen("groupLabel: \"Appearance\"", "groupLabel: \"Mismatch\"", 1),
            "conflicting-group-label",
        ),
    ] {
        let mut model = mount(&source);
        tick(&mut model);
        assert_eq!(emitted(&model, "diagnostic")["code"], expected);
        invoke(&mut model, "third");
        assert_eq!(child(&model, "seq"), Value::Int(0));
    }
}

#[test]
fn omitted_query_keeps_its_default_with_a_same_named_parent_field() {
    let mut model = mount(
        &APP.replace(
            "var selected_query str = \"\"",
            "var query str = \"parent field\"",
        )
        .replace("query: .selected_query, ", ""),
    );
    tick(&mut model);
    assert_eq!(child(&model, "draft"), Value::str(""));
    assert_eq!(emitted(&model, "latest_query")["query"], "");
    assert_eq!(
        model.read_state("query").unwrap(),
        Value::str("parent field")
    );
}

#[test]
fn same_revision_command_change_invalidates_a_rendered_action() {
    let mut model = model();
    tick(&mut model);
    let original_epoch = child(&model, "result_epoch");
    let binding = child(&model, "binding");
    let mut commands = model.read_state_as_vec("choices").unwrap();
    commands.remove(0);
    model.write_state_vec("choices", commands).unwrap();
    let _ = model.view();
    action(
        &mut model,
        "Invoke",
        &[binding, original_epoch, Value::str("second")],
    );
    assert_eq!(child(&model, "seq"), Value::Int(0));
    tick(&mut model);
    assert_eq!(child(&model, "active_id"), Value::str("second"));
    invoke(&mut model, "second");
    assert_eq!(child(&model, "seq"), Value::Int(1));
}

#[test]
fn two_same_title_mounts_keep_independent_request_identity() {
    let mut model = mount(&APP.replace(
        "onerror: .OnConfig) {}",
        "onerror: .OnConfig) {}\n                CommandPalette (open: .shown, commands: .choices) {}",
    ));
    let mounts = model
        .bridge()
        .child_instance_identities_of_type("CommandPalette");
    assert_eq!(mounts.len(), 2);
    for id in &mounts {
        model.on_with_input_for_instance("CommandPalette", "Tick", None, Some(id));
    }
    let _ = model.view();
    let read = |model: &DynamicComponent, id: &str, field| {
        model.bridge().read_instance_state(id, field).unwrap()
    };
    let token_a = read(&model, &mounts[0], "instance_token");
    let token_b = read(&model, &mounts[1], "instance_token");
    assert_ne!(token_a, token_b);
    let binding_a = read(&model, &mounts[0], "binding");
    let binding_b = read(&model, &mounts[1], "binding");
    let epoch_b = read(&model, &mounts[1], "result_epoch");
    let state_b = model.bridge().child_instance_state_id(&mounts[1]).unwrap();
    model
        .bridge_mut()
        .call_handler_for(
            "CommandPalette",
            "Invoke",
            state_b,
            &[binding_a, epoch_b.clone(), Value::str("first")],
        )
        .unwrap();
    assert_eq!(read(&model, &mounts[1], "seq"), Value::Int(0));
    model
        .bridge_mut()
        .call_handler_for(
            "CommandPalette",
            "Invoke",
            state_b,
            &[binding_b, epoch_b, Value::str("first")],
        )
        .unwrap();
    assert_eq!(read(&model, &mounts[1], "seq"), Value::Int(1));
    assert_eq!(read(&model, &mounts[0], "seq"), Value::Int(0));
}

#[test]
fn relative_package_generates_after_relocation() {
    let root = std::env::var_os("PLAN096_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("D:/autostack/plan-artifacts/plan-096"));
    fs::create_dir_all(&root).unwrap();
    let scratch = tempfile::Builder::new()
        .prefix("moved-palette-")
        .tempdir_in(root)
        .unwrap();
    let moved = scratch.path();
    let source = repository();
    for (directory, files) in [
        (
            "examples/command-palette",
            &["pac.at", "src/front/app.at"][..],
        ),
        (
            "packages/command-palette",
            &[
                "pac.at",
                "package.at",
                "command_palette.at",
                "palette_data.at",
                "palette_search.at",
            ][..],
        ),
        (
            "packages/themes",
            &["pac.at", "theme-package.json", "themes.at"][..],
        ),
    ] {
        for file in files {
            let relative = Path::new(directory).join(file);
            let destination = moved.join(&relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(source.join(relative), destination).unwrap();
        }
    }
    let app = moved.join("examples/command-palette");
    let descriptor = auto_project_model::parse_file(&app.join("pac.at")).unwrap();
    let request = auto_project_model::GenerationRequest::from_descriptor(&descriptor);
    let mut project =
        auto_ui_tools::vue::VueProject::from_workspace_with_request(&app, &request).unwrap();
    project.output_dir = moved.join("vue");
    project.generate().unwrap();
    let app = fs::read_to_string(project.output_dir.join("src/App.vue")).unwrap();
    let view =
        fs::read_to_string(project.output_dir.join("src/components/CommandPalette.vue")).unwrap();
    assert!(app.contains("<CommandPalette ") && app.contains("@invokeRequested=\"ApplyCommand\""));
    assert!(view.contains("function palette_local(") && view.contains("@keydown.down.prevent="));
    assert!(view.contains(":close-on-outside=\"false\""));
    fs::rename(
        moved.join("packages/command-palette"),
        moved.join("withdrawn-palette"),
    )
    .unwrap();
    let options = auto_ui_tools::tool_ops::GenerateOptions {
        strict: Some(true),
        ..Default::default()
    };
    let error = auto_ui_tools::tool_ops::generate_report_with(&request, &options)
        .expect_err("a missing palette package must fail strict generation");
    assert!(error.to_string().contains("command-palette"), "{error}");
}
