//! PLAN-014 T-04：shadcn 责任（generate_shadcn_attrs 巨表、事件/model 绑定映射、组件注册/import 发射）（自 ui_gen/vue.rs impl 块原样收编）。
//! 单 VueGenerator state、公开路径与字节输出零改——纯 impl 代码移动；
//! 方法可见性按跨模块调用需要提升（其余保持私有语义）。
use super::*;

impl VueGenerator {
    pub(super) fn generate_shadcn_attrs(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
    ) -> (Vec<String>, Option<String>, Option<String>) {
        let mut attrs = Vec::new();
        let mut slot_content: Option<String> = None;
        let mut slot_children: Option<String> = None;

        // PLAN-682 F2: explicit `key: <expr>` forwards as a dynamic `:key`
        // binding. The early-exit component arm's explicit-key contract
        // (`has_explicit_key`) never reached the shadcn curated path:
        // curated arms are prop whitelists, so `key` was silently dropped
        // and the Plan 360 catch-all only saw no `:key=` in the emitted
        // string and appended a counter literal (`:key="'Tag-N'"`) —
        // declarative bindings (e.g. an editor's `key: .store.active_key`,
        // the key-change-remounts seeding semantics) broke and hosts lost
        // remount-on-switch (jade D-03/D-17 generator half). Emitting here
        // (single point for every curated arm) makes Plan 360's
        // `contains(":key=")` check skip the counter naturally.
        if let Some(AuraPropValue::Expr(key_expr)) = props.get("key") {
            if let Ok(key_bound) = self.expr_to_vue_bound_value(key_expr) {
                attrs.push(format!(":key=\"{}\"", key_bound));
            }
        }

        // Template ref escape hatch: `ref: "menuEl"` → static `ref="menuEl"`
        // attribute + a `const menuEl = ref<HTMLElement | null>(null)` in
        // <script setup> (from self.template_refs). Handled generically so
        // every shadcn-mapped element supports it without per-arm changes.
        // Plan 040: when the tag maps to a real vue component (PascalCase —
        // e.g. autodown_editor/autodown → engine components), the ref
        // captures a component instance — type it `ref<any>` (same contract
        // as ext components; the child's defineExpose surface is unknown
        // here). Native-tag pseudo-mappings (`col` → "div") stay HTMLElement.
        if let Some(value) = props.get("ref") {
            let ref_name = match value {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(name)) => name.to_string(),
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name)) => name.to_string(),
                _ => String::new(),
            };
            if !ref_name.is_empty() {
                if !self.template_refs.contains(&ref_name) {
                    self.template_refs.push(ref_name.clone());
                }
                if self
                    .shadcn_component_name(tag)
                    .is_some_and(|n| n.chars().next().is_some_and(char::is_uppercase))
                {
                    self.component_ref_names.insert(ref_name.clone());
                }
                attrs.push(format!("ref=\"{}\"", ref_name));
            }
        }

        // Inline style object: `style_obj: { top: f"${.y}px", "z-index": 50 }`
        // → `:style="{ top: `${y}px`, 'z-index': 50 }"`. Handled generically
        // (like `ref` above) so every shadcn-mapped element supports it.
        if let Some(AuraPropValue::StyleBinding(bindings)) = props.get("style_obj") {
            attrs.push(format!(":style=\"{}\"", self.style_obj_to_vue(bindings)));
        }

        // v-show visibility directive: show: .cond → v-show="cond". Handled
        // generically (like style_obj above) so every shadcn-mapped element
        // supports it. Infallible here (no Result in scope) — an exotic expr
        // that fails conversion is not emitted, but never silently (R013).
        if let Some(AuraPropValue::Expr(expr)) = props.get("show") {
            match self.expr_to_vue_bound_value(expr) {
                Ok(cond) => attrs.push(format!("v-show=\"{}\"", cond)),
                Err(e) => self.warn(
                    "R013",
                    crate::ui_gen::validators::Severity::Warning,
                    format!("v-show on `{}`: {}; directive not emitted", tag, e),
                ),
            }
        }

        // v-html content directive: html: .content → v-html="content". Handled
        // generically (like show above) so every shadcn-mapped element
        // supports it; same infallible context, same R013-on-failure policy.
        if let Some(AuraPropValue::Expr(expr)) = props.get("html") {
            match self.expr_to_vue_bound_value(expr) {
                Ok(content) => attrs.push(format!("v-html=\"{}\"", content)),
                Err(e) => self.warn(
                    "R013",
                    crate::ui_gen::validators::Severity::Warning,
                    format!("v-html on `{}`: {}; directive not emitted", tag, e),
                ),
            }
        }

        // PLAN-781 T-04: scroll-anchor → data-scroll-anchor（shadcn 路单点
        // 转发——curated 臂是 prop 白名单，未知 prop 静默丢；此处在所有臂
        // 之前统一发射，与 plain 路同形；push_passthrough_attrs 跳过防双发）。
        if let Some(value) = props.get("scroll-anchor") {
            self.scroll_ctl_helpers_needed = true;
            self.push_scroll_anchor_attr(&mut attrs, value, tag);
        }

        // Normalize tag for matching (kebab-case -> snake_case, lowercase for case-insensitive matching)
        let normalized_tag = tag.replace('-', "_").to_lowercase();

        // Plan 012 Batch D: snapshot so the post-match choke point can tell
        // which attrs the arm itself pushed (see below).
        let attrs_before_match = attrs.len();

        match normalized_tag.as_str() {
            // === Button ===
            "button" => {
                // Handle variant prop (default, secondary, destructive, outline, ghost, link)
                // `submit` 是行为语义（form submit 布线，extract 层消费），
                // 非视觉变体——不下发 shadcn Button（其 variant 表无此值）。
                if let Some(value) = props.get("variant") {
                    if let Some(variant) = self.extract_string_value(value) {
                        if variant != "submit" {
                            attrs.push(format!("variant=\"{}\"", variant));
                        }
                    } else if let AuraPropValue::Expr(e) = value {
                        if let Ok(expr_vue) = self.expr_to_vue_bound_value(e) {
                            attrs.push(format!(":variant=\"{}\"", expr_vue));
                        } else {
                            attrs.push("variant=\"default\"".to_string());
                        }
                    }
                }
                // Handle size prop (sm, default, lg, icon)
                if let Some(value) = props.get("size") {
                    if let Some(size) = self.extract_string_value(value) {
                        attrs.push(format!("size=\"{}\"", size));
                    } else if let AuraPropValue::Expr(e) = value {
                        if let Ok(expr_vue) = self.expr_to_vue_bound_value(e) {
                            attrs.push(format!(":size=\"{}\"", expr_vue));
                        } else {
                            attrs.push("size=\"default\"".to_string());
                        }
                    }
                }
                // Handle disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // PLAN-053 批4: `title` → 原生 tooltip 属性透传（shadcn Button
                // 经 vue attr fallthrough 落到底层 button 元素；与 VM 轨
                // convert_button 的 title→EE03 iced tooltip 对齐——musk 会话
                // 侧栏 `title: .s.id` 悬停显示会话 id）。此前 button 臂静默
                // 丢弃（原生 span/div 路径本就透传，仅 shadcn Button 丢）。
                if let Some(value) = props.get("title") {
                    if let Some(title) = self.extract_string_value(value) {
                        attrs.push(format!("title=\"{}\"", title));
                    } else if let AuraPropValue::Expr(e) = value {
                        if let Ok(expr_vue) = self.expr_to_vue_bound_value(e) {
                            attrs.push(format!(":title=\"{}\"", expr_vue));
                        }
                    }
                }
                // Handle style/class prop
                self.push_style_class(&mut attrs, props);
                // Build slot children for icon + text
                let mut button_children = Vec::new();
                if let Some(icon_name) =
                    props.get("icon").and_then(|v| self.extract_string_value(v))
                {
                    let lucide_component = Self::lucide_component_name(icon_name);
                    self.lucide_icons.insert(lucide_component.clone());
                    button_children.push(format!(r#"<{} class="h-4 w-4" />"#, lucide_component));
                }
                if let Some(value) = props.get("text") {
                    if let Ok(text) = self.prop_to_text_content(value) {
                        button_children.push(text);
                    }
                }
                if !button_children.is_empty() {
                    slot_children = Some(button_children.join(""));
                }
            }

            // === Layout Elements (Row, Col, Scroll, etc.) ===
            // These always need their structural flex classes, even when user provides style/class.
            // User classes are appended after the structural defaults (deduped to avoid repetition).
            "row" => {
                let mut classes = vec!["flex".to_string(), "flex-row".to_string()];
                if let Some(value) = props.get("gap") {
                    if let Some(g) = self.extract_string_value(value) {
                        if !g.is_empty() {
                            classes.push(format!("gap-{}", g));
                        }
                    }
                }
                // Plan 053 后续: class 与 style prop 分开处理。class(类名)→ :class;
                // style(CSS 声明串,如 "display:grid; ... " + expr)→ :style。之前
                // get_style_class(style 优先)把动态 style 误送进 layout_dynamic_class_attr
                // → 产 :class → CSS 声明被当类名失效(表格 grid 列对齐就是这么坏的)。
                if let Some(value) = props.get("class") {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        // Plan 496 M5：插值 class 拆分（col/grid 臂同型）。
                        if let Some((statics, expr)) = Self::interpolated_class_parts(user_class) {
                            for c in statics {
                                for tok in c.split_whitespace() {
                                    if !classes.iter().any(|d| d == tok) {
                                        classes.push(tok.to_string());
                                    }
                                }
                            }
                            attrs.push(format!(":class=\"{}\"", expr));
                        } else {
                            for c in user_class.split_whitespace() {
                                if !classes.iter().any(|d| d == c) {
                                    classes.push(c.to_string());
                                }
                            }
                        }
                    } else if let Some(class_attr) = self.layout_dynamic_class_attr(value) {
                        attrs.push(class_attr); // PLAN-026 缺陷①: 动态 class 绑定
                    }
                }
                if let Some(value) = props.get("style") {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        // 静态 style(历史 Tailwind-in-style 用法)合并进 class。
                        for c in user_class.split_whitespace() {
                            if !classes.iter().any(|d| d == c) {
                                classes.push(c.to_string());
                            }
                        }
                    } else if let Some(style_attr) = self.layout_dynamic_style_attr(value) {
                        attrs.push(style_attr); // 动态 CSS 声明 → :style
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
                // Pass-through for arbitrary HTML attributes (e.g. draggable,
                // ondragstart handled as event elsewhere) that aren't covered
                // by the layout-specific class logic above. Mirrors the plain
                // element branch: skip class/style/gap/text/special keys; v-bind
                // expressions (:key="..."), static literals (key="...").
                self.push_passthrough_attrs(&mut attrs, props);
            }

            "col" | "column" => {
                let mut classes = vec!["flex".to_string(), "flex-col".to_string()];
                if let Some(value) = props.get("gap") {
                    if let Some(g) = self.extract_string_value(value) {
                        if !g.is_empty() {
                            classes.push(format!("gap-{}", g));
                        }
                    }
                }
                // Plan 053 后续: class 与 style prop 分开处理。class(类名)→ :class;
                // style(CSS 声明串,如 "display:grid; ... " + expr)→ :style。之前
                // get_style_class(style 优先)把动态 style 误送进 layout_dynamic_class_attr
                // → 产 :class → CSS 声明被当类名失效(表格 grid 列对齐就是这么坏的)。
                if let Some(value) = props.get("class") {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        // Plan 496 M5：插值 class 拆分（row 臂同型）。
                        if let Some((statics, expr)) = Self::interpolated_class_parts(user_class) {
                            for c in statics {
                                for tok in c.split_whitespace() {
                                    if !classes.iter().any(|d| d == tok) {
                                        classes.push(tok.to_string());
                                    }
                                }
                            }
                            attrs.push(format!(":class=\"{}\"", expr));
                        } else {
                            for c in user_class.split_whitespace() {
                                if !classes.iter().any(|d| d == c) {
                                    classes.push(c.to_string());
                                }
                            }
                        }
                    } else if let Some(class_attr) = self.layout_dynamic_class_attr(value) {
                        attrs.push(class_attr); // PLAN-026 缺陷①: 动态 class 绑定
                    }
                }
                if let Some(value) = props.get("style") {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        // Plan 496 M5：插值 class 拆分（extract_classes 同型）
                        // ——静态段并入 class，`${.field}` 段转 `:class` 拼接
                        // 表达式（desktop.at 根 bg 形态）。
                        if let Some((statics, expr)) = Self::interpolated_class_parts(user_class) {
                            for c in statics {
                                for tok in c.split_whitespace() {
                                    if !classes.iter().any(|d| d == tok) {
                                        classes.push(tok.to_string());
                                    }
                                }
                            }
                            attrs.push(format!(":class=\"{}\"", expr));
                        } else {
                            // 静态 style(历史 Tailwind-in-style 用法)合并进 class。
                            for c in user_class.split_whitespace() {
                                if !classes.iter().any(|d| d == c) {
                                    classes.push(c.to_string());
                                }
                            }
                        }
                    } else if let Some(style_attr) = self.layout_dynamic_style_attr(value) {
                        attrs.push(style_attr); // 动态 CSS 声明 → :style
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
                self.push_passthrough_attrs(&mut attrs, props);
            }

            "scroll" => {
                // ScrollArea support (Plan 105)
                // viewport class for styling
                self.push_style_class(&mut attrs, props);
                // orientation (vertical, horizontal, both)——PLAN-656:
                // legacy orientation 缺席时由 axis 映射（y→vertical,
                // x→horizontal；both 超出 ScrollArea 表达面——语义完整形态
                // 在 plain 模式 overflow-auto，spec 记录 shadcn 限制）。
                let orientation: Option<String> = props
                    .get("orientation")
                    .and_then(|v| self.extract_string_value(v).map(|s| s.to_string()))
                    .or_else(|| {
                        props
                            .get("axis")
                            .and_then(|v| self.extract_string_value(v).map(|s| s.to_string()))
                            .map(|a| {
                                if a == "x" {
                                    "horizontal".to_string()
                                } else {
                                    "vertical".to_string()
                                }
                            })
                    });
                if let Some(orientation) = orientation {
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
                // PLAN-656 T-07: controller 寻址锚 + hidden 内联样式。
                // F-781-R02（T-20）：所有权标记（与 plain 臂同款，全发射）。
                attrs.push("data-scroll-pane=\"1\"".to_string());
                if let Some(AuraPropValue::Expr(expr)) = props.get("controller") {
                    self.scroll_ctl_helpers_needed = true;
                    let bound = self.bound_value_or_warn(
                        expr,
                        "element `scroll-pane` prop `controller`",
                        "''",
                    );
                    attrs.push(format!(":data-scroll-ctl=\"{}\"", bound));
                }
                // PLAN-692: size prop → --sb-size CSS var（thumb 宽 px；ScrollArea
                // 组件消费，rail 预留 +6px hover 空间）。与 hidden 内联样式合并。
                {
                    let mut style_parts: Vec<String> = Vec::new();
                    if let Some(size) = props.get("size").and_then(|v| self.extract_int_value(v)) {
                        style_parts.push(format!("--sb-size: {}px", size));
                    }
                    if let Some(v) = props.get("scrollbar") {
                        if self.extract_string_value(v).as_deref() == Some("hidden") {
                            style_parts.push("scrollbar-width: none".to_string());
                        }
                    }
                    if !style_parts.is_empty() {
                        attrs.push(format!("style=\"{}\"", style_parts.join("; ")));
                    }
                }
                // scroll hide delay
                if let Some(value) = props.get("hide_delay") {
                    if let Some(delay) = self.extract_int_value(value) {
                        attrs.push(format!(":scroll-hide-delay=\"{}\"", delay));
                    }
                }
                // Plan 053 后续: auto_scroll: "<prop>" — auto-scroll to bottom when
                // the watched prop (a list, e.g. blocks) changes. Registers a
                // sentinel template ref + a spec; node_to_html injects a trailing
                // <div ref="..."> as the last child and generate_script emits the
                // watch + nextTick(scrollIntoView). VM target ignores this prop.
                if let Some(value) = props.get("auto_scroll") {
                    if let Some(src) = self.extract_string_value(value) {
                        let idx = self.scroll_auto_scroll.len();
                        let ref_name = format!("__scrollBottom{}", idx);
                        if !self.template_refs.contains(&ref_name) {
                            self.template_refs.push(ref_name.clone());
                        }
                        self.scroll_auto_scroll
                            .push((src.to_string(), ref_name.clone()));
                        self.pending_scroll_sentinel = Some(ref_name);
                    }
                }
            }

            "container" => {
                // Default: max-w-7xl mx-auto + user style
                let mut classes = vec!["max-w-7xl".to_string(), "mx-auto".to_string()];
                if let Some(value) = self.get_style_class(props) {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        classes.push(user_class.to_string());
                    } else if let Some(class_attr) = self.layout_dynamic_class_attr(value) {
                        attrs.push(class_attr); // PLAN-026 缺陷①: 动态 class 绑定
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
            }

            "center" => {
                // Default: flex items-center justify-center h-full + user style
                // Note: do NOT add w-full here — if user has max-w-*, mx-auto handles centering.
                // If no max-width is specified, the element fills width naturally via flex.
                let mut classes = vec![
                    "flex".to_string(),
                    "flex-col".to_string(),
                    "items-center".to_string(),
                    "justify-center".to_string(),
                    "h-full".to_string(),
                ];
                if let Some(value) = self.get_style_class(props) {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        classes.push(user_class.to_string());
                        // If user style has max-w-*, add mx-auto to center the constrained element
                        if user_class.contains("max-w-") {
                            classes.push("mx-auto".to_string());
                        }
                    } else if let Some(class_attr) = self.layout_dynamic_class_attr(value) {
                        attrs.push(class_attr); // PLAN-026 缺陷①: 动态 class 绑定
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
            }

            "grid" => {
                let mut classes = vec!["grid".to_string()];
                // Plan 448 I: dynamic cols/gap expressions bind the
                // equivalent inline styles. A dynamic Tailwind class name
                // (`grid-cols-${n}`) is invisible to JIT scanning and
                // silently does nothing — this literal-only support is what
                // forced 038-minesweeper's three per-difficulty grid blocks.
                let mut inline_style_parts: Vec<String> = Vec::new();
                // cols prop → grid-template-columns
                if let Some(value) = props.get("cols") {
                    if let Some(n) = self.extract_int_value(value) {
                        classes.push(format!("grid-cols-{}", n));
                    } else if let AuraPropValue::Expr(expr) = value {
                        if let Ok(js) = self.expr_to_vue_bound_value(expr) {
                            inline_style_parts.push(format!(
                                "'grid-template-columns: repeat(' + ({js}) + ', minmax(0, 1fr))'"
                            ));
                        }
                    }
                }
                // gap prop — Plan 412 F5: grid 的 gap prop 语义是像素(与 VM 的
                // convert_grid 一致;row/col 的 gap prop 是 Tailwind 单位、str 传入)。
                // px/4 整除 → gap-N 单位类,否则 gap-[Npx] arbitrary;
                // 表达式 → 动态内联 gap 样式(Plan 448 I)。
                if let Some(value) = props.get("gap") {
                    if let Some(n) = self.extract_int_value(value) {
                        if n >= 0 && n % 4 == 0 {
                            classes.push(format!("gap-{}", n / 4));
                        } else {
                            classes.push(format!("gap-[{}px]", n));
                        }
                    } else if let AuraPropValue::Expr(expr) = value {
                        if let Ok(js) = self.expr_to_vue_bound_value(expr) {
                            inline_style_parts.push(format!("'gap: ' + ({js}) + 'px'"));
                        }
                    } else {
                        classes.push("gap-4".to_string());
                    }
                }
                if let Some(value) = self.get_style_class(props) {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        classes.push(user_class.to_string());
                    } else if let Some(class_attr) = self.layout_dynamic_class_attr(value) {
                        attrs.push(class_attr); // PLAN-026 缺陷①: 动态 class 绑定
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
                if !inline_style_parts.is_empty() {
                    // Merge into an existing :style if one is present (the
                    // grid arm itself emits none today; defensive only).
                    let joined = inline_style_parts.join(" + '; ' + ");
                    if let Some(pos) = attrs.iter().position(|a| a.starts_with(":style=\"")) {
                        let old = attrs[pos].clone();
                        let inner = old
                            .strip_prefix(":style=\"")
                            .and_then(|s| s.strip_suffix('"'))
                            .unwrap_or("");
                        attrs[pos] = format!(":style=\"({inner}) + '; ' + ({joined})\"");
                    } else {
                        attrs.push(format!(":style=\"{joined}\""));
                    }
                }
            }

            // === Plan 412 §4.3: square 占位块 ===
            // 色块 + 双轴居中内容。color/size/h/w/class props → Tailwind 类;
            // text prop → 纯文本内容(继承 div 的 text-{c}-600);children 优先。
            // 类串与 VM convert_square 对应(VM 用 Container center_x/y 居中,
            // 此处用 flex items-center justify-center)。
            "square" => {
                let color = props
                    .get("color")
                    .and_then(|v| self.extract_string_value(v))
                    .filter(|s| !s.is_empty())
                    .unwrap_or("blue");
                let size = props
                    .get("size")
                    .and_then(|v| self.extract_int_value(v))
                    .unwrap_or(12) as i64;
                // 维度:"full" 或数字(Tailwind 单位),缺省回退 size。
                let dim = |v: Option<&AuraPropValue>| -> Option<String> {
                    match v.and_then(|pv| self.extract_string_value(pv)) {
                        Some(s) if s == "full" => Some("full".to_string()),
                        Some(s) if !s.is_empty() => Some(s.to_string()),
                        _ => v
                            .and_then(|pv| self.extract_int_value(pv))
                            .map(|n| n.to_string()),
                    }
                };
                let h = dim(props.get("h")).unwrap_or_else(|| size.to_string());
                let w = dim(props.get("w")).unwrap_or_else(|| size.to_string());
                let mut classes = vec![
                    format!("h-{}", h),
                    format!("w-{}", w),
                    "rounded-md".to_string(),
                    "border".to_string(),
                    format!("bg-{}-500/40", color),
                    format!("border-{}-500", color),
                    format!("text-{}-600", color),
                    "font-medium".to_string(),
                    "text-sm".to_string(),
                    "flex".to_string(),
                    "items-center".to_string(),
                    "justify-center".to_string(),
                ];
                if let Some(value) = props.get("class") {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    for c in user_class.split_whitespace() {
                        if !classes.iter().any(|d| d == c) {
                            classes.push(c.to_string());
                        }
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
                if let Some(value) = props.get("text") {
                    if let Ok(text) = self.prop_to_text_content(value) {
                        slot_children = Some(text);
                    }
                }
            }

            "grid-item" | "grid_item" => {
                self.push_style_class(&mut attrs, props);
            }

            // === Link (Navigation Link) ===
            "link" => {
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // href (optional, if present use as link destination)
                if let Some(value) = props.get("href") {
                    let href = self.extract_string_value(value).unwrap_or("#");
                    attrs.push(format!("href=\"{}\"", href));
                }
            }

            // === Nav Link (Sidebar navigation item) ===
            "nav_link" => {
                // Plan 408: NavLink uses 'href' prop, but router-link needs 'to'.
                // Fall back to href if to is not set.
                if let Some(value) = props.get("to").or_else(|| props.get("href")) {
                    let to = self.extract_string_value(value).unwrap_or("#");
                    attrs.push(format!("to=\"{}\"", to));
                }
                if let Some(label) = props
                    .get("label")
                    .and_then(|v| self.extract_string_value(v))
                {
                    let icon_name = props.get("icon").and_then(|v| self.extract_string_value(v));
                    if let Some(icon) = icon_name {
                        let lucide_component = Self::lucide_component_name(icon);
                        self.lucide_icons.insert(lucide_component.clone());
                        slot_children = Some(format!(
                            r#"<div class="flex flex-row items-center gap-2 rounded-md px-2 py-1.5 text-sm"><{} class="h-4 w-4 shrink-0" /><span>{}</span></div>"#,
                            lucide_component, label
                        ));
                    } else {
                        slot_children = Some(format!(r#"<span>{}</span>"#, label));
                    }
                }
            }

            // === CodeBlock ===
            "codeblock" => {
                // lang prop for language identifier
                if let Some(value) = props.get("lang") {
                    let lang = self.extract_string_value(value).unwrap_or("text");
                    attrs.push(format!("data-lang=\"{}\"", lang));
                }
                // code content becomes slot content
                if let Some(value) = props.get("code") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === PreviewCard ===
            "previewcard" | "preview-card" => {
                // title prop (default: "Preview")
                let title = if let Some(value) = props.get("title") {
                    self.extract_string_value(value)
                        .unwrap_or("Preview")
                        .to_string()
                } else {
                    "Preview".to_string()
                };
                // auto and vue props are stored as data attributes for the code section
                if let Some(value) = props.get("auto") {
                    if let Some(auto_code) = self.extract_string_value(value) {
                        attrs.push(format!(
                            "data-auto=\"{}\"",
                            auto_code.replace("\"", "&quot;").replace("<", "&lt;")
                        ));
                    }
                }
                if let Some(value) = props.get("vue") {
                    if let Some(vue_code) = self.extract_string_value(value) {
                        attrs.push(format!(
                            "data-vue=\"{}\"",
                            vue_code.replace("\"", "&quot;").replace("<", "&lt;")
                        ));
                    }
                }
                let _ = title; // Suppress unused variable warning
            }

            // === AutoDownEditor (Plan 354 Phase C) ===
            // Rich AutoDown WYSIWYG editor (Tiptap) consumed from @autodown/editor.
            // Maps snake_case AURA props to the wrapper's camelCase Vue props.
            "autodown_editor" | "autodowneditor" => {
                // content → :content (bound to the note body markdown).
                // Supports state refs (content: .body → body) and field access
                // (content: .note.body → note.body).
                match props.get("content") {
                    Some(AuraPropValue::Expr(expr)) => {
                        match self.expr_to_vue_bound_value(expr) {
                            Ok(js_expr) => attrs.push(format!(":content=\"{}\"", js_expr)),
                            // Plan 012 P0#13 follow-up: was silently skipped.
                            Err(e) => self.warn(
                                "R013",
                                crate::ui_gen::validators::Severity::Warning,
                                format!("autodown_editor `content`: {}; prop not emitted", e),
                            ),
                        }
                    }
                    Some(value) => {
                        // Literal string content
                        let content = self.extract_string_value(value).unwrap_or("");
                        attrs.push(format!("content=\"{}\"", content));
                    }
                    None => {}
                }

                // placeholder → placeholder="..."（字面）/ :placeholder（绑定）
                // —— plan 040 契约扩展（AutoDownEditor 空态提示文案）。
                match props.get("placeholder") {
                    Some(AuraPropValue::Expr(expr)) => match self.expr_to_vue_bound_value(expr) {
                        Ok(js_expr) => attrs.push(format!(":placeholder=\"{}\"", js_expr)),
                        Err(e) => self.warn(
                            "R013",
                            crate::ui_gen::validators::Severity::Warning,
                            format!("autodown_editor `placeholder`: {}; prop not emitted", e),
                        ),
                    },
                    Some(value) => {
                        let placeholder = self.extract_string_value(value).unwrap_or("");
                        attrs.push(format!("placeholder=\"{}\"", placeholder));
                    }
                    None => {}
                }

                // can_edit → :canEdit (bool). Defaults to true when omitted so
                // the editor is interactive by default.
                attrs.push(self.bool_prop_binding(props, "can_edit", "canEdit", true));

                // show_actions → :showActions (bool). Defaults to true.
                attrs.push(self.bool_prop_binding(props, "show_actions", "showActions", true));

                // PLAN-051 T5: theme declaration entry — dark_mode → :dark-mode
                // (bound .dark_mode state flips the editor pane's .is-dark);
                // accent → :accent (string/state → data-accent). Defaults
                // false/indigo = the pre-051 look, byte-for-byte.
                attrs.push(self.bool_prop_binding(props, "dark_mode", "darkMode", false));
                match props.get("accent") {
                    Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))) => {
                        attrs.push(format!("accent=\"{}\"", s));
                    }
                    Some(AuraPropValue::Expr(expr)) => match self.expr_to_vue_bound_value(expr) {
                        Ok(js_expr) => attrs.push(format!(":accent=\"{}\"", js_expr)),
                        Err(e) => self.warn(
                            "R013",
                            crate::ui_gen::validators::Severity::Warning,
                            format!("autodown_editor `accent`: {}; prop not emitted", e),
                        ),
                    },
                    _ => {}
                }

                // style/class (editor chrome sizing).
                self.push_style_class(&mut attrs, props);
                // NOTE: events (onupdate/onsave/oncancel → @update/@save/@cancel)
                // are attached by the generic event loop at the end of this fn.
            }

            // === AutoDown 只读渲染（Plan 040 主名翻转）===
            // StreamingRenderer（@autodown/engine）——MarkdownRender 的超集
            // （内部组合 MarkdownRender 渲染段落），换绑不丢能力（plan 022
            // §10 的 MarkdownRender 直绑路径退役）。content → :source（绑定）
            // / source（字面）。streaming 族 props 进契约（plan 040）：
            // 显式 streaming 优先；否则 final 反相发射为 :streaming="!final"；
            // placeholder_block_id / placeholder_height / scroll_sync snake→
            // kebab 映射。VM 臂忽略并登记豁免（PLAN-042/043）。
            "autodown" | "markdown" | "markdown_editor" => {
                match props.get("content") {
                    Some(AuraPropValue::Expr(expr)) => {
                        match self.expr_to_vue_bound_value(expr) {
                            Ok(js_expr) => attrs.push(format!(":source=\"{}\"", js_expr)),
                            // Plan 012 P0#13 follow-up: was silently skipped.
                            Err(e) => self.warn(
                                "R013",
                                crate::ui_gen::validators::Severity::Warning,
                                format!("autodown `content`: {}; prop not emitted", e),
                            ),
                        }
                    }
                    Some(value) => {
                        // Literal string source.
                        let content = self.extract_string_value(value).unwrap_or("");
                        attrs.push(format!("source=\"{}\"", content));
                    }
                    None => {}
                }
                // streaming：显式 prop 优先；否则 final 反相（streaming = !final）。
                if props.contains_key("streaming") {
                    attrs.push(self.bool_prop_binding(props, "streaming", "streaming", false));
                } else if let Some(value) = props.get("final") {
                    match value {
                        AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => {
                            attrs.push(format!(":streaming=\"{}\"", !b));
                        }
                        AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => {
                            attrs.push(format!(":streaming=\"{}\"", s != "true"));
                        }
                        AuraPropValue::Expr(expr) => match self.expr_to_vue_bound_value(expr) {
                            Ok(js_expr) => {
                                attrs.push(format!(":streaming=\"!({})\"", js_expr));
                            }
                            Err(_) => {
                                attrs.push(":streaming=\"false\"".to_string());
                            }
                        },
                        _ => attrs.push(":streaming=\"false\"".to_string()),
                    }
                }
                // placeholder_block_id → :placeholder-block-id（VM v1 忽略）。
                match props.get("placeholder_block_id") {
                    Some(AuraPropValue::Expr(expr)) => {
                        if let Ok(js_expr) = self.expr_to_vue_bound_value(expr) {
                            attrs.push(format!(":placeholder-block-id=\"{}\"", js_expr));
                        }
                    }
                    Some(value) => {
                        if let Some(s) = self.extract_string_value(value) {
                            attrs.push(format!("placeholder-block-id=\"{}\"", s));
                        }
                    }
                    None => {}
                }
                // placeholder_height → :placeholder-height（数字/绑定；VM v1 忽略）。
                match props.get("placeholder_height") {
                    Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Int(n))) => {
                        attrs.push(format!(":placeholder-height=\"{}\"", n));
                    }
                    Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Float(f, _))) => {
                        attrs.push(format!(":placeholder-height=\"{}\"", f));
                    }
                    Some(AuraPropValue::Expr(expr)) => {
                        if let Ok(js_expr) = self.expr_to_vue_bound_value(expr) {
                            attrs.push(format!(":placeholder-height=\"{}\"", js_expr));
                        }
                    }
                    _ => {}
                }
                // scroll_sync → :scroll-sync（默认 true；VM v1 忽略）。
                attrs.push(self.bool_prop_binding(props, "scroll_sync", "scrollSync", true));
                // PLAN-051 T5: theme declaration entry — dark_mode → :dark-mode
                // (bound .dark_mode state flips the render pane's .is-dark);
                // accent → :accent (string/state → data-accent). Defaults
                // false/indigo = the pre-051 look, byte-for-byte.
                attrs.push(self.bool_prop_binding(props, "dark_mode", "darkMode", false));
                match props.get("accent") {
                    Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))) => {
                        attrs.push(format!("accent=\"{}\"", s));
                    }
                    Some(AuraPropValue::Expr(expr)) => match self.expr_to_vue_bound_value(expr) {
                        Ok(js_expr) => attrs.push(format!(":accent=\"{}\"", js_expr)),
                        Err(e) => self.warn(
                            "R013",
                            crate::ui_gen::validators::Severity::Warning,
                            format!("autodown `accent`: {}; prop not emitted", e),
                        ),
                    },
                    _ => {}
                }
                // style/class (wrapper sizing).
                self.push_style_class(&mut attrs, props);
            }

            // === ChatMessage ===
            // Plan 400 B-phase: bind role/content/timestamp/thinking props.
            // Plan 015 P0#3: iterate in sorted key order — props is a HashMap
            // (RandomState), so the unsorted iteration order churns the
            // generated attribute order across builds (jade gap 56).
            "chat_message" | "ChatMessage" => {
                let mut sorted_props: Vec<(&String, &AuraPropValue)> = props.iter().collect();
                sorted_props.sort_by(|a, b| a.0.cmp(b.0));
                for (key, value) in sorted_props {
                    if let AuraPropValue::Expr(expr) = value {
                        if let Ok(v) = self.expr_to_vue_bound_value(expr) {
                            attrs.push(format!(":{}=\"{}\"", key, v));
                        }
                    }
                }
            }

            // === Input ===
            "input" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    // Plan 028 F5: also fold Index-position values (dynamic
                    // keys, `.answers[q.id]`).
                    if let Some(model) = self
                        .extract_state_ref(value)
                        .or_else(|| self.extract_state_index_ref(value))
                    {
                        // Prop-backed value (v-model contract child widget):
                        // props are read-only, so emit one-way :modelValue and
                        // let the event handler emit update:modelValue upward.
                        if self.prop_names.iter().any(|p| p == &model) {
                            attrs.push(format!(":modelValue=\"{}\"", model));
                        } else {
                            attrs.push(format!("v-model=\"{}\"", model));
                        }
                    }
                }
                // type prop
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("text");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // style/class
                self.push_style_class(&mut attrs, props);
            }

            // === Label ===
            "label" => {
                // for prop (link to input id)
                if let Some(value) = props.get("for") {
                    let for_val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("for=\"{}\"", for_val));
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // Plan 012: label is emitted as a NATIVE <label> even in shadcn
                // mode (registry maps it to the native tag), so it must get the
                // full plain-path class handling — static `class`, dynamic
                // `:class` exprs and conditional ternaries (Batch A gap 20).
                // Without this the shadcn path silently dropped `class:`.
                self.push_native_classes(&mut attrs, tag, props);
                // Plan 481: selectable: true → 显式 user-select: text(防应用级
                // none 吞掉;浏览器默认可选是 vue 现状,prop 双端语义声明 +
                // a2vue 金样锚点)。静态 style 与 :style 绑定 Vue 合并,不冲突。
                if let Some(value) = props.get("selectable") {
                    if self.extract_bool_value(value) {
                        attrs.push("style=\"user-select: text\"".to_string());
                    }
                }
            }

            // === Text (Typography) ===
            "text" | "Text" | "span" | "Span" | "p" | "P" => {
                // Extract class/style for Tailwind
                self.push_style_class(&mut attrs, props);
                // Plan 481: selectable: true → 显式 user-select: text
                // (同 label 臂;缺省零改动)。
                if let Some(value) = props.get("selectable") {
                    if self.extract_bool_value(value) {
                        attrs.push("style=\"user-select: text\"".to_string());
                    }
                }
                // Text content becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Headings (Typography) ===
            "h1" | "H1" | "h2" | "H2" | "h3" | "H3" | "h4" | "H4" | "h5" | "H5" | "h6" | "H6" => {
                // Extract class/style for Tailwind
                self.push_style_class(&mut attrs, props);
                // Text content becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === CodeEditor (Plan 413) ===
            "code_editor" => {
                // v-model for value (content: preferred, mirrors textarea).
                if let Some(value) = props.get("content").or_else(|| props.get("value")) {
                    if let Some(model) = self
                        .extract_state_ref(value)
                        .or_else(|| self.extract_state_index_ref(value))
                    {
                        if self.prop_names.iter().any(|p| p == &model) {
                            attrs.push(format!(":modelValue=\"{}\"", model));
                        } else {
                            attrs.push(format!("v-model=\"{}\"", model));
                        }
                    } else if let Some(lit) = self.extract_string_value(value) {
                        // PLAN-692: 字面串 content（画廊 AutoLang 示例形态）此前被
                        // 静默丢弃——CodeEditor 挂载即空白。字面量不可写，走单向
                        // 初始值 `:model-value`（编辑器内部状态自持，update 无人
                        // 消费即不回写）。JS 单引号字面量 + `"` 实体化（属性分隔符
                        // 保护；Vue 属性值先做实体解码再编译表达式）。
                        let js = Self::escape_js_string(lit).replace('"', "&quot;");
                        attrs.push(format!(":model-value=\"'{}'\"", js));
                    }
                }
                // Static config props.
                if let Some(value) = props.get("lang") {
                    if let Some(lang) = self.extract_string_value(value) {
                        attrs.push(format!("lang=\"{}\"", lang));
                    }
                }
                let bool_attr =
                    |attrs: &mut Vec<String>,
                     key: &str,
                     prop: &str,
                     default: bool,
                     props: &HashMap<String, AuraPropValue>| {
                        let on = match props.get(key) {
                            Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b))) => *b,
                            _ => default,
                        };
                        attrs.push(format!(":{}=\"{}\"", prop, on));
                    };
                bool_attr(&mut attrs, "line_numbers", "line-numbers", true, props);
                bool_attr(&mut attrs, "wrap", "wrap", false, props);
                bool_attr(&mut attrs, "vi", "vi", false, props);
                bool_attr(
                    &mut attrs,
                    "highlight_current_line",
                    "highlight-current-line",
                    true,
                    props,
                );
                // PLAN-089(musk T-09): readonly viewer → CodeEditor.vue 的
                // Codemirror editable/readOnly 扩展（壳同计划补位）。
                bool_attr(&mut attrs, "readonly", "readonly", false, props);
                if let Some(value) = props.get("tab_width") {
                    if let Some(n) = self.extract_int_value(value) {
                        attrs.push(format!(":tab-width=\"{}\"", n));
                    }
                }
                if let Some(value) = props.get("font_size") {
                    if let Some(n) = self.extract_int_value(value) {
                        attrs.push(format!(":font-size=\"{}\"", n));
                    }
                }
                // Regex search binding (state ref or literal).
                if let Some(value) = props.get("search").or_else(|| props.get("query")) {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!(":search=\"{}\"", model));
                    } else if let Some(lit) = self.extract_string_value(value) {
                        attrs.push(format!("search=\"{}\"", lit));
                    }
                }
                // key → element id (state identity for the shell).
                if let Some(value) = props.get("key").or_else(|| props.get("id")) {
                    if let Some(k) = self.extract_string_value(value) {
                        attrs.push(format!("data-editor-key=\"{}\"", k));
                    } else {
                        // PLAN-677 T-05: 动态 key（循环内 `key: t.key` 等
                        // 表达式形态）→ 绑定 editor-key。store 侧
                        // code_editor_* 内建按同一键经 editorBridge 路由到
                        // 对应 CodeMirror 实例（vm 侧按键寻址的同构臂）。
                        if let Some(AuraPropValue::Expr(e)) = Some(value) {
                            if let Ok(expr) = self.expr_to_vue_bound_value(e) {
                                attrs.push(format!(":editor-key=\"{}\"", expr));
                            }
                        }
                    }
                }
            }

            // === Textarea ===
            "textarea" => {
                // Plan 057 续:highlight/ghost 由叠加层包裹消费,不透传。
                if props.contains_key("highlight") || props.contains_key("ghost") {
                    // handled by textarea_rich_overlay at the emission site
                }
                // v-model for value
                if let Some(value) = props.get("value") {
                    // Plan 028 F5: also fold Index-position values (dynamic
                    // keys, `.answers[q.id]`).
                    if let Some(model) = self
                        .extract_state_ref(value)
                        .or_else(|| self.extract_state_index_ref(value))
                    {
                        // Prop-backed value (v-model contract child widget):
                        // one-way :modelValue, update goes out via the handler.
                        if self.prop_names.iter().any(|p| p == &model) {
                            attrs.push(format!(":modelValue=\"{}\"", model));
                        } else {
                            attrs.push(format!("v-model=\"{}\"", model));
                        }
                    }
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // rows
                if let Some(value) = props.get("rows") {
                    let rows = self.extract_int_value(value).unwrap_or(3);
                    attrs.push(format!(":rows=\"{}\"", rows));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // style/class
                self.push_style_class(&mut attrs, props);
            }

            // === Checkbox ===
            "checkbox" => {
                // reka-ui CheckboxRoot uses modelValue (not checked), so use v-model / :model-value
                if let Some(value) = props.get("checked") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    } else if self.extract_bool_value(value) {
                        // Static true value - use :model-value for controlled mode
                        attrs.push(":model-value=\"true\"".to_string());
                    } else if let AuraPropValue::Expr(expr) = value {
                        // Dynamic expression (e.g., todo.done) — one-way :model-value binding
                        match self.expr_to_vue_bound_value(expr) {
                            Ok(js_expr) => attrs.push(format!(":model-value=\"{}\"", js_expr)),
                            // Plan 012 P0#13 follow-up: was silently skipped.
                            Err(e) => self.warn(
                                "R013",
                                crate::ui_gen::validators::Severity::Warning,
                                format!("checkbox `checked` (shadcn): {}; binding not emitted", e),
                            ),
                        }
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Switch/Toggle ===
            "toggle" | "switch" => {
                // v-model:checked for checked state (dynamic) or :default-checked (static)
                if let Some(value) = props.get("checked") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:checked=\"{}\"", model));
                    } else if self.extract_bool_value(value) {
                        // Static true value - use default-checked for uncontrolled mode
                        attrs.push(":default-checked=\"true\"".to_string());
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Select ===
            "select" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // Plan 012: forward `class:`/`style:` to the shadcn Select root
                // (Vue attr fallthrough) instead of silently dropping them.
                self.push_native_classes(&mut attrs, tag, props);
            }

            // === SelectItem ===
            "selectitem" | "select_item" => {
                // value for selection
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === SelectValue ===
            "selectvalue" | "select_value" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            // === SelectTrigger ===
            "selecttrigger" | "select_trigger" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === SelectLabel ===
            "selectlabel" | "select_label" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Slider ===
            // PLAN-661 T-04（R-5）：原生 `<input type="range">` 属性面——
            // shadcn-vue Slider 组件路径退役（Plan 408 :default-value 数组
            // 值不同步旧限）。value 响应式单向绑定（:value），回写走事件
            // 臂的 @input 载荷派发（valueAsNumber，对齐 a2r f32 语义）。
            "slider" => {
                attrs.push("type=\"range\"".to_string());
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        // prop 只读面单向绑定（v-model 同款纪律，041a⑤）。
                        if self.prop_names.iter().any(|p| p == &model) {
                            attrs.push(format!(":modelValue=\"{}\"", model));
                        } else {
                            attrs.push(format!(":value=\"{}\"", model));
                        }
                    } else if let Some(val) = self.extract_float_value(value) {
                        attrs.push(format!("value=\"{}\"", val));
                    }
                }
                if let Some(value) = props.get("min") {
                    let min = self.extract_float_value(value).unwrap_or(0.0);
                    attrs.push(format!("min=\"{}\"", min));
                }
                if let Some(value) = props.get("max") {
                    let max = self.extract_float_value(value).unwrap_or(100.0);
                    attrs.push(format!("max=\"{}\"", max));
                }
                if let Some(value) = props.get("step") {
                    let step = self.extract_float_value(value).unwrap_or(1.0);
                    attrs.push(format!("step=\"{}\"", step));
                }
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // 原生元素——plain 路径 class 通道（label 臂同款）。
                self.push_native_classes(&mut attrs, tag, props);
            }

            // === Progress ===
            "progress" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    } else if let Some(int_val) = self.extract_int_value(value) {
                        attrs.push(format!(":model-value=\"{}\"", int_val));
                    } else {
                        // Plan 043: dynamic expression (e.g. a tuple-cell text like
                        // field[1].Text) — render the expr and bind to :model-value.
                        // Progress's model-value expects a number; the expression may
                        // be a string cell (e.g. "42"), so wrap in Number(...) to
                        // satisfy vue-tsc and coerce at runtime.
                        if let AuraPropValue::Expr(expr) = value {
                            match self.expr_to_vue_bound_value(expr) {
                                Ok(expr_str) => {
                                    attrs.push(format!(":model-value=\"Number({})\"", expr_str))
                                }
                                // Plan 012 P0#13 follow-up: was silently skipped.
                                Err(e) => self.warn(
                                    "R013",
                                    crate::ui_gen::validators::Severity::Warning,
                                    format!("progress `value`: {}; binding not emitted", e),
                                ),
                            }
                        }
                    }
                }
                // max
                if let Some(value) = props.get("max") {
                    let max = self.extract_int_value(value).unwrap_or(100);
                    attrs.push(format!(":max=\"{}\"", max));
                }
            }

            // === Badge ===
            "badge" => {
                // variant
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // Handle style/class prop
                self.push_style_class(&mut attrs, props);
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Card ===
            "card" => {
                // Card variant (default, outline, ghost)
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // Card title becomes header
                if let Some(value) = props.get("title") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Tabs ===
            "tabs" => {
                // v-model for active tab value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    } else if let Some(val) = self.extract_string_value(value) {
                        attrs.push(format!("default-value=\"{}\"", val));
                    }
                }
                // default value
                if let Some(value) = props.get("default") {
                    let default = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("default-value=\"{}\"", default));
                }
                // PLAN-641：defaultvalue 别名（schema 词表名；default 为历史拼写）。
                if !attrs.iter().any(|a| a.starts_with("default-value=")) {
                    if let Some(value) = props.get("defaultvalue") {
                        if let Some(val) = self.extract_string_value(value) {
                            attrs.push(format!("default-value=\"{}\"", val));
                        }
                    }
                }
                // PLAN-641：形态 prop 直传 registry Tabs（default|enclosed）。
                if let Some(value) = props.get("variant") {
                    if let Some(val) = self.extract_string_value(value) {
                        attrs.push(format!("variant=\"{}\"", val));
                    }
                }
            }

            // === Tab ===
            "tab" => {
                // Tab value (required for TabsTrigger)
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // Disabled state
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Separator/Divider ===
            "divider" | "separator" => {
                // orientation (horizontal, vertical)
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
                // decorative (accessibility)
                if let Some(value) = props.get("decorative") {
                    if self.extract_bool_value(value) {
                        attrs.push("decorative".to_string());
                    }
                }
                // label for accessibility
                if let Some(value) = props.get("label") {
                    let label = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("label=\"{}\"", label));
                }
            }

            // === AlertDialog Sub-components ===
            "alertdialog" | "alert_dialog" => {
                // v-model:open for dialog state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    } else if let AuraPropValue::Expr(expr) = value {
                        if let Ok(js_expr) = self.expr_to_vue_bound_value(expr) {
                            attrs.push(format!(":open=\"{}\"", js_expr));
                        }
                    }
                }
            }
            "alertdialogtrigger" | "alert-dialog-trigger" => {
                // text becomes slot content, as-child for button styling
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                if let Some(value) = props.get("asChild") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }
            "alertdialogcontent" | "alert-dialog-content" => {
                // class for styling
                self.push_style_class(&mut attrs, props);
            }
            "alertdialogheader"
            | "alert-dialog-header"
            | "alertdialogfooter"
            | "alert-dialog-footer" => {
                // Container components - class handled by extract_classes
            }
            "alertdialogtitle" | "alert-dialog-title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "alertdialogdescription" | "alert-dialog-description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "alertdialogaction" | "alert-dialog-action" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "alertdialogcancel" | "alert-dialog-cancel" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Dialog Sub-components ===
            "dialogtrigger" | "dialog_trigger" | "dialog-trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "dialogcontent" | "dialog_content" | "dialog-content" => {
                self.push_style_class(&mut attrs, props);
            }
            "dialogheader" | "dialog_header" | "dialog-header" | "dialogfooter"
            | "dialog_footer" | "dialog-footer" => {
                self.push_style_class(&mut attrs, props);
            }
            "dialogtitle" | "dialog_title" | "dialog-title" => {
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "dialogdescription" | "dialog_description" | "dialog-description" => {
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "dialogclose" | "dialog_close" | "dialog-close" => {
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Card Sub-components ===
            "cardheader" | "cardcontent" | "cardfooter" => {
                // These are container components - class is handled by extract_classes
            }
            "cardtitle" | "carddescription" => {
                // Text content for title/description
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Tabs Sub-components ===
            "tabslist" | "tabs_list" => {
                // TabsList is a container - class handled by extract_classes
            }
            "tabstrigger" | "tabs_trigger" => {
                // value is required for TabsTrigger
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "tabscontent" | "tabs_content" => {
                // value is required for TabsContent
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            // === Avatar ===
            "avatar" => {
                // Avatar in shadcn-vue is a wrapper that needs AvatarImage and AvatarFallback children
                let mut generated_children = String::new();

                // Generate AvatarImage if src provided
                if let Some(value) = props.get("src") {
                    let src = self.extract_string_value(value).unwrap_or("");
                    let alt = props
                        .get("alt")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or("");
                    generated_children.push_str(&format!(
                        r#"<AvatarImage src="{}" alt="{}" />{}"#,
                        src, alt, "\n"
                    ));
                    // Register AvatarImage component for imports
                    self.shadcn_components_used
                        .insert("AvatarImage".to_string());
                }

                // Generate AvatarFallback if fallback provided
                if let Some(value) = props.get("fallback") {
                    let fallback_text = self.prop_to_text_content(value).unwrap_or_default();
                    generated_children.push_str(&format!(
                        r#"<AvatarFallback>{}</AvatarFallback>"#,
                        fallback_text
                    ));
                    // Register AvatarFallback component for imports
                    self.shadcn_components_used
                        .insert("AvatarFallback".to_string());
                }

                // Set generated children if any were created
                if !generated_children.is_empty() {
                    slot_children = Some(generated_children);
                }
            }

            // === AvatarImage (when used as standalone element) ===
            "avatarimage" | "avatar_image" => {
                if let Some(value) = props.get("src") {
                    let src = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("src=\"{}\"", src));
                }
                if let Some(value) = props.get("alt") {
                    let alt = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("alt=\"{}\"", alt));
                }
            }

            // === AvatarFallback (when used as standalone element) ===
            "avatarfallback" | "avatar_fallback" => {
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === AspectRatio ===
            "aspectratio" | "aspect_ratio" => {
                // ratio prop (e.g., 16/9 = 1.777)
                if let Some(value) = props.get("ratio") {
                    if let Some(ratio) = self.extract_float_value(value) {
                        attrs.push(format!(":ratio=\"{}\"", ratio));
                    } else if let Some(ratio) = self.extract_int_value(value) {
                        attrs.push(format!(":ratio=\"{}\"", ratio));
                    }
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // ========================================
            // Phase 4: Overlay & Feedback
            // ========================================

            // === Dialog/Modal ===
            "dialog" | "modal" => {
                // v-model:open for dialog state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    } else if let AuraPropValue::Expr(expr) = value {
                        if let Ok(js_expr) = self.expr_to_vue_bound_value(expr) {
                            attrs.push(format!(":open=\"{}\"", js_expr));
                        }
                    }
                }
                // title for DialogTitle
                if let Some(value) = props.get("title") {
                    let title = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("data-title=\"{}\"", title));
                }
                // description for DialogDescription
                if let Some(value) = props.get("description") {
                    let desc = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("data-description=\"{}\"", desc));
                }
            }

            // === Tooltip ===
            "tooltip" => {
                // content for TooltipContent
                if let Some(value) = props.get("content") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // side (top, right, bottom, left)
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("top");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // delay duration
                if let Some(value) = props.get("delay") {
                    if let Some(delay) = self.extract_int_value(value) {
                        attrs.push(format!(":delay-duration=\"{}\"", delay));
                    }
                }
            }

            // === Spinner/Skeleton ===
            "spinner" => {
                // Skeleton uses class for sizing
                self.push_style_class(&mut attrs, props);
                // width
                if let Some(value) = props.get("width") {
                    if let Some(width) = self.extract_int_value(value) {
                        attrs.push(format!("style=\"width: {}px\"", width));
                    }
                }
                // height
                if let Some(value) = props.get("height") {
                    if let Some(height) = self.extract_int_value(value) {
                        attrs.push(format!("style=\"height: {}px\"", height));
                    }
                }
            }

            // ========================================
            // Phase 5: Data Components
            // ========================================

            // === Table ===
            "table" => {
                // Table wrapper class
                self.push_style_class(&mut attrs, props);
            }

            "thead" | "tbody" | "tr" => {
                // Table structure elements - minimal props
                self.push_style_class(&mut attrs, props);
            }

            "th" | "td" => {
                // Table cells
                self.push_style_class(&mut attrs, props);
                // colspan
                if let Some(value) = props.get("colspan") {
                    if let Some(span) = self.extract_int_value(value) {
                        attrs.push(format!(":colspan=\"{}\"", span));
                    }
                }
                // rowspan
                if let Some(value) = props.get("rowspan") {
                    if let Some(span) = self.extract_int_value(value) {
                        attrs.push(format!(":rowspan=\"{}\"", span));
                    }
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === shadcn-vue Table components ===
            "table_caption" => {
                // class
                self.push_style_class(&mut attrs, props);
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "table_header" | "table_body" | "table_row" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "table_head" | "table_cell" => {
                // class
                self.push_style_class(&mut attrs, props);
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Tree ===
            "tree" => {
                // Tree container
                self.push_style_class(&mut attrs, props);
            }

            "tree_item" | "tree-item" => {
                // Tree item with expanded state
                if let Some(value) = props.get("expanded") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // Text content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // ========================================
            // Phase 6: Form Components
            // ========================================

            // === RadioGroup ===
            "radiogroup" | "radio-group" => {
                // v-model for selected value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // name for form grouping
                if let Some(value) = props.get("name") {
                    let name = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("name=\"{}\"", name));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Radio ===
            "radio" => {
                // value for this radio option
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // id for label association
                if let Some(value) = props.get("id") {
                    let id = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("id=\"{}\"", id));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // label text
                if let Some(value) = props.get("label") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // ========================================
            // Phase 7: Feedback & Navigation
            // ========================================

            // === Alert ===
            "alert" => {
                // variant: default, destructive
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // title for AlertTitle
                if let Some(value) = props.get("title") {
                    let title = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("data-title=\"{}\"", title));
                }
                // description/text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                if let Some(value) = props.get("description") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Toast/Toaster (Sonner) ===
            "toast" | "toaster" => {
                // position: top-left, top-center, top-right, bottom-left, bottom-center, bottom-right
                if let Some(value) = props.get("position") {
                    let position = self.extract_string_value(value).unwrap_or("bottom-right");
                    attrs.push(format!("position=\"{}\"", position));
                }
                // richColors for colored toasts
                if let Some(value) = props.get("rich_colors") {
                    if self.extract_bool_value(value) {
                        attrs.push(":rich-colors=\"true\"".to_string());
                    }
                }
                // expand for expanded toasts
                if let Some(value) = props.get("expand") {
                    if self.extract_bool_value(value) {
                        attrs.push(":expand=\"true\"".to_string());
                    }
                }
            }

            // === Dropdown Menu ===
            "dropdown" | "dropdown_menu" => {
                // v-model:open for menu state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "dropdown_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "dropdown_content" => {
                // side: top, right, bottom, left
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("bottom");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // align: start, center, end
                if let Some(value) = props.get("align") {
                    let align = self.extract_string_value(value).unwrap_or("center");
                    attrs.push(format!("align=\"{}\"", align));
                }
            }

            "dropdown_item" => {
                // value for selection
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "dropdown_separator" => {
                // No special attributes
            }

            "dropdown_label" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // ========================================
            // Phase 8: Popover, Sheet, Breadcrumb
            // ========================================

            // === Popover ===
            "popover" => {
                // v-model:open for popover state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "popover_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "popover_content" => {
                // side: top, right, bottom, left
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("bottom");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // align: start, center, end
                if let Some(value) = props.get("align") {
                    let align = self.extract_string_value(value).unwrap_or("center");
                    attrs.push(format!("align=\"{}\"", align));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Sheet (Side Drawer) ===
            "sheet" => {
                // v-model:open for sheet state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "sheet_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "sheet_content" => {
                // side: top, right, bottom, left
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("right");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sheet_header" | "sheet_footer" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sheet_title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Breadcrumb ===
            "breadcrumb" | "breadcrumb_list" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "breadcrumb_item" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "breadcrumb_link" => {
                // href for link
                if let Some(value) = props.get("href") {
                    let href = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("href=\"{}\"", href));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick for navigation
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "breadcrumb_separator" => {
                // No special attributes
            }

            "breadcrumb_page" => {
                // text becomes slot content (current page, not clickable)
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // ========================================
            // Phase 9: High Priority Components
            // ========================================

            // === Accordion ===
            "accordion" => {
                // type: single, multiple
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("single");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // collapsible (for single type)
                if let Some(value) = props.get("collapsible") {
                    if self.extract_bool_value(value) {
                        attrs.push(":collapsible=\"true\"".to_string());
                    }
                }
                // default-value for initially expanded item
                if let Some(value) = props.get("default") {
                    let default = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("default-value=\"{}\"", default));
                }
            }

            "accordion_item" => {
                // value (required)
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            "accordion_trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "accordion_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "alert_dialog_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "alert_dialog_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "alert_dialog_header" | "alert_dialog_footer" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "alert_dialog_title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "alert_dialog_description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "alert_dialog_action" | "alert_dialog_cancel" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick event
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            // === Command (Command Palette) ===
            // PLAN-692 W-2: 组件家族=assets/shadcn-ui/command（shadcn-vue 官方
            // Listbox 实现），schema web:component 翻转后走 shadcn 路径——此前
            // backends web:"none" 使这些标签全部降级为无交互 stub div。
            // search 态由 CommandInput 经注入上下文自持（ListboxFilter v-model
            // → filterState.search），root 的 query 绑定与 placeholder 发射
            // 均已移除（reka/官方实现无此 API，误发射只会落成噪音属性）。
            "command" => {
                self.push_style_class(&mut attrs, props);
            }

            "command_input" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self
                        .extract_string_value(value)
                        .unwrap_or("Type a command...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "command_list" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "command_empty" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "command_group" => {
                // heading
                if let Some(value) = props.get("heading") {
                    let heading = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("heading=\"{}\"", heading));
                }
            }

            "command_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick for selection
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "command_shortcut" => {
                // text becomes slot content (e.g., "⌘K")
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "command_separator" => {
                // No special attributes
            }

            // === Form ===
            "form" => {
                // id for form identification
                if let Some(value) = props.get("id") {
                    let id = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("id=\"{}\"", id));
                }
                // class
                self.push_style_class(&mut attrs, props);
                // onsubmit event
                if events.contains_key("onsubmit") {
                    // Handled by event handlers below
                }
            }

            "form_field" => {
                // name (required)
                if let Some(value) = props.get("name") {
                    let name = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("name=\"{}\"", name));
                }
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
            }

            "form_item" | "form_control" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "form_label" => {
                // for (htmlFor)
                if let Some(value) = props.get("for") {
                    let for_val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("for=\"{}\"", for_val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "form_description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "form_message" => {
                // Error message (auto-bound to form validation)
            }

            // === Navigation Menu ===
            "nav_menu" => {
                // orientation: horizontal, vertical
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            "nav_menu_list" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "nav_menu_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            "nav_menu_link" => {
                // href
                if let Some(value) = props.get("href") {
                    let href = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("href=\"{}\"", href));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "nav_menu_trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "nav_menu_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "nav_menu_indicator" => {
                // No special attributes
            }

            // === Sidebar ===
            "sidebar" => {
                // side: left, right
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("left");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // variant: sidebar, floating, inset
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("sidebar");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // collapsible: offcanvas, icon, none
                if let Some(value) = props.get("collapsible") {
                    let collapsible = self.extract_string_value(value).unwrap_or("offcanvas");
                    attrs.push(format!("collapsible=\"{}\"", collapsible));
                }
            }

            "sidebar_header" | "sidebar_footer" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sidebar_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sidebar_group" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sidebar_group_label" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "sidebar_group_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sidebar_menu" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sidebar_menu_item" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sidebar_menu_button" => {
                // tooltip
                if let Some(value) = props.get("tooltip") {
                    let tooltip = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("tooltip=\"{}\"", tooltip));
                }
                // isActive
                if let Some(value) = props.get("active") {
                    if self.extract_bool_value(value) {
                        attrs.push(":is-active=\"true\"".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "sidebar_trigger" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "sidebar_provider" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Stepper ===
            "stepper" => {
                // v-model for current step
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // orientation: horizontal, vertical
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
            }

            "stepper_item" => {
                // step (required)
                if let Some(value) = props.get("step") {
                    if let Some(step) = self.extract_int_value(value) {
                        attrs.push(format!(":step=\"{}\"", step));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            "stepper_trigger" => {
                // onclick for step navigation
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "stepper_indicator" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "stepper_title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "stepper_description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "stepper_separator" => {
                // No special attributes
            }

            // ========================================
            // Phase 10: Medium Priority Components
            // ========================================

            // === Calendar ===
            "calendar" => {
                // v-model for selected date
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // default-placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Pick a date");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // weekday-format
                if let Some(value) = props.get("weekday") {
                    let weekday = self.extract_string_value(value).unwrap_or("short");
                    attrs.push(format!("weekday-format=\"{}\"", weekday));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Carousel ===
            "carousel" => {
                // Build opts object for Embla options (align, loop, etc.)
                let mut opts_parts: Vec<String> = Vec::new();

                // align option
                if let Some(value) = props.get("align") {
                    let align = self.extract_string_value(value).unwrap_or("center");
                    opts_parts.push(format!("align: '{}'", align));
                }

                // loop option
                if let Some(value) = props.get("loop") {
                    if self.extract_bool_value(value) {
                        opts_parts.push("loop: true".to_string());
                    }
                }

                // orientation option (vertical/horizontal)
                // This is a direct prop on Carousel for shadcn-vue styling
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }

                // Output opts if any options were specified
                if !opts_parts.is_empty() {
                    attrs.push(format!(":opts=\"{{ {} }}\"", opts_parts.join(", ")));
                }

                // class
                self.push_style_class(&mut attrs, props);
            }

            "carousel_content" | "carousel_item" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "carousel_prev" | "carousel_previous" | "carousel_next" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Combobox ===
            "combobox" => {
                // v-model for selected value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // open state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "combobox_input" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Select...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "combobox_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick for selection
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "combobox_trigger" => {
                // as-child
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "combobox_empty" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Context Menu ===
            "context_menu" => {
                // v-model:open for menu state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "context_menu_trigger" => {
                // class
                self.push_style_class(&mut attrs, props);
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "context_menu_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "context_menu_item" => {
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // inset
                if let Some(value) = props.get("inset") {
                    if self.extract_bool_value(value) {
                        attrs.push("inset".to_string());
                    }
                }
                // variant (default, destructive)
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    if variant != "default" {
                        attrs.push(format!("variant=\"{}\"", variant));
                    }
                }
                // class
                self.push_style_class(&mut attrs, props);
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // shortcut - rendered as ContextMenuShortcut inside the item
                // (handled in node_to_html when children are processed)
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "context_menu_separator" => {
                // No special attributes
            }

            "context_menu_label" => {
                // inset
                if let Some(value) = props.get("inset") {
                    if self.extract_bool_value(value) {
                        attrs.push("inset".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "context_menu_shortcut" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "context_menu_checkbox_item" => {
                // model-value for checked state
                if let Some(value) = props.get("checked") {
                    if self.extract_bool_value(value) {
                        attrs.push(":model-value=\"true\"".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "context_menu_radio_group" => {
                // model-value for selected value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("model-value=\"{}\"", val));
                }
            }

            "context_menu_radio_item" => {
                // value (required)
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "context_menu_sub" => {
                // open
                if let Some(value) = props.get("open") {
                    if let Some(ref_name) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", ref_name));
                    }
                }
            }

            "context_menu_sub_trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // inset
                if let Some(value) = props.get("inset") {
                    if self.extract_bool_value(value) {
                        attrs.push("inset".to_string());
                    }
                }
            }

            "context_menu_sub_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Drawer (Vaul) ===
            "drawer" => {
                // v-model:open for drawer state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // direction: left, right, top, bottom
                if let Some(value) = props.get("direction") {
                    let direction = self.extract_string_value(value).unwrap_or("bottom");
                    attrs.push(format!("direction=\"{}\"", direction));
                }
            }

            "drawer_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "drawer_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "drawer_header" | "drawer_footer" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "drawer_title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "drawer_description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "drawer_close" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            // === Hover Card ===
            "hover_card" => {
                // v-model:open for hover card state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // open-delay
                if let Some(value) = props.get("open_delay") {
                    if let Some(delay) = self.extract_int_value(value) {
                        attrs.push(format!(":open-delay=\"{}\"", delay));
                    }
                }
                // close-delay
                if let Some(value) = props.get("close_delay") {
                    if let Some(delay) = self.extract_int_value(value) {
                        attrs.push(format!(":close-delay=\"{}\"", delay));
                    }
                }
            }

            "hover_card_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "hover_card_content" => {
                // side
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("bottom");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Number Field ===
            "number_field" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // min/max/step
                if let Some(value) = props.get("min") {
                    if let Some(min) = self.extract_int_value(value) {
                        attrs.push(format!(":min=\"{}\"", min));
                    }
                }
                if let Some(value) = props.get("max") {
                    if let Some(max) = self.extract_int_value(value) {
                        attrs.push(format!(":max=\"{}\"", max));
                    }
                }
                if let Some(value) = props.get("step") {
                    if let Some(step) = self.extract_int_value(value) {
                        attrs.push(format!(":step=\"{}\"", step));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            "number_field_input" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "number_field_increment" | "number_field_decrement" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Pagination ===
            "pagination" => {
                // v-model:page for current page
                if let Some(value) = props.get("page") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:page=\"{}\"", model));
                    }
                }
                // total
                if let Some(value) = props.get("total") {
                    if let Some(total) = self.extract_int_value(value) {
                        attrs.push(format!(":total=\"{}\"", total));
                    }
                }
                // per-page / items-per-page
                if let Some(value) = props.get("per_page") {
                    if let Some(per_page) = self.extract_int_value(value) {
                        attrs.push(format!(":items-per-page=\"{}\"", per_page));
                    }
                }
                if let Some(value) = props.get("itemsPerPage") {
                    if let Some(items) = self.extract_int_value(value) {
                        attrs.push(format!(":items-per-page=\"{}\"", items));
                    }
                }
                // sibling-count
                if let Some(value) = props.get("sibling_count") {
                    if let Some(count) = self.extract_int_value(value) {
                        attrs.push(format!(":sibling-count=\"{}\"", count));
                    }
                }
            }

            "pagination_list" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "pagination_item" => {
                // value (page number)
                if let Some(value) = props.get("value") {
                    if let Some(val) = self.extract_int_value(value) {
                        attrs.push(format!(":value=\"{}\"", val));
                    }
                }
                // onclick for page change
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "pagination_ellipsis" => {
                // No special attributes
            }

            "pagination_prev" | "pagination_next" | "pagination_first" | "pagination_last" => {
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            // === Pin Input (OTP) ===
            "pin_input" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // length (number of pins)
                if let Some(value) = props.get("length") {
                    if let Some(length) = self.extract_int_value(value) {
                        attrs.push(format!(":length=\"{}\"", length));
                    }
                }
                // type: text, password
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("text");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // otp (native autocomplete)
                if let Some(value) = props.get("otp") {
                    if self.extract_bool_value(value) {
                        attrs.push("otp".to_string());
                    }
                }
            }

            "pin_input_slot" => {
                // index
                if let Some(value) = props.get("index") {
                    if let Some(index) = self.extract_int_value(value) {
                        attrs.push(format!(":index=\"{}\"", index));
                    }
                }
            }

            "pin_input_separator" => {
                // No special attributes
            }

            // === Tags Input ===
            "tags_input" => {
                // v-model for tags array
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Add tag...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // max-tags
                if let Some(value) = props.get("max") {
                    if let Some(max) = self.extract_int_value(value) {
                        attrs.push(format!(":max-tags=\"{}\"", max));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            "tags_input_field" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "tags_input_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            "tags_input_delete" => {
                // onclick to remove tag
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            // === Toggle Group ===
            // PLAN-528 W7: normalize 把 `-` 折成 `_`,页面书写 togglegroup /
            // toggle-group / toggle_group 三种拼写全部归并到这里。
            "toggle_group" | "togglegroup" => {
                // v-model for selected value(s)
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // type: single, multiple
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("single");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // PLAN-528 W7: variant (default/outline) + size (default/sm/lg)
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                if let Some(value) = props.get("size") {
                    let size = self.extract_string_value(value).unwrap_or("default");
                    if size != "default" {
                        attrs.push(format!("size=\"{}\"", size));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            "toggle_group_item" | "togglegroupitem" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // aria-label (PLAN-528 W7: 兼容 label 拼写)
                let aria_label = props.get("aria-label").or_else(|| props.get("label"));
                if let Some(value) = aria_label {
                    let label = self.extract_string_value(value).unwrap_or("");
                    if !label.is_empty() {
                        attrs.push(format!("aria-label=\"{}\"", label));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // ========================================
            // Phase 11: Low Priority Components
            // ========================================

            // === Button Group ===
            "button_group" => {
                // orientation: horizontal, vertical
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
                // size
                if let Some(value) = props.get("size") {
                    let size = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("size=\"{}\"", size));
                }
                // variant
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
            }

            // === Chart ===
            "chart" => {
                // config for chart styling
                if let Some(value) = props.get("config") {
                    let config = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!(":config=\"{}\"", config));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Collapsible ===
            "collapsible" => {
                // v-model:open for expanded state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // default-open
                if let Some(value) = props.get("default_open") {
                    if self.extract_bool_value(value) {
                        attrs.push(":default-open=\"true\"".to_string());
                    }
                }
            }

            "collapsible_trigger" => {
                // class
                self.push_style_class(&mut attrs, props);
                // as-child
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "collapsible_content" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Input Group ===
            "input_group" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Input OTP ===
            "input_otp" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // length
                if let Some(value) = props.get("length") {
                    if let Some(length) = self.extract_int_value(value) {
                        attrs.push(format!(":length=\"{}\"", length));
                    }
                }
                // pattern
                if let Some(value) = props.get("pattern") {
                    let pattern = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("pattern=\"{}\"", pattern));
                }
            }

            // === Kbd (Keyboard) ===
            "kbd" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Menubar ===
            // PLAN-760: remote-frame 宿主占位——宿主前端（canvas_embed_web
            // 同族）querySelector `[data-remote-frame]` 接管;url 经
            // data-remote-frame-url 携带（字面量直发;动态绑定走
            // :data-remote-frame-url）。本仓 web 臂不做 JS 渲染器。
            //（本表 match 的是 normalized_tag：连字符已转下划线。）
            "remote_frame" => {
                attrs.push("data-remote-frame=\"1\"".to_string());
                if let Some(value) = props.get("url") {
                    match value {
                        AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => {
                            attrs.push(format!("data-remote-frame-url=\"{s}\""));
                        }
                        AuraPropValue::Expr(expr) => {
                            let bound =
                                self.bound_value_or_warn(expr, "component prop `url`", "''");
                            attrs.push(format!(":data-remote-frame-url=\"{bound}\""));
                        }
                        AuraPropValue::StyleBinding(_) => {}
                    }
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            "menubar" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "menubar_menu" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            "menubar_trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "menubar_content" => {
                // align
                if let Some(value) = props.get("align") {
                    let align = self.extract_string_value(value).unwrap_or("start");
                    attrs.push(format!("align=\"{}\"", align));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            "menubar_item" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "menubar_separator" => {
                // No special attributes
            }

            "menubar_label" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Native Select ===
            "native_select" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // name
                if let Some(value) = props.get("name") {
                    let name = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("name=\"{}\"", name));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Range Calendar ===
            "range_calendar" => {
                // v-model for date range
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self
                        .extract_string_value(value)
                        .unwrap_or("Pick a date range");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Resizable ===
            "resizable" | "resizable_panel_group" => {
                // direction: horizontal, vertical
                if let Some(value) = props.get("direction") {
                    let direction = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("direction=\"{}\"", direction));
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            "resizable_panel" => {
                // default-size
                if let Some(value) = props.get("default_size") {
                    if let Some(size) = self.extract_int_value(value) {
                        attrs.push(format!(":default-size=\"{}\"", size));
                    }
                }
                // min-size
                if let Some(value) = props.get("min_size") {
                    if let Some(size) = self.extract_int_value(value) {
                        attrs.push(format!(":min-size=\"{}\"", size));
                    }
                }
                // max-size
                if let Some(value) = props.get("max_size") {
                    if let Some(size) = self.extract_int_value(value) {
                        attrs.push(format!(":max-size=\"{}\"", size));
                    }
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            "resizable_handle" => {
                // with-handle (show drag handle)
                if let Some(value) = props.get("with_handle") {
                    if self.extract_bool_value(value) {
                        attrs.push(":with-handle=\"true\"".to_string());
                    }
                }
                // class
                self.push_style_class(&mut attrs, props);
            }

            // === Auto Complete ===
            "autocomplete" => {
                // v-model for selected value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // open state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "autocomplete_input" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Search...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "autocomplete_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "autocomplete_list" => {
                // class
                self.push_style_class(&mut attrs, props);
            }

            "autocomplete_empty" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Image ===
            "image" | "img" => {
                for key in &["src", "alt"] {
                    if let Some(value) = props.get(*key) {
                        if let Some(state_ref) = self.extract_state_ref(value) {
                            attrs.push(format!(":{}=\"{}\"", key, state_ref));
                        } else if let Ok(val) = self.prop_to_attr_value(value) {
                            match value {
                                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(_)) => {
                                    attrs.push(format!("{}={}", key, val));
                                }
                                AuraPropValue::Expr(_) => {
                                    attrs.push(format!(":{}={}", key, val));
                                }
                                _ => {
                                    attrs.push(format!("{}={}", key, val));
                                }
                            }
                        }
                    }
                }
                // Plan 606: fit prop -> Tailwind object-{fit} class
                if let Some(fit_val) = props.get("fit") {
                    let fit_str = match fit_val {
                        AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => s.as_str(),
                        _ => "",
                    };
                    let object_class = match fit_str {
                        "cover" => "object-cover",
                        "contain" => "object-contain",
                        "fill" => "object-fill",
                        "none" => "object-none",
                        "scale-down" => "object-scale-down",
                        _ => "",
                    };
                    if !object_class.is_empty() {
                        let mut props_with_fit = props.clone();
                        let existing = self.get_style_class(props).and_then(|v| match v {
                            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => {
                                Some(s.to_string())
                            }
                            _ => None,
                        });
                        let combined = match existing {
                            Some(cls) => format!("{cls} {object_class}"),
                            None => object_class.to_string(),
                        };
                        props_with_fit.insert(
                            "class".to_string(),
                            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(combined.into())),
                        );
                        props_with_fit.remove("style");
                        self.push_style_class(&mut attrs, &props_with_fit);
                    } else {
                        self.push_style_class(&mut attrs, props);
                    }
                } else {
                    self.push_style_class(&mut attrs, props);
                }
            }

            // Plan 484: shadcn chart 族发射臂退役——裸名 bar-chart 等由
            // official 包 Auto 组件承接(known_sub_widgets 折叠),原生 tag
            // 不再走 shadcn 发射。
            _ => {
                // Default handling for other components - extract class/style.
                self.push_style_class(&mut attrs, props);
                // Plan 505 B3（债 P497-2）：无专用臂的注册组件（wm 族
                // window_thumbnail/virtual_window 等）props 透传——原 v1
                // 只透 class，wid/win/fallback_icon 静默丢弃；Vue 侧
                // defineProps 契约（如 VirtualWindow.vue 的 win）由此
                // 喂到。与子组件通用臂同款：排序迭代 + 绑定表达式；
                // class/style 已由上面消费，跳过不双发。
                {
                    let mut sorted_bind: Vec<(&String, &AuraPropValue)> = props.iter().collect();
                    sorted_bind.sort_by(|a, b| a.0.cmp(b.0));
                    for (key, value) in sorted_bind {
                        if matches!(key.as_str(), "class" | "style" | "style_obj") {
                            continue;
                        }
                        // PLAN-682 F2：显式 `key:` 已在函数顶部单点转发为
                        // `:key` 绑定；本透明转发环（无专用臂的注册部件
                        // 走此处，与 layout 臂的 push_passthrough_attrs
                        // 双漏斗同契约）不再重发 `:key`——同元素双 key
                        // 会被 Vue 编译器 X_DUPLICATE_ATTRIBUTE 拒收。
                        if key == "key" && attrs.iter().any(|a| a.starts_with(":key=")) {
                            continue;
                        }
                        let value_str = match value {
                            AuraPropValue::Expr(expr) => self.bound_value_or_warn(
                                expr,
                                &format!("component prop `{}`", key),
                                "null",
                            ),
                            AuraPropValue::StyleBinding(_) => continue,
                        };
                        attrs.push(format!(":{}=\"{}\"", key, value_str));
                    }
                }
            }
        }

        // Plan 012 Batch D (P0#11 leftover): forward `class:`/`style:` on the
        // shadcn arms that never looked at it (DialogTitle, Sidebar*, overlay
        // primitives, … — ~130 arms where it was dropped silently). Vue attr
        // fallthrough applies class/style to components automatically, so the
        // full plain-path handling (push_native_classes: static class, dynamic
        // :class, conditional ternary, __style__ CSS-string marker) is safe
        // for every emitted element. Arms that already consumed the class/
        // style props pushed a class/:class/:style attr and are skipped here,
        // so nothing is ever emitted twice. Nothing is dropped anymore, so
        // R011 never fires from this path.
        if props.contains_key("class") || props.contains_key("style") {
            let arm_emitted_class = attrs[attrs_before_match..].iter().any(|a| {
                a.starts_with("class=") || a.starts_with(":class=") || a.starts_with(":style=")
            });
            if !arm_emitted_class {
                self.push_native_classes(&mut attrs, tag, props);
            }
        }

        // Add event handlers
        let mut sorted_ev: Vec<(&String, &AuraEvent)> = events.iter().collect();
        sorted_ev.sort_by(|a, b| a.0.cmp(b.0));
        for (event, aura_event) in sorted_ev {
            // Parser-minted modal trigger toggles are needed by backends that
            // don't own overlay state. Vue's Reka/Vaul trigger components
            // already update the controlled `v-model:open` root; forwarding
            // this second toggle can immediately close the overlay again.
            // Keep explicit user handlers intact.
            if aura_event
                .handler
                .trim_start_matches('.')
                .starts_with("__dlg_toggle_")
            {
                continue;
            }
            // .window/.document modifiers → global listener, no template attr
            if self.try_register_global_listener(event, aura_event) {
                continue;
            }
            let vue_event = self.shadcn_event_to_vue(tag, event);
            // Plan 408 P10 / §7.3 缺陷 4: callback prop short-circuit (shadcn path).
            if let Some(attr) = self.try_callback_prop_attr(aura_event, &vue_event) {
                attrs.push(attr);
                continue;
            }
            let mut handler_fn =
                self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
            // Plan 421: code_editor oncursor/oncontextmenu — forward the
            // scaffold shell's emit payload ({line, column} / {x, y}) when
            // the on-block handler declares params and the binding passes none.
            let payload_forwarded =
                match self.code_editor_event_payload_call(tag, event, aura_event) {
                    Some(call) => {
                        handler_fn = call;
                        true
                    }
                    None => false,
                };
            // Track used handler (without params for matching)
            let handler_name = self.handler_to_function_call(&aura_event.handler);
            // If inside a for-loop, pass the loop variable's .id as argument (e.g., SelectNote(note.id))
            // Only append if handler doesn't already have params from aura_event
            if let Some(ref loop_var) = self.current_loop_var {
                if aura_event.params.is_empty() && !payload_forwarded {
                    handler_fn = format!("{}({})", handler_fn, loop_var);
                    self.loop_param_handlers
                        .insert(handler_name.clone(), loop_var.clone());
                }
            }
            self.used_handlers.insert(handler_name);
            let handler_fn = Self::ime_guard_enter_handler(&vue_event, handler_fn);
            attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
        }

        (attrs, slot_content, slot_children)
    }

    /// Convert an `if` / `else if` / `else` chain used as a `style:`/`class:`
    /// value into a Vue `:class` ternary binding string.
    ///
    /// `if a { "x" } else if b { "y" } else { "z" }` →
    /// `a ? 'x' : (b ? 'y' : 'z')`. Plan 043 M5 #tag-coloring: the previous
    /// implementation only read `branches.first()` + the final `else`, so
    /// else-if chains silently dropped every branch after the first.
    pub(super) fn if_expr_to_style_ternary(&self, if_stmt: &auto_lang_core::ast::If) -> String {
        let t = self.build_style_ternary(&if_stmt.branches, &if_stmt.else_);
        // Plan 054 M6: wrap the whole ternary in parens so it's safe inside
        // string concatenation (`'fw:' + (c ? a : b)`). Without the outer
        // parens, JS `+` binds tighter than `?:`, so `'fw:' + c ? a : b`
        // parses as `('fw:' + c) ? a : b` — always truthy, always the
        // then-branch (observed: CodeView恒 bold/italic regardless of span).
        format!("({})", t)
    }

    pub(super) fn build_style_ternary(
        &self,
        branches: &[auto_lang_core::ast::Branch],
        else_: &Option<auto_lang_core::ast::Body>,
    ) -> String {
        if let Some((first, rest)) = branches.split_first() {
            // Plan 012 P0#13 follow-up: an unsupported condition form used
            // to silently become `false`; keep that fallback, warn R013.
            let cond = self.bound_value_or_warn(
                &first.cond,
                "if-expression condition in style/class ternary",
                "false",
            );
            let then = self.style_branch_value(&first.body.stmts);
            let else_part = self.build_style_ternary(rest, else_);
            // Assemble then-branch: Leaf → 'str', Nested → (ternary).
            let then_str = match &then {
                StyleBranch::Leaf(s) => format!("'{}'", s),
                StyleBranch::Nested(t) => format!("({})", t),
            };
            if else_part.is_empty() {
                format!("{} ? {} : ''", cond, then_str)
            } else if else_part.starts_with('\'') {
                // Leaf else string — emit directly, no parens.
                format!("{} ? {} : {}", cond, then_str, else_part)
            } else {
                // Nested ternary from an else-if chain — parenthesize.
                format!("{} ? {} : ({})", cond, then_str, else_part)
            }
        } else {
            // Final else branch — Leaf gets quotes, Nested returns raw ternary
            // (the caller wraps it in parens via the "not starts with '"'" branch).
            else_
                .as_ref()
                .map(|b| match self.style_branch_value(&b.stmts) {
                    StyleBranch::Leaf(s) => format!("'{}'", s),
                    StyleBranch::Nested(t) => t,
                })
                .unwrap_or_default()
        }
    }

    /// Classify an `if` branch body into a leaf string or a nested if-ternary.
    ///
    /// - `{ "cls" }` or `{ return "cls" }` → `Leaf("cls")`
    /// - `{ if cond { x } else { y } }`    → `Nested("<ternary>")` (recursive)
    /// - anything else                     → `Leaf("")` (fallback, preserves
    ///   the old `style_branch_str` behavior of returning empty)
    pub(super) fn style_branch_value(&self, stmts: &[auto_lang_core::ast::Stmt]) -> StyleBranch {
        for st in stmts {
            match st {
                // String leaf: `{ "cls" }` or `{ return "cls" }`
                auto_lang_core::ast::Stmt::Return(e) => {
                    if let auto_lang_core::ast::Expr::Str(s) = e.as_ref() {
                        return StyleBranch::Leaf(s.to_string());
                    }
                }
                auto_lang_core::ast::Stmt::Expr(e) => {
                    if let auto_lang_core::ast::Expr::Str(s) = e {
                        return StyleBranch::Leaf(s.to_string());
                    }
                    if let auto_lang_core::ast::Expr::If(if_stmt) = e {
                        return StyleBranch::Nested(self.if_expr_to_style_ternary(if_stmt));
                    }
                }
                // Nested if as a statement: `{ if cond { x } else { y } }`
                auto_lang_core::ast::Stmt::If(if_stmt) => {
                    return StyleBranch::Nested(self.if_expr_to_style_ternary(if_stmt));
                }
                _ => {}
            }
        }
        StyleBranch::Leaf(String::new())
    }

    /// Extract the `style`/`class` prop into template attributes for
    /// shadcn-mapped elements (generate_shadcn_attrs).
    ///
    /// Handles both a static string (`class="..."`) and a conditional
    /// `style: if cond { "a" } else { "b" }` (`:class="cond ? 'a' : 'b'"`).
    /// Plan 043 M5 #5: the plain-element path (extract_classes) already did
    /// both; the shadcn path only handled the static string, silently
    /// dropping conditional styles on registry widgets like `text` → span.
    /// Plan 448 D: structural class-safety of a manual string concat used as
    /// a style value. `Str` literals, string-valued `if` chains, and `+`
    /// concatenations thereof are class-safe (every operand resolves to
    /// Tailwind class fragments). Anything else — state refs, method calls,
    /// numeric exprs — keeps the Plan 043 H5 inline-CSS reading (`:style`),
    /// because `"color: rgb(" + r + …` is indistinguishable from a class
    /// concat once refs are allowed. Use the array form
    /// `style: ["base", .dynClass]` for dynamic parts.
    pub(super) fn is_class_safe_concat(expr: &auto_lang_core::ast::Expr) -> bool {
        match expr {
            auto_lang_core::ast::Expr::Str(_) => true,
            auto_lang_core::ast::Expr::If(if_stmt) => {
                let branch_safe = |b: &auto_lang_core::ast::Branch| {
                    b.body.stmts.iter().all(|st| match st {
                        auto_lang_core::ast::Stmt::Expr(e) => Self::is_class_safe_concat(e),
                        _ => false,
                    })
                };
                if_stmt.branches.iter().all(branch_safe)
                    && if_stmt
                        .else_
                        .as_ref()
                        .map(|b| {
                            b.stmts.iter().all(|st| match st {
                                auto_lang_core::ast::Stmt::Expr(e) => Self::is_class_safe_concat(e),
                                _ => false,
                            })
                        })
                        .unwrap_or(true)
            }
            auto_lang_core::ast::Expr::Bina(l, auto_val::Op::Add, r) => {
                Self::is_class_safe_concat(l) && Self::is_class_safe_concat(r)
            }
            _ => false,
        }
    }

    /// Translate one element of a `style: [...]` array into its JS fragment
    /// (quoted literal / style ternary / bound value). Shared by the
    /// plain-element and shadcn paths.
    pub(super) fn style_array_part_js(
        &self,
        e: &auto_lang_core::ast::Expr,
    ) -> crate::ui_gen::GenResult<String> {
        match e {
            auto_lang_core::ast::Expr::Str(s) => {
                Ok(format!("'{}'", Self::escape_js_string(s.as_str())))
            }
            auto_lang_core::ast::Expr::If(if_stmt) => Ok(self.if_expr_to_style_ternary(if_stmt)),
            other => self.expr_to_vue_bound_value(other),
        }
    }

    pub(super) fn push_style_class(&self, attrs: &mut Vec<String>, props: &HashMap<String, AuraPropValue>) {
        if let Some(value) = self.get_style_class(props) {
            let is_explicit_class = props.contains_key("class") && !props.contains_key("style");
            match value {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::If(if_stmt)) => {
                    attrs.push(format!(
                        ":class=\"{}\"",
                        self.if_expr_to_style_ternary(if_stmt)
                    ));
                }
                // Plan 448 D: array-of-class-parts → :class join (same
                // semantics as the plain-element path).
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Array(elems)) => {
                    let parts: Vec<String> = elems
                        .iter()
                        .map(|e| self.style_array_part_js(e))
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap_or_default();
                    if !parts.is_empty() {
                        attrs.push(format!(
                            ":class=\"[{}].filter(Boolean).join(' ')\"",
                            parts.join(", ")
                        ));
                    } else {
                        self.warn(
                            "R013",
                            crate::ui_gen::validators::Severity::Warning,
                            "style: array with no renderable parts; binding not emitted"
                                .to_string(),
                        );
                    }
                }
                AuraPropValue::Expr(other_expr) => {
                    // Plan 043 H5: a dynamic style expression that is neither a
                    // string literal nor a conditional — e.g. a string concat
                    // like `"color: rgb(" + span.r + "," + ...`. It produces a
                    // CSS declaration string at runtime; emit as :style (not a
                    // Tailwind class). Falls back to no attr if the expr can't
                    // be rendered.
                    if let auto_lang_core::ast::Expr::Str(_) = other_expr {
                        // Plain string literal → static class (Tailwind classes).
                        if let Some(s) = self.extract_string_value(value) {
                            if !s.is_empty() {
                                // Plan 496 M5：插值 class 拆分（extract_classes
                                // 同型）——静态段入 class，`${.field}` 段转
                                // `:class` 拼接表达式。
                                if let Some((statics, expr)) = Self::interpolated_class_parts(s) {
                                    if !statics.is_empty() {
                                        attrs.push(format!("class=\"{}\"", statics.join(" ")));
                                    }
                                    attrs.push(format!(":class=\"{}\"", expr));
                                } else {
                                    attrs.push(format!("class=\"{}\"", s));
                                }
                            }
                        }
                    } else if Self::is_class_safe_concat(other_expr) || is_explicit_class {
                        // Plan 448 D: literal/conditional-only concat is a
                        // class string by structure — :class, not :style.
                        // When explicitly passed via `class:`, always emit :class.
                        match self.expr_to_vue_bound_value(other_expr) {
                            Ok(expr_str) => attrs.push(format!(":class=\"{}\"", expr_str)),
                            Err(e) => self.warn(
                                "R011",
                                crate::ui_gen::validators::Severity::Warning,
                                format!(
                                    "class expression could not be rendered and was dropped: {}",
                                    e
                                ),
                            ),
                        }
                    } else {
                        match self.expr_to_vue_bound_value(other_expr) {
                            Ok(expr_str) => attrs.push(format!(":style=\"{}\"", expr_str)),
                            // Plan 012: never drop a class/style expr silently.
                            // Plan 012 P0#13 follow-up: this branch is now
                            // actually reachable (the catch-all errs instead
                            // of returning "null") — include the error detail.
                            Err(e) => self.warn(
                                "R011",
                                crate::ui_gen::validators::Severity::Warning,
                                format!("class/style expression could not be rendered and was dropped: {}", e),
                            ),
                        }
                    }
                }
                _ => {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    } else {
                        // Plan 012: a class/style prop value we don't know how
                        // to emit (e.g. a StyleBinding on the shadcn path) used
                        // to vanish without a trace.
                        self.warn(
                            "R011",
                            crate::ui_gen::validators::Severity::Warning,
                            "class/style prop value not emitted on this element (unsupported value shape)",
                        );
                    }
                }
            }
        }
    }

    /// Plan 012: full plain-path class handling for schema-registered elements
    /// that stay on the shadcn attrs path (`generate_shadcn_attrs`) — either
    /// because they emit a NATIVE tag there (`label`) or map to a shadcn
    /// component that should still receive user classes via attr fallthrough
    /// (`select`). Mirrors the plain-element path: static `class`, dynamic
    /// `:class` exprs (Batch A gap 20), conditional ternaries, and dynamic
    /// `:style` CSS-string expressions (independent channels, Plan 408 P12).
    pub(super) fn push_native_classes(
        &self,
        attrs: &mut Vec<String>,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
    ) {
        let (static_classes, dynamic_class, dynamic_style) = self.extract_classes(tag, props);
        if !static_classes.is_empty() {
            attrs.push(format!("class=\"{}\"", static_classes));
        }
        if let Some(dc) = dynamic_class {
            attrs.push(format!(":class=\"{}\"", dc));
        }
        if let Some(ds) = dynamic_style {
            attrs.push(format!(":style=\"{}\"", ds));
        }
    }

    /// IME guard for `@keyup.enter` handlers (LAUNCHER-001 R-02 / 452 spike).
    /// Composition confirm Enter must not fire onenter actions.
    pub(super) fn ime_guard_enter_handler(vue_event: &str, handler_fn: String) -> String {
        if vue_event.starts_with("@keyup.enter") {
            // handler_fn is either `Name` (0-arity) or `Name(...)` (call form).
            let invoke = if handler_fn.contains('(') {
                handler_fn
            } else {
                format!("{handler_fn}()")
            };
            format!(
                "($event) => {{ if (!($event.isComposing || $event.keyCode === 229)) {{ {invoke} }} }}"
            )
        } else {
            handler_fn
        }
    }

    /// Convert AutoUI event to Vue event for shadcn-vue components
    pub(super) fn shadcn_event_to_vue(&self, _tag: &str, event: &str) -> String {
        let (base, modifiers) = Self::split_event_key(event);
        let mut vue = match base {
            "onclick" | "onClick" | "on_click" => "@click".to_string(),
            "oninput" | "onInput" => "@update:modelValue".to_string(),
            "onchange" | "onChange" => "@update:modelValue".to_string(),
            "onenter" | "onEnter" => "@keyup.enter".to_string(),
            _ => format!("@{}", Self::base_event_to_dom(base)),
        };
        for m in modifiers {
            if m == "window" || m == "document" {
                continue; // global target — handled elsewhere
            }
            vue.push('.');
            vue.push_str(Self::vue_modifier(m));
        }
        vue
    }

    /// Extract string value from AuraPropValue
    pub(super) fn extract_string_value<'a>(&self, value: &'a AuraPropValue) -> Option<&'a str> {
        match value {
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract boolean value from AuraPropValue
    pub(super) fn extract_bool_value(&self, value: &AuraPropValue) -> bool {
        match value {
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => *b,
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => s == "true",
            _ => false,
        }
    }

    /// Extract integer value from AuraPropValue
    pub(super) fn extract_int_value(&self, value: &AuraPropValue) -> Option<i64> {
        match value {
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Int(n)) => Some(*n as i64),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => s.parse().ok(),
            _ => None,
        }
    }

    /// Extract float value from AuraPropValue
    pub(super) fn extract_float_value(&self, value: &AuraPropValue) -> Option<f64> {
        match value {
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Float(n, _)) => Some(*n),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Int(n)) => Some(*n as f64),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => s.parse().ok(),
            _ => None,
        }
    }

    /// Extract state reference from AuraPropValue
    ///
    /// Accepts every shape the parser can produce for a `.state` ref:
    /// - `Ident("name")` — bare identifier (hand-built ASTs, some legacy paths)
    /// - `Ident(".name")` — legacy dot-prefixed identifier (dot stripped)
    /// - `Dot(Ident("self") | Ident("."), "name")` — what `dot_item` actually
    ///   produces for `.name` in real widget source (parser.rs dot_item).
    /// PLAN-037 T5: the writable-state-slot target of a model-channel value.
    /// `.field` (Dot(self, field) or leading-dot Ident) -> Some("field");
    /// `store.field` where `store` is a store dep -> Some("store.field").
    /// Anything else (expression, prop, literal) -> None — the caller must
    /// raise the "model channel requires a writable state slot" error.
    pub(super) fn model_channel_slot(&self, expr: &auto_lang_core::ast::Expr) -> Option<String> {
        use auto_lang_core::ast::Expr;
        match expr {
            Expr::Dot(obj, field) => {
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    if obj_name == "self" {
                        return Some(field.to_string());
                    }
                    if self.store_deps.iter().any(|d| d == obj_name) {
                        return Some(format!("{}.{}", obj_name, field));
                    }
                }
                None
            }
            Expr::Ident(name) if name.starts_with('.') => Some(name[1..].to_string()),
            _ => None,
        }
    }

    /// PLAN-037 T5: try to emit the model-channel binding for a call-site
    /// prop. Returns Some(attr) when `key` is a model var of `tag` (emitting
    /// `v-model:key="slot"`), or a hard error when the channel is fed a
    /// non-writable value. Returns None when `key` is not a channel (normal
    /// prop path continues).
    pub(super) fn try_model_channel_attr(
        &mut self,
        tag: &str,
        key: &str,
        slot: Option<String>,
    ) -> Result<Option<String>, crate::ui_gen::GenError> {
        match self.sub_widget_models.get(tag) {
            Some(channels) if channels.iter().any(|v| v == key) => {}
            _ => return Ok(None),
        }
        match slot {
            Some(slot) => {
                // Plan 443: a v-model emission IS the binding event — record
                // it so the child-side scan knows this channel is bound (and
                // may downgrade the child's model var to defineModel).
                self.emitted_model_bindings
                    .push((tag.to_string(), key.to_string()));
                Ok(Some(format!("v-model:{}=\"{}\"", key, slot)))
            }
            None => {
                // Plan 443 续(435 挂账②):通道收到非可写值(字面量/表达式/
                // props 引用)时降级为单向 prop 绑定而非硬错 —— 记录绑定使
                // 子件侧仍把该通道编译为 defineModel(值经 prop 流入;子件
                // 变更 emit 无人监听,天然单向),与 443 '未绑定保持 ref' 的
                // 降级哲学互补。gallery 官方组件大量使用字面量喂通道形态。
                self.emitted_model_bindings
                    .push((tag.to_string(), key.to_string()));
                self.warn(
                    "R016",
                    crate::ui_gen::validators::Severity::Info,
                    format!(
                        "model channel `{}.{}` fed a non-writable value; degraded to one-way `:{}` prop binding",
                        tag, key, key
                    ),
                );
                Ok(None) // 走普通 prop 发射路径
            }
        }
    }

    pub(super) fn extract_state_ref(&self, value: &AuraPropValue) -> Option<String> {
        match value {
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name)) => {
                let stripped = name.as_str().strip_prefix('.').unwrap_or(name.as_str());
                if stripped.is_empty() {
                    None
                } else {
                    Some(stripped.to_string())
                }
            }
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Dot(obj, field)) => match obj.as_ref() {
                auto_lang_core::ast::Expr::Ident(obj_name)
                    if obj_name.as_str() == "self" || obj_name.as_str() == "." =>
                {
                    Some(field.to_string())
                }
                auto_lang_core::ast::Expr::Ident(obj_name) => {
                    let stripped = obj_name
                        .as_str()
                        .strip_prefix('.')
                        .unwrap_or(obj_name.as_str());
                    if stripped.is_empty() {
                        Some(field.to_string())
                    } else {
                        Some(format!("{}.{}", stripped, field))
                    }
                }
                auto_lang_core::ast::Expr::Dot(..) => {
                    let s = self
                        .expr_to_vue_bound_value(match value {
                            AuraPropValue::Expr(e) => e,
                            _ => unreachable!(),
                        })
                        .ok()?;
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Plan 028 F5: normalize an Index target to its bare state name —
    /// `.answers[k]` parses with the target as either Ident(".answers") or
    /// Dot(Ident("."), "answers"); both yield "answers".
    pub(super) fn index_target_name(target: &auto_lang_core::ast::Expr) -> Option<String> {
        match target {
            auto_lang_core::ast::Expr::Ident(name) => {
                let stripped = name.as_str().strip_prefix('.').unwrap_or(name.as_str());
                if stripped.is_empty() {
                    None
                } else {
                    Some(stripped.to_string())
                }
            }
            auto_lang_core::ast::Expr::Dot(obj, field) => match obj.as_ref() {
                auto_lang_core::ast::Expr::Ident(obj_name)
                    if obj_name.as_str() == "self" || obj_name.as_str() == "." =>
                {
                    Some(field.to_string())
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Plan 028 F5: `value: .answers[q.id]` — Index-position state ref for a
    /// dynamic-key v-model. Returns `answers[q.id]` or None.
    pub(super) fn extract_state_index_ref(&self, value: &AuraPropValue) -> Option<String> {
        if let AuraPropValue::Expr(auto_lang_core::ast::Expr::Index(target, index)) = value {
            let t = Self::index_target_name(target)?;
            let idx = self.expr_to_vue_bound_value(index).ok()?;
            return Some(format!("{}[{}]", t, idx));
        }
        None
    }

    /// Get style or class prop value (style takes priority over class)
    /// This supports the transition from 'class' to 'style' prop naming
    pub(super) fn get_style_class<'a>(
        &self,
        props: &'a HashMap<String, AuraPropValue>,
    ) -> Option<&'a AuraPropValue> {
        props.get("style").or_else(|| props.get("class"))
    }

    /// PLAN-026 缺陷①: 布局基元（col/row/grid/center/container）的动态 class 绑定。
    ///
    /// `generate_shadcn_attrs` 的布局分支用 `extract_string_value` 取 class，但它对
    /// 非静态字符串（`Expr::If` / computed / ident）返回 None，导致
    /// `col { class: .rowClass }` 的动态绑定被静默丢弃。这里补齐：
    /// `Expr::If` → 三元；其他 `Expr` → `:class` 表达式。静态 `Expr::Str` 返回 None
    /// （由调用方按字符串追加到 classes）。
    pub(super) fn layout_dynamic_class_attr(&self, value: &AuraPropValue) -> Option<String> {
        use auto_lang_core::ast::Expr;
        match value {
            AuraPropValue::Expr(Expr::If(if_stmt)) => Some(format!(
                ":class=\"{}\"",
                self.if_expr_to_style_ternary(if_stmt)
            )),
            AuraPropValue::Expr(other) => match self.expr_to_vue_bound_value(other) {
                Ok(s) if s != "null" => Some(format!(":class=\"{}\"", s)),
                _ => None,
            },
            _ => None,
        }
    }

    /// Plan 053 后续: 布局基元的动态 **style** 绑定(CSS 声明串,如
    /// `"display:grid; grid-template-columns: repeat(" + n + ", 1fr)"`)。
    ///
    /// 镜像 `layout_dynamic_class_attr`,但产出 `:style`(CSS 声明必须走 style 通道,
    /// 走 :class 会被当类名而失效)。之前 row/col 用 get_style_class(style 优先)
    /// 把 style prop 的动态值喂给 layout_dynamic_class_attr → 误产 :class →
    /// 表格 grid 列对齐失效(整个 Table 渲染虽不报错,但 grid 不生效)。
    pub(super) fn layout_dynamic_style_attr(&self, value: &AuraPropValue) -> Option<String> {
        use auto_lang_core::ast::Expr;
        match value {
            // Plan 458: `style: if` branches carry Tailwind CLASS lists (the
            // 006/015 theme-switching convention), so they must bind :class —
            // the previous If→:style arm emitted class names as CSS
            // declarations, which Vue drops as invalid, making conditional
            // styles silent no-ops on layout primitives. Only free-form
            // dynamic expressions are CSS declaration strings (Plan 053,
            // e.g. "display:grid; " + n) and keep binding :style.
            AuraPropValue::Expr(Expr::If(if_stmt)) => Some(format!(
                ":class=\"{}\"",
                self.if_expr_to_style_ternary(if_stmt)
            )),
            // Plan 448 D: array-of-class-parts joins into ONE :class — the
            // array form is explicitly class semantics, refs included
            // (same reasoning as Plan 458 above).
            AuraPropValue::Expr(Expr::Array(elems)) => {
                let parts: Vec<String> = elems
                    .iter()
                    .map(|e| self.style_array_part_js(e))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_or_default();
                if parts.is_empty() {
                    None
                } else {
                    Some(format!(
                        ":class=\"[{}].filter(Boolean).join(' ')\"",
                        parts.join(", ")
                    ))
                }
            }
            AuraPropValue::Expr(other) => {
                // Plan 448 D: literal/conditional-only concat is a class
                // string by structure → :class (same reasoning as the
                // plain-element path).
                if Self::is_class_safe_concat(other) {
                    return self
                        .expr_to_vue_bound_value(other)
                        .ok()
                        .filter(|s| s != "null")
                        .map(|s| format!(":class=\"{}\"", s));
                }
                match self.expr_to_vue_bound_value(other) {
                    Ok(s) if s != "null" => Some(format!(":style=\"{}\"", s)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Build a `:kebab-attr="value"` binding for a boolean-style prop that
    /// also accepts an expression reference (e.g. `can_edit: .editable`).
    ///
    /// Looks up the prop under both `snake_key` (AURA convention, e.g.
    /// `can_edit`) and `camel_key` (Vue convention, e.g. `canEdit`). When the
    /// prop is present as a bool/bool-string literal, emits the literal value.
    /// When present as another expression, emits the bound JS value. When
    /// absent, emits `default_val` so the consumer always gets a defined prop.
    /// (Plan 354 Phase C: used by AutoDownEditor's can_edit / show_actions.)
    pub(super) fn bool_prop_binding(
        &self,
        props: &HashMap<String, AuraPropValue>,
        snake_key: &str,
        camel_key: &str,
        default_val: bool,
    ) -> String {
        // Derive the kebab-case Vue attribute from the camelCase key
        // (canEdit → can-edit, showActions → show-actions).
        let kebab_attr: String = camel_key.chars().fold(String::new(), |acc, c| {
            if c.is_ascii_uppercase() {
                if acc.is_empty() {
                    c.to_ascii_lowercase().to_string()
                } else {
                    format!("{}-{}", acc, c.to_ascii_lowercase())
                }
            } else {
                format!("{}{}", acc, c)
            }
        });
        match props.get(snake_key).or_else(|| props.get(camel_key)) {
            Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b))) => {
                format!(":{}=\"{}\"", kebab_attr, b)
            }
            Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))) => {
                format!(":{}=\"{}\"", kebab_attr, s == "true")
            }
            Some(AuraPropValue::Expr(expr)) => {
                // Dynamic expression (e.g. can_edit: .editable → editable).
                match self.expr_to_vue_bound_value(expr) {
                    Ok(js_expr) => format!(":{}=\"{}\"", kebab_attr, js_expr),
                    // Plan 012 P0#13 follow-up: the Err branch is now
                    // reachable — keep the default-value fallback, warn R013.
                    Err(e) => {
                        self.warn(
                            "R013",
                            crate::ui_gen::validators::Severity::Warning,
                            format!(
                                "bool prop `{}`: {}; fell back to `{}`",
                                snake_key, e, default_val
                            ),
                        );
                        format!(":{}=\"{}\"", kebab_attr, default_val)
                    }
                }
            }
            _ => format!(":{}=\"{}\"", kebab_attr, default_val),
        }
    }

    /// Split an event key into (base, modifiers).
    /// "onwheel.document.capture" → ("onwheel", ["document", "capture"])
    ///
    /// Quoted custom event names (`on"autodown:slash-open"`) may contain
    /// characters that are illegal in identifiers (':'/'-'); the base then
    /// runs to the closing quote and only what follows it is modifiers.
    pub(super) fn split_event_key(event: &str) -> (&str, Vec<&str>) {
        if let Some(rest) = event.strip_prefix("on\"") {
            if let Some(close) = rest.find('"') {
                let end = 3 + close + 1; // just past the closing quote
                let base = &event[..end];
                let mods: Vec<&str> = event[end..]
                    .strip_prefix('.')
                    .map(|t| t.split('.').filter(|m| !m.is_empty()).collect())
                    .unwrap_or_default();
                return (base, mods);
            }
        }
        let mut parts = event.split('.');
        let base = parts.next().unwrap_or(event);
        (base, parts.collect())
    }

    /// Map the base event key (without modifiers) to the DOM/Vue event name.
    /// Covers the full common set: keyboard, mouse, wheel, focus, pointer,
    /// touch, drag, scroll. Unknown `onxxx` keys fall back to stripping `on`.
    /// Quoted custom names (`on"autodown:slash-open"`) map to the raw name.
    pub(super) fn base_event_to_dom(base: &str) -> String {
        if let Some(inner) = base.strip_prefix("on\"").and_then(|b| b.strip_suffix('"')) {
            return inner.to_string();
        }
        match base.to_ascii_lowercase().as_str() {
            "onclick" | "on_click" => "click",
            "ondblclick" | "on_double_click" => "dblclick",
            "oninput" => "input",
            "onchange" => "change",
            "onblur" => "blur",
            "onfocus" => "focus",
            "onfocusin" => "focusin",
            "onfocusout" => "focusout",
            // Keyboard
            "onkeydown" => "keydown",
            "onkeyup" => "keyup",
            "onkeypress" => "keypress",
            // Mouse
            "onmousedown" => "mousedown",
            "onmouseup" => "mouseup",
            "onmousemove" => "mousemove",
            "onmouseenter" => "mouseenter",
            "onmouseleave" => "mouseleave",
            "onmouseover" => "mouseover",
            "onmouseout" => "mouseout",
            // Wheel / context menu / scroll
            "onwheel" => "wheel",
            "oncontextmenu" | "on_context_menu" => "contextmenu",
            "onscroll" => "scroll",
            // Plan 421: code_editor shell emits `cursor` with a
            // `{ line, column }` payload (iced-side on_cursor equivalent).
            "oncursor" | "on_cursor" => "cursor",
            // Pointer Events
            "onpointerdown" => "pointerdown",
            "onpointerup" => "pointerup",
            "onpointermove" => "pointermove",
            "onpointerenter" => "pointerenter",
            "onpointerleave" => "pointerleave",
            "onpointerover" => "pointerover",
            "onpointerout" => "pointerout",
            "onpointercancel" => "pointercancel",
            // Touch
            "ontouchstart" => "touchstart",
            "ontouchmove" => "touchmove",
            "ontouchend" => "touchend",
            "ontouchcancel" => "touchcancel",
            // Drag & drop
            "ondragstart" => "dragstart",
            "ondrag" => "drag",
            "ondragend" => "dragend",
            "ondragover" => "dragover",
            "ondragenter" => "dragenter",
            "ondragleave" => "dragleave",
            "ondrop" => "drop",
            // Fallback: strip a single leading "on" (onupdate → update, etc.)
            other => other.strip_prefix("on").unwrap_or(other),
        }
        .to_string()
    }

    /// Normalize an event modifier for Vue templates.
    /// Auto names are mapped to Vue's modifier vocabulary where they differ.
    pub(super) fn vue_modifier(m: &str) -> &str {
        match m {
            "escape" => "esc",
            "del" => "delete",
            other => other,
        }
    }

    /// Vue event binding for a KNOWN SUB-WIDGET callback prop.
    ///
    /// Sub-widgets emit their msg variant names from defineEmits (`Run`,
    /// `OpenPath`, `Stop`), so the parent-side callback prop `on_run` must
    /// bind to `@Run` — NOT the DOM fallback in `base_event_to_dom`, which
    /// strips a leading "on" and would emit `@_run` (never fired by the
    /// child). Uses the same `on_pick` ↔ `Pick` convention as
    /// `prop_to_ts_type`. Non-`on_` events (e.g. DOM events forwarded to the
    /// child) keep the normal DOM mapping.
    pub(super) fn sub_widget_event_to_vue(&self, event: &str) -> String {
        if let Some(base) = event.strip_prefix("on_") {
            return format!("@{}", Self::snake_to_pascal(base));
        }
        self.auto_event_to_vue(event)
    }

    /// Plan 444 (ash-shell-057 ①b): Vue event binding for a sub-widget
    /// CALLBACK prop (`on_x: .Y`), resolving against the child's emit
    /// roster (`sub_widget_msgs`) when the driver provided one:
    ///
    /// - `PascalOf(x)` is a declared child emit → `@PascalOf(x)` (the
    ///   corpus convention: k2 `on_select` ↔ `Select`, 017 `on_send` —
    ///   the child rewrites `on_send()` to `emit('Send')`). Also the
    ///   fallback when the roster has no entry for the tag.
    /// - else the bound handler names a child emit (`on_delete:
    ///   .DeleteBlock`, child re-emits its own `DeleteBlock` via the
    ///   empty-handler bridge — the ash-gui pass-through idiom) →
    ///   `@<handler>` so the listener matches what the child fires.
    /// - else keep the prop-derived name and warn (neither side fires it).
    pub(super) fn sub_widget_callback_event_to_vue(
        &self,
        tag: &str,
        event: &str,
        aura_event: &AuraEvent,
    ) -> String {
        let fallback = self.sub_widget_event_to_vue(event);
        let base = match event.strip_prefix("on_") {
            Some(b) => b,
            None => return fallback,
        };
        let prop_pascal = Self::snake_to_pascal(base);
        let emits = match self.sub_widget_msgs.get(tag) {
            Some(e) if !e.is_empty() => e,
            _ => return fallback,
        };
        if emits.iter().any(|e| e == &prop_pascal) {
            return fallback;
        }
        let handler_name = Self::base_pattern(&aura_event.handler)
            .trim_start_matches('.')
            .trim_matches('"');
        if !handler_name.is_empty()
            && handler_name
                .chars()
                .next()
                .map(|c| c.is_ascii_uppercase())
                .unwrap_or(false)
            && emits.iter().any(|e| e == handler_name)
        {
            self.warn(
                "R044",
                crate::ui_gen::validators::Severity::Warning,
                format!(
                    "callback prop `{}` on `{}` does not match any child emit; binding to the child's `{}` emit instead (pass-through idiom). Rename the prop to `on_{}` (or add a `{}` msg variant) to follow the convention.",
                    event, tag, handler_name,
                    handler_name.to_lowercase(),
                    prop_pascal
                ),
            );
            return format!("@{}", handler_name);
        }
        self.warn(
            "R045",
            crate::ui_gen::validators::Severity::Warning,
            format!(
                "callback prop `{}` on `{}` matches no child emit (roster: {:?}); the binding will never fire",
                event, tag, emits
            ),
        );
        fallback
    }

    /// Convert AutoUI event name (with optional `.modifier` chain) to a Vue
    /// template event binding, e.g. `onkeydown.up.prevent` → `@keydown.up.prevent`.
    ///
    /// `.window` / `.document` modifiers are NOT handled here — they mark
    /// global listeners and are intercepted by `try_register_global_listener`
    /// before this function is called.
    pub(super) fn auto_event_to_vue(&self, event: &str) -> String {
        let (base, modifiers) = Self::split_event_key(event);
        // Existing shorthands keep their historical expansion.
        let mut vue = match base {
            "onenter" | "onEnter" => "@keyup.enter".to_string(),
            "onsubmit" | "onSubmit" => "@submit.prevent".to_string(),
            _ => format!("@{}", Self::base_event_to_dom(base)),
        };
        for m in modifiers {
            if m == "window" || m == "document" {
                continue; // global target — handled elsewhere
            }
            vue.push('.');
            vue.push_str(Self::vue_modifier(m));
        }
        vue
    }

    /// Render the addEventListener/removeEventListener options argument
    /// (", { capture: true, passive: false }" or ""). For removal only the
    /// capture flag matters (per DOM spec), so passive is omitted there.
    pub(super) fn listener_options(gl: &GlobalListener, for_removal: bool) -> String {
        let mut opts: Vec<String> = Vec::new();
        if gl.capture {
            opts.push("capture: true".to_string());
        }
        if !for_removal {
            if let Some(passive) = gl.passive {
                opts.push(format!("passive: {}", passive));
            }
        }
        if opts.is_empty() {
            String::new()
        } else {
            format!(", {{ {} }}", opts.join(", "))
        }
    }

    /// Make an arbitrary DOM event name safe for embedding in a JS identifier.
    /// Custom event names may contain ':'/'-' (e.g. "autodown:slash-open"),
    /// which would otherwise produce an invalid wrapper function name.
    pub(super) fn sanitize_ident(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    /// Intercept `.window` / `.document` event modifiers and register a global
    /// listener instead of emitting a template attribute. Returns true when
    /// the event was claimed (caller must skip normal attribute emission).
    ///
    /// Declaration: `onmousemove.window: .DragMove($event)` or
    /// `onwheel.document.capture.prevent: .LockWheel($event)`.
    pub(super) fn try_register_global_listener(&mut self, event: &str, aura_event: &AuraEvent) -> bool {
        let (base, modifiers) = Self::split_event_key(event);
        let target = modifiers
            .iter()
            .find(|m| **m == "window" || **m == "document");
        let target = match target {
            Some(t) => t.to_string(),
            None => return false,
        };

        let dom_event = Self::base_event_to_dom(base);
        let prevent = modifiers.iter().any(|m| *m == "prevent");
        let stop = modifiers.iter().any(|m| *m == "stop");
        let capture = modifiers.iter().any(|m| *m == "capture");
        let passive = if modifiers.iter().any(|m| *m == "passive") {
            Some(true)
        } else if prevent {
            // Chrome treats document/window wheel & touch listeners as passive
            // by default — preventDefault requires an explicit passive: false.
            Some(false)
        } else {
            None
        };

        let handler_fn = self.handler_to_function_call(&aura_event.handler);
        self.used_handlers.insert(handler_fn.clone());

        // Build the handler call. In addEventListener context the DOM event is
        // the listener's argument `e` (Vue's `$event` does not exist here).
        let call_args: Vec<String> = if aura_event.params.is_empty() {
            vec!["e".to_string()]
        } else {
            aura_event
                .params
                .iter()
                .map(|p| {
                    // Same `this.` stripping as template event args
                    // (vue_event_param); addEventListener context additionally
                    // maps `$event` to the listener's argument `e`.
                    Self::vue_event_param(p)
                        .replace("$event", "e")
                        .replace('"', "'")
                })
                .collect()
        };
        let call = format!("{}({})", handler_fn, call_args.join(", "));

        // A wrapper is needed when prevent/stop must run or when the handler
        // takes adapted arguments. Otherwise the bare function ref suffices
        // (the DOM passes the event object as first argument).
        let needs_wrapper = prevent || stop || !aura_event.params.is_empty();
        let (listener, wrapper) = if needs_wrapper {
            let wrapper_fn = format!(
                "__auto_gl_{}_{}",
                Self::sanitize_ident(&dom_event),
                handler_fn
            );
            let mut body = String::new();
            if stop {
                body.push_str("  e.stopPropagation()\n");
            }
            if prevent {
                body.push_str("  e.preventDefault()\n");
            }
            body.push_str(&format!("  {}\n", call));
            // `e: any` — the concrete type (KeyboardEvent/MouseEvent/WheelEvent)
            // depends on the DOM event; `any` keeps field access (e.clientY,
            // e.deltaY) type-checkable without narrowing machinery.
            let src = format!("function {}(e: any) {{\n{}}}\n\n", wrapper_fn, body);
            (wrapper_fn, Some(src))
        } else {
            (handler_fn, None)
        };

        let entry = GlobalListener {
            target,
            event: dom_event,
            listener,
            capture,
            passive,
            wrapper,
        };
        // Dedup identical registrations (same listener may be declared on
        // multiple elements).
        if !self.global_listeners.iter().any(|g| {
            g.target == entry.target && g.event == entry.event && g.listener == entry.listener
        }) {
            self.global_listeners.push(entry);
        }
        true
    }

    /// Plan 367 P0-1: Convert PascalCase to snake_case for callback prop matching.
    /// e.g. "TogglePin" → "toggle_pin", "DeleteActive" → "delete_active"
    pub(super) fn pascal_to_snake(s: &str) -> String {
        let mut result = String::new();
        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        result
    }

    /// Plan 367 P0-2: Indent every line of a body string by the given prefix.
    /// Used when wrapping transpiled handler bodies into function/onMounted.
    /// Ensures multi-line bodies have consistent indentation.
    pub(super) fn indent_body(body: &str, indent: &str) -> String {
        body.lines()
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indent, line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Plan 367 P0-4/P1-1: Map an Auto type to its TypeScript equivalent.
    /// Used for emit payload types and prop types.
    pub(super) fn auto_type_to_ts_type(ty: &auto_lang_core::ast::Type) -> String {
        use auto_lang_core::ast::Type;
        match ty {
            Type::StrSlice | Type::StrOwned | Type::StrFixed(_) | Type::CStrLit => {
                "string".to_string()
            }
            Type::Int
            | Type::I64
            | Type::Uint
            | Type::U64
            | Type::USize
            | Type::Float
            | Type::Double => "number".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::List(inner) => format!("{}[]", Self::auto_type_to_ts_type(inner)),
            Type::Slice(_) => "any[]".to_string(),
            Type::Option(inner) => format!("{} | null", Self::auto_type_to_ts_type(inner)),
            Type::User(decl) => {
                let name = decl.name.as_str();
                match name {
                    "msg" => "() => void".to_string(),
                    "str" => "string".to_string(),
                    "int" | "i64" | "uint" => "number".to_string(),
                    "bool" => "boolean".to_string(),
                    // Plan 012 P2 (gap 43): DSL `map` (map-literal type) is
                    // loosely typed — emit `any` (jade treats `x: map` as an
                    // untyped object), never a broken `import type { map }`.
                    "map" => "any".to_string(),
                    // PLAN-037 Phase 0: `Value` is Auto's dynamic builtin —
                    // emit `any`, never the identifier itself (TS2304) and
                    // never an api.ts import (TS2305).
                    "Value" => "any".to_string(),
                    // PLAN-037 Phase 4: bare `obj`/`list` are dynamic builtins
                    // in widget props (component fn extraction had erased them
                    // to any; the widget path maps types directly).
                    "obj" => "any".to_string(),
                    "list" => "any[]".to_string(),
                    // Custom types (e.g. Note) — use the type name directly.
                    // The interface should be imported from api.ts.
                    other => other.to_string(),
                }
            }
            _ => "any".to_string(),
        }
    }

    /// Plan 055 Phase E(a): TS type for a store state ref declaration. Scalars
    /// (str/int/bool) and arrays of scalars get real types; user-defined types
    /// fall back to `any` — importing them needs a separate pass (Phase E(e))
    /// to avoid "Cannot find name 'Block'" when the store file hasn't imported
    /// the api.ts interface. This is a safe first step: `var input str` →
    /// `ref<string>`, `var next_id int` → `ref<number>`, while `var blocks
    /// List<Block>` stays `ref<any[]>` until imports land.
    pub(super) fn store_ref_ts_type(ty: &auto_lang_core::ast::Type) -> String {
        use auto_lang_core::ast::Type;
        match ty {
            Type::List(inner) => {
                // Array of user/unknown type → any[] (can't import element yet).
                if matches!(
                    inner.as_ref(),
                    Type::User(_) | Type::Array(_) | Type::Slice(_) | Type::List(_)
                ) {
                    "any[]".to_string()
                } else {
                    format!("{}[]", Self::auto_type_to_ts_type(inner))
                }
            }
            Type::User(_) => "any".to_string(),
            _ => Self::auto_type_to_ts_type(ty),
        }
    }

    /// Plan 055 Phase E(b): recover handler parameter types by zipping the
    /// handler's parameter names (`store.handler_params[pattern]`) with the
    /// matching message variant's payload types (`store.messages[].variants`).
    /// Returns (name, ts_type) pairs; unmatched params fall back to `any`.
    /// Uses `store_ref_ts_type` so user-defined payload types stay `any`
    /// (avoiding "Cannot find name" until Phase E(e) imports land).
    pub(super) fn handler_param_types(store: &crate::aura::AuraStore, pattern: &str) -> Vec<(String, String)> {
        let after_dot = pattern.trim_start_matches('.');
        let action = after_dot.split('(').next().unwrap_or(after_dot);
        let names = match store.handler_params.get(pattern) {
            Some(n) => n,
            None => return Vec::new(),
        };
        for msg in &store.messages {
            for v in &msg.variants {
                if v.name == action {
                    return names
                        .iter()
                        .zip(v.payload.iter())
                        .map(|(n, ty)| (n.clone(), Self::store_ref_ts_type(ty)))
                        .collect();
                }
            }
        }
        names
            .iter()
            .map(|n| (n.clone(), "any".to_string()))
            .collect()
    }

    /// Strip a trailing param list from an on-block handler pattern key.
    /// Plan 374 embeds param names in handler keys (".Scrolled(e)") so the
    /// Rust backend can recover them; the Vue backend matches handlers by
    /// base name (".Scrolled"), so normalize before lookup.
    pub(super) fn base_pattern(pattern: &str) -> &str {
        match pattern.find('(') {
            Some(i) if pattern.ends_with(')') => &pattern[..i],
            _ => pattern,
        }
    }

    /// Plan 408 P10 / §7.3 缺陷 4（方案 B prop-as-callback）: if the handler
    /// is a bare identifier that matches a prop name of the current
    /// widget/component fn, it's a callback prop — return an assembled
    /// `@event="props.<name>(...)"` attribute so callers can short-circuit
    /// the normal handler path (which would mangle it into `on<name>` and
    /// synthesize an empty stub). Returns None when the handler is not a
    /// callback prop.
    pub(super) fn try_callback_prop_attr(&self, aura_event: &AuraEvent, vue_event: &str) -> Option<String> {
        let bare = Self::base_pattern(&aura_event.handler);
        if !bare.is_empty()
            && !bare.starts_with('.')
            && !bare.contains("::")
            && self.prop_names.iter().any(|p| p == bare)
        {
            let args = aura_event.params.join(", ");
            Some(format!("{}=\"props.{}({})\"", vue_event, bare, args))
        } else {
            None
        }
    }

    /// TS type for a widget prop.
    ///
    /// `on_*: msg` callback props resolve the matching msg variant via the
    /// `on_pick` ↔ `Pick` naming convention and type the callback from its
    /// payload: `Pick(str)` → `(arg0: string) => void`, `Stop` → `() => void`.
    /// Non-msg props fall through to the generic Auto→TS mapping.
    pub(super) fn prop_to_ts_type(prop: &AuraProp, widget: &AuraWidget) -> String {
        use auto_lang_core::ast::Type;
        if let Type::User(decl) = &prop.type_info {
            if decl.name.as_str() == "msg" {
                // on_pick → Pick; on_run_smart → RunSmart; on_open_path → OpenPath.
                let variant_name = prop
                    .name
                    .strip_prefix("on_")
                    .map(|s| Self::snake_to_pascal(s))
                    .unwrap_or_default();
                for msg in &widget.messages {
                    if let Some(variant) = msg.variants.iter().find(|v| v.name == variant_name) {
                        if variant.payload.is_empty() {
                            return "() => void".to_string();
                        }
                        let args: Vec<String> = variant
                            .payload
                            .iter()
                            .enumerate()
                            .map(|(i, ty)| format!("arg{}: {}", i, Self::auto_type_to_ts_type(ty)))
                            .collect();
                        return format!("({}) => void", args.join(", "));
                    }
                }
                // No matching variant — the prop is a plain signal: () => void.
                return "() => void".to_string();
            }
        }
        Self::auto_type_to_ts_type(&prop.type_info)
    }

    /// True when an `on_*: msg` prop is a callback CHANNEL (delivered via
    /// the parent's `@Pascal` emit binding, never bound as a data prop).
    ///
    /// Plan 043 M5 R4: callbacks with a matching msg variant
    /// (`on_run` ↔ `Run`) are delivered parent→child through the emit
    /// (`@Run="handler"` — Vue turns the listener into an `onRun`
    /// fallthrough), and the child's generated on-block never calls
    /// `props.on_run`. Declaring the prop as REQUIRED made the parent's
    /// usage miss `on_run` → TS2345, so it was skipped from defineProps.
    /// Plan 444 (ash-shell-057 ①a): ALL `on_*: msg` props are channels —
    /// variants without a name match (ash-gui's `on_delete` with its Msg
    /// `DeleteBlock`) kept a required snake_case prop that NO parent
    /// binding can ever satisfy (`@Delete` delivers `onDelete`, never
    /// `on_delete`). The child rewrites every `props.on_x()` / bare
    /// `on_x()` call to `emit('<Pascal>', ...)` (musk-022 / PLAN-037 T3),
    /// so the Pascal event is the one and only delivery channel.
    pub(super) fn prop_is_emitted_callback(prop: &AuraProp, widget: &AuraWidget) -> bool {
        use auto_lang_core::ast::Type;
        let _ = widget;
        if let Type::User(decl) = &prop.type_info {
            if decl.name.as_str() == "msg" {
                return prop.name.starts_with("on_");
            }
        }
        false
    }

    /// Plan musk-022 callback-relay fix: the snake_names of `on_xxx` callback
    /// props that ARE in defineProps (not emitted-callback-skipped). A handler
    /// body calling these as `props.on_xxx(...)` must be rewritten to
    /// `emit('<Pascal>', ...)` so the parent's `@<Pascal>` binding fires — the
    /// prop is never passed as `:on_xxx` by the parent (it binds `@Pascal`).
    pub(super) fn real_callback_prop_snakes(widget: &AuraWidget) -> Vec<String> {
        widget
            .props
            .iter()
            .filter_map(|p| {
                if !p.name.starts_with("on_") {
                    return None;
                }
                // Only props that are NOT emitted-callbacks (i.e. they're in defineProps).
                if Self::prop_is_emitted_callback(p, widget) {
                    return None;
                }
                p.name.strip_prefix("on_").map(|s| s.to_string())
            })
            .collect()
    }

    /// PLAN-037 T3: the snake_names of `on_xxx` callback props that ARE
    /// emitted-callbacks (dropped from defineProps because a matching msg
    /// variant exists). A handler body invoking these as `props.on_xxx(...)`
    /// must be rewritten to `emit('<Pascal>', ...)` — the prop does not exist
    /// at runtime, while the Pascal emit is declared via the msg variant.
    pub(super) fn emitted_callback_prop_snakes(widget: &AuraWidget) -> Vec<String> {
        widget
            .props
            .iter()
            .filter_map(|p| {
                if !p.name.starts_with("on_") {
                    return None;
                }
                if !Self::prop_is_emitted_callback(p, widget) {
                    return None;
                }
                p.name.strip_prefix("on_").map(|s| s.to_string())
            })
            .collect()
    }

    /// Collect api.ts interface names referenced by an Auto type, recursing
    /// into containers (`List<T>`, `[]T`, `[N]T`, `Option<T>`,
    /// `GenericInstance<T, …>`, …). Built-in/pseudo types (`msg`, `str`,
    /// `int`, `List`, `Array`, …) are excluded — they map to TS primitives
    /// or `() => void` and have no api.ts interface.
    pub(super) fn collect_custom_types(ty: &auto_lang_core::ast::Type, out: &mut Vec<String>) {
        use auto_lang_core::ast::Type;
        match ty {
            Type::User(decl) => {
                let name = decl.name.as_str().to_string();
                if !Self::is_builtin_type_name(&name) && !out.iter().any(|n| n == &name) {
                    out.push(name);
                }
            }
            Type::List(inner)
            | Type::Option(inner)
            | Type::Result(inner)
            | Type::Reference(inner)
            | Type::Linear(inner) => {
                Self::collect_custom_types(inner, out);
            }
            Type::Map(k, v) => {
                Self::collect_custom_types(k, out);
                Self::collect_custom_types(v, out);
            }
            Type::Slice(s) => Self::collect_custom_types(&s.elem, out),
            Type::Array(a) => Self::collect_custom_types(&a.elem, out),
            Type::RuntimeArray(r) => Self::collect_custom_types(&r.elem, out),
            Type::Ptr(p) => Self::collect_custom_types(&p.of.borrow(), out),
            Type::GenericInstance(inst) => {
                for arg in &inst.args {
                    Self::collect_custom_types(arg, out);
                }
            }
            Type::Union(u) => {
                for f in &u.fields {
                    Self::collect_custom_types(&f.ty, out);
                }
            }
            Type::Tuple(ts) => {
                for t in ts {
                    Self::collect_custom_types(t, out);
                }
            }
            Type::Handle { task_type } => Self::collect_custom_types(task_type, out),
            Type::Fn(params, ret) => {
                for p in params {
                    Self::collect_custom_types(p, out);
                }
                Self::collect_custom_types(ret, out);
            }
            _ => {}
        }
    }

    /// Built-in / pseudo type names that have no api.ts interface (TS
    /// primitives, the `msg` callback pseudo-type, stdlib containers).
    pub(super) fn is_builtin_type_name(name: &str) -> bool {
        matches!(
            name,
            "msg" | "str" | "int" | "i64" | "uint" | "u64" | "usize" | "byte" | "char"
            | "float" | "double" | "bool"
            // PLAN-037 Phase 0: `Value` is Auto's dynamic builtin (dynamic
            // any) — mapping it as a custom api.ts type produced
            // `import type { Value } from '@/lib/api'` (TS2305).
            // PLAN-037 Phase 4: bare `obj`/`list` likewise.
            | "Value" | "obj" | "list"
            // Plan 012 P2 (gap 43): lowercase `map` is the DSL map-literal
            // type — built-in, not an api.ts interface.
            | "map"
            | "List" | "Array" | "Map" | "Option" | "Result" | "String"
        )
    }

    /// Convert snake_case to PascalCase for msg-variant lookup
    /// (`on_open_path` → `OpenPath`).
    pub(super) fn snake_to_pascal(s: &str) -> String {
        let mut result = String::new();
        let mut cap = true;
        for ch in s.chars() {
            if ch == '_' {
                cap = true;
            } else if cap {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
                cap = false;
            } else {
                result.push(ch);
            }
        }
        result
    }

    /// Look up on-block handler params by base pattern (".Name"), tolerant of
    /// Plan 374 parameterized keys (".Name(e)").
    pub(super) fn get_handler_params<'a>(
        handler_params: &'a std::collections::HashMap<String, Vec<String>>,
        base_key: &str,
    ) -> Option<&'a Vec<String>> {
        handler_params
            .iter()
            .find(|(k, _)| Self::base_pattern(k) == base_key)
            .map(|(_, v)| v)
    }

    /// Plan 008 (auto-os-config dual-backend input contract) + DEBTS 引擎窗口
    /// #6 root fix: decide whether an input/textarea text handler should be
    /// CALLED with the typed TEXT on the vue backend. The vm backend hands the
    /// text to handlers that declare exactly one parameter (`.OnInput(t)`);
    /// handlers that declare zero parameters read the bound state instead
    /// (v-model keeps it in sync — HistorySearch's `.OnQuery -> {}` is the
    /// canonical case) and must stay bare calls: passing
    /// `($event.target as HTMLInputElement).value` to a 0-param function is
    /// TS2554. Explicit argument shapes (map literals, $event forwarding,
    /// multi-arg) always keep their bound arguments; handlers with no
    /// on-block entry (0-param synthesized stubs) only take the text via an
    /// explicit single-identifier binding.
    pub(super) fn input_text_handler_wants_text_arg(&self, aura_event: &AuraEvent) -> bool {
        // PLAN-055 T9(⑥): 实参为 `$event` 字面时同样按文本通道包
        // ($event.target as HTMLInputElement).value——此前 `$` 不属 bare 字符集，
        // `oninput: .OnSearchInput($event)` 生成 `@input="OnSearchInput($event)"`
        // 把原生 Event 传给 (val: string) 形参，与 v-model 竞争把 Event 写进
        // 绑定字段（musk 搜索框失效现场）。
        let param_is_text_channel = |p: &str| {
            let t = p.trim();
            t == "$event" || t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        };
        let bare_or_simple = aura_event.params.is_empty()
            || (aura_event.params.len() == 1
                && !aura_event.params[0].trim().is_empty()
                && param_is_text_channel(&aura_event.params[0]));
        if !bare_or_simple {
            return false;
        }
        let base = Self::base_pattern(&aura_event.handler);
        match Self::get_handler_params(&self.handler_params, base) {
            Some(declared) => declared.len() == 1,
            None => !aura_event.params.is_empty(),
        }
    }

    /// Convert handler pattern to function name
    ///
    /// The result must be a valid JS identifier: emit names may contain
    /// characters like ':' ("update:modelValue" v-model contracts), so the
    /// final name is sanitized (update:modelValue → update_modelValue). The
    /// verbatim emit name is recovered via the on-block pattern when needed.
    pub(super) fn pattern_to_handler_name(&self, pattern: &str) -> String {
        let pattern = Self::base_pattern(pattern);
        let name = if pattern.starts_with('.') {
            // Dot-prefixed handlers map directly to function name (Vue convention)
            pattern[1..].to_string()
        } else if let Some(variant) = pattern.split("::").last() {
            // Pattern like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", pattern)
        };
        Self::sanitize_ident(&name)
    }

    /// Convert handler reference to function call
    /// (same sanitization as pattern_to_handler_name — the two must agree so
    /// template references match the generated function definitions)
    pub(super) fn handler_to_function_call(&self, handler: &str) -> String {
        let handler = Self::base_pattern(handler);
        let name = if handler.starts_with('.') {
            // Dot-prefixed handlers map directly to function name (Vue convention)
            handler[1..].to_string()
        } else if let Some(variant) = handler.split("::").last() {
            // Handler like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", handler)
        };
        Self::sanitize_ident(&name)
    }

    /// Find the state variable that likely represents the "active id" for a given handler.
    /// Heuristic: looks for state vars ending with "_id" (e.g., "active_id" for "SelectNote")
    pub(super) fn find_active_id_var(&self, _handler_name: &str) -> Option<String> {
        // Look for a state variable ending with "_id"
        for name in &self.state_names {
            if name.ends_with("_id") {
                return Some(name.clone());
            }
        }
        None
    }

    /// Convert handler to Vue function call with parameters
    pub(super) fn handler_to_function_call_with_params(&self, handler: &str, params: &[String]) -> String {
        let func_name = self.handler_to_function_call(handler);
        if params.is_empty() {
            func_name
        } else {
            // Replace double quotes with single quotes in params to avoid HTML attr quoting issues
            let safe_params: Vec<String> = params
                .iter()
                .map(|p| Self::vue_event_param(p).replace('"', "'"))
                .collect();
            format!("{}({})", func_name, safe_params.join(", "))
        }
    }

    /// Plan 421: build the handler call for a code_editor `oncursor` /
    /// `oncontextmenu` binding, forwarding the scaffold shell's emit payload.
    ///
    /// The scaffolded CodeEditor.vue emits `cursor` with `{ line, column }`
    /// (1-based — the iced-side getters are 0-based, handlers ported from
    /// iced must not `+1` again) and `contextmenu` with `{ x, y }`. When the
    /// on-block handler declares parameters and the view binding passes none,
    /// the payload is threaded through (mirrors the sub-widget `$event`
    /// forwarding pattern): one declared param receives the whole payload
    /// object, two or more receive the destructured position args.
    ///
    /// Returns None when the binding is not a code_editor position event, the
    /// DSL already passes explicit args (they win), or the handler declares
    /// no params (nothing to forward — plain `@cursor="Handler"`).
    pub(super) fn code_editor_event_payload_call(
        &self,
        tag: &str,
        event: &str,
        aura_event: &AuraEvent,
    ) -> Option<String> {
        if !matches!(tag, "code_editor" | "codeEditor") {
            return None;
        }
        let (base_key, _mods) = Self::split_event_key(event);
        let dom = Self::base_event_to_dom(base_key);
        if !matches!(dom.as_str(), "cursor" | "contextmenu") {
            return None;
        }
        // Explicit DSL args at the binding site win over the emit payload.
        if !aura_event.params.is_empty() {
            return None;
        }
        // PLAN-671 ⑤：声明了形参的载荷事件，载荷转发优先于循环参。此前
        // v-for 内绑定的 cursor/contextmenu（`.EditorCtx(x, y)` 双参形）
        // 被此处早退让位给循环参——调用点发 `EditorCtx(i)`、签名随之
        // `(i: any)`，而 body 引用未绑定的声明形参 x/y（TS2304）。循环参
        // 仅对未声明形参的 handler 保持权威（下方 `(0, _) => None` 自然
        // 落回发射点的循环参臂，列表语义不变）。
        let bare = Self::base_pattern(&aura_event.handler);
        let name = bare.split("::").last().unwrap_or(bare);
        let key = format!(".{}", name.trim_start_matches('.'));
        let declared = Self::get_handler_params(&self.handler_params, &key)?;
        let args = match (declared.len(), dom.as_str()) {
            (0, _) => return None,
            (1, _) => "$event".to_string(),
            (_, "cursor") => "$event.line, $event.column".to_string(),
            _ => "$event.x, $event.y".to_string(),
        };
        Some(format!(
            "{}({})",
            self.handler_to_function_call(&aura_event.handler),
            args
        ))
    }

    /// Adapt one event-arg param string for the Vue template.
    ///
    /// The event-arg parser renders a standalone `.field` state reference as
    /// `this.field` (correct for ArkTS). In Vue `<script setup>` templates
    /// `this` is invalid — state refs are bare setup-scope bindings that Vue
    /// auto-unwraps — so every `this.` marker is stripped, wherever it sits in
    /// the param string: leading (`.H(.x)`), inside a map literal
    /// (`.H({ q: .x })`), or nested in a call (`.H(fmt(.x))`). P0#12: the old
    /// code stripped only a leading prefix, so map/nested positions emitted
    /// `this.query` and broke silently at runtime.
    ///
    /// A preceding identifier/`.` character guards against mangling member
    /// access like `obj.this`, and quoted string contents are left untouched.
    pub(super) fn vue_event_param(param: &str) -> String {
        let b = param.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(b.len());
        let mut i = 0;
        let mut quote: Option<u8> = None;
        while i < b.len() {
            let c = b[i];
            if let Some(q) = quote {
                out.push(c);
                if c == b'\\' && i + 1 < b.len() {
                    out.push(b[i + 1]);
                    i += 2;
                    continue;
                }
                if c == q {
                    quote = None;
                }
                i += 1;
                continue;
            }
            if c == b'"' || c == b'\'' {
                quote = Some(c);
                out.push(c);
                i += 1;
                continue;
            }
            // Plan 559 W2: VM-track change-payload syntax written in the .at
            // source (`onchange: .ApplyEntry(e.key, $event.target.value)`).
            // Vue's EventTarget carries neither member and `target` is
            // nullable — narrow to the concrete element type so the gen tree
            // passes vue-tsc (TS2339/TS18047, os-config collection_browser /
            // config_editor family).
            if c == b'$'
                && (i == 0 || !matches!(b[i - 1], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_'))
            {
                if b[i..].starts_with(b"$event.target.value") {
                    out.extend_from_slice(b"($event.target as HTMLInputElement).value");
                    i += b"$event.target.value".len();
                    continue;
                }
                if b[i..].starts_with(b"$event.target.checked") {
                    out.extend_from_slice(b"($event.target as HTMLInputElement).checked");
                    i += b"$event.target.checked".len();
                    continue;
                }
            }
            if c == b't'
                && b[i..].starts_with(b"this.")
                && (i == 0
                    || !matches!(
                        b[i - 1],
                        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$' | b'.'
                    ))
            {
                i += 5; // skip "this."
                continue;
            }
            out.push(c);
            i += 1;
        }
        // Only ASCII `this.` sequences were removed; everything else was
        // copied verbatim, so the output is valid UTF-8.
        String::from_utf8(out).unwrap_or_else(|_| param.to_string())
    }

    // ========================================================================
    // shadcn-vue Support Methods (using unified WidgetRegistry)
    // ========================================================================

    /// Register a shadcn-vue component as used
    pub(super) fn register_shadcn_component(&mut self, tag: &str) {
        if self.is_shadcn() {
            if let Some(component_name) = self.shadcn_component_name(tag) {
                self.shadcn_components_used.insert(component_name);
            }
        }
    }

    /// Generate shadcn-vue import statements using unified registry
    pub(super) fn generate_shadcn_imports(&self) -> String {
        if self.shadcn_components_used.is_empty() {
            return String::new();
        }

        // Collect all tags used and their imports
        let mut imports_by_path: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for component_name in &self.shadcn_components_used {
            // Find the widget spec that contains this component
            for (_, spec) in self.widget_registry.all_widgets().iter() {
                if let Some(mapping) = spec.backend("vue") {
                    if &mapping.component == component_name
                        || mapping.extra_components.contains(component_name)
                    {
                        if let Some(ref import_path) = mapping.import {
                            imports_by_path
                                .entry(import_path.clone())
                                .or_default()
                                .push(component_name.clone());
                        }
                    }
                }
            }
        }

        // Generate import statements
        let mut imports = Vec::new();
        for (path, mut names) in imports_by_path {
            names.sort();
            names.dedup();
            imports.push(format!(
                "import {{ {} }} from '{}'\n",
                names.join(", "),
                path
            ));
            // Plan 412 续:用到 <Toaster/>(toast-provider)的页面同样需要
            // vue-sonner v2 的样式表,否则 toaster 容器 static 定位会挤压
            // 页面(与 needs_toast_import 路径双保险)。
            if path == "@/components/ui/sonner" {
                imports.push("import 'vue-sonner/style.css'\n".to_string());
            }
        }

        imports.sort();
        imports.join("")
    }

    /// Get shadcn-vue component name for a tag
    pub(super) fn shadcn_component_name(&self, tag: &str) -> Option<String> {
        if self.is_shadcn() {
            self.widget_registry
                .get_primary_component("vue", tag)
                .or_else(|| {
                    let kebab = tag.replace('_', "-");
                    (kebab != tag)
                        .then(|| self.widget_registry.get_primary_component("vue", &kebab))
                        .flatten()
                })
        } else {
            None
        }
    }

}
