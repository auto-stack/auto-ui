//! PLAN-014 T-06：样式/widget 构建责任（viewport 改写、容器/按钮/行/滚动构建、焦点收集与网格、内嵌 plan732/737/plan095 测试）（自 iced/renderer.rs 原样收编）。
//! 消息序/active判据/last-owner exit/纹理与external pin寿命零改——纯移动。
use super::*;
use super::devtools::*;
use super::host::*;

/// Trait for converting abstract View<M> into Iced Element
///
/// This trait enables rendering the abstract view tree using the Iced framework
/// with full style support through IcedStyle.
pub trait IntoIcedElement<M: Clone + Debug + 'static> {
    /// Convert abstract view into Iced Element
    fn into_iced(self) -> iced::Element<'static, M>;
}

/// Mutable class access for the style-carrying View variants — used by the
/// Row/Column dispatch points to apply axis-aware fixes BEFORE children
/// recurse into `into_iced`/`render_dynamic_view` (from_style has no parent
/// context, so the axis must be resolved at the dispatch site; see the
/// Row/Column arms). Variants not listed here keep the legacy behaviour.
fn view_classes_mut<M: Clone + Debug>(v: &mut AbstractView<M>) -> Option<&mut Vec<StyleClass>> {
    use AbstractView as V;
    match v {
        V::Row { style: Some(s), .. }
        | V::Column { style: Some(s), .. }
        | V::Text { style: Some(s), .. }
        | V::Button { style: Some(s), .. }
        | V::Input { style: Some(s), .. }
        | V::Textarea { style: Some(s), .. }
        | V::CodeEditor { style: Some(s), .. }
        | V::Image { style: Some(s), .. } => Some(&mut s.classes),
        _ => None,
    }
}

/// 横向主轴(Row 直接子)修正:剥 FlexCol,阻断 from_style 的"FlexCol→补
/// height=Fill"猜测(横向 Shrink row 里该 Fill 无界可填 → 整卡塌 0)。
pub(crate) fn axis_fix_row_child<M: Clone + Debug>(mut c: AbstractView<M>) -> AbstractView<M> {
    if let Some(classes) = view_classes_mut(&mut c) {
        classes.retain(|cl| !matches!(cl, StyleClass::FlexCol));
    }
    c
}

/// 纵向主轴(Column 直接子)修正:flex-1 是 CSS flex-grow 的主轴(纵向)
/// 语义 → 转写为显式 Height(Full);原先被误设的 width=Fill 一并撤除
/// (col 子项的 flex-1 本就不该撑宽)。
pub(crate) fn axis_fix_col_child<M: Clone + Debug>(mut c: AbstractView<M>) -> AbstractView<M> {
    if let Some(classes) = view_classes_mut(&mut c) {
        let mut has_grow = false;
        classes.retain(|cl| match cl {
            StyleClass::Flex1 | StyleClass::FlexAuto | StyleClass::Grow => {
                has_grow = true;
                false
            }
            _ => true,
        });
        if has_grow && !classes.iter().any(|cl| matches!(cl, StyleClass::Height(_))) {
            classes.push(StyleClass::Height(SizeValue::Full));
        }
    }
    c
}

/// PLAN-642 T-11: distributed（justify-between/around/evenly）列判定——
/// 与 build_column 的垫片发射共用同一口径，子项 grow 剥离条件随之同步。
pub(crate) fn style_is_distributed(style: Option<&Style>) -> bool {
    style
        .map(IcedStyle::from_style)
        .and_then(|is| is.justify_content)
        .is_some_and(|j| {
            matches!(
                j,
                IcedJustify::Between | IcedJustify::Around | IcedJustify::Evenly
            )
        })
}

/// PLAN-642 T-11: distributed 列的直接子修正——剥 grow（Flex1/FlexAuto/
/// Grow）且不补 Height(Full)。iced 0.14 flex 第三 pass 对 FillPortion 子
/// min=max=份额硬钉，且列内子项按剩余量逐个配给：份额 < 内容自然高时
/// 后续子项被配给至 0×0 隐没（008 定价卡特性行 18 节点只画 1 行的根因，
/// headless 复现 p642_t11_008_feat_rows_probe）。CSS flex 的 grow 子有
/// min-content 钳制永不隐没；iced 无该钳制，distributed 列里 grow 让渡
/// 给垫片独占（子项保持自然高，justify 分布语义不变；欠额场景差异仅为
/// "grow 子不再撑高"，由垫片吸收等量空隙）。
pub(crate) fn axis_fix_col_child_distributed<M: Clone + Debug>(
    mut c: AbstractView<M>,
) -> AbstractView<M> {
    if let Some(classes) = view_classes_mut(&mut c) {
        classes.retain(|cl| {
            !matches!(
                cl,
                StyleClass::Flex1 | StyleClass::FlexAuto | StyleClass::Grow
            )
        });
    }
    c
}

// ═══ PLAN-663 C1b: 视口边界重写（iframe 语义）═══
//
// `h-screen`/`w-screen`（SizeValue::Screen）在独立窗口根 = Fill（满窗，
// iced_length 缺省臂）；但嵌入"定高(px)容器"（gallery 视口 frame、虚拟桌
// 面窗等）后，视口单位应重锚定到该边界（CSS iframe 模型：嵌入框建立新视
// 口）。若不重写，Screen→Fill 在 Shrink 祖先（T-12 兜底的 Shrink scroll、
// justify-center 让渡列高）下失去定高锚点，整树塌缩为最小内容高——
// ui-gallery VM 臂全屏 demo 只剩一条播控条的根因。
//
// 实现形态：渲染前的 AbstractView 树重写 pre-pass（与 axis_fix_* /
// inherit_text_color 同族），而非渲染器穿线 context——零渲染签名改动，
// Plan 319 双入口（render_dynamic_view / into_iced）各调一次即可，且可独
// 立单测。边界判定仅认 **arbitrary px 定高/定宽**（`h-[720px]`）；spacing
// 定高（`h-44`）不建立边界（控制爆炸半径，KNOWN-DEBT 登记）。嵌套边界逐
// 轴覆盖外层锚点。`h-full`（Full）不重写：其 CSS 原义是父容器百分比，重写
// 会破坏 demo 内部的局部布局语义（全屏外壳的仓内惯例恰是 h-screen）。

/// 视口锚：可仅有单轴（`w-[1024px]` 只锚宽）。
#[derive(Clone, Copy, Default, PartialEq, Debug)]
struct ViewportAnchor {
    w: Option<f32>,
    h: Option<f32>,
}

impl ViewportAnchor {
    const NONE: ViewportAnchor = ViewportAnchor { w: None, h: None };

    fn merge(self, inner: ViewportAnchor) -> ViewportAnchor {
        ViewportAnchor {
            w: inner.w.or(self.w),
            h: inner.h.or(self.h),
        }
    }
}

/// 从节点样式提取其**自身**建立的视口边界（仅 arbitrary px 定值，>0）。
fn viewport_boundary_of(style: Option<&Style>) -> ViewportAnchor {
    let Some(s) = style else {
        return ViewportAnchor::NONE;
    };
    let mut a = ViewportAnchor::NONE;
    for c in &s.classes {
        match c {
            StyleClass::Height(SizeValue::Pixels(px)) if *px > 0.0 => a.h = Some(*px),
            StyleClass::Width(SizeValue::Pixels(px)) if *px > 0.0 => a.w = Some(*px),
            _ => {}
        }
    }
    a
}

/// 可承载/传递视口边界的样式容器变体（含既有 view_classes_mut 覆盖面 +
/// Container/Scrollable 两个嵌入链上的关键变体）。
fn viewport_classes_mut<M: Clone + Debug>(v: &mut AbstractView<M>) -> Option<&mut Vec<StyleClass>> {
    use AbstractView as V;
    match v {
        V::Row { style: Some(s), .. }
        | V::Column { style: Some(s), .. }
        | V::Text { style: Some(s), .. }
        | V::Button { style: Some(s), .. }
        | V::Input { style: Some(s), .. }
        | V::Textarea { style: Some(s), .. }
        | V::CodeEditor { style: Some(s), .. }
        | V::Image { style: Some(s), .. }
        | V::Container { style: Some(s), .. }
        | V::Scrollable { style: Some(s), .. } => Some(&mut s.classes),
        _ => None,
    }
}

/// 渲染入口调用：整树重写视口单位（见模块注释）。
pub(crate) fn rewrite_viewport_units<M: Clone + Debug>(root: &mut AbstractView<M>) {
    expand_margin_y_auto_walk(root);
    rewrite_viewport_walk(root, ViewportAnchor::NONE);
}

// ═══ PLAN-663 C2: my-auto/m-auto 垂直安全居中 ══
//
// CSS 中 `margin-top/bottom: auto` 在定高 flex 容器内吸收剩余空间（safe
// center：内容矮→居中，高→贴顶溢出）。iced 无 margin-auto 布局原语，渲染
// 前把「定高(px)且无 overflow 的 Column」内携带 my-auto/m-auto 的直接子改
// 写为 h-full justify-center 包裹列：iced justify-center 让渡列高给外包
// container（center_y），矮内容被垂直居中；高内容溢出裁剪——与 CSS 语义
// 对齐（无滚动场景；滚动由调用方 frame 的 overflow 层承接，见 T-12 组
// 合）。带 overflow 的列不展开（滚动交互动 T-12 机制，双重语义不可叠加）。

fn column_definite_height_no_overflow(style: Option<&Style>) -> bool {
    let Some(s) = style else { return false };
    let mut definite = false;
    for c in &s.classes {
        match c {
            StyleClass::Height(SizeValue::Pixels(px)) if *px > 0.0 => definite = true,
            StyleClass::OverflowYAuto
            | StyleClass::OverflowYHidden
            | StyleClass::OverflowHidden => return false,
            _ => {}
        }
    }
    definite
}

fn child_has_margin_y_auto<M: Clone + Debug>(v: &AbstractView<M>) -> bool {
    let style = match v {
        AbstractView::Row { style, .. }
        | AbstractView::Column { style, .. }
        | AbstractView::Text { style, .. }
        | AbstractView::Button { style, .. }
        | AbstractView::Input { style, .. }
        | AbstractView::Textarea { style, .. }
        | AbstractView::CodeEditor { style, .. }
        | AbstractView::Image { style, .. }
        | AbstractView::Container { style, .. }
        | AbstractView::Scrollable { style, .. } => style,
        _ => return false,
    };
    let Some(s) = style else { return false };
    s.classes
        .iter()
        .any(|c| matches!(c, StyleClass::MarginYAuto | StyleClass::MarginAuto))
}

fn expand_margin_y_auto_walk<M: Clone + Debug>(v: &mut AbstractView<M>) {
    let transform_children = matches!(v, AbstractView::Column { .. }) && {
        let style = match &*v {
            AbstractView::Column { style, .. } => style.as_ref(),
            _ => None,
        };
        column_definite_height_no_overflow(style)
    };
    if transform_children {
        if let AbstractView::Column { children, .. } = v {
            for c in children.iter_mut() {
                if child_has_margin_y_auto(c) {
                    let inner = std::mem::replace(c, AbstractView::Empty);
                    *c = AbstractView::Column {
                        children: vec![inner],
                        spacing: 0,
                        padding: 0,
                        style: Style::parse("h-full w-full flex-col justify-center").ok(),
                        onclick: None,
                        on_right_click: None,
                    };
                }
            }
        }
    }
    // 继续下潜（包裹列的子不再含 my-auto——刚从原位搬入；其余子照常扫描）
    match v {
        AbstractView::Row { children, .. } | AbstractView::Column { children, .. } => {
            for c in children.iter_mut() {
                expand_margin_y_auto_walk(c);
            }
        }
        AbstractView::Grid { cells, .. } => {
            for c in cells.iter_mut() {
                expand_margin_y_auto_walk(c);
            }
        }
        AbstractView::Container { child, .. }
        | AbstractView::Scrollable { child, .. }
        | AbstractView::MouseArea { content: child, .. } => {
            expand_margin_y_auto_walk(child);
        }
        AbstractView::Overlay { base, content, .. } => {
            expand_margin_y_auto_walk(base);
            expand_margin_y_auto_walk(content);
        }
        _ => {}
    }
}

/// 对 `v` 的直接子逐个：先按当前锚重写子自身视口类，再携锚下潜。
fn rewrite_viewport_walk<M: Clone + Debug>(v: &mut AbstractView<M>, inherited: ViewportAnchor) {
    // 自身边界（对子生效）；不可变借用先行结束，再可变借子。
    let own = {
        // 复用只读通道：走 viewport_classes_mut 的不可变版本等价——直接
        // match style 字段。
        let style = match v {
            AbstractView::Row { style, .. }
            | AbstractView::Column { style, .. }
            | AbstractView::Text { style, .. }
            | AbstractView::Button { style, .. }
            | AbstractView::Input { style, .. }
            | AbstractView::Textarea { style, .. }
            | AbstractView::CodeEditor { style, .. }
            | AbstractView::Image { style, .. }
            | AbstractView::Container { style, .. }
            | AbstractView::Scrollable { style, .. } => style.as_ref(),
            _ => None,
        };
        viewport_boundary_of(style)
    };
    let eff = inherited.merge(own);
    match v {
        AbstractView::Row { children, .. } | AbstractView::Column { children, .. } => {
            for c in children.iter_mut() {
                rewrite_viewport_node(c, eff);
            }
        }
        AbstractView::Grid { cells, .. } => {
            for c in cells.iter_mut() {
                rewrite_viewport_node(c, eff);
            }
        }
        AbstractView::Container { child, .. }
        | AbstractView::Scrollable { child, .. }
        | AbstractView::MouseArea { content: child, .. } => {
            rewrite_viewport_node(child, eff);
        }
        AbstractView::Overlay { base, content, .. } => {
            rewrite_viewport_node(base, eff);
            rewrite_viewport_node(content, eff);
        }
        _ => {}
    }
}

/// 重写单节点自身的视口单位类（Screen/MinHeight 标记 → 边界定值），随后
/// 继续下潜（节点自身若是边界，对孙辈换锚）。
fn rewrite_viewport_node<M: Clone + Debug>(v: &mut AbstractView<M>, anchor: ViewportAnchor) {
    if let Some(classes) = viewport_classes_mut(v) {
        for c in classes.iter_mut() {
            match *c {
                StyleClass::Height(SizeValue::Screen) if anchor.h.is_some() => {
                    *c = StyleClass::Height(SizeValue::Pixels(anchor.h.unwrap_or(0.0)));
                }
                StyleClass::Width(SizeValue::Screen) if anchor.w.is_some() => {
                    *c = StyleClass::Width(SizeValue::Pixels(anchor.w.unwrap_or(0.0)));
                }
                // min-h-screen 族：parse 落 f32::MAX 标记（renderer 以
                // >= 9999.0 判 Fill），边界内重写为边界定高。
                StyleClass::MinHeight(m) if m >= 9999.0 && anchor.h.is_some() => {
                    *c = StyleClass::MinHeight(anchor.h.unwrap_or(0.0));
                }
                _ => {}
            }
        }
    }
    rewrite_viewport_walk(v, anchor);
}

/// Helper to compute effective spacing: style.gap takes priority, then legacy spacing.
/// Plan 412: axis-aware — Row consumes gap-x (axis-specific wins over bare gap),
/// Column consumes gap-y. The other axis is ignored, mirroring CSS grid/flex semantics.
pub(super) fn effective_spacing(legacy: u16, style: Option<&Style>, horizontal: bool) -> f32 {
    if let Some(s) = style {
        let iced_style = IcedStyle::from_style(s);
        let axis = if horizontal {
            iced_style.gap_x
        } else {
            iced_style.gap_y
        };
        if let Some(g) = axis.or(iced_style.gap) {
            return g;
        }
    }
    legacy as f32
}

/// Compute iced Padding (per-axis) from style, falling back to legacy u16.
/// PLAN-619 R2: 委托 `IcedStyle::effective_padding` —— 单侧 > 轴 > 统一 >
/// legacy。此前 uniform 命中即 early-return，`p-2 py-1.5` 的 per-axis
/// 覆盖被整个丢弃（VM 便签行比 Vue 高 4px 的根因）。
pub(super) fn iced_padding_from_is(is: &IcedStyle, legacy: u16) -> iced::Padding {
    is.effective_padding(legacy)
}

fn iced_padding(legacy: u16, style: Option<&Style>) -> iced::Padding {
    if let Some(s) = style {
        let is = IcedStyle::from_style(s);
        return iced_padding_from_is(&is, legacy);
    }
    iced::Padding::new(legacy as f32)
}

/// Build an Iced container::Style from IcedStyle, covering background, border, shadow, text_color.
pub(super) fn build_container_style(is: &IcedStyle) -> iced::widget::container::Style {
    use iced::Background;
    let has_radius = is.has_border_radius();
    let border =
        if is.rounded || is.border || has_radius || is.border_width.map_or(false, |w| w > 0.0) {
            let width = is.border_width.unwrap_or(if is.border { 1.0 } else { 0.0 });
            let color = is.border_color.unwrap_or_else(|| {
                let (r, g, b) = crate::ui::style_iced_adapter::resolve_border_rgb();
                iced::Color::from_rgb8(r, g, b)
            });
            iced::Border {
                color: if width > 0.0 {
                    color
                } else {
                    iced::Color::TRANSPARENT
                },
                width,
                radius: is.effective_border_radius(),
            }
        } else {
            iced::Border::default()
        };
    let shadow = if is.shadow {
        let (offset_y, blur) = match is.shadow_size {
            Some(IcedShadowSize::Sm) => (1.0, 2.0),
            Some(IcedShadowSize::Md) => (2.0, 4.0),
            Some(IcedShadowSize::Lg) => (4.0, 8.0),
            Some(IcedShadowSize::Xl) => (8.0, 16.0),
            Some(IcedShadowSize::Xxl) => (12.0, 24.0),
            _ => (2.0, 4.0),
        };
        iced::Shadow {
            // Plan 527 T4: shadow-{color} 彩色阴影 —— 保留默认阴影 alpha(0.15),
            // 只替换色相(对齐 Tailwind 彩色阴影 = color × 默认阴影透明度)。
            color: {
                let (sr, sg, sb) = is
                    .shadow_color
                    .map(|c| (c.r, c.g, c.b))
                    .unwrap_or((0.0, 0.0, 0.0));
                iced::Color::from_rgba(sr, sg, sb, 0.15)
            },
            offset: iced::Vector::new(0.0, offset_y),
            blur_radius: blur,
        }
    } else {
        iced::Shadow::default()
    };
    // Build background: use gradient if both from/to colors present, else solid
    let background =
        if is.gradient_clip_text && is.gradient_from.is_some() && is.gradient_to.is_some() {
            // PLAN-625 T-05: 渐变裁剪文字降级——渐变仅填充文字(CSS bg-clip:text),
            // iced 无对应能力;按底盒绘制会形成透明文字+实心渐变块,整体抑制,
            // 文字回落继承色(from_style 臂已清 transparent text_color)。
            None
        } else if is.gradient_from.is_some() && is.gradient_to.is_some() {
            let from = is.gradient_from.unwrap();
            let to = is.gradient_to.unwrap();
            let angle = match is.gradient_dir {
                Some(crate::ui::style::GradientDir::ToB) | None => 180.0_f32.to_radians(),
                Some(crate::ui::style::GradientDir::ToT) => 0.0,
                Some(crate::ui::style::GradientDir::ToR) => 90.0_f32.to_radians(),
                Some(crate::ui::style::GradientDir::ToL) => 270.0_f32.to_radians(),
                Some(crate::ui::style::GradientDir::ToBR) => 135.0_f32.to_radians(),
                Some(crate::ui::style::GradientDir::ToBL) => 225.0_f32.to_radians(),
                Some(crate::ui::style::GradientDir::ToTR) => 45.0_f32.to_radians(),
                Some(crate::ui::style::GradientDir::ToTL) => 315.0_f32.to_radians(),
            };
            use iced::gradient::Linear;
            // Plan 527 T4: from/via/to 多 stop + 位置百分比(默认 0/50/100)——
            // iced 0.14 Linear::add_stop 任意 offset,via 缺席时保持双 stop 旧观感。
            let mut lin = Linear::new(angle).add_stop(is.gradient_from_pos.unwrap_or(0.0), from);
            if let Some(via) = is.gradient_via {
                lin = lin.add_stop(is.gradient_via_pos.unwrap_or(0.5), via);
            }
            Some(Background::Gradient(
                lin.add_stop(is.gradient_to_pos.unwrap_or(1.0), to).into(),
            ))
        } else {
            is.background_color.map(Background::Color)
        };
    iced::widget::container::Style {
        background,
        text_color: is.text_color,
        border,
        shadow,
        ..Default::default()
    }
}

/// Build an Iced button::Style from IcedStyle.
/// Plan 409 §10 续 20: 简单代码语法高亮(auto/vue/bash 通用)→ 着色 Span 列表。
/// tokenizer:comment / string / number+boolean / keyword / function / operator / punct / ident。
///
/// 2026-08-22(show 下沉):扫描逻辑与色板移至 vm::shell_bridge::highlight_rgb
/// —— auto.shell.emit_show 的 Code payload 与本处 font-mono 自动高亮共用一份
/// 实现,避免两份漂移。None = 默认前景色(color_maybe)。
///
/// Plan 442 A6: a known lang (codeblock 的 lang-<token> class)改走 syntect
/// highlight-only API——与 code_editor 同引擎同主题(musk-038 T16 决策 (a));
/// 无 lang 的 mono 文本保持共享手写 tokenizer(shell show payload 等)。
pub(super) fn highlight_code(code: &str, lang: Option<&str>) -> Vec<iced::widget::text::Span<'static, ()>> {
    use iced::widget::text::Span;
    let colored: Vec<(String, Option<(u8, u8, u8)>)> = match lang {
        Some(l) if !l.is_empty() => {
            let (dark, accent) = crate::ui::code_editor::theme::theme_source();
            crate::ui::code_editor::core::highlight::highlight_segments(l, code, dark, &accent)
        }
        _ => auto_lang_core::vm::shell_bridge::highlight_rgb(code),
    };
    colored
        .into_iter()
        .map(|(text, rgb)| {
            let mut sp = Span::new(text);
            if let Some((r, g, b)) = rgb {
                sp = sp.color(iced::Color::from_rgb8(r, g, b));
            }
            sp
        })
        .collect()
}

/// Plan 411 P2-A④: 表头单元格样式——font-medium + text-muted-foreground
/// (vue 侧 `<th class="font-medium text-muted-foreground">`)。递归下钻到
/// Text 视图追加 class;表头可能是 Row/Container 包裹的复合内容。
pub(super) fn apply_table_header_style<M: Clone + Debug>(v: &mut AbstractView<M>) {
    match v {
        AbstractView::Text { style, .. } => {
            let st = style.get_or_insert_with(Default::default);
            st.classes.push(StyleClass::FontMedium);
            st.classes.push(StyleClass::TextColor(Color::OnSurface));
        }
        AbstractView::Row { children, .. } | AbstractView::Column { children, .. } => {
            for c in children {
                apply_table_header_style(c);
            }
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            apply_table_header_style(child);
        }
        _ => {}
    }
}

/// Plan 411 P2-A④: 单元格 px-4/py-3 padding(vue `<td class="px-4 py-3">`)。
/// Plan 045 T2: `width` Some 时容器定宽（Fixed(w)）——列宽应用的落点；
/// None 维持 Shrink 自然宽（现状）。
fn table_cell_container<'a, M: 'static>(
    el: iced::Element<'a, M>,
    width: Option<f32>,
) -> iced::Element<'a, M> {
    let mut c = iced::widget::container(el).padding(iced::Padding {
        top: 12.0,
        bottom: 12.0,
        left: 16.0,
        right: 16.0,
    });
    if let Some(w) = width {
        c = c.width(iced::Length::Fixed(w));
    }
    c.into()
}

/// Plan 045 T2: 列宽两态分派纯函数——`col_widths` Some 且列号在范围内返回
/// 固定宽（px）；None 或长度不足的尾列返回 None（自然宽）。超列数的尾值
/// 无消费方（自然截断）。表头与体行走同一分派。
pub(super) fn col_fixed_width(col_widths: Option<&[f32]>, col: usize) -> Option<f32> {
    col_widths.and_then(|ws| ws.get(col).copied())
}

/// Plan 411 P2-A④: 行 border-b——iced 容器边框是四边整圈,单侧下边线用
/// 1px 填充分隔条子元素模拟(vue `border-b border-border`)。
fn table_row_rule<M: 'static>() -> iced::Element<'static, M> {
    let (r, g, b) = crate::ui::style_iced_adapter::resolve_border_rgb();
    iced::widget::container(
        iced::widget::Space::new()
            .width(iced::Length::Shrink)
            .height(iced::Length::Fixed(0.0)),
    )
    .width(iced::Length::Fill)
    .height(iced::Length::Fixed(1.0))
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb8(r, g, b))),
        ..Default::default()
    })
    .into()
}

/// Plan-050 C1: content-subtree 按钮的内容对齐决策——web flex 语义到 iced
/// 容器对齐的最小映射（仅消费四界面布局用到的组合；未命中返回 None = 维持
/// iced Button 默认内容居中）。justify-content 优先于 text-align（前者是
/// 布局声明，后者是文本对齐兜底）。
pub(super) fn plan050_content_align(
    is: Option<&IcedStyle>,
) -> (
    Option<iced::alignment::Horizontal>,
    Option<iced::alignment::Vertical>,
) {
    use iced::alignment::{Horizontal, Vertical};
    let Some(is) = is else { return (None, None) };
    let h = match is.justify_content {
        Some(crate::ui::style_iced_adapter::IcedJustify::Start) => Some(Horizontal::Left),
        Some(crate::ui::style_iced_adapter::IcedJustify::End) => Some(Horizontal::Right),
        Some(crate::ui::style_iced_adapter::IcedJustify::Center) => Some(Horizontal::Center),
        _ => is.text_align.map(|a| match a {
            crate::ui::style_iced_adapter::IcedTextAlign::Left => Horizontal::Left,
            crate::ui::style_iced_adapter::IcedTextAlign::Center => Horizontal::Center,
            crate::ui::style_iced_adapter::IcedTextAlign::Right => Horizontal::Right,
        }),
    };
    let v = is.align_items.map(|a| match a {
        crate::ui::style_iced_adapter::IcedAlign::Start => Vertical::Top,
        crate::ui::style_iced_adapter::IcedAlign::Center => Vertical::Center,
        crate::ui::style_iced_adapter::IcedAlign::End => Vertical::Bottom,
    });
    (h, v)
}

/// Plan 409/414 高度臂 × Plan-050 显式对齐类的让位规则:按钮带高度类时
/// 其内容容器默认双向居中(CSS button 语义);样式串携带显式对齐类
/// (text-left/justify-*、items-*)的轴以显式值为准——否则 050 的外层
/// 包装包不住已 Fill 的内层容器,nav-item(h-9 w-full text-left)内容
/// 恒居中而 web 轨左对齐,双端漂移。
pub(super) fn plan414_content_alignment(
    p050_ax: Option<iced::alignment::Horizontal>,
    p050_ay: Option<iced::alignment::Vertical>,
) -> (iced::alignment::Horizontal, iced::alignment::Vertical) {
    use iced::alignment::{Horizontal, Vertical};
    (
        p050_ax.unwrap_or(Horizontal::Center),
        p050_ay.unwrap_or(Vertical::Center),
    )
}

/// PLAN-050 P2 #3: mt-auto（MarginTop(Auto)）在 iced 无 margin-auto 几何——
/// Column 发射时在该子项前方插一根 Fill 高度占位条，恢复 web 侧"推到底部"
/// 语义。判定独立成函数钉住契约：mt-auto 经 master 通用 mt-<size> 路径解析，
/// 曾有专用变体被通用路径打死的前科，此函数是回归防线。
pub(super) fn plan050_mt_auto_spacer(classes: &[StyleClass]) -> bool {
    classes
        .iter()
        .any(|x| matches!(x, StyleClass::MarginTop(SizeValue::Auto)))
}

pub(super) fn build_button_style(is: &IcedStyle) -> iced::widget::button::Style {
    use iced::Background;
    let has_radius = is.has_border_radius();
    let border =
        if is.rounded || is.border || has_radius || is.border_width.map_or(false, |w| w > 0.0) {
            let width = is.border_width.unwrap_or(if is.border { 1.0 } else { 0.0 });
            let color = is.border_color.unwrap_or_else(|| {
                let (r, g, b) = crate::ui::style_iced_adapter::resolve_border_rgb();
                iced::Color::from_rgb8(r, g, b)
            });
            iced::Border {
                color: if width > 0.0 {
                    color
                } else {
                    iced::Color::TRANSPARENT
                },
                width,
                radius: is.effective_border_radius(),
            }
        } else {
            iced::Border::default()
        };
    let shadow = if is.shadow {
        let (offset_y, blur) = match is.shadow_size {
            Some(IcedShadowSize::Sm) => (1.0, 2.0),
            Some(IcedShadowSize::Md) => (2.0, 4.0),
            Some(IcedShadowSize::Lg) => (4.0, 8.0),
            Some(IcedShadowSize::Xl) => (8.0, 16.0),
            Some(IcedShadowSize::Xxl) => (12.0, 24.0),
            _ => (2.0, 4.0),
        };
        iced::Shadow {
            // Plan 527 T4: shadow-{color} 彩色阴影 —— 保留默认阴影 alpha(0.15),
            // 只替换色相(对齐 Tailwind 彩色阴影 = color × 默认阴影透明度)。
            color: {
                let (sr, sg, sb) = is
                    .shadow_color
                    .map(|c| (c.r, c.g, c.b))
                    .unwrap_or((0.0, 0.0, 0.0));
                iced::Color::from_rgba(sr, sg, sb, 0.15)
            },
            offset: iced::Vector::new(0.0, offset_y),
            blur_radius: blur,
        }
    } else {
        iced::Shadow::default()
    };
    // Plan 527 T6: opacity-{N} —— iced 无元素级 opacity 组,按钮面降级为
    // 背景/文字 alpha 乘以倍率(visible 而非静默;disabled:opacity-50 等
    // 变体类经 merged_with_variant 到此消费)。
    let opacity = is.opacity.unwrap_or(1.0);
    let fade = |c: iced::Color| iced::Color {
        a: c.a * opacity,
        ..c
    };
    iced::widget::button::Style {
        background: is.background_color.map(|c| Background::Color(fade(c))),
        text_color: fade(is.text_color.unwrap_or_else(|| {
            // Plan 408: dark-mode-aware default (equivalent to vue body text-foreground).
            crate::ui::style_iced_adapter::resolve_semantic_rgb(
                &crate::ui::style::Color::OnBackground,
            )
            .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
            .unwrap_or(iced::Color::BLACK)
        })),
        border,
        shadow,
        ..Default::default()
    }
}

/// Check if an IcedStyle has visual properties that need container wrapping.
fn needs_visual_wrap(is: &IcedStyle) -> bool {
    is.background_color.is_some()
        || is.border
        || is.rounded
        || is.has_border_radius()
        || is.shadow
        || is.text_color.is_some()
}

/// Convert IcedFontWeight to iced::Font.
/// Plan 411 P1-C: Inter 字体内嵌——对齐 vue 栈字体族 `Inter, system-ui...`。
/// Inter 以 SIL OFL 1.1 发布(全文见 assets/INTER-OFL.txt),静态三字重
/// regular/medium/semibold 打进二进制;中文字形由 cosmic-text 回退系统字体。
pub const INTER_FONT_REGULAR: &[u8] = include_bytes!("../assets/Inter-Regular.ttf");
pub const INTER_FONT_MEDIUM: &[u8] = include_bytes!("../assets/Inter-Medium.ttf");
pub const INTER_FONT_SEMIBOLD: &[u8] = include_bytes!("../assets/Inter-SemiBold.ttf");

/// Plan 518：随包内嵌默认壁纸(脚本自制,免授权——生成器 scratch/p518)。
/// `builtin:` 虚拟方案经 load_image_bytes 消费;ricepaper = 默认
/// (stella 权威图同款暖米宣纸底),inkwash = 宣纸+水墨远山可选项
/// (设置壁纸栏键入 `builtin:inkwash`)。深浅主题共用浅色——stella dark
/// 实证壁纸与主题解耦。
pub const WALLPAPER_RICEPAPER: &[u8] = include_bytes!("../assets/wallpaper-ricepaper.jpg");
pub const WALLPAPER_INKWASH: &[u8] = include_bytes!("../assets/wallpaper-inkwash.jpg");

/// Inter family 引用 —— application 的 `.default_font` 与带字重文本共用。
pub const INTER_FONT: iced::Font = iced::Font {
    family: iced::font::Family::Name("Inter"),
    ..iced::Font::DEFAULT
};

pub(super) fn font_weight_to_iced(weight: &IcedFontWeight) -> iced::Font {
    // Plan 411 P1-C: 带字重的文本钉住 Inter family,内嵌三字重才会真正
    // 被选中(Medium 此前近似映射 Semibold,内嵌后恢复正身);bold/light
    // 超出内嵌字重由 cosmic-text 合成/回退。
    match weight {
        IcedFontWeight::Bold => iced::Font {
            family: INTER_FONT.family,
            weight: iced::font::Weight::Bold,
            ..Default::default()
        },
        IcedFontWeight::Medium => iced::Font {
            family: INTER_FONT.family,
            weight: iced::font::Weight::Medium,
            ..Default::default()
        },
        IcedFontWeight::Normal => INTER_FONT,
        IcedFontWeight::Light => iced::Font {
            family: INTER_FONT.family,
            weight: iced::font::Weight::Light,
            ..Default::default()
        },
        IcedFontWeight::ExtraLight => iced::Font {
            family: INTER_FONT.family,
            weight: iced::font::Weight::Thin,
            ..Default::default()
        },
        IcedFontWeight::SemiBold => iced::Font {
            family: INTER_FONT.family,
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        },
        // Plan 527 T5: 全字重档拆分(thin/extrabold/black 正身)
        IcedFontWeight::Thin => iced::Font {
            family: INTER_FONT.family,
            weight: iced::font::Weight::Thin,
            ..Default::default()
        },
        IcedFontWeight::ExtraBold => iced::Font {
            family: INTER_FONT.family,
            weight: iced::font::Weight::ExtraBold,
            ..Default::default()
        },
        IcedFontWeight::Black => iced::Font {
            family: INTER_FONT.family,
            weight: iced::font::Weight::Black,
            ..Default::default()
        },
    }
}

/// Wrap an iced element with external spacing for margin simulation.
/// Handles:
/// - `margin_top` (mt-*): external top spacing via container padding
/// - `margin_bottom` (mb-*): external bottom spacing via container padding
///   (Plan 448 对齐批: 此前被静默丢弃,VM 标题 mb-6 不生效导致与 vue 间距分歧)
/// - `margin_left` (ml-*): external left spacing via container padding
/// - `margin_right` (mr-*): external right spacing via container padding
/// - `mx-auto` (both flags): container fills remaining width, content centered
/// - `ml-auto` alone: container fills remaining width, content pushed right
/// - `mr-auto` alone: container fills remaining width, content pushed left
///
/// PLAN-619 R3: 四值经 `IcedStyle::effective_margin` 解析（单侧 > 轴 > 统一），
/// 因此 `mx-*`/`my-*`/`m-*` 与显式单侧同等生效——此前只读单侧字段，
/// `m-*` 族其余成员全部静默丢失。
pub(super) fn wrap_with_margin<M: Clone + Debug + 'static>(
    el: iced::Element<'static, M>,
    is: &IcedStyle,
) -> iced::Element<'static, M> {
    use iced::widget::container;
    let (top, right, bottom, left) = is.effective_margin();
    let needs_wrap = top != 0.0
        || bottom != 0.0
        || left != 0.0
        || right != 0.0
        || is.margin_left_auto
        || is.margin_right_auto;
    if !needs_wrap {
        return el;
    }
    let mut cont = container(el);
    if top != 0.0 || bottom != 0.0 || left != 0.0 || right != 0.0 {
        cont = cont.padding(iced::Padding {
            top,
            right,
            bottom,
            left,
        });
    }
    if is.margin_left_auto && is.margin_right_auto {
        // mx-auto: center horizontally
        cont = cont.width(iced::Length::Fill).center_x(iced::Length::Fill);
    } else if is.margin_left_auto {
        // ml-auto: push content to the right
        cont = cont
            .width(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Right);
    } else if is.margin_right_auto {
        // mr-auto: push content to the left
        cont = cont
            .width(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Left);
    }
    cont.into()
}

// ============================================================================
// Shared style helpers for Column, Row, Container
// ============================================================================
// These functions unify the styling/container-wrapping logic that was previously
// duplicated between IntoIcedElement::into_iced() and render_dynamic_view().
// Both paths call these helpers after rendering children their own way.

/// Apply style properties to a Column widget and optionally wrap in a Container
/// for visual styles (background, border) or vertical alignment (justify).
///
/// Takes a column with spacing set and children already pushed.
/// Returns the final styled element (possibly wrapped in container).
fn apply_column_style<M: Clone + Debug + 'static>(
    col: iced::widget::Column<'static, M>,
    padding: u16,
    style: Option<&Style>,
    widget_id: Option<String>,
    hover: Option<crate::ui::iced::hover_area::HoverFlag>,
) -> iced::Element<'static, M> {
    let iced_style = style.map(|s| IcedStyle::from_style(s));
    // PLAN-002 B：hover 变体类 → 第二套已构建样式（merged_with_variant
    // 覆盖 base，同按钮臂语义）；无 hover 声明 = None，走原路径。
    let hover_cs = style
        .filter(|s| s.has_variant(crate::ui::style::Variant::Hover))
        .map(|s| {
            build_container_style(&IcedStyle::merged_with_variant(
                s,
                crate::ui::style::Variant::Hover,
            ))
        });
    let has_visual = iced_style
        .as_ref()
        .map_or(false, |is| needs_visual_wrap(is))
        || hover_cs.is_some();
    let pd = iced_padding(padding, style);

    // Apply width/height/alignment to column
    let mut justify_center = false;
    let mut justify_end = false;
    let mut col = col;
    if let Some(ref is) = iced_style {
        // Width
        if let Some(ref w) = is.width {
            col = col.width(iced_length(w));
        }
        if let Some(mw) = is.max_width {
            if is.width.is_none() {
                col = col.width(iced::Length::Fill);
            }
            col = col.max_width(mw);
        }
        // PLAN-080 T-01: max-w-[N%] 不再强制 Fill——CSS max-width 语义是
        // 内容贴合 + 上限收窄（musk 消息行实机对拍实证：强制 Fill 令气泡
        // 恒占满 70% 宽，短消息也被撑开，与 vue 轨 content-hug 分叉）。
        // 收窄由尾部 MaxWidthPct 委托 widget 在 layout 期完成。
        // Height — skip when justify needs it on container instead
        let needs_v_align = matches!(
            is.justify_content,
            Some(IcedJustify::Center | IcedJustify::End)
        );
        if !needs_v_align {
            if let Some(ref h) = is.height {
                col = col.height(iced_length(h));
            } else if let Some(mh) = is.min_height {
                // min-h-screen (marker 9999.0) → Fill; other px values → Fixed
                if mh >= 9999.0 {
                    col = col.height(iced::Length::Fill);
                } else {
                    col = col.height(iced::Length::Fixed(mh));
                }
            }
        }
        // Alignment
        if let Some(ref a) = is.align_items {
            col = col.align_x(iced_alignment_horizontal(*a));
        }
        // Justify tracking
        if let Some(ref j) = is.justify_content {
            match j {
                IcedJustify::Center => justify_center = true,
                IcedJustify::End => justify_end = true,
                _ => {}
            }
        }
    }

    let col_max_width = iced_style.as_ref().and_then(|is| is.max_width);
    let col_max_width_pct = iced_style.as_ref().and_then(|is| is.max_width_pct);
    let needs_wrap = justify_center || justify_end || has_visual;
    // PLAN-619 R3: 单侧 > 轴 > 统一（`mx-*`/`my-*`/`m-*` 此前被丢弃）。
    let (mt, mr, mb, ml) = iced_style
        .as_ref()
        .map(|is| is.effective_margin())
        .unwrap_or((0.0, 0.0, 0.0, 0.0));
    let needs_margin_wrap = mt != 0.0
        || mb != 0.0
        || ml != 0.0
        || mr != 0.0
        || iced_style.as_ref().map_or(false, |is| {
            is.margin_left_auto || is.margin_right_auto || is.margin_top_auto
        });

    let el = if needs_wrap {
        // PLAN-642 T-12: overflow-hidden + justify-Center/End 列的 scroll 兜底。
        // iced 0.14 flex 对定高列的子项按"剩余量"逐个配给,超出列高的内容被
        // 压成 0×0 隐没(内嵌画廊 frame 内 demo 超出 720 的内容不可见且不可
        // 滚动——P2-009 根因;scratch 实测 h-[300px] frame 行 13+ 全塌缩)。
        // 此处把该类列的内容包进 Shrink 高度 Scrollable:内容矮于列高时被
        // 容器 center_y/align_y 垂直居中(016 自由尺寸 app 居中验收),高于
        // 列高时 Shrink 被定高封顶、内容完整布局并滚动(009 滚动验收)。
        // CSS 语义偏差:overflow-hidden ≈ overflow-y:auto(登记于计划)。
        // 作用域限定:仅 justify-Center/End(此处 needs_v_align 已把列高让渡给
        // 容器,scroll 以 Shrink 接管剩余量语义);非 justify 列自身的
        // height/Fill 语义与 Shrink scroll 组合会破坏布局(画廊根 h-screen
        // 实测整页塌缩),维持原状。
        let overflow_fallback = (justify_center || justify_end)
            && iced_style.as_ref().map_or(false, |is| {
                matches!(is.overflow_y, Some(IcedOverflow::Hidden))
            });
        let inner: iced::Element<'static, M> = if overflow_fallback {
            let col_el: iced::Element<'static, M> = col.into();
            let sc = iced::widget::scrollable(col_el)
                .width(iced::Length::Fill)
                .height(iced::Length::Shrink);
            sc.into()
        } else {
            col.into()
        };
        let mut cont = container(inner);
        cont = cont.padding(pd);
        if justify_center {
            let col_w = if let Some(ref is) = iced_style {
                if let Some(ref w) = is.width {
                    iced_length(w)
                } else {
                    iced::Length::Fill
                }
            } else {
                iced::Length::Fill
            };
            let col_h = if let Some(ref is) = iced_style {
                if let Some(ref h) = is.height {
                    iced_length(h)
                } else {
                    iced::Length::Fill
                }
            } else {
                iced::Length::Fill
            };
            cont = cont.width(col_w).height(col_h).center_y(col_h);
        } else if justify_end {
            let col_w = if let Some(ref is) = iced_style {
                if let Some(ref w) = is.width {
                    iced_length(w)
                } else {
                    iced::Length::Fill
                }
            } else {
                iced::Length::Fill
            };
            let col_h = if let Some(ref is) = iced_style {
                if let Some(ref h) = is.height {
                    iced_length(h)
                } else {
                    iced::Length::Fill
                }
            } else {
                iced::Length::Fill
            };
            cont = cont
                .width(col_w)
                .height(col_h)
                .align_y(iced::alignment::Vertical::Bottom);
        } else {
            // Non-justify wrap: propagate column's width and height to container
            if let Some(ref is) = iced_style {
                // PLAN-055: self-end 列的视觉容器抱合——CSS 里 flex 子项
                // align-self:flex-end 时收缩到内容宽并靠交叉轴末端。此前
                // visual-wrap 容器无条件 Fill（块级语义），chat 气泡
                // （bg-primary + self-end）被拉成全宽紫条、右对齐失效。
                // End → Shrink+align_x(Right)；Start/Center 同理；
                // 无 align_self 维持块级 Fill（AI 消息 hairline 全宽不回归）。
                match is.align_self {
                    Some(crate::ui::style_iced_adapter::IcedAlign::End) => {
                        cont = cont
                            .width(iced::Length::Shrink)
                            .align_x(iced::alignment::Horizontal::Right);
                    }
                    Some(crate::ui::style_iced_adapter::IcedAlign::Start) => {
                        cont = cont
                            .width(iced::Length::Shrink)
                            .align_x(iced::alignment::Horizontal::Left);
                    }
                    Some(crate::ui::style_iced_adapter::IcedAlign::Center) => {
                        cont = cont
                            .width(iced::Length::Shrink)
                            .align_x(iced::alignment::Horizontal::Center);
                    }
                    _ => {
                        let col_width_fill =
                            matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)))
                                || is.width.is_none();
                        if col_width_fill {
                            cont = cont.width(iced::Length::Fill);
                        } else if let Some(ref w) = is.width {
                            cont = cont.width(iced_length(w));
                        }
                        let col_height_fill =
                            matches!(is.height, Some(IcedSize::Full | IcedSize::FillPortion(_)))
                                || is.min_height.map_or(false, |mh| mh >= 9999.0);
                        if col_height_fill {
                            cont = cont.height(iced::Length::Fill);
                        } else if let Some(ref h) = is.height {
                            cont = cont.height(iced_length(h));
                        } else if let Some(mh) = is.min_height {
                            cont = cont.height(iced::Length::Fixed(mh));
                        }
                        if let Some(mw) = is.max_width {
                            cont = cont.max_width(mw);
                        }
                    }
                }
            }
        }
        // Apply visual styles (background, border, rounded, shadow)
        if let Some(ref is) = iced_style {
            if has_visual {
                let cs = build_container_style(is);
                cont = cont.style(layout_style_fn(cs, hover_cs, hover.clone()));
            } else if let Some(bg) = is.background_color {
                cont = cont.style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg)),
                    ..Default::default()
                });
            }
        }
        if let Some(id) = widget_id {
            cont = cont.id(id);
        }
        cont.into()
    } else if col_max_width.is_some() || col_max_width_pct.is_some() {
        let mut cont = container(col.padding(pd));
        if let Some(ref is) = iced_style {
            // PLAN-080 T-01: pct-only（无 px max-width、无显式 width）不
            // Fill——Shrink 抱合 + 尾部 pct 收窄 = CSS 语义；px max-width
            // 保留既有 Fill 块级语义（Container::max_width 收 Fill 子件）。
            let pct_only =
                col_max_width.is_none() && col_max_width_pct.is_some() && is.width.is_none();
            let col_width_fill = !pct_only
                && (matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)))
                    || is.width.is_none());
            if col_width_fill {
                cont = cont.width(iced::Length::Fill);
            } else if let Some(ref w) = is.width {
                cont = cont.width(iced_length(w));
            }
            let col_height_fill =
                matches!(is.height, Some(IcedSize::Full | IcedSize::FillPortion(_)))
                    || is.min_height.map_or(false, |mh| mh >= 9999.0);
            if col_height_fill {
                cont = cont.height(iced::Length::Fill);
            } else if let Some(ref h) = is.height {
                cont = cont.height(iced_length(h));
            } else if let Some(mh) = is.min_height {
                cont = cont.height(iced::Length::Fixed(mh));
            }
        }
        if let Some(mw) = col_max_width {
            cont = cont.max_width(mw);
        }
        if let Some(id) = widget_id {
            cont = cont.id(id);
        }
        cont.into()
    } else if let Some(id) = widget_id {
        // G4e (411 P2-B #1): plain (unstyled) columns dropped the widget
        // id, so the layout collector never saw them. Wrap in a
        // layout-transparent container carrying the id (container()
        // with no modifiers is a pass-through in iced layout).
        container(col.padding(pd)).id(id).into()
    } else {
        col.padding(pd).into()
    };

    let el = if needs_margin_wrap {
        let mut cont = container(el);
        if mt != 0.0 || mb != 0.0 || ml != 0.0 || mr != 0.0 {
            cont = cont.padding(iced::Padding {
                top: mt,
                right: mr,
                bottom: mb,
                left: ml,
            });
        }
        // Handle mx-auto / ml-auto / mr-auto / mt-auto
        if let Some(ref is) = iced_style {
            // PLAN-080: mt-auto 推底——列内自动上边距吃满剩余空间（CSS flex
            // 语义），iced 无原生承载，套 Fill 高 + align_y(Bottom) 容器。
            if is.margin_top_auto {
                cont = cont
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .align_y(iced::alignment::Vertical::Bottom);
            }
            if is.margin_left_auto && is.margin_right_auto {
                cont = cont.width(iced::Length::Fill).center_x(iced::Length::Fill);
            } else if is.margin_left_auto {
                cont = cont
                    .width(iced::Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right);
            } else if is.margin_right_auto {
                cont = cont
                    .width(iced::Length::Fill)
                    .align_x(iced::alignment::Horizontal::Left);
            }
        }
        cont.into()
    } else {
        el
    };
    // PLAN-077: max-w-[N%] 百分比上限——像素上限走 Container::max_width 快路径，
    // 百分比在最终元素最外层套委托 widget（layout 期按父级 offered 宽度收窄）。
    match col_max_width_pct {
        Some(pct) => {
            let capped: iced::Element<'static, M> =
                crate::ui::iced::max_width::MaxWidthPct::new(el, pct / 100.0).into();
            // PLAN-080 T-01: self-end/start/center 在 pct 臂的对齐抱合——
            // Fill 容器 + align_x（A6 items-end 同款模式）。此前 pct 分支
            // 丢 align_self（musk 用户消息行 self-end 失效贴左，实机对拍
            // x=429 vs vue 827 定罪）。
            match iced_style.as_ref().and_then(|is| is.align_self) {
                Some(crate::ui::style_iced_adapter::IcedAlign::End) => container(capped)
                    .width(iced::Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .into(),
                Some(crate::ui::style_iced_adapter::IcedAlign::Start) => container(capped)
                    .width(iced::Length::Fill)
                    .align_x(iced::alignment::Horizontal::Left)
                    .into(),
                Some(crate::ui::style_iced_adapter::IcedAlign::Center) => container(capped)
                    .width(iced::Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .into(),
                _ => capped,
            }
        }
        None => el,
    }
}

/// Emulate CSS `justify-content` on an iced Row, which has no native main-axis
/// justification. Returns `(leading, between, trailing)` spacer weights telling
/// the row builder where to inject `Space`-with-FillPortion spacers. Layout is
/// left-to-right, so Fill spacers compete for — and split — the row's
/// remaining width by portion weight:
///   start   → none
///   end     → leading spacer pushes children to the right edge
///   center  → leading + trailing spacers flank the children (equal halves)
///   between → a spacer between each adjacent pair spreads them out
///   around  → edges get half a gap, pairs a full gap: lead/trail=1, between=2
///             (with n children the total weight is 2+2(n-1)=2n → each unit is
///             W/(2n) so lead = W/(2n) = d/2 and between = W/n = d — exactly
///             CSS space-around)
///   evenly  → n+1 equal spacers including both edges → each = W/(n+1) = d
///             (exactly CSS space-evenly)
pub(super) fn row_justify_spacers(j: Option<IcedJustify>) -> (Option<u16>, Option<u16>, Option<u16>) {
    match j {
        Some(IcedJustify::Center) => (Some(1), None, Some(1)),
        Some(IcedJustify::End) => (Some(1), None, None),
        Some(IcedJustify::Between) => (None, Some(1), None),
        Some(IcedJustify::Around) => (Some(1), Some(2), Some(1)),
        Some(IcedJustify::Evenly) => (Some(1), Some(1), Some(1)),
        _ => (None, None, None), // Start / None
    }
}

/// Apply style properties to a Row widget and optionally wrap in a Container
/// for visual styles (background, border).
fn apply_row_style<M: Clone + Debug + 'static>(
    row: iced::widget::Row<'static, M>,
    padding: u16,
    style: Option<&Style>,
    widget_id: Option<String>,
    hover: Option<crate::ui::iced::hover_area::HoverFlag>,
) -> iced::Element<'static, M> {
    let iced_style = style.map(|s| IcedStyle::from_style(s));
    // PLAN-002 B：hover 变体类 → 第二套已构建样式（同 apply_column_style）。
    let hover_cs = style
        .filter(|s| s.has_variant(crate::ui::style::Variant::Hover))
        .map(|s| {
            build_container_style(&IcedStyle::merged_with_variant(
                s,
                crate::ui::style::Variant::Hover,
            ))
        });
    let has_visual = iced_style
        .as_ref()
        .map_or(false, |is| needs_visual_wrap(is))
        || hover_cs.is_some();
    let pd = iced_padding(padding, style);
    let row_max_width = iced_style.as_ref().and_then(|is| is.max_width);
    let row_max_width_pct = iced_style.as_ref().and_then(|is| is.max_width_pct);

    // Apply width and alignment to row
    let mut r = row;
    if let Some(ref is) = iced_style {
        if let Some(ref w) = is.width {
            r = r.width(iced_length(w));
        } else if is.justify_content.is_some() {
            // justify-content only has effect with a defined main-axis size;
            // the Fill-spacer emulation needs room to distribute.
            r = r.width(iced::Length::Fill);
        }
        if let Some(ref h) = is.height {
            r = r.height(iced_length(h));
        }
        if let Some(ref a) = is.align_items {
            r = r.align_y(iced_alignment_vertical(*a));
        }
    }

    let el = if has_visual {
        let mut cont = container(r);
        cont = cont.padding(pd);
        // Propagate row's width/height to wrapping container
        if let Some(ref is) = iced_style {
            let row_width_fill =
                matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)));
            if row_width_fill {
                cont = cont.width(iced::Length::Fill);
            } else if let Some(ref w) = is.width {
                cont = cont.width(iced_length(w));
            }
            let row_height_fill =
                matches!(is.height, Some(IcedSize::Full | IcedSize::FillPortion(_)));
            if row_height_fill {
                cont = cont.height(iced::Length::Fill);
            } else if let Some(ref h) = is.height {
                cont = cont.height(iced_length(h));
            }
        }
        if let Some(mw) = row_max_width {
            cont = cont.max_width(mw);
        }
        if let Some(ref is) = iced_style {
            let cs = build_container_style(is);
            cont = cont.style(layout_style_fn(cs, hover_cs, hover.clone()));
        }
        if let Some(id) = widget_id {
            cont = cont.id(id);
        }
        cont.into()
    } else if row_max_width.is_some() || row_max_width_pct.is_some() {
        r = r.padding(pd);
        let mut cont = container(r);
        // Propagate row's width/height to wrapping container
        if let Some(ref is) = iced_style {
            let row_width_fill =
                matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)));
            if row_width_fill {
                cont = cont.width(iced::Length::Fill);
            } else if let Some(ref w) = is.width {
                cont = cont.width(iced_length(w));
            }
            let row_height_fill =
                matches!(is.height, Some(IcedSize::Full | IcedSize::FillPortion(_)));
            if row_height_fill {
                cont = cont.height(iced::Length::Fill);
            } else if let Some(ref h) = is.height {
                cont = cont.height(iced_length(h));
            }
        }
        if let Some(mw) = row_max_width {
            cont = cont.max_width(mw);
        }
        if let Some(id) = widget_id {
            cont = cont.id(id);
        }
        cont.into()
    } else if let Some(id) = widget_id {
        // G4e: same as apply_column_style — keep the id on plain rows.
        container(r.padding(pd)).id(id).into()
    } else {
        r.padding(pd).into()
    };

    // Apply external margin_top/margin_bottom and mx-auto/ml-auto/mr-auto
    // PLAN-619 R3: 单侧 > 轴 > 统一（同 apply_column_style）。
    let (mt, mr, mb, ml) = iced_style
        .as_ref()
        .map(|is| is.effective_margin())
        .unwrap_or((0.0, 0.0, 0.0, 0.0));
    let needs_margin_wrap = mt != 0.0
        || mb != 0.0
        || ml != 0.0
        || mr != 0.0
        || iced_style.as_ref().map_or(false, |is| {
            is.margin_left_auto || is.margin_right_auto || is.margin_top_auto
        });
    let el = if needs_margin_wrap {
        let mut cont = container(el);
        if mt != 0.0 || mb != 0.0 || ml != 0.0 || mr != 0.0 {
            cont = cont.padding(iced::Padding {
                top: mt,
                right: mr,
                bottom: mb,
                left: ml,
            });
        }
        if let Some(ref is) = iced_style {
            if is.margin_left_auto && is.margin_right_auto {
                cont = cont.width(iced::Length::Fill).center_x(iced::Length::Fill);
            } else if is.margin_left_auto {
                cont = cont
                    .width(iced::Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right);
            } else if is.margin_right_auto {
                cont = cont
                    .width(iced::Length::Fill)
                    .align_x(iced::alignment::Horizontal::Left);
            }
        }
        cont.into()
    } else {
        el
    };
    // PLAN-077: max-w-[N%] 百分比上限（同 apply_column_style 尾部）。
    match row_max_width_pct {
        Some(pct) => crate::ui::iced::max_width::MaxWidthPct::new(el, pct / 100.0).into(),
        None => el,
    }
}

/// Apply style properties to a Container widget (width, height, centering, visual styles).
fn apply_container_style<M: Clone + Debug + 'static>(
    mut cont: iced::widget::Container<'static, M>,
    padding: u16,
    width: Option<u16>,
    height: Option<u16>,
    center_x: bool,
    center_y: bool,
    style: Option<&Style>,
    widget_id: Option<String>,
    hover: Option<crate::ui::iced::hover_area::HoverFlag>,
) -> iced::Element<'static, M> {
    let max_width_pct = style.and_then(|s| IcedStyle::from_style(s).max_width_pct);
    cont = cont.padding(iced_padding(padding, style));
    // PLAN-002 B：hover 变体类 → 第二套已构建样式（同 apply_column_style）。
    let hover_cs = style
        .filter(|s| s.has_variant(crate::ui::style::Variant::Hover))
        .map(|s| {
            build_container_style(&IcedStyle::merged_with_variant(
                s,
                crate::ui::style::Variant::Hover,
            ))
        });

    if let Some(ref s) = style {
        let is = IcedStyle::from_style(s);

        if center_x || center_y {
            // When centering, the container must fill its parent so it has room
            // to center the content. We apply width/height from style to this
            // container directly (not to a nested inner container).
            if center_x {
                match is.width {
                    Some(ref ws) => {
                        cont = cont.width(iced_length(ws));
                    }
                    None => {
                        cont = cont.width(iced::Length::Fill);
                    }
                }
                if let Some(mw) = is.max_width {
                    cont = cont.max_width(mw);
                }
                cont = cont.align_x(iced::alignment::Horizontal::Center);
            } else {
                if let Some(ref ws) = is.width {
                    cont = cont.width(iced_length(ws));
                } else if let Some(w) = width {
                    if w > 0 {
                        cont = cont.width(iced::Length::Fixed(w as f32));
                    }
                }
                if let Some(mw) = is.max_width {
                    cont = cont.max_width(mw);
                }
            }

            if center_y {
                match is.height {
                    Some(ref h) => {
                        cont = cont.height(iced_length(h));
                    }
                    None => {
                        cont = cont.height(iced::Length::Fill);
                    }
                }
                if let Some(mh) = is.max_height {
                    cont = cont.max_height(mh);
                }
                cont = cont.align_y(iced::alignment::Vertical::Center);
            } else {
                match is.height {
                    Some(ref h) => {
                        cont = cont.height(iced_length(h));
                    }
                    None => {
                        if let Some(h) = height {
                            if h > 0 {
                                cont = cont.height(iced::Length::Fixed(h as f32));
                            }
                        }
                    }
                }
                if let Some(mh) = is.max_height {
                    cont = cont.max_height(mh);
                }
            }
        } else {
            // Normal (non-centered) container
            if let Some(ref ws) = is.width {
                cont = cont.width(iced_length(ws));
            } else if let Some(w) = width {
                if w > 0 {
                    cont = cont.width(iced::Length::Fixed(w as f32));
                }
            } else if let Some(mw) = is.min_width {
                // PLAN-051 P2-④: min-w 消费（语义近似=Fixed 下限——内容更宽
                // 时 iced 不再撑大,自滚/截断场景正确;登记）。沿 Column 臂
                // 1524 同款：min-h-screen 哨兵(9999.0) → Fill。
                cont = cont.width(if mw >= 9999.0 {
                    iced::Length::Fill
                } else {
                    iced::Length::Fixed(mw)
                });
            }
            match is.height {
                Some(ref h) => {
                    cont = cont.height(iced_length(h));
                }
                None => {
                    if let Some(h) = height {
                        if h > 0 {
                            cont = cont.height(iced::Length::Fixed(h as f32));
                        }
                    } else if let Some(mh) = is.min_height {
                        // PLAN-051 P2-④: min-h 消费——musk input-compose 的
                        // min-h-20 此前被丢 → 输入框容器高度塌 0。
                        cont = cont.height(if mh >= 9999.0 {
                            iced::Length::Fill
                        } else {
                            iced::Length::Fixed(mh)
                        });
                    }
                }
            }
            if let Some(mw) = is.max_width {
                cont = cont.max_width(mw);
            }
            if let Some(mh) = is.max_height {
                cont = cont.max_height(mh);
            }
        }

        // Visual styles (background, border, rounded, shadow)
        if needs_visual_wrap(&is) || hover_cs.is_some() {
            let cs = build_container_style(&is);
            cont = cont.style(layout_style_fn(cs, hover_cs, hover.clone()));
        }
    } else {
        if let Some(w) = width {
            if w > 0 {
                cont = cont.width(iced::Length::Fixed(w as f32));
            }
        }
        if let Some(h) = height {
            if h > 0 {
                cont = cont.height(iced::Length::Fixed(h as f32));
            }
        }

        // No style but centering requested — fill parent and align center
        if center_x {
            cont = cont
                .width(iced::Length::Fill)
                .align_x(iced::alignment::Horizontal::Center);
        }
        if center_y {
            cont = cont
                .height(iced::Length::Fill)
                .align_y(iced::alignment::Vertical::Center);
        }
    }

    let el: iced::Element<'static, M> = if let Some(id) = widget_id {
        cont.id(id).into()
    } else {
        cont.into()
    };
    match max_width_pct {
        // PLAN-077: max-w-[N%] 百分比上限（同 col/row 臂尾部）——居中臂的
        // width(Fill) 先行吃满父宽，收窄由本委托 widget 在 layout 期完成。
        Some(pct) => crate::ui::iced::max_width::MaxWidthPct::new(el, pct / 100.0).into(),
        None => el,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Shared generic widget builders (Plan 319 — unify VM/Rust rendering).
//
// These sit one layer above the `apply_*_style` helpers: they take
// **already-built** child `Element`s and own the widget *shape* — spacing,
// padding, justify-spacers, width/height, children loop. Both converters
// (`into_iced`, generic `M`, rust mode; `render_dynamic_view`, IcedMessage,
// VM mode) call the *same* builder, so the shape can never drift between
// modes again. The converters differ only in HOW they produce the child
// elements: `into_iced` recurses via `child.into_iced()`, while
// `render_dynamic_view` recurses via itself (to give each child its own
// `wrap_debug` instrumentation + VM text capture). The builder is agnostic
// to that — it just arranges whatever children it's handed.
// ─────────────────────────────────────────────────────────────────────────

/// Build a Row from pre-built child elements, applying justify-spacers and
/// the shared `apply_row_style` (width/height/margin/visual wrap + id).
/// Plan 412: flex-row-reverse flips children; items-stretch wraps each child
/// in a height-Fill container (iced has no cross-axis stretch alignment).
/// Plan-050 T4/C2: 单侧边框的 1px 填充条模拟——推广 table_row_rule 先例
/// （iced 容器边框四边整圈,无法单侧;按类串 border-b/t/l/r 在对应方向拼一条
/// border-border 色 1px 线）。无单侧边框类时原样返回。
fn apply_side_borders<M: Clone + Debug + 'static>(
    el: iced::Element<'static, M>,
    is: Option<&IcedStyle>,
) -> iced::Element<'static, M> {
    let Some(is) = is else { return el };
    if !(is.border_bottom || is.border_top || is.border_left || is.border_right) {
        return el;
    }
    let border_col = is.border_color.unwrap_or_else(|| {
        let (r, g, b) = crate::ui::style_iced_adapter::resolve_border_rgb();
        iced::Color::from_rgb8(r, g, b)
    });
    // PLAN-054 T1: 单侧宽度档优先（border-l-N → side_border_width）；
    // 缺省回落整圈 border_width（border-N）再 1px。
    let border_w = is.side_border_width.or(is.border_width).unwrap_or(1.0);
    let line_style = move |_: &_| iced::widget::container::Style {
        background: Some(iced::Background::Color(border_col)),
        ..Default::default()
    };
    let hline = move || -> iced::Element<'static, M> {
        iced::widget::container(
            iced::widget::Space::new()
                .width(iced::Length::Fill)
                .height(iced::Length::Fixed(0.0)),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(border_w))
        .style(line_style)
        .into()
    };
    let vline = move || -> iced::Element<'static, M> {
        iced::widget::container(
            iced::widget::Space::new()
                .width(iced::Length::Fixed(0.0))
                .height(iced::Length::Fill),
        )
        .width(iced::Length::Fixed(border_w))
        .height(iced::Length::Fill)
        .style(line_style)
        .into()
    };
    let mut el = el;
    if is.border_top {
        el = iced::widget::Column::with_children(vec![hline(), el])
            .width(iced::Length::Fill)
            .into();
    }
    if is.border_bottom {
        el = iced::widget::Column::with_children(vec![el, hline()])
            .width(iced::Length::Fill)
            .into();
    }
    if is.border_left {
        el = iced::widget::Row::with_children(vec![vline(), el])
            .height(iced::Length::Fill)
            .into();
    }
    if is.border_right {
        el = iced::widget::Row::with_children(vec![el, vline()])
            .height(iced::Length::Fill)
            .into();
    }
    el
}

pub(super) fn build_row<M: Clone + Debug + 'static>(
    mut children: Vec<iced::Element<'static, M>>,
    spacing: u16,
    padding: u16,
    style: Option<&Style>,
    widget_id: Option<String>,
    hover: Option<crate::ui::iced::hover_area::HoverFlag>,
) -> iced::Element<'static, M> {
    let eff_spacing = effective_spacing(spacing, style, true);
    let iced_style = style.map(|s| IcedStyle::from_style(s));
    let justify = iced_style.as_ref().and_then(|is| is.justify_content);
    if iced_style.as_ref().map_or(false, |is| is.row_reverse) {
        children.reverse();
    }
    let (lead, between, trail) = row_justify_spacers(justify);
    let spacer =
        |portion: u16| iced::widget::Space::new().width(iced::Length::FillPortion(portion));
    let stretch = iced_style.as_ref().map_or(false, |is| is.items_stretch);
    // 行自身定高（非 Shrink 显式高；含 flex-1 在 Column 直接子位被
    // axis_fix_col_child 转写的 Height(Full)）→ StretchLine 有界入射时
    // 吃满上限。仅 auto 语义（行高=max 子项内容高）下，041 主行
    // （flex-1 items-stretch）这类定高行整行塌内容高，explorer/编辑区
    // 悬空半窗（Plan 655 回归，fix-stretch-fill）。
    let line_fill_height = iced_style
        .as_ref()
        .and_then(|is| is.height.as_ref())
        .map_or(false, |h| !matches!(h, IcedSize::Shrink));
    let row_widget = if stretch {
        // PLAN-655: 等高行走 StretchLine 两阶段布局（CSS align-items:stretch
        // 原语）。旧形态的 height:Fill 包装在无界祖先（scroll 内容臂）下
        // 塌缩 0 高（P642-D12，008 定价卡消失）；子项现保持原始形态，
        // 行高（=max 子项内容高；定高行=入射上限，见 line_fill_height）
        // 与子项拉伸由控件内部供给，justify 垫片照常并入序列（FillPortion
        // Space 主轴配给语义不变）。
        let mut items: Vec<iced::Element<'static, M>> = Vec::new();
        if let Some(p) = lead {
            items.push(spacer(p).into());
        }
        let mut first = true;
        for child in children {
            if let Some(p) = between {
                if !first {
                    items.push(spacer(p).into());
                }
            }
            first = false;
            items.push(child);
        }
        if let Some(p) = trail {
            items.push(spacer(p).into());
        }
        row([crate::ui::iced::stretch_line::stretch_line(
            items,
            eff_spacing as f32,
            line_fill_height,
        )
        .into()])
    } else {
        let mut row_widget = row([]).spacing(eff_spacing);
        if let Some(p) = lead {
            row_widget = row_widget.push(spacer(p));
        }
        let mut first = true;
        for child in children {
            if let Some(p) = between {
                if !first {
                    row_widget = row_widget.push(spacer(p));
                }
            }
            first = false;
            row_widget = row_widget.push(child);
        }
        if let Some(p) = trail {
            row_widget = row_widget.push(spacer(p));
        }
        row_widget
    };
    apply_side_borders(
        apply_row_style(row_widget, padding, style, widget_id, hover),
        iced_style.as_ref(),
    )
}

/// Build a Column from pre-built child elements + shared `apply_column_style`.
/// Plan 412: flex-col-reverse flips children; items-stretch wraps each child
/// in a width-Fill container; justify-between/around/evenly get vertical
/// Fill spacers (mirroring build_row's spacer emulation; Center/End keep the
/// container-wrap path inside apply_column_style).
pub(crate) fn build_column<M: Clone + Debug + 'static>(
    mut children: Vec<iced::Element<'static, M>>,
    spacing: u16,
    padding: u16,
    style: Option<&Style>,
    widget_id: Option<String>,
    hover: Option<crate::ui::iced::hover_area::HoverFlag>,
) -> iced::Element<'static, M> {
    let eff_spacing = effective_spacing(spacing, style, false);
    let iced_style = style.map(|s| IcedStyle::from_style(s));
    if iced_style.as_ref().map_or(false, |is| is.col_reverse) {
        children.reverse();
    }
    let justify = iced_style.as_ref().and_then(|is| is.justify_content);
    let distributed = matches!(
        justify,
        Some(IcedJustify::Between | IcedJustify::Around | IcedJustify::Evenly)
    );
    let stretch = iced_style.as_ref().map_or(false, |is| is.items_stretch);
    // PLAN-054 T4 (A6): items-end —— 列交叉轴(横向)右对齐:每个子项包进
    // Fill+align_x(End) 容器(iced 列无子项对齐;musk 用户消息列
    // `items-end` 此前被静默丢弃,气泡与 header 贴左)。仅 End 臂新增,
    // Start/Center 维持既有(忽略)行为,零回归面。
    let align_end = iced_style.as_ref().map_or(false, |is| {
        is.align_items == Some(crate::ui::style_iced_adapter::IcedAlign::End)
    });
    let spacer =
        |portion: u16| iced::widget::Space::new().height(iced::Length::FillPortion(portion));
    let mut col_widget = column([]).spacing(eff_spacing);
    let mut first = true;
    for child in children {
        if distributed {
            if let Some(p) = match justify {
                Some(IcedJustify::Between) if !first => Some(1),
                Some(IcedJustify::Around) => Some(if first { 1 } else { 2 }),
                Some(IcedJustify::Evenly) => Some(1),
                _ => None,
            } {
                col_widget = col_widget.push(spacer(p));
            }
        }
        first = false;
        let child = if stretch {
            container(child).width(iced::Length::Fill).into()
        } else if align_end {
            container(child)
                .width(iced::Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .into()
        } else {
            child
        };
        col_widget = col_widget.push(child);
    }
    if distributed {
        // Trailing spacer for around (half gap) and evenly (full gap);
        // between has no edge spacers.
        match justify {
            Some(IcedJustify::Around) | Some(IcedJustify::Evenly) => {
                col_widget = col_widget.push(spacer(1));
            }
            _ => {}
        }
    }
    // Plan 057 续(块高度修复):删除 build_column 尾部的 max-h → Fixed(N) 限高
    // Scrollable 包裹。它曾让短内容也占满 N 高(用户反馈的"空 block 半页空白"),
    // 且与 view builder 的 needs_scroll(overflow-y-auto → View::Scrollable →
    // build_scrollable 的 Shrink + Container::max_height 封顶)形成双重包裹:
    // 内层 col 同样携带 max-h 样式,此处再包 Fixed(N) 把 intrinsic 钉死在 N,
    // 外层 Shrink 解析为 min(N, N) = N,Shrink 封顶完全失效。max-h 现在统一由
    // build_scrollable 的 cap 分支处理(CSS max-height 语义)。
    apply_side_borders(
        apply_column_style(col_widget, padding, style, widget_id, hover),
        iced_style.as_ref(),
    )
}

/// Build a Container around a single pre-built child + shared
/// `apply_container_style`.
pub(super) fn build_container<M: Clone + Debug + 'static>(
    child: iced::Element<'static, M>,
    padding: u16,
    width: Option<u16>,
    height: Option<u16>,
    center_x: bool,
    center_y: bool,
    style: Option<&Style>,
    widget_id: Option<String>,
    hover: Option<crate::ui::iced::hover_area::HoverFlag>,
) -> iced::Element<'static, M> {
    let cont = container(child);
    let iced_style = style.map(IcedStyle::from_style);
    apply_side_borders(
        apply_container_style(
            cont, padding, width, height, center_x, center_y, style, widget_id, hover,
        ),
        iced_style.as_ref(),
    )
}

/// Plan 043 T1: Scrollable offset 绑定写入的 pending 队列。
///
/// build 面（view→element）无法返回 iced Task，写入请求先在此登记，
/// update 面（`update_inner` 尾部）排空并发射 `operation::scroll_to`——
/// devtools.pending_scroll_to_center 同款「pending + update 排空」先例。
/// 去抖：仅当绑定值与该 id 上次请求值之差超 0.5px 才入队，同帧重复 build /
/// 无变化重渲染不重复触发。键 = scrollable widget id（写入要求稳定 id）。
pub(super) struct PendingScrollOffsetEntry {
    pub(super) last_requested: (f32, f32),
    pub(super) pending: Option<(f32, f32)>,
}

pub(super) fn pending_scroll_offsets(
) -> &'static std::sync::Mutex<std::collections::HashMap<String, PendingScrollOffsetEntry>> {
    static REG: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, PendingScrollOffsetEntry>>,
    > = std::sync::OnceLock::new();
    REG.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// build 面登记：绑定值变化（>0.5px）才入队。
pub(crate) fn note_scroll_offset(id: &str, offset: (f32, f32)) {
    let mut reg = pending_scroll_offsets().lock().unwrap();
    let entry = reg
        .entry(id.to_string())
        .or_insert(PendingScrollOffsetEntry {
            last_requested: offset,
            pending: Some(offset),
        });
    let (dx, dy) = (
        offset.0 - entry.last_requested.0,
        offset.1 - entry.last_requested.1,
    );
    if dx.abs() > 0.5 || dy.abs() > 0.5 {
        entry.last_requested = offset;
        entry.pending = Some(offset);
    }
}

/// update 面排空：取走全部待写入项（id → 绝对像素偏移）。
pub(crate) fn drain_pending_scroll_offsets() -> Vec<(String, (f32, f32))> {
    let mut reg = pending_scroll_offsets().lock().unwrap();
    reg.iter_mut()
        .filter_map(|(id, e)| e.pending.take().map(|off| (id.clone(), off)))
        .collect()
}

/// Build a Scrollable around a single pre-built child. Width/height come
/// from style (preferred) or the legacy numeric fields; id is set when the
/// caller supplies one (VM path injects the aura id for bounds collection).
/// Plan 043 T1: offset/on_scroll 为滚动位置写入/读出双臂——写入经 pending
/// 队列（要求 widget_id 稳定），读出经 iced on_scroll 事件回调。
/// PLAN-656 T-05: axes → iced Direction 三值（hidden 用 `Scrollbar::hidden()`
/// 结构性归零 rail 命中区，非仅视觉透明）；controller 经
/// ui::scroll::controller 注册表绑定稳定 id 并缓存测量（update 期
/// drain_resolved_intents 排空消费）。
///
/// PLAN-781 T-05：pane 内容为 managed 桥（子树含 ManagedScrollContent）
/// 判定——controller pane build 期据此调 mark_controller_managed（9922
/// to_child 的 unsupported 合同）。有界递归穿透包装变体
/// （Container/Column/Row/AnchorSlot/ScrollAnchorMark/MouseArea——
/// VM 轨 aura→View 转换的包装现实）；**不进嵌套 Scrollable**——内层
/// pane 的 managed 内容归属内层 controller，不得误标外层。
pub(super) fn content_is_managed_bridge<M: Clone + Debug + 'static>(v: &AbstractView<M>) -> bool {
    fn walk<M: Clone + Debug + 'static>(v: &AbstractView<M>, depth: u8) -> bool {
        if depth == 0 {
            return false;
        }
        match v {
            AbstractView::ManagedScrollContent { .. } => true,
            AbstractView::Container { child, .. }
            | AbstractView::AnchorSlot { child, .. }
            | AbstractView::ScrollAnchorMark { child, .. }
            | AbstractView::MouseArea { content: child, .. } => walk(child, depth - 1),
            AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
                children.iter().any(|c| walk(c, depth - 1))
            }
            _ => false,
        }
    }
    walk(v, 6)
}
#[allow(clippy::too_many_arguments)]
pub(super) fn build_scrollable<M: Clone + Debug + 'static>(
    child: iced::Element<'static, M>,
    width: Option<u16>,
    height: Option<u16>,
    style: Option<&Style>,
    widget_id: Option<String>,
    offset: Option<(f32, f32)>,
    on_scroll: Option<crate::ui::view::ScrollCallback<M>>,
    axes: crate::ui::scroll::ScrollAxes,
    scrollbar_policy: crate::ui::scroll::ScrollbarPolicy,
    controller: Option<&crate::ui::view::ScrollControllerBinding>,
) -> iced::Element<'static, M> {
    if let (Some(id), Some(off)) = (&widget_id, offset) {
        if std::env::var("P043_DEBUG").is_ok() {
            eprintln!("[P043-NOTE] id={id} off={off:?}");
        }
        note_scroll_offset(id, off);
    }
    // PLAN-656: controller pane 挂句柄派生稳定 id（跨重建不变、每 controller
    // 唯一）并注册绑定；scroll_to 意图排空时经注册表回找该 id。
    let controller_widget_id = controller.map(|c| format!("scroll_ctl_{}", c.key()));
    if let (Some(c), Some(id)) = (controller, &controller_widget_id) {
        crate::ui::scroll::bind_controller(c.key(), id);
        // F-4 终段：写臂重发注册表 offset（terminal/015 生产先例）。offset 在
        // 同构重建下由 iced Tree 状态保持（obs pane 无写臂跨重建保持滚动位，
        // 截图实证）；重发是结构性重建/极端序列下的防御，同值去抖使其在
        // 常规路径零开销。
        let snap = crate::ui::scroll::controller::controller_snapshot(c.key());
        // L0 fix(2026-10-11)：回显仅限写臂投影值（offset_from_measure=false——
        // 结构性重建后 scroll_to 未落盘的防御补发，terminal/015 先例语义）。
        // 读臂实测值（用户滚轮/拖拽落点）注册表已是真源，回显只会把滚动
        // 期间的滞后快照强行写回——auto-edit diff 视图大幅滚轮后滚动条
        // 抖动格斗源。实测值永不回显。
        if snap.viewport_w > 0.0 && !snap.offset_from_measure {
            note_scroll_offset(id, (snap.offset_x as f32, snap.offset_y as f32));
        }
    }
    let mut cap: Option<f32> = None;
    let mut s = scrollable(child);
    // PLAN-656: axis 三值 + hidden 策略（结构性 `Scrollbar::hidden()`：
    // width 0 → rail total_bounds 零面积 → 无命中目标；wheel/keyboard/
    // controller 滚动不受影响）。
    let hidden = scrollbar_policy == crate::ui::scroll::ScrollbarPolicy::Hidden;
    let make_scrollbar = || {
        if hidden {
            scrollable::Scrollbar::hidden()
        } else {
            scrollable::Scrollbar::new()
        }
    };
    s = match axes {
        crate::ui::scroll::ScrollAxes::Y => s, // iced 默认纵向
        crate::ui::scroll::ScrollAxes::X => {
            s.direction(scrollable::Direction::Horizontal(make_scrollbar()))
        }
        _other => s.direction(scrollable::Direction::Both {
            horizontal: make_scrollbar(),
            vertical: make_scrollbar(),
        }),
    };
    // PLAN-656: 读出臂——controller pane 的测量缓存包装由（M 具体化的）
    // 动态渲染臂合成（泛型路径无法凭空构造 M；其 controller 无用户
    // on-scroll 时不缓存测量，见 plan r2 执行注记）。
    if let Some(cb) = on_scroll {
        // Viewport 六测量 → ScrollMetrics → 宿主消息（DSL 侧三测量
        // scrollTop/scrollHeight/clientHeight 对应 offset_y/content_h/viewport_h）。
        s = s.on_scroll(move |vp: scrollable::Viewport| {
            let abs = vp.absolute_offset();
            let b = vp.bounds();
            let cbounds = vp.content_bounds();
            cb.call(crate::ui::view::ScrollMetrics {
                offset_x: abs.x,
                offset_y: abs.y,
                viewport_w: b.width,
                viewport_h: b.height,
                content_w: cbounds.width,
                content_h: cbounds.height,
            })
        });
    }
    if let Some(ref st) = style {
        let is = IcedStyle::from_style(st);
        // 2026-08-22(max-h 封顶修复):cap 判定从 is.height 的 None 分支
        // 提出,与 height 推断解耦 —— from_style 对 `w-full + overflow-y`
        // 会推断 height=Fill("竖向填充意图"),旧逻辑只在没有 height 时才
        // 封顶,于是 max-h-[400px] 的代码块长到内容高(实测 3058px)、内部
        // 滚动永不出现。CSS max-height 语义:有 cap 一律 Shrink +
        // Container::max_height 封顶(短内容收缩、长内容滚动),显式
        // h-*/Fill 让位。
        if let Some(mh) = is.max_height {
            if mh > 0.0 {
                cap = Some(mh);
            }
        }
        if let Some(ref ws) = is.width {
            match ws {
                IcedSize::Fixed(f) => s = s.width(iced::Length::Fixed(*f as f32)),
                // PLAN-663: Screen 视口单位缺省=Fill(满窗);嵌入边界内已在
                // 渲染前被 rewrite_viewport_units 重写为 Fixed(边界)。
                IcedSize::Full | IcedSize::Screen => s = s.width(iced::Length::Fill),
                IcedSize::FillPortion(n) => s = s.width(iced::Length::FillPortion(*n)),
                IcedSize::Shrink => s = s.width(iced::Length::Shrink),
                IcedSize::Percent(v) => {
                    s = s.width(iced::Length::FillPortion(percent_fill_units(*v)))
                }
            }
        } else if let Some(w) = width {
            if w > 0 {
                s = s.width(iced::Length::Fixed(w as f32));
            }
        }
        match is.height {
            Some(IcedSize::Fixed(f)) => {
                if cap.is_none() {
                    s = s.height(iced::Length::Fixed(f as f32));
                }
            }
            Some(IcedSize::Full | IcedSize::Screen) => {
                if cap.is_none() {
                    s = s.height(iced::Length::Fill);
                }
            }
            Some(IcedSize::FillPortion(n)) => {
                if cap.is_none() {
                    s = s.height(iced::Length::FillPortion(n));
                }
            }
            Some(IcedSize::Shrink) => {
                if cap.is_none() {
                    s = s.height(iced::Length::Shrink);
                }
            }
            Some(IcedSize::Percent(v)) => {
                if cap.is_none() {
                    s = s.height(iced::Length::FillPortion(percent_fill_units(v)));
                }
            }
            None => {
                if cap.is_none() {
                    if let Some(h) = height {
                        if h > 0 {
                            s = s.height(iced::Length::Fixed(h as f32));
                        }
                    }
                }
            }
        }
    } else {
        if let Some(w) = width {
            if w > 0 {
                s = s.width(iced::Length::Fixed(w as f32));
            }
        }
        if let Some(h) = height {
            if h > 0 {
                s = s.height(iced::Length::Fixed(h as f32));
            }
        }
    }
    // Plan 409 §10 续 4: 半透明悬浮滚动条(接近 vue:thumb 半透明、track 透明、细)。
    s = s.style(|_theme: &iced::Theme, _status: scrollable::Status| scrollbar_style());
    // PLAN-656: controller pane 的句柄派生 id 优先（scroll_to 目标必须命中；
    // 该节点让出 bounds 采集 id，与 auto_scroll 固定 id 同款取舍）。review
    // F-4 修复：原 `.or()` 顺序写反——vnode bounds id 优先吞掉 controller
    // id，注册表绑定的 scroll_ctl_* 永远匹配不到真实 widget（读回键为
    // vnode_*，scroll_to 落空，预热循环空转）。
    let widget_id = controller_widget_id.or(widget_id);
    if let Some(id) = widget_id {
        s = s.id(id);
    }
    if let Some(max_h) = cap {
        let shrink = s.height(iced::Length::Shrink);
        return iced::widget::container(shrink)
            .max_height(max_h)
            .width(iced::Length::Fill)
            .into();
    }
    s.into()
}

/// Plan 409 §10 续 4: vue 风格滚动条 — thumb 半透明、track 透明、细圆角。
pub(super) fn scrollbar_style() -> scrollable::Style {
    // PLAN-024:thumb 回归 3px(022 现款)——全局圆 thumb 在非角落
    // pane 视觉突兀(用户实机门 2026-09-19);右下角 pane 的 thumb 底角
    // 圆化挂 prop 化待办(见 024 §10,涉 .at 语言层 prop)。
    let border = iced::Border {
        width: 0.0,
        radius: iced::border::Radius::new(3.0),
        color: iced::Color::TRANSPARENT,
    };
    let thumb = iced::Background::Color(iced::Color::from_rgba(0.9, 0.9, 0.9, 0.3));
    let rail = scrollable::Rail {
        background: None,
        border,
        scroller: scrollable::Scroller {
            background: thumb,
            border,
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: iced::Background::Color(iced::Color::TRANSPARENT),
            border,
            shadow: iced::Shadow::default(),
            icon: iced::Color::TRANSPARENT,
        },
    }
}

/// Plan 483: text_input 的稳定唯一 Id 派生。主键 = on_change 的
/// widget+event(`.at` 单一真源中静态稳定、跨重建不变、同参输入亦互异);
/// 无主键时退 placeholder+width+password 三元组(泛型路径,与旧行为持平)。
/// 取代「全窗固定 prompt_input」——iced `Focus` operation 对**所有**同 Id
/// 的 focusable 逐个置焦(focusable.rs),共享 Id 即一次聚焦全置焦:
/// 未捕获 Tab→__focus_prompt / __focus_input / refocus 等路径任一触发,
/// 多 input 视图即双焦点+键盘双投递(上游 auto-musk 011)。
pub(super) fn derive_input_id(
    primary: Option<(&str, &str)>,
    placeholder: &str,
    width: Option<u16>,
    password: bool,
) -> iced::widget::Id {
    match primary {
        Some((widget, event)) => format!("auto_input_{}_{}", widget, event).into(),
        None => format!(
            "auto_input_{}_{}_{}",
            placeholder,
            width.unwrap_or(0),
            password
        )
        .into(),
    }
}

/// Build a `TextInput` shape: placeholder, value, and width. The width logic
/// follows the (formerly VM-mode) `render_dynamic_view` semantics as the
/// canonical single source — style width wins (Full→Fill, Fixed→Fixed), else
/// the legacy numeric `width`, else Shrink. The caller still wires
/// `on_input`/`on_submit`, since text-capture wiring is inherently mode-
/// specific (generic `M` vs `IcedMessage`).
pub(super) fn build_input_shape<M: Clone + Debug + 'static>(
    placeholder: &str,
    value: &str,
    width: Option<u16>,
    password: bool,
    style: Option<&Style>,
) -> iced::widget::TextInput<'static, M> {
    let mut input_widget = text_input(placeholder, value);
    // PLAN-050/483: 无显式 Id 的 text_input 在同视图多输入场景共享编辑器
    // 状态。泛型路径(无 widget+event 主键可取)以 placeholder+width+
    // password 三元组派生稳定 Id;VM 主路径在 render_dynamic_view 以
    // widget+event 主键再派生(见 derive_input_id)。
    input_widget = input_widget.id(derive_input_id(None, placeholder, width, password));
    if password {
        input_widget = input_widget.secure(true);
    }
    let default_style = Style::parse("border rounded-md px-3 py-2 text-sm").ok();
    let effective_style = style.or(default_style.as_ref());
    if let Some(s) = effective_style {
        let iced_style = IcedStyle::from_style(s);
        let effective_width = iced_style
            .width
            .map(|w| match w {
                IcedSize::Fixed(f) => Some(f as u16),
                IcedSize::Full | IcedSize::Screen => None,
                IcedSize::FillPortion(_) => None,
                IcedSize::Shrink => None,
                IcedSize::Percent(_) => None,
            })
            .unwrap_or(width);
        if let Some(w) = effective_width {
            if w > 0 {
                input_widget = input_widget.width(iced::Length::Fixed(w as f32));
            }
        }
        if let Some(ref w) = iced_style.width {
            if matches!(w, IcedSize::Full) && width.is_none() {
                input_widget = input_widget.width(iced::Length::Fill);
            }
        }
        // Size / typography
        if let Some(fs) = effective_font_size(&iced_style) {
            input_widget = input_widget.size(fs);
        } else {
            input_widget = input_widget.size(14.0);
        }
        // Padding
        if let Some(pad) = iced_style.padding {
            input_widget = input_widget.padding(pad);
        } else if iced_style.padding_x.is_some() || iced_style.padding_y.is_some() {
            let px = iced_style.padding_x.unwrap_or(12.0);
            let py = iced_style.padding_y.unwrap_or(8.0);
            input_widget = input_widget.padding(iced::Padding {
                top: py,
                right: px,
                bottom: py,
                left: px,
            });
        } else {
            input_widget = input_widget.padding(iced::Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0,
            });
        }

        let bg = iced_style
            .background_color
            .unwrap_or(iced::Color::TRANSPARENT);
        let border_color = iced_style.border_color.unwrap_or_else(|| {
            let (r, g, b) = crate::ui::style_iced_adapter::resolve_border_rgb();
            iced::Color::from_rgb8(r, g, b)
        });
        let border_width =
            iced_style
                .border_width
                .unwrap_or(if iced_style.border { 1.0 } else { 1.0 });
        let radius = iced_style.border_radius.unwrap_or(6.0);
        let value_color = iced_style.text_color.unwrap_or_else(|| {
            crate::ui::style_iced_adapter::resolve_semantic_rgb(
                &crate::ui::style::Color::OnBackground,
            )
            .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
            .unwrap_or(iced::Color::WHITE)
        });
        let placeholder_color = crate::ui::style_iced_adapter::resolve_semantic_rgb(
            &crate::ui::style::Color::OnSurface,
        )
        .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
        .unwrap_or(iced::Color::from_rgba(0.6, 0.6, 0.6, 0.7));

        let focus_border_color =
            crate::ui::style_iced_adapter::resolve_semantic_rgb(&crate::ui::style::Color::Primary)
                .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                .unwrap_or(iced::Color::from_rgb8(59, 130, 246));
        let focus_border_width = (border_width + 1.0).max(2.0);

        input_widget = input_widget.style(move |_theme, status| {
            let is_focused = matches!(status, iced::widget::text_input::Status::Focused { .. });
            let active_border_color = if is_focused {
                focus_border_color
            } else if border_width > 0.0 {
                border_color
            } else {
                iced::Color::TRANSPARENT
            };
            let active_border_width = if is_focused {
                focus_border_width
            } else {
                border_width
            };
            iced::widget::text_input::Style {
                background: iced::Background::Color(bg),
                border: iced::Border {
                    color: active_border_color,
                    width: active_border_width,
                    radius: radius.into(),
                },
                icon: iced::Color::TRANSPARENT,
                placeholder: placeholder_color,
                value: value_color,
                selection: iced::Color::from_rgba(0.3, 0.5, 0.9, 0.4),
            }
        });
    } else if let Some(w) = width {
        if w > 0 {
            input_widget = input_widget.width(iced::Length::Fixed(w as f32));
        }
    }
    input_widget
}

/// Plan 409 §10 续 5: Overlay 浮层定位 —— Fill 宽 + 水平对齐(right/left)+
/// top spacer(top offset)。content 自带 style(bg-popover/border/shadow)。
fn build_floating_layer<M: Clone + Debug + 'static>(
    content: iced::Element<'static, M>,
    position: crate::ui::view::OverlayPosition,
) -> iced::Element<'static, M> {
    build_floating_layer_capture(content, position, None)
}

/// `capture=None` = 默认 opaque（交互浮层既有语义）；`Some(false)` =
/// 浮层根带 pointer-events-none（被动框不截获下方点选）。
fn build_floating_layer_capture<M: Clone + Debug + 'static>(
    content: iced::Element<'static, M>,
    position: crate::ui::view::OverlayPosition,
    capture: Option<bool>,
) -> iced::Element<'static, M> {
    build_floating_layer_full(content, position, capture.unwrap_or(true), (None, None))
}

/// PLAN-095 T-03: 百分比浮层装配（musk canvas 框形态：left/top/width/height
/// 全 %）。Fill 容器内 [spacer(top), row[spacer(left), content(w%), filler],
/// filler(bottom)] —— FillPortion 三段=精确百分比份额；缺配对单元的轴按
/// 0/自然尺寸退化（不半猜半偏移）。capture=false（pointer-events-none 根）
/// 不包 opaque：被动框不截获下方点选（AC-04）。
#[allow(clippy::too_many_arguments)]
fn percent_floating_layer<M: Clone + Debug + 'static>(
    content: iced::Element<'static, M>,
    position: crate::ui::view::OverlayPosition,
    capture: bool,
    content_units: (Option<u16>, Option<u16>),
) -> iced::Element<'static, M> {
    let (w_units, h_units) = content_units;
    let top_units = match position.top {
        Some(crate::ui::view::OverlayLength::Percent(t)) if t > 0.0 && h_units.is_some() => {
            Some(percent_fill_units(t))
        }
        _ => None,
    };
    let left_units = match position.left {
        Some(crate::ui::view::OverlayLength::Percent(l)) if l > 0.0 && w_units.is_some() => {
            Some(percent_fill_units(l))
        }
        _ => None,
    };
    let top_px = match position.top {
        Some(crate::ui::view::OverlayLength::Px(v)) => v,
        _ => 0.0,
    };
    let opaque_content = |content: iced::Element<'static, M>| {
        if capture {
            iced::widget::opaque(content)
        } else {
            content
        }
    };
    let mut col = iced::widget::Column::<M>::new()
        .width(iced::Length::Fill)
        .height(if top_units.is_some() {
            iced::Length::Fill
        } else {
            iced::Length::Shrink
        });
    if let Some(tu) = top_units {
        col = col.push(iced::widget::Space::new().height(iced::Length::FillPortion(tu)));
    }
    let mut mid = iced::widget::Row::<M>::new()
        .width(iced::Length::Fill)
        .height(match (h_units, top_units) {
            (Some(hu), Some(_)) => iced::Length::FillPortion(hu),
            _ => iced::Length::Shrink,
        });
    if let Some(lu) = left_units {
        mid = mid.push(iced::widget::Space::new().width(iced::Length::FillPortion(lu)));
    }
    mid = mid.push(opaque_content(content));
    if let (Some(lu), Some(wu)) = (left_units, w_units) {
        let filler = 1000u16.saturating_sub(lu).saturating_sub(wu);
        if filler > 0 {
            mid = mid.push(iced::widget::Space::new().width(iced::Length::FillPortion(filler)));
        }
    }
    col = col.push(mid);
    if let (Some(tu), Some(hu)) = (top_units, h_units) {
        let filler = 1000u16.saturating_sub(tu).saturating_sub(hu);
        if filler > 0 {
            col = col.push(iced::widget::Space::new().height(iced::Length::FillPortion(filler)));
        }
    }
    let col = col.padding(iced::Padding {
        top: top_px,
        ..iced::Padding::ZERO
    });
    col.into()
}

#[cfg(test)]
mod plan732_wikilink_payload_tests {
    use super::*;

    /// PLAN-732 L 组（VM 桥腿）：Typed args 双 Str 经 `from_dynamic` →
    /// `encode_payload` 编码进 IcedMessage.event（Send 边界）→ 派发侧
    /// `decode_payload` 还原——`.OpenWikiLink(str, str)` 形参绑定口径
    ///（499 M2 mouse-area 坐标实参同路；空锚 = 空串占位，非缺席）。
    #[test]
    fn dual_str_payload_crosses_send_boundary() {
        let msg = DynamicMessage::Typed {
            widget_name: "Editor".into(),
            event_name: "OpenWikiLink".into(),
            args: vec![
                auto_val::Value::Str("目标页".into()),
                auto_val::Value::Str("锚点甲".into()),
            ],
        };
        let iced_msg = IcedMessage::from_dynamic(&msg);
        assert_eq!(iced_msg.widget, "Editor");
        let (clean, args) = crate::ui::dynamic::decode_payload(&iced_msg.event);
        assert_eq!(clean, "OpenWikiLink", "事件名与 PAYLOAD_SEP 不冲突");
        assert_eq!(args.len(), 2, "双实参到齐");
        assert!(
            matches!(&args[0], auto_val::Value::Str(s) if s.as_str() == "目标页"),
            "{args:?}"
        );
        assert!(
            matches!(&args[1], auto_val::Value::Str(s) if s.as_str() == "锚点甲"),
            "{args:?}"
        );

        // 无锚变体：空串实参在位（decode 后 args[1] = Str("")，非空缺）。
        let msg = DynamicMessage::Typed {
            widget_name: "Editor".into(),
            event_name: "OpenWikiLink".into(),
            args: vec![
                auto_val::Value::Str("无锚页".into()),
                auto_val::Value::Str(String::new().into()),
            ],
        };
        let iced_msg = IcedMessage::from_dynamic(&msg);
        let (clean, args) = crate::ui::dynamic::decode_payload(&iced_msg.event);
        assert_eq!(clean, "OpenWikiLink");
        assert!(
            matches!(&args[1], auto_val::Value::Str(s) if s.as_str().is_empty()),
            "{args:?}"
        );
    }
}

#[cfg(all(test, feature = "autodown"))]
mod plan737_mcp_link_dispatch_tests {
    use super::*;
    use crate::ui::autodown_editor as ade;
    use crate::ui::autodown_editor::DocInput;
    use crate::ui::code_editor::core::{EditorButton, NullClipboard};

    const WHITE: crate::ui::code_editor::theme::Rgba = crate::ui::code_editor::theme::Rgba {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };

    fn run_fs<R>(f: impl FnOnce(&mut cosmic_text::FontSystem) -> R) -> R {
        static FS: std::sync::OnceLock<std::sync::RwLock<cosmic_text::FontSystem>> =
            std::sync::OnceLock::new();
        crate::ui::code_editor::core::set_font_system_call(|with| {
            let mut guard = FS
                .get_or_init(|| std::sync::RwLock::new(cosmic_text::FontSystem::new()))
                .write()
                .unwrap();
            with(&mut guard);
        });
        crate::ui::code_editor::core::with_font_system(f)
    }

    /// PLAN-737 根因回归（真实 merged 窗 T-02 解锁缺陷）：MCP 合成通道
    /// `__mcp_drag_ade` 的完整点击（单点 press+release 直经 core 门）放行
    /// `link_activated` 后，必须经装配期注册的 LinkCallback 构造 VM 消息
    /// ——此前该输出在通道内被丢弃（门 same=true 恒成立、派发恒不发生，
    /// 探针 N 臂 nav_seq 恒 0）。测试镜像生产节拍：首帧 sync 先于编辑壳
    /// 注册（no-op）→ 注册 → 次帧 sync 重建 → render_frame 布局写回 →
    /// 点击门闭合 → 按 **裸键**（MCP 载荷形态）派发。
    #[test]
    fn plan737_mcp_drag_single_point_full_click_dispatches_wiki_activation() {
        // 测试字体系统引导（镜像 core.rs tests::run_fs：callback 先于首个
        // sync_external 安装——生产由 DocEditor::new 装源，测试无 widget 期）。
        run_fs(|_fs| {});
        let key = "wiki/plan737-dispatch.ad";
        let sk = ade::storage_key(key);
        // 生产节拍第一腿：lowering 期 sync 先于 DocEditor::new 注册——首帧
        // no-op（UNREGISTERED，返回 false）。
        assert!(!ade::autodown_editor_sync(
            &sk,
            "见 [[目标页#锚点甲]] 文。",
            true
        ));
        // 编辑壳注册（第二腿此前）→ 次帧 sync 真重建。
        let core = ade::autodown_editor(key);
        assert!(ade::autodown_editor_sync(
            &sk,
            "见 [[目标页#锚点甲]] 文。",
            true
        ));
        run_fs(|fs| {
            let _ = core.render_frame(fs, 600.0, WHITE, None);
        });
        let region = core
            .link_regions()
            .first()
            .cloned()
            .expect("布局写回含链接命中区");
        let (cx, cy) = (
            region.rect.x + region.rect.w / 2.0,
            region.rect.y + region.rect.h / 2.0,
        );

        // 装配期注册（convert_view_messages AutodownEditor 臂同款闭包——
        // 消息构造单源：Typed 双 Str → from_dynamic/encode_payload）。
        let widget = key.to_string();
        let cb = crate::ui::view::LinkCallback::new(move |m: crate::ui::view::WikiLinkMetrics| {
            IcedMessage::from_dynamic(&DynamicMessage::Typed {
                widget_name: widget.clone(),
                event_name: "OpenWikiLink".into(),
                args: vec![
                    auto_val::Value::Str(m.target.into()),
                    auto_val::Value::Str(m.anchor.into()),
                ],
            })
        });
        ade_link_dispatch_register(&sk, cb);

        // 单点完整点击（__mcp_drag_ade 同构：press→release 逐段直调 core）。
        let click = |x: f32, y: f32| -> crate::ui::autodown_editor::DocOutput {
            run_fs(|fs| {
                core.handle_input(
                    fs,
                    DocInput::MousePressed {
                        button: EditorButton::Left,
                        x,
                        y,
                    },
                    &mut NullClipboard,
                )
            });
            run_fs(|fs| {
                core.handle_input(
                    fs,
                    DocInput::MouseReleased {
                        button: EditorButton::Left,
                        x,
                        y,
                    },
                    &mut NullClipboard,
                )
            })
        };
        let out = click(cx, cy);
        let act = out.link_activated.expect("完整点击门放行激活");
        assert_eq!(act.target, "目标页");
        assert_eq!(act.anchor, "锚点甲");

        // 派发腿：**裸键**（MCP 载荷 sk 不带前缀）查表——键幂等归一后同域。
        let msg = ade_link_dispatch_message(key, &act).expect("裸键派发命中");
        assert_eq!(msg.widget, key);
        let (clean, args) = crate::ui::dynamic::decode_payload(&msg.event);
        assert_eq!(clean, "OpenWikiLink");
        assert!(
            matches!(&args[0], auto_val::Value::Str(s) if s.as_str() == "目标页"),
            "{args:?}"
        );
        assert!(
            matches!(&args[1], auto_val::Value::Str(s) if s.as_str() == "锚点甲"),
            "{args:?}"
        );

        // 归一形键同样命中（装配键域直查）。
        assert!(ade_link_dispatch_message(&sk, &act).is_some());
        // 未知键零派发（未挂载/泛型轨语义）。
        assert!(ade_link_dispatch_message("wiki/absent.ad", &act).is_none());
    }

    /// PLAN-737 通道阴性锚：press-only（`__mcp_click` ghost 语义，无抬起
    /// 腿）门不闭合——零激活零派发（K-04 冻结负例不因派发表而弱化）。
    #[test]
    fn plan737_press_only_still_never_dispatches() {
        run_fs(|_fs| {});
        let key = "wiki/plan737-press-only.ad";
        let sk = ade::storage_key(key);
        let core = ade::autodown_editor(key);
        assert!(ade::autodown_editor_sync(&sk, "[[Goals]] 页。", true));
        run_fs(|fs| {
            let _ = core.render_frame(fs, 600.0, WHITE, None);
        });
        let region = core
            .link_regions()
            .first()
            .cloned()
            .expect("链接命中区在案");
        let (cx, cy) = (
            region.rect.x + region.rect.w / 2.0,
            region.rect.y + region.rect.h / 2.0,
        );
        let out = run_fs(|fs| {
            core.handle_input(
                fs,
                DocInput::MousePressed {
                    button: EditorButton::Left,
                    x: cx,
                    y: cy,
                },
                &mut NullClipboard,
            )
        });
        assert!(out.link_activated.is_none(), "press-only 门不闭合零激活");
    }
}

#[cfg(test)]
mod plan095_focus_tests {
    use super::*;

    fn input(widget: &str, event: &str) -> FocusableInput {
        FocusableInput {
            id: iced::widget::Id::from(format!("textarea_{widget}_{event}")),
            widget: widget.to_string(),
            event: event.to_string(),
        }
    }

    #[test]
    fn focus_key_resolution_normalizes_at_event_keys() {
        let inputs = vec![input("FocusProbe", "Input"), input("FocusProbe", "TiInput")];
        // .at 事件键全形（带点前缀与 $event 实参尾）。
        assert!(resolve_focus_target(&inputs, ".Input($event)").is_some());
        assert!(resolve_focus_target(&inputs, ".TiInput").is_some());
        // 裸事件名。
        assert!(resolve_focus_target(&inputs, "TiInput").is_some());
        // widget.event 限定形。
        assert!(resolve_focus_target(&inputs, "FocusProbe.TiInput").is_some());
        // 缺失目标 → None（可定位 miss，不猜第一个控件）。
        assert!(resolve_focus_target(&inputs, ".NoSuch").is_none());
        assert!(resolve_focus_target(&inputs, "").is_none());
    }

    /// PLAN-747：消费步三臂——命中终结（ok + focus 任务）/稳定 miss 终结
    /// （视图已含输入而目标缺席，或 5 轮重试耗尽）/竞态窗口不终结（重试）。
    #[test]
    fn plan747_focus_pending_step_three_arms() {
        let inputs = vec![input("FocusProbe", "TiInput")];
        let (id, done, result) = focus_pending_step(&inputs, ".TiInput", 0);
        assert!(id.is_some() && done && result == "ok");

        let (id, done, result) = focus_pending_step(&inputs, ".NoSuch", 0);
        assert!(id.is_none() && done && result == "miss:.NoSuch");

        let (id, done, result) = focus_pending_step(&[], ".NoSuch", 5);
        assert!(id.is_none() && done && result == "miss:.NoSuch");

        let (id, done, result) = focus_pending_step(&[], ".TiInput", 0);
        assert!(id.is_none() && !done && result.is_empty());
    }

    /// PLAN-747 借用纪律回归：pending 消费形态（判别式克隆先行 + 块内
    /// borrow_mut 清除）不得 panic——钉住「edition 2021 if-let 判别式
    /// Ref 守卫跨块」类回归（修复前块内 borrow_mut 必 panic 且 pending
    /// 永不清除 = update 每消息重入 panic，2026-10-09 审计 4611 例）。
    #[test]
    fn plan747_focus_pending_borrow_discipline() {
        let focus_pending = std::cell::RefCell::new(Some(("QuickEdit".to_string(), 0u32)));
        // 修复后消费形态：克隆语句先行（守卫随 let 语句结束释放）。
        let pending = focus_pending.borrow().clone();
        if let Some((key, tries)) = pending {
            assert_eq!(key, "QuickEdit");
            assert_eq!(tries, 0);
            *focus_pending.borrow_mut() = None;
        }
        assert!(focus_pending.borrow().is_none(), "pending 应被清除");
    }
}

/// PLAN-095 T-04: 当前视图中可聚焦输入控件（Input/Textarea）的稳定 Id 与
/// handler 寻址键（遍历面与 collect_input_ids/collect_media_surfaces 同族）。
pub(crate) struct FocusableInput {
    pub id: iced::widget::Id,
    pub widget: String,
    pub event: String,
}

pub(super) fn collect_focusable_inputs(view: &AbstractView<IcedMessage>, out: &mut Vec<FocusableInput>) {
    match view {
        AbstractView::Input {
            placeholder,
            on_change,
            on_submit,
            width,
            password,
            ..
        } => {
            let primary = on_change
                .as_ref()
                .map(|m| (m.widget.as_str(), m.event.as_str()))
                .or_else(|| {
                    on_submit
                        .as_ref()
                        .map(|m| (m.widget.as_str(), m.event.as_str()))
                });
            out.push(FocusableInput {
                id: derive_input_id(primary, placeholder, *width, *password),
                widget: primary.map(|(w, _)| w.to_string()).unwrap_or_default(),
                event: primary.map(|(_, e)| e.to_string()).unwrap_or_default(),
            });
        }
        AbstractView::Textarea {
            placeholder,
            on_change,
            ..
        } => {
            // 与 Textarea 构建臂的 Id 派生严格同式（主键优先/legacy 回退）。
            let legacy = format!("__textarea_{}", placeholder.len());
            let id = match on_change.as_ref() {
                Some(m) if !m.widget.is_empty() && !m.event.is_empty() => {
                    iced::widget::Id::from(format!("textarea_{}_{}", m.widget, m.event))
                }
                _ => iced::widget::Id::from(format!("textarea_{}", legacy)),
            };
            let (widget, event) = on_change
                .as_ref()
                .map(|m| (m.widget.clone(), m.event.clone()))
                .unwrap_or_default();
            out.push(FocusableInput { id, widget, event });
        }
        AbstractView::Column { children, .. }
        | AbstractView::Row { children, .. }
        | AbstractView::List {
            items: children, ..
        } => {
            for child in children {
                collect_focusable_inputs(child, out);
            }
        }
        AbstractView::MouseArea { content, .. } => collect_focusable_inputs(content, out),
        AbstractView::Popover {
            anchor, content, ..
        } => {
            if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                collect_focusable_inputs(w, out);
            }
            collect_focusable_inputs(content, out);
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            collect_focusable_inputs(child, out);
        }
        AbstractView::Grid { cells, .. } => {
            for cell in cells {
                collect_focusable_inputs(cell, out);
            }
        }
        AbstractView::Overlay { base, content, .. } => {
            collect_focusable_inputs(base, out);
            collect_focusable_inputs(content, out);
        }
        _ => {}
    }
}

/// 目标键解析：剥 `.` 前缀与 `($event)` 实参尾后与 (event) 或 "widget.event"
/// 限定形比较（大小写敏感；web 形 CSS 选择器在 VM 轨无对应物，不猜测）。
fn resolve_focus_target(inputs: &[FocusableInput], key: &str) -> Option<iced::widget::Id> {
    let normalized = key
        .trim()
        .trim_start_matches('.')
        .split("($")
        .next()
        .unwrap_or("")
        .trim_end()
        .to_string();
    if normalized.is_empty() {
        return None;
    }
    let by_event = inputs.iter().find(|i| i.event == normalized);
    if let Some(hit) = by_event {
        return Some(hit.id.clone());
    }
    if let Some((w, e)) = normalized.split_once('.') {
        if let Some(hit) = inputs.iter().find(|i| i.widget == w && i.event == e) {
            return Some(hit.id.clone());
        }
    }
    None
}

/// PLAN-095 T-04 消费步决策（PLAN-747 抽出独立可测）：
/// 返回 (focus 任务目标, 是否终结 pending, `__focus_result` 值)。
/// 终结=false 表示挂载竞态窗口重试（重试计数 +1 由调用方落）。
pub(super) fn focus_pending_step(
    inputs: &[FocusableInput],
    key: &str,
    tries: u32,
) -> (Option<iced::widget::Id>, bool, String) {
    if let Some(id) = resolve_focus_target(inputs, key) {
        (Some(id), true, "ok".to_string())
    } else if !inputs.is_empty() || tries >= 5 {
        (None, true, format!("miss:{key}"))
    } else {
        (None, false, String::new())
    }
}

/// PLAN-095 T-03: 浮层根声明 pointer-events-none → 被动框（不截获下方点选）。
pub(crate) fn abs_layer_passthrough<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> bool {
    extract_view_style(view)
        .map(|s| {
            s.classes
                .iter()
                .any(|c| matches!(c, StyleClass::PointerEventsNone))
        })
        .unwrap_or(false)
}

/// 内容根的百分比宽/高 → FP 单位（三段装配 filler 平衡输入）。
pub(super) fn overlay_percent_units<M: Clone + std::fmt::Debug>(
    view: &AbstractView<M>,
) -> (Option<u16>, Option<u16>) {
    let Some(s) = extract_view_style(view) else {
        return (None, None);
    };
    let is = crate::ui::style_iced_adapter::IcedStyle::from_style(&s);
    let units = |sz: Option<IcedSize>| match sz {
        Some(IcedSize::Percent(v)) => Some(percent_fill_units(v)),
        _ => None,
    };
    (units(is.width), units(is.height))
}

/// 带内容百分比单元的浮层装配（百分比几何 + 穿透的完整入参形态）。
pub(super) fn build_floating_layer_full<M: Clone + Debug + 'static>(
    content: iced::Element<'static, M>,
    position: crate::ui::view::OverlayPosition,
    capture: bool,
    content_units: (Option<u16>, Option<u16>),
) -> iced::Element<'static, M> {
    // PLAN-536 重测修正(2026-09-04)：悬浮层几何改**显式 spacer**——原
    // container(Fill)+align_x 在 Stack 子路径不生效(× 落左上,musk 会话卡
    // 实测),spacer 是不会说谎的几何：top spacer 垂直定位,水平方向
    // Fill-spacer + 定宽 right/left spacer 夹出水平位置。
    // PLAN-022 T-05(2026-09-19)：悬浮层根改**贴内容紧致(Shrink)**——
    // 根若 Fill,Stack 内 Opaque 包装(helpers.rs opaque:update 尾对
    // ButtonPressed `cursor.is_over(layout.bounds())` 即 capture_event)
    // 的捕获边界=整个 Stack 矩形 → 顶层浮层吞掉全窗口按压,下层
    // (第二槽/基础层)交互全灭(auto-term 分屏实测:仅 slot1 可交互,
    // 右面板无法聚焦输入/滚动条死)。top/left 锚 → Shrink 根贴内容,
    // Opaque 捕获=内容矩形,与 CSS absolute 命中语义一致;right 锚需
    // Fill 行做右夹持,保留 Fill 根退路(捕获过宽为已知代价,当前无
    // 消费方)。
    // PLAN-023 T-02(2026-09-19)根修:Opaque **下沉到 content 级**——
    // T-01 判决(docs/plans/evidence/023/t01-verdict.md):opaque 边界
    // =整条 spacer 行矩形(0..left+w),Stack 子层共原点 → 带偏移浮层
    // 吞掉从原点起全部按压(下层槽位死,探针 R1/R3 确证)。本函数
    // 改为对 content 包 opaque、spacer 留在捕获边界外(命中语义:
    // spacer 空白区穿透到下层);调用点(四 Stack 臂/Overlay 臂)对
    // 本函数返回值不再包外层 opaque,零偏移分支维持 opaque(层)。
    // PLAN-023 T-02 续:top 偏移改 **Column padding**——零宽 Space(w=0,
    // h=top)在 Shrink 列内主轴高度被 flex 吞(探针 R8 实测:w=0 → 子件
    // y 不动;w=1 → 正确 +100;iced_core-0.14.0 flex.rs 源码推演与二进
    // 制矛盾,零宽空间主轴尺寸失效,归因记判决工件)。padding 是 flex
    // 一等语义(pad.1 起始偏移),实车槽位 top-[y] 全依赖此路径。
    // PLAN-095 T-03: 百分比几何（musk canvas 框 left/top/width/height 全 %）
    // 走三段 FillPortion 装配——iced 0.14 无 Relative 长度，portion 三段在
    // Fill 容器内=精确份额；缺 width%/height% 配对的轴退化为 px/0（记录
    // 在案，不半猜）。
    let percent_mode = matches!(
        position.top,
        Some(crate::ui::view::OverlayLength::Percent(_))
    ) || matches!(
        position.left,
        Some(crate::ui::view::OverlayLength::Percent(_))
    );
    if percent_mode {
        return percent_floating_layer(content, position, capture, content_units);
    }
    let fl_len = |l: Option<crate::ui::view::OverlayLength>| -> Option<iced::Length> {
        l.map(|v| match v {
            crate::ui::view::OverlayLength::Px(px) => iced::Length::Fixed(px),
            crate::ui::view::OverlayLength::Percent(p) => {
                iced::Length::FillPortion(percent_fill_units(p))
            }
        })
    };
    let top = position
        .top
        .unwrap_or(crate::ui::view::OverlayLength::Px(0.0));
    let top = match top {
        crate::ui::view::OverlayLength::Px(v) => v,
        crate::ui::view::OverlayLength::Percent(_) => 0.0, // % top 交由 padding 形态见下
    };
    let right_anchored = position.left.is_none() && position.right.is_some();
    // PLAN-095 T-03: 浮层穿透——content 根带 PointerEventsNone（musk canvas
    // 框 pointer-events:none 族）时不包 opaque：opaque 的捕获边界=内容矩形，
    // 被动选择框/Agent 框会截获下方 canvas 点选（AC-04）。空白 spacer 仍在
    // 捕获边界外（PLAN-023 语义不变）。
    let opaque_content = |content: iced::Element<'static, M>| {
        if capture {
            iced::widget::opaque(content)
        } else {
            content
        }
    };
    let mut row = iced::widget::Row::<M>::new();
    match position.left {
        Some(left) => {
            if let Some(len) = fl_len(Some(left)) {
                if left.is_positive() {
                    row = row.push(iced::widget::Space::new().width(len));
                }
            }
            row = row.push(opaque_content(content));
        }
        None => {
            if right_anchored {
                row = iced::widget::Row::<M>::new().width(iced::Length::Fill);
                row = row.push(iced::widget::Space::new().width(iced::Length::Fill));
                row = row.push(opaque_content(content));
                if let Some(right) = position.right {
                    if right.is_positive() {
                        if let Some(len) = fl_len(Some(right)) {
                            row = row.push(iced::widget::Space::new().width(len));
                        }
                    }
                }
            } else {
                row = row.push(opaque_content(content));
            }
        }
    }
    let column = iced::widget::column![row].padding(iced::Padding {
        top,
        ..iced::Padding::ZERO
    });
    let column = if right_anchored {
        column.width(iced::Length::Fill)
    } else {
        column
            .width(iced::Length::Shrink)
            .height(iced::Length::Shrink)
    };
    column.into()
}

/// PLAN-536 T10(D1 根修): 动态路径 abs 拆分层的偏移消费判定。
/// 渲染器 Column/Row 臂的 absolute 拆分(Plan 057 2.2 / PLAN-530 步骤3)门槛
/// 是纯 `absolute` 类、落原点不消费偏移——与 builder hoist 臂(absolute+z,
/// View::Overlay → build_floating_layer offset 感知)判定/语义分裂,musk
/// 会话卡 ×(V2 改构丢 z-10)落 builder 门外被本路径接盘,"落左上+遮卡片"
/// 实录。本判定:top/right/left 任一非零 → 用 build_floating_layer 同款
/// spacer 几何定位;全零/无偏移(inset-0 ghost 叠加族)→ None=保持落原点
/// 满铺语义不变。bottom 不参与(T6 边界:px 偏移模型不含 bottom,仅 bottom
/// 非零时保持原点,不做 top 化错位)。pub(crate): plan536 探针单测直测。
pub(crate) fn dynamic_abs_layer_position<M: Clone + std::fmt::Debug>(
    view: &AbstractView<M>,
) -> Option<crate::ui::view::OverlayPosition> {
    let is = crate::ui::style_iced_adapter::IcedStyle::from_style(extract_view_style(view)?);
    let px_or_pct = |px: Option<f32>, pct: Option<f32>| match (px, pct) {
        (Some(v), _) => Some(crate::ui::view::OverlayLength::Px(v)),
        (None, Some(p)) => Some(crate::ui::view::OverlayLength::Percent(p)),
        (None, None) => None,
    };
    let pos = crate::ui::view::OverlayPosition {
        top: px_or_pct(is.top_offset, is.top_percent),
        right: px_or_pct(is.right_offset, is.right_percent),
        bottom: px_or_pct(is.bottom_offset, is.bottom_percent),
        left: px_or_pct(is.left_offset, is.left_percent),
    };
    let has_offset = pos.top.as_ref().map_or(false, |v| v.is_positive())
        || pos.right.as_ref().map_or(false, |v| v.is_positive())
        || pos.left.as_ref().map_or(false, |v| v.is_positive());
    has_offset.then_some(pos)
}

/// Plan 412 F2: per-cell metadata the grid builder needs but can't recover
/// from an already-built `iced::Element`. Callers extract these from the cell
/// `View`'s style before converting it: `col-span-N` → span, any `w-*` width
/// class → the cell opts the whole grid into compact (content-width) tracks.
#[derive(Debug, Clone, Copy, Default)]
pub struct GridCellSpec {
    pub span: usize,
    pub explicit_width: bool,
}

/// Extract `(span, explicit_width)` from a grid cell `View`'s style.
pub(super) fn grid_cell_spec<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> GridCellSpec {
    let style = extract_view_style(view);
    let mut spec = GridCellSpec {
        span: 1,
        explicit_width: false,
    };
    if let Some(s) = style {
        for c in &s.classes {
            match c {
                StyleClass::ColSpan(n) => spec.span = (*n as usize).max(1),
                // Only a *fixed* width (w-8/w-[30px]) opts the grid into compact
                // content-width tracks. w-full is the CSS grid default (item
                // stretches to its track) and stays in equal-track mode.
                StyleClass::Width(
                    crate::ui::style::SizeValue::Fixed(_) | crate::ui::style::SizeValue::Pixels(_),
                ) => {
                    spec.explicit_width = true;
                }
                _ => {}
            }
        }
    }
    spec
}

/// Plan 412 F2: CSS grid auto-placement(简化版)。按源码顺序填充,当前列 +
/// span 超出 `cols` 则换行;span > cols 钳制到 cols。返回每行的
/// `(cell_idx, start_col, span)` 列表(纯函数,便于单测分配器边界:尾行填充、
/// span 钳制、整行 span)。row-span 不做(降级为 col-span,Plan 412 §5)。
pub(super) fn grid_row_placements(spans: &[usize], cols: usize) -> Vec<Vec<(usize, usize, usize)>> {
    let cols = cols.max(1);
    let mut placements: Vec<Vec<(usize, usize, usize)>> = Vec::new();
    let mut current: Vec<(usize, usize, usize)> = Vec::new();
    let mut cur_col = 0usize;
    for (i, &raw_span) in spans.iter().enumerate() {
        let span = raw_span.clamp(1, cols);
        if cur_col + span > cols {
            placements.push(std::mem::take(&mut current));
            cur_col = 0;
        }
        current.push((i, cur_col, span));
        cur_col += span;
    }
    if !current.is_empty() {
        placements.push(current);
    }
    placements
}

/// Build a CSS-Grid-like layout from pre-built cell elements. Two track modes
/// (Plan 412 F2):
///
/// **equal tracks** (default — no cell carries an explicit `w-*` class):
/// CSS `grid-cols-N` means N equal tracks filling the grid width. Emulated by
/// giving every row `width(Fill)` + `spacing(0)` and placing each cell in a
/// `FillPortion(span)` container whose horizontal padding is the slot's share
/// of the gap. With all rows summing to exactly `cols` portions every slot is
/// `W/cols` wide, and cell `i` (starting column `c`, span `s`) renders at
/// `c·g/cols` from its slot's left edge with width `s·W/cols − (cols−s)·g/cols`
/// — pixel-exact CSS grid geometry (`span cell = s tracks + (s−1) gaps`),
/// including col-span via the CSS auto-placement simplification (fill
/// sequentially, wrap when `col + span > cols`). row-span degrades to
/// col-span (documented in Plan 412 §5).
///
/// **compact tracks** (any cell has `w-*`, e.g. the colour-picker's `w-8 h-8`
/// swatches): the previous behaviour — cells pushed directly (Shrink), rows
/// Shrink-width so the grid hugs its content; keeps the Plan 402 §13.10
/// sizing semantics and the Plan 411 colour-picker parity.
///
/// This is the SINGLE source of truth for grid decomposition (Plan 319).
pub(super) fn build_grid<M: Clone + Debug + 'static>(
    cols: usize,
    gap: u16,
    cells: Vec<(iced::Element<'static, M>, GridCellSpec)>,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    let cols = cols.max(1);
    if cells.is_empty() {
        return apply_column_style(column([]), 0, style, widget_id, None);
    }

    let equal_tracks = !cells.iter().any(|(_, spec)| spec.explicit_width);

    if !equal_tracks {
        // Compact mode: legacy path — direct cells, final row padded with
        // empty Fill cells, Shrink rows stacked in a gap-spaced column.
        let mut els: Vec<iced::Element<'static, M>> = cells.into_iter().map(|(el, _)| el).collect();
        let pad = cols - (els.len() % cols);
        if pad != cols {
            for _ in 0..pad {
                els.push(text("").width(iced::Length::Fill).into());
            }
        }
        let mut iter = els.into_iter();
        let mut rows: Vec<iced::Element<'static, M>> = Vec::new();
        loop {
            let mut row_b = row([]).spacing(gap as f32);
            let mut count = 0;
            for _ in 0..cols {
                match iter.next() {
                    Some(cell) => {
                        row_b = row_b.push(cell);
                        count += 1;
                    }
                    None => break,
                }
            }
            if count == 0 {
                break;
            }
            rows.push(row_b.into());
        }
        let col_widget = column(rows)
            .spacing(gap as f32)
            .align_x(iced::Alignment::Center);
        return apply_column_style(col_widget, 0, style, widget_id, None);
    }

    // Equal-track mode: CSS auto-placement via the shared pure placer.
    let spans: Vec<usize> = cells.iter().map(|(_, spec)| spec.span).collect();
    let placements = grid_row_placements(&spans, cols);

    let gap_f = gap as f32;
    let cols_f = cols as f32;
    // Placement order is ascending cell index, so a sequential iterator over
    // `cells` lines up with `placements` exactly.
    let mut iter = cells.into_iter();
    let mut rows: Vec<iced::Element<'static, M>> = Vec::with_capacity(placements.len());
    for row_places in placements {
        let mut row_b = row([]).spacing(0.0).width(iced::Length::Fill);
        let mut occupied = 0usize;
        for (_idx, start, span) in row_places {
            let (el, _spec) = iter.next().expect("placement count matches cells");
            // Inner Fill wrapper stretches the cell to its track (CSS grid
            // default: items fill the track), so cell backgrounds/borders and
            // centered content line up with the vue rendering.
            let stretched = container(el).width(iced::Length::Fill);
            let cell = container(stretched)
                .width(iced::Length::FillPortion(span as u16))
                .padding(iced::Padding {
                    top: 0.0,
                    bottom: 0.0,
                    left: start as f32 * gap_f / cols_f,
                    right: (cols - start - span) as f32 * gap_f / cols_f,
                });
            row_b = row_b.push(cell);
            occupied += span;
        }
        // Pad the trailing run so every row totals `cols` portions — without
        // this the final row's cells would stretch into the empty tracks.
        if occupied < cols {
            row_b = row_b.push(
                iced::widget::Space::new()
                    .width(iced::Length::FillPortion((cols - occupied) as u16)),
            );
        }
        rows.push(row_b.into());
    }
    let col_widget = column(rows).spacing(gap_f).align_x(iced::Alignment::Center);
    apply_column_style(col_widget, 0, style, widget_id, None)
}

// NOTE: Textarea shape is NOT extracted into a shared builder. iced's
// `TextEditor` carries a generic `Highlighter` type parameter, so a clean
// shared return type would leak iced internals into the signature. The
// textarea "shape" is also trivial (a height ternary) and its `on_action`
// handler is inherently mode-specific, so each converter keeps its own
// inline construction — see the `Textarea` arms in `into_iced` and
// `render_dynamic_view`.

/// # Unification rule for new widgets (Plan 319 — read before extending)
///
/// VM mode and Rust mode convert the *same* `AbstractView<M>` tree to iced via
/// **two** entry points: this generic `into_iced<M>` (Rust mode) and the
/// `IcedMessage`-specific `render_dynamic_view` (VM mode, adds DevTools
/// instrumentation). To keep them from drifting, follow this rule:
///
/// 1. **A new widget gets exactly ONE arm here, in `into_iced`.** If it has
///    styled/iterated children, factor the shape into a shared generic
///    `build_<widget><M>` helper (see `build_row` / `build_column` /
///    `build_container` / `build_scrollable` / `build_grid` / `build_input_shape`)
///    and call it with `widget_id = None`.
/// 2. **`render_dynamic_view` NEVER reimplements widget shape.** Its same-named
///    arm only: (a) recurse into each child to attach per-node instrumentation,
///    (b) call the *same* `build_*` helper (passing the instrumented children +
///    the `widget_id` from the debug id map), (c) `wrap_debug` the result.
/// 3. **No third copy.** A widget's spacing/padding/width/children-loop lives in
///    exactly one place — its `build_*` helper. Both converters delegate to it.
///
/// The grid "tower" bug (VM rendered fine, Rust collapsed to a vertical stack)
/// was caused by exactly the drift this rule forbids: the layout lived in two
/// `build_*`-less copies that diverged. `View::Grid` + `build_grid` exists so
/// that cannot recur — grid decomposition is now a single source of truth.
/// Plan 490 G4：布局件（row/col/div=Container）点击包装——有 onclick 时
/// 以 mouse_area 包装、on_release 发射声明消息（402 右键 / 484 hover 两
/// mouse_area 先例同 API 形态）；检视捕获模式不包（inspect mouse_area 需
/// 独占命中，同按钮右键守卫）；None = 原样直出零开销。
///
/// PLAN-002 B：同一包装点收拢布局件三臂——onclick（左键，490 G4）+ 右键
/// （`oncontextmenu` → mouse_area on_right_press，402/526 T10 同 API）+
/// hover 态（`hover:` 变体类 → HoverArea + 共享标志，见 hover_area.rs）。
/// 单个 mouse_area 承载左右两键（mouse_area 不捕获未声明事件，内层交互件
/// 不受影响）；hover 只包 HoverArea（纯委托，不发消息不重建）。
pub(crate) fn wrap_layout_events<'a, M: Clone + 'static>(
    el: iced::Element<'a, M>,
    onclick: Option<M>,
    on_right_click: Option<M>,
    hover: Option<crate::ui::iced::hover_area::HoverFlag>,
) -> iced::Element<'a, M> {
    if inspect_capture_active() {
        return el;
    }
    let el: iced::Element<'a, M> = if onclick.is_some() || on_right_click.is_some() {
        let mut ma = mouse_area(el);
        if let Some(msg) = onclick {
            ma = ma.on_release(msg);
        }
        if let Some(msg) = on_right_click {
            ma = ma.on_right_press(msg);
        }
        ma.into()
    } else {
        el
    };
    match hover {
        Some(flag) => crate::ui::iced::hover_area::HoverArea::new(el, flag).into(),
        None => el,
    }
}

/// PLAN-002 B：布局件 hover 标志——仅当样式声明了 `hover:` 变体类时构造
/// （无声明 = None，零开销路径不变）。标志与样式闭包共享（apply_*_style）。
pub(crate) fn layout_hover_flag(
    style: Option<&Style>,
) -> Option<crate::ui::iced::hover_area::HoverFlag> {
    style
        .filter(|s| s.has_variant(crate::ui::style::Variant::Hover))
        .map(|_| crate::ui::iced::hover_area::HoverFlag::default())
}

/// PLAN-002 B：布局件样式闭包——base/hover 两套已构建样式按共享标志二选一
/// （标志翻转只发 request_redraw；`hover:` 类经 merged_with_variant 覆盖
/// base，语义同按钮臂）。无 hover 声明时等价 `move |_| base`。
pub(crate) fn layout_style_fn(
    base: iced::widget::container::Style,
    hover: Option<iced::widget::container::Style>,
    flag: Option<crate::ui::iced::hover_area::HoverFlag>,
) -> impl Fn(&iced::Theme) -> iced::widget::container::Style + 'static {
    move |_| match (&flag, hover) {
        (Some(f), Some(h)) if f.load(std::sync::atomic::Ordering::Relaxed) => h,
        _ => base,
    }
}

