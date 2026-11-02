//! PLAN-014 T-05：子组件渲染责任（prepare_child_render_state*/fire_child_init*/child_init_identity/render_child_widget*）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

use super::route_memo::count_keyed_loops_free;
impl<'a> AuraViewBuilder<'a> {
    /// Steps shared by the tracked/untracked child-widget render paths:
    /// resolve props from parent state (1), collect child state field names
    /// (2), sync matching parent state → child (3), seed model-var defaults
    /// (4), and ensure the child state object exists on the VM heap (5).
    /// Returns the child's heap state-object id for `override_state_obj_id`.
    pub(super) fn prepare_child_render_state(
        &self,
        child_widget: &auto_ui_compiler::aura::AuraWidget,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> u64 {
        self.prepare_child_render_state_snap(child_widget, props, bindings)
            .0
    }

    /// PLAN-711 T-03: 同 [`Self::prepare_child_render_state`]，另返回本次
    /// resolve 的 props 快照——Init demand 登记携带（统一根态约束：延迟
    /// 派发前按快照重播种，见 `InitDemand.props` 注）。
    pub(super) fn prepare_child_render_state_snap(
        &self,
        child_widget: &auto_ui_compiler::aura::AuraWidget,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> (u64, Vec<(String, auto_val::Value)>) {
        // 1. Resolve prop values from parent state.
        let mut resolved_props: HashMap<String, Value> = HashMap::new();
        for (prop_name, prop_value) in props {
            if let AuraPropValue::Expr(expr) = prop_value {
                if let Some(val) = self.resolve_expr_to_value(expr, bindings) {
                    resolved_props.insert(prop_name.clone(), val);
                }
            }
        }

        // 2. Collect child state field names (model vars + injected props).
        let mut child_field_names: Vec<String> = child_widget
            .state_vars
            .iter()
            .map(|v| v.name.clone())
            .collect();
        for prop_name in resolved_props.keys() {
            if !child_field_names.contains(prop_name) {
                child_field_names.push(prop_name.clone());
            }
        }

        // 3. Also sync matching parent state → child (editing, edit_title, etc.)
        //    so child handlers can read/write shared parent state.
        for field_name in &child_field_names {
            if !resolved_props.contains_key(field_name) {
                if let Ok(parent_val) = self.bridge.read_state(field_name) {
                    resolved_props.insert(field_name.clone(), parent_val);
                }
            }
        }

        // 4. Seed child model-var defaults (e.g. `var collapsed bool = false`)
        //    that the parent didn't pass. Plan 049: without this, a child view
        //    that reads its own model var (`if !.collapsed`, a computed like
        //    `collapse_glyph`) hits read_state Err on first render → the
        //    condition short-circuits to false (body hidden) and the string
        //    falls back to "${collapsed}" / "${collapse_glyph}".
        //    Only runs when the parent didn't provide the prop (parent wins).
        //    PLAN-536 T8(题6 根修): 缺省播种只在根态**缺字段**时发生——此前
        //    每帧无条件重播种,子件 handler 对本件模型字段的写入(受控 open
        //    翻转等)在下一帧重建时被打回默认值:musk delete_confirm_open=true
        //    而视图恒无 modal 的实录机制。首次挂载播种语义不变。
        for state_var in &child_widget.state_vars {
            if !resolved_props.contains_key(&state_var.name) {
                resolved_props.insert(
                    state_var.name.clone(),
                    eval_initial_without_vm(&state_var.initial),
                );
            }
        }

        // 4b. Plan 437 Phase 2: prop 声明默认值播种 —— `(index: str = "m",
        // colors: List<str> = [...])` 一类 widget-parens 默认值此前只在
        // vue 轨生效(withDefaults),VM 轨不播种 → 子组件 handler 读未传
        // prop 时 GET_FIELD 直接失败(donut 的 colors 即此)。父级显式传值
        // 优先(parent wins)。PLAN-536 T8: 同 4 的缺字段守卫——已存在的
        // 字段不重播种(防 handler 写入被逐帧打回默认)。
        // PLAN-711 Q-05: 被守卫跳过的声明默认值另录 snapshot_defaults——
        // 构建**不写**（536-T8 纪律原样），但进入 Init demand 快照：派发
        // 重播种时把兄弟实例遗留的同名字段恢复回本实例声明语义（bar#1
        // type 残留 bar#2 "stacked" 实录——快照缺 type 使派发 Init 走错
        // 分支，yTick 全错档）。
        let mut snapshot_defaults: Vec<(String, auto_val::Value)> = Vec::new();
        for prop in &child_widget.props {
            if !resolved_props.contains_key(&prop.name) {
                if self.bridge.read_state(&prop.name).is_ok() {
                    if let Some(default_expr) = &prop.default {
                        snapshot_defaults
                            .push((prop.name.clone(), eval_initial_without_vm(default_expr)));
                    }
                    continue;
                }
                if let Some(default_expr) = &prop.default {
                    resolved_props.insert(prop.name.clone(), eval_initial_without_vm(default_expr));
                }
            }
        }

        // 5. Ensure child state object exists on the VM heap + write props.
        let root_id =
            self.bridge
                .ensure_child_state(&child_widget.name, &child_field_names, &resolved_props);
        let mut snapshot = resolved_props
            .into_iter()
            .collect::<Vec<(String, auto_val::Value)>>();
        snapshot.extend(snapshot_defaults);
        (root_id, snapshot)
    }

    /// Render a child widget by looking it up in the registry.
    ///
    /// This resolves props from parent state, injects them as state fields,
    /// creates a child VmBridge, and recursively renders the child's view tree.
    /// Plan 320: render a child widget WITHOUT creating a new VM.
    /// Uses the same VmBridge (same VM), creates/updates the child's state
    /// object on the heap, and renders the child's view tree with an
    /// override_state_obj_id so read_state reads from the child's state.
    /// Plan 437 Phase 2: 子组件声明了 Init handler 时,在 props 播种后于
    /// 渲染期补发一次(每渲染帧重放)。handler 缺失(HandlerNotFound)静默
    /// —— namespaced 导出不存在 = 该子组件没有 Init,常态;其余错误记
    /// warn 不中断渲染。
    pub(super) fn fire_child_init_if_any(
        &self,
        child_widget: &auto_ui_compiler::aura::AuraWidget,
        state_obj_id: u64,
        init_props: Vec<(String, auto_val::Value)>,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) {
        // Init 在提取期被移入 lifecycle 向量(extract.rs),不在 handlers 表。
        let has_init = child_widget.lifecycle.iter().any(|l| l.name == "Init");
        if !has_init {
            return;
        }
        // PLAN-536 T3(题2 收敛): Init 收敛为挂载语义——本组件生命周期内每
        // 子件只派发一次,不随脏重建帧重放。437 v1 的逐帧重放使副作用型
        // Init(ForgeStore.Init→LoadSessionList)随帧重入(musk 单会话期
        // 1.6 万+次实录);props 仍每帧重播种(ensure_child_state),派生值
        // 响应沿 vue 语义归 watch/computed。
        // os-config 016: 挂载身份 = 组件名 + 调用点 `key:` prop(可解析时)
        // ——对齐 vue 的按 key 重挂载语义:同名子件 key 变化(含切回先前
        // key)即重发 Init,否则子件继续渲染上一个 key 的数据体。无 key
        // 调用点身份即组件名,每帧不变,536 防重放语义不变。
        let init_identity = self.child_init_identity(child_widget, props, bindings);
        // PLAN-711 T-03 (M-01): 只登记——同代际重复 build 不二次入队（AC-04）；
        // 派发由泵驱动（FIFO 依赖序：页 demand 在前，本件等前序到终态）。
        let _decision = self.bridge.register_init_demand(
            &child_widget.name,
            &init_identity,
            state_obj_id,
            init_props,
        );
    }

    /// PLAN-045 T-05b：子件挂载身份单源（组件名 + 调用位 `key:` prop）。
    pub(super) fn child_init_identity(
        &self,
        child_widget: &auto_ui_compiler::aura::AuraWidget,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> String {
        let base = self
            .extract_string_with(props, "key", bindings)
            .filter(|k| !k.is_empty())
            .map(|k| format!("{}#{}", child_widget.name, k))
            .unwrap_or_else(|| child_widget.name.clone());
        // PLAN-712 T-17 第三层：outlet 页渲染期并入路由身份（见
        // OUTLET_INIT_IDENTITY 文档）——参数变化 = 新代际 = Init 重派。
        OUTLET_INIT_IDENTITY.with(|c| match c.borrow().as_deref() {
            Some(route) => format!("{}@{}", base, route),
            None => base,
        })
    }

    /// PLAN-051 C2: 把 Component 调用位上的回调绑定（`onsend: .SendInput($event)`
    /// / `on_send: .SendMessage` 形态）记入 child_emit 路由表。实测该绑定落
    /// Component 节点的 **events**（AuraEvent 已解析形态）；props 路径（Plan 345
    /// 非 native on* 分流的 Expr 原样形态）作兜底双扫。DOM-native 键（onclick/
    /// oninput/…）不是 emit 回调，跳过。键按父侧声明原样（"onsend"/"on_send"/…），
    /// 派发侧以 "on"+子 msg 名（声明式）或剥离调用名（体内式）查表。每帧重建
    /// 重放，同键后写覆盖。
    pub(super) fn record_child_callback_routes_for(
        parent_widget: String,
        child_name: String,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
    ) {
        if std::env::var("AUTO_DEBUG_EMIT").is_ok() {
            eprintln!(
                "[VM-EMIT-REG] child={} events={:?} on-props={:?}",
                child_name,
                events.keys().collect::<Vec<_>>(),
                props
                    .keys()
                    .filter(|k| k.starts_with("on"))
                    .collect::<Vec<_>>()
            );
        }
        for (key, ev) in events.iter() {
            if auto_ui_compiler::aura::extract::is_native_event_key(key) {
                continue;
            }
            // auto-down PLAN-051 T11 补面：Plan-367 引号式监听落 events 的键
            // 是原样 `on"SetTheme"` 形（内嵌引号，CustomScrollbar 的
            // `on"update:scrollTop"` 同形）——原样键查表 miss（musk C2 双样
            // 点只有 onsend/on_send 属性式）。剥引号归一 + 无 on 前缀补前缀，
            // 与派发侧 "on"+子 msg 名两侧折叠命中。
            let cleaned: String = key.replace('"', "");
            let folded_key = if cleaned.starts_with("on") {
                cleaned
            } else {
                format!("on{}", cleaned)
            };
            crate::ui::child_emit::record_route(
                &child_name,
                &folded_key,
                crate::ui::child_emit::ParentRoute {
                    parent_widget: parent_widget.clone(),
                    handler: ev.handler.trim_start_matches('.').to_string(),
                    params: ev.params.clone(),
                },
            );
        }
        for (key, pv) in props.iter() {
            if !key.starts_with("on") {
                continue;
            }
            let AuraPropValue::Expr(expr) = pv else {
                continue;
            };
            let Some((handler, params)) = callback_prop_route(expr) else {
                continue;
            };
            crate::ui::child_emit::record_route(
                &child_name,
                key,
                crate::ui::child_emit::ParentRoute {
                    parent_widget: parent_widget.clone(),
                    handler,
                    params,
                },
            );
        }
    }

    pub(super) fn render_child_widget(
        &self,
        child_widget: &auto_ui_compiler::aura::AuraWidget,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
        slot_fills: Option<&SlotFills>,
    ) -> View<DynamicMessage> {
        // os-007（P530-D1）：环守卫——见 active_child_widgets 字段注记。
        let cycling = self
            .active_child_widgets
            .borrow()
            .contains(&child_widget.name);

        if cycling {
            return View::Empty;
        }
        // PLAN-652: 实例化即登记 mounted（条件臂命中路径）。
        if let Some(sink) = self.mounted_sink {
            sink.borrow_mut().insert(child_widget.name.clone());
        }
        // PLAN-654: path 级登记（帧内序号自增）。
        if let Some(psink) = self.mount_path_sink {
            psink.register(&child_widget.name);
        }
        Self::record_child_callback_routes_for(
            self.widget_name.clone(),
            child_widget.name.clone(),
            props,
            events,
        );
        let (child_state_id, init_props) =
            self.prepare_child_render_state_snap(child_widget, props, bindings);
        // Plan 437 Phase 2: 子组件 Init 补发 —— 此前 VM 轨只有根 widget 的
        // Init 会触发(fire_init),视图中实例化的子组件 Init 从不运行,
        // 派生计算型组件(chart 几何)在 VM 轨无渲染期计算路径(vue 轨
        // onMounted 正常,跨轨语义缺口)。统一 state 架构下逐实例顺序
        // props → Init → build,每个渲染帧重放:纯派生 Init 幂等;副作用
        // 型子组件 Init 会在每次脏重建时重放(v1 近似,债务在案)。
        self.fire_child_init_if_any(child_widget, child_state_id, init_props, props, bindings);
        // PLAN-711 T-05 (M-04/Q-01): pending demand → 轻量骨架占位。Init
        // 完成（epoch 失效→重建全量）前不渲染子树产物——pending 占位不成
        // 为可永久命中的 memo 产物（完成 bump epoch 已失效）；嵌套需求按
        // 依赖阶段发现：子件嵌套 child 由完成帧的下一次构建发现（占位不
        // 永久早退）。失败相位给可诊断错误占位（不无限 Loading；reload
        // 换身份即重试）。无 demand/已终态原样全量构建。
        match self.bridge.init_demand_phase(&child_widget.name) {
            Some(crate::ui::vm_bridge::InitDemandPhase::Queued)
            | Some(crate::ui::vm_bridge::InitDemandPhase::InFlight) => {
                return View::Text {
                    content: format!("Loading… ({})", child_widget.name),
                    style: None,
                    selectable: false,
                };
            }
            Some(crate::ui::vm_bridge::InitDemandPhase::Failed) => {
                return View::Text {
                    content: format!(
                        "⚠ {}.Init failed — see [VM-HANDLER] log (reload to retry)",
                        child_widget.name
                    ),
                    style: None,
                    selectable: false,
                };
            }
            _ => {}
        }

        // Build a child view builder using the SAME bridge but with
        // override_state_obj_id pointing to the child's state object.
        // Plan 476: slot_fills 透传——子视图的 outlet 命中时切回父作用域
        // (fills.parent) 求值填充,父 state/computed/事件路由随之生效。
        let child_builder = AuraViewBuilder {
            bridge: self.bridge,
            widget_name: child_widget.name.clone(),
            widget_registry: self.widget_registry,
            native_registry: self.native_registry,
            override_state_obj_id: Some(child_state_id),
            routes: None,
            computed: Some(&child_widget.computed),
            preview_states: self.preview_states,
            nav_group_states: self.nav_group_states,
            nav_memo_armed: std::cell::Cell::new(false),
            slot_fills,
            active_child_widgets: {
                let mut active = self.active_child_widgets.borrow().clone();
                active.insert(child_widget.name.clone());
                RefCell::new(active)
            },
            mounted_sink: self.mounted_sink,
            mount_path_sink: self.mount_path_sink,
            // PLAN-031 T-03：子组件实例借父作用域快照作为局部设计基线。
            design_frames: RefCell::new(self.design_frames.borrow().clone()),
        };

        child_builder.build(&child_widget.view_tree)
    }

    /// D-GAP-4 (013-todo audit B12(iii)): tracked counterpart of
    /// [`render_child_widget`]. The plain variant builds the child via the
    /// UNTRACKED converter, so the BuildProbe never records the child
    /// subtree — its style/event bindings vanish from the MCP snapshot
    /// (the subtree renders and handlers DO dispatch — live-verified
    /// `.TodoList.StartEdit` with `editing_id: -1 → 0` — but agents see no
    /// `onclick:` lines and cannot locate/click them). Continue the SAME
    /// path/id_map/probe into the child: its nodes occupy the flattened
    /// View positions under the component call, so probe entries land on
    /// exactly the VNodeIds the vtree snapshot assigns.
    pub(super) fn render_child_widget_tracked(
        &self,
        child_widget: &auto_ui_compiler::aura::AuraWidget,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        slot_fills: Option<&SlotFills>,
    ) -> View<DynamicMessage> {
        // os-007（P530-D1）：环守卫——见 active_child_widgets 字段注记。
        let cycling = self
            .active_child_widgets
            .borrow()
            .contains(&child_widget.name);
        if cycling {
            return View::Empty;
        }
        // PLAN-652: 实例化即登记 mounted（tracked 双胎同款）。
        if let Some(sink) = self.mounted_sink {
            sink.borrow_mut().insert(child_widget.name.clone());
        }
        // PLAN-654: path 级登记（tracked 双胎同款）。
        if let Some(psink) = self.mount_path_sink {
            psink.register(&child_widget.name);
        }
        Self::record_child_callback_routes_for(
            self.widget_name.clone(),
            child_widget.name.clone(),
            props,
            events,
        );
        // PLAN-814 R9-03：count_keyed_loops_free 在 ui-iced 门内——诊断
        // 输出随同门（headless 构建无 iced 布局诊断面）。
        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
            eprintln!(
                "[MEMO-DIAG] child_widget={} keyed_loops={}",
                child_widget.name,
                count_keyed_loops_free(&child_widget.view_tree)
            );
        }

        let (child_state_id, init_props) =
            self.prepare_child_render_state_snap(child_widget, props, bindings);
        // Plan 437 Phase 2: 同 render_child_widget —— 子组件 Init 补发
        // (tracked 双胎保持同一渲染语义)。
        self.fire_child_init_if_any(child_widget, child_state_id, init_props, props, bindings);
        // PLAN-711 T-05 (M-04/Q-01): pending demand → 轻量骨架占位。Init
        // 完成（epoch 失效→重建全量）前不渲染子树产物——pending 占位不成
        // 为可永久命中的 memo 产物（完成 bump epoch 已失效）；嵌套需求按
        // 依赖阶段发现：子件嵌套 child 由完成帧的下一次构建发现（占位不
        // 永久早退）。失败相位给可诊断错误占位（不无限 Loading；reload
        // 换身份即重试）。无 demand/已终态原样全量构建。
        match self.bridge.init_demand_phase(&child_widget.name) {
            Some(crate::ui::vm_bridge::InitDemandPhase::Queued)
            | Some(crate::ui::vm_bridge::InitDemandPhase::InFlight) => {
                return View::Text {
                    content: format!("Loading… ({})", child_widget.name),
                    style: None,
                    selectable: false,
                };
            }
            Some(crate::ui::vm_bridge::InitDemandPhase::Failed) => {
                return View::Text {
                    content: format!(
                        "⚠ {}.Init failed — see [VM-HANDLER] log (reload to retry)",
                        child_widget.name
                    ),
                    style: None,
                    selectable: false,
                };
            }
            _ => {}
        }

        // Plan 476: slot_fills 透传(untracked 双胎同款语义)。
        let child_builder = AuraViewBuilder {
            bridge: self.bridge,
            widget_name: child_widget.name.clone(),
            widget_registry: self.widget_registry,
            native_registry: self.native_registry,
            override_state_obj_id: Some(child_state_id),
            routes: None,
            computed: Some(&child_widget.computed),
            preview_states: self.preview_states,
            nav_group_states: self.nav_group_states,
            nav_memo_armed: std::cell::Cell::new(false),
            slot_fills,
            active_child_widgets: {
                let mut active = self.active_child_widgets.borrow().clone();
                active.insert(child_widget.name.clone());
                RefCell::new(active)
            },
            mounted_sink: self.mounted_sink,
            mount_path_sink: self.mount_path_sink,
            // PLAN-031 T-03：子组件实例借父作用域快照作为局部设计基线。
            design_frames: RefCell::new(self.design_frames.borrow().clone()),
        };

        child_builder.convert_node_tracked_ctx(
            &child_widget.view_tree,
            path,
            id_map,
            probe,
            bindings,
        )
    }
}
