//! PLAN-014 T-04：标签映射责任（map_tag/is_svg_element/normalize_tag/style_obj_to_vue）（自 ui_gen/vue.rs impl 块原样收编）。
//! 单 VueGenerator state、公开路径与字节输出零改——纯 impl 代码移动；
//! 方法可见性按跨模块调用需要提升（其余保持私有语义）。
use super::*;

impl VueGenerator {
    /// Map AutoUI tag to HTML tag or shadcn-vue component
    pub(super) fn map_tag(&mut self, tag: &str, self_closing: bool) -> String {
        // Priority: known sub-widgets > external (widget `use`) components >
        // shadcn components > HTML fallback
        // If tag matches a known sub-widget, treat as custom component reference
        // Plan 435 P4:折叠桥接 —— kebab tag(copy-button)↔ Pascal widget 名
        // (CopyButton),返回规范名保证 import/文件名大小写正确;
        // 内置可解析的 tag 不桥接(builtin 优先)。
        let sub_widget_canon: Option<String> = if self.known_sub_widgets.contains(tag) {
            Some(tag.to_string())
        } else if !tag_is_builtin(tag) && tag_has_component_shape(tag) {
            self.known_sub_widgets
                .iter()
                .find(|w| fold_eq(w, tag))
                .cloned()
        } else {
            None
        };
        if let Some(canon) = sub_widget_canon {
            if !self.component_refs.contains(&canon) {
                self.component_refs.push(canon.clone());
            }
            return canon;
        }

        // Widget-declared external components (`use { component: ... }`) win
        // over the built-in registry so user declarations can shadow/extend it.
        if let Some(comp) = self.ext_components.get(tag) {
            return comp.name.clone();
        }

        // If in shadcn mode and tag has a shadcn component, use it
        if self.is_shadcn() {
            // nav-link maps to router-link (not a shadcn component)
            if tag == "nav-link" {
                return "router-link".to_string();
            }
            // theme-toggle maps to ThemeToggle custom component
            if tag == "theme-toggle" || tag == "theme_toggle" {
                self.component_refs.push("ThemeToggle".to_string());
                self.use_theme_toggle = true;
                return "ThemeToggle".to_string();
            }
            // Plan 413: code_editor renders the scaffolded CodeMirror shell
            // (src/components/CodeEditor.vue, written by auto-man). Imported
            // via the component_refs default-style path; degrades to a plain
            // textarea inside the shell when deps are missing is NOT needed —
            // the scaffold ships them.
            if tag == "code_editor" || tag == "codeEditor" {
                self.component_refs.push("CodeEditor".to_string());
                return "CodeEditor".to_string();
            }
            // tooltip-provider → TooltipProvider. reka-ui <Tooltip> requires a
            // <TooltipProvider> ancestor (inject context); without it the page
            // white-screens. Insert into shadcn_components_used so the import is
            // auto-generated from the registry (tooltip → @/components/ui/tooltip).
            if tag == "tooltip-provider" {
                self.shadcn_components_used
                    .insert("TooltipProvider".to_string());
                return "TooltipProvider".to_string();
            }
            // Plan 408: shadcn-vue deprecated the declarative toast components
            // (Toast/ToastTitle/ToastDescription) in favor of Sonner, which is
            // programmatic (toast() + a root-level <Toaster/>). The DSL has no
            // declarative sonner equivalents, so the toast card structure is
            // rendered as plain HTML (buildable output). Sonner scaffolding is
            // triggered by toast-provider below, which maps to <Toaster/> via
            // the widget registry (previously these tags bypassed component
            // detection entirely and were mapped to div/span).
            if tag == "toast" || tag == "toast-title" || tag == "toast-description" {
                return (if tag == "toast" { "div" } else { "span" }).to_string();
            }
            // Plan 408 P8 / §7.5 缺陷 7: keep native HTML table elements as-is.
            // These are semantic HTML tags; mapping `table` to shadcn <Table>
            // forced an unwanted import + component wrapper for users who want
            // plain `<table>` (e.g. escape-hatch StreamingTable → component fn
            // with native table markup). PascalCase `Table` still resolves to
            // the shadcn component via the sub-widget/ext-component path above.
            // Note: `col`/`caption`/`colgroup` are intentionally excluded —
            // `col` collides with the Auto layout `col` (→ div/flex column).
            if matches!(
                tag,
                "table" | "thead" | "tbody" | "tfoot" | "tr" | "th" | "td"
            ) {
                return tag.to_string();
            }
            // PLAN-661 T-04（R-5 裁定）：slider 双模式统一原生
            // `<input type="range">`——shadcn-vue Slider 组件路径退役
            //（Plan 408 :default-value 数组值不同步旧限让位；与 a2r/VM
            // 轨语义对齐的最短路径，零组件依赖）。
            if tag == "slider" || tag == "Slider" {
                return "input".to_string();
            }
            if let Some(component_name) = self.shadcn_component_name(tag) {
                self.register_shadcn_component(tag);
                return component_name.to_string();
            }
        }

        // Fallback to plain HTML tags
        match tag {
            // Layout (no shadcn components, use Tailwind)
            "col" | "column" | "Col" | "Column" => "div".to_string(),
            "row" | "Row" => "div".to_string(),
            "grid" | "Grid" => "div".to_string(),
            // Plan 412 §4.3: demo 占位块(色块 + flex 居中数字)
            "square" | "Square" => "div".to_string(),
            "scroll" | "Scroll" | "scroll_test_content" => "div".to_string(),
            "container" | "Container" => "div".to_string(),
            // PLAN-031 T-05：`panel` 是基础表面容器（§5.4）——Vue 侧与 VM
            // 容器臂同名对齐，显式落 `<div>`。只收小写：PascalCase `Panel`
            // 是子组件引用（slot 填充语义），不能坍缩成 div。
            "panel" => "div".to_string(),
            "center" | "Center" => "div".to_string(),

            // Plan 041a①(musk 041 Phase 5 T19): 原生 HTML 语义元素直通。
            // 这些元素在 schema 为 native_html(web:"none")——shadcn 注册表
            // 不支持故落此兜底,此前无 arm 恒坍缩为 div,原生语义不可达
            // (musk 曾以 tab 元素/ext 组件逃生,见 KNOWN-DEBT 041a①)。
            // native_select/native_button 为显式原生逃生名(避开 select 的
            // shadcn Select 拦截与 button 的 Button 映射)。
            "native_select" => "select".to_string(),
            "native_button" => "button".to_string(),
            "pre" | "Pre" => "pre".to_string(),
            "code" | "Code" => "code".to_string(),
            "ol" | "Ol" => "ol".to_string(),
            "ul" | "Ul" => "ul".to_string(),
            "li" | "Li" => "li".to_string(),
            "dl" | "Dl" => "dl".to_string(),
            "dt" | "Dt" => "dt".to_string(),
            "dd" | "Dd" => "dd".to_string(),
            "option" | "Option" => "option".to_string(),
            "optgroup" => "optgroup".to_string(),
            "figure" | "Figure" => "figure".to_string(),
            "figcaption" => "figcaption".to_string(),
            "blockquote" => "blockquote".to_string(),

            // HTML5 semantic elements
            "header" | "Header" => "header".to_string(),
            "nav" | "Nav" => "nav".to_string(),
            "main" | "Main" => "main".to_string(),
            "section" | "Section" => "section".to_string(),
            "aside" | "Aside" => "aside".to_string(),
            "footer" | "Footer" => "footer".to_string(),
            "article" | "Article" => "article".to_string(),

            // Content
            "button" | "Button" => "button".to_string(),
            "input" | "Input" => "input".to_string(),
            "textarea" | "Textarea" => "textarea".to_string(),
            "checkbox" | "Checkbox" => "input".to_string(),
            "toggle" | "Toggle" => "button".to_string(),
            // Plan 562 T6: Plain 模式（pac.at shadcn:off）sidebar_menu_button
            // 保 button 原生语义——否则坍缩为 div 丢键盘/类型语义（auto-os-config
            // 迁移实测）。契约类不内联：Plain 哲学 = 项目自带样式，shadcn 组件
            // 类与 sidebar 令牌在此模式均为死类（os-config tailwind 令牌表无
            // sidebar-accent/accent 条目）。active/size 等 props 走通用属性
            // 透传（:active 落为原生属性，兼作测试锚）。
            "sidebar_menu_button"
            | "sidebar-menu-button"
            | "sidebar_menu_sub_button"
            | "sidebar-menu-sub-button" => "button".to_string(),
            "select" | "Select" => "select".to_string(),
            "option" | "Option" => "option".to_string(),
            "link" | "Link" => "a".to_string(),
            "codeblock" | "code-block" | "CodeBlock" | "Codeblock" => "pre".to_string(),
            "codepane" | "code-pane" | "CodePane" => "div".to_string(),
            "previewcard" | "preview-card" | "PreviewCard" => "div".to_string(),

            // Typography (no shadcn components) - PascalCase maps to lowercase HTML
            "h1" | "H1" => "h1".to_string(),
            "h2" | "H2" => "h2".to_string(),
            "h3" | "H3" => "h3".to_string(),
            "h4" | "H4" => "h4".to_string(),
            "h5" | "H5" => "h5".to_string(),
            "h6" | "H6" => "h6".to_string(),
            "text" | "Text" => "span".to_string(),
            "label" | "Label" => "label".to_string(),
            "span" | "Span" => "span".to_string(),
            "p" | "P" => "p".to_string(),

            // Data
            "table" | "Table" => "table".to_string(),
            "thead" | "Thead" => "thead".to_string(),
            "tbody" | "Tbody" => "tbody".to_string(),
            "tr" | "Tr" => "tr".to_string(),
            "th" | "Th" => "th".to_string(),
            "td" | "Td" => "td".to_string(),
            "tree" | "Tree" => "ul".to_string(),
            "tree_item" | "tree-item" | "TreeItem" => "li".to_string(),

            // Navigation
            "tabs" | "Tabs" => "div".to_string(),
            "tab" | "Tab" => "button".to_string(),

            // Overlay
            "modal" | "Modal" => "div".to_string(),
            "tooltip" | "Tooltip" => "span".to_string(),

            // Form
            "slider" | "Slider" => "input".to_string(),
            "radio" | "Radio" => "input".to_string(),
            "radiogroup" | "radio-group" | "RadioGroup" => "div".to_string(),

            // Feedback
            "progress" | "Progress" => "progress".to_string(),
            "badge" | "Badge" => "span".to_string(),
            "spinner" | "Spinner" => "div".to_string(),

            // Display - Card is a special component, not a plain div
            "card" => "div".to_string(),
            "avatar" | "Avatar" => "img".to_string(),
            "aspectratio" | "aspect-ratio" | "AspectRatio" => "div".to_string(),

            // Media
            "image" | "Image" => "img".to_string(),
            "img" | "Img" => "img".to_string(),
            "icon" | "Icon" => "span".to_string(),
            "video" | "Video" => "video".to_string(),
            "audio" | "Audio" => "audio".to_string(),

            // Utility
            "divider" | "Divider" => "hr".to_string(),
            "spacer" | "Spacer" => "div".to_string(),

            // Special
            "div" => "div".to_string(),
            // PLAN-760: remote-frame 宿主占位 div（shadcn 模式经 carried_vue
            // component "div" 同路;此 fallback 保非 shadcn 形态）。
            "remote-frame" => "div".to_string(),
            "+" => {
                if self_closing {
                    "span".to_string()
                } else {
                    "span".to_string()
                }
            }
            "-" => {
                if self_closing {
                    "span".to_string()
                } else {
                    "span".to_string()
                }
            }

            // Plan 442 A4: SVG 元素按原名直通(此前 svg/path 退化成
            // <div :viewBox=...>,SVG 语义全丢——musk-038 T9 canary)。
            // 小写直通;PascalCase(如 Svg)仍走组件引用路径。text 不在此
            // 列(与 DSL text→span 冲突),SVG 文本节点暂不支持。
            "svg" | "path" | "circle" | "rect" | "line" | "polyline" | "polygon" | "ellipse"
            | "g" | "defs" | "use" | "stop" | "linearGradient" | "radialGradient" | "clipPath"
            | "mask" => tag.to_string(),

            _ => {
                // Check if it's a PascalCase component name
                if tag
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    self.component_refs.push(tag.to_string());
                    tag.to_string()
                } else {
                    "div".to_string()
                }
            }
        }
    }

    /// Plan 442 A4: the native-SVG passthrough tag set (see `map_tag`).
    pub(super) fn is_svg_element(tag: &str) -> bool {
        matches!(
            tag,
            "svg"
                | "path"
                | "circle"
                | "rect"
                | "line"
                | "polyline"
                | "polygon"
                | "ellipse"
                | "g"
                | "defs"
                | "use"
                | "stop"
                | "linearGradient"
                | "radialGradient"
                | "clipPath"
                | "mask"
        )
    }

    /// Plan 442 A4: literal expr → static SVG attribute value (strings
    /// raw-escaped, numbers/bools as-is). Anything dynamic returns None and
    /// keeps the v-bind path below.
    pub(super) fn svg_static_attr_value(expr: &auto_lang_core::ast::Expr) -> Option<String> {
        use auto_lang_core::ast::Expr;
        match expr {
            Expr::Str(s) | Expr::CStr(s) => Some(s.to_string().replace('"', "&quot;")),
            Expr::Bool(b) => Some(b.to_string()),
            Expr::Int(v) => Some(v.to_string()),
            Expr::Uint(v) => Some(v.to_string()),
            Expr::I8(v) => Some(v.to_string()),
            Expr::U8(v) | Expr::Byte(v) => Some(v.to_string()),
            Expr::I64(v) => Some(v.to_string()),
            Expr::U64(v) => Some(v.to_string()),
            Expr::Float(_, raw) | Expr::Double(_, raw) => Some(raw.to_string()),
            _ => None,
        }
    }

    /// Normalize tag name to lowercase for matching
    /// Plan 435 P7-4(D5):统一入口 —— schema canonical(aliases 驱动
    /// Col/Column/CardHeader 等差异拼写)。**跨元素别名保形**:div(≡container)
    /// 与 hr(≡divider)是渲染层等价、类发射层历史区分的纯 HTML 直通拼写,
    /// 归一会让普通 div 吃到容器默认类 —— 跳过。未登记拼写原样返回(用户组件)。
    pub(super) fn normalize_tag(tag: &str) -> &str {
        const KEEP_AS_IS: &[&str] = &["div", "hr"];
        if KEEP_AS_IS.contains(&tag) {
            return tag;
        }
        crate::aura::default_schema_cached()
            .and_then(|s| crate::aura::schema::normalize_tag(tag, s))
            .unwrap_or(tag)
    }

    /// Render a key for a JS object literal: bare when it is a valid JS
    /// identifier, single-quoted otherwise (CSS props like `z-index`, class
    /// names like `line-through`).
    pub(super) fn js_obj_key(name: &str) -> String {
        let valid = !name.is_empty()
            && name
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
                .unwrap_or(false)
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
        if valid {
            name.to_string()
        } else {
            format!("'{}'", name.replace('\\', "\\\\").replace('\'', "\\'"))
        }
    }

    /// Render a `style_obj: { ... }` map as a Vue `:style` object binding.
    /// Values are arbitrary expressions (state refs, f-string px concat, …).
    /// The `as any` cast keeps loosely-typed values (e.g. a plain `string`
    /// state var for `visibility`, whose CSS type is a literal union)
    /// assignable to Vue's `StyleValue`.
    pub(super) fn style_obj_to_vue(&self, bindings: &[AuraStyleBinding]) -> String {
        let parts: Vec<String> = bindings
            .iter()
            .map(|b| {
                // Plan 012 P0#13 follow-up: was a silent `null` fallback;
                // keep the fallback but warn R013 with the style key.
                let v = self.bound_value_or_warn(
                    &b.condition,
                    &format!("style_obj binding `{}`", b.style_name),
                    "null",
                );
                format!("{}: {}", Self::js_obj_key(&b.style_name), v)
            })
            .collect();
        format!("({{ {} }} as any)", parts.join(", "))
    }

    /// Extract Tailwind classes from tag and props
    /// Returns (static_classes, dynamic_class_binding)
    /// Plan 496 M5：class/style 字符串携带 `${.field}` 插值时（desktop.at
    /// 根 `style: "w-full h-full p-3 ${.__desktop_bg}"` 形态）——此前插值
    /// 段原样落入静态 class（浏览器侧废 token）。拆分：静态段并入 class，
    /// 插值段转 `:class` 拼接表达式（`'w-full ' + __desktop_bg`）。
    /// 字段名非法（空/非标识符）→ None（维持旧行为）。
    /// Plan 503: pub(crate) —— plan503_tests 直测循环成员插值拆分。
    /// 形态: `${.field}` → 状态 ref;`${member.field}`(Plan 503) → v-for
    /// 成员点路径表达式(:class 绑定落在 v-for 作用域内求值)。
    pub fn interpolated_class_parts(s: &str) -> Option<(Vec<String>, String)> {
        if !s.contains("${") {
            return None;
        }
        let mut statics: Vec<String> = Vec::new();
        let mut parts: Vec<String> = Vec::new();
        let mut rest = s;
        while let Some(pos) = rest.find("${") {
            let head = rest[..pos].trim();
            if !head.is_empty() {
                statics.push(head.to_string());
                parts.push(format!("'{}'", head));
            }
            let after = &rest[pos..];
            let end = after.find('}')?;
            let inner = &after[2..end];
            let valid_ident =
                |n: &str| !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_');
            let expr = if let Some(name) = inner.strip_prefix('.') {
                if !valid_ident(name) {
                    return None;
                }
                name.to_string()
            } else if let Some((root, field)) = inner.split_once('.') {
                if !valid_ident(root) || !valid_ident(field) {
                    return None;
                }
                format!("{root}.{field}")
            } else {
                return None;
            };
            parts.push(expr);
            rest = &after[end + 1..];
        }
        let tail = rest.trim();
        if !tail.is_empty() {
            statics.push(tail.to_string());
            parts.push(format!("'{}'", tail));
        }
        Some((statics, parts.join(" + ")))
    }

    /// Plan 653 D4: Tailwind 类串模板 → 真 inline CSS 声明串。
    ///
    /// app 源惯用 `style: f"absolute z-10 top-[${.y}px] …"`(Tailwind 类
    /// 表,几何值是运行时插值)。该串按 CSS 声明语义发成 `:style` 会被
    /// 浏览器整串忽略(类名不是合法 CSS),槽位 div 零定位零尺寸落文档
    /// 流,页面塌成纵向文档(653 实测 2209px 翻页);运行时插值又排除
    /// :class 任意值路线(任意值类是编译期字面量)。全部 token 可识别
    /// 时翻译为等价 inline CSS;任一 token 未知/形态不符则整体放弃
    /// (返回 None,回退原语义)——不静默丢样式,不做部分翻译。
    ///
    /// 输入是已渲染的 JS 表达式串(模板字面量/引号串);`${…}` 插值
    /// 原样保留。识别集 = 本仓 app 惯用面:position 关键字、z-{n}、
    /// top/left/right/bottom/w/h/min-w/min-h/max-w/max-h/inset-[…]、
    /// bg-[…]。
    pub(super) fn tailwind_class_template_to_inline_style(expr: &str) -> Option<String> {
        let content = expr.trim();
        let (content, quote) = if let Some(inner) =
            content.strip_prefix('`').and_then(|c| c.strip_suffix('`'))
        {
            (inner, '`')
        } else if let Some(inner) = content.strip_prefix('"').and_then(|c| c.strip_suffix('"')) {
            (inner, '"')
        } else if let Some(inner) = content
            .strip_prefix('\'')
            .and_then(|c| c.strip_suffix('\''))
        {
            (inner, '\'')
        } else {
            // 拼接/函数调用等非字面量形态不在此层处理。
            return None;
        };
        if content.is_empty() {
            return None;
        }
        // 按空白切 token;`${…}` 插值内部不切。
        let mut tokens: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut in_interp = false;
        for ch in content.chars() {
            if in_interp {
                cur.push(ch);
                if ch == '}' {
                    in_interp = false;
                }
            } else {
                match ch {
                    '$' => {
                        in_interp = true;
                        cur.push(ch);
                    }
                    c if c.is_whitespace() => {
                        if !cur.is_empty() {
                            tokens.push(std::mem::take(&mut cur));
                        }
                    }
                    c => cur.push(c),
                }
            }
        }
        if !cur.is_empty() {
            tokens.push(cur);
        }
        // 含 CSS 冒号的串(如 "color: rgb(…)")是声明串不是类串,不翻译。
        if tokens.iter().any(|t| t.contains(':')) {
            return None;
        }
        let mut decls: Vec<String> = Vec::new();
        for t in &tokens {
            decls.push(Self::tailwind_token_to_css(t)?);
        }
        Some(format!("{}{}{}", quote, decls.join(";"), quote))
    }

    /// 单个 Tailwind token → CSS 声明。未知 token 返回 None(上层整体放弃)。
    pub(super) fn tailwind_token_to_css(token: &str) -> Option<String> {
        match token {
            "absolute" => return Some("position:absolute".to_string()),
            "relative" => return Some("position:relative".to_string()),
            "fixed" => return Some("position:fixed".to_string()),
            "sticky" => return Some("position:sticky".to_string()),
            _ => {}
        }
        let (prop, val) = token.split_once('-')?;
        let v = |val: &str| -> Option<String> {
            val.strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .map(|s| s.to_string())
        };
        match prop {
            "z" => Some(format!("z-index:{}", val)),
            "top" | "left" | "right" | "bottom" => Some(format!("{}:{}", prop, v(val)?)),
            "w" => Some(format!("width:{}", v(val)?)),
            "h" => Some(format!("height:{}", v(val)?)),
            "min-w" => Some(format!("min-width:{}", v(val)?)),
            "min-h" => Some(format!("min-height:{}", v(val)?)),
            "max-w" => Some(format!("max-width:{}", v(val)?)),
            "max-h" => Some(format!("max-height:{}", v(val)?)),
            "inset" => Some(format!("inset:{}", v(val)?)),
            "bg" => Some(format!("background:{}", v(val)?)),
            _ => None,
        }
    }

}
