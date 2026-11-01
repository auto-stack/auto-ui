//! PLAN-014 T-05：容器类标签责任（column/scroll/row/grid/container/center/thumbnails/媒体/分隔）（自 aura_view_builder.rs impl 块原样收编）。
//! &VmBridge 借用、求值次序、键/路由/挂载与调试身份、guard 恢复零改——
//! 纯 impl 代码移动；方法可见性按跨模块调用需要提升。
use super::*;

impl<'a> AuraViewBuilder<'a> {

    // ========================================================================
    // Layout converters
    // ========================================================================

    /// Convert a column element.
    // Tracked twin: convert_column_tracked_ctx — keep widget logic in sync.
    pub(super) fn convert_column(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self
            .extract_u16(props, "spacing")
            .or_else(|| self.extract_u16(props, "gap").map(|g| g * 4))
            .unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // Plan 476: 拼接感知收集(slot outlet 的 fill 子节点作兄弟展开)。
        let child_views: Vec<View<DynamicMessage>> = self
            .expand_children_spliced(children, bindings)
            .into_iter()
            // Plan 370 (Issue 1): drop visually-empty children (View::Empty
            // or View::Text with blank content). False `if` branches and
            // non-matching `for` iterations yield these, and the renderer
            // turns them into text("") — a one-line-tall spacer. Stacking
            // several produced large blank gaps in the NavTree.
            .filter(|v| !is_visually_empty(v))
            .collect();

        // Plan 409 §10 续 5: 把 position:absolute 的子节点 hoist 出流式布局(用
        // Overlay 悬浮,不挤压布局)。色板的 `row(style:"absolute right-4 ...")` 走这。
        let mut floats: Vec<(View<DynamicMessage>, crate::ui::view::OverlayPosition)> = Vec::new();
        let child_views: Vec<View<DynamicMessage>> = child_views
            .into_iter()
            .filter_map(|v| match extract_absolute_position(&v) {
                Some(pos) => {
                    floats.push((v, pos));
                    None
                }
                None => Some(v),
            })
            .collect();

        // Plan 412 F1: 交叉类仲裁 — col 元素带 `grid grid-cols-N`/`flex` 类时
        // 布局语义被类覆盖(元素语义为默认,显式类优先)。absolute hoist 在前,
        // 浮层不占格。
        let red = rederive_layout(style.as_ref());
        if let Some(RederivedLayout::Grid { cols, gap }) = red {
            let base = View::Grid {
                cols,
                gap,
                cells: child_views,
                style,
            };
            return fold_floats(base, floats);
        }
        let use_row = matches!(red, Some(RederivedLayout::Row));

        let mut builder = if use_row {
            View::<DynamicMessage>::row()
        } else {
            View::<DynamicMessage>::col()
        }
        .spacing(spacing)
        .padding(padding);

        // Plan 048:提取 overflow 标志 + style clone(builder.with_style 会 move style)。
        let needs_scroll = style.as_ref().map_or(false, |s| {
            s.classes.iter().any(|c| {
                matches!(
                    c,
                    crate::ui::style::StyleClass::OverflowYAuto
                        | crate::ui::style::StyleClass::OverflowAuto
                )
            })
        });
        let scroll_style = if needs_scroll { style.clone() } else { None };

        // PLAN-748 T-07：滚动内容 col 拿视觉类剥离后的余集（tracked 双胎
        // 同款，见 split_scroll_visual_classes）。
        match (style, scroll_style.as_ref()) {
            (Some(s), Some(sc)) => {
                let (_, rest) = split_scroll_visual_classes(sc);
                builder = builder.with_style(rest);
            }
            (Some(s), None) => {
                builder = builder.with_style(s);
            }
            _ => {}
        }

        for child in child_views {
            builder = builder.child(child);
        }

        let col_view = builder.build();
        // Plan 048:overflow-y-auto / overflow-auto → Scrollable。
        let base = if needs_scroll {
            let sc = scroll_style.expect("needs_scroll ⇒ Some");
            let (viewport_visual, _) = split_scroll_visual_classes(&sc);
            let scroll_view = View::Scrollable {
                child: Box::new(col_view),
                width: None,
                height: None,
                style: if viewport_visual.is_some() {
                    Some(ensure_full_scroll_dims(&sc))
                } else {
                    Some(sc)
                },
                auto_scroll: false,
                offset: None,
                on_scroll: None,
                axes: crate::ui::scroll::ScrollAxes::Y,
                scrollbar_policy: crate::ui::scroll::ScrollbarPolicy::Auto,
                controller: None,
            };
            match viewport_visual {
                Some(v) => View::Container {
                    child: Box::new(scroll_view),
                    padding: 0,
                    width: None,
                    height: None,
                    center_x: false,
                    center_y: false,
                    style: Some(v),
                    onclick: None,
                    on_right_click: None,
                },
                None => scroll_view,
            }
        } else {
            col_view
        };
        // Plan 409 §10 续 5: 有 absolute 子节点 → 包成 Overlay(浮在 base 上层,不挤压)。
        fold_floats(base, floats)
    }

    /// Convert a scroll element — always scrollable (schema: Scrollable container).
    // Tracked twin: convert_scroll_tracked_ctx — keep widget logic in sync.
    // Plan 409 §10 续 3: scroll 标签之前落 fallback 丢 style(padding/flex/overflow)。
    pub(super) fn convert_scroll(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self
            .extract_u16(props, "spacing")
            .or_else(|| self.extract_u16(props, "gap").map(|g| g * 4))
            .unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // Plan 476: 拼接感知收集 + 视觉空过滤(既有行为)。
        let child_views: Vec<View<DynamicMessage>> = self
            .expand_children_spliced(children, bindings)
            .into_iter()
            .filter(|v| !is_visually_empty(v))
            .collect();

        let mut builder = View::<DynamicMessage>::col()
            .spacing(spacing)
            .padding(padding);
        // style(px-3 py-4 padding + flex-1/h-full)进内层 col;Scrollable 层也带
        // style(flex-1/h-full → build_scrollable 读 Fill 得有界高度才能真正滚动)。
        // F-781-D01 收口：内层 col 的样式经 scroll_content_style 剥除滚动轴
        // 尺寸类（h-32 类 Fixed 高会把内容钳到 viewport→pane 永不可滚动）。
        let scroll_style = style.clone();
        // PLAN-656 T-03: scroll-pane 语义提前——内层 col 的样式剥除需要轴信息。
        let (axes, scrollbar_policy, controller, on_scroll) =
            self.scroll_pane_semantics(props, events, bindings);
        if let Some(s) = style {
            builder = builder.with_style(Self::scroll_content_style(s, axes));
        }
        for child in child_views {
            builder = builder.child(child);
        }
        let col_view = builder.build();
        // Plan 057 续:`auto_scroll: "…"` 标记 → VM 端唯一挂 blocklist_scroll id
        // 的主列表(snap_to_end 目标);块内 max-h 滚动区不再共享同 id。
        let auto_scroll = self.extract_string(props, "auto_scroll").is_some();
        // PLAN-781 T-03: controller pane 的锚点归属后置注入。
        let col_view = match controller.as_ref() {
            Some(c) => Self::inject_anchor_pane(col_view, c.key()),
            None => col_view,
        };
        View::Scrollable {
            child: Box::new(col_view),
            width: None,
            height: None,
            style: scroll_style,
            auto_scroll,
            offset: None,
            on_scroll,
            axes,
            scrollbar_policy,
            controller,
        }
    }

    /// Convert a row element.
    // Tracked twin: convert_row_tracked_ctx — keep widget logic in sync.
    pub(super) fn convert_row(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self
            .extract_u16(props, "spacing")
            .or_else(|| self.extract_u16(props, "gap").map(|g| g * 4))
            .unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // Flatten Conditional children: in a row, multiple condition children
        // should be spread horizontally, not wrapped in a Column.
        // Plan 047:also flatten ForLoop children(for cell in row → cells 横向排列,
        // 而非被 ForLoop 包成单个 Column 导致纵向堆叠)。
        let mut child_views: Vec<View<DynamicMessage>> = Vec::new();
        for n in children {
            match n {
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
                    for child_node in body {
                        child_views.push(self.convert_node_with(child_node, bindings));
                    }
                }
                AuraNode::ForLoop {
                    var,
                    index,
                    iterable,
                    body,
                    ..
                } => {
                    // 同 convert_grid(:1675):用 for_loop_iterations spread 进 row
                    child_views
                        .extend(self.for_loop_iterations(var, index, iterable, body, bindings));
                }
                _ => {
                    // Plan 476: 拼接感知——slot outlet 的 fill 子节点作兄弟展开
                    // (父作用域求值,横向 spread 与本 row 语义一致)。
                    child_views.extend(self.expand_one_child_untracked(n, bindings));
                }
            }
        }

        // PLAN-536 T6(题5 ①②): absolute 悬浮层 hoist——与 convert_column
        // 同规(409 §10 续 5 先例)。row 此前缺臂,行内悬浮元素(会话卡 ×)
        // 退化流内挤压兄弟。
        let mut floats: Vec<(View<DynamicMessage>, crate::ui::view::OverlayPosition)> = Vec::new();
        let child_views: Vec<View<DynamicMessage>> = child_views
            .into_iter()
            .filter_map(|v| match extract_absolute_position(&v) {
                Some(pos) => {
                    floats.push((v, pos));
                    None
                }
                None => Some(v),
            })
            .collect();

        // Plan 412 F1: 交叉类仲裁 — row 元素带 `grid grid-cols-N`/`flex-col` 类时
        // 布局语义被类覆盖。absolute hoist 在前,浮层不占格(409 同款)。
        if let Some(RederivedLayout::Grid { cols, gap }) = rederive_layout(style.as_ref()) {
            let cells: Vec<View<DynamicMessage>> = child_views
                .into_iter()
                .filter(|v| !is_visually_empty(v))
                .collect();
            let base = View::Grid {
                cols,
                gap,
                cells,
                style,
            };
            return fold_floats(base, floats);
        }
        let use_col = matches!(
            rederive_layout(style.as_ref()),
            Some(RederivedLayout::Column)
        );

        let mut builder = if use_col {
            View::<DynamicMessage>::col()
        } else {
            View::<DynamicMessage>::row()
        }
        .spacing(spacing)
        .padding(padding);

        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        for child in child_views {
            // Plan 370 (Issue 1): drop View::Empty spacers (see convert_column).
            if is_visually_empty(&child) {
                continue;
            }
            builder = builder.child(child);
        }

        let row_view = builder.build();
        // PLAN-536 T6: 有 absolute 子节点 → 包成 Overlay(浮在 base 上层)。
        fold_floats(row_view, floats)
    }

    /// Iterate a `for` loop's iterable, converting its body once per item, and
    /// return the resulting views **flat** (one per iteration; multi-node bodies
    /// are wrapped in a Column). Used by `convert_grid` to flatten `for`
    /// children into individual grid cells — a bare `for` inside a grid must
    /// yield one cell per iteration, not a single wrapping Column. (Plan 323.)
    pub(super) fn for_loop_iterations(
        &self,
        var: &str,
        index: &Option<String>,
        iterable: &str,
        body: &[AuraNode],
        bindings: &Bindings,
    ) -> Vec<View<DynamicMessage>> {
        let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
        // Plan 053 后续(ash-gui VM 表头丢失根因):带点路径的 iterable(如
        // view-fn 参数替换后的 "output.columns")必须走 resolve_iterable 的
        // 逐字段解引用 —— 此前只查 bindings/read_state(按整名查字段必然失败)
        // 静默返回空,row 内嵌的 for-loop(表头列名)整体消失。col 路径一直走
        // resolve_iterable,所以数据行(for 在 col 直接子级)不受影响。
        let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
        let store_field = self.store_source_field(stripped);
        let has_inner_dot = stripped.contains('.') && store_field.is_none();
        let stripped = store_field.as_deref().unwrap_or(stripped);
        let state_name = state_name.strip_prefix('.').unwrap_or(state_name);
        let state_name = store_field.as_deref().unwrap_or(state_name);
        let array = if has_inner_dot {
            match self.resolve_iterable(iterable, bindings) {
                Some(elems) => auto_val::Array::from(elems),
                None => return Vec::new(),
            }
        } else if let Some(val) = bindings.get(state_name).cloned() {
            match val {
                Value::Array(arr) => arr,
                Value::Int(id) if id >= 4_000_000 => {
                    auto_val::Array::from(self.bridge.index_list_all(id as usize))
                }
                Value::VmRef(r) => auto_val::Array::from(self.bridge.index_list_all(r.id)),
                _ => return Vec::new(),
            }
        } else {
            // PLAN-051 C3 对齐:state 读 miss → computed 求值回退(resolve_iterable
            // 内置;grid/row 展开路径此前漏齐 —— `for cell in .days` 这类 computed
            // for 源在 grid/row 内展开成 0 cell,Plan 522 016 迁移实证)。
            match self.resolve_iterable(iterable, bindings) {
                Some(elems) => auto_val::Array::from(elems),
                None => return Vec::new(),
            }
        };
        array
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                if !self.matches_search(item) {
                    return None;
                }
                let mut loop_bindings = bindings.clone();
                loop_bindings.insert(var.to_string(), item.clone());
                if let Some(idx_var) = index {
                    loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                }
                let views: Vec<View<DynamicMessage>> = body
                    .iter()
                    .map(|n| self.convert_node_with(n, &loop_bindings))
                    .collect();
                if views.is_empty() {
                    None
                } else if views.len() == 1 {
                    // Plan 370 (Issue 1): skip Empty body views (see convert_node_with ForLoop).
                    let v = views.into_iter().next().unwrap();
                    if matches!(v, View::Empty) {
                        None
                    } else {
                        Some(v)
                    }
                } else {
                    Some(View::Column {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    })
                }
            })
            .collect()
    }

    /// Convert a grid element. iced has no native grid layout, so decompose
    /// into a **Column of Rows**: chunk the (grid-item) children into rows of
    /// `cols`, each row a horizontal Row. Cells that carry `text-center`
    /// auto-expand to Fill width in the iced text renderer (see `into_iced`'s
    /// `Text` arm), so the columns come out equally sized — a faithful calendar
    /// grid without a real grid primitive. `grid-item` itself is transparent:
    /// it falls through to the generic fallback, which returns its single inner
    /// child, so converting each grid-item yields the cell content directly.
    /// Tracked twin: `convert_grid_tracked_ctx` — keep in sync.
    pub(super) fn convert_grid(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let cols = self
            .extract_u16(props, "cols")
            .or_else(|| self.extract_u16(props, "columns"))
            .or_else(|| self.eval_u16_prop(props, "cols", bindings))
            .map(|c| (c as usize).max(1))
            .unwrap_or(1);
        let gap = self
            .extract_u16(props, "gap")
            .or_else(|| self.eval_u16_prop(props, "gap", bindings))
            .unwrap_or(0);
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // Plan 412 F1/F5: class 里的 grid-cols-N / gap-N 优先于 prop(同 tracked 版)。
        let (cols, gap) = match style.as_ref() {
            Some(s) => {
                let c = s
                    .classes
                    .iter()
                    .rev()
                    .find_map(|cl| match cl {
                        StyleClass::GridCols(n) => Some((*n as usize).max(1)),
                        _ => None,
                    })
                    .unwrap_or(cols);
                let g = s
                    .classes
                    .iter()
                    .rev()
                    .find_map(|cl| match cl {
                        StyleClass::Gap(size) => Some(size.to_pixels()),
                        _ => None,
                    })
                    .unwrap_or(gap);
                (c, g)
            }
            None => (cols, gap),
        };

        // Flatten `for`-loop children into individual cells: a bare `for`
        // inside a grid must yield one cell per iteration, not a single
        // wrapping Column (Plan 323). Other children convert to one cell each.
        // Plan 476: 拼接感知——slot outlet 的 fill 子节点作兄弟 cell 展开。
        let cells: Vec<View<DynamicMessage>> = children
            .iter()
            .flat_map(|n| match n {
                AuraNode::ForLoop {
                    var,
                    index,
                    iterable,
                    body,
                    ..
                } => self.for_loop_iterations(var, index, iterable, body, bindings),
                other => self
                    .expand_one_child_untracked(other, bindings)
                    .into_iter()
                    .filter(|v| !matches!(v, View::Empty))
                    .collect::<Vec<_>>(),
            })
            .collect();

        if cells.is_empty() {
            return View::Empty;
        }

        // Grid decomposition (final-row padding + w-full rows + col-of-rows)
        // now lives in ONE place: the shared generic `build_grid` in the iced
        // renderer. Construct `View::Grid` here;
        // both render paths (render_dynamic_view VM, into_iced rust) consume
        // it identically, so they can never drift again. (Plan 319.)
        View::Grid {
            cols,
            gap,
            cells,
            style,
        }
    }

    /// Convert a container element.
    // Tracked twin: convert_container_tracked_ctx — keep widget logic in sync.
    /// Plan 412 F1: style 布局类重派生(grid→Grid / flex-col→Column / flex→Row),
    /// 与 tracked 版保持一致;无布局类走原 Container 路径。
    pub(super) fn convert_container(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let width = self.extract_u16(props, "width");
        let height = self.extract_u16(props, "height");
        let style = self.extract_style_with(props, bindings); // Plan 057 (2.5)

        // Plan 476: 拼接感知收集 + 视觉空过滤(既有行为)。
        let child_views: Vec<View<DynamicMessage>> = self
            .expand_children_spliced(children, bindings)
            .into_iter()
            .filter(|v| !is_visually_empty(v))
            .collect();

        match rederive_layout(style.as_ref()) {
            Some(RederivedLayout::Grid { cols, gap }) => {
                let mut style = style.unwrap_or_default();
                if padding > 0
                    && !style
                        .classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Padding(_)))
                {
                    style = style.add(StyleClass::Padding(SizeValue::Fixed(padding)));
                }
                if let Some(w) = width {
                    if !style
                        .classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Width(_)))
                    {
                        style = style.add(StyleClass::Width(SizeValue::Pixels(w as f32)));
                    }
                }
                if let Some(h) = height {
                    if !style
                        .classes
                        .iter()
                        .any(|c| matches!(c, StyleClass::Height(_)))
                    {
                        style = style.add(StyleClass::Height(SizeValue::Pixels(h as f32)));
                    }
                }
                View::Grid {
                    cols,
                    gap,
                    cells: child_views,
                    style: Some(style),
                }
            }
            Some(RederivedLayout::Row) => View::Row {
                children: child_views,
                spacing: 0,
                padding,
                style,
                onclick: None,
                on_right_click: None,
            },
            Some(RederivedLayout::Column) => View::Column {
                children: child_views,
                spacing: 0,
                padding,
                style,
                onclick: None,
                on_right_click: None,
            },
            None => {
                let child_view = if child_views.is_empty() {
                    View::Empty
                } else if child_views.len() == 1 {
                    child_views.into_iter().next().unwrap()
                } else {
                    View::Column {
                        children: child_views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                        onclick: None,
                        on_right_click: None,
                    }
                };
                let mut builder = View::container(child_view).padding(padding);
                if let Some(w) = width {
                    builder = builder.width(w);
                }
                if let Some(h) = height {
                    builder = builder.height(h);
                }
                if let Some(s) = style {
                    builder = builder.with_style(s);
                }
                builder.build()
            }
        }
    }

    /// Convert a center element: wraps child in a centered container.
    // Tracked twin: convert_center_tracked_ctx — keep widget logic in sync.
    pub(super) fn convert_center(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        let child_view = if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            self.convert_node_with(&children[0], bindings)
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .collect();
            // Plan 448 续 (center parity): see convert_center_tracked_ctx —
            // items-center so each child centers horizontally like Vue.
            View::Column {
                children: views,
                spacing: 0,
                padding: 0,
                style: Some(Style::default().add(StyleClass::ItemsCenter)),
                onclick: None,
                on_right_click: None,
            }
        };

        // center defaults to w-full h-full so it fills its parent and centers content
        let full_style = match style {
            Some(s) => s
                .add(StyleClass::Width(SizeValue::Full))
                .add(StyleClass::Height(SizeValue::Full)),
            None => Style::default()
                .add(StyleClass::Width(SizeValue::Full))
                .add(StyleClass::Height(SizeValue::Full)),
        };
        let mut builder = View::container(child_view).center_x().center_y();
        builder = builder.with_style(full_style);

        builder.build()
    }

    /// Convert an image element: create View::Image for actual rendering.
    /// Convert an image or icon element.
    /// Plan 408: `icon (name: "bell")` uses a `lucide:{name}` synthetic src
    /// that the iced renderer resolves to a bundled SVG glyph.
    /// Plan 497: window_thumbnail —— 每窗口真缩略 leaf。wid/fallback_icon
    /// 平行字符串 prop（wid 为动态绑定，如 `wid: r.wid`）；像素资产不经
    /// VM 状态，由 iced 渲染臂查进程级快照缓存直绘（T1 定案裁剪式整窗
    /// 快照；miss → request_capture + lucide fallback）。
    pub(super) fn convert_window_thumbnail(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let wid = self
            .extract_string_with(props, "wid", bindings)
            .unwrap_or_default();
        let fallback_icon = self
            .extract_string_with(props, "fallback_icon", bindings)
            .unwrap_or_else(|| "app-window".to_string());
        let style = self.extract_style_with(props, bindings);
        View::WindowThumbnail {
            wid,
            fallback_icon,
            style,
        }
    }

    /// PLAN-012 W3：整桌面等比预览 leaf（window_thumbnail 同型——宿主
    /// 合成数据面 `iced::workspace_preview`，协议零字段增量 SD-02）。
    /// props: ws（分区 id）/ fallback（miss 窗占位 icon，缺省 app-window）。
    pub(super) fn convert_workspace_preview(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let ws = self
            .extract_string_with(props, "ws", bindings)
            .unwrap_or_default();
        let fallback_icon = self
            .extract_string_with(props, "fallback", bindings)
            .unwrap_or_else(|| "app-window".to_string());
        let style = self.extract_style_with(props, bindings);
        View::WorkspacePreview {
            ws,
            fallback_icon,
            style,
        }
    }

    /// PLAN-760：宿主侧远程帧消费 leaf（window_thumbnail 同型——宿主数据
    /// 面 `iced::remote_frame_registry`，帧不进 VM 状态）。props: url（bridge
    /// WS 端点，url 即身份）；class/style 经 extract_style_with 统一入
    /// Style（Chrome classes 同 window_thumbnail 先例）。
    pub(super) fn convert_remote_frame(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let url = self
            .extract_string_with(props, "url", bindings)
            .unwrap_or_default();
        let style = self.extract_style_with(props, bindings);
        View::RemoteFrame { url, style }
    }

    pub(super) fn convert_image_or_icon(
        &self,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let mut style = self.extract_style_with(props, bindings);
        // Plan 606: fit prop -> StyleClass::ObjectFit
        if let Some(fit) = self.extract_string_with(props, "fit", bindings) {
            let class_str = match fit.as_str() {
                "cover" => "object-cover",
                "contain" => "object-contain",
                "fill" => "object-fill",
                "none" => "object-none",
                "scale-down" => "object-scale-down",
                _ => "",
            };
            if !class_str.is_empty() {
                if let Ok(c) = StyleClass::parse_single(class_str) {
                    let mut s = style.unwrap_or_default();
                    s.classes.push(c);
                    style = Some(s);
                }
            }
        }
        // icon: name → 图标 src。PLAN-018：带协议前缀的值（iconfile:/hicon:
        // /lucide:）原样透传（iconfile 位图后端/协议族回退链依赖前缀 intact
        // ——此前无条件 "lucide:" 前缀把 iconfile: 值扭曲成
        // "lucide:iconfile:x"，两后端都不识别→图标空白）；裸名 →
        // "lucide:{name}" synthetic src（既有语义不变）。
        if let Some(name) = self.extract_string_with(props, "name", bindings) {
            if !name.is_empty() {
                let style = self.with_icon_size(style, props, bindings);
                // PLAN-621: state 契约——on/off 注入语义色与激活描边（见
                // with_state_tint 契约注释）；未声明零扰动。
                let style = self.with_state_tint(style, props, bindings);
                let src = if name.starts_with("iconfile:")
                    || name.starts_with("hicon:")
                    || name.starts_with("lucide:")
                {
                    name
                } else {
                    format!("lucide:{}", name)
                };
                return View::Image { src, style };
            }
        }
        // image: src as-is with loop variable / state bindings support
        let src = self
            .extract_string_with(props, "src", bindings)
            .unwrap_or_default();
        View::Image { src, style }
    }

    /// `icon` 的尺寸口径（**两端必须一致**，与 `ui_gen/vue.rs` 的 icon 臂同一套）。
    ///
    /// 优先级：**显式 `w-*`/`h-*` 类 > `size` prop > 默认**。
    ///
    /// 为什么要有这条：`size` 此前是**死参数**——schema 声明了 `default: "24"`，
    /// 但 VM 这条臂从来不读它（只吃 `class`），Vue 端也把它丢了并硬编码
    /// `w-5 h-5`；于是同一个 `icon (name: "x", size: 12)` 两端不一样大，而且
    /// 谁传了 `class: "h-3 w-3"` 也会被 Vue 的硬编码类**静默盖掉**（Tailwind
    /// 同权重，靠样式表顺序定胜负）。现在：显式类优先、`size` 次之、都没有才
    /// 落到共享默认值。
    pub(super) fn with_icon_size(
        &self,
        style: Option<Style>,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<Style> {
        /// 与 Vue 端注入的 `w-5 h-5` 对应（Tailwind 间距刻度 × 4px）。
        const DEFAULT_ICON_PX: f32 = 20.0;
        let mut s = style.unwrap_or_default();
        let has_w = s.classes.iter().any(|c| matches!(c, StyleClass::Width(_)));
        let has_h = s.classes.iter().any(|c| matches!(c, StyleClass::Height(_)));
        if has_w && has_h {
            return Some(s);
        }
        let px = self
            .extract_u16(props, "size")
            .or_else(|| self.eval_u16_prop(props, "size", bindings))
            .map(|v| v as f32)
            .filter(|v| *v > 0.0)
            .unwrap_or(DEFAULT_ICON_PX);
        if !has_w {
            s.classes.push(StyleClass::Width(SizeValue::Pixels(px)));
        }
        if !has_h {
            s.classes.push(StyleClass::Height(SizeValue::Pixels(px)));
        }
        Some(s)
    }

    /// `icon` 的 state 契约（PLAN-621，**两端同规则**，Web 侧同式实现在
    /// `ui_gen/vue.rs` 的 icon 臂）。
    ///
    /// - `state: "on"`（或 bool 绑定求值 true）→ 注入
    ///   `TextColor(Primary)`（**运行时 accent 预设主色**，theme::
    ///   resolve_semantic_rgb 的 Primary 专臂）+ `StrokeWidth(基档 + 0.5)`
    ///   激活加重（lucide 描边技法独有的非颜色线索）；
    /// - `state: "off"`（或 false）→ 注入 `TextColor(OnSurface)`
    ///   （= muted-foreground dim，双盘 token）；
    /// - 未声明/未知值 → 原样返回（**零扰动**，既有金样不得 diff）。
    ///
    /// 优先级（镜像 [`Self::with_icon_size`] 的「显式作者意图优先」）：类集
    /// 已有 `text-*` → 不注入颜色（描边无作者字面通道，照常生效）。
    ///
    /// 描边基档公式与 renderer lucide 路径同式（Plan 518 G4②）：有效盒
    /// ≥48px → 1.5，否则 2.0；**绝对值在此算好**，renderer 只读不加工。
    pub(super) fn with_state_tint(
        &self,
        style: Option<Style>,
        props: &HashMap<String, AuraPropValue>,
        bindings: &Bindings,
    ) -> Option<Style> {
        // 字面量 "on"/"off" 先行（extract_string_with 对 bool 绑定会解析成
        // "true"/"false"，不命中即落 bool 通道），bool 绑定（.liked）走
        // extract_bool_expr（Expr::Bool 直读 / 变量经 resolve 求值）。
        let state: Option<bool> = match self
            .extract_string_with(props, "state", bindings)
            .as_deref()
        {
            Some("on") => Some(true),
            Some("off") => Some(false),
            _ => self.extract_bool_expr(props, "state", bindings),
        };
        let on = match state {
            Some(v) => v,
            None => return style,
        };
        let mut s = style.unwrap_or_default();
        let has_text = s
            .classes
            .iter()
            .any(|c| matches!(c, StyleClass::TextColor(_)));
        if !has_text {
            let color = if on {
                crate::ui::style::Color::Primary
            } else {
                crate::ui::style::Color::OnSurface
            };
            s.classes.push(StyleClass::TextColor(color));
        }
        if on {
            let px = effective_icon_box_px(&s);
            let base = if px >= 48.0 { 1.5 } else { 2.0 };
            s.classes.push(StyleClass::StrokeWidth(base + 0.5));
        }
        Some(s)
    }

    /// PLAN-712 T-04: 把 AURA 的 `video` 节点翻成后端中立的 [`View::Video`]。
    ///
    /// **契约（§2.3）在这一层做「作者面 → 节点字段」的翻译**，两端一致：
    /// - `paused` —— 作者写 `paused: .is_playing == false`，此处拿到的是**已求值的
    ///   bool**（表达式求值在 `extract_bool` 之前完成），故不在这里再取反；
    /// - `volume` —— 作者面 0..100，原样透传（mpv 同刻度）；
    /// - `rate` —— 倍速 float，`<= 0` 一律按 1.0（0 倍速会冻住播放）；
    /// - `position` —— seek 目标（秒），缺省 `None` 表示不下发位置。
    ///
    /// **上行**（`ontimeupdate` 等）：按声明的 `on*` 构建带载荷 handler
    ///（载荷由 VM 端事件泵在分发时刻回填，见 [`MediaEventPayload`]）；
    /// 未声明的键保持 `None`。
    pub(super) fn convert_video(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let src = self
            .extract_string_with(props, "src", bindings)
            .unwrap_or_default();
        let paused = self
            .extract_bool_expr(props, "paused", bindings)
            .unwrap_or(true);
        let position = self.extract_f64_with(props, "position", bindings);
        let volume = self
            .extract_f64_with(props, "volume", bindings)
            .map(|v| v.round().clamp(0.0, 100.0) as i32)
            .unwrap_or(100);
        let muted = self
            .extract_bool_expr(props, "muted", bindings)
            .unwrap_or(false);
        let rate = self
            .extract_f64_with(props, "rate", bindings)
            .filter(|r| *r > 0.0)
            .unwrap_or(1.0);
        // 媒体引擎稳定键（`key:`；2026-10-10 小组件独立播放配套）——字面量
        // 即可（会话域由宿主渲染期 thread-local 注入，作者无需感知）。
        let media_key = self
            .extract_string_with(props, "key", bindings)
            .filter(|s| !s.is_empty());
        // 显示名：优先 title，其次 label，最后退回源路径的末段（降级文案要用真实名字）。
        let label = self
            .extract_string_with(props, "title", bindings)
            .or_else(|| self.extract_string_with(props, "label", bindings))
            .or_else(|| self.extract_string_with(props, "alt", bindings))
            .unwrap_or_else(|| src.rsplit(['/', '\\']).next().unwrap_or("").to_string());
        let style = self.extract_style(props);
        // 上行 handler：载荷在分发时刻回填（ScrollCallback 同款——事件名取自
        // `.OnTime` 这类 handler 模式，widget 名用于宿主路由）。
        let media_up = |key: &str| {
            aura_events_get_base(events, key).map(|ev| {
                let handler = extract_handler_name(&ev.handler).to_string();
                let widget = self.widget_name.clone();
                crate::ui::view::MediaEventHandler::new(
                    move |payload: crate::ui::view::MediaEventPayload| {
                        let arg = match payload {
                            crate::ui::view::MediaEventPayload::Time(s) => Value::Float(s),
                            crate::ui::view::MediaEventPayload::Duration(s) => Value::Float(s),
                            crate::ui::view::MediaEventPayload::Playing(b) => Value::Bool(b),
                            crate::ui::view::MediaEventPayload::Error(s) => Value::Str(s.into()),
                        };
                        DynamicMessage::Typed {
                            widget_name: widget.clone(),
                            event_name: handler.clone(),
                            args: vec![arg],
                        }
                    },
                )
            })
        };
        View::Video {
            src,
            paused,
            position,
            volume,
            muted,
            rate,
            label,
            on_time_update: media_up("ontimeupdate"),
            on_loaded_metadata: media_up("onloadedmetadata"),
            on_play_state: media_up("onplaystatechange"),
            on_media_error: media_up("onmediaerror"),
            on_ended: aura_events_get_base(events, "onended").map(|ev| {
                let handler = extract_handler_name(&ev.handler).to_string();
                DynamicMessage::Typed {
                    widget_name: self.widget_name.clone(),
                    event_name: handler,
                    args: Vec::new(),
                }
            }),
            media_key,
            style,
        }
    }

    /// Build the backend-neutral ImageSurface node from Aura props/events.
    ///
    /// Every value is resolved through the binding-aware extraction helpers so
    /// loop variables, state refs and conditional expressions follow the same
    /// VM path as ordinary widgets.  Event aliases mirror the Vue contract:
    /// load/error/wheel/pan/double-click.  Runtime pointer/load payloads are
    /// appended by the native renderer; the base handler and declared Aura
    /// arguments are preserved here for deterministic VM dispatch.
    pub(super) fn convert_image_surface(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let src = self
            .extract_string_with(props, "src", bindings)
            .unwrap_or_default();
        let alt = self
            .extract_string_with(props, "alt", bindings)
            .unwrap_or_default();
        let as_u32 = |value: Option<f64>| -> u32 {
            value
                .filter(|v| v.is_finite() && *v >= 0.0)
                .map(|v| v.min(u32::MAX as f64) as u32)
                .unwrap_or(0)
        };
        let width = as_u32(self.extract_f64_with(props, "width", bindings));
        let height = as_u32(self.extract_f64_with(props, "height", bindings));
        let quality = self
            .extract_f64_with(props, "quality", bindings)
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 100.0) as u8)
            .unwrap_or(90);
        let fit = self
            .extract_string_with(props, "fit", bindings)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "contain".to_string());
        let zoom = self
            .extract_f64_with(props, "zoom", bindings)
            .filter(|v| v.is_finite() && *v > 0.0)
            .map(|v| v as f32)
            .unwrap_or(1.0);
        let offset_x = self
            .extract_f64_with(props, "offset_x", bindings)
            .filter(|v| v.is_finite())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let offset_y = self
            .extract_f64_with(props, "offset_y", bindings)
            .filter(|v| v.is_finite())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let rotation = self
            .extract_f64_with(props, "rotation", bindings)
            .filter(|v| v.is_finite())
            .map(|v| (v as i32).rem_euclid(360))
            .unwrap_or(0);
        let filter = self
            .extract_string_with(props, "filter", bindings)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "high".to_string());

        let event = |names: &[&str]| {
            names
                .iter()
                .find_map(|name| aura_events_get_base(events, name))
                .map(|ev| self.event_to_message_with(ev, bindings))
        };
        View::ImageSurface {
            src,
            alt,
            width,
            height,
            quality,
            fit,
            zoom,
            offset_x,
            offset_y,
            rotation,
            filter,
            on_error: event(&["onerror", "on_error", "error"]),
            on_loaded: event(&["onload", "onloaded", "on_loaded", "loaded"]),
            on_wheel: event(&["onwheel", "wheel"]),
            on_pan: event(&["onpan", "pan"]),
            on_double_click: event(&["ondblclick", "dblclick", "doubleclick"]),
            style: self.extract_style_with(props, bindings),
        }
    }

    /// Convert a progress element: shows a progress bar from 0.0 to 1.0.
    pub(super) fn convert_progress(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, auto_ui_compiler::aura::AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        // Extract value and max, compute progress ratio. bindings 版:for 循环
        // 内 `value: item.pct` 的循环变量绑定解析(auto-os-config Storage 卡
        // 组件化时发现——空 bindings 版循环内恒 0%)。
        let value = self
            .extract_f64_with(props, "value", bindings)
            .unwrap_or(0.0);
        let max = self
            .extract_f64_with(props, "max", bindings)
            .unwrap_or(100.0);
        let progress = if max > 0.0 {
            (value / max).clamp(0.0, 1.0)
        } else {
            0.0
        };

        View::ProgressBar {
            progress: progress as f32,
            style,
            on_seek: self.progress_seek_arm(events, bindings),
        }
    }

    /// `progress` 的 `onseek` 臂 —— 与 mouse-area 的 `onmousemove` 同型
    /// （`PointerMoveHandler` 把逻辑坐标/比例追加为 Float 实参），只是这里
    /// 交给作者的是**横向比例 0..1**（第二个实参恒 0.0，进度条是一维的）。
    ///
    /// 比例而不是值：到这一层 `value/max` 已经归一，`max` 不再可得；让两端
    /// 统一交比例，作者写 `store.SeekTo(.duration * $0)` 就不会跨端漂移。
    pub(super) fn progress_seek_arm(
        &self,
        events: &HashMap<String, auto_ui_compiler::aura::AuraEvent>,
        bindings: &Bindings,
    ) -> Option<crate::ui::view::PointerMoveHandler<DynamicMessage>> {
        let event = aura_events_get_base(events, "onseek")?;
        let base = self.event_to_message_with(event, bindings);
        Some(crate::ui::view::PointerMoveHandler::new(
            move |fraction: f32, _y: f32| match &base {
                DynamicMessage::Typed {
                    widget_name,
                    event_name,
                    args,
                } => {
                    let mut new_args = args.clone();
                    // 与 mouse_area_move_arm 同规约：+1e-3 分数化，绕开
                    // auto_val nanbox 整值 float 实参绑定的腐坏路径。
                    new_args.push(Value::Float(fraction as f64 + 0.001));
                    new_args.push(Value::Float(0.001));
                    DynamicMessage::Typed {
                        widget_name: widget_name.clone(),
                        event_name: event_name.clone(),
                        args: new_args,
                    }
                }
                other => other.clone(),
            },
        ))
    }

    /// Convert a spacer element: fills remaining space in a flex layout.
    /// Plan 412 F6: 默认 w-full h-full — 主轴吃剩余空间(Row 中宽 Fill、Column
    /// 中高 Fill,双向 Fill 让 spacer 在任一方向的 flex 容器里都占据主轴剩余,
    /// 对齐 vue 侧 spacer→div 补 `flex-1` 的语义),交叉轴 Fill 复现 CSS 默认
    /// stretch。显式 style(spacer (style: "w-8"))不受影响。
    pub(super) fn convert_spacer(&self, props: &HashMap<String, AuraPropValue>) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        let child = View::Empty;
        let mut builder = View::container(child);
        if let Some(s) = style {
            builder = builder.with_style(s);
        } else {
            builder = builder.with_style(Style::parse("flex-1").unwrap());
        }
        builder.build()
    }

    /// Plan 412 §4.3: demo 占位块 `square` — 色块 + 双轴居中数字。
    /// props:color(Tailwind 色名,默认 blue)/ size(方形边长单位,默认 12)/
    /// h/w(单位数或 "full",覆盖 size)/ class(附加类如 col-span-2)/
    /// text(内容;children 存在时优先 children)。
    /// VM 用 Container center_x/center_y 居中(vue 端用 flex items/justify)。
    pub(super) fn convert_square(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let color = self
            .extract_string(props, "color")
            .unwrap_or_else(|| "blue".to_string());
        let extra = self.extract_string(props, "class").unwrap_or_default();
        let size = self.extract_u16(props, "size").unwrap_or(12);
        // 维度:数字(Tailwind 单位)或 "full";缺省回退 size。
        let dim = |name: &str| -> Option<String> {
            if let Some(s) = self.extract_string(props, name) {
                if s == "full" {
                    return Some("full".to_string());
                }
            }
            self.extract_u16(props, name).map(|n| n.to_string())
        };
        let h = dim("h").unwrap_or_else(|| size.to_string());
        let w = dim("w").unwrap_or_else(|| size.to_string());
        let mut cls = format!(
            "h-{h} w-{w} rounded-md border bg-{c}-500/40 border-{c}-500 text-{c}-600 font-medium text-sm",
            c = color
        );
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
        let style = Style::parse(&cls).unwrap_or_default();

        let child = if children.is_empty() {
            match self.extract_string(props, "text") {
                Some(t) if !t.is_empty() => View::Text {
                    content: t,
                    style: None,
                    selectable: false,
                },
                _ => View::Empty,
            }
        } else if children.len() == 1 {
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
        };

        View::container(child)
            .center_x()
            .center_y()
            .with_style(style)
            .build()
    }

    /// Convert a divider element: renders a horizontal line separator.
    pub(super) fn convert_divider(&self, _props: &HashMap<String, AuraPropValue>) -> View<DynamicMessage> {
        let child = View::Empty;
        let mut builder = View::container(child);
        builder = builder.with_style(Style::parse("w-full h-1 bg-gray-200").unwrap());
        builder.build()
    }

}
