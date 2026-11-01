//! PLAN-014 T-04：样式辅助责任（extract_classes Tailwind 类提取）（自 ui_gen/vue.rs impl 块原样收编）。
//! 单 VueGenerator state、公开路径与字节输出零改——纯 impl 代码移动；
//! 方法可见性按跨模块调用需要提升（其余保持私有语义）。
use super::*;

impl VueGenerator {
    pub(crate) fn extract_classes(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
    ) -> (String, Option<String>, Option<String>) {
        let mut classes = Vec::new();
        let mut dynamic_class: Option<String> = None;
        let mut dynamic_style: Option<String> = None;

        // Normalize tag to lowercase for matching
        let normalized_tag = Self::normalize_tag(tag);

        // In shadcn mode, skip default classes for components that have shadcn versions
        // (shadcn components have their own styling).
        // However, layout primitives (row, col, etc.) always need their flex classes
        // regardless of mode — they map to <div> and have no shadcn styling of their own.
        let layout_primitives = [
            "row",
            "col",
            "column",
            "grid",
            "scroll",
            "center",
            "container",
            "square",
            "spacer",
        ];
        let is_layout_primitive = layout_primitives.contains(&normalized_tag);
        let skip_defaults = !is_layout_primitive
            && self.is_shadcn()
            && self.widget_registry.is_backend_supported("vue", tag);

        // pac.at `default_classes: off` (Plan 014): skip the doc-theme default
        // classes below for every non-layout-primitive tag. Stacks with the
        // shadcn/user-class skips above; layout primitives keep their classes
        // (they're structural — without them the layout collapses).
        let skip_defaults = skip_defaults || (!is_layout_primitive && !self.default_classes);

        // Check if user has provided a class or style attribute
        let has_user_class = props.contains_key("class") || props.contains_key("style");

        // For elements that should skip default classes when user provides their own class.
        // This covers semantic HTML elements, layout elements, typography, and form elements
        // that may need fully custom styling (e.g., TodoMVC uses todomvc-app-css).
        let user_class_skip_elements = [
            // Semantic HTML5
            "header",
            "nav",
            "main",
            "aside",
            "footer",
            "article",
            "section",
            // Typography
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "text",
            "p",
            // Form
            "button",
            "input",
            "checkbox",
            "link",
            "label",
            "textarea",
            // Data
            "tree",
            "tree_item",
            "tree-item",
        ];
        let skip_semantic_defaults =
            has_user_class && user_class_skip_elements.contains(&normalized_tag);

        // Extract gap prop for layout elements
        let user_style_has_gap =
            props
                .get("class")
                .or_else(|| props.get("style"))
                .map_or(false, |v| {
                    if let AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) = v {
                        s.split_whitespace()
                            .any(|tok| tok.starts_with("gap-") || tok == "gap")
                    } else {
                        false
                    }
                });
        let gap_class = if let Some(value) = props.get("gap") {
            match value {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => format!(" gap-{}", s),
                _ => " gap-4".to_string(),
            }
        } else if user_style_has_gap {
            String::new()
        } else {
            " gap-4".to_string()
        };

        // Default classes based on tag (only in Plain mode or for non-shadcn elements)
        if !skip_defaults && !skip_semantic_defaults {
            match normalized_tag {
                // Layout
                "col" | "column" => classes.push(format!("flex flex-col{}", gap_class)),
                "row" => classes.push(format!("flex flex-row{}", gap_class)),
                "grid" => classes.push("grid".to_string()),
                // Plan 412 §4.3: square 占位块 —— 色块 + flex 双轴居中。
                // 与 VM convert_square 的类串对应(VM 用 Container center_x/y)。
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
                    classes.push(format!(
                        "h-{h} w-{w} rounded-md border bg-{c}-500/40 border-{c}-500 text-{c}-600 font-medium text-sm flex items-center justify-center",
                        c = color
                    ));
                }
                // PLAN-656 T-07: axis 三值 → overflow 类（y/x/both）；
                // scrollbar: always → *-scroll。
                "scroll" | "scrollable" | "scroll-pane" => {
                    let axis = props
                        .get("axis")
                        .and_then(|v| self.extract_string_value(v))
                        .or_else(|| {
                            props.get("direction").and_then(|v| self.extract_string_value(v))
                        })
                        .unwrap_or("y");
                    let always = props
                        .get("scrollbar")
                        .and_then(|v| self.extract_string_value(v))
                        .map(|p| p == "always")
                        .unwrap_or(false);
                    let suffix = if always { "scroll" } else { "auto" };
                    match axis {
                        "x" | "horizontal" => classes.push(format!("overflow-x-{suffix}")),
                        "both" => classes.push(format!("overflow-{suffix}")),
                        _ => classes.push(format!("overflow-y-{suffix}")),
                    }
                }
                "container" => classes.push("max-w-7xl mx-auto".to_string()),
                "center" => classes.push("flex flex-col items-center justify-center h-full".to_string()),

                // HTML5 semantic elements (only add defaults if user hasn't provided class)
                "header" => classes.push("w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60".to_string()),
                "nav" => classes.push("flex items-center gap-4".to_string()),
                "main" => classes.push("flex-1".to_string()),
                "aside" => classes.push("w-64 border-r bg-background".to_string()),
                "footer" => classes.push("w-full border-t bg-background".to_string()),
                "article" => classes.push("prose max-w-none".to_string()),

                // Typography. Plan 409 §8: headings / links use the theme
                // color (text-primary) so page titles & nav are accent-driven.
                "h1" => classes.push("text-4xl font-bold tracking-tight mb-4 text-primary".to_string()),
                "h2" => classes.push("text-3xl font-bold tracking-tight mt-8 mb-4 text-primary".to_string()),
                "h3" => classes.push("text-xl font-semibold mb-3 text-primary".to_string()),
                "h4" => classes.push("text-lg font-semibold mb-2".to_string()),
                "h5" => classes.push("text-base font-semibold mb-1".to_string()),
                "h6" => classes.push("text-sm font-semibold mb-1".to_string()),
                "text" | "p" => classes.push("text-muted-foreground leading-7".to_string()),

                // Content
                "button" => classes.push("px-4 py-2 rounded".to_string()),
                // Plan 409 §8: input/textarea borders use the theme color.
                "input" => classes.push("border-primary rounded px-2 py-1".to_string()),
                "textarea" => classes.push("border-primary rounded px-2 py-1".to_string()),
                "checkbox" => classes.push("w-4 h-4".to_string()),
                "toggle" => classes.push("relative w-10 h-6 rounded-full".to_string()),
                "select" => classes.push("border rounded px-2 py-1".to_string()),
                "link" => classes.push("text-sm font-medium text-muted-foreground hover:text-foreground transition-colors cursor-pointer".to_string()),
                "codeblock" | "code-block" => classes.push("relative rounded-lg border bg-zinc-950 text-zinc-50 overflow-x-auto".to_string()),
                "codepane" | "code-pane" => classes.push("relative rounded-lg border bg-zinc-950 text-zinc-50 overflow-hidden".to_string()),
                "previewcard" | "preview-card" => classes.push("rounded-lg border overflow-hidden".to_string()),
                "label" => {
                    // Don't add default classes for native <label> elements
                    // (shadcn Label component is a separate widget, not plain label)
                }

                // Data
                "table" => classes.push("w-full border-collapse".to_string()),
                "thead" => classes.push("bg-muted/50".to_string()),
                "th" => classes.push("border px-4 py-2 text-left font-semibold".to_string()),
                "td" => classes.push("border px-4 py-2".to_string()),
                "tree" => classes.push("list-none pl-4".to_string()),
                "tree_item" | "tree-item" => classes.push("py-1".to_string()),

                // Navigation
                "tabs" => classes.push("flex border-b".to_string()),
                "tabslist" | "tabs-list" => classes.push("inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground".to_string()),
                "tabstrigger" | "tabs-trigger" => classes.push("inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50".to_string()),
                "tabscontent" | "tabs-content" => classes.push("mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2".to_string()),
                "tab" => classes.push("px-4 py-2 border-b-2 border-transparent".to_string()),

                // Overlay
                "modal" => classes.push("fixed inset-0 bg-black/80 flex items-center justify-center".to_string()),

                // Form
                "slider" => classes.push("w-full".to_string()),
                "radiogroup" | "radio-group" => classes.push("flex flex-col gap-2".to_string()),

                // Feedback
                "progress" => classes.push("w-full h-2 rounded".to_string()),
                "badge" => classes.push("px-2 py-1 text-xs rounded-full".to_string()),
                "spinner" => classes.push("animate-spin w-6 h-6 border-2 border-muted-foreground border-t-primary rounded-full".to_string()),

                // Display
                "card" => classes.push("rounded-lg border bg-card text-card-foreground shadow-sm".to_string()),
                "cardheader" | "card-header" => classes.push("flex flex-col space-y-1.5 p-6".to_string()),
                "cardtitle" | "card-title" => classes.push("text-lg font-semibold leading-none tracking-tight".to_string()),
                "carddescription" | "card-description" => classes.push("text-sm text-muted-foreground".to_string()),
                "cardcontent" | "card-content" => {
                    // Don't add default padding - let users control via class prop
                }
                "cardfooter" | "card-footer" => classes.push("flex items-center p-6 pt-0".to_string()),
                "avatar" => classes.push("w-10 h-10 rounded-full".to_string()),
                "aspectratio" | "aspect-ratio" => classes.push("relative w-full".to_string()),

                // Media
                "image" => classes.push("max-w-full".to_string()),
                // "icon" 的默认尺寸**不在这里**注入：icon 臂要按
                // 「显式 w-/h- 类 > size > 默认」决定，无条件塞默认会静默
                // 盖掉用户显式写的尺寸类（Tailwind 同权重）。见 icon 臂。

                // Utility
                "divider" => classes.push("shrink-0 bg-border".to_string()),
                "separator" => classes.push("shrink-0 bg-border".to_string()),
                "spacer" => classes.push("flex-1".to_string()),

                _ => {}
            }
        }

        // Process 'style' (dynamic class binding) and 'class' (static classes) independently.
        // Both can coexist: class provides static Tailwind utilities, style provides dynamic :class.
        // Process 'style' prop first (generates dynamic :class binding)
        if let Some(value) = props.get("style") {
            match value {
                AuraPropValue::StyleBinding(bindings) => {
                    // Generate dynamic class binding: { completed: todo.done, editing: todo.editing }
                    // Use expr_to_vue_bound_value (no .value suffix) because Vue templates auto-unwrap refs
                    let binding_strs: Vec<String> = bindings
                        .iter()
                        .map(|b| {
                            // Plan 012 P0#13 follow-up: was a silent `false`
                            // fallback; keep it but warn R013.
                            let cond = self.bound_value_or_warn(
                                &b.condition,
                                &format!("style: class binding `{}`", b.style_name),
                                "false",
                            );
                            // Class names may contain '-' (e.g. "line-through")
                            // — quote keys that aren't valid JS identifiers.
                            format!("{}: {}", Self::js_obj_key(&b.style_name), cond)
                        })
                        .collect();
                    dynamic_class = Some(format!("{{ {} }}", binding_strs.join(", ")));
                }
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => {
                    // Plan 496 M5：插值 class 拆分（静态段 + :class 表达式）。
                    if let Some((statics, expr)) = Self::interpolated_class_parts(s) {
                        for c in statics {
                            if is_layout_primitive {
                                let existing: Vec<&str> = classes
                                    .iter()
                                    .flat_map(|cl| cl.split_whitespace())
                                    .collect();
                                if !existing.contains(&c.as_str()) {
                                    classes.push(c);
                                }
                            } else {
                                classes.push(c);
                            }
                        }
                        dynamic_class = Some(match dynamic_class {
                            Some(prev) => format!("{prev} + ' ' + ({expr})"),
                            None => expr,
                        });
                    } else if is_layout_primitive {
                        // Dedup: for layout primitives, split user classes and skip any already present
                        for c in s.split_whitespace() {
                            let existing: Vec<&str> = classes
                                .iter()
                                .flat_map(|cl| cl.split_whitespace())
                                .collect();
                            if !existing.contains(&c) {
                                classes.push(c.to_string());
                            }
                        }
                    } else {
                        classes.push(s.to_string());
                    }
                }
                AuraPropValue::Expr(auto_lang_core::ast::Expr::If(if_stmt)) => {
                    // Plan 346 + 043 M5 #tag-coloring: conditional style →
                    // Vue :class ternary. Full if/else-if/else chains become
                    // nested ternaries (if_expr_to_style_ternary).
                    dynamic_class = Some(self.if_expr_to_style_ternary(if_stmt));
                }
                // Plan 448 D: `style: ["base", if … {…} else {…}, …]` —
                // array-of-class-parts. The array form is EXPLICITLY class
                // semantics (unlike a bare concat, which may be an inline
                // CSS declaration string), so every element binds into one
                // :class join: literals quoted, conditionals as ternaries,
                // state refs as bound values; empty parts filtered at
                // runtime so optional classes never double-space.
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Array(elems)) => {
                    let parts: Vec<String> = elems
                        .iter()
                        .map(|e| self.style_array_part_js(e))
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap_or_default();
                    if parts.is_empty() {
                        self.warn(
                            "R013",
                            crate::ui_gen::validators::Severity::Warning,
                            "style: array with no renderable parts; binding not emitted"
                                .to_string(),
                        );
                    } else {
                        let joined = format!("[{}].filter(Boolean).join(' ')", parts.join(", "));
                        dynamic_class = Some(match dynamic_class {
                            Some(prev) => format!("({}) + ' ' + {}", prev, joined),
                            None => joined,
                        });
                    }
                }
                AuraPropValue::Expr(other_expr) => {
                    // Plan 043 H5 / Plan 408 P12 §10.1: a dynamic style
                    // expression that is neither a string literal nor a
                    // conditional — e.g. a string concat like `"color: rgb("
                    // + r + "," + ...`. Produces a CSS declaration string at
                    // runtime → emit as an independent `:style` binding. The
                    // old `__style__` marker (which shared the class channel
                    // and leaked when dynamic class coexisted) is gone —
                    // style now has its own return slot.
                    //
                    // Plan 448 D carve-out: a concat whose operands are all
                    // string literals / string-conditionals is class-safe by
                    // structure (`style: "base" + if … {…} else {…}`) —
                    // route it to :class, not :style (a Tailwind class string
                    // in :style is silently discarded as invalid CSS).
                    if Self::is_class_safe_concat(other_expr) {
                        match self.expr_to_vue_bound_value(other_expr) {
                            Ok(expr_str) => {
                                dynamic_class = Some(match dynamic_class {
                                    Some(prev) => format!("({}) + ' ' + ({})", prev, expr_str),
                                    None => expr_str,
                                });
                            }
                            Err(e) => self.warn(
                                "R013",
                                crate::ui_gen::validators::Severity::Warning,
                                format!("style: class concat: {}; binding not emitted", e),
                            ),
                        }
                    } else {
                        match self.expr_to_vue_bound_value(other_expr) {
                            Ok(expr_str) => {
                                // Plan 653 D4: 插值类串(f-string)落到此通道时
                                // 是 Tailwind 类表而非 CSS 声明——`:style` 整串
                                // 被浏览器忽略,槽位零定位(653 D4 病灶)。
                                // 全 token 可识别的类串模板翻译为等价
                                // inline CSS;其余形态原样保留。
                                let expr_str =
                                    Self::tailwind_class_template_to_inline_style(&expr_str)
                                        .unwrap_or(expr_str);
                                dynamic_style = Some(expr_str);
                            }
                            // Plan 012 P0#13 follow-up: used to be silently
                            // dropped (the Err branch was unreachable while the
                            // catch-all returned Ok("null")); warn R013.
                            Err(e) => self.warn(
                                "R013",
                                crate::ui_gen::validators::Severity::Warning,
                                format!("style: dynamic expression: {}; binding not emitted", e),
                            ),
                        }
                    }
                }
                _ => {}
            }
        }
        // Process 'class' prop (static Tailwind classes)
        if let Some(value) = props.get("class") {
            match value {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => {
                    // Plan 496 M5：插值 class 拆分（style 分支同型）。
                    if let Some((statics, expr)) = Self::interpolated_class_parts(s) {
                        for c in statics {
                            if is_layout_primitive {
                                let existing: Vec<&str> = classes
                                    .iter()
                                    .flat_map(|cl| cl.split_whitespace())
                                    .collect();
                                if !existing.contains(&c.as_str()) {
                                    classes.push(c);
                                }
                            } else {
                                classes.push(c);
                            }
                        }
                        dynamic_class = Some(match dynamic_class {
                            Some(prev) => format!("{prev} + ' ' + ({expr})"),
                            None => expr,
                        });
                    } else if is_layout_primitive {
                        for c in s.split_whitespace() {
                            let existing: Vec<&str> = classes
                                .iter()
                                .flat_map(|cl| cl.split_whitespace())
                                .collect();
                            if !existing.contains(&c) {
                                classes.push(c.to_string());
                            }
                        }
                    } else {
                        classes.push(s.to_string());
                    }
                }
                // Plan 012 Batch A (gap 20): a NON-literal `class:` expression
                // (`class: .cls`, `class: if ...`) used to be silently dropped
                // here (`_ => {}`), emitting the bare element with no class at
                // all. Bind it dynamically instead — same treatment as the
                // dynamic `style:` prop (ternary for If, :class expr otherwise).
                AuraPropValue::Expr(auto_lang_core::ast::Expr::If(if_stmt)) => {
                    let ternary = self.if_expr_to_style_ternary(if_stmt);
                    dynamic_class = Some(match dynamic_class {
                        Some(existing) => format!("[{}, {}]", existing, ternary),
                        None => ternary,
                    });
                }
                AuraPropValue::Expr(other_expr) => {
                    // Plan 012 P0#13: the catch-all in expr_to_vue_bound_value
                    // used to yield literal "null" for expression forms it
                    // can't render; it now returns Err. Never emit that (or
                    // drop the prop) silently — reject with a loud R011.
                    match self.expr_to_vue_bound_value(other_expr) {
                        Ok(expr_str) if expr_str != "null" => {
                            dynamic_class = Some(match dynamic_class {
                                Some(existing) => format!("[{}, {}]", existing, expr_str),
                                None => expr_str,
                            });
                        }
                        Ok(_) => self.warn(
                            "R011",
                            crate::ui_gen::validators::Severity::Warning,
                            "class: expression form is not supported and was not emitted",
                        ),
                        Err(e) => self.warn(
                            "R011",
                            crate::ui_gen::validators::Severity::Warning,
                            format!(
                                "class: expression form is not supported and was not emitted: {}",
                                e
                            ),
                        ),
                    }
                }
                _ => {}
            }
        }

        (classes.join(" "), dynamic_class, dynamic_style)
    }

}
