//! PLAN-014 T-05：表达式求值责任（read_state*/resolve_iterable/convert_text_with/eval_computed/resolve_expr_to_value/eval_condition_with*）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {
    /// Plan 320: read a state field. When override_state_obj_id is set (rendering
    /// a child widget's view), reads from the child's state object. Otherwise
    /// reads from the root widget's state (legacy behavior).
    /// Plan 370 D-GAP-4: also handles "store.field" paths by stripping the
    /// "store." prefix (store fields are merged into root state as bare names).
    pub(super) fn read_state(&self, field_name: &str) -> Result<auto_val::Value, String> {
        // Strip "store." prefix for merged store fields
        let name = field_name.strip_prefix("store.").unwrap_or(field_name);
        // PLAN-041 T-02: outlet memo gate resolves read slots against the
        // page instance (scoped); the subtree's own override still wins.
        if let Some(child_id) = self.slot_resolution_override.get().or(self.override_state_obj_id) {
            // Try child state first, fall back to root state for store fields
            match self.bridge.read_child_state(child_id, name) {
                Ok(v) => Ok(v),
                Err(_) => self.bridge.read_state(name).map_err(|e| e.to_string()),
            }
        } else {
            self.bridge.read_state(name).map_err(|e| e.to_string())
        }
    }

    /// Plan 320: read a state field as Vec<Value> (override-aware).
    /// Plan 370 D-GAP-4: also handles "store.field" paths by stripping the
    /// "store." prefix (store fields are merged into root state as bare names).
    pub(super) fn read_state_as_vec(&self, field_name: &str) -> Result<Vec<auto_val::Value>, String> {
        let name = field_name.strip_prefix("store.").unwrap_or(field_name);
        if let Some(child_id) = self.slot_resolution_override.get().or(self.override_state_obj_id) {
            // Try child state first, fall back to root state for store fields
            match self.bridge.read_child_state_as_vec(child_id, name) {
                Ok(v) => Ok(v),
                Err(_) => self
                    .bridge
                    .read_state_as_vec(name)
                    .map_err(|e| e.to_string()),
            }
        } else {
            self.bridge
                .read_state_as_vec(name)
                .map_err(|e| e.to_string())
        }
    }

    /// Plan 370 (Issue 2): resolve a `for`-loop iterable that may be a
    /// dotted path like `.note.tags` or `.store.notes`. Simple field names
    /// (`.notes`) go through `read_state`/`read_state_as_vec`; dotted paths
    /// (`.note.tags`) are resolved field-by-field via `resolve_expr_to_value`
    /// so that a prop object's sub-fields (e.g. a Note's tags array stored
    /// as a heap-id Int) can be iterated.
    /// PLAN-633: for 源/字段读取的 store 别名识别——`.store.X` 字面别名与
    /// 真名限定形态（`.TodoStore.X`，发射器 store 接收者限定产物）统一
    /// 落根态裸字段 X。返回 Some(裸字段名) = store 源；None = 常规路径。
    pub(super) fn store_source_field(&self, stripped: &str) -> Option<String> {
        if !stripped.contains('.') {
            return None;
        }
        let first = stripped.split('.').next().unwrap_or("");
        if first == "store" {
            return stripped.split('.').nth(1).map(|x| x.to_string());
        }
        if crate::ui::handler_codegen::view_store_alias_real_name(first).is_some()
            || self.bridge.store_alias_real_name(first).is_some()
        {
            return stripped.split('.').nth(1).map(|x| x.to_string());
        }
        None
    }
    pub(super) fn resolve_iterable(
        &self,
        iterable: &str,
        bindings: &Bindings,
    ) -> Option<Vec<auto_val::Value>> {
        // Simple state field (no interior dot after stripping the leading '.')
        let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
        let store_field = self.store_source_field(stripped);
        let has_inner_dot = stripped.contains('.') && store_field.is_none();
        let stripped = store_field.as_deref().unwrap_or(stripped);
        if !has_inner_dot {
            // Delegate to existing state-read helpers.
            if let Ok(arr) = self.read_state_as_vec(stripped) {
                return Some(arr);
            }
            // PLAN-051 C3 (KD-047 UPSTREAM①清偿): state 读 miss → computed
            // 求值回退——musk `for msg in .filteredMessages` 以 computed 为
            // for 源，此前直接 None → WARN 刷屏 + 气泡列表整体空。
            if let Some(v) = self.eval_computed(stripped, bindings) {
                let arr = self.value_to_iter_vec(&v);
                // PLAN-536 Phase 2 T13: for 源 computed 每次重建解析行数追踪
                // (KD P536-D2/057④ 定罪:refill 后投影到底见几行)。
                if std::env::var("AUTO_DEBUG_POLLTRACE").is_ok() && stripped == "filteredMessages" {
                    eprintln!(
                        "[POLLTRACE][FOR] computed .{stripped} -> {} rows",
                        arr.as_ref().map(|a| a.len()).unwrap_or(usize::MAX)
                    );
                }
                return arr;
            }
            return None;
        }
        // Dotted path — build an Expr and resolve. `.note.tags` →
        // Dot(Dot(Ident("."), "note"), "tags").
        let expr = Self::parse_dot_path_to_expr(iterable)?;
        let val = self.resolve_expr_to_value(&expr, bindings)?;
        self.value_to_iter_vec(&val)
    }

    /// PLAN-051 C3: computed/state 求值结果 → 迭代 Vec。Value::Array 直取；
    /// Int ≥4M / VmRef 为堆 ListData 引用，经 bridge 展开。
    pub(super) fn value_to_iter_vec(&self, val: &auto_val::Value) -> Option<Vec<auto_val::Value>> {
        match val {
            auto_val::Value::Array(arr) => Some(arr.iter().cloned().collect()),
            // Plan 390 §15 H3b: arrays are ListData<Value> in heap_objects (4M+).
            auto_val::Value::Int(id) if *id >= 4_000_000 => {
                // Heap array id — deref via bridge
                Some(self.bridge.index_list_all(*id as usize))
            }
            auto_val::Value::VmRef(r) => Some(self.bridge.index_list_all(r.id)),
            _ => None,
        }
    }

    /// Build a chained Dot expr from a dotted path string like `.note.tags`.
    pub(super) fn parse_dot_path_to_expr(path: &str) -> Option<Expr> {
        let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();
        if parts.is_empty() {
            return None;
        }
        // First segment is the root (e.g. "" from leading '.', or "note").
        // For `.note.tags`, parts = ["note", "tags"].
        let mut expr = Expr::Ident(parts[0].into());
        for part in &parts[1..] {
            expr = Expr::Dot(Box::new(expr), part.to_string().into());
        }
        Some(expr)
    }
}

impl<'a> AuraViewBuilder<'a> {
    /// Convert an AuraTextContent to a string, resolving interpolations.
    pub(super) fn convert_text_with(
        &self,
        content: &AuraTextContent,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let resolved = match content {
            AuraTextContent::Literal(s) => s.clone(),
            AuraTextContent::Interpolated {
                template,
                bindings: tpl_bindings,
            } => self.resolve_interpolation_with(template, tpl_bindings, bindings),
        };

        View::Text {
            content: resolved,
            style: None,
            selectable: false,
        }
    }

    // ========================================================================
    // State resolution
    // ========================================================================

    /// Resolve a string interpolation template containing `${.field}` references.
    /// Resolve interpolation with loop variable bindings support.
    pub(super) fn resolve_interpolation_with(
        &self,
        template: &str,
        tpl_bindings: &[String],
        loop_bindings: &Bindings,
    ) -> String {
        let mut result = resolve_i18n_template(template);

        for field_name in tpl_bindings {
            // Plan 057 (2.4, ash-gui 表格): 深层点路径绑定(如 .output.Table.atom_type
            // 或 view-fn 参数/循环变量的 output.columns)此前按整段名查 state →
            // Err → 输出 "${...}" 字面量。构造嵌套 Dot 表达式走
            // resolve_expr_to_value(bindings/computed/state + Obj 遍历),失败再回退。
            // 前导点区分根:带点 = self 根;不带 = 裸 Ident 根(首段查 bindings)。
            let pattern_dot = format!(
                "${{{}}}",
                format!(".{}", field_name.trim_start_matches('.'))
            );
            let pattern_bare = format!("${{{}}}", field_name);
            let value_str = if field_name.contains('.') {
                let trimmed = field_name.trim_start_matches('.');
                let mut parts = trimmed.split('.');
                let root_name = parts.next().unwrap_or("");
                let mut expr = Expr::Ident(
                    if field_name.starts_with('.') {
                        ".".to_string()
                    } else {
                        root_name.to_string()
                    }
                    .into(),
                );
                if !field_name.starts_with('.') {
                    for part in parts {
                        expr = Expr::Dot(Box::new(expr), part.into());
                    }
                } else {
                    for part in parts {
                        expr = Expr::Dot(Box::new(expr), part.into());
                    }
                }
                match self.resolve_expr_to_value(&expr, loop_bindings) {
                    Some(v) => value_to_display_string(&v),
                    None => self.read_state_as_string_with(trimmed, loop_bindings),
                }
            } else {
                self.read_state_as_string_with(field_name, loop_bindings)
            };
            result = result.replace(&pattern_dot, &value_str);
            result = result.replace(&pattern_bare, &value_str);
        }

        result
    }

    /// Read a state field value as a display string, checking loop bindings first.
    pub(super) fn read_state_as_string_with(&self, field_name: &str, bindings: &Bindings) -> String {
        // Check loop bindings first (e.g., "note" in `for note in .notes`)
        if let Some(val) = bindings.get(field_name) {
            return value_to_display_string(val);
        }
        // EDGE-16 第五层:查 computed(如 .status_glyph),命中则求值。
        if let Some(val) = self.eval_computed(field_name, bindings) {
            return value_to_display_string(&val);
        }
        match self.read_state(field_name) {
            Ok(value) => value_to_display_string(&value),
            Err(_) => format!("${{{}}}", field_name),
        }
    }

    /// Resolve a base AST `Expr` to a display string with loop variable bindings.
    pub(super) fn resolve_expr_to_string_with(&self, expr: &Expr, bindings: &Bindings) -> String {
        match expr {
            Expr::Str(s) => self.resolve_literal_interpolation_with(s, bindings),
            Expr::Int(i) => i.to_string(),
            Expr::Float(f, _) => f.to_string(),
            Expr::Double(f, _) => f.to_string(),
            Expr::Bool(b) => b.to_string(),
            // State reference: identifier whose name starts with "." (e.g. ".count").
            Expr::Ident(name) => {
                let s = name.as_str();
                if crate::ui::style::has_style_recipe(s) {
                    if let Ok(desugared) = crate::ui::style::expand_recipe_call(s, &[]) {
                        return self.resolve_expr_to_string_with(&desugared, bindings);
                    }
                }
                let field_name = s.trim_start_matches('.');
                self.read_state_as_string_with(field_name, bindings)
            }
            // Field access: object.field → Dot(object, field)
            Expr::Dot(object, field) => {
                // Plan 402: handle .store.X path — flatten to read root state X.
                // Mirrors resolve_expr_to_value's store flattening (D-GAP-4).
                // Without this, `.store.mines_label` resolves to empty (read_state
                // has no "store" field since store fields are bare-named in root
                // state) → empty text node → filtered out → label disappears.
                if let Expr::Dot(inner_obj, store_alias) = object.as_ref() {
                    // PLAN-633: `.store.X` 字面别名之外,真名限定形态
                    // (`.TodoStore.X`,发射器 store 接收者限定产物)同读根态
                    // 裸字段——多 store 语境(画廊宿主)下泛型别名歧义,发射
                    // 期已真名化,视图侧须同口径。
                    if matches!(inner_obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self")
                        && (crate::ui::handler_codegen::view_store_alias_real_name(
                            store_alias.as_str(),
                        )
                        .is_some()
                            || self
                                .bridge
                                .store_alias_real_name(store_alias.as_str())
                                .is_some())
                    {
                        return self.read_state_as_string_with(field.as_str(), bindings);
                    }
                }
                // Plan 370 (Issue 4): single-level self-ref `.field` (parsed as
                // Dot(Ident("."), field)) — read directly from state, mirroring
                // the fix in resolve_expr_to_value. Without this, `.edit_title`
                // falls through to resolve_expr_to_value(Ident(".")) which reads
                // state field "" and returns None → empty string.
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "." || name.as_str() == "self" {
                        return self.read_state_as_string_with(field.as_str(), bindings);
                    }
                }
                let obj_val = self.resolve_expr_to_value(object, bindings);
                let field_str = field.as_str();
                match obj_val {
                    Some(Value::Obj(map)) => map
                        .get(field_str)
                        .map(|v| value_to_display_string(&v))
                        .unwrap_or_default(),
                    Some(Value::Int(id)) if id >= 4_000_000 => {
                        let raw = Value::Int(id);
                        let materialized = self.bridge.materialize_obj_ref(&raw);
                        if let Value::Obj(map) = materialized {
                            map.get(field_str)
                                .map(|v| value_to_display_string(&v))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        }
                    }
                    Some(Value::VmRef(r)) => {
                        let materialized = self.bridge.materialize_obj_ref(&Value::VmRef(r));
                        if let Value::Obj(map) = materialized {
                            map.get(field_str)
                                .map(|v| value_to_display_string(&v))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        }
                    }
                    _ => String::new(),
                }
            }
            // Plan 339: if-expression for conditional string values (e.g. style).
            // Plan 053 后续:遍历全部分支(else-if 链修复,同 resolve_expr_to_value)。
            Expr::If(if_expr) => {
                for branch in &if_expr.branches {
                    let cond_val = self.resolve_expr_to_value(&branch.cond, bindings);
                    let is_true = match cond_val {
                        Some(Value::Bool(false)) | Some(Value::Nil) | None => false,
                        Some(Value::Int(i)) if i == 0 => false,
                        _ => true,
                    };
                    if is_true {
                        // then body must be a single expression (Plan 339 contract)
                        if let Some(stmt) = first_meaningful_stmt(&branch.body) {
                            match stmt {
                                auto_lang_core::ast::Stmt::Expr(e) => {
                                    return self.resolve_expr_to_string_with(e, bindings)
                                }
                                auto_lang_core::ast::Stmt::If(nested_if) => {
                                    return self.resolve_expr_to_string_with(
                                        &auto_lang_core::ast::Expr::If(nested_if.clone()),
                                        bindings,
                                    );
                                }
                                _ => {}
                            }
                        }
                        return String::new();
                    }
                }
                if let Some(else_body) = &if_expr.else_ {
                    if let Some(stmt) = first_meaningful_stmt(else_body) {
                        match stmt {
                            auto_lang_core::ast::Stmt::Expr(e) => {
                                return self.resolve_expr_to_string_with(e, bindings)
                            }
                            auto_lang_core::ast::Stmt::If(nested_if) => {
                                return self.resolve_expr_to_string_with(
                                    &auto_lang_core::ast::Expr::If(nested_if.clone()),
                                    bindings,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                String::new()
            }
            // Plan 057 (2.4, ash-gui 表格列对齐): style 里的字符串拼接
            // (如 "grid grid-cols-" + n)此前无 Bina 臂 → 求值为空串,动态
            // 拼的 Tailwind 类整体丢失。委托 resolve_expr_to_value 的
            // Bina(Add)(display-form 拼接,958819ab2 已实现)。
            Expr::Bina(_, Op::Add, _) => match self.resolve_expr_to_value(expr, bindings) {
                Some(v) => value_to_display_string(&v),
                None => String::new(),
            },
            // Plan 437 Phase 2:索引表达式(`seg["d"]`)—— svg 子树动态
            // 属性此前无 Index 臂 → 空串 → attr 被丢(循环段记录的路径/
            // 颜色整体消失)。委托 resolve_expr_to_value 的 Index 臂
            // (含 VmRef 记录物化)后取 display 形态。
            Expr::Index(_, _) => match self.resolve_expr_to_value(expr, bindings) {
                Some(v) => value_to_display_string(&v),
                None => String::new(),
            },
            // PLAN-050 T9 (C7): t("k")/i18n.t("k") 文本/prop 位——经查表
            // 取文案（未命中回落 key），此前无 Call 臂 → 恒空串。
            Expr::Call(call) => {
                if let Expr::Ident(name) = call.name.as_ref() {
                    if crate::ui::style::has_style_recipe(name.as_str()) {
                        if let Ok(desugared) =
                            crate::ui::style::expand_recipe_call(name.as_str(), &call.args.args)
                        {
                            return self.resolve_expr_to_string_with(&desugared, bindings);
                        }
                    }
                }
                if let Some(key) = call_expr_t_key(call) {
                    // PLAN-051 P2-②b: 第二实参记录字面量 → {k} 插值。
                    let params = {
                        let resolver = |e: &auto_lang_core::ast::Expr| {
                            Some(self.resolve_expr_to_string_with(e, bindings))
                        };
                        t_call_params(&resolver, call)
                    };
                    let text = crate::ui::i18n_lookup::lookup(&key).unwrap_or(key);
                    return crate::ui::i18n_lookup::substitute_params(&text, &params);
                }
                match self.resolve_expr_to_value(expr, bindings) {
                    Some(v) => value_to_display_string(&v),
                    None => String::new(),
                }
            }
            // D-GAP-4 (013-todo footer): `text f"${.store.active_count} items
            // left"` parses the f-string as the text ELEMENT'S PRIMARY PROP
            // (Expr::FStr), unlike a bare child f-string which becomes
            // AuraTextContent::Interpolated. No FStr arm here → empty string →
            // the text node dropped as visually-empty → later siblings SHIFT
            // one slot left, misaligning every probe/VTree path after it
            // (footer buttons lost their style/event records). Evaluate each
            // part (literals are Expr::Str; interpolations recurse through
            // the Ident/Dot arms, so `.store.X` flattening applies too).
            // Same drop reproduced App-level (031 `text f"selected: …"`).
            Expr::FStr(fstr) => fstr
                .parts
                .iter()
                .map(|part| self.resolve_expr_to_string_with(part, bindings))
                .collect::<Vec<_>>()
                .join(""),
            // Plan 448 D: `style: ["a", if … {…} else {…}]` — array-of-class-
            // parts prop. Join each resolved part with a single space,
            // skipping empties (a conditional part may yield ""), so
            // "items-center gap-1" + optional "opacity-40" never double-
            // spaces. Elements resolve through the full chain (Str/If/
            // Ident/Dot refs all supported — the array form is explicitly
            // class semantics by construction).
            Expr::Array(elems) => elems
                .iter()
                .map(|e| self.resolve_expr_to_string_with(e, bindings))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
            // Plan 448 D: manual string concat in prop position
            // (`style: "base" + if … {…} else {…}`). Previously fell to the
            // empty-string fallback — a concat style silently vanished on
            // the VM path.
            Expr::Bina(l, Op::Add, r) => format!(
                "{}{}",
                self.resolve_expr_to_string_with(l, bindings),
                self.resolve_expr_to_string_with(r, bindings)
            ),
            _ => String::new(),
        }
    }

    /// EDGE-16 第五层:求值 computed 属性。查 self.computed 表,命中则用其
    /// expr 在当前 bindings 下递归求值(resolve_expr_to_value)。用 visited
    /// 防递归循环(computed 引用自身)。未命中返回 None。
    pub(super) fn eval_computed(&self, name: &str, bindings: &Bindings) -> Option<Value> {
        if let Some(computed_list) = self.computed {
            if let Some(c) = computed_list.iter().find(|c| c.name == name) {
                // 防 computed 递归引用自身(bindings 里已有同名则跳过)
                if !bindings.contains_key(name) {
                    // PLAN-047 T-06（档 C SD-10）: computed 信号网——bindings-free
                    // 保守面（bindings 参与的求值位置相关，不入网沿 v1 边界；
                    // keyed-for 项内 computed 沿项级条目承载）。inline 表达式与
                    // block 体隐藏 VM fn 双通道同构：命中复用缓存值，miss 录制
                    // 求值入网；guard 嵌套使外层 memo 条目/信号收编本信号的
                    // dep 面（pull 式级联）。
                    if bindings.is_empty() {
                        if let Some(hit) = self.bridge.computed_signal_hit(&self.widget_name, name)
                        {
                            return Some(hit);
                        }
                        let (out, rec) = self.fill_with_recording(|| match &c.expr {
                            Expr::Block(_) => {
                                self.bridge.call_computed_fn(&self.widget_name, name).ok()
                            }
                            _ => self.resolve_expr_to_value(&c.expr, bindings),
                        });
                        if let Some(v) = &out {
                            self.bridge.computed_signal_store(
                                &self.widget_name,
                                name,
                                v.clone(),
                                &rec,
                            );
                        }
                        return out;
                    }
                    // Plan 448 H2: block-bodied computeds execute as hidden
                    // VM fns (`__computed_<Widget>_<Prop>`, synthesized with
                    // the handlers) — statement semantics the inline
                    // expression resolver cannot provide. A missing fn (older
                    // construction path) degrades to None, the old behavior.
                    if let Expr::Block(_) = &c.expr {
                        return self.bridge.call_computed_fn(&self.widget_name, name).ok();
                    }
                    return self.resolve_expr_to_value(&c.expr, bindings);
                }
            }
        }
        None
    }

    /// Resolve a base AST `Expr` to a Value, checking loop bindings and VmBridge state.
    pub(super) fn resolve_expr_to_value(&self, expr: &Expr, bindings: &Bindings) -> Option<Value> {
        // PLAN-047 T-05: 求值通道探针（AC-02 零重解析断言观测面——version_fast
        // 命中帧本函数零调用；relaxed 原子累计，诊断面零行为影响）。
        self.bridge
            .resolve_probe_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match expr {
            // State reference: identifier whose name starts with "." (e.g. ".count")
            // or a plain identifier (loop var / state).
            Expr::Ident(name) => {
                let field_name = name.as_str().trim_start_matches('.');
                // Plan 057 (Bug 3): bare `self` / `.` — the ROOT of deep dot
                // chains (parser dot_item builds .a.b.c as
                // Dot(Dot(Ident("self"), a), b)...). Previously treated as a
                // field NAMED "self" → None → the whole chain resolved empty →
                // text nodes filtered as visually-empty (表格块的动态文本
                // 因此整体不显示;单层 .field 因 Dot 臂特判侥幸存活)。
                // Resolve to the state OBJECT itself so chained .field reads
                // walk it naturally.
                if field_name == "self" || field_name.is_empty() {
                    let id = self.bridge.state_obj_id();
                    return Some(self.bridge.materialize_obj_ref(&Value::Int(id as i32)));
                }
                bindings
                    .get(field_name)
                    .cloned()
                    .or_else(|| self.eval_computed(field_name, bindings))
                    .or_else(|| self.read_state(field_name).ok())
            }
            // Field access: object.field → Dot(object, field)
            Expr::Dot(object, field) => {
                // Plan 370 D-GAP-4: handle .store.X path — flatten to read root state X.
                // Store fields are merged into root state as bare names.
                if let Expr::Dot(inner_obj, store_alias) = object.as_ref() {
                    // PLAN-748 T-02：泛名 `store` 别名无条件展平——与
                    // read_state（无条件剥 "store." 前缀）和 store_source_field
                    //（泛名无条件）同口径。空快照（无 store 工程）拒展平使
                    // `.store.X` 实参链整链断裂（P733-R2 p053_1 两红根因：
                    // 无 store 声明语料的 musk 同构 `.store.messages`
                    // computed 链在子件/根件均 None）；真名限定仍走快照门
                    //（PLAN-633 语义保持）。
                    if matches!(inner_obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self")
                        && store_alias.as_str() == "store"
                    {
                        return self.read_state(field.as_str()).ok();
                    }
                    // PLAN-633: 同上——真名限定形态与 `.store.X` 同读根态。
                    if matches!(inner_obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self")
                        && (crate::ui::handler_codegen::view_store_alias_real_name(
                            store_alias.as_str(),
                        )
                        .is_some()
                            || self
                                .bridge
                                .store_alias_real_name(store_alias.as_str())
                                .is_some())
                    {
                        return self.read_state(field.as_str()).ok();
                    }
                }
                // Also handle bare "store.X" (Ident("store") Dot field)
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "store" {
                        return self.read_state(field.as_str()).ok();
                    }
                    // Plan 370 (Issue 2): handle single-level self-reference
                    // `.field` (parsed as Dot(Ident("."), field)). Without this,
                    // `.note` in a child widget (e.g. EditorPanel's `text note.title`)
                    // falls through to resolve_expr_to_value(Ident(".")) which
                    // reads state field "" and fails → note resolves to None →
                    // the entire panel renders empty.
                    if name.as_str() == "." || name.as_str() == "self" {
                        // EDGE-16 第五层:`.field` 可能是循环变量(for b in ...
                        // 里的 .b),先查 bindings,再回退 state。
                        if let Some(v) = bindings.get(field.as_str()) {
                            return Some(v.clone());
                        }
                        return self
                            .eval_computed(field.as_str(), bindings)
                            .or_else(|| self.read_state(field.as_str()).ok());
                    }
                }
                let obj = self.resolve_expr_to_value(object, bindings)?;
                let field_str = field.as_str();
                match obj {
                    Value::Obj(map) => map.get(field_str),
                    // Plan 320: raw struct heap id from Index — materialize to Obj
                    // so FieldAccess can read fields.
                    Value::Int(id) if id >= 4_000_000 => {
                        let raw = Value::Int(id);
                        let materialized = self.bridge.materialize_obj_ref(&raw);
                        if let Value::Obj(map) = materialized {
                            map.get(field_str)
                        } else {
                            None
                        }
                    }
                    // Plan 322: VmRef struct instances (from list rebuild after delete)
                    Value::VmRef(r) => {
                        let materialized = self.bridge.materialize_obj_ref(&Value::VmRef(r));
                        if let Value::Obj(map) = materialized {
                            map.get(field_str)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            // Index access: target[index]
            Expr::Index(target, index) => {
                let target_val = self.resolve_expr_to_value(target, bindings)?;
                let index_val = self.resolve_expr_to_value(index, bindings)?;
                match (&target_val, &index_val) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let idx = *i as usize;
                        if idx < arr.len() {
                            Some(arr[idx].clone())
                        } else {
                            None
                        }
                    }
                    (Value::Obj(map), Value::Str(key)) => map.get(key.as_str()),
                    // Plan 318: index into a list/array stored as a VmRef or Int
                    // array_id (List<T> / Vec from `var x = []; x.push(...)`). The
                    // EditorPanel's `note: .notes[.active_id]` reads `.notes`
                    // (a VmRef to ListData) and indexes it. Deref to Vec<Value>
                    // first, then index. Use read_state_as_vec via a temp field
                    // name when the target is a StateRef; otherwise deref inline.
                    (Value::VmRef(r), Value::Int(i)) => {
                        // Plan 318/337: return the raw element (struct heap id)
                        // without materializing. View FieldAccess handles
                        // materialization via materialize_obj_ref; handler
                        // GET_FIELD needs the raw id to do heap_objects lookup.
                        self.bridge.index_list(r.id, *i)
                    }
                    // Plan 390 §15 H3b: arrays are ListData<Value> in heap_objects (4M+).
                    (Value::Int(id), Value::Int(i)) if *id >= 4_000_000 => {
                        self.bridge.index_list(*id as usize, *i)
                    }
                    _ => None,
                }
            }
            Expr::Int(i) => Some(Value::Int(*i)),
            Expr::Float(f, _) => Some(Value::Double(*f)),
            Expr::Double(f, _) => Some(Value::Double(*f)),
            Expr::Bool(b) => Some(Value::Bool(*b)),
            Expr::Str(s) => Some(Value::Str(s.clone())),
            // Plan 437 Phase 2: list literals in child-prop bindings (chart
            // 组件的 fields/colors)——逐元素解析为 Value::Array;任一元素
            // 失败整体 None(与现有 arm 的保守语义一致)。ensure_child_state
            // 会把 Array 转成 heap ListData(VM bytecode 读法)。
            Expr::Array(elems) => {
                let mut values = Vec::with_capacity(elems.len());
                for e in elems {
                    match self.resolve_expr_to_value(e, bindings) {
                        Some(v) => values.push(v),
                        None => return None,
                    }
                }
                Some(Value::Array(auto_val::Array { values }))
            }
            // PLAN-733 T-02: 对象字面量臂——computed 实参/嵌套元素里的
            // `{ key: v }` 形此前落 `_ => None`，整个实参解析判 incomplete
            // → fn 调用 computed 整体字面回退（`${cc}` 形）。键限字面量
            // 形态（Named/Int/Bool/Str Key），值递归求值。
            Expr::Object(pairs) => {
                let mut obj = auto_val::Obj::new();
                for pair in pairs {
                    let v = self.resolve_expr_to_value(&pair.value, bindings)?;
                    let key: auto_val::ValueKey = match &pair.key {
                        auto_lang_core::ast::Key::NamedKey(n) => {
                            auto_val::ValueKey::Str(n.to_string().into())
                        }
                        auto_lang_core::ast::Key::IntKey(i) => auto_val::ValueKey::Int(*i),
                        auto_lang_core::ast::Key::BoolKey(b) => auto_val::ValueKey::Bool(*b),
                        auto_lang_core::ast::Key::StrKey(s) => {
                            auto_val::ValueKey::Str(s.as_str().to_string().into())
                        }
                    };
                    obj.set(key, v);
                }
                Some(Value::Obj(Box::new(obj)))
            }
            // PLAN-048 L1 (musk VM 数据桥): None/Nil/Null literals resolve to
            // Value::Nil so computed comparisons like `.token != None` can
            // evaluate. Previously the literal fell to `_ => None`, voiding
            // any computed that compared against None (auth gate).
            Expr::None | Expr::Nil | Expr::Null => Some(Value::Nil),
            // Plan 339: binary comparison for if-expressions (e.g., i == .active_index)
            Expr::Bina(left, Op::Eq, right) => {
                let l = self.resolve_expr_to_value(left, bindings)?;
                let r = self.resolve_expr_to_value(right, bindings)?;
                Some(Value::Bool(l == r))
            }
            Expr::Bina(left, Op::Neq, right) => {
                let l = self.resolve_expr_to_value(left, bindings)?;
                let r = self.resolve_expr_to_value(right, bindings)?;
                Some(Value::Bool(l != r))
            }
            // Plan 053 后续(ash-gui VM): && / || for computed conditions (e.g.
            // cwd_display's `.store.home != "" && .store.cwd.starts_with(...)`).
            // Without these the whole condition resolves to None and the computed
            // falls back to the literal "${name}" placeholder.
            Expr::Bina(left, Op::And, right) => {
                let l = self.resolve_expr_to_value(left, bindings)?;
                if matches!(&l, Value::Bool(false) | Value::Nil) {
                    return Some(Value::Bool(false));
                }
                let r = self.resolve_expr_to_value(right, bindings)?;
                Some(Value::Bool(!matches!(&r, Value::Bool(false) | Value::Nil)))
            }
            Expr::Bina(left, Op::Or, right) => {
                let l = self.resolve_expr_to_value(left, bindings)?;
                if !matches!(&l, Value::Bool(false) | Value::Nil) {
                    return Some(Value::Bool(true));
                }
                let r = self.resolve_expr_to_value(right, bindings)?;
                Some(Value::Bool(!matches!(&r, Value::Bool(false) | Value::Nil)))
            }
            // PLAN-077 (auto-musk ReportCard/ToolGateCard computed `??`): left
            // resolves to a non-Nil value → left; otherwise (resolution miss or
            // Nil) → right. Mirrors the VM bytecode `??` semantics for the
            // view-build fast path; previously fell to `_ => None`, voiding
            // any computed containing `??` and rendering the literal
            // "${name}" placeholder in VM snapshots.
            Expr::NullCoalesce(left, right) => match self.resolve_expr_to_value(left, bindings) {
                Some(v) if !matches!(v, Value::Nil) => Some(v),
                _ => self.resolve_expr_to_value(right, bindings),
            },
            // Plan 053 后续(ash-gui VM): string concat via `+` (e.g. `"~" + .store.cwd...`
            // in cwd_display, `"⚙ " + .store.job_list.len()` in jobs_label). When either
            // operand resolves to a string, concatenate display forms.
            Expr::Bina(left, Op::Add, right) => {
                let l = self.resolve_expr_to_value(left, bindings)?;
                let r = self.resolve_expr_to_value(right, bindings)?;
                let is_str = |v: &Value| matches!(v, Value::Str(_) | Value::String(_));
                if is_str(&l) || is_str(&r) {
                    let mut s = value_to_display_string(&l);
                    s.push_str(&value_to_display_string(&r));
                    return Some(Value::Str(s.into()));
                }
                match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Some(Value::Int(a + b)),
                    (Value::Double(a), Value::Double(b)) => Some(Value::Double(a + b)),
                    (Value::Float(a), Value::Float(b)) => Some(Value::Float(a + b)),
                    _ => None,
                }
            }
            // Plan 053 后续(ash-gui VM): a small set of string/list method calls on
            // state-backed receivers — starts_with / substr / len / trim — enough for
            // the cwd abbreviation computed and siblings. Generic dispatch stays in
            // the VM; this is the view-build fast path only.
            Expr::Call(call) => {
                // PLAN-050 T9 (C7): t("k")/i18n.t("k") prop 位最小查表。
                // PLAN-733: 键动态化——t(r.label_key)/t("prefix_" + r.state)
                // 形此前非字面量键直接落空（键臂返回 None → 裸 Call 臂按未
                // 知 fn 处理 → 整体 None → 行文本空被过滤）。动态解析首实参
                // 为串后同表查表；查不到回显键串（与字面量臂缺键回显同语义）。
                let t_dynamic_key = match call.name.as_ref() {
                    Expr::Ident(name) if name.as_str() == "t" => Some(()),
                    Expr::Dot(_, method) if method.as_str() == "t" => Some(()),
                    _ => None,
                };
                if t_dynamic_key.is_some() {
                    let key = call_expr_t_key(call).or_else(|| {
                        call.args.args.first().and_then(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e) => {
                                let s = self.resolve_expr_to_string_with(e, bindings);
                                if s.is_empty() {
                                    None
                                } else {
                                    Some(s)
                                }
                            }
                            _ => None,
                        })
                    });
                    if let Some(key) = key {
                        // PLAN-051 P2-②b: 第二实参记录字面量 → {k} 插值。
                        let params = {
                            let resolver = |e: &auto_lang_core::ast::Expr| {
                                Some(self.resolve_expr_to_string_with(e, bindings))
                            };
                            t_call_params(&resolver, call)
                        };
                        let text = crate::ui::i18n_lookup::lookup(&key).unwrap_or_else(|| key);
                        return Some(Value::Str(
                            crate::ui::i18n_lookup::substitute_params(&text, &params).into(),
                        ));
                    }
                }
                // PLAN-051 C3: 裸 fn 名调用——computed 体内的 use.web helper
                // 别名链（musk filteredMessages => chatSearchFilter(
                // chatActivePath(.store.messages, ...), ...)）。实参逐个递归
                // 求值后经 bridge.call_vm_fn 在 VM 内执行（裸名/限定名双查）。
                // 失败沿既有保守语义返回 None（回落字面量/空）。
                if let Expr::Ident(fname) = call.name.as_ref() {
                    let mut arg_vals: Vec<Value> = Vec::with_capacity(call.args.args.len());
                    let mut complete = true;
                    for a in &call.args.args {
                        match a {
                            auto_lang_core::ast::Arg::Pos(e) => {
                                match self.resolve_expr_to_value(e, bindings) {
                                    Some(v) => arg_vals.push(v),
                                    None => {
                                        complete = false;
                                        break;
                                    }
                                }
                            }
                            _ => {
                                complete = false;
                                break;
                            }
                        }
                    }
                    if complete {
                        match self.bridge.call_vm_fn(fname.as_str(), &arg_vals) {
                            Ok(v) => return Some(v),
                            Err(e) => {
                                // PLAN-053 P-053-6: computed/prop 位置的 helper
                                // 调用失败此前被静默吞掉（None → 空串/空列表，
                                // 无任何线索——musk 消息正文空即是此形态藏了
                                // CALL_SPEC Regex 报错整整一批）。AUTO_DEBUG_EMIT
                                // 下显形。
                                if std::env::var("AUTO_DEBUG_EMIT").is_ok() {
                                    eprintln!(
                                        "[VM-CALLFN-VIEW] {} in {} FAILED: {:?} args={:?}",
                                        fname, self.widget_name, e, arg_vals
                                    );
                                }
                            }
                        }
                    }
                    return None;
                }
                if let Expr::Dot(recv_expr, method) = call.name.as_ref() {
                    let recv = self.resolve_expr_to_value(recv_expr, bindings)?;
                    let arg = |i: usize| -> Option<i32> {
                        call.args.args.get(i).and_then(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e) => self
                                .resolve_expr_to_value(e, bindings)
                                .and_then(|v| match v {
                                    Value::Int(n) => Some(n),
                                    Value::Float(f) => Some(f as i32),
                                    Value::Double(f) => Some(f as i32),
                                    _ => None,
                                }),
                            _ => None,
                        })
                    };
                    let arg_str = |i: usize| -> Option<String> {
                        call.args.args.get(i).and_then(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e) => self
                                .resolve_expr_to_value(e, bindings)
                                .map(|v| value_to_display_string(&v)),
                            _ => None,
                        })
                    };
                    fn as_str(v: &Value) -> std::borrow::Cow<'_, str> {
                        match v {
                            Value::Str(s) => std::borrow::Cow::Borrowed(s.as_str()),
                            Value::String(s) => std::borrow::Cow::Owned(s.to_string()),
                            other => std::borrow::Cow::Owned(value_to_display_string(other)),
                        }
                    };
                    match method.as_str() {
                        "len" => match &recv {
                            Value::Str(v) => Some(Value::Int(v.as_str().len() as i32)),
                            Value::String(v) => Some(Value::Int(v.len() as i32)),
                            Value::Array(arr) => Some(Value::Int(arr.len() as i32)),
                            _ => None,
                        },
                        "starts_with" => {
                            let pre = arg_str(0)?;
                            Some(Value::Bool(as_str(&recv).starts_with(&pre)))
                        }
                        "substr" => {
                            let s = as_str(&recv).into_owned();
                            let start = arg(0)? as usize;
                            let end = arg(1).map(|e| e as usize).unwrap_or(s.len());
                            let start = start.min(s.len());
                            let end = end.max(start).min(s.len());
                            Some(Value::Str(s[start..end].to_string().into()))
                        }
                        "trim" => {
                            let s = as_str(&recv).trim().to_string();
                            Some(Value::Str(s.into()))
                        }
                        // Plan 053 后续:`.str()` / `.to_string()` — exit_label 等
                        // computed 用 `":" + .block.exit_code.str()` 拼串。
                        "str" | "to_string" => {
                            let s = value_to_display_string(&recv);
                            Some(Value::Str(s.into()))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            // Plan 339: conditional if-expression for style/attribute values.
            // Plan 053 后续:遍历全部分支 —— else-if 链在 AST 里是 branches 数组的
            // 多个条目,只看 branches.first() 会跳过中间分支(如 kind==Dir 命中而
            // kind==CodeAtRs 永远落 else)。
            Expr::If(if_expr) => {
                // PLAN-054 T3: 分支体求值失败不整链报废——`if x != None { x.f }
                // else { fallback }` 的兜底意图是"取不到就落默认";此前 then 体
                // 任一点 None(如 x 为零默认 Int(0) 时 x.f 解析失败)→ 整个
                // computed → None → 文本位出 "${name}" 字面量(musk 工作区行
                // "Y${currentTitle}" 现场)。改为:分支体落空继续走后续
                // else-if/else 链,真实 obj 全路径无损。
                let mut selected: Option<Value> = None;
                let mut decided = false;
                for branch in &if_expr.branches {
                    let Some(cond_val) = self.resolve_expr_to_value(&branch.cond, bindings) else {
                        continue;
                    };
                    let is_true = match &cond_val {
                        Value::Bool(false) | Value::Nil => false,
                        Value::Int(i) if *i == 0 => false,
                        _ => true,
                    };
                    if !is_true {
                        continue;
                    }
                    decided = true;
                    // then body must be a single expression (Plan 339 contract)
                    if let Some(stmt) = first_meaningful_stmt(&branch.body) {
                        match stmt {
                            auto_lang_core::ast::Stmt::Expr(e) => {
                                match self.resolve_expr_to_value(e, bindings) {
                                    Some(v) => selected = Some(v),
                                    None => decided = false,
                                }
                            }
                            auto_lang_core::ast::Stmt::If(nested_if) => {
                                match self.resolve_expr_to_value(
                                    &auto_lang_core::ast::Expr::If(nested_if.clone()),
                                    bindings,
                                ) {
                                    Some(v) => selected = Some(v),
                                    None => decided = false,
                                }
                            }
                            _ => {}
                        }
                    }
                    if selected.is_some() {
                        break;
                    }
                }
                if selected.is_none() && !decided {
                    if let Some(else_body) = &if_expr.else_ {
                        if let Some(stmt) = first_meaningful_stmt(else_body) {
                            match stmt {
                                auto_lang_core::ast::Stmt::Expr(e) => {
                                    selected = self.resolve_expr_to_value(e, bindings);
                                }
                                auto_lang_core::ast::Stmt::If(nested_if) => {
                                    selected = self.resolve_expr_to_value(
                                        &auto_lang_core::ast::Expr::If(nested_if.clone()),
                                        bindings,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                selected
            }
            // D-GAP-4: f-string in a value position (non-primary-prop props,
            // style_obj values, …) — evaluate to the joined display string
            // (mirrors the resolve_expr_to_string_with FStr arm).
            Expr::FStr(fstr) => {
                let s = fstr
                    .parts
                    .iter()
                    .map(|part| self.resolve_expr_to_string_with(part, bindings))
                    .collect::<Vec<_>>()
                    .join("");
                Some(Value::Str(s.into()))
            }
            _ => None,
        }
    }

    /// Check if a loop item matches the current search filter.
    ///
    /// Reads the `search` state field. If it's empty or doesn't exist, all items match.
    /// If non-empty and the item is an Obj, checks if any string field contains the search text.
    pub(super) fn matches_search(&self, item: &Value) -> bool {
        let search_text = match self.read_state("search") {
            Ok(Value::Str(s)) => s.to_string(),
            Ok(Value::String(s)) => s.to_string(),
            _ => return true, // no search field or non-string → show all
        };
        if search_text.is_empty() {
            return true;
        }
        let search_lower = search_text.to_lowercase();
        match item {
            Value::Obj(map) => {
                // Check title field for a match
                let title = map
                    .get("title")
                    .map(|v| value_to_display_string(&v))
                    .unwrap_or_default();
                title.to_lowercase().contains(&search_lower)
            }
            _ => true, // non-obj items always match
        }
    }

    /// Find an operator (e.g., " || " or " && ") at parenthesis depth 0 only.
    /// Returns the byte position of the operator start, or None if not found at top level.
    pub(super) fn find_operator_at_depth0(cond: &str, op: &str) -> Option<usize> {
        let mut depth = 0i32;
        let op_bytes = op.as_bytes();
        let cond_bytes = cond.as_bytes();
        let mut i = 0;
        while i + op_bytes.len() <= cond_bytes.len() {
            match cond_bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            if depth == 0 && cond_bytes[i..].starts_with(op_bytes) {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Evaluate a condition string against current state with loop variable bindings.
    ///
    /// Supports patterns like:
    /// - `.running == "true"` — state ref compared to string literal
    /// - `.count > 0` — state ref compared to number
    /// - `.active_id == i` — state ref compared to loop index variable
    /// - `.flag` — bare state ref (truthy check)
    pub(super) fn eval_condition_with(&self, condition: &str, bindings: &Bindings) -> bool {
        let cond = condition.trim();
        let r = self.eval_condition_with_inner(condition, bindings);
        // PLAN-051 DIAG (temp): condition-arm truth probe.
        if cond.contains("use_engine") || cond.contains("editing_format") {
            eprintln!("[PLAN051-COND] {cond:?} => {r}");
        }
        r
    }

    pub(super) fn eval_condition_with_inner(&self, cond: &str, bindings: &Bindings) -> bool {
        let cond = cond.trim();

        // Strip outer parentheses for grouped expressions like (...) —
        // do this BEFORE operator splitting so that inner || / && are not split prematurely.
        // Repeat in case of nested parens like ((expr)).
        let mut cond = cond;
        loop {
            if cond.starts_with('(') && cond.ends_with(')') {
                // Verify the closing ')' matches the opening '(' (balanced)
                let mut depth = 0i32;
                let mut matched = true;
                for (i, ch) in cond.char_indices() {
                    match ch {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    if depth == 0 && i < cond.len() - 1 {
                        // Closing paren found before end — outer parens don't match
                        matched = false;
                        break;
                    }
                }
                if matched {
                    cond = cond[1..cond.len() - 1].trim();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Handle || (OR) — split at top level only (paren depth 0)
        if let Some(pos) = Self::find_operator_at_depth0(cond, " || ") {
            let left = &cond[..pos];
            let right = &cond[pos + 4..];
            return self.eval_condition_with(left, bindings)
                || self.eval_condition_with(right, bindings);
        }

        // Plan 049: leading negation — `! .collapsed` / `!.collapsed` / `!x`.
        // Handled AFTER || / && (so `!a || b` splits on || first, then the left
        // arm recurses here) but BEFORE operator splitting, so `a != b` (the
        // "!=" appears mid-string, not as a prefix) is unaffected. Guard "!="
        // as a bare prefix (e.g. `!= something`) — that is a comparison, not
        // a negation.
        if cond.starts_with('!') {
            let rest = cond[1..].trim_start();
            if !rest.starts_with('=') {
                return !self.eval_condition_with(rest, bindings);
            }
        }

        // Handle && (AND) — split at top level only (paren depth 0)
        if let Some(pos) = Self::find_operator_at_depth0(cond, " && ") {
            let left = &cond[..pos];
            let right = &cond[pos + 4..];
            return self.eval_condition_with(left, bindings)
                && self.eval_condition_with(right, bindings);
        }

        // Find operator to split into lhs op rhs
        let (lhs, op, rhs) = if let Some(pos) = cond.find(" == ") {
            (&cond[..pos], "==", cond[pos + 4..].trim())
        } else if let Some(pos) = cond.find(" != ") {
            (&cond[..pos], "!=", cond[pos + 4..].trim())
        } else if let Some(pos) = cond.find(" > ") {
            (&cond[..pos], ">", cond[pos + 3..].trim())
        } else if let Some(pos) = cond.find(" < ") {
            (&cond[..pos], "<", cond[pos + 3..].trim())
        } else if let Some(pos) = cond.find(" >= ") {
            (&cond[..pos], ">=", cond[pos + 4..].trim())
        } else if let Some(pos) = cond.find(" <= ") {
            (&cond[..pos], "<=", cond[pos + 4..].trim())
        } else if cond.starts_with('.') {
            // Bare state ref — truthy check.
            // Plan 370 (Issue 2): dotted prop paths like `.note.pinned`
            // need field-by-field resolution (read_state only looks up a
            // single field name, so `read_state("note.pinned")` fails).
            let path = &cond[1..];
            // PLAN-749 G3: store 前缀多段路径（`.store.env.ok`——存储可达
            // 嵌套对象读，PLAN-081 家族）此前被排除在 expr 通道外，只能落
            // read_state 全串名必 miss → 末段 computed 兜底 → 恒 false。
            // resolve_expr_to_value 的 Dot 臂含 Ident("store") 特判（比较臂
            // 同款通道实测工作），解除排除。
            let has_inner_dot = path.contains('.');
            if has_inner_dot {
                if let Some(expr) = Self::parse_dot_path_to_expr(cond) {
                    if let Some(v) = self.resolve_expr_to_value(&expr, bindings) {
                        return v.as_bool();
                    }
                }
                // expr 解析/求值 miss 不再硬 false——回落下方 read_state +
                // computed 既有通道（语义只增不减）。
            }
            // PLAN-055: 裸 computed 真值判定兜底——`if .hasTime` / `if .isUser`
            // 一类视图级条件引用的是 computed（widget computed{} 表），不是
            // state 字段。此前只查 read_state，失败即 false——musk 用户消息
            // 恒走 chat_message 的 else 臂（AI 排版：hairline+左对齐，气泡
            // 分支永不渲染）、时间标签（hasTime）恒隐藏。对齐无点臂的
            // PLAN-048 L1 同款 computed 兜底。
            match self.read_state(path) {
                Ok(v) => return v.as_bool(),
                Err(_) => {
                    // 与无点臂 PLAN-048 L1 同口径：剥 store. 前缀后查 computed 表。
                    let name = path.strip_prefix("store.").unwrap_or(path);
                    let name = name.rsplit('.').next().unwrap_or(name);
                    return self
                        .eval_computed(name, bindings)
                        .map(|v| v.as_bool())
                        .unwrap_or(false);
                }
            }
        } else {
            // Try binding path truthy check
            if let Some(v) = self.resolve_binding_path(cond, bindings) {
                return v.as_bool();
            }
            // PLAN-055: 字面量布尔条件——`if true {}` / `if false {}` 此前落
            // eval_computed("true") → None → 恒 false（探针 P055-3c 实锤）。
            if cond == "true" {
                return true;
            }
            if cond == "false" {
                return false;
            }
            // PLAN-048 L1: bare `store.X` computed truthy check — same
            // computed-table fallback as the operator path above.
            let name = cond.strip_prefix("store.").unwrap_or(cond);
            if let Some(v) = self.eval_computed(name, bindings) {
                return v.as_bool();
            }
            // PLAN-749 G1: fn 调用真值位兜底（`if extractRunId(.tc)` 形——
            // musk chat_message.at:339 坑①家族第三型：同 fn 在 computed/
            // text 位工作、条件位此前静默返空）。
            return self
                .resolve_condition_operand_fallback(cond, bindings)
                .map(|v| v.as_bool())
                .unwrap_or(false);
        };

        // Read state value for lhs
        // Normalize spaces inside .len() so "notes.len ( )" matches ".len()" suffix.
        // The parser may produce "len ( )" with spaces inside the parens.
        let lhs_normalized = lhs
            .replace(" ( ", "(")
            .replace("( ", "(")
            .replace(" )", ")");
        // PLAN-053 P-053-2: nil 态追踪 —— Nil 的显示串是 ""，与 null 家族
        // 字面量("None"/"null"/"nil")字符串比较恒不等(musk gate 卡常显
        // 根因)。比较阶段按 null 语义处理，见下方 compare 段。
        let mut lhs_nil = false;
        let lhs_val = if let Some(field_name) = lhs_normalized.strip_suffix(".len()") {
            // Strip leading dot from state ref (e.g., ".todos" → "todos")
            // PLAN-642 T-15: store 限定形态（`.NotesStore.notes.len()`，发射器
            // store 接收者限定产物）此前把 "NotesStore.notes" 整段当单字段名
            // read_state 必 miss → 恒 false（015 内嵌 "No notes yet" 实锤）。
            // 经 store_source_field 展平为裸字段（`.store.X`/真名限定同口径，
            // 含 bridge 存档查表）。
            let mut field_name = field_name.trim_start_matches('.').to_string();
            if let Some(bare) = self.store_source_field(&field_name) {
                field_name = bare;
            }
            match self.read_state(&field_name) {
                Ok(Value::Array(arr)) => arr.len().to_string(),
                Ok(other) => {
                    // Also try read_state_as_vec for Value::Int(array_id) refs
                    match self.read_state_as_vec(&field_name) {
                        Ok(vec) => vec.len().to_string(),
                        Err(_) => value_to_display_string(&other),
                    }
                }
                Err(_) => return false,
            }
        } else if let Some(val) = self.resolve_binding_path(lhs, bindings) {
            // Binding path (e.g., "todo.done")
            lhs_nil = matches!(val, Value::Nil | Value::Null);
            value_to_display_string(&val)
        } else if lhs.starts_with('.') {
            // State ref (e.g., ".filter") or nested prop path (e.g., ".block.output)
            let name = &lhs[1..];
            if name.contains('.') {
                // EDGE-16: 嵌套路径(.block.output)用 Dot expr resolve,
                // read_state 只查单字段名取不到。
                if let Some(expr) = Self::parse_dot_path_to_expr(lhs) {
                    match self.resolve_expr_to_value(&expr, bindings) {
                        Some(v) => {
                            lhs_nil = matches!(v, Value::Nil | Value::Null);
                            value_to_display_string(&v)
                        }
                        None => return false,
                    }
                } else {
                    return false;
                }
            } else {
                match self.read_state(name) {
                    Ok(v) => {
                        lhs_nil = matches!(v, Value::Nil | Value::Null);
                        value_to_display_string(&v)
                    }
                    Err(_) => {
                        // PLAN-749 G2: 单段名 read_state miss 回落 computed
                        // （truthy 臂 PLAN-048 L1 同口径——比较臂此前漏配，
                        // `if .c_run != ""` 类单段 computed 比较恒假）。
                        let cname = name.strip_prefix("store.").unwrap_or(name);
                        match self.eval_computed(cname, bindings) {
                            Some(v) => {
                                lhs_nil = matches!(v, Value::Nil | Value::Null);
                                value_to_display_string(&v)
                            }
                            None => {
                                // PLAN-749 G1: CALL/方法链比较位兜底
                                // （`if extractRunId(.tc) != ""` 形）。
                                match self.resolve_condition_operand_fallback(lhs, bindings) {
                                    Some(v) => {
                                        lhs_nil = matches!(v, Value::Nil | Value::Null);
                                        value_to_display_string(&v)
                                    }
                                    None => return false,
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // PLAN-048 L1 (musk auth gate): bare `store.X` / `StoreName.X` /
            // plain computed refs — read_state strips the "store." prefix but
            // only hits materialized state fields; store COMPUTEDS (e.g.
            // AuthStore's `authenticated => .token != None`) live only in the
            // computed table, so fall back to eval_computed (last path
            // segment) before giving up.
            let name = lhs.strip_prefix("store.").unwrap_or(lhs);
            let name = name.rsplit('.').next().unwrap_or(name);
            match self.read_state(name) {
                Ok(v) => {
                    lhs_nil = matches!(v, Value::Nil | Value::Null);
                    value_to_display_string(&v)
                }
                Err(_) => match self.eval_computed(name, bindings) {
                    Some(v) => {
                        lhs_nil = matches!(v, Value::Nil | Value::Null);
                        value_to_display_string(&v)
                    }
                    None => {
                        // PLAN-749 G1: fn 调用比较位兜底——裸名臂的实参段
                        // （`extractRunId(.tc)` 类 CALL 不以 '.' 开头，此前落
                        // 本臂 read_state/eval_computed 全 miss → 恒 false，
                        // musk chat_message.at:339 坑①家族第三型）。
                        match self.resolve_condition_operand_fallback(lhs, bindings) {
                            Some(v) => {
                                lhs_nil = matches!(v, Value::Nil | Value::Null);
                                value_to_display_string(&v)
                            }
                            None => return false,
                        }
                    }
                },
            }
        };

        // Resolve rhs: check loop bindings first, then try as literal
        let mut rhs_nil = false;
        let rhs_val = if let Some(val) = bindings.get(rhs) {
            rhs_nil = matches!(val, Value::Nil | Value::Null);
            value_to_display_string(val)
        } else if let Some(val) = self.resolve_binding_path(rhs, bindings) {
            rhs_nil = matches!(val, Value::Nil | Value::Null);
            value_to_display_string(&val)
        } else if rhs.starts_with('.') {
            // Plan 420: RHS 的 state 引用解析 —— 与 LHS 对称(此前 RHS 从不
            // 查 state,`i == .n` / `t.key == .active` 退化成与字面量
            // ".n"/".active" 比较,for 循环体内一切对 state 的等值判断恒假)。
            let name = &rhs[1..];
            if name.contains('.') {
                if let Some(expr) = Self::parse_dot_path_to_expr(rhs) {
                    match self.resolve_expr_to_value(&expr, bindings) {
                        Some(v) => {
                            rhs_nil = matches!(v, Value::Nil | Value::Null);
                            value_to_display_string(&v)
                        }
                        None => return false,
                    }
                } else {
                    return false;
                }
            } else {
                match self.read_state(name) {
                    Ok(v) => {
                        rhs_nil = matches!(v, Value::Nil | Value::Null);
                        value_to_display_string(&v)
                    }
                    Err(_) => {
                        // PLAN-749 G2/G1：RHS 单段名与 LHS 同口径（computed
                        // 表 + CALL 兜底——此前 RHS 单段 miss 直接 false）。
                        let cname = name.strip_prefix("store.").unwrap_or(name);
                        match self.eval_computed(cname, bindings) {
                            Some(v) => {
                                rhs_nil = matches!(v, Value::Nil | Value::Null);
                                value_to_display_string(&v)
                            }
                            None => match self.resolve_condition_operand_fallback(rhs, bindings) {
                                Some(v) => {
                                    rhs_nil = matches!(v, Value::Nil | Value::Null);
                                    value_to_display_string(&v)
                                }
                                None => return false,
                            },
                        }
                    }
                }
            }
        } else {
            rhs.trim_matches('"').to_string()
        };

        // Compare
        // PLAN-053 P-053-2: null 家族语义 —— 裸字面量 None/null/nil(无引号,
        // 引号串会被上面 trim 成带引号前的原文而在此不匹配)与 Nil 态等值:
        // Nil 显示串 "" vs 字面量 "None" 的纯字符串比较恒不等,musk
        // `.store.current_gate != None` 守卫因此拦不住 nil 态(gate 卡常显)。
        // 两侧均为 null 家族(字面量或 Nil 态)时相等;仅一侧为 null 字面量
        // 时与任何非空值不等;其余维持显示串比较。
        let lhs_is_null_lit = matches!(lhs.trim(), "None" | "null" | "nil" | "Nil");
        let rhs_is_null_lit = matches!(rhs.trim(), "None" | "null" | "nil" | "Nil");
        let lhs_nullish = lhs_is_null_lit || lhs_nil;
        let rhs_nullish = rhs_is_null_lit || rhs_nil;
        match op {
            "==" => {
                if lhs_nullish || rhs_nullish {
                    lhs_nullish && rhs_nullish
                } else {
                    lhs_val == rhs_val
                }
            }
            "!=" => {
                if lhs_nullish || rhs_nullish {
                    !(lhs_nullish && rhs_nullish)
                } else {
                    lhs_val != rhs_val
                }
            }
            ">" | "<" | ">=" | "<=" => {
                let lhs_num: f64 = match lhs_val.parse() {
                    Ok(n) => n,
                    Err(_) => return false,
                };
                let rhs_num: f64 = match rhs_val.parse() {
                    Ok(n) => n,
                    Err(_) => return false,
                };
                match op {
                    ">" => lhs_num > rhs_num,
                    "<" => lhs_num < rhs_num,
                    ">=" => lhs_num >= rhs_num,
                    "<=" => lhs_num <= rhs_num,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// PLAN-749 G1/G2：条件操作数既有通道（binding/state/computed/len）全
    /// miss 后的通用兜底——整段按表达式 fragment 解析（CALL/方法链/索引/
    /// 混合形态）后经 resolve_expr_to_value 统一求值（其 Call 臂含
    /// bridge.call_vm_fn 与 t()/style recipe 通道、Dot 臂含 store 特判）。
    /// 仅兜底不前置：既有绿臂语义零扰动；解析失败回 None（调用方落原
    /// false 语义）。
    pub(super) fn resolve_condition_operand_fallback(&self, src: &str, bindings: &Bindings) -> Option<Value> {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return None;
        }
        // 操作数不该再含比较/逻辑操作符（外层已拆分）——防御性跳过，
        // 避免把复合条件当单表达式误解析。
        if [" == ", " != ", " && ", " || ", " > ", " < ", " >= ", " <= "]
            .iter()
            .any(|op| trimmed.contains(op))
        {
            return None;
        }
        let expr = auto_lang_core::parser::Parser::parse_expr_fragment(trimmed)?;
        self.resolve_expr_to_value(&expr, bindings)
    }

    /// Resolve `${.field}` interpolation patterns in a literal string.
    ///
    /// F-strings like `f"Count: ${.count}"` are extracted as `Expr::Str`
    /// with the template preserved. This method scans for `${.name}` patterns
    /// and substitutes current state values.
    /// Resolve `${.field}` interpolation patterns with loop bindings support.
    ///
    /// Plan 503: also resolves loop-member form `${member.field}` (e.g.
    /// launcher brand-color chip `style: "h-10 w-10 ${r.chip}"` inside
    /// `for r in .rows`) via resolve_expr_to_value over the loop bindings.
    pub(super) fn resolve_literal_interpolation_with(&self, s: &str, bindings: &Bindings) -> String {
        if !s.contains("${") {
            return s.to_string();
        }

        let mut result = s.to_string();
        // Scan for ${.fieldname} patterns and resolve from state
        let bytes = s.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        let mut replacements: Vec<(String, String)> = Vec::new();

        while i + 2 < len {
            if i + 3 < len && &bytes[i..i + 3] == b"${." {
                // Found start of interpolation: ${.
                let start = i;
                let mut end = i + 3;
                while end < len && bytes[end] != b'}' {
                    end += 1;
                }
                if end < len && bytes[end] == b'}' {
                    let field_name = &s[start + 3..end];
                    // PLAN-626 T-01: allow multi-segment dotted paths
                    // (`${.store.line}`). Previously single-segment only — a
                    // dotted name failed this validation and the raw
                    // `${.store.line}` stayed visible (auto-edit statusbar).
                    let seg_ok = |seg: &str| {
                        !seg.is_empty() && seg.chars().all(|c| c.is_alphanumeric() || c == '_')
                    };
                    let segments: Vec<&str> = field_name.split('.').collect();
                    if !field_name.is_empty() && segments.iter().all(|seg| seg_ok(seg)) {
                        let resolved = if segments.len() == 1 {
                            Some(self.read_state_as_string_with(field_name, bindings))
                        } else {
                            // Multi-segment: rebuild the AST chain (rooted at
                            // "." like the parser's dot_item) and ride the
                            // same resolver conditions use — `.store.X`
                            // flattens to a root-state read there (D-GAP-4).
                            let mut expr = Expr::Ident(".".to_string().into());
                            for seg in &segments {
                                expr = Expr::Dot(Box::new(expr), seg.to_string().into());
                            }
                            self.resolve_expr_to_value(&expr, bindings)
                                .map(|v| value_to_display_string(&v))
                        };
                        if let Some(value) = resolved {
                            let full_pattern = s[start..end + 1].to_string();
                            replacements.push((full_pattern, value));
                        }
                        // Unresolvable multi-segment path: leave the raw
                        // `${.a.b}` in place (dots intact) instead of
                        // splicing a stripped placeholder.
                    }
                }
                i = end + 1;
            } else if i + 2 < len && &bytes[i..i + 2] == b"${" {
                // Plan 503: loop-member form ${member.field} — single dot
                // path into the loop-bound Obj row.
                let start = i;
                let mut end = i + 2;
                while end < len && bytes[end] != b'}' {
                    end += 1;
                }
                if end < len && bytes[end] == b'}' {
                    let inner = &s[start + 2..end];
                    if let Some((root, field)) = inner.split_once('.') {
                        if !root.is_empty()
                            && !field.is_empty()
                            && root.chars().all(|c| c.is_alphanumeric() || c == '_')
                            && field.chars().all(|c| c.is_alphanumeric() || c == '_')
                        {
                            let expr = Expr::Dot(
                                Box::new(Expr::Ident(root.to_string().into())),
                                field.to_string().into(),
                            );
                            if let Some(v) = self.resolve_expr_to_value(&expr, bindings) {
                                let full_pattern = s[start..end + 1].to_string();
                                replacements.push((full_pattern, value_to_display_string(&v)));
                            }
                        }
                    }
                }
                i = end + 1;
            } else {
                i += 1;
            }
        }

        for (pattern, value) in replacements {
            result = result.replace(&pattern, &value);
        }
        result
    }

    // ========================================================================
    // Event helpers
    // ========================================================================

}
