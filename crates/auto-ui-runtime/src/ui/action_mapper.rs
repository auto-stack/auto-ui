//! # Action Mapper — Map MCP actions to VmBridge calls (Plan 278)
//!
//! Translates high-level MCP actions (press, type_text, toggle, etc.)
//! into concrete VmBridge operations (call_handler, write_state).
//!
//! ## Action → Handler Mapping
//!
//! | Action        | Component       | VmBridge Call                                |
//! |---------------|-----------------|----------------------------------------------|
//! | press         | Button          | call_handler(event_name, [])                 |
//! | type_text     | Input/Textarea  | write_state(field, value) + call_handler     |
//! | toggle        | Checkbox        | call_handler(event_name, [])                 |
//! | select_option | Select/Radio    | call_handler(event_name, [index, label])     |
//! | set_value     | Slider          | write_state(field, value) + call_handler     |

use std::collections::HashMap;

use crate::ui::mcp_types::UiActionType;
use auto_ui_compiler::aura::AuraNodeId;

/// Error returned when an action cannot be performed.
#[derive(Debug)]
pub enum ActionError {
    ElementNotFound(AuraNodeId),
    InvalidAction {
        action: UiActionType,
        component_kind: String,
    },
    NoHandler {
        action: UiActionType,
        element_id: AuraNodeId,
    },
    MissingValue(UiActionType),
    InvalidValue(String),
    ExecutionError(String),
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionError::ElementNotFound(id) => write!(f, "Element not found: #{}", id),
            ActionError::InvalidAction {
                action,
                component_kind,
            } => {
                write!(
                    f,
                    "Action '{}' is not valid for component type '{}'",
                    action, component_kind
                )
            }
            ActionError::NoHandler { action, element_id } => {
                write!(
                    f,
                    "No handler for action '{}' on element #{}",
                    action, element_id
                )
            }
            ActionError::MissingValue(action) => {
                write!(f, "Action '{}' requires a value parameter", action)
            }
            ActionError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ActionError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

/// Context needed to perform an action.
///
/// This trait abstracts over the actual state mutation mechanism,
/// allowing the action mapper to work with any backend that can
/// read/write state and call handlers.
pub trait ActionContext {
    /// Read all current state values.
    fn read_all_state(&self) -> HashMap<String, auto_val::Value>;

    /// Write a state field value.
    fn write_state(&mut self, field: &str, value: auto_val::Value) -> Result<(), String>;

    /// Call an event handler by name.
    fn call_handler(&mut self, event_name: &str, args: &[auto_val::Value]) -> Result<(), String>;

    /// Get the input-to-state mapping (event_name -> state_field_name).
    fn input_state_map(&self) -> &HashMap<String, String>;
}
