//! The ordinary component model mailbox is private to this package. No native
//! language operation, alternate MCP dispatcher or OS picker is introduced.
use super::{DialogConfig, DialogEvent, DialogModel, Failure};
use crate::ui::vm_bridge::VmBridge;
use auto_val::Value;
use std::collections::{HashMap, VecDeque};

#[derive(serde::Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum Command {
    Open { config: DialogConfig },
    Navigate { path: String },
    Input { value: String },
    Up,
    Refresh,
    Name { value: String },
    Filter { id: String },
    Hidden { value: bool },
    Toggle { id: String },
    Enter { id: String },
    Clear,
    Page { index: usize },
    Confirm,
    Cancel,
}

struct Instance {
    state_id: u64,
    last_command: String,
    closed_prop_observed: bool,
    model: DialogModel,
    events: VecDeque<serde_json::Value>,
}

#[derive(Default)]
pub(crate) struct ComponentIo {
    instances: HashMap<String, Instance>,
}

fn text(bridge: &VmBridge, identity: &str, field: &str) -> String {
    match bridge.read_instance_state(identity, field) {
        Ok(Value::Str(s)) => s.to_string(),
        _ => String::new(),
    }
}

fn write(bridge: &mut VmBridge, identity: &str, field: &str, value: String) -> bool {
    if text(bridge, identity, field) == value {
        return false;
    }
    bridge
        .write_instance_state(identity, field, Value::str(value))
        .is_ok()
}

fn error_event(error: Failure) -> serde_json::Value {
    serde_json::json!({"kind":"error", "payload":serde_json::to_string(&error).unwrap()})
}

impl ComponentIo {
    pub fn retain_live(&mut self, bridge: &VmBridge) {
        let live = bridge.child_instance_identities_of_type("FileDialog");
        self.instances.retain(|key, _| live.contains(key));
        // A view can observe false and the next root handler can immediately
        // change it back to true before the next IO pump. Retain that boundary
        // rather than sampling only the final value of the parent-backed prop.
        for (identity, slot) in &mut self.instances {
            if matches!(bridge.read_instance_state(identity, "open"), Ok(Value::Bool(false))) {
                slot.closed_prop_observed = true;
            }
        }
    }

    pub fn pump(&mut self, bridge: &mut VmBridge) -> bool {
        let live = bridge.child_instance_identities_of_type("FileDialog");
        self.instances.retain(|key, _| live.contains(key));
        let mut changed = false;
        for identity in live {
            let Some(state_id) = bridge.child_instance_state_id(&identity) else {
                continue;
            };
            let slot = self
                .instances
                .entry(identity.clone())
                .or_insert_with(|| Instance {
                    state_id,
                    last_command: String::new(),
                    closed_prop_observed: false,
                    model: DialogModel::default(),
                    events: VecDeque::new(),
                });
            if slot.state_id != state_id {
                *slot = Instance {
                    state_id,
                    last_command: String::new(),
                    closed_prop_observed: false,
                    model: DialogModel::default(),
                    events: VecDeque::new(),
                };
            }
            // Parent props are refreshed by view construction, independently
            // of the 50ms component clock. Observe a closed prop at every host
            // boundary so an immediate reopen cannot miss the false edge.
            let desired_open = matches!(bridge.read_instance_state(&identity, "open"), Ok(Value::Bool(true)));
            if !desired_open || slot.closed_prop_observed {
                slot.closed_prop_observed = false;
                if matches!(bridge.read_instance_state(&identity, "seen_open"), Ok(Value::Bool(true))) {
                    changed |= bridge.write_instance_state(&identity, "seen_open", Value::Bool(false)).is_ok();
                }
                slot.model.cancel();
            }
            let command = text(bridge, &identity, "fd_command");
            if !command.is_empty() && command != slot.last_command {
                slot.last_command = command.clone();
                match serde_json::from_str::<Command>(&command) {
                    Ok(Command::Open { config }) => {
                        if desired_open && slot.model.open(config) {
                            slot.events.clear();
                            changed |= write(bridge, &identity, "fd_event", String::new());
                        }
                    }
                    Ok(Command::Navigate { path }) => slot.model.navigate(path),
                    Ok(Command::Input { value }) => slot.model.set_path_input(value),
                    Ok(Command::Up) => slot.model.up(),
                    Ok(Command::Refresh) => slot.model.refresh(),
                    Ok(Command::Name { value }) => slot.model.set_name_query(value),
                    Ok(Command::Filter { id }) => slot.model.set_filter(&id),
                    Ok(Command::Hidden { value }) => slot.model.set_show_hidden(value),
                    Ok(Command::Toggle { id }) => {
                        slot.model.toggle(&id);
                    }
                    Ok(Command::Enter { id }) => {
                        slot.model.enter(&id);
                    }
                    Ok(Command::Clear) => slot.model.clear_selection(),
                    Ok(Command::Page { index }) => slot.model.set_page(index),
                    Ok(Command::Confirm) => {
                        slot.model.confirm();
                    }
                    Ok(Command::Cancel) => {
                        slot.model.cancel();
                    }
                    Err(error) => slot.events.push_back(error_event(Failure::new(
                        "invalid_path",
                        format!("FileDialog request: {error}"),
                        true,
                    ))),
                }
            }
            slot.model.poll();
            for event in slot.model.take_events() {
                slot.events.push_back(match event {
                    DialogEvent::Accepted(files) => serde_json::json!({"kind":"accepted", "payload":serde_json::json!({"files":files}).to_string()}),
                    DialogEvent::Cancelled => serde_json::json!({"kind":"cancelled", "payload":""}),
                    DialogEvent::Error(error) => error_event(error),
                });
            }
            changed |= write(
                bridge,
                &identity,
                "fd_view",
                serde_json::to_string(&slot.model.projection()).unwrap(),
            );
            // The VM's current emit bridge carries one event per handler. Keep
            // a queue here instead of losing an error or terminal event when
            // two results become ready between component ticks.
            if text(bridge, &identity, "fd_event").is_empty() {
                if let Some(event) = slot.events.pop_front() {
                    changed |= write(bridge, &identity, "fd_event", event.to_string());
                }
            }
        }
        changed
    }
}
