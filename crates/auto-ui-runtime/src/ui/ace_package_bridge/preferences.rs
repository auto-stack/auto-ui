use ace_core::source::age_package::AgePackageComponent;
use auto_lang_core::vm::ffi::stdlib::{storage_host_publish, storage_host_read_fresh};
use auto_ui_dialect::preferences::{preference_bindings, PreferenceBinding};

use super::AceStateValue;

pub(crate) fn checked_bindings(
    component: &AgePackageComponent,
) -> Result<Vec<PreferenceBinding>, String> {
    let bindings = preference_bindings(
        component
            .members
            .iter()
            .filter(|row| row.role == "bind")
            .map(|row| (row.name.as_str(), row.text.as_deref().unwrap_or_default())),
    )?;
    for binding in &bindings {
        if !component.fields.iter().any(|(name, ty)| {
            name == &binding.field && matches!(ty.as_str(), "str" | "Str" | "bool")
        }) || !component.handler_msgs.contains(&binding.restore)
            || !component
                .messages
                .iter()
                .any(|msg| msg.name == binding.restore && msg.payload == ["str"])
        {
            return Err(format!(
                "ace.widget.storage-binding: {} needs a text or bool field and a str restore handler",
                binding.key
            ));
        }
    }
    Ok(bindings)
}

pub(super) fn restore(bindings: &[PreferenceBinding]) -> impl Iterator<Item = (&str, String)> {
    bindings.iter().filter_map(|binding| {
        storage_host_read_fresh(&binding.key).map(|value| (binding.restore.as_str(), value))
    })
}

pub(super) fn save_changes(
    bindings: &[PreferenceBinding],
    previous: &[(String, AceStateValue)],
    current: &[(String, AceStateValue)],
) {
    let get = |name: &str| {
        let previous = previous
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value);
        let current = current
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value);
        (previous, current)
    };
    for binding in bindings {
        let (previous, current) = get(&binding.field);
        if previous == current {
            continue;
        }
        let value = match current {
            Some(AceStateValue::Str(value)) => value.clone(),
            Some(AceStateValue::Bool(value)) => value.to_string(),
            _ => continue,
        };
        storage_host_publish(&binding.key, value);
    }
}
