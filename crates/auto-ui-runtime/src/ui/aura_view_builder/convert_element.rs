//! PLAN-014 T-05：节点转换主分派责任（convert_element/convert_element_inner 巨表）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {
    /// PLAN-781 T-01：scroll-anchor 属性包装面（决策报告 §4）——薄入口，
    /// 实际分派在 [`Self::convert_element_inner`]。非空 key → 零布局
    /// `View::ScrollAnchorMark` 包装；空/类型错 = stderr 告警不包装。
    pub(super) fn convert_element(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let view = self.convert_element_inner(tag, props, events, children, bindings);
        self.maybe_wrap_scroll_anchor(view, props, bindings)
    }

    pub(super) fn convert_element_inner(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // PLAN-656 review F-1：别名拼写归一（scroll-pane→scroll 等）。
        let tag = Self::normalize_dispatch_tag(tag);
        // Plan 409 §10 续: hidden/md:hidden 元素按桌面语义不渲染(is_hidden:
        // 含 Hidden 且无 display 覆盖,见 Style::is_hidden)。
        if self.extract_style(props).map_or(false, |s| s.is_hidden()) {
            return View::Empty;
        }
        match tag {
            // Plan 476: slot outlet——按调用位填充渲染（父作用域求值），
            // 未命中渲染 fallback children。五大容器的 children 走拼接
            // 展开（expand_children_spliced），此臂覆盖其余位形。
            "slot" | "Slot" => self.render_slot_outlet(props, children, bindings),
            // Core layout widgets
            "col" | "column" => {
                let mut v = self.convert_column(props, children, bindings);
                self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                v
            }
            "row" => {
                let mut v = self.convert_row(props, children, bindings);
                self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                v
            }
            // Plan 463 T5: taskbar —— 桌面 shell 底栏（I4）；row 语义。
            // 镜像 tracked 层同名臂（文件 D-GAP 规则）。
            "taskbar" => self.convert_row(props, children, bindings),
            // PLAN-641 T-03：tabs 复合组件折叠——有状态 View::Tabs（此前落
            // unknown fallback → Column）。契约见 convert_tabs 文档。
            "tabs" | "Tabs" => self.convert_tabs(props, events, children, bindings),

            "grid" => self.convert_grid(props, children, bindings),
            // Plan 409 §10 续 3: HTML 语义/布局标签(scroll/aside/main/header...),
            // 之前落 fallback 丢 style(padding/flex/overflow),导致 sidebar 无 padding、
            // 无滚动条、Home 页溢出被裁。scroll → 可滚动 column;其余 → container。
            "scroll" | "scrollable" => self.convert_scroll(props, events, children, bindings),
            // PLAN-656 T-06: synthetic managed content（capability-test 专用）。
            "scroll_test_content" => self.managed_scroll_content_view(props, bindings),
            // Plan 482: nav 容器支持 search:true 集成搜索行，其余语义容器不变。
            "nav" => self.convert_nav_container(props, events, children, bindings),
            "aside" | "main" | "header" | "section" | "footer" | "article" => {
                self.convert_container(props, children, bindings)
            }
            // Plan 442 A4: svg 元素子树 → 序列化 SVG 文档经 View::Image 渲染
            // (此前落 unknown fallback → View::Empty)。与 tracked 层同名臂镜像。
            "svg" => self.convert_svg_image(props, children, bindings),
            // Plan 484: hover 命中区 —— 与 tracked 层同名臂镜像(文件 D-GAP 规则)。
            "mouse-area" => self.convert_mouse_area_untracked(props, events, children, bindings),
            // Plan 563: 状态驱动画布 —— 与 tracked 层同名臂镜像(D-GAP;
            // leaf 无 probing 面,共用 convert_canvas)。
            "canvas" => self.convert_canvas(props, events, bindings),
            // Plan 497: popover —— 与 tracked 层同名臂镜像(D-GAP 规则)。
            // tracked 兜底(taskbar 等无专门臂的容器)会整体 delegate 到本
            // untracked 实现,此前缺臂导致 popover 落 fallback 容器直通
            // children(锚/内容/overlay 语义全失——shell.at dock hover 预览
            // 实测暴露)。debug 上下文用空实例(open 属性驱动形态不依赖
            // 自管 slot_id;probe/id_map 仅调试注记)。
            "popover" => {
                let mut path = Vec::new();
                let mut id_map = crate::ui::debug_id_map::DebugIdMap::default();
                let mut probe = crate::ui::debug::BuildProbe::default();
                self.convert_popover(
                    props,
                    events,
                    children,
                    &mut path,
                    &mut id_map,
                    &mut probe,
                    bindings,
                )
            }
            // PLAN-717: mirror the tracked calendar arm above.
            "calendar" | "Calendar" => self.convert_calendar(props, bindings),
            // Plan 497: 每窗口真缩略 leaf（与 tracked 层同名臂镜像，D-GAP；
            // 字面形式与 render_support/schema.rs 三表同款 window_thumbnail）。
            "window_thumbnail" => self.convert_window_thumbnail(props, bindings),
            // PLAN-012 W3: 整桌面等比预览 leaf（tracked/untracked 双臂镜像，
            // window_thumbnail 同款 D-GAP 纪律；SD-02 DSL 合同面）。
            "workspace_preview" => self.convert_workspace_preview(props, bindings),
            // PLAN-760: 宿主侧远程帧消费 leaf（tracked/untracked 双臂镜像，
            // window_thumbnail 同款 D-GAP 纪律；url 即身份）。
            "remote-frame" => self.convert_remote_frame(props, bindings),

            // Command palette preview components are also web imports. Give
            // the gallery's static sample VM-native controls before placeholder
            // dispatch can consume the original tag.
            "command" => {
                let p = self.with_class_prop(
                    props,
                    bindings,
                    "w-full rounded-md border bg-popover text-popover-foreground shadow-sm",
                );
                self.convert_column(&p, children, bindings)
            }
            "command_list" | "command-list" => {
                let p = self.with_class_prop(props, bindings, "flex flex-col gap-1 p-1");
                self.convert_column(&p, children, bindings)
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
                group_children.extend(
                    children
                        .iter()
                        .map(|child| self.convert_node_with(child, bindings))
                        .filter(|view| !is_visually_empty(view)),
                );
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
            // The gallery's initial state contains results, so hide the empty
            // state until a VM search handler is active.
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

            // Static VM layout keeps every gallery slide visible in one row;
            // the Vue backend retains Embla's scrolling and gesture behavior.
            "carousel" => self.convert_column(props, children, bindings),
            "carousel_content" | "carousel-content" => {
                let p = self.with_class_prop(props, bindings, "w-full gap-2");
                self.convert_row(&p, children, bindings)
            }
            "carousel_item" | "carousel-item" => {
                let p = self.with_class_prop(
                    props,
                    bindings,
                    "w-24 shrink-0 rounded-md border bg-card p-2",
                );
                self.convert_container(&p, children, bindings)
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

            // PLAN-050 T7 (C5): use.web component 声明的图标组件（lucide 集，
            // `Folder { size: 14 }`）→ View::Image{lucide:kebab}，renderer 以
            // 既有 lucide glyph 直绘（svg 直绘 + currentColor tint）。此前落
            // unknown fallback → View::Empty（rail/设置面板图标全空）。未知
            // glyph 由 renderer 空占位兜底（与缺组件等价的降级）。
            // PLAN-053 P-053-6: registry 已注册同名适配器 widget 优先于图标
            // 回落（tracked 双胎同款守卫,详见 tracked 臂注释）。
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

            // Core element widgets
            "text" | "label" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span" | "a"
            | "link" | "small" | "strong" | "em" | "b" | "i" => {
                self.convert_text_element(tag, props, events, children, bindings)
            }
            "button" | "btn" | "native_button" => {
                self.convert_button(props, events, children, bindings)
            }
            // native_button：vue 侧显式原生逃生名（避开 button→shadcn
            // Button 映射，schema.rs 2514 注）；VM 侧与 button 同臂
            //（PLAN-051 T6：demo settings 弹层按钮两轨原生）。
            // PLAN-530 步骤7（W12）：toggle_group VM 映射——横排连体 button
            // 组（untracked 镜像臂,D-GAP 规则）。见 toggle_group_rewrite_children。
            "togglegroup" | "toggle-group" | "toggle_group" => {
                let rewritten = self.toggle_group_rewrite_children(props, children, bindings);
                let mut v = self.convert_row(props, &rewritten, bindings);
                self.set_layout_events(&mut v, events, bindings);
                v
            }
            // 组外裸 item → 单 button（组内由重写改道,不走此臂）。
            "togglegroup-item" | "toggle-group-item" | "toggle_group_item" => {
                self.convert_button(props, events, children, bindings)
            }
            // PLAN-530 步骤8（W13）：alert-dialog 全族 → 模态 Popover 原语
            // （untracked 镜像臂,D-GAP 规则）。见 convert_alert_dialog。
            // PLAN-533 T5: dialog 家族（可关闭模态）同走模态 Popover 臂。
            "dialog" | "Dialog" => {
                self.convert_alert_dialog(props, children, bindings, ModalDialogFamily::Dialog)
            }
            "dropdown-menu" | "dropdown_menu" | "dropdownmenu" | "DropdownMenu" => self
                .convert_alert_dialog(props, children, bindings, ModalDialogFamily::DropdownMenu),
            "alert-dialog" | "alert_dialog" | "alertdialog" => {
                self.convert_alert_dialog(props, children, bindings, ModalDialogFamily::Alert)
            }
            // PLAN-534: sheet/drawer → 贴边 Popover（untracked 镜像臂,
            // D-GAP 规则）。见 convert_side_panel。
            "sheet" | "Sheet" => {
                let placement = self.side_panel_placement(props, "side", bindings);
                self.convert_side_panel(props, children, bindings, placement, false)
            }
            "drawer" | "Drawer" => {
                let placement = self.side_panel_placement(props, "direction", bindings);
                self.convert_side_panel(props, children, bindings, placement, true)
            }
            // PLAN-534: hovercard → 非模态 Popover（untracked 镜像臂,
            // D-GAP 规则）。见 convert_hovercard。
            "hovercard" | "hover-card" | "hover_card" | "HoverCard" => {
                self.convert_hovercard(props, children, bindings)
            }
            // PLAN-695 T-11: menubar → 声明式组件族/actions 合成（untracked
            // 镜像臂,D-GAP 规则）——此前仅 tracked 表有此臂,动态 VM 解析
            // 路径（画廊 preview 实况预览）落 unknown 兜底整树 Empty,
            // baseline `vb0_not_in_vb1 menubar` 预存漂移由本臂消除。
            "menubar" => {
                if !children.is_empty() {
                    self.convert_menubar_component(props, children, bindings, None)
                } else {
                    self.convert_menubar(props, bindings, None)
                }
            }
            // PLAN-534: sheet/drawer/hover-card trigger/content 组外兜底
            // 透传（untracked 镜像,镜像 dialog 先例）。
            "sheet-trigger" | "sheet_trigger" | "sheettrigger" | "drawer-trigger"
            | "drawer_trigger" | "drawertrigger" | "hover-card-trigger" | "hover_card_trigger"
            | "hovercard-trigger" | "hovercardtrigger" | "sheet-content" | "sheet_content"
            | "sheetcontent" | "drawer-content" | "drawer_content" | "drawercontent"
            | "hover-card-content" | "hover_card_content" | "hovercard-content"
            | "hovercardcontent" => {
                let views: Vec<View<DynamicMessage>> = children
                    .iter()
                    .map(|c| self.convert_node_with(c, bindings))
                    .collect();
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
            // PLAN-534: sheet/drawer styled 子臂（untracked 镜像）。
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
                self.convert_column(&p, children, bindings)
            }
            "sheet-footer" | "sheet_footer" | "sheetfooter" | "drawer-footer" | "drawer_footer"
            | "drawerfooter" => {
                let p = self.with_class_prop(props, bindings, "flex justify-end gap-2");
                let mut v = self.convert_row(&p, children, bindings);
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
                let views: Vec<View<DynamicMessage>> = children
                    .iter()
                    .map(|c| self.convert_node_with(c, bindings))
                    .collect();
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
                self.convert_column(&p, children, bindings)
            }
            "alert-dialog-footer"
            | "alert_dialog_footer"
            | "alertdialog-footer"
            | "dialog-footer"
            | "dialog_footer"
            | "dialogfooter" => {
                let p = self.with_class_prop(props, bindings, "flex justify-end gap-2");
                let mut v = self.convert_row(&p, children, bindings);
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
            // Plan 482: nav 组件族 —— nav_contract 契约类转换（hover/active
            // 三态 + icon/desc/badge 槽 + to/onclick 双模式）。
            "nav-item" | "nav_item" => self.convert_nav_item(props, events, children, bindings),
            "nav-group" | "nav_group" => self.convert_nav_group(props, events, children, bindings),
            // Plan 561: sidebar_* VM 契约子集 —— tracked 站同名臂的镜像
            // （D-GAP 规则；契约类见 ui_gen::sidebar_contract）。
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
            // Plan 409 §10 续 8: badge = shadcn Badge(水平 inline + variant 配色)。
            "badge" => self.convert_badge(props, children, bindings),

            // Layout wrappers
            "center" => self.convert_center(props, children, bindings),

            // Input widgets
            "input" => self.convert_input(props, events, bindings),
            "textarea" => self.convert_textarea(props, events, bindings),
            // PLAN-661 T-03: slider（双表镜像纪律——表二臂）。
            "slider" => self.convert_slider(props, events, bindings),
            // Plan 446 批五 U4: select 控件 VM 端渲染。此前 view-builder 无
            // select 路由——快照结构在（源树回退）、渲染丢（os-config §P/U4
            // 被迫"select renders as free text"）。iced 侧 AbstractView::Select
            // 全支持（pick_list），本臂补齐产出端。
            "select" | "Select" => self.convert_select(props, events, children, bindings),
            // Plan 413: code editor widget (syntax highlighting, line
            // numbers, wrap, vi/undo, IME).
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
                    // 自本计划真消费（绑定求值传入 render，同 streamed 臂）；
                    // scroll_sync 真消费（同 streamed 臂）。
                    let _ = props.get("streaming");
                    let placeholder = self.autodown_placeholder(props, bindings);
                    let (scroll_sync, offset, on_scroll, details_onclick) =
                        self.autodown_scroll_binding(props, events, bindings, None);
                    // PLAN-045 T5：列宽状态 + 落定通道（oncolresize）。
                    let table_widths = self.autodown_table_widths(props, bindings);
                    let col_resize = self.autodown_on_col_resize_binding(events);
                    let mut doc = crate::ui::autodown_render::render_document_with(
                        &content,
                        is_final,
                        details_onclick.as_deref(),
                        placeholder,
                        table_widths.as_ref(),
                        col_resize.as_ref(),
                    );
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
            // PLAN-055 ④/T12: pre/code 进容器转换臂（tracked 臂镜像，D-GAP 规则）。
            "container" | "div" | "pre" | "code" => {
                // PLAN-055: 同 tracked 臂——html 去标签降级先于普通转换。
                if let Some(stripped) = self.div_html_stripped_text(props, bindings) {
                    let mut enriched: Vec<AuraNode> = children.to_vec();
                    enriched.push(AuraNode::Text(AuraTextContent::Literal(stripped)));
                    let mut v = self.convert_container(props, &enriched, bindings);
                    self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                    return v;
                }
                let mut v = self.convert_container(props, children, bindings);
                self.set_layout_events(&mut v, events, bindings); // Plan 490 G4
                v
            }

            // ImageSurface / Image / Icon.  ImageSurface keeps all dynamic
            // transform props and normalizes its five event hooks into the
            // same DynamicMessage shape consumed by the renderer.
            "imagesurface" | "image-surface" | "image_surface" | "ImageSurface" => {
                self.convert_image_surface(props, events, bindings)
            }
            // PLAN-617 T-19: `video` —— 受控媒体契约的 VM 侧节点（原生播放面）。
            // 与 tracked 臂同源；两条路都必须有臂，否则 untracked 语境会退回占位。
            "video" | "Video" => self.convert_video(props, events, bindings),
            "img" | "image" | "icon" => self.convert_image_or_icon(props, bindings),

            // Utility widgets
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

            // Child widget lookup or fallback
            _ => {
                // Plan 408: nav-link renders as a navigable button (like link).
                // nav-link is parsed as an Element (not Component), so handle it
                // here in the fallback before child-widget lookup. Extract the
                // `to`/`href` prop for navigation and `label`/`text` for display.
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
                // Plan 409 §10 续 9: category-section → 标题行 + Grid(对齐 vue
                // index.vue:45-51 的 色点+name+count 标题 + grid-cols)。之前
                // 直接堆 Column,component-card 单列排布,与 vue 网格不一致。
                if tag == "category-section" || tag == "category_section" {
                    let name = self.extract_string(props, "name").unwrap_or_default();
                    let count = self.extract_string(props, "count").unwrap_or_default();
                    let color = self.extract_string(props, "color").unwrap_or_default();
                    let dot_bg = match color.as_str() {
                        "blue" => "bg-blue-500",
                        "emerald" => "bg-emerald-500",
                        "amber" => "bg-amber-500",
                        "purple" => "bg-purple-500",
                        "rose" => "bg-rose-500",
                        // Plan 412 F7: Layout 分组 sky(#0ea5e9)
                        "sky" => "bg-sky-500",
                        _ => "bg-primary",
                    };
                    let dot = View::Container {
                        child: Box::new(View::Empty),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse(&format!("h-3 w-3 rounded-full {}", dot_bg)).ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    let name_view = View::Text {
                        content: name,
                        style: Style::parse("text-base font-semibold uppercase tracking-wider text-muted-foreground").ok(),
                        selectable: false,
                    };
                    let mut title_kids: Vec<View<DynamicMessage>> = vec![dot, name_view];
                    if !count.is_empty() {
                        title_kids.push(View::Text {
                            content: format!("({})", count),
                            style: Style::parse("text-xs text-muted-foreground").ok(),
                            selectable: false,
                        });
                    }
                    let title = View::Row {
                        children: title_kids,
                        spacing: 0,
                        padding: 0,
                        style: Style::parse("items-center gap-2 mb-4").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    // component-cards → Grid(cols=2, gap-3)
                    // Plan 409 §10 续 10: 设 CATEGORY_COLOR 供 component-card
                    // 继承颜色(save/restore 支持嵌套)。
                    let prev_color = CATEGORY_COLOR.with(|c| {
                        let prev = c.borrow().clone();
                        *c.borrow_mut() = color.clone();
                        prev
                    });
                    let cells: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    CATEGORY_COLOR.with(|c| *c.borrow_mut() = prev_color);
                    // Plan 409 §10 续 11: 响应式列数(对齐 vue sm/lg/xl 断点)。
                    // PLAN-814 R9-03：window_width 单源在 compiler theme
                    // （thread_local 纯函数）——直连，绕开 iced 门面。
                    let win_w = crate::ui::style::theme::window_width();
                    let cols = if win_w < 640.0 {
                        1
                    } else if win_w < 1024.0 {
                        2
                    } else if win_w < 1280.0 {
                        3
                    } else {
                        4
                    };
                    // Plan 409 §10 续 14: 等宽 grid(col[rows])。每 row(w-full) 含
                    // cols 个 cell;cell(component-card)自带 flex-1,在 row 里等分
                    // 宽度,实现上下左右对齐(不用 View::Grid,因其 cell Shrink 不齐)。
                    let grid = {
                        let mut rows_views: Vec<View<DynamicMessage>> = Vec::new();
                        let mut iter = cells.into_iter();
                        loop {
                            let chunk: Vec<View<DynamicMessage>> =
                                iter.by_ref().take(cols).collect();
                            if chunk.is_empty() {
                                break;
                            }
                            rows_views.push(View::Row {
                                children: chunk,
                                spacing: 12,
                                padding: 0,
                                style: Style::parse("w-full").ok(),
                                onclick: None,
                                on_right_click: None,
                            });
                        }
                        View::Column {
                            children: rows_views,
                            spacing: 12,
                            padding: 0,
                            style: Style::parse("w-full").ok(),
                            onclick: None,
                            on_right_click: None,
                        }
                    };
                    return View::Column {
                        children: vec![title, grid],
                        spacing: 16,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    };
                }
                // Plan 409 §10 组 E: preview-card / codeblock VM 识别。vue codegen
                // 对它们做特殊处理(generate_previewcard_html / generate_codeblock_html
                // 生成预览区 + code toggle + Auto/Vue tabs + copy);VM 无这些命令式
                // 能力,简化:preview-card → Column(递归 children,即真正的预览 UI),
                // codeblock → Text(代码内容;否则空 children 落到 Empty,安装命令整
                // 段消失)。
                if tag == "preview-card" || tag == "preview_card" || tag == "previewcard" {
                    // Plan 409 §10 续 19: 预览卡片(对齐 vue button.vue:126-180):
                    // 圆角 border 容器 + 居中预览区 + Code toggle(展开/收起) +
                    // Auto/Vue tab + 代码区。show/tab 来自 preview_states(局部 UI state)。
                    let id = self
                        .extract_string(props, "id")
                        .unwrap_or_else(|| "preview".to_string());
                    let ui = self
                        .preview_states
                        .and_then(|m| m.get(&id))
                        .copied()
                        .unwrap_or_default();
                    let child_views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    let inner: View<DynamicMessage> = if child_views.len() == 1 {
                        child_views.into_iter().next().unwrap()
                    } else {
                        View::Column {
                            children: child_views,
                            spacing: 8,
                            padding: 0,
                            style: None,
                            onclick: None,
                            on_right_click: None,
                        }
                    };
                    let preview_area = View::Container {
                        child: Box::new(inner),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: true,
                        center_y: true,
                        style: Style::parse("min-h-[100px] w-full p-4").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    // Plan 409 §10 续 21: 合并 Code title 行与 Auto/Vue tab 行为一行
                    // toolbar:[Auto][Vue] ... [copy][chevron](展开/收起)。代码区在
                    // 下方(展开时)。Plan 411: 对齐 vue 合并版 —— 右侧 copy icon
                    // (点击写入系统剪贴板,icon 换 check)+ chevron 图标;Code 文字不用。
                    // 当前 tab 的代码同时供 copy 参数与展示区使用。
                    let (auto_code, vue_code) = {
                        let mut vg = VueGenerator::new();
                        vg.gen_previewcard_code(props, children)
                    };
                    let code = if matches!(ui.tab, crate::ui::dynamic::PreviewTab::Vue) {
                        vue_code
                    } else {
                        auto_code
                    };
                    // Plan 411: active tab 下划线用 2px 主题色条实现 ——
                    // border-b-2(仅底边)解析器不支持,iced button 边框四边统一。
                    // 结构:定宽列(w-[72px]) + Button(w-full + text-center,px-4
                    // py-2 保证正常高度) + 2px 条(w-full)。Fill 只在 Fixed 宽度
                    // 父级内解析 —— 此前条放在 Shrink 环境里导致 tab 被撑宽/
                    // 左缘溢出;文字内边距写在 Text 上无效(py 丢失致高度塌陷)。
                    let mk_tab = |label: &str, tab_str: &str, active: bool| {
                        let btn = View::Button {
                            disabled: false,
                            label: label.to_string(),
                            content: None,
                            onclick: crate::ui::interpreter::DynamicMessage::Typed {
                                widget_name: self.widget_name.clone(),
                                event_name: "__preview_tab".to_string(),
                                args: vec![auto_val::Value::str(&id), auto_val::Value::str(tab_str)],
                            },
                            style: Style::parse(if active {
                                "w-full px-4 py-2 text-xs font-medium text-center bg-zinc-900 text-zinc-100"
                            } else {
                                "w-full px-4 py-2 text-xs text-center text-zinc-400"
                            }).ok(),
                            on_right_click: None,
                        };
                        let underline = View::Container {
                            child: Box::new(View::Empty),
                            padding: 0,
                            width: None,
                            height: None,
                            center_x: false,
                            center_y: false,
                            style: Style::parse(if active {
                                "h-[2px] w-full bg-primary"
                            } else {
                                "h-[2px] w-full"
                            })
                            .ok(),
                            onclick: None,
                            on_right_click: None,
                        };
                        View::Column {
                            children: vec![btn, underline],
                            spacing: 0,
                            padding: 0,
                            style: Style::parse("w-[72px]").ok(),
                            onclick: None,
                            on_right_click: None,
                        }
                    };
                    let tabs_inner = View::Row {
                        children: vec![
                            mk_tab(
                                "Auto",
                                "auto",
                                matches!(ui.tab, crate::ui::dynamic::PreviewTab::Auto),
                            ),
                            mk_tab(
                                "Vue",
                                "vue",
                                matches!(ui.tab, crate::ui::dynamic::PreviewTab::Vue),
                            ),
                        ],
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    };
                    let icon_btn_style = "px-2 py-1.5 text-xs text-muted-foreground bg-transparent";
                    let copy_btn = View::Button {
                        disabled: false,
                        label: format!(
                            "\u{EE01}{}\u{EE02}",
                            if ui.copied { "check" } else { "copy" }
                        ),
                        content: None,
                        onclick: crate::ui::interpreter::DynamicMessage::Typed {
                            widget_name: self.widget_name.clone(),
                            event_name: "__preview_copy".to_string(),
                            args: vec![auto_val::Value::str(&id), auto_val::Value::str(&code)],
                        },
                        style: Style::parse(icon_btn_style).ok(),
                        on_right_click: None,
                    };
                    let toggle = View::Button {
                        disabled: false,
                        label: format!(
                            "\u{EE01}{}\u{EE02}",
                            if ui.show {
                                "chevron-down"
                            } else {
                                "chevron-right"
                            }
                        ),
                        content: None,
                        onclick: crate::ui::interpreter::DynamicMessage::Typed {
                            widget_name: self.widget_name.clone(),
                            event_name: "__preview_toggle".to_string(),
                            args: vec![auto_val::Value::str(&id)],
                        },
                        style: Style::parse(icon_btn_style).ok(),
                        on_right_click: None,
                    };
                    let right_icons = View::Row {
                        children: vec![copy_btn, toggle],
                        spacing: 4,
                        padding: 0,
                        // Plan 411: tabs 贴 toolbar 左缘(vue 无容器内边距),右侧
                        // icon 组留 pr-2 呼吸位 —— 此前容器 px-2 让 Auto 左侧多出空白。
                        style: Style::parse("pr-2").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    let toolbar = View::Row {
                        children: vec![tabs_inner, right_icons],
                        spacing: 0,
                        padding: 0,
                        style: Style::parse(
                            "items-center justify-between border-t bg-zinc-800 w-full",
                        )
                        .ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    let mut col_kids: Vec<View<DynamicMessage>> = vec![preview_area, toolbar];
                    // 展开时:代码区(按 tab 选 auto/vue)。
                    if ui.show {
                        let code_text = View::Text {
                            content: code,
                            style: Style::parse("font-mono text-sm text-zinc-50").ok(),
                            selectable: false,
                        };
                        let code_area = View::Container {
                            child: Box::new(code_text),
                            padding: 0,
                            width: None,
                            height: None,
                            center_x: false,
                            center_y: false,
                            style: Style::parse("p-4 bg-zinc-950 border-t w-full").ok(),
                            onclick: None,
                            on_right_click: None,
                        };
                        col_kids.push(code_area);
                    }
                    return View::Container {
                        child: Box::new(View::Column {
                            children: col_kids,
                            spacing: 0,
                            padding: 0,
                            style: None,
                            onclick: None,
                            on_right_click: None,
                        }),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        // Plan 411: p-[1px] 模拟 CSS border-box 语义 —— iced 的
                        // border 画在边界内侧但不参与布局,子元素全宽叠在边框上:
                        // 自带背景的 toolbar/code 盖住边框画到边缘,无背景的预览区
                        // 露出边框,视觉上预览区比下方窄 1px。1px 内边距让子元素
                        // 整体收进边框,两侧对齐(vue 中 border 本就在 content box 外)。
                        style: Style::parse(
                            "rounded-lg border bg-zinc-900 w-full overflow-hidden p-[1px]",
                        )
                        .ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                if tag == "codeblock" || tag == "code_block" || tag == "code-block" {
                    // Plan 409 §10 续 16/18: 代码块(对齐 vue button.vue:110-123):
                    // 圆角 border 容器 + header(lang 标签) + 深色 monospace 代码。
                    let code = self
                        .extract_string(props, "code")
                        .or_else(|| self.extract_string(props, "text"))
                        .or_else(|| self.extract_children_text(children, bindings))
                        .unwrap_or_default();
                    let lang = self.extract_string(props, "lang").unwrap_or_default();
                    let lang_label = if lang.is_empty() {
                        "code".to_string()
                    } else {
                        lang.clone()
                    };
                    let header = View::Container {
                        child: Box::new(View::Text {
                            content: lang_label,
                            style: Style::parse("text-xs font-medium").ok(),
                            selectable: false,
                        }),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse("px-4 py-2 border-b bg-zinc-800 text-zinc-400").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    // Plan 442 A6: lang-<token> class carries the language to
                    // the renderer's syntect highlight path (read-only code
                    // highlighting, musk-038 T16 决策 (a)); lang-less blocks
                    // keep the hand-rolled tokenizer.
                    let lang_class = if !lang.is_empty() && !lang.contains(char::is_whitespace) {
                        format!(" lang-{lang}")
                    } else {
                        String::new()
                    };
                    let code_text = View::Text {
                        content: code,
                        style: Style::parse(&format!("font-mono text-sm text-zinc-50{lang_class}"))
                            .ok(),
                        selectable: false,
                    };
                    let code_area = View::Container {
                        child: Box::new(code_text),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse("p-4").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    return View::Container {
                        child: Box::new(View::Column {
                            children: vec![header, code_area],
                            spacing: 0,
                            padding: 0,
                            style: None,
                            onclick: None,
                            on_right_click: None,
                        }),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse("rounded-lg border bg-zinc-950 overflow-hidden w-full")
                            .ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                // Plan 409 §10 续 16: table → 表格(Column[Row[cells]]),对齐 vue。
                // table/thead/tbody → Column(tr 堆叠);tr → Row(th/td 横排);
                // th/td → Container(border + padding 形成网格) > Text。
                if tag == "table" {
                    let views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    return View::Column {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: Style::parse("border rounded w-full text-sm").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                if tag == "thead"
                    || tag == "tbody"
                    || tag == "tfoot"
                    || tag == "table-header"
                    || tag == "table-body"
                    || tag == "table-footer"
                {
                    let views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    return View::Column {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    };
                }
                if tag == "tr" || tag == "table-row" {
                    let views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    return View::Row {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    };
                }
                if tag == "th" || tag == "td" || tag == "table-head" || tag == "table-cell" {
                    let is_head = tag == "th" || tag == "table-head";
                    // Plan 446 批五 U5: 表头预设对齐 vue 轨（border + font-semibold，
                    // 无内置底色——此前 bg-muted 在 iced 暗色主题下呈"暗底白字"），
                    // 且 th/td 的 class/style prop 并入预设（后者优先，同
                    // convert_button 惯例）——此前用户样式被整体忽略。
                    let (cell_preset, text_style) = if is_head {
                        (
                            "px-4 py-2 border border-input flex-1 text-left font-semibold",
                            "text-foreground",
                        )
                    } else {
                        (
                            "px-4 py-2 border border-input flex-1",
                            "text-muted-foreground",
                        )
                    };
                    let user_cell = self
                        .extract_string_with(props, "class", bindings)
                        .or_else(|| self.extract_string_with(props, "style", bindings));
                    let cell_style = match user_cell.as_deref() {
                        Some(u) if !u.trim().is_empty() => {
                            format!("{} {}", cell_preset, u)
                        }
                        _ => cell_preset.to_string(),
                    };
                    // Plan 438 M2: shadcn kebab 族(table-head/table-cell)与 HTML
                    // 别名合一渲染——vue 侧两族映射同构,VM 侧此前只认 HTML 别名,
                    // kebab 源(gallery/025 形态)整表空白。非文本子节点(badge 等)
                    // 优先按子视图渲染,text prop 为纯文本单元格兜底(带 bindings——
                    // extract_string 的空 bindings 解不出 p.name/.nameH 等引用)。
                    let child_views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    let content = if child_views.len() == 1 {
                        child_views.into_iter().next().unwrap()
                    } else if child_views.len() > 1 {
                        View::Row {
                            children: child_views,
                            spacing: 0,
                            padding: 0,
                            style: None,
                            onclick: None,
                            on_right_click: None,
                        }
                    } else {
                        let text = self
                            .extract_children_text(children, bindings)
                            .or_else(|| self.extract_string_with(props, "text", bindings))
                            .unwrap_or_default();
                        View::Text {
                            content: text,
                            style: Style::parse(text_style).ok(),
                            selectable: false,
                        }
                    };
                    // 可点击列头(排序):带 onclick 的 th/table-head 渲染为
                    // Button(View::Text 无 onclick 字段,与上方 text→Button 同款)。
                    if is_head {
                        if let Some(event) = aura_events_get_base(events, "onclick") {
                            if let View::Text { content, .. } = &content {
                                return View::Button {
                                    disabled: false,
                                    label: content.clone(),
                                    onclick: self.event_to_message_with(event, bindings),
                                    style: Style::parse(&cell_style).ok(),
                                    on_right_click: None,
                                    content: None,
                                };
                            }
                        }
                    }
                    return View::Container {
                        child: Box::new(content),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse(&cell_style).ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                // Plan 450 / 019 批次三: AutoDown 面板词汇 → iced 降级渲染。
                // registry 登记见 widget/registry.rs register_document_panel_widgets;
                // 这里分解为既有 View 变体(Plan 319 单臂规则 —— 不新增 View 变体,
                // renderer 零改动)。tracked 链路经 convert_element_tracked_ctx 的
                // 委托到达这里(与 codeblock/table 同款:纯展示组件,无 probe 需求)。
                // tag 集与 registry 别名对齐:heading/quote/callout/details/
                // math_block/query_block/embed_block(codeblock/table/list/h1..h6/
                // separator 已有各自臂)。
                if tag == "heading" {
                    let level = self.extract_u16(props, "level").unwrap_or(1).clamp(1, 6);
                    // 与上方 h1..h6 默认样式同源(text-primary + 页边距,plan 409 §8)。
                    let style = match level {
                        1 => "text-4xl font-bold tracking-tight text-primary mb-4",
                        2 => "text-3xl font-bold tracking-tight text-primary mt-8 mb-4",
                        3 => "text-xl font-semibold text-primary mb-3",
                        4 => "text-lg font-semibold mb-2",
                        5 => "text-base font-semibold mb-1",
                        _ => "text-sm font-semibold mb-1",
                    };
                    let content = self
                        .extract_children_text(children, bindings)
                        .or_else(|| self.extract_string(props, "text"))
                        .unwrap_or_default();
                    return View::Text {
                        content,
                        style: Style::parse(style).ok(),
                        selectable: false,
                    };
                }
                if tag == "quote" || tag == "blockquote" {
                    let views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    let inner = if views.is_empty() {
                        View::Text {
                            content: String::new(),
                            style: None,
                            selectable: false,
                        }
                    } else if views.len() == 1 {
                        views.into_iter().next().unwrap()
                    } else {
                        View::Column {
                            children: views,
                            spacing: 4,
                            padding: 0,
                            style: None,
                            onclick: None,
                            on_right_click: None,
                        }
                    };
                    return View::Container {
                        child: Box::new(inner),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse("border-l-4 pl-4 py-2 w-full text-muted-foreground")
                            .ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                if tag == "callout" {
                    let kind = self.extract_string(props, "kind").unwrap_or_default();
                    let title = self.extract_string(props, "title").unwrap_or_default();
                    let (tint_cls, title_style) = match kind.as_str() {
                        "tip" | "success" => (
                            "border-emerald-500/40 bg-emerald-500/10",
                            "text-sm font-medium text-emerald-400",
                        ),
                        "warning" | "warn" => (
                            "border-amber-500/40 bg-amber-500/10",
                            "text-sm font-medium text-amber-400",
                        ),
                        "danger" | "error" | "caution" => (
                            "border-red-500/40 bg-red-500/10",
                            "text-sm font-medium text-red-400",
                        ),
                        "note" | "info" => (
                            "border-blue-500/40 bg-blue-500/10",
                            "text-sm font-medium text-blue-400",
                        ),
                        _ => (
                            "border-zinc-500/40 bg-zinc-500/10",
                            "text-sm font-medium text-zinc-400",
                        ),
                    };
                    let mut kids: Vec<View<DynamicMessage>> = Vec::new();
                    if !title.is_empty() {
                        kids.push(View::Text {
                            content: title,
                            style: Style::parse(title_style).ok(),
                            selectable: false,
                        });
                    }
                    kids.extend(
                        children
                            .iter()
                            .map(|n| self.convert_node_with(n, bindings))
                            .filter(|v| !matches!(v, View::Empty)),
                    );
                    return View::Container {
                        child: Box::new(View::Column {
                            children: kids,
                            spacing: 8,
                            padding: 0,
                            style: None,
                            onclick: None,
                            on_right_click: None,
                        }),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse(&format!("rounded-lg border {tint_cls} p-4 w-full"))
                            .ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                if tag == "details" {
                    // Details → Accordion 单项(对齐表"可对齐 Accordion 族"裁定)。
                    // VM 降级无 toggle 回写,on_toggle 留 None → 静态展开式折叠头。
                    use crate::ui::view::AccordionItem;
                    let summary = self
                        .extract_string(props, "summary")
                        .unwrap_or_else(|| "Details".to_string());
                    let views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    return View::Accordion {
                        items: vec![AccordionItem {
                            title: summary,
                            icon: None,
                            children: views,
                            expanded: true,
                        }],
                        allow_multiple: false,
                        on_toggle: None,
                        style: None,
                    };
                }
                // 以下三面板是"注册位"面板(消费方注册渲染器,plan 017 待澄清 #2):
                // iced 侧无专用渲染器,降级为可见的源码/引用文本,避免内容静默丢弃。
                if tag == "math_block" || tag == "mathblock" || tag == "math-block" {
                    let source = self
                        .extract_string(props, "source")
                        .or_else(|| self.extract_children_text(children, bindings))
                        .unwrap_or_default();
                    return View::Container {
                        child: Box::new(View::Text {
                            content: source,
                            style: Style::parse("font-mono text-sm").ok(),
                            selectable: false,
                        }),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse("rounded-lg border bg-card p-4 w-full").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                if tag == "query_block" || tag == "queryblock" || tag == "query-block" {
                    let query = self
                        .extract_string(props, "query")
                        .or_else(|| self.extract_children_text(children, bindings))
                        .unwrap_or_default();
                    return View::Container {
                        child: Box::new(View::Text {
                            content: query,
                            style: Style::parse("font-mono text-xs text-muted-foreground").ok(),
                            selectable: false,
                        }),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse("rounded-lg border p-3 w-full").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                if tag == "embed_block" || tag == "embedblock" || tag == "embed-block" {
                    let target = self.extract_string(props, "target").unwrap_or_default();
                    return View::Container {
                        child: Box::new(View::Text {
                            content: format!("↪ {target}"),
                            style: Style::parse("text-sm text-muted-foreground").ok(),
                            selectable: false,
                        }),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse("rounded-lg border bg-muted p-3 w-full").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
                // Plan 410: component-card → navigable link button (to + name + desc).
                // Plan 409 §10 续 15: 完整卡片样式(对齐 vue index.vue:52-60):
                // icon box(h-10 w-10 rounded-lg border bg-{color}-500/10) + 文字列
                // (主标题 font-medium text-sm + 副标题 text-xs muted,换行展示)。
                // 之前 render_link_button_with_icon 把 name+desc 合并成单行 label,
                // 没体现卡片结构。颜色继承父 category-section 的 color。
                if tag == "component-card" || tag == "component_card" || tag == "componentcard" {
                    let to = self.extract_string(props, "to").unwrap_or_default();
                    let name = self.extract_string(props, "name").unwrap_or_default();
                    let desc = self.extract_string(props, "desc").unwrap_or_default();
                    let icon = self.extract_string(props, "icon").unwrap_or_default();
                    let color = CATEGORY_COLOR.with(|c| c.borrow().clone());
                    let (border_cls, box_bg, icon_cls) = match color.as_str() {
                        "blue" => ("border-blue-500/40", "bg-blue-500/10", "text-blue-400"),
                        "emerald" => (
                            "border-emerald-500/40",
                            "bg-emerald-500/10",
                            "text-emerald-400",
                        ),
                        "amber" => ("border-amber-500/40", "bg-amber-500/10", "text-amber-400"),
                        "purple" => (
                            "border-purple-500/40",
                            "bg-purple-500/10",
                            "text-purple-400",
                        ),
                        "rose" => ("border-rose-500/40", "bg-rose-500/10", "text-rose-400"),
                        // Plan 412 F7: Layout 分组 sky(#0ea5e9)
                        "sky" => ("border-sky-500/40", "bg-sky-500/10", "text-sky-400"),
                        _ => ("border-border", "bg-primary/10", "text-primary"),
                    };
                    // icon box:圆角方块 + bg-{color}-500/10 + 居中的彩色图标。
                    let icon_box = View::Container {
                        child: Box::new(View::Image {
                            src: format!("lucide:{}", icon),
                            style: Style::parse(&format!("h-5 w-5 {}", icon_cls)).ok(),
                        }),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: true,
                        center_y: true,
                        style: Style::parse(&format!(
                            "h-10 w-10 shrink-0 rounded-lg border {} {}",
                            border_cls, box_bg
                        ))
                        .ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    // 文字列:主标题(font-medium text-sm) + 副标题(text-xs muted),换行。
                    let mut text_kids: Vec<View<DynamicMessage>> = vec![View::Text {
                        content: name.clone(),
                        style: Style::parse("font-medium text-sm").ok(),
                        selectable: false,
                    }];
                    if !desc.is_empty() {
                        text_kids.push(View::Text {
                            content: desc,
                            style: Style::parse("text-xs text-muted-foreground").ok(),
                            selectable: false,
                        });
                    }
                    let text_col = View::Column {
                        children: text_kids,
                        spacing: 2,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    };
                    // content: icon_box + 文字列(顶部对齐)。
                    let content = View::Row {
                        children: vec![icon_box, text_col],
                        spacing: 12,
                        padding: 0,
                        style: Style::parse("items-start gap-3 w-full").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                    return View::Button {
                        disabled: false,
                        label: name,
                        content: Some(Box::new(content)),
                        onclick: crate::ui::interpreter::DynamicMessage::Typed {
                            widget_name: self.widget_name.clone(),
                            event_name: "__navigate".to_string(),
                            args: vec![auto_val::Value::str(to)],
                        },
                        style: Style::parse(&format!(
                            "rounded-xl border {} bg-card p-4 text-left flex-1",
                            border_cls
                        ))
                        .ok(),
                        on_right_click: None,
                    };
                }
                // PLAN-066: 原生外部组件注册表——内置臂穷尽后、.at AuraWidget
                // 前查（原生优先语义保持，提案 066 §3.3）。命中即接管。
                if let Some(nv) = self.native_widget_view(tag, props, events, children, bindings) {
                    return nv;
                }
                // Check if this tag matches a registered child widget
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(tag) {
                        // Plan 476: 调用位 children 里的 slot 填充传给子构建器。
                        let fills = self.slot_fills_for(children, bindings);
                        return self.render_child_widget(
                            child_widget,
                            props,
                            events,
                            bindings,
                            fills.as_ref(),
                        );
                    }
                }

                // Fallback: wrap children in a column, filtering out Empty views
                let views: Vec<View<DynamicMessage>> = children
                    .iter()
                    .map(|n| self.convert_node_with(n, bindings))
                    .filter(|v| !matches!(v, View::Empty))
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
    }

    /// Plan 408: Map common lucide icon names to emoji for vm/iced rendering.
    /// iced has no icon font; this gives nav-link items visual distinction.
    pub(super) fn icon_to_emoji(icon: &str) -> Option<&'static str> {
        match icon {
            "bell" => Some("\u{1F514}"),                // 🔔
            "command" => Some("\u{2318}"),              // ⌘
            "image" => Some("\u{1F5BC}"),               // 🖼️
            "layout-grid" => Some("\u{25A6}"),          // ▦
            "menu" => Some("\u{2630}"),                 // ☰
            "mouse-pointer-click" => Some("\u{1F5B1}"), // 🖱️
            "navigation" => Some("\u{1F9ED}"),          // 🧭
            "search" => Some("\u{1F50D}"),              // 🔍
            "square-stack" => Some("\u{1F4E6}"),        // 📦
            "type" => Some("\u{1F4DD}"),                // 📝
            "home" => Some("\u{1F3E0}"),                // 🏠
            "settings" => Some("\u{2699}"),             // ⚙️
            "layers" => Some("\u{1F5C2}"),              // 🗂️
            "arrow-right" => Some("\u{27A1}"),          // ➡️
            "folder" => Some("\u{1F4C1}"),              // 📁
            "mail" => Some("\u{1F4E7}"),                // 📧
            "book" => Some("\u{1F4D6}"),                // 📖
            "github" => Some("\u{1F47D}"),              // 👽 (no github emoji)
            "check-square" => Some("\u{2705}"),         // ✅
            _ => None,
        }
    }

    /// Plan 401/VM-routing: render a `link (to: "/path")` as a clickable
    /// button. The onclick carries a `__navigate` DynamicMessage with the target
    /// path as its sole arg; the update loop intercepts `__navigate` and sets
    /// `__current_route`, causing `outlet` to re-render the new page. The label
    /// comes from the link's text or its first text child.
    /// Plan 408: optional icon name is mapped to emoji and prepended to label.
    pub(super) fn render_link_button(
        &self,
        text: &str,
        children: &[auto_ui_compiler::aura::AuraNode],
        to: &str,
        _bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // Plan 409 §10 组 C: header link（AuraNode::Link，如 Docs/Widgets）走这里，
        // 用主题色 text-primary。nav-link/component-card 不走这里（它们直接调
        // render_link_button_with_icon 并传 themed=false）。
        self.render_link_button_with_icon(text, children, to, "", _bindings, true)
    }

    /// Plan 408: render_link_button with an optional icon name.
    ///
    /// Plan 409 §6: when the link has child nodes, convert them into a content
    /// container rendered *inside* the button — vue parity for
    /// `link (to:) { text / row / icon ... }`. The `label` is still derived
    /// from the children (for the snapshot builder / accessibility), but the
    /// rendered content is the container, not a flattened `to` string.
    pub(super) fn render_link_button_with_icon(
        &self,
        text: &str,
        children: &[auto_ui_compiler::aura::AuraNode],
        to: &str,
        icon: &str,
        bindings: &Bindings,
        themed: bool,
    ) -> View<DynamicMessage> {
        // Plan 409 §6: convert child nodes into a content subtree. When
        // non-empty, the button renders this container instead of the label
        // string (link is no longer a leaf).
        let content: Option<Box<View<DynamicMessage>>> = if children.is_empty() {
            None
        } else {
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
        };

        // Resolve a display label: prefer explicit text, else the children's
        // text (recursively — so a `text (text: "Docs")` child yields "Docs",
        // not the raw path), else fall back to the path itself.
        let mut label = if !text.is_empty() {
            text.to_string()
        } else if let Some(derived) = self.extract_children_text(children, bindings) {
            derived
        } else {
            to.to_string()
        };
        // Plan 408: embed icon name in label using PUA markers. Only applies to
        // the label fallback (nav-link passes empty children → content is
        // None); when a content container is present the icon belongs to the
        // children subtree instead.
        if !icon.is_empty() && content.is_none() {
            label = format!("\u{EE01}{}\u{EE02}{}", icon, label);
        }
        View::Button {
            disabled: false,
            label,
            content,
            onclick: crate::ui::interpreter::DynamicMessage::Typed {
                widget_name: self.widget_name.clone(),
                event_name: "__navigate".to_string(),
                args: vec![auto_val::Value::str(to)],
            },
            // Plan 409 §10 组 C: themed=true → text-primary（header Docs/Widgets
            // link，走 AuraNode::Link，与 vue router-link 默认 text-primary 一致）；
            // themed=false → None（nav-link/component-card 普通色，§8.3 与 sidebar
            // 一致）。Button 无显式 text_color 时 renderer 用 OnBackground 默认色。
            // Plan 411 P1-A: `to` 命中当前路由时加 active 高亮（对齐 vue
            // router-link-active 的 bg-accent 块），header link 不参与。
            style: if themed {
                Style::parse("text-primary").ok()
            } else {
                let active = !to.is_empty()
                    && match self.read_state("__current_route") {
                        Ok(auto_val::Value::Str(s)) => s.as_str() == to,
                        _ => false,
                    };
                if active {
                    Style::parse("bg-accent text-accent-foreground font-medium rounded-md").ok()
                } else {
                    None
                }
            },
            on_right_click: None,
        }
    }

}
