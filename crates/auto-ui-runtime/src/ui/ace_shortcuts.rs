use ace_core::source::age_component::{ComponentMemberRow, ComponentMsgVariant};
use std::collections::BTreeMap;

pub(super) fn widget_shortcuts(
    members: &[ComponentMemberRow],
    messages: &[ComponentMsgVariant],
) -> BTreeMap<String, String> {
    members
        .iter()
        .filter_map(|row| {
            if row.role != "bind" {
                return None;
            }
            let key: String = serde_json::from_str(&row.name).ok()?;
            let message = row.text.as_ref()?;
            if key.is_empty()
                || key.contains(':')
                || !messages.iter().any(|candidate| {
                    candidate.name == *message
                        && candidate.payload.is_empty()
                        && !candidate.outbound
                })
            {
                return None;
            }
            Some((key, message.clone()))
        })
        .collect()
}
