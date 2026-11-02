//! PLAN-014 T-05：prop 提取责任（extract_*/prop 家族）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

/// PLAN-031 T-03（V-04）：元素 design 帧 guard——Drop 弹出本元素 frame；
/// 嵌套构建、转换失败与兄弟分支都恢复上一作用域。
pub(super) struct DesignScopeGuard<'a> {
    frames: &'a RefCell<
        Vec<Option<std::collections::BTreeMap<String, auto_lang_core::ast::Expr>>>,
    >,
}

impl Drop for DesignScopeGuard<'_> {
    fn drop(&mut self) {
        self.frames.borrow_mut().pop();
    }
}

impl<'a> AuraViewBuilder<'a> {
    /// 进入一个元素：带 `design:` 对象则合成（最近祖先 frame + 本元素同路径
    /// 覆盖）并入栈；无 design 压入 None——祖先局部值不会绘制到未声明的子件，
    /// 兄弟/另一实例各持自己的 frame（V-04）。
    pub(super) fn enter_element_design(
        &self,
        props: &HashMap<String, AuraPropValue>,
    ) -> DesignScopeGuard<'_> {
        use auto_lang_core::ast::Expr;
        let own: Option<Vec<(String, Expr)>> = match props.get("design") {
            Some(AuraPropValue::Expr(Expr::Object(pairs))) => Some(
                pairs
                    .iter()
                    .map(|p| (Self::design_key_text(&p.key), p.value.as_ref().clone()))
                    .collect(),
            ),
            Some(_) => {
                eprintln!("[PLAN-031] design 必须是对象字面量——本元素局部设计跳过");
                None
            }
            None => None,
        };
        let frame = own.map(|pairs| {
            let mut base: std::collections::BTreeMap<String, Expr> = self
                .design_frames
                .borrow()
                .iter()
                .rev()
                .find_map(|f| f.clone())
                .unwrap_or_default();
            for (k, v) in pairs {
                base.insert(k, v);
            }
            base
        });
        self.design_frames.borrow_mut().push(frame);
        DesignScopeGuard {
            frames: &self.design_frames,
        }
    }

    /// design 对象键 → 槽名（AST Key 的 Display 带 `(name …)` 包装，不能直接用）。
fn design_key_text(key: &auto_parser_api::ast::Key) -> String {
    use auto_parser_api::ast::Key;
    match key {
        Key::NamedKey(name) => name.to_string(),
        Key::StrKey(s) => s.to_string(),
        Key::IntKey(i) => i.to_string(),
        Key::BoolKey(b) => b.to_string(),
    }
}

/// 本元素（且仅本元素声明了 design）的合成 frame；未声明返回 None。
    pub(super) fn current_design_frame(
        &self,
    ) -> Option<std::collections::BTreeMap<String, auto_lang_core::ast::Expr>> {
        self.design_frames.borrow().last().cloned().flatten()
    }
}

impl<'a> AuraViewBuilder<'a> {
    pub(super) fn extract_string(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<String> {
        self.extract_string_with(props, key, &Bindings::new())
    }

    /// Extract a string property with loop variable bindings support.
    /// Extract text content from child nodes (for elements like button whose
    /// label comes from inner `text` children rather than a primary prop).
    /// Walks children recursively, resolving each text element's "text"/literal
    /// content with the given bindings, and joins them with newlines. Returns
    /// None if no text found. Recurses into container elements (row/col/etc.)
    /// so that a button like:
    ///   button { row { text note.title } text note.time }
    /// yields "Welcome\nJust now" (newline-separated), so the iced renderer
    /// can render title and time on separate lines with different styling.
    pub(super) fn extract_children_text(&self, children: &[AuraNode], bindings: &Bindings) -> Option<String> {
        let parts: Vec<String> = children
            .iter()
            .filter_map(|c| match c {
                AuraNode::Element { tag, props, .. }
                    if matches!(
                        tag.as_str(),
                        "text" | "label" | "h1" | "h2" | "h3" | "p" | "span"
                    ) =>
                {
                    self.extract_string_with(props, "text", bindings)
                        .or_else(|| self.extract_string_with(props, "label", bindings))
                }
                AuraNode::Element { tag, children, .. }
                    if matches!(
                        tag.as_str(),
                        "row" | "col" | "column" | "container" | "scrollable" | "grid"
                    ) =>
                {
                    // Recurse into layout containers to find nested text.
                    self.extract_children_text(children, bindings)
                }
                AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                AuraNode::Text(AuraTextContent::Interpolated {
                    template,
                    bindings: tpl_bindings,
                }) => Some(self.resolve_interpolation_with(template, tpl_bindings, bindings)),
                _ => None,
            })
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    pub(super) fn extract_string_with(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<String> {
        let prop = props.get(key)?;
        match prop {
            AuraPropValue::Expr(expr) => {
                let result = self.resolve_expr_to_string_with(expr, bindings);
                Some(result)
            }
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// PLAN-055: div 的 `html:` prop（v-html）VM 轨降级取值——已求值字符串
    /// 去除 HTML 标签并解码常见实体，落纯文本。仅消费已求值 props（绑定
    /// 期字符串），不进入 eval/call 链（textarea mentions 的 UAF 同构雷区
    /// 规避口径，见 493 注记）。None = 无 html prop（普通 div，行为不变）。
    pub(super) fn div_html_stripped_text(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<String> {
        let prop = props.get("html")?;
        let AuraPropValue::Expr(expr) = prop else {
            return None;
        };
        // `.html` 一类 computed 引用经 resolve_expr_to_value 求值（有 computed
        // 表兜底）；字符串字面量（可能带 ${} 插值）走 string resolver。
        // 直接用 resolve_expr_to_string_with 会把未解析 Dot 格式化成
        // "${html}" 字面量（P055 实机首验现场）。
        let raw = match self.resolve_expr_to_value(expr, bindings) {
            Some(Value::Str(s)) => s.to_string(),
            Some(other) => value_to_display_string(&other),
            None => {
                // resolve_expr_to_value 不查 computed 表——`.html` 落 None。
                // 对裸 computed/Dot 引用补 computed 表兜底（与条件求值同口径）。
                let computed_name = match expr {
                    Expr::Dot(obj, field) if matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self") => {
                        Some(field.as_str())
                    }
                    Expr::Ident(n) => Some(n.as_str().trim_start_matches('.')),
                    _ => None,
                };
                match computed_name.and_then(|name| self.eval_computed(name, bindings)) {
                    Some(Value::Str(s)) => s.to_string(),
                    // computed 值非 Str（如 render_mentions_default 经 regex
                    // native 返回位型 Int，实机显示 "203200"）——回落子组件
                    // state 的 content prop 原始文本（v-html-of-content 形态；
                    // 子渲染期 state root 已切到子组件，read_state 直达）。
                    _ => match self.read_state("content") {
                        Ok(Value::Str(s)) => s.to_string(),
                        _ => self.resolve_expr_to_string_with(expr, bindings),
                    },
                }
            }
        };
        let stripped = strip_html_tags(&raw);
        if stripped.trim().is_empty() {
            None
        } else {
            Some(stripped)
        }
    }

    /// Extract a u16 property from AuraNode props.
    /// PLAN-018 D10:int(scheme)字面量提取(负值合法:哨兵 -1 跟随主题)。
    pub(super) fn extract_i32_prop(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<i32> {
        match props.get(key)? {
            AuraPropValue::Expr(expr) => match expr {
                Expr::Int(i) => Some(*i),
                _ => None,
            },
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// PLAN-018 D10:scheme 的 bindings 求值形态(`scheme: .scheme`)。
    pub(super) fn eval_i32_prop(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<i32> {
        let AuraPropValue::Expr(expr) = props.get(key)? else {
            return None;
        };
        let val = self.resolve_expr_to_value(expr, bindings)?;
        Some(val.as_int())
    }

    pub(super) fn extract_u16(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<u16> {
        match props.get(key)? {
            AuraPropValue::Expr(expr) => match expr {
                Expr::Int(i) => {
                    if *i >= 0 && *i <= u16::MAX as i32 {
                        Some(*i as u16)
                    } else {
                        None
                    }
                }
                Expr::Float(f, _) | Expr::Double(f, _) => {
                    if *f >= 0.0 && *f <= u16::MAX as f64 {
                        Some(*f as u16)
                    } else {
                        None
                    }
                }
                // 绑定/变量表达式（icon { size: .size } 的 widget 参数引用）
                // 经 resolve 求值取整——原先非字面量恒 None，size 恒落默认值
                // （musk 工具栏图标传 12/14/16 全部无效的根因，r8 实测定罪）。
                _ => None,
            },
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// Plan 448 I: dynamic grid `cols:`/`gap:` — a non-literal expression
    /// evaluates per view rebuild (state changes re-run the build, the same
    /// cycle style props ride). Literals stay on `extract_u16` (no eval
    /// churn); eval failure or out-of-range falls through to the caller's
    /// default (cols 1 / gap 0) instead of clamping garbage.
    pub(super) fn eval_u16_prop(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<u16> {
        let AuraPropValue::Expr(expr) = props.get(key)? else {
            return None;
        };
        match expr {
            Expr::Int(_) | Expr::Float(..) | Expr::Double(..) => None,
            other => match self.resolve_expr_to_value(other, bindings) {
                Some(Value::Int(i)) if i >= 0 && i <= u16::MAX as i32 => Some(i as u16),
                Some(Value::Float(f)) if f >= 0.0 && f <= u16::MAX as f64 => Some(f as u16),
                _ => None,
            },
        }
    }

    /// Extract a bool property from AuraNode props.
    pub(super) fn extract_bool(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<bool> {
        match props.get(key)? {
            AuraPropValue::Expr(Expr::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Extract a float property from AuraNode props (supports StateRef resolution).
    pub(super) fn extract_f64(&self, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<f64> {
        let empty_bindings = HashMap::new();
        self.extract_f64_with(props, key, &empty_bindings)
    }

    /// Extract a float property from AuraNode props with loop bindings.
    pub(super) fn extract_f64_with(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<f64> {
        match props.get(key)? {
            AuraPropValue::Expr(expr) => match expr {
                Expr::Int(i) => Some(*i as f64),
                Expr::Float(f, _) | Expr::Double(f, _) => Some(*f),
                _ => {
                    let val = self.resolve_expr_to_value(expr, bindings)?;
                    match val {
                        Value::Int(i) => Some(i as f64),
                        Value::Float(f) => Some(f as f64),
                        Value::Double(f) => Some(f),
                        _ => None,
                    }
                }
            },
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// Extract a style property from AuraNode props.
    ///
    /// Looks for a "class" or "style" prop and parses it into a Style object.
    pub(super) fn extract_style(&self, props: &HashMap<String, AuraPropValue>) -> Option<Style> {
        let style_str = self
            .extract_string(props, "class")
            .or_else(|| self.extract_string(props, "style"))?;

        Style::parse(&style_str).ok()
    }

    /// Plan 057 (2.5): binding-aware style extraction for container converts
    /// (row/col/scroll/container/input/textarea). Mirrors the text-element
    /// pattern (:1511): dynamic exprs (ternaries, string concat like
    /// "grid grid-cols-" + n) resolve against loop bindings/state first;
    /// falls back to the static extract_style. Without this, dynamic styles
    /// on containers silently resolve to empty (iced 只读静态串的根因).
    pub(super) fn extract_style_with(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<Style> {
        let mut style = self
            .extract_string_with(props, "class", bindings)
            .or_else(|| self.extract_string_with(props, "style", bindings))
            .and_then(|s| Style::parse(&s).ok())
            .or_else(|| self.extract_style(props));
        // PLAN-031 T-03（V-04）：元素级 `design:` 局部作用域的**本元素**合成
        // 值（祖先未覆盖字段继承 + 本元素覆盖）；无 design 的元素不应用任何
        // 局部值。槽引用在消费时按当前 scope 主题+模式解析，字面量原样生效。
        self.merge_current_design(style, bindings)
    }

    /// 把**本元素**声明的 design 合成值并入已有样式（无声明原样返回）。
    /// 供不走 `extract_style_with` 的控件臂（input/textarea 的 preset 路径）
    /// 复用，保证 `design { placeholder/radius/... }` 在控件上同样生效。
    pub(super) fn merge_current_design(
        &self,
        style: Option<Style>,
        bindings: &Bindings,
    ) -> Option<Style> {
        let Some(frame) = self.current_design_frame() else {
            return style;
        };
        let extra = apply_design_values(&frame, bindings);
        if extra.is_empty() {
            return style;
        }
        let base = style.unwrap_or_else(|| Style {
            classes: Vec::new(),
            hover_classes: Vec::new(),
            variant_classes: Vec::new(),
        });
        Some(merge_design_classes(base, extra))
    }
}

/// PLAN-031 T-03/T-05：design 合成 frame → 结构类。
///
/// 支持面（§5.3 冻结）：`radius` 长度 + 颜色槽 `background`/`foreground`/`placeholder`/
/// `border`/`ring`（focus 环）。未在支持面内的槽只跳过该槽并给可定位诊断，
/// 父级有效值保持（失败保旧有效子树）——不静默吞掉，也不擅自扩成通用
/// 样式语言。
fn apply_design_values(
    frame: &std::collections::BTreeMap<String, auto_lang_core::ast::Expr>,
    bindings: &Bindings,
) -> Vec<auto_ui_compiler::style::class::StyleClass> {
    use auto_ui_compiler::style::class::{RoundedSize, StyleClass};
    use auto_ui_compiler::style::color::Color;
    let mut out = Vec::new();
    for (key, expr) in frame {
        let resolved = resolve_design_value(expr, bindings);
        let Some(text) = resolved else {
            eprintln!("[PLAN-031] design.{key} 槽引用解析失败——本槽跳过（旧有效样式保留）");
            continue;
        };
        // 颜色槽共用解析：`#rrggbb`/`#rgb`/具名/槽值 → Color::Rgba。
        let color = |slot: &str| -> Option<Color> {
            match auto_ui_compiler::design_tokens::theme_tree::parse_color(&text) {
                Some((r, g, b)) => Some(Color::Rgba { r, g, b, a: 255 }),
                None => {
                    eprintln!(
                        "[PLAN-031] design.{slot} 值 `{text}` 非颜色字面量——本槽跳过（旧有效样式保留）"
                    );
                    None
                }
            }
        };
        match key.as_str() {
            "radius" | "corner-radius" | "rounded" => {
                let n = text.trim_end_matches("dp").trim_end_matches("px").trim();
                match n.parse::<f32>() {
                    // 任意值四角同径 = RoundedT(Px) + RoundedB(Px) 对
                    //（adapter：T 设上角对、B 设下角对）。
                    Ok(v) if v >= 0.0 => {
                        out.push(StyleClass::RoundedT(Some(RoundedSize::Px(v))));
                        out.push(StyleClass::RoundedB(Some(RoundedSize::Px(v))));
                    }
                    _ => eprintln!(
                        "[PLAN-031] design.radius 值 `{text}` 非非负长度字面量——本槽跳过（旧有效圆角保留）"
                    ),
                }
            }
            "background" | "bg" => {
                if let Some(c) = color("background") {
                    out.push(StyleClass::BackgroundColor(c));
                }
            }
            "foreground" | "text" | "color" => {
                if let Some(c) = color("foreground") {
                    out.push(StyleClass::TextColor(c));
                }
            }
            "border" => {
                if let Some(c) = color("border") {
                    out.push(StyleClass::BorderColor(c));
                }
            }
            "placeholder" | "placeholder-color" => {
                if let Some(c) = color("placeholder") {
                    out.push(StyleClass::PlaceholderColor(c));
                }
            }
            "ring" | "focus-ring" | "focus_ring" => {
                if let Some(c) = color("ring") {
                    out.push(StyleClass::RingColor(c));
                    // 槽只给颜色；焦点环宽度按基础控件契约取 2px（不新增语法）。
                    out.push(StyleClass::RingWidth(2.0));
                }
            }
            other => {
                eprintln!(
                    "[PLAN-031] design 槽 `{other}` 在冻结支持面外（radius/background/\
                     foreground/border/ring/placeholder）"
                )
            }
        }
    }
    out
}

/// design 值解析：点链/裸名 → 当前主题+模式的槽值；字面量 → 原样。
fn resolve_design_value(
    expr: &auto_lang_core::ast::Expr,
    bindings: &Bindings,
) -> Option<String> {
    use auto_ui_compiler::design_tokens::theme_tree;
    match expr {
        auto_lang_core::ast::Expr::Str(s) => Some(s.to_string()),
        auto_lang_core::ast::Expr::Int(i) => Some(i.to_string()),
        auto_lang_core::ast::Expr::Float(v, _) | auto_lang_core::ast::Expr::Double(v, _) => {
            Some(v.to_string())
        }
        auto_lang_core::ast::Expr::Dot(_, _) | auto_lang_core::ast::Expr::Ident(_) => {
            let segs = flatten_ref_expr(expr)?;
            let is_dark = auto_ui_compiler::style::theme::dark_mode();
            let tree = auto_ui_compiler::style::theme::active_theme_tree(is_dark)?;
            let path: Vec<String> = segs;
            let value = theme_tree::resolve_slot(&tree, &path).ok()?;
            value.as_str().map(|s| s.to_string())
        }
        _ => None,
    }
}

fn flatten_ref_expr(expr: &auto_lang_core::ast::Expr) -> Option<Vec<String>> {
    match expr {
        auto_lang_core::ast::Expr::Ident(n) => Some(vec![n.to_string()]),
        auto_lang_core::ast::Expr::Dot(root, field) => {
            let mut segs = flatten_ref_expr(root)?;
            segs.push(field.to_string());
            Some(segs)
        }
        _ => None,
    }
}

/// 结构类并入 Style（去重：同类后写胜——每个字段族各保最后一个）。
fn merge_design_classes(
    mut base: Style,
    extra: Vec<auto_ui_compiler::style::class::StyleClass>,
) -> Style {
    use auto_ui_compiler::style::class::StyleClass;
    // 族判定按 extra 实际含有的类，逐族摘掉 base 同族旧值后整体并入——
    // 这样"本元素 design 覆盖 class 里的同类声明"与 Tailwind 的
    // 后写胜一致，同时不误删未涉及的族。
    // 注意 radius 族是 T/B 两个类组成的**一组**（四角同径），必须成对保留
    // ——逐类 retain 会把刚压入的 RoundedT 在压入 RoundedB 时删掉（旧实现
    // 只剩 RoundedB，半径读回为 None）。
    let has_radius = extra
        .iter()
        .any(|c| matches!(c, StyleClass::RoundedT(_) | StyleClass::RoundedB(_)));
    let has_bg = extra
        .iter()
        .any(|c| matches!(c, StyleClass::BackgroundColor(_)));
    let has_fg = extra.iter().any(|c| matches!(c, StyleClass::TextColor(_)));
    let has_border = extra
        .iter()
        .any(|c| matches!(c, StyleClass::BorderColor(_)));
    let has_ring = extra
        .iter()
        .any(|c| matches!(c, StyleClass::RingColor(_)));
    let has_placeholder = extra
        .iter()
        .any(|c| matches!(c, StyleClass::PlaceholderColor(_)));
    base.classes.retain(|c| match c {
        StyleClass::RoundedT(_) | StyleClass::RoundedB(_) => !has_radius,
        StyleClass::BackgroundColor(_) => !has_bg,
        StyleClass::TextColor(_) => !has_fg,
        StyleClass::BorderColor(_) => !has_border,
        StyleClass::RingColor(_) => !has_ring,
        StyleClass::PlaceholderColor(_) => !has_placeholder,
        _ => true,
    });
    base.classes.extend(extra);
    base
}
