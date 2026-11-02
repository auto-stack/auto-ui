use super::*;

// PLAN-031 F-03：design 表达式 → TS 投影（theme）。
// 支持面：字符串字面量、`.field`/`field` 引用（props 与 state 两种来源）。
// 其它表达式返回 None（回落到 theme_name 保留名约定，并保持零行为变化）。
impl VueGenerator {
    /// 本 widget 的 `design { theme: … }` → 组件内可求值的 TS 表达式。
    pub(super) fn design_theme_ts(&self) -> Option<String> {
        use auto_lang_core::ast::Expr as E;
        let (_, expr) = self
            .design_entries
            .iter()
            .find(|(k, _)| k == "theme" || k == "family")?;
        match expr {
            E::Str(s) => Some(format!("'{}'", Self::escape_js_string(s))),
            E::CStr(s) => Some(format!("'{}'", Self::escape_js_string(s))),
            E::Dot(root, field) => {
                let name = match root.as_ref() {
                    E::Ident(n) if n.as_str() == "." || n.as_str() == "self" => field.as_str(),
                    _ => return None,
                };
                self.ref_to_ts(name)
            }
            E::Ident(n) => self.ref_to_ts(n.as_str().trim_start_matches('.')),
            _ => None,
        }
    }

    /// 已声明字段引用 → TS 取值表达式（prop 与 state 的访问形态不同）。
    fn ref_to_ts(&self, name: &str) -> Option<String> {
        if self.prop_names.iter().any(|p| p == name) {
            return Some(format!("String(props.{name} ?? '')"));
        }
        if self.state_names.iter().any(|s| s == name) {
            return Some(format!("String({name}.value ?? '')"));
        }
        None
    }

    /// 是否声明了可投影的 design 家族绑定（用于运行时注入门）。
    pub(super) fn has_design_theme(&self) -> bool {
        self.design_theme_ts().is_some()
    }
}
