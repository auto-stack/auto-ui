//! PLAN-014 T-05：nav/sidebar 责任（nav_route_active/nav_item/nav_group/sidebar 家族）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {
    /// Evaluate a bool prop that may be a literal (`true`) or a bound
    /// expression (`.cur == "chats"`); literals first, exprs via the standard
    /// resolve_expr_to_value channel.
    pub(super) fn extract_bool_expr(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<bool> {
        match props.get(key)? {
            AuraPropValue::Expr(Expr::Bool(b)) => Some(*b),
            AuraPropValue::Expr(expr) => self
                .resolve_expr_to_value(expr, bindings)
                .map(|v| v.as_bool()),
            _ => None,
        }
    }

    /// Plan 482: is `to` the current route? Exact match, or prefix-segment
    /// match ("/chats" is active on "/chats/1") unless `exact` tightens it.
    pub(super) fn nav_route_active(&self, to: &str, exact: bool) -> bool {
        if to.is_empty() {
            return false;
        }
        let current = match self.read_state("__current_route") {
            Ok(auto_val::Value::Str(s)) => s.to_string(),
            _ => return false,
        };
        if current == to {
            return true;
        }
        if exact {
            return false;
        }
        let prefix = format!("{}/", to.trim_end_matches('/'));
        current == prefix.trim_end_matches('/') || current.starts_with(&prefix)
    }

    /// Lucide-name heuristic: kebab-case lower alphanumeric (`message-square`).
    /// Emoji / CJK / arbitrary text falls through to the text path.
    pub(super) fn is_lucide_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 48
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && name.starts_with(|c: char| c.is_ascii_lowercase())
    }

    /// Plan 482: `nav-item` → View::Button with contract classes. Router mode
    /// (`to:`) dispatches `__navigate` (same interception as link/nav-link);
    /// state mode (`onclick:`) carries the user message. Active styling is
    /// decided at build time (explicit `active:` expr > route auto-detect) and
    /// REPLACES the hover classes — mirroring NavItem.vue so hover can never
    /// override the selected background on either end.
    pub(super) fn convert_nav_item(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use auto_ui_compiler::ui_gen::nav_contract as nc;

        let to = self
            .extract_string_with(props, "to", bindings)
            .unwrap_or_default();
        let label = self
            .extract_string_with(props, "label", bindings)
            .or_else(|| self.extract_string_with(props, "text", bindings))
            .unwrap_or_default();
        let desc = self
            .extract_string_with(props, "desc", bindings)
            .unwrap_or_default();
        let badge = self
            .extract_string_with(props, "badge", bindings)
            .unwrap_or_default();
        let icon = self
            .extract_string_with(props, "icon", bindings)
            .unwrap_or_default();
        let size = self
            .extract_string_with(props, "size", bindings)
            .unwrap_or_default();
        let disabled = self.extract_bool(props, "disabled").unwrap_or(false)
            || self
                .extract_bool_expr(props, "disabled", bindings)
                .unwrap_or(false);
        let exact = self.extract_bool(props, "exact").unwrap_or(false);

        let active = self
            .extract_bool_expr(props, "active", bindings)
            .unwrap_or_else(|| self.nav_route_active(&to, exact));

        // Click target: `to` wins (router mode); else the user message; else a
        // graceful no-op message (renderer intercepts "__noop").
        let onclick = if !to.is_empty() {
            crate::ui::interpreter::DynamicMessage::Typed {
                widget_name: self.widget_name.clone(),
                event_name: "__navigate".to_string(),
                args: vec![auto_val::Value::str(&to)],
            }
        } else {
            match aura_events_get_base(events, "onclick")
                .or_else(|| aura_events_get_base(events, "click"))
            {
                Some(event) => self.event_to_message_with(&event, bindings),
                None => crate::ui::interpreter::DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: "__noop".to_string(),
                    args: Vec::new(),
                },
            }
        };

        // Content: explicit children win (vue slot parity); else synthesize
        // icon | texts(label+desc) | badge from the props.
        let content: Option<Box<View<DynamicMessage>>> = if !children.is_empty() {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .filter(|v| !matches!(v, View::Empty))
                .collect();
            if views.is_empty() {
                None
            } else if views.len() == 1 {
                Some(Box::new(views.into_iter().next().unwrap()))
            } else {
                Some(Box::new(View::Row {
                    children: views,
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                    on_right_click: None,
                }))
            }
        } else {
            let is_lg = size == "lg";
            let mut parts: Vec<View<DynamicMessage>> = Vec::new();
            if !icon.is_empty() {
                if Self::is_lucide_name(&icon) {
                    let icon_cls = if is_lg { nc::ICON_LG } else { nc::ICON_MD };
                    parts.push(View::Image {
                        src: format!("lucide:{}", icon),
                        style: Style::parse(icon_cls).ok(),
                    });
                } else {
                    // Emoji / literal text icon.
                    parts.push(View::Text {
                        content: icon,
                        style: None,
                        selectable: false,
                    });
                }
            }
            if !label.is_empty() || !desc.is_empty() {
                let mut texts: Vec<View<DynamicMessage>> = Vec::new();
                if !label.is_empty() {
                    texts.push(View::Text {
                        content: label.clone(),
                        style: None,
                        selectable: false,
                    });
                }
                if !desc.is_empty() {
                    texts.push(View::Text {
                        content: desc,
                        style: Style::parse(nc::TEXT_DESC).ok(),
                        selectable: false,
                    });
                }
                // flex-1 pushes the badge to the row's trailing edge (iced
                // Fill) — same geometry as NavItem.vue's flex-1 min-w-0.
                let texts_cls = if badge.is_empty() {
                    "flex flex-col".to_string()
                } else {
                    format!("flex flex-col {}", nc::TEXTS_FILL)
                };
                parts.push(View::Column {
                    children: texts,
                    spacing: 0,
                    padding: 0,
                    style: Style::parse(&texts_cls).ok(),
                    onclick: None,
                    on_right_click: None,
                });
            }
            if !badge.is_empty() {
                parts.push(View::Text {
                    content: badge,
                    style: Style::parse(nc::BADGE_PILL).ok(),
                    selectable: false,
                });
            }
            if parts.is_empty() {
                None
            } else {
                // Gap/alignment on the content row matches the base class
                // (gap-2 items-center / lg: gap-3 items-start).
                let row_cls = if is_lg {
                    "items-start gap-3"
                } else {
                    "items-center gap-2"
                };
                Some(Box::new(View::Row {
                    children: parts,
                    spacing: 0,
                    padding: 0,
                    style: Style::parse(row_cls).ok(),
                    onclick: None,
                    on_right_click: None,
                }))
            }
        };

        // Snapshot/accessibility label: explicit label, else children text,
        // else the route path.
        let label = if !label.is_empty() {
            label
        } else if let Some(derived) = self.extract_children_text(children, bindings) {
            derived
        } else {
            to.clone()
        };

        // Contract class assembly — base + state (+ user append last).
        let base = match size.as_str() {
            "sm" => nc::ITEM_BASE_SM,
            "lg" => nc::ITEM_BASE_LG,
            _ => nc::ITEM_BASE_MD,
        };
        let mut class = base.to_string();
        if active {
            class.push(' ');
            class.push_str(nc::ITEM_ACTIVE);
        } else if !disabled {
            class.push(' ');
            class.push_str(nc::ITEM_HOVER);
        }
        if disabled {
            class.push(' ');
            class.push_str(nc::ITEM_DISABLED);
        }
        if let Some(user) = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
        {
            if !user.trim().is_empty() {
                class.push(' ');
                class.push_str(user.trim());
            }
        }

        View::Button {
            disabled,
            label,
            content,
            onclick,
            style: Style::parse(&class).ok(),
            on_right_click: None,
        }
    }

    /// Plan 482: `nav-group` → header (label text or collapsible toggle
    /// button) + member column. Open state: bound `open:` expr wins; unbound
    /// collapsible groups use the builder-fed `nav_group_states` map (toggled
    /// via the `__nav_toggle` internal message; default open).
    pub(super) fn convert_nav_group(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use auto_ui_compiler::ui_gen::nav_contract as nc;

        let label = self
            .extract_string_with(props, "label", bindings)
            .or_else(|| self.extract_string_with(props, "text", bindings))
            .unwrap_or_default();
        let collapsible = self.extract_bool(props, "collapsible").unwrap_or(false)
            || self
                .extract_bool_expr(props, "collapsible", bindings)
                .unwrap_or(false);
        let indent = self.extract_bool(props, "indent").unwrap_or(false);

        let key = format!("__nav_group_open:{}", label);
        let open = self
            .extract_bool_expr(props, "open", bindings)
            .unwrap_or_else(|| {
                self.nav_group_states
                    .as_ref()
                    .and_then(|m| m.get(&key).copied())
                    .unwrap_or(true)
            });

        // Header.
        let header = if collapsible {
            let onclick = match aura_events_get_base(events, "ontoggle")
                .or_else(|| aura_events_get_base(events, "toggle"))
            {
                Some(event) => self.event_to_message_with(&event, bindings),
                None => crate::ui::interpreter::DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: "__nav_toggle".to_string(),
                    args: vec![auto_val::Value::str(&key)],
                },
            };
            // Plan 482:文本 chevron(▾/▸)——与非 shadcn vue 内联轨同构。
            let chevron = if open { "▾" } else { "▸" };
            let content = Box::new(View::Row {
                children: vec![
                    View::Text {
                        content: chevron.to_string(),
                        style: Style::parse("text-[11px] text-muted-foreground w-[14px] shrink-0")
                            .ok(),
                        selectable: false,
                    },
                    View::Text {
                        content: label.clone(),
                        style: None,
                        selectable: false,
                    },
                ],
                spacing: 0,
                padding: 0,
                style: Style::parse("items-center gap-2").ok(),
                onclick: None,
                on_right_click: None,
            });
            let mut class = format!("{} {}", nc::GROUP_TOGGLE, nc::GROUP_TOGGLE_HOVER);
            if let Some(user) = self
                .extract_string_with(props, "class", bindings)
                .or_else(|| self.extract_string_with(props, "style", bindings))
            {
                if !user.trim().is_empty() {
                    class.push(' ');
                    class.push_str(user.trim());
                }
            }
            View::Button {
                disabled: false,
                label,
                content: Some(content),
                onclick,
                style: Style::parse(&class).ok(),
                on_right_click: None,
            }
        } else {
            View::Text {
                content: label,
                style: Style::parse(nc::GROUP_LABEL).ok(),
                selectable: false,
            }
        };

        // Member column — converted only when open (state flip rebuilds).
        let mut views: Vec<View<DynamicMessage>> = vec![header];
        if open {
            let member: Vec<View<DynamicMessage>> = children
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .filter(|v| !matches!(v, View::Empty))
                .collect();
            if !member.is_empty() {
                let content_cls = if indent {
                    format!("{} {}", nc::GROUP_CONTENT, nc::GROUP_CONTENT_INDENT)
                } else {
                    nc::GROUP_CONTENT.to_string()
                };
                views.push(View::Column {
                    children: member,
                    spacing: 0,
                    padding: 0,
                    style: Style::parse(&content_cls).ok(),
                    onclick: None,
                    on_right_click: None,
                });
            }
        }

        View::Column {
            children: views,
            spacing: 0,
            padding: 0,
            style: Style::parse("flex flex-col").ok(),
            onclick: None,
            on_right_click: None,
        }
    }

    /// Plan 482: the integrated search row for `nav (search: true)` — icon +
    /// bound input, classes from the contract (SEARCH_ROW/SEARCH_INPUT).
    pub(super) fn convert_nav_search_row(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use auto_ui_compiler::ui_gen::nav_contract as nc;

        let placeholder = self
            .extract_string_with(props, "search_placeholder", bindings)
            .unwrap_or_else(|| "Search...".to_string());
        let value = self
            .extract_string_with(props, "search_value", bindings)
            .unwrap_or_default();

        let mut input = View::<DynamicMessage>::input(placeholder).value(value);
        if let Some(event) = aura_events_get_base(events, "onsearch")
            .or_else(|| aura_events_get_base(events, "oninput"))
        {
            input = input.on_change(self.event_to_message_with(&event, bindings));
        }
        if let Ok(s) = Style::parse(nc::SEARCH_INPUT) {
            input = input.with_style(s);
        }

        View::Row {
            children: vec![
                // 🔍 文本字形——与非 shadcn vue 轨同构（免 lucide 集依赖）。
                View::Text {
                    content: "🔍".to_string(),
                    style: Style::parse(&format!("{} text-muted-foreground", nc::ICON_MD)).ok(),
                    selectable: false,
                },
                input.build(),
            ],
            spacing: 0,
            padding: 0,
            style: Style::parse(nc::SEARCH_ROW).ok(),
            onclick: None,
            on_right_click: None,
        }
    }

    /// Plan 482: `nav` container — when `search: true`, prepend the integrated
    /// search row to the children; otherwise plain container (aside-family).
    pub(super) fn convert_nav_container(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let search = self.extract_bool(props, "search").unwrap_or(false)
            || self
                .extract_bool_expr(props, "search", bindings)
                .unwrap_or(false);
        if !search {
            return self.convert_container(props, children, bindings);
        }
        let search_row = self.convert_nav_search_row(props, events, bindings);
        let mut child_views: Vec<View<DynamicMessage>> = vec![search_row];
        for n in children {
            let v = self.convert_node_with(n, bindings);
            if !matches!(v, View::Empty) {
                child_views.push(v);
            }
        }
        // Keep the container's own style semantics (flex-col etc. come from
        // the user's style prop, default mirrors aside usage).
        let style = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .unwrap_or_else(|| "flex flex-col gap-1".to_string());
        View::Column {
            children: child_views,
            spacing: 0,
            padding: 0,
            style: Style::parse(&style).ok(),
            onclick: None,
            on_right_click: None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Plan 561: sidebar_* —— VM 端契约子集（设计 sidebar-family-and-nav-
    // retirement §3.3 D3）。契约类唯一来源 ui_gen::sidebar_contract（单测
    // 逐 token 锚定 shadcn 原版资产，双端不可漂移）。结构等价验收，不追求
    // 像素级复刻 web；offcanvas 动画、icon-collapse、Rail 拖拽、tooltip、
    // cookie/快捷键/mobile 均不在子集。dispatch 双站（tracked/untracked）
    // 镜像，实现集中在本节（同 nav 先例）。
    // ─────────────────────────────────────────────────────────────────────

    /// `sidebar_provider`：open 状态容器（web 端无视觉盒，VM 同样透明）。
    /// 受控 `open:` 或 `default_open:` 求值后注入直连 `sidebar` 子节点
    /// （sidebar 自身 schema 无 open prop——状态归 provider，shadcn 同构）；
    /// 显式写了 `open:` 的 sidebar 不被覆盖。
    pub(super) fn convert_sidebar_provider_raw(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let _ = events;
        let open = self
            .extract_bool_expr(props, "open", bindings)
            .or_else(|| self.extract_bool(props, "open"))
            .or_else(|| self.extract_bool_expr(props, "default_open", bindings))
            .or_else(|| self.extract_bool(props, "default_open"))
            .unwrap_or(true);
        let rewritten: Vec<AuraNode> = children
            .iter()
            .map(|n| match n {
                AuraNode::Element {
                    tag,
                    props: cp,
                    events: ce,
                    children: cc,
                    span,
                    debug_id,
                } if tag == "sidebar" && !cp.contains_key("open") => {
                    let mut cp2 = cp.clone();
                    cp2.insert(
                        "open".to_string(),
                        AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(open)),
                    );
                    AuraNode::Element {
                        tag: tag.clone(),
                        props: cp2,
                        events: ce.clone(),
                        children: cc.clone(),
                        span: *span,
                        debug_id: *debug_id,
                    }
                }
                _ => n.clone(),
            })
            .collect();
        // PLAN-012 F2：provider 契约形态 = 弹性填充列（web class
        // `flex-1 min-h-0 flex flex-col`）——vm 臂必须显式给合列
        // Height(Fill)：透明 passthrough 丢样式后 provider 无界，内层
        // sidebar_content 的 Scrollable 拿不到有界高度 → 永不出滚动条，
        // 矮窗口下侧栏整段溢出裁剪（顶部项消失，用户实机 09-12）。
        self.convert_children_passthrough_styled(
            &rewritten,
            bindings,
            "h-full w-full flex flex-col",
        )
    }

    /// 透明容器：独子直返，多子合列（provider/menu_item/sub_item 共用）。
    pub(super) fn convert_children_passthrough(
        &self,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        self.convert_children_passthrough_styled(children, bindings, "")
    }

    /// 同上带合列样式变体（style 非空时多子合列消费；独子直返不包——
    /// 调用方需弹性填充时自选）。
    pub(super) fn convert_children_passthrough_styled(
        &self,
        children: &[AuraNode],
        bindings: &Bindings,
        column_style: &str,
    ) -> View<DynamicMessage> {
        let views: Vec<View<DynamicMessage>> = children
            .iter()
            .map(|n| self.convert_node_with(n, bindings))
            .filter(|v| !matches!(v, View::Empty))
            .collect();
        if views.len() == 1 {
            views.into_iter().next().unwrap()
        } else if views.is_empty() {
            View::Empty
        } else {
            View::Column {
                children: views,
                spacing: 0,
                padding: 0,
                style: if column_style.is_empty() {
                    None
                } else {
                    crate::ui::style::Style::parse(column_style).ok()
                },
                onclick: None,
                on_right_click: None,
            }
        }
    }

    /// `sidebar` 根容器：契约基座 + VM 宽度（w-64 = web `--sidebar-width:16rem`
    /// 等价）。`open:`（provider 注入或显式）驱动 offcanvas 桌面显隐——
    /// 关且 collapsible != "none" 时整树不渲染（滑出动画不做）；`icon`
    /// 模式不在 VM 子集，按展开处理。variant="floating" 加圆角描边；
    /// side="right" 的贴边由父级排布承担（VM 无 fixed 层）。
    pub(super) fn convert_sidebar_root(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use auto_ui_compiler::ui_gen::sidebar_contract as sc;
        let open = self
            .extract_bool_expr(props, "open", bindings)
            .or_else(|| self.extract_bool(props, "open"))
            .unwrap_or(true);
        let collapsible = self
            .extract_string_with(props, "collapsible", bindings)
            .unwrap_or_else(|| "offcanvas".to_string());
        if !open && collapsible != "none" {
            return View::Empty;
        }
        let variant = self
            .extract_string_with(props, "variant", bindings)
            .unwrap_or_else(|| "sidebar".to_string());
        let mut class = format!("{} {}", sc::SIDEBAR_BASE, sc::WIDTH_VM);
        if variant == "floating" {
            class.push(' ');
            class.push_str(sc::VARIANT_FLOATING_VM);
        }
        let p = self.with_class_prop(props, bindings, &class);
        self.convert_column(&p, children, bindings)
    }

    /// sidebar_header / sidebar_footer 分区（契约同串，列容器）。
    pub(super) fn convert_sidebar_region(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
        base: &str,
    ) -> View<DynamicMessage> {
        let p = self.with_class_prop(props, bindings, base);
        self.convert_column(&p, children, bindings)
    }

    /// sidebar_content：可滚动主区（flex-1 + overflow-auto → scroll 臂）。
    pub(super) fn convert_sidebar_content(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let p = self.with_class_prop(
            props,
            bindings,
            auto_ui_compiler::ui_gen::sidebar_contract::CONTENT_BASE,
        );
        let no_events: HashMap<String, AuraEvent> = HashMap::new();
        self.convert_scroll(&p, &no_events, children, bindings)
    }

    /// sidebar_separator：细分隔线（h-px 为 VM 显式高度补全，web 端由
    /// Separator 组件自带）。
    pub(super) fn convert_sidebar_separator(&self) -> View<DynamicMessage> {
        View::Container {
            child: Box::new(View::Empty),
            padding: 0,
            width: None,
            height: None,
            center_x: false,
            center_y: false,
            onclick: None,
            on_right_click: None,
            style: Style::parse(&format!(
                "h-px {}",
                auto_ui_compiler::ui_gen::sidebar_contract::SEPARATOR
            ))
            .ok(),
        }
    }

    /// `sidebar_group`：分组容器。`collapsible: true` 时首个
    /// sidebar_group_label 子节点（或 `label:` prop）渲染为开合按钮
    /// （内部 `__nav_toggle` 消息 + nav_group_states 通道复用，键名
    /// `__sidebar_group_open:<label>` 命名空间隔离），其余子节点收起不渲染。
    pub(super) fn convert_sidebar_group_raw(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        use auto_ui_compiler::ui_gen::sidebar_contract as sc;
        let collapsible = self.extract_bool(props, "collapsible").unwrap_or(false)
            || self
                .extract_bool_expr(props, "collapsible", bindings)
                .unwrap_or(false);
        let p = self.with_class_prop(props, bindings, sc::GROUP_BASE);
        if !collapsible {
            return self.convert_column(&p, children, bindings);
        }
        // 可折叠：定位 label（首个 sidebar_group_label 子节点的文本，或 label: prop）。
        let label_text = self
            .extract_string_with(props, "label", bindings)
            .or_else(|| {
                children.iter().find_map(|n| match n {
                    AuraNode::Element {
                        tag,
                        props: lp,
                        children: lc,
                        ..
                    } if tag == "sidebar_group_label" || tag == "sidebar-group-label" => self
                        .extract_string_with(lp, "text", bindings)
                        .or_else(|| self.extract_children_text(lc, bindings)),
                    _ => None,
                })
            })
            .unwrap_or_default();
        let key = format!("__sidebar_group_open:{}", label_text);
        let open = self
            .extract_bool_expr(props, "open", bindings)
            .unwrap_or_else(|| {
                self.nav_group_states
                    .as_ref()
                    .and_then(|m| m.get(&key).copied())
                    .unwrap_or(true)
            });
        let toggle = View::Button {
            disabled: false,
            label: label_text.clone(),
            content: Some(Box::new(View::Row {
                children: vec![
                    View::Text {
                        content: label_text,
                        style: None,
                        selectable: false,
                    },
                    View::Text {
                        content: if open {
                            "▾".to_string()
                        } else {
                            "▸".to_string()
                        },
                        style: Style::parse("ml-auto text-muted-foreground").ok(),
                        selectable: false,
                    },
                ],
                spacing: 0,
                padding: 0,
                style: Style::parse("w-full items-center").ok(),
                onclick: None,
                on_right_click: None,
            })),
            onclick: crate::ui::interpreter::DynamicMessage::Typed {
                widget_name: self.widget_name.clone(),
                event_name: "__nav_toggle".to_string(),
                args: vec![auto_val::Value::str(&key)],
            },
            style: Style::parse(&format!("{} w-full text-left", sc::GROUP_LABEL)).ok(),
            on_right_click: None,
        };
        let mut views: Vec<View<DynamicMessage>> = vec![toggle];
        if open {
            for n in children {
                if matches!(n, AuraNode::Element { tag, .. } if tag == "sidebar_group_label" || tag == "sidebar-group-label")
                {
                    continue; // label 已进 toggle
                }
                let v = self.convert_node_with(n, bindings);
                if !matches!(v, View::Empty) {
                    views.push(v);
                }
            }
        }
        let user_class = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .unwrap_or_default();
        let class = if user_class.trim().is_empty() {
            sc::GROUP_BASE.to_string()
        } else {
            format!("{} {}", user_class.trim(), sc::GROUP_BASE)
        };
        View::Column {
            children: views,
            spacing: 0,
            padding: 0,
            style: Style::parse(&class).ok(),
            onclick: None,
            on_right_click: None,
        }
    }

    /// sidebar_group_label 独立用法（非折叠组内）：文本 + 契约类。
    pub(super) fn convert_sidebar_group_label(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let p = self.with_class_prop(
            props,
            bindings,
            auto_ui_compiler::ui_gen::sidebar_contract::GROUP_LABEL,
        );
        self.convert_text_element(tag, &p, events, children, bindings)
    }

    /// sidebar_group_action：组头右上动作按钮。
    pub(super) fn convert_sidebar_group_action(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        self.convert_sidebar_action_button(
            props,
            events,
            children,
            bindings,
            auto_ui_compiler::ui_gen::sidebar_contract::GROUP_ACTION,
        )
    }

    /// sidebar_menu_action：行内动作按钮（与 group_action 同构，类不同）。
    pub(super) fn convert_sidebar_menu_action(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        self.convert_sidebar_action_button(
            props,
            events,
            children,
            bindings,
            auto_ui_compiler::ui_gen::sidebar_contract::MENU_ACTION,
        )
    }

    /// 动作按钮共用实现：children 优先，否则 text:/icon: 合成；onclick 缺省 __noop。
    pub(super) fn convert_sidebar_action_button(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
        base: &str,
    ) -> View<DynamicMessage> {
        let text = self
            .extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_string_with(props, "label", bindings))
            .unwrap_or_default();
        let content: Option<Box<View<DynamicMessage>>> = if !children.is_empty() {
            let v = self.convert_children_passthrough(children, bindings);
            Some(Box::new(v))
        } else if !text.is_empty() {
            Some(Box::new(View::Text {
                content: text.clone(),
                style: None,
                selectable: false,
            }))
        } else {
            None
        };
        let onclick = match aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "click"))
        {
            Some(event) => self.event_to_message_with(&event, bindings),
            None => crate::ui::interpreter::DynamicMessage::Typed {
                widget_name: self.widget_name.clone(),
                event_name: "__noop".to_string(),
                args: Vec::new(),
            },
        };
        let mut class = base.to_string();
        if let Some(user) = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
        {
            if !user.trim().is_empty() {
                class.push(' ');
                class.push_str(user.trim());
            }
        }
        let label = if !text.is_empty() {
            text
        } else {
            self.extract_children_text(children, bindings)
                .unwrap_or_default()
        };
        View::Button {
            disabled: false,
            label,
            content,
            onclick,
            style: Style::parse(&class).ok(),
            on_right_click: None,
        }
    }

    /// sidebar_menu_badge：行尾徽标文本。
    pub(super) fn convert_sidebar_menu_badge(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let text = self
            .extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_children_text(children, bindings))
            .unwrap_or_default();
        let mut class = auto_ui_compiler::ui_gen::sidebar_contract::MENU_BADGE.to_string();
        if let Some(user) = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
        {
            if !user.trim().is_empty() {
                class.push(' ');
                class.push_str(user.trim());
            }
        }
        View::Text {
            content: text,
            style: Style::parse(&class).ok(),
            selectable: false,
        }
    }

    /// `sidebar_menu_button` / `sidebar_menu_sub_button`（sub=true）——契约
    /// 三态 + `to:` 路由双模式（Plan 548 D2 的 VM 半）。active 决定序：
    /// 显式 `active:` 表达式 > `to:` 路由自动探测（exact/前缀段，同
    /// nav-item）；active 态整串替换 hover（双端 either/or，hover 永不
    /// 压过选中底）。`to:` 非空时 dispatch `__navigate`（route/mod.rs
    /// 机制，`__current_route` + 历史栈），优先于 onclick。
    pub(super) fn convert_sidebar_menu_button(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
        sub: bool,
    ) -> View<DynamicMessage> {
        use auto_ui_compiler::ui_gen::sidebar_contract as sc;

        let to = self
            .extract_string_with(props, "to", bindings)
            .unwrap_or_default();
        let text = self
            .extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_string_with(props, "label", bindings))
            .unwrap_or_default();
        let icon = self
            .extract_string_with(props, "icon", bindings)
            .unwrap_or_default();
        let size = self
            .extract_string_with(props, "size", bindings)
            .unwrap_or_default();
        let variant = self
            .extract_string_with(props, "variant", bindings)
            .unwrap_or_default();
        let disabled = self.extract_bool(props, "disabled").unwrap_or(false)
            || self
                .extract_bool_expr(props, "disabled", bindings)
                .unwrap_or(false);
        let exact = self.extract_bool(props, "exact").unwrap_or(false);

        let active = self
            .extract_bool_expr(props, "active", bindings)
            .unwrap_or_else(|| self.nav_route_active(&to, exact));

        // 点击目标：to 优先（路由模式）；否则用户消息；否则 __noop。
        let onclick = if !to.is_empty() {
            crate::ui::interpreter::DynamicMessage::Typed {
                widget_name: self.widget_name.clone(),
                event_name: "__navigate".to_string(),
                args: vec![auto_val::Value::str(&to)],
            }
        } else {
            match aura_events_get_base(events, "onclick")
                .or_else(|| aura_events_get_base(events, "click"))
            {
                Some(event) => self.event_to_message_with(&event, bindings),
                None => crate::ui::interpreter::DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: "__noop".to_string(),
                    args: Vec::new(),
                },
            }
        };

        // 内容：显式 children 优先（多子合为契约行——基座 flex row gap-2
        // 的 VM 同构，icon+text 横排；竖列会在 h-8 定高下裁掉文本）；
        // 否则 icon + text 合成行。
        let content: Option<Box<View<DynamicMessage>>> = if !children.is_empty() {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .filter(|v| !matches!(v, View::Empty))
                .collect();
            if views.len() == 1 {
                Some(Box::new(views.into_iter().next().unwrap()))
            } else if views.is_empty() {
                None
            } else {
                Some(Box::new(View::Row {
                    children: views,
                    spacing: 0,
                    padding: 0,
                    style: Style::parse("items-center gap-2").ok(),
                    onclick: None,
                    on_right_click: None,
                }))
            }
        } else {
            let mut parts: Vec<View<DynamicMessage>> = Vec::new();
            if !icon.is_empty() {
                if Self::is_lucide_name(&icon) {
                    parts.push(View::Image {
                        src: format!("lucide:{}", icon),
                        style: Style::parse("h-4 w-4 shrink-0").ok(),
                    });
                } else {
                    parts.push(View::Text {
                        content: icon,
                        style: None,
                        selectable: false,
                    });
                }
            }
            if !text.is_empty() {
                parts.push(View::Text {
                    content: text.clone(),
                    style: None,
                    selectable: false,
                });
            }
            if parts.is_empty() {
                None
            } else {
                Some(Box::new(View::Row {
                    children: parts,
                    spacing: 0,
                    padding: 0,
                    style: Style::parse("items-center gap-2").ok(),
                    onclick: None,
                    on_right_click: None,
                }))
            }
        };

        // 契约类装配：基座 + 尺寸 + 变体 + 状态（+ 用户追加最后）。
        let mut class = if sub {
            sc::MENU_SUB_BUTTON_BASE.to_string()
        } else {
            sc::MENU_BUTTON_BASE.to_string()
        };
        let size_cls = match (sub, size.as_str()) {
            (true, "sm") => Some(sc::MENU_SUB_BUTTON_SIZE_SM),
            (false, "sm") => Some(sc::MENU_BUTTON_SIZE_SM),
            (false, "lg") => Some(sc::MENU_BUTTON_SIZE_LG),
            (false, _) => Some(sc::MENU_BUTTON_SIZE_DEFAULT),
            (true, _) => None, // sub md 的 text-sm 已在基座
        };
        if let Some(c) = size_cls {
            class.push(' ');
            class.push_str(c);
        }
        if !sub && variant == "outline" {
            class.push(' ');
            class.push_str(sc::MENU_BUTTON_OUTLINE_VM);
        }
        if active {
            class.push(' ');
            class.push_str(sc::MENU_BUTTON_ACTIVE);
        } else if !disabled {
            class.push(' ');
            class.push_str(sc::MENU_BUTTON_HOVER);
        }
        if disabled {
            class.push(' ');
            class.push_str(sc::MENU_BUTTON_DISABLED);
        }
        if let Some(user) = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
        {
            if !user.trim().is_empty() {
                class.push(' ');
                class.push_str(user.trim());
            }
        }

        let label = if !text.is_empty() {
            text
        } else if let Some(derived) = self.extract_children_text(children, bindings) {
            derived
        } else {
            to.clone()
        };

        View::Button {
            disabled,
            label,
            content,
            onclick,
            style: Style::parse(&class).ok(),
            on_right_click: None,
        }
    }

}
