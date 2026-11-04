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

#[cfg(test)]
mod plan059_design_gate_tests {
    use super::*;

    fn sfc_of(src: &str) -> String {
        let session = auto_lang_core::session::CompilerSession::ui();
        let _ = auto_lang_core::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(src).expect("parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract");
        let mut gen = VueGenerator::new_shadcn();
        gen.generate_sfc(&widget).expect("generate")
    }

    /// PLAN-059（auto-os）反控：design.theme 投影（031 F-03）在无
    /// accent_color/theme_name 状态的 App 上必须仍注入主题运行时——
    /// 此前双臂门控静默截断，vue 轨官方包不上身（reader/todo 同形）。
    #[test]
    fn design_only_app_emits_theme_runtime() {
        let sfc = sfc_of(
            r#"widget App {
    msg { Init }
    design { theme: "Auto" }
    view { col { text "hi" {} } }
}
"#,
        );
        assert!(
            sfc.contains("__designTheme"),
            "design projection computed must emit:\n{}",
            sfc
        );
        assert!(
            sfc.contains("__AUTO_TREE_THEMES__"),
            "official tree palette must be consumable at runtime"
        );
    }

    /// 零变化裁定不变：三无 app（无 accent/theme_name/design）维持零注入。
    #[test]
    fn plain_app_stays_zero_injection() {
        let sfc = sfc_of(
            r#"widget App {
    msg { Init }
    view { col { text "hi" {} } }
}
"#,
        );
        assert!(!sfc.contains("__designTheme"));
        assert!(!sfc.contains("__AUTO_TREE_THEMES__"));
        assert!(!sfc.contains("THEME_PALETTES"));
    }
}
