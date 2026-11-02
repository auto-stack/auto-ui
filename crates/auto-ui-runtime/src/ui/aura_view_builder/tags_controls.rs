//! PLAN-014 T-05：表单控件责任（button/input/slider/select/terminal/code_editor/checkbox/text_element/toggle_group）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {
    /// Plan 446 批五 U7(链4): 文本类子元素的文本内容提取（绑定感知，
    /// 走 props 直取链 extract_string_with——与宿主元素同款求值）。
    pub(super) fn child_element_text(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<String> {
        self.extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_string_with(props, "content", bindings))
            .or_else(|| self.extract_string_with(props, "label", bindings))
    }

    pub(super) fn convert_text_element(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let content = self
            .extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_string_with(props, "content", bindings))
            .or_else(|| self.extract_string_with(props, "label", bindings))
            .unwrap_or_else(|| {
                // Try to get content from child text nodes
                children
                    .iter()
                    .filter_map(|c| match c {
                        AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                        AuraNode::Text(AuraTextContent::Interpolated {
                            template,
                            bindings: tpl_bindings,
                        }) => {
                            Some(self.resolve_interpolation_with(template, tpl_bindings, bindings))
                        }
                        // Plan 446 批五 U7(链4): 同 tracked 变体——文本类子元素
                        // 的内容折叠(绑定感知)。
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

        // Map heading tags to styled text
        let styled_content = match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => content,
            _ => content,
        };

        // If this text element has an onclick/click event, render it as a
        // Button so the click handler fires (View::Text has no onclick field).
        // The Button renderer applies chromeless styling when no bg/border is
        // present, so the visual appearance matches plain text.
        if let Some(event) = aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "click"))
        {
            let onclick = self.event_to_message_with(event, bindings);
            return View::Button {
                disabled: false,
                label: styled_content,
                onclick,
                style,
                on_right_click: None,
                content: None,
            };
        }

        // Plan 481:opt-in selectable(v1 默认 false)。与 tracked 变体同型。
        let selectable = self.extract_bool(props, "selectable").unwrap_or(false);

        View::Text {
            content: styled_content,
            style,
            selectable,
        }
    }

    /// Convert a button element.
    // PLAN-530 步骤7（源 PLAN-528 W12）：toggle_group VM 映射——横排连体
    // button 组。此前 schema（aura 三表）/vue 轨（W7 registry）都在,VM 无
    // 消费臂 → 页面 togglegroup 塌缩 unknown fallback,B/I/U 纵向裸排。
    // 实现：组 → row；item 子节点重写为 "button"（既有 button 臂承接
    // label/content/onclick）并按位置注入连体类（首/尾/中圆角 + 相邻
    // -ml-px 边框叠压）;variant=outline → border 透明底;size sm/lg →
    // padding 档。页面书写的 [&>*+*] CSS 选择器被解析器自然跳过（unmapped
    // 报告通道,不静默丢语义）。
    pub(super) fn toggle_group_rewrite_children(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> Vec<AuraNode> {
        let variant = self
            .extract_string_with(props, "variant", bindings)
            .unwrap_or_default();
        let size = self
            .extract_string_with(props, "size", bindings)
            .unwrap_or_default();
        let outline = variant == "outline";
        let size_class = match size.as_str() {
            "sm" => "px-2.5 text-sm",
            "lg" => "px-4 text-base",
            _ => "px-3",
        };
        let n = children.len();
        let mut out = Vec::with_capacity(n);
        for (i, child) in children.iter().enumerate() {
            let AuraNode::Element {
                tag,
                props: item_props,
                events,
                children: item_children,
                span,
                debug_id,
            } = child
            else {
                out.push(child.clone());
                continue;
            };
            let is_item = matches!(
                tag.as_str(),
                "togglegroup-item" | "toggle-group-item" | "toggle_group_item"
            );
            if !is_item {
                out.push(child.clone());
                continue;
            }
            let mut new_variant: Option<&str> = None;
            let mut classes = String::from(size_class);
            if outline {
                // variant 传导:item button 用 outline 预设(border+bg-background
                // +text-foreground),叠边/圆角由位置类承担。
                new_variant = Some("outline");
                classes.push_str(" -ml-px");
            } else {
                // ghost 档:透明底 + 前景文字（primary 默认预设的填充底色在
                // 连体组里语义错误）。
                classes.push_str("bg-transparent text-foreground hover:bg-secondary");
            }
            classes.push(' ');
            if n > 1 {
                if i == 0 {
                    classes.push_str("rounded-l-md rounded-r-none");
                } else if i == n - 1 {
                    classes.push_str("rounded-r-md rounded-l-none");
                } else {
                    classes.push_str("rounded-none");
                }
            } else {
                classes.push_str("rounded-md");
            }
            let mut merged = classes;
            if let Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))) =
                item_props.get("class")
            {
                merged.push_str(s.as_str());
            }
            let mut new_props = item_props.clone();
            if let Some(v) = new_variant {
                new_props.insert(
                    "variant".to_string(),
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(v.into())),
                );
            }
            new_props.insert(
                "class".to_string(),
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(merged.into())),
            );
            out.push(AuraNode::Element {
                tag: "button".to_string(),
                props: new_props,
                events: events.clone(),
                children: item_children.clone(),
                span: *span,
                debug_id: *debug_id,
            });
        }
        out
    }

    pub(super) fn convert_button(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // Plan 409 §10 组 A: icon-only buttons (header hamburger / search /
        // theme) previously fell back to the default "Button" text because the
        // `icon` prop was ignored. When an `icon` is present, embed it in the
        // label via PUA markers (same scheme nav-link uses, §2.3) and only use
        // a real text/label/child text if provided — so `button (icon:"menu")`
        // renders just the icon, not "Button".
        let icon = self
            .extract_string_with(props, "icon", bindings)
            .unwrap_or_default();
        let label = if !icon.is_empty() {
            let text_part = self
                .extract_string_with(props, "text", bindings)
                .or_else(|| self.extract_string_with(props, "label", bindings))
                .or_else(|| self.extract_children_text(children, bindings))
                .unwrap_or_default();
            format!("\u{EE01}{}\u{EE02}{}", icon, text_part)
        } else {
            // Plan 409 §10 续: 无 text/label/children 的 button(如主题色板的纯
            // 色块)用空 label,而非占位 "Button"(对齐 vue 空 slot 行为)。
            self.extract_string_with(props, "text", bindings)
                .or_else(|| self.extract_string_with(props, "label", bindings))
                .or_else(|| self.extract_children_text(children, bindings))
                .unwrap_or_default()
        };

        // PLAN-053 批4: `title` prop → EE03 PUA tooltip marker。vue 轨把 title
        // 映射为原生属性(浏览器悬停提示);VM 轨此前只有 toolbar 合成按钮埋
        // EE03,普通 button 的 title 被静默丢弃。统一接线后 renderer Button 臂
        // 剥离 EE03 尾段并把按钮包进 iced tooltip(300ms 延迟防误触)。
        // binding-aware:musk 会话列表 `title: .s.id` 按循环变量逐项求值。
        // PLAN-054 T1: EE03 后缀推迟到构造 View::Button 时才拼入 label——
        // 内容子树的 leading Text 必须用干净 label,否则 EE03 PUA 字形落
        // 可见文本流(A1 卡片 "Y<id>" / A2 "Y新建会话" 常显的根因)。
        let pua_title = self
            .extract_string_with(props, "title", bindings)
            .unwrap_or_default();

        // `variant` selects a base style preset (Tailwind classes); the user's
        // class/style augments it. "text"/absent = chromeless (renders as text
        // via the renderer's class-driven style); "primary" = theme-colored
        // filled button (Plan 409 §8: theme-aware instead of hardcoded blue).
        let variant = self
            .extract_string_with(props, "variant", bindings)
            .unwrap_or_default();
        // Plan 409 §10 续 7:variant → shadcn-vue 对齐的 Tailwind class preset
        // (由 renderer 的 class 驱动样式)。default/缺省 = primary(主题色填充,与
        // shadcn-vue Button 一致);secondary/destructive = 对应填充色;outline =
        // 边框;ghost/link = 透明。user class 追加在后,可覆盖 preset(如主题色板
        // 的 bg-xxx-500 覆盖 bg-primary,因 IcedStyle 后解析的 BackgroundColor 覆盖前者)。
        // PLAN-571: preset 表收敛到 ui::style::variants 单一事实源——default
        // 一等化（UA 预填等价基线：muted 填充+发丝描边，任意表面可辨），
        // primary|submit 显式醒目档，secondary 深一档纯填充。本臂与 rust
        // codegen 臂共用同表；Vue 臂 cva 由 ui_gen 的互锁测试锚定。
        let preset: &str = crate::ui::style::variants::button_variant_preset(variant.as_str());
        // Plan 409 §10 续 17: size → shadcn 对齐的尺寸 preset(h/px)。button 默认
        // Shrink,需显式 height 才能区分 sm/default/lg(renderer button 分支读 height)。
        let size = self
            .extract_string_with(props, "size", bindings)
            .unwrap_or_default();
        let size_preset: &str = crate::ui::style::variants::button_size_preset(size.as_str());
        // Plan 414 R13 fix: variant=icon carries its own square sizing in the
        // variant preset (h-7 w-7); the default "h-10 px-4" size preset would
        // override it (later classes win) AND its px-4 starves the 14px svg
        // (28px box - 32px horizontal padding = negative content width,
        // icon invisible). Empty the size preset unless an explicit size prop
        // is given.
        let size_preset: &str = if variant == "icon" && size.is_empty() {
            ""
        } else {
            size_preset
        };
        let style = {
            // Binding-aware so a class can come from the loop variable, e.g.
            // `class: cell.style` where each cell carries its own Tailwind class.
            let user = self
                .extract_string_with(props, "class", bindings)
                .or_else(|| self.extract_string_with(props, "style", bindings));
            // Plan 446 批五 U7(链3): 求值成功但样式类全被丢弃(自定义类名不在
            // Tailwind 词汇表,如 os-config 的 "e-row active")时显式告警——
            // 此前静默回退 preset 外观(紫块/白字/h-10),下游无从定位
            // (§P 处方:"求值失败→静默兜底"改显式告警)。
            if let Some(u) = user.as_deref() {
                if !u.trim().is_empty()
                    && Style::parse(u)
                        .map(|s| s.classes.is_empty())
                        .unwrap_or(true)
                {
                    log::warn!(
                        "[446-U7] button class/style resolved to '{}' but no style classes \
                         survived parsing (custom class names outside the Tailwind vocabulary) \
                         — button falls back to its preset look",
                        u
                    );
                }
            }
            let base = if preset.is_empty() {
                size_preset.to_string()
            } else {
                format!("{} {}", preset, size_preset)
            };
            let merged = match user.as_deref() {
                None => base,
                Some(c) => format!("{} {}", base, c),
            };
            if merged.is_empty() {
                None
            } else {
                Style::parse(&merged).ok()
            }
        };

        // Resolve the onclick event handler to a DynamicMessage.
        // Base-aware lookup: event keys may carry modifiers (onclick.self).
        let onclick = aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "click"))
            .map(|event| self.event_to_message_with(event, bindings))
            .unwrap_or_else(|| DynamicMessage::String("click".to_string()));

        // Plan 402: resolve oncontextmenu (right-click) handler for flagging
        let on_right_click = aura_events_get_base(events, "oncontextmenu")
            .or_else(|| aura_events_get_base(events, "contextmenu"))
            .map(|event| self.event_to_message_with(event, bindings));

        // Plan 057 (ash-gui): buttons with STRUCTURAL children (conditional
        // text, nested elements — e.g. ToolSidebar's `button .c.name { if …
        // { text .c.description … } }`) used to lose every child except the
        // label: extract_children_text skips Conditional nodes and View::Button
        // carried no subtree, so vue-style two-line buttons rendered name-only.
        // When any child is more than a plain text node, convert the children
        // into the Plan 409 §6 `content` subtree instead — the renderer draws
        // it as the button body (inheriting the button's text color); the
        // label stays for snapshot/inspect tooling.
        //
        // The inline primary prop (`button .c.name` parses as a text PROP,
        // not a child node — see parser's primary-prop shorthand) is not part
        // of `children`, so a prop-sourced label must be prepended as the
        // leading Text view or the name vanishes once the renderer switches
        // to the content subtree. Its style keeps only the button's
        // typography classes (size/weight/family/text color) so the name
        // renders like the old label without picking up bg/padding classes.
        let content = if children.iter().any(|c| !matches!(c, AuraNode::Text(_))) {
            let mut views: Vec<View<DynamicMessage>> = Vec::new();
            let label_from_prop = self.extract_children_text(children, bindings).is_none();
            if label_from_prop && !label.is_empty() {
                let text_style = style.as_ref().map(|s| Style {
                    classes: s
                        .classes
                        .iter()
                        .filter(|c| {
                            matches!(
                                c,
                                StyleClass::TextXs
                                    | StyleClass::TextSm
                                    | StyleClass::TextBase
                                    | StyleClass::TextLg
                                    | StyleClass::TextXl
                                    | StyleClass::Text2Xl
                                    | StyleClass::Text3Xl
                                    | StyleClass::Text4Xl
                                    | StyleClass::Text5Xl
                                    | StyleClass::Text6Xl
                                    | StyleClass::Text7Xl
                                    | StyleClass::Text8Xl
                                    | StyleClass::Text9Xl
                                    | StyleClass::FontBold
                                    | StyleClass::FontMedium
                                    | StyleClass::FontNormal
                                    | StyleClass::FontSerif
                                    | StyleClass::FontSans
                                    | StyleClass::FontMono
                                    | StyleClass::TextColor(_)
                            )
                        })
                        .cloned()
                        .collect(),
                    // hover: utilities only feed button hover styling; a text
                    // child view never consumes them.
                    hover_classes: Vec::new(),
                    variant_classes: Vec::new(),
                });
                views.push(View::Text {
                    content: label.clone(),
                    style: text_style,
                    selectable: false,
                });
            }
            views.extend(
                children
                    .iter()
                    .map(|c| self.convert_node_with(c, bindings))
                    .filter(|v| !is_visually_empty(v)),
            );
            match views.len() {
                0 => None,
                1 => Some(Box::new(views.into_iter().next().unwrap())),
                // PLAN-050 C1 残余收尾: 布局方向按按钮自身 flex 类——`flex`（行,
                // Tailwind 默认）包 Row,`flex-col` 才 Column;gap→spacing,
                // items-*/justify-* 随 style 下传（renderer row/column 臂消费
                // 对齐）。此前恒 Column → 图标叠文字上方且整体居中（vue 轨
                // flex 行形态）。
                _ => {
                    // PLAN-536 重测回归撤销(2026-09-04):button 结构子件内的
                    // absolute hoist 臂使 Button{content:Overlay} 在画布上
                    // 渲染为空壳(musk 会话列表卡片实测:名称/× 全失,仅剩
                    // 圆角描边;截图 p536_retest_rail)。悬浮消费方改走兄弟
                    // 结构(musk chats_view:× 移出卡片 button,由外层 col
                    // 的 col-arm hoist 承接)。本臂待 renderer 侧
                    // Button{content:Overlay} 渲染根修后再启。
                    let is_col = style.as_ref().map_or(false, |s| {
                        s.classes.iter().any(|c| matches!(c, StyleClass::FlexCol))
                    });
                    let spacing = style
                        .as_ref()
                        .and_then(|s| {
                            s.classes.iter().find_map(|c| match c {
                                StyleClass::Gap(g) => Some(g.to_pixels()),

                                _ => None,
                            })
                        })
                        .unwrap_or(0);
                    let layout_style = style.as_ref().map(|s| Style {
                        classes: s
                            .classes
                            .iter()
                            .filter(|c| {
                                matches!(
                                    c,
                                    StyleClass::ItemsCenter
                                        | StyleClass::ItemsStart
                                        | StyleClass::ItemsEnd
                                        | StyleClass::JustifyStart
                                        | StyleClass::JustifyEnd
                                        | StyleClass::JustifyCenter
                                        | StyleClass::JustifyBetween
                                )
                            })
                            .cloned()
                            .collect(),
                        hover_classes: Vec::new(),
                        variant_classes: Vec::new(),
                    });
                    if is_col {
                        Some(Box::new(View::Column {
                            children: views,
                            spacing,
                            padding: 0,
                            style: layout_style,
                            onclick: None,
                            on_right_click: None,
                        }))
                    } else {
                        Some(Box::new(View::Row {
                            children: views,
                            spacing,
                            padding: 0,
                            style: layout_style,
                            onclick: None,
                            on_right_click: None,
                        }))
                    }
                }
            }
        } else {
            None
        };

        // Plan 423 P3: disabled / disabled-if —— 表达式统一走 eval_condition_with
        // (支持 `.field`、比较运算与 state 引用;求值失败=禁用,保守)。
        let disabled = if let Some(cond) = self.extract_string_with(props, "disabled-if", bindings)
        {
            !self.eval_condition_with(&cond, bindings)
        } else {
            self.extract_string_with(props, "disabled", bindings)
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false)
        };

        View::Button {
            disabled,
            // PLAN-054 T1: EE03 title 尾段在此统一拼入(renderer/snapshot 按
            // EE03 拆分);label 本体保持干净,内容子树 leading Text 不受污染。
            label: if pua_title.is_empty() {
                label
            } else {
                format!("{}\u{EE03}{}", label, pua_title)
            },
            onclick,
            style,
            on_right_click,
            content,
        }
    }

    /// Plan 409 §10 续 8: badge → shadcn Badge(inline-flex 水平 + variant 配色)。
    /// 之前 badge 落 default fallback → Column(垂直) + style 丢失 + variant 忽略,
    /// 导致 icon/text 两行、outline 无边框。现对齐 vue Badge:水平排列 + 圆角
    /// 胶囊 + variant 配色(outline=边框,default/secondary/destructive=填充)。
    pub(super) fn convert_badge(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let variant = self
            .extract_string_with(props, "variant", bindings)
            .unwrap_or_default();
        let preset: &str = match variant.as_str() {
            "outline" => "border border-input text-foreground",
            "secondary" => "bg-secondary text-secondary-foreground",
            "destructive" => "bg-destructive text-destructive-foreground",
            _ => "bg-primary text-primary-foreground", // default
        };
        // shadcn Badge 基础:inline-flex items-center + 圆角胶囊 + 紧凑 padding + 小字。
        let base =
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium";
        let user = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings));
        let merged = match user {
            Some(c) => format!("{} {} {}", base, preset, c),
            None => format!("{} {}", base, preset),
        };
        // PLAN-031 T-05：本元素 design（placeholder/radius 等）并入控件 preset。
        let style = self.merge_current_design(Style::parse(&merged).ok(), bindings);
        // 子元素 → Row(水平),对齐 vue Badge 的 inline-flex(items-center 居中)。
        // Plan 438 M2: badge (text: "…", variant: "…") 形态——text prop 是
        // gallery/应用的主流通法,此前只认 children,prop 形态渲染为空 Row。
        let mut child_views: Vec<View<DynamicMessage>> = children
            .iter()
            .map(|n| self.convert_node_with(n, bindings))
            .filter(|v| !matches!(v, View::Empty))
            .collect();
        if child_views.is_empty() {
            let text = self
                .extract_string_with(props, "text", bindings)
                .unwrap_or_default();
            if !text.is_empty() {
                child_views.push(View::Text {
                    content: text,
                    style: None,
                    selectable: false,
                });
            }
        }
        View::Row {
            children: child_views,
            spacing: 0,
            padding: 0,
            style,
            onclick: None,
            on_right_click: None,
        }
    }

    /// Convert an input element.
    pub(super) fn convert_input(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let placeholder = self
            .extract_string_with(props, "placeholder", bindings)
            .or_else(|| self.extract_string_with(props, "text", bindings))
            .unwrap_or_default();

        // Resolve value from state if it's a StateRef
        let value = self
            .extract_string_with(props, "value", bindings)
            .unwrap_or_default();

        // Plan 448 / Design 22: input adopts Shadcn-Vue default preset
        // (border rounded-md px-3 py-2 text-sm)
        let user = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings));
        let default_preset = "border rounded-md px-3 py-2 text-sm";
        let merged = match user.as_deref() {
            None => default_preset.to_string(),
            Some(c) => format!("{} {}", default_preset, c),
        };
        // PLAN-031 T-05：本元素 design（placeholder/radius 等）并入控件 preset。
        let style = self.merge_current_design(Style::parse(&merged).ok(), bindings);
        let width = self.extract_u16(props, "width");
        let password = self.extract_bool(props, "password").unwrap_or(false)
            || self.extract_string(props, "type").as_deref() == Some("password");

        // Plan 062 T9:on_change/on_submit 改用绑定解析版(_with)——单行
        // input 的事件参数(如 .Filter(.block.id))此前不烘焙,渲染层收到
        // 光杆事件名,BlockItem.Filter 桥恒 id=-1(059 §4.3 过滤失效真身)。
        // 无参事件两版等价,不影响既有 input(oninput: .OnQuery 等)。
        let on_change = aura_events_get_base(events, "onchange")
            .or_else(|| aura_events_get_base(events, "change"))
            .or_else(|| aura_events_get_base(events, "oninput"))
            .or_else(|| aura_events_get_base(events, "input"))
            .map(|event| self.event_to_message_with(&event, bindings));

        let on_submit = aura_events_get_base(events, "onenter")
            .or_else(|| aura_events_get_base(events, "enter"))
            .map(|event| self.event_to_message_with(&event, bindings));

        // PLAN-031 T-05（F-02）：`disabled` 与 Button 同口径（字面量/表达式）；
        // 禁用时渲染层丢弃编辑/提交 handler（不派发）。
        let disabled = self
            .extract_string_with(props, "disabled", bindings)
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let mut builder = View::<DynamicMessage>::input(placeholder).value(value);
        if disabled {
            builder = builder.disabled(true);
        }
        if password {
            builder = builder.password();
        }
        // PLAN-031 F-02（二次修复）：禁用态在**构造期**就不挂 handler——
        // 不依赖某个渲染臂的额外门（VM 帧渲染走 render_dynamic_view，曾被漏掉）。
        if !disabled {
            if let Some(msg) = on_change {
                builder = builder.on_change(msg);
            }
            if let Some(msg) = on_submit {
                builder = builder.on_submit(msg);
            }
        }
        if let Some(w) = width {
            builder = builder.width(w);
        }
        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        builder.build()
    }

    /// PLAN-661 T-03: `slider` → View::Slider（a2r 轨 025 fixture 语义基准：
    /// value 绑 float 字段（int 容差降级）、onchange 载荷 msg（f32 实参）、
    /// step）。value/min/max/step 经 extract_f64_with 解析（StateRef/字面量
    /// 两形态）；缺省 0/100/None 对齐 vue 轨现行缺省。
    pub(super) fn convert_slider(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, auto_ui_compiler::aura::AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let value = self
            .extract_f64_with(props, "value", bindings)
            .unwrap_or(0.0) as f32;
        let min = self.extract_f64_with(props, "min", bindings).unwrap_or(0.0) as f32;
        let max = self
            .extract_f64_with(props, "max", bindings)
            .unwrap_or(100.0) as f32;
        let step = self
            .extract_f64_with(props, "step", bindings)
            .map(|s| s as f32);

        // 直构 View::Slider（不经 builder .on_change——闭包重包会丢
        // SliderChangeHandler 的标签旁路，快照 actions 面）。
        View::Slider {
            min,
            max,
            value,
            step,
            style: self.extract_style_with(props, bindings),
            on_change: self.slider_change_arm(events, bindings),
        }
    }

    /// `slider` 的 `onchange` 臂——[`Self::progress_seek_arm`] 同款载荷通道：
    /// event_to_message_with 出基消息，[`SliderChangeHandler`] 把新值作为
    /// Float 实参追加（`.SetVol(v float)` 形态，025 语义）。
    pub(super) fn slider_change_arm(
        &self,
        events: &HashMap<String, auto_ui_compiler::aura::AuraEvent>,
        bindings: &Bindings,
    ) -> Option<crate::ui::view::SliderChangeHandler<DynamicMessage>> {
        let event = aura_events_get_base(events, "onchange")
            .or_else(|| aura_events_get_base(events, "change"))?;
        let base = self.event_to_message_with(event, bindings);
        // 标签旁路：快照 actions 挂 set_value 的 handler 名（§5.3）。
        let label = extract_handler_name(&event.handler).to_string();
        Some(crate::ui::view::SliderChangeHandler::new_labeled(
            move |v: f32| match &base {
                DynamicMessage::Typed {
                    widget_name,
                    event_name,
                    args,
                } => {
                    let mut new_args = args.clone();
                    new_args.push(Value::Float(v as f64));
                    DynamicMessage::Typed {
                        widget_name: widget_name.clone(),
                        event_name: event_name.clone(),
                        args: new_args,
                    }
                }
                other => other.clone(),
            },
            &label,
        ))
    }

    /// Convert a select element.
    /// Plan 446 批五 U4: `select { option "a" {} … }` → View::Select。
    /// OS-016（os-config 走查衍生）三契约补齐：
    ///   D2 选项收集支持 for 循环子节点——逐迭代绑定循环变量后扫循环体内
    ///      的 option 元素（数据驱动选项;此前只认直接 option 子节点,循环
    ///      选项全漏 → options=[] → pick_list 空壳,os-config 016 批实况）。
    ///   D3 option 双属性：(label,value) 平行数组——label 供 pick_list 展示,
    ///      value 供选中态匹配与回调回传;value prop 缺省回退 label 文本
    ///      （兼容 446-U4 纯文本形态）。选中态匹配:select 的 value/selected
    ///      prop 先对 value 数组,再回退 label 文本（旧行为）。
    ///   D4 回调参数：事件参数照 event_to_message_with 同款静态解析（循环
    ///      绑定/字面量）,含 `$event` 的参数位在回调时替换为所选 value
    ///      （vue 轨 `.Apply(e, $event.target.value)` 的 vm 对位）;无 `$event`
    ///      位保留 446-U4 单串编码契约（`name\u{1F}s\u{1F}value`,handler 以
    ///      选中值为实参;decode_payload 在 on_with_input_for 消费）。
    pub(super) fn convert_select(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let mut labels: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();
        self.collect_select_options(children, bindings, &mut labels, &mut values);

        let cur = self
            .extract_string_with(props, "value", bindings)
            .or_else(|| self.extract_string_with(props, "selected", bindings));
        let selected_index = self
            .extract_string_with(props, "index", bindings)
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|i| *i < labels.len().max(1))
            .or_else(|| {
                let cur = cur?;
                values
                    .iter()
                    .position(|v| v == &cur)
                    .or_else(|| labels.iter().position(|l| l == &cur))
            });
        let on_select = aura_events_get_base(events, "onselect")
            .or_else(|| aura_events_get_base(events, "select"))
            .or_else(|| aura_events_get_base(events, "onchange"))
            .or_else(|| aura_events_get_base(events, "change"))
            .map(|event| {
                let handler = extract_handler_name(&event.handler).to_string();
                let widget_name = self.widget_name.clone();
                // D4:静态参数位（循环绑定/字面量,同 event_to_message_with
                // 口径）构建期解析;`$event` 位记 None,回调时以所选 value 补位。
                let static_args: Vec<Option<Value>> = event
                    .params
                    .iter()
                    .map(|param| {
                        if param.contains("$event") {
                            None
                        } else {
                            Some(
                                self.resolve_binding_path(param, bindings)
                                    .or_else(|| self.parse_event_param_expr(param, bindings))
                                    .unwrap_or_else(|| parse_event_param_literal(param)),
                            )
                        }
                    })
                    .collect();
                let has_event_slot = static_args.iter().any(|a| a.is_none());
                let values = values.clone();
                crate::ui::view::SelectCallback::new(move |idx, selected: &str| {
                    // D3:按索引回查 value,缺失回退 pick_list 回传的 label。
                    let chosen = values
                        .get(idx)
                        .cloned()
                        .unwrap_or_else(|| selected.to_string());
                    if has_event_slot {
                        // vue 对位:.Apply(e, $event…) → Apply(e, selected_value)
                        let args: Vec<Value> = static_args
                            .iter()
                            .map(|a| a.clone().unwrap_or_else(|| Value::str(&chosen)))
                            .collect();
                        DynamicMessage::Typed {
                            widget_name: widget_name.clone(),
                            event_name: handler.clone(),
                            args,
                        }
                    } else {
                        // 446-U4 既有契约:handler 以选中值为实参（单串编码,
                        // 与循环按钮实参同一条已证 decode 路径）。
                        DynamicMessage::String(format!("{}\u{1F}s\u{1F}{}", handler, chosen))
                    }
                })
            });
        let style = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .and_then(|s| Style::parse(&s).ok());
        View::Select {
            options: labels,
            selected_index,
            on_select,
            style,
        }
    }

    /// OS-016 D2:select 选项收集——直接 option 子节点 + for 循环子节点
    /// （逐迭代绑定循环变量后扫循环体;iterable 解析口径同 for_loop_iterations:
    /// 带点路径 resolve_iterable / 平名 bindings 与索引列表 / computed 回退;
    /// 不做 matches_search 过滤——下拉选项非列表视图,不该被搜索态裁剪）。
    pub(super) fn collect_select_options(
        &self,
        children: &[AuraNode],
        bindings: &Bindings,
        labels: &mut Vec<String>,
        values: &mut Vec<String>,
    ) {
        for child in children {
            match child {
                AuraNode::Element {
                    tag,
                    props: cprops,
                    children: ckids,
                    ..
                } if tag == "option" || tag == "Option" => {
                    let label = self
                        .child_element_text(cprops, bindings)
                        .or_else(|| {
                            ckids
                                .iter()
                                .filter_map(|k| match k {
                                    AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                                    _ => None,
                                })
                                .next()
                        })
                        .unwrap_or_default();
                    if label.is_empty() {
                        continue;
                    }
                    // D3:value prop 绑定感知（循环变量 o.value 可解析）,
                    // 缺省回退 label（纯文本 option 形态）。
                    let value = self
                        .extract_string_with(cprops, "value", bindings)
                        .unwrap_or_else(|| label.clone());
                    labels.push(label);
                    values.push(value);
                }
                AuraNode::ForLoop {
                    var,
                    index,
                    iterable,
                    body,
                    ..
                } => {
                    let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
                    let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
                    let has_inner_dot = stripped.contains('.') && !stripped.starts_with("store.");
                    let array = if has_inner_dot {
                        match self.resolve_iterable(iterable, bindings) {
                            Some(elems) => auto_val::Array::from(elems),
                            None => continue,
                        }
                    } else if let Some(val) = bindings.get(state_name).cloned() {
                        match val {
                            Value::Array(arr) => arr,
                            Value::Int(id) if id >= 4_000_000 => {
                                auto_val::Array::from(self.bridge.index_list_all(id as usize))
                            }
                            Value::VmRef(r) => {
                                auto_val::Array::from(self.bridge.index_list_all(r.id))
                            }
                            _ => continue,
                        }
                    } else {
                        match self.resolve_iterable(iterable, bindings) {
                            Some(elems) => auto_val::Array::from(elems),
                            None => continue,
                        }
                    };
                    for (i, item) in array.iter().enumerate() {
                        let mut loop_bindings = bindings.clone();
                        loop_bindings.insert(var.clone(), item.clone());
                        if let Some(idx_var) = index {
                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                        }
                        self.collect_select_options(body, &loop_bindings, labels, values);
                    }
                }
                _ => {}
            }
        }
    }

    /// PLAN-066: pub(crate)——native_widget global 的无 feature 降级注册
    /// （"autodown_editor" → textarea，D-GAP-3 链经注册表保持）也经本方法。
    pub(crate) fn convert_textarea(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let placeholder = self
            .extract_string_with(props, "placeholder", bindings)
            .unwrap_or_default();

        // Plan 370: autodown_editor uses `content:` for its text; standard
        // inputs use `value:`. Accept either so editor bodies render.
        let value = self
            .extract_string_with(props, "value", bindings)
            .or_else(|| self.extract_string_with(props, "content", bindings))
            .unwrap_or_default();

        // Plan 448 / Design 22: textarea adopts Shadcn-Vue default preset
        let user_ta = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings));
        let default_ta_preset = "border rounded-md px-3 py-2 text-sm";
        let merged_ta = match user_ta.as_deref() {
            None => default_ta_preset.to_string(),
            Some(c) => format!("{} {}", default_ta_preset, c),
        };
        let style = self.merge_current_design(Style::parse(&merged_ta).ok(), bindings);
        let height = self.extract_u16(props, "height");

        let on_change = aura_events_get_base(events, "onchange")
            .or_else(|| aura_events_get_base(events, "change"))
            .or_else(|| aura_events_get_base(events, "oninput"))
            .or_else(|| aura_events_get_base(events, "input"))
            .or_else(|| aura_events_get_base(events, "onupdate"))
            .or_else(|| aura_events_get_base(events, "update"))
            .map(|event| self.event_to_message(&event.handler));

        // Plan 053 M4: textarea Enter → on_submit (mirrors convert_input).
        let on_submit = aura_events_get_base(events, "onenter")
            .or_else(|| aura_events_get_base(events, "enter"))
            .map(|event| self.event_to_message(&event.handler));

        // Plan 057 续(富文本输入):highlight/ghost props — 语法着色段与灰色
        // 建议后缀(VM 原生 Highlighter / Vue 叠加层)。缺省为空 = 存量行为。
        // PLAN-493: mentions 能力声明优先——受限名单解析(state-rooted,绝不
        // 落入 eval_computed/call_vm_fn——html 纯文本降级的 call 链 UAF 同构
        // 雷区,21bba9b34 回滚实证)→ builder 期纯函数段计算;带 mentions 时
        // highlight prop 段让位。computed/调用链名单 → None = VM 无高亮
        // 降级(迁移前现状,非回归;Vue 轨经 codegen 原生响应式消费不受限)。
        let highlight = match self.resolve_mention_names(props, bindings) {
            Some(names) => {
                let segs = mention_segments(&value, &names);
                if std::env::var("AUTO_DEBUG_MENTIONS").is_ok() {
                    eprintln!(
                        "[493-MENTIONS] names={} value={:?} segs={:?}",
                        names.len(),
                        value,
                        segs
                    );
                }
                segs
            }
            None => self.resolve_highlight_spans(props, bindings),
        };
        // PLAN-031 T-05（F-02）：textarea 同 input 的禁用口径。
        let disabled_ta = self
            .extract_string_with(props, "disabled", bindings)
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let mut builder = View::<DynamicMessage>::textarea(placeholder).value(value);
        if disabled_ta {
            builder = builder.disabled(true);
        }
        // PLAN-031 F-02（二次修复）：同 input——禁用态构造期不挂 handler。
        if !disabled_ta {
            if let Some(msg) = on_change {
                builder = builder.on_change(msg);
            }
            if let Some(msg) = on_submit {
                builder = builder.on_submit(msg);
            }
        }
        if let Some(h) = height {
            builder = builder.height(h);
        }
        if let Some(s) = style {
            builder = builder.with_style(s);
        }
        builder = builder.highlight(highlight);
        builder = builder.ghost(
            self.extract_string_with(props, "ghost", bindings)
                .unwrap_or_default(),
        );

        // Plan 057 续(Tab 补全):收集 onkeydown.* 绑定 — 规范化键名(剥
        // onkeydown. 前缀与 .prevent/.stop 修饰)→ handler。VM 端经 iced
        // key_binding 以 Binding::Custom 拦截(iced 默认静默丢弃 Tab);
        // Vue 端由 codegen 的 @keydown.* 原生处理(不经此字段)。
        // PLAN-051 C1: 修饰段改为整段过滤(exact/prevent/stop/capture/
        // self/once)——musk 的 `onkeydown.enter.exact.prevent` 此前落键
        // "enter.exact" 永不命中 iced 归一化键名 "enter"(真键盘 Enter 不
        // 派发)。vue 的 .exact=无其他修饰键,裸键名天然满足该语义。
        // 实参沿按钮 onclick 同款 event_to_message_with 烘焙(此前恒空,
        // .send(.text) 形态 Enter 派发丢文本)。
        let mut keydown = std::collections::HashMap::new();
        for (ev_key, ev) in events.iter() {
            if let Some(rest) = ev_key.strip_prefix("onkeydown.") {
                let norm = rest
                    .split('.')
                    .filter(|seg| {
                        !matches!(
                            *seg,
                            "prevent" | "stop" | "exact" | "capture" | "self" | "once"
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(".")
                    .to_lowercase();
                if !norm.is_empty() {
                    keydown.insert(norm, self.event_to_message_with(ev, bindings));
                }
            }
        }
        // N-12：禁用态连 keydown.* 绑定也不挂——Binding::Custom 会在无
        // on_change 时同样派发消息，与"禁用不派发"冲突。
        builder = builder.keydown(if disabled_ta {
            std::collections::HashMap::new()
        } else {
            keydown
        });

        // Plan 057 续(行内编辑 keymap):`keymap: .edit_mode` 绑定或静态串,
        // 缺省 "emacs"。VM 端据此选渲染层行编辑键位表;Vue 端忽略
        // (浏览器原生支持 Emacs 基础键)。
        builder = builder.keymap(
            self.extract_string_with(props, "keymap", bindings)
                .unwrap_or_else(|| "emacs".to_string()),
        );

        builder.build()
    }

    /// PLAN-009 P1: convert a `terminal` element(auto-term 引擎网格视口)。
    /// Props: `key`/`id`(状态存储键),`cols`/`rows`(cell 几何,默认
    /// 80×24),`lines`(行文本数组,数据面甲 props-feed)。交互事件随 T4
    /// 迁入(004/005 交互套件)。
    pub(super) fn convert_terminal(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let key = self
            .extract_string_with(props, "key", bindings)
            .or_else(|| self.extract_string_with(props, "id", bindings))
            .unwrap_or_else(|| "term".to_owned());
        // 014 几何随动:cols/rows 支持动态绑定(`cols: .cols`——resize 回
        // 流每拍改模型变量),eval_u16_prop 走 bindings 求值(Plan 448 I)。
        let cols = self
            .extract_u16(props, "cols")
            .or_else(|| self.eval_u16_prop(props, "cols", bindings))
            .unwrap_or(80);
        let rows = self
            .extract_u16(props, "rows")
            .or_else(|| self.eval_u16_prop(props, "rows", bindings))
            .unwrap_or(24);

        let mut lines: Vec<String> = Vec::new();
        if let Some(AuraPropValue::Expr(expr)) = props.get("lines") {
            if let Some(val) = self.resolve_expr_to_value(expr, bindings) {
                // OS-013 T3:state 里的 list 存为 heap ListData id(Plan 390
                // §15 H3b)或 VmRef——裸 as_array() 一概看不见(terminal 视口
                // 空白)。经 bridge 物化(Index 臂同族:chart props/notes[.id]
                // 先例),再逐元素取文本。
                // PLAN-013 T1:改经 read_str_list_value——内联 Array 的元素
                // 可能是负字符串表哨兵(engine_rows shim 的 ListData<i32>
                // 编码),裸 as_str() 会整屏变空串(013 at-app 实测)。
                lines = self.bridge.read_str_list_value(&val);
            }
        }

        let style = self.extract_style_with(props, bindings);
        let scroll_offset = self.extract_u16(props, "scroll_offset").unwrap_or(0);
        let preedit = self
            .extract_string_with(props, "preedit", bindings)
            .filter(|s| !s.is_empty());
        let on_select = aura_events_get_base(events, "onselect")
            .or_else(|| aura_events_get_base(events, "select"))
            .map(|event| self.event_to_message(&event.handler));
        let on_menu = aura_events_get_base(events, "oncontextmenu")
            .or_else(|| aura_events_get_base(events, "contextmenu"))
            .or_else(|| aura_events_get_base(events, "onmenu"))
            .map(|event| self.event_to_message(&event.handler));
        // 014 直键入:oninput 信号位(载荷走 TerminalCore 键入队列,宿主
        // 引擎泵排空裸写;消息只当触发器——scalar 消息不带载荷)。
        // PLAN-026 needs_fix: 改带参编码(event_to_message_with,捷径表
        // 同款)——`oninput: .KeyIn($event)` 的 $event 实参随消息编码,
        // dispatch 期由键入队列侧车替换(split 形态键入跨进程载荷通道);
        // 无参 .KeyIn 旧形态编码不变(零参消息,行为同旧)。
        let on_input = aura_events_get_base(events, "oninput")
            .or_else(|| aura_events_get_base(events, "input"))
            .or_else(|| aura_events_get_base(events, "onkey"))
            .map(|event| self.event_to_message_with(event, bindings));
        // 014 光标格:app 每拍从引擎回读喂入(0,0 = 未喂入占位)。
        let cursor_row = self
            .extract_u16(props, "cursor_row")
            .or_else(|| self.eval_u16_prop(props, "cursor_row", bindings))
            .unwrap_or(0);
        let cursor_col = self
            .extract_u16(props, "cursor_col")
            .or_else(|| self.eval_u16_prop(props, "cursor_col", bindings))
            .unwrap_or(0);
        // PLAN-028 T-05:引擎回滚历史行数(app 每拍从 nums 尾段回读喂入;
        // 0 = 未喂入占位哨兵,渲染臂据以落注册表)。
        let history = self
            .extract_i32_prop(props, "history")
            .or_else(|| self.eval_i32_prop(props, "history", bindings))
            .unwrap_or(0);
        // PLAN-018 D10:配色方案 prop(int;缺省 -1 = 跟随桌面主题)。字面量
        // 或 state 绑定求值(scheme 随主题切换的重挂场景)。
        let scheme = self
            .extract_i32_prop(props, "scheme")
            .or_else(|| self.eval_i32_prop(props, "scheme", bindings))
            .unwrap_or(crate::ui::terminal::TERMINAL_SCHEME_FOLLOW_THEME);
        // PLAN-019 D4:应用级捷径表 —— onkeydown.<键名> 事件收集(Textarea
        // Plan 057 收集器同款:剥前缀与 prevent/stop/exact/capture/self/once
        // 修饰段,小写规范化;实参经 event_to_message_with 烘焙)。命中发
        // 消息不落 VT 队列,未命中原样透传(terminal_key_binding_name 命名)。
        let mut shortcuts: Vec<(String, DynamicMessage)> = Vec::new();
        for (ev_key, ev) in events.iter() {
            if let Some(rest) = ev_key.strip_prefix("onkeydown.") {
                let norm = rest
                    .split('.')
                    .filter(|seg| {
                        !matches!(
                            *seg,
                            "prevent" | "stop" | "exact" | "capture" | "self" | "once"
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(".")
                    .to_lowercase();
                if !norm.is_empty() {
                    shortcuts.push((norm, self.event_to_message_with(ev, bindings)));
                }
            }
        }
        View::Terminal {
            key,
            cols,
            rows,
            lines,
            scroll_offset,
            preedit,
            on_select,
            on_menu,
            on_input,
            cursor_row,
            cursor_col,
            history,
            scheme,
            shortcuts,
            style,
        }
    }

    /// Plan 413: convert a `code_editor` element. Sub-element properties:
    /// `content:`/`value:` (external value), `oninput:`/`onchange:` (text
    /// changed), `oncursor:` (caret moved), `oncontextmenu:` (right click).
    /// Config props: `lang`, `line_numbers`, `wrap`, `vi`,
    /// `highlight_current_line`, `tab_width`, `font_size`, `key`.
    pub(super) fn convert_code_editor(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let key = self
            .extract_string_with(props, "key", bindings)
            .or_else(|| self.extract_string_with(props, "id", bindings))
            .unwrap_or_else(|| "editor".to_owned());

        // External value; content: mirrors textarea/autodown conventions.
        let value = self
            .extract_string_with(props, "value", bindings)
            .or_else(|| self.extract_string_with(props, "content", bindings))
            .unwrap_or_default();

        let lang = self
            .extract_string_with(props, "lang", bindings)
            .or_else(|| self.extract_string_with(props, "language", bindings))
            .unwrap_or_else(|| "none".to_owned());

        let bool_prop = |name: &str, default: bool| -> bool {
            props
                .get(name)
                .and_then(|v| match v {
                    AuraPropValue::Expr(expr) => self
                        .resolve_expr_to_value(expr, bindings)
                        .map(|val| val.as_bool()),
                    _ => None,
                })
                .unwrap_or(default)
        };

        let line_numbers = bool_prop("line_numbers", true);
        let wrap = bool_prop("wrap", false);
        let vi = bool_prop("vi", false);
        let highlight_current_line = bool_prop("highlight_current_line", true);
        // PLAN-089(musk T-09): readonly viewer (bool prop, binding-aware).
        let readonly = bool_prop("readonly", false);

        let tab_width = self.extract_u16(props, "tab_width").unwrap_or(4) as usize;
        let font_size = self
            .extract_string_with(props, "font_size", bindings)
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(15.0);

        // PLAN-670 F-W3:on_change/on_cursor 改用绑定解析版(_with)——
        // code_editor 的事件参数(如 .SrcChanged(i) 的循环变量 i)此前经光杆
        // 版落 parse_event_param_literal 空串,int 形参收 Str("") → handler
        // IndexError(auto-edit 矩阵 18 次)。Plan 062 T9 input 同款镜像;
        // 无参事件两版等价。
        let on_change = aura_events_get_base(events, "oninput")
            .or_else(|| aura_events_get_base(events, "input"))
            .or_else(|| aura_events_get_base(events, "onchange"))
            .or_else(|| aura_events_get_base(events, "change"))
            .or_else(|| aura_events_get_base(events, "onupdate"))
            .or_else(|| aura_events_get_base(events, "update"))
            .map(|event| self.event_to_message_with(&event, bindings));

        let on_cursor = aura_events_get_base(events, "oncursor")
            .or_else(|| aura_events_get_base(events, "cursor"))
            .map(|event| self.event_to_message_with(&event, bindings));

        let on_context_menu = aura_events_get_base(events, "oncontextmenu")
            .or_else(|| aura_events_get_base(events, "contextmenu"))
            .map(|event| self.event_to_message(&event.handler));

        // Plan 413 follow-up: regex search pattern (live highlight).
        let search = self
            .extract_string_with(props, "search", bindings)
            .or_else(|| self.extract_string_with(props, "query", bindings))
            .unwrap_or_default();

        let style = self.extract_style_with(props, bindings);

        let mut builder = View::<DynamicMessage>::code_editor(key)
            .value(value)
            .lang(lang)
            .line_numbers(line_numbers)
            .wrap(wrap)
            .vi(vi)
            .highlight_current_line(highlight_current_line)
            .readonly(readonly)
            .tab_width(tab_width)
            .font_size(font_size);
        if let Some(msg) = on_change {
            builder = builder.on_change(msg);
        }
        if let Some(msg) = on_cursor {
            builder = builder.on_cursor(msg);
        }
        if let Some(msg) = on_context_menu {
            builder = builder.on_context_menu(msg);
        }
        builder = builder.search(search);
        if let Some(s) = style {
            builder = builder.with_style(s);
        }
        builder.build()
    }

    /// PLAN-493: `mentions:` prop 的受限名单解析——只接受 state-rooted
    /// 形态：裸 Ident/自引用 Dot → bindings → read_state_as_vec（同步深
    /// 拷贝，引用即弃）；`.store.X`/`store.X` 按并入根态的裸字段读
    /// （D-GAP-4 约定）；str 字面量表 → 逐元素。其余形状（含任何
    /// Call/computed 链）→ None：**不进 call 链**（UAF 红线，见
    /// convert_textarea 注释）——computed 名单在 VM 轨降级为无高亮。
    pub(super) fn resolve_mention_names(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<Vec<String>> {
        let expr = match props.get("mentions") {
            Some(AuraPropValue::Expr(e)) => e,
            _ => return None,
        };
        match expr {
            Expr::Array(elems) => {
                let mut names = Vec::with_capacity(elems.len());
                for e in elems {
                    if let Expr::Str(s) = e {
                        names.push(s.as_str().to_string());
                    }
                }
                Some(names)
            }
            Expr::Ident(name) => {
                self.state_rooted_string_list(name.as_str().trim_start_matches('.'), bindings)
            }
            Expr::Dot(obj, field) => {
                let is_store_path = matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "store")
                    || matches!(obj.as_ref(),
                        Expr::Dot(inner, f) if f.as_str() == "store"
                            && matches!(inner.as_ref(), Expr::Ident(n)
                                if n.as_str() == "." || n.as_str() == "self"));
                let is_self_path = matches!(obj.as_ref(),
                    Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self");
                if is_store_path || is_self_path {
                    self.state_rooted_string_list(field.as_str(), bindings)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// PLAN-493: state-rooted 字段 → Vec<String>（bindings 命中 Array 直接
    /// 取；否则 read_state_as_vec 同步深拷贝）。Str/String 元素之外跳过。
    pub(super) fn state_rooted_string_list(&self, field: &str, bindings: &Bindings) -> Option<Vec<String>> {
        if field.is_empty() {
            return None;
        }
        if let Some(Value::Array(arr)) = bindings.get(field) {
            return Some(strings_from_values(&arr.values));
        }
        let items = self.read_state_as_vec(field).ok()?;
        Some(strings_from_values(&items))
    }

    /// Plan 057 续(富文本输入):解析 `highlight: <list of {text, kind}>` prop
    /// 为连续 (text, kind) 段。支持根状态字段、循环变量/深层路径(经
    /// resolve_expr_to_value),列表元素经 materialize_obj_ref 取字段。
    pub(super) fn resolve_highlight_spans(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Vec<(String, String)> {
        let expr = match props.get("highlight") {
            Some(AuraPropValue::Expr(e)) => e,
            _ => return Vec::new(),
        };
        let items: Vec<auto_val::Value> = match self.resolve_expr_to_value(expr, bindings) {
            Some(auto_val::Value::Array(arr)) => arr.values,
            Some(auto_val::Value::VmRef(r)) => match self.bridge.vmref_to_vec(r.id) {
                Ok(v) => v,
                Err(_) => return Vec::new(),
            },
            Some(auto_val::Value::Int(id)) if id >= 4_000_000 => {
                match self.bridge.index_list_all(id as usize) {
                    v if !v.is_empty() => v,
                    _ => return Vec::new(),
                }
            }
            _ => return Vec::new(),
        };
        items
            .iter()
            .filter_map(|item| {
                let obj = self.bridge.materialize_obj_ref(item);
                if let auto_val::Value::Obj(map) = obj {
                    let text = map
                        .get("text")
                        .map(|v| v.as_str().to_string())
                        .unwrap_or_default();
                    let kind = map
                        .get("kind")
                        .map(|v| v.as_str().to_string())
                        .unwrap_or_default();
                    if text.is_empty() && kind.is_empty() {
                        None
                    } else {
                        Some((text, kind))
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Convert a checkbox element.
    pub(super) fn convert_checkbox(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let label = self
            .extract_string(props, "text")
            .or_else(|| self.extract_string(props, "label"))
            .unwrap_or_default();

        // Resolve checked from state ref, literal, or binding path (e.g., todo.done)
        let is_checked = props
            .get("checked")
            .or_else(|| props.get("is_checked"))
            .map(|v| match v {
                AuraPropValue::Expr(expr) => self
                    .resolve_expr_to_value(expr, bindings)
                    .map(|val| val.as_bool()),
                _ => None,
            })
            .flatten()
            .unwrap_or(false);

        let on_toggle = aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "change"))
            .or_else(|| aura_events_get_base(events, "onchange"))
            .map(|event| self.event_to_message_with(event, bindings));

        let style = self.extract_style(props);

        let mut view = View::checkbox(is_checked, label);
        if let Some(msg) = on_toggle {
            view = view.on_toggle(msg);
        }
        if let Some(s) = style {
            if let View::Checkbox {
                style: ref mut st, ..
            } = view
            {
                *st = Some(s);
            }
        }
        view
    }

    // ========================================================================
    // Text content conversion
    // ========================================================================

}
