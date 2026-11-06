//! # State Migration - Hot reload state preservation
//!
//! This module provides state migration between widget definitions during hot reload.
//! When a `.at` file changes and the widget definition is updated, state migration
//! preserves values for fields that still exist, uses defaults for new fields,
//! and drops removed fields.
//!
//! ## Plan 205 Phase 4
//!
//! Phase 4 adds hot reload support to DynamicComponent:
//! - State migration between old and new widget definitions
//! - Type-safe migration: if a field changes type, use the new default
//! - File modification time tracking for simple change detection

use std::collections::HashMap;

use auto_lang_core::ast::Expr;
use auto_lang_core::ast::Type;
use auto_ui_compiler::aura::AuraStateDef;
use auto_val::{Op, Value};

/// Result of a state migration operation.
///
/// Provides counts of preserved, added, and dropped fields for diagnostics.
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// Number of fields whose values were preserved from the old state.
    pub preserved: usize,
    /// Number of new fields initialized with default values.
    pub added: usize,
    /// Number of fields that were dropped (existed in old but not in new).
    pub dropped: usize,
    /// Names of dropped fields.
    pub dropped_names: Vec<String>,
}

/// PLAN-056: typed migration outcome. `Reject` names the same-name fields
/// whose declared type changed (or that are unknown/opaque on either side) —
/// the whole candidate bundle is rejected and the old bundle keeps serving.
#[derive(Debug, Clone)]
pub enum MigrationOutcome {
    /// Migration succeeded: migrated state plus the per-field report.
    Ok(HashMap<String, Value>, MigrationReport),
    /// The candidate cannot be committed: same-name type conflict / opaque
    /// value on the named fields. Old state must remain in service.
    Reject(Vec<String>),
}

/// PLAN-056: typed field-definition table for migration. `name → declared
/// type` for root model fields plus every store-as-child / child model field
/// that participates in the unified root state (VM 轨 store 字段平铺根态).
pub type StateDefMap = HashMap<String, Type>;

/// Build the unified root-state definition map for a set of widgets
/// (root + children + stores). Each widget's model `state_vars` contributes
/// name → type; collisions across widgets resolve to the first entry (the
/// unified root state holds one slot per name, so the type must agree).
pub fn flattened_state_defs<'a>(
    widgets: impl Iterator<Item = &'a AuraStateDef>,
) -> StateDefMap {
    let mut map = StateDefMap::new();
    for def in widgets {
        map.entry(def.name.clone()).or_insert_with(|| def.type_info.clone());
    }
    map
}

/// PLAN-056: typed migrate-or-reject for a candidate bundle commit.
///
/// Unlike the legacy [`migrate_state`] (which falls back to the new default on
/// a type change), a same-name field whose declared type changed between the
/// committed source and the candidate rejects the whole candidate — the plan's
/// "同名不同类型默认拒绝整候选" contract. Unknown/opaque types on either side
/// also reject (never optimistically write an old `Value` into a field whose
/// type is not established).
///
/// Returns [`MigrationOutcome::Ok`] with the migrated state + report, or
/// [`MigrationOutcome::Reject`] naming the conflicting fields.
pub fn migrate_state_typed(
    old_state: &HashMap<String, Value>,
    old_defs: &StateDefMap,
    new_defs: &StateDefMap,
) -> MigrationOutcome {
    let mut new_state = HashMap::new();
    let mut preserved = 0usize;
    let mut added = 0usize;
    let mut dropped_names = Vec::new();
    let mut rejected: Vec<String> = Vec::new();

    for (name, new_ty) in new_defs {
        if let Some(old_val) = old_state.get(name) {
            let compatible = match old_defs.get(name) {
                Some(old_ty) => types_compatible(old_ty, new_ty),
                // No declared old type: fall back to kind-level matching
                // against the runtime value (unknown-but-evident); opaque
                // values still reject (plan §5 "未知/不透明类型亦拒绝").
                None => value_kind_matches_type(old_val, new_ty),
            };
            if compatible {
                new_state.insert(name.clone(), old_val.clone());
                preserved += 1;
            } else {
                rejected.push(name.clone());
            }
        } else {
            // New field: the new bridge already seeded its default.
            added += 1;
        }
    }

    if !rejected.is_empty() {
        return MigrationOutcome::Reject(rejected);
    }

    // Host slots (`__route_params` / `__current_route` / `__auto*`-adjacent)
    // never migrate and never appear in dropped_names; the new bridge seeds
    // them lazily (spec §3: 宿主字段不得出现在 dropped_names).
    for name in old_state.keys() {
        if name.starts_with("__") {
            continue;
        }
        if !new_defs.contains_key(name.as_str()) {
            dropped_names.push(name.clone());
        }
    }
    let dropped = dropped_names.len();

    let report = MigrationReport {
        preserved,
        added,
        dropped,
        dropped_names,
    };

    MigrationOutcome::Ok(new_state, report)
}

/// Kind-level compatibility between a runtime `Value` and a declared type —
/// used only when the old source had no declared type for the field. Opaque
/// containers / custom types fail closed (reject).
fn value_kind_matches_type(value: &Value, ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Uint | Type::USize | Type::I64 | Type::U64 | Type::Byte => {
            matches!(value, Value::Int(_) | Value::Uint(_))
        }
        Type::Float | Type::Double => matches!(value, Value::Double(_) | Value::Float(_)),
        Type::Bool => matches!(value, Value::Bool(_)),
        Type::Char => matches!(value, Value::Int(_)),
        Type::CStrLit | Type::StrSlice | Type::StrOwned | Type::StrFixed(_) => {
            matches!(value, Value::Str(_))
        }
        // Arrays / objects / custom named types: the runtime value must be a
        // container we can move across bridges; opaque heap refs reject.
        Type::Void => false,
        _ => matches!(value, Value::Array(_) | Value::Obj(_)),
    }
}

/// Migrate state from an old widget definition to a new one.
///
/// For each field in the new definition:
/// - If the field existed in the old state **and** the type is compatible,
///   the old value is preserved.
/// - If the field is new or the type changed, the default value from the
///   new definition's `initial` expression is used.
///
/// Fields that existed in the old state but are absent from the new definition
/// are dropped.
///
/// # Arguments
///
/// * `old_state` - Current state as a name-to-value map (from `VmBridge::read_all_state`)
/// * `old_fields` - Old field definitions (for type info)
/// * `new_fields` - New field definitions
///
/// # Returns
///
/// A tuple of `(migrated_state, migration_report)`.
pub fn migrate_state(
    old_state: &HashMap<String, Value>,
    old_fields: &[AuraStateDef],
    new_fields: &[AuraStateDef],
) -> (HashMap<String, Value>, MigrationReport) {
    // Build a lookup of old field types for type-compatibility checks
    let old_field_types: HashMap<&str, &Type> = old_fields
        .iter()
        .map(|f| (f.name.as_str(), &f.type_info))
        .collect();

    let mut new_state = HashMap::new();
    let mut preserved = 0usize;
    let mut added = 0usize;
    let mut dropped_names = Vec::new();

    // Migrate fields present in the new definition
    for field in new_fields {
        if let Some(old_val) = old_state.get(&field.name) {
            // Field exists in both -- check type compatibility
            // If we have old type info, use it; otherwise, optimistically
            // preserve the old value (runtime case where type defs are not available).
            let type_compatible = match old_field_types.get(field.name.as_str()) {
                Some(old_ty) => types_compatible(old_ty, &field.type_info),
                None => true, // No old type info: assume compatible
            };

            if type_compatible {
                // Type-compatible: keep old value
                new_state.insert(field.name.clone(), old_val.clone());
                preserved += 1;
            } else {
                // Type changed: use new default
                new_state.insert(field.name.clone(), eval_default(&field.initial));
                added += 1;
            }
        } else {
            // New field: use default value
            new_state.insert(field.name.clone(), eval_default(&field.initial));
            added += 1;
        }
    }

    // Track dropped fields (for the report)
    let new_field_names: HashMap<&str, ()> =
        new_fields.iter().map(|f| (f.name.as_str(), ())).collect();

    for name in old_state.keys() {
        if !new_field_names.contains_key(name.as_str()) {
            dropped_names.push(name.clone());
        }
    }
    let dropped = dropped_names.len();

    let report = MigrationReport {
        preserved,
        added,
        dropped,
        dropped_names,
    };

    (new_state, report)
}

/// Check if two types are compatible for state migration.
///
/// Compares the short type names (via [`Type::unique_name`]). This is
/// intentionally conservative -- any type name change forces a
/// re-initialization to avoid silent data corruption.
fn types_compatible(a: &Type, b: &Type) -> bool {
    // Use the short_name/unique_name comparison since Type does not impl PartialEq
    match (a, b) {
        // Simple scalars: compare discriminants
        (Type::Int, Type::Int) => true,
        (Type::Uint, Type::Uint) => true,
        (Type::USize, Type::USize) => true,
        (Type::I64, Type::I64) => true,
        (Type::U64, Type::U64) => true,
        (Type::Float, Type::Float) => true,
        (Type::Double, Type::Double) => true,
        (Type::Bool, Type::Bool) => true,
        (Type::Byte, Type::Byte) => true,
        (Type::Char, Type::Char) => true,
        (Type::CStrLit, Type::CStrLit) => true,
        (Type::StrSlice, Type::StrSlice) => true,
        (Type::StrOwned, Type::StrOwned) => true,
        (Type::Void, Type::Void) => true,

        // Str(N): compare as string types regardless of capacity
        (Type::StrFixed(_), Type::StrFixed(_)) => true,

        // For complex types, compare unique names as a string approximation
        _ => a.unique_name() == b.unique_name(),
    }
}

/// Evaluate a base `auto_lang_core::ast::Expr` to a default `Value`.
///
/// Phase 3: this mirrors the `eval_expr_to_value` function in `vm_bridge.rs`,
/// but operates on the base AST `Expr` (the old `AuraExpr` was eliminated).
/// Duplicated here to keep the state migration module self-contained and avoid
/// coupling to the VmBridge internals. `pub(crate)` since PLAN-056's instance
/// migration uses it to seed new model-field defaults onto migrated instances.
pub(crate) fn eval_default(expr: &Expr) -> Value {
    match expr {
        Expr::Int(i) => Value::Int(*i),
        Expr::I64(i) => Value::Int(*i as i32),
        Expr::Uint(u) => Value::Uint(*u),
        Expr::U64(u) => Value::Uint(*u as u32),
        Expr::Byte(b) => Value::Int(*b as i32),
        Expr::I8(i) => Value::Int(*i as i32),
        Expr::U8(u) => Value::Int(*u as i32),
        Expr::Float(f, _) => Value::Double(*f),
        Expr::Double(f, _) => Value::Double(*f),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Char(c) => Value::Int(*c as i32),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::CStr(s) => Value::Str(s.clone()),
        Expr::Ident(_) => Value::Int(0),
        Expr::Bina(_, Op::Add, _) => Value::Int(0),
        Expr::Bina(_, _, _) => Value::Int(0),
        Expr::Unary(_, _) => Value::Int(0),
        Expr::Array(elements) => {
            let values: Vec<Value> = elements.iter().map(eval_default).collect();
            Value::Array(auto_val::Array::from(values))
        }
        Expr::Object(pairs) => {
            let mut obj = auto_val::Obj::new();
            for pair in pairs {
                obj.set(pair.key.to_astr(), eval_default(&pair.value));
            }
            Value::Obj(Box::new(obj))
        }
        // Complex expressions default to Nil for safety
        _ => Value::Nil,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create an AuraStateDef with the given name, type, and initial value.
    fn field(name: &str, type_info: Type, initial: Expr) -> AuraStateDef {
        AuraStateDef {
            name: name.to_string(),
            type_info,
            initial,
            decorators: vec![],
        }
    }

    #[test]
    fn test_migrate_preserves_compatible_fields() {
        let old_state = HashMap::from([
            ("count".to_string(), Value::Int(42)),
            ("label".to_string(), Value::str("hello")),
        ]);
        let old_fields = vec![
            field("count", Type::Int, Expr::Int(0)),
            field("label", Type::StrFixed(0), Expr::Str("".to_string().into())),
        ];
        let new_fields = vec![
            field("count", Type::Int, Expr::Int(0)),
            field("label", Type::StrFixed(0), Expr::Str("".to_string().into())),
        ];

        let (migrated, report) = migrate_state(&old_state, &old_fields, &new_fields);

        assert_eq!(migrated.get("count"), Some(&Value::Int(42)));
        assert_eq!(migrated.get("label"), Some(&Value::str("hello")));
        assert_eq!(report.preserved, 2);
        assert_eq!(report.added, 0);
        assert_eq!(report.dropped, 0);
    }

    #[test]
    fn test_migrate_adds_new_fields() {
        let old_state = HashMap::from([("count".to_string(), Value::Int(5))]);
        let old_fields = vec![field("count", Type::Int, Expr::Int(0))];
        let new_fields = vec![
            field("count", Type::Int, Expr::Int(0)),
            field("enabled", Type::Bool, Expr::Bool(true)),
        ];

        let (migrated, report) = migrate_state(&old_state, &old_fields, &new_fields);

        assert_eq!(migrated.get("count"), Some(&Value::Int(5)));
        assert_eq!(migrated.get("enabled"), Some(&Value::Bool(true)));
        assert_eq!(report.preserved, 1);
        assert_eq!(report.added, 1);
        assert_eq!(report.dropped, 0);
    }

    #[test]
    fn test_migrate_drops_removed_fields() {
        let old_state = HashMap::from([
            ("count".to_string(), Value::Int(5)),
            ("legacy".to_string(), Value::str("old")),
        ]);
        let old_fields = vec![
            field("count", Type::Int, Expr::Int(0)),
            field(
                "legacy",
                Type::StrFixed(0),
                Expr::Str("".to_string().into()),
            ),
        ];
        let new_fields = vec![field("count", Type::Int, Expr::Int(0))];

        let (migrated, report) = migrate_state(&old_state, &old_fields, &new_fields);

        assert_eq!(migrated.get("count"), Some(&Value::Int(5)));
        assert!(migrated.get("legacy").is_none());
        assert_eq!(report.preserved, 1);
        assert_eq!(report.added, 0);
        assert_eq!(report.dropped, 1);
        assert!(report.dropped_names.contains(&"legacy".to_string()));
    }

    #[test]
    fn test_migrate_type_changed_uses_new_default() {
        let old_state = HashMap::from([("value".to_string(), Value::Int(42))]);
        let old_fields = vec![field("value", Type::Int, Expr::Int(0))];
        let new_fields = vec![
            // Type changed from Int to Str
            field(
                "value",
                Type::StrFixed(0),
                Expr::Str("default".to_string().into()),
            ),
        ];

        let (migrated, report) = migrate_state(&old_state, &old_fields, &new_fields);

        // Old int value should NOT be preserved since type changed
        assert_eq!(migrated.get("value"), Some(&Value::str("default")));
        assert_eq!(report.preserved, 0);
        assert_eq!(report.added, 1);
    }

    #[test]
    fn test_migrate_empty_old_state() {
        let old_state = HashMap::new();
        let old_fields: Vec<AuraStateDef> = vec![];
        let new_fields = vec![
            field("count", Type::Int, Expr::Int(10)),
            field(
                "name",
                Type::StrFixed(0),
                Expr::Str("test".to_string().into()),
            ),
        ];

        let (migrated, report) = migrate_state(&old_state, &old_fields, &new_fields);

        assert_eq!(migrated.get("count"), Some(&Value::Int(10)));
        assert_eq!(migrated.get("name"), Some(&Value::str("test")));
        assert_eq!(report.preserved, 0);
        assert_eq!(report.added, 2);
        assert_eq!(report.dropped, 0);
    }

    #[test]
    fn test_migrate_empty_new_fields() {
        let old_state = HashMap::from([("count".to_string(), Value::Int(5))]);
        let old_fields = vec![field("count", Type::Int, Expr::Int(0))];
        let new_fields: Vec<AuraStateDef> = vec![];

        let (migrated, report) = migrate_state(&old_state, &old_fields, &new_fields);

        assert!(migrated.is_empty());
        assert_eq!(report.preserved, 0);
        assert_eq!(report.added, 0);
        assert_eq!(report.dropped, 1);
    }

    #[test]
    fn test_migrate_full_replacement() {
        // Complete replacement: all fields change
        let old_state = HashMap::from([
            ("a".to_string(), Value::Int(1)),
            ("b".to_string(), Value::Int(2)),
        ]);
        let old_fields = vec![
            field("a", Type::Int, Expr::Int(0)),
            field("b", Type::Int, Expr::Int(0)),
        ];
        let new_fields = vec![
            field("x", Type::StrFixed(0), Expr::Str("new".to_string().into())),
            field("y", Type::Bool, Expr::Bool(false)),
        ];

        let (migrated, report) = migrate_state(&old_state, &old_fields, &new_fields);

        assert_eq!(migrated.get("x"), Some(&Value::str("new")));
        assert_eq!(migrated.get("y"), Some(&Value::Bool(false)));
        assert!(migrated.get("a").is_none());
        assert!(migrated.get("b").is_none());
        assert_eq!(report.preserved, 0);
        assert_eq!(report.added, 2);
        assert_eq!(report.dropped, 2);
    }

    #[test]
    fn test_migration_report_dropped_names() {
        let old_state = HashMap::from([
            ("a".to_string(), Value::Int(1)),
            ("b".to_string(), Value::Int(2)),
            ("c".to_string(), Value::Int(3)),
        ]);
        let old_fields = vec![
            field("a", Type::Int, Expr::Int(0)),
            field("b", Type::Int, Expr::Int(0)),
            field("c", Type::Int, Expr::Int(0)),
        ];
        let new_fields = vec![field("a", Type::Int, Expr::Int(0))];

        let (_, report) = migrate_state(&old_state, &old_fields, &new_fields);

        assert_eq!(report.preserved, 1);
        assert_eq!(report.dropped, 2);
        assert_eq!(report.dropped_names.len(), 2);
        assert!(report.dropped_names.contains(&"b".to_string()));
        assert!(report.dropped_names.contains(&"c".to_string()));
    }

    #[test]
    fn test_eval_default_complex_expr() {
        // Complex expressions should default to Int(0) for binary
        let val = eval_default(&Expr::Bina(
            Box::new(Expr::Int(1)),
            Op::Add,
            Box::new(Expr::Int(2)),
        ));
        assert_eq!(val, Value::Int(0)); // Binary defaults to Int(0)
    }

    #[test]
    fn test_eval_default_array() {
        let val = eval_default(&Expr::Array(vec![Expr::Int(1), Expr::Int(2)]));
        match val {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], Value::Int(1));
                assert_eq!(arr[1], Value::Int(2));
            }
            other => panic!("Expected Array, got {:?}", other),
        }
    }
}
