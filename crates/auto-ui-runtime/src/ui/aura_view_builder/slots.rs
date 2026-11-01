//! PLAN-014 T-05：slot 责任（slot_fills_for/render_slot_outlet*/expand_children_spliced*/outlet 路由渲染）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {
    /// Plan 476: 组装 widget 调用位的 slot 填充集（children 无任何填充时
    /// 返回 None，走既有零变化路径）。parent 捕获 self（父作用域 builder），
    /// bindings 捕获调用位循环变量——fill 求值时切回二者即得父作用域。
    pub(super) fn slot_fills_for<'s>(
        &'s self,
        children: &'s [AuraNode],
        bindings: &'s Bindings,
    ) -> Option<SlotFills<'s>> {
        let fills = extract_slot_fills(children);
        if fills.is_empty() {
            None
        } else {
            Some(SlotFills {
                parent: self,
                fills,
                bindings,
            })
        }
    }

    /// Plan 476: 渲染子 widget 视图里的 slot outlet（非拼接上下文的直转路径，
    /// 即 outlet 不在五大容器 children 里的罕见位形）。命中调用位填充 → 切回
    /// 父作用域 builder 求值 fill 子节点（单子直通、多子 Column 兜底——拼接
    /// 感知容器 `expand_children_spliced` 会把多子作兄弟展开，不走这里）；
    /// 未命中 → 渲染 outlet 自身 children 作 fallback 内容（子作用域，对齐
    /// vue `generate_slot_outlet_html`）；双空 → Empty。
    pub(super) fn render_slot_outlet(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let name = auto_ui_compiler::aura::slot_element_name(props);
        if let Some(fills) = self.slot_fills.and_then(|f| f.fills.get(&name)) {
            let sf = self.slot_fills.unwrap();
            let views: Vec<View<DynamicMessage>> = fills
                .iter()
                .map(|n| sf.parent.convert_node_with(n, sf.bindings))
                .filter(|v| !is_visually_empty(v))
                .collect();
            return if views.is_empty() {
                View::Empty
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
        }
        // Fallback 内容（子作用域求值）
        if children.is_empty() {
            View::Empty
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .filter(|v| !is_visually_empty(v))
                .collect();
            if views.is_empty() {
                View::Empty
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
            }
        }
    }

    /// Plan 476: `render_slot_outlet` 的 tracked 双胎——fill 子节点经父作用域
    /// builder 的 tracked 转换继续 path/id_map/probe，编号按幸存槽位推进
    /// （D-GAP-4 RESULTING-slot 约定，同 convert_column_tracked_ctx）。
    pub(super) fn render_slot_outlet_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let name = auto_ui_compiler::aura::slot_element_name(props);
        let mut views: Vec<View<DynamicMessage>> = Vec::new();
        let mut slot_idx = 0usize;
        if let Some(fills) = self.slot_fills.and_then(|f| f.fills.get(&name)) {
            let sf = self.slot_fills.unwrap();
            for n in fills {
                path.push(slot_idx);
                let v = sf
                    .parent
                    .convert_node_tracked_ctx(n, path, id_map, probe, sf.bindings);
                path.pop();
                if is_visually_empty(&v) {
                    continue;
                }
                views.push(v);
                slot_idx += 1;
            }
        } else {
            for n in children {
                path.push(slot_idx);
                let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
                path.pop();
                if is_visually_empty(&v) {
                    continue;
                }
                views.push(v);
                slot_idx += 1;
            }
        }
        if views.is_empty() {
            View::Empty
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
        }
    }

    /// Plan 476: 拼接感知的 children 转换（Vue 布局精确语义）：slot outlet
    /// 命中 fill 时其各子节点作为**多个兄弟**视图产出（父作用域求值；布局
    /// 轴向由本容器决定），而非单包装；outlet 未命中 fill → 其 fallback
    /// children（子作用域）；普通节点 → 常规单视图。**不做视觉空过滤**——
    /// 各容器沿用既有过滤规则处置展开产物，非 outlet 路径零行为变化。
    pub(super) fn expand_children_spliced(
        &self,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> Vec<View<DynamicMessage>> {
        let mut views: Vec<View<DynamicMessage>> = Vec::new();
        for n in children {
            views.extend(self.expand_one_child_untracked(n, bindings));
        }
        views
    }

    /// Plan 476: 单个 child 节点的拼接展开（untracked）：outlet 命中 fill →
    /// 父作用域转换产出 0..N 视图；未命中 → fallback children（子作用域）；
    /// 普通节点 → 1 视图。返回未过滤的产出。
    pub(super) fn expand_one_child_untracked(
        &self,
        n: &AuraNode,
        bindings: &Bindings,
    ) -> Vec<View<DynamicMessage>> {
        if let Some((props, outlet_children)) = slot_outlet_parts(n) {
            let name = auto_ui_compiler::aura::slot_element_name(props);
            let mut produced: Vec<View<DynamicMessage>> = Vec::new();
            match self.slot_fills.and_then(|f| f.fills.get(&name)) {
                Some(fills) => {
                    let sf = self.slot_fills.unwrap();
                    for node in fills {
                        produced.push(sf.parent.convert_node_with(node, sf.bindings));
                    }
                }
                None => {
                    for node in outlet_children {
                        produced.push(self.convert_node_with(node, bindings));
                    }
                }
            }
            produced
        } else {
            vec![self.convert_node_with(n, bindings)]
        }
    }

    /// Plan 476: `expand_children_spliced` 的 tracked 双胎之一——**幸存槽位
    /// 编号**（D-GAP-4 RESULTING-slot 约定；convert_column/row_tracked_ctx
    /// 专用）：视觉空视图出列、不占槽位，后续子节点左移。
    pub(super) fn expand_children_spliced_resulting(
        &self,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> Vec<View<DynamicMessage>> {
        let mut views: Vec<View<DynamicMessage>> = Vec::new();
        let mut slot_idx = 0usize;
        for n in children {
            let produced =
                self.expand_one_child_spliced(n, path, id_map, probe, bindings, slot_idx);
            for v in produced {
                if is_visually_empty(&v) {
                    continue;
                }
                views.push(v);
                slot_idx += 1;
            }
        }
        views
    }

    /// Plan 476: `expand_children_spliced` 的 tracked 双胎之二——**源索引
    /// 编号**（convert_scroll/container/grid_tracked_ctx 既有约定：按原始
    /// children 下标记 path，视觉空后置过滤）。outlet 的多个展开子视图共享
    /// outlet 自身的源索引——多子填充时后写的 probe 条目覆盖先写的（快照
    /// 降级，vtree/渲染/事件不受影响），v1 已知限制登记 KNOWN-DEBT。
    pub(super) fn expand_children_spliced_source(
        &self,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> Vec<View<DynamicMessage>> {
        let mut views: Vec<View<DynamicMessage>> = Vec::new();
        for (i, n) in children.iter().enumerate() {
            let produced = self.expand_one_child_spliced(n, path, id_map, probe, bindings, i);
            views.extend(produced);
        }
        views
    }

    /// Plan 476: 单个 child 节点的拼接展开（tracked）：outlet 命中 fill →
    /// 父作用域 tracked 转换产出 0..N 视图（依次记 path 段 base_idx 起）；
    /// 未命中 → fallback children（子作用域）；普通节点 → 1 视图。
    /// 返回未过滤的产出（调用方按各自约定过滤/编号）。
    pub(super) fn expand_one_child_spliced(
        &self,
        n: &AuraNode,
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
        base_idx: usize,
    ) -> Vec<View<DynamicMessage>> {
        if let Some((props, outlet_children)) = slot_outlet_parts(n) {
            let name = auto_ui_compiler::aura::slot_element_name(props);
            let mut produced: Vec<View<DynamicMessage>> = Vec::new();
            match self.slot_fills.and_then(|f| f.fills.get(&name)) {
                Some(fills) => {
                    let sf = self.slot_fills.unwrap();
                    for node in fills {
                        path.push(base_idx);
                        let v = sf.parent.convert_node_tracked_ctx(
                            node,
                            path,
                            id_map,
                            probe,
                            sf.bindings,
                        );
                        path.pop();
                        produced.push(v);
                    }
                }
                None => {
                    for node in outlet_children {
                        path.push(base_idx);
                        let v = self.convert_node_tracked_ctx(node, path, id_map, probe, bindings);
                        path.pop();
                        produced.push(v);
                    }
                }
            }
            produced
        } else {
            path.push(base_idx);
            let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
            path.pop();
            vec![v]
        }
    }

}

impl<'a> AuraViewBuilder<'a> {
    /// Plan 401/VM-routing: render the page widget for the current route.
    ///
    /// Reads `__current_route` (e.g. "/book/1") from VM state, matches it
    /// against the routes table (honouring `:param` segments), and renders the
    /// matching page widget via `render_child_widget` — the iced equivalent of
    /// vue's `<router-view>`. Dynamic-segment values are written into the
    /// `__route_params` state object so page handlers can read them via
    /// `router.param("id")`. No routes / no match / empty route → `View::Empty`.
    pub(super) fn render_outlet(&self, bindings: &Bindings) -> View<DynamicMessage> {
        // PLAN-708 T-01：无语料 prop 的 fallback 入口 = prop 未设（三态表
        // Unset 臂 → outlet 缺省 on）。
        self.render_outlet_impl(bindings, None, None)
    }

    /// PLAN-045 T-05b：outlet 页产物 memo（opt-in：`AUTO_OUTLET_MEMO=1`；
    /// 裸 `outlet` 节点无 props 载体，语料级 prop 留待档 B 显式 memo 语法）。
    /// 命中路径重放挂载/路由簿记（产物无关面），Init 身份变化帧弃缓存全量
    /// 渲染（Init 派发语义逐字节保持）。
    pub(super) fn render_outlet_impl(
        &self,
        bindings: &Bindings,
        tracked: Option<(&mut Vec<usize>, &mut DebugIdMap, &mut BuildProbe)>,
        prop_memo: Option<bool>,
    ) -> View<DynamicMessage> {
        // PLAN-045 T-07 拆账：outlet 页构建耗时单列（AUTO_MEMO_DIAG 门）。
        let __diag = std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1");
        let __t0 = if __diag {
            Some(std::time::Instant::now())
        } else {
            None
        };
        let __out = self.render_outlet_inner(bindings, tracked, prop_memo);
        if let (true, Some(t0)) = (__diag, __t0) {
            eprintln!(
                "[VM-VIEW-OUTLET] outlet_ms={}",
                (std::time::Instant::now() - t0).as_millis()
            );
        }
        __out
    }

    pub(super) fn render_outlet_inner(
        &self,
        bindings: &Bindings,
        tracked: Option<(&mut Vec<usize>, &mut DebugIdMap, &mut BuildProbe)>,
        prop_memo: Option<bool>,
    ) -> View<DynamicMessage> {
        let (Some(registry), Some(routes)) = (self.widget_registry, self.routes) else {
            return View::Empty;
        };

        // Current route path. Default to the first route (the index "/") when
        // unset, so the app boots into its home page without an explicit init.
        let current = match self.read_state("__current_route") {
            Ok(auto_val::Value::Str(s)) if !s.is_empty() => s.to_string(),
            _ => routes.first().map(|r| r.path.clone()).unwrap_or_default(),
        };
        // PLAN-712 T-17 第三层：本 outlet 渲染覆盖到的所有页 Init 登记，其
        // 代际身份并入当前路由路径（guard 于本函数任一返回路径清零）。
        let _outlet_identity_guard = OutletInitIdentityGuard::set(&current);

        // Match the current path against each route pattern. A pattern segment
        // starting with ':' matches any single path segment and is captured as
        // a route param (e.g. "/book/:id" matches "/book/3" → id=3).
        let cur_segs: Vec<&str> = current
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        for route in routes {
            let pat_segs: Vec<&str> = route
                .path
                .trim_matches('/')
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();
            if pat_segs.len() != cur_segs.len() {
                continue;
            }
            let mut params: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            let mut matched = true;
            for (pat, cur) in pat_segs.iter().zip(cur_segs.iter()) {
                if let Some(param_name) = pat.strip_prefix(':') {
                    params.insert(param_name.to_string(), cur.to_string());
                } else if *pat != *cur {
                    matched = false;
                    break;
                }
            }
            if !matched {
                continue;
            }
            // Found the route. (Param persistence into __route_params is handled
            // by the navigation codegen — render time is read-only here.)
            let _ = params;
            // Plan 408: look up the page widget via the route module alias map
            // (route.module → widget.name, e.g. "button" → "ButtonPage").
            // Using the alias map instead of registry.get avoids shadowing
            // built-in UI elements like <button>, <input>.
            if let Some(page_widget) = registry.get_by_route_module(&route.module) {
                let empty_props: HashMap<String, AuraPropValue> = HashMap::new();
                let empty_events: HashMap<String, AuraEvent> = HashMap::new();
                // Plan 476: 页面路由无调用位填充,outlet 页面不带 slot fills。
                // PLAN-708 T-01（S-01）三态表（优先级冻结，r2 §5）：
                //   env=0 → 强制关（诊断逃生门，覆盖显式 true）
                //   env=1 → 强制开（兼容层保留）
                //   未设/其他值 → prop：未设=on（r2 缺省面变更）/false=off/true=on
                // 046 并集契约变化点：显式 `memo:false` × env=1 现为 on
                //（原 off）——测试矩阵 plan708_memo_tristate 全覆盖。
                let memo_on = Self::resolve_outlet_memo(Self::memo_env_gate(), prop_memo);
                if !memo_on {
                    return match tracked {
                        Some((path, id_map, probe)) => self.render_child_widget_tracked(
                            page_widget,
                            &empty_props,
                            &empty_events,
                            bindings,
                            path,
                            id_map,
                            probe,
                            None,
                        ),
                        None => self.render_child_widget(
                            page_widget,
                            &empty_props,
                            &empty_events,
                            bindings,
                            None,
                        ),
                    };
                }
                return self.render_outlet_page_memo(
                    page_widget,
                    &empty_props,
                    &empty_events,
                    bindings,
                    tracked,
                );
            }
            // Keep a failed declared page distinguishable from an unmatched route.
            return View::Text {
                content: registry.route_failure(&route.module).map(str::to_owned).unwrap_or_else(|| {
                    format!("VM_ROUTE_PAGE_LOAD_ERROR: module={} stage=registration: declared route has no registered page widget", route.module)
                }),
                style: None,
                selectable: false,
            };
        }
        // No route matched the current path.
        View::Text {
            content: format!("<outlet: no route for {}>", current),
            style: None,
            selectable: false,
        }
    }

    /// PLAN-045 T-05b：outlet 页产物 memo 门（AUTO_OUTLET_MEMO=1 opt-in）。
    /// 键 = (ctx, SITE_OUTLET_PAGE, 页模板+组件模板骨架指纹, probe_on)；命中
    /// 条件同 SD-02（seq 快速路径 / 读槽值指纹慢路径，读槽含组件模板内状态
    /// 读）。命中帧重放挂载/回调路由簿记；Init 身份变化帧弃缓存（Init 派发
    /// 语义保持）。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_outlet_page_memo(
        &self,
        page_widget: &auto_ui_compiler::aura::AuraWidget,
        empty_props: &HashMap<String, AuraPropValue>,
        empty_events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
        mut tracked: Option<(&mut Vec<usize>, &mut DebugIdMap, &mut BuildProbe)>,
    ) -> View<DynamicMessage> {
        use crate::ui::memo_deps::{
            globals_fingerprint, scan_static_with_components, GlobalEpisode, MemoEntry, MemoKey,
            ScanVerdict,
        };
        const SITE_OUTLET_PAGE: u8 = 6;

        // 循环守卫（同 render_child_widget 首检）。
        if self
            .active_child_widgets
            .borrow()
            .contains(&page_widget.name)
        {
            return View::Empty;
        }
        // PLAN-711 T-03 (M-01): Init 登记化——渲染路径只登记 demand（判定
        // ≠完成，派发由泵驱动）；身份变化帧仍走全量 build（memo 旁路保留：
        // 新页内容必须真构建，Init 完成后的内容刷新由泵侧 dirty 传播承担）。
        let has_init = page_widget.lifecycle.iter().any(|l| l.name == "Init");
        if has_init {
            let identity = self.child_init_identity(page_widget, empty_props, bindings);
            if self
                .bridge
                .init_identity_changed(&page_widget.name, &identity)
            {
                let (child_state_id, init_props) =
                    self.prepare_child_render_state_snap(page_widget, empty_props, bindings);
                let _decision = self.bridge.register_init_demand(
                    &page_widget.name,
                    &identity,
                    child_state_id,
                    init_props,
                );
                return self.render_outlet_page_full(
                    page_widget,
                    empty_props,
                    empty_events,
                    bindings,
                    tracked,
                );
            }
        }
        // 簿记重放（产物无关面——非命中帧由 render_child_widget 内部做同款）。
        if let Some(sink) = self.mounted_sink {
            sink.borrow_mut().insert(page_widget.name.clone());
        }
        if let Some(psink) = self.mount_path_sink {
            psink.register(&page_widget.name);
        }
        Self::record_child_callback_routes_for(
            self.widget_name.clone(),
            page_widget.name.clone(),
            empty_props,
            empty_events,
        );
        let _child_state_id = self.prepare_child_render_state(page_widget, empty_props, bindings);

        // 静态扫描（含组件模板读槽）+ 骨架键。
        let probe_on = tracked
            .as_ref()
            .map(|(_, _, p)| p.is_enabled())
            .unwrap_or(false);
        let page_slice = std::slice::from_ref(&page_widget.view_tree);
        let slots = match scan_static_with_components(empty_props, page_slice, self.widget_registry)
        {
            ScanVerdict::Slots(sl) => sl,
            ScanVerdict::Degrade(reason) => {
                self.bridge.with_memo_cache(|c| c.note_degraded());
                // PLAN-708 T-01：Degrade 不再静默——诊断门下输出原因
                // （DataTable 每帧全量重建 9.5s 的可观测性缺口，T-00
                // baseline §3.3 实录；诊断面正式采集归 T-06）。
                if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[MEMO-DIAG] site=6 DEGRADE page={} reason={}",
                        page_widget.name, reason
                    );
                }
                return self.render_outlet_page_full(
                    page_widget,
                    empty_props,
                    empty_events,
                    bindings,
                    tracked,
                );
            }
        };
        let key = MemoKey {
            ctx_state_obj: self.memo_ctx_obj(),
            site: SITE_OUTLET_PAGE,
            skeleton_fp: crate::ui::memo_deps::skeleton_fingerprint_with_components(
                empty_props,
                page_slice,
                self.widget_registry,
            ),
            probe_on,
            item_key: None,
            ui_epoch: crate::ui::memo_deps::ui_epoch(),
        };
        let seq = self.bridge.state_mutation_seq();
        let gfp = {
            let ep = GlobalEpisode {
                theme_epoch: crate::ui::style::theme::theme_epoch(),
                menubar_open: crate::ui::action_config::menubar_open(),
                popover_open: crate::ui::action_config::popover_open(),
                action_config_ptr: crate::ui::action_config::action_config()
                    .map(|a| std::sync::Arc::as_ptr(&a) as usize)
                    .unwrap_or(0),
            };
            globals_fingerprint(&ep)
        };
        let cached = self.bridge.with_memo_cache(|c| {
            c.lookup(&key).map(|e| {
                (
                    e.seq_at_fill,
                    e.globals_fp,
                    e.dyn_fp,
                    e.product.clone(),
                    e.probe_replay.clone(),
                    e.idmap_replay.clone(),
                    e.dyn_deps.clone(),
                )
            })
        });
        if let Some((
            seq_at_fill,
            gfp_entry,
            dyn_fp,
            product,
            probe_replay,
            idmap_replay,
            dyn_deps,
        )) = cached
        {
            if seq_at_fill == seq && gfp_entry == gfp {
                self.bridge.with_memo_cache(|c| {
                    c.note_hit();
                    c.note_seq_fast(Some(SITE_OUTLET_PAGE));
                });
                if let Some((_, id_map, probe)) = tracked.as_mut() {
                    probe.merge_entries(probe_replay);
                    id_map.merge_entries(idmap_replay);
                }
                if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                    eprintln!("[MEMO-DIAG] site=6 HIT fast page={}", page_widget.name);
                }
                return product;
            }
            if gfp_entry == gfp {
                // PLAN-047 T-05（档 C SD-11）: version_fast——零重解析命中面。
                if let Some(pairs) = &dyn_deps {
                    if self.bridge.deps_unchanged(pairs) {
                        self.bridge.with_memo_cache(|c| {
                            c.refresh_seq(&key, seq);
                            c.note_hit();
                            c.note_version_fast(Some(SITE_OUTLET_PAGE));
                        });
                        if let Some((_, id_map, probe)) = tracked.as_mut() {
                            probe.merge_entries(probe_replay);
                            id_map.merge_entries(idmap_replay);
                        }
                        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                            eprintln!(
                                "[MEMO-DIAG] site=6 HIT version-fast page={}",
                                page_widget.name
                            );
                        }
                        return product;
                    }
                }
                let cur = self
                    .memo_slots_fp(&slots, bindings)
                    .map(|fp| Self::memo_combine_dyn(fp, None));
                if matches!((cur, dyn_fp), (Some(a), Some(b)) if a == b) {
                    let refreshed = dyn_deps.as_ref().map(|pairs| {
                        pairs
                            .iter()
                            .map(|(k, _)| {
                                let v = self.bridge.vm().path_version(k.heap_id, &k.path);
                                (k.clone(), v)
                            })
                            .collect()
                    });
                    self.bridge.with_memo_cache(|c| {
                        c.refresh_seq(&key, seq);
                        if let Some(pairs) = refreshed {
                            c.refresh_deps(&key, pairs);
                        }
                        c.note_hit();
                        c.note_fp_slow(Some(SITE_OUTLET_PAGE));
                    });
                    if let Some((_, id_map, probe)) = tracked.as_mut() {
                        probe.merge_entries(probe_replay);
                        id_map.merge_entries(idmap_replay);
                    }
                    if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                        eprintln!("[MEMO-DIAG] site=6 HIT slow page={}", page_widget.name);
                    }
                    return product;
                }
            }
            self.bridge.with_memo_cache(|c| c.note_miss());
            if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                eprintln!("[MEMO-DIAG] site=6 MISS page={}", page_widget.name);
            }
        } else if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
            eprintln!("[MEMO-DIAG] site=6 FILL page={}", page_widget.name);
        }
        // fill：全量渲染 + 入缓存。PLAN-047 T-05: 求值期依赖录制（桥通道 +
        // 引擎读臂双通道）——录制集进条目 version_fast 面；**槽表达式估值
        // 同在录制域内**（静态扫描面含未走分支的槽——其读通道必须入集，
        // 否则 version_fast 漏失效面=陈旧风险）。
        let reborrowed = tracked
            .as_mut()
            .map(|(path, id_map, probe)| (&mut **path, &mut **id_map, &mut **probe));
        let ((out, slots_fp), rec) = self.fill_with_recording(|| {
            let out = self.render_outlet_page_full(
                page_widget,
                empty_props,
                empty_events,
                bindings,
                reborrowed,
            );
            let slots_fp = self.memo_slots_fp(&slots, bindings);
            (out, slots_fp)
        });
        let dyn_deps = self.rec_to_dyn_deps(&rec);
        if let Some(slots_fp) = slots_fp {
            let reborrowed = tracked
                .as_mut()
                .map(|(path, id_map, probe)| (&mut **path, &mut **id_map, &mut **probe));
            let (probe_snap, idmap_snap) = match reborrowed {
                Some((path, id_map, probe)) if probe.is_enabled() => {
                    let u16b: Vec<u16> = path.iter().map(|&x| x as u16).collect();
                    (probe.snapshot_prefix(&u16b), id_map.snapshot_prefix(path))
                }
                _ => (Vec::new(), Vec::new()),
            };
            let entry = MemoEntry {
                seq_at_fill: self.bridge.state_mutation_seq(),
                globals_fp: gfp,
                read_exprs: slots,
                dyn_fp: Some(Self::memo_combine_dyn(slots_fp, None)),
                product: out.clone(),
                probe_replay: probe_snap,
                idmap_replay: idmap_snap,
                replay_relative: false,
                dyn_deps,
            };
            self.bridge.with_memo_cache(|c| c.insert(key, entry));
        } else {
            self.bridge.with_memo_cache(|c| c.note_degraded());
        }
        out
    }

    pub(super) fn render_outlet_page_full(
        &self,
        page_widget: &auto_ui_compiler::aura::AuraWidget,
        empty_props: &HashMap<String, AuraPropValue>,
        empty_events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
        tracked: Option<(&mut Vec<usize>, &mut DebugIdMap, &mut BuildProbe)>,
    ) -> View<DynamicMessage> {
        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
            eprintln!(
                "[MEMO-DIAG] page_full enter page={} tracked={}",
                page_widget.name,
                tracked.is_some()
            );
        }
        match tracked {
            Some((path, id_map, probe)) => self.render_child_widget_tracked(
                page_widget,
                empty_props,
                empty_events,
                bindings,
                path,
                id_map,
                probe,
                None,
            ),
            None => {
                self.render_child_widget(page_widget, empty_props, empty_events, bindings, None)
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Plan 482: nav-item / nav-group / nav(search:) — the VM side of the nav
    // contract. All layout classes come from `ui::nav_contract` (single source
    // shared with the NavItem.vue/NavGroup.vue scaffold; a unit test locks the
    // two together). nav-item reuses View::Button (content subtree + hover
    // classes), so the iced renderer needs no new arm.
    // ─────────────────────────────────────────────────────────────────────

}
