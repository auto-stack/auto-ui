//! PLAN-014 T-05：面板类标签责任（menubar/menu/modal/side_panel/hovercard/popover/toolbar/avatar）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

use super::route_memo::MemoGateBegin;
impl<'a> AuraViewBuilder<'a> {
    /// Plan 414 §6: `sep` — inline separator widget (toolbar/menu group
    /// gaps). Orientation-aware, theme-aware 1px hairline; user classes
    /// append after the base (later classes win).
    /// Plan 418 P2-3: synthesize the menubar from the action config
    /// (`menubar {}` DSL tag). Open state lives in action_config::MENUBAR_OPEN
    /// (renderer-side local UI state, preview-card pattern); toggle buttons
    /// emit `__menubar_toggle(id)` internal messages handled in iced update.
    /// Plan 422 P2: 面板改挂在触发按钮的 Popover 上(iced overlay 机制定位,
    /// BottomStart = 左缘对齐),估位偏移与 2000px catch 一并退役 —— 按钮文
    /// 字任意宽度不再错位,点击捕获由 overlay 语义承担。popover 结构每帧
    /// 恒定(open 驱动面板显隐),view diff 稳定。
    /// PLAN-630 T-01: shared menu-item presentation — the actions-DSL
    /// synthesis AND the declarative menubar component both lower to this:
    /// [leading slot (check/action icon/blank) + title] packed left |
    /// [shortcut] packed right (right-aligned text). 626 T-02 的
    /// justify-start/text-left 让位与 629 T-06 的两组布局在此收口。
    /// PLAN-630 T-01: shared menu-item presentation — the actions-DSL
    /// synthesis AND the declarative menubar component both lower to this:
    /// [leading slot (check/action icon/blank) + title] packed left |
    /// [shortcut] packed right (right-aligned text). 626 T-02 的
    /// justify-start/text-left 让位与 629 T-06 的两组布局在此收口。
    /// PLAN-695 T-04：配色主题化根修——字面 zinc-200/300/500 → popover
    /// 语义 token（浅色主题可读）；disabled 置灰（muted 前景 + 整项
    /// opacity-50，对齐 shadcn `data-[disabled]:opacity-50`）；onclick
    /// 改 Option——None 渲染置灰静态行（此前整项隐藏，"不发声的菜单项
    /// 莫名消失"）。PLAN-695 T-07：leading_icon 参数化——checkbox 勾选
    /// lucide:check / radio 选中 lucide:circle-dot（shadcn 指示器语义）。
    pub(super) fn menu_item_button_view(
        &self,
        title: &str,
        icon: Option<String>,
        shortcut: Option<String>,
        checked: bool,
        enabled: bool,
        onclick: Option<DynamicMessage>,
        leading_icon: &str,
    ) -> View<DynamicMessage> {
        // 置灰通道：显式 disabled 或无 handler（不可激活 → muted 视觉，
        // a2r 降层同语义——PLAN-695 T-04 三路一致）。
        let interactive = enabled && onclick.is_some();
        let (fg, item_opacity) = if interactive {
            ("text-popover-foreground", "")
        } else {
            ("text-muted-foreground", " opacity-50")
        };
        let leading: View<DynamicMessage> = if checked {
            View::Image {
                src: leading_icon.to_string(),
                style: Style::parse(&format!("w-4 h-4 {fg} shrink-0")).ok(),
            }
        } else if let Some(icon) = icon.filter(|i| !i.is_empty()) {
            View::Image {
                src: format!("lucide:{icon}"),
                style: Style::parse(&format!("w-4 h-4 {fg} shrink-0")).ok(),
            }
        } else {
            View::Text {
                content: String::new(),
                style: Style::parse("w-4 h-4 shrink-0").ok(),
                selectable: false,
            }
        };
        let left_group = View::Row {
            children: vec![
                leading,
                View::Text {
                    content: title.to_string(),
                    style: Style::parse(&format!("text-[12px] {fg}")).ok(),
                    selectable: false,
                },
            ],
            spacing: 0,
            padding: 0,
            style: Style::parse("items-center gap-2").ok(),
            onclick: None,
            on_right_click: None,
        };
        let shortcut_text = View::Text {
            content: shortcut.unwrap_or_default(),
            style: Style::parse(&format!(
                "text-[11px] text-muted-foreground w-14 text-right{item_opacity}"
            ))
            .ok(),
            selectable: false,
        };
        // hover:bg-accent = shadcn `focus:bg-accent` 的 iced 对位（iced 无
        // menu 高亮协议，按钮 Status::Hovered 即悬停反馈）；accent 双盘
        // token 随主题/深浅自解。disabled 走置灰通道，不挂 hover。
        // rounded-sm 静止态零视觉（radius 无 border 不上色），仅 hover 块圆角。
        let row_style = Style::parse(&format!(
            "h-7 w-full px-0 py-0 justify-start text-left rounded-sm{item_opacity}"
        ))
        .ok();
        let Some(onclick) = onclick else {
            // 无 handler：置灰静态行（不可点、可读——不再隐藏）。
            return View::Row {
                children: vec![left_group, shortcut_text],
                spacing: 0,
                padding: 0,
                style: row_style,
                onclick: None,
                on_right_click: None,
            };
        };
        View::Button {
            disabled: !enabled,
            label: title.to_string(),
            onclick,
            // PLAN-626 T-02: 显式 justify-start/text-left 走 plan050→plan414
            // 让位通道压过按钮 content 容器的 Center 默认。
            style: Style::parse(&format!(
                "h-7 w-full px-0 py-0 justify-start text-left rounded-sm hover:bg-accent{item_opacity}"
            ))
            .ok(),
            on_right_click: None,
            content: Some(Box::new(View::Row {
                children: vec![left_group, shortcut_text],
                spacing: 0,
                padding: 0,
                style: Style::parse("w-full justify-between items-center gap-2 px-2").ok(),
                onclick: None,
                on_right_click: None,
            })),
        }
    }

    /// PLAN-695 T-06/T-07: 菜单面板子项统一构建——item / checkbox-item /
    /// radio-group / radio-item / label / shortcut / separator / sub
    ///（submenu 嵌套）/ group 全 kind 在此消费；声明式主面板与 submenu
    /// 面板共用（recursion 经此自洽）。返回 (views, probe 事件表)——事件
    /// 路径相对 content 根（调用方叠 [popover_idx, 1] 前缀记录）。
    /// 开合通道沿用 `__MenubarToggle`：submenu 复合键
    /// `<menu_id>::sub-<idx>` 进同一 `MENUBAR_OPEN` 注册表，无新消息变体
    ///（menubar-snapshot 规则 2/3 兼容——`__` 前缀不触发自动关闭判据）。
    pub(super) fn build_menu_panel_items(
        &self,
        nodes: &[AuraNode],
        bindings: &Bindings,
        open_key_prefix: &str,
        open: &Option<String>,
        depth: usize,
        radio_value: &str,
    ) -> (Vec<View<DynamicMessage>>, Vec<(Vec<usize>, String)>) {
        const MAX_SUB_DEPTH: usize = 4;
        let node_tag = |n: &AuraNode| -> Option<String> {
            match n {
                AuraNode::Element { tag, .. } => Some(tag.clone()),
                _ => None,
            }
        };
        let node_text = |n: &AuraNode| -> Option<String> {
            match n {
                AuraNode::Text(auto_ui_compiler::aura::AuraTextContent::Literal(s)) => {
                    Some(s.clone())
                }
                _ => None,
            }
        };
        let mut items: Vec<View<DynamicMessage>> = Vec::new();
        let mut events: Vec<(Vec<usize>, String)> = Vec::new();
        for (item_idx, item_node) in nodes.iter().enumerate() {
            let Some(itag) = node_tag(item_node) else {
                continue;
            };
            let itag = itag.replace('_', "-");
            let Some((iprops, ievents, ikids)) = (match item_node {
                AuraNode::Element {
                    props,
                    events,
                    children,
                    ..
                } => Some((props, events, children)),
                _ => None,
            }) else {
                continue;
            };
            match itag.as_str() {
                "menubar-separator" => {
                    let mut sep_props = HashMap::new();
                    sep_props.insert(
                        "orientation".to_string(),
                        AuraPropValue::Expr(Expr::Str("horizontal".into())),
                    );
                    items.push(self.convert_sep(&sep_props, bindings));
                }
                "menubar-item" | "menubar-checkbox-item" => {
                    let is_checkbox = itag == "menubar-checkbox-item";
                    let title = self
                        .extract_string_with(iprops, "title", bindings)
                        .or_else(|| ikids.iter().find_map(node_text))
                        .unwrap_or_default();
                    let icon = self.extract_string_with(iprops, "icon", bindings);
                    let shortcut = self.extract_string_with(iprops, "shortcut", bindings);
                    let checked = if is_checkbox {
                        self.extract_string_with(iprops, "checked", bindings)
                            .map(|c| self.eval_condition_with(&c, bindings))
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    // PLAN-695 T-04：disabled 置灰——enabled 表达式与
                    // 静态 disabled 字面量双通道（置灰非隐藏）。
                    let enabled = self
                        .extract_string_with(iprops, "enabled", bindings)
                        .map(|e| self.eval_condition_with(&e, bindings))
                        .unwrap_or(true)
                        && !matches!(
                            iprops.get("disabled"),
                            Some(AuraPropValue::Expr(Expr::Bool(true)))
                        );
                    let onclick = ievents
                        .get("onclick")
                        .or_else(|| aura_events_get_base(ievents, "onclick"))
                        .map(|ev| self.event_to_message_with(ev, bindings));
                    events.push((vec![item_idx], format!("__menubar_item({:?})", title)));
                    items.push(self.menu_item_button_view(
                        &title,
                        icon,
                        shortcut,
                        checked,
                        enabled,
                        onclick,
                        "lucide:check",
                    ));
                }
                // PLAN-695 T-07: 分组标签——muted 小字非可点（修复消费方
                // "行尾" 标签被静默吞）。
                "menubar-label" => {
                    let title = self
                        .extract_string_with(iprops, "text", bindings)
                        .or_else(|| ikids.iter().find_map(node_text))
                        .unwrap_or_default();
                    items.push(View::Text {
                        content: title,
                        style: Style::parse(
                            "h-6 w-full px-2 text-[11px] text-muted-foreground text-left",
                        )
                        .ok(),
                        selectable: false,
                    });
                }
                // PLAN-695 T-07: 独立快捷键子元素——右对齐 muted 行。
                "menubar-shortcut" => {
                    let text = self
                        .extract_string_with(iprops, "text", bindings)
                        .or_else(|| ikids.iter().find_map(node_text))
                        .unwrap_or_default();
                    items.push(View::Text {
                        content: text,
                        style: Style::parse(
                            "h-6 w-full pr-2 text-[11px] text-muted-foreground text-right",
                        )
                        .ok(),
                        selectable: false,
                    });
                }
                // PLAN-695 T-07: radio 族——组值绑定（state_ref 经 bindings
                // 解析）下传子项作选中判定；onclick 回写 store（用户
                // handler 责任，与 shadcn update:value 等价面）。
                "menubar-radio-group" => {
                    let group_value = self
                        .extract_string_with(iprops, "value", bindings)
                        .unwrap_or_default();
                    let (radio_items, radio_events) = self.build_menu_panel_items(
                        ikids,
                        bindings,
                        open_key_prefix,
                        open,
                        depth,
                        &group_value,
                    );
                    events.extend(radio_events.into_iter().map(|(p, h)| {
                        let mut full = vec![item_idx];
                        full.extend(p);
                        (full, h)
                    }));
                    items.push(View::Column {
                        children: radio_items,
                        spacing: 0,
                        padding: 0,
                        style: Style::parse("w-full").ok(),
                        onclick: None,
                        on_right_click: None,
                    });
                }
                "menubar-radio-item" => {
                    let value = self
                        .extract_string_with(iprops, "value", bindings)
                        .unwrap_or_default();
                    let title = self
                        .extract_string_with(iprops, "title", bindings)
                        .or_else(|| ikids.iter().find_map(node_text))
                        .unwrap_or_default();
                    let icon = self.extract_string_with(iprops, "icon", bindings);
                    let shortcut = self.extract_string_with(iprops, "shortcut", bindings);
                    let enabled = self
                        .extract_string_with(iprops, "enabled", bindings)
                        .map(|e| self.eval_condition_with(&e, bindings))
                        .unwrap_or(true)
                        && !matches!(
                            iprops.get("disabled"),
                            Some(AuraPropValue::Expr(Expr::Bool(true)))
                        );
                    let onclick = ievents
                        .get("onclick")
                        .or_else(|| aura_events_get_base(ievents, "onclick"))
                        .map(|ev| self.event_to_message_with(ev, bindings));
                    // 选中判定：外层 radio-group 绑定值（参数下传；脱组项
                    // 空串 = 按未选中渲染）。
                    let selected = !radio_value.is_empty() && radio_value == value;
                    events.push((
                        vec![item_idx],
                        format!("__menubar_radio({:?}={:?})", title, value),
                    ));
                    items.push(self.menu_item_button_view(
                        &title,
                        icon,
                        shortcut,
                        selected,
                        enabled,
                        onclick,
                        "lucide:circle-dot",
                    ));
                }
                // PLAN-695 T-06: submenu 三件套——sub-trigger 行（chevron
                // 右置）+ 嵌套 Popover（RightTop 顶对齐右弹，面板样式同
                // token）。开合键 = `<前缀>::sub-<idx>` 复合键；外点层级
                // 关闭走 on_dismiss = toggle(sub_id)（只关本层）。
                "menubar-sub" if depth < MAX_SUB_DEPTH => {
                    let sub_id = format!("{}::sub-{}", open_key_prefix, item_idx);
                    let sub_open = open.as_deref() == Some(sub_id.as_str());
                    let mut sub_title = String::new();
                    let mut sub_content: &[AuraNode] = &[];
                    for sk in ikids {
                        if let AuraNode::Element {
                            tag: stag,
                            props: sprops,
                            children: skids,
                            ..
                        } = sk
                        {
                            match stag.replace('_', "-").as_str() {
                                "menubar-sub-trigger" => {
                                    sub_title = self
                                        .extract_string_with(sprops, "text", bindings)
                                        .or_else(|| skids.iter().find_map(node_text))
                                        .unwrap_or_default();
                                }
                                "menubar-sub-content" => sub_content = skids,
                                _ => {}
                            }
                        }
                    }
                    events.push((vec![item_idx], format!("__menubar_toggle(\"{}\")", sub_id)));
                    let toggle_msg = DynamicMessage::Typed {
                        widget_name: self.widget_name.clone(),
                        event_name: "__menubar_toggle".to_string(),
                        args: vec![Value::str(sub_id.as_str())],
                    };
                    let chevron = View::Image {
                        src: if sub_open {
                            "lucide:chevron-down".to_string()
                        } else {
                            "lucide:chevron-right".to_string()
                        },
                        style: Style::parse("w-3.5 h-3.5 text-muted-foreground shrink-0").ok(),
                    };
                    let title_text = View::Text {
                        content: sub_title.clone(),
                        style: Style::parse("text-[12px] text-popover-foreground").ok(),
                        selectable: false,
                    };
                    let trigger = View::Button {
                        disabled: false,
                        label: sub_title,
                        onclick: toggle_msg.clone(),
                        style: Style::parse(
                            "h-7 w-full px-0 py-0 justify-start text-left rounded-sm hover:bg-accent",
                        )
                        .ok(),
                        on_right_click: None,
                        content: Some(Box::new(View::Row {
                            children: vec![title_text, chevron],
                            spacing: 0,
                            padding: 0,
                            style: Style::parse(
                                "w-full justify-between items-center gap-2 px-2",
                            )
                            .ok(),
                            onclick: None,
                            on_right_click: None,
                        })),
                    };
                    // 子面板递归（group 内 item 序扩展事件路径）。
                    let (sub_views, sub_events) = self.build_menu_panel_items(
                        sub_content,
                        bindings,
                        &sub_id,
                        open,
                        depth + 1,
                        radio_value,
                    );
                    // PLAN-698 T-03b：浮动式恢复——嵌套 Popover（iced 0.14
                    // overlay::Nested 官方嵌套协议 + Panel::overlay 收集钩子
                    // 落地，见 popover.rs T-03a spike）。RightTop=T-06 顶对齐
                    // 右弹变体。内联臂降为降级路径（AUTO_MENU_SUBMENU_INLINE=1
                    // 时走旧形态，判位/回退用，P695-D2 划线）。
                    events.extend(sub_events.into_iter().map(|(p, h)| {
                        let mut full = vec![item_idx];
                        full.extend(p);
                        (full, h)
                    }));
                    let submenu_inline_fallback =
                        std::env::var("AUTO_MENU_SUBMENU_INLINE").as_deref() == Ok("1");
                    if !sub_views.is_empty() && !submenu_inline_fallback {
                        let mut sub_panel_style = Style::parse(
                            "w-44 bg-popover text-popover-foreground border border-border shadow-md py-1",
                        )
                        .ok();
                        if let Some(st) = sub_panel_style.as_mut() {
                            if !st.classes.iter().any(|c| matches!(c, StyleClass::Width(_))) {
                                let owned = std::mem::take(st);
                                *st = owned.add(StyleClass::Width(SizeValue::Auto));
                            }
                        }
                        let sub_panel = View::Column {
                            children: sub_views,
                            spacing: 0,
                            padding: 0,
                            style: sub_panel_style,
                            onclick: None,
                            on_right_click: None,
                        };
                        items.push(View::Popover {
                            anchor: PopoverAnchor::Widget(Box::new(trigger)),
                            content: Box::new(sub_panel),
                            placement: PopoverPlacement::RightTop,
                            open: sub_open,
                            on_dismiss: None,
                        });
                    } else {
                        items.push(trigger);
                        if sub_open {
                            items.push(View::Column {
                                children: sub_views,
                                spacing: 0,
                                padding: 0,
                                style: Style::parse(
                                    "w-full pl-4 ml-2 border-l border-border py-0.5",
                                )
                                .ok(),
                                onclick: None,
                                on_right_click: None,
                            });
                        }
                    }
                }
                // PLAN-695 T-07: 语义分组容器——透传（样式中立）。
                "menubar-group" => {
                    let (group_items, group_events) = self.build_menu_panel_items(
                        ikids,
                        bindings,
                        open_key_prefix,
                        open,
                        depth,
                        radio_value,
                    );
                    events.extend(group_events.into_iter().map(|(p, h)| {
                        let mut full = vec![item_idx];
                        full.extend(p);
                        (full, h)
                    }));
                    items.push(View::Column {
                        children: group_items,
                        spacing: 0,
                        padding: 0,
                        style: Style::parse("w-full").ok(),
                        onclick: None,
                        on_right_click: None,
                    });
                }
                _ => {}
            }
        }
        (items, events)
    }

    /// PLAN-630 T-01: declarative menubar component family (shadcn
    /// Menubar 语义的 VM lowering) —
    /// `menubar { menubar-menu (value) { menubar-trigger "文件"
    ///   menubar-content { menubar-item (title, icon, shortcut)
    ///     { onclick } menubar-separator
    ///     menubar-checkbox-item (title, checked) { onclick } } } }`.
    /// 触发/开合/定位走公共 Popover 原语（BottomStart + MENUBAR_OPEN
    /// 注册表，与 actions 合成同机制）；菜单项 presentation 共享
    /// [`Self::menu_item_button_view`]。
    pub(super) fn convert_menubar_component_raw(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
        path: Option<(&[usize], &mut BuildProbe)>,
    ) -> View<DynamicMessage> {
        use crate::ui::view::{PopoverAnchor, PopoverPlacement};

        let (base_vec, mut probe_mut): (Option<Vec<usize>>, Option<&mut BuildProbe>) = match path {
            Some((b, p)) => (Some(b.to_vec()), Some(p)),
            None => (None, None),
        };
        macro_rules! record {
            ($($idx:expr),* => $handler:expr) => {
                if let (Some(base), Some(probe)) = (&base_vec, probe_mut.as_deref_mut()) {
                    let mut child = base.clone();
                    $(child.push($idx);)*
                    let p: Vec<u16> = child.iter().map(|&x| x as u16).collect();
                    probe.record_event(&p, "onclick", $handler);
                }
            };
        }

        fn node_tag(n: &AuraNode) -> Option<&str> {
            match n {
                AuraNode::Element { tag, .. } => Some(tag.as_str()),
                _ => None,
            }
        }
        fn node_children(n: &AuraNode) -> &[AuraNode] {
            match n {
                AuraNode::Element { children, .. } => children,
                _ => &[],
            }
        }
        fn node_text(n: &AuraNode) -> Option<String> {
            match n {
                AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                _ => None,
            }
        }
        fn node_props(n: &AuraNode) -> Option<&HashMap<String, AuraPropValue>> {
            match n {
                AuraNode::Element { props, .. } => Some(props),
                _ => None,
            }
        }

        let open = crate::ui::action_config::menubar_open();
        let mut children_out: Vec<View<DynamicMessage>> = Vec::new();
        let mut menu_index = 0usize;
        for menu_node in children {
            let Some(tag) = node_tag(menu_node) else {
                continue;
            };
            let tag_lc = tag.replace('_', "-");
            if tag_lc != "menubar-menu" {
                continue;
            }
            let menu_id = node_props(menu_node)
                .and_then(|mp| self.extract_string_with(mp, "value", bindings))
                .unwrap_or_else(|| format!("menu-{menu_index}"));
            menu_index += 1;
            let (trigger_title, content_nodes) = {
                let mut title = String::new();
                let mut content: &[AuraNode] = &[];
                for kid in node_children(menu_node) {
                    match node_tag(kid).unwrap_or("").replace('_', "-").as_str() {
                        "menubar-trigger" => {
                            title = node_props(kid)
                                .and_then(|kp| self.extract_string_with(kp, "text", bindings))
                                .or_else(|| node_children(kid).iter().find_map(node_text))
                                .unwrap_or_default();
                        }
                        "menubar-content" => content = node_children(kid),
                        _ => {}
                    }
                }
                (title, content)
            };

            // PLAN-695 T-06 修正：复合键注册表下单值语义——外层菜单在自身
            // 或任一 submenu（`<id>::…` 前缀）开态时都保持展开，否则子菜单
            // 键替换整值会把外层面板一起关掉（实机走查实锤）。
            let is_open = open
                .as_deref()
                .map(|o| o == menu_id.as_str() || o.starts_with(&format!("{}::", menu_id)))
                .unwrap_or(false);
            record!(children_out.len(), 0 => &format!(r#"__menubar_toggle("{}")"#, menu_id));
            // PLAN-695 T-05：trigger 文本 token 化（T-04）+ 开态 accent 高亮
            //（对齐 shadcn data-[state=open]:bg-accent）。闭态 hover 高亮
            //（hover:bg-accent）为组件族默认——shadcn 上游 trigger 无 hover,
            // 桌面 menubar 惯例悬停标题应有反馈,资产侧 MenubarTrigger.vue
            // 同批补齐,两轨同词汇。
            let trigger_btn = View::Button {
                disabled: false,
                label: trigger_title.clone(),
                onclick: DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: "__menubar_toggle".to_string(),
                    args: vec![Value::str(menu_id.as_str())],
                },
                style: Style::parse(&format!(
                    "h-7 px-3 rounded-sm text-[12px] {}",
                    if is_open {
                        "bg-accent text-accent-foreground"
                    } else {
                        "mr-1 text-foreground hover:bg-accent hover:text-accent-foreground"
                    }
                ))
                .ok(),
                on_right_click: None,
                content: None,
            };
            // PLAN-695 T-05: hover-switch——开态下指针进入其他 trigger 直接
            // 切换（MouseArea on_enter → __MenubarToggle，`__` 前缀内部消息
            // 不触发 menubar-snapshot 规则 3 的自动关闭判据）；关闭态悬停
            // 不开（无包裹），本菜单已开不重入。
            let trigger = if open.is_some() && !is_open {
                View::MouseArea {
                    content: Box::new(trigger_btn),
                    on_enter: Some(DynamicMessage::Typed {
                        widget_name: self.widget_name.clone(),
                        event_name: "__menubar_toggle".to_string(),
                        args: vec![Value::str(menu_id.as_str())],
                    }),
                    on_exit: None,
                    on_double_click: None,
                    on_click: None,
                    on_context_menu: None,
                    on_release: None,
                    on_move: None,
                    logical_extent: None,
                    style: None,
                }
            } else {
                trigger_btn
            };

            let mut items: Vec<View<DynamicMessage>> = Vec::new();
            if is_open {
                // PLAN-695 T-06/T-07: 全 kind 面板构建（item/checkbox/
                // radio/label/shortcut/separator/sub/group）收口 helper，
                // 主面板与 submenu 子面板共用（见 build_menu_panel_items）。
                let (built, built_events) = self.build_menu_panel_items(
                    content_nodes,
                    bindings,
                    menu_id.as_str(),
                    &open,
                    0,
                    "",
                );
                items = built;
                if let (Some(base), Some(pr)) = (&base_vec, probe_mut.as_deref_mut()) {
                    for (rel, handler) in &built_events {
                        let mut child = base.clone();
                        child.push(children_out.len());
                        child.push(1);
                        child.extend(rel.iter().copied());
                        let p: Vec<u16> = child.iter().map(|&x| x as u16).collect();
                        pr.record_event(&p, "onclick", handler);
                    }
                }
            }

            // PLAN-695 T-04：面板配色字面 #16171B/zinc-700 → popover/border
            // 语义 token（浅色主题白板深字可读；深色 registry dark 表同源）。
            let mut panel_style = Style::parse(
                "w-44 bg-popover text-popover-foreground border border-border shadow-md py-1",
            )
            .ok();
            if let Some(st) = panel_style.as_mut() {
                if !st.classes.iter().any(|c| matches!(c, StyleClass::Width(_))) {
                    let owned = std::mem::take(st);
                    *st = owned.add(StyleClass::Width(SizeValue::Auto));
                }
            }
            let panel = View::Column {
                children: items,
                spacing: 0,
                padding: 0,
                style: panel_style,
                onclick: None,
                on_right_click: None,
            };
            children_out.push(View::Popover {
                anchor: PopoverAnchor::Widget(Box::new(trigger)),
                content: Box::new(panel),
                placement: PopoverPlacement::BottomStart,
                open: is_open,
                on_dismiss: Some(DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: "__menubar_close".to_string(),
                    args: vec![],
                }),
            });
        }

        let user = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .unwrap_or_default();
        View::Row {
            children: children_out,
            spacing: 0,
            padding: 0,
            style: if user.is_empty() {
                None
            } else {
                Style::parse(&user).ok()
            },
            onclick: None,
            on_right_click: None,
        }
    }

    pub(super) fn convert_menubar_raw(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
        path: Option<(&[usize], &mut BuildProbe)>,
    ) -> View<DynamicMessage> {
        use crate::ui::action_config::{action_config, menubar_open, MenuItem};
        use crate::ui::view::{PopoverAnchor, PopoverPlacement};

        // Split the optional (path, probe) once: matching through a shared
        // reference would downgrade the &mut binding, so we own the pieces.
        let (base_vec, mut probe_mut): (Option<Vec<usize>>, Option<&mut BuildProbe>) = match path {
            Some((b, p)) => (Some(b.to_vec()), Some(p)),
            None => (None, None),
        };
        macro_rules! record {
            ($($idx:expr),* => $handler:expr) => {
                if let (Some(base), Some(probe)) = (&base_vec, probe_mut.as_deref_mut()) {
                    let mut child = base.clone();
                    $(child.push($idx);)*
                    let p: Vec<u16> = child.iter().map(|&x| x as u16).collect();
                    probe.record_event(&p, "onclick", $handler);
                }
            };
        }

        let empty = || View::Column {
            children: vec![],
            spacing: 0,
            padding: 0,
            style: None,
            onclick: None,
            on_right_click: None,
        };
        let Some(cfg) = action_config() else {
            return empty();
        };
        if cfg.menus.is_empty() {
            return empty();
        }

        let open = menubar_open();
        let mut children: Vec<View<DynamicMessage>> = Vec::new();
        for menu in &cfg.menus {
            // 前缀感知开态（submenu 复合键兼容，同声明式臂注）。
            let is_open = open
                .as_deref()
                .map(|o| o == menu.id.as_str() || o.starts_with(&format!("{}::", menu.id)))
                .unwrap_or(false);
            // §8.4①: probe 路径按真实子位置记录 —— popover 占 children.len(),
            // 触发按钮是其 anchor 子(子序 0,与 snapshot/render_dynamic_view
            // 的 Popover 子序约定一致)。
            record!(children.len(), 0 => &format!("__menubar_toggle(\"{}\")", menu.id));
            // PLAN-695 T-05：开态 accent 高亮 + hover-switch（声明式臂同款，
            // 见 convert_menubar_component 注）。
            let trigger_btn = View::Button {
                disabled: false,
                label: menu.title.clone(),
                onclick: DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: "__menubar_toggle".to_string(),
                    args: vec![Value::str(menu.id.as_str())],
                },
                style: Style::parse(&format!(
                    "h-7 px-3 rounded-sm text-[12px] {}",
                    if is_open {
                        "bg-accent text-accent-foreground"
                    } else {
                        "mr-1 text-foreground hover:bg-accent hover:text-accent-foreground"
                    }
                ))
                .ok(),
                on_right_click: None,
                content: None,
            };
            let trigger = if open.is_some() && !is_open {
                View::MouseArea {
                    content: Box::new(trigger_btn),
                    on_enter: Some(DynamicMessage::Typed {
                        widget_name: self.widget_name.clone(),
                        event_name: "__menubar_toggle".to_string(),
                        args: vec![Value::str(menu.id.as_str())],
                    }),
                    on_exit: None,
                    on_double_click: None,
                    on_click: None,
                    on_context_menu: None,
                    on_release: None,
                    on_move: None,
                    logical_extent: None,
                    style: None,
                }
            } else {
                trigger_btn
            };

            // 面板项路径 [base, popover_idx, 1, item_idx](content 是 popover
            // 的子序 1;sep 占位由 items.len() 自然计数,Action 项记录时机
            // 在 push 之前)。
            let mut items: Vec<View<DynamicMessage>> = Vec::new();
            if is_open {
                for item in &menu.items {
                    match item {
                        MenuItem::Separator => {
                            // PLAN-626 T-02: 下拉面板内 sep 必须横向通栏
                            // （convert_sep 默认 vertical 是 toolbar/行内语义
                            // ——此前 menubar 从不传 orientation，下拉里渲染
                            // 成 w-7 盒内短竖线，用户实机反馈）。
                            let mut sep_props = HashMap::new();
                            sep_props.insert(
                                "orientation".to_string(),
                                AuraPropValue::Expr(Expr::Str("horizontal".into())),
                            );
                            items.push(self.convert_sep(&sep_props, bindings));
                        }
                        MenuItem::Action(id) => {
                            let Some(a) = cfg.action_by_id(id) else {
                                continue;
                            };
                            let handler = a.handler.trim_start_matches('.').to_string();
                            record!(children.len(), 1, items.len() => &a.handler);
                            // Plan 418 §8.4③ + 423 P3: checked-if / enabled-if
                            // 统一走 eval_condition_with 表达式引擎(支持
                            // `.field`、比较运算、state 引用;求值失败=未勾选/
                            // 禁用,保守)。每项固定 16px 勾选槽,已勾选时渲染
                            // lucide check 图标(对齐 VSCode 菜单)。
                            let checked = a
                                .checked_if
                                .as_deref()
                                .map(|cond| self.eval_condition_with(cond, bindings))
                                .unwrap_or(false);
                            let enabled = a
                                .enabled_if
                                .as_deref()
                                .map(|cond| self.eval_condition_with(cond, bindings))
                                .unwrap_or(true);
                            // PLAN-629 T-06: 前导槽 = 勾选（checked 时）或
                            // action 声明的 lucide icon（用户要求每项带
                            // icon）；title 紧贴前导槽左对齐——旧结构
                            // [check][title][shortcut] 三子 justify-between
                            // 的弹性撑杆把 title 推到面板中部（左大半空）。
                            // 改两组：[槽+title] | [shortcut]，Between 只在
                            // 组间插撑杆 → 左组贴左、快捷键贴右。
                            const LEADING_EMPTY: &str = "w-4 h-4 shrink-0";
                            let onclick = DynamicMessage::Typed {
                                widget_name: self.widget_name.clone(),
                                event_name: handler,
                                args: vec![],
                            };
                            items.push(self.menu_item_button_view(
                                &a.title,
                                a.icon.clone(),
                                a.shortcut.clone(),
                                checked,
                                enabled,
                                Some(onclick),
                                "lucide:check",
                            ));
                        }
                    }
                }
            }
            // chrome 全部留在面板列自身(bg/border/shadow 走既有 visual
            // wrap);定位交给 popover(overlay 层,不占文档流)。
            // PLAN-626 T-02: 宽度防线镜像 convert_popover 的 PLAN-526 T27
            // 注入——menubar 面板不经 convert_popover，width 类若解析失败
            // 会按块级语义 Fill 撑满宿主宽；w-48→w-44 收窄贴近 shadcn 菜单
            // 内容宽。
            // PLAN-695 T-04：面板配色字面 #16171B/zinc-700 → popover/border
            // 语义 token（浅色主题白板深字可读）。
            let mut panel_style = Style::parse(
                "w-44 bg-popover text-popover-foreground border border-border shadow-md py-1",
            )
            .ok();
            if let Some(s) = panel_style.as_mut() {
                if !s.classes.iter().any(|c| matches!(c, StyleClass::Width(_))) {
                    let owned = std::mem::take(s);
                    *s = owned.add(StyleClass::Width(SizeValue::Auto));
                }
            }
            let panel = View::Column {
                children: items,
                spacing: 0,
                padding: 0,
                style: panel_style,
                onclick: None,
                on_right_click: None,
            };
            children.push(View::Popover {
                anchor: PopoverAnchor::Widget(Box::new(trigger)),
                content: Box::new(panel),
                placement: PopoverPlacement::BottomStart,
                open: is_open,
                on_dismiss: Some(DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: "__menubar_close".to_string(),
                    args: vec![],
                }),
            });
        }

        let user = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .unwrap_or_default();
        View::Row {
            children,
            spacing: 0,
            padding: 0,
            style: if user.is_empty() {
                None
            } else {
                Style::parse(&user).ok()
            },
            onclick: None,
            on_right_click: None,
        }
    }

    /// Plan 422 P3/P4: `popover` DSL 标签 —— 锚定弹层的应用侧入口。三形态:
    ///
    /// * 坐标锚(contextmenu):`popover (open: .ctx_open, x: .ctx_x, y: .ctx_y,
    ///   ondismiss: .CtxClose, class: "面板样式") { 面板子节点… }` —— 面板
    ///   左上角对齐 (x, y) 落点;class 给面板 chrome(bg/border/shadow)。
    /// * shadcn 嵌套(gallery 兼容,自管开合):`popover { popover-trigger {
    ///   按钮等 } popover-content { 面板子节点… } }` —— 触发按钮点击开合
    ///   (内部 __popover_toggle 消息,menubar 同型注册表),无 open 属性。
    /// * widget 锚(first-child):`popover (placement: "bottom-start") {
    ///   触发按钮; 面板子节点… }` —— children[0] 为锚,其余为面板内容。
    ///
    /// 显式 `open` 属性优先(应用态驱动);缺失时走自管注册表。
    /// 子路径约定与 vnode/snapshot/find_view_by_path 的 extract_children 同序:
    /// widget 锚 anchor=0/content=1;坐标锚 content=0。
    // PLAN-530 步骤8（源 PLAN-528 W13）：alert_dialog VM 映射——复用 Plan 422
    // Popover 原语的模态形态（PopoverPlacement::Modal：面板视口居中 + 全屏
    // 半透明遮罩 + 面板外点击整吞）。此前 alert-dialog-* 全族落 unknown
    // fallback（overlay 家族 "renders as Column"），点击无弹层。
    // shadcn AlertDialog 语义：外点/Esc 不关闭（on_dismiss=None），关闭仅经
    // cancel/action 操作钮的 VM handler 翻转 `open` 绑定状态。
    /// PLAN-533 T5: 裸文本 trigger（`dialog-trigger "Open"` → text prop
    /// 无子节点）→ Button（vue trigger 语义）。onclick 经 convert_button
    /// 取 parser 铸造的 `.__dlg_toggle_<n>`（或用户显式绑定）;无文字且无
    /// 子件 → Empty。
    pub(super) fn bare_trigger_button_view(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let has_label = ["text", "label"].iter().any(|k| {
            matches!(
                props.get(*k),
                Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))) if !s.is_empty()
            )
        });
        if has_label {
            self.convert_button(props, events, &[], bindings)
        } else {
            View::Empty
        }
    }

    /// PLAN-533 T7: 模态对话框族别——Alert/Dialog 共用模态卡 chrome,
    /// DropdownMenu 锚定菜单（BottomStart + p-1 紧凑档）。
    pub(super) fn modal_dialog_family(tag: &str) -> ModalDialogFamily {
        let norm: String = tag
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_lowercase();
        match norm.as_str() {
            "dropdownmenu" => ModalDialogFamily::DropdownMenu,
            "dialog" => ModalDialogFamily::Dialog,
            _ => ModalDialogFamily::Alert,
        }
    }

    /// PLAN-533 T6: 铸造形态（open 绑 `__dlg_open_N`）→ dismiss 折算消息
    /// `__dlg_close_N`。非铸造形态（用户显式 open 绑定/无 open prop）返回
    /// None（自管语义,Phase 1 不接管）。
    pub(super) fn minted_dismiss_msg(
        widget: &str,
        props: &HashMap<String, AuraPropValue>,
    ) -> Option<DynamicMessage> {
        let AuraPropValue::Expr(auto_lang_core::ast::Expr::Dot(obj, field)) = props.get("open")?
        else {
            return None;
        };
        if !matches!(
            obj.as_ref(),
            auto_lang_core::ast::Expr::Ident(b) if b.as_str() == "self" || b.as_str() == "."
        ) {
            return None;
        }
        field
            .to_string()
            .strip_prefix("__dlg_open_")
            .map(|n| DynamicMessage::Typed {
                widget_name: widget.to_string(),
                event_name: format!("__dlg_close_{n}"),
                args: vec![],
            })
    }

    pub(super) fn alert_dialog_split_children(
        children: &[AuraNode],
    ) -> (Option<&AuraNode>, Option<&AuraNode>) {
        let mut trigger: Option<&AuraNode> = None;
        let mut content: Option<&AuraNode> = None;
        for c in children {
            if let AuraNode::Element { tag, .. } = c {
                let t = tag.as_str();
                if matches!(
                    t,
                    "alert-dialog-trigger"
                        | "alert_dialog_trigger"
                        | "alertdialog-trigger"
                        | "dialog-trigger"
                        | "dialog_trigger"
                        | "dialogtrigger"
                        | "dropdown-menu-trigger"
                        | "dropdown_menu_trigger"
                        | "dropdownmenutrigger"
                        // PLAN-534: sheet/drawer/hovercard trigger 全形态
                        // （kebab/snake/连写;hover-card 另有 hovercard- 混合形）。
                        | "sheet-trigger"
                        | "sheet_trigger"
                        | "sheettrigger"
                        | "drawer-trigger"
                        | "drawer_trigger"
                        | "drawertrigger"
                        | "hover-card-trigger"
                        | "hover_card_trigger"
                        | "hovercard-trigger"
                        | "hovercardtrigger"
                ) && trigger.is_none()
                {
                    trigger = Some(c);
                } else if matches!(
                    t,
                    "alert-dialog-content"
                        | "alert_dialog_content"
                        | "alertdialog-content"
                        | "dialog-content"
                        | "dialog_content"
                        | "dialogcontent"
                        | "dropdown-menu-content"
                        | "dropdown_menu_content"
                        | "dropdownmenucontent"
                        // PLAN-534: sheet/drawer/hovercard content 全形态。
                        | "sheet-content"
                        | "sheet_content"
                        | "sheetcontent"
                        | "drawer-content"
                        | "drawer_content"
                        | "drawercontent"
                        | "hover-card-content"
                        | "hover_card_content"
                        | "hovercard-content"
                        | "hovercardcontent"
                ) && content.is_none()
                {
                    content = Some(c);
                }
            }
        }
        (trigger, content)
    }

    /// class prop 合并（静态直拼;已有动态绑定串按当前求值拍平——构建期
    /// 一次求值,VM 轨可接受）。
    ///
    /// PLAN-619 R1：用户类串同时接受 `class:` 与 `style:` 两个键——DSL 里
    /// `style:` 是主要写法（本仓示例几乎全用它），旧实现只读 `class:`，于是
    /// sidebar 契约族的用户类（`sidebar_content (style: "px-2 pb-2")`、
    /// `sidebar_menu_button { style: "py-1.5 …" }`）被整串静默丢弃，只剩契约
    /// 基座。其余 8 处样式取值点都是 `.or_else(style)`，此处补齐同一口径。
    pub(super) fn with_class_prop(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
        extra: &str,
    ) -> HashMap<String, AuraPropValue> {
        let existing = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .unwrap_or_default();
        let merged = if existing.is_empty() {
            extra.to_string()
        } else {
            format!("{existing} {extra}")
        };
        let mut p = props.clone();
        p.insert(
            "class".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(merged.into())),
        );
        p
    }

    pub(super) fn convert_alert_dialog_raw(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
        family: ModalDialogFamily,
    ) -> View<DynamicMessage> {
        use crate::ui::view::{PopoverAnchor, PopoverPlacement};
        let (trigger, content) = Self::alert_dialog_split_children(children);
        // 锚:trigger 子件原位渲染（无 trigger = 空 anchor,open 属性直驱）。
        // PLAN-533 T5: 裸文本 trigger（text prop 无子）→ Button,onclick 取
        // parser 铸造的 toggle（或用户显式绑定）。
        let anchor_view = match trigger {
            Some(AuraNode::Element {
                children: t_children,
                props: t_props,
                events: t_events,
                ..
            }) => {
                if t_children.is_empty() {
                    self.bare_trigger_button_view(t_props, t_events, bindings)
                } else {
                    let views: Vec<View<DynamicMessage>> = t_children
                        .iter()
                        .map(|c| self.convert_node_with(c, bindings))
                        .collect();
                    match views.len() {
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
            }
            _ => View::Empty,
        };
        // 面板:content 子件装 col,挂 shadcn AlertDialogContent 同款 chrome。
        let panel_children: Vec<View<DynamicMessage>> = match content {
            Some(AuraNode::Element {
                children: c_children,
                ..
            }) => c_children
                .iter()
                .map(|c| self.convert_node_with(c, bindings))
                .collect(),
            _ => Vec::new(),
        };
        // PLAN-533 T7: dropdown-menu 族锚定菜单 chrome（shadcn
        // DropdownMenuContent 同款 p-1 紧凑档 + BottomStart）;对话框族
        // 保持 w-96 模态卡。
        let (panel_chrome, placement) = match family {
            ModalDialogFamily::DropdownMenu => (
                "w-44 bg-popover border border-border rounded-md shadow-md p-1 gap-1",
                PopoverPlacement::BottomStart,
            ),
            _ => (
                "w-96 bg-background border border-border rounded-lg shadow-lg p-6 gap-4",
                PopoverPlacement::Modal,
            ),
        };
        let panel = View::Column {
            children: panel_children,
            spacing: 0,
            padding: 0,
            style: Style::parse(panel_chrome).ok(),
            onclick: None,
            on_right_click: None,
        };
        let open = match props.get("open") {
            Some(AuraPropValue::Expr(e)) => matches!(
                self.resolve_expr_to_value(e, bindings),
                Some(Value::Bool(true))
            ),
            _ => false,
        };
        // PLAN-533 T6: 可关闭族（dialog）铸造形态的 ESC/外点/锚点 dismiss
        // 折算 __dlg_close_N（update:open(false)）;alert 族与显式绑定自管
        // 形态不接管。
        let on_dismiss = if matches!(
            family,
            ModalDialogFamily::Dialog | ModalDialogFamily::DropdownMenu
        ) {
            Self::minted_dismiss_msg(&self.widget_name, props)
        } else {
            None
        };
        View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(anchor_view)),
            content: Box::new(panel),
            placement,
            open,
            on_dismiss,
        }
    }

    /// convert_alert_dialog 的 tracked 镜像（D-GAP 规则）：trigger/content
    /// 子件经 path 追踪递归,inspector/MCP 快照可定位面板内节点。
    pub(super) fn convert_alert_dialog_tracked_ctx_raw(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
        family: ModalDialogFamily,
    ) -> View<DynamicMessage> {
        use crate::ui::view::{PopoverAnchor, PopoverPlacement};
        let (trigger, content) = Self::alert_dialog_split_children(children);
        let anchor_view = match trigger {
            Some(AuraNode::Element {
                children: t_children,
                props: t_props,
                events: t_events,
                ..
            }) => {
                // PLAN-533 T5: 裸文本 trigger（text prop 无子）→ Button
                // （铸造/显式 onclick 经 convert_button 接线）。
                if t_children.is_empty() {
                    self.bare_trigger_button_view(t_props, t_events, bindings)
                } else {
                    let mut views: Vec<View<DynamicMessage>> = Vec::new();
                    for (i, c) in t_children.iter().enumerate() {
                        path.push(i);
                        views.push(self.convert_node_tracked_ctx(c, path, id_map, probe, bindings));
                        path.pop();
                    }
                    match views.len() {
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
            }
            _ => View::Empty,
        };
        let mut panel_children: Vec<View<DynamicMessage>> = Vec::new();
        if let Some(AuraNode::Element {
            children: c_children,
            ..
        }) = content
        {
            for (i, c) in c_children.iter().enumerate() {
                path.push(i);
                panel_children
                    .push(self.convert_node_tracked_ctx(c, path, id_map, probe, bindings));
                path.pop();
            }
        }
        // PLAN-533 T7: dropdown-menu 族锚定菜单 chrome（shadcn
        // DropdownMenuContent 同款 p-1 紧凑档 + BottomStart）;对话框族
        // 保持 w-96 模态卡。
        let (panel_chrome, placement) = match family {
            ModalDialogFamily::DropdownMenu => (
                "w-44 bg-popover border border-border rounded-md shadow-md p-1 gap-1",
                PopoverPlacement::BottomStart,
            ),
            _ => (
                "w-96 bg-background border border-border rounded-lg shadow-lg p-6 gap-4",
                PopoverPlacement::Modal,
            ),
        };
        let panel = View::Column {
            children: panel_children,
            spacing: 0,
            padding: 0,
            style: Style::parse(panel_chrome).ok(),
            onclick: None,
            on_right_click: None,
        };
        let open = match props.get("open") {
            Some(AuraPropValue::Expr(e)) => matches!(
                self.resolve_expr_to_value(e, bindings),
                Some(Value::Bool(true))
            ),
            _ => false,
        };
        // PLAN-533 T6: 可关闭族（dialog）铸造形态的 ESC/外点/锚点 dismiss
        // 折算 __dlg_close_N（update:open(false)）;alert 族与显式绑定自管
        // 形态不接管。
        let on_dismiss = if matches!(
            family,
            ModalDialogFamily::Dialog | ModalDialogFamily::DropdownMenu
        ) {
            Self::minted_dismiss_msg(&self.widget_name, props)
        } else {
            None
        };
        View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(anchor_view)),
            content: Box::new(panel),
            placement,
            open,
            on_dismiss,
        }
    }

    /// PLAN-534 D2: sheet 的 `side` / drawer 的 `direction` prop → Edge
    /// 贴边 placement。字符串字面优先,非字面经 resolve_expr_to_value 取
    /// str;非法值落缺省 right 并打日志（构建期一次裁定）。
    pub(super) fn side_panel_placement(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> crate::ui::view::PopoverPlacement {
        use crate::ui::view::PopoverPlacement;
        let raw = match props.get(key) {
            Some(AuraPropValue::Expr(Expr::Str(s))) => Some(s.to_string()),
            Some(AuraPropValue::Expr(e)) => {
                self.resolve_expr_to_value(e, bindings)
                    .and_then(|v| match v {
                        Value::Str(s) => Some(s.to_string()),
                        _ => None,
                    })
            }
            _ => None,
        };
        match raw.as_deref() {
            Some("left") => PopoverPlacement::EdgeLeft,
            Some("top") => PopoverPlacement::EdgeTop,
            Some("bottom") => PopoverPlacement::EdgeBottom,
            Some("right") | None => PopoverPlacement::EdgeRight,
            Some(other) => {
                eprintln!("[PLAN-534] invalid {key}={other:?}, falling back to right");
                PopoverPlacement::EdgeRight
            }
        }
    }

    /// PLAN-534 D2: sheet/drawer 面板 chrome（与 rust.rs 轨同串,保双轨
    /// 视觉一致）。横向（L/R）w-96 定宽 + h-full 拉满;纵向（T/B）w-full
    /// 拉满;drawer 竖向追加贴缘圆角（bottom rounded-t / top rounded-b）。
    pub(super) fn side_panel_chrome(placement: crate::ui::view::PopoverPlacement, is_drawer: bool) -> String {
        use crate::ui::view::PopoverPlacement;
        let base = match placement {
            PopoverPlacement::EdgeLeft | PopoverPlacement::EdgeRight => {
                "w-96 bg-background border shadow-lg p-6 gap-4 h-full".to_string()
            }
            _ => "bg-background border shadow-lg p-6 gap-4 w-full".to_string(),
        };
        if is_drawer {
            match placement {
                PopoverPlacement::EdgeBottom => return format!("{base} rounded-t-lg"),
                PopoverPlacement::EdgeTop => return format!("{base} rounded-b-lg"),
                _ => {}
            }
        }
        base
    }

    /// PLAN-534 D2: drawer 竖向装饰把手——全宽容器内居中的 w-8 h-1 圆角
    /// 条（vaul 视觉;纯装饰无手势,差异登记 KNOWN-DEBT）。
    pub(super) fn drawer_handle_view() -> View<DynamicMessage> {
        View::Container {
            child: Box::new(View::Container {
                child: Box::new(View::Empty),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                onclick: None,
                on_right_click: None,
                style: Style::parse("w-8 h-1 rounded-full bg-muted").ok(),
            }),
            padding: 0,
            width: None,
            height: None,
            center_x: true,
            center_y: false,
            onclick: None,
            on_right_click: None,
            style: Style::parse("w-full py-2").ok(),
        }
    }

    /// PLAN-534 D2: sheet/drawer 贴边面板转换（side 参数化,双标签共用;
    /// untracked 镜像臂）。trigger/content 拆解 + 裸文本 trigger→Button +
    /// 铸造 dismiss 折算,与 dialog 族同构;placement 为 Edge 族（scrim +
    /// 面板外/Esc 关,shadcn Sheet 语义）。
    pub(super) fn convert_side_panel(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
        placement: crate::ui::view::PopoverPlacement,
        is_drawer: bool,
    ) -> View<DynamicMessage> {
        use crate::ui::view::PopoverAnchor;
        let (trigger, content) = Self::alert_dialog_split_children(children);
        // 锚:trigger 子件原位渲染（裸文本 trigger → Button,同 dialog 族）。
        let anchor_view = match trigger {
            Some(AuraNode::Element {
                children: t_children,
                props: t_props,
                events: t_events,
                ..
            }) => {
                if t_children.is_empty() {
                    self.bare_trigger_button_view(t_props, t_events, bindings)
                } else {
                    let views: Vec<View<DynamicMessage>> = t_children
                        .iter()
                        .map(|c| self.convert_node_with(c, bindings))
                        .collect();
                    match views.len() {
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
            }
            _ => View::Empty,
        };
        let mut panel_children: Vec<View<DynamicMessage>> = match content {
            Some(AuraNode::Element {
                children: c_children,
                ..
            }) => c_children
                .iter()
                .map(|c| self.convert_node_with(c, bindings))
                .collect(),
            _ => Vec::new(),
        };
        // drawer 竖向装饰把手:bottom 首子前插 / top 末子追加。
        if is_drawer {
            match placement {
                crate::ui::view::PopoverPlacement::EdgeBottom => {
                    panel_children.insert(0, Self::drawer_handle_view())
                }
                crate::ui::view::PopoverPlacement::EdgeTop => {
                    panel_children.push(Self::drawer_handle_view())
                }
                _ => {}
            }
        }
        let panel = View::Column {
            children: panel_children,
            spacing: 0,
            padding: 0,
            style: Style::parse(&Self::side_panel_chrome(placement, is_drawer)).ok(),
            onclick: None,
            on_right_click: None,
        };
        let open = match props.get("open") {
            Some(AuraPropValue::Expr(e)) => matches!(
                self.resolve_expr_to_value(e, bindings),
                Some(Value::Bool(true))
            ),
            _ => false,
        };
        // sheet/drawer 均可关闭族:铸造形态 ESC/外点/锚点 dismiss 折算
        // __dlg_close_N;显式绑定自管形态返回 None 不接管（同 dialog）。
        let on_dismiss = Self::minted_dismiss_msg(&self.widget_name, props);
        View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(anchor_view)),
            content: Box::new(panel),
            placement,
            open,
            on_dismiss,
        }
    }

    /// convert_side_panel 的 tracked 镜像（D-GAP 规则）：trigger/content
    /// 子件经 path 追踪递归,inspector/MCP 快照可定位面板内节点。
    pub(super) fn convert_side_panel_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
        placement: crate::ui::view::PopoverPlacement,
        is_drawer: bool,
    ) -> View<DynamicMessage> {
        use crate::ui::view::PopoverAnchor;
        let (trigger, content) = Self::alert_dialog_split_children(children);
        let anchor_view = match trigger {
            Some(AuraNode::Element {
                children: t_children,
                props: t_props,
                events: t_events,
                ..
            }) => {
                if t_children.is_empty() {
                    self.bare_trigger_button_view(t_props, t_events, bindings)
                } else {
                    let mut views: Vec<View<DynamicMessage>> = Vec::new();
                    for (i, c) in t_children.iter().enumerate() {
                        path.push(i);
                        views.push(self.convert_node_tracked_ctx(c, path, id_map, probe, bindings));
                        path.pop();
                    }
                    match views.len() {
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
            }
            _ => View::Empty,
        };
        let mut panel_children: Vec<View<DynamicMessage>> = Vec::new();
        if let Some(AuraNode::Element {
            children: c_children,
            ..
        }) = content
        {
            for (i, c) in c_children.iter().enumerate() {
                path.push(i);
                panel_children
                    .push(self.convert_node_tracked_ctx(c, path, id_map, probe, bindings));
                path.pop();
            }
        }
        if is_drawer {
            match placement {
                crate::ui::view::PopoverPlacement::EdgeBottom => {
                    panel_children.insert(0, Self::drawer_handle_view())
                }
                crate::ui::view::PopoverPlacement::EdgeTop => {
                    panel_children.push(Self::drawer_handle_view())
                }
                _ => {}
            }
        }
        let panel = View::Column {
            children: panel_children,
            spacing: 0,
            padding: 0,
            style: Style::parse(&Self::side_panel_chrome(placement, is_drawer)).ok(),
            onclick: None,
            on_right_click: None,
        };
        let open = match props.get("open") {
            Some(AuraPropValue::Expr(e)) => matches!(
                self.resolve_expr_to_value(e, bindings),
                Some(Value::Bool(true))
            ),
            _ => false,
        };
        let on_dismiss = Self::minted_dismiss_msg(&self.widget_name, props);
        View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(anchor_view)),
            content: Box::new(panel),
            placement,
            open,
            on_dismiss,
        }
    }

    /// PLAN-534 D4: hovercard 转换（untracked 镜像臂）——trigger 包
    /// View::MouseArea 接 hover 进/出（parser 铸造 `__dlg_enter_N/leave_N`
    /// 或用户显式绑定,经通用事件解析同槽读取）;trigger 已有 onclick 的经
    /// 子件自身转换保留并存。content 装 col 挂非模态 chrome,
    /// placement=Bottom（锚相对,悬于锚下）;on_dismiss=None（无 scrim、
    /// 无外点整吞/Esc——关闭只靠 leave;open/close-delay v1 不消费,
    /// KNOWN-DEBT 登记）。降级切换点:MouseArea×overlay 实测异常（如
    /// leave 被 overlay 拦截）→ parser 改铸 toggle + 去本臂 MouseArea
    /// 包裹（差异记录 README）。
    pub(super) fn convert_hovercard(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use crate::ui::view::{PopoverAnchor, PopoverPlacement};
        let (trigger, content) = Self::alert_dialog_split_children(children);
        let (anchor_inner, on_enter, on_exit) = match trigger {
            Some(AuraNode::Element {
                children: t_children,
                props: t_props,
                events: t_events,
                ..
            }) => {
                let on_enter = aura_events_get_base(t_events, "onmouseenter")
                    .or_else(|| aura_events_get_base(t_events, "onhover"))
                    .map(|event| self.event_to_message_with(event, bindings));
                let on_exit = aura_events_get_base(t_events, "onmouseleave")
                    .or_else(|| aura_events_get_base(t_events, "onhoverout"))
                    .map(|event| self.event_to_message_with(event, bindings));
                let inner = if t_children.is_empty() {
                    self.bare_trigger_button_view(t_props, t_events, bindings)
                } else {
                    let views: Vec<View<DynamicMessage>> = t_children
                        .iter()
                        .map(|c| self.convert_node_with(c, bindings))
                        .collect();
                    match views.len() {
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
                };
                (inner, on_enter, on_exit)
            }
            _ => (View::Empty, None, None),
        };
        // 锚包 MouseArea:命中区=trigger 渲染面,hover 进/出驱动铸造 state。
        let anchor_view = View::MouseArea {
            content: Box::new(anchor_inner),
            on_enter,
            on_exit,
            on_double_click: None,
            on_click: None,
            on_context_menu: None,
            on_release: None,
            on_move: None,
            logical_extent: None,
            style: match trigger {
                Some(AuraNode::Element { props: t_props, .. }) => {
                    self.extract_style_with(t_props, bindings)
                }
                _ => None,
            },
        };
        let panel_children: Vec<View<DynamicMessage>> = match content {
            Some(AuraNode::Element {
                children: c_children,
                ..
            }) => c_children
                .iter()
                .map(|c| self.convert_node_with(c, bindings))
                .collect(),
            _ => Vec::new(),
        };
        let panel = View::Column {
            children: panel_children,
            spacing: 0,
            padding: 0,
            style: Style::parse("w-80 bg-popover border rounded-lg shadow-md p-4").ok(),
            onclick: None,
            on_right_click: None,
        };
        let open = match props.get("open") {
            Some(AuraPropValue::Expr(e)) => matches!(
                self.resolve_expr_to_value(e, bindings),
                Some(Value::Bool(true))
            ),
            _ => false,
        };
        View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(anchor_view)),
            content: Box::new(panel),
            placement: PopoverPlacement::Bottom,
            open,
            on_dismiss: None,
        }
    }

    /// convert_hovercard 的 tracked 镜像（D-GAP 规则）：trigger/content
    /// 子件经 path 追踪递归。
    pub(super) fn convert_hovercard_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use crate::ui::view::{PopoverAnchor, PopoverPlacement};
        let (trigger, content) = Self::alert_dialog_split_children(children);
        let (anchor_inner, on_enter, on_exit) = match trigger {
            Some(AuraNode::Element {
                children: t_children,
                props: t_props,
                events: t_events,
                ..
            }) => {
                let on_enter = aura_events_get_base(t_events, "onmouseenter")
                    .or_else(|| aura_events_get_base(t_events, "onhover"))
                    .map(|event| self.event_to_message_with(event, bindings));
                let on_exit = aura_events_get_base(t_events, "onmouseleave")
                    .or_else(|| aura_events_get_base(t_events, "onhoverout"))
                    .map(|event| self.event_to_message_with(event, bindings));
                let inner = if t_children.is_empty() {
                    self.bare_trigger_button_view(t_props, t_events, bindings)
                } else {
                    let mut views: Vec<View<DynamicMessage>> = Vec::new();
                    for (i, c) in t_children.iter().enumerate() {
                        path.push(i);
                        views.push(self.convert_node_tracked_ctx(c, path, id_map, probe, bindings));
                        path.pop();
                    }
                    match views.len() {
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
                };
                (inner, on_enter, on_exit)
            }
            _ => (View::Empty, None, None),
        };
        let anchor_view = View::MouseArea {
            content: Box::new(anchor_inner),
            on_enter,
            on_exit,
            on_double_click: None,
            on_click: None,
            on_context_menu: None,
            on_release: None,
            on_move: None,
            logical_extent: None,
            style: match trigger {
                Some(AuraNode::Element { props: t_props, .. }) => {
                    self.extract_style_with(t_props, bindings)
                }
                _ => None,
            },
        };
        let mut panel_children: Vec<View<DynamicMessage>> = Vec::new();
        if let Some(AuraNode::Element {
            children: c_children,
            ..
        }) = content
        {
            for (i, c) in c_children.iter().enumerate() {
                path.push(i);
                panel_children
                    .push(self.convert_node_tracked_ctx(c, path, id_map, probe, bindings));
                path.pop();
            }
        }
        let panel = View::Column {
            children: panel_children,
            spacing: 0,
            padding: 0,
            style: Style::parse("w-80 bg-popover border rounded-lg shadow-md p-4").ok(),
            onclick: None,
            on_right_click: None,
        };
        let open = match props.get("open") {
            Some(AuraPropValue::Expr(e)) => matches!(
                self.resolve_expr_to_value(e, bindings),
                Some(Value::Bool(true))
            ),
            _ => false,
        };
        View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(anchor_view)),
            content: Box::new(panel),
            placement: PopoverPlacement::Bottom,
            open,
            on_dismiss: None,
        }
    }

    pub(super) fn convert_popover(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use crate::ui::action_config::popover_open;
        use crate::ui::view::{PopoverAnchor, PopoverPlacement};

        let expr_value = |key: &str| -> Option<Value> {
            match props.get(key)? {
                AuraPropValue::Expr(e) => self.resolve_expr_to_value(e, bindings),
                _ => None,
            }
        };
        let has_open_prop = props.contains_key("open");
        let open_prop = expr_value("open").map(|v| matches!(v, Value::Bool(true)));
        let coord = |key: &str| -> Option<f32> {
            match expr_value(key)? {
                Value::Float(f) => Some(f as f32),
                Value::Int(i) => Some(i as f32),
                Value::Uint(u) => Some(u as f32),
                _ => None,
            }
        };
        let (px, py) = (coord("x"), coord("y"));
        // PLAN-528 W9:缺省对齐——shadcn popper 默认 align=center,故 shadcn
        // 嵌套形态(无坐标锚)缺省 Bottom(锚中);坐标锚(contextmenu)保持
        // BottomStart(菜单右下展开约定)。显式 placement 恒优先。
        let placement_prop = self.extract_string_with(props, "placement", bindings);
        let placement = placement_prop
            .as_deref()
            .and_then(|s| match s.to_ascii_lowercase().as_str() {
                "bottom" => Some(PopoverPlacement::Bottom),
                "bottom-start" | "bottomstart" => Some(PopoverPlacement::BottomStart),
                "bottom-end" | "bottomend" => Some(PopoverPlacement::BottomEnd),
                "top" => Some(PopoverPlacement::Top),
                "top-start" | "topstart" => Some(PopoverPlacement::TopStart),
                "top-end" | "topend" => Some(PopoverPlacement::TopEnd),
                "left" => Some(PopoverPlacement::Left),
                "right" => Some(PopoverPlacement::Right),
                // PLAN-695 T-06: submenu 顶对齐右弹（menubar-sub 臂直用变体，
                // placement prop 解析一并收录保持词表完备）。
                "right-top" | "righttop" => Some(PopoverPlacement::RightTop),
                // PLAN-631 F-7: 指针定位——open 翻真时面板出现在最近一次
                // 右键指针位置(渲染器会话级记忆,坐标不进 VM 状态)。
                "pointer" => Some(PopoverPlacement::Pointer),
                _ => None,
            })
            .unwrap_or(if props.contains_key("x") && props.contains_key("y") {
                // PLAN-668 R-05：坐标锚缺省按**声明面**判（x/y props 在场即
                // BottomStart 菜单约定）——运行期可求值性不改变锚形态。
                PopoverPlacement::BottomStart
            } else {
                PopoverPlacement::Bottom
            });
        // PLAN-631 F-7: placement "pointer" 无需坐标锚——无 x/y 时合成原点
        // 点锚（渲染期面板原点被最近按下位置取代；未记录时退化为坐标锚
        // 语义，面板落窗原点）。触发件与面板可分离：单实例菜单挂视图根。
        let (px, py) = match (px, py) {
            (None, None) if placement == PopoverPlacement::Pointer => (Some(0.0f32), Some(0.0f32)),
            other => other,
        };
        // 面板 chrome:popover 标签的 class 落在 content 列上(visual wrap 绘制)。
        // PLAN-528 W9:class 缺省时给 shadcn PopoverContent 同款默认面板
        // chrome(bg-popover + border + rounded-md + shadow-md + p-4)——vue 端
        // PopoverContent 组件自带 chrome,VM 端面板否则裸透明。
        let mut panel_style = self.extract_style_with(props, bindings).or_else(|| {
            // w-72 与 shadcn PopoverContent 一致(shadcn 原版即 w-72 固定宽);
            // 缺宽则面板列被 overlay 的 viewport 上限拉满整窗。
            Style::parse("w-72 bg-popover border border-border rounded-md shadow-md p-4").ok()
        });
        // PLAN-526 T27：面板列无 width 类时注入 Width(Auto)（hug）——
        // 否则 visual-wrap 容器按块级语义 Fill，面板被撑满宿主宽
        // （任务栏 icon 右键菜单横贯整个桌面，用户六轮反馈 #5）。
        // （528 W9 的 w-72 缺省自带 width 类，不受此注入影响。）
        if let Some(s) = panel_style.as_mut() {
            if !s.classes.iter().any(|c| matches!(c, StyleClass::Width(_))) {
                let owned = std::mem::take(s);
                *s = owned.add(StyleClass::Width(SizeValue::Auto));
            }
        }

        // shadcn 嵌套形态分区:popover-trigger / popover-content 子标签拆解。
        let mut trigger_node: Option<&AuraNode> = None;
        let mut content_node: Option<&AuraNode> = None;
        let mut plain: Vec<&AuraNode> = Vec::new();
        for n in children {
            match popover_node_tag(n) {
                Some("popover-trigger") | Some("popover_trigger") => trigger_node = Some(n),
                Some("popover-content") | Some("popover_content") => content_node = Some(n),
                _ => plain.push(n),
            }
        }

        // 自管开合:slot id 按构建路径键(结构稳定则跨重建稳定)。
        let slot_id = format!(
            "pv_{}",
            path.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join("_")
        );
        let self_managed_open = || popover_open().as_deref() == Some(slot_id.as_str());

        let internal_toggle = DynamicMessage::Typed {
            widget_name: self.widget_name.clone(),
            event_name: "__popover_toggle".to_string(),
            args: vec![Value::str(&slot_id)],
        };

        // PLAN-010 N6b 根修:ondismiss 在解析层被 on* 前缀升格进 events 桶
        // (parser.rs ViewEvent 分流),props 查不到——events 兜底基名提取,
        // 否则合成 __popover_close 对 VM 态 popover(状态位驱动 open)是
        // no-op,外点/Esc 关闭全失效(桌面壳 icon/blank 菜单实录)。
        let ondismiss_handler = || {
            self.extract_string_with(props, "ondismiss", bindings)
                .or_else(|| {
                    auto_ui_compiler::aura::aura_events_get_base(events, "ondismiss")
                        .map(|ev| ev.handler.clone())
                })
        };

        // PLAN-668 R-05（027 在册裁定方向）：坐标锚判定按声明面（x/y props
        // 在场）而非运行期可求值——拖拽幽灵 popover（desktop.at，x/y 绑
        // .__desktop_cursor_x/y 宿主回写态）在字段未填充的语境求值 None，
        // 此前被误判 widget 锚形态走首子锚分支 + __popover_close 兜底
        // （VM 态 popover 无此处理语义，外点/Esc 关闭语义受损）。声明了
        // x/y 即坐标锚；不可求值时锚点退 (0,0)。
        let coord_declared = props.contains_key("x") && props.contains_key("y");
        let anchor_xy = if coord_declared {
            (Some(px.unwrap_or(0.0)), Some(py.unwrap_or(0.0)))
        } else {
            (None, None)
        };
        match anchor_xy {
            (Some(x), Some(y)) => {
                let open = open_prop.unwrap_or_else(self_managed_open);
                let on_dismiss = ondismiss_handler().map(|h| DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: h.trim_start_matches('.').to_string(),
                    args: vec![],
                });
                View::Popover {
                    anchor: PopoverAnchor::Point { x, y },
                    content: Box::new(View::Column {
                        children: self
                            .convert_popover_items(children, 0, path, id_map, probe, bindings),
                        spacing: 0,
                        padding: 0,
                        style: panel_style,
                        onclick: None,
                        on_right_click: None,
                    }),
                    placement,
                    open,
                    on_dismiss,
                }
            }
            _ => {
                // 锚子节点:trigger 形态取其 children;first-child 形态取 plain[0]。
                let anchor_src: Vec<&AuraNode> = if let Some(t) = trigger_node {
                    popover_node_children(t).iter().collect()
                } else {
                    plain.iter().take(1).copied().collect()
                };
                // 内容子节点:content 形态的 children + (first-child 形态的)其余 plain。
                let mut content_src: Vec<&AuraNode> = content_node
                    .map(|c| popover_node_children(c).iter().collect())
                    .unwrap_or_default();
                if trigger_node.is_none() {
                    content_src.extend(plain.iter().skip(1).copied());
                } else {
                    content_src.extend(plain.iter().copied());
                }

                // 锚转换:单子直接占槽 0;多子包 Row(槽 0,子槽 i)。
                let mut anchor = if anchor_src.len() == 1 {
                    path.push(0);
                    let v =
                        self.convert_node_tracked_ctx(anchor_src[0], path, id_map, probe, bindings);
                    path.pop();
                    v
                } else if anchor_src.is_empty() {
                    View::Empty
                } else {
                    let inner: Vec<View<DynamicMessage>> = self.convert_popover_items(
                        &anchor_src.iter().map(|n| (*n).clone()).collect::<Vec<_>>(),
                        0,
                        path,
                        id_map,
                        probe,
                        bindings,
                    );
                    View::Row {
                        children: inner,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    }
                };

                let open = open_prop.unwrap_or_else(self_managed_open);
                let on_dismiss = Some(
                    ondismiss_handler()
                        .map(|h| DynamicMessage::Typed {
                            widget_name: self.widget_name.clone(),
                            event_name: h.trim_start_matches('.').to_string(),
                            args: vec![],
                        })
                        .unwrap_or_else(|| DynamicMessage::Typed {
                            widget_name: self.widget_name.clone(),
                            event_name: "__popover_close".to_string(),
                            args: vec![],
                        }),
                );

                // 自管形态:默认 onclick(无 handler 时 convert_button 产出
                // String("click"))的 Button 锚注入内部 toggle(点锚开合)。
                // 已有真实 onclick 的锚保留用户语义(改用 open 属性驱动)。
                if !has_open_prop {
                    if let View::Button { onclick, .. } = &mut anchor {
                        let is_default = match onclick {
                            DynamicMessage::String(s) => s.is_empty() || s == "click",
                            DynamicMessage::Typed { event_name, .. } => event_name.is_empty(),
                        };
                        if is_default {
                            *onclick = internal_toggle;
                        }
                    }
                }

                let owned_content: Vec<AuraNode> =
                    content_src.iter().map(|n| (*n).clone()).collect();
                View::Popover {
                    anchor: PopoverAnchor::Widget(Box::new(anchor)),
                    content: Box::new(View::Column {
                        children: self.convert_popover_items(
                            &owned_content,
                            1,
                            path,
                            id_map,
                            probe,
                            bindings,
                        ),
                        spacing: 0,
                        padding: 0,
                        style: panel_style,
                        onclick: None,
                        on_right_click: None,
                    }),
                    placement,
                    open,
                    on_dismiss,
                }
            }
        }
    }

    /// Plan 422 P3: popover 面板子节点转换(slot 计数与 column 的
    /// "RESULTING slot" 约定一致 —— 视觉空节点出列,后续槽位左移)。
    pub(super) fn convert_popover_items(
        &self,
        children: &[AuraNode],
        base_slot: usize,
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> Vec<View<DynamicMessage>> {
        let mut items: Vec<View<DynamicMessage>> = Vec::new();
        let mut slot = 0usize;
        path.push(base_slot);
        for n in children {
            path.push(slot);
            let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
            path.pop();
            if is_visually_empty(&v) {
                continue;
            }
            items.push(v);
            slot += 1;
        }
        path.pop();
        items
    }

    /// Plan 418 P2-3: synthesize the toolbar from the action config
    /// (`toolbar {}` DSL tag) — icon buttons (PUA label scheme, same as
    /// convert_button's icon path) + separators; style from the tag merges on.
    pub(super) fn convert_toolbar_raw(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
        mut path: Option<(&[usize], &mut BuildProbe)>,
    ) -> View<DynamicMessage> {
        use crate::ui::action_config::{action_config, MenuItem};

        let empty = || View::Column {
            children: vec![],
            spacing: 0,
            padding: 0,
            style: None,
            onclick: None,
            on_right_click: None,
        };
        let Some(cfg) = action_config() else {
            return empty();
        };

        let mut children: Vec<View<DynamicMessage>> = Vec::new();
        for item in &cfg.toolbar {
            match item {
                MenuItem::Separator => children.push(self.convert_sep(&HashMap::new(), bindings)),
                MenuItem::Action(id) => {
                    let Some(a) = cfg.action_by_id(id) else {
                        continue;
                    };
                    let handler = a.handler.trim_start_matches('.').to_string();
                    // §8.4①: 按真实子位置记录(children.len() —— sep 也占
                    // 位;独立计数器会令 sep 之后的按钮索引全部前移错位)。
                    if let Some((base, probe)) = &mut path {
                        let mut child = base.to_vec();
                        child.push(children.len());
                        let p: Vec<u16> = child.iter().map(|&x| x as u16).collect();
                        probe.record_event(&p, "onclick", &a.handler);
                    }
                    let icon = a.icon.clone().unwrap_or_default();
                    let label = if icon.is_empty() {
                        a.title.clone()
                    } else {
                        // Icon-only: the title rides a third PUA marker (EE03)
                        // so the resting render is a bare svg; the renderer
                        // wraps it in an iced tooltip (shown on hover). The
                        // hover: classes give the icon square a rounded
                        // highlight box while hovered.
                        format!("\u{EE01}{}\u{EE02}\u{EE03}{}", icon, a.title)
                    };
                    children.push(View::Button {
                        // Plan 423 P3: enabled-if → disabled 态(表达式引擎)。
                        disabled: !a
                            .enabled_if
                            .as_deref()
                            .map(|cond| self.eval_condition_with(cond, bindings))
                            .unwrap_or(true),
                        label,
                        onclick: DynamicMessage::Typed {
                            widget_name: self.widget_name.clone(),
                            event_name: handler,
                            args: vec![],
                        },
                        style: Style::parse(
                            "h-7 w-7 px-0 py-0 hover:bg-zinc-700/50 hover:rounded-md",
                        )
                        .ok(),
                        on_right_click: None,
                        content: None,
                    });
                }
            }
        }

        let user = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .unwrap_or_default();
        View::Row {
            children,
            spacing: 0,
            padding: 0,
            style: if user.is_empty() {
                None
            } else {
                Style::parse(&user).ok()
            },
            onclick: None,
            on_right_click: None,
        }
    }

    pub(super) fn convert_sep(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let orientation = self
            .extract_string_with(props, "orientation", bindings)
            .unwrap_or_else(|| "vertical".to_owned());
        let (base, vertical) = if orientation == "horizontal" {
            ("w-full h-px bg-border", false)
        } else {
            ("w-7 h-7", true)
        };
        let mut style = Style::parse(base).ok();
        if let Some(user) = self.extract_style_with(props, bindings) {
            style = match style {
                Some(mut s) => {
                    s.classes.extend(user.classes);
                    Some(s)
                }
                None => Some(user),
            };
        }
        let line = ViewBuilder::col()
            .with_style(Style::parse("w-px h-4 bg-border").unwrap())
            .build();
        if !vertical {
            let mut b = ViewBuilder::col();
            if let Some(st) = style {
                b = b.with_style(st);
            }
            return b.build();
        }
        // Vertical (Plan 414 user spec): a w-7/h-7 box matching the
        // variant=icon buttons, 1px line construction-centered via the
        // Container center flags. Deliberately NOT a button: no pointer
        // cursor on hover, no onclick registration. Spacing is governed
        // purely by the parent row's gap.
        View::Container {
            child: Box::new(line),
            padding: 0,
            width: None,
            height: None,
            center_x: true,
            center_y: true,
            style,
            onclick: None,
            on_right_click: None,
        }
    }

    /// Convert an avatar element: colored circle placeholder.
    pub(super) fn convert_avatar(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // os-007 T4（P534-D4 根因修复）：centering 容器在样式无显式宽高时
        // 被 apply_container_style 设为 Fill×Fill，shrink 上下文（hovercard
        // 的 Popover 锚/行内）解析为零高——534 期触发器 hit area 零高的
        // 底层根因。对齐 vue 端 avatar 臂（w-10 h-10 rounded-full 恒注入）
        // 补默认尺寸；显式样式已带 w-*/h-* 时不重复注入。
        let has_size_class = |p: &Style| {
            p.classes
                .iter()
                .any(|c| matches!(c, StyleClass::Width(_) | StyleClass::Height(_)))
        };
        let style = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .filter(|s| !s.is_empty())
            .map(|s| {
                let needs_size = Style::parse(&s)
                    .map(|p| !has_size_class(&p))
                    .unwrap_or(true);
                let merged = if needs_size {
                    format!("{s} w-10 h-10")
                } else {
                    s
                };
                Style::parse(&merged).ok()
            })
            .unwrap_or_else(|| Style::parse("w-10 h-10 bg-gray-300 rounded-full").ok());

        // os-007（P534-D4）：有子件→avatar-image/avatar-fallback 子件组合
        // （经 convert_node_with 同表分发）；无子件保持灰圆占位（回归面）。
        let child = if !children.is_empty() {
            if children.len() == 1 {
                self.convert_node_with(&children[0], bindings)
            } else {
                let views: Vec<View<DynamicMessage>> = children
                    .iter()
                    .map(|n| self.convert_node_with(n, bindings))
                    .collect();
                View::Column {
                    children: views,
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                    on_right_click: None,
                }
            }
        } else {
            // os-007 T4：props 形态 desugar（对齐 vue 端 avatar 臂——
            // src→AvatarImage、fallback→AvatarFallback）；两 prop 皆无
            // 保持灰圆占位。
            let mut views: Vec<View<DynamicMessage>> = Vec::new();
            if let Some(src) = self
                .extract_string_with(props, "src", bindings)
                .filter(|s| !s.is_empty())
            {
                views.push(View::Image { src, style: None });
            }
            if let Some(fb) = self.extract_string_with(props, "fallback", bindings) {
                views.push(View::Text {
                    content: fb,
                    style: None,
                    selectable: false,
                });
            }
            match views.len() {
                0 => View::Text {
                    content: "".to_string(),
                    style: None,
                    selectable: false,
                },
                1 => views.into_iter().next().unwrap(),
                _ => View::Column {
                    children: views,
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                    on_right_click: None,
                },
            }
        };
        let mut builder = View::container(child);
        builder = builder.center_x().center_y();
        if let Some(s) = style {
            builder = builder.with_style(s);
        }
        builder.build()
    }

    // ========================================================================
    // Element converters
    // ========================================================================

    /// Convert a text element.
    ///
    /// Text content can come from:
    /// - A `text` or `content` prop
    /// - A child text node
    /// - The tag's main argument
    /// Plan 446 批五 U7: 文本类 tag 清单（children 折叠与元素转换共用）。
    pub(crate) const TEXT_LIKE_TAGS: &'static [&'static str] = &[
        "text", "label", "h1", "h2", "h3", "h4", "h5", "h6", "p", "span", "a", "link", "small",
        "strong", "em", "b", "i",
    ];

}


// ─────────────────────────────────────────────────────────────────────
// PLAN-045 T-04/T-05: memo 门包装层。原始转换体全部原样保留在 `*_raw`
// （非 memo 路径 = 包装直落 raw，行为逐字节一致——用户硬性要求）；包装
// 仅在 `memo: true`（provider 系为武装旗标）且扫描/上下文门通过时插缓存。
// ─────────────────────────────────────────────────────────────────────

impl<'a> AuraViewBuilder<'a> {
    /// menubar 声明式组件族门（`menubar { menubar-menu … }`）。
    pub(super) fn convert_menubar_component(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
        mut path: Option<(&[usize], &mut BuildProbe)>,
    ) -> View<DynamicMessage> {
        let probe_on = matches!(&path, Some((_, p)) if p.is_enabled());
        let gate = self.memo_gate_begin(
            Self::MEMO_SITE_MENUBAR_COMPONENT,
            props,
            children,
            bindings,
            probe_on,
            None,
            true,
        );
        let fill = match gate {
            MemoGateBegin::Hit {
                product,
                probe_replay,
                idmap_replay,
            } => {
                if let Some((_, probe)) = path.as_mut() {
                    probe.merge_entries(probe_replay);
                }
                let _ = idmap_replay; // menubar 无 id_map 面
                return product;
            }
            MemoGateBegin::Off => None,
            MemoGateBegin::Fill(f) => Some(f),
        };
        let raw_path = path.as_mut().map(|(b, p)| (*b, &mut **p));
        let (out, rec) = self.fill_with_recording(|| {
            let out = self.convert_menubar_component_raw(props, children, bindings, raw_path);
            // PLAN-047 T-05: 槽表达式在录制域内估值——静态扫描面全部读通
            // 道入 dep 集（未走分支的槽也入，保守）。
            if let Some(f) = &fill {
                let _ = self.memo_slots_fp(&f.slots, bindings);
            }
            out
        });
        match fill {
            Some(f) => {
                let snap = path
                    .as_ref()
                    .map(|(base, p)| {
                        let u16b: Vec<u16> = base.iter().map(|&x| x as u16).collect();
                        p.snapshot_prefix(&u16b)
                    })
                    .unwrap_or_default();
                self.memo_gate_store(f, bindings, out, snap, vec![], rec)
            }
            None => out,
        }
    }

    /// menubar actions DSL 门（空标签合成形态）。
    pub(super) fn convert_menubar(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
        mut path: Option<(&[usize], &mut BuildProbe)>,
    ) -> View<DynamicMessage> {
        let probe_on = matches!(&path, Some((_, p)) if p.is_enabled());
        let gate = self.memo_gate_begin(
            Self::MEMO_SITE_MENUBAR_DSL,
            props,
            &[],
            bindings,
            probe_on,
            None,
            true,
        );
        let fill = match gate {
            MemoGateBegin::Hit {
                product,
                probe_replay,
                idmap_replay,
            } => {
                if let Some((_, probe)) = path.as_mut() {
                    probe.merge_entries(probe_replay);
                }
                let _ = idmap_replay;
                return product;
            }
            MemoGateBegin::Off => None,
            MemoGateBegin::Fill(f) => Some(f),
        };
        let raw_path = path.as_mut().map(|(b, p)| (*b, &mut **p));
        let (out, rec) = self.fill_with_recording(|| {
            let out = self.convert_menubar_raw(props, bindings, raw_path);
            // PLAN-047 T-05: 槽表达式录制域内估值（同 menubar 组件门）。
            if let Some(f) = &fill {
                let _ = self.memo_slots_fp(&f.slots, bindings);
            }
            out
        });
        match fill {
            Some(f) => {
                let snap = path
                    .as_ref()
                    .map(|(base, p)| {
                        let u16b: Vec<u16> = base.iter().map(|&x| x as u16).collect();
                        p.snapshot_prefix(&u16b)
                    })
                    .unwrap_or_default();
                self.memo_gate_store(f, bindings, out, snap, vec![], rec)
            }
            None => out,
        }
    }

    /// toolbar actions DSL 门。
    pub(super) fn convert_toolbar(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
        mut path: Option<(&[usize], &mut BuildProbe)>,
    ) -> View<DynamicMessage> {
        let probe_on = matches!(&path, Some((_, p)) if p.is_enabled());
        let gate = self.memo_gate_begin(
            Self::MEMO_SITE_TOOLBAR,
            props,
            &[],
            bindings,
            probe_on,
            None,
            true,
        );
        let fill = match gate {
            MemoGateBegin::Hit {
                product,
                probe_replay,
                idmap_replay,
            } => {
                if let Some((_, probe)) = path.as_mut() {
                    probe.merge_entries(probe_replay);
                }
                let _ = idmap_replay;
                return product;
            }
            MemoGateBegin::Off => None,
            MemoGateBegin::Fill(f) => Some(f),
        };
        let raw_path = path.as_mut().map(|(b, p)| (*b, &mut **p));
        let (out, rec) = self.fill_with_recording(|| {
            let out = self.convert_toolbar_raw(props, bindings, raw_path);
            // PLAN-047 T-05: 槽表达式录制域内估值（同 menubar 组件门）。
            if let Some(f) = &fill {
                let _ = self.memo_slots_fp(&f.slots, bindings);
            }
            out
        });
        match fill {
            Some(f) => {
                let snap = path
                    .as_ref()
                    .map(|(base, p)| {
                        let u16b: Vec<u16> = base.iter().map(|&x| x as u16).collect();
                        p.snapshot_prefix(&u16b)
                    })
                    .unwrap_or_default();
                self.memo_gate_store(f, bindings, out, snap, vec![], rec)
            }
            None => out,
        }
    }

    /// dialog/dropdown/alert 模态族门（untracked 臂——无 probe 面）。
    pub(super) fn convert_alert_dialog(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
        family: ModalDialogFamily,
    ) -> View<DynamicMessage> {
        let gate = self.memo_gate_begin(
            Self::MEMO_SITE_DIALOG_FAMILY,
            props,
            children,
            bindings,
            false,
            None,
            true,
        );
        let fill = match gate {
            MemoGateBegin::Hit {
                product,
                probe_replay,
                idmap_replay,
            } => {
                let _ = (probe_replay, idmap_replay);
                return product;
            }
            MemoGateBegin::Off => None,
            MemoGateBegin::Fill(f) => Some(f),
        };
        let (out, rec) = self.fill_with_recording(|| {
            let out = self.convert_alert_dialog_raw(props, children, bindings, family);
            // PLAN-047 T-05: 槽表达式录制域内估值（同 menubar 组件门）。
            if let Some(f) = &fill {
                let _ = self.memo_slots_fp(&f.slots, bindings);
            }
            out
        });
        match fill {
            Some(f) => self.memo_gate_store(f, bindings, out, vec![], vec![], rec),
            None => out,
        }
    }

    /// dialog/dropdown/alert 模态族门（tracked 臂——probe + id_map 双重放）。
    pub(super) fn convert_alert_dialog_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
        family: ModalDialogFamily,
    ) -> View<DynamicMessage> {
        let gate = self.memo_gate_begin(
            Self::MEMO_SITE_DIALOG_FAMILY,
            props,
            children,
            bindings,
            probe.is_enabled(),
            None,
            true,
        );
        let fill = match gate {
            MemoGateBegin::Hit {
                product,
                probe_replay,
                idmap_replay,
            } => {
                probe.merge_entries(probe_replay);
                id_map.merge_entries(idmap_replay);
                return product;
            }
            MemoGateBegin::Off => None,
            MemoGateBegin::Fill(f) => Some(f),
        };
        let (out, rec) = self.fill_with_recording(|| {
            let out = self.convert_alert_dialog_tracked_ctx_raw(
                props, children, path, id_map, probe, bindings, family,
            );
            // PLAN-047 T-05: 槽表达式录制域内估值（同 menubar 组件门）。
            if let Some(f) = &fill {
                let _ = self.memo_slots_fp(&f.slots, bindings);
            }
            out
        });
        match fill {
            Some(f) => {
                let u16b: Vec<u16> = path.iter().map(|&x| x as u16).collect();
                let probe_snap = probe.snapshot_prefix(&u16b);
                let idmap_snap = id_map.snapshot_prefix(path);
                self.memo_gate_store(f, bindings, out, probe_snap, idmap_snap, rec)
            }
            None => out,
        }
    }

    /// sidebar provider 级武装（`sidebar_provider (memo: true)`）——子树内
    /// `sidebar_group` 臂凭旗标入 nav memo 门；provider 本体不建条目
    /// （passthrough 便宜，组级条目才是收益粒度，PLAN-045 §3/Q-02）。
    pub(super) fn convert_sidebar_provider(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let armed = Self::memo_prop_on(props) && self.memo_ctx_ok(bindings);
        if !armed {
            return self.convert_sidebar_provider_raw(props, events, children, bindings);
        }
        self.nav_memo_armed.set(true);
        let out = self.convert_sidebar_provider_raw(props, events, children, bindings);
        self.nav_memo_armed.set(false);
        out
    }

    /// sidebar_group nav 块门（PLAN-045 §3：nav 块粒度派生值键——路由变化
    /// 仅 active 翻转组失效，其余组全命中）。
    pub(super) fn convert_sidebar_group(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let gate = if self.nav_memo_armed.get() {
            let extra = self.memo_nav_group_extra_dyn(props, children, bindings);
            self.memo_gate_begin(
                Self::MEMO_SITE_SIDEBAR_GROUP,
                props,
                children,
                bindings,
                false,
                Some(extra),
                false,
            )
        } else {
            MemoGateBegin::Off
        };
        let fill = match gate {
            MemoGateBegin::Hit {
                product,
                probe_replay,
                idmap_replay,
            } => {
                let _ = (probe_replay, idmap_replay);
                return product;
            }
            MemoGateBegin::Off => None,
            MemoGateBegin::Fill(f) => Some(f),
        };
        let (out, rec) = self.fill_with_recording(|| {
            let out = self.convert_sidebar_group_raw(props, children, bindings);
            // PLAN-047 T-05: 槽表达式录制域内估值（extra_dyn 派生面不走
            // version_fast，录集仍服务 fp_slow 基线刷新一致性）。
            if let Some(f) = &fill {
                let _ = self.memo_slots_fp(&f.slots, bindings);
            }
            out
        });
        match fill {
            Some(f) => self.memo_gate_store(f, bindings, out, vec![], vec![], rec),
            None => out,
        }
    }
}
