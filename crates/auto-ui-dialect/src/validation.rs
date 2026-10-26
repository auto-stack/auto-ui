use crate::{ModelBlock, MsgDecl, UiCode, UiStmt};
use std::collections::HashSet;

pub fn validate_ui(ui: &UiCode) -> Result<(), String> {
    let mut widgets = HashSet::new();
    let mut stores = HashSet::new();
    for stmt in ui.domain_stmts() {
        match stmt {
            UiStmt::WidgetDecl(widget) => {
                if !widgets.insert(widget.name.as_str().to_string()) {
                    return Err(validation_error(format!(
                        "duplicate widget declaration `{}`",
                        widget.name
                    )));
                }
                if let Some(model) = &widget.model {
                    validate_model(model)?;
                }
                validate_messages(&widget.messages)?;
            }
            UiStmt::StoreDecl(store) => {
                if !stores.insert(store.name.as_str().to_string()) {
                    return Err(validation_error(format!(
                        "duplicate store declaration `{}`",
                        store.name
                    )));
                }
                if let Some(model) = &store.model {
                    validate_model(model)?;
                }
                validate_messages(&store.messages)?;
            }
            UiStmt::MsgDecl(msg) => validate_messages(std::slice::from_ref(msg))?,
            UiStmt::ModelBlock(model) => validate_model(model)?,
            _ => {}
        }
    }
    Ok(())
}

fn validate_model(model: &ModelBlock) -> Result<(), String> {
    let mut fields = HashSet::new();
    for field in &model.fields {
        if !fields.insert(field.name.as_str().to_string()) {
            return Err(validation_error(format!(
                "duplicate model field `{}`",
                field.name
            )));
        }
    }
    Ok(())
}

fn validate_messages(messages: &[MsgDecl]) -> Result<(), String> {
    for message in messages {
        let mut variants = HashSet::new();
        for variant in &message.variants {
            if !variants.insert(variant.name.as_str().to_string()) {
                return Err(validation_error(format!(
                    "duplicate message variant `{}`",
                    variant.name
                )));
            }
            if variant.payload_names.len() != variant.payload.len() {
                return Err(validation_error(format!(
                    "message variant `{}` has mismatched payload names and types",
                    variant.name
                )));
            }
        }
    }
    Ok(())
}

fn validation_error(message: String) -> String {
    format!("UI domain validation failed: {message}")
}
