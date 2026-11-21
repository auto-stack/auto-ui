use std::collections::BTreeSet;

/// A host preference binding, authored as `"storage:key:field" -> .Restore`.
#[derive(Clone, Debug)]
pub struct PreferenceBinding {
    pub key: String,
    pub field: String,
    pub restore: String,
}

pub fn is_preference_binding(key: &str) -> bool {
    key.trim_matches('"').starts_with("storage:")
}

pub fn preference_bindings<'a>(
    rows: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<Vec<PreferenceBinding>, String> {
    let identifier = |value: &str| {
        let mut chars = value.bytes();
        chars
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == b'_')
            && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
    };
    let mut keys = BTreeSet::new();
    let mut fields = BTreeSet::new();
    let mut result = Vec::new();
    for (name, target) in rows {
        let Some(path) = name.trim_matches('"').strip_prefix("storage:") else {
            continue;
        };
        let Some((key, field)) = path.split_once(':') else {
            return Err(format!(
                "storage binding must specify a key and model field: {name}"
            ));
        };
        let restore = target.trim_start_matches('.');
        if key.is_empty()
            || !key
                .bytes()
                .all(|ch| ch.is_ascii_alphanumeric() || b"._-".contains(&ch))
            || !identifier(field)
            || !identifier(restore)
        {
            return Err(format!("invalid storage binding: {name} -> {target}"));
        }
        if !keys.insert(key) || !fields.insert(field) {
            return Err(format!("duplicate storage binding for {key} or {field}"));
        }
        result.push(PreferenceBinding {
            key: key.into(),
            field: field.into(),
            restore: restore.into(),
        });
    }
    result.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(result)
}
