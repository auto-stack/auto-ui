//! PLAN-014 T-05：事件绑定责任（event_to_message*/resolve_binding_path/mouse_area/canvas/svg）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {
    /// Convert an event handler pattern to a DynamicMessage (no bindings).
    ///
    /// Patterns like ".Inc", "Msg::Inc", or "Inc" are normalized to a
    /// `DynamicMessage::Typed` with the widget name and extracted handler name.
    pub(super) fn event_to_message(&self, handler: &str) -> DynamicMessage {
        self.event_to_message_impl(handler, &Bindings::new())
    }

    /// Convert an AuraEvent to a DynamicMessage with loop variable bindings.
    ///
    /// Resolves event parameters from bindings (e.g., loop variable `i`)
    /// and encodes integer parameters into the event_name using `name:idx`
    /// format (e.g., `"SelectNote:2"`), leveraging the existing indexed event
    /// dispatch in the iced renderer.
    pub(super) fn event_to_message_with(&self, event: &AuraEvent, bindings: &Bindings) -> DynamicMessage {
        let event_name = extract_handler_name(&event.handler).to_string();
        // Resolve each declared parameter. A param can be:
        //   - a loop-variable reference (e.g. `i`, `note.id`) → resolve from
        //     bindings;
        //   - a literal (e.g. `"indigo"`, `42`) → use directly.
        // Previously only binding references were resolved; literals like
        // `onclick: .SetAccent("indigo")` were dropped (param not in bindings
        // → args empty → handler received no arg → no-op / stack mismatch).
        let mut args: Vec<Value> = Vec::with_capacity(event.params.len());
        for param in &event.params {
            // PLAN-712 T-12: `$` 前缀实参（`$0`/`$1`/`$event`…）是**事件期回调
            // 占位**——Vue 侧由生成器替换成真实取值（ui_gen/vue.rs「`$0` 需要
            // 被生成器替换成真实取值」），VM 侧由各回调臂（progress onseek /
            // mouse-area onmousemove / pen）在事件现场把 payload 追加进 args。
            // 此前占位串落 parse_event_param_literal 兜底臂 → Str("$0") 占据
            // args[0]，handler 形参绑到垃圾字面量、真载荷错位到 args[1]——
            // 030 进度条 `onseek: .SeekFraction($0)` 点击恒换算 0（空参 `()`
            // 形态不受影响，故 020 同控件正常）。剥除占位后载荷对位由回调臂
            // 保证，`($0)` 与 `()` 两形态 VM 端同义、与 Vue 生成器对齐。
            if param.trim().starts_with('$') {
                continue;
            }
            if let Some(val) = self.resolve_binding_path(param, bindings) {
                args.push(val);
            } else if let Some(val) = self.parse_event_param_expr(param, bindings) {
                args.push(val);
            } else {
                args.push(parse_event_param_literal(param));
            }
        }
        DynamicMessage::Typed {
            widget_name: self.widget_name.clone(),
            event_name,
            args,
        }
    }

    /// Internal: convert handler string to DynamicMessage (used by event_to_message).
    /// Resolve a dotted binding path like "note.id" from loop variable bindings.
    /// Splits on '.', looks up the root in bindings, then navigates fields on Obj values.
    /// Plan 059 续(§B8):把事件参数文本解析为 Expr 并经视图表达式解析器
    /// 求值。用于前导点路径(.block.id)—— binding 表只有循环变量。
    pub(super) fn parse_event_param_expr(&self, param: &str, bindings: &Bindings) -> Option<Value> {
        // parser 把前导点形式规整为 "this." 前缀(this.block.id)—— 两种都剥。
        let path = if let Some(r) = param.strip_prefix("this.") {
            r
        } else if let Some(r) = param.strip_prefix('.') {
            r
        } else {
            return None;
        };
        if path.is_empty() {
            return None;
        }
        // PLAN-051 C1: 裸单段名(this.text)实测 parse_expr_fragment 不收
        // (fragment 解析面向点路径)——按 Ident 兜底走同一 bindings/computed/
        // state 解析链,.send(.text) 形态的事件实参此前因此落字面量串。
        let expr = auto_lang_core::parser::Parser::parse_expr_fragment(path)
            .unwrap_or_else(|| Expr::Ident(auto_lang_core::ast::Name::from(path)));
        self.resolve_expr_to_value(&expr, bindings)
    }

    pub(super) fn resolve_binding_path(&self, path: &str, bindings: &Bindings) -> Option<Value> {
        let clean_path = path
            .strip_prefix("this.")
            .or_else(|| path.strip_prefix('.'))
            .unwrap_or(path);
        let parts: Vec<&str> = clean_path.split('.').collect();
        if parts.is_empty() {
            return None;
        }
        // First segment: look up in bindings
        let mut val = bindings.get(parts[0])?.clone();
        // Remaining segments: field access on Obj
        for field in &parts[1..] {
            match val {
                Value::Obj(map) => {
                    val = map.get(*field)?;
                }
                Value::Int(id) if id >= 4_000_000 => {
                    let materialized = self.bridge.materialize_obj_ref(&Value::Int(id));
                    if let Value::Obj(map) = materialized {
                        val = map.get(*field)?;
                    } else {
                        return None;
                    }
                }
                Value::VmRef(r) => {
                    let materialized = self.bridge.materialize_obj_ref(&Value::VmRef(r));
                    if let Value::Obj(map) = materialized {
                        val = map.get(*field)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        Some(val)
    }

    pub(super) fn event_to_message_impl(&self, handler: &str, _bindings: &Bindings) -> DynamicMessage {
        let handler_name = extract_handler_name(handler);
        DynamicMessage::Typed {
            widget_name: self.widget_name.clone(),
            event_name: handler_name.to_string(),
            args: vec![],
        }
    }

    // ========================================================================
    // Property extraction helpers
    // ========================================================================

    /// Extract a string property from AuraNode props (no bindings).
    /// Plan 442 A4: `svg` 元素子树(含 path/circle/... 形状子元素)序列化为
    /// SVG 文档,以 `svgdoc:` 前缀经 View::Image 下发——iced renderer 用既有
    /// svg::Handle 缓存路径渲染(单色 currentColor 文档走画时着色,多彩文档
    /// 原样)。此前 svg/path 落 unknown-tag fallback → View::Empty,VM 轨
    /// 完全不渲染(musk-038 T9 canary 对侧的 native canary)。
    /// Plan 499 M2: mouse-area onmousemove 臂 + coords 逻辑幅面解析。
    /// 事件闭包在调用现场把逻辑坐标 (x, y) 追加为 Float 实参(经
    /// encode_payload/decode_payload 管道直达 VM handler 形参);coords
    /// 形如 "560x300"(组件 viewBox 逻辑幅面),缺省 = None(raw px 模式)。
    pub(super) fn mouse_area_move_arm(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> (
        Option<crate::ui::view::PointerMoveHandler<DynamicMessage>>,
        Option<(f32, f32)>,
    ) {
        let on_move = aura_events_get_base(events, "onmousemove").map(|event| {
            let base = self.event_to_message_with(event, bindings);
            crate::ui::view::PointerMoveHandler::new(move |x: f32, y: f32| match &base {
                DynamicMessage::Typed {
                    widget_name,
                    event_name,
                    args,
                } => {
                    let mut new_args = args.clone();
                    // PLAN-043 T9: +1e-3 分数化——auto_val nanbox 整值 float
                    // 实参绑定腐坏绕道（DEBTS 043 引擎债；0.001px 不可见）。
                    new_args.push(Value::Float(x as f64 + 0.001));
                    new_args.push(Value::Float(y as f64 + 0.001));
                    DynamicMessage::Typed {
                        widget_name: widget_name.clone(),
                        event_name: event_name.clone(),
                        args: new_args,
                    }
                }
                other => other.clone(),
            })
        });
        let extent = self
            .extract_string_with(props, "coords", bindings)
            .and_then(|s| parse_coords_extent(&s));
        (on_move, extent)
    }

    /// Plan 563: pen 事件 → PointerMoveHandler(mouse_area_move_arm 的
    /// on_move 同型:调用现场把逻辑坐标 (x, y) 追加为 Float 实参,含
    /// +0.001 分数化——auto_val nanbox 整值 float 实参绑定腐坏绕道)。
    pub(super) fn pen_handler(
        &self,
        event: &AuraEvent,
        bindings: &Bindings,
    ) -> crate::ui::view::PointerMoveHandler<DynamicMessage> {
        let base = self.event_to_message_with(event, bindings);
        crate::ui::view::PointerMoveHandler::new(move |x: f32, y: f32| match &base {
            DynamicMessage::Typed {
                widget_name,
                event_name,
                args,
            } => {
                let mut new_args = args.clone();
                new_args.push(Value::Float(x as f64 + 0.001));
                new_args.push(Value::Float(y as f64 + 0.001));
                DynamicMessage::Typed {
                    widget_name: widget_name.clone(),
                    event_name: event_name.clone(),
                    args: new_args,
                }
            }
            other => other.clone(),
        })
    }

    /// Plan 563: canvas 元素 → View::Canvas(tracked/untracked 共用——
    /// 无子元素 leaf,无 text probing 面)。scene prop 绑定场景前缀名,
    /// 帧内读平行字符串双表 `<前缀>_pts` / `<前缀>_meta`(B12 规避形态,
    /// 028 先例)解析为结构化场景快照;缺 pts 表 = 空画布(初始态)。
    /// meta 项/位缺失宽容:默认 #111827/3px/非 eraser。
    pub(super) fn convert_canvas(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let scene = self
            .extract_canvas_scene(props, bindings)
            .unwrap_or_default();
        let logical_extent = self
            .extract_string_with(props, "coords", bindings)
            .and_then(|s| parse_coords_extent(&s));
        let clear = self.extract_string_with(props, "clear", bindings);
        let on_pen_start =
            aura_events_get_base(events, "onpenstart").map(|e| self.pen_handler(e, bindings));
        let on_pen_move =
            aura_events_get_base(events, "onpenmove").map(|e| self.pen_handler(e, bindings));
        let on_pen_end =
            aura_events_get_base(events, "onpenend").map(|e| self.pen_handler(e, bindings));
        // PLAN-661 T-05（R-1）：onhit → ElementHitHandler（命中元素 id
        // 字符串载荷；标签旁路供快照 actions 挂 press）。
        let on_hit = aura_events_get_base(events, "onhit").map(|e| self.hit_handler(e, bindings));
        let style = self.extract_style_with(props, bindings);
        View::Canvas {
            scene,
            logical_extent,
            clear,
            on_pen_start,
            on_pen_move,
            on_pen_end,
            on_hit,
            style,
        }
    }

    /// scene prop(`scene: .strokes`)→ 读 `<name>_pts` / `<name>_meta`
    /// 双表并解析。`.field` 在 parser 层解析为 Dot(Ident("self"|"."),
    /// field),legacy 裸名/点整体 Ident 兼收;其余形态 → None(空画布)。
    /// PLAN-661 T-05：扩读图元三表族 `<name>_nodes` / `<name>_edges` /
    /// `<name>_labels`（平行串表 → 结构化图元；缺表 = 空，strokes-only
    /// 现行为逐字节等价）。
    pub(super) fn extract_canvas_scene(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<crate::ui::view::CanvasScene> {
        let prefix = match props.get("scene")? {
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name)) => {
                name.trim_start_matches('.').to_string()
            }
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Dot(obj, field)) => match obj.as_ref() {
                auto_lang_core::ast::Expr::Ident(base)
                    if base.as_str() == "self" || base.as_str() == "." =>
                {
                    field.to_string()
                }
                _ => return None,
            },
            _ => return None,
        };
        let pts = self.read_state_as_vec(&format!("{prefix}_pts")).ok()?;
        let meta = self
            .read_state_as_vec(&format!("{prefix}_meta"))
            .unwrap_or_default();
        let nodes = self
            .read_state_as_vec(&format!("{prefix}_nodes"))
            .unwrap_or_default();
        let edges = self
            .read_state_as_vec(&format!("{prefix}_edges"))
            .unwrap_or_default();
        let labels = self
            .read_state_as_vec(&format!("{prefix}_labels"))
            .unwrap_or_default();
        let _ = bindings;
        let mut scene = parse_canvas_scene(&pts, &meta);
        scene.nodes = parse_canvas_nodes(&nodes);
        scene.edges = parse_canvas_edges(&edges);
        scene.labels = parse_canvas_labels(&labels);
        Some(scene)
    }

    /// PLAN-661 T-05（R-1）：onhit 事件 → ElementHitHandler——调用现场把
    /// 命中元素 id 作为 Str 实参追加（`.OnNodeTap(id string)` 范式）。
    pub(super) fn hit_handler(
        &self,
        event: &AuraEvent,
        bindings: &Bindings,
    ) -> crate::ui::view::ElementHitHandler<DynamicMessage> {
        let base = self.event_to_message_with(event, bindings);
        let label = extract_handler_name(&event.handler).to_string();
        crate::ui::view::ElementHitHandler::new_labeled(
            move |id: String| match &base {
                DynamicMessage::Typed {
                    widget_name,
                    event_name,
                    args,
                } => {
                    let mut new_args = args.clone();
                    new_args.push(Value::Str(id.into()));
                    DynamicMessage::Typed {
                        widget_name: widget_name.clone(),
                        event_name: event_name.clone(),
                        args: new_args,
                    }
                }
                other => other.clone(),
            },
            &label,
        )
    }

    /// Plan 484: hover 命中区 untracked 臂(convert_mouse_area 的镜像)。
    /// 无 text probing 需求,children 走 untracked 拼接展开。
    pub(super) fn convert_mouse_area_untracked(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let on_enter = aura_events_get_base(events, "onmouseenter")
            .or_else(|| aura_events_get_base(events, "onhover"))
            .map(|event| self.event_to_message_with(event, bindings));
        let on_exit = aura_events_get_base(events, "onmouseleave")
            .or_else(|| aura_events_get_base(events, "onhoverout"))
            .map(|event| self.event_to_message_with(event, bindings));
        // Plan 496 M5: ondblclick → mouse_area on_double_click(桌面图标
        // 双击启动原语;iced on_double_click / vue @dblclick)。
        // Plan 498 M0: onclick → mouse_area on_click(chart legend 点击
        // 切换显隐;iced on_press / vue @click)。
        let on_double_click = aura_events_get_base(events, "ondblclick")
            .map(|event| self.event_to_message_with(event, bindings));
        // PLAN-043 T9: onmousedown 优先占 on_press 槽（真按下语义——
        // CustomScrollbar 拖拽发射面；onclick 保留 Plan 498 近似回退）；
        // onmouseup → on_release（拖拽收尾原语）。
        let on_click = aura_events_get_base(events, "onmousedown")
            .or_else(|| aura_events_get_base(events, "onclick"))
            .map(|event| self.event_to_message_with(event, bindings));
        let on_release = aura_events_get_base(events, "onmouseup")
            .map(|event| self.event_to_message_with(event, bindings));
        // PLAN-526 T10: oncontextmenu → 右键事件（iced on_right_press;
        // 桌面空白/任务栏右键菜单挂点）。
        let on_context_menu = aura_events_get_base(events, "oncontextmenu")
            .map(|event| self.event_to_message_with(event, bindings));
        // Plan 499 M2: onmousemove + coords(镜像臂)。
        let (on_move, logical_extent) = self.mouse_area_move_arm(props, events, bindings);
        let style = self.extract_style_with(props, bindings);
        let child_views: Vec<View<DynamicMessage>> = self
            .expand_children_spliced(children, bindings)
            .into_iter()
            .filter(|v| !is_visually_empty(v))
            .collect();
        let content = if child_views.is_empty() {
            View::Empty
        } else {
            let mut builder = View::<DynamicMessage>::col();
            for child in child_views {
                builder = builder.child(child);
            }
            builder.build()
        };
        View::MouseArea {
            content: Box::new(content),
            on_enter,
            on_exit,
            on_double_click,
            on_click,
            on_context_menu,
            on_release,
            on_move,
            logical_extent,
            style,
        }
    }

    /// Plan 484: hover 命中区 → View::MouseArea。onmouseenter/onmouseleave
    /// 经通用事件解析(event_to_message_with,支持绑定/字面量参数)→
    /// DynamicMessage::Typed → dispatch 进 VM handler。children 走拼接展开
    /// (通常为空,占位尺寸由 style 的 w-[..]/h-[..] 承担)。
    pub(super) fn convert_mouse_area(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let on_enter = aura_events_get_base(events, "onmouseenter")
            .or_else(|| aura_events_get_base(events, "onhover"))
            .map(|event| self.event_to_message_with(event, bindings));
        let on_exit = aura_events_get_base(events, "onmouseleave")
            .or_else(|| aura_events_get_base(events, "onhoverout"))
            .map(|event| self.event_to_message_with(event, bindings));
        // Plan 496 M5: ondblclick → mouse_area on_double_click(untracked 镜像臂)。
        // Plan 498 M0: onclick → mouse_area on_click(untracked 镜像臂)。
        let on_double_click = aura_events_get_base(events, "ondblclick")
            .map(|event| self.event_to_message_with(event, bindings));
        // PLAN-043 T9: onmousedown 优先 / onmouseup → on_release（镜像臂）。
        let on_click = aura_events_get_base(events, "onmousedown")
            .or_else(|| aura_events_get_base(events, "onclick"))
            .map(|event| self.event_to_message_with(event, bindings));
        let on_release = aura_events_get_base(events, "onmouseup")
            .map(|event| self.event_to_message_with(event, bindings));
        // PLAN-526 T10: oncontextmenu → 右键事件（untracked 镜像臂）。
        let on_context_menu = aura_events_get_base(events, "oncontextmenu")
            .map(|event| self.event_to_message_with(event, bindings));
        // Plan 499 M2: onmousemove + coords(限频流臂)。
        let (on_move, logical_extent) = self.mouse_area_move_arm(props, events, bindings);
        let style = self.extract_style_with(props, bindings);
        // PLAN-089(T-07① 根修): mouse-area 子树压子索引——此前 content 与
        // wrapper 同路径展开，build 探针路径与 view 树路径错位(computed
        // class/events 合流到同键 + MCP press 按 wrapper 路径落到错位节
        // 点=FileTree 行 press "No press handler" 的仪器面根因)。push(0)
        // 与 vnode_converter 的 MouseArea→[content] 下探对齐。
        path.push(0);
        let child_views: Vec<View<DynamicMessage>> = self
            .expand_children_spliced_source(children, path, id_map, probe, bindings)
            .into_iter()
            .filter(|v| !is_visually_empty(v))
            .collect();
        path.pop();
        let content = if child_views.is_empty() {
            View::Empty
        } else {
            let mut builder = View::<DynamicMessage>::col();
            for child in child_views {
                builder = builder.child(child);
            }
            builder.build()
        };
        View::MouseArea {
            content: Box::new(content),
            on_enter,
            on_exit,
            on_double_click,
            on_click,
            on_context_menu,
            on_release,
            on_move,
            logical_extent,
            style,
        }
    }

    pub(super) fn convert_svg_image(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // Plan 445 M3：动态 props（`d: .lineD` 等）经 bindings/state 解析。
        let doc = self.serialize_svg_element("svg", props, children, bindings);
        // 类串双键取值:`class:` 与 `style:` 都是 .at 元素的合法写法
        // (extract 按属性名原样入 props,svg 走 props 通道无规范化)。
        // 此前只认 "class",`svg (style: "...")` 形态的整条类串丢失——
        // 尺寸类缺失 → svgdoc container 无界撑满行(auto-os-config 概要页
        // Uptime 行图标/文字分离、Memory donut 尺寸失控的根因)。
        let style = self
            .extract_string(props, "class")
            .or_else(|| self.extract_string(props, "style"))
            .and_then(|c| crate::ui::style::Style::parse(&c).ok());
        View::Image {
            src: format!("svgdoc:{doc}"),
            style,
        }
    }

    /// Plan 442 A4: SVG 元素树 → SVG 文档字符串。props 经表达式解析器取值
    /// （字面量行为不变；动态 props 见 Plan 445 M3——此前仅字面量透传，
    /// A4 消费面是静态图标数据）;class/style 不进文档(class 已由外层
    /// View::Image 承载);非 SVG 子元素忽略。
    pub(super) fn serialize_svg_element(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> String {
        let escape = |v: &str| {
            v.replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        };
        let mut attrs = String::new();
        for (key, value) in props {
            if key == "class" || key == "style" {
                continue;
            }
            // Plan 502 M1: svg `<text>` 的位置参数文本是元素内容不是属性
            // (`text seg["label"] { x: … }` 的 positional → props["text"])。
            if tag == "text" && key == "text" {
                continue;
            }
            if let AuraPropValue::Expr(expr) = value {
                let resolved = self.resolve_expr_to_string_with(expr, bindings);
                if !resolved.is_empty() {
                    attrs.push_str(&format!(" {}=\"{}\"", key, escape(&resolved)));
                }
            }
        }
        // Plan 502 M1: text 元素内容 = props["text"](resvg 原生栅格化)。
        let text_content = if tag == "text" {
            props
                .get("text")
                .and_then(|v| match v {
                    AuraPropValue::Expr(expr) => {
                        Some(self.resolve_expr_to_string_with(expr, bindings))
                    }
                    AuraPropValue::StyleBinding(_) => None,
                })
                .map(|s| escape(&s))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let mut inner = self.serialize_svg_children(children, bindings);
        if !text_content.is_empty() {
            inner.push_str(&text_content);
        }
        if inner.is_empty() {
            format!("<{tag}{attrs}/>")
        } else {
            format!("<{tag}{attrs}>{inner}</{tag}>")
        }
    }

    /// Plan 445 后续修复：svg 子树的子节点序列化——Element 直出，
    /// **Conditional 先评估条件取真/假分支再递归**（此前被静默跳过，
    /// `if .dVisible { path … }` 一类条件包裹的数据 path 整体不出现在
    /// svgdoc 文档 → VM 轨 bar/line/area 图表区无数据，只有字面量
    /// 网格/轴线；runtime SVGDOC 打印实锤）。
    pub(super) fn serialize_svg_children(&self, children: &[AuraNode], bindings: &Bindings) -> String {
        let mut inner = String::new();
        for child in children {
            match child {
                AuraNode::Element {
                    tag: ctag,
                    props: cprops,
                    children: cchildren,
                    ..
                } => {
                    if is_svg_shape_tag(ctag) {
                        inner.push_str(
                            &self.serialize_svg_element(ctag, cprops, cchildren, bindings),
                        );
                    }
                }
                AuraNode::Conditional {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    let is_true = self.eval_condition_with(condition, bindings);
                    let empty = Vec::new();
                    let body = if is_true {
                        then_body
                    } else {
                        else_body.as_ref().unwrap_or(&empty)
                    };
                    inner.push_str(&self.serialize_svg_children(body, bindings));
                }
                // Plan 437 Phase 2:for 循环展开 —— Conditional 修复(Plan 445
                // 后续修复)的同族缺口。症状相同:循环包裹的数据 path 整体
                // 不出现在 svgdoc(组件化 chart 的 seg 循环全空)。迭代语义
                // 与主转换器 ForLoop 臂一致(bindings → heap-id/VmRef 列表 →
                // read_state_as_vec),逐项绑定 var 后递归序列化循环体。
                AuraNode::ForLoop {
                    var,
                    index,
                    iterable,
                    body,
                    ..
                } => {
                    let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
                    let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
                    let items: Vec<Value> =
                        if stripped.contains('.') && !stripped.starts_with("store.") {
                            self.resolve_iterable(iterable, bindings)
                                .unwrap_or_default()
                        } else if let Some(val) = bindings.get(state_name).cloned() {
                            match val {
                                auto_val::Value::Array(arr) => arr.iter().cloned().collect(),
                                auto_val::Value::Int(id) if id >= 4_000_000 => {
                                    self.bridge.index_list_all(id as usize)
                                }
                                auto_val::Value::VmRef(r) => self.bridge.index_list_all(r.id),
                                _ => Vec::new(),
                            }
                        } else {
                            self.read_state_as_vec(state_name).unwrap_or_default()
                        };
                    for (i, item) in items.iter().enumerate() {
                        let mut loop_bindings = bindings.clone();
                        loop_bindings.insert(var.clone(), self.bridge.materialize_obj_ref(item));
                        if let Some(idx_var) = index {
                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                        }
                        inner.push_str(&self.serialize_svg_children(body, &loop_bindings));
                    }
                }
                _ => {}
            }
        }
        inner
    }

}
