use auto_lang_core::{parser::Parser, session::CompilerSession};
use auto_ui_runtime::ui::{
    component::Component, dynamic::DynamicComponent, widget_registry::WidgetRegistry,
};
use auto_val::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn mount_model() -> DynamicComponent {
    let consumer = r#"
widget App {
    model {
        var mounted bool = true
        var shown bool = true
        var target str = "notebook-7"
        var revision int = 0
        var initial str = "Inbox"
        var response str = ""
    }
    view {
        col {
            if .mounted {
                PromptDialog (open: .shown, contextId: .target, revision: .revision,
                    initialValue: .initial, response: .response) {}
            }
        }
    }
}
"#;
    let mut component = mount_prompt(consumer);
    act(&mut component, "Tick", &[]);
    component
}

fn mount_prompt(consumer: &str) -> DynamicComponent {
    let package = repository().join("packages/prompt-dialog");
    let model = fs::read_to_string(package.join("prompt_dialog.at")).unwrap();
    let text = fs::read_to_string(package.join("prompt_text.at")).unwrap();
    let source = format!(
        "{text}\n{consumer}\n{}",
        model.replace("use prompt_text: prompt_trim\n", "")
    );
    mount_source(&source)
}

fn mount_source(source: &str) -> DynamicComponent {
    let ast = Parser::from(source)
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
    let widget = auto_ui_compiler::aura::extract_widget_from_decl(&declarations[0]).unwrap();
    let mut registry = WidgetRegistry::new();
    for declaration in &declarations[1..] {
        registry.register(auto_ui_compiler::aura::extract_widget_from_decl(declaration).unwrap());
    }
    let mut component = DynamicComponent::with_registry_and_imports_from_decls(
        &declarations[0],
        &declarations[1..],
        &widget,
        registry,
        ui.core_stmts().cloned().collect(),
        &HashMap::new(),
        false,
    )
    .unwrap();
    let _ = component.view();
    component
}

fn act(model: &mut DynamicComponent, name: &str, values: &[Value]) {
    model
        .call_widget_handler("PromptDialog", name, values)
        .unwrap();
}

fn read(model: &DynamicComponent, field: &str) -> Value {
    let bridge = model.bridge();
    let instances = bridge.child_instance_identities_of_type("PromptDialog");
    assert_eq!(instances.len(), 1);
    bridge.read_instance_state(&instances[0], field).unwrap()
}

fn read_at(model: &DynamicComponent, id: &str, field: &str) -> Value {
    model.bridge().read_instance_state(id, field).unwrap()
}

fn act_at(model: &mut DynamicComponent, id: &str, event: &str, args: &[Value]) {
    let state = model.bridge().child_instance_state_id(id).unwrap();
    model.bridge_mut().call_handler_for("PromptDialog", event, state, args).unwrap();
}

#[test]
fn same_label_instances_recover_configuration_and_reject_a_previous_target_response() {
    let mut model = mount_prompt(r#"
widget App {
    msg { ShowConfigError(str) }
    model {
        var label str = ""
        var target str = "a"
        var reply str = ""
        var errors int = 0
    }
    view {
        col {
            PromptDialog (open: true, contextId: .target, title: "重命名",
                initialValue: "Shared", acceptLabel: .label, response: .reply,
                onerror: .ShowConfigError) {}
            PromptDialog (open: true, contextId: "other", title: "重命名",
                initialValue: "Shared", acceptLabel: "重命名") {}
        }
    }
    on { .ShowConfigError(payload) -> { .errors = .errors + 1 } }
}
"#);
    let ids = model.bridge().child_instance_identities_of_type("PromptDialog");
    assert_eq!(ids.len(), 2);
    for _ in 0..2 {
        for id in &ids {
            model.on_with_input_for_instance("PromptDialog", "Tick", None, Some(id));
        }
    }
    let first = ids.iter().find(|id| read_at(&model, id, "target") == Value::str("a")).unwrap();
    let other = ids.iter().find(|id| *id != first).unwrap();
    assert_eq!(model.read_state("errors").unwrap(), Value::Int(1));
    assert_eq!(read_at(&model, first, "config_code"), Value::str("empty-accept-label"));
    let binding_a = read_at(&model, first, "binding");
    act_at(&mut model, first, "Submit", std::slice::from_ref(&binding_a));
    assert_eq!(read_at(&model, first, "seq"), Value::Int(0));

    model.write_state("label", Value::str("重命名")).unwrap();
    let _ = model.view();
    model.on_with_input_for_instance("PromptDialog", "Tick", None, Some(first));
    assert_eq!(read_at(&model, first, "config_code"), Value::str(""));
    act_at(&mut model, first, "Submit", std::slice::from_ref(&binding_a));
    assert_eq!(read_at(&model, first, "phase"), Value::str("submitting"));
    let token = read_at(&model, first, "instance_token");
    assert_ne!(token, read_at(&model, other, "instance_token"));
    let reply_a = serde_json::json!({
        "instanceToken": token.as_str(), "session": 1, "seq": 1, "contextId": "a",
        "status": "rejected", "message": "old target", "retryable": false,
    });
    model.write_state("target", Value::str("b")).unwrap();
    model.write_state("reply", Value::str(&reply_a.to_string())).unwrap();
    let _ = model.view();
    model.on_with_input_for_instance("PromptDialog", "Tick", None, Some(first));
    assert_eq!(read_at(&model, first, "target"), Value::str("b"));
    assert_eq!(read_at(&model, first, "draft"), Value::str("Shared"));
    assert_eq!(read_at(&model, first, "rejection"), Value::str(""));
    act_at(&mut model, first, "Submit", &[binding_a]);
    assert_eq!(read_at(&model, first, "seq"), Value::Int(1));
    let binding_b = read_at(&model, first, "binding");
    act_at(&mut model, first, "Submit", &[binding_b]);
    assert_eq!(read_at(&model, first, "seq"), Value::Int(2));
    assert_eq!(read_at(&model, other, "seq"), Value::Int(0));

    let other_binding = read_at(&model, other, "binding");
    act_at(&mut model, other, "Submit", &[other_binding]);
    assert_eq!(read_at(&model, other, "seq"), Value::Int(1));
    assert_eq!(read_at(&model, first, "seq"), Value::Int(2));
}

#[test]
fn typed_boolean_inputs_preserve_legacy_values_and_bound_text() {
    let mut model = mount_source(r#"
widget App {
    msg { Plain(bool), Bound(str, bool), Enter }
    model {
        var checked str = ""
        var scope str = ""
        var enters int = 0
    }
    view { text "input dispatch fixture" }
    on {
        .Plain(value) -> { .checked = if value { "checked" } else { "unchecked" } }
        .Bound(prefix, value) -> {
            .scope = prefix
            .checked = if value { "checked" } else { "unchecked" }
        }
        .Enter -> { .enters = .enters + 1 }
    }
}
"#);
    for text in ["on", "true", "1", "yes", "off"] {
        model.on_with_input("Plain", Some(text.into()));
        let expected = if text == "off" { "unchecked" } else { "checked" };
        assert_eq!(model.read_state("checked").unwrap(), Value::str(expected));
    }
    model.on_with_input("Bound\u{1f}s\u{1f}source", Some("on".into()));
    assert_eq!(model.read_state("scope").unwrap(), Value::str("source"));
    assert_eq!(model.read_state("checked").unwrap(), Value::str("checked"));
    model.on_with_input("Bound\u{1f}s\u{1f}explicit\u{1f}s\u{1f}$event.target.checked", Some("off".into()));
    assert_eq!(model.read_state("scope").unwrap(), Value::str("explicit"));
    assert_eq!(model.read_state("checked").unwrap(), Value::str("unchecked"));
    model.on_with_input("Enter", Some("not an argument".into()));
    assert_eq!(model.read_state("enters").unwrap(), Value::Int(1));
}

fn feedback(model: &mut DynamicComponent, token: &Value, status: &str, retryable: bool) {
    let rendered = serde_json::json!({
        "instanceToken": token.as_str(),
        "session": 1,
        "seq": read(model, "seq").as_int(),
        "contextId": "notebook-7",
        "status": status,
        "message": "already exists",
        "retryable": retryable
    });
    model
        .write_state("response", Value::str(&rendered.to_string()))
        .unwrap();
    let _ = model.view();
    act(model, "Tick", &[]);
}

#[test]
fn model_retains_its_lock_and_rejects_retired_bindings_and_foreign_feedback() {
    let mut current = mount_model();
    let retired_token = read(&current, "instance_token");
    current.write_state("mounted", Value::Bool(false)).unwrap();
    let _ = current.view();
    current.write_state("mounted", Value::Bool(true)).unwrap();
    let _ = current.view();
    act(&mut current, "Tick", &[]);
    let token = read(&current, "instance_token");
    let old_binding = read(&current, "binding");
    assert_ne!(retired_token, token);

    // A new initial value can retire the old binding at the same revision.
    current
        .write_state("initial", Value::str("Journal"))
        .unwrap();
    let _ = current.view();
    act(&mut current, "Submit", std::slice::from_ref(&old_binding));
    assert_eq!(read(&current, "seq"), Value::Int(0));
    act(&mut current, "Tick", &[]);
    let binding = read(&current, "binding");
    assert_ne!(old_binding, binding);
    assert_eq!(read(&current, "draft"), Value::str("Journal"));
    act(&mut current, "Submit", std::slice::from_ref(&binding));
    assert_eq!(read(&current, "seq"), Value::Int(1));

    feedback(&mut current, &retired_token, "rejected", true);
    act(&mut current, "Submit", std::slice::from_ref(&binding));
    assert_eq!(read(&current, "seq"), Value::Int(1));
    assert_eq!(read(&current, "phase"), Value::str("submitting"));

    feedback(&mut current, &token, "rejected", false);
    act(&mut current, "Submit", std::slice::from_ref(&binding));
    assert_eq!(read(&current, "seq"), Value::Int(1));
    act(
        &mut current,
        "Edit",
        &[binding.clone(), Value::str("读书日志")],
    );
    act(&mut current, "Submit", std::slice::from_ref(&binding));
    assert_eq!(read(&current, "seq"), Value::Int(2));
    feedback(&mut current, &token, "rejected", true);
    act(&mut current, "Submit", std::slice::from_ref(&binding));
    assert_eq!(read(&current, "seq"), Value::Int(3));

    current.write_state("revision", Value::Int(1)).unwrap();
    current
        .write_state("initial", Value::str("读书日志"))
        .unwrap();
    feedback(&mut current, &token, "accepted", false);
    assert_eq!(read(&current, "phase"), Value::str("accepted"));
    act(&mut current, "Submit", &[binding]);
    assert_eq!(read(&current, "seq"), Value::Int(3));

    current.write_state("shown", Value::Bool(false)).unwrap();
    let _ = current.view();
    act(&mut current, "Tick", &[]);
    assert_eq!(read(&current, "seq"), Value::Int(3));
    current.write_state("shown", Value::Bool(true)).unwrap();
    let _ = current.view();
    act(&mut current, "Tick", &[]);
    assert_eq!(read(&current, "session"), Value::Int(2));
    let binding = read(&current, "binding");
    act(&mut current, "Close", &[binding.clone(), Value::str("cancel")]);
    act(&mut current, "Close", &[binding, Value::str("escape")]);
    assert_eq!(read(&current, "phase"), Value::str("cancelled"));
    assert_eq!(read(&current, "seq"), Value::Int(4));
}

#[test]
fn relocated_prompt_dialog_generates_the_consumer_and_shared_text_logic() {
    let root = std::env::var_os("PLAN094_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("D:/autostack/plan-artifacts/plan-094"));
    fs::create_dir_all(&root).unwrap();
    let scratch = tempfile::Builder::new()
        .prefix("moved-package-")
        .tempdir_in(root)
        .unwrap();
    let moved = scratch.path();
    let source = repository();
    let app = moved.join("examples/prompt-dialog");
    let package = moved.join("packages/prompt-dialog");
    let themes = moved.join("packages/themes");
    fs::create_dir_all(app.join("src/front")).unwrap();
    fs::create_dir_all(&package).unwrap();
    fs::create_dir_all(&themes).unwrap();
    for file in ["pac.at", "src/front/app.at"] {
        fs::copy(
            source.join("examples/prompt-dialog").join(file),
            app.join(file),
        )
        .unwrap();
    }
    for file in ["pac.at", "package.at", "prompt_dialog.at", "prompt_text.at"] {
        fs::copy(
            source.join("packages/prompt-dialog").join(file),
            package.join(file),
        )
        .unwrap();
    }
    for file in ["pac.at", "theme-package.json", "themes.at"] {
        fs::copy(source.join("packages/themes").join(file), themes.join(file)).unwrap();
    }
    let descriptor = auto_project_model::parse_file(&app.join("pac.at")).unwrap();
    let request = auto_project_model::GenerationRequest::from_descriptor(&descriptor);
    let mut project =
        auto_ui_tools::vue::VueProject::from_workspace_with_request(&app, &request).unwrap();
    project.output_dir = moved.join("vue");
    project.generate().unwrap();

    let app_source = fs::read_to_string(project.output_dir.join("src/App.vue")).unwrap();
    assert_eq!(app_source.matches("<PromptDialog ").count(), 2);
    assert!(app_source.contains("@submitRequested=\"SaveNotebook\""));
    let component =
        fs::read_to_string(project.output_dir.join("src/components/PromptDialog.vue")).unwrap();
    assert!(component.contains("function prompt_trim("));
    assert!(component.contains("globalThis.crypto.getRandomValues"));
    assert!(component.contains(":close-on-outside=\"false\""));
    assert!(component.contains(":aria-description=\"field_error\""));
    assert!(!component.contains("v-model=\"draft\""));
}
