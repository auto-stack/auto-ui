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

fn mount_tag_input(consumer: &str) -> DynamicComponent {
    let package = repository().join("packages/tag-input");
    let model = fs::read_to_string(package.join("tag_input.at")).unwrap();
    let text = fs::read_to_string(package.join("tag_text.at")).unwrap();
    let source = format!(
        "{text}\n{consumer}\n{}",
        model.replace("use tag_text: tag_trim, ascii_lower, tag_contains\n", "")
    );
    mount_source(&source)
}

fn tag_identity(model: &DynamicComponent) -> String {
    let bridge = model.bridge();
    let instances = bridge.child_instance_identities_of_type("TagInput");
    assert_eq!(instances.len(), 1);
    instances[0].clone()
}

fn read(model: &DynamicComponent, field: &str) -> Value {
    model.bridge().read_instance_state(&tag_identity(model), field).unwrap()
}

fn read_root(model: &DynamicComponent, field: &str) -> Value {
    model.read_state(field).unwrap()
}

fn tick(model: &mut DynamicComponent) {
    let id = tag_identity(model);
    model.on_with_input_for_instance("TagInput", "Tick", None, Some(&id));
}

fn act(model: &mut DynamicComponent, event: &str, args: &[Value]) {
    let id = tag_identity(model);
    match event {
        "Edit" => {
            let text = args[0].as_str().to_string();
            model.on_with_input_for_instance("TagInput", "Edit", Some(text), Some(&id));
        }
        "Commit" | "Dismiss" | "BackspaceDelete" => {
            model.on_with_input_for_instance("TagInput", event, None, Some(&id));
        }
        "Remove" => {
            let tag = args[0].as_str().to_string();
            let event = format!("Remove\u{1f}s\u{1f}{tag}");
            model.on_with_input_for_instance("TagInput", &event, None, Some(&id));
        }
        _ => model.on_with_input_for_instance("TagInput", event, None, Some(&id)),
    }
}

fn consumer_base() -> &'static str {
    r#"
widget App {
    msg { Changed(str) }
    model {
        var tags List = ["Reading", "Work"]
        var contextId str = "demo-a"
        var revision int = 0
        var suggestions List = ["Design", "Development", "Work"]
        var maxTags int = 5
        var response str = ""
        var last_changed str = ""
    }
    view {
        col {
            TagInput (tags: .tags, contextId: .contextId, revision: .revision,
                suggestions: .suggestions, maxTags: .maxTags, response: .response,
                onchanged: .Changed) {}
        }
    }
    on { .Changed(payload) -> { .last_changed = payload } }
}
"#
}

fn confirm_add(model: &mut DynamicComponent) -> Value {
    let payload = read_root(model, "last_changed");
    let parsed: serde_json::Value = serde_json::from_str(payload.as_str()).unwrap();
    let token = parsed["instanceToken"].as_str().unwrap().to_string();
    // the app applies the payload's tags array and bumps revision in the same
    // snapshot, then replies accepted with the echoed identity
    let new_tags = parsed["tags"].as_array().unwrap().clone();
    let mut tags: Vec<Value> = new_tags
        .iter()
        .map(|v| Value::str(v.as_str().unwrap()))
        .collect();
    if tags.is_empty() {
        tags.push(Value::str("Reading"));
    }
    model
        .write_state("tags", Value::Array(auto_val::Array { values: tags }))
        .unwrap();
    model.write_state("revision", Value::Int(1)).unwrap();
    let reply = serde_json::json!({
        "instanceToken": token, "seq": 1, "contextId": "demo-a",
        "status": "accepted", "message": "",
    });
    model.write_state("response", Value::str(&reply.to_string())).unwrap();
    let _ = model.view();
    Value::str(&reply.to_string())
}

#[test]
fn add_dedup_and_confirm_respect_identity_and_revision() {
    let mut model = mount_tag_input(consumer_base());
    tick(&mut model);
    assert_eq!(read(&model, "phase"), Value::str("idle"));
    assert_eq!(read(&model, "count"), Value::Int(2));

    // A duplicate add at the mounted state must not emit a request.
    act(&mut model, "Edit", &[Value::str("Work")]);
    act(&mut model, "Commit", &[]);
    assert_eq!(read(&model, "seq"), Value::Int(0));
    assert_eq!(read(&model, "input_err"), Value::str("标签已存在"));

    // Enter commits the draft "Design" -> one changed with the full array.
    act(&mut model, "Edit", &[Value::str("Design")]);
    act(&mut model, "Commit", &[]);
    assert_eq!(read(&model, "phase"), Value::str("pending"));
    assert_eq!(read(&model, "seq"), Value::Int(1));
    let payload = read_root(&model, "last_changed");
    let parsed: serde_json::Value = serde_json::from_str(payload.as_str()).unwrap();
    assert_eq!(parsed["action"], "add");
    assert_eq!(parsed["tag"], "Design");
    assert_eq!(parsed["tags"].as_array().unwrap().len(), 3);

    // Accepted reply with matching identity + advanced tags/revision confirms.
    confirm_add(&mut model);
    tick(&mut model);
    assert_eq!(read(&model, "phase"), Value::str("idle"));
    assert_eq!(read(&model, "draft"), Value::str(""));
    assert_eq!(read(&model, "base_revision"), Value::Int(1));

    // A duplicate of an already-accepted tag is still blocked after the confirm.
    act(&mut model, "Edit", &[Value::str("Work")]);
    act(&mut model, "Commit", &[]);
    assert_eq!(read(&model, "seq"), Value::Int(1));
    assert_eq!(read(&model, "input_err"), Value::str("标签已存在"));
}

#[test]
fn limit_blocks_adds_but_allows_removal() {
    let consumer = r#"
widget App {
    msg { Changed(str) }
    model {
        var tags List = ["Reading", "Work"]
        var contextId str = "demo-b"
        var revision int = 0
        var suggestions List = ["Design", "Development", "Work"]
        var maxTags int = 2
        var response str = ""
        var last_changed str = ""
        var last_error str = ""
    }
    view {
        col {
            TagInput (tags: .tags, contextId: .contextId, revision: .revision,
                suggestions: .suggestions, maxTags: .maxTags, response: .response,
                onchanged: .Changed, onerror: .ErrSink) {}
        }
    }
    on {
        .Changed(payload) -> { .last_changed = payload }
        .ErrSink(payload) -> { .last_error = payload }
    }
}
"#;
    let mut model = mount_tag_input(consumer);
    tick(&mut model);
    act(&mut model, "Edit", &[Value::str("Development")]);
    act(&mut model, "Commit", &[]);
    assert_eq!(read(&model, "seq"), Value::Int(0));
    assert_eq!(read(&model, "input_err"), Value::str("已达标签上限"));

    // removal is still allowed
    act(&mut model, "Remove", &[Value::str("Work")]);
    assert_eq!(read(&model, "seq"), Value::Int(1));
    assert_eq!(read(&model, "phase"), Value::str("pending"));
    let payload = read_root(&model, "last_changed");
    let parsed: serde_json::Value = serde_json::from_str(payload.as_str()).unwrap();
    assert_eq!(parsed["action"], "remove");
    assert_eq!(parsed["tags"].as_array().unwrap().len(), 1);

    // rejected reply keeps the request state and does not touch tags
    let parsed = read_root(&model, "last_changed");
    let parsed: serde_json::Value = serde_json::from_str(parsed.as_str()).unwrap();
    let token = parsed["instanceToken"].as_str().unwrap();
    let reply = serde_json::json!({
        "instanceToken": token, "seq": 1, "contextId": "demo-b",
        "status": "rejected", "message": "busy",
    });
    model.write_state("response", Value::str(&reply.to_string())).unwrap();
    let _ = model.view();
    tick(&mut model);
    assert_eq!(read(&model, "phase"), Value::str("idle"));
    assert_eq!(read(&model, "rejection"), Value::str("busy"));
}

#[test]
fn foreign_token_and_stale_target_feedback_are_ignored() {
    let mut model = mount_tag_input(consumer_base());
    tick(&mut model);
    let token = read(&model, "instance_token");
    let foreign = serde_json::json!({
        "instanceToken": "not-this-instance", "seq": 1, "contextId": "demo-a",
        "status": "accepted", "message": "",
    });
    model.write_state("response", Value::str(&foreign.to_string())).unwrap();
    let _ = model.view();
    tick(&mut model);
    assert_eq!(read(&model, "phase"), Value::str("idle"));
    assert_eq!(read(&model, "seq"), Value::Int(0));

    // a stale-target reply for a request never sent is ignored
    let stale = serde_json::json!({
        "instanceToken": token.as_str(), "seq": 7, "contextId": "demo-a",
        "status": "accepted", "message": "",
    });
    model.write_state("response", Value::str(&stale.to_string())).unwrap();
    let _ = model.view();
    tick(&mut model);
    assert_eq!(read(&model, "seq"), Value::Int(0));
}

#[test]
fn negative_max_tags_reports_config_error_once_and_recovers() {
    let consumer = r#"
widget App {
    msg { ErrSink(str) }
    model {
        var tags List = ["Reading"]
        var contextId str = "demo-c"
        var revision int = 0
        var maxTags int = -1
        var response str = ""
        var error_count int = 0
    }
    view {
        col {
            TagInput (tags: .tags, contextId: .contextId, revision: .revision,
                maxTags: .maxTags, response: .response, onerror: .ErrSink) {}
        }
    }
    on { .ErrSink(payload) -> { .error_count = .error_count + 1 } }
}
"#;
    let mut model = mount_tag_input(consumer);
    tick(&mut model);
    tick(&mut model);
    assert_eq!(read_root(&model, "error_count"), Value::Int(1));
    assert_eq!(read(&model, "config_code"), Value::str("invalid-max-tags"));

    // fix the config: no further error, modifications recover
    model.write_state("maxTags", Value::Int(5)).unwrap();
    let _ = model.view();
    tick(&mut model);
    assert_eq!(read_root(&model, "error_count"), Value::Int(1));
    assert_eq!(read(&model, "config_code"), Value::str(""));
    act(&mut model, "Edit", &[Value::str("Design")]);
    act(&mut model, "Commit", &[]);
    assert_eq!(read(&model, "seq"), Value::Int(1));
}

#[test]
fn two_instances_keep_distinct_tokens_and_sequences() {
    let consumer = r#"
widget App {
    msg { Changed(str) }
    model {
        var tags_a List = ["Reading"]
        var tags_b List = ["Work"]
        var contextId str = "a"
        var revision int = 0
        var response str = ""
    }
    view {
        col {
            TagInput (tags: .tags_a, contextId: "a", revision: .revision, response: .response, onchanged: .Changed) {}
            TagInput (tags: .tags_b, contextId: "b", revision: .revision, response: .response, onchanged: .Changed) {}
        }
    }
    on { .Changed(payload) -> { } }
}
"#;
    let mut model = mount_tag_input(consumer);
    let instances = model.bridge().child_instance_identities_of_type("TagInput");
    assert_eq!(instances.len(), 2);
    for id in &instances {
        model.on_with_input_for_instance("TagInput", "Tick", None, Some(id));
    }
    let a = instances
        .iter()
        .find(|id| model.bridge().read_instance_state(id, "target").unwrap() == Value::str("a"))
        .unwrap()
        .clone();
    let b = instances
        .iter()
        .find(|id| *id != &a)
        .unwrap()
        .clone();
    let ta = model.bridge().read_instance_state(&a, "instance_token").unwrap();
    let tb = model.bridge().read_instance_state(&b, "instance_token").unwrap();
    assert_ne!(ta, tb);
    // add on instance a only bumps its seq
    model.on_with_input_for_instance("TagInput", "Edit", Some("Design".into()), Some(&a));
    model.on_with_input_for_instance("TagInput", "Commit", None, Some(&a));
    assert_eq!(model.bridge().read_instance_state(&a, "seq").unwrap(), Value::Int(1));
    assert_eq!(model.bridge().read_instance_state(&b, "seq").unwrap(), Value::Int(0));
}


#[test]
fn backspace_on_empty_draft_removes_the_last_tag_once() {
    let mut model = mount_tag_input(consumer_base());
    tick(&mut model);
    // empty draft backspace -> remove the last tag (Work) with one request
    act(&mut model, "BackspaceDelete", &[]);
    assert_eq!(read(&model, "seq"), Value::Int(1));
    assert_eq!(read(&model, "phase"), Value::str("pending"));
    let payload = read_root(&model, "last_changed");
    let parsed: serde_json::Value = serde_json::from_str(payload.as_str()).unwrap();
    assert_eq!(parsed["action"], "remove");
    assert_eq!(parsed["tag"], "Work");
    assert_eq!(parsed["tags"].as_array().unwrap().len(), 1);
    // a non-empty draft must not trigger the removal
    act(&mut model, "Edit", &[Value::str("de")]);
    act(&mut model, "BackspaceDelete", &[]);
    assert_eq!(read(&model, "seq"), Value::Int(1));
}

#[test]
fn stale_target_aborts_the_pending_request_and_rejects_old_replies() {
    let mut model = mount_tag_input(consumer_base());
    tick(&mut model);
    act(&mut model, "Edit", &[Value::str("Design")]);
    act(&mut model, "Commit", &[]);
    assert_eq!(read(&model, "phase"), Value::str("pending"));
    // the parent switches the target before confirming
    model.write_state("contextId", Value::str("demo-b")).unwrap();
    let _ = model.view();
    tick(&mut model);
    assert_eq!(read(&model, "phase"), Value::str("idle"));
    assert_eq!(read(&model, "draft"), Value::str(""));
    assert_eq!(read(&model, "target"), Value::str("demo-b"));
    // a late accepted reply for the OLD target must not settle anything
    let old_token = read(&model, "instance_token");
    let late = serde_json::json!({
        "instanceToken": old_token.as_str(), "seq": 1, "contextId": "demo-a",
        "status": "accepted", "message": "",
    });
    model.write_state("response", Value::str(&late.to_string())).unwrap();
    let _ = model.view();
    tick(&mut model);
    assert_eq!(read(&model, "phase"), Value::str("idle"));
    assert_eq!(read(&model, "seq"), Value::Int(1));
}

#[test]
fn accepted_with_mismatched_tags_is_invalid_feedback_and_external_replacement_wins() {
    let mut model = mount_tag_input(consumer_base());
    tick(&mut model);
    act(&mut model, "Edit", &[Value::str("Design")]);
    act(&mut model, "Commit", &[]);
    let token = read(&model, "instance_token");
    // accepted reply, but the parent advanced tags to something else
    let mismatch = serde_json::json!({
        "instanceToken": token.as_str(), "seq": 1, "contextId": "demo-a",
        "status": "accepted", "message": "",
    });
    model.write_state("response", Value::str(&mismatch.to_string())).unwrap();
    let mut ext = vec![Value::str("Reading"), Value::str("Design"), Value::str("Other")];
    model
        .write_state("tags", Value::Array(auto_val::Array { values: ext.clone() }))
        .unwrap();
    ext.clear();
    model.write_state("revision", Value::Int(1)).unwrap();
    let _ = model.view();
    tick(&mut model);
    eprintln!("dbg phase={:?} fb={:?} seen={:?} base={:?} rev={:?} seq={:?}",
        read(&model, "phase"), read(&model, "feedback_err"),
        read(&model, "seen_response"), read(&model, "base_revision"), read(&model, "revision"), read(&model, "seq"));
    assert_eq!(read(&model, "phase"), Value::str("idle"));
    assert_eq!(read(&model, "feedback_err"), Value::str("确认与当前标签不一致"));
}

#[test]
fn same_revision_external_replacement_invalidates_the_old_request() {
    let mut model = mount_tag_input(consumer_base());
    tick(&mut model);
    act(&mut model, "Edit", &[Value::str("Design")]);
    act(&mut model, "Commit", &[]);
    assert_eq!(read(&model, "phase"), Value::str("pending"));
    // tags replaced at the SAME revision (external replacement, no reply)
    let replaced = vec![Value::str("Reading"), Value::str("Work"), Value::str("New")];
    model
        .write_state("tags", Value::Array(auto_val::Array { values: replaced }))
        .unwrap();
    let _ = model.view();
    tick(&mut model);
    // old request aborted; no false success claim; tags-only change keeps draft
    assert_eq!(read(&model, "phase"), Value::str("idle"));
    assert_eq!(read(&model, "confirm_seen"), Value::Bool(false));
}

#[test]
fn relocated_tag_input_generates_the_consumer_and_shared_text_logic() {
    let root = std::env::var_os("PLAN097_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("D:/autostack/plan-artifacts/plan-097"));
    fs::create_dir_all(&root).unwrap();
    let scratch = tempfile::Builder::new()
        .prefix("moved-tag-input-")
        .tempdir_in(root)
        .unwrap();
    let moved = scratch.path();
    let source = repository();
    let app = moved.join("examples/tag-input");
    let package = moved.join("packages/tag-input");
    let themes = moved.join("packages/themes");
    fs::create_dir_all(app.join("src/front")).unwrap();
    fs::create_dir_all(&package).unwrap();
    fs::create_dir_all(&themes).unwrap();
    for file in ["pac.at", "src/front/app.at"] {
        fs::copy(source.join("examples/tag-input").join(file), app.join(file)).unwrap();
    }
    for file in ["pac.at", "package.at", "tag_input.at", "tag_text.at"] {
        fs::copy(source.join("packages/tag-input").join(file), package.join(file)).unwrap();
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
    assert_eq!(app_source.matches("<TagInput ").count(), 3);
    assert!(app_source.contains("@changed=\"RequestA\""));
    let component =
        fs::read_to_string(project.output_dir.join("src/components/TagInput.vue")).unwrap();
    assert!(component.contains("function tag_trim("));
    assert!(component.contains("globalThis.crypto.getRandomValues"));
    assert!(component.contains("candidates_visible"));
}
