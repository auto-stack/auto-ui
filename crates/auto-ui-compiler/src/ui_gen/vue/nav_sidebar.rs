//! PLAN-014 T-04：nav/sidebar HTML 生成（generate_nav_* / generate_sidebar_*）（自 ui_gen/vue.rs impl 块原样收编）。
//! 单 VueGenerator state、公开路径与字节输出零改——纯 impl 代码移动；
//! 方法可见性按跨模块调用需要提升（其余保持私有语义）。
use super::*;

impl VueGenerator {
    /// Plan 482: `<nav-item>` → NavItem 组件（shadcn）或内联契约标记。
    pub(super) fn generate_nav_item_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        use crate::ui_gen::nav_contract as nc;

        let ind = "  ".repeat(indent);
        let mut attrs: Vec<String> = Vec::new();

        // to / label / desc / badge / size / exact —— 静态或绑定。
        for key in ["to", "label", "desc", "badge", "size", "exact"] {
            if let Some(value) = props.get(key) {
                if let Some(frag) = self.nav_attr_fragment(key, value) {
                    attrs.push(frag);
                }
            }
        }
        if attrs
            .iter()
            .any(|a| a.starts_with("to=") || a.starts_with(":to="))
        {
            self.needs_router = true;
        }
        // active / disabled —— 布尔绑定。
        for key in ["active", "disabled"] {
            if let Some(value) = props.get(key) {
                if let Some(frag) = self.nav_attr_fragment(key, value) {
                    attrs.push(frag);
                }
            }
        }
        // icon：字面量 lucide 名 → 收集 import 以组件传入；emoji/绑定 → icon prop。
        if let Some(icon_value) = props.get("icon") {
            if let Some(name) = self.extract_string_value(icon_value) {
                let is_lucide = !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                    && name.starts_with(|c: char| c.is_ascii_lowercase());
                if is_lucide {
                    let lucide = Self::lucide_component_name(&name);
                    self.lucide_icons.insert(lucide.clone());
                    attrs.push(format!(":icon-comp=\"{}\"", lucide));
                } else if !name.is_empty() {
                    attrs.push(format!("icon=\"{}\"", Self::escape_html_attr(&name)));
                }
            } else if let Some(frag) = self.nav_attr_fragment("icon", icon_value) {
                attrs.push(frag);
            }
        }
        if let Some(click) = self.nav_click_attr(events) {
            attrs.push(click);
        }
        if let Some(cls) = self.nav_user_class_attr(props) {
            attrs.push(cls);
        }

        let mut html = String::new();
        if self.is_shadcn() {
            self.shadcn_components_used.insert("NavItem".to_string());
            if children.is_empty() {
                html.push_str(&format!("{}<NavItem {} />\n", ind, attrs.join(" ")));
            } else {
                html.push_str(&format!("{}<NavItem {}>\n", ind, attrs.join(" ")));
                for child in children {
                    html.push_str(&self.node_to_html(child, indent + 1)?);
                }
                html.push_str(&format!("{}</NavItem>\n", ind));
            }
            return Ok(html);
        }

        // 非 shadcn：内联契约标记。静态基类 + :class 三态切换（active 字面
        // 布尔则烘焙进静态串；表达式则进 :class 三元）。
        let size = props
            .get("size")
            .and_then(|v| self.extract_string_value(v))
            .unwrap_or("");
        let base = match size {
            "sm" => nc::ITEM_BASE_SM,
            "lg" => nc::ITEM_BASE_LG,
            _ => nc::ITEM_BASE_MD,
        };
        let static_active = props
            .get("active")
            .and_then(|v| match v {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);
        let static_disabled = props
            .get("disabled")
            .and_then(|v| match v {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);
        let mut static_cls = base.to_string();
        if static_active {
            static_cls.push(' ');
            static_cls.push_str(nc::ITEM_ACTIVE);
        } else if static_disabled {
            static_cls.push(' ');
            static_cls.push_str(nc::ITEM_DISABLED);
        }
        // Plan 482 补:用户静态 class 并入基类串（重复 class 属性在 Vue 模板
        // 里后者胜出且告警）；绑定态 class 保留独立 :class。
        if let Some(user) = props.get("class").or_else(|| props.get("style")) {
            if let Some(u) = self.extract_string_value(user) {
                if !u.trim().is_empty() {
                    static_cls.push(' ');
                    static_cls.push_str(u.trim());
                }
            }
        }
        let mut inline_attrs: Vec<String> = vec![format!("class=\"{}\"", static_cls)];
        if let Some(state_ref) = props
            .get("class")
            .or_else(|| props.get("style"))
            .and_then(|v| {
                if self.extract_string_value(v).is_some() {
                    None
                } else {
                    self.extract_state_ref(v)
                }
            })
        {
            inline_attrs.push(format!(":class=\"{}\"", state_ref));
        }
        // Plan 482 补:data-active 语义锚(测试/无障碍;os-config e2e 同款用法)。
        if let Some(value) = props.get("active") {
            match value {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(true)) => {
                    inline_attrs.push("data-active=\"true\"".to_string());
                }
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(false)) => {
                    inline_attrs.push("data-active=\"false\"".to_string());
                }
                AuraPropValue::Expr(expr) => {
                    let cond = self.bound_value_or_warn(expr, "nav-item active", "false");
                    inline_attrs.push(format!(":data-active=\"!!({})\"", cond));
                }
                _ => {}
            }
        }
        if !static_active && !static_disabled {
            if let Some(value) = props.get("active") {
                if let AuraPropValue::Expr(expr) = value {
                    let cond = self.bound_value_or_warn(expr, "nav-item active", "false");
                    inline_attrs.push(format!(
                        ":class=\"{} ? '{}' : '{}'\"",
                        cond,
                        nc::ITEM_ACTIVE.replace('"', "'"),
                        nc::ITEM_HOVER.replace('"', "'")
                    ));
                } else {
                    inline_attrs.push(format!("class=\"{}\"", nc::ITEM_HOVER));
                }
            } else {
                inline_attrs.push(format!("class=\"{}\"", nc::ITEM_HOVER));
            }
        }
        // 嵌套路由模式 → router-link；否则 button。
        let has_to = attrs
            .iter()
            .any(|a| a.starts_with("to=") || a.starts_with(":to="));
        let tag_name = if has_to { "router-link" } else { "button" };
        if tag_name == "button" {
            inline_attrs.push("type=\"button\"".to_string());
            if static_disabled {
                inline_attrs.push("disabled".to_string());
            } else if let Some(value) = props.get("disabled") {
                if let Some(frag) = self.nav_attr_fragment("disabled", value) {
                    inline_attrs.push(frag);
                }
            }
        } else {
            if let Some(to) = props.get("to").and_then(|v| self.extract_string_value(v)) {
                inline_attrs.push(format!("to=\"{}\"", Self::escape_html_attr(&to)));
            } else if let Some(value) = props.get("to") {
                if let Some(frag) = self.nav_attr_fragment("to", value) {
                    inline_attrs.push(frag);
                }
            }
            self.needs_router = true;
        }
        if let Some(click) = attrs.iter().find(|a| a.starts_with("@click=")).cloned() {
            inline_attrs.push(click);
        }

        // 内容：children 优先；否则 icon + label/desc + badge 合成。
        let icon_cls = if size == "lg" {
            nc::ICON_LG
        } else {
            nc::ICON_MD
        };
        let mut content = String::new();
        if !children.is_empty() {
            for child in children {
                content.push_str(&self.node_to_html(child, indent + 1)?);
            }
        } else {
            let cind = format!("{}  ", ind);
            if let Some(icon_value) = props.get("icon") {
                if let Some(name) = self.extract_string_value(icon_value) {
                    let is_lucide = !name.is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                        && name.starts_with(|c: char| c.is_ascii_lowercase());
                    if is_lucide {
                        let lucide = Self::lucide_component_name(&name);
                        self.lucide_icons.insert(lucide.clone());
                        content
                            .push_str(&format!("{}<{} class=\"{}\" />\n", cind, lucide, icon_cls));
                    } else if !name.is_empty() {
                        content.push_str(&format!(
                            "{}<span class=\"inline-flex items-center justify-center {}\">{}</span>\n",
                            cind, icon_cls, Self::escape_html_text(&name)
                        ));
                    }
                } else if let AuraPropValue::Expr(expr) = icon_value {
                    let js = self.bound_value_or_warn(expr, "nav-item icon", "''");
                    content.push_str(&format!(
                        "{}<span class=\"inline-flex items-center justify-center {}\">{{{{{}}}}}</span>\n",
                        cind, icon_cls, js
                    ));
                }
            }
            let has_label = props.contains_key("label");
            let has_desc = props.contains_key("desc");
            if has_label || has_desc {
                let badge = props
                    .get("badge")
                    .and_then(|v| self.extract_string_value(v))
                    .unwrap_or_default();
                let texts_cls = if badge.is_empty() {
                    "flex flex-col min-w-0".to_string()
                } else {
                    format!("flex flex-col min-w-0 {}", nc::TEXTS_FILL)
                };
                content.push_str(&format!("{}<span class=\"{}\">\n", cind, texts_cls));
                if has_label {
                    if let Some(value) = props.get("label") {
                        let frag = match value {
                            AuraPropValue::Expr(expr) => {
                                format!(
                                    "{{{{{}}}}}",
                                    self.bound_value_or_warn(expr, "nav-item label", "''")
                                )
                            }
                            _ => self.prop_to_text_content(value)?,
                        };
                        content.push_str(&format!(
                            "{}  <span class=\"nav-name truncate\">{}</span>\n",
                            cind, frag
                        ));
                    }
                }
                if has_desc {
                    if let Some(value) = props.get("desc") {
                        let frag = match value {
                            AuraPropValue::Expr(expr) => {
                                format!(
                                    "{{{{{}}}}}",
                                    self.bound_value_or_warn(expr, "nav-item desc", "''")
                                )
                            }
                            _ => self.prop_to_text_content(value)?,
                        };
                        content.push_str(&format!(
                            "{}  <span class=\"{} truncate\">{}</span>\n",
                            cind,
                            nc::TEXT_DESC,
                            frag
                        ));
                    }
                }
                content.push_str(&format!("{}</span>\n", cind));
            }
            if let Some(value) = props.get("badge") {
                let frag = match value {
                    AuraPropValue::Expr(expr) => {
                        format!(
                            "{{{{{}}}}}",
                            self.bound_value_or_warn(expr, "nav-item badge", "''")
                        )
                    }
                    _ => self.prop_to_text_content(value)?,
                };
                content.push_str(&format!(
                    "{}<span class=\"{}\">{}</span>\n",
                    cind,
                    nc::BADGE_PILL,
                    frag
                ));
            }
        }

        if content.is_empty() {
            html.push_str(&format!(
                "{}<{} {} />\n",
                ind,
                tag_name,
                inline_attrs.join(" ")
            ));
        } else {
            html.push_str(&format!(
                "{}<{} {}>\n",
                ind,
                tag_name,
                inline_attrs.join(" ")
            ));
            html.push_str(&content);
            html.push_str(&format!("{}</{}>\n", ind, tag_name));
        }
        Ok(html)
    }

    /// Plan 548 T6: `<sidebar_menu_button>` / `<sidebar_menu_sub_button>` ——
    /// shadcn SidebarMenuButton/SidebarMenuSubButton + to/active 扩展臂(D2)。
    ///
    /// 资产 props 真名(crates/auto-ui-tools/assets/shadcn-ui/sidebar/ — PLAN-815 T-04 迁,
    /// SidebarMenuButtonChild.vue / SidebarMenuSubButton.vue 的 defineProps):
    /// variant/size/isActive/asChild(class 之外);isActive 在模板发 kebab 形
    /// `:is-active`,asChild 发 `as-child`。
    ///
    /// - `to:` → as-child 多态:RouterLink **内嵌**(button 套 a 是非法
    ///   HTML),静态 `to="..."` / 绑定 `:to="..."`;置位 needs_router。
    /// - `active` → `:is-active`;`to` 存在且 active 缺省时按 $route.path
    ///   自动探测(exact 或前缀段匹配)。$route 由 vue-router 全局注入,
    ///   模板内免 import。
    /// - variant/size/tooltip/text/class/onclick 与通用 shadcn 臂同规透传。
    pub(super) fn generate_sidebar_menu_button_html(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        let comp = if fold_eq(tag, "sidebar_menu_sub_button") {
            "SidebarMenuSubButton"
        } else {
            "SidebarMenuButton"
        };
        self.shadcn_components_used.insert(comp.to_string());

        let mut attrs: Vec<String> = Vec::new();
        // variant / size / tooltip —— 静态或绑定透传(资产 defineProps 真名)。
        for key in ["variant", "size", "tooltip"] {
            if let Some(value) = props.get(key) {
                if let Some(frag) = self.nav_attr_fragment(key, value) {
                    attrs.push(frag);
                }
            }
        }
        // active → :is-active;缺省且带 to 时 $route.path 自动探测。
        let to_value = props.get("to");
        if let Some(value) = props.get("active") {
            match value {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => {
                    attrs.push(format!(":is-active=\"{}\"", b));
                }
                AuraPropValue::Expr(expr) => {
                    let js =
                        self.bound_value_or_warn(expr, "sidebar menu button `active`", "false");
                    attrs.push(format!(":is-active=\"{}\"", js));
                }
                _ => {}
            }
        } else if let Some(to) = to_value {
            let detect = match to {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))
                | AuraPropValue::Expr(auto_lang_core::ast::Expr::CStr(s)) => {
                    let p = Self::escape_js_string(s.as_str());
                    format!(
                        "$route.path === '{0}' || $route.path.startsWith('{0}' + '/')",
                        p
                    )
                }
                AuraPropValue::Expr(expr) => {
                    let js = self.bound_value_or_warn(expr, "sidebar menu button `to`", "''");
                    format!(
                        "$route.path === ({0}) || $route.path.startsWith(({0}) + '/')",
                        js
                    )
                }
                _ => String::new(),
            };
            if !detect.is_empty() {
                attrs.push(format!(":is-active=\"{}\"", detect));
            }
        }
        if let Some(click) = self.nav_click_attr(events) {
            attrs.push(click);
        }
        if let Some(cls) = self.nav_user_class_attr(props) {
            attrs.push(cls);
        }

        // as-child 多态:to 存在(路由模式必须)或用户显式 as_child: true。
        let explicit_as_child = props
            .get("as_child")
            .map(|v| self.extract_bool_value(v))
            .unwrap_or(false);
        let has_to = to_value.is_some();
        let as_child = has_to || explicit_as_child;
        if has_to {
            self.needs_router = true;
        }

        // 内容:children 优先;否则 text prop 作 slot 文本。as-child 路由
        // 模式下内容缩进再进一层(RouterLink 内)。
        let inner_indent = if has_to { indent + 2 } else { indent + 1 };
        let cind = "  ".repeat(inner_indent);
        let mut content = String::new();
        if !children.is_empty() {
            for child in children {
                content.push_str(&self.node_to_html(child, inner_indent)?);
            }
        } else if let Some(value) = props.get("text") {
            content.push_str(&format!("{}{}\n", cind, self.prop_to_text_content(value)?));
        }

        let as_child_str = if as_child { " as-child" } else { "" };
        let attr_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };

        let mut html = String::new();
        if has_to {
            // <Comp as-child ...><RouterLink to="...">content</RouterLink></Comp>
            let to_attr = match to_value {
                Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)))
                | Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::CStr(s))) => {
                    format!("to=\"{}\"", Self::escape_html_attr(s.as_str()))
                }
                Some(value) => self
                    .nav_attr_fragment("to", value)
                    .unwrap_or_else(|| "to=\"\"".to_string()),
                None => unreachable!("has_to checked"),
            };
            let ind1 = "  ".repeat(indent + 1);
            html.push_str(&format!("{}<{}{}{}>\n", ind, comp, as_child_str, attr_str));
            html.push_str(&format!("{}<RouterLink {}>\n", ind1, to_attr));
            html.push_str(&content);
            html.push_str(&format!("{}</RouterLink>\n", ind1));
            html.push_str(&format!("{}</{}>\n", ind, comp));
        } else if content.is_empty() {
            html.push_str(&format!(
                "{}<{}{}{} />\n",
                ind, comp, as_child_str, attr_str
            ));
        } else {
            html.push_str(&format!("{}<{}{}{}>\n", ind, comp, as_child_str, attr_str));
            html.push_str(&content);
            html.push_str(&format!("{}</{}>\n", ind, comp));
        }
        Ok(html)
    }

    /// Plan 482: `<nav-group>` → NavGroup 组件（shadcn）或内联折叠组。
    pub(super) fn generate_nav_group_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        use crate::ui_gen::nav_contract as nc;

        let ind = "  ".repeat(indent);
        let label = props
            .get("label")
            .or_else(|| props.get("text"))
            .and_then(|v| self.extract_string_value(v))
            .unwrap_or_default();
        let collapsible = props
            .get("collapsible")
            .and_then(|v| match v {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => Some(*b),
                _ => None,
            })
            .unwrap_or_else(|| props.contains_key("collapsible"));
        let indent_children = props
            .get("indent")
            .and_then(|v| match v {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);

        let mut html = String::new();
        if self.is_shadcn() {
            self.shadcn_components_used.insert("NavGroup".to_string());
            let mut attrs: Vec<String> = Vec::new();
            if !label.is_empty() {
                attrs.push(format!("label=\"{}\"", Self::escape_html_attr(&label)));
            }
            if collapsible {
                attrs.push(":collapsible=\"true\"".to_string());
            }
            if let Some(value) = props.get("open") {
                if let Some(frag) = self.nav_attr_fragment("open", value) {
                    attrs.push(frag);
                }
            }
            if let Some(event) = events.get("ontoggle").or_else(|| events.get("toggle")) {
                let handler_fn =
                    self.handler_to_function_call_with_params(&event.handler, &event.params);
                let handler_name = self.handler_to_function_call(&event.handler);
                self.used_handlers.insert(handler_name);
                attrs.push(format!("@toggle=\"{}\"", handler_fn));
            }
            if indent_children {
                attrs.push(":indent=\"true\"".to_string());
            }
            if let Some(cls) = self.nav_user_class_attr(props) {
                attrs.push(cls);
            }
            html.push_str(&format!("{}<NavGroup {}>\n", ind, attrs.join(" ")));
            for child in children {
                html.push_str(&self.node_to_html(child, indent + 1)?);
            }
            html.push_str(&format!("{}</NavGroup>\n", ind));
            return Ok(html);
        }

        // 非 shadcn：内联折叠组（open 必须绑定——无局部状态可挂）。
        let content_cls = if indent_children {
            format!("{} {}", nc::GROUP_CONTENT, nc::GROUP_CONTENT_INDENT)
        } else {
            nc::GROUP_CONTENT.to_string()
        };
        if !collapsible {
            html.push_str(&format!("{}<div class=\"nav-group flex flex-col\">\n", ind));
            if !label.is_empty() {
                html.push_str(&format!(
                    "{}  <div class=\"{}\">{}</div>\n",
                    ind,
                    nc::GROUP_LABEL,
                    Self::escape_html_text(&label)
                ));
            }
            html.push_str(&format!("{}  <div class=\"{}\">\n", ind, content_cls));
            for child in children {
                html.push_str(&self.node_to_html(child, indent + 2)?);
            }
            html.push_str(&format!("{}  </div>\n", ind));
            html.push_str(&format!("{}</div>\n", ind));
            return Ok(html);
        }

        let Some(open_value) = props.get("open") else {
            self.warn(
                "R008",
                crate::ui_gen::validators::Severity::Warning,
                "nav-group in non-shadcn mode uses collapsible without a bound `open` — \
                 there is no local state to hold; rendering always-open. Bind `open` (and \
                 `ontoggle`) or enable shadcn mode for built-in fold state."
                    .to_string(),
            );
            html.push_str(&format!("{}<div class=\"nav-group flex flex-col\">\n", ind));
            if !label.is_empty() {
                html.push_str(&format!(
                    "{}  <div class=\"{}\">{}</div>\n",
                    ind,
                    nc::GROUP_LABEL,
                    Self::escape_html_text(&label)
                ));
            }
            html.push_str(&format!("{}  <div class=\"{}\">\n", ind, content_cls));
            for child in children {
                html.push_str(&self.node_to_html(child, indent + 2)?);
            }
            html.push_str(&format!("{}  </div>\n", ind));
            html.push_str(&format!("{}</div>\n", ind));
            return Ok(html);
        };
        let open_js = match open_value {
            AuraPropValue::Expr(expr) => self.bound_value_or_warn(expr, "nav-group open", "true"),
            _ => "true".to_string(),
        };
        let mut toggle_attr = String::new();
        if let Some(event) = events.get("ontoggle").or_else(|| events.get("toggle")) {
            let handler_fn =
                self.handler_to_function_call_with_params(&event.handler, &event.params);
            let handler_name = self.handler_to_function_call(&event.handler);
            self.used_handlers.insert(handler_name);
            toggle_attr = format!(" @click=\"{}\"", handler_fn);
        }
        let chevron_cls = "text-[11px] text-muted-foreground w-[14px] shrink-0";
        html.push_str(&format!("{}<div class=\"nav-group flex flex-col\">\n", ind));
        html.push_str(&format!(
            "{}  <button type=\"button\" class=\"{} {}\"{}>\n",
            ind,
            nc::GROUP_TOGGLE,
            nc::GROUP_TOGGLE_HOVER,
            toggle_attr
        ));
        html.push_str(&format!(
            "{}    <span v-if=\"{}\" class=\"{}\">▾</span>\n",
            ind, open_js, chevron_cls
        ));
        html.push_str(&format!(
            "{}    <span v-else class=\"{}\">▸</span>\n",
            ind, chevron_cls
        ));
        html.push_str(&format!(
            "{}    <span class=\"truncate\">{}</span>\n",
            ind,
            Self::escape_html_text(&label)
        ));
        html.push_str(&format!("{}  </button>\n", ind));
        html.push_str(&format!(
            "{}  <div v-show=\"{}\" class=\"{}\">\n",
            ind, open_js, content_cls
        ));
        for child in children {
            html.push_str(&self.node_to_html(child, indent + 2)?);
        }
        html.push_str(&format!("{}  </div>\n", ind));
        html.push_str(&format!("{}</div>\n", ind));
        Ok(html)
    }

    /// Plan 482: `<nav (search: true, ...)>` 容器 —— 顶部集成搜索行 + 子节点。
    /// 仅 search 开启时拦截；普通 nav 走通用容器路径。
    pub(super) fn generate_nav_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        use crate::ui_gen::nav_contract as nc;

        let ind = "  ".repeat(indent);
        let placeholder = props
            .get("search_placeholder")
            .and_then(|v| self.extract_string_value(v))
            .unwrap_or("Search...");

        let mut html = String::new();
        let mut nav_attrs: Vec<String> = Vec::new();
        if let Some(cls) = self.nav_user_class_attr(props) {
            nav_attrs.push(cls);
        }
        let attr_str = if nav_attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", nav_attrs.join(" "))
        };
        html.push_str(&format!("{}<nav{}>\n", ind, attr_str));

        // 搜索行：icon + input（:value 绑定 + onsearch 处理器）。
        html.push_str(&format!("{}  <div class=\"{}\">\n", ind, nc::SEARCH_ROW));
        if self.is_shadcn() {
            self.lucide_icons.insert("Search".to_string());
            html.push_str(&format!(
                "{}    <Search class=\"{} text-muted-foreground shrink-0\" />\n",
                ind,
                nc::ICON_MD
            ));
        } else {
            // 非 shadcn 轨：🔍 文本字形——与 VM 端同构，免 lucide 依赖。
            html.push_str(&format!(
                "{}    <span class=\"{} text-muted-foreground shrink-0\">🔍</span>\n",
                ind,
                nc::ICON_MD
            ));
        }
        let mut input_attrs: Vec<String> = vec![format!("class=\"{}\"", nc::SEARCH_INPUT)];
        // Plan 482 修正：search_value 可能是子组件 prop（如 NavTree(search:)），
        // v-model 写 prop 编译报错——统一发 :value，状态更新交给 onsearch
        // handler（携带输入文本）回写。
        if let Some(value) = props.get("search_value") {
            if let Some(state_ref) = self.extract_state_ref(value) {
                input_attrs.push(format!(":value=\"{}\"", state_ref));
            } else if let AuraPropValue::Expr(expr) = value {
                let js = self.bound_value_or_warn(expr, "nav search_value", "''");
                input_attrs.push(format!(":value=\"{}\"", js));
            }
        }
        input_attrs.push(format!(
            "placeholder=\"{}\"",
            Self::escape_html_attr(&placeholder)
        ));
        if let Some(event) = events.get("onsearch") {
            let handler_name = self.handler_to_function_call(&event.handler);
            let wants_text = self.input_text_handler_wants_text_arg(event);
            let handler_fn = if wants_text {
                format!(
                    "{}(($event.target as HTMLInputElement).value)",
                    handler_name
                )
            } else {
                self.handler_to_function_call_with_params(&event.handler, &event.params)
            };
            self.used_handlers.insert(handler_name);
            input_attrs.push(format!("@input=\"{}\"", handler_fn));
        }
        html.push_str(&format!("{}    <input {} />\n", ind, input_attrs.join(" ")));
        html.push_str(&format!("{}  </div>\n", ind));

        for child in children {
            html.push_str(&self.node_to_html(child, indent + 1)?);
        }
        html.push_str(&format!("{}</nav>\n", ind));
        Ok(html)
    }

    pub(super) fn generate_category_section_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        let name = props
            .get("name")
            .and_then(|v| self.extract_string_value(v))
            .unwrap_or("Category");
        let color = props
            .get("color")
            .and_then(|v| self.extract_string_value(v))
            .unwrap_or("gray");
        let count = props
            .get("count")
            .and_then(|v| self.extract_int_value(v).map(|n| n.to_string()))
            .or_else(|| {
                props
                    .get("count")
                    .and_then(|v| self.extract_string_value(v))
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();

        let (item_classes, dot_class) = Self::category_color_classes(color);
        self.lucide_icons.insert("ArrowRight".to_string());

        let mut html = String::new();
        html.push_str(&format!("{}<div>\n", ind));
        html.push_str(&format!(
            "{}  <div class=\"flex items-center gap-2 mb-4\">\n",
            ind
        ));
        html.push_str(&format!(
            "{}    <span class=\"h-2.5 w-2.5 rounded-full {}\" />\n",
            ind, dot_class
        ));
        html.push_str(&format!("{}    <h2 class=\"m-0 text-sm font-semibold uppercase tracking-wider text-muted-foreground\">{}</h2>\n", ind, name));
        html.push_str(&format!(
            "{}    <span class=\"text-xs text-muted-foreground/60\">({})</span>\n",
            ind, count
        ));
        html.push_str(&format!("{}  </div>\n", ind));
        html.push_str(&format!("{}  <div class=\"grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3\">\n", ind));

        let has_search = self.state_names.contains(&"searchQuery".to_string());

        for child in children {
            if let AuraNode::Element {
                tag: child_tag,
                props: child_props,
                ..
            } = child
            {
                if child_tag == "component-card"
                    || child_tag == "component_card"
                    || child_tag == "componentcard"
                {
                    let to = child_props
                        .get("to")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or("#");
                    let card_name = child_props
                        .get("name")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or("");
                    let desc = child_props
                        .get("desc")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or("");
                    let icon_name = child_props
                        .get("icon")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or("");
                    let lucide_component = Self::lucide_component_name(icon_name);
                    self.lucide_icons.insert(lucide_component.clone());

                    let vshow = if has_search {
                        format!(
                            r#" v-show="!searchQuery || '{}'.toLowerCase().includes(searchQuery.toLowerCase()) || '{}'.toLowerCase().includes(searchQuery.toLowerCase())""#,
                            card_name, desc
                        )
                    } else {
                        String::new()
                    };

                    html.push_str(&format!(
                        r#"{}    <router-link to="{}"{} class="group flex items-start gap-3 rounded-xl border p-4 text-left transition-all duration-200 hover:shadow-md hover:-translate-y-0.5 hover:border-primary/30 bg-card">
{}      <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border {}">
{}        <{} class="h-5 w-5" />
{}      </div>
{}      <div class="min-w-0">
{}        <div class="font-medium text-sm truncate">{}</div>
{}        <div class="text-xs text-muted-foreground truncate">{}</div>
{}      </div>
{}      <ArrowRight class="h-4 w-4 ml-auto shrink-0 text-muted-foreground opacity-0 -translate-x-2 transition-all group-hover:opacity-100 group-hover:translate-x-0" />
{}    </router-link>
"#,
                        ind, to, vshow,
                        ind, item_classes,
                        ind, lucide_component,
                        ind,
                        ind,
                        ind, card_name,
                        ind, desc,
                        ind,
                        ind,
                        ind
                    ));
                }
            }
        }

        html.push_str(&format!("{}  </div>\n", ind));
        html.push_str(&format!("{}</div>\n", ind));
        Ok(html)
    }

}
