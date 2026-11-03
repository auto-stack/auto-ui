//! PLAN-014 T-06：view 投影责任（into_iced 巨分派、dynamic_view 族、fit_aware_root）（自 iced/renderer.rs 原样收编）。
//! 消息序/active判据/last-owner exit/纹理与external pin寿命零改——纯移动。
use super::*;
use super::devtools::*;
use super::host::*;
use super::media::*;
use super::shell_state::*;
use super::style_widgets::*;
use super::subscriptions::*;
use iced::widget::column;

impl<M: Clone + Debug + 'static> IntoIcedElement<M> for AbstractView<M> {
    fn into_iced(self) -> iced::Element<'static, M> {
        match self {
            // PLAN-066: 原生外部组件（Element 通道）——查 NativeWidgetRegistry
            // 的 iced Element factory 表。本计划未注册任何 factory（首个后端
            // 原生件落地时建表消费，提案 066 §3.2），零高 Space 防御臂与
            // Empty 同款（不占布局位）。
            AbstractView::Custom { .. } => iced::widget::Space::new()
                .width(iced::Length::Shrink)
                .height(iced::Length::Fixed(0.0))
                .into(),
            // PLAN-063 T-04d-2: 锚槽 → 记录布局坐标的委托 wrapper。
            AbstractView::AnchorSlot { index, child } => {
                let el = child.into_iced();
                iced::Element::new(crate::ui::anchor_slot::AnchorSlot {
                    index: index as usize,
                    child: el,
                })
            }
            // PLAN-781 T-03: 锚点标记 → wrapper widget（operate 期以合成 id
            // 上报最终位置 bounds；layout/draw 全委托零布局）。
            AbstractView::ScrollAnchorMark { pane, key, child } => {
                let el = child.into_iced();
                iced::Element::new(crate::ui::iced::scroll_anchor::ScrollAnchor::new(
                    &pane, &key, el,
                ))
            }
            AbstractView::Empty => {
                // Plan 370 (Issue 1): render Empty as a zero-height Space
                // instead of text(""). A text("") still reserves one line of
                // vertical height in iced, so stacking several Empty views
                // (from false `if` branches / non-matching `for` iterations)
                // produced large blank gaps. A zero-height Space collapses.
                iced::widget::Space::new()
                    .width(iced::Length::Shrink)
                    .height(iced::Length::Fixed(0.0))
                    .into()
            }

            // PLAN-080 UAT F-UAT-2: Rich 段落——单段落跨 span 连续折行
            // （markdown 行内序列唯一正确承载；Row 摆多 Text 不回流）。
            AbstractView::Rich { spans, style } => {
                // F-UAT-2 勘误②：span 颜色仅在显式声明时设置（mark 类如
                // text-primary/foreground），否则**不设色走 iced 祖先继承**
                // ——与普通 Text 同链。曾加 OnBackground 强制回退，实测该
                // 语义解析疑落深色 → 深底深字整段不可见（用户"一度解决又
                // 乱回去"的分界=本回退层引入），移除后恢复 b3ab1520 的
                // 用户验证态。
                let mut spans_vec: Vec<iced::widget::text::Span<'static, ()>> = Vec::new();
                for sp in spans {
                    let mut span = iced::widget::text::Span::new(sp.content.clone());
                    if let Some(ref st) = sp.style {
                        let is = IcedStyle::from_style(st);
                        if let Some(color) = is.text_color {
                            span = span.color(color);
                        }
                        if let Some(fs) = effective_font_size(&is) {
                            span = span.size(fs);
                        }
                        if is.underline {
                            span = span.underline(true);
                        }
                        if is.line_through {
                            span = span.strikethrough(true);
                        }
                        if is.font_family.as_deref() == Some("mono") {
                            span = span.font(iced::Font {
                                family: iced::font::Family::Monospace,
                                ..iced::Font::DEFAULT
                            });
                        }
                        if matches!(is.font_weight, Some(IcedFontWeight::Bold)) {
                            span = span.font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..iced::Font::DEFAULT
                            });
                        }
                    }
                    spans_vec.push(span);
                }
                let mut rich = iced::widget::text::Rich::<(), M>::with_spans(spans_vec);
                if let Some(ref s) = style {
                    let is = IcedStyle::from_style(s);
                    if let Some(fs) = effective_font_size(&is) {
                        rich = rich.size(fs);
                    }
                }
                let mut el: iced::Element<'static, M> = rich.into();
                if let Some(ref s) = style {
                    el = wrap_with_margin(el, &IcedStyle::from_style(s));
                }
                el
            }
            AbstractView::Text {
                content,
                style,
                selectable,
                ..
            } => {
                // Plan 409 §10 续 20: font-mono 的 Text 当代码 → Rich 语法高亮。
                let is_code = style
                    .as_ref()
                    .map(|s| IcedStyle::from_style(s).font_family.as_deref() == Some("mono"))
                    .unwrap_or(false);
                if is_code {
                    // Plan 442 A6: codeblock 携带的 lang-<token> class 决定
                    // syntect 语法选择(见 aura_view_builder codeblock 分支)。
                    let lang = style.as_ref().and_then(|s| {
                        s.classes.iter().find_map(|c| match c {
                            StyleClass::CodeLang(l) => Some(l.clone()),
                            _ => None,
                        })
                    });
                    let spans = highlight_code(&content, lang.as_deref());
                    let mut rich =
                        iced::widget::text::Rich::<(), M>::with_spans(spans).font(iced::Font {
                            family: iced::font::Family::Monospace,
                            ..iced::Font::DEFAULT
                        });
                    // L0 fix(diff 视图行高抖动/auto-edit 2026-10-11):此臂此前
                    // 只应用字号——nowrap/宽度/对齐全被忽略,font-mono 长行
                    // 折行使行高翻倍(diff 行 24px→48px,左右栏折行数不同→纵
                    // 向错位;折行高度随视口宽度振荡→滚动条高频抖动)。对齐
                    // 下方 text 构造链:wrap_none→单行+clip,width/align 同参
                    // (行号列 w-12 text-right 依赖)。
                    let iced_style = style.as_ref().map(IcedStyle::from_style);
                    if let Some(ref is) = iced_style {
                        if let Some(fs) = effective_font_size(is) {
                            rich = rich.size(fs);
                        }
                        let wrap_none = is.wrap_none;
                        if wrap_none {
                            rich = rich.wrapping(iced::widget::text::Wrapping::None);
                        }
                        if let Some(ref w) = is.width {
                            rich = rich.width(iced_length(w));
                        }
                        if let Some(ref align) = is.text_align {
                            use crate::ui::style_iced_adapter::IcedTextAlign;
                            if is.width.is_none() {
                                rich = rich.width(iced::Length::Fill);
                            }
                            match align {
                                IcedTextAlign::Center => {
                                    rich = rich.align_x(iced::alignment::Horizontal::Center);
                                }
                                IcedTextAlign::Right => {
                                    rich = rich.align_x(iced::alignment::Horizontal::Right);
                                }
                                IcedTextAlign::Left => {}
                            }
                        }
                        // truncate/fill 意图才套 Fill+clip 容器（独占行宽截
                        // 断语义）；纯 whitespace-nowrap=仅禁折行保自然宽——
                        // 否则并排 diff 行内多段 shrink-0 文本齐抢 Fill 被
                        // 裁成等宽截断（L0 fix 2026-10-11 二阶）。
                        let fill_clip = is.truncate_flag
                            || matches!(
                                is.width,
                                Some(crate::ui::style_iced_adapter::IcedSize::Full)
                            );
                        let el: iced::Element<'static, M> = if fill_clip {
                            iced::widget::container(rich)
                                .width(iced::Length::Fill)
                                .clip(true)
                                .into()
                        } else {
                            rich.into()
                        };
                        wrap_with_margin(el, is)
                    } else {
                        let el: iced::Element<'static, M> = rich.into();
                        el
                    }
                } else if selectable {
                    // Plan 481: SelectableText 分流 —— 与下方 text 构造链同参
                    // 镜像(缺省 false 路径零改动,I3 配置差异形态)。label 无
                    // 独立 View 变体(aura_view_builder 折叠为 Text),此臂即
                    // text/label 共用分流点。
                    use crate::ui::iced::selectable_text::SelectableText;
                    let mut st = SelectableText::new(content.clone());

                    if let Some(ref s) = style {
                        let iced_style = IcedStyle::from_style(s);

                        if let Some(fs) = effective_font_size(&iced_style) {
                            st = st.size(fs);
                        }
                        if let Some(color) = iced_style.text_color {
                            st = st.color(color);
                        } else if let Some((r, g, b)) =
                            crate::ui::style_iced_adapter::resolve_semantic_rgb(
                                &crate::ui::style::Color::OnBackground,
                            )
                        {
                            st = st.color(iced::Color::from_rgb8(r, g, b));
                        }
                        if let Some(ref weight) = iced_style.font_weight {
                            st = st.font(font_weight_to_iced(weight));
                        }
                        if let Some(ref family) = iced_style.font_family {
                            let fam = match family.as_str() {
                                "serif" => iced::font::Family::Serif,
                                "mono" => iced::font::Family::Monospace,
                                _ => iced::font::Family::SansSerif,
                            };
                            let weight = iced_style
                                .font_weight
                                .as_ref()
                                .map(font_weight_to_iced)
                                .unwrap_or(iced::Font::DEFAULT);
                            st = st.font(iced::Font {
                                family: fam,
                                weight: weight.weight,
                                stretch: weight.stretch,
                                style: weight.style,
                            });
                        }
                        if let Some(ref w) = iced_style.width {
                            st = st.width(iced_length(w));
                        }
                        if let Some(ref align) = iced_style.text_align {
                            use crate::ui::style_iced_adapter::IcedTextAlign;
                            if iced_style.width.is_none() {
                                st = st.width(iced::Length::Fill);
                            }
                            match align {
                                IcedTextAlign::Center => {
                                    st = st.align_x(iced::alignment::Horizontal::Center);
                                }
                                IcedTextAlign::Right => {
                                    st = st.align_x(iced::alignment::Horizontal::Right);
                                }
                                IcedTextAlign::Left => {}
                            }
                        }
                    }

                    let el: iced::Element<'static, M> = st.into();
                    if let Some(ref s) = style {
                        wrap_with_margin(el, &IcedStyle::from_style(s))
                    } else {
                        el
                    }
                } else if style.as_ref().map_or(false, |s| {
                    let is = IcedStyle::from_style(s);
                    is.line_through || is.underline
                }) {
                    let iced_style = IcedStyle::from_style(style.as_ref().unwrap());
                    let mut span = iced::widget::text::Span::new(content);
                    if iced_style.line_through {
                        span = span.strikethrough(true);
                    }
                    if iced_style.underline {
                        span = span.underline(true);
                    }
                    if let Some(fs) = effective_font_size(&iced_style) {
                        span = span.size(fs);
                    }
                    if let Some(color) = iced_style.text_color {
                        span = span.color(color);
                    } else if let Some((r, g, b)) =
                        crate::ui::style_iced_adapter::resolve_semantic_rgb(
                            &crate::ui::style::Color::OnBackground,
                        )
                    {
                        span = span.color(iced::Color::from_rgb8(r, g, b));
                    }
                    if let Some(ref weight) = iced_style.font_weight {
                        span = span.font(font_weight_to_iced(weight));
                    }
                    if let Some(ref family) = iced_style.font_family {
                        let fam = match family.as_str() {
                            "serif" => iced::font::Family::Serif,
                            "mono" => iced::font::Family::Monospace,
                            _ => iced::font::Family::SansSerif,
                        };
                        let weight = iced_style
                            .font_weight
                            .as_ref()
                            .map(font_weight_to_iced)
                            .unwrap_or(iced::Font::DEFAULT);
                        span = span.font(iced::Font {
                            family: fam,
                            weight: weight.weight,
                            stretch: weight.stretch,
                            style: weight.style,
                        });
                    }
                    let mut rich = iced::widget::text::Rich::<(), M>::with_spans(vec![span]);
                    if let Some(ref w) = iced_style.width {
                        rich = rich.width(iced_length(w));
                    }
                    if let Some(ref align) = iced_style.text_align {
                        use crate::ui::style_iced_adapter::IcedTextAlign;
                        if iced_style.width.is_none() {
                            rich = rich.width(iced::Length::Fill);
                        }
                        match align {
                            IcedTextAlign::Center => {
                                rich = rich.align_x(iced::alignment::Horizontal::Center);
                            }
                            IcedTextAlign::Right => {
                                rich = rich.align_x(iced::alignment::Horizontal::Right);
                            }
                            IcedTextAlign::Left => {}
                        }
                    }
                    let el: iced::Element<'static, M> = rich.into();
                    wrap_with_margin(el, &iced_style)
                } else {
                    let mut text_widget = text(content);

                    if let Some(ref s) = style {
                        let iced_style = IcedStyle::from_style(s);

                        if let Some(fs) = effective_font_size(&iced_style) {
                            text_widget = text_widget.size(fs);
                        }
                        if let Some(color) = iced_style.text_color {
                            text_widget = text_widget.color(color);
                        } else {
                            // Plan 408: no explicit text color → apply dark-mode-aware
                            // default (equivalent to vue's body { text-foreground }
                            // inheritance). Without this, iced defaults to BLACK which
                            // is invisible on dark backgrounds.
                            if let Some((r, g, b)) =
                                crate::ui::style_iced_adapter::resolve_semantic_rgb(
                                    &crate::ui::style::Color::OnBackground,
                                )
                            {
                                text_widget = text_widget.color(iced::Color::from_rgb8(r, g, b));
                            }
                        }
                        if let Some(ref weight) = iced_style.font_weight {
                            text_widget = text_widget.font(font_weight_to_iced(weight));
                        }
                        // Apply font family (font-serif/sans/mono)
                        if let Some(ref family) = iced_style.font_family {
                            let fam = match family.as_str() {
                                "serif" => iced::font::Family::Serif,
                                "mono" => iced::font::Family::Monospace,
                                _ => iced::font::Family::SansSerif,
                            };
                            let weight = iced_style
                                .font_weight
                                .as_ref()
                                .map(font_weight_to_iced)
                                .unwrap_or(iced::Font::DEFAULT);
                            text_widget = text_widget.font(iced::Font {
                                family: fam,
                                weight: weight.weight,
                                stretch: weight.stretch,
                                style: weight.style,
                            });
                        }
                        // Plan 527 T5: 行高 —— 绝对档(leading-3..10)优先,相对倍率次之
                        if let Some(px) = iced_style.line_height_px {
                            text_widget = text_widget.line_height(
                                iced::widget::text::LineHeight::Absolute(iced::Pixels(px)),
                            );
                        } else if let Some(lh) = iced_style.line_height {
                            text_widget = text_widget
                                .line_height(iced::widget::text::LineHeight::Relative(lh));
                        }
                        // Apply width (e.g., from flex-1)
                        if let Some(ref w) = iced_style.width {
                            text_widget = text_widget.width(iced_length(w));
                        }
                        if let Some(ref align) = iced_style.text_align {
                            use crate::ui::style_iced_adapter::IcedTextAlign;
                            if iced_style.width.is_none() {
                                text_widget = text_widget.width(iced::Length::Fill);
                            }
                            match align {
                                IcedTextAlign::Center => {
                                    text_widget =
                                        text_widget.align_x(iced::alignment::Horizontal::Center);
                                }
                                IcedTextAlign::Right => {
                                    text_widget =
                                        text_widget.align_x(iced::alignment::Horizontal::Right);
                                }
                                IcedTextAlign::Left => {}
                            }
                        }
                    }

                    // 2026-08-22(侧栏描述换行治理):truncate/whitespace-nowrap →
                    // 单行(Wrapping::None)+ clip 容器裁横向溢出,对齐 CSS truncate。
                    let wrap_none = style
                        .as_ref()
                        .map(|s| IcedStyle::from_style(s).wrap_none)
                        .unwrap_or(false);
                    if wrap_none {
                        text_widget = text_widget
                            .wrapping(iced::widget::text::Wrapping::None)
                            .width(iced::Length::Fill);
                    }
                    let el: iced::Element<'static, M> = if wrap_none {
                        iced::widget::container(text_widget)
                            .width(iced::Length::Fill)
                            .clip(true)
                            .into()
                    } else if let Some(ref s) = style {
                        let iced_style = IcedStyle::from_style(s);
                        // PLAN-619 R4: `height`（如 sidebar_group_label 契约的
                        // `h-8`）此前在这一臂被整条忽略——Text 直接进 iced，盒子
                        // 高度只剩行盒，分组标签比 Vue 矮 16px 并连带整个列表上移。
                        // `items-center` 同步折成盒内纵向居中（CSS `display:flex;
                        // align-items:center` 的行内等价）。
                        let box_height =
                            iced_style.height.as_ref().map(iced_length).or_else(|| {
                                iced_style
                                    .min_height
                                    .filter(|mh| *mh < 9999.0)
                                    .map(iced::Length::Fixed)
                            });
                        let has_box_style = iced_style.background_color.is_some()
                        || iced_style.border
                        || iced_style.has_border_radius()
                        || iced_style.border_width.map_or(false, |w| w > 0.0)
                        || iced_style.padding.is_some()
                        || iced_style.padding_x.is_some()
                        || iced_style.padding_y.is_some()
                        || iced_style.padding_top.is_some()
                        || iced_style.padding_bottom.is_some()
                        || iced_style.padding_left.is_some()
                        || iced_style.padding_right.is_some()
                        || iced_style.max_width.is_some()
                        || iced_style.max_height.is_some()
                        // PLAN-077: max-w-[N%] 同样要求盒承载（百分比收窄由
                        // 下方 MaxWidthPct 委托 widget 完成）。
                        || iced_style.max_width_pct.is_some()
                        || box_height.is_some()
                        || iced_style.shadow;

                        if has_box_style {
                            let pad = iced_padding_from_is(&iced_style, 0);
                            let cs = build_container_style(&iced_style);
                            let mut cont = iced::widget::container(text_widget)
                                .padding(pad)
                                .style(move |_| cs);
                            if let Some(mw) = iced_style.max_width {
                                cont = cont.max_width(mw);
                            }
                            if let Some(mh) = iced_style.max_height {
                                cont = cont.max_height(mh);
                            }
                            if let Some(h) = box_height {
                                cont = cont.height(h);
                                if matches!(
                                    iced_style.align_items,
                                    Some(crate::ui::style_iced_adapter::IcedAlign::Center)
                                ) {
                                    cont = cont.align_y(iced::alignment::Vertical::Center);
                                }
                            }
                            let el: iced::Element<'static, M> = cont.into();
                            match iced_style.max_width_pct {
                                Some(pct) => {
                                    crate::ui::iced::max_width::MaxWidthPct::new(el, pct / 100.0)
                                        .into()
                                }
                                None => el,
                            }
                        } else {
                            text_widget.into()
                        }
                    } else {
                        text_widget.into()
                    };
                    let el = if let Some(ref s) = style {
                        let iced_style = IcedStyle::from_style(s);
                        wrap_with_margin(el, &iced_style)
                    } else {
                        el
                    };
                    match style
                        .as_ref()
                        .and_then(|s| IcedStyle::from_style(s).max_width_pct)
                    {
                        Some(pct) => {
                            crate::ui::iced::max_width::MaxWidthPct::new(el, pct / 100.0).into()
                        }
                        None => el,
                    }
                }
            }

            AbstractView::Button {
                label,
                content,
                onclick,
                style,
                on_right_click,
                disabled,
            } => {
                // Plan 418 P2-3 续: EE03 PUA marker carries an icon-button
                // tooltip (synthesized toolbar buttons embed the action title
                // there — the label itself stays icon-only so the resting
                // render is a bare svg). Stripped here; wrapped around the
                // final element at the arm's end.
                let (label, pua_tooltip): (String, Option<String>) = match label.find('\u{EE03}') {
                    Some(i) => (
                        label[..i].to_string(),
                        Some(label[i + '\u{EE03}'.len_utf8()..].to_string()),
                    ),
                    None => (label, None),
                };
                let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

                // Plan 409 §6: if the button carries a content subtree (a `link`
                // with children — `link (to:) { text/row/icon ... }`), render it
                // directly as the button content. The label is kept for the
                // snapshot builder / accessibility.
                // Plan 409 §8: inherit the button's own text color into the
                // content subtree — Text children without an explicit color
                // (e.g. `link (to: "/") { text "Docs" }`) become theme-colored
                // instead of falling back to the body default (vue parity).
                let inherit_color = style.as_ref().and_then(|s| {
                    // PLAN-055: 取最后一个 TextColor（与 IcedStyle::from_style
                    // "后类胜"语义一致）——此前 find_map 取首个，preset 的
                    // `text-primary-foreground` 恒压过用户后置的
                    // `text-muted-foreground`：图标继承 OnPrimary（深藏青）
                    // 落深底不可见，按钮文字却是浅色——图文两色。
                    s.classes.iter().rev().find_map(|c| match c {
                        StyleClass::TextColor(color) => Some(*color),
                        _ => None,
                    })
                });
                // Plan 057 (ash-gui VM 修复): icon-button 形态 —— style 同时带固定
                // 宽和高(如 block 头部 `h-6 w-6` 的 chevron/copy 按钮)。这类按钮的
                // variant/size preset 会前置 `px-4`(16px),用户 class 只有 w/h 时
                // 16px×2 的水平内边距超过按钮宽度,iced 的内容区收敛到 ≤0,icon/文字
                // 一律不绘制(块头操作按钮全空白的根因)。CSS 侧这些按钮靠
                // `flex items-center justify-center` 居中,iced 无 flex——此处对
                // 双固定尺寸按钮:内边距清零 + 内容在盒内双向居中,复现 CSS 语义。
                let fixed_both = matches!(
                    (
                        iced_style.as_ref().and_then(|is| is.width),
                        iced_style.as_ref().and_then(|is| is.height)
                    ),
                    (
                        Some(crate::ui::style_iced_adapter::IcedSize::Fixed(_)),
                        Some(crate::ui::style_iced_adapter::IcedSize::Fixed(_))
                    )
                );
                let button_content: iced::Element<'static, M> = if let Some(mut content_view) =
                    content
                {
                    if let Some(color) = inherit_color {
                        inherit_text_color(&mut content_view, color);
                    }
                    (*content_view).into_iced()
                } else if label.starts_with('\u{EE01}') {
                    let end = label.find('\u{EE02}').unwrap_or(label.len());
                    let icon_name = &label[3..end.min(label.len())];
                    let text_label = &label[end.saturating_add(3).min(label.len())..];
                    // PLAN-018：iconfile:<stem> = 双主题位图文件（回退链首位
                    // iconfile → hicon → lucide）。Plan 515 D1：hicon:<slot> =
                    // native 真图标（raster——486 占位清偿；14px 与邻位 lucide
                    // 图标同档）。两 raster 源合流同一元素臂。
                    // PLAN-526 T9：图标盒跟随按钮字号（text-lg → 18px），
                    // 回退 14px（462 档）——大框小图实测反馈闭环。
                    // PLAN-018-FU7：iconfile 按显示档装载（512 直挂经无
                    // mipmap 的 Linear min_filter 缩 ~28× 硬边——见
                    // icon_file::load_sized 注）。
                    let icon_px = iced_style
                        .as_ref()
                        .and_then(|is| is.font_size.as_ref())
                        .map(font_size_to_f32)
                        .unwrap_or(14.0);
                    let raster_icon: Option<iced::widget::image::Handle> = {
                        if let Some(handle) = crate::ui::iced::icon_file::load_sized(
                            icon_name,
                            crate::ui::style::theme::dark_mode(),
                            icon_px,
                        ) {
                            Some(handle)
                        } else {
                            crate::ui::iced::native_icon::parse_field(icon_name).map(|icon| {
                                iced::widget::image::Handle::from_rgba(icon.w, icon.h, icon.rgba)
                            })
                        }
                    };
                    if let Some(handle) = raster_icon {
                        let icon_el = iced::widget::image(handle)
                            .width(iced::Length::Fixed(icon_px))
                            .height(iced::Length::Fixed(icon_px));
                        if text_label.is_empty() {
                            // PLAN-012 O1：icon-only 内容盒居中——iced 0.14 button
                            // 布局 = layout::padded 后 content **左上放置**（无
                            // center；iced_widget-0.14.2 button.rs layout/draw
                            // 实读），固定尺寸按钮（dock h-10 w-10 等）的图标
                            // 随主题 padding 偏左上（实机像素实测 dx=-2.7/
                            // dy=-3.0 逻辑 px，与 padding=8 → 内容盒 24×24 推
                            // 算精确吻合；headless 探针主题 padding 异值故未
                            // 复现）。Fill×Fill + 双向 center 在固定盒内居中，
                            // auto-size 盒退化为原尺寸（Shrink 语境 Fill=子尺
                            // 寸）no-op。hicon 臂与 lucide 臂同修。
                            iced::widget::container(icon_el)
                                .width(iced::Length::Fill)
                                .height(iced::Length::Fill)
                                .center_x(iced::Length::Fill)
                                .center_y(iced::Length::Fill)
                                .into()
                        } else {
                            let mut tw = text(text_label.to_string());
                            if let Some(ref is) = iced_style {
                                if let Some(ref fs) = is.font_size {
                                    tw = tw.size(font_size_to_f32(fs));
                                }
                                if let Some(c) = is.text_color {
                                    tw = tw.color(c);
                                }
                            }
                            // PLAN-615 T-03 (W1): 同按钮标签行高钳（图标与文字盒错位同源）。
                            let has_lh = iced_style.as_ref().is_some_and(|s| {
                                s.line_height.is_some() || s.line_height_px.is_some()
                            });
                            if !has_lh {
                                tw = tw.line_height(iced::widget::text::LineHeight::Relative(1.0));
                            }
                            iced::widget::row!(icon_el, tw)
                                .spacing(6)
                                .align_y(iced::alignment::Vertical::Center)
                                .into()
                        }
                    } else if let Some(svg_str) = lucide_svg_doc(icon_name) {
                        // Plan 409 §10 续: PUA icon(button label 内嵌的 nav-link
                        // 图标)与文字同色 —— button 的 text_color,无则 OnBackground
                        // (renderer §3.4,避免 resvg 把 currentColor 画成黑色)。
                        // 2026-08-21 方案 A:改画时着色(svg::Style.color);悬停色取
                        // 按钮自身 hover:text-*(浏览器里 button:hover 的 color 经
                        // 继承级联到内联 SVG,此处以 svg 自身 Hovered 状态近似 ——
                        // 命中区为图标盒而非整个按钮,图标贴文本时差异可忽略)。
                        let text_color =
                            iced_style
                                .as_ref()
                                .and_then(|is| is.text_color)
                                .or_else(|| {
                                    crate::ui::style_iced_adapter::resolve_semantic_rgb(
                                        &crate::ui::style::Color::OnBackground,
                                    )
                                    .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                                });
                        let hover_color = style.as_ref().and_then(|s| {
                            s.hover_classes.iter().find_map(|c| match c {
                                StyleClass::TextColor(color) => {
                                    Some(crate::ui::style_iced_adapter::convert_color(color))
                                }
                                _ => None,
                            })
                        });
                        let base = text_color.unwrap_or(iced::Color::BLACK);
                        let handle = get_or_create_svg_handle(
                            &format!("lucide:{}", icon_name),
                            svg_str.as_bytes().to_vec(),
                        );
                        // PLAN-526 T9：图标盒跟随按钮字号（text-lg → 18px），
                        // 回退 14px——与 hicon 臂同口径。
                        let icon_px = iced_style
                            .as_ref()
                            .and_then(|is| is.font_size.as_ref())
                            .map(font_size_to_f32)
                            .unwrap_or(14.0);
                        let icon_el = iced::widget::svg(handle)
                            .style(move |_, status| {
                                let color = match status {
                                    iced::widget::svg::Status::Hovered => {
                                        hover_color.unwrap_or(base)
                                    }
                                    _ => base,
                                };
                                iced::widget::svg::Style { color: Some(color) }
                            })
                            .width(iced::Length::Fixed(icon_px))
                            .height(iced::Length::Fixed(icon_px));
                        let mut tw = text(text_label.to_string());
                        if let Some(ref is) = iced_style {
                            if let Some(ref fs) = is.font_size {
                                tw = tw.size(font_size_to_f32(fs));
                            }
                            if let Some(c) = is.text_color {
                                tw = tw.color(c);
                            }
                        }
                        // PLAN-615 T-03 (W1): 同按钮标签行高钳（图标与文字盒错位同源）。
                        let has_lh = iced_style
                            .as_ref()
                            .is_some_and(|s| s.line_height.is_some() || s.line_height_px.is_some());
                        if !has_lh {
                            tw = tw.line_height(iced::widget::text::LineHeight::Relative(1.0));
                        }
                        // Icon-only (no text part): center the svg in the
                        // button's content box — iced 0.14 button places content
                        // top-left (no centering); fixed-size buttons showed the
                        // icon offset up-left by the theme padding (PLAN-012 O1
                        // 实机像素实测与机制推断互证，见 hicon 臂同款注释)。
                        // Fill container centers in fixed boxes; no-op in
                        // auto-size (Shrink) buttons.
                        if text_label.is_empty() {
                            iced::widget::container(icon_el)
                                .width(iced::Length::Fill)
                                .height(iced::Length::Fill)
                                .center_x(iced::Length::Fill)
                                .center_y(iced::Length::Fill)
                                .into()
                        } else {
                            iced::widget::row!(icon_el, tw)
                                .spacing(6)
                                .align_y(iced::alignment::Vertical::Center)
                                .into()
                        }
                    } else {
                        // Unknown icon: fall back to plain text label
                        text(text_label.to_string()).into()
                    }
                } else if label.contains('\n') {
                    let lines: Vec<&str> = label.split('\n').collect();
                    let mut col = iced::widget::Column::<M>::with_capacity(lines.len());
                    let default_size = font_size_to_f32(&IcedFontSize::Sm);
                    let default_weight = font_weight_to_iced(&IcedFontWeight::Medium);
                    for (i, line) in lines.iter().enumerate() {
                        let mut tw = text(line.to_string());
                        // Apply the button's own text styles to the first line (title).
                        if i == 0 {
                            if let Some(ref is) = iced_style {
                                tw = tw.size(
                                    is.font_size.as_ref().map_or(default_size, font_size_to_f32),
                                );
                                if let Some(color) = is.text_color {
                                    tw = tw.color(color);
                                }
                                tw = tw.font(
                                    is.font_weight
                                        .as_ref()
                                        .map_or(default_weight, font_weight_to_iced),
                                );
                            } else {
                                tw = tw.size(default_size).font(default_weight);
                            }
                        } else {
                            // Metadata lines: smaller (12px) + muted gray.
                            tw = tw.size(12.0).color([0.5, 0.5, 0.5, 1.0]);
                        }
                        col = col.push(tw);
                    }
                    col.into()
                } else if let Some(ref is) = iced_style {
                    let default_size = font_size_to_f32(&IcedFontSize::Sm);
                    let default_weight = font_weight_to_iced(&IcedFontWeight::Medium);
                    if is.line_through || is.underline {
                        let mut span = iced::widget::text::Span::new(label.clone());
                        if is.line_through {
                            span = span.strikethrough(true);
                        }
                        if is.underline {
                            span = span.underline(true);
                        }
                        span =
                            span.size(is.font_size.as_ref().map_or(default_size, font_size_to_f32));
                        if let Some(color) = is.text_color {
                            span = span.color(color);
                        }
                        span = span.font(
                            is.font_weight
                                .as_ref()
                                .map_or(default_weight, font_weight_to_iced),
                        );
                        let mut rich = iced::widget::text::Rich::<(), M>::with_spans(vec![span]);
                        if let Some(ref align) = is.text_align {
                            let _ = align;
                            rich = rich.width(iced::Length::Fill);
                            match align {
                                crate::ui::style_iced_adapter::IcedTextAlign::Center => {
                                    rich = rich.align_x(iced::alignment::Horizontal::Center);
                                }
                                crate::ui::style_iced_adapter::IcedTextAlign::Right => {
                                    rich = rich.align_x(iced::alignment::Horizontal::Right);
                                }
                                crate::ui::style_iced_adapter::IcedTextAlign::Left => {}
                            }
                        }
                        rich.into()
                    } else {
                        let mut text_widget = text(label.clone());
                        text_widget = text_widget
                            .size(is.font_size.as_ref().map_or(default_size, font_size_to_f32));
                        if let Some(color) = is.text_color {
                            text_widget = text_widget.color(color);
                        }
                        text_widget = text_widget.font(
                            is.font_weight
                                .as_ref()
                                .map_or(default_weight, font_weight_to_iced),
                        );
                        // PLAN-615 T-03 (W1): 按钮标签行盒钳到 1.0 行高——iced 0.14 文本
                        // 默认 Relative(1.3)，额外 leading 全部落在字形上方，无高度类
                        // 按钮（shrink 高度 = 行盒高）的字形在按钮内系统性偏下（calc 数字
                        // 键盘实测 ~0.15em）。行高 1.0 让行盒贴合字形，光学居中恢复；
                        // 高度类按钮本就经 Plan 414 容器居中，此钳同样消除行盒漂移。
                        // 用户显式 leading-* 类仍优先（iced_adapter 双轨字段）。
                        // web 侧 Tailwind text-lg 行高 1.75rem 与 iced 的盒高差是既有
                        // 双端差异，不在本钳范围（双端按钮高度 parity 另行台账）。
                        if is.line_height.is_none() && is.line_height_px.is_none() {
                            text_widget = text_widget
                                .line_height(iced::widget::text::LineHeight::Relative(1.0));
                        }
                        // Plan 411: text-center/left/right on button labels — wide
                        // buttons (e.g. preview-card tabs) need horizontal alignment.
                        // Unlike the Text arm, ALWAYS Fill the label: is.width is the
                        // BUTTON's width class (w-full here), not the text's, so the
                        // is_none() guard would skip Fill and align_x would no-op.
                        if let Some(ref align) = is.text_align {
                            let _ = align;
                            text_widget = text_widget.width(iced::Length::Fill);
                            match align {
                                crate::ui::style_iced_adapter::IcedTextAlign::Center => {
                                    text_widget =
                                        text_widget.align_x(iced::alignment::Horizontal::Center);
                                }
                                crate::ui::style_iced_adapter::IcedTextAlign::Right => {
                                    text_widget =
                                        text_widget.align_x(iced::alignment::Horizontal::Right);
                                }
                                crate::ui::style_iced_adapter::IcedTextAlign::Left => {}
                            }
                        }
                        text_widget.into()
                    }
                } else {
                    // PLAN-615 T-03 (W1): 无样式类按钮同钳行高（默认样式路径的偏下同源）。
                    text(label.clone())
                        .size(font_size_to_f32(&IcedFontSize::Sm))
                        .font(font_weight_to_iced(&IcedFontWeight::Medium))
                        .line_height(iced::widget::text::LineHeight::Relative(1.0))
                        .into()
                };

                // Plan 309 续篇 II: in inspect-capture mode, render the button
                // WITHOUT on_press so it doesn't capture the press — the
                // `wrap_debug` mouse_area then captures inspect hover/click over
                // it. The button keeps its custom `move |_, _| bs` style below,
                // so it still renders normally (status is ignored). Alt (capture
                // off) restores the native onclick.
                // Plan 409 §10 续 21: Fixed-height button 的 content 垂直居中
                // (iced button padded 默认 content 顶部对齐,h-9/h-10/h-11 会偏上)。
                // 包一层 Fill+center_y 容器让文字纵向居中。
                // Plan 057 (ash-gui VM 修复): 双固定尺寸 icon-button 走双向居中
                // (见上方 fixed_both 注释),复现 `flex items-center justify-center`。
                // Plan 414 R12: fixed-size buttons center their content HORIZONTALLY
                // too - iced button content hugs the left edge, so a fixed-width
                // button holding a 16px icon (toolbar w-8) sat left-of-center and
                // the gaps next to a sep read uneven (user-verified via debug
                // borders). CSS buttons center by default; shrink-width buttons
                // are unaffected (content width = button width).
                // Fill-width ONLY for buttons with an explicit width class: a Fill
                // content wrapper also makes height-only buttons (menus, tabs)
                // claim the whole row and stretch equally - regression fixed by
                // scoping center_x to width-classed buttons.
                // Plan-050 C1: content-subtree 按钮消费 .at 的宽度/对齐类——
                // w-full 已由 is.width→btn.width(Fill) 承接;此处补内容对齐
                // (justify-start/text-left→水平,items-start→垂直),Fill 容器
                // 承载对齐;未命中维持 iced 默认居中。
                // 先于 409/414 分支计算:高度臂的默认居中必须给显式对齐类
                // 让位(iced 容器默认居中,外层 050 包装包不住内层 center_x
                // ——nav-item(h-9 w-full text-left)内容恒居中的根因)。
                let (p050_ax, p050_ay) = plan050_content_align(iced_style.as_ref());
                let has_width = iced_style.as_ref().map_or(false, |is| is.width.is_some());
                let has_height = iced_style.as_ref().map_or(false, |is| is.height.is_some());
                let button_content: iced::Element<'static, M> = if fixed_both {
                    iced::widget::container(button_content)
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill)
                        .center_x(iced::Length::Fill)
                        .center_y(iced::Length::Fill)
                        .into()
                } else if has_height {
                    let (ax, ay) = plan414_content_alignment(p050_ax, p050_ay);
                    let mut cont = iced::widget::container(button_content)
                        .height(iced::Length::Fill)
                        .align_y(ay);
                    if has_width {
                        cont = cont.width(iced::Length::Fill).align_x(ax);
                    }
                    cont.into()
                } else if has_width && (p050_ax.is_some() || p050_ay.is_some()) {
                    let mut c = iced::widget::container(button_content).width(iced::Length::Fill);
                    if let Some(ax) = p050_ax {
                        c = c.align_x(ax);
                    }
                    if let Some(ay) = p050_ay {
                        c = c.align_y(ay);
                    }
                    c.into()
                } else {
                    button_content
                };
                let mut btn = button(button_content);
                // Plan 423 P3: disabled 态 —— 不挂 on_press(点击无消息);
                // inspect 捕获模式同样不挂(原语义)。灰样式在下方 style 段追加。
                if !inspect_capture_active() && !disabled {
                    btn = btn.on_press(onclick);
                }

                // Apply visual styling to button. Always apply a class-driven
                // style: build_button_style yields a chromeless (transparent)
                // button when no background/border class is present, so a
                // text-only button (e.g. `button(variant: "text")` or any button
                // with only text classes) renders as text instead of iced's
                // default Primary (blue).
                if let Some(ref is) = iced_style {
                    let bs = build_button_style(is);
                    // `hover:` utilities: build a hovered variant with the
                    // hover classes applied on top of the base classes
                    // (later application wins, so hover:bg-* overrides the
                    // base background). iced re-invokes this closure on
                    // status changes — native hover feedback without a view
                    // rebuild (e.g. block header icon buttons light up via
                    // hover:bg-white/10). hover:text-* here only reaches the
                    // button's own label/text_color — icons recolor through
                    // their own svg Status styling (see Image lucide arm /
                    // PUA icon arm, 2026-08-21 方案 A), other pre-built child
                    // content is unaffected.
                    // Plan 527 T6: 变体管道泛化 —— base+变体合并构建各状态样式(变体后应用
                    // 胜出,`hover:` 先例同构);无声明时等价 base。iced 状态
                    // 回调面:Hovered→hover、Pressed→active(缺 active 声明时
                    // 回 hover,保持 legacy 行为)、Focused→focus、
                    // Disabled→disabled。focus: iced 0.14 button Status 无
                    // Focused 档(上限),类可声明可合并(IcedStyle::
                    // merged_with_variant),按钮回调面登记 parsed-only(KNOWN-DEBT)。
                    let bs_hover = match style.as_ref() {
                        Some(s) if s.has_variant(crate::ui::style::Variant::Hover) => {
                            build_button_style(&IcedStyle::merged_with_variant(
                                s,
                                crate::ui::style::Variant::Hover,
                            ))
                        }
                        _ => bs,
                    };
                    let bs_active = match style.as_ref() {
                        Some(s) if s.has_variant(crate::ui::style::Variant::Active) => {
                            build_button_style(&IcedStyle::merged_with_variant(
                                s,
                                crate::ui::style::Variant::Active,
                            ))
                        }
                        _ => bs_hover,
                    };
                    let bs_disabled = match style.as_ref() {
                        Some(s) if s.has_variant(crate::ui::style::Variant::Disabled) => {
                            build_button_style(&IcedStyle::merged_with_variant(
                                s,
                                crate::ui::style::Variant::Disabled,
                            ))
                        }
                        _ => bs,
                    };
                    // Plan 423 P3: disabled 灰态 —— 文本色压成 zinc-500(全部
                    // 状态压灰,禁用不应有可点提示)。
                    let (mut bs, mut bs_hover, mut bs_active, mut bs_disabled) =
                        (bs, bs_hover, bs_active, bs_disabled);
                    if disabled {
                        let gray = iced::Color::from_rgb8(113, 113, 122); // zinc-500
                        bs.text_color = gray;
                        bs_hover.text_color = gray;
                        bs_active.text_color = gray;
                        bs_disabled.text_color = gray;
                    }
                    btn = btn.style(move |_, status| match status {
                        iced::widget::button::Status::Hovered => bs_hover,
                        iced::widget::button::Status::Pressed => bs_active,
                        iced::widget::button::Status::Disabled => bs_disabled,
                        _ => bs,
                    });
                    if fixed_both {
                        // Plan 057 (ash-gui VM 修复): 双固定尺寸按钮内边距清零
                        // (preset 的 px-4=16px×2 会把 w-6=24px 的内容区压到 ≤0,
                        // icon 不绘制);内容居中由上方 wrapper 负责。
                        btn = btn.padding(0.0);
                    } else if let Some(px) = is.padding {
                        btn = btn.padding(px);
                    } else if is.padding_x.is_some() || is.padding_y.is_some() {
                        let px_x = is.padding_x.unwrap_or(8.0);
                        let px_y = is.padding_y.unwrap_or(4.0);
                        btn = btn.padding([px_y, px_x]);
                    } else {
                        // Plan 402: default padding so tiny-content buttons
                        // (e.g. a minesweeper cell with " ") have a clickable
                        // size instead of collapsing to ~0.
                        btn = btn.padding(8.0);
                    }
                    if let Some(ref w) = is.width {
                        btn = btn.width(iced_length(w));
                    }
                    if let Some(ref h) = is.height {
                        btn = btn.height(iced_length(h));
                    }
                } else {
                    // No style prop at all: chromeless instead of iced Primary.
                    // Plan 408: chromeless + dark-mode-aware text color.
                    let default_text = crate::ui::style_iced_adapter::resolve_semantic_rgb(
                        &crate::ui::style::Color::OnBackground,
                    )
                    .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                    .unwrap_or(iced::Color::WHITE);
                    btn = btn.style(move |_, _| iced::widget::button::Style {
                        background: None,
                        text_color: default_text,
                        ..Default::default()
                    });
                }

                // Plan 409 §10 续 14/17: button 支持 width/flex-1 + height(让
                // component-card 等宽、button size sm/lg 区分高度)。
                // flex-1 在 iced_adapter 里设 is.width=Fill,这里统一用 is.width/height。
                if let Some(ref is) = iced_style {
                    if let Some(ref w) = is.width {
                        btn = btn.width(iced_length(w));
                    }
                    if let Some(ref h) = is.height {
                        btn = btn.height(iced_length(h));
                    }
                }

                // Plan 402: wrap button in mouse_area for right-click (contextmenu)
                // support. iced's button has no native right-click event; mouse_area's
                // on_right_press fires on right mouse button press.
                let el: iced::Element<'static, M> = btn.into();
                let el: iced::Element<'static, M> = if let Some(right_msg) = on_right_click {
                    if !inspect_capture_active() {
                        mouse_area(el).on_right_press(right_msg).into()
                    } else {
                        el
                    }
                } else {
                    el
                };
                // Wrap in container if margin_top (from mt-*) needs to be applied
                let el: iced::Element<'static, M> = if let Some(ref is) = iced_style {
                    wrap_with_margin(el, is)
                } else {
                    el
                };
                // Plan 418 P2-3 续: EE03 tooltip — icon-only toolbar buttons
                // show their action title in a small dark rounded bubble
                // below the icon while hovered (menu-panel palette).
                if let Some(tip) = pua_tooltip {
                    iced::widget::tooltip(
                        el,
                        iced::widget::text(tip).size(12.0),
                        iced::widget::tooltip::Position::Bottom,
                    )
                    .gap(6.0)
                    // 300ms delay: iced Tooltip opens with invalidate_layout
                    // at delay=ZERO — a hover instantly rebuilds the tree and
                    // kills any in-flight button press (toolbar icons became
                    // unclickable). Resting hover still shows the bubble.
                    .delay(std::time::Duration::from_millis(300))
                    .style(|_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb8(
                            0x16, 0x17, 0x1B,
                        ))),
                        border: iced::Border {
                            color: iced::Color::from_rgb8(0x3F, 0x3F, 0x46),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        text_color: Some(iced::Color::from_rgb8(0xE4, 0xE4, 0xE7)),
                        ..Default::default()
                    })
                    .into()
                } else {
                    el
                }
            }

            AbstractView::Row {
                children,
                spacing,
                padding,
                style,
                onclick,
                on_right_click,
            } => {
                // 轴向修正(概要页对拍,EDGE-16 家族):见 axis_fix_row_child。
                // PLAN-022 T-03:absolute 子脱流叠层(镜像 render_dynamic_view
                // Row 臂/PLAN-530 步骤3)——into_iced 此前无分区,absolute 子
                // 落流式布局(rust 轨 Component 路径专有缺口;app.at 分屏槽位
                // 实测全叠原点/挤出视口,槽位矩形模型侧正确)。
                let hover = layout_hover_flag(style.as_ref());
                let mut normal: Vec<iced::Element<'static, M>> = Vec::new();
                let mut absolute: Vec<AbstractView<M>> = Vec::new();
                for child in children {
                    let is_abs = extract_view_style(&child)
                        .map(|s| s.classes.iter().any(|c| matches!(c, StyleClass::Absolute)))
                        .unwrap_or(false);
                    if is_abs {
                        absolute.push(child);
                    } else {
                        normal.push(axis_fix_row_child(child).into_iced());
                    }
                }
                let base = build_row(
                    normal,
                    spacing,
                    padding,
                    style.as_ref(),
                    None,
                    hover.clone(),
                );
                let el: iced::Element<'static, M> = if absolute.is_empty() {
                    base
                } else {
                    let mut stk = iced::widget::Stack::new().push(base);
                    for child in absolute {
                        // PLAN-051 P2: 空层不渲染不入栈(同 render_dynamic_view)。
                        if is_empty_stack_layer(&child) {
                            continue;
                        }
                        // PLAN-536 T10: 偏移判定在 move 前取好(child 随分区 move)。
                        let pos = dynamic_abs_layer_position(&child);
                        // PLAN-095 T-03: 被动框穿透（pointer-events-none 根
                        // 不包 opaque，不截获下方点选）+ 百分比几何入参。
                        let passthrough = abs_layer_passthrough(&child);
                        let units = overlay_percent_units(&child);
                        let abs_el = child.into_iced();
                        // PLAN-536 T10: 非零偏移浮层消费 offset(× 落左上)。
                        // PLAN-023 T-02: 带偏移分支的 opaque 已在
                        // build_floating_layer 内下沉到 content 级(spacer
                        // 空白区穿透);零偏移层无 spacer,外层 opaque 边界
                        // 本=内容矩形,保持（穿透根除外）。
                        let abs_el = match pos {
                            Some(pos) => {
                                build_floating_layer_full(abs_el, pos, !passthrough, units)
                            }
                            None if !passthrough => iced::widget::opaque(abs_el),
                            None => abs_el,
                        };
                        stk = stk.push(abs_el);
                    }
                    let clip = style
                        .as_ref()
                        .map(|s| {
                            s.classes
                                .iter()
                                .any(|c| matches!(c, StyleClass::OverflowHidden))
                        })
                        .unwrap_or(false);
                    stk.clip(clip).into()
                };
                // Plan 490 G4：布局件点击（row/col/div onclick parity）。
                // PLAN-002 B：右键 + hover 同包装点（wrap_layout_events）。
                wrap_layout_events(el, onclick, on_right_click, hover)
            }

            AbstractView::Column {
                children,
                spacing,
                padding,
                style,
                onclick,
                on_right_click,
            } => {
                // 轴向修正:flex-1 主轴语义转写,见 axis_fix_col_child。
                // PLAN-050 P2 #3: mt-auto → 列内弹性占位——此前 margin 系静默
                // 跳过,`mt-auto` 工具行（rail 底部工具栏）失去贴底语义。
                // PLAN-054 T4 (A6): 子项 self-end/start/center → 交叉轴(横向)
                // Fill+align 包裹——iced 列无 per-child 对齐,消息行
                // `self-end` 右对齐此前被静默丢弃。
                // PLAN-642 T-11: distributed 列（justify-between 族）子项 grow
                // 剥离,防 iced 0.14 配给制 0×0 隐没（见 axis_fix_col_child_distributed）。
                let distributed = style_is_distributed(style.as_ref());
                // PLAN-022 T-03:absolute 子脱流叠层(镜像 render_dynamic_view
                // Column 臂/PLAN-530 步骤3;同 Row 臂注——rust 轨 Component
                // 路径 app.at 分屏槽位/分隔条全靠它)。
                let (flow_idx, abs_idx) = column_layer_partition(&children);
                let mut els: Vec<iced::Element<'static, M>> = Vec::new();
                for i in &flow_idx {
                    let c = &children[*i];
                    let mt_auto =
                        extract_view_style(c).map_or(false, |s| plan050_mt_auto_spacer(&s.classes));
                    if mt_auto {
                        els.push(iced::widget::Space::new().height(iced::Length::Fill).into());
                    }
                    let align_self = extract_view_style(c).and_then(|s| {
                        crate::ui::style_iced_adapter::IcedStyle::from_style(s).align_self
                    });
                    let el = if distributed {
                        axis_fix_col_child_distributed(c.clone())
                    } else {
                        axis_fix_col_child(c.clone())
                    }
                    .into_iced();
                    let el = match align_self {
                        Some(crate::ui::style_iced_adapter::IcedAlign::End) => {
                            iced::widget::container(el)
                                .width(iced::Length::Fill)
                                .align_x(iced::alignment::Horizontal::Right)
                                .into()
                        }
                        Some(crate::ui::style_iced_adapter::IcedAlign::Start) => {
                            iced::widget::container(el)
                                .width(iced::Length::Fill)
                                .align_x(iced::alignment::Horizontal::Left)
                                .into()
                        }
                        Some(crate::ui::style_iced_adapter::IcedAlign::Center) => {
                            iced::widget::container(el)
                                .width(iced::Length::Fill)
                                .align_x(iced::alignment::Horizontal::Center)
                                .into()
                        }
                        _ => el,
                    };
                    els.push(el);
                }
                // Plan 490 G4：同 Row（PLAN-002 B：右键 + hover 同包装点）。
                let hover = layout_hover_flag(style.as_ref());
                let base = build_column(els, spacing, padding, style.as_ref(), None, hover.clone());
                let el: iced::Element<'static, M> = if abs_idx.is_empty() {
                    base
                } else {
                    let mut stk = iced::widget::Stack::new().push(base);
                    for &i in &abs_idx {
                        // PLAN-051 P2: 空层不渲染不入栈(挡死下层交互件聚焦/点击)。
                        if is_empty_stack_layer(&children[i]) {
                            continue;
                        }
                        // PLAN-095 T-03: 被动框穿透 + 百分比几何入参。
                        let passthrough_b = abs_layer_passthrough(&children[i]);
                        let units_b = overlay_percent_units(&children[i]);
                        let abs_el = children[i].clone().into_iced();
                        // PLAN-536 T10: 非零偏移浮层消费 offset(× 落左上根修);
                        // 零偏移(inset-0 ghost 族)保持落原点。
                        // PLAN-023 T-02: 同 Row 臂——opaque 下沉 content 级,
                        // 零偏移分支维持外层 opaque（穿透根除外）。
                        let abs_el = match dynamic_abs_layer_position(&children[i]) {
                            Some(pos) => {
                                build_floating_layer_full(abs_el, pos, !passthrough_b, units_b)
                            }
                            None if !passthrough_b => iced::widget::opaque(abs_el),
                            None => abs_el,
                        };
                        stk = stk.push(abs_el);
                    }
                    let clip = style
                        .as_ref()
                        .map(|s| {
                            s.classes
                                .iter()
                                .any(|c| matches!(c, StyleClass::OverflowHidden))
                        })
                        .unwrap_or(false);
                    stk.clip(clip).into()
                };
                wrap_layout_events(el, onclick, on_right_click, hover)
            }

            AbstractView::Input {
                placeholder,
                value,
                on_change,
                on_submit,
                width,
                password,
                style,
                disabled,
            } => {
                let mut input_widget =
                    build_input_shape(&placeholder, &value, width, password, style.as_ref(), disabled);

                // PLAN-031 T-05（F-02）：禁用态不派发编辑/提交消息（输入被忽略），
                // 表面由 style 的禁用呈现承担；与 Vue 侧原生 disabled 语义对齐。
                if !disabled {
                    // Wire on_input for text change tracking
                    if let Some(msg) = on_change {
                        input_widget = input_widget.on_input(move |text| {
                            INPUT_TEXT.with(|t| *t.borrow_mut() = text.to_string());
                            msg.clone()
                        });
                    }

                    // Wire on_submit for Enter key press
                    if let Some(msg) = on_submit {
                        input_widget = input_widget.on_submit(msg);
                    }
                }

                let el: iced::Element<'static, M> = input_widget.into();
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    wrap_with_margin(el, &iced_style)
                } else {
                    el
                }
            }

            AbstractView::Textarea {
                placeholder,
                value,
                on_change,
                on_submit,
                height,
                style,
                disabled,
                keymap,
                ..
            } => {
                // Plan 446 批五 U3:超大内容降级只读预览(rust 模式同款守卫,
                // 机理见 TEXTAREA_INLINE_EDIT_CAP)。
                if textarea_needs_preview(&value) {
                    let el = oversized_textarea_preview(&value);
                    if let Some(ref s) = style {
                        let is = IcedStyle::from_style(s);
                        wrap_with_margin(el, &is)
                    } else {
                        el
                    }
                } else {
                    let key = format!("__textarea_{}", placeholder.len());

                    let content = get_textarea_content(&key, &value);
                    let ph: &'static str = leaked_placeholder(&placeholder);
                    let mut editor = text_editor(content).placeholder(ph);
                    // PLAN-051 P2: 稳定 Id——iced text_editor 点击聚焦的前提
                    // (update 内 state.focus(id) 仅在 id 存在时执行;无 Id 则点
                    // 击永不聚焦=无光标无法输入,composer 命中区修复后仍复现
                    // 的最后缺口)。镜像 ash-gui 臂的 textarea_{key} 派生。
                    // PLAN-095 T-04 注：VM 动态主路径(render_dynamic_view)的
                    // Textarea 臂以 on_input handler 主键派生 Id（聚焦寻址面）；
                    // 本泛型臂保 legacy 键（非 IcedMessage 树不可读 handler）。
                    editor = editor
                        .id(iced::widget::Id::from(format!("textarea_{}", key)))
                        // 此前不消费,可见容器内可点区只有顶部 30px 条(composer 命中区)。
                        .height(textarea_editor_height(height, style.as_ref()));

                    // Plan 053 后续(ash-gui VM): apply class-driven text styling.
                    // Previously the whole style prop was ignored (`style: _`), so the
                    // Vue transparent-textarea + colored-overlay technique rendered the
                    // typed text TWICE — once in the editor's default color, once in
                    // the overlay spans. Applying text_color makes `text-transparent`
                    // hide the editor copy (caret rides the same color), leaving only
                    // the overlay; the placeholder stays a visible muted gray.
                    if let Some(ref s) = style {
                        let is = IcedStyle::from_style(s);
                        if let Some(fs) = effective_font_size(&is) {
                            editor = editor.size(fs);
                        }
                        // PLAN-031 T-05：placeholder 槽独立生效——只声明
                        // placeholder（无 text 色）也要装样式闭包。
                        if is.text_color.is_some() || is.placeholder_color.is_some() {
                            let mut value_color = is.text_color.unwrap_or_else(|| {
                                crate::ui::style::theme::resolve_semantic_rgb(
                                    &crate::ui::style::Color::OnBackground,
                                )
                                .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                                .unwrap_or(iced::Color::WHITE)
                            });
                            // PLAN-051 P2: 全透明 value 色(如 musk composer 的
                            // text-transparent+backdrop 叠加技法)在 VM 侧回退为可见
                            // 前景——VM 无 backdrop 渲染层,透明即"输入成功但不可视"。
                            // Vue 轨自渲染其 textarea,不受此臂影响。
                            if value_color.a <= 0.001 {
                                value_color = crate::ui::style::theme::resolve_semantic_rgb(
                                    &crate::ui::style::Color::OnBackground,
                                )
                                .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                                .unwrap_or(iced::Color::WHITE);
                            }
                            let value_color = value_color;
                            let placeholder_color = is
                                .placeholder_color
                                .unwrap_or(iced::Color::from_rgba(0.55, 0.58, 0.65, 0.7));
                            editor = editor.style(move |_theme, _status| {
                                iced::widget::text_editor::Style {
                                    background: iced::Background::Color(iced::Color::TRANSPARENT),
                                    border: iced::Border::default(),
                                    placeholder: placeholder_color,
                                    value: value_color,
                                    selection: iced::Color::from_rgba(0.3, 0.5, 0.9, 0.35),
                                }
                            });
                        }
                    }

                    // Plan 053 M4: Enter fires on_submit (onenter) — the newline is
                    // already inserted by content.perform; input_value carries the
                    // post-Enter content so the handler's bound field picks it up.
                    let is_enter = |action: &text_editor::Action| {
                        matches!(action, text_editor::Action::Edit(text_editor::Edit::Enter))
                    };
                    let el: iced::Element<'static, M> = if let Some(msg) = on_change {
                        let action_key = key.clone();
                        let submit_clone = on_submit.clone();
                        editor
                            .on_action(move |action| {
                                if is_enter(&action) {
                                    if let Some(sm) = submit_clone.clone() {
                                        let text = textarea_perform_action(&action_key, action);
                                        INPUT_TEXT.with(|t| *t.borrow_mut() = text);
                                        return sm.clone();
                                    }
                                }
                                let text = textarea_perform_action(&action_key, action);
                                INPUT_TEXT.with(|t| *t.borrow_mut() = text);
                                msg.clone()
                            })
                            .into()
                    } else {
                        editor.into()
                    };
                    if let Some(ref s) = style {
                        let is = IcedStyle::from_style(s);
                        wrap_with_margin(el, &is)
                    } else {
                        el
                    }
                }
            }

            AbstractView::CodeEditor {
                key,
                value,
                lang,
                line_numbers,
                wrap,
                vi,
                highlight_current_line,
                readonly,
                tab_width,
                font_size,
                on_change,
                on_cursor,
                on_context_menu,
                search,
                style: _,
            } => build_code_editor_generic(
                &key,
                &value,
                &lang,
                line_numbers,
                wrap,
                vi,
                highlight_current_line,
                readonly,
                tab_width,
                font_size,
                &search,
                on_change,
                on_cursor,
                on_context_menu,
            ),

            // PLAN-009 P1: terminal 组件——状态入注册表(terminal(key,…)),
            // feed 数据面甲(props)经 iced widget 每帧消费;T4 交互事件经
            // 固定消息上抛,载荷读注册表(selected_text/scroll_offset/menu)。
            // PLAN-656 T-06: synthetic managed content——logical extent 自绘
            // widget（host 注册表 draw 期 viewport 观察）。
            AbstractView::ManagedScrollContent {
                key,
                logical_w,
                logical_h,
                ..
            } => crate::ui::iced::managed_content::ManagedScrollContentWidget::new(
                key.clone(),
                logical_w,
                logical_h,
            )
            .into_element(),

            AbstractView::Terminal {
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
            } => {
                let core = crate::ui::terminal::terminal(&key, cols, rows);
                // PLAN-018 D10:scheme prop 随帧落注册表(显式 ≥0 覆盖;
                // -1 = 跟随主题,绘制期解析)。
                core.set_scheme(scheme);
                crate::ui::terminal::terminal_feed(core, &lines);
                // PLAN-022 T-03 探针(DBG 门控):喂入面数值(key/行数/首行)。
                if std::env::var("AUTO_MA_DBG")
                    .map(|v| v == "1")
                    .unwrap_or(false)
                {
                    let first = lines
                        .first()
                        .map(|l| l.chars().take(40).collect::<String>())
                        .unwrap_or_default();
                    eprintln!(
                        "[P22-FEED] key={key} lines={} cols={cols} rows={rows} first={first:?}",
                        lines.len()
                    );
                }
                // PLAN-022:scroll_offset prop 非零才落位(014 光标哨兵同
                // 款)——0 恒写会把引擎泵回写的 display_offset 每帧清零,
                // 官方滚动条的读出基线(scroll_view_target)随之断裂。
                if scroll_offset != 0 {
                    crate::ui::terminal::terminal_set_scroll_offset(core, scroll_offset as usize);
                }
                // 014:光标格随帧落注册表(app 从引擎回读喂入;preedit/光标
                // 层同源)。形状恒 Block;非零才落位——(0,0) 是未喂入哨兵,
                // 直接设在 core 上的光标(测试/宿主策略)不被覆写。
                if cursor_row != 0 || cursor_col != 0 {
                    crate::ui::terminal::terminal_set_cursor(
                        core,
                        cursor_row as usize,
                        cursor_col as usize,
                        crate::ui::terminal::TermCursorShape::Block,
                    );
                }
                // PLAN-028 T-05:回滚历史随帧落注册表(app 从 nums 尾段回读
                // 喂入;0 = 未喂入哨兵)。虚拟画布 =(rows+history)×CELL_H
                // 随之复原,iced scrollable 恢复"内容高于视口出条"。
                if history > 0 {
                    crate::ui::terminal::terminal_set_history(core, history as usize);
                }
                let el: iced::Element<'static, M> = crate::ui::terminal::iced::Terminal {
                    core,
                    key: key.clone(),
                    scroll_offset,
                    preedit: preedit.clone(),
                    on_select: on_select.clone(),
                    on_menu: on_menu.clone(),
                    on_input: on_input.clone(),
                    shortcuts: shortcuts.clone(),
                    // PLAN-022 T-02 换装:虚拟滚动模式 + 官方 scrollable
                    // 包装(见下);width 仍为 016 网格公式。
                    virtual_scroll: true,
                    width: iced::Length::Fixed(
                        cols as f32 * crate::ui::terminal::iced::cell_w()
                            + 2.0 * crate::ui::terminal::iced::PAD,
                    ),
                    height: iced::Length::Fixed(
                        rows as f32 * crate::ui::terminal::iced::CELL_H
                            + 2.0 * crate::ui::terminal::iced::PAD,
                    ),
                }
                .into();
                // 固定网格尺寸 ≠ 客户区:右/底余量(≤一格宽/一行高)若露出
                // 根容器 bg-background(9,14,26) 即用户可见"浅色带"。涂同色
                // (终端 DEFAULT_BG)填满可用空间,余量隐形;子件左上对齐,
                // PAD 贴窗角。016 复审 T-07。
                // PLAN-018 D10:余量涂色随 scheme 表(light 方案余量同浅底)。
                let margin_bg = {
                    let scheme = crate::ui::terminal::terminal_resolve_scheme(core);
                    let palette = crate::ui::terminal::terminal_effective_palette(scheme);
                    let v = palette[1];
                    crate::ui::terminal::iced::rgb_u32(v)
                };
                let el: iced::Element<'static, M> = iced::widget::container(el)
                    // PLAN-022:容器入 scrollable(滚动轴内容高=虚拟画布,
                    // Shrink=自然高;Fill 在无穷轴会塌 0)。宽 Fill 仍吃
                    // 视口宽,右缘余量条由容器同色涂满。
                    // PLAN-024:容器底色圆角 = 虚拟窗框 WIN_RADIUS——
                    // 方角余量条/画布 bg 在窗框圆角处探出 app 边界(用户
                    // 实机门实录);同色叠层圆角化后角落共同透明,视觉
                    // 与窗框圆角吻合。
                    .width(iced::Length::Fill)
                    .height(iced::Length::Shrink)
                    .align_x(iced::alignment::Horizontal::Left)
                    .align_y(iced::alignment::Vertical::Top)
                    .style(move |_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(margin_bg)),
                        border: iced::Border {
                            color: iced::Color::TRANSPARENT,
                            width: 0.0,
                            radius: 16.0.into(),
                        },
                        ..Default::default()
                    })
                    .into();
                // PLAN-022 T-02 写臂(引擎 → 视图):引擎 display_offset
                // 变化(键入贴底/方案切换/引擎侧滚动)≠ 上次绑定值 → 经
                // Plan 043 pending 队列发 scroll_to,thumb 跟随、单拍收敛;
                // 绑定即 scroll_to 回声,置抑制让读出臂(draw 期 viewport
                // 观察)吞一次,防回灌环路。首帧 bound=-1 → 强制绑定贴底
                // (终端惯例:打开即实时视图)。
                let scroll_key = format!("terminal_scroll_{}", key);
                if let Some(y) = <crate::ui::terminal::iced::Terminal<M>>::bind_request_y(core) {
                    if std::env::var("P024_TRACE").is_ok() {
                        eprintln!(
                            "[P024-TRACE] bind key={key} scroll_to y={y:.1} hist={} off={}",
                            crate::ui::terminal::terminal_history(core),
                            crate::ui::terminal::terminal_scroll_offset(core)
                        );
                    }
                    note_scroll_offset(&scroll_key, (0.0, y));
                }
                // 官方 scrollable 包装:滚动条视觉/交互(scrollbar_style
                // 现款)归 iced;读出臂在 widget draw 期(PLAN-022 语义:
                // scrollable offset = 视图投影,引擎 display_offset 单源)。
                let el: iced::Element<'static, M> = iced::widget::scrollable(el)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .id(scroll_key)
                    .style(
                        |_theme: &iced::Theme, _status: iced::widget::scrollable::Status| {
                            scrollbar_style()
                        },
                    )
                    .into();
                // PLAN-024(用户实机门实录):视口级底色容器——画布高
                // (rows×CELL_H+2PAD)与视口高的**非整行残差**(DPI 下可达
                // 一行)在 scrollable 底部露出 app 根浅色。外层 Fill 容器
                // 同色涂满视口消残差带;**方角**(2026-09-20 裁定:窗框
                // 圆角由 app.at 底部状态栏 rounded-b-2xl 承担,分屏拓扑
                // 无关——per-pane 圆角在横分下中间缝两角误圆)。
                let el: iced::Element<'static, M> = iced::widget::container(el)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .style(move |_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(margin_bg)),
                        ..Default::default()
                    })
                    .into();
                if let Some(ref s) = style {
                    let is = IcedStyle::from_style(s);
                    wrap_with_margin(el, &is)
                } else {
                    el
                }
            }

            AbstractView::AutodownEditor {
                key,
                value,
                is_final,
                on_change,
                on_focus,
                on_link,
                placeholder,
                style: _,
            } => build_autodown_editor_generic(
                &key,
                &value,
                is_final,
                on_change,
                on_focus,
                on_link,
                placeholder,
            ),

            AbstractView::Checkbox {
                is_checked,
                label,
                on_toggle,
                style,
            } => {
                let iced_style = style.as_ref().map(IcedStyle::from_style);

                let cb_size = iced_style
                    .as_ref()
                    .and_then(|is| match (&is.width, &is.height) {
                        (Some(IcedSize::Fixed(w)), Some(IcedSize::Fixed(h))) => Some(w.min(*h)),
                        (Some(IcedSize::Fixed(w)), None) => Some(*w),
                        (None, Some(IcedSize::Fixed(h))) => Some(*h),
                        _ => None,
                    })
                    .unwrap_or(20.0);

                let primary_color = iced_style
                    .as_ref()
                    .and_then(|is| is.text_color)
                    .unwrap_or(iced::Color::from_rgb8(59, 130, 246)); // blue-500 or text-{color}
                let border_radius = iced_style
                    .as_ref()
                    .map(|is| is.effective_border_radius())
                    .unwrap_or(4.0.into());
                let custom_border_color = iced_style.as_ref().and_then(|is| is.border_color);

                let mut checkbox_widget = checkbox(is_checked).size(cb_size);

                checkbox_widget = checkbox_widget.style(move |_theme, status| {
                    let is_checked = match status {
                        iced::widget::checkbox::Status::Active { is_checked }
                        | iced::widget::checkbox::Status::Hovered { is_checked }
                        | iced::widget::checkbox::Status::Disabled { is_checked } => is_checked,
                    };
                    let is_hovered =
                        matches!(status, iced::widget::checkbox::Status::Hovered { .. });

                    let border_color = if is_checked {
                        primary_color
                    } else if is_hovered {
                        custom_border_color.unwrap_or(iced::Color::from_rgb8(156, 163, 175))
                    // gray-400
                    } else {
                        custom_border_color.unwrap_or(iced::Color::from_rgb8(209, 213, 219))
                        // gray-300
                    };

                    let background = if is_checked {
                        iced::Background::Color(primary_color)
                    } else {
                        iced::Background::Color(iced::Color::TRANSPARENT)
                    };

                    let icon_color = if is_checked {
                        iced::Color::WHITE
                    } else {
                        iced::Color::TRANSPARENT
                    };

                    iced::widget::checkbox::Style {
                        background,
                        icon_color,
                        border: iced::Border {
                            color: border_color,
                            width: 1.5,
                            radius: border_radius,
                        },
                        text_color: None,
                    }
                });

                // Plan 309 续篇 II: drop the handler in inspect-capture mode so
                // the checkbox is non-interactive (wrap_debug mouse_area picks).
                let handler = if inspect_capture_active() {
                    None
                } else {
                    on_toggle
                };
                let checkbox_with_handler = if let Some(msg) = handler {
                    checkbox_widget.on_toggle(move |_| msg.clone())
                } else {
                    checkbox_widget
                };

                // Apply text style to label
                let mut label_widget = text(label.clone());
                if let Some(ref is) = iced_style {
                    if let Some(fs) = effective_font_size(is) {
                        label_widget = label_widget.size(fs);
                    }
                    if let Some(color) = is.text_color {
                        label_widget = label_widget.color(color);
                    }
                }

                let el: iced::Element<'static, M> = if label.is_empty() {
                    let mut cont = container(checkbox_with_handler)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center);
                    if let Some(ref is) = iced_style {
                        if let Some(ref w) = is.width {
                            cont = cont.width(iced_length(w));
                        }
                        if let Some(ref h) = is.height {
                            cont = cont.height(iced_length(h));
                        }
                    }
                    cont.into()
                } else {
                    let mut row_widget = row![checkbox_with_handler, label_widget]
                        .align_y(iced::alignment::Vertical::Center)
                        .spacing(4);
                    if let Some(ref is) = iced_style {
                        if let Some(ref w) = is.width {
                            row_widget = row_widget.width(iced_length(w));
                        }
                        if let Some(ref h) = is.height {
                            row_widget = row_widget.height(iced_length(h));
                        }
                    }
                    row_widget.into()
                };

                if let Some(ref is) = iced_style {
                    wrap_with_margin(el, is)
                } else {
                    el
                }
            }

            AbstractView::Container {
                child,
                padding,
                width,
                height,
                center_x,
                center_y,
                style,
                onclick,
                on_right_click,
            } => {
                // Plan 490 G4：div/Container 点击（musk specs 树形态消费面）。
                // PLAN-002 B：右键 + hover 同包装点。
                let hover = layout_hover_flag(style.as_ref());
                wrap_layout_events(
                    build_container(
                        child.into_iced(),
                        padding,
                        width,
                        height,
                        center_x,
                        center_y,
                        style.as_ref(),
                        None,
                        hover.clone(),
                    ),
                    onclick,
                    on_right_click,
                    hover,
                )
            }

            AbstractView::Scrollable {
                child,
                width,
                height,
                style,
                auto_scroll,
                offset,
                on_scroll,
                axes,
                scrollbar_policy,
                controller,
            } => {
                // Plan 049 → 057 续:仅 `auto_scroll` 标记的主列表挂固定 Id
                // (snap_to_end 目标)。此前所有 Scrollable 共享该 Id,块内 max-h
                // 滚动区会抢先命中 snap/scroll 操作 → 自动滚动失灵。
                // Plan 043 T1:offset 写入要求稳定 id——auto_scroll 标记自带
                // blocklist_scroll;其余 offset 绑定由调用面(aura_view_builder
                // scroll_sync 臂)传入显式 id,无 id 的 offset 静默不写入。
                let scroll_id = if auto_scroll {
                    Some("blocklist_scroll".to_string())
                } else {
                    None
                };
                // PLAN-656: 泛型路径 controller 直传（测量缓存包装仅在动态
                // 臂可行——M 具体；此处无用户 on-scroll 时不缓存测量）。
                // PLAN-781 T-05: managed 桥标记（9922 unsupported 合同）。
                if let (Some(c), true) = (
                    controller.as_ref(),
                    content_is_managed_bridge(child.as_ref()),
                ) {
                    crate::ui::scroll::controller::mark_controller_managed(c.key());
                }
                build_scrollable(
                    child.into_iced(),
                    width,
                    height,
                    style.as_ref(),
                    scroll_id,
                    offset,
                    on_scroll,
                    axes,
                    scrollbar_policy,
                    controller.as_ref(),
                )
            }

            AbstractView::Grid {
                cols,
                gap,
                cells,
                style,
            } => {
                let els: Vec<(iced::Element<'static, M>, GridCellSpec)> = cells
                    .into_iter()
                    .map(|c| {
                        let spec = grid_cell_spec(&c);
                        (c.into_iced(), spec)
                    })
                    .collect();
                build_grid(cols, gap, els, style.as_ref(), None)
            }

            // Plan 409 §10 续 5: Overlay = iced Stack 分层。base 在底,content 浮
            // 在上层(按 position 定位),不挤压 base 布局。opaque 吃点击穿透。
            AbstractView::Overlay {
                base,
                content,
                position,
            } => {
                let base_el = base.into_iced();
                // PLAN-023 T-02: build_floating_layer 已含 content 级 opaque,
                // 不再包外层(spacer 空白区穿透到 base,命中=内容矩形)。
                // PLAN-095 T-03: Overlay content 穿透判定（被动框不拦点选）。
                let overlay_passthrough = abs_layer_passthrough(&content);
                let overlay_units = overlay_percent_units(&content);
                let content_el = build_floating_layer_full(
                    content.into_iced(),
                    position,
                    !overlay_passthrough,
                    overlay_units,
                );
                iced::widget::stack![base_el, content_el].into()
            }

            // Plan 484: hover 命中区 —— iced mouse_area 透明包裹,仅转发
            // enter/exit 消息(通用事件分发)。style(尺寸/定位类)经
            // build_container 承载——命中区必须定宽高,空内容才有可命中面积。
            // Plan 499 M2: 带 on_move 时再包 PointerArea——坐标换算(bounds
            // px → coords 逻辑幅面)+ ≤30Hz 限频 + 量化去重;不带 on_move
            // 的存量 mouse-area 映射零改动。
            AbstractView::MouseArea {
                content,
                on_enter,
                on_exit,
                on_double_click,
                on_click,
                on_context_menu,
                on_release,
                on_move,
                logical_extent,
                style,
            } => {
                // PLAN-021 T-05 取证(AUTO_MA_DBG=1 门控):构建面接线状态。
                if std::env::var("AUTO_MA_DBG")
                    .map(|v| v == "1")
                    .unwrap_or(false)
                {
                    let (sw, sh) = style
                        .as_ref()
                        .map(|s| {
                            let is = IcedStyle::from_style(s);
                            (format!("{:?}", is.width), format!("{:?}", is.height))
                        })
                        .unwrap_or_else(|| ("None".into(), "None".into()));
                    eprintln!(
                        "[MA_BUILD] press={} release={} dbl={} rclick={} move={} w={sw} h={sh}",
                        on_click.is_some(),
                        on_release.is_some(),
                        on_double_click.is_some(),
                        on_context_menu.is_some(),
                        on_move.is_some()
                    );
                }
                // PLAN-021 线 B 根修:iced 0.14 mouse_area layout 直通子件,
                // 事件面 `!cursor.is_over(自身 bounds)` 即早退——尺寸类原挂
                // 外层 build_container(空内容 → 自身 0×0 bounds),press/
                // hover 全死而渲染正常(020 双区探针计数 0 与 split 分隔条
                // 拖拽死的断点)。内容侧镜像一个仅承载宽高的透明容器,使命
                // 中区=可视区;外层树形与样式归属不动(absolute 抬升/z-order
                // /绘制均不感知本改动)。
                let sized_content = match style.as_ref() {
                    Some(s) => {
                        let is = IcedStyle::from_style(s);
                        let mut c = iced::widget::container(content.into_iced());
                        if let Some(ref ws) = is.width {
                            c = c.width(iced_length(ws));
                        }
                        if let Some(ref hs) = is.height {
                            c = c.height(iced_length(hs));
                        }
                        c.into()
                    }
                    None => content.into_iced(),
                };
                let mut ma = mouse_area(sized_content);
                if let Some(msg) = on_enter {
                    ma = ma.on_enter(msg);
                }
                if let Some(msg) = on_exit {
                    ma = ma.on_exit(msg);
                }
                // Plan 496 M5: ondblclick → iced mouse_area on_double_click
                //（桌面图标双击启动原语；inspect 捕获态丢臂与其余 handler 同规则）。
                if !inspect_capture_active() {
                    if let Some(msg) = on_double_click {
                        ma = ma.on_double_click(msg);
                    }
                    // Plan 498 M0: onclick → iced mouse_area on_press(chart
                    // legend 点击切换显隐;同 inspect 捕获态丢臂规则)。
                    // PLAN-043 T9: on_click 槽由 aura 臂按 onmousedown 优先
                    // 填充（真按下语义）；onmouseup → on_release（拖拽收尾）。
                    if let Some(msg) = on_click {
                        ma = ma.on_press(msg);
                    }
                    // PLAN-526 T10: oncontextmenu → iced on_right_press
                    //（桌面空白/任务栏右键菜单；同 inspect 捕获态丢臂规则）。
                    if let Some(msg) = on_context_menu {
                        ma = ma.on_right_press(msg);
                    }
                    if let Some(msg) = on_release {
                        ma = ma.on_release(msg);
                    }
                }
                let inner: iced::Element<'static, M> = ma.into();
                let wrapped: iced::Element<'static, M> =
                    if let Some(handler) = on_move.filter(|_| !inspect_capture_active()) {
                        let mut pa = crate::ui::iced::pointer_area::PointerArea::new(inner);
                        if let Some((w, h)) = logical_extent {
                            pa = pa.extent(w, h);
                        }
                        let f = std::sync::Arc::new(move |x: f32, y: f32| handler.call(x, y));
                        pa.on_move(f).into()
                    } else {
                        inner
                    };
                // PLAN-631 F-5：mouse-area hover 样式对——`hover:` 变体类经
                // HoverArea + 共享标志零重建翻转（镜像布局件臂；无声明 =
                // None，零开销路径不变）。
                let hover = layout_hover_flag(style.as_ref());
                let built = build_container(
                    wrapped,
                    0,
                    None,
                    None,
                    false,
                    false,
                    style.as_ref(),
                    None,
                    hover.clone(),
                );
                match hover {
                    Some(flag) => crate::ui::iced::hover_area::HoverArea::new(built, flag).into(),
                    None => built,
                }
            }

            // Plan 563: 状态驱动画布 —— CanvasPainter(canvas::Program 直绘,
            // DrawListPainter 形态参考但独立实现:不经 DrawOp 线协议,v1
            // 无路径 op)+ PenArea 事件层(pen 三件套,按下门控/限频/出界
            // 收笔)。inspect 捕获态丢事件臂(与其余 handler 同规则)。
            // PLAN-661 T-05：PenArea 增 tap 命中层（onhit + canvas_hit_test）。
            AbstractView::Canvas {
                scene,
                logical_extent,
                clear,
                on_pen_start,
                on_pen_move,
                on_pen_end,
                on_hit,
                style,
            } => {
                let painter = CanvasPainter {
                    scene: scene.clone(),
                    clear: clear.clone(),
                    extent: logical_extent,
                };
                // iced canvas 默认 Fixed(DEFAULT_SIZE) 方块——显式 Fill
                // 随外层 container(承载 style 尺寸类 w-/h-)撑满。
                let canvas_el: iced::Element<'static, M> = iced::widget::canvas(painter)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .into();
                let wrapped: iced::Element<'static, M> = if !inspect_capture_active()
                    && (on_pen_start.is_some()
                        || on_pen_move.is_some()
                        || on_pen_end.is_some()
                        || on_hit.is_some())
                {
                    let mut pa = crate::ui::iced::pen_area::PenArea::new(canvas_el);
                    if let Some((w, h)) = logical_extent {
                        pa = pa.extent(w, h);
                    }
                    if let Some(h) = on_pen_start {
                        let f = std::sync::Arc::new(move |x: f32, y: f32| h.call(x, y));
                        pa = pa.on_pen_start(f);
                    }
                    if let Some(h) = on_pen_move {
                        let f = std::sync::Arc::new(move |x: f32, y: f32| h.call(x, y));
                        pa = pa.on_pen_move(f);
                    }
                    if let Some(h) = on_pen_end {
                        let f = std::sync::Arc::new(move |x: f32, y: f32| h.call(x, y));
                        pa = pa.on_pen_end(f);
                    }
                    // PLAN-661 T-05：tap 命中（R-1）——hit_test 从 scene
                    // nodes 构建（声明序倒序 topmost），PenArea 容差
                    // tap 判定后派发 onhit(id)。
                    if let Some(h) = on_hit {
                        let nodes = scene.nodes.clone();
                        let f = std::sync::Arc::new(move |id: String| h.call(id));
                        pa = pa
                            .on_hit(f)
                            .hit_test(std::sync::Arc::new(move |x: f32, y: f32| {
                                crate::ui::view::canvas_hit_test(&nodes, x, y)
                            }));
                    }
                    pa.into()
                } else {
                    canvas_el
                };
                build_container(
                    wrapped,
                    0,
                    None,
                    None,
                    false,
                    false,
                    style.as_ref(),
                    None,
                    None,
                )
            }

            // Plan 422: 锚定弹层 —— wrapper widget(Tooltip 同型),open 时
            // 经 iced overlay 机制置顶;chrome 由 content 自带。inspect 捕获
            // 模式下丢 on_dismiss(与其余 handler 同规则)。
            AbstractView::Popover {
                anchor,
                content,
                placement,
                open,
                on_dismiss,
            } => {
                use crate::ui::iced::popover::Popover as PopoverWidget;
                use crate::ui::view::PopoverAnchor;
                let (anchor_point, anchor_is_empty, anchor_el): (
                    Option<(f32, f32)>,
                    bool,
                    iced::Element<'static, M>,
                ) = match anchor {
                    PopoverAnchor::Widget(w) => {
                        let empty = matches!(&*w, AbstractView::Empty);
                        (None, empty, w.into_iced())
                    }
                    // 坐标锚:零尺寸占位(anchor 轨道不影响布局),
                    // 面板定位由 at_point 决定。
                    PopoverAnchor::Point { x, y } => {
                        (Some((x, y)), false, iced::widget::Space::new().into())
                    }
                };
                let mut p = PopoverWidget::new(anchor_el, content.into_iced())
                    .placement(placement)
                    .open(open)
                    // PLAN-530 步骤8（W13）：Modal 放置 = 模态形态（全屏遮罩
                    // + 面板外点击整吞），alert-dialog 臂专用。
                    .modal(placement.is_modal_chrome())
                    .anchor_is_empty(anchor_is_empty);
                if let Some((x, y)) = anchor_point {
                    p = p.at_point(x, y);
                }
                let handler = if inspect_capture_active() {
                    None
                } else {
                    on_dismiss
                };
                if let Some(msg) = handler {
                    p = p.on_dismiss(msg);
                }
                p.into()
            }

            AbstractView::Radio {
                label,
                is_selected,
                on_select,
                style,
            } => {
                let checkbox_widget = checkbox(is_selected);

                // Plan 309 续篇 II: drop the handler in inspect-capture mode.
                let handler = if inspect_capture_active() {
                    None
                } else {
                    on_select
                };
                let checkbox_with_handler = if let Some(msg) = handler {
                    checkbox_widget.on_toggle(move |_| msg.clone())
                } else {
                    checkbox_widget
                };

                // Apply text style to label
                let mut label_widget = text(label.clone());
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    if let Some(fs) = effective_font_size(&iced_style) {
                        label_widget = label_widget.size(fs);
                    }
                    if let Some(color) = iced_style.text_color {
                        label_widget = label_widget.color(color);
                    }
                }

                row![checkbox_with_handler, label_widget].spacing(4).into()
            }

            AbstractView::Select {
                options,
                selected_index,
                on_select,
                style: _,
            } => {
                let selected_value = selected_index.and_then(|i| options.get(i).cloned());

                // Plan 309 续篇 II: in inspect-capture mode, render as static
                // text (the None branch) so it doesn't capture the press.
                let on_select = if inspect_capture_active() {
                    None
                } else {
                    on_select
                };
                match on_select {
                    Some(callback) => {
                        let options_clone = options.clone();
                        let picklist_widget =
                            pick_list(options, selected_value, move |selected_string| {
                                let index = options_clone
                                    .iter()
                                    .position(|s| *s == selected_string)
                                    .unwrap_or(0);
                                callback.call(index, selected_string.as_str())
                            });
                        picklist_widget.into()
                    }
                    None => {
                        let display_text = selected_value
                            .unwrap_or_else(|| options.first().cloned().unwrap_or_default());
                        text(display_text).into()
                    }
                }
            }

            AbstractView::List {
                items,
                spacing,
                style,
            } => {
                let eff_spacing = effective_spacing(spacing, style.as_ref(), false);
                let eff_padding = if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    iced_style.padding.unwrap_or(0.0)
                } else {
                    0.0
                };

                let mut col_widget = column([]);
                col_widget = col_widget.spacing(eff_spacing);
                if eff_padding > 0.0 {
                    col_widget = col_widget.padding(eff_padding);
                }

                for item in items {
                    col_widget = col_widget.push(item.into_iced());
                }

                col_widget.into()
            }

            AbstractView::Table {
                headers,
                rows,
                spacing: _,
                col_spacing,
                style: _,
                table_key: _,
                col_widths,
                on_col_resize,
            } => {
                // Plan 045 T3 + PLAN-082 T-05: 表格统一分派 TableResize 自持
                // 网格 widget——on_col_resize Some=拖拽臂（临时宽实时重排 +
                // 松手 publish）；None=纯展示臂（自然宽列对齐——每列宽=该列
                // header+body 最大内容宽，跨行严格对齐。旧 per-row Row
                // lowering 各行独立 hug，行间列 x 漂移，截图实证）。
                // applied_widths（tablewidths 状态通道）两臂同样生效。
                let header_cells: Vec<_> = headers
                    .into_iter()
                    .map(|mut h| {
                        apply_table_header_style(&mut h);
                        h.into_iced()
                    })
                    .collect();
                let body_rows: Vec<Vec<_>> = rows
                    .into_iter()
                    .map(|r| r.into_iter().map(|c| c.into_iced()).collect())
                    .collect();
                return crate::ui::iced::table_resize::table_resize(
                    header_cells,
                    body_rows,
                    col_spacing as f32,
                    col_widths,
                    on_col_resize,
                )
                .into();
            }

            AbstractView::Slider {
                min,
                max,
                value,
                on_change,
                step,
                style: _,
            } => {
                // Plan 309 续篇 II: in inspect-capture mode render a static
                // read-out instead of the interactive slider (iced's slider
                // requires a callback). Cosmetic-only; 015-notes has no sliders.
                // PLAN-661 T-02: 无 on_change 同理——iced slider 强制
                // Fn(f32) -> Message 回调，泛型 M 无凭空 no-op 消息可造，
                // 无 handler = 无交互 = 静态读出（值可见）。
                if inspect_capture_active() || on_change.is_none() {
                    text(format!("{}", value)).into()
                } else {
                    use iced::widget::slider;
                    let Some(change_handler) = on_change else {
                        unreachable!("guarded above");
                    };
                    let mut slider_widget =
                        slider(min..=max, value, move |v| change_handler.call(v));

                    if let Some(step_value) = step {
                        slider_widget = slider_widget.step(step_value);
                    }

                    slider_widget.into()
                }
            }

            AbstractView::ProgressBar {
                progress,
                style,
                on_seek,
            } => {
                use iced::widget::progress_bar;
                let (is, height, width, radius) = if let Some(ref s) = style {
                    let is = IcedStyle::from_style(s);
                    let h = is
                        .height
                        .as_ref()
                        .map(iced_length)
                        .unwrap_or(iced::Length::Fixed(8.0));
                    let w = is
                        .width
                        .as_ref()
                        .map(iced_length)
                        .unwrap_or(iced::Length::Fill);
                    let r = if is.has_border_radius() || is.rounded {
                        is.effective_border_radius()
                    } else {
                        iced::border::Radius::from(9999.0)
                    };
                    (Some(is), h, w, r)
                } else {
                    (
                        None,
                        iced::Length::Fixed(8.0),
                        iced::Length::Fill,
                        iced::border::Radius::from(9999.0),
                    )
                };

                let (pr_r, pr_g, pr_b) = crate::ui::style::theme::resolve_semantic_rgb(
                    &crate::ui::style::Color::Primary,
                )
                .unwrap_or((99, 102, 241));
                let bar_color = is
                    .as_ref()
                    .and_then(|i| i.text_color.or(i.background_color))
                    .unwrap_or_else(|| iced::Color::from_rgb8(pr_r, pr_g, pr_b));
                let (bg_r, bg_g, bg_b) = crate::ui::style::theme::resolve_semantic_rgb(
                    &crate::ui::style::Color::Secondary,
                )
                .unwrap_or((226, 232, 240));
                let bg_color = iced::Color::from_rgb8(bg_r, bg_g, bg_b);

                let pb = progress_bar(0.0..=1.0, progress)
                    .length(width)
                    .style(move |_theme| progress_bar::Style {
                        background: iced::Background::Color(bg_color),
                        bar: iced::Background::Color(bar_color),
                        border: iced::Border {
                            radius,
                            ..Default::default()
                        },
                    });

                let cont = container(pb).width(width).height(height);
                // `onseek` 存在时整条进度条变成可点/可拖的 seek 控件：
                // 按下或按住拖动 → 交 0..1 的横向比例（SeekArea 自己记按下态，
                // 悬停不 scrub）。没有 `onseek` 时是纯展示条，零行为差异。
                let seekable: iced::Element<'_, M> = if let Some(handler) = on_seek.clone() {
                    let f = std::sync::Arc::new(move |frac: f32| handler.call(frac, 0.0));
                    crate::ui::iced::seek_area::SeekArea::new(cont)
                        .on_seek(f)
                        .into()
                } else {
                    cont.into()
                };
                if let Some(ref is_ref) = is {
                    wrap_with_margin(seekable, is_ref)
                } else {
                    seekable
                }
            }

            // Plan 010: Unified Navigation Components
            AbstractView::Accordion {
                items,
                allow_multiple: _,
                on_toggle,
                style: _,
            } => {
                use iced::widget::container;

                let mut accordion_widget = column([]);

                for (idx, item) in items.into_iter().enumerate() {
                    let header_text = if let Some(icon) = item.icon {
                        format!("{} {}", icon, item.title)
                    } else {
                        item.title.clone()
                    };

                    let header_button = if let Some(callback) = &on_toggle {
                        let callback_clone = callback.clone();
                        button(text(header_text)).on_press(callback_clone.call(idx, !item.expanded))
                    } else {
                        button(text(header_text))
                    };

                    let children_view: iced::Element<M> =
                        if item.expanded && !item.children.is_empty() {
                            let mut children_col = column([]);
                            for child in item.children {
                                children_col = children_col.push(child.into_iced());
                            }
                            children_col.into()
                        } else {
                            text("").into()
                        };

                    let section = container(column![header_button, children_view].spacing(4));
                    accordion_widget = accordion_widget.push(section);
                }

                container(accordion_widget).padding(10).into()
            }

            AbstractView::Sidebar {
                content,
                width,
                collapsible: _,
                position: _,
                style: _,
            } => {
                use iced::widget::container;
                use iced::Length;

                let sidebar_container = container(content.into_iced())
                    .width(Length::Fixed(width))
                    .height(Length::Fill);

                sidebar_container.into()
            }

            AbstractView::Tabs {
                labels,
                contents,
                selected,
                position: _,
                on_select,
                style,
                variant,
            } => {
                use iced::widget::container;

                // PLAN-641：enclosed 圆角 token——tabs 根 style 的 rounded-*
                // 类驱动激活 cell 顶部圆角（有值→Chrome 观感；无→IDE 直角
                // 观感）；底部圆角恒 0（与内容面板连通）。
                let top_radius = style.as_ref().map(|s| {
                    let r = IcedStyle::from_style(s).effective_border_radius();
                    iced::border::Radius {
                        top_left: r.top_left,
                        top_right: r.top_right,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    }
                });

                let dark = crate::ui::style::theme::dark_mode();
                let token_rgb = |t: auto_ui_compiler::design_tokens::registry::TokenName| {
                    crate::ui::style::theme::active_theme_rgb(t, dark)
                        .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                };

                match variant {
                    crate::ui::view::TabsVariant::Default => {
                        // default：按钮托盘形态（PLAN-641 零回归口径——现状
                        // 原样保留，含 [label] 选中标记；点击接 on_select）。
                        let mut tabs_widget = column([]);

                        let mut tab_buttons_row = row([]);
                        for (idx, label) in labels.iter().enumerate() {
                            let is_selected = idx == selected;
                            let label_text = if is_selected {
                                format!("[{}]", label)
                            } else {
                                label.clone()
                            };

                            let mut tab_button = button(text(label_text));
                            if let Some(cb) = &on_select {
                                tab_button = tab_button.on_press(cb.call(idx));
                            }
                            tab_buttons_row = tab_buttons_row.push(tab_button);
                        }

                        tabs_widget = tabs_widget.push(tab_buttons_row);

                        if let Some(content) = contents.get(selected) {
                            tabs_widget = tabs_widget
                                .push(container(content.clone().into_iced()).padding(20));
                        }

                        container(tabs_widget).into()
                    }
                    crate::ui::view::TabsVariant::Enclosed => {
                        // enclosed 连通形态结构契约（Zed 式，fix-tabs-merged-look
                        // 重构——交付版"激活底色=页面底色"在无框面板上不可辨，
                        // merged 无视觉锚点）：
                        // ① 每个 cell 自带 1px 边框（padding-reveal：外层
                        //    container bg=边框色，内衬 1px 露出）；
                        // ② 激活 cell 底部开口（padding-bottom 0）+ 背景=面板
                        //    背景 → 与下方内容面板无边框区直接连通；
                        // ③ 内容面板自带边框（左/右/下，顶部开口）→ 边框的
                        //    连接可见。
                        let bg = token_rgb(
                            auto_ui_compiler::design_tokens::registry::TokenName::Background,
                        );
                        let strip_bg =
                            token_rgb(auto_ui_compiler::design_tokens::registry::TokenName::Muted);
                        let cell_bg = token_rgb(
                            auto_ui_compiler::design_tokens::registry::TokenName::Secondary,
                        );
                        let active_fg = token_rgb(
                            auto_ui_compiler::design_tokens::registry::TokenName::Foreground,
                        );
                        let inactive_fg = token_rgb(
                            auto_ui_compiler::design_tokens::registry::TokenName::MutedForeground,
                        );
                        // 边框色：Border token，缺省走 iced adapter 的边框解析。
                        let frame_rgb = crate::ui::style::theme::active_theme_rgb(
                            auto_ui_compiler::design_tokens::registry::TokenName::Border,
                            dark,
                        )
                        .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                        .unwrap_or_else(|| {
                            let (r, g, b) = crate::ui::style_iced_adapter::resolve_border_rgb();
                            iced::Color::from_rgb8(r, g, b)
                        });

                        let mut strip = row([]);
                        for (idx, label) in labels.iter().enumerate() {
                            let is_active = idx == selected;
                            let label_color = if is_active { active_fg } else { inactive_fg };
                            let cell_fill = if is_active { bg } else { cell_bg };
                            let radius = if is_active {
                                top_radius.unwrap_or_default()
                            } else {
                                iced::border::Radius::default()
                            };
                            // 激活 cell 底部开口：padding-bottom 0（其余三边
                            // 1px 内衬露出边框色）。
                            let frame_pad = if is_active {
                                iced::Padding {
                                    top: 1.0,
                                    right: 1.0,
                                    bottom: 0.0,
                                    left: 1.0,
                                }
                            } else {
                                iced::Padding {
                                    top: 1.0,
                                    right: 1.0,
                                    bottom: 1.0,
                                    left: 1.0,
                                }
                            };

                            // cell 本体：外层 frame（bg=边框色，padding 内衬）
                            // > 内层 fill（bg=cell 填充）。button 仅作点击命
                            // 中面，样式透明化（覆盖 iced 默认 primary 底色）。
                            let inner = container(
                                text(label.clone())
                                    .color(label_color.unwrap_or(iced::Color::WHITE)),
                            )
                            .center_y(iced::Length::Fill)
                            .style(move |_| container::Style {
                                background: cell_fill.map(iced::Background::Color),
                                border: iced::Border {
                                    radius,
                                    ..iced::Border::default()
                                },
                                ..container::Style::default()
                            });
                            let cell = container(inner)
                                .padding(frame_pad)
                                .center_y(iced::Length::Fill)
                                .style(move |_| container::Style {
                                    background: Some(iced::Background::Color(frame_rgb)),
                                    border: iced::Border {
                                        radius,
                                        ..iced::Border::default()
                                    },
                                    ..container::Style::default()
                                });

                            let mut cell_hit = button(cell).padding(0);
                            if let Some(cb) = &on_select {
                                cell_hit = cell_hit.on_press(cb.call(idx));
                            }
                            cell_hit = cell_hit.style(move |_theme, _status| {
                                iced::widget::button::Style {
                                    background: None,
                                    text_color: label_color.unwrap_or(iced::Color::WHITE),
                                    border: iced::Border {
                                        radius,
                                        ..iced::Border::default()
                                    },
                                    ..iced::widget::button::Style::default()
                                }
                            });
                            strip = strip.push(cell_hit);
                        }

                        let mut tabs_widget = column([]);
                        tabs_widget =
                            tabs_widget.push(container(strip).height(36.0).style(move |_| {
                                container::Style {
                                    background: strip_bg.map(iced::Background::Color),
                                    ..container::Style::default()
                                }
                            }));

                        if let Some(content) = contents.get(selected) {
                            // 面板自带边框：外层 bg=边框色 + padding
                            // [0,1,1,1]（顶部开口，激活 tab 连通处无横线），
                            // 内层 bg=背景色。
                            tabs_widget = tabs_widget.push(
                                container(
                                    container(content.clone().into_iced())
                                        .padding(12)
                                        .width(iced::Length::Fill)
                                        .style(move |_| container::Style {
                                            background: bg.map(iced::Background::Color),
                                            ..container::Style::default()
                                        }),
                                )
                                .width(iced::Length::Fill)
                                .padding(iced::Padding {
                                    top: 0.0,
                                    right: 1.0,
                                    bottom: 1.0,
                                    left: 1.0,
                                })
                                .style(move |_| container::Style {
                                    background: Some(iced::Background::Color(frame_rgb)),
                                    ..container::Style::default()
                                }),
                            );
                        }

                        tabs_widget.into()
                    }
                }
            }

            AbstractView::NavigationRail {
                items,
                selected: _,
                width,
                show_labels,
                on_select: _,
                style: _,
            } => {
                use iced::widget::container;
                use iced::Length;

                let mut rail_widget = column([]);

                for item in items {
                    let item_text = if show_labels {
                        format!("{}  {}", item.icon, item.label)
                    } else {
                        item.icon.to_string()
                    };

                    let item_text_with_badge = if let Some(badge) = &item.badge {
                        format!("{} ({})", item_text, badge)
                    } else {
                        item_text
                    };

                    let nav_button = button(text(item_text_with_badge));
                    rail_widget = rail_widget.push(nav_button);
                }

                container(rail_widget)
                    .width(Length::Fixed(width))
                    .height(Length::Fill)
                    .padding(10)
                    .into()
            }

            // Plan 497: 每窗口真缩略渲染臂——T1 定案裁剪式整窗快照的
            // 消费端。命中进程级快照缓存 → image(Handle::from_rgba) 直绘
            // 降采样像素；过期 → stale-while-revalidate（旧图续帧不跌
            // fallback，request_capture 冷却去重静默重抓——hover/分区与
            // app 切换场景「TTL 到点跳 icon」断档的根治）；真 miss →
            // 标记异步抓取（宿主 update 排空 take_capture_requests，一次
            // 整窗 screenshot 服务全部请求）+ 本帧 fallback lucide 图标
            // （复用 Image 臂路径，含 native wid "N<slot>" parse 失败
            // 天然走 fallback——待澄清②）。
            AbstractView::WindowThumbnail {
                wid,
                fallback_icon,
                style,
            } => {
                let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                let eff_w = is
                    .as_ref()
                    .and_then(|is| is.width.as_ref().map(iced_length));
                let eff_h = is
                    .as_ref()
                    .and_then(|is| is.height.as_ref().map(iced_length));
                let wid_opt = wid.parse::<u64>().ok().map(crate::ui::session::Wid);
                let snap = wid_opt.and_then(crate::ui::iced::snapshot::snapshot_window_stale);
                if let Some((snap, fresh)) = snap {
                    if !fresh {
                        // SWR：过期不删——旧图本帧续绘，重抓异步补。
                        if let Some(w) = wid_opt {
                            crate::ui::iced::snapshot::request_capture(w);
                        }
                    }
                    let handle = iced::widget::image::Handle::from_rgba(snap.w, snap.h, snap.rgba);
                    let mut img = iced::widget::image(handle)
                        .filter_method(iced::widget::image::FilterMethod::Nearest);
                    if let Some(w) = eff_w {
                        img = img.width(w);
                    }
                    if let Some(h) = eff_h {
                        img = img.height(h);
                    }
                    // 缩略 chrome：消费面 style 类（border/rounded/bg）落在
                    // 包裹 container 上（与 Image 臂 container 约束同型）。
                    let border_radius = is.as_ref().and_then(|is| is.border_radius).unwrap_or(0.0);
                    let border_width = is.as_ref().and_then(|is| is.border_width).unwrap_or(0.0);
                    let border_color = is
                        .as_ref()
                        .and_then(|is| is.border_color)
                        .unwrap_or(iced::Color::TRANSPARENT);
                    let bg = is
                        .as_ref()
                        .and_then(|is| is.background_color)
                        .map(iced::Background::Color);
                    let mut cont = container(img).style(move |_t| container::Style {
                        border: iced::Border {
                            color: border_color,
                            width: border_width,
                            radius: border_radius.min(9999.0).into(),
                        },
                        background: bg,
                        ..Default::default()
                    });
                    if let Some(w) = eff_w {
                        cont = cont.width(w);
                    }
                    if let Some(h) = eff_h {
                        cont = cont.height(h);
                    }
                    cont.into()
                } else {
                    if let Some(w) = wid_opt {
                        crate::ui::iced::snapshot::request_capture(w);
                    }
                    let icon = if fallback_icon.is_empty() {
                        "app-window".to_string()
                    } else {
                        fallback_icon.clone()
                    };
                    // PLAN-002 A2 取证探针（AUTO_POPOVER_DEBUG=1；定案后移除）。
                    if std::env::var("AUTO_POPOVER_DEBUG").as_deref() == Ok("1") {
                        eprintln!(
                            "[thumb-fallback] wid={wid} eff_w={eff_w:?} eff_h={eff_h:?} style={:?}",
                            is.as_ref().map(|is| is.width.clone()),
                        );
                    }
                    AbstractView::Image {
                        src: format!("lucide:{icon}"),
                        style: style.clone(),
                    }
                    .into_iced()
                }
            }

            // PLAN-760: 宿主侧远程帧消费 leaf 渲染——canvas RemoteFramePainter
            //（draw = 注册表最新 DisplayList 等比 fit 直绘 / 相位占位；
            // update = 指针事件 → InputMsg 注册表直发）。note(url) 续租
            // （url 退场 = 租约到期断连，remote_frame_registry 生命周期）。
            // style chrome（边框/圆角/宽高）走容器包装（window_thumbnail
            // 同款纪律）。
            AbstractView::RemoteFrame { url, style } => {
                use crate::ui::iced::remote_frame_registry::RemoteFramePainter;
                let _ = crate::ui::iced::remote_frame_registry::note(&url);
                let canvas_el: iced::Element<'static, M> =
                    iced::widget::canvas(RemoteFramePainter::<M>::new(url.clone()))
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill)
                        .into();
                let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                let mut surface = iced::widget::container(canvas_el);
                if let Some(ref is) = is {
                    let background = is.background_color.map(iced::Background::Color);
                    let border_color = is.border_color.unwrap_or(iced::Color::TRANSPARENT);
                    let border_width = is.border_width.unwrap_or(0.0);
                    let border_radius = is.border_radius.unwrap_or(0.0);
                    surface = surface.style(move |_theme| iced::widget::container::Style {
                        background,
                        border: iced::Border::default()
                            .rounded(border_radius.min(9999.0))
                            .width(border_width)
                            .color(border_color),
                        ..Default::default()
                    });
                }
                surface.into()
            }
            // PLAN-012 W3：整桌面等比预览渲染臂——宿主合成（SD-02）。
            // 分层 = 壁纸基色底（#hex 直铺 / None → 主色占位）→ 该分区
            // 逐窗 snapshot tile（Contain 等比贴盒居中；SWR 续帧 + miss
            // 占位+icon+request_capture 预抓，window_thumbnail 同款纪律）。
            // tile 定位沿 title_menu 的 padding-left/top 形态（Fill 容器
            // + 内层 Fixed 内容）。盒尺寸取 style Fixed 宽高（缺省 176×64
            // ——shell 卡片消费形态），数据缺席 = 纯壁纸底。
            AbstractView::WorkspacePreview {
                ws,
                fallback_icon,
                style,
            } => {
                use crate::ui::iced::workspace_preview as wp;
                let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                let box_w = is
                    .as_ref()
                    .and_then(|is| is.width.clone())
                    .and_then(|w| match w {
                        IcedSize::Fixed(f) => Some(f),
                        _ => None,
                    })
                    .unwrap_or(176.0);
                let box_h = is
                    .as_ref()
                    .and_then(|is| is.height.clone())
                    .and_then(|h| match h {
                        IcedSize::Fixed(f) => Some(f),
                        _ => None,
                    })
                    .unwrap_or(64.0);
                let data = wp::current();
                let border_radius = is.as_ref().and_then(|is| is.border_radius).unwrap_or(0.0);
                let mut layers: Vec<iced::Element<'static, M>> = Vec::new();
                // ① 壁纸底层（2026-09-22 升级：非 `#` spec——图片路径/
                // builtin:——直绘真壁纸（desktop_wallpaper_element 同源
                // handle 缓存，Cover 铺满）；`#hex` 仍走基色底，缺席 →
                // 主色占位。原纯色臂把图片壁纸画成主色块，pager 卡观感
                // 为"紫色板"而非桌面）。
                let wp_src = data
                    .as_ref()
                    .map(|d| d.wallpaper_src.clone())
                    .unwrap_or_default();
                if !wp_src.is_empty() && !wp_src.starts_with('#') {
                    layers.push(desktop_wallpaper_element::<M>(&wp_src));
                } else {
                    let bg = data
                        .as_ref()
                        .and_then(|d| d.wallpaper)
                        .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                        .or_else(|| {
                            crate::ui::style_iced_adapter::resolve_semantic_rgb(
                                &crate::ui::style::Color::Primary,
                            )
                            .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                        })
                        .unwrap_or(iced::Color::from_rgb8(0x33, 0x33, 0x44));
                    let bg_style = move |_t: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(bg)),
                        ..Default::default()
                    };
                    layers.push(
                        iced::widget::container(iced::widget::Space::new())
                            .width(iced::Length::Fill)
                            .height(iced::Length::Fill)
                            .style(bg_style)
                            .into(),
                    );
                }
                if let Some(d) = data.as_ref() {
                    let fallback_icon = if fallback_icon.is_empty() {
                        "app-window".to_string()
                    } else {
                        fallback_icon.clone()
                    };
                    if let Some(tiles) = d.workspaces.get(ws.as_str()) {
                        for t in tiles {
                            let (tx, ty, tw, th) = wp::tile_rect(t, d.usable, box_w, box_h);
                            if tw <= 0.5 || th <= 0.5 {
                                continue;
                            }
                            let wid_opt = Some(t.wid).map(crate::ui::session::Wid);
                            let snap =
                                wid_opt.and_then(crate::ui::iced::snapshot::snapshot_window_stale);
                            let inner: iced::Element<'static, M> = if let Some((snap, fresh)) = snap
                            {
                                if !fresh {
                                    if let Some(w) = wid_opt {
                                        crate::ui::iced::snapshot::request_capture(w);
                                    }
                                }
                                iced::widget::image(iced::widget::image::Handle::from_rgba(
                                    snap.w, snap.h, snap.rgba,
                                ))
                                .filter_method(iced::widget::image::FilterMethod::Nearest)
                                .width(iced::Length::Fixed(tw))
                                .height(iced::Length::Fixed(th))
                                .into()
                            } else {
                                if let Some(w) = wid_opt {
                                    crate::ui::iced::snapshot::request_capture(w);
                                }
                                // miss = 占位块 + fallback icon 居中。
                                let ph = crate::ui::style_iced_adapter::resolve_semantic_rgb(
                                    &crate::ui::style::Color::Surface,
                                )
                                .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                                .unwrap_or(iced::Color::from_rgb8(0x44, 0x44, 0x55));
                                let icon_src = format!("lucide:{fallback_icon}");
                                let icon_style =
                                    crate::ui::style::Style::parse("w-4 h-4 text-muted-foreground")
                                        .ok();
                                let icon_el = AbstractView::<M>::Image {
                                    src: icon_src,
                                    style: icon_style,
                                }
                                .into_iced();
                                iced::widget::container(icon_el)
                                    .width(iced::Length::Fixed(tw))
                                    .height(iced::Length::Fixed(th))
                                    .center_x(iced::Length::Fixed(tw))
                                    .center_y(iced::Length::Fixed(th))
                                    .style(move |_t| iced::widget::container::Style {
                                        background: Some(iced::Background::Color(ph)),
                                        ..Default::default()
                                    })
                                    .into()
                            };
                            let (plx, ply) = (tx.max(0.0), ty.max(0.0));
                            layers.push(
                                iced::widget::container(inner)
                                    .width(iced::Length::Fill)
                                    .height(iced::Length::Fill)
                                    .padding(iced::Padding {
                                        top: ply,
                                        bottom: 0.0,
                                        left: plx,
                                        right: 0.0,
                                    })
                                    .into(),
                            );
                        }
                    }
                }
                // 2026-09-22 修：stack 显式 Fixed(box)——原 Shrink 下
                // Fill 子层（壁纸底/窗 tile 层）在 Shrink 测量语境解析
                // 为 0×0，整预览塌缩不可见（pager 空卡实录）。
                let root = iced::widget::stack(layers)
                    .width(iced::Length::Fixed(box_w))
                    .height(iced::Length::Fixed(box_h));
                let mut cont = iced::widget::container(root)
                    .width(iced::Length::Shrink)
                    .height(iced::Length::Shrink);
                if let Some(w) = is
                    .as_ref()
                    .and_then(|is| is.width.as_ref().map(iced_length))
                {
                    cont = cont.width(w);
                }
                if let Some(h) = is
                    .as_ref()
                    .and_then(|is| is.height.as_ref().map(iced_length))
                {
                    cont = cont.height(h);
                }
                cont.style(move |_t| iced::widget::container::Style {
                    border: iced::Border {
                        radius: border_radius.min(9999.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
            }

            AbstractView::Image { src, style } => {
                // PLAN-018：iconfile:<stem> = 双主题位图（回退链首位）。
                // Plan 515 D1：hicon:<slot> = native 真图标 raster
                //（window_thumbnail 的 fallback_icon / image 直挂两消费面）。
                // PLAN-018-FU7：显式 Fixed 宽/高 → 按显示档装载；无显式
                // 尺寸档 → 512 原生档（回退链同源）。
                let style_for_px = style.as_ref().map(IcedStyle::from_style);
                let display_px = match (
                    style_for_px.as_ref().and_then(|s| s.width.as_ref()),
                    style_for_px.as_ref().and_then(|s| s.height.as_ref()),
                ) {
                    (Some(IcedSize::Fixed(w)), _) => Some(*w),
                    (_, Some(IcedSize::Fixed(h))) => Some(*h),
                    _ => None,
                };
                let raster_icon: Option<iced::widget::image::Handle> = {
                    let dark = crate::ui::style::theme::dark_mode();
                    let iconfile = display_px
                        .and_then(|px| crate::ui::iced::icon_file::load_sized(&src, dark, px))
                        .or_else(|| crate::ui::iced::icon_file::load(&src, dark));
                    if let Some(handle) = iconfile {
                        Some(handle)
                    } else {
                        crate::ui::iced::native_icon::parse_field(&src).map(|icon| {
                            iced::widget::image::Handle::from_rgba(icon.w, icon.h, icon.rgba)
                        })
                    }
                };
                if let Some(handle) = raster_icon {
                    let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                    let mut img = iced::widget::image(handle);
                    if let Some(ref is) = is {
                        if let Some(ref w) = is.width {
                            img = img.width(iced_length(w));
                        }
                        if let Some(ref h) = is.height {
                            img = img.height(iced_length(h));
                        }
                    }
                    return img.into();
                }
                // Plan 408: lucide: icon prefix → render bundled SVG glyph.
                if src.starts_with("lucide:") {
                    let icon_name = &src[7..];
                    let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                    let w = is
                        .as_ref()
                        .and_then(|is| is.width.as_ref().map(iced_length));
                    let h = is
                        .as_ref()
                        .and_then(|is| is.height.as_ref().map(iced_length));
                    // Plan 518 G4②:大尺寸细线——固定宽或高 ≥48px 的独立图标
                    // 用 stroke-width 1.5（stella 线性观感）;PUA 按钮内嵌路径
                    // 尺寸随字号（<48px）保持默认 2。
                    let large = [&w, &h]
                        .into_iter()
                        .flatten()
                        .any(|len| matches!(len, iced::Length::Fixed(v) if *v >= 48.0));
                    let sw = if large { 1.5 } else { 2.0 };
                    // PLAN-621: icon state=on 的激活加重——builder 已按
                    // 「基档 + 0.5」算好绝对值放进 StrokeWidth 类，此处只读
                    // 不加工；无该类时保持既有基档（零扰动）。
                    let sw = is.as_ref().and_then(|s| s.stroke_width).unwrap_or(sw);
                    if let Some(svg_str) = lucide_svg_doc_with(icon_name, sw) {
                        // Plan 409 §10 组 C → 2026-08-21 方案 A(ash-gui hover):
                        // 画时着色 —— svg::Style.color 由 iced 光栅化器把不透明
                        // 像素的 RGB 整体替换(按 (handle,size,color) 缓存,引擎
                        // 级 currentColor),取代旧的"颜色烘焙进 SVG 字节"(view
                        // 构建时定死,悬停无法变化)。基础色:自身 text-*(按钮内
                        // 经 inherit_text_color 已继承按钮色),回落 OnBackground
                        // (resvg 会把 currentColor 画成黑色)。悬停色:图标自己的
                        // hover:text-* —— svg::Status::Hovered 命中区即图标包围
                        // 盒,与 CSS 元素级 :hover 语义一致(hover:text 写在 icon
                        // 上,浏览器里也只在图标盒内触发,而非整个按钮)。
                        // 注:wgpu tint 只替换 RGB、忽略 Color.a —— 颜色自带透明
                        // 度(如 text-red-400/80)经 .opacity() 补偿;悬停色 alpha
                        // 忽略(opacity 构建期恒定)。
                        let base_color = is
                            .as_ref()
                            .and_then(|is| is.text_color)
                            .or_else(|| {
                                crate::ui::style_iced_adapter::resolve_semantic_rgb(
                                    &crate::ui::style::Color::OnBackground,
                                )
                                .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                            })
                            .unwrap_or(iced::Color::BLACK);
                        let hover_color = style.as_ref().and_then(|s| {
                            s.hover_classes.iter().find_map(|c| match c {
                                StyleClass::TextColor(color) => {
                                    Some(crate::ui::style_iced_adapter::convert_color(color))
                                }
                                _ => None,
                            })
                        });
                        let handle = get_or_create_svg_handle(&src, svg_str.as_bytes().to_vec());
                        let mut svg_widget = iced::widget::svg(handle).style(move |_, status| {
                            let color = match status {
                                iced::widget::svg::Status::Hovered => {
                                    hover_color.unwrap_or(base_color)
                                }
                                _ => base_color,
                            };
                            iced::widget::svg::Style { color: Some(color) }
                        });
                        if base_color.a < 1.0 {
                            svg_widget = svg_widget.opacity(base_color.a);
                        }
                        svg_widget = svg_widget.width(w.unwrap_or(iced::Length::Fixed(16.0)));
                        svg_widget = svg_widget.height(h.unwrap_or(iced::Length::Fixed(16.0)));
                        // PLAN-054 T4 (A11): margin 系(ml-auto 贴行右端)在图标
                        // 出口消费——此前 container 直接返回,ml-auto 静默丢失。
                        // PLAN-619 R3: 判定走 effective_margin（单侧 > 轴 > 统一）。
                        if let Some(ref is) = is {
                            if is.has_margin() {
                                return wrap_with_margin(container(svg_widget).into(), is);
                            }
                        }
                        return container(svg_widget).into();
                    }
                    // Unknown icon name: render empty placeholder
                    return container(iced::widget::text("")).into();
                }
                // Plan 442 A4: 内联 SVG 文档(view builder 序列化的 svg 元素
                // 子树)→ 复用缓存 svg::Handle 渲染。单色文档(以 currentColor
                // 描述填色/描边,如 musk 038 52-icon 数据层)走 lucide 同款
                // 画时着色(resvg 把 currentColor 画成黑,tint 整体替换不透明
                // 像素 RGB);多彩文档保持原色(tint 会毁掉配色)。
                if let Some(doc) = src.strip_prefix("svgdoc:") {
                    let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                    let w = is
                        .as_ref()
                        .and_then(|is| is.width.as_ref().map(iced_length));
                    let h = is
                        .as_ref()
                        .and_then(|is| is.height.as_ref().map(iced_length));
                    // 混合色文档(静态色 + currentColor,如 donut 的灰底环 +
                    // accent 主弧)不能用整体 tint:tint 栅格化后全像素替换,
                    // 静态色环也被涂成主色(概要页 Memory donut 全黑的根因)。
                    // 改为构建期把 currentColor 字面量内联替换成实际色值——
                    // 文档不再含 currentColor,多彩路径保持各像素原色。
                    let (doc, tint) = if doc.contains("currentColor") {
                        let base_color = is
                            .as_ref()
                            .and_then(|is| is.text_color)
                            .or_else(|| {
                                crate::ui::style_iced_adapter::resolve_semantic_rgb(
                                    &crate::ui::style::Color::OnBackground,
                                )
                                .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                            })
                            .unwrap_or(iced::Color::BLACK);
                        let css = format!(
                            "#{:02x}{:02x}{:02x}",
                            (base_color.r * 255.0).round() as u8,
                            (base_color.g * 255.0).round() as u8,
                            (base_color.b * 255.0).round() as u8,
                        );
                        (doc.replace("currentColor", &css), base_color.a < 1.0)
                    } else {
                        (doc.to_string(), false)
                    };
                    let handle = get_or_create_svg_handle(&src, doc.as_bytes().to_vec());
                    let mut svg_widget = svg(handle);
                    if tint {
                        let base_color = is
                            .as_ref()
                            .and_then(|is| is.text_color)
                            .unwrap_or(iced::Color::BLACK);
                        svg_widget = svg_widget.style(move |_, _| iced::widget::svg::Style {
                            color: Some(base_color),
                        });
                        if base_color.a < 1.0 {
                            svg_widget = svg_widget.opacity(base_color.a);
                        }
                    }
                    if let Some(w) = w {
                        svg_widget = svg_widget.width(w);
                    }
                    if let Some(h) = h {
                        svg_widget = svg_widget.height(h);
                    }
                    // 尺寸必须同时约束外层 container:iced 0.14 Container::new
                    // 取 size_hint().fluid(),Svg 的 size_hint 默认 Fill → 无
                    // 约束时 container 撑满行内剩余宽(图标 Contain 居中画在
                    // 半宽处,同行文字被推到最远端——概要页 Uptime 行错乱形态)。
                    let mut svg_cont = container(svg_widget);
                    if let Some(w) = w {
                        svg_cont = svg_cont.width(w);
                    }
                    if let Some(h) = h {
                        svg_cont = svg_cont.height(h);
                    }
                    return svg_cont.into();
                }
                let bytes = load_image_bytes(&src);
                let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                let eff_w = is.as_ref().and_then(|is| is.width.map(|w| iced_length(&w)));
                let eff_h = is
                    .as_ref()
                    .and_then(|is| is.height.map(|h| iced_length(&h)));
                let px_w = is.as_ref().and_then(|is| is.width).and_then(|w| match w {
                    IcedSize::Fixed(f) => Some(f),
                    _ => None,
                });
                let px_h = is.as_ref().and_then(|is| is.height).and_then(|h| match h {
                    IcedSize::Fixed(f) => Some(f),
                    _ => None,
                });
                let border_radius = is.as_ref().and_then(|is| is.border_radius).unwrap_or(0.0);
                let border_width = is.as_ref().and_then(|is| is.border_width).unwrap_or(0.0);
                let border_color = is
                    .as_ref()
                    .and_then(|is| is.border_color)
                    .unwrap_or(iced::Color::TRANSPARENT);
                let shadow = is.as_ref().map_or(false, |is| is.shadow);
                let bg = is
                    .as_ref()
                    .and_then(|is| is.background_color)
                    .map(iced::Background::Color);

                let eff_img_w = match (px_w, border_width) {
                    (Some(w), bw) if bw > 0.0 => Some((w - 2.0 * bw).max(0.0)),
                    (Some(w), _) => Some(w),
                    _ => None,
                };
                let eff_img_h = match (px_h, border_width) {
                    (Some(h), bw) if bw > 0.0 => Some((h - 2.0 * bw).max(0.0)),
                    (Some(h), _) => Some(h),
                    _ => None,
                };
                let img_border_radius = if border_radius >= 9999.0 {
                    9999.0
                } else {
                    (border_radius - border_width).max(0.0)
                };

                if let Some(data) = bytes {
                    let data = if border_radius > 100.0
                        && (src.ends_with(".svg") || src.contains("/svg"))
                    {
                        String::from_utf8(data)
                            .map(|mut s| {
                                s = s.replace("rx=\"0\" ry=\"0\"", "rx=\"140\" ry=\"140\"");
                                s.into_bytes()
                            })
                            .unwrap_or_else(|e| e.into_bytes())
                    } else {
                        data
                    };
                    // Use cached handle to avoid flickering — same URL reuses the same Handle
                    let inner: iced::Element<'static, M> = if src.ends_with(".svg")
                        || src.contains("/svg")
                        || src.contains("image/svg+xml")
                    {
                        let handle = get_or_create_svg_handle(&src, data);
                        let mut svg_widget = svg(handle);
                        if let Some(w) = eff_img_w {
                            svg_widget = svg_widget.width(iced::Length::Fixed(w));
                        } else if let Some(w) = eff_w {
                            svg_widget = svg_widget.width(w);
                        }
                        if let Some(h) = eff_img_h {
                            svg_widget = svg_widget.height(iced::Length::Fixed(h));
                        } else if let Some(h) = eff_h {
                            svg_widget = svg_widget.height(h);
                        }
                        svg_widget.into()
                    } else {
                        let handle = get_or_create_image_handle(
                            &src,
                            data,
                            img_border_radius,
                            eff_img_w,
                            eff_img_h,
                        );
                        let mut img_widget = iced::widget::image(handle);
                        // Plan 527 T4: object-fit → ContentFit(缺失时 iced 默认 Contain)
                        if let Some(fit) = is.as_ref().and_then(|is| is.object_fit) {
                            img_widget = img_widget.content_fit(match fit {
                                crate::ui::style::ObjectFit::Contain => iced::ContentFit::Contain,
                                crate::ui::style::ObjectFit::Cover => iced::ContentFit::Cover,
                                crate::ui::style::ObjectFit::Fill => iced::ContentFit::Fill,
                                crate::ui::style::ObjectFit::None => iced::ContentFit::None,
                                crate::ui::style::ObjectFit::ScaleDown => {
                                    iced::ContentFit::ScaleDown
                                }
                            });
                        }
                        if let Some(w) = eff_img_w {
                            img_widget = img_widget.width(iced::Length::Fixed(w));
                        } else if let Some(w) = eff_w {
                            img_widget = img_widget.width(w);
                        }
                        if let Some(h) = eff_img_h {
                            img_widget = img_widget.height(iced::Length::Fixed(h));
                        } else if let Some(h) = eff_h {
                            img_widget = img_widget.height(h);
                        }
                        img_widget.into()
                    };

                    let mut cont = container(inner)
                        .center_x(eff_w.unwrap_or(iced::Length::Shrink))
                        .center_y(eff_h.unwrap_or(iced::Length::Shrink))
                        .clip(true);
                    if let Some(w) = eff_w {
                        cont = cont.width(w);
                    }
                    if let Some(h) = eff_h {
                        cont = cont.height(h);
                    }
                    if border_radius > 0.0 || border_width > 0.0 || shadow || bg.is_some() {
                        let br = if border_radius >= 9999.0 {
                            match (px_w, px_h) {
                                (Some(w), Some(h)) => w.min(h) / 2.0,
                                (Some(w), None) => w / 2.0,
                                (None, Some(h)) => h / 2.0,
                                _ => 9999.0,
                            }
                        } else {
                            border_radius
                        };
                        let bw = border_width;
                        let bc = border_color;
                        cont = cont.style(move |_| container::Style {
                            background: bg,
                            border: iced::Border::default().rounded(br).width(bw).color(bc),
                            shadow: if shadow {
                                iced::Shadow {
                                    offset: iced::Vector::new(0.0, 2.0),
                                    blur_radius: 8.0,
                                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                                }
                            } else {
                                iced::Shadow::default()
                            },
                            ..Default::default()
                        });
                    }
                    cont.into()
                } else {
                    // Fallback: show a colored placeholder with initials
                    let initials = extract_initials(&src);
                    let child = text(initials).size(14).color(iced::Color::WHITE);
                    let mut cont = container(child)
                        .center_x(iced::Length::Fill)
                        .center_y(iced::Length::Fill);
                    let placeholder_bg = is
                        .as_ref()
                        .and_then(|is| is.background_color)
                        .unwrap_or_else(|| iced::Color::from_rgb(0.24, 0.47, 0.85));
                    let br = border_radius.max(9999.0);
                    let bw = border_width;
                    let bc = border_color;
                    if let Some(w) = eff_w {
                        cont = cont.width(w);
                    }
                    if let Some(h) = eff_h {
                        cont = cont.height(h);
                    }
                    cont = cont.style(move |_| container::Style {
                        background: Some(iced::Background::Color(placeholder_bg)),
                        border: iced::Border::default().rounded(br).width(bw).color(bc),
                        ..Default::default()
                    });
                    cont.into()
                }
            }

            AbstractView::ImageSurface {
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
                on_error,
                on_loaded,
                on_wheel,
                on_pan,
                on_double_click,
                style,
            } => {
                // Callback messages are retained in the View contract and
                // consumed by the input adapter (Task 25); this paint pass
                // must not synthesize duplicate notifications.
                let _ = (on_error, on_loaded, on_wheel, on_pan, on_double_click);
                render_image_surface(
                    src, alt, width, height, quality, fit, zoom, offset_x, offset_y, rotation,
                    filter, style,
                )
            }

            // PLAN-617 T-19: `video` —— 原生播放面。
            //
            // 走自定义 shader widget（自持持久纹理、每帧原地更新），**不经过**
            // iced 的 Handle/atlas 通道——那条路正是 `renderer.rs:2889` 记录的
            // 闪烁机理（见 Design 30 §4.5）。上行事件由事件泵 wrapper
            //（`video_uplink`，PLAN-712 T-04）按帧取走并合成宿主消息。
            AbstractView::Video {
                src,
                paused,
                position,
                volume,
                muted,
                rate,
                label,
                on_time_update,
                on_loaded_metadata,
                on_play_state,
                on_ended,
                on_media_error,
                media_key,
                style,
            } => {
                // PLAN-721 T-19：build-time trace——T-19 定案最后一轴。同一
                // binding 的两次相邻构建烤出相反 paused（TogglePlay 写 false
                // 后的 OnTime 重建烤回 true）即「读侧世界分裂/陈旧 memo 回放」
                // 实锤（本作用域无 session state，对照读由 per-app snapshot
                // 面承担）。
                if crate::ui::sched_diag::enabled() {
                    eprintln!(
                        "[SCHED-DIAG] video_build t={}ms paused(baked)={} src..{}",
                        crate::ui::dynamic::sched_diag_t0().elapsed().as_millis(),
                        paused,
                        src.chars().rev().take(24).collect::<String>(),
                    );
                }
                render_video(
                    src,
                    paused,
                    position,
                    volume,
                    muted,
                    rate,
                    label,
                    on_time_update,
                    on_loaded_metadata,
                    on_play_state,
                    on_ended,
                    on_media_error,
                    media_key,
                    style,
                )
            }
        }
    }
}

/// 当前视图构建的媒体会话域（`dynamic_view_impl` 进入时置 app_id，离开
/// 恢复）——`video` 的 `key:` 属性与之合成引擎稳定键（[`crate::ui::mpv::widget::stable_runtime_key`]），
/// 同会话的 mini/main 视图面与面板反复开关都落到同一 mpv 运行时。
thread_local! {
    static MEDIA_VIEW_SCOPE: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// 当前媒体会话域读值（mpv 稳定键合成的模块外只读口；缺席 = None）。
/// PLAN-053 基线修复：master `cfe325d4` 的 media.rs 在 mpv-widget 下裸引
/// 本模块私有 static（E0425）——以访问器形态修复，不改原 thread_local 可见性。
pub(crate) fn current_media_view_scope() -> Option<u64> {
    MEDIA_VIEW_SCOPE.with(|c| c.get())
}

/// 会话域内执行一次视图构建（作用域守卫：嵌套/异常路径都恢复前值）。
pub(crate) fn with_media_view_scope<R>(ns: u64, f: impl FnOnce() -> R) -> R {
    MEDIA_VIEW_SCOPE.with(|c| {
        let prev = c.replace(Some(ns));
        let out = f();
        c.set(prev);
        out
    })
}


/// View function for one App's window, used by `view_desktop_fn` (iced::daemon
/// per-window view). This is a standalone function (not a closure) so that Rust
/// can correctly infer the higher-ranked lifetime bound `for<'a> ViewFn<'a, ...>`.
///
/// Plan 459：拆借视图由调用方按窗口构造（`split_ref_at`）；`sync_mcp` 仅在
/// primary App 的视图为 true —— MCP 快照永远指向 primary（T8 单 App 语义）。
pub(super) fn dynamic_view(
    state: crate::ui::session::SessionViewRef<'_>,
    sync_mcp: bool,
) -> iced::Element<'_, IcedMessage> {
    // PLAN-033：投影期安装本组件主题 scope——语义色转换（style_iced_adapter
    // convert_color）与 view 构建期（view_with_debug_gated guard）同读一份
    // 快照。缺此安装时投影落 legacy 全局槽，design 绑定族不达像素（V-02
    // 双端一致性的根因）。
    let _theme_guard = state.component.theme_scope_guard();
    dynamic_view_impl(state, sync_mcp, false)
}

/// PLAN-002 N5：vwin 托管 app 客户区渲染——根节点默认底角圆角（与窗框
/// `WIN_RADIUS` 对齐；见 [`round_bottom_root_default`]）。
pub(super) fn dynamic_view_vwin(
    state: crate::ui::session::SessionViewRef<'_>,
    sync_mcp: bool,
) -> iced::Element<'_, IcedMessage> {
    // PLAN-033：同 dynamic_view——vwin 托管面投影亦在组件 scope 内转换。
    let _theme_guard = state.component.theme_scope_guard();
    dynamic_view_impl(state, sync_mcp, true)
}

/// PLAN-002 N5：vwin 托管 app 根节点默认底角圆角（纯函数，测试靶）——
/// 根节点自带不透明底色时其方角会探出圆角窗框。根 style 已含任何
/// radius 类（全角/方向性）= 应用作者显式声明，不干预；根无 style
/// （透明）不探出，不处理。非容器根（text/scrollable 等）同样跳过。
pub(super) fn round_bottom_root_default<M: Clone + std::fmt::Debug>(view: &mut crate::ui::view::View<M>) {
    use crate::ui::style::{RoundedSize, StyleClass};
    let style = match view {
        crate::ui::view::View::Column { style, .. }
        | crate::ui::view::View::Row { style, .. }
        | crate::ui::view::View::Container { style, .. }
        | crate::ui::view::View::Grid { style, .. } => style,
        _ => return,
    };
    let Some(s) = style else { return };
    let has_radius = s.classes.iter().any(|c| {
        matches!(
            c,
            StyleClass::Rounded
                | StyleClass::RoundedSm
                | StyleClass::RoundedMd
                | StyleClass::RoundedLg
                | StyleClass::RoundedXl
                | StyleClass::Rounded2Xl
                | StyleClass::Rounded3Xl
                | StyleClass::RoundedFull
                | StyleClass::RoundedNone
                | StyleClass::RoundedT(_)
                | StyleClass::RoundedB(_)
                | StyleClass::RoundedL(_)
                | StyleClass::RoundedR(_)
                | StyleClass::RoundedTL(_)
                | StyleClass::RoundedTR(_)
                | StyleClass::RoundedBL(_)
                | StyleClass::RoundedBR(_)
        )
    });
    if !has_radius {
        s.classes.push(StyleClass::RoundedB(Some(RoundedSize::Xxl)));
        // PLAN-024 复测裁定(2026-09-20):顶部保持方角——顶部圆角在
        // tab 条/工具栏区呈缺角观感(用户明确不要);顶部窗框圆角由
        // 标题条 chrome 层视觉承载。
    }
}

fn dynamic_view_impl(
    state: crate::ui::session::SessionViewRef<'_>,
    sync_mcp: bool,
    round_bottom: bool,
) -> iced::Element<'_, IcedMessage> {
    // 媒体会话域（2026-10-10 小组件独立播放配套）：本次视图构建内 `video`
    // `key:` 与 app_id 合成引擎稳定键——mini face 与主窗两条构建路同域，
    // 共享同一 mpv 运行时（升格/降格无缝交接）。
    with_media_view_scope(state.app_id.0, || {
        dynamic_view_impl_inner(state, sync_mcp, round_bottom)
    })
}

fn dynamic_view_impl_inner(
    state: crate::ui::session::SessionViewRef<'_>,
    sync_mcp: bool,
    round_bottom: bool,
) -> iced::Element<'_, IcedMessage> {
    // PLAN-024：face（命名视图活渲染面）渲染分支——dashboard 面板的格位
    // 直显。与主视图管线完全分离：不走 app 级 dirty/cached 缓存（与该 app
    // 主窗的渲染缓存互不踩踏）、不做 MCP 同步、不碰 DevTools 捕获——每帧
    // 经 view_named 新鲜构建（与 app 主窗共享同一 component/VM 桥，状态
    // 一致即视觉一致 = 活渲染面本体）。面板关闭即不推 face 层（装配层
    // 门控），重建成本只在面板打开期发生（§10.4 度量点留 T-08）。
    if let Some(face_name) = state.view_name {
        let mut path: Vec<usize> = Vec::new();
        return match state.component.view_named(face_name) {
            Some(view) => {
                let converted = convert_view_messages(view);
                render_dynamic_view(converted, None, &mut path)
            }
            None => iced::widget::text(format!("[dashboard] face `{face_name}` missing"))
                .size(14)
                .into(),
        };
    }

    // Plan 309 续篇 II: set the single INSPECT_CAPTURE flag read by
    // `into_iced` + `wrap_debug` during this build. Plain click/hover =
    // inspect over all widgets; Alt held = native. T4c 裁定 M1：修饰键直读
    // desktop.current_modifiers（唯一源，update 臂经消息载荷写回）。
    let alt_held = state.desktop.current_modifiers.borrow().alt();
    let capture =
        state.app.devtools.debug_mode && *state.app.devtools.inspect_mode.borrow() && !alt_held;
    INSPECT_CAPTURE.with(|c| c.set(capture));

    // PLAN-530 步骤4 泄漏定位门控：P530_NOMCP=1 时整段跳过 per-frame MCP
    // 同步 + capture 路径（view_with_debug_gated/vtree/id_map），用于
    // A/B 判别每帧保留源。
    let p530_nomcp = std::env::var("P530_NOMCP").as_deref() == Ok("1");

    // PLAN-721 T-DOCS-1: desktop 内嵌 app 轻量快照自发布（T8 单 App 语义
    // 之外的 per-app 面）——primary 走下方完整同步块；内嵌 app 在自身脏帧
    // 发布紧凑面（state 全量 + 源模板 AURA 文本）到 app_surfaces，供
    // autoui_snapshot 的 `app` 选择器消费。双门控防每帧税：仅 MCP 最近
    // 活跃（Plan 314 心跳纪律）+ 自身视图真脏。
    if !sync_mcp && !p530_nomcp {
        if let Some(ref mcp_handle) = state.desktop.mcp_shared {
            let mcp_active = mcp_handle.lock().unwrap().mcp_active_recently(5);
            if mcp_active && *state.app.view_dirty.borrow() {
                // 键身份：源路径上溯（跳过 front/src/pages 段）取 app 目录
                // 名——根 widget 名全 app 撞名（"App"），app 目录名才是
                // 稳定唯一面（registry id 同源目录名）。
                let app_label = {
                    let from_source = state.component.source_path().and_then(|p| {
                        let mut cur: &std::path::Path = p.as_ref();
                        loop {
                            let parent = cur.parent()?;
                            let name = parent
                                .file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            if name.is_empty()
                                || name == "front"
                                || name == "src"
                                || name == "pages"
                            {
                                cur = parent;
                            } else {
                                return Some(name);
                            }
                        }
                    });
                    from_source.unwrap_or_else(|| state.component.widget_name().to_string())
                };
                let state_vals = state.component.read_all_state_materialized();
                let mut pairs: Vec<(&String, &auto_val::Value)> = state_vals.iter().collect();
                pairs.sort_by(|a, b| a.0.cmp(b.0));
                let mut state_text = String::new();
                for (k, v) in pairs {
                    state_text.push_str(&format!("  {} = {}\n", k, v));
                }
                // PLAN-721 T-4：子命名空间段——child_state_map 逐实例自有
                // 字段面（T-19 作用域分裂定案：写入是否落错对象）。
                for (cname, cfields) in state.component.bridge().read_all_child_states() {
                    state_text.push_str(&format!("== child [{}] ==\n", cname));
                    let mut cpairs: Vec<(&String, &auto_val::Value)> = cfields.iter().collect();
                    cpairs.sort_by(|a, b| a.0.cmp(b.0));
                    for (k, v) in cpairs {
                        state_text.push_str(&format!("  {} = {}\n", k, v));
                    }
                }
                // PLAN-721 T-4：root 字段可达实例对象段（store Obj 全字段
                // ——VM 内方法调用 world vs root 扁平镜像分裂对质）。
                for (fname, fid, ffields) in state.component.bridge().read_root_field_objects() {
                    state_text.push_str(&format!("== obj {}@{} ==\n", fname, fid));
                    let mut fpairs: Vec<(&String, &auto_val::Value)> = ffields.iter().collect();
                    fpairs.sort_by(|a, b| a.0.cmp(b.0));
                    for (k, v) in fpairs {
                        state_text.push_str(&format!("  {} = {}\n", k, v));
                    }
                }
                let ws_now = *state.window_size.borrow();
                let mut builder =
                    crate::ui::aura_snapshot_builder::AuraSnapshotBuilder::new(&state_vals)
                        .with_status(false);
                if ws_now.width > 0.0 && ws_now.height > 0.0 {
                    builder = builder.with_viewport(ws_now.width, ws_now.height);
                }
                let template_text = builder.build(
                    state.component.widget_name(),
                    state.component.view_template(),
                );
                let key = format!("{}:{}", state.app_id.0, app_label);
                let surface = crate::ui::mcp_server::AppSurface {
                    widget_name: app_label,
                    state_text,
                    template_text,
                    updated_ms: crate::ui::dynamic::sched_diag_t0().elapsed().as_millis() as u64,
                };
                mcp_handle.lock().unwrap().set_app_surface(key, surface);
            }
        }
    }

    // Plan 370 D-GAP-2/D-GAP-5: sync dark mode + accent to iced_adapter thread_locals
    // so semantic colors (bg-primary, text-foreground, etc.) resolve correctly.
    // PLAN-050：值变化时标 view_dirty——fence 家族 chrome 等在 view 求值期
    // 解析类串（family_of 按 dark_mode 选 static），Element 缓存若不随主题
    // 翻转重建，预览臂会卡在首帧的暗色档（编辑臂 palette 在 draw 期取用
    // 不受影响，两臂曾因此分叉）。
    // PLAN-725 T-01：块上移至 MCP 同步块之前——view_dirty 写点必须先于
    // 脏帧单建读点（原位置在同步块后，主重建读在其后自然接住；单建
    // 提升后由本块先落位保序）。
    {
        static LAST_SYNCED_DARK: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(true);
        if let Ok(dark_val) = state.component.read_state("dark_mode") {
            let is_dark = match dark_val {
                auto_val::Value::Bool(b) => b,
                _ => false,
            };
            if LAST_SYNCED_DARK.swap(is_dark, std::sync::atomic::Ordering::SeqCst) != is_dark {
                *state.app.view_dirty.borrow_mut() = true;
                // PLAN-051 T10（DEBTS 050 处置）：值变化臂顺带重着色既有
                // fence buffer——编辑壳 hljs 主题在 buffer 构建期选定，不随
                // 全局翻转自换（wontfix 前提「运行时切换器」由 settings 面
                // 落地成立）。
                // PLAN-551：调用随 autodown_editor 模块同门控 all(autodown,
                // code-editor)——ui-iced 单开时跳过。
                #[cfg(feature = "autodown")]
                crate::ui::autodown_editor::retheme_all_fence_buffers();
            }
            crate::ui::style_iced_adapter::set_dark_mode(is_dark);
        }
    }
    if let Ok(accent_val) = state.component.read_state("accent_color") {
        let name = match accent_val {
            auto_val::Value::Str(s) => s.as_str().to_string(),
            _ => "indigo".to_string(),
        };
        crate::ui::style_iced_adapter::set_accent_name(&name);
    }

    // ── PLAN-725 T-01：脏帧单建（单帧单建本体）─────────────────────────
    // view_with_debug_gated 此前每脏帧跑两遍（MCP 同步块第一遍 + 主重建
    // 第二遍——T-00 实测 s3a/s3b 双计）。现将主重建的模板建**提升**到同步
    // 块之前，产物（view/id_map/probe）存 frame_build；同步块与主重建段
    // 共用（同步块消费克隆，主重建段消费原件）。非脏帧零成本（if dirty
    // 直落）。capture_debug 一并提前（唯一计算点——快道/needs_bounds/
    // live_probe 保留决策与建_probe 门一致）。
    let dirty = *state.app.view_dirty.borrow();
    // PLAN-725 T-00：帧脏标记入分段探针（present 行 dirty= 字段）。
    crate::ui::frame_segments::note_dirty(dirty);
    let mcp_active = !p530_nomcp && state.desktop.mcp_shared.is_some();
    let mcp_wants_live = mcp_active
        && (dirty
            || state
                .desktop
                .mcp_shared
                .as_ref()
                .map(|m| m.lock().unwrap().mcp_active_recently(30))
                .unwrap_or(false));
    let capture_debug = state.app.devtools.debug_mode || mcp_wants_live;
    let mut frame_build: Option<(
        crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
        crate::ui::debug_id_map::DebugIdMap,
        crate::ui::debug::BuildProbe,
    )> = None;
    if dirty {
        let p725_t_build = std::time::Instant::now();
        // Plan 307 Task 18: gate the probe by debug_mode. When F12 is off the
        // probe is disabled (all record_* no-ops → zero overhead), and
        // live_probe is set to None so the inspector UI degrades to placeholders.
        let (view, id_map, probe) = state.component.view_with_debug_gated(capture_debug);
        if capture_debug {
            *state.app.live_probe.borrow_mut() = Some(probe.clone());
        } else {
            *state.app.live_probe.borrow_mut() = None;
        }
        crate::ui::frame_segments::note_s3b_build(p725_t_build.elapsed());
        frame_build = Some((view, id_map, probe));
    }

    // PLAN-725 T-02：编辑器 on_change 载荷静态消费判定的绑定表面随帧下发
    //（render_dynamic_view 无 state；小 map 克隆 ≪ 一次 O(doc) 全文物化）。
    P725_INPUT_BINDINGS.with(|b| {
        *b.borrow_mut() = Some(state.component.input_state_map().clone());
    });

    // Sync state to MCP shared handle for AI agent inspection (Plan 278)
    // Must run in view() — not update() — because iced may not fire any events
    // initially, meaning update() might never run before an MCP client connects.
    if sync_mcp && !p530_nomcp {
        if let Some(ref mcp_handle) = state.desktop.mcp_shared {
            let mut mcp = mcp_handle.lock().unwrap();
            // PLAN-062 Phase2 T11：MCP 同步块此前**每帧**全量模板重建
            // （view_with_debug_gated + vtree 快照 + read_all_state）——三个
            // 500ms 消息泵（__hot_reload/PollStream/__bounds_collected）每条
            // 消息都过 view()，实机 ~6 重建/s 定罪（AUTO_DEBUG_KEYS 计数 ×
            // 每重建 446-U7 WARN 3.4 条 = 实测 855/45s 吻合），叠加 retain
            // 泄漏即空闲斜率主驱动。快照语义只需与视图同步：仅当视图真变
            // （view_dirty——boot 首帧恒 true）、窗口尺寸变化、或尚未同步过
            // 时才重建；静止视图零重建，快照仍然准确（视图未变）。update
            // 间隙的异步脏写经 hot_reload 500ms 泵桥接（12095），MCP 快照
            // 至多 500ms 陈旧——与既有异步语义一致。
            let ws_now = *state.window_size.borrow();
            let gate_dirty = *state.app.view_dirty.borrow();
            let gate_ws = ws_now.width > 0.0
                && *state.app.mcp_synced_ws.borrow() != (ws_now.width, ws_now.height);
            if !gate_dirty && !gate_ws && mcp.has_view() {
                drop(mcp);
            } else {
                // PLAN-725 T-00：S3a 打点起点（MCP 同步块第一遍全量建——块体）。
                let p725_t_s3a = std::time::Instant::now();
                if !mcp.has_view() {
                    eprintln!("AutoUI MCP: first state sync in view()");
                }
                // Plan 370 D-GAP-4: materialize VmRef list fields (e.g. store.notes) to
                // inline Value::Array so the MCP snapshot/inspect tools can expand
                // `for` loops and evaluate `.len()` without VM heap access.
                // PLAN-633: 状态采集移到本帧视图构建**之后**——挂载期子件 Init
                // （画廊内嵌全栈 demo：Demo*.Init → store → #[api] → db 种子）在
                // view 构建中派发并写根态，先采后建会把 Init 写入漏在本帧快照外，
                // 且 view_dirty 门控使其后无再同步帧 → MCP 快照永久滞留挂载前空值
                // （013 todos=[] 实证；白盒直读同刻为 4 条）。
                // PLAN-682 F1a：同步块自带属性面。此前探针被丢弃（_probe）且
                // computed 恒写空 map，styled_vtree 的属性面（style/onclick 行）
                // 只能靠 __bounds_collected 回路的 from_live 覆盖——该回路仅脏帧
                // 运行，纯 resize 等非脏帧冻结为空属性态 ⇒ 快照投影双态（jade
                // D-18 / auto-edit F-RV6 家族，字节基线间歇漂移）。此处接住探针，
                // 按与 live_cache 合并（Plan 309 Phase 2b / Plan 371 Task 8）同构
                // 的 path→VNodeId 派生填充 raw_class/events；bounds 等测量字段仍
                // 由 bounds 回路回填（ComputedNodeLite 全 Option，缺失即省略）。
                // 代价：本块原本就在重建视图，门 false → true 只是把既有构建的
                // 探针记录打开（record_* 为纯内存写入，无额外遍历）。
                // PLAN-725 T-01：单帧单建——脏帧复用主重建产物（提升点已建，
                // 此处零模板走查）；非脏同步帧（gate_ws/首同步）无提升产物，就地
                // 自建维持原语义（probe=true）。复用形态：vtree 与 mcp.update 各
                // 取一克隆，主重建段消费原件（AbstractView 克隆 O(widgets) ≪ 模板
                // 重解释——T-00 实测 s3a 5.2ms vs 全建 3.3ms 的减量即此）。
                let mut p725_sync_owned: Option<(
                    crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
                    crate::ui::debug_id_map::DebugIdMap,
                    crate::ui::debug::BuildProbe,
                )> = None;
                let (view, id_map, sync_probe): (
                    &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
                    &crate::ui::debug_id_map::DebugIdMap,
                    &crate::ui::debug::BuildProbe,
                ) = match frame_build.as_ref() {
                    Some((v, im, p)) => (v, im, p),
                    None => {
                        p725_sync_owned = Some(state.component.view_with_debug_gated(true));
                        let (v, im, p) = p725_sync_owned.as_ref().expect("just set");
                        (v, im, p)
                    }
                };
                let mut computed: HashMap<
                    crate::ui::vnode::VNodeId,
                    crate::ui::mcp_server::ComputedNodeLite,
                > = HashMap::new();
                for (path_u16, entry) in sync_probe.snapshot() {
                    let vid =
                        crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(path_u16));
                    let node = computed.entry(vid).or_default();
                    if entry.raw_class.is_some() {
                        node.raw_class = entry.raw_class.clone();
                    }
                    if !entry.events.is_empty() {
                        // ComputedNodeLite.events 是 (event, handler) 元组面——
                        // 探针的 EventHandlerInfo 经 from_computed 同款映射转换。
                        node.events = entry
                            .events
                            .iter()
                            .map(|e| (e.event.clone(), e.handler.clone()))
                            .collect();
                    }
                    // PLAN-088 T-02B: for 循环上下文随探针入快照（for_iter prop 的
                    // 序列化臂 vtree_atom 既有；此前仅 raw_class/events 入站，实例
                    // 标注到不了 MCP 消费面）。
                    if let Some(f) = &entry.for_context {
                        node.for_context = Some((f.var.clone(), f.index, f.value_repr.clone()));
                    }
                }
                let state_vals = state.component.read_all_state_materialized();
                let input_map = state.component.input_state_map().clone();
                let view_template = Some(state.component.view_template().clone());
                // Plan 446 批一(J1 诊断层根因修复): styled_vtree 此前仅在
                // __bounds_collected 回路后设置,而 bounds 只在 view() 脏重建时请求
                // ——boot 后无重建则 styled 永不落盘,快照首问必走源树回退(子 widget
                // 裸引用/循环未展开 = os-config C1/J1 现场"空壳")。这里随每帧 MCP
                // 同步直接推送 vtree 快照;bounds/计算样式注释仍由 6403 回路后补。
                {
                    let span_map = state.component.span_map().clone();
                    let vtree = crate::ui::vnode_converter::view_to_vtree_with_paths(
                        view.clone(),
                        |path: &[u16]| {
                            let p: Vec<usize> = path.iter().map(|&x| x as usize).collect();
                            id_map
                                .get(&p)
                                .and_then(|aura_id| span_map.get(&aura_id))
                                .and_then(|info| info.span)
                                .map(|(offset, len)| crate::ui::debug::SourceSpan { offset, len })
                        },
                    );
                    mcp.set_styled_vtree(crate::ui::mcp_server::StyledNodeSnapshot {
                        widget_name: state.component.widget_name().to_string(),
                        vtree: vtree.clone(),
                        computed,
                    });
                    // Plan 483 D4：缓存这份与 shared.view 同源的 vtree，供
                    // `__bounds_collected` 回路覆盖 styled_vtree 时取用（见
                    // AppState::mcp_sync_vtree 文档注释——禁止改用 live_vtree 源）。
                    *state.app.mcp_sync_vtree.borrow_mut() = Some(vtree);
                }
                // PLAN-725 T-01：mcp.update 消费克隆（主重建段消费原件）。
                mcp.update(
                    view.clone(),
                    id_map.clone(),
                    state_vals,
                    input_map,
                    view_template,
                    state.component.key_bindings().clone(),
                );
                // PLAN-646: 源码全文随帧发布——`autoui_select_rect` 信封切片用
                //（ensure 幂等：装载过零开销；未装载过此处读盘一次）。
                ensure_source_loaded(state);
                if let Some(ref code) = *state.app.source_code.borrow() {
                    mcp.set_source_code(code.clone());
                }
                // Sync window size for layout annotations (Plan 281)
                let ws = state.window_size.borrow();
                let iced::Size { width, height } = *ws;
                if width > 0.0 && height > 0.0 {
                    mcp.set_window_size(width, height);
                    *state.app.mcp_synced_ws.borrow_mut() = (width, height);
                }
                // PLAN-725 T-00：S3a MCP 同步块计时发布（vtree 转换+read_all_state_
                // materialized+mcp.update 整段；builds 仅在就地自建臂计——T-01 复用
                // 臂的建已由 s3b 提升点计过）。
                if p725_sync_owned.is_some() {
                    crate::ui::frame_segments::note_s3a_mcp(p725_t_s3a.elapsed());
                } else {
                    crate::ui::frame_segments::note_s3a_mcp_nobuild(p725_t_s3a.elapsed());
                }
            } // PLAN-062 T11 gate_dirty/gate_ws 门控结束
        }
    } // sync_mcp 门控（459：仅 primary App 视图执行 MCP 同步）

    // Plan 409 §10 续 11: 同步窗口宽度,供 VM builder 响应式布局(grid 列数)。
    crate::ui::style_iced_adapter::set_window_width(state.window_size.borrow().width);
    // PLAN-020 T-00b: 高度随帧同步(window_size = 逻辑 px;分屏矩形投影
    // 窗口尺寸面的消费源,与 width 同点同规约)。
    crate::ui::style_iced_adapter::set_window_height(state.window_size.borrow().height);
    // PLAN-028 T-02:窗口尺寸真值发布(分体轨桥;变化才发布——每帧两次
    // f32 比较,发布仅随真实变化,稳态零写入)。消费面:同进程 .at
    // storage.get 直读;分体轨后端经共享 storage 文件按拍拉取
    // (AUTO_VM_STORAGE_FILE 钉扎,runbook 惯例)。__window_resized 事件臂
    // 的 vm.window_inner_height 发布(Plan 046-B)语义保留。
    {
        static PUBLISHED: std::sync::OnceLock<std::sync::Mutex<Option<(f32, f32)>>> =
            std::sync::OnceLock::new();
        let (sw, sh) = {
            let sz = state.window_size.borrow();
            (sz.width, sz.height)
        };
        let mut last = PUBLISHED
            .get_or_init(|| std::sync::Mutex::new(None))
            .lock()
            .unwrap();
        let changed = match *last {
            Some((lw, lh)) => (lw - sw).abs() >= 0.5 || (lh - sh).abs() >= 0.5,
            None => true,
        };
        if changed {
            *last = Some((sw, sh));
            auto_lang_core::vm::ffi::stdlib::storage_host_publish(
                "vm.window_inner_width",
                format!("{:.0}", sw),
            );
            auto_lang_core::vm::ffi::stdlib::storage_host_publish(
                "vm.window_inner_height",
                format!("{:.0}", sh),
            );
        }
    }
    // PLAN-530 步骤2 表面追踪：view 重建时的宽度信号轨迹。
    if std::env::var("P530_TRACE").as_deref() == Ok("1") {
        let sz = state.window_size.borrow();
        eprintln!(
            "[P530-TRACE] view rebuild: dirty={} window={:.0}x{:.0} app_id={}",
            *state.app.view_dirty.borrow(),
            sz.width,
            sz.height,
            state.app_id.0
        );
    }

    // Resolve pending hover messages: pick the smallest counter (= deepest element).
    // This handles the case where nested mouse_areas both fire on_move — child has
    // smaller counter, so it wins. When mouse leaves child, only parent fires on_move,
    // so parent becomes the new deepest candidate.
    {
        let mut pending = state.app.devtools.pending_hovers.borrow_mut();
        if !pending.is_empty() {
            if let Some(best) = pending.iter().min_by_key(|(c, _)| *c) {
                *state.app.devtools.hovered_widget.borrow_mut() = Some(best.1.clone());
            }
            pending.clear();
        }
    }

    // === View rendering with Element cache ===
    //
    // iced calls view(&self) after each update() and requires owned Element return.
    // Element doesn't impl Clone, so we use take() to return the cached instance.
    //
    // When view_dirty=true: rebuild AbstractView from template → render Element → cache it → take and return.
    // When view_dirty=false: if cache exists, take and return it directly (preserves button hover/press state).
    //                         if cache is empty (shouldn't happen normally), rebuild from cached AbstractView.

    // PLAN-725 T-01：dirty 读点已随单建提升（dark 同步后——主题翻转帧本帧
    // 即脏）；frame_build 同源。

    // Fast path: return cached Element when nothing changed.
    if !dirty {
        if let Some(el) = state.app.cached_rendered.borrow_mut().take() {
            // PLAN-731 T-00: 防御快道同样过 S5 根探针（路径覆盖一致）。
            return crate::ui::iced::frame_probe::FrameProbe::new(
                crate::ui::iced::right_press_area::PointerPressArea::new(el),
            )
            .into();
        }
        // Cache empty — fall through to rebuild (uses cached AbstractView if available)
    }

    // Sync console buffer → console_output for DevTools Console tab.
    // Plan 459：缓冲条目带 AppId 标签（0=进程级），按游标增量排空——各 App
    // 只收自己的 print/console_log 行 + 进程级行（boot/shell），双窗日志
    // 互不串扰；游标同时消除旧全量拷贝路径的重复累积。
    {
        let mut buf = state.app.devtools.console_buffer.lock().unwrap();
        let start = state.app.devtools.console_drained.get().min(buf.len());
        let fresh: Vec<String> = buf[start..]
            .iter()
            .filter(|(tag, _)| *tag == 0 || *tag == state.app_id.0)
            .map(|(_, line)| line.clone())
            .collect();
        state.app.devtools.console_drained.set(buf.len());
        drop(buf);
        if !fresh.is_empty() {
            state.app.devtools.console_output.borrow_mut().extend(fresh);
        }
    }

    // Plan 314 Task 4: decouple *data* capture from the F12 visual overlay.
    // `capture_debug` gates the live VTree/probe/inspector-cache population so an
    // MCP client gets the full runtime VTree + box model + computed style WITHOUT
    // opening F12. Visual overlays (hover/selected highlight, inspect mouse_area)
    // remain gated on `debug_mode` alone (see `wrap_debug`'s `if !self.debug_mode`
    // early-return), so MCP-only capture never perturbs the rendered layout.
    // PLAN-650 E-2/E-4：capture 以 F12 为主；MCP 仅在 **dirty 重建帧** 追加
    //（静止 fall-through 帧不再为 MCP 付 live_vtree/needs_bounds——快照走
    // 既有 gate_dirty/gate_ws 同步，视图未变时仍然准确）。
    // PLAN-725 T-01：dirty/mcp_active/mcp_wants_live/capture_debug 已随单建
    // 提升点计算（快道前唯一计算点），此处直用。

    // Plan 314 Task 4: request a layout-bounds collection this frame whenever we
    // are capturing DevTools/MCP data. `update()` checks `needs_bounds` at its
    // end and, when true, runs the LayoutCollector → `__bounds_collected` →
    // `backfill_bounds` + `set_layout_bounds` chain — the ONLY writer of the
    // measured `ComputedNode.bounds` (the `bbox`) and `layout_bounds`. This
    // view() body is reached only on a rebuild (steady state returns the cached
    // Element at the fast path above), so it bounds the round-trips to ~one per
    // changed frame, not every frame. Gated on `capture_debug` so ordinary
    // non-debug/non-MCP runs pay zero bounds-collection overhead.
    // PLAN-650 E-2：非 dirty 帧不请求 bounds（结构未变，沿用上次测量）。
    if capture_debug && dirty {
        *state.app.devtools.needs_bounds.borrow_mut() = true;
    }

    // PLAN-631 T-01：剖析计时——builder（模板→AbstractView 转换）与
    // render（AbstractView→iced Element）两段耗时随 `[P631-PROFILE]` 行
    // 逐重建帧吐出（P631_PROFILE=1 启用；Style::parse 计数见 ui::style::profile）。
    let p631_profile = crate::ui::style::profile::enabled();
    let p631_t_builder = p631_profile.then(std::time::Instant::now);
    // PLAN-725 T-00：S3b 打点起点（脏帧主建 / 非脏 fall-through 克隆）。
    let p725_t_s3b = std::time::Instant::now();
    let (converted, debug_id_map) = if dirty {
        // PLAN-725 T-01：单帧单建——消费提升点的构建产物（view/id_map/
        // probe；live_probe 保留决策已随提升点落位），此处只做注入/转换/
        // 缓存写回。probe=true 形态的 MCP 快照消费已由同步块克隆完成。
        let (mut view, debug_id_map, _probe) = frame_build
            .take()
            .expect("PLAN-725 T-01: dirty frame must carry the hoisted build");
        let debug_id_map = Some(debug_id_map);
        let p725_t_tail = std::time::Instant::now();
        inject_todo_list(&mut view, &state.app.todos, state.component.widget_name());
        if !state.app.input_values.is_empty() {
            patch_input_values(&mut view, &state.app.input_values);
        }
        // PLAN-002 N5：vwin 客户区根节点默认底角圆角（convert 前作用于源
        // 树；缓存帧继承已合并结果，app 与 flag 恒定配对无漂移）。
        if round_bottom {
            round_bottom_root_default(&mut view);
        }
        let converted = convert_view_messages(view);
        *state.app.cached_converted_view.borrow_mut() = Some(converted.clone());
        *state.app.devtools.cached_debug_id_map.borrow_mut() = debug_id_map.clone();
        // PLAN-062 F2: 新缓存已写回——换代提交帧账本（释放上一脏帧经
        // retain_heap_result 拿下的宿主份额；旧缓存树同帧淘汰，无悬挂）。
        state.component.commit_dirty_frame();
        // PLAN-725 T-00：S3b 尾段（注入/转换/缓存写回——建本体已在提升点
        // 计入 note_s3b_build）。
        crate::ui::frame_segments::note_s3b_clone(p725_t_tail.elapsed());
        (converted, debug_id_map)
    } else {
        // Cache miss on non-dirty frame: rebuild from cached AbstractView (cheaper than template rebuild)
        let cached = state.app.cached_converted_view.borrow();
        if let Some(ref converted) = *cached {
            // `live_probe` is intentionally NOT refreshed here: the probe is
            // template-derived and stable across cache hits, so the retained
            // probe from the last dirty rebuild remains valid.
            let debug_id_map = state.app.devtools.cached_debug_id_map.borrow().clone();
            // PLAN-725 T-00：S3b fall-through 克隆臂（深克隆 cached_converted_
            // view——无 view_with_debug_gated 调用，不计 builds）。
            let out = (converted.clone(), debug_id_map);
            drop(cached);
            crate::ui::frame_segments::note_s3b_clone(p725_t_s3b.elapsed());
            out
        } else {
            drop(cached);
            // Plan 307 Task 18: gate the probe by debug_mode (same as the dirty
            // branch above). When F12 off the probe is disabled + live_probe None.
            let (mut view, debug_id_map, probe) =
                state.component.view_with_debug_gated(capture_debug);
            let debug_id_map = Some(debug_id_map);
            if capture_debug {
                *state.app.live_probe.borrow_mut() = Some(probe);
            } else {
                *state.app.live_probe.borrow_mut() = None;
            }
            inject_todo_list(&mut view, &state.app.todos, state.component.widget_name());
            if !state.app.input_values.is_empty() {
                patch_input_values(&mut view, &state.app.input_values);
            }
            let converted = convert_view_messages(view);
            *state.app.cached_converted_view.borrow_mut() = Some(converted.clone());
            *state.app.devtools.cached_debug_id_map.borrow_mut() = debug_id_map.clone();
            // PLAN-062 F2: 首帧构建同契约换代提交（与 dirty 分支一致）。
            state.component.commit_dirty_frame();
            // PLAN-725 T-00：S3b 首帧/缓存击穿重建臂（builds 计数）。
            crate::ui::frame_segments::note_s3b_build(p725_t_s3b.elapsed());
            (converted, debug_id_map)
        }
    };
    let p631_builder_us = p631_t_builder
        .as_ref()
        .map(|t0| t0.elapsed().as_micros() as u64);

    // Plan 307 Task 5: build a live VTree once per frame for the DevTools inspector.
    // `converted` is the exact View<IcedMessage> tree about to be rendered. Built
    // here (before `converted` is moved into render_dynamic_view and before
    // `debug_id_map` is moved into debug_ctx) as a side-effect snapshot only.
    // PLAN-650 E-2/E-4：仅 debug_mode 或 dirty 重建帧构建；静止 fall-through
    // 非 F12 帧跳过（MCP 快照仍由 gate_dirty 同步块提供，视图未变即准确）。
    if !p530_nomcp && (state.app.devtools.debug_mode || dirty) {
        if let Some(id_map) = &debug_id_map {
            let span_map = state.component.span_map().clone();
            let vtree = crate::ui::vnode_converter::view_to_vtree_with_paths(
                converted.clone(),
                |path: &[u16]| {
                    let p: Vec<usize> = path.iter().map(|&x| x as usize).collect();
                    id_map
                        .get(&p)
                        .and_then(|aura_id| span_map.get(&aura_id))
                        .and_then(|info| info.span)
                        .map(|(offset, len)| crate::ui::debug::SourceSpan { offset, len })
                },
            );
            *state.app.live_vtree.borrow_mut() = Some(vtree);
        } else {
            *state.app.live_vtree.borrow_mut() = None;
            *state.app.live_probe.borrow_mut() = None;
            *state.app.live_cache.borrow_mut() = None;
        }
    }

    // Clear view_dirty after consuming the change.
    // Do this BEFORE rendering so that subscriptions/events arriving during
    // render processing don't get missed.
    *state.app.view_dirty.borrow_mut() = false;

    let debug_ctx = if let Some(id_map) = debug_id_map {
        let span_map = state.component.span_map().clone();
        Some(DebugRenderCtx {
            hovered_id: state.app.devtools.hovered_widget.borrow().clone(),
            selected_id: state.app.devtools.selected_widget.borrow().clone(),
            wrapper_counter: std::cell::RefCell::new(0),
            span_map,
            debug_id_map: id_map,
            id_to_aura: std::cell::RefCell::new(std::collections::HashMap::new()),
            aura_to_id: std::cell::RefCell::new(std::collections::HashMap::new()),
            element_styles: std::cell::RefCell::new(std::collections::HashMap::new()),
            tree_stack: std::cell::RefCell::new(Vec::new()),
            component_tree: std::cell::RefCell::new(None),
            debug_mode: state.app.devtools.debug_mode,
            capture_data: capture_debug,
            inspector_cache: std::cell::RefCell::new(crate::ui::debug::InspectorCache::new()),
        })
    } else {
        None
    };

    // Plan 483: 脏重建时清填 input Id 登记(DFS 序同渲染侧;缓存帧沿用
    // 上次登记——结构未变)。
    // PLAN-650 E-2：仅 dirty 帧刷新；非 dirty fall-through 沿用上次（结构未变）。
    if dirty {
        let mut ids = Vec::new();
        collect_input_ids(&converted, &mut ids);
        *state.app.devtools.input_ids.borrow_mut() = ids;
    }

    let mut path = Vec::new();
    let p631_t_render = p631_profile.then(std::time::Instant::now);
    // PLAN-725 T-00：S4 Element 段打点（render_dynamic_view 全树新建）。
    let p725_t_s4 = std::time::Instant::now();
    let rendered = render_dynamic_view(converted, debug_ctx.as_ref(), &mut path);
    crate::ui::frame_segments::note_s4_element(p725_t_s4.elapsed());
    // PLAN-725 T-00：臂累积表随 s4 发布（勘定钻取面）。
    crate::ui::frame_segments::flush_arms();
    if p631_profile {
        let (calls, nanos) = crate::ui::style::profile::take();
        let builder_ms = p631_builder_us.unwrap_or(0) as f64 / 1000.0;
        let render_ms = p631_t_render
            .map(|t0| t0.elapsed().as_micros() as u64)
            .unwrap_or(0) as f64
            / 1000.0;
        eprintln!(
            "[P631-PROFILE] rebuild builder_ms={:.2} render_ms={:.2} style_parse_calls={} style_parse_ms={:.2}",
            builder_ms,
            render_ms,
            calls,
            nanos as f64 / 1_000_000.0
        );
    }

    // Plan 412 续(toast 修正 3):toast 的消费/入队/到期 Task 都在 update
    // (&mut)完成;dynamic_view 只按 DynamicState.toasts 渲染恒定双层
    // Stack —— 槽 0 = 主内容,槽 1 = toast 层(无 toast 时为零尺寸空层)。
    // 根节点结构恒定,iced 内部 widget-tree 的 diff 得以保留 scrollable
    // 滚动位置等交互状态;toast 层不设 opaque —— 命中测试穿透,不夺焦点、
    // 不拦截主界面交互,纯悬浮展示。
    let toasts = state.desktop.toasts.borrow();
    let toast_el: iced::Element<'static, IcedMessage> = if toasts.is_empty() {
        iced::widget::container(iced::widget::Space::new()).into()
    } else {
        build_toast_layer(&toasts)
    };
    // Plan 412 续(toast 修正 4):Stack 默认 Shrink×Shrink —— 布局时 base 层
    // (主内容)按收缩限制排版,整窗随内容收缩、悬浮层 Fill 落在收缩盒里,
    // toast 卡片全部挤出可视区(iced 文档:子节点用 Fill 策略时必须显式给
    // Stack 设宽高)。显式 Fill×Fill:主内容恢复整窗排版,toast 层九宫格
    // 锚点以整窗为参照。
    // Plan 504：fit 待测量期间整链 Shrink —— Fill 会把测量钉回窗口尺寸。
    // Plan 512：fit 窗常驻 Shrink + 测量锚点（fit_enabled）——动态重测
    // 才能量到内容自然尺寸（首测后回 Fill 会把重测钉回窗口尺寸）。
    let fit_pending = state.fit_pending.get() || state.fit_enabled.get();
    let mut stack = iced::widget::Stack::new().push(rendered).push(toast_el);
    // PLAN-646: marquee 拖拽中注入全窗蒙层（canvas 纯绘制；非 opaque 层
    // 命中测试穿透——toast 层同款，不夺焦点不拦截交互）。零尺寸不注入。
    if let Some(m) = *state.app.devtools.marquee.borrow() {
        let r = m.rect();
        if r.width > 0.5 || r.height > 0.5 {
            stack = stack.push(
                iced::widget::canvas(MarqueePainter { rect: r })
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill),
            );
        }
    }
    let rendered: iced::Element<'static, IcedMessage> = if fit_pending {
        stack
            // P679-D1：宽 = Fill（同锚点——Shrink 宽会零化 Fill 后代），
            // 高 = Shrink（自然高测量；toast Fill 高在 fit 窗塌 0 = 登记
            // 边界：fit 窗内 toast 不显示）。
            .width(iced::Length::Fill)
            .height(iced::Length::Shrink)
            .into()
    } else {
        stack
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    };

    // Copy element style metadata and component tree from DebugRenderCtx to DynamicState
    if let Some(ref ctx) = debug_ctx {
        let styles = ctx.element_styles.borrow();
        *state.app.devtools.debug_element_styles.borrow_mut() = styles.clone();
        let tree = ctx.component_tree.borrow();
        *state.app.devtools.component_tree.borrow_mut() = tree.clone();
        // Cache aura_to_id mapping for source-click → component-highlight reverse lookup
        let aura_map = ctx.aura_to_id.borrow();
        *state.app.aura_to_id_cache.borrow_mut() = aura_map.clone();
        // Plan 307 Task 12: copy the `VNodeId <-> iced widget id` map into
        // DynamicState for later bounds backfill (Task 13) and inspector panels
        // (tasks 15-16). InspectorCache derives Clone.
        let mut cache = ctx.inspector_cache.borrow().clone();

        // Plan 307 Task 17: derive `hovered_vnode` from the aura_N hover string
        // using the freshly-built per-frame cache (VNodeId <-> iced id map).
        // `hovered_widget` was resolved from pending_hovers earlier in this same
        // view() pass. Mirror it into hovered_vnode so the left-tree hover tint
        // (Task 14, keyed on VNodeId) tracks the same node the overlay highlights.
        // When hovered_widget is None the cursor left all widgets → clear it.
        // Done before moving `cache` into live_cache.
        let hovered_aura = state.app.devtools.hovered_widget.borrow().clone();
        let new_hovered_vnode = hovered_aura.as_deref().and_then(|s| cache.iced_to_vnode(s));
        *state.app.devtools.hovered_vnode.borrow_mut() = new_hovered_vnode;

        // Plan 309 Phase 2b: merge `raw_class` from `live_probe` into the cache
        // by path → VNodeId, so the Computed tab (which reads the cache) can
        // render the declared class string. The probe is keyed by the SAME
        // build path the VTree flattens to (Plan 309 Phase 1 Fix A reconciled
        // the ForLoop single-body case), so `id_from_path` resolves to the
        // VNodeId the Computed tab selects by. Probe entries without a class
        // are skipped (no-op record_raw_class never created them).
        //
        // Plan 371 Task 8: ALSO merge `events` so autoui_vtree / autoui_find
        // can show handler info per node (previously events were always empty
        // in the live snapshot).
        if let Some(probe) = state.app.live_probe.borrow().as_ref() {
            for (path_u16, entry) in probe.snapshot() {
                let vid = crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(path_u16));
                let node = cache.get_mut_or_default(vid);
                if entry.raw_class.is_some() {
                    node.raw_class = entry.raw_class.clone();
                }
                if !entry.events.is_empty() {
                    node.events = entry.events.clone();
                }
                // PLAN-088 T-02B: ForIter 同站合并（缺此则 __bounds_collected
                // 回路的 from_live 覆盖会把同步块已并入的 for_context 抹掉）。
                if entry.for_context.is_some() {
                    node.for_context = entry.for_context.clone();
                }
            }
        }

        // Plan 307 Task 18 / Plan 314 Task 4: retain the per-frame cache when
        // F12/debug is on OR an MCP client may read it (`capture_debug`). When
        // neither, drop it to None so no inspector data lingers and the panels
        // degrade to placeholders. (The cache object is always constructed above
        // — this just gates whether it is retained.)
        if capture_debug {
            *state.app.live_cache.borrow_mut() = Some(cache);
        } else {
            *state.app.live_cache.borrow_mut() = None;
        }
    }

    let result: iced::Element<'static, IcedMessage> = if state.app.devtools.debug_mode {
        if *state.app.devtools.devtools_open.borrow() {
            // Row layout: [main content] [draggable divider] [DevTools panel]
            let panel = render_devtools_panel(state);
            let is_dragging = *state.app.devtools.dragging_divider.borrow();
            let divider_bg = if is_dragging {
                iced::Color::from_rgb(0.3, 0.5, 0.9) // blue while dragging
            } else {
                iced::Color::from_rgb(0.82, 0.82, 0.82) // gray normally
            };
            let divider = mouse_area(
                container(iced::widget::Space::new().width(6))
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(divider_bg)),
                        ..Default::default()
                    })
                    .width(6)
                    .height(iced::Length::Fill),
            )
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__divider_press".to_string(),
                input_value: None,
            });
            let layout = row![rendered, divider, panel]
                .width(iced::Length::Fill)
                .height(iced::Length::Fill);
            container(layout)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        } else {
            fit_aware_root(rendered, state.app_id, fit_pending)
        }
    } else {
        fit_aware_root(rendered, state.app_id, fit_pending)
    };

    // Pick up pending screenshot request from MCP thread (Plan 285).
    // Plan 462：仅 primary（sync_mcp 门控，T8 单 App 语义）—— desktop 模式
    // 下非 primary App 的 view 此前会偷走共享请求，而它们几乎收不到
    // update，请求滞留至 MCP 工具超时（实测复现于 ui_desktop 双窗 demo）。
    if sync_mcp {
        if let Some(ref mcp_handle) = state.desktop.mcp_shared {
            if let Some(req) = mcp_handle.lock().unwrap().take_screenshot_request() {
                *state.app.devtools.screenshot_request.borrow_mut() = Some(req);
            }
        }
    }

    // PLAN-631 F-7: 指针按下记忆根包装——placement "pointer" 的锚源(窗口
    // 根单包装,纯委托,ButtonPressed 事件现场记账;见 ui::iced::right_press_area)。
    let result: iced::Element<'static, IcedMessage> =
        crate::ui::iced::right_press_area::PointerPressArea::new(result).into();

    // PLAN-731 T-00: S5 分段根包装探针（最外层——括号覆盖含指针包装在内的
    // 全树 layout/draw；纯委托零行为差，门关零开销）。
    let result: iced::Element<'static, IcedMessage> =
        crate::ui::iced::frame_probe::FrameProbe::new(result).into();

    // PLAN-725 T-04① 勘定：put-then-take 死写移除。iced view() 契约要求
    // owned Element 返回——原「写缓存后立即 take 返回」使缓存入口态恒空，
    // 上方快道从不命中（:18005 旧注释「Element 缓存 take 空后的
    // fall-through 帧」早已承认）。非脏帧真实路径=cached_converted_view
    // 克隆+全树 Element 重建（fall-through 臂，孤儿帧行可见其成本）。
    // 快道保留为防御面（未来若引入真缓存写者即激活）；本帧产物直接返回。
    result
}

/// Plan 504：App 窗口根容器。常态 Fill×Fill；`fit_pending`（pac.at
/// `window: "fit"`，见 [`startup_window_fit`] / `LaunchSpec.fit`）期间
/// Shrink×Shrink 并挂 `aura_fit_root_<appid>` Id —— LayoutCollector 首帧
/// 经此锚点量出内容自然尺寸，`__fit_measured` 路由据此收缩窗口。
/// Plan 512：fit 窗**常驻** Shrink + 锚点（fit_pending || fit_enabled）——
/// 首测后回 Fill 会把动态重测钉回窗口尺寸；用户锁定放大窗时内容左上角
/// 对齐、余量露窗口底色（v1 语义，待澄清③重解锁为增强候选）。
/// Plan 512 S3 实证修正：锚点外套一层 vertical scrollable —— 活树布局受
/// 当前窗口钳制（内容自然高超出窗口即被裁到窗口高，增长方向永远量不到）；
/// scrollable 内部以无限高约束排版内容，锚点（挂在内容侧）由此量到真实
/// 自然尺寸。副作用：内容超出窗口的瞬态（重测 lag / 用户锁定缩小窗）出
/// 滚动条而非裁剪，语义可接受。宽度方向受滚动视口钳制（v1 仅高度方向
/// 可增长量测，见 plan 512 待澄清）。
fn fit_aware_root(
    content: iced::Element<'static, IcedMessage>,
    app_id: crate::ui::session::AppId,
    fit_active: bool,
) -> iced::Element<'static, IcedMessage> {
    if fit_active {
        // PLAN-002 N4（用户复核裁定 2026-09-09）：内容在放大窗内居中——
        // 最大化/手动拉大的 fit 窗内容不再左上角沉底（512 v1 语义升级，
        // 待澄清③就此定案）。
        // PLAN-040 F1（2026-09-22 用户裁定：fit 窗宽度也随内容）：滚动区
        // 改 Both 双向 + 隐藏滚动条，锚点回 Shrink×Shrink——双轴无界约束
        // 下量到内容**自然宽 × 自然高**（P679-D1 的宽度 Fill 视口钳制使
        // Fill 根 app 恒量到窗口宽、fit 宽度方向永不生效，calculator 停在
        // 60% viewport 实录）。003 的 Shrink 宽链塌 0 发生在**有界**视口
        // （Fill 后代解析 0）；无界约束下 Fill 后代解析为内在尺寸，与 512
        // S3 高度方向同机理。
        container(
            scrollable(
                container(content)
                    .width(iced::Length::Shrink)
                    .height(iced::Length::Shrink)
                    .id(iced::widget::Id::from(format!(
                        "aura_fit_root_{}",
                        app_id.0
                    ))),
            )
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::hidden(),
                horizontal: scrollable::Scrollbar::hidden(),
            })
            .width(iced::Length::Fill)
            .height(iced::Length::Fill),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center_x(iced::Length::Fill)
        .center_y(iced::Length::Fill)
        .into()
    } else {
        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

