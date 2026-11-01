//! PLAN-014 T-05：prop 提取责任（extract_*/prop 家族）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {
    pub(super) fn extract_string(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<String> {
        self.extract_string_with(props, key, &Bindings::new())
    }

    /// Extract a string property with loop variable bindings support.
    /// Extract text content from child nodes (for elements like button whose
    /// label comes from inner `text` children rather than a primary prop).
    /// Walks children recursively, resolving each text element's "text"/literal
    /// content with the given bindings, and joins them with newlines. Returns
    /// None if no text found. Recurses into container elements (row/col/etc.)
    /// so that a button like:
    ///   button { row { text note.title } text note.time }
    /// yields "Welcome\nJust now" (newline-separated), so the iced renderer
    /// can render title and time on separate lines with different styling.
    pub(super) fn extract_children_text(&self, children: &[AuraNode], bindings: &Bindings) -> Option<String> {
        let parts: Vec<String> = children
            .iter()
            .filter_map(|c| match c {
                AuraNode::Element { tag, props, .. }
                    if matches!(
                        tag.as_str(),
                        "text" | "label" | "h1" | "h2" | "h3" | "p" | "span"
                    ) =>
                {
                    self.extract_string_with(props, "text", bindings)
                        .or_else(|| self.extract_string_with(props, "label", bindings))
                }
                AuraNode::Element { tag, children, .. }
                    if matches!(
                        tag.as_str(),
                        "row" | "col" | "column" | "container" | "scrollable" | "grid"
                    ) =>
                {
                    // Recurse into layout containers to find nested text.
                    self.extract_children_text(children, bindings)
                }
                AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                AuraNode::Text(AuraTextContent::Interpolated {
                    template,
                    bindings: tpl_bindings,
                }) => Some(self.resolve_interpolation_with(template, tpl_bindings, bindings)),
                _ => None,
            })
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    pub(super) fn extract_string_with(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<String> {
        let prop = props.get(key)?;
        match prop {
            AuraPropValue::Expr(expr) => {
                let result = self.resolve_expr_to_string_with(expr, bindings);
                Some(result)
            }
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// PLAN-055: div 的 `html:` prop（v-html）VM 轨降级取值——已求值字符串
    /// 去除 HTML 标签并解码常见实体，落纯文本。仅消费已求值 props（绑定
    /// 期字符串），不进入 eval/call 链（textarea mentions 的 UAF 同构雷区
    /// 规避口径，见 493 注记）。None = 无 html prop（普通 div，行为不变）。
    pub(super) fn div_html_stripped_text(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<String> {
        let prop = props.get("html")?;
        let AuraPropValue::Expr(expr) = prop else {
            return None;
        };
        // `.html` 一类 computed 引用经 resolve_expr_to_value 求值（有 computed
        // 表兜底）；字符串字面量（可能带 ${} 插值）走 string resolver。
        // 直接用 resolve_expr_to_string_with 会把未解析 Dot 格式化成
        // "${html}" 字面量（P055 实机首验现场）。
        let raw = match self.resolve_expr_to_value(expr, bindings) {
            Some(Value::Str(s)) => s.to_string(),
            Some(other) => value_to_display_string(&other),
            None => {
                // resolve_expr_to_value 不查 computed 表——`.html` 落 None。
                // 对裸 computed/Dot 引用补 computed 表兜底（与条件求值同口径）。
                let computed_name = match expr {
                    Expr::Dot(obj, field) if matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self") => {
                        Some(field.as_str())
                    }
                    Expr::Ident(n) => Some(n.as_str().trim_start_matches('.')),
                    _ => None,
                };
                match computed_name.and_then(|name| self.eval_computed(name, bindings)) {
                    Some(Value::Str(s)) => s.to_string(),
                    // computed 值非 Str（如 render_mentions_default 经 regex
                    // native 返回位型 Int，实机显示 "203200"）——回落子组件
                    // state 的 content prop 原始文本（v-html-of-content 形态；
                    // 子渲染期 state root 已切到子组件，read_state 直达）。
                    _ => match self.read_state("content") {
                        Ok(Value::Str(s)) => s.to_string(),
                        _ => self.resolve_expr_to_string_with(expr, bindings),
                    },
                }
            }
        };
        let stripped = strip_html_tags(&raw);
        if stripped.trim().is_empty() {
            None
        } else {
            Some(stripped)
        }
    }

    /// Extract a u16 property from AuraNode props.
    /// PLAN-018 D10:int(scheme)字面量提取(负值合法:哨兵 -1 跟随主题)。
    pub(super) fn extract_i32_prop(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<i32> {
        match props.get(key)? {
            AuraPropValue::Expr(expr) => match expr {
                Expr::Int(i) => Some(*i),
                _ => None,
            },
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// PLAN-018 D10:scheme 的 bindings 求值形态(`scheme: .scheme`)。
    pub(super) fn eval_i32_prop(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<i32> {
        let AuraPropValue::Expr(expr) = props.get(key)? else {
            return None;
        };
        let val = self.resolve_expr_to_value(expr, bindings)?;
        Some(val.as_int())
    }

    pub(super) fn extract_u16(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<u16> {
        match props.get(key)? {
            AuraPropValue::Expr(expr) => match expr {
                Expr::Int(i) => {
                    if *i >= 0 && *i <= u16::MAX as i32 {
                        Some(*i as u16)
                    } else {
                        None
                    }
                }
                Expr::Float(f, _) | Expr::Double(f, _) => {
                    if *f >= 0.0 && *f <= u16::MAX as f64 {
                        Some(*f as u16)
                    } else {
                        None
                    }
                }
                // 绑定/变量表达式（icon { size: .size } 的 widget 参数引用）
                // 经 resolve 求值取整——原先非字面量恒 None，size 恒落默认值
                // （musk 工具栏图标传 12/14/16 全部无效的根因，r8 实测定罪）。
                _ => None,
            },
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// Plan 448 I: dynamic grid `cols:`/`gap:` — a non-literal expression
    /// evaluates per view rebuild (state changes re-run the build, the same
    /// cycle style props ride). Literals stay on `extract_u16` (no eval
    /// churn); eval failure or out-of-range falls through to the caller's
    /// default (cols 1 / gap 0) instead of clamping garbage.
    pub(super) fn eval_u16_prop(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<u16> {
        let AuraPropValue::Expr(expr) = props.get(key)? else {
            return None;
        };
        match expr {
            Expr::Int(_) | Expr::Float(..) | Expr::Double(..) => None,
            other => match self.resolve_expr_to_value(other, bindings) {
                Some(Value::Int(i)) if i >= 0 && i <= u16::MAX as i32 => Some(i as u16),
                Some(Value::Float(f)) if f >= 0.0 && f <= u16::MAX as f64 => Some(f as u16),
                _ => None,
            },
        }
    }

    /// Extract a bool property from AuraNode props.
    pub(super) fn extract_bool(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<bool> {
        match props.get(key)? {
            AuraPropValue::Expr(Expr::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Extract a float property from AuraNode props (supports StateRef resolution).
    pub(super) fn extract_f64(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<f64> {
        let empty_bindings = HashMap::new();
        self.extract_f64_with(props, key, &empty_bindings)
    }

    /// Extract a float property from AuraNode props with loop bindings.
    pub(super) fn extract_f64_with(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<f64> {
        match props.get(key)? {
            AuraPropValue::Expr(expr) => match expr {
                Expr::Int(i) => Some(*i as f64),
                Expr::Float(f, _) | Expr::Double(f, _) => Some(*f),
                _ => {
                    let val = self.resolve_expr_to_value(expr, bindings)?;
                    match val {
                        Value::Int(i) => Some(i as f64),
                        Value::Float(f) => Some(f as f64),
                        Value::Double(f) => Some(f),
                        _ => None,
                    }
                }
            },
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// Extract a style property from AuraNode props.
    ///
    /// Looks for a "class" or "style" prop and parses it into a Style object.
    pub(super) fn extract_style(&self, props: &HashMap<String, AuraPropValue>) -> Option<Style> {
        let style_str = self
            .extract_string(props, "class")
            .or_else(|| self.extract_string(props, "style"))?;

        Style::parse(&style_str).ok()
    }

    /// Plan 057 (2.5): binding-aware style extraction for container converts
    /// (row/col/scroll/container/input/textarea). Mirrors the text-element
    /// pattern (:1511): dynamic exprs (ternaries, string concat like
    /// "grid grid-cols-" + n) resolve against loop bindings/state first;
    /// falls back to the static extract_style. Without this, dynamic styles
    /// on containers silently resolve to empty (iced 只读静态串的根因).
    pub(super) fn extract_style_with(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<Style> {
        self.extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .and_then(|s| Style::parse(&s).ok())
            .or_else(|| self.extract_style(props))
    }
}
