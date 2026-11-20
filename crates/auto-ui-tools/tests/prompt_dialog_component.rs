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
    let package = repository().join("packages/prompt-dialog");
    let model = fs::read_to_string(package.join("prompt_dialog.at")).unwrap();
    let text = fs::read_to_string(package.join("prompt_text.at")).unwrap();
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
    let source = format!(
        "{text}\n{consumer}\n{}",
        model.replace("use prompt_text: prompt_trim\n", "")
    );
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
    let widget = auto_ui_compiler::aura::extract_widget_from_decl(&declarations[0]).unwrap();
    let mut registry = WidgetRegistry::new();
    registry.register(auto_ui_compiler::aura::extract_widget_from_decl(&declarations[1]).unwrap());
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
    act(&mut component, "Tick", &[]);
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
