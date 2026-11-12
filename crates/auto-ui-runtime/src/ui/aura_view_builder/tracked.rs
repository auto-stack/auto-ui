//! PLAN-014 T-05：tracked 管线责任（convert_node_with 分派、*_tracked_ctx 家族、tracked twins）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {
    /// Dispatch an AuraNode to the appropriate converter with loop variable bindings.
    pub(super) fn convert_node_with(&self, node: &AuraNode, bindings: &Bindings) -> View<DynamicMessage> {
        match node {
            AuraNode::Element {
                tag,
                props,
                events,
                children,
                ..
            } => self.convert_element(tag, props, events, children, bindings),
            AuraNode::Text(text_content) => self.convert_text_with(text_content, bindings),
            AuraNode::MemoBlock {
                deps, exact, body, ..
            } => {
                // PLAN-046 T-06 执行期修正：untracked 帧同走共享块门（伪造
                // 空簿记通道；probe 禁用 → probe_on=false 键面）。
                let mut path0 = Vec::new();
                let mut id_map0 = DebugIdMap::default();
                let mut probe0 = BuildProbe::new_disabled();
                self.convert_memo_block(
                    deps,
                    *exact,
                    body,
                    &mut path0,
                    &mut id_map0,
                    &mut probe0,
                    bindings,
                )
            }
            AuraNode::ForLoop {
                var,
                index,
                iterable,
                key_expr,
                body,
                ..
            } => {
                // PLAN-046 T-06 执行期修正：桌面窗口渲染帧走本 untracked 轨
                //（DynamicComponent::view()=build()），keyed 门必须双轨可达
                // ——伪造空簿记通道复用同一 convert_for_keyed（机制单源）；
                // probe 禁用 → probe_on=false 键面，与 tracked 帧条目互不串。
                if let Some(key_expr) = key_expr {
                    let mut path0 = Vec::new();
                    let mut id_map0 = DebugIdMap::default();
                    let mut probe0 = BuildProbe::new_disabled();
                    return self.convert_for_keyed(
                        var,
                        index,
                        iterable,
                        key_expr,
                        body,
                        &mut path0,
                        &mut id_map0,
                        &mut probe0,
                        bindings,
                    );
                }
                // Strip leading dot from iterable name (e.g., ".notes" → "notes")
                let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
                // Plan 370 (Issue 2): for dotted prop paths like `.note.tags`,
                // resolve via resolve_iterable (handles field-by-field deref).
                // For simple state fields, use read_state/read_state_as_vec.
                let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
                let store_field = self.store_source_field(stripped);
                let has_inner_dot = stripped.contains('.') && store_field.is_none();
                let stripped = store_field.as_deref().unwrap_or(stripped);
                let state_name = store_field.as_deref().unwrap_or(state_name);
                // elems → loop children(堆引用与 computed 回退两路共用;语义
                // 与下方 Ok(Value::Array) 主路径一致:matches_search 过滤 +
                // 循环变量/index 绑定 + 空/单/多子聚合)。
                let render_for_children = |elems: Vec<Value>| -> View<DynamicMessage> {
                    let children: Vec<View<DynamicMessage>> = elems
                        .iter()
                        .enumerate()
                        .filter_map(|(i, item)| {
                            // Apply search filter if 'search' state exists and is non-empty
                            if !self.matches_search(item) {
                                return None;
                            }
                            let mut loop_bindings = bindings.clone();
                            loop_bindings
                                .insert(var.clone(), self.bridge.materialize_obj_ref(item));
                            if let Some(idx_var) = index {
                                loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                            }
                            let views: Vec<View<DynamicMessage>> = body
                                .iter()
                                .map(|n| self.convert_node_with(n, &loop_bindings))
                                .collect();
                            if views.is_empty() {
                                None
                            } else if views.len() == 1 {
                                Some(views.into_iter().next().unwrap())
                            } else {
                                Some(View::Column {
                                    children: views,
                                    spacing: 0,
                                    padding: 0,
                                    style: None,
                                    onclick: None,
                                    on_right_click: None,
                                })
                            }
                        })
                        .collect();
                    View::Column {
                        children,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    }
                };
                let array: auto_val::Array = if has_inner_dot {
                    match self.resolve_iterable(iterable, bindings) {
                        Some(elems) => auto_val::Array::from(elems),
                        None => return View::Empty,
                    }
                } else {
                    // Plan 046:裸标识符 iterable 可能是外层循环绑定的变量 ——
                    // 先查 bindings(同 tracked 路径修复)。命中则解包成 Array。
                    if let Some(val) = bindings.get(state_name).cloned() {
                        match val {
                            Value::Array(arr) => arr,
                            Value::Int(id) if id >= 4_000_000 => {
                                auto_val::Array::from(self.bridge.index_list_all(id as usize))
                            }
                            Value::VmRef(r) => {
                                auto_val::Array::from(self.bridge.index_list_all(r.id))
                            }
                            _ => return View::Empty,
                        }
                    } else {
                        // Read the iterable array from VmBridge state
                        match self.read_state(state_name) {
                            Ok(Value::Array(arr)) => arr,
                            Ok(_other) => {
                                // Try read_state_as_vec for Value::Int(array_id) refs
                                match self.read_state_as_vec(state_name) {
                                    Ok(vec) => {
                                        return render_for_children(vec);
                                    }
                                    Err(_) => return View::Empty,
                                }
                            }
                            Err(_) => {
                                // PLAN-051 C3 对齐(本臂此前漏齐):state 读 miss →
                                // computed 求值回退。tracked for 路径(下方 ~963)早有
                                // 该回退,本 convert 路径没有 —— `for cell in .days`
                                // 以 computed 为 for 源(Plan 522 016 迁移)在此渲染整空。
                                match self
                                    .eval_computed(state_name, bindings)
                                    .and_then(|v| self.value_to_iter_vec(&v))
                                {
                                    Some(vec) => return render_for_children(vec),
                                    None => return View::Empty,
                                }
                            }
                        }
                    }
                };

                let children: Vec<View<DynamicMessage>> = array
                    .iter()
                    .enumerate()
                    .filter_map(|(i, item)| {
                        // Apply search filter if 'search' state exists and is non-empty
                        if !self.matches_search(item) {
                            return None;
                        }
                        let mut loop_bindings = bindings.clone();
                        // Bind loop variable (e.g., "note" → Value::Obj{title, body, time})
                        loop_bindings.insert(var.clone(), self.bridge.materialize_obj_ref(item));
                        // Bind index variable if present (e.g., "i" → Value::Int(0))
                        if let Some(idx_var) = index {
                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                        }
                        // Convert body nodes with the loop bindings active
                        let views: Vec<View<DynamicMessage>> = body
                            .iter()
                            .map(|n| self.convert_node_with(n, &loop_bindings))
                            .collect();
                        if views.is_empty() {
                            None
                        } else if views.len() == 1 {
                            // Plan 370 (Issue 1): if the single body view is
                            // Empty (e.g. a false `if` inside the loop), skip
                            // it so the loop doesn't emit a text("") spacer.
                            let v = views.into_iter().next().unwrap();
                            if matches!(v, View::Empty) {
                                None
                            } else {
                                Some(v)
                            }
                        } else {
                            Some(View::Column {
                                children: views,
                                spacing: 0,
                                padding: 0,
                                style: None,
                                onclick: None,
                                on_right_click: None,
                            })
                        }
                    })
                    .collect();

                View::Column {
                    children,
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                    on_right_click: None,
                }
            }
            AuraNode::Conditional {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let is_true = self.eval_condition_with(condition, bindings);
                let empty = Vec::new();
                let body = if is_true {
                    then_body
                } else {
                    else_body.as_ref().unwrap_or(&empty)
                };
                let children: Vec<View<DynamicMessage>> = body
                    .iter()
                    .map(|n| self.convert_node_with(n, bindings))
                    .collect();
                if children.is_empty() {
                    View::Empty
                } else if children.len() == 1 {
                    children.into_iter().next().unwrap()
                } else {
                    View::Column {
                        children,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    }
                }
            }
            AuraNode::Component {
                name,
                props,
                events,
                children,
                ..
            } => {
                // Plan 408: nav-link renders as a navigable button (like link).
                if name == "nav-link" || name == "nav_link" {
                    let prop_map: HashMap<String, AuraPropValue> = props
                        .iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    let to = self
                        .extract_string(&prop_map, "to")
                        .or_else(|| self.extract_string(&prop_map, "href"))
                        .unwrap_or_default();
                    let label = self
                        .extract_string(&prop_map, "label")
                        .or_else(|| self.extract_string(&prop_map, "text"))
                        .unwrap_or_default();
                    let icon = self.extract_string(&prop_map, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(
                        &label,
                        &[],
                        &to,
                        &icon,
                        bindings,
                        false,
                    );
                }
                // Plan 482: nav-item / nav-group 组件形态分发（Element 形态为主，
                // 此处镜像 nav-link 的防御性覆盖）。
                if name == "nav-item" || name == "nav_item" {
                    let prop_map: HashMap<String, AuraPropValue> = props
                        .iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    return self.convert_nav_item(&prop_map, events, children, bindings);
                }
                if name == "nav-group" || name == "nav_group" {
                    let prop_map: HashMap<String, AuraPropValue> = props
                        .iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    return self.convert_nav_group(&prop_map, events, children, bindings);
                }
                // Plan 410: category-section → column (recurse component-card
                // children). Vue codegen builds a fancy card grid; VM renders a
                // simple column so the home page's component list isn't blank.
                if name == "category-section" || name == "category_section" {
                    let child_views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .filter_map(|n| {
                            let v = self.convert_node_with(n, bindings);
                            if matches!(v, View::Empty) {
                                None
                            } else {
                                Some(v)
                            }
                        })
                        .collect();
                    return View::Column {
                        children: child_views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    };
                }
                // Plan 410: component-card → navigable link button (to + name + desc).
                if name == "component-card" || name == "component_card" || name == "componentcard" {
                    let prop_map: HashMap<String, AuraPropValue> = props
                        .iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    let to = self.extract_string(&prop_map, "to").unwrap_or_default();
                    let card_name = self.extract_string(&prop_map, "name").unwrap_or_default();
                    let desc = self.extract_string(&prop_map, "desc").unwrap_or_default();
                    let label = if desc.is_empty() {
                        card_name
                    } else {
                        format!("{} — {}", card_name, desc)
                    };
                    let icon = self.extract_string(&prop_map, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(
                        &label,
                        &[],
                        &to,
                        &icon,
                        bindings,
                        false,
                    );
                }
                // Look up child widget in registry
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(name) {
                        let prop_values: HashMap<String, AuraPropValue> = props
                            .iter()
                            .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                            .collect();
                        // Plan 476: 调用位 children 里的 slot 填充传给子构建器。
                        let fills = self.slot_fills_for(children, bindings);
                        return self.render_child_widget(
                            child_widget,
                            &prop_values,
                            events,
                            bindings,
                            fills.as_ref(),
                        );
                    }
                }
                // PLAN-095 T-07 (G-11): 未解析子件引用给可定位诊断——此前
                // 静默降级 `<Name />` 文本占位（093 §8.16「子件整体空渲染」
                // 的构成面之一），消费方无从定位。
                eprintln!(
                    "[AURA-CHILD-MISS] component '{}' not resolvable at this site — declare it in the same file (sibling widget) or import via `use <module>: {}`",
                    name, name
                );
                View::Text {
                    content: format!("<{} />", name),
                    style: None,
                    selectable: false,
                }
            }
            AuraNode::Outlet { memo } => {
                // Plan 401/VM-routing: render the page widget matching the
                // current route (the iced equivalent of vue's <router-view>).
                self.render_outlet_impl(bindings, None, *memo)
            }
            AuraNode::Link {
                text, children, to, ..
            } => {
                // Plan 401/VM-routing: render a link as a clickable button whose
                // onclick carries the target path as a __navigate message. The
                // update loop intercepts __navigate and sets __current_route.
                self.render_link_button(text, children, to, bindings)
            }
        }
    }

    /// Tracked node conversion: deep recursion that records per-path data into
    /// both `DebugIdMap` (AuraNodeId) and `BuildProbe` (state bindings). This is
    /// the Plan 307 Task 9 deep-threaded path; the untracked `build()` path
    /// never reaches here, so its behaviour is unchanged.
    /// PLAN-046 T-03：keyless for 的原始求值体——tracked 1107 臂逐字节搬移
    /// （raw 面零重排；key_expr 缺省与 keyed 降级回退共用此单源实现）。
    pub(super) fn convert_for_unkeyed(
        &self,
        var: &str,
        index: &Option<String>,
        iterable: &str,
        body: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
        let array: Vec<Value> = match self.resolve_for_iterable(iterable, state_name, bindings) {
            Some(v) => v,
            None => return View::Empty,
        };
        let child_views: Vec<View<DynamicMessage>> = array
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                // Apply search filter if 'search' state exists and is non-empty
                if !self.matches_search(item) {
                    return None;
                }
                let mut loop_bindings = bindings.clone();
                loop_bindings.insert(var.to_string(), self.bridge.materialize_obj_ref(item));
                if let Some(idx_var) = index {
                    loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                }
                self.render_for_item_view(
                    i,
                    item,
                    var,
                    index,
                    iterable,
                    body,
                    path,
                    id_map,
                    probe,
                    &loop_bindings,
                )
            })
            .collect();
        View::Column {
            children: child_views,
            spacing: 0,
            padding: 0,
            style: None,
            onclick: None,
            on_right_click: None,
        }
    }

    /// PLAN-046 T-03：for iterable 三通道解析（自 tracked 1107 臂逐字节搬移）：
    /// 内点链 → `resolve_iterable`（Plan 370 逐字段解引用）；裸标识符 →
    /// bindings 查表（`row` 等外层循环变量，Plan 046·auto-lang 注记）→
    /// `read_state_as_vec` → computed 回退（PLAN-051 C3）。`None` = 解析
    /// 失败（调用方落 `View::Empty`，与原臂 return 语义一致）。
    pub(super) fn resolve_for_iterable(
        &self,
        iterable: &str,
        state_name: &str,
        bindings: &Bindings,
    ) -> Option<Vec<Value>> {
        // Plan 370 (Issue 2): for dotted prop paths like `.note.tags`,
        // resolve via resolve_iterable (handles field-by-field deref).
        let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
        let has_inner_dot = stripped.contains('.') && !stripped.starts_with("store.");
        if has_inner_dot {
            return self.resolve_iterable(iterable, bindings);
        }
        // Plan 046:裸标识符 iterable(如内层 for 的 `row`)可能是外层循环
        // 绑定的变量 —— 先查 bindings,命中则解包成 Vec<Value>(同 resolve_iterable
        // :253-263 的 match 逻辑)。未命中再 fallback 到 read_state_as_vec。
        if let Some(val) = bindings.get(state_name).cloned() {
            return match val {
                auto_val::Value::Array(arr) => Some(arr.iter().cloned().collect()),
                auto_val::Value::Int(id) if id >= 4_000_000 => {
                    Some(self.bridge.index_list_all(id as usize))
                }
                auto_val::Value::VmRef(r) => Some(self.bridge.index_list_all(r.id)),
                _ => {
                    log::warn!(
                        "view_builder: bindings['{}'] is not iterable: {:?}",
                        state_name,
                        val
                    );
                    None
                }
            };
        }
        // Read the iterable. `read_state_as_vec` handles BOTH an inline
        // `Value::Array` and a `Value::Int(array_id)` heap-array reference
        // (the latter is how `var x = []; x.push(...)` arrays are stored —
        // e.g. 016-calendar's `.days`). A bare `read_state` + `Value::Array`
        // match misses the heap-id form and silently renders an empty loop.
        match self.read_state_as_vec(state_name) {
            Ok(v) => Some(v),
            Err(e) => {
                // PLAN-051 C3: state 读 miss → computed 求值回退
                //（musk filteredMessages 链式 fn 调用形态的 for 源）。
                let dbg_vec = self
                    .eval_computed(state_name, bindings)
                    .and_then(|v| self.value_to_iter_vec(&v));
                if std::env::var("AUTO_DEBUG_EMIT").is_ok() {
                    eprintln!(
                        "[VM-FOR] {} computed fallback rows={:?}",
                        state_name,
                        dbg_vec.as_ref().map(|v| v.len())
                    );
                }
                match dbg_vec {
                    Some(v) => Some(v),
                    None => {
                        log::warn!(
                            "view_builder: read_state_as_vec('{}') failed: {}",
                            state_name,
                            e
                        );
                        None
                    }
                }
            }
        }
    }

    /// PLAN-046 T-03：单项渲染（自 tracked 1107 臂逐字节搬移的 per-item 体）。
    /// 返回 `None` = 搜索过滤剔除 / 体视图全空（与原臂 filter_map 语义一致）。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_for_item_view(
        &self,
        i: usize,
        item: &Value,
        var: &str,
        index: &Option<String>,
        iterable: &str,
        body: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        loop_bindings: &Bindings,
    ) -> Option<View<DynamicMessage>> {
        // Include iteration index in path to ensure unique debug IDs
        // across loop iterations (without this, all iterations produce
        // identical paths, causing duplicate iced widget IDs).
        //
        // Plan 309 Phase 1 (Fix A): only push the body-node index
        // when the body has >1 node. When the body is a single
        // node, the iteration yields that node *directly* (no
        // wrapping Column — see the `views.len() == 1` unwrap
        // below), so the node's flattened VTree path is `[p, i]`.
        // Unconditionally pushing `bi` (=0) recorded it at
        // `[p, i, 0]`, diverging from the VTree path and leaving
        // the inspector's AutoUI / source data empty for loop
        // bodies. The multi-node case still wraps each iteration
        // in a Column, so `bi` must be pushed there to match the
        // extra VTree level. `record_for` is computed after the
        // push, so it auto-reflects the corrected depth.
        let body_len = body.len();
        let views: Vec<View<DynamicMessage>> = body
            .iter()
            .enumerate()
            .filter_map(|(bi, n)| {
                path.push(i); // iteration index
                if body_len > 1 {
                    path.push(bi);
                } // body node index (multi-node only)
                  // Record this iteration's context against the
                  // body node's path (Plan 307 Task 10). `index`
                  // is the 0-based iteration counter `i`, NOT the
                  // loop's optional index-variable name. Keep
                  // `iterable_repr` in its original ".notes" form.
                let for_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
                probe.record_for(
                    &for_path,
                    ForIter {
                        var: var.to_string(),
                        index: Some(i),
                        value_repr: value_to_display_string(item),
                        iterable_repr: iterable.to_string(),
                    },
                );
                let v = self.convert_node_tracked_ctx(n, path, id_map, probe, loop_bindings);
                if body_len > 1 {
                    path.pop();
                }
                path.pop();
                Some(v)
            })
            .collect();
        if views.is_empty() {
            None
        } else if views.len() == 1 {
            // Plan 370 (Issue 1): skip Empty body views (see convert_node_with ForLoop).
            let v = views.into_iter().next().unwrap();
            if matches!(v, View::Empty) {
                None
            } else {
                Some(v)
            }
        } else {
            Some(View::Column {
                children: views,
                spacing: 0,
                padding: 0,
                style: None,
                onclick: None,
                on_right_click: None,
            })
        }
    }

    pub(super) fn convert_node_tracked_ctx(
        &self,
        node: &AuraNode,
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // Record this node's debug_id at the current path
        let node_debug_id = match node {
            AuraNode::Element { debug_id, .. } => *debug_id,
            AuraNode::ForLoop { debug_id, .. } => *debug_id,
            AuraNode::Conditional { debug_id, .. } => *debug_id,
            AuraNode::Component { debug_id, .. } => *debug_id,
            AuraNode::Link { debug_id, .. } => *debug_id,
            AuraNode::MemoBlock { debug_id, .. } => *debug_id,
            _ => None,
        };
        if let Some(aura_id) = node_debug_id {
            id_map.record(path, aura_id);
        }

        match node {
            AuraNode::Element {
                tag,
                props,
                events,
                children,
                ..
            } => self.convert_element_tracked_ctx(
                tag, props, events, children, path, id_map, probe, bindings,
            ),
            AuraNode::Text(text_content) => {
                self.convert_text_tracked_ctx(text_content, path, probe, bindings)
            }
            AuraNode::ForLoop {
                var,
                index,
                iterable,
                key_expr,
                body,
                ..
            } => {
                // PLAN-046 T-03：keyed-for（`key:` 子句）走项级缓存门；
                // keyless 循环 = 原始体直落（convert_for_unkeyed 逐字节
                // 搬移自本臂，缺省路径零行为差——AC-01 前提）。
                if let Some(key_expr) = key_expr {
                    return self.convert_for_keyed(
                        var, index, iterable, key_expr, body, path, id_map, probe, bindings,
                    );
                }
                self.convert_for_unkeyed(var, index, iterable, body, path, id_map, probe, bindings)
            }
            AuraNode::MemoBlock {
                deps, exact, body, ..
            } => {
                // PLAN-046 T-04：显式 memo 块门（deps 值指纹 ∪ 自动可证读槽；
                // exact = 纯 deps 信任模式）。
                self.convert_memo_block(deps, *exact, body, path, id_map, probe, bindings)
            }
            AuraNode::Conditional {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let is_true = self.eval_condition_with(condition, bindings);
                let empty = Vec::new();
                let body = if is_true {
                    then_body
                } else {
                    else_body.as_ref().unwrap_or(&empty)
                };
                // Plan 309 Phase 1 (Fix A, companion to the ForLoop fix):
                // mirror the loop's behaviour — only push the child index when
                // there is >1 child, because the single-child `child_views.len()
                // == 1` unwrap below yields that child directly (no wrapping
                // Column), so the node's flattened VTree path drops the index
                // level. Pushing it unconditionally diverged from the VTree path
                // for single-child conditionals, leaving inspector data empty.
                let body_len = body.len();
                let child_views: Vec<View<DynamicMessage>> = body
                    .iter()
                    .enumerate()
                    .map(|(i, n)| {
                        if body_len > 1 {
                            path.push(i);
                        }
                        let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
                        if body_len > 1 {
                            path.pop();
                        }
                        v
                    })
                    .collect();
                if child_views.is_empty() {
                    View::Empty
                } else if child_views.len() == 1 {
                    child_views.into_iter().next().unwrap()
                } else {
                    View::Column {
                        children: child_views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    }
                }
            }
            AuraNode::Component {
                name,
                props,
                events,
                children,
                ..
            } => {
                // Plan 408: nav-link renders as a navigable button (like link).
                if name == "nav-link" || name == "nav_link" {
                    let prop_map: HashMap<String, AuraPropValue> = props
                        .iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    let to = self
                        .extract_string(&prop_map, "to")
                        .or_else(|| self.extract_string(&prop_map, "href"))
                        .unwrap_or_default();
                    let label = self
                        .extract_string(&prop_map, "label")
                        .or_else(|| self.extract_string(&prop_map, "text"))
                        .unwrap_or_default();
                    let icon = self.extract_string(&prop_map, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(
                        &label,
                        &[],
                        &to,
                        &icon,
                        bindings,
                        false,
                    );
                }
                // Plan 482: nav-item / nav-group 组件形态分发（Element 形态为主，
                // 此处镜像 nav-link 的防御性覆盖）。
                if name == "nav-item" || name == "nav_item" {
                    let prop_map: HashMap<String, AuraPropValue> = props
                        .iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    return self.convert_nav_item(&prop_map, events, children, bindings);
                }
                if name == "nav-group" || name == "nav_group" {
                    let prop_map: HashMap<String, AuraPropValue> = props
                        .iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    return self.convert_nav_group(&prop_map, events, children, bindings);
                }
                // Plan 410: category-section → column (recurse component-card
                // children). Vue codegen builds a fancy card grid; VM renders a
                // simple column so the home page's component list isn't blank.
                if name == "category-section" || name == "category_section" {
                    let child_views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .filter_map(|n| {
                            let v = self.convert_node_with(n, bindings);
                            if matches!(v, View::Empty) {
                                None
                            } else {
                                Some(v)
                            }
                        })
                        .collect();
                    return View::Column {
                        children: child_views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    };
                }
                // Plan 410: component-card → navigable link button (to + name + desc).
                if name == "component-card" || name == "component_card" || name == "componentcard" {
                    let prop_map: HashMap<String, AuraPropValue> = props
                        .iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    let to = self.extract_string(&prop_map, "to").unwrap_or_default();
                    let card_name = self.extract_string(&prop_map, "name").unwrap_or_default();
                    let desc = self.extract_string(&prop_map, "desc").unwrap_or_default();
                    let label = if desc.is_empty() {
                        card_name
                    } else {
                        format!("{} — {}", card_name, desc)
                    };
                    let icon = self.extract_string(&prop_map, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(
                        &label,
                        &[],
                        &to,
                        &icon,
                        bindings,
                        false,
                    );
                }
                // Look up child widget in registry
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(name) {
                        let prop_values: HashMap<String, AuraPropValue> = props
                            .iter()
                            .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                            .collect();
                        // D-GAP-4: tracked variant so the child subtree's
                        // style/event bindings reach the BuildProbe (snapshot).
                        // Plan 476: slot 填充透传(tracked 双胎)。
                        let fills = self.slot_fills_for(children, bindings);
                        return self.render_child_widget_tracked(
                            child_widget,
                            &prop_values,
                            events,
                            bindings,
                            path,
                            id_map,
                            probe,
                            fills.as_ref(),
                        );
                    }
                }
                // PLAN-093 G-16：tracked 臂 miss 诊断——此前的静默文本占位
                // 使导入子件模板的组件引用零痕迹空渲染（095 根文件臂有
                // AURA-CHILD-MISS，本臂没有——排障时两臂都必须可观测）。
                eprintln!(
                    "[AURA-CHILD-MISS][tracked] component '{}' not resolvable at this site — declare it in the same file (sibling widget) or import via `use <module>: {}`",
                    name, name
                );
                View::Text {
                    content: format!("<{} />", name),
                    style: None,
                    selectable: false,
                }
            }
            AuraNode::Outlet { memo } => {
                // Plan 401/VM-routing: render the page widget matching the
                // current route (the iced equivalent of vue's <router-view>).
                self.render_outlet_impl(bindings, Some((path, id_map, probe)), *memo)
            }
            AuraNode::Link {
                text, children, to, ..
            } => {
                // Plan 401/VM-routing: render link as a clickable button (same
                // as the untracked path); tracked children are flattened into
                // the button label.
                let _ = (path, id_map, probe);
                self.render_link_button(text, children, to, bindings)
            }
        }
    }

    /// Tracked convert_element: dispatches by tag and recurses children with
    /// path/probe tracking (deep), instead of delegating to the untracked
    /// converters. Layout/prop extraction mirrors the untracked converters
    /// exactly; only the child recursion differs (it carries the side-channels).
    /// Plan 490 G4：布局件（row/col/taskbar/container|div）onclick 提取——
    /// 事件在分发点可得而 convert_* 不收 events，故分发臂就地 set 进结果
    /// （沿 :2076 text onclick→Button 的事件提取先例；onclick/click 双键
    /// 收取，大小写不敏感由 aura_events_get_base 承担）。非三节点视图
    /// 静默忽略（行为持平）。
    /// PLAN-002 B：同一提取点收拢 `oncontextmenu`（右键）——布局件在 Vue 轨
    /// 早已泛映射 @contextmenu（:14470 元素级事件表），VM 此前只有 button/
    /// mouse-area 挂点（526 Q6/Q9① 债）。
    pub(super) fn set_layout_events(
        &self,
        v: &mut View<DynamicMessage>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) {
        let onclick = auto_ui_compiler::aura::aura_events_get_base(events, "onclick")
            .or_else(|| auto_ui_compiler::aura::aura_events_get_base(events, "click"))
            .map(|event| self.event_to_message_with(event, bindings));
        let on_right_click = auto_ui_compiler::aura::aura_events_get_base(events, "oncontextmenu")
            .map(|event| self.event_to_message_with(event, bindings));
        if onclick.is_none() && on_right_click.is_none() {
            return;
        }
        match v {
            View::Row {
                onclick: oc,
                on_right_click: rc,
                ..
            }
            | View::Column {
                onclick: oc,
                on_right_click: rc,
                ..
            }
            | View::Container {
                onclick: oc,
                on_right_click: rc,
                ..
            } => {
                if let Some(msg) = onclick {
                    *oc = Some(msg);
                }
                if let Some(msg) = on_right_click {
                    *rc = Some(msg);
                }
            }
            _ => {}
        }
    }

    /// PLAN-717: render the current (or selected) month for VM-only Calendar
    /// components. Vue uses the installed calendar widget; this native grid
    /// keeps the VM preview visible and aligned with that widget's month view.
    pub(super) fn convert_calendar(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use chrono::Datelike;

        fn cell(content: String, class: &str) -> View<DynamicMessage> {
            let class = format!("w-9 h-9 {class}");
            View::Container {
                child: Box::new(View::Text {
                    content,
                    style: None,
                    selectable: false,
                }),
                padding: 0,
                width: Some(36),
                height: Some(36),
                center_x: true,
                center_y: true,
                onclick: None,
                on_right_click: None,
                style: Style::parse(&class).ok(),
            }
        }

        let today = chrono::Local::now().date_naive();
        let selected = self
            .extract_string_with(props, "selected", bindings)
            .and_then(|value| chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok());
        let display_date = selected.unwrap_or(today);
        let month = display_date.with_day(1).unwrap_or(display_date);
        let leading_days = month.weekday().num_days_from_sunday() as usize;
        let days_in_month = month
            .checked_add_months(chrono::Months::new(1))
            .and_then(|next_month| next_month.pred_opt())
            .map(|last_day| last_day.day() as usize)
            .unwrap_or(31);

        let mut cells = Vec::with_capacity(49);
        for weekday in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
            cells.push(cell(
                weekday.to_string(),
                "text-xs font-medium text-muted-foreground",
            ));
        }
        for index in 0usize..42 {
            let day_of_month = index
                .checked_sub(leading_days)
                .map(|day| day + 1)
                .filter(|day| *day <= days_in_month);
            let Some(day_of_month) = day_of_month else {
                cells.push(cell(String::new(), ""));
                continue;
            };
            let date = month.with_day(day_of_month as u32).unwrap_or(month);
            let is_selected = selected.map_or(date == today, |selected| date == selected);
            let style = if is_selected {
                "text-sm rounded-md bg-primary text-primary-foreground"
            } else {
                "text-sm rounded-md"
            };
            cells.push(cell(day_of_month.to_string(), style));
        }

        View::Column {
            children: vec![
                View::Text {
                    content: month.format("%B %Y").to_string(),
                    style: Style::parse("text-sm font-medium text-center").ok(),
                    selectable: false,
                },
                View::Grid {
                    cols: 7,
                    gap: 0,
                    cells,
                    style: None,
                },
            ],
            spacing: 8,
            padding: 12,
            style: Style::parse("w-fit rounded-md border bg-background").ok(),
            onclick: None,
            on_right_click: None,
        }
    }

    /// PLAN-641 T-03：tabs 复合组件折叠——`tabs` 根（+ `tab`/`tabs-trigger`/
    /// `tabs-content` 子件与 `tabs-list`/`tabslist`/`tabrow` 透明包装）折叠为
    /// 有状态 `View::Tabs`（此前落 unknown fallback → Column，VM 轨无 tabs）。
    /// 契约（Q2 受控最小集，§9 T-01 定案）：
    /// - labels：trigger 的 `text`/`label` prop 或文本子件；
    /// - contents：`tabs-content` 子件；`tab` 平铺形态（A2UI 折叠产物）取其子件；
    /// - selected 解析序：`active`（索引）→ `value`（索引或按值匹配）→
    ///   `defaultvalue`/`default` → 0；越界钳制到末 tab；
    /// - on_select：根 `onselect` 或首个 trigger `onclick`，事件
    ///   args=[选中 tab 的 value 字符串]；无绑定则点击不切换（受控语义，
    ///   对齐 Vue 端受控 v-model 缺省）；
    /// - variant：`variant` prop → TabsVariant::parse（未知值回退 Default，
    ///   PLAN-641 AC-01 兜底）。
    pub(super) fn convert_tabs(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        fn tag_lc(tag: &str) -> String {
            tag.to_ascii_lowercase().replace('_', "-")
        }
        fn collect<'a>(
            node: &'a AuraNode,
            triggers: &mut Vec<&'a AuraNode>,
            contents: &mut Vec<&'a AuraNode>,
            flats: &mut Vec<&'a AuraNode>,
        ) {
            let AuraNode::Element { tag, children, .. } = node else {
                return;
            };
            match tag_lc(tag).as_str() {
                "tab" => flats.push(node),
                "tabs-trigger" | "tabstrigger" => triggers.push(node),
                "tabs-content" | "tabscontent" => contents.push(node),
                "tabs-list" | "tabslist" | "tabrow" | "tab-row" => {
                    for c in children {
                        collect(c, triggers, contents, flats);
                    }
                }
                // 未知子件忽略（契约面收窄；catalog 文档随 spec 沉淀）。
                _ => {}
            }
        }

        let mut trigger_nodes: Vec<&AuraNode> = Vec::new();
        let mut content_nodes: Vec<&AuraNode> = Vec::new();
        let mut flat_tabs: Vec<&AuraNode> = Vec::new();
        for c in children {
            collect(c, &mut trigger_nodes, &mut content_nodes, &mut flat_tabs);
        }

        // prop 值解析：静态字面量与状态引用（.demo）都穿透到 Value
        // （resolve_expr_to_value 走 bindings/computed/read_state 链）。
        let prop_value = |key: &str| -> Option<Value> {
            let p = props.get(key)?;
            match p {
                AuraPropValue::Expr(e) => self.resolve_expr_to_value(e, bindings),
                _ => None,
            }
        };
        let prop_str = |key: &str| -> Option<String> {
            prop_value(key).and_then(|v| match v {
                Value::Str(s) => Some(s.to_string()),
                Value::Int(i) => Some(i.to_string()),
                Value::Uint(u) => Some(u.to_string()),
                _ => None,
            })
        };
        let prop_int = |key: &str| -> Option<u64> {
            prop_value(key).and_then(|v| match v {
                Value::Int(i) if i >= 0 => Some(i as u64),
                Value::Uint(u) => Some(u as u64),
                Value::Float(f) if f >= 0.0 => Some(f as u64),
                _ => None,
            })
        };

        let use_flat = !flat_tabs.is_empty();
        let mut labels: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();
        if use_flat {
            for (i, t) in flat_tabs.iter().enumerate() {
                if let AuraNode::Element { props: tp, .. } = t {
                    labels.push(
                        self.extract_string_with(tp, "label", bindings)
                            .or_else(|| self.extract_string_with(tp, "text", bindings))
                            .unwrap_or_else(|| format!("Tab {}", i + 1)),
                    );
                    values.push(
                        self.extract_string_with(tp, "value", bindings)
                            .unwrap_or_else(|| i.to_string()),
                    );
                }
            }
        } else {
            for (i, t) in trigger_nodes.iter().enumerate() {
                if let AuraNode::Element {
                    props: tp,
                    children: tch,
                    ..
                } = t
                {
                    labels.push(
                        self.extract_string_with(tp, "text", bindings)
                            .or_else(|| self.extract_string_with(tp, "label", bindings))
                            .or_else(|| {
                                // 子件文本：直接 Text 子件 + text-like 元素
                                // 子件（`tabstrigger { text "Alpha" }` 的
                                // 解析形态是 Element{text}——复审 P641-R1
                                // 抓漏：浅层 Text 匹配漏掉该形态，标签
                                // 静默回退 "Tab N"）。同
                                // convert_text_element 的子件折叠链。
                                tch.iter().find_map(|c| match c {
                                    AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                                    AuraNode::Text(AuraTextContent::Interpolated {
                                        template,
                                        bindings: tpl_bindings,
                                    }) => Some(self.resolve_interpolation_with(
                                        template,
                                        tpl_bindings,
                                        bindings,
                                    )),
                                    AuraNode::Element {
                                        tag,
                                        props: cprops,
                                        children: cch,
                                        ..
                                    } if Self::TEXT_LIKE_TAGS.contains(&tag.as_str()) => {
                                        // child_element_text 只扫 props；
                                        // `text "Alpha"` 的字面量在元素自身
                                        // 子件里，须再落一层。
                                        self.child_element_text(cprops, bindings).or_else(|| {
                                            cch.iter().find_map(|d| match d {
                                                AuraNode::Text(AuraTextContent::Literal(s)) => {
                                                    Some(s.clone())
                                                }
                                                _ => None,
                                            })
                                        })
                                    }
                                    _ => None,
                                })
                            })
                            .unwrap_or_else(|| format!("Tab {}", i + 1)),
                    );
                    values.push(
                        self.extract_string_with(tp, "value", bindings)
                            .unwrap_or_else(|| i.to_string()),
                    );
                }
            }
        }

        // selected 解析：active（索引）→ value（索引或按值匹配）→
        // defaultvalue/default（同前）→ 0；越界钳制。
        let resolve_selected = |expr_idx: Option<u64>, expr_val: Option<String>| -> usize {
            if let Some(idx) = expr_idx {
                return idx as usize;
            }
            if let Some(v) = expr_val {
                if let Some(idx) = values.iter().position(|x| *x == v) {
                    return idx;
                }
                if let Ok(n) = v.parse::<usize>() {
                    return n;
                }
            }
            0
        };
        let has_active_or_value = props.keys().any(|k| {
            let k = k.to_ascii_lowercase();
            k == "active" || k == "value"
        });
        let mut selected = resolve_selected(prop_int("active"), prop_str("value"));
        if !has_active_or_value {
            selected = resolve_selected(
                prop_int("defaultvalue"),
                prop_str("defaultvalue").or_else(|| prop_str("default")),
            );
        }
        if labels.is_empty() {
            selected = 0;
        } else if selected >= labels.len() {
            selected = labels.len() - 1;
        }

        // contents：content 节点子件序列（多子件包一列）；平铺形态取 tab 子件。
        let content_sources: Vec<Vec<&AuraNode>> = if use_flat {
            flat_tabs
                .iter()
                .map(|t| match t {
                    AuraNode::Element { children, .. } => children.iter().collect(),
                    _ => Vec::new(),
                })
                .collect()
        } else {
            content_nodes
                .iter()
                .map(|c| match c {
                    AuraNode::Element { children, .. } => children.iter().collect(),
                    _ => Vec::new(),
                })
                .collect()
        };
        let mut contents: Vec<View<DynamicMessage>> = Vec::new();
        for parts in content_sources {
            let views: Vec<View<DynamicMessage>> = parts
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .collect();
            contents.push(if views.len() == 1 {
                views.into_iter().next().unwrap()
            } else {
                View::Column {
                    children: views,
                    spacing: 8,
                    padding: 0,
                    style: None,
                    onclick: None,
                    on_right_click: None,
                }
            });
        }

        // on_select：根 onselect 优先，缺省取首个 trigger onclick；事件
        // args 覆写为 [选中 tab 的 value 字符串]（handler 契约：首参=值，
        // details_onclick 运行时注参同款）。
        let root_onselect = aura_events_get_base(events, "onselect");
        let trigger_onclick = trigger_nodes.iter().find_map(|t| match t {
            AuraNode::Element { events: te, .. } => aura_events_get_base(te, "onclick"),
            _ => None,
        });
        // 回调载荷契约（fix-tabs-merged-look 修订）：首参=被点 tab 的
        // value 字符串（values 已含索引串兜底，点击时按索引现取），
        // `.at` 侧 `on { .Select(t) -> { .state = t } }` 直绑，免索引→值换算。
        let values_for_cb = values.clone();
        let on_select = root_onselect.or(trigger_onclick).map(|ev| {
            let handler = extract_handler_name(&ev.handler).to_string();
            let widget = self.widget_name.clone();
            let values = values_for_cb;
            crate::ui::view::TabsSelectCallback::new(move |idx| {
                let payload = values.get(idx).cloned().unwrap_or_else(|| idx.to_string());
                DynamicMessage::Typed {
                    widget_name: widget.clone(),
                    event_name: handler.clone(),
                    args: vec![Value::Str(payload.into())],
                }
            })
        });

        let variant = prop_str("variant")
            .map(|v| crate::ui::view::TabsVariant::parse(&v))
            .unwrap_or_default();

        View::Tabs {
            labels,
            contents,
            selected,
            position: crate::ui::view::TabsPosition::Top,
            on_select,
            style: self.extract_style(props),
            variant,
        }
    }

    /// PLAN-781 T-01：tracked 入口同款锚点包装（D-GAP 镜像纪律）。
    pub(super) fn convert_element_tracked_ctx(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // PLAN-031 T-03（V-04）：tracked/untracked 同一 design 作用域语义。
        let _design_scope = self.enter_element_design(props);
        let view = self.convert_element_tracked_ctx_inner(
            tag, props, events, children, path, id_map, probe, bindings,
        );
        self.maybe_wrap_scroll_anchor(view, props, bindings)
    }

    pub(super) fn convert_element_tracked_ctx_inner(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // PLAN-656 review F-1：别名拼写归一（scroll-pane→scroll 等）。
        let tag = Self::normalize_dispatch_tag(tag);
        // Record event handler bindings for this element, at this node's own
        // path (set by the caller's `path.push(child_index)`). Runs before the
        // `match tag` dispatch so it is unconditional — every element with
        // events (button/input/textarea/checkbox, etc.) is captured regardless
        // of whether its tag falls into the tracked or untracked converter arm.
        if !events.is_empty() {
            let ev_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
            for (event_name, ev) in events.iter() {
                // Plan 418 7.3-3: record the handler WITH its params — the
                // bare name made arg-differentiated bindings (e.g. the four
                // .MenuToggle("file"/"edit"/"view"/"help") menubar buttons)
                // indistinguishable in snapshots/inspector.
                let handler_with_params = if ev.params.is_empty() {
                    ev.handler.clone()
                } else {
                    format!("{}({})", ev.handler, ev.params.join(", "))
                };
                probe.record_event(&ev_path, event_name, &handler_with_params);
            }
        }

        // Plan 309 Phase 2b: record the declared class string (the `class`
        // prop, falling back to the inline `style=` prop) against this node's
        // path, so the inspector's Computed tab can show the original tokens.
        // `extract_string` resolves interpolations (static classes verbatim);
        // `Style` parsing (used by `extract_style`) would discard
        // whitespace/order and the `style=` fallback, so the probe keeps the
        // fuller string. `record_raw_class` is a no-op for `None`, so
        // class-less elements never gain a spurious probe entry.
        let raw_class = self
            .extract_string(props, "class")
            .or_else(|| self.extract_string(props, "style"));
        if raw_class.is_some() {
            let rc_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
            probe.record_raw_class(&rc_path, raw_class);
        }

        // Plan 409 §10 续: hidden/md:hidden 元素按桌面语义不渲染(放在 raw_class
        // 探针之后,inspector 仍能看到节点;event/raw_class 已先记录)。
        if self.extract_style(props).map_or(false, |s| s.is_hidden()) {
            return View::Empty;
        }

        match tag {
            // Plan 476: slot outlet——按调用位填充渲染（父作用域求值），
            // 未命中渲染 fallback children。五大容器的 children 走拼接
            // 展开（expand_children_spliced*），此臂覆盖其余位形。
            "slot" | "Slot" => {
                self.render_slot_outlet_tracked_ctx(props, children, path, id_map, probe, bindings)
            }
            // Core layout widgets — recurse children with path tracking.
            "col" | "column" => {
                let mut v =
                    self.convert_column_tracked_ctx(props, children, path, id_map, probe, bindings);
                self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                v
            }
            "row" => {
                let mut v =
                    self.convert_row_tracked_ctx(props, children, path, id_map, probe, bindings);
                self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                v
            }
            // Plan 463 T5: taskbar —— 桌面 shell 底栏（I4 登记）；row 语义
            // 水平排布,贴底锚定由宿主 shell 层装配做。镜像 untracked 同名臂。
            "taskbar" => {
                let mut v =
                    self.convert_row_tracked_ctx(props, children, path, id_map, probe, bindings);
                self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                v
            }
            // PLAN-641 T-03：tabs 复合组件折叠——有状态 View::Tabs（此前落
            // unknown fallback → Column）。契约见 convert_tabs 文档。
            "tabs" | "Tabs" => self.convert_tabs(props, events, children, bindings),
            // PLAN-530 步骤7（W12）：toggle_group VM 映射——横排连体 button
            // 组（tracked 镜像臂,D-GAP 规则）。见 toggle_group_rewrite_children。
            "togglegroup" | "toggle-group" | "toggle_group" => {
                let rewritten = self.toggle_group_rewrite_children(props, children, bindings);
                let mut v =
                    self.convert_row_tracked_ctx(props, &rewritten, path, id_map, probe, bindings);
                self.set_layout_events(&mut v, events, bindings);
                v
            }
            // 组外裸 item → 单 button（组内由重写改道,不走此臂）。
            "togglegroup-item" | "toggle-group-item" | "toggle_group_item" => {
                self.convert_button(props, events, children, bindings)
            }
            // PLAN-530 步骤8（W13）：alert-dialog 全族 → 模态 Popover 原语
            // （tracked 镜像臂,D-GAP 规则）。见 convert_alert_dialog_tracked_ctx。
            // PLAN-533 T5: dialog 家族（可关闭模态）同走模态 Popover 臂。
            "dialog" | "Dialog" => self.convert_alert_dialog_tracked_ctx(
                props,
                events,
                children,
                path,
                id_map,
                probe,
                bindings,
                ModalDialogFamily::Dialog,
            ),
            "dropdown-menu" | "dropdown_menu" | "dropdownmenu" | "DropdownMenu" => self
                .convert_alert_dialog_tracked_ctx(
                    props,
                events,
                    children,
                    path,
                    id_map,
                    probe,
                    bindings,
                    ModalDialogFamily::DropdownMenu,
                ),
            "alert-dialog" | "alert_dialog" | "alertdialog" => self
                .convert_alert_dialog_tracked_ctx(
                    props,
                events,
                    children,
                    path,
                    id_map,
                    probe,
                    bindings,
                    ModalDialogFamily::Alert,
                ),
            // PLAN-534: sheet/drawer → 贴边 Popover（Edge 族:scrim+外点/Esc
            // 关,shadcn Sheet 语义）。side/direction prop 四向,缺省 right。
            "sheet" | "Sheet" => {
                let placement = self.side_panel_placement(props, "side", bindings);
                self.convert_side_panel_tracked_ctx(
                    props, children, path, id_map, probe, bindings, placement, false,
                )
            }
            "drawer" | "Drawer" => {
                let placement = self.side_panel_placement(props, "direction", bindings);
                self.convert_side_panel_tracked_ctx(
                    props, children, path, id_map, probe, bindings, placement, true,
                )
            }
            // PLAN-534: hovercard → 非模态 Popover（Bottom 锚定 + MouseArea
            // hover 触发;on_dismiss=None,关闭只靠 leave）。
            "hovercard" | "hover-card" | "hover_card" | "HoverCard" => {
                self.convert_hovercard_tracked_ctx(props, children, path, id_map, probe, bindings)
            }
            // PLAN-534: sheet/drawer/hover-card trigger/content 组外兜底
            // 透传（root 臂已分区;组外裸渲染,镜像 dialog 先例）。
            "sheet-trigger" | "sheet_trigger" | "sheettrigger" | "drawer-trigger"
            | "drawer_trigger" | "drawertrigger" | "hover-card-trigger" | "hover_card_trigger"
            | "hovercard-trigger" | "hovercardtrigger" | "sheet-content" | "sheet_content"
            | "sheetcontent" | "drawer-content" | "drawer_content" | "drawercontent"
            | "hover-card-content" | "hover_card_content" | "hovercard-content"
            | "hovercardcontent" => {
                let mut views: Vec<View<DynamicMessage>> = Vec::new();
                for (i, c) in children.iter().enumerate() {
                    path.push(i);
                    views.push(self.convert_node_tracked_ctx(c, path, id_map, probe, bindings));
                    path.pop();
                }
                match views.len() {
                    0 => View::Empty,
                    1 => views.into_iter().next().unwrap(),
                    _ => View::Row {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    },
                }
            }
            // PLAN-534: sheet/drawer styled 子臂（镜像 dialog 同名臂）。
            "sheet-title" | "sheet_title" | "sheettitle" | "drawer-title" | "drawer_title"
            | "drawertitle" => {
                let p = self.with_class_prop(props, bindings, "text-lg font-semibold");
                self.convert_text_element(tag, &p, events, children, bindings)
            }
            "sheet-description" | "sheet_description" | "sheetdescription"
            | "drawer-description" | "drawer_description" | "drawerdescription" => {
                let p = self.with_class_prop(props, bindings, "text-sm text-muted-foreground");
                self.convert_text_element(tag, &p, events, children, bindings)
            }
            "sheet-header" | "sheet_header" | "sheetheader" | "drawer-header" | "drawer_header"
            | "drawerheader" => {
                let p = self.with_class_prop(props, bindings, "flex flex-col gap-2");
                self.convert_column_tracked_ctx(&p, children, path, id_map, probe, bindings)
            }
            "sheet-footer" | "sheet_footer" | "sheetfooter" | "drawer-footer" | "drawer_footer"
            | "drawerfooter" => {
                let p = self.with_class_prop(props, bindings, "flex justify-end gap-2");
                let mut v =
                    self.convert_row_tracked_ctx(&p, children, path, id_map, probe, bindings);
                self.set_layout_events(&mut v, events, bindings);
                v
            }
            "sheet-close" | "sheet_close" | "sheetclose" | "drawer-close" | "drawer_close"
            | "drawerclose" => {
                // 镜像 dialog-close:outline 按钮,onclick 取铸造 close。
                let mut p = props.clone();
                p.insert(
                    "variant".to_string(),
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("outline".into())),
                );
                self.convert_button(&p, events, children, bindings)
            }
            "alert-dialog-trigger"
            | "alert_dialog_trigger"
            | "alertdialog-trigger"
            | "alert-dialog-content"
            | "alert_dialog_content"
            | "alertdialog-content"
            | "dialog-trigger"
            | "dialog_trigger"
            | "dialogtrigger"
            | "dialog-content"
            | "dialog_content"
            | "dialogcontent" => {
                // 透传:子件原位渲染（dialog 臂已分区;组外兜底裸渲染）。
                let mut views: Vec<View<DynamicMessage>> = Vec::new();
                for (i, c) in children.iter().enumerate() {
                    path.push(i);
                    views.push(self.convert_node_tracked_ctx(c, path, id_map, probe, bindings));
                    path.pop();
                }
                match views.len() {
                    0 => View::Empty,
                    1 => views.into_iter().next().unwrap(),
                    _ => View::Row {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    },
                }
            }
            "alert-dialog-title" | "alert_dialog_title" | "alertdialog-title" | "dialog-title"
            | "dialog_title" | "dialogtitle" => {
                let p = self.with_class_prop(props, bindings, "text-lg font-semibold");
                self.convert_text_element(tag, &p, events, children, bindings)
            }
            "alert-dialog-description"
            | "alert_dialog_description"
            | "alertdialog-description"
            | "dialog-description"
            | "dialog_description"
            | "dialogdescription" => {
                let p = self.with_class_prop(props, bindings, "text-sm text-muted-foreground");
                self.convert_text_element(tag, &p, events, children, bindings)
            }
            "alert-dialog-header"
            | "alert_dialog_header"
            | "alertdialog-header"
            | "dialog-header"
            | "dialog_header"
            | "dialogheader" => {
                let p = self.with_class_prop(props, bindings, "flex flex-col gap-2");
                self.convert_column_tracked_ctx(&p, children, path, id_map, probe, bindings)
            }
            "alert-dialog-footer"
            | "alert_dialog_footer"
            | "alertdialog-footer"
            | "dialog-footer"
            | "dialog_footer"
            | "dialogfooter" => {
                let p = self.with_class_prop(props, bindings, "flex justify-end gap-2");
                let mut v =
                    self.convert_row_tracked_ctx(&p, children, path, id_map, probe, bindings);
                self.set_layout_events(&mut v, events, bindings);
                v
            }
            "dialog-close" | "dialog_close" | "dialogclose" => {
                // PLAN-533 T5: dialog-close → outline 按钮（cancel 同款预设）。
                let mut p = props.clone();
                p.insert(
                    "variant".to_string(),
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("outline".into())),
                );
                self.convert_button(&p, events, children, bindings)
            }
            // PLAN-533 T7: dropdown-menu 子件——item（有 onclick 走按钮,
            // 纯文本项 text 预设）/label/separator。
            "dropdown-menu-item" | "dropdown_menu_item" | "dropdownmenuitem" => {
                let preset =
                    "w-full px-2 py-1.5 text-sm cursor-pointer hover:bg-secondary text-start";
                if events
                    .iter()
                    .any(|(e, _)| matches!(e.as_str(), "onclick" | "onClick" | "on_click"))
                {
                    let mut b = props.clone();
                    b.insert(
                        "variant".to_string(),
                        AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("ghost".into())),
                    );
                    let b2 = self.with_class_prop(&b, bindings, preset);
                    self.convert_button(&b2, events, children, bindings)
                } else {
                    let p = self.with_class_prop(props, bindings, preset);
                    self.convert_text_element(
                        tag,
                        &p,
                        &std::collections::HashMap::new(),
                        children,
                        bindings,
                    )
                }
            }
            "dropdown-menu-label" | "dropdown_menu_label" | "dropdownmenulabel" => {
                let p = self.with_class_prop(props, bindings, "px-2 py-1.5 text-sm font-semibold");
                self.convert_text_element(
                    tag,
                    &p,
                    &std::collections::HashMap::new(),
                    children,
                    bindings,
                )
            }
            "dropdown-menu-separator" | "dropdown_menu_separator" | "dropdownmenuseparator" => {
                View::Container {
                    child: Box::new(View::Empty),
                    padding: 0,
                    width: None,
                    height: None,
                    center_x: false,
                    center_y: false,
                    onclick: None,
                    on_right_click: None,
                    style: Style::parse("w-full h-px bg-border my-1").ok(),
                }
            }
            "alert-dialog-cancel" | "alert_dialog_cancel" | "alertdialog-cancel" => {
                // outline 变体预设（border+bg-background+text-foreground）,
                // 裸类注入无文字色,暗底上 Cancel 字形不可见。
                let mut p = props.clone();
                p.insert(
                    "variant".to_string(),
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("outline".into())),
                );
                self.convert_button(&p, events, children, bindings)
            }
            "alert-dialog-action" | "alert_dialog_action" | "alertdialog-action" => {
                self.convert_button(props, events, children, bindings)
            }
            "grid" => self.convert_grid_tracked_ctx(props, children, path, id_map, probe, bindings),
            "center" => {
                self.convert_center_tracked_ctx(props, children, path, id_map, probe, bindings)
            }
            // PLAN-055 ④/T12: pre/code 进容器转换臂（带样式）——此前落
            // unknown fallback 成 style:None 的 Column，类串整体丢弃
            // （musk think 展开区/specs fixture-code 的 padding/border-t/
            // max-h 全失）。容器臂的 Style::parse 覆盖 py-[Npx]/px-[Npx]/
            // border-t/max-h-[Npx]；VM 无滚动，max-h 尽力 clip（滚动残项
            // 留档 KD 行 86 ⑤）。
            "container" | "div" | "pre" | "code" => {
                // PLAN-055: div{html: …}（v-html）VM 轨降级——去标签纯文本。
                // 此前 html prop 被静默丢弃（builder 无消费臂），UserMessage
                // 气泡体渲染为空 div；musk 聊天气泡文案缺失面。mention 高亮
                // 为 web-only 增强（VM 降级登记），此处只读已求值 props，
                // 不触碰 eval 链（493 UAF 雷区规避口径不变）。
                if let Some(stripped) = self.div_html_stripped_text(props, bindings) {
                    let mut enriched: Vec<AuraNode> = children.to_vec();
                    enriched.push(AuraNode::Text(AuraTextContent::Literal(stripped)));
                    let mut v = self.convert_container(props, &enriched, bindings);
                    self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                    return v;
                }
                let mut v = self
                    .convert_container_tracked_ctx(props, children, path, id_map, probe, bindings);
                self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                v
            }
            // Plan 442 A4: svg 元素子树 → 序列化 SVG 文档经 View::Image 渲染
            // (此前落 unknown fallback → View::Empty,VM 轨 svg 完全不渲染)。
            // 镜像 convert_element 的同名臂(文件 D-GAP 规则)。
            "svg" => self.convert_svg_image(props, children, bindings),
            // Plan 484: hover 命中区 —— 透明 mouse_area(仅 enter/exit 事件
            // 转发,无视觉)。chart 组件 tooltip 的命中原语;尺寸/绝对定位由
            // style 承担,children 通常为空。
            "mouse-area" => {
                self.convert_mouse_area(props, events, children, path, id_map, probe, bindings)
            }
            // Plan 563: 状态驱动画布 —— scene 双表 → 结构化场景快照
            // + pen 事件三件套(无子元素 leaf,tracked/untracked 共用)。
            "canvas" => self.convert_canvas(props, events, bindings),
            // Plan 497: 每窗口真缩略 leaf（与 tracked 层同名臂镜像，D-GAP；
            // 字面形式与 render_support/schema.rs 三表同款 window_thumbnail）。
            "window_thumbnail" => self.convert_window_thumbnail(props, bindings),
            // PLAN-012 W3: 整桌面等比预览 leaf（tracked/untracked 双臂镜像，
            // window_thumbnail 同款 D-GAP 纪律；SD-02 DSL 合同面）。
            "workspace_preview" => self.convert_workspace_preview(props, bindings),
            // PLAN-760: 宿主侧远程帧消费 leaf（tracked/untracked 双臂镜像，
            // window_thumbnail 同款 D-GAP 纪律；url 即身份）。
            "remote-frame" => self.convert_remote_frame(props, bindings),
            // Plan 409 §10 续 3: HTML 语义/布局标签(scroll/aside/main/header...),
            // 之前落 fallback 丢 style。scroll → 可滚动 column;其余 → container。
            "scroll" | "scrollable" => self
                .convert_scroll_tracked_ctx(props, events, children, path, id_map, probe, bindings),
            // PLAN-656 T-06: synthetic managed content（capability-test 专用）。
            "scroll_test_content" => self.managed_scroll_content_view(props, bindings),
            // Plan 482: nav 容器支持 search:true 集成搜索行（子节点随 untracked
            // 转换，同 button 先例）。
            "nav" => self.convert_nav_container(props, events, children, bindings),
            // Plan 561: sidebar_* VM 契约子集 —— 实现集中在 untracked 转换器组
            // （本节镜像 untracked 站同名臂，D-GAP 规则；契约类见
            // ui_gen::sidebar_contract）。
            "sidebar" => self.convert_sidebar_root(props, children, bindings),
            "sidebar_provider" | "sidebar-provider" => {
                self.convert_sidebar_provider(props, events, children, bindings)
            }
            "sidebar_header" | "sidebar-header" => self.convert_sidebar_region(
                props,
                children,
                bindings,
                auto_ui_compiler::ui_gen::sidebar_contract::HEADER_BASE,
            ),
            "sidebar_footer" | "sidebar-footer" => self.convert_sidebar_region(
                props,
                children,
                bindings,
                auto_ui_compiler::ui_gen::sidebar_contract::FOOTER_BASE,
            ),
            "sidebar_content" | "sidebar-content" => {
                self.convert_sidebar_content(props, children, bindings)
            }
            "sidebar_separator" | "sidebar-separator" => self.convert_sidebar_separator(),
            "sidebar_inset" | "sidebar-inset" => self.convert_sidebar_region(
                props,
                children,
                bindings,
                auto_ui_compiler::ui_gen::sidebar_contract::INSET_BASE,
            ),
            "sidebar_group" | "sidebar-group" => {
                self.convert_sidebar_group(props, children, bindings)
            }
            "sidebar_group_label" | "sidebar-group-label" => {
                self.convert_sidebar_group_label(tag, props, events, children, bindings)
            }
            "sidebar_group_content" | "sidebar-group-content" => self.convert_sidebar_region(
                props,
                children,
                bindings,
                auto_ui_compiler::ui_gen::sidebar_contract::GROUP_CONTENT,
            ),
            "sidebar_group_action" | "sidebar-group-action" => {
                self.convert_sidebar_group_action(props, events, children, bindings)
            }
            "sidebar_menu" | "sidebar-menu" => self.convert_sidebar_region(
                props,
                children,
                bindings,
                auto_ui_compiler::ui_gen::sidebar_contract::MENU_BASE,
            ),
            "sidebar_menu_item" | "sidebar-menu-item" => self.convert_sidebar_region(
                props,
                children,
                bindings,
                auto_ui_compiler::ui_gen::sidebar_contract::MENU_ITEM,
            ),
            "sidebar_menu_action" | "sidebar-menu-action" => {
                self.convert_sidebar_menu_action(props, events, children, bindings)
            }
            "sidebar_menu_badge" | "sidebar-menu-badge" => {
                self.convert_sidebar_menu_badge(props, children, bindings)
            }
            "sidebar_menu_button" | "sidebar-menu-button" => {
                self.convert_sidebar_menu_button(props, events, children, bindings, false)
            }
            "sidebar_menu_sub" | "sidebar-menu-sub" => self.convert_sidebar_region(
                props,
                children,
                bindings,
                auto_ui_compiler::ui_gen::sidebar_contract::MENU_SUB,
            ),
            "sidebar_menu_sub_item" | "sidebar-menu-sub-item" => {
                self.convert_children_passthrough(children, bindings)
            }
            "sidebar_menu_sub_button" | "sidebar-menu-sub-button" => {
                self.convert_sidebar_menu_button(props, events, children, bindings, true)
            }
            // PLAN-031 T-05：panel/card 与 untracked 层同名臂镜像（D-GAP 规则）
            // ——结构容器保留 class/style 与 design 局部 frame。
            "aside" | "main" | "header" | "section" | "footer" | "article" | "panel" | "card" => {
                self.convert_container_tracked_ctx(props, children, path, id_map, probe, bindings)
            }

            // Text-bearing elements. The text/interpolation state bindings are
            // captured at this node's current path (the text element's path),
            // which is what the inspector wants.
            "text" | "label" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span" | "a"
            | "link" | "small" | "strong" | "em" | "b" | "i" => self
                .convert_text_element_tracked_ctx(
                    tag, props, events, children, path, probe, bindings,
                ),

            // Leaf/atom widgets with no AuraNode children — fall back to the
            // untracked converter. They have no nested text to probe (Task 9
            // scope is text interpolation only).
            "button" | "btn" | "native_button" => {
                self.convert_button(props, events, children, bindings)
            }
            // native_button：vue 侧显式原生逃生名（避开 button→shadcn
            // Button 映射，schema.rs 2514 注）；VM 侧与 button 同臂
            //（PLAN-051 T6：demo settings 弹层按钮两轨原生）。
            // Plan 482: nav 组件族 —— nav_contract 契约类转换（hover/active
            // 三态 + icon/desc/badge 槽 + to/onclick 双模式）。
            "nav-item" | "nav_item" => self.convert_nav_item(props, events, children, bindings),
            "nav-group" | "nav_group" => self.convert_nav_group(props, events, children, bindings),
            // Plan 409 §10 续 8: badge = shadcn Badge(水平 inline + variant 配色)。
            "badge" => self.convert_badge(props, children, bindings),
            "input" => self.convert_input(props, events, bindings),
            "textarea" => self.convert_textarea(props, events, bindings),
            // PLAN-661 T-03: slider（025 fixture 语义基准——value/onchange
            // 载荷/step；SliderChangeHandler 经 T-02 newtype 可跨消息映射）。
            "slider" => self.convert_slider(props, events, bindings),
            // Plan 413: code editor widget (syntax highlighting, line
            // numbers, wrap, vi, undo, IME).
            "code_editor" | "codeEditor" | "codeeditor" => {
                self.convert_code_editor(props, events, bindings)
            }
            // PLAN-009 P1: native terminal component(auto-term 引擎网格)。
            "terminal" | "Terminal" => self.convert_terminal(props, events, bindings),
            // Plan 019 批次九拆分（PLAN-066 T2 迁出）：`autodown_editor` 别名
            // 走可编辑文档编辑器——原硬编码臂迁经 NativeWidgetRegistry 派发
            // （native_widget::global 注册，案 a sugar：factory 仍产
            // View::AutodownEditor，快照 kind 与断言面零改动）。臂体见
            // convert_autodown_editor_native；无 feature 时 global 注册
            // convert_textarea 降级（D-GAP-3 链保持）。

            // Plan 019 批次七: markdown/autodown → 只读真渲染（autodown-core
            // parse_blocks → 面板树 → View）。无 feature 时维持 D-GAP-3
            // textarea 降级。
            "autodown" | "markdown_editor" | "markdown" => {
                #[cfg(feature = "autodown")]
                {
                    let content = self
                        .extract_string_with(props, "content", bindings)
                        .or_else(|| self.extract_string_with(props, "value", bindings))
                        .unwrap_or_default();
                    let is_final = props
                        .get("final")
                        .map(|v| match v {
                            AuraPropValue::Expr(expr) => self
                                .resolve_expr_to_value(expr, bindings)
                                .map(|val| val.as_bool()),
                            _ => None,
                        })
                        .flatten()
                        .unwrap_or(true);
                    // Plan 040 契约扩展——VM v1 豁免收口（PLAN-044 T4）：
                    // streaming 恒按 final（豁免保留）；placeholder_*（ghost）
                    // 自本计划真消费（绑定求值传入 render）；scroll_sync
                    // 消费同前——true 时文档外包 View::Scrollable（offset
                    // 绑定写入 + on_scroll 消息读出，稳定 id 由
                    // render_dynamic_view 的 vnode_* 路径派生）。
                    let _ = props.get("streaming");
                    let placeholder = self.autodown_placeholder(props, bindings);
                    let (scroll_sync, offset, on_scroll, details_onclick) =
                        self.autodown_scroll_binding(props, events, bindings, None);
                    // PLAN-045 T5：列宽状态 + 落定通道（oncolresize）。
                    let table_widths = self.autodown_table_widths(props, bindings);
                    let col_resize = self.autodown_on_col_resize_binding(events);
                    // PLAN-041 T8：流式增量（结构键 diff + 未变块复用），
                    // 按节点 path 身份挂缓存。
                    let cache_key = format!("{path:?}");
                    let mut reg = autodown_stream_registry().lock().unwrap();
                    let cache = stream_registry_entry(&mut reg, cache_key);
                    let mut doc = crate::ui::autodown_render::render_document_streamed_with(
                        cache,
                        &content,
                        is_final,
                        details_onclick.as_deref(),
                        placeholder,
                        table_widths.as_ref(),
                        col_resize.as_ref(),
                    );
                    // PLAN-058 T12（转介④）：元素 class 的观感段（py-*/px-*/
                    // p-* 内边距 token）消费——内容侧 padded Container（vue
                    // 的 padding 语义在滚动容器内壁）。混合类的结构段
                    // （flex-1/min-h-0/overflow-hidden）仍不并入包装层
                    //（Fill×Fill 视口语义由 Scrollable 自带，043 T6 炸布局
                    // 注记维持）。
                    let pad_class = self.autodown_padding_class(props, bindings);
                    if !pad_class.is_empty() {
                        if let Ok(st) = Style::parse(&pad_class) {
                            doc = View::Container {
                                child: Box::new(doc),
                                padding: 0,
                                width: None,
                                height: None,
                                center_x: false,
                                center_y: false,
                                style: Some(st),
                                onclick: None,
                                on_right_click: None,
                            };
                        }
                    }
                    if scroll_sync {
                        // PLAN-043 T6：包装层取纯 w-full h-full 合成样式
                        //（Fill×Fill 视口约束——元素 class 混合样式实测炸
                        // 布局），内层文档列收缩到内容全高外滚。
                        doc = View::Scrollable {
                            child: Box::new(doc),
                            width: None,
                            height: None,
                            style: Style::parse("w-full h-full").ok(),
                            auto_scroll: false,
                            offset,
                            on_scroll,
                            axes: crate::ui::scroll::ScrollAxes::Y,
                            scrollbar_policy: crate::ui::scroll::ScrollbarPolicy::Auto,
                            controller: None,
                        };
                    }
                    return doc;
                }
                #[cfg(not(feature = "autodown"))]
                self.convert_textarea(props, events, bindings)
            }
            "checkbox" | "check" => self.convert_checkbox(props, events, bindings),
            "imagesurface" | "image-surface" | "image_surface" | "ImageSurface" => {
                self.convert_image_surface(props, events, bindings)
            }
            // PLAN-617 T-19: `video` —— 受控媒体契约的 VM 侧节点（原生播放面）。
            "video" | "Video" => self.convert_video(props, events, bindings),
            "img" | "image" | "icon" => self.convert_image_or_icon(props, bindings),
            "progress" => self.convert_progress(props, events, bindings),
            "spacer" => self.convert_spacer(props),
            // Plan 412 §4.3: demo 占位块(纯展示,无 probe 需求)
            "square" => self.convert_square(props, children, bindings),
            "divider" | "hr" => self.convert_divider(props),
            "sep" | "separator" => self.convert_sep(props, bindings),
            // os-007（origin PLAN-577/P534-D4）：avatar 家族补齐——
            // avatar 容器转换子件；avatar-image 复用 img 图源臂；
            // avatar-fallback 走文本臂（居中由容器注入）。
            "avatar" => self.convert_avatar(props, children, bindings),
            "avatar-image" => self.convert_image_or_icon(props, bindings),
            "avatar-fallback" => {
                self.convert_text_element("avatar-fallback", props, events, children, bindings)
            }
            // Plan 418 P2-3: config-driven menubar/toolbar (auto-edit.at).
            // path/probe pass through so synthesized buttons land in the
            // snapshot's event index (MCP clickability). §8.4①: probe off
            // (capture_debug=false)时 record_event 早退,零开销。
            "menubar" => {
                let p = path.clone();
                // PLAN-630 T-01: 有子节点 = 声明式组件族；空标签 = actions
                // DSL 合成（向后兼容，T10 热重载锚不受影响）。
                if !children.is_empty() {
                    self.convert_menubar_component(props, children, bindings, Some((&p, probe)))
                } else {
                    self.convert_menubar(props, bindings, Some((&p, probe)))
                }
            }
            "toolbar" => {
                let p = path.clone();
                self.convert_toolbar(props, bindings, Some((&p, probe)))
            }
            // Plan 422 P3: `popover` DSL 标签 —— 锚定弹层的应用侧入口
            // (contextmenu 坐标锚 / widget 锚两形态,见 convert_popover)。
            "popover" => {
                self.convert_popover(props, events, children, path, id_map, probe, bindings)
            }

            // PLAN-717: Calendar is supplied by the Vue widget package, but
            // the VM backend has no native calendar widget. Render the month
            // as a native date grid so Calendar and DatePicker samples remain
            // visible instead of collapsing to an empty unknown component.
            "calendar" | "Calendar" => self.convert_calendar(props, bindings),

            // Command palette examples are registered as web components too;
            // dispatch them here before the generic imported-component fallback.
            "command" => {
                let p = self.with_class_prop(
                    props,
                    bindings,
                    "w-full rounded-md border bg-popover text-popover-foreground shadow-sm",
                );
                self.convert_column_tracked_ctx(&p, children, path, id_map, probe, bindings)
            }
            "command_list" | "command-list" => {
                let p = self.with_class_prop(props, bindings, "flex flex-col gap-1 p-1");
                self.convert_column_tracked_ctx(&p, children, path, id_map, probe, bindings)
            }
            "command_group" | "command-group" => {
                let mut group_children = Vec::new();
                if let Some(heading) = self
                    .extract_string_with(props, "heading", bindings)
                    .filter(|s| !s.is_empty())
                {
                    group_children.push(View::Text {
                        content: heading,
                        style: Style::parse(
                            "px-2 py-1.5 text-xs font-medium text-muted-foreground",
                        )
                        .ok(),
                        selectable: false,
                    });
                }
                for (i, child) in children.iter().enumerate() {
                    path.push(i);
                    let view = self.convert_node_tracked_ctx(child, path, id_map, probe, bindings);
                    path.pop();
                    if !is_visually_empty(&view) {
                        group_children.push(view);
                    }
                }
                View::Column {
                    children: group_children,
                    spacing: 2,
                    padding: 0,
                    style: self.extract_style_with(props, bindings),
                    onclick: None,
                    on_right_click: None,
                }
            }
            "command_input" | "command-input" => {
                let p = self.with_class_prop(
                    props,
                    bindings,
                    "w-full border-b rounded-none px-3 py-2 text-sm",
                );
                self.convert_input(&p, events, bindings)
            }
            "command_item" | "command-item" => {
                let mut p = self.with_class_prop(
                    props,
                    bindings,
                    "w-full justify-start px-2 py-1.5 text-sm",
                );
                p.insert(
                    "variant".to_string(),
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("ghost".into())),
                );
                self.convert_button(&p, events, children, bindings)
            }
            "command_empty" | "command-empty" => View::Empty,
            "command_separator" | "command-separator" => View::Container {
                child: Box::new(View::Empty),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                onclick: None,
                on_right_click: None,
                style: Style::parse("w-full h-px bg-border my-1").ok(),
            },
            "command_shortcut" | "command-shortcut" => {
                let p = self.with_class_prop(
                    props,
                    bindings,
                    "ml-auto text-xs tracking-widest text-muted-foreground",
                );
                self.convert_text_element(tag, &p, events, children, bindings)
            }

            // Static VM layout: preserve each slide in a compact horizontal row.
            "carousel" => {
                self.convert_column_tracked_ctx(props, children, path, id_map, probe, bindings)
            }
            "carousel_content" | "carousel-content" => {
                let p = self.with_class_prop(props, bindings, "w-full gap-2");
                self.convert_row_tracked_ctx(&p, children, path, id_map, probe, bindings)
            }
            "carousel_item" | "carousel-item" => {
                let p = self.with_class_prop(
                    props,
                    bindings,
                    "w-24 shrink-0 rounded-md border bg-card p-2",
                );
                self.convert_container_tracked_ctx(&p, children, path, id_map, probe, bindings)
            }
            "carousel_previous" | "carousel-previous" => {
                let mut p = props.clone();
                p.entry("text".to_string()).or_insert_with(|| {
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("‹".into()))
                });
                p.entry("variant".to_string()).or_insert_with(|| {
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("outline".into()))
                });
                p.entry("size".to_string()).or_insert_with(|| {
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("icon".into()))
                });
                self.convert_button(&p, events, children, bindings)
            }
            "carousel_next" | "carousel-next" => {
                let mut p = props.clone();
                p.entry("text".to_string()).or_insert_with(|| {
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("›".into()))
                });
                p.entry("variant".to_string()).or_insert_with(|| {
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("outline".into()))
                });
                p.entry("size".to_string()).or_insert_with(|| {
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("icon".into()))
                });
                self.convert_button(&p, events, children, bindings)
            }

            // PLAN-050 T7 (C5): 图标组件臂（tracked 层镜像,见 convert_element
            // 同名臂注释）。图标是叶子节点,无 probe 文本需求。
            // PLAN-053 P-053-6: registry 已注册同名适配器 widget（renderer.vm.at
            // Markdown 纯文本降级等 use.web component 的 VM 形态）优先于图标
            // 回落——原臂无条件吞掉,注册组件被画成 lucide glyph(View::Image),
            // 消息正文整体成图标占位。
            tag if self.is_imported_component(tag)
                && self.widget_registry.and_then(|r| r.get(tag)).is_none() =>
            {
                // PLAN-625 T-09(b): 非 icon 的 web-ecosystem 组件不再画 lucide
                // glyph 占位（组件 ≠ 图标;ui-gallery AppViewport 实证=空盒无
                // 信息）——降级为可读占位卡（组件名 + Web 专属提示）。
                // PLAN-748 T-02: 判别器改为 lucide 名集成员资格
                // （is_icon_component_tag），PascalCase lucide 名恢复 glyph。
                if is_icon_component_tag(tag) {
                    self.convert_icon_component(tag, props, bindings)
                } else {
                    self.convert_web_component_placeholder(tag, props, bindings)
                }
            }

            // Child widget lookup or fallback.
            _ => {
                // Plan 408: nav-link renders as a navigable button (like link).
                if tag == "nav-link" || tag == "nav_link" {
                    let to = self
                        .extract_string(props, "to")
                        .or_else(|| self.extract_string(props, "href"))
                        .unwrap_or_default();
                    let label = self
                        .extract_string(props, "label")
                        .or_else(|| self.extract_string(props, "text"))
                        .unwrap_or_default();
                    let icon = self.extract_string(props, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(
                        &label,
                        &[],
                        &to,
                        &icon,
                        bindings,
                        false,
                    );
                }
                // Plan 412 续(toast VM 化):toast-provider 是 vue 端 <Toaster/>
                // 挂载点;VM 端的悬浮层由 renderer 在 dynamic_view 顶层注入
                // (handler 的 toast() 调用 → __toast state),这里渲染为空,
                // 避免落 fallback 产生占位噪音。
                if tag == "toast-provider" || tag == "toast_provider" || tag == "toaster" {
                    return View::Empty;
                }
                // PLAN-066: 原生外部组件注册表——内置臂穷尽后、.at AuraWidget
                // 前查（原生优先语义保持，提案 066 §3.3）。命中即接管。
                if let Some(nv) = self.native_widget_view(tag, props, events, children, bindings) {
                    return nv;
                }
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(tag) {
                        // D-GAP-4: tracked variant so the child subtree's
                        // style/event bindings reach the BuildProbe (snapshot).
                        // Plan 476: slot 填充透传(tracked 双胎)。
                        let fills = self.slot_fills_for(children, bindings);
                        return self.render_child_widget_tracked(
                            child_widget,
                            props,
                            events,
                            bindings,
                            path,
                            id_map,
                            probe,
                            fills.as_ref(),
                        );
                    }
                }
                // Plan 409 §10 续 22: 其余 tag(category-section/component-card/
                // preview-card/codeblock/table 等)delegate 到 untracked convert_element,
                // 复用其完整实现(响应式 grid/卡片/代码块/表格/toggle 等),避免在
                // tracked 重复维护。这些纯展示组件无 debug probe 需求(渲染正确即可)。
                self.convert_element(tag, props, events, children, bindings)
            }
        }
    }

    /// Tracked convert_column — mirrors `convert_column` but recurses via
    /// `convert_node_tracked_ctx` so each child gets its own path + probe data.
    pub(super) fn convert_column_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self
            .extract_u16(props, "spacing")
            .or_else(|| self.extract_u16(props, "gap").map(|g| g * 4))
            .unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // D-GAP-4: number children by RESULTING slot — visually-empty views
        // are dropped from the built column, shifting later siblings left;
        // recording at the post-filter position keeps probe/VTree paths
        // identical by construction (mirrors convert_row_tracked_ctx).
        // Plan 476: 拼接感知收集——slot outlet 的 fill 子节点作兄弟展开
        // (父作用域 tracked 转换,编号同幸存槽位约定)。
        let child_views =
            self.expand_children_spliced_resulting(children, path, id_map, probe, bindings);

        // PLAN-536 T6(题5 ①②): absolute 悬浮 hoist 激活到 tracked 主渲染
        // 路径——此前 hoist 臂只挂在未追踪 convert_column(409 §10 续 5),
        // 真实视图走 build_with_debug_gated 从不过臂,"absolute 类 VM 不
        // 消费退化为流内排布"(musk 会话卡 × 实录)即此。语义定界:
        // absolute+偏移+z → 父容器 overlay 层(锚=父 bounds,不占流内格);
        // absolute 无 z → 分层技巧(textarea 叠加族)保留流内。
        let mut floats: Vec<(View<DynamicMessage>, crate::ui::view::OverlayPosition)> = Vec::new();
        let child_views: Vec<View<DynamicMessage>> = child_views
            .into_iter()
            .filter_map(|v| match extract_absolute_position(&v) {
                Some(pos) => {
                    floats.push((v, pos));
                    None
                }
                None => Some(v),
            })
            .collect();

        // Plan 412 F1: 交叉类仲裁 — col 元素带 `grid grid-cols-N`/`flex` 类时
        // 布局语义被类覆盖(元素语义为默认,显式类优先)。hoist 在前,浮层不占格。
        if let Some(RederivedLayout::Grid { cols, gap }) = rederive_layout(style.as_ref()) {
            let base = View::Grid {
                cols,
                gap,
                cells: child_views,
                style,
            };
            return fold_floats(base, floats);
        }
        let use_row = matches!(rederive_layout(style.as_ref()), Some(RederivedLayout::Row));

        let mut builder = if use_row {
            View::<DynamicMessage>::row()
        } else {
            View::<DynamicMessage>::col()
        }
        .spacing(spacing)
        .padding(padding);
        // Plan 048:提取 overflow 标志 + style clone(builder.with_style 会 move style)。
        // style_clone 传给 Scrollable,让 build_scrollable 读到 flex-1 → height(Fill)。
        let needs_scroll = style.as_ref().map_or(false, |s| {
            s.classes.iter().any(|c| {
                matches!(
                    c,
                    crate::ui::style::StyleClass::OverflowYAuto
                        | crate::ui::style::StyleClass::OverflowAuto
                )
            })
        });
        let scroll_style = if needs_scroll { style.clone() } else { None };
        // PLAN-748 T-07：滚动内容 col 拿视觉类剥离后的余集（bg/border 上移
        // 视口层，见 split_scroll_visual_classes）。
        match (style, scroll_style.as_ref()) {
            (Some(s), Some(sc)) => {
                let (_, rest) = split_scroll_visual_classes(sc);
                builder = builder.with_style(rest);
            }
            (Some(s), None) => {
                builder = builder.with_style(s);
            }
            _ => {}
        }
        for child in child_views {
            builder = builder.child(child);
        }
        let col_view = builder.build();
        // PLAN-536 T6: overflow 包 Scrollable 之后再包 Overlay 悬浮层——
        // 浮层锚=父容器(含滚动外壳)整体 bounds。
        if needs_scroll {
            let sc = scroll_style.expect("needs_scroll ⇒ Some");
            let (viewport_visual, _) = split_scroll_visual_classes(&sc);
            let scroll_view = View::Scrollable {
                child: Box::new(col_view),
                width: None,
                height: None,
                style: if viewport_visual.is_some() {
                    Some(ensure_full_scroll_dims(&sc))
                } else {
                    Some(sc)
                },
                auto_scroll: false,
                offset: None,
                on_scroll: None,
                axes: crate::ui::scroll::ScrollAxes::Y,
                scrollbar_policy: crate::ui::scroll::ScrollbarPolicy::Auto,
                controller: None,
            };
            // PLAN-748 T-07：视觉子集上移视口容器（Shrink 包装随 Scrollable
            // Fill 解析满高；build_container 经 apply_side_borders 承载单侧
            // 边框）。无视觉类 → 零包装层（现行为不变）。
            let base = match viewport_visual {
                Some(v) => View::Container {
                    child: Box::new(scroll_view),
                    padding: 0,
                    width: None,
                    height: None,
                    center_x: false,
                    center_y: false,
                    style: Some(v),
                    onclick: None,
                    on_right_click: None,
                },
                None => scroll_view,
            };
            return fold_floats(base, floats);
        }
        fold_floats(col_view, floats)
    }

    /// PLAN-656 T-06: synthetic managed content 提取——`scroll-test-content`
    ///（capability-test 专用，非 public widget；plan r2 §11）。props：
    /// key（稳定宿主键，缺省 "cap"）/ extent_w / extent_h（逻辑 px，缺省
    /// 2M×10M）。双臂共用（D-GAP）。
    pub(super) fn managed_scroll_content_view(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let key = self
            .extract_string_with(props, "key", bindings)
            .unwrap_or_else(|| "cap".to_string());
        let num = |name: &str, default: f64| {
            props
                .get(name)
                .and_then(|v| match v {
                    AuraPropValue::Expr(expr) => self.resolve_expr_to_value(expr, bindings),
                    _ => None,
                })
                .and_then(|val| match val {
                    Value::Float(f) => Some(f),
                    Value::Double(d) => Some(d),
                    Value::Int(i) => Some(i as f64),
                    Value::Uint(u) => Some(u as f64),
                    _ => None,
                })
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(default)
        };
        let logical_w = num(
            "extent_w",
            crate::ui::scroll::SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_W,
        );
        let logical_h = num(
            "extent_h",
            crate::ui::scroll::SYNTHETIC_MANAGED_DEFAULT_LOGICAL_EXTENT_H,
        );
        // axes prop 仅供信息面（host 创建恒 BOTH；实际滚动轴由外层
        // scroll-pane 的 axis 决定）。
        View::ManagedScrollContent {
            key,
            logical_w,
            logical_h,
            axes: crate::ui::scroll::ScrollAxes::BOTH,
        }
    }

    /// PLAN-656 复审修复（review F-1）：schema 别名拼写在派发面归一——
    /// builder 收到的是**原始 tag**（resolve_tag 折叠在 schema 校验层），
    /// 四表同步收口曾把 vb 臂的别名拼写移除，导致 VM 轨 `scroll-pane` 沦为
    /// unknown fallback（快照实证 y-rows 裸挂 col、无 scrollable 节点）。
    /// 归一单点在两派发入口（tracked/untracked），臂内保持 canonical。
    pub(super) fn normalize_dispatch_tag(tag: &str) -> &str {
        // if/else 链（非 `match tag {`——schema_drift 语句级派发表计数器只认
        // match 形态，小别名归一不构成新派发表维度）。
        if tag == "scroll-pane" {
            "scroll"
        } else if tag == "scroll-test-content" {
            "scroll_test_content"
        } else {
            tag
        }
    }

    /// PLAN-656 T-03: scroll-pane 语义提取——tracked/untracked 双臂共用
    /// （D-GAP：共享逻辑单点）。解析 `axis:`/`scrollbar:`/`controller:`
    /// props 与 `onscroll` 事件；legacy `direction:` 映射 axis（axis 优先）。
    pub(super) fn scroll_pane_semantics(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> (
        crate::ui::scroll::ScrollAxes,
        crate::ui::scroll::ScrollbarPolicy,
        Option<crate::ui::view::ScrollControllerBinding>,
        Option<crate::ui::view::ScrollCallback<DynamicMessage>>,
    ) {
        // axis 优先；缺失时 legacy direction 词表同源映射（vertical→y
        // 缺省回落，horizontal→x，both→both）。
        let axes = self
            .extract_string_with(props, "axis", bindings)
            .or_else(|| self.extract_string_with(props, "direction", bindings))
            .map(|kw| crate::ui::scroll::ScrollAxes::from_keyword(&kw))
            .unwrap_or(crate::ui::scroll::ScrollAxes::Y);
        let policy = self
            .extract_string_with(props, "scrollbar", bindings)
            .map(|kw| crate::ui::scroll::ScrollbarPolicy::from_keyword(&kw))
            .unwrap_or(crate::ui::scroll::ScrollbarPolicy::Auto);
        let controller = self
            .extract_string_with(props, "controller", bindings)
            .filter(|s| !s.is_empty())
            .map(crate::ui::view::ScrollControllerBinding::new);
        // on-scroll 观察面（v1 执行裁定：8 个位置实参，顺序=record 字段序
        // offset_x..progress_y——VM handler 桥的 push_value 对 heap 实参是
        // 占位 0，record 对象实参不可达；具名 record 经 scroll_state() native。
        // 双端契约：Vue 端同序位置实参，见 spec 兼容节）。
        let on_scroll = aura_events_get_base(events, "onscroll").map(|ev| {
            let handler = extract_handler_name(&ev.handler).to_string();
            let widget = self.widget_name.clone();
            crate::ui::view::ScrollCallback::new(move |m: crate::ui::view::ScrollMetrics| {
                let make_axis =
                    |offset: f32, viewport: f32, content: f32| crate::ui::scroll::ScrollAxisState {
                        offset: offset as f64,
                        viewport_extent: viewport as f64,
                        content_extent: content as f64,
                    };
                let x = make_axis(m.offset_x, m.viewport_w, m.content_w);
                let y = make_axis(m.offset_y, m.viewport_h, m.content_h);
                DynamicMessage::Typed {
                    widget_name: widget.clone(),
                    event_name: handler.clone(),
                    args: vec![
                        auto_val::Value::Float(x.offset),
                        auto_val::Value::Float(y.offset),
                        auto_val::Value::Float(x.viewport_extent),
                        auto_val::Value::Float(y.viewport_extent),
                        auto_val::Value::Float(x.content_extent),
                        auto_val::Value::Float(y.content_extent),
                        auto_val::Value::Float(crate::ui::scroll::geometry::progress(&x)),
                        auto_val::Value::Float(crate::ui::scroll::geometry::progress(&y)),
                    ],
                }
            })
        });
        (axes, policy, controller, on_scroll)
    }

    /// F-781-D01 收口（w01 勘定，2026-10-15）：scroll-pane 内层 col 继承
    /// pane 样式的**视觉类**（padding/border/bg/radius——Plan 409 §10 续 3
    /// 双写语义保留），但**滚动轴尺寸类必须剥除**：Fixed/百分比/min/max 高
    /// 落到内容 col 会把限高传导给整个子树（iced 限高→溢出行挤压为零高、
    /// content==viewport→pane 永不可滚动，锚点/offset/滚动条全失效）；Fill
    /// 高（h-full/flex-1）侥幸于滚动轴压缩语义取 intrinsic，剥除为幂等。
    /// vue 轨是单元素映射（overflow-* 直接上 pane 元素）无此双写面——剥除
    /// 即对齐 vue 行为。宽度类仅在水平/双轴 pane 剥除；竖直 pane 的
    /// w-full 内容铺宽依赖保留。
    pub(super) fn scroll_content_style(style: Style, axes: crate::ui::scroll::ScrollAxes) -> Style {
        use crate::ui::scroll::ScrollAxes;
        use crate::ui::style::StyleClass;
        let strip_h = axes.y;
        let strip_w = axes.x;
        let is_axis_sizing = |c: &StyleClass| match c {
            StyleClass::Height(_)
            | StyleClass::HeightPercent(_)
            | StyleClass::MinHeight(_)
            | StyleClass::MaxHeight(_) => strip_h,
            StyleClass::Width(_)
            | StyleClass::WidthPercent(_)
            | StyleClass::MinWidth(_)
            | StyleClass::MaxWidth(_) => strip_w,
            _ => false,
        };
        let mut style = style;
        style.classes.retain(|c| !is_axis_sizing(c));
        style.hover_classes.retain(|c| !is_axis_sizing(c));
        style.variant_classes.retain(|(_, c)| !is_axis_sizing(c));
        style
    }

    /// Tracked convert_scroll — mirrors `convert_scroll` but recurses via
    /// `convert_node_tracked_ctx` (path/probe tracking).
    pub(super) fn convert_scroll_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self
            .extract_u16(props, "spacing")
            .or_else(|| self.extract_u16(props, "gap").map(|g| g * 4))
            .unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // Plan 476: 拼接感知收集(源索引编号) + 视觉空后置过滤(既有行为)。
        let child_views: Vec<View<DynamicMessage>> = self
            .expand_children_spliced_source(children, path, id_map, probe, bindings)
            .into_iter()
            .filter(|v| !is_visually_empty(v))
            .collect();

        let mut builder = View::<DynamicMessage>::col()
            .spacing(spacing)
            .padding(padding);
        let scroll_style = style.clone();
        // PLAN-656 T-03: scroll-pane 语义提前——内层 col 的样式剥除需要轴信息。
        let (axes, scrollbar_policy, controller, on_scroll) =
            self.scroll_pane_semantics(props, events, bindings);
        if let Some(s) = style {
            builder = builder.with_style(Self::scroll_content_style(s, axes));
        }
        for child in child_views {
            builder = builder.child(child);
        }
        let col_view = builder.build();
        // Plan 057 续:`auto_scroll: "…"` 标记 → VM 端唯一挂 blocklist_scroll id
        // 的主列表(snap_to_end 目标);块内 max-h 滚动区不再共享同 id。
        let auto_scroll = self.extract_string(props, "auto_scroll").is_some();
        // PLAN-781 T-03: controller pane 的锚点归属后置注入（tracked 镜像）。
        let col_view = match controller.as_ref() {
            Some(c) => Self::inject_anchor_pane(col_view, c.key()),
            None => col_view,
        };
        View::Scrollable {
            child: Box::new(col_view),
            width: None,
            height: None,
            style: scroll_style,
            auto_scroll,
            offset: None,
            on_scroll,
            axes,
            scrollbar_policy,
            controller,
        }
    }

    /// Tracked convert_grid — mirrors `convert_grid` but recurses via
    /// `convert_node_tracked_ctx` so each grid-item cell captures its own
    /// path + probe data (text bindings, raw class). Probe paths follow the
    /// SOURCE structure (flat grid-item indices under the grid); the row
    /// grouping is a rendering detail and does not perturb probe indexing.
    pub(super) fn convert_grid_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let cols = self
            .extract_u16(props, "cols")
            .or_else(|| self.extract_u16(props, "columns"))
            .or_else(|| self.eval_u16_prop(props, "cols", bindings))
            .map(|c| (c as usize).max(1))
            .unwrap_or(1);
        let gap = self
            .extract_u16(props, "gap")
            .or_else(|| self.eval_u16_prop(props, "gap", bindings))
            .unwrap_or(0);
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // Plan 412 F1/F5: 语义 grid 元素的 class 里出现 grid-cols-N / gap-N 时
        // 类优先(与 CSS class 覆盖 attribute 的语义一致;gap 类是 Tailwind 单位
        // ×4 = 像素,gap prop 保持像素原值)。
        let (cols, gap) = match style.as_ref() {
            Some(s) => {
                let c = s
                    .classes
                    .iter()
                    .rev()
                    .find_map(|cl| match cl {
                        StyleClass::GridCols(n) => Some((*n as usize).max(1)),
                        _ => None,
                    })
                    .unwrap_or(cols);
                let g = s
                    .classes
                    .iter()
                    .rev()
                    .find_map(|cl| match cl {
                        StyleClass::Gap(size) => Some(size.to_pixels()),
                        _ => None,
                    })
                    .unwrap_or(gap);
                (c, g)
            }
            None => (cols, gap),
        };

        // Flatten `for`-loop children into individual cells, assigning each cell
        // a sequential `cell_idx` path so build-time paths match the render-time
        // paths `render_dynamic_view`'s Grid arm visits (Plan 323). A bare `for`
        // inside a grid must yield one cell per iteration, not a wrapping Column.
        let mut cells: Vec<View<DynamicMessage>> = Vec::new();
        let mut cell_idx: usize = 0;
        for n in children.iter() {
            match n {
                AuraNode::ForLoop {
                    var,
                    index,
                    iterable,
                    body,
                    ..
                } => {
                    let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
                    // Plan 046:裸标识符 iterable 可能是外层循环变量(嵌套 for),
                    // 先查 bindings(同主 ForLoop 修复)。未命中再 fallback 到 state。
                    let array: Vec<Value> = if let Some(val) = bindings.get(state_name).cloned() {
                        match val {
                            Value::Array(arr) => arr.iter().cloned().collect(),
                            Value::Int(id) if id >= 4_000_000 => {
                                self.bridge.index_list_all(id as usize)
                            }
                            Value::VmRef(r) => self.bridge.index_list_all(r.id),
                            _ => continue,
                        }
                    } else {
                        // Use resolve_iterable so heap-array refs (Value::Int(array_id),
                        // the form `var x = []; x.push(...)` produces — e.g. .days) are
                        // iterated AND state misses fall back to computed evaluation
                        // (PLAN-051 C3; Plan 522 016 迁移的 `for cell in .days` 以
                        // computed 为源)。Otherwise the grid's for renders empty.
                        match self.resolve_iterable(iterable, bindings) {
                            Some(v) => v,
                            None => continue,
                        }
                    };
                    let body_len = body.len();
                    for (i, item) in array.iter().enumerate() {
                        if !self.matches_search(item) {
                            continue;
                        }
                        let mut loop_bindings = bindings.clone();
                        loop_bindings.insert(var.clone(), self.bridge.materialize_obj_ref(item));
                        if let Some(idx_var) = index {
                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                        }
                        let views: Vec<View<DynamicMessage>> = body
                            .iter()
                            .enumerate()
                            .filter_map(|(bi, bn)| {
                                path.push(cell_idx);
                                if body_len > 1 {
                                    path.push(bi);
                                }
                                let for_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
                                probe.record_for(
                                    &for_path,
                                    ForIter {
                                        var: var.clone(),
                                        index: Some(i),
                                        value_repr: value_to_display_string(item),
                                        iterable_repr: iterable.clone(),
                                    },
                                );
                                let v = self.convert_node_tracked_ctx(
                                    bn,
                                    path,
                                    id_map,
                                    probe,
                                    &loop_bindings,
                                );
                                if body_len > 1 {
                                    path.pop();
                                }
                                path.pop();
                                Some(v)
                            })
                            .collect();
                        let cell = if views.is_empty() {
                            continue;
                        } else if views.len() == 1 {
                            views.into_iter().next().unwrap()
                        } else {
                            View::Column {
                                children: views,
                                spacing: 0,
                                padding: 0,
                                style: None,
                                onclick: None,
                                on_right_click: None,
                            }
                        };
                        if matches!(cell, View::Empty) {
                            continue;
                        }
                        cells.push(cell);
                        cell_idx += 1;
                    }
                }
                other => {
                    // Plan 476: 拼接感知——slot outlet 的 fill 子节点作兄弟 cell
                    // 展开(父作用域 tracked 转换;空 cell 不占格)。
                    for v in self
                        .expand_one_child_spliced(other, path, id_map, probe, bindings, cell_idx)
                    {
                        if matches!(v, View::Empty) {
                            continue;
                        }
                        cells.push(v);
                        cell_idx += 1;
                    }
                }
            }
        }

        if cells.is_empty() {
            return View::Empty;
        }

        // Decomposition (final-row padding + w-full rows + col-of-rows) moved
        // to the shared generic `build_grid` (Plan 319). Per-cell tracked
        // recursion is preserved, so cell i is still recorded at path [..i].
        // Bonus: build-time path [..i] now matches the render-time path that
        // `render_dynamic_view`'s Grid arm visits — previously the col-of-rows
        // split caused a build/render path mismatch for grid descendants.
        View::Grid {
            cols,
            gap,
            cells,
            style,
        }
    }

    /// Tracked convert_row — mirrors `convert_row`'s Conditional-flattening but
    /// recurses via tracked converters. (For Task 9 scope, only text bindings
    /// matter; the flattening is preserved for behavioural parity.)
    pub(super) fn convert_row_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self
            .extract_u16(props, "spacing")
            .or_else(|| self.extract_u16(props, "gap").map(|g| g * 4))
            .unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // D-GAP-4: children are recorded at their RESULTING slot (flattened
        // position), not the source child index. Two flattening steps diverge
        // from source indices: (a) an if/else body spliced into the row (a
        // 3-node body previously recorded all 3 nodes at the SAME [i] path,
        // merging their probe entries — the 013 todo rows showed one checkbox
        // carrying Toggle+StartEdit+Delete events and the wrong class);
        // (b) visually-empty views dropped from the built row shift later
        // siblings left. Numbering by surviving-slot keeps probe/VTree paths
        // identical by construction.
        let mut child_views: Vec<View<DynamicMessage>> = Vec::new();
        let mut slot: usize = 0;
        for n in children {
            match n {
                AuraNode::Conditional {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    // Flatten Conditional children: in a row, multiple condition
                    // children spread horizontally, not wrapped in a Column
                    // (mirrors untracked convert_row). Each spliced child converts
                    // at the CURRENT slot and advances it on survival — a 3-node
                    // body previously recorded all 3 at one path, merging their
                    // probe entries (013 todo rows: one checkbox carrying three
                    // events and the wrong class).
                    let is_true = self.eval_condition_with(condition, bindings);
                    let empty = Vec::new();
                    let body = if is_true {
                        then_body
                    } else {
                        else_body.as_ref().unwrap_or(&empty)
                    };
                    for child_node in body {
                        path.push(slot);
                        let v = self
                            .convert_node_tracked_ctx(child_node, path, id_map, probe, bindings);
                        path.pop();
                        if is_visually_empty(&v) {
                            continue;
                        }
                        child_views.push(v);
                        slot += 1;
                    }
                }
                AuraNode::ForLoop {
                    var,
                    index,
                    iterable,
                    body,
                    ..
                } => {
                    // Plan 047:flatten ForLoop into row(同 untracked convert_row)。
                    // 用 for_loop_iterations(放弃 row 内 ForLoop 的 probe 追踪,
                    // inspector 调试信息不影响渲染正确性)。Count survivors so
                    // later siblings' slots stay aligned.
                    for v in self.for_loop_iterations(var, index, iterable, body, bindings) {
                        if is_visually_empty(&v) {
                            continue;
                        }
                        child_views.push(v);
                        slot += 1;
                    }
                }
                _ => {
                    // Plan 476: 拼接感知——slot outlet 的 fill 子节点作兄弟展开
                    // (父作用域 tracked 转换,编号同幸存槽位约定)。
                    for v in self.expand_one_child_spliced(n, path, id_map, probe, bindings, slot) {
                        if is_visually_empty(&v) {
                            continue;
                        }
                        child_views.push(v);
                        slot += 1;
                    }
                }
            }
        }

        // PLAN-536 T6(题5 ①②): absolute 悬浮 hoist(tracked 主路径,row 臂
        // 与 convert_column_tracked_ctx 同批激活;语义定界见该处注释)。
        let mut floats: Vec<(View<DynamicMessage>, crate::ui::view::OverlayPosition)> = Vec::new();
        let child_views: Vec<View<DynamicMessage>> = child_views
            .into_iter()
            .filter_map(|v| match extract_absolute_position(&v) {
                Some(pos) => {
                    floats.push((v, pos));
                    None
                }
                None => Some(v),
            })
            .collect();

        // Plan 412 F1: 交叉类仲裁 — row 元素带 `grid grid-cols-N`/`flex-col` 类时
        // 布局语义被类覆盖。hoist 在前,浮层不占格。
        if let Some(RederivedLayout::Grid { cols, gap }) = rederive_layout(style.as_ref()) {
            let cells: Vec<View<DynamicMessage>> = child_views
                .into_iter()
                .filter(|v| !is_visually_empty(v))
                .collect();
            let base = View::Grid {
                cols,
                gap,
                cells,
                style,
            };
            return fold_floats(base, floats);
        }
        let use_col = matches!(
            rederive_layout(style.as_ref()),
            Some(RederivedLayout::Column)
        );

        let mut builder = if use_col {
            View::<DynamicMessage>::col()
        } else {
            View::<DynamicMessage>::row()
        }
        .spacing(spacing)
        .padding(padding);
        if let Some(s) = style {
            builder = builder.with_style(s);
        }
        for child in child_views {
            // Plan 370 (Issue 1): drop View::Empty spacers (see convert_column).
            if is_visually_empty(&child) {
                continue;
            }
            builder = builder.child(child);
        }
        let row_view = builder.build();
        // PLAN-536 T6: 有 absolute 子节点 → 包成 Overlay(浮在 base 上层)。
        fold_floats(row_view, floats)
    }

    /// Tracked convert_container — mirrors `convert_container`.
    /// Plan 412 F1: style 布局类重派生 — `grid grid-cols-N` → View::Grid、
    /// `flex-col` → View::Column、`flex` → View::Row(背景/边框/尺寸类留在
    /// style 上,由渲染层继续消费)。无布局类走原 Container 路径。
    pub(super) fn convert_container_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let width = self.extract_u16(props, "width");
        let height = self.extract_u16(props, "height");
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // Plan 476: 拼接感知收集(源索引编号) + 视觉空后置过滤(既有行为)。
        let child_views: Vec<View<DynamicMessage>> = self
            .expand_children_spliced_source(children, path, id_map, probe, bindings)
            .into_iter()
            .filter(|v| !is_visually_empty(v))
            .collect();

        match rederive_layout(style.as_ref()) {
            Some(RederivedLayout::Grid { cols, gap }) => {
                let mut style = style.unwrap_or_default();
                // Container 的 legacy prop(padding/width/height)搬进 style,
                // Grid 渲染走 apply_column_style 只读 style。
                if padding > 0
                    && !style
                        .classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Padding(_)))
                {
                    style = style.add(StyleClass::Padding(SizeValue::Fixed(padding)));
                }
                if let Some(w) = width {
                    if !style
                        .classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Width(_)))
                    {
                        style = style.add(StyleClass::Width(SizeValue::Pixels(w as f32)));
                    }
                }
                if let Some(h) = height {
                    if !style
                        .classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Height(_)))
                    {
                        style = style.add(StyleClass::Height(SizeValue::Pixels(h as f32)));
                    }
                }
                View::Grid {
                    cols,
                    gap,
                    cells: child_views,
                    style: Some(style),
                }
            }
            Some(RederivedLayout::Row) => View::Row {
                children: child_views,
                spacing: 0,
                padding,
                style,
                onclick: None,
                on_right_click: None,
            },
            Some(RederivedLayout::Column) => View::Column {
                children: child_views,
                spacing: 0,
                padding,
                style,
                onclick: None,
                on_right_click: None,
            },
            None => {
                let child_view = if child_views.is_empty() {
                    View::Empty
                } else if child_views.len() == 1 {
                    child_views.into_iter().next().unwrap()
                } else {
                    View::Column {
                        children: child_views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    }
                };
                let mut builder = View::container(child_view).padding(padding);
                if let Some(w) = width {
                    builder = builder.width(w);
                }
                if let Some(h) = height {
                    builder = builder.height(h);
                }
                if let Some(s) = style {
                    builder = builder.with_style(s);
                }
                builder.build()
            }
        }
    }

    /// Tracked convert_center — mirrors `convert_center`.
    pub(super) fn convert_center_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        let child_view = if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            path.push(0);
            let v = self.convert_node_tracked_ctx(&children[0], path, id_map, probe, bindings);
            path.pop();
            v
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    path.push(i);
                    let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
                    path.pop();
                    v
                })
                .collect();
            // Plan 448 续 (center parity): Vue's `center` compiles to
            // `flex flex-col items-center justify-center` — children are
            // EACH horizontally centered, not left-packed inside a shrink
            // column. iced Column defaults cross-axis children to start, so
            // a narrow text above a wider row sat left of the row's center
            // (002-counter VM vs Vue). Carrying ItemsCenter lets
            // build_column's align_items mapping center them like Vue.
            View::Column {
                children: views,
                spacing: 0,
                padding: 0,
                style: Some(Style::default().add(StyleClass::ItemsCenter)),
                onclick: None,
                on_right_click: None,
            }
        };

        let full_style = match style {
            Some(s) => s
                .add(StyleClass::Width(SizeValue::Full))
                .add(StyleClass::Height(SizeValue::Full)),
            None => Style::default()
                .add(StyleClass::Width(SizeValue::Full))
                .add(StyleClass::Height(SizeValue::Full)),
        };
        let mut builder = View::container(child_view).center_x().center_y();
        builder = builder.with_style(full_style);
        builder.build()
    }

    /// Tracked plain text node conversion. For an interpolated text node this
    /// records each `${.field}` binding at the current node's path. Literal text
    /// records nothing. The produced View is identical to `convert_text_with`.
    pub(super) fn convert_text_tracked_ctx(
        &self,
        content: &AuraTextContent,
        path: &mut Vec<usize>,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let resolved = match content {
            AuraTextContent::Literal(s) => s.clone(),
            AuraTextContent::Interpolated {
                template,
                bindings: tpl_bindings,
            } => self.resolve_interpolation_tracked(template, tpl_bindings, bindings, path, probe),
        };
        View::Text {
            content: resolved,
            style: None,
            selectable: false,
        }
    }

    /// Resolve an interpolation template AND record each binding into the probe
    /// at the current path. Returns the same resolved string as
    /// `resolve_interpolation_with`.
    pub(super) fn resolve_interpolation_tracked(
        &self,
        template: &str,
        tpl_bindings: &[String],
        loop_bindings: &Bindings,
        path: &mut Vec<usize>,
        probe: &mut BuildProbe,
    ) -> String {
        let mut result = resolve_i18n_template(template);
        // Only build the probe path when there is at least one binding to record;
        // an empty `tpl_bindings` needs no probe entries.
        if !tpl_bindings.is_empty() {
            let probe_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
            for field_name in tpl_bindings {
                let pattern = format!("${{{}}}", format!(".{}", field_name));
                let value_str = self.read_state_as_string_with(field_name, loop_bindings);
                // Record the state binding at the current node's path.
                probe.record_state(&probe_path, pattern.clone(), value_str.clone());
                result = result.replace(&pattern, &value_str);
            }
        }
        result
    }

    /// Tracked convert_text_element — mirrors `convert_text_element`'s content
    /// extraction but, when the content comes from an interpolated child text
    /// node, records the binding at the current (text element) path.
    pub(super) fn convert_text_element_tracked_ctx(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let probe_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
        let content = self
            .extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_string_with(props, "content", bindings))
            .or_else(|| self.extract_string_with(props, "label", bindings))
            .unwrap_or_else(|| {
                // Try to get content from child text nodes. For interpolated
                // children, also record each binding at this element's path.
                children
                    .iter()
                    .filter_map(|c| match c {
                        AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                        AuraNode::Text(AuraTextContent::Interpolated {
                            template,
                            bindings: tpl_bindings,
                        }) => {
                            // Record bindings for this child, attributed to the
                            // text element (current path) — consistent with the
                            // plain-text-node case.
                            for field_name in tpl_bindings {
                                let pattern = format!("${{{}}}", format!(".{}", field_name));
                                let value_str =
                                    self.read_state_as_string_with(field_name, bindings);
                                probe.record_state(&probe_path, pattern, value_str);
                            }
                            Some(self.resolve_interpolation_with(template, tpl_bindings, bindings))
                        }
                        // Plan 446 批五 U7(链4): 文本类子元素的内容折叠(绑定感知)。
                        // 此前只认 Text 节点——`label { text (text: m.label) {} }`
                        // 的子元素求值被跳过,宿主元素内容为空(os-config §P 现场
                        // "整个 label 缺位"根因)。
                        AuraNode::Element {
                            tag, props: cprops, ..
                        } if Self::TEXT_LIKE_TAGS.contains(&tag.as_str()) => {
                            self.child_element_text(cprops, bindings)
                        }
                        _ => None,
                    })
                    .collect::<Vec<String>>()
                    .join("")
            });

        // Plan 053 后续(ash-gui VM):绑定感知的样式求值。原 extract_style 只读
        // 静态字符串(class/style 的字面量),style: If 三元引用循环变量
        // (如 `cell.Tagged.kind == "Dir"`)时条件求值必然失败 → 永远落 else
        // 分支,表格按 kind 着色失效。条件只含 state 字段时(如 .in_continuation)
        // 无 bindings 也能求值,所以此前 prompt_symbol 的三元侥幸生效。
        let mut style = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .and_then(|s| Style::parse(&s).ok())
            .or_else(|| self.extract_style(props));

        // Apply default heading styles, merging with user-provided styles.
        // Plan 409 §8: headings carry the theme color (text-primary) so page
        // titles / section headers use the accent — "主要操作和显眼的内容".
        if matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
            let default = match tag {
                "h1" => Style::parse("text-4xl font-bold tracking-tight text-primary mb-4").ok(),
                "h2" => {
                    Style::parse("text-3xl font-bold tracking-tight text-primary mt-8 mb-4").ok()
                }
                "h3" => Style::parse("text-xl font-semibold text-primary mb-3").ok(),
                "h4" => Style::parse("text-lg font-semibold mb-2").ok(),
                "h5" => Style::parse("text-base font-semibold mb-1").ok(),
                "h6" => Style::parse("text-sm font-semibold mb-1").ok(),
                _ => None,
            };
            if let Some(mut default) = default {
                if let Some(user) = style.take() {
                    default.classes.extend(user.classes);
                }
                style = Some(default);
            }
        }

        // Heading styling is applied via the `style` field, not by transforming
        // `content`; matches untracked behaviour.
        //
        // If this text element has an onclick/click event, render it as a
        // Button so the click handler fires (View::Text has no onclick field).
        if let Some(event) = aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "click"))
        {
            let onclick = self.event_to_message_with(event, bindings);
            return View::Button {
                disabled: false,
                label: content,
                onclick,
                style,
                on_right_click: None,
                content: None,
            };
        }

        // Plan 481:opt-in selectable(v1 默认 false)。text/label 同臂。
        let selectable = self.extract_bool(props, "selectable").unwrap_or(false);

        View::Text {
            content,
            style,
            selectable,
        }
    }

    /// Convert an AuraNode::Element to a View variant based on the tag name.
    /// PLAN-050 T7 (C5): tag 是否为 `use.web component` 声明的外部组件名
    /// （musk 图标即此形态——ports/icons.at → lucide）。装载器经
    /// register_imported_components 注册 UI-owned 名单。
    pub(super) fn is_imported_component(&self, tag: &str) -> bool {
        imported_components_registry()
            .map(|set| set.contains(tag))
            .unwrap_or(false)
    }

    /// PLAN-050 T7 (C5): 图标组件 → View::Image{lucide:kebab}。size prop
    /// （lucide 惯例,px）映射 Width/Height 固定像素;renderer 端 glyph 缺失
    /// 时空占位（与既有 unknown-icon 行为一致）。
    /// PLAN-054 T4 (A11): `class` prop 下传——musk 会话卡 "N 条" 行
    /// `Info { size: 11, class: "text-muted-foreground shrink-0 ml-auto" }`
    /// 的 ml-auto/着色此前整串丢弃,图标紧跟文本而非贴行右端。
    /// PLAN-625 T-09(b): web-ecosystem 组件的 VM 降级占位卡——组件名 +
    /// Web 专属提示 + 尽力展示首个字符串型 prop 值（如 AppViewport 的
    /// `app: .selected_id`）。零交互、纯可读，Vue 臂不受影响。
    pub(super) fn convert_web_component_placeholder(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let mut lines = vec![format!("⚙ {tag}（Web 端组件）")];
        for key in ["app", "id", "name", "src", "value"] {
            if let Some(v) = self.extract_string_with(props, key, bindings) {
                if !v.is_empty() {
                    lines.push(format!("{key}: {v}"));
                }
            }
        }
        lines.push(
            "该内嵌容器为 Web 端组件，VM 端暂不内嵌示例画面；完整交互请使用 auto run（Vue 端）查看"
                .to_string(),
        );
        let mut children: Vec<View<DynamicMessage>> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let cls = if i == 0 {
                "text-xs font-mono text-muted-foreground"
            } else if i == lines.len() - 1 {
                "text-[11px] text-muted-foreground/80"
            } else {
                "text-[11px] font-mono text-muted-foreground/70"
            };
            children.push(View::Text {
                style: Style::parse(cls).ok(),
                content: line.clone(),
                selectable: false,
            });
        }
        View::Column {
            children,
            spacing: 4,
            padding: 0,
            style: Style::parse(
                "w-full rounded-lg border border-border/60 bg-muted/30 items-center justify-center gap-1 p-3",
            )
            .ok(),
            onclick: None,
            on_right_click: None,
        }
    }

    pub(super) fn convert_icon_component(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let size = self.extract_u16(props, "size").unwrap_or(16) as f32;
        let mut classes = vec![
            StyleClass::Width(SizeValue::Pixels(size)),
            StyleClass::Height(SizeValue::Pixels(size)),
        ];
        if let Some(user_class) = self.extract_string_with(props, "class", bindings) {
            if let Ok(user) = Style::parse(&user_class) {
                classes.extend(user.classes);
            }
        }
        let style = Style {
            classes,
            hover_classes: Vec::new(),
            variant_classes: Vec::new(),
        };
        View::Image {
            src: format!("lucide:{}", pascal_to_kebab_icon(tag)),
            style: Some(style),
        }
    }

    /// PLAN-043 T2：autodown 元素滚动/折叠事件面。返回
    /// (offset 写入臂, on_scroll 读出臂, Details 点击注入工厂)。
    /// - `scroll_sync` 求值真 → 文档外包 `View::Scrollable`：`scroll_top`
    ///   绑定求值进 offset（Int/Double/Float 均收），`onscroll` 事件进
    ///   on_scroll（六测量取三 top/height/client 编码为 args 随宿主通道
    ///   回 DSL handler）。
    /// - `ondetailsclick` 事件独立于 scroll_sync：summary 行 onclick 注入
    ///   Typed 消息（args = ["d<块结构键>"]，键 = block_key 内容哈希）。
    pub(super) fn autodown_scroll_binding(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
        editor_key: Option<&str>,
    ) -> (
        bool,
        Option<(f32, f32)>,
        Option<crate::ui::view::ScrollCallback<DynamicMessage>>,
        Option<Box<dyn Fn(u64) -> DynamicMessage>>,
    ) {
        let scroll_sync = props
            .get("scroll_sync")
            .and_then(|v| match v {
                AuraPropValue::Expr(expr) => self.resolve_expr_to_value(expr, bindings),
                _ => None,
            })
            .map(|val| val.as_bool())
            .unwrap_or(false);

        // Details 折叠注入不随 scroll_sync 门控（独立事件面）。
        let details_onclick = aura_events_get_base(events, "ondetailsclick").map(|ev| {
            let handler = extract_handler_name(&ev.handler).to_string();
            let widget = self.widget_name.clone();
            Box::new(move |key: u64| DynamicMessage::Typed {
                widget_name: widget.clone(),
                event_name: handler.clone(),
                args: vec![Value::str(format!("d{key}"))],
            }) as Box<dyn Fn(u64) -> DynamicMessage>
        });

        if !scroll_sync {
            return (false, None, None, details_onclick);
        }
        // PLAN-063 T-04d: sync_anchor——编辑壳锚块高亮开关。开启时 scroll
        // 回调把「首个完整可见块」索引写入编辑器 core（ade 存储），
        // DocEditor draw 臂读出描边高亮（仅 autodown+code-editor 双 feature
        // 下编辑器本体存在）。
        let sync_anchor = props
            .get("sync_anchor")
            .and_then(|v| match v {
                AuraPropValue::Expr(expr) => self.resolve_expr_to_value(expr, bindings),
                _ => None,
            })
            .map(|val| val.as_bool())
            .unwrap_or(false);
        let p063_sk = if sync_anchor {
            editor_key.map(|s| s.to_string())
        } else {
            None
        };
        // PLAN-063 T-04d-2: sink 注册与 sync_anchor 解耦——sync_anchor 是
        // 左栏编辑器的高亮开关；sync_anchor_target 属右栏 autodown 元素
        //（锚槽与 scroll_top 写臂所在）。有 target prop 即注册（字段名
        // 字符串字面量，剥前导点）。
        // PLAN-814 R9-03：anchor_slot 是 iced 布局期槽（ui-iced 门）——
        // headless 构建无同步目标（缺席=不注册）。
        if let Some(field) = self.extract_string_with(props, "sync_anchor_target", bindings) {
            eprintln!("[P063-SINK] registered");
            crate::ui::anchor_slot::set_target_sink(Some(
                field.trim_start_matches('.').to_string(),
            ));
        }
        let offset = props
            .get("scroll_top")
            .and_then(|v| match v {
                AuraPropValue::Expr(expr) => self.resolve_expr_to_value(expr, bindings),
                _ => None,
            })
            .and_then(|val| match val {
                Value::Float(f) => Some(f as f32),
                Value::Double(d) => Some(d as f32),
                Value::Int(i) => Some(i as f32),
                Value::Uint(u) => Some(u as f32),
                _ => None,
            })
            .map(|top| (0.0, top));
        let on_scroll = aura_events_get_base(events, "onscroll").map(|ev| {
            let handler = extract_handler_name(&ev.handler).to_string();
            let widget = self.widget_name.clone();
            crate::ui::view::ScrollCallback::new(move |m: crate::ui::view::ScrollMetrics| {
                #[cfg(feature = "autodown")]
                if let Some(sk) = &p063_sk {
                    if let Some(idx) = crate::ui::autodown_editor::core::set_anchor_from_scroll(
                        sk,
                        m.offset_y as f64,
                        m.viewport_h as f64,
                    ) {
                        eprintln!("[P063-T] pending={}", idx);
                        crate::ui::anchor_slot::set_pending_anchor(idx);
                    }
                }
                #[cfg(not(feature = "autodown"))]
                if let Some(_sk) = &p063_sk {
                    let _ = _sk;
                }
                DynamicMessage::Typed {
                    widget_name: widget.clone(),
                    event_name: handler.clone(),
                    // PLAN-043 T6：实参序 (height, client, top)。VM 轨由
                    // update 层 rust 直写快道消费（renderer.rs T6 拦截——
                    // 引擎 handler 对 float 实参/算术写入腐坏，DEBTS 登记；
                    // handler 保留为 vue 契约面）。
                    args: vec![
                        Value::Float(m.content_h as f64),
                        Value::Float(m.viewport_h as f64),
                        Value::Float(m.offset_y as f64),
                    ],
                }
            })
        });
        (true, offset, on_scroll, details_onclick)
    }

    /// PLAN-045 T5：表格列宽状态求值——`table_col_widths` prop
    /// （Map<表键, 列宽 px 列表>，键形 "t{block_key}" 与 OnColResize 消息
    /// payload 同源）→ render 入参 HashMap<u64, Vec<f32>>。坏键/坏值条目
    /// 跳过（不整体失败）；prop 缺席 = None（自然宽现状）。
    pub(super) fn autodown_table_widths(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<std::collections::HashMap<u64, Vec<f32>>> {
        let v = props.get("table_col_widths").and_then(|v| match v {
            AuraPropValue::Expr(expr) => self.resolve_expr_to_value(expr, bindings),
            _ => None,
        })?;
        let obj = match v {
            auto_val::Value::Obj(o) => o,
            _ => return None,
        };
        let mut m = std::collections::HashMap::new();
        for (k, val) in obj.iter() {
            let Some(u) = k
                .to_string()
                .trim()
                .strip_prefix('t')
                .and_then(|d| d.parse::<u64>().ok())
            else {
                continue;
            };
            let widths: Vec<f32> = match val {
                auto_val::Value::Array(a) => a
                    .iter()
                    .filter_map(|w| match w {
                        auto_val::Value::Float(f) => Some(*f as f32),
                        auto_val::Value::Double(d) => Some(*d as f32),
                        auto_val::Value::Int(i) => Some(*i as f32),
                        auto_val::Value::Uint(u) => Some(*u as f32),
                        _ => None,
                    })
                    .collect(),
                _ => continue,
            };
            m.insert(u, widths);
        }
        Some(m)
    }

    /// PLAN-045 T5：列宽落定消息通道——`oncolresize` 事件 → TableColResizeFn
    ///（Typed 消息 args = [Str "t{表键}", Int col, Float width]；043
    /// onscroll/044 onfocusblock 同族装配）。表键进闭包捕获
    ///（details_onclick 同款「键进闭包」通道）。
    pub(super) fn autodown_on_col_resize_binding(
        &self,
        events: &HashMap<String, AuraEvent>,
    ) -> Option<crate::ui::view::TableColResizeFn<DynamicMessage>> {
        aura_events_get_base(events, "oncolresize").map(|ev| {
            let handler = extract_handler_name(&ev.handler).to_string();
            let widget = self.widget_name.clone();
            std::sync::Arc::new(move |key: u64, m: crate::ui::view::ColResizeMetrics| {
                DynamicMessage::Typed {
                    widget_name: widget.clone(),
                    event_name: handler.clone(),
                    args: vec![
                        auto_val::Value::str(format!("t{key}")),
                        auto_val::Value::Int(m.col as i32),
                        auto_val::Value::Float(m.width as f64),
                    ],
                }
            }) as crate::ui::view::TableColResizeFn<DynamicMessage>
        })
    }

    /// PLAN-058 T12（转介④）：autodown 元素 class 的观感段提取——仅
    /// py-*/px-*/p-* 内边距 token（含 arbitrary 形态 py-[Npx]），供内容侧
    /// padded Container 消费。结构段 token（flex/min/overflow）不取（043
    /// T6 混合类挂 Scrollable 炸布局注记维持）。
    pub(super) fn autodown_padding_class(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> String {
        self.extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .unwrap_or_default()
            .split_whitespace()
            .filter(|t| t.starts_with("py-") || t.starts_with("px-") || t.starts_with("p-"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// PLAN-044 T4：autodown ghost 占位消费——`placeholder_block_id`/
    /// `placeholder_height` 双 prop 绑定求值（043 scroll_top 同机制：
    /// 数值 prop 直取、null/缺席即 None）。id 口径 = `block-${index}` 顶层
    /// 索引（vue data-block-id 同源）：字符串带 `block-` 前缀剥前缀、裸数字
    /// 串与 Int 直接收。仅 id 命中（非 None）即出 ghost；height 缺席置 0。
    pub(super) fn autodown_placeholder(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<(usize, f32)> {
        let num = |name: &str| {
            props.get(name).and_then(|v| match v {
                AuraPropValue::Expr(expr) => self.resolve_expr_to_value(expr, bindings),
                _ => None,
            })
        };
        let id = match num("placeholder_block_id")? {
            auto_val::Value::Str(s) => {
                let digits = s.trim().strip_prefix("block-").unwrap_or(s.trim());
                digits.parse::<usize>().ok()?
            }
            auto_val::Value::Int(i) if i >= 0 => i as usize,
            auto_val::Value::Uint(u) => u as usize,
            auto_val::Value::Float(f) if f >= 0.0 => f as usize,
            auto_val::Value::Double(d) if d >= 0.0 => d as usize,
            _ => return None,
        };
        let height = match num("placeholder_height") {
            Some(auto_val::Value::Float(f)) => f as f32,
            Some(auto_val::Value::Double(d)) => d as f32,
            Some(auto_val::Value::Int(i)) => i as f32,
            Some(auto_val::Value::Uint(u)) => u as f32,
            _ => 0.0,
        };
        Some((id, height))
    }

    /// PLAN-044 T4/T5：编辑壳块聚焦读出装配——`onfocusblock` 事件进
    /// View::AutodownEditor.on_focus（FocusCallback newtype，043 onscroll
    /// 同款宿主消息通道；事件名 = vue 轨 EngineEditor `focusblock` emit
    /// 的 codegen 直映 onX→@X，双轨单源）。FocusMetrics 载荷编码为 Typed
    /// 消息实参 (block_index, height)；失焦变体 block=None 编码 (Int -1,
    /// 0.0)。VM 轨由 update 层 rust 直写快道消费（043 T6 同拦截，引擎
    /// float 实参腐坏规避）；handler 保留为 vue 契约面。
    pub(super) fn autodown_on_focus_binding(
        &self,
        events: &HashMap<String, AuraEvent>,
    ) -> Option<crate::ui::view::FocusCallback<DynamicMessage>> {
        aura_events_get_base(events, "onfocusblock").map(|ev| {
            let handler = extract_handler_name(&ev.handler).to_string();
            let widget = self.widget_name.clone();
            crate::ui::view::FocusCallback::new(move |m: crate::ui::view::FocusMetrics| {
                DynamicMessage::Typed {
                    widget_name: widget.clone(),
                    event_name: handler.clone(),
                    args: vec![
                        auto_val::Value::Int(m.block.map(|b| b as i32).unwrap_or(-1)),
                        auto_val::Value::Float(m.height as f64),
                    ],
                }
            })
        })
    }

    /// PLAN-732：wikilink 激活读出装配——事件绑定双形态：引号完整名
    /// `on "open-wiki-link"`（Plan-367 引号式监听键 `on"open-wiki-link"`，
    /// split_aura_event_key 引号感知 base；jade Vue `on "open-wiki-link"`
    /// 同名，契约 C-01 主形态）与属性简写 `onopenwikilink`。载荷双 Str
    ///（target/target_anchor，核心侧已按首个 `#` 拆分 trim）；事件名 =
    /// handler 名（onfocusblock 同族 codegen 直映，如 `.OpenWikiLink` →
    /// "OpenWikiLink"）——派发侧 decode_payload 直达 VM handler 形参。
    pub(super) fn autodown_on_link_binding(
        &self,
        events: &HashMap<String, AuraEvent>,
    ) -> Option<crate::ui::view::LinkCallback<DynamicMessage>> {
        let ev = aura_events_get_base(events, "on\"open-wiki-link\"")
            .or_else(|| aura_events_get_base(events, "onopenwikilink"))?;
        let handler = extract_handler_name(&ev.handler).to_string();
        let widget = self.widget_name.clone();
        Some(crate::ui::view::LinkCallback::new(
            move |m: crate::ui::view::WikiLinkMetrics| DynamicMessage::Typed {
                widget_name: widget.clone(),
                event_name: handler.clone(),
                args: vec![
                    auto_val::Value::Str(m.target.into()),
                    auto_val::Value::Str(m.anchor.into()),
                ],
            },
        ))
    }

    /// PLAN-066 T2: `autodown_editor` 转换体——原两站点硬编码字符串臂
    /// （"autodown_editor" | "autodowneditor"，臂体逐字节相同）的公共提取，
    /// 经 NativeWidgetRegistry 派发（native_widget::global 注册；案 a
    /// sugar：内部仍产 View::AutodownEditor，快照 kind 与 vm-smoke 断言面
    /// 零改动）。门控同原臂；无 feature 时 global 注册 convert_textarea
    /// 降级（D-GAP-3 链保持）。
    #[cfg(feature = "autodown")]
    pub(crate) fn convert_autodown_editor_native(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let key = self
            .extract_string_with(props, "key", bindings)
            .or_else(|| self.extract_string_with(props, "id", bindings))
            .unwrap_or_else(|| "doc".to_owned());
        let value = self
            .extract_string_with(props, "content", bindings)
            .or_else(|| self.extract_string_with(props, "value", bindings))
            .unwrap_or_default();
        let is_final = props
            .get("final")
            .map(|v| match v {
                AuraPropValue::Expr(expr) => self
                    .resolve_expr_to_value(expr, bindings)
                    .map(|val| val.as_bool()),
                _ => None,
            })
            .flatten()
            .unwrap_or(true);
        let on_change = aura_events_get_base(events, "oninput")
            .or_else(|| aura_events_get_base(events, "input"))
            .or_else(|| aura_events_get_base(events, "onchange"))
            .or_else(|| aura_events_get_base(events, "change"))
            .map(|event| self.event_to_message(&event.handler));
        let style = self.extract_style_with(props, bindings);
        // PLAN-048 T7（W4）：placeholder 空态文案真消费——编辑壳
        // content 空且非聚焦时渲染浅灰占位（Plan 040「读取后
        // 忽略」豁免摘除）。
        let placeholder = self.extract_string_with(props, "placeholder", bindings);
        // PLAN-043 T3：scroll_sync 消费同 autodown 臂——编辑壳
        //（DocEditor 全内容高、外滚）外包 View::Scrollable，
        // 编辑栏 offset 绑定写入 + onscroll 消息读出。
        // PLAN-044 T4：onfocus 事件进 on_focus 读出臂（块聚焦
        // ghost 消息）。
        let p063_editor_sk = crate::ui::autodown_editor::storage_key(key.as_str());
        let (scroll_sync, offset, on_scroll, _details) =
            self.autodown_scroll_binding(props, events, bindings, Some(p063_editor_sk.as_str()));
        let on_focus = self.autodown_on_focus_binding(events);
        // PLAN-732：wikilink 激活读出（on "open-wiki-link" / onopenwikilink）。
        let on_link = self.autodown_on_link_binding(events);
        // PLAN-043 T6：包装层取纯 w-full h-full 合成样式——元素
        // class（flex-1/min-h-0/overflow-hidden 混合）直接挂
        // Scrollable 实测炸布局；Fill×Fill 视口约束 + 内层收缩
        // 到内容全高（外滚）。编辑壳自身样式保留在内层。
        if scroll_sync {
            return View::Scrollable {
                child: Box::new(View::AutodownEditor {
                    key,
                    value,
                    is_final,
                    on_change,
                    on_focus,
                    on_link,
                    placeholder,
                    style,
                }),
                width: None,
                height: None,
                style: Style::parse("w-full h-full").ok(),
                auto_scroll: false,
                offset,
                on_scroll,
                axes: crate::ui::scroll::ScrollAxes::Y,
                scrollbar_policy: crate::ui::scroll::ScrollbarPolicy::Auto,
                controller: None,
            };
        }
        View::AutodownEditor {
            key,
            value,
            is_final,
            on_change,
            on_focus,
            on_link,
            placeholder,
            style,
        }
    }

    /// PLAN-066: 原生外部组件注册表派发——内置臂穷尽后、.at AuraWidget 前查
    /// （原生优先语义保持；提案 066 §3.3 派发序）。registry 字段缺席回落进
    /// 程级 global（feature 门控注册；测试经 with_native_registry 注入自有
    /// 实例隔离）。View 工厂命中=直接产 View；Element 入口命中=产
    /// View::Custom（name+props 明文透传，排序保快照确定性）；工厂返回
    /// None=放弃接手，继续后续派发链。
    pub(super) fn native_widget_view(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> Option<View<DynamicMessage>> {
        let registry = match self.native_registry {
            Some(r) => r,
            None => crate::ui::native_widget::global(),
        };
        let entry = registry.lookup(tag)?;
        match entry {
            crate::ui::native_widget::NativeWidgetEntry::View(factory) => {
                let ctx = crate::ui::native_widget::NativeCtx {
                    builder: self,
                    tag,
                    props,
                    events,
                    children,
                    bindings,
                };
                factory(&ctx)
            }
            crate::ui::native_widget::NativeWidgetEntry::Element => {
                // Element 通道：Expr 属性经 resolve→display，StyleBinding 跳过
                // （样式走 style 面）。HashMap 迭代序不定——按键排序保快照
                // 确定性（MCP 断言面依赖）。
                let mut resolved: Vec<(String, String)> = props
                    .iter()
                    .filter_map(|(k, v)| match v {
                        AuraPropValue::Expr(expr) => self
                            .resolve_expr_to_value(expr, bindings)
                            .map(|val| (k.clone(), value_to_display_string(&val))),
                        AuraPropValue::StyleBinding(_) => None,
                    })
                    .collect();
                resolved.sort();
                let mut evs: Vec<(String, DynamicMessage)> = events
                    .iter()
                    .map(|(k, ev)| (k.clone(), self.event_to_message(&ev.handler)))
                    .collect();
                evs.sort_by(|a, b| a.0.cmp(&b.0));
                Some(View::Custom {
                    name: tag.to_string(),
                    props: resolved,
                    events: evs,
                    style: self.extract_style_with(props, bindings),
                })
            }
        }
    }

    /// PLAN-781 T-03: 后置注入 pane 归属——把本 pane 子树内**未限定**的
    /// ScrollAnchorMark 填上 controller key。遇嵌套 Scrollable 停（内层
    /// convert_scroll 已注入其自己的归属，外层不得覆盖）。
    pub(super) fn inject_anchor_pane(view: View<DynamicMessage>, pane_key: &str) -> View<DynamicMessage> {
        match view {
            View::ScrollAnchorMark { pane, key, child } => {
                let pane = if pane.is_empty() {
                    std::sync::Arc::from(pane_key)
                } else {
                    pane
                };
                View::ScrollAnchorMark {
                    pane,
                    key,
                    child: Box::new(Self::inject_anchor_pane(*child, pane_key)),
                }
            }
            View::Row {
                children,
                spacing,
                padding,
                style,
                onclick,
                on_right_click,
            } => View::Row {
                children: children
                    .into_iter()
                    .map(|c| Self::inject_anchor_pane(c, pane_key))
                    .collect(),
                spacing,
                padding,
                style,
                onclick,
                on_right_click,
            },
            View::Column {
                children,
                spacing,
                padding,
                style,
                onclick,
                on_right_click,
            } => View::Column {
                children: children
                    .into_iter()
                    .map(|c| Self::inject_anchor_pane(c, pane_key))
                    .collect(),
                spacing,
                padding,
                style,
                onclick,
                on_right_click,
            },
            // 嵌套 pane / 叶子：停。
            other => other,
        }
    }

    /// PLAN-781 T-01：`scroll-anchor: "<key>"` → 零布局锚点包装。
    /// 空 key/类型不可解析 = stderr 告警 + 原样返回（不失效页面）。
    pub(super) fn maybe_wrap_scroll_anchor(
        &self,
        view: View<DynamicMessage>,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        if !props.contains_key("scroll-anchor") {
            return view;
        }
        let key = self.extract_string_with(props, "scroll-anchor", bindings);
        match key {
            Some(k) if !k.is_empty() => View::ScrollAnchorMark {
                pane: std::sync::Arc::from(""),
                key: std::sync::Arc::from(k.as_str()),
                child: Box::new(view),
            },
            _ => {
                eprintln!("scroll-anchor: empty or unresolvable key ignored");
                view
            }
        }
    }

}
